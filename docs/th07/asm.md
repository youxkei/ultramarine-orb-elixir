# th07, as read off its exe

What is written here was read off `th07.exe` of **md5 `0126afce1e805370d36c3482445e98da`**, which is the one
build `game::th07` has addresses for. Every address below is that build's and means nothing about another.

To read it again:

```sh
i686-w64-mingw32-objdump -d -M intel th07.exe > th07.asm       # the whole of .text, once
i686-w64-mingw32-objdump -d --start-address=0x443550 --stop-address=0x4439a8 -M intel th07.exe
i686-w64-mingw32-objdump -s --start-address=0x49ecf8 --stop-address=0x49ed78 th07.exe   # a table
```

`objdump` loses the thread after a `jmp` table or a misaligned byte and starts printing data as
instructions; where a listing here disagrees with a dump, dump the one function again from its own `push
ebp`.

`.text` is `0x401000`, `.rdata` `0x48d000` (file `0x8c200`) and `.data` `0x49c000` (file `0x9ac00`), so a
string at an `.rdata` address is at `va - 0x48d000 + 0x8c200` in the file. The game's own log strings are
CP932.

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the
constant, and why the code is shaped that way is `docs/adr/`. What is here is what the game does, so that
the next person reading these functions starts where this stopped.

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

The rest of the game asks for a scene by writing one absolutely: 1 at `0x403881`, `0xa` at `0x403908`, 6 at
`0x403b8d`, `0x403bfe`, `0x404204` and `0x41e7a6`, 7 at `0x403bb8`. `0x426f60` groups 3, `0xb` and `0xc` —
it loads `0x498524`, 0x200 bytes through the sprite manager, for every scene that is *not* one of those three.

## The game manager, and what the run's own fields are

`0x42f3c5` registers scene 2's object, `0x626270`: a calc element at `0x62f8b4` and a draw element at
`0x62f8d4`, both priority 2, both with `0x626270` as their argument — update `0x42d8d5`, added callback
`0x42e83e`, deleted `0x42f2e4`, draw `0x42e1d4`. The object is at least `0x95e8` bytes, which is a manager's
size and not a screen's.

Its added callback writes `[this+0x93d7] = [this+0x93d6] + 2 * [this+0x93d5]`, the `character * 2 + shot`
every table of either game is indexed by — so **the character is `+0x93d5` and the shot type is `+0x93d6`**.
It branches at `0x42e894` on the scene wanted being **3** and skips the whole build there, which is th06's
`STATE_GAMEMANAGER_REINIT` shape: 2 is a run being built and 3 is the same manager re-initialised, which is
what a stage transition is. It also stamps `timeGetTime()` into `0x575ae4` and `1.0f` into `0x575ac8`, and
seeds the generator through `0x4318d0` on `0x49fe20` with the remainder of `0xffff`.

**Three objects come off the allocator (`0x47d441`) in it — `0x42e937`, `0x42e94e` and the `malloc` the seal
section names — and their sizes say which is which**: `[gm+0x0]`
is `[ebp-0x30] + 0x10` bytes, a length worked out from the run — the record of inputs a replay is; `[gm+0x4]`
is 0x38 bytes, zeroed straight after with a `rep stos` of 14 dwords, and is the run's header `[0x626274]`;
and `[gm+0x8]` is 0xc8 bytes, the run's own numbers at `[0x626278]`. `0x42ea66` to `0x42eae3` walks the third
and sets `+0x0`, `+0x4`, `+0x8`, `+0x18`, `+0x28` and `+0x2c` to nothing, the byte at `+0x20` to nothing,
`+0xc` to `0x186a0`, and `+0x30` to `0x32` or `0xc8` — the second where `[gm+0x10]` is 4 or more, which is
what makes **`+0x10` the difficulty**.

The fields below are named by a comparison against a limit or by the format they are drawn with, which is the
cheapest evidence there is and weaker than the write that moves a field. What each **is** to orb is beside
its own constant in `game/th07/mod.rs`; what is here is the rest of the object.

| field | evidence |
| --- | --- |
| `+0x0`, `+0x20` | **the score the panel shows and the continues used.** `0x42bdf8` draws `+0x0` with `%.8d` under a hundred million and `%.9d` over it, then the byte `+0x20` with `%1d` one glyph along — this game showing the continues as the score's last digit. `0x4042e0`, the continue path, adds one to that byte, and the ending chooser at `0x41e73f` reads it to pick `end00b.end` over `end00.end`: a run that continued gets the other ending |
| `+0x4` | **the score.** `0x42f45d`, the run's teardown, cuts both of the manager's chain elements, clamps this field to `0x3b9ac9ff` where it is `0x3b9aca00` or more, and copies it into `+0x0`. A cap of 999,999,999 applied as a run ends is a score's, and `+0x0` is the copy a panel's number chases |
| `+0xc` | **the high score shown, and not the next extend.** Drawn by the shape `+0x0` is drawn by — `%.8d` at `0x42bf0d` under `0x5f5e100` and `%.9d` over it — and the callback starts it at `0x186a0`, the 100,000 these games show before any run has beaten it. The build reads it out of the score file at `0x42e732` through `0x444f0d` and again at `0x42e79c` out of a table indexed by difficulty, and zeroes the byte `+0x10` beside it at `0x42e7f8`. A field a run *starts* from the score file with is a record and not a threshold |
| `+0x14`, `+0x18` | two counters of one event, incremented together by the `__thiscall` at `0x43eb90` whenever `[0x4d44f8]` is nothing, each under its own cap: `cmp DWORD PTR [eax+0x14],0x270f` at `0x43eba8` and `cmp DWORD PTR [ecx+0x18],0xf423f` at `0x43ebcb` |
| `+0x24`, `+0x28`, `+0x2c`, `+0x30` | **the point item system, and no rank anywhere in it.** An item collected at `0x433247` adds a tenth of its value to the score and one to both `+0x24` and `+0x28`; `0x4333bd` compares `+0x28` against `+0x30` and, where it has reached it, calls `0x42d83a` on the manager, adds one to `+0x2c` and sets `+0x30` to `(+0x2c - 2) * 0x1f4 + 0x320`. So `+0x28` is the point items collected, `+0x2c` how far the point value has climbed and `+0x30` the next step's count — and the `0x32` or `0xc8` the build puts there by difficulty is the first step, higher for the Extra and the Phantasm, **which is what made it look like th06's rank starting at 50 or 200** |
| `+0x88` | a score baseline. Every reader takes `[0x62f88c] - [run+0x88]` (`0x4048f7`, `0x41b3ff`, `0x433170` and on) and compares the difference against `0xc350` and `0x186a0` — 50000 and 100000, thresholds and not frame counts. The continue path copies the one into the other at `0x4043bf`, which is that difference being put back to nothing |

### The panel, and the route that is closed

`0x42bda0` to `0x42bfa0` draws it, four calls of the text drawing at `0x402060` into the buffer at
`0x134ce18` with the position advanced by a float between them: `[run+0x0]` with `%.8d` under a hundred
million and `%.9d` over it, then `(byte)[run+0x20]` with `%1d`; then `[run+0xc]` and `(byte)[run+0x10]` the
same way. Over a hundred million it writes `1.0` and `0x498b20` into the two floats at `0x13542dc` and
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
| the item manager | `0x575c70`, the third big static, whose init at `0x426f60` loads its own sprites and which the bomb calls `0x4326f0` and `0x433a90` on. Its own count is at `+0xae2ec`, kept by the walk at `0x4329e0` over `0x44c` slots of `0x288` bytes |

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
| 1 | the attract demo. `0x45bfb1` clears the replay bit where it is set and then opens `score.dat`, which is a demo ending through the same tail a run uses, and `0x4018e0` runs a block only during a demo, on `[player+0xa028]` and `[player+0x9e50]` |
| 2 | nothing has needed it. Read at `0x439195`, `0x4393a0` and three times around `0x442cd9` |
| 3 | a replay being watched. `0x455b45` is the load: it reads the file named at `0x62f654` through `0x431330`, takes it through `0x4433b0`, logs `0x4959a0` where nothing came back, sets bit 3 from `(1 & 1) << 3` and bit 1 beside it, clears `0x62f650`, and fills the run from the replay's header — its byte `+0x57` into `0x626280`, its `+0x56` halved into `0x62f645` |
| 4 | **a run that ended of its own accord, and it cannot be used for one.** `or 0x10` at `0x41e5a3`, `0x42a68e` and `0x42a6ba`, read at `0x446bb7` and `0x448a78`; `0x42a68e` also marks the table at `0x62e6c4` by difficulty and counts one into `0x62f52c`. A run given up through the pause menu does not set it, so the bit separates finished from left — **but nothing in the exe ever clears it.** The five `and` writes to this word clear bits 0, 1 and 3 and none of them touches `0x10`, so a second run of one launch reads as finished before it starts |

The pause menu reads two of them: bit 0 sends it to scene **6** and bit 3 to scene **7**, each after writing
`[run+0x4]` into `[run+0x0]` — the score and the number the panel chases being equalised as a run ends — and
the byte at `0x62f64d` is cleared on both paths. Those two scenes are a second reading of the two bits: a
practice run leaves to where its stage was chosen and a replay to where it was picked.

**The scenes with no name yet** are 7, 8, `0xa`, `0xb` and `0xc`. Nothing of orb's reaches them, and
`0x426f60`'s grouping of 3, `0xb` and `0xc` is where a read of them would start.

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

The title screen's own handler is `0x4555dd` and dispatches on `[front+0x8]`: phase 0 at `0x455621`, which is
the block the item dispatch at `0x455cdd` lives in, and 1, 2 and 3 at `0x45599d`, `0x45612e` and `0x456184`.

## The pause menu

`0x403b60`, a `__thiscall` on an object of five items of `0x24c` bytes each — fields at `+0x1c0` and
`+0x1ce`, a state at `+0x0` and a timer at `+0x4`. It plays `0xa` through the sound player at `0x4ba0d8` as
the cursor moves and asks for scene 6 at `0x404204` once that timer reaches `0x14`. The continue path
`0x4042e0` is inside the same function, so this is the pause menu with a continue on it.

## The score file's own phases

`[obj+8]` is the phase, and it decides everything.

| where | what |
| --- | --- |
| `0x44a302(kind)` | allocates `0xce6c` bytes and constructs the score-file object (`0x444a5b`). Five callers: `ecx = 2` at `0x434277` (after the message pump — the way out) and at `0x42e646`; `ecx = 0` at `0x437f3a`; `ecx = 1` at `0x438044` and `0x438283` |
| `kind 1` | `[obj+8] = 0xa`, or `0x12` where the practice bit is set, then registers the job `0x445db3` on the chain — the screen's own flow |
| `kind 2` | `[obj+8] = 0x13` and calls `0x449b05` inline: **the save on the way out** |
| `0x449b05` | reads `score.dat` (`0x449f82`), then parses its `CATK` records over `0x626288` (`0x44a059` → `0x445069`) — *skipped* where the phase is `0xa` or `0x12` (`0x44a040`, `0x44a049`) — then walks those records per shot into the object (`0x44a145`) and, **only where the phase is `0x13`** (`0x44a1e0`), writes the file (`0x44a1e9` → `0x44a1f9` → `0x44552c` → `0x431540`) |
| `0x445069` | the parse, `(ecx = the file image, edx = an array)`: for every `CATK` whose version byte is 1 and whose card number at `+0x28` is under `0x8d`, `rep movs` `0x1e` dwords from the chunk into `edx + card * 0x78`. File → memory, whole records, unconditional |
| `0x44552c` | the write, whose one caller is `0x44a1f9`: it serialises out of `0x626288` itself (`0x445713`), so what reaches the file is what memory holds. `0x44a1f9` writes only where the object still holds a file image, then frees it and clears the field |
| `0x42e634` | the second `kind 2`: it saves, then **zeroes every record** (`rep stos` of `0x1086` dwords over `0x626288`) and rebuilds each one's `CATK` header with its card number and no counts. Its one caller is `0x42eae7`, inside the game manager's added callback — so a run being built is what empties them |
| `0x42f94c` | not a merge, though it sits between the parse and the write: it counts the records whose last word is set and unlocks at `0x3c` of them |

So a `kind 2` save is a round trip that cannot carry anything — the parse overwrites memory with the file
immediately before the write reads memory — and no phase writes without it. **The parse has three call
sites**, which is what says where a session's counts can be put back: `0x42e735` inside `0x42e634`, *after*
it empties the records, so a run's own records are the file's cumulative with that run's counts already on
top; `0x44a059`, the save's own; and `0x45bff3` inside `0x45bf15`, the read the front end makes for its
menus, over the same array.

## The seal over a run's numbers

The run's own numbers live in one object, `[0x626278]`, `malloc(0xc8)` at `0x42e93f` with **no memset** —
what is not written is whatever the heap held.

`0x4012b0`, `GameManager::Reseal`, `__thiscall` on the game manager `0x626270`, writes five fields:

| field | what |
| --- | --- |
| `[run+0x3c]` | `(rand % 0x186a0) + 0x198f`, drawn from `0x49fe20` |
| `[run+0x98]` | a second draw, the same shape |
| `[run+0xac]` | a copy of `[run+0x3c]` |
| `[run+0xb0]` | the sum `0x42d7be` takes |
| `[gm+0x9614]` | the float of `[run+0xb0] + [run+0x98]` |

`0x42d7be` sums the **bytes** of four ranges: `run+0x34` through `run+0xac` (its length worked out at
`0x42d7cc` from those two field addresses), `[0x626278]+0xb4` for 0x14 bytes, all 0x38 bytes of the run's
header at `[0x626274]`, and 0x38 bytes at `0x575a68`.

`0x404fe0` is the check, and it makes **two comparisons and takes the sum again in neither**:

- `[run+0xac]` against `[run+0xbc] * 0xfc + [run+0x3c]`. `0xfc` is `(run+0xac) - (run+0x34) + 0x84`, worked
  out at `0x405003`; `[run+0xbc]` is nothing while a run is played, so this is `[run+0xac]` against
  `[run+0x3c]` — which a reseal has just made equal.
- `[run+0xb0] + [run+0x98]` against `[gm+0x9614]` rounded back to a whole number.

It answers 1 where either fails, and the caller then fills 0xb4 dwords from `0x575950` with `0xffffffff` —
the device pointer, the viewport, the pad mapping and the supervisor's scene are all inside that range, so
the game answers a changed number by destroying itself. Every guarded write of the game's own asks the
check first: `0x42d5d8` for the lives, `0x42d61d` for the bombs, `0x4325eb` for the power, `0x43b75b` for
the deaths, and five sites besides.

**`[gm+0x9614]` is `0x62f884`, and it is the seal's own value rather than anything about the cherry.** Every
`fstp` into it between `0x42dcb8` and `0x42e62c` is one of these calls written out inline.

### The noise the seal is drawn over

`0x42e3da`, called at `0x42e951` right after the `malloc`, fills the run's object with
`(rand % 0x186a0) + 0x198f` apiece: `[run+0x88]`, the seven words at `[run+0x34]`, eight at `[run+0x8c]`,
and float pairs at `[run+0x54]` and `[run+0x60]` — the words standing between the deaths at `+0x50`, the
lives at `+0x5c` and the bombs at `+0x68`.

`0x43b5c0` draws `+0x34` through `+0x44` and the floats at `+0x70`, `+0x74`, `+0x78` again outright, from
`0x43e38c` and `0x43eb74`. So none of them is a number a run carries, and nothing in the exe reads any of
the seven except `+0x3c`, which is the seal's.

`[run+0x88]` is the exception: it is the baseline every reader of the cherry takes off, so its random value
is the cherry's own obfuscation and not noise.

## Starting a replay, and starting one at a stage past the first

Inside `GameManager::AddedCallback` (`0x42e83e`), per stage, at `0x42f05f`:

```
if ([0x62f648] >> 3) & 1 {                  a replay is playing
    0x42e38c(gm)                            gm+0x9634, gm+0x963c, gm+0x9638 from [0x49f5d0 + shot*0xc]
    0x443aa0(ecx=1, edx=0x62f654)           the replay manager, reading
    save 0x49fe20 -> 0x4012b0 -> put it back
} else {
    0x443aa0(ecx=0, edx=0x497e1c)           the replay manager, writing
}
[gm+0x95e4] = [0x49fe20]                    the stage's own seed written down
0x4074c0(STAGE_BUILD)                       the stage laid out
```

So **the numbers go back before the seed is written down and before the stage is built**, and the reseal
between them is inside the replay branch alone.

`0x443aa0` registers the manager: `malloc(0xd8)` into `[0x4b9e48]` where there is none, and a chain job
apiece — `0x443040` at priority 0x10 for writing, `0x443550` at priority 5 for reading. **Where the manager
already exists it calls the job directly**, which is what every stage past the first does.

### `0x443550`, which is the whole of what a stage's start is put back from

`0x443571` through `0x44372f` runs once: the file read at `0x431330`, decoded at `0x4433b0`, and the seven
stage slots' pointers fixed up to absolute and their lengths worked out.

From `0x443734` is the part that runs for every stage:

```
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
| `0x4437ea` | `0x62f8a4` | `record+0x25` |
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
| `0x443931` | `[run+0x2c]` the point value's step | `record+0x18` |
| `0x443940` | `[run+0x30]` what its next step wants | `record+0x1c` |
| `0x443949` | `[replay+0xa4]` | the inputs |
| `0x44398c` | `[run+0x4]` and `[run+0x0]`, the score | the first dword of `[replay + 0x14 + stage*4]`, for stages 2 to 6 |

**And nothing at all of the player object**, `0x4bdad8` — not its state byte at `+0x2408`, not its
position. A replay started at stage 5 builds its player the same way a run does.

`[0x626274]` is the run's header, `malloc(0x38)` at `0x45bf3f`. Only two of its fields are read while a
stage is played: `+0x1c`, the difficulty, held against 3 at `0x42829a` and `0x42b492`, and `+0x25`, a flag
tested at `0x402b0d`, `0x42e0f7`, `0x43a19e`, `0x442d0e` and `0x442f02`.

**`0x62f8a4` does differ between a run played into a stage and a run built at it**, and a pair of launches is
what says so: the same stage 2 held `0x20` entered by a transition against `0x10` built by a resume, read off the
dump at the stage's start on both sides.

Two readings said otherwise and both were wrong in the same way. `0x42e38c`, which fills it out of
`[0x49f5d0 + shot*0xc]`, is called from `0x42eafb` as well as from `0x42f06c` — but `0x42eafb` is *inside* the
block `0x42e89b` jumps over, so a transition reaches neither the fill nor the zeroing of `[run+0x18]` at
`0x42ea7c` beside it. And the grep that found no other writer looked for the address as an operand:
**`0x40ecf1` hands the address out**, as one entry of the dispatcher that answers with a pointer to the
variable an index names — the entry beside `[0x626280]`, the difficulty — so what writes it is a script, and no
absolute store in the exe mentions it.

What reads it is the player's own per-frame code, nine sites in `0x40e` and `0x412`, four of them with
`fimul`: `0x412bab` works out `current + (target - current) * this / [0x498a84]` and writes it back where it
came from, over the pair at `[player+0x2ba8]` and `[player+0x2bac]` that `0x40fe0a` and `0x40fe17` set to
-0.5 and +0.5. So the number is a rate, twice the rate is a world whose player walks at twice the speed, and a
few hundred frames of that is a world that collects items the run did not.

`[0x626274]`, the run's header, is the other half of the same pair and needs no restoring: the two entries into
one stage 2 held all fourteen of its words the same.

### And the mirror: where a stage's record is written

`0x443040` writes one, indexed the same way, and its caller is `0x443d17` — `0x443aa0`'s mode 0, reached from
`0x42f1d0`, which is the **normal run's** branch of the same callback. `0x42f1d0` is past `0x42f0b4`, so a
stage's record is written *after* the stage has been built, while a replay's restore runs *before*.

It writes two things the restore above does not read back: the score, `record+0x0` from `[run+0x4]` at
`0x4431a9` — the restore takes that from `[replay + 0x14 + stage*4]` instead — and the word at `record+0x20`
from `0x62f854`. The header it fills is `0xe8` bytes with `[0x496aa8]` as its first dword and `0x1100` in the
word at `+0x4`; `+0x1c` and `+0x38` are seven slots each, a stage's record and a stage's inputs, `0x70800`
bytes of them, indexed by `[0x62f85c]`.

The float fields go in through `0x48b8a0`, which is `_ftol2`: `fistp` and then a correction by the sign and
the remainder, which is **truncation toward zero** and not a round.

## What a collection is worth

`0x42f5a2` is the cherry going up, `__thiscall` on the game manager taking an amount:
`[gm+0x961c] += amount` held at `[gm+0x9618]`, and `[gm+0x9620] += amount` where the amount is positive and
`[0x4bfee5]` is nothing. Those three are `0x62f88c`, `0x62f888` and `0x62f890` — the cherry, what it fills
towards, and the cherry+.

Its callers pass a constant at `0x433712` (0x14), `0x43372a` (0xa) and `0x433749` (0x1e), and a worked-out
amount at `0x4338d9` and `0x433998`. The worked-out one is `([run+0x18] / 40) * 10 + 300`, read at
`0x4338e3` — so **`[run+0x18]` decides what an item's cherry is**. `0x43369a` makes the same sum for the
score instead.

`[0x62f644]` is an index into the 31 dwords at `0x49ecf8`:

```
10 20 30 40 50 60 70 80 90 100 200 300 400 500 600 700 800 900 1000
2000 3000 4000 5000 6000 7000 8000 9000 10000 11000 12000 12800
```

`0x432efe` adds one to it, `0x432f19` holds it at 30, `0x432f27` reads the table by it and `0x432f42` adds
a tenth of what it finds to `[run+0x4]`. It goes back to nothing at `0x432fb2` and where the player is hit,
at `0x440d6a`.

`0x43eb90` adds one to `[run+0x14]` and one to `[run+0x18]` — the first held at 9999 and drawn on the panel
at `0x42a395`, the second at 999999 — spawns an effect at the player's position, plays sound 0x1e, and takes
**2000** points at `0x43ed12`, where it also advances `[0x12fe0d0]` by `([0x62f88c] - [run+0x88]) / 1500 * 20
+ 2500`. `0x43eb0d` calls it only where `[0x4bdad8+0x2408]` is neither 1 nor 2: `0x4410a0` writes 1 in the
player's death path, `0x43edc0` writes 2, and twelve sites between `0x408a23` and `0x40e234` write 3, each
after spawning four things and playing sound 13.

`[0x4bfeb4]` and `[0x4d44f8]`, which gate branches at `0x433691`, `0x433702` and `0x43eba1`, are **read and
never written** anywhere in the exe.

## A replay file, and what its header says

`"./replay/th7_%.2d.rpy"` (`0x4967bc`) is the name of a slot, and the replay list reads all fifteen of them
in the loop at `0x45aa11`: `0x431330` for the bytes and `0x4433b0` for the rest, which answers a pointer or
zero and is what a slot the list does not show has failed. What the list then copies out of that pointer is
`0xe8` bytes apiece.

`0x4433b0` is the whole of the reading, in four steps:

```
[+0x00] == [0x496aa8]                       the magic, "T7RP"
[+0x04] == 0x1100                           the version, as a word
key = [+0x0d]                               and from +0x10 to the end of the file:
    *p -= key; key += 7                      one byte at a time
sum = 0x3f000318 + every byte from +0x0d     against [+0x08], which is that sum written down
malloc([+0x18] + 0x54)                       the buffer the list reads
    the file's own first 0x54 bytes copied over it
    0x45ef00(src=file+0x54, dst=+0x54, [+0x14] in, [+0x18] out)
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
`0x447198`, the replay's, and `0x10` and `0x11` reach `0x447fd0` — the walk `docs`' own note above names.

`0x446a66` walks a cursor at `[this+0x20]` over the 96 characters at `[0x49ec30]` — ±0x10 a row, ±1 a column,
the 94th of them the space item, and a slot holding `0x20` skipped as the grid's own hole. On the shot key it
copies the character under the cursor into the name: `0x446faf` writes it at `[this+0xbfaf + [this+0x10]]`,
`0x447087` writes a space there where the space item was chosen, and `0x446fd7` caps `[this+0x10]` at eight. So
the field is **`[screen+0xbfaf]`, eight bytes with spaces to the end** — inside the `0xce6c` bytes `0x44a302`
allocates.

**And it is the same eight bytes the header above carries at `+0x5e`**: seven of one machine's
`replay/th7_*.rpy` read out as one name in six and the same name less a letter in the seventh. Which is why orb reads the field
rather than asking for a name of its own — see `Game::the_name_its_own_entry_holds`.

## What is not established

- What event the twelve `[player+0x2408] = 3` sites are. They are in the item code's own address range and
  each spawns four things and plays sound 13; which item or which collection is not read.
- What `[0x12fe0d0]` is.
- Which script instruction writes `0x62f8a4`, and what the number means to the pair it walks. That it changes
  during a run is established — a pair of launches read `0x20` against `0x10` — and so is the route, the
  dispatcher at `0x40ecf1`; which `.ecl` opcode takes that route is not.
- What the 54 bytes of `[0x626274]` nothing reads are for.
- What the rest of a replay's header is. The shot, the score and the per-stage scores the slot's own stage list
  draws are all in there and none of them is placed; **and whether anything in it says the run was cleared is
  not read either**, which is the question it was opened for — the two Phantasm replays on the machine it was
  read on both end with the player out of lives, so there was nothing to hold a cleared one against.
- **Which frame a turning bullet turns on, held against what the frame does.** Two of the three places a
  bullet's step is rebuilt are read: `0x4258a0`, on `[bullet+0xbf4] & 0xc00`, rebuilds it from the angle at
  `+0xbbc` and a speed of `[+0xbb0] × (1 − t / [+0xd30])`, which is a bullet slowing to a stop — a straight
  projection puts one *further* than it will be, so it is early and safe. `0x425700`, on `& 0x400`, negates
  that angle and rebuilds the step from the speed at `+0xd48`, which is **a bullet that turns round**: a
  straight projection of one is wrong about its direction and not only its distance. The frame it turns on is
  readable — `+0xd5c` counts up and `0x425a23` compares it with `+0xd60` — and nothing yet reads the third
  place.
- Where the difficulty is before a run starts. `0x459ead`, the shot select's own decide, copies it into
  `0x626280` out of the byte at `0x575a89`, which is where the difficulty select left it — so what that byte
  is written by, and whether anything else reads it, is not read.
- What leaves a screen's vm pointer stale. `0x450db1` is `movsx edx, WORD PTR [ecx+0x4]` with `ecx` loaded
  from `[arg+0x1e0]` at `0x450d9c`, inside the script walk at `0x450d60`, whose own first act at `0x450d73`
  is to return where `[arg+0x1e0]` is null — so a fault there is a pointer that is neither null nor memory.
  `0x455170`'s own object is a screen and `[0x4b9e44]` the manager it hands a vm to at `0x4555a0`, guarded on
  the field `+0xb0c8` being non-zero; where that field is cleared, and whether the teardown clears it before
  or after the job is cut from the chain, is not read.
