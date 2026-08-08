# What the fake does not copy

**Read out of the game's own code, not off this tree.** The fake game is both the writer and the reader of
the memory it drives orb through, so a place where it does something 紅魔郷 does not is a place every
scenario agrees with: nothing in this repository can tell. What told is a decompilation of the same binary
the project is built against, held honest by the exe's own bytes — the route is *Re-deriving it* in
[0008](../adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md), which is also where the rule for
which of these is a defect lives.

**The offsets came out clean.** Twenty-one function addresses, thirteen prologue byte arrays, every struct
offset `Th06` reads through with each struct's total re-derived against the decompilation's own
`ZUN_ASSERT_SIZE`, every enum value and button mask: not one wrong. So nothing on that layer is in this
file. What is here is the fourteen places where the fake's **behaviour over time** differs, and the order
worth working through them in.

## Where the code is

| | |
| --- | --- |
| `crates/orb-sim/tests/fake/th06.rs` | the fake game — `Fake::update`, `build`, `stage_numbers_in_place`, `shakes_the_screen`, `update_the_player`, `own_render`, `input`, `MAP`, `FRESH` |
| `crates/orb-sim/tests/fake/mod.rs` | the half of a launch that is any game's, and the vocabulary a scenario drives one by |
| `crates/orb-core/src/game/th06/image.rs` | where each thing in the game's memory lives — every `Image` method the fake writes through |
| `crates/orb-core/src/game/th06/mod.rs` | the offsets and addresses themselves, and `Th06`'s reads and writes |
| `crates/orb/src/lib.rs` | orb's hook entry points, `Originals`, `attach_to` |
| `crates/orb/src/resume.rs` | `resume::stage_building` and `stage_begun`, which are what the two stage hooks are for |
| `crates/orb-sim/tests/scenario_*.rs` | the scenarios — one file per subject, and where a new one goes |

## How to read it

**A divergence is a defect when it makes a scenario pass that the real game would fail.** That is 0008's
test, and it is why this list is not a list of everything the fake does differently: the generator, the
boss's arrival, the stage's length and the layouts are declared stand-ins and stay that way. The table's
last column says which is which.

**Every item is red first.** The behaviour is already written down in the decompilation, so the scenario
asserting it can be watched fail before the fake can pass it. A fix whose scenario was never red has its
assertion back-derived from the code under suspicion, which is the failure 0008 is written against — treat
it as unmade. See 0008's *Decision*. Each item below carries the assertion its red scenario makes, and
**one item has no red available**; it says so where it is.

## Working on it

**The decompilation is a prerequisite, not a reference.** The `src/…` paths in the table below are inside
the clone, and without it none of them resolve:

```sh
git clone --depth 1 https://github.com/GensokyoClub/th06.git
```

Check it is about this exe before believing a line of it — `sha256sum "$ORB_DEST/東方紅魔郷.exe"` against
the hash in its `README.md`. 0008's *Re-deriving it* has the rest of the route: address to function name,
global to name, prologue to bytes, a struct's total to its assertion, and whether a function is decompiled
or still a stub.

**Running it.** `cargo xtask test` for the suite and `cargo xtask clippy` for the lint, both as `README.md`
describes; `.husky/pre-commit` runs those two and `cargo fmt --check`. **The suite was 327 passed, 0 failed, 0 ignored when this
was written**, across 29 binaries of which 19 are `scenario_*.rs`. A later count that disagrees has had work
done on it — the command is what settles whether this file is still true, not the number.

**A scenario is a `#[test]` in one of `crates/orb-sim/tests/scenario_*.rs`**, and each runs in a process of
its own: `in_its_own_process` spawns the binary again for the one test and asserts the child said
`1 passed`, so a filter matching nothing cannot report green. What a scenario is allowed to say is the host
and the input — see [0001](../adr/0001-a-fake-th06-drives-orb-end-to-end.md) — and everything else it reads
back, out of the game's own memory through `Th06::read_state` and `Th06::reproduction`, or off the screen.

## Re-reading each one

These are where each was read, so a finding that has moved is a diff rather than a claim to take on trust.
`th06/` is the clone.

| | what the fake does | where the game's own is | costs |
| --- | --- | --- | --- |
| D1 | leaves the input word after a scene is built | `src/Supervisor.cpp`, `Supervisor::OnUpdate`, the assignment at the end of the state switch | catches |
| D2 | skips every chain job on the build frame | `src/Chain.cpp` `AddToCalcChain`/`RunCalcChain`, `src/ChainPriorities.hpp`, `src/GameManager.cpp` `RegisterChain` | catches |
| D3 | starts a stage with the player normal | `src/Player.cpp`, `Player::AddedCallback` | catches |
| D4 | draws two numbers a shake frame | `src/Rng.cpp` `GetRandomU32`, `src/ScreenEffect.cpp` `ShakeScreen` | catches |
| D5 | says the arcade region is written once | `src/GameManager.cpp`, `AddedCallback`'s non-REINIT branch | prose |
| D6 | clamps the player to the arcade region | same branch, `playerMovementArea*` | low |
| D7 | says the key map is configured | `src/Controller.cpp` `GetInput`, `src/Supervisor.hpp` `GameConfiguration` | prose |
| D8 | answers the keyboard alone from `input()` | `src/Controller.cpp`, `GetInput`'s tail call | low, refused |
| D9 | calls 80 frames the bomb shake's length | `src/BombData.cpp:159,293,299,388,498,502` | prose |
| D10 | never breaks the walk early | `src/GameManager.cpp` `OnUpdate`, `src/Chain.cpp` `RunCalcChain` | low, refused |
| D11 | leaves a failed `clrd` read all zero | `src/ResultScreen.cpp` `ParseClrd`, `src/GameManager.hpp` `HasReachedMaxClears` | prose, and a mechanism worth fixing |
| D12 | reads the score file per stage | `src/GameManager.cpp`, `AddedCallback`'s non-REINIT branch | catches |
| D13 | resets lives, bombs and power per stage | same, and the absence of any write to `livesRemaining`/`bombsRemaining` anywhere in it | catches |
| D14 | never writes `subRank` or `rank` | same, and `crates/orb-core/src/game/th06/mod.rs:1560` for orb's read | low |

One more is not a divergence but a trap: the fake writes the player's position and `g_Gui`'s draw job
*before* `orb::stage_building`, where `AddedCallback` has `Stage::RegisterChain` first and
`Player::RegisterChain` and `Gui::RegisterChain` after. `resume::stage_building` writes only the seed today,
so nothing reads what is out of order — which is exactly why it will not announce itself later.

And one is still unresolved: `CURRENT_CARD` at `0x005a5f98` has no name in `globals.csv`, and the exe
touches it only from `EclManager::RunEcl`. The provenance agrees with what orb reads it as; the meaning is
not confirmed — a read of the ECL spell card instruction, or a run against the real game, is what would.

## The order

**1. The REINIT branch — one condition, three findings (D12, D13, D5).** At the moment
`stage_numbers_in_place` runs, `Image::scene()` is still `Scene::Rebuilding`, because `build` calls
`orb::stage_begun` before it writes the supervisor's copy. So the game's own
`curState != SUPERVISOR_STATE_GAMEMANAGER_REINIT` needs no new plumbing. Into that branch: the score file's
read, the reset of lives, bombs, power and deaths, the arcade region and the player movement area. The other
branch is the two lines the game's own else is — `guiScore = score`, `nextScoreIncrement = 0`.

That read is also what calls `Th06::set_captures`, so the record of spell cards is written back from the
file as often as the file is opened — once a run in the game, once a stage here. A resume plays a run's
buttons in with that record held (`resume::hold_captures`), so the two meet, and the fix changes how often
they do.

> **Red:** a run played across a stage boundary with a life already lost — `stages_last`, then `hit` —
> asserting `Th06::read_state`'s `lives` on the next stage is what the last one left rather than `FRESH`'s
> two, and the same for `bombs` and `power`. Second red: `score_file_opens` over that run holds **one**
> read at the run's start and none at the transition. Third red: a bomb's shake left the arcade region
> moved, the run given up, a new run started — `arcade_region_size` is back at the game's `(384, 448)`.

**2. The frame a scene is built on (D1, D2).** Drop the early return from `Fake::update`, zero the input
word after `build`, fall through to the scene's own update. This is where the expectation churn is, and it
is at *every* transition rather than only a stage's. `enters_the_ending` does not go through `build`, so the
ending's 29,040 updates are untouched.

> **Red:** on the frame a stage is built, `read_state`'s `stage_frames` is already 1 — the stage's own first
> update having run in that same frame — where it is 0 today. And `Image::input_now` on that frame is zero,
> which is what `Supervisor::OnUpdate` leaves behind and what makes a button still held read as a fresh
> press on the frame after.

**3. The stage's first player (D3).** Spawning for the respawn frames, then invulnerable counting down from
120, then normal. Both halves are owed because orb reads both: `read_state` carries spawning as `unsettled`,
and `make_invulnerable` leaves spawning alone.

> **Red:** `puts_a_bullet_on_the_player` from a stage's first frame, asserting no life is lost inside the
> 120 frames and one is lost after them — where today the first frame kills. And `read_state`'s `unsettled`
> is true over the respawn frames at a stage's start.

**4. The two cheap ones.** Split the single `reproducing` call so the seed goes in before
`orb::stage_building` and the position and `g_Gui`'s job after, which closes the trap above. Write `subRank`
and `rank` (D14), which wants setters on `Image` — the offsets are already in `game_manager`.

> **Red for `rank`:** `Th06::reproduction`'s `rank` at a stage's start is the difficulty's own, out of
> `g_DifficultyInfo` in the decompilation — read the value there first — where it is zero today.
> **No red is available for the split**, and that is the finding rather than an omission: nothing reads
> what is out of order while `stage_building` writes only the seed, so no scenario can fail on it. It goes
> in as a tidy with the reasoning in a comment beside it, and the day `stage_building` grows a second write
> is the day it would have cost a debugging session instead.

**5. The hooks no scenario reaches.** `init_d3d_device` is always installed in a real launch and is not in
`Originals`: the fake writes the device at attach where production finds it through this hook, so reaching
it means the fake having a phase before its device. Restructures `Fake::attach`, and pairs with giving the
fake a startup sequence so `creates_its_window` stops being a scenario poking a hook.
`get_controller_input` and `save_replay` are an `Originals` entry and a scenario each.
`hook::install_import` cannot be reached by a laid-out game at all — there is no import table — so it wants
a test over a synthesized one rather than a scenario, and `memtrack` stays off for the same reason.

> **Red:** a launch whose game has no device until its startup runs, asserting orb draws nothing before the
> hook and its overlay is ready after — where today the device is there before orb is attached and the hook
> is dead code. For `save_replay`, a `block_replay_save` launch asserting the game's own record was not
> written. For `get_controller_input`, a Verbose launch asserting the joystick span reaches the perf line.

**6. The numbers (D4, D6, D11).** Four draws a shake frame and the three cases per axis. The movement area
at the game's `(8, 16)`/`(368, 416)`. A failed `clrd` read leaving magic, version and every difficulty entry
at 1, with the Extra item gated on `== 99`.

> **Red:** `Th06::reproduction`'s `randoms` over N frames of a shake rises by `4 * N`, not `2 * N`.
> `player_area` reads the game's `(16, 416)` rather than the arcade region's `(16, 448)`.
> After a score file that is not there, `Image::unlocks` holds `CLRD` and difficulty entries at 1 rather
> than zeros — **and `Extra Start` is still off the drawn menu**, which is the assertion that keeps the
> mechanism honest rather than only the outcome.

**7. The prose (D5, D7, D9, D11).** No code moves and no scenario is owed: what is wrong is four comments —
why the arcade region is not written per stage, where the keyboard's map comes from, `SHAKE_FRAMES = 80`
being one of six, and what a failed `clrd` read leaves. Correcting a comment that misstates why is not a
change to the fake.

**8. The chain, last (D10, and D2's root).** `Fake::update` as a walk over priority-ordered jobs. **Not
first** — it moves the same frame accounting item 2 moves, and doing both at once means being unable to say
which moved what.

> **Red:** three things no scenario can ask today. A job answering "remove me" leaves the walk with the
> rest of the jobs still run. `Chain::Cut` on a job in the middle unlinks that one and no other. And a
> chapter restored puts the chain's own linked list back — `g_Chain` and the static elements are in
> `.data`, so `gui_in_the_draw_chain` and a shake's element come back with the memory, which nothing has
> ever asserted.

## What it costs in expectations

Measured, so the cost is visible rather than feared.

- **No scenario writes an absolute frame number** (no `248`) or `lives == 2` / `bombs == 3` directly, so
  items 1 and 2 churn less than they look like they will.
- **The write is `scenario_the_score_file.rs`'s counted reads and writes** — lines 133, 155, 195, 234 and
  237 — because item 1 takes the per-stage read away.
- **`frames_until` waits are conditions rather than counts**, so most of item 2's one-frame shift is
  absorbed without an edit. The ones that are not will fail loudly, naming what they were waiting for.

## What stays

The declared stand-ins [0001](../adr/0001-a-fake-th06-drives-orb-end-to-end.md) already carries: the
generator, the boss's arrival and its card, the stage's length, the demo's patience, the player's speed, the
shake's pixels, the tracks, the record's layout, the title and ranking layouts. And two 0008 refused rather
than deferred — the pad's merge into the input word, and the game's own pause and retry menus. Both get a
*why not* comment where somebody would otherwise be tempted back to them; 0008's *What was weighed and
rejected* is the reasoning.

**When the last item lands, this file goes**, and what has to survive goes with it: a reason an alternative
was rejected to a comment beside the code that would tempt somebody back, a measurement's recipe to
whatever runs it, anything still undecided to `docs/adr/`, and anything built and waiting on a run against
the real game to `TODO.md`.
