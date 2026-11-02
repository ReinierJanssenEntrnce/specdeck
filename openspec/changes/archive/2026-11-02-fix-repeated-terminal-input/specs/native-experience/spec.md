# Spec Delta

## ADDED Requirements

### Requirement: Screen reader support in the terminal
The terminal SHALL make its lines available to the platform's screen reader only while the user's Screen reader support setting is on. The setting SHALL be off until the user turns it on, SHALL be remembered per user, SHALL apply to the host's and a viewer's terminal alike, and SHALL take effect at once, also in an open session.

#### Scenario: Off by default
- **WHEN** a user who never changed the setting hosts or joins a session
- **THEN** the terminal does not expose its lines to the screen reader

#### Scenario: Turned on during a session
- **WHEN** a user turns on Screen reader support in Settings while a session is open
- **THEN** the screen reader can read the terminal's lines in that session without leaving or rejoining it

#### Scenario: Remembered
- **WHEN** a user who turned on Screen reader support starts the app again
- **THEN** the setting is still on

#### Scenario: Only for that person
- **WHEN** the host turns on Screen reader support
- **THEN** no viewer's terminal changes
