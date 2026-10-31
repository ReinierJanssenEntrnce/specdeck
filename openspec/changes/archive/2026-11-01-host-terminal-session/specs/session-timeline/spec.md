## MODIFIED Requirements

### Requirement: Timeline support per agent
The timeline SHALL be built for Claude Code sessions. For an agent whose events the app does not capture yet (OpenCode and omp), and for a terminal session, the session SHALL run and be shared as a terminal mirror, and all participants SHALL see from the start that the timeline is not available for that session.

#### Scenario: OpenCode session
- **WHEN** the host starts an OpenCode session
- **THEN** the terminal is shared with viewers, and the timeline says it is not available for OpenCode

#### Scenario: Terminal session
- **WHEN** the host starts a terminal session
- **THEN** the terminal is shared with viewers, and the timeline and files views say that a terminal session has no timeline, without suggesting that one will follow

#### Scenario: Jump to prompt without a timeline
- **WHEN** a viewer is in an omp session
- **THEN** the prompt list and touched files list are not offered, and terminal search, copy and follow still work

#### Scenario: No usage without a timeline
- **WHEN** the host starts an OpenCode, omp or terminal session
- **THEN** no usage is shown
