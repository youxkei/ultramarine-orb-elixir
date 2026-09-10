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
  (`0x426d74` onwards, four `fcomp`). Where the state at `+0x9e0` is 0 it calls `Player::Die` (`0x427770`)
  and answers 1 whether it did or not.
- **0** where neither.

`0x426df0` is the same test with `[0x46a548]` — 0.2667 — added to each side, and it is the graze: the
bullet's `+0x5c3` is set from its answer and the hit test is asked on the frame after.

### The count moves on the hit and the life goes later

`Player::Die` writes state 2 at `0x4277bc`, puts the timer at `+0x75b4` back to `-999`, plays the death sound,
and
**puts the death count up there and then**: `0x4277f9` reads `[0x69bcc0]`, which is `GameManager+0x20`, and
`0x427802` writes it back one higher. Nothing in the exe ever takes it down again — the only other sites are
the two reads the result screen makes (`0x42d58c`, `0x42d5a8`) — so what that field counts is **hits and not
lives lost**, and a hit a bomb saves is counted with the rest.

**Which leaves a window between the count and the death**, and it is `respawnTimer` at `+0x9d8`:

| | |
| --- | --- |
| `0x42896d` | `Player::OnUpdate` takes a bomb only where that field is not nought — with a bomb in hand and no dialogue up |
| `0x428a71`, `0x428a84` | the dying branch: state 2, and the field asked about again |
| `0x428a9a` | one off it for this frame, where it was not nought already |
| `0x428ab6` onwards | the frame it reaches nought, which is where the death is made final: the power items spawned and the power dropped, `[0x69d4ba]` read for whether there is a life to lose |
| `0x428f9e`, `0x42a09c` | what is left in it — 6 after a respawn, 8 by `Player::AddedCallback` at a run's start |

So a bomb pressed inside those frames — the four at `0x405577`, `0x4065a2`, `0x406ac7`, `0x407174`, each
writing state 3 over the 2 — is a deathbomb, and the run goes on with the life it had. The **life**
itself is not taken until the fade after all that: `0x428f19` onwards walks state 1, and `0x428ded` is where
`[0x69d4ba]` comes down.

**Which is what makes the count a poor thing for anything above the game to act on.** th07 moves its own in
the death handler, after its window — see `Th07`'s `DEATHS` — so a count that moved there is a death. Here it
moves as the hit lands, and the code that reads it has to ask `respawnTimer` what it means:
`Th06::where_a_counted_death_stands`.

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
| `+0x230`, `+0x234` | how far along it runs, near and far: `0x415e35` grows the far end by `+0x240` times the motion scale, and `+0x238` is the length it is held to. **A firing beam of this game reaches further every frame**, which is the one thing about one that moves — stage 2's boss fires icicles straight down and lets them extend, and a bot that took the length as settled stood five units under a far end and died there nineteen times |
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

## What starting a run does, and what an enemy is spawned out of

**The item is the shot type select's, `0x436dae` to `0x436fc5`**, and every write it makes is one of these:
the shot from its own cursor into `GameManager+0x181e`, the starting lives and bombs out of the options at
`Supervisor+0x12c` and `+0x12d` into `+0x181a` and `+0x181b` — 2 and 3 where the difficulty is Extra
(`0x436e0f`) — `currentStage` nought, or 6 for Extra, `isInReplay` nought, one call into the sound player to
fade the music, the multiplier the refresh ladder at `0x436e9d` picked into `Supervisor+0x1ac`, and
`curState` = 2 at `0x436e1d`. `0x436dbc` is where it goes to the practice stage select instead, on
`isInPracticeMode`.

**`GameManager::RegisterChain` is `0x41bb02`**, and three of the things it does are keyed on `currentStage`
as it stands at the build: the enemy manager registered with the pair at `[stage*8 + 0x4764e8]`
(`0x41c07d`), the ECL loaded by name from `[stage*4 + 0x4764c4]` into the interpreter at `0x487e50`
(`0x41c0c1`), and the game's own replay recorder — `ReplayManager::RegisterChain` at `0x42a240`, mode 0 for
recording and 1 for playing one back, chosen on `isInReplay`. **That one refuses to record while
`Supervisor+0x1ac` is under the 0.99f at `0x46b734`** (`0x42a246`), and writes 1.0 over it where it does; so
on a display the ladder puts under 0.99 the game keeps no replay of a run at all.

`Ecl::Load` is `0x407340`: `[0x487e50]` is the file as it was read, `[0x487e54]` is the sub table at
`+0x10`, and the loop at `0x4073a5` turns each sub's offset into a pointer. Stage 1's holds 24 subs, and the
timeline the enemy manager walks sits at `+0x472e` of the same block.

**`EnemyManager::SpawnEnemy` is `0x411390`, and what a new enemy starts as is a copy**: `rep movsd` of 0x3b2
dwords — one enemy — from the template at `manager+0x8` over the free slot (`0x4113f2`), then the life and
the position patched in, then two calls on the interpreter at `0x487e50` with `enemy+0x990` and the sub id
out of the timeline entry (`0x40744x` sets the context: `[ctx] = subs[subId]`, `[ctx+4] = -999`,
`[ctx+0x48] = subId`).

`EnemyManager::OnTick` is `0x4123e0`: 256 slots of `0xec8` from `manager+0xed0`, a slot skipped where bit 7
of `[enemy+0xe50]` is clear, the count written to `manager+0xee5bc`, and — where bit 2 of `[enemy+0xe51]` is
clear — `[enemy+0xc0]`'s `+0x2c` and `+0x30` read at `0x412499` and handed to a test with the enemy's own
position. **That read is where an enemy with no sprite takes the process down**, `+0xc0` being nought until
the ECL instruction at `0x40189a` writes `anmManager + (byte - 0x15) * 0x38` into it.

## The name entry a run ends at, and the name a replay carries

`ResultScreen::OnUpdate` is `0x42d98e` and dispatches on `[this+8]`; **state 9 is the name entry** and its
handler is `0x42bc85`. What that walks is a cursor at `[this+0x20]` over the 96 characters at `[0x4784d4]` —
±0x10 a row, ±1 a column, the 94th of them the space item, and a slot holding `0x20` skipped as the grid's own
hole. On the shot key it copies the character under the cursor into the name: `0x42c14e` writes it at
`[this+0x5193 + [this+0x10]]`, `0x42c165` writes a space there where the space item was chosen, `0x42c176`
caps `[this+0x10]` at eight, and `0x42c226` is the delete. So the field is **`[screen+0x5193]`, eight bytes
with spaces to the end**.

**And that is the name the game writes into a replay of its own.** Three of one machine's `replay/th6_*.rpy`
were read: the key is the byte at `+0x0e` and every byte from `+0x10` has it subtracted with the key climbing
by 7 — *the first byte included*, so the key the loop starts with is `byte + 7`, which is what told the three
files apart from every other candidate. What comes out holds `MM/DD/YY` and a NUL at `+0x10` and then eight
bytes at `+0x19`, spaces to the end: `YXK` in two of them and `YKI` in the third. The body is not compressed,
unlike th07's.

Which is why orb reads the field rather than asking for a name of its own — see
`Game::the_name_its_own_entry_holds`.

## The name of the track being streamed is nowhere in memory

`Supervisor::PlayAudio` is `0x424b5d` and takes the path the stage's data names its track by. It copies that
path into two of its **own stack buffers** — `[ebp-0x208]` and `[ebp-0x108]`, a byte at a time at `0x424bf1`
and `0x424c51` — finds the last dot in each with `strrchr` at `0x45ca20`, and writes `wav` over the extension
of the first and `pos` over the second's. Then it hands `[ebp-0x208]` to `SoundPlayer::LoadBGM` at `0x430a50`
and `[ebp-0x108]` to `0x430e10`, both with `ecx = 0x6d3f50`, the sound player.

`0x430a50` opens what it was given through a **stream object of its own on the stack**, `[ebp-0xac]`,
constructed at `0x43b9b0` and destroyed at `0x43ba10` before it returns — the open is `0x43ba60` at
`0x430b47` and the path is used once more at `0x430b5b`, to be printed in the failure it reports through
`0x41e940`. What outlives the call is the mmio handle, the length and the loop points, which is what
`wave_file`'s own offsets in `game/th06/mod.rs` read.

So **a launch cannot say which track is streaming by name**: the path is a local of a call that has returned.
What tells one track from another is `Th06::music_identity`, out of the wave file's length and its two loop
points, and that is what a line about the sound carries for this game — see `Game::the_track_heard`, which
answers nothing here. th07's own record does carry a name, so its line has one.

## The music thread refills once per wake and holds nothing against the cursor

`SoundPlayer::LoadBGM` (`0x430a50`) works a chunk out at `0x430c3c`–`0x430c82` as
`nSamplesPerSec << 1 × nBlockAlign >> 2` rounded down to `nBlockAlign`, which for the 44100 Hz 16-bit stereo
wav every track is comes to 88,200 bytes — half a second. `0x430ce0` passes 4 as how many of them there are
and `0x43aa6f` multiplies the two into `DSBUFFERDESC`'s `dwBufferBytes`, so the sound buffer is 352,800 bytes,
two seconds. `0x43b420` and `0x43acf5` put the two sizes where `streaming_sound::NOTIFY_SIZE` and
`streaming_sound::BUFFER_SIZE` in `game/th06/mod.rs` read them, with `BUFFER_COUNT` 1.

`0x430c8d` makes **one auto-reset event** with `CreateEventA`. `0x43ab1b` allocates a `DSBPOSITIONNOTIFY` per
chunk and `0x43ab4c`–`0x43ab81` fills each one's offset with the last byte of its chunk and each one's handle
with **that same event**, which `0x43ab94` hands to `SetNotificationPositions`. `0x430cbd` starts the thread at
`0x431370` that waits on it — the wait is on the player's two handles at `0x6d4580` with a 191 ms timeout, and
where it answers the first of them the thread calls `CStreamingSound::ServiceBuffer` at `0x43b560` and goes
round.

The refill locks the buffer at `[this+0x2c]` for `[this+0x28]` bytes (`0x43b61c`–`0x43b63a`, which are
`NEXT_WRITE_OFFSET` and `NOTIFY_SIZE`), reads the wav into it through `WaveFile::Read` at `0x43c080`, and moves
the offset on by a chunk. **`GetCurrentPosition` is called nowhere on that path**: where the next chunk goes is
a count of how many times the event has been consumed, and it is never held against the play cursor. So a
thread that does not reach the wait again before the next notification position comes round has two signals
collapse into one — an auto-reset event set twice is set once — refills once where it should have refilled
twice, and is a chunk out from then on. Nothing in the game brings it back; a new stream does, which is what
going to the title screen makes.

The ring is four chunks, so the first three missed refills are inaudible, each writing a chunk that the cursor
has yet to reach. The fourth writes the chunk the cursor is in, under the mixer, and a seam is heard at every
chunk boundary — twice a second — for as long as the stream lives. That is what a 紅魔郷 run under heavy load
heard from part way through stage 6's boss to the end of the run, and what `audio::Refills` counts and
`Th06::a_missed_refill_is_never_made_up` answers for.

## The plate a spell card's name is shown on

The panel keeps the name in an `AnmVm` at `impl+0x20f4` (`GuiImpl::enemySpellcardName`) and the plate behind it
in the one at `impl+0x1ed4`, the vms being 0x110 bytes apart (`0x1ed4`, `0x1fe4`, `0x20f4`, `0x2204`, … at
`0x41b3ca` onwards, one `AnmVm` constructor call each). `Gui::SetSpellcardName` (`0x417bfd`) picks the plate's
sprite into the first (`0x417c0c`) and bakes the name into the second (`0x417cb4`, then `DrawStringFormat` at
`0x417cc0`).

**Whether the plate is on the screen is one bit.** The panel's draw reads `[impl+0x2174] & 1` at `0x41b078` — the
name vm's flags word at `+0x80`, whose lowest bit is `AnmVmFlags::isVisible` — and where it is clear jumps past
both the plate (`0x1ed4`, positioned from the name's own position at `0x2184` and drawn) and the name
(`0x20f4`, drawn at `0x41b11b`). `AnmManager::Draw` (`0x432ad0`) asks the same bit of any vm handed to it
(`0x432adc`) and answers `-1` without drawing where it is clear. The bit is the ANM script's to set and clear:
the card's script is started on the vm as the card is shown, and the card's end sends the interrupt that hides
it (`Gui.cpp:35` and `:430` of the decompilation). So `[impl+0x20f4+0x80] & 1` is what `Th06` answers
`a_cards_name_is_shown` with.

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
