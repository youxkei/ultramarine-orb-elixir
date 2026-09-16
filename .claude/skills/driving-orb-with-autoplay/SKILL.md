---
name: driving-orb-with-autoplay
description: How to run a game orb supports without a person at the keyboard and watch it — which directory to drive, how a launch is started and waited on, what to say every 30 seconds, when to close a run, and where a picture and a kept file go. Use whenever a task in ultramarine_orb_elixir needs the real game run — confirming a fix on the real thing, recording a resume file, holding two launches against each other, or measuring anything that takes a run to reach.
---

# Driving orb with autoplay

`--autoplay=<recipe>` makes orb play a run to a point the recipe names. The recipe grammar is the doc of
`orb_config::recipe` — every step, every `until` condition, and what each is for — and this skill is how a
launch of one is driven and watched.

## The directory

- Drive a **copy of the install**, with the game's own files and none of its `replay*/` directories. A
  played game writes its records beside itself, and the copy is where they land.
- Put the build under test into the copy before each launch: `cargo xtask build --release`, then copy
  `target/i686-pc-windows-gnu/release/orb.exe` into it. The copy's `orb.yaml` stays as it is.
- Read the commit off the launch's first log line — `orb <version> attached to pid N, built from <commit>`
  — and hold it against `jj log -r @ -T commit_id.short(12)` before reading anything else off the run.

## The launch

- Play the cases in `crates/orb-real/recipes/<game>/` with `cargo xtask real <copy> [<case>...]`, which
  builds, copies `orb.exe`, launches, waits and checks each trace. Run a case against the laid-out game
  first, in `orb-e2e`'s `the_cases_the_real_game_is_tested_with`.
- Keep a recipe written for one measurement **beside the copy's exe**, and launch it with
  `bash .claude/skills/driving-orb-with-autoplay/scripts/drive-a-run.sh <copy> <game-exe> <recipe> --log=verbose --pace=8`
  in the background. It refuses while the game is running, and returns when the game has gone.
- Run one game at a time. Ask `scripts/game.sh gone <game-exe>` before a launch and `scripts/game.sh close
  <game-exe>` after a launch of your own that is over or has gone wrong. The exe to name is the game's
  (`game::KNOWN` in `orb-core`).
- Give every launch `--log=verbose`: the per-frame lines are the ones that say what orb did to the run.
- Give a launch of a whole run `--pace=8`, which is a Normal run in a few minutes of wall clock with the
  same frames played. `orb_config::args` says what `--pace` does.
- End a recipe where the run ends. A step after `play_until` waits at the logo screen.

## Watching

- Watch the copy's `orb.log` with a poll, and **say the state every 30 seconds** — every 2 seconds, with
  the state line every 15th tick, while a screen that stands for seconds is what is being watched.
- Poll between tool calls with short commands (`wc -c` on the log, the newest `run:` line). A monitor's
  events arrive between calls, so do other work while the run plays instead of blocking on the launch.
- Read the log with `tail -c 2000000` and `tail -n +"$from"`: `tail -f` sees nothing grow on a Windows
  drive through the interop, and a grep over the whole of a verbose log costs more than the interval.
- Cut the log to the launch first, since `orb.log` is one file across launches:

      from=$(grep -an "new run ----" "$log" | tail -1 | cut -d: -f1)

  and match a monitor's end condition inside that cut too.
- End a filter in `grep --line-buffered`, or trim inside `sed`'s substitution; a `cut` or `head` as the
  last stage buffers the lines the monitor was armed for.
- Arm the poll with the launch, and add a filter for the lines the measurement turns on beside it.

Say three things a tick, and carry the state line verbatim — a step number says which step a recipe is on
and the state line says whether a run is on at all:

```sh
recent=$(tail -c 2000000 "$log")
state=$(printf '%s\n' "$recent" | grep -aE "^\[[0-9]+ms\] f[0-9]+ scene=" | tail -1 \
  | sed -E 's/^\[[0-9]+ms\] //; s/ seed=[^ ]*//' | cut -c1-120)
step=$(printf '%s\n' "$recent" | grep -a "autoplay: step" | tail -1 | sed -E 's/^.*autoplay: //' | cut -c1-48)
event=$(printf '%s\n' "$recent" | grep -aE \
  "run ended|ending (skipped|run out)|died in chapter|retry: |replay: (kept|declined)|front end: screen" \
  | tail -1 | sed -E 's/^\[[0-9]+ms\] //' | cut -c1-90)
echo "${step:-no step} | ${state:-no state yet} | ${event:-nothing yet}"
```

- the state: the newest `f<N> scene=A->B stage=S frames=F … deaths=D lives=L` line
- the step: the newest `autoplay: step N of M …`
- the event: the newest of `run ended after N retries`, `ending run out in N update(s)`,
  `ending skipped`, `died in chapter N`, `retry: …`, `replay: kept`, `front end: screen N`

Close the monitor of a run that is over (`TaskStop`) before starting the next launch.

## When to close a run

Read the newest state line and the log's size, and close the game — `scripts/game.sh close` — saying what
the log said before it, in these two states:

- **stalled**: `wc -c` on the log has not moved for several ticks
- **game over**: `lives=0` with a `play` step still waiting — the game's continue prompt is up and the
  log keeps growing

Leave it playing in these:

- the same chapter with `frames=` climbing — a boss's own attack
- `script=` standing still with `frames=` climbing — a boss attack holding the stage's script clock
- the same chapter with `died in chapter N` and `hit while` between — a run retrying it
- `autoplay: step N of M gave up` — the recipe's own `give_up_after` ends that step, and the launch goes on

## Pictures

- Ask for a `screenshot` step on a game that draws through D3D8 (th06, th07): `orb_core::shot` writes the
  back buffer at the end of the frame.
- Take a picture of th06c or th06nc from outside, from the monitor on the line that says the screen is up:
  `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .claude/skills/driving-orb-with-autoplay/scripts/shot.ps1 -Exe th06c.exe -Out <C:\path\to.png>`
  (`-Exe th06nc.exe` for the other),
  with the recipe holding that screen for longer than one poll interval.

## Across launches

- Copy the first launch's log out to the scratchpad (`cat "$log" > <scratchpad>/first.log`) before the
  second launch appends to or rotates `orb.log`.
- Chain the stages where several have to be measured: each launch picks the run up, plays on to the next
  stage's fight and leaves it there.
- Keep a file the **released** orb wrote — one a person played the released exe to produce — in
  `resume-fixtures/` beside the copy, and copy it out of `pointdevice_resume/` before driving any launch
  at that slot.
