# th06c, as read off its exe

What is written here was read off `th06c.exe` of **md5 `3d01e7d946556d3a28a738064a8efdc7`**, 4,353,536
bytes, which is 「東方紅魔郷: Classic」, the Steam release of 東方紅魔郷 — the build Steam installed on
2026-09-25, in place of the one of md5 `acadbc0cabf3bade16c22fb884957bcd` (4,039,168 bytes) this
document was first read off. *What the update changed* below is the difference between the two, and every
address here has been moved across to this build.

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
objdump -d --start-address=0x14009b960 --stop-address=0x14009b9e0 -M intel th06c.exe
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
| the exe | `PE32+ executable (GUI) x86-64`, seven sections |
| `.text` | `0x140001000`, `0x2b9694` bytes |
| `.rdata` | `0x1402bb000`, `0x687b8` |
| `.data` | `0x140324000`, `0xcb600` |
| `.pdata` | `0x140bc7000`, `0xd398` — the unwind tables a 64-bit build carries, which `.text`'s own functions can be listed out of |
| `.fptable` | `0x140bd5000`, `0x100` |
| `.rsrc` | `0x140bd6000`, `0x2aae8` |
| `.reloc` | `0x140c01000`, `0x1140` |
| what it was built with | MSVC with its CRT linked into the exe: nothing of `MSVCP140.dll`, `VCRUNTIME140.dll` or the `api-ms-win-crt-*.dll` is imported, and `malloc`, `free`, `fopen` and the rest are functions of `.text` — see *What the update changed* |
| what it is sold through | `steam_api64.dll`, beside the exe and imported outright |

**Its files are its own too**: `th06ST.dat`, `th06CM.dat`, `th06ED.dat`, `th06FN.dat`, `th06IN.dat`,
`th06MD.dat` and `th06TL.dat` beside the exe where th06 1.02h has `紅魔郷ST.dat` and the rest; `bgm/`
holding `th06_01.mid` and its neighbours where 1.02h streams `.wav`; `th06.cfg` and `th06.env` where
1.02h keeps `東方紅魔郷.cfg`; and `data/title/th06logo.dds`, a DDS where 1.02h has its own format.

## What it imports, and what it asks for by name instead

Four DLLs are imported: `steam_api64.dll`, `KERNEL32.dll`, `USER32.dll` and `WINMM.dll`. What `USER32.dll`
gives it is the whole of what it takes from the window manager at load time, and the list is short enough
to be worth having whole — its IAT begins at `0x1402bb378`:

`AdjustWindowRectEx`, `CallNextHookEx`, `CheckDlgButton`, `CheckRadioButton`,
`DialogBoxIndirectParamW`, `EndDialog`, `EnumDisplayMonitors`, `GetAsyncKeyState` (`0x1402bb3f0`),
`GetKeyboardLayout`, `GetSystemMetrics`, `GetWindowLongPtrA`, `GetWindowRect` (`0x1402bb3e8`),
`IsDlgButtonChecked`, `MapVirtualKeyExW`, `MessageBoxA`, `MonitorFromWindow`, `SetWindowLongPtrA`,
`SetWindowPos`.

**And what is not in it is the point**: no `CreateWindowEx`, no `RegisterClass`, no `PeekMessage`, no
`GetMessage`, no `DispatchMessage`, no `DefWindowProc`, no `ShowWindow` — and no Direct3D at all. Those
names are in `.rdata` as *strings* instead, which is what `GetProcAddress` is given: `CreateWindowExA`
and `CreateWindowExW`, `RegisterClassA`/`W`/`ExA`/`ExW`, `PeekMessageA`/`W`, `GetMessageA`/`W`,
`DispatchMessageA`/`W`, `DefWindowProcA`/`W`, `ShowWindow`, `D3D11CreateDevice`, `CreateDXGIFactory`,
`CreateDXGIFactory1`, `CreateDXGIFactory2`, `XInputGetState`. `KERNEL32.dll` gives it `LoadLibraryA`,
`LoadLibraryW` and `GetProcAddress` (`0x1402bb0a0`, called from 349 places).

So **a patch over this game's import table reaches almost nothing of it.** What th06 1.02h has orb walk
in — an import to overwrite, a fixed call to the device's own `Present` — is here a pointer the game
looked up and put somewhere of its own.

## Where it keeps what it looked up

**A table of function pointers in `.data`, filled once at startup.** The walk is one `LoadLibrary` per
module and then one `GetProcAddress` per name, unrolled — no loop and no table of names to walk, so every
answer has a slot of its own and every slot has one writer. Three import slots are the whole of the
machinery: `GetProcAddress` at `0x1402bb0a0`, `LoadLibraryW` at `0x1402bb0a8` and `LoadLibraryA` at
`0x1402bb0b0`.

Three hundred and forty-six names are resolved that way. To list them again — the module each came from,
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
        strings[int(at, 16) - 0x2b9c00 + 0x1402bb000] = text   # .rdata: file 0x2b9c00 loads at 0x1402bb000
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
        if found.group(2) in ("0x1402bb0a8", "0x1402bb0b0"):
            module, awaiting = pending.get("rcx", "?"), None
        elif found.group(2) == "0x1402bb0a0":
            awaiting = (module, pending.get("rdx", "?"), found.group(1))
    elif (found := STORE.match(line)) and awaiting:
        print(f"{awaiting[0]:<14} {awaiting[1]:<24} -> {found.group(1)}  (call at 0x{awaiting[2]})")
        awaiting = None
PY
```

**A pointer kept in a register is not in that table**, and the renderer's own is one: the module the
`XInput` pair comes from is walked for — `xinput1_4.dll`, then `xinput1_3.dll`, then `xinput9_1_0.dll`,
wide strings in a list on the stack — and its handle is kept at `0x140441f88`, where `D3D11CreateDevice`
is resolved inside the function that makes the device and left in `r15`.

### The slots orb has business with

| what | slot | where the answer is asked for |
| --- | --- | --- |
| `RegisterClassExW` | `0x140442788` | `0x140094a52` |
| `CreateWindowExW` | `0x140442588` | `0x140094419` |
| `ShowWindow` | `0x1404425a0` | `0x14009446a` |
| `AdjustWindowRectEx` | `0x140442608` | `0x1400945c9` |
| `SetWindowLongPtrA` | `0x140442628` | `0x140094635` |
| `SetWindowLongPtrW` | `0x140442630` | `0x140094650` |
| `PeekMessageW` | `0x1404426d8` | `0x140094800` |
| `GetMessageW` | `0x1404426e8` | `0x140094836` |
| `DispatchMessageW` | `0x140442748` | `0x14009497a` |
| `DefWindowProcW` | `0x140442720` | `0x1400948f3` |
| **`GetKeyboardState`** | `0x140442908` | `0x140094f62` |
| `XInputGetState` | `0x140441f90` | `0x14008b7b0` |
| `XInputSetState` | `0x140441f98` | `0x14008b7cb` |
| `CreateDXGIFactory2` | `0x1408971d8` | `0x1402522e9` |
| `CreateDXGIFactory1` | `0x1408971d8` | `0x14025238b` |
| `CreateDXGIFactory` | `0x1408971a0` | `0x14025242b` |
| `D3D11CreateDevice` | — kept in `r15` | `0x140251b60` |

**`GetKeyboardState` is the read a run is played by**, which is the one the whole 256-key snapshot comes
through; `GetAsyncKeyState` is imported outright and is not it — see below. Both of the XInput calls are
one site apiece, and the two factory slots are the same slot twice because the walk asks for the newest
name first and falls back.

**And the frame is handed over through a swapchain's own vtable**, which no name resolves: `Present` is a
method on what `CreateDXGIFactory*` and `D3D11CreateDevice` between them make, so where it is called from
is a reading of the function at `0x140251ab0` onwards and not a slot in the table above.

**What it draws with is a D3D11-class renderer**: `D3DCompiler_33.dll` through `D3DCompiler_47.dll` are
named in `.rdata`, which is the fallback walk an app that compiles shaders at runtime makes, and so are
`DXGI_FORMAT_BC1_UNORM`, `BC2`, `BC3`, `BC7_UNORM` and `BC7_UNORM_SRGB`. **What it plays sound with** is
`DSound.DLL` for the effects and `Mfplat.dll` with `Mfreadwrite.dll` — Media Foundation — for the music.

## The keyboard: one 256-byte state, filled two ways

`0x14008e180` onwards is where a frame's keys are read, and what it fills is **the 256 bytes at
`0x14042893c`**:

| | |
| --- | --- |
| `0x140441fd0` | an object whose vtable is called at `+0x48` with `(0x100, 0x14042893c)` — `0x14008e1c8`, again at `0x14008e1f6` — and at `+0x38` with nothing, `0x14008e1d9`. A device's own state read into that buffer, in other words, with an acquire between the two tries |
| `0x14008e22c` | `GetKeyboardState` into a 256-byte buffer on the stack, whose `[rsp+0xc1]` — `VK_RSHIFT` — is kept at `0x140428972` |
| `0x14008e285` | the same read again on the other path, after which `0x14042893c` is memset to nought (`0x1400bb280`, `(buffer, 0, 0x100)`) and filled out of that stack buffer through the table at `0x1402bc0b0`: 0x41 entries walked as words at `+0x2`, so a mapping from the game's own key numbers to virtual keys |
| `0x140428938` | a timestamp the walk writes — `0x14008e20a` and `0x14008e269`, held against `timeGetTime`'s answer (`0x1404421c0`) at `0x14008e25d`. What it holds to one per tick is the walk; the `GetKeyboardState` that reaches it is made 66 times a second against a game that moves 60, which is measured below |
| `0x140441fb8`, `0x140441fd8` | a handle each: the first decides whether the device path is taken at all (`0x14008e1a2`), the second is waited on through `WaitForSingleObject` (`0x1404422a8`) and reset through `ResetEvent` (`0x140442258`) |

**And the buffer is indexed by DirectInput's own scan codes.** The table at `0x1402bc0b0` is what the
`GetKeyboardState` path fills it through, walked at `0x14008e2c0`: six bytes an entry — a virtual key, the
index to write it at, and two bytes of nought — ending at a key of `0xffff`. `A` is `0x41 -> 0x1e`, `B` is
`0x42 -> 0x30`, `C` `0x43 -> 0x2e`, `D` `0x44 -> 0x20`, `E` `0x45 -> 0x12`, which is `DIK_A`, `DIK_B`,
`DIK_C`, `DIK_D`, `DIK_E`. So the other path needs no translation at all, which is what says what it is:
the object at `0x140441fd0` is a DirectInput keyboard, its `+0x48` is `GetDeviceState` with 256 bytes, and
`GetKeyboardState` is the fallback for a machine it could not be made on.

The five bytes the walk does not go through are the mouse's, read out of the same stack buffer by virtual
key and kept apart: `VK_LBUTTON` at `0x14042aa44`, `VK_RBUTTON` at `0x14042aa45`, `VK_MBUTTON` at
`0x14042aa46`, and `0x14042aa47` and `0x14042aa48` behind a build number at `0x140452438` against `0x108`.

**So a key orb wants the game to see is a `0x80` at `0x14042893c` plus that key's own scan code** — or a
virtual key in the 256 bytes `GetKeyboardState` was given, which the game then translates through that
table itself, and which is what orb does.

**Standing in front of the slot is the wrong way to reach that read.** The slot holds `0x0` when orb is
attached — measured, `0x7ff617c98988` read as nought — because the game's own startup is what writes it,
and a pointer put there before that is one the game overwrites. What orb stands in front of instead is the
**lookup**: `GetProcAddress` is imported at `0x1402bb0a0`, so answering the ask for `GetKeyboardState` with
orb's own read leaves the game writing orb's pointer into its own slot. That needs no address of the
game's at all, which is what makes it hold across a base the loader chose.

**And a key written into those 256 bytes is acted on**, which is the whole reason the read is the seam
worth having: a launch whose keys were a decide held in bursts walked the title menu into its difficulty
select, 「モードを選択してね」 with Easy, Normal and Hard on it.

**But the read is not the tick, and it is made more often than the game moves.** Counted directly,
once orb had a hook on the frame to count against: **683, 692, 736, 761 and 773 reads against every
600 walks of the calc chain**, which is the frame below. So between a seventh and a quarter of the
reads see the world the read before them saw, and the timestamp at `0x140428938` holds the *walk* of
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

`0x14009b992` is the only call of it in the exe, and the function it is in — `0x14009b960`, three
arguments — is a filter over a keystroke somebody else read:

| | |
| --- | --- |
| `0x14009b96f`, `0x14009b980` | two flags at `0x140443f60` and `0x1404513a4`, both against 1, and the function answers 0 where either is not set |
| `0x14009b98b` | and where its first argument is not nought |
| `0x14009b98d` | `0x11` — `VK_CONTROL` — into `ecx` and the call, whose answer is taken by its sign bit (`sar edx,0xf`) |
| `0x14009b9a0` onwards | its third argument's own `[rbx]` against `0x1b`, `0x09` and `0x73` — Escape, Tab and F4 — and the bits `0x20` and `0x80` of `[rbx+8]` |
| `0x14009b9c1` | 1 where one of those matched, 0 at `0x14009b9d6` where none did |

Which is a shortcut being swallowed — Control with Escape, Tab or F4 — and not the word a run is played
with. **So the keyboard a run acts on does not come through `GetAsyncKeyState` at all**, and the two
flags above are worth reading for what turns that swallowing on.

## The dialog it puts up before it starts anything

**Every launch, and whatever is beside the exe.** `DialogBoxIndirectParamW` is imported outright
(`0x1402bb3b0`), with `EndDialog`, `CheckRadioButton`, `CheckDlgButton` and `IsDlgButtonChecked` beside
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

### What it asks that DLL, and which slot of each interface

**Nine imports, at `0x1402bb478`..`0x1402bb4b8`** in this order: `SteamInternal_SteamAPI_Init`,
`SteamAPI_RunCallbacks`, `SteamInternal_FindOrCreateUserInterface`, `SteamAPI_RestartAppIfNecessary`,
`SteamAPI_RegisterCallback`, `SteamAPI_UnregisterCallback`, `SteamAPI_GetHSteamUser`, `SteamAPI_Shutdown`
and `SteamInternal_ContextInit`. `orb::steam` answers them by name, so the order is nothing it reads.

**Its startup is `0x140048940`**: `SteamAPI_RestartAppIfNecessary` at `0x14004894e` with the app id
`0x48ce48` loaded at `0x140048949` — and `jne` to the exit at `0x140048984` where it answers true — then
`SteamInternal_SteamAPI_Init(0x14030d110, 0)` at `0x140048961`, whose `test eax,eax; je` takes **nought**
as the good answer and anything else to the message above. Past those it takes `0x28` bytes for a callback
object, fills it with the vtable at `0x14030d068` and the handler at `0x140048910`, hands it to
`SteamAPI_RegisterCallback` at `0x1400489ec`, and keeps it at `0x140bc2a78`.

**Four interfaces, each through a getter of its own**, and each getter has its own context block in
`.data` whose first word is that getter:

| version string | getter | block |
| --- | --- | --- |
| `SteamUtils010` (`0x14030d0a8`) | `0x140048850` | `0x1403ef170` |
| `STEAMUSERSTATS_INTERFACE_VERSION013` (`0x14030d0b4`) | `0x140048880` | `0x1403ef158` |
| `STEAMAPPS_INTERFACE_VERSION009` (`0x14030d0e0`) | `0x1400488b0` | `0x1403ef128` |
| `SteamInput006` (`0x14030d100`) | `0x1400488e0` | `0x1403ef140` |

**And the game dereferences what it gets back without a check**, which is why an interface has to be
answered at all and not merely refused: `0x14002c2f9` onwards is `mov rcx,[rax]; mov rax,[rcx]; call
[rax+0x110]`, and with no client `SteamInternal_FindOrCreateUserInterface` hands back nothing for those
two instructions to walk.

**Seven slots of those four are called:**

| interface | slot | method | where, and what the answer does |
| --- | --- | --- | --- |
| Utils | `+0x110` | `IsSteamRunningOnSteamDeck` | `0x14002c2ff`; true sets `[0x140bc28d4]` at `0x14002c30d` |
| UserStats | `+0x48` | `StoreStats` | `0x14002c605`, in the frame loop behind a flag at `0x14002c5f0`; true clears `[0x1409552db]` |
| Apps | `+0x20` | `GetCurrentGameLanguage` | `0x140048a5e`; the string is compared against `0x14030ccd8` onwards |
| Input | `+0x00` | `Init` | `0x140048a49`, with `bExplicitlyCallRunFrame` false |
| Input | `+0x08` | `Shutdown` | `0x140048c39` |
| Input | `+0x30` | `GetConnectedControllers` | `0x140048cc1`; `jle` past the rest where it is nought |
| Input | `+0x128` | `GetInputTypeForHandle` | `0x140048ce0`, a jump table over 5..13 |

**Which slot is which method was read off the DLL beside the exe**, not guessed from a header: every flat
`SteamAPI_IStream*_*` export of `steam_api64.dll` is a wrapper whose body calls exactly one vtable slot,
so the export table and one disassembly name them together.

```sh
objdump -p steam_api64.dll                    # the ordinals, the names and the export RVAs
objdump -d -M intel steam_api64.dll           # each wrapper's own `call QWORD PTR [rax+N]`
```

Join the two by ordinal: the export address table's row for a name gives the wrapper's address, and the
first `call`/`jmp QWORD PTR [rax+N]` in it is that method's slot. `orb::steam` is what answers them, and
its `CALLED` is this table as indices.

## The field, and the bullets in it

**The field is 384 by 448 with its centre at (192, 224)**, which is th06 1.02h's own space, and the pair
is used as a centre outright: `0x14001e9e9` and `0x14001e9fe` load `192.0` and `224.0`, add `sin`/`cos`
of an angle times a radius at `[rbx+0x64]`, and hand the result to the bullet manager. So a coordinate of
this game's is read against those numbers and nothing has to be scaled.

**Its width and height are read out of `.data` and not out of an immediate**: `0x1403ed024` and
`0x1403ed028`, two floats the startup fills, and every comparison a coordinate is held against goes
through them — `0x14000e13d` and `0x14000e164` in the bullet manager, `0x140020957` and `0x14002097e`,
and `0x140049f09` onwards, which is where the player is put back. `0x1400079b6` and `0x1400079c5` turn
the same two into integers, so they are the size in pixels.

**And the rectangle it is drawn in is `0x14030e260`**, four floats — `32`, `16`, `416`, `464` — loaded
whole by one `movdqa` at `0x14000a194`, `0x140055c47` and `0x140056ae0`; `0x14030e278` holds `640` and
`480` beside it. So the field sits at (32, 16) of a 640 by 480 screen and a coordinate of the game's is
the field's own, with nothing added.

**The floats either side of those are one sorted pool and not a table of the field's.** `.rdata` from
`0x14030dfa8` to `0x14030e220` is every float literal the build has, in ascending order —
`0.1745…`, `0.2`, `0.25`, … `8`, `9`, `10`, … `448`, `464`, … `1e8`, then the negatives — so an address
in it names one constant and says nothing about what the constant is for. What a value's address *is*
good for is the grep that finds the code loading it, which is how the two paragraphs above were read.

**The bullet manager is a static at `0x140955410`**, `0xe8810` bytes of it, zeroed at
`0x140024edd`–`0x140024eef` by the startup that registers every manager. That startup also puts its
update at `[0x1403ec538]`, its draw at `[0x1403ec540]` and the manager itself at `[0x1403ec568]`, which
is the callback chain this game runs a frame through.

Its update is `0x14000db90`, and the array is read off that function's own loop:

| | |
| --- | --- |
| where the array starts | `0x140955410 + 8` — `lea rbx,[rcx+8]` at `0x14000dba4` |
| how far apart | `0x5d0` — `add rbx,0x5d0` at `0x14000e4da` |
| how many | `0x280`, six hundred and forty — `cmp ebp,0x280; jl` at `0x14000e4fb` |
| how many are live | `[0x140955410 + 0xe8808]`, counted up per live one at `0x14000dccb` |
| one bullet's step | `+0x08`, three floats — `rsi = rbx+8`, read at `0x14000dcf9` onwards |
| one bullet's place | `+0x30`, three floats — `rdi = rbx+0x30`, written at `0x14000dd32` |
| **which of five states it is in** | `+0x44`, a word — `cmp WORD PTR [rbx+0x44], 0` at `0x14000dcc0`, and nought is a slot nothing is in; `0x14000dcd2` jumps on it |
| the size the player's own test is given | `+0x5a4`, two floats, handed over at `0x14000e47e` beside the place — and that test halves what it is given |
| whether it is armed | `+0x5c8`, a byte: nought runs `0x14004c3e0` and sets it to 1, and only then does `0x14000e47e` run the player's own test |
| the frame it is on | `+0x28` and `+0x2c`, the second counting up and the first keeping what it was |

**The state decides both whether it can hit and what its step is scaled by**, and there is no one
motion scale on this manager at all:

| state | where it is moved | what its step is multiplied by | tested against the player |
| --- | --- | --- | --- |
| 1 | `0x14000e372` | nothing: `[+0x30] += [+0x8]` | **yes** — this is the only path that reaches `0x14000e3fb` and the two calls of `0x14004c690` |
| 2 | `0x14000dd57` | `1.0 / 2.0` | no |
| 3 | `0x14000dd6d` | `1.0 / 2.5` | no |
| 4 | `0x14000dd7e` | `1.0 / 3.0` | no |
| 5 | `0x14000dcf9` | nothing either: `[+0x30] += [+0x8]` | no |

**The jump at `0x14000dcd2` is a chain of subtractions and not a table.** `0x14000dcd6` onwards takes
`r15d` — one — off the state four times over, sending 1 to `0x14000de04`, 2, 3 and 4 to the divisor
chain at `0x14000dd52`, and letting 5 fall through to `0x14000dcf9`, which is where the step is added
whole.

**State 5 was read as a half, and a run contradicted it.** `0x14000dd15`'s `mulss xmm1,xmm13` is in
the code that fall-through reaches and `xmm13` is `0x14030dfc8`'s `0.5`, so the half looked like the
answer — but what that block scales is `[rsi]`..`[rsi+8]` into `[rdi]`..`[rdi+8]` off the `rbx+0x490`
the `lea` before it takes, and **not** the bullet's own `+0x8` into its `+0x30`. A launch said so
outright: `the widest of them moved 3.30,5.38 saying 1.65,2.69, in the game's own state 5` — twice
what a half predicts, on both axes.

**A flat half over all five is what one launch cost.** Read that way, the rules judged ten of ten
bullets as moved by something other than their own step, so each was feared over the box it had just
swept and none of them was projected: the model could dodge only what was already on the player, and
that run was on its last life at stage 3's boss.

**A state-1 bullet's step is recomputed every frame** where its own flags at `+0x14` say so —
`0x14000de4e` and `0x14000e008` write `cos(angle) × speed` into `+0x8` off the angle at `+0x40` — so a
bullet that turns is one no straight projection describes, whatever the scale.

**And the whole update is skipped where the byte at `0x1403ed7d2` is not nought** — `0x14000db9d`, which
answers 1 and returns. Six other functions read that byte, so it is the game's own "everything is
held" flag rather than this manager's.

`0x14000cbc0` is what fires one, `__thiscall` on the manager with a descriptor on the stack.

## The lasers, in that same static

**`0x140a3dc28` onwards, and the same update walks them**: `0x14000e50d` puts the array at `+0xe8818`
of the bullet manager — which is that address — and the loop after the bullets' own runs sixty-four of
them, `0x278` apart (`0x14000e690`, `0x14000e697`). A slot whose `+0x254` is nought holds nothing
(`0x14000e570`), and `0x14000e66e` clears it once the near end has run off the far edge. The startup
zeroes the region at `0x140024ef6` and `0x14000d920` is what walks it for a free slot.

**What one is is a line and not a box**, which the player's own test for it says: `0x14004c890` turns
the player's offset from the laser's origin by its angle — `sin` at `0x1402b6c80` and `cos` at
`0x1402b6390` — and compares the result with a box that is *along* and *across*.

**And that test halves the size it is given, the same way the bullets' does**: `xmm6` is
`[0x14030dfc8]` at `0x14004c917` and `0x14004c95a` and `0x14004c96c` multiply both of the size's floats
by it. The width at `+0x4` is halved once already on the way in, at `0x14000e5dc` — so **a laser reaches
a quarter of its width to either side of its own line**, and reading that as a half is a lane twice as
wide as the game's. What that cost is two hits of an Extra run, each with `nowhere open to it` and
`-5.1 of room` on every one of the nine moves, the nearest laser `8.2` off a player whose lane was
modelled `12.0` wide: the gaps of a fan of lasers all read as shut.

**What the test compares, register by register**, which is what says which of those halvings is the
width's and which axis each float of the pair is on. Its arguments are the box centre in the beam's own
frame (`rdx`, `[rsp+0x40]` at the call site), the size pair (`r8`, `[rsp+0x30]`), the origin (`r9`,
`+0x258`) and the angle on the stack:

- `0x14004c8f9` and `0x14004c902` take the player's own position, `[0x140bbfc38]` and `[0x140bbfc3c]`,
  less the origin
- `0x1402b6c80` and `0x1402b6390` are `sin` and `cos` of the angle, and `0x14004c940` and `0x14004c967`
  turn that offset into `dx·cos + dy·sin` **along** the beam and `dy·cos − dx·sin` **across** it
- `0x14004c95a` halves the pair's **first** float against the along axis and `0x14004c96c` its
  **second** against the across axis, so `[rsp+0x30]` is the length and `[rsp+0x34]` the width
- `0x14004c94d` and `0x14004c95e` are the player's own half, `[0x140bbfc54]` and `[0x140bbfc58]`, read
  straight and not halved
- the four `comiss`/`ja` at `0x14004c997` onwards are one box overlap: the player's box about the turned
  offset against the beam's about `rdx`
- and **the origin goes into both sides of that comparison and cancels**: `0x14004c955` and
  `0x14004c97d` add `+0x258` and `+0x25c` to the turned offset, and `0x14000e601` and `0x14000e60f`
  put the same two into `[rsp+0x40]` and `[rsp+0x44]` beside the along midpoint — so what the test
  compares is the beam's own frame after all, `[near, far]` along it and the width across

So the live path at `0x14000e5c2` hands over `(far − near, width × 0.5)` and the test halves both: the
length is the two ends as they are read, and the width reaches a quarter of itself to each side.

**And a fading one narrows by `frames / +0x18` of its width**: `0x14000e8bd` onwards reads `+0x4` and
`+0xc`, works out `width × (1 − frames ÷ +0x18)`, halves it with the same `[0x14030dfc8]` and writes it
to `[rsp+0x30]` — the length's slot rather than the width's, which is not what a narrowing beam would
want and is written down here as what the exe does. `+0x108` beside it gets the same number times
`[0x14030df9c]`, which is `0.0625`.

| | |
| --- | --- |
| `+0x4` | **how wide it is**, and the test is given half — `0x14000e5dc` |
| `+0x8`, `+0xc` | the frames it has been in its phase, the pair a `Timer` is |
| `+0x10`, `+0x14` | **how far along it runs**, near end and far |
| `+0x18` | **the frames it goes out over**, which the fraction above is of — and nought here skips the whole of that block (`0x14000e8bb`) |
| `+0x240` | **which way it points** |
| `+0x24c` | **how much further the far end reaches each frame**, added straight on with no scale (`0x14000e57d`); the near end follows once the two are `+0x268` apart, and stops at nought |
| `+0x244` | a flag the warning and the fading branches both test — `0x14000e71b` and `0x14000e87e` — and take another way where it is set |
| `+0x248` | **how many frames it fires for**: `0x14000e84c` compares `+0xc` with it and moves the phase on |
| `+0x250` | **how many frames its warning runs for**: `0x14000e7e7` does the same for phase nought |
| `+0x254` | whether the slot is in use |
| `+0x258` | **where it is fired from**, which is the pivot the test turns the player about |
| `+0x268` | how long it is once grown |
| `+0x26c` | **the frame of its first phase it starts hitting on**: `0x14000e7a1` runs the test only from there, so one still warning is drawn and hits nothing |
| `+0x270` | its phase: nought is that warning, 1 fires every frame (`0x14000e847`), 2 fires and narrows (`0x14000e93d`), and past that `0x14000e664` turns it off |

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

**And this is how a sampled address is read back.** `orb-core`'s sampler writes where the frame's thread
was as the module and the offset into it — `th06c.exe+0xa98e9` — so the address in this document is
`0x140000000` plus that offset, and the listing above says which function it is inside. An offset in
another module is read the same way against that module's exports:

```sh
objdump -p /mnt/c/Windows/System32/ntdll.dll   # the export table, to find the name below an offset
```

**Adjacent entries are often one function and not several.** `0x140049ea0`–`0x14004b398` comes back as
nine entries whose ranges touch, and the code runs straight through every boundary: the extra entries are
chained unwind info for a function whose prologue is too long for one. So a listing says where a function
*can* begin and the instruction at that address says whether it does — a `mov [rsp+0xd8],r13` with `rdi`
already holding an object is inside one.

## The callback chains, and every job registered in them

**Two lists, and a frame is a walk of each**: the calc chain's head is the static `ChainElem` at
`0x1403ec5f0` and the draw chain's is `0x1403ec5b0`. Both are sentinels rather than pointers — an
insertion compares against `[head]` and links through `[head+0x28]`, so an empty chain is
`[0x1403ec618]` (and `[0x1403ec5d8]`) being nought, which is the comparison twenty-five places in the
exe make before they walk.

`ChainElem` is `0x40` bytes, and every registration in the exe writes the same shape — read off
`0x140049c06`–`0x140049d2c`, which builds three of them, and confirmed against the inlined one at
`0x140024f1e` onwards:

| | |
| --- | --- |
| `+0x0` | `short priority`, and the list is kept in ascending order of it |
| `+0x2` | a bit set by `or BYTE PTR [rax+0x2],0x1` on every heap-allocated one — th06 1.02h's `isHeapAllocated` |
| `+0x8` | **the callback a frame runs**, called with `+0x38` in `rcx` |
| `+0x10` | the one called **once, as it is registered**, and nulled straight after — `mov rax,[rbx+0x10]; mov rcx,[rbx+0x38]; call rax` at `0x140049d8b`, then `mov [rbx+0x10],rdi` |
| `+0x18` | the one called as it is taken out |
| `+0x20`, `+0x28` | the links, and `+0x28` is the one a walk follows |
| `+0x30` | the element's own address, written by whoever built it |
| `+0x38` | what the three callbacks are called on |

**And the priority is what settles the order of a frame**, which three of them are read off their own
registrations: the input job at `0x140058330` writes nought (`xor edi,edi` at `0x14005884f`, `mov [rbx],di`
at `0x1400588d1`), the player writes 7 (`mov esi,0x7` at `0x140049cb1`, `mov [rbx],si` at `0x140049d3e`)
and the bullets write 11 (`mov r13d,0xb` at `0x140024f4f`, `mov [0x1403ec530],r13w` at `0x140024f5f`).
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
| `0x140049d30` | `0x140bb89f0` — **the player** | `0x14004a4b0` | `0x140049ea0` | `0x14004a480` |
| `0x140024f3a` | `0x140955410` — **the bullets** | `0x14000db90` | `0x14000f010` | `0x14000f2c0` |
| `0x14000f3e0` | `0x140b7d260` | | | |
| `0x14001787b` | `0x140a7e030`, `0x140a4d030` | `0x140017630` | `0x140017720` | `0x140017800` |
| `0x140017fc1` | | `0x140018110` — **the ending** | `0x140018c40` | `0x140018ec0` |
| `0x14002052c` | `0x140a7e1e0` | `0x140020780` | `0x140021890` | `0x140021980` |
| `0x140036ee5` | `0x140ba73c0` | `0x14002ccf0` | `0x140037050` | `0x140037970` |
| `0x140039b55` | `0x140bc60a8`, `0x140bc2cf0` | `0x140039ca0` | `0x14003a200` | `0x14003a980` |
| `0x14004d260` | | `0x14004d800` | `0x14004d940` | `0x14004dc00` |
| `0x14004fdf3` | | `0x14004ff40` | `0x1400544a0` | `0x140054ab0` |
| `0x140057334` | `0x1403eef30` | `0x140056160` | `0x140056e90` | `0x140057550` |
| `0x1400588c6` | `0x140bc2ae0`, `0x140bc2580` | `0x140058330` | `0x140058a00` | `0x140058e40` |

**And which job an ending is was read off a launch and not off the exe**, which is the way to anything of
this game that neither moves nor is pointed at. `th06ED.dat` is the file one reads, and its string at
`0x14030bf08` is named once in `.text`, by `lea r8` at `0x140018c62` — which is inside the ending's own
code and says what the ending loads, not which callback of the chain is the ending. So the chain itself
is the instrument:
`game::th06c::say_the_jobs_when_they_change` walks it every frame and writes every callback down on the
frame the set of them changes, and one launch of a Normal clear then named the job outright:

```text
f98220   chain: 10 job(s): … 0x14004a4b0 … 0x14000db90 0x140027180 0x14004d800
f98220   chain: 4 job(s): 0x140058330 0x1400077f0 0x140018110 0x14004d800
f106680  chain: 4 job(s): 0x140058330 0x1400077f0 0x14004ff40 0x14004d800
```

**Which is `8,460` frames of it, in the scene the stage it followed had.** The frame lines across the
whole of that read `scene=11 stage=5 enemies=0 bullets=0 boss=false`, so the game manager's own state says
a stage is still running and the ending is a job drawn over one with nothing left in it — the same shape
the screen a run ends at has, and the reason `Game::read_state` answers `State::in_ending` by asking
whether the chain holds this callback. The screen a run ends at is registered on the frame after this job
leaves, which is what `0x14004ff40` appearing at `f106680` is.

**And `0x14030e260`'s own statics are past the end of `.data`'s raw bytes.** `objdump -h` says `.data` is
`0xca600` long, which is what is *in the file*; the managers above sit from `0x140955410` to
`0x140bc60a8`, so the section's virtual size runs to `.pdata`'s `0x140bc7000` and everything between is
zero-filled at load. A read of the game's state that only looks at `.data`'s raw extent finds none of
them — measured: a search for the player's position over `0x140324000`–`0x1403ef590` and the private
heap came back with heap addresses and no static one.

## The frame, and the loop that runs it

**The loop is `0x14002c5b7`..`0x14002c639`**, four calls and a jump back: something of the window's
(`0x140099a70`, handed two pointers into the frame's own stack), `0x14009ca50`, Steam's
`SteamAPI_RunCallbacks` through `[0x1402bb480]`, and then **the frame at `0x1400258a0`**, which it leaves on a non-zero answer from
that or from `0x1400a14e0`. Nothing in the loop waits: what paces a run is inside the frame.

**And the frame is this, in order:**

| | |
| --- | --- |
| `0x1400258bb` | the view: nought for the corner at `0x140bc28fc` and `0x280` by `0x1e0` — 640 by 480 — at `0x140bc2904` and `0x140bc2908`, and then **`0x1400591d0`**, which flushes the sprites the renderer's batch holds (below), hands `0x140075940` the corner, the corner plus the size and the two floats at `0x140bc290c` and `0x140bc2910`, and puts the matrix in place: `0x1400d05a0`, whose answer is a `0x40`-byte matrix copied to its stack, and `0x1400778a0` after it |
| `0x1400258e0` | **the calc chain walked, unconditionally** — `0x1400256b0` |
| `0x140025900` | and walked **again up to seven more times**, `cmp ebx,0x7`, while `[0x1403ed7b8]` is set, `[0x1403ed7d1]` and `[0x1403ed7cc]` are clear, `[0x140bc28ac]` is 2 and `[0x140955390]` has `0x8000` |
| `0x140025992` | how many of those to run, out of the sound: `0x1400a4700` and `0x1400a4600` answer a position in samples, `0xac44` — 44,100 — stands in where the first answers nothing, and `0x1400a9660` is handed the result |
| `0x140025a2d` | **the draw chain walked, inlined** — `lea rdi,[0x1403ec5b0]` and the same walk as below, in the frame's own code and not in a function of its own, with the renderer's batch of sprites flushed after every job |
| `0x140025abb` | `0x140025af0`, which is the frame's own timing: it calls a clock through the pointer at `0x140bc2a80`, averages over `0x5f` of them and divides by a hundred |

**And that timing is a frame limiter and not only a measurement.** `0x140025c00` onwards works out how far
off its own deadline the frame is, turns that into a wait with the frequency at `0x140bc2a98`, and calls
the pointer at `0x140bc2a90` with it — then reads the clock again and waits some more while anything is
left, and finally `0x140025c54` spins on `pause` until the deadline itself. So a frame of this game holds
sixty a second by sleeping and then spinning, and orb's own loop does not call it at all: measured, it was
`14.7ms` of a `16.6ms` frame, and a launch at `--pace=2` ran at sixty a second like every other until it
was left out. What goes with it is the `60.00fps` the game draws in its own corner, which the same
function works out and formats (`0x1400219b0` at `0x140025d23`).

**So a run's rate is the sound's**, not the display's and not a sleep: the frame runs as often as the
loop gets round to it and the calc chain is walked as many times as the samples played say it should
be. Which is what a launch at `--pace=2` measured — 66.0 words a second asked of the rules became
136, and stage 1 took 79 seconds against 169 — the clocks orb answers being what the sound's own
position is read against.

### The chain walk

**`0x1400256b0` is the calc chain's**, and the draw chain's is the same walk inlined at
`0x140025a2d`. Both start at the head's `+0x8`, call it with `+0x38` in `rcx`, and follow `+0x28`;
what the callback answers decides what the walk does next:

| | |
| --- | --- |
| 2 | call it again, without moving on — `je` straight back to the call |
| 0 | take the element out: `0x14000f620` with the element in `rdx` and the walk already moved on |
| 1 | start the walk over from the head (calc only) |
| 3, 4, 5 | end the walk, each with its own answer for the frame |

**And the draw walk draws what each job queued before it acts on the answer**, anything but the 2 that
calls it again. Every draw of this build queues its sprites into a batch at the end of the renderer's own
block — `0x32ba8` bytes where the build before had `0x22c40`, `mov ecx,0x32ba8` at `0x14002c571` — and
nothing reaches the screen until the batch is flushed: `0x140025a51`..`0x140025a88` read the renderer out
of `0x1409552d0`, and where its count at `+0x32b9c` is above nought call `0x140062330(renderer + 0x22c3c,
count, 4, [renderer + 0x32ba0], 1)` and zero the count. 136 sites touch that count, the batch being how
the whole of the drawing now goes; `0x1400591d0` flushes it too, before it moves the viewport, and
`0x140059160` is a second wrapper round the same flush.

**And the element's own `+0x28` is read after its callback has returned**, `0x140025709` against the
call at `0x1400256e8`, so a job registered by the job being run — by a scene building itself, which is
what a transition is — is reached in that same walk. **Which is why a scene of this game gets its first
update on the frame it was built on**, and why `State::stage_frames` reads one rather than nought on the
first frame orb sees a stage.

**And the calc walk is where the frame's queued sound effects go**: `0x140025734` onwards walks the
three dwords at `0x1403eea40`, each an index into `0x1403eec64`, and hands every one that is not
`-1` to `0x1400a7e60` as `(index, 1, 1)` — the same job th06 1.02h does in
`SoundPlayer::PlaySounds` after its own update.

**What that leaves for orb is an update to stand in and no draw to stand in.** The calc chain has a
function of its own, so one call per walk is a seam a patch can take; the draw chain is inlined in
the frame, so the only seams either side of it are the frame itself and the jobs registered in it.

### The shader compiler this exe names and does not load

`.rdata` holds `D3DCompiler_47` down to `D3DCompiler_33`, fifteen names walked newest first:

```sh
strings -a th06c.exe | grep -i 'd3dcompiler'
```

**And by the frame orb's overlay is built on, none of them is in the process.** Measured over a
real-game session, every launch: `canvas: the vertex shader would not compile: none of 15 is loaded with
a D3DCompile in it`. So the names say what the renderer would reach for and not what it has reached for
— its own shaders are compiled somewhere later than this, or not at runtime at all. What orb does about
it is bring one in itself; `orb_api::module::brings_in` is that call, and the walk still asks the game's
own first so that orb compiles through the compiler the game's shaders went through where there is one.

## The sound library's own block, and why a chapter must not put it back

**The effect play at `0x1400a7e60` takes a lock before it queues anything.** `0x1400a800d` loads
`0x140497f50` into `rcx` and `0x14030cae4` — the name the wrapper keeps — into `rdx`, and calls
`0x1402444e0`; `0x1400a8045` hands the same lock to `0x140244590` on the way out. Those two wrappers are
`EnterCriticalSection` and `LeaveCriticalSection` with the library's own bookkeeping around them, and
`0x140244460` is `InitializeCriticalSection` with the same. Each is reached through a slot the startup
fills by name — `0x140442448`, `0x140442458`, `0x140442460` — and each wrapper's object is `0x230` bytes:
the `CRITICAL_SECTION` itself, the lock's name copied in at `+0x28`, and the id of the thread that took
it at `+0x22c`.

**Five of them are initialised together**, at `0x1400a5126` through `0x1400a515d`: `0x140497a80`,
`0x1404984b0`, `0x140497ce0`, `0x140497f50` and `0x1404981c0`. **And each guards a pool that sits at its
own end**: `0x1400a8036` hands `0x140497f10` to `0x1400e22a0` while holding `0x140497f50`, and
`0x1400ab7b9` hands `0x140497cb0` to `0x1400e22d0` while holding `0x140497ce0` — both of those being the
lock before it plus `0x230`.

**The block's bounds are where the references stop.** Every address anything in the exe reaches in
`0x140496f80`–`0x140498f80` lies between `0x1404979b4` and `0x1404987a8`, and every instruction that
reaches one is at `0x1400a14xx` or past it — the sound library's own code, and the runtime helpers at
`0x14024xxxx`. Not one of the game's own functions touches a byte of it:

```sh
objdump -d -M intel th06c.exe | grep -oE '^ *1[0-9a-f]+:.*# 0x14044[de][0-9a-f]+'
```

**Which is why orb holds the whole of it back from a restore.** A chapter put back over a lock says a
thread holds it that does not, and over a pool says a worker has work to do that it has already done —
and neither the threads nor what they are doing is anything a snapshot rewinds. Measured on the real
game: 909 of the 917 samples a_death_answered_by_the_chapter_again gave up were
`ntdll!ZwWaitForAlertByThreadId+0x14` under `RtlEnterCriticalSection+0xf2`, under `0x140244536`, under
`0x1400a8019`, under `0x140025783` — the frame blocked on `0x140497f50` for 35ms to 137ms every time a
shot hit an enemy, and only after a chapter had been put back.

## The renderer's table of windows, and the views it makes again every frame

**The renderer keeps its windows in a table of 256 entries of `0x118` at `0x1407e3898`**, with the count
open at `0x1407e3890` — `0x1402348fd` compares it with `0x100` before a window is made and `0x140234971`
counts it up — and the index of the one being drawn to at `0x1407f5098`, where the table ends. The code
indexes it off `0x1407e3750`, so the fields read there as `+0x150` onwards; off the entry they are:

| field | what it holds | read or written at |
| --- | --- | --- |
| `+0x0` | whether the entry is in use | `0x140235605` |
| `+0x8` | a window handle, asked `IsWindow` through `0x140442598` | `0x14023b700` |
| `+0x18` | the swapchain | `0x140235509`, `0x140235446` |
| `+0x50` | the window handle whose client is asked | `0x14023b741` |
| `+0xd0` | the back buffer, out of `GetBuffer` | `0x14023551c` |
| `+0xd8` | that buffer's width and height, out of its own description | `0x14023555b` |
| `+0x108` | the render target view made of it | `0x140235573` |
| `+0x110` | a second view made of it | `0x140235598` |

**Every frame, the window's client is held against the buffer's size.** `0x14023b630` — reached from the
picture's finish, `0x140072c85` calling `0x14021d0a0`, which calls it at `0x14021d126` — asks
`GetClientRect` through `0x140442600` into `rbp-0x80` (both slots filled by name at `0x140094463` and
`0x1400945c2`), and where the width or height differs from `+0xd8`
it calls `0x1402353b0` with the client's. That releases the two views (`0x140235310`, through
`0x140252ab0`, which counts `0x140897220` down and tails into the object's `Release`), calls
`ResizeBuffers` through `0x140252630`, and `0x1402354d0` takes the buffer again, reads its size into
`+0xd8` and makes both views again.

**So a window left a size other than its buffers' has its views made again on every frame**, which is
what a launch that plays itself is: orb holds the client at `320x240` and leaves the buffers the
`640x480` the game asked for — see `crate::dxgi`. And a chapter put back over the table names views
released since: the real game's first frame after a retry faulted at `0x140252ab9`, `jmp [rax+0x10]` off
a vtable of nought, which is why `Th06c::live_handles` holds the whole table back.

## The player

**A static object at `0x140bb89f0`**, `0x9b88` bytes of it — `0x140049bb0` clears it in two pieces,
`0x7374` from the front at `0x140049bd1` and `0x2814` from `0x140bbfd6c` at `0x140049be5`, which is that
address plus that size. The same function builds the three `ChainElem`s of the table above and hands each
of them `0x140bb89f0`, at priority 7 in the calc chain and 5 in the draw chain.

| | |
| --- | --- |
| `+0x7248`, `+0x724c`, `+0x7250` | **where the player is**, three floats, and the third is a depth rather than a coordinate — `0x140049f53` writes `0.49` into it and nothing else does |
| `+0x7228`, `+0x722c` | what a frame's movement is scaled by on each axis before it reaches the position — `mulss` at `0x14004afd4` and `0x14004afe4` |
| `+0x7500` | **which way it is being moved**, one signed byte, and nought is standing still: `0x14004b1a0` reads it, `0x14004b1b3` returns where it is nought, and what follows subtracts one at a time down a chain of comparisons |
| `+0x410`, `+0x414` and `+0x41c`, `+0x420` | **the two corners the game's own hit test compares**, `0x14004c752`–`0x14004c788`, and `0x14004b087` onwards writes both from the position every frame |
| `+0x7264` | **the half those corners are the position plus and minus**: `0x14004b060` hands it to the subtract at `0x14000fef0` and the add at `0x140009580`, and `0x14004c94d` reads it straight as the player's half in the laser's own test |
| `+0x9830` | the half of a second box, written the same way into `+0x9b74` and `+0x9b80`, and what that box is for is unread |
| `+0x7274` onwards | **sixteen boxes of `0x10` bytes** — width, height, and a centre — which the hit test at `0x14004c6f6` walks before it reaches the player's own: a hazard inside one of them is answered 1 rather than 2, and `0x14004a59b` onwards clears all sixteen every frame |
| `+0x73b8` | **which of four states it is in**, one byte: nought is the only one a hit lands in (`0x14004c78e` gates on it), 2 is what the hit writes (`0x14004c7f4`), 3 is the invulnerability `0x14004ac4e` counts down and 1 is what `0x140049ea0` and `0x14004a9d8` leave behind |
| `+0x7374`, `+0x7378` | that countdown, the pair a `Timer` is — the frame `+0x7378` reaches nought is the frame state 3 becomes nought |
| `+0x7254` | **the frames a bomb still takes the hit back in**, taken one off a frame at `0x14004a7ae`; `0x14004a655` is the bomb refusing to fire where it is nought, and the frame it reaches nought is the frame the death is made final — `0x14004a7bd` onwards reads the lives |
| `+0x75cc` | frames of invulnerability, which `0x14000a190` holds against 60 to pick the alpha it draws the field's flash at, and `0x14004a6cb` sets |
| `+0x438` onwards | its own ANM vms, `0x160` apart — `0x14004a19a` onwards writes the same word into eighteen of them |

**The position is clamped to a rectangle in `.data`**, and this is the one measurement that says the pair
above is the position and not a copy of it: `0x14004b006`–`0x14004b05f` puts `+0x7248` back to
`[0x1403ee7f4]` where it is under it and to `[0x1403ee7f4] + [0x1403ee7fc]` where it is over, and does the
same to `+0x724c` with `[0x1403ee7f8]` and `[0x1403ee800]`. Left, top, width, height, in the field's own
coordinates.

**And `0x140049ea0` is where a life starts**, which says where in the field that is: `+0x7248` is
`[0x1403ed024] * 0.5` and `+0x724c` is `[0x1403ed028] - 64.0` — the field's width halved, and 64 up from
its bottom.

### What a stage's own build leaves the player as, and the 270 frames after it

**`0x140049ea0` is the player's register and it runs at every stage's build**, not only at a run's start:
its own `cmp DWORD PTR [0x140bc28ac],0x3` at `0x140049ec1` is it telling a stage move from a run's start by
the supervisor's state, and what it reads to fill the four speeds is the character at `0x1403ed02c` and the
shot at `0x1403ed02d`, out of the table at `0x1403e0090`.

**What it leaves behind is three writes**: the state `1` at `0x14004a132`, the count `+0x7378` at **120**
(`0x14004a14c`), and the timer `+0x7374` at **-999** (`0x14004a156`).

**The walk-in that follows lasts thirty frames.** `0x14004ab7a`'s `cmp r8d,0x1e` is the gate, and past it
`0x14004ab80` onwards writes the state `3`, the count **`0xf0`**, the timer -999, a scale of `1.0` on both
axes and **six** frames of deathbomb window at `+0x7254`. State 3 then counts down —
`0x14004ac4e`–`0x14004ac69`, which puts the state back to nought and the count to nothing where it runs out,
and flashes the player (`+0x74cc` to `0xff404040`) every few frames while it does not.

**So a stage of this game begins with 30 + 240 = 270 frames in which nothing can reach the player**, the
hit at `0x14004c78e` being gated on that byte reading nought. The death's own respawn is the same walk with
another count under it: `0x14004a9d8` writes the state `1` and `0x14004aa1c` leaves the count at nothing,
where the register leaves 120 — and no state but 3 ever reads that field.

### What one frame of a direction moves it by

**Four speeds on the player**, and the switch at `0x14004ad7f` — a jump table of eight at `0x14004b91c`,
indexed by the direction below less one — picks the pair, while the focus bit picks which of the pair:

| | |
| --- | --- |
| `+0x7380` | straight |
| `+0x7384` | straight, focused |
| `+0x7388` | diagonal |
| `+0x738c` | diagonal, focused |

**And the diagonal pair is the straight one divided by the root of two**, worked out once at
`0x14004a0f4`–`0x14004a16c` rather than held as its own number: `movss xmm1,[rbx+0x7380]`,
`divss xmm1,xmm0` where `xmm0` is what `0x1402b6fd0` answered, into `+0x7388`, and the focused pair the
same.

**What reaches the position is that speed times the player's own scale for the axis** — `+0x7228` across
and `+0x722c` down, `mulss` at `0x14004afd4` and `0x14004afe4` — and nothing else: there is no motion
scale in the player's path, where the bullet manager's has one.

**The frame's own velocity is kept on the player too**, `+0x73bc` across and `+0x73c0` down, written at
`0x14004afc9` and `0x14004afdc` from the pair the position was moved by.

### The word a frame is played with

**`0x140955390` is the word the run acts on**, and `0x140955394` is the frame before it — the pair an
edge is taken from, held against each other at `0x14004a683` where a bomb is pressed. `0x14002cd69`
clears both, and `0x140955398` and `0x1409553ac` with them, in the **front end's own state nought** —
that state's case in the table at `0x140036414`, which goes on to load `th06TL.dat` and
`data/title01.anm` through `0x14002cdb8` onwards. What those last two words are is unread.

**And that is the only clear of the pair in the exe.** Three places write either word, which a grep for
`90a110` and `90a114` over the whole disassembly answers: this one, the input job's own copy and read at
`0x1400583b8` and `0x1400583cf`, and the replay playback at `0x14004d8ff`. So this game zeroes the pair
where its title's data is loaded and **at no transition of its own** — where th06 1.02h zeroes it at the
end of every one of its supervisor's, so that a scene's first update is acted on with nothing held.

Its bits are th06 1.02h's own, read off what tests them:

| | |
| --- | --- |
| `0x01` | shot |
| `0x02` | bomb — `and edx,0x2` at `0x14004a67a`, and what follows takes one off the bombs in stock |
| `0x04` | focus — `and r9d,0x4; setne al` into `+0x737c` at `0x14004ad5d` |
| `0x08` | tested at `0x14002844d` and `0x140028695`, and what it is is unread |
| `0x10`, `0x20`, `0x40`, `0x80` | up, down, left, right — `0x14004ad29` onwards |

**The eight directions are `+0x7270`**, written by that same walk and nought where nothing is held. Which
number is which way is read off the jump table at `0x14004b91c`, which the walk enters at that number less
one and whose eight cases each pick a sign for each axis: **1 up, 2 down, 3 left, 4 right, 5 up and left,
6 up and right, 7 down and left, 8 down and right**.

**And two directions of one axis held at once is not a stand-still: the walk keeps the first it tests.**
`0x14004ad29` tests `0x10` and takes that branch whatever else is set, so `0x20` is never looked at with it
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

**The job at `0x140058330`**, whose calc-chain priority is nought and which therefore runs first of the
frame: `0x1400583b8` copies `0x140955390` into `0x140955394`, `0x1400583c4` calls `0x14000fa80` for the
new word, and `0x1400583cf` writes it back. So the word is built, the player moves on it and the bullets
test against where it moved, all inside one walk of the calc chain.

**And `0x14000fa80` builds it out of the 256 bytes at `0x14042893c`**, which the read at `0x14008e280`
fills: `GetKeyboardState` into the stack, `0x14008e297` memsets those 256 bytes to nought, and the walk at
`0x14008e2c0` puts `snapshot[entry.key]` at `buffer[entry.index]` for every entry of the table at
`0x1402bc0b0` until its `0xffff`. **So the game's own read is a whole state and not a set of presses** —
every key nobody holds is written up — and that read is made at its own rate rather than once a frame.

## The hit test itself, and what it halves

`0x14004c690` is what every hazard asks about the player, and it is two boxes overlapping:

- `xmm3` is `[0x14030dfc8]`, **a half**, and every size that reaches this function is multiplied by it:
  the hazard's own, handed in `r8` — `[r8]` and `[r8+0x4]` at `0x14004c6a9` and `0x14004c6b0`, each
  `mulss xmm3` — and the player's, `mulss` at `0x14004c707` and `0x14004c72a`. So a `SIZE` field is a
  width and what a hit is is half of it on each side, which is what orb reads them as.
- The hazard's place is `rdx`, and the edges are that place plus and minus its own half:
  `0x14004c6e2`–`0x14004c6f2`.
- **Sixteen boxes of the player's come first.** `lea rax,[rcx+0x7274]` and the walk at `0x14004c6f6`
  onwards compares the hazard against each of them and answers 1 rather than 2 where one contains it;
  a slot whose first float is nought is empty and skipped, which `0x14004a59b` onwards makes of all
  sixteen every frame. **That prologue is not the player's own box**, and reading it as one is a player
  modelled as a point: a launch that did read `half of it 0.0,0.0` off it had nowhere open to it on
  every frame of a stage.
- **The player's own box is the two corners at `+0x410` and `+0x41c`**, compared at
  `0x14004c752`–`0x14004c788`, which is `+0x7264`'s half either side of the position. It comes out
  `1.2` by `1.2`. They are absolute and they are the position's: a hit's dump read them against
  `+0x7248` and got `193.4,365.4..195.9,367.9` against a position of `194.6,366.6`.
- **And the bullet is already where its step took it.** `0x14000e377`–`0x14000e397` adds `+0x8`,`+0xc`
  into `+0x30`,`+0x34` and `0x14000e485` hands *that* to the test — so the place compared is this frame's
  and not the place the bullet was read at.

## The field the player may move in

**`0x1403ed01c` onwards, four floats initialised in `.data`** — `32`, `16`, `384`, `448`. The first pair
is where the field sits on a 640 by 480 screen and what `0x14000a0ad` and `0x14000a0c4` add to a position
to draw it; the second is the field's own size, and every comparison a coordinate is held against reads
`0x1403ed024` and `0x1403ed028`.

**`0x1403ee7f4` onwards is a second rectangle, and it is the one the player is held to** — left, top,
width, height, read at `0x14004b006`–`0x14004b05f`. It initialises to nought in `.data` and no
instruction in the flat listing writes it; what does is not established, and the flat listing is not
evidence of that — see the note on `.pdata` above.

## The run's own numbers

| | |
| --- | --- |
| `0x1403ee804` | **lives**, one byte. `0x14002b638` stops it at 8 and `0x14002b647` steps it up; `0x14004a7bd` is where a death reads it |
| `0x1403ee805` | **bombs**, one byte, the same shape at `0x14002b609` and `0x14002b618`, and what `0x14004a6b2` takes one off when a bomb is pressed |
| both together | `0x14002cc30` writes `0x302` over the pair as a run starts — two lives and three bombs |
| `0x1403ee7e0` | a dword something multiplies into at `0x14002b8e0` |
| `0x1403ee7ec` | a dword `0x14001556a` adds `0x708` to and `0x140020243` steps up by one |
| `0x1403ed034` | **the power**, a word. `0x14002ab8e` is the `0x80` every case of the item walk writes it back at, so 128 is where it stops; `0x140027286` is the `Power * 100` line of the bonus screen; and `0x14004a82c` takes 16 off it on a death, or to nought where it had less |
| `0x1403ed030` | **the stage, counted from one**. `0x140027255` multiplies it by a thousand into `Stage * 1000 = %5d`, and `0x14002731e`, `0x140027214` and `0x1400274a4` all treat six as the last — which is what gives the lives and bombs their bonus on an all clear and nowhere else |
| `0x1403ed7d8` | **the difficulty**, nought for Easy up to four for the Extra: `0x140027393` picks between `Easy Rank * 0.5`, `Normal Rank * 1.0`, `Hard Rank * 1.2`, `Lunatic Rank * 1.5` and `Extra Rank * 2.0` in that order |
| `0x1403ed7c0` | **the score**, which every case of the item walk adds to |
| `0x1403ee7e0` | **the graze**, `0x1400272b7`'s `Graze * 10` |
| `0x1403ed7d4` | **the point items collected**, a word `0x14002b2d8` steps up beside the score |

**Every one of those is a field of the game manager, the static at `0x1403ed010`** — `lea rdi,[rip+0x37f411]`
at `0x140024148` hands it to the stage callback and the deinit registered at `0x14002413a` and
`0x14002414f` — and the section after the front end has the rest of its fields. `0x1403ed7d4` and
`0x1403ed7d6` are two words: the first is what the item walk steps, the second is what the replay's
stage record saves (`0x14004da71` onwards) and puts back (`0x14004db64`), so it is the second a stage is
restarted from.

**The front end is the static `0x140ba73c0`**, `0x11628` bytes, and its state is the dword at
`+0x11608`: `0x14002cd2b` reads it and `0x14002cd48` jumps through a table of twenty-one at
`0x140036414`. Its frame callback is `0x14002ccf0`, which is also what clears the input word above. It
was taken for the game manager while the run's own numbers were being read, and the run's numbers are
none of them in it: what is in it is the screens a run is chosen on, and the run's are the static above.

**Twelve to seventeen share one entry**: `0x14002fa4d` takes twelve off and jumps through a second table
of six at `0x14003648c`, whose first case calls `0x140022950` and writes thirteen back into `+0x11608`.
**They are not a stage.** A launch that steered only in those six never steered at all — its run was over
at `得点 000001800` with the lives row empty and the game asking 「コンティニュ〜？」 — and what state 13
does is copy `0x70`-byte records into a list at `0x14002fab4` onwards, which is a screen of the front end
and not a stage.

## The front end

**Its cursor is `0x140ba73c0 + 0x28`**, one dword and one field whatever list is up: `0x140030740` steps
it up and wraps it at seven, and the decide at `0x140030785` reads the record of the item it names
through `+0x11610`. That record's own bytes are what a run is started from — `+0x7` into
`0x1403ed7d8`, the difficulty, and `+0x6` split in two at `0x1400307ef` and `0x140030803` into
`0x1403ed02c` and `0x1403ed02d`.

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
| 1 | **the logo screen, and what starts the demo**: `0x14002d06d` steps `+0x9454` up, any bit of the input word resets it to nought, and `0x14002d094` jumps to `0x14002f9f6` — the demo's own start — once it reaches `0x2d0`, which is 720 frames. A press moves it on to state 2 instead. See the section after this one, the demo itself not being this state |
| 2 | the title, and its first item is what starts a run |
| 6 | what the title's first two items open, and **it takes no press**: its line arrives and the screen becomes 7 half a second later with every step of the recipe pressing nothing. A recipe that presses here presses at the difficulty instead, and takes whatever item the score file left that cursor on — Lunatic, on three launches running |
| 7 | **the difficulty**, and its items *are* the difficulty: a launch whose step overshot and left the cursor on item 3 played `run: stage 1 on difficulty 3` — Lunatic — so item 1 is Normal. **For the Extra, which has no difficulty select, this is the character select**: a launch walking in from the title's second item said `screen 6, cursor on item 1`, `screen 7, cursor on item 0`, `screen 9`, and `run: stage 7 on difficulty 4` off the decide at 9 — so the Extra's two selects carry a full run's first two numbers |
| 9 | the list after it, which takes two presses on some launches and one on others — the character select of a full run, the shot select of the Extra |
| 11 | **a stage**, and it stays 11 for the whole of one: `run: stage 1 … scene 11` and then `run: stage 2 … scene 11` |

Which of the twelve that are left is which is unread. **Where each one's code is, is not**: the table of
twenty-one at `0x140036414` holds an RVA apiece, and `0x14002fa4d`'s second table of six holds the six
that twelve to seventeen share, so a state's case can be disassembled without a launch to reach it —

```sh
python3 -c '
import struct; d=open("th06c.exe","rb").read()
def at(va): return va-0x140001000+0x400
def case(table, i): return 0x140000000+struct.unpack_from("<I", d, at(table)+4*i)[0]
for i in range(21): print(f"state {i:2d} 0x{case(0x140036414, i):x}")
for i in range(6):  print(f"state {12+i:2d} 0x{case(0x14003648c, i):x}")'
```

— which says state 0 is `0x14002cd69`, 1 `0x14002d06d`, 2 `0x14002d509`, 11 `0x1400353b3`, and the six
`0x14002fa69`, `0x14002fa7d`, `0x14002fc96`, `0x14003012b`, `0x14003010d` and `0x1400306b6`.

## The game manager, the supervisor, and the seams a stage's numbers go in at

**The game manager is the static at `0x1403ed010`.** Its stage callback is `0x1400242d0` and its deinit
`0x1400254d0`, registered at `0x14002413a` and `0x14002414f` with the static as the element's argument
(`lea rdi,[rip+0x37f411]` at `0x140024148`), so the callback is entered with it in `rcx`. Its fields, each
with the instruction it was read off:

| | | |
| --- | --- | --- |
| `+0x8` | the run's captures | `inc DWORD PTR [0x1403ed018]` at `0x140014e99` |
| `+0x1c`, `+0x1d` | the character and the shot, bytes | `0x14004d997` puts `character * 2 + shot` into the replay's header; written at `0x140035329` and `0x140035901` |
| `+0x20` | the stage, from one | `0x14004da49` |
| `+0x24` | the power, a word | `0x1403ed034` |
| `+0x7a8` | a replay is playing | `0x1403ed7b8`, see the next section |
| `+0x7ac`, `+0x7b0` | the score the panel draws and the score the run counts | `0x14004da37` saves the second; `0x140014e1f` adds the bonus to it |
| `+0x7bc`, `+0x7bd` | the pause menu and the continue menu are up, bytes | th06 1.02h's own two, at the same distance from the score |
| `+0x7be` | the run is running, the byte the recording is gated on | `0x14004d800`; cleared at `0x14002568e` |
| `+0x7c0` | practice | |
| `+0x7c1` | the playback is the demo | `0x1403ed7d1` |
| `+0x7c6` | the point items, a word | `0x14004db64` |
| `+0x7c8` | the difficulty | `0x1403ed7d8` |
| `+0x7cc` | sixty-four `CATK` records of `0x40` | `0x1403ed7dc`, below |
| `+0x17e0` | the retries, a byte | drawn by the result screen |
| `+0x17e1` | the power items towards the score, a byte | `0x14004dbba` |
| `+0x17e4` | the field the player is held in, four floats | `0x1403ee7f4` |
| `+0x17f4`, `+0x17f5`, `+0x17f6` | the lives, the bombs, the extra lives the score paid for | `0x1403ee804`, `0x1403ee805` |
| `+0x17fc` | four `CLRD` records of `0x18` | `0x140054a0d` |
| `+0x195c` | **the seed the stage started from**, a word | `0x140024e9c` copies the generator into it as the stage begins |
| `+0x196c` | **the frames the stage has run** | |
| `+0x1970`, `+0x1974`, `+0x1978`, `+0x197c` | the rank, its most, its least, the sub-rank | `0x1400243cf` starts the rank at eight; `0x140024e7e` picks the bounds out of a table by difficulty |

**The supervisor is the static at `0x140bc2580`**, whose first field is the pointer to the sound object
(`mov rcx,[0x140bc2580]; test rcx,rcx; je` at `0x14002cc82`). Its state is the dword at `+0x32c`
(`0x140bc28ac`) and the state its update last acted on the dword before it at `+0x328`, written from the
first as the update's last act — so the two disagreeing is a change asked for and not yet made, which is
the one frame a run has been chosen on and nothing of it built. **The numbers are th06 1.02h's**: the run's
start at `0x14002cc40` writes two, the result screen leaving writes one at `0x140050d79`, an ending's end
writes seven at `0x140018f05`, and `0x1400242f9` reads three as the stage callback begins to tell a stage
move from a run's start. `+0x344` and `+0x345` are the lives and the bombs a run starts with, out of
`th06.cfg`'s `+0x10` and `+0x11`: `0x14002cc0b` copies them over the pair and `0x14002cc30` writes two and
three instead where the difficulty is four.

**The run's own start is `0x14002cc00`**: the lives and the bombs from the configuration, the supervisor's
state to two, `0x1403ed7b8` cleared, and the music stopped and the track put to `-1` at `0x14002cc9e`
onwards — which writes `0x1403eea38` to `0x1403eea60` together, the block a restore leaves alone.

**The generator is one word at `0x14095538c`**, every draw being `xor cx,[0x14095538c]; sub cx,0x6553;
shl cx,2; shr ax,0xe; add ax,cx; mov [0x14095538c],ax` (`0x1400243f6` onwards), and the count of draws is
the dword at `0x140955388`, stepped beside every one and cleared at `0x140024e93` as the stage begins.
**The stage callback draws 2048 of them before the file is parsed over the records** — `0x1400243f0` to
`0x1400249a5`, thirty-two words into each of the sixty-four records — which is th06 1.02h's own shape and
the reason a resumed stage's seed goes in before that callback runs anything.

**And that callback is one function reached two ways**, the whole difference being its first condition:
`0x1400242f9`'s `cmp DWORD PTR [0x140bc28ac],0x3` jumps a stage move to `0x140024cf5` and lets a run's first
stage fall through. What falls through is a run being *made* — the movement area at `+0xc`..`+0x18` and the
box at `+0x17e4`..`+0x17f0`, the rank at `+0x1970` set to eight (`0x1400243cf`), and the `0x186a0` the next
extra life costs at `0x140024382`. **What the move takes is eight instructions and no number of the run's**:
`0x140024cf5` copies the score at `+0x7b0` into the panel's own copy at `+0x7ac`, zeroes `+0x7b4`, the
sub-rank at `+0x197c` (`0x140024d10`), `+0x7c4`, `+0x17d0` and `+0x7bc`, and steps the stage word at `+0x20`
up by one. The lives, the bombs, the power and the rank are written nowhere in it, which is what a run is:
six stages with one set of numbers walked through them.

**The two seams.** The stage callback `0x1400242d0` puts the stage's numbers in place — the rank's bounds
at `0x140024e7e`, the seed copied at `0x140024e9c` — and then calls the stage's own build `0x1400572d0`
at `0x140024ea3`, the one call of it in the exe, with the stage in `ecx`; it answers nought where it built
one and `-1` where it could not (`0x140024ebf`). So the seed of a resumed run goes in in front of
`0x1400572d0` and the rest of its numbers after `0x1400242d0` has returned, which is what `Hooks::stage_begun`
and `Hooks::stage_building` are over. Their prologues: `48 89 5c 24 18 48 89 4c 24 08 55 56 57 41 54`
and `48 89 5c 24 08 48 89 74 24 10 57 48 83 ec 20`, fifteen bytes each and none rip-relative.

**The replay manager is the pointer at `0x140955380`**, an object of `0x78` bytes made at `0x14004d170`
for a recording as much as for a playback: `+0x8` says which, `+0x30` is the frames of the stage the
record is at (cleared at `0x14004d950`, stepped at `0x14004d861`), `+0x70` the `T6RP` header, and
`+0x38` onwards the seven stages' records. **Its stage record is what a stage is restarted from**: the
recording init `0x14004d940` saves the score (`0x1403ed7c0`), the seed (`0x1403ee96c`), the point items
(`0x1403ed7d6`), the power, the lives, the bombs, the rank (`0x1403ee980`) and the power items
(`0x1403ee7f1`) at `0x14004da71` onwards, and the playback init `0x14004dae0` puts every one of them back
at `0x14004db3a` onwards — the seed into the generator at `0x14004db70` with the count cleared at
`0x14004db77`. Those eight, and orb's own two beside them, are `th06c/resume.rs`. An entry of the record
is `{frame, input & 0x3ff, previous & 0x3ff}`, twelve bytes, written where the word changes
(`0x14004d80f`–`0x14004d85e`); the playback puts the low ten bits of `0x140955390` back at `0x14004d8ff`.

**The card records are `0x1403ed7dc`**, sixty-four of `0x40`: `CATK` at `+0`, the name's checksum at
`+0x12`, the name at `+0x18`, the attempts at `+0x3c` (capped at 9999, `0x140014d4a`) and the captures
at `+0x3e` (`0x140014e35`). The card a boss is on is the dword at `0x140b7c1d8`, shifted by six into the
array at `0x140014c99`; whether one is up is `0x140b7c1d0` (`0x140014d64`). Both counts are gated on
`0x1403ed7b8` being clear, and both are inside the enemy manager's static — `+0xfdff8` and `+0xfdff0` of
`0x140a7e1e0`, whose `+0xfdfdc` is the script's clock (`0x1400201c2`) and `+0xfe008` the pointers to the
bosses (`0x14001458a`).

**The boss's attack timer is the dword at `+0x4` of the enemy** and its limit at `+0xbb4`: the panel's
countdown is `([+0xbb4] - [+0x4]) / 60` at `0x1400202df`–`0x140020300`, and `0x14002030c` ends the attack
where the two meet.

**The panel's block is the pointer at `0x140955358`**, made at `0x1400251f0`; `+0x23a0` is which line of a
conversation is up and negative for none (`0x140021232`, and the playback's fast-forward at `0x14004d870`).
`0x140955364` is the byte that says a boss is drawn, set at `0x140014592` and cleared at `0x140020b7a`.

**The player's states** at `+0x73b8` are nought, one for a respawn, two for the hit and three for
invulnerable; `+0x7254` is the frames a bomb still takes the hit back in, `+0x7378` the frames of
invulnerability, and `+0x97d0` is set while a bomb goes off. **This exe keeps no count of deaths**: nothing
of the game manager is stepped by `0x14004c7f4`'s write of the hit, so orb counts the hits itself.

**The result screen's states** at `screen + 0x5194`: ten is the question about saving a replay, whose case
calls the save at `0x1400528cc`, and seventeen is the way out, whose case puts the supervisor back to one at
`0x140050d79`.

**And the ranking is the same screen reached through the title's Score item**: that item writes the front
end's state to ten at `0x14002e9bd`, whose case at `0x1400362c4` puts the supervisor into six once the screen
has been up `0x3c` frames — th06 1.02h's `SUPERVISOR_STATE_RESULTSCREEN` — and the supervisor builds the
result screen for it. The item after it writes eighteen, whose case puts the supervisor into nine after the
same wait (`0x1400362f7`), and the front end's state four puts it into four (`0x1400362b7`). Which is how orb
has a run given up written into the file: the screen's deinit `0x140054ab0` writes it, and that screen is
built for the ranking as much as for a run's own end.

**The title's own frames** are `front end + 0x9450`, zeroed as each screen arrives: the title reads its
decide past `0x14` (`0x14002d614`) and the shot select past `0x1e` (`0x1400354d8`). The title's items are
th06 1.02h's eight in its order — `0x14002d592` steps the cursor over the second where no clear unlocked
the Extra — so nought starts a run, one the Extra, two a practice, three is the replay list and four the
ranking.

**What that step is decided by** is `cmp DWORD PTR [rdi+0x28],0x1` at `0x14002d520` — the cursor on the
second item and nothing else reaches the check — and then twelve `cmp BYTE PTR [...],0x63` each jumping
past the step where the byte is a clear, from `0x14002d526` to `0x14002d589`. The twelve bytes are
`0x1403ee819`–`0x1403ee81b`, `0x1403ee831`–`0x1403ee833`, `0x1403ee849`–`0x1403ee84b` and
`0x1403ee861`–`0x1403ee863`: three consecutive bytes in each of four blocks `0x18` apart, which is the
size of a `CLRD` record and one block per shot. Where none is `0x63`, `0x14002d592`'s `inc eax` and
`0x14002d594`'s `mov DWORD PTR [rdi+0x28],eax` put the cursor on the third item. `CLEARED` in
`th06c/score_file.rs` is that `0x63`, and `a_clear_is_among` is what builds a copy the check answers
for.

**The score file** is read once at startup (`0x14002c239`, into `0x140bc2ad8`), never again; every screen
parses that copy through `0x14004e600`, and the result screen's deinit `0x140054ab0` writes the whole file
back out of memory at `0x140054ade` onwards. The parse of the `CATK` records over the array is
`0x14004e7d0`, called at `0x14005497c` from the result screen's init `0x1400544a0` — and skipped there for
states nine and seventeen (`cmp eax,0x9` at `0x14005495c`, `cmp eax,0x11` at `0x140054965`), the two a run of
its own arrives in, so a finished run's own count of card attempts is not laid over by the file's.

**That read is inline in the startup and not a function**: `0x14002c1f4`..`0x14002c2c0` sits in the middle
of the routine that begins at `0x14002c0xx`, storing the generator's seed just before it — `fopen`
(`0x140290de0`, `"score.dat"` at `0x14030d628`, `"rb"` at `0x14030d4c8`), `fseek`/`ftell`, `malloc`
(`0x14028f790`), `fread`, `fclose`, and the block and its length stored at `0x140bc2ad8` and
`0x140bb89ec`. Every one of those is a function of the exe's own CRT, and the file is opened in the end
through the `CreateFileW` it imports. So there is no prologue for a jump to go over, and orb brackets the read from the attach to
the first settling of the mode instead — `Game::reads_its_file_once_at_startup`.

**Every parse decrypts a fresh block**: `0x14004e600` takes the length at `0x140bb89ec`, `malloc`s that
much, `memcpy`s the copy in (`0x1402b8920`) and deciphers the block from its third byte at `0x14004e663`
— `add r9b,cl; ror r9b,5; xor` — so the copy itself stays as the file was. **And the write rebuilds the copy
from what the screens hold**: `0x140055460`..`0x1400554e5` lays the records out again, `0x1400554f6` sums
the checksum and `0x140055530` enciphers, then `malloc` (`0x140055555`), `memcpy`, `free` of the block that
was there (`0x14028f780`, called at `0x140055577`), the new block and length stored
(`0x14005557c`, `0x140055583`) and the file written from it (`0x14005559d`). Which is why a copy orb installs
has to come off the game's own `malloc`: the game frees it. `score_file.rs` is the cipher and the record
walk as read off those two.

## The demo, and a replay being played back

**Two bytes, and the demo is the narrower of them.** Both are `.data` and both are one byte:

| | |
| --- | --- |
| `0x1403ed7b8` | **a replay is being played back.** `0x14003079b` sets it where the replay list's own decide runs, `0x14002f9fd` sets it where the logo screen starts the demo, and `0x14002cc77` clears it — in the same code that writes `0x302` over the lives and the bombs at `0x14002cc30`, which is a run starting. What reads it is `0x140024e02`: set, the path at `0x1403ee86c` goes to `0x14004d170` to be *read*; clear, `replay/th6_00.rpy` at `0x14030c428` goes to that same call from `0x1400253c6` to be written |
| `0x1403ed7d1` | **and the playback is the attract demo.** `0x14002f9fd` is the only writer that sets it and `0x140037930` the only one that clears it |
| `0x1403ee86c` | **the path being played.** `0x14002fa11`–`0x14002fa2b` copies the 21 bytes of `data/demo/demo00.rpy` (`0x14030cc38`) over it; `0x1400307c0` copies the NUL-terminated name out of the record the replay list's cursor is on. How long the buffer is, is unread |
| `0x140ba73c0 + 0x9454` | **how long the logo screen has been idle**, in frames — see state 1 above |

**The demo is played inside whichever screen started it.** Its start at `0x14002f9f6` sets both bytes,
writes the path, and puts `3` into the stage at `0x1403ed030` and `3` into the difficulty at
`0x1403ed7d8` — `r13d`, which the frame callback's prologue loads at `0x14002cd55` — and then returns
**without writing the manager's state at all**. So the state stays whatever it was: one launch's own
lines read `scene 1` for the whole of a demo started off the logo screen and `scene 2` for one started
off the title, with a run's own lives and bombs moving underneath — `2 live and 3 bomb(s)`, then a bomb
spent.

**And the state is not what tells a demo from a run.** Another launch read
`stage 7 on difficulty 4, scene 11` while the screen said `DEMO PLAY` over stage 4's library, and a
third `stage 4 on difficulty 1, scene 11` for 34,200 frames of one. Neither of those pairs is the `3`
and `3` the start writes, so the numbers a playback ends up showing come out of the replay's own header,
and where *that* is read is unread — which leaves `0x1403ed7b8` as the only thing that answers, and is
why `sight::in_a_run` asks it rather than asking the scene.

**What said it was a demo was a picture**: ` DEMO PLAY` at `0x14030ca90`, which `0x140029b3f` draws in
place of the stage's own name — `Stage %d` at `0x14030ca60` and the two at `0x14030ca70` and
`0x14030ca80` — while `0x1403ed7d1` is set. The same byte gates the frame's own fast-forward at
`0x140025909`: `0x1400258a0` runs the calc chain up to seven extra times where a replay is playing
(`0x1403ed7b8`), the playback is *not* the demo, `0x1403ed7cc` is clear, `0x140bc28ac` is 2 and
`0x140955390`'s `0x8000` is held.

**And the hand-over back to the title tells them apart as well.** `0x140037050` is what a stage ends
into: `0x140037576` tests `0x1403ed7d1` and, where it is set, jumps to `0x140037924` — which clears that
byte and `0x1403ee808` and returns, without the `bgm/th06_01.mid` at `0x14030cc70` that a run's own
ending gets. `0x1403ed7b8` is not cleared there, so it stands through the title a demo hands back to and
until the next run starts.

## The replay a run records, and the name it is registered under

**The replay manager is the pointer at `0x140955380`**, and `0x14004d170` is the one call that reads or
writes a file: `cl` says which — `1` reads the path at `0x1403ee86c`, nought writes `replay/th6_00.rpy`
from `0x1400253c6` — and `0x14004d544` is where the record is made when the manager has none.

**The record is `manager + 0x70`**, `0x70` bytes from `0x140286c70`, and is the replay file's own header:

| | |
| --- | --- |
| `+0x0` | `T6RP`, written as the dword `0x50523654` at `0x14004d591` |
| `+0x4` | a word, `0x103` at `0x14004d5b1` |
| `+0x6` | the character doubled plus the shot — `[0x1403ed02c] * 2 + [0x1403ed02d]` at `0x14004d597`, the same pair the front end's own decide split in two |
| `+0x7` | the difficulty, off `0x1403ed7d8` at `0x14004d5bb` |
| `+0x10` | the date, `MM/DD/YY` and a NUL |
| `+0x19` | **the name, eight bytes**, initialised to `NO NAME` by the `movabs` at `0x14004d5c5` |

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

**The grid is at `0x14030d500`** — `ABCDEFGHIJKLMNOPQRSTUVWXYZ.,:;¥@abcdefghijklmnopqrstuvwxyz+-/*=%`
and the rest of the 96 th06 1.02h offers, `¥` being the `0x5c` that build has too.

### The entry itself

**It is a state of the screen a run ends at**, and that screen is reached the one way a heap object of
this game's is: the job it registers in the calc chain. `0x14004fd6d` allocates the `0x40`-byte element,
`0x14004fd94` puts the update `0x14004ff40` in it and `0x14004fdcd` puts the screen itself at the
element's `+0x38` — so a walk of the chain from `0x1403ec5f0` for that callback finds the screen.

| | |
| --- | --- |
| `screen + 0x5194` | **its own state**, read at `0x14004ff71` and jumped through a table of eighteen at `0x1400535c8`. `0x14004fde9` writes it as the screen is built: **nine** where `[0x1403ed7d0]` is clear and seventeen where it is not, the latter being a run asked for no name |
| `screen + 0x0` | **the grid cursor**, the object's first dword. `0x140052023` reads it, `0x14005202b` takes the character at the grid, `0x140052026` compares it with `0x5e` — the space item, whose case writes `0x20` at `0x14005203c` — and `0x140052036` leaves the typing for anything past it |
| `screen + 0x5173` | **the name, eight bytes.** `0x140052044` writes the character at `[screen + 0x5173 + index]` |
| `screen + 0x517c` | **how many have been typed**, the index above — capped at seven by the `cmovge` at `0x140052020`, stepped up at `0x140052058`, stopped at eight by `0x140052051`, and reaching eight is what `0x140052063` puts the cursor on `0x5f` for |

**Which is th06 1.02h's own shape with the cursor moved to the front**: that build has the name at
`[this+0x5193]`, the count at `[this+0x10]` and the cursor at `[this+0x20]`.

**And the grid is reached off the image base in a register**: `movzx edx,[rax+r11*1+0x2c8af0]` at
`0x14005202b`, `r11` holding `0x140000000` from the `lea` at `0x14004ff91`. That is why no search of the
exe finds a reference to it — **no rip-relative displacement anywhere in `.text` resolves to
`0x14030d500`, to the pointer at `0x14030ae00` that holds it, or to `score.dat` at `0x14030c228`, and
none of the three is a `movabs` operand**, swept by reading every four-byte window of `.text` as a
displacement and the whole file for the addresses as words. **So a negative from that sweep says
nothing about this exe**: it keeps its own base in a register and indexes with a 32-bit displacement.

## The enemies

**A static of `0xff020` bytes at `0x140a7e1e0`**, whose frame callback is `0x140020780` and whose array
that function's own loop says the shape of: the live count in the manager's first dword
(`inc DWORD PTR [r14]` at `0x14002085d`), the array at `+8`, `0xfd0` apart (`0x140021357`), two hundred
and fifty-six of them (`cmp r15d,0x100` at `0x140021365`).

| | |
| --- | --- |
| `+0xa8` | where it is, three floats — `lea rbp,[rbx+0xa8]` at `0x140020860`, and `+0xfa4` onwards is what the walk adds to it |
| `+0xb4` | the flags the walk itself gates on: `cmp BYTE PTR [rbx+0xb4],0; jge` at `0x140020850` skips a slot whose top bit is clear, so `0x80` is in use |
| `+0xb5` | **and whether its body is run against the player at all**: `0x140020d82` requires `0x4`, `0x140020daf` runs the test only where the low two bits are both up, and `0x140020d9c` refuses where `+0xb6`'s `0x8` is set |
| `+0x2a4` | **the size that test is given**, three floats, each multiplied by two thirds at `0x140020db5` before `0x14004c690` — which halves what it is given, so a hit is a third of this on each side |
| `+0x21c` | its life, which the body taking the player off it drops by ten at `0x140020e14`, and `+0x220` is the most of it |
| `+0xb5`'s `0x8` | **the one the fight is with.** `0x140021222` is the only test of that bit, and for the enemy carrying it `0x14002123b` works `+0x21c / +0x220` out into `0x14095534c` — which is the life bar the panel draws, so whoever has the bit is the boss. Nothing keeps a pointer to it |

## The items

**A static of `0x2a160` bytes at `0x140b7d260`**, walked by `0x14002aa70`: the live count in its first
dword, the array at `+8`, `0x150` apart (`0x14002ba32`), five hundred and twelve of them
(`cmp r9d,0x200` at `0x14002ba56`). A slot whose first byte is nought holds nothing — `0x14002abe0`.

**Its kind is the signed byte at `+0x34`**, and `0x14002ae97` jumps through a table of seven at
`0x14002bb1c`. What each number is is what its own case does:

| | |
| --- | --- |
| 0 | `inc r15w` into the power at `0x14002af2e` — a power item |
| 1 | the point items and the score stepped up at `0x14002b2d8` — a point item |
| 2 | `add r15w,0x8` into the power at `0x14002b39e` — the big power item |
| 3 | the bombs, stopped at eight and stepped up at `0x14002b609` |
| 4 | the power written to its cap from `0x14002b6eb` — the one that fills it |
| 5 | the lives, the same shape at `0x14002b638` — a 1up |
| 6 | a score off the graze at `0x14002b8db`, and the bullet manager spawns this one as kind 6 at `0x14000e46d` where a bomb clears a bullet |

`+0x10` is where it is, three floats, and `0x14002ae68`–`0x14002ae91` is the box around the player it is
collected inside — `0x140bc2568` onwards, three floats this reading has not gone into.

**And the bullet's own size is `+0x5a4`**, which `0x14000e47e` hands to `0x14004c690` beside the position
at `+0x30`: that is the box the player is tested against, and `+0x5c8` is the byte the graze sets.

## What the update changed

The build of md5 `acadbc0c` and this one are the same source built again, and **what moved is where
things are, not what they are**: 4,154 of the old exe's 4,239 functions out of `.pdata` have a match here
whose bytes agree once the rip-relative displacements and call targets are masked, 3,208 of them exactly,
and among the 600,615 instructions of the matched pairs no field of an object this document names is read
at another offset. **There is no one slide**: `.text` moves by a different small amount almost function by
function, `.rdata` by `+0x34dd0` to `+0x46c50`, and `.data` in six runs from `+0x49000` to `+0x4b280` — so
each address here was moved by the function or the reference it is in, never by a formula.

What did change, and what orb had to change with it:

- **The CRT is linked into the exe.** `malloc` is `0x14028f790`, `free` `0x14028f780` and `fopen`
  `0x140290de0`, called directly, and the heap they use is the one `GetProcessHeap` answers once at
  `0x1402adf64`, kept at `0x140955178` — no `HeapCreate`, no `VirtualAlloc`. Threads come from
  `CreateThread` and files open through `CreateFileW`, both imported. Which is what
  `Memory::ProcessHeapImport`, `Threads::CreateThread` and `FileOpen::CreateFileW` are.
- **The frame's view is one call**, `0x1400591d0`, where the frame used to set the viewport and the
  matrix itself — see the frame's own table — and **every draw queues its sprites into the renderer's
  batch**, which the draw walk flushes after each job (*The chain walk*). The renderer is `0x32ba8`
  bytes where it was `0x22c40`, the batch appended past every field it had.
- **The frame's own prologue is another**: `mov [rsp+0x10],rbx; mov [rsp+0x18],rbp; mov [rsp+0x20],rsi`
  at `0x1400258a0`, with no stack cookie in the frame any more. The two other prologues orb writes over,
  `0x1400242d0` and `0x1400572d0`, are the bytes they were.
- **The nine Steam imports are in another order**, which nothing reads, orb answering them by name.

## What is not established

- **Which of the moved addresses were read by hand rather than matched.** Nine are the medium-confidence
  ones of the move: the two reads of the field's floats at `0x1400079b6` and `0x1400079c5`, which this
  build takes in another order; the matrix calls at `0x140059272` and `0x1400592a8`, now inside the view's
  own function; the start of the startup's inline score read at `0x14002c1f4`, whose first two
  instructions were hoisted above it; and the CRT's `memcpy`, `cosf`, `sinf` and `sqrtf` at
  `0x1402b8920`, `0x1402b6390`, `0x1402b6c80` and `0x1402b6fd0`, which were import thunks before. Each was
  read side by side against the old build, and none of them is an address orb writes to.
- **Whether the CRT's objects off the heap read the same after a restore.** With the CRT in the exe, every
  allocation it makes for itself — an open `FILE` and the lock inside it, the table of low-level handles
  `0x140954b70` points into — comes off the heap `GetProcessHeap` answered, which is the game's own heap
  and put back with a chapter. Its statics in `.data` are held back (`Th06c::live_handles`); these are not,
  and a file opened or closed between a chapter being taken and being put back is the case that would
  find its handle table as it was.
- **Where the `HWND` the game plays in is kept**, and what its class's `WndProc` does with a message:
  the calls that make the window and pump it are the slots above, and what they are given and what is
  done with what they answer is unread.
- **Which of the draw chain's jobs calls `Present`**, and from what address. What the swapchain *is* is
  established and needed no address at all: `0x1402522e9` asks `GetProcAddress` for
  `CreateDXGIFactory2` — the string is at `0x1402c9860`, and the module it asks of is kept at
  `0x1408971a0` — so orb answers that name, hands the game a factory of the host's own with
  `CreateSwapChainForHwnd` written over, and takes the swapchain the game then makes. A launch says so:

  ```text
  hook: the game asked for CreateDXGIFactory2 and got orb's, the host's being at 0x7fff4d1bf290
  frame: the swapchain this game makes will be orb's to present through
  frame: both of the presents are orb's to call now, the game's own doing nothing
  ```

  **And it is not among the objects the release names.** `0x140251a24` onwards releases `0x1408971b8`,
  `0x230`, `0x238`, `0x240`, `0x248` and `0x260`; a launch asked every one of those and every pointer in
  the renderer's own object besides — nine objects whose vtable is a mapped image's — and not one
  answered to `IID_IDXGISwapChain`. So the swapchain is kept somewhere no reading of the exe has found,
  and the factory is the way to it.
- **What a launch with no client does past the frames a recipe reaches.** The nine imports, the four
  interfaces and the seven slots are read — see *What it asks that DLL* — and every slot but the
  language's is answered with nought, which the game passes over. What has not been watched is a whole
  run of one: a slot nobody has seen called would be an unread call into `orb::steam`'s own vtable, whose
  every other entry answers nought as well.
- **Which of the front end's twenty-one states is which screen**, past the six named above and the
  practice stage select.
- **A count of deaths**: none was found in the game manager, and orb counts the hits itself.
- **Where the game's sprites keep their textures**, which a restore would have to leave alone were one
  released between two chapters of a stage; only the renderer's own COM statics are known.
- **What the sound library is**, past the block at `0x1404979b4` its own code alone reaches and the five
  locks and pools in it: which of its threads takes which lock, what the pool at a lock's end holds, and
  why the thread holding `0x140497f50` held it for a tenth of a second when a chapter had been put back
  over it. What orb needed was that the block is the library's and not the run's, which the references
  answer; the rest is unread.
- **What `0x140a3dc28 + 0x9e00` onwards is.** Sixty-four lasers of `0x278` fill the first `0x9e00` of
  the `0xf408` the startup zeroes there, and what the rest holds is unread.
- **What writes `0x1403ee7f4`**, the rectangle the player is held to — see the section on the field.
- **Which three of a `CLRD` record's ten stages-reached the title's Extra check reads.** The twelve
  addresses are established and so is the `0x18` between the four blocks — see the title's items — and
  where a block begins is not, so the three consecutive bytes are `+0xc`–`+0xe` of the array reached
  with retries or `+0x11`–`+0x13` of the one reached without, and which difficulties those are is
  unread. Orb itself needs none of it, installing the file's own records and leaving the check to the
  game; what rests on it is `orb-e2e`'s fake, whose `score_file::a_clear_is_among` offers the Extra
  for a clear anywhere in either array where the real title reads three bytes of each record. The
  cases put their clear in `+0xd`, the second of the array reached with retries
  (`clrd(0, [1, CLEARED, 1, 1, 1], [1; 5])`), which the first candidate has the title reading and the
  second has it not — so which candidate holds is what says whether those cases stand on a byte the
  real game looks at. The first is the likelier of the two and that is an argument rather than a
  measurement: it has the title reading the first three difficulties, and an Easy clear unlocking the
  Extra would be the game's own rule broken were Easy a difficulty this game can be cleared on.
- **Which of the bullet's five states is a bullet still coming out and which is one already spent.**
  Which one *hits* is established — state 1 and no other — and what the other four mean is not.
- **What scaled one bullet's step by √2 on one frame of one launch.** It moved `-2.25,5.13` while its
  step said `-1.60,3.66` — `1.41` on both axes, the same number twice, which is a scale and not a
  bullet turning; and since the step a dump prints was itself scaled by the half the model applied to
  state 5 at the time, what the game applied to the raw step was `0.707` against the model's `0.5`.
  What is ruled out: neither `1.41421` nor `0.70711` is a float of the pool at
  `0x14030dfa8`..`0x14030e0c8` or anywhere else in `.rdata`, as an `f32` or an `f64`; no ratio between
  the five states' own multipliers is `1.4`, `0x14000dd8d`'s divisor chain loading `1.0` from
  `0x14030dff0` against `2.0`, `2.5` and `3.0`. **And the state-5 half it was read alongside was a
  factor of two and is fixed** — see the table above — after which five launches in a row have every
  dump reading `0 bullet(s) no step of their own explained` and not one has met this again. So it is
  either a branch nothing has reached since or a reading of a frame the game was part-way through, and
  `steer::OutBy` names the state in the line beside what a bullet moved so that the next occurrence
  says which.
- **What `+0x7500` of the player is for.** It takes the same eight values `+0x7270` does and the switch
  at `0x14004b1a0` turns them into the rotation `+0x73a0` onwards is drawn with.
- **What its `.dat` files hold and how**, and whether `th06.cfg` and `th06.env` are anything like
  1.02h's configuration.
- **Which of the replay list's writes starts a playback at a stage of its choosing**, which is what
  `Game::jump_to_stage` would be.
- **Whether the device read at `0x140441fd0`'s `+0x48` is used as well as `GetKeyboardState`**, and what
  the two flags at `0x140443f60` and `0x1404513a4` turn on.
