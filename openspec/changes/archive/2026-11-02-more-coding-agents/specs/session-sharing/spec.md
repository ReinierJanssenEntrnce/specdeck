# Spec Delta

## ADDED Requirements

### Requirement: Matching app versions
A viewer SHALL only be admitted when the host's and the viewer's apps speak the same session protocol. Otherwise the viewer SHALL NOT be admitted and SHALL be told who needs to update: the viewer, when their app is older, or the host, when the host's app is older.

#### Scenario: Viewer on an older app
- **WHEN** a viewer whose app speaks an older session protocol than the host's tries to join
- **THEN** the viewer is not admitted and is told to update their app to join the session

#### Scenario: Host on an older app
- **WHEN** a viewer whose app speaks a newer session protocol than the host's tries to join
- **THEN** the viewer is not admitted and is told that the host needs to update their app
