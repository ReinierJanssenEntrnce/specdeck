---
title: Host a session
description: Share a coding agent or your terminal with your team.
---

## Start a session

1. Under **Host a session** (or **File › Host a Session…**, **⌘N** on macOS, **Ctrl+Shift+N** on Windows and Linux), pick the **agent**, or **Terminal** to share your shell without an agent. Agents that are not installed are shown as *Not installed*. SpecDeck remembers what you chose last.
2. Select one of your **recent folders**, or **Choose a folder and start…** and pick the project folder the agent or shell should start in. Recent folders list the last five folders you hosted in, newest first; they are also under **File › Open Recent**. Select **×** next to one to remove it from the list (the folder itself is not touched).
3. The agent or your shell starts in the terminal. Use it as you normally would. In a terminal session, typing `exit` ends the shell; the session stays open for viewers to read until you end it.
   - The first time in a new folder, Claude Code asks whether you trust it. Answer it in the terminal.
4. Share the **invite code** at the top, for example `7K2M-9QXH-4TPA`. **Copy** copies the code; the link button copies a `specdeck://join/…` link, which opens SpecDeck and joins when a teammate clicks it.

:::caution[Terminal sessions show everything]
With **Terminal**, viewers see all output, including anything secret a command prints, such as tokens in environment variables or `.env` files. Passwords you type at a prompt are not shown, because the shell does not echo them.
:::

The link also names your sharing service, so it works for viewers whose SpecDeck uses another one. When you and your viewers use [different sharing services](/specdeck/guide/join/#through-another-sharing-service), share the link rather than the bare code.

You host one session at a time. To host or join another, end this one first.

## While you host

| To | Do this |
|---|---|
| See who is watching | Open **People** in the side panel |
| Stop people from joining with the current code | Select the **new code** button next to the invite code. People already watching stay; share the new code with anyone still to join |
| Remove someone | In **People**, select **Remove** next to their name and confirm. They are disconnected immediately, and the invite code changes so they cannot rejoin with it |
| Limit the number of viewers | In **People**, set **Max viewers** (1 to 20). Lowering it never disconnects anyone already watching |
| End the session for everyone | Select **End session** and confirm. The agent or shell stops and the invite code stops working |

The status next to the folder name shows what the agent is doing: **working**, **waiting for input**, or **exited**. For OpenCode and omp it shows **Live**, because their activity is not shared; a terminal session also shows **Live**, and **Shell exited** once you exit the shell. When Claude Code asks your permission, for example to run a command, the status shows **waiting** and the timeline shows the question, so viewers see why it paused.

If you set up your own Claude Code status line, it keeps showing in the terminal. SpecDeck reads the same figures from it for the bar at the bottom of the window, which everyone in the session sees, including your 5-hour and 7-day limits.

## Closing the window

**Closing the window does not end the session.** The agent keeps running and viewers stay connected. While you host, SpecDeck shows an icon in the menu bar (macOS) or the system tray (Windows, Linux). Its menu shows what the agent is doing and how many people watch, and offers:

| Menu item | What it does |
|---|---|
| **Open Window** | Brings the session back, with everything that happened while the window was closed. On macOS, clicking SpecDeck in the Dock does the same |
| **Copy Invite Code** | Copies the code, also while the window is closed |
| **End Session…** | Ends the session for everyone, after asking |

Quitting SpecDeck (**⌘Q** on macOS, **File › Quit** elsewhere) while you host asks first, because it ends the session for everyone.

## When sharing is unavailable

If SpecDeck's sharing service cannot be reached, the session still runs on your computer. A banner says that sharing is unavailable, and the invite code appears as soon as the service is back.
