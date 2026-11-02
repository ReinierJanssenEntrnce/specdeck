# Spec Delta

## MODIFIED Requirements

### Requirement: Choose a coding agent
The host SHALL choose what a session runs: a coding agent (Claude Code, OpenCode, omp, Codex, Grok or pi) or a plain terminal. The app SHALL show which agents are installed, preselect the choice the host used last, and not offer to start an agent that is not installed.

#### Scenario: Several agents installed
- **WHEN** Claude Code and OpenCode are installed and omp is not
- **THEN** the host can choose Claude Code, OpenCode or Terminal, and omp is shown as not installed

#### Scenario: Last used agent
- **WHEN** the host's previous session ran OpenCode
- **THEN** OpenCode is preselected for the next session

#### Scenario: Last used terminal
- **WHEN** the host's previous session was a terminal session
- **THEN** Terminal is preselected for the next session

#### Scenario: Codex, Grok or pi
- **WHEN** Codex is installed and the host chooses it and a folder
- **THEN** the session runs `codex` in that folder, and viewers see the agent named Codex with its terminal and no timeline

#### Scenario: Status of a new agent
- **WHEN** a session runs an agent that reports its state through the Program Status Protocol, such as Codex
- **THEN** participants see whether it is working or waiting, as for omp
