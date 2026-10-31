# Tasks

## 1. Project setup

- [x] 1.1 Pin tools in `mise.toml` (Rust stable, Node 24, pnpm, just, cmake) and verify `mise install` succeeds on a clean checkout
- [x] 1.2 Create the Cargo workspace (`crates/protocol`, `crates/core`, `crates/signalling`) and the Tauri 2 app (`app/src-tauri`, `app/ui` with React, TypeScript and Vite); identifier `com.entrnce.specdeck`, product name SpecDeck, version 1.0.0. Verify `just dev` opens an empty SpecDeck window
- [x] 1.3 Add the `datachannel` crate (vendored) to `core` with a smoke test that opens a loopback peer connection and exchanges one message; verify `cargo test -p core` passes on macOS
- [x] 1.4 Add `just` recipes (`install`, `dev`, `check`, `package`) and lint/format config (`clippy`, `rustfmt`, ESLint); verify `just check` passes
- [ ] 1.5 Add a CI workflow running `just check` and `just package` on macOS, Windows and Ubuntu, failing when an installer exceeds 25 MB; verify a green run, including the libdatachannel build on Windows
- [x] 1.6 Write `README.md` (what SpecDeck 1.x is, layout, development commands, relation to 0.x) and verify each documented command runs as written

## 2. Session engine: terminal and timeline

- [x] 2.1 Implement the agent adapter interface and the launcher: login-shell environment probe with 3 s timeout and fallback, executable lookup per adapter, prerequisite checks, pty start in the chosen folder via `portable-pty`; adapters for Claude Code (with Git Bash check on Windows), OpenCode and omp (terminal-only, timeline not supported). Verify with tests for found / not found / missing Git Bash / installed-agent detection, and a manual start of each installed agent on macOS from the Finder-launched app
- [x] 2.2 Implement the output log: 16 ms batching, gap-free sequence numbers, resize records, append-only file with user-only permissions deleted on close, and cursors that read from any sequence. Verify with unit tests for ordering, no gaps, resize interleaving and file cleanup
- [x] 2.3 Implement the headless `alacritty_terminal` mirror and the query-reply switch (host window attached vs not). Verify with a test that sends a device-attributes query with no window attached and receives exactly one reply on the pty
- [ ] 2.4 Measure replay: record three real one-hour Claude Code sessions, report raw and zstd-compressed log size and xterm.js replay time in `docs/replay-measurements.md`; verify the report states whether the 5 s catch-up target holds and, if not, raise it with the user before group 5
- [x] 2.5 Implement the Claude Code hook endpoint (`127.0.0.1`, random port, bearer token from environment, immediate `204`) and the `--settings` file generation. Verify with tests for authorised, unauthorised and malformed requests
- [x] 2.6 Implement the hook payload adapter and timeline builder (prompts, live and final responses, tool calls and results including not-run, file changes with diffs, file reads, output-sequence stamping), using the hook payload fixtures copied from `../claude-share/spikes/claude-hooks/fixtures` into `crates/core/tests/fixtures/claude-hooks`. Verify all fixture tests pass
- [x] 2.7 Implement degraded mode (no event within 10 s of Enter, or unparsable payload; recovery on the next valid event) and Claude status (`starting`, `working`, `waiting`, `exited`). Verify with tests for each transition
- [x] 2.8 Implement the host session lifecycle in `core`: start with the chosen agent, input from host only, host resize, agent exit while session stays open, close (process ended, log deleted), timeline-not-supported and status `unknown` for agents without an event source. Verify with integration tests driving fake `claude` and `opencode` scripts
- [x] 2.9 Implement the usage reader: check that the transcript path is in Claude Code's own folder, read only new lines, count each message once (subagents in the session total), context fill and window size, and the `Usage` timeline entry. Verify with tests against sanitized transcript fixtures: a normal session, a compacted session, a subagent, a path outside Claude Code's folder, and a corrupt line
- [x] 2.10 Implement the status-line relay and git state: the relay script and `statusLine` setting with the host's own command and padding, the `/status` route, the `StatusLine` entry (changes only), and the `Git` entry from `git status` every 5 s for every agent. Verify with tests: the host's command still prints its line, payloads become entries, unchanged payloads emit nothing, no git repository, dirty and ahead/behind, and a fake `claude` session end to end

## 3. Protocol

- [x] 3.1 Implement invite codes: generation, display grouping, input normalisation, HKDF room id and auth key. Verify with tests including `7k2m9qxh4tpa` ≡ `7K2M-9QXH-4TPA` and fixed test vectors
- [x] 3.2 Implement SDP authentication (HMAC under the auth key) and verify a substituted fingerprint is rejected in a test
- [x] 3.3 Implement control messages with `postcard` and strict viewer-message decoding (no input message exists). Verify round-trip tests and rejection of unknown variants
- [x] 3.4 Implement signed stream frames (output, resize, history batch, timeline event) with Ed25519 and sequence checks. Verify tests for tampered payload, moved sequence and changed kind
- [x] 3.5 Implement protocol version checking with refusals that name which side must update; verify with tests
- [x] 3.6 Document the protocol in `crates/protocol/README.md` (channels, messages, frames, handshake, late join, reconnect) and verify it matches the types by reviewing each message against the code

## 4. Signalling service and relay

- [x] 4.1 Implement the `axum` WebSocket signalling service: in-memory rooms, relaying authenticated offers, answers and ICE candidates, room close. Verify with integration tests using two test clients
- [x] 4.2 Add rate limiting per client IP and room, and short-lived TURN credentials (TURN REST API). Verify tests for refused attempts after the limit and credential expiry
- [x] 4.3 Add a container image and a local stack (signalling + coturn on Colima's Kubernetes, the overlay from 4.4) started with `just local-up`; verify two clients connect through coturn with direct connections disabled
- [x] 4.4 Add Kubernetes manifests (`deploy/k8s/base`, `local`) for signalling and coturn and a README covering ports and secrets; verify `kubectl apply -k deploy/k8s/local` works on Colima

## 5. Sharing and streaming in the engine

- [x] 5.1 Implement the host side of sharing: register room, admit with `welcome`, refuse (outdated, full, removed, ended), participant list with duplicate-name labels, viewer limit 1–20, revoke and remove with code rotation and `rejoin-code`. Verify with engine-to-engine tests through the local signalling service
- [x] 5.2 Implement per-viewer sender tasks reading from log cursors with buffered-amount backpressure and zstd history batches. Verify a test where one throttled viewer does not delay the pty or another viewer, and then catches up
- [x] 5.3 Implement the viewer side: join with code, verify frames, drop and `resend` on gaps or failed signatures, automatic reconnect with resume token (also after a revoke), recreating the peer connection when no relay candidate arrives (coturn 437), the "network does not allow the connection" message when no connection can be made, session-ended handling. Verify tests for a 20-second outage and for a session closed during an outage
- [x] 5.4 Implement "sharing unavailable" on the host (session runs locally, retries, code appears when signalling returns) and the viewer's "service cannot be reached" error. Verify with tests that stop and restart signalling
- [x] 5.5 Verify mirror exactness: a test runs recorded full-screen Claude Code, OpenCode and omp sessions with resizes, joins a viewer mid-way, and compares the host's and viewer's xterm-headless screen, colours and cursor
- [x] 5.6 Verify capacity: a load test with 20 viewers and continuous output keeps host input echo under 50 ms and delivers output to every viewer within 1 s

## 6. App UI

- [x] 6.1 Implement the Tauri command and event bridge between UI and `core` (binary streaming via IPC channels). Verify with a UI test harness that renders streamed output
- [x] 6.2 Implement the start screen: username (required before host or join), agent choice (installed agents, last used preselected, missing ones shown as not installed), host with folder dialog, recent folders (five, newest first, missing ones hidden, removable, not added on failure), join with code and error messages. Verify with component tests for each scenario
- [x] 6.3 Implement the host view: xterm.js terminal with input and resize, invite code with copy code and copy link, participants with remove, viewer limit, new code, close session with confirmation, one-session-at-a-time prompt. Verify component tests and a manual session on macOS
- [x] 6.4 Implement the viewer view: read-only terminal (discarding `onData`), read-only banner, status (live, working, waiting, ended), keep reading after end. Verify component tests and that no input command exists in the viewer path
- [x] 6.5 Implement the timeline and file panel: prompts, live markdown responses, tool entries, diffs, touched-files list with per-file diff history, jump to terminal position, and the "not available for this agent" state (no prompt list or touched files). Verify component tests against fixture-built timelines and a terminal-only session
- [x] 6.6 Implement follow/free-scroll, search across terminal and timeline, copy, and the prompt list with jump. Verify component tests for each `viewer-experience` scenario
- [ ] 6.7 Implement close-window-keeps-session and reattach by local replay. Verify manually: close the window with a viewer connected, reopen, and see full scrollback while the viewer stays connected
- [x] 6.8 Add the local two-app test setup: `just local-dev` starts the local signalling + coturn stack (4.3) and two app instances, a host and a viewer, each with its own profile (`SPECDECK_PROFILE`: separate settings and data directory, separate single-instance key) pointed at the local signalling service; `just local-down` stops it. Verify by hosting in one window, joining with the code in the other, and seeing output, timeline and participants live
- [ ] 6.9 Write `docs/user-guide.md` for hosting and joining, and verify each step against the running app
- [x] 6.10 Show usage in the UI: a status bar at the bottom of the host and viewer windows (folder, git state, model and effort, context gauge with percentage and used/size, session total, 5h and 7d rate limits with time to reset), small and quiet in style, tokens per prompt in the Timeline, an unavailable state, and nothing for OpenCode and omp. Verify with component tests and a live session

## 7. Native integration

- [x] 7.1 Build per-platform menus with `CmdOrCtrl` shortcuts and Open Recent; keep terminal-conflicting keys for the terminal. Verify on macOS (Windows and Linux in 8.3): Cmd/Ctrl+F opens search, Ctrl+C without selection interrupts Claude on Windows and Linux
- [x] 7.2 Add the tray / menu-bar icon while hosting (state, viewer count, open window, copy code, close session). Verify on macOS (Windows and Linux in 8.3), and with 7.5 that starting the app again reopens the window on GNOME without an AppIndicator extension
- [x] 7.3 Add notifications (viewer joined/left, Claude waiting, session ended) only when unfocused, with a settings toggle. Verify on macOS (Windows and Linux in 8.3) including that clicking focuses the session
- [ ] 7.4 Add appearance: follow OS light/dark live (including terminal theme), system fonts, macOS overlay title bar and vibrancy, Windows Mica, Linux opaque fallback. Verify by switching the OS theme with the app open on each OS
- [ ] 7.5 Add window state and pane-size persistence, single instance and `specdeck://join/<code>` deep links (including the leave-current-session prompt). Verify on each OS by starting twice and opening a link from a browser
- [x] 7.6 Add the settings window (username, notifications) opened from the platform menu. Verify Cmd+, on macOS (File > Settings on Windows and Linux in 8.3)
- [ ] 7.7 Check keyboard-only use and screen-reader labels (VoiceOver, Narrator, Orca) for the start screen, host and viewer views; fix gaps and verify by joining a session using only the keyboard
- [ ] 7.8 Update `docs/user-guide.md` for menus, tray, notifications and join links, and verify against the app

## 8. Packaging and end-to-end verification

- [ ] 8.1 Produce installers per platform (`.dmg` arm64 and x64, NSIS `.exe`, AppImage and `.deb`) with the `specdeck://` scheme registered; verify each installs and launches on a clean machine
- [ ] 8.2 Add `scripts/measure-footprint` and record cold start and one-hour viewing memory on the reference machines; verify the targets (1.5 s, 250 MB) are met or report the gap
- [ ] 8.3 Run the cross-platform checklist: host and viewer on different operating systems, a viewer relayed through TURN only, 20 viewers, window close and reopen with viewers connected, viewer reconnect after network loss; on Windows and Linux the menus and shortcuts (Ctrl+F opens search, Ctrl+C without selection interrupts the agent) the tray icon (including starting the app again on GNOME without an AppIndicator extension), notifications with click-to-focus, and File > Settings; record results in `docs/verification.md`

## Workflow follow-up

- Deploy signalling and coturn to a test namespace once the cluster operators confirm the relay port mapping.
- Archive the change after review.
