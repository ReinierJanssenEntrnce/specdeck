# Proposal

## Why

Hosts who use Codex, Grok or pi have to start them by hand inside a Terminal session today. They then lose the agent's name in the session and its install hint on the start screen, and their viewers are told it is a plain shell. All three are terminal agents that SpecDeck can run the same way as omp, and the status code already knows their names (`agent-status-signals` deferred adding them as agents).

## What Changes

- Three new agents in the picker: **Codex** (`codex`), **Grok** (`grok`) and **pi** (`pi`). Each is found on the login shell's `PATH`, started in the chosen folder like OpenCode and omp, and gets an icon, labels, version, path and install page through `agent-picker-details`.
- Viewers see the terminal, and the agent's status (working, waiting, its message when it needs the host) for those that report it through the Program Status Protocol. None of them has a timeline.
- **BREAKING**: the session protocol goes from version 1 to 2, because the agent is part of the message that admits a viewer and older apps cannot read the new agents. A viewer on an older SpecDeck is told to update the app, for every session; a host on an older SpecDeck is told to update by a newer viewer.
- The docs and the start screen's "no coding agent installed" hint list the new agents.
- Not in this change: timelines for any agent but Claude Code, login checks, and Gemini or other agents.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-hosting`: "Choose a coding agent" lists Codex, Grok and pi among the coding agents.
- `session-sharing`: adds "Matching app versions", which writes down the existing version check this change relies on (a viewer is refused and told who needs to update).

## Impact

- `crates/protocol/src/agent.rs`: three `AgentId` variants (appended, so existing variants keep their encoding), placed before Terminal in `ALL` for display order, with `display_name` and `reports_status`.
- `crates/protocol/src/control.rs` and `crates/protocol/README.md`: `PROTOCOL_VERSION` 2.
- `crates/core/src/agent.rs`: executables and install pages. pi gets no Git Bash check on Windows, because it finds a bash in more ways than that check knows (design decision 4).
- `crates/core/examples/launch.rs`, `capture.rs`: the new ids.
- `crates/core/tests/fixtures/program-status/`: recordings of the agents that are installed for this change.
- `app/ui/src/api/types.ts`, `screens/HostScreen.tsx` (`AGENT_NAMES`), `screens/StartScreen.tsx` (icons, hint).
- `README.md`, `docs/site/src/content/docs/guide/first-steps.md`, `host.md`, `follow.md`, `troubleshooting.md`.
- Depends on `agent-picker-details` (rows, versions, install pages), which is implemented but not yet archived.
