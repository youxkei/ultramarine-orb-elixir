# 18. th07's script clock is the first of four main contexts

**Status:** accepted and built. `SCRIPT_FRAMES` in `crates/orb-core/src/game/th07/mod.rs` is
`+0x9545fc` of the enemy manager, `Th07::read_state` answers `State::script_frames` off it, and the state
line of a laid-out run carries it — `crates/orb-e2e/src/th07.rs`.

It answers one of the two questions [0004](0004-th07-is-a-second-game-chosen-at-the-attach.md) parked:
*whether the midstage table's shape — script frame numbers per stage — holds where stages are not one
script on one clock.* It holds, on the first context's clock, and the table's shape does not change.

## Context

**th06 has one timeline and th07 has four scripts.** `State::script_frames` exists because a midstage
boundary has to be expressed in a clock that runs the same way however the player is doing and on every
difficulty — not in the stage's own frames, which a dialogue and a menu stop. In th06 that clock is the
enemy manager's `TIMELINE_TIME_CURRENT`: one timeline, spawning the stage's enemies, one number.

th07 has no timeline. Its enemy manager at 0x9a9b00 keeps **four main contexts** of 0x10 bytes from
`+0x9545f4`, each a timer with the instruction it is up to after it — `previous`, `current_f`, `current`,
`instruction` — and 0x41f6f0 runs each of them: while `current` has reached the time word at the head of the
instruction it runs that instruction and steps the pointer on by the size at the instruction's `+0x6`, then
copies `current` into `previous` and steps the timer through the supervisor's own helper at 0x43958d. The
instruction set is its own and small, thirteen opcodes through the table at 0x41fd2f, whose first case is the
spawn that copies 0x13d2 dwords of template into one of the manager's 0x1e0 enemy slots and starts that
enemy's ECL. The stage's spawns come out of `data/ecldataN.ecl`, loaded by [`G_STAGE`] into the holder at
0x1347938, whose subs the manager's update assigns to the four contexts at 0x420762.

So there are four clocks where th06 has one, all four stepped once per frame, and the four differ only in
what each is running and whether it is running at all — a context whose instruction has a negative time word
is finished and 0x41f6f0 returns from it at once.

## Decision

**`script_frames` is the first context's `current`**, at `+0x9545fc`.

The four are not four clocks of a stage; they are four readers of one file, and the first is the one the
stage's own spawns come out of. The manager is re-registered for every stage, by the load at 0x42f11d, so the
clock starts again with the stage — which is what a per-stage boundary number needs.

The midstage table keeps its shape: frame numbers per stage, as `Game::midstage_table` already has them for
th06. Nothing about the table changes for this game.

## What follows

**The boundaries themselves are still a measurement, and it is a launch's.** *Building the midstage table* in
`SPEC.md` is a run with `--chapter-tuning` reading the boundaries out of what orb writes, and that run is
what says whether this clock behaves as the table needs — in particular whether it advances through a
dialogue, which reading cannot settle: the manager's update runs the contexts on every frame it reaches, and
which frames those are is the same question the stage's own clock has beside it in
`docs/todo/th07-the-run.md`.

If it turns out that the boundaries want the other three contexts as well — a stage whose spawns come out of
more than the first — what changes is this decision and not the table: the trait's method answers one number
per stage, and which number that is, is this file's.

## Alternatives, and why not

**The counter at 0x62f898.** It goes up once per frame in the manager's update at 0x4206f0, and only on
frames where no dialogue is running, which is the shape `script_frames` wants. But nothing resets it, an ECL
instruction adds 0x708 to it at 0x4157dc, and 0x448xxx divides it by a constant for something's animation
phase. A clock the script can jump and nothing restarts cannot carry per-stage boundaries.

**The stage's own frames**, `[gm+0x95e8]`. That is `State::stage_frames`, which orb already answers: it counts
the stage's frames rather than the script's, and the two are different clocks — which is why the trait carries
both.

**The sum, or the maximum, of the four contexts' clocks.** Both are numbers with no meaning in the game: a
boundary read out of one run would not be a boundary in another, since which contexts are running depends on
what the stage has reached.
