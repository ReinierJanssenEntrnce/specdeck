---
title: Join a session
description: Join with an invite code or a link, and choose how you follow sessions.
---

## Join

- **With a link:** click the `specdeck://join/…` link the host shared. SpecDeck opens and joins.
- **With a code:** under **Join a session** (or **File › Join a Session…**, **⌘J** on macOS, **Ctrl+Shift+J** on Windows and Linux), type or paste the invite code or link and select **Join**. Upper or lower case and dashes do not matter, and the code is formatted as you type.

If you are watching another session, SpecDeck asks before it leaves that one. If a link does not work, SpecDeck says so on the start screen; see [Troubleshooting](/specdeck/guide/troubleshooting/) for what each message means.

## Through another sharing service

A join link also names the host's sharing service. If it is not the one you use, for example because the host's team [runs its own](/specdeck/self-host/overview/), SpecDeck asks **Join through …?** before it connects:

- **Join** uses that service for this session only.
- Tick **Use this service from now on** to make it your sharing service in [Settings](/specdeck/guide/settings/).
- **Cancel** connects nowhere.

The other service cannot read the session, but it sees your network address, so join only through links you trust.

A code typed on its own always uses your own sharing service. When host and viewer use different services, share the link.

:::note[SpecDeck 1.0]
SpecDeck 1.0 cannot open links from newer versions by clicking them. On 1.0, paste the link into **Join a session** instead, or update SpecDeck.
:::

## Overview or Developer

The first time you join, SpecDeck asks how you want to follow sessions:

- **Overview:** the conversation and the plan. You see what was asked, what the agent did in plain words, its answers, and the documents it wrote. The terminal is hidden.
- **Developer:** the host's terminal, read-only, next to every tool call, command and diff.

SpecDeck remembers your answer for every session you join, and you can change it at any time under **View** (see [Follow a session](/specdeck/guide/follow/)). **Leave** takes you back to the start screen.

## When the connection drops

If your connection drops, SpecDeck reconnects on its own and catches up; you do not miss anything. When the host ends the session, you can keep reading what was shared until you leave.
