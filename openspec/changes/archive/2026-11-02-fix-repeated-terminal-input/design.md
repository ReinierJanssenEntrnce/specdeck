# Design

## Context

See proposal.md for motivation, and the `session-hosting` and `native-experience` deltas for the behavior.

What is there today:

- `TerminalView.tsx` creates xterm.js 6.0.0 with `screenReaderMode: true` for both modes, with the comment that it lets VoiceOver, Narrator and Orca read the terminal's lines.
- In xterm, screen reader mode changes keyboard input: a handled keydown is not cancelled (`_keyDown` sets `_keyDownHandled` instead of calling `cancel`), so the character also lands in xterm's hidden `<textarea>`. That text box is only emptied on Enter or Ctrl+C. Several input paths then read it back and send it to the pty: `_handleAnyTextareaChanges` sends the whole value when the old value is not a prefix of the new one or the length is unchanged, and the end of a composition sends `textarea.value.substring(start)`. With screen reader mode off, keydown cancels the event, the text box stays empty, and those paths have nothing to repeat.
- The reproduction from the host terminal (plain letters, no dead keys) shows the line sent again from its first letter several times, which matches the text box being sent in full.
- Settings are a JSON file read into `Settings` (`settings.rs`) with `#[serde(default)]`, so new fields load from older files. A boolean setting such as `check_updates` goes `Settings` → `Profile` (`lib.rs`) → a `set_*` command that saves and calls `settings_changed` → the settings window's checkbox (`SettingsScreen.tsx`). The main window refreshes its profile on `profile-changed` (`App.tsx`) and passes values such as `view` down to `HostScreen` / `ViewerScreen` → `Workspace` → `TerminalView`.
- `TerminalView` creates its terminal once per session screen; its options are set in that one effect.

## Goals / Non-Goals

**Goals:**

- Typed text in the host's terminal reaches the pty once.
- Screen reader users can still get the terminal's lines read, by turning on one setting.

**Non-Goals:**

- Detecting a running screen reader (VoiceOver, Narrator, Orca) from the OS. Each platform needs its own native check, and a webview cannot tell. Could later set the default.
- Fixing the repeat while screen reader mode is on, inside or around xterm. Left to xterm upstream; the setting's hint names the trade-off.
- Any change to the viewer's input handling: a viewer's terminal sends nothing to the session, so the repeat cannot reach it there.

## Decisions

### 1. Screen reader mode off by default, on through a setting

- **Alternative: follow the OS screen reader state.** Needs a native check per platform and a way to notice it changing. More code for the same outcome for most users; left for later (Non-Goals).
- **Alternative: keep screen reader mode on and empty the text box after each key.** Works against xterm's own input handling, which relies on the text box for input methods (Chinese, Japanese) and dead keys; it would trade one input bug for others.
- **Alternative: block `insertReplacementText` and similar input events.** Only one of the paths that repeat text goes through those events; the reproduction did not involve any of them.

### 2. One setting for host and viewer

The setting is `screen_reader: bool` in `Settings`, default `false`, carried in `Profile` as `screenReader` and saved by a `set_screen_reader` command that calls `settings_changed`, like `set_notifications`. It applies to both modes: a viewer has no input to repeat, but reading the terminal matters to them as much, and one setting is simpler to explain than two.

### 3. Change it on a live terminal

`TerminalView` gets a `screenReader` prop. The terminal is still created once; a separate effect sets `term.options.screenReaderMode` whenever the prop changes. xterm supports changing this option at run time (it creates or disposes its accessibility tree on the option change), so a session in progress picks up the setting without being reopened. The value flows `App` (profile) → `HostScreen` / `ViewerScreen` → `Workspace` → `TerminalView`, the same path as `view`.

## Risks / Trade-offs

- [Screen reader users who relied on the terminal being read lose that until they turn the setting on] → The setting sits in Settings with a plain name; the Keyboard and screen readers page says to turn it on.
- [With the setting on, typing in the host's terminal can still repeat text] → The setting's hint says so. A host who uses a screen reader is the case to revisit if xterm fixes its input handling.
- [The cause is inferred from xterm's code and one reproduction, not from a test that drives a real keyboard] → A manual check in tasks: type a long line with Backspace corrections in the host terminal with the setting off (no repeat) and on (the old behavior can come back), on macOS with US and US International layouts.

## Migration Plan

Nothing to migrate: older settings files load with the setting off. Rolling back restores the always-on mode and ignores the stored field.
