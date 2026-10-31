//! The whole text of documents the agent changes, so participants can read them as documents and
//! not only as diffs (`viewer-views` design decision 2). Only Markdown files count, and only the
//! ones the agent itself created or changed.

use std::path::Path;

use serde_json::Value;

use crate::hooks::HookPayload;

/// Larger documents are announced without their text; their diffs are still sent.
pub const MAX_DOCUMENT_BYTES: u64 = 200 * 1024;

/// Claude Code's tools that change a file named by `file_path`.
const FILE_TOOLS: [&str; 3] = ["Write", "Edit", "MultiEdit"];

/// A changed document's text as it was right after the change; `None` when it is too large.
/// `path` is absolute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentText {
    pub path: String,
    pub text: Option<String>,
}

/// Whether participants can read `path` as a document.
pub fn is_document(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

/// The text of `path` on disk; `None` when it cannot be read (it was deleted, for example).
pub fn read(path: &str) -> Option<DocumentText> {
    let size = std::fs::metadata(path).ok()?.len();
    let text = if size > MAX_DOCUMENT_BYTES {
        None
    } else {
        Some(String::from_utf8_lossy(&std::fs::read(path).ok()?).into_owned())
    };
    Some(DocumentText { path: path.to_owned(), text })
}

/// The document one of Claude Code's file tools just changed, read from disk. Must run before the
/// hook is answered, so the text is the one the change produced.
pub fn from_file_tool(payload: &HookPayload) -> Option<DocumentText> {
    let text = |key: &str| payload.get(key).and_then(Value::as_str);
    if text("hook_event_name")? != "PostToolUse" || !FILE_TOOLS.contains(&text("tool_name")?) {
        return None;
    }
    let path = payload.get("tool_input")?.get("file_path")?.as_str()?;
    is_document(path).then(|| read(path)).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn post(tool: &str, path: &Path) -> HookPayload {
        json!({
            "hook_event_name": "PostToolUse", "prompt_id": "p", "tool_use_id": "t",
            "tool_name": tool, "tool_input": { "file_path": path }
        })
        .as_object()
        .unwrap()
        .clone()
    }

    #[test]
    fn markdown_files_are_documents() {
        assert!(is_document("openspec/changes/x/proposal.md"));
        assert!(is_document("/repo/README.MD"));
        assert!(is_document("notes.markdown"));
        assert!(!is_document("src/server.ts"));
        assert!(!is_document("Makefile"));
    }

    #[test]
    fn reads_the_text_an_edit_left_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("design.md");
        std::fs::write(&path, "# Design\n\nNew section.\n").unwrap();
        for tool in ["Write", "Edit", "MultiEdit"] {
            assert_eq!(
                from_file_tool(&post(tool, &path)),
                Some(DocumentText {
                    path: path.to_string_lossy().into(),
                    text: Some("# Design\n\nNew section.\n".into())
                })
            );
        }
    }

    #[test]
    fn source_files_reads_and_failures_give_no_document() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("server.ts");
        std::fs::write(&source, "export {}").unwrap();
        assert_eq!(from_file_tool(&post("Edit", &source)), None);
        let doc = dir.path().join("a.md");
        std::fs::write(&doc, "# A").unwrap();
        assert_eq!(from_file_tool(&post("Read", &doc)), None);
        let mut pre = post("Edit", &doc);
        pre.insert("hook_event_name".into(), json!("PreToolUse"));
        assert_eq!(from_file_tool(&pre), None);
        // A failed or deleted file: nothing to read.
        assert_eq!(from_file_tool(&post("Write", &dir.path().join("gone.md"))), None);
    }

    #[test]
    fn large_documents_come_without_text() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.md");
        std::fs::write(&path, "x".repeat(MAX_DOCUMENT_BYTES as usize + 1)).unwrap();
        assert_eq!(read(&path.to_string_lossy()).unwrap().text, None);
        std::fs::write(&path, "x".repeat(MAX_DOCUMENT_BYTES as usize)).unwrap();
        assert!(read(&path.to_string_lossy()).unwrap().text.is_some());
    }
}
