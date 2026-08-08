# 6. The frame loop waits on a high-resolution timer, and the log is stamped off the performance counter

**The file the measurements below are timed against is `crates/orb-e2e/src/pacing.rs` now**, moved by [0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md); it was `scenario_pacing.rs` when they were taken and they say so.

**Status:** accepted and built. `frame::Pacing::wait_until` waits on a
`CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` waitable timer, made behind the seam on first use; nothing
asks `timeBeginPeriod` for anything and there is no `Sleep` left in the tree; every log line is
stamped off `QueryPerformanceCounter`; and a host that cannot make the timer is turned away by the
launcher before it starts anything and by the DLL on its first wait.

It changes one thing inside the frame loop [0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md)
built and overturns nothing of it: how the wait to a frame's own deadline is made. Everything that
document settled about what the loop is and which calls it hands over stands.

**[0007](0007-the-spins-pause-is-behind-the-seam.md) overturns two things said below**: that
`orb_sim::READ_TICKS` is what carries the spin to its deadline (decision 9), and that
`Clock::set_read_cost` is the only lever left on the suite's time (the last of *What was weighed and
rejected*). The spin's `pause` went behind the seam, which took `scenario_pacing.rs` from the 76.6
seconds this document leaves it at to 17.5.

**The measurement it waited on has been taken** — `scripts/wait-probe.c`, and *What was measured* at
the end is what it said. It moved one number and no decisions, and not in the direction expected:
`SPIN_US` stays 1500, because what the probe found is that 1500 was too small for the wait it was
covering and is right for the one replacing it. Two claims in this document were wrong before it ran
and are marked where they stood.

## Context

**The frame loop idles on purpose and does its work at the end of the turn.** The whole of the input lag
still worth winning is there: the work — read the keyboard, update, draw, hand over — takes about a
millisecond of a 16.6ms turn, so doing it where waiting for the blank leaves it reads the keyboard a
refresh and a half before the frame it appears in, and doing it at the end reads the keyboard just
before. `wait_for_slot` ends in exactly that:

```rust
self.wait_until(blank + cadence - self.ticks(self.prepare_us));
```

**Which makes the deadline a real one.** A wait that overshoots hands the frame over after its blank has
gone, and the frame waits out another refresh — one lost, visibly. So the accuracy of the wait is the
accuracy of the cadence.

**`Sleep` cannot do it, and the spin is what covers the difference.** `Sleep` is accurate to the system
timer tick, which `timeBeginPeriod(1)` brings to about a millisecond and which is fifteen without it —
nearly two refreshes at 120Hz, and exactly the size of the stutter measured whenever the pacing fell back
to the clock. So `sleep_until` slept most of the way and spun the last `SPIN_US = 1500µs`.

**Where 1500 came from was only half written down.** Its own comment said why the last stretch is spun and
why the rest is not — `Sleep` cannot be trusted with it, and spinning the whole wait would take the share
the sound needs — but nothing said why the figure was 1500 rather than 1200 or 2500. A millisecond and a
half is `Sleep`'s millisecond with headroom, which is the reading that fits, and it was a reading and not a
measurement. That is what made it worth measuring rather than adjusting: the number going out was softer
than it looked, and it turned out to be **wrong in the direction nobody had guessed** — `Sleep` overshoots
by up to 2153µs on this host, which a 1500µs spin does not cover.

**What the spin costs, on both sides.** On the real host it is a core busy for a millisecond and a half
of every frame, and `SPIN_US`'s own comment is about that: spinning the whole wait would take the share
the sound and the rest of the system need. In the simulator it is worse and measurable — the counter
moves one tick per read *because* the spin needs it to, so 1500µs at 0.1µs a tick is **15,000 real
iterations a frame**, about 1.2ms of CPU per simulated frame. Measured before `a_test_per_rate!` split
the tables in `scenario_pacing.rs`: the file was 539.6 seconds of work in 126 seconds of wall clock
across sixteen cores, of which **121.6 seconds was one test** — and after the split it was 54.8 seconds,
with the spin the whole of what is left. This is the cost that made a smaller spin look like the prize;
the measurement took that off the table and the exact wait then put the file up to 76.6 seconds — see
*What follows from it*.

**And a second thing hung off `timeBeginPeriod` that has nothing to do with sleeping.** `GetTickCount`
is what stamped every log line, and it is updated at the system tick — so the millisecond orb asked
for was thought to be quietly also what made the log's stamps worth reading, where the simulator did
not work that way at all:

| | the real host | `orb-sim` |
| --- | --- | --- |
| `Win::ticks` | `GetTickCount`, following the system tick | the performance counter divided down, exact to the millisecond |

**Which was wrong about the real host, and the divergence was worse than this said.** Measured:
`GetTickCount` advances by 15 or 16ms *whether or not* `timeBeginPeriod(1)` is in force. The
millisecond never bought the stamp anything, so orb's log has never been able to say when anything
happened to better than a system tick while the simulator answered to the millisecond — which is the
one shape this repository's simulator is written to avoid, and it was already there before this
document proposed to remove the millisecond.

## Decision

1. **Wait on a waitable timer created with `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`** —
   `CreateWaitableTimerExW` once, then `SetWaitableTimer` with a relative due time and
   `WaitForSingleObject` per wait. It is not tied to the system tick, so it neither needs
   `timeBeginPeriod` nor changes anything outside this process. Measured: with no resolution in force
   it returns within a millisecond of where it was aimed, where a waitable timer made *without* the
   flag returns 15ms late at the same moment.
2. **The seam's `clock::sleep_millis(u32)` becomes `clock::wait(ticks: i64) -> bool`**, a wait in the
   counter's own units. Milliseconds are the granularity being left behind, so they are not in the
   signature — and the call this replaced rounded its wait down to whole milliseconds, which handed
   the spin up to a millisecond more than it was asked to cover: margin nobody counted on and that a
   wait of an exact number of milliseconds did not have. The timer handle stays behind the seam and is
   created on first use: nothing about a handle crosses, which is the rule the thread ids and the
   log's token already follow. `false` is the creation having failed.
3. **No path for anything below Windows 10 1803, and a host that cannot make the timer gets a dialog and
   no launch.** The high-resolution flag is that build's and older Windows is not a target — so a
   creation that fails is not a configuration to carry code for, it is a machine orb does not run on, and
   the thing to do about it is say so where somebody will read it and stop. Not carry on slowly: a launch
   that quietly paced worse than it says it does is the one outcome the whole of `frame.rs` exists to
   avoid.
4. **The dialog and the giving up are behind the seam**, in the four places anything behind it lives:
   `Win::message_box` and `Win::exit_process`, their facades in `orb_api::window` and the new
   `orb_api::process`, the real ones in `orb-api/src/real/window.rs` and `real/process.rs`, and `Sim`'s
   answers — which write down what was said and what code was asked for, and are read back by
   `Sim::dialogs` and `Sim::exited`. `orb-sim` writes them down instead of putting a modal window up: a
   suite that raised a real `MessageBoxW` would wait for a click that is never coming, and one that
   really exited would take the harness's child with it. Which is this seam's own test for whether a
   call belongs behind it: there is behaviour here no scenario could otherwise reach.
5. **Where the dialog is raised is decided by `DllMain`, not by taste.** `Pacing::configure` runs inside
   it — `DllMain` calls `attach`, which calls `configure` — and a `MessageBoxW` under the loader lock
   loads `comctl32` and `uxtheme` and pumps messages into a process that is not finished starting, which
   is the textbook deadlock. So the timer is **created on first use from the frame hook**, which is the
   game's main thread with a window and a pump, and `Pacing::no_timer` — the log line, the modal, the
   `ExitProcess` — is reached from there.
6. **And the launcher checks the same thing before it injects anything**, which is where the dialog will
   be seen in practice. `can_make_the_timer` in `launcher/src/main.rs` makes one of its own and gives it
   straight back, before the config is read or the settings are asked for; where it cannot, the launcher
   puts up a modal, prints a line and starts nothing — so the ordinary case for an unsupported host is a
   launcher that declines, with the game never started, rather than a game killed on its first frame.
   The DLL keeps 3 for the case the launcher was not the way in.
7. **Drop `timeBeginPeriod` and `timeEndPeriod`** once nothing waits through `Sleep`. They exist only to
   make `Sleep` accurate, and there is no `Sleep` left to make accurate. Six things went with them and
   were easy to miss because they are not in `frame.rs`: `Win::begin_period` and `Win::end_period`, their
   facades in `orb_api::clock`, `frame::release` and its call in `DllMain`'s `DLL_PROCESS_DETACH`, and on
   the simulator's side `Display::begin_period`, `end_period`, `refuse_period` and `period_held` — the
   third of which is what `pacing_no_timer.rs` drove. `Win32_Media` left `orb-api`'s feature list with
   them, and `Win32_System_SystemInformation` left with `GetTickCount`.

   Measured, and it is why this is a tidying rather than a trade: the timer measures the same with the
   millisecond in force as without it.
8. **Stamp the log off `QueryPerformanceCounter` rather than `GetTickCount`.** The reasoning this was
   given — that dropping the millisecond would otherwise take every log stamp back to the system tick —
   was wrong in a way that only strengthens it: the stamp was *already* at the system tick, 15ms, with
   the millisecond in force. So this is not a consequence of 7 at all. It is a repair of a divergence
   that was there all along, in which the simulator was kinder than the host and the scenarios that read
   a stamp were passing over a host that could not say when anything happened.

   Done above the seam rather than behind it: `clock::ticks` divides `counter()` by `frequency() / 1000`,
   `Win::ticks` is gone from the trait, and the simulator's own derivation stops being a divergence and
   becomes the same arithmetic. orb reads the counter every frame already, so the stamp costs nothing
   new.
9. **`SPIN_US` stays 1500**, and it is now a measured number rather than a reading of `Sleep`'s
   millisecond. The histogram sets it, and what the histogram says is that the timer's worst excursion
   over 4000 frame-scale waits is 1416µs while `Sleep`'s is 2153µs: 1500 covers the first with 84µs over
   it and never covered the second. **This document expected the opposite** — a figure of a few hundred
   microseconds, or none at all — and *What was measured* is why it is not that. The spin stays, its cost
   on a real core stays, and `scenario_pacing.rs` gets slower rather than faster for it.

   Which leaves `orb_sim::READ_TICKS` load-bearing after all: the tick a simulated counter read costs is
   what lets the spin reach its deadline, and there is still a spin. (**Overturned by
   [0007](0007-the-spins-pause-is-behind-the-seam.md)**: the spin's `pause` now costs simulated time and
   is what carries it there.)

## What was weighed and rejected

- **`NtDelayExecution` with `NtSetTimerResolution`.** Reaches 0.5ms, which is coarser than the documented
  timer above, and both are undocumented. Nothing to gain for the cost of depending on them.
- **Keeping `Sleep` and shrinking `SPIN_US` anyway.** The spin is what covers `Sleep`'s overshoot, so a
  smaller spin without a better wait trades dropped frames for CPU. It is the one change that makes the
  thing worse in the way somebody playing would notice — and the measurement puts a number on how much
  worse it already was: at the 8000µs aim `Sleep`'s p99 is 1577µs, so more than one wait in a hundred was
  already overshooting past the whole 1500µs spin.
- **Spinning the whole wait.** Either as the ordinary path or as what to do when the timer cannot be made.
  A core at 100% every frame; `SPIN_US`'s own comment already refuses it and the reason — the sound's
  share — is why. As the answer to a failed creation it is worse than refusing to run, because it is a
  launch that works and paces badly, which nobody reports as a fault against the right thing.
- **Keeping `Sleep` for the case the timer cannot be made.** It is the smallest possible fallback and it
  is still a second wait to keep working, a second path for a scenario to cover, and a configuration that
  ships. A host below 1803 is not a host orb runs on, and saying so once is cheaper than supporting it
  quietly forever.
- **Raising the dialog from `configure`, where the failure is found.** It is inside `DllMain`; see
  decision 5.
- **Waking the waiting thread at `THREAD_PRIORITY_TIME_CRITICAL`**, to shrink the spin by shrinking what
  it covers. The reason to expect something: what is left of the overshoot looks like the scheduler
  rather than the timer — the wake on a timer that is *already due* costs 520µs, and `DwmFlush` on this
  same machine returns within 100µs of a refresh 81% of the time, so a blocking wait here can be woken
  faster than this one is being.

  Measured, and it does nothing. Same run, the four frame-scale aims, ordinary priority against
  time-critical: medians 52/88/212/276µs against 61/88/220/260, p99 883/723/771/779 against
  958/773/720/828, worst 1014/1432/1025/1145 against 1160/1088/898/1165. The differences are smaller
  than the run-to-run spread — the ordinary column's own worst at the 4000µs aim was 933µs in one run and
  1432µs in another. So the priority is not what `DwmFlush` is getting either, and why that call is woken
  better is not established.

  Which leaves nothing to try that would make the *wait* lighter: the flag did the work it could, the
  timer resolution is no help and the priority is no help. What is left there is a trade rather than a
  win: `SPIN_US` at the p99 rather than the worst, about 1000µs, at a frame lost somewhere near one in a
  thousand.
- **Yielding inside the spin** — `SwitchToThread` or `Sleep(0)` in place of `pause`, so that the sound
  and the rest of the system get the core back for the last 1500µs of every turn. This is the obvious
  improvement, it is what `SPIN_US`'s own comment is worried about, and `scripts/spin-probe.c` measured
  it: 600 turns of 16600µs a variant, idle and then with a thread of load per processor.

  | spin | idle: worst landing | idle: cycles | loaded: landing median / p99 | loaded: what the load got done |
  | --- | --- | --- | --- | --- |
  | `pause`, as built | +80.3µs | 3387 Mcycles | +2370 / +12183µs | 272.9e9 |
  | `SwitchToThread` | +109.7µs | 3340 Mcycles | +3110 / +13883µs | 271.3e9 |
  | `Sleep(0)` | +72.5µs | 3341 Mcycles | +1444 / +14020µs | 263.0e9 |

  It fails at both things it was for. **Idle, it saves nothing** — the cycles are the same, because a
  yield with nobody waiting returns at once and the loop goes round as often. **Loaded, it makes the rest
  of the system worse off**, 0.6% and 3.6% less work done by the load than the plain spin managed, while
  the landings get worse: none of the six hundred were a refresh late with `pause`, one was with
  `SwitchToThread` and three were with `Sleep(0)`. So the yield gives up the deadline and buys nobody
  anything, and it would have cost a seam function of its own to do it.

  That load — a busy thread per processor at the same priority — is harsher than a game, which is why the
  idle rows are the ones to read for orb's own case, and they say there is nothing here. One thing worth
  keeping from the loaded rows: with every core busy the frame loop cannot hold its deadline at all,
  whatever the spin does, being late by 1.4 to 3.1ms in the median.
- **`MONITORX`/`MWAITX`, or `TPAUSE`/`UMWAIT`** — parking the core in a low-power state until a TSC
  deadline, which is what "wait without spending a core" would actually be. The AMD pair is on this
  machine (`spin-probe.c` reports it out of CPUID) and executes from user mode; Intel's is the other one.
  Not taken, on what it buys: a core parked that way is still the thread the OS thinks is running, so the
  sound gets nothing back and only the power is saved — 3.4 Gcycles per ten seconds of frames, nothing on
  a desktop and something on a laptop. Against that, inline assembly in a game's DLL, a CPUID branch, a
  second path for Intel, a seam function, and no scenario able to reach any of it.
- **Making the simulator's counter coarser instead.** `Clock::set_read_cost` exists for this and nothing
  uses it. Measured against the 54.8-second baseline this had before the wait became exact: at 1µs a read
  `scenario_pacing.rs` went from 54.8 to **10.1 seconds**, at 10µs to
  **5.4**, with exactly one scenario failing either way and failing for the right reason —
  `budget`'s `READS_US = 10` is the allowance for how much the counter reads *inside* a measured span add
  to it, and it is derived from a read costing one tick. At 1µs a span the game spent 4000µs in reads
  back as 4029, at 10µs as 4203.

  Rejected as the *answer* on the grounds that it buys the tests' speed by giving up the resolution the
  pacing is measured at, where the timer buys the same speed by making the real thing better. **Half of
  that is now known to be false**: the timer buys no speed at all, since the spin it was expected to
  shrink is staying — and the exact wait made the file slower. So this is not the alternative to anything
  any more: it is the only lever left on the file's 76.6 seconds, still available, and still with the same
  condition attached: if it is ever taken, `READS_US` has to be derived from the read cost rather than
  written as a literal. (**Overturned by [0007](0007-the-spins-pause-is-behind-the-seam.md)**: charging
  the spin's `pause` instead reaches the spin without touching a measured span, and took the file to 17.5
  seconds. This one is still there and still unused.)

## What follows from it

- **The wait's worst excursion drops from over the spin to inside it**, which is the whole of what
  changes about a frame's chances of making its blank. A wait aimed at the deadline less `SPIN_US` lands
  *on* the deadline whenever it overshoots by less than the spin; `Sleep`'s p99 at the 8000µs aim is
  1577µs, so more than one wait in a hundred overshot past the whole spin, and the timer does not do it in
  4000 waits. What that was worth is in
  `scripts/background-flush-probe.c`'s numbers beside `frame::Pacing::grid`: a frame handed over 1000µs
  before a blank made it 54 times in 60 and one handed over 2000µs before made it 60 times in 60, so
  several hundred microseconds of lateness is a refresh at stake and not a rounding error.
- **The spin does not shrink, and `scenario_pacing.rs` gets 40% slower.** This document said 1500 → 200µs
  would be 7.5× fewer iterations a frame and would put the file near 8 seconds. It went the other way:
  measured on this machine with the file run on its own, **54.1 seconds before and 76.6 after**.

  The reason is arithmetic and it is the simulator's alone. The call this replaced rounded its wait down
  to whole milliseconds, so under a simulated `Sleep` that waits *exactly* as long as it is asked the
  loop came back about 1000µs from the deadline and spun that; the wait is exact now, so it comes back
  at `SPIN_US` and spins all 1500. Half again the iterations, which is the 1.41 measured.

  On a real host it is the other way about, because a real wait overshoots: the old one landed roughly
  `1500 ± 500`µs from the deadline once `Sleep`'s own 450µs is set against the rounding, and the timer
  lands 260µs past where it was aimed, so the spin averages nearer 1240µs than 1500. The busy core gets
  slightly *less* busy and stops handing frames over late. What got worse is only the suite, and only
  because the simulator does not model the overshoot — which it should not: a modelled overshoot makes
  every assertion about a wait a statement about the overshoot.

  So the suite's 76.6 seconds now have exactly one lever, the read cost above, and it is worth 40% more
  than when it was measured. (**Overturned by [0007](0007-the-spins-pause-is-behind-the-seam.md)**, which
  found a second one and took the file to 17.5 seconds — below where it stood before this document.)
- **`pacing_no_timer.rs` is about a call that no longer exists**, and it went where it had to. It was
  `Pacing::configure` against a host that refuses the millisecond timer; there is no millisecond timer to
  refuse, so it is now the scenario over decisions 3 to 5 — a simulated host that refuses the creation,
  and orb writing the line, putting up the modal and ending the process rather than pacing anyway, with a
  second scenario asserting that the frame's turn is not waited out after that. Which is only writable
  because those two are behind the seam: the same file, asking the new question.
- **`configure` no longer asks the host for anything**, and the line about coarse waits is gone with the
  question it answered. What a launch's log says about its waiting is now only what the pacing lines say,
  and the span they used to call `sleep` is called `wait`.
- **A failed creation ends the launch rather than degrading it**, and that is what *no fallback* was given
  to mean. There is no second wait in the tree, no `Sleep` kept as a spare, and no slow path to
  accidentally ship on: the dialog says the host cannot do what orb needs and the process stops. Which
  also means the failure is loud in the one way it has to be — a launch that paced badly while its log
  said it was pacing well is the thing `frame.rs` is written against, and it is exactly what a quiet
  fallback would produce.
- **It costs a `MessageBoxW` and an `ExitProcess` in a game's process, and neither belongs in `DllMain`.**
  Hence the timer being made from the frame hook rather than from `configure`. A `panic!` would not do
  instead: the crate aborts on panic and the crash handler would write a `module+offset` line, which says
  a fault in orb where what happened is a host that is not supported.
- **Nothing about waiting for a blank changes.** `DwmFlush` is still the wait that anchors a frame on a
  real refresh, and it is still not spun on — see
  [0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) for what the loop is made of.
  What changes is only the wait to the frame's own deadline inside the turn.
- **Log stamps become exact to the millisecond on the real host**, which they never were, and which is
  what makes `log_deferral`'s reading of two stamps mean the same thing on both hosts.

## What was measured

`scripts/wait-probe.c`, in the shape of `scripts/compositor-probe.c` and
`scripts/background-flush-probe.c` and built the same way orb is — `i686-w64-mingw32-gcc`, so the
creation it asks about is asked from a 32-bit process, which is what orb is. On this machine: **Windows
10.0 build 26200, an AMD Ryzen 7 9800X3D, the counter at 10MHz**, 1000 waits aimed at each deadline.

**First, the thing that decides whether the rest of this document means anything.** A 32-bit process
*can* create the timer here, so decisions 1, 2, 7, 8 and 9 are built rather than moot.

**What each kind of wait returns at**, as return minus deadline, over the deadlines the frame loop
actually asks for:

| aimed at | 2000µs | 4000µs | 8000µs | 16600µs |
| --- | --- | --- | --- | --- |
| the timer with the flag, p99 | +938µs | +698µs | +828µs | +781µs |
| the timer with the flag, worst of 1000 | +998µs | +933µs | +1284µs | +1416µs |
| the same with `timeBeginPeriod(1)` in force, worst | +1010µs | +1534µs | +922µs | +1285µs |
| `Sleep` under `timeBeginPeriod(1)`, p99 | +1585µs | +1677µs | +1577µs | +924µs |
| `Sleep` under `timeBeginPeriod(1)`, worst of 1000 | +2086µs | +2011µs | +2153µs | +1644µs |
| a waitable timer **without** the flag, median | +13584µs | +11567µs | +7576µs | +14652µs |

Four things in that:

1. **The flag is what does it.** The last row is the same call at the same moment without
   `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` and it lands 15ms late every time, which is the system tick
   with nothing asked of it. So the accuracy in the rows above it is the flag's own and not something the
   host was doing anyway.
2. **`timeBeginPeriod` adds nothing to it**, which is what makes decision 7 a tidying. The two timer
   worsts straddle each other across the four columns; there is no column where the millisecond helps.
3. **`SPIN_US = 1500` covers the timer and did not cover `Sleep`.** 1416 against 2153 is the whole of the
   case for the change, and it is not the case this document was written on. Three runs of the same 1000
   waits put the timer's worst at 1216, 1416 and 1432µs, so the margin is 68µs at the worst seen — thin,
   and the first thing to suspect if frames are ever late here.
4. **The wait never returns early.** The least overshoot in every column is positive — +8.0µs at the
   2000µs aim, +12.1µs at 16600 — so `wait_until`'s loop issues one wait and then spins, and the short
   aims below never happen in a frame. Which matters, because they are the worst the timer does: aimed at
   500µs it overshot by up to 1996µs, and aimed at 200µs it never returned in less than 391µs at all.

**What `GetTickCount` advances by**, which is what every log line was stamped with: **15 to 16ms, with
`timeBeginPeriod(1)` in force and without it alike** — 13 changes in 200ms either way. This is the
measurement that says decision 8 is not a consequence of decision 7, and it contradicts what this
document assumed when it was written.

**What the wait costs to make**, since `SetWaitableTimer` and `WaitForSingleObject` are two calls a frame
where `Sleep` was one, inside the turn the pacing is accounting for: `SetWaitableTimer` **0.322µs**
against `Sleep(0)`'s 0.190µs, over 20,000 each. A tenth of a microsecond of a 16,600µs turn, so the
handover budget does not notice it.

And one number that is neither a call cost nor a wait: **the wake on a timer that is already due takes
519.9µs.** It is here because it is the floor the short aims run into, and it is the same ~0.5ms that
shows up as a mode in the 500µs and 1000µs histograms. Why it is there is not established — a plausible
mechanism is not a finding — and nothing in orb depends on it, since no wait it makes is that short.
