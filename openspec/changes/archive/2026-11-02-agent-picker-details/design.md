# Design

## Context

See proposal.md for motivation, and the `session-hosting` delta for the behavior.

What is there today:

- `AgentId` (`crates/protocol/src/agent.rs`) lists the agents in display order with `display_name` and `has_timeline` (Claude Code only). Status reports come from Claude Code and omp (Program Status Protocol, `agent-status-signals`); OpenCode reports nothing.
- `crates/core/src/agent.rs` finds each agent's executable on the login shell's `PATH` (`program`, `detect`); Terminal is the login shell (`$SHELL`, else the platform's usual shells), PowerShell on Windows.
- `profile()` in `app/src-tauri/src/lib.rs` builds `AgentInfo { id, name, installed, has_timeline }` from `detect` every time the profile is read, which is on every settings change. It does not keep the executable's path.
- `StartScreen.tsx` renders `AgentChoice` buttons (`role="radio"`) in a three-column grid (`.agents` in `styles.css`), with one detail line; a missing agent is a disabled button with a tooltip.
- The app has no way to open a web page: no opener or shell plugin, no external links in the UI.
- The installed agents here print their versions as `2.1.309 (Claude Code)`, `opencode v2.2.5` and `omp/18.15.0`.
- The start screen must be usable within 1.5 s of a cold start (`native-experience`, Resource footprint).

## Goals / Non-Goals

**Goals:**

- One place per agent for what the picker says about it, so that adding Codex, Grok and pi next is a few lines each.
- The picker is complete and usable before any version is known.

**Non-Goals:**

- Checking whether an agent is logged in, or its version is recent enough.
- A version for Terminal. Shell versions do not help the host choose, and PowerShell is slow to start.
- Brand logos.

## Decisions

### 1. What an agent offers lives in `AgentId`

`AgentId` gets `reports_status()` (Claude Code, omp) next to `has_timeline()`, and `crates/core/src/agent.rs` gets `install_page(agent) -> Option<&'static str>`:

| Agent | Install page |
|---|---|
| Claude Code | `https://code.claude.com/docs/en/setup` |
| OpenCode | `https://opencode.ai/docs/` |
| omp | `https://omp.sh` |
| Terminal | none |

`AgentInfo` gains `path: Option<String>` (the executable, or Terminal's shell), `reports_status: bool` and `install_page: Option<String>`. The UI turns these into labels; icons are chosen in the UI by id (Lucide: `Sparkles` for Claude Code, `Braces` for OpenCode, `Bot` for omp, `SquareTerminal` for Terminal, `Bot` for an id it does not know).

- **Alternative: keep the labels and links in the UI only.** The UI already has `AGENT_NAMES`, but the facts (timeline, status, where to install) are about the agents, which Rust already describes; one place there keeps the next change small.

### 2. Versions come from a separate command, cached per executable

A new `agent_versions` command returns `{ agent, version }` for each installed agent except Terminal. The start screen calls it after it renders and fills the versions in when they arrive. The profile stays fast and unchanged in shape apart from decision 1.

Each version is read by running `<executable> --version` with the login shell's environment, stdin closed, and a time limit of 5 s, after which the process is killed. The first version-like token of the output (`digits.digits`, optionally more `.digits` and a `-`/`+` suffix) is the version; `v` and names around it are dropped. Nothing found, an error or the time limit means no version.

Results are cached in `AppState` by executable path and its modification time, so the start screen pays once per run and again only after the agent was updated. The agents are read in parallel.

- **Alternative: read versions inside `profile()`.** It would make every settings change and every start-screen render wait for up to six processes.
- **Alternative: read version files from install directories.** Every installer lays them out differently; `--version` is what each agent supports.

### 3. Install pages open through Rust, from a fixed list

`tauri-plugin-opener` is added and used from Rust only: a command `open_install_page(agent)` looks up the agent's page from decision 1 and opens it. The webview gets no opener permission, so the UI cannot open arbitrary addresses.

- **Alternative: show an install command to copy.** The commands differ per platform and installer (Homebrew, npm, scripts), and change more often than the pages that list them.

### 4. Rows for installed agents, links for missing ones

The picker lists only installed agents and Terminal, as rows (still a `radiogroup` of `role="radio"` buttons) with a radio mark. Agents that are not installed follow on one line under "Not installed", as small links to their install pages:

```
+--------------------------------------------------------------+
| (*) [icon]  Claude Code                              2.1.309 |
|             Timeline, diffs and plans · Status               |
|             ~/.local/bin/claude                              |
+--------------------------------------------------------------+
| ( ) [icon]  Terminal                                         |
|             Your shell                                       |
|             /bin/zsh                                         |
+--------------------------------------------------------------+
Not installed  (omp ->)  (Codex ->)
```

The path is shortened with `~` for the home folder and truncated at the start when it does not fit, with the full path in the tooltip. The not-installed line is a labelled group of buttons, so keyboard and screen reader users reach each install page.

- **Alternative (first version): a disabled row per missing agent, with an install button after it.** The button sat outside the row and broke its edge, and greyed-out rows took room from the agents the host can actually choose. Changed after review.

## Risks / Trade-offs

- [An agent's `--version` starts something heavier than expected, or prompts] → stdin closed, 5 s limit, killed after; read once per executable and run.
- [Install page addresses move] → One table in `agent.rs`; a task checks that each opens the install instructions.
- [A new dependency] → `tauri-plugin-opener` is maintained by the Tauri project, like the plugins the app already uses, and is only reachable from Rust.
- [Seven rows later may push the recent folders down] → Rows are compact (three short lines); revisit with the next change if the start screen needs to scroll.

## Migration Plan

Nothing to migrate: `AgentInfo` is only exchanged between the app's own Rust and UI.
