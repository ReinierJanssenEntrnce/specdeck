//! Replays hook payloads recorded from real Claude Code sessions through the timeline builder.

use std::path::Path;

use serde_json::Value;
use specdeck_core::hooks::{HOOK_EVENTS, HookPayload};
use specdeck_core::timeline::TimelineBuilder;
use specdeck_protocol::timeline::{DiffHunk, Entry, FileChangeKind, ToolStatus};

/// The payloads of one recorded session, in order, without events the engine does not register.
fn payloads(session: &str) -> Vec<HookPayload> {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude-hooks").join(session);
    let mut files: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    files
        .into_iter()
        .map(|file| match serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap() {
            Value::Object(payload) => payload,
            other => panic!("not an object: {other}"),
        })
        .filter(|payload| HOOK_EVENTS.contains(&payload["hook_event_name"].as_str().unwrap()))
        .collect()
}

fn replay(session: &str) -> Vec<Entry> {
    let mut builder = TimelineBuilder::new("/workspace/project");
    payloads(session).into_iter().flat_map(|p| builder.handle(p).unwrap()).collect()
}

fn kind(entry: &Entry) -> String {
    serde_json::to_value(entry).unwrap()["kind"].as_str().unwrap().to_owned()
}

#[test]
fn edit_session() {
    let entries = replay("edit-session");
    assert_eq!(
        entries.iter().map(kind).collect::<Vec<_>>(),
        [
            "prompt",
            "tool-call",
            "tool-result",
            "file-change",
            "tool-call",
            "tool-result",
            "file-change",
            "tool-call",
            "tool-result",
            "response",
        ]
    );

    let Entry::FileChange { path, change, hunks, .. } = &entries[3] else { panic!() };
    assert_eq!(path, "notes.txt");
    assert_eq!(*change, FileChangeKind::Created);
    assert_eq!(
        hunks,
        &[DiffHunk {
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: 1,
            lines: vec!["+one".into()]
        }]
    );

    let Entry::FileChange { change, hunks, .. } = &entries[6] else { panic!() };
    assert_eq!(*change, FileChangeKind::Edited);
    assert_eq!(hunks[0].lines, ["-one", "+two"]);

    // The shell command was blocked by a permission check, so it never got a result.
    let Entry::ToolCall { tool_name, tool_use_id: called, .. } = &entries[7] else { panic!() };
    assert_eq!(tool_name, "Bash");
    let Entry::ToolResult { tool_use_id, status, .. } = &entries[8] else { panic!() };
    assert_eq!((tool_use_id, *status), (called, ToolStatus::NotRun));

    let Entry::Response { text, .. } = &entries[9] else { panic!() };
    assert!(text.starts_with("I wrote `notes.txt`"));
}

#[test]
fn failure_session() {
    let entries = replay("failure-session");
    assert_eq!(
        entries.iter().map(kind).collect::<Vec<_>>(),
        [
            "prompt",
            "message",
            "tool-call",
            "tool-result",
            "tool-call",
            "tool-result",
            "message",
            "response",
            "session-ended",
        ]
    );

    let Entry::Message { text, r#final, .. } = &entries[1] else { panic!() };
    assert_eq!((text.as_str(), *r#final), ("Starting.", true));

    // A failed read is a failed tool call, and does not count as a touched file.
    let Entry::ToolResult { status, error, .. } = &entries[3] else { panic!() };
    assert_eq!(*status, ToolStatus::Failed);
    assert!(error.as_deref().unwrap().starts_with("File does not exist."));
    assert!(!entries.iter().any(|e| matches!(e, Entry::FileRead { .. })));
}

#[test]
fn all_events_of_a_prompt_share_its_id() {
    let entries = replay("edit-session");
    let Entry::Prompt { prompt_id, .. } = &entries[0] else { panic!() };
    for entry in &entries {
        let json = serde_json::to_value(entry).unwrap();
        assert_eq!(json["promptId"].as_str(), Some(prompt_id.as_str()), "{json}");
    }
}

#[test]
fn malformed_payload_is_an_error() {
    let mut builder = TimelineBuilder::new("/workspace/project");
    let mut payload = payloads("edit-session").remove(0);
    payload.remove("prompt");
    let error = builder.handle(payload).unwrap_err();
    assert_eq!(error.event, "UserPromptSubmit");
}

#[test]
fn unregistered_event_is_an_error() {
    let mut builder = TimelineBuilder::new("/workspace/project");
    let payload = serde_json::json!({ "hook_event_name": "SessionStart" });
    let Value::Object(payload) = payload else { unreachable!() };
    assert!(builder.handle(payload).is_err());
}

#[test]
fn successful_read_is_a_touched_file_outside_paths_stay_absolute() {
    let mut builder = TimelineBuilder::new("/workspace/project");
    let read = |path: &str| {
        let payload = serde_json::json!({
            "hook_event_name": "PostToolUse", "prompt_id": "p", "tool_use_id": path,
            "tool_name": "Read", "tool_input": { "file_path": path }, "tool_response": {}
        });
        let Value::Object(payload) = payload else { unreachable!() };
        payload
    };
    let inside = builder.handle(read("/workspace/project/src/main.rs")).unwrap();
    assert!(matches!(&inside[1], Entry::FileRead { path, .. } if path == "src/main.rs"));
    let outside = builder.handle(read("/etc/hosts")).unwrap();
    assert!(matches!(&outside[1], Entry::FileRead { path, .. } if path == "/etc/hosts"));
    let sibling = builder.handle(read("/workspace/project-other/a.txt")).unwrap();
    assert!(
        matches!(&sibling[1], Entry::FileRead { path, .. } if path == "/workspace/project-other/a.txt")
    );
}
