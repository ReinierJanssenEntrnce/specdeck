//! Token usage and context fill of a Claude Code session, read from the transcript Claude Code
//! writes (design decision 10a).
//!
//! Hook payloads name the transcript in `transcript_path`. The reader accepts only a `.jsonl` file
//! under Claude Code's own projects folder, reads only what was added since its last read, and
//! counts each model call once: Claude Code repeats a call's usage on every line of that answer.
//! Subagents write their own transcripts in `<session>/subagents/`; they add to the session total
//! but not to the context fill. Lines the reader does not use are skipped; a line that is not
//! JSON makes usage unavailable for the rest of the session rather than undercounted.

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde_json::Value;
use specdeck_protocol::timeline::{Entry, TokenCounts};

use crate::environment::Environment;
use crate::hooks::HookPayload;

const DEFAULT_WINDOW: u64 = 200_000;
const LARGE_WINDOW: u64 = 1_000_000;

/// Claude Code's projects folder, where it keeps transcripts: `$CLAUDE_CONFIG_DIR/projects`, or
/// `.claude/projects` in the home folder.
pub fn projects_dir(env: &Environment) -> Option<PathBuf> {
    if let Some(config) = env.get("CLAUDE_CONFIG_DIR").filter(|dir| !dir.is_empty()) {
        return Some(PathBuf::from(config).join("projects"));
    }
    let home = env.get("HOME").or_else(|| env.get("USERPROFILE")).filter(|dir| !dir.is_empty())?;
    Some(PathBuf::from(home).join(".claude").join("projects"))
}

pub struct UsageReader {
    projects: Option<PathBuf>,
    main: Option<Cursor>,
    subagents: BTreeMap<PathBuf, Cursor>,
    /// Usage per model call, so a call seen again replaces its earlier count.
    calls: HashMap<String, TokenCounts>,
    totals: TokenCounts,
    context: Option<u64>,
    model_known: bool,
    /// The session's model has the 1M context window.
    large_window: bool,
    /// Claude Code finished answering, so its transcript must exist.
    answered: bool,
    unavailable: Option<String>,
    last: Option<Entry>,
}

struct Cursor {
    path: PathBuf,
    offset: u64,
}

impl UsageReader {
    pub fn new(projects: Option<PathBuf>) -> Self {
        Self {
            projects,
            main: None,
            subagents: BTreeMap::new(),
            calls: HashMap::new(),
            totals: TokenCounts::default(),
            context: None,
            model_known: false,
            large_window: false,
            answered: false,
            unavailable: None,
            last: None,
        }
    }

    /// Learns the transcript from a hook payload. Call [`UsageReader::poll`] afterwards.
    pub fn on_hook(&mut self, payload: &HookPayload) {
        if payload.get("hook_event_name").and_then(Value::as_str) == Some("Stop") {
            self.answered = true;
        }
        let Some(path) = payload.get("transcript_path").and_then(Value::as_str) else {
            return;
        };
        let path = PathBuf::from(path);
        if self.main.as_ref().is_some_and(|main| main.path == path) {
            return;
        }
        match self.check_path(&path) {
            // `/clear` starts a new transcript; the session total carries on.
            Ok(()) => {
                self.main = Some(Cursor { path, offset: 0 });
                self.subagents.clear();
            }
            Err(reason) => self.fail(reason),
        }
    }

    /// Reads what was added to the transcripts. Returns an entry when the usage shown changed.
    pub fn poll(&mut self) -> Option<Entry> {
        if self.unavailable.is_none()
            && let Err(reason) = self.read()
        {
            self.fail(reason);
        }
        let entry = match &self.unavailable {
            Some(reason) => Entry::UsageUnavailable { reason: reason.clone() },
            None if self.totals.total() == 0 && self.context.is_none() => return None,
            None => Entry::Usage {
                tokens: self.totals,
                context_tokens: self.context,
                context_window: self.window(),
            },
        };
        if self.last.as_ref() == Some(&entry) {
            return None;
        }
        self.last = Some(entry.clone());
        Some(entry)
    }

    fn fail(&mut self, reason: impl Into<String>) {
        if self.unavailable.is_none() {
            self.unavailable = Some(reason.into());
        }
    }

    fn check_path(&self, path: &Path) -> Result<(), String> {
        let outside = || "Claude Code's transcript is not in Claude Code's own folder".to_string();
        let projects = self
            .projects
            .as_ref()
            .and_then(|dir| dir.canonicalize().ok())
            .ok_or("Claude Code's folder was not found")?;
        if !path.is_absolute() || path.extension().is_none_or(|ext| ext != "jsonl") {
            return Err(outside());
        }
        // The transcript itself may not exist yet; its folder does.
        let parent = path.parent().and_then(|dir| dir.canonicalize().ok()).ok_or_else(outside)?;
        if !parent.starts_with(&projects) {
            return Err(outside());
        }
        Ok(())
    }

    fn read(&mut self) -> Result<(), String> {
        let Some(mut main) = self.main.take() else { return Ok(()) };
        let result = self.read_file(&mut main, true);
        let subagent_dir = main.path.with_extension("").join("subagents");
        self.main = Some(main);
        match result {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if self.answered {
                    return Err("Claude Code's transcript was not found".into());
                }
                return Ok(());
            }
            Err(error) => {
                return Err(format!("Claude Code's transcript could not be read: {error}"));
            }
            Ok(()) => {}
        }

        if let Ok(dir) = std::fs::read_dir(&subagent_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "jsonl") {
                    self.subagents
                        .entry(path.clone())
                        .or_insert_with(|| Cursor { path, offset: 0 });
                }
            }
        }
        let paths: Vec<PathBuf> = self.subagents.keys().cloned().collect();
        for path in paths {
            let mut cursor = self.subagents.remove(&path).expect("listed above");
            let result = self.read_file(&mut cursor, false);
            self.subagents.insert(path, cursor);
            result.map_err(|error| format!("A subagent transcript could not be read: {error}"))?;
        }
        Ok(())
    }

    /// Reads the complete lines added since the last read; a line still being written waits.
    fn read_file(&mut self, cursor: &mut Cursor, main: bool) -> io::Result<()> {
        let mut file = File::open(&cursor.path)?;
        if file.metadata()?.len() < cursor.offset {
            // Rewritten from the start; calls already counted are recognised by their id.
            cursor.offset = 0;
        }
        file.seek(SeekFrom::Start(cursor.offset))?;
        let mut added = Vec::new();
        file.read_to_end(&mut added)?;
        let Some(end) = added.iter().rposition(|&b| b == b'\n') else { return Ok(()) };
        for line in added[..end].split(|&b| b == b'\n') {
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let value: Value = serde_json::from_slice(line).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "a line is not valid JSON")
            })?;
            self.line(&value, main);
        }
        cursor.offset += end as u64 + 1;
        Ok(())
    }

    fn line(&mut self, line: &Value, main: bool) {
        let sidechain = line.get("isSidechain").and_then(Value::as_bool).unwrap_or(false);
        match line.get("type").and_then(Value::as_str) {
            Some("assistant") => {
                let Some(message) = line.get("message") else { return };
                let (Some(id), Some(usage)) =
                    (message.get("id").and_then(Value::as_str), message.get("usage"))
                else {
                    return;
                };
                let count = |field| usage.get(field).and_then(Value::as_u64).unwrap_or(0);
                let tokens = TokenCounts {
                    input: count("input_tokens"),
                    output: count("output_tokens"),
                    cache_read: count("cache_read_input_tokens"),
                    cache_write: count("cache_creation_input_tokens"),
                };
                // Messages Claude Code makes up itself, such as errors, carry no usage.
                if tokens.total() == 0 {
                    return;
                }
                if let Some(earlier) = self.calls.insert(id.to_owned(), tokens) {
                    self.totals -= earlier;
                }
                self.totals += tokens;
                if main && !sidechain {
                    let context = tokens.input + tokens.cache_read + tokens.cache_write;
                    self.context = Some(context);
                    self.model_known = true;
                    // More than the default window proves the large one, also after compaction.
                    self.large_window |= context > DEFAULT_WINDOW
                        || line
                            .get("requestedModel")
                            .and_then(Value::as_str)
                            .is_some_and(|model| model.ends_with("[1m]"));
                }
            }
            Some("system")
                if main
                    && line.get("subtype").and_then(Value::as_str) == Some("compact_boundary") =>
            {
                if let Some(tokens) =
                    line.pointer("/compactMetadata/postTokens").and_then(Value::as_u64)
                {
                    self.context = Some(tokens);
                }
            }
            _ => {}
        }
    }

    fn window(&self) -> Option<u64> {
        if !self.model_known {
            return None;
        }
        Some(if self.large_window { LARGE_WINDOW } else { DEFAULT_WINDOW })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serde_json::json;

    use super::*;

    const NORMAL: &str = include_str!("../tests/fixtures/claude-transcript/normal.jsonl");
    const COMPACTED: &str = include_str!("../tests/fixtures/claude-transcript/compacted.jsonl");
    const AFTER_COMPACT: &str =
        include_str!("../tests/fixtures/claude-transcript/after-compact.jsonl");
    const SUBAGENT: &str = include_str!("../tests/fixtures/claude-transcript/subagent.jsonl");

    /// A projects folder with one project, and the path its transcript would have.
    fn projects() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("projects").join("-work-app");
        std::fs::create_dir_all(&project).unwrap();
        (dir, project.join("0b1e5a2c.jsonl"))
    }

    fn new_reader(dir: &tempfile::TempDir) -> UsageReader {
        UsageReader::new(Some(dir.path().join("projects")))
    }

    fn hook(event: &str, transcript: &Path) -> HookPayload {
        let value = json!({ "hook_event_name": event, "transcript_path": transcript });
        value.as_object().unwrap().clone()
    }

    fn append(path: &Path, text: &str) {
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    fn usage(entry: Option<Entry>) -> (TokenCounts, Option<u64>, Option<u64>) {
        match entry {
            Some(Entry::Usage { tokens, context_tokens, context_window }) => {
                (tokens, context_tokens, context_window)
            }
            other => panic!("expected usage, got {other:?}"),
        }
    }

    #[test]
    fn counts_each_call_once_and_reports_context_fill() {
        let (dir, transcript) = projects();
        append(&transcript, NORMAL);
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("Stop", &transcript));

        let (tokens, context, window) = usage(reader.poll());
        assert_eq!(
            tokens,
            TokenCounts { input: 5, output: 50, cache_read: 12_000, cache_write: 12_300 }
        );
        assert_eq!(context, Some(2 + 12_000 + 300));
        assert_eq!(window, Some(200_000));
        assert_eq!(reader.poll(), None, "nothing changed");
    }

    #[test]
    fn reads_only_complete_new_lines() {
        let (dir, transcript) = projects();
        let (first, rest) = NORMAL.split_at(NORMAL.find("\"msg_02\"").unwrap());
        append(&transcript, first);
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("PostToolUse", &transcript));
        assert_eq!(usage(reader.poll()).0.output, 40, "the half-written line waits");

        append(&transcript, rest);
        assert_eq!(usage(reader.poll()).0.output, 50);
    }

    #[test]
    fn compaction_lowers_the_context_but_not_the_total() {
        let (dir, transcript) = projects();
        let (before, boundary) =
            COMPACTED.split_at(COMPACTED.find("{\"parentUuid\":null,\"logical").unwrap());
        append(&transcript, before);
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("Stop", &transcript));
        let (total, context, window) = usage(reader.poll());
        assert_eq!(context, Some(341_002));
        assert_eq!(window, Some(1_000_000), "the session uses the 1M model");

        append(&transcript, boundary);
        let (after, context, _) = usage(reader.poll());
        assert_eq!(context, Some(28_000));
        assert_eq!(after, total);

        append(&transcript, AFTER_COMPACT);
        let (after, context, _) = usage(reader.poll());
        assert_eq!(context, Some(29_002));
        assert_eq!(after.total(), total.total() + 29_022);
    }

    #[test]
    fn large_context_implies_the_large_window() {
        let (dir, transcript) = projects();
        append(&transcript, &COMPACTED.replace("claude-opus-5-5[1m]", "claude-opus-5-5"));
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("Stop", &transcript));
        assert_eq!(usage(reader.poll()).2, Some(1_000_000));
    }

    #[test]
    fn subagents_add_to_the_total_but_not_the_context() {
        let (dir, transcript) = projects();
        append(&transcript, NORMAL);
        let subagents = transcript.with_extension("").join("subagents");
        std::fs::create_dir_all(&subagents).unwrap();
        append(&subagents.join("agent-a0123456789abcdef.jsonl"), SUBAGENT);
        std::fs::write(subagents.join("agent-a0123456789abcdef.meta.json"), "{}").unwrap();
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("Stop", &transcript));

        let (tokens, context, _) = usage(reader.poll());
        assert_eq!(
            tokens,
            TokenCounts { input: 10, output: 150, cache_read: 12_000, cache_write: 32_300 }
        );
        assert_eq!(context, Some(12_302));
    }

    #[test]
    fn refuses_a_transcript_outside_claude_codes_folder() {
        let (dir, _) = projects();
        let elsewhere = tempfile::tempdir().unwrap();
        let transcript = elsewhere.path().join("t.jsonl");
        append(&transcript, NORMAL);
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("Stop", &transcript));
        assert!(matches!(reader.poll(), Some(Entry::UsageUnavailable { .. })));

        // Not even through `..`.
        let (dir, inside) = projects();
        let escape = inside.parent().unwrap().join("..").join("..").join("..").join("t.jsonl");
        let mut escaped = new_reader(&dir);
        escaped.on_hook(&hook("Stop", &escape));
        assert!(matches!(escaped.poll(), Some(Entry::UsageUnavailable { .. })));
    }

    #[test]
    fn a_corrupt_line_makes_usage_unavailable_for_good() {
        let (dir, transcript) = projects();
        append(&transcript, NORMAL);
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("Stop", &transcript));
        usage(reader.poll());

        append(&transcript, "{\"type\":\"assistant\",\"mess\n");
        assert!(matches!(reader.poll(), Some(Entry::UsageUnavailable { .. })));
        append(&transcript, AFTER_COMPACT);
        assert_eq!(reader.poll(), None, "stays unavailable");
    }

    #[test]
    fn a_missing_transcript_counts_only_once_claude_answered() {
        let (dir, transcript) = projects();
        let mut reader = new_reader(&dir);
        reader.on_hook(&hook("UserPromptSubmit", &transcript));
        assert_eq!(reader.poll(), None, "not written yet");
        reader.on_hook(&hook("Stop", &transcript));
        assert!(matches!(reader.poll(), Some(Entry::UsageUnavailable { .. })));
    }

    #[test]
    fn projects_dir_follows_claude_config_dir() {
        let env: Environment = [("HOME", "/home/a")].into_iter().collect();
        assert_eq!(projects_dir(&env), Some(PathBuf::from("/home/a/.claude/projects")));
        let env: Environment =
            [("HOME", "/home/a"), ("CLAUDE_CONFIG_DIR", "/cfg")].into_iter().collect();
        assert_eq!(projects_dir(&env), Some(PathBuf::from("/cfg/projects")));
    }
}
