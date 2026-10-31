# Tasks

Carried over from the archived `native-shared-session-mvp` (its task number in brackets). "Design" means that change's `design.md`.

## 1. GitHub, CI and releases

- [x] 1.1 [1.5] Push the repository to GitHub and get `ci.yml` green: `just check` and `just package` on macOS, Windows and Ubuntu, failing when an installer exceeds 25 MB, including the libdatachannel build on Windows
- [ ] 1.2 [8.6] Fill in the pnpm dependency hash in `nix/package.nix` from the first CI run of the Nix job; verify `nix build` and `nix run` on x86_64-linux in CI and on aarch64-linux in a local container, and that the binary starts
- [ ] 1.3 [8.4] Verify `release.yml` with a test tag (on a fork or as a prerelease): tag check against the app version, draft release, matrix builds uploaded under stable names, signing and notarization when secrets exist, publish when all uploaded; and that the asset names match what the cask and docs use
- [ ] 1.4 [8.1] Verify the installers per platform (`.dmg` arm64 and x64, NSIS `.exe`, AppImage and `.deb`) with the `specdeck://` scheme registered: each installs and launches on a clean machine
- [x] 1.5 Give the size check a separate AppImage limit of 100 MB (installers that use the system's web engine stay at 25 MB) in `scripts/check-installer-size.sh`, `ci.yml` and `release.yml`; verify that the script passes and fails at both limits, and that the Ubuntu CI job passes

## 2. Measurements

- [ ] 2.1 [2.4] Record three real one-hour Claude Code sessions and report raw and zstd-compressed log size and xterm.js replay time in `docs/replay-measurements.md`; state whether the 5 s catch-up target holds and, if not, raise it with the user
- [ ] 2.2 [8.2] Add `scripts/measure-footprint` and record cold start and one-hour viewing memory on the reference machines; verify the targets (1.5 s, 250 MB) are met or report the gap

## 3. Manual verification

- [ ] 3.1 [6.7] Close the host window with a viewer connected, reopen it from the Dock or tray, and see full scrollback while the viewer stays connected
- [ ] 3.2 [6.9, 7.8] Read `docs/user-guide.md` against the running app: hosting and joining, menus, tray, notifications and join links; fix what differs
- [ ] 3.3 [7.7] Check keyboard-only use and screen-reader labels (VoiceOver, Narrator, Orca) for the start screen, host and viewer views; fix gaps and verify by joining a session using only the keyboard
- [ ] 3.4 [8.3] Run the cross-platform checklist and record the results in `docs/verification.md`: host and viewer on different operating systems, a viewer relayed through TURN only, 20 viewers, window close and reopen with viewers connected, viewer reconnect after network loss; on Windows and Linux the menus and shortcuts (Ctrl+F opens search, Ctrl+C without selection interrupts the agent), the tray icon (including starting the app again on GNOME without an AppIndicator extension, and that ending a session removes it without a crash), notifications with click-to-focus, File > Settings, Mica on Windows 11 with the opaque fallback on Windows 10 and Linux, single instance and join links from a browser, and window size and position restored after a restart (also on a Mac without a tiling window manager such as AeroSpace)

## 4. Deployment

- [ ] 4.1 Deploy signalling and coturn to a test namespace once the cluster operators confirm the relay port mapping, with the namespace enforcing the `restricted` Pod Security level; verify a session relayed through it
