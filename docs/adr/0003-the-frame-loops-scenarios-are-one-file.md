# 3. The frame loop's scenarios are one file, and what judges a rate is functions in it

**Status:** accepted, not built. The plan at the end is what is left, and until it is done the frame
loop's scenarios are twelve files in `orb/tests` sharing two modules, and `pacing/mod.rs` is a module
that also starts launches and runs frames.

It follows [0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md), which moved those
scenarios into `orb/tests` and left them in the shape the harness they replaced had: a file apiece.

## Context

**A file apiece was a reason, and the reason has gone.** `orb-sim/tests/log_writes.rs` still records
it:

> One test to a file, because orb's log is process-global by design: the file handle, the level and the
> frame's thread are statics, and two tests in one binary would be writing each other's log. A file is
> a binary of its own, so each of these owns its process.

A file is no longer what owns the process. `fake::in_its_own_process` spawns the test binary again for
each `#[test]` — `--exact <name> --nocapture --test-threads=1` — so every scenario owns a process
wherever it lives, and the file boundary buys nothing. `pacing_rates.rs` had already noticed the other
half: "They can share a binary because the pacing is a value now — it was a page of statics, and one
process could only ever have paced one display."

**What the split costs, and one of them is a defect it hid.** Twelve of the seventeen files in
`orb/tests` declare `mod fake;` and `mod pacing;`, which is 1361 lines and 509 lines compiled into each
of them. And because `dead_code` is worked out per binary, both modules *need*
`#![allow(dead_code)]` — what one file does not use is another file's. That allow hid four dead
assertions: `assert_never_sixty`, `assert_settles_at_sixty`, `assert_holds_sixty` and
`assert_never_settles_at_sixty`, 77 lines of them, dead in the harness before the move and dead after
it, and they were nearly committed as part of 0002. A module nothing can see the dead ends of is a
module that keeps them.

The same shape put six identical copies of `the_run()` in six files — Normal, Reimu A, stage one,
five fields each — and had `pacing/mod.rs` re-declare `A_SECOND = 60`, which is `frame::LOGIC_HZ`
written a second time where `LOGIC_HZ` was private. Sixty is the game's own logic rate and orb is
injected into the game to hold it there; a scenario with its own idea of a second is a scenario whose
idea can drift from the one being paced.

**And `pacing/mod.rs` is doing two unrelated things.** What it is *for* is the reader's side of a run:
the tolerances a measured second is judged by, the arithmetic over the ticks the game handed frames
over at, and the parse of the `frame:` line orb writes about itself. None of that is anything the
injected DLL does — nobody in a real run parses orb's log, and nothing in production has `AT_SIXTY`.

What it also does is start launches (`launched`, `launched_with`, `the_run`), run frames
(`until_reported`), and reach the host directly (`orb_sim::Clock::micros_for_ticks`, and
`game.sim().display().compositor_period()` through the game). Starting a launch and running frames are
the fake game's, and the host is the fake game's too. A module that judges a rate and also knows how to
begin a run is a second place that starts a game.

## Decision

**One file holds every scenario about orb's own frame loop, and the judging is plain functions in it.**

- `orb/tests/pacing.rs`, with the ten `pacing_*.rs`, `frame_loop.rs` and `log_deferral.rs` in it: 46
  tests over one subject — the loop's shape, the rate it holds, and the log it writes about itself.
- The judgement is functions at that file's top level. No module, and so no `#![allow(dead_code)]`: a
  helper nothing calls reads as dead, which is the property that was missing.
- **Those functions take values and nothing else** — microseconds, the log's lines, and the failure
  message as a `&str`. Not `&Fake`, not the host. Which is what makes them the reader's side rather
  than a second game.
- Each former file becomes an inner `mod` with its own doc header, because the sections do not share
  their numbers: `WORK_US` is 700 in most of them, 1500 in two and 4000 in the budget's.
- Starting a launch, running frames and reading the host are the fake game's, and gain the names that
  say so: `Fake::attach_watching_the_pacing`, `fake::the_run`, `Fake::handovers_us`,
  `Fake::refresh_period_us`, `Fake::frames_until_the_log_holds_another`.
- `frame::LOGIC_HZ` becomes `pub`, so a second is the rate the game's timers assume and not a number
  written again.

## Consequences

**What it costs.** One file of about 1900 lines, and `cargo test --test pacing_blanks` becomes a filter
on a test name instead. The names gain their section — `blanks::a_120hz_display_gets_two_blanks…` —
which `in_its_own_process` hands to `--exact` unchanged, so nothing about the process-per-scenario
changes.

**What it buys.** Seventeen test binaries become six, and `fake/mod.rs` is compiled six times rather
than seventeen. The judgement has no `allow(dead_code)` over it, so the next helper that stops being
called says so. And it can be read on its own: every function is arithmetic over a slice, which is
also what would let it have tests of its own if it ever wants them.

**What stays as it is, deliberately.** `Fake::sim()` stays public and the scenarios go on setting the
host up and reading it back to assert — a display declared at the launch, a window moved behind, a
compositor told to slow down. That is a scenario talking to the host it declared, which is not the same
as the *judgement* reaching around the game; closing it was tried and is not wanted.

**What it rules out.**

- **The judgement as a shared module.** It would need `allow(dead_code)` back for the same reason and
  be compiled per binary again, and the whole of what it gained is not needing either.
- **The judgement inside `fake`.** One shared module instead of two, at the price of the game knowing
  what counts as sixty and how orb's log line is spelled. Sixty is the game's; the tolerance around it
  is not, and neither is the log's format.

## Plan

1. The judgement made pure where it stands: every function taking values, `orb_sim` and `crate::fake`
   out of its imports, `launched`/`launched_with`/`the_run` and `until_reported` out of it. The
   corresponding additions to `Fake`, and `LOGIC_HZ` made `pub`.
2. `orb/tests/pacing.rs` with those functions at its top level and one section moved in, compiling
   before the next.
3. The rest of the sections, one at a time and compiling between: `blanks`, `by_clock`, `load`,
   `disagrees`, `budget`, `fractional`, `compose`, `converges`, `rates`, `holds`, then `frame_loop`
   and `log_deferral`.
4. The twelve files and the module directory deleted.
5. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. The count is 252 and
   must not move: nothing here changes what is asserted, only where it is written and what it is
   handed.

Step 1 is where the four dead assertions go, since they are dead either way and a module without the
allow will not compile with them in it.
