# 2. The frame loop's two calls into the game are addresses it hands over

**Status:** accepted and built. `Game::frame_calls` answers with the two, `render` calls through them,
and a laid-out 紅魔郷 drives the frame loop's scenarios through `render` itself: the `pacing_*` and
`log_deferral` stopped being tests of `orb-core` against a copy of the loop, and `frame_loop.rs` covered
the loop's own shape. `orb-sim/tests/pacing/mod.rs` and its 414 lines are gone. What the built shape
does differently from the plan below is at the end of *Consequences*.

**[0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md) has overturned one
thing this says, and it is built**: `Originals` is `orb_core::runtime`'s along with the eleven hook bodies,
so the three calls below that *had already crossed out of `orb` into `orb-core`* are no longer the
exception this document calls them. Two of `Originals`' fields changed shape on the way — the game's own
`CreateWindowExA` and `CreateFileA` are declared there with `*mut c_void` where Windows says `HWND`,
`HMENU` and `HANDLE`, those being what they are — because the crate that names them may name no
`windows-sys`.

Where those scenarios are written is no longer a file apiece:
[0003](0003-the-frame-loops-scenarios-are-one-file.md) put them in one file, a section each, with what
judges a rate as functions in it, and [0005](0005-every-scenario-lives-in-orb-sims-tests.md) moved that
file to `orb-sim/tests/scenario_pacing.rs`. Nothing of what they assert changed with either, so the
paths and file names below are the shape this decision left and not the tree's.

It overturns one claim of [0001](0001-a-fake-th06-drives-orb-end-to-end.md): that a game laid out by
hand cannot drive `render`. The obstacle that document names is real — `Th06::present` and
`Th06::play_sounds` call the game's own code — and the conclusion drawn from it was not.

**And the argument reached eleven more calls since**, eight through `Originals` and three out of `orb-core`.

`Originals` now carries `create_window`, `create_file`, `stop_recording`, `create_game_window`,
`joystick_position`, `get_controller_input`, `save_replay` and `init_d3d_device`: the game's own
`CreateWindowExA`, which orb's rewrite of the window arguments calls through; its own `CreateFileA`, which
the score file's fork calls through; its own `ReplayManager::StopRecording`, which orb's hook over it calls
through where the record being finished off is a run's own rather than a replay's; its own
`GameWindow::Create`, which the hook that overrules the display setting calls through once that setting has
been written; its own `joyGetPosEx` as its import table held it, which the replacement of that entry calls
through on the reads it has no sample of its own to answer with; its own
`Controller::GetControllerInput`, the tail call inside the keyboard read that the hook exists to time and
does nothing else to; its own `ReplayManager::SaveReplay`, whose write a cleared run's hook refuses and
whose teardown it calls through; and its own `GameWindow::InitD3dDevice`, which orb gets in front of to
redirect the device's `Present` before anything is presented through it. A real launch reaches four of those
by patching the exe's import table and four by patching a prologue, and a laid-out game has neither — so
the same answer applies eight times over: it hands the functions over and calls the hooks itself.
`scenario_the_window.rs`'s eight scenarios drive the first and the fourth that way,
`scenario_the_score_file.rs`'s six the second, `scenario_moving_between_a_replays_stages.rs`'s eight the
third, `scenario_mode_on_a_winmm_pad.rs`'s three the fifth and the sixth,
`scenario_a_clear_on_demand.rs`'s five the seventh, and
`scenario_the_launch_before_its_device.rs`'s one the eighth.

**Two of the eight moved a gate, and that is the one thing handing a function over is not neutral about.**
A real launch decides whether to time the joystick's read and whether to refuse a replay write by
installing the hook or not — the first only at Verbose, the second only under `--clear`. A game that hands
its own function over has one call site whichever way the launch was configured, so the installation cannot
be the decision there: `BLOCK_REPLAY_SAVE` and `TIME_JOYSTICK` are set where `attach` decides to install,
and both begin `true` so that a real launch, which only ever reaches the hook because it was installed,
behaves exactly as it did. Without that, a laid-out launch nobody asked to block a save would have had one
blocked — which is the seam answering a question it was not asked.

**The other three are not hooks, and they are `Th06`'s rather than `Originals`'.** `Chain::Cut` at
0x41cde0, `SoundPlayer::StopBGM` at 0x430f80 and `Supervisor::PlayAudio` at 0x424b5d are calls *out* of
`orb-core` into the game — `cut_screen_shake` takes a shake still running at a stage move down through the
first, and a restore whose track has been replaced since the chapter was taken puts the sound down through
the second and starts it again through the third — so there is no trampoline to fill and nothing to call
through: what a laid-out game hands over is the address itself. Each is kept in a static behind the same
`cfg(any(test, feature = "sim"))` the laid-out image is behind, so the shipped DLL has the constant and no
atomic in the path, and the three are written by one `handed_over!` macro rather than three times over.
Which is the one place the list has crossed out of `orb` into `orb-core`, and the reason is the one this
document set: code is the one thing an address space laid out by hand cannot hold, and a scenario that
reached one of those calls would be jumping into memory nothing has mapped.
`scenario_moving_between_a_replays_stages.rs` drives the first and
`scenario_the_music_across_a_restore.rs` the other two.

Nothing about the reasoning below changes; what changes is that the list of calls handed over is not
closed, and the test for adding to it is the one this document set.

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
measurement on the machine was read off two things: the `fps` on the status line beside the game, and the
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
the clock, orb's own line is held against it, and the status line is its own piece of work.

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
measurements beside `frame::Pacing::grid` stay what says the pacing works on one.

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
cannot reach, and reaching it is a seam of its own.
