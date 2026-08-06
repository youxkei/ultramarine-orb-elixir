# 4. th07 is a second `Game` chosen at the attach, and it declines what has not been measured

**Status:** accepted, not built. Nothing of th07 is in the tree: `Game` has one implementation,
`orb/src/lib.rs` holds `static GAME: Th06 = Th06` and names it in 27 places, the launcher pins
`東方紅魔郷.exe` by md5, and the only game a scenario can attach to is a laid-out 紅魔郷. The plan at the
end is what is left.

It stands on three decisions. [0001](0001-a-fake-th06-drives-orb-end-to-end.md) put a game that plays the
game's part in front of orb and ruled out a second `Game` written for tests.
[0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) made the frame loop's two calls into
the game addresses the game hands over, and stored them in statics at the attach because a hook has
nothing but the ABI's arguments — which is the shape the choice of game takes here.
[0003](0003-the-frame-loops-scenarios-are-one-file.md) left the 46 scenarios about the frame loop judging
microseconds and log lines and nothing else, which is what makes them a second game's as well.

## Context

**The seam is real, and it is not the whole of what a second game needs.** `Game` is 62 methods of
addresses and offsets, and orb's own logic is written against it rather than against 紅魔郷: `Runtime`
holds `game: &'static dyn Game`, and `chapter.rs`, `tuning.rs`, `resume` and the frame loop take
`&dyn Game`. `attach_to(&'static dyn Game, …)` is already the entry a laid-out game comes in through. So
nothing about how orb works has to be reworked for a second game. Three things it hardcodes do.

**One static names the game 27 times.** `orb/src/lib.rs`:

```rust
/// The one game orb knows how to run inside.
static GAME: Th06 = Th06;
```

The hooks are plain `extern "fastcall"` functions with nothing but the ABI's arguments, so none of them
can be handed a game — they read the static. Which is exactly the problem 0002 had with `Th06::present`
and `Th06::play_sounds`, and it has the same answer: work it out once at the attach and keep it in a
static beside `RUN_CALC_CHAIN_TARGET`, `PLAY_SOUNDS` and `PRESENT`. What the attach already has to work it
out from is the exe's path — `orb_api::module::host_exe()` — and the PE it is reading `.data` out of
anyway.

**The launcher knows one exe and one version.** `GAME_EXE` is `東方紅魔郷.exe`, `GAME_EXE_MD5` is 1.02h's,
and `verify_game_exe` refuses anything else with "orb only supports 1.02h". So `scripts/run.sh` pointed at a
th07 directory stops before the game starts, which is the right thing to do and a message about the wrong
thing.

**The test rig is 紅魔郷's, twice over.** `orb-core/src/game/th06/image.rs` is 834 lines laying out th06's
own address space — `DATA 0x00476000..0x006e79fc` and the blocks around it — and the types it hands a
scenario are 紅魔郷's model of a game: `Scene`, `Supervising`, `FrontEnd`, `Screen`, `Playing`,
`Reproducing`. `orb/tests/fake/mod.rs` is 1400 lines playing that game's part, naming `Th06` in 24 places:
the supervisor's build-then-copy order, the title menu's item indices, the `catk` record, the shot-type
screen, the bits of its own input word.

**But the frame loop's 46 scenarios touch none of that.** They declare a display, start a launch, run
frames, and read the hand-overs and the log — `Fake::attach_watching_the_pacing`, `Fake::frames`,
`Fake::handovers_us`, `Fake::log`. What stands between them and a second game is not the scenarios and not
the judging; it is that the only thing to attach them to is a th06.

**Two questions were parked until there was a second game.** *Support Touhou games other than 紅魔郷* in
`TODO.md` says them: whether `State` says everything a chapter needs for a game whose scoring or resources
work differently, and whether the midstage table's shape — script frame numbers per stage, seven of them in
`th06/chapters.rs` because 紅魔郷 has seven — holds where stages are not one script on one clock. Neither is
answerable by reading 紅魔郷 harder, and neither has stopped anything so far because there has been nothing
to ask it of.

**What is not known about th07, which is all of it.** No address of it is in the tree, and none is going
in from anywhere but the exe: a claim about the game's behaviour is established by measurement and the
measurement is kept. That the game is even the same *shape* — a chain of registered jobs, a supervisor
with a scene it wants, a score file with a record per spell card — is itself a claim to be established by
disassembly rather than assumed from 紅魔郷. The exe this is about is `th07.exe`, 650752 bytes, md5
`0126afce1e805370d36c3482445e98da`; which version that is, the file name does not say, so the md5 is what
pins it the way 紅魔郷's pins 1.02h.

## Decision

**th07 is a `Game` of its own that answers what has been measured and declines the rest, and the game orb
attaches to is chosen once, at the attach.**

- **`orb-core/src/game/th07/`**, the shape `th06/` has: `mod.rs` for the addresses and offsets, `image.rs`
  for the same address space laid out, `chapters.rs` if and when there is a midstage table. No branch
  inside `Th06`, and nothing of th07 in orb's own logic — that is what the seam is for.

- **A stub declines; it does not panic.** Every method is written out — 61 of the 62, `midstage` being the
  one with a body of its own to inherit. The ten that return
  `Option` answer `None` where the answer is not measured, and the trait already says what each `None`
  costs — no replay to suppress, a run that cannot be picked up again, a resumed run without its seed, the
  captures left as the ranking read them, no spell-card counting, no question over the game's own menu.
  `Hooks` is the same: two patches are required, `update` and `draw`, and eleven are `Option`. The rest
  answer the emptiest answer that is *true* — no regions, no music, not windowed — and **no method answers
  a guess**. An `unimplemented!()` anywhere in there would make the first run that reached it take the
  game down, which is the one thing an injected DLL must not do to somebody's play session.

- **The first slice is the frame loop and the overlay**, because that is the part of orb that needs nothing
  about a run: the window, the device, the chain, the two frame calls, the play area, the content size. And
  **the stub is how the list is found** rather than something to know first — a `Th07` that declines
  everything declinable, run against a game laid out from what has been measured so far, says in the log
  exactly which answer it was missing next.

- **The game is a table read once.** Exe name, md5, and the `&'static dyn Game` — matched at the attach
  from `host_exe()`, stored in a static the hooks read, and named in the log with the version it matched.
  A process that matches nothing is a run where orb does nothing and says which games it knows, which is
  what an unrecognised md5 already gets from the launcher. The launcher reads the same table, so the two
  cannot disagree about which games exist.

- **The two parked questions come due later, and declining is what buys the time.** A `Th07` whose
  `midstage_table` is empty and whose `stage_begun` and `stage_building` are `None` has no chapters, so
  neither question stands in the way of a game orb paces. They fall due at the step where th07 starts being
  played, and what answers them is that game's own code — whether a stage is one script on one clock, and
  what a chapter of it would have to carry — rather than the shape 紅魔郷's answer has.

- **`Fake` splits along the line 0003 already drew.** The host and the launch are any game's — the display,
  the clock, the keyboard, the recording device, what a frame's work costs, `in_its_own_process`, and the
  frames — and playing 紅魔郷's part is th06's. A second game brings its own half and nothing else.

- **th07's e2e stub is one scenario**, `orb/tests/th07.rs`: a laid-out 妖々夢, orb attached to `Th07`,
  frames run through `render`, and the log holding the lines that say orb got in — the `.data` bounds it
  found, the display it settled on, the overlay built, and a `frame:` line at sixty. Which is the smallest
  thing that says the seam holds for a second game, and it fails loudly the day one of those addresses is
  wrong.

## Consequences

**What it costs.** A second `image.rs` of addresses only the exe can say, the static becoming a lookup, and
the launcher's one exe becoming a table. And every method th07 declines is a feature 完全無欠 does not have
there: no chapters, no retry menu, no run picked up again, no card counted. So th07 is a game orb *paces*
long before it is a game orb *plays*, and the log has to say which of the two this run is.

**What it buys.** The seam gets its second implementation, which is the only thing that ever says it is a
seam: `Game`'s 62 methods have been shaped by one game, and what a second one finds is where the trait
asked for something only 紅魔郷 has. Everything 0001 claimed about a laid-out game and 0003 about judging
values is put to a game that was not the one it was written for. And the pacing — measured over four hosts
and eleven displays — becomes true of another game for the price of an image.

**What stays as it is, deliberately.** The 46 pacing scenarios stay th06's until there is a th07 image with
frames to run. They are about orb's loop and not about which game is under it, so running them twice buys
one more `render` caller and costs the whole table twice over.

**What it rules out.**

- **A branch inside `Th06`.** 0001's and 0002's rule, for the third time: a game that is a special case is
  a game whose special case is what production runs.
- **`unimplemented!()` for what is not measured yet.** The trait's own `Option`s are the way to decline,
  and a method that cannot decline gets the emptiest true answer instead.
- **A build or a feature flag per game.** The launcher injects into whatever the directory it was pointed at
  holds, so a wrong build would be a run with another game's addresses — silent, and reading somebody
  else's memory. One DLL that recognises the process it is in fails with a message instead.
- **Asking the game per call.** A lookup per frame for an answer that cannot change inside a process, and
  the hooks have nothing to ask with.
- **Guessing th07's addresses from 紅魔郷's, or from a table somebody else published.** Neither is a
  measurement. What another project's table is good for is knowing where to look.

## Plan

1. `GAME` becomes a static chosen at the attach, from a table of one entry — `Th06`, its exe and its md5 —
   so no run changes and the shape is in place. The log names which game matched and on what.
2. The launcher reads that same table: `verify_game_exe` says which games and versions orb knows, instead
   of one.
3. `Fake` splits into the host half and 紅魔郷's half. Nothing asserted changes; the count does not move.
4. `orb-core/src/game/th07/mod.rs` with `Th07` declining everything the trait lets it decline, and the
   addresses a frame needs read out of `th07.exe` (md5 above) — each written down in `DONE.md` with how it
   was found, as th06's were.
5. `orb-core/src/game/th07/image.rs` laying out that much of the space, and `orb/tests/th07.rs`: the one
   scenario above.
6. A run on the machine — `scripts/run.sh <the th07 directory>`, which already takes it as an argument —
   with the `orb.log` written beside that exe kept in `DONE.md`: the rate over a reporting period, the gaps
   in refreshes, and nothing claimed beyond what the log says.

Steps 1 to 3 are orb's own shape and can land before any address of th07 exists; 4 onwards cannot start
before the disassembly. What must not happen in between is an address written down because it is where
紅魔郷 keeps the same thing.
