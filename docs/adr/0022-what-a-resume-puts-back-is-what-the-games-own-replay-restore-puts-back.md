# 22. What a resume puts back is what the game's own replay restore puts back

**Status:** accepted and built. `Game::run_state` and `Game::set_run_state` in
`crates/orb-core/src/game/mod.rs` are the pair, `th07::Numbers` in
`crates/orb-core/src/game/th07/resume.rs` is 妖々夢's answer, and
`the_number_th07s_shot_is_scaled_by.rs` in `crates/orb-e2e/src/` is the rule for the field that
established it. `Game::the_runs_own_bytes` is the instrument.

## Context

A resume does not put a world back. It asks the game to **build** the stage, writes the run's numbers over
what the build filled in, seeds the generator, and plays the recorded buttons in. So a chapter is
reproducible exactly as far as the world is a function of three things: the seed, the buttons, and the
numbers written back. Anything else the world depends on is a thing the replay gets a *build's* value for
where the run had its own.

**And a run walks into most of its stages.** 妖々夢's `GameManager::AddedCallback` is one function reached
two ways — 0x42e894 compares the scene asked for against the rebuild scene — and the block it jumps over is
the one that fills a run's numbers in. A resume enters every stage on the *filled* side, whichever way the
run it is reproducing entered it. So every field that block writes, and that `set_run_state` does not put
back, is a field where the two differ.

**Which is not a difference any reproduction line shows.** Five pairs of launches of 妖々夢 agreed on every
field a frame's line carries — the word, the player, the generator, the numbers drawn, the items in the air,
the score, the cherry, the updates taken — for hundreds of frames, and then collected different items. The
fields a line carries are the ones somebody thought to name, so a difference in a field nobody named reads
as no difference at all.

**The field turned out to be one the game itself saves.** `[0x62f8a4]` is written into a stage's replay
record at 0x4432c9 and put back from it at 0x4437ea — one instruction after the point items, which orb had
always carried. It is a rate the player's own per-frame code multiplies by, and the same 2面 held `0x20`
entered by a transition against `0x10` built by a resume. See `docs/th07/asm.md`.

## Decision

**The set a game puts back is the set that game's own replay restore puts back**, field for field, and
`Game::run_state` is where a game says what that set is.

A game with replays has already enumerated it: its replay can be watched from a stage past the first, so
whatever it writes into a stage's record and reads back out is exactly what a stage of a run in progress has
to be restarted from. That list is a **reading of the game**, not a judgement — so a field is carried
because the record carries it, and a field is left out because the record leaves it out and the reason is
written down.

**A field is not left out because nothing seems to write it.** What said `[0x62f8a4]` was a constant was a
grep for the address as an instruction operand; what writes it is a script, through the address the
dispatcher at 0x40ecf1 hands out. See the measurement rule in `CLAUDE.md`.

**And the instrument for a game whose record cannot be read is a dump of both entries.**
`Game::the_runs_own_bytes` says the run's own bytes at a stage's start, and a launch that walks into the
stage held against a launch that resumes into it names the offset that differs. An offset is a thing the
disassembly can be asked about; a guess about which field matters is not.

### The alternative that was built and taken back

When the parting would not explain itself, the file was made to hold a stage's *start* in place of the
chapters inside a stage the run had walked into. It worked — the start replays exactly — and it was wrong.
What it cost was the thing orb exists for: a session interrupted at 4面's boss came back at 4面's first
frame, and the whole stage was played again. It also read as an answer, which is worse than reading as a
question: an unexplained difference had been turned into a documented limitation, in `SPEC.md` and in an ADR
of its own, and the reason to keep looking had been written out of the tree.

So the rule is the other way round. **A replay that does not reproduce is a field that has not been found**,
and narrowing what the file holds is not a fix for it.

## What follows

**A new game's resume starts by reading its replay restore**, not by writing a struct and adding to it when
a run comes back wrong. The fields fall out of one function, and each of them has a name in the record.

**A game orb has not read that from answers nothing.** `Game::run_state` is a game's own, so a game with no
reading behind it keeps no runs — which is ADR 0004's rule, and this is what makes it worth keeping: a
resume that carries most of a run's numbers is a resume that parts from itself somewhere no line names.

**And the fake copies the difference.** 妖々夢's build fills the shot's own number out of a per-shot table,
so `Fake::builds_the_stage` fills it and its transition does not — without that, an e2e test asserting the
field is put back passes on a fake where nothing ever overwrote it. Which is ADR 0008: every fidelity fix is
red first, and this one was red only after the fake was made to lose the field the way the game does.
