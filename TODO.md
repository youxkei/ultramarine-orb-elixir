# To do

## Judge the flash a run gets

A chapter beginning now washes the play field in a run somebody is playing, and not only in a
judging pass. Dimmer and shorter in a run: `FLASH_PLAYING` in `lib.rs`, against
`FLASH_JUDGING`'s alpha 0xc0 held five frames of sixteen.

What is known: **alpha 0x40 held two frames of ten went unnoticed while playing.** Which is the
difference between the two cases — somebody judging a boundary is looking at the frame the wash
is on, and somebody playing is watching the player. It is 0x70 held three of fourteen now, and
that has not been looked at yet.

If it goes the other way and a wash bright enough to notice is a wash in the way, the answer is
a different shape rather than another number: the field's edge rather than over it, which says a
chapter began without taking any of the field away.

## Going back more than one chapter

The retry menu offers this chapter and the stage's start, because those are the two snapshots
kept. Nothing about a boundary asks it to be a good place to restart from — a chapter can begin
on the frame the player was hit, and restoring that one kills whoever uses it — so the way out
of a bad one has to be the chapter before it, and in a run someone is playing there is no way
back to it. Stepping has one, but only a replay can step.

What it costs is known: a chapter's snapshot is five or six regions of about four megabytes, so
a whole stage's worth is forty to fifty. The shape is a stack of them per stage, dropped from
the chapter restored onwards, and a menu that lists what is there rather than two fixed choices.

## What a restore across a graphics load looks like

The crash it used to cause is gone — see [DONE.md](DONE.md) — because the memory holding
Direct3D's texture handles is left as a restore finds it. What is left is cosmetic and has not
been looked at: for the frames between such a restore and the game loading those graphics again,
the sprite tables the snapshot put back describe one set of graphics while the slots hold
another, so something may be drawn from the wrong texture. Stepping back into stage 6's midstage
from its boss fight is where to look.

Left out of that on purpose: `AnmManager`'s surfaces and its vertex buffer. The game releases
those when it loses the device, which a stage does not run into, and a range left out of a
restore is a range the snapshot no longer describes.

## Support Touhou games other than 紅魔郷

The chapter, snapshot, retry and frame-loop code is written against `Game` and `State` and
knows nothing about which game it is in. What has to be supplied per game is a `Game`
implementation: a table of addresses and a handful of accessors. One DLL carries them all and
picks by the exe it finds itself inside, since none of the work is per-game code — see
[SPEC.md](SPEC.md) for why that is not split the way vpatch's is.

The th06-specific things that live outside `game/th06/` and would each become a table:

| | |
| --- | --- |
| `launcher/main.rs` | `GAME_EXE` and `GAME_EXE_MD5`, and the error text naming 1.02h |
| `orb/lib.rs` | `static GAME: Th06`, chosen rather than fixed |
| `orb/lib.rs` | the `vpatch_th06.dll` name in the "vpatch loaded" line |
| `orb/game/th06/chapters.rs` | `MIDSTAGE` is seven stages because 紅魔郷 has seven |

Two things to settle before the second game rather than after: whether `State` says everything
a chapter needs for a game whose scoring or resources work differently, and whether the
midstage table's shape — script frame numbers per stage — holds where stages are not one
script on one clock.

## What a clear left open

The skip stopping at the staff roll and `--clear` reaching one are both measured — see
[DONE.md](DONE.md). Three numbers that clear did not settle:

- **What the skip's frame costs.** All the log will say is `gaps in refreshes 5+x1`: one frame of
  the 600 took five refreshes or more, which is where the buckets stop, and the average interval
  and `0 shown late` did not move. 29,040 updates at the 13µs an ending update cost before would
  be 377ms, and reading the script adds a walk of the job chain with a `VirtualQuery` at each step
  to every one of them. `--pacing` logs every frame that missed the cadence with where its time
  went, and the `gap at worst` in that run's reports says what the skip's frame came to, so a
  clear under it would say.
- **Why the roll ran 7,286 frames** where `staff00.end`'s waits add up to 7,830. The one wait in
  it that input can cut short is `@w1200` with a second argument of 4, and whether a key was
  pressed over those two minutes is not written down. A clear that keeps its hands off the
  keyboard through the roll would settle it.
- **The updates per ending.** 29,040 for the one that clear reached, against script waits of
  23,340 for Reimu A, 26,940 for Reimu B, 32,940 for Marisa A and 34,140 for Marisa B — none of
  which it is, and the log does not say which character it was either. Worth one line per ending
  as they get seen, since what the skip has to run out is what those come to.

## Resume a chapter after quitting

A snapshot is only good inside the process that took it: it holds pointers to Direct3D and
DirectSound objects, and those addresses differ per launch. So closing the game loses where
you were, which for the one thing orb exists to do — grinding a late chapter — is the wrong
place to lose it.

Restoring raw memory cannot be made to work across launches without recreating those objects
and fixing up every pointer to them, which is not a road worth going down. The other shape is
to write down what the chapter *is* rather than what memory looked like:

- let the game enter the stage the normal way, so it builds its own objects;
- restore the fields that describe the run — power, bombs, lives, score, the random seed, the
  chapter's script frame;
- run the stage script forward to that frame with drawing suppressed, the way the ending skip
  runs the ending out.

That is a second, weaker restore mechanism living alongside the exact one, and it has to be
weaker: anything the script did on the way that is not in the list above will not have
happened. Whether that matters depends on how much of a midstage's state is script-derived,
which is worth finding out before committing to it.

## What a chapter-mode score is

The scores of runs orb could rewind are kept apart from the game's now — see
[DONE.md](DONE.md) — but that only keeps them from being compared with runs somebody played. It
does not make them comparable with each other: a miss costs a rewind rather than a life, so the
number rewards grinding a chapter until it goes perfectly, which is a different game from the
one the ranking was built for.

Nothing here is broken, so there is nothing that has to be done. What would make the file worth
reading is the retries beside the score, since a clear with none of them and a clear with sixty
are not the same clear, and `RETRY` is already counted. That means orb's own format rather than
the game's, and with it the game's ranking screen no longer being where these are read.

## What the fixed stutter costs

Three to five frames of every six hundred used to come out three refreshes apart instead of two,
once every two or three seconds. The cause and the fix are in [DONE.md](DONE.md), and a replay
played back through stages 0 to 3 under `--log=quiet --pacing` settled what was still open about
them: 37,800 frames, three that missed their blank, 49 of 63 periods with nothing off the cadence
at all, and the compositor's drawing time converging to 2550µs and staying.

Two things that were open are answered by that run and are not questions any more. What the
compositor wants does not depend on the load — the same 2450–2550µs came out of a title screen
and out of a stage 3 boss fight with 524 bullets up — and the 3700–3850µs an earlier run showed
was the shaving random-walking upward for want of a floor, not a heavier stage needing more.

What is left:

- **The lag.** `prepare` sits at 3200–4500µs against the 2100µs it used to, so the cadence costs
  something like 1.5ms of input lag on every frame. Against the 16.7ms a frame that orb exists to
  save it is under a tenth, and it buys a cadence that does not break, but nobody has been asked
  whether that is the trade they want. `MISS_STEP_US`, `SHAVE_US` and `SHAVE_FRAMES` move it.
- **Whether exempting the frame after a load leaves one stutter a session.** It should: the
  floor lands a step above whatever missed and the shaving stops there, and with boss entry no
  longer able to raise it, the 2450µs that already held for thirteen quiet periods should hold
  for the run. That is a prediction from the run above, not something a run has shown — the same
  replay under `--log=quiet --pacing` would show it, as `the compositor gets 2450us` unchanged
  from the first climb to the end and one `1x1` in the whole log.
- **Whether the miss the ratchet trips on is always a real one.** The frame that missed at 2400µs
  did not show up as a broken gap in the same period's `gaps in refreshes`, which stayed `2x600`.
  Either the two counters are a frame out of step at a period boundary — `measure_compose` runs at
  the top of a frame and the gap is worked out at the bottom, with the report between them — or
  that overshoot sat right on the half-refresh boundary the two round opposite ways from. It errs
  toward giving the compositor longer, which is the safe direction, but it means the floor can end
  up a step above what it needs to be, and three steps of that is 150µs of lag.
- **Which refresh rates have actually been run.** Three: 120Hz, 119.88Hz and 144Hz, all on the
  same machine and the same monitor at three settings — see [DONE.md](DONE.md). The third one
  earned its place by breaking both branches it touched, neither of which had been run before, so
  the list below is not a formality. The rates that would each exercise a different part of the
  arithmetic:

  | | |
  | --- | --- |
  | 60Hz | one refresh a frame, where the compositor's share and the drawing must both fit in 16.7ms rather than 6.9 |
  | 75Hz, 100Hz | ratios of 1.25 and 1.67, so the count is one *or* two and the grid spends most frames on the shorter one |
  | 240Hz | four refreshes a frame, a whole multiple again but with a 4.2ms refresh, so the share has less than a quarter of the room it has at 60 |
  | under 60Hz | deliberately the clock, since one frame per blank would run the game slow and take the music with it |

  The mechanism does not special-case any of them, which is the argument that they work, and no
  measurement is the reason that is only an argument.
- **A second display, and a mixed-rate desktop.** What the compositor wants is cleared and found
  again when the mode or the monitor changes, so nothing carries over wrongly, but the case that
  matters is the game on one monitor while the compositor times another: the fractional path is
  refused there — `agrees` is false and the clock takes it — and that refusal has not been seen
  happen. A whole multiple is *not* refused there, which is the older hazard the code comments
  describe and which nothing has re-checked since.

## Confirm `self_check`

It should report zero saved regions failing to restore, and no untracked region changing outside
the process heap. It pauses the game for as long as fingerprinting every private page takes,
which is why running it is a deliberate session rather than something left on.

## Play a stage with a pad

The read is orb's thread's now and the frame pays a copy, and a pad that turned up mid-run drove
the menus — see [DONE.md](DONE.md). What no run has been through is a stage: shot and bomb under
a pattern, the four directions at the speed they are used at, and the auto-repeat behind holding
one, none of which a menu asks for. Worth doing once by somebody who plays with a pad.

**One loose end from the run that settled the calibration.** The handover said so once a
second rather than once, which is a write into the game's memory every time it says it. The
caps were being read beside every position then, so the likeliest reading is that
`joyGetDevCapsA` does not fill all 404 bytes the same way twice — the tail is `szOEMVxD`, 260
bytes of it. They are taken once per appearance now, which should leave one line, and the line
says the offset it differed from so that a repeat says what is putting the game's copy back. A
retried chapter is the one thing that legitimately does, since the caps are in `.data`.
Unwatched since the change; it needs a pad asleep at launch and woken afterwards, which is the
only way into the winmm branch with a device on it.

The other branch is measured but not through the game. A pad attached *before* the game starts
is one `EnumDevices` finds, and then the frame's read is DirectInput's rather than winmm's:
about a microsecond a probe measured, and orb neither replaces nor times it. Its up-to-400
`Acquire` retries on `DIERR_INPUTLOST` have never been seen to happen — a device that goes to
sleep mid-stage is how they would be, which is worth watching for on a machine whose pad is
wireless, since 400 of them land on one frame.

## Map the DLL by hand, if a temp file is itself the objection

The launcher carries `orb.dll` and unpacks it to `%TEMP%\orb`, which is one file to install
but still a file written at run time. Mapping the image into the game directly would write
nothing at all.

Measured against the built DLL, so this is what it would actually take:

| | |
| --- | --- |
| `.reloc`, 11kB | relocations have to be applied — the image is not loaded at its preferred base |
| `.tls`, 0x18 bytes | **a TLS directory exists.** The loader normally allocates a slot per thread and fixes up `_tls_index`; a manual map has to do that itself, for the game's existing threads and any created later. Rust's std uses TLS, and getting this subtly wrong fails far from the cause |
| no `.pdata` | nothing to register: 32-bit SEH is the `FS:[0]` chain the CRT sets up. Easier than it would be on x64 |
| `.CRT`, 0x30 | initialisers, run by calling the entry point |

The mechanics are a few hundred lines. The costs are that TLS hazard, and that a manually
mapped image is in no module list — so `GetModuleFileNameW` returns nothing for it and the
crash handler can no longer say which module an address is in. `module+offset` out of the log is
how a fault in orb's own code gets located at all, so that is not a small thing to give up.

Resolving imports from the injector rather than from a stub inside the target also assumes
system DLLs share a base across processes. They usually do within a boot session. Usually.

## Smaller things

- The chapter names on the status line and in the retry menu, and the bar being cleared to
  black before each draw, have not been seen on the real screen. What to look at: a stack that
  loses a line — `HOLD` going away — leaving nothing of it behind, the name reading right in
  the retry menu, `MIDSTAGE` picking its count up after a midboss rather than starting again,
  and a `--judge` pass still showing `CH 05` with why the chapter changed under it, which is
  what that pass is read for.
- The retry menu is still drawn in the game's back buffer with the D3D overlay, which is
  correct there (it belongs over the game), but it shares the accumulation problem if it ever
  draws where the game does not repaint.
- The status line's numbers refresh every 30 frames. Fine to read, but it means a spike
  lasting less than half a second can be missed on screen; the log has every frame that missed
  the cadence, up to six per report — it says how many more there were.
