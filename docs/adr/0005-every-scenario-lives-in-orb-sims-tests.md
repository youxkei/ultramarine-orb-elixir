# 5. Every scenario lives in `orb-sim`'s `tests/`, and its file name says it is one

**Status:** accepted and built. `crates/orb-sim/tests/` holds all of them: the seven `scenario_*.rs` and
the `fake/` they share, moved out of `crates/orb/tests/`, beside the four tests of the log and of
`Pacing::configure` that were already there and keep the names of their subjects.
`crates/orb-sim/Cargo.toml` dev-depends on `orb`, `crates/orb/tests/` is gone, and no test's code
changed.

**[0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md) has moved them
again, and overturned the one reason this gives for the crate they landed in.** *`orb-sim` is where the
thing every one of them installs lives* is true of every consumer of a library and was not a reason to host
its users' tests; what it left unanswered is the cost this document records as its own — `cargo test -p
orb-sim` building `orb` — and the `#![allow(dead_code)]` over `fake/`, which was there only because those
were twenty-three binaries. Both are gone: the scenarios are `#[cfg(test)]` modules of
`crates/orb-e2e/src/`, `crates/orb-sim/Cargo.toml` dev-depends on `orb` no longer, and the `scenario_`
prefix went with the move. The four tests no game drives stayed here, which leaves two directories along a
boundary that is one: *a game drives it* against *no game does*.

**It overturns one claim of [0001](0001-a-fake-th06-drives-orb-end-to-end.md)**: that the 紅魔郷 which
plays the game's part cannot live in `orb-sim` because a game calling orb's hooks has to name `orb`, and
`orb-sim` is what `orb` is built on. Cargo takes that cycle. The statuses of
[0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md),
[0003](0003-the-frame-loops-scenarios-are-one-file.md) and
[0004](0004-th07-is-a-second-game-chosen-at-the-attach.md) describe the tree by the paths this moved,
and carry the new ones.

## Context

**Two `tests/` directories held two levels, and neither name said which.**

| | |
| --- | --- |
| `orb/tests/` | seven files, 62 tests: a game is laid out, orb is attached to it, keys are pressed, frames are run, and the game's memory, the game's records and orb's log are read back |
| `orb-sim/tests/` | four files, four tests: `orb_core::log` and `frame::Pacing::configure` driven directly, with no game anywhere in them |

`tests/` is cargo's word for an integration test and says nothing beyond that, so which level a file
holds was in neither its directory nor its name. `orb/tests/th07.rs` reads as a unit test of
`game::th07` and is a whole launch. `orb/tests/pacing.rs` — 46 scenarios over orb's own frame loop — and
`orb-sim/tests/pacing_no_timer.rs` — a wait or two against a host that cannot make the timer they are on
— share a word and differ by a level. The only place the distinction was written down at all is
`log_writes.rs`'s own header, in prose:

> **A unit test of the log, and not a scenario**, which is why it is still here and not among the runs in
> `orb/tests`.

**The reason for the split was a cycle, and there is no cycle.** [0001]'s *Where it landed* said:

> **The game is `orb/tests/fake`, not `orb-sim`.** The plan put it in the simulator, and it cannot go
> there: a game that calls orb's hooks has to name `orb`, and `orb-sim` is what `orb` is built *on* —
> `orb-core` depends on it.

Every fact in that is true and the conclusion does not follow. Cargo refuses a cycle in the *normal*
build graph; a cycle with a dev-dependency edge in it is one it resolves, because the libraries in it can
still be built in an order — and this workspace has had one the whole time, which
`crates/orb-core/Cargo.toml` records beside the dependency that closes it:

> `orb-sim`'s own `tests/` depend back on this crate, which cargo allows: that direction is a
> dev-dependency, so it is outside the normal build graph and the two do not form a cycle in it.

`orb-sim --dev--> orb --> orb-core --(sim)--> orb-sim` closes by the same rule. Measured before anything
was moved: the dev-dependency added and one test in `orb-sim/tests/` naming `orb::menu_ui::NORMAL`, which
built and passed.

**`orb-sim` is where the thing every one of them installs lives.** A scenario's first act is to put a
`Sim` in front of `orb_api` and its last is to drop it, and the four tests already there do the same. What
had them in two crates was the cycle and nothing else.

## Decision

**A test a game drives is a `scenario_*.rs` in `crates/orb-sim/tests/`. A test there that no game drives
is named for its subject, and not beginning `scenario_` is what says which it is.**

| | |
| --- | --- |
| `scenario_pacing.rs` | 46, orb's own frame loop |
| `scenario_pointdevice_run.rs` | 2, one 完全無欠モード run end to end |
| `scenario_legacy_run.rs` | 1, the same game answered the other way |
| `scenario_mode_question.rs` | 6, the question over the game's title menu |
| `scenario_mode_on_the_pad.rs` | 3, that question answered on a controller the game owns |
| `scenario_the_run_read_back.rs` | 4, `Th06::read_state` over a game that was played there |
| `scenario_th07.rs` | 1, a 妖々夢 that declines everything |
| `fake/` | the game's part: `mod.rs` is any launch's half, `th06.rs` and `th07.rs` each game's own |
| `log_writes.rs`, `log_off_thread.rs`, `log_overflow.rs`, `pacing_no_timer.rs` | 4, no game in any of them |

- `crates/orb-sim/Cargo.toml` gains `orb` in `[dev-dependencies]`, and reaches it through the `rlib`
  beside the DLL that `crates/orb/Cargo.toml` already declares for this.
- `crates/orb/tests/` is gone. `orb` keeps its own dev-dependencies for its own `#[cfg(test)]` modules,
  which need every one of them: `snapshot` names `orb_sim::Sim`, `chapter` names
  `th06::image::Image`, and `resume` builds a MessagePack map with `rmpv`.
- One binary per file, as before. `in_its_own_process` spawns the binary it is in, so a scenario owns a
  process wherever the file sits and nothing about the process-per-scenario changes.

## Consequences

**What it costs.** `cargo test --test pacing` is `--test scenario_pacing`, and the same for the other six.
`cargo test -p orb-sim` now builds the `orb` crate, so the simulator's own four tests wait on it. And
every path naming one of these files moves with it, in the documents and in the four earlier decisions.

**What it buys.** One directory, and `ls` of it answers the question that used to need `SPEC.md`: eleven
files beside the `fake/` they share, seven of them saying which level they are in the first nine
characters of their names. The pair that
collided is `scenario_pacing.rs` beside `pacing_no_timer.rs`, where the difference is now the thing you
read first rather than something to know. And the split that had no reason left in it is gone, rather than
documented.

**What it rules out.**

- **A `tests/scenario/` subdirectory.** Cargo auto-discovers `tests/*.rs` and `tests/*/main.rs`, so seven
  files under `tests/scenario/` would each need a `[[test]]` entry carrying its path — a manifest to keep
  in step with the directory, for the same thing a prefix says with nothing to keep in step.
- **Two letters rather than the word.** The arrangement is borrowed from another project of this author's,
  where the full-stack suite lives in the simulator crate and the layer is two letters in the file name
  (`lb.rs` for the backend alone, `lf.rs` for the whole runtime). `scenario` is this repository's own word
  for the same thing — `SPEC.md`, every decision here and `fake/mod.rs` all use it, and
  `log_writes.rs` draws the contrast in it — and `CLAUDE.md` asks for the code's own words unabbreviated.
- **Renaming and not moving.** It would have answered the names and left two `tests/` directories split
  along a boundary that is not a boundary: the same simulated Windows installed from both, and the reason
  they were apart disproved.

**What it does not change.** The seven files compiled and passed where they landed before any of them was
edited; what changed in them afterwards is doc comments that named the old paths, and two that named
`orb-sim/tests/mode.rs`, gone since `1540e14` drove the mode question from a game's own keyboard instead
of a `Pad` a test built. The count is 258 either side of the move — 62
scenarios, four tests over the simulated Windows with no game in them, and 192 in `#[cfg(test)]` modules.
