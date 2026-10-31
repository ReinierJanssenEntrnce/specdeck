# Design

## Context

This repository is empty apart from the OpenSpec setup. Motivation and scope are in `proposal.md`; behaviour is in `specs/`.

The Electron SpecDeck in `../claude-share` is the reference for what works. Its archived design (`openspec/changes/archive/2026-10-30-shared-claude-session/design.md` there) and spikes (`docs/spikes/`) settled several questions this design reuses: Claude Code HTTP hooks carry everything the timeline needs, WebRTC with TURN through the team's Kubernetes cluster works for NAT traversal, and invite-code-derived keys keep the signalling service blind. That design rejected Tauri because Rust had no xterm-compatible snapshot serializer and each OS has a different webview. Decisions 4 and 5 answer those two objections.

Constraints:

- The app never talks to a model provider itself; it runs the host's installed agent CLI (`claude`, `opencode` or `omp`) and observes it.
- One app for macOS, Windows and Linux, as host or viewer.
- The only server-side parts are the signalling service and coturn; neither may read session content.
- Clean break from SpecDeck 0.x: no protocol, recording or settings compatibility is required.

## Goals / Non-Goals

**Goals:**

- One Rust session engine that host and viewer both use, tested without a UI.
- The host's terminal and every viewer's terminal process the same bytes in the same order, so mirroring is exact by construction.
- Native platform behaviour delivered through Tauri's own facilities, without per-platform UI code.

**Non-Goals:**

- A session that survives the app process (crash or quit). Only the window is optional while hosting (decision 2).
- A native (non-web) terminal renderer. xterm.js stays.
- Viewer-to-viewer forwarding, multiple hosted sessions, recording, auto-update and code signing.
- Diffs for files changed by shell commands Claude runs; only file-editing tools produce diffs.
- Migrating 0.x settings (see Open Questions).

## Decisions

### 1. Tauri 2 app over a Cargo workspace

```
crates/protocol     wire types, codec, signing, invite-code crypto (no I/O)
crates/core         session engine: pty, terminal mirror, output log, hooks,
                    timeline, peer connections, host and viewer roles
crates/signalling   matchmaking WebSocket service and TURN credentials (binary)
app/src-tauri       Tauri shell: commands, events, menus, tray, plugins
app/ui              React + TypeScript + Vite web UI (xterm.js, timeline, diffs)
```

- The UI only renders and sends user intents through Tauri commands; all session logic lives in `core`, so it is testable with `cargo test` and two engines can talk to each other in one test process.
- *UI framework*: React 19, the same as SpecDeck 0.x, so the team's knowledge carries over. Svelte or Solid would be slightly lighter, but the webview's base memory dominates.
- *Alternatives*: Leptos/Dioxus (Rust UI in the webview; smaller ecosystem for terminals, markdown and diffs).

### 2. Session engine in the app process; window optional

The host session runs in the Tauri process. Closing the window destroys the webview (freeing its memory) but keeps the process, which stays reachable from the tray or menu bar. Reopening creates a new webview that reattaches by replaying the output log locally (decision 4). Quitting asks for confirmation (`session-hosting`).

- *Why*: SpecDeck 0.x needed a separate Node daemon and a local socket because Electron windows and main process were tied together. A native app that keeps running in the menu bar is the platform's own pattern for this, and removes the second process, the IPC protocol and its auth token.
- *Trade-off*: an app crash ends the session. Accepted for the MVP.
- *Alternative*: Rust sidecar daemon (`externalBin`) with local IPC. Possible later because `core` does not depend on Tauri.

### 3. PTY and launching the agent

- `portable-pty` (ConPTY on Windows, Unix pty elsewhere).
- GUI apps on macOS and Linux do not inherit the shell's `PATH`. Before launching, `core` reads the environment of the user's login shell (`$SHELL -l -i -c env`, with a timeout) and uses its `PATH` to find the agent's executable, then starts it directly with that environment. On Windows it uses the process environment. Agent-specific prerequisites come from the agent adapter (decision 3a), for example Claude Code's Git Bash check via `CLAUDE_CODE_GIT_BASH_PATH` or the default install locations.

### 3a. Agent adapters

Everything agent-specific sits behind one `Agent` interface in `core`; the pty, output log, streaming and viewers never know which agent runs.

| Part | Claude Code | OpenCode | omp |
|---|---|---|---|
| Executable | `claude` | `opencode` | `omp` |
| Prerequisites | Git Bash on Windows | none | none |
| Extra launch args/env | `--settings` hook file, hook token env (decision 10) | none | none |
| Event source | HTTP hooks → timeline and status | none yet | none yet |

- Detection runs the same `PATH` lookup for each adapter, so the start screen can show which agents are installed.
- An adapter without an event source makes the session report `timeline: not supported for <agent>` from the start (distinct from degraded mode, which is about missing events from an agent that should send them) and status `unknown`.
- *Later*: OpenCode exposes events through plugins and its local server, and omp through extensions; their adapters are a follow-up change after researching those APIs. The protocol already carries the agent id and the timeline-support state, so adding them needs no protocol change.
- *Alternative*: an "any command" agent where the host types a command line. Left out: it would make the host choose argv and environment by hand, and the three named agents cover the request.
- Input reaches the pty only from the host's own UI through a Tauri command. The protocol has no message type that carries input (`session-streaming`).

### 4. Output log as the single source of terminal state

Every pty read is batched (about 16 ms) into an output record with a gap-free sequence number. Resizes are records in the same log. The log is an append-only file in the app's cache directory with user-only permissions, deleted when the session closes, plus an in-memory index.

- **Late join, reconnect and window reattach are the same operation**: start a cursor at a sequence number (0, the viewer's last sequence + 1, or 0 for the host's new window) and send records from there. The receiving xterm.js processes exactly the bytes the host's did, at the sizes the host had, so the result is identical, including full-screen views, colours and cursor (`session-streaming`).
- **Catch-up transfer**: when a cursor is far behind, records are sent in zstd-compressed batches (redraw-heavy terminal output compresses very well). The viewer writes history into a hidden terminal and shows it when it is caught up.
- **Backpressure**: each viewer has its own sender task that reads from its cursor and sends while the data channel's buffered amount is under a threshold. A slow viewer simply falls behind and catches up from the log; it never blocks the pty or other viewers.
- *Why*: answers the "no snapshot serializer in Rust" objection by not needing one. SpecDeck 0.x's raw-log replay was already its planned fallback.
- *Trade-off*: replay cost grows with session length. Task 2.4 measures real sessions; if a one-hour session needs more than about 5 s to catch up on broadband, add periodic checkpoints serialized from the headless mirror (decision 5) in a later change.
- *Alternative*: serialize snapshots from `alacritty_terminal`'s grid to ANSI (fidelity work and a second source of truth).

### 5. Headless mirror for terminal queries and status

`core` feeds every output record into an `alacritty_terminal` instance with no renderer.

- **Query replies**: when a host window is attached, its xterm.js answers terminal queries (its `onData` is the host's input path). When no window is attached, the headless mirror's replies (`PtyWrite` events) go to the pty instead, so Claude Code never waits on an unanswered query. Viewer terminals discard everything their xterm.js emits, so exactly one answer reaches the session.
- The mirror's size follows the host's resizes.

### 6. WebRTC in Rust on both sides (libdatachannel)

Host and viewer both use the `datachannel` crate (libdatachannel bindings, vendored build). The UI receives streams from Rust through a Tauri IPC channel and never touches WebRTC.

- *Why*: WebKitGTK's WebRTC support varies by distribution, and WKWebView, WebView2 and WebKitGTK differ in data channel behaviour. Rust on both sides gives one implementation, which SpecDeck 0.x's spike already proved against coturn. This answers the "different webview per OS" objection for networking.
- Each peer connection has a reliable ordered `control` channel and a reliable ordered `stream` channel. Star topology, up to 20 viewers.
- *Alternatives*: `webrtc-rs` (pure Rust, no C++ build, but large and in maintenance flux); `str0m` (sans-IO, no TURN client).

### 7. Wire protocol v1

- `control` messages: `serde` enums encoded with `postcard`, length-checked; unknown variants from viewers are rejected. Viewer to host: `hello` (protocol version, username, last sequences, resume token), `resend`, `leave`. Host to viewer: `welcome` (viewer id, session public key, size, resume token, rejoin room, agent id, timeline support), `refused` (reason), `participants`, `status`, `session-ended`.
- `stream` frames: `kind` (output, resize, history batch, timeline event), sequence, payload, Ed25519 signature over everything before it. A viewer drops a frame whose sequence is not the next expected one, or that fails verification, and sends `resend`.
- Signing uses `ed25519-dalek` with a key pair per session; the public key is trusted because it arrives over the code-authenticated connection (decision 8). Signing per 16 ms batch keeps the cost low.
- `checkVersion` accepts equal protocol versions only, and the refusal says which side must update.

### 8. Invite codes, join links and trust

Same scheme as SpecDeck 0.x, reimplemented:

- Code: 12 random Crockford base32 characters (about 60 bits), shown as `XXXX-XXXX-XXXX`; input is normalised (case, dashes, `O`→`0`, `I`/`L`→`1`).
- HKDF-SHA256 derives a **room id** (sent to signalling) and an **auth key** (never sent). SDP offers and answers carry an HMAC under the auth key, so the signalling service cannot substitute DTLS fingerprints (`session-sharing`: End-to-end encryption).
- Revoke and remove rotate the code. Admitted viewers reconnect through a **rejoin room** the host opens for the whole session (its id comes in `welcome`), authenticating offer and answer with their own resume token instead of the code. A viewer that was offline while the code changed can therefore still return; a removed viewer's token is forgotten, so it cannot; and after the session closed the rejoin room is gone, which tells the viewer the session ended.
- Join link: `specdeck://join/<code>`, registered with `tauri-plugin-deep-link`. The link contains the code, so it is as secret as the code.

### 9. Signalling service and TURN

`crates/signalling`: an `axum` WebSocket service with in-memory rooms. It relays authenticated SDP and ICE messages, confirms opened rooms, rate-limits join attempts per client IP and room, and issues short-lived TURN credentials (TURN REST API shared secret). coturn listens on UDP.

- **TURN over UDP only (MVP)**: libdatachannel's bundled ICE backend, libjuice, does not support TURN over TCP or TLS. Viewers on networks that block UDP entirely cannot connect; the app says so instead of waiting. Decided for the MVP; the pilot shows whether a TCP/443 path is needed. *Alternatives, if it is*: libdatachannel built with libnice (TURN over TCP/TLS, but GLib in every build and installer), or a fallback that tunnels the stream through the signalling service over TLS 443 with app-level end-to-end encryption. Deployment follows SpecDeck 0.x's Kubernetes setup in a separate namespace; the open port-exposure question with the cluster operators (0.x README, Status) applies here too. Local development uses coturn in a container.

### 10. Timeline from Claude Code HTTP hooks

As in SpecDeck 0.x (its design decision 6, verified in `docs/spikes/claude-hooks.md`): `claude` starts with an extra `--settings` file registering HTTP hooks to a `127.0.0.1` endpoint on a random port (`axum`), authorised with a per-session bearer token passed by environment variable. The endpoint answers `204` immediately and never influences Claude.

- One adapter module maps hook payloads to `TimelineEvent`s and is tested against the recorded hook payloads copied from `../claude-share/spikes/claude-hooks/fixtures`.
- Each event is stamped with the current output sequence, which links entries to terminal positions.
- Degraded mode: no event within 10 s of the host pressing Enter, or an unparsable payload, marks the timeline unavailable; the next valid event restores it.
- Status for viewers (`working`, `waiting`) and the host's "Claude is waiting" notification come from the same events (`UserPromptSubmit`, `Stop`, `Notification`).

### 10a. Usage and status bar from the Claude Code transcript, the status line and git

Every hook payload carries `transcript_path`, the JSONL transcript Claude Code writes. The host reads it only when the path is a `.jsonl` file under Claude Code's projects folder (`~/.claude/projects`, or `$CLAUDE_CONFIG_DIR/projects`).

- The host learns the path from any hook and then reads only the lines added since its last read, after each hook and every 250 ms: a compaction sends no hook the engine registers, and a final answer can reach the transcript after its `Stop` hook. A missing transcript counts as a problem only after `Stop`, when it must exist.
- Once a session's context exceeds 200k, the 1M window stays, also after compaction lowers the context again.
- Each assistant message is counted once, by its message id: Claude Code repeats the same usage on several lines of one answer. Subagent transcripts next to the main one count towards the session total, not towards context fill.
- Context fill is the latest main-conversation answer's input tokens plus cache-read and cache-write tokens. It is measured against 200k, or 1M when the model id ends in `[1m]` or usage already exceeds 200k; when the window size is unknown, only the token count is shown.
- Usage travels as a timeline entry, `Usage`, carrying running totals and stamped like the other entries, so late joiners and reconnecting viewers get it through the existing timeline replay without a new message type. Per-prompt tokens are the difference between the `Usage` entries at the start and end of a prompt.
- Model, effort, fast mode, exact context and rate limits reach only Claude Code's status-line command, so SpecDeck **relays the status line**: its `--settings` file sets `statusLine` to a small POSIX `sh` script in the data folder (Claude Code runs status lines through Git Bash on Windows). The script posts its input to the hook endpoint's `/status` route with the hook token, using `curl` with a one-second limit, and then runs the host's own status-line command with the same input, so the terminal keeps the host's status line. The host's command and padding are read from Claude Code's settings in the usual order (`.claude/settings.local.json` and `.claude/settings.json` in the folder, then the user's settings) and passed in `SPECDECK_STATUSLINE_COMMAND`. Without `curl`, the bar shows what the transcript and git provide.
- Status-line data becomes a `StatusLine` timeline entry (model, window, effort, fast, context, rate limits with percentages rounded to whole numbers and reset times), emitted only when it changes. It replaces the transcript's context figure while it is fresh. Money cost is not shown.
- Git state comes from the host running `git status --porcelain=v2 --branch -uno` in the session folder every 5 s (as the host's own status line does), for every agent, and travels as a `Git` timeline entry when it changes.
- *Risk*: the transcript format is not a public interface. A line that cannot be parsed marks usage unavailable instead of showing wrong numbers; fixture tests guard the format.

### 11. Native integration through Tauri

| Behaviour (`native-experience`) | Mechanism |
|---|---|
| Menus and shortcuts | Tauri menu API, built per platform (app menu on macOS; File/Edit/View/Help elsewhere), `CmdOrCtrl` accelerators. The host terminal gets first refusal on keys: shortcuts that clash with terminal keys (Ctrl+C without selection, Ctrl+F) are not registered as accelerators on Windows and Linux and are handled in the UI instead |
| Tray / menu bar | Tauri tray icon, created when hosting starts and removed when it ends; template icon on macOS |
| Notifications | `tauri-plugin-notification`, sent only when the window is unfocused or closed |
| Dialogs | `tauri-plugin-dialog` for folder choice and confirmations |
| Appearance | Webview `prefers-color-scheme` plus Tauri theme events; system font stack; `window-vibrancy` (macOS vibrancy sidebar, Windows Mica) with an opaque fallback on Linux; overlay title bar on macOS |
| Recent folders | Settings store; also File > Open Recent. No Dock menu: Tauri has no API for it, and File > Open Recent already offers the folders |
| Window state | `tauri-plugin-window-state` (keeps windows on-screen); pane sizes in the settings store |
| Single instance, deep links | `tauri-plugin-single-instance` forwarding argv/URLs to the running instance; `tauri-plugin-deep-link` |
| Settings | `tauri-plugin-store` JSON in the app config directory: username, notifications on/off, recent folders, last used agent |
| Local testing with two apps | Debug builds read `SPECDECK_PROFILE`; a non-default profile uses its own config and data directories and window-state file, and skips the single-instance check (the plugin locks on the app identifier), so `just local-dev` can run a host and a viewer side by side against the local signalling stack. Release builds ignore it |
| Accessibility | Semantic HTML with ARIA labels; the webviews map these to each platform's accessibility API |

Identifier `com.entrnce.specdeck`, product name SpecDeck, version 1.0.0, so on macOS and Windows the native app installs in place of the Electron one.

### 12. Footprint measurement

Reference machines: Apple M1 with 8 GB (macOS 15), an x64 laptop with 8 GB (Windows 11), the same laptop with Ubuntu 24.04. Installer size is checked in CI on every build; start time and memory are measured by a script in the release checklist.

## Risks / Trade-offs

- [macOS Local Network privacy blocks an app launched from Finder or a link until the user allows it, and ties the permission to the code signature] → `NSLocalNetworkUsageDescription` explains the prompt; the bundle is ad-hoc signed with its identifier (`signingIdentity: "-"`), so the permission holds per build. Only a Developer ID signature (with signed installers, a later change) keeps it across updates. Found in task 7.5: started from a terminal the app inherits the terminal's permission, which hides the problem in development.
- [Replay time for long, redraw-heavy sessions] → Measure on real sessions (task 2.4); compressed batches; checkpoints from the headless mirror if needed.
- [xterm.js performance or WebGL availability on WebKitGTK] → Use the WebGL renderer where available, fall back to the DOM renderer; include Linux in the footprint measurements.
- [libdatachannel's C++/CMake build on Windows and in CI] → Vendored feature; build on all three platforms in CI from the first task; `webrtc-rs` as fallback behind the same `core` interface.
- [GNOME without an AppIndicator extension shows no tray icon] → Starting the app again reopens the session window through single instance; documented in the user guide.
- [Login-shell environment probing is slow or hangs on unusual shell configs] → Timeout of 3 s, then fall back to the process `PATH` and common install locations, with a clear error if `claude` is still not found.
- [OpenCode or omp behave differently in a pty, for example in how they query the terminal or redraw] → Include an OpenCode and an omp session in the mirror-exactness check (task 5.5).
- [Claude Code changes its hook payloads] → One adapter with fixture tests; degraded mode keeps the terminal mirror working.
- [App crash ends the session for everyone] → Accepted for the MVP (decision 2); sidecar daemon possible later.
- [0.x and 1.x users try to share with each other] → They cannot meet (different signalling); release notes and the in-app "invalid code" message mention that both sides need SpecDeck 1.x.
- [Networks that block UDP entirely (some corporate networks)] → Cannot connect in the MVP (TURN over UDP only, decision 9); clear message to the user; revisit after the pilot.
- [coturn refuses a new allocation from an address it still holds one for (437), for example after a quick restart; libjuice does not retry] → The engine recreates the peer connection, and with it its socket, when no relay candidate arrives.
- [Invite code brute force] → 60-bit codes, signalling rate limits, codes valid only while the session is open.

## Migration Plan

1. Deploy the new signalling service and coturn to a separate namespace; 0.x keeps its own deployment.
2. Pilot SpecDeck 1.x with one team; both hosts and viewers switch at once.
3. Announce 1.x to 0.x users through the 0.x update notice (0.x cannot update itself to 1.x), then retire the 0.x deployment.

Rollback: keep distributing 0.x and leave its deployment running until the pilot succeeds.

## Open Questions

- Whether to import the username and recent folders from 0.x settings on first start (small, independent task).
- Exact relay port mapping in the cluster (inherited from 0.x, needs the cluster operators).
