# th10, as read off its exe

What is written here was read off `th10.exe` of **md5 `7dc488d82c81dd4aee4ba098b8804d83`**, 487,936 bytes,
`東方風神録　～ Mountain of Faith. ver 1.00a` by its own window title (`0x46f66c`), which is the one build
`game::th10` has addresses for. Every address below is that build's and means nothing about another.

To read it again:

```sh
i686-w64-mingw32-objdump -d -M intel th10.exe > th10.asm        # the whole of .text, once
i686-w64-mingw32-objdump -d -M intel --start-address=0x439390 --stop-address=0x439540 th10.exe
i686-w64-mingw32-objdump -s --start-address=0x470c90 --stop-address=0x470c98 th10.exe   # a constant
analyzeHeadless <project dir> <name> -import <exe> -scriptPath <dir> -postScript DumpAll.java <out.c>   # every function decompiled, by Ghidra
```

`objdump` loses the thread after a `jmp` table and starts printing data as instructions; where a listing
here disagrees with a dump, dump the one function again from its own first instruction. Most of this game's
functions have no `push ebp` to start from: the compiler passed arguments in whichever register suited it
(`esi`, `edi`, `eax`), which is said beside each function below that does.

The sections are `.text` at `0x401000` (file `0x400`), `.rdata` at `0x466000` (file `0x65000`), `.data` at
`0x473000` (file `0x71c00`) and `.rsrc` at `0x49a000`. `.data` is `0x4200` bytes in the file and more in the
image — `SizeOfImage` is `0x9c000` — so everything past `0x477200` is zero-filled at load and none of the
game's objects' pointers are in the file. The import address table is at `0x466000` and the import directory at
`0x4718b8` (the PE header's data directories 12 and 1); the game calls some imports
through thunks of its own at `0x45xxxx` (`jmp [0x4662b4]` at 0x452176 for `Direct3DCreate9`). The game's
own strings are CP932.

**This file is a reading and not a plan.** What orb *does* with any of it is in the code beside the
constant.

## What is not established

- **A count of the run's deaths of the game's own.** No field of the run's numbers at `0x474c40` counts a death:
  the death, 0x4269d0, takes a life (0x426a15), a third of the faith above 5000 off `0x474c4c` (0x426a22, a faith
  under 5000 put up to 5000 at 0x426a2a) and `0x400` off the word at `0x474c98`, held within ±`0x400` (0x426cb6
  through 0x426cde), and the dying state takes `0x40` off the power on its third frame, not below nought (0x425ab6)
  — so `State::deaths` is orb's own count of the hits; whether the score file or the player keeps a count of misses
  is not looked for.
- **Player state 3.** Its handler exists (0x425d25) and both of the player's tests refuse it, but no store of 3
  into the state was found among every `mov [reg+0x458],…` in the exe.
- **What `[[0x47770c]+0x9eb8]` refusing a hit means beyond a conversation.** That field is the conversation
  running (0x415df9, 0x415490); both tests take nothing while it is set, and so does the bomb.
- **Where a bullet's spawn ends.** A bullet in state 2 turns 1 on the frame the variable `I0` of its sprite's
  script — `bullet+0x314` — is set (0x40635e), which the `.anm` script writes; the frames that takes are data
  in `bullet.anm` and not in the exe.
- **The speeds.** `+0x3d4` and its three neighbours are the `.sht` file's own floats times a hundred; the
  numbers themselves are in the files inside `th10.dat`.
- **Whether each stage's `main` waits for its fights on instructions 339 and 340** or polls in a loop of its
  own, and whether `main` lives the whole stage — a conversation's start (0x40e6a0, called by the conversation's
  constructor 0x415b00 at 0x415d9b) marks every enemy without a flag of `0x10|0x40|0x4000|0x8000`, and every one
  with `0x80` whichever it has (0x40e6bb through 0x40e6c6), with `0x20000` (0x40e6f4), and the enemy manager's next
  walk, 0x40d750, destroys each enemy so marked (0x40d762, 0x40d798); two of the script's instructions call 0x40e6a0
  as well (0x41007f, 0x410e00); and which flags `main` carries is not read. Both are in the `.ecl` files inside `th10.dat` and not the exe.
- **The labels of the pause menu's and the end of a run's items**, which are read off what each item does and
  not off the sprites drawn for them.
- **Scenes 5, 6, 8 and 9**: the switch builds nothing for them and no writer of any was found. And **what sets
  bit `0x1000` of `[0x491ff4]`**, under which every writer of scene 2 writes it (see *The supervisor and its
  scenes*): the loading object takes it off (0x41fdb2, 0x41fed2), and no store that puts it on was found.
- **The order of `thbgm.fmt`'s records and what each holds past `+0x14`**, which are in that file and not the
  exe; and **whether the stage's song is on the stream by the frame orb takes the stage's first chapter on** —
  the stage's start only queues it (0x4183e1), and where in the frame orb reads against the manager's own call
  carrying it out is not watched.
- **What has the window orb plays in at the front for some 231 milliseconds of a launch**, with
  `D3DCREATE_NOWINDOWCHANGES` on its device and the game's `SetForegroundWindow` swallowed (2026-10-03 21:59,
  before the swallowed call at 0x438cb1). It is none of the game's own calls that bring a window forward: the exe
  imports no `SetActiveWindow` or `AttachThreadInput`; its `SetForegroundWindow` (0x438cb1, 0x4391d2, 0x43a1b1,
  and the thunk at 0x452392) goes through the one import entry orb writes over; its `SystemParametersInfoA` is
  the screen saver's and the display's power settings, `0x10` and `0x11`, `0x53` to `0x56` (0x438fbf, 0x4392e1,
  0x439356); its `ShowWindow` is the teardown's hide (0x438e55); and its `DialogBoxParamA` (0x438b8c) runs only
  where bit `0x100` of the configuration at `0x491d78` is set, which the configuration's defaults set (0x418c62).
  What is left is the device's own making and Windows bringing a new process's first window forward, between
  which nothing is read.

## The input

**`0x44a5f0` is the read**, called once a walk from the supervisor's calc job (0x41ffad), which is priority 1
and the first job of every walk. `ecx` is the player, always nought; it ends in a plain `ret`, and it stores the
word itself: the frame before's into `+0x2`, this frame's into `+0x0`, the buttons gone down into `+0x6` and those
let go into `+0x8` of the player's `0x6a`-byte input at `0x474e30` (0x44a8bd through 0x44a926). **And an
auto-repeat word into `+0x4`** (`0x474e34`), off sixteen `u16` counters from `+0xa`, one a button: a counter
counts the frames its button is held and is put at nought where it is not, and a button held fires in `+0x4` on
its 26th frame and every eighth frame after, the counter taken back by eight each time it fires (0x44a8d2 through
0x44a90a). **The menus move their cursors on `[+0x6] | [+0x4]`**, so a direction held repeats — the pause menu
(0x422d1c), the end of a run (0x4238ad), the title's screens (0x42d52b, 0x42f640, 0x43080e, 0x430d78, …).

**The keyboard is read only while the game is the active application**: the read skips straight to the pad's half
where `[0x4924fc]` is nought (`je` at 0x44a61d), which the WndProc 0x4390e0 sets to `WM_ACTIVATEAPP`'s `wParam`
(0x43911d) and 0x439730 starts at 1 (0x43978b). A `WM_LBUTTONDOWN` on the window is answered with
`SetForegroundWindow` on it (0x4391d2).

**What it stores is what `0x44a190` answers**, the pad's half: `push 0; push ebx; mov ecx,edi; call 0x44a190`
at 0x44a8b3, the keyboard's word in `ecx`, the player and the pad pushed, and `ret 0x8` (0x44a32b). Its other two
callers (0x44ac96, 0x44afea) are in reads nothing calls.

The keyboard comes through `IDirectInputDevice8::GetDeviceState` on `[0x491c38]` where bit `0x200` of
`[0x491ff4]` is set (0x44a628, 0x44a766) and through `GetKeyboardState` otherwise (0x44a636); the pad through
DirectInput's `[0x491c3c]` where bit `0x400` is and `joyGetPosEx` otherwise. A failed read of the keyboard is a
word of nought and an `Acquire` (0x44a76b through 0x44a78e).

**On the default settings the keyboard is DirectInput's.** The input's set-up, 0x43ba90, runs on a thread of its
own, which the supervisor's calc job's add-callback 0x4201b0 starts through `_beginthreadex` at 0x420249, its
handle into `0x474dd8`: it takes bits `0x600` off `[0x491ff4]` (0x43ba90), makes the devices in 0x43b8d0, and
sets `0x200` where `[0x491c38]` is there and `0x400` where `[0x491c3c]` is (0x43baa4 through 0x43badd). 0x43b8d0
makes no device only where bit `0x8` of `[0x491d78]` — `th10.cfg`'s `+0x30` — is set (0x43b8df), and the
configuration's defaults, 0x418c40, set only `0x100` there (0x418c62); otherwise it makes the keyboard with
cooperative level `0x16`, `DISCL_NONEXCLUSIVE | DISCL_FOREGROUND | DISCL_NOWINKEY` (0x43b9bc), and the pad
DirectInput's enumeration finds (0x43ba19, 0x43baf0) with level `10`, `DISCL_NONEXCLUSIVE | DISCL_BACKGROUND`
(0x43ba3e).

| bit | key | what reads it |
| --- | --- | --- |
| `0x0001` | Z | the shot; decide |
| `0x0002` | X | the bomb; cancel |
| `0x0004` | Shift | the focus |
| `0x0008` | Esc | the pause; cancel |
| `0x0010` `0x0020` `0x0040` `0x0080` | the arrows, and 8, 2, 4 and 6 of the numeric keypad | up, down, left, right |
| `0x0050` `0x0090` `0x0060` `0x00a0` | 7, 9, 1 and 3 of the numeric keypad | up and left, up and right, down and left, down and right |
| `0x0100` | Ctrl | a conversation run on (0x416a44, 0x416a49) |
| `0x0200` | Q | the pause menu's own way to the title (0x422e9b) |
| `0x0400` | S | no test of it found |
| `0x0800` | Home, P | the screenshot (0x439250) |
| `0x1000` | Enter | decide |
| `0x2000` | D | no test of it found |
| `0x4000` | R | the pause menu's own way to a retry (0x422e28) |

The `GetKeyboardState` half reads Shift and Ctrl as `VK_SHIFT` and `VK_CONTROL`, either side (0x44a63c through
0x44a74f); the DirectInput half reads `DIK_LSHIFT` and `DIK_LCONTROL` alone (0x44a796 through 0x44a8b1), so the
right Shift and the right Ctrl do nothing there. Ctrl is read off the run's copy, `[0x474e5c]` (0x416a44, `test
ah,0x1` at 0x416a49), behind the conversation's own flag that lets it run on, `test [ebp+0x7c],0x1` at 0x416a3a.

Every menu takes `[0x474e36] & 0x1001` as decide and `& 0xa` as cancel.

**The pad's half, `0x44a190`**, reads joystick 0 one of two ways. Where the input's set-up found a DirectInput
pad (0x43bac5 through 0x43badd, `[0x491c3c]` and bit `0x400`), it is `Poll` (0x44a33b) — and where that fails,
one `Acquire` (0x44a351), and only where that one answers `DIERR_INPUTLOST` up to 400 more while they answer it
(0x44a360 through 0x44a37b), the frame's word then the keyboard's alone with no `GetDeviceState` either way
(0x44a37d, 0x44a3c2) — and `GetDeviceState` of `0x110`, a
`DIJOYSTATE2` (0x44a3bb), its axes ranged ±1000 (0x43bb30) and read against the thresholds at `0x491d5e` and
`0x491d60`, `+0x16` and `+0x18` of the configuration and 600 apiece by default (0x418c4d, 0x44a457 through
0x44a4b4). Otherwise it is `joyGetPosEx(0)` with every field asked for (0x44a1f7), each axis past a quarter of the
travel the `JOYCAPSA` at `0x4918b8` gives it from its middle (0x44a293 through 0x44a315). **Neither looks at the
hat.** The buttons are by the mapping at `+0x58` of the player's input, nine `i16` copied out of `th10.cfg`'s `+0x4`
(0x4208ff through 0x420934) with the exe's `.data` holding `0, 1, 2, 3, -1, -1, -1, -1, 4` there for a file that
does not answer (0x418c7a). The read takes five of them, the first four and the ninth: the shot `0x1`, the bomb
`0x2`, the focus `0x4`, the pause `0x8` and the skip `0x100` (0x44a205 through 0x44a291); a negative one names none.

**The run acts on a copy**, which the replay's calc job makes at priority `0xb` (0x42a3d0 → 0x4297d0), the copy
of the frame before kept at `0x474e5e` either way (0x4297f5, 0x429908). Recording, `[0x474e5c] = [0x474e30] &
0x1f7` (0x4297e9 through 0x429809); and where bit `0x200` of `[0x491d78]` is set, a shot held for eight frames has
the focus `0x4` put in as well, its count at `0x474e5a` (0x429806 through 0x429842). Then 0x40ac20 on `0x474e30`
works out the copy's buttons gone down into `0x474e62` and those let go into `0x474e64` (0x40ac6a through 0x40ac86),
and its repeat into `0x474e60` (0x40ac40 through 0x40ac68) — putting at nought not that word but `[0x474e34]`, the
live input's own repeat (0x40ac29); the replay records the three words `0x474e5c`, `0x474e62` and `0x474e64`
(0x4298b2 through 0x4298ca). Playing a replay back, the replay's own six bytes a frame into the same three
(0x42993e through 0x429974). The player's job is
priority `0x10` and reads `0x474e5c` (0x4250b3), so **the player moves on the frame's own keys**.

## The supervisor and its scenes

**`0x491c28` is the supervisor**: its `+0x0` is the `hInstance` (0x438b51), `+0x4` the `IDirect3D9` (`0x491c2c`,
0x438bca), `+0x8` the device (`0x491c30`), `+0x48` the HWND (`0x491c70`, read at 0x43b8d3), `+0xe4` the present
parameters (`0x491d0c`, 0x439232), `+0x150` the configuration's flags (`0x491d78`, 0x43b8df), `+0x3cc` the input's
and the display's flags (`0x491ff4`, 0x40ac90) and `+0x750` the fog's word (`0x492378`, 0x420cd0). Its calc job
`0x41ff80` reads the keys and then switches the scene at 0x4218d0 wherever the scene it is on, `[0x491fb4]`, is not
the one wanted, `[0x491fb8]`:

| scene | what the switch does |
| --- | --- |
| 0, 1 | the game's startup and its loading |
| 2 | nothing; it is written in place of the scene a writer would have wanted wherever bit `0x1000` of `[0x491ff4]` is set — by 0x40ac90 in place of the one it is handed, by 0x4181cb in place of 3, 0x4186a1, 0x4231f0 and 0x423390 in place of 4, 0x40ba11 in place of 15 and 0x414937 in place of 14 |
| 3 | the game leaving |
| 4 | **the title**, built by 0x42cd50 |
| 7 | **a run**: the game object built by 0x4180e0 |
| 10 | a retry: the run built again from the stage it started at |
| 11 | **the stage after the one won**: the game object built again with the run's numbers carried, and 7 wanted |
| 12 | a replay, built the way a run is |
| 13 | the run built again where it is |
| 14 | **the ending**, with the staff roll inside it (0x40b940) |
| 15 | the title again after the ending |

**The game object `[0x477810]`** is `0x60` bytes: its flags at `+0x58` — `0x4` while its stage is built on the
loader thread (0x41787a, 0x417c30) and `0x10` from the pause menu's opening (0x422af4), the game over (0x4231d0, at
0x423244) and a run ended by its clear (0x423370, at 0x4233e4), which only the pause menu's resume takes off
(0x422c38, from 0x4231ab) and so stays set past the pause menu's ways to the title and to a retry (0x423193,
0x42316c) — and at `+0x5c` a `1` for a replay played. While any of `0x10`, `0x20` and `0x40` is set the game
object's calc job answers `3` (0x4186e2 through 0x4186fd), which stops the walk where it stands.

**The stage is built on a thread**: 0x4180e0 starts 0x417c70 through `_beginthreadex`, which runs 0x417870 —
the run's numbers put in place, every manager of the stage made, the music loaded — while the game's own calc job
steps nothing (0x41864b).

## The front end

**The title `[0x47784c]`** is `0x5acc` bytes, its calc job at priority 6: the screen at `+0x1c` (the switch at
0x42ce37), the screen's own step at `+0x20`, the cursor at `+0x24` and the frames since the screen or its step was
set at `+0x2b4`.

| screen | what |
| --- | --- |
| 1 | the title waiting for a key: `[0x474e36] & 0x160b` |
| 2 | **the main menu**, eight items: a run, the Extra, practice, the replays, the player's data, the music room, the options, and leaving |
| 6 | the difficulty |
| 7 | the character |
| 8 | the shot |
| 9 | the practice's stage |
| 12 | the replays |

Each of those waits out the frames it ignores a key for — ten on screens 1, 2 and 9, six on 6, 7 and 8 — and reads
one on the next. **A run is written into the run's numbers screen by screen**: the difficulty `[0x474c74]` by the
difficulty select (0x4305cb), which leaves the Extra's 4 as it is (0x4305bf); the character `[0x474c68]` by the
character select (0x430997); the shot `[0x474c6c]` by the shot select (0x430ec0, 0x430ef6). And for a run not
practised (0x430f01) the shot select's decide writes the stage `[0x474c7c]` and the stage the run started at
`[0x474c80]`, 1 or 7 for the Extra, the stage's record `[0x477848]` (0x430f16 through 0x430f4f), and the scene
wanted as 7 (0x430f5a).

## The pause and the end of a run

**`[0x477830]` runs the pause menu, a run's end, its name entry and its replay offer**: its state at `+0x4` is
nought where none is up, 1 to 5 the pause menu (0x422c80) and 6 to 13 the end of a run (0x4236f0) — 12 the
ranking's name entry, 10 and 11 the replay's slots and its name — and `0x15` once state 13 has carried out its
item (0x424330), a state the end of a run's table has no entry for and so steps nothing (0x423709). **The pause
menu's item for the title writes the scene wanted as 4** (0x40ac90, called with 4 at 0x42319a), and the
supervisor takes the game down and builds the title.

**The end of a run** goes through the states by the table at 0x42438c, each but two waiting out ten frames of the
timer at `+0x10` first — state 8 reads its keys from its first frame (0x42389b) and state 13 waits out twelve
(0x42431d):

| state | at | what |
| --- | --- | --- |
| 7 | 0x42377e | the run's score into its record; for a run not practised, 0x423570, and then the menu where `+0x1e8` is set and the name entry where it is nought (0x423859) |
| 8 | 0x42389b | **the menu**, three items: playing on, the replay's save and the title; up and down move the cursor `+0x24`, a decide (`[0x474e36] & 0x1001`) takes it |
| 10, 11 | 0x423a7f, 0x423cdf | the replay's slots and its name |
| 12 | 0x423f93 | the ranking's name entry, back to the menu as it ends |
| 13 | 0x42431d | the item taken carried out twelve frames on: items 1 and 2 the title (0x42433f), item 0 the scene wanted as 10 where `+0x1e4` is set and 13 where it is not (0x424360) |

**`+0x1e4` is the run cleared**, and for a cleared Extra 0x423570 moves the run past it: with `[0x474c7c]` at 7 it
writes 8 into both the stage `[0x474c7c]` and the stage the run started at `[0x474c80]` and points `[0x477848]` at
the stage record after the Extra's, `0x474908` (0x42358f through 0x4235a3), and puts back 7 (0x4235de) only where
the stage still reads 7 after 0x421fa0 — which it does not, so the stage stays 8. **And the menu the name entry
hands back to stands on item 0** (0x4241de, and twice more at 0x424247 and 0x4242f4), where the one state 7 puts up
stands on item 2 (0x42388e). So a decide after the name entry of a cleared Extra starts the run over (scene 10,
0x421a80) from stage 8, whose record names no `.std`: the stage object's set-up fails (0x40225a), logs
`ステージデータが読み込めません。データが壊れています` (`0x46cc70`, at 0x402263), and the game leaves and puts its
log up in a MessageBoxA (0x438f77) that stands until it is answered. Caught on 2026-10-02 with cdb on a legacy
Extra: the menu's item carried out with the cursor at 0, `+0x1e4` at 1 and both stage words at 8, then the scene
wanted 10 and the set-up's failure.

## The run's numbers

The code hands them over as `0x474c40`:

| address | what |
| --- | --- |
| `0x474c44` | the score over ten |
| `0x474c48` | the power, nought to `100`, shown as `5.00` at `100`; a bomb costs `20` (0x4259d4) |
| `0x474c4c` | the faith over ten |
| `0x474c68` | the character, 0 Reimu and 1 Marisa |
| `0x474c6c` | the shot, 0 to 2 |
| `0x474c70` | the lives |
| `0x474c74` | the difficulty, 4 the Extra |
| `0x474c7c` | the stage from 1, 7 the Extra |
| `0x474c88` | the frames since the stage began (0x418769) |
| `0x474ca0` | the run's flags: `0x10` a practice, `0x20` the attract demo |

**The generator** is the word at `0x4918b0`, stepped at 0x44b9b0, seeded once from `timeGetTime` as the game
starts (0x4201d0) and not again at a stage: a replay keeps each stage's seed and puts it back.

## The player

**`[0x477834]`**, `0x4478` bytes, its calc job `0x425730` at priority `0x10` — ahead of the lasers' `0x13` and the
bullets' `0x14`, so a frame's fire is tested against the player where the frame's move put it.

| | |
| --- | --- |
| `+0x3c0` | where it is, two floats: the whole hundredths at `+0x3cc` times `0.01`, clamped to `-184` to `184` across and `32` to `432` down (0x425446 through 0x4254c2) |
| `+0x3d4` … `+0x3e0` | the four speeds in hundredths: straight, focused straight, diagonal, focused diagonal (0x4248dc through 0x424926) |
| `+0x404`, `+0x410` | the corners of its box, built from `+0x41c` (0x425f00 through 0x425f97) |
| `+0x41c` | the half of its box, half of `[0x476fb8 + character * 4]`: `1.0` for Reimu and `1.75` for Marisa |
| `+0x458` | its state: 0 coming back in, 1 playing, 2 dying, 4 hit and inside the eight frames a bomb takes it back in (0x425a96) |
| `+0x460`, `+0x474`, `+0x488` | three timers of the game's, each its previous count, its count, its float, the game speed's pointer and its flags, a word apiece: the shot's, its count `+0x464` held at -1 while it is off (0x426301); the state's, its count `+0x478` (0x426a44 through 0x426a8f); and the one every update steps, its count `+0x48c` (0x4261f2), which holds the focus off while below 4 (0x425177). 0x424d90 starts the last two (0x424df0 through 0x424e56) |
| `+0x4310` | the frames nothing can hit it for, counting down |

**Both of the player's tests** — the bullets' and the bodies' `0x4266b0`, and the lasers' `0x4267f0` — answer
nought for a box that touches the player while a conversation is on or the state is 2, 3 or 4 (0x42671e through
0x426750, 0x426924 through 0x42694e). Otherwise they differ over `+0x4310`. **The lasers' refuses outright** while
it is above nought (0x426950 through 0x426958), and takes the hit through `0x426cf0` and answers 1 where it is not.
**The bullets' answers 1 either way** and calls `0x426cf0` only where `+0x4310` is below 1 (0x426756 through
0x426766) — so a bullet that touches a player nothing can hit is cancelled all the same, its state put at 3
(0x4064ad). A box that misses the player but comes within 24 of it answers 2, the graze (0x426771 through
0x4267dd, the `24.0` at `0x470be8`; 0x42696b through 0x4269b4 for the lasers).

**`0x426cf0` takes the hit**: the state at 4 (0x426cff) and its timer at `+0x478` started at nought (0x426e44),
`+0x4310` at 6 (0x426ea5), and where the card `[0x4776f4]`'s word at `+0x3738` is above `0x3b` (0x426ed5),
bit 1 taken off its `+0x378c` and off seven more of its words (0x426ede through 0x426f53).

## The bullets

**`[0x4776f0]`**, `0x3e0b54` bytes: `0x7d0` bullets of `0x7f0` bytes from `+0x60` and how many survived the last
walk at `+0x5c`. A bullet's word at `+0x446` is its state — 1 active, 2 coming out, 3 cancelled — and it is
tested in state 1 alone and only where bit 1 of `+0x0` is set; it moves `+0x3c0` a frame times the game's speed
(`[0x476f78]`), half that in states 2 and 3; its box is the width and height at `+0x3f0` about `+0x3b4`, and the
test is two boxes.

## The lasers

**`[0x47781c]`** keeps up to `0x100` of them on a list — the first at `[+0x18]`, each one's next at `+0x8`, the
count at `+0x438`. Two kinds, told apart by the vtable: `0x46da60` travels and is tested once longer than sixteen
and wider than three; `0x46da10` stays where it was fired from and goes through a warning line two wide (`+0xc` 3,
tested by nothing), a widening (4), its whole width (2) and a shrinking (5), tested in 4 and 2. Both hand their
test the stretch from a tenth of the length to nine tenths, and a width the test halves (0x4268ed): half the drawn
width `w` where it is below 32, and from 32 on `w - (w + 16) / 2` for one that travels (0x41d670 through 0x41d697)
and `w - (w + 16) / 3` for one that stays (0x41e8e7 through 0x41e90e) — so a half-width of `w / 4` below 32, and
`(w - 16) / 4` and `(w - 8) / 3` from it.

## The enemies, the boss and the card

**`[0x477704]`**: the boss the fight is with at `+0x10`, a list of enemies from `+0x58` — each element the enemy
and the next — and the count at `+0x60`. An enemy is `0x2518` bytes: its main script's time at `+0x8`, where it is
at `+0x1068`, its body's size at `+0x10f4`, the frames its attack has run at `+0x115c`, its life at `+0x23fc`, and
its flags at `+0x2480`, of which `0x12` skip the body's test (0x40e2bd). **The card `[0x4776f4]`** runs while
bit 0 of `+0x378c` is set, and `+0x3788` is its number.

**Which card a number is**, by the game's own names for them: a `CR` record of `scoreth10.dat`, unpacked as
`score_file::decode` unpacks it, holds a card's name every `0x90` bytes from `+0x59c` — Shift-JIS, nought-ended,
card 0 first — 110 of them, one per card and difficulty, a card the Easy and the Normal share being two. Read off the
install's file (2026-10-04), the numbers the ways are named by are: 3 `秋符「オータムスカイ」` of stage 1; 62
`奇跡「白昼の客星」` and 70 `準備「神風を喚ぶ星の儀式」` of stage 5; 78 `神祭「エクスパンデッド・オンバシラ」`, 82
`筒粥「神の粥」`, 86 `贄符「御射山御狩神事」`, 90 `天流「お天水の奇跡」` and 94 `「マウンテン・オブ・フェイス」` of stage 6;
and of the Extra 98 `神符「杉で結ぶ古き縁」`, 99 `神符「神が歩かれた御神渡り」`, 100 `開宴「二拝二拍一拝」`, 101
`土着神「手長足長さま」`, 102 `神具「洩矢の鉄の輪」`, 103 `源符「厭い川の翡翠」`, 104 `蛙狩「蛙は口ゆえ蛇に呑まるる」`, 106
`土着神「ケロちゃん風雨に負けず」` and 107 `土着神「宝永四年の赤蛙」`.

## The stage's script

**`0x44e1a0` runs one script** — `eax` the script, one float pushed, `ret 0x4` — and `0x44fd10` runs all of an
enemy's, its main one first (0x44fd29) and the ones it started after, from the list whose head is at
`enemy+0x1030` (0x450130 puts each new one in straight after it). The enemy update calls it with
`[[enemy+0x1164]]`, which is `[0x476f78]`, the game speed (0x40e0e9 through 0x40e0fe). A script is `0x1024`
bytes, the main one inside the enemy at `+0x8`: its time, a float, at `+0x0`; where it is at `+0x4`; a
`0x1000`-byte stack at `+0x8` with its top at `+0x1008` and its frame at `+0x100c`.

**Each run** executes every instruction whose time — the `i32` at the instruction's `+0x0`, its number the
`i16` at `+0x4` and its size the `u16` at `+0x6` — is no later than the script's time (0x44e1c6 through
0x44e1d0), and then adds the float it was handed to the time (0x44fb96 through 0x44fb9f). **An instruction
answering `-1` holds the script** where it is, with neither the time nor the place moved (0x44fb72 → 0x44fba1),
and only two do: **339 while a conversation `[[0x47770c]+0x9eb8]` is up with its `+0x78` at nought**
(0x410e1c through 0x410e3a), and **340 while the boss slot `[[0x477704]+0x10]` is set** (0x410e52 through
0x410e63).

**The time is the time of the sub the script is in, and not the stage's.** `call` (11, 0x44df70) pushes the
caller's time and place and starts the sub at time nought (0x44e0d3 through 0x44e15a); `return` (10, 0x44e20c)
pops them back (0x44e222 through 0x44e250); a jump (12, 0x44e40f, and 13 and 14 on what they pop) sets the time
to the number the jump carries at `+0x14` (0x44e411). So a loop of two frames reads 1, 2, 1, 2.

**The stage's own script is `main`** (`0x46d278`): on a stage's first frame — the game object's `+0x14` at
nought (0x41819b) — every enemy is destroyed (`0x409f90`) and `0x40cfb0` spawns `main` into the emptied list
(0x41828a through 0x418299), appending at the tail (0x40d162); the list's head changes only where the head is
taken off (0x40daf4 through 0x40db09), so `main` is the head while it lives. Nothing keeps a pointer to it: the
spawner's answer is dropped at 0x41829e.

**And `main` does not always run the stage.** Measured: stage 2's `main` sat on one jump for the whole stage,
its time 1, 2, 3 and round again, while the enemy it had spawned on the stage's first frame — flags `0x110`, no
body (`0x10`) and no life — ran the stage, its time climbing 301, 601 and on to 5864, where it stood on
instruction 340 for the boss. The Extra's `main` sat on a loop of twelve frames the same way. Stage 1's `main`
ran the stage itself, its time standing at 4370 for the boss.

**No clock of the game's stands for a fight.** The stage's frames at `0x474c88` are stepped at 0x418774 alone,
past the build's test (0x41864b) and the pause's (0x4186e2 through 0x4186fd), and through a boss and a
conversation alike: it is put at nought at 0x4178bb, 0x41826c and 0x41846b, and a launch read it at nought from
the frame a stage's build began to its first frame, then 1 — 32 frames between two stages, 55 before a run's
first. The enemy manager's frames at `+0x44` go on through both as well, and take one step more at every
conversation's start (0x40e707 through 0x40e753).

## The items

**`[0x477818]`**: `0x96` items of `0x3f0` bytes from `+0x14` that take every type, and more after them that take
the small faith alone. An item's state is `+0x3dc` — 1 and 2 falling, 3 and 4 flying to the player — its type
`+0x3e0` and where it is `+0x3ac`.

## The window, the device and the frame

WinMain's message loop is `0x438cf1` to `0x438d8f`: `PeekMessageA` (0x438cfb) with `TranslateMessage` and
`DispatchMessageA` behind it; and on nothing pending, `[[0x491c30]+0xc]` (0x438d25), whose answer is held
against `0x88760869` at 0x438d48 — `D3DERR_DEVICENOTRESET`, so that slot is
`IDirect3DDevice9::TestCooperativeLevel` and **`0x491c30` is the device**. Where it answers that, the loop
releases the thirty-two COM objects the sprite manager holds from its `+0x3ad4e0` (0x438a30, at 0x438d54) and calls
`Reset` with the parameters at `0x491d0c` (0x438d66), and where `Reset` succeeds sets the device's render states
again (0x439d20), writes `[0x491fd4] = 3` and sets bit `0x10` of `[0x491ff4]` (0x438d6d through 0x438d84), which
the next frame answering `0` takes off (0x438d3e through 0x438d43). `0x491c2c` is the `IDirect3D9`
`Direct3DCreate9(0x20)` answered (0x438bc3), and `0x491c28` the supervisor, whose `+0x0` is the `hInstance`
(0x438b51). On `D3D_OK` the loop calls **`0x439390(0x4924f0)`** — `push 0x4924f0; call 0x439390` at 0x438d2c — which is the
frame, and **`0x4924f0` is the window**: its `+0x0` is the HWND `ShowWindow` (0x438e55), `MoveWindow`
(0x438e67) and `DestroyWindow` (0x438e73) are handed at the teardown, and `SetForegroundWindow` (0x438cb1).

**The frame is `stdcall`**: its one argument is on the stack (`mov ebp,[esp+0x8]` at 0x439391, after the
`push ebp`) and every way out is `ret 0x4`. So are its two walks — see *The chain* — which is what
`game::FrameEntered::OnTheStack` is for.

The device's vtable is read at twelve slots, every one of them agreeing with `IDirect3DDevice9`: `+0xc`
TestCooperativeLevel, `+0x40` Reset (0x438d66, 0x439232), `+0x44` Present (0x439214), `+0xa4` BeginScene
(0x439496), `+0xa8` EndScene (0x439502), `+0xb0` SetTransform (0x421567), `+0xbc` SetViewport (0x439419),
`+0xe4` SetRenderState (0x420cfa), `+0x104` SetTexture (0x4394f4), `+0x10c` SetTextureStageState
(0x442f68), `+0x14c` DrawPrimitiveUP (0x442fb1) and `+0x164` SetFVF (0x442f8f).

**The frame at `0x439390`, in order**:

- the time, `0x439540`, which answers seconds as a double off `QueryPerformanceCounter` over the frequency
  at `0x492508` — or off `timeGetTime` where the frequency reads nought (0x439569 through 0x4395df);
- **the limiter**: the time is held against `[this+0x48]`, the moment the next frame is due, and a frame
  that comes early goes out through `Sleep(0)` (0x43952b) answering `0`; one that is due steps `[this+0x48]`
  on by the double at `0x470c90`, `1/60`, until it is past the time (0x4393d0 through 0x4393e7);
- `0x442f50` with `esi = [0x491c10]` — the queue of quads drawn, see below;
- `0x421480` with `edi = 0x491e94` into `[0x491fac]`, which sets the camera's two transforms, and then
  `SetViewport([0x491fac] + 0xcc)` (0x439411 through 0x439419);
- `[0x491fb0] = 1` (0x439425);
- **the calc chain**: `0x449c00([0x491be4])` (0x43942f), its answer held in `esi`;
- **the sound**: `0x43ddf0(0x492590)` (0x43943b), `stdcall` on the sound manager — see *The sound* — which runs
  on every frame that walks the chain, before the frame reads the walk's answer;
- the walk's answer read (`test esi,esi` at 0x439440), which is the frame's — `0` from the walk makes the frame
  answer `1` (0x439450) and `-1` makes it answer `2` (0x43946a), both after `0x44c150` with `esi = 0x492254`;
  every other answer carries on. `0x44c150` stops the thread the object at `esi` holds: its handle at `+0x4`,
  `+0xc` set to ask it to stop, `WaitForSingleObject` for 200ms at a time with `Sleep(1)` between until it has
  (0x44c161 through 0x44c19f), and `CloseHandle`. WinMain's teardown after its message loop stops the same thread
  in `0x420270`, called at 0x438d9a with the supervisor in `eax` — the input set-up's thread at `0x474dd4` first
  (0x42027a), then `0x492254` (0x42027f through 0x42028e) — and makes the same call once more in `0x438880`
  (0x4388a8), called after it at 0x438dab; so a frame that leaves without it leaves the thread to be stopped
  there;
- **the frame skip**: the byte `[this+0x14]` counted up and held against the byte `0x491d66` plus one
  (0x439473 through 0x439488), and the drawing below skipped on a frame it is short of;
- `BeginScene` (0x439496);
- the queue's count to nought and its two cursors back to its start (0x4394a7 through 0x4394b7), the dword
  `0x492378` forced to `0xff` (0x4394c2), and `0x420cd0` with `edi = 0x491c28`, which where
  `[0x491c28+0x750]` — that dword — is not nought (0x420cd6) draws the queue (0x420cdb), puts the word at nought
  (0x420ced) and sets render state `0x1c`, the fog, to 0 (0x420cfa); the frame's own store making it do all
  three every frame;
- **the draw chain**: `0x449d40([0x491be4])` (0x4394d8);
- the queue drawn again, `0x442f50` with `esi = [0x491c10]`;
- `SetTexture(0, null)` (0x4394f4) and `EndScene` (0x439502);
- `[this+0x14] = 0` (0x439508) and **the present**, `0x4391f0` (0x43950c);
- the time again, less the frame's start at `[this+0x38]`, into the double at `0x4923a0` (0x439511 through
  0x43951b) — on a frame the skip leaves undrawn as well, the skip's jump landing there (0x439488) — and `0`.
  Nothing in the exe reads `0x4923a0` by its address.

**Which is the update before the draw**, as th08's frame has it, and what orb's own loop over it takes out is
the limiter and the frame skip.

**The present at `0x4391f0` takes nothing and pops nothing** (`ret` at 0x4392af and 0x4392db):
`Present(0, 0, 0, 0)` (0x439214); where that fails, `0x438a30` on the sprite manager releasing its thirty-two
objects (0x439220), `Reset` with the parameters at `0x491d0c` (0x439232), and — whatever `Reset` answers — the
render states set again (0x439d20, at 0x439235) and `[0x491fd4] = 2` (0x43923a); then `0x438a60` with
`esi = [0x491c10]`, and the screenshot — `snapshot/th%.3d.bmp` (`0x46f6d0`), the first of a
thousand names not yet taken — where bit `0x800` of `[0x474e36]` is down (0x439250).

### The queue of quads the drawing fills

**`0x491c10`** holds the sprite manager, a `0x732460`-byte allocation WinMain takes at 0x438c48 and builds
with `0x445900`. `0x442f50`, entered with the manager in `esi`, draws what is queued in it: nothing where
`[esi+0x3adac8]` — the count — is nought (0x442f58), and otherwise two `SetTextureStageState`,
`SetFVF(0x144)`, `DrawPrimitiveUP(4, count * 2, [esi+0x72dad0], 0x1c)`, the drawn-from cursor
`[esi+0x72dad0]` up to the write cursor `[esi+0x72dacc]`, the count to nought and `[esi+0x58]` one higher
(0x442f5a through 0x442fd1).

## The chain

**`0x491be4`** holds the chain, a `0x48`-byte allocation WinMain takes at 0x438ba5 and builds with `0x449aa0`,
which is entered with it in `eax`: the calc list's head is an element at `+0x14` whose `+0x0` points back at
the chain (0x449ab9), and the draw list's is at `+0x38` (0x449ac2 through 0x449ad7).

**`0x449c00` walks the calc list and `0x449d40` the draw list**, each `stdcall` on the chain (`mov eax,[esp+0x1c]`
after four pushes and `sub esp,0x8` at 0x449c30; `ret 0x4` at 0x449d0e and 0x449e2e), each under the critical
section at `0x492274` with the byte `0x49231c` counting its depth, which each leaves around every callback it runs
(0x449c65 and 0x449c7a, 0x449d9b and 0x449db0). A list element is three words — the job at
`+0x0`, the next element at `+0x4`, the one before it at `+0x8` — and a walk starts from `[chain+0x18]` for
the calc list (0x449c34) and `[chain+0x3c]` for the draw list (0x449d6a).

**A job is `0x24` bytes** (the `push 0x24; call 0x452493` that every one is allocated with), made by `0x449ed0`
with its callback pushed (0x449f11):

| | |
| --- | --- |
| `+0x0` | its priority, which the insertion walks the list past while it is the higher (0x449b26, 0x449bb6) |
| `+0x4` | its flags: bit 0 set as 0x449ed0 makes it (0x449f0e), which says the job is freed as it is taken off a list (0x449fb4 through 0x449fc7); bit 1 set while it is to be run (0x449c58, 0x449d90) |
| `+0x8` | the callback, `__thiscall` on `+0x20` (0x449c6d, 0x449c70) |
| `+0xc` | a callback run once as the job is added, on `+0x20`, and cleared (0x449ae0 through 0x449af1) |
| `+0x10` | a callback the walk runs where the job answers `7` (0x449cbb) |
| `+0x14` | the job's own element of the list it is on |
| `+0x20` | the argument its callbacks are called on |

**`0x449ae0` adds a job to the calc list and `0x449b70` to the draw list**, each entered with the job in `esi`
and its priority in `edi` and the chain pushed (`ret 0x4` at 0x449b60 and 0x449bf0).

**What a callback answers is read through a table**, `0x449d14` for the calc walk and `0x449e34` for the draw:

| answer | the calc walk | the draw walk |
| --- | --- | --- |
| `0` | the job taken off (`0x449f60`), and on | the same |
| `1` | on to the next job | the same |
| `2` | the job run again while bit 1 of its flags stands | the same |
| `3` | the walk stops and answers `1` | the walk stops and answers `1` |
| `4` | the walk stops and answers `0` | the walk stops and answers `0` |
| `5` | the walk stops and answers `-1` | the walk stops and answers `-1` |
| `6` | the walk starts again from the first job, its count at nought | — |
| `7` | `+0x10` run, and on | — |

An answer past the table — above `7` for the calc walk, `5` for the draw — counts the job and goes on (0x449c8d,
0x449dc3).

**And what a walk answers otherwise is the count of the jobs it reached** (`inc [esp+0x10]` at 0x449cc7), so
a list of one job answers `1` and an empty one `0` — the same two numbers as a job asking the walk to stop,
which the frame reads as a frame to leave on. A job whose callback at `+0x8` is null is passed over and not
counted (0x449c56, 0x449d8e); one whose bit 1 is clear is counted without being run (0x449c5c, 0x449d94).

## The sound

**`0x492590` is the sound manager**, `0x14b4` dwords WinMain zeroes at 0x438c18 and hands to
`CreateThread(…, 0x43ccf0, 0x492590, …)` at 0x438c21, the thread's handle going into `0x4977a8`. The frame's
call on it, `0x43ddf0`, is `stdcall` (`mov ebp,[esp+0x14]` after `sub esp,0xc; push ebp` at 0x43ddf4, `ret 0x4`
at 0x43de08), answers nought at once where `[this+0x610]` — the DirectSound object — is nothing (0x43de00), and
otherwise works through the commands queued from `[this+0x1f88]`, a table at `0x43e37c` dispatching on each.
`[this+0x5208]` is the music's stream (0x43de45).

**The thread WinMain starts on it is the sound's startup and not its stream**: `0x43ccf0` calls `0x43cd30` on
the manager, waits for `[0x4977b4]` to be set and ends, writing `[0x4977bc] = 1` (0x43cd1e). A second thread,
`0x43d080` (created at 0x43d105, handle into `0x4977ac`), loads the thirty-seven sound effects named from
`0x474b40` into `0x4977c0`. **The music's own thread is `0x43e3a0`**, made where a stream is opened
(0x43d70d): its id goes into `[this+0x614]` — `lea eax,[esi+0x614]` is the `lpThreadId` pushed at 0x43d6fa —
its handle into `[this+0x618]` (0x43d71b), and the event it waits on into `[this+0x520c]` (0x43d6ee).

### Which track is playing, by name

**`[this+0x1f84]` is `thbgm.fmt`**, records of `0x34` bytes: the start-up loads `../../bgm/thbgm.fmt` (0x46dc14)
through 0x44b360 (0x41fa28) and stores it at `0x494514` (0x41fa34), which is `0x492590+0x1f84` — the store
0x43d670 makes at 0x43d684, inlined; 0x43d670 itself has no caller. A record starts with the track's file name,
16 bytes with its terminating zero, and has where the track is in `thbgm.dat` at `+0x10` and how long it is at
`+0x14` (the preload reads `+0x14` bytes from `+0x10`, 0x43d8b0 through 0x43d8fe). A record whose first byte is
zero ends the table (0x43d30b, 0x43d353).

**`0x43d2a0` finds a record by name**, the path in `ecx` and the manager on the stack (`[esp+0x90]` at
0x43d2fd): it cuts the path past its last `/` (`strrchr`, 0x452930, at 0x43d2b1) or, where there is none, past its
last `\` (0x43d2c9), copies what is left to the stack, and compares it with each record's start, two bytes a
round to the terminating zero (0x43d317 through 0x43d339). It answers the record's index, and **0 where none
matches** (0x43d35d through 0x43d364). Three calls: 0x43d7a3, 0x43d8a0 (the preload into memory), 0x43e155.

**The manager holds two songs a stage loads, as paths**, `0x100` bytes apiece from `[this+0x4108]`:
- **0x420a90** (`stdcall`, slot and path) copies the path, writes `wav` over what follows its last `.`
  (0x420abd through 0x420ad8) and queues command 1 with the slot (0x420ae7). The command carried out
  (0x43de7f through 0x43de9d) is `0x43d7d0`, which copies the path into the slot (0x43d824 through 0x43d83b —
  `0x496698` is `0x492590+0x4108`) whatever else it then does.
- **0x420b10** (`stdcall`, slot and a number) queues command 2 with the slot (0x420b3d) and sets the score
  object's byte `+0x1d892` of the number (0x420b4c). The command carried out takes the slot's path (0x43e13e
  through 0x43e148 — or the command's own name where the slot is negative, 0x43e151), finds its record
  (0x43e155), and puts the stream's source on it, `0x44dca0` writing it to `[source+0x90]` (0x43e15c through
  0x43e170, 0x44dcba) — only where the source's `+0x7c` is nought and its `+0x8c` is not -1, answering
  `E_FAIL` and writing nothing otherwise (0x44dca0 through 0x44dcb7).

**A stage's two songs are its record's**, `[0x477848]`, `0x474788` on by `0x30` a stage, whose `+0x10` and
`+0x14` are paths: the build loads `+0x10` into slot 0 and `+0x14` into slot 1 after the stage's objects, either
way the stage was built and not for the attract demo (`0x474ca0 & 0x20`, 0x417ba6 through 0x417bd1). The stage
begun plays slot 0 with its record's number at `+0x24` (0x4183d6 through 0x4183e1, again unless the demo), and
the conversation before the stage's own boss plays slot 1 with the number at `+0x28` (0x416a91 through
0x416a9c) — **so slot 1 is the fight's song.** Stage 1's are `bgm/th10_00.wav` and `bgm/th10_01.wav` (0x46ecb0,
0x46eca0), stage 2's `bgm/th10_03.wav` and `bgm/th10_04.wav`, on to the Extra's `bgm/th10_15.wav` and
`bgm/th10_16.wav`; the title loads `bgm/th10_02.wav` (0x46f018) into slot 0 and plays it (0x42d013 through
0x42d023). The stage records can be listed with:

```sh
i686-w64-mingw32-objdump -s --start-address=0x474788 --stop-address=0x474908 th10.exe
```
