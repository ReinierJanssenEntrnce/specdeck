## RENAMED Requirements

- FROM: `### Requirement: Install through Homebrew and Nix`
- TO: `### Requirement: Install through package managers`

## MODIFIED Requirements

### Requirement: Install through package managers
Each release SHALL be installable on macOS with Homebrew (`brew install --cask nickhartjes/tap/specdeck`), on Windows with winget (`winget install SpecDeck.SpecDeck`), and on Linux with Nix from the project's flake, which builds SpecDeck from source. All three SHALL follow new final releases without manual edits.

#### Scenario: Homebrew on macOS
- **WHEN** a user on an Apple silicon or Intel Mac runs `brew install --cask nickhartjes/tap/specdeck`
- **THEN** SpecDeck for their processor is installed in Applications, and `brew upgrade` installs later releases

#### Scenario: winget on Windows
- **WHEN** a user on Windows 10 or 11 runs `winget install SpecDeck.SpecDeck`
- **THEN** SpecDeck is installed for that user without administrator rights and without a SmartScreen prompt, it opens `specdeck://` join links, and `winget upgrade` installs later releases

#### Scenario: winget on a machine that blocks bypassing SmartScreen
- **WHEN** a user on a Windows machine whose policy prevents bypassing SmartScreen runs `winget install SpecDeck.SpecDeck`
- **THEN** SpecDeck installs and starts

#### Scenario: Publisher on Windows
- **WHEN** SpecDeck is installed on Windows, by winget or from the installer
- **THEN** Installed apps and `winget list` show **SpecDeck** as its publisher, and winget recognises the installed copy as `SpecDeck.SpecDeck`

#### Scenario: Nix on Linux
- **WHEN** a user on x86_64 or aarch64 Linux runs `nix run github:nickhartjes/specdeck`
- **THEN** Nix builds SpecDeck from source and starts it, and it opens `specdeck://` join links once installed in their profile

#### Scenario: New release
- **WHEN** a final release tag is pushed and its installers are published
- **THEN** the Homebrew cask points at the new version, a winget update for the new version is submitted, and the flake builds the tagged source

#### Scenario: Prerelease
- **WHEN** a prerelease tag such as `v1.4.0-rc.1` is published
- **THEN** neither the Homebrew cask nor winget is updated
