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

    // No events from this agent: only the engine's own entries, the start and the folder's git
    // state (not a repository here), which a background check reports shortly after the start.
    tokio::time::timeout(Duration::from_secs(5), async {
        while !session.timeline().iter().any(|e| matches!(e.entry, Entry::Git { .. })) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("no git state");
    let entries: Vec<Entry> = session.timeline().into_iter().map(|e| e.entry).collect();
    assert_eq!(
        entries,
        [
            Entry::SessionStarted,
            Entry::Git {
                folder: bin.path().file_name().unwrap().to_string_lossy().into(),
                branch: None,
                dirty: false,
                ahead: 0,
                behind: 0,
                worktree: None
            }
        ]
    );
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

    let entries: Vec<Entry> = session
        .timeline()
        .into_iter()
        .map(|e| e.entry)
        .filter(|e| !matches!(e, Entry::Git { .. }))
        .collect();
    assert_eq!(
        entries,
        [
            Entry::SessionStarted,
            Entry::Prompt { prompt_id: "p1".into(), text: "hi".into() },
            Entry::Response { prompt_id: "p1".into(), text: "hello".into() },
        ]
    );
    // The prompt happened after "starting" was written, so it points into the output.
    let prompt = session.timeline().into_iter().find(|e| matches!(e.entry, Entry::Prompt { .. }));
    assert!(prompt.unwrap().output_seq >= 0);

    // The process exited, but the session stays open until the host closes it.
    assert_eq!(session.status(), AgentStatus::Exited);
    assert!(session.log().path().exists());
    session.close();
}

#[tokio::test]
async fn usage_follows_claude_codes_transcript() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let project = home.path().join(".claude/projects/-work");
    std::fs::create_dir_all(&project).unwrap();
    let call = |id: &str, output: u32| {
        format!(
            r#"{{"type":"assistant","isSidechain":false,"requestedModel":"claude-sonnet-5-5","message":{{"id":"{id}","usage":{{"input_tokens":10,"cache_read_input_tokens":0,"cache_creation_input_tokens":1000,"output_tokens":{output}}}}}}}"#
        )
    };
    // The second call reaches the transcript after the hook that named it, as a final answer can.
    fake_agent(
        bin.path(),
        "claude",
        &format!(
            r#"url=$(sed -n 's/.*"url": "\(http[^"]*\)".*/\1/p' "$2" | head -n 1)
t="$HOME/.claude/projects/-work/s1.jsonl"
echo '{first}' > "$t"
curl -s -o /dev/null -X POST -H "Authorization: Bearer $SPECDECK_HOOK_TOKEN" -d "{{\"hook_event_name\":\"Stop\",\"prompt_id\":\"p1\",\"transcript_path\":\"$t\"}}" "$url"
sleep 0.3
echo '{second}' >> "$t"
read wait"#,
            first = call("msg_1", 5),
            second = call("msg_2", 7),
        ),
    );
    let mut env = env(bin.path());
    env.set("HOME", home.path());

    let (session, mut events) =
        HostSession::start(config(AgentId::ClaudeCode, bin.path(), data.path()), &env)
            .await
            .unwrap();
    let last = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let SessionEvent::Timeline(event) = events.recv().await.expect("session ended")
                && let Entry::Usage { tokens, context_tokens, context_window } = event.entry
                && tokens.output == 12
            {
                return (tokens.total(), context_tokens, context_window);
            }
        }
    })
    .await
    .expect("usage of both calls did not arrive");
    assert_eq!(last, (2 * 1010 + 12, Some(1010), Some(200_000)));
    session.close();
}

#[tokio::test]
async fn status_line_reaches_specdeck_and_the_hosts_own_line_still_shows() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    std::fs::write(
        home.path().join(".claude/settings.json"),
        r#"{"statusLine":{"type":"command","command":"read -r x; echo host-line-shown"}}"#,
    )
    .unwrap();
    // Runs the status-line command from the --settings file with a payload, as Claude Code does.
    fake_agent(
        bin.path(),
        "claude",
        r#"cmd=$(sed -n 's/.*"command": "\(sh .*\)".*/\1/p' "$2" | head -n 1)
printf '%s\n' '{"model":{"display_name":"Opus 5.5 (1M context)"},"rate_limits":{"five_hour":{"used_percentage":23,"resets_at":1793500000}}}' | sh -c "$cmd"
read wait"#,
    );
    let mut env = env(bin.path());
    env.set("HOME", home.path());

    let (session, mut events) =
        HostSession::start(config(AgentId::ClaudeCode, bin.path(), data.path()), &env)
            .await
            .unwrap();
    wait_for_output(&session, &mut events, "host-line-shown").await;
    let status_line = || {
        session.timeline().into_iter().find_map(|event| match event.entry {
            Entry::StatusLine { model, five_hour, .. } => Some((model, five_hour)),
            _ => None,
        })
    };
    let (model, five_hour) = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(found) = status_line() {
                return found;
            }
            events.recv().await.expect("session ended");
        }
    })
    .await
    .expect("the status line did not reach SpecDeck");
    assert_eq!(model.as_deref(), Some("Opus 5.5"));
    assert_eq!(five_hour.map(|limit| limit.used_percent), Some(23));
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

#[tokio::test]
async fn files_a_shell_command_changes_reach_the_timeline() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .current_dir(repo.path())
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::write(repo.path().join("README.md"), "# Title\nold line\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "first"]);
    // Edits the file between the hooks around a Bash call, as Claude Code's shell tool does; the
    // hook answer must come first, so the edit waits for curl.
    fake_agent(
        bin.path(),
        "claude",
        r#"url=$(sed -n 's/.*"url": "\(http[^"]*\)".*/\1/p' "$2" | head -n 1)
post() { curl -s -o /dev/null -X POST -H "Authorization: Bearer $SPECDECK_HOOK_TOKEN" -d "$1" "$url"; }
post '{"hook_event_name":"UserPromptSubmit","prompt_id":"p1","prompt":"edit the readme"}'
post '{"hook_event_name":"PreToolUse","prompt_id":"p1","tool_name":"Bash","tool_use_id":"t1","tool_input":{"command":"sed"}}'
sed 's/old line/new line/' README.md > README.tmp && mv README.tmp README.md
post '{"hook_event_name":"PostToolUse","prompt_id":"p1","tool_name":"Bash","tool_use_id":"t1","tool_input":{"command":"sed"},"tool_response":{}}'
echo posted
exit 0"#,
    );

    let mut path = env(bin.path());
    path.set("HOME", repo.path());
    let (session, mut events) =
        HostSession::start(config(AgentId::ClaudeCode, repo.path(), data.path()), &path)
            .await
            .unwrap();
    next_matching(&mut events, |e| matches!(e, SessionEvent::Exited { .. })).await;
    let changes: Vec<Entry> = session
        .timeline()
        .into_iter()
        .map(|e| e.entry)
        .filter(|e| matches!(e, Entry::FileChange { .. }))
        .collect();
    assert_eq!(
        changes,
        [Entry::FileChange {
            prompt_id: "p1".into(),
            tool_use_id: "t1".into(),
            path: "README.md".into(),
            change: specdeck_protocol::timeline::FileChangeKind::Edited,
            hunks: vec![specdeck_protocol::timeline::DiffHunk {
                old_start: 1,
                old_lines: 2,
                new_start: 1,
                new_lines: 2,
                lines: vec![" # Title".into(), "-old line".into(), "+new line".into()],
            }],
        }]
    );
    session.close();
}

#[tokio::test]
async fn terminal_session_runs_the_hosts_login_shell() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    // Shows how it was started, then works like a shell until the host types exit.
    fake_agent(
        bin.path(),
        "myshell",
        r#"echo "args:$*"; echo "cwd:$(pwd -P)"; echo "marker:${CLAUDECODE:-none}"
read line; echo "got:$line"; read line; exit 4"#,
    );
    let mut env = env(bin.path());
    env.set("SHELL", bin.path().join("myshell"));
    // Started from a Claude Code session, as during development: the shell must not inherit it.
    env.set("CLAUDECODE", "1");

    let (session, mut events) =
        HostSession::start(config(AgentId::Terminal, folder.path(), data.path()), &env)
            .await
            .unwrap();
    wait_for_output(&session, &mut events, "marker:").await;
    let output = output_text(&session);
    assert!(output.contains("args:-l"), "{output}");
    let cwd = folder.path().canonicalize().unwrap();
    assert!(output.contains(&format!("cwd:{}", cwd.display())), "{output}");
    assert!(output.contains("marker:none"), "{output}");
    assert_eq!(session.status(), AgentStatus::Unknown);
    // No hooks or status line for a shell.
    let written: Vec<String> = std::fs::read_dir(data.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("hooks-") || name.starts_with("statusline-"))
        .collect();
    assert!(written.is_empty(), "{written:?}");

    session.input(b"ls\r").unwrap();
    wait_for_output(&session, &mut events, "got:ls").await;
    tokio::time::timeout(Duration::from_secs(5), async {
        while !session.timeline().iter().any(|e| matches!(e.entry, Entry::Git { .. })) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("no git state");

    session.input(b"exit\r").unwrap();
    let exited = next_matching(&mut events, |e| matches!(e, SessionEvent::Exited { .. })).await;
    assert_eq!(exited, SessionEvent::Exited { code: 4 });
    // The shell ended, but the session stays open until the host ends it.
    assert_eq!(session.status(), AgentStatus::Exited);
    assert!(session.log().path().exists());
    session.close();
}

#[tokio::test]
async fn openspec_documents_and_plans_reach_the_timeline() {
    let bin = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    // Writes a change as /opsx:propose would, ticks a task, then edits the proposal without
    // touching the plan; each step reported through hooks after the file is written.
    fake_agent(
        bin.path(),
        "claude",
        r#"url=$(sed -n 's/.*"url": "\(http[^"]*\)".*/\1/p' "$2" | head -n 1)
post() { curl -s -o /dev/null -X POST -H "Authorization: Bearer $SPECDECK_HOOK_TOKEN" -d "$1" "$url"; }
c="$PWD/openspec/changes/demo"
mkdir -p "$c"
post '{"hook_event_name":"UserPromptSubmit","prompt_id":"p1","prompt":"/opsx:propose demo"}'
write() { post "{\"hook_event_name\":\"PreToolUse\",\"prompt_id\":\"p1\",\"tool_name\":\"$1\",\"tool_use_id\":\"$2\",\"tool_input\":{\"file_path\":\"$3\"}}"
  printf '%b' "$4" > "$3"
  post "{\"hook_event_name\":\"PostToolUse\",\"prompt_id\":\"p1\",\"tool_name\":\"$1\",\"tool_use_id\":\"$2\",\"tool_input\":{\"file_path\":\"$3\"},\"tool_response\":{\"filePath\":\"$3\",\"originalFile\":null,\"content\":\"x\"}}"; }
write Write t1 "$c/proposal.md" '# Proposal\n\nWhy we do this.\n'
write Write t2 "$c/tasks.md" '- [ ] 1.1 one\n- [ ] 1.2 two\n'
sleep 1
write Edit t3 "$c/tasks.md" '- [x] 1.1 one\n- [ ] 1.2 two\n'
sleep 1
write Edit t4 "$c/proposal.md" '# Proposal\n\nWhy we really do this.\n'
sleep 1
echo posted
exit 0"#,
    );

    let (session, mut events) = HostSession::start(
        config(AgentId::ClaudeCode, folder.path(), data.path()),
        &env(bin.path()),
    )
    .await
    .unwrap();
    next_matching(&mut events, |e| matches!(e, SessionEvent::Exited { .. })).await;
    // The last plan read may still be on its way.
    tokio::time::sleep(Duration::from_millis(800)).await;

    let entries: Vec<Entry> = session.timeline().into_iter().map(|e| e.entry).collect();
    let documents: Vec<(String, Option<String>)> = entries
        .iter()
        .filter_map(|e| match e {
            Entry::Document { path, text, .. } => Some((path.clone(), text.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        documents,
        [
            (
                "openspec/changes/demo/proposal.md".into(),
                Some("# Proposal\n\nWhy we do this.\n".into())
            ),
            (
                "openspec/changes/demo/tasks.md".into(),
                Some("- [ ] 1.1 one\n- [ ] 1.2 two\n".into())
            ),
            (
                "openspec/changes/demo/tasks.md".into(),
                Some("- [x] 1.1 one\n- [ ] 1.2 two\n".into())
            ),
            (
                "openspec/changes/demo/proposal.md".into(),
                Some("# Proposal\n\nWhy we really do this.\n".into())
            ),
        ]
    );
    // Each document follows its own change.
    let first = entries.iter().position(|e| matches!(e, Entry::Document { .. })).unwrap();
    assert!(
        matches!(&entries[first - 1], Entry::FileChange { path, .. } if path.ends_with("proposal.md"))
    );

    let plans: Vec<(String, u32, u32)> = entries
        .iter()
        .filter_map(|e| match e {
            Entry::Plan { change, tasks, .. } => Some((change.clone(), tasks.done, tasks.total)),
            _ => None,
        })
        .collect();
    // The proposal edit at the end changed nothing in the plan, so no third plan.
    assert_eq!(plans, [("demo".into(), 0, 2), ("demo".into(), 1, 2)]);
    session.close();
}
