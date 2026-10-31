## ADDED Requirements

### Requirement: View settings per person
Each participant SHALL choose how a session is shown with two settings: whether the terminal is shown, and whether steps are shown as a plain-language summary or with technical details. The settings SHALL be remembered per user for every later session, SHALL take effect immediately, and SHALL NOT change what any other participant sees. The status bar SHALL be shown with either choice.

#### Scenario: Viewer hides the terminal
- **WHEN** a viewer turns the terminal off during a live session
- **THEN** the conversation fills the window, the viewer stays connected, and nobody else's view changes

#### Scenario: Terminal back without waiting
- **WHEN** a viewer who had the terminal off for an hour turns it back on
- **THEN** the terminal shows the current screen and full scrollback at once, without replaying the session

#### Scenario: Settings remembered
- **WHEN** a viewer chooses summary steps without the terminal, closes the app and joins another session the next day
- **THEN** that session opens without the terminal and with summary steps

#### Scenario: Host keeps the terminal
- **WHEN** the host opens the view settings
- **THEN** the host can choose summary or details for the steps, and the terminal stays shown, since the host types into it

### Requirement: First-join question
The first time a user joins a session as a viewer, the app SHALL ask how they want to follow sessions: Overview (terminal off, summary steps) or Developer (terminal on, details). The answer SHALL set both view settings, be remembered, and the question SHALL say that it can be changed later. It SHALL NOT be asked again unless the user asks for it.

#### Scenario: Product owner joins for the first time
- **WHEN** a user who never joined a session joins one and chooses Overview
- **THEN** the session shows the conversation without the terminal, with summary steps, and the next session opens the same way without asking

#### Scenario: Asked again on request
- **WHEN** the user chooses to see the question again from the view settings
- **THEN** the question is shown, and their answer replaces both settings

### Requirement: Live line
When the terminal is not shown, the conversation SHALL end with a line saying what the agent is doing right now and for how long, such as the description of the command it runs, or what it waits for when it waits for the host.

#### Scenario: Claude runs a command
- **WHEN** Claude Code has been running a shell command described as "Run the test suite" for 12 seconds and a viewer has the terminal off
- **THEN** the viewer sees a line like "Claude is running the test suite · 12 s"

#### Scenario: Claude waits for the host
- **WHEN** Claude Code asks the host for permission to run a command
- **THEN** viewers without the terminal see that Claude waits for the host, and for what

### Requirement: Plain-language steps
With summary steps, what the agent did for a prompt SHALL be shown in plain language: a shell command by the description the agent gave it, consecutive file reads as one "looked at N files" line, file changes as one "changed N files" line, and a plan change as "wrote the plan" or "updated the plan". Each line SHALL show whether it succeeded, failed or is still running, and SHALL open to its technical detail (commands, paths, outcome) on request.

#### Scenario: Claude edits files and runs tests
- **WHEN** Claude reads three files, changes two, and runs a command described as "Run the test suite" that fails
- **THEN** a summary viewer sees "Looked at 3 files", "Changed 2 files" and "Ran the test suite" marked as failed

#### Scenario: Viewer opens a step
- **WHEN** a summary viewer opens "Changed 2 files"
- **THEN** the two file paths are shown, and from there the diffs

#### Scenario: Command without a description
- **WHEN** a shell command has no description
- **THEN** the summary line says that a command ran, with its outcome, and the command itself only when the step is opened

### Requirement: Views without a timeline
For a session without a timeline, the terminal SHALL stay shown whatever the view settings are, with a short note that this session can only be followed in the terminal. The settings themselves SHALL stay unchanged for later sessions.

#### Scenario: Overview viewer joins an OpenCode session
- **WHEN** a viewer whose terminal setting is off joins an OpenCode session
- **THEN** the terminal is shown with a note why, and their next Claude Code session opens without the terminal again
