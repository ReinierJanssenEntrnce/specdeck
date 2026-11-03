# Tasks

## 1. Check the target laptop

- [ ] 1.1 On the company-managed laptop that blocked the installer, run `winget search Microsoft.PowerToys` and verify the `winget` source answers with a result
- [ ] 1.2 On the same laptop, download the 1.3.0 installer without Mark-of-the-Web (`Invoke-WebRequest https://github.com/nickhartjes/specdeck/releases/download/v1.3.0/SpecDeck-x64-setup.exe -OutFile $env:TEMP\SpecDeck-setup.exe`) and run it once, only as this check; verify it installs and starts with no SmartScreen or application-control block. If it is blocked, stop here: this change does not help that machine (design, Risks)

## 2. Publisher

- [ ] 2.1 Set `bundle.publisher` to `"SpecDeck"` in `app/src-tauri/tauri.conf.json` (design decision 2); verify with `just package` on Windows (or the CI Windows job) that the installer's uninstall entry shows Publisher **SpecDeck** in Installed apps
- [ ] 2.2 On a Windows machine with 1.3.0 installed, install a build with the new publisher over it; verify Installed apps lists one SpecDeck (publisher SpecDeck), it is in the same folder, settings are kept, and `specdeck://` links still open it

## 3. First winget version

- [ ] 3.1 Fork `microsoft/winget-pkgs` under the repository owner, and add the classic token with `public_repo` scope as repository secret `WINGET_TOKEN`; verify `gh secret list` shows it
- [ ] 3.2 After the first release that carries both the new publisher and the identifier from `rename-app-identifier` (check `jq .identifier app/src-tauri/tauri.conf.json` at that tag reads `nl.nickhartjes.specdeck`), create the manifest with `komac new` for `SpecDeck.SpecDeck` matching design decision 3 (nullsoft, user scope, x64, versioned URL, `specdeck` protocol, `UpgradeBehavior: install`, publisher SpecDeck, MIT, repository and docs links); verify `winget validate` passes on it
- [ ] 3.3 Submit the manifest to `microsoft/winget-pkgs` and follow the pull request until it merges; verify `winget show SpecDeck.SpecDeck` returns the version
- [ ] 3.4 On the company-managed laptop, uninstall SpecDeck, run `winget install SpecDeck.SpecDeck`; verify it installs without a SmartScreen prompt or administrator rights, starts, opens a `specdeck://` join link, and that `winget list SpecDeck.SpecDeck` shows publisher SpecDeck and the version

## 4. Release automation

- [ ] 4.1 Add `.github/workflows/winget.yml` (design decisions 4 and 5): `workflow_call` and `workflow_dispatch` with `tag`, `ubuntu-latest`, a pinned Komac release, `komac update SpecDeck.SpecDeck --version <tag without v> --urls <versioned installer URL> --submit`, skipped with a `::warning::` while `WINGET_TOKEN` is missing, with a header comment listing the secret as `homebrew.yml` does; verify with `actionlint` (or the CI lint) that it parses
- [ ] 4.2 Call it from `release.yml` after `publish` with the same prerelease condition as `homebrew`, and list `WINGET_TOKEN` in the secrets comment at the top of `release.yml`; verify the job graph by running a prerelease tag and seeing the winget job skipped
- [ ] 4.3 Run `winget.yml` by hand (`workflow_dispatch`) for the next final release, or let that release run it; verify the pull request in `microsoft/winget-pkgs` has the published installer's SHA256, passes validation, and that `winget upgrade SpecDeck.SpecDeck` installs it on a machine with the previous version

## 5. Documentation

- [ ] 5.1 Update `docs/site/src/content/docs/install.md` per design decision 7 (Windows row, first launch, update section); verify the docs site builds and the page reads correctly
- [ ] 5.2 Update `README.md`: the Windows row in the install table, the unsigned-release paragraph (winget avoids the SmartScreen prompt; managed machines may not allow Run anyway), and the release paragraph (winget job and `WINGET_TOKEN`); verify the commands in it match the docs

## Workflow follow-up

- In `release-and-verification`, note under task 1.4 that the Windows install was verified through winget on a managed machine (task 3.4 here); signing in task 1.3 stays open.
- Archive this change after the first automated winget update has merged.
