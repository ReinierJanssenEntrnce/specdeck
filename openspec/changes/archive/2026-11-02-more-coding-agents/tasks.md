# Tasks

## 1. Agents

- [x] 1.1 Add `Codex`, `Grok` and `Pi` to `AgentId` in `crates/protocol/src/agent.rs`, appended after `Terminal`, with `ALL` in the order of design decision 1, display names "Codex", "Grok" and "pi", and `reports_status()` true for Codex only. Update the `wire_names` test and add the new variants to the status and timeline tests. Verify `cargo test -p specdeck-protocol` passes.
- [x] 1.2 In `crates/core/src/agent.rs`, add the executables `codex`, `grok` and `pi` to `program`, and their install pages to `install_page`; no Git Bash check for pi (design decision 4). Extend `detects_installed_agents` with a fake `codex`, and the Git Bash test with the new agents. Verify `cargo test -p specdeck-core` passes.
- [x] 1.3 Add the new ids to `crates/core/examples/launch.rs` and `capture.rs`. Verify `cargo clippy --workspace --all-targets -- -D warnings` passes.

## 2. Protocol version

- [x] 2.1 Set `PROTOCOL_VERSION` to 2 in `crates/protocol/src/control.rs` and in `crates/protocol/README.md` update the "Current version" line (version 2 added Codex, Grok and pi to the agents) and the agents named in the `Welcome` row. Verify with the existing `check_version` tests and a test that a version 1 `Hello` is refused with `ViewerOutdated` by a version 2 host in `crates/core` (the sharing tests), and that `cargo test --workspace` passes.

## 3. App and docs

- [x] 3.1 Add the new ids to `AgentId` in `app/ui/src/api/types.ts`, to `AGENT_NAMES` in `HostScreen.tsx`, and their icons (design decision 1) to `AGENT_ICONS` in `StartScreen.tsx`; update the "No coding agent is installed" hint to name all six agents. Verify with a test in `StartScreen.test.tsx` that a profile with Codex, Grok and pi shows them before Terminal with their labels and install buttons, and that `pnpm lint`, `pnpm typecheck` and `pnpm test` pass.
- [x] 3.2 Name the new agents in `README.md`, the agent table in `docs/site/src/content/docs/guide/first-steps.md` (with install links), `host.md` and `follow.md` (status per agent, the Terminal row's example), and add a line to `troubleshooting.md` that the update message appears when host and viewer use versions from before and after this release. Verify `just docs-build` succeeds.

## 4. Check with the real agents

- [x] 4.1 Install Codex, Grok and pi (the user may skip any that cannot be installed, such as Grok without a subscription). Check each install page in a browser shows its install instructions, and that each agent's own docs agree with design decision 4 for pi on Windows; correct the table and decision 4 if not.
- [x] 4.2 For each installed agent, record its raw output in a session with the status query answered, as in `crates/core/tests/fixtures/program-status/README.md` (a prompt with a tool call, a permission or question if the agent has one, and quitting), with printed text replaced. Add the recordings and a row per agent to the README; set `reports_status()` from what the recordings show and add a parsing test per recording like the existing ones. Verify `cargo test --workspace` passes.
- [x] 4.3 In `just local-dev`, host each installed new agent and join as viewer. Verify the picker shows its version, path and labels, the session starts in the chosen folder, the viewer sees the agent's name and terminal, and the status changes between working and waiting for agents that report it. Then run `just check` and verify it passes.
