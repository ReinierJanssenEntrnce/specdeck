# Spec Delta

## Purpose

Lets a user on Windows, macOS or Linux start and run a coding agent (Claude Code, OpenCode or omp) in a terminal session on their own machine as the host, using their own agent login, with exclusive control over that session.

## ADDED Requirements

### Requirement: App role per session
The app SHALL let the user choose, per session, to host a new session or to join an existing one as a viewer. The same installed app SHALL support both roles on Windows, macOS and Linux.

#### Scenario: Choose host
- **WHEN** the user chooses to host a session
- **THEN** the app starts the host flow and does not ask for an invite code

#### Scenario: Choose viewer
- **WHEN** the user chooses to join a session
- **THEN** the app asks for an invite code and does not start a local Claude Code session

### Requirement: Username
The app SHALL require a username before the user hosts or joins, and SHALL show it to other participants. The username is a display name, not an authenticated identity.

#### Scenario: No username set
- **WHEN** a user without a username tries to host or join
- **THEN** the app asks for a username first

#### Scenario: Username shown to others
- **WHEN** a participant with username "nick" is in a session
- **THEN** every other participant sees "nick" in the participant list

### Requirement: Choose a coding agent
The host SHALL choose which coding agent a session runs: Claude Code, OpenCode or omp. The app SHALL show which of them are installed, preselect the agent the host used last, and not offer to start an agent that is not installed.

#### Scenario: Several agents installed
- **WHEN** Claude Code and OpenCode are installed and omp is not
- **THEN** the host can choose Claude Code or OpenCode, and omp is shown as not installed

#### Scenario: Last used agent
- **WHEN** the host's previous session ran OpenCode
- **THEN** OpenCode is preselected for the next session

### Requirement: Start an agent session
When hosting, the app SHALL start the chosen agent's locally installed CLI in a terminal in a folder the host selects, using the host's existing login for that agent. The app SHALL NOT authenticate to the agent's model provider or send requests to it itself.

#### Scenario: Host starts a session
- **WHEN** the host selects a folder and starts a Claude Code session
- **THEN** a terminal opens in that folder running `claude`, and the host can use it as a normal terminal

#### Scenario: Host starts an OpenCode or omp session
- **WHEN** the host selects a folder and starts an OpenCode or omp session
- **THEN** a terminal opens in that folder running `opencode` or `omp`, and the host can use it as a normal terminal

#### Scenario: Agent not installed
- **WHEN** the host starts a session and the chosen agent's CLI cannot be found
- **THEN** the app explains that the agent must be installed and logged in, and no session is created

#### Scenario: Missing Windows prerequisite
- **WHEN** the host starts a Claude Code session on Windows and Git Bash, which Claude Code needs, cannot be found
- **THEN** the app names the missing prerequisite, and no session is created

### Requirement: One hosted session at a time
The app SHALL host at most one session at a time. While a session is hosted, starting another SHALL require closing the current one first.

#### Scenario: Host tries to start a second session
- **WHEN** a session is running and the host chooses to host another folder
- **THEN** the app asks whether to close the running session first, and starts nothing unless the host confirms

### Requirement: Session survives closing the window
Closing the host's window SHALL NOT end a hosted session. The session SHALL keep running and viewers SHALL stay connected until the host closes the session or quits the app.

#### Scenario: Host closes the window
- **WHEN** the host closes the app window while a session is running
- **THEN** Claude Code keeps running, viewers stay connected, and the app stays available from the tray or menu bar

#### Scenario: Host reopens the window
- **WHEN** the host reopens the window of a running session
- **THEN** the window shows the session's current terminal state and full scrollback

### Requirement: Quitting while hosting
Quitting the app while a session is hosted SHALL ask the host to confirm, because quitting ends the session for everyone.

#### Scenario: Host quits with viewers connected
- **WHEN** the host quits the app while a session with 3 viewers is running
- **THEN** the app asks for confirmation, naming that 3 viewers will be disconnected, and ends the session only if the host confirms

### Requirement: Host owns input
Only the host SHALL be able to send input to the session. Input from any other participant SHALL be ignored.

#### Scenario: Host types a prompt
- **WHEN** the host types into the terminal
- **THEN** the input reaches the Claude Code session

#### Scenario: Input from a viewer
- **WHEN** a message carrying terminal input arrives from a viewer
- **THEN** the session does not receive it

### Requirement: Host owns terminal size
The session's terminal size SHALL be set by the host only. Viewer window sizes SHALL NOT change it.

#### Scenario: Host resizes
- **WHEN** the host resizes their terminal view
- **THEN** the session's terminal size changes and all viewers receive the new size

#### Scenario: Viewer resizes
- **WHEN** a viewer resizes their window
- **THEN** the session's terminal size does not change

### Requirement: Close a session
The host SHALL be able to close a session. Closing SHALL end the agent process, disconnect all viewers and invalidate the invite code.

#### Scenario: Host closes the session
- **WHEN** the host closes the session
- **THEN** the agent process ends, every viewer is shown that the session has ended, and the invite code no longer works

#### Scenario: Agent exits on its own
- **WHEN** the agent process exits while the session is running
- **THEN** the host and all viewers see that it has exited, and the session stays open until the host closes it

### Requirement: Recent folders
The app SHALL remember up to five folders the user most recently hosted in, newest first, each once, and offer them on the start screen and in the platform's recent-items menu. Selecting one SHALL start a session there without a folder dialog.

#### Scenario: Host again in the same folder
- **WHEN** a host who earlier hosted in `~/projects/shop` selects it from the recent folders
- **THEN** a session starts in `~/projects/shop` with the currently chosen agent, without a folder dialog

#### Scenario: Most recent first
- **WHEN** a host hosts in folder A, then in folder B
- **THEN** the recent folders show B above A, each once

#### Scenario: Session fails to start
- **WHEN** a session cannot start in the chosen folder, for example because the agent is not installed
- **THEN** the folder is not added to the recent folders

#### Scenario: Folder no longer exists
- **WHEN** a recent folder has been deleted or moved
- **THEN** the app does not list it

#### Scenario: Remove a folder from the list
- **WHEN** the host removes a folder from the recent folders
- **THEN** it is no longer listed, and nothing on disk changes
