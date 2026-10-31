# session-timeline Specification

## Purpose

Turns a hosted Claude Code session into a structured, navigable timeline of prompts, responses, tool calls and file changes, so participants can follow what happened without reading raw terminal output.

## Requirements

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
Each file change made by Claude Code's file-editing tools SHALL appear in the timeline with a diff showing exactly what changed in that file. When the session folder is in a git repository, files that Claude Code's shell commands create, change or delete SHALL also appear, with their diffs, under the command that changed them. Outside a git repository, changes made by shell commands are not shown. Snapshots for this SHALL NOT change the repository.

#### Scenario: Claude edits a file
- **WHEN** Claude Code edits an existing file
- **THEN** the timeline shows a diff of that edit with removed and added lines

#### Scenario: Claude creates a file
- **WHEN** Claude Code creates a new file
- **THEN** the timeline shows the file's full content as added lines

#### Scenario: Claude edits a file through a shell command
- **WHEN** Claude Code changes a file in a git repository with a shell command, for example a script or `sed -i`
- **THEN** the timeline shows a diff of that change under the command, and the file is listed as changed

#### Scenario: Claude deletes a file through a shell command
- **WHEN** Claude Code deletes a file in a git repository with a shell command
- **THEN** the timeline shows the file's former content as removed lines, and the file is listed as deleted

#### Scenario: Snapshots leave the repository alone
- **WHEN** Claude Code runs shell commands in a git repository
- **THEN** the repository's index, objects and refs are unchanged by SpecDeck

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
### Requirement: Document text
When Claude Code creates or changes a Markdown file, the host SHALL send the file's full current text to all participants with the change, so it can be read as a whole. Only files Claude Code created or changed SHALL be sent, and a document larger than 200 KB SHALL be sent as a note that it is too large, with its diffs still shown. Deleting a document SHALL be shown as deleted, without text.

#### Scenario: Claude edits a document
- **WHEN** Claude Code edits three lines of a 200-line `proposal.md`
- **THEN** participants receive the diff and the document's full new text

#### Scenario: Change through a shell command
- **WHEN** a shell command changes a Markdown file in a git repository
- **THEN** participants receive its full new text with the change, as for Claude's file tools

#### Scenario: Very large document
- **WHEN** Claude Code changes a 1 MB Markdown file
- **THEN** participants see its diffs and a note that the document is too large to show whole

#### Scenario: Source code is not sent whole
- **WHEN** Claude Code edits a source file such as `server.ts`
- **THEN** participants receive only its diff, as before

### Requirement: Plan state
When Claude Code creates or changes a file inside an OpenSpec change (`openspec/changes/<name>/`), the host SHALL send that change's plan state to all participants: the change's name, its documents in the order and with the names its schema gives them, which of them exist, and how many of its tasks are done. The plan state SHALL be sent again whenever a file of the change changes, and it SHALL be read from the OpenSpec CLI on the host when installed, else from OpenSpec's default layout.

#### Scenario: Claude proposes a change
- **WHEN** Claude Code writes the proposal, specs, design and tasks of `host-terminal-session`
- **THEN** participants receive a plan state naming that change with those documents and "0 of 8" tasks done

#### Scenario: Tasks ticked
- **WHEN** Claude Code ticks a task in the change's tasks file
- **THEN** participants receive the plan state with one more task done

#### Scenario: Custom schema
- **WHEN** the change uses a schema whose first document is a feature brief
- **THEN** the plan state lists the feature brief first, with the schema's name for it

#### Scenario: OpenSpec CLI not installed
- **WHEN** the OpenSpec CLI is not on the host and Claude writes `openspec/changes/x/proposal.md`
- **THEN** participants receive a plan state for `x` built from the default layout: proposal, specs, design and tasks
