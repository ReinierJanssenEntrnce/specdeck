# Proposal

## Why

SpecDeck's viewers are not only developers. Product owners and business developers watch sessions live too. They care about what was asked, what Claude did and what it concluded, and above all about the OpenSpec documents it writes. Today the screen is mostly terminal, tool names, paths and diffs. The documents they want to read reach them only as diffs.

## What Changes

- Each viewer chooses how to follow a session with two remembered settings: **show the terminal** (on or off), and **steps** as a plain-language **summary** or technical **details**. The status bar stays in both.
- The first time someone joins a session, SpecDeck asks how they want to follow sessions: **Overview** (terminal off, summary) or **Developer** (terminal on, details). The choice is remembered and can be changed any time under **View**.
- With the terminal off, the conversation fills the window. A live line says what Claude is doing right now ("Claude is running the test suite · 12 s") or what it waits for.
- Summary steps describe what Claude did in words: Claude's own description of each shell command, "Looked at 3 files", "Changed 2 files", whether it succeeded or failed. Each summary line opens to the technical detail.
- **Readable documents:** when Claude creates or changes a Markdown file, everyone can read the whole document, formatted, with the parts changed since they last read it highlighted. To make this possible, the host sends the full current text of changed documents, not only their diff.
- **The plan card:** when Claude writes or changes an OpenSpec change, the conversation shows that change as a plan. Its documents come in the order and with the names of the change's own schema, so custom steps such as a feature brief appear without changes to SpecDeck. Task progress ("2 of 8 done") follows the change's tasks live.
- Sessions without a timeline (OpenCode, omp and terminal sessions) keep the terminal shown, with a note saying why.
- Not in this change: a product owner hosting without the terminal (a prompt box and answering Claude's questions in the conversation) follows as its own change, after a spike into running Claude Code headless.

## Capabilities

### New Capabilities

- `plan-documents`: reading the documents Claude writes, and showing OpenSpec changes as plans with their documents and task progress.

### Modified Capabilities

- `viewer-experience`: view settings per person (terminal, step detail), the first-join question, the conversation layout with the live line, and plain-language steps.
- `session-timeline`: the host sends the full text of changed documents and the state of OpenSpec changes, within the existing rule that only files Claude touched are shared.

## Impact

- `crates/protocol`: new timeline entries for document text and plan state.
- `crates/core`: reading changed documents (from disk, or from the shell-change snapshot) and asking the OpenSpec CLI for a change's documents and tasks, both on the host.
- `app/src-tauri`: two view settings and whether the first-join question was answered.
- `app/ui`: the View menu, the first-join question, the conversation layout and live line, summary steps, the plan card and the document reader.
- `docs/user-guide.md`: following a session as a viewer, reading plans.
- No new dependencies. The OpenSpec CLI on the host is optional: without it, SpecDeck recognises OpenSpec's default layout.
