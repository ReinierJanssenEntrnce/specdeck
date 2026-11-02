# Spec Delta

## MODIFIED Requirements

### Requirement: Session status
Participants SHALL see whether the session is live, whether the agent is working or waiting for the host, and when the session has ended. For an agent whose events the app does not capture, working and waiting SHALL be shown as unknown rather than guessed, unless the agent reports its own state (see `agent-status`). A terminal session SHALL show whether it is live or ended, and, while a program in it reports its state, whether that program is working or waiting.

#### Scenario: Agent is working
- **WHEN** Claude Code is processing a prompt
- **THEN** viewers see that the agent is working

#### Scenario: Agent without captured events
- **WHEN** the session runs an agent whose events the app does not capture and that does not report its state
- **THEN** viewers see that the session is live and that the agent's activity is unknown

#### Scenario: Agent that reports its state
- **WHEN** the session runs omp and omp reports that it is working
- **THEN** viewers see that the agent is working

#### Scenario: Terminal session
- **WHEN** the session is a terminal session and no program in it reports its state
- **THEN** viewers see that the session is live, with no working, waiting or unknown activity

#### Scenario: Agent started in a terminal session
- **WHEN** the host starts `claude` in a terminal session and Claude Code reports that it is working
- **THEN** viewers see that a program in the terminal is working, and see only that the session is live again after Claude Code exits

#### Scenario: Session ends
- **WHEN** the host closes the session
- **THEN** viewers see that the session has ended and can keep reading what they received until they leave

## ADDED Requirements

### Requirement: Request shown with the status
While the agent waits on a request for the host, a session without a timeline SHALL show the request's message with the status, until the agent stops waiting. A session with a timeline SHALL show the request in the timeline only.

#### Scenario: Request in a session without a timeline
- **WHEN** omp reports that it needs permission with the message "approve Write: src/main.rs"
- **THEN** viewers see that it is waiting for the host, with that message

#### Scenario: Agent goes on
- **WHEN** the host allows the request and omp reports working
- **THEN** viewers see that omp is working, without the message
