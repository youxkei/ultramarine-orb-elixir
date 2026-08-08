# 7. The spin's `pause` is behind the seam, so that a simulated spin costs simulated time

**The file the measurements below are timed against is `crates/orb-e2e/src/pacing.rs` now**, moved by [0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md); it was `scenario_pacing.rs` when they were taken and they say so.

**Status:** accepted and built. `frame::wait_until` ends in `clock::spin_once()` rather than
`std::hint::spin_loop()`; the seam carries a `Win::spin_once` that is the `pause` instruction on a real
host and `orb_sim::PAUSE_TICKS` of the counter under a simulated one.

It follows [0006](0006-the-frame-loop-waits-on-a-high-resolution-timer.md), which established that the
spin has to stay 1500µs, and **overturns two things that document says**: that `orb_sim::READ_TICKS` is
what carries the spin to its deadline, and that `Clock::set_read_cost` is the only lever left on the
suite's time. Neither is true now.

## Context

**The spin is the whole of what the suite spends its time on, and it is not a Win32 call.** `wait_until`
waits on the timer to `SPIN_US = 1500µs` before the frame's deadline and spins the rest. On a real host
that spin is a `pause` instruction and a `QueryPerformanceCounter` per turn — no kernel, no call, about
twenty cycles for the `pause`. Under `orb-sim` the counter is a number a test moves, and the only thing
moving it inside the spin was the read cost, one tick — 0.1µs — at a time. **1500µs at 0.1µs a turn is
fifteen thousand real iterations per simulated frame**, and that was 76.6 of `scenario_pacing.rs`'s 76.6
seconds and most of the suite's 95.

**The obvious lever was measured and rejected in 0006.** `Clock::set_read_cost` makes every counter read
cost more, which reaches the spin — but it also reaches every read *inside a span the game is measuring*,
and `frame::budget`'s `READS_US = 10` is an allowance derived from a read costing one tick. Measured
there: at 1µs a read, a span the game spent 4000µs in came back as 4029, and at 10µs as 4203, with a
scenario failing for exactly that reason. So the knob works and corrupts the thing the scenarios are
about.

**`pause` appears in exactly one place in orb**, which is the observation this rests on. Nothing else
spins. So a cost attached to `pause` — rather than to reading the clock — reaches the spin and reaches
nothing else at all.

## Decision

1. **The `pause` goes behind the seam**, as `Win::spin_once`, with a facade at
   `orb_api::clock::spin_once`. On a real host it is `std::hint::spin_loop()` and compiles to the single
   `f3 90` it always did — in a build with no `sim` feature the facade has no branch in it, which is the
   property `mem::read` is built on.
2. **The simulated host charges `PAUSE_TICKS = 10` ticks for it**, one microsecond, so a simulated
   frame's spin is 150 turns where it was 15,000.
3. **The number is the coarsest step that leaves every scenario's answer unchanged**, and that is the
   rule for choosing it rather than a model of anything. A real `pause` is a hundredth of a microsecond;
   charging a hundred times that is deliberate. What it costs is where a frame lands: the loop reads the
   counter each turn and stops once past the deadline, so a landing falls in `[0, PAUSE_TICKS)` past it
   rather than within a tick.
4. **The step above it is recorded because it fails**, and that record is the point. At 100 ticks —
   10µs — `scenario_pacing.rs` is 7.1 seconds instead of 17.5, and
   `holds::the_whole_multiple_with_no_room_on_a_restless_desktop` fails: seed 3, a second at 46.83 frames
   against a bound of 47. That is the scenario with a 250ms stage load on a display whose compositor has
   no room, so the one with least to give. **The answer to that failure is not to loosen the bound**, and
   a note saying so lives beside the constant, because a suite whose assertions move to make it faster
   has stopped being a check.
5. **`Clock::set_pause_cost` exists for a scenario that needs a finer landing than the default**, in the
   shape of `Display::as_a_metronome` — declared by the scenario that needs it, not tuned globally.
   Nothing uses it yet.

## What was weighed and rejected

- **`Clock::set_read_cost`, which was already there.** See the context: it buys the same time and
  corrupts the measured spans while doing it. It stays for what it is documented as — what a read of a
  real counter costs — and the reason to reach for this instead is that `pause` is only in the spin.
- **Leaving the spin as a bare instruction and accepting the 76.6 seconds.** Which is what 0006
  concluded, having no better lever. The cost of that conclusion is a suite nobody runs as often as they
  would run a fast one, and the thing it was protecting — the resolution the pacing is measured at — is
  not what this gives up: every span the game measures is exactly as accurate as before, and the only
  number that moves is a frame's landing, by up to a microsecond.
- **Modelling a faithful `pause` of a hundredth of a microsecond.** That is *slower* than what it
  replaced, since the counter read already costs ten times it. A faithful cost here buys nothing and
  charges for the privilege.
- **Reading the seam as "the Win32 calls", which would have made this an exception to it.** It is not
  that, and the first draft of this document treated the widening as a cost to be justified. The seam is
  orb's side of *the host* — `orb-api`'s own title says so — and what decides membership is the rule
  `READ_TICKS` was already there for: whether it is the host doing something to orb that no test could
  otherwise decide. *How long a spin takes* is exactly that. A simulated host that cannot charge for a
  `pause` has to pretend spinning is free, which is the same false kindness as a counter that does not
  move when it is read.

  So there is no exception here and nothing was traded. What did have to change is the two places that
  wrote the seam down as Windows-only — `orb-api`'s module doc and `SPEC.md`'s section — because a
  definition that makes a correct member look like a special case is the part that was wrong.

## What follows from it

- **`scenario_pacing.rs` is 17.5 seconds and the suite 30.3**, from 76.6 and 95.0, with all 297 passing
  and no assertion touched. That undoes 0006's own regression — the exact wait had taken the file from
  54.1 to 76.6 — and then some.
- **`READ_TICKS` stops being load-bearing**, which is the second thing 0006 said that this changes. Its
  documented reason was that the spin would otherwise never reach its deadline; the spin now arrives on
  the `pause` alone. It stays, because a real read does cost time, but nothing proves it any more and its
  comment says so.
- **A simulated frame lands up to a microsecond past its deadline** where it used to land within a tick.
  Smaller than the wake delay already modelled on a flush's return (`USUAL_US = 100µs`) and than the
  0.0µs median with an 80.3µs worst that `scripts/spin-probe.c` measured on a real host, so the
  simulator has not become the kinder of the two.
- **The seam is written down as the host's rather than as Win32's**, in `orb-api`'s module doc and in
  `SPEC.md`'s section, both of which said Windows and now say the host. `Win::spin_once` is the only
  member that is not a call, and what admits it is the test that admits every other one — the host doing
  something to orb that no scenario could otherwise decide. Anything added beside it has to pass that
  test too, and being an instruction is not by itself an argument in either direction.
