# 8. The fake game copies the game orb is injected into, and every fidelity fix is red first

**Status:** accepted, not built. Nothing in `crates/orb-sim/tests/fake` looks like this yet, and the
divergences named below are all still there. It stands on
[0001](0001-a-fake-th06-drives-orb-end-to-end.md), which built the fake 紅魔郷 and named its own cost —
*a wrong offset makes the writer and the reader wrong together, and only the real game says otherwise* —
and this is where that got an answer which is not a run against the real game. The answer arrived with a
second finding 0001 did not anticipate, and that finding is what this decision is about: the layer 0001
worried over is sound, and the layer it did not mention has drift in it.

## Context

0001 answered *is the fake wrong in the same direction as the reads it feeds?* by putting the confirmation
on the real game: the offsets are 紅魔郷's to confirm, each written down beside the offset it was read at.
That is a good answer for an offset and no answer at all for a behaviour. Where the fake decides *when*
something happens — which frame a scene's first update falls on, how many numbers a screen shake draws,
what a stage start puts back — there was no witness of any kind, and a scenario asserting the fake's answer
would pass whatever that answer was.

**There is now a witness that is not a run.** [GensokyoClub/th06](https://github.com/GensokyoClub/th06)
reconstructs 1.02h, and the binary it targets has the same sha256 as the exe this project is built
against — `9f76483c46256804792399296619c1274363c31cd8f1775fafb55106fb852245`. So the game's own code can be
read, and — because agreement with a decompilation this repository already cites in `SPEC.md` is not
independent of it — the exe's own bytes can be held against what is read. Both were done. The recipe is at
the end of this document, because the measurement that decided a shape belongs beside the reasoning it
settled.

**Everything that is an address or a number is right.** Twenty-one function addresses; thirteen prologue
byte arrays read out of the exe's `.text`, which are the bytes `hook::install` copies into a trampoline and
so the ones a real launch falls over if they are wrong; every struct offset `Th06` reads through, with each
struct's total re-derived against the decompilation's own `ZUN_ASSERT_SIZE` so that a field-by-field match
is a match rather than a coincidence; every enum value and button mask. Not one was wrong. `WAS_PRESSED` is
the same expression the fake's `pressed` closure is. `set_run_seed`'s doc — *the callback has just zeroed
the count, which is what a stage starts from* — is `g_Rng.generationCount = 0; mgr->randomSeed = g_Rng.seed;`
sitting immediately before `Stage::RegisterChain`, exactly as it says. **The layer 0001 said it could not
catch is the layer that turned out to be sound.**

**Everything that is a behaviour over time has drift in it.** Fourteen places, of which six change what a
scenario can catch. The fake reads the score file once per stage where the game reads it once per run; it
resets lives, bombs and power at every stage where a continuous run carries them; it skips every chain job
on the frame a scene is built where the game runs the new scene's first update in that same frame with the
input word zeroed; it starts a stage with the player killable where the game starts them spawning and
invulnerable for 120 frames; its screen shake draws two numbers a frame where the game draws four; it never
writes `subRank` or `rank`, both of which orb reads. None of these is visible from inside the suite,
because the fake is both the writer and the reader — which is 0001's own stated cost, arriving in the layer
0001 did not name.

**And the comparison had been aimed at the wrong game.** The first pass of this cross-check held the fake
against 紅魔郷. That is not what orb runs in. orb replaces `GameWindow::Render` outright, so the frameskip
loop that function is mostly made of is dead code in a default launch, and a fake that reproduced it would
be spending fidelity on something no launch executes. Meanwhile the walk `Chain::RunCalcChain` performs is
*not* replaced — orb calls through to it — so the fake owes that walk everything. The two are opposite
obligations, and nothing in the fake said which was which.

## Decision

**The fake game is faithful to the game orb is injected into, not to 紅魔郷, and the seam is what decides
where fidelity is owed.**

`attach_to` stores each `Originals` entry in the same static a real launch fills with `hook::install`'s
trampoline. orb's own code is therefore identical in both, and the only thing that can differ is what lies
on the far side of a hook. Three obligations follow, and they are different obligations:

- **What orb calls through, the fake owes in full.** `Fake::update` stands in for the whole of
  `Chain::RunCalcChain` — fourteen jobs — and every one of the six consequential divergences is inside it.
  `Controller::GetInput`, `GameManager::AddedCallback`, `Stage::RegisterChain`, `MainMenu::AddedCallback`,
  `ResultScreen::AddedCallback`, `ReplayManager::StopRecording`: same.
- **What orb replaces, the fake owes nothing.** The game's own `Render` is the case, and `own_render` is
  right to be one walk with no wait: it is the `--no-frame-loop` configuration's stand-in and that
  configuration is the only one where the game's loop is alive at all.
- **What orb never reads or writes, the fake need not have.** This is what keeps the fake a game and not a
  port.

**A divergence is a defect when it makes a scenario pass that the real game would fail.** That is the whole
test. Everything else is a stand-in, and a stand-in is declared where it stands — which is what 0001
already does for the generator, the boss's arrival and the rest, and what this adds nothing to.

**Every fidelity fix is red first, and here that is not a working preference.** A fidelity fix has a
property an ordinary change does not: the behaviour being claimed is already written down outside this
repository, so the scenario asserting it can be written and watched fail *before* the fake is capable of
passing it. Doing it that way is what turns a claim read out of somebody else's source into something this
repository executes.

Landed the other way round — fake first, scenario after — a green scenario proves only that the scenario
agrees with the fake. That is 0001's writer-and-reader-wrong-together in a new place: the offsets got an
external witness and the behaviours would get their assertions back-derived from the very code under
suspicion. **So the order is: read the game's own code, write the scenario from it, watch it fail against
the fake as it is, then change the fake.** A fix whose scenario was never red is a fix with no witness, and
should be treated as unmade.

## What was weighed and rejected

- **Holding the fake against 紅魔郷, which is what the first pass did.** It gets both halves wrong at once:
  it charges the fake for reproducing code orb has replaced, and it says nothing about the one function —
  the chain walk — where the fake is standing in for everything. The relation to check is *fake ↔ injected
  game*, and that relation is legible because `attach_to` and `attach` fill the same statics.
- **Starting with the chain.** Modelling `Fake::update` as a real walk over priority-ordered jobs is the
  root fix for the build-frame divergence and for the pause menu's early `BREAK`, and it is the last step
  rather than the first. It moves every scenario's frame accounting, and so does the build-frame fix that
  has to happen anyway; doing both at once means re-deriving expectations against two moving things and
  being unable to say which moved them. The chain goes last, once the numbers have settled.
- **Modelling the pad's merge into the input word.** `Controller::GetInput` returns keyboard **or** pad, so
  in a real launch a pad moves the player; in the fake it only answers orb's own menus. Copying this means
  the fake taking on `GetControllerInput`'s mapping arithmetic — which is precisely what the fake's own
  comment refuses, on the grounds that where an axis becomes a direction is the game's business and not a
  thing to reimplement twice. It stays refused, and what changes is that the refusal gets written down
  beside `input()` as a *why not*, where somebody tempted back to it will read it.
- **Modelling the game's own pause and retry menus, so that `GameManager::OnUpdate`'s early `BREAK` can
  be reached.** The menus are absent by design and `gives_the_run_up_at_its_own_pause` is a declared verb
  standing in for them. The early break arrives for free once the chain is a walk, so this is not work of
  its own — it is one more thing that becomes reachable at the last step.
- **Fixing every divergence.** Four of the fourteen are prose: the reason given for the arcade region not
  being written per stage, where the keyboard's key map comes from, `SHAKE_FRAMES = 80` being one of six
  bomb shake lengths, and what a failed `clrd` read leaves behind. A comment that misstates why is worth
  correcting and is not a change to the fake.
- **Leaving the confirmation on a run against the real game, as 0001 did.** A run confirms the offsets and
  cannot confirm a frame boundary: nobody can see, from a running game, which frame its supervisor built a
  scene on. The decompilation can be read for exactly that, and the exe's bytes keep the reading honest.
  The real-game run is still what the frame-accounting changes below have to be re-confirmed against once
  they are built — which is what `TODO.md` carries such things in — but it is no longer the only witness,
  and it was never the right one for this.

## What follows from it

Ordered, and **the first step of every item is a scenario that fails.** What has to outlive the work is
here. The fourteen divergences one by one, where in the game's own code each was read so that a finding
which has moved shows up as a diff, and what the reordering costs in expectations, are a worklist in
`docs/todo/` — which is deleted when the work is done, and is why nothing here leans on it.

**First, because one condition settles three findings.** At the moment `stage_numbers_in_place` runs,
`Image::scene()` is still `Scene::Rebuilding` — `build` calls `orb::stage_begun` before it writes the
supervisor's copy — so the game's own `curState != SUPERVISOR_STATE_GAMEMANAGER_REINIT` can be written with
no new plumbing. Into that branch go the score file's read, the reset of lives, bombs, power and deaths,
and the arcade region and player movement area; the other branch is the two lines the game's own is. A
stage transition then carries what a run carries.

**Then the frame a scene is built on.** Drop the early return from `Fake::update`, zero the input word
after `build`, and fall through to the scene's own update — which is the order `Supervisor::OnUpdate` has.
This moves frame accounting at *every* transition, the result screen and the front end included, so it is
where the expectation churn is. `enters_the_ending` does not go through `build`, so the ending's 29,040
updates are untouched.

**Then the stage's first player.** The game writes spawning, 120 frames of invulnerability and a respawn
timer of 8. orb reads both halves — `read_state` carries spawning as `unsettled`, and `make_invulnerable`
deliberately leaves spawning alone — so both are owed: spawning for the respawn frames, then invulnerable
counting down, then normal.

**Then the two cheap ones.** Split the single `reproducing` call so the generator's seed goes in before
`orb::stage_building` and the player's position and `g_Gui`'s draw job after it, which is the order
`AddedCallback` has and closes a trap that is latent only because `stage_building` writes nothing else
today. And write `subRank` and `rank`, which needs setters on `Image` and until then leaves every
assertion about a restored rank comparing zero with zero.

**Then the hooks with no scenario over them.** `init_d3d_device` is always installed in a real launch and
is not in `Originals` at all: the fake writes the device at attach, where production attaches before the
device exists and finds it through this hook. Reaching it means the fake having a phase before its device,
which restructures `Fake::attach` and pairs with giving the fake a startup sequence so that
`creates_its_window` stops being a scenario poking a hook. `get_controller_input` and `save_replay` are
lighter: an `Originals` entry and a scenario each. `hook::install_import` cannot be reached by a laid-out
game at all — there is no import table to patch — so it wants a test of its own over a synthesized one
rather than a scenario, and `memtrack` stays off in every scenario for the same reason.

**Then the numbers.** Four generator draws per shake frame rather than two, and the three cases per axis
the game picks between. The player's movement area at the game's own `(8, 16)`/`(368, 416)` rather than the
arcade region's, since orb reads it into `Reproduction`. A failed `clrd` read leaving what the game's memset
and fixup leave — magic, version, and every difficulty entry at 1 — with the Extra item gated on the
`== 99` the game gates it on, so the mechanism agrees and not only the outcome.

**Then the prose**, which changes no code.

**Last, the chain.** `Fake::update` as a walk over priority-ordered jobs, each answering as a chain
callback does. It is the root fix for the build frame and for the pause menu's early break, and it makes
three things reachable that are not reachable now: what a walk does when a job asks to be removed, what
`Chain::Cut` does to a list, and the fact that `g_Chain` and the static chain elements live in `.data` — so
a chapter's restore rewinds the chain's own linked list, and no scenario has ever asked whether it comes
back.

**What this does not ask for** is anything in the declared-divergence list 0001 already carries. The
generator stays the fake's own, and so do the boss's arrival, the card's start, the stage's length, the
tracks and the layouts. What a scenario is about is that the same buttons from the same seed arrive at the
same place.

## Re-deriving it

The numbers above are stale the moment either tree moves; the commands are not.

```sh
# The decompilation, and the check that it is about this exe and no other.
git clone --depth 1 https://github.com/GensokyoClub/th06.git
sha256sum "$ORB_DEST/東方紅魔郷.exe"
grep -o '9f76483c[a-f0-9]*' th06/README.md

# An address orb holds -> the function it is the start of.
awk -F, -v k=0x41ca10 '$2==k {print $1}' th06/config/mapping.csv

# An address inside a function -> which function, and how far in.
awk -F, -v k=$((0x42f5cd)) \
  '{s=strtonum($2); if (k>=s && k<s+strtonum($3)) printf "%s +0x%x\n", $1, k-s}' \
  th06/config/mapping.csv

# A global orb holds -> its name.
awk -F, '{gsub(/ /,"",$2); if ($2=="0x0069bca0") print $1}' th06/config/globals.csv

# A prologue orb copies into a trampoline -> the bytes actually there.
objdump -s -j .text --start-address=0x41ca10 --stop-address=0x41ca16 "$ORB_DEST/東方紅魔郷.exe"

# The exe's own answer, where agreement with the decompilation would not be independent.
objdump -d -M intel --start-address=0x42f5bc --stop-address=0x42f5e0 "$ORB_DEST/東方紅魔郷.exe"

# Every instruction that touches an address, for a global the decompilation does not name.
objdump -d -M intel "$ORB_DEST/東方紅魔郷.exe" | grep '0x5a5f98'

# A struct offset: add the fields up from the top of the header and check the total against the
# decompilation's own assertion. A total that matches is what makes the fields in between trustworthy.
grep -n 'ZUN_ASSERT_SIZE' th06/src/GameManager.hpp

# And before believing any behaviour: whether that function is decompiled or still a stub.
grep -c 'th06::Chain::RunCalcChain' th06/config/implemented.csv
```

One global is still unnamed. `CURRENT_CARD` at `0x005a5f98` is in no `globals.csv` row, and the exe
touches it only from inside `EclManager::RunEcl` — which is the interpreter that runs a spell card
declaration, so the provenance agrees with what orb reads it as. That it *is* the current card is not
confirmed by name, and a run against the real game or a read of the ECL instruction is what would confirm
it.
