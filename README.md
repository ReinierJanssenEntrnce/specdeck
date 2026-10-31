# SpecDeck

SpecDeck lets one person (the **host**) run a coding agent (Claude Code, OpenCode or omp) on their computer while teammates (the **viewers**) follow along from anywhere. Viewers see the terminal and, for Claude Code, a timeline of prompts, answers, tool calls and file diffs. They join with an invite code and are read-only: only the host types into the agent, on the host's own login.

This is SpecDeck 1.x, a native rewrite of the Electron app (0.x, in `../claude-share`) on Tauri 2 with a Rust session engine. 1.x and 0.x cannot join each other's sessions.

Status: in development. What is being built is in [`openspec/changes/native-shared-session-mvp`](openspec/changes/native-shared-session-mvp) (proposal, specs, design and tasks).

## Repository layout

| Path | Contents |
|---|---|
| [`crates/protocol`](crates/protocol) | Wire protocol shared by hosts and viewers: messages, frames, signing, invite codes (no I/O) |
| [`crates/core`](crates/core) | Session engine: runs the agent in a pty, output log, timeline, WebRTC connections |
| [`crates/signalling`](crates/signalling) | Matchmaking service and TURN credentials |
| [`app/src-tauri`](app/src-tauri) | Tauri shell: windows, menus, tray, commands |
| [`app/ui`](app/ui) | Web UI (React, TypeScript, Vite) |
| [`openspec`](openspec) | Specs and change proposals |
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

Run `just` to list all recipes.

npm packages come from the registry your npm configuration points at. If that is the company's CodeArtifact and installs fail with `401 Unauthorized`, refresh the login:

```sh
aws codeartifact login --tool npm --repository npm-store --domain entrnce --domain-owner 350124346922 --region eu-central-1 --profile entrnce
```

## CI

[`ci.yml`](.github/workflows/ci.yml) runs `just check` and `just package` on macOS, Windows and Ubuntu, and fails when an installer is larger than 25 MB ([`scripts/check-installer-size.sh`](scripts/check-installer-size.sh)).

## License

[MIT](LICENSE)
