# Tasks

## 1. What each agent offers

- [x] 1.1 Add `AgentId::reports_status()` (true for Claude Code and omp) in `crates/protocol/src/agent.rs`. Verify with a unit test listing the value for every agent.
- [x] 1.2 Add `install_page(agent)` in `crates/core/src/agent.rs` with the pages from design decision 1, and none for Terminal. Verify with a unit test, and open each address in a browser to check it shows that agent's install instructions; replace any that do not.

## 2. Versions

- [x] 2.1 Add a version reader in `crates/core/src/agent.rs`: run an executable with `--version`, the given environment, stdin closed and a time limit, kill it after the limit, and take the first version-like token of its output. Verify with unit tests for the parsing (`2.1.309 (Claude Code)` → `2.1.309`, `opencode v2.2.5` → `2.2.5`, `omp/18.15.0` → `18.15.0`, `1.0.0-beta.2` kept whole, no digits → none) and a test with a script that sleeps past the limit (none, and it returns soon after the limit).
- [x] 2.2 Add the `agent_versions` command in `app/src-tauri/src/lib.rs`: installed agents except Terminal, read in parallel, cached in `AppState` by executable path and modification time. Verify `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.

## 3. Agent info and install pages

- [x] 3.1 Extend `AgentInfo` in `lib.rs` with `path`, `reports_status` and `install_page` (design decision 1), and the UI's `AgentInfo` in `app/ui/src/api/types.ts` to match; update `demo.ts` with paths and pages for the demo agents. Verify `cargo test --workspace` and `pnpm typecheck` pass.
- [x] 3.2 Add `tauri-plugin-opener`, initialise it, and add the `open_install_page` command that opens only the agent's page from `install_page`, with no opener permission in `capabilities/`. Add `agentVersions` and `openInstallPage` to the bridge (`bridge.ts`, `tauri.ts`, `demo.ts`). Verify `just check` passes and, in `just dev` with an agent removed from `PATH`, that its install link opens the browser.

## 4. Picker

- [x] 4.1 Rework `AgentChoice` and `.agents` in `StartScreen.tsx` and `styles.css` into rows (design decision 4): icon, name, version when known, labels for what viewers get, the path (`~`-shortened, full path in the tooltip), Terminal's shell, and missing agents as install-page links on one "Not installed" line below the rows (design decision 4). Fetch `agentVersions` after the screen renders and fill versions in as they arrive. Verify with tests in `StartScreen.test.tsx`: labels and paths per agent, a version that appears after the screen is shown, no version when the bridge returns none, the install button calling `openInstallPage('omp')`, and the existing radio and preselection tests updated, with a missing agent no longer a radio. Verify `pnpm lint` and the a11y tests pass.
- [x] 4.2 Describe the picker in `docs/site/src/content/docs/guide/host.md` (step 1: what each choice shows, the install link) and add install links to the table in `first-steps.md`. Verify `just docs-build` succeeds.

## 5. Check

- [x] 5.1 In `just dev`, open the start screen with Claude Code, OpenCode and omp installed, and once with `PATH` set so omp is missing. Verify the picker shows at once, versions appear within a few seconds, labels match the `session-hosting` scenarios, the install link opens omp's page, and keyboard and VoiceOver reach every choice and the install link. Then run `just check` and verify it passes.
