# Design

## Context

See proposal.md for why. The current state this design builds on:

- `crates/core/src/activity.rs` decides the status. It has two modes, chosen by `AgentId::has_timeline()`: Claude Code's status follows the timeline entries built from hooks. Every other agent, and Terminal, is `Unknown` until it exits (`set_status` forces `Unknown` when there is no timeline).
- `crates/core/src/mirror.rs` runs alacritty's `Term` over the output, so terminal queries get exactly one answer (design decision 5 of `native-shared-session-mvp`). The mirror's replies go to the agent only while no host window is attached (`Responder`). Otherwise the host's xterm.js answers through the host's input. alacritty silently drops OSC numbers it does not know, so OSC 7501 never reaches any handler today.
- Wire compatibility: control messages, including `HostMessage::Status { status }`, are postcard with exact decoding. Adding a field, or a variant to `AgentStatus`, breaks every viewer on an older version. Timeline events are JSON, but a viewer that cannot parse one treats the frame as malformed and asks for a resend (`viewer.rs` `handle_frame`), so a new `Entry` variant would make older viewers ask for resends without end. Optional fields on existing entries are safe (as in `show-host-interactions`).
- `app/src-tauri/src/host.rs` turns every `Entry::Attention` and `Entry::Response` into a host notification ("<agent> is waiting for you").
- The OSC 7501 spec: <https://www.superlogical.com/rex/docs/build/program-status>. Orca's implementation (`src/shared/program-status/`, `src/main/agent-hooks/README.md` in stablyai/orca) was read for reference. It lists which agents send reports and notes that Claude Code stops drawing its spinner character in the window title once the query is answered. SpecDeck does not use window titles.
- Claude Code hooks docs: an HTTP hook that answers 2xx with an empty body is "success, equivalent to exit code 0 with no output", so a `PermissionRequest` hook answered with the endpoint's empty 204 makes no decision. `PostCompact` input carries `trigger: manual | auto`.

## Goals / Non-Goals

**Goals:**
- One place (`Activity`) that combines hooks, program reports and process exit into the status, with a precedence rule that can be tested without a pty.
- No wire protocol change: older viewers keep working and just see better statuses.
- Reports from untrusted programs (anything the host runs in a Terminal session) cannot flood the timeline, the host's notifications or the viewers' screens.

**Non-Goals:**
- Showing report progress percentages, child records (subagents) or the `done`/`error` result after a program exits. Only the root record drives the status.
- Answering the OSC 7501 query from viewers, or a per-session setting to turn reports off.
- A timeline for any agent other than Claude Code.

## Decisions

### 1. A second, small parser instead of extending the mirror

A new module, `program_status.rs`, runs its own `vte::Parser` (re-exported by `alacritty_terminal`, so no new dependency) over the same output chunks as the mirror. It implements only `osc_dispatch` (OSC 7501) and `esc_dispatch` (RIS, `ESC c`). It produces protocol messages (query or report) and resets. Parsing and validating a report follows the spec's grammar and limits: `key=value` pairs separated by `:`, base64 `msg`/`title`, a 4096-byte cap on the sequence (checked on the raw OSC bytes, since `vte` with `std` buffers OSC without a limit), unknown keys ignored, unknown states rejected.

Alternatives: alacritty's `Term` exposes no hook for unknown OSC numbers. A hand-written byte scanner, as Orca has, would have to handle chunk boundaries, BEL vs ST and C1 controls again, all of which `vte` already does.

### 2. The host core always answers the OSC 7501 query

The session writes the query reply to the pty itself, whether or not a host window is attached, outside `Responder`. xterm.js ignores OSC 7501, so this is still exactly one answer. The reply is `ESC ] 7501 ; ? : states=idle,working,done,blocked,error : kinds=permission,question,auth ESC \`.

This also gets the order right. Programs may detect support by sending `CSI c` after the query and treating a device-attributes answer that arrives first as "unsupported". The core's reply leaves before the host window's device-attributes answer (which goes through the UI), and before the mirror's (the scanner runs first on each chunk).

Alternative: register an OSC 7501 handler in the host's xterm.js and let the mirror answer only without a window, like the other queries. That is more code in two places for the same result, and the reply would wait on the UI round trip.

### 3. `Activity` combines three sources

```
           hooks (Claude Code)      program reports (OSC 7501)      process exit
                   |                          |                          |
                   v                          v                          v
   +-----------------------------------------------------------------------------+
   | Activity                                                                    |
   |   hook_status     : status from timeline entries (as today)                 |
   |   pending_waits   : open AskUserQuestion / ExitPlanMode calls               |
   |   program         : root record state, or none                              |
   |                                                                             |
   |   status = Exited                       if the process exited               |
   |          = hook status                  if hooks + timeline available       |
   |          = program state (mapped)       else if a report is in force        |
   |          = hook status / Starting       else if the agent has hooks         |
   |          = Unknown                      otherwise                           |
   +-----------------------------------------------------------------------------+
```

Mapping: `working` → `Working`; `idle`, `done`, `blocked`, `error` → `Waiting`. `AgentStatus` keeps its five values. Terminal sessions are `Unknown` with no report in force, and the UI already shows that as "Live" for a terminal.

New inputs: `on_report(report, now)`, `on_program_ended()` (reset, shell back in front) and `on_compact(started)` (manual compact: `true` sets `Working`, `false` sets `Waiting`). `on_entries` additionally tracks question and plan calls: a `ToolCall` for `AskUserQuestion` or `ExitPlanMode` adds its `tool_use_id` to `pending_waits` and sets `Waiting`, and the matching `ToolResult` removes it and sets `Working`. These calls never come from helpers.

Alternative: a separate status tracker per source, merged in `session.rs`. Rejected because the degraded-mode rule (timeline available or not) already lives in `Activity`, and precedence depends on it.

### 4. Requests reuse `Entry::Attention`, status reuses `AgentStatus`

A `blocked` or `error` report creates `Entry::Attention { message }`. That gives host notifications (`host.rs`), the timeline notice for Claude Code in degraded mode, and older viewers' handling for free. The message is built as:

| Report | Message |
|---|---|
| `blocked`, `kind=permission` | "<name> needs permission: <msg>" |
| `blocked`, `kind=question` | "<name> asks: <msg>" |
| `blocked`, `kind=auth` | "<name> needs you to sign in: <msg>" |
| `blocked`, no kind | "<name> waits for you: <msg>" |
| `error` | "<name> stopped with an error: <msg>" |

Without a usable `msg`, the text after the colon is left out. `<name>` is the agent's display name for agent sessions. In a Terminal session it is the report's `app` when it is a known id (`claude-code` → "Claude Code", `omp`, `codex` → "Codex", `grok`, `pi`), else "A program".

Dedup and rate limit, in `Activity`: an Attention is created only when the (state, kind, msg) triple differs from the last one that created a notice, and at most one every two seconds. The status itself is never rate-limited. While hooks decide the status (decision 3), reports create no notices: Claude Code's own `Notification` hook already does.

Alternatives: optional fields on `HostMessage::Status`, or a new `Entry::ProgramStatus`. Both break older viewers (see Context).

### 5. Cleaning report text

In `program_status.rs`, before a message leaves the module: reject the message (keep the report) if the decoded text has C0, DEL or C1 controls. Remove zero-width and text-direction characters (U+200B–U+200F, U+202A–U+202E, U+2060–U+2064, U+2066–U+2069, U+FEFF). Cut to 300 characters on a character boundary, with "…". The `app` value is limited to the spec's grammar (`[A-Za-z0-9_.+-]`, 32 bytes) and only used through the known-id table.

### 6. When a reported state ends

The program's record is cleared on:
- `state=clear` without an id. The recordings (`crates/core/tests/fixtures/program-status/`) show that Claude Code and omp both send it when they exit.
- process exit (agent sessions; also ends the session status as `Exited`)
- RIS (`ESC c`)
- Terminal sessions on macOS and Linux: in the existing once-a-second tick, `MasterPty::process_group_leader()` equals the shell's pid (`Child::process_id()`) after having been different. That means the program in front has exited, also when it was killed before it could clear its report.

OSC 133 `A` (a prompt starts) does not clear the record, although the protocol spec says it should. omp writes prompt marks for its own commands while it runs (see the omp recording), so honouring them would make omp's status flicker between working and live.

Unlike the spec, which keeps `done` and `error` records after the program exits, SpecDeck clears all states here. It shows a status, not a list of results, and "Waiting" after Claude Code has exited would be wrong.

Windows has no foreground-process query on ConPTY. There, a Terminal session clears only on `clear`, reset or shell exit (see Risks).

### 7. Claude Code hooks: `PermissionRequest`, `PreCompact` and `PostCompact`

`hooks.rs` adds all three to `HOOK_EVENTS`, and `PermissionRequest` to the events that take a tool matcher. The endpoint keeps answering an empty 204, which the hooks docs define as no decision (and for `PreCompact`, as not blocking the compaction).

What the recordings show (`claude-hooks/permission-session`, `question-session`, `plan-session`, `compact-session`):
- `PermissionRequest` arrives after the call's `PreToolUse` and before the `permission_prompt` `Notification`. It carries `tool_name` and `tool_input` but no `tool_use_id`.
- `AskUserQuestion` and `ExitPlanMode` send a `PermissionRequest` and a "Claude needs your permission" notice too.
- A manual `/compact` sends no `UserPromptSubmit` and no `Stop`, only `PreCompact` and then `PostCompact`, both with `trigger: "manual"`. Without them, the host's Enter starts the degraded-mode timer and nothing arrives while compaction runs, so the timeline is marked unavailable after 10 seconds.

`timeline.rs`:
- `PermissionRequest { tool_name, tool_input }`: find the newest open call of the main conversation with the same tool name and input, and emit one `Attention`:
  - for a tool, mark the call `awaiting_permission`, with the message "Claude wants to use <tool>: <summary>". The summary is the input's `command`, `file_path`, `path`, `url` or `pattern`, cut to 200 characters, else nothing.
  - for `AskUserQuestion`, "Claude asks: <first question's text>" (plus " (and N more)" for several), without marking it. These two ask for permission to show their own dialog, as today.
  - for `ExitPlanMode`, "Claude has a plan for you to review", without marking it.

  The call remembers that it has a notice, and the `permission_prompt` `Notification` that follows creates no second one. A `Notification` without a preceding `PermissionRequest` works as today and also marks the call as having a notice, so a late `PermissionRequest` adds none. If no open call matches, the old "last open call" rule applies.
- `PreCompact` and `PostCompact` with `trigger: "manual"`: no entry. `session.rs` calls `Activity::on_compact(true)` and `on_compact(false)`. Both count as hook events for the degraded-mode timer. Automatic ones are ignored.
- `ATTENTION_TYPES` gains `elicitation_url_dialog`.

### 8. UI

`StatusPill` takes an optional `request` text. Host and viewer screens keep the message of the latest `Attention` entry and drop it when the status leaves `waiting`. They pass it to the pill only when the session has no timeline, because the timeline shows it otherwise. For a terminal session, `working` and `waiting` get the labels "Working" and "Waiting for input". Today it only ever has `unknown` and `exited`. The UI must keep receiving timeline events for sessions with `timeline_supported: false`, which a task checks.

## Risks / Trade-offs

- [Answering the query changes how Claude Code draws, beyond the title spinner] → The recordings with and without the answer differ only in the window title (spinner) and the reports.
- [A program in a Terminal session spams reports] → Notices are deduplicated and limited to one every two seconds, report text is cleaned and cut, and the parser drops oversized sequences. The status still changes as fast as reports come, and costs nothing on the wire beyond a small control message.
- [Report text shows things the host did not mean to share] → Viewers already see all terminal output, and the start screen already warns about it for Terminal sessions. A report adds no information that was not already in the output.
- [On Windows, a Terminal session keeps the last reported state when a program exits without sending `clear` (for example when it is killed)] → Accepted. Claude Code and omp send `clear` on a normal exit, so it only happens after a crash or kill, and the next report or a reset replaces it.
- [A later Claude Code version sends the `Notification` before the `PermissionRequest`] → The `Notification` then creates the notice as today. A `PermissionRequest` that finds its call already has a notice only marks the call, so there is still one notice (decision 7).
- [A viewer who joins, or a host window that reopens, while the agent waits takes the latest notice in the timeline as the request, even when that notice is from an earlier wait] → Accepted. Without a protocol change the status cannot say which notice it belongs to. A live participant clears the request as soon as the status leaves waiting.
- [Two parsers run over all output] → `vte` is a table-driven state machine. The work is small next to alacritty's `Term`, which already processes every byte.

## Migration Plan

Nothing to migrate. The protocol is unchanged, so host and viewer versions can be mixed. An older viewer shows the improved statuses and ignores nothing. Rollback is a revert.

## Open Questions

- Should Codex, grok and pi get their own `AgentId` now that they would have a status for free? Deferred by the user. Until then they run in a Terminal session and get a status there.
