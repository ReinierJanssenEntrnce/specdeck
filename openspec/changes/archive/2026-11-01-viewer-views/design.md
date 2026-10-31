# Design

## Context

The timeline already carries what most of this needs: prompts, live and final replies, tool calls with their input (a Bash call's input includes Claude's own `description`), results, file changes with hunks (also from shell commands, through the git snapshots of decision 10b in the archived `2026-10-31-native-shared-session-mvp`), status and attention entries. Viewers rebuild everything from the timeline, and late joiners get it by replay. Two things are missing: the full text of documents (an `Edit` only carries hunks), and which documents an OpenSpec change has (only OpenSpec knows its schema). The UI has a Markdown renderer (`react-markdown` with GFM) used for replies.

`host-terminal-session` (not yet archived) modifies the "Session status", "Status bar" and "Timeline support per agent" requirements. This change only adds requirements, so the two do not conflict.

## Goals / Non-Goals

**Goals:**
- Two per-person view settings, a first-join question, and the conversation layout, built in the UI on the existing timeline.
- Full document text and OpenSpec plan state as two new timeline entries, so live viewers and late joiners get them the same way.
- Work with any OpenSpec schema, including custom steps such as a feature brief.

**Non-Goals:**
- A product owner hosting without the terminal (a prompt box, answering Claude's questions in the conversation): a separate change, after a spike into headless Claude Code.
- Plan state for agents without a timeline, and showing archived changes.
- Reading untouched files: the "Touched files list" rule stays.
- Remembering "last read" across sessions or devices: it lives in the app for the session.

## Decisions

### 1. Two new timeline entries, beside `FileChange`

- `Document { path, text: Option<String> }`, emitted right after a `FileChange` of a Markdown file (`.md`, `.markdown`). `text` is `None` when the file is over 200 KB. A deleted document gets no `Document`: its `FileChange` says deleted.
- `Plan { change, documents: [{ id, name, paths, exists }], tasks: { done, total } }`, emitted when a file under `openspec/changes/<name>/` (not `archive/`) changed, and only when it differs from the last `Plan` for that change.

Separate entries rather than fields on `FileChange`: `FileChange` stays small for source code, and keeping every `Document` version is what "changed since you last read it" compares against. *Alternative*: send only the latest text on request. That would need a request channel from viewers to the host, which the protocol deliberately does not have (viewers cannot send anything that reaches the session).

### 2. Where the document text comes from

The host takes the text at the moment of the change, in the hook handler, before Claude Code continues:

- `Write`: the tool response's `content`, as file creation already uses.
- `Edit`/`MultiEdit`: the file read from disk on `PostToolUse`.
- Shell commands: the blob in the "after" snapshot tree (`git cat-file`), which is exactly what the diff was taken from.

### 3. Plan state from the OpenSpec CLI, with a fallback

When a change's file changed, the host runs, in the session folder with the session's environment and a 5-second limit, `openspec status --change <name> --json`, which gives the artifacts in schema order with their existing paths, and `openspec list --json`, which gives `completedTasks` and `totalTasks`. The calls run off the hook path and are debounced (300 ms), because `openspec` starts Node: a burst of writes during `/opsx:propose` costs one call. Without the CLI, or when it fails, the fallback recognises OpenSpec's default layout: proposal, specs (`specs/**/spec.md`), design and tasks, counting `- [x]` against `- [ ]` in `tasks.md` the way OpenSpec does.

Document names come from the artifact id, made readable ("feature-brief" becomes "Feature brief"). A glob artifact such as specs is one entry with several paths, shown as "Specs (3)". The plan's title is the proposal's first heading when the proposal exists, else the change name made readable. That is done in the UI from the `Document` entries.

### 4. View settings in the app's settings

`Settings` gains `view: Option<View>` with `terminal: bool` and `steps: Summary | Details`. `None` means the user never answered the first-join question, which is shown on the first join as a viewer. Hosts never get the question; for them `terminal` is ignored, and `steps` applies to the side panel.

### 5. The terminal keeps running while hidden

The viewer's terminal stays mounted and keeps writing output while hidden. It is moved out of view with its size kept (not `display: none`, which xterm cannot measure), so showing it again is instant, with full scrollback. *Alternative*: unmount and replay from the log on show. That is simpler, but slow for long sessions, which the spec rules out.

### 6. Summary steps are computed in the UI

A pure function in `model/` turns a turn's tool calls into summary lines:
- `Bash`/`PowerShell` with its `description` (else "Ran a command");
- runs of `Read` (also `Glob`/`Grep`) as "Looked at N files";
- runs of file changes as "Changed N files";
- changes under `openspec/changes/<name>/` as "Wrote the plan" (when the change is new in the session) or "Updated the plan";
- a subagent call (`Task`/`Agent`) by its description;
- anything else as "Used <tool>".

Each line takes its outcome from the results it groups (failed when any failed, running while any runs), and keeps the calls it stands for to show when opened. The live line is the summary of the running call, plus the elapsed time, or the latest `Attention` while the agent waits.

### 7. Marking what changed since last read

The reader splits a document into top-level blocks (headings, paragraphs, list items, code blocks) using the Markdown parser's syntax tree. It compares them with the version the participant last opened by a longest-common-subsequence over block text. Blocks only in the new version are marked **New**, edited blocks **Changed**, each with a label, not colour alone. "Last read" is the `Document` version each participant last had open, kept in the UI per path.

## Risks / Trade-offs

- [A document revised often makes the timeline grow, since every version is kept] → The 200 KB cap per document. Plans are a few documents of a few KB, but `/opsx:apply` rewrites `tasks.md` once per ticked task; if the replay measurements show that matters, send only the latest `tasks.md` within a prompt.
- [`openspec` is slow to start or missing from the GUI app's `PATH`] → It runs with the detected login-shell environment, debounced and off the hook path, with the default-layout fallback.
- [OpenSpec's JSON output changes] → The fallback covers the default layout; a parse failure logs and falls back instead of failing the session.
- [Claude's command descriptions are missing or technical] → They are written for humans by Claude Code itself; without one, the line says a command ran and the detail stays one click away.
- [Hidden terminal and screen readers] → The hidden terminal is removed from the accessibility tree (`aria-hidden`, `inert`) while hidden.
