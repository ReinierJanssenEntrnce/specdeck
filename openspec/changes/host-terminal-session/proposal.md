# Proposal

## Why

Hosts sometimes want to share a plain terminal, for example to walk viewers through a build, a deployment or a debugging session, without a coding agent in between. Today every hosted session starts Claude Code, OpenCode or omp, so the only way to do that is through an agent's shell escape.

## What Changes

- The host can choose **Terminal** next to the coding agents. It starts the host's own shell in the chosen folder: the login shell on macOS and Linux, PowerShell on Windows.
- Terminal is always offered: it needs nothing installed beyond the operating system.
- A terminal session is shared like an OpenCode or omp session: live terminal, late joiners, read-only viewers, invite codes, and a status bar with the folder and git state. It has no timeline, no touched files and no usage, and its activity shows as unknown.
- When Terminal is selected, the start screen reminds the host that viewers see all output, including any secrets a command prints.
- When no coding agent is installed, the start screen still offers Terminal and says that agents need to be installed for a timeline.
- The wire protocol's agent identifier gains a `terminal` value. 1.x has not been released, so no compatibility handling is needed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-hosting`: the host chooses between the coding agents and a plain terminal; a terminal session starts the host's shell; Terminal is always available.
- `session-timeline`: Terminal joins OpenCode and omp as a session without a timeline, worded so that it does not call a shell an agent whose events are "not captured yet".
- `viewer-experience`: the status bar and session status for a terminal session behave as for the terminal-only agents.

## Impact

- `crates/protocol` (`AgentId`), `crates/core` (agent adapter: shell lookup and arguments per platform), `app/src-tauri` (agent list, tray and notification texts that name the agent), `app/ui` (start screen choice and reminder, empty-state texts in Timeline and Files).
- `docs/user-guide.md` (hosting a terminal session).
- No new dependencies.
