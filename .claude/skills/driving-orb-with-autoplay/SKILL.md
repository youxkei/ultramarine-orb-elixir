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
- **Every launch of a game gets a trace, and the scripts see to it**: `drive-a-run.sh` and `scripts/run.sh`
  add `--trace=` under the copy's `traces/` — named by the time and the recipe, and printed as `trace: …` —
  wherever the call names none, and `cargo xtask real` writes each case's into its session. A launch
  started any other way is given one by hand. The user asked for this outright, after a session had gone
  back to watching `orb.log`.
- **Read a trace with `orb-real show <file> [<needle>]`**, which is the
  cheap record: one msgpack frame apiece with the whole `State`, the word the run was played with, orb's
  own question and the lines the log said that frame, written in chunks every few seconds — and nothing
  at all while the flag is off. `show` prints the frames that said something, state and lines together,
  so what a verbose log was read for is read as fields. The reader is
  `target/i686-pc-windows-gnu/release/orb-real.exe`, which `cargo xtask real` builds and
  `cargo xtask build --release` builds with everything else. Write the trace to a file of its own beside
  the copy, the game being the process that opens it.
  - **So a trace is the log, one frame at a time, and that is what to read once a launch is over.** Every
    line `log!` said is in the record of the frame it was said on, so `orb-real show <file> "autoplay:"`
    is the recipe's own steps *with* the state each landed on — which is what grepping `orb.log` for them
    cannot give. A second log file per subject would be the same lines again with the frame thrown away.
  - **And a launch is watched by `orb-real watching <trace>`**, which prints the three a tick wants out of
    that file: the newest frame's whole state, the newest `autoplay: step`, and the newest of the things
    that happen to a run, with the frame each was said on. Better than three greps of the log, whose state
    line is said once every so many frames — the newest record *is* the newest frame — and one command
    instead of five. A chunk is whole records, so the file is never read mid-record; what it is, is the
    launch as of a few seconds ago, which is what a 30-second tick wants anyway.
  - **And the path it is given is the one Windows spells**, `orb-real.exe` being a Windows program: a
    `/mnt/<drive>/...` handed to it comes back `アクセスが拒否されました (os error 5)`, which reads as a file
    somebody else has open and is a path it could not resolve. Write `<drive>:\...\<trace>.msgpack`, the
    same spelling `--trace` itself takes.
  - **What a trace cannot answer is whether the process is alive at all**: `scripts/game.sh running`
    answers that.
    - **And one zero from it is not a game that has gone.** A monitor armed to stop on the first `0`
      stopped on a launch that was in stage 1's boss fight and went on to play for another twenty
      minutes. Take two in a row before saying so.
  - **And a playback takes a record per update of its own**, which is what a resume is read by: the whole
    playback runs inside *one* frame of the host, so every one of those records carries that frame's
    number and it is `ms` and the state that tell them apart. `orb-real watching` shows the newest, so a
    launch answering `つづきから` at a chapter deep in a run reads as the run being put back, frame by
    frame.
- **And `--shot-every=<seconds>` puts a picture of the frame into the trace that often**, which is how a
  launch is looked through afterwards: `orb-real watching` counts them and the records hold the bytes as a
  BMP. Nothing without `--trace`. Ten seconds over a run of twenty minutes is 120 of them.
- **Put what a measurement needs into the trace, and not into the log.** A new instrument is a field of
  `orb_core::trace::Record` — a struct of the game's own words, handed over the way `trace::picture` and
  `trace::over_a_hit` are and kept for the next record — read back with `orb-real show` and asserted on in
  an e2e test through `orb_core::trace::read`. **Not a `log!` or a `detail!` line**: a line is written for a
  person and a check against one is a check against its wording, and somebody watching said so twice when
  an instrument went into the log, the second time with the log about to be switched off for it.
  `trace::OverAHit` is the example, what stood over the player on the frame a hit is seen, which
  `orb-real show <trace> "over a hit"` prints.
- **And `--log=verbose` where its own lines are the measurement, knowing what it costs.** The per-frame
  `resume:` line is a read per bullet slot plus the world's fingerprint, every frame a run is playing:
  measured on th08, `reproduction=26343us/frame` of a `frame=52144us` — half the frame, and it grows with
  the fire, 5,270us early in a stage against 26,343us in the thick of one. So a long run is driven at
  `--log=normal` with a trace, and verbose is for the launch whose question is what the rules *wanted*
  (the hit dumps) or what a resume parted on.
  - **And `update=` in the `perf:` line is orb's own per-frame work and not the game's update**, which
    is the trap `profile::Phase::Ahead`'s own doc was written for: `Update` is recorded twice a frame —
    the span before the game's update and the span after it — so the figure is per *frame* over two
    calls, and `ahead=` is the first span alone. On that th08 launch `update=27534us` was
    `reproduction` plus `state` and the game's own chain was the rest of the frame.
- Give a launch of a whole run `--pace=8`, which is a Normal run in a few minutes of wall clock with the
  same frames played. `orb_config::args` says what `--pace` does. **What it cannot do is beat the cost of
  a frame**: at eight it asks for eight updates a drawn frame and gets as many as the frame's own
  microseconds allow — two, on the launch above.
- **Use `--clear` as little as possible, and not to save time.** The user wants the real cases played as
  somebody playing them would, so a long run is played and a question about what `--clear` itself does is
  answered by reading `runtime` rather than by driving it. Two launches of it went out in one session on
  the reasoning that a setup step is not a measurement, and neither was wanted.
  - **What it is for is a run the rules cannot play at all** — a clear the bot cannot reach without a bomb
    the case forbids, or records that want a clear the bot loses under. There it is the right flag and the
    thing to say is which run the rules could not play.
- End a recipe where the run ends. A step after `play_until` waits at the logo screen.

## Watching

- **Watch the launch's trace and not `orb.log`**, and **say the state every 30 seconds**:
  `bash scripts/watch-a-run.sh <the copy, or a session's directory> <game-exe>` as a monitor, which reads
  the newest `.msgpack` under the directory through `orb-real watching` and says `stuck:` where the
  recipe's step, the scene, the run's `frames=` and its `script=` have not moved over two looks. `orb.log`
  is one file across launches and the next launch starts it over, so what it said of a launch is gone the
  moment the next comes up — which is how a session's watched attempt was left with nothing to read.
- A monitor's events arrive between calls, so do other work while the run plays instead of blocking on the
  launch. Close the monitor of a run that is over (`TaskStop`) before starting the next launch.

Say three things a tick, and carry the state line verbatim — a step number says which step a recipe is on
and the state line says whether a run is on at all. `orb-real watching` prints all three:

- the state: the newest frame's `f<N> <ms> scene=A->B stage=S frames=F … deaths=D lives=L`
- the step: the newest `autoplay: step N of M …`
- the event: the newest of `died in chapter N`, `autoplay: hit at`, a chapter's own line, `run ended`,
  `retry: …` and `front end: screen N`

A look can land on a chunk being written — `record 1 cannot be read` — and the next one reads it whole.

## When to close a run

Read the trace's newest frame and how many it holds, and close the game — `scripts/game.sh close` — saying
what the trace said before it, in these two states:

- **stalled**: the trace's frame count has not grown for several ticks, or `watch-a-run.sh` says `stuck:`
- **game over**: `lives=0` with a `play` step still waiting — the game's continue prompt is up and the
  log keeps growing
- **the same attempt over again**: two `died in chapter N` at the *same* `frames=` and the same `seed=`,
  with a `retry chapter N (retry M)` between them. A restore puts the game's memory and its generator
  back, and nothing of `orb-bot` is random, so a chapter can be lost the same way every retry — and a
  launch waiting for that to come out differently waits for nothing. **The test is the numbers
  repeating and not the fact of a retry**: a retry does sometimes land somewhere else, so read two
  deaths before calling it. Both halves are measured on th08's stage 1 — the change's own message has
  the frames.
  **And the frames are that launch's own**: a game that seeds itself from the clock at startup plays a
  different stage every launch, so two launches of one recipe are two different runs and a frame number
  from yesterday names nothing today. What holds one fight still is a resume — `pointdevice_resume/`
  keeps the stage's own seed — so a change measured against a fight is measured by picking that chapter
  up, not by starting the run again

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
- **a fight whose game holds its stage script for the whole of it** rather than for each attack, where
  `script=` then freezes on one number from the frame the boss arrives and the tick's three fields cannot
  show the fight advancing at all. **What says it is going is `spell=`**, the state line's field for the
  card that is up: a launch closed on ten thousand frames of one `script=` had gone two cards further on
  inside them. So put `spell=` in the tick and read it before calling a fight stuck — th08 is the game
  this was measured on, and `game/th08/asm.md` says which of its clocks does what.

## Pictures

- Ask for a `screenshot` step on a game that draws through D3D8 (th06, th07): `orb_core::shot` writes the
  back buffer at the end of the frame.
- Take a picture of th06c or th06nc from outside, from the monitor on the line that says the screen is up:
  `powershell.exe -NoProfile -ExecutionPolicy Bypass -File .claude/skills/driving-orb-with-autoplay/scripts/shot.ps1 -Exe th06c.exe -Out <C:\path\to.png>`
  (`-Exe th06nc.exe` for the other),
  with the recipe holding that screen for longer than one poll interval.
- **`--shot-every=<seconds>` keeps the back buffer in the trace as the launch runs**, and
  `orb-real picture <trace> <out.bmp> [<frame>]` writes one of them out — the newest, or the first taken on
  or after a frame — so that a screen kept can actually be looked at. A 32-bit BMP, which reads as an image.
  The count alone (`orb-real watching`) says a screen was kept and nothing about what was on it.

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

- **And the debugger has to be the game's own width.** `x64/cdb.exe` on a 32-bit game answers
  `Cannot debug pid N, Win32 error 0n87` and nothing else — th06, th07 and th08 are 32-bit, so theirs is
  `Debuggers/x86/cdb.exe`, and the `x64` above is th06c's and th06nc's.

- The `-c` list runs at the attach's own breakpoint: `g` runs the target, and when it stops the rest
  prints the last event, the exception record, the registers and a stack, and `q` ends it — so the launch
  is not left held by a debugger waiting for input.
- **`q` takes the game down with it, so a read of a live run ends in `qd`.** A run in play is the thing the
  question is about, and reading one field of it is not a reason to end it: a `db`/`dd` of an address
  followed by `q` answered which card th08's stage 4 fight was on and closed the launch that was in it. A
  read across a game somebody is watching is theirs to allow either way — see the rule above about not
  disturbing a run.
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
- **Measure a change to one card by picking the run up at the chapter before it, and not by playing the run
  to it again.** A whole-run case is three quarters of an hour to reach the Extra's fifth card; a pick-up
  plays that stage's own frames back in and is at the card in a minute. Somebody watching asked why every
  launch started from the first stage, and there was no reason: `pointdevice_resume/` holds only the chapter
  a launch ended in, so run
  `bash .claude/skills/driving-orb-with-autoplay/scripts/keep-every-chapter.sh <session or copy dir> <dir beside the copy> <copy>/orb.log`
  in the background beside every session, and each chapter written is kept as it is written. Pick one up
  with a `--keep-in=` of its own holding the file in its `pointdevice_resume/`, so the kept one is never
  written over, and a recipe that walks to the run's own team select and answers `question: resume`.
  The whole-run cases are for the end, with the build that is handed over.
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
