# viewer-experience Specification

## Purpose

Lets each participant read and navigate a shared session at their own pace (follow, scroll, search, copy, jump to prompts) without affecting anyone else's view or the session itself.

## Requirements

### Requirement: Independent navigation
Each viewer's scroll position, selection, search and opened timeline entries SHALL be local to that viewer and SHALL NOT change the view of the host or any other viewer.

#### Scenario: Viewer scrolls back
- **WHEN** a viewer scrolls back through the terminal output
- **THEN** the host's and other viewers' views do not move

#### Scenario: Viewer opens a diff
- **WHEN** a viewer opens a diff from the timeline
- **THEN** no other participant's view changes

### Requirement: Follow and free-scroll
A viewer SHALL be able to switch between following the live output and scrolling freely. Scrolling back SHALL switch to free-scroll; returning to follow SHALL jump to the latest output.

#### Scenario: Scrolling leaves follow mode
- **WHEN** a viewer who is following scrolls up
- **THEN** the view stays where the viewer scrolled while new output arrives, and the app shows that new output is available

#### Scenario: Return to live
- **WHEN** a viewer in free-scroll chooses to follow again
- **THEN** the view jumps to the latest output and keeps following it

### Requirement: Search
A viewer SHALL be able to search the full terminal output and timeline of the session and step through the matches.

#### Scenario: Search finds earlier output
- **WHEN** a viewer searches for text that appeared earlier in the session
- **THEN** the matches are highlighted and the viewer can step through them from the start of the session to the latest output

#### Scenario: No match
- **WHEN** a viewer searches for text that does not occur
- **THEN** the app shows that there are no matches

### Requirement: Copy
A viewer SHALL be able to copy text from the terminal output, prompts, responses and diffs to their clipboard. Copying SHALL NOT send anything to the session.

#### Scenario: Copy a diff
- **WHEN** a viewer selects lines of a diff and copies them
- **THEN** the selected text is on the viewer's clipboard

### Requirement: Jump to prompt
A viewer SHALL be able to see a list of all prompts in the session and jump to any of them in both the timeline and the terminal output.

#### Scenario: Jump to an earlier prompt
- **WHEN** a viewer selects the third prompt of the session
- **THEN** the terminal and timeline scroll to that prompt and the viewer is in free-scroll

### Requirement: Read-only for viewers
The viewer interface SHALL make clear that the session is read-only and SHALL offer no control that sends input to the session.

#### Scenario: Viewer interface
- **WHEN** a viewer is in a session
- **THEN** the interface shows that they are viewing and has no input field or control for the agent session

### Requirement: Session status
Participants SHALL see whether the session is live, whether the agent is working or waiting for the host, and when the session has ended. For an agent whose events the app does not capture, working and waiting SHALL be shown as unknown rather than guessed.

#### Scenario: Agent is working
- **WHEN** Claude Code is processing a prompt
- **THEN** viewers see that the agent is working

#### Scenario: Agent without captured events
- **WHEN** the session runs an agent whose events the app does not capture
- **THEN** viewers see that the session is live and that the agent's activity is unknown

#### Scenario: Session ends
- **WHEN** the host closes the session
- **THEN** viewers see that the session has ended and can keep reading what they received until they leave

### Requirement: Status bar
A small bar at the bottom of the session window SHALL show all participants the session folder and, when it is a git repository, its branch, whether it has uncommitted changes, how far it is ahead of or behind its upstream, and the linked worktree. Values that are not known SHALL be left out rather than guessed.

#### Scenario: Session in a git repository
- **WHEN** the host runs a session in a git repository with uncommitted changes
- **THEN** every participant's status bar shows the folder and the branch marked as changed

#### Scenario: Terminal-only agent
- **WHEN** the session runs OpenCode or omp
- **THEN** the status bar shows the folder and git state only

### Requirement: Agent figures in the status bar
For a Claude Code session the status bar SHALL also show the model with its context window size, reasoning effort and fast mode, the context fill as a gauge with percentage and tokens used of the window, the session's token total, and the host account's 5-hour and 7-day rate-limit use with the time left until each resets. The host's own Claude Code status line SHALL keep working unchanged in the terminal.

#### Scenario: Claude Code answers
- **WHEN** Claude Code answers a prompt
- **THEN** every participant's status bar shows the model, the context gauge and the rate limits

#### Scenario: The host has their own status line
- **WHEN** the host's Claude Code settings define a status line
- **THEN** the terminal still shows the host's own status line, and the status bar shows SpecDeck's figures

#### Scenario: Rate limits not reported
- **WHEN** Claude Code has not reported rate limits, for example before its first answer or on an account without them
- **THEN** the status bar leaves the rate limits out
