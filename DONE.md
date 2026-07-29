# Done

Everything here has been run on the real game, not only compiled. Where a claim was
settled by measurement rather than by looking at the screen, the measurement is named.

## Working

- **Injection.** The launcher checks the exe's md5, starts it suspended, loads `orb.dll`
  through a remote `LoadLibraryW`, and resumes it. `DllMain` runs before the game's entry
  point, so the memory hooks see its first allocation.
- **State accessors.** Stage, difficulty, script frames, deaths, lives, bombs, power, enemy
  and bullet counts, boss presence, boss attack and spellcard — checked against the screen.
- **Snapshot and restore.** `.data`, the game CRT's heaps and reservations, and the music.
  Save and restore by hand mid-stage, play on, restore again: the run continues without
  breaking.
- **Music across a restore.** Rewound for the midstage and the midboss, which share the
  stage theme, and left playing through a boss-fight restore. Verified by ear over a full
  1→6 run.
- **Chapters and the retry menu.** Boundaries at the stage start, the boss appearing and
  each boss attack. A deliberate miss opens the menu; retrying the chapter returns power,
  bombs, position and music to that point. Retrying the stage works too.
- **Borderless fullscreen.** Rewriting the game's own `CreateWindowExA` arguments, so there
  is no frame to remove and nothing flashes first. Aspect ratio kept, the rest black.
- **Ending skipped.** Run out inside the frame it starts on, so no frame of it is drawn,
  and never during a demo or a replay.
- **Replay writing suppressed.** Only the write; the game's scene-change teardown, which
  goes through the same function with null arguments, still runs. Stubbing the whole
  function crashed the game later.
- **Replay-driven automation.** `during_replay` with `replay_speed` lets a replay of a full
  run do the playing, for tuning and for the stress mode.
- **Append-only log**, with a `quiet`/`normal`/`verbose` level, and a crash line naming the
  faulting module and offset.
- **One file to install.** `orb-launcher.exe` carries `orb.dll` inside itself and unpacks it
  to `%TEMP%\orb` before injecting. Cargo's artifact dependencies build the cdylib first and
  hand over its path, so the two halves are always the same build. The unpacked file is
  named for its own checksum, because a mapped image cannot be replaced while loaded and a
  fixed name would stop the next build being tried while a game is running; stale ones go on
  the next launch that finds them unloaded.

## The frame loop

Settled by measurement, after several wrong turns recorded in SPEC.md.

- **Exactly 60fps, locked to the game's monitor.** `frame: 600 frames, 16666us apart, gaps
  in refreshes 2x600` — every frame of 600 exactly two refreshes apart on a 120Hz display.
- **`0 shown late`.** The compositor's own count of frames it could not show at the refresh
  they were aimed at, over the same window.
- **A frame of input lag removed** by updating before drawing, and the frame's work started
  as late as it will fit rather than at the top of its turn. `INPUT LAG` on screen is the
  measured time from the keyboard read to that frame reaching the screen, across the two
  frames those ends fall in.
- **The joystick read was the whole problem.** 9.3ms a frame of the 16.67ms budget, against
  ~1µs for the keyboard. `joystick: false` took the frame from a spread of one to four
  refreshes to an exact two.
- **`timeBeginPeriod(1)` restored.** The game asked for it inside the loop that was
  replaced; without it `Sleep` fell back to the 15.6ms system tick and the clock path
  stuttered in 33ms steps.
- **Input dropped while the window is behind, and the keyboard re-acquired on the way
  back**, so keys meant for something else do not reach the game and the game never reads
  an unacquired device.

## The status line

Drawn with GDI onto the window, in the black beside the game: `CH`, `RETRY`, `INPUT LAG`
and the frame rate, stacked in whichever letterbox bar is wider and lined up with the
game's edge. Shown whatever the game is doing, including demos and menus.

Not in the game's back buffer, for two reasons that were both measured: all of it is shown
inside the letterbox, and the game does not clear it between frames — text there
accumulated into a row of white pixels.
