# Proposal

## Why

`native-shared-session-mvp` built SpecDeck 1.x and is archived (`openspec/changes/archive/2026-10-31-native-shared-session-mvp/`), but 11 of its tasks could not be finished yet. Some need the repository on GitHub (CI, release, installers, the Nix hash). Others are measurements and manual checks only the host can run. This change keeps them tracked until 1.0 is released.

## What Changes

- Get CI green on GitHub and verify the release workflow, the installers per platform and the Nix flake (its pnpm dependency hash can only be computed where registry.npmjs.org is reachable).
- Measure replay time and resource footprint against the targets in the archived design.
- Run the manual checks: reopening the window with a viewer connected, the user guide against the app, keyboard and screen-reader use, and the cross-platform checklist.
- Deploy signalling and coturn to a test namespace once the cluster operators confirm the relay port mapping.
- Allow the Linux AppImage up to 100 MB. The first CI build produced an 85 MB AppImage: unlike the other installers it carries its own WebKitGTK and GTK libraries, so the 25 MB installer limit cannot hold for it. The other installers keep the 25 MB limit (the `.deb` is 11 MB).
- Otherwise no behavior changes: the requirements are in the main specs synced from the archived change. A check that fails is fixed against those specs, or raised as its own change if a requirement has to change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-experience`: the Resource footprint requirement allows the AppImage up to 100 MB; every other installer stays at 25 MB.

## Impact

- GitHub repository `nickhartjes/specdeck`: workflows `ci.yml`, `release.yml`, `homebrew.yml`, and secrets for signing, the signalling URL and the Homebrew tap key.
- `nix/package.nix` (pnpm dependency hash), `docs/user-guide.md`, new `docs/replay-measurements.md`, `docs/verification.md` and `scripts/measure-footprint`.
- The cluster deployment in `deploy/k8s`.
