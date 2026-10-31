use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// One entry of a session's timeline.
///
/// Every event carries `output_seq`, the newest terminal output record when it happened (-1
/// before any output), which lets a participant jump from the entry to its place in the terminal,
/// and `at`, milliseconds since the Unix epoch. Sent to viewers as JSON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEvent {
    pub output_seq: i64,
    pub at: u64,
    #[serde(flatten)]
    pub entry: Entry,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum Entry {
    SessionStarted,
    SessionEnded,
    Prompt {
        prompt_id: String,
        text: String,
    },
    /// Assistant text as it is shown; several per prompt, including text between tool calls.
    Message {
        prompt_id: String,
        message_id: String,
        text: String,
        r#final: bool,
    },
    /// The final response of a prompt.
    Response {
        prompt_id: String,
        text: String,
    },
    ToolCall {
        prompt_id: String,
        tool_use_id: String,
        tool_name: String,
        input: Map<String, Value>,
    },
    ToolResult {
        prompt_id: String,
        tool_use_id: String,
        status: ToolStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    FileChange {
        prompt_id: String,
        tool_use_id: String,
        path: String,
        change: FileChangeKind,
        hunks: Vec<DiffHunk>,
    },
    FileRead {
        prompt_id: String,
        tool_use_id: String,
        path: String,
    },
    /// Structured events cannot be captured; the terminal mirror still works.
    TimelineUnavailable {
        reason: String,
    },
    /// Events arrive again after `TimelineUnavailable`.
    TimelineAvailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolStatus {
    Succeeded,
    Failed,
    /// The call never ran, for example because a permission check blocked it.
    NotRun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileChangeKind {
    Created,
    Edited,
}

/// A unified-diff hunk; `lines` start with ' ', '-' or '+'.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_with_kind_tag_and_camel_case_fields() {
        let event = TimelineEvent {
            output_seq: 4,
            at: 1000,
            entry: Entry::ToolResult {
                prompt_id: "p".into(),
                tool_use_id: "t".into(),
                status: ToolStatus::NotRun,
                error: None,
            },
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "outputSeq": 4, "at": 1000, "kind": "tool-result",
                "promptId": "p", "toolUseId": "t", "status": "not-run"
            })
        );
        assert_eq!(serde_json::from_value::<TimelineEvent>(json).unwrap(), event);
    }

    #[test]
    fn message_final_field_is_named_final() {
        let entry = Entry::Message {
            prompt_id: "p".into(),
            message_id: "m".into(),
            text: "hi".into(),
            r#final: true,
        };
        assert_eq!(serde_json::to_value(&entry).unwrap()["final"], true);
    }
}
