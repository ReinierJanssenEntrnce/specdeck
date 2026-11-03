# Design

## Context

See proposal.md - Why. Current state:

- `release.yml` builds the NSIS installer on `windows-latest` and uploads it to a draft release as `SpecDeck-x64-setup.exe` (with `--clobber`), plus `SpecDeck-x64-setup.exe.sig` for the updater. `publish` then publishes the draft, and `homebrew` (via `homebrew.yml`, final releases only) rewrites the cask from the published `.dmg` files.
- `tauri.conf.json` sets no `bundle.windows` and no `bundle.publisher`. The NSIS installer therefore installs per user (Tauri's default `installMode` is `currentUser`, no UAC), and the publisher Windows shows is derived from the identifier `com.entrnce.specdeck`.
- The Tauri updater (`auto-update`) downloads and runs the same NSIS installer for in-app updates.
- No Windows code-signing certificate is set (`WINDOWS_CERTIFICATE` is absent), so the installer is unsigned.

## Goals / Non-Goals

**Goals:**
- A winget package that installs the same NSIS installer the release publishes, with no installer built just for winget.
- Each final release reaches winget without anyone editing a manifest by hand.
- One publisher name, **SpecDeck**, wherever Windows or winget shows one.

**Non-Goals:**
- Code signing, Microsoft Store/MSIX, Scoop or Chocolatey.
- Changing the app identifier or where settings live: that is `rename-app-identifier`.
- Getting past application control (WDAC, AppLocker): winget does not help there, and only signing plus an allowlist would.

## Decisions

### 1. Community repository, `SpecDeck.SpecDeck`

The package goes into `microsoft/winget-pkgs` (the default `winget` source) as `SpecDeck.SpecDeck`, publisher `SpecDeck`, moniker `specdeck`. A private REST source would need hosting and every user would have to add it, which a locked laptop may not allow. The identifier is permanent once merged, so it is chosen now and not changed.

### 2. `bundle.publisher: "SpecDeck"`

Set in `tauri.conf.json` so the installer writes **SpecDeck** as the publisher in the uninstall entry. winget matches installed apps to packages using that entry (display name, publisher, product code), so the manifest's `Publisher` and the installer's must agree, or `winget upgrade` may not recognise an installed copy. The identifier becomes `nl.nickhartjes.specdeck` in `rename-app-identifier`, which moves the settings over. That change lands before or in the same release as the first winget submission (task 3.2), so the uninstall entry winget first records already has the new identity and publisher.

Tauri's NSIS template also uses the publisher in a registry path under `HKCU\Software\<publisher>\<product>` that it reads to find an earlier install. The default install folder does not depend on the publisher, so an upgrade still lands in the same place, but task 2.2 checks it on a real 1.3.0 install: one entry in Installed apps, same folder, settings kept.

### 3. Manifest shape

One installer entry, `InstallerType: nullsoft` (winget then knows the silent switch), `Scope: user`, `Architecture: x64`, the versioned URL `https://github.com/nickhartjes/specdeck/releases/download/vX.Y.Z/SpecDeck-x64-setup.exe`, `Protocols: [specdeck]`, `UpgradeBehavior: install`. The stable asset name means only the version and hash change per release. The locale manifest reuses `shortDescription` and `longDescription` from `tauri.conf.json`, license MIT, and links to the repository and docs.

### 4. Updates through a reusable `winget.yml`, called after `publish`

It mirrors `homebrew.yml`: `workflow_call` with `tag`, plus `workflow_dispatch` to resubmit an existing tag; `release.yml` calls it after `publish` with the same "no `-` in the tag" condition. It runs on `ubuntu-latest` with Komac (`komac update SpecDeck.SpecDeck --version X.Y.Z --urls <url> --submit`), a release binary pinned to a version. Komac downloads the published installer, computes its SHA256, carries the rest of the manifest over from the previous version, and opens the pull request from the owner's fork of `winget-pkgs`.

Alternatives: `wingetcreate update` needs a Windows runner and does the same thing; the `winget-releaser` action wraps Komac and would bring another third-party action into a workflow that holds a write token. Calling Komac directly keeps the token in one step.

Komac hashing the published asset, not the build output, is deliberate: the manifest has to match what users download, also when a Windows job was re-run and `--clobber` replaced the file before publishing.

### 5. A token, skipped while missing

`WINGET_TOKEN` is a classic GitHub token with `public_repo` scope (Komac needs it to push to the fork and open the pull request in `microsoft/winget-pkgs`). When it is missing, the job logs a `::warning::` and succeeds, like `HOMEBREW_TAP_DEPLOY_KEY`. A failed submission does not fail the release: the installers are already published, and the job can be re-run by hand.

### 6. First version by hand

Komac's `update` needs the package to exist. The first manifest, for the release current when this lands, is created with `komac new` (or `wingetcreate new`), checked against decision 3, tested with `winget validate`, and submitted by hand. A moderator reviews it; later updates to an existing package usually merge after the automated validation alone.

### 7. Documentation

`install.md`: the Windows row becomes `winget install SpecDeck.SpecDeck`, or `SpecDeck-x64-setup.exe`. "The first launch" says that winget installs without a SmartScreen prompt, and that a downloaded installer gets a SmartScreen warning which, on computers whose organisation does not allow bypassing it, cannot be passed: use winget there. "Update" adds that the in-app updater also works for winget installs and that `winget upgrade` works too. `README.md` gets the same two points. The docs do not suggest unblocking the file or other ways around the policy.

## Risks / Trade-offs

- [Validation or moderators reject an unsigned installer, or Defender flags it in the validation sandbox] → Unsigned installers are accepted in `winget-pkgs`; if the first submission is held, address the reviewer's comments before relying on the automated job.
- [The company network or a later policy blocks winget or its community source] → Task 1.1 checks the source on the target laptop before anything is built; a policy change later leaves the GitHub download, and signing (task 1.3 of `release-and-verification`) as the lasting fix.
- [Application control blocks the unsigned installer even without Mark-of-the-Web] → Task 1.2 runs the installer without Mark-of-the-Web on the target laptop first; if it is blocked, this change stops and signing comes first.
- [Changing the publisher splits the uninstall entry or loses the install folder on upgrade from 1.3.0] → Task 2.2 checks it before release.
- [The in-app updater and `winget upgrade` both update] → Both install the same NSIS installer over the existing install, and winget reads the installed version from the uninstall entry, so whichever runs first wins and the other sees an up-to-date copy.
- [A winget release lags the GitHub release by hours to a day while validation runs] → Accepted: installed apps update through the in-app updater anyway.
- [`WINGET_TOKEN` is a broad classic token] → Only the winget job's Komac step receives it.

## Migration Plan

1. Land the publisher change and `rename-app-identifier` in a release, then submit that release's manifest by hand (decision 6).
2. Add `WINGET_TOKEN` and the fork; later final releases are submitted by `winget.yml`.
3. Rollback: removing the job stops new submissions; the package stays at its last version in winget. Removing the package from `winget-pkgs` takes a pull request there.
