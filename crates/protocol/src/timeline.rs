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
    /// The whole text of a Markdown file the agent just created or changed, sent right after its
    /// `FileChange`; `None` when it is too large to send.
    Document {
        prompt_id: String,
        tool_use_id: String,
        path: String,
        text: Option<String>,
    },
    /// An OpenSpec change the agent works on; each one replaces the previous for that change.
    Plan {
        change: String,
        /// In the order of the change's schema.
        documents: Vec<PlanDocument>,
        tasks: TaskProgress,
    },
    /// The agent waits for the host, for example to allow a tool; `message` says what for.
    Attention {
        message: String,
    },
    /// Structured events cannot be captured; the terminal mirror still works.
    TimelineUnavailable {
        reason: String,
    },
    /// Events arrive again after `TimelineUnavailable`.
    TimelineAvailable,
    /// The session's token usage so far; each one replaces the previous. The tokens a prompt used
    /// are the difference between the last `Usage` before it and the last one during it.
    Usage {
        tokens: TokenCounts,
        /// Tokens in the agent's context window now, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        context_tokens: Option<u64>,
        /// The size of the context window, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        context_window: Option<u64>,
    },
    /// Usage cannot be read; the session and the timeline are not affected.
    UsageUnavailable {
        reason: String,
    },
    /// What Claude Code reported to its status line; each one replaces the previous.
    StatusLine {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        context_window: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        context_tokens: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effort: Option<String>,
        #[serde(default)]
        fast: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        five_hour: Option<RateLimit>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seven_day: Option<RateLimit>,
    },
    /// The session folder's name and git state; each one replaces the previous.
    Git {
        /// The folder's name, without the rest of the host's path.
        folder: String,
        /// `None` when the folder is not in a git repository.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(default)]
        dirty: bool,
        #[serde(default)]
        ahead: u32,
        #[serde(default)]
        behind: u32,
        /// The linked worktree's name, when the folder is in one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        worktree: Option<String>,
    },
}

/// One document of an OpenSpec change, as its schema names it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDocument {
    /// The schema's id for it, such as `proposal` or `feature-brief`.
    pub id: String,
    pub name: String,
    /// Its files, relative to the session folder; several for a document such as specs.
    pub paths: Vec<String>,
    pub exists: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskProgress {
    pub done: u32,
    pub total: u32,
}

/// Use of one of the host account's rate-limit windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    /// Whole percent.
    pub used_percent: u32,
    /// When the window resets, in seconds since the Unix epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<u64>,
}

/// Tokens used in a session, summed over every model call, subagents included.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenCounts {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl TokenCounts {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write
    }
}

impl std::ops::AddAssign for TokenCounts {
    fn add_assign(&mut self, other: Self) {
        self.input += other.input;
        self.output += other.output;
        self.cache_read += other.cache_read;
        self.cache_write += other.cache_write;
    }
}

impl std::ops::SubAssign for TokenCounts {
    fn sub_assign(&mut self, other: Self) {
        self.input -= other.input;
        self.output -= other.output;
        self.cache_read -= other.cache_read;
        self.cache_write -= other.cache_write;
    }
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
    /// Only from shell commands; the hunks remove the file's former content.
    Deleted,
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
    fn usage_serialises_with_camel_case_counts() {
        let entry = Entry::Usage {
            tokens: TokenCounts { input: 1, output: 2, cache_read: 3, cache_write: 4 },
            context_tokens: Some(8),
            context_window: None,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "usage",
                "tokens": { "input": 1, "output": 2, "cacheRead": 3, "cacheWrite": 4 },
                "contextTokens": 8
            })
        );
        assert_eq!(serde_json::from_value::<Entry>(json).unwrap(), entry);
    }

    #[test]
    fn documents_carry_their_text_or_null_when_too_large() {
        let document = |text: Option<&str>| Entry::Document {
            prompt_id: "p".into(),
            tool_use_id: "t".into(),
            path: "openspec/changes/x/proposal.md".into(),
            text: text.map(Into::into),
        };
        assert_eq!(
            serde_json::to_value(document(Some("# Proposal"))).unwrap(),
            serde_json::json!({
                "kind": "document", "promptId": "p", "toolUseId": "t",
                "path": "openspec/changes/x/proposal.md", "text": "# Proposal"
            })
        );
        assert_eq!(serde_json::to_value(document(None)).unwrap()["text"], Value::Null);
    }

    #[test]
    fn plan_lists_documents_in_schema_order_with_task_progress() {
        let entry = Entry::Plan {
            change: "x".into(),
            documents: vec![
                PlanDocument {
                    id: "feature-brief".into(),
                    name: "Feature brief".into(),
                    paths: vec!["openspec/changes/x/brief.md".into()],
                    exists: true,
                },
                PlanDocument {
                    id: "specs".into(),
                    name: "Specs".into(),
                    paths: vec![],
                    exists: false,
                },
            ],
            tasks: TaskProgress { done: 2, total: 8 },
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "plan", "change": "x",
                "documents": [
                    { "id": "feature-brief", "name": "Feature brief", "paths": ["openspec/changes/x/brief.md"], "exists": true },
                    { "id": "specs", "name": "Specs", "paths": [], "exists": false }
                ],
                "tasks": { "done": 2, "total": 8 }
            })
        );
        assert_eq!(serde_json::from_value::<Entry>(json).unwrap(), entry);
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
