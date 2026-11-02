# Program status recordings

Raw pty output of real agents, recorded on macOS on 2026-11-02 at 120×40 with a terminal that answers the usual queries (device attributes, background colour, kitty keyboard). To keep nothing private, every printed character is replaced with `.`, and window titles (OSC 0/1/2) and hyperlinks (OSC 8) are removed. All other escape sequences are kept in their original order and position.

| File | Agent | OSC 7501 query answered | What happens |
|---|---|---|---|
| `claude-code-question.bin` | Claude Code 2.1.309 | yes | trust prompt, a prompt that runs `touch b.txt` and asks an `AskUserQuestion`, the answer, `/exit` |
| `claude-code-permission.bin` | Claude Code 2.1.309 (`--permission-mode default`) | yes | a prompt that runs `rm c.txt`, the permission prompt allowed, `/exit` |
| `claude-code-unanswered.bin` | Claude Code 2.1.309 | no | a one-word prompt, `/exit` |
| `omp.bin` | omp 18.15.0 | not asked | a prompt that runs `ls`, Ctrl+C twice, Ctrl+D |
| `codex.bin` | Codex CLI 0.166.0 | not asked | a prompt that runs `ls` (no permission asked), Ctrl+C twice |
| `pi.bin` | pi 1.4.0 | yes | a prompt that runs `ls` (no permission asked), Ctrl+D |
| `grok.bin` | Grok Build 1.0.70 | yes | the folder trust question answered with `y`, a prompt that runs `ls` (no permission asked), Ctrl+Q |

What they show:

- Claude Code sends `OSC 7501 ; ?` followed by `CSI c`, and reports only when the query is answered first. Unanswered, it puts its spinner in the window title instead (removed here) and sends no reports.
- Root-record states, in order:
  - `claude-code-question.bin`: idle, working, blocked (`kind=question`, `msg` "answer: Do you prefer tea or coffee? (Tea · Coffee)"), working, done, working (`/exit`), clear
  - `claude-code-permission.bin`: idle, working, blocked (`kind=permission`, `msg` "approve Bash: rm c.txt"), working, done, working (`/exit`), clear
  - `omp.bin`: idle, idle, working, working, done, clear
- Both agents send `state=clear` when they exit.
- omp writes OSC 133 prompt marks (`A`, `B`, `C`, `D`) for its own commands while it runs, between its working reports. A prompt mark therefore does not mean that omp has exited.
- Claude Code also sends OSC 9;4 progress and an OSC 777 notification ("Claude needs your permission"), including for a question.

Codex, pi and Grok were recorded on 2026-11-02 for `more-coding-agents`, the same way, and the working directory (OSC 7) is also removed from them:

- `codex.bin`: idle, working (starting up), idle, working, done, idle, clear. Codex reports without being asked, and is idle again once a turn is done.
- `pi.bin`: pi asks `OSC 7501 ; ?` and then reports idle, working, done, clear.
- `grok.bin`: Grok asks `OSC 7501 ; ?`, reports its folder trust question as blocked (`kind=permission`, "Trust this folder?"), then idle, working four times with a `msg` ("Waiting for response…", "Thinking"), and done ("Turn complete in 10s."). Its `app` is `grok-build`. It sent no `clear` in the 4 seconds after Ctrl+Q, which first closed a data-sharing notice; the recording was stopped there.
