# SpecDeck user guide

SpecDeck lets one person (the **host**) run a coding agent, or just a terminal, on their computer while teammates (the **viewers**) follow along from anywhere. Viewers see the terminal and, for Claude Code, a timeline of every prompt, answer, tool call and file change. Each person scrolls and searches at their own pace.

Viewers are read-only. Only the host types into the agent, and the agent runs on the host's own login.

## Before you start

**Everyone** needs SpecDeck for macOS, Windows or Linux, and an internet connection. SpecDeck connects through your team's sharing service, so no VPN is needed. It uses UDP; on networks that block UDP entirely (some company networks), joining does not work and SpecDeck says so.

**The host** can share a coding agent, installed and logged in, or a plain terminal:

| Choice | What viewers see |
|---|---|
| Claude Code (`claude`) | The terminal, plus the timeline, file diffs and the list of touched files |
| OpenCode (`opencode`) | The terminal |
| omp (`omp`) | The terminal |
| Terminal | Your own shell: your login shell on macOS and Linux, PowerShell on Windows. Nothing to install |

Check an agent by running it in a terminal: it should start without asking you to log in. On **Windows**, Claude Code also needs **Git for Windows** (Git Bash).

**On macOS**, the first time you host or join, macOS asks whether SpecDeck may *find and connect to devices on your local network*. Select **Allow**: SpecDeck connects directly to teammates on the same network when it can. If you chose **Don't Allow**, turn SpecDeck on under **System Settings › Privacy & Security › Local Network**.

## Set your name

The first time you open SpecDeck, it asks what others should call you. To change it later, select **Change** next to your name on the start screen, or open **Settings** (see below).

Two people may use the same name; the second is shown as `Anna (2)`. Names are labels, not logins: anyone with the invite code can join under any name.

## Host a session

1. Under **Host a session** (or **File › Host a Session…**, **⌘N** on macOS, **Ctrl+Shift+N** on Windows and Linux), pick the **agent**, or **Terminal** to share your shell without an agent. Agents that are not installed are shown as *Not installed*. SpecDeck remembers what you chose last.
   - With **Terminal**, viewers see all output, including anything secret a command prints, such as tokens in environment variables or `.env` files. Passwords you type at a prompt are not shown, because the shell does not echo them.
2. Select one of your **recent folders**, or **Choose a folder and start…** and pick the project folder the agent or shell should start in. Recent folders list the last five folders you hosted in, newest first; they are also under **File › Open Recent**. Select **×** next to one to remove it from the list (the folder itself is not touched).
3. The agent or your shell starts in the terminal. Use it as you normally would. In a terminal session, typing `exit` ends the shell; the session stays open for viewers to read until you end it.
   - The first time in a new folder, Claude Code asks whether you trust it. Answer it in the terminal.
4. Share the **invite code** at the top, for example `7K2M-9QXH-4TPA`. **Copy** copies the code; the link button copies a `specdeck://join/…` link, which opens SpecDeck and joins when a teammate clicks it.

You host one session at a time. To host or join another, end this one first.

While you host:

| To | Do this |
|---|---|
| See who is watching | Open **People** in the side panel |
| Stop people from joining with the current code | Select the **new code** button next to the invite code. People already watching stay; share the new code with anyone still to join |
| Remove someone | In **People**, select **Remove** next to their name and confirm. They are disconnected immediately, and the invite code changes so they cannot rejoin with it |
| Limit the number of viewers | In **People**, set **Max viewers** (1 to 20). Lowering it never disconnects anyone already watching |
| End the session for everyone | Select **End session** and confirm. The agent or shell stops and the invite code stops working |

The status next to the folder name shows what the agent is doing: **working**, **waiting for input**, or **exited**. For OpenCode and omp it shows **Live**, because their activity is not shared; a terminal session also shows **Live**, and **Shell exited** once you exit the shell. When Claude Code asks your permission, for example to run a command, the status shows **waiting** and the timeline shows the question, so viewers see why it paused.

If you set up your own Claude Code status line, it keeps showing in the terminal. SpecDeck reads the same figures from it for the bar at the bottom of the window, which everyone in the session sees, including your 5-hour and 7-day limits.

**Closing the window does not end the session.** The agent keeps running and viewers stay connected. While you host, SpecDeck shows an icon in the menu bar (macOS) or the system tray (Windows, Linux). Its menu shows what the agent is doing and how many people watch, and offers:

| Menu item | What it does |
|---|---|
| **Open Window** | Brings the session back, with everything that happened while the window was closed. On macOS, clicking SpecDeck in the Dock does the same |
| **Copy Invite Code** | Copies the code, also while the window is closed |
| **End Session…** | Ends the session for everyone, after asking |

Quitting SpecDeck (**⌘Q** on macOS, **File › Quit** elsewhere) while you host asks first, because it ends the session for everyone.

If SpecDeck's sharing service cannot be reached, the session still runs on your computer. A banner says that sharing is unavailable, and the invite code appears as soon as the service is back.

## Join a session

- **With a link:** click the `specdeck://join/…` link the host shared. SpecDeck opens and joins.
- **With a code:** under **Join a session** (or **File › Join a Session…**, **⌘J** on macOS, **Ctrl+Shift+J** on Windows and Linux), type or paste the invite code or link and select **Join**. Upper or lower case and dashes do not matter, and the code is formatted as you type.

If you are watching another session, SpecDeck asks before it leaves that one. If a link does not work, SpecDeck says so on the start screen.

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
| Switch between Timeline, Files and People | Select the tab, or press **⌘1**, **⌘2**, **⌘3** (macOS) or **Ctrl+1**, **Ctrl+2**, **Ctrl+3** (Windows, Linux) |
| Copy | Select text in the terminal, the timeline or a diff and copy it: **⌘C** on macOS; on Windows and Linux **Ctrl+C** copies while text is selected (in your own terminal, without a selection it interrupts the agent, as in any terminal) and **Ctrl+Shift+C** always copies. Copying never sends anything to the session |
| Make the side panel wider or narrower | Drag the line between the terminal and the side panel, or focus it and use the left and right arrow keys. SpecDeck remembers the width |
| See what the agent works with and has used | Look at the bar at the bottom of the window: the folder, its git branch (● means uncommitted changes, ↑↓ commits ahead of and behind its upstream) and, for Claude Code, the model and effort, how full the context window is, the tokens used in the session, and the host's 5-hour and 7-day limits with the time until they reset. Hover over a figure for details. Each prompt in the **Timeline** shows the tokens it used. Most of the token total is context the agent reads again from its cache on every call |

What you scroll, search or open only changes your own view.

## Notifications

While SpecDeck is not the window in front, or its window is closed, it tells you what needs you:

| Who | When |
|---|---|
| Host | Someone joins or leaves |
| Host | Claude Code finishes a prompt, or asks your permission |
| Viewer | The host ends the session, or removes you |

Click a notification to bring the session to the front. Nothing is shown while you are looking at SpecDeck. To turn notifications off, open **Settings**.

## Settings

Open **Settings** with **SpecDeck › Settings…** or **⌘,** on macOS, or **File › Settings…** or **Ctrl+,** on Windows and Linux. Changes apply at once.

| Setting | What it does |
|---|---|
| Your name | The name others see in sessions you start or join from now on |
| Show notifications | Turns all of SpecDeck's notifications on or off |

## The window

- SpecDeck reopens where you left it, at the same size, unless that screen is no longer connected.
- It follows your computer's light or dark appearance, also when you switch while it is open. On macOS and Windows 11 the window lets your desktop show through, as the system's own apps do.
- Only one SpecDeck runs at a time. Opening it again, or clicking a join link, brings the running one to the front.

## Keyboard and screen readers

Everything in SpecDeck works with the keyboard alone, and every control has a name that screen readers (VoiceOver, Narrator, Orca) announce. They also read the terminal.

| To | Press |
|---|---|
| See all keyboard shortcuts | **⌘/** (macOS) or **F1** (Windows, Linux), or **Help › Keyboard Shortcuts** |
| Move between the toolbar, the terminal and the side panel | **F6**, or **Shift+F6** to go back. In your own terminal, Tab goes to the agent, so F6 is the way out |
| Move on from a viewer's terminal | **Tab**, as anywhere else |
| Switch side-panel tabs | The left and right arrow keys, once a tab has focus |

