# 3. The frame loop's scenarios are one file, and what judges a rate is functions in it

**Status:** accepted and built. `orb/tests/pacing.rs` is 2249 lines: the functions that judge a rate at
its top level, and under them twelve sections holding the 46 scenarios about orb's own frame loop. The
ten `pacing_*.rs`, `frame_loop.rs`, `log_deferral.rs` and `orb/tests/pacing/mod.rs` are gone; `Fake` has
the launch, the hand-overs, the refresh period and the wait for a line that those scenarios used to reach
the host for; and `frame::LOGIC_HZ` is `pub`. What the built shape does differently from the decision
below is at the end.

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
`assert_never_settles_at_sixty`, 77 lines of them, dead in the harness for the whole of 0002's work and
caught by hand as it landed rather than by anything that could say so. A module nothing can see the dead
ends of is a module that keeps them.

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

**What it costs.** One file of 2249 lines, and `cargo test --test pacing_blanks` becomes a filter
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

**What the built shape does differently.** Six things, each found in the doing. Nothing here changed
what is asserted: the count is 252 before and after, 46 of them in the one binary.

- **There were no dead assertions left to remove.** The four are in 0002's diff as deletions — they were
  found by hand while that work landed, so `orb/tests/pacing/mod.rs` never held them. What the allow cost
  is still what the context says: nothing in twelve binaries could have said they were dead.
- **`launched` and `launched_with` are one function.** `Fake::attach_watching_the_pacing(display, name,
  work)` takes the `Work` a scenario declares, and a section whose frames all cost the same says
  `Work::flat(WORK_US)` at the launch. Two names for one launch were two names for `Work::flat`.
- **The judgement reads the log's lines rather than the game.** `reports`, `reported`, `allowance_us` and
  `last_said` take `&[String]`; the three assertions take the microseconds, the seed and what orb last
  said as a `&str`. Which is what "values and nothing else" came to at the call sites.
- **The hand-overs come back in microseconds**, so `turns` is subtraction and `refreshes` is given the
  period as a number. `Clock::micros_for_ticks` is inside `Fake::handovers_us` and
  `Fake::refresh_period_us` now, and the judging imports nothing of `orb_sim` at all.
- **One spelling of the report line, and it is on the judging side.** `until_reported` became
  `Fake::frames_until_the_log_holds_another(A_REPORT)`: the fake game runs frames until one more line
  holds what it was handed, and what a report line is known by stays a constant beside `Reported`.
  `"us apart"` is `Pacing::report`'s alone, so that filter wants one needle where it used to want two.
- **`fake::the_run` replaced two of the copies and not six.** The other five are in the files this
  decision does not touch — `legacy_run`, `mode_question`, `mode_on_the_pad`, `the_run_read_back` and
  `pointdevice_run` — each of which starts a launch of its own. The duplication the context counts is two
  down rather than gone.
