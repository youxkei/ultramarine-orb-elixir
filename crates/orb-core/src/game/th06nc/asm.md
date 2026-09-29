# th06nc, as read off its exe

What is written here was read off `th06nc.exe` of **md5 `fe47d9c26828e4671a2d2e4bf00af5a4`**, 5,857,792
bytes, which is 「東方紅魔郷: New Classic」 — Steam app id 4659620, the other 紅魔郷 sold beside th06c (app
id 4924840) — as Steam installed it on 2026-09-25, in place of ver 1.03 of md5
`b0cc28ec904e9efce2439ad857c938b5` (5,539,328 bytes) this document was first read off. *What the update
changed* below is the difference between the two, and every address here has been moved across to this
build.

**It is built on th06c's engine and is another game.** The host layer is the same code: the keyboard read
is a function of the same `0x26f` bytes with its own statics, the Steam startup and the chain walks read
the same instruction for instruction, and the same names are asked of `GetProcAddress`. The game on top
is rewritten — its front end has thirty-two states and leaves the chain
for a run, its bullet is another size, its player is hit as a circle — so nothing of `th06c/asm.md`'s
addresses holds here and what does hold is the *shape*, which is what each reading below was found by.

To read it again:

```sh
objdump -d -M intel th06nc.exe > th06nc.asm     # the whole of .text, once
objdump -p th06nc.exe                          # the DLLs it imports, and where each one's IAT starts
objdump -h th06nc.exe                          # the sections and where they are loaded
strings -a -t x th06nc.exe                     # the names it resolves at runtime, and its own file names
```

Its image base is `0x140000000` and every address below is a virtual address at that base. **And that is
not where it runs**: the exe carries a `.reloc` and is built with ASLR on — a launch came up at
`0x7ff7faf90000` — so an address here is an address in a running process only once the difference
between the base the loader used and `0x140000000` has been added to it. Orb reads the base it was loaded
at and says so in its log's third line.

## What kind of build it is

| | |
| --- | --- |
| the exe | `PE32+ executable (GUI) x86-64`, seven sections |
| `.text` | `0x140001000`, `0x2f3b14` bytes |
| `.rdata` | `0x1402f5000`, `0x771fa` |
| `.data` | `0x14036d000`, `0x1e8000` in the file and `0x908884` loaded; its statics run on to `.pdata` at `0x140c76000` |
| `.pdata` | `0x140c76000`, `0xf444` — 5,211 `RUNTIME_FUNCTION`s, from file offset `0x553200` |
| `.fptable` | `0x140c86000`, `0x100` |
| `.rsrc` | `0x140c87000`, `0x322b0` |
| `.reloc` | `0x140cba000`, `0x1260` |
| what it was built with | MSVC with its CRT linked into the exe: `steam_api64.dll`, `KERNEL32.dll`, `USER32.dll` and `ole32.dll` are the whole of its imports, and `malloc`, `free`, `realloc`, `fopen` and the rest are functions of `.text` — see *What the update changed* |
| what it is sold through | `steam_api64.dll`, imported outright — the same nine names th06c imports |

**Its files are under `data/`**: `data/th06ST.dat`, `th06CM`, `th06ED`, `th06FN`, `th06IN`, `th06MD`,
`th06TL`, `data/bgm/th06_01.opus` and its neighbours where th06c has `bgm/th06_01.mid`, `data/msg1.dat`
to `msg7.dat`, `data/title/th06logo.png`; and beside the exe `th06.cfg` (44 bytes, where th06c's is 28),
`th06.env`, `score.dat` and `replay/th6_NN.rpy`.

## What it imports, and what it asks for by name instead

`USER32.dll` gives it eleven names, its IAT at `0x1402f5368`: `GetWindowRect`, `MonitorFromWindow`,
`SetWindowPos`, `AdjustWindowRectEx`, `EnumDisplayMonitors`, `GetAsyncKeyState`, `CallNextHookEx`,
`GetKeyboardLayout`, `MessageBoxA`, `MapVirtualKeyExW`, `GetSystemMetrics`.

**And no `DialogBoxIndirectParamW`**, nor `EndDialog`, `CheckRadioButton` or `IsDlgButtonChecked`, and no
string naming a dialog control: **this exe puts no startup dialog up**, where th06c puts one up every
launch. A launch with no `th06.cfg` beside the exe came up in a window of `3862x2216` — the whole of a
monitor — and wrote one.

Everything else is a string `GetProcAddress` is given — the import at `0x1402f5080`, called from 351
places, with `LoadLibraryW` at `0x1402f5070` and `LoadLibraryA` at `0x1402f5098`. The names orb answers
are all there: `GetKeyboardState` (`0x1402f8f58`), `CreateWindowExW` (`0x1402f8740`), `MoveWindow`
(`0x1402f8840`), `XInputGetState` (`0x1402f72a8`), `CreateDXGIFactory2` (`0x140303820`) with
`CreateDXGIFactory1` and `CreateDXGIFactory` beside it, and `D3D11CreateDevice` (`0x140303a50`). The
renderer is D3D11 — `D3DCompiler_33.dll` to `_47.dll` are named — the effects are `DSound.DLL`'s and the
music `Mfplat.dll`'s. A launch said so: `the swapchain this game makes will be orb's to present through`.

## Steam

`0x140068fb0` is the startup that reaches it: `SteamAPI_RestartAppIfNecessary` at `0x140068fbe`,
`SteamInternal_SteamAPI_Init` at `0x140068fd1` with `test eax,eax; je` after it, and the interfaces
through `SteamInternal_ContextInit` at `0x1400690ad` and `0x1400690c4` — each followed by `mov rcx,[rax];
mov rax,[rcx]; call` — with `[vtable+0x20]` at `0x1400690d0` being the language, compared against the
strings at `0x140351388` onwards. The same shape th06c's has at `0x140048790`, so `orb::steam` answers
it unchanged: a launch with no Steam client running reached its title. The import slots are
`0x1402f53f0`–`0x1402f5428`, and `SteamAPI_RunCallbacks` is called once round the loop at `0x1400475b7`.

**And `0x1400693c0(name)` is the whole of the achievements**: the interface at `0x140554a10` asked
through `[0x1402f5428]`, `[vtable+0x30]` called with the name, then the same interface asked again and
`[vtable+0x48]` tail-called — set, then store. **Two names in the exe and three call sites**:
`Achievement01` (`0x140351358`) at `0x14004dc1f`, screen 25's own, where the screen it came from is 14 —
a run started and not a practice — and `Achievement21` (`0x140352080`) at `0x14007280d` and
`0x1400729bb`, the two edges the name entry ends on. So a launch of this game earns two achievements,
both on paths orb drives, and `--without-steam` is what keeps a driven run from doing it.

## The keyboard: one 256-byte state, filled two ways

**`0x1400afef0` is the read**, `0x26f` bytes, the same function th06c has at `0x14008f930` with its own
statics:

| | |
| --- | --- |
| `0x1405a7a70` | **the DirectInput keyboard**: its vtable's `+0x48` is called with `(0x100, 0x14058e3dc)` at `0x1400affc8` and again at `0x1400afff6`, `+0x38` between them at `0x1400affd9`. Nought here sends the read down the other path |
| `0x14058e3dc` | **the 256 bytes the game plays a run off** |
| `0x1400b002c`, `0x1400b0085` | `GetKeyboardState` through the slot at `0x1405a83a8`, into a buffer on the stack; the second path memsets the 256 bytes and fills them through the table below at `0x1400b00c0` |
| `0x1402f6060` | **the table from a virtual key to the index it is kept at**, six bytes an entry, ending at `0xffff`: `0x41 -> 0x1e`, `0x42 -> 0x30`, `0x43 -> 0x2e` — `DIK_A`, `DIK_B`, `DIK_C`, so the buffer is indexed by DirectInput's scan codes |
| `0x14058e3d8` | the timestamp the walk writes, held against `timeGetTime`'s answer through `0x1405a7c60` |
| `0x1405a7a44`, `0x1405a7a84` | the two flags that send the read down the `GetKeyboardState` path |
| `0x1405904e4`–`0x1405904e8` | the five mouse buttons, read out of the same stack buffer |

The read is called from `0x1400abce3` and `0x1400ac55e`.

**The table has no `VK_SHIFT` in it.** Its 110 entries hold `0xa0` at scan `0x2a` and `0xa1` at `0x36` —
`VK_LSHIFT` and `VK_RSHIFT` — and nothing at all for `0x10`, where th06c's table at `0x1402872e0` keeps
`0x10` at that same `0x2a`. So a walk of it looking the focus up under `VK_SHIFT` finds nothing, which is
what `key::FOCUS_IN_THE_GAMES_TABLE` is for. `Enter` has two entries, `0x0d -> 0x1c` and
`0x0d -> 0x9c`, the numpad's; the walk takes the first.

**And between that buffer and the word there is a second copy of it.** `0x1400abc90` calls the read where
`[0x14058e3d0]` says there is a device, and then walks `0x14058e3dc` for `0x100` bytes writing
`(byte >> 7)` into the buffer it was handed — `0x140c6e190`, one byte a key holding 1 or 0. So the 256
bytes orb writes a press into are the 256 the word is built from, one `shr` apart.

**The word itself is assembled out of a block of key bindings**, `0x140c6e07c`–`0x140c6e158`, each byte
a scan code into that copy: `0x140013499` onwards reads the binding, reads
`[0x140c6e190 + that]`, and turns a byte that is set into the bits it carries —
`neg al; sbb ecx,ecx; and ecx,<bits>; or r8d,ecx`, twenty-six of those in a row. The bits are the word's
own: `0x10`, `0x20`, `0x40` and `0x80` for the four directions and the diagonals as pairs of them
(`0x90`, `0x50`, `0xa0`, `0x60`), `0x4` for the focus, `0x100` for the decide, `0x8` for the menu. Which
bindings a launch holds is `th06.cfg`'s, so a laid-out game has to fill that block before the word it
builds means anything.

**And four of those twenty-six carry more than one bit each**, which is what makes the shot and the
decide one key: `0x8101` at `0x14001370f` — the shot's `0x1`, the decide's `0x100` and the
fast-forward's `0x8000` together — `0x202` at `0x1400136ae` and `0x1400136c7`, the bomb's `0x2` with the
cancel's `0x200`, and `0x600` and `0x1020` beside them.

**And the whole of that compose is one function that takes nothing and answers in `eax`.** `0x140013310`
opens `lea r14,[0x140c6e190]` and `call 0x1400abc90` with that buffer in `rcx`, so it makes the read
itself and fills the copy it goes on to walk; nothing is handed to it, the `eax` the supervisor's job
loads at `0x14007b8a6` being the pair's own copy and dead across the call. It ends `jmp 0x140012f00` with
the keyboard's bits in `ecx`, and that function ORs the pad's word into them — `0x1400abd40` for the
state, `0x140012fd3` onwards for the bits — and returns the word in `eax` at `0x14001325a`. So the word
can be composed from wherever the keys stand, by a call.

**Two things it does besides composing**, which is what a second call in one frame costs:
`0x14001336d`–`0x140013488` fills the scan codes at `0x140c6e080` through `0x1402f5368` and `0x1402f5370`
where the keyboard layout at `[0x140c6e180]` is not the one `[0x140abae8c]` was set for, which is once a
launch and then never; and the pad's half steps `[0x140abad70]` at `0x140013014` and `0x140013034`, the
frames a pad's own focus has been held, between nought and `0x10`, firing `0x4` from eight up.

**So the front end's own screens test the cancel first and the pair `0x3` after it**, which is how a
decide is read there. Screen 14 is the case this was read off — `0x14004d52a` takes `0x200` of
`[0x140abae80]` against `[0x140abae84]`'s for the cancel and `0x14004d62e` falls through to `r15d`, held
at 3 by `0x14004c826` and fourteen other sites of that function, for the decide — and a press of the
bomb sets both `0x2` and `0x200`, so what reaches the `0x3` test is the shot alone. `0x14004e351` and
`0x14004e4e3` are the two sites that hold `r15d` at anything else, `r10d` and 1.

**And the word it comes out as is `0x140abae80`**, with the frame before it at `0x140abae84`: the front
end's state 0 clears both, `0x140abae68` and `0x140abae7c` at `0x140049234` as a run starts, the
player's update reads it at `0x14006a0fd` and five more places, and the loop's fast-forward reads `0x8000`
of it at `0x14003d92b`. Its bits are th06 1.02h's: the player's walk at `0x14006a790` onwards tests `0x10`,
`0x20`, `0x40` and the sign bit for up, down, left and right and `0x4` for the focus at `0x14006a7c3`, and
the front end writes `0x100` into it at `0x1400491cc` where a screen is to be skipped.

**The frame before's word is written from this one, once a walk of the calc chain, by the supervisor's own
job**: `0x14007b8a6` reads `[0x140abae80]`, `0x14007b8ac` writes it into `[0x140abae84]`, and only then does
`0x14007b8b2` read the keyboard through `0x140013310` and `0x14007b8b7` write the answer into
`[0x140abae80]`. `[0x140abae68]` is cleared beside it and set again at `0x14007b8da` where the word has
not changed — the repeat, whose whole arithmetic is `0x14007b8c3` onwards: **`[0x140abae7c]` counts the
frames the word has not changed**, `0x14007b8f2` putting it to nought on the frame it does and
`0x14007b8ee` stepping it up on every frame it does not; under thirty (`0x14007b8d1`) nothing fires; at
thirty and past it `0x14007b8d6`'s `test al,0x7` sets `[0x140abae68]` every eighth frame; and
`0x14007b8e4` holds the count itself between thirty and `0x26` so that it keeps firing rather than
running away. **And the front end's own screens read that word beside the edge of the input word** —
`0x1400491e1` in the first table's dispatcher, `0x14004a2eb` at the replay list, `0x14004ab32` and
`0x14004ac92` at the options, and `0x14004bf12`, `0x14004cc45`, `0x14004e968` and `0x14004f9e8` — so a
key held at one of them moves the cursor once, waits half a second and then moves it every eighth frame.
Which is what the recipes' six-frame presses are not: each of those is one edge. Which is what
`Game::swallow_input` rests on: a word of every bit left
in `0x140abae80` is the frame before's on the next frame, so no bit can be set in this one and not in
that, and an edge is a bit that is.

**And that is once a walk of the chain and not once a frame.** The job has no guard around those three
writes: every path through its first half — the track it moves and the state it asks for — falls through
to `0x14007b8a6`. So a frame that walks the chain twice reads the keyboard twice and composes two words,
and the count at `0x140abae7c` and the edge either side of it are counted in walks. **Which is what a
playback is**: `0x14003d93f` calls `0x14003d700`, the same whole chain walk the frame's own
`0x14003d8ef` calls, up to seven more times, and only while `[0x14053d3dc]` says a replay is playing and
`0x8000` is in the word. So a run a person plays is one walk, one read and one word a frame, and a run
being put back is up to eight of each.

**And six places write that word, which is the whole of them** — what
`the_frame_a_scene_of_th06nc_is_built_on` asks about a transition is answered here, two of the six being
one:

| | |
| --- | --- |
| `0x14007b8b7` | the supervisor's own job, the keyboard's answer — above |
| `0x140049197` | **nought**, in the front end's own update, where the byte at `[front end + 0x168d0]` is set: `0x14004918c` compares it with nought and jumps past this where it is |
| `0x1400491cc` | `0x100`, the decide, where the screen is one of `0x6a00`'s bits — 9, 11, 13 and 14 — and `[front end + 0x9638]` is exactly `0xf`: a select going by on its own |
| `0x140049234` | **nought into all four** — this word, the frame before's, `0x140abae68` and (at `0x140049649`'s case) `0x140abae7c` — at a case the front end's own jump table at `0x140049232` reaches |
| `0x140049649` | **and nought into all four again**, at another case of that table, `0x14004963c` having asked the supervisor for state 4 just above it |
| `0x14006d17f` | a playback's own feed: `eax = [0x140abae80] & 0xfffffc00 \| [entry + 4]`, the low ten bits replaced out of the record while the rest stay |

**So a transition does zero it**, and zeroes the frame before's with it — which is what keeps an edge
from being read across one. The ten bits `0x14006d17f` replaces are the run's own buttons, which is the
same mask `0x14006d070` holds a recording against: `and r9d,0x3ff` there against `[rdx+4]`, gated on
`[0x14053d402]`.

## The frame, which `main`'s loop calls

**`0x140046da0` is `main`**, and the loop runs from `0x140047570` to `0x140047613`:

| | |
| --- | --- |
| `0x140047570` | the loop's head: `[0x140abadec]` non-zero leaves it |
| `0x140047585`–`0x1400475b2` | the window's size read (`0x1400bb880`) and handed to `0x14007c890` on `0x140c6db90`; `0x1400be870` off the byte at `0x140c6e020` |
| `0x1400475b7` | `SteamAPI_RunCallbacks`, and the overlay asked through the interface at `0x140554a10` |
| `0x1400475fb` | **the frame**, `0x14003d8b0`, whose answer goes into `esi`: nought goes on to the pump, anything else leaves the loop at `0x140047613` |
| `0x140047606` | `0x1400c33b0`, the message pump: nought carries the loop on, anything else leaves it at `0x140047613` |

**And the frame is `0x14003d8b0`**, which takes nothing and answers `eax`:

| | |
| --- | --- |
| `0x14003d8e5` | `0x14007ccb0` on `0x140c6db90` — the renderer's own object — with nought and the float at `0x140352a90` |
| `0x14003d8ea` | **the calc chain walked**: `lea rcx,[rsp+0x30]; call 0x14003d700`, the answer a byte and what the frame leaves with written through the pointer |
| `0x14003d910`–`0x14003d95f` | and walked **again up to seven more times** while `[0x14053d3dc]` is set, `[0x14053d400]` is clear, the state asked for at `[0x140c6dfbc]` is 2 — a run — and `[0x140abae80]` has `0x8000`: a replay being fast-forwarded, each extra walk counted into `0x1405542e0` |
| `0x14003d966`–`0x14003d9f8` | **the sound's catch-up**: where the count is above nought and a track is playing (`0x1405542bc` not `-1`, `0x1405542cc` set), `0x1400c66f0` answers the position, `0x320` samples a walk are added — 48,000 a second — held inside the loop at `0x1405542d0`..`0x1405542d8` where `0x1405542cd` says the track loops, or under `0x1400c6c70`'s length where it does not, and `0x1400cb610(seek, track)` seeks |
| `0x14003d9fd`–`0x14003dafc` | **the draw**, inline — see *The draw* below |
| `0x14003db06` | `0x14003db50`, which puts the frame on the screen and then waits for the deadline |

Where the calc walk answered non-zero, `0x14003db1d` answers what it left at `[rsp+0x30]` with nothing
drawn — and every non-zero answer leaves a non-zero, `0x14003d7a1` and `0x14003d78b` writing 1 and 2 —
where a frame that was drawn answers nought.

**Its prologue is `48 89 5c 24 08 48 89 6c 24 10 48 89 74 24 18`**, three `mov`s of fifteen bytes and
nothing rip-relative, which is where orb's own frame stands: `runtime::render` in its place, calling each
piece of it itself — see `Th06nc`'s `THE_FRAME`.

**Which makes every row of the frame a reading of the exe run on its own and not of an orb launch**: the
fast-forward's extra walks, the sound catch-up, the draw and the present are all inside the function orb
stands in for. orb does the camera, the catch-up and the draw itself, and it does *not* do the
fast-forward — what multiplies a frame's updates under orb is `runtime::render`'s own repeats. Worth
having in mind before anything about a playback's walks is read off the table above.

### The chain walk

**`0x14003d700` is the calc chain's**, `0xe9` bytes: `lea rbp,[0x14053c090]`, then each element's `+0x8`
called with its `+0x38` in `rcx` and `+0x28` followed. What the callback answers, off the chain of
compares at `0x14003d73a`: **2** calls it again, **0** takes it out through `0x140012e10` with the element
in `rdx`, **3**, **4** and **5** end the walk and **6** starts it over from the head — and **1**, like
every other answer the walk has no case for, moves on to the next element and counts in `edi` as an
element that ran. Each of the four that ends the walk writes `0x140c6db90`'s first byte to nought and
calls `0x1400781d0`, which flushes the frame's queued sound effects out of `0x1405542c0`, before it
answers. The walk returns `al` non-zero to leave the frame, and writes 1 or 2 through `rcx` for what it
leaves with — **and a chain no element of ran at all leaves the frame**: `0x14003d76d` tests `edi` and
`0x14003d771` writes 1 with `al` set where it is nought.

**And the draw chain's is inline in the frame** at `0x14003da73`, `lea rdi,[0x14053c050]` — the same
walk, 0 taking the element out at `0x14003db0f`, and the renderer's batch of sprites flushed after every
job before its answer is acted on (`0x14003da97`..`0x14003dace`, see *The draw*).

`0x140012e10` takes an element out of either chain: it searches `0x14053c090`'s links and then
`0x14053c050`'s for `rdx`, unlinks it, calls its `+0x18` on its `+0x38` and frees the `0x40` bytes where
`+0x2`'s low bit says they were allocated.

`ChainElem` is the `0x40` bytes th06c's is — `+0x8` the callback, `+0x10` the one called as it is
registered, `+0x18` as it is taken out, `+0x20` and `+0x28` the links, `+0x30` the element's own
address, `+0x38` what the callbacks are called on — read off the player's registration at `0x14006952e`
onwards and the front end's at `0x1400573bd`.

### The draw

`0x14003d9fd`..`0x14003dafc` inside the frame — the build before had it as a function of its own,
`0x2f9` bytes: `0x1400019d0`, which puts the screenshot's two flags at `0x140abae8a` and `0x140abae89`
down; `0x14007ccb0` on `0x140c6db90` with nought and the float again; `0x1400855f0` into four ints on the
stack, which is the viewport as it stands; `0x140097160` with the four bytes at `0x1405547f0` — `+2`, `+1`,
`+0` and `+3` in that order — the viewport the clear is made over; `0x140081330(0)`, the clear;
`0x140097160` with the four ints, the viewport put back; the draw chain walked; and then the screenshot's
own path, now a function of its own at `0x140001c70`, before `-1` is written into
`[0x140abac30]+0x20f80`. Orb makes each of those calls itself, in that order — see `Th06nc`'s
`walks_the_draw`.

**Every sprite a draw job makes is queued into the renderer's batch and drawn only when it is
flushed.** The renderer is `0x37070` bytes where the build before's was `0x27108` — `mov ecx,0x37070` at
`0x140047521` — and the batch is its end: the sprites at `+0x27104`, their count at `+0x37064`, and a word
the flush is handed at `+0x37068`. The walk flushes after every job, where the count is above nought, with
`0x140083eb0(renderer + 0x27104, count, 4, [renderer + 0x37068], 1)` and zeroes the count; the camera at
`0x14007ccb0` flushes it too before it moves anything, and 99 instructions of the exe touch the count.

### The timing

`0x14003db50`, `0x41b` bytes: the window's size again (`0x140085c40`), the viewport's four ints at
`0x140c6e05c`–`0x140c6e068`, and where the viewport does not cover the window the bars around it drawn
through `0x140097c90`, `0x140097ae0`, `0x1400a3880`, `0x140082a30` and `0x140097590`; then
`0x140097150`, which is `jmp 0x140094620` — the present, through the swapchain orb took; then the clock
through `0x140c6e300`, the deadline out of the period at `0x140c6e320`, the base at `0x140c6e348` and the
count at `0x140c6e350` — `0x14003dccd` starts base and count over where the clock is more than a period
past `base + period × (count + 1)` — a sleep through `0x140c6e310` while more than `0x5f`-averaged slack
is left, and a `pause` spin at `0x14003ddb4` until the deadline. After it the fps into `0x140c6e3f0`
through `0x14003a060` and `%.02lffps` at `0x1403526f8`.

**Orb calls it whole with the base and the count written to nought first**, which puts every clock a
period past the deadline and starts the limiter over without a wait — see `Th06nc::end_drawing`, which
has why that and not the pieces.

## The managers, and what registers them

| registered at | the object | the frame's callback |
| --- | --- | --- |
| `0x1400694d0` onwards | `0x140549ff0`, `0x7854` bytes zeroed, and `0x14055184c`, `0x2a5c` — **the player**, its element kept at `0x140551898` | `0x140069f30`, and `0x14006b850` for the draw chain |
| `0x14003ca43` onwards | `0x140436ef0`, `0xf5010` bytes zeroed — **the bullets and the lasers**, with `0x14052bf08` (`0x10108`) beside it; the element is the static at `0x140436eb0`, priority `0xb` | `0x140010fa0` |
| `0x140038ae9` onwards | `0x140aee0b0` — **the enemies**, its element the static at `0x14053c190`, priority 9 | `0x140038cd0` |
| | `0x140bfb2f0` — **the items**, walked by `0x140044240` from inside the bullets' update at `0x140011017` | |
| `0x1400573bd` onwards | **the front end**, allocated, its element kept at `0x140c53468` and the front end at that element's `+0x38` | `0x1400490a0`, a wrapper that calls `0x140049140` once — or up to `0x258` times while `[front end+0x168d0]` is set |
| `0x14003c0a0` | **the stage**, the static `0x14053ca70` for the calc chain at priority 4 and `0x14053ca30` for the draw chain at priority 3, both on the game manager `0x14053cab0` — see *The stage's two seams* | `0x14003baa0`, and `0x14003c080` for the draw chain |
| `0x14007c070`, from `main` at `0x14004754d` | **the supervisor**, allocated, on the game's own object `0x140c6db90` | `0x14007b7f0`, with `0x14007c230` and `0x14007c6d0` as the callbacks it is registered and taken out by |

**The front end leaves the chain for the whole of a run.** A launch's `chain:` lines read four jobs
with `0x1400490a0` among them through the front end, three without it on the frame the run's lives and
bombs were written, and then eleven — `0x14007b7f0 0x140009800 0x140077250 0x14003baa0 0x140079360
0x140069f30 0x140038cd0 0x14002b130 0x140010fa0 0x14003e530 0x14006d070` — for the stage. So the
element at `0x140c53468` points at freed memory during a run — read through it, the state came back as
the float `1.0` — and the front end is found by walking the chain for its callback, and a stage is the
player's job being in the chain and the stage being one to seven.

**And above both of them there is a supervisor**, which had been read as nothing: `0x14007c070`,
called from `main` at `0x14004754d`, allocates a `0x40`-byte element, puts `0x14007b7f0` at its `+0x8`,
`0x14007c230` at its `+0x10` and `0x14007c6d0` at its `+0x18`, and the game's own object
**`0x140c6db90`** at its `+0x38` — the same object `th06.cfg`'s 44 bytes sit at `+0x434` of. It writes
`[0x140c6dfbc]` to `-1` first.

So `0x14007b7f0` is the job that drives everything, and it moves between states out of **three fields of
that object**:

| | |
| --- | --- |
| `+0x428` | **the state it has acted on**, read at `0x14007b8fa`, and the table at `0x14007be28` — twelve entries — is jumped through on it, `0x14007b915` refusing anything past `0xb` |
| `+0x42c` | **and the state wanted**, read at `0x14007b901`, which each of those cases then switches on through the tables at `0x14007be58` (ten entries, indexed by it less two) and `0x14007be80` (twelve, less one) |
| `+0x430` | the state acted on before this one, written `[+0x428]` at `0x14007b90f` |

**Which of the two is which is `0x14007baf1`, the exit every case falls to**: `mov eax,[rdi+0x42c]; mov
[rdi+0x428],eax` — the acted-on state caught up to the wanted one — with `[rdi+0x290]` stepped up
beside it. And while the two agree the job does nothing, `0x14007b909` leaving straight for that exit.
So what it is is a machine of transitions between a pair of states rather than of one state's own cases.

The four states a run needs, read off the cases:

| | |
| --- | --- |
| **1, the front end** | the outer table's entry for nought is `0x14007b9b3`, which writes 1 into `+0x42c` and calls **`0x140057190`, which builds the front end** — `0x1400573bd`'s registration is inside it |
| **2, a run** | the front end's own inner table starts at `0x14007b94c`, and it is indexed by `+0x42c` less two — so wanting two from the front end calls `0x14003c0a0`, the stage's own chain. `0x14007bac7` and `0x14007bcaf` are that function's other two callers, and `0x14007bc7e`–`0x14007bd0b` writes the numbers a stage starts with just before the last |
| **7, the screen a run ends at** | a run's own inner table's seventh entry is `0x14007b9e3`: it waits for `[0x140c6e378]`'s `+0x34` to read 2, takes the stage's two elements out, takes `0xbb48` bytes off `malloc`, builds the screen with `0x14006f5d0` and falls into the tail at `0x14007bdc3` — 1 into the screen's `+0x9e99` and, by `[0x14053d404]`, nine or seventeen into its `+0x9e94`. `0x14007bd81`'s third branch, the case for leaving state 10 with 7 wanted, is the other way in and shares that tail |
| **8, a run over** | a run's own inner table is indexed by `+0x42c` less one, and its eighth entry is `0x14007bb13`: the stage's two elements `0x14053ca70` and `0x14053ca30` taken out through `0x140012e10`, the score file handed to `0x14006db10` where `[0x140abae58]` holds one — **which writes nothing**, the null record reaching the teardown, see *The score file* — `0x14053d3dc` and `0x14053d405` cleared, and 1 asked for itself. **It builds no screen**, which is what makes it the state a run is *given up* into where 7 is the state one finishes into |

**So giving a run up is 8 asked for and not 1 written**: wanting 1 out of 2 is `0x14007bae1`, which
calls `0x14007b770` and falls to the exit — the stage's elements left in the chain.

**And the run's own start is `0x14004f320`**, called once in the exe — at `0x14004dc06`, from the shot
select's decide, with the front end in `rcx`. What it does, in order:

- reads `[front end + 0x168d1]` into `0x14053d404`, the byte that has a run asked for no name;
- takes the starting stock out of `th06.cfg` — `[0x140c6dfd4]` into the lives and `[0x140c6dfd5]` into
  the bombs — and `[0x140c6dfdb]` into `0x14053d414`, the mode byte, through a `setne`;
- writes `0x302` over the lives and bombs where the difficulty is 4 or that mode byte is set, and `2`
  over the lives alone otherwise;
- **asks the supervisor for 2** at `0x14004f38b`, and clears `0x14053d3dc`;
- **stops the track**: `0x1400ceb30` and `0x140081740` on `[0x1405542bc]`, then `-1` into it and nought
  into `0x1405542cc` and `0x1405542b8`;
- clears `0x140c6de2c` and `0x1405542e0`, the extra walks.

Which is why a run orb starts itself has to *call* it rather than write what it writes — th06c's own run
start is `0x14002d4c0` and the one thing a copy of it missed there was the track stopped and put to
`-1`. And the three numbers a run is: the difficulty screen writes `0x14053d410` at `0x14004caee` and
`0x14004cafd`, the character screen `0x14053cad0` at `0x14004cd28`, and the shot screen `0x14053cad1`
at `0x14004d21d` and `0x14004d32a`.

And two more of the front end's: `0x14007b966` builds the screen a run ends at, `0xbb48` bytes off
`malloc` through `0x14006f5d0` — `0x14007bd98` is where a run's own is built instead — and
`0x14007b996` and `0x14007b9a0` reach `0x140059900` and `0x140013bf0`.

### The two tables whole

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for name, table, first, n in (("left", 0x14007be28, 0, 12), ("from the front end", 0x14007be58, 2, 10), ("from a run", 0x14007be80, 1, 12)):
    for i in range(n):
        off = struct.unpack_from("<I", d, at(table)+4*i)[0]
        print(f"{name} {i+first:2d} 0x{0x140000000+off:x}")'
```

| the state being left | | the state wanted | |
| --- | --- | --- | --- |
| 0 | `0x14007b9b3` — writes 1 into `+0x42c` and builds the front end | 2 | `0x14007b94c` — `call 0x14003c0a0`, the stage registered |
| 1 | `0x14007b931` — **the inner dispatch**, `+0x42c` less two through the second table | 3 | the exit |
| 2 | `0x14007b9c9` | 4 | `0x14007be15` |
| 3, 4, 5, 8 | the exit at `0x14007baf1`, so those four have no tear-down of their own | 5 | `0x14007b956` — `mov eax,5; ret`, which is **the walk ended and the frame left**: 5 is the game going |
| 6 | `0x14007b9aa` | 6 | `0x14007b966` — the screen a run ends at built, `0xbb48` off `malloc`, which is where `0x14007bdf2` registers it at priority 13 |
| 7 | `0x14007bd0d` | 7, 8 | the exit |
| 9, 11 | `0x14007bd6a` | 9 | `0x14007b996` |
| 10 | `0x14007bd81` | 10 | `0x14007bb79` |
| | | 11 | `0x14007b9a0` |

And the run's own inner table, `0x14007be80`, indexed by `+0x42c` less one:

| the state wanted out of a run | |
| --- | --- |
| 1 | `0x14007bae1` — **the front end written outright**, `0x14007b770` and then the exit, the stage's two elements left in the chain |
| 3 | `0x14007baaf` — **the next stage**: `0x14053ca70` and `0x14053ca30` taken out through `0x140012e10` and `0x14003c0a0` called again |
| 4 | `0x14007be15` |
| 7 | `0x14007b9e3` |
| 8 | `0x14007bb13` — **the run over**, the state a run is given up into |
| 10 | `0x14007bb66` — **the ending**, which is what asks for 10: `0x14004148d` writes `0xa` where a run has finished, the difficulty is not four and `[0x14053d3dc]` says no replay is playing, and this branch takes the stage's two elements out and calls `0x14002c1d0` at `0x14007bb91` — the ending's own registration — where `[r8+0x34]` reads 2. **The Extra asks for 7 instead** at `0x14004147e`, and a replay for 8 |
| 12 | `0x14007bc27` |
| 2, 5, 6, 9, 11 | the exit at `0x14007baf1` |

**And the job's first job is the music's fade**, before any of that: `0x14007b804` leaves where
`[0x1405542bc]` is `-1` or `[0x1405542b8]` is clear, and otherwise counts `[0x1405542b0]` down — at
nought it clears `0x1405542b8`, calls `0x1400ceb30` and `0x140081740` on the track, writes `-1` into
`[0x1405542bc]` and clears `[0x1405542cc]` and `[0x1405542e0]`; above nought it seeks instead, through
`0x1400c4120` or `0x1400cc4b0` by the byte at `0x140c6dfd6`. So `0x1405542b0` is the fade's own frames
left and `0x1405542b8` is what says one is running, which are two of the statics
`Game::audio_state` copies.

**And `0x140c6dfbc` is that `+0x42c`**, which is what every `[0x140c6dfbc]` written down elsewhere here
turns out to be: `0x140c6db90 + 0x42c`. So it is not a static beside the supervisor — it *is* the state
asked for, and every write to it is an ask:

| | |
| --- | --- |
| `0x14007c081` | `-1` at the boot, with `+0x428` written nought beside it — which is what makes the job run its case for nought and build the front end |
| `0x14000b482` | **8 from the pause menu's own job**, which is a run given up; `0x14003bd5a` writes 8 too |
| `0x14000bcae`, `0x14000bdf7`, `0x14000c3f4`, `0x14002d8c6` | 7 |
| `0x14000cfb9` | `0xc` |
| `0x140013e9c`, `0x14003ceea` | 1, the front end |

**And 3 is the next stage of a run**: `0x14003c27c` takes the stage's build down its stage-following
path — `0x14003c78d`, which raises the drawn score to the counted one and counts the stage on — where
the state asked for is 3, and `0x14003bb01` reads 2 there for a stage already running.

**And it is the front end and not the game manager**, which is what it had been written down as here:
its thirty-two states are this game's *screens* and none of them is a stage. What a run is made of is
the static at `0x14053cab0`, which stays where it is — see *The game manager* below. The same split
th06c has, whose front end is `0x140b5c140` and whose game manager is the static `0x1403a33a0`.

**And the ending is a job too.** The frame the sixth stage's jobs left, a cleared run's `chain:` line
read five: the three that are always there, `0x14006d070` of the stage's, and **`0x14002c450`**, which is
registered at `0x14002c2c6` by `0x14002c1d0` — called from `0x14007bb91`, inside the job at
`0x14007b7f0` that a stage ends into — and reads the input word at `0x14002c4a1` the way a screen that
is pressed through does. The front end is not back while it runs, so `State::in_ending` is the chain
holding this callback, and `run: the ending is running` is the line a clear is read by.

**And the screen a run ends at is a job as well**: `0x14006fa70` arrived on the frame the ending's job
left, stood with `0x14006d070` and then alone, and the front end came back after it — at state 2, the
title — with `score.dat` and `replay/th6_01.rpy` written beside the exe. A run pressed through with
`decide` reaches the title that way, and what the screen holds is *The entry a name is typed into* below.

**The Extra has no ending.** Its stage's jobs left straight into `0x14006fa70` on both of two launches —
the one that beat Flandre and the one that died under her — so what tells them apart is her life as
she left: `5` where her last attack was emptied, the field then standing empty for a thousand frames
before the jobs went; `1000` where the run died inside her survival card, which holds her life at that
for its whole length. Her attacks read off the `run: the boss has` lines: the midboss at `6000` and
`1800`, then `15000`, `1800`, `8000`, `1200`, `7182`, `1900`, `7984`, `2400`, `9689`, `1400`, `9359`,
`2200`, `10000`, `1520`, `9382`, `1800`, `1000` held for `5,000` frames, `6000`, `15000`, `3900`,
`4000`, and `6000` last.

**Everything is held** is `0x14053d409`: read by the bullets (`0x140010faf`), the enemies
(`0x140039a7c`), the player (`0x140069f3d`) and four others, written at `0x140034d3d`.

## The front end's states, which are this game's screens

**The state is `[front end + 0x168b0]`**, and its cases:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(32): print(f"state {i:2d} 0x{0x140000000+struct.unpack_from("<I", d, at(0x14004e848)+4*i)[0]:x}")'
```

— which says states 15 to 20 and 26 share `0x140049f38`, and 28 to 31 `0x14004e32f`. **Which state is
which screen was read off launches' `run:` lines**, pressing at each:

| | |
| --- | --- |
| 1 | the logo, which goes to 2 on its own |
| 2 | the title, and a decide opens the menu |
| 3 | **the main menu**, ten items — see the table below. 0 starts a run, 1 the Extra — skipped while it is locked, a fresh launch's tap of down going 0, 2, 3 … 9, and offered once a run has been cleared. The Extra's walk is the same screens as a run's with the difficulty already 4 at screen 9 |
| 8 | goes by on its own after item 0, and after items 1 and 2 too — its case is where `0x14004c278` loads the banner saying which of the four kinds of run is starting |
| 9 | **the difficulty**, four items — `0x14004c872` onwards compares the cursor with 0 to 3 — and a launch that took item 1 played on difficulty 1 |
| 11 | the character |
| 13 | the shot |
| 14, 24, 25 | **each takes a press**: a launch that pressed nothing at 14 sat there for 1,800 frames, and one that pressed nothing at 24 the same |
| 24, 14 | **and a cancel at each is the way back one screen**, measured by `the-cancel-at-the-last-screen.yaml`: a tap at 24 read `front end: screen 14` and a second tap read `13`. Which is what orb holds that key back for at 24 — the question about picking a run up stands there, and a cancel that reached the screen underneath would walk somebody answering 「どちらでもない」 back out of the run they asked for. See `the_run_of_th06nc_left_behind`, and `fake::th06nc`, whose two arms for it this launch put in |
| 14 | **and it is the mode select**, which a picture settled: 「開始するモードを選択してください」 over two items, **標準モード** and **チャレンジモード**, with a line of description under each. Its cursor is `[front end + 0x28]` and walks 0 and 1 — and a launch that decided the second started a run whose HUD reads **`Challenge Mode`** over `NORMAL`. So this is the screen behind `0x14053d414`, the byte `0x140075137` picks the name `Standard` or `Challenge` by. **Its case is `0x14004d44d` to `0x14004d6c2` and writes the choice on both of its edges** — the cancel's going back to 13, the decide's on to 24 — see *`0x14053d414`, the byte that changes what a run starts with* |
| 25 | **and it is the last screen before the stage**, which is what `Th06nc::run_pointed_at` answers on: `0x14004dbe9` waits for `[0x140c6e378]`'s `+0x34` to reach 2, `0x14004dc06` calls `0x14004f320` — the numbers a run starts with, taken out of `th06.cfg` — and where `[front end + 0x168cc]`, the screen it came from, is 14, `0x14004dc1f` sets the `Achievement01` achievement and the case answers nought at `0x14004e810`, which is the front end leaving the chain |
| 22 | **the screen a practice walks through and a plain run does not**, which is where a stage practised on its own is chosen: `orb-real`'s `a_stage_practised_on_its_own` read `14, 22, 24, 25` off the item-2 walk where every other case reads `14, 24, 25`, and the run that came out of it was stage 1 with the cursor still on item 0 |
| 16, 17, 18 | **the three the front end walks when a run is given up**, in that order, **16 and 17 within a frame or two and 18 where it comes to rest** — timed by `the-front-end-after-a-run.yaml`: `front end: screen 16` 16ms after `leave: true`, 17 one frame after 16, 18 seven frames after 17, and then 21,600 frames on 18 with nothing pressed and no screen after it. They are the **attract cycle** rather than a way back to the menu: **18 is the replay list**, `0x14004a2a5`, which waits forty frames, moves a cursor on `0x40` and `0x80` — left and right — through the entries at `+0x34 + n*0x200` that state 16 formats `replay/th6_%.2d.rpy` into, and whose decide calls `0x14006c670` and then `0x14004ede0`, the replay's own start. So a launch that taps the **decide** there starts a replay instead of getting anywhere, and what leaves it is the cancel — which puts the main menu up with its cursor on the item that ends the game |
| 29, 30, 31 | **and the rest of the Spell Practice's own walk**, which a launch read as `27, 28, 29, 30, 31` and then a scene: `run: stage 1 on difficulty 1` with the front end out of the chain. Their cases are `0x14004e32f`, shared with 28. **29 is the card list** — a picture, `screen29-spell-practice.png`, reads `Spell Practice` over a column of card rows with the boss's name beside them and 博麗霊夢 with her shot on the right — and **a decide on a row with no card behind it does nothing**: two launches that only pressed the decide there stood on 29 for thousands of frames, and one that walked the cursor down a row at a time between decides went through to the scene. 30 is where the difficulty arrives, `difficulty 0` on 29 and `1` on 31 |
| 27, 28 | **the two the Spell Practice walks**, which the main menu's item 3 opens: a launch pressed it and read `front end: screen 27` and then `28`, and a picture of 28 reads **`Spell Practice`** over a list of the cards stage by stage. The run it starts is played under `0x14053d405`, the scene's own byte |
| 20, 26 | **and the two a replay the front end started runs through**: a launch whose recipe tapped the decide at 18 read `screen 20`, then `26`, then the front end out of the chain and a playback running stage 1 and stage 2 for three minutes with orb keeping nothing of it — `resume: stage N of a run orb is not keeping`. `leave: true` gives that playback up the way it gives a run up, and the cycle starts again at 16. **And it is a replay and not an attract demo — this build plays none**, which two launches that pressed nothing say: a minute of sitting on 18 brought neither 20 nor a playback, and `the-front-end-after-a-run.yaml`'s 21,600 frames on it wrote not one `resume: frame N played back` line, with `scene=18->-1` and the left run's own `frames=9307` standing still throughout. Every launch that tapped the decide there got one instead. So 18's own decide — `0x14006c670` and then `0x14004ede0` — is the whole of how a playback starts, which is what lets `Th06nc::in_a_run` carry no demo term and a judging pass over this game step at all |
| — | and then the front end leaves the chain and the stage's jobs arrive with `stage 1` |

**And nothing writes that state while a run is on**: the field keeps the screen the run was started
from for the whole of it. `the-front-end-after-a-run.yaml` wrote `front end: screen 25` and then nothing
at all until `16` after the `leave`, a whole run later, and `the_front_end_of_th06nc_walked` asserts the
same field still reading 25 with a stage running.

**And the two ways back from a run are two different screens**, which two launches say between them: a run
**given up** leaves the front end on **16** and the attract cycle, and a run that **finished** — one taken
through the screen a run ends at — comes back on **2, the title**. `the-offer-written-away.yaml` read
`result: the screen a run ends at is down` and then `front end: screen 2` a quarter of a second later with
no 16 between them, where `the-front-end-after-a-run.yaml`'s `leave: true` read 16 sixteen milliseconds
after the give-up. So what the front end comes up on says which kind of end it was.

### The main menu's ten items

**Its decide is edge-checked at `0x140049aa5`** — `[0x140abae80]` against `[0x140abae84]`, both masked —
and the item is then jumped to through the table at `0x14004e8c8`, ten entries bounded by
`cmp eax,0x9; ja` at `0x140049ad5`:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(10):
    off = struct.unpack_from("<I", d, at(0x14004e8c8)+4*i)[0]
    print(f"item {i} 0x{0x140000000+off:x}")'
```

| | |
| --- | --- |
| 0 | `0x140049af2` — **a run**: `+0x168d1` cleared and the `0xad` entries at `+0x96ca` filled with 4, `0x120` apart |
| 1 | `0x140049b25` — **the Extra**, which leaves at `0x140049b96` unless one of `0x140549d51`, `0x140549d52`, `0x140549d53` — the clear records at `+0xd2a1` onwards — is `0x63` |
| 2 | `0x140049cbc` — **the one whose run is asked for no name**: the only item that writes `+0x168d1` to 1, which the run's own start copies into `0x14053d404` and `0x14007bdde` then reads to open the way out instead of the name entry. `+0x168d3` cleared beside it |
| 3 | `0x140049d28` — `+0x168d3` to 1 and `+0x168d1` cleared, which is the other way round from item 2. **And that byte only ever picks a picture**: its six readers are all inside the front end's own update, none of them writes a static, and `0x140053c48` is the one write of nought to it from outside — so unlike `+0x168d1` there is nothing of it for a run to read. **The item is the Spell Practice**, which a picture settled: it walks `front end: screen 27` and then `28`, and 28 reads **`Spell Practice`** over a list of the cards stage by stage, with `SPELL PRACTICE` across the bottom. So the byte a run of it is played under is `0x14053d405` — the scene's own, see *The card records* and `Th06nc::a_run_is_practice`, which answers `practice` off it |
| 4 | `0x140049d8d` — **the replay list**: the table filled, then `0x140049db0` writes **`0xf`** into the state and `0x140049dba` clears `+0x168d1`. State 15's own case writes 16, and 16 is the list — see *The second table, and the replay list behind it*. **The one item besides the three that start a run that orb asks over**: `Th06nc::menu_pointed_at` answers `Menu::Replays` here and the press is held back, so a rewindable run gets orb's own four-deep list in place of state 16 |
| 5 | `0x140049dc3` — the table filled, then `0x140049de2` writes **`0xc`** into the state; state 12's case `0x14004d92c` asks the supervisor for **6**. **And it is 夢の記録**, which a picture settled — `menu-item5-screen12.png` beside the copy: `夢の記録` over `Result of Score and Spell Card` with `SCORE & SPELLCARD` across the bottom, a row apiece for `EASY`, `NORMAL`, `HARD`, `LUNATIC` and `EXTRA`, one more row and `QUIT`. A decide on a row opens that difficulty's own ranking, ten rows of a name and a score |
| 6, 7 | `0x140049df1`, `0x140049e35` — the front end left in state `0x15` and `0x17`. **A launch pressed both**: item 6 came up as `front end: screen 21` with the job `0x140059bf0` in the chain and item 7 as `screen 23` with `0x140013e70`, and the bomb button — this game's own cancel, the `0x200` its screens test — came back from each. Neither job is the screen a run ends at, so neither item is the ranking. **And item 7 is the credits**, `menu-item7-screen23.png`: `CREDIT` across the bottom and a page counter reading `1 - 2 / 14` |
| 8 | `0x140049e75` — state 4, **the options**, which a picture names rather than elimination — `menu-item8-options.png`: `OPTION` across the bottom over nine rows, `Player` 1 to 5, `Bomb` 0 to 3, `BGM Vol` and `SE Vol` 0 to 9, **楽曲モード** with `NewClassic` and `Classic`, 画面設定, コントローラ設定, 設定をリセット and `Quit`. **A launch pressed it**: `front end: screen 4`, and the chain gained nothing — only the front end's own job — so that screen is one the front end draws itself and builds nothing of its own. Its own cursor walks those nine, and **the bomb does not leave it**: one press moved the cursor instead, `front end: screen 4, cursor on item 6`, so the way back is the `Quit` row |
| 9 | `0x140049ec7` — state 5, and **the game going**: a launch pressed it, read `state: cur=5` for a few frames and then `cur=-1` with the front end out of the chain, and the process ended there — the recipe's own two steps after the press never ran |

**And items 2 and 3 are two kinds of run that are neither a plain one nor the Extra**, which the screen
after them says by the picture it loads. `0x14004c278`, inside state 8's case, picks one of four names by
those two bytes and the difficulty:

| | |
| --- | --- |
| `+0x168d3` set | `data/title/select00e.png` (`0x1403517e8`) |
| else `+0x168d1` set | `data/title/select00c.png` (`0x140351808`) |
| else `[0x14053d410]` is 4 | `data/title/select00b.png` (`0x140351828`) — the difficulty being the Extra's |
| else | `data/title/select00.png` (`0x140351848`) |

— and `0x14004c470`, `0x14004c719`, `0x14004dc98`, `0x14004de90` and `0x14004e139` read the same pair for
the screens after it. **So four banners and four kinds of run**: a plain one, the Extra, item 2's and
item 3's.

**And item 2 is the stage practised on its own, which a picture settled.** A launch pressed that item and
held the screen after it for a minute: it came up on **screen 9**, the difficulty select — so item 2 walks
into a run the way items 0 and 1 do — with **`PRACTICE`** across the bottom of it. The recipe is
`the-item-2-banner.yaml` beside the copy, and the picture wanted a minute's hold because fifteen seconds
was shorter than the watch's own poll and the screen had gone before one was asked for.

**So the byte a run carries the practice by is `0x14053d404`**, which `RUN_START` writes from item 2's
`+0x168d1` — and it is the same byte that opens the way out instead of the name entry at `0x14007bdde`,
a stage practised on its own being scored by nobody. `Th06nc::read_state` and `Th06nc::run_start` answer
`practice` off it; what item 3 is is still only its banner, `select00e.png`.

**And orb asks over three of them**: `Th06nc::menu_pointed_at` answers `Menu::Run` for items 0, 1 and 2 —
each of the three being a run, which is what the mode is a question about — and nothing for the other
seven. `Th06nc::run_pointed_at` then answers `practice` off `+0x168d1` on the last screen before the
stage, a screen before `RUN_START` copies it, which is what leaves `Th06nc::run_slot` with no name to keep
a practice run under.

### The second table, and the replay list behind it

**Screens 15 to 26 share the outer table's `0x140049f38`, which dispatches again.** That block takes the
state out of `[rsp+0x30]`, adds `-15`, bounds it against `0xb` and jumps through a table of twelve at
**`0x14004e8f0`**:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(12):
    off = struct.unpack_from("<I", d, at(0x14004e8f0)+4*i)[0]
    print(f"state {i+15:2d} 0x{0x140000000+off:x}")'
```

| | |
| --- | --- |
| 15 | `0x140049f5e` — `0x14003b110` called and `0x10` written into `[front end + 0x168b0]`, which is the state: this screen puts up state 16 |
| 16 | `0x140049f72` — **the replay list**. `0x14004a042` formats `replay/th6_%.2d.rpy` (`0x140351368`) with a number that steps up, sixty times — the loop steps `rdi` by `0x78` while it is under `0x1c20` — into `[front end + 0x34 + n*0x200]`, and `0x14004a05c` formats `0x140351380` beside it into `[front end + 0x7834 + n*8]`; then `0x14004a15a` loads `data/replay00.anm` (`0x140351998`) into the object at `0x140abac30` |
| 17 | `0x14004a09f` |
| 18, 19, 20 | `0x14004a2a5`, `0x14004a29d`, `0x14004a64f` |
| 21 to 25 | `0x14004e346` — never reached, the outer table giving each of those five a case of its own |
| 26 | `0x14004a7f0` |

**And the press that reaches it is the menu's item 4**, through state 15: item 4's case writes `0xf` at
`0x140049db0`, state 15's case calls `0x14003b110` and writes `0x10` at `0x140049f63`, and state 16 is the
list. Which is what item 4 looked like writing nothing — the state it writes is two states away from the
screen somebody sees.

**And the states the other items open, read off their own cases**:

| | |
| --- | --- |
| 4 (item 8) | `0x14004a831` — `0x14004e950(front end, 9)` and then a walk of entries `0x120` bytes apart at `[front end + 0x9640]`, each gated on the byte at `[front end + 0x158e0 + i]`, the indices coming out of `0x1403510a8`. Nine items and no job of its own, which is the shape of the options |
| 12 (item 5) | `0x14004d92c` — where `[front end + 0x9638]` has reached `0x32`, `r11d` is written into `0x140c6dfbc`, the state the supervisor is wanted in, and the case answers nought. `r11d` holds **6** from `0x1400491f3` |
| 21 (item 6) | `0x14004d947` — the same shape with **9** written. So the music room is the supervisor's own state 9, built after the front end asks for it — its job `0x140059bf0` reaches `data/bgm/th06_%d` through `0x140059680` |
| 23 (item 7) | `0x14004d966` — the same shape with **11**, `r13d` holding `0xb` from `0x140049220` |

**And the unlock bytes the menu reads are four groups of three**, `0x18` apart: `0x140549d51`, `+0x102`,
`+0x103`, then `0x140549d69`, `0x140549d81` and `0x140549d99` with their own two each, every one compared
against `0x63` at `0x1400498eb` onwards. Which is what lights the Extra's item — see item 1 above, and
*The clear records*.

**How those launches were driven**, so that a build whose menu differs can be read the same way: a recipe
beside the copy that walks to the item by its own cursor number, decides, holds the screen for fifteen
seconds and quits — `press: [down] … until: { screen: 3, cursor: N }`, then `press: [decide]`, then
`wait_until: { frames: 900 }` — driven through
`.claude/skills/driving-orb-with-autoplay/scripts/drive-a-run.sh` at `--log=verbose --pace=8`, and read
off the `front end: screen N` and `chain:` lines. **Items 2 and 3 are left out** because both start a
run, and **the item is waited for by its own number** rather than by how many taps it took, the Extra
being skipped while it is locked.

**But none of the ten is a ranking**, which three launches between them say: items 0 and 1 start a run,
items 2 and 3 start one too — they are the pair that writes `[front end + 0x168d1]` — items 4 and 5 write
no state at all, item 6 opens screen 21 with the job `0x140059bf0`, item 7 screen 23 with `0x140013e70`,
item 8 screen 4 with no job of its own, and item 9 quits the game. **Not one of them builds the screen a
run ends at**, which is what a ranking of this game is — see below, where that screen's own build is
driven from the byte the *stage's* build sets and not from any item here.

**But one more build of the screen a run ends at is read, and it is not the front end's.** `0x14006f5d0`
is that build and the exe calls it from five places: `0x14007b966`, `0x14007ba37`, `0x14007bdbd` and
`0x1400763a9` — the supervisor's own — and `0x14000cdbc`. The last of those is reached this way:
`0x140009800`, the chain job at priority 2 and the second of the three a launch's `chain:` line reads
before a run, ends with `cmp BYTE PTR [0x14053d406],0` at `0x140009961` and, where that byte is set, calls
`0x14000c960(its object + 0x6e10)` at `0x140009971`; inside that, `0x14000cd6f` waits for `[0x140c6e378]`'s
`+0x34` to read nought and `[0x140c6e370]` to be nought, and `0x14000cd97` then takes `0xbb48` bytes off
`malloc`, memsets them and builds.

**And `0x14000c960` is a machine of six states over `[its object + 0xd94]`**, `cmp eax,5; ja` at
`0x14000c987` bounding it and the table at `0x14000d068` holding the six:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(6):
    off = struct.unpack_from("<I", d, at(0x14000d068)+4*i)[0]
    print(f"state {i} 0x{0x140000000+off:x}")'
```

| | |
| --- | --- |
| 0 | `0x14000c9ab` — the init, which picks 12 or 9 by `[+0xd98]` at `0x14000ca47` |
| 1 | `0x14000ca74` — **a menu of three items**, which is what a run over puts up: gated on `[+0xb4c]` being four or more, the word at `0x140abae80` held against `0x140abae84`'s, `0x10` and `0x20` moving `[+0xb44]` modulo three, `0x4000` — the cancel — going to state 5 at `0x14000cb99`, and `0x100` — the decide — branching on the cursor at `0x14000cbe1`. **Item 1 is the ranking**: `0x14000cc1f` requires `[0x14053d3dc]` clear, `[0x140abae58]` non-nought and that object's `+0x70` non-nought, and `0x14000ccd8` then writes **state 2** |
| 2 | `0x14000cd6f` — **the screen built**: `0x14000cd97` off `malloc`, `0x14000cdbc` the build, and then `0x14000cdc2` writes 1 into its `+0x9e9a` and `0x14000cdc9` **12 into its `+0x9e94`** — `result_screen::STATE`, and twelve is the ranking — with the screen kept at `[0x140c6e370]` |
| 3 | `0x14000ce2b` — waits for `[0x140c6e370]` to be nought, the screen having taken itself down, and goes back to state 1 |
| 4 | `0x14000ce7e` — **the way back to the front end**: past `[+0xb4c]` of `0x14` it fades through `[0x140c6e378]` and then `0x14000cf0a` writes state nought, `0x14000cf10` clears `0x14053d406` and `0x14000cf19` writes **1 into `[0x140c6dfbc]`** — the supervisor's state wanted, the front end |
| 5 | `0x14000cf25` — the same shape as 4, which is what the menu's own cancel goes to, **and the one site in the exe that asks the supervisor for 12**: past the fade, `0x14000cfa5` puts `[+0xd94]` back to nought, `0x14000cfab` clears `0x14053d406`, `0x14000cfb2` sets `[0x140aee0ac]` and `0x14000cfb9` writes **`0xc`** into `[0x140c6dfbc]` — where state 4 writes 1 in the same place. So the decide out of the ranking goes to the front end and the cancel out of the menu goes to state 12 |

**So this is the path the ranking is shown through**, and `0x14053d406` is the byte it runs behind: state
2 is `result_screen::RANKING`'s own build and `0x140c6e370` is where that screen is kept while it is up.

**And the job runs three of these machines, on three sub-objects of one static.** `0x140009b30` builds the
chain element at `0x140436e30` with `0x140009800` as its callback and `0x140429270` as the object, and that
callback's tail hands a sub-object to each: `0x140009947` calls `0x14000a850` on `+0x52d0`, `0x14000995c`
calls `0x14000bc20` on `+0x6070` behind `[0x14053d401]`, and `0x140009971` calls `0x14000c960` on `+0x6e10`
behind `[0x14053d406]`. The table above is the third of them — the one `THE_RANKINGS_MACHINE` reads — and
the first has a table of its own, **fifteen** states at `0x14000b5d0`, dispatched on the same `+0xd94`
offset of *its* object:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(15):
    off = struct.unpack_from("<I", d, at(0x14000b5d0)+4*i)[0]
    print(f"state {i} 0x{0x140000000+off:x}")'
```

Its **state 7**, `0x14000b497`, is the exe's other write of `0xc` into the supervisor's wanted state:
gated on its own `[+0xb4c]` reaching `0x14`, `0x14000b4b5` writes 12 there too. So the two writes of 12
are the two machines' own ways out, and neither is a run's end.

**And it is a run's own path and not a menu item's.** The byte has two writers — `0x14000bc61`, inside a
function the same job calls at `0x14000995c` behind `[0x14053d401]`, and `0x14003be26`, which is **the
stage's own build** — and none of the main menu's ten items writes it, the launches above having pressed
every one of the eight that does not start a run. So a ranking of th06nc is something a run reaches,
where th06's and th06c's is an item somebody chooses; what states 1, 4 and 5 of the machine are for is
what a launch that reaches one would say.

**So `Game::show_ranking` has nothing to ask of this game's *front end***, which is what the six methods
orb's ranking is driven through are written against: the path orb takes on th06 and th06c — write what
the menu's own item writes, wait for the screen, tell it to leave — has no item here. What it would have
to drive instead is this machine, which is a run's own: `what_a_job_runs_on(0x140009800) + 0x6e10` is the
object, `+0xd94` the state, and item 1's own gates are the three above.

**And driving it would write nothing, which is what settles it.** The one write of this game's score file
is the name entry's own confirm — see *The one write of this file, and the six calls that are not it* —
so the screen this machine's state 2 builds is a screen that writes as it goes down on th06 and writes
nothing here. `Th06nc::leave_run`'s own state 8 is the same story: `0x14007bb13` calls
`0x14006db10(object, 0, 0, 0)` at `0x14007bb42` and the null record reaches the teardown. So orb declines
the six methods a ranking is driven through and answers `writes_its_own_records` true instead, which is
th07's answer for th07's reason: a screen built and taken down would spend `commit_records`'s whole frame
allowance and leave the file as it was.

**What a Pointdevice run of this game counted therefore stands in memory and nowhere else**, and reaches
the file only if the session goes on to finish a run and type a name into it. Making even that much hold
wants the put-back `Hooks::card_records_read` is for, and **the seam it would go over cannot take the
patch**: `0x14006e310` — where the file's `CATK` records are laid over the ones in memory, called from
`0x14003c5e7`, `0x140057819` and `0x140075a0b` — has `sub rsp,8; mov r11,rdx; test rdx,rdx` and then a
`jne` at `0x14006e31a`, which is twelve bytes where a 64-bit jump needs fourteen and a thirteenth byte
inside a relative jump cannot be relocated.

**And four of its screens press themselves through.** `0x1400491af`, in the front end's own frame: where
its state is under `0xf` and that state's bit is set in the mask `0x6a00` — states **9, 11, 13 and 14**,
the difficulty, the character, the shot and the screen after them — and `[front end + 0x9638]` is
exactly `0xf`, `0x1400491cc` writes `0x100` into the input word itself. So those four advance on their
own fifteenth frame whatever anybody presses. State 3, the main menu, is not in that mask.

**That is not why a recipe's steps at those screens used to give up, though**, which this note said for
one change and was wrong about: the give-ups were the missing hold — the bot's word reached the game's
keys through nothing that raised a question or wrote a frame down, see
`walks_the_calc_chain`. With the word going through `runtime::the_frames_word`, a launch of
`into-a-chapter.yaml` walked all thirty-three steps with none given up, where the launch before it had
given up two and stopped at the seventh. What the self-advance means is that a step at one of those four
has a screen under it that would leave anyway, not that the press does nothing.

**The main menu ignores its own decide for twenty frames.** Its case is `0x1400498d4`: the cursor is
moved through ten items by `0x14004e950`, the Extra's item 1 is stepped over where none of the clear
records' bytes at `+0xd2a1` onwards is `0x63`, and then `0x140049a8d` compares
`[front end + 0x9638]` with `0x14` and **leaves the case for anything under it** — so the edge check at
`0x140049aa5`, the game's own `0x100` of `0x140abae80` against `0x140abae84`'s, is not reached for the
screen's first twenty frames. Which is what `Game::menu_takes_a_press` answers a count for; a launch
that answered it true on the screen's first frame stood no question of orb's at all.

**Its cursor is `[front end + 0x28]`**, the dword the difficulty state compares with 0 to 3. `+0x30` is a
counter of the state's own frames — every state writes nought into it as it comes up and a launch read it
climbing once a frame — and `+0x9638` is another.

**And its menus repeat a held key**, the way th06c's do: six frames a press is one item.

## The game manager, `0x14053cab0`, which a run is made of

**One static object, and every number of a run is a field of it.** It is the `+0x38` of both chain
elements the stage's build registers — so it is what `0x14003baa0` and `0x14003c250` are called on — and
each address below was first written down here as a static of its own.

| offset | address | |
| --- | --- | --- |
| `+0x0` | `0x14053cab0` | a dword `0x14006a22a` steps up, three instructions before `0x14006a581` puts the run in the retry menu — **and it is not the deaths**: a launch of nineteen of them read it as nought on every one of its 335 frame lines, the frame each death was found on included. What it is, is not established, and orb counts the player's own hits instead — see `orb_core::game::th06nc`'s `DEATHS` |
| `+0x4` | `0x14053cab4` | **and how many bombs it has used**, `0x14006a182` |
| `+0x8`, `+0xc` | `0x14053cab8` | **the cards the run has captured and the cards it has attempted**, stepped up at `0x140027c45` and `0x140027c5d` beside the per-card record's own count at `[records + 0x130]`. `+0x8` is where th06c counts its captures too |
| `+0x10`..`+0x1c` | `0x14053cac0` | four floats the build writes `236, 16, 384, 448` at `0x14003c2bd`, and the pause menu writes the same sixteen bytes over at `0x14003bb7c` |
| `+0x20`, `+0x21` | `0x14053cad0` | **the character and the shot**, one byte each: `0x14004ef15` and `0x14004ef29` split a replay header's `character * 2 + shot` back into them, and `0x14003c4a6` reads the pair to pick the ranking record a run is held against |
| `+0x24` | `0x14053cad4` | **the stage, counted from one**: `0x14003e6ac` multiplies it by a thousand into `Stage * 1000 =  %5d`, `0x14003e669` compares it with 6, and a launch's first stage read `stage 1`. The Extra read `stage 7` — `0x14007bc9c` writes 6 as it starts and the stage's own start counts it on |
| `+0x28` | `0x14053cad8` | **the power**, a word the item walk's two power cases write back at `0x1400446b1` and `0x140044a9b`, `0x80` being where the full-power item writes it at `0x140044e61`, and the build starting it at nought — or at eight where `[0x14053d414]` is set, `0x14003c334` |
| `+0x930` | `0x14053d3e0` | **the score the panel draws**, a qword, which the extend check at `0x14003c9c0` holds against the thresholds in the table at `0x14034e0c8` |
| `+0x938` | `0x14053d3e8` | **the score the run counts**, the qword every bonus is added to — `0x1f4` for a point item at `0x140011967`, `0x3e8` at `0x140044e46`, `0xa` at `0x14004471f`, and an enemy's own at `0x140039404`. th06c has the same pair, the drawn one rising to this |
| `+0x948` | `0x14053d3f8` | **the score to beat**, which the build writes at `0x14003c5dd` as the larger of the file's own top score and a million |
| `+0x950` | `0x14053d400` | **the pause menu**: `0x14003bb75` sets it where the menu button was newly pressed inside a stage, the pause menu's own job at `0x14000affe` and `0x14000b472` clear it, and `0x1400572fd` clears it as the front end comes up |
| `+0x951` | `0x14053d401` | **and the menu a run out of lives is asked to continue from**, which the player's own death sets at `0x14006a581` |
| `+0x95c` | `0x14053d40c` | **the point items collected**, a word the item walk's point case steps up at `0x140044a21` |
| `+0x960` | `0x14053d410` | **the difficulty**, compared with 4 for the Extra at `0x14007bc97` and written 4 by the build at `0x14003c345` where the stage is six or past it; Normal is 1 and the Extra 4 on the `run:` lines |
| `+0x968`..`+0xd268` | `0x14053d418` | **the card records**, 134 of `0x180` — see below |
| `+0xd26c` | `0x140549d1c` | **the graze**, a dword capped at `0x1869f`: the bullet's own test steps it at `0x14006c55f` and the laser's at `0x1400118f2`, both behind `[0x140553eb8]` — **a byte no instruction in the exe writes and which the file has at nought**, so every gate on it passes and the graze is always counted; its five readers are those two steps, `0x14003698b`, `0x14003740b` and the item walk's `0x140044f52`. `0x14000beea` writes the count back to nought. **The HUD draws it and a stage's end scores it** — `0x14003e70a` reads it for `Graze *   10 = %6d` at `0x140350938`, ten times the count, and a picture of a run's own HUD reads `Graze 0` under the power |
| `+0xd270` | `0x140549d20` | **and a second graze count nothing reads**, capped at `0xf423f`: those same two tests step it a line after the graze — `0x140011907` and `0x14006c574` — and those four instructions are the whole of what touches it in the exe. Nothing clears it either, where `0x14000beea` clears the one above, so it is a total the build keeps and never shows |
| `+0xd280` | `0x140549d30` | **the rectangle the player is held to**, left, top, width and height — `0x14006aade`–`0x14006ab49` puts the position back inside it, and the build writes `8, 16, 368, 416` at `0x14003c2d9` |
| `+0xd290` | `0x140549d40` | **lives**, one byte: the item walk's 1up case stops it at 8 and steps it up at `0x140044cc7`; `0x14007bca6` writes `0x302` over it and the byte after as an Extra run starts, and `0x14007bce0` writes `[0x140c6dfd4]` — a chosen starting stock — for the rest. A launch's Normal run started with `2 live and 3 bomb(s)` |
| `+0xd291` | `0x140549d41` | **bombs**, one byte, the same shape at `0x140044c87`, and `0x14007bced` from `[0x140c6dfd5]` |
| `+0xd292` | `0x140549d42` | **how many extra lives the score has paid for**, the byte the extend check at `0x14003c9d2` climbs, which the build starts at nought or at 4 for the Extra, `0x14003c367` |
| `+0xd294`..`+0xd2f4` | `0x140549d44` | **the clear records**, four of `0x18` — see below |
| `+0xd408` | `0x140549eb8` | **the seed the stage started from**, which `0x14003c9f1` copies out of the generator |
| `+0xd418` | `0x140549ec8` | **the frames the stage has run**, which the stage's own job steps up at `0x14003c04d` and the chain build writes nought into at `0x14003c0f7` |
| `+0xd41c` | `0x140549ecc` | **the rank**, out of a table by difficulty at `0x14034e0e0` — `0x14003c9eb` |

And beside it, statics that are not fields of it:

| | |
| --- | --- |
| `0x14053d3dc` | **a replay is being played back**: set at `0x14004eeb3`, cleared at `0x14004f395` and `0x140053b97`; the frame's fast-forward at `0x14003d910` runs only where it is set, and refuses at `0x14003d919` where the pause menu is up |
| `0x14053d414` | a byte the run start reads for which starting stock applies, and the item walk reads for whether a 1up counts the deaths down instead; `0x14004ef3a` sets it from a replay header's mode being 1 — a mode of this build, unread |
| `0x140549eb4` | **the stage a replay is playing**, written at `0x14004efa7` |

**And the attract demo has no flag here.** `0x14053d400` had been written down as one; it is the pause
menu, set by a key press inside a stage. Both callers of the replay's own start `0x14004ede0` are the
front end's replay list — `0x14004a419` walks the header's stages for the first one recorded and
`0x14004a725` takes the one the cursor is on — so nothing has been found that a demo would be told by.

### The card records

**134 of `0x180` bytes** at `+0x968`, laid out by `0x14003c390`–`0x14003c457`: each is preceded by `0xc0`
words drawn from the generator, so that a record the game has not named holds noise and not zeroes —
th06c's do the same. Of each, `+0x0` is `CATK`, `+0x4` and `+0x6` are `0x180` and `0x180`, `+0x8` is the
version `0x15`, `+0x10` is the card's own index as a word, and `+0x3c` is the attempts, written nought;
`+0x40` onwards is memset and `+0xe0`..`+0x180` zeroed. th06c's is `0x40` bytes and 64 of them, so
nothing of that layout carries.

**And what the card's own start writes into one splits on the mode**, `0x140027a0e` testing
`[0x14053d405]` — the byte a scene out of the table at `0x140c6e3e0` is played under:

- **an ordinary run steps the attempts**, `0x140027a2a` reading the word at `+0x3c`, capping it against
  `0x270f` and `0x140027a3a` writing it back — with `[0x140bfa190]` put into the dword at `+0xc` beside
  it, and the name's own sum into the byte at `+0x12` (`0x140027a0a`);
- **and a scene steps a count of its own instead**: `0x140027a43` onwards takes the shot and the character
  from `0x14053cad1` and `0x14053cad0` as `shot + character*2`, adds the difficulty from `[0x14053d410]`
  four times over, and steps the dword at `+0xe0 + that*4` — capped against `0x3b9ac9ff`. So sixteen
  dwords of the record are the scene mode's own attempts, one per difficulty and shot, and the word at
  `+0x3e` is written by neither.

**And `+0x3e` is the times the card was captured in the main game, a field this build never steps.** The
Spell Practice screen is what shows it: `0x1400535db` formats `%s %u/%u` (`0x1403516b0`) with the
localised string of the key `EXE_SPELL_PRACTICE_MAIN_GAME` (`0x1403516c0`), `[0x1400534fe]`'s `+0x3e` as
the first number and `+0x3c`'s attempts as the second, and the line above it is the same pair of numbers
under `EXE_SPELL_PRACTICE_CAPTURED` (`0x140351690`) out of the sixteen dwords. **And the two accesses
above are every access the exe has to a record's `+0x3e`** — `0x1400534fe` reads it and
`0x14006e403` caps it at `0x270f` as the file is parsed, beside `0x14006e3f9` doing the same to `+0x3c`.
So a capture count in the file is kept, capped and drawn, and nothing in this build raises one — **and a
copy's own file holds them**: 23 of its 134 records carry an attempt and most of those a capture beside
it, which is a count that was in the file before this build parsed it.

```sh
objdump -d th06nc.exe | grep 'WORD PTR \[r[a-z0-9]*+0x3e\]' | grep -vc rsp
```

— **3**, the three above. The two the `rsp` filter drops are `0x1400360d8` and `0x1400360e0`, a stack
structure of the boss's own attack and not a record.

`0x14006e310(the file, the records)` at `0x14003c5e7` is what lays the file's own `CATK` records over
these, and what it does with one is read off:

- **the walk** is the parsed block's own: `movsxd rax,[rcx]` is where its records begin and `[rcx+0x10]`
  its length, and each record is advanced past by the word at `+0x4` (`0x14006e4c6`). Those two fields are
  the block `0x14006e140` answers with and not the file's header — that function copies the records region
  into a fresh `0x18 + n` bytes and writes `0x18` at `+0x0` and `0x18 + n` at `+0x10` (`0x14006e267`);
- **what it takes a record for**: `CATK` in the dword at `+0x0` and `0x15` in the byte at `+0x8`
  (`0x14006e350`, `0x14006e35d`), anything else skipped;
- **which record of memory it goes into**: the word at `+0x10`, turned into
  `lea rdx,[rax+rax*2]; shl rdx,7` — `card * 0x180` — and `0x180` bytes copied off the magic as three
  turns of `0x80` (`0x14006e390`);
- **and a number from `0x86` ends the walk rather than skipping that record**: `0x14006e370`'s `jae`
  jumps to `0x14006e4da`, the function's own way out, so the records behind it are not laid down either;
- **then the clamps**: the words at `+0x3c` and `+0x3e` down to `0x270f` (`0x14006e3f3`, `0x14006e3fd`)
  and the dwords across `+0xe0`..`+0x130` down to `0x3b9ac9ff` (`0x14006e420` onwards).

`score_file::catk` builds one of these and `image::Image::the_files_card_records_are_laid_over` is that
read, so that a laid-out game brings a file's counts in where this does. **What the word at `+0x3e` is
has not been read** — th06's record keeps a capture count beside its attempts and nothing here has been
read as one.

### The clear records

**Four of `0x18` bytes** at `+0xd294`, one per difficulty, laid out by `0x14003c603`: `CLRD` at `+0x0`,
`0x18`/`0x18` at `+0x4`, `0x15` at `+0x8`, `0x01010101` at `+0xc` and `+0x11`, and `1` at `+0x14` and
`+0x19`. `0x14003c676` lays the file's own over them, choosing by the record's `+0x16` where that is
under four.

## The generator

| | |
| --- | --- |
| `0x140abae64` | **the seed**, one word. Every draw in the exe is inlined — `xor cx,[0x140abae64]; sub cx,0x6553; movzx eax,cx; shl cx,2; shr ax,0xe; add ax,cx` with the sum written back — and there are 216 references to it, `0x14000db9a` and `0x14003c3a0` among them |
| `0x140abae60` | **and how many draws have been made**, stepped up beside each: by one at `0x14003c3a0` and by two where a draw takes two words, `0x14000dba1`. 122 references |

The same shape th06c's is, whose seed is `0x14090a10c` with its count four bytes below it. The stage's
own build copies the seed into `+0xd408` and writes the count back to nought, `0x14003c9f1` and
`0x14003c9f8`.

## The stage's two seams

**`0x14003c0a0` registers the stage's two chain elements**, called three times in the exe —
`0x14007b94c`, `0x14007bac7` and `0x14007bcaf`, all three from the job `0x14007b7f0`, whose own case at
`0x14007bc7e`–`0x14007bd0b` writes the stage, the difficulty, the lives and the bombs first. It sets the
calc element `0x14053ca70` up with `0x14003baa0` at its `+0x8`, `0x14003c250` at its `+0x10`,
`0x14003cf20` at its `+0x18` and the game manager at its `+0x38`, links it into `0x14053c090`, calls the
`+0x10`, and then does the same for the draw element `0x14053ca30` with `0x14003c080`. Answers nought
where it built one and `-1` where the `+0x10` refused.

**And the `+0x10` is called from inside that registration**, at `0x14003c173` — so the build runs inside
the supervisor's own case and not on the frame after it, which is what the next section's branch is read
against.

**What that case writes the stage as, which is where a run's own first stage comes from**: `0x14007bc68`
asks whether a replay is playing and takes the replay's own stage out of `[0x140549eb4]` where one is;
otherwise `0x14007bc84` reads the practice byte `[0x14053d404]` and `0x14007bc8f` the scene's
`[0x14053d405]`, and either of those set takes `0x14007bcd3` — **the stage decremented**, so the build's
own step up lands on the stage the front end chose. With neither set, `0x14007bc97` compares the
difficulty with four: **the Extra's stage is written 6** at `0x14007bc9c`, with `0x302` — two lives and
three bombs — beside it at `0x14007bca6`, and every other run's stage is written nought at `0x14007bccb`.
Then `0x14007bcd9` takes the lives and the bombs out of `th06.cfg`'s `+0x444` and `+0x445`, with the
Extra's `0x302` written over them again where the difficulty is four.

**`0x14003c250` is that `+0x10` — the stage's own build**, called once in the exe with the game manager
in `rcx`, which is what makes it `Entered::InEcx`. What it puts in place is every number of a stage: the
four floats, the power, the card records, the clear records, the score to beat, the extra lives, the
rank, the seed's copy and the draw count back to nought — and then the player (`0x1400694d0`), the
bullets (`0x14003ca43`) and the rest are registered. So the moment a resumed run's numbers can be
written over is where this *returns*, which is what `Hooks::stage_begun` is; its prologue is
`mov [rsp+0x10],rbx; mov [rsp+0x18],rbp; mov [rsp+0x20],rsi`, fifteen bytes and none of them
rip-relative.

**Its first act is a branch on the supervisor's state wanted.** `0x14003c27c` is
`cmp DWORD PTR [0x140c6dfbc],3` — the supervisor's `+0x42c` — and `0x14003c299` takes the taken side to
`0x14003c78d`, which copies `+0x938` over `+0x930` and clears `+0x940`; the other side, `0x14003c29f`
onwards, writes the four floats, the stock out of `[0x140549d40]` and `[0x140549d41]`, and the card
records. Both then reach `0x14003c7a2` — `0x14003c78b` being `jmp 0x14003c7a2` — and the common tail
copies the seed at `0x14003c9f1`.

**And the stage is counted on in that shared tail and not in either side of the branch**:
`0x14003c7a2` is `mov r8d,[rdi+0x24]`, `0x14003c7bc` is `lea edx,[r8+1]` and `0x14003c7c0` writes it
back. So **every** build of a stage raises the manager's own count, which is why a run's first stage
comes up as one out of the nought the front end leaves in there, and why the number a resume writes into
that field is the stage counted from nought — see `Th06nc::start_stage`.

**Which two launches of `orb-real` and one instrument were needed to tell apart.** Written with
`STAGE_COUNTS_FROM` added back, a run picked up in its second stage came up in its third:
`resume: not put back — stage 3 came up where stage 2 was asked for`, `run: stage 2` said off what
`start_stage` wrote and `run: stage 3` after the build. The branch above reads like the thing that did
it, and `Th06nc::seed_the_stage` — which says the supervisor's pair and the stage field from inside that
same build — answered `the supervisor on Some(1) wanting Some(2), and the stage now reads Some(3)`,
ruling it out and leaving the tail. `0x14007b94c`, the case for wanting 2 out of the front end, is
`call 0x14003c0a0; jmp` and writes nothing either way.

**`0x14007a4a0` is where a seed can go in**, called once in the exe — at `0x14003ca09`, three
instructions past the copy — and it registers the job `0x140079360` on the static `0x1405547b0`. Its
prologue is `mov [rsp+8],rbx; mov [rsp+0x10],rbp; mov [rsp+0x18],rsi`, fifteen bytes. **Inside the build
and not in front of it**, because the build draws `0xc0 × 134` words laying its card records out: a seed
written at `0x14003c250`'s prologue would be 20,592 draws old by the time `0x14003c9f1` copied it.

## The score file

**And what one holds is read, by decrypting a copy's own** — the cipher below, and a walk of the records
by the length each carries:

```sh
python3 -c '
import struct
d = open("score.dat","rb").read()
key, out = d[1], bytearray(d[:2])
for b in d[2:]:
    p = b ^ key
    out.append(p)
    key = (key + p) & 0xff
    key = ((key // 32) + (key * 8)) & 0xff        # ror 8-bit by 5
print("length", struct.unpack_from("<I", out, 0x10)[0], "of", len(d))
off = 0x14
while off + 8 <= len(out):
    size = struct.unpack_from("<H", out, off + 4)[0]
    print(f"{off:#x} {out[off:off+4].decode()} {size} bytes, version {out[off+8]:#x}")
    off += size'
```

— which on a copy of 51,636 bytes walked to that exact end in **141 records**: `TH6K` once at `0x14`,
`HSCR` once at `0x20`, `CLRD` **four times** from `0x44`, `CATK` **134 times** from `0xa4` — the 134 the
manager keeps — and `MUSC` once at `0xc9a4`, every one of them version `0x15`. **So five kinds, and the
record header is `{ the magic, the length as a word, the length again as a word, the version as a byte,
three of nought }`.**

| | |
| --- | --- |
| `TH6K` | `0xc` bytes, the header record and nothing past its own version |
| `HSCR` | `0x24` bytes — `+0xc` the score as a qword (a copy read `0x0596f99c`), `+0x14` the shot combination, `+0x15` **the difficulty**, out of the screen's own `+0xa8`, which `0x140071ecc` copies from `[0x14053d410]`: a launch of the Extra read `its own 4` at every state of its walk and wrote **4** into the file, `+0x16` the mode, `+0x17` **`0x63` where the run finished and the stage it stopped in where it did not** — `0x1400722e2`'s `cmove` off `[0x14053d403]`, the byte a finished run sets, against `[0x14053cad4]`; the copy's read `0x63` — `+0x18` the eight bytes of the name (a copy read `ORB` and five spaces), `+0x20` four of nought. **No graze and no count of continues** |
| `CLRD` | `0x18` bytes, **four of them — one per shot** — with twelve payload bytes apiece: a copy's first read `01 63 01 01 01 01 63 01 01 01 00 00` and its second `01 01 01 01 01 01 01 01 01 01 01 00`, so `0x63` is a clear and `0x1` is not. These are the records `0x140549d51` onwards is laid from, and what item 1 of the menu asks about |
| `CATK` | `0x180` bytes, 134 of them — see *The card records*. A copy's own: **23 of the 134 carry an attempt** at `+0x3c`, most of those a capture at `+0x3e` beside it, and **not one has a scene attempt** among the sixteen dwords at `+0xe0` — so cards are offered by that copy's Spell Practice and none has ever been played there |
| `MUSC` | `0x10` bytes, four of payload: a copy read `ff 1f 03 00`, a mask of the tracks the music room has |

**The version byte is `0x15` in the file, which is what the parse wants** — `0x14003c528` refuses
anything else — so the `0x10` that `0x1400722c3` writes at `+0x8` of the record it builds at the screen's
own `+0x9e58` is not this field's value on the way out. That record is the node `0x14006e290` hangs in
the ranking's list for drawing; what the file gets is stamped by the write.

| | |
| --- | --- |
| `0x14006e140` | answers the file as one parsed block, or nought. Called from `0x14003c45d`, `0x140046f91`, `0x1400576ae`, `0x140075833` and `0x1400763b6` |
| `0x14006e310(block, records)` | lays the file's `CATK` records over the ones in memory, `0x180` bytes a card off the magic and the version `0x15`. Called from `0x14003c5e7`, `0x140057819` and `0x140075a0b` |

**And all three lay it over the same array, which is the one orb counts an attempt in.** `0x14003c5e7`
passes `manager + 0x968` and the other two the static `0x14053d418`, and those are one address —
`0x14053cab0 + 0x968` — which is `game_manager::CARD_HISTORY` and what `CURRENT_CARD`'s own
`card * 0x180` is an offset into.

| | |
| --- | --- |
| `0x14003c5e7` | **inside the stage's own build, and on its run-start branch alone** — between `0x14003c29f` and the `0x14003c78b` that jumps to the tail, after the layout loop at `0x14003c390` writes `CATK` and a card number into all 134. So the file's counts land on the array **once a run** and a stage transition leaves them alone, which is th07's shape exactly: that game empties every record as it builds a run and reads the file straight back into them |
| `0x140057819` | behind `0x1400577a2` testing the array's first dword for `'CATK'` — where it is not, `0x1400577d0`'s loop lays out the 134 with `0x180` in both halves of `+0` and `0x15` at `+0x4` first, which is the array being built for the first time |
| `0x140075a0b` | the screen a run ends at reading it, and only in the states that are not the two name entries, `0x1400759eb` and `0x1400759f4` sending nine and seventeen elsewhere |

**Whether the write at `0x14006db10` serialises that array is not read**, and it is the question a put-back
turns on: the seven `memcpy`s at `0x14006df26`–`0x14006dfef` take their sources from the score file
object's own buffers, the `+0x40` to `+0x70` of `[object + 0x70]` that `0x14006e058` frees, and neither
that array nor the manager is named anywhere in that function. So a put-back into the array orb counts in
may be a put-back the write never reads.
| `0x14006db10(file, record, name, keep)` | **the one write, and the six calls that are not it** — below |

### The one write of this file, and the six calls that are not it

**`0x14006db10` is entered seven times in the exe and writes on one of them.** `0x14006db4f` tests its
second argument and sends a null one to `0x14006e058`, which is **not** the write: that block frees the
seven buffers hanging off `[file + 0x70]` — `+0x40` through `+0x70` — and then `[file + 0x10]`, through
the exe's own `free` at `0x1402c6540`, and returns. The write is `0x14006e038`, on the other side of the branch,
and `0x14006e04f` tests the fourth argument to say whether the teardown runs after it.

```sh
grep -n 'call.*14006c420' th06nc.asm | while IFS=: read -r line rest; do
  sed -n "$((line-3)),${line}p" th06nc.asm | grep -E 'edx|call'; echo; done
```

— six of the seven read `xor edx,edx` three instructions before the call:

| | |
| --- | --- |
| `0x14007301f` | **the write**: the name entry's own confirm, which passes the record it has just built on the stack and `result screen + 0x9e88` — the eight bytes somebody typed — as the name, with `[screen + 0x9e9a]` as the fourth argument. `0x140073037` then writes 2 into `result_screen::STATE` |
| `0x14007b7bb`, `0x14007bb42`, `0x14007bc63`, `0x14007bd30`, `0x14007bd60`, `0x14007c7c0` | **the run's own ends**, each behind `[0x140abae58]` being non-nought and each with `xor edx,edx` — so each frees the file object and writes nothing |

With a record it builds a `0x78`-byte ranking entry out of `[file + 0x70]`: the score `[0x14053d3e0]`,
the run's own `[0x14053d3e8]`, the stage `[0x14053cad4]`, the name copied in, and the date through
`time`, `localtime` and `strftime` of `0x140351f18` into nine bytes.

**And the entry's own layout**, off the copy at `0x14006db66` onwards — which takes the record it is handed
in `rcx` field by field into the `0x78` bytes at `rbp-0x59` and is where each offset below was read:

| | |
| --- | --- |
| `+0x0`, `+0x4`, `+0x6`, `+0x7`, `+0x8` | a dword, a word and three bytes of the record, copied as they stand; `+0x9`, `+0xb` and `+0xc` are zeroed beside them |
| `+0x10`..`+0x13` | four bytes the record carries — and **four the write draws again from the game's own generator**: `0x14006dd7c` steps `[0x140abae60]` by four and the inlined draws at `0x14006dd95`, `0x14006ddb4` and their fellows write `(rng & 0x7f) + 0x40` into `+0x12`, `+0x13`, `+0x10` and `+0x11`, which is a printable byte apiece |
| `+0x14`..`+0x1c` | **the date**, nine bytes: `time` (`0x1402c05a8`), `localtime` (`0x1402c05a0`) and `strftime` (`0x1402c05b0`) of `0x140351f18` into `rbp-0x45` |
| `+0x1d`..`+0x25` | **the name**, nine bytes: the loop at `0x14006dd40` copies it a byte at a time until the terminator, with `+0x26` zeroed after it |
| `+0x28` | **the run's own**, `[0x14053d3e8]` — the same qword `0x14006dd36` writes into the span the stage picks, `[[file + 0x70] + 0x40 + 8*(stage - 1)]` off `[0x14053cad4]` at `0x14006dcea` |
| `+0x30`, `+0x34`, `+0x38` | **the run's slowdown, three times over** — `0x14006dd08` writes it into `+0x34`, `0x14006dd20` writes it plus `1.12` into `+0x30` and `0x14006dd25` it plus `2.34` into `+0x38`, so the two offsets are a check on the one number and not three numbers. And the number is read: `0x14006dcac` divides `[0x140c6df2c]` by `[0x140c6df30]`, `0x14006dcd2` takes `0.5` off it and `0x14006dcda` doubles it, `0x14006dce6` holds that between nought and one, `0x14006dcf0` takes it off one and `0x14006dd00` multiplies by `100.0` — a percentage that is nought at full speed |
| `+0x40`..`+0x70` | **the seven spans**, a pointer apiece — zeroed at `0x14006dc04` onwards and filled at `0x14006dc2e` through `0x14006dca8`, which is what the write concatenates |

**And the slowdown itself is a pair of floats the frame's timing keeps.** `0x14003ceb9` zeroes both as one
qword where a run is built; `0x14003dea2` adds the frame's nominal rate into `0x140c6df30` and
`0x14003def2` adds into `0x140c6df2c` one of four numbers by where the rate measured in `xmm6` fell —
the whole nominal above `54.0`, `48.0` above `42.0`, `36.0` above `30.0` and `30.0` below that, the
thresholds being `0x140352b28`, `0x140352b18` and `0x140352b00`. So the ratio is one where every frame
came out on time, and `0x1400752ea` is where the game draws it: the same divide, the same `- 0.5` and
doubling, and a bar out of what it leaves.

**And what the seven spans are is read off a file written and the file before it.** A launch let an Extra
run end, typed a name and pressed state 13's own decide — the one write, `0x14007301f` — and the copy's
`score.dat` went from **51,636 bytes to 51,672**, exactly the `0x24` of one `HSCR`. Decrypted and walked,
the two files' own regions are:

| | before | after | |
| --- | --- | --- | --- |
| `TH6K` | 12 | 12 | identical |
| the `HSCR`s | 36 | **72** | the new record **second**, in score order — 923,700 under the 93,780,380 already there — and the old one byte for byte |
| the `CLRD`s | 96 | 96 | **one byte differs**, the clear this run earned |
| the `CATK`s | 51,456 | 51,456 | identical: no card was captured |
| `MUSC` | 16 | 16 | **one byte differs**, a track unlocked |

**So each span is a kind, and the `HSCR`s are two of them** — the records above the new one and the
records below it, which is what an insert into a sorted run needs. With the file's own `0x14`-byte header
that is seven: the header, `TH6K`, the `HSCR`s before, the `HSCR`s after, the `CLRD`s, the `CATK`s and
`MUSC`. **And each is re-laid from memory rather than copied**, which the two changed bytes say: the run
raised a clear record and unlocked a track, and both came out in the file without any record being
inserted for them.

**And the record that went in is the second of the two, which says the body is copied and the header is
stamped.** The new one is at `0x44`, not `0x20`: `score 923700, +0x14 = 0, +0x15 = 4, +0x16 = 0,
+0x17 = 7, name `ORB` and five spaces` — the run was the **Extra**, and its `+0x15` is that difficulty's
own 4 while `+0x17` is the stage it stopped in, 7, because it died rather than finished. The `0x20` one
is the record the file already held, a Normal run's: `score 93780380, +0x15 = 1, +0x17 = 0x63`. **So
every field of the node reaches the file unchanged** — `[screen + 0xa8]`'s difficulty among them, which
a launch read as `difficulty 4 and its own 4` at every state of the walk `9, 15, 16, 10, 12` — **and the
version is the write's own**: the node's `+0x8` is `0x10` (`0x1400722c3`) and the file's is `0x15`, so
the write lays the header down and copies the body.

**And the list is sorted by score descending**, which the two records' order says: 923,700 went *after*
93,780,380. `0x14006e290` walks while `[record + 0xc]` of the node it is at beats the one being
inserted, so the spans that split the `HSCR`s are the records above the new one and the records below.

**And what it writes is seven spans of the file object's own, concatenated.** `0x14006df0c` onwards is
the same seven instructions seven times over: the start of span *i* is `[[file + 0x70] + 0x40 + 8i]`, its
end is `[file + 0x38 + 8i]`, the length is the difference, and `memcpy` puts it at `r14`, which is
stepped on by that length each time — `0x14006df26`, `0x14006df48`, `0x14006df6a`, `0x14006df8c`,
`0x14006dfae`, `0x14006dfd0` and `0x14006dfef`. Then `0x14006dff9` calls `0x14006c810` and
`0x14006e038` writes the buffer out. The teardown at `0x14006e058` frees those same seven, `+0x40`
through `+0x70`.

**So the file is the object's spans and not the game's own arrays — and this build never writes a card
count at all.** Three readings together say so:

- the write names neither `game_manager::CARD_HISTORY` nor the manager, taking the seven spans as they
  stand;
- **`+0x968` of the manager is referenced twice in the whole exe and both are destinations** —
  `0x14003c353`'s `lea rsi,[rcx+0x968]`, the layout loop's, and `0x14003c5d6`'s `lea rdx,[rdi+0x968]`,
  the lay-over's. Nothing copies that array out:

  ```sh
  grep -nE '0x968\]' th06nc.asm
  ```

- and a span cannot *be* that array, because `0x14006e058` hands all seven to `free`, `0x1402c6540` — a
  static of `.data` handed to `free` is a launch that dies, and launches do not.

So a run's attempts at a card live in memory for the session and nowhere else: the array is written by
the layout and by the file's lay-over, read by the card's own start at `0x140027929` and by the screen
that shows the records, and never serialised. Which is what `Th06nc::put_back_the_cards_a_run_left` is
for and the whole of what it can be for — keeping the counts across the run build that would otherwise
lay the file back over them.

**Which is what `Th06nc::writes_its_own_records` answers true off**: no ranking screen orb built and took
down would write anything here, this game's write being behind somebody typing a name. th06's is the
other way round — its screen going down is the write — and that is the difference the two answers are.

**The block holds records back to back**: `[block]` is the offset the first is at, `[block+0x10]` the
end, and `[block+8]` a tree the `HSCR` records are hung in. A record's `+0x4` is its own length, which
the walk at `0x14003c520` advances by.

**`HSCR`** (`0x52435348`) is a ranking record: `+0x8` the version `0x15`, `+0xc` the score as a qword,
`+0x14` the shot combination, `+0x15` the difficulty and `+0x16` the mode — the four `0x14003c52e`
onwards matches on to find the score a stage is started against.

**And the build of one is read field for field**, at the screen a run ends at, `+0x9e58` of its own
object: `0x1400722ca` the magic, `0x1400722c3` a byte **`0x10`** at `+0x8`, `0x1400722e5` a byte at
`+0x9`, `0x1400722bc` the score qword `[0x14053d3e8]` at `+0xc`, `0x140072295` the shot combination at
`+0x14` as `[obj + 4] * 2 + [0x14053cad1]`, `0x1400722a2` and `0x1400722af` the difficulty and the mode
at `+0x15` and `+0x16`, `0x1400722ec` the stage at `+0x17` where `[0x14053d403]` is nought, and
`+0x18` onwards the name the entry typed — `+0x9e70`, the entry `Th06nc::types_its_name` writes.
`0x14006e290(the list, the record)` then hangs it in the ranking's own list, a `0x18`-byte node of
`{ the record, its owner, the next }` inserted by `[record + 0xc]` descending. **Those are all of them,
so no graze reaches an `HSCR`** — and the `0x78`-byte entry carries none either, which together is the
whole of that question.

**And the `0x10` that `0x1400722c3` writes into `+0x8` of that node is not a version**: every version
compare in the exe is against `0x15` — nine of them, `0x14003c528`, `0x14003c668`, `0x14004701c`,
`0x14005775e`, `0x14006e35d`, `0x14006e5ea`, `0x14007590c`, `0x140075a89` and `0x14007649c` — and none
against `0x10`, so a record carrying it is one no parse of this build would accept. The node's `+0x4`,
where a record keeps its own length, is never written either. **And the file's own record holds `0x15`
and every body field of that node unchanged** — a launch's Extra run wrote `+0x15 = 4` and `+0x17 = 7`
into it, the difficulty the screen held and the stage the run stopped in. So the write lays the header
down itself, the magic, the two lengths and the version, and copies the body: what `0x1400722c3` writes
is neither of those and has no reader in the exe.

```sh
objdump -d th06nc.exe | grep -c 'cmp *BYTE PTR \[r[a-z0-9]*+0x8\],0x15'
```

**And the cipher is th06c's byte for byte**, read off `0x14006e140`: the key is seeded from the byte at
`+1`, every byte from `+2` on is `xor`ed with the key and the key is then the key plus the plain byte
just decrypted, rotated right by five (`add r9b,cl; ror r9b,5` at `0x14006e1b0`); a sixteen-bit sum of
the plain bytes from `+4` on sits in the word at `+2`; `+8` is the offset the first record is at and
`+0x10` the file's length; and the first record must be `TH6K`. What differs from th06c is the record
version — `0x15` where th06c's is `0x10` — and the bodies. The file itself is `[0x140c6e368]` with its
length in `[0x140c6e18c]`, and under `0x18` bytes is refused.

## The allocator, which checks itself and kills the process

**This build allocates out of a TLSF of its own** — Two-Level Segregated Fit, the real-time allocator
whose blocks carry a tag and whose free blocks hang off a two-level index — and **its consistency checks
end in a deliberate write to address nought**. Seven of its messages are in `.rdata` as wide strings:

| | |
| --- | --- |
| `0x1403015d0` | `TLSF カテゴリ構造エラー 1  MemTag->Size %d` |
| `0x140301620` | `TLSF カテゴリ構造エラー 2  MemTag->Size %d` |
| `0x1403028de` | `TLSFエラー　　カテゴリ２テーブルエラー` |
| `0x14030293e` | `TLSFエラー　　空きメモリタグサイズエラー  MSB:%d  i:%d  SLI:%d  j:%d` |
| `0x1403029de` | `TLSFエラー　　空きメモリタグカテゴリ情報エラー  MemTag->FLI:%d  MSB:%d  MemTag->SLI:%d  SLI:%d` |
| `0x140302aae` | `TLSFエラー　　空きメモリタグリストに使用メモリタグが含まれていました` |
| `0x140302b2e` | `TLSFエラー　　カテゴリ１テーブルエラー` |

**The first two are the ones a restore reaches**, and they are the only two any code refers to by name:
`0x14026bc41` and `0x14026bc73` each compute a block's aligned end out of `[MemTag + 0x20]` and
`[MemTag + 0x28]` — `div` by the alignment in `rbx`, the remainder taken off — compare it with the size
asked for in `rdi`, and where the block does not reach it load the string into `rcx` and call the
reporter `0x1400c2ac0`. What follows that call, at **`0x14026bc81`**, is

```text
c7 04 25 00 00 00 00 ff ff ff ff    mov DWORD PTR ds:0x0,0xffffffff
```

— a write to address nought, which is how this game ends a process it has decided is unsound.

**Which is what kills it when a chapter of stage 1 is put back**, and the reading is a debugger's own
report rather than an argument. `cdb` attached across a `--stress=60` launch of `at-a-card.yaml` caught
it on the seventeenth restore, the first of a `music=keep` chapter:

```sh
# the launch, in the background, and then the pid off orb.log's first line
bash .claude/skills/driving-orb-with-autoplay/scripts/drive-a-run.sh <copy> th06nc.exe \
  at-a-card.yaml --log=verbose --speed=1 --without-steam --no-resume --stress=60
"/mnt/c/Program Files (x86)/Windows Kits/10/Debuggers/x64/cdb.exe" -p <pid> \
  -logo '<copy as Windows spells it>\cdb-stress.log' -c "g; .lastevent; .exr -1; r; kb; q"
```

```text
(15610.147ec): Access violation - code c0000005 (first chance)
ExceptionAddress: 00007ff709fc9d71 (th06nc+0x0000000000269d71)
Attempt to write to address 0000000000000000
rbx=0000000000000010 rdi=0000000000000300 r14=000001c19028ce10 rbp=000001c19028cea0
th06nc+0x269d71 : th06nc+0xf271a : th06nc+0xf2812 : th06nc+0xf139f : th06nc+0x8fc9f
  : th06nc+0x95316 : th06nc+0x1b5c : th06nc+0x3bfa4 : orb!DllMain+...
```

— so the run asked the allocator for `0x300` bytes at `0x10` alignment inside the game's own update, the
MemTag at `0x1c19028ce10` did not describe a block that big, and the check killed the process. **The
`0xc0000005` is real and the address is nought**, which is why nothing in the log read like a crash: the
write is the game's own last instruction and not a pointer it followed by accident.

**The object it all hangs off is a static**, `0x1407e91e8`: `0x1400f4255` builds it once — the flag at
`0x1407e91e0` and `lea rdi,[0x1407e91e8]` — and the allocate at `0x14026b850` takes it in `rcx`, with
`+0x38` a mode, `+0x40` the default alignment and `+0xd0` a lock of its own. **Its upper bound is where
the next object begins**, `0x1407f1600`, nothing between the two being referenced from anywhere. So
`0x1407e91e0..0x1407f1600` is what `Th06nc::live_handles` holds back — the one range there whose reason
is the heap rather than a handle or a thread — and with it held back the same probe took retries 17 to 25
of the two chapters that had killed it and ran its recipe out.

## The renderer

`main` takes `0x37070` bytes off the heap at `0x140047521`, zeroes them, builds them with `0x1400021c0`
and writes the pointer to `0x140abac30`. The frame leaves `-1` in its `+0x20f80` after the draw, and the
last `0xff6c` bytes are the batch of sprites — see *The draw*.

**And the assets a stage loads are inside that block**, which is what says a texture released between two
chapters is already held back: the stage's own load at `0x14003f440` onwards reads
`[[0x140abac30] + rax*8 + 0x26190]` for thirty-two of them — `r14` counting from nought while the
destination steps `0x120` at a time up to `0x2520` — and `+0x26c58` and `+0x26c60` for two more, all of
them under the `0x37070` the block is. `Th06nc::live_handles` puts that whole block in what a restore
leaves alone, so nothing of what a stage loaded is written back over.

## The sound layer's own block

**Thirty-two objects of `0x388` bytes at `0x1407e20c0`**, which is the block a restore has to leave
alone — read off the table itself:

| | |
| --- | --- |
| `0x1400f2f40` | indexes it: `lea r12,[0x1407e20c0]; imul rsi,rax,0x388`, the slot's first dword at `[rsi+r12]` |
| `0x1400f3610` | walks it: `mov r14d,0x20` for how many, `add rbx,0x388` for the step |
| `0x1400fbb94` | holds an index against `0x20` besides |
| `+0x70` of a slot | **a lock**, whose address `0x1400f2f85` takes and which `0x140264720` and `0x1402647d0` enter and leave around the body of every walk |

**And the bounds are where the references stop**: `0x1407e9190` — the last eight bytes the range covers
past the thirty-second slot — is referenced nowhere, and `0x1407e91c0`, where it ends, is referenced
from seven places as something else's first field.

**A lock put back from a snapshot says a thread is somewhere it is not**, which is what holding th06c's
own block back was for — `0x14044da30`..`0x14044e830` there, five objects of that library's, and what
putting them back cost that game was the frame rate after a chapter.

## The lock, and every static one of them

**Four functions wrap `CRITICAL_SECTION`**, each resolving its own by name rather than importing it —
`0x1400b5dd6` and the unrolled `GetProcAddress` calls around it write the pointers, and `objdump -p`
lists none of the four:

| | | |
| --- | --- | --- |
| `0x1402646a0` | init | `InitializeCriticalSection` through `[0x1405a7ee8]`, then `0x200` bytes zeroed at `+0x28` |
| `0x140264660` | delete | `DeleteCriticalSection` through `[0x1405a7ef0]` |
| `0x140264720` | enter | `GetCurrentThreadId` through `[0x1405a7d30]`, then `EnterCriticalSection` through `[0x1405a7ef8]` |
| `0x1402647d0` | leave | `LeaveCriticalSection` through `[0x1405a7f00]` |

**So the object is `0x230` bytes**: the `CRITICAL_SECTION` itself at `+0` — `0x28` on x86-64 — the enter's
name at `+0x28`, which `0x140264798` copies in under a `0x1ff` clamp and `0x1402647a4` terminates, the
enter's own number at `+0x228` from `r8d`, and the calling thread's id at `+0x22c`.

**Enter takes the object in `rcx`**, so reading the `lea rcx` before each of its 93 call sites says which
object every lock in the build is:

```sh
python3 -c '
import re
from collections import Counter
lines = open("th06nc.asm").read().splitlines()
held = [(0x1407e20c0, 0x1407e91c0), (0x1405542b0, 0x1405542e8), (0x1409fcc48, 0x1409fccb0)]
sites = inside = 0
unread = []
outside = Counter()
for i, ln in enumerate(lines):
    if not re.search(r"call\s+0x140264720$", ln):
        continue
    sites += 1
    where = None
    for back in range(1, 16):
        m = re.search(r"lea\s+rcx,\[rip[^]]*\]\s+#\s+(0x[0-9a-f]+)", lines[i - back])
        if m:
            where = int(m.group(1), 16)
            break
    if where is None:
        unread.append(re.match(r"\s+([0-9a-f]+):", ln).group(1))
    elif any(lo <= where < hi for lo, hi in held):
        inside += 1
    else:
        outside[where] += 1
print(sites, "call site(s);", inside, "inside a block held back;", len(unread), "unread:", unread)
for where, n in sorted(outside.items()):
    print(f"  0x{where:x}  {n} site(s)")'
```

Of the 93, **19 take an object inside a block a restore already leaves alone** — the sound layer's own
`+0x70`, reached as `[r12+0x70]` and so counted by the `lea rcx` this reads — **20 cannot be read at the
site**, `0x1400b9570` and the twelve in `0x1400f1xxx` taking a register the walk set and the seven in
`0x14026xxxx` taking the caller's argument or its `+0xd0`, and **54 take one of twelve statics**:

| | |
| --- | --- |
| `0x140301670`, `0x140301800` | `.rdata`, which no snapshot copies |
| `0x1405ca1d0`, `0x1405cb520` | `.data`, one site each |
| `0x1405fd520`, `0x1405fd780`, `0x1405fd9f0`, `0x1405fdc60`, `0x1405fdf50` | `.data`, the five `0x1400c70b6` inits in a row |
| `0x1407f9a20`, `0x140805b48`, `0x1409b3468` | `.data`, 8, 15 and 7 sites |

**The ten in `.data` are copied into a chapter and written back by a restore**, `.data` being
`0x14036d000`..`0x140c75884` whole, which is why `Game::live_handles` names them.

### The five queues the frame's effects are flushed through

**`0x1405fd454`..`0x1405fe180`**, a header and five queues, which `0x1400c7078` onwards builds: the five
locks initialized in a row at `0x1400c70b6`–`0x1400c70ed`, then `0x1400f3850` called on the pairs
`(0x1405fd4e0, 0x1405fd500)`, `(0x1405fdc20, 0x1405fdc40)`, `(0x1405fde90, 0x1405fdeb0)` and
`(0x1405fdef0, …)`, and `0x1400cf98c` onwards takes them back down. Each queue is its `0x230` lock with
a head, a tail and a count after it, so the five run `0x1405fd520`, `0x1405fd780`, `0x1405fd9f0`,
`0x1405fdc60`, `0x1405fdf50` — `0x230` to `0x270` apart — and the header from `0x1405fd454` holds
`0x1405fd480`, the flag that says the block is built, referenced from 35 places.

**And the bounds are where the references stop**: nothing between `0x1405fd2a0` and `0x1405fd454` is
referenced at all, and the fifth lock ends exactly at `0x1405fe180`, where a run of small fields the
media's own settings are begins.

**The frame waits on one of these locks every frame, and after a chapter goes back it waits for good.**
The sampler's stack, with the launch's own module bases taken off: `ntdll.dll+0x163fd4` under
`ntdll.dll+0x2fb7f`, `ntdll.dll+0x4016f`, `ntdll.dll+0x413e2` — the wait inside
`RtlEnterCriticalSection` — under **`th06nc.exe+0x262866`**, the instruction after
`call [0x1405a7ef8]` in the lock enter, under `th06nc.exe+0xc82cf`, which is the
`lea rcx,[0x1405fd9f0]` at `0x1400c9fa3` returned into, under `th06nc.exe+0x76ab7` — the flush of the
frame's queued effects — under `th06nc.exe+0x3beed`, the calc walk's tail, under orb's own frame. Eight
frames after a restore the frame's thread stopped coming out of that wait, while orb's own threads went
on logging for seconds. `--stress=120` restoring the chapter ten times over a run never hung, and
`--no-memory` — the snapshot narrowed to `.data` — hung the same way, which is what says the block is
`.data`'s and not the heap's.

## The renderer's COM objects

**Eleven statics between `0x1409fcc48` and `0x1409fcca8`**, which
`0x140271c30` releases one after another: `0x1409fcca8` and `0x1409fcca0` each have their `+0x370` and
`+0x378` called before `Release`, and then `0x1409fcc48`, `0x1409fcc98`, `0x1409fcc50`, `0x1409fcc58`,
`0x1409fcc60`, `0x1409fcc68`, `0x1409fcc70`, `0x1409fcc78` and `0x1409fcc90` are released in that order,
with the sixteen bytes at `0x1409fcc80` zeroed among them. So `0x1409fcc48`..`0x1409fccb0` covers every
pointer that release names. Whether the statics either side of it — `0x1409fcc38`, `0x1409fcc40`,
`0x1409fccb0`, `0x1409fccb8`, each referenced from elsewhere — are objects too is not read.

**The field is 384 by 448** and this exe holds the pair as literals rather than in `.data`: `0x140352d08`
and `0x140352d14`, loaded by the bullets' update at `0x140011b13` and the enemies' at `0x140038d3f`.
`.rdata` from `0x1403529e8` on is one sorted pool of every float literal the build has — `0.03125`,
`0.5`, `0.6667`, `1.0`, `2.0`, `2.5`, `3.0`, … `384`, `448`, `480`, `640` — the same shape th06c's is.

## The player

**A static object at `0x140549ff0`**, `0x7854` bytes, with a second piece of `0x2a5c` at `0x14055184c`;
`0x14006c090` — the hit test — is handed it as `rcx` by the bullets at `0x140011b41` and the enemies at
`0x140039351`.

| | |
| --- | --- |
| `+0x410`–`+0x7410` | **the player's own shots**, **eighty slots of `0x170` bytes**, which is where this block's bulk goes: a launch that said which of its pages a frame changed had every page from `+0x400` to `+0x1000` moving on 380 to 478 frames of 600 where `+0x7700`, the player's own fields, moved on 381 and nothing between `+0x1100` and `+0x7600` moved at all — the slots past the first few being ones a frame never has that many shots to fill. The update fills them from `0x14006b2f1`, walking `rbx` from `+0x410` with `add rbx,0x170` and `cmp r14d,0x50` and taking the first slot whose `+0x10` is nought; the stage's own register clears all eighty of those words at `0x140069d74`–`0x140069e3f`. The enemies' update is what takes life off one — `0x140039479`'s `sub DWORD PTR [rbx+0x234],r8d`, gated on `[rbx+0xbd] & 0x10` |
| `+0x410 + N*0x170` | **one shot**: `+0x10` a word that is nought where the slot holds nothing (`0x14006b300`, and `0x14006b35f` writes 1 into the slot it takes), `+0x12` its kind (compared with 2 and 3), `+0x8` and `+0xc` two floats its step, `+0x13c` and `+0x140` two floats where it is, `+0x160` and `+0x164` the half-sizes its hit box is grown by — 48, 42 or 32, written at `0x14006b5bd`, `0x14006b5cd` and `0x14006b5dd` by `+0x50`'s own id — and `+0x15c` a word the damage it does |
| `+0x408`, `+0x40c` | **the phase of the shot being fired, and the phase the update before it had**: `0x14006b38a` steps the pair every update — `[+0x40c]` up one and `[+0x408]` keeping what it was — and the walk of the eighty slots above takes the new value as the shot it emits, skipping altogether where the two are equal (`0x14006b2c6`). `0x14006ae14`'s `test r12b,r11b` is what starts a cycle: `r12d` is 1 from `0x14006a769` and `r11d` is the composed word from `0x14006aa94` with no call between them, so this is the word's own `0x1` — and where the pair is idle and `[[0x140abae28]+0x36b0]` is below nought, `0x14006ae30` writes `0` into `+0x40c` and `-999` into `+0x408`. `0x14006b39e` ends the cycle at thirty and `0x14006b3b4` puts `-999` and `-1` back, which is idle; the stage's register writes the same pair at `0x140069e46` |
| `+0x7730`, `+0x7734` | **where the player is**, which the move at `0x14006aace` writes and the laser test reads as the statics `0x140551720` and `0x140551724` |
| `+0x7710`, `+0x7714` | what a frame's movement is scaled by on each axis — `mulss` at `0x14006aaa6` and `0x14006aab6` |
| `+0x7750` | which way it is being moved, 1 to 8, written by the walk at `0x14006a784` onwards |
| `+0x785c` | whether the focus is held, off the word's `0x4` |
| `+0x7860`, `+0x7864`, `+0x7868`, `+0x786c` | **the four speeds**: straight, straight focused, diagonal, diagonal focused — the walk at `0x14006a854` onwards picks one by the direction and the focus |
| `+0x774c` | **the radius a hit is measured against** — see below |
| `+0x7754` onwards | **sixteen boxes of `0x10` bytes**, which `0x14006bfd0` walks before the player's own: a hazard inside one is answered 1 rather than 2 |
| `+0x7898` | **which state the player is in**, one byte: nought is the only one a hit lands in (`0x14006c1b5`), 1 is what the register and the walk-in write, 2 is what the hit writes (`0x14006c201`), 3 is what `0x14006a6b3` writes as the invulnerability begins |
| `+0x773c` | **the frames a bomb still takes the hit back in**: the hit writes 8 at `0x14006c20e`, and the laser's own hit the same at `0x14006c45b`; taken one off a frame of the hit's state at `0x14006a21b`, and the frame it reaches nought is the frame the death is made final — `0x14006a22a` steps the count at `0x14053cab0`; `0x14006a0e2` is the bomb refusing to fire where it is nought |
| `+0x9ec8` | **whether a bomb is going off**, one byte: `0x14006a1a4` writes 1 as the bomb's own start runs, `0x140069e9e` clears it as the stage's register leaves the player, and `0x14000d8fc`, `0x14000e2a7`, `0x14000e9ca` and `0x14000ef47` — one per shot — clear it as each bomb finishes. Read at `0x14006a0b8`, the press's own gate, and at `0x14006b2d2`, `0x14006b54d`, `0x14006b7e7`, `0x14006b856` and `0x14006ba40`, the hit test's |
| `+0x7854`, `+0x7858` | a `Timer`: `+0x7858` is **the frames a state has left** and `+0x7854` the `-999` every state writes beside it |
| `+0x7718` | **a ticker that thins an effect out**, one byte: `0x14006b6ea` steps it up for a bullet the player is inside the reach of and `0x14006b7c0` for an item it takes, and the only reads of it are the two `test`s beside those — `0x7` and `0x3` — which let the call of `0x14002af80` through one time in eight and one in four. **It is not the graze count**, which is the manager's own `+0xd26c` — see *The game manager* |
| `+0x9ed0`, `+0x9ed8` | **the two functions the bomb of the shot in hand is run by**, a pointer each: the stage's own register picks them at `0x140069e65` and `0x140069e88` out of the table at `0x140426eb0`, two pointers apiece, indexed by the shot and the character it reads at `0x140069e51` from `0x14053cad0` and `0x14053cad1` as `shot + character*2`. `+0x9ed0` is the one called, both by the bomb's own start at `0x14006a1c5` and every frame the bomb is out at `0x14006a0c1`; `0x14006a121` refuses the press where it is null |
| `+0x9f20` | **how many frames this bomb lasts**: the start writes `0x3e7` at `0x14006a1bb` and each shot's own function writes its own over it — `0x12c`, `0x8c`, `0xfa` and `0x12c` at `0x14000da4e`, `0x14000e3f7`, `0x14000eb4b` and `0x14000f0b7` — and `0x14000d8e4` and its three fellows compare the clock below against it, clearing `+0x9ec8` where it has run out |
| `+0x7ac4`, `+0x7ac0` | **the bomb's own clock and the frame before's**: `0x14000e0d7` writes the clock a frame on and `0x14000e0cf` writes what it held into `+0x7ac0` beside it, which is the pair the four bombs' own steps are read against. The start writes `0` and `-999` at `0x14006a1ab` and `0x14006a1b1` |

### The 270 frames of a stage nothing can reach its player in

**The same 270 th06c has**, read off this exe rather than carried over:

| | |
| --- | --- |
| `0x140069b2c` | **the register**, gated on `[0x14053d405]`: where that byte is nought it leaves the state and the count at nought, and otherwise the state **1** and the count **0x78** — `mov eax,0x78; mov cl,1` at `0x140069b3b`, written at `0x140069b42` and `0x140069b4f` with `-999` into `+0x7854` |
| `0x14006a4b7` | **the walk-in's own gate**, `cmp ecx,0x1e` — thirty frames — and past it `0x14006a4ce` writes the state **1** again with the player put at `0x43400000`, `0x43c00000` (192, 384) |
| `0x14006a6a9` | **and the gate that hands it over**, `cmp r8d,0x1e`, past which `0x14006a6b3` writes the state **3** and `0x14006a6cf` the count **0xf0** |
| `0x14006a37e` | **and state 3 counts down to nothing**: the state held against 3, `0x14006a391` takes one off the count and `0x14006a3a1` writes nought over both where it runs out |

**So the hit test cannot reach the player for 30 + 240 = 270 frames of every stage**, `0x14006c1b5` gating
a hit on that byte being nought.

**And the update branches on that byte twice, which is what a `--clear` run of this game never gets past
the first stage's boss for.** `Th06nc::make_invulnerable` writes the state **3** and the count `0xf0`
before every one of the game's updates while `--clear` is on — `runtime`'s own `fast_clear && in_game`,
which with `--speed=1` is the whole of what that flag still does to a run besides refusing the score file
and turning the chapters off. So `0x14006a37e` takes the state-3 arm every frame, the count never runs
out, and the update goes through `0x14006a703` — the invulnerability arm — rather than `0x14006a745`.
**Both arms then meet at `0x14006a753`, which reads the byte again** and branches on it twice:
`0x14006a773` compares it with `bl` and `0x14006a77b` with `r12b`, either jumping to `0x14006ae58`, and
anything else falls through to `0x14006a784`. **`r12b` is 1** from `0x14006a769` and **`bl` is 2** from
`0x14006a1e9`, which is on the straight path — `0x14006a1e2` reads the state byte into `eax` two
instructions before it and `0x14006a205` compares the pair at once — so the two that skip are **the hit
and the walk-in**, and **nought and three are steered**.

**And the fall-through is the player's own steering.** `0x14006a784` writes `[player + 0x7750]` and the
tests that follow are of `r11b` against `0x10`, `0x20` and `0x40` — `Th06nc`'s own `button::UP`, `DOWN`
and `LEFT` — so `r11b` is the word a run is played with, read from `[0x140abae80]` at `0x14006a1db`, and
`+0x7750` is a direction out of it bounded by `cmp ebx,0x7` at `0x14006a7d1`. Its last instruction
`0x14006ae50` writes `[player + 0x9f34]` and falls into `0x14006ae58`, so both paths run the tail and the
branch decides only whether the steering happens.

**So `--clear` standing still at stage 1's boss card is not this branch at all.** `make_invulnerable`
pins the state at 3, 3 is on the steering side of the pair, and a run under the flag is steered every
frame exactly as a playing one is.

**And it is not the flag either, which two launches settle: what stood still was the recipe.** The launch
that read `script=5093` with the frames climbing past it was inside its own
`wait_until: { frames: 1800 }` — a step that presses nothing, so the run stands where it is and the
stage's script does not advance. Driven again and read beside `autoplay: step N of M`, the boss's life
went `7486` down to `798` and stopped **on the frame that step began**, with `chose=0x0000` on 5,404 of
the frames after it against `0x1000` on twelve. And driven with a recipe that asks the run *past* her —
`play_until: { stage: 2, chapter: 1 }` — the same flag took it through the card, into stage 2 and on to
`stage 2 chapter 2 at frame 2499`. So the flag finishes a card, and a state line standing still is a
recipe's business as often as a fault's.

### The hit test, which is a circle and not a box

**`0x14006c090` is what every hazard asks about the player**: `rcx` the player, `rdx` the hazard's
position, `r8` its size, `r9b` whether it is round.

- `0x14006c096`–`0x14006c0f4` works the hazard's edges out — position plus and minus half its size — and
  hands them to **the sixteen boxes** at `0x14006bfd0`, which answers 1 where one holds the hazard.
- **Round** (`r9b` set, which the bullets pass — `movzx r9d,r12b` with `r12d` 1 at `0x140011b39`):
  `0x14006c122` takes the smaller of the size's two floats, halves it, adds `[player+0x774c]`, squares
  the sum and compares it with the squared distance between the two centres.
- **Not round** (the enemies pass nought at `0x140039360`): `0x14006c15e` onwards clamps the player's
  position into the hazard's box — the nearest point of the box — and compares the squared distance to it
  with `[player+0x774c]` squared.
- Either way a hit is `0x14006c201` writing 2 into `+0x7898`, 8 into `+0x773c`, and the answer 2.

**So the player is a circle here** where th06c's is a `1.2` by `1.2` box, and orb hands the rules the
radius as a half on each axis and a bullet's smaller half on each of its — boxes that hold the circles
the game tests, so a move the model calls clear is clear of them.

**And a laser's test is `0x14006c2b0`**, the same turn-about-the-origin th06c's is: `sin` and `cos`
through `0x1402efff0` and `0x1402ef700`, the player's position off the statics `0x140551720` and
`0x140551724`, and the size halved with `0x140352a44`'s `0.5` at `0x14006c36a`.

### The shots' own test, which runs the other way round

**`0x14006b430` is what an enemy asks about the player's shots**: `rcx` the player, `rdx` the enemy's
position, `r8` its size, `r9` a byte it sets where one of them reached — and it answers, in `eax`, the
damage the whole eighty did.

- `0x14006b43f`–`0x14006b4a5` works the enemy's edges out — `[r8]` and `[r8+4]` halved by `0x140352a44`'s
  `0.5`, plus and minus `[rdx]` and `[rdx+4]` — into `xmm8` through `xmm11`.
- `0x14006b4d0` then walks the eighty slots, `rbx` from `player + 0x550` with `add rbx,0x170` and
  `sub rdi,r12` counting `0x50` down: `[rbx-0x130]` nought skips the slot, and a slot whose kind at
  `[rbx-0x12e]` is neither 1 nor 2 is skipped as well.
- `0x14006b4f4`–`0x14006b547` is the box test, the shot's place at `+0x13c` and `+0x140` grown by its
  half-sizes at `+0x160` and `+0x164` against those four edges, and `0x14006b55c` adds the slot's own
  `+0x15c` into `r14d` where it holds — **a third of it where a bomb is out**, `0x14006b561`'s
  `imul 0x55555556` under `[player+0x9ec8]`.
- **A shot that lands is then spent or turned**: kind 2 has its damage quartered at `0x14006b586` and
  floored at one, picks a new pair of half-sizes at `0x14006b5bd`, `0x14006b5cd` or `0x14006b5dd` — 48,
  42 or 32 — by `+0x50`'s own id, and every sixth of `[rbx-0x13c]` calls the effect at `0x14002af80`;
  kind 3 steps `[player+0x7718]` and calls the same one time in eight; anything else is slowed by
  `0x140352a04` and put to kind 2 at `0x14006b6cb`.
- `0x14006b742` onwards is a second walk, thirty-two slots of `0xc` bytes from `player + 0x184` with a
  count at `player + 0x380` — the same box test, `add DWORD PTR [rdi],eax` scoring each hit, and the byte
  in `r9` set where one reached while a bomb is out.

## The bullets, in `0x140436ef0`

The update is `0x140010fa0`, and the array is read off its own loop:

| | |
| --- | --- |
| where the array starts | `+8` — `lea rbx,[rcx+0x8]` at `0x140010fb6` |
| how far apart | `0x620` — `add rbx,0x620` at `0x140011ca8` |
| how many | `0x280` — `cmp r14d,0x280` at `0x140011cc9` |
| how many are live | `+0xf5008`, counted up at `0x1400110ab` |
| one bullet's step | `+0x8`, three floats, added into the position at `0x140011191`–`0x1400111af` |
| one bullet's place | `+0x30`, three floats — `lea rdx,[rbx+0x30]` at `0x140011b3d` |
| **which of five states it is in** | `+0x44`, a word — `cmp WORD PTR [rbx+0x44],r13w` at `0x1400110a0`, nought being a slot nothing is in |
| the size the test is given | `+0x5f4`, two floats — `lea r8,[rbx+0x5f4]` at `0x140011b32` |
| whether it is armed | `+0x618`, a byte `0x140011b25` gates the test on |
| its angle and speed | `+0x40` and `+0x24`, out of which state 1 rebuilds the step at `0x14001122f` |

**The state decides what its step is scaled by, and the five are th06c's**: states 2, 3 and 4 divide one
by `2.0`, `2.5` and `3.0` — `0x140011138`, `0x14001114e` and `0x14001115f` load `0x140352ad0`,
`0x140352ae8` and `0x140352b08` — and states 1 and 5 add the step whole. Only state 1's path reaches the
test at `0x140011b48`.

## The lasers, in that same static

**`0x140436ef0 + 0xf5018` onwards**: the update walks sixty-four of them from `0x140011cdc` with its
pointer `0x14` into each — `lea rbx,[r15+0xf502c]` — `0x298` apart (`0x140011e5f`), `mov edi,0x40`.
Read off that pointer and written down from the struct's own start:

| | |
| --- | --- |
| `+0x4` | **how wide it is**, halved at `0x140011daa` into the size the test is handed |
| `+0x8`, `+0xc` | the frames it has been in its phase, the pair a `Timer` is |
| `+0x10`, `+0x14` | **how far along it runs**, near end and far: the far end grows by `+0x26c` a frame at `0x140011d4d`, and the near end follows once the two are `+0x288` apart |
| `+0x18` | how many frames it fades over |
| `+0x260` | **which way it points** |
| `+0x264` | flags, `0x1` tested at `0x140011ee1` |
| `+0x268` | how many frames it fires for — `0x140011fb8` compares the frames with it and moves the phase on |
| `+0x26c` | how much further the far end reaches each frame |
| `+0x270` | **how many frames its warning runs for**, compared at `0x140011f5f` |
| `+0x274` | whether the slot is in use — `cmp BYTE PTR [rbx+0x260],r13b` at `0x140011d40` |
| `+0x278` | **where it is fired from** |
| `+0x288` | how long it is once grown |
| `+0x290` | its phase: nought the warning, 1 firing, 2 fading, and past that `0x140011e3d` turns it off |

**The first fields are th06c's and the rest sit `0x20` further on**, the struct being `0x20` bytes
longer. **And only the firing phase hits**: the one call of `0x14006c2b0` is at `0x140011fb3`, on phase
1's branch — the warning at `0x140011ee1` and the fade at `0x140011fea` make none.

## The stage's background script, and the clock it runs on

**`0x140079360` is its job**, the one `0x14007a4a0` registers at priority 6 on the static `0x1405547b0`
(`0x220` bytes, memset at `0x14007a4c2`) from inside the stage's build. Of that object:

| | |
| --- | --- |
| `+0x24` | which instruction it is on, the index `0x1400793a2` reads |
| `+0x64` | the stage, which `0x14007a4a0` is handed in `ecx` and writes at `0x14007a4f3` |
| `+0xb4` | **its own clock**, which `0x140079428` holds each instruction's own time against — `cmp DWORD PTR [rsi],eax; jl` leaves the walk where the clock has not reached it |
| `+0x1f0` | the script, whose instructions are `0x14` bytes each: the time at `+0x0`, the opcode as a word at `+0x4`, and three dwords after it |

**Six opcodes**, `0x1400793f2` refusing anything past five and `0x140079b68` being the table — and what
its cases do is the background: `0x140098730` with a colour out of one dword's three bytes and
`0x140098870` with two floats. So this is the stage's *background* script and not the script the enemies
come out of, which is the one a midstage boundary is counted on — the next section.

## `0x14053d414`, the byte that changes what a run starts with

**One byte the run's own start writes, and it is which of the game's two modes the run is in** — two
pictures settled that: **screen 14 is 「開始するモードを選択してください」 over 標準モード and
チャレンジモード** (see the table of the screens), and a launch that decided the second started a run whose
HUD reads **`Challenge Mode`** over `NORMAL`. It matters because it moves the numbers a run begins on,
which is what a slot name would have to tell apart — see `Th06nc::run_slot`, which names it not yet and
says why.

**It picks between two names.** `0x140075137` loads it, `0x14007514e`'s `neg al; sbb rcx,rcx; and ecx,8`
turns it into nought or eight, and `0x14007515d` takes the pointer at `0x14034ed50 + that`:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x1402f5000+0x2bf000        # .rdata
for a in (0x14034ed50, 0x14034ed58):
    p = struct.unpack_from("<Q", d, at(a))[0]
    o = at(p); print(f"{a:#x} -> {d[o:d.index(bytes(1), o)].decode()}")'
```

— `Standard` at `0x140351fd8` and `Challenge` at `0x140351100`.

| | |
| --- | --- |
| `0x14004f348` | **where it comes from**: a `setne` off `[0x140c6dfdb]`, one of the 44 bytes of `th06.cfg` that sit at the supervisor's `+0x434`, inside `0x14004f320` — the numbers a run starts with, called from screen 25 and from nowhere else |
| `0x14004ef3a` | and a replay's own start sets it by `sete` out of the header — the `+0x6` of 1 that header note names |
| `0x14006cfcc`, `0x14006d97f` | the same `sete` off the same field on two more playback paths, each with `[header + 0x8]` going into the difficulty just above it where `[header + 0x6]` is not 3 |
| `0x140053b90` | clears it |

**And `[0x140c6dfdb]` is written by screen 14 alone, on both of its edges** — the case runs from
`0x14004d44d` to `0x14004d6c2`, screen 22's beginning at `0x14004d6c3`, and the two pairs of writes are
inside it:

| | |
| --- | --- |
| `0x14004d553`, `0x14004d55c` | **the cancel's**: `0x14004d52a` tests `0x200` in `[0x140abae80]` against `[0x140abae84]`'s for the edge, `0x14004d547` reads `[front end + 0x28]` — the cursor — and writes **1** where it is 1 and **0** where it is nought, writing nothing for anything else; then `0x14004d568` puts the state to `0xd`, the shot select, `0x14004d56e` clears the frames it has been up for, `0x14004d57f` sets its cursor from `[0x14053cad1]`, the shot, and `0x14004d590`'s loop fills `0xad` entries of `[front end + 0x96ca]` |
| `0x14004d65f`, `0x14004d668` | **the decide's**, the same three lines off the same cursor after `0x140078290(0x1405542b0, 10)` queues the sound; then `0x14004d6a0` writes `0x14053cad4`, the stage a run starts at — **6** where the difficulty at `[0x14053d410]` is 4 or more and nought otherwise — and `0x14004d6af` calls `0x14004f1e0(front end, 14, cursor)`, which puts the state to `0x18`, screen 24, and writes 14 into `[front end + 0x168cc]`, the screen it came from. Where `[front end + 0x168d1]` is set — a practice — `0x140057020` is called instead |

**So the choice is in that byte from screen 14 onwards, which is before screens 24 and 25 and before the
run**: `0x14004dc06`, screen 25's own, is where `0x14004f320` turns it into `0x14053d414`, and
`0x14004778c` writes the 44 bytes back to `th06.cfg` when `main` returns.

**And what `th06.cfg` holds is the last *exit's* choice**, which is why a probe read nought there after a
launch that played a Challenge run: the byte is written to the file only at `0x14004778c`, on the way out
of `main`, and every launch after that one played a Standard run. The copy beside the exe read
`00` at byte `0x17` with `01` at `0x15` when this was written — the difficulty last chosen being Normal
— so nothing in the file disagrees with the reading. Reading byte `0x17` straight after a Challenge
launch has exited is what would say it outright, and it is worth one launch rather than a paragraph.

**The 44 bytes' own offsets are the object's less `0x434`**, which is the trap that probe fell into:
`+0x44b` is byte **`0x17`** of a 44-byte file and `+0x454` is byte `0x20`, and a probe that seeks
`0x44b` in it finds nothing at all.

**The lives and the bombs a run starts with, read in full**, which is the reason to care and does not
test this byte at all:

```text
14004dbf5: cmp  DWORD PTR [0x14053d410],0x4   ; the difficulty; 4 is the Extra
14004dc09: je   0x14004f37c                   ; -> 0x302
14004dc0b: test dl,dl                         ; dl = [front end + 0x168d1], the run asked for no name
14004dc0d: jne  0x14004f37c                   ; -> 0x302
14004dc0f: test ecx,ecx                       ; ecx = [0x140c6dfdb]
14004dc11: je   0x14004f385                   ; -> neither written; the config's own lives and bombs stand
14004dc13: mov  BYTE PTR [0x140549d40],0x2    ; two lives, the config's bombs
14004dc1c: mov  WORD PTR [0x140549d40],0x302  ; two lives and three bombs
```

So what moves the numbers is `[0x140c6dfdb]` itself, the same byte `0x14053d414` is the `setne` of.

`0x14053d414` is read from twenty places besides: the stage's build at `0x14003c328` and `0x14003c4ad`,
the player's at `0x14006a2b8` and `0x14006a594`, the items', the bombs', the score file's at
`0x14006cfcc` and `0x14006d97f`, and the run's end at `0x14007bcfc`.

## The clock the enemies' own script runs on

**`enemy manager + 0x10c0bc`, `0x140bfa16c`**, which is the clock a midstage boundary is counted in: it
moves the same way however the run goes.

**The enemies' job steps it at its own end.** `0x140038cd0` takes the manager in `rcx` and keeps it in
`r12`, walks the `0x100` slots, and its last four instructions before the epilogue are
`mov ecx,[r12+0x10c0bc]`, `mov [r12+0x10c0b8],ecx`, `inc ecx`, `mov [r12+0x10c0bc],ecx` at `0x140039ac7`,
`0x140039b11`, `0x140039b19` and `0x140039b1b`. So `+0x10c0b8` is what the clock was the frame before,
which `0x140001328` starts at `-1` while `0x140001322` clears the clock itself.

**And it is the script's clock and not a frame count**, because the spawner holds it against the
timeline: `0x140037d9b` reads it, `0x140037daa` compares it with the word at the timeline's cursor —
`+0x10c128`, the pointer `0x140037cf7` sets out of `[0x140abada8]` and `0x140037d21` falls back to
`[0x140abad90]` for — and a `jne` skips the spawn where the two differ. `0x140037d01` and `0x140038762`
write the clock out of the script, which is how a timeline jumps.

The manager's other tail fields, all reached through that base: `+0x10c0c0` and `+0x10c0c8` the two
pointers `0x140038a90`'s registration is handed in `rcx` and `rdx` (`0x140038ad2` and `0x140038ab6`),
`+0x10c0d0` a dword the job holds against nought at `0x14003940b`, `+0x10c0d4` a byte it holds against
nought at `0x14003943b`, and `+0x10c0dc` the byte `0x14002789d` sets and `0x14006c1c8` clears.

## The conversation, `[0x140abae28]`

**The object a conversation runs in, behind the pointer at `0x140abae28`** — the one the stage's own asset
load fills: `data/msg1.dat` through `data/msg7.dat` are chosen one per stage at `0x14003f163` onwards and
the chosen name is handed to the loader `0x140040460` at `0x14003f412`. That loader frees and refills two
pointers of `[its own + 0x38]`, which is this same object: the script's bytes into `+0x38f8` through
`0x14003a0c0(name, 0, 0)`, and a second buffer into `+0x3b58` which `0x140040548` writes.

| | |
| --- | --- |
| `+0x36b0` | **which line of the conversation is being shown**, and `-1` for none. `0x1400405c0` — the dialogue's own start, called from `0x140038723` — writes the line it was asked for at `0x1400406c4`, and `0x1400401f3` and `0x140040510` write `-1` as one ends. **And the game reads it as the question of whether a line is up**: `0x14006a0d6` refuses the bomb press while it is nought or more, `0x14006ae20` gates the player on the same compare and `0x140038757` the script's own side of it. This is what `Th06nc::a_line_of_dialogue_is_shown` and `State::in_dialogue` answer |
| `+0x38f8` | the script's own bytes, whose first dword is how many lines it has — `0x1400405d7` refuses a line at or past it |
| `+0x3b58` | the second buffer, indexed by the line at `0x1400406ca` |
| `+0x3670`, `+0x3674` | **the first two entries of a colour table**, `0xffffff` and `0xfff0f0`, which the start writes at `0x1400406fe` and `0x140040708`. It is a table because `0x1400409e6` reads it as `[object + 0x3670 + n*4]`, `n` being the word at `+0x4` of the line's own record — and **four tables go together**: `0x1400409e6`, `0x1400409d2`, `0x1400409de` and `0x1400409ca` read `+0x3670`, `+0x3680`, `+0x3690` and `+0x36a0` at that same `n` and hand all four to `0x140007ba0` at `0x1400409f2`, the text draw the Spell Practice screen calls with `0xffffff`, `0x10101` and `0x40202` at `0x140053636`. So the four are one colour each and `n` is who is speaking |
| `+0x3680` | the second of those four, whose first two entries the start writes as one qword at `0x1400406d3` |
| `+0x3668` | **a pointer the object walks**, written at `0x14004051b` and `0x1400406df` and read at `0x14004081c`, `0x140040836`, `0x140040bd0`, `0x140040e33` and `0x140040f3c` |
| `+0x3420` | **how many characters of the line are out**: `0x140040d24` reads it, `0x140040d2a` holds it against the line's own length at `[record + 4]` and `0x140040d34` steps it by one, and `0x140040d17` is the eight a press has to wait for — see *What advances a line* below |
| `+0x3424` | written `0xf` as a dword at `0x1400406f4` and read as a **byte** at `0x140040a8c`, so what it holds is fifteen and not four bytes of it |
| `+0x36f0`, `+0x3810` | **one table's first two entries, a word apiece at a `0x120` stride**: `0x3810 - 0x36f0` is exactly `0x120`, and `0x140040a19` writes `[+0x36f0 + n*0x120]` with `n` the word at `+0x6` of the line's own record — the same shape as the colour tables above, whose first two entries the start also writes. `0x1400406e6` and `0x1400406ed` put `-1` into those two, and `0x1400409a9` holds entry one against `si`, `0x1400409b0` skipping the line where `si` is the greater. And the word is **`0x731 + n`**, `r15d` being loaded with `0x731` at `0x1400408a5` and never touched again — a run of ids from 1,841 up, one per drawable, which is the shape a script id of this game's own drawing has. `si`, the counter entry one is held against, starts at nought at `0x140040874` |

**And th06c's own handle on the line does not carry here.** That build's is a panel field its frame gates
the playback's fast-forward on; this build's gate reads four other things — `0x14003d910` onwards asks
`[0x14053d3dc]` set, `[0x14053d400]` clear, `[0x140c6dfbc]` exactly 2 and `0x8000` of `[0x140abae80]` —
so the field above was read from this exe and not taken across.

### What advances a line

**A press, and the press is an edge of three bits together.** `0x140040cfa` masks `[0x140abae80]` with
**`0x301`** — the shot's own `0x1`, the decide's `0x100` and the cancel's `0x200` — `0x140040d08` masks
`[0x140abae84]` the same, and the line moves on only where the two differ and the first is not nought:
a bit that went down is no press. `0x140040d17` then wants `[+0x3420]` at eight or more, so a line
younger than eight characters ignores the press, and where the edge is missing `0x140040d24` steps that
count instead — one character a frame until it reaches `[record + 4]`, the line's own length.

**And `0x8` held skips the conversation outright**, no edge wanted: `0x140040ced` and `0x140040813` test
that bit — the one binding of `0x1400136f9`, and *which* key that is the exe does not say; see below —
and jump straight to `0x140040bd0`,
the same place an edge reaches. `[+0x3900]` gates both, so it is what says a conversation may be
skipped at all.

**Which is why a run the rules drive spends thousands of frames in one**: `orb_bot::steer`'s
`SHOT_LET_GO_EVERY` lets the shot go for one frame in sixty, and a *release* is not the edge this wants —
the press on the frame after it is. A Normal run driven by the rules read the line up at stage frames
5,125 and 5,900 of its first two stages with the script clock standing still throughout, and each stage
took about twice the frames its own script did. **The line itself is read**, which three launches before
this one had not managed: `f6000 … stage=0 frames=5125 script=5093 … dialogue` and twenty more samples
like it, over two stages.

**And `0x8` is not orb's to hold**, which is why the shot is what was changed instead: the bit is built at
`0x1400136f9` out of the binding byte `[0x140c6e09d]` — the chain reads a binding, then the keyboard
state at it, then the *next* binding, so the mask stands one load behind the byte it is about — one of a
block `0x140013375` fills whenever
`GetKeyboardLayout` differs from `[0x140c6e180]` — a block past `.data`'s raw bytes, so the exe says
nothing about which key it holds. `Th06nc::BUTTONS` therefore has no `"skip"`, `orb_bot::steer`'s
`Keys::skip` is nought for this game, and the release is the whole of what carries a line.

**So the rules let the shot go every ten frames while a line is up** —
`orb_bot::steer::SHOT_LET_GO_EVERY_IN_A_CONVERSATION`, ten because `0x140040d17` wants eight characters
out and a cadence under that spends its edges on a line too young to take them. `th06nc/sight.rs` reads
the line for it, which it did not while where the game kept it was unread, and
`a_conversation_of_th06nc_pressed_past` in `orb-e2e` is what holds the cadence.

**And it halves the conversation, measured either side of the change.** Two launches drove
`the-lines-a-conversation-costs.yaml` into stage 1's own conversation — the one its script clock stops at
`5093` for — and the `dialogue` flag stood

| the shot let go every | from stage frame | to | frames | press edges |
| --- | --- | --- | --- | --- |
| 60 | 5,091 | 5,477 | **386** | 6, which is the cadence and not a count — that launch was given `--no-resume` and wrote no word a frame |
| 10 | 5,092 | 5,283 | **191** | 19, counted, exactly ten frames apart in the per-frame `resume:` words |

Half and not a sixth, because `0x140040d34`'s own typing is a character a frame up to `[record + 4]` and
no cadence shortens that. **And a stage running to twice its script frames is the fight and not the
conversation**, which the same launches say: stage 1's script stops at `5096` and the state lines read
`frames=8240 script=5096 boss=true` for thousands of frames after the line is gone, a boss attack holding
that clock. The earlier reading that blamed the conversation for the whole gap was reading the fight.

**And the start is where the field a line must not be reached in is cleared too**: it walks the `0x280`
bullets at `0x140436f3c`, `0x620` apart, writing 5 into each whose word is neither nought nor 5, and the
sixty-four enemies at `0x14052c198`, `0x298` apart, putting each live one's byte to 2 with `-999` beside
it.

## The enemies

**The static `0x140aee0b0`**, whose update is `0x140038cd0`: the live count in the manager's first dword
(`inc DWORD PTR [r12]` at `0x140038dcd`), the array at `+8`, `0x10b0` apart (`0x140039aae`), two hundred
and fifty-six of them (`cmp r13d,0x100` at `0x140039aba`).

| | |
| --- | --- |
| `+0xb0` | where it is, three floats — `lea rsi,[rbx+0xb0]` at `0x140038dd1`, handed to the test |
| `+0xbc` | the flags the walk gates on: `cmp BYTE PTR [rbx+0xbc],0; jge` at `0x140038dc0` skips a slot whose top bit is clear |
| `+0xbd` | **and whether its body is run against the player**: `0x140039312` requires `0x4`, `0x140039338` runs the test only where the low two bits are both up, and `0x14003931a` refuses where `+0xbe`'s `0x8` is set |
| `+0x2c0` | **the size that test is given**, three floats, each multiplied by two thirds — `0x140352a60` at `0x140039363` — before `0x14006c090` halves what it is given, so a hit is a third of this on each side |
| `+0x234` | its life, which the body taking the player off it drops by ten at `0x1400393a3` |
| `+0x4` | **the frames its attack has run**, and `+0xc54` the frames it may: `0x140038870`, where `+0xbd`'s `0x8` is up, divides `[+0xc54] - [+0x4]` by sixty into the panel's countdown and ends the attack where the two meet — th06c's `0x1400202df` instruction for instruction, the limit at `+0xbb4` there. Put back to nought as the script moves the boss on, which is what says the next attack began and how far into a card the run is. Found by `objdump -d th06nc.exe \| grep -A2 'mov    eax,0x88888889' \| grep 'sub .*\[rcx+0x4\]'`, the one divide by sixty after a field's subtraction |
| `+0xbd`'s `0x8` | **the one the fight is with**: `0x1400396b0` and `0x1400398f0` test it and divide the life into the bar the panel draws. **Driven**: a probe recipe that waited on `boss: true` read it at stage frame 1,925 of the first stage, which is why four launches that asked inside 1,800 frames read `boss=` false on every line |

**And past the array**, which ends at `+0x10b008`, the manager's own fields — the cluster
`0x140bfa178`..`0x140bfa21c`, of which three are read:

| | |
| --- | --- |
| `0x140bfa180` | **whether a spell card is up, as which kind of card it is**: `0x1400278a4` writes the kind as one starts, and the card's end — the script's instruction at `0x140027a7f` — skips everything where it is nought, holds it against 1 at `0x140027a9b`, and writes nought into it at `0x140027c76` |
| `0x140bfa18c` | **whether the card up can still be taken**: `0x14002789d` sets it as one starts, the player's death at `0x14006c1c8` and `0x14006c40d` and the bomb at `0x14006a1cd` clear it, the card's end pays its bonus at `0x140027ac0` only where it is still set, and nothing clears it as the card ends — so it stays up through every attack after a card taken, which is why it is not what orb reads a card by |
| `0x140bfa190` | the dword the bonus is worked out of, `imul` at `0x140027ad8` and `add` at `0x140027af2` |
| `0x140bfa178` | a pointer the manager's own registration writes at `0x140038ab6` |

## The items

**A static at `0x140bfb2f0`**, walked by `0x140044240`: the array at `+8` (`lea r14,[0x140bfb2f8]` at
`0x140044264`), `0x160` apart (`imul rdi,rcx,0x160` at `0x140011b72`, where the bullets' update reaches
into it), five hundred and twelve of them (`cmp eax,0x200` at `0x140044ddd`). A slot whose first byte is
nought holds nothing — `0x1400443a0`. `+0x10` is where it is.

**Its kind is the signed byte at `+0x34`**, and `0x140044600` jumps through the table of seven at
`0x140045168` — whose cases are th06c's in th06c's order: 0 the power (`0x14004461a`), 1 the point item
(`0x14004492b`), 2 the big power (`0x140044a4e`), 3 the bombs stopped at eight (`0x140044c87`), 4 the
power written to its cap (`0x140044d3d`), 5 the 1up (`0x140044cb6`), 6 the one a bomb leaves behind
(`0x140044f36`).

## The menu a run out of lives is asked to continue from

**`0x14000bc20` is where a run that has lost its last life goes**, and it is the one machine that decides
between the continue and the screen a run ends at. The byte it is entered on is
`game_manager::IS_IN_RETRY_MENU`, `0x14053d401`, which the player's own death writes 1 into at
`0x14006a581` and every path out of this machine clears; `Th06nc::read_state` reads it as `paused`, a
frame with it up being a frame nothing of the run moved in.

**Four answers, in the order the machine asks them:**

| | |
| --- | --- |
| `[0x14053d405]` set and `[0x14053d3dc]` clear | a scene out of the table rather than a run: `0x14000bc61` writes 1 into `[0x14053d406]` and it returns |
| `[0x14053d404]` set | a stage practised on its own: `0x14000bcae` writes **7** into `[0x140c6dfbc]`, the supervisor's wanted state, which is the screen a run that finished ends at |
| `[0x14053d3dc]` set | a replay being played: `0x14000bccc` writes **8**, a run given up |
| `[0x140549d2c]` at 3 or more, or `[0x14053d410]` 4 or more | **no continue is offered**: `0x14000bd00` and `0x14000bd0d` leave for `0x14000c3f4`, which writes **7** — so a run past its third continue, and every run of the Extra, walks straight onto the screen a run ends at |

**And where none of those holds, the prompt goes up** and the machine runs on `[the run's own + 0xd94]`,
a phase of nought to four with `+0xb4c` as its own clock:

- **phase 3** is the continue taken — `0x14000be4e`, where `+0xb4c` has reached `0x1e`: it clears the
  retry byte and `0x14000bea6` steps `[0x140549d2c]` up by one, which is **the count of continues** the
  first answer above is capped against;
- **phase 4** is the run let go — `0x14000bd5c`, where `+0xb4c` has reached `0x14` and
  `[0x140c6e378]`'s `+0x34` reads 2: it clears the retry byte and writes **7** into the supervisor's
  wanted state.

**What the prompt shows is a picture**: `コンティニュ〜？` over two items with the first of them lit —
`the-continue-prompt.png` beside the copy, taken while a launch sat on it. **And it waits rather than
counting itself out**: one launch held it for four minutes with the stage's clock stopped at
`frames=2193`. **One tap of down and a decide at it took the continue**, the run coming back with two
lives and going on to `deaths=4 lives=1` — so which of the two items each is, and which bits move that
cursor, is not established; what is established is the way past it, which is the Extra.

## The ending, and the states a clear walks after it

**The ending is 33,204 updates long**, measured: a clear of Normal read `supervisor: state 2, wanting 10`
and then `10/10` — the one state `0x14004148d` asks for — with `run: the ending is running, with 6 live
and 6 bomb(s) in hand` beside it, and `ending skipped, 33204 frames run` sixty-eight seconds of wall
clock later. **Sixty-eight seconds and not nine minutes** because `runtime` runs the ending out inside
the frames it starts on: the whole of it cost that, an update of it being tens of microseconds.

**And what it reads its text out of is `data\th06ED.dat`, loaded into slot 5.** The ending's own build is
`0x14002d294` onwards: `0x14002d2c4` sets `[0x14053d403]`, the run-finished byte, `0x14002d2cb` sets
`[0x140c6db98]`, and then `0x14002d2e5` calls `0x14007cfd0(0x140c6db90, 5, "data\th06ED.dat")` — the
name at `0x1403501f8`. **The staff roll is three sheets beside it**: `0x14002d2fc`, `0x14002d313` and
`0x14002d325` call `0x140002440` with `data/staff01.anm`, `data/staff02.anm` and `data/staff03.anm` at
`0x140350208`, `0x140350220` and `0x140350238`, in slots `0x31`, `0x32` and `0x33` and with the ids
`0x620`, `0x627` and `0x62e`. So there *is* a roll of its own to tell from the ending, and what does not
tell them apart is the script field orb watches.

**An earlier reading of this had the wrong address**: `0x140019671` is nlohmann::json's own error
builder, appending `"while parsing "` — `0x14034f168`, four bytes' worth of slide away from the name
above. The two are a `.rdata` mapping apart and nothing else.

**And it was run out rather than stopped.** `ENDING_SKIP_LIMIT` is 54,000 updates and the loop used
33,204 of them, so what ended it is the ending's own flag going out — which is also what says **orb
cannot tell this game's staff roll from its ending**: the line reads `ending skipped` and not
`ending run out … where its staff roll begins`, `moved_on(script, state.ending_script)` never having
fired. For th06 that split is worth having, the roll being the part somebody watching would keep; here
the ending simply ends and the whole of it goes by in one loop.

**Then the supervisor goes 10 to 7**, the screen a run that *finished* ends at, and that screen opens in
**state 9** — the name entry. So **a clear walks `9, 15, 17`**: the entry, the offer its end item writes
at `0x1400727d5`, and the way out — because `Th06nc::skip_replay_prompt` puts 17 there, thirty
milliseconds after 15 stood. The four states between, `16, 10, 12, 14`, are the *replay-saving* path a
person takes, which `the-state-12-held.yaml` and `the-state-13-held.yaml` walk instead.

## The entry a name is typed into

**It is a state of the screen a run ends at**, reached the one way a heap object of this game's is: the
job it registers in the calc chain. The job a stage ends into, `0x14007b7f0`, allocates the screen's
`0xbb48` bytes at `0x14007bd98`, builds it with `0x14006f5d0`, marks it a run's own at `+0x9e99`, writes
its state at `0x14007bdde` — **nine** where `[0x14053d404]` is clear and seventeen where it is not, the
latter being a run asked for no name — and hands it to `0x14006f3f0`, which allocates the `0x40`-byte
element, puts the update `0x14006fa70` at its `+0x8` and the screen at its `+0x38`. The machine a run
over puts up builds the same screen at `0x14000cd97` and writes state **twelve** into it at
`0x14000cdc9` with `+0x9e9a` set — the ranking, see *The main menu's ten items*, where that machine is
read.

**And a picture says what state nine shows**: the **name entry over the ranking** — ten rows of the
difficulty the run was on with this run's own lit among them, the character grid under it and its `End`
item at the end — taken while a launch held it, `the-end-state-9.png` beside the copy. What brought the
screen up for it is an Extra run that stood still until it was out of lives, that being the one run this
game asks no continue about; the log read `result: the screen a run ends at is in state 9` and
`score: the run finished, and the screen it finished at is what writes`.

**The eighteen states are eight handlers**, four of them shared:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(18):
    off = struct.unpack_from("<I", d, at(0x140073ef4)+4*i)[0]
    print(f"state {i}: 0x{0x140000000+off:x}")'
```

| | |
| --- | --- |
| 0, 1 | `0x14006fbb0` and `0x1400702c5` — the screen's own first two |
| 2 | `0x140071095`, which is where `0x140073037` puts it once the file has been written |
| 3, 4, 5, 7 | `0x140071193`, and 6 the `0x140071184` nine bytes in front of it |
| 8 | `0x140071732` |
| **9** | `0x140071e9c` — **the entry the rest of this section is about** |
| 10 – 14 | `0x140072b05`, which splits them again: 10 at `0x1400739e8`, 11 at `0x140073842`, **12 at `0x14007324c` — the list of replay slots**, **13 at `0x140072d6f` — the second entry, whose confirm at `0x14007301f` is the one write of the score file**, and 14 falling through at `0x140072b34` |
| 15, 16 | `0x140073d93` — the way out, and 15 is what the entry's own end item writes at `0x1400727d5` |
| 17 | `0x14006fb0e` — the state a run asked for no name opens in |

**And what 10, 12 and 13 show is read off pictures**, `result-state10.png`, `result-state12.png` and
`result-state13.png` beside the copy, taken while a launch held each: **10 is
「今回のリプレイを保存しますか？」 over `はい` and `いいえ`** — the same question 15 puts up, still
drawn — **13 is the name typed for the slot**, the chosen row lit over the same alphabet grid state 9
has (a launch's read `No.01 ORB_ 09/21/26 Reimu Standard EXTRA`), which is the entry `0x140072d3a` takes
the address of at `+0x9e88` and whose confirm is the one write of the score file — and **12 is the list
of fifteen replay slots**,
a row apiece under `No.`, `Name`, `Date`, `Player`, `Mode` and `Detail`, with the one slot this copy
holds reading `No.01 ORB 09/20/26 Reimu Standard NORMAL` and `No.02` to `No.15` empty. So the three
questions in a row are whether to save (15), which slot to save into (12) and, where that slot already
holds a replay, 14 — which is the state `0x1400734f3`'s check of `T6RP` and `0x10f` reaches, and the
slots it indexes at `+0x9eb0` `0x78` apart are the rows of 12.

**The `Mode` column is the game's own mode**, `Standard` there against `Challenge` — a replay carries it
in its header's `+0x6`, see *The second table, and the replay list behind it*, and `0x14053d414` is what
a run is played under. **Which is also why 12 is not the ranking**: that name belongs to state 12 of the
*other* machine, `0x14000c960`'s, which `0x14000cdc9` writes — two machines with a state 12 each, and
this table is the screen a run ends at.

**And the name entry is for a run that made the top ten, which is what a launch found and the exe says.**
`0x14006e290(the list, the record)` answers **the rank** — the count of nodes its walk passed, `esi`
stepping once per record whose score beats this one — and `0x140072341` holds that against **ten**: under
it the entry stands, and at ten or more `0x14007234f` writes **`0xf`** into `[+0x9e94]`, the offer, with
`0x140072359` clearing `[+0x4450]` beside it. So a run outside the table is never asked for a name.

**Measured**: a launch let a Normal run die out, took the continue its prompt offered, let it die again,
declined the second prompt and then **pressed nothing at all for 3,600 frames**. The supervisor went
`2/7` and the screen came up in **state 15** and stood there — no state 9 on that launch or on the one
before it, where every Extra run that died out read 9. Two continues' worth of deaths is a score that
misses this copy's own table, and `0x14007bdde`'s nine is written all the same: what takes it away is the
rank.

**And the ranking put up from the menu is this same screen, opened in state 0.** A launch took the main
menu's item 5 — 夢の記録 — and read `front end: screen 12`, then `supervisor: state 1, wanting 6` and
`6/6`, and then **`result: the screen a run ends at is in state 0`**, `1` and `4`. So supervisor 6 builds
this object whether a run earned it or the menu asked for it, which is what `Th06nc::show_ranking` and
`result_screen::A_RUNS_OWN` are for: the byte at `+0x9e99` is what tells the two apart, a run's own screen
setting it and the ranking's build leaving it alone.

**And the job at `0x140009800` is not what that screen runs in.** The instrument said
`ranking: 0 frame(s) up, picking on 0x0` **once**, five seconds *before* the screen came up, and nothing
after — so `what_a_job_runs_on(0x140009800)` answered nothing for the whole of the ranking being up.

**What that job is instead is the ranking a *scene* puts up**, which a `--clear` launch that beat a card
of the Spell Practice read: the machine came up a second and a half after `run: the boss is gone with 878
life left` and counted `ranking: 1 frame(s) up` to `405`, with `[obj + 0xd98]` at nought the whole way —
which is `0x14000ca51`'s twelve-row branch. Its states are the six above, its way out through the
**cancel** at state 1, and that way out is the supervisor's 12.

**And 11 and 16 are states that wait for nobody.** A launch drove a Normal run that stood still, took
the continue its prompt offered and stood still again — which is what sets `[0x140549d2c]` — and the
screen it then reached walked **15, 16, 10, 11, 2** and went down: 16 arrived and handed on within half a
second, 11 within *sixteen milliseconds* of 10, and **2 is where the file has been written**,
`0x140073037` putting it there, with `score: score.dat opened as the game's own, write` beside it. So a
picture of either is that screen mid-transition and not a screen of its own. **And no state 9 stood in
that launch's log**, where the Extra runs before it all showed one — whether a run that has continued is
asked for a name at all is the next thing to read there.

**State 14 is a question with a live cursor**: `0x140072b34` pulses two colour words by `[+0x9e7c]`
being nought or not, waits for `[+0x4450]` to reach `0x14`, and then reads `0x140abae80` against
`0x140abae84` for `0x200` and `0x400` — which move it — and `0x100`, the decide. **And state 10 hands on
to 11** where `[0x140549d2c]` is set: `0x1400739fb` picks **nine or twelve** by that byte — `cmovne`
between the two — writes that one number into the same field of every drawable `0x120` apart, from
`+0x25a` to `+0x2bfa`, and writes 11 into `[+0x9e94]` where it picked twelve. **The number is a field of
each drawable and not a line of characters**, which is what an earlier reading of this had: the write is
one value into thirty-eight places at a fixed stride, the same shape as state 13's path setting `0xf`
into that field from `0x140072c0f` on. What the field selects is unread, and `[0x140549d2c]` is a byte of
the records — the clear records this game keeps are `0x140549d51` onwards — whose own meaning is unread
too.

**And state 14's decide goes to 13**, which is what puts the question in front of the name entry: with
the cursor at nought `0x140072bfb` clears the grace, the run of word writes from `0x140072c0f` on sets
`0xf` into a field of every drawable `0x120` apart, `0x140072d3a` takes the address of the name buffer at
`+0x9e88` and `0x140072d5b` writes **13** into `[+0x9e94]`. With the cursor anywhere else the decide
reaches `0x140073c31`, which calls `0x14006e9e0` — the queue of sound commands at `0x1405542c0` given
`0xa`, and then `0x14006e8b0` — and leaves the handler.

**And what state 13 does with the name is write it into the replay as well as the score file**:
`0x140072fa3` reads `[+0x9e7c]` as the character the cursor is on and `0x140072fc8` and `0x140072fd9`
write into `+0x9e88`, `0x140072fed` takes `replay/th6_%.2d.rpy`, `0x140073014` and `0x140073024` hand
`+0x9e88` on, and `0x140073043` writes the state the entry leaves for. **So one name goes into both
files**, and the two references to that path in this screen are `0x140072fed` — state 13's — and
`0x140073283`, the ranking's.

**What is left of it is what state 14's two items say.** The question is read and so is where each answer
goes; which of them is はい is not, and that is a picture of the screen. `Game::skip_replay_prompt`
declines until it is: writing 14's own "somewhere else" over a name entry somebody is standing at is
th06's own defect, and 14 is the state in front of the entry rather than behind it.

| | |
| --- | --- |
| `screen + 0x9e94` | **its own state**, read at `0x14006faa8` and jumped through the table above |
| `screen + 0x0` | **the grid cursor**, the object's first dword. `0x1400726d0` reads it, `0x1400726d8` takes the character at the grid, `0x1400726d3` compares it with `0x5e` — the space item, whose case writes `0x20` at `0x1400726e9` — and `0x1400726e3` leaves the typing for anything past it. Left and right step it by one at `0x1400725c0` and `0x140072640`, up and down by sixteen at `0x1400724e7` and `0x140072550`, each skipping a space of the grid |
| `screen + 0x9e70` | **the name, eight bytes.** `0x1400726f1` writes the character at `[screen + 0x9e70 + index]` |
| `screen + 0x9e7c` | **how many have been typed**, the index above — capped at seven by the `cmovge` at `0x1400726cd`, stepped up at `0x140072703`, stopped at eight by `0x1400726fe`, and reaching eight is what `0x140072710` puts the cursor on `0x5f` for. The constructor writes one into it at `0x14006f6dc` and other states use it as their own cursor |
| `screen + 0x4450` | the state's own frame count, which the entry reads no key before reaches thirty at `0x14007249a` |

**The grid is at `0x140351f40`** — `ABCDEFGHIJKLMNOPQRSTUVWXYZ.,:;\@abcdefghijklmnopqrstuvwxyz+-/*=%`
and the rest of the 96 — reached off the image base in `r13`, `[rax+r13*1+0x30c250]`, the same way
th06c reaches its own.

**The end item writes the name out**: `0x1400727d5` copies the eight bytes to `0x140c6db90` through
`0x14007d850` and puts the state at fifteen. **Which is th06c's shape with every offset moved**: that
build has the state at `+0x5194`, the name at `+0x5173` and the count at `+0x517c`.

**And there is a second entry, into `+0x9e88`**, typed in state thirteen at `0x140072fa3`, which no run
of a recipe's reaches. **It is the name the score file's own record is built from**: its end item at `0x14007301f` hands `+0x9e88` and the byte at `+0x9e9a` to
`0x14006db10` with a record beside them, which is the one write of that file in the whole exe — see *The
one write of this file, and the six calls that are not it*. So the two entries are not the same name:
`+0x9e70`'s end item copies its eight bytes into `0x140c6db90`, the object `th06.cfg` sits in, and
`+0x9e88`'s reach the file.

**And the one a run arrives at is `+0x9e70`'s, which the copy beside the exe says**:
`Th06nc::types_its_name` writes `+0x9e70` and nothing else, the launches that used it typed `ORB`, and
`th06.cfg`'s own name-last-taken at `+0x454` reads `ORB` and five spaces — which only the end item of
*that* entry writes, through `0x14007d850`. So the state a played run opens the screen in is the entry
whose name reaches the configuration, and what the score file's own record is named from is the entry
**no** run of a recipe's has reached.

## The window, which `th06.env` says the size of

**The game's two files are two stretches of its own object at `0x140c6db90`**: `th06.cfg`'s 44 bytes at
`0x140c6dfc4` (`+0x434`) and `th06.env`'s twelve at `0x140c6e01c` (`+0x48c`). `0x14007d240` reads both
at startup, called from `main` at `0x1400470d2`, with defaults built at `0x14007d33c` where a file is
missing, and `main` writes both back at exit, `0x140047785` and `0x1400477a6`. **No byte of the 44 is the
window.** What they are:

| | |
| --- | --- |
| `+0x434`..`+0x43d` | **the keys**, five words the key config writes together at `0x14004c0b3`. A copy beside the exe held `12, 13, 9, 14, 4` |
| `+0x444`, `+0x445` | **the lives and the bombs a run starts with**, which is where the run's own start takes them from: `0x14004f32b` and `0x14004f341` read them and `0x14004f33b` and `0x14004f363` write them into `0x140549d40` and `0x140549d41`. A copy beside the exe held `2` and `3`, which is what every launch's first `run:` line reads. The options screen shows them, `0x14004a89f` and `0x14004a979` among its reads, and steps neither |
| `+0x446`, `+0x447` | **two settings of nought to nine**, which the options screen steps: `0x14004abda` and `0x14004abbb` step them down while they are over nought and `0x14004ad3f` and `0x14004ad20` up while they are under nine, `0x14004ad28` and `0x14004ad05` being the compares. A copy held `7` and `7` |
| `+0x448` | **楽曲モード, `NewClassic` or `Classic`** — which music the game plays, named by a picture of the option screen's fifth row (`menu-item8-options.png` beside the copy). That row is what steps it: `0x14004acd3`'s `inc` with `0x14004ace6` wrapping it to nought at two, `0x14004ab7e` stepping it the other way, and each change followed by `0x14007d8d0(0x140c6db90, "data/bgm/th06_01.opus")` — the title's own track asked for again, which is what a change of soundtrack needs. A copy held `0` |
| `+0x449` | **the difficulty last chosen**, `0x140c6dfd9`, which `0x140049d0f` tests and `0x140049d1c` puts back to 1 where it is four or more — a copy left on `4` by an Extra run reads Normal on the way in. **Both of those are in the menu's item 2 case**, and `0x140049cfc` and `0x140049d05` do the same to the run's own `[0x14053d410]` two instructions earlier: a practice run is not one of the Extra, so taking that item puts both difficulties back |
| `+0x44a` | **the decide and the cancel swapped**: `0x140013094` reads it and builds the indices `12 + it` and `13 - it` into the button states at `[rsp+0x20]`, the first of the two contributing `0x200` to the word and the second `0x100` — so nought makes 12 the cancel and 13 the decide and anything else swaps them. The key config screen toggles it with `0x14004bf63`'s `sete`, draws the pair in that order at `0x14004ba41`, and `0x14004c1d5` puts it to nought; `0x14007d7bb` writes it back after the defaults are laid down, so a reset keeps it. A copy held `0` |
| `+0x44b` | **the mode last chosen**, Standard or Challenge, which screen 14 writes on either of its edges and the run's own start turns into `0x14053d414` — see *`0x14053d414`, the byte that changes what a run starts with*. A copy held `0` |
| `+0x450` | **a dword whose bit `0x10` puts a column of raw numbers on the screen**, and that bit is the only one of it any instruction reads — seven accesses in all, five `test … ,0x10` and two dword loads whose one use is a `test al,0x10`. Under it `0x140042d9d` draws `[0x140549d1c]`, the graze, with `"%d"` (`0x14034f03c`) at `1228, 207` — out past the play field — and `0x140042ddf` the word at `0x14053d40a` beside it; `0x140042431`, `0x1400426f0`, `0x140042862`, `0x140042955` and `0x140042eff` are the other five, each gating a draw of its own. A copy held nought |
| `+0x454`..`+0x45b` | **the name the entry last took**, eight bytes, which `0x14007d850` writes. A copy held `ORB` and five spaces |

**And the offsets above are the object's, not the file's**: the 44 bytes begin at `+0x434`, so `+0x444`
is byte `0x10` of `th06.cfg`, `+0x44b` byte `0x17` and `+0x454` byte `0x20`. `0x140047751` onwards is
where the correspondence is plain — two `movups`, a `movsd` and a `mov` copying `0x140c6dfc4`'s 44 bytes
into the buffer the write is handed.

**The window is `th06.env`'s.** `[0x140c6e020]` is the screen mode: `main` reads it at `0x1400472e5`
after the file and calls one of three functions on the game's object, each of which logs
`ChangeWindowMode実行` through `0x1400b93b0` and hands `0x140098a40` the size the picture is made at:

| mode | function | what it makes | the size |
| --- | --- | --- | --- |
| 0 | `0x14007e1b0` | a window with a frame — the flag `0x1400b93b0(1)` raises logs `ウインドウモードフラグが立てられました`. The game's own name for it is **`Window`** | the item `[0x140c6e021]` names, `0x14007e36c` |
| 1 | `0x14007df70` | a window the size of the monitor it is on, from `0x140085690`'s display mode or `GetSystemMetrics` where there is no window yet. The game's own name for it is **`Borderless`** | the monitor's, `0x14007e0e7` |
| 2 | `0x14007dcb0` | the display taken, the flag dropped at `0x14007dde1` and `0x1400988f0(2)`. The game's own name for it is **`Full Screen`** | the monitor's mode, `0x14007ddda` |

**The names are the 画面設定 screen's**, which is where the three are chosen — `screen7-the-screen-settings.png`
beside the copy. The option screen's 画面設定 row opens **front end screen 7**, `0x14004c1f2`, whose own
rows are 画面モード with `Borderless`, `Window` and `Full Screen`, 解像度 stepped between angle brackets
(`‹ 1280×720 ›` under a launch orb had asked for a window of that size), `V-Sync` with `ON` and `OFF`,
設定をリセット and 設定を適用して戻る — so that last row is the way back and the bomb is not.

**So the borderless window is mode one and neither nought nor two**, which is what `env::THE_MONITOR`
already answers for `Screen::Fullscreen`: a window the size of the monitor *is* the borderless one, mode
nought is the plain window with a frame, and mode two is the display taken. Nothing in the exe says so in
words; the screen does.

**And the mode is the file's byte 4 and not its byte nought**: the twelve bytes begin at the object's
`+0x48c`, `0x140c6e01c`, so `[0x140c6e020]` is four in and `[0x140c6e021]` five. A copy read
`02 01 00 00 00 04 00 00 01 00 00 00` after a launch orb had put into a window of 1280x720 — byte 4
nought, the window mode, and byte 5 item 4, that size.

**`[0x140c6e024]`, byte 8, is the `V-Sync` of that screen.** It is the screen's own second two-state row:
`0x14004fa5f` and `0x14004fa7d` flip it with a `sete` off its being nought, `0x14004ffb3` writes it as
設定を適用して戻る is taken, and `0x14004fdfb` holds it against the screen's own copy at `[+0x1a720]` to
say whether anything changed. **And what it reaches is the renderer**: `0x140099ff0(it != 0)` writes the
*negation* into `[0x1405551a4]` and calls `0x14002e5d0`, and `[0x1405551a4]` is what `0x14023f381` tests
before `0x140276700` — the wait — deep in the engine. `main` makes that call at `0x1400472e0` on the way
in and at `0x140046bde` and `0x140046c14` on the key that flips the mode. A copy read `01`, which is
V-Sync on, and orb's own pacing is measured against a game presenting that way.

`0x140046bc0` flips the mode between nought and one on a key. The window is made after the choice, at
`0x1400473d0` through `0x1400c3370`, so the size is chosen before there is a window and nothing orb
answers runs between the read and the choice.

**Each mode has a byte of its own for which of four sizes**, `[0x140c6e021]`, `[0x140c6e022]` and
`[0x140c6e023]` for modes nought, one and two — read at `0x14007e27b`, `0x14007e03e` and `0x14007dd9f`
as `[game + 0x491]`, `+0x492` and `+0x493` — and the option screen steps each at `0x14004fada`,
`0x14004f9f8` and `0x14004f9ef`. In mode nought the item is the window; in the other two it only
caps the picture, the window being the monitor's either way.

| item | size | where, in mode nought |
| --- | --- | --- |
| 0 | the largest of the four that fits the monitor, tried largest first at `0x14007e2e4` | |
| 1 | 3840x2160 | `0x14007e2d0` |
| 2 | 2560x1440 | `0x14007e2c4` |
| 3 | 1920x1080 | `0x14007e2dc` |
| 4 | 1280x720 | `0x14007e2ba` |

`0x14007e2fa` writes the item settled on back, and the option screen calls the mode's function on the
running game at `0x14004feaa`, `0x14004fe9c` and `0x14004fe8e` — which is how `Th06nc::prepare_frame`
sizes the window to `Config::screen` on the first frame: the mode and the item are written, the mode's
function called, and one frame of the env's own window goes by first. The launches that read it: the copy's
env at mode one came up the whole of a 4K monitor, `3862x2216` with its frame; mode one entered on the
running game made the window its monitor's `2160x3840`; mode nought entered with item 4 made it
`1302x776` with its frame, 1280x720 inside, the game drawn whole in it.

**And the picture's size is a function of its own**, `0x140098a40(width, height, bits, hertz)`, which
every mode function ends in: it keeps the four at `0x140555138` onwards, refuses bits other than 8, 16 and
32 at `0x140098aa7`, takes `640x480` for a size of nought, and remakes the renderer at the size through
`0x1400f2a00` — or `0x140093d30` where one is up. Where in the window the picture then goes is four
dwords at `0x140c6e05c` — left, top, width, height, letterboxed to 16:9 — which the mode functions write
at `0x14007e168`, `0x14007e3ee` and `0x14007df22` and the draw reads at `0x14003db94` and forty more
places, with a byte at `0x140c6df34` beside them saying the picture is taller than 1080. A launch that
plays itself is what these are called with a size of orb's own for: the window mode entered with item 4
and then `0x140098a40(width, height, 32, 60)`, the four dwords written `0, 0, width, height` — measured
at 640x360 as a window of `662x416` with its frame, and at 320x180 as `342x236`, the game drawn whole in
either.

`[0x140c6e024]` is a byte `main` hands `0x140099ff0` as a flag at `0x1400472e0`.

## The replay

**`[front end + 0x168b8]` is the replay it loaded**, and `0x14004ede0(front end, stage)` is what starts
one — the function `0x14053d3dc` is set by at `0x14004eeb3`.

**And the second argument is that stage, counted from nought, which both callers say**: `0x14004a419`
passes the count its own loop stops on — `0x14004a400` walks the header's `+0x40` upwards eight bytes at a
time while the offset is under `0x78`, stepping `edx` for every null it passes, and `0x14004a411` refuses
the call where that count has reached seven, so what it hands over is the first stage the replay has a
record for. `0x14004a725` passes `[front end + 0x28]`, the cursor's own index, having first read
`[[front end + 0x168b8] + 0x40 + 8*that]` and refused a null. So the pair is the replay's own object and
an index into the seven pointers below.

Of the header:

| | |
| --- | --- |
| `+0x6` | **what mode the run was recorded in**: `3` is a scene out of the table at `0x140c6e3e0`, whose count is `[0x140c6e2ac]` and whose entries are `0x28` apart — `0x14004ee53` sets `0x14053d405` for one and reads its stage and difficulty out of the entry — and `1` is what `0x14004ef3a` sets `0x14053d414` from. **And the two playback paths read it twice each**: `0x14006cfb4` and `0x14006d967` copy `[header + 0x8]` into the difficulty only where it is *not* 3, a scene's difficulty being the table entry's, and `0x14006cfcc` and `0x14006d97f` then set the mode byte from its being 1 |
| `+0x7` | **the character in its high bits and the shot in `0x1`**, split at `0x14004ef15` and `0x14004ef29` |
| `+0x8` | **the difficulty**, or the scene's own id where `+0x6` is `3` |
| `+0x40` onwards | **one pointer per stage**: the first that is not nought is the one played, and `0x14004a419`'s walk stops at seven |

**And a stage's own record is read field for field**, `0x14006cfd3` onwards being where the playback puts
each back — the seven numbers, and then the buttons:

| | |
| --- | --- |
| `+0x8` | **the seed**, a word, into `0x140abae64` |
| `+0xa` | **the point items collected**, a word, into `0x14053d40c` |
| `+0xc` | **the power**, a byte widened into the word at `0x14053cad8` |
| `+0xd`, `+0xe` | **the lives and the bombs**, into `0x140549d40` and `0x140549d41` — the two `0x14004ef72` and `0x14004ef87` read at a playback's start as well |
| `+0x10` | **how far up the point item's own value table the run has got**, into `0x140549d2d`: the item walk's point case steps that byte at `0x14004462a` while the power is full, holds it at 30 (`0x140044636`, the compare being against `0x1f`), and `0x14004464f` indexes the dwords at `stage + 0x308580` by it to add to the score |
| `+0x12` | **the manager's own first dword**, a signed word widened into `[0x14053cab0]` — see *The game manager*, where what that dword is is not established, and note that this is a field a replay carries per stage |
| `+0x14` onwards | **the buttons**, `0x14006d00e` writing `record + 0x14` into the playback's own cursor |

**And the buttons are entries of twelve bytes, run-length by frame.** `0x14006d140` is the feed:
`[state + 0x30]` counts the frames of the stage, `0x14006d160` walks the cursor on while that count has
reached the *next* entry's `+0xc` — the following entry's own frame — `0x14006d173` replaces the low ten
bits of `[0x140abae80]` out of `[cursor + 4]`, and **`0x14006d185` is the edge's own half**: on the frame
that equals `[cursor]` exactly, the frame before's word at `0x140abae84` is written from `[cursor + 8]`
instead of from the buttons, so a press recorded at an entry's boundary reads as an edge when it is
played back. `0x14006d197` and `0x14006d19d` clear the repeat's flag and its count every frame of a
playback. The recording is the same shape from the other side: `0x14006d08d` holds
`[0x140abae80] & 0x3ff` against `[cursor + 4]` and begins a new entry where they differ, `0x14006d0a2`
putting the cursor on `entry + 0xc` and `0x14006d0b0` the stage's end pointer `0x18` past it.

`[front end + 0x963c]` is which replay slot, `0x140549db4` where its path is copied, and the front end is
left in state `0x1a` at `0x14004f015`.

**And the header is freed as the playback starts, which is why no jump can index it later**: the replay
list's own decide reads `[front end + 0x168b8]` at `0x14004a422`, frees it through `0x1402c6540` and writes
nought over the pointer, so what the playback runs on is its own state and not that header. The front
end's *taken out* callback frees the same field again at `0x140057ad5` — a null by then — and the front
end leaves the chain for the whole of a run. See `Th06nc::jump_to_stage`, which declines for exactly that
reason.

## What the update changed

The build of ver 1.03 and this one are the same source built again, and **what moved is where things
are, not what they are**: 4,927 of the old exe's 5,055 functions out of `.pdata` and the code between them
have a match here whose bytes agree once the rip-relative displacements and call targets are masked, 3,802
of them exactly, and of the 371 small numbers th06nc's sources name as a field or an offset none is read
at another one but the renderer's size. **There is no one slide**: `.text` moves by a different small
amount almost function by function, `.rdata` by `+0x34e60` to `+0x45d20`, and `.data` in runs from
`+0x4a000` to `+0x4c228` — so each address here was moved by the function or the reference it is in, never
by a formula.

What did change, and what orb had to change with it:

- **The frame is a function.** In ver 1.03 it was a stretch of `main`'s loop and orb stood in fourteen of
  its bytes with a stub of its own; here the whole of it is `0x14003d8b0`, which the loop calls once
  round, so orb stands in its prologue the way it stands in th06c's — *The frame, which `main`'s loop
  calls*.
- **The draw is inline in the frame**, where it was a function of its own orb called whole, and **every
  draw queues its sprites into the renderer's batch**, which the draw walk flushes after each job — *The
  draw*. The renderer is `0x37070` bytes where it was `0x27108`.
- **The CRT is linked into the exe.** `malloc` is `0x1402cdb80`, `free` `0x1402c6540`, `realloc`
  `0x1402d0f10`, `fopen` `0x1402cdfe0` and `_beginthreadex` `0x1402cde18`, called directly; the heap they
  use is the one `GetProcessHeap` answers once at `0x1402e72d4`, kept at `0x140abac18`; threads come from
  the one `CreateThread` call at `0x1402cde88`, and files open through `CreateFileW` at `0x1402ecc5e`,
  `0x1402eccbc` and `0x1402ecedc`. Which is what `Memory::ProcessHeapImport`, `Threads::CreateThread`
  and `FileOpen::CreateFileW` are.
- **And the CRT's own state is in `.data`**, which a chapter is a copy of: its fifteen locks at
  `0x140aba1a0`, one more at `0x140aba108` and the three standard streams at `0x140426390` are held back
  from a restore by `Th06nc::live_handles`.
- **`data/th06CM.dat`, `th06ED.dat` and `th06TL.dat` changed with it** and `th06ST.dat` did not. orb
  reads none of them; the stage scripts and conversations are `th06ST.dat`'s, so the midstage table and
  the readings of a stage stand, where a measurement made against the ending's or the front end's own
  assets — the ending's 33,204 updates — is the old files'.

## What is not established

- **Whether the CRT's objects off the heap read the same after a restore.** With the CRT in the exe, every
  allocation it makes for itself — an open `FILE` and the lock inside it, the table of low-level handles
  `0x140aba5e0` points into — comes off the heap `GetProcessHeap` answered, which is the game's own heap
  and put back with a chapter. Its statics in `.data` are held back; these are not, and a file opened or
  closed between a chapter being taken and being put back is the case that would find its handle table
  as it was.
- **Whether a launch can walk the supervisor's state 12** — **the path is read and the one press it
  wants is not orb's to press.** The two writes of `0xc` into `[0x140c6dfbc]` are each a machine's own
  way out and neither is a run's end: `0x14000cfb9` is **state 5** of `0x14000c960`, which state 1's
  **cancel** goes to, and `0x14000b4b5` is **state 7** of `0x14000a850`, gated on that machine's own
  `[+0xb4c]` reaching `0x14`. Three launches walked the first of those to its door: `27, 28, 29, 30, 31`
  into a card of the Spell Practice, the card beaten under `--clear`, the machine up inside the
  supervisor's `2/2` — and then nothing. **A card is not ended by dying in it** (`deaths=11 lives=0`,
  the card restarting each time) and **not by being given up** either (`wanting 8` and the front end
  after it, the same 8 an ordinary run given up asks for); beaten, it puts the machine up and leaves the
  supervisor where it was. **And three `bomb` presses at that menu moved nothing**: `ranking:` counted
  from 1 to 3,111 with no reset, where state 5 writes `[+0xb4c]` back to nought at `0x14000cba3`.
  Which the exe explains: the bit state 1 reads as its cancel is `0x4000`, and the word's one source of
  that bit is `0x140013499`'s binding byte `[0x140c6e093]` indexed into the keyboard state — a block
  filled at `0x140013375` whenever `GetKeyboardLayout` differs from `[0x140c6e180]`, which is past
  `.data`'s raw bytes and so zero in the file. **So what is left is which key that byte holds**, read out
  of the running game rather than off the exe, and `the-supervisors-twelve.yaml` beside the copy is the
  recipe waiting for it. The other
  eleven are read off the exe *and* watched on a launch — **10 by a clear**, which read `2/10` and then
  `10/10` and then `10/7`, and **6 by the menu's item 5**, `1/6` and then `6/6` — and **state 4 is dead**
  as far as four launches and every write of the field carrying a literal can say. Every other is
  read: 1 the front end, 2 a run, 3 the next stage of one, 5 the game going, 6 the screen a run ends at
  asked of the front end, 7 the screen a run that *finished* ends at, 8 a run given up, **9 the music
  room** — item 6's state 21 asks for it and its job reaches `data/bgm/th06_%d` — **10 the ending** and
  **11 the credits**, which item 7's state 23 asks for and a picture of that screen reads `CREDIT` over
  a page counter. **And 6 is the screen a run ends at either way**: item 5's own screen 12 asks for it
  too, and what that screen then shows is 夢の記録 — a row per difficulty over `SCORE & SPELLCARD` — so
  the ranking browser and the screen a finished run lands on are one object, which is what
  `Th06nc::show_ranking` and `result_screen::A_RUNS_OWN` are about. And **nothing asks for 4**: among the writes of
  `[0x140c6dfbc]` — the state wanted, the supervisor's `+0x42c` — the literals are 1, 2, 3, 7, 8, `0xa`,
  `0xc` and `-1`, and no site writes four — thirty-one writes of it in all, of which the rest carry a
  register; its own case is `0x14007be15` from every table it appears in,
  which is the nothing-to-do branch. **And two launches watching the field never saw one**, which is what
  `say_the_supervisors_state_when_it_changes` was added for — a line of orb's saying the pair
  `[+0x428]` and `[+0x42c]` the frame either changes. Between them they walked the front end, a run, a
  stage transition, a run given up, the screen a run that *finished* ends at, the music room and the
  credits, and what those two fields ever held is **acted on 0, 1, 2, 7, 9 and 11** and **wanted -1, 0,
  1, 2, 3, 7, 8, 9 and 11**. So the reading is that the state is dead; what no launch has walked yet is
  6 asked of the menu, 10 the ending and 12.

  **And the pair says which of the twelve the job ever acts on.** 3 and 8 are wanted and never acted on —
  `2/3` goes back to `2/2` and `2/8` to `1/1` — because the case of the state they are asked *from*
  handles them and `0x14007baf1` writes the next state over `[+0x428]`. So a run's next stage and a run
  given up are asked for inside the run's own case, and the screen a run ends at instead **holds the
  supervisor at 7 for its whole life**: the second launch's field stayed `7/7` through the name entry and
  four decides, the front end coming back only when that screen's own way out writes 1.
  - **State 10 is the ending, off the one site that asks for it.** `0x14004148d` writes `0xa` where a run
    has finished, the difficulty is not 4 and `[0x14053d3dc]` says no replay is playing — and the two
    branches beside it are what make it the ending: **the Extra asks for 7 instead** (`0x14004147e`, with
    `[0x14053d403]` set and the score written into `[0x14053d3e0]`), which is the screen a run that
    finished ends at, and **a replay asks for 8**, a run given up. The ending's own job is registered in
    the leave handler those states share, `0x14007bb91` calling `0x14002c1d0` where `[r8+0x34]` is 2.
  - **What each state *shows* cannot be read off the exe**: this game draws that text out of its own
    glyph sheets, and a search of the exe finds none of `リプレイ`, `保存`, `はい`, `いいえ` or
    `コンティニュ` — its one Japanese string is `東方動作記録`, a replay file's own header. A picture
    per state is the reading, taken from outside with the launch holding each one, and
    `the-state-12-held.yaml` beside the copy is the shape that works: five seconds on the states before
    the one wanted and sixty on it, since two watches polling every thirty seconds went past a
    thirty-second hold without a shot.
  - **And a run can be lost at the keyboard after all**, which an earlier reading of this had backwards:
    `orb_bot`'s hit draw does dodge everything where the chapter is nought, but the rules steer only
    inside a `play_until` step — a `press` step's word is the step's own and a `wait_until`'s is no word
    at all — so a recipe that stands still in the normal mode loses the run. Two launches did:
    `deaths=3 lives=0` with the stage's clock stopped at `frames=2193`. **What stops there is this
    game's continue prompt**, which waits rather than counting itself out — one sat on it for four
    minutes — and which `Th06nc::read_state` reads as `paused`. A picture of it shows `コンティニュ〜？`
    over two items with the first lit, and **one tap of down and a decide at it took the continue**, the
    run coming back with two lives and going on to `deaths=4 lives=1`. So a lost run is not the cheap
    way onto that screen; a run **given up** is, and that is what the recipe
    `the-states-the-screen-a-run-ends-at-walks.yaml` beside the copy drives.
  - The entry a *clear* arrives at is read and stays read: the launch of `orb_real`'s
    `a_run_played_to_its_clear_without_a_bomb` that came back `ok over 100818 frames` walked
    `ending skipped, 31587 frames run` straight into the state at `result_screen::TYPING_A_NAME`, and
    fourteen was never entered.
- **Which of the front end's screens hand on with no press, and why 11 did not.** `0x1400491af` writes
  the decide into the word itself where `[front end + 0x9638]` is exactly `0xf`, which is what
  `fake::th06nc`'s `A_SELECT_GOES_BY_ON_ITS_OWN_AFTER` was written from — and it hands the difficulty, the
  character and the shot on by themselves after fifteen frames. **A launch says the real game does not**:
  one that decided at 9 and then only waited stood on **screen 11 for 1,200 frames**, where the launch
  beside it that pressed at 11, 13 and 14 walked `9, 11, 13, 14, 24`. So either `+0x9638` is not the count
  that comment takes it for or the write is gated on something else, and which screens self-advance is
  unread. **The fake is over-generous until it is**, which `the_run_of_th06nc_left_behind` says of its own
  walk, and `orb-real`'s recipes press at each select and so do not lean on it.
- **What the other writer of `[obj + 0xd98]` puts there**, `obj` being
  `what_a_job_runs_on(0x140009800) + 0x6e10` — **and what the two fields are is read**: `+0xb4c` is the
  frames its own state has been up, `0x14000b5c0`, `0x14000c3e9` and `0x14000d046` stepping it,
  `0x14000ca6e` putting it back to nought as a state is asked for, and its compares being the four, the
  seven and the `0x14` that gate the machine's own moves; and `+0xd98` is the byte that picks between
  them, `0x14000ca51`'s `cmovne` taking **9, the name entry**, where it is not nought and **12, the
  ranking**, where it is — and the same byte picks the pictures `0x776` and `0x775` at `0x14000c9c9` and a
  2 against a nought at `0x14000c9ae`. **And what looked like a byte no instruction writes** — all four
  accesses reads, `0x14000c9ae`, `0x14000c9c9`, `0x14000ca51` and `0x14000d199`, each a byte — **and not from
  the job's own base either**: the sub-object sits at `+0x6e10`, so the same byte is `+0x7ba8` of the job
  and no instruction addresses **that displacement** — and that is because the machine's object is a
  **static**: `0x140009b30` builds the chain element at `0x140436e30` with `0x140009800` as its callback
  and **`0x140429270`** as the object it runs on, so the field sits at a fixed address and the compiler
  reaches it rip-relative. `0x140429270 + 0x6e10 + 0xd98` is **`0x140430e18`**, and two instructions
  write it:
  - `0x14000bc6e` writes **nought**, inside `0x14000bc2b` — gated on `[0x14053d405]` being set, a scene,
    and `[0x14053d3dc]` clear, no replay playing. It copies the score `[0x14053d3e8]` into
    `[0x14053d3e0]`, sets `[0x14053d406]`, and clears `+0xd94` (`0x140430e14`) and `+0xb4c`
    (`0x140430bcc`) beside the byte. So a scene's own end puts it to nought, which is the **12** branch;
  - `0x14003be3a` writes **`al`** — nought where `ecx` is above nought and `[0x140aee0ad]` equals `dil`,
    **1** otherwise, which is the **9** branch. **And the machine it sits in is read**: `0x14003bd1c`
    dispatches on `[0x140c6e39c]`, its own state — nought, one, two and three, with three writing **8**
    into the supervisor's wanted at `0x14003bd5a` — and this is **case two**, which reads
    `[[0x140bfa198] + 0x234]` into `[0x140c6e2a8]` as that `ecx`, counts `[0x140c6e398]` down at
    `0x14003bd8f` and leaves while it is above nought. At nought it writes `[0x14053d406]`, the score
    into `[0x14053d3e0]`, `+0xd94`, the byte and `+0xb4c`, and `0x14003be46` moves the machine to its
    state three.

  **So both writers turn the machine on rather than ending a run**, each writing `[0x14053d406]` in the
  same breath, and which of them runs is which end the game came to: `0x14000bc6e` is a **scene's**, the
  object `0x14000995c` hands to `0x14000bc20` behind `[0x14053d401]`, and `0x14003be3a` is a **stage's**,
  beside the `0x14003be26` this file already reads as the stage's own build. Neither touches the
  supervisor at all.

  **And a scene's is measured.** A `--clear` launch walked into a card of the Spell Practice, played it
  out — `run: the boss is gone with 878 life left`, a card whose timer ran out with an invulnerable
  player, which is a card captured — and then read `ranking: 1 frame(s) up, picking on 0x0` a second and
  a half later, counting to 405 with the byte at nought the whole way and **the supervisor never leaving
  `2/2`**. So a scene's end is the nought, which is `0x14000ca51`'s **12**, the ranking; the machine
  comes up inside the supervisor's state 2; and what is left to read is only the other writer's `al`,
  which wants a run played to its own end rather than a card.

  **And a card does not end by being died in.** A launch started one and stood still through it:
  `deaths=11 lives=0` with the supervisor at `2/2` throughout, the card restarting each time — which is
  what the Spell Practice is for.

  **Nor by being given up**: a launch that started a card and then left it read
  `retry: the run is given up`, `supervisor: state 2, wanting 8` and then `1/1` — **the same 8 an
  ordinary run given up asks for**, and the front end after it. So what is left is a card **beaten**,
  which is the one way into that end nothing has driven yet.
