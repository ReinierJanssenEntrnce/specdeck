## MODIFIED Requirements

### Requirement: Resource footprint
The app SHALL stay small and quick: an installer of at most 25 MB per platform, a usable start screen within 1.5 seconds of a cold start, and at most 250 MB memory in total while viewing a one-hour session, measured on the reference machines in the design. The Linux AppImage SHALL be at most 100 MB, because it carries its own copy of the web engine and GTK libraries that the other installers take from the system.

#### Scenario: Cold start
- **WHEN** the user starts the app on a reference machine after a reboot
- **THEN** the start screen is usable within 1.5 seconds

#### Scenario: Installer size
- **WHEN** a release is built for any platform
- **THEN** each installer is at most 25 MB, except the AppImage, which is at most 100 MB

#### Scenario: AppImage carries its own web engine
- **WHEN** the Linux release is built
- **THEN** the `.deb` is at most 25 MB and the AppImage, which bundles WebKitGTK, at most 100 MB
