# Tasks

## 1. Host: document text and plan state

- [x] 1.1 Add the `Document` and `Plan` timeline entries to `crates/protocol` (design decision 1); verify their wire format with protocol tests, including `text: null` for a document over 200 KB
- [x] 1.2 Emit `Document` after each `FileChange` of a Markdown file, with the text from the `Write` response, from disk after `Edit`/`MultiEdit`, and from the "after" snapshot for shell changes (decision 2); verify with timeline tests for each source, the 200 KB cap, deletion (no `Document`), and that a source file gets no `Document`
- [x] 1.3 Emit `Plan` for changes under `openspec/changes/<name>/` (decision 3): the OpenSpec CLI with a 5 s limit, debounced, off the hook path; the default-layout fallback; only when it changed. Verify with tests: a fake `openspec` returning a custom schema with a feature brief first, no `openspec` on `PATH` (fallback with ticked and open tasks), a failing CLI (fallback), archive paths ignored, and one CLI call for a burst of writes
- [x] 1.4 Verify end to end in `crates/core/tests/host_session.rs`: a fake `claude` writes a proposal and edits a tasks file through hooks; the timeline has the documents' full text and two `Plan` entries with the task count going up

## 2. View settings and the first-join question

- [ ] 2.1 Add the view settings to the app (decision 4): stored, returned with the profile, and a command to set them; verify with settings tests (default `None`, round trip) and an app test of the command
- [ ] 2.2 Add the View menu (terminal on/off for viewers, summary/details for everyone, "show the first-join question again") and the first-join question for viewers whose view is not set; verify with UI tests for both answers, remembering, asking again, the host never being asked, and the a11y test
- [ ] 2.3 Hide the viewer terminal without unmounting it (decision 5), and keep it shown with a note for sessions without a timeline; verify with UI tests that output written while hidden is there when shown, that the hidden terminal is out of the accessibility tree, and the note for an OpenCode session

## 3. Conversation

- [ ] 3.1 Implement summary steps (decision 6) as a pure model function; verify with unit tests for each rule: Bash with and without description, grouped reads and changes, plan written vs updated, subagent, other tools, failed and running outcomes
- [ ] 3.2 Show the conversation layout with summary or detail steps, steps that open to their detail, and the live line (running step with elapsed time, or the attention message); verify with UI tests on a timeline fixture for both settings and the live line in both cases

## 4. Plans and documents

- [ ] 4.1 Add the document reader: formatted Markdown, close at any time, "too large" with diffs offered, changes since last read marked New or Changed with labels (decision 7) and a switch to hide the marks; verify with unit tests for the block comparison and UI tests for first read (no marks), a revised section, per-participant last read, and the large-document case
- [ ] 4.2 Add the plan card: documents in schema order with "changed" markers, task progress, live updates, several plans newest first; verify with UI tests on `Plan` fixtures for the default schema, a custom schema with a feature brief, and a task tick
- [ ] 4.3 Link plan steps to the reader ("Read the plan", "Read the changes"), opening on the documents that step changed; verify with a UI test

## 5. Documentation and integration

- [ ] 5.1 Update `docs/user-guide.md`: following a session as a viewer (the two views, the first-join question, the View menu), reading documents and plans; verify each step against the running app
- [ ] 5.2 Run a real `/opsx:propose` and `/opsx:apply` in a hosted session with an Overview viewer and a Developer viewer; check the plan card, the reader with marks after a revision, task progress during apply, the live line, and switching the terminal on and off; record the result in this task
