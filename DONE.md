# Done

Everything here has been run on the real game, not only compiled. Where a claim was
settled by measurement rather than by looking at the screen, the measurement is named.

## Working

- **Injection.** The launcher checks the exe's md5, starts it suspended, loads `orb.dll`
  through a remote `LoadLibraryW`, and resumes it. `DllMain` runs before the game's entry
  point, so the memory hooks see its first allocation.
- **State accessors.** Stage, difficulty, script frames, deaths, lives, bombs, power, enemy
  and bullet counts, boss presence, boss attack and spellcard — checked against the screen.
  `GameManager.currentStage` counts from one while a stage is running, not from zero: stage 4
  practice reads 4 from its first frame, and a 1→6 replay reads 1 through 6. `read_state`
  subtracts one, so everything above it counts stages from zero as it always assumed.
- **Snapshot and restore.** `.data`, the game CRT's heaps and reservations, and the music.
  Restore a chapter mid-stage, play on, restore it again: the run continues without breaking.
  Direct3D's own objects are not in it and are left alone by a restore, which is what the
  crash below was about.
- **Music across a restore.** Rewound for the midstage and the midboss, which share the
  stage theme, and left playing through a boss-fight restore. Verified by ear over a full
  1→6 run.
- **Chapters and the retry menu.** Boundaries at the stage start, the boss appearing and
  each boss attack. A deliberate miss opens the menu; retrying the chapter returns power,
  bombs, position and music to that point. Retrying the stage works too.
- **The midstage chapter table, all seven stages.** Seventeen boundaries in
  `game/th06/chapters.rs`, built by watching a Lunatic replay of a 1→6 run and an Extra replay,
  judging a stage at a time: forty-seven gaps were proposed or placed, seventeen kept — ten of
  those put there by hand where the detector had missed one — and the other thirty judged out and
  remembered in `tuning.txt`, so that replaying a stage cannot bring them back. Every one of
  Extra's is by hand: what the detector offered there was refused, its waves leaving gaps a
  second and a half long where a chapter wants somewhere to stand.

  What the pass leant on, each of which had to be put right first: a replay that plays out the
  same way twice, stepping between boundaries and holding on one to look at it, moving between
  stages, and a boundary judged only with the game standing on it. And a pass is a word to the
  launcher rather than a file to edit — `--collect` for the one nobody watches, `--judge` for the
  one somebody does — which is what made judging a stage at a time cheap enough to do.
- **A restore that crosses a graphics load.** Stepping back into a stage's midstage from its
  boss fight used to fault inside `AnmManager::ReleaseTexture` — the game releasing a texture
  the restore had brought back from before the boss's graphics were loaded, whose memory had
  since been reused: `0xc0000005` reading `0x313d616d`, which is the bytes of `ma=1`. The
  memory holding Direct3D's handles is now left as a restore finds it, the way the sound
  system's is. Checked by doing it again in stage 6, where it had crashed twice.
- **Moving between a replay's stages**, with the replay still playing out the way it was
  recorded. Settled by a line per update — the replay's input clock, the buttons it fed, the
  player's position, and how many numbers the generator has given out — written for two
  passes over stage 1 and compared: 9011 frames, the whole stage including the boss, agreeing
  to the last digit across a move out to stage 2 and back.

  Three things had to be put right for that, and none was visible as anything but the
  player being hit where the recording was not:

  - The game ends the run's recording at every stage teardown, writing a blank input and a
    frame number no run reaches into the record it holds — which during playback is the
    replay's own, at the entry playback has reached. Leaving a stage part way therefore
    terminated it there, and playing it again the player took no input from that frame on.
    In the log at 297414375ms, before this was held back: out of stage 1 around script frame
    250 and straight back, three lives gone by frame 1027.
  - Starting the replay at a stage is not the run carrying on, so the score and the count of
    extra lives it has paid for go back to nothing first. Stage 1 otherwise began with the
    score the run was left on — 7417420 at 303310937ms — crossed 10,000,000 at frame 3240 and
    took an extra life the recording never had, which raised rank from 21 to 23. The
    generator's stream shifted three frames 173 frames later, and the player, moving on
    recorded inputs, was hit.
  - A screen shake outlives the stage that started it, which is deliberate for the fade
    between two stages and wrong for a shake: it writes the play field's rectangle from two
    numbers out of the generator every frame, and `Player::AddedCallback` measures where the
    player starts from that rectangle. A bomb within the shake's 80 frames of a stage move
    therefore began the next stage with the player at `192.00,380.87` where it starts one at
    `192.00,384.00`, and four numbers a frame going out of the stream. Checked by leaving
    stage 2 while a bomb's shake was running — the log says it was taken down at
    61041250ms — and holding the stage 1 that followed against a menu-started pass of it:
    identical from its first frame to the 742nd, where the replay was stopped by hand.
- **Borderless fullscreen.** Rewriting the game's own `CreateWindowExA` arguments, so there
  is no frame to remove and nothing flashes first. Aspect ratio kept, the rest black.
- **Ending skipped.** Run out inside the frame it starts on, so no frame of it is drawn,
  and never during a demo or a replay.
- **Replay writing suppressed.** Only the write; the game's scene-change teardown, which
  goes through the same function with null arguments, still runs. Stubbing the whole
  function crashed the game later.
- **Replay-driven automation.** `--replay` with `--speed` lets a replay of a full run do the
  playing, for building the table and for the stress mode.
- **Append-only log**, with `--log=quiet|normal|verbose`, and a crash line naming the
  faulting module and offset.
- **One file to install.** `orb-launcher.exe` carries `orb.dll` inside itself and unpacks it
  to `%TEMP%\orb` before injecting. Cargo's artifact dependencies build the cdylib first and
  hand over its path, so the two halves are always the same build. The unpacked file is
  named for its own checksum, because a mapped image cannot be replaced while loaded and a
  fixed name would stop the next build being tried while a game is running; stale ones go on
  the next launch that finds them unloaded.

## The frame loop

Settled by measurement.

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
