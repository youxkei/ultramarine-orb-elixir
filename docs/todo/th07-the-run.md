# th07's run, and the order it is worth reading in

What is left of [issue 2](https://github.com/youxkei/ultramarine-orb-elixir/issues/2) after the frame,
the device setup and the pad reads: **everything a run is made of.**

**What is left of *that* is one measurement and the confirmations a launch makes**, and the measurement is the
midstage table at the end of this file: a boundary inside a stage is read off a run with `--chapter-tuning` and
off nothing else, so it cannot be walked out of the exe the way every other line here was. Everything else is
in — `Th07::rewinds` answers `true`, so a run of 妖々夢 is chaptered, marked, lost, retried and picked up, with a
chapter at every stage's start and none inside one until that run happens.

**And every one of those has now been played on the real game**, which is the other half of "in": a run
chaptered and retried with its music and its cards, given up, picked up out of its file in a later launch and
landing on the frame that was written down, field for field. What each launch caught is the table below, and the
one thing none of them could produce is the table this file is still open for: `--collect` and `--judge` work
over this game — `a_chapter_table_collected_for_th07.rs` — and what they are waiting for is somebody playing a
Lunatic stage under them.

**So what is left is a launch and nothing that can be written first**, which is these four and each is a
session somebody plays:

- **The midstage table**, above — `scripts/run.sh <dir> --collect` over a Lunatic replay, which is the whole
  word for a tuning pass a replay drives, then `--judge` to step between what it proposed, and the
  `chapters.rs` a pass writes pasted over `crates/orb-core/src/game/th07/chapters.rs`.
- **`frames=` and `script=` on a run that pauses.** Both clocks stop under either menu — the reading is beside
  [`STAGE_FRAMES`] — and a run paused on a stage is what confirms it: neither number moves while the menu is up.
- **`--counted-a-card=96`, twice against a copy of the install**, which is the recipe below: the file's own
  counts climbing by one a launch rather than doubling.
- **Every `read_state` field being the field it is named for**, which is the standing one — an address orb was
  given is an address a laid-out game agrees with whatever the game holds there.

Here rather than in the issue because the measurements below have to be repeatable — every one is a
walk through `th07.exe` that anybody can take again, and a number in a paragraph nobody can re-derive
is one nobody can tell is still true. **This file is deleted when the work is done**, with each thing
in it gone to the place the conventions give it: an address to the constant it is the reason for, a
claim about the game to an e2e test, a decision about shape to `docs/adr/`.

## What the launches settled, and where each of them now lives

Everything below this section was read out of the exe. These were read off the game *running*, which is the only
place they could be read, and each is now an e2e test with the fake made to model the game's own order first —
because in every one of them the laid-out game had been agreeing with orb rather than with 妖々夢.

| what a launch said | where the answer lives |
| --- | --- |
| the input read hands the word to the run a frame late, which is 妖々夢's famous frame of lag | `Game::hand_the_read_to_the_run`, and the three chain priorities beside `G_INPUT_THE_RUN_ACTS_ON` |
| a run picked up out of its file came up with the game's own lives, bombs and power — `resume: not put back` | `Th07::read_state`'s `in_run`: the supervisor's scene lags a run's build by the rest of one update |
| and then played a stage nobody played, parting from the run at its first frame | `STAGE_BUILD`: the seed goes in before the build draws, and `set_run_state` holds the generator still over the reseal |
| and the stage's own song played from the top for the seconds that took | the silence in the fed branch of the input read, put back by the landing's `play_from` |
| 霊夢B's unfinished run was offered with the cursor on 霊夢A | `run_pointed_at` reads the cursor; `G_SHOT_CHOSEN` is written where an item is *decided* |
| 中断データあり was written over the difficulty's own name | `Th07::mark_corner` and `DIFFICULTY_LABEL_TOP`, both read off a screen |
| orb's screens were not in the game's face after all | `OWN_FACE` carries the height and the 2× bake — [0019](../adr/0019-a-game-that-ships-no-font-is-drawn-in-the-machines-own-face.md) |
| a boss beaten left a chapter with nothing to fight | `chapter.rs`, where a chapter the defeat closed is dropped |
| the practice stage select showed a legacy run's scores | `read_what_the_modes_file_records` |

## How to take the walk again

```sh
cp /path/to/th07/th07.exe .            # md5 0126afce1e805370d36c3482445e98da, 650752 bytes
i686-w64-mingw32-objdump -d -M intel th07.exe > th07.asm
```

`.text` is 0x401000, `.rdata` 0x48d000 (file 0x8c200), `.data` 0x49c000 (file 0x9ac00) — a string at a
`.rdata` address is at `va - 0x48d000 + 0x8c200` in the file, and the game's own log strings are in
CP932. Every address in `orb-core/src/game/th07/mod.rs` was read this way and says what identifies it.

## What is measured and not yet written down anywhere else

**The supervisor is 0x575950**, which the frame already reaches for the fog. Its own job on the chain
is registered by 0x439000 — the shape `th06::Supervisor::RegisterChain` has: it makes a chain element
with 0x437c70 as its callback, 0x438986 as its added callback and 0x438de2 as its deleted one, hangs
the supervisor off the element's `+0x1c`, and adds it to the calc chain at priority 0.

**The scene and the scene wanted, and the two scenes a run is, have gone to the code**: they are
`SUPERVISOR_SCENE`, `SUPERVISOR_WANTED`, `SCENE_RUN` and `SCENE_RUN_AGAIN` in `th07/mod.rs`, each with
what identifies it, and `Th07::read_state` answers `scene`, `wanted`, `playing` and `in_run` off them.
`in_game` is answered off them too, once the two bits it turns on were read: the gameplay scene with neither
the attract demo's bit nor the replay's set — `RUN_IS_A_DEMO` and `RUN_IS_A_REPLAY`, both below.

**The scenes the transition switch has cases for** are 0, 1, 2, 5, 6, 8 and 9, at 0x437eac, 0x437ee3,
0x437fb1, 0x437f7b, 0x4381d1, 0x438216 and 0x438249, with a default at 0x438294. Each tears the scene
being left down. Scene 0 sets the wanted scene to 1 and is the startup: nothing is torn down there.

**And the scenes written absolutely**, which is how the rest of the game asks for one: 1 at 0x403881,
0xa at 0x403908, 6 at 0x403b8d, 0x403bfe, 0x404204 and 0x41e7a6, 7 at 0x403bb8. A check at 0x426f60
groups 3, 0xb and 0xc — it loads something (0x498524, 0x200 bytes through the sprite manager) for
every scene that is *not* one of those three.

**And how a scene is built, which is the way to what each number means.** The case for the scene being
*left* holds a switch on the scene *wanted*, and each branch calls the register of the scene being
entered. Out of scene 1 (0x437ee3): wanted 2 registers through 0x42f3c5, wanted 5 through 0x44a302
called on nothing, wanted 8 through 0x43b4db, wanted 9 through 0x42f45d and then 0x41e820. So those
four are `RegisterChain`s, one scene apiece, and the object each hangs off its chain element is that
scene's own.

**Scene 2's object is 0x626270**, off 0x42f3c5: it registers a calc element at 0x62f8b4 and a draw
element at 0x62f8d4, both at priority 2, with 0x626270 as the argument of each — its update is
0x42d8d5, its added callback 0x42e83e, its deleted 0x42f2e4 and its draw 0x42e1d4. The object is at
least 0x95e8 bytes, which is the size of a manager rather than a screen.

**0x626270 is the game manager, which is the next thing to write down.** Its added callback at 0x42e83e
writes
`[this+0x93d7] = [this+0x93d6] + 2 * [this+0x93d5]`, which is the `character * 2 + shot` every table of
either game is indexed by — so **the character is `+0x93d5` and the shot type is `+0x93d6`**, which is
two of `RunStart`'s five. The same callback branches at 0x42e894 on the scene wanted being **3** and
skips the whole build there, which is the shape th06's `STATE_GAMEMANAGER_REINIT` has: 2 is a run being
built and 3 is the same manager re-initialised, which is what a stage transition is. It also stamps
`timeGetTime()` into 0x575ae4 and 1.0f into 0x575ac8, and seeds the generator through 0x4318d0 on
0x49fe20 with the remainder of 0xffff.

**And what that callback builds a run out of, which is where `RunState` will come from.** `[this+0x4]`
and `[this+0x8]` are two objects it allocates at 0x42e937 and 0x42e94e (0x47d441 being the allocator),
and the run's numbers are the second: 0x42ea66 to 0x42eae3 walks `[this+0x8]` and sets `+0x0`, `+0x4`,
`+0x8`, `+0x18`, `+0x28` and `+0x2c` to nothing, the byte at `+0x20` to nothing, `+0xc` to **0x186a0**,
and `+0x30` to **0x32 or 0xc8** — the second where `[this+0x10]` is 4 or more, which is what makes
`+0x10` **the difficulty**. It is *not* what makes `+0x30` a rank, and that reading is corrected in the
table below: the pair 50 and 200 by difficulty is 紅魔郷's rank and this game's first point-value step, and
only the code that moves the field tells them apart. `[this+0x4]`'s byte at `+0x25` is cleared beside them.

So: the game manager is 0x626270, its difficulty is `+0x10`, its character and shot are `+0x93d5` and
`+0x93d6` with the combined index at `+0x93d7`, and the run's own numbers hang off `+0x8` — 0x626278
absolutely, which is how the rest of the game reaches them: 287 places read that address.

**What the caps on those fields say they are.** Every one below is a comparison against a limit that
names the field, which is the cheapest evidence there is for a number:

| field | evidence |
| --- | --- |
| `+0x0`, `+0x20` | **the score the panel shows and the continues used**, which is one row of it: 0x42bdf8 draws `+0x0` with `%.8d` under a hundred million and `%.9d` over it and then the byte `+0x20` with `%1d` one glyph along, which is 妖々夢 showing the continues as the score's last digit. The continue path at 0x4042e0 adds one to that byte, and the ending chooser at 0x41e73f reads it to pick `end00b.end` over `end00.end` — a run that continued gets the other ending |
| `+0x4` | **the score.** 0x42f45d is the run's teardown — it cuts both of the game manager's chain elements and then clamps this field to 0x3b9ac9ff where it is 0x3b9aca00 or more, and copies it into `+0x0`. A cap of 999,999,999 applied as a run ends is a score's, and `+0x0` is the copy a panel's own number chases |
| `+0x14`, `+0x18` | incremented together, each under its cap, by the `__thiscall` at 0x43eb90 whenever `[0x4d44f8]` is nothing — two counters of the same event, and neither is anything `State` or `RunState` carries |
| `+0xc` | **the high score shown.** Drawn by exactly the shape `+0x0` is drawn by — under 0x5f5e100 with `%.8d` at 0x42bf0d and over it with `%.9d`, a `%1d` byte after each — and the callback starts it at 0x186a0, which is the 100,000 these games show before any run has beaten it. A score would start at nothing |
| `+0x14` | `cmp DWORD PTR [eax+0x14],0x270f` at 0x43eba8 — 9999, a counter's cap |
| `+0x18` | `cmp DWORD PTR [ecx+0x18],0xf423f` at 0x43ebcb — 999999, beside the one above |
| `+0x24`, `+0x28`, `+0x2c`, `+0x30` | **the point item system, and no rank anywhere in it.** An item collected at 0x433247 adds a tenth of its value to the score and one to both `+0x24` and `+0x28`; 0x4333bd compares `+0x28` against `+0x30` and, where it has reached it, calls 0x42d83a on the game manager, adds one to `+0x2c` and sets `+0x30` to `(+0x2c - 2) * 0x1f4 + 0x320`. So `+0x28` is the point items collected, `+0x2c` is how far the point value has climbed and `+0x30` is the next step's count — the 0x32 or 0xc8 the build puts there by difficulty is the first step, higher for the Extra and the Phantasm, **which is what made it look like 紅魔郷's rank starting at 50 or 200.** It is not one, and `RunState::rank` is nothing for this game |
| `+0x88` | always `[0x62f88c] - [run+0x88]` (0x4048f7, 0x41b3ff, 0x433170 and on), and that difference is compared against 0xc350 and 0x186a0 — 50000 and 100000, which are score thresholds and not frame counts. So `+0x88` is a score baseline and 0x62f88c is the total it is measured against; the continue path copies the one into the other at 0x4043bf, which is the difference being put back to nothing |
| `+0x10`, `+0x20` | **the continues used**, one for each score above it: the byte the continue path at 0x4042e0 adds one to is `+0x20`, and each is drawn with `%1d` immediately after its own score and one glyph along from it, which is this game showing the continues as the score's last digit |

**What the panel draws, in the order it draws them** (0x42bda0 to 0x42bfa0, four calls of the text
drawing at 0x402060 into the buffer at 0x134ce18, each with the position advanced by a float between
them): `[run+0x0]` with `%.8d` under a hundred million and `%.9d` over it, then `(byte)[run+0x20]` with
`%1d`; then `[run+0xc]` and `(byte)[run+0x10]` the same way. Over a hundred million it writes 1.0 and
0x498b20 into the two floats at 0x13542dc and 0x13542e0 first, which is the ninth digit being squeezed
into the width the eight had.

**The labels beside those lines are not text**: every other constant that function reaches for between
0x42bb00 and 0x42bf60 is a float, so what says which line is which is a sprite and there is no string to
read. That route is closed for every field, and it is worth writing down as closed — the lives and the
bombs were found by the events that move them instead, and everything below took the same way in.

**The power is `+0x7c`, and it is a float.** Three places do the same three instructions — `fld DWORD PTR
[0x626278+0x7c]`, the float-to-int at 0x48b8a0, `cmp eax,0x80` — at 0x415478, 0x42c3e3 and 0x432710, and
the continue path above puts that same field back to nothing. 128 is 妖々夢's full power, so the field
compared against it is the power, and it is held as a float where 紅魔郷 holds an integer.

**Which is the first real material for the question [0004](../adr/0004-th07-is-a-second-game-chosen-at-the-attach.md)
parked**: whether `State` says everything a chapter needs for a game whose resources work differently.
`RunState::power` is a `u16` there, and a `u16` cannot carry this. Whether that matters is a decision to
take with the rest of `RunState` in front of it — the field can be rounded, or the trait can carry what
each game keeps — and it belongs in `docs/adr/` when it is taken rather than settled here in passing.

**The lives, the bombs, the deaths and the count of bombs used have gone to the code**: they are `LIVES`,
`BOMBS` and `DEATHS` in `th07/mod.rs`, floats at `+0x5c`, `+0x68` and `+0x50`, each with the event that
moves it beside it, and `Th07::read_state` answers all three. `+0x6c` is the bombs used and is in that
comment too, being the field that names `+0x68`; nothing of orb's carries it. They were never bytes: the
`movzx` row above is the continues, and a single digit on the panel is a sprite count rather than a byte.

**The three objects the added callback allocates**, with their sizes, which is what says which is which:
`[gm+0x0]` is `[ebp-0x30] + 0x10` bytes, a length worked out from the run — the record of inputs a replay
is, which is what `resume` will want; `[gm+0x4]` is 0x38 bytes, zeroed straight after with a `rep stos` of
14 dwords; and `[gm+0x8]` is 0xc8 bytes, which is the run's numbers and is why fields up to `+0x88` fit in
it.

A field named from a cap or a format alone stays in this file and out of `Th07` — what goes to the code is
what the write that moves it says. That rule is what the frame loop cost once already.

## The stage was counted from the wrong end, and how that was caught

`G_STAGE` went into the code as 0-based on the strength of one reading — the switch at 0x407081 bounding it
at 7 over a table of eight — and **it is 1-based while a stage is played**: that switch subtracts one at
0x40708d before the bound, which is the instruction the first reading walked past. The build at 0x42e83e
adds the one at 0x42ef50 and then loads `data/ecldataN.ecl` and `data/stgNenm.anm` by the result, out of the
tables at 0x49f560 and 0x49f588 whose first usable entry is index 1. So orb reads the word less one, the way
紅魔郷's own 1-based field is read, and `docs/todo` carries this paragraph rather than the code: what the code
says now is what is true.

**What caught it was arithmetic on two addresses that were already in the tree**: the game manager is
0x626270 and the update reads the run-kind word as `[gm+0x93d8]`, which is 0x62f648 — the same word the menu
writes absolutely. Once the manager's base is known to hold these globals as its own fields, `[gm+0x95ec]`
is 0x62f85c, and the loader indexing the ECL table by that field could be compared against what the menus
write into it. Two spellings of one address disagreeing is the cheapest evidence there is, and it is worth
looking for on purpose: the offsets of the manager and the absolute globals are the same numbers twice.

**And the e2e test could not have caught it.** The laid-out game wrote the stage orb reads, so both sides
were one story: 4 in, 4 out, whatever the game itself holds. What catches it is a test written against the
game's own word — `the_games_stage_word_is` beside `is_playing`, with the pairs 1→0 and 8→7 in it — and
that shape is worth using for every field where the game counts differently from orb.

## What the player turned out to be, and what is left beside it

**The player is the static object at 0x4bdad8** and has gone to the code with its state, its bomb flag and
the dialogue's pointer. Two things about it are worth keeping here. Its register at 0x4429d0 fills 0x2df9e
dwords from that address, which ends at 0x575950 — the supervisor's own address, so the player and the
supervisor are neighbours in `.data` and a write past the end of one is the other. And the object holds far
more than the player: `+0x16a30` is the bomb's own object, `+0xb7e5c` to `+0xb7e64` the three chain elements,
and the fields the shot uses run to `+0xb7e6c`.

The two bytes of the manager's own head that looked like a second pair of menu flags are neither, and both
answers are now beside [`GAME_MENU_UP`]: `[gm+0xd]` marks a frame the game's own slowdown did not advance the
run on, and `[gm+0xc]` is read at five addresses and written at none.

## The enemy manager, whose fields the rest of a run will be read out of

**The manager is the static at 0x9a9b00.** Its count, its eight boss pointers, the spellcard pair and the
first main context's clock have gone to the code. The rest of what was read of it is here because the
snapshot and the chapters will want it:

| | |
| --- | --- |
| the slots | 0x1e0 of them from `+0x4f50`, each 0x4f48 bytes, alive being bit 7 of the slot's `+0x2e28`. The update walks all of them at 0x4207c7 |
| an enemy's script clock | `+0x6f0` of the slot: the ECL runner at 0x410520 compares it against the time word at the head of each instruction, takes the opcode from the instruction's `+0x4` less one, bounds that at 0xa0 and jumps through the table of 161 at 0x417838. The instruction's byte `+0x9` is a difficulty mask, ANDed against `[0x626284]` — the game manager's `+0x14`, which is the difficulty as a bit |
| the ECL file | 0x1347938, loaded by 0x40e420 out of the table at 0x49f560 by [`G_STAGE`], with its subs counted in the word at `+0x2` |
| the main scripts | **four contexts of 0x10 bytes from `+0x9545f4`**, run by 0x41f6f0: `+0xc` is the instruction, `+0x8` is that context's own clock, and the instruction set is a different and smaller one — the opcode bounded at 0xc through the table at 0x41fd2f, whose first case is the spawn that copies 0x13d2 dwords of template into a slot and starts its ECL |
| the item manager | 0x575c70, the third big static, whose init at 0x426f60 loads its own sprites and which the bomb calls 0x4326f0 and 0x433a90 on. Its own count is at `+0xae2ec`, kept by the walk at 0x4329e0 over 0x44c slots of 0x288 bytes. Nothing of orb's carries it — it is here because a chapter's snapshot will have to put the items back |

**Which of the four main contexts is the stage's script was the one decision in `read_state`, and it is
taken**: the first, in [0018](../adr/0018-th07s-script-clock-is-the-first-of-four-main-contexts.md), with the
counter at 0x62f898 among the alternatives it beat. What is left of it is a launch reading `script=` on a run
that pauses and one that does not — which is what a chapter's own frame numbers are written in, and the
reading that says both clocks stop together is beside [`STAGE_FRAMES`].

## `RunState`, read but not yet written

**`Th07::run_state` and `Th07::set_run_seed` are in.** Seven of the ten fields are the run's own numbers and
the seed is `[gm+0x95e4]`, the copy the build leaves beside the generator at 0x42f0a4 — which is what "the
seed the stage began with" means, and what a replay's stage record is given at 0x4432e3. The other three are
nothing and say why beside the method: this game counts no power items, pays no extra lives out of the score,
and has no rank.

**`Th07::set_run_state` is in as well**, and the seal came with it: `RESEAL` is 0x4012b0, handed over the way
the frame's own three calls are, and orb calls it after the numbers and puts the seed back *after that* —
because sealing draws twice from the generator. That order is the game's own at 0x443790, and the reason it is
that order is now in the code beside the constant.

**The three counters a resumed run used to be short of are in**, and what they are is 妖々夢's **cherry**:
0x62f88c is the cherry, 0x62f888 is what it is filling towards, 0x62f890 is the cherry+, and all three are
kept absolutely with `[run+0x88]` as the baseline every reader takes off them. What each is was read off what
moves it — the item at 0x42016e and the bomb at 0x440ae7 spend the cherry, 0x43375f tests it against the
second, and 0x4428aa is where the third passing 0xc350 puts a border up.

**`RunState` grew three fields for them**, which was the first of the three options this file used to cost out,
and what settled it is that the struct already holds both games' vocabularies: `rank` and `power_items` are
紅魔郷's and 妖々夢 answers nothing for them, so the trade the other way is the same one. The resume file went
to version 2 with them, so a file written by an earlier build is refused with a line in the log — which is what
that version number is for. `set_run_state` writes the three against the baseline *this* stage has and does the
one thing the game's own restore does rather than writing outright: the cherry+ held to 0xc350 with the border
put up through 0x441960, which is a handed-over call for `handed_over!`'s own reason.

## The walk `set_run_state` has to make, which the game's own replay already makes

`Game::set_run_state` says *the way the game's own replay puts a stage's numbers back*, and **0x443790 is
that function**, the mirror of the recorder at 0x443040. What it does, in its order, is the list to
implement against — and it is the answer to how orb must write these numbers at all, since every write here
is followed by the reseal:

| it writes | from | how |
| --- | --- | --- |
| the difficulty, 0x626280 | `rec[0x57]` of the header | outright |
| the character and shot, 0x62f645..0x62f647 | `header[0x56]` | outright |
| `[run+0x28]` | `rec[0x4]` | outright |
| 0x62f8a4 | `rec[0x25]` | outright |
| the lives, `[run+0x5c]` | `rec[0x23]` | as a float, **then 0x4012b0** |
| the bombs | `rec[0x24]` | through 0x401390, a game manager method taking the count |
| the power, `[run+0x7c]` | `rec[0x22]` | as a float, **then 0x4012b0** |
| `[run+0x18]` | `rec[0x14]` | outright |
| 0x62f644 | `rec[0x26]` | outright |
| 0x62f88c, 0x62f888, 0x62f890 | `rec[0x8]`, `rec[0xc]`, `rec[0x10]` | each plus `[run+0x88]`, and 0x62f890 clamped to that baseline plus 0xc350 with 0x441960 called on the player where it was over |
| `[gm+4]`'s 0x38 bytes | `header[0x70]` | `rep movs` of 0xe dwords |
| the seed, 0x49fe20 | `rec[0x20]` | a word |
| `[run+0x1c]` | `rec[0x27]` | outright |
| `[run+0x2c]`, the rank `[run+0x30]` | `rec[0x18]`, `rec[0x1c]` | outright |

**The score is not in that list**, and that is the one thing to settle before writing `set_run_state`: the
recorder stores it (`rec[0x0]` from `[run+0x4]`, at 0x4431a9) and the loader does not restore it, so a replay
gets its score from somewhere else. Where, is the next thing to read.

**Three of `RunState`'s ten have no address yet**: `point_items`, `power_items` and `extra_lives`. The place
to look is the three globals above — 0x62f888, 0x62f88c and 0x62f890, each kept as a difference from
`[run+0x88]` and compared against 0xc350 and 0x186a0, which are 妖々夢's cherry thresholds — and `[run+0x98]`,
`[run+0xac]` and `[run+0xb0]`, which are the fields the seal is computed from.

**And `+0xc` is the high score, not the next extend**, which is worth writing down because the panel cannot
tell the two apart: the build reads it out of the score file at 0x42e732 through 0x444f0d, and again at
0x42e79c out of a table indexed by difficulty, and zeroes the byte `+0x10` beside it at 0x42e7f8. A field a
run *starts* from the score file with is a record, not a threshold.

## The numbers are sealed, and a snapshot has to seal them again

**Every one of the five guarded adds calls 0x4012b0 on the game manager after its write**, and 0x404fe0
before it is the check that pairs with it: it recomputes what `[run+0xac]` and `[run+0xb0]` should be from
`[run+0x34]`, `[run+0x3c]`, `[run+0x98]`, `[run+0xbc]` and the float at `[gm+0x9614]`, and answers whether
they agree. **Where they do not, the caller fills 0xb4 dwords from 0x575950 with 0xffffffff** — and inside
that range are the device pointer at `+0x8`, the viewport, `PAD_MAPPING`, and the scene and the scene
wanted. So th07 answers a number changed behind its back by destroying its own graphics and its own
transition: it is not a check that refuses the change, it is one that wrecks the game a moment later.

**Which decides how a chapter of this game is put back.** Restoring the whole 0xc8-byte block carries the
sealed fields with it and the check passes; writing lives or a score into it one field at a time does not,
and orb would be the reason a launch went to pieces two seconds after a retry. Whether orb restores the
block, or writes the fields and calls 0x4012b0, or hands the five adds over the way the frame's calls are
handed over, is a decision for the snapshot work — with the reasoning worth keeping, so `docs/adr/`.

## The replay's own records, which are a run's numbers written down

0x443040 is where a replay's header is built, and it is the shortest description of a run this exe has.
The header is 0xe8 bytes off the allocator with `[0x496aa8]` as its first dword and 0x1100 in the word at
`+0x4`; `+0x57` is the difficulty from 0x626280, `+0x56` the `character * 2 + shot`, `+0x70` is 0x38 bytes
copied wholesale out of `[0x626274]`, and `+0x1c` and `+0x38` are seven slots each — a stage's record and
a stage's inputs, 0x70800 bytes of them, indexed by `[0x62f85c]`.

**A stage's record is filled from the run's numbers at the moment the stage begins**, which is the same
set a chapter needs: `+0x0` from `[run+0x4]`, `+0x4` from `[run+0x28]`, `+0x8`, `+0xc` and `+0x10` from
0x62f88c, 0x62f888 and 0x62f890 each less `[run+0x88]`, `+0x14` from `[run+0x18]`, `+0x18` from
`[run+0x2c]`, `+0x1c` from `[run+0x30]`, the word at `+0x20` from 0x62f854, and four bytes taken through
the float-to-int at 0x48b8a0: `+0x22` the power, `+0x23` the lives, `+0x24` the bombs, `+0x25` from
0x62f8a4, `+0x26` from 0x62f644, `+0x27` from `[run+0x1c]`. 0x443790 onwards is the same walk backwards —
a replay's stage being loaded — and it is the list of what a stage of this game starts from.

**The menu a run is stopped from is 0x403b60**, a `__thiscall` on an object of five items of 0x24c bytes
each (fields at `+0x1c0` and `+0x1ce`, a state at `+0x0` and a timer at `+0x4`), which plays 0xa through
the sound player at 0x4ba0d8 as the cursor moves and asks for scene 6 at 0x404204 once that timer reaches
0x14. The continue path above is inside the same function, so this is the pause menu with a continue on
it — and `State::paused` is what its being up says.

**And what kind of run it is lives in a bitfield at 0x62f648**, bits 0 to 4, read in some ninety places.
The pause menu reads two of them: bit 0 sends it to scene **6** and bit 3 to scene **7**, each after
writing `[run+0x4]` into `[run+0x0]` — the pair being equalised as a run ends, which is what a score and
the number a panel is chasing do there. The byte at 0x62f64d is cleared on both paths. Those two scenes
are a second reading of the two bits: a practice run leaves to where its stage was chosen and a replay
leaves to where it was picked, and they are not the same screen.

**Bit 3 is a replay being watched**, which is `State::replay`. 0x455b45 is the load: it reads the file
named at 0x62f654 through 0x431330, takes it through 0x4433b0, logs 0x4959a0 where nothing came back, and
then sets bit 3 from `(1 & 1) << 3` and bit 1 beside it, clears 0x62f650, and fills the run from the
replay's own header — its byte `+0x57` into **0x626280**, which is the game manager's `+0x10`, and its
byte `+0x56` halved into 0x62f645.

**That confirms the difficulty from a second direction**: `+0x10` is what a replay's header writes its
difficulty into. And `+0x56` being halved into 0x62f645 says the header keeps `character * 2 + shot` the
way every table does, so 0x62f645 is the character of the run being watched.

**Bit 0 is a practice run and has gone to the code** as `RUN_IS_A_PRACTICE`, named the way bit 3 was —
by the menu item that sets it. The title menu's eight items dispatch through the table at 0x45622d
(0x455ce4, 0x455e32, 0x455d8b, 0x455ee2, 0x455f95, 0x455f6c, 0x455fbe, 0x455ffd), and of the four that
start something only the third sets the bit; the third item of that menu is Practice Start. 0x45a583 sets
it as well and is the other way into a practice run.

**Bit 1 is the attract demo and has gone to the code** as `RUN_IS_A_DEMO`, with what identifies it beside
it — and `State::in_game` with it, which is the gameplay scene and neither of the two runs nobody is
playing. Two more readers of that bit are worth keeping here rather than in a comment: 0x45bfb1 clears the
replay bit where it is set and then opens `score.dat`, which is a demo ending going through the same tail a
run does, and 0x4018e0 runs a block only during a demo, on `[player+0xa028]` and `[player+0x9e50]`, which
is the way in to the object the player is.

**Bit 4 looks like `Game::run_finished` and cannot be used as it**, which is worth writing down as a trap: a
run ending of its own accord sets it at 0x42a68e, beside marking the table at 0x62e6c4 by difficulty and
counting one into 0x62f52c, and a run *given up* through the pause menu does not — so the bit separates
finished from left, which is exactly what that method asks. **But nothing in the exe ever clears it.** The five
`and` writes to this word clear bits 0, 1 and 3 and none of them touches 0x10, so a second run would read as
finished before it started. What `run_finished` needs is either where the word is reset at a run's build —
which is not one of the writes found — or the result screen's own object, the way 紅魔郷 answers it.

**Bit 2 is the one bit of that word nothing has needed**, and where to start on it if something does: it is
read at 0x439195, 0x4393a0 and three times around 0x442cd9. Bit 4's own addresses are above — set with
`or 0x10` at 0x41e5a3, 0x42a68e and 0x42a6ba, read at 0x446bb7 and 0x448a78.

**And the scenes orb reads are all named**: 1 the front end, 2 a run, 3 a run being rebuilt, 5 the ranking, 6
the screen a run ends at and 9 the ending, each with its reading beside its constant in `th07/mod.rs`. What is
unnamed is 7, 8, 0xa, 0xb and 0xc, and nothing of orb's reaches them — 0x426f60 groups 3, 0xb and 0xc, which is
where a read of them would start.

**And 3 is never a scene orb reads, which the transition's own block settles.** 0x4380b4 writes 3 into the
scene *wanted*, 0x42f45d tears the stage down, 0x42f3c5 registers the manager for the next stage — which
is where the added callback runs and reads that 3 — and 0x4380df writes 2 back into `wanted` before the
update returns, with 0x4382c0 copying `wanted` into the scene at the end of it. So the whole transition is
one update and the pair orb reads either side of it is `2 -> 2`: `playing` is true through every stage of a
run, and what `State::in_run` covering 3 buys is a field of a run in progress being answered for whoever
reads one from inside that update. `crates/orb-e2e/src/a_stage_transition_of_th07.rs` is the e2e test over
it, and `Image::the_next_stage_is_being_asked_for` is where a laid-out transition writes the pair.

## The order the rest is worth reading in

1. ~~**`Th07::read_state`**~~ — **done, every field of it.** What each is read off is beside the constant
   in `th07/mod.rs`, and the thing that came out of doing it is below: the numbers are sealed. The way in was
   the same each time and is worth keeping for the rest: the event that moves the field, not where 紅魔郷 keeps
   the same thing.
2. ~~**The two questions [0004](../adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) parked**~~ —
   **both answered by the run being played.** `State` says what a chapter of this game needs once `RunState`
   carries the cherry as well as 紅魔郷's rank and power items, and the table's shape holds: script frame
   numbers per stage, on the clock [0018](../adr/0018-th07s-script-clock-is-the-first-of-four-main-contexts.md)
   settled, with `Game::midstage_table` answering each game's own `chapters.rs` — seven rows for 紅魔郷 and
   eight for 妖々夢, neither of them a number in the code that reads them.
3. ~~**The snapshot**~~ — **done, the audio with it.** `live_handles` is the sprite manager's 0x108 textures
   at `+0x282ac`, found by the release at 0x44e6f0 whose own two offsets check each other.

   **The audio went in as one piece and the shape of it is the thing to know**: `Music` used to be
   紅魔郷's — a buffer, a countdown and an `HMMIO` — and what 妖々夢 has instead is a Win32 handle on
   `thbgm.dat`, which it opens itself at 0x44bff0 and reads every track out of. So `audio::Track` is now two
   shapes, winmm's and this one, and everything above them works in *differences* of positions, which is what
   lets one rewind serve both. The three objects are the player (0x39cc bytes, its own ctor at 0x44b51a), the
   stream at `+0x39bc` of it (0x78 bytes, allocated at 0x45cb93) and the source at `+0xc` of that (0x94 bytes,
   0x45c933) — and the source is the *streaming* kind, `+0x7c` at nothing, so the position really is a file
   pointer and not memory. `music_identity` is the record the source is reading, `+0x10`, `+0x18` and `+0x1c`
   of it, which is 紅魔郷's own three numbers in this game's terms. `stop_music` and `restart_stage_music`
   stay the game's own two calls, which answer for both sound backends.
4. ~~**The stage's two moments**~~ — **done, and they are not the two 紅魔郷 has.**

   **`stage_begun` is the game manager's added callback, 0x42e83e**, prologue `55 8b ec 83 ec 40` read out of
   the exe's own bytes — the function whose end is where a stage's numbers have just been put in place, which
   is what that seam is for, and orb's own `set_run_state` is called after it returns.

   **`stage_building` is the stage's own build, 0x4074c0**, prologue `55 8b ec 83 ec 08 57` — and it was
   declined first, for a stage's worth of reasoning a launch then refuted. The reasoning was that 妖々夢 takes
   its stage's seed *during* the build (0x42f0a4 copies `[0x49fe20]` into `[gm+0x95e4]`) and that its own replay
   restore puts the seed back at the *end* of a restore, so the seed could go in where the numbers go in — the
   tail of `Th07::set_run_state`.
   
   What that missed is that both of those are *inside* 0x42e83e, and orb's hook over that callback runs when it
   returns: by then the build has drawn the stage out of the generator, so writing the seed there rewinds it
   under a stage already laid out. A run picked up out of its file parted from the run at the stage's first
   frame — `the generator is 0x6167 where the run had 0xe33d` — and 0x4074c0 is where the seed goes instead: the
   twelve instructions before its one call site are the game settling that seed, and its entry is the last
   moment before anything is drawn. `set_run_state` no longer writes the seed at all, and holds the generator
   still over the reseal the way 0x42f07e–0x42f094 does.

   **And the calling convention is why the patch could not simply be declared**: this game's chain calls its
   callbacks `__thiscall`, with the object in `ecx` — `RUN_CALC_CHAIN`'s own walk does
   `mov ecx,[elem+0x1c]; call [elem+0x4]` — while `runtime::stage_begun` is `extern "C"` and calls the
   original with the argument on the stack, as 紅魔郷's `__cdecl` callback wants. Hooked that way, 0x42e83e
   would read `[ebp-0x2c]` out of whatever `ecx` happened to hold, which is a crash and not a wrong number.
   What went in is the shape the pad read already had: `PadRead` became `game::Entered`, which both seams now
   carry, `runtime::stage_begun_in_ecx` is the `fastcall` twin, and the DLL picks between them.
5. ~~**The score file**~~ — **done, `Game::rewinds` with it**: the records are read (`read_practice_scores` through the game's own three calls), the ranking
   screen is asked for, waited on and left, the captures are held across a rewind and across the ranking's own
   read, a card's attempt is counted into both words the game counts into, and the record is resealed after
   every write of orb's — see the two seals below, which are what a count written without one costs.
6. ~~**The front end**~~ — **done, every seam of it**, and what each took is below. `menu_pointed_at` is in: the object is reached by walking the chain for its update at
   0x4554d6 — the register at 0x45c632 keeps its address nowhere else — `+0xd0f8` is which of its fifteen
   screens is up, 0 being the title (the update bounds that field at 0xe and takes the jump-table index out of
   the byte table at 0x4555ce, whose entry for 0 is the only one reaching the title's own handler at 0x4555dd),
   and `+0x0` is the cursor over the eight items. `run_chosen` is in as well — the front end still the scene
   with a run the scene wanted, which the chain's own priorities make a whole frame: the supervisor is 0 and
   the front end is 3, so an item that asks for a run writes the wanted scene after the supervisor has already
   looked. `menu_takes_a_press`, `run_pointed_at`, `run_slot` and the ranking screen's own pair came after it,
   and each is below.

   **`menu_takes_a_press` is in, and it was two fields deeper than the cursor.** The title screen's handler at
   0x4555dd dispatches on its own phase, `[front+0x8]` — 0 at 0x455621, which is the block the item dispatch
   at 0x455cdd lives in, and 1, 2 and 3 at 0x45599d, 0x45612e and 0x456184 — and the gate on the press itself
   is `[front+0xd0fc]`, the frames the screen has been up: 0x455c6d compares it against 0xa and leaves the
   update where it is under, so the ten frames after the title appears take nothing. The press is then a fresh
   one where `[0x4b9e4c] & 0x1001` differs from `[0x4b9e54] & 0x1001` at 0x455c96, which is the game's own
   business and not orb's.

   **`run_pointed_at` is in**, and it is not the title menu's cursor: the trait wants the difficulty and the character
   *already chosen* with the shot under the cursor, which is the shot select screen — and **the fifteen screens
   turn out to be three flows of three.** The byte table at 0x4555ce reads
   `[0, 8, 1, 2, 3, 4, 5, 6, 3, 4, 5, 7, 3, 4, 5]`, so handler index 3 is reached by screens 4, 8 and 12,
   index 4 by 5, 9 and 13, and index 5 by 6, 10 and 14: one triple per way of starting a run, and the same
   three handlers for all of them. Index 4 is 0x457fe5 and writes the character at 0x4593b7 and 0x459475;
   index 5 is 0x459518 and writes the shot at 0x459e7d and 0x459f72, and 0x459e9a — the normal start, which
   sets the difficulty from the configuration byte at 0x575a89 and asks for scene 2 — is inside it. So the
   shot select is the screen orb answers on, and it is three values rather than one.

   **And which triple is which is read**: each title item writes its own first screen — Start writes 4 at
   0x455d20, Practice Start writes 8 at 0x455dc7, and Extra Start writes 0xc at 0x455e77. So the shot select
   is screen **6** for a normal run, **10** for a practice run and **14** for the Extra, and `run_pointed_at`
   answers on the first and the third and `None` on the second, which is what the trait asks for.

   **One trap left in it**: the difficulty is *not* in 0x626280 yet on that screen. 0x459ead is where the shot
   select's own decide copies it there, out of the byte at 0x575a89 — which is where the difficulty select
   before it left the choice. So what `run_pointed_at` reads for the difficulty is 0x575a89, and 0x626280 is
   only right once the run has started. The character and the shot are 0x62f645 and 0x62f646, both written by
   the two screens under the cursor.

   **And there is a trap for it in the stage word**: answered before the build, `[0x62f85c]` is still what a
   menu wrote — 0 for a normal start, the practice menu's own 0-based choice, 6 or 7 for the Extra and the
   Phantasm — while `run_start`, answered after the build, reads the same word one higher. Two methods, the
   same field, read differently, which is [`G_STAGE`]'s own note in the code.

   **`run_slot` is in and names the run** — `lunatic-sakuya-b` — which is 紅魔郷's own shape here and what
   makes a listing of `pointdevice_resume/` read as the runs somebody left. The two orders it needs are read
   rather than guessed: the difficulty is 0x626280's own numbering, `Easy` through `Phantasm`, and the shot is
   `character * 2 + shot`, which is the byte at 0x62f647 every table the game indexes by them is in — the
   player's load at 0x442478 switches on the character and takes `data/player00.anm`, `player01.anm` or
   `player02.anm`. A run whose numbers are none of this game's falls back to the numbers, a name being the one
   thing that cannot be made up.

   **The game's own names are not what orb writes**, and that is deliberate: they are there —
   `ReimuA` eight bytes apart from 0x495b28 downwards, `Easy` twelve apart from 0x495628 — but nothing in the
   exe indexes either table, so which string goes with which index would be inferred from the layout. orb's own
   words for the same indices say the same thing and are read out of the code that uses them.
7. ~~**The panel**~~ — **done, and the piece whose numbers were not all in the exe needed none of them.**

   `panel_tile` takes the tile out of the vm the game lays its own background with rather than out of a
   sheet layout written down here: `[vm+0x1e4]` is the sprite that vm's script put on it, `[sprite]` is the
   slot of the texture — the same `ANM_TEXTURES` array `live_handles` walks — and four floats of it are the
   piece of the sheet. All four addresses come out of the draw at 0x44f770 and 0x44efb0, so the *file* is
   only ever a check on what the game answers.

   The check was taken all the same, and it is the one measurement here that is not a walk of the exe:
   `th07.dat` is a `PBG4` archive, its file table and each member LZSS with a 13-bit offset and a 4-bit
   length, and `front.anm`'s first entry is `data/front/front.png` at 256x256 with fourteen sprites — sprite
   12 the 32x32 tile at (0, 224), sprite 10 the 16x16 life mark at (64, 80), sprite 11 the bomb's beside it.
   Its scripts are not its sprites in order: script 9 draws sprite 10 and script 10 draws sprite 11, which is
   what ties the lives' vm (`GuiImpl+0x14ac`, global script 0x609) to the 16x16 mark. `lives_row`'s own
   corner and step are in the exe — the loop at 0x42bc5f, from 0x498c50 (496.0) by 0x498a80 (16.0) at
   0x498c54 (96.0) — and the anchor is the corner because the same instruction lays the tile grid from
   (416, 16) in 32s.

   `repaint_lives_row` writes **two** things and 紅魔郷 writes one: the lives' pair of bits in `[0x49fbf0+4]`
   asks for the marks, and 0x575ab4 asks for the ground under them, this panel having no erase bar of its
   own. `draws_lives_row` is the panel's own draw job being on the draw list, 0x427f22 off the chain's
   `+0x20` list.

## `Game::rewinds` answers `true`, and what it took

**The tie between the answer and the tuning run is worth writing down**, because nothing about it is obvious:
`Runtime::chaptering` is `config.chapters && mode == Pointdevice`, and the mode is Pointdevice only where
`rewinds()` is true. Nothing observes a chapter otherwise — so a `--chapter-tuning` launch of a game whose
`rewinds()` is `false` **writes nothing at all**, and the midstage table could never be measured. The answer
comes before the table, not after it, and the table's entries are empty rather than absent so that a tuning
pass has one line to write per stage.

**What the answer rests on is `crates/orb-e2e/src/a_pointdevice_run_of_th07.rs`**, which plays a run of a
laid-out 妖々夢 through the whole of the mode: the question over the title menu, the answer on the keyboard,
a run built, the stage's first chapter at frame 249 — a snapshot of 14MB of this game's own data — the mark
over the lives with the panel's own tile in the strips, a death, the retry menu read off the screen in the
face the machine lent, and the chapter put back with the numbers it had. Three things the laid-out game
needed for that, each of which is what a real one does:

- **its own calls handed over**, all of them and not only the frame's three: the score file read a mode
  chosen at the menu makes, the seal after a chapter's numbers, the border the cherry+ earns, the two music
  calls and the baking of a card's name. A launch that left one unanswered jumped into the real game's
  addresses — which in a laid-out one is memory nothing has mapped, and the child process died with no output
  at all;
- **the keys**, mapped to the word 妖々夢's own read builds — shot 0x1, bomb 0x2 and the four directions from
  0x10 — because orb decides a press off the word the *game* read and a game whose read answers nothing sees
  no press;
- **the two clocks moving**, which is `Fake::the_run_goes_on`: orb waits a stage's settling out before the
  first chapter of one, so a laid-out run whose frames never moved is a run no chapter is ever taken in.

And one thing the laid-out game had wrong: an object laid out and left at nothing has its dialogue's line at
*zero*, which every reader takes for a line being played — so a stage with a panel had a dialogue up on every
frame and no chapter was ever due. The load's own -1 goes in with the object now.

## The e2e sweep: what 紅魔郷 is asked and 妖々夢 is not yet

**Every e2e test over 紅魔郷 is a question that can be put to 妖々夢 now**, and the ones already put are the
files with `_of_th07` in the name beside `th07.rs` and `a_pointdevice_run_of_th07.rs`. What each of those took
is written in it; what the rest will take is the same three kinds of thing — a laid-out object the image does
not lay out yet, a call of the game's own handed over, or the game's part played frame by frame in the fake.

Done: the mode question and the whole of one run (`a_pointdevice_run_of_th07.rs`), the card's name across a
restore, the music across one and which track a chapter gets back under a changed track, under the song the
stage was come into with, and inside the restore's own frame, the mark over the lives with the panel's own tile
and
without it, the score file per mode with the front end's own read left in the game's file, the pad by word and
by number, the letterbox, the frame's order and its queue of quads, the fog, the scene, the stage and the run's
own numbers, the chapters a fight causes and the card that names itself late, a chapter begun under a bomb,
更に前からやり直す and the chapters listed behind the one that was lost, the two confirmations declined, a region
a restore cannot write, the handles a restore leaves alone and the chain's own list, `--self-check`, what a state
read back says of a run and of a replay, a practice run, a legacy run and the question that chooses one, a stage
transition, the frame a scene is built on and the input words it clears, an ending run out and its staff roll left
to play, the screen a finished run ends at, the stepping keys inside a stage and between a replay's stages, orb's
own screens in English, the mode question in every way it can be answered and cancelled, the same question and the
retry menu on a pad, the window orb makes and the two it hands straight on, the window going behind and the keys
that are dropped there, keys another program sent, the mouse pointer, the cadence a run of it settles at on four
shapes of display, `--clear` and the two fields that keep its player alive, `--collect` and `--judge` over its own
waves, and the resume: a chapter written down off the frame, the run picked up out of that file, and the ten ways
the file or the stage behind that question is declined.

**Three things the sweep found, every one of them a hole rather than a difference**, and each is fixed with the
e2e test that found it:

- `Th07::run_start` answered `practice: false` outright, so a practice run of 妖々夢 was written down as a whole
  run and offered back. The bit is [`G_RUN_KIND`]'s bit 0, which `read_state` already read.
- `Th07::menu_takes_a_press` answered only for the title menu, so the press at the shot select was never held
  back and the question about a run left unfinished could never be asked: 妖々夢 wrote a resume file that nothing
  could ever offer.
- `Th07::pad` answered `Pad::default()`, so **no question of orb's could be answered on a pad** — the mode
  question, the retry menu, the list of chapters behind one and the resume question, all of them keyboard-only in
  a game whose own menus a pad drives. What it answers now is the reading orb sampled put through the game's own
  mapping and tested against the two masks its front end reads a decide and a back with.

And two claims in the code that were true when they were written and are not now: `acquire_input` and
`take_sent_keys` each said what they said because `Hooks::input` was `None` and orb put no question over this
game's menus. The reason both answers are still right is the read itself — 0x430b76 is `GetKeyboardState` — and
that is what they say now.

**And nothing of 紅魔郷's own files is left to port.** Three of them turned out to be asked of this game already
rather than portable:

- `the_process_orb_woke_up_in` walks the whole table — every entry, every build named, and every one found by its
  own exe — so 妖々夢 is in both of its assertions.
- `mode_on_a_winmm_pad` is the branch where the game has no DirectInput device of its own, which for 妖々夢 is
  *every* launch: the pad orb's menus and its word are asked about in `the_mode_of_th07_on_the_pad.rs` and
  `th07.rs` is winmm's joystick 0 sampled on a thread of orb's own, which is that file's device.
- What a replay's own record does across a stage teardown is a question this game answers by not having it:
  `Th07::hooks` declines `stop_recording` because 妖々夢's teardown writes no entry and its recorder refuses a
  frame during playback, which is written where it is declined.

And `a_chapter_out_of_the_table` has nothing to read here yet by design: `Th07::midstage_table` is eight empty
rows until the pass in `a_chapter_table_collected_for_th07.rs` is run against a real replay, which is the
measurement at the end of this file.

**And the last two of 紅魔郷's are settled**, both about the game's own player:

- **`the_player_a_stage_starts`** asserts 紅魔郷's spawning branch turning 120 into 240. 妖々夢 does the opposite
  and `the_player_a_stage_of_th07_starts.rs` is that read written down: the register every stage's build runs
  leaves the state a run is played in with nothing on the timer, and the hit test reads no count of frames at all,
  so a stage of this game begins with a player anything can kill. What the e2e test holds is what a build leaves
  in the two fields `--clear` writes; how long the player is unkillable for is nothing, and there is no test to be
  written about a length that does not exist.
- **`the_dpad_moving_the_player`**'s second half is the game's own arithmetic — a step per frame — and stays
  unmodelled for the reason [0008](../adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md) rejected
  modelling the pad's merge into the input word: where an axis becomes a direction is the game's business and not
  a thing to reimplement twice. The half that is orb's — the hat's direction reaching the word the game's own read
  answers, which is `dpad_moves` — is asked of this game in `th07.rs`'s own pad test, on both of the devices a pad
  can be read on.

## The crash on the way out of a run

**Settled**: the pump orb ran to build the ranking is gone, and orb waits for the game's own save instead — the
whole of it is beside `SAVE_KIND_WRITES` in `th07/mod.rs`, which is where somebody reaching for that call next
will read it. What is below is how the fault was found.

**Where three launches got to, in the order the log has them**: a practice run of the stage the log calls
`stage=5`, a death, a chapter put back, the run ended — and then, while the ranking that write needs is being
built and taken down, `crash: code 0xc0000005 at 0x00450db1 in th07.exe+0x50db1`, reading an address ending
`…168` each time and a different heap block each run.

**What that address is**: 0x450db1 is `movsx edx, WORD PTR [ecx+0x4]` with `ecx` loaded from `[arg+0x1e0]`
at 0x450d9c, inside the script walk at 0x450d60 — whose own first act, at 0x450d73, is to return where
`[arg+0x1e0]` is null. So the faulting read is `[[arg+0x1e0]+4]`, the script of the thing it was handed, and
the pointer it went through is not null and no longer memory the process can read.

**And it is not the restore, which two runs settled.** The third fault came with `--no-chapters` — orb loaded
with none of its own work happening, no chapter taken and nothing restored — so the list a restore puts back is
not what leaves that pointer stale. What the same run answered as well, `orb_core::crash` now writing the
registers and the callers up the frame-pointer chain:

```
crash: reading 0x17f6b16c, which is not memory the process can read
crash: … ecx=0x17f6b168 … edi=0x00626218        ← edi is G_CHAIN
crash: called from th07.exe+0x550ea             ← inside 0x4550c0
crash: called from th07.exe+0x55585             ← inside 0x455170
crash: called from th07.exe+0x2fd91             ← inside RUN_CALC_CHAIN
crash: called from orb-….dll+0x952e1            ← orb, four frames of it
crash: called from th07.exe+0x341fc             ← the game's own loop
```

**So the fault is inside a walk of the chain that orb asked for**, and the three lines above it in the log say
which walk: `score: the ranking is up`, `score: asked it to leave`, and then the fault. That is
`runtime::commit_records`, which asks for the game's own update again and again inside one frame to have the
ranking built and taken down with nothing drawn. The call that faults is a screen handing the anm manager a vm
it holds at `+0xb0c8` — 0x4555a0, guarded on that field being non-zero — so the screen is there and the vm it
points at is not.

**One thing about that pump was wrong and is fixed**: it ignored what the walk answered, and 妖々夢's own frame
*returns* on two of those answers — 0x4347f1 and 0x434801 test them, 0x4347fc and 0x43480c return — so a walk
asked for after one of them is a walk no frame of the game's own makes, over a chain whose scene is being taken
apart. The pump now stops at `CHAIN_EXIT_ERROR` and says so:
`score: the walk gave up after N update(s); nothing more is asked of it`. Held by
`th07.rs`'s `the_ranking_of_a_run_of_th07_is_built_and_the_walk_stops_where_the_games_own_frame_returns`, which
was watched failing at three walks past that answer and then at one, and which is also the first e2e test over
this game to reach `commit_records` at all — the hole the fault fell through.

**And then the pump stopped being asked for at all**, which closes this route rather than fixing it:
`commit_records` returns on `Game::writes_its_own_records` before it walks anything, and 妖々夢 answers that
`true` — so no launch of this game asks the chain for an update again, and the walk the fault was inside is one
nothing makes. The line to look for now is
`score: this game writes its own file; what the run counted waits for the save it makes itself`.

What is kept above is the reading, for a fault that comes back from somewhere else: `--no-chapters` as the
bisect that took the restore out of it, `orb_core::crash`'s registers and callers as what named the walk, and
the three things the game's own frame does around a walk that a pump of orb's would not — 0x434794's own call,
`[ecx+0xa0]` at 0x4347d4 and `PLAY_SOUNDS` at 0x4347ec.

And if a fault of that shape does turn up again:

- **`--self-check --stress=600`**, on that same practice stage. It restores every snapshot as it is taken and
  names every region that does not come back byte for byte. `--no-memory` beside it is the bisect: a snapshot of
  `.data` alone, so a fault that goes away with it is a heap block the restore writes back.
- **What frees the vm the screen still points at**, read off the exe: 0x455170's own object is the screen and
  `[0x4b9e44]` the manager it hands the vm to, so what to find is where that field is cleared and whether the
  teardown clears it before or after the job is cut from the chain.

None of these is an `orb.yaml` key: the fault options are command-line ones, `crates/orb-config/src/args.rs`
under its own help heading, and `scripts/run.sh <dir> [option...]` passes them straight through.

## Why the ranking is built at all, and what it would take not to

**Settled**: it is not built by orb at all any more. `Game::writes_its_own_records` answers `true` for this game,
so what a session counted is put back over the file's own records at the parse the game's own save makes — see
`Th07::put_back_the_cards_a_run_left` and `its_records_are_about_to_be_written`, and a launch's
`score: card 91 now 126 …` line. What is below is the reading that led there.

**Read because the pump that builds it is where the fault above lives**, and the question was whether 妖々夢
needs it: 紅魔郷 writes its score file only as the ranking screen goes down, and if this game wrote its own on
the way out then the pump could go. It does write on the way out, and that write is **not** the one the
session's counts reach the file through.

The whole of it, in the game's own terms — `[obj+8]` is the phase field, and it is the very number orb reads
back as `ranking_state` and writes as `leave_ranking`:

| where | what |
| --- | --- |
| `0x44a302(kind)` | allocates 0xce6c bytes, constructs the score-file object (0x444a5b) |
| `kind 1` | `[obj+8] = 0xa`, or `0x12` where the practice bit at `[G_RUN_KIND]&1` is set, then registers the job 0x445db3 on `G_CHAIN` — the screen's own flow |
| `kind 2` | `[obj+8] = 0x13` and calls 0x449b05 inline. Reached once, from 0x434277, right after the message pump ends: **the save on the way out** |
| 0x449b05 | reads `score.dat` (0x449f82), then **parses its `CATK` records over `G_CARD_RECORDS`** (0x44a059 → 0x445069, a `rep movs` of 0x78 bytes per card) — *skipped* where the phase is 0xa or 0x12 (0x44a040, 0x44a049) — then walks those records per shot into the object (0x44a145) and, **only where the phase is 0x13** (0x44a1e0), writes the file (0x44a1e9 → 0x44a1f9 → 0x44552c → 0x431540) |

So the phase decides everything: 0xa and 0x12 keep what memory holds and write nothing, and 0x13 writes and
takes the file's own records over memory first. **The save on the way out is 0x13**, which is why a session's
counts do not reach the file through it, and why the screen's own phases are the only path that has them.

**And the rest of that machine is now read, which says the pump can write nothing at all.** Every address below
was read off the exe:

| where | what |
| --- | --- |
| 0x445069 | the parse, `(ecx = the file image, edx = an array)`: walks the image's chunks, and for every `CATK` whose version byte is 1 and whose card number at `+0x28` is under 0x8d, `rep movs` 0x1e dwords from **the chunk into `edx + card * 0x78`**. `edx` is 0x626288 at its one call site, which is `G_CARD_RECORDS` — so the parse is **file → memory**, whole records, unconditional |
| 0x44552c | the write, whose one caller is 0x44a1f9: it serialises out of **0x626288 itself** (0x445713), so what reaches the file is what memory holds |
| 0x44a1f9 | writes only where the object still holds a file image (`[obj] != 0`), then frees it and clears the field |
| the callers of `0x44a302(kind)` | five: `ecx = 2` at 0x434277 (after the message pump — the way out) and at 0x42e646; `ecx = 0` at 0x437f3a; `ecx = 1` at 0x438044 and 0x438283 |
| 0x42e634 | the second `kind 2`: it saves, then **zeroes all 141 records** (`rep stos` of 0x1086 dwords over 0x626288, which is 0x8d × 0x78 exactly) and rebuilds each one's `CATK` header with its card number and no counts. Its one caller is 0x42eae7, inside [`GAME_MANAGER_ADDED`] — so a run being built is what empties them |
| 0x42f94c | not a merge, though it sits between the parse and the write: it counts the records whose last word is set and unlocks at 0x3c of them |

So the write's source is memory and the parse overwrites memory with the file immediately before it: **a `kind 2`
save is a round trip that cannot carry anything, and no phase writes without it.** Which settles what the pump
is worth on this game — the screen's own phases are 0xa and 0x12, neither writes, so orb driving that screen to
its end writes nothing whatever it does. The counts a session made are lost at the next run's build, which
empties the records, or at the way out, which round-trips them.

**That is built, and the pump is gone with it**: `Game::save_its_records` asks for the `kind 2` call where a run
has ended, `Hooks::card_records_read` is the parse hooked, and what the run counted is written down on the frame
it ends and added on top of the file's own records inside that save. `the_cards_a_run_of_th07_counted.rs` holds
the whole of it against a laid-out score file, and `no_ranking_is_built_for_a_run_of_th07_that_ended` holds that
no update of the game's own is asked for any more — so the fault in the section above has nothing left to happen
inside.

**The two things that were only reasoned are read now, and the first of them was wrong.** The parse has three
call sites, not one — `grep -n 'call   0x445069'`:

| where | what |
| --- | --- |
| 0x42e735 | inside 0x42e634, **after** it empties the records: the run's build reads the file straight back into them. So a run's own records are the file's cumulative *with* that run's counts on top, and adding them to the file's again counts the history twice |
| 0x44a059 | the save's own, which is the seam orb puts the run's records back at |
| 0x45bff3 | inside 0x45bf15, the read the front end makes for its menus: the same parse, over the same array |

So what orb does at that seam is a **put back** and not an add — `Game::put_back_the_cards_a_run_left`. The wrong
one passed until the laid-out build read its file the way 0x42e735 does; with that in, it came out as
`left: (15, 7)` against `right: (8, 4)`, which is the history doubled.

**And the guard is held by a test now**, which is the third call site's doing: the note is let go of only where a
write follows the read it was put back at, since a read that writes nothing leaves what the run counted in memory
and nowhere else. The laid-out front end runs that callback where the game's does — `the_front_end_reads_the_file_for_its_menus`,
at the title a given-up run comes back to as well as the one it started from — so with `A_WRITE_FOLLOWS` taken out
`the_cards_a_run_of_th07_counted` fails on the file holding `(7, 3)` where the run counted `(8, 4)`.

**And a launch says the whole of it without a run being played to a spell card**, which is what
`--counted-a-card=<card>` is for: it counts one attempt and one capture against that card, notes them the way a
run ending does and asks for the same save. So the write and the read back are one command apiece, against a real
exe and a real file:

```sh
cp th07.exe th07.dat th07.cfg score.dat target/th07-trial/   # a copy, never the install somebody plays
cargo xtask build --release && cp target/i686-pc-windows-gnu/release/orb.exe target/th07-trial/
cd target/th07-trial && ./orb.exe --log=verbose --no-chapters --counted-a-card=96   # twice, killing it between
```

What to read is `score: card N now A attempt(s) and C capture(s) for shot S, where the file had a and c`, once a
launch: the second launch's `where the file had` is the first's `now`, `A` climbs by one each time rather than
doubling, and `the records this run left put back over 1 the file's own differ in` is the one record moving and
the other 140 standing. `--no-chapters` because the fork sends a pointdevice launch's opens to
`pointdevice_score.dat` while the front end's own unlocks read still comes out of the game's file, which mixes
the two files' numbers in that line.

**A run somebody plays cannot be driven from a script, and the reason is [`Game::take_sent_keys`]**: 紅魔郷
answers it by letting its own keyboard device go, so `--sent-keys` makes a session drivable with `SendInput`;
妖々夢's answers `false`, so synthesized keys reach nothing — a launch with `--sent-keys` says
`the game had no keyboard device to let go of` and its own demo plays on through every key sent at it. Which is
why the flag above exists: what it takes out of the check is the playing, not the game.

## What cannot come before a launch, and so is not "the implementation"

- **The midstage table.** *Building the midstage table* in `SPEC.md` is a procedure of playing the game
  with `--chapter-tuning` and reading the boundaries out of what it writes. There is nothing to write
  down before that run happens, which is why it is last here and why the table is a file of its own when
  it arrives.
- **Every `read_state` field's meaning.** The e2e tests can say orb reads the address it was given; that
  the address is the field is what a launch says, and a laid-out th07 is written from the same constants.
  So each field lands with an e2e test *and* wants a launch behind it — which is the same rule the frame
  loop cost once already.

## The bot, which is how the launches above stop costing somebody an afternoon

`--autoplay=<recipe>` hands orb a recipe and orb plays the run to it: `press`, `play_until`, `wait_until`,
`quit`, whose grammar is `orb-config`'s `recipe` and whose walker is the `orb-bot` crate. **Off unless that
argument is given** — there is no `orb.yaml` key for it, so no launch of somebody's own can turn it on by
accident — and what it presses goes in where a playback's own recorded words go in, so a run it played leaves
the same score file and the same ranking a run somebody played leaves. Anything less is not a run to measure a
resume against.

The rules 妖々夢 is played by are in `crates/orb-core/src/game/th07/bot.rs`, each with the e2e test in
`the_bot_playing_th07.rs` that pins it. **What no laid-out game can say is whether they clear the game**, and
that is this section: a launch says it, and a run that dies says which rule to look at.

### What is left between the bot and a run it is never hit on

**The hit log is the instrument, and reading it is the whole method.** `autoplay: hit at …, having pressed …,
with … of room — nearest was …` is written on the frame a hit comes out of, and the size of the room tells the
three kinds of wrongness apart: twenty-odd units of room is a hazard the model does not have at all, four or
five is a model that has it and misses by less than a step, and none at all is every move being shut. Every
fix below was found that way and no other way.

What has been closed, each now standing beside the thing it is about rather than here:

- **A bullet's four motion branches**, read and ruled out: 0x425ccd adds `+0xb98` whole, 0x425faf and 0x426287
  add half of it and 0x42619f a third, so none of them moves a bullet further than the projection does. There
  is no turning branch, and a straight projection is never late.
- **A bullet's own size**, which is per kind and written at the spawn — 4, 5, 6, 8, 10 and 24, the last at
  0x4275ca. The widest is what sets `LOOK_AHEAD`, and its reasoning is the constant's own comment.
- **An enemy's body and its motion**, and **a laser's turn**, both by remembering last frame: neither an enemy
  nor a laser has a step or a rate anywhere on it, so the difference from last frame is the only reading there
  is. `bot::moved` and `bot::turned`.
- **A boss's slot outliving its boss**, which cost 妖々夢's 4面 22505 of its 38820 frames against the left
  wall. `Th07::boss` carries it.

**And what a hit turned out to mean most often is not a hazard missed but a hazard invented.** Three of the
readings above are the game saying a thing *cannot* touch the player, and each one had been costing hits by
making the bot dodge it:

- **An enemy with no body**, the low two bits of `+0x2e29` (0x420e86, 0x420e9f). 6面's boss fires 自機狙い off
  one its script keeps at the player, so dodging it is a corner with no way out — three hits in one chapter
  reading `nowhere open to it`.
- **A bullet still coming out**, which is any state but the first of the five at 0x426add. Most of a wall of
  bullets on the frame it is fired.
- **A 予告線 before it fires**, `+0x4e0` against `+0x4c4` (0x4266dc). The one lane a laser pattern leaves open
  is the one the warning is drawn across.

So the shape to look for in a `nowhere open to it` is not a missing hazard: it is one of these.

**Two questions are open.** Which of 妖々夢's 4面 bosses the fight is *with* — three share the stage and all
three are on the screen for the last cards, so standing under the first slot inside the field is standing under
whichever it happens to be. Nothing has been read that tells them apart; `ENEMY_LIFE` is the candidate and no
launch has confirmed the decoys have none. It costs time rather than hits — an attack that is not being shot
times out — so it is behind the other one. And **the bottom edge corners the bot the way a side did**: a hit at
`293.8,432.0` with nowhere open to it, ninety units clear of either side and sixteen off the bottom.
[`IN_FROM_THE_SIDE`] holds the x the bot *waits* on off the sides, and the y is held by
[`ABOVE_THE_BOTTOM`] — but a dodge goes where the field allows, and a pattern that pushes the bot down for
several frames leaves it somewhere three of the nine moves are the wall.

**And the shape of that second one is the shape of the whole rule.** Every question the rules ask is *is this
move hit*, and none of them is *is where it lands a place to live* — so a move clear for every frame looked at
can step into a pocket the frame after closes, and walking into the dense part of a pattern is never counted
against a move. Which is what those two hits are: not a hazard missed but a place chosen.

The cheap version of the answer was tried and does not do it. Each of the nine landings can be asked how many
of the nine moves *from there* are open, one frame on — nine by nine is the work nine by [`LOOK_AHEAD`] already
is, so it costs nothing. But it cannot be *ranked* above the wanting: five of the bot's own tests failed at
once, and what they were saying is that a bot which steps whenever a neighbour is a shade freer is a bot that
fidgets and never arrives. Ranked below it, as a threshold on how cramped is too cramped, the number has to be
guessed — and the place those two hits came from had four or five of the nine still open two frames before
there was nothing.

So the real answer is not a term in this ranking. It is to search a few frames wide rather than one move deep:
the bot decides again every frame, so what a move is worth is what the *best* move after it is worth, and that
is a different shape from nine one-frame moves each followed in a straight line. That is the next piece of work
on the rules, and it is worth doing against the same instrument as everything above — a hit reading `nowhere
open to it` is this, and one reading a unit or two of room is [`ROOM_TO_SPARE`]'s lead below.

**What is left of the projection, and it is one lead.** `ROOM_TO_SPARE` is two units of slack on top of the
game's own box test,
and it is there because twenty-five hits in a row read between half a unit and one of room — a projection out by
about a unit with nothing naming the source. Two of the three places a bullet's step is rebuilt have now been
read, and one of them is that source:

- **0x4258a0**, on `[bullet+0xbf4] & 0xc00`, rebuilds the step from the angle at `+0xbbc` and a speed of
  `[+0xbb0] × (1 − t / [+0xd30])` — a bullet that slows to a stop. A straight projection puts such a bullet
  *further* than it will be, so it is early rather than late, and safe.
- **0x425700**, on `[bullet+0xbf4] & 0x400`, negates that angle and rebuilds the step from the speed at
  `+0xd48`. **A bullet that turns round**, and a straight projection of one is wrong about its direction and not
  just its distance — which is the shape of a hit the model called clear. The frame it turns on is readable:
  `+0xd5c` counts up and 0x425a23 compares it with `+0xd60`.

So the work is the same shape as the 予告線's: trust the straight line for the frames before the turn and treat
the frames after it as what they are, which is nothing known. And the same instrument decides whether it was
worth it — a hit reading about two units of room is this, and one reading twenty is a hazard still missing.

### The lasers, and what reading them settled

`bot::steer` reads all three now — every bullet, every enemy and every laser — and a beam is judged in its own
frame rather than as a point: the landing turned about the origin is *along* and *across*, which is the shape
the player's own test at 0x43e6b0 makes of it. The angle it will have is the angle it has plus as many of last
frame's turns as there are frames to look, `bot::turned`, since nothing in a laser's update moves its angle and
so there is no rate to read. The table below is the reading the code stands on, and every row of it is a
constant in `game/th07/mod.rs` now.

Out of the two walks of the 0x40 slots of 0x4ec bytes at 0x995f80:

| what | where | how it was read |
| --- | --- | --- |
| alive | `+0x4d4` | what both walks test — 0x418b80 and 0x42483d |
| the origin | `+0x498`, three floats | 0x418cec hands `laser+0x498` to the geometry at 0x4185d0 as the beam's base |
| the angle | `+0x4a4` | 0x4248db hands it to the sin/cos at 0x417af0 |
| along the beam | `+0x4a8` near, `+0x4ac` far | 0x418be0 takes their difference for a length and 0x418c04 their midpoint |
| how wide | `+0x4b4`, out of `+0x4b8` | 0x4248ac copies the one into the other |
| which phase | `+0x4e8` | both walks skip a laser whose phase has reached 2, and 0x424877 sets it to 2 |
| a flag word | `+0x4e4` | 0x42484b tests bit 2 of it against the reason the walk was made |

**What is not read is the one thing that matters**: the test a laser hits the *player* by. Both callers of the
geometry at 0x4185d0 — 0x4189ed and 0x418cec — hand it a *bullet's* position at `+0xb8c`, so both are a laser
against a bullet and neither is against 0x4bdad8. Which leaves two possibilities and no way to choose between
them yet: the player's own test is somewhere these two greps did not reach, or **a laser reaches the player as
one of the bullet slots' own five states**, in which case the box test at 0x425e89 already covers it.

It was the first: **0x43e6b0 is the player's own laser test**, and 0x426713 and 0x426727 hand it a laser's
origin and angle, which it turns the player into the beam's frame with through 0x4319b0. So a beam is its own
hazard and not a bullet wearing one, and the bot reads it as one.

**妖々夢 has two kinds**, and one reading covers both: the beam about to fire is thin and the beam firing is
wide, so the width it has *now* makes the warning line a hazard exactly the size it looks — nothing has to know
which of the two a slot holds. What tells a beam that is done from one that is not is the phase at `+0x4e8`,
which both of the game's own walks skip at 2.

### What the parting has been narrowed to, and the one measurement left

**A stage picked up plays a different world from the frame it starts on, and the generator is where it shows.**
Measured over five pairs of launches of 妖々夢, each one a run played from nothing into 2面 and left for the
title at its midboss, then picked up again:

- **1面 reproduces exactly.** A file written at 1面's chapter 9 replayed 9619 frames and landed *field for
  field*. So neither the buttons nor the playback's own machinery is at fault.
- **2面 parts every time**, and always the same way: an item collected on one pass and passed through on the
  other — `counted` up by one and the score by 200 on the run's side, four more numbers drawn — with the frame
  it happens on moving between pairs (803, 824, 827, 853, 935, 993).
- **Everything a frame's line carries agrees up to that frame**: the word, the player's own position, the
  generator, how many numbers it has drawn, the items in the air, the score, the three cherry counts, and how
  many updates the frame took. The file is written and read back identically, field for field. The bot's word
  does not overwrite a playback's — the fed buttons are returned above it.
- **The parting is an item's own homing mark**, `+0x27f`, which 0x432cd2 writes. Folding an item's whole slot
  found the two passes differing 223 frames before the generator did; folding the item's own fields alone —
  a slot's first 0x24c bytes are a sprite, which a playback never advances because it never draws — found them
  differing at the block that mark is in and nowhere else.
- **The only input to that mark is 0x432c74's comparison of the player's y against the line items come in
  above**, and both agree on every frame: the line never differed across 2826 frames, and the player's y is in
  the line that agrees.

**And the frame a stage starts on agrees.** The first frames of one pair's 2面, run against playback, with the
word each was seeded from beside them:

| | the run | the playback |
| --- | --- | --- |
| the stage's seed | `seed=0xa71a` | `seeded with seed=0xa71a` |
| frame 2 | `randoms=1496 rng=0x4eaa` | `randoms=1496 rng=0x4eaa` |
| frame 4 | `randoms=88 rng=0x9cc3` | `randoms=88 rng=0x9cc3` |

So the seeding lands and the stage's opening draws agree to the word. **A reading that said otherwise was one
pair's run held against another pair's playback**: `orb.log` is one file across a session and a copy of it holds
every launch, so a stage's own lines have to be cut to the launch as well as to the stage. What that mistake
looked like was the same count of draws with different values from the first frame — which is impossible for one
generator, and that is what said the comparison was wrong rather than the game.

**Which leaves the difference where the entry is.** Three launches said so without any code: a run played into
2面 by a *transition* and left, a launch that picked that up and played on to a later chapter of the same stage
and left, and a launch that picked *that* up. The first pickup parted and landed on the wrong frame; the second
landed *field for field* over 6296 replayed frames. What the second was reproducing is a stretch its own run had
played in a stage **built** the way a pickup builds one — and what the first was reproducing is a stretch played
in a stage **transitioned into**. 0x42e894 is that branch and `a_stage_transition_of_th07.rs` is what read it.

**And the signature is deterministic.** Both of those pickups reached frame 827 of their 2面 with `0x096e` where
the file said `0x0f8d` — the same two words in two different launches, the second replaying a file the first
wrote. So the two worlds differ in a fixed way from the stage's start and the generator is only where it first
*shows*: everything a frame's line carries agrees for hundreds of frames, and the first thing to come out
differently is an item's homing mark.

**The measurement left is what the two entries leave differently.** Not the generator, which agrees; not the
numbers, which are written and read identically; not the items' own fields at the start. A stage's whole memory
is what a chapter snapshot already covers — `--self-check` fingerprints every region it saves — so a snapshot
taken at the start of a transitioned-into 2面 held against one taken at the start of a built 2面 names the
regions, and there is no need to guess a field. A fix that works turns `0x096e` against `0x0f8d` into no parting
line at all.

### The recipes the resume's own parting is measured with

The parting is the one thing in this file still open — `counted` climbing on the playback and not on the run
that was recorded, at the frame the two of them part. It is measured by playing the same stretch twice, which
is two launches: one that plays into stage 2 and is closed there, and one that takes the run back.

**It is run against a copy of the install and never against the one somebody plays in.** A played game writes
its records beside itself — `score.dat`, the replays, `pointdevice_resume/`, `orb.log` — so a bot let loose in
that directory overwrites the evidence of every session before it, including the one it was started to explain.
What the copy needs is `th07.exe`, `th07.dat`, `thbgm.dat`, `th07.cfg` and `score.dat`, and it wants none of the
`replay*/` directories: the run this measures is one the bot plays from nothing.

The copy has to be somewhere a Windows process can open, which a path under this repository is not — the
recipe is read by the game's own process, at the attach.

**Two launches and no pause menu.** A run wants closing in the middle for a resume to have anything to pick up,
and `quit` closes the game where it stands; leaving through the pause menu instead would be a cursor to walk
down a screen a recipe cannot see the items of.

`played-into-stage-2.yaml`, beside that copy's exe since the game's own process is what reads it:

```yaml
- press: [decide]                        # the title, the difficulty, the character, the shot and orb's own
  frames: 10                             # question, and stopping once the run is actually running
  until: { stage: 1, chapter: 1 }
  give_up_after: 3600
- play_until: { stage: 2, chapter: 3 }    # the rules, as far as 2面's midboss
  give_up_after: 36000
- quit: true
```

**Every step that could wait for something that will not happen says how long it is worth waiting through.** A
run that dies before stage 2 leaves orb's own retry menu up, which the rules do not press — and a launch left
there runs until somebody notices and closes it by hand, which is exactly what happened once. The bound is what
makes the launch close itself, and `autoplay: step N of M gave up` in the log is what says it did.

**The tap waits for the first chapter and not for the mode question**, which is what the first launch of it had
to say: a tap of `decide` answers that question the frame it comes up, so what the step was waiting for was gone
before the frame the walker next looked at. The step never ended, the rules never got a frame, and the run sat
at its starting position until something shot it — `player=192.00,384.00` for all 1676 frames of it, and not one
`autoplay: step N of M done` in the log. A chapter is reached rather than offered, so no press can undo one.

`took-the-run-back.yaml`:

```yaml
- wait_until: { question: resume }
  give_up_after: 1800
- press: [decide]
  frames: 2
- play_until: { frames: 600 }             # long enough for the reproduction lines to part
- quit: true
```

```sh
scripts/run.sh <copy> --autoplay='<copy in windows>\played-into-stage-2.yaml' --log=verbose
scripts/run.sh <copy> --autoplay='<copy in windows>\took-the-run-back.yaml' --log=verbose
```

The first launch's log wants keeping before the second overwrites it: `orb.log` is one file beside the exe, and
what the parting is read out of is the two runs' `reproduction` lines side by side.

**What a launch has to say** is two things, and the first is not about the parting at all: whether the presses
got through the front end. `autoplay: step N of M done` is one line per step, so a recipe that stopped on its
first is a screen that wanted a press this recipe has not got. Only once every step is done does the second
question — which frame the two `reproduction` lines part on, and which field of them differs — have a run
behind it.

## The font, which was a question and is now a decision

Taken: [0019](../adr/0019-a-game-that-ships-no-font-is-drawn-in-the-machines-own-face.md). There is no
`font.ttf` beside th07's exe — th06 ships one and th07 keeps its fonts inside `th07.dat` — so every screen
of orb's own had nowhere to be drawn, which is a mode whose retry menu nobody can see. What orb does now is
ask the host for a face of its own where the game ships none, and say in the log which of the two a launch
got. The two alternatives and what they cost are in that file.
