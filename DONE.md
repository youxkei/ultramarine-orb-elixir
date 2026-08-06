# Done

Everything here has been run rather than only compiled, and each entry says on what: the game on
this machine, or the suite driving the real code against a simulated Windows. Where a claim was
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
- **A chapter picked up after the game was closed.** Two launches, at 200740000ms and 200755453ms
  in the log, with the front end driven by keys sent from another program — see *Driving the game
  without a keyboard* below for what that took and why it is honest:
  - A pointdevice run started, `Lunatic ReimuA`, and its first chapter written down as soon as it
    was reached: `resume: stage 1 chapter 1 (MIDSTAGE 1) at frame 9, 9 frame(s) of buttons, diff=3
    char=0-0 score=0 seed=0x7c76 lives=2 bombs=3 power=0 rank=16 written to
    pointdevice_resume\lunatic-reimu-a.txt`. Its `start` line held the same numbers as the
    `stage 1 chapter 1 (stage start)` line beside it, seed included, and the file was named for the
    run rather than for the stage.
  - `died in chapter 1`, the retry menu, and the run left. Both ways of leaving it are on record:
    this session took *the stage again* and was then killed mid-run, and the session before it took
    the third item — `retry: the run given up on the keyboard`, `retry: the run is given up; the
    game is on its way to the title`. The file was still there after the game was gone.
  - A new launch: `resume: 1 run(s) left unfinished: lunatic-reimu-a`. The same difficulty and
    character chosen again, and the question came up as the shot was chosen —
    `resume: lunatic-reimu-a was left; asking where to start` — with the file read there rather
    than at startup.
  - `つづきから` answered: `resume: from where it stopped, answered on the keyboard`, the run built
    at that stage, 8 updates run inside the frame that built it, and **`resume: the landing is the
    frame that was written down, field for field`** — stage 1 frame 9, chapter 1 (MIDSTAGE 1),
    against chapter 1 (MIDSTAGE 1) at frame 9.
  - **What that check caught before it agreed**, on the run before this one:
    `resume: the landing is not the frame written down: randoms=2 against randoms=0`, every other
    field of the line the same. Building a stage draws two numbers from the generator on stage 1,
    and the game's own replay is called from inside `GameManager::AddedCallback` *before* the
    loading, so those two draws advance its seed as they did the recorded run's. orb wrote the seed
    after the loading instead, leaving the generator two numbers behind and the run a different one
    from the landing on. The seed now goes in before the stage is built and the rest of the numbers
    after it, which is what the second run says field for field.
  - **And what it caught the third time, which is where the seed actually goes.** A Normal ReimuA run
    left in stage 4's `MIDSTAGE 3` at frame 3395 was picked up and came apart: the player was hit at
    frame 743, four lives went in 3394 frames, and the landing was `player=290.74,134.51 against
    player=186.54,253.03` with `randoms` 1955 out. The stage's own line said why —
    `resume: the generator seeded 0xc381`, and then `stage 4 chapter 1 (stage start) … seed=0x789c`.
    `GameManager::AddedCallback` draws from the generator **2048 times before it copies the seed**:
    0x41bc4f fills 64 records of 32 `u16` at `manager+0x30`, one `Rng::GetRandomU16` (0x41e780) each,
    and that generator is `seed = rotl16(((seed ^ 0x9630) - 0x6553), 2)` — every draw rewrites it. The
    block is skipped when `curState == 3`, the state between two stages of a run, which is why an
    ordinary stage 2 draws none of them and a resume, built from the menu, draws all 2048. Confirmed by
    arithmetic rather than by argument: 2048 draws from `0xc381` are `0x789c`, and from the stage-2
    file's `0x0d45` they are `0x6e26`, which is what that stage showed. The seed is now written on the
    way into `Stage::RegisterChain` (0x4044c0, called from 0x41c00d and nowhere else in the exe) —
    after the 2048, before the stage is built out of the generator, which is the same window the game's
    own replay writes in.
  - **The landing check agreed with a wrong seed, so the seed is now one of its fields.** The stage-2
    resume that reported `field for field` had `0x6e26` where `0x0d45` was written down: the player's
    place comes from the player's own inputs, 紅魔郷's bullets come from the stage's script, and the
    two runs even took the same hit at frame 2064 — so nothing in that line showed it. `rng=` is in the
    reproduction line now, and the comparison holds lines together by field name rather than by
    position, so a line written before the field existed is still checked on what it does have and
    `resume: nothing was held against rng` says what it could not.
  - **And what it caught the second time, on a chapter deep enough to show it.** A Normal ReimuA run
    left in `MIDBOSS NONSPELL 1` was picked up: 2008 updates ran, the landing was that chapter at
    frame 2009 as written down, `input=0x0091 player=94.75,219.03 items=3 rank=17` all the same — and
    `randoms=3295 against randoms=3425`, with `score=400760` landing as `359550`. The player was on
    the pixel it was written down on, so the buttons went in right; what differed was everything the
    generator fed. The stage's own line said why: `seed=0x90b9` where `0x4fea` had been written down.
    Writing the seed on the frame the question is answered is a frame too early — whatever the front
    end draws in between moves it off — so it is written on `GameManager::AddedCallback`'s way in
    instead, which is after the front end is gone, before the stage is loaded out of the generator,
    and before the callback copies the seed into `randomSeed`. **Not re-checked yet** — see
    [TODO.md](TODO.md).
  - The same session showed the song being left at the track's opening milliseconds, with the
    position that had been written down (`song=5900628`) never read: the restore was asked of the
    chapter's *kind*, and a midboss is not one of the midstage kinds although the stage's own song is
    what plays through it. It is asked of the song now, from the same place a retry asks it, so the
    two cannot disagree about a chapter.
  - **And with the song put back where the chapter had it, a section of it repeating.** Heard once
    near the end of a resumed stage 1's midstage and once near the end of a resumed stage 2's, and
    read off the game afterwards: `WaveFile::Read` (0x43c080) clamps every read to `m_ck.cksize`
    (0x43c1aa) and subtracts what it read from it (0x43c1be), and `StreamingSound::ServiceBuffer`
    takes the track's loop only where a read comes up *short against that countdown* (0x43b759 →
    `ResetFile(TRUE)` at 0x43b76f). The file's own end is never asked. Seeking the file 5,900,628
    bytes forward and leaving the countdown alone therefore left the stream believing it had that much
    more sound than the file held: it read past the end of the `data` chunk, where `Read` fails rather
    than returning short — `mmioAdvance` leaves nothing to copy (0x43c233) — so no loop was taken and
    the buffer was left going round its own contents. It came right by itself after the skipped bytes
    had been counted off, which is why it was one episode and not a hang. The countdown is now moved
    with the file, to the loop point less where the file ends up, and the loop point is taken as the
    position plus the countdown as they stand — the pair the game reads, rather than the header's loop
    fields, since a track without a loop end runs on `cksize` instead. **Not re-heard yet** — see
    [TODO.md](TODO.md).
- **A later stage's chapter written down, with a run's numbers behind it.** A Normal ReimuB run
  played by hand to stage 2 and stopped there. Every chapter of stage 1 was written as it was
  reached — frames 9, 2009, 2651, 4472, 5714, 6115, 6736, 7316, each one the frames before it, so a
  chapter deep in a boss fight is 7316 frames of buttons rather than the nine stage 1's first is —
  and then stage 2's, which is the case stage 1 cannot show:
  `resume: stage 2 chapter 2 (MIDSTAGE 2) at frame 880, 880 frame(s) of buttons, diff=1 char=0-1
  score=3760130 seed=0x7b9d lives=2 bombs=3 power=83 rank=24`. On stage 1 every one of those numbers
  is zero and the stage orb writes is the 0 the menu had already written; here the score, the power,
  the rank, the point items and the stage are all the run's own, read where the game itself reads
  them. Its `landing` line is a stage being played rather than a stage's first frame:
  `player=191.21,432.00 randoms=1481 items=16 score=3881080 subrank=34`. **Picking that one up has
  not been watched** — see [TODO.md](TODO.md).
  - **A run is offered only where all three of difficulty, character and shot match**, which the same
    session showed by accident: a launch that chose ReimuA where the run left was ReimuB said
    `resume: 1 run(s) left unfinished: normal-reimu-b` at startup and then
    `resume: nothing was left of normal-reimu-a`, asked nothing, and started a fresh run whose own
    chapters went to `normal-reimu-a.txt`. The ReimuB file was not touched, which is what the file per
    run is for. Nothing on the game's own screens remembers which shot the last run used —
    `MainMenu::RegisterChain` memsets the menu, so its cursor starts at ReimuA every time — so
    picking a run up means choosing the same three again.
- **The question asked on the press, and cancelled without the run starting.** A Normal ReimuB run left
  in stage 5's `MIDSTAGE 3` at frame 7477, chosen again from the front end on the pad. The shot type
  select's own decide is held back, so the press put the question up rather than the run:
  `resume: normal-reimu-b was left; asking where to start`, and then
  `resume: neither, answered on the pad; no run is started`. Four times in a row — the questions at
  334506843, 334508187, 334509156 and 334510156, each cancelled within 600ms and each followed by
  another press on the same screen 600 to 900ms later.
  - That sequence is the proof the screen never moved: its own back goes to the character select
    (0x436c49), from where a second question would have needed the character and the shot chosen again,
    and the presses came too close together for that. **The key that cancelled has to be held back as
    well** for it — see `Game::menu_cancel` for what was tried first and why it delivered the press it
    was meant to stop.
  - The file was still there after all four, which is the whole point of `キャンセル`: nothing was
    started, so nothing wrote over it.
  - **And then answered `つづきから` from the same screen**, ten seconds after the question went up:
    `resume: from where it stopped, answered on the pad`, the press handed back for one read, the shot
    type select choosing its own item, and the chapter going in on the frame the run was registered —
    `resume: ... the run is being built at that stage`. 7476 updates played in and
    **`resume: the landing is the frame that was written down, field for field`**, with the song picked
    up at 21968908 in its own file. So the answer remembered against its slot arrives on the one frame
    the game will take a stage for a run it has not built.
  - **The run was then played**, which is what says the shot button reaches the game on the frames
    orb is not holding it: stage 5's boss through two chapters — `stage 5 chapter 9 ... a boss
    spellcard` at frame 11610 and `chapter 10 ... a boss nonspell` at 12297 — with its life going down
    5420, 3833, 2312 and then 14954 to 7338 on the next attack. A miss in chapter 10 put the retry menu
    up, and the run was given up from it on the pad.
- **Driving the game without a keyboard**, which is what made the above possible and is worth
  writing down for every other unwatched thing in [TODO.md](TODO.md). The launcher's dialog is got
  past with `--config` naming a file that says `ask_at_startup: false`. The game's own screens
  cannot be, as it stands: keys injected with `SendInput` — tried carrying the virtual key with its
  scancode, and as the scancode alone with `KEYEVENTF_SCANCODE` — are accepted by the system
  (`SendInput` returns 1) and not seen by the game, which sat idle into its attract demo twice.
  `Controller::GetInput` takes the keyboard `DISCL_EXCLUSIVE | DISCL_FOREGROUND` and such a device
  does not see them. `--sent-keys` has orb let that device go — `Unacquire`, `Release`, the pointer
  cleared, which is what `Supervisor::RegisterChain` does with a device it cannot set up — and the
  game then reads `GetKeyboardState`, its own other way, which does see them. The first press after
  that ended the attract demo, which is what proved it. Two things learned about the timing: a
  press inside the title's own opening animation is spent on nothing, and the demo eats one to
  leave, so what works is a press every 1.1 seconds until the log says the screen moved.
- **The hook at the moment a stage's numbers are put in place**, which is what picking a run up
  rests on. One launch to the title screen, at 199297250ms in the log, left standing
  until the attract demo took itself into a stage:
  - `stage start hook installed, original at 0x01150000` — the prologue expected at
    `GameManager::AddedCallback`, 0x41bb02, is the one that is there. Read off the file first, so
    this is the check agreeing with the read rather than the only evidence.
  - `resume: stage 4 of a run orb is not keeping; nothing of it is written down`, 16 seconds in,
    which is the hook firing where the game registers a stage. Stage 4 is the demo's, and the
    demo's own path writes `currentStage = 3` before it asks for a run: the hook reads the stage
    the way the rest of orb does, *after* that callback has raised the number, and it came out as
    the same stage. The demo is refused rather than recorded, and `pointdevice_resume/` was still
    not there afterwards.
  - `input=2us/frame worst=109us calls=600` with `0 shown late` and `gaps in refreshes 2x598`
    over the two reporting periods: the input read is now hooked on every launch rather than only
    where the window may be behind, and the frame it costs is what it cost before.
  - Nothing else in those 66 lines: no panic, no fault, no line about a hook it could not install.
  Started with `--config` naming a temporary file that says `ask_at_startup: false`, since the
  launcher's dialog waits for a keypress and nobody was there to give it one; the game's own
  `orb.yaml` was read by the DLL as always. Closed by killing it, having never left the title
  screen.
  - **And the files those chapters are kept in are found and named.** Two put in
    `pointdevice_resume/` by hand and a launch made the same way:
    `resume: 2 run(s) left unfinished: lunatic-marisa-b, normal-reimu-a-practice5`, in order and
    named for the runs they belong to. Which run a chapter belongs to is the same string, so this
    is also the naming the question chooses by. Both were taken away again afterwards, being
    nobody's runs. The second is a name orb wrote at the time and does not any more — a practice run
    is kept no longer, see [SPEC.md](SPEC.md) — and a file called that today is one somebody made,
    which is what both of these were.
- **The mode question over the game's own menu.** `Game Start`, `Extra Start` and `Practice Start`
  each put the question up; `Score` asked which of the two rankings. Answered with both, six times over
  one session, and the cursor started on whichever mode orb was already in — `was normal` on the ones
  after the first.
  - **On the press that would have chosen the item**, once the title menu's own decide was held back:
    `menu: Run is under the cursor, asking which mode` and `menu: Scores is under the cursor, asking
    which mode`, each on the frame the shot button went down and with the menu behind it untouched.
    Answered, `mode: answered on the pad` is followed by the item being taken as usual — the press is
    handed back for one read and the menu chooses it itself.
  - **The ranking follows the mode.** With normal chosen, the ranking screen — scene 6, the game's
    `SUPERVISOR_STATE_RESULTSCREEN` — opened the game's own file, which the log shows by there
    being no `score:` line at all for it. With pointdevice chosen, the same screen opened
    `pointdevice_score.dat`, once as it was built and again as it was written on the way out, and
    the title menu then read the same file when it rebuilt itself.
  - **Neither is an answer, and the menu does not move for it.** Cancelled five times in a row on the
    pad — `mode: not chosen on the pad; the menu is where it was` at 334495687, 334497000, 334498296,
    334499828 and 334500484 — with a fresh `menu: Run is under the cursor` between each, from a press
    that took 300 to 1200ms to arrive. That sequence is the proof the title menu never saw the cancel:
    its own back puts the cursor on `Quit` (0x4381b0), an item orb asks nothing about, so a cancel that
    reached it would have made the next press report nothing at all. The shortest of the five was
    answered 297ms after the question appeared, so the ten frames of grace are not in the way of
    somebody who means it. Nothing of the title was redrawn either — the fade it plays on the way back
    from an item is what the press being held back avoids.
  - **And on the pad, both ways**: the five cancels above and `mode: answered on the pad` with the mode
    taken. A menu of orb's freezes the game, so the game is not reading the pad on those frames and one
    does nothing there unless orb reads it itself — which is how it came to be missing, and why the log
    says which hand answered.
  - **And it is driven in tests now, keys and all.** `GetKeyboardState` went behind the seam and what
    the question decides — the two modes, the cursor, which key answers — went into `orb-core` beside
    it, apart from the labels it draws them with. Thirteen scenarios in `orb-sim/tests/mode.rs` press
    the keys somebody would: the cursor starting on the mode orb is in, so 完全無欠モード is one press
    away from a run that was one; either of the two keys the game decides with deciding here; the bomb
    key and escape cancelling and choosing neither; the ten frames of grace with the key that put the
    question up held down the whole way, and it still not answering when the grace runs out; a held key
    answering once rather than once a frame; a host that refuses to say what is down reading as nothing
    down; and the pad doing all of it and being named as the hand that did.

    **One of them found a defect.** Everything reads as released while another window is in front —
    otherwise typing elsewhere would drive the game — but zeroing alone made the way back an edge:
    every key read as up, then read as down, which is a press by the rule those menus use. So a key
    held through an alt-tab chose a mode on the frame the window came forward. Whatever is down on the
    first frame orb is reading again counts as already held now, and the first read of all goes the same
    way.
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
- **The stage retried, and the run given up**, on stage 6 Lunatic with the pad in hand. Three
  deaths in one run, each answered differently:

  ```
  died in chapter 1
  retry: asking about the stage again
  retry: the stage again on the pad                    8.1s after the question went up
  retry stage from chapter 1 (retry 1)

  stage 6 chapter 6 at frame 4661 (script 3172): a boss nonspell, boss, music=keep
  died in chapter 6
  retry: asking about the stage again
  retry: the stage again on the pad                    1.0s after it
  restore: the track has changed since this snapshot; taking the music down
  music: stopped through the game
  music: restarting bgm/th06_12.mid
  retry stage from chapter 1 (retry 2)

  died in chapter 1
  retry: asking about the run given up
  retry: the run given up on the pad                   3.1s after it
  retry: the run is given up; the game is on its way to the title
  score: pointdevice_score.dat opened in place of the game's own
  run ended after 2 retries
  f7918 scene=1                                        the title menu, 265ms after the answer
  f8115 scene=4                                        the game closed, 3.3s later
  ```

  - **The stage's start comes back, and the music with it**, which was the half of the retry menu
    no run had shown. The second death was in a chapter the boss's own track was playing under, so
    that chapter was snapshotted `music=keep` and the stage's own snapshot names a stream the game
    freed when the track changed. It was asked to do it rather than copied back — `StopBGM`, then
    `PlayAudio` with the path read out of the memory the restore had just put back — and the run
    carried on from the stage's start as far as a death in chapter 1. Between the two retries the
    fight was walked back up through chapters 2 to 6: a table boundary at script 1535, a midboss
    nonspell, its spellcard, the midboss beaten, and the boss's first attack.
  - **Both confirmations were answered yes, and the graces are not in the way**: 8.1s, 1.0s and
    3.1s between a question appearing and being answered, against the 12 frames a confirmation
    holds its keys off for.
  - **Giving up lands on the title menu and starts nothing.** `scene=1` 265ms after the answer, and
    no `menu: Run chosen, asking which mode` between it and the game being closed 3.3s later —
    which is what says the front end did not take the press that answered orb's question as an item
    of its own. `run ended after 2 retries` on the same millisecond as the title is the snapshots
    being dropped, by the same path that ends any other run.
  - **Neither score file was written.** `score.dat` and `pointdevice_score.dat` both came out of the
    session at the mtime they went into it with, hours before it started. The
    `score: pointdevice_score.dat opened` line 15ms after the answer is the title menu reading
    `clrd` back out as it rebuilds, which is an open and not a write.
  - **Refusing one, both ways**, from a later run on the same stage:

    ```
    died in chapter 3
    retry: asking about the stage again
    retry: the stage again — answered no, back to the choices
    retry: asking about the run given up
    retry: the run given up — answered no, back to the choices
    retry: the chapter again on the pad
    retry chapter 3 (retry 1)
    ```

    Two questions asked and refused on the cursor's own いいえ, and then the chapter taken from the
    same menu — so a refusal leaves the menu where it was with nothing done, and the chapter it was
    standing on is still the one the next press restores. Cancelling was pressed on four more
    questions later in the run, `— cancelled, back to the choices` each time, and the fifth answer
    gave the run up. What the log does not say is which button cancelled: bomb and the menu button
    both do, and the line names neither.
  - What this run could not show: the keyboard. Every one of these was answered on the pad.
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
  - **Measured when orb's file was `orb_score.dat`, the fork was the `own_score_file` key, and
    orb's file was started as a copy of the game's**, which is what those log lines say. It is
    `pointdevice_score.dat` now, the fork follows the mode chosen in the game, and nothing copies
    `score.dat` into it — see [SPEC.md](SPEC.md) — so the `started as a copy` and `not copied from
    the game's` lines above are not lines any session writes now. The seam and the comparison are
    the same code. What those changes left to check is in [TODO.md](TODO.md).
- **What a session counted about spell cards survives being stopped, and an attempt at a chapter
  counts.** Both were watched on 2026-08-05 in play.
  - A run ended anywhere but the result screen is followed by a trip through the game's own ranking,
    which is where it writes: `score: a run ended; what it counted waits for the trip through the
    ranking` at 371528265ms, `score.dat opened as the game's own, write` 297ms later, and
    `score: taken through the ranking in 84 update(s) — cur=1 wanted=1` — the scene back at the front
    end. The same shape with `pointdevice_score.dat` for a pointdevice run. Both modes, and both ways
    out of a run: orb's retry menu and the game's own `ESC`.
  - The counts read off the game's own screen afterwards were up: the attempt count against the card
    a chapter was retried at, and the capture count for a card taken in a legacy run stopped partway.
  - **Four attempts at this before it worked, and what each cost**, since the shape of the mistake is
    the useful part: writing `curState` — the game's *result* — instead of asking the way the game
    asks left the front end white twice and doubled once; taking `curState == 6` for the ranking being
    up caught a frame mid-transition; guarding the trip's loops on `CHAIN_EXIT_SUCCESS`, which is 0 and
    so also an ordinary chain answer, ended them after one update and left the screen standing where
    the player met it; and the front end's cursor was written into the menu object being discarded
    rather than the one built on the way back. The decompilation settles all four in a read —
    `ResultScreen.cpp:1527-1535` for how that screen leaves, `MainMenu.cpp:848` for the request being
    a reservation the front end acts on 60 frames later.
- **What a missing `pointdevice_score.dat` does to the unlocks: it locks them.** A launch on
  2026-08-04 with no such file — `score.dat` beside it untouched, mtime `2026-08-02_18:47:50` —
  showed `Extra Start` locked on the title menu with pointdevice chosen, and the log's only score
  line for that menu was `score: pointdevice_score.dat opened in place of the game's own` at
  338359890ms. So a failed read is not left as a no-op: `clrd`'s parse at 0x42b502 clears its
  destination before it looks for the chunk — four records memset at 0x42b535 — which is what
  answered the question [TODO.md](TODO.md) had been carrying about this.
  - The three reads were then told apart in the exe, and `MainMenu::AddedCallback`'s (0x43a5c0) is
    the only one the front end's own items are lit from: it fills `g_GameManager` at 0x69ccd0 and
    0x69cd30 with `clrd` and `pscr` and parses nothing else. The other two —
    `GameManager::AddedCallback` (0x41bcdc, once per stage) and the ranking screen's added callback
    (0x42f47f) — read all four chunks, ranking and captures included. The table of which parses each
    calls is in [SPEC.md](SPEC.md). The write has one caller in the whole exe, 0x42f5cd in that
    screen's deleted callback.
  - **The same session made the file, without finishing a run**: `pointdevice_score.dat` came out
    at 4,224 bytes, `4f733fc56b8e80d3a511acfc7ba8cb0d`, against `score.dat`'s 8,724. Leaving the
    ranking screen is what wrote it — the deleted callback writes whether a score was entered or the
    ranking was only looked at — so orb's file appears with an empty record rather than waiting for
    a clear.
  - `score.dat` was written in that session too, its mtime moving to `2026-08-04_19:01:40`, which is
    the same callback leaving a *normal* ranking. Whether its contents changed is not established:
    no md5 was taken before the launch, and the `004c8eda5a29a4ff985529838c21efe5` above is from
    2026-08-01, three writes ago. It reads `ec50b0a2e69c0e56c18d11ca68e9d73a` now. The lesson for
    the next session that means to say a file came out untouched is to md5 it first.
- **The front end's read is the game's own file and every other open follows the mode.** The bracket
  is on `MainMenu::AddedCallback` — see [SPEC.md](SPEC.md) — and a session on 2026-08-05 showed both
  halves of it:
  - `unlocks read hook installed, original at 0x02940000` at 357601218ms, so the six bytes at
    0x43a464 were the `push ebp; mov ebp,esp; sub esp,0x10` expected of it.
  - **Nothing between that and the title menu**: the menu was up at 357605156ms (`f0 scene=1`) with
    no `score:` line anywhere in between, where the same point in the session before the bracket had
    one — `score: pointdevice_score.dat opened in place of the game's own` at 338359890ms. A
    pass-through open is not logged, so a missing line is the read going to `score.dat`.
  - **And the ranking still pointdevice's own**: answering pointdevice at the *Score* item
    (357616406ms) and the screen coming up at 357617437ms (`f555 scene=6`) was followed by
    `score: pointdevice_score.dat opened in place of the game's own` 31ms later.
  - Neither file was written by any of that: `score.dat` and `pointdevice_score.dat` kept the
    mtimes they went in with, `2026-08-04_19:01:40` and `2026-08-04_19:03:35`.
  - What the log cannot show is what the menu then offered. Whether `Extra Start` and the practice
    stages are lit, and what the spell card history holds in each mode, is in [TODO.md](TODO.md).
  - **Both files written by their own ranking screen, in one session.** Answering pointdevice at the
    *Score* item sent the read to orb's file at 357617468ms and the write at `2026-08-05_00:21:53`;
    answering the game's own ranking next — `mode: normal, was pointdevice` at 357638484ms — read and
    wrote `score.dat`, mtime `2026-08-05_00:22:28`. The mode an answer leaves behind cannot reach a
    screen that reads the file, because every item that opens it asks first.
  - **The inferences above rest on a line that is no longer missing.** Those sessions logged an open
    only when the file was swapped, so a read of the game's own file and a read that never happened
    looked the same, and reading one for the other is exactly the mistake that was made. Every open
    now says which file it landed in and what it was for — see [SPEC.md](SPEC.md) — so a session run
    today says these things outright instead of by absence.
- **Replay writing suppressed.** Only the write; the game's scene-change teardown, which
  goes through the same function with null arguments, still runs. Stubbing the whole
  function crashed the game later. This is what `--clear` still does. A pointdevice run is not
  offered the screen that saves a replay at all now, which is a different mechanism and unmeasured
  — see [TODO.md](TODO.md).
- **Replay-driven automation.** `--replay` with `--speed` lets a replay of a full run do the
  playing, for building the table and for the stress mode.
- **A whole run driven end to end with no game and no Windows, by a 紅魔郷 that plays the game's
  part.** In the suite rather than on the machine, and here because what it establishes is what a run
  *does*, over the code that does it, with orb reached the way the DLL is reached: `attach_to`
  puts a runtime in place with the fake game's own functions where `hook::install`'s trampolines would
  be, and the game then calls `run_draw_chain` and `run_calc_chain` in its own draw-then-update order,
  with its input read inside the update. See [docs/adr/0001](docs/adr/0001-a-fake-th06-drives-orb-end-to-end.md).

  The fake game's state is its memory and nothing else, so a chapter restored underneath it takes the
  run back with it. It presses no buttons for anybody: a scenario says which window is in front,
  presses keys, and runs frames, and reads back the game's own memory, the game's own record of a
  spell card, and what orb put in the log.

  **`pointdevice_run.rs`, one 完全無欠 run, in the order somebody playing meets it.** `Z` at the title
  menu puts orb's question over it — `menu: Run is under the cursor, asking which mode` — and the
  game's own cursor has not moved, the press having been held back. Answered, the press is handed over,
  the shot type select takes it, and `resume: nothing was left of normal-reimu-a` says the file was
  looked for. The stage begins and takes its first chapter at frame 248 — `STAGE_SETTLE_FRAMES` plus
  the whole of `MUSIC_WAIT_FRAMES`, a laid-out game having no track for the snapshot to wait for — and
  from there the count of lives is painted over: a quad covering the row the game counts them in, with
  the word `DISABLE` over it — read off the screen as text, by baking the same string through the same
  font and holding it against what the texture the quad went through was uploaded with.

  **Which is how every one of these reads the menus.** The mode question: `モードを選ぶ` with its two
  choices a line apart, 完全無欠モード in `SELECTED` and レガシーモード in `NORMAL` — which is what says
  where the cursor is, since `menu_ui` draws it as a colour — the `▶` on that same line and to its left,
  and under them the line saying what the choice means. The retry menu: the chapter it is offering by
  the name the detector gave it, `MIDBOSS SPELL 1`, `RETRY 0` under it, and the three ways on a line
  apart with the cursor on チャプターをやり直す.

  Then a fight and its card, each a chapter of its own: `chapter 2 at frame 400 (script 400): a midboss
  nonspell` and `chapter 3 at frame 500 (script 500): a midboss spellcard`, each written down as it
  begins — `chapter 3 (MIDBOSS SPELL 1) at frame 500, 500 frame(s) of buttons`. A death on the card,
  and `died in chapter 3` with the game frozen under the menu: ten frames later its own clock has not
  moved and something is still being drawn over it. Answered チャプターをやり直す, **the whole `State`
  read back out of the memory is the one read at the chapter, field for field**, `retry chapter 3
  (retry 1)`, and `retry: attempt 2 at this spell card`.

  **The attempt reaches the 完全無欠 ranking, and the screen shows it.** The run is given up two
  chapters later, which is where orb walks the game through the screen that ranking is shown on with
  nothing drawn — `score: the captures in memory cleared for the ranking about to be read`, `the
  ranking is up`, `asked it to leave`, `taken through the ranking in 5 update(s)` — and going down is
  what writes the file. So the scenario then opens that ranking the way anybody does: the title menu's
  `Score`, orb's question over it (`どちらのスコアを見る`, which is not the question a run gets), and the
  screen itself, whose row for that card reads **2** across from `CARD 3` — the game's own attempt
  where the card started, and the one the retry was. Nothing on it says 1, which is what it would say
  if the retry had not been counted or if the read that fills the record had lost what the session
  counted. Cleared by that read and put back by orb, which is the mechanism rather than a coincidence.

  **And the same run picked up again**, out of the file: the record of what a run pressed goes when the
  run ends, and what the playback is fed is what `resume::load` read off the disk —
  `resume: normal-reimu-a was left; asking where to start`, out of a file whose own line names the
  chapter it holds. つづきから builds the stage again, seeds the generator where the file says
  (`resume: the generator seeded 0x6a06`), plays **700 updates** of the run's own buttons in inside the
  one frame that built it with nothing of it drawn, and lands with **`resume: the landing is the frame
  that was written down, field for field`** — the generator, the count of numbers drawn, the player's
  place, the score at 5250 and the run's numbers all agreeing. The record puts back the captures the
  playback would otherwise have counted its own way through (`4096 byte(s) of captures put back`) and
  counts one attempt for the landing itself: `resume: attempt 3 at this spell card`.

  **`legacy_run.rs`, the same game answered the other way**, which is every one of those things orb has
  to *not* do. レガシーモード chosen — `mode: normal, was pointdevice` — and then: no chapter over the
  frames where the other run took three, nothing drawn over the count of lives, `resume: stage 1 of a
  run orb is not keeping; nothing of it is written down`, and a death that costs a life with no menu
  offered and the game still updating on the frame after — and no `DISABLE` anywhere on it. Nothing is
  left to pick up. Out of lives, the run ends and goes through the same ranking screen, and that
  screen's row for the card reads **1** and never 2 — the card's own start and nothing of orb's. Which
  of the two files it lands in is the import hook's and stays `score.rs`'s own tests' to hold, since a
  scenario cannot install one.

  **What it found, in the code and not in itself:** `memtrack::regions` answered a laid-out game out of
  a `#[cfg(test)]` branch, and `cfg(test)` is false in a crate compiled as a dependency of a test
  binary. So the first run of the first scenario got the production heap walk with no heaps tracked —
  a chapter covering `.data` and nothing else — and the clock of the attack a chapter began on did not
  come back with the chapter, the fight's own block being outside it. It is a seam facade now,
  `orb_api::mem::game_regions`, and a chapter over a laid-out game covers **3 regions of 2570748
  bytes** where it covered one.
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

- **Exactly 60fps, locked to the game's monitor, whether or not its rate divides into 60 — on a
  desktop whose compositor is timing that monitor.** Which is every desktop of one monitor, and the
  three rows below. It is not every desktop: see *A mixed-rate desktop* under this heading.

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
- **A mixed-rate desktop: the compositor is not timing the monitor the game is on, and the flush
  follows the compositor.** Measured with `scripts/compositor-probe.c` on three monitors — DISPLAY1
  the primary at 120Hz, DISPLAY2 at 120Hz, DISPLAY3 at 144Hz:

  | | |
  | --- | --- |
  | what the compositor says | `qpcRefreshPeriod=6944.4us`, 144.00Hz, `rateRefresh=10000000/69444` — the 144Hz monitor's rate, **not the primary's** |
  | what `DwmFlush` waits for | 119 gaps, mean 6946.1µs = 143.97Hz |
  | with a window on DISPLAY2 (120Hz) | `monitor_refresh=120Hz`, flush mean 6946.0µs = 143.97Hz |
  | with a window on DISPLAY3 (144Hz) | `monitor_refresh=144Hz`, flush mean 6946.1µs = 143.96Hz |
  | with a window on DISPLAY1 (120Hz, primary) | `monitor_refresh=120Hz`, flush mean 6945.5µs = 143.98Hz |

  So the two numbers `settle` compares really can disagree, and which monitor the window is on
  changes only the first of them: 119 flushes apiece put all three windows on the same 143.97Hz.
  `EnumDisplaySettingsW` answers about the window's monitor and the compositor answers about its
  own, and a flush waits on the compositor's whatever the window is doing.

  **The compositor followed neither the primary nor the game's window but the fastest monitor**,
  which is what makes this ordinary rather than exotic: a 120Hz primary to play on and a 144Hz
  monitor beside it is the configuration, and it is the one this machine was already in.

  The gaps are a grid with jitter, not a metronome: least 5724–5878µs and most 8020–8593µs about a
  6946µs mean, so ±1.1–1.4ms. And the shape of it is not spread over that range — in 200µs bands about
  the mean, 96 of the 119 gaps are within 100µs and the rest are single excursions, one long gap
  followed by a short one summing back to twice the refresh. So it is one late *wake* rather than a
  drifting clock, which is how `orb-sim` models it now: an exact grid of blanks with the flush's return
  delayed off that distribution. See the note at the top of `orb-sim/src/display.rs`.

  **And the game on such a desktop runs a fifth too fast.** Measured: `orb.exe --log=quiet --pacing`
  with the game fullscreen on the 120Hz primary and the 144Hz monitor attached, four periods of 600
  frames —

  | | |
  | --- | --- |
  | what `settle` decided | `frame: 120Hz monitor but the compositor is timing 144Hz; pacing by the clock` |
  | what it actually did | `0 frame(s) paced by the clock` in the same run, every period |
  | the rate | `13966us apart`, `14090us`, `13897us`, `13894us` — **71.0 to 72.0 frames a second**, where sixty was asked for |
  | the cadence | `off the cadence by under -1500us:552` of 599: almost every frame short by more than a millisecond and a half |
  | the misses | `refreshes past the blank aimed at 0x580 1x18 3+x1` |
  | the compositor's share | climbed 2800 → 3400 → 3600 → 3900µs, chasing misses more time could not fix |

  13888µs is two refreshes of 144Hz exactly, and that is what the frames are landing on: the blanks
  are the compositor's and the cadence is counted in the monitor's. **72 frames a second is the
  number `frame::settle`'s own comment recorded years ago** — "144 read for a 120Hz display ran the
  game at 72 frames a second" — so this reproduces it rather than finding something new. What is new
  is that it happens with the game on the *primary*, and that the log names the wrong mechanism while
  it does.

  Two lines of the same run contradicting each other is the defect to fix: `adopt` sets
  `BLANK_PACED` from the whole multiple alone, so the `agrees` check changes what is written and not
  what is done. See *Put Windows behind a seam* in [TODO.md](TODO.md).

  **`orb-sim` predicted 48 frames a second for this and was wrong**, which is worth keeping because of
  what was wrong with it. It modelled the host as a metronome: without jitter no frame overshoots far
  enough to be filed as late, so the compositor's share never climbs, the handover stays late and the
  frames take the *third* refresh. On the machine the ±1.4ms files misses, the share climbs to 3900µs,
  the handover moves early and they take the *second*. Modelling the wake delays the host really has
  moved the simulator to 69–70 frames a second, the same direction and near the number — but the
  remaining gap is the compose time, which nothing reports, so the scenario asserts the direction and
  this table keeps the number.
- **The loop itself is driven in tests now, against a simulated Windows.** `frame.rs` is in `orb-core`
  with its host calls behind `orb_api`, and twenty-one of the forty-one scenarios in `orb-sim/tests` run the real
  `pacing`/`settle`/`wait` over a display a test declares: the counter, `Sleep`, `timeBeginPeriod`, the
  monitor's reported rate, `DwmGetCompositionTimingInfo` and `DwmFlush`. Its own thirteen tests were all
  arithmetic and not one of them drove the loop.

  What they establish, none of which had a test before: the rate each reported refresh rate gets (60,
  120, 144, 165, 240 all hold sixty; 59 and 119 get the display's own rate, which is right), the
  fractional 2-2-3-2-3 pattern, the budget rising after a frame overran, the allowance climbing to
  cover a compositor that has slowed, the clock path when no rate is available, a host that refuses the
  millisecond timer, and the mixed-rate desktop above.

  How they judge it is the same question somebody playing would ask — **what share of the seconds were
  sixty frames a second, within half a frame, from three seconds in.** Not the exact turn: the simulated
  host is deliberately not a metronome, since a scenario that only holds against one is a scenario about
  arithmetic. It draws its wake delays and its compositor's spikes from a seeded stream, each scenario
  runs over several seeds, and the seed is in every assertion so a failure replays. A display the pacing
  accepts holds every second of the run; a mixed-rate one holds none.

  **A spike costs its own second and never the run, and how much it costs turns on the rate.** Measured
  with the spike rate deliberately three hundred times the one half an hour of play suggests, so that
  several land in every fifteen-second run: at 144Hz and 165Hz not one second is off sixty, at 120Hz and
  240Hz the worst reads 59.03 and 59.52, and **at 60Hz the worst reads 58.07 and one seed lost four of
  its twelve seconds.** 60Hz is the rate with no room — a missed blank there costs a whole refresh, which
  is a whole frame, and the frame after it cannot come early enough to make the second back. The
  allowance climbs and nothing cascades at any of them.

  **And asking for the promise rather than for the behaviour found a deadlock.** The scenario asserts
  sixty frames a second for *any* compose time the pacing has room to cover, and seven of forty (rate,
  compose) pairs failed it: a compositor taking 3200µs or more *persistently* — not spiking to it — locked
  a 60Hz display at 30.00 frames a second, over 20,000 frames, with the allowance frozen at 2800µs and
  10,121 of the misses charged to a stage load long over. 144Hz and 165Hz did the same from 3800µs, at
  52.37 and 49.16.

  **The cause was one word of state.** `measure_compose` excuses the frame after a stage load, and it did
  that by setting a flag that only a frame *landing where it was aimed* cleared. A compositor slow enough
  that no frame lands is then a compositor no frame ever climbs for: the flag stays set, every miss is
  charged to the load, and the climb that would fix the rate never runs. Taking the flag instead of
  reading it — one frame's grace, which is what its own comment always said — fixes every one of the seven.

  What set the flag in those runs was not a load at all. A cadence is a whole game turn however fast the
  display is, so at 60Hz one refresh *is* one turn and the smallest miss there is lands on the guard's
  boundary, with jitter deciding which of the two it gets called. Measured: at 60Hz a run of one-refresh
  misses never moved the allowance off its 2500µs start, where at 120Hz the same miss climbed it to 6249µs.
  A real load is nowhere near that branch — measured at 60, 120 and 144Hz, a quarter-second frame costs
  exactly one frame its blank and the frame after it lands, so nothing is charged there — and three loads
  in a run leave the allowance exactly where it was, which `pacing_load.rs` now holds it to.

  **What is left is a limit and not a defect**: the allowance cannot pass one refresh without showing the
  frame a refresh early, so a 240Hz display cannot cover a compositor wanting more than 3124µs and settles
  at 48.00 instead. That is asserted as what it is, with nothing ignored — see *A 240Hz display leaves the
  compositor less room* in [TODO.md](TODO.md).
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
- **A long frame no longer runs the game fast for a third of a second afterwards.** A 252ms
  `RunCalcChain` — a run ending and the next scene being built — pinned the work estimate to its
  ceiling, three quarters of a game frame. The frames that followed wanted two milliseconds and
  were then started 12.5ms before a blank 8.3ms away: handed over that early they were composed
  for the blank *before* the one they were aimed at, `DwmFlush` returned there with them, so the
  anchor the next aim is counted from moved a refresh early and the frame after was handed over
  just as early again. One frame per refresh is one update per refresh, so the game and everything
  in it ran at double speed for the thirty frames the estimate took to decay back —
  `gaps in refreshes 1x29 2x569` after one such frame and `1x88 2x547 5+x3` after three.

  A frame that wanted more than the whole budget does not set it now, being a scene built rather
  than a frame drawn. Watched over a played pointdevice run with chapter retries in it: 52,833
  frames in 86 periods, twelve of them frames of five refreshes or more — six in the 220–270ms
  class and one of 499ms.

  | | |
  | --- | --- |
  | the estimate over the session | 3166–6418µs, against the 12,500µs ceiling it used to be pinned to |
  | the period holding the 499ms frame and two more stalls | `gaps in refreshes 2x681 3x1 5+x3`, and `refreshes past the blank aimed at 0x682 3+x4` |
  | one-refresh gaps in the whole session | two, of 12,262 and 12,366µs, which are under the bucket's 1.5-refresh boundary rather than doubled frames — both reached the flush 620 and 756µs before the blank they were aiming at |
  | frames shown before the blank they were aimed at | none, over all 86 periods |
  | the spread | 25,970 gaps within 250µs of the cadence and 20,412 within 750µs of it, with both ends of the band — beyond ±2ms — empty |

  That last count is the other half of the fix, and it is why the fault could sit in a run for as
  long as it did: `MISSED` counts refreshes *past* the aim and takes `max(0)`, so a frame a refresh
  early read as one that landed exactly where it asked to. Every counter said the pacing was
  perfect and only the frame rate on the status line disagreed. It says so now, and reading zero
  over that session also answers the case that needs no load at all — a 10ms update followed by a
  2ms one, where the estimate would sit a refresh above the lighter frame's work — which did not
  happen in fifteen minutes of play.
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
- **A brush stroke over the count of lives in a pointdevice run**, with `DISABLE` on it, seen on
  the screen in stage 1. The stroke is `brush.rs`: a picture of a real one, baked to 144x30 of
  coverage — a generated stroke was tried first, through several shapes, and none of them read as a
  brush. The count is not painted out; the game is asked to repaint that row every frame — `Gui`'s
  `flags.flag0` written back to 2, which is the game's own "this row changed" bits — so the stars are
  drawn again under the ink and show through where it is dry, and nothing of orb's accumulates on a
  panel that is otherwise repainted only for a stage's first 250 frames.

  The two strips the stroke reaches past that row are painted with the game's own panel tile,
  `front.anm`'s 32x32 at (0, 224) out of the texture in slot 13, laid on the grid the game lays it
  on. The first attempt read `g_AnmManager + textures` without dereferencing the pointer first —
  `live_handles` does dereference it — found no texture, fell back to a flat colour, and the two
  strips were visible as exactly the patch the tile exists to avoid. That is what the line
  `lives: the panel's own tile is what the strips are painted with` is in the log for, against
  `lives: no panel tile; the strips are painted flat and will show as a patch`.

  **Drawn on the run rather than on the frame**, which it was not at first: the count came back for
  an instant at every stage boundary. A stage transition leaves the gameplay scene for exactly one
  frame — `f44096 scene=3 stage=2` and then `f44097 scene=2 stage=3 frames=1` — and the log's
  timestamps put those 265ms apart, because the game builds the next stage inside that frame. Every
  transition of the run went the same way, 250ms to 265ms each, so the one frame the mark was asked
  about and said no to was a quarter of a second of screen. The frame a chapter is put back on is
  the same case for a different reason: the update drops what it knows of the frame it froze on,
  since a chapter put back is not a continuation of it. Both are a run's own frames, so what the
  mark is drawn on is whether the run being tracked is one somebody is playing — taken with the
  stage's snapshot, where a run and a replay have just been told apart, and dropped when the run is
  left. Both were seen gone on the screen, at a stage boundary and after a retry.

  **And past the end of the run, for as long as the game is still painting that row**, which is what
  leaving a run showed once the frames above were right: the stars, plain, for the whole fade to the
  title. `esc` and then やめる ends the run on a single frame — `run ended after 8 retries` and
  `f20724 scene=1` together, with `f20700 scene=2 paused` before them — and the panel stays on the
  screen after it, so the row the game paints on that frame is the one left standing there. A mark
  that stopped with the run stopped one frame early.

  What ends it instead is the game's own `Gui` no longer being in the draw chain. `Gui::RegisterChain`
  at 0x41b252 registers two statics: 0x69bc7c through `AddToCalcChain` at priority 0xc, and 0x69bc5c
  through `AddToDrawChain` at priority 0xb with `Gui::OnDraw` (0x417502) at +4 and `&g_Gui` at +0x1c.
  Which of the two `Chain` lists is which came out of the lines they log — "add calc chain (pri = %d)"
  at 0x46afb8 against "add draw chain (pri = %d)" at 0x46afd4 — and the draw list's head is the calc
  list's 0x20 further in. So the mark is drawn while 0x69bc5c is in that list, which also means it can
  never be drawn over a screen that is no longer the panel.

  How much that is worth is one frame, and the log says so where each run ends:
  `lives: the mark stayed on the panel for 1 frame(s) after the run ended`, against `run ended after
  8 retries` and `f676 scene=1` on the frame before it. One frame — the last one the game paints that
  row in — is the whole of what was missing, and it is a frame that stays on the screen for the fade
  to the title. Giving the same run up from orb's own retry menu instead needed none: no line, the
  chain already cut on the frame the run ended, and nothing painted that row after the last marked
  frame. Both were seen on the screen keeping the mark to the end.

  Two, not one, in the ask. One was tried first, to leave no repaint standing for the frame after the
  last marked one, and it is not what put the count back: the panel being laid over a stage's first
  250 frames sets all five of those fields to 2 itself, at 0x41a2b6, so during those frames the value
  orb writes decides nothing.
- **A pad that only the game could see, now answering orb's own menus.** The mode question was
  answered on the pad — `mode: answered on the pad` — with the pad in XInput's second slot, which is
  the case that used to leave orb's menus dead while the game's own worked. Why: the game polls its
  own DirectInput device where its enumeration found one and never asks winmm, and winmm did not
  have the pad at all. Measured on this machine, and each of the three interfaces was asked: winmm
  reports 16 devices, index 0 being `mid=413d pid=2104` with no buttons and no axes answering every
  field zero, and 1 to 15 `JOYERR_UNPLUGGED` at 13µs each; DirectInput enumerates
  `Controller (Xbox 360 Controller)`; XInput has it in slot 1 with slot 0 empty. So orb's menus now
  read what the game reads — `Poll` and `GetDeviceState` on its controller, the buttons by the
  numbers the mapping names and the Y axis against `cfg.padYAxis` in the ±1000 the game gave its
  axes — and a device with no buttons and no axes is no longer taken for a pad: it drives no menu
  and its caps are not written into the game's calibration.

## The status line

Drawn with GDI onto the window, in the black beside the game: the chapter, `RETRY`,
`INPUT LAG` and the frame rate, stacked in whichever letterbox bar is wider and lined up
with the game's edge. Shown whatever the game is doing, including demos and menus.

Not in the game's back buffer, for two reasons that were both measured: all of it is shown
inside the letterbox, and the game does not clear it between frames — text there
accumulated into a row of white pixels.
