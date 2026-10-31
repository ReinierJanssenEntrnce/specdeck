## ADDED Requirements

### Requirement: Terminal session
Besides the coding agents, the host SHALL be able to start a plain terminal session, which runs the host's own shell in the selected folder: the host's login shell on macOS and Linux, and PowerShell on Windows. Terminal SHALL always be offered, since it needs nothing installed beyond the operating system.

#### Scenario: Host starts a terminal session on macOS
- **WHEN** the host whose login shell is zsh selects Terminal and a folder
- **THEN** zsh starts in that folder with the host's usual environment, and the host can use it as a normal terminal

#### Scenario: Host starts a terminal session on Windows
- **WHEN** the host selects Terminal and a folder on Windows
- **THEN** PowerShell starts in that folder

#### Scenario: No coding agent installed
- **WHEN** none of Claude Code, OpenCode and omp is installed
- **THEN** the host can still choose Terminal and start a session, and the app says that a coding agent must be installed for a timeline

#### Scenario: Shell exits
- **WHEN** the host types `exit` in a terminal session
- **THEN** the session stays open for viewers to read until the host ends it, as when an agent exits on its own

### Requirement: Reminder that viewers see everything
When the host selects Terminal, the start screen SHALL remind the host that viewers see all terminal output, including any secrets a command prints, before the session starts. The reminder SHALL NOT require an extra confirmation step.

#### Scenario: Terminal selected
- **WHEN** the host selects Terminal on the start screen
- **THEN** a short note says that everyone invited sees all output, including secrets a command prints, and starting a session needs no extra click

## MODIFIED Requirements

### Requirement: Choose a coding agent
The host SHALL choose what a session runs: a coding agent (Claude Code, OpenCode or omp) or a plain terminal. The app SHALL show which agents are installed, preselect the choice the host used last, and not offer to start an agent that is not installed.

#### Scenario: Several agents installed
- **WHEN** Claude Code and OpenCode are installed and omp is not
- **THEN** the host can choose Claude Code, OpenCode or Terminal, and omp is shown as not installed

#### Scenario: Last used agent
- **WHEN** the host's previous session ran OpenCode
- **THEN** OpenCode is preselected for the next session

#### Scenario: Last used terminal
- **WHEN** the host's previous session was a terminal session
- **THEN** Terminal is preselected for the next session
