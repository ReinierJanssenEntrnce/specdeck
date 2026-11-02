# Spec Delta

## ADDED Requirements

### Requirement: Agent details in the picker
Each choice in the agent picker SHALL show an icon, its name, and what viewers get with it: a timeline with diffs and plans, its status, or the terminal only. An installed agent SHALL also show its version and where its executable was found, and Terminal the shell it starts. An agent that is not installed SHALL offer a link to its install page, opened in the browser. Reading versions SHALL NOT delay the start screen; a version not yet known or not readable SHALL be left out.

#### Scenario: What viewers get
- **WHEN** the host looks at the picker with Claude Code, OpenCode and omp installed
- **THEN** Claude Code shows timeline, diffs and plans, and status; omp shows status and the terminal; OpenCode shows the terminal only

#### Scenario: Version and location
- **WHEN** Claude Code 2.1.309 is installed at `/opt/homebrew/bin/claude`
- **THEN** its choice shows version 2.1.309 and that path

#### Scenario: Terminal
- **WHEN** the host's login shell is `/bin/zsh`
- **THEN** the Terminal choice shows that it starts `/bin/zsh`

#### Scenario: Version read in the background
- **WHEN** the start screen opens and an agent takes several seconds to report its version
- **THEN** the picker is usable at once, without that version, and the version appears once it is known

#### Scenario: Version cannot be read
- **WHEN** an installed agent does not answer `--version` within the time limit, or answers with no version in it
- **THEN** its choice shows no version and can still be chosen

#### Scenario: Not installed
- **WHEN** omp is not installed
- **THEN** omp is not among the choices, and is listed under "Not installed" with a link that opens its install page in the browser
