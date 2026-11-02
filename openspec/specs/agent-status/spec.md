# agent-status Specification

## Purpose

Works out what the agent in a hosted session is doing (working, waiting for the host, exited or unknown) from the sources the session offers: the agent's hook events and the status reports a program writes into the terminal.

## Requirements

### Requirement: Program status reports
The host SHALL read status reports that a program in the session writes into the terminal using the Program Status Protocol (OSC 7501), and SHALL answer the protocol's query so that programs which wait for an answer start reporting. Each query SHALL get exactly one answer, whether or not the host's window is open. Reports SHALL NOT change what the terminal shows.

#### Scenario: omp reports its state
- **WHEN** the host runs an omp session and omp reports that it is working
- **THEN** the session's status becomes working

#### Scenario: Claude Code asks whether the terminal supports reports
- **WHEN** Claude Code sends the OSC 7501 query while the host's window is open
- **THEN** it receives exactly one answer, and starts sending reports

#### Scenario: Query without a host window
- **WHEN** a program sends the OSC 7501 query while the host's window is closed
- **THEN** it receives exactly one answer

#### Scenario: Viewers do not answer
- **WHEN** a viewer's terminal replays output that contains an OSC 7501 query
- **THEN** nothing from the viewer reaches the session

### Requirement: Status from reports
The status SHALL follow the program's report about itself: working while it reports working, waiting for the host while it reports idle, done, blocked or error. Reports about a program's children, such as its subagents, SHALL NOT change the status. When no report is in force, the status SHALL be what it would be without reports.

#### Scenario: Program finishes a task
- **WHEN** omp reports working and then done
- **THEN** the status goes from working to waiting for the host

#### Scenario: Subagent report
- **WHEN** Claude Code reports a blocked subagent while it is itself working
- **THEN** the status stays working

### Requirement: Requests for the host from reports
When a program reports that it is blocked on the host (a permission, a question or a sign-in), or that it failed, the host SHALL create a notice that the program waits for the host, with the program's own message when it gives one. A program that repeats the same report SHALL NOT create another notice, and a program SHALL NOT create more than one notice every two seconds.

#### Scenario: Permission
- **WHEN** omp reports blocked for permission with the message "approve Write: src/main.rs"
- **THEN** participants see a notice that omp needs permission, with that message

#### Scenario: Same report again
- **WHEN** a program sends the same blocked report three times in a row
- **THEN** one notice is created

#### Scenario: Flood of reports
- **WHEN** a program sends a different blocked report 100 times in one second
- **THEN** at most one notice is created in that second, and the status still follows the latest report

### Requirement: Report text is cleaned
A report's message SHALL be shown only after it is cleaned: a message with control characters SHALL be dropped, invisible formatting and text-direction characters SHALL be removed, and a message SHALL be cut to 300 characters. A report that breaks the protocol's limits or encoding SHALL be ignored.

#### Scenario: Message with an escape sequence
- **WHEN** a report's message decodes to text containing an escape character
- **THEN** the report is used for the status, and its message is not shown

#### Scenario: Very long message
- **WHEN** a report's message is 2,000 characters long
- **THEN** participants see its first 300 characters

#### Scenario: Malformed report
- **WHEN** a report's message is not valid base64
- **THEN** the report is ignored

### Requirement: Reports end with the program
A program's reported state SHALL end when the program clears it, when the program exits, or when the terminal is reset. In a terminal session on macOS and Linux, a reported state SHALL also end within two seconds of the shell being back in the foreground. A prompt mark (OSC 133) SHALL NOT end a reported state, since agents write prompt marks for their own commands while they run.

#### Scenario: Claude Code exits in a terminal session
- **WHEN** the host runs `claude` in a terminal session, it reports idle, and the host quits it
- **THEN** the session's status goes back to showing only that the session is live

#### Scenario: Program exits without clearing
- **WHEN** a program in a terminal session on macOS reports working and is killed before it can clear its report
- **THEN** within two seconds the session's status goes back to showing only that the session is live

#### Scenario: Prompt marks while working
- **WHEN** omp reports working and then writes a prompt mark for a command it runs
- **THEN** the status stays working

#### Scenario: Agent session exits
- **WHEN** an omp session's process exits after reporting working
- **THEN** the status is exited

### Requirement: Hooks before reports
For an agent whose hook events the app captures, the hook events SHALL decide the status while the timeline is available, and reports SHALL NOT create notices then. While the timeline is unavailable, the status and notices SHALL follow the program's reports when it sends them. When hook events arrive again, they SHALL decide the status again.

#### Scenario: Hooks working
- **WHEN** Claude Code's hooks report a tool call and its OSC 7501 report says idle
- **THEN** the status is working

#### Scenario: Hooks broken
- **WHEN** the timeline is unavailable for a Claude Code session and Claude Code reports working
- **THEN** the status is working

### Requirement: Waiting on a question or plan
While Claude Code shows the host a question (`AskUserQuestion`) or a plan to approve (`ExitPlanMode`), the status SHALL be waiting for the host. It SHALL become working again when the question is answered or the plan is decided.

#### Scenario: Claude asks a question
- **WHEN** Claude Code asks "Which database?" with two options
- **THEN** the status is waiting for the host until the host picks one, then working

#### Scenario: Plan approval
- **WHEN** Claude Code presents a plan for approval
- **THEN** the status is waiting for the host until the host approves or rejects it

### Requirement: Manual compact
While the host compacts Claude Code's context by hand, the status SHALL be working, and when the compaction finishes it SHALL be waiting for the host. A manual compaction SHALL NOT make the timeline unavailable, however long it takes. An automatic compaction during a turn SHALL NOT change the status.

#### Scenario: Host runs /compact
- **WHEN** the host runs `/compact`, which Claude Code reports without a prompt or a turn end
- **THEN** the status is working until the compaction finishes, then waiting for the host

#### Scenario: Long compaction
- **WHEN** a manual compaction takes 40 seconds
- **THEN** the timeline stays available

#### Scenario: Automatic compaction
- **WHEN** Claude Code compacts automatically while working on a prompt
- **THEN** the status stays working
