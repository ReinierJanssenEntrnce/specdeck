## MODIFIED Requirements

### Requirement: Session status
Participants SHALL see whether the session is live, whether the agent is working or waiting for the host, and when the session has ended. For an agent whose events the app does not capture, working and waiting SHALL be shown as unknown rather than guessed. A terminal session SHALL show only whether it is live or ended, since a shell has no working or waiting state.

#### Scenario: Agent is working
- **WHEN** Claude Code is processing a prompt
- **THEN** viewers see that the agent is working

#### Scenario: Agent without captured events
- **WHEN** the session runs an agent whose events the app does not capture
- **THEN** viewers see that the session is live and that the agent's activity is unknown

#### Scenario: Terminal session
- **WHEN** the session is a terminal session
- **THEN** viewers see that the session is live, with no working, waiting or unknown activity

#### Scenario: Session ends
- **WHEN** the host closes the session
- **THEN** viewers see that the session has ended and can keep reading what they received until they leave

### Requirement: Status bar
A small bar at the bottom of the session window SHALL show all participants the session folder and, when it is a git repository, its branch, whether it has uncommitted changes, how far it is ahead of or behind its upstream, and the linked worktree. Values that are not known SHALL be left out rather than guessed.

#### Scenario: Session in a git repository
- **WHEN** the host runs a session in a git repository with uncommitted changes
- **THEN** every participant's status bar shows the folder and the branch marked as changed

#### Scenario: Terminal-only agent
- **WHEN** the session runs OpenCode, omp or a terminal
- **THEN** the status bar shows the folder and git state only
