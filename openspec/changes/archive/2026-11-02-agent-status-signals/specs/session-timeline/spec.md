# Spec Delta

## ADDED Requirements

### Requirement: Permission requests name the action
When Claude Code asks the host for permission, the timeline SHALL show one notice saying what is asked: for a tool, the tool and what it would act on (the command for a shell tool, the file path for a file tool, the URL for a web tool); for a question, the question; for a plan, that a plan waits for review. The request SHALL be linked to the tool call it is for, also when other tool calls are open.

#### Scenario: Permission for a command
- **WHEN** Claude Code asks permission to run `rm -rf build`
- **THEN** the timeline shows one notice that Claude wants to use Bash, with `rm -rf build`

#### Scenario: Question
- **WHEN** Claude Code asks "Which database?" with `AskUserQuestion`
- **THEN** the timeline shows one notice "Claude asks: Which database?"

#### Scenario: Plan
- **WHEN** Claude Code presents a plan with `ExitPlanMode`
- **THEN** the timeline shows one notice that Claude has a plan for the host to review

#### Scenario: Several tools open
- **WHEN** a helper's tool call is open and Claude Code asks permission for its own `Write` to `src/main.rs`
- **THEN** the allow or deny decision is shown on the `Write` call, not on the helper's call

#### Scenario: MCP dialog in a browser
- **WHEN** an MCP server asks the host for input through a browser dialog
- **THEN** the timeline shows a notice that Claude waits for the host
