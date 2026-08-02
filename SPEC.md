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

## Pointdevice and normal

**Which of the two a run is, is asked where the run is started**, over the game's own title
menu, because that is where it belongs: a run with chapters and a run without are two different
things to start, and 紺珠伝 asks it in the same place. Not a key in `orb.yaml`, which is for
what somebody sets once.

| | |
| --- | --- |
| 完全無欠モード | chapters, snapshots, the retry menu, the wash a chapter gets, the retry count on the status line, and `pointdevice_score.dat` |
| レガシーモード | the game as it was: dying costs a life, a replay can be saved, and the score goes in the game's own `score.dat` |

On screen they are 紺珠伝's own two names, since that is where the mode comes from and those are
the names somebody who wants it knows. In the code, the log and the file it writes they are
pointdevice and normal — the English of the first, and what the second actually is.

**Neither is an answer too.** `x` — the game's own bomb key, which its menus read as back — escape,
or the pad's cancel, and the front end goes back to the title the way its own back button does:
`gameState` to `STATE_CHARACTER_LOAD` and its timer to zero, which 36 frames later reaches
`STATE_STARTUP` and falls through to the menu. Copied from the difficulty select's `RETURNMENU`
branch rather than undoing what the chosen item did, because the sprites are already running the
fade that branch would set and the cursor is already on the item that was chosen.

**A menu of orb's has to read the pad itself.** Freezing the game stops its input read, so on those
frames a pad drives nothing — which looks like the pad being broken, since it worked on the game's
own menu one keypress earlier. So both of orb's menus take the pad from the sample orb's own thread
keeps for the game, and hand it to the game to be read as *its* buttons, out of
`g_Supervisor.cfg.controllerMapping` — the same copy `Controller::GetControllerInput` reads.

**Shoot and menu decide; bomb cancels.** Which is not what the game's own menus do:
`TH_BUTTON_SELECTMENU` is `TH_BUTTON_ENTER | TH_BUTTON_SHOOT` and `TH_BUTTON_RETURNMENU` is
`TH_BUTTON_MENU | TH_BUTTON_BOMB`, so there the menu button is a back. Following that put cancel on
the menu button, which on the pad this was run with is button 0 — where a thumb rests. The most
obvious button on the pad closed the question instead of answering it, and three launches went by
before the mapping said why. orb's own menus have no pause for that button to open, so it decides
instead: the button most easily reached should not be the destructive one. The launcher prints the
mapping it read for the same reason, that being the only place it is written down in a form anybody
can look at.

Up and
down come from that mapping too, and from the Y axis read the way that function reads it — the centre
halfway between `g_JoyCaps`' bounds and a dead zone of a quarter of the travel either side, its low
side being up since the axis is measured downwards — and from the hat, which is where a d-pad reports
and which the game itself does not read at all. All of it on the press
rather than the holding, and the previous frame's state is kept through the grace frames as well —
a button held from before the menu opened must not become a press the moment the grace ends.

The question goes over three of the title menu's items — a full run, the Extra stage, practice —
because each of them starts a run, and over `Score`, because the two modes have a ranking each.
One answer for both: orb is in one mode at a time, and the ranking of pointdevice runs *is* the
file pointdevice runs are written to. So looking at the other ranking puts orb in the other mode,
and the next run asks again.

**How the moment is caught.** `g_MainMenu.gameState` at 0x6dc8b0 — 0x81f0 into the 0x10f34-byte
`MainMenu` at 0x6d46c0, past its `AnmVm vm[122]` of 0x110 each — is read once a frame while the
supervisor says the front end is what is running. All three of the items that start a run set it to
`STATE_DIFFICULTY_LOAD`, and `Score` sets it to `STATE_SCORE`; each then waits 60 frames before the
game acts on it, so there is a second to spare. It is the *change* that is acted on, and coming
back from the difficulty select goes through `STATE_CHARACTER_LOAD` rather than through either, so
a choice is caught once. `MainMenu::RegisterChain` memsets the whole struct and writes the state
afresh, so nothing there is left over from the last time round.

**Both of the supervisor's states have to say front end**, not just `curState`.
`Supervisor::OnUpdate` is chain priority 0 and assigns `wantedState = curState` as its last act, so
a screen that leaves by setting `curState` itself — which is how the ranking leaves — produces one
frame ending with `curState` already saying front end, `wantedState` still saying where the game
was, and the front end not yet rebuilt. `gameState` on that frame is whatever the screen being left
was entered from, and for the ranking that is `STATE_SCORE`: leaving the ranking read as choosing
it, and the question was asked again for nothing. Seen in the log as
`menu: Scores chosen, asking which mode` on the same millisecond as the score file being written on
the way out. Requiring `wantedState` too excludes exactly that frame, and the frame after it the
memset has taken the stale state away.

Nothing has to be undone by the answer, which is why the question can be asked after the game has
acted on the keypress rather than instead of it: what the game has done by then is start a fade
and set a state, and both of them are wanted whichever mode is chosen. The game is frozen the
frame after — `RunCalcChain` returning `CHAIN_BREAK` while the drawing carries on, the mechanism
the retry menu uses — so one update of the front end runs after the keypress and none of the
second that follows it. The play field's viewport is *not* set for these frames, unlike a retry:
what is underneath is a menu, and `prepare_frame` has already given the frame the whole output.

**The key that answers it does not also reach the game.** `g_CurFrameInput` is left as every
button, so that the `held & ~held-last-frame` every one of the game's own `WAS_PRESSED` is finds
nothing on the frame the game carries on into: `Supervisor::OnUpdate` runs first in the calc chain
— priority 0 against the main menu's 2 — and its first act is
`g_LastFrameInput = g_CurFrameInput; g_CurFrameInput = GetInput()`, so that is the one thing which
reads it. What is genuinely held still reads as held, from the fresh read; only the edge goes, and
only for that frame.

Not zero, which is what the game itself writes into those three at a scene change: zero leaves
`g_LastFrameInput` empty and turns every button still down into a fresh press, which is the
opposite of what is wanted. The game gets away with it because the screens it changes to guard
their own first frames with timers — which is also why doing nothing here would have worked, and
is exactly the kind of reason not to rely on: it is a property of the screen behind the question
rather than something orb decided. A retry needs none of this, since the snapshot restore puts the
same state back with the rest of `.data`.

**The fade goes on underneath it**, because `MainMenu::OnDraw` is what advances it —
`numFramesSinceActive` against the 60 the chosen item set `framesActive` to — and the drawing is
the half that is not frozen. So the title screen darkens to the black the item's fade was heading
for and stays there while the question is up, which is a background for it rather than a problem
with it. The vms' own scripts do stop, since those are stepped by `ExecuteScript` at the end of
`MainMenu::OnUpdate`.

**No replay is offered for a pointdevice run.** A replay is the inputs and nothing about the
rewinds between them, so one recorded here plays back as a run that dies where this one carried
on. The screen that offers to save one is `ResultScreen`, reached through the job it registers —
`ResultScreen::OnUpdate` at 0x42d98e, the same walk the ending's object is found by — and its
`resultScreenState` at +0x8 is written from `SAVE_REPLAY_QUESTION` to `EXIT` on the frame it
arrives there, before the frame timer reaches the 60 that starts the question's own animation. So
no part of it is ever drawn. `EXIT` is the game's own way out and not something invented for
this: it is the state a practice run's result screen is registered in, and its `OnUpdate` case
sets the supervisor back to the title menu and takes the job out — which runs
`DeletedCallback`, so the score file is still written on the way. The alternative was answering
the question for the player, which means writing the interrupt each of the screen's 38 sprites is
to run next and then waiting out the fade they play.

Refusing the write instead — which is what `--clear` still does, since a cleared run *does* reach
that screen — would leave somebody naming a replay file that never appears.

**Nobody is asked** where there is nobody to ask: a pass over a replay (`--collect`, `--judge`,
`--replay`), a tuning session (`--tune`) and a clear (`--clear`) take the mode they are given,
which is pointdevice. A menu frozen on a question nobody answers is a pass that never ends. Nor
is anything asked with `--no-chapters`, where the mode is normal because there is nothing for it
to be: orb is loaded with none of its own work happening. And nothing is asked without the
overlay, which is what would draw the question — a frozen game with an invisible question over it
looks broken, and what is lost by not asking is the mode orb is in already.

## The frame loop

```
prepare_frame          the game's full-output viewport, and its background clear
DwmFlush()             returns just after a blank
sleep                  until (blank + one frame) − (our drawing + the compositor's)
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
and that is where input-to-screen is measured from and to. It is also what says whether the
frame made the blank it was aimed at, since what the flush waits for is that frame being
composed: see *The compositor's drawing time*.

**Cadence.** One game frame is a whole number of refreshes of the monitor the game's window
is on, taken from `MonitorFromWindow` and `EnumDisplaySettingsW`: two at 120Hz, one at 60Hz.
The compositor's `qpcRefreshPeriod` follows one monitor of the desktop, which on a mixed-rate
desktop need not be this one, so it is used only when it agrees with that rate.

**Both of those numbers are rounded, and neither rounding may decide anything.**
`dmDisplayFrequency` is whole Hz, so the NTSC-derived rates report short — 119.88Hz says 119, and
59.94Hz says 59 — while the compositor's period in whole microseconds puts the same display at
120. Two consequences, and both were live faults until a 119.88Hz display was actually run:

- A rate within two per cent of a multiple of 60 *is* that multiple, and gets its constant count.
  Read as fractional instead, 119 sends the grid chasing an exact sixtieth against a rate 0.8%
  away from one, which it settles by putting a one-refresh frame in about once a second. The
  period is taken from the nominal multiple as well, that being the nearer of the two: a 119.88Hz
  refresh is 8341µs, which 120 puts at 8333 and 119 at 8403.
- Agreement between the two is within two per cent, not equality. At 119 against 120 an equality
  test refused the blanks altogether and paced a 119.88Hz display by the clock. What the test is
  for is the compositor timing a different monitor — 144 against 120, which ran the game at 72
  frames a second — and that is not a rounding apart.

A rate that is not a whole multiple of 60 has no one cadence to keep — 144Hz is 2.4 refreshes a
frame — so there each frame goes on whichever blank is nearest where a sixtieth-of-a-second grid
has got to: two refreshes, three, two, three, two. Measured over 600 frames at 144Hz, `gaps in
refreshes 2x360 3x240`, which is 2.4 exactly and 60.00 frames a second. Every frame is still shown
at a blank rather than at a moment on a clock that lands wherever it likes among them.

Individual frames there are unequal by a refresh — 13.9ms against 20.8 — which is what 144 over 60
comes to and not something pacing can undo.

**Both the grid and the phase it is measured against are absolute**, and that is the whole of what
makes it come out even. The aim is the blank nearest the grid, counted from a kept blank; and only
a frame that landed where it was aimed moves that phase on. Measuring the next aim from the last
*landing* instead cannot correct itself — a frame that lands a refresh late becomes the reference,
so the following aim asks for one refresh fewer and the lateness is absorbed rather than undone.
That settles: measured at 144Hz as an aim averaging 2.2 refreshes with a frame in five landing a
refresh late, which adds back to the 2.4 the display wants, so the rate reads correct while a
fifth of the frames are shown somewhere nobody asked for. Anchoring the aim instead took that from
117 frames of every 600 to none.

A rate that *does* divide is given the same count every frame instead of following that grid.
A display sold as 120Hz is often 119.88, and chasing an exact sixtieth there would spend three
refreshes on a frame every few minutes to make the difference up. Two every time is 59.94fps —
a tenth of a percent on a clock nobody can see, against a hitch anybody can.

Only a display whose refresh rate is not known at all, or one the compositor will not admit to
timing, is paced by the clock. A replay being run fast, or a run being cleared fast, keeps
the cadence like anything else: `--speed` is updates per drawn frame, so the frames come one per
turn and only carry more of the game with them.

`DWM_TIMING_INFO.cRefresh` counts compositions of the window rather than refreshes of the
display, and is not used.

**The compositor's drawing time.** The window between `Present` and the blank is not idle: it is
where the compositor composes the desktop and gets it onto the screen for that blank. So a
frame's turn holds two drawing times, the game's and the compositor's, and both have to finish
before the blank or the frame is shown at the one after.

How long the compositor wants is not something it will say, so it is measured, and what measures
it is `DwmFlush`'s own return.

The flush waits for the compositor to compose the next frame rather than for the next blank as
such, so it returns at the blank the frame just handed over *reached*. Its overshoot against the
blank that frame was aimed at is therefore a per-frame answer to whether the frame made it, and
the two cases do not overlap: every frame that made its blank came back within ±900µs of it, and
every frame that missed came back 5944µs or more late, on a refresh of 8333. Half a refresh is
the boundary and one frame decides it — nothing to average, and no waiting for a fault that
happens three times in six hundred frames.

Raised 100µs the moment a frame misses, and **never lowered**. Every microsecond of it is input
lag on every frame, so the least that works is what is wanted — but the only way to learn that a
value is too little is a frame missing its blank at it, so a value that comes down is a value
being wagered again, and a lost wager is a stutter in the middle of a run. It starts at 2500µs
and climbs from there; a display wanting less pays the difference in lag and nothing else.

Shaving it back was tried and is why it is not done. From 2000µs at 100µs a second the first
frame missed at 2000, which set a floor of 2100 that said nothing about 2100 being enough; the
walk down passed the real edge without dwelling anywhere long enough to catch a value that only
fails sometimes; and from 2100 it climbed back at a stutter a step, through the whole of stage 1.

Three kinds of miss are counted and only one of them climbs:

| | |
| --- | --- |
| overshoot beyond a whole turn | a stage load, or an update that ran long |
| the frame after one of those | still picking itself up, and not the compositor's to answer for |
| a frame whose own drawing outgrew its budget | late whatever the compositor had been given |

The middle one was measured: of three climbs over a 37,800-frame replay, two happened in the
periods where a boss appeared — the game stopping for 225ms while it loads one — and the value
they climbed from had sat through thirteen quiet periods without missing once. The last was
measured too, and worse: at 144Hz a heavy frame at startup climbed the share to its ceiling, and
since the budget was capped at the same figure the drawing had no allowance at all, so every
frame reached the compositor late and every one of those asked to climb again. 120 frames of
every 600, for the rest of the run.

**Two ceilings, and they are not the same.** A frame is handed over `compose` before the blank it
is aimed at — that is the whole of what decides where the handover lands, because the drawing
happens before it and only moves when the drawing starts. So it is the compositor's share, and
not the budget, that has to stay inside one refresh: hand over earlier than the blank before the
aimed one and the compositor takes it at that earlier blank. The budget may run to most of a game
frame, since it only decides how early the drawing starts.

Getting that wrong is invisible at 120Hz, where half a game frame is exactly one refresh. At
144Hz a refresh is 6944µs, and a share of 8333 collapsed the gaps to one refresh apiece — `gaps
in refreshes 1x418 2x179`, a hundred frames a second.

`--compose=N` pins the share, which is how it is swept: pinned small enough that frames are known
to miss, then walked up until they stop. A pinned value is also the floor the work estimate is
clamped to, or the clamp would hold the sweep at `COMPOSE_FLOOR_US` and every reading below it
would be the same reading — held under the ceiling as well, since `clamp` panics when they cross
and this runs inside the frame loop.

**What the compositor will not answer.** `cFramesLate` reads zero through runs where 57 frames
of every 600 missed their blank, so it is reported and never acted on. `qpcFrameDisplayed`,
`cFrameDisplayed`, `cFramesDropped`, `cFramesMissed` and `cRefreshesDisplayed` are all zero,
while `cFrameSubmitted` and `cFrameConfirmed` in the same read move — so the call works and that
family is not populated for the desktop composition, which is the only thing
`DwmGetCompositionTimingInfo` will report on: it takes an `HWND` and accepts only null.

orb's own present-to-present gaps cannot stand in for it either. They say when a frame was
handed over, not which blank showed it, and they only wobble at all because the pacing is
anchored to the flush — a frame loop paced purely by the clock would show a perfect 16.67ms
while frames slipped a refresh apiece.

**The work estimate.** How long the frame's work takes is measured and tracked near the worst
of the recent frames rather than their average, because aiming at the average means missing
the handover on every frame heavier than it.

`timeBeginPeriod(1)` is asked for at startup and released on detach. Without it `Sleep` is
only accurate to the system tick, some fifteen milliseconds.

**What a late frame says.** `--pacing` writes a line per frame whose gap was not the cadence,
and the line accounts for the whole gap in spans that add up to it:

```
after present   what orb did once the last frame was handed over
loop            the game's own frame loop, between orb returning and being called back
clear           prepare_frame
pace            the frame's turn worked out, settle's display query inside it
flush           DwmFlush, and how far its anchor sits after the compositor's own qpcVBlank
sleep           the rest of the turn
update sound draw present
```

The one that decides is not in that list. `DwmFlush` returns at the *next* blank, so a frame
must reach it before the blank that is its own turn — and the frame is handed over some
`COMPOSE_US` before that blank, a couple of milliseconds, which is all `after present`,
`loop`, `clear` and `pace` have between them. A frame that arrives after its blank has gone
waits out another whole refresh, and nothing it does afterwards wins that refresh back. So the
line leads with how far before or after its blank the frame reached the flush, measured there
rather than worked out from the gap afterwards: a gap of the wrong size says only that
something went wrong somewhere.

`after present` and `loop` are in the line because they used to be the only part of a frame
nothing measured, while being the part that time belongs to.

The anchor the arrival is measured against is when the last flush came back, which is taken to
be a blank and is not checked anywhere else. So the line says how far it sits after the blank
the compositor reports as its last: a flush that overshoots a real blank and a flush that
returns on time against an anchor a refresh early are otherwise the same line. That query is
made only while `--pacing` is on, and after the flush, so it is spent out of the slack rather
than out of the compositor's own.

Per period, alongside the gap buckets, the worst arrival against the blank and how many frames
were past it — the rate being what says whether a stutter is one cause or the weather. Arrivals
beyond a whole turn are counted apart and left out of the worst: the frame after a stage load
is that late through no fault of the compositor, and one of those would otherwise be the
whole of the worst.

**The log is an instrument and is weighed as one.** A `WriteFile` takes what it takes, and one
in the millisecond before a handover costs that frame a refresh, so what writing the log cost
is written down beside what it is reporting — per frame and per period, the frame's own thread
and orb's kept apart, since the appends serialise on one handle and either can hold a frame up.
For the same reason the frame loop's own lines are not written where they are worked out: they
are held and written on the far side of the flush, where what is left of the turn is slack.

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

**The joystick is read on a thread of orb's own.** `Controller::GetControllerInput` (0x41cfc0,
`__cdecl`) is a tail call inside `GetInput` that adds a joystick's buttons to the keyboard's.
Where it gets them is `joyGetPosEx(0, JOY_RETURNALL)`, winmm's, through the exe's import
table. The DirectInput branch beside it is only entered where the game's `EnumDevices` found
an attached game controller at startup — where none was, `g_Supervisor.controller` (0x6c6d2c)
stays null and every frame goes to winmm.

Where nothing answers, that call takes 8.7ms and spends nearly all of it on the CPU — see
[DONE.md](DONE.md) — which is half a 16.67ms frame, and being work rather than waiting there
is nowhere cheap in the frame to put it. Where a joystick does answer it costs under a
microsecond, so what it charges for is the looking and not the reading. orb
redirects the exe's import of it and answers the game out of the last sample a thread of its
own took: every 4ms while a joystick answers, once a second while none does, and never sooner
than the read itself took, so no device can hold a core of its own. What a sample means —
which button is shot, where an axis becomes a direction, the auto-repeat behind holding one —
is left to the game's function, all of it downstream of the call orb replaced.

Where a controller was enumerated the frame's read is that other branch's `Poll` and
`GetDeviceState`, which orb leaves alone, and the sample answers only the startup check that
asks whether a pad exists at all.

**The calibration goes with the sample.** `GetControllerInput` places the centre of each axis
at `(wXmin + wXmax) / 2`, with a dead zone of a quarter of the travel, out of the `JOYCAPSA`
at 0x69d760 — which the game fills once, in its startup check, and only where a joystick
answered there. A pad plugged in later is measured against zeros, and its centred axes read as
held over. So orb writes the answering device's caps there, with every sample it hands over
rather than once when the device appeared: the caps are in `.data`, and restoring a chapter
from before the pad arrived puts the zeros back.

There is no setting for any of this. There was one — the read cost most of a frame and turning
it off was the only way out — and now that the frame pays a copy there is nothing left for it
to be off for. `GetControllerInput` is still hooked at `verbose`, to time what it now costs.

## How much of the screen the game gets

The arguments of the game's own `CreateWindowExA` are rewritten on the way through, so the
window is the size it is going to be from the moment it exists — no frame to remove afterwards
and nothing to flash first. Its window class gets a black background brush, and that is the
letterbox.

`screen: fullscreen` is that window borderless — `WS_POPUP | WS_VISIBLE`, and nothing else,
since a caption or a frame is exactly what puts a border on one — covering the monitor.
`screen: 1280x720` is that window with a caption to move it by and a system menu to close it
with, centred on the monitor, and nothing to resize it with: the size is one of the settings, so
dragging the edge would be a second place to say it and the one that is not written down. The
size is of what is *inside* the window, `AdjustWindowRect` being what turns it into the window to
ask for, so `1280x720` is 1280x720 of game however thick this machine's frames are. A window too
big for the monitor goes against its top-left corner rather than half off the top, since a caption
above the screen cannot be dragged back onto it.

**Always a window, either way.** The game's own fullscreen setting is overruled before it creates
anything: a game that has taken the display exclusively has no window to size, and orb needs one
to write its numbers in the black beside the game.

**Display scaling is off**, said with `SetProcessDPIAware` at the same moment — before the window
exists, which is the only moment it can be said. So a size in `orb.yaml` is that many pixels of
screen, and the monitor a fullscreen window is measured against is the whole of it. The size the
window came out with is logged beside the size that was asked for, and again when the device is
created, because those are two different numbers whenever anything in between has an opinion.

The back buffer stays 640x480. Windowed, the game asks for `D3DSWAPEFFECT_COPY`, the swap
effect that honours a destination rectangle on `Present`, so the back buffer goes into a
centred rectangle of the game's aspect ratio. Which is why a 16:9 window is worth offering at
all: the black it leaves down the sides is where the status line goes.

**Anything outside the game that squares windows up wins**, and there is nothing orb can do about
it: such a tool acts on the window after it exists, which is after every moment orb has. It takes
the black beside the game with it, and with that the numbers written there. Measured on a machine
running one — a `1280x720` window was created with a client of exactly 1280x720, still 1280x720
when the device was created, and 2880x2160 three and a half seconds later, which is 4:3 filling the
height of a 3840x2160 monitor. The client the window came out with is logged next to the size asked
for, and again at the device, so a run where this happens says so rather than looking like orb
getting the size wrong. vpatch's `[Window]` section is the same hazard from inside.

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

**Neither it nor `RETRY` is there in a normal run**, where there are no chapters: an empty name
and a retry count that cannot move are two lines saying the run is not the one they describe.
What is left is the lag, the compositor's share and the frame rate, which are about the machine
and are the same in both modes.

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
game's ranking — the same reason no replay is offered for one. Refusing the write would lose
the record altogether, so the file is forked: while orb is in pointdevice mode every open of
`score.dat` becomes an open of `pointdevice_score.dat`, those runs are ranked against each other
in the game's own format and on its own screen, and `score.dat` comes out of such a run unchanged
because it is never opened.

**Which file is open is a runtime switch, not a setting.** The mode is chosen inside the game —
see *Pointdevice and normal* — and the fork follows it, because a normal run and the ranking of
normal runs are the game's own file: that is where a run anybody could have played belongs. A
launch starts in pointdevice, which is what orb is for, and in normal with `--no-chapters`, where
there is nothing to fork because nothing can rewind.

The consequence worth knowing: `MainMenu::AddedCallback` opens the score file too, and parses
`clrd` and `pscr` out of it into `g_GameManager` — which is what the title menu's Extra item and
its practice stages are lit from. So the unlocks the menu shows are the ones in the file the mode
last chosen points at, and they change when the mode does. Two files means two records of what has
been cleared, and there is no third place for their union to live.

orb's file starts as a copy of the game's, so that what a `score.dat` has already unlocked —
practice on a stage that has been reached, the Extra stage — is not locked again by playing
through orb. Only where there is nothing there yet, which `CopyFileA` with `bFailIfExists`
decides for itself; after that the two are separate records and the game's is only read again by
a normal run.

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
`pointdevice_score.dat` is not itself taken for the game's file and forked again — a second pass
over it would open `pointdevice_pointdevice_score.dat` — and a relative name resolves where the
game's own open would have resolved it. The paths are handled as the bytes the game gave: a
directory name in the game's code page is not necessarily UTF-8, and the copy goes through
`CopyFileA` rather than `std::fs` for that reason. They are converted to text in one place, the
log.

With `--no-chapters` the hook is not installed at all: nothing can rewind that run, and its score
belongs in the game's own file.

**With `--clear` nothing is written**, whichever mode the run is in — there the hook is in the
path for the refusing rather than for the forking, and the file that must not be written is
whichever this run would have written. A cheated clear is not a score: orb's file is where
runs that cannot be compared with the game's are kept, and a clear nobody could have played at
the top of *that* ranking is the same mistake one file further on. The refusal is decided before
the fork is, so it covers both files. The open for writing is
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

`orb.exe` is the only file. It carries `orb.dll` inside itself — cargo's artifact
dependencies (`-Z bindeps`, which is why the toolchain is nightly) build the cdylib first and
hand over its path — and unpacks it to `%TEMP%\orb` before injecting, since `LoadLibrary`
needs a path.

The unpacked file is named for its own checksum rather than reused, because a mapped image
cannot be replaced while it is loaded. Stale copies are removed by the next launch that finds
them unloaded.

`orb.yaml` and `orb.log` are therefore found in the directory of the exe this is running
inside — the game's, which is where the launcher is installed too — and not relative to the
DLL.

Where the game and the DLL are is the launcher's own question, and `--game-dir=PATH` and
`--orb-dll=PATH` are how it is answered when they are not where they normally are. The DLL is
inside the game, so its answer is where it is; neither is handed on to it, and neither is a key
in `orb.yaml`, which therefore holds no paths and nothing belonging to one machine.

One DLL covers every game it is taught, and is not split per game the way vpatch's is. vpatch
patches per-game code; orb's per-game part is a `Game` implementation, which is a table of
addresses and a handful of accessors. Splitting would copy the snapshot, chapter, retry and
frame-loop code into every DLL and make the launcher carry several payloads.

## Configuration

**Three places, split by who sets them and when.** `orb.yaml` holds what somebody playing sets
and leaves set; a launch's arguments hold what is different every time it is run; and the mode a
run is in is asked inside the game, where the run is started — see *Pointdevice and normal*.

`orb.yaml` is five keys: `screen`, `skip_ending`, `always_draw`, `boundary_flash` and
`ask_at_startup`. YAML read with serde; four switches written `true` or `false`, and `screen`
written `fullscreen` or a size like `1280x720`. `deny_unknown_fields`, so a key nobody reads is
an error naming it — including one that used to be a key, which is a file to edit rather than one
to pass over quietly. A setting that is not read is a setting somebody thinks is on.

**The launcher asks for all five before it starts the game**, and writes back what it is told.
Which is why they are the five they are: each is about the machine the game is being played on,
and somebody who has just installed one file has nothing to edit. It is orb's own window rather
than a dialog resource — a resource means a `.rc` and a resource compiler in the build, for six
controls — with the system's own message font asked for rather than a face named, since a face
named here is one that is missing on somebody's machine and the text is Japanese.

**A real dialog, class `#32770`**, from a `DLGTEMPLATE` built in memory: a resource would mean a
`.rc` and a resource compiler in the build, and this is six controls. Being one rather than looking
like one is the point — a window manager decides whether to leave a window alone by asking what it
is, and the dialog class is the answer it looks for. A window of orb's own class with a dialog's
styles was tiled by the one on the machine this was run on, which is how the difference showed up.
Its measurements are therefore in dialog units, a quarter of the font's average character width
across and an eighth of its height down, so the whole of it scales with the font the system gives
it and nothing in it is in pixels — which is also why display scaling costs the dialog nothing. The
dialog manager brings the rest: tab between the controls, return on `はじめる`, escape on
`やめる`. The window sizes
offered are 16:9 and then 4:3 at the heights monitors have, biggest of each first and the game's
own 640x480 last, filtered to those that fit the primary monitor with a window frame's worth of
room to spare. 16:9 above because that is the ratio that leaves black for the status line, and
biggest first because on a large monitor the size wanted is the large one. The frame is allowed
for by a fixed 16x40 rather than measured, because the window being filtered for is the game's: it
does not exist yet, and it is created by the other half of orb inside a process that has not
started. A monitor is therefore never offered a window as tall as itself — that is what
fullscreen is for.

**The dialog answers to a pad**, which no dialog does by itself and which matters because the person
it is put in front of is about to play a game with one in their hands. `joyGetPosEx` on a thread of
its own — the same call the game makes, and slow for the same reason: 15ms with a pad awake and 33ms
with none, so a message loop cannot be made to wait for it — read again as soon as each read
finishes, because a press is only ever seen if a read lands while the button is down. A cycle that
waited 120ms between reads was 155ms long and lost quick taps between two of them, which is what a
pad that answers *sometimes* looks like; the launcher now also prints whether a pad answered at all
and how many pushes it sent, since a pad that was never there and one that was pushed and ignored
want opposite things done about them. The thread posts what it sees to the dialog,
which turns it into what the dialog manager and the controls already answer to.

**What the pad does is a menu's, not a dialog's.** Up and down move a row at a time, and the two
buttons are one row — left and right choose between them, the way a menu with two answers on one
line works. Left and right otherwise change what the row holds more than one of, which is the list
of sizes; a switch is on or off, so turning one over is the decide button's job rather than a
sideways push, which does nothing there. The decide button does whatever the row it is on needs
doing to it: a list is opened, a switch is turned over, a button is pressed. While the list is open
it has the whole pad — up and down move inside it and either button closes it, a dropped list's
selection already being whatever it is showing. The row is worked out from whatever has the focus
rather than remembered, so the pad picks up wherever a mouse or the tab key left off, and the focus
is moved through the dialog with `WM_NEXTDLGCTL` so that its own idea of where it is goes along.

A stick and a d-pad both drive it:
a hat reports in `dwPOV` — hundredths of a degree clockwise from up, and 0xffff for pushed
nowhere — and not on the axes at all, so reading only the axes leaves a d-pad dead. Up is the *low*
side of the Y axis, that axis being measured downwards; getting it the other way round is a dialog
that moves the wrong way, which is what it did once. Nothing about the controls is aware
that a pad exists. Which button is which comes out of the first 18 bytes of the game's own
configuration file — its `ControllerMapping`, nine `i16` — so the dialog answers to the buttons the
game will, and falls back to the game's own defaults where there is no file to read.

**Display scaling is ignored on both sides.** `SetProcessDPIAware` before either process puts
anything on the screen: the launcher before it measures the monitor, and orb inside the game
before the window is created. Without it every size is a size Windows quietly multiplies — a
1280x720 window asked for on a monitor at 150% covers 1920x1080 of screen, and the monitor a
fullscreen window is measured against reads as two thirds of itself. What it costs is that the
settings window's own layout is then in real pixels and has to be scaled by hand, from
`GetDeviceCaps(LOGPIXELSY)` against the 96 the numbers are written in; the font does not, since
what the system hands back for its own windows is already the size it wants at that dpi.

`ask_at_startup` is the last of the questions. Answer no and the next launch starts the game
straight away; `--settings` asks whichever way it is set, which is the way back. Closing the
window rather than answering it starts no game and writes nothing.

**No file is every default**, which is what leaves the launcher the one file to install — and
what makes the default `ask_at_startup: true`, so a first launch asks and there is a file
afterwards. A file that `--config=PATH` names and does not find is an error even so, since a path
somebody typed is one they meant, and answering it with the defaults would leave them watching for
a setting nothing read.

The file is written out as text with a comment over each key rather than through a serialiser,
which would leave five bare keys and nothing beside them to say what any of it is for. There is
no copy of it in this repository to install: it is written by the thing that asks, and a second
hand-kept copy is one that goes stale.

Everything to do with building the midstage table, reaching an ending, or looking into a fault
is an argument to `orb.exe` instead — `--help` lists them — because a file is the wrong
place for something that is different every time it is run. The two passes over a replay are one
word each:

| | |
| --- | --- |
| `--collect` | propose boundaries over the whole replay, at 64 updates a frame, with nothing stopping and nobody at the keyboard |
| `--judge` | step between them at one update a frame and decide about each: the pass somebody watches |

and beside them `--tune`, `--replay`, `--speed=N`, `--log=quiet|normal|verbose`, `--pacing`,
`--compose=N`, `--self-check`, `--stress=N`, and `--no-chapters`, `--no-memory`,
`--no-frame-loop` and `--no-hooks` for taking orb apart until a fault stops happening.

`--game-dir=PATH`, `--orb-dll=PATH`, `--config=PATH` and `--settings` are the launcher's own
four, and the only ones it does not hand on: each answers a question the DLL, being inside a game
that has already started, never has to ask. Keeping them here is also what keeps a path with a
space in it off the line the DLL reads.

**`--no-frame-loop`** leaves the frame to the game: its own order, draw before update, and its
own pacing, with the update and the draw still hooked so chapters carry on. The frame of input
lag comes back with it and frames are doubled and dropped again, which is why it is asked for at
a launch and not somewhere it could be left set.

One clap definition for all of them, read by both halves: the launcher out of its own
arguments, the DLL out of the command line the launcher wrote. A value goes onto its option,
`--speed=64` rather than `--speed 64`, and clap is told to require the `=` — the DLL picks the
options out of that whole command line by the two dashes they begin with, since everything
before them is a path that may hold anything, so a value standing in a word of its own is a
line the two halves would read differently.

**`--pacing`** writes what every frame that missed the cadence spent its turn on, at whatever
`--log` says rather than as a tier of it — see *What a late frame says*. Its own switch because
what the log writes is one of the reasons a frame misses its blank, so it goes with
`--log=quiet`: nothing in the file but the startup lines and this, and every write in the run is
one it made. It also turns on the two questions that cost a call each to answer and are asked
once a frame: whether the anchor is a blank, and which blank the last frame reached.

**`--compose=N`** pins the time left for the compositor to draw in, which is otherwise found
while running — see *The compositor's drawing time*. For sweeping it, `N` being what the sweep
steps in.

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
has no score to keep in either ranking, and no replay either. `--clear` is the one thing that
still refuses the write of one, rather than not being offered it: a cleared run does reach the
screen that offers to save a replay, since that screen is only kept from a pointdevice run and
this run is not one. A replay holds the inputs and nothing about the player having been
unhittable, so playing one back is a run that dies where this one did not.

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
| `launcher/settings.rs` | the dialog that asks for the five settings before the game starts |
| `launcher/pad.rs` | reading a pad on the launcher's side, so that dialog answers to one |
| `crates/orb-config` | `orb.yaml` — read by both halves, written by the launcher — and the command line |
| `orb/lib.rs` | `DllMain`, the hooks orb installs, and the frame it runs in place of the game's |
| `orb/hook.rs` | trampoline and import-table hooks |
| `orb/memtrack.rs` | import hooks recording the heaps and reservations the game takes from the OS |
| `orb/snapshot.rs` | save and restore of `.data`, those regions, and the music |
| `orb/threads.rs` | finding and suspending the game's other threads, leaving the audio one alone |
| `orb/audio.rs` | the sound buffer and file position, which live outside the game's memory |
| `orb/frame.rs` | the frame loop's pacing and its measurements |
| `orb/chapter.rs` | where chapters begin, and which snapshots are kept |
| `orb/retry_ui.rs` | the menu shown where the chapter was lost |
| `orb/mode_ui.rs` | the question put over the game's own menu: pointdevice or normal |
| `orb/score.rs` | the fork of the game's score file, and the refusing of a clear run's write |
| `orb/mem.rs` | the reads and writes of the game's memory, and what makes an address safe to read |
| `orb/tuning.rs` | building the midstage table |
| `orb/window.rs` | the window and how big it is, the letterbox, the status line |
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
- A replay of a pointdevice run, because a rewound run does not play back: the screen that offers
  to save one is skipped rather than the write being refused. Its score is kept, in orb's own
  file — see *The score file*. A normal run saves replays the way the game always did.
- Sound effects cut off on a restore rather than rewinding. Only the music is restored.
