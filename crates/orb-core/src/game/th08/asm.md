# th08, as read off its exe

What is written here was read off `th08.exe` of **md5 `77b6785e04a3406e50be68714a193650`**, 840,704 bytes,
which is the one build `game::th08` has addresses for. Every address below is that build's and means
nothing about another.

To read it again:

```sh
i686-w64-mingw32-objdump -d -M intel th08.exe > th08.asm        # the whole of .text, once
i686-w64-mingw32-objdump -d --start-address=0x441e70 --stop-address=0x442052 -M intel th08.exe
i686-w64-mingw32-objdump -s --start-address=0x4c7fac --stop-address=0x4c7fd4 th08.exe   # a table
```

`objdump` loses the thread after a `jmp` table or a misaligned byte and starts printing data as
instructions; where a listing here disagrees with a dump, dump the one function again from its own
`push ebp`.

The sections are `.text` at `0x402000` (file `0x1e00`), `.rdata` at `0x4b4000` (file `0xb3a00`),
`.data` at `0x4c6000` (file `0xc4e00`) and `.data1` at `0x18da000` (file `0xcbc00`), so a string at an
`.rdata` address is at `va - 0x4b4000 + 0xb3a00` in the file. The game's own log strings are CP932.

**`.data` is 0x6e00 bytes in the file and twenty megabytes in the image**, which is what makes this game
read differently from th06 and th07: `SizeOfImage` is `0x14dc000` where `.data`'s raw size ends at
`0x4cce00`, so everything between there and `.data1` at `0x18da000` is zero-filled at load. The game
manager, the input word and the stage number all live out in that span — `0x160f508`, `0x164d528`,
`0x164d2cc` — and none of them is in the file. Which is also why a static of this game cannot be found by
grepping the file's bytes for its value the way th06's can.

**Which orb reads out of the PE and writes in the log every launch**, and that line is the range a
chapter of this game would be a copy of: `.data 0x004c6000..0x018d9840 (21051456 bytes)`.

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the
constant. What is here is what the game does, so that the next person reading these functions starts
where this stopped.

**And two spellings of one address disagreeing is the cheapest evidence there is, worth looking for on
purpose.** This game holds its globals as fields of the game manager as well as absolutely, so the same
word has two names — `[gm+0x3ddc4]` is `0x164d2cc` — and the offsets and the absolute addresses are the
same numbers twice. Every address below that has two spellings says so.

## What is not established

- **Where a shot's hit pays a time orb.** The human–youkai gauge is placed — `[run+0x22]`, clamped at
  0x43c0bb and moved every frame by the sign of `[player+0x3]` at 0x44beb6 — and a launch unfocused under a
  boss read it at `-10000`, its clamp, so the human end is the negative one. An enemy *killed* at either end
  drops one orb (0x42da25); what a *hit* pays at the human end, which is the strategy wiki's
  `人間逢魔が時にショットを敵に当てる` and what `Frame::shoots_a_boss_unfocused` plays for, is not found in
  the exe. That stage 1 rose from 2,094 orbs to 3,171 with the focus let go under its boss is the whole of
  what says it.
- **What the two unnamed item classes are worth.** All nine of them are read below; 6 is score off the
  graze and 7 is what a cancelled bullet leaves, and the shared vocabulary `game::Worth` has no name for
  either, so both go over as `Worth::Other`.
- **Which entries of an enemy's trail are not boxes.** The walk is read — see *The enemies* — and orb
  hands over the ones it visits, every sixth from the first. What the five entries between each pair are
  for is not read: the array holds one entry a frame and the walk tests one in six of them, so the
  others are either a drawing's own or a finer trail nothing collides with.
- **What a pad's directions do to the word.** The pad's own read is read — see *The pad, which is
  WINMM's and not DirectInput8's* — and its buttons go through the same fold the configuration's do;
  what is not read is whether the stick's axes or a hat reach that word, and through which fields of the
  `JOYINFOEX` the read fills. Nothing of orb's needs it while both pad hooks are declined.
- **Where the stage script's clock is put back.** That it *is* put back a launch measured — see *The
  clock the stage's own script is run on*, where stage 2 counts from nothing — and what is not read is
  the write that does it: the one write of a context's `+0xc` is 0x42c814's, which fires only where that
  field is still nothing, and the added callback that loads the next stage's `.ecl` leaves the array
  alone. Nothing of orb's needs the write, the clock's own behaviour being what a midstage table is
  written in.
- **What the sixth of `PLST`'s blocks of counts is for.** Every field of `PLST` is read — see *The score
  file* — and the blocks are indexed by `[0x160f538]`, which holds Easy to Extra as 0 to 4: what puts a 5
  there, whose runs the sixth block counts, is not found. `FLSP` is its nine-byte header and nothing of
  the `0x17` bytes after it is read by the exe at all.
- **What a sprite record of an `.anm` slot holds past its texture.** The slot's own six fields, the
  `0xc`-byte entry a header's texture and block are, and the `0x44`-byte sprite record's `+0x4` are read
  — see *The `.anm` slots the same object holds* — and the rest of those `0x44` bytes is not: nothing of
  orb's reads a piece of a sheet, `Th08::panel_tile` answering `None` for that reason.
- **Three of the record's fifteen fields**, which orb carries by address rather than guessing at:
  `[run+0x1c]`, `0x164d334` and `0x164d0b0`. A resume writes each back where the game's own restore puts
  it, so what is not known is only what they mean. The fourth it carried that way, `[run+0x22]`, is the
  gauge above.
- **Where in its track the stream is, and what the four calls a frame make of it.** The manager, its
  extent, the stream's own extent, the event and the thread a played track is, the three slots a stage's
  build fills, the call that plays one and the record the source is reading are all read — see *The
  sound* — so which track is playing is `Th08::music_identity`'s answer. What is not read is the stream's
  position in it: the four methods 0x447764 calls on the stream each update, one of which is where the
  refill's cursor moves. Nothing of orb's needs it, a restore leaving the stream alone and the song playing
  on from where it had got to, so `Th08::music` answers `None` for that reason as much as for this one.

## The window, the device and the frame

WinMain is **`0x4418c0`**. It writes `hInstance` into `0x17ce758` at 0x4418d0, reads three
`SystemParametersInfoA` answers into `0x17ce720`, `0x17ce724` and `0x17ce728`, takes
`QueryPerformanceFrequency` into `0x17ce714` at 0x441982, and then loops from 0x441ab9.

**The loop is three things.** `PeekMessageA` (0x441ad2) with `TranslateMessage`/`DispatchMessageA`
behind it; then `[[0x17ce760]+0xc]` on nothing pending, whose answer is compared against `0x88760869`
at 0x441b36 — `D3DERR_DEVICENOTRESET`, so that call is `IDirect3DDevice8::TestCooperativeLevel` and
**`0x17ce760` is the device**; and then `0x441e70` as a `__thiscall` on **`0x17ce700`**, which is the
frame. `0x17ce700` is the HWND as well as the object's own `+0x0`: it is what `ShowWindow` (0x441c88),
`MoveWindow` (0x441c9f) and `DestroyWindow` (0x441cab) are handed at the teardown.

The device's vtable is read at six slots and every one of them agrees with `IDirect3DDevice8`:
`+0xc` TestCooperativeLevel, `+0x14` ResourceManagerDiscardBytes (0x445738), `+0x38` Reset (0x441b5d),
`+0x3c` Present (0x44207f), `+0x88` BeginScene (0x441fc2), `+0x8c` EndScene (0x442022), `+0xa0`
SetViewport (0x441f42) and `+0xf4` SetTexture (0x44200e).

**The frame at `0x441e70`, in order**: the time out of `0x442190` into `[this+0x2c]`; the limiter over
`[this+0x34]`, `[this+0x3c]` and the double at `0x4b5bd8`, which returns through `Sleep(0)`
(0x442046) on a frame that is early; the queue of quads drawn once (0x441f02); `SetViewport` with the
whole output written into `0x17ce820` as `{0, 0, 0x280, 0x1e0}`; the **calc chain** through
`0x43ca50`; the sound queue through `0x45d790`; the frame-skip counter `[this+0x10]` against the byte
`0x17ce893`; `BeginScene`; the queue emptied (0x441fce), the fog flag forced to `0xff` (0x441fd3) and
the fog put out (0x441fe2); the **draw chain** through `0x43cb60`; the queue drawn (0x441ff7);
`SetTexture(0, null)`; `EndScene`; the time again; and `0x442060`, which presents.

**Which is the update before the draw, where th07's frame has them the other way round.** So orb's own
loop over this game keeps the order the game already had rather than turning one round, and what it
takes out is the frame-skip counter and the limiter.

**And what the frame answers is the calc walk's answer, mapped**: `0` from that walk makes the frame
answer `1` and `-1` makes it answer `2` (0x441f5f through 0x441f8e), and every other frame answers
`0` (0x44204c). Those are the three the message pump reads.

So the two seams orb's own loop needs are the chain's two runners and not the frame's own address, the
same as th07 — and `0x442060` is what `FrameCalls::present` is, since that is where this game's
`Present` is called from.

### The queue of quads the drawing fills

**`0x18bdc90`** holds the pointer to it, `SprtCtrlInf`, the `0x2a2570`-byte allocation WinMain takes
at 0x4419d0. Three functions move it and **every offset below is read off those three**:

| | |
| --- | --- |
| `+0x18` | how many times it has been drawn, `+1` at 0x462ef9 |
| `+0x2520` | how many quads are in it |
| `+0x2524` | where the buffer starts, which an empty queue's cursors point at |
| `+0x2a2524` | the cursor the next quad is written at |
| `+0x2a2528` | the cursor the next draw starts from |

- **`0x462e00` empties it**: the count to nothing (0x462e0a), the write cursor back to `this+0x2524`
  (0x462e20) and the drawn-from cursor after it (0x462e32).
- **`0x462f10` queues one**: six `0x1c`-byte vertices copied out of the four it is handed — the source
  offsets are 0, `0x1c`, `0x38`, `0x1c`, `0x38`, `0x54`, which is the two triangles of a quad — the
  write cursor `0xa8` further on (0x462fb5) and the count one higher (0x462fcd).
- **`0x462e40` draws what is queued**: nothing at all where the count is nothing (0x462e4a), and
  otherwise two `SetTextureStageState` through `[device+0xfc]`, `SetVertexShader(0x144)` through
  `[device+0x130]`, `DrawPrimitiveUP(4, count * 2, [this+0x2a2528], 0x1c)` and `[device+0x120]` — then
  the drawn-from cursor up to the write cursor, the count back to nothing and `+0x18` one higher.

**orb reads none of these**, calling only the first and the last of the three; they are read because
`th08/image.rs` lays the object out for `orb-e2e`, and a laid-out queue that did not answer the way
the real one does is a frame of orb's loop passing here and faulting on the machine.

### The `.anm` slots the same object holds, and this game's textures with them

The object at `0x18bdc90` is the sprite manager as well as the queue, which is how th07 keeps its own:
`+0x24` begins an array of **`0x100`** slots of **`0x1c`** bytes, one `.anm` file loaded into each.

**The count and the stride are the free's own.** 0x466060, `__thiscall` on the manager with a slot,
declines a slot below 0 or `0x100` or more (0x46606b and 0x466071), indexes `slot * 0x1c` and reads
three fields of it; the loader from 0x4658ca writes those three and three more at
`this + slot * 0x1c + 0x24`:

| | |
| --- | --- |
| `+0x0` | the slot's own number |
| `+0x4` | the loaded file — nothing here is a slot with nothing in it (0x466088) |
| `+0x8` | how many headers the file chains, counted by walking `[hdr+0x38]` (0x465947) |
| `+0xc` | the sprites, `0x44` bytes apiece — 0x406880 answers `[+0xc] + i * 0x44`, which is what a game sprite's `+0x224` is set from (0x402d44) |
| `+0x10` | one dword per script |
| `+0x14` | one `0xc`-byte entry per header, which 0x466060 walks `[+0x8]` of |

**An entry of that last array is a texture and a block**, off 0x466160: it releases `[entry+0x0]`
through its vtable's `+0x8` — a COM object, so that field is the texture DirectX made — and hands
`[entry+0x4]` back to the game's own allocator at `0x164f598`. Which makes the array the one place a
slot's handles live, and `Th08::live_handles` is a range per loaded slot over it.

The draw reads the texture back out through the sprite rather than the slot: 0x462cd3 takes
`[[sprite+0x224]+0x4]` into the queue's `+0x24bc` and 0x462d07 hands that to `SetTexture`.

## The chain

**`0x164f548`** is the chain, `0x43ca50` runs its calc list and `0x43cb60` its draw list, both
`__thiscall` on it with the prologue `55 8b ec 83 ec 18`. **The calc list's first element is the chain
itself** — 0x43ca6c takes `this` as the element it starts from, not a pointer kept at `+0x0` — and the
draw list's is `this + 0x20` (0x43cb70 through 0x43cb76). Both have a null callback, which the walk
skips (0x43ca7f, 0x43cb92), so the first job of each list is that element's next.

An element is walked as `mov ecx,[elem+0x1c]; call [elem+0x4]` (0x43ca9b and 0x43ca9e, and again at
0x43cbae), so **`+0x4` is the callback, `+0x14` the next and `+0x1c` the argument** — th07's shape
exactly. `0x43c880` adds a calc element and `0x43c960` a draw one, each taking the element in `edx`…
`push`ed and the priority pushed after it (0x43aaa5 through 0x43aab1).

`0x43ef70` and `0x43efb0`, called on `0x17ce758` either side of every callback, are the profiler's
brackets and nothing a run depends on.

**What a walk answers is the count of the jobs it ran**, `[ebp-0x8]` climbing by one per callback that
was reached (0x43cb11 and 0x43cc1f) — and a callback whose `+0x4` is null is skipped without being
counted (0x43ca7f, 0x43cb92).

**And each callback's own answer is switched on**, through the table at `0x43cb38` for the calc list's
seven cases and the one at `0x43cc46` for the draw list's six:

| answer | what the walk does |
| --- | --- |
| 0 | takes the element off the chain through `0x43cf50`, counts it, and goes on (0x43cac6) |
| 1 | counts it and goes on, which is what a job that carries on answers |
| 2 | calls the same callback again (0x43caec) |
| 3 | leaves the walk with **1** (0x43caf7) |
| 4 | leaves the walk with **0** (0x43caee) |
| 5 | leaves the walk with **-1** (0x43cb00) |
| 6 | starts the whole walk again from the head (0x43cb09) — calc list only |

So the three numbers orb's own loop has to read are the walk's `0` and `-1`, which the frame turns
into its own two exits, and anything else, which is a frame that carries on. Exactly th07's shape at
this game's addresses.

## The supervisor and the scenes

**`0x17ce758`** is the supervisor: its `+0x8` is the device (`0x17ce760`), its constructor at 0x445b7a
zeroes `0xd9` dwords, and 0x441850 fills the same `0xd9` dwords with `0xffffffff`, so the object is
`0x364` bytes.

Its update is **`0x445453`**–`0x445b79`, the address 0x445ed2 pushes as it registers the element.
`[this+0x158]` and `[this+0x15c]` are compared at 0x44568a,
and where they differ the scene being left is kept in `[this+0x160]` and `scene %d -> %d` (`0x4b6614`)
is printed with `+0x15c` pushed first and `+0x158` second. So **`+0x158` is the scene now and `+0x15c`
the scene wanted**, which puts the scene wanted at `0x17ce8b4` absolutely — the address the rest of the
game writes a scene into, at nineteen sites.

`[this+0x154]` is the update's own frame counter: incremented at 0x445b1a and, every `0xfa0` frames,
used to write `0100d` (`0x4b660c`) through 0x44858d.

**The transition is a switch on the scene being left**, with cases at 0x44571c (0), 0x445753 (1),
0x445821 (2), 0x4457eb (5), 0x445a1c (6), 0x445a61 (8) and 0x445a94 (9) and a default at 0x445adf —
the same seven th07 has. Inside each is a switch on the scene wanted, and each branch calls the
register of the scene being entered:

| wanted | out of | register |
| --- | --- | --- |
| 1 | 0, and 2 by way of 0x44571c | `0x47146d` with `ecx` 0, or 1 out of scene 7 |
| 2 | 1 | `0x43aa5c` |
| 3 | 2 | `0x43aa5c`, the wanted scene left at 3 for the build to read |
| 5, 6 | 1, 2, 9 | `0x4582a0`, `ecx` 0 for 5 and 1 for 6 |
| 8 | 1 | `0x449254` |
| 9 | 1, 2 | `0x4296a0`, the ending |
| -1 | any | nothing; the update answers 4 and WinMain leaves the loop |

**Scene 9 is the ending, and `0x4296a0` says so in its own words.** It allocates `0x2ab8` bytes, hands
`0x4b4a0c` — **`EndingInf`** — to the chain's own naming call at 0x429de0, hangs a calc element with the
callback `0x429860` at priority 5 and a draw element with `0x4298f0`, and writes `0x429980` into the
object's `+0x8` and `0x429d40` into `+0xc`. The three script names sit in `.rdata` right after that one:
`end01c.end`, `end02c.end` and `end03c.end` at `0x4b4a18`, `0x4b4a28` and `0x4b4a38`. A run that clears
goes from scene 2 into this one and the screen a run ends at is registered out of it, which is the row
above.

**And a run's build is a thread, which the register only starts.** `0x43aa5c` adds the run manager to
the chain and the chain runs its added callback, 0x43aaf4, on the spot — but that callback is forty
instructions: it branches on the scene wanted at 0x43aafd, sets the loading screen up through 0x44886f
and 0x4489ca, and pushes **`0x43abd7`** and `0` to `0x4486ba`, which is `CreateThread` (import
`0x4b40b0`) with the handle kept at `[this+0x284]`, `[this+0x290]` set to 1 beside it and
`[this+0x294]` the load pending. It returns at 0x43abd6 with nothing of the stage in place, and
`[this+0x158] = [this+0x15c]` at 0x445b0b follows in the same update. **`0x43abd7`–0x43b935 is the
build**, on that thread: the last run's two blocks freed (0x43acd3, 0x43acf5) and this one's allocated —
`0x10` bytes at 0x43ad33, `0x3c` at 0x43ad40, `0xe4` at 0x43ad57 into `[gm+0x8]` — the configuration's
`0xf` dwords copied in at 0x43ad7c, the starting lives at 0x43ae42 on the run-start branch, the player's
build through 0x44c230, the seed's copy at 0x43b3cb and 0x409b20 three instructions after it.

The supervisor's update reads `+0x290` at 0x4455a4 and answers 1 at 0x4455bc while it is set, before its
sound and its input read; `0x44871b` is the join — `WaitForSingleObject` (`0x4b40b4`) on the handle,
`CloseHandle` (`0x4b405c`), both words cleared. **So the moment a stage's numbers are in place on the
main thread is the first frame `+0x290` reads clear**, which `Th08::a_stage_has_just_been_built` answers
and `Th08::mid_build` is the complement of. Measured: a patch over 0x43aaf4 recorded
`score: 107981937, lives: -128, deaths: -2147483648` at its return on two launches — a block nothing had
written — and the run's own frame 0 read the same bytes where frame 1 read the run.

**And the 1 is not a stop.** The walk's table at `0x43cb38` takes 1 to 0x43cb0e — the count stepped and
the next job — so every job on the chain behind the supervisor runs on those frames too. And the build puts
the stage's jobs on the chain part way through: straight after the seed's copy and 0x409b20, 0x4311a0
(0x43b414) adds the bullets' update 0x431240 at priority 0xe and 0x42c590 (0x43b441) the enemies' 0x42c660
at 0xb, both through 0x43c880, and the thread goes on to four calls of 0x447d56 before it clears its word.
**So a build slower than a frame is frames of the stage run on a build still going on**, and before anything
of a picked-up run is back in: three pick-ups of one file began their stage at `frames=5` with `script=5`,
`7` and `8`, and the two ahead read `the generator is 0xeede where the run had 0xe08d` on the playback's first
frame — 418 draws from the stage's seed where the run's own was 2 — as one of a clear's six stage starts read
`frames=4 script=6`.

**And the build waits on the supervisor.** A launch that walked nothing on those frames stood still for good
with the thread's stack at 0x43adf6 → 0x44c305 → 0x43c8ab → 0x44d6d5 → 0x465a8a: putting the player's job
up (0x44c230) runs its added callback on the thread, which asks the object at `[0x18bdc90]` for a file
through 0x465a40 — an entry out of 0x465860 with `+0x18` set to 1 — and sleeps a millisecond at a time
(0x465a82) until `+0x18` is nought again or `+0x28c`, the cancel, is set. What clears it is 0x466000, which
hands every pending entry of `0x100` to 0x465ba0 (cleared at 0x465c3a) and which the supervisor's update
calls at 0x445570, before it reads the loader's word. So what orb walks on those frames is the supervisor's
update alone — see `Game::walks_a_frame_of_its_build`.

**How a run is given up, and what it costs to get wrong.** Two sites of the run manager's own update
write `[0x17ce8b4] = 1` — 0x43a366 on the edge of the input word and 0x43a498 on a count reaching
`0x1b6c` — and **the second returns 3**. Both of those are the *demo's*: `[gm+0x3dbb4]` is which of the
four attract demos is running, written by the front end at 0x467adb, and `[gm+0x3dbb8]` its own frame
count. What the give-up actually is, is in *How a run of th08 ends* below. Three is not a scene: it is
the answer the chain runner switches on, and its jump table at
`0x43cb38` sends 3 to 0x43caf7, which **stops the walk where it stands** — see *The chain* above for all
seven of those answers.

So nothing after the manager runs on the frame a run is given up, and orb — writing the scene from a
recipe step, outside the walk — cannot answer for the manager. Doing only the write took the game down
on the next frame: `code 0xc0000005 at 0x00406a1a`, `reading 0x00000004`, the stack
`0x406a1a <- 0x404a67 <- 0x40234e <- 0x43caa1`. That last is the chain runner, and 0x402200 is the
update of the element the game adds at priority 1 on the text object `0x4cce20` — registered once at
startup and never taken off, so after the teardown it read what the teardown had freed. **What orb
writes instead is the two bits and the scene the run itself wants**, which is *How a run of th08 ends*.

So **scene 1 is the front end, scene 2 is a run and scene 3 is that run re-initialised** — and 0xa, 0xb
and 0xc are three more ways to reach the re-initialisation: each sets the wanted scene to 3 or leaves it
where it is, calls `0x43aa5c`, and then writes 2 back (0x445905, 0x445941, 0x44597d). `0x43bf6e` is
called first in every branch out of scene 2, which is that scene's own teardown.

## The game manager, and what a run is built into

`0x43aa5c` registers scene 2. It is short enough to read whole:

- the calc element is **`0x164d344`** — `[+0x4] = 0x439bc7` (the update), `[+0x8] = 0x43aaf4` (the added
  callback), `[+0xc] = 0x43be2c` (the deleted one), `[+0x1c] = 0x160f508` — added at priority **2**;
- the draw element is **`0x164d364`** — `[+0x4] = 0x43aa03`, `[+0x1c] = 0x160f508` — added at priority
  **5**.

So **the game manager is `0x160f508`**, and it is a static rather than an allocation: `[eax+0x3ddc0]`
is written at 0x43aa9e, which makes the object at least `0x3ddc4` bytes.

**The added callback is `0x43aaf4`.** It reads the scene wanted and sets `0x17ce8bc` to 0 where that is
3, 0xb or 0xc and to 1 otherwise — the flag that says a fresh run from a stage transition, th07's
0x42e894 branch in another spelling. Then it hands `0x43abd7` to 0x4486ba, and that is where the build
is.

**The build is `0x43abd7`.** In it:

- `timeGetTime()` into `0x17ce904`;
- `1 << [gm+0x3ddc4]` into the word `[gm+0x3ddd0]`, and `[gm+0x3ddc4]` copied into `[gm+0x3ddc8]`;
- `[gm+0x30]` against 4: `1 << [gm+0x30]` into `[gm+0x34]` where it is less, `0xf` where it is 4 or
  more. So **`+0x30` is the difficulty**, which `0x4458db`'s own `cmp DWORD PTR ds:0x160f538,0x4`
  spells absolutely — the two agree;
- a random out of `0x164d520` through `0x43ed20`, remainder of `0xffff`, plus `0x10`: the size of the
  allocation kept at `[gm+0x0]`, which is the record of inputs a replay is;
- `0x3c` bytes at `[gm+0x4]`, the run's header, filled by a `rep movs` of `0xf` dwords **from
  `0x17ce870`** — the configuration, which WinMain writes to `th08.cfg` at 0x441d43 and which the file
  on disk is 60 bytes of;
- `0xe4` bytes at `[gm+0x8]`, the run's own numbers, and so at **`0x160f510`** absolutely;
- `[header+0x1c] = 2` where the difficulty is 4 or more and `8` where `0x164d0b4 & 1`, otherwise
  whatever the configuration holds, which is 2 in the file beside this game. It is then converted with
  `fild` and stored as a **float** into `[run+0x74]` at 0x43ae42. That is the whole of what has been
  read about this game's lives.

### The run's numbers

**The panel is what named most of these**, and it is the strongest evidence in this file: the in-game
panel draws one number per row down the right of the screen at x=488, and a picture of a launch says
which row is which. The rows, off the `y` each draw is given: 40 the hi-score, 56 the score, 88 the
Player stars, 104 the Spell stars, 136 the power, 152 the graze, 168 the point items, 184 the time.

**And the replay's own text header named two more.** 0x453600 onward writes the lines a `.rpy` carries —
`最終ステージ\t%s` (0x4b70f4), `ミス回数\t%d` (0x4b70e4), `ボム回数\t%d` (0x4b70d4) and
`処理落ち率\t%f%%` (0x4b70c0) — and the two counts come out of the run through getters of their own.

**A one-line getter on `[[gm+0x8]+N]` is how this game reaches most of them**, which is why they can be
enumerated at all: nineteen such getters exist, and the ones below are the ones whose caller says what
the field is.

| field | what | evidence |
| --- | --- | --- |
| `+0xc` | **the graze** | drawn at (488, 152) with `%d`, and 0x44a9a4 caps it at `0xf423f` — 999,999, a count and not a frame |
| `+0x30`, `+0x38` | **the point items collected, and how many the next step of their value wants** | drawn at (488, 168) as one, a `/` (0x4b4cdc) and the other — the `0/100` of a fresh run, the 100 being what 0x405121 puts in `+0x38` |
| `+0x3c`, `+0x40` | **the time orbs picked up, and how many this stage's boss wants for her Last Spell** | drawn at (488, 184) through the getters 0x421b60 and 0x421b80, one `%d` and then the other with a `/` between them — the same shape as the point items above; and 0x41fb0b is what the second number is *for*, see below |
| `+0x64` | **the misses**, a float | 0x45363d reads it through 0x453c80 and prints `ミス回数\t%d` |
| `+0x74` | **the lives**, a float | the Player stars at (488, 88) come from 0x42f2b0, which reads it; and the build writes the run header's starting lives into it at 0x43ae42 with `fild`. Two readings, and the only field here with two |
| `+0x80` | **the bombs**, a float | the Spell stars, the row under the Player stars, from 0x4398ce |
| `+0x84` | **the bombs spent**, a float | 0x45365c reads it through 0x453ca0 and prints `ボム回数\t%d` |
| `+0x98` | **the power**, a float, full at 128 | 0x422480 is `float_to_int([[gm+0x8]+0x98])`, and (488, 136) draws what it answers with `%d` under 128 and `MAX` (0x4b4cd8) at 128 or more |
| `+0x22` | a word five getters read (0x406cf0 through 0x406da0) and 0x44e140 writes | not established |
| `+0x28` | a field three getters read | not established |
| `+0x8` | **the score** | 0x43bf94 compares it against `0x3b9aca00` and 0x43bfa2 clamps it to `0x3b9ac9ff`, which is 999,999,999 applied where a run is torn down |
| `+0x0`, `+0x29` | **the score the panel shows, and its last digit** | drawn at (488, 56) with `%.9d` and then the byte with `%1d` one glyph along, which is this game's ten-digit score split in two; 0x4048c9 copies `+0x8` into `+0x0` |
| `+0x14`, `+0x18` | the hi-score shown and its last digit | drawn at (488, 40) the way the score at 56 is drawn |
| `+0x4` | a counter capped at `0x1869f` at 0x44a981 | what it counts is not established |
| `+0x2c`, `+0x34` | two more of the point-item system | `+0x2c` against `0x7d0` at 0x4412c8, `+0x34` against 6 and 3 at 0x440481 and 0x4404ca |

**What a stage carried on does to the two scores**: the run manager's build, on the branch 0x43ac89 through
0x43acb8 sends a stage carried on down — no practice, no Extra — copies `+0x8` into `+0x0` (0x43b14a), and
every build zeroes `+0x8` (0x43b808). **And the run manager's update gives it back**: past its pause, it clamps
`+0x8` to 999,999,999 and, where the two differ and `+0x8` is the lower, writes `+0x0` into it (0x43a6c5
through 0x43a713). So a stage's numbers read at its build hold the run's score in `+0x0` alone, and from its
first update on in both.

### The Last Spell, and the count a run has to have reached for one

**0x41f420 is the ECL's own variable getter**, `__fastcall` with the enemy in `ecx` and the variable's
number in `edx`: it subtracts `0x2710`, bounds the remainder by `0x64` and jumps through the 101-entry
table at 0x41fb8b (0x41f44c). So the numbers a script can ask for are **10000 to 10100**, and the table is
the whole list of them.

**Variable 10098** — the table's entry 98, which is 0x41fb0b — is the one a boss's script asks whether to
run her Last Spell:

```text
mov ecx,0x160f508 ; call 0x421b60    ; [[gm+0x8]+0x3c], the time orbs picked up
mov ecx,0x4ea670  ; call 0x453cc0    ; [card+0x10] of the card that is up
mov ecx,0x1653648 ; call 0x4417e0    ; the live items whose class is 7
mov ecx,0x160f508 ; call 0x421b80    ; [[gm+0x8]+0x40], what this stage wants
cmp esi,eax ; setl dl ; dec edx ; and edx,0x2
```

so it answers **2** where those three summed have reached the fourth and **0** where they have not.
0x4417e0 is the count of the ones still in the air: it walks the item list's `+0x2dc` chain from
`[items+0x17b088]` and counts the items whose `+0x2d4` is **7**, which is the class a cancelled bullet
leaves — so a time orb drawn toward the player counts before it arrives.

### What stops a card's own update, which is what an unhittable player never lets go of

**The card's update is 0x418010**, the chain job registered at priority 0xc, and all it does is call
**0x416b90** on the card. That function's first act is two gates, and where either holds it answers **1**
— carry on, do nothing:

```text
mov eax,ds:0x164d0b4 ; shr eax,0xa ; and eax,1 ; test eax,eax ; jne <answer 1>
movsx ecx,BYTE PTR ds:0x160f534 ; test ecx,ecx ; je <the card's own work>
```

So **bit 10 of the run's flags word** or **the byte at `0x160f534`** — `[gm+0x2c]` — freezes a card where
it stands. **0x425070 is what writes that byte**, `__fastcall` with an object in `edx` whose `+0x10` it
takes, and **the only writer that clears it is 0x44ab57, inside 0x44ab40** — the function that takes the
life, two instructions past the string it logs.

**Which is why a launch under `--clear` stopped dead on the last stage's Last Spell.** orb's
`make_invulnerable` writes `PLAYER_STATE` and the hit countdown itself and never lets 0x44ab40 run, so a
card that begins with that byte set never advances: `script=4264` frozen, `boss_life=20000`, `bullets=0`,
held from the run's frame 66,138 to 108,378 with nothing changing.

**And the card pays that number out as it ends.** 0x415599 through 0x4155b9, at the tail of a card
ending: where `[0x4ea670+0x10]` is over nought it is handed to **0x418220** — `__thiscall` on the game
manager, which adds its argument to `[run+0x3c]`, `[run+0x44]` and `[gm+0x3daac]`, declining a negative
one that would take the first below nought — and then zeroed. `[0x4ea670+0x104]` goes to **0x4181f0** a
few instructions in front of it, which adds a tenth of what it is given to `[run+0x8]`, the score. The
four other callers of 0x418220 are **1** for a collected time orb (0x441373), **-15** for a bomb
(0x44cc10), and the death path at 0x44cd46, which hands it `-500` where the count is over `0x1388` and
`-count/10` where it is not. **What fills `+0x10` while the card runs is not established.**

**And what a card pays is how much of its own time was left.** `[0x4ea670+0x10]` starts at nought —
0x415227, in the card's own start — and 0x416295 through 0x41633d sets it as the card ends, out of two
counters of the card read through 0x40d3b0: `[card+0x114]`, whose `count - count/7` is the bar, and
`[card+0x108]`, the time left. **Over that bar it is `0x3e8`**, a thousand; under it and over `0xb4` —
a hundred and eighty — it scales, `(left - 0xb4) * 0x384` over `(bar - 0xb4)`, so up to nine hundred; and
where bit 27 of the enemy's own flags is set instead it is a flat **`0x2bc`**, seven hundred (0x4162a2).

**So the norm is met by taking cards quickly**, a thousand orbs apiece at best against a stage's 2,500 to
9,999 — which is why a run of the rules, whose damage goes in slowly, has read 6,900 of 8,500 at its best
and no Last Spell has ever been due.

**And the same comparison picks a colour**, which is the second spelling of it: 0x441328 does
`setge dl; dec edx; and edx,0x107f; add edx,0xdfffef80` over `+0x3c` against `+0x40` and hands the answer
to the score popup 0x403330 that a collected time orb draws (0x4412b0).

The strategy wiki says the rule from the other side:
`ボスを倒したとき、それまでに拾った刻符アイテムがノルマ以上あれば、敵がラストスペルを発動する（難易度Normal以上）`.

## The player

**`0x17d5ef8`**, a static and not an allocation. 0x44c230 zeroes `0x38acc` dwords of it — `0xe2b30`
bytes — and registers its calc element at `[player+0xe2b10]`: update 0x44c390, added 0x44d650, deleted
0x44dc60, priority 9. The name is the game's own: 0x43adf1 calls 0x44c230 and, where it answers,
prints `error : プレイヤーの初期化に失敗しました` (0x4b5930).

**Where it is: three floats at `[player+0x2b4]`.** The added callback is what says so, because a stage
puts the player at its own start there:

- 0x44d745 loads `0x164d2e4` — `[gm+0x3dddc]`, the play area's width, 384 — divides by the 2 at
  `0x4b42ec` and 0x44d765 writes the 192 into the first float;
- 0x44d767 loads `0x164d2e8` — `[gm+0x3dde0]`, the height, 448 — subtracts the 64 at `0x4b42c8` and
  0x44d787 writes the 384 into the second;
- 0x44d798 writes `0x3efae148`, which is 0.49, into the third.

The accessor in between, 0x40b460, answers its own `this`, so `[player+0x2b4]` is the vector itself.
**A position of this game is therefore measured from the play area's own corner**, 0 to 384 across and
0 to 448 down, and not from the screen's.

**`[player+0x0]` is a byte its update switches on, and all four of its values are read.** 0x44c47f
compares it against 2 and 0x44c49a against 1, and both run 0x44d180; 0x44cbf0 runs for the 2 alone.
The hit handler 0x44ab40 writes **2** at 0x44ab84 and puts nothing back into the countdown at
`player+0xe2af4`; 0x44d180 ramps the player's colour over `0x1e` frames of that countdown (0x44d210)
and writes **3** the frame it reaches `0x1e` (0x44d24a, 0x44d252). The three box tests decline while
the byte is not nothing — 0x44a333, 0x44a445 and 0x44a8f2 each answer 1 and take no life — and
0x44a51f declines for 2 and 1 by name.

So **0** is a player that can be hit, **2** is the thirty-frame window a deathbomb is spent in, **1**
is the player coming in at the start, and **3** is the player untouchable and blinking (0x44d3cf). Over
one launch of stage 1 the byte read 0 on 713 of the frames orb printed, 3 on 528, 1 on 152 and 2 on 62.

**The timeline, off the update that steps them**:

- 0x44d2c0, called every player update at 0x44c4ad after 0x44d180, adds one to the count in states 0, 1
  and 2 (0x44d409, through 0x406640) and takes one off in state 3 (0x44d35c, through 0x418110) — one a
  update at the ordinary frame step, `[0x17ce8e0]` of 1.0 (0x425369).
- **A stage's build leaves state 1 with 120 under it**: 0x44c230 on both of the build's paths — 0x43adf1
  for a run starting, 0x43b181 for a stage carried on — and the added callback at once (0x43c8a8), all of
  it on the build's thread; 10 in a spell practice (0x406c70). **And state 1 stands for as long as the
  build is held**: while `[gm+0x38]` is set the game manager's update returns 3 (0x439fe2) and the walk
  ends before the player's priority 9.
- **State 1 at a count of thirty or more is state 3 with `0xf0`** (0x44d180: 0x44d24a, 0x44d252,
  0x44d29f — the `0xf0` skipped in a spell practice, 0x44d28f), so a stage's first player update is
  already state 3; the update it goes in on takes it to 239, and 0x44d39f writes 0 on the 240th with the
  count back to nothing (0x44d3ad).
- **A hit is state 2 with the count from nothing** (0x44ab84); at thirty 0x44cbf0 writes 1, puts the
  player at (192, 384) (0x44d035, 0x44d057), sets the count to nothing (0x44d07a) and takes the life
  (0x44d100) — so 31 updates of the hit, 30 of coming in and 239 untouchable.
- **States 1 and 2 skip the player's input**, 0x44aec0 (0x44c4b5 through 0x44c4cb): no move, no focus, no
  shot, no hit box refreshed. There is no walk-in: the position is written at the build (0x44d765,
  0x44d787) and at the respawn and nowhere else.

Four other writers put a player in state 3 from 0: 70 at 0x416a3d, 0x42b709 and 0x42bae4, and 90 at
0x42d7c3.

**The countdown itself is three dwords**, read off the setter at 0x406610 that the hit handler reaches
through 0x4065f0 with a 0: `+0x8` takes the count — which 0x40d3b0 answers and 0x44d245 compares against
`0x1e` — `+0x4` takes nothing, and `+0x0` takes `-0x3e7`.

**Which is the whole of what `Game::make_invulnerable` is for this game.** There is no window beside the
hit's own: every one of the four tests above keys on the state byte alone, so what makes the player
unhittable is the state a hit leaves — and that state ends in a death rather than in the game taking it
back, so the count has to go back to nothing with it. An update that writes both never reaches the frame
the life would go on.

`0x18b896c` and `0x18b8970` are the two characters a team is, cleared at the player's teardown
(0x44dcd0, 0x44dcf4); the player's own code reads `[[0x18b896c]+0x4]`, `+0xc` and `+0x10` for the
active one's numbers. `0x44a930` is the graze — it steps both `[run+0x4]` and `[run+0xc]` by the same
amount under their own caps.

## The four tests a hazard is put to, and which of them takes the life

**Three of them take a life and one does not**, and telling them apart is what says which shapes a bot
has to be told about.

| test | shape | called by | what it does |
| --- | --- | --- | --- |
| 0x44a230 | `centre ± size / 2` | the bullets, 0x4316b1 | 0x449ff0 first, then the box; **the life** |
| 0x44a360 | the same, no 0x449ff0 | the enemies, 0x42c362 | **the life** |
| 0x44a470 | the same, grown by `20.0` | the bullets 0x4315f2, the enemies 0x42c309 | the graze |
| 0x44a6a0 | a box turned by an angle | the lasers 0x431ea1/0x431f3b/0x43208a, enemies 0x4247ec and five more | **the life** |

**What taking the life is, in all three**: bit 1 into `[[0x18b8a28]+0xda]`, and then — where
`[player+0x0]` is nothing — the game manager's 0x44e160 and the player's own 0x44ab40, which is where
the state byte becomes 2. 0x44a34e, 0x44a460 and 0x44a91b are the three sites, and there are no others.

**0x44a470 is the one that does not**, which the `20.0` at `0x4b6e98` added to every side of the box
already says, and its callers gate it further: 0x42c2e8 asks the enemy's own timer for a multiple of
six, so an enemy body is put to it one frame in six.

**0x449ff0 is not a test of the player at all.** It walks `0xc0` entries of `0x40` bytes from
`player+0xbb834`, reading `[+0x3c]` as live, `[+0x0]` and `[+0x4]` as a place and `[+0x8]` as a radius
against the double at `0x4b4c90`, and 0x44a230 answers **2** where a hazard is inside one. 0x431615
takes that 2 and writes kind `5` into the bullet, so what those 192 are is something of the player's
that *erases* a bullet — and a bullet inside one never reaches the player's box at all.

## The lasers

**This game keeps them in the bullet manager**, which is why looking for a manager of their own on the
chain found nothing: the eleven elements the chain runs are the ones listed below and none of them is a
laser's.

**The array, off the second walk in the same update (0x431240):** 0x431b72 puts the cursor at
`bullets + 0x660938`, 0x431ba2 steps it by `0x59c` and 0x431bab bounds it at `0x100`. `0x100 * 0x59c`
is `0x59c00` and `0x660938 + 0x59c00` is `0x6ba538`, which is where the bullets' own live count sits —
so the lasers fill the manager from the end of the bullet array to that count and nothing else is in
there.

`[laser+0x584]` is what the walk reads first (0x431bbb): nothing means the slot is free.

| field | what it is |
| --- | --- |
| `+0x548` | where it is fired from, three floats |
| `+0x554` | which way it points |
| `+0x558` | how far along it the near end is |
| `+0x55c` | and the far end |
| `+0x560` | what `far - near` is capped at (0x431bfc, 0x431c1b) |
| `+0x564` | how wide it is |
| `+0x568` | how wide it is drawn this frame |
| `+0x56c` | how much further `far` reaches each frame |
| `+0x570` | how long the first phase lasts |
| `+0x574` | which frame of the first phase it starts hitting on |
| `+0x578` | how long the second lasts |
| `+0x584` | whether the slot holds one |
| `+0x588` | a timer, whose whole part is `+0x590` |
| `+0x594` | a word of flags, of which bit 0 is read |
| `+0x598` | which of its three phases it is in |

**The box the walk hands 0x44a6a0** is centred at `origin.x + near + (far - near) / 2` (0x431ca1
through 0x431ccb) and `origin.y` (0x431cd1), and is `far - near` by `width / 2` — which 0x44a6a0 halves
again, so a quarter of the width is what it reaches on each side. **The laser runs along `x` from its
origin** and the angle is what the test turns the player's position out of.

**The phases, off the switch at 0x431d48.** 0 grows the width from nothing to `+0x564` over `[+0x570]`
frames (0x431e18 through 0x431e41) and is the only one whose test is gated — 0x431e75 asks the timer
whether it has reached `[+0x574]`. 1 is the laser firing and 2 is it going out; both test the player
every frame. `+0x56c` is multiplied by `0x17ce8e0` before it is added (0x431bcf), which is how much of
a frame this frame is worth and is 1.0 where nothing slows the world down (0x425369).

## The items

**`0x1653648`**, and the manager the chain runs at priority 0xd is *not* it — which is what made this
one hard to find. 0x4b3878 builds it and 0x440010 lays its pool out: `0x831` slots of `0x2e4` bytes from
the object's own start, `0x2e4 * 0x831` being exactly `0x17adac`, where the pool's bookkeeping begins.

**The live ones are a list rather than an array**: `[items+0x17b088]` is the first (0x440516),
`[item+0x2dc]` the next (0x44099c) and nothing the end (0x440584). `[items+0x17ada8]` is how many, which
the walk counts fresh (0x440543, 0x44059d); `[items+0x17ada4]` is the round-robin index 0x4400a0 spawns
into. A free slot of a pool is not marked, so the list is the only honest walk — and orb bounds its own
at `0x831`, a list read wrong being a frame that never ends otherwise.

- `[item+0x2a4]` is where it is, three floats; `+0x2a8` is the one the fall is read off (0x440978,
  against the field's own height) and the one the auto-collect line is compared with (0x440e5e)
- `[item+0x2b4]` is how fast it falls (0x4409d4)
- `[item+0x2d4]` is what picking it up does
- `[item+0x2d7]` of 3 is one the player's own test is skipped on (0x4409f5)

**The pickup is the player's own test at 0x44a5a0** (0x440a17), and what it does is the switch at
0x440a63 over `[item+0x2d4]`, through the jump table at `0x440cc5`:

| class | handler | what it does |
| --- | --- | --- |
| 0 | 0x440cf0 | power, by the thresholds at `0x4c7a6c` |
| 1 | 0x440e40 | a point item, worth `[run+0x24]` above the line at `[[0x18b896c]+0x1c]` |
| 2 | 0x441170 | power, eight of it (0x4411bd) |
| 3 | 0x440a9e | a bomb, one more under a cap of eight |
| 4 | 0x440aeb | power to the 128 it stops at |
| 5 | 0x440adc → 0x439b29 | a life, one more under a cap of eight, read off `[run+0x74]` |
| 6 | 0x440b96 | score off the graze, `graze / 0x28 * 0xa + 0x12c` |
| 7 | 0x440bf9 → 0x4412b0 | what a cancelled bullet leaves, three at a time (0x43164b) |
| 8 | 0x441020 | a point item, which is what 0 and 2 become at full power (0x44010e) |

**0x4400a0`(&position, class, count)`** is the spawn, and it rewrites its own arguments: class 0 or 2
becomes 8 at full power (0x44010e), class 7 takes a count of 3 (0x44011b) and class 0xa becomes class 7
with a count of 5 (0x44012a).

## The effects, which are not the items

**`0x4ece60`**, one static object of about 570 kilobytes, which the chain runs at priority 0xd: 0x42866a
registers the calc element at `0x577ebc`, update 0x427bf0. **Nothing here is picked up** — its update
calls no test of the player — so orb reads it for nothing and it is written down only because its shape
is so nearly the item manager's: a position at `+0x2a4`, a kind byte, a live byte.

**The array is at `this + 0x1c`, stride `0x360`, and the update walks `0x28d` of them** (0x427caf
through 0x427cc3). `0x1c + 0x360 * 0x28d` is `0x89f5c`, which is where the five further slots the
update hangs off `[this+0x8b03c]` and after begin — so the count checks out against the layout.

- `[item+0x350]` is whether the slot holds one, written 1 by the drop at 0x4254ed
- `[item+0x351]` is which kind it is, the drop's own first argument
- `[item+0x2a4]` is where it is, three floats copied from the drop's second (0x425500)
- `[item+0x348]` comes from `[0x4c6d34 + kind * 0xc]`
- `[this+0x8]` is how many are live, counted fresh each frame (0x427c05, 0x427d10)

**The drop is 0x425430`(kind, &position, ?, colour)`**, a `__thiscall` on the manager. It rotates
`[this+0x0]` over `0x200` slots looking for a free one, clears `0xd8` dwords of it — `0x360`, the whole
slot — and fills the four fields above. 0x44ab77 calls it with kind `6` at the player's own position as
a life is taken, which is the power a death scatters.

**What the kinds are worth is not read.** The table at `0x4c6d30` is `0xc` bytes a row — a sprite id,
and two function pointers — and rows 0 through at least 0x2f are used: the kinds pushed at the 93 call
sites include 1, 3, 5, 6, 0xc and 0x2f. None of those functions has been read, so orb hands no item
over.

`0x42841f` pushes the player's own position into the vector subtract against an item's, which is the
attract: an item within `[0x4b49bc]` of the player is pulled towards it by `[item+0x238]`.

## The bullets

**`0xf54e90`**, one static object of about seven megabytes. 0x4311a0 registers it: the calc element at
`0x160f408`, update 0x431240, added 0x433070, deleted 0x433820, priority 0xe, the object hung off
`[elem+0x1c]`.

**The array, off the walk at 0x43129e: slot `i` is at `bullets + 0x1a880 + i * 0x10b8`, and there are
`0x600` of them.** The walk runs backwards, which is what makes the stride readable — 0x431254 starts
the cursor at `this + 0x1a880`, 0x431b64 takes `0x10b8` off it each slot, and 0x431b58 adds `0x645000`
back when the index wraps. `0x600 * 0x10b8` is exactly `0x645000`, so the three numbers check each
other.

`[bullet+0xdb8]` is the word the walk reads first: nothing means the slot is free, and 0x4312ef takes
one off it and compares against 4, so a live bullet is one of five kinds.

**`[bullets+0x6ba538]` is how many are live**, counted fresh each frame: 0x431283 puts nothing there
before the walk and 0x4312d0 adds one per slot that holds a bullet. It sits past the array's end,
`0x1a880 + 0x645000` being `0x65f880`.

**Where a bullet is and how big it is: `+0xd44` and `+0xd34`**, the pair the walk hands the player's own
tests — 0x4315d9 and 0x4315e3 push `this + 0xd34` and `this + 0xd44` into 0x44a470, and 0x431699 and
0x4316a3 push the same two into 0x44a230. A whole width and height, for the reason the enemy's is.
`[bullet+0xdbd]` is set to 1 where that test answered 1, which is a bullet already grazed.

### What moves one, and what does not

**`0x410a70(position, step)` adds three floats to three**, and that is the one place a bullet's position
takes its step: kind 1 calls it with `+0xd50` whole (0x4314a4, 0x4314ae), gated on the byte at
`0x160f534` being nothing, and the other four divide the step first — 0x43178a, 0x43189c, 0x4319ad and
0x431aae by 2.0, 2.5, 3.0 and 2.0.

**And kinds 2 to 4 are a bullet's spawn, whose last frame ends in kind 1's path — which is the half
again.** The switch at 0x4312ff takes one off `[bullet+0xdb8]` and jumps through the table at
`0x432156`: kind 1 to 0x431322, kinds 2 to 5 to 0x43176e, 0x431880, 0x431991 and 0x431aa2. The branch
for kind 2 adds `+0xd50 / 2` (0x43179f), tests the player (0x449ff0, a 2 being a hit), and asks
0x45ea00 on `0x18bdc90` with the sprite at `+0x2a4` whether its animation has ended (0x4317f3): while
it has not, `jmp 0x431aeb` to the walk's tail and the kind stays; the frame it has, `[bullet+0xdbe]`
being nothing, `jmp 0x431306` (0x43187b) — and 0x431306 is `mov word [ecx+0xdb8],1`, three
instructions before kind 1's own entry, so that frame goes on down kind 1's path to the whole add at
0x4314ae as well. Kinds 3 and 4 are the same shape at 0x43198c and 0x431a9d. So a spawning bullet moves
its divided step a frame and `+0xd50 / 2 + +0xd50` on the frame its spawn ends, and every frame after
is kind 1's: that is the `1.10,-2.07` measured against `0.73,-1.38` claimed, and the three bullets a
frame that had `moved 2.99 saying 2.00` in fifty dumps of one hit — the boss's fresh bullets, each on
the frame its spawn ended. `+0xdbe` set on that frame writes 5 over the kind instead (0x431812), and
kind 5's own branch never joins kind 1's: after its half it asks the same 0x45ea00 (0x431ad8) and goes
to 0x432170 or to the tail, so a kind-5 bullet moves half a step a frame and stays 5.
`Th08::how_much_of_its_step_a_bullet_takes` hands over the ending frame's multiple and `sight.rs` takes
the whole step back off it on a frame the spawn cannot end, which is how the two are told apart. Both
readings were measured by getting them wrong: the share alone read three bullets a frame `moved 2.99
saying 2.00` on the frame their spawn ended, and the ending frame's multiple on every frame read
`22 bullet(s) no step of their own explained ... moved -1.00,0.01 saying -3.02,0.03, in the game's own
state 2` in every dump of stage 4's boss — 152 bullets three times too fast.

**What the ending frame is asked by**: 0x4317f3 calls
`0x45ea00([0x18bdc90], bullet+0x2a4)`, so the sprite is `bullet+0x2a4`; 0x45ea10 answers ended outright
where `[sprite+0x220]` — the animation's own instruction — is nothing, 0x45ea2c answers not ended on bit
`0x13` of `[sprite+0x1f8]`, and where the word at `[sprite+0x1fe]` is nothing 0x45ea69 reads the elapsed
count out of `[sprite+0x40]` (0x40d3b0 being `return this->+0x8` on `sprite+0x38`) and 0x45ea70 compares
it against the wait at `[[sprite+0x220]+0x4]`: a wait still ahead of the count is an animation that
carries on. **So a wait still ahead is a frame the spawn cannot end on**, which is what `sight.rs` asks
and the whole of what one comparison can say: where the wait has run out, that function walks on to the
next instruction and can end there or set another, so those frames go over as frames the spawn may end.

**`0x42ffc0` is the bullet's own script and it moves nothing.** `[bullet+0xdcc]` indexes the `0x18`-byte
entries at `+0xdd0`, eighteen of them (0x42ffd7), each entry's `+0x10` a bitmask the interpreter
dispatches on and `+0x14` a flag; the two branches that touch `+0xd44` **read** it — 0x43061e to play a
sound there through the sound manager, 0x43065c to spawn something there — and neither writes it.

**`[bullet+0xdac]` is a bitmask of nine behaviours**, each its own function, and none of them adds to the
position either: 0x432210 re-aims the step through 0x4286e0, 0x4329f0 wraps the x round the field's own
width, and 0x432830 works on the step. **And bit `0x10` is 0x4322b0, which adds to the step** — read
looking for the half again, which turned out to be the kind's own frame above, and kept for what it is,
the one behaviour that grows the step: while the timer at `+0xfac` has run for fewer frames than
`+0xfcc` asks (0x40b8e0 compares its `+0x8`, so `+0xfb4`, against the argument), the three floats at
`+0xfc0` scaled by the frame step (0x409120, through 0x404720) go into `+0xd50` through 0x410a70 at
0x432312, and the bit is cleared at 0x4322de once the timer has run out. The walk runs the behaviours
(0x43132d on) before it switches on the kind, so the step a frame adds to the position is the one this
has already grown — and the step read after a frame is one growth short of the one about to be taken,
which `sight.rs` adds back. 0x432321 then measures `+0xd50` against `+0xd54` through 0x4031e0 and
`[0x4b456c]` and writes their angle into `+0xd74`; nothing there clamps the step.

**And `0x4251b0` is the world slowed down**, which is what the frame step at `0x17ce8e0` is for: it
writes `1.0 / [arg+0x10]` there — the 1.0 at `0x4b4338`, so the step is the reciprocal of a whole
number — and then walks all `0x600` bullet slots and **scales each live one's step at `+0xd50` by it**
(0x425224 through 0x425234). 0x425369 writes the 1.0 back. So a bullet's step is a per-*update*
quantity and the frame step says how many updates a frame is worth — and the walk over `0x600` slots
here is the third number to agree about how many there are.

**A bullet's own state beyond the kind is not read.**

**Held against the game on the real thing**: a launch with both walks in read
`enemies=14/14 bullets=44/44` and then `16/16` — the game's own count first and orb's walk second — and
the nearest hazard to the player came out as `(198.74,367.35)+(4.00x4.00)`, `(198.40,368.21)`,
`(198.05,369.07)`, `(197.70,369.92)` over four frames, which is a 4-by-4 bullet moving smoothly inside a
play area of 384 by 448.

## The player's own hit box

**`[player+0x3d4]` is the half**, and the box the game's own tests read is made out of it every frame:
0x44bc02 calls the vector subtract on `player+0x2b4` with `player+0x3d4` and copies the answer into
`+0x38c`, and 0x44bc37 calls the add with the same two and copies that into `+0x398`. The test at
0x44a2c7 through 0x44a306 then compares a hazard's own box against those four floats — `[+0x38c] <=
right`, `[+0x390] <= bottom`, `[+0x398] >= left`, `[+0x39c] >= top`, an overlap and nothing else. So
`+0x38c` is the corner nearest the origin, `+0x398` the far one, and `+0x3d4` is what a position is
moved by to get each.

**And the two tests a bullet's walk hands its position and size to are the hit and the graze.**
0x44a230 is the hit: the bullet's box is its position less and plus half its size (`/ 2.0`, the float
at `0x4b42ec`) and nothing more, against `+0x38c` and `+0x398`. 0x44a470 is the graze: the same box
**widened by `20.0` on every side** (the float at `0x4b6e98`, 0x44a4c4 through 0x44a513) against a
second box at `+0x3a4` and `+0x3b0`, with `[0x160f534]` being 1 or 2 — the player's own hit states —
answering nothing first. So a bullet's hit box is what `sight.rs` hands over, its size halved, and the
twenty is the graze's alone.

**The value is in a file and not in the exe.** 0x44d7e3 writes `[spec+0xc] / 2` into `[player+0x3d8]`
and 0x44d7f5 copies that into `+0x3d4`, with `+0x3dc` taking a flat 5.0; `+0x3e0` is a second half taken
the same way from `[spec+0x10] / 2`. `spec` is `[0x18b896c]`, a `.sht` out of `th08.dat` — the table of
names is `0x4c7ce0`, `ply00a.sht` through `ply03as.sht`, indexed by `0x164d0b1`.

**The table is sixteen pointers and eight files**, which is what a slot has to be named around:

| index | file | | index | file |
| --- | --- | --- | --- | --- |
| 0 | `ply00a` | | 8 | `ply02a` |
| 1 | `ply01a` | | 9 | `ply02as` |
| 2 | `ply02a` | | 10 | `ply03a` |
| 3 | `ply03a` | | 11 | `ply03as` |
| 4 | `ply00a` | | 12 | `ply00as` |
| 5 | `ply00as` | | 13 | `ply01as` |
| 6 | `ply01a` | | 14 | `ply02as` |
| 7 | `ply01as` | | 15 | `ply03as` |

So four of the eight are pointed at twice — 0 and 4 both give `ply00a`, 5 and 12 both `ply00as` — and
**a run named by the file alone would be a file two runs share**, which is why `Th08::run_slot` puts the
index in the name beside the stem. Which team member each index is is not read: nothing in this exe ties
one to a person's name.

## The player's four speeds, measured

**Not in the exe either**, for the same reason, so they were measured instead: a recipe held each
direction for twenty frames and orb's own line printed the player's position every frame. Differencing
consecutive frames over the whole launch gives four steps and no others:

| held | step per frame |
| --- | --- |
| one direction | **4.00** |
| two directions | **2.83** on each axis |
| focused, one direction | **2.00** |
| focused, two directions | **1.41** on each axis |

40 frames apiece for the straight ones and 28 to 29 for the diagonals, out of 1,363 frames. So focused
is half of unfocused and a diagonal is the straight speed over √2 — the total speed is the same in
every direction — and the one number orb needs is the straight unfocused speed, of that team, out of
the `.sht`. Which field of the `.sht` it is has not been settled; `Th08::the_teams_spec_said` prints
the first sixteen floats of it so that the four above can be matched against them.

## What the missing piece was

**There is one array of `0x40`-byte entries at `player + 0xb8834`, and two walks over it.** 0x44c5b0 —
the first call the player's update makes — walks `0x180` of them from the array's start, counting
`[+0x24]` down and stepping `[+0x8] += [+0xc]`, which is something that moves on its own and expires.
0x449ff0 — which 0x44a230 calls before it does any box arithmetic, and answers 2 on — walks `0xc0` of
them from `+0xbb834`, which is `0xb8834 + 0xc0 * 0x40`: the array's upper half. There it reads `[+0x3c]`
as a live byte, `[+0x0]` and `[+0x4]` as a position and `[+0x8]` as a radius, against the double at
`0x4b4c90`.

So a hazard that touches one of the upper 192 has hit *something of the player's* before the player's own
box is ever considered. **What those 192 are, and which entry is the player's own body, is not settled**,
so `Game::what_a_bot_sees` has no `half` to give and answers nothing.

The four speeds are not read either. The candidates are in the active team member's spec, `[0x18b896c]`:
`+0x4` (0x43ae4f reads it as a float and converts it to an int), and `+0xc` and `+0x10`, which 0x44d7da
and 0x44d811 halve by the 2 at `0x4b42ec` into `[player+0x3d8]` and `[player+0x3e4]`.

## Everything the chain runs, by priority

Read off the calls to `0x43c880` — the add of a calc element — with the element each pushes, and then
off each element's own register, which is where the object and the update are written. The priorities
are the order a frame's calc walk takes.

| prio | object | update | what it is |
| --- | --- | --- | --- |
| 0 | `0x17ce758` | 0x445453 | the supervisor, whose update decides the scene |
| 1 | `0x4cce20` | 0x402200 | the text the panel and the menus are drawn with |
| 2 | `0x160f508` | 0x439bc7 | the game manager, which is a run |
| 4 | `0x18bde08` | 0x467399 | the front end |
| 8 | `0x4ea650` | — | registered at 0x409bb8, beside the `.std` loader at 0x4098f0 |
| 9 | `0x17d5ef8` | 0x44c390 | the player |
| 0xb | `0x577f20` | 0x42c660 | the enemies |
| 0xc | `0x4ea670` | 0x418010 | **the spell card** — see *The spell card that is up* |
| 0xd | `0x4ece60` | 0x427bf0 | registered at 0x42866a; sets up five lists `0x360` apart at `+0x89f5c` and after, and counts into `+0x8` |
| 0xe | `0xf54e90` | 0x431240 | the bullets |
| 0xf | an allocation of `0x230b8` bytes at `0x160f500` | 0x4338ca | registered at 0x437b7f |

**0xd is the one that is not named.** It sets up five lists `0x360` apart at `+0x89f5c` and after and
counts into `+0x8`, which is the shape of an item or effect manager.

**And the front end's own pointer is written once in the whole exe and never cleared**: 0x4714b9, inside
the register at 0x47146d, is the only store `objdump` finds to `0x18bde08`, and the allocation is
`0x14490` bytes. A run's build frees the object and that static goes on naming it, so **every read of
`[0x18bde08]` during a run is a read of freed memory** — measured, a launch of a Normal run read
`front end: screen 1900876895, cursor on item 0` on every frame of stage 1. `Th08::front_end` is
therefore gated on the scene being 1 and not on the pointer, there being nothing about the pointer to
test.

## The enemies

**`0x577f20`**, one static object of about ten megabytes. 0x42c5e3 registers it: the calc element at
`0xf54e30`, update 0x42c660, added 0x42ebf0, deleted 0x42ee80, priority 0xb, the object hung off
`[elem+0x1c]` at `0xf54e4c`.

**The array, off the walk at 0x42c854: slot `i` is at `enemies + 0x53d0 * (i + 1)`, and there are
`0x1e0` of them.** The cursor starts at `this + 0x53d0` (0x42c83b), steps by `0x53d0` each slot
(0x42c869), and the bound is `0x1e0` (0x42c872) — so the array runs to `+0x9d79d0`, which is what puts
the count at `+0x9dcdc4` and the sixteen-byte-per-slot side table at `+0x9dcdd0` past its end rather
than inside it.

`[enemy+0x3324] & 1` is what the walk reads first (0x42c882): a slot whose bit 0 is clear holds nothing.

**Where an enemy is and how big it is: `+0x2d88` and `+0x2d70`**, read off the hand-over rather than off
a draw — 0x42cf6d pushes `this + 0x2d70` and 0x42cf71 pushes `this + 0x2d88` into 0x42c290, and
0x42c309 hands the same two to the player's own test at 0x44a470. 0x424730 reads `+0x2d88` and `+0x2d8c`
of the same shape, so the position is at least two floats.

**The size is a whole width and height and not a half.** 0x44a230 builds its box as
`position ± size / 2`, dividing by the 2 at `0x4b42ec` four times (0x44a274 through 0x44a2bb).

**`[enemies+0x9dcdc4]` is how many are live**, counted fresh each frame: 0x42c84a puts nothing there
before the walk.

**The flags at `[enemy+0x3324]`**, of which five bits are read:

| bit | what it says | read at |
| --- | --- | --- |
| 0 | the slot holds an enemy | 0x42c882 |
| 1 | **this enemy is the boss** | 0x41c9b5, 0x42c375 |
| 2 | it has a body to run into | 0x42cf56 |
| 7 | it is grazed rather than touched | 0x42c2cc, 0x42c38a |
| 31 | **its boxes are passed over** while `[0x17d6ed4]` is not nothing | 0x42cf3e, 0x42cf47 |

**Bit 1 is the boss's, off the two ECL instructions that write it.** 0x41c4f0 tells the run screen a
boss is up — `0x160f428.0x422c20(1)`, which is its `+0x2c` byte — and then sets the bit (0x41c51d);
0x41c5a0 tells it none is and clears the bit (0x41c5ca). Two other places agree: 0x42c375 stops the
player's own body taking life off an enemy that has it, and 0x41c9b5 gates the life bar's own update on
it. So the fight is with the live enemy that has bit 1, and `+0x2c` of the run screen says whether there
is one.

**`[enemy+0x2dfc]` is its life and `+0x2e00` the maximum**, written together by the life-bar instruction
at 0x41c989 and 0x41c97a, and 0x42c391 takes `0xa` off the first where the player's own body touched it.
0x41ca91 divides by the maximum to get the bar, and 0x4230e0 writes the eight segments of it into
`0x160f428 + 0x3c` and `+ 0x5c`.

**`[enemy+0x2e14]` is a timer of its own, and `+0x2e1c` the count inside it**, which is how many frames
that enemy has been alive. 0x42c2d6 hands `enemy + 0x2e14` to 0x40d3d0, whose whole body is
`[this+0x8] != [this+0x0]` — the count against the count of the frame before — and 0x42c2ed hands the
same object to 0x40d410, `[this+0x8] % n`, with the `6` the wide first box test is gated on. 0x40d3b0 is
that field's getter, `return [this+0x8]`, and 0x40d3f0 is `[this+0x8] == arg`. So the count sits at `+0x8`
of the timer, which is `+0x2e1c` of the enemy, and the game reads it as an integer: 0x41f635 reads it
straight and 0x42035a `fild`s it.

**Which is the only thing that tells the enemy in a slot from the next enemy the slot is given.** A slot
freed and filled the frame after holds an enemy whose position has nothing to do with the last one's, and
nothing else on an enemy differs between two cells of one grid — `蘇活「生命遊戯」` bears and kills them
by the dozen every few frames. See `Hazard::frames`, and the five hits one Normal run took to a box
`2408.0,4164.9` that was the read `8.0` grown over a distance between two cells.

`0x18b89b4` is **not** the boss, which is worth writing down because it looks like it: 0x42d4d9 writes
whichever enemy of the walk is nearest by the distance 0x4031e0 works out, having declined any whose
`[+0x2da4]` is set, and 0x42c89a clears it as that enemy's own bit 0 goes — a shot's target chosen
afresh each frame. `0x18b899c` through `0x18b89a4` hold that enemy's position, copied at 0x42d40e.

### The beam an enemy's own script fires, which is the third way a run dies

`0x44ab40` takes the life, and three sites call it: 0x44a34e inside the bullet box test 0x44a230,
0x44a460 inside the enemy box test 0x44a360, and **0x44a91b inside 0x44a6a0** — the rotated-box test the
laser walk hands its lasers to (0x431ea1, 0x431f3b, 0x43208a). Three more functions hand it an
**enemy**: `0x424730`, `0x424820` and `0x424910`, `__fastcall` on the enemy, six calls between them
(0x4247ec, 0x42480b and the same pair in each). Each lays a box `590` long (`0x44138000`) from
`[enemy+0x2d88..0x2d8c] - [[enemy+0x2ca0]+0x38..0x3c]` along the angle at `[enemy+0x14]`, and tests
the player against it every frame at one width — `160`, `240`, `288` (`0x43200000`, `0x43700000`,
`0x43900000`) — and every twelfth frame (0x40ebc0 on the timer at `+0x2e14` with `0xc`) at a narrower
one, `128`, `192`, `224`. 0x44a6a0 halves the size it is given once (0x44a793), so the box tested is
`±295` along and `±80`, `±120` or `±144` across.

**`[enemy+0x2ca0]` is the enemy's script context, and `[context+0x10]` the behaviour it runs every
frame.** The interpreter at 0x4184b0 has an instruction whose handler at 0x41d569 reads the table at
`0x4c6cb0` by the instruction's argument and writes the entry into `[context+0x10]` (0x41d57f), with the
instruction itself into `[context+0x14]`; 0x424730 is the table's entry 9 (`0x4c6cd4`). So an enemy
whose context holds one of the three addresses is firing that beam now, and `sight.rs` hands it over as
a beam of the width tested every frame — body or no body, the body's own bit being no gate on this.

**The table holds 32 of them and three test the player**, which is what says the list is complete:
`0x4c6cb0` through `0x4c6d30` is 32 function pointers, and 0x424730, 0x424820 and 0x424910 are entries
**9, 11 and 25**; the beams are the only entries that reach 0x44a6a0, and no entry reaches 0x44a230 or
0x42c290 at all. The call is 0x41e870 — `call [[enemy+0x2ca0]+0x10]` with the enemy in `ecx` — at the
tail of the interpreter's own instruction loop, behind a gate at 0x41e7fb that skips it where
`[enemy+0x2dfc]` is nought or less, and the handler at 0x41d593 writes nought over the field to stop it.

**And those nine sites are every way a run dies**, the three tests having one caller each and no other:
0x44a230 is called from 0x4316b1 alone (the bullet walk), 0x44a360 from 0x42c362 alone — inside
0x42c290, which is called from 0x42cf7a (the body) and 0x42d063 (the trail's boxes), both behind
0x42cf5e testing bit 2 of `[enemy+0x3324]` — and 0x44a6a0 from the three beams' six sites and the laser
walk's three. So a hit naming no bullet, no solid enemy, no laser and no beam is a hit orb read one of
those four wrong, and there is no fifth thing to look for.

**`[[enemy+0x2ca0]+0x38..0x3c]` is `295 * (cos, sin)` of the enemy's own angle, and the game writes it
long before the behaviour.** So the enemy sits at the *middle* of the 590-long box, and that vector is
what says a beam is there at all: a launch watched one stand at `109.5,376.0` with `angle=2.011` and
`anchor=-125.7,266.9` — which is that product to a tenth — for 17.5 seconds of a run at `--pace=8` with
`[context+0x10]` **nought**, and `[context+0x10]` became 0x424730 on the very frame the player was hit at
`113.3,369.0`, 8 units from the box's middle. **So the behaviour field says a beam is testing the player
this frame and never says one is coming**, and a bot that waits for it is a bot already hit; `sight.rs`
reads the anchor instead and hands the beam over from the frame it appears, at the narrowest of the three
widths until the behaviour says which — and as a beam that has not fired, the behaviour being the whole of
what says one has.

**A launch is what said this was there to read.** A whole Normal run in Pointdevice mode lost stage 4's
boss twenty-eight times at `103.3,368.9` on a move judged `236.7 of room`: `22/22` bullets agreeing,
every one of them 280 units off, no laser alive, and the nearest enemy a bodiless one at `130.8,371.5`
sitting directly under the boss at `130.8,76.5` — the player 27 units from a column 160 wide. **And the
beam is not laid from the boss**: the one that killed the run at `113.3,369.0` ran from `235.2,109.1` at
2.011 rad while the boss stood at about `210.8`, so a rule that keeps a run a distance from her x is no
dodge for it.

**And the anchor says only that an enemy is one that lays a beam.** The card the fight dies on is number
**90** of the 222, `恋符「マスタースパーク」` — read off the up card's own `+0x14` at 0x4ea684 and its number
at 0x4ea678 with a debugger attached to a launch sitting in the chapter — and it stands **four** bodiless
enemies up with an anchor apiece, firing them in turn. A launch that read every armed one as a lane already
testing the player had all four across the bottom of the field, `half of it 80.0,80.0` at `262.8,367.1`,
`189.3,435.0`, `120.8,412.1` and `116.1,436.7`, and died 95 times at `91.4,432.0` with `nowhere open to it`
— every retry of the chapter on the same frame of it, `frames=31716 seed=0x727f boss_life=1445
attack_frames=1205`, the nearest lane 46 units inside. So **the behaviour field is what says a lane is
live** and the anchor is a warning, which is what `sight.rs` hands one over as: a beam
`game::FIRES_WHEN_NOTHING_SAYS` frames off, at the narrowest of the three widths until the behaviour says
which.

The strategy wiki for this fight says the same thing from the other side —
`斜めに撃たせ下で撃ちこみ、斜めの位置に逃げるの繰り返し`, and `レーザーは光が薄い部分に当たり判定なし`: the
lane that takes a life is the bright core and not the whole draw, which is the `160` of the three and not
the `288`.

### The boxes a long enemy has past its body

**`[enemy+0x3394]` is a trail**, `0x1c` bytes an entry and two floats of each one a position — the same
shape `+0x2d88` is, since both go to the same 0x42c290. `[enemy+0x5350]` is how many entries are filled,
as a signed word, and `[enemy+0x534c]` a flags byte.

**The walk is 0x42cfaa through 0x42d068**: an index that starts at **1** and steps by **6** (0x42cfb6),
bounded by that word (0x42cfbf), each visited entry handed to 0x42c290 with a size (0x42d058). So one
entry in six is a box and the other five are not, which is what *What is not established* keeps.

**Three gates stand over it, and two are the body's own**: 0x42cf3e and 0x42cf47 skip the whole of both
where bit 31 of the flags is set and `[0x17d6ed4]` is not nothing; 0x42cf5e skips both where bit 2 —
having a body to run into — is clear; and **0x42cf8b skips the trail alone where `[enemy+0x534c]` is
nothing at all**, which is every enemy but the long ones.

**The size each box is tested with is the enemy's own, tapered where bit 1 of that byte is set**:
0x42cfe0 through 0x42d03a works out `[enemy+0x2d70] * index / count`, so the far end of a long enemy is
narrower than its head. Where the bit is clear each box is the whole size.

So `sight` hands one `Kind::Trail` over per visited entry, behind the same two gates, with the same
`ENEMY_BOX_OF_ITS_SIZE` the body's box gets — 0x42c290 being the one test both go through.

## How a run of th08 ends

**`0x439bc7` is the run manager's own ending section**, a `__thiscall` on the game manager, and what
decides whether it does anything at all is **bits 5 and 6 of `0x164d0b4`** — the same word the run's
kind is in, `[gm+0x3dbac]`:

| `(word >> 5) & 3` | what happens |
| --- | --- |
| 0 | 0x439bec jumps past the whole of it: the run goes on |
| 2 | the ending starts — 0x439c13 ors `0x60` in, 0x439c1f writes `0x160f540 = 1`, 0x439c29 puts `0x164d0a4` to `-1` |
| 1 or 3 | 0x439c04 jumps to the hand-over tail at 0x439f3c |

**`0x164d0a4` — `[gm+0x3db9c]` — is the scene the run wants next**, `-1` for none: 0x439fa8 asks whether
it is `0` or more and 0x439fb6 is the one place that writes it into the supervisor's own wanted scene.
The tail gates that on a press: 0x439f3c wants `[0x164d528] & 0x1001` to have changed since
`[0x164d530]`, unless the run reads its own input (bit 3 of the kind) or the stage is 6, 7 or 8.

What the ending's own body then puts in `0x164d0a4` is read off four sites: **6** where the run kind's
bit 0 is set (0x439d3b, which also copies `[run+0x8]` over `[run+0x0]` — the real score over the shown
one) and where 0x439ef8 is reached; **9** where `[run+0x28]` has reached `0xc` (0x439e09) and at
0x439f17; **7** at 0x439e43 and where a replay has no stage left (0x439dc8, walking the nine pointers
at `[[0x18b8a28]+8] + 0x20 + i*4`); and **3** where there is a next stage (0x439ddc, 0x439e25), which is
`SCENE_RUN_AGAIN`.

**Two of those paths return 3 from the update** — 0x439d45 and 0x439e13 — and 3 is what stops the
chain's walk where it stands, which is what made orb's first try at leaving a run fault. So the way out
is not a write of the wanted scene: it is `[gm+0x3db9c] = 1` and these two bits set to 1, which is what
`Th08::leave_run` writes. **Measured**: a launch read `scene=2->2` on orb frame 900 and `scene=1->1`
with `front end: screen 0, cursor on item 0` on frame 902, and no `crash:` line anywhere in it.

## The replay, and the set a stage of a run is restarted from

**`0x18b8a28` is the replay manager** — `ReplayInf`, off the allocator call that names it — and the
chain runs its element at priorities 6, 7, 0x11 and 0x12 (0x452060), with update 0x452830 where it is
recording and 0x452d60 where it is playing back. `[+0x8]` is its data, `ReplayDataInf`, whose header is
`0x134` bytes (`imul eax,eax,0x134` at 0x46e286, `rep movs` of `0x4d` dwords at 0x46e29c); `+0x6a` is
the team and `+0x6b` the difficulty (0x467be8, 0x467ba7).

**Nine per-stage records, at `data + 0x20 + i*4`**, each a `0xd2f00`-byte allocation the allocator call
at 0x452bc9 names `rep data`; 0x439d9f walks them to find the next stage a replay covers, and
`data + 0x44 + i*4` is a second one of the same size per stage. A record's first `0x24` bytes are its
header and the input buffer begins at `+0x24` — 0x452cef takes `record + 0x24` into `[replay+0x50]`.

**And the header is what a stage of a run has to be restarted from**, which is what `CLAUDE.md` asks a
resume be shaped from. Read off the fill at 0x452b5f through 0x452ce6:

| in the record | out of | what it is |
| --- | --- | --- |
| `+0x0` | `[run+0x8]` | the score |
| `+0x4` | `[run+0x30]` | the point items taken |
| `+0x8` | `[run+0xc]` | the graze |
| `+0xc` | `[run+0x34]` | — |
| `+0x10` | `[run+0x38]` | the point items the night asks for |
| `+0x14` | `[run+0x24]` | what a point item is worth, which 0x440e50 reads as one is taken |
| `+0x18` | `[run+0x22]`, a word | — |
| `+0x1a` | `0x164d2c4`, a word | the word the generator started the stage on |
| `+0x1c` | 0x422480 → `[run+0x98]` | the power |
| `+0x1d` | 0x42f2b0 → `[run+0x74]` | the lives |
| `+0x1e` | 0x4398ce → `[run+0x80]` | the bombs |
| `+0x1f` | `0x164d334`, a byte | — |
| `+0x20` | `0x164d0b0`, a byte | — |
| `+0x21` | `[run+0x1c]`, a byte | — |
| `+0x22` | 0x406dd0 → `[run+0x28]`, a byte | the field 0x439df7 compares against `0xc` |

**And the generator goes in at the buffer's own start**: 0x452d13 and 0x452d1e put two words of nothing
at `record + 0x24` and `+0x26`, and 0x452d34 writes `0x453c20(0x164d520)` — the generator's word — into
`+0x28`.

**The restore is 0x452ee1 through 0x452fe5**, and every field goes back where the fill took it from, with
three exceptions worth having: the power, the lives and the bombs go through the game manager's own
setters rather than into the run (0x452f27 → 0x406fa0, 0x452f03 → 0x406e20, 0x452f15 → 0x406f30, each of
which turns the whole number into the float the run keeps); the seed goes through 0x453be0, which is a
write of the word and nothing else (0x452f94); and **the score is not put back at all** — a replay
arrives at it by playing the buttons in. Two more come out of the data's own header rather than the
stage's record: `[data+0x6a]` into the character and the shot (0x452e99, 0x452eb6) and `[data+0x6b]`
into the difficulty (0x452ed5), and 0x452f85 copies `0xf` dwords from `data + 0xb4` into whatever
`0x160f50c` points at.

## The seal over the run's numbers

**The run's numbers are sealed, and the enemy manager stops the chain where the seal is broken.** Its update
asks 0x40bb80 where its own timer `[enemies+0x9dced0]` is a multiple of 200 (0x42de49, through 0x40d410's
`count % n`), and where that answers true it answers **4** at 0x42de71 — which leaves the chain's walk on the
spot (the table at 0x43cb38) — and jumps past its own timer's step at 0x42de78. So the next frame is the same
multiple and the same check, and every job registered after the enemy manager (priority `0xb`) goes unrun from
then on: the card object at `0xc`, the bullets at `0xe`, the run screen at `0xf` and the replay's element at
`0x11`, which is what copies the frame's read into the word the player moves on (0x452340 into `0x164d52c`).

0x40bb80 compares two things, both off the run at `[gm+0x8]`:

| | against |
| --- | --- |
| `[run+0xc8]` | `[run+0xd8] * 0x274 + [run+0x50]` |
| `[run+0xcc] + [run+0xb4]` | the float at `[gm+0x3ddf4]` (`0x164d2fc`), through 0x4a3e70 |

**0x439a2e is the sum**, `__thiscall` on the game manager: 0x439ac5 over the run's `0x80` bytes from `+0x48`,
its `0x14` from `+0xd0`, `0x3c` of the block `[gm+0x4]` names, `0x3c` of the configuration at `0x17ce870` and
`0x168` of the game manager from `+0x3da2c` — `0x274` bytes in all, and 0x439ac5 adds `[run+0xd8]` to
`[run+0xc8]` for each byte it adds up.

**The run manager's update takes it every frame** (0x43a4aa through 0x43a4d7): `[run+0x50]` over `[run+0xc8]`,
then the sum, and the sum plus `[run+0xb4]` into `[gm+0x3ddf4]` — so the first row holds by construction and
the second asks whether `[run+0xcc]` is the sum of those bytes as they are now. It also writes a constant over
that float where any of seven words from `[run+0x48]` is outside `0x198f` to `0x1a02f` (0x43a4f9) or either
float at `[run+0x78]` is outside a pair of bounds (0x43a540).

**0x406e50 is the reseal**: two draws of the generator (`0x406ef0` over `0x164d520`, under `100000`, plus
`0x198f`) into `[run+0x50]` and `[run+0xb4]`, the first copied over `[run+0xc8]`, the sum into `[run+0xcc]`, and
the float written as the run manager writes it. The power's and the lives' setters (0x406fa0, 0x406e20) call it
after writing their float, and the bombs' (0x406f30) does the same inline. The build reseals four times
(0x43ae4f, 0x43ae85, 0x43afcf, 0x43aff3), before 0x409b20 is called and so before a resumed run's seed goes in.

**Measured**: a run picked up in stage 2, whose power was written straight over the `0.0` its build had
sealed, read `xcc=16295 xb4=7534` against a float of `24039` at every check — `210` apart, the four bytes of
`72.0` — and the enemy manager answered 4 from the stage's first frames on, with a breakpoint on 0x452310
never hit in the whole launch; a stage 1 pick-up under that breakpoint reached it from its first frame to its
last.

## Where a stage is built, which is where a resume goes in

**`0x43aaf4` is the build**, and what names it is 0x43aa5c: the run manager's own registration, which
writes 0x439bc7 as the element's update, 0x43aaf4 as its added callback and 0x43be2c as its deleted one
into `0x164d344`, hangs the game manager off `[elem+0x1c]` and adds the element to the chain at priority
2. The chain calls an added callback as `mov ecx,[elem+0x1c]; call [elem+0x8]`, so the build is a
`__thiscall` on the game manager — which is what `Entered::InEcx` is for.

It branches on the wanted scene at 0x43aafd — **3, 0xb and 0xc are a stage that carries a run on** and
anything else is a run starting — and runs to 0x43b932, `0x164d0b4 >> 3 & 1` at 0x43b388 taking a replay
down a path of its own that saves and puts the generator's word back around 0x43b3b9's call.

**0x43b3cb copies the generator's word into `[gm+0x3ddbc]`**, which is the seed a stage started from, and
**0x409b20 is the job called three instructions later** — from 0x43b3e2 and from nowhere else in the exe.
That is the one place a resumed run's seed can go in, and writing `[gm+0x3ddbc]` with it is what leaves
the two agreeing; `Th08::seed` writes four numbers for that reason, the generator's own kept copy at
`+0x2` and its count of draws at `+0x4` being the other two.

So `Hooks::stage_building` is a patch over 0x409b20 and `Hooks::stage_begun` one over 0x43aaf4, entered
in `ecx`.

## The score file, and the records it is read into

**`score.dat`** (named at `0x4b5994`), read through `0x45a5e0` and written through `0x43e8f0` with the
name in `ecx` — the buffer handed to that write is what `0x43e4a0` encoded, out of the parameters `0x59`,
`0x79`, `0x100` and `0xc00` (0x45422b through 0x454248). `0x45afc0` is **not** the write: it frees the
loaded image and the scratch chunk it carries.

**Its header**, as the save at 0x45412a builds it: `+0x1` and `+0x6` random bytes out of `0x410c00`,
`+0x2` a word checksum of every byte from `+0x4` on, `+0x4` the version `1`, `+0x8` the offset the first
chunk is at (`0x1c`), and `+0x10` the whole length. The reader walks from `image + [image+0x8]` for
`[image+0x10] - [image+0x8]` bytes.

**A chunk is a tag, a length and a version**: `+0x0` four characters, `+0x4` its whole length as a word
(which is what the walk steps by), `+0x6` that length again and `+0x8` a version byte. Six tags are read,
each by a reader of its own, and `warning : score.dat chksum error` names the check the load makes:

| tag | reader | version | index | bytes | read into |
| --- | --- | --- | --- | --- | --- |
| `CATK` | 0x45aac0 | 3 | word at `+0xc`, `< 0xde` | `0x22c` | `gm + 0x40 + i * 0x22c` |
| `CLRD` | 0x45ac90 | 4 | byte at `+0x21`, `< 0xd` | `0x24` | `gm + 0x3c490 + i * 0x24` |
| `PSCR` | 0x45ae00 | 2 | byte at `+0x174`, `< 0xc` | `0x178` | `gm + 0x3c664 + i * 0x178` |
| `PLST` | 0x45af30 | 2 | none | `0x228` | `gm + 0x3d804` |
| `FLSP` | 0x45ac00 | 1 | none | `0x20` | `gm + 0xc` |
| `LSNM` | 0x45ab70 | 1 | none | `0x18` | the result screen's `+0x46450` |
| `HSCR` | 0x45a950 | 4 | byte at `+0x14` and byte at `+0x15` | — | appended to a list through 0x45a500 |

### What a replay writes down per frame, which is the whole input word

**0x452310 is the element that does it**, one of the four the replay registers (0x452060), and it runs
behind bit 2 of the run's flags word. Its first act is to shift the words the game keeps: `[0x164d52c]`
into `[0x164d534]` and `[0x164d528]` into `[0x164d52c]` — so **this frame's word, the frame before's and
the one before that** are three statics and not a buffer.

**And the write itself is 0x4523af through 0x4523f9**: it takes `[0x164d528]` as a **word**, steps the
cursor at `[replay+0x50]` by 2, keeps that cursor in `[replay + stage*4 + 0x54]` — the per-stage end, the
stage index coming from `[0x164d2d0]` — and writes the word at the cursor. So a replay of this game is
**two bytes a frame and the whole input word**, not a mask of it: nothing is taken out for the pause
menu's own bit.

**Every thirtieth frame it writes something more** (`idiv 0x1e` on `[replay+0x0]` at 0x4523fc), out of
`[0x17ce8f0]` and `[0x17cea90]`; what that is, is not read.

**`FLSP`'s own first nine bytes are its chunk header and nothing else of it is read.** 0x453ff5 through
0x454015 writes the tag `FLSP` into `[gm+0xc]`, the size `0x20` into the word at `[gm+0x10]`, that same
word again into `[gm+0x12]` and the version `1` into the byte at `[gm+0x14]`, and then hands `0x20` bytes
from `[gm+0xc]` to the writer. The `0x17` bytes after that header are read by nothing.

**And `PLST` is the Player Data screen's, every number of it.** Its block is `gm + 0x3d804` — `0x164cd0c`
as a static — and the screen that draws it (0x4578f0 through 0x457d60) names each row with a format
string of its own: `総起動時間`, `総プレイ時間`, `プレイ回数` over `Easy Norm Hard Luna Extra Total`,
`クリア回数`, `コンティニュー` and `プラクティス` (0x4b7a9c down to 0x4b79a4). A grep of the listing for
the block's span, absolute or off the game manager, answers in 188 lines, three of them just past its
ends, and every field below is one of the other 185:

```sh
grep -E '0x164c(d[0-9a-f]|e[0-9a-f]|f[0-3])[0-9a-f]\b|0x3d([89][0-9a-f]|a[0-2])[0-9a-f]\b' th08.asm
```

| | |
| --- | --- |
| `+0x0` | the chunk header: `PLST`, the size `0x228` twice and the version 2 (0x445fe5 through 0x445fef) |
| `+0xc`, `+0x10`, `+0x14`, `+0x18` | **総起動時間**, the time the game has been running: hours, minutes, seconds and milliseconds — 0x448331 divides a count of milliseconds by `0x3e8` into `+0x14` and keeps the remainder in `+0x18`, and 0x4483a0 through 0x448404 carries each into the one before at `0x3c` |
| `+0x1c` through `+0x28` | **総プレイ時間**, the time played, in the same four fields — 0x44847f adds a quotient of `0xea60`, a minute of milliseconds, into `+0x20` |
| `+0x2c + d * 0x44` | **the counts of difficulty `d`**, seven blocks of seventeen dwords: `d` is `[0x160f538]` for Easy to Extra and a sixth that index can hold, and the seventh, at `+0x1c4`, is every difficulty's |
| block `+0x0` | the runs started on that difficulty (0x43b05c, the run's build) |
| block `+0x4 + c * 4` | the same for character `c`, twelve of them (0x43b08d) — the screen's rows |
| block `+0x34` | the runs started again off the game-over menu, which asks for scene 10 (0x4045f1) where the build counts it (0x43b0b9); the screen draws no row of it |
| block `+0x38` | **クリア回数** — the ending's start (0x429a2f), and the run manager's stage end where it ORs `0x8000` into the run's `CLRD` word (0x439eaa through 0x439ed5) |
| block `+0x3c` | **コンティニュー** — the game-over menu's other branch (0x405132 through 0x4051b6), which puts the point items at `[run+0x30]` back to nothing and their want at `[run+0x38]` to 100 and bumps this beside the runs started |
| block `+0x40` | **プラクティス** — the build, where bit 0 of the run kind is set and bit 14 is not (0x43b0ea) |
| `+0x1c4` | and the whole game's runs started, which the front end holds two of its own sprites hidden under until it reaches 30 and 60 (0x46862d, 0x468657) |
| `+0x208` | the Music Room's tracks heard, a byte each — 0x44806d writes `1` at the track being played, and 0x449a6e is the room reading one back |
| `+0x21a`, `+0x21b` | two more of those bytes, which the ending writes: `1` into both on one of its paths (0x429c08, 0x429c0f) and `0x12` into the first on the other (0x429c43) |

**The Total column is the game's own sums** except in one row: 0x459f36 through 0x459f75 adds each of the
five difficulties' clears into `+0x1fc` and continues into `+0x200`, and the screen's continue row prints
`+0x1bc` — the sixth block's own — under Total rather than that sum.

**`HSCR` is the ranking and is not read into a block**: 0x45a950 takes a character and a difficulty and
collects every chunk whose `+0x14` and `+0x15` are those two into a list, which is how one screen's rows
are gathered. `[chunk+0x26]` is read as a signed byte at 0x45aa4e.

**And the blocks the other five land in are one run of the game manager, boundary for boundary** — every
one of these was arrived at by adding the count to the base and finding the next block's own base:

| | |
| --- | --- |
| `gm + 0x40` | the 222 live `CATK` records, `0x1e228` bytes |
| `gm + 0x1e268` | the same 222 records kept aside, copied there at 0x43bdc1 as `0x788a` dwords |
| `gm + 0x3c490` | the 13 `CLRD` records, `0x1d4` bytes |
| `gm + 0x3c664` | the 12 `PSCR` records, `0x11a0` bytes |
| `gm + 0x3d804` | the one `PLST` block, `0x228` bytes |
| `gm + 0x3da2c` | `0x168` bytes, zeroed at 0x43bdd2 and then given the character, the difficulty and `0xf` dwords of `th08.cfg` out of `0x17ce870` |
| `gm + 0x3db94` | cleared at 0x43bdfc |

**0x43bbe1 is where all of that is set up and loaded.** It zeroes the 222 records, stamps each of them
with its own tag, length, version 3 and index, loads the file, reads `CATK`, `CLRD` and `PSCR` out of it
— not `PLST` and not `FLSP`, which the front end's own load at 0x445fa3 reads with them — bumps a `PSCR`
count where the run kind's bit 0 is set, frees the image, and copies the 222 records to the kept copy.

**And what the front end's load then does with them is light its own items**: 0x44604c through 0x4460b7
asks the game manager three questions — 0x43c1e9, 0x43c2c5 and 0x447d04 — and puts each answer's bit 0
into **bits 15, 16 and 17 of the run kind word** at `gm + 0x3dbac`. So which of the title's items are
offered is three bits of that word, read out of the file every launch.

**What an unlit item does to the cursor, measured.** A launch of a copy whose records hold no clear
pressed `down` at the title for 1,800 frames and orb read the cursor going **0, 3, 4, 5, 6, 7, 8 and
round to 0**: items **1 and 2 are skipped outright** rather than shown and refused, which is Extra Start
and Practice Start on a copy that has cleared nothing. So a recipe cannot walk to either before a clear
is in the records, and a `until: { cursor: 1 }` on such a copy gives up its whole bound.

**And item 8 is Quit**: the same launch's decide landed there and the supervisor's wanted went to `-1`
on the next frame, which is the process ending — so a walk that overshoots the list closes the game.

### What a clear writes into a `CLRD` record, and what the Extra is lit from

**A `CLRD` record is a character's, and it holds two arrays of five words, one per difficulty**: `+0xc`
and `+0x16`, Easy to Extra, with the record's own index at `+0x21`. A dump of all thirteen out of a
running game at the title — `dw 0x164b998 L0xea` under a non-invasive `cdb`, after three Normal clears
by team 0 — read `0001 0007 0001 0001 0001` in both of team 0's arrays and in the thirteenth record,
which is the total, and `0001` everywhere else.

**The Extra is lit from bit 14 of the first array.** `0x43c1e9` answers yes where `0x43c15f` does for
any of the records 0 to 3 — the four teams — and `0x43c15f(team)` tests `0x4000` in each of that
record's `+0xc` words for Easy, Normal, Hard and Lunatic (0x43c176 through 0x43c1c4). `0x43c23b` is the
same shape over `+0x16` and `0x8000`.

**Three writers, and only one of them writes those bits:**

| site | what it ORs in, and where |
| --- | --- |
| 0x43ac0c | not a write of a record: the stage's build sets `[gm+0x3ddd0]` to `1 << stage` |
| 0x439c50–0x439d02 | at every stage's end — the run manager's update 0x439bc7, where bits 5–6 of the run kind read 2 — that bit into the difficulty's `+0x16` word, and into its `+0xc` word as well where `[run+0x29]` is nought; the thirteenth record's with them; skipped where bit 3 of the run kind is set |
| 0x429a00–0x429c07 | at the ending's own start — `0x429980`, the `+0x8` callback of the scene's element — **`0x4000` where the stage word is 7 and `0x8000` otherwise**, into the same word the row above chose; skipped outright where bit 4 of the run kind is clear |

**And a Final A clear reaches the ending with bit 4 clear.** 0x439d4d sends a stage word of 6, 7 or 8 to
the last-stage path at 0x439e34, which sets bit 4 on Easy to Lunatic (0x439f0f) and asks for scene 9 —
but 0x404938, on the same difficulties, clears bit 4 again and asks for scene 9 itself wherever
`0x406dd0()` answers 11 or more **or the stage word is not 7** (0x404966 through 0x4049b0). So the one
clear that writes a bit the title reads is Kaguya's, which writes `0x4000` — the bit `0x43c15f` tests.

**The route to Kaguya is the stage-end bits, not the ending's.** 0x439856 is called at 0x433b47 through
0x433b95 as `(bit 6, record [0x164d0b1], word k)` for k of 1, 2 and 3 — the arguments pushed right to
left — so it asks whether the character's `+0x16` word for Normal, Hard or Lunatic has **bit 6**, which
is `1 << 6`: stage word 6, Eirin's, passed. What is not established is why three Normal clears of Eirin's
stage left that word at `0x0007` and not with bit 6 in it.

**And the file is written from memory, so the loss is in memory.** The save is 0x453d0d, called from one
place — 0x459fd2, where the screen holds a loaded image — and it writes the thirteen records straight out
of `0x164b998` (0x453e9c–0x453efe). The readers of the chunk into memory are four: the run's build on its
run-start branch (0x43af9b, through the loader 0x43bbe1), the supervisor's own added callback at startup
(0x445fa3), the title's own load (0x470a6c, which shows the message for an item newly lit and then reads
`CLRD`, `PSCR`, `CATK` and `FLSP` back), and the result screen's build at 0x459bbf — which **skips** the
reload for the phases 9, 0x11, 0x16 and 0x12 (0x459c80–0x459c9f), the ones a run's own end and the save
itself enter it in. A dump of team 0's words as Eirin's stage began read `0x0037` — stages 1, 2, 3, 4B
and 5 passed that run — and the next launch's title read `0x0007` again: the bits were in memory and not
in the file.

### The spell card that is up

**`0x4ea670`** is the card, one static object of `0x2644` bytes that 0x414590 builds. `SpellCard::Start`
is **`0x4152a0`**, a `__thiscall` on it reached from 0x4212ca with six pushes, the first of them the
card's own number out of the ECL's argument block (`[arg+0xe]`, a word):

- it sets **bit 0** and bit 2 of the flags word at `+0x0` and clears bits 3, 4, 6, 7, 9 and 10
  (0x4152b9 through 0x415350);
- writes that number into `+0x8` (0x41535b);
- copies the name it was handed into the record's `+0x10` (0x415800);
- and counts the attempt, which is the block below.

**Bit 0 is what says a card is up**: `0x4178a0` answers it and the enemy walk asks it at 0x42b05d and
0x42d091, where a card being up changes what a hit does. `0x405260` answers bit 9 instead, which
`Start` clears; `+0xfc` is the bonus a capture is worth.

### What a card pays in time orbs, which is what the Last Spell is decided by

**`+0x10` is that payout**, answered by the getter `0x453cc0`, and the one thing that reads it is the Last
Spell's own question at 0x41fb0b: the orbs picked up (`0x421b60` on the game manager), this, and the orbs
still in the air (`0x4417e0` over the item list, counting class 7) summed and compared against the stage's
norm (`0x421b80`), answering **2** where the sum has reached it and **0** where it has not. So the payout
is a term in that question and never a number the panel's own count holds.

**And it is written at the capture, on a ramp** — 0x416262 through 0x416347, in the branch bit 2 of the
card's flags word takes:

| address | what it writes |
| --- | --- |
| 0x416283 | `[card+0x104] = [card+0xfc]`, the score bonus kept |
| 0x4162a2 | **700** flat, where bit 27 of `[[card+0x4]+0x3324]` — the enemy's own flags — is set |
| 0x4162b7 | otherwise `total` is `[card+0x114]`'s count and `most = total - total / 7` |
| 0x4162ef | **1,000** where `[card+0x108]`'s count — the time left — is `most` or more |
| 0x416338 | `(left - 180) * 900 / (most - 180) + 100` where the left is `0xb4` or more |
| 0x416340 | **100** below that |

So a card taken inside the first seventh of its own time pays the whole thousand and every frame saved
before that pays a share of it. `Th08`'s `CARD_TIME_ORBS` carries the same reading beside the offset, and
the reproduction line says all three numbers.

### What a `CATK` record holds, which is one spell card's own history

`+0x10` is the card's **name**, copied out of the card object's `+0x14` at 0x415800. `+0xe` is a checksum
byte and `+0xf` takes the difficulty (0x4164f3). Then **six arrays of thirteen dwords**, `+0xf0` through
`+0x227` — twelve characters and a thirteenth slot that is the total across them:

**They are the same three things twice: once for an ordinary run and once for a spell practice**, and
which pair is written is bit 14 of the run kind — the bit `0x406c70` answers and
`RUN_IS_A_SPELL_PRACTICE` is. 0x415adb branches the attempt on it and 0x416500 the capture.

| offset | total at | what | written at |
| --- | --- | --- | --- |
| `+0xf0` | `+0x120` | the best bonus taken, raised to `[card+0xfc]` | 0x416539, 0x416560 |
| `+0x124` | `+0x154` | the **attempts** of an ordinary run | 0x415b1a, 0x415b39 |
| `+0x158` | `+0x188` | the **captures** of an ordinary run | 0x41658f, 0x4165b4 |
| `+0x18c` | `+0x1bc` | the best bonus of a **spell practice** | 0x41665c, 0x416683 |
| `+0x1c0` | `+0x1f0` | its **attempts** | 0x415b7c, 0x415ba1 |
| `+0x1f4` | `+0x224` | its **captures** | 0x4166b2, 0x4166d7 |

Every count stops climbing at `0x270f` and every total is the thirteenth slot of its own array, which is
what the `+0x34` between each pair's two addresses says. The character the twelve are indexed by is
`[gm+0x3dba9]`, which is `G_CHARACTER`.

0x4159a0 through 0x415a3b sums the record's name as signed bytes and then the low byte of all six arrays
over all thirteen slots, and compares it against `+0xe`; where they differ every one of the six is zeroed
(0x415a74 onwards), which is the file's own tamper check. 0x416772 writes it back.

`PLST` and `FLSP` are copied in and out whole — what each field of the first holds is its own table above.

## What is not read of the rest

**An enemy's hit boxes past the first.** 0x42d063 walks `enemy + 0x3394 + i * 0x1c` and hands each of
them the same turned-box test at 0x44a6a0 the body gets, so a big boss's far edges are boxes orb does
not hand over.

### The generator

**`0x164d520` is the whole of the state**, and the draw at 0x43ecc0 is short enough to read off:

```text
w = [this+0x0]                  # a word
w = (w ^ 0x9630) - 0x6553
w = (w & 0xc000) >> 14 | w << 2 # written as `lea ecx,[edx+eax*4]`
[this+0x0] = w                  # and answered in ax
[this+0x4] += 1
```

0x43ed20 calls it twice and glues the two words into a dword, which is what every other draw goes
through; `0x406ef0` takes a bounded one by `div`ing that dword by its argument.

**The object has two more fields.** `+0x2` is a kept copy the game puts back into the word at 0x43a1b1,
and `+0x4` is the count of draws. Both would have to go back with the word for a chapter to draw what
the run drew.

**And the word a stage started from is kept in the game manager**: 0x43b3d2 copies the generator into
`[gm+0x3ddbc]` — `0x164d2c4` absolutely — having saved and put back the word around the call before it
(0x43b3ac and 0x43b3c2), which is a stage's build taking its own seed without spending the run's.

At startup the seed is the low word of `timeGetTime()`: 0x4464ee takes the time into `0x17ce900` and
0x4464ff writes its low half into the generator.

### The run's own frame count

**`[gm+0x3de04]`** — `0x164d30c` absolutely — which the enemy manager's update steps at 0x42c6ad, the
one place in the exe that adds one to it.

**It is the run's clock and not the stage's, which a launch settled and this section had wrong.** The
eighth launch of the Normal recipe cleared stage 1, and the two state lines either side of the
transition read `stage=0 diff=1 frames=18218` and `stage=1 diff=1 frames=18218` — the same number —
with the run's own first frame at `frames=0`. Nothing resets it at a stage change, and no other field
looked at does either.

**Which is what a midstage table of this game cannot be written in.** A boundary inside a stage is a
number on a clock, and on this one the number for anything past the first stage is the whole run so far
plus the way in — different every run. th06's and th07's rows are per-stage counts because their clocks
restart, so the table needs a clock that restarts; the stage script's own, below, is the candidate.

**The one gate on it is the dialogue**, which is what makes it a clock a midstage boundary can be a
number on: 0x42c69c asks 0x4358bb on `0x160f428` and 0x42c6a1 skips the increment where the answer is
yes, and that question is *is a line up* and nothing else — see the dialogue below. A death does not
stop it, a bomb does not, and the frames a player spends pressing through a conversation are the frames
it leaves out. So it advances the same way however the player is doing, and `State::stage_frames` is
what orb fills from it — the frames of the run, under the name the state carries for them.

**Two other fields look like this one and are not it.** `[gm+0x3de08]` — `0x164d310` — is stepped by the
run's own update at 0x43a269 behind a *second* gate, `[0x164d0bd] == 1`, and orb read `frames=0` through
a whole stage with it. `[gm+0x3daa8]` is stepped at 0x43a1a1, in a block that also writes the play area
back and plays the `Pause` sound — once a pause rather than once a frame — and read `frames=0` too.

### The clock the stage's own script is run on

**`[enemies+0x9dcdd0]` is an array of sixteen contexts of `0x10` bytes**, which is the one thing between
the `0x1e1` enemies below it and the timer at `+0x9dced0` above: 0x42a243 array-constructs `0x10` of
`0x10` bytes there with 0x449f50, the enemy manager's own constructor doing it once.

**The walk is the enemy manager's update.** 0x42c7d5 runs as many contexts as the loaded `.ecl` says —
0x42dfb0 answers `[[0x4eccb8]+0x6]` as a word — and hands each to 0x42a8a0 as
`enemies + i * 0x10 + 0x9dcdd0`, filling that context's `+0xc` from 0x42dfd0, which is
`[[0x4eccb8] + i * 4 + 8]`, where it is still nothing.

**A context is a timer and an instruction pointer.** `+0x0`, `+0x4` and `+0x8` are the timer shape every
other clock in this game has — 0x406660 copies the count into `+0x0` and steps `(+0x4, +0x8)` through
the supervisor's 0x447421, 0x40d3b0 answers `+0x8`, and 0x406610 sets `+0x8` with `+0x4` at nothing and
`+0x0` at `-0x3e7` — and `+0xc` is where in the script the context stands.

Run of one context, at 0x42a8a0:

| address | what it does |
| --- | --- |
| 0x42a8b6 | the instruction's time at `[ip]` below nothing ends the run: the timer is stepped and 0x42a8a0 returns |
| 0x42a8cb | 0x40d3f0 answers whether the count at `+0x8` **equals** that time |
| 0x42a8e5 | the instruction's `[ip+0x7]` against `[0x160f53c]`, which is the difficulty's own bit |
| 0x42a912 | the opcode at `[ip+0x4]`, `0x10` and under, through the table at 0x42ad60 |
| 0x42ad3d | `+0xc` on by the byte at `[ip+0x6]`, and round again |
| 0x42ad57 | 0x406640 steps the timer once, as the frame's run of that context ends |

So `[enemies+0x9dcdd8]` — the first context's count — is the clock the `.ecl`'s own instruction times
are written in, and it counts the frames of a script rather than of a run.

**It starts again with the stage, which a launch measured.** The two state lines either side of a stage
1 cleared read `stage=0 frames=17132 script=4283` and `stage=1 frames=17132 script=0`, and stage 2
counted 0, 50, 110, 170 from there while the run's clock carried on from 17,132. So this is the clock
`State::script_frames` comes off and the run's above is what `State::stage_frames` comes off.

**And it is reproducible where the run's clock is not.** Stage 1's midboss arrived at `script=2935` on
three launches and its boss at `script=4176` on four; the run's clock read 18,218 frames for that same
stage on one launch and 16,346 on another. Which is the property a midstage table's numbers need.

**A fight holds it whole**, rather than attack by attack: 4176 stood for ten thousand frames of the
stage 1 boss on one launch, through two of her cards. So a boundary inside a fight is not a number this
clock can carry, and `spell=` is what says a fight is advancing.

**The dialogue does not stop it**, though it stops the run's clock: it went 4223 to 4283 across the
boss's own conversation while `frames=` stood at 17,132. The gate at 0x42c6a1 is the run clock's alone —
the walk below it in the same update runs on a frame it skips.

What nothing in the exe writes is `+0x9dcddc` but 0x42c814, and the enemy manager's added callback at
0x42ebf0 zeroes the `.ecl` manager's `0x62` dwords from `0x4eccb8` without touching this array — so
where the reset itself is written is not read, and the launch is what says there is one.

### The dialogue

**`0x160f500` holds the screen a stage is played on**, written at 0x437b6e, and `[screen+0x8]` is the
`0x230b8`-byte `FRScreenImplInf` allocation the allocator call at 0x437b3d names. The chain runs that
screen's element — `0x160f4e4`, update 0x4338ca — at priority 0xf.

**`[inner+0x2181c]` is which line is up.** 0x4358bb is the game's own question about it and answers yes
where the field is `0` or more or exactly `-2`; 0x43397f logs `msg start %d` (`0x4b4cac`) as a
conversation begins, and 0x4339a2 zeroes the `0x55c` dwords from `+0x21814` that carry it. So the
numbered values are a line with words on it and `-2` is the frame the screen is handed back on, which is
the two predicates orb answers: `State::in_dialogue` takes both and
`Game::a_line_of_dialogue_is_shown` takes the numbered ones.

**And the instruction that starts one puts the run clock on 1,800.** The stage `.ecl`'s interpreter is
0x4185f1 — `movsx eax, [ip+0x4]`, less one, against `0xb7`, through the table of 184 entries at
0x41ecbe — and its instruction **148** is the arm at 0x41d686:

| address | what it does |
| --- | --- |
| 0x41d689 | `[ip+0xa] & 1` chooses the argument: a variable through 0x41f420, or `[ip+0xc]` taken as it stands |
| 0x41d6bd | that argument handed to 0x423130 on `0x160f428`, which is the conversation |
| 0x41d6cd | `0x708` — **1800** — added to `0x164d30c`, the run clock |
| 0x41d6d3 | written back, and the arm joins the switch's exit at 0x41e7e4 |

So the other writer of the run clock is not a counter at all, and it is the whole of the pair: nothing
else in the exe writes `0x164d30c` but this and 0x42c6ad's step, which a grep of the disassembly for the
address answers in seven lines.

**Which is what a 1,800-frame gap in a recording of a run is.** A Normal run of stage 1 wrote
`15743 frame(s) of buttons and the generator on 10344 of them` — 5,399 entries nothing was ever noted
against — and its own per-frame lines go `frame 5122 played with frames=5122` and then
`frame 6922 with frames=6922`, three times over the stage, at 5,122, 6,922 and 11,327. Those are not
frames the game ran without being read: they are numbers the clock never took, one instruction having
put it past them. The frames a conversation does run are noted against the number the clock stands on
while a line is up, each write winning over the last.

## The stage

**`0x164d2cc`** is the stage, counted from **zero**, and two loaders index tables by it:

- 0x409925 takes `[0x164d2cc * 4 + 0x4c731c]` — the `.std` table — and 0x409949 takes the same index
  into `0x4c7340`, the `_s.std` one;
- 0x42ed3b takes `[0x164d2cc * 4 + 0x4c73cc]`, the `.ecl` table, unless `0x164d0b4 >> 0xe & 1`, where it
  takes `0x4c73f0` instead — the spell-practice set.

It is also `[gm+0x3ddc4]`, which the build reads (above), so the absolute address and the offset agree.

**Nine stages**, which is what both those tables hold:

| index | `.std` | `.ecl` | the name at `0x4c7fac` |
| --- | --- | --- | --- |
| 0 | `stage1.std` | `ecldata1.ecl` | `Stage 1` |
| 1 | `stage2.std` | `ecldata2.ecl` | `Stage 2` |
| 2 | `stage3.std` | `ecldata3.ecl` | `Stage 3` |
| 3 | `stage4a.std` | `ecldata4a.ecl` | `Stage 4-uncanny` |
| 4 | `stage4b.std` | `ecldata4b.ecl` | `Stage 4-powerful` |
| 5 | `stage5.std` | `ecldata5.ecl` | `Stage 5` |
| 6 | `stage6.std` | `ecldata6.ecl` | `Stage 6-Eirin` |
| 7 | `stage7.std` | `ecldata7.ecl` | `Stage 6-Kaguya` |
| 8 | `stage8.std` | `ecldata8.ecl` | `Extra Stage` |

The name table has a tenth entry, `Last Word` at `0x4c7fd0`, which no `.std` or `.ecl` of the nine
belongs to: the Last Words load `ecldata_yk.ecl` and its five siblings out of the table at `0x4c7418`
instead. So nine is the number of stages a midstage table has rows for, and the Last Words are not one
of them.

**And a run walks six of those nine indices, not the first six.** 0x43c4b3 is the chooser, `__thiscall`
on the game manager, which the run manager calls at 0x439e20 as a stage ends and which writes the next
index into `[gm+0x3ddc4]`:

| the index a stage ends on | the index written next |
| --- | --- |
| 0 | 1 |
| 1 | 2 |
| 2 | **3 or 4 by the character** |
| 3 or 4 | 5 |
| 5 | 7 where `(0x164d0b4 >> 0xb) & 3` is anything, and **6** where it is nothing |
| 6 | 7 |

The character's own branch is two tables: the byte at `[0x43c5d5 + [0x164d0b1]]` — `00 01 02 03 00 00
01 01 02 02 03 03`, twelve bytes for the twelve characters — indexes the jump table at `0x43c5c5`, whose
four entries write 4, 3, 3 and 4. So the first team's character 0 takes case 0 and goes to `stage4b`.

**The run manager ends the run rather than choosing where the stage is already 6, 7 or 8** (0x439d4d):
each of those jumps to 0x439e34, the ending's own path. So an ordinary run's last stage is the 6 or 7 the
table above picks at stage 5, and the stage after it is no stage — which is what a recipe waiting for
this game's last stage has to be written against: `stage: 7` in the log's numbering, which is one higher
than this index.

The other branch at 0x439d74, taken where `(0x164d0b4 >> 3) & 1` is set, scans `stage + 1` through 8 for
the next index whose data is loaded — `[[0x18b8a28] + 0x8 + i * 4 + 0x20]` being anything — and asks for
scene 7 where there is none.

The difficulty names sit right after at **`0x4c7fd4`**: `Easy`, `Normal`, `Hard`, `Lunatic`, `Extra` —
five, read at 0x45489e and 0x4549df, which is what makes 4 the Extra and puts `cmp ...,0x4; jge` on the
right side of it.

`0x164d0b4` is the run's kind. Two of its bits are read: `& 1` at 0x43acaa and 0x43add9, which forces
the header's starting lives to 8, and `>> 0xe & 1` at 0x42ed2c, which is the spell practice the `sp`
ECL set belongs to. `0x164d0b8` is a word beside it holding the card, compared against `0xcd` at
0x42ed62.

## The input

**The keyboard read is `0x43d970`**, prologue `55 8b ec 81 ec 08 01 00 00` — th07's prologue, byte for
byte. It takes a `0x100`-byte buffer on its own stack, fills it with `GetKeyboardState` (0x43d996), and
folds each key into one word by testing the byte against `0x80`. Where `0x17ce768` — the supervisor's
`+0x10` — is not zero it jumps to 0x43dd18 instead, which is the DirectInput path.

The keys and the bits, read off the stack offsets (`vk = 0x100 - offset`):

| bit | keys |
| --- | --- |
| `0x0001` | `Z` |
| `0x0002` | `X` |
| `0x0004` | `VK_SHIFT` |
| `0x0008` | `VK_ESCAPE` |
| `0x0010` | `VK_UP`, numpad 8 |
| `0x0020` | `VK_DOWN`, numpad 2 |
| `0x0040` | `VK_LEFT`, numpad 4 |
| `0x0080` | `VK_RIGHT`, numpad 6 |
| `0x0050`, `0x0090`, `0x0060`, `0x00a0` | numpad 7, 9, 1, 3 — two directions apiece |
| `0x0100` | `VK_CONTROL` |
| `0x0200` | `Q` |
| `0x0400` | `S` |
| `0x0800` | `VK_HOME`, `P` |
| `0x1000` | `VK_RETURN` |
| `0x2000` | `D` |
| `0x4000` | `R` |

So shot, bomb, focus, menu and skip are `0x1`, `0x2`, `0x4`, `0x8` and `0x100`, which is th07's
assignment.

**And the branch at 0x43dd18 is the same keyboard through DirectInput and not a pad.** `0x17ce768` is a
COM object, and what the branch calls on it is `[vtable+0x24]` with `0x100` and a buffer of its own —
`IDirectInputDevice8::GetDeviceState` over a keyboard's `0x100` bytes. Where that answers `0x8007001e`
or anything else non-zero it calls `[vtable+0x1c]` — `Acquire` — and folds a word of nothing (0x43dd69,
0x43dd94). Otherwise it folds the same bits out of the buffer at the **scan codes** rather than the
virtual keys: `0xc8`, `0xd0`, `0xcb` and `0xcd` into `0x10`, `0x20`, `0x40` and `0x80`, which are
`DIK_UP`, `DIK_DOWN`, `DIK_LEFT` and `DIK_RIGHT`.

Which is why orb's patch covers both: it stands over the whole of 0x43d970 as a `Read::LookedUp` and
answers the word, so whichever branch the machine would have taken is never run.

### The word the run acts on, which is a copy made a frame late

The read lands in `[0x164d528]` (0x4455f5), with last frame's copied into `[0x164d530]` in front of it
(0x4455e4) — the pair every screen and the pause take their edges off (0x4037bc, 0x43a0e8 through
0x43a10e). **The run moves on another pair**, `[0x164d52c]` and `[0x164d534]`:

| where | what |
| --- | --- |
| 0x44aee8 through 0x44b01a | the player's job, at chain priority 9, tests the directions of `[0x164d52c]` (`0x50` first) |
| 0x44c8e1 through 0x44c903 | the same job bombs where bit `0x2` of `[0x164d52c]` differs from `[0x164d534]` |
| 0x4349d8 through 0x434bee | a conversation takes its next line off the same pair |
| 0x452332, 0x452340 | the run's replay, whose element is at priority 0x11, copies `[0x164d52c]` into `[0x164d534]` and the read into `[0x164d52c]` — where bit 2 of `[0x164d0b4]` is set (0x452319) |
| 0x4525aa through 0x4525c1 | a replay being played copies the same first word on and its own entry into it |
| 0x451fb2, 0x451fbb | registering the run's replay puts both to nothing |

So the player moves on the word the element copied a frame ago: the priority it reads at is ahead of the one
that writes it. Measured over a whole run before orb did anything about it, the player's move followed the
word orb chose on the earlier of two frames 5,104 times and contradicted it 16. `Th08::hand_the_read_to_the_run`
writes the pair at the read instead, which is th07's answer to the same gap.

### The pad, DirectInput8's where one was found and WINMM's otherwise

**`0x43d0c0` asks whether there is one**: it builds a `JOYINFOEX` on its own stack with `dwSize = 0x34`
and `dwFlags = 0xff`, calls `joyGetPosEx(0, …)` — `ds:0x4b423c` — and where that fails logs the string at
`0x4b59b0` and returns 1. Where it answers it calls `joyGetDevCapsA(0, 0x164d388, 0x194)` — `ds:0x4b4270` —
and returns 0, so the caps live at `0x164d388`.

**`0x43d120` reads it every frame** and folds it into the word it is handed in `cx`, one of two ways.
**Where `[0x17ce76c]` holds a device** — the DirectInput pad the input's set-up found, its axes ranged ±1000
(`push 0xfffffc18` at 0x406c11) — it is `Poll` (`[edx+0x64]`, 0x43d3c1), `Acquire` while that answers
`DIERR_INPUTLOST` (0x43d3f6), and `GetDeviceState` of `0x110` (0x43d48b): each button through 0x43d6e0 by its
byte, and the two axes against the thresholds at `0x17ce888` and `0x17ce88a`, 600 by default (0x447812, 0x43d627
through 0x43d6be). **Otherwise** it is `joyGetPosEx(0)` (0x43d15e), the word handed back untouched where that
fails: `dwButtons` at `+0x20` through 0x43d760, and each axis past a quarter of the travel the caps at
`0x164d388` give it from its middle (0x43d2dd through 0x43d39e). Neither looks at the hat. Either way the shot's
button held is also the focus, `0x4`, after ten frames of it, where the byte at `+0x13e` of `0x17ce758` says so
(0x43ef50; the count is `0x164d51c`).

So a pad reaches this game the way it reaches th10, through its own read; the winmm branch's call is the one
orb's own joystick hook answers. `Th08::hooks` declines both pad hooks — see that method, where the reason is
written: a real pad works natively and orb's own d-pad does not reach a run of this game.

**There is one input word and not two.** `0x164d528` is written at four sites and no more: the
supervisor's own read (0x4455f5), the same read OR'd into whatever is already there where
`0x160f535` — the game manager's `+0x2d` — is not zero (0x44567d), the scene transition's own clear
(0x445af9), and the loading screen's read (0x47133d). Last frame's is `0x164d530`, copied from
`0x164d528` before the read (0x4455e4), and `0x164d538` is the auto-repeat word with `0x164d53c` as its
hold counter — `0x1e` frames before the first repeat and one every eight after it (0x445615 through
0x44565a).

Which is why **the pause button cannot be handed to a run of this game the way th07's can**: th07 keeps
the run's word apart from the read, and here the front end's screens and the run both read
`0x164d528`. Every screen of the front end reads a press as `(cur & mask) != (last & mask)` over that
pair — 0x4037bc for one — and the two masks are **`0x1001`** at 105 sites and **`0xa`** at 74, the same
decide and back th07 has.

## The window's own settings

The configuration is `0x17ce870`, `0x3c` bytes, written to `th08.cfg` at 0x441d43 and copied into every
run's header. **`0x4477b1` is what reads it**, `__thiscall` on `0x17ce758` with the file's name at
`0x4b5b88`: it zeroes `0xf` dwords from `0x17ce870` — which is the file's own 60 bytes exactly — opens
the file through 0x43e660, and writes its own defaults where that fails. So the file's offset *n* is
`0x17ce870 + n`.

**The first nine words of it are the pad's buttons**, one per bit of the game's own word in its order, read
off the folds at 0x43d17a through 0x43d2d8 and 0x43d4ac through 0x43d622 — each word is handed to 0x43d760
(winmm) or 0x43d6e0 (DirectInput) with the bit it stands for:

| file offset | in memory | bit |
| --- | --- | --- |
| 0 | `0x17ce870` | `0x1`, the shot — its own branch for the focus after it |
| 2 | `0x17ce872` | `0x2`, the bomb |
| 4 | `0x17ce874` | `0x4`, the focus |
| 6 | `0x17ce876` | `0x8`, the menu |
| 8 | `0x17ce878` | `0x10`, up |
| 10 | `0x17ce87a` | `0x20`, down |
| 12 | `0x17ce87c` | `0x40`, left |
| 14 | `0x17ce87e` | `0x80`, right |
| 16 | `0x17ce880` | `0x100`, the skip |

Which is the layout `launcher::pad::Mapping::read` already reads — offset 0 as the decide and offsets 2
and 6 as the two cancels — so that reader is right for this game. The copy of the install measured reads
`02 00 05 00 04 00 00 00 ff ff ff ff ff ff ff ff`: the shot on pad button 2, the bomb on 5, the focus on
4, the menu on 0, and `-1` for the directions nothing is assigned to.

Once read the rest is spread over the supervisor: the byte **`0x17ce892`** — `+0x13a` — is what
0x447bc0 tests before printing `ウィンドウモードで起動します`, so it is **windowed**; the byte
`0x17ce893` beside it is the frame-skip limit the frame counts `[this+0x10]` up to; and `[+0x150]`,
`0x17ce8a8`, is a flags dword whose bit 0xe is the vertical sync the log calls
`垂直同期を取りません` and whose answer is kept at `0x17ce8d4`.

**The play area is 32, 16, 384 by 448**, written into the game's own `D3DVIEWPORT8` at `0x17ce820` at
0x40925e through 0x40927c and set through `+0xa0`. The same six dwords carry the whole output — `0, 0,
640, 480` at 0x441f1b — so `0x17ce820` is the one viewport this game has and the frame writes it twice.

**And the same four numbers have a second spelling**, which is what makes them measured rather than
read once: the pause at 0x43a120 writes them as floats into `0x164d2dc`, `0x164d2e0`, `0x164d2e4` and
`0x164d2e8` — `[gm+0x3ddd4]` through `[gm+0x3dde0]`, the width being the third of the four — out of the
constants 32, 16, 384 and 448 at
`0x4b42cc`, `0x4b42d4`, `0x4b4528` and `0x4b4940`. Those four are what the rest of the game clips
against, and **the player's own start is `0x164d2e4 / 2`** (0x44d745, dividing by the 2 at `0x4b42ec`),
so a position inside the play area is measured from its own corner and not from the screen's.

## The sound

`0x18b8a68` is the sound manager. **`0x45d790`** is what runs its queue: it reads `[this+0x610]`,
answers 0 where that is nothing, and otherwise switches on `[this+0x1f88] - 1` over eight cases. The
frame calls it once (0x441f55) and WinMain's teardown loops on it until it answers 0 (0x441ba4), which
is what a queue runner does and what th07's `0x44c9c0` does.

**The queue is `+0x1f88`**: 31 slots of `0x10c` bytes whose first field is the kind, which is th07's own
shape at th07's own stride. 0x45e1e0 puts one in — it walks `i < 0x1f`, takes the first slot whose `+0x0`
is nothing (0x45e211) and writes the kind there (0x45e22c) — so a first slot holding a kind is a command
the runner has not carried out yet, which is what `Th08::a_sound_command_is_in_flight` answers.

**The manager is `0x5224` bytes**, off its own constructor at 0x45c190: `rep stos` of `0x1489` dwords
over `this`, then `-1` into `+0x408 + i*4` for `i < 0x80` — the eighty sound slots — and into
`+0x620 + i*4` for `i < 0xc`. It then allocates **four** bytes, constructs them with 0x4716e0 and keeps
the object at `+0x610`: a wrapper whose one field is the `IDirectSound` 0x471c40 tests as `[[this]]`.

### What a track being played is

**`0x45cc70` is PlayBGM**, `__thiscall` on the manager with the track's name: it copies that name into
`+0x5108` byte by byte, declines where `[this+0x610]` or `[this]` is nothing, takes the current track
down through **`0x45d2d0`**, and works the buffer out of the loaded file's own format at `[this+0x1f84]`
— `+0x2c` the block align as a word and `+0x24` the bytes a second. Then:

| what it makes | the call | where it goes |
| --- | --- | --- |
| the event a refill waits on | `CreateEventA(0, 0, 0, 0)`, `ds:0x4b40c4` | `+0x520c` |
| the streaming thread | `CreateThread(0, 0, 0x45e0f0, [0x17ce79c], 0, this+0x614)`, `ds:0x4b40b0` | the id in `+0x614`, the handle in `+0x618` |
| the stream itself | 0x471c40 on `[this+0x610]`, handed `this+0x5208` as an out-pointer | `+0x5208` |

Which is th07's shape at th07's own offsets for the last three: the thread's id at `+0x614` and its
handle at `+0x618` are where that game keeps its own.

**The stream is `0x78` bytes**, allocated at 0x471eb3 and constructed by 0x472aa0; 0x471f04 writes it
through the out-pointer, 0x471f16 copies the `0x24`-byte buffer description into its `+0x34`, and
0x471f20 and 0x471f2b put the manager's wrapper in `+0x58` and the chunk size in `+0x70`. The array
0x471dbb allocates — one 8-byte `DSBPOSITIONNOTIFY` per notification, handed to
`IDirectSoundNotify::SetNotificationPositions` at 0x471e2d — is freed again at 0x471ea4, so nothing of
the game's holds it.

**And the fields of it are th07's fields at th07's own offsets**, which is a thing read here and not
carried across: 0x472aa0 hands its base constructor 0x471f60 the buffers, and that one

| writes | out of | which makes it |
| --- | --- | --- |
| `+0x0` | `0x4b96bc`, then `0x4b96c0` at 0x472ac0 | the vtable, the derived class's over the base's |
| `+0x4` | an allocation of `count * 4` at 0x471f7a, filled from the caller's array | the array of sound buffers |
| `+0x8` | the second argument | how many bytes one buffer holds |
| `+0xc` | the fourth | the source the bytes are read through |
| `+0x10` | the third | how many buffers that array is of |
| `+0x5c`, `+0x60`, `+0x64`, `+0x68` | nothing (0x472ac9 through 0x472af0) | the counters a refill moves, `+0x64` being where the next chunk goes |
| `+0x6c` | 0x472ae0's own argument | how much goes in at a time |

which is `STREAM_BUFFER`, `STREAM_BUFFER_BYTES`, `STREAM_SOURCE`, `STREAM_BUFFER_COUNT`,
`STREAM_WRITE_AT` and `STREAM_CHUNK` of `game/th07/mod.rs` at the same numbers. **The source is the same
class too**: 0x471ca0 allocates `0x94` bytes for it and 0x4731f0 constructs them, and `0x94` is th07's
`SOURCE_BYTES`. So the two games share the sound engine, and what says so here is this game's own
constructors rather than the other game's constants.

`0x447764` on the manager calls four methods of that stream every supervisor update — 0x472b50,
0x472be0, 0x472cc0 and 0x472c50 — and what each of the four does has not been read.

**The name PlayBGM is handed is the archive and not the track.** All four of its call sites pass a string
literal, and the strings are `thbgm.dat` (0x4b67cc) and `th08.dat` (0x4b66c4) — so the stream is opened
over the whole music file and which track is playing is a seek inside it, which is th07's shape too.
Where `[0x17ce8a8] >> 0xd & 1` is set the name is copied into `0x18bdb70` instead of played (0x4466b2).

**What plays a named track is `0x447f21`**, `__thiscall` on `0x17ce758` with a record and a second
argument. Three call sites reach it:

- 0x44907e and 0x449236, the Music Room's own screen — the static `0x17cf4c8`, whose element the chain
  runs with the update 0x449399 — with `[this+0x78] * 0x292 + [this+0x84]`, so that screen's list is an
  array of `0x292`-byte records and `+0x78` the track it is on;
- 0x434dce, inside a run: the record is `[0x4e4824] + i * 0x80 + 0x290` and the index
  `[[inner+0x21818]+0x4]`, one field below the dialogue's own line — so the conversation's script is what
  asks, out of a table of `0x80`-byte records.

**And `0x17ce88f` picks the backend it goes to**: 0x447f30 tests it against 2 and takes the object at
`[0x17ce8e4]` through 0x444680, 0x444510 and 0x444620, and against 1 and copies the record's name onto
its own stack instead. `+0x5218` of the manager is not a track either: 0x45c41f sets it to `0x64` and
the queue runner's first case hands it to 0x472840, which is a volume.

**What a stage's build puts in place is three slots**, which is th07's shape: `0x447d56` is
`__thiscall` on `0x17ce758` with a slot and a record — the same two backends as the play, 2 going to
`0x4441a0([0x17ce8e4], …)` — and the build calls it three times off the stage's own info block at
`[0x4e4824]`:

| slot | the record | the gate |
| --- | --- | --- |
| 0 | `+0x290` | none, at 0x43b71d |
| 1 | `+0x310` | its first byte not `0x20`, a space, at 0x43b734 |
| 2 | `+0x390` | the same test at 0x43b763 |

So the track records are `0x80` bytes apart and a slot whose name begins with a space is one this stage
has none for. The conversation's own script then *plays* one of them — 0x434dce, with
`[0x4e4824] + i * 0x80 + 0x290` and `i` from `[[inner+0x21818]+0x4]`.

**And what the streaming play actually does is queue a command with a file name.** 0x448019 finds the
`.` of the record's name with 0x4a48b0 and writes `w`, `a`, `v` over what follows it (0x448026 through
0x448034), and 0x448048 hands the result to 0x45e1e0 on the sound manager as a command of **kind 2**
with `-1` beside it. So a track is played by name, and the name the manager keeps at `+0x5108` — which
`PlayBGM` copied there — is the name of whatever is playing.

**The second argument to 0x447f21 is the track's own number**: 0x44806d writes `1` into
`[0x164cf14 + n]` where the run's kind has neither bit 1 nor bit 3 set, which is the array the Music
Room lights its own list out of. The conversation's play takes that number out of a table as well —
`[0x4c7240 + …]` at 0x434da2, indexed by the same slot.

**What is still unread is which slot is playing now** as a number orb could compare — the name at
`+0x5108` is the nearest thing to one — and the stream's own position with it.

**The teardown says which of these are handles and not memory.** 0x45c4b0 releases `[this+0x608]`
through its vtable's `+0x48` and then its `+0x8` — a COM object, so `+0x608` is the interface itself —
deletes the stream at `+0x5208` through its own virtual destructor, and takes the wrapper at `+0x610`
down through 0x45c480.
