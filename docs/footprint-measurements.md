# Footprint measurements

SpecDeck aims for a usable start screen within 1.5 seconds of a cold start, and at most 250 MB of memory in total while viewing a one-hour session (`native-experience`, "Resource footprint"). The reference machines are an Apple M1 with 8 GB (macOS 15), an x64 laptop with 8 GB (Windows 11), and the same laptop with Ubuntu 24.04.

## How to measure

Build the installers with `just package`, quit SpecDeck, then:

```sh
scripts/measure-footprint start 5      # right after a reboot: the first start is the cold one
scripts/measure-footprint memory 60    # join a one-hour session in the window that opens, then press Enter
```

`start` times each start until the start screen reports that it can be used. `memory` samples every 10 seconds and reports the peak: the app's process plus its web engine's (on macOS the WebKit processes, counted as physical footprint as Activity Monitor shows it; on Linux the process tree, as RSS). For a fair figure, view a real Claude Code session of about an hour, with the timeline.

## Results

| Machine | Start (first / median of 5) | Memory, one hour of viewing (peak) | Notes |
|---|---|---|---|
| Apple M5 Pro, 48 GB, macOS 26.7 (not a reference machine) | 1.08 s / 0.95 s | 205 MB | 2026-11-01. Not after a reboot. Memory while viewing a terminal session that printed about 4 MB over the hour, with the terminal shown: 114 MB at the start, about 200 MB after 35 minutes, flat after that |

Reference machines: not measured yet.
