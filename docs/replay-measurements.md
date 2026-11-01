# Replay measurements

Late join, reconnect and reattaching the host window all replay the session's output log from the start (design decision 4). This page records what that costs for real sessions, to decide whether checkpoints are needed. Target: a viewer joining a one-hour session is caught up within about 5 seconds on broadband.

## How to measure

1. Record a session of about an hour with real work, without resizing the terminal window:

   ```sh
   mkdir -p ~/specdeck-recordings
   script -q ~/specdeck-recordings/claude-1.raw claude
   ```

   Use `opencode` or `omp` instead of `claude` for those agents.

2. Measure it:

   ```sh
   node scripts/measure-replay.mjs ~/specdeck-recordings/claude-1.raw 120 40
   ```

   Pass the terminal's columns and rows (`tput cols`, `tput lines`).

The script reports the raw and zstd-compressed size, how long `@xterm/headless` takes to process the whole log, and an estimate of the catch-up time: transfer at 20 Mbit/s plus replay. The UI's renderer adds work on top of the headless parse, so treat the replay figure as a lower bound.

## Results

| Recording | Agent | Duration | Raw | zstd | xterm replay | Catch-up estimate |
|---|---|---|---|---|---|---|
| synthetic (20,000 redraws) | none | n/a | 1.88 MB | 0.06 MB | 70 ms | 0.1 s |
| OpenSpec propose, revise, apply and archive of a small docs change (2026-11-01) | Claude Code | about 40 min | 0.76 MB | 0.07 MB | 40 ms at 54x63 | 0.1 s |

The second row is SpecDeck's own output log of the session, measured with the same script: the log holds the terminal output as recorded, with a few 4-byte resize records in between. One-hour sessions: not recorded yet.

## Conclusion

Pending the real recordings.
