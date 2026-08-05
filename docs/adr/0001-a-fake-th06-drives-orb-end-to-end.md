# 1. A fake 紅魔郷 drives orb, and a test only sends input

**Status:** accepted, not built. The plan at the end is what is left.

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
- The injection made reachable from a test. Today orb's `RUNTIME` is assembled by `DllMain` and the
  hooks are installed by patching the game's imports, so nothing outside the DLL can get a runtime to
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

## Plan

1. `FakeGame` in `orb-sim`, over the existing `Image`: its own frame, its own state, and the hook
   calls in the real order.
2. Whatever `orb` has to expose for a runtime to be attached to a game that is not a real process.
3. One scenario first, the whole of a pointdevice run, replacing the one that got the shape wrong:
   mode chosen, stage, spell card, death, retry, the attempt count in the game's record, the run left
   in a file and picked up by a second fake game.
4. Then the legacy run beside it, which asserts the same things do *not* happen: no retry menu, no
   attempt counted, the score in the game's own file.
