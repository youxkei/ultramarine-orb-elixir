# th11, as read off its exe

What is written here was read off `th11.exe` of **md5 `6af1e3b37f28293f4fc127ee01d2632c`**, 688,128 bytes,
`東方地霊殿　～ Subterranean Animism. ver 1.00a` by its own window title (`0x496894`), which is the one build
`game::th11` has addresses for. Every address below is that build's and means nothing about another.

To read it again:

```sh
i686-w64-mingw32-objdump -d -M intel th11.exe > th11.asm        # the whole of .text, once
scripts/decompile.sh <out dir> th11.exe                          # every function decompiled, by Ghidra
i686-w64-mingw32-objdump -d -M intel --start-address=0x445510 --stop-address=0x445e00 th11.exe   # WinMain
i686-w64-mingw32-objdump -d -M intel --start-address=0x446080 --stop-address=0x446ae0 th11.exe   # the three frames, their presents, the time
i686-w64-mingw32-objdump -d -M intel --start-address=0x456b70 --stop-address=0x457100 th11.exe   # the chain
i686-w64-mingw32-objdump -d -M intel --start-address=0x44a340 --stop-address=0x44a8f4 th11.exe   # the sound's commands
i686-w64-mingw32-objdump -d -M intel --start-address=0x42b0e0 --stop-address=0x42b3d0 th11.exe   # the scene switch
i686-w64-mingw32-objdump -d -M intel --start-address=0x42bf30 --stop-address=0x42bfe0 th11.exe   # the pause object's dispatch
i686-w64-mingw32-objdump -s --start-address=0x42bfb4 --stop-address=0x42bfe0 th11.exe            # its two tables
i686-w64-mingw32-objdump -d -M intel --start-address=0x454190 --stop-address=0x454360 th11.exe   # an .anm file's load
i686-w64-mingw32-objdump -d -M intel --start-address=0x4356e0 --stop-address=0x435c50 th11.exe   # the replay at a fresh build
i686-w64-mingw32-objdump -d -M intel --start-address=0x420390 --stop-address=0x420810 th11.exe   # the game object's frames 0 and 30
i686-w64-mingw32-objdump -s --start-address=0x4a3828 --stop-address=0x4a3a28 th11.exe            # the stage records
```

Each claim below was read in the disassembly; Ghidra's decompilation was used to find it, and where a line rests
on the decompilation alone it says so.

The sections are `.text` at `0x401000` (file `0x400`), `.rdata` at `0x48b000` (file `0x8a400`), `.data` at
`0x4a2000` (file `0xa0600`, `0x6400` bytes in the file) and `.rsrc` at `0x4cb000`; the import address table is at
`0x48b000`. The image is loaded at `0x400000` with its relocations stripped (`Characteristics 0x103`). The game's
own strings are CP932, and the exe keeps RTTI, so its classes are named by their vtables (`.?AVLaserLineInf@@` at
`0x4a7a38`).

The imports read below: `Sleep` `0x48b080`, `EnterCriticalSection`/`LeaveCriticalSection` `0x48b084`/`0x48b088`,
`CloseHandle` `0x48b0a0`, `QueryPerformanceCounter` `0x48b0d0`, `CreateThread` `0x48b0ec`, `WaitForSingleObject`
`0x48b0f0`, `CreateEventA` `0x48b0f4`, `RegisterClassA` `0x48b210`, `DestroyWindow` `0x48b228`, `GetWindowRect`
`0x48b230`, `SetForegroundWindow` `0x48b234`, `SendMessageA` `0x48b238`, `GetKeyboardState` `0x48b244`,
`SetWindowLongA` `0x48b248`, `CreateWindowExA` `0x48b250`, `PeekMessageA` `0x48b254`, `DefWindowProcA` `0x48b260`,
`SetWindowPos` `0x48b264`, `ShowWindow` `0x48b268`, `SystemParametersInfoA` `0x48b274`, `DialogBoxParamA`
`0x48b280`, `timeGetTime` `0x48b298`, `joyGetPosEx` `0x48b29c`, `Direct3DCreate9` `0x48b2a4` (through the thunk `jmp
[0x48b2a4]` at 0x45f9b6).

**The calling convention is th10's**: most functions take their object in a register (`eax`, `esi`, `edi`, `ebx`),
said beside each. **A timer** is th10's five words — its previous count, its count, its float, a pointer to the game
speed and its flags, bit 0 its initialised bit — and 0x406100 sets one (`eax` the timer, the count pushed); a timer
below is named by its first word, and its count is the second.

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the constant.

## The window, the device and the frame

WinMain is `0x445510` (`ret 0x10` at 0x445d9e). The mutex `Touhou 11 App` (0x44559d) ends a second copy. The
configuration is `0x3c` bytes at **`0x4c3448`** (`th11.cfg`, version `0x110003` at `+0x0`), read by `0x429eb0`
(called at 0x4455ee) and defaulted by `0x420d80` (the configuration in `esi`). **`0x4c3280` is the supervisor** (see
*The supervisor and its scenes*): its `+0x0` is the `hInstance` (0x4455de), **`0x4c3284`** the `IDirect3D9`
`Direct3DCreate9(0x20)` answered (0x44570e, stored at 0x445713), **`0x4c3288` the device**; and `0x4c3268` holds
the sprite manager.

**The window is `0x4c3d88`**, whose `+0x0` is the HWND: `CreateWindowExA` stores into it (0x446cc3), and
`ShowWindow(…, 0)` (0x445c4c), `MoveWindow` (0x445c5d) and `DestroyWindow` (0x445c6a) are handed it at the teardown,
and `SetForegroundWindow` (0x4457d8). Its copy `0x4c32d8` (0x446cd3) is what the music's thread is created with and
the keyboard's cooperative level is set on.

### The window's making, `0x446ae0`

Called at 0x445737 with `ebx` the `hInstance` (0x445733); plain `ret`, answering `1` where no window was made.

- `RegisterClassA` (0x446b5a), class `BASE` (`0x49688c`), window procedure **`0x445e00`** (0x446b36); it writes
  `[0x4c3d94] = 1` and `[0x4c3d98] = 0` first (0x446b3e, 0x446b48).
- The window's size is bits 2–3 of the flags `0x4c3dc0`, taken from the configuration's window size, the byte
  `0x4c3465` (`+0x1d`, 0x446b60 through 0x446b79): 0 the whole screen, 1, 2 and 3 a window of 640×480, 960×720 and
  1280×960 inside. `[0x4c3394] = (size != 0)` (0x446b8c), which is the present parameters' `Windowed`, below.
- Bit `0x10` of `0x4c3dc0` is set where the frame skip byte `0x4c3466` is nought and the configuration byte
  `0x4c346b` is `2`, and cleared otherwise (0x446b85 through 0x446ba2); it chooses the frame, below. The defaults
  set `0x4c346b` to `2` and `0x4c3466` to `0` (0x420deb, 0x420da4), so **a default configuration sets it.**
- The whole screen: `CreateWindowExA(0, "BASE", title, 0x90000000 /*WS_POPUP|WS_VISIBLE*/, 0, 0, 640, 480, 0, 0,
  hInstance, 0)` at 0x446c28. A window: style `0x100b0000` (`WS_VISIBLE|WS_SYSMENU|WS_MINIMIZEBOX|WS_MAXIMIZEBOX`,
  no caption), at `[0x4c3470]`, `[0x4c3474]` (the configuration's `+0x28`/`+0x2c`, `0x80000000` by default,
  0x420e01), the size plus twice `SM_CXFIXEDFRAME`/`SM_CYFIXEDFRAME` and `SM_CYCAPTION`, at 0x446cb5. Ghidra's
  decompilation has the two calls the other way round; the disassembly is as written here.
- `GetWindowRect` into `0x4c3290` (0x446cc8), then **`SendMessageA(hwnd, WM_SYSCOMMAND, SC_MINIMIZE)`, `Sleep(16)`,
  `SendMessageA(hwnd, WM_SYSCOMMAND, SC_RESTORE)`** (0x446cfa, 0x446cfe, 0x446d16).

### The window procedure, `0x445e00`

`stdcall`, four arguments, `ret 0x10` on every way out.

| message | what it does |
| --- | --- |
| `WM_ACTIVATEAPP` (`0x1c`) | **`[0x4c3d94] = wParam`** and `[0x4c3d98] = (wParam == 0)` (0x445e9e, 0x445ea4), then `DefWindowProcA` |
| `WM_SIZE` (`5`) with `SIZE_MAXIMIZED` | where bit 0 of `0x4c3dc0` is set (WinMain sets it before the loop, 0x445806): bits 2–3 off and bit `0x2`, a change of mode asked, on (0x445e75 through 0x445e7d) — the maximize box goes to the whole screen |
| `WM_CLOSE` (`0x10`) | bit `0x80` of `0x4c3810` on and `0x100` off, answering `1` (0x445e36 through 0x445e47) |
| `WM_ERASEBKGND` (`0x14`) | answers `1` |
| `WM_SETCURSOR` (`0x20`) | the cursor shown where windowed (`[0x4c3394]`) or inactive (`[0x4c3d98]`), hidden otherwise |
| `WM_SYSCOMMAND` `0xf090`, `0xf100` (`SC_KEYMENU`) | swallowed, answering `1` (0x445ee6 through 0x445efb) |
| `WM_LBUTTONDOWN` (`0x201`) | **`SetForegroundWindow(hwnd)`** (0x445ecc), then `DefWindowProcA` |

**`[0x4c3d94]` is the game being the active application**, and the keyboard is read only while it is set (see *The
input*).

### The device's making, `0x446d30`

Called at 0x445750, plain `ret` (0x447269), `0` on success. `GetAdapterDisplayMode` (`IDirect3D9+0x20`, 0x446d5f)
into `0x4c341c` … `0x4c3428` (the width, the height, the refresh at `0x4c3424` and the format at `0x4c3428`). The
present parameters are built at `[esp+0x50]`, laid out as `D3DPRESENT_PARAMETERS`:

| field | value |
| --- | --- |
| `+0x0`/`+0x4` size | 640×480 (0x446f03, 0x446f0b) |
| `+0x8` format | windowed, the display's (0x446ecb); the whole screen, `0x17` R5G6B5 where bit 0 of `[0x4c3480]` is set, else `0x16` X8R8G8B8 or `0x17` by `0x4c3462` (0x446dc7 through 0x446e0d) |
| `+0x18` swap effect | `1`, DISCARD (0x446e7c, 0x446ea6, 0x446ecf) |
| `+0x1c` device window | `0` (the `memset` at 0x446d46) |
| `+0x20` windowed | `1` where windowed (0x446ef6) |
| `+0x24`/`+0x28` depth | on, `0x50` D16 (0x446f13, 0x446f17) |
| `+0x2c` flags | `1`, `D3DPRESENTFLAG_LOCKABLE_BACKBUFFER` (0x446f1f) |
| `+0x30` refresh | the whole screen `60` (0x446e35); `0` on the path through `0x4c37fc` (0x446e9f) |
| `+0x34` interval | `0x80000000` IMMEDIATE where bit `0x10` of `0x4c3dc0` is set or `0x4c346b == 3` (or, windowed, at a refresh other than 60); otherwise `1` ONE (0x446e40 through 0x446e63, 0x446ed3 through 0x446eef) |

A window at a refresh other than 60 takes bit `0x10` off `0x4c3dc0` (0x446dab).

**`CreateDevice` is `IDirect3D9+0x40`**, `CreateDevice(0, D3DDEVTYPE_HAL, [0x4c3d88], flags, &pp, &0x4c3288)`,
tried with `0x40` HARDWARE_VERTEXPROCESSING (0x446f5d), then `0x20` SOFTWARE (0x446f9d), then `D3DDEVTYPE_REF` with
`0x20` (0x446fde); bit 1 of `[0x4c3480]` skips the first two (0x446f30). There is no `D3DCREATE_NOWINDOWCHANGES` and
no `FPU_PRESERVE` in the game's own flags. The parameters are copied to **`0x4c3374`** (`rep movs` of `0xe` dwords,
0x4470ca), which every `Reset` is handed; WinMain's change of mode writes its `+0x8` (`0x4c337c`), `+0x20`
(`0x4c3394`), `+0x30` (`0x4c33a4`) and `+0x34` (`0x4c33a8`) (0x445922 through 0x445986), keeping the old ones at
`0x4c33ac` (0x445907). Then the camera's transforms (`SetTransform`, 0x44717d, 0x447194), `GetViewport` into
`0x4c335c` (0x4471a9), `GetDeviceCaps` into `0x4c381c` (0x4471bb), the render states `0x447270` (0x44724a) and
`[0x4c3d8c] = 0` (0x447257).

The device's vtable as the game reads it, every slot agreeing with `IDirect3DDevice9`: `+0xc` TestCooperativeLevel
(0x445863), `+0x1c` GetDeviceCaps (0x4471bb), `+0x40` Reset (0x4459a6, 0x4463b5, 0x446609, 0x44682c), `+0x44`
Present (0x446395, 0x4465e9, 0x44680c), `+0x4c` GetRasterStatus (0x446300, 0x446346), `+0x5c` CreateTexture (in
`0x4453b0`), `+0xa4` BeginScene (0x446160, 0x44653e, 0x4466fd), `+0xa8` EndScene (0x4461c3, 0x4465a1, 0x446760),
`+0xb0` SetTransform (0x44717d), `+0xbc` SetViewport (0x4460f1, 0x4464cf, 0x44668e), `+0xc0` GetViewport (0x4471a9),
`+0xe4` SetRenderState (0x42a32f, in `0x447270`), `+0x104` SetTexture (0x4461b3, 0x446591, 0x446750), `+0x10c`
SetTextureStageState (0x44fd31), `+0x14c` DrawPrimitiveUP (0x44fd80) and `+0x164` SetFVF (0x44fd5c).

### The message loop

WinMain zeroes the time's floor `0x4c3de0` (0x4457a4), calls `SetForegroundWindow([0x4c3d88])` (0x4457d8), builds
the supervisor's jobs (`0x4298c0`, 0x4457de), sets bit 0 of `0x4c3dc0` and loops from `0x445825` to `0x445b27`:
`PeekMessageA` (0x445833) with `TranslateMessage` and `DispatchMessageA` behind it; and on nothing pending,
`[[0x4c3288]+0xc]` (0x445863), `TestCooperativeLevel`:

- `D3D_OK` and no change of mode asked (bit 1 of `0x4c3dc0`, 0x44586f): **one of three frames**, each pushed
  `0x4c3d88`:
  - **`0x446080`** where bit `0x10` of `0x4c3dc0` is set (0x445874, the call at 0x44587e) — the default;
  - **`0x446650`** where `[0x4c33a8] == 1` (the interval ONE) and the frame skip `0x4c3466` is nought (the call at
    0x44589b);
  - **`0x446420`** otherwise (the call at 0x4458a7) — the one shaped as th10's.

  An answer of `0` takes bit `0x10` off `0x4c3810` (0x4458b8) and loops; any other leaves the loop (0x4458b2).
- `0x88760869` `D3DERR_DEVICENOTRESET` (0x4458c4), or `D3D_OK` with a change of mode asked: `[0x4c3dc4] = 10`
  (0x4458cf), the parameters for the new mode, `0x42b600`, `0x445320` (which releases the render-target textures of
  the thirty-two `.anm` files from the sprite manager's `+0x4350c0`, see *The sprite manager*), `Reset(0x4c3374)`
  (0x4459a6); and on success `0x447270` (the render states), `0x4453b0` (the textures made again), bit `0x10` of
  `0x4c3810` on, **`[0x4c37f4] = 3`** (0x4459d4), the window's style and place for the mode (`SetWindowLongA`
  0x445a5c/0x445abc, `SetWindowPos` 0x445a7c/0x445ada, and `ShowWindow(…, SW_SHOWNORMAL)` at 0x445a8b for a window),
  `0x42b530(0x4c3280)` (0x445b13), and bit 1 off. `Reset` answering `D3DERR_DEVICELOST` loops; any other failure
  leaves.
- The loop also leaves where `[0x4c3d8c]` is not nought (0x445817, 0x445b21); its only store found is the `0` at
  0x447257.

After the loop: the window's place saved (0x445b4d), **`0x429650` with the supervisor in `ebx`** (0x445b6e), and the
chain torn down (`0x4451b0`, 0x445b7f). A frame's answer `2` (`[esp+0xc] == 2`, 0x445c82) builds everything again
from 0x44565a; `1` ends the process. `0x429650` drains the sound's commands (`0x44a340` until it answers nought,
0x429653), writes `[0x4c90a4] = 2`, and stops the loading thread (`0x459430` with `esi = 0x4c3a70`, 0x42966b);
`0x4451b0` stops it again (0x4451e3). The sound's two threads are waited out in `0x449050` (0x445ba4) after
`[0x4c90a4] = 2` (0x445b9e).

**Every frame is `stdcall`**: its one argument on the stack (`mov ebp,[esp+0xc]` after two pushes, at 0x446089,
0x446422 and 0x446652) and every way out `ret 0x4` (0x44611f, 0x44613a, 0x4461e3; 0x4464fd, 0x446518, 0x4465bf;
0x4466bc, 0x4466d7, 0x446780). **The chain's walks are not**: see *The chain*.

### The time, `0x446920`

No arguments, plain `ret`, the answer on the FPU stack: seconds as a double, `QueryPerformanceCounter` (0x446952)
less the start `0x4c3da8` over the frequency `0x4c3da0` (both taken in `0x445fb0`, 0x446001 and 0x44600c), or
`timeGetTime` over 1000 where the frequency is nought (0x4469c0); either way less `0x4c3de0`, the smallest value seen,
so it never steps back (0x44698e, 0x4469e9). It runs under the critical section `0x4c3b08` while bit `0x8000` of
`0x4c3810` stands.

### The frame at `0x446420`, in order

The window's `+0x40` is this frame's time, `+0x48` the time before, `+0x50` the moment the next frame is due and
`+0x14` the skip counter.

- the time into `[this+0x40]` (0x44642d); where it is below `[this+0x48]`, `[this+0x50]` put to it (0x44643a); the
  time into `[this+0x48]` (0x44643d);
- **the limiter**: where `([this+0x50] − time) × 1000 ≥ 1.5` (`0x498400`), `Sleep(1)` (0x446458); where the time is
  not past `[this+0x50]`, the frame answers `0` (0x446469 → 0x4465b9); otherwise `[this+0x50]` steps on by `1/60`
  (`0x497f68`) until it is not below the time (0x44647c through 0x446495);
- `0x44fd10` with `esi = [0x4c3268]` — the queue drawn (0x44649f);
- `0x42a810` with `edi = 0x4c359c`, stored first into `[0x4c37cc]` (0x4464a9), which sets the camera's two
  transforms, and `SetViewport([0x4c37cc] + 0xcc)` (0x4464cf);
- `[0x4c37d0] = 1` (0x4464d7);
- **the calc chain**: `ebx = [0x4c3234]`, `call 0x456cb0` (0x4464d1, 0x4464e1). **The sound is not called here**:
  its commands are carried out inside this walk, by the supervisor's calc job (see *The sound*);
- the walk's answer: `0` → `0x459430` with `esi = 0x4c3a70` (the loading thread stopped) and the frame answers `1`
  (0x4464ec through 0x4464fd); `-1` → the same and `2` (0x446505 through 0x446518); anything else carries on;
- **the frame skip**: the byte `[this+0x14]` counted up and held against `[0x4c3466] + 1` (0x44651b through
  0x44652e), a frame short of it jumping to the time's store at 0x4465ab;
- `BeginScene` (0x44653e);
- the queue's count `[sm+0x435620]` to nought and its two cursors `[sm+0x7b5624]` and `[sm+0x7b5628]` to
  `sm+0x435624` (0x446540 through 0x446557), `[0x4c3c10] = 0xff` (0x446562), and `0x42a300` with `edi = 0x4c3280`
  (0x44656c), which where `[0x4c3280+0x990]` — that dword — is not nought draws the queue, puts the dword at nought
  and sets render state `0x1c`, the fog, to 0 (0x42a300 through 0x42a32f); the frame's own store making it do all
  three every frame;
- **the draw chain**: `call 0x456e10` (0x446571), with no argument;
- the queue drawn again, `0x44fd10` with `esi = [0x4c3268]` (0x44657c);
- `SetTexture(0, null)` (0x446591) and `EndScene` (0x4465a1);
- `[this+0x14] = 0` (0x4465a3) and **the present, `0x4465d0`** (0x4465a6);
- the time less `[this+0x40]` into the double `0x4c3c38` (0x4465b3), and `0`. Nothing reads `0x4c3c38` by its
  address.

**The present at `0x4465d0` takes nothing and pops nothing** (`push ecx` … `pop ecx; ret` at 0x446645): `0x446880`
first (0x4465d1) — `0x445460`, which copies the back buffer into four of the sprite manager's capture records (see
*The sprite manager*), then the screenshot, `snapshot/th%.3d.bmp` (`0x496848`), the first of a thousand names not yet
taken, where bit `0x40000` of `[0x4c92b4]` is down (0x44689a); then `Present(0, 0, 0, 0)` (0x4465e9); where that
fails, `0x42b600`, `0x445320`, `Reset(0x4c3374)` (0x446609), `0x4453b0`, `0x42b530(0x4c3280)` and `0x447270` —
whatever `Reset` answers — and `[0x4c37f4] = 2` (0x44661f); then `0x419d90` where `[0x4a8d80]` (the frames-per-second
count, 0x446632) and `0x40c310` where `[0x4a8d6c]` (0x446640).

### The frame at `0x446080`, the default, and its present `0x4461f0`

The same steps from the queue drawn on (0x4460bb through 0x4461e3: the calc walk at 0x446103, the draw walk at
0x446193, the skip at 0x446150, and the present `0x4461f0` at 0x4461ca with **`esi` the window**, `mov esi,ebp` at
0x4461c5). **The limiter is gone**: only where bit `0x10` of `0x4c3dc0` stands and `[this+0x40]` is past
`[this+0x50]` is `[this+0x50]` stepped on by `1/60` (0x446080 through 0x4460b9); `[this+0x40]` here is the time the
present took at its start (0x4461fa). The pacing is the present's:

- `[esi+0x40]` = the time; `[esi+0x74]` = `((the last present [esi+0x60] + 1/60) − time) × 1000 − 2.5` truncated
  (0x4461fd through 0x446226);
- a sleep length per mode at `[esi+0x80 + [esi+0x78] × 0xc]`, with its ceiling at `+0x7c` and a count at `+0x84`
  (`0x446ae0` fills four modes, `15`, `12`, `12` and `8`, at `0x4c3e04` … `0x4c3e30`, and puts the index `0x4c3e00`
  = `[0x4c3d88+0x78]` at nought): lowered to `[esi+0x74]` where it is above it, otherwise raised by one after
  fifteen frames, not past the ceiling (0x446229 through 0x446274);
- where less than `1/57` s (`0x498410`) has passed since `[esi+0x68]`: `Sleep(1)` until 13 ms (`0x498408`) have
  (0x4462d1), then **`GetRasterStatus` polled** until in the vertical blank, until the scan line steps back, or until
  `1/57` s has passed (0x4462fd through 0x44634e); where `1/57` s passes, the sleep length one lower (0x446361
  through 0x446373);
- `[esi+0x68]` = the time (0x44637a); `0x446880` (0x44637d); `Present(0,0,0,0)` (0x446395) — the interval is
  IMMEDIATE in this mode; on a failure the reset steps of `0x4465d0` and `[0x4c37f4] = 2` (0x44639b through
  0x4463cb);
- `0x419d90` where `[0x4a8d80]` (0x4463de), `0x40c310` where `[0x4a8d6c]` (0x4463ec);
- `Sleep` of the sleep length where it is above nought (0x44640a), `[esi+0x60]` = the time (0x446411), plain `ret`
  (0x446419).

### The frame at `0x446650` and its present `0x446790`

No limiter at all (0x446650 through 0x446780: the calc walk at 0x4466a0, the draw walk at 0x446730, the skip at
0x4466ed, the present `0x446790` at 0x446767 with `esi` the window, 0x446762): it runs where the interval is ONE,
so `Present` waits for the blank. `0x446790`: where `[0x4c346b] == 1` and less than `1/60` has passed since
`[esi+0x60]`, `Sleep` until 3.5 ms (`0x4983f8`) before the next `1/60` mark (0x446799 through 0x4467de); then
`[esi+0x68]` = the time, `0x446880`, `Present` (0x44680c), the reset steps on a failure, `[esi+0x60]` = the time
(0x446851), `0x419d90` and `0x40c310`; `push ecx` … `pop ecx; ret`.

**Against th10**: three frames, chosen every frame, where th10 has one; **the default configuration runs
`0x446080`**, which paces in its present (`Sleep(1)` loops, `GetRasterStatus` polled, an adaptive `Sleep` after an
IMMEDIATE `Present`). Only `0x446420` has th10's limiter, and it sleeps 1 ms where 1.5 ms or more remain where th10
went out through `Sleep(0)`. The window's due moment is at `+0x50` (th10 `+0x48`, the frame's start `+0x38`). The
present calls the capture and the screenshot `0x446880` **before** `Present` (th10 after), and the frames-per-second
count `0x419d90` and `0x40c310` after it. The reset releases the textures and **makes them again** through `0x4453b0`,
with `0x42b600` before and `0x42b530(0x4c3280)` after.

### The queue of quads the drawing fills

**`0x4c3268`** holds the sprite manager, a `0x7bd894`-byte allocation WinMain takes at 0x445762 and builds with
`0x4526f0` (pushed, 0x445771). **`0x44fd10`**, entered with the manager in `esi`, plain `ret`, draws what is queued
in it: nothing where `[esi+0x435620]` — the count — is nought (0x44fd10); otherwise `SetTextureStageState(0, 6, 0)`,
`SetTextureStageState(0, 3, 0)`, `SetFVF(0x144)`, `DrawPrimitiveUP(4, count × 2, [esi+0x7b5628], 0x1c)`, the
drawn-from cursor `[esi+0x7b5628]` up to the write cursor `[esi+0x7b5624]`, `[esi+0xac]` one higher and the count
to nought (0x44fd1d through 0x44fd94). th10's count was at `+0x3adac8` and its counter at `+0x58`.

### SetForegroundWindow, SystemParametersInfoA, ShowWindow and DialogBoxParamA

- `SetForegroundWindow`: WinMain before the loop (0x4457d8); **the window procedure on `WM_LBUTTONDOWN`
  (0x445ecc)**; `0x4476e0` (`push eax; call [0x48b234]; ret`, 0x4476e1); and the thunk `0x45fba2`. No call to either
  of the last two was found. All four go through the one import entry `0x48b234`.
- `SystemParametersInfoA`: `0x445fb0` (called at 0x4455e4) gets `0x10`, `0x53` and `0x54` into `0x4c3db4`,
  `0x4c3db8` and `0x4c3dbc` and sets `0x11`, `0x55` and `0x56` to nought (0x445fc2 through 0x445ffa) — the screen
  saver and the display's power settings; the teardown sets them back (0x445d51, 0x445d5e, 0x445d6c). `0x446020` does
  the same, and no call to it was found.
- `ShowWindow`: the teardown's hide (0x445c4c), and `SW_SHOWNORMAL` after a change of mode to a window (0x445a8b).
- `DialogBoxParamA(hInstance, 0xcb, 0, 0x447910, 0)` (0x445626) runs where bit `0x100` of `[0x4c3480]` — the
  configuration's `+0x38` — is set or the Shift key is down (`GetKeyboardState`, byte `0x10`, 0x445612). The
  defaults set the bit (0x420d8c); the dialog's procedure sets or clears it (0x447961, 0x44796d), and writes the
  window size (0x447984 through 0x4479c8).

## The chain

**`0x4c3234`** holds the chain, a `0x4c`-byte allocation WinMain takes at 0x44565c and builds inline (0x445668
through 0x44569e; `0x456b30`, entered with it in `eax`, is the same build as a function): two `0x24`-byte head jobs
at `+0x0` and `+0x24`, the calc list's head element at `+0x14` whose `+0x0` points back at the chain (0x44567e), the
draw list's at `+0x38` (0x445695), and **`+0x48`, a dword the calc walk reads** (put at nought at 0x44569e). A list
element is three words — the job at `+0x0`, the next element at `+0x4`, the one before it at `+0x8` — and a walk
starts from `[chain+0x18]` for the calc list (0x456ce0) and `[chain+0x3c]` for the draw list (0x456e3e).

**`0x456cb0` walks the calc list and `0x456e10` the draw list.**

- **`0x456cb0` takes the chain in `ebx`** (`mov eax,[ebx+0x18]` at 0x456ce0, `ebx` set by the frames at 0x4460f3,
  0x4464d1 and 0x446690) and ends in a plain `ret` (0x456deb); nothing is on the stack. This is what
  `game::FrameEntered::ChainInEbx` is for.
- **`0x456e10` takes nothing**: it reads `[0x4c3234]` itself (0x456e25) and ends in a plain `ret` (0x456f12).
- Each runs under the critical section `0x4c3a90` with the byte `0x4c3bb0` counting its depth, while bit `0x8000`
  of `0x4c3810` stands (WinMain sets it at 0x44555b and takes it off at the teardown, 0x445d1c), and leaves it around
  every callback it runs (0x456d27 and 0x456d4e, 0x456e70 and 0x456e97).
- A callback is `__thiscall` on the job's `+0x20` (`mov ecx,[esi+0x20]; mov edx,[esi+0x8]; call edx`, 0x456d38).
- **Where `[chain+0x48]` is not nought, the calc walk runs no job's `+0x8`**: it runs each job's `+0x10` where there
  is one, and counts the job (0x456d15 → 0x456d90). The teardowns set it to `1` before a last calc walk (0x4451e8
  then 0x4451ef; 0x456fae then 0x456fb5), so `+0x10` is each job's leaving callback.

**A job is `0x24` bytes**, made by **`0x456fd0`** (`stdcall`, the callback pushed, `ret 0x4` at 0x457009): bit 0 of
its flags set (0x456ffc) and the callback at `+0x8` (0x457000). Most callers build theirs inline (`push 0x24; call
0x45fce4`, then the fields; the supervisor's at 0x4298cb through 0x429924).

| | |
| --- | --- |
| `+0x0` | its priority; the insertion walks past every job whose priority is below it (`cmp [edx],ebx; jge`, 0x456bbc) |
| `+0x4` | its flags: bit 0 says the job is freed as it is taken off a list (0x4570d4 through 0x4570e7); bit 1 is set while it is to be run (0x456d0b, 0x456e5e) |
| `+0x8` | the callback, `__thiscall` on `+0x20` |
| `+0xc` | a callback run once as the job is added, on `+0x20`, and cleared; its answer is the add's (0x456b70 through 0x456b88) |
| `+0x10` | a callback the calc walk runs where the job answers `7` or where `[chain+0x48]` is set (0x456d90) |
| `+0x14` | the job's own element of the list it is on |
| `+0x20` | the argument its callbacks are called on |

**`0x456b70` adds a job to the calc list and `0x456c10` to the draw list**: the job in `esi`, its priority in
**`ebx`**, the chain read from `[0x4c3234]` (0x456b77, 0x456c17), plain `ret` (0x456c02, 0x456ca2). **`0x457080` takes
a job off**: `__fastcall`, the job in `ecx` and the chain in `edx`, plain `ret` (0x4570ae, 0x4570f1); `0x457030` does
the same under the critical section, the job in `esi` and the chain pushed (`ret 0x4` at 0x457079).

**What a callback answers is read through a table**, `0x456dec` for the calc walk (`cmp edi,8; ja` at 0x456d5b) and
`0x456f14` for the draw (`cmp edi,5; ja` at 0x456ea4):

| answer | the calc walk | the draw walk |
| --- | --- | --- |
| `0` | the job taken off (0x456d7e), counted, and on | the job taken off (0x456ebe), counted, and on |
| `1` | counted, and on to the next job (0x456d9c) | the same (0x456ec3) |
| `2` | the job run again while bit 1 of its flags stands, otherwise counted (0x456d67) | the same (0x456eb0) |
| `3` | the walk stops and answers `1` (0x456db3) | the walk stops and answers `1` (0x456ed9) |
| `4` | the walk stops and answers `0` (0x456da9) | the walk stops and answers `0` (0x456ecf) |
| `5` | the walk stops and answers `-1` (0x456dbd) | the walk stops and answers `-1` (0x456ee3) |
| `6` | the walk starts again from the first job, its count at nought (0x456ce0) | — |
| `7` | `+0x10` run, counted, and on (0x456d90) | — |
| `8` | the walk stops and answers `0` (0x456da9) | — |

An answer past the table — above `8` for the calc walk, `5` for the draw — counts the job and goes on (0x456d5e,
0x456ea7). **And otherwise a walk answers the count of the jobs it reached** (`inc [esp+0xc]` at 0x456d9c), so an
empty list answers `0`. A job whose `+0x8` is null is passed over and not counted (0x456cfa, 0x456e55); one whose bit
1 is clear is counted without being run (0x456d0f, 0x456e62).

**The supervisor's jobs** (`0x4298c0`, called from WinMain at 0x4457de), every one on `0x4c3280`: the calc job
**`0x429030` at priority 1** (`lea ebx,[ebp-0x2]` with `ebp = 3`, 0x42990e; its `+0xc` is `0x429560`, 0x429924),
the lowest calc priority among the 35 callers of `0x456b70`; the draw jobs `0x4290e0` at 1, `0x429220` at `0xb`,
`0x4293d0` at `0xd`, `0x4292b0` at `0x23`, `0x429420` at `0x25`, `0x429340` at `0x2e`, `0x429470` at `0x2f` and
`0x4293a0` at `0x44` (0x42997b through 0x429b96).

**Against th10**: th10's two walks were `stdcall` on the chain pushed, `ret 0x4`; here **the calc walk takes the
chain in `ebx` and the draw walk takes nothing**, both ending in a plain `ret`. The adds take the priority in `ebx`
(th10 `edi`) and the chain from the global (th10 pushed it); the take-off is `__fastcall`. The calc walk's table has
a ninth entry, **`8`, which stops it answering `0`**; answer `0` counts the job in both walks; and `[chain+0x48]` is
new. The chain is `0x4c` bytes (th10 `0x48`).

## The sound

**`0x4c3e80` is the sound manager**, `0x52f4` bytes that `0x448ff0` zeroes (`stdcall`, the HWND pushed, called at
0x44574b) before `CreateThread(…, 0x449100, 0x4c3e80, 0, &0x4c90a0)` (0x449022), the handle into `0x4c9098` and the
HWND into `0x4c90a8`. `[this+0x610]` (`0x4c4490`) is the DirectSound object (`DirectSoundCreate8`, ordinal 11, in
`0x449140`), `[this+0x5208]` (`0x4c9088`) the music's stream, `[this+0x1f84]` (`0x4c5e04`) `thbgm.fmt`,
`[this+0x1f88]` (`0x4c5e08`) the queue of commands and `[this+0x4108]` (`0x4c7f88`) the two songs' slots.

**There is no call on the manager in the frame.** **`0x44a340` carries out the commands**: no argument (it addresses
the manager absolutely), plain `ret` (0x44a38d, 0x44a8d3), under the critical section `0x4c3b98` (its depth byte
`0x4c3bbb`). It answers nought at once where `[0x4c4490]` — the DirectSound object — is nothing (0x44a365); otherwise
it dispatches the command at `0x4c5e08` on the table `0x44a8d4` (the command less one, eight entries, 0x44a3a6), then
services the sound effects, and **answers `[0x4c5e08]`, the command left at the head** (0x44a8c7). Its callers are
the supervisor's calc job (0x429058) and the teardown's drain (0x429653). **So the sound runs once per calc walk that
reaches the supervisor's priority-1 job, inside the walk and before the input is read** — where th10's frame called
its sound manager, `0x43ddf0(0x492590)`, between the calc walk and reading its answer.

**A command is `0x10c` bytes**, thirty-one at most: `+0x0` its number, `+0x4` a slot or a number, `+0x8` its step and
`+0xc` a name. **`0x44a9c0` queues one**: the manager in `eax`, the name in `edi`, `stdcall` on the number and the
slot (`[esp+0x8]`, `[esp+0xc]` after `push esi`), `ret 0x8` (0x44aa54). A command carried out is shifted off the head
(0x44a622 through 0x44a651); **only command 1 lets the next one run in the same call** (`[esp+0x10] = 1` at
0x44a402, read at 0x44a655); every other step of every other command takes a call of its own, its step counted up at
0x44a744.

| command | handler | what it does |
| --- | --- | --- |
| 1 | 0x44a3ca | `0x449c70` (the manager in `ebx`, `stdcall` on the slot and the name, `ret 0x8`): the name copied into the slot (0x449cc1 through 0x449cd8) unless the slot already holds it; where bit `0x10` of `[0x4c3480]` is set, the track also read into memory from `thbgm.dat` (0x449d3f through 0x449e63) |
| 2 | 0x44a40f | the play, by steps. Streaming (bit `0x10` of `[0x4c3480]` clear, or the slot negative): 0 `0x45ab70(0)`, 1 `0x45a4f0`, **2 the record found and the stream's source put on it** (0x44a690), 3 the stream started (`0x45b100`, `0x45a6f0`), 4 `0x45aa60`, and off the queue at 7 (0x44a517). From memory: 0 `0x449e80`, 2, 5, 7, and off at `0x14` |
| 3 | 0x44a584 | 0 `0x45ab70(1)`, off at 1 |
| 4 | 0x44a525 | the stream closed: 0 `0x45ab70(1)`, 1 `PostThreadMessageA([0x4c4494], WM_QUIT)` (0x44a559), 2 waits for `[0x4c4498]`, 3 the handles closed and the stream deleted (0x44a706 through 0x44a735), off at 10 |
| 5 | 0x44a5a7 | the fade: `[stream+0x1c] = 1`, and `[stream+0x14]`, `[+0x18]` the number times `0x497e98` |
| 6, 7 | 0x44a5e7, 0x44a605 | where `[0x4c3463] == 1`: `0x45ac00`, `0x45ac80` |
| 8 | 0x44a3ad | `0x45aaf0([0x4c9168])` |

**The thread WinMain starts on it is the sound's startup, `0x449100`**: `0x449140` on the manager (`ecx`, the HWND
in `ebx`), which makes the DirectSound object and its primary buffer and `SetTimer(hwnd, 0, 250)`, then waits for
`[0x4c90a4]` to be set and writes `[0x4c90ac] = 1` (0x449130). **A second thread, `0x449480`** — created at 0x4295ad
by the supervisor calc job's add-callback `0x429560`, and by `0x4494f0`, its handle into `0x4c909c` — loads the
forty-six (`0x2e`) effect files named from `0x4a36b0` (`se_plst00.wav` first) through `0x458400` into `0x4c90b0`
(0x449483 through 0x4494ab), touching no input. **The music's own thread is `0x44a900`**, made where a stream is
opened (`0x449b30`, the manager in `eax`, plain `ret`; called from the loading thread `0x4289e0` at 0x428af1): created
at 0x449bac with **`[0x4c32d8]`, the HWND, as its argument**, its id into `[this+0x614]` (`lea eax,[esi+0x614]`
pushed at 0x449b9f), its handle into `[this+0x618]` (0x449bba) and the event it waits on into `[this+0x520c]`
(0x449b8d). It waits with `MsgWaitForMultipleObjects` on `0x4c908c`, refills through `0x45aee0` (0x44a900 on), and
ends on `WM_QUIT`.

### Which track is playing, by name

**`[this+0x1f84]` is `thbgm.fmt`**, records of `0x34` bytes: the loading thread `0x4289e0` loads
`../../bgm/thbgm.fmt` (`0x494b60`) through `0x458400` (0x428aae) and stores it at `0x4c5e04` (0x428ab3), which is
`0x4c3e80+0x1f84` — the store `0x449b00` makes at 0x449b12, inlined; `0x449b00` itself has no caller. A record starts
with the track's file name, 16 bytes compared to its terminating zero; `+0x10` is where the track is in `thbgm.dat`
and `+0x14` its length (`SetFilePointer` to `+0x10` at 0x449db5, `+0x14` bytes allocated and read at 0x449dc6 and
0x449e18). A record whose first byte is zero ends the table (0x44971a, 0x449762). The stream's open reads the first
record's `+0x24` and `+0x2c` (0x449b6e, 0x449b72).

**`0x4496b0` finds a record by name**, the path in `ecx` and the manager on the stack (`[esp+0x90]` at 0x44970d),
`ret 0x4` (0x449784): it cuts the path past its last `/` (`strrchr`, `0x460560`, at 0x4496cd) or, where there is
none, past its last `\` (0x4496dc), copies what is left to the stack, and compares it with each record's start, two
bytes a round to the terminating zero (0x449725 through 0x44973f). It answers the record's index, and **0 where none
matches** (0x449762 through 0x449768). Three calls: 0x449c2f, 0x449d9b (the preload into memory) and 0x44a6ab
(command 2's step 2).

**Command 2's step 2** (0x44a690 through 0x44a6c3) takes the slot's path (`0x4c7f88 + slot × 0x100`), or the
command's own name where the slot is negative, finds its record, and calls **`0x45b2d0`** with `ecx` the stream's
source `[[0x4c9088]+0xc]`, **`eax` the record's address** (`index × 0x34 + [0x4c5e04]`) and a nought pushed (`ret
0x4`), which writes **the record's address into `[source+0x90]`** (0x45b31d) — only where `[source+0x7c]` is nought,
answering `E_FAIL` otherwise (0x45b2d4 through 0x45b2e3), and where `[source+0x8c]` is `-1` after trying `0x45b220`
once (0x45b2e6 through 0x45b31a). It logs the record's name, `+0x18` and `+0x1c` with `%s %d %d` (`0x497c8c`,
0x45b331). th10's source was handed the record's index and not its address.

**The manager holds the two songs a stage loads, as paths**, `0x100` bytes apiece from `[this+0x4108]` = `0x4c7f88`:

- **`0x42a0d0`** (`stdcall`, the slot and the path, `ret 0x8` at 0x42a148) copies the path, writes `wav` over what
  follows its last `.` (0x42a102 through 0x42a11a) and queues command 1 with the slot (0x42a129).
- **`0x42a150`** (`stdcall`, the slot and a number, `ret 0x8` at 0x42a198): where bit `0x10` of `[0x4c3480]` is set,
  command 4 first (0x42a168); then command 2 with the slot (0x42a17e), and the byte `[[0x4a8ebc] + number + 0x2ddfa]`
  set — the score file's song heard (0x42a18d).
- **`0x42a1c0`** (`stdcall`, a path, `ret 0x4`) plays a path straight: command 2 with the slot `-1` and the path as
  `.wav` (0x42a221), and the same byte set with `esi` as the number (0x42a219).
- `0x42a240` (no argument, plain `ret`) queues command 4 or 3 by bit `0x10` of `[0x4c3480]`; `0x42a270` (`stdcall`, a
  float) queues command 5.

**A stage's two songs are its record's** `+0x10` and `+0x14` (see *The stage records*). The stage's build, on the
loading thread, unless the attract demo (`[0x4a5758] & 0x20`, 0x41fd3c): `0x42a240` (0x41fd45), `+0x10` into slot 0
(0x41fd57), `+0x14` into slot 1 (0x41fd68), and then **`Sleep(16)` until the queue `[0x4c5e08]` is empty** (0x41fd87
through 0x41fd9e). The stage begun (`0x420390`, the game object's calc job) plays slot 0 with the record's `+0x34`
(`0x42a150(0, [rec+0x34])`, 0x420525, 0x42052b; and 0x42053d plus `0x420ba0` — command 6 — where `[0x4a5728] == 6`),
again unless the demo (0x42050d); and `0x41d380`, the conversation's calc, plays slot 1 with `+0x38` (0x41e9cf through
0x41e9dd) — **so slot 1 is the fight's song.** The title loads `bgm/th11_00.wav` (`0x495f34`) into slot 0 and plays it
with number 0 (0x4404a6 through 0x4404b6, and 0x441324 through 0x441334). The game over's and the cleared run's
openers queue `bgm/th10_17.wav` (`0x495394`, pushed at 0x42d630), th10's name, which th11 holds as well as its own
`bgm/th11_17.wav` (`0x495a90`); so does the ranking after the ending (0x43f6b0).

**Against th10**: a command now takes several calls — a play puts the source on its record at its third call and
leaves the queue at its eighth — and the stage's build waits for the queue to empty. Forty-six sound effects where
th10 has thirty-seven, loaded by a thread the supervisor's calc job's add-callback starts.

## The input

**`0x4576b0` is the read**, called once a walk from the supervisor's calc job `0x429030` (`call` at 0x429062). That job
is added at priority 1 — `mov [eax+0x8],0x429030` at 0x429913, `call 0x456b70` at 0x42992b, the add putting the
priority in from `ebx` (`mov [esi],ebx` at 0x456baf) and calling the job's add-callback, here 0x429560, with `ecx =
[job+0x20]` (0x456b81). **The read takes no argument** — nothing in `ecx`, nothing on the stack — and ends in a plain
`ret` (0x457a27) with the word in `eax` (0x457a17). It stores into one fixed input at `0x4c92a8`: the frame before's
word into `+0x4` (`0x4c92ac`) and this frame's into `+0x0` (0x4579f2 through 0x457a05), **both `u32`**, and calls
0x459b50 with `ecx = 0x4c92a8` (0x457a00, 0x457a0b).

**0x459b50 works out the rest** (`ecx` the input, plain `ret` at 0x459bb8, `esi` and `edi` kept): the buttons gone
down, `cur & (cur ^ prev)`, into `+0xc` (`0x4c92b4`) and those let go, `~cur & (cur ^ prev)`, into `+0x10`
(`0x4c92b8`) (0x459ba1 through 0x459bb4); **an auto-repeat word into `+0x8`** (`0x4c92b0`) off thirty-two `u32`
counters from `+0x14` (`0x4c92bc`), one a bit (`edi = 0x20` at 0x459b6d): a counter counts the frames its bit is held
and is put at nought where it is not (0x459b8f), and a bit held fires in `+0x8` on its 26th frame (`cmp [eax],0x1a` at
0x459b82) and every eighth frame after, the counter taken back by eight each time it fires (0x459b8a). And **a word of
its own at `+0x12c`** (`0x4c93d4`), each bit set on every frame from its counter's eighth on (`cmp [eax],0x8` and `or
[ecx+0x12c],esi`, 0x459b77, 0x459b7c).

The input is `0x148` bytes and there is an array of them: 0x457a30 (`ecx` an index, `esi` a word; `imul
ecx,ecx,0x148; add ecx,0x4c92a8`) and 0x457a50 (the index on the stack, `ret 0x4` at 0x457db0) read into entry *n*,
and the static initialiser below writes a second entry's mapping at `0x4c9524` (`0x4c93f0 + 0x134`, 0x48ac4d). Neither
0x457a30, 0x457a50 nor a third read, 0x457dc0 (which answers the word without storing it, plain `ret` at 0x4580f8), has
a caller: no `call` to them and no pointer to them anywhere in the file.

The layout of `0x4c92a8`, from 0x459b50, 0x40de10 and 0x459bd0:

| offset | address | what |
| --- | --- | --- |
| `+0x0` `+0x4` | `0x4c92a8` `0x4c92ac` | the live word, the frame before's |
| `+0x8` `+0xc` `+0x10` | `0x4c92b0` `0x4c92b4` `0x4c92b8` | its repeat, gone down, let go |
| `+0x14` | `0x4c92bc` | 32 `u32` counters of the live word |
| `+0x94` | `0x4c933c` | 32 `u32` counters of the run's copy |
| `+0x114` | `0x4c93bc` | the auto-focus count (below) |
| `+0x118` `+0x11c` | `0x4c93c0` `0x4c93c4` | the run's copy, the frame before's |
| `+0x120` `+0x124` `+0x128` | `0x4c93c8` `0x4c93cc` `0x4c93d0` | the copy's repeat, gone down, let go |
| `+0x12c` `+0x130` | `0x4c93d4` `0x4c93d8` | the held-eight-frames words, the live one's and the copy's |
| `+0x134` | `0x4c93dc` | nine `i16`, the pad's mapping |

**The menus move their cursors on `[+0xc] | [+0x8]`**, so a direction held repeats — the pause menu tests
`[0x4c92b4] & 0x10` and then `[0x4c92b0] & 0x10` (0x42c9fb, 0x42ca04), and the same for `0x20` (0x42ca16, 0x42ca1f).

**The keyboard is read only while the game is the active application**: the read skips straight to the pad's half
where `[0x4c3d94]` is nought (`je 0x4579eb` at 0x4576cd), which the window procedure sets to `WM_ACTIVATEAPP`'s
`wParam` (`cmp esi,0x1c` and `je` at 0x445e0f and 0x445e18, the store at 0x445e9e) and the window's making starts at 1
(0x446b3e); so does `0x458100` (0x458110), and code at 0x457a7c and 0x457dd7.

**What it stores is what `0x457270` answers**, the pad's half: `mov ecx,esi; call 0x457270` at 0x4579eb, the
keyboard's word in `ecx`, nothing pushed, the answer in `eax`, and a plain `ret` (0x4573fe, 0x457475, 0x45759b; it has
a `push ebp; mov ebp,esp; and esp,-8` frame) — what `game::Read::PadHalfInEcx` is for. Its other two callers
(0x457d83, 0x4580de) are in the reads nothing calls.

The keyboard comes through `IDirectInputDevice8::GetDeviceState(0x100)` on `[0x4c32a0]` where bit `0x400` of
`[0x4c3810]` is set (the `test` at 0x4576d3, the call at 0x457862) and through `GetKeyboardState` otherwise
(0x4576e8); the pad through DirectInput's `[0x4c32a4]` where bit `0x800` is and `joyGetPosEx` otherwise (0x45728a). A
read of the keyboard that answers `DIERR_INPUTLOST` or anything but nought is a word of nought and one `Acquire`
(`[vtbl+0x1c]`, 0x457864 through 0x45787e).

**The input's set-up runs on the main thread, in WinMain**, before `Direct3DCreate9`: it probes the joystick with
0x4571a0 (0x4456aa) — `joyGetPosEx(0)`, then `(1)`, and where either answers, `joyGetDevCapsA(0, 0x4c2f08, 0x194)` —
clears the keyboard state's high bits with 0x458170 (0x4456af), takes bits `0xc00` off `[0x4c3810]` (0x4456b4), makes
the devices in 0x447ab0 with `eax = 0x4c3280` (0x4456be, 0x4456c3), and sets `0x400` where `[0x4c32a0]` is there and
`0x800` where `[0x4c32a4]` is (0x4456c8 through 0x445702). 0x447ab0 (`eax` the object, plain `ret`, `-1` or 0 in
`eax`) makes no device only where bit `0x8` of `[0x4c3480]` — `th11.cfg`'s `+0x38` — is set (`test [esi+0x200],0x8`
at 0x447ab3), and the defaults 0x420d80 set only `0x100` there (0x420d8c). Otherwise it calls
`DirectInput8Create([0x4c3d90], 0x800, 0x48caec, 0x4c328c, 0)` (0x447ad8), `CreateDevice(0x48cc5c, 0x4c32a0)`
(0x447b13) and `SetDataFormat(0x48b55c)` (0x447b41), sets the keyboard's cooperative level `0x16`,
`DISCL_NONEXCLUSIVE | DISCL_FOREGROUND | DISCL_NOWINKEY`, on the window `[0x4c32d8]` (0x447b7b through 0x447b82), and
`Acquire`s it (0x447bc2); then `EnumDevices(4, 0x447cb0, 0, 1)` — game controllers, attached ones only (0x447bdd
through 0x447be9) — whose callback (`stdcall`, `ret 0x8`) makes the first one `CreateDevice` takes into `0x4c32a4` and
stops (0x447cb0 through 0x447ce3); and for a pad `SetDataFormat(0x48b764)` (0x447bfd), the level `10`,
`DISCL_NONEXCLUSIVE | DISCL_BACKGROUND` (0x447c07, 0x447c0e), `GetCapabilities` into `0x4c32ac` (`dwSize 0x2c`,
0x447c10, 0x447c28) and `EnumObjects(0x447cf0, 0, 0)` (0x447c3c), whose callback puts every axis (`dwType & 3`,
0x447cfa) at a range of ±1000 through `SetProperty(4, …)` (0x447d23 through 0x447d3b). Two copies of the set-up's call
with its flags' writes, 0x447c60 and 0x447d60, have no caller and no pointer to them. th10 ran its set-up on a
`_beginthreadex` thread the supervisor's add-callback started; th11's add-callback's `CreateThread` is the effects'
loader (see *The sound*).

The keyboard's bits, decoded from both halves (0x4576ee through 0x457848 for `GetKeyboardState`, 0x457883 through
0x4579e9 for DirectInput; the shifts and ors followed key by key):

| bit | key (VK / DIK) | what reads it |
| --- | --- | --- |
| `0x0001` | Z (`0x5a` / `0x2c`) | the shot; decide |
| `0x0002` | X (`0x58` / `0x2d`) | the bomb (`[0x4c93c0] & 0x2`, 0x4311dd); cancel |
| `0x0004` | — | no key and no pad button sets it |
| `0x0008` | Shift (`VK_SHIFT 0x10` / `DIK_LSHIFT 0x2a`) | the focus (`[0x4c93c0] >> 3 & 1` into the player's `+0x8d20`, 0x430368 through 0x430373) |
| `0x0010` `0x0020` `0x0040` `0x0080` | the arrows, and 8, 2, 4 and 6 of the numeric keypad | up, down, left, right (0x430293 on) |
| `0x0050` `0x0090` `0x0060` `0x00a0` | 7, 9, 1 and 3 of the numeric keypad | up and left, up and right, down and left, down and right |
| `0x0100` | Esc (`0x1b` / `0x01`) | the pause (`[0x4c92b4] & 0x100`, 0x42bf53); cancel |
| `0x0200` | Ctrl (`VK_CONTROL 0x11` / `DIK_LCONTROL 0x1d`) | a conversation run on (0x41d3ad, 0x41e977); also tested live at 0x40f152, 0x4362ff and 0x436d55 |
| `0x0400` | — | put in only by the replay's calc job (below); read gone down at 0x4308fe |
| `0x10000` | Q (`0x51` / `0x10`) | the pause menu's own way to the title (0x42cba6) |
| `0x20000` | S (`0x53` / `0x1f`) | no test of it found |
| `0x40000` | Home, P (`0x24`, `0x50` / `0xc7`, `0x19`) | the screenshot (`[0x4c92b4] & 0x40000`, 0x44689a in 0x446880) |
| `0x80000` | Enter (`0x0d` / `0x1c`) | decide |
| `0x100000` | D (`0x44` / `0x20`) | no test of it found |
| `0x200000` | R (`0x52` / `0x13`) | the pause menu's own way to a retry (0x42cb38) |
| `0x800000` | F10 (`0x79` / `0x44`) | gone down, toggles bits of `[0x4c3dc0]` in the supervisor's calc job (0x42908a) |

The `GetKeyboardState` half reads Shift and Ctrl as `VK_SHIFT` and `VK_CONTROL`, either side (`[esp+0x14]` and
`[esp+0x15]` of a buffer at `esp+0x4`, 0x457794, 0x45772a); the DirectInput half reads `DIK_LSHIFT` and `DIK_LCONTROL`
alone (`[esp+0x2e]`, `[esp+0x21]`, 0x45792b, 0x4578c1), so the right Shift and the right Ctrl do nothing there. Ctrl is
read off the run's copy, `[0x4c93c0] & 0x200` (0x41d3ad, 0x41e977), behind the conversation's own flag that lets it
run on, `test [ebp+0x90],0x1` (0x41d3a2, 0x41e96a), in 0x41d380.

**Every menu takes `[0x4c92b4] & 0x80001` as decide** (Z and Enter; 0x42ca55 among them) **and `& 0x102` as cancel**
(X and Esc; 0x42d1a5, 0x42dc83, 0x43a32a, …); `& 0x80103` is any of the four (0x439fd6, 0x43ecd3), and the live word
itself `& 0x80103` at 0x4206f4.

**The pad's half, `0x457270`**, reads joystick 0 one of two ways. Where the set-up found a DirectInput pad (bit
`0x800`), it is `Poll` (`[vtbl+0x64]`, 0x457407) — and where that fails, one `Acquire` (0x45741d), and only where that
one answers `DIERR_INPUTLOST` (`0x8007001e`) up to 400 more while they answer it (0x457430 through 0x45745d, each
passing `"error : DIERR_INPUTLOST %d"` at `0x497750` to 0x459bc0, a bare `ret`), the word then the keyboard's alone
with no `GetDeviceState` either way (0x457424, 0x457452, 0x45745f) — and `GetDeviceState` of `0x110`, a
`DIJOYSTATE2`, zeroed first (0x457476 through 0x45749f; a failure is the keyboard's word alone, 0x4574a3), its axes read
signed against the thresholds at `0x4c345e` and `0x4c3460`, `+0x16` and `+0x18` of the configuration and 600 apiece by
default (0x420d93 through 0x420dad): `lX < -t` left, `lX > t` right, `lY < -t` up, `lY > t` down (0x45752b through
0x457582). Otherwise it is `joyGetPosEx(0)` with `dwFlags 0xff` (0x4572bf, 0x4572c7; a failure is the keyboard's word
alone, 0x4572cf), each axis past a quarter of the travel the `JOYCAPSA` at `0x4c2f08` gives it from its middle —
`wXmin`/`wXmax` at `0x4c2f2c`/`0x4c2f30`, `wYmin`/`wYmax` at `0x4c2f34`/`0x4c2f38`, compared unsigned (0x457384
through 0x4573e8). **Neither looks at the hat.** The buttons are by the mapping at `+0x134` of the input, nine `i16`
at `0x4c93dc`, a negative one naming none (`test cx,cx; jl`). The read takes five of them: `[0]` the shot `0x1`, `[1]`
the bomb `0x2`, `[2]` the focus `0x8`, `[3]` the pause `0x100` and `[8]` the skip `0x200` (`dwButtons` bit `1 << n` at
0x4572e8 through 0x457382; `rgbButtons[n] >> 7` at 0x4574a9 through 0x457529).

**The mapping is copied out of `th11.cfg`'s `+0x4`** — eighteen bytes, `0x4c344c..0x4c345d` into `0x4c93dc..0x4c93ed`
(0x429f41 through 0x429f76) — by the configuration's load 0x429eb0 (`stdcall`, the supervisor on the stack, `ret 0x4`
at 0x42a0cb), which reads the 60-byte file into `0x4c3448` (0x429ece, 0x429ef0) and copies the mapping only where the
file is version `0x110003`, `0x3c` bytes long and its bytes `+0x1a..+0x1f` are in range (0x429efa through 0x429f3f).
Otherwise the defaults 0x420d80 (`esi` the configuration, plain `ret`) copy the mapping the other way, into the file's
`+0x4` (0x420db9 through 0x420de2), and the mapping is what a static initialiser, 0x48abf0 (its pointer at `0x48b320`,
in the C runtime's table of initialisers), wrote at load: `0, 1, 2, 4, -1, -1, -1, -1, 3` (0x48abf2 through 0x48ac44).
The configuration menu reads the pad's raw buttons through 0x4575a0 into 128 bytes at `0x4c98a0` (called at 0x43bb39)
and writes the mapping (0x43bbe7, 0x43bbf4); 0x458100 (the buffer in `esi`, answering 0 inactive, 1 DirectInput and 2
`GetKeyboardState`) is the raw keyboard for 0x43d847 and 0x43ed01.

**The run acts on a copy**, which the replay's calc job makes at priority `0xb`: recording, 0x436d20 (`mov eax,ecx;
jmp 0x435fe0`), added at 0x435923 with `ebx = 0xb` (0x43591c); playing back, 0x436d30 (`mov esi,ecx; call 0x436190`),
added at 0x435b4c with `ebx = 0xb` (0x435b47). The copy of the frame before goes into `0x4c93c4` either way (0x436008,
0x4361af).

Recording, **`[0x4c93c0] = [0x4c92a8] & 0xffff`** (`movzx eax,WORD PTR [0x4c92a8]`, 0x43600d), so Q, S, Home and P,
Enter, D, R and F10 never reach the run, and Esc and Ctrl do (th10 kept `& 0x1f7`). Then, by bit `0x200` of
`[0x4c3480]` (`th11.cfg`'s `+0x38`, 0x435ff9):

- **set**: Shift held puts `0x400` in (0x43601b through 0x436024), and a shot held for eight frames has the focus
  `0x8` put in as well, its count at `0x4c93bc`, held at 8 and put at nought where the shot is not (0x436029 through
  0x43604e);
- **clear**: `0x400` goes in where the shot and Shift are both held and each of their counters of the copy —
  `0x4c933c` (bit 0) and `0x4c9348` (bit 3), the last frame's — is under 5 (0x43605a through 0x436076), which is the
  two pressed together.

**`0x400` gone down is the switch of shot type 4** (`[0x4a5714] + 3 * [0x4a5710] == 4`, 0x4308d3 through 0x4308e3):
it steps the player's element `+0x8bac` on modulo 5 (0x4308fe through 0x430921).

Then 0x40de10 (`ecx = 0x4c92a8`, 0x436080, 0x436085; plain `ret` at 0x40de98) works out the copy's buttons gone down
into `0x4c93cc` and those let go into `0x4c93d0` (0x40de74 through 0x40de91), and its repeat into `0x4c93c8` and its
eight-frame word into `0x4c93d8` off the counters at `+0x94` (0x40de1d through 0x40de72) — the same 26 and 8 as the
live word's, and touching nothing of the live input (th10's 0x40ac29 put the live repeat at nought). The replay
records the three words `0x4c93c0`, `0x4c93cc` and `0x4c93d0` as `u16`, six bytes a frame (0x436106 through
0x43613a), a new block after 900 of them (`cmp eax,0x384` at 0x436157), and every thirtieth frame one byte off
`[[0x4a8d80]+0x34]` (0x436091 through 0x4360f8). Playing a replay back puts the replay's own six bytes a frame into
the same three (0x43620c through 0x436243), an all-`0xffff` record ending it (0x4361e4 through 0x4361f7) and nought
into all three past the stage's records (0x436296 through 0x4362a2); 0x40de10 is not called, so `0x4c93c8` and
`0x4c93d8` are not worked out on playback.

A third writer of the copy, 0x40d600's state 4 (0x40d63d through 0x40d662), copies the live word unmasked, calls
0x40de10, and then puts the live `[0x4c92b4]` into `0x4c93cc`; it is a calc job at priority 5 (0x40d2a0, 0x40d2a5 →
0x40ddd0 → 0x40d600) of a viewer of sprites and `.ecl` files (its strings `SprtView`, `../../data/*.ecl`).

**The player's job is priority `0x10`** (`mov ebx,0x10` at 0x42f95d, `mov [esi+0x8],0x431c50` at 0x42f962), 0x431c50 →
0x431070 (0x431c51) → 0x430290 (0x43121b), which reads `0x4c93c0` (0x430293, 0x430368), so **the player moves on the
frame's own keys**.

**Against th10**: the words are `u32` and not `u16`, in a `0x148`-byte input (th10 `0x6a` at `0x474e30`), with the
run's copy and its counters inside it and not in globals of their own; the read takes no argument (th10 `ecx` the
player) and the edges and the repeat moved out into 0x459b50; the pad's half takes the keyboard's word in `ecx` and
nothing on the stack (th10 pushed the player and the pad, `ret 0x8`). The bits moved: the focus `0x8` (th10 `0x4`), Esc
`0x100` (`0x8`), Ctrl `0x200` (`0x100`), and Q, S, Home and P, Enter, D and R from `0x200` … `0x4000` up to `0x10000` …
`0x200000`; F10 is new and `0x4` is set by nothing. Decide is `& 0x80001` (th10 `0x1001`) and cancel `& 0x102` (th10
`0xa`). The default mapping is `0, 1, 2, 4, -1, -1, -1, -1, 3` (th10 `0, 1, 2, 3, -1, -1, -1, -1, 4`), written by a
static initialiser and not held in `.data`. DirectInput is chosen by `0x400` and `0x800` of `[0x4c3810]` (th10 `0x200`
and `0x400` of `[0x491ff4]`); the configuration's flags are at its `+0x38` (th10 `+0x30`). The screenshot is the
present's 0x446880 testing `0x40000` gone down.

## The supervisor and its scenes

**`0x4c3280` is the supervisor**, `0x9c4` bytes, zeroed and given the flags `0x4240` by its static initialiser
(`0x42b9f0`, the `memset` of `0x9c4` and `or eax,0x4240` at 0x42ba06): `+0x0` the `hInstance` (0x4455de), `+0x4` the
`IDirect3D9` (`0x4c3284`, 0x445713), `+0x8` the device (`0x4c3288`, `TestCooperativeLevel` through `[[0x4c3288]]+0xc`
at 0x445858), `+0xf4` the present parameters `Reset` is handed (`0x4c3374`, 0x4459a0), `+0x1ac` … `+0x1b4` three
surfaces (`0x4c342c`, see *The sprite manager*), `+0x434` the camera the stage's background copies into
(`0x4c36b4`), **`+0x554` the scene it is on (`0x4c37d4`)**, **`+0x558` the scene wanted (`0x4c37d8`)**, `+0x55c` the
scene before the switch (0x42b12e), **`+0x564` the run's numbers to be started afresh (`0x4c37e4`, below)**, `+0x588`
an `.anm` of its own, **`+0x590` the input's and the display's flags (`0x4c3810`)**, `+0x594` the generator's seed as
it was taken (`0x4c3814`, 0x429588), `+0x810` its twelve critical sections (`0x4c3a90`) and `+0x990` the fog's dword
(`0x4c3c10`).

**Its calc job is `0x429030`**, priority 1, added by 0x4298c0 at 0x42992b with `0x429560` as its add-callback (`+0xc`,
0x429924), `__thiscall` on the supervisor (`mov esi,ecx` at 0x429031; Ghidra gives it `__fastcall`), plain `ret`. It
wants scene 3 where `[+0x590] & 0x180` is `0x80` — the bits `WM_CLOSE` sets and clears — and `[+0x800]` is nought
(0x429033 through 0x42904e); carries out the sound's commands (`call 0x44a340`, 0x429058); calls `0x42b8e0`; reads the
keys (`0x4576b0`, 0x429062); answers 4 where `0x4548d0` answers not nought (0x429070); toggles bits of `[0x4c3dc0]` on
F10 gone down (0x42908a); answers 4 or 1 by `[+0x80c]` where that is set (0x4290b2 through 0x4290c9); and otherwise
answers what **the scene switch `0x42b0e0`** answers, with the supervisor in `eax` (0x4290cb, 0x4290cd).

**`0x42b0e0`**: `eax` the supervisor (`mov edi,eax` at 0x42b0e3), plain `ret`, answering 1, or 4 for scene 3 and 5 for
scene 17. Nothing happens where `+0x554` equals `+0x558` (0x42b0f0). Otherwise, under the critical section at `+0x888`
(`0x4c3b08`) where bit `0x8000` of `+0x590` is set, it switches on `+0x558` through the table at `0x42b35c` (18
entries, 0 to 17, `cmp eax,0x11` at 0x42b14c, read at 0x42b151) and then writes `+0x558` into `+0x554` (0x42b1bb,
0x42b1c1):

| scene | at | what the switch does |
| --- | --- | --- |
| 0 | 0x42b158 | the startup: 1 wanted and the loading object (`0x428e10`) built into `+0x9a8`; 3 wanted where that fails (0x42b16d) |
| 1 | — | nothing: the loading, whose job `0x428ed0` takes bit `0x2000` off `+0x590` (0x428f06) and wants 4 (0x428f10) |
| 2 | — | nothing; written in place of the scene a writer wanted wherever bit **`0x2000`** of `[0x4c3810]` is set (below) |
| 3 | 0x42b177 | the game leaving: `0x429830` tears every object down, and the switch answers 4 |
| 4 | 0x42b18d | **the title**, built by `0x439900` (0x42b1ac). Coming from a run (7) the game object goes first (`0x420340`, 0x42b1a7); from the ending (15) the ending goes (`0x40f100`, 0x42b1e2); from 1 or 2 nothing goes; and from any other scene **the title is not built** (the byte table at `0x42b3b4`, entry 3 → 0x42b1b1) |
| 5, 6, 8, 9 | — | nothing (the table's default, 0x42b1b1) |
| 7 | 0x42b220 | **a run**: the title torn down where it came from 4 (`0x439970`), `+0x564` = 1, and the game object built by `0x4202d0(0)` |
| 10 | 0x42b29f | **the run started over** (the code's `SCENE_START_OVER`): the game object torn down (`0x420340`), `+0x564` = 1, 7 wanted, the stage `[0x4a5728]` put back to the stage the run started at `[0x4a572c]` and its record `[0x4a8ec8]` to `0x4a3828 + stage * 0x40` (0x42b2b4 through 0x42b2c8), and `0x4202d0(0)` |
| 11 | 0x42b2d7 | **a replay started over**: as 10, but `0x4202d0(1)` (`push ebx`, `ebx` 1, at 0x42b2fe) |
| 12 | 0x42b269 | **the stage after the one won**: `+0x564` = 0 (0x42b279), the game object torn down where it came from 7, 7 wanted, and `0x4202d0([old game + 0x74])` — the replay flag carried (0x42b270 through 0x42b295) |
| 13 | 0x42b240 | **a replay from the title**, and the attract demo: the title torn down where it came from 4, 7 wanted, `+0x564` = 1, `0x4202d0(1)` (0x42b24e) |
| 14 | 0x42b30e | **a continue**: the game object torn down, `+0x564` = 1, 7 wanted, `0x4202d0(0)` — the stage left where it is |
| 15 | 0x42b32f | **the ending**: the game object torn down where it came from 7, and `0x40f080` |
| 16 | 0x42b1e9 | **the title after the ending**: from 7 the game object goes, from 15 the ending; 4 wanted, `[0x4c3250] = 3` (0x42b214), and the title built — from 2 as well, and from any other scene with nothing torn down |
| 17 | 0x42b347 | the game leaving as 3, `0x429830`, the switch answering 5 |

**`+0x564` says the run's numbers are started afresh**: the stage's build puts the score, the lives and the rest in
place only where `[0x4c37e4]` is set (0x41f932, 0x41f975), and scene 12 alone leaves it at nought.

**Scene 2 and bit `0x2000`.** `0x40dea0` writes a scene with the supervisor in `eax` and the scene in `ecx`, plain
`ret`: `[eax+0x558] = ecx`, or 2 where bit `0x2000` of `[eax+0x590]` is set (0x40dea0 through 0x40debb). Its callers
hand it 3 (0x40d78a, 0x438baf, both `lea ecx,[edi+0x3]` with `edi` nought there), **12 at a stage's end** (0x41eda3), 4
at the attract demo's 3600th frame (0x42074b), and 4 from the end of a run and the pause menu (0x42d4ec, 0x42e5dc,
0x42f301). These write the same choice inline, `(bit 0x2000) ? 2 : n`: 16 (0x40f188), 15 (0x41b430), 3 (0x4203f8, a
stage whose build failed; 0x439509, a title whose build failed; 0x439d71, the title's screen 2), and 4 (0x42071a, the
demo taken back by a key or by the game object's flags `0x70`; 0x42d588 and 0x42d6d8, a replay's game over and a
replay's end). **Not gated**: the selects' 7 (0x43d5fc, 0x43da3a), the replays' 13 (0x439b00, 0x43e1d2), the start-overs'
10 and 11 (0x42d4b5 — `10 + ([game+0x74] != 0)` — 0x42e5f6, 0x42f2db), the end of a run's 10 or 14 (0x42e5bb: 14 where
`[+0x1f4]` is nought, 10 where it is set), and 3 at 0x428a8d and 0x42904e. **Nothing was found that sets bit
`0x2000`**: the only stores to `[0x4c3810]` that touch bits above `0x1000` are the initialiser's `0x4240` (0x42ba06,
0x48ab30), the `0x8000` of 0x44555b and 0x42a7b0, and the take-off at 0x428f06.

**Against th10**: the scene is at `+0x554` and the scene wanted at `+0x558`, the flags at `+0x590` (th10 `+0x3cc`), and
the scene-2 bit is `0x2000` (th10 `0x1000`, its writer 0x40ac90). **The scenes are renumbered from 10 on**, eighteen of
them: 10 the run started over and 11 a replay started over (th10: 10 the retry, 11 the next stage), **12 the next
stage** for runs and replays both, the replay flag carried (th10's 11), 13 a replay from the title (th10's 12), **14 the
continue** (th10's 13), 15 the ending and 16 the title after it (th10's 14 and 15), and 17 a second way out answering
5. `+0x564` is new.

### The game object

**`[0x4a8e88]`**, `0x78` bytes, built by **`0x4202d0`**: `stdcall`, one argument (`mov eax,[esp+0x8]` after `push
edi` at 0x420317, `ret 0x4` at 0x420330), which it stores as **`+0x74`, the replay flag** (0x420325) — 1 for a replay
played, nought for a run. It zeroes the object, sets `+0x60 |= 4` (0x42031b) and starts the build thread (`0x42a500`,
0x420328). `0x420340` tears it down (`0x41fe10`, then `free`). Its `+0x4` is the stage record's `+0x0` (0x41fc1b),
`+0x8` its calc job, `+0xc` its draw job, `+0x10` a timer whose count `+0x14` is **the frames the game object has run**
(stepped by `0x459270` at 0x4207fd), and `+0x24` … `+0x60` the part a replay's header keeps a copy of.

**Its flags at `+0x60`** (th10's `+0x58`):

- `0x4` while the stage is built on the loader thread: set by `0x4202d0` (0x42031b) and again as the build starts
  (0x41f903), and taken off as it ends (0x41fdbe);
- `0x8` where the build failed (0x41fcff): the calc job then wants 3 (or 2) on the next frame (0x4203d4 through
  0x4203f8);
- **`0x10` the game paused**: from the pause menu's opening (`0x42c620`, at 0x42c684), a replay's end (`0x42c760`, at
  0x42c7cb), the game over (`0x42d560`, at 0x42d601) and a run's end by its clear (`0x42d6b0`, at 0x42d73f), and taken
  off only by `0x42c880` (0x42c885), the pause menu's resume (called at 0x42d508);
- `0x80`, written by the calc job on every frame it waits on `0x4` (0x4206cb);
- `0x800` where the stage after another was built inside the same game object — the frame-0 path with `[0x4a8d5c]`
  set (0x420424) — taken off at frame 30, when that stage is set up (0x42043a);
- `0x2` and `0x400` both set skip every bullet's update (see *The bullets*).

**The game object's calc job is `0x420840`**, priority `0xa` (`mov ebx,0xa` at 0x41fb94, added at 0x41fba6 by the
build), `__thiscall` on the object, which pushes `ecx` and calls **`0x420390`**, `stdcall` on it (`mov edi,[ebp+0x8]`
at 0x4203ba, `ret 0x4` at 0x420405, 0x4206de, 0x420780 and 0x42080a). While `+0x60 & 4` it answers 1 having stepped
nothing (0x4206c4 through 0x4206de). **While any of `0x10`, `0x20` and `0x40` is set it answers `3`** (0x42076e through
0x420789 → 0x420775), which stops the walk; that is tested after the frame's own work (`0x41ef00` at 0x420769) and
before the frame counters are stepped. Otherwise it steps `0x4a5734` and `0x4a5738` (0x4207ee, 0x4207f4) and its own
timer, and answers 1.

**On the game object's frame 0** (`[+0x14]` nought, 0x4203c7) and not after a failed build, every enemy is destroyed
(`0x40d0c0([0x4a8d7c])`, 0x42046a), both stage frame counters are put at nought (0x420474, 0x42047a), `main` is spawned
(0x420495 through 0x42049e) and, unless the attract demo (`[0x4a5758] & 0x20`), the stage's song is started from its
record's `+0x34` (0x420525). The same is done on frame 30 (`0x1e`) of a game object with `0x800` (0x420560 on,
0x4205b1, 0x4205d2). What else that first frame runs is in *The stage's background* and *The resume path*.

**The stage is built on a thread**: `0x42a500` starts `0x41fe00` (`jmp 0x41f8f0`) through `_beginthreadex` (0x42a558,
the thread's procedure pushed at 0x42a531), its handle into `0x4c3a74` (0x42a56a). **`0x41f8f0`** sets `+0x60 |= 4`,
waits for the loading to finish (`Sleep(1)` while `[[0x4c3268]] >= 0`, 0x41f909 through 0x41f930), **where `+0x564` is
set puts the run's numbers in place** (0x41f932 through 0x41fb01, see *The run's numbers*), adds the game object's two
jobs, builds the replay (`0x435e00`, 0x41fc2b) and every manager of the stage — or, with `[0x4a5758] & 2`, keeps them
and goes through `0x436f30` (0x41fc98) — loads the `.std` (`0x4029c0([rec+0x4])`, 0x41fc42) and the `.ecl`
(`0x411070([rec+0xc])`) and, unless the attract demo, the stage's two songs (`0x42a0d0(0, [rec+0x10])`, `0x42a0d0(1,
[rec+0x14])`), and takes `+0x60 & 4` off (0x41fdbe) and `[0x4a5758] & 0xb` off (0x41fdc2).

**`0x41fe10` tears the game object down**, writing the score file first (0x41fe17, see *The score file*), and by the
scene wanted marks the run's flags `[0x4a5758]`: 12 sets `2`, the stage's managers kept for the next build (0x41fe73);
14, a continue, with the stage past the one the run started at (`cmp eax,[0x4a5728]` at 0x41fedb) adds one to the
continues used `[0x4a573c]`, at most 9 (0x41fee3 through 0x41fef3), and sets `8` (0x41fefd); and `1` is set (0x41ff65)
for 14 on the stage the run started at, counting nothing, and for 10 and 11 only where the stage is the one the run
started at (Ghidra's reading of 0x41fe10; 4 and 16 skip it). For 12 it keeps the previous stage's `.std` at
`[0x4a8d5c]` (0x4200bc through 0x4200c7).

**Against th10**: the game object is `0x78` bytes (th10 `0x60`), its flags at `+0x60` (th10 `+0x58`), the replay flag a
dword at `+0x74` (th10 `+0x5c`) handed to the builder as its argument; the failure flag `0x8` and the fade's `0x800` are
new.

## The front end

**The title `[0x4a8ecc]`** is `0x5b94` bytes (`push 0x5b94` at 0x439902), a `TitleInf` (its vtable written by the
constructor `0x439420`, `esi` the memory). `0x439900` builds it and starts its loading thread `0x4394f0`, which builds
the jobs in `0x4395a0` and wants 3 (or 2) where that fails (0x439509). **Its calc job is `0x439f10`**, priority 6
(`mov ebx,0x6` at 0x4395e1, added at 0x4395f9), `__thiscall` on the title, pushing `ecx` into **`0x4399b0`**, `stdcall`
on it (`ret 0x4` at 0x439e09, 0x439e35).

| offset | what |
| --- | --- |
| `+0x1c` | **the screen**, switched on at 0x439a6e through the table at `0x439e38` |
| `+0x20` | **the screen's step**: 0 the set-up, 1 waiting, 2 reading keys, 3 leaving forward, 4 leaving back |
| `+0x24` | **the cursor**; `+0x28` the cursor the frame before, `+0x2c` the count of items |
| `+0x2b0` | a timer: `+0x2b4` its count, **the frames since the screen or its step was set**, `+0x2b8` its float, `+0x2bc` the game speed's pointer `0x4a7948` |
| `+0x58d4` | the clear's name entry (screen 14): the name; `+0x58e0` how many are typed, `+0x58e4` whether the clear is ranked at all (nought where it is), `+0x58ec` the grid's place |
| `+0x58e8` | the difficulty kept while the Extra's 4 stands in `[0x4a5720]` (0x43a52e) |

`0x4390e0` sets the screen (`eax` the title, `ecx` the screen; `+0x1c`, `+0x20` at nought and the timer at nought) and
`0x439140` the step (`eax` the title, `ecx` the step, the timer at nought). Both plain `ret`.

| screen | handler | what |
| --- | --- | --- |
| 0 | inline | the set-up, by `[0x4c3250]`: 0 the first title (screen 1), 1 back to the main menu (screen 1, the cursor on the Extra where the difficulty is 4), 2 the replays (screen 11), 3 after the ending (screen 14) |
| 1 | `0x43a0a0` | **the main menu**, eight items: a run, the Extra, practice, the replays, the player's data, the music room, the options, and leaving |
| 2 | inline | wants 3, or 2 under bit `0x2000` (0x439d61), and falls into the stop of screen 9 |
| 3 | `0x43a6f0` | the options |
| 4 | `0x43b9d0` | an option's sub-screen, reached from the options' item 2 |
| 5 | `0x43c7c0` | **the difficulty** |
| 6 | `0x43cbe0` | **the character** |
| 7 | `0x43d0b0` | **the partner**, whose cursor is the shot |
| 8 | `0x43d6a0` | **practice's stage** |
| 9, 12 | `0x42a240` | nothing but a stop |
| 10 | `0x43e790` | the player's data |
| 11 | `0x43dd90` | **the replays** |
| 13 | `0x4408f0` | the music room |
| 14 | `0x43f6b0` | the ranking's name entry after the ending |
| 15 | `0x43ffa0` | the replay's slots after the ending |

**The frames each screen ignores keys for** are its steps 0 and 1: step 1 turns 2 once the count is past `0x50` on the
main menu (0x43a22d), past 6 on the options (0x43a758), the difficulty (0x43c8b6), the character (0x43cd27) and the
partner (0x43d3a3), and past 10 on practice's stage (0x43d72f). A decide sets step 3 (4 for the main menu), and the
select carries it out in step 3 or 4 once its count is past 0x14 on the main menu (0x43a4d0), 0xd on the difficulty
and the character (0x43caee, 0x43cf85), and on frame 10 and past `0x27` on the partner and practice's stage (0x43d50f,
0x43d57a; 0x43d9b5, 0x43d9e8). A cancel (`[0x4c92b4] & 0x102`) sets step 4, which goes back one screen once its count is
past 5. A decide is `[0x4c92b4] & 0x80001`, and up and down are `0x10` and `0x20` of `[0x4c92b4] | [0x4c92b0]` (left
and right `0x40` and `0x80` on the character).

**A run is written into the run's numbers screen by screen**:

- **the main menu** (step 4): item 0 takes practice off (`[0x4a5758] &= ~0x10`, 0x43a4f3) and goes to screen 5; item 1
  keeps the difficulty in `+0x58e8` and writes 4 into both `[0x4a3824]` and **the difficulty `[0x4a5720]`** (0x43a534,
  0x43a539); item 2 sets practice (`|= 0x10`, 0x43a57e) and goes to screen 5 with the difficulty `[0x4a3824]`, 1 where
  that is above 3 (0x43a5b2, 0x43a5b8). The main menu offers the Extra by the endings seen (see *The score file*);
- **the difficulty** (screen 5, step 3): **`[0x4a5720]` = the cursor** unless it is the Extra's 4 (0x43cb11 through
  0x43cb1e), `[0x4a3824]` the same (0x43cb23), and screen 6 with the character `[0x4a5710]` put at the one remembered
  in `[0x4c325c]` (0x43cb51);
- **the character** (screen 6, step 3): **`[0x4a5710]` = the cursor** (0x43cfb6), remembered in `[0x4c325c]`
  (0x43cfbc); the partner's cursor and `[0x4c3260]` put at nought for a character changed, or **the partner
  `[0x4a5714]`** kept from `[0x4c3260]` for the same one (0x43d036); then screen 7;
- **the partner** (screen 7, step 3): **`[0x4a5714]` = the cursor**, and `[0x4c3260]` (0x43d553 for practice on frame
  10, 0x43d58d at `0x28`). Practice goes to screen 8 (0x43d575). **For a run not practised** (0x43d59e) the title's
  screen is set to 2 (0x43d5ae), and **the stage `[0x4a5728]` and the stage the run started at `[0x4a572c]`** are
  written 1 and the stage's record `[0x4a8ec8]` `0x4a3868` — or 7 and `0x4a39e8` for difficulty 4 — (0x43d5b3 through
  0x43d5ec), and **the scene wanted is written 7** (0x43d5fc), not through `0x40dea0`;
- **practice's stage** (screen 8): a decide counts only for a stage the score file marks offered (`cmp byte
  [..+0x5b1],0` at 0x43d80a), and then **the stage and the stage started at = the cursor + 1**, its record `0x4a3828 +
  stage * 0x40`, and **7 wanted** (0x43da11 through 0x43da3a). It reads the keyboard's state as it decides: a key 1 to
  9 held (`DIK_1` … `DIK_9` at `0x4c9692` …) writes 1 to 9 into `[0x4a5764]` (0x43d83d through 0x43d9a3), which the
  build makes the lives less one;
- **the replays** (screen 11): the stage chosen + 1 into the stage, the stage started at and the record, and **13
  wanted** (0x43e1b3 through 0x43e1d2), the character, the partner and the difficulty from the replay (0x43e210,
  0x43e219, 0x43e22a).

**The attract demo**: screen 1 counts `[0x4a575c]` while no key is held (`[0x4c92a8]` nought) and at `0x708` frames
loads `demo/demo%d.rpy` (`[0x4a5760]`, 0 to 3, the names at `0x4a7d60`), sets `[0x4a5758] |= 0x20` and takes `0x40`
off (0x439aa3), takes the first stage the replay holds, its character, partner and difficulty, and **wants 13**
(0x439af6 through 0x439b3e).

**Against th10**: the title is `0x5b94` bytes (th10 `0x5acc`) and its screens wait and read in steps inside one handler
each: there is no screen waiting for a key, and the main menu is screen 1, the difficulty 5, the character 6, the
partner 7, practice's stage 8 and the replays 11 (th10: 2, 6, 7, 8, 9 and 12). The offsets `+0x1c`, `+0x24` and
`+0x2b4` are th10's; `+0x20` is the step.

## The run's numbers

**`0x4a56e0`, `0x90` bytes**, zeroed by `0x420e20` (which then writes the difficulty and the power's unit 1,
0x420e36 and 0x420e3b, and answers `0x4a56e0`) — th10's `0x474c40`:

| address | what |
| --- | --- |
| `0x4a56e0` | the high score, raised to the score where it is lower (0x42ba33, 0x41ef8c, 0x41fb18); from the score file for a run started afresh (0x41f9cc) |
| `0x4a56e4` | **the score**, over ten as th10's: the clear bonuses are paid in tenths (0x41ebd1 on) and the ranking stores it beside the continues (0x42bafa, 0x42bb1f) |
| `0x4a56e8` | **the power**, nought to `[0x4a5748]`: `0x420a00` adds `eax` and holds it to `[0x4a5748]` (0x420a09 through 0x420a29) |
| `0x4a56f0` | **the point items' value**: put at `[0x4a79e8 + difficulty * 4] * 100` with the run (0x41f9eb, 0x41fa25, 0x41fa2b — 25,000, 50,000, 100,000, 200,000 and 200,000, times 100); `0x433d10` takes `0x989680`, 10,000,000, off it, not below its start `[0x4a5768]` (0x433d10 through 0x433d29), and the death does so (0x4327d0); kept in a replay's stage over 100 (0x435866) and put back times 100 (0x435ac7) |
| `0x4a56f4` | a word put at nought in player states 0, 2 and 4 (0x41fb24, 0x431097, 0x431227, 0x4312c0), and held against 10000 by the items (0x42364a) |
| `0x4a56fc` … `0x4a570c` | a timer (`0x4a5704` its float, `0x4a5708` the game speed's pointer) |
| `0x4a5710` | **the character**: 0 Reimu and 1 Marisa, `pl00` and `pl01`, the `.sht` named by `[0x4a7a90 + (character * 3 + partner) * 4]` (0x42f906 through 0x42f916) |
| `0x4a5714` | **the partner** (the code's `G_SHOT_TYPE`), 0 to 2, `a` to `c` of the same name; `partner + 3 * character` is the shot type, 0 to 5, `pl00a.sht` … `pl01c.sht` |
| `0x4a5718` | **the lives**: 2 for a run, 9 for practice, `[0x4a5764] - 1` for practice with a number key held (0x41fa12 through 0x41fa56); one more by `0x420950`, at most 9; one less by `0x433d30` and at a death; below nought after a death, the game over (0x4313f5) |
| `0x4a571c` | **the life pieces**: `0x4209b0` adds one and makes a life of five (0x4209c4 through 0x4209de), and holds them at nought with 9 lives |
| `0x4a5720` | **the difficulty**, 0 to 3, 4 the Extra; the build writes 4 for stage 7 (0x41f98f); the graze's score tables are indexed by it (0x408bc8 on), and an enemy's rank bit is `1 << difficulty` (0x4108f0) |
| `0x4a5724` | the difficulty kept while the attract demo stands in it |
| `0x4a5728` | **the stage**, 1 to 6, 7 the Extra (8 the ranking's all clear) |
| `0x4a572c` | **the stage the run started at** |
| `0x4a5730` | the number + 1 of the last conversation started (`0x41d2a0`, 0x41d332), nought at a stage's build |
| `0x4a5734` | **the frames since the stage began**, nought on the game object's frame 0 (0x420474) and at the build (0x41f969), stepped at 0x4207ee |
| `0x4a5738` | the frames since the last conversation of a new number began, put at nought there (0x41d339) and at 0x420474 |
| `0x4a573c` | **the continues used**, at most 9 (0x41feec through 0x41fef3); nought for a run not continued (0x41f9de) |
| `0x4a5740` | the score file's byte `+0x1d` of the shot type and the difficulty (0x41f9d6) |
| `0x4a5744` | a word held within ±`0x400` (0x42383d through 0x42385e and six more) — th10's `0x474c98`: `-0x400` at a death (0x432a52), `+1` every 60 frames of an enemy up (0x431a9d on), `+0xc`, `+0x18` and `+0x100` by items and `-4` by an item lost off the bottom; `-0x200` for a continued run's build and nought otherwise (0x41fb01) |
| `0x4a5748` | **the power's top**, `[sht+0x24] * [sht+0x20]` of the shot type's `.sht` (0x42fa66 through 0x42fa6d) |
| `0x4a574c` | **the power one bomb costs**, the `.sht`'s `+0x24` (0x42fa79, 0x42fa7c) |
| `0x4a5750` | **the extends the score has given**: `0x41ef00` holds the score against `[0x4a3b1c + n * 4]` (or `[0x4a3d60 + n * 4]`), extends and counts (0x41efd8 through 0x41f006) |
| `0x4a5754` | **the graze**, at most 99,999,999 (0x40bb10, 0x4275da) |
| `0x4a5758` | **the run's flags**: `0x1` torn down for a continue or a start-over on the stage the run started at (0x41ff65), `0x2` the stage after one won, its managers kept (0x41fe73), `0x4` (0x41f430), `0x8` a continued run (0x41fefd), **`0x10` practice** (0x43a57e; read at 0x42d590, 0x42daa3, 0x42e783), **`0x20` the attract demo** (0x439aa3, 0x42bf4a), and `0x40`, taken off by the demo |
| `0x4a575c`, `0x4a5760` | the demo's idle count, and which demo is next |
| `0x4a5764` | practice's number key, above |
| `0x4a5768`, `0x4a576c` | the point items' value's start, and a second number by difficulty (`[0x4a79fc + difficulty * 4] * 100`, 0x41f9e4, 0x41f9f8) |

**There is no stock of bombs: a bomb costs power.** The player's update `0x431070` bombs where the bomb's bit is held
(`[0x4c93c0] & 2`) and the power holds at least one bomb's cost (`[0x4a56e8] / [0x4a574c]` not nought), and takes
`[0x4a574c]` off the power (0x4311cd through 0x4311f1 while hit, the deathbomb; 0x431265 through 0x43129d otherwise;
see *The player*). th10's bomb was a fixed 20 of its power.

**The run's numbers are put in place by the build** (`0x41f8f0`, where `+0x564` is set): the difficulty 4 for stage 7
(0x41f981, 0x41f98f), the high score from the score file (0x41f9cc), the continues at nought unless the run was
continued (flag `0x8`, 0x41f9c2 through 0x41f9de), the point value's start, the score at nought (0x41fa1f), the graze
and the life pieces at nought (0x41fa19 through 0x41fa30), the lives as above (2 for a run not practised, 0x41fa38),
the player built (0x4301b0 at 0x41fa5b), the power at nought for stage 1 and the Extra and at the top for practice
from stage 2 on (0x41fa68 through 0x41fa94), the options made for it (0x41fa99), the score file's count of plays
(below, 0x41facd through 0x41fae1), `0x4a5750` at nought (0x41faf7), and `0x4a5744` at `-0x200` with flag `0x8` and at
nought without it (0x41fae3 through 0x41fb01). So a continue starts the stage over at a score of nought.

**The generator** is the word at **`0x4c2f00`**, with the count of its draws at `0x4c2f04`, stepped by **`0x458bc0`**
(`esi` the generator, the 16 bits answered in `ax`, plain `ret`, under the critical section `0x4c3b80`): `+0x4` one
higher, then `x = ((x ^ 0x9630) - 0x6553)` and `x = (x >> 14) + x * 4` in 16 bits (0x458bdd through 0x458c01);
`0x458c30` and `0x458cd0` step it twice and answer 32 bits; inlined copies at 0x4376cc and 0x4444b8. **It is seeded
once from `timeGetTime`** (`call [0x48b298]` at 0x429582) as the supervisor's job is added (`0x429560`), into
`0x4c3814`, `0x4c2f00` and a second generator `0x4c2ef8` (0x429588 through 0x429593), and not again at a stage: a
replay keeps each stage's seed and puts it back (see *The resume path*). The score file's `ST` made empty draws
`0x200` times from `0x4c2f00` (0x437660), once, at the startup. `0x4c2ef8` is stepped by the effects (`mov
esi,0x4c2ef8` at 0x44b127 and six more) and kept by no replay.

**Against th10**: the lives come with five life pieces, at most 9; the graze is counted; the point items' value stands
where th10's faith did; and there are two stage frame counters, `0x4a5734` the stage's (th10's `0x474c88`) and
`0x4a5738` started again at each new conversation.

## The stage records

**`[0x4a8ec8]` is the stage's record of what it loads**, `0x4a3828 + stage * 0x40` (0x42b2c1, 0x41f463, 0x43da19):
`0x40` bytes, from stage 0 at `0x4a3828`.

| offset | what |
| --- | --- |
| `+0x00` | the stage's number (into the game object's `+0x4`, 0x41fc1b) |
| `+0x04` | the `.std` (`0x4029c0`, 0x41fc3e) |
| `+0x08` | the enemies' `.anm` |
| `+0x0c` | the `.ecl` (`0x411070`) |
| `+0x10`, `+0x14` | the two songs, into slots 0 and 1 (`0x42a0d0`) |
| `+0x18` … `+0x2c` | six `.msg`, by `(character + 2) * 3 + partner` (0x41a59e through 0x41a5b7): `stNN_00a` … `c` for character 0, `stNN_01a` … `c` for character 1 |
| `+0x30` | the stage's logo `.anm` |
| `+0x34` | the stage's song's number, played into slot 0 on the stage's first frame (0x420525) |
| `+0x38` | the boss's song's number, played into slot 1 by an instruction of the stage's script (0x41e9d7) |
| `+0x3c` | nought |

| stage | at | `.std` | songs | numbers |
| --- | --- | --- | --- | --- |
| 0 | `0x4a3828` | `stage00.std` | `bgm/th11_00.wav`, `bgm/th11_01.wav` | 0, 0 (its `.msg` all `st01_00.msg`) |
| 1 | `0x4a3868` | `stage01.std` | `bgm/th11_01.wav` (`0x495f24`), `bgm/th11_02.wav` (`0x495ed0`) | 1, 2 |
| 2 | `0x4a38a8` | `stage02.std` | `bgm/th11_03.wav`, `bgm/th11_05.wav` | 3, 4 |
| 3 | `0x4a38e8` | `stage03.std` | `bgm/th11_06.wav`, `bgm/th11_07.wav` | 5, 6 |
| 4 | `0x4a3928` | `stage04.std` | `bgm/th11_08.wav`, `bgm/th11_10.wav` | 7, 8 |
| 5 | `0x4a3968` | `stage05.std` | `bgm/th11_12.wav`, `bgm/th11_13.wav` | 9, 10 |
| 6 | `0x4a39a8` | `stage06.std` | `bgm/th11_14.wav`, `bgm/th11_16.wav` | 11, 12 |
| 7 | `0x4a39e8` | `stage07.std` | `bgm/th11_15.wav`, `bgm/th11_17.wav` | 13, 14 |

**`0x4a3a28`, "stage 8", is past the table**: the `.data` after the Extra's, holding `0, 0x434e20, 0x435210,
0x435250, …` (`objdump -s` over `0x4a3a28`–`0x4a3a68`) — its `+0x4` a code address. The ranking after the ending
(0x43f7f8, 0x43fffa) and a cleared Extra's end (`0x42d850`, 0x42d86c) point the record at it (see *The pause and the
end of a run*). th10's records were `0x30` bytes, with their songs' numbers at `+0x24` and `+0x28`.

## The stage's script

**`0x45b860` runs one script**: `eax` the script (`mov edi,eax` at 0x45b86a), one float pushed (`fadd [esp+0x134]`
after `sub esp,0x120` and four pushes, 0x45d29c), **`ret 0x4`** (0x45d2b1, 0x45d2c4). It answers nought, or -1 for a
script that ended (instruction 1, 0x45d2b4; a return with nothing to return to). A script: its time, a float, at
`+0x0`; where it is at `+0x4`; a `0x1000`-byte stack at `+0x8` with its top at `+0x1008`; its owner at `+0x1014`; the
rank's bit at `+0x101c`. An instruction: its time, an `i32`, at `+0x0`, its number an `i16` at `+0x4`, its size a
`u16` at `+0x6`, and its ranks at `+0xa`, a byte held against `[script+0x101c]` — an instruction of another rank is
passed over (0x45b893 through 0x45b89c).

**Each run** executes every instruction whose time is no later than the script's (`fild [ins]; fld [script]; fcompp`
at 0x45b879, again at 0x45d289), and then adds the float to the time (0x45d29a through 0x45d2a3). Instructions 0 to
`0x58` are the runner's own (the table at `0x45d2c8` by the bytes at `0x45d3bc`); every other goes to the owner's
handler, the first slot of the vtable at `[[script+0x1014]]` (0x45d269 through 0x45d273). **An instruction answering
`-1` holds the script** where it is, with neither the time nor the place moved (0x45d275 → 0x45d2a5). A jump (12,
0x45bac2) sets the time to the `i32` at the instruction's `+0x14` and moves the place by the one at `+0x10`; 13 and 14
do so on what they pop (0x45ba74, 0x45ba25); return (10, 0x45b8bd) pops the place and the time back; call is 11
(0x45ba10 → `0x45b610`). So, as in th10, the time is the time of the sub the script is in.

**`0x45d420` runs all of an enemy's scripts**: `edi` the enemy, one float pushed, `ret 0x4` (0x45d4a1, 0x45d4ab), from
the list whose head is the element at `enemy+0x1030` — `+0x0` the script, `+0x4` the next, `+0x8` the one before —
writing each script into `enemy+0x4` as it runs it (0x45d440), freeing each script but the first that answers -1, and
answering -1 itself where the first does (0x45d44c → 0x45d4a4). The enemy's update `0x411750` calls it with `edi =
[body+0x1614]` — the enemy, where `body` is `enemy+0x103c` — and the float `[[body+0x160]]`, which is `[0x4a7948]`,
**the game speed** (0x411aa1 through 0x411abb).

**An enemy** (`EnemyInf`, vtable `0x494074`) is `0x2678` bytes (`push 0x2678` in `0x4108f0`), made by `0x4111c0` (`edi`
the memory, the name pushed): `+0x4` the script running, **its main script at `+0x8`**, `+0x101c` the enemy itself
(the main script's `+0x1014`), `+0x1024` the rank's bit, `1 << difficulty` (`0x4108f0`), `+0x1030` the list's first
element (the main script), `+0x11a4` its element of the enemy manager's list, and `+0x2650` the enemy again (its
body's `+0x1614`). **The enemy's handler** is its vtable's first slot, `0x412e20`: `add ecx,0x103c; jmp 0x412e30`,
`__thiscall` on the body, plain `ret` (0x416bd3), switching on the number less `0x100` (`cmp ecx,0xc2` at 0x412e97, the
bytes at `0x416df0`, the table at `0x416bd4`).

**Only two instructions answer `-1` there**, the same two numbers as th10's: **339 while a conversation
`[[0x4a8d84]+0x4438]` is up with its `+0x8c` at nought** (0x415af6 through 0x415b16; `0x410720` reads
`[[[0x4a8d84]+0x4438]+0x8c]`), and **340 while the boss slot `[[0x4a8d7c]+0x1c]` is set** (0x415b1e through 0x415b2d).
338 starts a conversation (`0x41d2a0`, 0x415ad7), `stdcall` with its number: the conversation is an `0xac`-byte object
made from the `.msg` held at `[[0x4a8d84]+0x443c]`. **`[0x4a8d84]` is the object a stage's conversations are run
from**, its `+0x4438` the conversation running and nought where none is.

**The stage's own script is `main`** (`0x494258`), spawned on the game object's frame 0 by **`0x4108f0`**: `stdcall`
with the name pushed (`push 0x494258` at 0x420495), `ebx` a `0x50`-byte block of the enemy's settings (0x42049a), `ret
0x4` (0x410ade, 0x410b0e); it appends at the tail (0x410ae1 through 0x410b06). Its answer is dropped (0x4204a3 is the
next call).

**Against th10**: the runner handles 0 to `0x58` itself and hands every other instruction to its owner's vtable; the
main script is still at `enemy+0x8` and the list at `+0x1030`, but the enemy is `0x2678` bytes with its body at
`+0x103c`; the conversation is an object of its own (its `+0x8c` th10's `+0x78`) and the boss slot is at `+0x1c` (th10
`+0x10`).

## The player

**`[0x4a8eb4]`**, `0x8d24` bytes: 0x4301b0 allocates it (0x4301d1), 0x42f760 zeroes it and stores it here (0x42f852),
and 0x42f8a0 builds it (`[esp+0x4]` the player, `ret 0x4`). Its calc job is `0x431c50` → `0x431070` (`stdcall` on the
player) at **priority `0x10`** (0x42f95d), its draw job `0x431c60` at `0x16` (0x42f9b2). The lasers' calc is at `0x13`
(0x4249e3) and the bullets' at `0x14` (0x4084cd), so the order is th10's: **the frame's move before the frame's fire is
tested**. The enemies' is at `0x12` (0x410c71), the bomb's at `0x11` (0x4061ea) and the items' at `0x15` (0x4232af).

| | |
| --- | --- |
| `+0x14` | its sprite |
| `+0x87c` | **where it is**, three floats: `+0x888` and `+0x88c` times `0.0078125` (0x430859 through 0x430870, the double at `0x497ed0`) |
| `+0x888`, `+0x88c` | **where it is in 128ths**, two `i32`, clamped to `±0x5c00` (±184) across and `0x1000` to `0xd800` (32 to 432) down (0x4307c5 through 0x430849), the clamp skipped while `+0x920` is 99 or 100 (Yukari's gap, below) |
| `+0x890` … `+0x89c` | **the four speeds in 128ths**, `i32`: straight, focused straight, diagonal, focused diagonal — the `.sht`'s floats at `+0x10` … `+0x1c` times `128.0` (`0x497ed8`, 0x42fa20 through 0x42fa60); read at 0x4303fa through 0x430518 |
| `+0x8a0`, `+0x8a4` | the frame's step, the speed times `[0x4a7948]` (0x430290) |
| `+0x8cc`, `+0x8d8` | **the corners of its box**, three floats each, `+0x87c ∓ +0x8e4` (0x431741 on, every frame; 0x42fbcd at the build) |
| `+0x8e4`, `+0x8e8`, `+0x8ec` | **the half of its box**: half of `[0x4a7aa8 + character * 4]` across and down, `5.0` deep — `2.0` and `3.5` in the table, so **`1.0` for Reimu and `1.75` for Marisa**, the same for every partner (0x42fadc, 0x42fb21 through 0x42fb45) |
| `+0x8f0` … `+0x904` | halves of the item boxes: `+0x8f0`/`+0x8f4` half of `[0x4a7ab0 + c * 4]` (30, 35), `+0x8fc`/`+0x900` half of `[0x4a7ab8 + c * 4]` (50, 59), `5.0` deep |
| `+0x91c` | the direction held: 0 none, 1 up, 2 down, 3 left, 4 right, 5 to 8 the diagonals (0x430290) |
| `+0x920`, `+0x924` | Yukari's gap's state and its frames (below) |
| `+0x928` | **its state**: 0 coming back in, 1 playing, 2 dying, 3 a handler with no writer, 4 hit and inside the deathbomb's window (the switch at 0x43108e, its table `0x431ba4`) |
| `+0x92c` | the shot type's `.sht` record: `+0x4` the whole hitbox (`[0x4a7aa8 + c * 4]`), `+0x8` the speed items fly to it at (`[0x4a7ac0 + c * 4]`, 5.0 and 7.0), `+0xc` `[0x4a7ab0 + c * 4]` (60 and 70) (0x42fae9 through 0x42fb15) |
| `+0x930` | the shot's timer, its count `+0x934` held at -1 while the shot is off (0x42fab7, 0x431b69; 0x434380) |
| `+0x944` | **the state's timer**, its count `+0x948` (0x4327d0 and 0x432a90 start it at nought) |
| `+0x958` | the timer every update steps, its count `+0x95c`; the focus is held off while it is below 4 (0x430290) |
| `+0x96c` | **the player's shots**, `0x100` of `0x6c` bytes (see *The player's shots*) |
| `+0x756c` | a sprite the stage's first frame frees (0x42fec0) |
| `+0x7570` | **the options**, 8 of `0xe4` bytes: `+0x0` there or not, `+0x54`/`+0x58` where it is going and `+0x5c`/`+0x60` where it is in 128ths (closing 30% a frame on it, `+0x8ba0` = 30), `+0x64`/`+0x68` its place unfocused and `+0x6c`/`+0x70` focused against the player, and `+0xd4` a flag (see *The options made for the power*) (0x430b64 through 0x430d80) |
| `+0x7c90` | **how many options there are**, `power / [0x4a574c]` (0x432cc0, 0x43361d) |
| `+0x7c94` | the enemy the homing shots go for, the one nearest across (0x411750 at 0x412039, 0x410660) |
| `+0x8ba4`, `+0x8ba8` | Suika's and Aya's counts of frames (below) |
| `+0x8bac` | **the element Patchouli's options are on** |
| `+0x8bb0` | **the timer of the frames nothing can hit it for**, its count `+0x8bb4` counting down and its float `+0x8bb8` (0x431658 through 0x4316c2) |
| `+0x8bc4` | its flags: `0x2` untouchable (every test refuses), `0x4` Nitori's bomb armed, `0x8` the options leaving (0x430b3f) |
| `+0x8bcc`, `+0x8bd8` | the box an item is collected in, `± +0x8f0 * 0.5` — 15 for Reimu, 17.5 for Marisa (0x4317ff on) |
| `+0x8be4`, `+0x8bf0` | the box, focused, that pulls an item in, `± +0x8fc` (0x4318bb on) |
| `+0x8bfc`, `+0x8c08` | the same unfocused, `± +0x8f0` (0x431957 on) |
| `+0x8d20` | **focused**, `([0x4c93c0] >> 3) & 1` while an enemy is up and `+0x95c` is 4 or more, nought otherwise (0x430373, 0x43037b) |

**The states' frames** (0x431070):

- 0, coming back in: `+0x88c = 0xf000 - frame * 0x2800 / 60`, 480 up to 400 (0x431095 through 0x4310d9); fire
  cancelled about it for the first 30 frames (0x431112 through 0x431180); playing at frame 60 (0x431188).
- 1, playing: **the bomb** — 0x406510 called where `[0x4a8d84]` is there and its `+0x4438` (the conversation) is
  nought, the bomb's `[0x4a8d64]+0x3c` is nought, `power / [0x4a574c]` is not nought and **`[0x4c93c0] & 2` is held**,
  and then `[0x4a574c]` taken off the power (0x4311ac through 0x4311f9); then the move, `0x430290` (0x43121b); and the
  shot's step (see *The player's shots*).
- 4, hit: a bomb asked for the same way but with no test of the conversation (0x43124f through 0x4312aa) sets the
  state's timer at 60 and the state at 1; **on the eighth frame (`+0x948 >= 8`, 0x43122d) the death, 0x4327d0**, then
  `[[0x4a8d7c]+0x10]` one higher and `[[0x4a8d7c]+0x18]` at nought (0x431238 through 0x43124a), and on to state 2.
- 2, dying: on frame 3, **`[0x4a5748]` off the power**, not below nought (`sub eax,[0x4a5748]` at 0x4312da, 0x4312c6
  through 0x4312ee), and seven items (power `1` or big power `4` by how much it had, a full power `6` in the middle
  where the lives are below 1) thrown out (0x431381); from frame 30 (0x4313e6) the game over where the lives are below
  nought (0x4313f5, opening the pause object's game over at 0x431411, or a replay's end at 0x43149f), and otherwise
  state 0 at x 0, y 480 in 128ths, the game speed at `1.0` (0x4314b8), **nothing hitting it for 280** (`push 0x118` at
  0x431506).
- 3: frame 15 cancels fire (0x43153a); nothing writes 3 (every store to `+0x928`: 0x407625, 0x4078c4, 0x42feca,
  0x43119d, 0x4312aa, 0x4314b2, 0x432822, 0x432a9e).

**The tests, four of them, each with the shape to test in `eax`** (`ret`, or `ret 0x4`/`0xc`):

| test | used by | hit | graze |
| --- | --- | --- | --- |
| `0x431e00`, a box | a bullet without flag `0x10` (0x408bb0); `edx` the centre, `eax` the width and height, halved | the bullet's box meets `+0x8cc` … `+0x8d8` | the bullet's centre ±`24.0` (`0x498318`) meets the player's box (0x431eed through 0x431f5f) |
| `0x431f70`, a circle | a bullet with flag `0x10` (0x408bc3) and **an enemy's body** (0x411e48); the radius pushed | `d² < h² + r²`, `h` the `.sht`'s whole hitbox `[+0x92c]+0x4`, 2.0 or 3.5 (0x431f79 through 0x431fbf) | `d² < (h + max(r / 2.5, 40))² + r²` (0x431fc1 through 0x432008) |
| `0x432070`, a rectangle from its origin | both lasers (0x426052, 0x42753c); the angle, the width (halved inside) and the length | the rectangle meets the player's box (`+0x8e4`, `+0x8e8`) | it meets the player's box scaled by `16.0` (`0x497ee0`) |
| `0x432520`, a rectangle about its centre | an enemy's hit callback 1, `0x418050` (0x41812b) | as above | as above |

Each answers nought while a conversation is up (`[[0x4a8d84]+0x4438]`), while the state is 2, 3 or 4, or while bit
`0x2` of `+0x8bc4` is set (0x431e90 through 0x431ece; 0x432013 through 0x432047). **Past those, the box and the circle
answer 1 either way** and take the hit through `0x432a90` only where `+0x8bb4` is below 1 (0x431ed4 through 0x431ee4,
0x432049 through 0x432059) — a bullet that touches a player nothing can hit is cancelled all the same, its state put at
3 (0x408bda). **The two rectangles answer 1 only where they take the hit**, and nought where `+0x8bb4` is above nought
(0x43223d through 0x43227b, 0x4326f5 through 0x43273f). A bullet grazes once: 2 answered sets bit `0x4` of the
bullet's flags, and a bullet with it set grazes no more (0x408c0f). `0x4322f0` is the rectangle test without the hit or
the refusals, answering 1 on a touch; the warning-line bender `0x428610` alone calls it (0x4286d1).

**`0x432a90` takes the hit** (`eax` the player): the state at 4 (0x432a9e), its timer at nought, `+0x8bb4` at **6**
(0x432c29), 32 sprites of the hit put up; and where the card is up (`[[0x4a8d6c]+0x8e0] & 1`) and has run 60 frames or
more (`+0x88c > 0x3b`), its `+0x8e4` at nought and bits `0x22` of `+0x8e0` taken off — the capture lost; under 60
frames with a bomb going off (and not shot type 5), `0x20` put on instead.

**`0x4327d0` is the death** (`eax` the player): `[0x4a56f0]` less 10,000,000, not below `[0x4a5768]`; **a life off
`[0x4a5718]`**; the state at 2 (0x432822) and its timer at nought; **`+0x8bb4` at 180** (0x4328ac); every option let go
and `+0x7c90` at nought (0x4329ec); the card's capture lost as in the hit (0x432a0a through 0x432a4c); and `0x400` off
`[0x4a5744]` (0x432a52). It takes no power: that is state 2's third frame, above.

**The bomb `[0x4a8d64]`**, `0x4a0` bytes (0x406180, stored at 0x4061a6), its calc at `0x11`: `+0x3c` is a bomb going
off and `+0x14` its timer. `0x406510` starts one only where `+0x3c` is nought (0x40651b): `+0x3c` at 1 (0x40652e),
`+0x498` at 1 where a card had run 60 frames, and the shot type's own start — 0 `0x45dd70`, 1 `0x407a40`, 2 `0x407bb0`,
3 `0x406920`, 4 `0x407130`, 5 `0x407680`. **Each bomb holds the player unhittable through `0x406860(40)`** every frame
it runs and once more as it ends (0x4071bb, 0x407963, 0x407ade, 0x407d3a, 0x45de73), so a bomb leaves 40 frames after
it; shot type 3's start puts 120 (0x406ccf). The starts add one to `[[0x4a8d7c]+0x14]` (0x406cee, 0x45de20; Nitori's as
it goes off, 0x4078bf). **Shot type 0's bomb (Yukari) sets bit `0x2` of `+0x8bc4`** — the player untouchable, every
test refusing — and its end takes it off (0x45de06, 0x45de5e). **Shot type 5's bomb (Nitori) only arms**: bit `0x4` of
`+0x8bc4` (0x407714); its calc, 0x407720, sets it off — `+0x928` back at **1** — where the bit is clear or the state is
4 or 2 (0x407897 through 0x4078c4), so it takes a hit back; and the card's start does not count it as a bomb used
(0x40c650, `!= 5`).

**The partners' own moves** (0x430290):

- **Yukari (shot type 0), the gap**: a tap away from a side wall and back against it within 9 frames (`+0x920` 1→2 or
  3→4, 0x430e50) sets `+0x920` to 99 (left) or 100 (right): the player is driven `∓0x96` 128ths a frame, the clamp
  skipped, and past `∓0x67ff` it comes out on the other side at `±0xd000` (0x430f1f through 0x431056), for 45 frames.
  Two functions with no direct caller found, 0x430de0 and 0x430e20, set and clear bit `0x2` of `+0x8bc4` around it.
- **Suika (1)**: unfocused, still and not shooting for more than 10 frames (`+0x8ba4`, 0x43078f) calls 0x4245e0, which
  sends every falling item (state 1) flying to the player (state 3).
- **Aya (2)**: neither the shot nor the focus held for more than 10 frames (`+0x8ba8`, 0x43065f through 0x430665)
  **doubles the step** (0x430675 on).
- **Patchouli (4)**: `0x400` gone down cycles `+0x8bac` through 5 elements and makes the options again (0x4308fe on).
- **Nitori (5)**: the armed bomb above; and bodies are tested differently while it is armed (0x411b24, 0x411c3a).

**Against th10**: the player is `0x8d24` bytes (th10 `0x4478`); its place is at `+0x87c` (th10 `+0x3c0`) with the whole
units **128ths at `+0x888`** (th10 hundredths at `+0x3cc`), the clamp the same ±184 and 32 to 432; the speeds at
`+0x890` in 128ths (th10 `+0x3d4`, hundredths); the box's corners `+0x8cc`/`+0x8d8` and half `+0x8e4` (th10
`+0x404`/`+0x410`, `+0x41c`), the same 1.0 and 1.75; the state at `+0x928` (th10 `+0x458`) with the same values; the
timers' counts `+0x934`, `+0x948` and `+0x95c` (th10 `+0x464`, `+0x478`, `+0x48c`); the frames nothing hits it for at
`+0x8bb4` (th10 `+0x4310`); focused `+0x8d20` (th10 `+0x4474`); options 8 of `0xe4` from `+0x7570` (th10 from `+0x32d4`,
`0x98`). The circle test is new — round bullets and **every enemy's body**, which th10 tested as a box — with its graze
of `max(r / 2.5, 40)`; the lasers graze within the player's box times 16 (th10 within 24 of it); a bullet grazes once;
and a player flag every test refuses is new. The hit's 6, the death's 180, the coming back's 280 and the deathbomb's
window of 8 frames are th10's; **the power a death takes is the top, `[0x4a5748]`** (th10 `0x40`); and the enemy
manager counts the misses and the bombs (`+0x10`, `+0x14`), where th10 kept no count.

## The bullets

**`[0x4a8d68]`**, `0x46d678` bytes (0x408710, stored at 0x40877f): `0x7d1` bullets of `0x910` bytes from `+0x64` (the
vector constructor, 0x408390), the walk taking `0x7d0` of them (0x408d38), and how many it kept at `+0x5c` (0x408d0b).
Its calc is at `0x14` (0x4084cd), `0x408d00`, each bullet `0x408900` with the bullet in `ebx`.

| | |
| --- | --- |
| `+0x0` | its flags: `0x2` tested against the player (0x408b9c), `0x4` grazed, `0x8` to be destroyed (0x408903), `0x10` round — the circle test, with `+0x45c` its radius (0x408ba4) |
| `+0x8` | its sprite; `+0x3bc`, the sprite's variable, ends the spawn (state 2 → 1, 0x4089be on) |
| `+0x43c` | where it is, three floats |
| `+0x448` | **its step a frame**, three floats, added times `[0x4a7948]` (0x408b4c through 0x408b97), half that in states 2 and 3 (0x408939, 0x4089be) |
| `+0x45c` | **its box**, a width and a height (or the radius at `+0x45c`), halved by the test (0x408ba8) |
| `+0x464` | its age's timer, the count `+0x468` (0x408d90 through 0x408db7) |
| `+0x4b2` | **its state**, a `u16`: 0 free, **1 active**, 2 coming out, 3 cancelled (0x408916 through 0x408933; 3 written at 0x408bda) |

A bullet is tested in state 1 alone (0x408a57). The whole walk skips each bullet's update while the game object's
`+0x60` has both `0x2` and `0x400` (0x408d4a through 0x408d5f). th10's bullets were `0x7f0` bytes from `+0x60`, the
state at `+0x446`, the step at `+0x3c0`, the box at `+0x3f0` and the place at `+0x3b4`.

## The lasers

**`[0x4a8e94]`**, `0x47c` bytes (0x424940, stored at 0x424963), its calc at `0x13` (0x4249e3): a list whose head is a
laser of its own at `+0x10` (0x424941), **the first real one at `[+0x18]`**, each one's next at `+0x8` and the one
before at `+0x4`; the tail at `+0x450`, **the count at `+0x454`**, at most `0x100` (`0xff < count` refuses, 0x424df0),
and the next id at `+0x458`, from `0x10000`. 0x424c10 walks them: a laser whose `+0xc` is 1 is taken off; otherwise its
`vtable+0x8`, the calc, is run.

| kind | vtable | made | calc | starts in |
| --- | --- | --- | --- | --- |
| `LaserLineInf`, travels | `0x494a64` | `0xe8c` bytes, kind 0 (0x424ea0) | `0x425cc0` | phase 2 (0x4250b0) |
| `LaserInfiniteInf`, stays | `0x494abc` | `0xeb0` bytes, kind 1 (0x424ee0) | `0x4272c0` | phase 3, width `2.0` (0x4270f0) |

`LaserDataInf` (`0x494a0c`) is the base, the head's kind. **There are only these two**: the strings name `laser.cpp`'s
`LaserInf`, `LaserLineInf` and `LaserInfiniteInf` (0x49498c through 0x4949d8), and no curved laser.

A laser's own words: `+0xc` its phase, `+0x14` its timer (its count `+0x18`), `+0x3c` its origin (three floats), `+0x54`
its angle, `+0x58` its length, `+0x5c` its width, `+0x60` the length's speed, `+0x68` a byte set to cancel it, `+0x6c`
its id.

- **The one that travels** (0x425cc0): the length grows by `+0x60` times the speed to its most, `+0x450`, and then the
  origin moves by `+0x48`; tested where the length is above 16 and the width above 3 (0x425f94, 0x425ff6).
- **The one that stays** (0x4272c0): the length grows to `+0x460`, the angle turns by `+0x45c` a frame, and the origin
  follows the boss slot where bit 0 of `+0x490` is set. Its phases (`+0xc`, the switch at 0x4273bf, its table
  `0x427648`): **3 the warning line**, untested, until the timer reaches `+0x470` (0x4273c9); **4 widening** to `+0x468`
  over `+0x474` frames; **2 its whole width** until `+0x478` (0x427417); **5 shrinking** over `+0x47c`, and then gone.
  Tested in 4 and 2, the length above 16 (0x42747e on).

Both hand the test the stretch from a tenth of the length on, eight tenths long (`length / 10`, `length * 4 / 5`), and
a width the test halves: `w / 2` where `w < 32`, and from 32 on `w - (w + 16) / 2` for the one that travels (0x425ff0
through 0x426044) and `w - (w + 16) / 3` for the one that stays (0x4274da through 0x42752e) — th10's formulas. A hit
(1) cuts the laser about the player through its `vtable+0x18` with a 32-by-32 box (0x42606f, 0x427558 through
0x427577); a graze counts every third frame, the stayer's adding one to `[0x4a5754]` (0x427560 on, 0x4275e6). th10's
vtables were `0x46da60` and `0x46da10`, its count at `+0x438`.

## The enemies, the boss and the card

**`[0x4a8d7c]`, the enemy manager**, `0x78` bytes (0x411070, stored at 0x411095), its calc at `0x12` (0x410c71),
`0x4111a0` → `0x411120`:

| | |
| --- | --- |
| `+0x10` | **the misses**: one higher at every death (0x431247) |
| `+0x14` | **the bombs**: one higher at every bomb's start (0x406cee, 0x4078bf, 0x45de20) |
| `+0x18` | put at nought at a death (0x43124a) |
| `+0x1c` | **the boss slot 0**, of an array the `.ecl` writes (`mov [ecx+eax*4+0x1c],ebx` at 0x4114d4; 0x414ed0 through 0x414ee4, the index into the enemy's `+0x25c8`, its flag `0x80000`) |
| `+0x50` | the manager's timer (0x410be0) |
| `+0x64` | the `.ecl` (`EclResourceInf`) |
| `+0x68`, `+0x6c` | **the list's head and tail** |
| `+0x70` | **how many enemies are on it** (0x4108f0, 0x410b60); not nought is "an enemy is up" for the shot and the focus |

**An enemy** is `0x2678` bytes (`EnemyInf`, 0x4108f0 → 0x4111c0); its list element is at `+0x11a4` — the enemy, the
next element and the one before (0x4111c0, 0x410b60). Its update `0x411750` is handed `enemy + 0x103c`, and the
offsets below are the enemy's own:

| | |
| --- | --- |
| `+0x8` | its main script: its time, a float, at `+0x8`, where it is at `+0xc` (0x4111c0) |
| `+0x1070` | **where it is**, three floats (0x411750's `+0x34`; the boss's read by the card's start at 0x40c89d) |
| `+0x110c`, `+0x1110` | the box the player's shots hit |
| `+0x1114` | **its body's size**: the body's test is the circle of radius `+0x1114 * 0.5` (0x411e2c through 0x411e48) |
| `+0x1190` to `+0x11a0` | **the frames its current attack has run**, a timer: the count of the frame before at `+0x1190`, **the count at `+0x1194`**, the count as a float at `+0x1198`, the game's speed `0x4a7948` it steps by at `+0x119c` and its flags at `+0x11a0`. Put at nought as the enemy is made (0x4112e1 through 0x411324); **a frame on at the end of every update** (0x412756 through 0x4127c5: one on and the float one on where the speed is 1, the float stepped by the speed and taken whole otherwise); and **back at nought at each change of attack** by `0x417b40`, which the update asks for the next attack's script (0x411cdc, 0x411da6): where the life is down to the next of the eight marks at `+0x25d0` (stride `0x10`, the mark at `+0x25d0`, its time at `+0x25d4`, its scripts at `+0x25d8` and `+0x25dc`) it is put at the mark and the timer at nought (0x417bb8, 0x417bf8), and where the count has reached the mark's time (0x417c84) the same (0x417cf2) with flag `0x100000` put on; the panel's seconds, `+0x4440` and `+0x4444` of `[0x4a8d84]`, are that time less the count (0x417c34 through 0x417c7e). Read for the ECL variable `-9988` as the float (0x416fb1 the count, 0x4174b0's `0xffffd8fc` the float). th10's at `+0x1158` to `+0x1168`, reset at 0x41284c |
| `+0x252c`, `+0x2530` | **its life**, and the life it started with (0x4108f0; 0x4100e0 takes damage off it) |
| `+0x25ac` | the frames its body is off for, counting down (0x411e01) |
| `+0x25bc` | **its flags**: `0x2` and `0x20` skip the body's test (`test al,0x22` at 0x411df9), `0x400000` skips it too (0x411e0e), `0x80000` a boss, `0x200` a body that grazes (every third frame), `0x1000` seen on the field, `0x4000` updated this frame |
| `+0x2674` | a hit callback the `.ecl` can put in place of the body's test, from the table at `0x4a3da0` (0x414c0a, 0x414c11): 1 is `0x418050`, 32 centred rectangles in a ring about the enemy, 12 long and `life * 16 / max + 4` wide |

**The card `[0x4a8d6c]`**, `0x924` bytes (0x40bcc0, stored at 0x40bcf4), its calc at `0x16` (0x40bd3a). `0x40c650`
starts one (called by the script's instruction at 0x415bf1; `eax` the name, the number pushed): **its number at
`+0x8dc`**, its name at `+0x89c`, its timer `+0x888` (count `+0x88c`) from nought — **the frames the card has run**, stepped by
`0x459270` at 0x40c147 in its calc `0x40c630` → `0x40c0a0`, which steps nothing where bit 0 is off (0x40c0a9) — and **`+0x8e0` with `0x3` put on
and `0x18` taken off** (`and eax,0xffffffe7; or eax,0x3` at 0x40c6d8, 0x40c6db): **`0x1` a card running** (0x406160),
`0x2` its capture still to be had, `0x20` a bomb used during it, and `0x10` and `0x8` others. `+0x8e4` is its bonus,
starting at `(difficulty + stage) * 1,000,000` (0x40c650). `0x40cd60` is its capture, where `0x2` still stands
(0x40cdc2). What either counts in the score file is in *The score file*.

th10's manager was `0x68` bytes, the boss slot at `+0x10`, the list at `+0x58` and the count at `+0x60`; its enemy
`0x2518` bytes with its place at `+0x1068`, body `+0x10f4`, life `+0x23fc`, flags `+0x2480` with `0x12` skipping the
body, and the body off for `+0x2470`; its card `0x37b0` bytes, its number at `+0x3788` and flags at `+0x378c`.

## The items

**`[0x4a8e90]`**, `0x265e70` bytes (0x4231e0, stored at 0x423210), its calc at `0x15`: **`0x896` items of `0x478`
bytes from `+0x14`**. The first `0x96` take every type but 8 (0x424230, the search stopping past `0x95`); type 8, the
small one fire leaves, goes in the `0x800` after them in turn, `+0x265e68` the next (0x424230's first half; see *The
item manager's ring*).

| | |
| --- | --- |
| `+0x434` | **where it is**, three floats; `+0x440` its velocity |
| `+0x454` | its age (the timer from `+0x450`) |
| `+0x464` | **its state**: 0 free, **1 falling**, 2 thrown up and falling, **3 flying to the player** at the `.sht`'s `+0x8`, **4 pulled in** by the pull box at a third of that, 5 waiting `+0x474` frames to come out (type 8) |
| `+0x468` | **its type** (`+0x46c` the sprite's): 1 and 10 power, 4 and 11 big power, 6 full power — each through 0x420a00 — 2 a point, 3 a fixed score, 5 a piece of a life, 7 a life, 8 the small one fire leaves (10 points), 9 1000 points (what a power item becomes at full power, 0x424333) |

In state 1 an item flies to the player (state 3) unless the player is in state 2 or 4, or is below the line (`y >=
128.0`) with the item younger than 40 frames or `[0x4a56f4] < 10000` (0x42361a through 0x42365c); during a
conversation every falling one does. A falling item past `y = 472` is gone (0x423744 on). It is collected inside the
player's `+0x8bcc` box, and pulled in (state 4) inside `+0x8be4`'s box focused or `+0x8bfc`'s unfocused (0x423a17
through 0x423e28). th10's items were `0x3f0` bytes, their state at `+0x3dc`, type `+0x3e0` and place `+0x3ac`.

## The pause and the end of a run

**`[0x4a8eb0]`, the pause object, runs the pause menu, a run's end, its name entries and its replay offer**: `0x2e0`
bytes (0x42bea1), built by 0x42bea0 through 0x42bbf0, its pointer stored at 0x42bc4f, taken down by 0x42bdc0 (0x42be90
writes the pointer at nought). Its calc job is `0x42c600` at priority 9 and its draw job `0x42c610` at `0x3a` (0x42bc9a,
0x42bcef). Its state is `+0x4`; `+0x10` is a timer whose count is `+0x14`; `+0x38` the cursor and `+0x40` the count of
items; `+0xc8` a list of items (its count at `+0x10c`); `+0x1f0` how many letters are typed; `+0x1f4` the run cleared;
`+0x1f8` the score not placed in the ranking; `+0x1fc` set on a replay's own pause; `+0x204` 25 replay headers;
`+0x2cc` the name being entered; `+0x2d8` the game speed kept.

**0x42bf30 dispatches on the state** (`cmp eax,0x1a` at 0x42bf37, the byte table at `0x42bfc4`, the jump table at
`0x42bfb4`):

| state | handler |
| --- | --- |
| 0 | 0x42bf4a: opens the pause menu, 0x42c620, where the run is not the demo (`0x4a5758 & 0x20`, 0x42bf4a), the pause key is down (`[0x4c92b4] & 0x100`, 0x42bf53) or bit `0x10` of `[0x4c3810]` is set (0x42bf5f) — the bit a successful `Reset` sets and a frame answering `0` takes off (0x4458b8) — the game object's calc job is set to run (0x42f540: `[[game+0x8]+0x4] & 2`), and the game object's frames are 30 or more (`cmp [ecx+0x14],0x1e` at 0x42bf77) |
| 1 to 11 | **the pause menu**, 0x42c8d0 (its table `0x42d524` covers 1 to 10; 11 has no entry, 0x42c8ea) |
| 12 to 19 | **the end of a run lost**, 0x42d9d0 (its table `0x42e618`) |
| 20 to 26 | **the end of a run won**, 0x42e6b0 (its table `0x42f31c`) |
| 27 (`0x1b`) | the state once 19 or 26 has carried out its item (0x42e592, 0x42f2bf): past the byte table, so nothing is stepped (0x42bf3a) |

**Opening any of them** writes the state, starts both timers, sets **bit `0x10` of the game object's `+0x60`**
(0x42c684, 0x42c7cb, 0x42d601, 0x42d73f) and keeps the game speed `0x4a7948` in `+0x2d8`, putting 1.0 in its place.
**The pause menu's resume, `0x42c880`**, takes nothing: the pause flag taken off (0x42c885), the music let go on and the
game speed put back (0x42c880 through 0x42c8cd). A run's end puts the speed back through 0x42d7f0 and does not take
`0x10` off.

| opened by | state | from |
| --- | --- | --- |
| 0x42c620 | 1 | the pause key, through state 0 |
| 0x42c760 | 2 | **a replay's end**: lives below nought in a replay (0x431070, at 0x43149f), the replay's frames run out (0x4361f7), the stage-ending instruction of a replay (0x41ebc2), the ending's wait in a replay (0x41b413) |
| 0x42d560 | 13, or 21 for a practice (`lea edx,[edx*8+0xd]` at 0x42d5b0) | **lives below nought**, `0x4a5718 < 0` (0x431070, at 0x431411); `+0x1f4` at 0 (0x42d67a) |
| 0x42d6b0 | 21 (0x42d6e4) | **a stage won that ends the run**: the Extra's (0x41ed54) and a practice's (0x41eba3); `+0x1f4` at 1 |

Both 0x42d560 and 0x42d6b0 write the scene wanted as 4 straight away where the game object's `+0x74` is 1, a replay
played (0x42d567, 0x42d6b7), and queue `bgm/th10_17.wav` (see *The sound*).

### The pause menu, 0x42c8d0

| state | at | what |
| --- | --- | --- |
| 1 | 0x42c8fa | after ten frames: state 3, four items, the cursor on 0; for a replay played (`[game+0x74]`, 0x42c91f) item 2 put on the list at `+0xc8` |
| 2 | 0x42c957 | after ten frames: state 3, four items, items 2 and 0 put on that list, the cursor on 1, `+0x1fc` at 1 |
| 3 | 0x42c9f2 | **the menu**, read from its first frame: up and down (`0x4c92b4 \| 0x4c92b0`) move the cursor, and a decide (0x42ca55) takes it: 0 → state 4; 1 → 5, or 4 for a replay played (0x42cacc); 2 → 7 (0x42caea); 3 → 5, or 4 for a replay played (0x42cb22). Where `+0x1fc` is nought, R (`0x200000`) takes state 4 with the cursor on 3 (0x42cb38 through 0x42cb94) and the pause key state 4 with the cursor on 0 (0x42cda0 through 0x42cde8); Q (`0x10000`) takes state 4 with the cursor on 1 (0x42cba6 through 0x42cc02) |
| 4 | 0x42d460 | **the item carried out twelve frames on** (`cmp [ebp+0x14],0xc`), the state put at nought (0x42d470): item 0 resumes (0x42c880, at 0x42d508); 1 the title (0x40dea0 with 4, 0x42d4ec); 3 the scene wanted as 10, or 11 for a replay played (0x42d4b5); item 2 does nothing here |
| 5, 7 | 0x42cc1b | a yes or no on a cursor pushed by 0x4590f0, from frame 20, read from frame 30: yes → 6 from 5, 8 from 7 (0x42cd88 through 0x42cd93); no → 6 (0x42cd64) |
| 6 | 0x42d32c | after twenty frames, the pushed cursor popped (0x459130): yes → state 4, the menu's item carried out (0x42d393); no → back to 3 (0x42d365) |
| 8 | 0x42d3be | after twenty frames: state 9, and the 25 slots read, `th11_01.rpy` to `th11_25.rpy` (`0x495384`, 0x42d3ec, `+0x40 = 0x19`) |
| 9 | 0x42ce09 | **the replay's slots**: a decide → state 10, the name entry, with the date written into the replay's header and its stage — the stage, or 8 for a run cleared and not practised (0x42cec4); a cancel → state 4 with the cursor on 1, the title (0x42cfc1, then 0x42d027) |
| 10 | 0x42d078 | **the replay's name**, 13 columns over `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=.,!?@:;[]()_/{}\|~^#$%&*` and three places past it — `0x58` a space, `0x59` back, `0x5a` the end; the end saves the slot (0x436420 with 1) and goes back to 9 (0x42d2cc), a cancel on an empty name back to 9 (0x42d1d1) |

So **item 0 is the resume, 1 the title, 2 the replay's save and 3 the start-over**; 1 and 3 ask yes or no unless a
replay is being played, and 2 always asks.

### The end of a run lost, 0x42d9d0

| state | at | waits | what |
| --- | --- | --- | --- |
| 12 | 0x42d9fc | 10 frames, then a decide | → 13. **No store of 12 was found** (every `mov [reg+0x4],…` in the exe) |
| 13 | 0x42da94 | 10 | the score into the high score, 0x42ba20; for a run not practised (0x42daa3) the ranking, 0x42d850, then the name entry, state 18, where the score placed (`+0x1f8` at nought, 0x42db02 through 0x42db11), and otherwise **the menu, four items, the cursor on 0** (0x42db4e through 0x42db7c) |
| 14 | 0x42dbaf | none | **the menu**: up and down; a decide sets 19 (0x42dc4e), and for item 2 the replay's slots, state 16 (0x42dccb); a cancel puts the cursor on 1 |
| 15 | 0x42e600 | — | nothing |
| 16, 17 | 0x42dd80, 0x42dfe4 | 10 | the replay's slots and its name, as the pause menu's 9 and 10; the save is 0x436420 with 0; a cancel from the slots → 14 (0x42df44) |
| 18 | 0x42e299 | 10 | **the ranking's name entry**; its end copies the name into the ranking (the score object's `+0x1e` on, 0x42e48e) and goes to **the menu standing on item 0** (0x42e4d4 through 0x42e54f) |
| 19 | 0x42e582 | 12 | **the item carried out**, state 27 (0x42e592): item 0 the scene wanted as **14, the continue, where `+0x1f4` is nought, and 10, the start-over, where it is set** (0x42e5a7 through 0x42e5bb); 1 and 2 the title (0x42e5dc); 3 the start-over, 10 (0x42e5f6) |

**Its menu: 0 the continue, 1 the title, 2 the replay's save, 3 the start-over.** No test of the difficulty was found on
the way to the continue: 0x42d560, state 13 and state 14 read none.

### The end of a run won, 0x42e6b0

| state | at | waits | what |
| --- | --- | --- | --- |
| 20 | 0x42e6dc | 10, then a decide | → 21. **No store of 20 was found** either |
| 21 | 0x42e774 | 10 | 0x42ba20; **a practice** (0x42e783): the practice's score kept (see *The score file*) and the menu, state 22, three items, the cursor on 0 (0x42e7f7); **otherwise** 0x42d850, then the name entry, state 25, where the score placed (0x42e849), and where it did not **state 14, the menu of a run lost, three items, the cursor on 0** (0x42e886 through 0x42e8b4) |
| 22 | 0x42e8e7 | none | **the menu**: a decide sets 26 (0x42e986), and for item 1 the replay's slots, state 23 (0x42e9fd); a cancel puts the cursor on 0 |
| 23, 24 | 0x42eab6, 0x42ed14 | 10 | the replay's slots and its name; a cancel from the slots → 22 (0x42ec74) |
| 25 | 0x42efc9 | 10 | **the ranking's name entry**; its end → **the menu, state 22, standing on item 0** (0x42f204 through 0x42f27f) |
| 26 | 0x42f2b2 | 12 | **the item carried out**, state 27 (0x42f2bf): 0 and 1 the title (0x42f301); 2 the start-over, 10 (0x42f2db) |

**Its menu: 0 the title, 1 the replay's save, 2 the start-over.**

### `+0x1f4`, `+0x1f8`, and the Extra's stage 8

**`+0x1f4` is the run cleared** (0x42d6b0 writes 1, 0x42d560 nought), and **`+0x1f8` the score not placed**: 0x42d850
writes 1 where 0x42ba80 answers below nought (0x42d8ce → 0x42d9c0), and nought otherwise.

**0x42d850 moves a cleared Extra past itself and does not move it back**, as th10's 0x423570 does: with `[0x4a5728]` at
7 and `+0x1f4` set it writes 8 into the stage and the stage the run started at and points `[0x4a8ec8]` at `0x4a3a28`
(0x42d856 through 0x42d87b), calls 0x42ba80 (0x42d8a0), and puts back 7 and `0x4a39e8` only where the stage still reads
7 (`cmp ds:0x4a5728,edi` at 0x42d8a5) — which it does not, 0x42ba80 writing no stage. So the stage stays 8. The ranking
keeps that 8 as the entry's stage (0x42ba80 writes `[0x4a5728]` into the entry's `+0x4`).

**Where a cleared Extra goes from there**:

- its score placed → the name entry → the menu of state 22 **on item 0, the title**: a decide is harmless, and item 2,
  the start-over, wants scene 10;
- its score not placed → **state 14 on item 0** (0x42e886 through 0x42e8b4), whose item 0 with `+0x1f4` set is the
  scene wanted as 10 (0x42e5a7 through 0x42e5bb) — **one decide**.

**Scene 10 builds the stage the run started at**, 8 (0x42b2b4 through 0x42b2c8), and `0x4a3a28` is no stage's record
(see *The stage records*), so the build hands the stage object's set-up a code address for a `.std` name (0x41fc38
through 0x41fc42). The stage object's set-up logs `ステージデータが読み込めません。データが壊れています` (`0x4938e8`, at
0x4024e8) where its read fails, th10's failure.

**The cleared main game does it right**: the ranking for a run that reached the ending is the title's screen 14
(0x43f6b0), which writes 8, calls 0x42ba80 (0x43f802), and then puts the stage and its record back to 0 and `0x4a3828`
unconditionally.

### The continue

**A continue is scene 14**, from item 0 of a lost run's menu. Before the build the game object's teardown 0x41fe10 adds
one to the continues used `0x4a573c`, held at 9, and sets the run's flag `0x8` — only where the stage is not the one the
run started at; on the stage it started at, it sets `0x1` and counts nothing (see *The game object*). The build then
puts the score at nought, the lives at 2 for a run not practised, the continues used at nought unless flag `0x8` is set,
and `0x4a5744` at `-0x200` with flag `0x8` (see *The run's numbers*). **The ending reads the continues**: a run with none
plays `e06.msg` to `e11.msg`, one with any `e00.msg` to `e05.msg` (`+0x1c` = `character * 3 + partner`, plus 6 where
`0x4a573c` is nought, 0x40ec90). The ranking keeps them in an entry's `+0x5` (0x42ba80).

### The name entries

Both use the same thirteen-column grid of `0x5b` places (`+0x118 = 0x5b`, `+0x110` the place), eight letters at
`+0x2cc`, started from the last name entered — the score object's `+0x2dde0` (`ST+0xc`) — with trailing spaces off the
count `+0x1f0`; each end copies the name back into `+0x2dde0`.

- **The ranking's**: 0x42d850 finds the place through 0x42ba80 and puts the cursor on it; states 18 and 25 write the
  name into `record + 0x16 + (place + difficulty * 10) * 0x1c` (the score object's `+0x1e`, 0x42e48e and 0x42f1be).
- **The replay's**: states 10, 17 and 24; the slot is the cursor, the file `th11_%.2d.rpy` of the cursor plus one,
  written by 0x436420 into `replay/%s` (`0x4955a8`, at 0x436501).
- **The clear's own** is the title's screen 14, over the same grid and end place `0x5a`, its fields at the title's
  `+0x58d4`, `+0x58e0` and `+0x58ec` (see *The front end*); its decide copies the name into the ranking and puts the
  screen's third step up, which five frames on is screen 15.

### The ending and the staff roll

**`[0x4a8d78]` is the ending**, `0x28` bytes, built by 0x40f080 for scene 15; its set-up 0x40ec90 chooses `e%02d.msg`
from the table at `0x4a3da8` (read at 0x40ee18), marks the ending seen in the score object's `+0x2ddea + ending`
(`ST+0x16`, and `| 0x10` above Easy) and `+0x2ddf6` (`ST+0x22`) once an ending of six or more is seen. Its calc job
0x40f120 writes the scene wanted as 16 (or 2) once 0x40f4a0 answers it is done. **The ending is reached from the
stage's script**: a stage 6 won, not in a replay, sets bit `0x10` of `[0x4a8d84]+0x4420` (0x41ecb0, in the case of
instruction 21 of 0x41d380), and 380 frames on the scene wanted is 15 (0x41b3fc through 0x41b430), or the replay's end
where a replay is played (0x41b413). `staff.msg` is the thirteenth entry of the ending's table (`0x4a3dd8`).

**Against th10**: three handlers and 27 states where th10 has two and 13 — the pause menu 1 to 10 (th10 1 to 5), a lost
run 12 to 19, a won run 20 to 26, and 27 for done (th10 `0x15`). **The pause menu has four items** with the replay's save
among them, which th10's has not, and a replay's own pause (state 2). **The end of a run is split in two**: a lost run's
menu of four with **the continue** first, a won run's of three with the title first, where th10 has one menu of three
(playing on, the save, the title); and a won run whose score does not place lands in the lost run's menu, its item 0
the start-over. **The name entry hands back to item 0 in both**, where th10's stood on item 0 against its first menu's
item 2. **The Extra's stage 8 survives by the same defect**, reached in one decide where the Extra's score does not
place, and th11's "record" for stage 8 is code where th10's named no `.std`. th10's `+0x1e4` and `+0x1e8` are
`+0x1f4` and `+0x1f8`. The pause opens only from the game object's 30th frame (0x42bf77).

## The score file, `scoreth11.dat`

**The same scheme as th10's** — `score_file::decode` reads it with four constants changed:

- the header, as 0x4378f0 checks it: `TH11` (`cmp [eax],0x31314854` at 0x437900), a version word of **4** at `+0x8`
  (0x43790c), the packed length at `+0x10`, the unpacked at `+0x14`, the packed bytes from `+0x18`. The writer puts the
  file's whole length, packed plus `0x18`, at `+0x4`, and `0x100` at `+0xc` (0x437a60);
- deciphered by **0x4581c0** with a key of `0xac` (`mov al,0xac` at 0x437926) stepping by `0x35` over blocks of `0x10`
  (0x43791f, 0x43791d), and enciphered by 0x4582e0 with the same three (0x437bb0 through 0x437bb6);
- unpacked by **0x4426c0** (0x437952) and packed by 0x4423a0 (0x437b98). Their window and bit widths were not read off
  the code; th10's LZSS (a `0x2000` window, thirteen bits of offset, four of length over three) unpacks the install's
  file to exactly the length its header names, and every record's sum then adds up — 2026-10-07: packed `0x864a`,
  unpacked `0x2e214`, eight records all sound.

**The records**, each two letters, a version word of nought, the sum of its bytes from `+0x8` (0x437500), and its size:

- `CR`, **`0x68d4` bytes**, one per shot type, its number at `+0xc`: `character * 3 + partner` for 0 to 5, and 6 for
  all of them (the writer 0x437a70 writes each `CR`'s index into its `+0xc` and sums it again, 0x437ad0 through
  0x437ae1). 0x4378f0 copies a sound record to `object + 0x8 + number * 0x68d4` with **no test of the number** (0x43799e
  through 0x4379b2).
- `ST`, `0x448` bytes, at `object + 0x2ddd4` (0x4379e7).
- A record of any other letters ends the walk (0x4379c4 → 0x437a0f).

**A `CR` record**:

| | |
| --- | --- |
| `+0x10` | the ranking, 10 entries of `0x1c` a difficulty, five difficulties: `+0x0` the score over ten, `+0x4` the stage reached (8 for a cleared Extra), `+0x5` the continues, `+0x6` the name, nine bytes, `+0x10` the date (`_time64`), `+0x18` a float, `100 - a / b * 100` of two doubles of `[0x4a8d80]` (0x42ba80) |
| `+0x588` | one added at every build with `+0x564` set, not for a replay, held at 99999 (0x41facd through 0x41fae1, the object's `+0x590`) |
| `+0x590` | one per difficulty, added where stage 6 or the Extra is won (0x41ece8, 0x41ed85, the object's `+0x598`) |
| `+0x59c` | **the practice**, 25 entries of 8 bytes to `+0x664`, the entry `stage + difficulty * 6` with the stage counted from 1: `+0x0` the score, raised to the score at a practice's end (0x42dad8, 0x42e7bc); `+0x4` and `+0x5` set to 1 where that stage is won, not for a replay or the Extra (`cmp ecx,4` at 0x41eb39; 0x41eb92 and 0x41eb61, the object's `+0x5a8` and `+0x5a9`). **The practice's select reads `+0x5`**, for the cursor `c` as entry `c + 1 + difficulty * 6` (`cmp byte [ecx+edx*8+0x5b1],0` at 0x43d80a), the record being `object + 0x8 + (character * 3 + partner) * 0x68d4` (0x41eb53, 0x43d7f6) |
| `+0x664` | **the cards**, 175 of `0x90`: the name, `0x80` bytes; `+0x80` the captures; `+0x84` the attempts; `+0x88` the number; `+0x8c` the difficulty, 0 Easy to 4 Extra (0x437550 writes the numbers, from `+0x6ec`; `0x664 + 175 * 0x90 = 0x68d4`) |

**The score object `[0x4a8ebc]`**, `0x2e21c` bytes (0x437822): `+0x0` the file as read, `+0x4` the unpacked bytes, `+0x8`
the seven `CR` records and `+0x2ddd4` the `ST` (th10's at `+0x1d86c`). Of the `ST`: `+0xc` the last name entered,
`+0x16` twelve endings seen, `+0x22` the flag of an ending of six or more seen, `+0x26` on a byte per song heard
(0x42a18d, 0x42a219; the title's `+0x26` at 0x439a4b; `+0x37` at a run's end, 0x42d673 and 0x42d7b3), and `+0x46` the
`0x200` draws of the generator.

**The endings seen light the Extra**: a byte per ending at `object + 0x2ddea + ending` (`ST+0x16`), the ending being
`character * 3 + partner`, plus 6 for a run with no continue (0x40edb5 through 0x40edc0). The ending's set-up writes
**1**, then `| 0x10` where the difficulty is Normal or above (0x40ede8 through 0x40edfa). **The main menu offers the
Extra where any of `object + 0x2ddf0` … `+0x2ddf5` (`ST+0x1c` … `+0x21`, endings 6 to 11) has bit `0x10`** (0x439350,
called at 0x43a0c3 and four more); the selects test a shot type's byte the same way (0x439340, 0x43cc17, 0x43d0ef). The
code's `CLEARS` is `0x2ddea..0x2ddf6`.

**A shot type's record is made empty by `0x437550`**, the record in `eax`, plain `ret` (0x437657): `memset` of
`0x68d4`, `CR`, the size `0x68d4`, the ranking's ten defaults per difficulty and every card numbered (0x437550 through
0x437651); it is called for each of the seven records before the file is parsed (0x437793 through 0x4377bc). `ST` is
made empty by 0x437660 (`eax` the record). th10's was 0x42acb0, `stdcall`.

**Read once**, at the startup: the loading object's load, 0x4289e0, calls 0x437800 last (0x428b28), which builds the
object, reads `scoreth11.dat` (`0x4957b8`, at 0x43775a) through 0x458400 and parses it (0x4377bf). A file that does not
read leaves every record at the defaults 0x437660 and 0x437550 write — **and is never written**: 0x458400's null goes
into the object's `+0x0` (`mov [ebx],eax` at 0x437764), and 0x437a70 returns at once where that is null (`cmp
dword [ebx],0` at 0x437a7a, its first test). A
file that opens and does not parse is: 0x4378f0 lays nothing down for bytes without `TH11` and version 4 at their head
(`0x31314854`, `+0x8 == 4`) or for a record whose sum is not its own, `+0x0` is not null, and the first write puts the
defaults over the file.

**Written by 0x437a70**, the whole object every time: first thing in the game object's teardown 0x41fe10 (0x41fe17) — so
at every scene change that leaves or rebuilds a run — and in the loading object's teardown 0x428c30 (0x428d81) as the game
leaves.

**A card's start counts its attempt**: 0x40c650 writes the number into the card's `+0x8dc`, puts `0x3` on in
`+0x8e0`, and where no replay is played (`[[0x4a8eb8]+0x10] != 1`, 0x40c6e4) copies the name into the card of the shot
type's record and of record 6 (`object + 0x66c + number * 0x90`, and `+ 0x27b64`) and adds one to each one's `+0x84`,
held at 99999. **Its capture**: 0x40cd60, where `0x2` still stands, adds the bonus over ten to the score and one to each
one's `+0x80`, under the same test.

**Which card a number is**, read off the install's file (2026-10-07): the names any of the seven records holds. A name
is written only once a card has been attempted, so a card no shot type has met reads empty. E, N, H, L and X are the
record's `+0x8c`, 0 to 4, and `同` is the name before it. Cards 46 to 117 come in blocks of twelve by shot type, each
block named only in its own record and in record 6 (the `-- record` notes).

```text
  0 H 怪奇「釣瓶落としの怪」        1 L 怪奇「釣瓶落としの怪」
  2 E 罠符「キャプチャーウェブ」    3 N 罠符「キャプチャーウェブ」
  4 H 蜘蛛「石窟の蜘蛛の巣」        5 L 蜘蛛「石窟の蜘蛛の巣」
  6 E 瘴符「フィルドミアズマ」      7 N 瘴符「フィルドミアズマ」
  8 H 瘴気「原因不明の熱病」        9 L 瘴気「原因不明の熱病」
 10 E 妬符「グリーンアイドモンスター」  11 N 妬符「グリーンアイドモンスター」
 12 H 嫉妬「緑色の眼をした見えない怪物」  13 L 嫉妬「緑色の眼をした見えない怪物」
 14 E 花咲爺「華やかなる仁者への嫉妬」  15 N 花咲爺「華やかなる仁者への嫉妬」
 16 H 花咲爺「シロの灰」            17 L 花咲爺「シロの灰」
 18 E 舌切雀「謙虚なる富者への片恨」  19 N 舌切雀「謙虚なる富者への片恨」
 20 H 舌切雀「大きな葛籠と小さな葛籠」  21 L 舌切雀「大きな葛籠と小さな葛籠」
 22 E 恨符「丑の刻参り」            23 N 恨符「丑の刻参り」
 24 H 恨符「丑の刻参り七日目」      25 L 恨符「丑の刻参り七日目」
 26 E 鬼符「怪力乱神」  27 N 鬼符「怪力乱神」  28 H 鬼符「怪力乱神」  29 L 鬼符「怪力乱神」
 30 E 怪輪「地獄の苦輪」            31 N 怪輪「地獄の苦輪」
 32 H 枷符「咎人の外さぬ枷」        33 L 枷符「咎人の外さぬ枷」
 34 E 力業「大江山嵐」              35 N 力業「大江山嵐」
 36 H 力業「大江山颪」              37 L 力業「大江山颪」
 38 E 四天王奥義「三歩必殺」  39 N 同  40 H 同  41 L 同
 42 E 想起「テリブルスーヴニール」  43 N 想起「テリブルスーヴニール」
 44 H 想起「恐怖催眠術」            45 L 想起「恐怖催眠術」
 46 E 想起「二重黒死蝶」  47 N 同    48 H (empty)  49 L (empty)     -- record 0
 50 E 想起「飛行虫ネスト」  51 N 同  52 H (empty)  53 L (empty)     -- record 0
 54 E 想起「波と粒の境界」  55 N 同  56 H (empty)  57 L (empty)     -- record 0
 58 E 想起「戸隠山投げ」  59 N 同    60 H (empty)  61 L (empty)     -- record 1
 62 E 想起「百万鬼夜行」  63 N 同    64 H (empty)  65 L (empty)     -- record 1
 66 E 想起「濛々迷霧」  67 N 同      68 H (empty)  69 L (empty)     -- record 1
 70 E 想起「風神木の葉隠れ」  71 N 同  72 H (empty)  73 L (empty)   -- record 2
 74 E 想起「天狗のマクロバースト」  75 N 同  76 H (empty)  77 L (empty)  -- record 2
 78 E 想起「鳥居つむじ風」  79 N 同  80 H (empty)  81 L (empty)     -- record 2
 82 E 想起「春の京人形」  83 N  84 H  85 L 同                      -- record 3
 86 E 想起「ストロードールカミカゼ」  87 N  88 H  89 L 同          -- record 3
 90 E 想起「リターンイナニメトネス」  91 N  92 H  93 L 同          -- record 3
 94 E 想起「マーキュリポイズン」  95 N 同  96 H (empty)  97 L 同     -- record 4
 98 E 想起「プリンセスウンディネ」  99 N 同  100 H (empty)  101 L 同  -- record 4
102 E 想起「賢者の石」  103 N 同  104 H (empty)  105 L 同           -- record 4
106 E 想起「のびーるアーム」  107 N 同  108 H (empty)  109 L 同     -- record 5
110 E 想起「河童のポロロッカ」  111 N 同  112 H (empty)  113 L 同   -- record 5
114 E 想起「光り輝く水底のトラウマ」  115 N 同  116 H (empty)  117 L 同  -- record 5
118 E 猫符「キャッツウォーク」  119 N 同
120 H 猫符「怨霊猫乱歩」  121 L 同
122 E 呪精「ゾンビフェアリー」  123 N 同
124 H 呪精「怨霊憑依妖精」  125 L 同
126 E 恨霊「スプリーンイーター」  127 N 同
128 H 屍霊「食人怨霊」  129 L 同
130 E 贖罪「旧地獄の針山」  131 N 同
132 H 贖罪「昔時の針と痛がる怨霊」  133 L 同
134 E 「死灰復燃」  135 N 同
136 H 「小悪霊復活せし」  137 L 同
138 E 妖怪「火焔の車輪」  139 N  140 H  141 L 同
142 E 核熱「ニュークリアフュージョン」  143 N 同
144 H 核熱「ニュークリアエクスカーション」  145 L 核熱「核反応制御不能」
146 E 爆符「プチフレア」  147 N 爆符「メガフレア」  148 H 爆符「ギガフレア」  149 L 爆符「ペタフレア」
150 E 焔星「フィクストスター」  151 N 同
152 H 焔星「プラネタリーレボリューション」  153 L 焔星「十凶星」
154 E 「地獄極楽メルトダウン」  155 N 同
156 H 「ヘルズトカマク」  157 L 同
158 E 「地獄の人工太陽」  159 N 同
160 H 「サブタレイニアンサン」  161 L 同
162 X 秘法「九字刺し」  163 X 奇跡「ミラクルフルーツ」  164 X 神徳「五穀豊穣ライスシャワー」
165 X 表象「夢枕にご先祖総立ち」  166 X 表象「弾幕パラノイア」  167 X 本能「イドの解放」
168 X 抑制「スーパーエゴ」  169 X 反応「妖怪ポリグラフ」  170 X 無意識「弾幕のロールシャッハ」
171 X 復燃「恋の埋火」  172 X 深層「無意識の遺伝子」  173 X 「嫌われ者のフィロソフィ」
174 X 「サブタレイニアンローズ」
```

**Against th10**: `TH11`, version 4; `CR` records of `0x68d4` bytes (th10 `0x437c`); the cards from `+0x664` with the name
first and the counts after it (th10's names every `0x90` from `+0x59c`); 175 cards (th10 110). The decipher is 0x4581c0
and the unpacker 0x4426c0 (th10 0x44b0d0, 0x435dc0). The parse does not bound a `CR` record's number.

## What a snapshot, a restore and a resume touch

### The sprite manager

**The sprite manager `[0x4c3268]`, `0x7bd894` bytes**, holds **the `.anm` files**: 32 pointers from **`+0x4350c0`** to
`+0x435140`. The load 0x454190 refuses a slot `>= 0x20` (`cmp ebp,0x20` at 0x4541c8) and stores the file at
`[sm+0x4350c0+slot*4]` (0x45423d); 0x4548d0 and 0x4549a0 walk and free the same 32 (`cmp 0x20`). th10 has 33 from
`+0x3ad06c`.

**A file is `new(0x134)`** (0x454208, zeroed at 0x45422d): `+0x0` its slot, `+0x4` its name, `+0x108` the file's bytes.

- **Its textures**: the count at `+0x10c` (0x4542a6), the array at `+0x120` (0x4542b5), **`0x14` bytes each** (`lea
  ebp,[ecx+ecx*4]` and `*4`, 0x45429e; the walks step `add esi,0x14` at 0x44538d and 0x445437). An entry: **`+0x0` the
  `IDirect3DTexture9`** (released through `[vtbl+0x8]` at 0x445374 and in 0x4549e0; `GetLevelDesc`, `[vtbl+0x44]`, at
  0x454753), `+0x4` the image's bytes and `+0x8` their size (0x4543e0, freed in 0x4549e0), `+0xc` a format word
  (0x445431), and **`+0x10` bit 0 a render target** (0x44535a, 0x4453e8). th10's were `0x10` bytes.
- **Its sprites**: the count at `+0x114` (0x4542e6), the array at `+0x118` (0x4542db), **`0x48` bytes each** (`lea
  eax,[edi+edi*8]` and `*8`, 0x4542c0; `lea eax,[eax+eax*8]` and three doublings in 0x454b10, which `rep movs` `0x12`
  dwords into it). **Each holds at `+0x8` the address of its texture's entry** — `[file+0x120] + index * 0x14`, a
  pointer into the file's own array of textures and not the `IDirect3DTexture9` (`mov [esp+0x40],edx` with `edx =
  [edi+0x120]+ebx` at 0x454787 through 0x454797, the block copied from `esp+0x38` at 0x4547fa); `+0x0` is the file's
  slot (0x45479f) and `+0x4` the sprite's running number (0x4547ac). th10's were `0x44` bytes with the texture pointer
  itself at `+0x4`.
- Its scripts: the count at `+0x110` (0x4542f5), an array of pointers at `+0x11c` (0x4542ef, filled at 0x4548aa).
  `+0x124` is a "being loaded" word the loader waits on (0x454360), `+0x128` "to be freed" (0x4548d0), and `+0x130` is
  freed too (0x4549e0).

**No surfaces of the captures stand beside the files**: `+0x4350c0` *is* the array of files. **0x445320** (before every
`Reset`, 0x445993, 0x4465fc) walks the 32 files and releases only the textures whose entry has bit 0 of `+0x10` set,
writing nought over the pointer (0x445340 through 0x44539f); **0x4453b0** makes them again with `CreateTexture(w, h, 1,
D3DUSAGE_RENDERTARGET, [0x4c337c], D3DPOOL_DEFAULT, &entry)` (0x445403 through 0x44541f). These render targets are an
`.anm`'s `@` textures. The screen's captures copy the back buffer into such a texture by slot and index — four records
of `0x28` from `sm+0x0` (`{slot, index, rect…}`, 0x445460 → 0x454ec0, `GetBackBuffer` and
`D3DXLoadSurfaceFromSurface`) — so they hold no COM pointer. **So th10's `SURFACES_AT` … `SURFACES_END` has no
counterpart in the manager**; what names a COM object there is every file's array of textures (`+0x120`, `count *
0x14`) and, through the sprites' `+0x8`, those arrays' addresses.

**The COM surfaces outside the manager** are the supervisor's `+0x1ac`, `+0x1b0` and `+0x1b4` = **`0x4c342c`,
`0x4c3430` and `0x4c3434`** — two `GetSurfaceLevel`s of textures 2 and 3 of the supervisor's own `.anm` `[sup+0x588]`,
and the back buffer (`GetBackBuffer`) — made by 0x42b530 (0x42b55c, 0x42b5aa, 0x42b5f5) and released by 0x42b600 around
`Reset`.

### The supervisor's critical sections

**Twelve `CRITICAL_SECTION`s of `0x18` from `0x4c3a90` (the supervisor's `+0x810`) to `0x4c3bb0`, then a depth byte
apiece, `0x4c3bb0` … `0x4c3bbb`**: WinMain initialises 12 from `0x4c3a90` (`mov edi,0xc` at 0x44556f, 0x445575) and the
teardown deletes them (0x445d2f); 0x42a7b0 and 0x42a7e0 do the same on `[eax+0x810]`. Each use is
`EnterCriticalSection(cs)` and then `inc byte [0x4c3bb0 + i]`, under bit `0x8000` of `0x4c3810`. **Their extent is
`0x4c3a90..0x4c3bbc`** (th10 seven, `0x492274..0x492324`).

| i | section | depth | user |
| --- | --- | --- | --- |
| 0 | `0x4c3a90` | `0x4c3bb0` | the chain (0x456cb0, 0x456e10) and every list's add and take-off (0x401265 …) |
| 2 | `0x4c3ac0` | `0x4c3bb2` | the file loader 0x458400 (0x45841f), 0x449d39, 0x4585f7 |
| 3 | `0x4c3ad8` | `0x4c3bb3` | the log 0x458af0 (0x458b10) |
| 5 | `0x4c3b08` | `0x4c3bb5` | the time 0x446920 (0x44692f) and the scene switch (`[sup+0x888]`, 0x42b108, 0x42b1c9) |
| 6 | `0x4c3b20` | `0x4c3bb6` | the build thread's start 0x42a500 (0x42a50c, 0x42a572) |
| 9 | `0x4c3b68` | `0x4c3bb9` | 0x40f262, 0x41a80e … 0x41ac79, 0x41bc5d, 0x432b18 |
| 10 | `0x4c3b80` | `0x4c3bba` | **the generator** 0x458bc0 (0x458bcc) and its inlined copies (0x4376cc, 0x4444b8) |
| 11 | `0x4c3b98` | `0x4c3bbb` | the sound's commands 0x44a340 (0x44a352) and their queue 0x44a9c0 (0x44a9cf) |

Sections 1, 4, 7 and 8 have no absolute reference (`grep` for `0x4c3aa8`, `0x4c3af0`, `0x4c3b38`, `0x4c3b50`).

### The sound's objects on the heap

- **`+0x610`, the DirectSound wrapper**: `new(4)` (`push 0x4` at 0x4491a6, stored at 0x4491c6), holding the
  `IDirectSound8` `DirectSoundCreate8` writes into it (0x4491e0 through 0x4491e5). Th10's shape.
- **`+0x5208`, the music's stream**: built by 0x459eb0 (called at 0x449bfe, `ecx = &[mgr+0x5208]`), **`new(0x7c)`**
  (`push 0x7c` at 0x45a0fb), a `CStreamingSound` (vtable `0x497cbc`, 0x45ad81), the pointer stored at 0x45a128 (th10
  `0x78`). Its `+0x4` is the holder of its buffer, `new(4)`, holding the `IDirectSoundBuffer` (0x45a46d through
  0x45a47f); **`+0xc` its source, `new(0x9c)`** (`push 0x9c` at 0x459f28; `mov [esi+0xc],ecx` at 0x45a495 with the
  source from `ebp`, 0x459f4f, 0x45a11b; th10 `0x94`), whose `+0x90` is the record of `thbgm.fmt`; `+0x8` the buffer's
  size (0x45a48b), `+0x10` 1 (0x45a48e), `+0x5c` the wrapper (0x45a13e), and `+0x60` … `+0x70` (0x45ad75).
- **What the music's thread 0x44a900 writes while it streams**: the stream's `+0x78`, 1 and then 0 around each refill;
  0x45aee0 writes the stream's `+0x60`, `+0x64` and `+0x68` and locks and unlocks the buffer (`[vtbl+0x2c]`,
  `[vtbl+0x4c]`); and the source's `+0x8` (the bytes left), `+0x84` (the memory cursor), and its file handle `+0x8c`'s
  position by `ReadFile` and `SetFilePointer` (0x45b480, 0x45b3a0). The record (`[source+0x90]`, inside the
  `thbgm.fmt` buffer at `[mgr+0x1f84]`) is only read. So every heap object it writes is the stream, the source, or the
  `IDirectSoundBuffer` behind the holder.

### The item manager's ring

The `0x896` items of `0x478` from `+0x14` (0x4231ea through 0x4231f8) end at **`+0x265e64`**; then `+0x265e64` and
`+0x265e6c` are two counts the calc 0x423550 zeroes every frame (0x42356c, 0x423572), and **`+0x265e68` the small items'
ring index** — a slot's number and not a pointer: the slot is `mgr + 0x29e64 + index * 0x478` (0x424418 through 0x42442c;
`0x29e64 = 0x14 + 0x96 * 0x478`), and it is stepped modulo `0x800` (`and ecx,0x800007ff` at 0x42454e, stored at 0x42455f,
its only store). **A stage built fresh starts it at 0**: the manager is `new(0x265e70)` and `memset` whole (0x423462,
0x4234a6). **A stage begun with the managers kept does not**: the first frame's clear, `memset(mgr+0x14, 0, 0x265e50)`
(0x42044b through 0x42045b; 0x42058f through 0x420599 on frame 30), stops at `+0x265e64`. th10's manager was `0x21ceb8`
bytes.

### The player's shots

**`0x100` of `0x6c` from the player's `+0x96c`** (`lea ebp,[esi+0x96c]` at 0x433fa0, `add ebp,0x6c`, `cmp eax,0x100` at
0x433fc9), **free while `+0x48` is nought** (the fire's search, `cmp [ebp+0x48],edx` at 0x433fc2; the move's
`[this+0x28]` with `this = shot+0x20`, 0x434460), and written by the fire 0x433f90 (`eax` the `.sht`'s shot record,
`stdcall` on the place and the player, `ret 0x8` at 0x433fdc):

| | |
| --- | --- |
| `+0x0` … `+0x10` | its timer: the previous count, the count, the float, **`+0xc` the pointer `0x4a7948`** (the game speed, a static), the flags (0x433ff6 through 0x43400d) |
| `+0x14` … `+0x1c` | where it is (0x434026 through 0x434032) |
| `+0x28`, `+0x2c`, `+0x30`, `+0x34` | its speed, its angle and their steps (0x4341b2, 0x434094, 0x43411e, 0x4341be) |
| `+0x44` | bit 0 its kind of move (0x434440) |
| `+0x48` | **its state**: 1 flying, 2 going (0x433fe4, 0x4344a4) |
| `+0x4c`, `+0x50` | **two sprites, by their script's id** — looked up in the manager's lists `[sm+0x7b562c]` and `[sm+0x7b5634]` (`cmp [esi],ecx` at 0x434232), the shot ended where none matches; not pointers (0x43420f, 0x43427d) |
| `+0x58`, `+0x5c` | flags of the frame (0x434440) |
| `+0x60` | its damage, the record's `+0x2` (0x434018) |
| `+0x68` | **a pointer to its `.sht` shot record**, on the heap (0x433fe7) |

So **every byte but `+0x68` is the shot's own** (`+0xc` points at a static), and the sprites are named by id. The homing
target is the player's `+0x7c94`, one for all (0x431b84). th10's were `0x80` of `0x5c`, `0x44` of them its own.

- **The shot's step, `0x434380`, entered with the player in `eax`** (`mov edi,eax` at 0x434382), plain `ret`: only in
  player state 1 (`cmp [edi+0x928],1`, 0x434389); it starts the timer `+0x930` on a frame `[0x4c93c0] & 1` (0x4343a7
  through 0x4343b7), fires at each new count (0x4342f0, 0x4343d7), and past 13 steps it back 14 while the shot is held
  or puts it at -1 (0x4343e1 through 0x43440b). The player's update 0x431070 calls it only where
  `[[0x4a8d84]+0x4438]` is nought (no conversation), `[0x4a8d7c]` and its `+0x70` are there (an enemy up), and bit
  `0x10` of `[[0x4a8d84]+0x4420]` is clear (0x431af9 through 0x431b15).
- **The shots moved, `0x434440`, `stdcall` on the player** (`push esi` at 0x431b1a, `ret 0x4` at 0x4347bd), called
  straight after (0x431b1b), and on the other path too with the shot's timer put at -1 (0x431b90).

### The stage's background

- **The `.std` object `[0x4a8d60]`**, `new(0x3158)` (0x4029c0), stored by its set-up at 0x4024d9; the previous stage's is
  kept at **`[0x4a8d5c]`** by the teardown for scene 12 (0x4200bc through 0x4200c7) and freed once its `+0x2ff0 & 8` is
  up (0x42069e, 0x4206a8).
- **Its calc job `0x403900`** is `mov eax,ecx; jmp 0x402aa0` (0x403900), at priority `0xc` (`mov ebx,0xc` at 0x402612);
  **0x402aa0 takes the object in `eax`** (`mov ebx,eax` at 0x402aaa), plain `ret`. Its draw jobs are 0x403910 → 0x403130
  at 2 and 0x403920 → 0x4035a0 at 5 (0x402667, 0x4026bc), both `stdcall` on it.
- **The camera it copies into is `0x4c36b4`** (the supervisor's `+0x434`): `0x46` dwords from `std+0x3040` every calc
  that gets that far (0x402ba9 through 0x402bb9). The frame's own camera is `0x4c359c` (0x4464a4); a third, `0x4c3484`,
  is what an enemy's instruction adds to a place (0x413123). th10's camera 0 was `0x491d7c`.
- **Its fade gate**, `+0x2ff0` with the timer `+0x2ff4` (its count `+0x2ff8`): the calc returns at once where bit 8 is
  set, or bit 4 with the count `>= 60` (0x402ab4 through 0x402acf), and steps its script (0x4040f0, 0x404470) and its
  eight `0x434`-byte scripts only where bit 4 is clear or the count `< 30` (0x402b1c through 0x402b36, otherwise
  straight to the camera's copy). The draw at 5 counts it down by `-1.0` (`0x497f20`, 0x40383d through 0x40384d) and at
  nought takes 4 and 2 off (8 on where 2, 0x403874); the draw at 2, with 4 and a count `< 30`, puts the count at 1
  (0x403348 through 0x40336d).
- **The game object's frame 0** (0x4203c7): 0x405030 switches the new `.std`'s three jobs on (bit 1, 0x420408).
  **Fresh** (`[0x4a8d5c]` nought): the first frame's set-up and `main` at once (0x42043a on, `main` pushed at 0x420495).
  **Out of a fade**: 0x405130 starts the old one's 30-frame fade-out (the count 30, bit 2) and 0x4051d0 puts **the new
  one at the count 60, bit 4** (0x405215, 0x40522f), `+0x60 |= 0x800` (0x420424), and nothing else. **Frame 30 with
  `0x800`** (0x420560 on): `0x800` off, the same set-up and `main` (0x4205d2), and **the game object's timer put back at
  0** (`lea eax,[edi+0x10]; call 0x406100` with 0, 0x420686) — its frames count again from nought.
- **So** the new `.std`'s calc skips frame 0, copies its unmoved camera on frames 1 to 30 (the count 59 … 30), and first
  steps its script on frame 31 (the count 29), one frame after `main` is spawned. A fresh build steps it on the frame
  `main` is spawned (the game object's priority `0xa` before the `.std`'s `0xc`). **th11's counterpart of th10's
  `A_FADE_RUNS_THE_BACKGROUND_AHEAD_BY` reads as −1**: the background a fade begins is one frame *behind*, not 29 ahead.
  Read, not measured.

### The options made for the power

**`0x432cc0`, entered with the player in `ebx`** (`lea edi,[ebx+0x7624]` at 0x432d0e; its callers load `ebx` first:
0x41fa99, 0x435601, 0x436ee8 and 0x436f0d), plain `ret` (0x433658): the count `power / [0x4a574c]` (0x432cee) into
`+0x7c90` (0x43361d), and **1 into every option's `+0xd4`** (`+0x7644` on, in steps of `0xe4`, 0x433623). th10's was
0x426f70, `stdcall`.

**The option's flag `+0xd4`**: while it is set, the option's update puts its place in 128ths (`+0x5c`, `+0x60`) straight
on its target (`+0x54`, `+0x58`) and clears it (`cmp [edi+0x74],ebx` at 0x430bab, 0x430d37 through 0x430d43, `edi =
option+0x60`). **The playback sets it, and does not leave it at nought**: 0x435630 writes 1 into all eight (0x43566d
through 0x435697), and Yukari's gap does at its wrap (0x43101d). Its other writer is the option's own wrap at
±`0x6600` (0x433756, 0x433775).

### The resume path

**`0x435e00` builds the replay**, `stdcall` on one argument — the game object's `+0x74`, 0 to record and 1 to play back
(`push eax` at 0x41fc2a, the `call` at 0x41fc2b; `ret 0x4` at 0x435eaf and 0x435ec4). Its prologue is `6a ff 68 ab a8
48 00` (`push -1; push 0x48a8ab`). It takes `new(0x2dc)` with eight `0x24`-byte entries from `+0xa4` (the stages of a
file read, 0x435e41 through 0x435e56), zeroed, and then **0x4356e0** with `eax` the mode, `ecx` `0x4a8d88` and the
object pushed. **A fresh build is its only caller** (0x41fc27, taken where `[0x4a5758] & 2` is clear — every scene but
12, 0x41fe73), after the run's numbers and the player (0x4301b0 at 0x41fa5b, the options at 0x41fa9f) and before the
`.std` and every manager (0x41fc42 through 0x41fc91). Nothing in the build before it draws from `0x4c2f00`.

**`[0x4a8eb8]` is the replay**, `0x2dc` bytes, its mode at `+0x10`: 0 recording, **1 playing back** (`mov [ebp+0x10],eax`
at 0x4356ea, `cmp eax,0x1` at 0x4359fc); the card's start counts nothing where it is 1 (0x40c6e4). Its `+0x14` is the
file's header, `new(0x24)`, `t11r` and version 4; `+0x18` the run's header, `new(0x70)` — the character at `+0x5c`, the
partner `+0x60`, the difficulty `+0x64`, the stage it ended on `+0x68` (8 for a cleared run, written at the save), the
continues `+0x6c` (0x4358d8), the practice in bit 0 of `+0xa`, the date at `+0xc`, and a copy of the game object's
`+0x24` … `+0x60` at `+0x18` (0x435804 on); and **a stage's record, `new(0x90)`, at `+0x1c + stage * 4`**. Its jobs:
the calc 0x436d20 at `+0x8`, the calc 0x436d40 at `+0x1d0` and the draw 0x436d80 at `+0xc`.

**A stage's record**, `0x90` bytes (th10 `0x1c4`, with past places, option numbers and the focus that th11's lacks):

| | what | from | written | put back |
| --- | --- | --- | --- | --- |
| `+0x0` | the stage, a word | `0x4a5728` | the build | — |
| `+0x2` | **the generator's seed**, a word, its draws counted from nought | `0x4c2f00`, `0x4c2f04` | the build | the build |
| `+0xc` | the score | `0x4a56e4` | the build; the first frame where not fresh | the build |
| `+0x10` | the power, a word | `0x4a56e8` | 〃 | the build, held to `[0x4a5748]` |
| `+0x14` | `0x4a56f0 / 100`, rounded down to tens | `0x4a56f0` | 〃 | the build, times 100 |
| `+0x18`, `+0x1a` | the lives, the life pieces, words | `0x4a5718`, `0x4a571c` | 〃 | the build (the pieces not on the kept path) |
| `+0x1c` | th10's `0x474c98`, `-0x200` after a continue | `0x4a5744` | 〃 | the build |
| `+0x20`, `+0x24` | **where the player is**, in 128ths | the player's `+0x888`, `+0x88c` | the first frame (nought at a fresh build with `+0x564`) | the first frame, 0x435630 |
| `+0x28` | the continues used | `0x4a573c` | the build; the first frame where not fresh | the build |
| `+0x2c` | the extends | `0x4a5750` | the first frame | the build |
| `+0x30` | focused | the player's `+0x8d20` | the first frame | the first frame |
| `+0x34` | the graze | `0x4a5754` | the build; the first frame where not fresh | the build (not on the kept path) |
| `+0x38` | Patchouli's element | the player's `+0x8bac` | the first frame | the first frame, the options made again where it is not 0 |
| `+0x3c` … `+0x88` | twenty dwords stepping by `-0x21522153` from nought, filler (0x4358cb) | — | the build | — |
| `+0x8c` | bit 0: the supervisor's `+0x564`, the stage built with fresh numbers | `0x4c37e4` | the build | — |

**0x4356e0**, `+0x10` = the mode (0x4356ea) and `[0x4a8eb8]` = the object:

- **recording** (mode 0) fills all of the record at a run's first stage: **the seed, `[0x4c2f00]` (a word), into `+0x2`
  (0x435820, 0x435826) and the draw count `0x4c2f04` put at nought (0x43582a)**; bit 0 of `+0x8c` = `[0x4c37e4]`
  (0x435836); where `[0x4c37e4]` is set, `+0x20` and `+0x24` at nought (0x435845); `+0xc` the score (0x435859), `+0x10`
  the power (0x435862), `+0x14` (0x435866, 0x435889), `+0x18` the lives (0x435893), `+0x1a` the life pieces (0x43589e),
  `+0x1c` `0x4a5744` (0x4358a7), `+0x28` the continues, `+0x34` the graze (0x4358b3), and the filler;
- **playing back** (mode 1, 0x4359fc) reads the file (0x436b60); the game object's `+0x24` … `+0x60` from the header
  (0x435a34); the stage's entry `+0xa4 + stage * 0x24`, its record at that entry's `+0x10` (0x435a5a, 0x435a61); the
  character, the partner and the difficulty from the header (0x435a6f, 0x435a7b, 0x435a87); **the seed back from `+0x2`
  (0x435a8d, 0x435a91) and the draw count at nought (0x435a98)**; the score (0x435aa1), the power held to `[0x4a5748]`
  (0x435ab3 through 0x435abb), `0x4a56f0 = +0x14 * 100` (0x435ac7), the lives (0x435ad1), the life pieces (0x435adb),
  `0x4a5744` (0x435ae4), the continues (0x435aed), `0x4a5750` from `+0x2c` (0x435b04) and the graze (0x435b0f) —
  **unconditionally**, whatever `[0x4c37e4]`. The record's `+0x20` … `+0x38` wait for the first frame.

**0x436f30** (no argument, plain `ret`; the build's path with the managers kept, 0x41fc98; th10's 0x42a6a0): recording,
a new `0x90` record, the seed into `+0x2` (0x436f79) and the draw count at nought (0x436f8b), the stage `+0x0` and bit 0
of `+0x8c`; playing back, the seed back (0x436fea), the count at nought (0x436ff0), the score, the power held,
`0x4a56f0`, the lives, `0x4a5744`, the continues and `0x4a5750` (0x436ffd through 0x437051) — neither the life pieces
nor the graze.

**The stage's first frame** — the game object's frame 0 fresh (0x420480) or frame 30 out of a fade (0x4205bd) — runs, in
order: 0x408550 (the bullets); **0x42fec0 (the player: the state 1, the shot's timer at -1, the state's and the update's
timers at nought, the flags `& ~7`, the gap's state at nought, its sprite `+0x756c` freed, and the options for the power
through 0x432cc0)**; the items' `memset`; every enemy destroyed (0x40d0c0); 0x41f810; `0x4a5734` and `0x4a5738` at
nought; and then **0x436da0** (no argument, plain `ret`):

- recording: `+0x9c` (0x437420, 0x437300); where `[0x4c37e4]` is nought, `+0xc` … `+0x1c`, `+0x28` and `+0x34` (0x436e01
  through 0x436e6f); and always **`+0x20` and `+0x24`, the player's `+0x888` and `+0x88c`** (0x436e7d, 0x436e86), `+0x2c`
  `0x4a5750` (0x436e8f), `+0x38` the player's `+0x8bac` (0x436e98) and `+0x30` its `+0x8d20` (0x436eb3);
- **playing back, the player's part**: **0x435630** (`ecx` the record's `+0x20`, plain `ret`) writes `+0x888`, `+0x88c`
  and the floats `+0x87c`, `+0x880` (times `0.0078125`), and 1 into the eight options' `+0xd4` (0x435637 through
  0x435697); then `+0x8d20` (0x436eee) and `+0x8bac` (0x436ef7); and where `+0x38` is not nought, `+0x7c90` at nought and
  0x432cc0 twice (0x436f02 through 0x436f13).

**Against th10**: the replay's build is `stdcall` on the mode alone and not on two; the playback puts the numbers back at
the build unconditionally; and the player's place is in 128ths (th10 hundredths).

## What is not established

- **A count of the run's deaths that survives a stage.** The enemy manager's `+0x10` is one higher at every death
  (0x431247), but the manager is built again with every stage, so `State::deaths` is orb's own count of the hits.
- **What `[0x4a5748]` and `[0x4a574c]` are in numbers**: the `.sht`'s `+0x20` and `+0x24` are in `th11.dat`. That a
  death takes the whole top off the power is read off 0x4312da, and not watched.
- **Player state 3**: its handler exists (0x43153a) and every test refuses it, but no store of 3 into `+0x928` was found.
- **What `[0x4a56f0]` and `[0x4a56f4]` are** beyond what the death, the hit and the items do with them; and
  `0x4a5740`, `0x4a576c`, the run's flag `0x4`, `CR+0x58c` and the float at a ranking entry's `+0x18`, read where they
  are written and not further.
- **What the game object's `+0x60` bits `0x2|0x400` mean**, under which the bullets do not move.
- **Who calls 0x430de0 and 0x430e20** (Yukari's gap's untouchable bit): no `call` and no pointer to either was found.
- **th10's past places** (`+0x436c`): not looked for.
- **Whether a card is declared on the frame its attack begins**: orb reads the card's own timer while one is up and
  the enemy's `+0x1194` only between cards, so a card declared later than the shortest chapter would be a chapter
  of its own; on stage 1's launch of 2026-10-07 every card began a chapter of its own and no more.
- **The enemy's flags past `0x22`, `0x400000`, `0x80000`, `0x200`, `0x1000` and `0x4000`**, and how many boss slots
  there are.
- **How Nitori's armed bomb changes the body's test** (0x411b24, 0x411c3a), and the card numbers `0x9e` to `0xae` the
  enemy's update treats apart (0x411aff on): not read through.
- **What sets bit `0x2000` of `[0x4c3810]`**, under which every gated writer writes scene 2: no store that puts it on
  was found; the loading object takes it off (0x428f06).
- **Scenes 5, 6, 8 and 9**: the switch builds nothing for them, and no writer of any was found.
- **What the build does after a stage-8 start-over.** That the `.std` name is read from `0x4a3a28 + 0x4` was read; the
  file read it makes on a code address, whether 0x403fc0 fails and logs `0x4938e8`, whether the game then leaves with
  a message box as th10 does, and what `0x4029c0` does with the code address, were not watched.
- **Which partner each of 0 to 2 is, and which character 0 and 1 are**, by name: the exe holds only `pl00a` … `pl01c`.
  The names given beside the shot types in *The player* — Yukari, Suika, Aya, Patchouli and Nitori — are not read
  off the exe either.
- **The labels of the items** of the pause menu, of the end of a run's two menus and of the options' sub-screen 4, read
  off what each item does and not off the sprites drawn for them (`0x4b`, `0x59` to `0x5b`, `0x62`, `0x67` and `0x68`
  of the menu's `.anm`).
- **What bit `0x10` of `[0x4c3810]`, which opens the pause, is called**, beyond its writers (a successful `Reset` puts
  it on, a frame answering `0` takes it off).
- **Whether the cursor skips the items on the pause object's `+0xc8` list**: 0x459160 was not read.
- **Whether the continue is refused on any difficulty**: no test was found in the code read; one in the menu's drawing
  or in 0x459160 was not looked for.
- **Where `staff.msg` is loaded**: it is the thirteenth entry of the ending's table (`0x4a3dd8`), and no read of that
  index was found.
- **The packer's and the unpacker's parameters read off 0x4423a0 and 0x4426c0**: established only by the install's file
  unpacking to the right length with every sum sound.
- **The names of cards 48, 49, 52, 53, 56, 57, 60, 61, 64, 65, 68, 69, 72, 73, 76, 77, 80, 81, 96, 100, 104, 108, 112 and
  116**: no record of the install's file holds one; they are in the `.ecl` files and not the exe.
- **Whether an Easy clear with no continue after a Normal one wipes bit `0x10`** of its ending's byte: the `mov byte
  …,1` at 0x40ede8 reads so; not watched.
- **Whether priority 1 makes 0x429030 the first job of every walk**: it is the lowest calc priority among the 35
  callers of `0x456b70`, and nothing past that was looked for.
- **That `0x48caec`, `0x48cc5c`, `0x48b55c` and `0x48b764` are `IID_IDirectInput8A`, `GUID_SysKeyboard`,
  `c_dfDIKeyboard` and `c_dfDIJoystick2`**: read off the calls' shapes and the `0x110` state size; the bytes were not
  compared.
- **What `[0x4c3dc0]`'s bits that F10 toggles do** beyond the window's size, its change of mode and its frame (the
  window procedure's `WM_SIZE` branch at 0x445e55 reads bit `0x1`), and what the countdown `[0x4c3dc4]` gating it is.
- **What the live word's Ctrl tests at 0x40f152** (in 0x40f120) **and at 0x4362ff and 0x436d55** (the replay's own job,
  state 1) do with Ctrl: they answer 6 or other values off frame counts modulo 12 and 6, read as a skip but not
  followed.
- **Who reads `+0x12c` and `+0x130` of the input**, the held-eight-frames words: no absolute reference to `0x4c93d4` or
  `0x4c93d8` was found, and none through a register was looked for.
- **That 0x41d380 is the conversation's calc**: read off its Ctrl and decide tests, not off its constructor.
- **S (`0x20000`) and D (`0x100000`)**: no `test` of either against any input word was found, but a test through a
  register was not ruled out.
- **What `0x42b600` and `0x42b530` do** around `Reset` beyond the supervisor's three surfaces, whether those surfaces
  change outside `Reset`, what the supervisor's `.anm` at `+0x588` is, and what `0x40c310` on `[0x4a8d6c]` measures after
  `Present`.
- **What the configuration byte `0x4c346b` (`+0x23`) is called in the options**, whose `1`, `2` and `3` choose the
  pacing; and what bit `0x10` of `[0x4c3480]` (`+0x38`) is called — read off the code, it plays the tracks from memory
  in place of streaming them from `thbgm.dat`. The defaults leave it clear.
- **Whether `[chain+0x48]` has a writer besides the two teardowns** (0x4451e8, 0x456fae): only stores of that exact form
  were looked for.
- **Who sets `[0x4c3d8c]`**: its one store found is the `0` at 0x447257.
- **Whether `0x446020`, `0x4476e0`, `0x446ab0` and `0x449b00` are reached** through a table or not at all: no direct call
  was found.
- **The order of `thbgm.fmt`'s records and what `+0x18`, `+0x1c`, `+0x24` and `+0x2c` hold**, which are in that file and
  not the exe.
- **Whether the stage's song is on the stream by the frame orb takes the stage's first chapter on**: the stage begun only
  queues command 2, whose source is set at its third run, one run per calc walk that reaches the supervisor.
- **What brings the window to the front in a launch**: the game itself sends `SC_MINIMIZE` and then `SC_RESTORE` to its
  window right after making it (0x446cfa, 0x446d16) and calls `SetForegroundWindow` before the loop and on a left click;
  which of these, or Windows' own rule for a new process's first window, has the window at the front was not watched.
- **The −1 of the background out of a fade is read, not measured**; nor is whether any enemy rides `0x4c36b4` the way
  th10's flag `0x40000` rode camera 0 — the only camera an enemy reads that was found is `0x4c3484` (0x413123), which no
  `.std` writes.
- **Sections 1, 4, 7 and 8 of the twelve critical sections**: no user found by absolute address, and one through a
  register was not looked for.
- **Whether any `.anm` is loaded or freed inside a stage** (th10's reading says none): the callers of 0x454190 and
  0x4549a0 were not walked.
- **What the four capture records at the sprite manager's `+0x0` hold** past `+0x0` (the slot) and `+0x4` (the index),
  and who fills them.
- **The player's shots' fields `+0x38` … `+0x40`, `+0x54` and `+0x64`**, and what the shot's timer's round of 14 frames
  means in shots.
- **What 0x437420 and 0x437300 put in the replay's `+0x9c`**, and a file stage entry's `+0x0` … `+0x14` (0x435a3e through
  0x435a66).
