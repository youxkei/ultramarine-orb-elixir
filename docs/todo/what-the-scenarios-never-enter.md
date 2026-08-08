# What the scenarios never enter

**Measured, not read.** The scenarios drive orb through its own hooks and its own frame loop, so what
they cover is not something anybody can work out by reading — a `read_state` that runs on every frame of
every scenario looks the same in the source whether or not one line of it is ever asserted. This is the
list of orb's and orb-core's code that **no scenario executes at all**, taken off a coverage run, and the
order worth working through it in.

Everything here is about the scenarios. **The unit tests are deliberately not in the measurement**: what
matters is that the composed thing is covered, which is [0001](../adr/0001-a-fake-th06-drives-orb-end-to-end.md)'s
whole argument, and a line that only a unit test reaches is a line no scenario stands behind. The run is
`orb-sim`'s tests and nothing else — orb-sim does not depend on orb-core, so its own twelve unit tests
cannot execute a line of it, and the orb-core figures below are the scenarios' alone.

## How to read it

**Read the zeros; ignore the percentage.** Coverage counts what was *executed*, not what was *asserted*,
and in this suite the two come apart in one direction: `Th06::read_state` runs on every frame of every
scenario, so all of it reads as covered while only some of its fields are asserted anywhere. A percentage
driven upwards here buys nothing. What a zero says, though, is exact: nothing in the suite has ever run
that line, so no scenario can be relying on it.

## Measuring it again

```sh
LLVM_MINGW=<an llvm-mingw installation> cargo xtask coverage            # the table below
LLVM_MINGW=…                            cargo xtask coverage --missing  # the line ranges nothing ran
LLVM_MINGW=…                            cargo xtask coverage --html     # browsable, per line
```

**`--missing` is what the lists further down were read off**, file by file: it prints every line no test
executed, so a range that has moved or gone is visible as a diff rather than as a claim to take on trust.
`cargo xtask` with no task says what `LLVM_MINGW` has to be and where to get one.

**The numbers below are the suite at 309 tests**, of which the run measures the 127 that are `orb-sim`'s.
A later run that disagrees has either had work done on it or has grown a scenario, and `--missing` is what
tells the two apart.

### Why the task needs a target of its own

**It measures `i686-pc-windows-gnullvm`, not the `i686-pc-windows-gnu` everything else is built for**, and
that is not a choice — **rustup ships `profiler_builtins` for the first and not for the second**, so
`-C instrument-coverage` has no runtime to link against on the target that ships. The ABI, the CRT and
every `cfg` are the same; the linker and the unwinder are what differ, so what the run measures is the
code that ships. `.cargo/config.toml` carries the `crt-static` that target needs and says why, and
`crates/xtask/src/main.rs` carries the rest: the linker to hand over, the `WSLENV` the test binaries need
to be able to write a profile through WSL interop at all, and why `orb-launcher` is left out of the run —
its artifact dependency pins `orb`'s cdylib to the target with no profiler runtime, so a run including it
does not compile.

### Measuring on the shipping target was tried and does not work

Recorded so that nobody
spends the day on it again: `-Z build-std=std,panic_abort,profiler_builtins` does build the runtime from
compiler-rt's sources, and the binaries then link and run — but the profile is unreadable. compiler-rt
takes the bounds of its data, names and counters from one-element sentinel variables in `$A`/`$Z`
subsections, which assumes MSVC's chunk layout where the real data begins immediately after the sentinel.
With mingw's gcc and lld every boundary is padded instead: measured **names +4, counters +8 and data
+0x40**, against the `+1` and `+sizeof` the runtime computes. `llvm-profdata` rejects every file —
`symbol name is empty`, then `counter offset is negative`. `__attribute__((aligned(1)))` on the sentinels
removes none of it. Two of the three paddings can be justified from first principles and the third cannot,
which is where that road ends: numbers nobody can vouch for are worse than no numbers.

And with GNU ld rather than lld it does not get that far at all — `.lcovfun` and `.lcovmap` land at VMA 0,
below the image base, and the PE will not load. Ubuntu's mingw gcc cannot be made to use lld: it resolves
`/usr/bin/i686-w64-mingw32-ld` absolutely, and `-B`, `COMPILER_PATH` and `PATH` change none of it — `-B`
does not even redirect `as`.

## orb-core, as the scenarios reach it

| | lines | covered |
| --- | --- | --- |
| `frame.rs` | 537 | 97.2% |
| `log.rs` | 97 | 96.9% |
| `game/th06/image.rs` | 716 | 95.3% |
| `menu.rs` / `mode.rs` / `profile.rs` | 52 / 50 / 67 | 94% |
| `input.rs` | 25 | 88.0% |
| `game/th06/mod.rs` | 871 | **76.1%** |
| `game/th07/image.rs` | 38 | 71.1% |
| `audio.rs` | 224 | **63.0%** |
| `game/mod.rs` | 68 | **60.3%** |
| `game/th07/mod.rs` | 312 | 19.6% |

`th07`'s is a `Game` that declines everything — see
[0004](../adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) — so 19.6% is the shape of that decision
and not a gap.

## What is left, in the order worth doing

**1. The music put back with a chapter, byte for byte.** `Music::restore`, `Music::still_current` and the
`with_locked_buffer` instantiation behind them are never entered, which is most of what `audio.rs` is
missing. `scenario_the_music_across_a_restore.rs` covers the *seek* a resume makes — `play_from`, the
countdown moving with the file — and nothing covers a **retry** putting the buffer, the play cursor and the
file position back where the chapter had them. The retry walk is already written in three scenarios; what
it wants is a stage with `Fake::streams_its_song` under it. Cheapest thing here and the one that matters
most, the restore being what a chapter's music *is*.

`Music::margin` goes with it, reached only through `watch_music` — which needs `self_check` or
`stress_restore_frames`, and **no scenario sets either**.

**2. A pad winmm has — and the seam it wants first.** `Th06::pad`, `Th06::winmm_pad` and `axis` are never
entered: `scenario_mode_on_the_pad.rs` drives the DirectInput path, and `Controller::GetControllerInput`
asks winmm only where its own enumeration found no device. That is the path that used to work and the one
*What the pad now reaches* in [TODO.md](../../TODO.md) says must not have been broken — and nothing in the
suite would say if it had been.

**It is not a scenario anybody can write today**, which is the first thing to fix. What stands in the way,
and what is already in place:

- **The reads are two, and small.** `crates/orb/src/joystick.rs` calls winmm in exactly two places:
  `joyGetPosEx(DEVICE, &mut info)` in `poll`, and `joyGetDevCapsA` in `read_caps`. Everything else in that
  file is arithmetic over what they answered.
- **The neutral type already exists.** `Th06::pad` takes `Option<Reading>` — `Reading { buttons, y, pov }`,
  orb-core's own — and `winmm_pad` and `axis` are pure functions of that and `g_JoyCaps`, which a scenario
  can already lay out. So the seam only has to make `joystick::reading()` answerable.
- **The obstacle is the thread.** `orb_api::install` is per *thread*
  (`crates/orb-api/src/install.rs` says why: the harness runs tests side by side, so a static would be two
  tests writing each other's game), and `joystick::poll` runs on a thread orb spawns itself in
  `start_polling`. A seam alone therefore changes nothing: that thread would see no simulated host and fall
  through to the real winmm. **Read `orb-sim/tests/log_off_thread.rs` first** — it is a scenario about work
  done on another thread, so whatever it does about the installation is the precedent to follow or to
  reject with a reason.
- **What else that file asks of the host**, and what it would take to run the loop deterministically rather
  than only to sample it: `SetThreadPriority` with `GetCurrentThread`, and `Sleep`. `orb_api` has a
  `thread` seam and a clock already — `Win::wait` — so those are small moves of the same kind, and until
  they are made a scenario can drive `reading()` but not `poll`'s cadence.
- **What stays out of it.** `install`'s hook over the game's own import of `joyGetPosEx`, and `calibrate`
  writing a `JOYCAPSA` into the game's memory: those patch rather than read, and they belong with the
  unreachable list below.

Done when a scenario answers a pad on one of orb's own menus with the game holding **no** DirectInput
device — `Image::no_controller`, which item 5 lists as unused for want of exactly this — so the answer has
to come through `winmm_pad`.

**3. The rest of `th06/mod.rs` that a scenario can reach.** `live_handles` (the anm manager's texture
range, which stays null in a laid-out game — writing one makes the walk happen), `acquire_input` (the
keyboard device taken again after the window has been away), `panel_tile`, `stage_song` with `path_at`
under it, and `windowed` / `force_windowed`.

**4. `orb/src/tuning.rs`, and it is the largest zero in the tree.** 290 lines, **41 functions, none of them
ever entered** — the whole midstage table pass: `Tuning::new`, `load`, `add`, `propose`, `judge`,
`judge_down`, `reject`, `pass`, `write`, `table`, `begin_stage`. `--collect` and `--judge` are what a
chapter table is built with, and no scenario has ever run one. This was unreachable before: a pass needs a
replay to step through. **A replay record is in the laid-out game now** —
`Fake::watches_a_replay_of_its_stages`, and
`scenario_moving_between_a_replays_stages.rs` already moves between its stages — so a `--collect` scenario
is writable for the first time.

**5. Dead test support in `game/th06/image.rs`.** Four items nothing calls: `Image::no_controller`,
`Image::laid_out` (every scenario takes the seeded one), `Playing::default`, and
`Image::cuts_the_shake_from_the_chain` — the branch a shake takes when its own 80 frames run out, which the
one scenario about a shake never reaches because it moves stages half way through. Use them or delete them.

## What no scenario can reach, by construction

Not work to do, and here so that a zero beside them is not read as one:

- **`orb/src/hook.rs` (2.4%), `pe.rs` (0%), `memtrack.rs` (5.3%), `threads.rs` (0%), `crash.rs` (0%)** —
  patching a real prologue, reading a real PE, walking a real heap, a real thread, a real fault. *That the
  hooks hold* and *Addresses and the bytes at them* in [TODO.md](../../TODO.md) are these.
- **`Th06::stop_music` and `restart_stage_music`** — `StopBGM` and `PlayAudio` at their own addresses.
- **`Th06::hooks` and `frame_calls`** — a laid-out game hands its own functions over through `Originals`
  instead, which is [0002](../adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
- **`game::known_named` and `known_by_exe`** — which game a launch matched, which only a real launch does.
- **`orb/src/joystick.rs` (3.8%)** — as it stands, and only as it stands: the two winmm reads are not
  behind the seam and the poll loop runs on a thread of orb's own. That is **item 2 above** rather than a
  fact of life; what stays unreachable either way is the hook over the game's own import of `joyGetPosEx`
  and the `JOYCAPSA` `calibrate` writes into the game's memory.
- **`orb/src/window.rs` (43.8%)** — the half that talks to a real desktop.
