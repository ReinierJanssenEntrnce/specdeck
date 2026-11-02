# Proposal

## Why

The working/waiting status is only known for Claude Code, and only from its hooks. omp shows "Live" with unknown activity, and a Terminal session where the host starts `claude` or `codex` by hand shows nothing at all. Even for Claude Code, the status says "working" while Claude is actually waiting for the host to answer a question or approve a plan, which is the moment viewers most want to see.

Coding agents now report their own state in the terminal output through the Program Status Protocol (OSC 7501): omp and Codex send it without being asked, and Claude Code (2.1.295 and later) sends it once the terminal answers its query. Orca, a desktop app for running several agents side by side, uses this as a fallback behind its hooks. Its notes and code were the starting point for this change.

## What Changes

- **Program status reports:** the host reads OSC 7501 reports from the session's output and answers the protocol's query, exactly once, like the other terminal queries the host already answers. A report of working, waiting or blocked becomes the session's status for agents without hooks (omp, OpenCode once it supports it) and for Terminal sessions while a program in them reports.
- **Requests for the host from reports:** a `blocked` report (permission, question, sign-in) becomes a notice that the agent waits for the host, carrying the program's own one-line message, such as "approve Write: src/main.rs". Like Claude Code's permission notices, it notifies the host when the window is in the background.
- **Hooks first:** for Claude Code, hooks keep deciding the status and the timeline. Program reports take over the status only while the timeline is unavailable (degraded mode), so a broken hook setup no longer freezes the status.
- **Questions and plans are waiting:** while Claude Code shows an `AskUserQuestion` question or an `ExitPlanMode` plan, the status is "waiting for the host", not "working".
- **Permission requests name the action:** SpecDeck also registers Claude Code's `PermissionRequest` hook, so the notice says what Claude wants ("Claude wants to use Bash: rm -rf build", "Claude asks: Which database?") instead of "Claude needs your permission", and the permission is linked to the right tool call.
- **Manual compact:** SpecDeck registers `PreCompact` and `PostCompact`. A manual `/compact` sends no prompt and no `Stop`, so today the status does not change and the timeline is marked unavailable after 10 seconds. With them, compaction shows as working and then waiting.
- MCP dialogs that open in a browser (`elicitation_url_dialog`) also count as waiting for the host, like the dialogs in the terminal already do.
- Not in this change: new agents (Codex, Gemini and others), timelines for agents other than Claude Code, guessing status from window titles or spinner characters, and installing hooks into the host's own Claude Code settings.

## Capabilities

### New Capabilities

- `agent-status`: how the host works out what the agent is doing: from hook events, from the program's own status reports, or not at all. Covers which source wins, when a reported state ends, and how report text is cleaned before anyone sees it.

### Modified Capabilities

- `viewer-experience`: Session status. Agents and terminal programs that report their state show working and waiting instead of unknown, and a request for the host shows its message.
- `native-experience`: Notifications. The host is also notified when an agent or program that reports its state waits for the host.
- `session-timeline`: a Claude Code permission request names the tool and what it would act on.

## Impact

- `crates/core`: a new module that scans output for OSC 7501 reports and terminal resets, and answers the OSC 7501 query. `activity.rs` combines hook and program status. `session.rs` feeds output to the scanner, always writes its query reply, and for Terminal sessions on Linux and macOS checks once a second whether the shell is back in the foreground. `hooks.rs` registers `PermissionRequest`, `PreCompact` and `PostCompact`. `timeline.rs` reads them.
- `crates/protocol`: no change. The status uses the existing `AgentStatus` values, and requests reuse the existing `Attention` timeline entry, so viewers on older app versions keep working. `PROTOCOL_VERSION` stays at 1.
- `app/ui`: the status pill shows the latest request message for sessions without a timeline, and labels Terminal sessions while a program reports.
- Dependencies: `base64` for report messages (already in the lockfile). The OSC parser comes from `vte`, which `alacritty_terminal` already provides.
- `docs/site`: the "Follow a session" guide says which agents report their status.
