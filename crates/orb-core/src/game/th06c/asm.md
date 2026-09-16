# th06c, as read off its exe

What is written here was read off `th06c.exe` of **md5 `acadbc0cabf3bade16c22fb884957bcd`**, 4,039,168
bytes, which is the Steam release of 東方紅魔郷.

**It is its own game here and not another build of th06.** Nothing of `th06/asm.md` holds of it and
nothing here holds of that one: this exe is 64-bit where th06 1.02h is 32-bit, it was built with a
toolchain twenty years later, it keeps its data in files of other names, and it draws through a renderer
th06 has no code for. What the two share is the game somebody plays and not a line of the program that
plays it.

To read it again:

```sh
objdump -d -M intel th06c.exe > th06c.asm     # the whole of .text, once
objdump -p th06c.exe                          # the DLLs it imports, and where each one's IAT starts
objdump -h th06c.exe                          # the sections and where they are loaded
objdump -d --start-address=0x14009d230 --stop-address=0x14009d2b0 -M intel th06c.exe
strings -a th06c.exe                          # the names it resolves at runtime, and its own file names
```

Its image base is `0x140000000` and every address below is a virtual address at that base.

**And that is not where it runs.** The exe carries a `.reloc` and is built with ASLR on, and every launch
measured came up somewhere else — `0x7ff7579d0000`, `0x7ff7be160000`, `0x7ff750260000`,
`0x7ff6178a0000` — so an address from this document is an address in a running process only once the
difference between the base the loader used and `0x140000000` has been added to it. Orb reads the base it
was loaded at and says so in its log's third line:

```text
exe image base 0x7ff6178a0000
```

## What kind of build it is

| | |
| --- | --- |
| the exe | `PE32+ executable (GUI) x86-64`, six sections |
| `.text` | `0x140001000`, `0x284671` bytes |
| `.rdata` | `0x140286000`, `0x54d54` |
| `.data` | `0x1402db000`, `0xca600` |
| `.pdata` | `0x140b7c000`, `0xabc0` — the unwind tables a 64-bit build carries, which `.text`'s own functions can be listed out of |
| `.rsrc` | `0x140b87000`, `0x2aae8` |
| `.reloc` | `0x140bb2000`, `0x87c` |
| what it was built with | `MSVCP140.dll`, `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll` and ten `api-ms-win-crt-*.dll` — MSVC 2015 or later against the universal CRT |
| what it is sold through | `steam_api64.dll`, beside the exe and imported outright |

**Its files are its own too**: `th06ST.dat`, `th06CM.dat`, `th06ED.dat`, `th06FN.dat`, `th06IN.dat`,
`th06MD.dat` and `th06TL.dat` beside the exe where th06 1.02h has `紅魔郷ST.dat` and the rest; `bgm/`
holding `th06_01.mid` and its neighbours where 1.02h streams `.wav`; `th06.cfg` and `th06.env` where
1.02h keeps `東方紅魔郷.cfg`; and `data/title/th06logo.dds`, a DDS where 1.02h has its own format.

## What it imports, and what it asks for by name instead

Seventeen DLLs are imported. What `USER32.dll` gives it is the whole of what it takes from the window
manager at load time, and the list is short enough to be worth having whole — its IAT begins at
`0x140286230`:

`AdjustWindowRectEx`, `CallNextHookEx`, `CheckDlgButton`, `CheckRadioButton`,
`DialogBoxIndirectParamW`, `EndDialog`, `EnumDisplayMonitors`, `GetAsyncKeyState` (`0x140286238`),
`GetKeyboardLayout`, `GetSystemMetrics`, `GetWindowLongPtrA`, `GetWindowRect` (`0x140286230`),
`IsDlgButtonChecked`, `MapVirtualKeyExW`, `MessageBoxA`, `MonitorFromWindow`, `SetWindowLongPtrA`,
`SetWindowPos`.

**And what is not in it is the point**: no `CreateWindowEx`, no `RegisterClass`, no `PeekMessage`, no
`GetMessage`, no `DispatchMessage`, no `DefWindowProc`, no `ShowWindow` — and no Direct3D at all. Those
names are in `.rdata` as *strings* instead, which is what `GetProcAddress` is given: `CreateWindowExA`
and `CreateWindowExW`, `RegisterClassA`/`W`/`ExA`/`ExW`, `PeekMessageA`/`W`, `GetMessageA`/`W`,
`DispatchMessageA`/`W`, `DefWindowProcA`/`W`, `ShowWindow`, `D3D11CreateDevice`, `CreateDXGIFactory`,
`CreateDXGIFactory1`, `CreateDXGIFactory2`, `XInputGetState`. `KERNEL32.dll` gives it `LoadLibraryA`,
`LoadLibraryW` and `GetProcAddress` (`0x1402860b0`, called from 344 places).

So **a patch over this game's import table reaches almost nothing of it.** What th06 1.02h has orb walk
in — an import to overwrite, a fixed call to the device's own `Present` — is here a pointer the game
looked up and put somewhere of its own.

## Where it keeps what it looked up

**A table of function pointers in `.data`, filled once at startup.** The walk is one `LoadLibrary` per
module and then one `GetProcAddress` per name, unrolled — no loop and no table of names to walk, so every
answer has a slot of its own and every slot has one writer. Three import slots are the whole of the
machinery: `GetProcAddress` at `0x1402860b0`, `LoadLibraryW` at `0x1402860b8` and `LoadLibraryA` at
`0x1402860c0`.

Three hundred and forty-two names are resolved that way. To list them again — the module each came from,
the name, and the slot its answer lands in — pair each `lea rdx` with the store that follows its call:

```sh
objdump -d -M intel th06c.exe > th06c.asm
python3 - th06c.asm <<'PY'
import re, subprocess, sys
strings = {}
for line in subprocess.run(["strings", "-a", "-t", "x", "th06c.exe"],
                           capture_output=True, text=True).stdout.splitlines():
    at, _, text = line.strip().partition(" ")
    try:
        strings[int(at, 16) - 0x284c00 + 0x140286000] = text   # .rdata: file 0x284c00 loads at 0x140286000
    except ValueError:
        pass
LEA = re.compile(r"^\s+[0-9a-f]+:.*lea\s+(rcx|rdx),.*#\s*(0x[0-9a-f]+)")
CALL = re.compile(r"^\s+([0-9a-f]+):.*call\s+QWORD PTR \[rip.*#\s*(0x[0-9a-f]+)")
STORE = re.compile(r"^\s+[0-9a-f]+:.*mov\s+QWORD PTR \[rip[^\]]*\],rax.*#\s*(0x[0-9a-f]+)")
module, pending, awaiting = "?", {}, None
for line in open(sys.argv[1]):
    if found := LEA.match(line):
        pending[found.group(1)] = strings.get(int(found.group(2), 16), found.group(2))
    elif found := CALL.match(line):
        if found.group(2) in ("0x1402860b8", "0x1402860c0"):
            module, awaiting = pending.get("rcx", "?"), None
        elif found.group(2) == "0x1402860b0":
            awaiting = (module, pending.get("rdx", "?"), found.group(1))
    elif (found := STORE.match(line)) and awaiting:
        print(f"{awaiting[0]:<14} {awaiting[1]:<24} -> {found.group(1)}  (call at 0x{awaiting[2]})")
        awaiting = None
PY
```

**A pointer kept in a register is not in that table**, and the renderer's own is one: the module the
`XInput` pair comes from is walked for — `xinput1_4.dll`, then `xinput1_3.dll`, then `xinput9_1_0.dll`,
wide strings in a list on the stack — and its handle is kept at `0x1403f8008`, where `D3D11CreateDevice`
is resolved inside the function that makes the device and left in `r15`.

### The slots orb has business with

| what | slot | where the answer is asked for |
| --- | --- | --- |
| `RegisterClassExW` | `0x1403f8808` | `0x140096292` |
| `CreateWindowExW` | `0x1403f8608` | `0x140095c59` |
| `ShowWindow` | `0x1403f8620` | `0x140095caa` |
| `AdjustWindowRectEx` | `0x1403f8688` | `0x140095e09` |
| `SetWindowLongPtrA` | `0x1403f86a8` | `0x140095e75` |
| `SetWindowLongPtrW` | `0x1403f86b0` | `0x140095e90` |
| `PeekMessageW` | `0x1403f8758` | `0x140096040` |
| `GetMessageW` | `0x1403f8768` | `0x140096076` |
| `DispatchMessageW` | `0x1403f87c8` | `0x1400961ba` |
| `DefWindowProcW` | `0x1403f87a0` | `0x140096133` |
| **`GetKeyboardState`** | `0x1403f8988` | `0x1400967a2` |
| `XInputGetState` | `0x1403f8010` | `0x14008cff0` |
| `XInputSetState` | `0x1403f8018` | `0x14008d00b` |
| `CreateDXGIFactory2` | `0x14084d248` | `0x140250839` |
| `CreateDXGIFactory1` | `0x14084d248` | `0x1402508db` |
| `CreateDXGIFactory` | `0x14084d210` | `0x14025097b` |
| `D3D11CreateDevice` | — kept in `r15` | `0x1402500b0` |

**`GetKeyboardState` is the read a run is played by**, which is the one the whole 256-key snapshot comes
through; `GetAsyncKeyState` is imported outright and is not it — see below. Both of the XInput calls are
one site apiece, and the two factory slots are the same slot twice because the walk asks for the newest
name first and falls back.

**And the frame is handed over through a swapchain's own vtable**, which no name resolves: `Present` is a
method on what `CreateDXGIFactory*` and `D3D11CreateDevice` between them make, so where it is called from
is a reading of the function at `0x140250000` onwards and not a slot in the table above.

**What it draws with is a D3D11-class renderer**: `D3DCompiler_33.dll` through `D3DCompiler_47.dll` are
named in `.rdata`, which is the fallback walk an app that compiles shaders at runtime makes, and so are
`DXGI_FORMAT_BC1_UNORM`, `BC2`, `BC3`, `BC7_UNORM` and `BC7_UNORM_SRGB`. **What it plays sound with** is
`DSound.DLL` for the effects and `Mfplat.dll` with `Mfreadwrite.dll` — Media Foundation — for the music.

## The keyboard: one 256-byte state, filled two ways

`0x14008f9c0` onwards is where a frame's keys are read, and what it fills is **the 256 bytes at
`0x1403de9bc`**:

| | |
| --- | --- |
| `0x1403f8050` | an object whose vtable is called at `+0x48` with `(0x100, 0x1403de9bc)` — `0x14008fa08`, again at `0x14008fa36` — and at `+0x38` with nothing, `0x14008fa19`. A device's own state read into that buffer, in other words, with an acquire between the two tries |
| `0x14008fa6c` | `GetKeyboardState` into a 256-byte buffer on the stack, whose `[rsp+0xc1]` — `VK_RSHIFT` — is kept at `0x1403de9f2` |
| `0x14008fac5` | the same read again on the other path, after which `0x1403de9bc` is memset to nought (`0x1400bcb50`, `(buffer, 0, 0x100)`) and filled out of that stack buffer through the table at `0x1402872e0`: 0x41 entries walked as words at `+0x2`, so a mapping from the game's own key numbers to virtual keys |
| `0x1403de9b8` | a timestamp the walk writes — `0x14008fa4a` and `0x14008faa9`, held against `timeGetTime`'s answer (`0x1403f8240`) at `0x14008fa9d`. What it holds to one per tick is the walk; the `GetKeyboardState` that reaches it is made 66 times a second against a game that moves 60, which is measured below |
| `0x1403f8038`, `0x1403f8058` | a handle each: the first decides whether the device path is taken at all (`0x14008f9e2`), the second is waited on through `WaitForSingleObject` (`0x1403f8328`) and reset through `ResetEvent` (`0x1403f82d8`) |

**And the buffer is indexed by DirectInput's own scan codes.** The table at `0x1402872e0` is what the
`GetKeyboardState` path fills it through, walked at `0x14008fb00`: six bytes an entry — a virtual key, the
index to write it at, and two bytes of nought — ending at a key of `0xffff`. `A` is `0x41 -> 0x1e`, `B` is
`0x42 -> 0x30`, `C` `0x43 -> 0x2e`, `D` `0x44 -> 0x20`, `E` `0x45 -> 0x12`, which is `DIK_A`, `DIK_B`,
`DIK_C`, `DIK_D`, `DIK_E`. So the other path needs no translation at all, which is what says what it is:
the object at `0x1403f8050` is a DirectInput keyboard, its `+0x48` is `GetDeviceState` with 256 bytes, and
`GetKeyboardState` is the fallback for a machine it could not be made on.

The five bytes the walk does not go through are the mouse's, read out of the same stack buffer by virtual
key and kept apart: `VK_LBUTTON` at `0x1403e0ac4`, `VK_RBUTTON` at `0x1403e0ac5`, `VK_MBUTTON` at
`0x1403e0ac6`, and `0x1403e0ac7` and `0x1403e0ac8` behind a build number at `0x1404084b8` against `0x108`.

**So a key orb wants the game to see is a `0x80` at `0x1403de9bc` plus that key's own scan code** — or a
virtual key in the 256 bytes `GetKeyboardState` was given, which the game then translates through that
table itself, and which is what orb does.

**Standing in front of the slot is the wrong way to reach that read.** The slot holds `0x0` when orb is
attached — measured, `0x7ff617c98988` read as nought — because the game's own startup is what writes it,
and a pointer put there before that is one the game overwrites. What orb stands in front of instead is the
**lookup**: `GetProcAddress` is imported at `0x1402860b0`, so answering the ask for `GetKeyboardState` with
orb's own read leaves the game writing orb's pointer into its own slot. That needs no address of the
game's at all, which is what makes it hold across a base the loader chose.

**And a key written into those 256 bytes is acted on**, which is the whole reason the read is the seam
worth having: a launch whose keys were a decide held in bursts walked the title menu into its difficulty
select, 「モードを選択してね」 with Easy, Normal and Hard on it.

**But the read is not the tick, and it is made more often than the game moves.** Counted directly,
once orb had a hook on the frame to count against: **683, 692, 736, 761 and 773 reads against every
600 walks of the calc chain**, which is the frame below. So between a seventh and a quarter of the
reads see the world the read before them saw, and the timestamp at `0x1403de9b8` holds the *walk* of
the keys to one per tick and not the `GetKeyboardState` call that reaches it.

Counted against the wall clock instead, one launch's play step was asked for a word 62,400 times in
944.9 seconds — 66.0 a second, the rate of each six hundred of them running from 61 to 76 as the load
moved.

**What that costs is the whole model of a bot driven off this read**, and the earlier figures hid it: 120
reads in 2.0 seconds on the title is 60 a second, which is the rate a game moving 60 times a second has,
and a screen coming up at 300 in 8.4 seconds says nothing about a stage. On a read that sees a tick again
every bullet has moved nought while each still claims a step of its own, which
`orb_bot::steer::A_STEP_OF_ITS_OWN` reads as a bullet a script is moving — and such a bullet is feared
over what it just moved, which here is nothing. The model is then a field of bullets standing still: four
hits of one launch were dumped and three were decided on a read like that, reporting 100 of 100, 53 of 61
and 195 of 195 bullets explained by no step of their own. `orb_bot::steer::a_tick_read_again` is what
answers it, off the reading itself.

**What the screen of this game can be seen with, orb having no hook in its drawing**, is `PrintWindow`
with `PW_RENDERFULLCONTENT` (`2`) from outside the process. `BitBlt` off the screen — `CopyFromScreen` and
the like — comes back black: what it draws through is a DXGI swapchain and its content is in no GDI
surface.

## The one call of `GetAsyncKeyState` is not the run's own read

`0x14009d262` is the only call of it in the exe, and the function it is in — `0x14009d230`, three
arguments — is a filter over a keystroke somebody else read:

| | |
| --- | --- |
| `0x14009d23f`, `0x14009d250` | two flags at `0x1403f9fe0` and `0x140407424`, both against 1, and the function answers 0 where either is not set |
| `0x14009d25b` | and where its first argument is not nought |
| `0x14009d25d` | `0x11` — `VK_CONTROL` — into `ecx` and the call, whose answer is taken by its sign bit (`sar edx,0xf`) |
| `0x14009d270` onwards | its third argument's own `[rbx]` against `0x1b`, `0x09` and `0x73` — Escape, Tab and F4 — and the bits `0x20` and `0x80` of `[rbx+8]` |
| `0x14009d291` | 1 where one of those matched, 0 at `0x14009d2a6` where none did |

Which is a shortcut being swallowed — Control with Escape, Tab or F4 — and not the word a run is played
with. **So the keyboard a run acts on does not come through `GetAsyncKeyState` at all**, and the two
flags above are worth reading for what turns that swallowing on.

## The dialog it puts up before it starts anything

**Every launch, and whatever is beside the exe.** `DialogBoxIndirectParamW` is imported outright
(`0x140286288`), with `EndDialog`, `CheckRadioButton`, `CheckDlgButton` and `IsDlgButtonChecked` beside
it, so the template is built in memory rather than taken from `.rsrc`. Measured: a launch with the
`th06.cfg` and `th06.env` an earlier launch wrote put the same dialog up again.

Its title is `東方紅魔郷：Classic` and it asks 「解像度を選択してください」. Read off the dialog while it
was up — `EnumChildWindows` over it, each control's `GetDlgCtrlID`, class, text and `BM_GETCHECK`:

| control | what it is |
| --- | --- |
| `1` | 「ゲーム起動」, the button that starts the game — `IDOK` |
| `200` | ボーダーレスウィンドウ, and the one the dialog comes up with checked |
| `201`–`205` | フルスクリーン at 最大, 3840x2160, 2560x1440, 1920x1080, 1280x720 |
| `206`–`210` | ウィンドウ at 2560x1920, 1920x1440, 1280x960, 960x720, 640x480 |
| `400` | 「VSyncを有効」, a checkbox, and it comes up ticked |
| `100`, `102`–`111` | the static labels beside them |

**The radio buttons are read when the start button is pressed** and not as they are clicked, which is what
makes the order matter: a launch that clicked `1` first would come up in whatever `200` opened on.

## It needs Steam running, not just `steam_appid.txt`

`SteamAPI_RestartAppIfNecessary` is imported, so a process started outside Steam asks Steam to launch the
game and exits. `steam_appid.txt` beside the exe — holding `4924840`, this game's app id — is what stops
that, and it is **not** what makes the API work: with Steam closed the game prints
`SteamAPI_Init() failed. Steam must be running to play this game.` and exits with its own `log.txt` at the
122 bytes of its header. With Steam running and logged in, the same launch reaches the dialog above.

## The field, and the bullets in it

**The field is 384 by 448 with its centre at (192, 224)**, which is th06 1.02h's own space, and the pair
is used as a centre outright: `0x14001e729` and `0x14001e73e` load `192.0` and `224.0`, add `sin`/`cos`
of an angle times a radius at `[rbx+0x64]`, and hand the result to the bullet manager. So a coordinate of
this game's is read against those numbers and nothing has to be scaled.

**Its width and height are read out of `.data` and not out of an immediate**: `0x1403a33b4` and
`0x1403a33b8`, two floats the startup fills, and every comparison a coordinate is held against goes
through them — `0x14000de6d` and `0x14000de94` in the bullet manager, `0x140020697` and `0x1400206be`,
and `0x140049e49` onwards, which is where the player is put back. `0x140007458` and `0x140007480` turn
the same two into integers, so they are the size in pixels.

**And the rectangle it is drawn in is `0x1402c9720`**, four floats — `32`, `16`, `416`, `464` — loaded
whole by one `movdqa` at `0x140009ec4`, `0x140055ca7` and `0x14005692e`; `0x1402c9738` holds `640` and
`480` beside it. So the field sits at (32, 16) of a 640 by 480 screen and a coordinate of the game's is
the field's own, with nothing added.

**The floats either side of those are one sorted pool and not a table of the field's.** `.rdata` from
`0x1402c9470` to `0x1402c96e0` is every float literal the build has, in ascending order —
`0.1745…`, `0.2`, `0.25`, … `8`, `9`, `10`, … `448`, `464`, … `1e8`, then the negatives — so an address
in it names one constant and says nothing about what the constant is for. What a value's address *is*
good for is the grep that finds the code loading it, which is how the two paragraphs above were read.

**The bullet manager is a static at `0x14090a190`**, `0xe8810` bytes of it, zeroed at
`0x140024d1d`–`0x140024d2f` by the startup that registers every manager. That startup also puts its
update at `[0x1403a28c8]`, its draw at `[0x1403a28d0]` and the manager itself at `[0x1403a28f8]`, which
is the callback chain this game runs a frame through.

Its update is `0x14000d8c0`, and the array is read off that function's own loop:

| | |
| --- | --- |
| where the array starts | `0x14090a190 + 8` — `lea rbx,[rcx+8]` at `0x14000d8d4` |
| how far apart | `0x5d0` — `add rbx,0x5d0` at `0x14000e20a` |
| how many | `0x280`, six hundred and forty — `cmp ebp,0x280; jl` at `0x14000e22b` |
| how many are live | `[0x14090a190 + 0xe8808]`, counted up per live one at `0x14000d9fb` |
| one bullet's step | `+0x08`, three floats — `rsi = rbx+8`, read at `0x14000da29` onwards |
| one bullet's place | `+0x30`, three floats — `rdi = rbx+0x30`, written at `0x14000da62` |
| **which of five states it is in** | `+0x44`, a word — `cmp WORD PTR [rbx+0x44], 0` at `0x14000d9f0`, and nought is a slot nothing is in; `0x14000da02` jumps on it |
| the size the player's own test is given | `+0x5a4`, two floats, handed over at `0x14000e1ae` beside the place — and that test halves what it is given |
| whether it is armed | `+0x5c8`, a byte: nought runs `0x14004c320` and sets it to 1, and only then does `0x14000e1ae` run the player's own test |
| the frame it is on | `+0x28` and `+0x2c`, the second counting up and the first keeping what it was |

**The state decides both whether it can hit and what its step is scaled by**, and there is no one
motion scale on this manager at all:

| state | where it is moved | what its step is multiplied by | tested against the player |
| --- | --- | --- | --- |
| 1 | `0x14000e0a2` | nothing: `[+0x30] += [+0x8]` | **yes** — this is the only path that reaches `0x14000e12b` and the two calls of `0x14004c5d0` |
| 2 | `0x14000da87` | `1.0 / 2.0` | no |
| 3 | `0x14000da9d` | `1.0 / 2.5` | no |
| 4 | `0x14000daae` | `1.0 / 3.0` | no |
| 5 | `0x14000da29` | nothing either: `[+0x30] += [+0x8]` | no |

**The jump at `0x14000da02` is a chain of subtractions and not a table.** `0x14000da06` onwards takes
`r15d` — one — off the state four times over, sending 1 to `0x14000db34`, 2, 3 and 4 to the divisor
chain at `0x14000da82`, and letting 5 fall through to `0x14000da29`, which is where the step is added
whole.

**State 5 was read as a half, and a run contradicted it.** `0x14000da45`'s `mulss xmm1,xmm13` is in
the code that fall-through reaches and `xmm13` is `0x1402c9490`'s `0.5`, so the half looked like the
answer — but what that block scales is `[rsi]`..`[rsi+8]` into `[rdi]`..`[rdi+8]` off the `rbx+0x490`
the `lea` before it takes, and **not** the bullet's own `+0x8` into its `+0x30`. A launch said so
outright: `the widest of them moved 3.30,5.38 saying 1.65,2.69, in the game's own state 5` — twice
what a half predicts, on both axes.

**A flat half over all five is what one launch cost.** Read that way, the rules judged ten of ten
bullets as moved by something other than their own step, so each was feared over the box it had just
swept and none of them was projected: the model could dodge only what was already on the player, and
that run was on its last life at stage 3's boss.

**A state-1 bullet's step is recomputed every frame** where its own flags at `+0x14` say so —
`0x14000db7e` and `0x14000dd38` write `cos(angle) × speed` into `+0x8` off the angle at `+0x40` — so a
bullet that turns is one no straight projection describes, whatever the scale.

**And the whole update is skipped where the byte at `0x1403a3b62` is not nought** — `0x14000d8cd`, which
answers 1 and returns. Six other functions read that byte, so it is the game's own "everything is
held" flag rather than this manager's.

`0x14000c8f0` is what fires one, `__thiscall` on the manager with a descriptor on the stack.

## The lasers, in that same static

**`0x1409f29a8` onwards, and the same update walks them**: `0x14000e23d` puts the array at `+0xe8818`
of the bullet manager — which is that address — and the loop after the bullets' own runs sixty-four of
them, `0x278` apart (`0x14000e3c0`, `0x14000e3c7`). A slot whose `+0x254` is nought holds nothing
(`0x14000e2a0`), and `0x14000e39e` clears it once the near end has run off the far edge. The startup
zeroes the region at `0x140024d36` and `0x14000d650` is what walks it for a free slot.

**What one is is a line and not a box**, which the player's own test for it says: `0x14004c7d0` turns
the player's offset from the laser's origin by its angle — `sin` at `0x140284c01` and `cos` at
`0x140284bf5` — and compares the result with a box that is *along* and *across*.

**And that test halves the size it is given, the same way the bullets' does**: `xmm6` is
`[0x1402c9490]` at `0x14004c857` and `0x14004c89a` and `0x14004c8ac` multiply both of the size's floats
by it. The width at `+0x4` is halved once already on the way in, at `0x14000e30c` — so **a laser reaches
a quarter of its width to either side of its own line**, and reading that as a half is a lane twice as
wide as the game's. What that cost is two hits of an Extra run, each with `nowhere open to it` and
`-5.1 of room` on every one of the nine moves, the nearest laser `8.2` off a player whose lane was
modelled `12.0` wide: the gaps of a fan of lasers all read as shut.

**What the test compares, register by register**, which is what says which of those halvings is the
width's and which axis each float of the pair is on. Its arguments are the box centre in the beam's own
frame (`rdx`, `[rsp+0x40]` at the call site), the size pair (`r8`, `[rsp+0x30]`), the origin (`r9`,
`+0x258`) and the angle on the stack:

- `0x14004c839` and `0x14004c842` take the player's own position, `[0x140b749b8]` and `[0x140b749bc]`,
  less the origin
- `0x140284c01` and `0x140284bf5` are `sin` and `cos` of the angle, and `0x14004c880` and `0x14004c8a7`
  turn that offset into `dx·cos + dy·sin` **along** the beam and `dy·cos − dx·sin` **across** it
- `0x14004c89a` halves the pair's **first** float against the along axis and `0x14004c8ac` its
  **second** against the across axis, so `[rsp+0x30]` is the length and `[rsp+0x34]` the width
- `0x14004c88d` and `0x14004c89e` are the player's own half, `[0x140b749d4]` and `[0x140b749d8]`, read
  straight and not halved
- the four `comiss`/`ja` at `0x14004c8d7` onwards are one box overlap: the player's box about the turned
  offset against the beam's about `rdx`
- and **the origin goes into both sides of that comparison and cancels**: `0x14004c895` and
  `0x14004c8bd` add `+0x258` and `+0x25c` to the turned offset, and `0x14000e331` and `0x14000e33f`
  put the same two into `[rsp+0x40]` and `[rsp+0x44]` beside the along midpoint — so what the test
  compares is the beam's own frame after all, `[near, far]` along it and the width across

So the live path at `0x14000e2f2` hands over `(far − near, width × 0.5)` and the test halves both: the
length is the two ends as they are read, and the width reaches a quarter of itself to each side.

**And a fading one narrows by `frames / +0x18` of its width**: `0x14000e5ed` onwards reads `+0x4` and
`+0xc`, works out `width × (1 − frames ÷ +0x18)`, halves it with the same `[0x1402c9490]` and writes it
to `[rsp+0x30]` — the length's slot rather than the width's, which is not what a narrowing beam would
want and is written down here as what the exe does. `+0x108` beside it gets the same number times
`[0x1402c9464]`, which is `0.0625`.

| | |
| --- | --- |
| `+0x4` | **how wide it is**, and the test is given half — `0x14000e30c` |
| `+0x8`, `+0xc` | the frames it has been in its phase, the pair a `Timer` is |
| `+0x10`, `+0x14` | **how far along it runs**, near end and far |
| `+0x18` | **the frames it goes out over**, which the fraction above is of — and nought here skips the whole of that block (`0x14000e5eb`) |
| `+0x240` | **which way it points** |
| `+0x24c` | **how much further the far end reaches each frame**, added straight on with no scale (`0x14000e2ad`); the near end follows once the two are `+0x268` apart, and stops at nought |
| `+0x244` | a flag the warning and the fading branches both test — `0x14000e44b` and `0x14000e5ae` — and take another way where it is set |
| `+0x248` | **how many frames it fires for**: `0x14000e57c` compares `+0xc` with it and moves the phase on |
| `+0x250` | **how many frames its warning runs for**: `0x14000e517` does the same for phase nought |
| `+0x254` | whether the slot is in use |
| `+0x258` | **where it is fired from**, which is the pivot the test turns the player about |
| `+0x268` | how long it is once grown |
| `+0x26c` | **the frame of its first phase it starts hitting on**: `0x14000e4d1` runs the test only from there, so one still warning is drawn and hits nothing |
| `+0x270` | its phase: nought is that warning, 1 fires every frame (`0x14000e577`), 2 fires and narrows (`0x14000e66d`), and past that `0x14000e394` turns it off |

**And the origin at `+0x258` is not fixed for the life of a laser.** Nothing in the laser's own update
writes it, so it is the script that fired the laser that moves it, and the Extra boss's scripts do: the
wide beam she charges is fired from where she stands and carried with her, `114.0,58.1` on one frame and
`120.4,59.2` on the next — `6.4` across against the player's own `4.0` a frame. What that means for
reading the array is that a beam is a *place this frame* and not a place: the distance between two frames'
readings of one slot is the only thing that says where its line is going, and `+0xc` is what says the
slot still holds the same laser, a sweep giving each new one a slot whose angle, ends and width all match
the last one's.

**A sweep is new lasers and not one laser turning.** Nine of them stood at once in the Extra's own,
`6.4` apart along the row and one created per frame with `+0x26c` of `1` — so each hits from the frame
it appears and none of them is drawn as a warning first. Beside them stood the boss's own, twice their
width, `+0x270` of `1` with `+0xc` at `18` and `+0x26c` at `60`: **past the warning phase the countdown
is not read at all**, so that beam hits every frame of its charge whatever `+0x26c` says, and what
`+0xc` is good for is only saying that the laser in a slot is the one that was there last frame.

## Every function's bounds, out of `.pdata`

A 64-bit build carries a `RUNTIME_FUNCTION` for each function it can unwind, and this one's `.pdata` is
nothing but those: three RVAs each — start, end, unwind info — from file offset `0x3a4000`, ending at the
first entry whose start is nought. There are 3,664 of them, which is what makes a function's *start*
answerable at all in an exe with no symbols: `objdump -d` gives one flat listing of `.text`, and walking
backwards to the nearest `int3` finds a padding boundary rather than an entry.

```sh
python3 -c '
import struct; d=open("th06c.exe","rb").read()
for i in range(0, 0xabc0, 12):
    b,e,_ = struct.unpack_from("<III", d, 0x3a4000+i)
    if b==0: break
    print(f"0x{0x140000000+b:x} 0x{0x140000000+e:x} {e-b:#x}")'
```

**Adjacent entries are often one function and not several.** `0x140049de0`–`0x14004b2d8` comes back as
nine entries whose ranges touch, and the code runs straight through every boundary: the extra entries are
chained unwind info for a function whose prologue is too long for one. So a listing says where a function
*can* begin and the instruction at that address says whether it does — a `mov [rsp+0xd8],r13` with `rdi`
already holding an object is inside one.

## The callback chains, and every job registered in them

**Two lists, and a frame is a walk of each**: the calc chain's head is the static `ChainElem` at
`0x1403a2980` and the draw chain's is `0x1403a2940`. Both are sentinels rather than pointers — an
insertion compares against `[head]` and links through `[head+0x28]`, so an empty chain is
`[0x1403a29a8]` (and `[0x1403a2968]`) being nought, which is the comparison twenty-five places in the
exe make before they walk.

`ChainElem` is `0x40` bytes, and every registration in the exe writes the same shape — read off
`0x140049b46`–`0x140049c6c`, which builds three of them, and confirmed against the inlined one at
`0x140024d5e` onwards:

| | |
| --- | --- |
| `+0x0` | `short priority`, and the list is kept in ascending order of it |
| `+0x2` | a bit set by `or BYTE PTR [rax+0x2],0x1` on every heap-allocated one — th06 1.02h's `isHeapAllocated` |
| `+0x8` | **the callback a frame runs**, called with `+0x38` in `rcx` |
| `+0x10` | the one called **once, as it is registered**, and nulled straight after — `mov rax,[rbx+0x10]; mov rcx,[rbx+0x38]; call rax` at `0x140049ccb`, then `mov [rbx+0x10],rdi` |
| `+0x18` | the one called as it is taken out |
| `+0x20`, `+0x28` | the links, and `+0x28` is the one a walk follows |
| `+0x30` | the element's own address, written by whoever built it |
| `+0x38` | what the three callbacks are called on |

**And the priority is what settles the order of a frame**, which three of them are read off their own
registrations: the input job at `0x140058010` writes nought (`xor edi,edi` at `0x14005852f`, `mov [rbx],di`
at `0x1400585b1`), the player writes 7 (`mov esi,0x7` at `0x140049bf1`, `mov [rbx],si` at `0x140049c7e`)
and the bullets write 11 (`mov r13d,0xb` at `0x140024d8f`, `mov [0x1403a28c0],r13w` at `0x140024d9f`).
Ascending order, so one walk of the chain builds the word, then moves the player on it, then moves each
bullet and tests it against the player where it now stands.

**Which is what says how far ahead a bullet is of the move being judged**: the frame a hit happens on is
one both have taken the same number of steps into, so a model that steps the bullet one further than the
move credits it with a whole step of room the frame has not got — `orb_bot::steer`'s
`THE_NEXT_FRAME_AND_LOOK_AHEAD` is where that pairing is written down.

**So a frame's work is the args of that list**, and the list is what the jobs below were found through.
Each row is one registration site: what it hands over as `+0x38` where that is a static this document can
name, and the three callbacks in the order above. What each job *is* has been established for the three
named ones and for nothing else in the table.

| registered at | what it runs on | the frame's callback | registered | taken out |
| --- | --- | --- | --- | --- |
| `0x140049c70` | `0x140b6d770` — **the player** | `0x14004a3f0` | `0x140049de0` | `0x14004a3c0` |
| `0x140024d7a` | `0x14090a190` — **the bullets** | `0x14000d8c0` | `0x14000ed40` | `0x14000eff0` |
| `0x14000f110` | `0x140b31fe0` | | | |
| `0x1400175ab` | `0x140a32db0`, `0x140a01db0` | `0x140017360` | `0x140017450` | `0x140017530` |
| `0x140017d01` | | `0x140017e50` — **the ending** | `0x140018980` | `0x140018c00` |
| `0x14002026c` | `0x140a32f60` | `0x1400204c0` | `0x1400215d0` | `0x1400216c0` |
| `0x140036df5` | `0x140b5c140` | `0x14002cc00` | `0x140036f60` | `0x140037890` |
| `0x140039a85` | `0x140b7ae28`, `0x140b77a70` | `0x140039bd0` | `0x14003a110` | `0x14003a890` |
| `0x14004d1c0` | | `0x14004d760` | `0x14004d8a0` | `0x14004db60` |
| `0x14004fd63` | | `0x14004feb0` | `0x140054470` | `0x140054a80` |
| `0x140057054` | `0x1403a52c0` | `0x1400561c0` | `0x140056c40` | `0x140057270` |
| `0x1400585a6` | `0x140b77860`, `0x140b77300` | `0x140058010` | `0x1400586e0` | `0x140058b20` |

**And which job an ending is was read off a launch and not off the exe**, which is the way to anything of
this game that neither moves nor is pointed at. `th06ED.dat` is the file one reads and its string sits at
`0x1402c60f8`; **no reference to it is in `.text` at all** — not as a rip-relative displacement, not as an
indexed one with that offset, and not as a `movabs` operand — the same negative the name entry's own grid
gave before it was found through `0x140051f9b`'s indexing instead. So the chain itself is the instrument:
`game::th06c::say_the_jobs_when_they_change` walks it every frame and writes every callback down on the
frame the set of them changes, and one launch of a Normal clear then named the job outright:

```text
f98220   chain: 10 job(s): … 0x14004a3f0 … 0x14000d8c0 0x140026f90 0x14004d760
f98220   chain: 4 job(s): 0x140058010 0x1400072b0 0x140017e50 0x14004d760
f106680  chain: 4 job(s): 0x140058010 0x1400072b0 0x14004feb0 0x14004d760
```

**Which is `8,460` frames of it, in the scene the stage it followed had.** The frame lines across the
whole of that read `scene=11 stage=5 enemies=0 bullets=0 boss=false`, so the game manager's own state says
a stage is still running and the ending is a job drawn over one with nothing left in it — the same shape
the screen a run ends at has, and the reason `Game::read_state` answers `State::in_ending` by asking
whether the chain holds this callback. The screen a run ends at is registered on the frame after this job
leaves, which is what `0x14004feb0` appearing at `f106680` is.

**And `0x1402c9720`'s own statics are past the end of `.data`'s raw bytes.** `objdump -h` says `.data` is
`0xca600` long, which is what is *in the file*; the managers above sit from `0x14090a190` to
`0x140b7ae28`, so the section's virtual size runs to `.pdata`'s `0x140b7c000` and everything between is
zero-filled at load. A read of the game's state that only looks at `.data`'s raw extent finds none of
them — measured: a search for the player's position over `0x1402db000`–`0x1403a5600` and the private
heap came back with heap addresses and no static one.

## The frame, and the loop that runs it

**The loop is `0x14002c4b7`..`0x14002c539`**, four calls and a jump back: something of the window's
(`0x14009b340`, handed two pointers into the frame's own stack), `0x14009e320`, the C++ runtime's
`[0x140286668]`, and then **the frame at `0x1400256f0`**, which it leaves on a non-zero answer from
that or from `0x1400a2db0`. Nothing in the loop waits: what paces a run is inside the frame.

**And the frame is this, in order:**

| | |
| --- | --- |
| `0x14002572e` | a viewport of `0x280` by `0x1e0` — 640 by 480 — with two floats out of `0x140b7768c` and `0x140b77690`, handed to `0x140073ed0` |
| `0x140025769` | `0x14008ad00`, whose answer is a `0x40`-byte matrix copied to the stack, and `0x140075e30` after it |
| `0x1400257af` | **the calc chain walked, unconditionally** — `0x140025500` |
| `0x1400257c7` | and walked **again up to seven more times**, `cmp ebx,0x7`, while `[0x1403a3b48]` is set, `[0x1403a3b61]` and `[0x1403a3b5c]` are clear, `[0x140b7762c]` is 2 and `[0x14090a110]` has `0x8000` |
| `0x14002585d` | how many of those to run, out of the sound: `0x1400a5fd0` and `0x1400a5ed0` answer a position in samples, `0xac44` — 44,100 — stands in where the first answers nothing, and `0x1400aaf30` is handed the result |
| `0x1400258fd` | **the draw chain walked, inlined** — `lea rbx,[0x1403a2940]` and the same walk as below, in the frame's own code and not in a function of its own |
| `0x14002594c` | `0x140025990`, which is the frame's own timing: it calls a clock through the pointer at `0x140b77800`, averages over `0x5f` of them and divides by a hundred |

**And that timing is a frame limiter and not only a measurement.** `0x140025aa0` onwards works out how far
off its own deadline the frame is, turns that into a wait with the frequency at `0x140b77818`, and calls
the pointer at `0x140b77810` with it — then reads the clock again and waits some more while anything is
left, and finally `0x140025af4` spins on `pause` until the deadline itself. So a frame of this game holds
sixty a second by sleeping and then spinning, and orb's own loop does not call it at all: measured, it was
`14.7ms` of a `16.6ms` frame, and a launch at `--pace=2` ran at sixty a second like every other until it
was left out. What goes with it is the `60.00fps` the game draws in its own corner, which the same
function works out and formats (`0x1400216f0` at `0x140025bc3`).

**So a run's rate is the sound's**, not the display's and not a sleep: the frame runs as often as the
loop gets round to it and the calc chain is walked as many times as the samples played say it should
be. Which is what a launch at `--pace=2` measured — 66.0 words a second asked of the rules became
136, and stage 1 took 79 seconds against 169 — the clocks orb answers being what the sound's own
position is read against.

### The chain walk

**`0x140025500` is the calc chain's**, and the draw chain's is the same walk inlined at
`0x1400258fd`. Both start at the head's `+0x8`, call it with `+0x38` in `rcx`, and follow `+0x28`;
what the callback answers decides what the walk does next:

| | |
| --- | --- |
| 2 | call it again, without moving on — `je` straight back to the call |
| 0 | take the element out: `0x14000f350` with the element in `rdx` and the walk already moved on |
| 1 | start the walk over from the head (calc only) |
| 3, 4, 5 | end the walk, each with its own answer for the frame |

**And the calc walk is where the frame's queued sound effects go**: `0x140025584` onwards walks the
three dwords at `0x1403a4dd0`, each an index into `0x1403a4ff4`, and hands every one that is not
`-1` to `0x1400a9730` as `(index, 1, 1)` — the same job th06 1.02h does in
`SoundPlayer::PlaySounds` after its own update.

**What that leaves for orb is an update to stand in and no draw to stand in.** The calc chain has a
function of its own, so one call per walk is a seam a patch can take; the draw chain is inlined in
the frame, so the only seams either side of it are the frame itself and the jobs registered in it.

## The player

**A static object at `0x140b6d770`**, `0x9b88` bytes of it — `0x140049af0` clears it in two pieces,
`0x7374` from the front at `0x140049b11` and `0x2814` from `0x140b74aec` at `0x140049b25`, which is that
address plus that size. The same function builds the three `ChainElem`s of the table above and hands each
of them `0x140b6d770`, at priority 7 in the calc chain and 5 in the draw chain.

| | |
| --- | --- |
| `+0x7248`, `+0x724c`, `+0x7250` | **where the player is**, three floats, and the third is a depth rather than a coordinate — `0x140049e93` writes `0.49` into it and nothing else does |
| `+0x7228`, `+0x722c` | what a frame's movement is scaled by on each axis before it reaches the position — `mulss` at `0x14004af14` and `0x14004af24` |
| `+0x7500` | **which way it is being moved**, one signed byte, and nought is standing still: `0x14004b0e0` reads it, `0x14004b0f3` returns where it is nought, and what follows subtracts one at a time down a chain of comparisons |
| `+0x410`, `+0x414` and `+0x41c`, `+0x420` | **the two corners the game's own hit test compares**, `0x14004c692`–`0x14004c6c8`, and `0x14004afc7` onwards writes both from the position every frame |
| `+0x7264` | **the half those corners are the position plus and minus**: `0x14004afa0` hands it to the subtract at `0x14000fc20` and the add at `0x1400092b0`, and `0x14004c88d` reads it straight as the player's half in the laser's own test |
| `+0x9830` | the half of a second box, written the same way into `+0x9b74` and `+0x9b80`, and what that box is for is unread |
| `+0x7274` onwards | **sixteen boxes of `0x10` bytes** — width, height, and a centre — which the hit test at `0x14004c636` walks before it reaches the player's own: a hazard inside one of them is answered 1 rather than 2, and `0x14004a4db` onwards clears all sixteen every frame |
| `+0x73b8` | **which of four states it is in**, one byte: nought is the only one a hit lands in (`0x14004c6ce` gates on it), 2 is what the hit writes (`0x14004c734`), 3 is the invulnerability `0x14004ab8e` counts down and 1 is what `0x140049de0` and `0x14004a918` leave behind |
| `+0x7374`, `+0x7378` | that countdown, the pair a `Timer` is — the frame `+0x7378` reaches nought is the frame state 3 becomes nought |
| `+0x7254` | **the frames a bomb still takes the hit back in**, taken one off a frame at `0x14004a6ee`; `0x14004a595` is the bomb refusing to fire where it is nought, and the frame it reaches nought is the frame the death is made final — `0x14004a6fd` onwards reads the lives |
| `+0x75cc` | frames of invulnerability, which `0x140009ec0` holds against 60 to pick the alpha it draws the field's flash at, and `0x14004a60b` sets |
| `+0x438` onwards | its own ANM vms, `0x160` apart — `0x14004a0da` onwards writes the same word into eighteen of them |

**The position is clamped to a rectangle in `.data`**, and this is the one measurement that says the pair
above is the position and not a copy of it: `0x14004af46`–`0x14004af9f` puts `+0x7248` back to
`[0x1403a4b84]` where it is under it and to `[0x1403a4b84] + [0x1403a4b8c]` where it is over, and does the
same to `+0x724c` with `[0x1403a4b88]` and `[0x1403a4b90]`. Left, top, width, height, in the field's own
coordinates.

**And `0x140049de0` is where a life starts**, which says where in the field that is: `+0x7248` is
`[0x1403a33b4] * 0.5` and `+0x724c` is `[0x1403a33b8] - 64.0` — the field's width halved, and 64 up from
its bottom.

### What one frame of a direction moves it by

**Four speeds on the player**, and the switch at `0x14004acbf` — a jump table of eight at `0x14004b85c`,
indexed by the direction below less one — picks the pair, while the focus bit picks which of the pair:

| | |
| --- | --- |
| `+0x7380` | straight |
| `+0x7384` | straight, focused |
| `+0x7388` | diagonal |
| `+0x738c` | diagonal, focused |

**And the diagonal pair is the straight one divided by the root of two**, worked out once at
`0x14004a034`–`0x14004a0ac` rather than held as its own number: `movss xmm1,[rbx+0x7380]`,
`divss xmm1,xmm0` where `xmm0` is what `0x140284c07` answered, into `+0x7388`, and the focused pair the
same.

**What reaches the position is that speed times the player's own scale for the axis** — `+0x7228` across
and `+0x722c` down, `mulss` at `0x14004af14` and `0x14004af24` — and nothing else: there is no motion
scale in the player's path, where the bullet manager's has one.

**The frame's own velocity is kept on the player too**, `+0x73bc` across and `+0x73c0` down, written at
`0x14004af09` and `0x14004af1c` from the pair the position was moved by.

### The word a frame is played with

**`0x14090a110` is the word the run acts on**, and `0x14090a114` is the frame before it — the pair an
edge is taken from, held against each other at `0x14004a5c3` where a bomb is pressed. `0x14002cc79`
clears both as a run starts. `0x14090a118` and `0x14090a12c` are cleared with them and what they are is
unread.

Its bits are th06 1.02h's own, read off what tests them:

| | |
| --- | --- |
| `0x01` | shot |
| `0x02` | bomb — `and edx,0x2` at `0x14004a5ba`, and what follows takes one off the bombs in stock |
| `0x04` | focus — `and r9d,0x4; setne al` into `+0x737c` at `0x14004ac9d` |
| `0x08` | tested at `0x14002825d` and `0x1400284a5`, and what it is is unread |
| `0x10`, `0x20`, `0x40`, `0x80` | up, down, left, right — `0x14004ac69` onwards |

**The eight directions are `+0x7270`**, written by that same walk and nought where nothing is held. Which
number is which way is read off the jump table at `0x14004b85c`, which the walk enters at that number less
one and whose eight cases each pick a sign for each axis: **1 up, 2 down, 3 left, 4 right, 5 up and left,
6 up and right, 7 down and left, 8 down and right**.

**And two directions of one axis held at once is not a stand-still: the walk keeps the first it tests.**
`0x14004ac69` tests `0x10` and takes that branch whatever else is set, so `0x20` is never looked at with it
— **up beats down** — and inside each branch `0x40` is tested before the byte's sign bit, the sign
overwriting what `0x40` chose, so **right beats left**. `0x20` alone with neither `0x40` nor the sign
leaves `+0x7270` at 2 and moves down as its own case.

**Which makes a word holding both of an axis worse than a word holding neither**, and it is a word orb can
write by accident: `Th06c::presses_into_its_own_keys` writes the keys a word holds into the buffer below,
and a read of the game's own between two of orb's frames leaves the previous word's keys in it. Measured —
`the word the run acts on is 0x81f1` in a hit's dump, up, down, left and right at once, where the rules had
chosen `0x050`: the player went up and right out of a move chosen up and left, and six hits of one Normal
run were that same reversal.

### Where that word comes from

**The job at `0x140058010`**, whose calc-chain priority is nought and which therefore runs first of the
frame: `0x140058098` copies `0x14090a110` into `0x14090a114`, `0x1400580a4` calls `0x14000f7b0` for the
new word, and `0x1400580af` writes it back. So the word is built, the player moves on it and the bullets
test against where it moved, all inside one walk of the calc chain.

**And `0x14000f7b0` builds it out of the 256 bytes at `0x1403de9bc`**, which the read at `0x14008fac0`
fills: `GetKeyboardState` into the stack, `0x14008fad7` memsets those 256 bytes to nought, and the walk at
`0x14008fb00` puts `snapshot[entry.key]` at `buffer[entry.index]` for every entry of the table at
`0x1402872e0` until its `0xffff`. **So the game's own read is a whole state and not a set of presses** —
every key nobody holds is written up — and that read is made at its own rate rather than once a frame.

## The hit test itself, and what it halves

`0x14004c5d0` is what every hazard asks about the player, and it is two boxes overlapping:

- `xmm3` is `[0x1402c9490]`, **a half**, and every size that reaches this function is multiplied by it:
  the hazard's own, handed in `r8` — `[r8]` and `[r8+0x4]` at `0x14004c5e9` and `0x14004c5f0`, each
  `mulss xmm3` — and the player's, `mulss` at `0x14004c647` and `0x14004c66a`. So a `SIZE` field is a
  width and what a hit is is half of it on each side, which is what orb reads them as.
- The hazard's place is `rdx`, and the edges are that place plus and minus its own half:
  `0x14004c622`–`0x14004c632`.
- **Sixteen boxes of the player's come first.** `lea rax,[rcx+0x7274]` and the walk at `0x14004c636`
  onwards compares the hazard against each of them and answers 1 rather than 2 where one contains it;
  a slot whose first float is nought is empty and skipped, which `0x14004a4db` onwards makes of all
  sixteen every frame. **That prologue is not the player's own box**, and reading it as one is a player
  modelled as a point: a launch that did read `half of it 0.0,0.0` off it had nowhere open to it on
  every frame of a stage.
- **The player's own box is the two corners at `+0x410` and `+0x41c`**, compared at
  `0x14004c692`–`0x14004c6c8`, which is `+0x7264`'s half either side of the position. It comes out
  `1.2` by `1.2`. They are absolute and they are the position's: a hit's dump read them against
  `+0x7248` and got `193.4,365.4..195.9,367.9` against a position of `194.6,366.6`.
- **And the bullet is already where its step took it.** `0x14000e0a7`–`0x14000e0c7` adds `+0x8`,`+0xc`
  into `+0x30`,`+0x34` and `0x14000e1b5` hands *that* to the test — so the place compared is this frame's
  and not the place the bullet was read at.

## The field the player may move in

**`0x1403a33ac` onwards, four floats initialised in `.data`** — `32`, `16`, `384`, `448`. The first pair
is where the field sits on a 640 by 480 screen and what `0x140009ddd` and `0x140009df4` add to a position
to draw it; the second is the field's own size, and every comparison a coordinate is held against reads
`0x1403a33b4` and `0x1403a33b8`.

**`0x1403a4b84` onwards is a second rectangle, and it is the one the player is held to** — left, top,
width, height, read at `0x14004af46`–`0x14004af9f`. It initialises to nought in `.data` and no
instruction in the flat listing writes it; what does is not established, and the flat listing is not
evidence of that — see the note on `.pdata` above.

## The run's own numbers

| | |
| --- | --- |
| `0x1403a4b94` | **lives**, one byte. `0x14002b538` stops it at 8 and `0x14002b547` steps it up; `0x14004a6fd` is where a death reads it |
| `0x1403a4b95` | **bombs**, one byte, the same shape at `0x14002b509` and `0x14002b518`, and what `0x14004a5f2` takes one off when a bomb is pressed |
| both together | `0x14002cb40` writes `0x302` over the pair as a run starts — two lives and three bombs |
| `0x1403a4b70` | a dword something multiplies into at `0x14002b7e0` |
| `0x1403a4b7c` | a dword `0x14001529a` adds `0x708` to and `0x14001ff83` steps up by one |
| `0x1403a33c4` | **the power**, a word. `0x14002aa8e` is the `0x80` every case of the item walk writes it back at, so 128 is where it stops; `0x140027096` is the `Power * 100` line of the bonus screen; and `0x14004a76c` takes 16 off it on a death, or to nought where it had less |
| `0x1403a33c0` | **the stage, counted from one**. `0x140027065` multiplies it by a thousand into `Stage * 1000 = %5d`, and `0x14002712e`, `0x140027024` and `0x1400272b4` all treat six as the last — which is what gives the lives and bombs their bonus on an all clear and nowhere else |
| `0x1403a3b68` | **the difficulty**, nought for Easy up to four for the Extra: `0x1400271a3` picks between `Easy Rank * 0.5`, `Normal Rank * 1.0`, `Hard Rank * 1.2`, `Lunatic Rank * 1.5` and `Extra Rank * 2.0` in that order |
| `0x1403a3b50` | **the score**, which every case of the item walk adds to |
| `0x1403a4b70` | **the graze**, `0x1400270c7`'s `Graze * 10` |
| `0x1403a3b64` | **the point items collected**, a word `0x14002b1d8` steps up beside the score |

**The game manager is the static `0x140b5c140`**, `0x11628` bytes, and its state is the dword at
`+0x11608`: `0x14002cc3b` reads it and `0x14002cc58` jumps through a table of twenty-one at
`0x140036324`. Its frame callback is `0x14002cc00`, which is also what clears the input word above.

**Twelve to seventeen share one entry**: `0x14002f95d` takes twelve off and jumps through a second table
of six at `0x14003639c`, whose first case calls `0x1400227c0` and writes thirteen back into `+0x11608`.
**They are not a stage.** A launch that steered only in those six never steered at all — its run was over
at `得点 000001800` with the lives row empty and the game asking 「コンティニュ〜？」 — and what state 13
does is copy `0x70`-byte records into a list at `0x14002f9c4` onwards, which is a screen of the front end
and not a stage.

## The front end

**Its cursor is `0x140b5c140 + 0x28`**, one dword and one field whatever list is up: `0x140030650` steps
it up and wraps it at seven, and the decide at `0x140030695` reads the record of the item it names
through `+0x11610`. That record's own bytes are what a run is started from — `+0x7` into
`0x1403a3b68`, the difficulty, and `+0x6` split in two at `0x1400306ff` and `0x140030713` into
`0x1403a33bc` and `0x1403a33bd`.

**And its menus repeat a held key rather than taking one edge from it.** Measured by walking one: a
launch whose presses were thirty frames long moved two screens on one press and three on another — its
own `front end:` lines went 6, 7, 9 across steps that pressed nothing — so the aim written for the
difficulty was a step later than the difficulty and the run came out Lunatic three launches in a row.
Six frames a press is one item, and that is what a recipe of this game presses with; thirty is what
th06 1.02h and th07 want, whose menus do take the edge.

**Which screen is which number is read off a launch's own lines and not off the exe.** Walking a fresh
launch into a run said the game manager's state each time it changed, and what it said was:

| | |
| --- | --- |
| 0 | what a launch comes up through |
| 1 | **the logo screen, and what starts the demo**: `0x14002cf7d` steps `+0x9454` up, any bit of the input word resets it to nought, and `0x14002cfa4` jumps to `0x14002f906` — the demo's own start — once it reaches `0x2d0`, which is 720 frames. A press moves it on to state 2 instead. See the section after this one, the demo itself not being this state |
| 2 | the title, and its first item is what starts a run |
| 6 | what the title's first item opens, and **it takes no press**: its line arrives and the screen becomes 7 half a second later with every step of the recipe pressing nothing. A recipe that presses here presses at the difficulty instead, and takes whatever item the score file left that cursor on — Lunatic, on three launches running |
| 7 | **the difficulty**, and its items *are* the difficulty: a launch whose step overshot and left the cursor on item 3 played `run: stage 1 on difficulty 3` — Lunatic — so item 1 is Normal |
| 9 | the list after it, which takes two presses on some launches and one on others |
| 11 | **a stage**, and it stays 11 for the whole of one: `run: stage 1 … scene 11` and then `run: stage 2 … scene 11` |

Which of the twelve that are left is which is unread. **Where each one's code is, is not**: the table of
twenty-one at `0x140036324` holds an RVA apiece, and `0x14002f95d`'s second table of six holds the six
that twelve to seventeen share, so a state's case can be disassembled without a launch to reach it —

```sh
python3 -c '
import struct; d=open("th06c.exe","rb").read()
def at(va): return va-0x140001000+0x400
def case(table, i): return 0x140000000+struct.unpack_from("<I", d, at(table)+4*i)[0]
for i in range(21): print(f"state {i:2d} 0x{case(0x140036324, i):x}")
for i in range(6):  print(f"state {12+i:2d} 0x{case(0x14003639c, i):x}")'
```

— which says state 0 is `0x14002cc79`, 1 `0x14002cf7d`, 2 `0x14002d419`, 11 `0x1400352c3`, and the six
`0x14002f979`, `0x14002f98d`, `0x14002fbaa`, `0x14003003b`, `0x14003001d` and `0x1400305c6`.

## The demo, and a replay being played back

**Two bytes, and the demo is the narrower of them.** Both are `.data` and both are one byte:

| | |
| --- | --- |
| `0x1403a3b48` | **a replay is being played back.** `0x1400306ab` sets it where the replay list's own decide runs, `0x14002f90d` sets it where the logo screen starts the demo, and `0x14002cb87` clears it — in the same code that writes `0x302` over the lives and the bombs at `0x14002cb40`, which is a run starting. What reads it is `0x140024c45`: set, the path at `0x1403a4bfc` goes to `0x14004d0d0` to be *read*; clear, `replay/th6_00.rpy` at `0x1402c7a18` goes to that same call from `0x140025206` to be written |
| `0x1403a3b61` | **and the playback is the attract demo.** `0x14002f90d` is the only writer that sets it and `0x140037842` the only one that clears it |
| `0x1403a4bfc` | **the path being played.** `0x14002f921`–`0x14002f93b` copies the 21 bytes of `data/demo/demo00.rpy` (`0x1402c8228`) over it; `0x1400306d0` copies the NUL-terminated name out of the record the replay list's cursor is on. How long the buffer is, is unread |
| `0x140b5c140 + 0x9454` | **how long the logo screen has been idle**, in frames — see state 1 above |

**The demo is played inside whichever screen started it.** Its start at `0x14002f906` sets both bytes,
writes the path, and puts `3` into the stage at `0x1403a33c0` and `3` into the difficulty at
`0x1403a3b68` — `r13d`, which the frame callback's prologue loads at `0x14002cc65` — and then returns
**without writing the manager's state at all**. So the state stays whatever it was: one launch's own
lines read `scene 1` for the whole of a demo started off the logo screen and `scene 2` for one started
off the title, with a run's own lives and bombs moving underneath — `2 live and 3 bomb(s)`, then a bomb
spent.

**And the state is not what tells a demo from a run.** Another launch read
`stage 7 on difficulty 4, scene 11` while the screen said `DEMO PLAY` over stage 4's library, and a
third `stage 4 on difficulty 1, scene 11` for 34,200 frames of one. Neither of those pairs is the `3`
and `3` the start writes, so the numbers a playback ends up showing come out of the replay's own header,
and where *that* is read is unread — which leaves `0x1403a3b48` as the only thing that answers, and is
why `sight::in_a_run` asks it rather than asking the scene.

**What said it was a demo was a picture**: ` DEMO PLAY` at `0x1402c8080`, which `0x1400299b7` draws in
place of the stage's own name — `Stage %d` at `0x1402c8050` and the two at `0x1402c8060` and
`0x1402c8070` — while `0x1403a3b61` is set. The same byte gates the frame's own fast-forward at
`0x1400257d0`: `0x1400256f0` runs the calc chain up to seven extra times where a replay is playing
(`0x1403a3b48`), the playback is *not* the demo, `0x1403a3b5c` is clear, `0x140b7762c` is 2 and
`0x14090a110`'s `0x8000` is held.

**And the hand-over back to the title tells them apart as well.** `0x140036f60` is what a stage ends
into: `0x140037489` tests `0x1403a3b61` and, where it is set, jumps to `0x140037836` — which clears that
byte and `0x1403a4b98` and returns, without the `bgm/th06_01.mid` at `0x1402c8260` that a run's own
ending gets. `0x1403a3b48` is not cleared there, so it stands through the title a demo hands back to and
until the next run starts.

## The replay a run records, and the name it is registered under

**The replay manager is the pointer at `0x14090a100`**, and `0x14004d0d0` is the one call that reads or
writes a file: `cl` says which — `1` reads the path at `0x1403a4bfc`, nought writes `replay/th6_00.rpy`
from `0x140025206` — and `0x14004d4a4` is where the record is made when the manager has none.

**The record is `manager + 0x70`**, `0x70` bytes from `0x140283d20`, and is the replay file's own header:

| | |
| --- | --- |
| `+0x0` | `T6RP`, written as the dword `0x50523654` at `0x14004d4f1` |
| `+0x4` | a word, `0x103` at `0x14004d511` |
| `+0x6` | the character doubled plus the shot — `[0x1403a33bc] * 2 + [0x1403a33bd]` at `0x14004d4f7`, the same pair the front end's own decide split in two |
| `+0x7` | the difficulty, off `0x1403a3b68` at `0x14004d51b` |
| `+0x10` | the date, `MM/DD/YY` and a NUL |
| `+0x19` | **the name, eight bytes**, initialised to `NO NAME` by the `movabs` at `0x14004d525` |

**The file is encrypted from `+0x10` with the byte at `+0x0e`, the key climbing by seven** — and the
first byte takes the key already stepped once, which is the one difference from th06 1.02h's own:

```sh
python3 -c '
d=open("replay/th6_01.rpy","rb").read(); k=(d[0x0e]+7) & 0xff; out=bytearray()
for i in range(0x10, 0x30):
    out.append((d[i]-k) & 0xff); k=(k+7) & 0xff
print(bytes(out))'
```

which answers `09/13/26\x00AAAAAAAA\x00` for a run of this copy — **and those eight `A`s are what says
where the name entry is.** Nothing in the exe holds eight `A` bytes, so they were typed: that run was
played by the rules, which hold the shot key, and the entry took the first item of its grid eight times
over. The entry is therefore reached while the game manager's state is still 11 and the rules are still
being asked for a word — after the last boss, with the field empty, which one launch held for 8,800
frames before the state became 1.

**The grid is at `0x1402c8af0`** — `ABCDEFGHIJKLMNOPQRSTUVWXYZ.,:;¥@abcdefghijklmnopqrstuvwxyz+-/*=%`
and the rest of the 96 th06 1.02h offers, `¥` being the `0x5c` that build has too.

### The entry itself

**It is a state of the screen a run ends at**, and that screen is reached the one way a heap object of
this game's is: the job it registers in the calc chain. `0x14004fcdd` allocates the `0x40`-byte element,
`0x14004fd04` puts the update `0x14004feb0` in it and `0x14004fd3d` puts the screen itself at the
element's `+0x38` — so a walk of the chain from `0x1403a2980` for that callback finds the screen.

| | |
| --- | --- |
| `screen + 0x5194` | **its own state**, read at `0x14004fee1` and jumped through a table of eighteen at `0x140053538`. `0x14004fd59` writes it as the screen is built: **nine** where `[0x1403a3b60]` is clear and seventeen where it is not, the latter being a run asked for no name |
| `screen + 0x0` | **the grid cursor**, the object's first dword. `0x140051f93` reads it, `0x140051f9b` takes the character at the grid, `0x140051f96` compares it with `0x5e` — the space item, whose case writes `0x20` at `0x140051fac` — and `0x140051fa6` leaves the typing for anything past it |
| `screen + 0x5173` | **the name, eight bytes.** `0x140051fb4` writes the character at `[screen + 0x5173 + index]` |
| `screen + 0x517c` | **how many have been typed**, the index above — capped at seven by the `cmovge` at `0x140051f90`, stepped up at `0x140051fc8`, stopped at eight by `0x140051fc1`, and reaching eight is what `0x140051fd3` puts the cursor on `0x5f` for |

**Which is th06 1.02h's own shape with the cursor moved to the front**: that build has the name at
`[this+0x5193]`, the count at `[this+0x10]` and the cursor at `[this+0x20]`.

**And the grid is reached off the image base in a register**: `movzx edx,[rax+r11*1+0x2c8af0]` at
`0x140051f9b`, `r11` holding `0x140000000` from the `lea` at `0x14004ff01`. That is why no search of the
exe finds a reference to it — **no rip-relative displacement anywhere in `.text` resolves to
`0x1402c8af0`, to the pointer at `0x1402c63f0` that holds it, or to `score.dat` at `0x1402c7818`, and
none of the three is a `movabs` operand**, swept by reading every four-byte window of `.text` as a
displacement and the whole file for the addresses as words. **So a negative from that sweep says
nothing about this exe**: it keeps its own base in a register and indexes with a 32-bit displacement.

## The enemies

**A static of `0xff020` bytes at `0x140a32f60`**, whose frame callback is `0x1400204c0` and whose array
that function's own loop says the shape of: the live count in the manager's first dword
(`inc DWORD PTR [r14]` at `0x14002059d`), the array at `+8`, `0xfd0` apart (`0x140021097`), two hundred
and fifty-six of them (`cmp r15d,0x100` at `0x1400210a5`).

| | |
| --- | --- |
| `+0xa8` | where it is, three floats — `lea rbp,[rbx+0xa8]` at `0x1400205a0`, and `+0xfa4` onwards is what the walk adds to it |
| `+0xb4` | the flags the walk itself gates on: `cmp BYTE PTR [rbx+0xb4],0; jge` at `0x140020590` skips a slot whose top bit is clear, so `0x80` is in use |
| `+0xb5` | **and whether its body is run against the player at all**: `0x140020ac2` requires `0x4`, `0x140020aef` runs the test only where the low two bits are both up, and `0x140020adc` refuses where `+0xb6`'s `0x8` is set |
| `+0x2a4` | **the size that test is given**, three floats, each multiplied by two thirds at `0x140020af5` before `0x14004c5d0` — which halves what it is given, so a hit is a third of this on each side |
| `+0x21c` | its life, which the body taking the player off it drops by ten at `0x140020b54`, and `+0x220` is the most of it |
| `+0xb5`'s `0x8` | **the one the fight is with.** `0x140020f62` is the only test of that bit, and for the enemy carrying it `0x140020f7b` works `+0x21c / +0x220` out into `0x14090a0cc` — which is the life bar the panel draws, so whoever has the bit is the boss. Nothing keeps a pointer to it |

## The items

**A static of `0x2a160` bytes at `0x140b31fe0`**, walked by `0x14002a970`: the live count in its first
dword, the array at `+8`, `0x150` apart (`0x14002b932`), five hundred and twelve of them
(`cmp r9d,0x200` at `0x14002b956`). A slot whose first byte is nought holds nothing — `0x14002aae0`.

**Its kind is the signed byte at `+0x34`**, and `0x14002ad97` jumps through a table of seven at
`0x14002ba1c`. What each number is is what its own case does:

| | |
| --- | --- |
| 0 | `inc r15w` into the power at `0x14002ae2e` — a power item |
| 1 | the point items and the score stepped up at `0x14002b1d8` — a point item |
| 2 | `add r15w,0x8` into the power at `0x14002b29e` — the big power item |
| 3 | the bombs, stopped at eight and stepped up at `0x14002b509` |
| 4 | the power written to its cap from `0x14002b5eb` — the one that fills it |
| 5 | the lives, the same shape at `0x14002b538` — a 1up |
| 6 | a score off the graze at `0x14002b7db`, and the bullet manager spawns this one as kind 6 at `0x14000e19d` where a bomb clears a bullet |

`+0x10` is where it is, three floats, and `0x14002ad68`–`0x14002ad91` is the box around the player it is
collected inside — `0x140b772e8` onwards, three floats this reading has not gone into.

**And the bullet's own size is `+0x5a4`**, which `0x14000e1ae` hands to `0x14004c5d0` beside the position
at `+0x30`: that is the box the player is tested against, and `+0x5c8` is the byte the graze sets.

## What is not established

- **Where the `HWND` the game plays in is kept**, and what its class's `WndProc` does with a message:
  the calls that make the window and pump it are the slots above, and what they are given and what is
  done with what they answer is unread.
- **Which of the draw chain's jobs calls `Present`**, and from what address. What the swapchain *is* is
  established and needed no address at all: `0x140250839` asks `GetProcAddress` for
  `CreateDXGIFactory2` — the string is at `0x140294970`, and the module it asks of is kept at
  `0x14084d210` — so orb answers that name, hands the game a factory of the host's own with
  `CreateSwapChainForHwnd` written over, and takes the swapchain the game then makes. A launch says so:

  ```text
  hook: the game asked for CreateDXGIFactory2 and got orb's, the host's being at 0x7fff4d1bf290
  frame: the swapchain this game makes will be orb's to present through
  frame: both of the presents are orb's to call now, the game's own doing nothing
  ```

  **And it is not among the objects the release names.** `0x14024ff74` onwards releases `0x14084d228`,
  `0x230`, `0x238`, `0x240`, `0x248` and `0x260`; a launch asked every one of those and every pointer in
  the renderer's own object besides — nine objects whose vtable is a mapped image's — and not one
  answered to `IID_IDXGISwapChain`. So the swapchain is kept somewhere no reading of the exe has found,
  and the factory is the way to it.
- **What it asks `steam_api64.dll` for**, and what it does where that answers nothing.
- **Which of the game manager's twenty-one states is which screen**, past the six named above, and which
  of them a run is over on.
- **Where the line of dialogue that is up is kept.**
- **The stage's own clock and the script's**, the seed, and the deaths.
- **What `0x1409f29a8 + 0x9e00` onwards is.** Sixty-four lasers of `0x278` fill the first `0x9e00` of
  the `0xf408` the startup zeroes there, and what the rest holds is unread.
- **What writes `0x1403a4b84`**, the rectangle the player is held to — see the section on the field.
- **Which of the bullet's five states is a bullet still coming out and which is one already spent.**
  Which one *hits* is established — state 1 and no other — and what the other four mean is not.
- **What scaled one bullet's step by √2 on one frame of one launch.** It moved `-2.25,5.13` while its
  step said `-1.60,3.66` — `1.41` on both axes, the same number twice, which is a scale and not a
  bullet turning; and since the step a dump prints was itself scaled by the half the model applied to
  state 5 at the time, what the game applied to the raw step was `0.707` against the model's `0.5`.
  What is ruled out: neither `1.41421` nor `0.70711` is a float of the pool at
  `0x1402c9470`..`0x1402c9590` or anywhere else in `.rdata`, as an `f32` or an `f64`; no ratio between
  the five states' own multipliers is `1.4`, `0x14000dabd`'s divisor chain loading `1.0` from
  `0x1402c94b8` against `2.0`, `2.5` and `3.0`. **And the state-5 half it was read alongside was a
  factor of two and is fixed** — see the table above — after which five launches in a row have every
  dump reading `0 bullet(s) no step of their own explained` and not one has met this again. So it is
  either a branch nothing has reached since or a reading of a frame the game was part-way through, and
  `steer::OutBy` names the state in the line beside what a bullet moved so that the next occurrence
  says which.
- **What `+0x7500` of the player is for.** It takes the same eight values `+0x7270` does and the switch
  at `0x14004b0e0` turns them into the rotation `+0x73a0` onwards is drawn with.
- **What its `.dat` files hold and how**, and whether `th06.cfg` and `th06.env` are anything like
  1.02h's configuration.
- **What a replay of it is**, and where a score is written. *Whether* one is being played back is
  established — see the section on the demo — the record its header is made of and the name in it are
  established too, and what the rest of the file holds is not, so neither is where the stage and the
  difficulty a playback shows come from. `score.dat` is written on every run that ends and the code that
  writes it is unread.
- **Whether the device read at `0x1403f8050`'s `+0x48` is used as well as `GetKeyboardState`**, and what
  the two flags at `0x1403f9fe0` and `0x140407424` turn on.
