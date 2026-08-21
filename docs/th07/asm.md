# 妖々夢, as read off its exe

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

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the
constant, and why the code is shaped that way is `docs/adr/`. What is here is what the game does, so that
the next person reading these functions starts where this stopped.

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
what says so: the same 2面 held `0x20` entered by a transition against `0x10` built by a resume, read off the
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
one 2面 held all fourteen of its words the same.

### And the mirror: where a stage's record is written

`0x443040` writes one, indexed the same way, and its caller is `0x443d17` — `0x443aa0`'s mode 0, reached from
`0x42f1d0`, which is the **normal run's** branch of the same callback. `0x42f1d0` is past `0x42f0b4`, so a
stage's record is written *after* the stage has been built, while a replay's restore runs *before*.

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

## What is not established

- What event the twelve `[player+0x2408] = 3` sites are. They are in the item code's own address range and
  each spawns four things and plays sound 13; which item or which collection is not read.
- What `[0x12fe0d0]` is.
- Which script instruction writes `0x62f8a4`, and what the number means to the pair it walks. That it changes
  during a run is established — a pair of launches read `0x20` against `0x10` — and so is the route, the
  dispatcher at `0x40ecf1`; which `.ecl` opcode takes that route is not.
- What the 54 bytes of `[0x626274]` nothing reads are for.
