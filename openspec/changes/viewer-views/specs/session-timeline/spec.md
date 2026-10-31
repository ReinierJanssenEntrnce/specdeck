## ADDED Requirements

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
