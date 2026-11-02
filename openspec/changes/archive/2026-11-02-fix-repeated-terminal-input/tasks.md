# Tasks

## 1. Setting

- [x] 1.1 Add `screen_reader: bool` (default `false`) to `Settings` in `app/src-tauri/src/settings.rs`, and name it in the module comment. Verify with a unit test that a settings file without the field loads with it off, and that a saved `true` loads back as `true`.
- [x] 1.2 Carry it in `Profile` as `screenReader` and add a `set_screen_reader` command in `app/src-tauri/src/lib.rs` that saves it and calls `settings_changed`, like `set_notifications`; register the command. Verify `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.
- [x] 1.3 Add `screenReader` to `Profile` in `app/ui/src/api/types.ts`, `setScreenReader` to the bridge (`bridge.ts`, `tauri.ts`, and `demo.ts` with `false` in the demo profile). Verify `pnpm typecheck` passes.

## 2. Settings window

- [x] 2.1 Add a "Screen reader support" row with a checkbox to `app/ui/src/screens/SettingsScreen.tsx`, labelled and described like the Notifications row, with a hint saying the screen reader then reads the terminal and that typing in the host's terminal can repeat text while it is on. Verify with a test in `SettingsScreen.test.tsx` that the checkbox shows the profile's value and that ticking it calls `setScreenReader(true)`.
- [x] 2.2 Add the setting to the table in `docs/site/src/content/docs/guide/settings.md`, and in `docs/site/src/content/docs/guide/keyboard.md` replace "They also read the terminal." with a sentence that they read the terminal once **Screen reader support** is on in Settings. Verify `just docs-build` succeeds.

## 3. Terminal

- [x] 3.1 In `app/ui/src/terminal/TerminalView.tsx`, add a `screenReader` prop, create the terminal with `screenReaderMode` from it instead of `true`, and set `term.options.screenReaderMode` in a separate effect when the prop changes; move the VoiceOver comment to the prop. Verify `pnpm test` passes.
- [x] 3.2 Pass `profile.screenReader` from `App.tsx` to `HostScreen` and `ViewerScreen`, and from there through `Workspace` to `TerminalView`. Verify with tests in `HostScreen.test.tsx` and `ViewerScreen.test.tsx` (through the terminal stub's recorded props) that the terminal gets `screenReader` false by default and true when the screen is given true.

## 4. Check the fix

- [x] 4.1 In `just dev`, host a Terminal session and a Claude Code session with the setting off, and on macOS with the US and the US International layouts type a line longer than the terminal is wide, with Backspace corrections in the middle, then Enter. Verify the shell or agent receives the line exactly as typed, every time. Then turn the setting on during the session and verify VoiceOver reads the terminal's lines without rejoining. Record the result in the PR.
- [x] 4.2 Run `just check` and verify it passes.
