# Proposal

## Why

The Windows installer could not be installed on a company-managed Windows 11 laptop: SmartScreen blocked `SpecDeck-x64-setup.exe` and, because the machine's policy prevents bypassing it, offered no **Run anyway**. The install page tells Windows users to choose **More info › Run anyway**, which is not there on such machines. Signing would not fix this soon either: a newly signed installer has no SmartScreen reputation yet and is blocked the same way. Installers that winget downloads carry no Mark-of-the-Web, so SmartScreen does not check them, and `winget --info` on that laptop shows no winget policies.

## What Changes

- SpecDeck is published in the community winget repository as `SpecDeck.SpecDeck`, publisher **SpecDeck**, so `winget install SpecDeck.SpecDeck` installs it and `winget upgrade` finds new releases.
- The release workflow submits each final release to winget after it is published; prereleases are not submitted. The job is skipped with a warning while its token is missing, like the Homebrew job.
- The Windows installer names **SpecDeck** as its publisher, so Windows' Installed apps and winget show the same name. Today Tauri derives the publisher from the app identifier, which shows "entrnce". The identifier itself is renamed to `nl.nickhartjes.specdeck` in the separate change `rename-app-identifier`, which must ship before or in the same release as the first winget submission, so winget never records the old identity.
- The install page and README give winget as the way to install on Windows, and stop promising that SmartScreen always offers **Run anyway**.
- Signing the Windows installer stays out of scope: it remains open under `release-and-verification` (task 1.3).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-experience`: the requirement "Install through Homebrew and Nix" becomes "Install through package managers" and adds winget on Windows, including following new releases without manual edits.

## Impact

- `.github/workflows/release.yml`: a winget job after `publish`, for final releases only.
- New `.github/workflows/winget.yml`, callable from `release.yml` and by hand for an existing tag, like `homebrew.yml`.
- New repository secret `WINGET_TOKEN` (a GitHub token that can open pull requests from a fork of `microsoft/winget-pkgs`); a fork of `microsoft/winget-pkgs` under the repository owner.
- `app/src-tauri/tauri.conf.json`: `bundle.publisher`.
- External: a one-time manual submission of the first manifest to `microsoft/winget-pkgs`, reviewed by its moderators.
- Documentation: `docs/site/src/content/docs/install.md` (table, first launch, update) and `README.md`.
- Overlaps `release-and-verification` task 1.4 (installers install and launch on a clean machine) for Windows.
