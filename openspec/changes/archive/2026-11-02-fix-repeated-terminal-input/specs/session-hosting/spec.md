# Spec Delta

## MODIFIED Requirements

### Requirement: Host owns input
Only the host SHALL be able to send input to the session. Input from any other participant SHALL be ignored.

#### Scenario: Host types a prompt
- **WHEN** the host types into the terminal
- **THEN** the input reaches the Claude Code session

#### Scenario: Input from a viewer
- **WHEN** a message carrying terminal input arrives from a viewer
- **THEN** the session does not receive it

#### Scenario: Typed text arrives once
- **WHEN** the host types a line longer than the terminal is wide, at any speed and with corrections by Backspace, then presses Enter
- **THEN** the session receives each typed character once, in the order typed, and no part of the line is sent again
