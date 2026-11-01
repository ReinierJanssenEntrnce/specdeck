---
title: Install
description: Install SpecDeck on macOS, Windows or Linux.
---

Everyone in a session needs SpecDeck: the host and every viewer. Download it from the [latest release](https://github.com/nickhartjes/specdeck/releases/latest), or use a package manager.

| Platform | How |
|---|---|
| macOS 11 or later (Apple silicon) | `brew install --cask nickhartjes/tap/specdeck`, or `SpecDeck-arm64.dmg` |
| macOS 11 or later (Intel) | `brew install --cask nickhartjes/tap/specdeck`, or `SpecDeck-x64.dmg` |
| Windows | `SpecDeck-x64-setup.exe` |
| Linux | `SpecDeck-x86_64.AppImage`, `SpecDeck-amd64.deb`, or [Nix](#nix) |

The release files keep the same names from version to version, so a link such as `https://github.com/nickhartjes/specdeck/releases/latest/download/SpecDeck-arm64.dmg` always gets the newest one.

## The first launch

Releases are not signed yet, so your system warns the first time you open SpecDeck:

- **macOS**: allow SpecDeck once under **System Settings › Privacy & Security** (**Open Anyway**), or run `xattr -dr com.apple.quarantine /Applications/SpecDeck.app`. The Homebrew cask says the same when it installs.
- **Windows**: choose **More info › Run anyway** in the SmartScreen prompt.

On macOS, the first time you host or join, macOS also asks whether SpecDeck may *find and connect to devices on your local network*. Select **Allow**: SpecDeck connects directly to teammates on the same network when it can.

Installing also registers `specdeck://` links, so clicking a join link opens SpecDeck.

## Nix

The flake builds SpecDeck from source, for x86_64 and aarch64 Linux:

```sh
nix run github:nickhartjes/specdeck
```

To add it to a NixOS or Home Manager configuration, use the flake's `packages.<system>.specdeck` or `overlays.default`.

The Nix package uses the public sharing service, `wss://specdeck.hartj.es/signal`, like the release builds. To use your team's own service, override it:

```nix
specdeck.override { signallingUrl = "wss://specdeck.example.com/signal"; }
```

Or set the service in [Settings](/specdeck/guide/settings/), which works for every install.

## Update

- **Homebrew**: `brew upgrade --cask specdeck`.
- **Installers**: download the new release and install it over the old one. Your settings stay.
- **Nix**: update the flake input.

When the host's and a viewer's versions cannot work together, SpecDeck says **Update SpecDeck to join** or **The host needs to update**.
