//! Turns Claude Code hook payloads into timeline events (design decision 10). Payload shapes come
//! from the recorded hooks in `tests/fixtures/claude-hooks`; fields the timeline does not use are
//! ignored.

use std::path::{Component, Path};

use serde::Deserialize;
use serde_json::{Map, Value};
use specdeck_protocol::timeline::{DiffHunk, Entry, FileChangeKind, TimelineEvent, ToolStatus};

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
    /// Tool calls without a result yet: tool_use_id → prompt_id, in call order.
    open_tools: Vec<(String, String)>,
}

impl TimelineBuilder {
    pub fn new(cwd: impl Into<String>) -> Self {
        Self { cwd: cwd.into(), open_tools: Vec::new() }
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
        match Path::new(path).strip_prefix(&self.cwd) {
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
