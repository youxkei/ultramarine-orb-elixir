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
  - **Name the recipe by its own file name and not by a path**: the script `cd`s into the copy and hands
    `--autoplay=<recipe>` to a Windows program, so a `/mnt/...` path reaches orb as a file it cannot open
    and the launch comes up with `autoplay: None` — a game that plays nothing and a recipe whose steps
    never run.
  - **And a case's recipe beside the copy is a copy that goes stale.** `cargo xtask real` reads
    `crates/orb-real/recipes/<game>/` and writes each case's recipe into its own session directory, so the
    file beside the exe is whatever some earlier session left: two launches of a clear read
    `step 8 of 33 gave up` and raced the difficulty select into `diff=2` against a version with no answer
    to orb's mode question in it. Copy the repository's own file out under a name of its own before
    driving it, and say in its header that it is a copy and of what.
  - **A case's own `Case::speed` is the harness's and not the script's.** `cargo xtask real` passes
    `--speed={case.speed}`, which for th06nc's whole-game cases is **1**; a launch driven through
    `drive-a-run.sh` gets the default instead and overshoots every select the game repeats a held key on.
    Pass `--speed=` to match the case.
  - **And the script installs no build.** It launches the `orb.exe` already beside the game, so a launch
    meant to try a change runs whatever the last session left there unless the build is put in first.
    **`scripts/deploy.sh` is the 32-bit one and is wrong for a 64-bit game**: run against `th06c` or
    `th06nc` it overwrites the working exe with one that answers
    `orb: no game orb knows is in …; it knows 東方紅魔郷.exe 1.02h, th07.exe …`. Theirs is
    `cargo build --release --target x86_64-pc-windows-gnu -p orb-launcher`, copied in by hand —
    `orb_real::Game::target` is what the harness picks it by, and `xtask/src/real.rs` builds it that way.
- Run one game at a time. Ask `scripts/game.sh gone <game-exe>` before a launch and `scripts/game.sh close
  <game-exe>` after a launch of your own that is over or has gone wrong. The exe to name is the game's
  (`game::KNOWN` in `orb-core`).
  - **And nothing heavy runs beside it**: no `cargo xtask check`, no build, no test suite while the game is
    up. A launch is a measurement of what the game did frame by frame, and a compile taking the machine's
    cores is a launch whose pacing is the build's as much as orb's. Do the reading and the editing while a
    run plays, and leave the checks until `scripts/game.sh gone` answers.
- Give every launch `--log=verbose`: the per-frame lines are the ones that say what orb did to the run.
- Give a launch of a whole run `--pace=8`, which is a Normal run in a few minutes of wall clock with the
  same frames played. `orb_config::args` says what `--pace` does.
- **Use `--clear` as little as possible, and not to save time.** The user wants the real cases played as
  somebody playing them would, so a long run is played and a question about what `--clear` itself does is
  answered by reading `runtime` rather than by driving it. Two launches of it went out in one session on
  the reasoning that a setup step is not a measurement, and neither was wanted.
  - **What it is for is a run the rules cannot play at all** — a clear the bot cannot reach without a bomb
    the case forbids, or records that want a clear the bot loses under. There it is the right flag and the
    thing to say is which run the rules could not play.
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
- **a `wait_until` step being the one still waiting** — that step presses nothing, so the run stands where
  it is: the boss takes no more damage, the stage's script does not advance, and the state line reads the
  same numbers for as long as the step lasts. **Which looks exactly like a stall and is not one**, and it
  is worth reading the step before calling anything stuck: a th06nc launch under `--clear` was written up
  as a game that stands still in stage 1's boss fight — `script=5093` with `frames=` climbing — and the
  boss's life had stopped on the very frame `step 31 of 33 done` was written, `chose=0x0000` on 5,404 of
  the frames after it. The same flag with a recipe that asks the run *past* her took it into stage 2.

## Pictures

- Ask for a `screenshot` step on a game that draws through D3D8 (th06, th07): `orb_core::shot` writes the
  back buffer at the end of the frame.
- Take a picture of th06c or th06nc from outside, from the monitor on the line that says the screen is up:
  `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .claude/skills/driving-orb-with-autoplay/scripts/shot.ps1 -Exe th06c.exe -Out <C:\path\to.png>`
  (`-Exe th06nc.exe` for the other),
  with the recipe holding that screen for longer than one poll interval.

## When the game dies and says nothing

A process that goes with nothing more in the log has left no line to read, and `orb::crash`'s filter
catches only what reaches it. **A debugger attached across the launch says in one sitting what a
bisection of flags takes twenty launches to circle** — `cdb` is on this machine:

```sh
# the launch in the background, then its pid off orb.log's first line
pid=$(grep -a "attached to pid" <copy>/orb.log | tail -1 | sed -E 's/.*pid ([0-9]+),.*/\1/')
"/mnt/c/Program Files (x86)/Windows Kits/10/Debuggers/x64/cdb.exe" -p "$pid" \
  -logo '<copy as Windows spells it>\cdb.log' -c "g; .lastevent; .exr -1; r; kb; q"
```

- The `-c` list runs at the attach's own breakpoint: `g` runs the target, and when it stops the rest
  prints the last event, the exception record, the registers and a stack, and `q` ends it — so the launch
  is not left held by a debugger waiting for input.
- Write the log to the copy's own directory as a **new** file and read it from WSL afterwards; cdb is a
  Windows process, so the path is a Windows one.
- Attach as soon as the pid is up rather than at the moment of the fault: attaching costs the game a
  moment and nothing else, and what is wanted is the first fault of the launch.
- A fault the game *means* looks like a crash and is not one: th06nc ends a process it has decided is
  unsound with `mov dword ptr [0],0FFFFFFFFh`, so `c0000005` writing to address nought with the game's
  own module in `ExceptionAddress` is an assertion of its own and not a pointer it followed by accident.
  Read `.rdata` around the string the reporter is handed.

## Reading a launch that holds a span back

`--hold-back-at=SOURCE:OFFSET+LEN` takes a span out of every restore of a launch, which is how a restore
that kills the game is bisected. **A cut has four outcomes and only two of them are about the fault**:

- **it reached the boundary and lived** — the fault is among the bytes that span held back;
- **it died at the boundary** — the fault is not in that span;
- **it died earlier** — the flag and not the fault. A restore that puts `.data` back without a heap
  block, or half of one allocation, is a game whose pointers name memory that has moved;
- **the run went dead** — the stage runs on with its generator unmoved, nothing spawning and no chapter
  taken, until the recipe's own `give_up_after` ends the step. Same cause as the one above and no easier
  to read: the cut split a structure.

So a cut is written to cover **whole allocations**, and the log says what it actually held:
`restore: holding back 0x1cf000 bytes of the 0x200000 asked for at 0x0 into heap1.3` is a whole segment,
where `0x2000 bytes of the 0x180000 asked for` is a launch about nothing. And each restore says which
bytes of the span the run had changed — `restore: of heap1.1 the run changed 0x198 byte(s) in 216 run(s)
between 0xc0 and 0x3c48`, the first twelve written out as `0x1128+0x3 cc0d6d -> e86b3d` — which is the
span's own shortlist, since a byte the run has not touched is one a restore writes with what is there.

**And what names the block is not the bisect**: `restore: 11 static(s) of .data point into heap1.1:
+0x74bc08=…` is said once a launch per source, and a static that points into the span is a name `asm.md`
and a disassembly can look up. That is what turned th06nc's stage 2 from a span into
`[0x140a6ec08]`, the object a conversation runs in, in one launch.

## Across launches

- Copy the first launch's log out to the scratchpad (`cat "$log" > <scratchpad>/first.log`) before the
  second launch appends to or rotates `orb.log`.
- Chain the stages where several have to be measured: each launch picks the run up, plays on to the next
  stage's fight and leaves it there.
- Keep a file the **released** orb wrote — one a person played the released exe to produce — in
  `resume-fixtures/` beside the copy, and copy it out of `pointdevice_resume/` before driving any launch
  at that slot.
- **Give a launch that is measuring a run `--no-resume`, or a recipe that answers `question: resume` with
  はじめから.** Without either, a copy whose `pointdevice_resume/` holds a chapter opens the run by playing
  thousands of recorded frames back into it, and what the numbers are then about is a playback and not a
  run. Four launches read 17–24ms a frame that way where the same case fresh reads 3.3ms. `--no-chapters`
  offers no chapter at all, so a launch given it is always the fresh one — which is what made that
  comparison read as a mode's cost.
- Hold a run's own series against the playback that put it back with
  `scripts/part-of-a-playback.sh <the recording launch's log> <the playback launch's log>`: it names the
  first frame and the first *field* the two disagree about, and whether the playback holds the run's whole
  line a frame out of step. orb's own `resume: the playback parted from the run at frame N` names the
  generator and nothing else. **Add a field's name as a third argument** — `shot`, `lives`, `randoms`,
  `shot_at` — and every frame that one parts on is printed instead, with a count: a field that parts once
  and agrees again is a transient and one that parts for the rest of the chapter is a drift, which the
  first parting alone cannot tell apart.
- Read what a launch's snapshots covered with
  `scripts/the-sources-a-snapshot-saved.sh <the launch's log> [<snapshot>|all]`: a line per source —
  `data`, `heap1.3`, `reserve2` — with the ranges, the committed bytes and the lowest base it sat at.
  **Which sources there are is a launch's own answer**, and it is where a cut of the restore bisect comes
  from: `--hold-back-at=heap1.3:0+200000` holds that whole region out of every restore, the span being
  clamped to it. A cut that names a source rather than an offset into the heap is the point — the
  distances between a heap's segments are the OS's and differ every launch.
