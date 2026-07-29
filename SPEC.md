# Ultramarine Orb Elixir — specification

What orb does, and the facts about 東方紅魔郷 1.02h and about Windows that shape how. The
README is the guide to using it; this describes the thing itself.

**This document describes the final form only.** No history of what was tried and rejected,
and no record of what a mechanism used to be — only what it is, and the facts it rests on. The
reasons an alternative was rejected belong in a comment beside the code that would otherwise
tempt someone back to it, which is where they are. What has been checked and how is in
[DONE.md](DONE.md); what is left is in [TODO.md](TODO.md).

Only 1.02h (`md5 fa3d64768b1bfc50703dedc2db92f7fa`). Every address below was read off that
build and cross-checked against the
[GensokyoClub/th06](https://github.com/GensokyoClub/th06) decompilation.

## Chapters and retries

A chapter begins at a stage's start, when a boss appears, and at each boss attack.
Midstage boundaries come from a compiled-in table of script frame numbers, because a
stage's waves run on a clock and are reproducible. Boss boundaries are detected as the game
runs, so a difficulty with an extra attack gets an extra chapter with no table.

A retry restores a whole memory snapshot rather than asking the game to jump anywhere.
Nothing has to know what a boss script was in the middle of.

A snapshot covers:

- the exe's `.data`;
- the regions the game's CRT took from the OS, found by hooking `HeapCreate`,
  `HeapAlloc`, `HeapReAlloc`, `HeapFree` and `VirtualAlloc` in the exe's import table and
  walking the heaps;
- the music's stream position.

Direct3D and DirectSound objects are not saved. The game loads a stage's resources on entry
and creates nothing within a chapter, so restoring the pointers in `.data` leaves them
naming the same live objects. Since those addresses differ per launch, a snapshot is only
valid inside the process that took it, and chapter progress cannot be written to disk.

Restoring suspends the game's other threads, writes each region back, and resumes. The audio
thread is left running; the music is handled separately, rewound for the midstage and the
midboss and left playing through a boss-fight restore.

Two generations are kept: the current chapter and the stage's start, matching the two
choices the retry menu offers.

## The frame loop

```
prepare_frame          the game's full-output viewport, and its background clear
DwmFlush()             returns just after a blank
sleep                  until (blank + one frame) − (measured work + handover room)
update(chain)          the game's logic; reads the keyboard as its first act
draw(chain)            between BeginScene and EndScene
Present()              handed over, not waited on
```

The update runs before the draw, which is the opposite of the game's own order and removes a
frame of lag. The work is done at the end of the frame's turn rather than the start, so the
input it reads is as recent as it can be and still reach that frame.

`Present` does not wait for anything: windowed with `D3DSWAPEFFECT_COPY` it queues the frame
and returns. `DwmFlush` is the call that waits for the compositor, and it sits at the top of
the next frame — so a frame reaches the screen when the *following* frame's flush comes back,
and that is where input-to-screen is measured from and to.

**Cadence.** One game frame is a whole number of refreshes of the monitor the game's window
is on, taken from `MonitorFromWindow` and `EnumDisplaySettingsW`: two at 120Hz, one at 60Hz.
The compositor's `qpcRefreshPeriod` follows one monitor of the desktop, which on a mixed-rate
desktop need not be this one, so it is used only when it agrees with that rate. A rate that
is not a whole multiple of 60 has no fixed cadence to keep — 144Hz is 2.4 refreshes a frame —
and neither has a replay being run fast; both are paced by the clock instead.

`DWM_TIMING_INFO.cRefresh` counts compositions of the window rather than refreshes of the
display, and is not used.

**The handover room.** How long before the blank a frame must be handed over is not something
the compositor will say, so it is found by trying: `DWM_TIMING_INFO.cFramesLate` — its own
count of frames it could not show at the refresh they were aimed at — going up widens the
room, and staying flat shaves it back. Every microsecond of it is input lag, and every one
too few is a frame shown a refresh late.

**The work estimate.** How long the frame's work takes is measured and tracked near the worst
of the recent frames rather than their average, because aiming at the average means missing
the handover on every frame heavier than it.

`timeBeginPeriod(1)` is asked for at startup and released on detach. Without it `Sleep` is
only accurate to the system tick, some fifteen milliseconds.

## Input

`Controller::GetInput` (0x41d820, `__stdcall`) is the one place every key the game sees passes
through: `Supervisor::OnUpdate` calls it once a frame, and the only other writer of
`g_CurFrameInput` is replay playback.

**Not read while the window is behind.** The game's keyboard is a foreground DirectInput
device — `DISCL_NONEXCLUSIVE | DISCL_FOREGROUND`, acquired once at startup — so the system
unacquires it whenever the window goes behind and every read fails. The game checks only for
`DIERR_INPUTLOST` and treats anything else as a success, which hands it an uninitialised stack
buffer as key state. Reading while behind also spends that single `DIERR_INPUTLOST` report on
a moment when re-acquiring cannot succeed.

**Re-acquired on the way back.** orb calls `Acquire` itself — `g_Supervisor.keyboard` at
`G_SUPERVISOR + 0x10` (0x6c6d28), vtable slot 7 — and holds the keys back until it succeeds.

**The joystick is optional.** `Controller::GetControllerInput` (0x41cfc0, `__cdecl`) is a tail
call inside `GetInput` that adds a joystick's buttons to the keyboard's. It costs 9.3ms a
frame where the device does not answer, against about a microsecond for the keyboard, because
the game asks every frame and retries `Acquire` when it fails. `joystick: false` hooks it to
return the keyboard's buttons unchanged, which is what it does when there is no joystick.

## Borderless fullscreen

The arguments of the game's own `CreateWindowExA` are rewritten on the way through, so the
window is borderless and covering the monitor from the moment it exists — no frame to remove
afterwards and nothing to flash first. Its window class gets a black background brush, and
that is the letterbox.

The back buffer stays 640x480. Windowed, the game asks for `D3DSWAPEFFECT_COPY`, the swap
effect that honours a destination rectangle on `Present`, so the back buffer goes into a
centred rectangle of the game's aspect ratio.

A monitor-sized back buffer would not scale the game: its 2D drawing uses `D3DFVF_XYZRHW`,
whose coordinates are already screen space, so a viewport does not transform them. Scaling
that way would need a render target and a full-screen quad.

## The status line

`CH`, `RETRY`, `INPUT LAG` and the frame rate are drawn with GDI onto the window, in the black
beside the game, stacked in whichever letterbox bar is wider and lined up with the game's
edge. On a 16:9 monitor a 4:3 game leaves that black down the sides. Shown whatever the game
is doing, including demos and menus, and redrawn only when the text changes.

The game's back buffer is the wrong place for it twice over: all 640x480 of it is shown inside
the letterbox, so anything there is over the game; and the game does not clear it between
frames — it draws the static background once and redraws only the moving parts — so anything
in a corner it does not repaint stays there and accumulates. The black is the window's own
background and Direct3D never touches it.

The retry menu is drawn with the Direct3D overlay instead, because it belongs over the game.

`INPUT LAG` is measured, from the keyboard read to that frame being on screen, across the two
frames those ends fall in.

## The ending

The ending is run out inside the frame it starts on, so no frame of it is drawn, rather than
jumped over — the ending is also where the game sets the clear flag and enters the score.
Bounded at two minutes of game time, stopped by the scene changing, and never entered during a
demo or a replay.

`g_Supervisor.isInEnding` is at `G_SUPERVISOR + 0x19c` = 0x6c6eb4. The same flag gates the
game's own frame-rate counter (`cmpl $0x0, 0x6c6eb4` at 0x4240bb, jumping over the
`AsciiManager::AddString` at 0x4240e9), so it must not be written to hide that counter. The
counter is left alone; orb's numbers are outside the game's output.

## A latent crash in the game

`ResultScreen` passes `g_CharacterList[charUsed * 2]` as a *format string* to
`DrawStringFormat2`. `g_CharacterList` is `char*[6]` at 0x4784d8; `charUsed = 5` reads
0x478500, which holds the float 2.5 — the address `0x40200000`. `charUsed` is the shared
result-screen cursor, and erratic input can leave it outside the 0..2 this screen assumes.
Not orb's bug, but a reason orb must never feed the game stray input.

## How it is installed

`orb-launcher.exe` is the only file. It carries `orb.dll` inside itself — cargo's artifact
dependencies (`-Z bindeps`, which is why the toolchain is nightly) build the cdylib first and
hand over its path — and unpacks it to `%TEMP%\orb` before injecting, since `LoadLibrary`
needs a path.

The unpacked file is named for its own checksum rather than reused, because a mapped image
cannot be replaced while it is loaded. Stale copies are removed by the next launch that finds
them unloaded.

`orb.yaml` and `orb.log` are therefore found in the directory of the exe this is running
inside — the game's, which is where the launcher is installed too — and not relative to the
DLL.

One DLL covers every game it is taught, and is not split per game the way vpatch's is. vpatch
patches per-game code; orb's per-game part is a `Game` implementation, which is a table of
addresses and a handful of accessors. Splitting would copy the snapshot, chapter, retry and
frame-loop code into every DLL and make the launcher carry several payloads.

## Configuration

`orb.yaml`, read from the game's directory by both halves, so each has to know every key. A
flat list of `key: value`; an unknown key is an error rather than being ignored. What each key
does is in the comments of the file.

Two worth naming here:

- `joystick: false` — skips the game's joystick read; see *Input*.
- `log_level: quiet | normal | verbose` — `quiet` is startup and faults; `normal` adds a line
  a second on the frames and the sound and a line per scene; `verbose` adds a line per frame
  that did not come out on the cadence, saying where that frame's time went.

`orb.log` is appended to rather than started over, because a run worth looking at is usually
over before anyone looks. A crash adds a line naming the faulting module and offset.

## Building the midstage table

Boss boundaries need no table; a stage's waves are a script on a clock, so those boundaries are
frame numbers someone has to pick. With `chapter_tuning: true`:

- boundaries are proposed as you play, at the quiet moments between waves;
- `tuning_add_key` marks the current moment, for a gap the detector misses;
- `tuning_remove_key` drops the most recent mark;
- `tuning_write_key` writes `chapters.rs` beside the launcher.

Paste that over `crates/orb/src/game/th06/chapters.rs` and rebuild. Stages not tuned in a
session keep whatever is compiled in, so they can be done one at a time. With `during_replay: true` and `replay_speed` above 1, a replay of a full
run can do the playing.

## Checking the snapshot engine

`save_state_key` and `load_state_key` save and restore by hand at any point.

`self_check: true` additionally restores every snapshot immediately after taking it and
compares the result, and reports memory that changed since a snapshot without being covered by
it. It fingerprints every private page in the process, so it pauses the game for as long as
that takes.

`stress_restore_frames` restores the current chapter every N frames, a few times per chapter
and then moving on, so a replay walks through the midstage, the midboss and every boss attack
restoring as it goes.

## How it fits together

`orb.dll` is injected while the game is still suspended, so its `DllMain` runs before the
game's entry point and the memory hooks see the first allocation.

| | |
| --- | --- |
| `crates/launcher` | checks the exe, starts it suspended, injects `orb`, resumes it |
| `crates/orb-config` | `orb.yaml`, shared by both halves |
| `orb/hook.rs` | trampoline and import-table hooks |
| `orb/memtrack.rs` | import hooks recording the heaps and reservations the game takes from the OS |
| `orb/snapshot.rs` | save and restore of `.data`, those regions, and the music |
| `orb/audio.rs` | the sound buffer and file position, which live outside the game's memory |
| `orb/frame.rs` | the frame loop's pacing and its measurements |
| `orb/chapter.rs` | where chapters begin, and which snapshots are kept |
| `orb/retry_ui.rs` | the menu shown where the chapter was lost |
| `orb/tuning.rs` | building the midstage table |
| `orb/window.rs` | the borderless window, the letterbox, the status line |
| `orb/overlay.rs`, `text.rs`, `d3d8.rs` | drawing over the game's frame |
| `orb/game/mod.rs` | `Game` and `State`: everything above is written against these |
| `orb/game/th06/` | the addresses and offsets that make it 東方紅魔郷 |

Only `th06` implements `Game`. Porting to another Touhou game means supplying its addresses
and offsets.

## Not supported

- Chapter progress across launches.
- Scores and rankings are recorded as the game sees them; replay files are not written at all,
  because a rewound run does not play back.
- Sound effects cut off on a restore rather than rewinding. Only the music is restored.
