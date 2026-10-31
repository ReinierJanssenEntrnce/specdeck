//! Drives a hosted session end to end with fake agents in a real pty.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use specdeck_core::environment::Environment;
use specdeck_core::output_log::Record;
use specdeck_core::session::{HostConfig, HostSession, SessionEvent};
use specdeck_protocol::timeline::Entry;
use specdeck_protocol::{AgentId, AgentStatus};
use tokio::sync::mpsc::UnboundedReceiver;

fn fake_agent(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn env(bin: &Path) -> Environment {
    [("PATH", format!("{}:/usr/bin:/bin", bin.display()))].into_iter().collect()
}

fn config(agent: AgentId, folder: &Path, data: &Path) -> HostConfig {
    HostConfig { agent, folder: folder.to_owned(), cols: 80, rows: 24, data_dir: data.to_owned() }
}

/// All terminal output in the log so far, as text.
fn output_text(session: &HostSession) -> String {
    let records = session.log().read(0, usize::MAX).unwrap();
    records
        .iter()
        .filter_map(|r| match r {
            Record::Output { bytes, .. } => Some(String::from_utf8_lossy(bytes).into_owned()),
            Record::Resize { .. } => None,
        })
        .collect()
}

async fn wait_for_output(
    session: &HostSession,
    events: &mut UnboundedReceiver<SessionEvent>,
    text: &str,
) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !output_text(session).contains(text) {
            events.recv().await.expect("session ended");
        }
    })
    .await
    .unwrap_or_else(|_| panic!("no {text:?} in {:?}", output_text(session)));
}

async fn next_matching(
    events: &mut UnboundedReceiver<SessionEvent>,
    matches: impl Fn(&SessionEvent) -> bool,
) -> SessionEvent {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let event = events.recv().await.expect("session ended");
            if matches(&event) {
                return event;
            }
        }
    })
    .await
    .expect("event did not arrive")
}

#[tokio::test]
async fn terminal_only_agent_takes_host_input_and_resizes() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    fake_agent(bin.path(), "opencode", r#"echo ready; read line; echo "got:$line"; read wait"#);

    let (session, mut events) =
        HostSession::start(config(AgentId::OpenCode, bin.path(), data.path()), &env(bin.path()))
            .await
            .unwrap();
    assert_eq!(session.status(), AgentStatus::Unknown);
    wait_for_output(&session, &mut events, "ready").await;

    session.input(b"hello\r").unwrap();
    wait_for_output(&session, &mut events, "got:hello").await;

    session.resize(100, 30);
    next_matching(&mut events, |e| matches!(e, SessionEvent::Resized { cols: 100, rows: 30 }))
        .await;
    let records = session.log().read(0, usize::MAX).unwrap();
    let last = records.last().unwrap();
    assert!(matches!(last, Record::Resize { cols: 100, rows: 30, .. }), "{last:?}");
    // Gap-free and in order: the resize follows all earlier output.
    assert!(records.iter().enumerate().all(|(i, r)| r.seq() == i as u64));

    // No events from this agent: only the engine's own session-started entry.
    let timeline = session.timeline();
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].entry, Entry::SessionStarted);
    assert_eq!(session.status(), AgentStatus::Unknown);

    let log_path = session.log().path().to_owned();
    session.close();
    tokio::time::timeout(Duration::from_secs(5), async {
        while log_path.exists() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("output log was not deleted on close");
}

#[tokio::test]
async fn claude_code_hooks_build_the_timeline_and_exit_keeps_the_session_open() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    // Reads the hook URL from the --settings file and posts two events with the token from the
    // environment, as Claude Code does.
    fake_agent(
        bin.path(),
        "claude",
        r#"[ "$1" = "--settings" ] || { echo "no settings"; exit 9; }
url=$(sed -n 's/.*"url": "\(http[^"]*\)".*/\1/p' "$2" | head -n 1)
post() { curl -s -o /dev/null -X POST -H "Authorization: Bearer $SPECDECK_HOOK_TOKEN" -H 'Content-Type: application/json' -d "$1" "$url"; }
echo starting
post '{"hook_event_name":"UserPromptSubmit","prompt_id":"p1","prompt":"hi"}'
post '{"hook_event_name":"Stop","prompt_id":"p1","last_assistant_message":"hello"}'
echo posted
exit 3"#,
    );

    let (session, mut events) =
        HostSession::start(config(AgentId::ClaudeCode, bin.path(), data.path()), &env(bin.path()))
            .await
            .unwrap();

    let exited = next_matching(&mut events, |e| matches!(e, SessionEvent::Exited { .. })).await;
    assert_eq!(exited, SessionEvent::Exited { code: 3 });
    assert!(output_text(&session).contains("posted"), "{}", output_text(&session));

    let entries: Vec<Entry> = session.timeline().into_iter().map(|e| e.entry).collect();
    assert_eq!(
        entries,
        [
            Entry::SessionStarted,
            Entry::Prompt { prompt_id: "p1".into(), text: "hi".into() },
            Entry::Response { prompt_id: "p1".into(), text: "hello".into() },
        ]
    );
    // The prompt happened after "starting" was written, so it points into the output.
    assert!(session.timeline()[1].output_seq >= 0);

    // The process exited, but the session stays open until the host closes it.
    assert_eq!(session.status(), AgentStatus::Exited);
    assert!(session.log().path().exists());
    session.close();
}

#[tokio::test]
async fn agent_that_is_not_installed_creates_no_session() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let result =
        HostSession::start(config(AgentId::Omp, bin.path(), data.path()), &env(bin.path())).await;
    assert!(result.is_err());
    assert_eq!(std::fs::read_dir(data.path()).unwrap().count(), 0, "left files behind");
}
