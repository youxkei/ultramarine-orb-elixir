# th06, as read off its exe

What is written here was read off `東方紅魔郷.exe` of **md5 `fa3d64768b1bfc50703dedc2db92f7fa`**, the 1.02h
build `game::th06` has addresses for. Every address below is that build's and means nothing about another.

To read it again:

```sh
i686-w64-mingw32-objdump -d -M intel 東方紅魔郷.exe > th06.asm          # the whole of .text, once
i686-w64-mingw32-objdump -d --start-address=0x426c40 --stop-address=0x426df0 -M intel 東方紅魔郷.exe
i686-w64-mingw32-objdump -s --start-address=0x4200f8 --stop-address=0x420118 東方紅魔郷.exe   # a table
```

`objdump` loses the thread after a `jmp` table or a misaligned byte and starts printing data as
instructions; where a listing here disagrees with a dump, dump the one function again from its own `push
ebp`.

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the constant,
and why the code is shaped that way is `docs/adr/`. Globals and field names come from the GensokyoClub/th06
decompilation, and what is here is what the functions do — so that the next person reading them starts where
this stopped.

## What a hit is, and the path to it

**One test, two walks and one kill.** `0x426c40` is the player's own box test, `__thiscall` on the static
`g_Player` at `0x6ca628`, taking a centre and a size and halving the size itself. It answers 2, 1 or 0:

- **2** where the box overlaps one of the 16 boxes at `player+0x8b8`, each `{x, y, w, h}` of 0x10 bytes and
  skipped where its `w` is nought (`0x426cc9`). What the callers do with a 2 is turn the bullet into an item —
  `ItemManager` at `0x69e268`, kind 6 — so these are a bomb's own clearing boxes and not the player.
- **1** where it overlaps the player's own box, which is the two corners at `player+0x458` and `+0x464`
  (`0x426d74` onwards, four `fcomp`). Where the state at `+0x9e0` is 0 it calls `Player::Kill` (`0x427770`)
  and answers 1 whether it did or not.
- **0** where neither.

`0x426df0` is the same test with `[0x46a548]` — 0.2667 — added to each side, and it is the graze: the
bullet's `+0x5c3` is set from its answer and the hit test is asked on the frame after.

`Player::Kill` writes state 2 at `0x4277bc` and puts the timer at `+0x75b4` back to `-999`; the four bombs
(`0x405577`, `0x4065a2`, `0x406ac7`, `0x407174`) write state 3 over it, which is what makes a bomb inside that
window a deathbomb.

**The player's own corners are worked out per frame** inside `Player::OnUpdate`: `0x427f1e` writes the min as
`+0x440` minus the half at `+0x488`, and `0x427f95` writes the max as the sum. The half is the shot's own and
differs by character, which is why nothing here is a hitbox written down.

### The two walks

`BulletManager::OnTick` at `0x4148f0`, `this` = `0x5a5ff8`:

| | |
| --- | --- |
| `this+0x5600` | the bullets, 0x280 of them every 0x5c4 bytes — `0x4149dd`, `0x414a26`, `0x414a2f` |
| `+0x5be` | the state: nought is an empty slot (`0x414a41`), and `state - 1` indexes the five-case table at `0x4164e0` |
| `+0x560` | where it is, and `+0x550` the size the test is given (`0x415bf0`, `0x415bfa`) |
| `+0x56c` | the velocity, worked out from the angle at `+0x590` and the speed at `+0x584` by `fsincos` (`0x414f4c`) |
| `+0x5c0`, `+0x5b8` | a counter and the flags its own scripting reads |

`0x415986` is the movement: the velocity times `[0x6c6ec0]` added to the position, three components. **That
scale is the whole of what a frame moves anything by** — `0x427da5` uses it on the player's step too.

The hit test is reached at `0x415c00` from the tail every state but the fifth falls through to; the fifth's
case is `0x415c4b`, past that block, and `0x415c11` is what puts a bullet that has just hit the player into
it. So state 5 is the one live state that cannot hit.

`EnemyManager::OnTick` at `0x4123e0`, `this` = `0x4b79c8`:

| | |
| --- | --- |
| `this+0xed0` | the enemies, 0x100 of them every 0xec8 bytes — `0x4123f8`, `0x412422`, `0x41242a` |
| `+0xe50` | bit 7 says the slot holds one (`0x41243a`) |
| `+0xe51` | bits 0 and 1 gate the collision (`0x4126a6`, `0x4126c2`); with either down the test is not made |
| `+0xc6c` | where it is, and `+0xc78` the size — scaled by `1.0/1.5` at `0x4126e5` and halved by the test, so a third of it on each side |
| `+0xce4` | the life, which a body hit takes ten off (`0x41278e`) where bit 3 of `+0xe51` is down |

The arrays check each other: the bullets end at `0x5600 + 0x280 * 0x5c4` = `0xec000`, which is where the
lasers are, and the enemies end at `0xed0 + 0x100 * 0xec8` = `0xed6d0`, below the boss pointer at `0xee598`
and the live count at `0xee5bc`.

## The lasers

Also the bullet manager's, walked from `0xec000` in 0x40 slots of 0x270 (`0x415df4` onwards, and `0x414670`
is the spawn looking for a free one). `[+0x258]` is what says a slot holds one.

| | |
| --- | --- |
| `+0x220` | where it is fired from, and `+0x22c` the angle it points at |
| `+0x230`, `+0x234` | how far along it runs, near and far: `0x415e35` grows the far end by `+0x240` times the motion scale, and `+0x238` is the length it is held to |
| `+0x23c` | how wide it is |
| `+0x26c` | the phase, switched on at `0x415f75`: 0 draws the warning line, 1 fires, 2 fades |
| `+0x244`, `+0x264` | how long the phase lasts and how long it has run — `0x41615b` moves the phase on where the frames run out, so in phase 0 the difference is the frames until it starts hitting |

**The player's own test is called from the firing phase and from no other**, at `0x416203`, and it is a
different function: `0x427190`, which turns the box into the beam's own frame — along it and across it.

## The items

`ItemManager::OnTick` at `0x41f4a0`, `this` = `0x69e268`, which *is* the array: 0x200 slots of 0x144
(`0x41f4af`, `0x41f51e`, `0x41f527`), `[+0x141]` alive, `[+0x110]` where it is, `[+0x11c]` its velocity with
the y at `+0x120` capped at 3.0 and pulled by `[0x46b144]` a frame.

`[+0x140]` is which of the seven it is, and the table at `0x4200f8` is what each does:

| | | |
| --- | --- | --- |
| 0 | `0x41f948` | the power item: `+1` at `0x41f95f`, to 128 |
| 1 | `0x41fae6` | the point item |
| 2 | `0x41fd78` | the big power item: `+8` at `0x41fd8e` |
| 3 | `0x41ff02` | the bomb: `[0x69d4bb] + 1` at `0x41ff0e`, to 8 |
| 4 | `0x41ff89` | full power, which compares `[0x69d4b0]` with 128 |
| 5 | `0x41ff3d` | the 1up: `[0x69d4ba] + 1` at `0x41ff49`, to 8 |
| 6 | `0x420028` | what a bullet a bomb cleared turns into, worth a score off the chain |

`0x69d4ba` and `0x69d4bb` are `GameManager+0x181a` and `+0x181b`, the lives and the bombs, which is what says
which of 3 and 5 is which.

## The shot is also the focus key, and a word orb hands in does not go through that

`0x41cfc0`, the controller read, counts the frames the shot has been held in `[0x69d8f4]` — up to 16 at
`0x41d06e` — and **once that count is 8 it sets bit `0x4` in the word it hands back** (`0x41d09b`). Bit `0x4`
is the focus bit the movement switch reads, so holding the shot *is* holding focus in this game.

**And one frame of letting go does not undo it**: `0x41d0a3` takes 8 off the count where it is above 8 and
zeroes it only where it is not, so a release from 16 lands on 8 — which is still the count that sets the bit.

**And none of it reaches a word orb hands in**, which is the half that matters here: that read is one orb
replaces, so a bot's word goes to the game's own movement without passing through the counting. Measured — a
launch driven with the shot let go every eight frames carried the focus bit on exactly the frames a boss was up
and on no others, which is the rules' own `Keys::focus` and not the holding. What the counting still decides is
the word a *pad* hands in, which is `Th06::focusing`.

## What the player's step is

`Player::OnUpdate`'s movement, `0x427a09` onwards: `[player+0xa0c]` is the direction, 1 through 8, and its
switch reads one of four speeds — `+0x9f4` and `+0x9f8` for a straight move, `+0x9fc` and `+0xa00` for a
diagonal one, the second of each pair where the focus bit `0x4` is in `[0x69d904]`.

`0x427d75` keeps the pair it chose at `+0xa10` and `+0xa14`, and `0x427da5` and `0x427ddb` add them to the
position: each times the axis's own scale — `+0x9d0` across, `+0x9d4` down — times `[0x6c6ec0]`.

Then the clamp, `0x427e03` onwards: against `[0x69d6ec]` and `[0x69d6ec] + [0x69d6f4]` in x, `[0x69d6f0]` and
`[0x69d6f0] + [0x69d6f8]` in y. Those four are `GameManager+0x1a4c` and `+0x1a54`,
`playerMovementAreaTopLeftPos` and its size — **the field a move of the player's can land in**, which is not
the field its bullets fly through: `0x4156e3` and `0x415748` bounce a bullet off 0 and `[0x46a7c0]` = 384 in x
and 0 and `[0x46a7bc]` = 448 in y.

## What is not established

- Which of the five bullet states is which kind. That nought is an empty slot and 5 is one that has just hit is
  read; what the four cases at `0x414e8c`, `0x414a9e`, `0x414bda` and `0x414d15` are *for* is not, beyond that
  each ends in the same tail.
- What the flags at `[bullet+0x5b8]` mean, one bit at a time. Bit 0 is read where the velocity is worked out
  from the angle, bit 4 and bit 6 gate two of the scripted paths, and 0x800 and 0x100 are tested in the tail;
  none of them is named.
- What the 16 boxes at `[player+0x8b8]` are written by. That a hit on one turns a bullet into item 6 is read,
  and a bomb is what does that in the game — the writer is not found.
- Whether anything on an enemy is a step. The reading is that nothing is, which is why `what_a_bot_sees` hands
  a body over with none; what moves one is its own script, and no field of it was held against two frames.
- Which of `+0x24c` and `+0x250` bounds the laser's firing and fading phases. Both are compared against
  `+0x264` in that switch; which is which was not needed and was not read.
