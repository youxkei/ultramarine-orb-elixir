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

A chapter begins at a stage's start, at each attack of a fight — the boss's timer being reset,
which a spellcard beginning does too — and when a midboss is beaten, since that hands the stage
back and the waves after it want a chapter of their own. Not when a spellcard ends: either the
fight is moving on, which the timer says on the same frame, or the boss has just been beaten.
Not when the stage's own boss is beaten either: the stage is over, and a chapter whose first act
is that the fight is already won has nothing to retry.

Dialogue and a pause hold a boundary back, since neither is the run getting anywhere; a bomb and
a death do not, though a chapter beginning under either is a poor place to restart from. A
boundary the fight really has is worth more than one that is always comfortable, and the way out
of a bad start is the chapter before it. What the fight did while a boundary was held back is
taken as soon as one is allowed, so nothing is lost to the seconds a bomb or a death lasts.

The log names what began each chapter, so one that turns up where none belongs says which signal
produced it. Midstage boundaries come from a compiled-in table of script frame numbers, because
a stage's waves run on a clock and are reproducible. Boss boundaries are detected as the game
runs, so a difficulty with an extra attack gets an extra chapter with no table.

**A chapter beginning washes the play field green**, so that where dying will send you back to
is something seen rather than a number to read off the status line. Dim enough to read the
bullets through and gone inside a sixth of a second, because a chapter begins every few seconds
through a fight and the frame one lands on is a frame being dodged on. Not a stage's own start:
that is already unmistakable from the title, the music and an empty field, and a wash over its
first frame says nothing that was not obvious. Green because the game flashes white itself — a
bomb, a boss going down — and a mark that means something orb decided should not look like
something the game did; nothing in 紅魔郷 fills the play field with green. It holds a moment
before it fades, since a wash that starts fading at once reads as dim however bright its first
frame is. The play field and no further, because that is the part the game repaints every frame:
the panel beside it and the border around it are not, so a wash drawn there would never be drawn
over and would stay for good. The fade is counted in frames drawn rather than in the game's,
which is stopped wherever a step is holding the game still.

`boundary_flash: false` leaves the screen with nothing of orb's over the game. Only a run
somebody is playing is marked, and a judging pass — see *Building the midstage table* — never a
replay running for anything else: a collecting pass crosses a boundary every few drawn frames,
and a green field for twenty minutes is a mark to nobody.

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

Restoring suspends the game's other threads, puts back any page of a saved region that has
gone since — the game freeing a few megabytes hands part of a segment back to the OS, and the
copy has to have somewhere to write — writes each region back, and resumes. The audio thread
is left running; the music is handled separately, rewound for the midstage and the midboss and
left playing through a boss-fight restore.

**A track change is handled by the game, not by the copy.** Restoring a snapshot taken under
another track — the stage's start, once a boss has brought its own music — cannot put that
track back: the game freed its stream and released its sound buffer, and no memory copy brings a
released COM object back. Neither is the memory restorable around the stream that replaced it,
since that one was allocated after the snapshot: writing the snapshot back rolls its object out
from under the streaming thread, which is not suspended, and that was measured as an access
violation inside `DSOUND.dll` writing a buffer it no longer owned. Skipping the live ranges only
moves the problem, because the heap's bookkeeping still rolls back to before that stream existed
and the next track change then frees a block the allocator has been told is free.

So the game is asked to do it:

- `SoundPlayer::StopBGM` (0x430f80) before the copy, while the game's memory is still its own —
  it stops the buffer, quits and joins the streaming thread, and deletes the stream, which is
  the one free its allocator agrees with;
- the copy, with nothing held back;
- `backgroundMusic` and `backgroundMusicThreadHandle` cleared, because the restored state names
  the stream that was playing when the snapshot was taken and that one is long gone;
- `Supervisor::PlayAudio` (0x424b5d) with `g_Stage.stdData->songPaths[0]`, the stage's own
  track, read out of the memory that was just restored.

The track therefore starts again from its beginning rather than from where it was, which is what
a midstage restore does to the music anyway.

Two generations are kept: the current chapter and the stage's start, matching the two choices
the retry menu offers. Of this stage only — a snapshot of an earlier one would name Direct3D
textures the game released when it loaded this stage, and reloading them is not enough to make
it whole: an `AnmVm` holds its script as a raw pointer into the file's buffer, so a file
loaded again at another address leaves every live one of them pointing at nothing. Measured as
a jump to 0x00000000 on the frame after such a restore.

It is kept across a stage transition, which is not the run ending however much it looks like
one from outside: the game leaves the gameplay scene for `GAMEMANAGER_REINIT` while it tears
the last stage's managers down and builds the next one's. Only leaving the run for good takes
it.

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
and is paced by the clock instead. A replay being run fast, or a run being cleared fast, keeps
the cadence like anything else: `--speed` is updates per drawn frame, so the frames come one per
turn and only carry more of the game with them.

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

The chapter, `RETRY`, `INPUT LAG` and the frame rate are drawn with GDI onto the window, in the
black beside the game, stacked in whichever letterbox bar is wider and lined up with the game's
edge. `SCRIPT` and `TABLE` join them while a table is being built, and `HOLD` while the game's
update is held. On a 16:9 monitor a 4:3 game leaves that black down the sides. Shown whatever
the game is doing, including demos and menus, and redrawn only when the text changes.

**In a run the chapter is named for the part of the stage it is in, and numbered inside it**:
`MIDSTAGE 3`, `MIDBOSS SPELL 1`, `BOSS NONSPELL 2`. A count of the chapters gone by says
nothing about where the game is standing, while the second spellcard of a fight is how a fight
is talked about and how a chapter worth grinding is named to somebody else. *Midstage* rather
than "stage" because "STAGE 3" beside a stage number reads as the third stage. The waves are
one run of chapters through the whole stage — its start, each midstage boundary, and the waves
handed back when a midboss goes down — since a midboss interrupts them rather than starting a
new set: the chapter after the midboss of stage 4 is `MIDSTAGE 3`, not `MIDSTAGE 1` again.
Where there is no chapter, in a menu, that line is not there at all.

The count comes out of the frames this stage's chapters began at rather than a counter per
kind, because a retry puts the mark back and the run then reaches the same chapters again. A
counter would name the same chapter one further along every time it was played.

**A pass building the table shows `CH 05` and why the chapter changed instead** — see *Building
the midstage table*. That is a different question from what the chapter is, and the one worth
having while what is being decided is whether the boundary belongs in the table: the name would
say the same thing about a boundary to keep and a boundary to throw away.

**Each draw clears what it is going to write in, and everything the last draw wrote in, to
black first.** A stack with fewer lines than the one before it, or a smaller font, does not
reach every row that already had text in it. Clearing and writing are two operations, so both
go into a bitmap of their own and reach the window as one `BitBlt`: done straight on the
window, a refresh landing between them shows a bar with nothing in it, and at 120Hz that is a
flicker. A stack taller than the black there is runs off the edge of that bitmap and is
clipped there, rather than being drawn over the game.

**The size fits the widest line into the black there is.** What goes here runs from three
characters to twenty — a chapter named for the part of a stage it belongs to is the long one — and
a line clipped at the bar's edge cannot be read at all, which is worse than small. The line is
measured at the full size and the height scaled by what it overran by, once per change of text,
which lands inside a pixel or two because character widths go with the em height. Nothing shrinks
below the point where the text stops being readable anyway; past that, clipping the odd long line
is the better trade.

The game's back buffer is the wrong place for it twice over: all 640x480 of it is shown inside
the letterbox, so anything there is over the game; and the game does not clear it between
frames — it draws the static background once and redraws only the moving parts — so anything
in a corner it does not repaint stays there and accumulates. The black is the window's own
background and Direct3D never touches it.

The retry menu is drawn with the Direct3D overlay instead, because it belongs over the game.

`INPUT LAG` is measured, from the keyboard read to that frame being on screen, across the two
frames those ends fall in.

## The ending

With `skip_ending`, the ending is run out inside the frame it starts on, so no frame of it is
drawn, rather than jumped over — the ending is also where the game sets the clear flag and
enters the score. Never entered during a demo or a replay.

**The staff roll is kept**, which the scene cannot say: an ending and its roll are all one
scene, so what the skip stops on is the script the ending is reading. 紅魔郷 runs an ending
from a `.end` file of one-character instructions, one file per part of it, and the last
instruction of every one of the six — `end00`, `end01`, `end10`, `end11`, and the
`end00b`/`end10b` a clear on Easy or with a continue gets — is `@Fdata/staff00.end`, the roll.
`F`, at 0x40fc06 in the interpreter at 0x40f7c0, reads that file over the one running and
carries straight on into it, so nothing else marks the handover: the scene stays 10 and
`isInEnding` stays set across both. The skip stops on the update the file changes on and the
roll plays from there a frame at a time, with the track it starts for itself — the ending plays
`bgm/th06_16`, `staff00.end` starts `bgm/th06_17` three instructions after the handover and in
the same update as it, so the track changing is a second signal for the same boundary.

The script is read out of the `Ending` object, which is on the heap and in no global: it is
reached by walking the calc chain for the job whose callback is `Ending::OnUpdate` at 0x4109c0
and taking that element's argument at +0x1c, the same field `RunCalcChain` passes to the
callback. The file it loaded is at `Ending + 0x1114`. `Ending::LoadEndingFile` at 0x4106d0 reads
the new file before it frees the one it replaces, so the address is always a different one
after a handover — which is what makes comparing addresses enough. With no script to compare,
the skip runs the scene out the way it did before there was one to read.

**A stage 6 ending is 29,040 updates** on its own and 36,932 with the roll, after which the scene
is 7, the result screen. So the limit on the loop is above a whole ending — fifteen minutes of
game time — because it is a limit per frame rather than on the skip: the frame the loop stops in
goes on to draw whatever the ending is showing by then, which at two minutes put five frames of
it on the screen. An update of an ending cost 13µs when the whole 36,932 were timed, which puts
29,040 of them under 400ms; what the frame the skip runs in actually took is not measured, only
that it was five refreshes or more.

**Reaching one at all** takes clearing the game: `GameManager`'s end-of-run branch at 0x418f4e
sends a replay to state 8 and practice to the result screen, and only a run somebody played gets
state 10. So `--clear` — see *Configuration* — is how the ending is looked at without half an
hour of playing well first.

`g_Supervisor.isInEnding` is at `G_SUPERVISOR + 0x19c` = 0x6c6eb4. The same flag gates the
game's own frame-rate counter (`cmpl $0x0, 0x6c6eb4` at 0x4240bb, jumping over the
`AsciiManager::AddString` at 0x4240e9), so it must not be written to hide that counter. The
counter is left alone; orb's numbers are outside the game's output.

## The score file

A run with chapter retries is not a run anyone played, so its score does not belong in the
game's ranking — the same reason replay files are not written. Refusing the write would lose
the record altogether, so with `own_score_file` the file is forked: every open of `score.dat`
becomes an open of `orb_score.dat`, chapter-mode runs are ranked against each other in the
game's own format and on its own screen, and `score.dat` comes out of a session unchanged
because it is never opened.

orb's file starts as a copy of the game's, so that what a `score.dat` has already unlocked —
practice on a stage that has been reached, the Extra stage — is not locked again by playing
through orb. Only where there is nothing there yet, which `CopyFileA` with `bFailIfExists`
decides for itself; after that the two are separate records and the game's is never read
again.

**At the exe's import of `CreateFileA`**, not at the game's own score code, because that
import is where both of the game's own paths to the file end up. `score.dat` is one string, at
0x46af94, pushed at four places: three reads through the helper at 0x42b0d9, which decrypts
what it read (from 0x41bcdc, 0x42f47f and 0x43a5c0), and one write through
`FileSystem::WriteDataToFile` at 0x41e460, which encrypts before it writes (from 0x42bc15).
Both of those open with the CRT's `fopen` at 0x45ca0b — `"rb"` and `"wb"` — and the CRT is
statically linked, so the open reaches the OS at the exe's own import, IAT slot 0x46a150. That
slot is called from exactly two places: the CRT's open at 0x4677fa, and a file class at
0x43ceea that the score file never goes through.

So nothing here is per-game: no address, no offset, and nothing about the format or the
encryption. It is the seam `memtrack` hooks for the heap calls, and d3d8's and dsound's own
opens go through their own imports and are not in the path.

**Only the open is redirected, which is enough here and would not be everywhere.** That other
file class truncates a `"w"` by calling `DeleteFileA` on the name before creating it — at
0x43cea9, its only call site in the exe — so a game whose score write went that way would have
its own file deleted while orb's was written. 紅魔郷's does not go that way.

The whole file name is compared, ignoring case, and the directory the game named is kept. So
`orb_score.dat` is not itself taken for the game's file and forked again, and a relative name
resolves where the game's own open would have resolved it. The paths are handled as the bytes
the game gave: a directory name in the game's code page is not necessarily UTF-8, and the
copy goes through `CopyFileA` rather than `std::fs` for that reason. They are converted to
text in one place, the log.

With `--no-chapters` the fork is not installed: nothing can rewind that run, and its score
belongs in the game's own file.

**With `--clear` nothing is written**, whichever way `own_score_file` and `--no-chapters` are
set — there the hook is in the path for the refusing rather than for the forking, and the file
that must not be written is the game's own. A cheated clear is not a score: orb's file is where
runs that cannot be compared with the game's are kept, and a clear nobody could have played at
the top of *that* ranking is the same mistake one file further on. The open for writing is
refused rather than the write being sent elsewhere, which the game takes cleanly:
`WriteDataToFile` checks its `fopen`, returns -1, and its one caller — the `call` at 0x42bc1a —
drops that and frees the buffer either way. Reads go through, so the ranking screen and what the
file has unlocked are what they were.

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

Three keys are about this rather than about behaviour: `game_dir` says where the game is when
it is not beside the launcher and `orb_dll` names a DLL to load instead of the carried
one.

One DLL covers every game it is taught, and is not split per game the way vpatch's is. vpatch
patches per-game code; orb's per-game part is a `Game` implementation, which is a table of
addresses and a handful of accessors. Splitting would copy the snapshot, chapter, retry and
frame-loop code into every DLL and make the launcher carry several payloads.

## Configuration

**Two places, split by who sets them.** `orb.yaml` holds what somebody playing sets and leaves
set: `borderless`, `skip_ending`, `joystick`, `block_replay_save`, `own_score_file`,
`boundary_flash`, `own_frame_loop`, `always_draw`, and where the game and an override `orb.dll`
are. A flat list of `key: value`, and an unknown key is an error rather than being passed
over — a setting that is quietly not read is a setting somebody thinks is on.

Everything to do with building the midstage table, reaching an ending, or looking into a fault
is an argument to `orb-launcher` instead — `--help` lists them — because a file is the wrong
place for something that is different every time it is run. The two passes over a replay are one
word each:

| | |
| --- | --- |
| `--collect` | propose boundaries over the whole replay, at 64 updates a frame, with nothing stopping and nobody at the keyboard |
| `--judge` | step between them at one update a frame and decide about each: the pass somebody watches |

and beside them `--tune`, `--replay`, `--speed=N`, `--log=quiet|normal|verbose`, `--self-check`,
`--stress=N`, and `--no-chapters`, `--no-memory` and `--no-hooks` for taking orb apart until a
fault stops happening. `--config=PATH` is the launcher's own.

**`--clear`** is the one that is about neither: nothing can hit the player and the run goes at
`--speed`'s 64 updates a drawn frame, so half an hour of playing well is a minute of holding the
shot key. It is there because the ending is reached by clearing the game and by nothing else —
see *The ending* — and a boundary inside one cannot be looked at twice at half an hour a look.
What it writes, before each update, is the player's state — to the one the game's own bombs
write — and the frames of invulnerability left under it. Both, because the state on its own does
not last: `Player::OnUpdate` is chain priority 7 and the bullets are checked at 11, so a state
whose frames have run out is a state put back to normal before the hit test in the same update,
and the player dies with it written. Only from the states the player can be hit in or is already
invulnerable in, so a spawn or a death still plays out as itself.
It leaves no record: no score file is written at all — see *The score file* — because such a run
has no score to keep in either ranking, and no replay is written whichever way
`block_replay_save` is set, because a replay holds the inputs and nothing about the player having
been unhittable, so playing one back is a run that dies where this one did not.

The launcher reads them, refuses to start on anything it cannot, and hands them on the game's
own command line — which the game never looks at, `lpCmdLine` appearing once in the whole of its
`WinMain` as the parameter it ignores. orb reads them back off that inside the game, so the two
sides cannot disagree and nothing has to be written down between them.

**The keys are fixed in the code**: `a` adds a boundary, `d` writes the files, the arrow keys
step and judge, `space` holds, `shift` and `ctrl` change what stepping means. Whoever is
building a table is the only person who presses any of them, and a setting nobody changes is one
more thing that can be wrong.

`orb.log` is appended to rather than started over, because a run worth looking at is usually
over before anyone looks. A crash adds a line naming the faulting module and offset.

## Building the midstage table

Boss boundaries need no table; a stage's waves are a script on a clock, so those boundaries are
frame numbers someone has to pick. Under `--tune`, boundaries are proposed as the
stage is played, a second into each gap between waves: nothing left to shoot at, no boss fight
to interrupt, and two seconds at least into the chapter now running. One per gap, on the frame
the gap becomes one, so a long lull does not fill up with them.

Bullets in the air are not part of the test. The game was not written around chapters, so a
frame with none of them is rare — asking for one found three places in a stage where a retry
unit wants a dozen — and a snapshot restores whatever is in the air exactly, so a boundary does
not need the screen to be clear. What it needs is a gap in the script, and enemies are what say
that.

**One table for every difficulty**, because the clock it is written in does not move with the
play. The enemy timeline runs through the midstage and through a midboss fight, and stands
still only while the stage's own boss is fought: on stage 1 the chapters at script 2009, 2666
and 3180 track the stage's own clock exactly across the midboss, while every chapter of the
boss fight reads 5282. So a frame number always names the same point of the script, however
fast the midboss was killed and whatever a difficulty adds to it.

What differs is what is on screen at that point: a midboss still alive because this run was
slow, the leftovers of a spellcard a harder difficulty adds. A boundary worth keeping
therefore sits in a gap the script leaves rather than one a fast clear happened to open, and
the `tuning: gap of N frames` line is what tells them apart — a few hundred frames is the
script's, a few dozen is that run's. It is also why the difficulty to tune on is the hardest:
a gap on Lunatic is a gap on every difficulty below it, and the reverse does not hold.

What the pass has proposed is the list chapters are then counted from, exactly as the
compiled-in table is, so a stage played twice divides the same way both times. A boundary is
proposed only on ground the pass has not covered yet: replaying part of a stage neither writes
one down twice nor brings back one that was removed by hand.

**Two files, and what is decided is kept.** `chapters.rs` is the table as Rust, to paste over
`crates/orb/src/game/th06/chapters.rs`; `tuning.txt` beside it is the same boundaries with what
has been decided about each — `keep`, `adjust` or `drop`, and whether a person put it there —
and is read back when orb starts. So a stage is not one sitting's work: it can be looked at,
judged in part, left, and picked up where it was. Both are written whole as soon as anything is
decided, as well as at every stage's end and on `tuning_write_key` — looking at a few boundaries
and then closing the game is how a sitting ends, and nothing about it should have to be
remembered. So edit `tuning.txt` only while nothing is running.

Entering a stage keeps everything already known about it and forgets only how far the detector
had got, which is about the pass rather than about the stage. A stage neither visited nor read
back keeps whatever is compiled in, so stages can still be done one at a time: tune a stage,
paste the table in, rebuild, and go on to the next.

`--replay` puts a replay in charge of the playing, and the two passes over one are a word each,
each of which is `--tune --replay` and one more decision:

- **`--collect`**, which nobody watches: the replay runs to the end of the run, nothing stops,
  and every boundary is proposed and written to `chapters.rs` beside the launcher as each stage
  ends. The frames still come at the display's cadence and each carries 64 updates, so twenty
  minutes of run go by in twenty seconds — with one frame in 64 drawn, which is nothing to judge
  by eye.
- **`--judge`**, which somebody does: one update a frame, and stepping between the boundaries.
  What says whether one is in the right place is the frames around it in motion, since a still
  frame shows no bullets on screen and that is the exact thing the detector already tested.
  Stepping is instant whatever the speed, so above 1 buys nothing here.

The keys, read while a run is being tracked, which under `--replay` includes a replay. They are
fixed in the code rather than settings, since whoever is building a table is the only person who
presses them. The three stepping keys are a replay's alone — holding a run someone is playing
still is what the retry menu is for:

| | |
| --- | --- |
| `right` | runs updates with nothing drawn until the chapter changes, and holds the game there — the ending skip's mechanism, aimed at a boundary instead of the end of a scene |
| `left` | restores the stage's start, which rewinds the replay along with everything else, and runs forward to the chapter before this frame. A stage is a few thousand updates and an update is tens of microseconds, so it costs a visible pause |
| `space` | holds the game where it is, or lets it carry on. Anywhere, not only on a boundary |
| `shift`, held | held with next or back to move between stages, which the game does by starting the replay at one |
| `ctrl`, held | held with next or back to move between the boundaries judged out of the table, which the stepping otherwise passes over. The way one is reached, and so the way a `Rejected` is taken back |
| `up`, `down` | judge the boundary the chapter being looked at began at, one step per press: `Rejected` → `Adjust` → `Keep` and back. Neither end wraps, so pressing on past `Keep` cannot throw a boundary away |
| `a` | puts a boundary at the frame the game is on, for a gap the detector misses, and begins its chapter there. Judged like any other afterwards, so it can be taken out again |
| `d` | writes both files, which each stage's end does anyway |

**Neither key leaves the stage, and neither does playing on.** What lies outside a stage is
the game's to load rather than a snapshot's to put back, so a step stops at the stage's own
start going back, and the replay is held at the stage's end however it got there. That end is
the game leaving the gameplay scene to build the next stage — not the boss going down, which
is well before the stage is done with itself.

Moving between stages is asked of the game instead, with `chapter_across_key` held: it starts
the replay at another of its stages, the way its own menu does. `GameManager::RegisterChain`
raises `currentStage` by one and `ReplayManager::RegisterChain` reads that stage's record —
the seed, the rank, the lives, the bombs, the power, the score so far — so writing the stage
before the one meant and `GAMEMANAGER_REINIT` into `curState` is the whole of it. The
supervisor cuts this stage's chain and registers the next, and nothing of orb's has to be kept
whole. A stage the replay does not cover is refused rather than asked for, since the game's
own answer to that is to drop to its main menu.

The run's score goes back to nothing on the way, along with the count of extra lives it has
already paid for, because those are the one thing the game leaves to whichever path it took in.
`GameManager::RegisterChain` clears them only when it is starting a run rather than
reinitialising, and a stage move reinitialises; the replay then puts the score back from the
record of the stage before the one being started, which the first stage does not have. Left
alone, stage 1 begins with the score the run was left on, crosses the first extra life's
10,000,000 part way through, and gains a life the recording never had — and with it the rank the
extra life carries, which the enemies read. The count of extra lives goes back to none rather
than to what the score says, since the stage's own loop only ever raises it: from none it can
reach what the restored score has paid for, and from a later stage's count it cannot come down.

One thing has to be held back for that to be repeatable. Tearing a stage down ends the run's
recording — `GameManager::DeletedCallback` calls `ReplayManager::StopRecording`, which closes
the record of inputs off with a blank entry and a frame number no run reaches — and the game
does it whether the run was recorded or is a replay being watched. Played back, the record it
writes into is the replay's own, at the entry playback has reached, so leaving a stage part way
terminates that stage where it was left: play it again and the player takes no input from that
frame on, standing still until it is hit. So while a replay is being played back that function
does nothing, which costs a recording nothing — the game's own replay writing calls it only for
a run it recorded.

**Every chapter boundary, not only the midstage ones.** A boss's attacks are chapters too, and a
step that skipped them would put a fight out of reach. They are in no table — they are found as
the fight runs — and the script clock the table is written in stands still during a fight, so
what the stepping moves between is the stage frame each chapter of this stage has begun at.

**A boundary the fight has is taken wherever it falls.** Dying and bombing are both seconds
long and a fight ends or moves on under them, so neither holds a boundary back: a chapter that
begins mid-death kills whoever restores it and one that begins mid-bomb hands them a cleared
screen, and both are better than a boundary that is not where the attack began. The way out of a
start like that is the chapter before it. What is held back is dialogue and a pause, where the
run is not getting anywhere at all — and held rather than dropped, so what the fight did under
one is taken on the first frame that can carry a chapter, which for the dialogue a boss arrives
with is the frame the fight starts on.

**A fight underway outranks the table.** The enemy timeline runs on through a midboss, so a fight
that drags reaches the frames the waves after it are divided at, and a chapter beginning half way
through a fight is a retry point for neither the fight nor the waves. The entry is spent where it
falls rather than held back: the fight's own end is the boundary those waves want, and an entry
held over it would fire a frame later as that boundary's double. The same frame in a run that
killed the midboss quickly is past the fight and is a boundary — that is the clock's doing, and
the table cannot say which run it is in.

Both keys are relative to the frame the game is on rather than to the chapter it is in, and
strictly so: a boundary a frame or two behind is what back reaches, and forward from there moves
on rather than staying put. Neither stops at a boundary judged out of the table: it begins no
chapter, so stopping there would be stopping at nothing. It is judged out and not gone, though,
so `chapter_dropped_key` with either key goes between exactly those — in the frames of the script
clock, since nothing has a stage frame for a boundary no chapter ever began at — and that is how
a `Rejected` is reached and taken back.

**Why the chapter changed is on the status line, not what the chapter is.** A pass is deciding
whether a boundary belongs in the table, and what answers that is which signal produced it —
which is also what says where to look when one turns up where none belongs. So the line under
`CH 05` is one of:

| | |
| --- | --- |
| `STAGE`, `STAGE_AFTER_MIDBOSS`, `MIDBOSS_NONSPELL`, `MIDBOSS_SPELL`, `BOSS_NONSPELL`, `BOSS_SPELL` | the game's own, settled as the run goes. In no table, and nothing to judge |
| `AUTO 1886 KEEP` | the table's, as the detector proposed it, with what has been decided about it |
| `HAND 1886 KEEP` | the table's, put there by hand — the number nothing would find again |

Told apart that finely because which fight it is and whether the attack has a name is what says
where in a stage the game is standing. A boss arriving is its first attack starting, so there is
nothing else to call that; and what follows a midboss is the stage carrying on, so it is named
for that rather than for the defeat behind it. `HOLD` says only that the game's update is being
held, which is a different question and is on its own line.

A run shows the chapter's name in place of both lines — see *The status line* — because there
nothing is being judged and where dying will send the player is the whole of what the line is
for.

**Which fight it is comes from the music**, because that is where the game says it: a stage's data
names two songs, its own and its boss's, and the second is played for the fight the stage ends
with and for nothing else. The answer is watched for as long as the fight lasts and only ever
raised — it starts at the fight a stage runs on through, and the first frame the stage's own track
is gone it becomes the other for good. Both ends need that: the track changes when the fight
begins, which is not always the frame the boss arrives on, and by the frame the boss is beaten the
sound may be being taken down again. The defeat is where the answer matters most, a midboss going
down being a boundary and the stage's own boss going down being the stage ending.

**The judging keys work only with the game held on a boundary.** A frame is a sixtieth of a
second, so a key pressed while the game runs lands on whichever frame it reached, which is no way
to aim at one; and a chapter judged from anywhere inside it is a boundary edited from where it
cannot be seen. So what a key acts on is always the frame on screen, and the status line is
always naming it.

A boundary is `Keep` from the moment it is proposed. `Adjust` keeps it and writes it out marked
— `1886 /* adjust */` — for a gap that is real where the frame is not quite right. `Rejected`
keeps it out of the table while remembering it, because a decision outranks the detector: the
same stage played again would otherwise propose it back, and taking a rejection back means
finding it again.

One put there by hand is written out marked as well — `/* by hand */` — since that is the
number nothing would propose again if it were lost, and refusing one takes it out altogether
rather than remembering it as refused. There is nothing for a refusal to hold back there, and
`a` is the way back.

**The shortest a chapter may be does not apply to one put there by hand.** That floor is there to
stop a boss's opening flurry of script transitions carving out chapters a fraction of a second
long, and a hand is not that: stage 5's 2363 was added 54 frames after the boundary at 2309, and
being dropped on every pass but the one it was added on would lose what somebody wrote down while
leaving it in the table to look at.

**A boundary and the start of a chapter are one thing**, and adding one by hand begins its
chapter on its own frame. Crossing a boundary is otherwise noticed on the update where the script
clock has reached it, and for a frame the game is already standing on that is the update after —
so the chapter would begin one frame past the number written down, while the same table read on a
later pass began it on the number itself. That frame is what the status line shows, what the
flash goes off on and what a step lands on: three places for one thing to be, and the one on
screen the wrong one.

Beginning it there also keeps the detector from offering one of its own a frame or two later.
Both are reading the same lull — a hand goes down when the screen empties, the detector a second
into it — so a boundary added just before the detector would have spoken leaves two a frame
apart, and the shortest a chapter may be is the rule that already says no to that.

The frame a boundary falls on is one frame — a sixtieth of a second under `--judge`, and
one of the ones that go undrawn above it — so a step holds the game on it. What is held back is
the game's update; the drawing carries on, so that frame stays on screen and the status line's
`SCRIPT` is the number that would be written down.

**A judging pass washes brighter and longer than a run does**, and whichever way
`boundary_flash` is left: the setting is about a run somebody is playing, and a pass with no
wash has nothing to say a boundary has been reached by. A boundary is one frame among a stage's
thousands, and nothing is being dodged on the frame underneath, so it can take the field for the
sixth of a second it has — one frame in twenty is all that frame gets — and still be gone before
the next boundary. It marks one the game runs past under `space` as well as one a step stops on.
The wash itself is described under *Chapters and retries*.

Every gap long enough to have been a candidate goes to the log at `verbose`, taken or not, with
its length and how far into the chapter it fell. What `ENEMY_GAP_FRAMES` should be is a question
about this game's waves, and that line is how a pass answers it.

## Checking the snapshot engine

`--self-check` restores every snapshot immediately after taking it and compares the result, and
reports memory that changed since a snapshot without being covered by it. The one moment a
restore can be held against what it should have produced is the instant it was taken from, which
is why it happens there. It fingerprints every private page in the process, so it pauses the game
for as long as that takes.

Nothing saves and restores by hand any more. What the two keys that did are for is what chapter
retries, `left` between boundaries and `--stress` do already, and with the game's own bookkeeping
around them rather than without it.

`--stress=N` restores the current chapter every N frames, a few times per chapter
and then moving on, so a replay walks through the midstage, the midboss and every boss attack
restoring as it goes.

## How it fits together

`orb.dll` is injected while the game is still suspended, so its `DllMain` runs before the
game's entry point and the memory hooks see the first allocation.

| | |
| --- | --- |
| `crates/launcher` | checks the exe, starts it suspended, injects `orb`, resumes it |
| `crates/orb-config` | `orb.yaml` and the command line, shared by both halves |
| `orb/lib.rs` | `DllMain`, the hooks orb installs, and the frame it runs in place of the game's |
| `orb/hook.rs` | trampoline and import-table hooks |
| `orb/memtrack.rs` | import hooks recording the heaps and reservations the game takes from the OS |
| `orb/snapshot.rs` | save and restore of `.data`, those regions, and the music |
| `orb/threads.rs` | finding and suspending the game's other threads, leaving the audio one alone |
| `orb/audio.rs` | the sound buffer and file position, which live outside the game's memory |
| `orb/frame.rs` | the frame loop's pacing and its measurements |
| `orb/chapter.rs` | where chapters begin, and which snapshots are kept |
| `orb/retry_ui.rs` | the menu shown where the chapter was lost |
| `orb/score.rs` | the fork of the game's score file, and the refusing of a clear run's write |
| `orb/mem.rs` | the reads and writes of the game's memory, and what makes an address safe to read |
| `orb/tuning.rs` | building the midstage table |
| `orb/window.rs` | the borderless window, the letterbox, the status line |
| `orb/input.rs` | orb's own reading of the keyboard, for its keys rather than the game's |
| `orb/overlay.rs`, `text.rs`, `d3d8.rs` | drawing over the game's frame |
| `orb/log.rs`, `profile.rs` | the log and its levels, and where a frame's time went |
| `orb/crash.rs` | the handler that names the module and offset a fault happened at |
| `orb/game/mod.rs` | `Game` and `State`: everything above is written against these |
| `orb/game/th06/` | the addresses and offsets that make it 東方紅魔郷 |

Only `th06` implements `Game`. Porting to another Touhou game means supplying its addresses
and offsets.

## Not supported

- Chapter progress across launches.
- Returning the music to where it was across a track change: the track starts again from its
  beginning instead. See *Chapters and retries*.
- Replay files are not written at all, because a rewound run does not play back. A
  chapter-mode run's score is kept, but in orb's own file — see *The score file*.
- Sound effects cut off on a restore rather than rewinding. Only the music is restored.
