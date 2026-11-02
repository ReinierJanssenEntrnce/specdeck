# Proposal

## Why

Typing in the host's terminal sometimes sends the line again: the agent receives `the quick brown fox jumthe quick brown fox jumps over the lazy dog agthe quick…` for one typed sentence. The terminal runs xterm.js with screen reader mode always on. In that mode xterm leaves every typed character in its hidden text box until Enter, and some of its input paths send that text box, or its end, to the pty again. The host cannot work around it, and every prompt they type can arrive garbled.

## What Changes

- The terminal's screen reader mode is off by default, so typed text no longer collects in xterm's hidden text box and each keystroke reaches the session once.
- A new setting, **Screen reader support** in Settings, turns screen reader mode back on for people who use VoiceOver, Narrator or Orca. It is one setting per person, saved, and takes effect at once in an open session, for the host's and a viewer's terminal alike.
- The setting's hint says that typing in the host's terminal can repeat text while it is on, so that a screen reader user knows the trade-off.
- The docs (Settings, Keyboard and screen readers) describe the setting; the keyboard page no longer says that screen readers read the terminal without it.
- Not in this change: detecting a running screen reader from the OS and turning the setting on by itself, and fixing the repeat inside xterm while screen reader mode is on.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-hosting`: "Host owns input" gains a scenario that typed text reaches the session exactly once.
- `native-experience`: adds a "Screen reader support in the terminal" requirement (off by default, a setting per person, applied at once).

## Impact

- `app/ui/src/terminal/TerminalView.tsx`: `screenReaderMode` follows a new prop instead of being always on, and changes on a live terminal.
- `app/ui/src/screens/Workspace.tsx`, `HostScreen.tsx`, `ViewerScreen.tsx`, `App.tsx`: pass the setting from the profile to the terminal.
- `app/src-tauri/src/settings.rs`: a new `screen_reader` field, default `false`. Older settings files still load.
- `app/src-tauri/src/lib.rs`: the profile carries the setting; a `set_screen_reader` command saves it and tells the windows.
- `app/ui/src/api/` (`types.ts`, `bridge.ts`, `tauri.ts`, `demo.ts`) and `app/ui/src/screens/SettingsScreen.tsx`: the setting and its checkbox.
- `docs/site/src/content/docs/guide/settings.md` and `keyboard.md`.
- No new dependencies, no protocol change: the setting is local to each person.
- Behaviour change for anyone relying on screen readers reading the terminal today: they turn on the setting once.
