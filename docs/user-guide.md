# SpecDeck user guide

SpecDeck lets one person (the **host**) run a coding agent on their computer while teammates (the **viewers**) follow along from anywhere. Viewers see the agent's terminal and, for Claude Code, a timeline of every prompt, answer, tool call and file change. Each person scrolls and searches at their own pace.

Viewers are read-only. Only the host types into the agent, and the agent runs on the host's own login.

## Before you start

**Everyone** needs SpecDeck for macOS, Windows or Linux, and an internet connection. SpecDeck connects through your team's sharing service, so no VPN is needed. It uses UDP; on networks that block UDP entirely (some company networks), joining does not work and SpecDeck says so.

**The host** also needs at least one coding agent, installed and logged in:

| Agent | What viewers see |
|---|---|
| Claude Code (`claude`) | The terminal, plus the timeline, file diffs and the list of touched files |
| OpenCode (`opencode`) | The terminal |
| omp (`omp`) | The terminal |

Check an agent by running it in a terminal: it should start without asking you to log in. On **Windows**, Claude Code also needs **Git for Windows** (Git Bash).

## Set your name

The first time you open SpecDeck, it asks what others should call you. To change it later, select **Change** next to your name on the start screen.

Two people may use the same name; the second is shown as `Anna (2)`. Names are labels, not logins: anyone with the invite code can join under any name.

## Host a session

1. Under **Host a session**, pick the **agent**. Agents that are not installed are shown as *Not installed*. SpecDeck remembers the agent you used last.
2. Select one of your **recent folders**, or **Choose a folder and start…** and pick the project folder the agent should work in. Recent folders list the last five folders you hosted in, newest first; select **×** next to one to remove it from the list (the folder itself is not touched).
3. The agent starts in the terminal. Use it as you normally would.
   - The first time in a new folder, Claude Code asks whether you trust it. Answer it in the terminal.
4. Share the **invite code** at the top, for example `7K2M-9QXH-4TPA`. **Copy** copies the code; the link button copies a `specdeck://join/…` link.

While you host:

| To | Do this |
|---|---|
| See who is watching | Open **People** in the side panel |
| Stop people from joining with the current code | Select the **new code** button next to the invite code. People already watching stay; share the new code with anyone still to join |
| Remove someone | In **People**, select **Remove** next to their name and confirm. They are disconnected immediately, and the invite code changes so they cannot rejoin with it |
| Limit the number of viewers | In **People**, set **Max viewers** (1 to 20). Lowering it never disconnects anyone already watching |
| End the session for everyone | Select **End session** and confirm. The agent stops and the invite code stops working |

The status next to the folder name shows what the agent is doing: **working**, **waiting for input**, or **exited**. For OpenCode and omp it shows **Live**, because their activity is not shared.

**Closing the window does not end the session.** The agent keeps running and viewers stay connected. On macOS, click SpecDeck in the Dock to bring the window back; it shows the whole session again.

If SpecDeck's sharing service cannot be reached, the session still runs on your computer. A banner says that sharing is unavailable, and the invite code appears as soon as the service is back.

## Join a session

1. Under **Join a session**, paste the invite code or the `specdeck://` link the host shared. Upper or lower case and dashes do not matter, and the code is formatted as you type.
2. Select **Join**.

You see the host's terminal, read-only, and the side panel. **Leave** takes you back to the start screen.

If joining does not work, SpecDeck says why:

| Message | What it means |
|---|---|
| That code does not work | Check the code with the host. Codes stop working when the host ends the session or makes a new code |
| This session is full | The host reached their viewer limit |
| Too many attempts | Several wrong codes were tried from your network. Wait a minute |
| SpecDeck cannot reach its sharing service | Check your internet connection |
| Your network does not allow the connection | Your network blocks UDP. Try another network, such as your phone's hotspot |
| Update SpecDeck to join / The host needs to update | You and the host use versions that do not match |

If your connection drops, SpecDeck reconnects on its own and catches up; you do not miss anything. When the host ends the session, you can keep reading what was shared until you leave.

## Reading a session

| To | Do this |
|---|---|
| Read earlier output | Scroll up in the terminal. New output keeps arriving without moving your view; select **Jump to live** to follow it again |
| Find something | Press **⌘F** (macOS) or **Ctrl+F** (Windows, Linux) and type. **Enter** goes to the next match, **Shift+Enter** to the previous one. The search covers the terminal and the timeline. In the host's own terminal on Windows and Linux, use **Ctrl+Shift+F**, because Ctrl+F belongs to the terminal |
| See what happened per prompt | Open **Timeline**. Each prompt shows the agent's answer, its tool calls (succeeded, failed or did not run) and the diffs of files it changed |
| Go to a prompt | Use **Jump to** at the top of the timeline, or select a prompt: the terminal scrolls to where it happened |
| See which files changed | Open **Files**. Select a changed file to see all its diffs in order |
| Copy | Select text in the terminal, the timeline or a diff and copy it as usual. Copying never sends anything to the session |
| Make the side panel wider or narrower | Drag the line between the terminal and the side panel |
| See how much the agent has used | For Claude Code, the toolbar shows how full its context window is and the tokens used in the session; hover over it for the details. Each prompt in the **Timeline** shows the tokens it used. Most of the total is context the agent reads again from its cache on every call. OpenCode and omp do not report usage |

What you scroll, search or open only changes your own view.
