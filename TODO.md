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
  to every one of them. `--log=verbose` logs every frame that missed the cadence with where its
  time went, so a clear under it would say.
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

## Confirm `self_check`

It should report zero saved regions failing to restore, and no untracked region changing outside
the process heap. It pauses the game for as long as fingerprinting every private page takes,
which is why running it is a deliberate session rather than something left on.

## Decide the joystick default

`joystick` defaults to `true`, so the 9.3ms-a-frame read is on unless someone turns it off.
That is the right default for anyone who uses a joystick and the wrong one here.

Worth knowing what the device actually is before changing it: `g_Supervisor.controller` is at
0x6c6d2c, and `GetDeviceInfo` would name it. If it turns out to be something that is not a
joystick at all — a sensor, a wheel, a VR device the game grabbed because it enumerated
first — then defaulting to off is defensible.

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
  lasting less than half a second can be missed on screen; the log has every frame.
