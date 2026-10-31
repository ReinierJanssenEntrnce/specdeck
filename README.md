# SpecDeck

SpecDeck lets one person (the **host**) run a coding agent (Claude Code, OpenCode or omp) on their computer while teammates (the **viewers**) follow along from anywhere. Viewers see the terminal and, for Claude Code, a timeline of prompts, answers, tool calls and file diffs. They join with an invite code and are read-only: only the host types into the agent, on the host's own login.

This is SpecDeck 1.x, a native rewrite of the Electron app (0.x, in `../claude-share`) on Tauri 2 with a Rust session engine. 1.x and 0.x cannot join each other's sessions.

How to use the app: [docs/user-guide.md](docs/user-guide.md). Status: in development. What is being built is in [`openspec/changes/native-shared-session-mvp`](openspec/changes/native-shared-session-mvp) (proposal, specs, design and tasks).

## Install

| Platform | How |
|---|---|
| macOS (Apple silicon, Intel) | `brew install --cask nickhartjes/tap/specdeck`, or the `.dmg` from [Releases](https://github.com/nickhartjes/specdeck/releases) |
| Windows | `SpecDeck-x64-setup.exe` from [Releases](https://github.com/nickhartjes/specdeck/releases) |
| Linux | `nix run github:nickhartjes/specdeck` (builds from source, x86_64 and aarch64), or the AppImage or `.deb` from [Releases](https://github.com/nickhartjes/specdeck/releases) |

With Nix, add it to a system or home configuration through the flake's `packages.<system>.specdeck` or `overlays.default`. Builds from source use the sharing service set at build time; override it with `specdeck.override { signallingUrl = "wss://…/signal"; }`, or set `SPECDECK_SIGNALLING_URL` when starting the app.

Releases built before the signing certificates are in place are unsigned, so the system warns on first launch. On macOS, allow SpecDeck once under System Settings > Privacy & Security (Open Anyway), or run `xattr -dr com.apple.quarantine /Applications/SpecDeck.app`; the Homebrew cask says the same when it installs an unsigned release. On Windows, choose More info > Run anyway in the SmartScreen prompt.

SpecDeck 1.x replaces 0.x under the same names: `brew upgrade` and flake updates move 0.x installs to 1.x, which cannot join 0.x sessions, so a team switches together.

## Repository layout

| Path | Contents |
|---|---|
| [`crates/protocol`](crates/protocol) | Wire protocol shared by hosts and viewers: messages, frames, signing, invite codes (no I/O) |
| [`crates/core`](crates/core) | Session engine: runs the agent in a pty, output log, timeline, WebRTC connections |
| [`crates/signalling`](crates/signalling) | Matchmaking service and TURN credentials |
| [`app/src-tauri`](app/src-tauri) | Tauri shell: windows, menus, tray, commands |
| [`app/ui`](app/ui) | Web UI (React, TypeScript, Vite) |
| [`openspec`](openspec) | Specs and change proposals |
| [`deploy/k8s`](deploy/k8s) | Kubernetes manifests for signalling and coturn (`base`, `local`) ([README](deploy/k8s/README.md)) |
| [`docker`](docker) | Signalling service image |
| [`scripts`](scripts) | Helper scripts used by the `just` recipes and CI |

## Development

Tools are pinned in [`mise.toml`](mise.toml): Rust, Node, pnpm, just and cmake (needed to build libdatachannel). With [mise](https://mise.jdx.dev) installed, run `mise install`. On Linux, Tauri also needs the WebKitGTK development packages (see the CI workflow for the list).

```sh
just install   # install JavaScript dependencies
just dev       # start the desktop app in development mode
just check     # check formatting, lint, type-check and test (Rust and UI)
just fmt       # format the Rust code
just package   # build installers for this platform (target/release/bundle)
```

Run `just` to list all recipes. To try sharing on your own machine (Colima with Kubernetes, `colima start --kubernetes`):

```sh
just local-dev     # signalling and coturn on Colima's k3s, plus a host app and a viewer app
just local-up      # only deploy signalling and coturn to Colima's k3s
just local-test    # check that two peers connect through the relay only
just local-down    # remove the deployment
```

`just local-dev` opens two windows, **SpecDeck (host)** and **SpecDeck (viewer)**, each with its own settings (`SPECDECK_PROFILE`, debug builds only). Host a session in one and join with its invite code in the other. To look at the UI without a session, run `pnpm --filter @specdeck/ui dev` and open `http://localhost:5173/?demo=start`, `?demo=host` or `?demo=viewer` in a browser.

See [`deploy/k8s/README.md`](deploy/k8s/README.md) for ports, secrets and the differences from a real cluster.

npm packages come from the registry your npm configuration points at. If that is the company's CodeArtifact and installs fail with `401 Unauthorized`, refresh the login:

```sh
aws codeartifact login --tool npm --repository npm-store --domain entrnce --domain-owner 350124346922 --region eu-central-1 --profile entrnce
```

## CI

[`ci.yml`](.github/workflows/ci.yml) runs `just check` and `just package` on macOS, Windows and Ubuntu, and fails when an installer is larger than 25 MB ([`scripts/check-installer-size.sh`](scripts/check-installer-size.sh)).
It also builds the Nix flake.

## Releases

Push a tag that matches the version in `Cargo.toml` and `app/src-tauri/tauri.conf.json`, for example `v1.0.0`. [`release.yml`](.github/workflows/release.yml) builds the installers on each platform, uploads them to a GitHub release under stable names (`SpecDeck-arm64.dmg`, `SpecDeck-x64.dmg`, `SpecDeck-x64-setup.exe`, `SpecDeck-x86_64.AppImage`, `SpecDeck-amd64.deb`), publishes it, and has [`homebrew.yml`](.github/workflows/homebrew.yml) update the cask in `nickhartjes/homebrew-tap` ([`scripts/render-cask.sh`](scripts/render-cask.sh)). The Nix flake follows the repository, so it needs nothing per release. The secrets for the sharing service URL, signing and the tap are listed at the top of `release.yml`. Without the signing secrets the installers are unsigned (ad-hoc on macOS) and the cask explains how to open the app; without the tap deploy key the cask is not updated.

## License

[MIT](LICENSE)
