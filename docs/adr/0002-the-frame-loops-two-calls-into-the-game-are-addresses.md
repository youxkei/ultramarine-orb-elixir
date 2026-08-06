# 2. The frame loop's two calls into the game are addresses it hands over

**Status:** accepted and built. `Game::frame_calls` answers with the two, `render` calls through them,
and the frame loop's scenarios are eleven files in `orb/tests` that a laid-out 紅魔郷 drives through
`render` itself: the nine `pacing_*` and `log_deferral` moved out of `orb-sim/tests`, and `frame_loop.rs`
over the loop's own shape. `orb-sim/tests/pacing/mod.rs` and its 414 lines are gone. What the built shape
does differently from the plan below is at the end of *Consequences*.

It overturns one claim of [0001](0001-a-fake-th06-drives-orb-end-to-end.md): that a game laid out by
hand cannot drive `render`. The obstacle that document names is real — `Th06::present` and
`Th06::play_sounds` call the game's own code — and the conclusion drawn from it was not.

## Context

[0001](0001-a-fake-th06-drives-orb-end-to-end.md) put a 紅魔郷 that plays the game's part in front of
orb, and every scenario a game can drive is driven by one now — a whole run in each mode, the question
that chooses between them on the keyboard and on a controller the game answers with, a `State` read at
frames a game was played to. The frame loop was written off in that document as the one thing a game
could not drive, and that is wrong. This says why it is wrong, and what has to change.

**The twenty-one scenarios over the loop drove a copy of it.** `orb-sim/tests/pacing/mod.rs` was 414
lines and said so itself:

> The game's frame loop, as `orb/lib.rs` composes it, for the pacing scenarios to drive. The order here
> is that function's order and the marks are its marks. A harness that waited and handed over in some
> order of its own would be measuring itself.

Which was honest about what it was and could not fix it: nothing held the copy to the original. What those
scenarios established was the arithmetic of `frame::Pacing` — the cadence, the rates, the allowance
climbing, sixty frames a second for every compose time there is room for — and what they could not
establish is that `render` asks for any of it in the order they assumed. Nothing covered:

- the order itself: `prepare_frame` before the wait, because it ends in a `Clear` that blocks while the
  display still holds a buffer; the update before the draw, which is the frame of input lag removed; the
  present not waited on, `DwmFlush` at the top of the next frame being what waits;
- the marks `finished` is handed, which every number in the pacing log is worked out from;
- the four ways out of it before a frame happens at all — no runtime, no device, the window behind, a
  null chain target — one of which paces by the clock rather than returning, because the game's own loop
  calls straight back and a return that waits for nothing spins a core;
- what it makes of the chain's answers: `CHAIN_EXIT_SUCCESS` and `CHAIN_EXIT_ERROR` becoming the two
  `RenderResult` values the game's loop expects back.

**What orb *says* about the rate was uncovered as well, and that is the cheaper half.** Every pacing
measurement in `DONE.md` was read off two things: the `fps` on the status line beside the game, and the
line orb writes per reporting period — `frame: 711 frames, 16651us apart, 0 shown late, gaps in
refreshes 2x711`. Neither was asserted anywhere. The harness even called `report()`, but only to print it
in a failure message, so a run whose rate was right and whose report of it was wrong would pass. That
line is already behind the seam, the log being a simulated one in these scenarios, so it costs nothing
to hold orb to it.

The status line's `fps` is the same number formatted — `interval_us` is the measured gap between
handovers, smoothed 31 parts in 32 — and it cannot be read in a scenario at all: `window::write_beside`
wants a window it can `GetClientRect`, and returns without one. Which is a separate thing to want, and a
smaller one: an average over 32 frames held for 30 more says a run settled at sixty and cannot see the
second that lost four frames, which is the question these scenarios ask. So the rate stays measured from
the clock, orb's own line is held against it, and the status line is its own piece of work — see
[TODO.md](../../TODO.md).

**What blocked it was two methods, and nothing else.** Every other call `render` makes was answerable by a
laid-out game already: the device and the window come out of its memory, `prepare_frame` is a viewport
through the device's vtable and two writes and a read of the game's own memory, the chain's update and
draw are reached through `RUN_CALC_CHAIN_TARGET` and `RUN_DRAW_CHAIN_TARGET` — statics `attach_to` can
fill with the fake game's own hooks — and the clock, the display and the compositor are the simulated
Windows the pacing scenarios already declared. The two that could not be answered were

```rust
unsafe fn play_sounds(&self) {
    let play: unsafe extern "fastcall" fn(usize) = unsafe { std::mem::transmute(PLAY_SOUNDS) };
    unsafe { play(G_SOUND_PLAYER) };
}

unsafe fn present(&self) {
    let present: unsafe extern "fastcall" fn(usize) = unsafe { std::mem::transmute(PRESENT) };
    unsafe { present(G_GAME_WINDOW) };
}
```

`PLAY_SOUNDS` is 0x00431270 and `PRESENT` is 0x00420b50, and each of those is the whole of the method:
an address, and what to call it on. A laid-out address space answers reads rather than execution, so
there is nothing at either for a scenario to reach — and there is nowhere to put anything, the test
binary preferring 0x400000 with an image of 10.5MB, which contains both.

## Decision

**The two functions orb's own frame loop calls in the game are addresses the game hands over, and orb
calls them.** A `Game` answers with the function and what to call it on; `Th06` answers with its two
constants, and a game that is not a real process answers with functions of its own.

That is what the rest of the trait already does with everything orb calls: `chain` is an address, the
hooks are `Patch`es carrying a target, and the update and draw `render` runs are addresses orb was
handed and stored. These two are the odd ones out, and the inconsistency is not only a test's problem —
the `Game` seam exists so that porting to another Touhou game means supplying addresses and offsets, and
a port today has to write the transmute again for these two while writing a number for everything else.

**The frame loop is then a scenario like any other**, and the fake game drives it the way the real game
does: its own loop calls `render`, which is what `--no-frame-loop` turns off and what every shipped run
has on.

## Consequences

**What has to exist.**

- A `Game` method answering with both — the function and its argument, twice — and `render` doing the
  transmute where it does the call. `__thiscall` with one argument is `fastcall` with nothing on the
  stack, which is the note `run_calc_chain` already carries.
- `Originals` grows the two, and the frame loop's own entry point becomes reachable: a game with
  `own_frame_loop` on calls `render` per frame instead of the draw and update hooks.
- The fake game answers both. Its `present` is where a scenario counts a frame handed over — which is
  what the sim's `presented` already is — and its `play_sounds` is nothing, a laid-out game having no
  sound system.
- The pacing scenarios move to `orb/tests`, and `pacing/mod.rs`'s 414 lines go: the display a scenario
  declares stays, the loop it composed does not.

**What it costs.** Two vtable calls per frame fewer, and two pairs of relaxed loads in their place: the
four words are asked for once at the attach and kept beside the chain targets, which is also what lets
`attach_to` fill them with a laid-out game's own functions. And the transmute moves out of `th06` into
`lib.rs`, which is where the other two transmutes of the game's own functions already are.

**What it does not buy.** The host is still the simulator's: the wake jitter and the compositor's spikes
are drawn from a seeded stream, and what a scenario asserts is the *rate* rather than a turn to the
microsecond — see `orb-sim/src/display.rs`. Nothing here makes a laid-out game a machine, and the
measurements in `DONE.md` stay what says the pacing works on one.

**What it rules out.** The same thing 0001 does: a branch inside `Th06` that only a test takes. Two
alternatives were weighed and are rejected for that reason or a worse one.

- **Hooking the two**, so orb calls trampolines. Two more patched prologues in the shipped DLL, for
  nothing production wants, on a path that runs every frame.
- **A second `Game` for the fake game.** It would answer these two and everything else, which loses the
  one place the frame loop touches `Th06` — `prepare_frame`, the viewport and the background clear the
  game's own options ask for — and puts two implementations of a 2300-line trait in the tree to drift
  apart. 0001 rejected a second `Game` for the whole suite; there is no reason the frame loop should be
  the exception.

**What the moved scenarios then assert**, which is two things and not one. The rate from the clock, a
second at a time, as they did: what share of the seconds were sixty frames a second within half a
frame, from a few seconds in, over several seeds. And orb's own `frame:` line agreeing with it — the
count of frames, the interval, how many were shown late, and the histogram of gaps in refreshes — since
that line is what somebody reading a real run's log has to be able to believe.

**What it unblocks beyond the loop.** `log_deferral` is a scenario about `render` as much as about the
log: what the pacing writes about itself is held and written on the far side of the flush, where what is
left of the turn is slack. Driven through a copy of the loop, it says nothing about where the real one
drains.

**What the built shape does differently.** Four things, each found in the doing.

- **The two are statics, not a call per frame.** The plan had `render` asking the game for four words
  every frame. A game that is not a real process is `Th06` — 0001 rules out a second `Game` — so asking
  it would get 0x00431270 and 0x00420b50 whatever else was true, and the answer has to be *overridable*
  at the attach rather than asked for later. So `attach` stores what `Game::frame_calls` answered and
  `attach_to` stores a laid-out game's own, which is the shape `RUN_CALC_CHAIN_TARGET` beside them
  already had.
- **`Originals` grew three.** The game's own `Render` is the third, because three of the four ways out of
  `render` hand the frame back to it — and a scenario that drives one of those ways out with nothing
  there to hand it back to calls a null pointer.
- **Everything the moved scenarios read of the pacing, they read out of the log.** The harness held its
  own `Pacing` and asked it for the allowance. orb holds the frame loop's, and nothing outside orb does,
  so the allowance is read off the `frame:` line that says what the compositor is being given — which is
  the same line somebody looking into a stutter has, and the half this ADR called the cheaper one.
  It costs the scenarios one thing: those lines are written once per `profile::INTERVAL` frames and
  drained on the far side of the *next* frame's flush, so a scenario that wants one waits for it.
- **Three of the things named uncovered above still are.** A chain target that is null: `attach` and
  `attach_to` both fill those statics and nothing outside orb can empty them, so that one of the four ways
  out has no scenario. `prepare_frame` before the wait, and the present not being waited on: both are
  moments *inside* a frame, and what a scenario can read of one is the spans `frame.rs` writes out of the
  marks — which it writes only for a frame that came out off the cadence. What the marks are is held
  otherwise: the cadence every pacing scenario asserts is read from `presented`, and the lag `pacing_budget`
  reads off the report line is the game's own work as `waited`..`presented` measured it. The other three
  ways out, the update before the draw, the sounds between them and the chain's two exits are
  `frame_loop.rs`.

The status line is not covered either. It is the same numbers said again in the one place a scenario
cannot reach, and reaching it is a seam of its own — see *Draw a frame in a test and say what is on it*
in [TODO.md](../../TODO.md).
