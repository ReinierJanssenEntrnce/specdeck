# Design

## Context

Hosting already separates "what runs" from "how it is shared". The agent adapter (`crates/core/src/agent.rs`, decision 3a of the archived `2026-10-31-native-shared-session-mvp`) maps an `AgentId` to an executable on `PATH` and its prerequisites. The launcher runs that executable in a pty with the login-shell environment from `Environment::detect`. Everything after that (output log, mirror, streaming, git state) never looks at the agent. Agents without captured events (`has_timeline() == false`) already run as a terminal mirror with status `unknown`, which the UI shows as "Live".

## Goals / Non-Goals

**Goals:**
- Add a terminal session through the existing adapter seam, with no change to the session engine, sharing or streaming.
- Pick a shell that matches what the host gets from their own terminal app on each platform.

**Non-Goals:**
- Choosing a shell or a profile in the app (for example cmd, Git Bash or nushell on Windows); the default per platform is enough for now.
- Several terminals or tabs per session, and a timeline or command list for shell sessions.
- Detecting or redacting secrets in the output: the reminder on the start screen is the mitigation.

## Decisions

### 1. Terminal is a fourth `AgentId`

`AgentId::Terminal` (`"terminal"` on the wire), with `has_timeline() == false` and display name "Terminal". Every place that already handles a terminal-only agent then handles it unchanged: no hooks, the timeline-unavailable views, status `unknown`, and the git watcher.

*Alternative*: a separate session kind next to `AgentId`. That would need a second branch in the launcher, the session engine, the protocol's session info and the UI, for no behavior difference. The name "agent" is slightly off for a shell. User-facing texts say "Terminal" or "session" wherever they would otherwise say "the agent" (decision 4).

### 2. Which shell, per platform

The adapter resolves the shell instead of looking up a fixed executable name:

- **macOS and Linux**: `$SHELL` from the detected environment if it names an executable file; otherwise `/bin/zsh` on macOS, then `/bin/bash`, then `/bin/sh`. It starts as a login shell (`-l`). macOS's Terminal and most Linux terminals start login shells, so the host gets their profile, prompt and aliases. `-l` is accepted by sh, bash, zsh and fish.
- **Windows**: `pwsh.exe` (PowerShell 7) when it is on `PATH`, otherwise `powershell.exe` (Windows PowerShell, part of Windows), with `-NoLogo`. This is the default in Windows Terminal.

`detect` reports Terminal as installed whenever a shell resolves, which is always on supported systems. `missing_prerequisite` has nothing for Terminal. The launcher gets the resolved executable and arguments from the adapter instead of `executable_name` alone. Its error for an unresolvable shell reuses `NotInstalled` ("Terminal is not installed …" would be wrong), so it gets its own message: "No shell was found."

*Alternative*: always `/bin/sh` or `cmd.exe`. That is simpler, but the host would not get their own prompt, history and aliases, which is what makes a shared terminal useful.

### 3. Environment and markers

The shell gets the same environment as an agent: the detected login environment without the parent Claude Code session markers, plus `TERM` and `COLORTERM`. No hooks or status-line settings are written, because `has_timeline()` is false. The markers matter here too: a host who starts `claude` inside the shared terminal must not have it think it runs inside another Claude Code session.

### 4. Texts that name the agent

- Start screen: the choice lists the three agents, then Terminal with the detail "Your shell" ("Timeline and diffs" and "Terminal only" stay for the agents). The lead text becomes "Start a coding agent or a terminal in a project folder." Without any installed agent, the existing error becomes a hint: "No coding agent is installed. Install Claude Code, OpenCode or omp for a timeline; you can still share a terminal."
- The reminder (spec "Reminder that viewers see everything") is a muted line under the choice while Terminal is selected: "Everyone you invite sees all output, including any secrets a command prints."
- Timeline and Files empty states for Terminal: "A terminal session has no timeline" and "No file list for a terminal session", without the "for … the timeline is not available yet" wording used for OpenCode and omp.
- Status pill: "Live" without the "activity is not shared" tooltip. The exited label becomes "Shell exited" instead of "Agent exited". The host banner after exit says "The shell has exited".
- Tray and notifications already use `display_name()`; a terminal session sends no "waiting" notification, because it has no events.

### 5. Protocol

`AgentId` gains a variant. 1.x is unreleased, so there is no version bump and no unknown-variant fallback. Once 1.0 ships, adding an agent will need one (noted for the future, not built now).

## Risks / Trade-offs

- [Viewers see secrets a command prints (environment variables, `.env` files, tokens)] → The reminder on the start screen. Passwords typed at prompts are not echoed by the shell, so they are not in the output. The host can end the session or remove a viewer at any time.
- [A Finder-launched app on macOS may not have `$SHELL`] → `Environment::detect` reads the login shell's environment; if that fails, the fixed fallback order applies. The host still gets a working shell, just possibly not their own.
- [Long-running full-screen programs in the shell (vim, htop) redraw a lot] → Same as agents, which are full-screen programs too; covered by the mirror and replay work in the archived change.
- [Windows PowerShell's startup banner or profile prompts] → `-NoLogo` hides the banner; profile scripts run as in Windows Terminal, which is what the host expects.
