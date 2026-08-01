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
- **A pointer chase that cannot fault.** Every read through a pointer out of the game's
  structures asks the memory map first, and committed is not enough on its own: an address that is
  not aligned for what is being read is undefined behaviour, which in a build with the checks on
  is a non-unwinding abort. Found in the test binary, which is where those reads have no game to
  land in — it prefers 0x400000 and its image is 10.5MB, so `g_Chain`, `g_SoundPlayer` and
  `g_Player` are addresses inside its own image and the walk reads whatever it keeps there. It
  aborted with `STATUS_STACK_BUFFER_OVERRUN`, every test in the run having passed, and the run
  that kept its output said `read_volatile requires that the pointer argument is aligned` out of
  the chase after a track's identity. One run in twenty rather than every run because the binary
  is `DYNAMICBASE`: where those addresses fall, and what is at them, moves per run. **0 of 200
  runs** with alignment, `PAGE_NOACCESS` and `PAGE_GUARD` all refused, against 3 of 60 before.
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
- **Ending skipped and its staff roll kept**, and never during a demo or a replay. A stage 6
  clear ran the ending out in **29,040 updates inside the frame it began on** and stopped where
  the ending hands its script over to the roll:
  `ending run out in 29040 update(s), where its staff roll begins, track Some(1727006158) ->
  Some(3570673472)` — the script and the track changing on the same update, which is the two
  signals agreeing on the boundary. Nothing of the ending reached the screen. The roll then played
  on its own, **7,286 drawn frames over 122.0 seconds**, 16.74ms each with `0 shown late` and the
  audio never behind, and the scene after it was 7, the result screen.
  - It squares with the earlier clear that measured the ending and the roll together at **36,932
    updates** — `ending skipped, 7200 frames run, scene 10 -> 10` five times and then
    `932 frames run, scene 10 -> 7`, 484ms of wall clock, 13µs an update, with the scene after it
    opening the score file 47ms later. 36,932 − 29,040 = 7,892, against the 7,830 frames of waits
    in `staff00.end`.
  - Two things that measurement leaves open: the roll ran 544 frames short of those 7,830, and
    the only wait in it that input can cut short is one `@w1200` whose second argument is 4, which
    nobody was watching the keyboard for; and the frame the skip runs in is only known to have
    taken 5 or more refreshes, which is where that log line's buckets stop.
- **What an ending is made of**, read out of `紅魔郷ED.DAT`: 33 entries, unpacked the way the
  game does — a table whose every number is two bits of length and then that many bytes, and
  LZSS over an 8kB window with 13-bit offsets, 4-bit lengths and the window written from 1. An
  entry runs to the next one and the archive keeps the sum of those bytes, so the table having
  been read right is checked rather than assumed. An ending is a `.end` script of one-character
  instructions, one file per part of it: `end00`, `end01`, `end10` and `end11` for Reimu and
  Marisa with each shot, `end00b` and `end10b` for a clear on Easy or with a continue, and
  `staff00.end` for the staff roll. **All six end on `@Fdata/staff00.end`**, so the roll is the
  ending's last script and nothing else marks where it begins; `@mbgm/th06_16.mid` is the one
  track an ending plays, and `staff00.end` starts `bgm/th06_17` for the roll. The waits in each
  add up to 23,340 frames for Reimu A, 26,940 for Reimu B, 32,940 for Marisa A, 34,140 for
  Marisa B, 6,540 for either bad ending, and 7,830 — a little over two minutes — for the roll.
  Those are the waits alone: neither 36,932 nor the 29,040 an ending on its own came to is any of
  them plus the roll's 7,830, so something else in an ending takes frames as well, and which
  ending either clear was is not written down.
- **A clear on demand.** `--clear` cleared stages 1 to 6 in **50.7 seconds** of wall clock — the
  log's stage starts 5.3, 6.1, 7.7, 9.2 and 9.3 seconds apart — with `deaths=0` from the first
  stage to the ending and not one `died in chapter` line, on nothing but the shot key being held.
  It is what the ending above was reached with.
  - The player's state alone was not enough and the log said so: with the frames of invulnerability
    left where the last respawn had put them, `died in chapter 1` came 235ms after
    `stage 1 chapter 1 (stage start)`, and again after each of the two retries. `Player::OnUpdate`
    runs at chain priority 7 and the bullets are checked at 11, so the state expired before the
    hit test in the update it was written for. Writing the frames left with it fixed that.
  - **No score file was written.** The result screen's read went through —
    `score: orb_score.dat opened in place of the game's own`, 47ms into scene 7 — and the write
    was refused: `score: orb_score.dat not written, this run had nothing able to hit the player`.
    The game carried on past it, scene 7 to 1 to 4, which is `WriteDataToFile` checking its open
    and its caller dropping the answer. `score.dat` came out with the md5 and timestamp it went in
    with, `004c8eda5a29a4ff985529838c21efe5` and `2026-07-29_22:44:45`, and `orb_score.dat` with
    `eca4048d984295dc91ca4f55050a779a` and `2026-08-01_21:18:05`.
- **Scores kept out of the game's file.** With `own_score_file`, every open of `score.dat`
  becomes an open of `orb_score.dat` at the exe's `CreateFileA` import. Over a session that
  cleared stage 6: `score.dat` came out with the md5 it went in with,
  `004c8eda5a29a4ff985529838c21efe5`, and the timestamp it went in with, while
  `orb_score.dat` changed to `eca4048d984295dc91ca4f55050a779a`. The log has
  `score: orb_score.dat opened in place of the game's own` at every open, including one 47ms
  into the result screen, which is the score being entered going that way. It started as a
  copy of `score.dat`, so nothing it had unlocked was locked again, and the next launch of the
  session said `not copied from the game's, GetLastError 80` — the file being there already,
  which is the copy happening once and only once.
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
- **The joystick read was the whole problem, and what it cost was winmm looking for a device
  that is not there.** `joyGetPosEx(0, JOY_RETURNALL)` took 8.7ms to answer `JOYERR_PARMS`
  (min 7.8ms, max 10.8ms over 100 calls), and 100 back to back took 917ms of wall clock
  against 906ms of CPU — 703ms kernel, 203ms user — so it is work rather than waiting and no
  part of a 16.67ms frame was a cheap part to do it in. `joystick=8562us/frame worst=13227us`
  in the log for the frames it ran on, against ~1µs for the keyboard. DirectInput enumerated
  no attached game controller, which is why the game was in that branch at all.
- **The same call costs nothing where a joystick answers**, so the setting that turned the
  joystick off was never the answer to it, and it is gone. With an Xbox One pad attached
  `joyGetPosEx` returns in under a
  microsecond, and the branch the game takes when `EnumDevices` found a controller at startup
  — DirectInput — reads in about a microsecond a frame: `GetDeviceState` 1µs average and 9µs
  worst over 100 reads, `Poll` returning `DI_NOEFFECT` in under a microsecond, and one 2.1ms
  `Acquire` on the first read. Both branches timed outside the game, by a program that asks the
  way the game asks — `joyGetPosEx` with `JOY_RETURNALL`, and `EnumDevices` for
  `DI8DEVCLASS_GAMECTRL` with `DIEDFL_ATTACHEDONLY` then `Poll`, `Acquire`, `GetDeviceState`
  behind a foreground exclusive acquire — around `QueryPerformanceCounter`, with
  `GetThreadTimes` across the hundred calls for the split between work and waiting.
- **The read is on a thread of orb's own, and the frame pays a copy.** With nothing plugged
  in: `input=1us/frame worst=4us calls=600` and `joystick=0us/frame worst=1us
  calls=600` over 600 frames with the window in front, the frame itself
  `16763us worst=23817us`. The thread's own reads were 32ms for the first one in the process
  — winmm's joystick support coming up — then 8.7ms once a second while nothing answered. A
  pad turned up mid-run and was picked up within the second: `mid=045e pid=02ff "Microsoft PC
  ジョイスティック ド", 16 buttons, 5 axes`, after which the reads cost microseconds on the
  4ms cadence, agreeing with the probe.
- **A pad connected mid-run works, where it used to leave two directions held.** Seen on
  screen both ways. The fault was never orb's: `GetControllerInput` places the centre of each
  axis at `(wXmin + wXmax) / 2` with a dead zone of a quarter of the travel, read out of the
  `JOYCAPSA` at 0x69d760 — an address that appears exactly once in the whole exe, in the
  `joyGetDevCapsA` call the startup check makes, and that check only reads it where a joystick
  answered `joyGetPosEx` first. So a pad that was not there at startup was measured against
  zeros, where a centred axis of 32767 is far over a threshold of 0.
  orb hands the calibration over with the sample, and the run that checked it started with the
  pad asleep (`there is no joystick 0, read in 33785us`), had it wake mid-run
  (`mid=045e pid=02ff ... 16 buttons, 5 axes, X 0..65535, read in 2494us`), took the
  calibration on the next frame (`the game's axis calibration was not this device's`), and was
  then driven through the menus with the pad, nothing drifting. Reads settled at 2µs on the 4ms
  cadence, the frame at `input=2us/frame worst=180us calls=600`.
- **`timeBeginPeriod(1)` restored.** The game asked for it inside the loop that was
  replaced; without it `Sleep` fell back to the 15.6ms system tick and the clock path
  stuttered in 33ms steps.
- **Input dropped while the window is behind, and the keyboard re-acquired on the way
  back**, so keys meant for something else do not reach the game and the game never reads
  an unacquired device.

## The status line

Drawn with GDI onto the window, in the black beside the game: the chapter, `RETRY`,
`INPUT LAG` and the frame rate, stacked in whichever letterbox bar is wider and lined up
with the game's edge. Shown whatever the game is doing, including demos and menus.

Not in the game's back buffer, for two reasons that were both measured: all of it is shown
inside the letterbox, and the game does not clear it between frames — text there
accumulated into a row of white pixels.
