# Spec Delta

## Purpose

Turns a hosted Claude Code session into a structured, navigable timeline of prompts, responses, tool calls and file changes, so participants can follow what happened without reading raw terminal output.

## ADDED Requirements

### Requirement: Capture prompts
Each prompt the host submits to Claude Code SHALL appear as a timeline entry with its full text and the time it was submitted.

#### Scenario: Host submits a prompt
- **WHEN** the host submits a prompt in the session
- **THEN** a prompt entry with the full text and time appears in the timeline for all participants

### Requirement: Capture responses
Claude's response text SHALL appear in the timeline under the prompt it answers, while Claude is writing it, including text written between tool calls. Markdown in responses SHALL be rendered.

#### Scenario: Claude answers
- **WHEN** Claude Code responds to a prompt
- **THEN** the response text appears under that prompt and grows as Claude writes

#### Scenario: Response with a code block
- **WHEN** a response contains a fenced code block and a list
- **THEN** the timeline shows them formatted as a code block and a list

### Requirement: Capture tool calls
Each tool Claude Code uses SHALL appear as a timeline entry showing the tool, its input, and whether it succeeded, failed, or did not run.

#### Scenario: Claude runs a command
- **WHEN** Claude Code runs a shell command
- **THEN** a tool entry shows the command and whether it succeeded

#### Scenario: Tool call fails or is denied
- **WHEN** a tool call fails, or the host denies it
- **THEN** the tool entry shows that it failed or did not run

### Requirement: File change diffs
Each file change made by Claude Code's file-editing tools SHALL appear in the timeline with a diff showing exactly what changed in that file.

#### Scenario: Claude edits a file
- **WHEN** Claude Code edits an existing file
- **THEN** the timeline shows a diff of that edit with removed and added lines

#### Scenario: Claude creates a file
- **WHEN** Claude Code creates a new file
- **THEN** the timeline shows the file's full content as added lines

### Requirement: Touched files list
The app SHALL list every file Claude Code read, created or changed during the session, and SHALL let a participant open all diffs of a changed file. Files Claude did not touch SHALL NOT be browsable through the app.

#### Scenario: Open a touched file
- **WHEN** a participant selects a changed file in the touched files list
- **THEN** the app shows all diffs of that file made during the session, in order

#### Scenario: Untouched file
- **WHEN** a participant looks for a repository file that Claude Code never touched
- **THEN** the app does not offer it

### Requirement: Timeline stays in sync with the terminal
Timeline entries SHALL appear for all participants while the session runs, in the order the events happened, and each entry SHALL link to its position in the terminal output.

#### Scenario: Jump from timeline to terminal
- **WHEN** a participant selects a timeline entry
- **THEN** their terminal view scrolls to the output that belongs to that entry

#### Scenario: Late joiner sees full timeline
- **WHEN** a viewer joins mid-session
- **THEN** the viewer receives every timeline entry from the start of the session

### Requirement: Session usage
For a Claude Code session, all participants SHALL see how full the agent's context window is, the total tokens used in the session, and the tokens used per prompt. The figures SHALL update while the session runs and SHALL be complete for viewers who join late. Money cost SHALL NOT be shown. Where participants see these figures is set by the status bar requirement in `viewer-experience`.

#### Scenario: Claude answers a prompt
- **WHEN** Claude Code finishes answering a prompt
- **THEN** every participant sees the context fill, the session total and that prompt's tokens updated

#### Scenario: Late joiner sees usage
- **WHEN** a viewer joins mid-session
- **THEN** they see the same totals and per-prompt tokens as the host

#### Scenario: Context compacted
- **WHEN** Claude Code compacts its context
- **THEN** the context fill drops, and the session total does not

#### Scenario: Usage cannot be read
- **WHEN** the transcript is missing or unreadable
- **THEN** usage is shown as unavailable, and the session and timeline keep working

### Requirement: Degraded mode without structured events
If structured events cannot be captured, the session SHALL still run and be shared as a terminal mirror, and all participants SHALL be told that the timeline is unavailable. If events arrive again, the timeline SHALL resume.

#### Scenario: Event capture fails
- **WHEN** the host submits a prompt and no structured event arrives
- **THEN** the terminal mirror keeps working and the timeline shows that it is unavailable

#### Scenario: Events return
- **WHEN** structured events arrive again after the timeline was marked unavailable
- **THEN** the timeline shows new entries again

### Requirement: Timeline support per agent
The timeline SHALL be built for Claude Code sessions. For an agent whose events the app does not capture yet (OpenCode and omp), the session SHALL run and be shared as a terminal mirror, and all participants SHALL see from the start that the timeline is not available for that agent.

#### Scenario: OpenCode session
- **WHEN** the host starts an OpenCode session
- **THEN** the terminal is shared with viewers, and the timeline says it is not available for OpenCode

#### Scenario: Jump to prompt without a timeline
- **WHEN** a viewer is in an omp session
- **THEN** the prompt list and touched files list are not offered, and terminal search, copy and follow still work

#### Scenario: No usage without a timeline
- **WHEN** the host starts an OpenCode or omp session
- **THEN** no usage is shown
