# Spec Delta

## MODIFIED Requirements

### Requirement: Notifications
The app SHALL send OS notifications for events that need attention while its window is not focused: to the host when a viewer joins or leaves and when the agent waits for the host's input (for agents whose events the app captures, and for agents and terminal programs that report a request for the host), and to a viewer when the session ends. The user SHALL be able to turn them off.

#### Scenario: Agent waits for the host
- **WHEN** Claude Code finishes a turn or asks for permission while the host's window is in the background
- **THEN** the host gets a notification, and clicking it brings the session window to the front

#### Scenario: Reported request
- **WHEN** omp reports that it needs permission while the host's window is in the background
- **THEN** the host gets a notification with omp's message

#### Scenario: Window focused
- **WHEN** a viewer joins while the host's window has focus
- **THEN** no notification is shown

#### Scenario: Notifications turned off
- **WHEN** the user turned notifications off in settings
- **THEN** the app shows no notifications
