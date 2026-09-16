# th06nc, as read off its exe

What is written here was read off `th06nc.exe` of **md5 `b0cc28ec904e9efce2439ad857c938b5`**, 5,539,328
bytes, which is 「東方紅魔郷: New Classic」 ver 1.03 — Steam app id 4659620, the other 紅魔郷 sold beside
th06c (app id 4924840).

**It is built on th06c's engine and is another game.** The host layer is the same code: the keyboard read
is a function of the same `0x26f` bytes with its own statics, the Steam startup and the chain walks read
the same instruction for instruction, and the same names are asked of `GetProcAddress`. The game on top
is rewritten — its frame is inline in `main`, its game manager has thirty-two states and leaves the chain
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
| the exe | `PE32+ executable (GUI) x86-64`, six sections |
| `.text` | `0x140001000`, `0x2bea91` bytes |
| `.rdata` | `0x1402c0000`, `0x626ba` |
| `.data` | `0x140323000`, `0x1e7200` in the file; its statics run on to `.pdata` at `0x140c2a000` |
| `.pdata` | `0x140c2a000`, `0xccb4` — 4,367 `RUNTIME_FUNCTION`s, from file offset `0x508a00` |
| `.rsrc` | `0x140c37000`, `0x322b0` |
| `.reloc` | `0x140c6a000`, `0x9a8` |
| what it was built with | `MSVCP140.dll`, `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll` and ten `api-ms-win-crt-*.dll` |
| what it is sold through | `steam_api64.dll`, imported outright — the same nine names th06c imports |

**Its files are under `data/`**: `data/th06ST.dat`, `th06CM`, `th06ED`, `th06FN`, `th06IN`, `th06MD`,
`th06TL`, `data/bgm/th06_01.opus` and its neighbours where th06c has `bgm/th06_01.mid`, `data/msg1.dat`
to `msg7.dat`, `data/title/th06logo.png`; and beside the exe `th06.cfg` (44 bytes, where th06c's is 28),
`th06.env`, `score.dat` and `replay/th6_NN.rpy`.

## What it imports, and what it asks for by name instead

`USER32.dll` gives it eleven names, its IAT at `0x1402c0228`: `GetWindowRect`, `MonitorFromWindow`,
`SetWindowPos`, `AdjustWindowRectEx`, `EnumDisplayMonitors`, `GetAsyncKeyState`, `CallNextHookEx`,
`GetKeyboardLayout`, `MessageBoxA`, `MapVirtualKeyExW`, `GetSystemMetrics`.

**And no `DialogBoxIndirectParamW`**, nor `EndDialog`, `CheckRadioButton` or `IsDlgButtonChecked`, and no
string naming a dialog control: **this exe puts no startup dialog up**, where th06c puts one up every
launch. A launch with no `th06.cfg` beside the exe came up in a window of `3862x2216` — the whole of a
monitor — and wrote one.

Everything else is a string `GetProcAddress` is given — the import at `0x1402c0090`, called from 346
places, with `LoadLibraryW` at `0x1402c0080` and `LoadLibraryA` at `0x1402c00a8`. The names orb answers
are all there: `GetKeyboardState` (`0x1402c30f8`), `CreateWindowExW` (`0x1402c28e0`), `MoveWindow`
(`0x1402c29e0`), `XInputGetState` (`0x1402c1448`), `CreateDXGIFactory2` (`0x1402cd9a0`) with
`CreateDXGIFactory1` and `CreateDXGIFactory` beside it, and `D3D11CreateDevice` (`0x1402cdbd0`). The
renderer is D3D11 — `D3DCompiler_33.dll` to `_47.dll` are named — the effects are `DSound.DLL`'s and the
music `Mfplat.dll`'s. A launch said so: `the swapchain this game makes will be orb's to present through`.

## Steam

`0x1400678a0` is the startup that reaches it: `SteamAPI_RestartAppIfNecessary` at `0x1400678ae`,
`SteamInternal_SteamAPI_Init` at `0x1400678c1` with `test eax,eax; je` after it, and the interfaces
through `SteamInternal_ContextInit` at `0x14006799e` and `0x1400679b5` — each followed by `mov rcx,[rax];
mov rax,[rcx]; call` — with `[vtable+0x20]` at `0x1400679c1` being the language, compared against the
strings at `0x14030b6a0` onwards. The same shape th06c's has at `0x140048790`, so `orb::steam` answers
it unchanged: a launch with no Steam client running reached its title. The import slots are
`0x1402c05f0`–`0x1402c0630`, and `SteamAPI_RunCallbacks` is called once round the loop at `0x140045d27`.

## The keyboard: one 256-byte state, filled two ways

**`0x1400ae180` is the read**, `0x26f` bytes, the same function th06c has at `0x14008f930` with its own
statics:

| | |
| --- | --- |
| `0x14055cad0` | **the DirectInput keyboard**: its vtable's `+0x48` is called with `(0x100, 0x14054343c)` at `0x1400ae258` and again at `0x1400ae286`, `+0x38` between them at `0x1400ae269`. Nought here sends the read down the other path |
| `0x14054343c` | **the 256 bytes the game plays a run off** |
| `0x1400ae2bc`, `0x1400ae315` | `GetKeyboardState` through the slot at `0x14055d408`, into a buffer on the stack; the second path memsets the 256 bytes and fills them through the table below at `0x1400ae350` |
| `0x1402c1200` | **the table from a virtual key to the index it is kept at**, six bytes an entry, ending at `0xffff`: `0x41 -> 0x1e`, `0x42 -> 0x30`, `0x43 -> 0x2e` — `DIK_A`, `DIK_B`, `DIK_C`, so the buffer is indexed by DirectInput's scan codes |
| `0x140543438` | the timestamp the walk writes, held against `timeGetTime`'s answer through `0x14055ccc0` |
| `0x14055caa4`, `0x14055cae4` | the two flags that send the read down the `GetKeyboardState` path |
| `0x140545544`–`0x140545548` | the five mouse buttons, read out of the same stack buffer |

The read is called from `0x1400a9f73` and `0x1400aa7ee`.

**And the word it comes out as is `0x140a6ec60`**, with the frame before it at `0x140a6ec64`: the game
manager's state 0 clears both, `0x140a6ec48` and `0x140a6ec5c` at `0x140047ad4` as a run starts, the
player's update reads it at `0x1400689ed` and five more places, and the loop's fast-forward reads `0x8000`
of it at `0x140045db2`. Its bits are th06 1.02h's: the player's walk at `0x140069072` onwards tests `0x10`,
`0x20`, `0x40` and the sign bit for up, down, left and right and `0x4` for the focus at `0x1400690a5`, and
the game manager writes `0x100` into it at `0x140047a6c` where a screen is to be skipped.

## The frame, which is inline in `main`

**`0x1400454f0` is `main`**, `0xc5e` bytes, and the loop runs from `0x140045ce0` to `0x140045ea5` with
the whole frame in it — there is no function of the frame's own to stand in, where th06c has
`0x1400256f0`:

| | |
| --- | --- |
| `0x140045ce0` | the loop's head: `[0x140a6ebcc]` non-zero leaves it |
| `0x140045cf5`–`0x140045d22` | the window's size read (`0x1400b9ba0`) and handed to `0x14007ad30` on `0x140c21970`; `0x1400bcb90` off the byte at `0x140c21e00` |
| `0x140045d27` | `SteamAPI_RunCallbacks`, and the overlay asked through the interface at `0x140509db0` |
| `0x140045d78` | `0x14007b120` on `0x140c21970` — the renderer's own object — with two floats |
| `0x140045d7d` | **the calc chain walked**: `lea rcx,[rsp+0x70]; call 0x14003be80`, the answer a byte and what the frame leaves with written through the pointer |
| `0x140045d97`–`0x140045de2` | and walked **again up to seven more times** while `[0x1404f278c]` is set, `[0x1404f27b0]` is clear, `[0x140c21d9c]` is 2 and `[0x140a6ec60]` has `0x8000` — a replay being fast-forwarded — each extra walk counted into `0x140509690` |
| `0x140045dfa`–`0x140045e8c` | **the sound's catch-up**: where the count is above nought and a track is playing (`0x14050966c` not `-1`, `0x14050967c` set), `0x1400c4a10` answers the position, `0x320` samples a walk are added — 48,000 a second — held inside the loop at `0x140509680`..`0x140509688` where `0x14050967d` says the track loops, or under `0x1400c4f90`'s length where it does not, and `0x1400c9930(seek, track)` seeks |
| `0x140045e91` | **the draw frame**, `0x14003bf70` |
| `0x140045e96` | `0x14003c330`, which puts the frame on the screen and then waits for the deadline |
| `0x140045e9e` | `0x1400c16d0`, the message pump: nought carries the loop on, anything else leaves it at `0x140045eab` |

Where the calc walk answered non-zero, `0x140045de9` reads what it left at `[rsp+0x70]`: nought goes
to the pump at `0x140045e9e` with nothing drawn, anything else leaves the loop at `0x140045eab` — and
every non-zero answer leaves a non-zero, `0x14003bf21` and `0x14003bf0b` writing 1 and 2.

**The bytes at `0x140045d7d` are `48 8d 4c 24 70 e8 f9 60 ff ff 84 c0 75 5e`**, fourteen exactly — the
`lea`, the `call`, `test al,al` and `jne 0x140045de9` — which is what a 64-bit jump takes and is where
orb's own frame stands: `game::th06nc::frame_in_the_loop` is entered there by the jump, calls
`runtime::render`, and jumps to `0x140045e9e` or `0x140045eab` by what it answered. `main`'s `rsp` is a
multiple of sixteen there — three pushes and `0x120` off an entry that was eight past one, and
`movaps [rsp+0x100]` at `0x140045b90` says as much — so the stub needs only its shadow space. Measured:
`frame loop hook installed`, and a launch ran `600 frames, 16668us apart` with orb's own pacing lines.

### The chain walk

**`0x14003be80` is the calc chain's**, `0xe9` bytes: `lea rbp,[0x1404f1440]`, then each element's `+0x8`
called with its `+0x38` in `rcx` and `+0x28` followed. What the callback answers: 2 calls it again, 0
takes it out through `0x1400126e0` with the element in `rdx`, 1 starts over from the head, and 3, 4 and 5
end the walk — each writing `0x140c21970`'s first byte to nought and calling `0x140076a30`, which flushes
the frame's queued sound effects out of `0x140509670` — before it answers. The walk returns `al` non-zero
to leave the frame, and writes 1 or 2 through `rcx` for what it leaves with.

**And the draw chain's is inline in the draw frame `0x14003bf70`** at `0x14003c015`, `lea
rbx,[0x1404f1400]` — the same walk, 0 taking the element out at `0x14003c258`.

`0x1400126e0` takes an element out of either chain: it searches `0x1404f1440`'s links and then
`0x1404f1400`'s for `rdx`, unlinks it, calls its `+0x18` on its `+0x38` and frees the `0x40` bytes where
`+0x2`'s low bit says they were allocated.

`ChainElem` is the `0x40` bytes th06c's is — `+0x8` the callback, `+0x10` the one called as it is
registered, `+0x18` as it is taken out, `+0x20` and `+0x28` the links, `+0x30` the element's own
address, `+0x38` what the callbacks are called on — read off the player's registration at `0x140067e1e`
onwards and the game manager's at `0x140055c5d`.

### The draw frame

`0x14003bf70`, `0x2f9` bytes: `0x140001970`, then `0x14007b120` on `0x140c21970` with nought and a
float, `0x140083870` into four ints on the stack, `0x1400953f0` with the four bytes at `0x140509ba0`
(a colour), `0x14007f5b0(0)`, `0x1400953f0` with the four ints (the viewport), the draw chain walked,
and then a screenshot's own path behind the byte at `0x140a6ec6a` before `-1` is written into
`[0x140a6e9b0]+0x20f80`. Orb calls it whole as its draw.

### The timing

`0x14003c330`, `0x41b` bytes: the window's size again (`0x140083ec0`), the viewport's four ints at
`0x140c21e3c`–`0x140c21e48`, and where the viewport does not cover the window the bars around it drawn
through `0x140095f20`, `0x140095d70`, `0x1400a1b10`, `0x140080cb0` and `0x140095820`; then
`0x1400953e0`, which is `jmp 0x1400928b0` — the present, through the swapchain orb took; then the clock
through `0x140c220e0`, the deadline out of the period at `0x140c22100`, the base at `0x140c22128` and the
count at `0x140c22130` — `0x14003c4ad` starts base and count over where the clock is more than a period
past `base + period × (count + 1)` — a sleep through `0x140c220f0` while more than `0x5f`-averaged slack
is left, and a `pause` spin at `0x14003c594` until the deadline. After it the fps into `0x140c221d0`
through `0x140038740` and `%.02lffps` at `0x14030b9e8`.

**Orb calls it whole with the base and the count written to nought first**, which puts every clock a
period past the deadline and starts the limiter over without a wait — see `Th06nc::end_drawing`, which
has why that and not the pieces.

## The managers, and what registers them

| registered at | the object | the frame's callback |
| --- | --- | --- |
| `0x140067dc0` onwards | `0x1404ff3a0`, `0x7854` bytes zeroed, and `0x140506bfc`, `0x2a5c` — **the player**, its element kept at `0x140506c48` | `0x140068820`, and `0x14006a130` for the draw chain |
| `0x14003b1c3` onwards | `0x1403ec2a0`, `0xf5010` bytes zeroed — **the bullets and the lasers**, with `0x1404e12b8` (`0x10108`) beside it; the element is the static at `0x1403ec260`, priority `0xb` | `0x140010870` |
| `0x1400371c9` onwards | `0x140aa1e90` — **the enemies**, its element the static at `0x1404f1540`, priority 9 | `0x1400373b0` |
| | `0x140baf0d0` — **the items**, walked by `0x140042980` from inside the bullets' update at `0x1400108e7` | |
| `0x140055c5d` onwards | **the game manager**, allocated, its element kept at `0x140c07248` and the manager at that element's `+0x38` | `0x140047940`, a wrapper that calls `0x1400479e0` once — or up to `0x258` times while `[manager+0x168d0]` is set |

**The game manager leaves the chain for the whole of a run.** A launch's `chain:` lines read four jobs
with `0x140047940` among them through the front end, three without it on the frame the run's lives and
bombs were written, and then eleven — `0x140079c70 0x1400090d0 0x140075af0 0x14003a210 0x140077bb0
0x140068820 0x1400373b0 0x14002b280 0x140010870 0x14003cc50 0x14006b970` — for the stage. So the
element at `0x140c07248` points at freed memory during a run — read through it, the state came back as
the float `1.0` — and the manager is found by walking the chain for its callback, and a stage is the
player's job being in the chain and the stage being one to seven.

**And the ending is a job too.** The frame the sixth stage's jobs left, a cleared run's `chain:` line
read five: the three that are always there, `0x14006b970` of the stage's, and **`0x14002bac0`**, which is
registered at `0x14002b936` by `0x14002b840` — called from `0x14007a011`, inside the job at
`0x140079c70` that a stage ends into — and reads the input word at `0x14002bb11` the way a screen that
is pressed through does. The game manager is not back while it runs, so `State::in_ending` is the chain
holding this callback, and `run: the ending is running` is the line a clear is read by.

**And the screen a run ends at is a job as well**: `0x14006e3a0` arrived on the frame the ending's job
left, stood with `0x14006b970` and then alone, and the game manager came back after it — at state 2, the
title — with `score.dat` and `replay/th6_01.rpy` written beside the exe. A run pressed through with
`decide` reaches the title that way, and what the screen holds is *The entry a name is typed into* below.

**The Extra has no ending.** Its stage's jobs left straight into `0x14006e3a0` on both of two launches —
the one that beat Flandre and the one that died under her — so what tells them apart is her life as
she left: `5` where her last attack was emptied, the field then standing empty for a thousand frames
before the jobs went; `1000` where the run died inside her survival card, which holds her life at that
for its whole length. Her attacks read off the `run: the boss has` lines: the midboss at `6000` and
`1800`, then `15000`, `1800`, `8000`, `1200`, `7182`, `1900`, `7984`, `2400`, `9689`, `1400`, `9359`,
`2200`, `10000`, `1520`, `9382`, `1800`, `1000` held for `5,000` frames, `6000`, `15000`, `3900`,
`4000`, and `6000` last.

**Everything is held** is `0x1404f27b9`: read by the bullets (`0x14001087f`), the enemies
(`0x14003815c`), the player (`0x14006882d`) and four others, written at `0x14003341d`.

## The game manager's states, which are this game's screens

**The state is `[manager + 0x168b0]`**, and its cases:

```sh
python3 -c '
import struct; d=open("th06nc.exe","rb").read()
def at(va): return va-0x140001000+0x400
for i in range(32): print(f"state {i:2d} 0x{0x140000000+struct.unpack_from("<I", d, at(0x14004d0e8)+4*i)[0]:x}")'
```

— which says states 15 to 20 and 26 share `0x1400487d8`, and 28 to 31 `0x14004cbcf`. **Which state is
which screen was read off launches' `run:` lines**, pressing at each:

| | |
| --- | --- |
| 1 | the logo, which goes to 2 on its own |
| 2 | the title, and a decide opens the menu |
| 3 | **the main menu**, ten items: 0 starts a run, 1 the Extra — skipped while it is locked, a fresh launch's tap of down going 0, 2, 3 … 9, and offered once a run has been cleared — and 4 opens screens 15 to 18. The Extra's walk is the same screens as a run's with the difficulty already 4 at screen 9 |
| 8 | goes by on its own after item 0 |
| 9 | **the difficulty**, four items — `0x14004b112` onwards compares the cursor with 0 to 3 — and a launch that took item 1 played on difficulty 1 |
| 11 | the character |
| 13 | the shot |
| 14, 24, 25 | **each takes a press**: a launch that pressed nothing at 14 sat there for 1,800 frames, and one that pressed nothing at 24 the same |
| — | and then the manager leaves the chain and the stage's jobs arrive with `stage 1` |

**Its cursor is `[manager + 0x28]`**, the dword the difficulty state compares with 0 to 3. `+0x30` is a
counter of the state's own frames — every state writes nought into it as it comes up and a launch read it
climbing once a frame — and `+0x9638` is another.

**And its menus repeat a held key**, the way th06c's do: six frames a press is one item.

## The run's own numbers

| | |
| --- | --- |
| `0x1404ff0f0` | **lives**, one byte: the item walk's 1up case stops it at 8 and steps it up at `0x140043407`; `0x14007a126` writes `0x302` over it and the byte after as an Extra run starts, and `0x14007a160` writes `[0x140c21db4]` — a chosen starting stock — for the rest. A launch's Normal run started with `2 live and 3 bomb(s)` |
| `0x1404ff0f1` | **bombs**, one byte, the same shape at `0x1400433c7`, and `0x14007a16d` from `[0x140c21db5]` |
| `0x1404f1e88` | **the power**, a word the item walk's two power cases write back at `0x140042df1` and `0x1400431db`, and `0x80` being where the full-power item writes it at `0x1400435a1` |
| `0x1404f1e84` | **the stage, counted from one**: `0x14003cdcc` multiplies it by a thousand into `Stage * 1000 =  %5d`, `0x14003cd89` compares it with 6, and a launch's first stage read `stage 1`. The Extra read `stage 7` — `0x14007a11c` writes 6 as it starts and the stage's own start counts it on |
| `0x1404f27c0` | **the difficulty**, compared with 4 for the Extra at `0x14007a117`; Normal is 1 and the Extra 4 on the `run:` lines |
| `0x1404f278c` | **a replay is being played back**: set at `0x14004d753`, cleared at `0x14004dc35` and `0x140052437`; the frame's fast-forward at `0x140045d97` runs only where it is set |
| `0x1404f27b0` | **and the playback is the attract demo**, which that fast-forward refuses at `0x140045da0`; cleared at `0x140055b9d` |
| `0x1404f27c4` | a byte the run start reads for which starting stock applies, and the item walk reads for whether a 1up counts down `0x1404f1e60` instead — a mode of this build, unread |
| `0x1404ff0e0` | **the rectangle the player is held to**, left, top, width and height — `0x1400693c0`–`0x14006942b` |

**The field is 384 by 448** and this exe holds the pair as literals rather than in `.data`: `0x14030d008`
and `0x14030d014`, loaded by the bullets' update at `0x1400113e3` and the enemies' at `0x14003741f`.
`.rdata` from `0x14030cd00` on is one sorted pool of every float literal the build has — `0.03125`,
`0.5`, `0.6667`, `1.0`, `2.0`, `2.5`, `3.0`, … `384`, `448`, `480`, `640` — the same shape th06c's is.

## The player

**A static object at `0x1404ff3a0`**, `0x7854` bytes, with a second piece of `0x2a5c` at `0x140506bfc`;
`0x14006a980` — the hit test — is handed it as `rcx` by the bullets at `0x140011411` and the enemies at
`0x140037a31`.

| | |
| --- | --- |
| `+0x7730`, `+0x7734` | **where the player is**, which the move at `0x1400693b0` writes and the laser test reads as the statics `0x140506ad0` and `0x140506ad4` |
| `+0x7710`, `+0x7714` | what a frame's movement is scaled by on each axis — `mulss` at `0x140069388` and `0x140069398` |
| `+0x7750` | which way it is being moved, 1 to 8, written by the walk at `0x140069066` onwards |
| `+0x785c` | whether the focus is held, off the word's `0x4` |
| `+0x7860`, `+0x7864`, `+0x7868`, `+0x786c` | **the four speeds**: straight, straight focused, diagonal, diagonal focused — the walk at `0x140069136` onwards picks one by the direction and the focus |
| `+0x774c` | **the radius a hit is measured against** — see below |
| `+0x7754` onwards | **sixteen boxes of `0x10` bytes**, which `0x14006a8c0` walks before the player's own: a hazard inside one is answered 1 rather than 2 |
| `+0x7898` | **which state the player is in**, one byte: nought is the only one a hit lands in (`0x14006aaa5`), 2 is what the hit writes (`0x14006aaf1`), 3 is what `0x140068f95` writes as the invulnerability begins |
| `+0x773c` | **the frames a bomb still takes the hit back in**: the hit writes 8 at `0x14006aafe`, and the laser's own hit the same at `0x14006ad4b` |
| `+0x7854`, `+0x7858` | a `Timer` the hit starts over |

### The hit test, which is a circle and not a box

**`0x14006a980` is what every hazard asks about the player**: `rcx` the player, `rdx` the hazard's
position, `r8` its size, `r9b` whether it is round.

- `0x14006a986`–`0x14006a9e4` works the hazard's edges out — position plus and minus half its size — and
  hands them to **the sixteen boxes** at `0x14006a8c0`, which answers 1 where one holds the hazard.
- **Round** (`r9b` set, which the bullets pass — `movzx r9d,r12b` with `r12d` 1 at `0x140011409`):
  `0x14006aa12` takes the smaller of the size's two floats, halves it, adds `[player+0x774c]`, squares
  the sum and compares it with the squared distance between the two centres.
- **Not round** (the enemies pass nought at `0x140037a40`): `0x14006aa4e` onwards clamps the player's
  position into the hazard's box — the nearest point of the box — and compares the squared distance to it
  with `[player+0x774c]` squared.
- Either way a hit is `0x14006aaf1` writing 2 into `+0x7898`, 8 into `+0x773c`, and the answer 2.

**So the player is a circle here** where th06c's is a `1.2` by `1.2` box, and orb hands the rules the
radius as a half on each axis and a bullet's smaller half on each of its — boxes that hold the circles
the game tests, so a move the model calls clear is clear of them.

**And a laser's test is `0x14006aba0`**, the same turn-about-the-origin th06c's is: `sin` and `cos`
through `0x1402be07d` and `0x1402be071`, the player's position off the statics `0x140506ad0` and
`0x140506ad4`, and the size halved with `0x14030cd58`'s `0.5` at `0x14006ac5a`.

## The bullets, in `0x1403ec2a0`

The update is `0x140010870`, and the array is read off its own loop:

| | |
| --- | --- |
| where the array starts | `+8` — `lea rbx,[rcx+0x8]` at `0x140010886` |
| how far apart | `0x620` — `add rbx,0x620` at `0x140011578` |
| how many | `0x280` — `cmp r14d,0x280` at `0x140011599` |
| how many are live | `+0xf5008`, counted up at `0x14001097b` |
| one bullet's step | `+0x8`, three floats, added into the position at `0x140010a61`–`0x140010a7f` |
| one bullet's place | `+0x30`, three floats — `lea rdx,[rbx+0x30]` at `0x14001140d` |
| **which of five states it is in** | `+0x44`, a word — `cmp WORD PTR [rbx+0x44],r13w` at `0x140010970`, nought being a slot nothing is in |
| the size the test is given | `+0x5f4`, two floats — `lea r8,[rbx+0x5f4]` at `0x140011402` |
| whether it is armed | `+0x618`, a byte `0x1400113f5` gates the test on |
| its angle and speed | `+0x40` and `+0x24`, out of which state 1 rebuilds the step at `0x140010aff` |

**The state decides what its step is scaled by, and the five are th06c's**: states 2, 3 and 4 divide one
by `2.0`, `2.5` and `3.0` — `0x140010a08`, `0x140010a1e` and `0x140010a2f` load `0x14030cdd8`,
`0x14030cdf0` and `0x14030ce10` — and states 1 and 5 add the step whole. Only state 1's path reaches the
test at `0x140011418`.

## The lasers, in that same static

**`0x1403ec2a0 + 0xf5018` onwards**: the update walks sixty-four of them from `0x1400115ac` with its
pointer `0x14` into each — `lea rbx,[r15+0xf502c]` — `0x298` apart (`0x14001172f`), `mov edi,0x40`.
Read off that pointer and written down from the struct's own start:

| | |
| --- | --- |
| `+0x4` | **how wide it is**, halved at `0x14001167a` into the size the test is handed |
| `+0x8`, `+0xc` | the frames it has been in its phase, the pair a `Timer` is |
| `+0x10`, `+0x14` | **how far along it runs**, near end and far: the far end grows by `+0x26c` a frame at `0x14001161d`, and the near end follows once the two are `+0x288` apart |
| `+0x18` | how many frames it fades over |
| `+0x260` | **which way it points** |
| `+0x264` | flags, `0x1` tested at `0x1400117b1` |
| `+0x268` | how many frames it fires for — `0x140011888` compares the frames with it and moves the phase on |
| `+0x26c` | how much further the far end reaches each frame |
| `+0x270` | **how many frames its warning runs for**, compared at `0x14001182f` |
| `+0x274` | whether the slot is in use — `cmp BYTE PTR [rbx+0x260],r13b` at `0x140011610` |
| `+0x278` | **where it is fired from** |
| `+0x288` | how long it is once grown |
| `+0x290` | its phase: nought the warning, 1 firing, 2 fading, and past that `0x14001170d` turns it off |

**The first fields are th06c's and the rest sit `0x20` further on**, the struct being `0x20` bytes
longer. **And only the firing phase hits**: the one call of `0x14006aba0` is at `0x140011883`, on phase
1's branch — the warning at `0x1400117b1` and the fade at `0x1400118ba` make none.

## The enemies

**The static `0x140aa1e90`**, whose update is `0x1400373b0`: the live count in the manager's first dword
(`inc DWORD PTR [r12]` at `0x1400374ad`), the array at `+8`, `0x10b0` apart (`0x14003818e`), two hundred
and fifty-six of them (`cmp r13d,0x100` at `0x14003819a`).

| | |
| --- | --- |
| `+0xb0` | where it is, three floats — `lea rsi,[rbx+0xb0]` at `0x1400374b1`, handed to the test |
| `+0xbc` | the flags the walk gates on: `cmp BYTE PTR [rbx+0xbc],0; jge` at `0x1400374a0` skips a slot whose top bit is clear |
| `+0xbd` | **and whether its body is run against the player**: `0x1400379f2` requires `0x4`, `0x140037a18` runs the test only where the low two bits are both up, and `0x1400379fa` refuses where `+0xbe`'s `0x8` is set |
| `+0x2c0` | **the size that test is given**, three floats, each multiplied by two thirds — `0x14030cd74` at `0x140037a43` — before `0x14006a980` halves what it is given, so a hit is a third of this on each side |
| `+0x234` | its life, which the body taking the player off it drops by ten at `0x140037a83` |
| `+0xbd`'s `0x8` | **the one the fight is with**: `0x140037d90` and `0x140037fd0` test it and divide the life into the bar the panel draws |

## The items

**A static at `0x140baf0d0`**, walked by `0x140042980`: the array at `+8` (`lea r14,[0x140baf0d8]` at
`0x1400429a4`), `0x160` apart (`imul rdi,rcx,0x160` at `0x140011442`, where the bullets' update reaches
into it), five hundred and twelve of them (`cmp eax,0x200` at `0x14004351d`). A slot whose first byte is
nought holds nothing — `0x140042ae0`. `+0x10` is where it is.

**Its kind is the signed byte at `+0x34`**, and `0x140042d40` jumps through the table of seven at
`0x1400438a8` — whose cases are th06c's in th06c's order: 0 the power (`0x140042d5a`), 1 the point item
(`0x14004306b`), 2 the big power (`0x14004318e`), 3 the bombs stopped at eight (`0x1400433c7`), 4 the
power written to its cap (`0x14004347d`), 5 the 1up (`0x1400433f6`), 6 the one a bomb leaves behind
(`0x140043676`).

## The entry a name is typed into

**It is a state of the screen a run ends at**, reached the one way a heap object of this game's is: the
job it registers in the calc chain. The job a stage ends into, `0x140079c70`, allocates the screen's
`0xbb48` bytes at `0x14007a218`, builds it with `0x14006df00`, marks it a run's own at `+0x9e99`, writes
its state at `0x14007a25e` — **nine** where `[0x1404f27b4]` is clear and seventeen where it is not, the
latter being a run asked for no name — and hands it to `0x14006dd20`, which allocates the `0x40`-byte
element, puts the update `0x14006e3a0` at its `+0x8` and the screen at its `+0x38`. The *Score* item of
the main menu builds the same screen at `0x14000c667` in state twelve with `+0x9e9a` set, which is the
ranking with no run behind it.

| | |
| --- | --- |
| `screen + 0x9e94` | **its own state**, read at `0x14006e3d8` and jumped through a table of eighteen at `0x140072824`: 0 and 1 the result's own, 2 the ranking, 8 and **9 the entry**, 10 to 14 one case, 15 and 16 the way out, 17 the run asked for no name |
| `screen + 0x0` | **the grid cursor**, the object's first dword. `0x140071000` reads it, `0x140071008` takes the character at the grid, `0x140071003` compares it with `0x5e` — the space item, whose case writes `0x20` at `0x140071019` — and `0x140071013` leaves the typing for anything past it. Left and right step it by one at `0x140070ef0` and `0x140070f70`, up and down by sixteen at `0x140070e17` and `0x140070e80`, each skipping a space of the grid |
| `screen + 0x9e70` | **the name, eight bytes.** `0x140071021` writes the character at `[screen + 0x9e70 + index]` |
| `screen + 0x9e7c` | **how many have been typed**, the index above — capped at seven by the `cmovge` at `0x140070ffd`, stepped up at `0x140071033`, stopped at eight by `0x14007102e`, and reaching eight is what `0x140071040` puts the cursor on `0x5f` for. The constructor writes one into it at `0x14006e00c` and other states use it as their own cursor |
| `screen + 0x4450` | the state's own frame count, which the entry reads no key before reaches thirty at `0x140070dca` |

**The grid is at `0x14030c250`** — `ABCDEFGHIJKLMNOPQRSTUVWXYZ.,:;\@abcdefghijklmnopqrstuvwxyz+-/*=%`
and the rest of the 96 — reached off the image base in `r13`, `[rax+r13*1+0x30c250]`, the same way
th06c reaches its own.

**The end item writes the name out**: `0x140071105` copies the eight bytes to `0x140c21970` through
`0x14007bc00` and puts the state at fifteen. **Which is th06c's shape with every offset moved**: that
build has the state at `+0x5194`, the name at `+0x5173` and the count at `+0x517c`.

**And there is a second entry, into `+0x9e88`**, typed in state thirteen at `0x1400718d3` out of the
list state fourteen holds — its end item hands the name to `0x14006c420` together with the byte at
`+0x9e9a` — which no run of a recipe's reaches. What it names is not established.

## The window, which `th06.env` says the size of

**The game's two files are two stretches of its own object at `0x140c21970`**: `th06.cfg`'s 44 bytes at
`0x140c21da4` (`+0x434`) and `th06.env`'s twelve at `0x140c21dfc` (`+0x48c`). `0x14007b5d0` reads both
at startup, called from `main` at `0x14004586f`, with defaults built at `0x14007b6d4` where a file is
missing, and `main` writes both back at exit, `0x140046022` and `0x140046044`. Of the 44, the five words
at `+0x434` are written together at `0x14004a953` by the key config, `+0x449` and `+0x44b` are the
difficulty and the shot last chosen — `0x1400485af` puts the one back under four, `0x14004bdf3` writes
the other off the shot screen's cursor — and the eight at `+0x454` are the name the entry last took,
which `0x14007bc00` writes. **No byte of the 44 is the window.**

**The window is `th06.env`'s.** `[0x140c21e00]` is the screen mode: `main` reads it at `0x140045a88`
after the file and calls one of three functions on the game's object, each of which logs
`ChangeWindowMode実行` through `0x1400b7700` and hands `0x140096cd0` the size the picture is made at:

| mode | function | what it makes | the size |
| --- | --- | --- | --- |
| 0 | `0x14007c560` | a window with a frame — the flag `0x1400b7700(1)` raises logs `ウインドウモードフラグが立てられました` | the item `[0x140c21e01]` names, `0x14007c71c` |
| 1 | `0x14007c320` | a window the size of the monitor it is on, from `0x140083910`'s display mode or `GetSystemMetrics` where there is no window yet | the monitor's, `0x14007c497` |
| 2 | `0x14007c060` | the display taken, the flag dropped at `0x14007c191` and `0x140096b80(2)` | the monitor's mode, `0x14007c18a` |

`0x140045310` flips the mode between nought and one on a key. The window is made after the choice, at
`0x140045b74` through `0x1400c1690`, so the size is chosen before there is a window and nothing orb
answers runs between the read and the choice.

**Each mode has a byte of its own for which of four sizes**, `[0x140c21e01]`, `[0x140c21e02]` and
`[0x140c21e03]` for modes nought, one and two — read at `0x14007c62b`, `0x14007c3ee` and `0x14007c14f`
as `[game + 0x491]`, `+0x492` and `+0x493` — and the option screen steps each at `0x14004e37a`,
`0x14004e298` and `0x14004e28f`. In mode nought the item is the window; in the other two it only
caps the picture, the window being the monitor's either way.

| item | size | where, in mode nought |
| --- | --- | --- |
| 0 | the largest of the four that fits the monitor, tried largest first at `0x14007c694` | |
| 1 | 3840x2160 | `0x14007c680` |
| 2 | 2560x1440 | `0x14007c674` |
| 3 | 1920x1080 | `0x14007c68c` |
| 4 | 1280x720 | `0x14007c66a` |

`0x14007c6aa` writes the item settled on back, and the option screen calls the mode's function on the
running game at `0x14004e74a`, `0x14004e73c` and `0x14004e72e` — which is how `Th06nc::prepare_frame`
sizes the window to `Config::screen` on the first frame: the mode and the item are written, the mode's
function called, and one frame of the env's own window goes by first. The launches that read it: the copy's
env at mode one came up the whole of a 4K monitor, `3862x2216` with its frame; mode one entered on the
running game made the window its monitor's `2160x3840`; mode nought entered with item 4 made it
`1302x776` with its frame, 1280x720 inside, the game drawn whole in it.

**And the picture's size is a function of its own**, `0x140096cd0(width, height, bits, hertz)`, which
every mode function ends in: it keeps the four at `0x14050a198` onwards, refuses bits other than 8, 16 and
32 at `0x140096d37`, takes `640x480` for a size of nought, and remakes the renderer at the size through
`0x1400f0d20` — or `0x140091fc0` where one is up. Where in the window the picture then goes is four
dwords at `0x140c21e3c` — left, top, width, height, letterboxed to 16:9 — which the mode functions write
at `0x14007c518`, `0x14007c79e` and `0x14007c2d2` and the draw reads at `0x14003c374` and forty more
places, with a byte at `0x140c21d14` beside them saying the picture is taller than 1080. A launch that
plays itself is what these are called with a size of orb's own for: the window mode entered with item 4
and then `0x140096cd0(width, height, 32, 60)`, the four dwords written `0, 0, width, height` — measured
at 640x360 as a window of `662x416` with its frame, and at 320x180 as `342x236`, the game drawn whole in
either.

`[0x140c21e04]` is a byte `main` hands `0x140098280` as a flag at `0x140045a83`.

## What is not established

- **Which of the game manager's thirty-two states a run ends on**: the manager is out of the chain from
  the run's start to the title, and the screens between are jobs.
- **Which of screen modes nought and two is the borderless window `Screen::Fullscreen` means**, and what
  `[0x140c21e04]` switches — the setting is applied only where it is a window.
- **What `[0x1404f27b4]` is**, the byte that has a run asked for no name.
- **What the entry into `+0x9e88` names**, and what the states from ten to sixteen of the screen a run
  ends at show.
- **How long the ending runs**, and what `0x14002bac0` reads its text out of: `th06ED.dat` is opened
  from `0x140019154`, and the two have not been joined.
- **What the rest of `th06.cfg`'s 44 bytes hold** — the bytes from `+0x444` to `+0x448`, `+0x44a`, and
  the dword at `+0x450` whose bit `0x10` the option screen tests.
- **Where the line of dialogue that is up is kept.**
- **The stage's own clock and the script's**, the seed, and the deaths.
- **What `0x1404f27c4` and `0x1404f1e60` are** — the mode a 1up counts something down in.
- **What a replay of it is**, and where a score is written.
