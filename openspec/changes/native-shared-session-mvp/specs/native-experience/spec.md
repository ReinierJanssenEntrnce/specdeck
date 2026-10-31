# Spec Delta

## Purpose

Makes SpecDeck behave as a native desktop app on macOS, Windows and Linux: platform menus and shortcuts, tray or menu-bar presence, notifications, appearance, dialogs, single instance and a small resource footprint.

## ADDED Requirements

### Requirement: Platform menus
The app SHALL provide a native menu following each platform's conventions: the application menu on macOS, and a window menu bar on Windows and Linux. The menu SHALL offer host, join, copy invite code, find, settings, close window and quit.

#### Scenario: macOS application menu
- **WHEN** the app runs on macOS
- **THEN** the menu bar shows a SpecDeck menu with About, Settings, Hide and Quit, plus File, Edit, View, Window and Help menus

#### Scenario: Windows or Linux menu
- **WHEN** the app runs on Windows or Linux
- **THEN** the window has a File, Edit, View and Help menu, and Settings and Quit are under File

#### Scenario: Recent folders in the menu
- **WHEN** the host has hosted in two folders before
- **THEN** File offers both under a recent-folders submenu, newest first

### Requirement: Platform keyboard shortcuts
Menu actions SHALL use the platform's modifier key (Cmd on macOS, Ctrl on Windows and Linux) and standard shortcuts where one exists. Shortcuts SHALL NOT be taken while the host's terminal has focus, if the terminal needs that key.

#### Scenario: Find
- **WHEN** a viewer presses Cmd+F on macOS or Ctrl+F on Windows or Linux
- **THEN** search opens

#### Scenario: Ctrl+C in the host terminal
- **WHEN** the host presses Ctrl+C on Windows or Linux while the terminal has focus and no text is selected
- **THEN** the interrupt reaches the agent instead of copying

#### Scenario: Settings
- **WHEN** the user presses Cmd+, on macOS
- **THEN** the settings window opens

### Requirement: Tray or menu-bar presence while hosting
While a session is hosted, the app SHALL show an icon in the macOS menu bar or the Windows and Linux system tray. Its menu SHALL show the session state and viewer count and offer open window, copy invite code and close session.

#### Scenario: Host closed the window
- **WHEN** the host closes the window of a running session
- **THEN** the tray or menu-bar icon remains, and choosing Open window brings the session back

#### Scenario: Viewer count
- **WHEN** a third viewer joins
- **THEN** the tray or menu-bar menu shows 3 viewers

#### Scenario: No session
- **WHEN** no session is hosted
- **THEN** no tray or menu-bar icon is shown

### Requirement: Notifications
The app SHALL send OS notifications for events that need attention while its window is not focused: to the host when a viewer joins or leaves and when the agent waits for the host's input (for agents whose events the app captures), and to a viewer when the session ends. The user SHALL be able to turn them off.

#### Scenario: Agent waits for the host
- **WHEN** Claude Code finishes a turn or asks for permission while the host's window is in the background
- **THEN** the host gets a notification, and clicking it brings the session window to the front

#### Scenario: Window focused
- **WHEN** a viewer joins while the host's window has focus
- **THEN** no notification is shown

#### Scenario: Notifications turned off
- **WHEN** the user turned notifications off in settings
- **THEN** the app shows no notifications

### Requirement: Follow the OS appearance
The app SHALL follow the OS light or dark appearance and switch live when it changes. Window chrome, dialogs, menus and system fonts SHALL be the platform's own.

#### Scenario: OS switches to dark mode
- **WHEN** the user switches the OS to dark mode while the app is open
- **THEN** the app switches to its dark appearance, including the terminal, without restarting

#### Scenario: Native dialogs
- **WHEN** the host chooses a folder or confirms closing a session
- **THEN** the app uses the platform's own folder picker and dialog

### Requirement: Window state
The app SHALL remember the main window's size, position and pane sizes and restore them on the next start, unless that position is no longer on any screen.

#### Scenario: Restart
- **WHEN** the user resizes and moves the window, quits and starts the app again
- **THEN** the window opens with the same size and position

#### Scenario: Screen removed
- **WHEN** the window was last on an external screen that is no longer connected
- **THEN** the window opens on the main screen

### Requirement: Single instance
Only one instance of the app SHALL run per user. Starting the app again, or opening a join link, SHALL bring the running instance to the front and pass on the request.

#### Scenario: Start the app twice
- **WHEN** the user starts the app while it is already running
- **THEN** the running instance comes to the front and no second instance stays open

### Requirement: Keyboard and screen reader access
Every control SHALL be reachable and usable with the keyboard alone, and SHALL have a label that the platform's screen reader announces.

#### Scenario: Join without a mouse
- **WHEN** a user tabs to the invite code field, types a code and presses Enter
- **THEN** the app joins the session

#### Scenario: Screen reader on a control
- **WHEN** a screen reader user moves focus to the copy invite code button
- **THEN** the screen reader announces what it does

### Requirement: Resource footprint
The app SHALL stay small and quick: an installer of at most 25 MB per platform, a usable start screen within 1.5 seconds of a cold start, and at most 250 MB memory in total while viewing a one-hour session, measured on the reference machines in the design.

#### Scenario: Cold start
- **WHEN** the user starts the app on a reference machine after a reboot
- **THEN** the start screen is usable within 1.5 seconds

#### Scenario: Installer size
- **WHEN** a release is built for any platform
- **THEN** each installer is at most 25 MB

### Requirement: Install through Homebrew and Nix
Each release SHALL be installable on macOS with Homebrew (`brew install --cask nickhartjes/tap/specdeck`) and on Linux with Nix from the project's flake, which builds SpecDeck from source. Both SHALL follow new releases without manual edits.

#### Scenario: Homebrew on macOS
- **WHEN** a user on an Apple silicon or Intel Mac runs `brew install --cask nickhartjes/tap/specdeck`
- **THEN** SpecDeck for their processor is installed in Applications, and `brew upgrade` installs later releases

#### Scenario: Nix on Linux
- **WHEN** a user on x86_64 or aarch64 Linux runs `nix run github:nickhartjes/specdeck`
- **THEN** Nix builds SpecDeck from source and starts it, and it opens `specdeck://` join links once installed in their profile

#### Scenario: New release
- **WHEN** a release tag is pushed and its installers are published
- **THEN** the Homebrew cask points at the new version, and the flake builds the tagged source

