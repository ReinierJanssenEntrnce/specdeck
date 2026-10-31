# Tasks

## 1. Protocol and engine

- [x] 1.1 Add `AgentId::Terminal` (`"terminal"`, display name "Terminal", no timeline) to `crates/protocol` and include it in `AgentId::ALL` after the coding agents; verify the protocol tests cover its wire name and that `has_timeline()` is false
- [x] 1.2 Resolve the shell in the agent adapter (design decision 2): `$SHELL` if it is an executable file, else `/bin/zsh` on macOS, `/bin/bash`, `/bin/sh`, started with `-l`; on Windows `pwsh.exe` if on `PATH`, else `powershell.exe`, with `-NoLogo`. `detect` reports Terminal installed when a shell resolves, and the launcher takes executable and arguments from the adapter, with the error "No shell was found." when none resolves. Verify with unit tests for `$SHELL` set, `$SHELL` missing or not executable (fallback order), and the Windows choice between pwsh and powershell (with a fake `PATH`)
- [x] 1.3 Verify a terminal session end to end in `crates/core/tests/host_session.rs`: with `$SHELL` pointing at a fake shell script, the session starts it in the folder with `-l` and without parent Claude Code markers, takes host input, writes no hook settings, has status `unknown`, reports the folder's git state, and stays open after the shell exits

## 2. App

- [x] 2.1 List Terminal in the app's agent list (always last, detail "Your shell"), remember it as the last choice, and keep tray and notification texts correct for it; verify with the app crate tests and that `get_profile` returns Terminal as installed on this Mac
- [x] 2.2 Start screen (design decision 4): Terminal in the choice, the new lead text, the reminder line while Terminal is selected, and the hint instead of the error when no coding agent is installed. Verify with UI tests: Terminal selectable with no agent installed, the reminder shown only for Terminal and starting needs no extra click, last choice preselected
- [x] 2.3 Session views for Terminal: Timeline and Files empty states, the status pill ("Live" without the tooltip, "Shell exited"), and the host banner after exit; verify with UI tests for each text, for host and viewer, and that the a11y test still passes
- [x] 2.4 Update `docs/user-guide.md` (requirements, Host a session, the status explanation) for terminal sessions, including the reminder about secrets; verify each step against the running app
  - Checked on 2026-10-31 with the packaged app: Terminal is offered as "Your shell", the secrets reminder shows only while Terminal is selected, the folder is remembered under recent folders, the status reads Live and then Shell exited with the host banner, and a viewer who hides the terminal keeps it with a note. The tray menu itself was not checked (hidden by a crowded menu bar); its texts are covered by the app crate tests

## 3. Integration

- [x] 3.1 Host a terminal session from the packaged app launched from Finder, join it from the second dev app, run a few commands (including `git status` and a full-screen program such as `vim`), exit the shell, and check that the viewer sees everything and the session stays open until it is ended; record the result in the task and note any difference on Windows and Linux for `release-and-verification` task 3.4
  - Done on 2026-10-31 on macOS (packaged app opened from Finder, viewer from the debug build): the login shell started with the user's prompt; `git status`, `vim` (insert mode, `:wq`) and `exit` all reached the viewer as on the host; after `exit` both showed Shell exited, the session stayed open, and the Timeline and Files tabs said a terminal session has none. Ending the session crashed the app on macOS: the tray icon was removed off the main thread. Fixed by removing it on the main thread; the app now returns to its start screen and the icon is gone. For 3.4: check on Windows and Linux that ending a session removes the tray icon without a crash, and that PowerShell and the Linux login shell start as the host's shell

## Workflow follow-up

- When archiving, update the session-hosting main spec's Purpose, which says hosting runs "a coding agent", to include terminal sessions.
