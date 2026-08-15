# 4. th07 is a second `Game` chosen at the attach, and it declines what has not been measured

**Status:** accepted and built, with one thing it claimed disproved by the run that built it — and that
one thing since answered by
[0017](0017-the-frame-loop-has-a-seam-either-side-of-the-draw-chain.md), which is where the rest of
妖々夢's frame was read and where two of the readings below are corrected: the `calls=1200` in the perf
line is orb recording the update phase twice per hook call rather than the game updating twice a frame,
and the two calls on 0x4b9e44 are a queue of quads emptied and drawn rather than a render-state block.
`Hooks::render` is `Some` now.
`orb_core::game::KNOWN` names 紅魔郷 1.02h and the `th07.exe` of md5 `0126afce`, both halves read that one
table, `orb/src/lib.rs` chooses its game at the attach out of `host_exe()`, `orb-core/src/game/th07/`
holds a `Th07` that declines everything about a run, and `crates/orb-e2e/src/th07.rs` is the one
e2e test — `orb/tests/th07.rs` when this was written, moved by
[0005](0005-every-e2e-test-lives-in-orb-sims-tests.md), which is where every path below with `orb/tests`
in it went, and again by
[0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md). Step 6
happened: 妖々夢 was launched with orb in it and **orb's own frame loop took it down on the first frame**,
so `Hooks::render` is `None` and 妖々夢 keeps its own cadence. What that costs, and what reading the rest
of that frame would take, is
[issue 2](https://github.com/youxkei/ultramarine-orb-elixir/issues/2); the measurement is *What the two
launches said* below and
beside the `render: None`
itself. What stands is everything about the *shape* — the table, the choice at the attach, the split rig,
and a stub that declines rather than panics.

## What the two launches said

**妖々夢 has been launched with orb in it, twice, and the first launch took the game down.** The exe is
the one the table pins, `md5 0126afce1e805370d36c3482445e98da`. What the log gave up in order:

| | |
| --- | --- |
| `.data 0x0049c000..0x01365258 (15503960 bytes)` | read out of the loaded image, six times 紅魔郷's 2562556 |
| `game: th07.exe, and every address orb has for it was read off the build of md5 0126afce` | the table matching, off the exe's own name |
| `update hook installed` / `draw hook installed` | **the two required patches are right in the real image** — a prologue that did not match is what a wrong address says first, and 0x42fd60 and 0x42fe20 matched theirs |
| `screen: 1280x960 — window at 1277,580 sized 1286x1000, client 1280x960` | `content_size` being (640, 480) is what the 4:3 was worked out from |
| `frame: 120Hz compositor, one frame every 2 blank(s)` | the cadence settled before a frame ran |
| `crash: code 0xc0000005 at 0x0044f6aa in th07.exe+0x4f6aa`, `writing 0x00000000` | the first frame, gone |

**The same launch under `--no-frame-loop` — that frame left alone with orb's update and draw hooks inside
it — ran 600 frames and left cleanly**, which is what makes it the loop rather than either patch: `perf:
frame=17220us worst=395238us over 600 frames; update=8us/frame worst=1084us calls=1200 draw=3us/frame
worst=341us calls=570`. `calls=1200` over 600 frames is 妖々夢's frame-skip loop running the update twice a
frame, which 紅魔郷 has nothing of.

**Two more things the run said that nothing else could.** There is no `font.ttf` beside `th07.exe` —
紅魔郷 ships one and 妖々夢 keeps its fonts inside `th07.dat` — so `overlay: cannot load …\font.ttf` eight
times and then `overlay: unavailable`: **nothing orb draws is drawn there at all.** And orb forked the
score file of a game it declines every run feature of: `score: pointdevice_score.dat opened in place of
the game's own, write` on the way out, leaving a 192-byte file of its own. `score.dat` came through both
launches untouched — `647b08bc49eba267808df31e7f18af12`, 8467 bytes, at the mtime it went in with — and
`th07.cfg` too, which is the fork working; that it happens at all for this game is
[issue 2](https://github.com/youxkei/ultramarine-orb-elixir/issues/2).

It stands on three decisions. [0001](0001-a-fake-th06-drives-orb-end-to-end.md) put a game that plays the
game's part in front of orb and ruled out a second `Game` written for tests.
[0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) made the frame loop's two calls into
the game addresses the game hands over, and stored them in statics at the attach because a hook has
nothing but the ABI's arguments — which is the shape the choice of game takes here.
[0003](0003-the-frame-loops-e2e-tests-are-one-file.md) left the 46 e2e tests about the frame loop judging
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
own address space — `DATA 0x00476000..0x006e79fc` and the blocks around it — and the types it hands an
e2e test are 紅魔郷's model of a game: `Scene`, `Supervising`, `FrontEnd`, `Screen`, `Playing`,
`Reproducing`. `orb/tests/fake/mod.rs` is 1400 lines playing that game's part, naming `Th06` in 24 places:
the supervisor's build-then-copy order, the title menu's item indices, the `catk` record, the shot-type
screen, the bits of its own input word.

**But the frame loop's 46 e2e tests touch none of that.** They declare a display, start a launch, run
frames, and read the hand-overs and the log — `Fake::attach_watching_the_pacing`, `Fake::frames`,
`Fake::handovers_us`, `Fake::log`. What stands between them and a second game is not the e2e tests and not
the judging; it is that the only thing to attach them to is a th06.

**Two questions were parked until there was a second game.**
[Issue 2](https://github.com/youxkei/ultramarine-orb-elixir/issues/2)
says them: whether `State` says everything a chapter needs for a game whose scoring or resources
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

  **The stub did find the list, and the frame loop was not on it — it was the thing that broke.** The
  slice was chosen as the part that needs nothing about a run, and that is true of it; what is not true
  is that a frame is a list of addresses. Composing 妖々夢's frame means doing what its own frame does
  around the two chain walks, and that is code to be read rather than six numbers. So the first slice
  the *next* game should take is the window and the content size, which are two answers and were both
  right in the real image, and the frame loop should be the last thing attempted rather than the first.
  The overlay is not a slice at all where the game ships no font.

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

- **th07's e2e stub is one e2e test**, `orb/tests/th07.rs`: a laid-out 妖々夢, orb attached to `Th07`,
  frames run, and the log holding the lines that say orb got in. Which is the smallest thing that says the
  seam holds for a second game, and it fails loudly the day one of those addresses is wrong.

  **What it holds against is what the run showed, not what this expected.** Frames through the game's own
  frame rather than through `render`, since that is what a launch there installs; `overlay: unavailable`
  rather than the overlay built, 妖々夢 shipping no font; no `frame:` line at sixty, orb pacing nothing
  there. What it asks instead is that orb did *none* of what it does to 紅魔郷 — no chapter, no retry, no
  resume, no wash, no mode question — and wrote no `panic:` and no `crash:`. An e2e test asserting the
  four lines this paragraph originally named would have passed while the real game died, because the
  laid-out one has a font and answers every read.

## Consequences

**What it costs.** A second `image.rs` of addresses only the exe can say, the static becoming a lookup, and
the launcher's one exe becoming a table. And every method th07 declines is a feature 完全無欠 does not have
there: no chapters, no retry menu, no run picked up again, no card counted. So th07 is a game orb *paces*
long before it is a game orb *plays*, and the log has to say which of the two this run is.

**And it costs more than that, which the run found.** th07 is not a game orb paces either. Replacing
妖々夢's frame took the game down on its first frame — see the `render: None` in `th07/mod.rs` for the
faulting address and *What the two launches said* above for the pair that separated the loop from the two
patches — so
what a launch there has is the window at the right shape and orb's update and draw hooks inside the
game's own frame, and nothing else. No cadence of orb's, no frame of input lag removed, and nothing
drawn, 妖々夢 having no `font.ttf` beside its exe to build an overlay from. The paragraph below about what
this buys was written before any of that and is wrong where it says otherwise; it is left standing
because being wrong in a particular way is the finding.

**What it buys.** The seam gets its second implementation, which is the only thing that ever says it is a
seam: `Game`'s 62 methods have been shaped by one game, and what a second one finds is where the trait
asked for something only 紅魔郷 has. Everything 0001 claimed about a laid-out game and 0003 about judging
values is put to a game that was not the one it was written for. ~~And the pacing — measured over four
hosts and eleven displays — becomes true of another game for the price of an image.~~ **Not for the price
of an image.** The pacing is a *frame* orb has to compose, not a rate it applies to one, and what
composing 妖々夢's frame takes has not been read. The addresses lined up; the shape did not.

**What stays as it is, deliberately.** The 46 pacing e2e tests stay th06's. They were to wait for a th07
image with frames to run, and now there is no th07 loop for them to run — they are about orb's loop and
not about which game is under it, so running them twice would buy one more `render` caller and cost the
whole table twice over.

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

1. **Built.** `GAME` is a static chosen at the attach, from a table of one entry — `Th06`, its exe, the
   file it keeps its own configuration in, its md5 and what that build is called — so no run changes and
   the shape is in place. The log names the build a run's addresses were read off, and a process no entry
   names gets the list of games orb knows and nothing done to it.
2. **Built.** The launcher reads that same table: it finds the game by which entry's exe is in the
   directory it was pointed at, and `verify_game_exe` says which games and versions orb knows instead of
   one.
3. **Built.** `Fake` is split into the host half and 紅魔郷's half, over a `Launched` trait whose four
   required methods are the game's window, its own frame, the host its memory is in and the half of a
   launch it holds. Nothing asserted changed; the count did not move.
4. **Built.** `orb-core/src/game/th07/mod.rs` with `Th07` declining everything the trait lets it decline,
   and the addresses a frame needs read out of `th07.exe` (md5 above) — each written down beside the
   constant with how it was found rather than in a document of its own, since what identifies an address
   is the reason the code holds that number and belongs where somebody changing it will read it.
5. **Built.** `orb-core/src/game/th07/image.rs` laying out that much of the space — one range, the
   `.data` the section header reports — and `orb/tests/th07.rs`: the one e2e test above.
6. **Done, and it sent step 4 back.** Two launches on the machine, `orb.log` beside that exe — *What the
   two launches said* above:
   the first with orb's own loop, which the game did not survive one frame of, and the second under
   `--no-frame-loop`, which ran 600 frames and left cleanly. No rate over a reporting period, because
   `Hooks::render` is `None` now and orb paces no frame there — which is the thing the step was for
   finding out, and it could not have been found any other way.

Steps 1 to 3 are orb's own shape and can land before any address of th07 exists; 4 onwards cannot start
before the disassembly. What must not happen in between is an address written down because it is where
紅魔郷 keeps the same thing.

**What the disassembly and the run between them settled, and what each got wrong.** The addresses are
beside their constants in `th07/mod.rs`. Three things are worth naming here rather than only there,
because each is about this decision and not about a number.

The device pointer and the window are not two globals to be hunted separately: both fall out of the one
`IDirect3D8::CreateDevice` call, which is a better witness than either would have been alone. That is the
shape a third game should be looked for by.

th07's frame is called on a *static* window object rather than through a patched call site, with a
frame-skip loop inside it that 紅魔郷 has not. This decision took that for a difference to watch a run for.
It was the fault: the skip loop is not the only thing 紅魔郷's frame has not got, and the render-state
block 妖々夢 pushes around its drawing is what orb's loop left null.

And the lesson that outlives the numbers: **a `Game` can decline a method, and it cannot decline half of
one.** Every other seam in the trait is an answer orb reads. `Hooks::render` is orb saying it will do the
game's job instead, so it is the one seam where being *nearly* right is worse than declining — the
addresses were all correct and the game still died. Which is why `render` should be the last thing a new
game is given and not the first, and why the two required patches, `update` and `draw`, are the right
place for a second game to stop.
