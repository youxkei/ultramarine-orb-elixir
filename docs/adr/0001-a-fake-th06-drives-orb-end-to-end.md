# 1. A fake 紅魔郷 drives orb, and a test only sends input

**Status:** accepted and built. Two scenarios stand on it — `orb/tests/pointdevice_run.rs` and
`orb/tests/legacy_run.rs` — and *Where it landed* at the end says how the shape differs from the plan
this was written with, and what building it found.

## Context

orb is a DLL injected into a game. Nothing in it runs because orb decided to: the game calls it —
`Supervisor::OnUpdate` reaches `on_update`, the draw chain reaches `on_draw`, the frame loop and the
input read are hooks in the middle of the game's own. What orb does to a run is therefore not a
function anybody calls; it is what falls out of the game running with those five hooks in it.

The suite already reaches a long way into that. `orb-api` puts Windows behind a seam, `orb-sim`
answers it, and forty-one scenarios drive the real `orb-core` against a simulated host — the frame
loop against a display a test declares, the mode question against a keyboard a test presses, a whole
`State` read out of a game laid out by hand. What none of them do is *run the game*.

**An attempt that got the shape wrong, and how it gave itself away.** A scenario was written over a
whole pointdevice run — the mode chosen, a stage played, a death, the chapter put back, the run left
in a file and picked up again. It passed, and it was still the wrong thing, because of how it played
the game's part:

```rust
image.playing(in_memory(at, 2, 0));                    // the fake game's memory
observe(&image, &mut chapters, &playing(at, 2, 0));    // a State built by hand beside it
```

Two sources of truth. In production `lib.rs` reads the one: `let state = game.read_state()`. Handing
orb a `State` the test wrote means orb decides from what the test *said* while the memory holds
something else, and the two drifted the moment the scenario needed a boss: the hand-built `State`
said `boss_present: true` where `read_state` — reading the same memory the restore would put back —
said false, because the fixture's writer had no boss in it. The field-for-field assertion on the
restore failed, and the failure was the scenario's, not orb's.

The same mistake in a second place. To assert which file a pointdevice run's ranking goes to, a
`pub fn goes_to_ours()` was added to `score.rs` — a function no production caller has, wrapping a
private flag, so that the scenario had something to read. A test that asserts on an observer written
for the test cannot fail for a real reason. It was taken out again.

## Decision

**The fake 紅魔郷 is a game, not an address space.** It plays the role the real exe plays: it owns its
own memory, advances its own state, and calls orb's hooks in the order and at the points the real one
does — the same way the injected DLL is reached in production.

**A scenario's whole vocabulary is the host and the input.** It says how Windows behaves — the
monitor's rate, the compositor, the wake jitter, which window is in front — and then it presses keys
and pushes pad buttons. Everything else is read back: what orb wrote into the game, what it put in
the log, what the game's own records hold.

So a scenario looks like this, and nothing in it hands orb an opinion:

```
sim: a 120Hz display, the compositor agreeing, the game's window in front
game: stage 1, Normal, Reimu A
press Z at the title menu          → 完全無欠モード is chosen
run 300 frames                     → the stage settles, chapter 1 is taken
run until a boss and a spell card   → a chapter begins on the card
die                                → the retry menu is up
push the pad's decide              → チャプターをやり直す
                                   → the game's memory is what it was at the card
                                   → the record the score screen reads counts one more attempt
```

**What is read back is what the game holds, not what orb thinks.** The attempt count against a spell
card is the number the 完全無欠 ranking screen shows, and it lives in the game's own `CARD_HISTORY`
record — so a scenario reads it out of the fake game, through the fixture that laid it out. Nothing is
added to production so that a test has something to look at.

## Consequences

**What has to exist.**

- A `FakeGame` that owns the laid-out image and has a frame: read input, advance its own state, call
  orb's hooks, present. Its state is the only truth about the run; `read_state` is how orb learns it,
  as in production.
- The injection made reachable from a test. orb's `RUNTIME` was assembled by `DllMain` alone and the
  hooks installed by patching the game's imports, so nothing outside the DLL could get a runtime to
  call. A scenario needs the equivalent of "attached to this game" without a real process to patch.
- Orb's hook entry points callable — `on_update`, `on_draw`, the frame loop, the input read — in the
  order the real game reaches them, which is what `SPEC.md` describes and what a `FakeGame` has to
  copy rather than invent.
- The fake game able to *do* what a scenario needs to talk about: a stage's waves, a boss, a spell
  card starting, the player being hit. Those are its behaviour, written once, rather than a `State`
  each scenario assembles.

**What it costs.** The fake game is a second implementation of a small part of 紅魔郷, and it can be
wrong in the same direction as the reads it feeds — a wrong offset makes the writer and the reader
wrong together. That is already true of the laid-out image and is answered the same way: the offsets
are the real game's to confirm, and `DONE.md` keeps the measurements that do it. What the fake game
buys is everything built on top of the offsets, which is where orb's code is.

**What it rules out.** Scenarios that call orb's own functions to move a run along. If a scenario
needs orb to be in some state, the way to get there is to play the game into it.

**Where the tests live.** In `tests/`, not in a `#[cfg(test)]` module, because `cfg(test)` is false
there — so a scenario reaches the simulated Windows the only way the shipped DLL reaches the real one,
through the `sim` feature, and nothing it drives can have a test-only path in it.

## Where it landed

**The game is `orb/tests/fake`, not `orb-sim`.** The plan put it in the simulator, and it cannot go
there: a game that calls orb's hooks has to name `orb`, and `orb-sim` is what `orb` is built *on* —
`orb-core` depends on it. The only crate that can see both is `orb` itself, and the only place in it
where `cfg(test)` is false is `tests/`. So the fake game is a module shared by the scenarios, and the
split is the one the plan had between the two halves anyway: **where each thing in the game's memory
lives is `orb-core`'s `th06::image`, beside the offsets it is written from, and what the game *does*
over time is the fake game's.** A scenario names no address.

**One game per process, and the process is the game.** `RUNTIME`, the record of what a run has pressed,
which file its score goes to and the device it draws through are one apiece in orb, the way they are in
the game — and the runtime cannot be made one per thread even for a test: `DllMain` writes it on the
thread the launcher's remote `LoadLibraryW` runs on, and the frame hook reads it on the game's main
thread. So **a scenario spawns the test binary again for itself**, told to run that one test and nothing
else, and reports what the child made of it. Which is what lets several scenarios share a file and none
of them wait for another — the child is the launch, and this side is the harness. Where the harness is
not naming its threads there is nothing to tell a child to run, so the scenario runs in place, serially,
which is what `--test-threads=1` asked for.

The game is dropped rather than leaked, in the one order that works: the runtime first, so orb's overlay
is released through a device that is still there, then the device, then the simulated Windows it was all
read through.

**Both halves of a run in one launch, rather than a second fake game.** The plan had the run picked up
by a second game; a second one in the same process would want a second recording device while the
first still holds the lock. It is not a weaker scenario: the run is given up, which ends it — and
ending a run is what drops the record of everything it pressed — so what the pickup reads is the file
and only the file, which is the property the plan wanted a second process for.

**What orb exposes:** `attach_to`, with the game's own functions in place of the trampolines
`hook::install` would have left behind, its hook entry points `pub`, and `recording.rs` reachable from
`tests/`. Not the frame loop: orb's own runs the game's `Present` and its sound by calling the exe's
code at its own addresses, so a game that is not a real process has nothing there to call. A scenario
is therefore a `--no-frame-loop` run — the game's own draw-then-update order with orb's two hooks in
the middle of it, which is a configuration orb ships and a switch somebody can ask for.

**What a scenario reads off the screen is text, not only geometry.** A device is handed a bitmap and
never a string, so a quad says what it says only if the string is baked again through the same font and
held against what was uploaded — which is `recording::Screen::says`, and it is why the fake game's
overlay is built on the same `font.ttf` at the same two sizes orb builds its own with. That is what
lets a scenario say the retry menu named the chapter it was offering, and which item the cursor was on:
`menu_ui` draws that one in `SELECTED`, so the answer is a colour on the screen rather than a field to
read. The fake game draws its own ranking through the same machinery, so the count against a spell card
is a number somebody could read rather than one only the record holds.

**Two thirds of the host vocabulary above is not reached, and both follow from that.** The display and
its compositor are the frame loop's, and there is no frame loop here — the pacing's own twenty-one
scenarios in `orb-sim/tests` are where a monitor is declared. The pad is the other: `Game::pad` reads
the device the *game* reads, which is its own DirectInput controller or winmm's joystick 0, and neither
is something a laid-out game has — so a scenario answers on the keyboard, and what says a pad answers
these questions is `orb-sim/tests/mode.rs`, where a `Pad` is a value a test hands over.

**What was in `orb-sim/tests` and is a scenario has moved here.** Those drove the real `orb-core`
against a simulated host by *calling* it — `Question::update` handed a keyboard and a `Pad` — which is
the shape this rules out: a test working the question itself plays the game's part from a script of its
own, and cannot fail for the reasons a game supplies. The question about the mode is thirteen cases
over a game now, and the whole `State` is read at points a game reached by being played to them. The
pad went with them, and that took a controller: an object and a vtable in the game's own memory with
the addresses of three real functions in the slots its read calls through, the same shape as the
Direct3D device orb draws through. It is the first thing to run `Th06::controller_pad` at all — the
poll, the buttons out of the device's array, the stick against the game's own threshold, and the
mapping — where before a `Pad` was handed straight to the menu and none of that was reached.

**What could not move, and it is not a matter of effort.** The twenty scenarios over the frame loop
drive `pacing`, `settle` and `wait` through a harness that composes the loop the way `render` does,
because `render` itself cannot be reached: two of the calls it makes are `Th06::present` and
`Th06::play_sounds`, and those are the game's own code at 0x00420b50 and 0x00431270. A laid-out address
space answers reads, not execution, and there is nothing to put at those addresses — the test binary's
own image is already there. Making them answerable would mean a branch inside `Th06` that only a test
takes, which is the thing this decision is against. The four over the log are not scenarios at all:
what `log!` formats and which level keeps which line is nobody's behaviour but the log's. Both sets
stay where they are for a second reason as well — one test to a binary, orb's log and profile being
process-global — which `tests/` gives and a `#[cfg(test)]` module in `orb-core` cannot.

**It found the thing this was written to find.** `memtrack::regions` answered a laid-out game out of a
`#[cfg(test)]` branch — and `cfg(test)` is false in a crate compiled as a dependency of a test binary,
which is the whole reason these scenarios live in `tests/`. So the first scenario reached the
production heap walk with no heaps tracked and got a chapter covering `.data` and nothing else: the
fight's own block was outside it, and the clock of the attack a chapter began on did not come back
with the chapter. The branch is a seam facade now, `orb_api::mem::game_regions`, like every other host
call. A test-only path in the code under test is exactly what this decision was against, and one
scenario was enough to find the one that was there.
