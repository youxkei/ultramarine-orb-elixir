# th07, as read off its exe

What is written here was read off `th07.exe` of **md5 `0126afce1e805370d36c3482445e98da`**, which is the one
build `game::th07` has addresses for. Every address below is that build's and means nothing about another.

To read it again:

```sh
i686-w64-mingw32-objdump -d -M intel th07.exe > th07.asm       # the whole of .text, once
i686-w64-mingw32-objdump -d --start-address=0x443550 --stop-address=0x4439a8 -M intel th07.exe
i686-w64-mingw32-objdump -s --start-address=0x49ecf8 --stop-address=0x49ed78 th07.exe   # a table
analyzeHeadless <project dir> th07 -import th07.exe -scriptPath <dir> -postScript DumpAll.java th07.c
```

`objdump` loses the thread after a `jmp` table or a misaligned byte and starts printing data as
instructions; where a listing here disagrees with a dump, dump the one function again from its own `push
ebp`.

`.text` is `0x401000`, `.rdata` `0x48d000` (file `0x8c200`) and `.data` `0x49c000` (file `0x9ac00`), so a
string at an `.rdata` address is at `va - 0x48d000 + 0x8c200` in the file. The game's own log strings are
CP932.

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the
constant. What is here is what the game does, so that the next person reading these functions starts where
this stopped.

**And two spellings of one address disagreeing is the cheapest evidence there is, worth looking for on
purpose.** This game holds its globals as fields of the game manager as well as absolutely, so the same
word has two names — `[gm+0x93d8]` is `0x62f648` — and the offsets and the absolute addresses are the same
numbers twice. Which is how `G_STAGE` was caught being read from the wrong end: a loader indexing the ECL
table by `[gm+0x95ec]` could be held against what the menus write into `0x62f85c`, and one of the two had
to be wrong. Where a game counts something differently from orb, that comparison is the check to reach for
before a launch is spent on it.

## The supervisor, and how a scene is built

The supervisor is `0x575950`. `0x439000` registers its job the way `th06::Supervisor::RegisterChain` does:
a chain element with `0x437c70` as its callback, `0x438986` as its added callback and `0x438de2` as its
deleted one, the supervisor hung off the element's `+0x1c`, added to the calc chain at priority 0.

Its transition switch has cases for the scenes 0, 1, 2, 5, 6, 8 and 9 — `0x437eac`, `0x437ee3`, `0x437fb1`,
`0x437f7b`, `0x4381d1`, `0x438216`, `0x438249` — with a default at `0x438294`. Each tears the scene being
*left* down; scene 0 sets the scene wanted to 1 and is the startup, so nothing is torn down there.

**The case for the scene being left holds a switch on the scene wanted**, and each branch calls the register
of the scene being entered — which is the way to what each number means. Out of scene 1 (`0x437ee3`): wanted
2 through `0x42f3c5`, wanted 5 through `0x44a302` called on nothing, wanted 8 through `0x43b4db`, wanted 9
through `0x42f45d` and then `0x41e820`. So those four are `RegisterChain`s, one scene apiece, and the object
each hangs off its element is that scene's own.

The rest of the game asks for a scene by writing one absolutely into `0x575aa8`, 27 sites:

| wanted | where | who |
| --- | --- | --- |
| 1 | `0x403881` | the pause menu, back to the title |
| 1 | `0x42db85`, `0x42dc79` | the game manager's update, in its block for the attract demo (bit 1, `0x42db42`) |
| 1 | `0x43ab4b`, `0x445de2`, `0x44634d` | |
| 2 | `0x455c04`, `0x459ed0`, `0x45a65d`, `0x45b47d` | the front end's starts: the attract demo, the shot select, the practice stage select, the replay list |
| 3 | `0x427ea5` | the stage transition |
| 3 | `0x4380b4`, `0x4380f1` | inside the supervisor's own cases for `0xb` and `0xc` |
| 5 | `0x455f95` | the title's Result item |
| 6 | `0x403b8d`, `0x403bfe`, `0x404204` | the continue menu |
| 6 | `0x41e7a6` | |
| 6, 7, 9 | `0x42a593`, `0x42a5da` (7), `0x42a6ab`, `0x42a6d7` (9), `0x42a70f` (7) | a stage's end, inside `0x429c42`: 7 where a replay has no next stage |
| 7 | `0x403bb8` | the continue menu, during a replay |
| 8 | `0x455f6c` | the title's Music Room item |
| `0xa` | `0x403908` | the pause menu's restart |
| -1 | `0x456174` | the title screen's phase 2 (`0x45612e`), an `or` of `0xffffffff` |

`0x426f60` groups 3, `0xb` and `0xc`: it is the added callback of the bullet manager `0x62f958`, installed
by `0x4276a0` at `0x4276ec`, and it loads `data/etama.anm` (`0x498524`) into sprite slot `0xb` through
`0x44df90` for every scene wanted that is *not* one of those three.

## The game manager, and what the run's own fields are

`0x42f3c5` registers scene 2's object, `0x626270`: a calc element at `0x62f8b4` and a draw element at
`0x62f8d4`, both priority 2, both with `0x626270` as their argument — update `0x42d8d5`, added callback
`0x42e83e`, deleted `0x42f2e4`, draw `0x42e1d4`. The object is at least `0x95e8` bytes, which is a manager's
size and not a screen's.

Its added callback writes `[this+0x93d7] = [this+0x93d6] + 2 * [this+0x93d5]`, the `character * 2 + shot`
every table of either game is indexed by — so **the character is `+0x93d5` and the shot type is `+0x93d6`**.
It branches at `0x42e894` on the scene wanted being **3** and skips the whole build there, which is th06's
`STATE_GAMEMANAGER_REINIT` shape: 2 is a run being built and 3 is the same manager re-initialised, which is
what a stage transition is. It also stamps `timeGetTime()` into `0x575ae4` and `1.0f` into `0x575ac8`.

**Three objects come off the heap in it, and their sizes say which is which.** `[gm+0x0]` is
`malloc((rand % 0xffff) + 0x10)` — the draw through `0x4318d0` on `0x49fe20` at `0x42e8f8`, the `malloc`
`0x47d39d` at `0x42e91b` — and it is freed again at `0x42e971` (`0x47d285`) once the other two are made: a
block of random size whose only effect is to put the next two at a heap address that differs from run to
run. `[gm+0x4]` is 0x38 bytes from `operator new` (`0x47d441`) at `0x42e928`, and `0x42e95f`/`0x42e964` fill
it with a `rep movs` of 14 dwords from `0x575a68`: **the run's header `[0x626274]` is a copy of the config
block**, the 0x38 bytes of `th07.cfg` the loader near `0x4398c6`–`0x439ac7` checks and defaults. `[gm+0x8]`
is 0xc8 bytes from `0x47d441` at `0x42e93f`, the run's own numbers at `[0x626278]`. `0x42ea66` to `0x42eae3`
walks the third and sets `+0x0`, `+0x4`, `+0x8`, `+0x18`, `+0x28` and `+0x2c` to nothing, the byte at
`+0x20` to nothing, `+0xc` to `0x186a0`, and `+0x30` to `0x32` or `0xc8` — the second where `[gm+0x10]` is
4 or more, which is what makes **`+0x10` the difficulty**.

Before that, the header's byte `+0x1c` — the config's `0x575a84`, the starting lives, which the loader
defaults to 2 at `0x4398f6` — is forced to 2 where the difficulty is 4 or more (`0x42e9ba`) and to 8 in
practice (`0x42e9d0`), and where no replay is playing `0x42ea1c` writes it as a float into `[run+0x5c]`,
the lives.

The fields below are named by a comparison against a limit or by the format they are drawn with, which is the
cheapest evidence there is and weaker than the write that moves a field. What each **is** to orb is beside
its own constant in `game/th07/mod.rs`; what is here is the rest of the object.

| field | evidence |
| --- | --- |
| `+0x0`, `+0x20` | **the score the panel shows and the continues used.** `0x42bdf8` draws `+0x0` with `%.8d` under a hundred million and `%.9d` over it, then the byte `+0x20` with `%1d` one glyph along — this game showing the continues as the score's last digit. `0x4042e0`, the continue path, adds one to that byte, and the ending chooser at `0x41e73f` reads it to pick `end00b.end` over `end00.end`: a run that continued gets the other ending |
| `+0x4` | **the score.** `0x42f45d`, the run's teardown, cuts both of the manager's chain elements, clamps this field to `0x3b9ac9ff` where it is `0x3b9aca00` or more, and copies it into `+0x0`. A cap of 999,999,999 applied as a run ends is a score's, and `+0x0` is the copy a panel's number chases |
| `+0xc` | **the high score shown, and not the next extend.** Drawn by the shape `+0x0` is drawn by — `%.8d` at `0x42bf0d` under `0x5f5e100` and `%.9d` over it — and the callback starts it at `0x186a0`, the 100,000 these games show before any run has beaten it. The build reads it out of the score file at `0x42e732` through `0x444f0d` and again at `0x42e79c` out of a table indexed by difficulty, and zeroes the byte `+0x10` beside it at `0x42e7f8`. A field a run *starts* from the score file with is a record and not a threshold |
| `+0x14`, `+0x18` | two counters of one event, incremented together by the `__thiscall` at `0x43eb90` whenever `[0x4d44f8]` is nothing, each under its own cap: `cmp DWORD PTR [eax+0x14],0x270f` at `0x43eba8` and `cmp DWORD PTR [ecx+0x18],0xf423f` at `0x43ebcb` |
| `+0x24`, `+0x28`, `+0x2c`, `+0x30` | **the point items and the extends they give, and no rank anywhere in it.** An item collected at `0x433247` adds a tenth of its value to the score and one to both `+0x24` and `+0x28`. The loop from `0x4332e0` sets `+0x30` from `+0x2c` — for difficulties 0 to 3 `+0x2c * 75 + 50` under 3, `(+0x2c - 3) * 150 + 300` under 5 and `(+0x2c - 5) * 200 + 800` after; for 4 and 5 200, then 500, then `(+0x2c - 2) * 500 + 800` (`0x433397`) — and `0x4333bd` compares `+0x28` against it; where it has been reached, `0x4333ca` calls `0x42d83a` on the manager and `0x4333e0` adds one to `+0x2c`. `0x42d83a` is the extend: a life through `0x42d5cd(1)` where the lives are under 8, otherwise a bomb through `0x42d612(1)` where the bombs are under 8, with sound `0x1c`. So `+0x28` is the point items collected, `+0x2c` the extends they have given and `+0x30` the count the next one wants — and the `0x32` or `0xc8` the build puts there by difficulty is the first threshold, **which is what made it look like th06's rank starting at 50 or 200** |
| `+0x88` | a score baseline. Every reader takes `[0x62f88c] - [run+0x88]` (`0x4048f7`, `0x41b3ff`, `0x433170` and on) and compares the difference against `0xc350` and `0x186a0` — 50000 and 100000, thresholds and not frame counts. The continue path copies the one into the other at `0x4043bf`, which is that difference being put back to nothing |

### The panel, and the route that is closed

`0x42bda0` to `0x42bfa0` draws it, four calls of the text drawing at `0x402060` into the buffer at
`0x134ce18` with the position advanced by a float between them: `[run+0x0]` with `%.8d` under a hundred
million and `%.9d` over it, then `(byte)[run+0x20]` with `%1d`; then `[run+0xc]` and `(byte)[run+0x10]` the
same way. Over a hundred million `0x42be50` writes `0x498b20` (0.9) into `0x13542dc` and `1.0` into
`0x13542e0` first, which is the ninth digit being squeezed into the width the eight had.

**The labels beside those lines are not text.** Every other constant that function reaches for between
`0x42bb00` and `0x42bf60` is a float, so what says which line is which is a sprite and there is no string to
read. That route is closed for every field on the panel — the lives and the bombs were found by the events
that move them instead.

What the panel's own tile is comes out of the vm the game lays its background with rather than out of a sheet
layout: `[vm+0x1e4]` is the sprite that vm's script put on it, `[sprite]` is the slot of the texture, and
four floats of it are the piece of the sheet — all four addresses out of the draws at `0x44f770` and
`0x44efb0`. The row of lives is laid by the loop at `0x42bc5f`, from `0x498c50` (496.0) by `0x498a80` (16.0)
at `0x498c54` (96.0), the same instruction laying the tile grid from (416, 16) in 32s.

## The enemy manager, and the scripts a stage runs

The manager is the static at `0x9a9b00`.

| | |
| --- | --- |
| the slots | `0x1e0` of them from `+0x4f50`, each `0x4f48` bytes, alive being bit 7 of the slot's `+0x2e28`. The update walks all of them at `0x4207c7` |
| an enemy's script clock | `+0x6f0` of the slot. The ECL runner at `0x410520` compares it against the time word at the head of each instruction, takes the opcode from the instruction's `+0x4` less one, bounds that at `0xa0` and jumps through the table of 161 at `0x417838`. The instruction's byte `+0x9` is a difficulty mask, ANDed against `[0x626284]` — the manager's `+0x14`, the difficulty as a bit |
| the ECL file | `0x1347938`, loaded by `0x40e420` out of the table at `0x49f560` by the stage, with its subs counted in the word at `+0x2`. `0x49f588` is the table of `data/stgNenm.anm` beside it, and the first usable entry of each is index 1 |
| the main scripts | **four contexts of 0x10 bytes from `+0x9545f4`**, run by `0x41f6f0`: `+0xc` is the instruction, `+0x8` is that context's own clock, and the instruction set is a smaller one — the opcode bounded at `0xc` through the table at `0x41fd2f`, whose first case is the spawn that copies `0x13d2` dwords of template into a slot and starts its ECL |
| the item manager | `0x575c70`, the third big static, which the bomb calls `0x4326f0` and `0x433a90` on. Its own count is at `+0xae2ec`, zeroed at `0x4329e0` at the head of its update `0x432990`, which walks `0x44c` slots of `0x288` bytes |
| the bullet manager | `0x62f958`, registered by `0x4276a0`, whose added callback `0x426f60` loads the bullet sheet `data/etama.anm` |

## The player object, and what else is inside it

`0x4bdad8`, and its register at `0x4429d0` fills `0x2df9e` dwords from that address — which ends at
`0x575950`, the supervisor's own address, so the two are neighbours in `.data` and a write past the end of
one is the other. The object holds far more than the player: `+0x16a30` is the bomb's own object, `+0xb7e5c`
to `+0xb7e64` the three chain elements, and the fields the shot uses run to `+0xb7e6c`.

## What kind of run it is

`0x62f648` is a bitfield read in some ninety places, bits 0 to 4.

| bit | what, and how it was read |
| --- | --- |
| 0 | a practice run. The title menu's eight items dispatch through the table at `0x45622d` (`0x455ce4`, `0x455e32`, `0x455d8b`, `0x455ee2`, `0x455f95`, `0x455f6c`, `0x455fbe`, `0x455ffd`), and of the four that start something only the third sets it; the third item is Practice Start. `0x45a583` sets it as well and is the other way in |
| 1 | the attract demo. `0x45bfb1` clears the replay bit where it is set and then opens `score.dat`, which is a demo ending through the same tail a run uses, and `0x4018e0` runs a block only during a demo, on `[0x134ce18+0xa028]` and `[0x134ce18+0x9e50]` — fields of the text manager `0x134ce18`, whose calc job `0x4017e0` is registered by `0x401e30` and is also what runs the pause menu and the continue menu |
| 2 | **a run standing with neither menu up.** `0x42f25b` sets it at the end of every build, `0x42f39d` clears it in the manager's deleted callback `0x42f2e4`, and the manager's update rewrites it every frame at `0x42dd8f`–`0x42dd9d` as "neither `0x62f64c` nor `0x62f64d` is up" — all three through `[gm+0x93d8]`. The replay recorder `0x442cd0` returns at `0x442cd9` without recording where it is clear; the other reads are `0x439195`, `0x4393a0`, `0x442e57` and `0x442ee9` |
| 3 | a replay being watched. `0x455b45` is the attract demo's load in the title handler: it picks one of the three names at `0x49ea7c` by the counter `0x62f64e` mod 3 into `0x62f654` (`0x455aca`–`0x455ad6`), reads the file through `0x431330`, takes it through `0x4433b0`, logs `0x4959a0` where nothing came back, sets bit 3 from `(1 & 1) << 3` and bit 1 beside it, clears `0x62f650`, fills the run from the replay's header — its byte `+0x57` into `0x626280`, its `+0x56` halved into `0x62f645` — sets the stage to the first slot that holds a record (`0x455bc1`–`0x455be0`) and asks for scene 2 at `0x455c04`. A replay picked from the list sets bit 3 at `0x45b38a` and leaves bit 1 alone |
| 4 | **a run that ended of its own accord.** `or 0x10` at `0x41e5a3`, `0x42a68e` and `0x42a6ba`, read at `0x446bb7` and `0x448a78`. Before `0x42a68e`, on the Extra and the Phantasm, `0x42a688` counts one into `0x62f52c + difficulty * 0x2c`, and on the Extra alone `0x42a668` writes `0x63` into `0x62e6c4 + combined shot * 0x1c + difficulty`. A run given up through the pause menu does not set it, so the bit separates finished from left. **The added callback clears it on every build**: `0x42f294`–`0x42f2a0` is `and 0xffffffef` on `[gm+0x93d8]`, past the block a transition skips, so a run's start, a restart and every stage transition all begin with it clear. A grep over `ds:0x62f648` alone finds only the `and`s that clear bits 0, 1 and 3 |

**The continue menu `0x403b60` reads two of them before anything else**: bit 0 sends it to scene **6**
(`0x403b8d`) and bit 3 to scene **7** (`0x403bb8`), each after writing `[run+0x4]` into `[run+0x0]` — the
score and the number the panel chases being equalised as a run ends — and its own flag `0x62f64d` is cleared
on both paths (`0x403b75`, `0x403bb1`). So a practice run that loses its last life goes to the score-file
screen and a replay back to the front end. It also asks for scene 6 at once (`0x403bfe`) where the continues
used, the byte `[run+0x20]`, have reached `[0x62f8a0]` or the difficulty is 4 or more.

The pause menu `0x402780` asks for no scene but 1 (`0x403881`) and `0xa` (`0x403908`). It reads bits 0
and 3 (`0x40290a`, `0x402ad3`, `0x402b47`, `0x402e38`, `0x4030b3`) for its own items and keys, and none of
those reads leads to another scene.

**What the scenes the supervisor `0x437c70` builds out of scene 2 are**, by the branch each takes:

| scene | out of scene 2 |
| --- | --- |
| 1 | `0x42f45d`, then `0x443da0`, then back through the startup case |
| 3 | a stage transition: `0x42f45d` at `0x438123`, the manager registered again at `0x438128`, wanted set to 2 at `0x43813c` |
| 6 | the score-file screen: `0x42f45d`, then `0x44a302` with `ecx = 1` at `0x438044` — the name entry, or phase `0x12` in practice |
| 7 | out of a run to the front end: `0x42f45d`, `0x443da0`, wanted set to 1 at `0x438166`, and the front end `0x45c5d0` registered with `ecx = 1` at `0x438185` |
| 9 | `0x42f45d` and `0x41e820` (`0x4381b6`) |
| `0xa` | the pause menu's restart: `0x42f45d`, the stage set to 0 — or one less, in practice or at a difficulty of 4 or more (`0x43805f`–`0x438083`) — `0x443da0`, the manager registered again and wanted set to 2; the added callback then adds one to the stage |
| `0xb`, `0xc` | each writes 3 into `0x575aa8` (`0x4380b4`, `0x4380f1`), so the rebuild takes the reinit branch, and `0xb` takes one off the stage first; nothing in the exe writes `0xb` or `0xc` into `0x575aa8` or `[sup+0x158]`, so neither is reached |

Scene 8 is the music room: the title's item at `0x455f6c` asks for it and the case out of scene 1 registers
`0x43b4db`.

## The lasers, and the test one hits the player by

Two walks of the `0x40` slots of `0x4ec` bytes at `0x995f80`, and each field below is a constant in
`game/th07/mod.rs`; what is here is how each was read.

| what | where | how it was read |
| --- | --- | --- |
| alive | `+0x4d4` | what both walks test — `0x418b80` and `0x42483d` |
| the origin | `+0x498`, three floats | `0x418cec` hands `laser+0x498` to the geometry at `0x4185d0` as the beam's base |
| the angle | `+0x4a4` | `0x4248db` hands it to the sin/cos at `0x417af0` |
| along the beam | `+0x4a8` near, `+0x4ac` far | `0x418be0` takes their difference for a length and `0x418c04` their midpoint |
| how wide | `+0x4b4`, out of `+0x4b8` | `0x4248ac` copies the one into the other |
| which phase | `+0x4e8` | both walks skip a laser whose phase has reached 2, and `0x424877` sets it to 2 |
| a flag word | `+0x4e4` | `0x42484b` tests bit 2 of it against the reason the walk was made |

**`0x43e6b0` is the player's own laser test**, and `0x426713` and `0x426727` hand it a laser's origin and
angle, which it turns the player into the beam's frame with through `0x4319b0`. So a beam is its own hazard
and not a bullet wearing one — which took a second look: both callers of the geometry at `0x4185d0`,
`0x4189ed` and `0x418cec`, hand it a *bullet's* position at `+0xb8c`, so neither of those is against
`0x4bdad8` and a grep that stopped there says the player has no laser test at all.

## The frame's own walk of the chain

`0x4347f1` and `0x434801` test what the walk answered and `0x4347fc` and `0x43480c` **return** on it, so a
walk asked for after one of those answers is a walk no frame of the game's own makes. Around the walk the
frame also does three things: `0x434794`'s own call, `[ecx+0xa0]` at `0x4347d4`, and the sounds played at
`0x4347ec`.

## The front end's fifteen screens

The update bounds `[front+0xd0f8]` at `0xe` and takes its jump-table index out of the byte table at
`0x4555ce`, which reads `[0, 8, 1, 2, 3, 4, 5, 6, 3, 4, 5, 7, 3, 4, 5]` — so **the fifteen screens are three
flows of three sharing their handlers**: index 3 is reached by screens 4, 8 and 12, index 4 by 5, 9 and 13,
index 5 by 6, 10 and 14, one triple per way of starting a run. Index 4 is `0x457fe5` and writes the character
at `0x4593b7` and `0x459475`; index 5 is `0x459518` and writes the shot at `0x459e7d` and `0x459f72`, with
the normal start at `0x459e9a` inside it. Each title item writes its own first screen — Start 4 at
`0x455d20`, Practice Start 8 at `0x455dc7`, Extra Start `0xc` at `0x455e77` — so the shot select is screen 6,
10 or 14 by the way in.

**The difficulty lives in the config block before a run starts**: the byte `0x575a89`, `0x575a68 + 0x21`.
Index 3, the difficulty select, writes it at `0x457e41` and `0x457ef0` — plus 4 on screen `0xc`, at
`0x457e50` and `0x457eff` — and `0x459ead`, the shot select's own decide, copies it into `0x626280`; the
practice start copies it at `0x45a647`. The config loader defaults it to 1 at `0x4399be`, and a value of
6 or more sends it back to the defaults (`0x439ac7` → `0x439b29` → `0x4398f6`).

The title screen's own handler is `0x4555dd` and dispatches on `[front+0x8]`: phase 0 at `0x455621`, which is
the block the item dispatch at `0x455cdd` lives in, and 1, 2 and 3 at `0x45599d`, `0x45612e` and `0x456184`.

## The pause menu and the continue menu

Two functions with an object apiece — fields at `+0x1c0` and `+0x1ce`, a state at `+0x0` and a timer at
`+0x4` — and which is which is settled by the flag each clears, the pair `game::th07`'s `GAME_MENU_UP` reads:
`[gm+0x93dc]` is `0x62f64c` and `[gm+0x93dd]` is `0x62f64d`.

**The pause menu ends at `0x4039e4`** and its object has `0xa` items. `0x42d958` raises its flag, on the edge
of the menu button, and this clears it three times: `0x40342c` on the way back to the run, `0x40387a` before
it asks for scene 1, and `0x403901` before it asks for scene `0xa`. Both asks wait for the timer to reach
`0x14` first (`0x403868`, `0x4038ef`).

**The continue menu is `0x403b60`** and its object has five. `0x441161` raises its flag: the branch a death
takes where `[[0x626278]+0x5c]`, the lives as a float, is nothing left. It clears it at `0x403b75`, `0x403bb1`,
`0x403bf7`, `0x4041fd` and `0x404297`, and asks for scene 6 at `0x404204` once its own timer reaches `0x14`.
Through the sound player at `0x4ba0d8` it plays sound 0 as its cursor moves (`0x403fdd`, `0x404154`) and
`0xa` on the shot key (`0x404019`, `0x404190`). The continue itself is `0x4042e0`, inside it.

## The music paused under a menu

`0x44d2f0` is `SoundPlayer::SendCommand`, `__thiscall (kind, argument, name)`, which writes the kind into the
player's queue at `+0x73c` and returns; `0x44c9c0`'s table at `0x44d1d9` is where a kind goes, and kinds 6 and
7 go to `0x44d00e` and `0x44d057`. Both branch on the backend byte `[0x575a87]` being 1, leave the command in
the queue where the stream's `+0x74` is set, and otherwise call `0x45d910` or `0x45d960` on `[player+0x39bc]`:
the first clears the stream's `+0x30` and calls `IDirectSoundBuffer::Stop`, the second sets it and calls `Play`
with the stream's own `+0x20` and `+0x24`. The name each site hands is a string of the exe's, `Pause` at
`0x498a40` and `UnPause` at `0x498a38`.

| where | kind | what |
| --- | --- | --- |
| `0x42d9b1` | 6 | the manager, on the frame it raises the pause menu's flag |
| `0x403496` | 7 | the pause menu, on the way back to the run |
| `0x403c4c` | 6 | the continue menu, on its own first frame |
| `0x4044b0` | 7 | the continue menu, on the way back to the run |

**So every way out of either menu that leaves the run standing sends the 7, and the ways out that ask for a
scene send nothing** — the scene each asks for builds the run and its music again. Which is what makes
「最初からやり直す」 the one item a run can carry on from with its music still paused, and orb answers that item
itself.

The manager's 6 and the pause menu's 7 both ask first whether `[0x62f85c]` is 6 and `[0x49fbf0]` is under
`0x12c` (`0x42d99c`, `0x403481`); the continue menu's pair ask nothing. What the first word of the object at
`0x49fbf0` counts is not established — it is a global some fifty `__thiscall`s are made on, and these two
comparisons are the only places anything is held against it.

## The score file's own phases

`[obj+8]` is the phase, and it decides everything.

| where | what |
| --- | --- |
| `0x44a302(kind)` | allocates `0xce6c` bytes and constructs the score-file object (`0x444a5b`). Five callers: `ecx = 2` at `0x434277` (after the message pump — the way out, and only where `[0x62f4e0]` is not zero, `0x434269`) and at `0x42e646`; `ecx = 0` at `0x437f3a`; `ecx = 1` at `0x438044` and `0x438283` |
| `kind 1` | `[obj+8] = 0xa`, or `0x12` where the practice bit is set, then registers the job `0x445db3` on the chain — the screen's own flow |
| `kind 2` | `[obj+8] = 0x13` and calls `0x449b05` inline: **the save on the way out** |
| `0x449b05` | reads `score.dat` (`0x449f82`), then parses its `CATK` records over `0x626288` (`0x44a059` → `0x445069`) — *skipped* where the phase is `0xa` or `0x12` (`0x44a040`, `0x44a049`) — then walks those records per shot into the object (`0x44a145`) and, **only where the phase is `0x13`** (`0x44a1e0`), writes the file (`0x44a1e9` → `0x44a1f9` → `0x44552c` → `0x431540`) |
| `0x445069` | the parse, `(ecx = the file image, edx = an array)`: walks the chunks, and for every `CATK` whose version byte is 1 `rep movs` `0x1e` dwords from the chunk into `edx + card * 0x78`, the card number being the word at `+0x28`. The first such `CATK` whose card number is `0x8d` or more **ends the walk** (`0x4450cf`) — the records after it are not read. File → memory, whole records, unconditional |
| `0x44552c` | the write, whose one caller is `0x44a1f9`: it serialises out of `0x626288` itself (`0x445713`), so what reaches the file is what memory holds. `0x44a1f9` writes only where the object still holds a file image, then frees it and clears the field |
| `0x42e634` | the second `kind 2`: it saves, then **zeroes every record** (`rep stos` of `0x1086` dwords over `0x626288`) and rebuilds each one's `CATK` header with its card number and no counts, and at its end (`0x42e804`–`0x42e813`) copies the whole `0x1086` dwords to `0x62a4a0`. Its one caller is `0x42eae7`, inside the game manager's added callback — so a run being built is what empties them |
| `0x42f94c` | not a merge, though it sits between the parse and the write: it counts the records whose last word is set and unlocks at `0x3c` of them |

So a `kind 2` save is a round trip that cannot carry anything — the parse overwrites memory with the file
immediately before the write reads memory — and no phase writes without it. **The parse has three call
sites**, which is what says where a session's counts can be put back: `0x42e735` inside `0x42e634`, *after*
it empties the records, so a run's own records are the file's cumulative with that run's counts already on
top; `0x44a059`, the save's own; and `0x45bff3` inside `0x45bf15`, the read the front end makes for its
menus, over the same array.

## The seal over a run's numbers

The run's own numbers live in one object, `[0x626278]`, 0xc8 bytes from `operator new` at `0x42e93f` with
**no memset** — what is not written is whatever the heap held.

`0x4012b0`, `GameManager::Reseal`, `__thiscall` on the game manager `0x626270`, writes five fields:

| field | what |
| --- | --- |
| `[run+0x3c]` | `(rand % 0x186a0) + 0x198f`, drawn from `0x49fe20` |
| `[run+0x98]` | a second draw, the same shape |
| `[run+0xac]` | `[run+0x3c]` at `0x40133f`, which the sum then moves (below) |
| `[run+0xb0]` | the sum `0x42d7be` takes |
| `[gm+0x9614]` | the float of `[run+0xb0] + [run+0x98]` |

`0x42d7be` sums the **bytes** of four ranges through `0x42d75a`: `run+0x34` through `run+0xac` (its length,
0x78, worked out at `0x42d7cc` from those two field addresses), `[0x626278]+0xb4` for 0x14 bytes, all 0x38
bytes of the run's header at `[0x626274]`, and the 0x38 bytes at `0x575a68` the header was copied from.
**And for every byte it sums `0x42d75a` adds `[run+0xbc]` into `[run+0xac]`** (`0x42d794`–`0x42d7b1`): the
four ranges come to 0xfc bytes, so after any reseal `[run+0xac]` is `[run+0x3c] + 0xfc * [run+0xbc]`, mod
2^32.

`0x404fe0` is the check, and it makes **two comparisons and takes the sum again in neither**:

- `[run+0xac]` against `[run+0xbc] * 0xfc + [run+0x3c]`. `0xfc` is `(run+0xac) - (run+0x34) + 0x84`, worked
  out at `0x405003` — the byte count of the four ranges — so this holds after every reseal. `[run+0xbc]` is
  one of the five noise words at `+0xb4` and is never zero, so `[run+0xac]` never equals `[run+0x3c]`.
- `[run+0xb0] + [run+0x98]` against `[gm+0x9614]` truncated back to a whole number by `_ftol2` (`0x48b8a0`).

It answers 1 where either fails, and the caller then fills 0xb4 dwords from `0x575950` with `0xffffffff` —
the device pointer, the viewport, the pad mapping and the supervisor's scene are all inside that range, so
the game answers a changed number by destroying itself. Every guarded write of the game's own asks the
check first: `0x42d5d8` for the lives, `0x42d61d` for the bombs, `0x4325eb` for the power, `0x43b75b` for
the deaths, and six sites besides — `0x406879`, `0x41c9e8`, `0x42207c`, `0x43b7ab`, `0x44074e` and
`0x440b36`.

**`[gm+0x9614]` is `0x62f884`, and it is the seal's own value rather than anything about the cherry.**
There are three kinds of `fstp` into it besides `0x4012b0`'s:

- **the manager's update reseals inline on each update it runs through**, at `0x42dc8b`–`0x42dcb8`:
  `[run+0xac] = [run+0x3c]`, the sum through `0x42d7be` again, and `0x62f884 = sum + [run+0x98]` — but
  `[run+0xb0]` is not written. So the first comparison is put right every update, and the second is the one
  that catches a changed byte: `[run+0xb0]` still holds the sum of the last real reseal, and `0x62f884` now
  holds the sum of what memory holds.
- **the same update range-checks the noise** straight after: the seven words at `+0x34` (`0x42dcda`) and the
  eight at `+0x8c` (`0x42de14`) against `0x198f`..`0x1a02f`, and the float pairs at `+0x60` (`0x42dd21`) and
  `+0x54` (`0x42ddbf`) against 6543.0..106543.0. One out of range writes -9999.0 (`0x498c80`) into
  `0x62f884`, which the next check fails on.
- `0x42e3da` ends with the same inline reseal, `+0xb0` written this time (`0x42e5fa`–`0x42e62c`).

### The noise the seal is drawn over

`0x42e3da`, called at `0x42e951` right after the allocation, fills the run's object with
`(rand % 0x186a0) + 0x198f` apiece — `[run+0x88]`, the seven words at `[run+0x34]`, eight at `[run+0x8c]` and
five at `[run+0xb4]` (`0x42e5e6`) — and with a float draw times 100000.0 plus 6543.0 apiece: two at
`[run+0x54]`, two at `[run+0x60]`, three at `[run+0x70]` and two at `[run+0x80]`. Those stand between the
deaths at `+0x50`, the lives at `+0x5c`, the bombs at `+0x68` and the power at `+0x7c`.

`0x43b5c0` draws `+0x34` through `+0x44` and the floats at `+0x70`, `+0x74`, `+0x78` again outright, from
`0x43e38c` and `0x43eb74`. So none of them is a number a run carries: what reads the seven words at `+0x34`
is the seal — `+0x3c` as its key, and all seven in the manager's range check.

`[run+0x88]` is the exception: it is the baseline every reader of the cherry takes off, so its random value
is the cherry's own obfuscation and not noise.

## Starting a replay, and starting one at a stage past the first

Inside `GameManager::AddedCallback` (`0x42e83e`), per stage, at `0x42f05f`:

```text
if ([0x62f648] >> 3) & 1 {                  a replay is playing
    0x42e38c(gm)                            gm+0x9634, gm+0x963c, gm+0x9638 from [0x49f5d0 + difficulty*0xc]
    0x443aa0(ecx=1, edx=0x62f654)           the replay manager, reading
    save 0x49fe20 -> 0x4012b0 -> put it back
}                                           0x42f067 jumps straight on where it is not
[gm+0x95e4] = [0x49fe20]                    the stage's own seed written down
0x4074c0(STAGE_BUILD)                       the stage laid out
...
if not a replay:
    0x443aa0(ecx=0, edx=0x497e1c)           the replay manager, writing, at 0x42f1d0
```

So **the numbers go back before the seed is written down and before the stage is built**, and the reseal
between them is inside the replay branch alone.

`0x443aa0` registers the manager: 0xd8 bytes into `[0x4b9e48]` where there is none, and a chain job apiece
— `0x442cd0` at priority 0x10 for writing, with `0x443040` as its added callback, and `0x442ee0` at priority
5 for reading, with `0x443550`. **Where the manager already exists it calls the added callback directly**
(`0x443d17`, `0x443d24`), which is what every stage past the first does.

### `0x443550`, which is the whole of what a stage's start is put back from

`0x443571` through `0x44372f` runs once: the file read at `0x431330`, decoded at `0x4433b0`, and the seven
stage slots' pointers fixed up to absolute and their lengths worked out.

From `0x443734` is the part that runs for every stage:

```text
slot   = [0x62f85c] - 1        the stage, held at 6
record = [replay + slot*4 + 0x1c]         -1 returned where the slot is empty
inputs = [replay + slot*4 + 0x38]
```

and then, in this order:

| at | written | from |
| --- | --- | --- |
| `0x443794` | `0x62f645` | `header+0x56` halved — the character |
| `0x4437b0` | `0x62f646` | `header+0x56 & 1` — the shot |
| `0x4437bf` | `0x62f647` | `header+0x56` — the two combined |
| `0x4437cf` | `0x626280` | `header+0x57` |
| `0x4437e0` | `[run+0x28]` the point items | `record+0x4` |
| `0x4437ea` | `0x62f8a4` the rank | `record+0x25` |
| `0x443803` | `[run+0x5c]` the lives, then **reseal** | `record+0x23` |
| `0x44381d` | the bombs, through `0x401390`, which reseals | `record+0x24` |
| `0x443834` | `[run+0x7c]` the power, then **reseal** | `record+0x22` |
| `0x44384d` | `[run+0x18]` | `record+0x14` |
| `0x443859` | `[replay+0x84]` | `record+0x2c` |
| `0x443865` | `0x62f644` | `record+0x26` |
| `0x44387d` | `0x62f88c` the cherry | `record+0x8` + `[run+0x88]` |
| `0x443894` | `0x62f888` what it fills towards | `record+0xc` + `[run+0x88]` |
| `0x4438ab` | `0x62f890` the cherry+ | `record+0x10` + `[run+0x88]` |
| `0x4438db` | the cherry+ held at `[run+0x88] + 0xc350` and `0x441960` called on `0x4bdad8` | where it was over |
| `0x4438ff` | `[0x626274]`, 0x38 bytes | `[replay+0x4] + 0x70` |
| `0x443910` | `0x49fe20`, the generator | `record+0x20` |
| `0x443922` | `[run+0x1c]` | `record+0x27` |
| `0x443931` | `[run+0x2c]` the point-item extends given | `record+0x18` |
| `0x443940` | `[run+0x30]` the point items the next extend wants | `record+0x1c` |
| `0x443949` | `[replay+0xa4]` | the inputs |
| `0x44398c` | `[run+0x4]` and `[run+0x0]`, the score | the first dword of `[replay + 0x14 + stage*4]` — the previous stage's record, `+0x0` — for stages 2 to 6 |

**Of the player object, `0x4bdad8`, it does one thing**: the `0x441960` call at `0x4438e6`, made only where the
cherry+ was over its cap. It writes nothing of it directly — not its state byte at `+0x2408`, not its
position. A replay started at stage 5 builds its player the same way a run does.

`[0x626274]` is the run's header, the copy of the config block `0x575a68` the added callback makes at
`0x42e95f` (the front end's read `0x45bf15` allocates one as well, at `0x45bf3f`). Only two of its fields are
read while a stage is played: `+0x1c`, the starting lives (the config's `0x575a84`), held against 3 and 4 at
`0x42829a` and `0x42b492`, which cut the stage-clear bonus to five tenths (`0x42b4ab`) and two tenths
(`0x42b4bc`); and `+0x25`, the config's `0x575a8d`, a flag tested at `0x402b0d`, `0x42e0f7`, `0x43a19e`,
`0x442d0e` and `0x442f02`.

**`0x62f8a4` does differ between a run played into a stage and a run built at it**, and a pair of launches is
what says so: the same stage 2 held `0x20` entered by a transition against `0x10` built by a resume, read off the
dump at the stage's start on both sides.

**`0x62f8a4` is the rank, `[gm+0x9634]`.** `0x42e38c` fills it and its bounds out of
`[0x49f5d0 + [0x626280]*0xc]` — indexed by the difficulty (`0x42e393`), three words an entry: the starting
rank into `+0x9634`, its floor into `+0x963c`, its ceiling into `+0x9638`:

| difficulty | start | floor | ceiling |
| --- | --- | --- | --- |
| Easy | 16 | 12 | 20 |
| Normal, Hard, Lunatic | 16 | 10 | 32 |
| Extra, Phantasm | 16 | 15 | 16 |

It moves all through play. `0x42f4aa` adds an amount to the accumulator `[gm+0x9640]` and raises the rank one
for every 100 in it, held at the ceiling; `0x42f526` takes an amount off and lowers it the same way, held at the
floor. Both store through `[ecx+0x9634]`, which is why a grep for the absolute address finds no writer.
`0x42f4aa` is called at `0x4206eb`, `0x42d874` and `0x42d8be` (an extend, 200), `0x4330c8`, `0x4332bd`,
`0x4332cb`, `0x4335ab` and `0x43ece7` (a graze, 6); `0x42f526` at `0x432e33`, `0x440c8b` (a bomb, 200) and
`0x440fc4` (1600). The accumulator is zeroed at `0x42ef28`, past the skip target `0x42eee6`, so it starts
again every stage.

`0x42e38c` is called from `0x42eafb` as well as from `0x42f06c` — but `0x42eafb` is *inside* the block
`0x42e89b` jumps over, so a transition reaches neither the fill nor the zeroing of `[run+0x18]` at `0x42eaa0`
before it, and the rank a stage starts with is the one the last stage ended on. ECL reaches it as well:
**`0x40ecf1` hands the address out**, as one entry of the dispatcher that answers with a pointer to the
variable an index names — the entry beside `[0x626280]`, the difficulty.

What reads it is the ECL runner `0x410520`: at `0x412ad4` and `0x412b3c` it scales the bullet counts an enemy
holds in the shorts at `+0x2bb0` to `+0x2bb6`, and at `0x412bab` and `0x412c04` the speeds at `+0x2ba8` and
`+0x2bac` that `0x40fe0a` and `0x40fe17` set to -0.5 and +0.5, each as `low + (high - low) * rank / 32`
(`[0x498a84]` is 32.0) added to what the instruction asked for, a speed held at 0.3 or more (`0x412bdf`). So
a higher rank is more bullets and faster ones, and the `0x20` against `0x10` the pair of launches read is the
ceiling of Normal to Lunatic against the starting value.

`[0x626274]`, the run's header, is the other half of the same pair and needs no restoring: the two entries into
one stage 2 held all fourteen of its words the same.

### And the mirror: where a stage's record is written

`0x443040` writes one, indexed the same way, and its caller is `0x443d17` — `0x443aa0`'s mode 0, reached from
`0x42f1d0`, which is the **normal run's** branch of the same callback. `0x42f1d0` is past `0x42f0b4`, so a
stage's record is written *after* the stage has been built, while a replay's restore runs *before*.

Two of what it writes go somewhere other than the stage's own record. The score goes in at the *next*
stage's start: `0x443190`–`0x4431a9` writes `[run+0x4]` into the first dword of `[[replay+0x4] + 0x14 +
stage*4]`, the previous stage's record, which is the place the restore reads the score from at `0x443961` to
`0x44398c`. And the word at `record+0x20` comes from `0x62f854`, `[gm+0x95e4]` — the stage's seed — at
`0x4432e3`, which the restore puts back into `0x49fe20` at `0x443910`. The header it fills is `0xe8` bytes with `[0x496aa8]` as its first dword and `0x1100` in the
word at `+0x4`; `+0x1c` and `+0x38` are seven slots each, a stage's record and a stage's inputs, `0x70800`
bytes of them, indexed by `[0x62f85c]`.

The float fields go in through `0x48b8a0`, which is `_ftol2`: `fistp` and then a correction by the sign and
the remainder, which is **truncation toward zero** and not a round.

## What a collection is worth

`0x42f5a2` is the cherry going up, `__thiscall` on the game manager taking an amount:
`[gm+0x961c] += amount` held at `[gm+0x9618]`, and `[gm+0x9620] += amount` where the amount is positive and
`[0x4bfee5]` is nothing. Those three are `0x62f88c`, `0x62f888` and `0x62f890` — the cherry, what it fills
towards, and the cherry+.

Its seven callers pass a constant at `0x433712` (0x14), `0x43372a` (0xa) and `0x433749` (0x1e), and a
worked-out amount at `0x4338d9`, `0x433998`, `0x416722` (inside the ECL runner) and `0x42132d`. Where the
cherry+ reaches `[run+0x88] + 0xc350` it is held there (`0x42f62b` on) — the same cap the replay restore
applies at `0x4438c1`. The worked-out one is `([run+0x18] / 40) * 10 + 300`, read at
`0x4338e3` — so **`[run+0x18]` decides what an item's cherry is**. `0x43369a` makes the same sum for the
score instead.

`[0x62f644]` is an index into the 31 dwords at `0x49ecf8`:

```text
10 20 30 40 50 60 70 80 90 100 200 300 400 500 600 700 800 900 1000
2000 3000 4000 5000 6000 7000 8000 9000 10000 11000 12000 51200
```

`0x432efe` adds one to it, `0x432f19` holds it at 30, `0x432f27` reads the table by it and `0x432f42` adds
a tenth of what it finds to `[run+0x4]`. It goes back to nothing at `0x432fb2` and where the player is hit,
at `0x440d6a`.

`0x43eb90` adds one to `[run+0x14]` and one to `[run+0x18]` — the first held at 9999 and drawn on the panel
at `0x42a395`, the second at 999999 — spawns an effect at the player's position, plays sound 0x1e, and takes
**2000** points at `0x43ed12`, where it also advances `[0x12fe0d0]` by `([0x62f88c] - [run+0x88]) / 1500 * 20
+ 2500`. `0x43eb0d` calls it only where `[0x4bdad8+0x2408]` is neither 1 nor 2: `0x4410a0` writes 1 in the
player's death path and `0x43ee0e` writes 2.

**The twelve sites between `0x408a23` and `0x40e234` that write 3 are the bombs.** `0x442784`–`0x4427b6` install the bomb's routines out of the table at `0x49ec50`, four per
combined shot, into `[player+0x16a3c]` and `[player+0x16a44]`, and the twelve routines those two slots take —
`0x408710`, `0x4091b0`, `0x409dd0`, `0x40a3a0`, `0x40a7c0`, `0x40af10`, `0x40b7d0`, `0x40be20`, `0x40c2e0`,
`0x40ca50`, `0x40d4c0`, `0x40da80` — hold one site apiece, each after spawning four things and playing
sound 13. The bomb handler `0x4409f0` calls them at `0x440b18`/`0x440b26`
and, as a bomb starts, at `0x440c59`/`0x440c67`. 3 is written as well at `0x4412a6`, `0x441785` and `0x441d5c`.

**`[0x4bfeb4]` and `[0x4d44f8]` are `[player+0x23dc]` and `[player+0x16a20]`, the bomb being in use.** The
bomb's start sets both to 1 at `0x440bfa` and `0x440c07`, and they are cleared at `0x440a2e`, `0x440cda`,
`0x4427d8` and inside each bomb routine (`0x40874c` to `0x40dc07`). So the branches they gate are skipped
while a bomb runs: `0x433691`, `0x433702` — the cherry of `0x14` an item gives at `0x433712` — and
`0x43eb9a`, the graze's counters.

## A replay file, and what its header says

`"./replay/th7_%.2d.rpy"` (`0x4967bc`) is the name of a slot, and the replay list reads all fifteen of them
in the loop at `0x45aa11`: `0x431330` for the bytes and `0x4433b0` for the rest, which answers a pointer or
zero and is what a slot the list does not show has failed. What the list then copies out of that pointer is
`0xe8` bytes apiece.

`0x4433b0` is the whole of the reading, in six steps, any failing one freeing the file and answering zero:

```text
[+0x00] == [0x496aa8]                       the magic, "T7RP"
[+0x04] == 0x1100                           the version, as a word
key = [+0x0d]                               and from +0x10 to the end of the file:
    *p -= key; key += 7                      one byte at a time
sum = 0x3f000318 + every byte from +0x0d     against [+0x08], which is that sum written down
malloc([+0x18] + 0x54)                       the buffer the list reads
    the file's own first 0x54 bytes copied over it
    0x45ef00(src=file+0x54, dst=+0x54, [+0x14] in, [+0x18] out)
byte [buf+0x95] == 0                         0x4434f0
0x43a569(buf+0xe0, [buf+0xd8], [buf+0xdc]) == 0     on the supervisor, 0x443520
```

So `[+0x14]` is how many bytes the body is and `[+0x18]` how many it comes to, and **the header the list
shows is inside that body**: the first `0x54` bytes are the file's, and everything the list draws is past
them.

`0x45ef00` is LZSS, bit by bit and the most significant first, mask from `0x80`:

A `1` bit is followed by eight more, which are a byte. A `0` bit is followed by thirteen, which are an offset
into a window of `0x2000` — **and zero there ends the stream** — and then by four, which are three short of the
run to copy from that offset.

The window is `0x4b7e40`, written as the output is and from index 1, wrapping at `0x1fff`. A source read past
`[+0x14]` answers zero rather than stopping, which is what lets a truncated file come out as something.

The fields read out of the unpacked header, each by holding replays against each other that the game's own
list names:

| | |
| --- | --- |
| `+0x57` | the difficulty, in the order `Th07`'s own table has — `4` is the Extra and `5` the Phantasm |
| `+0x58` | the date as `MM/DD`, a NUL after it |
| `+0x5e` | the name, eight bytes, spaces to the end |
| `+0xe0` | `0100b`, which is the build |

## The name entry that screen starts in

The screen's phases are dispatched at `0x445dd4`: the byte table at `0x446a4f` indexes the jump table at
`0x446a1f`, and **phase `0xa` is the name entry**, whose handler is `0x446a66`. Phases `0xb` to `0xf` reach
`0x447198`, the replay's, and `0x10` and `0x11` reach `0x447fd0` — the walk the score file's own phases above
name.

`0x446a66` walks a cursor at `[this+0x20]` over the 96 characters at `[0x49ec30]` — ±0x10 a row, ±1 a column,
the one at index `0x5e` the space item, and a slot holding `0x20` skipped as the grid's own hole. On the shot
key it copies the character under the cursor into the name: `0x446faf` writes it at
`[this+0xbfaf + [this+0x10]]`, `0x446fc6` writes a space there where the space item was chosen, and `0x446fd7`
caps `[this+0x10]` at eight, putting the cursor on `0x5f` once eight are typed. The cancel and bomb keys (mask
`0xa`) are the backspace: `[this+0x10]` is taken down by one and `0x447087` writes a space where it now
points. So
the field is **`[screen+0xbfaf]`, eight bytes with spaces to the end** — inside the `0xce6c` bytes `0x44a302`
allocates.

**And it is the same eight bytes the header above carries at `+0x5e`**: seven of one machine's
`replay/th7_*.rpy` read out as one name in six and the same name less a letter in the seventh. Which is why orb reads the field
rather than asking for a name of its own — see `Game::the_name_its_own_entry_holds`.

## What is not established

- What `[0x12fe0d0]` is. It is `0x9a9b00 + 0x9545d0`, a field of the enemy manager, and a graze raises it at
  `0x43ed12`; what reads it is not.
- Which `.ecl` opcode writes the rank `0x62f8a4` through the dispatcher at `0x40ecf1` — the ECL variable that
  entry answers for — and whether any does.
- What the config's other fields in `[0x626274]` are to a run: only `+0x1c` and `+0x25` are read while a stage
  is played.
- What the first word of the object at `0x49fbf0` counts, which two of the four sound sites above hold against
  `0x12c` on the sixth stage — see *The music paused under a menu*.
- What the rest of a replay's header is. The shot, the score and the per-stage scores the slot's own stage list
  draws are all in there and none of them is placed; **and whether anything in it says the run was cleared is
  not read either**, which is the question it was opened for — the two Phantasm replays on the machine it was
  read on both end with the player out of lives, so there was nothing to hold a cleared one against.
- **Which frame a turning bullet turns on, held against what the frame does.** The bullet update calls five
  rebuilders of a bullet's step on flags of `[bullet+0xbf4]` (`0x425c0d`–`0x425c8a`): `0x425310` on `0x20`,
  `0x425400` on `0x40`, `0x425580` on `0x100`, `0x425700` on `0x80` and `0x4258a0` on `0xc00`. Two are read.
  `0x425700` rebuilds the step from the angle at `+0xbbc` and a speed of `[+0xbb0] × (1 − t / [+0xd30])`,
  which is a bullet slowing to a stop — a straight projection puts one *further* than it will be, so it is
  early and safe — and once `t` (`+0xd18`) reaches `+0xd30` it **aims the bullet again**, at the player
  through `0x442370` with the offset `+0xd20`, at the speed `+0xd1c`, `+0xd34` times over as `+0xd38` counts.
  `0x4258a0` is **a bullet that bounces**: where the bullet has left the field it takes the angle to `−a − π`
  off a side wall (x under 0 or past 384) and to `−a` off the top, or off the bottom (y past 448) where `0x400`
  is set, and rebuilds the step from the speed at `+0xd48`; `+0xd5c` counts the bounces and `0x425a23` clears
  both flags once it reaches `+0xd60`. A straight projection of either is wrong about its direction and not
  only its distance, and the frame each turns on is readable from those counters; the other three
  rebuilders are not read.
- What leaves a screen's vm pointer stale. `0x450db1` is `movsx edx, WORD PTR [ecx+0x4]` with `ecx` loaded
  from `[arg+0x1e0]` at `0x450d9c`, inside the script walk at `0x450d60`, whose own first act at `0x450d73`
  is to return where `[arg+0x1e0]` is null — so a fault there is a pointer that is neither null nor memory.
  `0x455170`'s own object is a screen and `[0x4b9e44]` the manager it hands a vm to at `0x4555a0`, guarded on
  the field `+0xb0c8` being non-zero; where that field is cleared, and whether the teardown clears it before
  or after the job is cut from the chain, is not read.
