//! Turns Claude Code hook payloads into timeline events (design decision 10). Payload shapes come
//! from the recorded hooks in `tests/fixtures/claude-hooks`; fields the timeline does not use are
//! ignored.

use std::path::{Component, Path};

use serde::Deserialize;
use serde_json::{Map, Value};
use specdeck_protocol::timeline::{DiffHunk, Entry, FileChangeKind, TimelineEvent, ToolStatus};

use crate::documents::DocumentText;
use crate::hooks::HookPayload;
use crate::shell_changes::CommandChanges;

#[derive(Debug, thiserror::Error)]
#[error("Unexpected {event} payload: {reason}")]
pub struct HookPayloadError {
    pub event: String,
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(tag = "hook_event_name")]
enum Payload {
    UserPromptSubmit {
        prompt_id: String,
        prompt: String,
    },
    MessageDisplay {
        prompt_id: String,
        message_id: String,
        delta: String,
        r#final: bool,
    },
    Stop {
        prompt_id: String,
        last_assistant_message: Option<String>,
    },
    PreToolUse(ToolCall),
    PostToolUse {
        #[serde(flatten)]
        call: ToolCall,
        #[serde(default)]
        tool_response: Value,
    },
    PostToolUseFailure {
        #[serde(flatten)]
        call: ToolCall,
        error: Option<String>,
    },
    SessionEnd,
    /// Claude Code asks the host something, for example permission to use a tool.
    Notification {
        #[serde(default)]
        message: String,
        #[serde(default)]
        notification_type: String,
    },
}

/// Notifications that mean Claude Code waits for the host; others (such as being idle) do not.
const ATTENTION_TYPES: [&str; 2] = ["permission_prompt", "elicitation_dialog"];

#[derive(Deserialize)]
struct ToolCall {
    prompt_id: String,
    tool_name: String,
    tool_use_id: String,
    tool_input: Map<String, Value>,
}

/// What file-editing tools report in `tool_response`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileResponse {
    file_path: String,
    /// `null` for a new file; absent when the tool does not report it.
    #[serde(default, deserialize_with = "present_or_null")]
    original_file: Option<Option<String>>,
    content: Option<String>,
    #[serde(default)]
    structured_patch: Vec<DiffHunk>,
}

/// Tells `"originalFile": null` (a new file) apart from a missing field.
fn present_or_null<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(d).map(Some)
}

const FILE_EDITING_TOOLS: [&str; 4] = ["Write", "Edit", "MultiEdit", "NotebookEdit"];
const FILE_READING_TOOLS: [&str; 1] = ["Read"];

pub struct TimelineBuilder {
    /// Session folder; file paths inside it are shown relative to it.
    cwd: String,
    /// The same folder with symlinks resolved, as the agent may report it (`/private/var` for
    /// `/var` on macOS).
    resolved_cwd: Option<std::path::PathBuf>,
    /// Tool calls without a result yet: tool_use_id → prompt_id, in call order.
    open_tools: Vec<(String, String)>,
}

impl TimelineBuilder {
    pub fn new(cwd: impl Into<String>) -> Self {
        let cwd = cwd.into();
        let resolved_cwd =
            Path::new(&cwd).canonicalize().ok().filter(|resolved| *resolved != Path::new(&cwd));
        Self { cwd, resolved_cwd, open_tools: Vec::new() }
    }

    /// The event for starting the agent, which the engine emits itself.
    pub fn session_started(&self) -> Entry {
        Entry::SessionStarted
    }

    /// Converts one hook payload into zero or more timeline entries.
    pub fn handle(&mut self, raw: HookPayload) -> Result<Vec<Entry>, HookPayloadError> {
        let event =
            raw.get("hook_event_name").and_then(Value::as_str).unwrap_or("unknown").to_owned();
        let payload: Payload = serde_json::from_value(Value::Object(raw))
            .map_err(|e| HookPayloadError { event, reason: e.to_string() })?;

        Ok(match payload {
            Payload::UserPromptSubmit { prompt_id, prompt } => {
                vec![Entry::Prompt { prompt_id, text: prompt }]
            }
            Payload::MessageDisplay { prompt_id, message_id, delta, r#final } => {
                vec![Entry::Message { prompt_id, message_id, text: delta, r#final }]
            }
            Payload::PreToolUse(call) => {
                self.open_tools.push((call.tool_use_id.clone(), call.prompt_id.clone()));
                vec![Entry::ToolCall {
                    prompt_id: call.prompt_id,
                    tool_use_id: call.tool_use_id,
                    tool_name: call.tool_name,
                    input: call.tool_input,
                }]
            }
            Payload::PostToolUse { call, tool_response } => {
                self.close_tool(&call.tool_use_id);
                let mut entries = vec![Entry::ToolResult {
                    prompt_id: call.prompt_id.clone(),
                    tool_use_id: call.tool_use_id.clone(),
                    status: ToolStatus::Succeeded,
                    error: None,
                }];
                if FILE_EDITING_TOOLS.contains(&call.tool_name.as_str()) {
                    entries.extend(self.file_change(&call, tool_response));
                }
                // Only successful reads count as touched; a failed read of a missing file does not.
                if FILE_READING_TOOLS.contains(&call.tool_name.as_str())
                    && let Some(path) = call.tool_input.get("file_path").and_then(Value::as_str)
                {
                    entries.push(Entry::FileRead {
                        prompt_id: call.prompt_id,
                        tool_use_id: call.tool_use_id,
                        path: self.display_path(path),
                    });
                }
                entries
            }
            Payload::PostToolUseFailure { call, error } => {
                self.close_tool(&call.tool_use_id);
                vec![Entry::ToolResult {
                    prompt_id: call.prompt_id,
                    tool_use_id: call.tool_use_id,
                    status: ToolStatus::Failed,
                    error,
                }]
            }
            Payload::Stop { prompt_id, last_assistant_message } => {
                // A tool call without a result by the end of the turn never ran.
                let (not_run, open): (Vec<_>, Vec<_>) = std::mem::take(&mut self.open_tools)
                    .into_iter()
                    .partition(|(_, p)| *p == prompt_id);
                self.open_tools = open;
                let mut entries: Vec<Entry> = not_run
                    .into_iter()
                    .map(|(tool_use_id, prompt_id)| Entry::ToolResult {
                        prompt_id,
                        tool_use_id,
                        status: ToolStatus::NotRun,
                        error: None,
                    })
                    .collect();
                if let Some(text) = last_assistant_message {
                    entries.push(Entry::Response { prompt_id, text });
                }
                entries
            }
            Payload::SessionEnd => vec![Entry::SessionEnded],
            Payload::Notification { message, notification_type } => {
                if ATTENTION_TYPES.contains(&notification_type.as_str()) {
                    vec![Entry::Attention { message }]
                } else {
                    Vec::new()
                }
            }
        })
    }

    /// `entries` with each document's text right after the change that produced it. A document
    /// is only sent with a change of its own, and never for a deletion.
    pub fn with_documents(&self, entries: Vec<Entry>, documents: Vec<DocumentText>) -> Vec<Entry> {
        if documents.is_empty() {
            return entries;
        }
        let mut out = Vec::with_capacity(entries.len() + documents.len());
        for entry in entries {
            let document = match &entry {
                Entry::FileChange { prompt_id, tool_use_id, path, change, .. }
                    if *change != FileChangeKind::Deleted =>
                {
                    documents.iter().find(|doc| self.display_path(&doc.path) == *path).map(|doc| {
                        Entry::Document {
                            prompt_id: prompt_id.clone(),
                            tool_use_id: tool_use_id.clone(),
                            path: path.clone(),
                            text: doc.text.clone(),
                        }
                    })
                }
                _ => None,
            };
            out.push(entry);
            out.extend(document);
        }
        out
    }

    /// Entries for the files a shell command changed, after its result (decision 10b).
    pub fn command_changes(&self, changes: CommandChanges) -> Vec<Entry> {
        changes
            .files
            .into_iter()
            .map(|file| Entry::FileChange {
                prompt_id: changes.prompt_id.clone(),
                tool_use_id: changes.tool_use_id.clone(),
                path: self.display_path(&file.path),
                change: file.change,
                hunks: file.hunks,
            })
            .collect()
    }

    fn close_tool(&mut self, tool_use_id: &str) {
        self.open_tools.retain(|(id, _)| id != tool_use_id);
    }

    fn file_change(&self, call: &ToolCall, response: Value) -> Option<Entry> {
        let response: FileResponse = serde_json::from_value(response).ok()?;
        let created = response.original_file == Some(None);
        let hunks = match (created, response.content) {
            (true, Some(content)) => vec![added_file(&content)],
            _ if !response.structured_patch.is_empty() => response.structured_patch,
            _ => return None,
        };
        Some(Entry::FileChange {
            prompt_id: call.prompt_id.clone(),
            tool_use_id: call.tool_use_id.clone(),
            path: self.display_path(&response.file_path),
            change: if created { FileChangeKind::Created } else { FileChangeKind::Edited },
            hunks,
        })
    }

    /// Paths inside the session folder relative to it; others unchanged.
    fn display_path(&self, path: &str) -> String {
        let full = Path::new(path);
        let inside = full.strip_prefix(&self.cwd).or_else(|e| {
            self.resolved_cwd.as_ref().map_or(Err(e), |resolved| full.strip_prefix(resolved))
        });
        match inside {
            Ok(inside)
                if !inside.as_os_str().is_empty()
                    && inside.components().all(|c| matches!(c, Component::Normal(_))) =>
            {
                inside.to_string_lossy().into_owned()
            }
            _ => path.to_owned(),
        }
    }
}

/// Stamps entries with the terminal position and time they happened.
pub fn stamp(entry: Entry, output_seq: Option<u64>, at: u64) -> TimelineEvent {
    TimelineEvent { output_seq: output_seq.map_or(-1, |seq| seq as i64), at, entry }
}

/// A hunk that adds every line of a new file.
fn added_file(content: &str) -> DiffHunk {
    let content = content.strip_suffix('\n').unwrap_or(content);
    let lines: Vec<String> = content.split('\n').map(|line| format!("+{line}")).collect();
    DiffHunk { old_start: 0, old_lines: 0, new_start: 1, new_lines: lines.len() as u32, lines }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_terminal_position_and_time() {
        assert_eq!(stamp(Entry::SessionStarted, None, 5).output_seq, -1);
        let event = stamp(Entry::SessionEnded, Some(41), 1_700_000_000_000);
        assert_eq!((event.output_seq, event.at), (41, 1_700_000_000_000));
    }

    fn payload(value: serde_json::Value) -> HookPayload {
        value.as_object().unwrap().clone()
    }

    fn change(path: &str, change: FileChangeKind) -> Entry {
        Entry::FileChange {
            prompt_id: "p".into(),
            tool_use_id: "t".into(),
            path: path.into(),
            change,
            hunks: Vec::new(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn paths_under_a_symlinked_folder_are_shown_relative() {
        let real = tempfile::tempdir().unwrap();
        let links = tempfile::tempdir().unwrap();
        let link = links.path().join("project");
        std::os::unix::fs::symlink(real.path(), &link).unwrap();
        let builder = TimelineBuilder::new(link.to_string_lossy());
        let resolved = real.path().canonicalize().unwrap().join("docs/a.md");
        assert_eq!(builder.display_path(&resolved.to_string_lossy()), "docs/a.md");
        assert_eq!(builder.display_path(&link.join("b.md").to_string_lossy()), "b.md");
        assert_eq!(builder.display_path("/elsewhere/c.md"), "/elsewhere/c.md");
    }

    #[test]
    fn a_document_follows_its_change() {
        let builder = TimelineBuilder::new("/work");
        let entries = vec![
            Entry::ToolResult {
                prompt_id: "p".into(),
                tool_use_id: "t".into(),
                status: ToolStatus::Succeeded,
                error: None,
            },
            change("docs/plan.md", FileChangeKind::Edited),
            change("src/server.ts", FileChangeKind::Edited),
            change("old.md", FileChangeKind::Deleted),
            change("big.md", FileChangeKind::Created),
        ];
        let documents = vec![
            DocumentText { path: "/work/big.md".into(), text: None },
            DocumentText { path: "/work/docs/plan.md".into(), text: Some("# Plan".into()) },
            // A document without a change of its own is never sent.
            DocumentText { path: "/work/untouched.md".into(), text: Some("secret".into()) },
        ];
        let document = |path: &str, text: Option<&str>| Entry::Document {
            prompt_id: "p".into(),
            tool_use_id: "t".into(),
            path: path.into(),
            text: text.map(Into::into),
        };
        let with = builder.with_documents(entries.clone(), documents);
        assert_eq!(
            with,
            [
                entries[0].clone(),
                entries[1].clone(),
                document("docs/plan.md", Some("# Plan")),
                entries[2].clone(),
                entries[3].clone(),
                entries[4].clone(),
                document("big.md", None),
            ]
        );
    }

    #[test]
    fn a_permission_request_asks_for_attention_and_idle_reminders_do_not() {
        let mut builder = TimelineBuilder::new("/work");
        let asked = builder.handle(payload(serde_json::json!({
            "hook_event_name": "Notification",
            "message": "Claude needs your permission to use Bash",
            "notification_type": "permission_prompt"
        })));
        assert_eq!(
            asked.unwrap(),
            [Entry::Attention { message: "Claude needs your permission to use Bash".into() }]
        );
        let idle = builder.handle(payload(serde_json::json!({
            "hook_event_name": "Notification",
            "message": "Claude is waiting for your input",
            "notification_type": "idle_prompt"
        })));
        assert!(idle.unwrap().is_empty());
    }
}
