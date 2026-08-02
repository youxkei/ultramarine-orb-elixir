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
- **The mode question over the game's own menu.** `Game Start`, `Extra Start` and `Practice Start`
  each froze the game the frame after the item was taken and put the question up; `Score` asked
  which of the two rankings. Answered with both, six times over one session:
  `menu: Run chosen, asking which mode` then `mode: normal, was pointdevice`, and the cursor
  starting on whichever mode orb was already in — `was normal` on the ones after it.
  - **The ranking follows the mode.** With normal chosen, the ranking screen — scene 6, the game's
    `SUPERVISOR_STATE_RESULTSCREEN` — opened the game's own file, which the log shows by there
    being no `score:` line at all for it. With pointdevice chosen, the same screen opened
    `pointdevice_score.dat`, once as it was built and again as it was written on the way out, and
    the title menu then read the same file when it rebuilt itself.
  - **Neither is an answer.** `x` on the question put the front end back on its way to the title
    menu four times in a row — `mode: not chosen, the menu is on its way back` — and each of those
    was followed by another `menu: Run chosen`, which is the proof that it got there: that line
    needs the title menu rebuilt and an item taken again. The second of the four was cancelled 469ms
    after the question appeared, so the ten frames of grace are not in the way of somebody who means
    it.
  - **And on the pad, both ways**: `mode: not chosen on the pad, the menu is on its way back`
    followed by the next `menu: Run chosen`, and `mode: answered on the pad` with the mode taken. A
    menu of orb's freezes the game, so the game is not reading the pad on those frames and one does
    nothing there unless orb reads it itself — which is how it came to be missing, and why the log
    says which hand answered.
  - **Leaving the ranking used to ask again**, on the same millisecond as the score file being
    written on the way out. `Supervisor::OnUpdate` assigns `wantedState = curState` as its last act
    and runs first in the chain, so the frame the ranking sets `curState` itself ends with the front
    end reported as running, the menu not yet rebuilt, and `gameState` still holding the
    `STATE_SCORE` it was entered from. Requiring `wantedState` to say front end as well excludes
    exactly that frame; two round trips through the ranking after the fix asked nothing.
- **Dying into the retry menu, and the chapter coming back**, on a stage 1 boss on Lunatic with the
  pad in hand:

  ```
  f3600 ... frames=2162 script=2162 ... lives=2 ... boss_life=5290 attack_frames=154
  died in chapter 2
  retry: the chapter again chosen on the pad
  restore: skipping 1 range(s), 1056 bytes
  retry chapter 2 (retry 6)
  f3651 ... frames=2010 script=2010 ... lives=2 ... boss_life=6000 attack_frames=2
  ```

  The stage clock went back 152 frames to where the chapter began, the boss's life back to the 6000
  its attack starts with, its timer to 2 — and `lives=2` with `deaths=0` on both sides of it, which
  is the whole of what orb is for: the miss cost a rewind and not a life. The fight then carried on,
  the boss's life ticking down again through 5456, 4792, 4342, 3466. 687ms passed between the death
  and the choice, so the menu's 24 frames of grace are not in the way. The skipped range is the
  Direct3D texture handles, which a restore deliberately leaves as it finds them.
- **No replay is offered for a pointdevice run**, measured over a `--clear` run to the ending —
  which is what reaches a result screen without half an hour of playing well, and which is a
  pointdevice run because `--clear` fixes the mode rather than asking: `mode: pointdevice to start
  with; nobody is asked`. Stages 1 to 6 in 35 seconds, then `ending run out in 29040 update(s),
  where its staff roll begins` — the same 29,040 as the ending measured before it, from the same
  track to the same track — and then:

  ```
  f5966 scene=7                                                     the result screen
  score: pointdevice_score.dat opened in place of the game's own     read as it was built
  result: no replay is offered for a run with chapters
  score: nothing written, this run had nothing able to hit the player
  f6545 scene=1                                                     the title menu
  score: pointdevice_score.dat opened in place of the game's own     the menu reading it again
  ```

  The result screen was up for 9.5 seconds between those two scene lines, which is the high-score
  name entry and the stats screen being played through as they always were — the state is written
  after those, not instead of them. Then the title menu, with no save-replay screen in between. And
  `score: nothing written` on the same millisecond is the proof that `DeletedCallback` still ran:
  that line is its write being refused, which is what `--clear` asks for. So writing the state
  leaves the game's own teardown in the path rather than skipping it.
  - What this run could not show, `--clear` refusing every write: that the score is *written* on the
    way out when a run is an ordinary one. That is one `score: pointdevice_score.dat opened` at the
    end of a run played without it.
- **Chapters in a pointdevice run**, from the same run: `stage 1 chapter 1 (stage start) at frame 9`
  and then chapters 2 to 6 of stage 1 at script frames 2009, 3449, 4472, 5280 and 5282 — a midboss
  nonspell twice, one out of the midstage table, a boss nonspell and a boss spellcard, so all four
  kinds of boundary — each keeping 5 regions of 4,643,324 bytes, and each of stages 1 to 6 starting
  its own chapter 1.
- **The settings asked before the game starts.** The dialog came up on each launch, and what was
  chosen in it was in `orb.yaml` and read back by both halves at the next launch:
  `screen: 1280x720`, `screen: fullscreen` and `screen: 2560x1440` each went through, with the DLL
  logging the same in its config line.
  - **And a pad answers it**, both ways. `orb: no game started (answered on the pad)` with nothing
    written, which is what closing it is supposed to do; and then
    `orb: settings written to ... (answered on the pad)` followed by the game starting, which is
    `はじめる` reached and pressed on the pad. A dialog answers to no pad by itself, so those lines
    are about orb rather than about the pad — which is why they are printed at all.
  - **And it had to be read faster than a dialog needs anything else.** At 120ms between reads, with
    a read costing 15 to 33ms, a cycle was up to 155ms and a quick tap fell between two of them: the
    pad answered sometimes and not others, which from the outside is a pad that does not work. Read
    again as soon as each read finishes, one session of it came out as
    `settings written ... (answered on the pad; a pad, pushed 29 time(s))` — every push through the
    rows, the sizes and the switches landing, and `はじめる` pressed on the pad at the end of it.
  - **Cancel was on the wrong button for three launches.** The game's own menus take
    `TH_BUTTON_RETURNMENU = TH_BUTTON_MENU | TH_BUTTON_BOMB` as back, so following the game put cancel
    on the menu button — which this pad's configuration has as button 0, where a thumb rests. Every
    attempt to answer the dialog closed it instead, logged three times as
    `no game started (answered on the pad)` with no way to see why until the launcher was made to
    print the mapping: `orb: pad — decide 2 or 0, cancel 5`. The menu button decides in orb's own
    menus now, there being no pause in them for it to open.
- **Borderless fullscreen.** Rewriting the game's own `CreateWindowExA` arguments, so there
  is no frame to remove and nothing flashes first. Aspect ratio kept, the rest black. With display
  scaling ignored it is the monitor's real pixels: `screen: fullscreen — window at 0,0 sized
  3840x2160, client 3840x2160` on a monitor that read as 2560x1440 before `SetProcessDPIAware`.
- **A window of a chosen size, and display scaling ignored.** `screen: 1280x720` came out as
  `screen: 1280x720 — window at 1277,700 sized 1286x760, client 1280x720`: the client is exactly
  the size asked for, the frame this machine adds is the 6x40 between the two, and the window is
  centred on a monitor read as 3840x2160 — which is the point of `SetProcessDPIAware`, since the
  same monitor read as 2560x1440 before it and every size would have been scaled behind the game's
  back. Still `client 1280x720` when the device was created.
  - **And it stayed, once the window managers on that machine were out of the way.**
    `screen: 2560x1440 — window at 637,340 sized 2566x1480, client 2560x1440`, centred exactly —
    (3840−2566)/2 = 637 and (2160−1480)/2 = 340 — and not one further `screen:` line for the rest of
    the session, where every run before it had been resized within four seconds. The status line had
    its black too: the game is letterboxed to 1920x1440 inside that client, 320 pixels of it either
    side, and the `no black to write in` line that a 4:3 client produces never appeared.
  - **And then something outside the game resized it**, to `client 2880x2160` three and a half
    seconds later — 4:3 filling the monitor's height. Not orb: the two client sizes logged before
    it are right, and the resize lands seconds after the last moment orb has any say. Identified as
    a borderless tool running on that machine, from its own configuration, which lists
    `東方紅魔郷.exe` with an aspect ratio of 4:3 and acts on window creation. The game's own code
    was ruled out first — `GameWindow::CreateGameWindow` calls `CreateWindowEx` once and stores the
    handle, and nothing else in it moves the window — as were vpatch, whose `[Window]` section is
    disabled and which was not loaded, and a wrapper `d3d8.dll`, of which there is none in the
    game's directory. It also takes the black beside the game, so the status line has nowhere to go
    and says so: `client 2880x2160, game 2880x2160 at 0,0 — no black to write in`.
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
- **Scores kept out of the game's file.** Every open of `score.dat` becomes an open of orb's own
  at the exe's `CreateFileA` import. Over a session that cleared stage 6: `score.dat` came out
  with the md5 it went in with, `004c8eda5a29a4ff985529838c21efe5`, and the timestamp it went in
  with, while orb's changed to `eca4048d984295dc91ca4f55050a779a`. The log has
  `score: orb_score.dat opened in place of the game's own` at every open, including one 47ms
  into the result screen, which is the score being entered going that way. It started as a
  copy of `score.dat`, so nothing it had unlocked was locked again, and the next launch of the
  session said `not copied from the game's, GetLastError 80` — the file being there already,
  which is the copy happening once and only once.
  - **Measured when orb's file was `orb_score.dat` and the fork was the `own_score_file` key**,
    which is what those log lines say. It is `pointdevice_score.dat` now and the fork follows the
    mode chosen in the game instead; the seam, the comparison and the seeding are the same code.
    What the change left to check is in [TODO.md](TODO.md).
- **Replay writing suppressed.** Only the write; the game's scene-change teardown, which
  goes through the same function with null arguments, still runs. Stubbing the whole
  function crashed the game later. This is what `--clear` still does. A pointdevice run is not
  offered the screen that saves a replay at all now, which is a different mechanism and unmeasured
  — see [TODO.md](TODO.md).
- **Replay-driven automation.** `--replay` with `--speed` lets a replay of a full run do the
  playing, for building the table and for the stress mode.
- **Append-only log**, with `--log=quiet|normal|verbose`, and a crash line naming the
  faulting module and offset.
- **One file to install.** `orb.exe` carries `orb.dll` inside itself and unpacks it
  to `%TEMP%\orb` before injecting. Cargo's artifact dependencies build the cdylib first and
  hand over its path, so the two halves are always the same build. The unpacked file is
  named for its own checksum, because a mapped image cannot be replaced while loaded and a
  fixed name would stop the next build being tried while a game is running; stale ones go on
  the next launch that finds them unloaded.

## The frame loop

Settled by measurement.

- **Exactly 60fps, locked to the game's monitor, whether or not its rate divides into 60.**

  | | |
  | --- | --- |
  | 120Hz | `600 frames, 16650us apart, gaps in refreshes 2x600` — every frame of 600 exactly two refreshes apart, seven periods in a row |
  | 119.88Hz | `600 frames, 16652us apart, gaps in refreshes 2x600`, `aimed at 0x600` — 59.94fps, the display's own rate halved, with the compositor's share never once climbing off its 2500µs start |
  | 144Hz | `600 frames, 16695us apart, gaps in refreshes 2x360 3x240` — 2.4 refreshes exactly, which is 60.00 frames a second, and `refreshes past the blank aimed at 0x600` in 15 of 20 periods |

  Frames at 144Hz are unequal by a refresh — 13.9ms against 20.8 — which is 144 over 60 and not
  something pacing can undo. What pacing settles is that every frame is shown at a blank, that the
  pattern is the regular one, and that the rate is 60 over any length of time. All three hold.

  It took the compositor's drawing time being driven by something that answers to hold the 120Hz
  case. It did not hold before: three to five frames of every 600 came out three refreshes apart,
  once every two or three seconds, for as long as there had been a frame loop.

  And it took the aim being anchored to an absolute blank, not to the last landing, to hold the
  144Hz one — see *The frame loop* in [SPEC.md](SPEC.md). Before that, 117 frames of every 600
  landed a refresh after the blank they asked for while the rate still read 60, because the aim and
  the lateness had settled into making each other up.

  The compositor wanted 2500–2600µs on all three, which is what says the 144Hz trouble was never
  its: the share was raised to 5208µs chasing it and the misses did not move.
- **Neither of the two rounded rate numbers decides anything on its own**, which 119.88Hz is the
  reason for. `dmDisplayFrequency` reported 119 and the compositor's period put the same display
  at 120, and the two faults that came of it were both live: an equality test between them refused
  the blanks and paced by the clock — `frame: 119Hz monitor and the compositor will not say` — and
  `119 % 60 != 0` sent the frames to the fractional grid, where chasing an exact sixtieth against
  a rate 0.8% off would have cost a one-refresh frame about once a second. A rate within two per
  cent of a multiple of 60 is now that multiple, and agreement between the two numbers is within
  two per cent rather than exact. Both boundaries have tests: 59, 119 and 239 are multiples; 75,
  100, 143, 144 and 165 are not; 119 and 120 are one display and 144 and 120 are two.
- **How near the blank a frame may be handed over is measured, and `DwmFlush`'s own return is
  what measures it.** The flush waits for the compositor to compose the next frame rather than
  for the next blank, so it returns at the blank *the frame just handed over reached* — which
  makes the overshoot against the blank that frame was aimed at a per-frame answer to whether
  it made it. The two cases separate cleanly: over one run every frame that made its blank
  came back within ±900µs of it, and every frame that missed came back 5944µs or more late.
  It is raised 500µs the moment a frame misses, shaved 100µs per clean second, and never
  shaved back to a value a frame has already missed at, so each value costs a stutter at most
  once. Watched doing exactly that over a replay played back through stages 0 to 3, bosses and
  spellcards included, up to 524 bullets on screen — 63 periods, 37,800 frames. The steps were
  slower then, 50µs per 600 frames from a start of 2500µs, so the values below are the ones that
  walk took and not the ones a run takes now:

  | | |
  | --- | --- |
  | periods of `gaps in refreshes 2x600`, nothing at all off the cadence | 49 of 63 |
  | frames that missed their blank by a refresh, out of 37,800 | **3** |
  | what those three bought | one climb each: the floor to 2450, then 2500, then 2550µs. Only the first was the compositor's: the other two fell in the periods where a boss appeared — stage 2's and stage 3's, the game stopping for 228986µs and 225368µs while it loads one, with `frames` and `script` parting company on the same line — and 2450µs had sat through thirteen quiet periods without missing once. The frame after a load is exempt now, and counted apart so the exemption is visible |
  | the other off-cadence frames | 8, every one beyond a whole turn — stage loads, `gap at worst` 455495µs and 224081µs — which the guard keeps out of the climb |
  | where it settled | 2550µs, held over the last 3000 frames |

  So a missed blank is a one-frame event, paid for once, that cannot happen again at that value.
  That floor is also what keeps the lag down. Without it the shaving walks back into values
  already known short, pays the 500µs again and random-walks upward — 3700–3850µs over one run,
  against 2550µs with it.
- **What the compositor wants does not depend on what the game is drawing.** The same 2450–2550µs
  came out of an idle title screen and out of a stage 3 boss fight, while the game's own drawing
  time over those periods ranged 697–1687µs and `prepare` tracked it frame by frame. Which is
  what keeping the two drawing times as separate numbers predicts: one is a property of the
  display, the other of the frame.
  Overshoots beyond a whole turn are left out of it: those are stage loads and updates that
  ran long, and giving the compositor longer for one is what ratcheted an earlier attempt to 7ms of lag.
  Established by pinning it with `--compose=N` and causing the fault rather than waiting
  for it — two settled periods of 600 frames each at seven values, on an unattended launch left
  at the title screen. Which scene that was is not in the log: those runs were at `--log=quiet`,
  which writes no state lines, and `--pacing` did not write one of its own until afterwards. So
  the load the sweep ran against is what an idle launch does and no more precise than that.

  | left for the compositor | 200µs | 800µs | 1500µs | 2000µs | 2500µs | 3000µs | 5000µs |
  | --- | --- | --- | --- | --- | --- | --- | --- |
  | off the cadence, per 600 | 53, 57 | 33, 36 | 5, 7 | 2, 1 | 0, 0 | 0, 0 | 0, 0 |

  Monotone, and at 200µs the interval sags to 18348µs — 54 frames a second.
- **`cFramesLate` is not evidence and nothing is judged on it.** The compositor's own count of
  frames it could not show at the refresh they were aimed at read `0 shown late` through every
  run above, including the ones where 57 frames of 600 missed their blank. It is still
  reported, as a number whose meaning is not what its name says. `qpcFrameDisplayed`,
  `cFrameDisplayed`, `cFramesDropped`, `cFramesMissed` and `cRefreshesDisplayed` are worse: all
  zero, while `cFrameSubmitted` and `cFrameConfirmed` in the same read moved 1211 over a
  period — so the call works and that family is not populated for the desktop query, which is
  the only one `DwmGetCompositionTimingInfo` accepts.
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
