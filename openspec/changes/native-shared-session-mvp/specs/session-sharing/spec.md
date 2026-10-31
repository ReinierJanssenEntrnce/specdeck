# Spec Delta

## Purpose

Controls who can join a hosted session and how remote participants connect: invite codes and join links, admitting and removing viewers, and encrypted peer-to-peer connections that work across networks.

## ADDED Requirements

### Requirement: Invite code
When a session starts, the app SHALL generate an invite code for it and show it to the host with a way to copy it. The code SHALL be hard to guess and valid only for that session.

#### Scenario: Code generated
- **WHEN** the host starts a session
- **THEN** the app shows an invite code, for example `7K2M-9QXH-4TPA`, and a way to copy it

#### Scenario: Code for another session
- **WHEN** a viewer enters an invite code that belonged to a different, closed session
- **THEN** the join is refused

### Requirement: Join with invite code
A viewer SHALL join a session by entering its invite code. Case and dashes in the code SHALL NOT matter. An invalid or expired code SHALL be refused with a clear message.

#### Scenario: Valid code
- **WHEN** a viewer enters a valid invite code for an open session
- **THEN** the viewer joins, and the host sees the viewer's username in the participant list

#### Scenario: Code typed differently
- **WHEN** a viewer enters `7k2m9qxh4tpa` for the code `7K2M-9QXH-4TPA`
- **THEN** the viewer joins as with the original code

#### Scenario: Invalid code
- **WHEN** a viewer enters a code that does not match any open session
- **THEN** the app shows that the code is invalid or has expired

#### Scenario: Repeated wrong codes
- **WHEN** many invalid codes are tried from the same client in a short time
- **THEN** further attempts are temporarily refused, and the app says to wait

### Requirement: Join link
The host SHALL be able to copy a join link that contains the invite code. Opening the link on a machine with the app installed SHALL open the app and join the session.

#### Scenario: Viewer opens a join link
- **WHEN** a viewer with the app installed opens a join link from a chat message
- **THEN** the app opens and joins the session, asking for a username first if none is set

#### Scenario: Join link while in another session
- **WHEN** a viewer who is already viewing a session opens a join link for a different session
- **THEN** the app asks whether to leave the current session before joining

### Requirement: Invite code lifetime
The invite code SHALL expire when the host closes the session. The host SHALL be able to revoke it while the session is open and get a new one; revoking SHALL NOT disconnect viewers who already joined.

#### Scenario: Session closed
- **WHEN** the host closes the session
- **THEN** its invite code and join link no longer work

#### Scenario: Host revokes the code
- **WHEN** the host revokes the invite code during a session
- **THEN** the old code no longer works, a new code is shown, and connected viewers stay connected

### Requirement: Participant list
All participants SHALL see who is in the session: the host and each connected viewer by username, updated as people join and leave.

#### Scenario: Viewer leaves
- **WHEN** a viewer leaves the session
- **THEN** the viewer disappears from every participant's list

#### Scenario: Duplicate usernames
- **WHEN** two participants use the same username
- **THEN** both are listed and can be told apart

### Requirement: Remove a viewer
The host SHALL be able to remove any viewer. A removed viewer SHALL be disconnected immediately and SHALL NOT be able to rejoin with the invite code that was valid when they were removed.

#### Scenario: Host removes a viewer
- **WHEN** the host removes a viewer
- **THEN** that viewer is disconnected, is told they were removed, and cannot rejoin until the host shares a new code

### Requirement: Viewer limit
The host SHALL be able to set the session's viewer limit from 1 to 20, with 20 as the default. A join over the limit SHALL be refused; lowering the limit SHALL NOT disconnect viewers who already joined.

#### Scenario: Session full
- **WHEN** a viewer tries to join a session that has as many viewers as its limit
- **THEN** the join is refused with a message that the session is full

#### Scenario: Limit below current viewers
- **WHEN** the host sets the limit to 5 while 8 viewers are connected
- **THEN** all 8 stay connected and no new viewer can join until fewer than 5 are connected

### Requirement: Remote connectivity
Viewers SHALL be able to join from a different network than the host, including networks behind NAT or firewalls, without a VPN and without the host opening ports.

#### Scenario: Viewer on a different network
- **WHEN** host and viewer are on different home networks behind NAT
- **THEN** the viewer can join and receive the session

#### Scenario: Direct connection blocked
- **WHEN** a network blocks direct peer-to-peer connections
- **THEN** the connection goes through the team's relay and the session still works

### Requirement: End-to-end encryption
All session content (terminal output, timeline, diffs) SHALL be encrypted between host and viewer. The matchmaking service and the relay SHALL NOT be able to read session content or learn the invite code.

#### Scenario: Traffic through the relay
- **WHEN** a viewer's traffic passes through the team's relay
- **THEN** the relay only handles encrypted data and cannot read terminal output, prompts or diffs

#### Scenario: Matchmaking service tries to intercept
- **WHEN** the matchmaking service substitutes its own connection details during a join
- **THEN** the viewer's app rejects the connection and does not show any session content

### Requirement: Matchmaking service unavailable
If the matchmaking service is unreachable, the app SHALL tell the user that sharing is unavailable. The host SHALL still be able to run the session locally, and the app SHALL keep retrying until sharing works.

#### Scenario: Service down when hosting
- **WHEN** the host starts a session while the matchmaking service is unreachable
- **THEN** the session runs locally, the host is told sharing is unavailable, and the invite code appears once the service is reachable

#### Scenario: Service down when joining
- **WHEN** a viewer tries to join while the matchmaking service is unreachable
- **THEN** the app says the sharing service cannot be reached
