# To do

## The midstage chapter table

`crates/orb/src/game/th06/chapters.rs` is still empty — `MIDSTAGE: [&[i32]; 7]` with nothing
in it. Boss chapters work without a table, so what is missing is the boundaries between a
stage's waves.

Most of what is needed is built. `chapter_tuning: true` with `during_replay: true` and
`replay_speed: 8` lets a replay of a full run do the playing: boundaries are proposed at the
quiet moments, the tuning keys correct them, and `tuning_write_key` writes the file.

What is missing is a way to look at a boundary. Judging one needs the frame it falls on, and at
eight updates to a drawn frame nothing drawn is within eight updates of it.

Moving between chapters during replay playback gives that: stop at a boundary and watch it,
step to the next, step back. Both directions are made of pieces that exist.

- **Forward** is the ending skip's mechanism: run updates without drawing until the chapter
  number goes up.
- **Back** is a restore followed by the same thing. A restore rewinds the replay with
  everything else, so restoring the stage's start and running forward to the boundary before
  the current one lands exactly there. A stage is a few thousand updates and an update is tens
  of microseconds, so it costs a visible pause and nothing more.

This is the original point of the project and the last substantial piece of it.

## Skip the ending but keep the staff roll

`skip_ending` currently runs out everything `isInEnding` covers. Whether the staff roll is
inside that or a scene of its own is not known, and guessing at it is how the last few
mistakes happened.

One real ending settles it. The log now writes `ending skipped, N frames run, scene 10 -> M`
when it finishes, so clearing stage 6 once — or playing a replay of a clear — says whether
the scene after the ending is the staff roll (leave it alone) or something later (the roll is
inside scene 10 and needs distinguishing another way).

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

## Keep orb's scores out of the game's

A run with chapter retries is not a run anyone played, so its score does not belong in the
game's ranking — `block_replay_save` already refuses to write replays for the same reason, but
scores go straight into `score.dat` as the game sees them.

Two ways, and the choice is about what the numbers are for:

- **Refuse the write** while chapters are on, the way replays are refused. Simple, and loses
  any record of a chapter-mode run.
- **Write orb's own file** and leave the game's alone, so chapter-mode runs are ranked against
  each other. Needs the score save and load hooked rather than just blocked, and a decision
  about what a chapter-mode score even means when a miss costs a rewind instead of a life.

Either way the game's own file must come out unchanged, which is the part to verify.

## Confirm `self_check`

Never run on a real session. It should report zero saved regions failing to restore, and no
untracked region changing outside the process heap. It pauses the game for as long as
fingerprinting every private page takes, which is why it was deferred.

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
crash handler can no longer say which module an address is in. Nearly everything that has
gone wrong in this project was found by reading `module+offset` out of the log, so that is
not a small thing to give up.

Resolving imports from the injector rather than from a stub inside the target also assumes
system DLLs share a base across processes. They usually do within a boot session. Usually.

## Smaller things

- The retry menu is still drawn in the game's back buffer with the D3D overlay, which is
  correct there (it belongs over the game), but it shares the accumulation problem if it ever
  draws where the game does not repaint.
- The status line's numbers refresh every 30 frames. Fine to read, but it means a spike
  lasting less than half a second can be missed on screen; the log has every frame.
