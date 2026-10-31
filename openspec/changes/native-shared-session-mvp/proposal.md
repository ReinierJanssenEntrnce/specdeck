# Proposal

## Why

SpecDeck today (the Electron app in `../claude-share`, version 0.x) lets a host run Claude Code while teammates follow read-only from anywhere. It works, but as an Electron app it is heavy (about 150 MB installed, slow cold start, high idle memory), runs its session engine in Node with native modules that need rebuild workarounds (`node-pty`, `node-datachannel`), and looks and behaves like a web page rather than an app of the platform it runs on. This change rebuilds SpecDeck as a native desktop app: a Rust session engine and native OS integration on Tauri 2, for macOS, Windows and Linux.

## What Changes

- New Tauri 2 desktop app, still called **SpecDeck** (version 1.0), for macOS, Windows and Linux; one install acts as host or viewer. It replaces the Electron app.
- The host chooses the coding agent per session: Claude Code, OpenCode or omp. Terminal sharing works for all three; the timeline is built for Claude Code in this change, and OpenCode and omp run terminal-only until their event adapters follow in a later change. For Claude Code, all participants also see the session's token usage and how full the context window is.
- New Rust session engine: starts the chosen agent's CLI in a pty, mirrors the terminal, captures Claude Code hook events into a timeline, and streams everything to viewers over WebRTC data channels. No Node runtime ships with the app.
- WebRTC runs in Rust on both host and viewer, so behaviour does not depend on each platform's webview.
- New Rust signalling service (matchmaking and TURN credentials), deployed with coturn.
- New wire protocol, designed for this app. **BREAKING**: native SpecDeck 1.x and Electron SpecDeck 0.x cannot join each other's sessions, and 0.x recordings cannot be opened.
- Native platform integration: system menus and shortcuts, tray/menu-bar presence while hosting, OS notifications, native dialogs, appearance that follows the OS, `specdeck://` join links, and footprint targets.
- MVP scope only. Not in this change (later changes): session recording and replay, chat/terminal interface modes, display scaling, custom theming beyond following the OS, auto-updates, distribution channels (Homebrew, Nix, signed installers).

## Capabilities

### New Capabilities

- `session-hosting`: Starting and running a coding agent session (Claude Code, OpenCode or omp) on the host's machine with the host's own login: username, choosing the agent and a folder (with recent folders), exclusive host input and terminal size, keeping the session alive when the window closes, ending it.
- `session-sharing`: Who can join and how: invite codes and join links, participant list, removing viewers, viewer limit, remote connectivity through NAT with relay fallback, end-to-end encryption, behaviour when the matchmaking service is unreachable.
- `session-streaming`: Delivering the host's terminal to viewers as an exact, live, read-only mirror, including late join, reconnect, capacity and stream authenticity.
- `session-timeline`: Turning Claude Code hook events into a timeline of prompts, responses, tool calls and file diffs, linked to terminal positions, with a degraded mode, and the session's token usage and context fill.
- `viewer-experience`: Reading a shared session at one's own pace: follow and free-scroll, search, copy, jump to prompt, read-only interface, session status.
- `native-experience`: How the app behaves as a native desktop app on each platform: menus and shortcuts, tray/menu-bar, notifications, appearance, single instance and deep links, and resource footprint.

### Modified Capabilities

None. This repository has no specs yet.

## Impact

- New code in this repository: a Cargo workspace (session engine, protocol, signalling) and a Tauri app with a TypeScript web UI.
- Dependencies: Tauri 2, `portable-pty`, `alacritty_terminal`, libdatachannel (via the `datachannel` crate), `ed25519-dalek`, `axum`/`tokio`, xterm.js in the UI.
- Infrastructure: a signalling service and coturn in the team's Kubernetes cluster, separate from the 0.x deployment.
- The Electron SpecDeck code in `../claude-share` is not changed. Teams must move to 1.x together, because 0.x and 1.x do not interoperate, and 0.x installs cannot update themselves to 1.x.
- Requires on the host: at least one supported agent installed and logged in; for Claude Code on Windows also Git Bash (as with 0.x).
