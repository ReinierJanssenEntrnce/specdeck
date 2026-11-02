# Proposal

## Why

The agent picker on the start screen shows a name and one short line per agent ("Timeline and diffs", "Terminal only", "Not installed"). A host cannot tell at a glance what their viewers will get with each choice, which version would start, or where it was found, and a missing agent gives no hint how to get it. The next change adds Codex, Grok and pi, which makes a bare list of names harder to choose from still.

## What Changes

- Each agent in the picker gets an icon, so the choices are told apart at a glance. Icons are generic (from the app's icon set), not the agents' brand logos.
- Each agent shows what viewers get with it: timeline, diffs and plans, and whether its status (working, waiting) is shown, or that viewers see the terminal only.
- An installed agent shows its version and where its executable was found; Terminal shows which shell it starts. The versions are read in the background, so they never delay the start screen; until one is known, or when it cannot be read, it is left out.
- Agents that are not installed are listed below the choices under "Not installed", each as a small link that opens its install page in the browser.
- The picker becomes a list of rows of installed agents instead of a grid of tiles, to fit the extra information and more agents later.
- Not in this change: new agents (Codex, Grok, pi; the next change), brand logos, checking whether an agent is logged in.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-hosting`: adds an "Agent details in the picker" requirement (icon, what viewers get, version and location, install link, never delaying the start screen).

## Impact

- `crates/protocol/src/agent.rs`: whether an agent reports its status (Claude Code and omp do, OpenCode does not), next to `has_timeline`.
- `crates/core/src/agent.rs`: reading an executable's version with a time limit, and each agent's install page.
- `app/src-tauri/src/lib.rs`: `AgentInfo` carries the executable's path, whether the agent reports its status and its install page; a new `agent_versions` command returns the versions, read once per executable and cached; a command to open an install page.
- `app/src-tauri/Cargo.toml`, `capabilities`: `tauri-plugin-opener`, to open install pages in the browser. A new dependency, from the Tauri project.
- `app/ui/src/screens/StartScreen.tsx`, `styles.css`, `api/` (types, bridge, tauri, demo): the new rows.
- `docs/site/src/content/docs/guide/host.md` and `first-steps.md`: what the picker shows.
- No protocol change between host and viewers.
