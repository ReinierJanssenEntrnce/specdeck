# Design

## Context

See proposal.md for motivation, and the `session-hosting` and `session-sharing` deltas for the behavior.

What is there today:

- `AgentId` (`crates/protocol/src/agent.rs`) has `ClaudeCode`, `OpenCode`, `Omp`, `Terminal`; `ALL` is the display order. After `agent-picker-details` it also answers `reports_status()`, and `crates/core/src/agent.rs` has each agent's executable (`program`) and install page.
- Agents without a timeline get their status from the Program Status Protocol in their output (`Activity` in `crates/core/src/activity.rs` reads reports for any agent that is not Claude Code), so a new agent gets a status with no new code if it sends reports. `program_status::app_name` already names `codex`, `grok`/`grok-build` and `pi`.
- Control messages are postcard (`crates/protocol/src/control.rs`), which encodes an enum variant by its index. `HostMessage::Welcome` carries the session's `AgentId`, so an app that does not know a variant cannot read the message that admits it. `PROTOCOL_VERSION` is 1, and `check_version` already refuses mismatched viewers with `ViewerOutdated` / `HostOutdated`, shown as "Update your app to join this session." and "The host needs to update their app."
- None of Codex, Grok or pi is installed on the machine this change was planned on, so their output has not been recorded. The `agent-status-signals` proposal states that Codex sends status reports without being asked; Grok and pi are unconfirmed.
- From the agents' install guides: Codex installs as `codex` (npm, Homebrew cask, or an installer script), Grok as `grok` (`x.ai/cli` installer, macOS, Linux, Windows), pi as `pi` (repository now `earendil-works/pi`, formerly `badlogic/pi-mono`). pi runs its bash tool through a bash on native Windows (see decision 4).

## Goals / Non-Goals

**Goals:**

- Each new agent is a few lines in the places that describe agents, with no agent-specific code in sessions or streaming.
- Older apps fail with a message that says what to do, never with a generic connection error.

**Non-Goals:**

- Timelines for these agents.
- Supporting `grok-build` as a second executable name for Grok. The program-status name `grok-build` stays known for Terminal sessions.
- A compatibility path for older viewers (decided by the user: bump the protocol).

## Decisions

### 1. Variants appended, display order from `ALL`

`Codex`, `Grok` and `Pi` are appended to the enum after `Terminal`, so the encoding of the existing variants does not change (stored `last_agent` is JSON and unaffected either way). `ALL` puts them before Terminal: Claude Code, OpenCode, omp, Codex, Grok, pi, Terminal. Wire names are `codex`, `grok` and `pi`, display names "Codex", "Grok" and "pi", matching `program_status::app_name`.

| Agent | Executable | Install page | Icon (Lucide) |
|---|---|---|---|
| Codex | `codex` | `https://developers.openai.com/codex/cli` | `CodeXml` |
| Grok | `grok` | `https://x.ai/cli` | `Zap` |
| pi | `pi` | `https://github.com/earendil-works/pi` | `Pi` |

`https://x.ai/cli` refuses scripted requests (a Cloudflare check), so it is checked in a browser; the Codex page redirects to OpenAI's current docs, which keeps the address valid if the docs move again.

### 2. Status claims come from recordings

`reports_status()` is true for an agent only when a recording shows it sending reports. Codex starts as true on the earlier change's finding and is confirmed by a recording; Grok and pi start as false and are switched to true if their recordings show reports. Recordings go into `crates/core/tests/fixtures/program-status/` with the same method and README as the existing ones (`agent-status-signals`, task 1.1), with printed text replaced. An agent that cannot be installed for this change keeps `false` and the README says it was not recorded.

Outcome (recorded 2026-11-02): all three report. Codex reports without being asked and is idle again after a turn; pi and Grok answer the query; Grok also reports its folder trust question as a permission request. `reports_status()` is true for all three.

- **Alternative: trust the protocol's app list.** Being named in `app_name` only means SpecDeck would show the name if a report arrived; it says nothing about whether the agent sends one, and the picker would then promise a status viewers never see.

### 3. Protocol version 2

`PROTOCOL_VERSION` becomes 2, and the protocol README's version line says why. `Hello` and `Refused` do not change, so an older viewer still reads the refusal and shows "Update your app to join this session.", and an older host refuses a newer viewer with `HostOutdated`. This applies to every session, also those that run an agent older apps know; the troubleshooting page already explains the message.

- **Alternative: keep version 1.** Older viewers would keep working for the old agents but fail on the new ones with a malformed-message error.
- **Alternative: send Terminal to older viewers.** The host does not know a viewer's app version, only its protocol version, so this would need the version bump anyway.

### 4. No prerequisite check for pi on Windows

pi runs its bash tool through a bash on Windows, but it finds one in several ways (`dist/utils/shell.js` in pi 1.4.0): Git Bash in the usual folders, any `bash.exe` on `PATH` (MSYS2, Cygwin, the WSL launcher), or a shell path set in its own settings. SpecDeck's check only knows Git Bash, so using it for pi would refuse hosts whose pi works. pi says itself what to install when it finds no bash; the docs name the options.

- **Alternative (planned first): Git Bash as pi's prerequisite, as for Claude Code.** Dropped after reading pi's shell lookup, as this decision allowed.

## Risks / Trade-offs

- [Command names or install pages differ from what the guides say] → Each is checked against the agent's own page in the tasks, and the table in decision 1 is the one place to change.
- [Everyone must update at once] → Auto-update downloads the new version in the background; the refusal says who needs to update. Release notes call out the protocol change.
- [An agent cannot be installed here, for example Grok needing a subscription] → It still runs (start and stop are the same for any agent); only its status claim stays off until someone records it.

## Migration Plan

Release host and viewer apps together as one version. Rolling back means releasing protocol 1 again; hosts and viewers then have to be on that version again too.
