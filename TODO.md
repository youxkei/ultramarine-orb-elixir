# To do

## What a played run still has to show

A `--clear` run has been through stages 1 to 6, the ending and the result screen with chapters
taken and no replay offered — `orb-e2e`'s `a_clear_on_demand` holds what that run measured. What it
cannot show is anything that needs somebody to be hittable, or a mode nobody chose.

**The retry menu on the keyboard.** Every item of it, both confirmations, and both ways of refusing
one are measured, and every one of them was answered on the pad — `retry_ui.rs`'s
`a_confirmation_holds_its_keys_off_before_it_answers` and the five beside it are the shape. The
keyboard goes through the same `Pressed`, so what is left is watching it: `z` or return decides, `x`
or escape cancels, and the graces are the same frames.

**Which button cancelled.** `retry: ... — cancelled, back to the choices` does not say, and neither
does the mode question's own cancel line beyond naming the pad. Bomb and the menu button both cancel
now, so a session where one of them has stopped working looks exactly like a session where nobody
pressed it — which is the fault the mode question's `By` was added for. The fix is the same shape:
say which, in the line that says it happened.

**What the confirmation looks like on the real screen.** Its question is the widest text either of
orb's menus draws, against a play field 384 pixels across, and a line clipped at that edge cannot
be read at all. Worth looking at once: the question, and the two answers under it with the cursor
on いいえ.

**What a question asked on the press has left unwatched.** Both questions going up on the press and both
being cancelled without the screen moving were watched on the pad — the presses 334506843, 334508187,
334509156 and 334510156, each cancelled within 600ms with the file still there after all four, and the
screen's own back going to the character select at 0x436c49 is what says it never moved. What that
sitting did not reach:

- **`はじめから` through this path.** `つづきから` was answered from the same screen ten seconds later —
  7476 updates played in and the landing the frame written down, field for field — the press handed back, the
  screen choosing its own item, the chapter going in on the frame the run was registered. The other item
  is the one that writes over a chapter, and what it has to show is the fresh run starting and the file
  going only when that run reaches a chapter of its own.
- **The keyboard.** Every line of the sitting says `on the pad`. The same `Pressed` is what both hands
  answer through, so what is left is watching it: `z` or return deciding, `x` or escape cancelling.
- **Every other item of the title menu**, none of which orb asks about and all of which now go through a
  press held back and handed over: `Replay`, `Music Room`, `Option`, `Quit`. No press anywhere may be
  eaten, and `Quit` is the one that would say so loudest.
- **The Extra shot type select**, which reaches the same state the same way, and a shot with no
  `中断データあり` under it, which must start its run on the press with no question at all.

**What the widest line of the mode question looks like.** Its longest is now
`進行状況は自動的にセーブされ、いつでも続きから遊べます` — 26 characters at an em of 15 against a 640-wide
output, so roughly 400 pixels and inside the screen by arithmetic rather than by having been looked at.
Worth one glance, with the cursor on each of the two modes, since the two now draw a different number of
lines.

**The rest of what the pad now reaches.** The mode question answers on it, on both of the two devices
the game's own read has — `orb-e2e`'s `mode_on_the_pad` over a controller the game owns, and
`orb-e2e`'s `mode_on_a_winmm_pad` over a pad winmm has where the game owns none, which is the path that
used to work and the one that must not have been broken. Which leaves two things that go through the same
reading and have not been pushed. The retry menu: up and down on the stick and on the d-pad, shoot
deciding, bomb or the menu button cancelling. And the settings dialog, which is the launcher's own reading
and not the game's — its line should say `a pad on XInput, pushed N time(s)` rather than `no pad
answered`, and that is where XInput's buttons being put into the order the game's mapping names them gets
tested.

**What a run with a real pad is now the only witness to** is that the two winmm reads still read: they went
behind the seam — `orb_api::joystick::position` and `caps` over `JOYINFOEX`'s and `JOYCAPSA`'s own
layouts — and the thread they happen on is spawned through `orb_api::thread::spawn` so that a scenario's
simulated host reaches it. Nothing about either is different in a shipped launch, and nothing but a launch
says so. The line to look for is the one the thread writes: `mid=045e pid=02ff … 16 buttons, 5 axes, X
0..65535, read in Nus`, and then `joystick: 250 reads, Nus each`.

**A life gained under the brush over the lives.** The mark itself is on the screen —
`orb-e2e`'s `the_mark_over_the_lives` holds what that showed — and what it has not been through is the
count changing under it: an extend at
10,000,000 or a 1UP item, where the star should appear under the ink rather than nowhere. That is
also the one moment the game would have repainted that row without being asked, so it is where orb
asking every frame and the game doing it anyway could disagree.

Two more of the same kind. A stage's first 250 frames, where the game is still laying the panel's
tiles itself over everything including the strips orb paints. And a stage where the count reaches
eight, which is as far right as the stars go — 624 — and so the only case that tests the stroke's
right end covering them.

What it costs is in the log already: every frame of a pointdevice run now opens an overlay frame
of its own, where before a run only did that on the fourteen frames of a wash, and an overlay
frame is a state block captured and applied. So the `draw` phase per period is the figure to
read — against the same stage in normal mode, which draws no mark.

**A run in normal mode**, which nothing has ever run: no chapter is observed, so no snapshot is
taken and no wash goes off; dying costs a life and puts up no menu of orb's; the status line loses
the chapter name and the `RETRY` line and keeps the lag, the compose time and the frame rate. The
score goes in the game's own `score.dat` — `004c8eda5a29a4ff985529838c21efe5` at the time of
writing, with a copy beside it, so whether the right file moved is one md5 each. And a normal run
*is* still offered a replay to save, and can still save one.

**The score written on the way out of an ordinary pointdevice run.** `--clear` refuses every write,
so what it proved is that `DeletedCallback` runs, not that it writes. The
`score: pointdevice_score.dat opened` line does not settle it either, which a played run has now
shown: it goes in on every open of the file whatever the access was asked for, so the title menu
reading `clrd` back out of it writes one too. Two runs abandoned have now left both score files at
the mtime they already had — one by closing the game while the retry menu was up, which was the
only way out of a run at the time, and one given up from the menu's own third item. Neither of
those reaches an ending or a game over, which is the case still open: what settles it is the file's
own mtime, or its md5, after a run that reaches one.

**The stick moving a cursor on a menu of orb's.** Both of orb's menus have now been answered on the
pad — `orb-e2e`'s `mode_on_the_pad` — but nothing says which part of it moved the cursor, and a d-pad and
a stick are read from different fields. What is left is watching the axis: the dead zone is a
quarter of the travel either side of centre, taken from `g_JoyCaps`, and this pad's caps had to be
written there by orb before the game would have believed them.

**Two more about the question itself**, neither of which a menu session reaches:

- that the demo does not start while it is up. The idle counter is in an update that is not running,
  which is the argument, but 720 frames of it has never been sat through.
- what the key that answers it does to the difficulty select. `g_CurFrameInput` is left as every
  button so that no edge exists on the frame the game carries on into: the cursor should not move,
  and a direction genuinely held across the answer should still move it on the frame after.

**What the front end offers with the bracket in.** Which file each open goes to is measured —
`orb-e2e`'s `the_score_file`'s `the_front_ends_read_is_the_games_own_file_and_the_ranking_follows_the_mode`
holds that session — and what is left is the half no log can show: `Extra Start` and the practice
stages lit in pointdevice mode where a `score.dat` has earned them, which is the whole point of the
bracket and has been read off the addresses rather than seen. One glance at the title menu with
pointdevice answered says it.

**Whether a pointdevice clear should unlock anything.** It does not now: the clear is written into
`pointdevice_score.dat`'s `clrd` — the game writes the file whole — and the front end is lit from
`score.dat`, so clearing the game in pointdevice mode leaves the Extra stage locked. Someone who
plays nothing but pointdevice runs is never offered it. The fix is the union of the two files'
`clrd`, which means orb parsing both and handing the game the answer, and with it orb knowing the
format and the encryption it has so far stayed out of. Nobody has been asked whether that is worth
it.

## What is left of the spell card record

Both halves work and are measured —
`orb-e2e`'s `the_score_file`'s `a_run_ended_away_from_the_result_screen_is_taken_through_the_ranking_to_write`
holds that session: a session that stops keeps what it counted, and a chapter retried counts an attempt.

**Who counts an attempt.** orb does it, at the chapter retry and at a resumed run's landing, by writing
`Catk::numAttempts`. The cleaner shape is the game counting it, which needs the chapter's snapshot to
predate the game's own count — a seam at the function that starts a card (the one holding 0x4096df), so
a retry replays that update. orb's own increment comes out at the same time or it double-counts.

**The pause when a run is given up.** The trip's eighty-odd updates run inside one frame, so the frame
is long enough to feel. Spreading them over frames is what would be drawn, which is the ranking
appearing for a second — the thing that was fixed. Nobody has been asked whether the pause is worth
trading back.

**A window closed mid-run** writes nothing: there is no front end to take through and the loop is on
its way out. 紅魔郷 loses that record too.


**That the spell card history is the mode's own now.** It read as `score.dat`'s in pointdevice mode,
twice, and the mechanism is settled — see [SPEC.md](SPEC.md): the record lives in one global the
parse never clears, and the file is written from that global. Two things were done about it, and
neither has been through a session. orb empties the block before a ranking is read, and the file that
had `score.dat`'s history in it was moved aside as `pointdevice_score.dat.dirty` — it was 4,224 bytes,
`a09a505fb32c41a5b03083f50f48a38d`, and the 8,724-byte `pointdevice_score.dat.bak` beside it is the
same size as `score.dat`, which is what the `CopyFileA` that used to seed orb's file left.

What a session has to show: the pointdevice ranking's history empty on a file that has no captures,
`score: the captures in memory cleared for the ranking about to be read` in the log at each ranking
read and not on the way out of a run, and then a card captured in each mode counted only in that
mode's file. The one that would say the clearing went too far is a run whose captures do not reach
its own file.

**The `orb_score.dat` an earlier version wrote** is not read or renamed by anything. A session that
has one has its old scores in a file nothing opens, and whether that is worth a rename is a question
nobody has been asked.

**A window of a chosen size, with nothing else resizing it.** The window comes out exactly the size
asked for — `orb-e2e`'s `the_window` holds the numbers — but on the machine it was run on a borderless tool then took
it to 4:3 filling the screen, so what a window of orb's own actually looks like has not been seen.
The game has to be left out of such a tool for the run. What is still to watch: the game
letterboxed inside a 16:9 window with the status line in the black beside it, that the 16x40 the
launcher allows for a frame is enough for the sizes it offers — this machine's frame was 6x40 —
and what a 4:3 window does to the status line, which then has no black to write in and should say
so once in the log.

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
the chapter restored onwards, and a menu that lists what is there rather than the two fixed
places it offers now.

## What a restore across a graphics load looks like

The crash it used to cause is gone — `0xc0000005` reading `0x313d616d`, the bytes of `ma=1`, inside
`AnmManager::ReleaseTexture`, checked by stepping back into stage 6's midstage again where it had
crashed twice — because the memory holding
Direct3D's texture handles is left as a restore finds it, which `snapshot.rs`'s
`a_live_handle_is_left_where_the_restore_finds_it` holds. What is left is cosmetic and has not
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

**Three of the four things that used to hardcode 紅魔郷 are tables now** — see
[docs/adr/0004](docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).
`orb_core::game::KNOWN` is the one table both halves read, the launcher's `GAME_EXE`, `GAME_EXE_MD5`
and its error naming 1.02h read out of it, `orb/lib.rs` chooses its game at the attach instead of
holding a `static GAME: Th06`, and `orb-sim/tests/fake` is a host half and a 紅魔郷 half. What is left of
that list is `game/th06/chapters.rs`, where `MIDSTAGE` is seven stages because 紅魔郷 has seven — and
a game that declines chapters does not reach it.

Two things to settle before the second game is *played* rather than merely paced: whether `State`
says everything a chapter needs for a game whose scoring or resources work differently, and whether
the midstage table's shape — script frame numbers per stage — holds where stages are not one
script on one clock. A game that declines chapters has neither in its way.

The plan for th07 — the `Game` that declines what has not been measured, the game chosen at the
attach, and the one e2e scenario that says the seam holds — is
[docs/adr/0004](docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md). Its steps 1 to 5 are
built; step 6 is below.

### What 妖々夢 does not get, and what each one would take

Two launches on the machine settled what orb does there — see
[docs/adr/0004](docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) for the log. It
attaches, matches the table, patches the update and the draw over their real prologues, sizes the
window, and **does nothing else at all**. What it does not do, in the order the effort goes:

**orb's own frame loop, which is the one thing a run *disproved*.** Replacing 妖々夢's frame at 0x4346e0
took the game down on its first frame — a null `rep movsd` at 0x44f6aa into a render-state block that
frame sets up and orb's loop does not. `Hooks::render` is `None` now, so the game keeps its own cadence
and its own frame of input lag, which is the whole of what orb would otherwise be there for. What it
would take is reading the rest of that frame rather than one more address: the two calls on 0x4b9e44 at
0x43472e and 0x434757, 0x43478e on the same object, 0x43a207 on 0x575950, and the frame-skip loop at
`[0x575c20 + 0x10]` against `[0x575a8b]` — measured running the update twice a frame. Whether
`prepare_frame` is where those belong or whether 妖々夢 wants a different loop is the design question
underneath, and it is the first thing to answer.

**Anything drawn**, because there is no `font.ttf` beside `th07.exe` to build an overlay from: 紅魔郷
ships one and 妖々夢 keeps its fonts inside `th07.dat`. So the status line, the wash and every menu of
orb's are out until a font comes from somewhere, and where it should come from is a question nobody has
been asked — Windows' own is what the drawing tests already use, and a font orb ships is a font orb
ships.

**The score file being forked for a game that cannot rewind.** A launch there wrote
`pointdevice_score.dat` on the way out, because `score::install` and `score::fork` are keyed on
`config.chapters` and not on the game: `Th07` declines every run feature, so nothing there can rewind
and its scores are runs anybody could have played, which belong in the game's own file. Nothing was
lost — `score.dat` came through both launches untouched — but a file appeared that should not have. The
fix is above the seam rather than in `Th07`, which is why it is a defect and not a `None`.

**And the run itself:** no chapters, no retry menu, no run picked up again, no card counted, no mode
question, no brush over the lives. Every one is a method of `Th07` answering `None` or nothing because
the exe has not been read for it, and each answer says what it costs where it is written. The two
questions parked at the top of this section — whether `State` says everything a chapter needs, and
whether the midstage table's shape holds — fall due at the first of these and not before.
`game/th06/chapters.rs`'s `MIDSTAGE` being seven stages because 紅魔郷 has seven is the one thing on the
old list that is still not a table.

**What no run has settled and no test can.** Every address in `orb-core/src/game/th07/mod.rs` other
than the two patched ones: a laid-out 妖々夢 is written from the same constants `Th07` reads, so a wrong
one is wrong on both sides at once. The device at 0x575958 and the window at 0x575c20 were used —
`screen: 1280x960 … client 1280x960` came out right, which wants both — and the chain, the two frame
calls, the viewport and the play area were not reached at all once `render` went to `None`.

## What a clear left open

The skip stopping at the staff roll and `--clear` reaching one are both measured —
`orb-e2e`'s `the_ending` and `orb-e2e`'s `a_clear_on_demand` hold what those runs showed. Three numbers
that clear did not settle:

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

## What picking a run up has not been through

A chapter has been written down, the game closed, and the chapter played back into place — the
landing agreeing with what was written down field for field, over two launches at 200740000ms and
200755453ms in the log. That two-process case is in *What only the real game can still answer* below,
and the check itself is `resume.rs`'s `a_landing_that_differs_says_which_field_it_was`. What it was,
though, is one chapter of stage 1 nine frames
in, driven by keys sent from another program. What is left is everything that is more than that.

**A chapter worth grinding, landing on every field — including the seed.** Chapters of 2009, 4597 and
3394 frames have been picked up now, and the landing check has moved the seed's write twice: out of the
frame before the stage, and out of the callback's own entry, which turned out to be 2048 draws early
because that callback fills a key table from the generator first — `GameManager::AddedCallback` draws
2048 times before it copies the seed, 0x41bc4f filling 64 records of 32 `u16` through
`Rng::GetRandomU16` (0x41e780), whose `seed = rotl16(((seed ^ 0x9630) - 0x6553), 2)` rewrites it every
draw, and 2048 draws from `0xc381` are `0x789c`. It is written on the way into `Stage::RegisterChain`
(0x4044c0) now; `resume.rs` carries the rest beside the write. The check itself now carries `rng=`,
having once agreed field for field with a seed
2048 draws out. **Nothing has been resumed since either fix**, so what is owed is one landing that
agrees with `rng=` among the fields it agrees on. The song's position, ignored in that same session and now decided by the song
rather than by the chapter's kind, is unwatched for the same reason: the chapter to hear it on is a
midboss's or a midstage one, and what should be heard is the phrase that was playing rather than the
track's first bar.

**The track's loop, on a resumed stage.** Putting the song back where the chapter had it worked and
then looped a section of itself near the end of the stage, which was the countdown the loop is taken
on being left where the file's old position had put it — moved with the file now, and
`orb-e2e`'s `the_music_across_a_restore`'s `a_sought_stream_keeps_its_countdown_and_still_takes_its_loop`
holds what was heard and the addresses it was read off. What is left is to hear a resumed stage's track run past that point and take its
loop where it should: the loop is minutes in, so it is the end of a resumed 道中 that shows it, and
`music: the track loops at …, so … byte(s) left from …` is the line to hold against how it sounds.

**Where the mark sits on the shot type select.** `中断データあり` is drawn there now, under the run the
cursor is on — see [SPEC.md](SPEC.md) for what it is for and why that screen. What has not been done is
look at it: the corner it is drawn in was chosen from the layout of the screen and not from a screen
with the line on it, so whether it collides with what the game draws, and whether the bottom left is
where the eye goes, are both open. It also says nothing on the character select, a screen earlier,
where the choice that loses a run is actually made — a mark there would have to stand for either shot
of that character, which is a different sentence.

**What the playback costs there.** Eight updates took no measurable time. A stage's worth is
thousands, each with `chapters.observe` behind it taking a snapshot at every boundary — and the
frame it all happens inside is one frame. `--pacing` says what that frame came to, and it will be
in the log as one enormous miss, which is worth seeing once so it is not read later as a fault.

**What writing it costs.** The whole file goes out every time a chapter begins, which in a fight is
every few seconds, on the same frame the chapter's snapshot is copied. A stage played to its boss
should be tens of kilobytes; whether that is a frame off the cadence is `--pacing` against the same
stage under `--no-resume`.

**Whether a converter is enough to read one by.** The file is MessagePack now, and printing it as
YAML takes a converter that knows nothing about the format — see [SPEC.md](SPEC.md). What that has not
been through is a session where something went wrong: every fault so far was found by reading the file
itself with `cat` beside the log, and whether one command in front of that changes how often anyone
looks is the thing to notice next time one is being chased.

**Whether the resumed run then plays as itself**, which nine frames cannot say much about: the
chapter's name and number on the status line, the retry count carrying on from the file's, the brush
over the lives, a deliberate miss putting the player back at that chapter, and the score on the
panel being the run's.

**The cases the sessions so far did not reach.** A resume of a resumed run, which is this machinery
a second time. A run of another character, which has its own file — two short runs settle that the
second does not write over the first. A practice run, which is not kept at all now: nothing should be
written, offered or marked for one, and what would say otherwise is a file appearing in the directory
while one is played. The Extra stage, stage 7 at difficulty 4. A run paused
mid-stage, where the claim that a paused frame is not in the record gets tested. A run given up and
*then* cleared in the same session, where the result screen has to take the file away. And a stage
longer than ten minutes of game time, the only way to reach the bound on how far a playback may run.

**What the sound does through a playback.** Thousands of updates queue their sound effects into the
game's own list inside one frame, and `PlaySounds` runs once at the end of it. Whether that is a
burst at the landing or nothing at all has not been heard — nine frames queue nothing. The music is
the stage's and then the boss's from their beginnings, which is what a chapter restore already does.

**Two things about the file itself.** Its `landing` line is `Reproduction`'s own text, so changing
what that line holds makes every file written before the change disagree at its first field, and the
version number does not catch it — the line is opaque to the reader. And a run written down by one
build and picked up by another whose midstage table differs lands on a chapter named and numbered by
the second build, which the landing check does not notice: what it compares is the reproduction, not
the name.

**And one thing the same reading changed elsewhere.** *No replay is offered for a pointdevice run*
stood in [SPEC.md](SPEC.md) on the reason that a rewound run does not play back. The game keeps its
record of inputs in the heap a snapshot covers, so it is rewound with everything else and each
attempt writes over the last from the chapter's own frame — which is the property orb's own record
leans on too. What would be saved is therefore the path that survived, playing back as a run nobody
could tell from a flawless one, and that is what `SPEC.md` says now. Settling it by measurement
rather than by reading means saving a replay from a normal-mode run and from a pointdevice run of
the same stage and watching what each plays back as.

## What a pointdevice score is

The scores of runs orb could rewind are kept apart from the game's now — `score.rs`'s
`the_game_score_file_is_forked_where_the_game_asked_for_it` and `orb-e2e`'s `the_score_file` — but that
only keeps them from being compared with runs somebody played. It
does not make them comparable with each other: a miss costs a rewind rather than a life, so the
number rewards grinding a chapter until it goes perfectly, which is a different game from the
one the ranking was built for.

Nothing here is broken, so there is nothing that has to be done. What would make the file worth
reading is the retries beside the score, since a clear with none of them and a clear with sixty
are not the same clear, and `RETRY` is already counted. That means orb's own format rather than
the game's, and with it the game's ranking screen no longer being where these are read.

## What the fixed stutter costs

Three to five frames of every six hundred used to come out three refreshes apart instead of two,
once every two or three seconds. The cause and the fix are beside `frame::Pacing::grid` and in
`orb-e2e`'s `pacing`'s `compose` section, and a replay
played back through stages 0 to 3 under `--log=quiet --pacing` settled what was still open about
them: 37,800 frames, three that missed their blank, 49 of 63 periods with nothing off the cadence
at all, and the compositor's drawing time converging to 2550µs and staying.

Two things that were open are answered by that run and are not questions any more. What the
compositor wants does not depend on the load — the same 2450–2550µs came out of a title screen
and out of a stage 3 boss fight with 524 bullets up — and the 3700–3850µs an earlier run showed
was the shaving random-walking upward for want of a floor, not a heavier stage needing more.

What is left:

- **What holding `budget_ceiling` under a refresh took away is the headroom for work that is heavy
  every frame, and a better fix would take nothing.** Swept at 120Hz with 2500µs allowed the covered
  work went from about 10ms to 5833µs — 5500µs a frame still holds the cadence, 6000µs runs at 40
  frames a second — and at 240Hz the same arithmetic leaves 1666µs against the millisecond the game
  actually takes. Nothing in play reaches it, which is why the ceiling was worth taking; what makes it
  a compromise rather than the answer is that the budget is the wrong place to enforce it.

  The invariant is on the *handover*: the frame must not be handed over before the blank before the one
  it is aimed at. Capping the budget only approximates that, because how early a frame really goes is
  the budget less that frame's own work, and the work is not known until it is done. Enforced where it
  belongs — a wait between the drawing and `PRESENT`, holding the frame until the earlier blank has
  gone — the budget needs no ceiling of its own and heavy frames stay covered. What that costs is a
  wait inside the frame and a rewrite of what *The work estimate* claims, so it is a decision for
  `docs/adr/` rather than an edit.

  `PLAY_SOUNDS` is what set the budget above a refresh: the game's own call, which orb makes between
  the update and the draw, ran 8438µs on the frame a spell card started and 12531µs at worst, four
  times in one session. That frame loses its blank either way — a budget is a prediction and a spike
  nothing has seen yet cannot be one — so what is left about it is only that orb chose *where* to call
  it. After `PRESENT` instead, the frame keeps its blank and the sound starts a frame later: sixteen
  milliseconds of audio latency against a lost refresh, with no measurement of either as a thing
  anybody notices.
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
- **Whether the grid's own way of running the game fast is gone**, which only a display that is
  not a whole multiple of 60 can reach — the same fault in the work estimate is the one that was found
  and measured, a 252ms `RunCalcChain` pinning the estimate to its ceiling and leaving `gaps in refreshes
  1x29 2x569` behind it. A grid moment left behind the blank in
  hand made the aim come out at one refresh a frame until the difference had been made up, and
  each of those frames is an update. It is dropped now and the arithmetic has a test, but no
  144Hz session has been through a stall shorter than four game frames, which is the only way
  in: beyond that the phase guard was already resetting the grid. What to watch is the same `1x`
  bucket, in a session whose log says `144Hz monitor is not a multiple of 60Hz`.
- **Which refresh rates have actually been run.** Three: 120Hz, 119.88Hz and 144Hz, all on the
  same machine and the same monitor at three settings — `600 frames, 16650us apart, gaps in refreshes
  2x600`, `16652us apart` at 59.94fps, and `16695us apart, gaps in refreshes 2x360 3x240` for 60.00. The third one
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
- **A second display, and a mixed-rate desktop — the setup is measured on real hardware now, and it
  is the ordinary one rather than an exotic one.** `scripts/compositor-probe.c` on a 120Hz primary
  with a 144Hz monitor beside it: the compositor reports 144.00Hz and its flushes come at 143.97Hz,
  and a window on any monitor gets that same 143.97Hz while `EnumDisplaySettingsW` answers about the
  monitor the window is on. The numbers are beside `frame::Pacing::grid` and at the top of
  `orb-sim/src/display.rs`.

  **The compositor followed the fastest monitor, not the primary and not the game's**, so "the game
  on the wrong monitor" is not what it takes: a 120Hz primary to play on and a 144Hz monitor attached
  is enough, and it is the configuration this machine was already in.

  **The game on it ran a fifth too fast, and that is fixed** — the cadence is counted in the
  compositor's own spacing now. The run that showed it is `13966us apart`, `14090us`, `13897us` and
  `13894us` over four periods of 600, 71 to 72 frames a second with `0 frame(s) paced by the clock`;
  `orb-e2e`'s `pacing`'s `disagrees` section is the fix asserted, and the fault is beside
  `frame::Pacing::grid`.

  What is left is a run on the machine to confirm it there. The simulator holds every second of every
  compositor rate at sixty, but it models one blank grid, so the thing it cannot speak for is the half
  this desktop actually has: a frame shown on the compositor's blank still has the window's own 120Hz
  panel to reach. What to watch in a `--log=quiet --pacing` run is `144Hz compositor is not a multiple
  of 60Hz`, the interval at 16666µs rather than 13888, and the buckets alternating `2x` and `3x`.

  Also still unmeasured is the rest of that bullet's table — the rates themselves, 240Hz and
  under-60Hz.

## A display under 60Hz is still paced by the clock

`adopt` takes the blanks only at or above 60Hz, and below it goes to the clock: there is no blank to put
a sixtieth of a second on, so one frame per blank would run a 50Hz display's game at 50 — seventeen per
cent slow, with the music to match — and the clock at least keeps the game's own speed.

Which is a choice rather than a limit, and the alternative has never been tried: a 50Hz grid could take
*two* blanks for some frames and one for others, the way the fractional path does above 60, and land
every frame on a blank at an average of 1.2. That would be 60 frames a second on a 50Hz panel, which
means five frames of every six shown for one refresh and one for two — judder by construction, but the
game at its own speed and every frame on a blank rather than on a clock that lands anywhere.

Nobody has a 50Hz desktop to want this on, which is why it is here and not done. The `rates` section of
`orb-e2e`'s `pacing` could reach it in a line.

## Move the rest of the suite onto the space

The mechanism is in — see *Reaching the game's memory with no game there* in [SPEC.md](SPEC.md) —
and `chapter.rs`'s tests are through it: every one drives the real `observe`, over a laid-out
game, with the snapshot really taken and `retry_chapter` really putting the memory back. The six
helpers that used to set `mark` and insert into `starts` in their own words are gone, and with
them the reason none of those tests could fail when `observe` changed. Breaking the production
`starts` insert now fails four of them; before, it failed none.

**`read_state` is read out of an image now, and `chapter.rs` still is not handed one.** The parse
itself is covered: `orb-e2e`'s `the_run_read_back` reads the whole `State` back off a game
that got where it is by being played — the game's own terms, over the same offset constants `Th06` reads
through — plus the four chases that have to come back as nothing rather than fault (the bosses pointer
before a fight, the dialogue index through a `GuiImpl` on the heap, the laser array, and the ending's
script). What is left is the
handover: `chapter.rs`'s scenarios still build a `State` by hand and step the detector with it, so
what they cannot catch is `observe` reading a different stage from the one the image holds.

That handover is covered now, once: `orb-e2e`'s `pointdevice_run` drives a run in which nothing hands
orb a `State` at all — the game advances its own memory and `read_state` is how orb learns it, as in
production — so a chapter beginning on the wrong frame, or on a card the memory does not hold, fails
there. What is left is that `chapter.rs`'s own twenty-seven still cannot fail for that reason, which
matters for the cases only they reach: the shortest a chapter may be, a boundary judged out, a bomb
swallowing a transition.

Laying one out also cost seven regions where the old image had five: the player is 0x98f0 bytes at
0x006ca628, the enemy manager sits at 0x004b79c8, and the bullet manager's laser array is 0xec000
into it, which puts laser zero at 0x00691ff8 and the array's tail inside the static data. Nothing
about that was guessed — each was found by a read panicking with the address it wanted.

**What the game that plays the game's part does not do yet**, each of which is a scenario somebody
could write next and none of which is in the way of the two that exist:

- **One stage, and never a second.** So the state between two of them — where the game tears one down
  and builds the next, and where a run's own numbers are put back from the stage before — is unvisited,
  and with it `jump_to_stage` and the terminator `stop_recording` drops during playback.
- ~~**No sound.**~~ Laid out now — [`orb_sim::Sound`](crates/orb-sim/src/sound.rs), the eight slots of
  `IDirectSoundBuffer` answered through the seam and a real `mmioSeek`/`mmioRead` pair — so a stage that
  streams one takes its first chapter at
  frame 8 rather than after the 248 `STAGE_BEGINS` names, and the whole of what a chapter does to the music
  is reached: the rewind byte for byte, the position a resume is picked up at, and the track a restore has
  to take down and start again. Which is `orb-e2e`'s `the_music_across_a_restore`'s six. A stage with
  *no* song is still the default, and still costs those 248 frames, because that is what every scenario
  which is not about the music wants.
- ~~**No replay and no demo**~~, so nothing reaches the tuning passes, the stepping keys, or the rule that
  a replay is a run orb tracks but does not offer a retry to. All three are reached now:
  `Fake::watches_a_replay_of_its_stages` is the record,
  `orb-e2e`'s `moving_between_a_replays_stages` moves between its stages, and
  `orb-e2e`'s `a_chapter_table_collected` runs a `--collect` and a `--judge` pass over one.
- **No ending, no practice run and no Extra**, and no `--clear`.
- **No boundary out of the *baked* midstage table.** Stage 1's one entry is script frame 4472 and its fake
  stage is over by 700, so what those runs exercise is the fight's own boundaries; the baked table's path
  is `chapter.rs`'s twenty-seven. A boundary a *pass* proposes is reached —
  `orb-e2e`'s `a_chapter_table_collected`, whose gap is at script 259 — since a proposal goes into the
  same list the baked one is read from.
- **One of the frame loop's four ways out.** The other three and the loop's order are the `frame_loop`
  section of `orb-e2e`'s `pacing`;
  a chain target that is null has no scenario, because `attach` and `attach_to` both fill those statics
  and nothing outside orb can empty them. See
  [docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
- ~~**A pad it has, and winmm's side of one it has not.**~~ Both now: `Image::no_controller` is the game
  whose enumeration found none, `orb_sim::Joystick` is the device winmm has, and
  `orb-e2e`'s `mode_on_a_winmm_pad` pushes it — over a seam of its own, `orb_api::joystick`, and a thread
  that carries the installation with it.

**No track plays in a laid-out game unless a scenario asks for one**, and a stage with none takes
`STAGE_SETTLE_FRAMES` *plus* `MUSIC_WAIT_FRAMES` to begin — the 248 frames `STAGE_BEGINS` names, in front
of every test that is not about the music. `Fake::plays_its_songs` is the numbers a track is told apart by
and `Fake::streams_its_song` is the whole of one, buffer and file handle included, which brings that down
to the settle alone. What the real game is still the only witness to is the *frames*: how long a chapter's
music takes to come up in a real launch, and what a listener hears across a restore.

**`self_check`'s inventory is skipped under a simulated Windows** — see `fingerprint_untracked`. It is a walk
of every private page in the process, which is a question about the process and not about the
game.

## The wait on the high-resolution timer has not paced a real frame

[docs/adr/0006](docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md) is the decision and it
is `accepted and built`: `wait_until` waits on a `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` waitable timer
made behind the seam on first use, `clock::wait` takes the counter's own ticks, nothing asks
`timeBeginPeriod` for anything and no `Sleep` is left in the tree, and every log line is stamped off
`QueryPerformanceCounter`. A host that cannot create the timer is turned away — by the launcher's
`can_make_the_timer` before it starts anything, and by `Pacing::no_timer` on the first wait for the case
the launcher was not the way in. `scripts/wait-probe.c` is the probe this waited on, the ADR carries what
it said, and `SPIN_US`'s own comment carries the histogram that keeps it at 1500.

**What is left is a run.** The probe measured the wait in a process of its own and `pacing_no_timer.rs`
covers the host that has none, but no frame has reached a screen through it. Three things a session would
settle and no scenario can:

- **that the timer can be made inside the *game's* process.** Two of the three places are covered: the
  launcher's `can_make_the_timer` runs on every launch, and `orb-api`'s own `real::clock::tests` create
  the timer and wait on it through the shipped code on this host. What is left is the DLL's, made from
  the frame hook of a process that has loaded d3d8 and dsound — and the answer to a failure there is a
  modal and an `ExitProcess` nothing has yet seen happen for real.
- **what the cadence comes out as**, against the real runs beside `frame::Pacing::grid` — `600 frames,
  16650us apart, gaps in refreshes 2x600` is what the wait being replaced produced at 120Hz, and it is
  the line to compare against. The pacing log's spans are the reading, and the one that used to be called
  `sleep` is now `wait`.
- **that the modal is legible over the game.** It is raised with no owner window and
  `MB_SYSTEMMODAL | MB_SETFOREGROUND` because the game is drawing through Direct3D, and whether that is
  enough is a claim about the host.

**The log's stamps changed on the real host and only there.** They were `GetTickCount`, which this
machine advances by 15 to 16ms whether or not the millisecond is asked for, and they are now exact —
which is what `log_deferral`'s reading of two stamps always assumed and never had. Nothing in the
simulator moves, because the simulator has derived its stamp from the counter all along.

**The suite got slower and then much faster.** The exact wait took `orb-e2e`'s `pacing` from 54.1 seconds
to 76.6 on its own, because the spin now runs the whole of `SPIN_US` where the old call's rounding down to
whole milliseconds left it about 1000µs to run — the simulator's own arithmetic, not a cost a real machine
pays, since a real wait overshoots into the spin and a real one now overshoots less. Then the spin's
`pause` went behind the seam and was charged for, which is
[docs/adr/0007](docs/adr/0007-the-spins-pause-is-behind-the-seam.md): **17.5 seconds for the file and
30.3 for the suite**, from 76.6 and 95.0, with all 297 passing and no assertion touched.

## A 240Hz display leaves the compositor less room than it may want

**Not a defect, and the reason it is here is that nothing says it is happening.** The allowance cannot
pass one refresh: a frame is handed over that long before the blank it is aimed at, so an allowance over
a refresh hands it over before the blank *before* that one, and the compositor takes it there — the frame
is shown a refresh early and the whole reckoning moves with it. `compose_ceiling` therefore stops at
three quarters of a refresh, and `frame.rs` has the 144Hz measurement of what going past it did:
`gaps in refreshes 1x418 2x179`, a hundred frames a second.

Three quarters of a refresh is 12500µs at 60Hz and **3124µs at 240Hz**. A compositor wanting 3200µs — the
figure the mixed-rate run on the machine had orb's own allowance chasing, on its way 2800 → 3400 → 3600 →
3900µs — is inside the geometry at every rate
anyone plays at except the fastest, and outside it there.

Measured, 240Hz with the compositor held at 3200µs over 20,000 frames: the allowance climbs to exactly
3124µs and stops, no miss is charged to anything, and the rate settles at **48.00** for the rest of the
run — every fifth frame taking an extra refresh. The `converges` section of `orb-e2e`'s `pacing` asserts
that shape, so a run reading 48 with the allowance *below* the ceiling would be a different fault and
would be caught.

What is left is the reporting. A run in this state looks, in the log, like a run whose compositor is
merely slow: the allowance sits at a number and the misses are counted, with nothing to say the number is
the most there is and no amount of waiting will change it. What that wants is a line at the point the
climb is refused — the allowance asked for more than the display has room for — which is a decision about
the log rather than about the pacing.

Nobody has seen this happen. It wants a 240Hz display and a compositor taking three quarters of a
refresh, and this machine has neither.

## Put Windows behind a seam, for the mechanisms a test cannot otherwise reach

**Running the suite on any host is off the table, and this is the measurement that took it off.**
`orb-core` does not compile for `x86_64-unknown-linux-gnu` and no seam over Windows will make it:
`Th06` calls the game's own functions through transmuted pointers, by the conventions MSVC6 compiled
them with — `extern "thiscall"` and `extern "fastcall"`, which exist on 32-bit x86 and nowhere else.
Not being Windows was never the whole of the requirement. Being x86 is the other half, and what is on
the far side of *that* is the game's own code rather than the host's.

So the prize the earlier version of this section named — a suite that runs anywhere, off the Windows
runner and the 32-bit mingw — was never on offer. The best available was ever "anywhere that targets
32-bit x86", which wants a toolchain about as particular as the mingw one already in place, for a DLL
that will never be loaded by anything but Windows. CI is one job, on Windows, and that is right.

**What the seam is worth is the other thing, and it is worth it one mechanism at a time.** A
mechanism whose behaviour turns on something a test cannot make Windows do is a mechanism nothing
tests. Four of those are covered now and none of them had a test before:

- **The log's deferral.** Which lines are held and which are written where they stand turns on
  whether the caller is the thread the frame loop claimed. With a real `GetCurrentThreadId` a test
  has no way to be two threads; with the clock behind the seam it can also say what a line's stamp
  should be. Four scenarios, including the giving-up path when no drain is coming.
- **Choosing 完全無欠モード.** What the mode question *draws* was already asserted against a recording
  device; what it *decides* is a function of `GetKeyboardState` and so was decided by hand. With the
  keyboard behind the seam a scenario presses the keys somebody would, and thirteen of them do —
  including the alt-tab that found a defect: everything reads as released while another window is in
  front, so a key held across the way back read as a fresh press and chose a mode. `input.rs` counts
  the first frame it is reading again as already held now.
- **How much of the screen the game gets.** Which monitor a window is laid out against is a size the
  host reports two different answers for — 2560x1440 before `SetProcessDPIAware` and 3840x2160 after —
  and how big a window has to be to hold a client of a chosen size is this machine's frame, 6x40.
  Neither is a number a test could move, so nothing held the layout to either. Six scenarios do now, and
  see *The window* below.
- **`Th06::read_state` out of a laid-out game.** Not about the host at all — see *Move the rest of
  the suite onto the space* above, which asked for it.

So the question to ask of each of the forty-five Win32 functions left is not whether it keeps the suite
off a Linux runner. It is whether there is behaviour behind it that no test can reach today. Where
the answer is no, leave it where it is.

**Where it stands, measured.** `orb-api` holds the seam: the `Win` trait, the neutral types, and the
facades for the game's memory, the clock and its timer, the display and its compositor, which keys are
down, which thread is running, which window is in front and how big it and its monitor are, the log
file and the loaded modules.
The distinct Win32 functions behind it are what a count of the `real/` modules names, which is where they
all are and the only place above `orb` that reaches `windows-sys` at all:

```sh
$ ls crates/orb-api/src/real/
```

The three most recent are three COM interfaces and a rasteriser rather than functions of Win32's own — the
eighteen slots of `IDirect3DDevice8` orb calls, the eight of `IDirectSoundBuffer`, and *bake this string at
this height* — which is
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md):
Windows reached through a pointer the game handed over is Windows a grep for `windows-sys` does not find.

`orb-core` holds everything that decides what happens to a run: the eleven hook bodies and the `Runtime`
they carry between frames, the chapters, the snapshots, the resume, the tuning, the drawing and the three
menus, the frame loop, and the whole of `game/` — `Game`, `State` and the two thousand three hundred lines
of `Th06`. **Not one of its files uses `windows-sys`, and nothing checks that by hand any more**: `cargo
xtask seam` builds it and `orb-sim` for a 32-bit host with no Windows on it, so an import that reached past
the seam fails to compile. `orb-sim` implements `Win` over an address space, a clock a test moves itself, a
display and compositor it declares, a keyboard it presses, a pad, a sound, the device orb draws through and
the strings it bakes.

What is left in `orb` is the injection, and every one of its nine files names `windows_sys` — which is the
boundary being where it says it is rather than where a grep happens to fall. Three of the launcher's four
name it too. The counts:

```sh
$ ls crates/orb/src/ && grep -rlc windows_sys crates/orb-core/src crates/orb-sim/src
```

**A hundred and fifty of the 356 tests are `orb-e2e`'s, and every one of them is driven by a game.** In
`crates/orb-e2e/src/`, where each is a `#[cfg(test)]` module beside the fake it drives,
a 紅魔郷 that plays the game's part drives them through orb's own hooks and through orb's own frame loop —
see *Running the game with no game there* in [SPEC.md](SPEC.md): a whole run in each mode, the question
that chooses between them on the keyboard and on a controller the game answers with, the whole of a
`State` read at frames a game reached by being played to them, the pacing over a display the scenario
declares, the window orb makes on a monitor it declares, the mark over the count of lives at both edges
of a run, a `--clear` through six stages with a bullet on the player throughout, which of the two
score files each of the game's own opens lands in, and a key another program sent reaching the game only
once orb has let its exclusive keyboard device go.

The frame loop was the last of those to arrive and cost one change, which
[docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) is: `render`
called `Th06::present` and `Th06::play_sounds`, the game's own code at 0x00420b50 and 0x00431270 where a
laid-out address space has nothing to execute — and each of those methods *was* an address and an
argument, so a `Game` that hands them over rather than making the call leaves a laid-out game two
functions of its own to answer with. The 414-line harness that composed a copy of that loop, and whose
own comment said it was a copy, is gone with them.

The four files left in `orb-sim/tests` are not scenarios, which is what their being `orb-sim`'s rather
than `orb-e2e`'s says. Three are the
log's own business — what `log!` formats and which level keeps which line — and the fourth is a host that
cannot create the timer every wait is made on, which is one wait and no frame. They want a process each, orb's log and
profile being process-global, which a file in `tests/` gives and `orb-core`'s own `#[cfg(test)]` — one
binary for all of its tests — does not.

**The boundary is enforced in CI**, which it was not: `cargo xtask seam` is a step of the workflow and a
line of `.husky/pre-commit`, and it checks `orb-core` and `orb-sim` for `i686-unknown-linux-gnu` — a host
with no Windows, and 32-bit because `Th06` calls the game's own methods by conventions that exist on x86
and nowhere else. It needs no linker for that target, never linking.

**What is still where it was**, with the question above asked of each: DirectInput's `EnumDevices` and
winmm's own enumeration for finding a pad; `RegisterClassA`, whose whole business is the black brush the
window class paints the letterbox with — the one exception
[docs/adr/0010](docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md)
names, a hook body being allowed the one call its rewrite consists of; and the two winmm functions a track
is moved through, which are found by name in the game's own copy of the library and called through a
transmuted address rather than through the seam.

The three that were the head of that list are done: the heap walk is `orb-api`'s `real/mem.rs` beside the
two walks it already did and answers `mem::game_regions` itself; the status line's GDI is two seam
functions and its layout is `orb-core`'s; and the joystick's sampling thread is `orb-core`'s, its `Sleep`
and `SetThreadPriority` having become `clock::sleep` and `thread::below_normal`. With them
`crates/orb-e2e` names no `orb` at all and the `rlib` beside the DLL is gone, which is what
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md)'s
step 8 expected — and nothing is handed from `orb-core` back into `orb` any more, the `Present` slot's
`VirtualProtect` being `mem::replace_word` now.

**The window — done, and it is the fourth mechanism the seam was worth cutting for.** What decides how
much of the screen the game gets is two numbers only the host knows, and neither could be moved by a
test: `SetProcessDPIAware`, `MonitorFromPoint`, `GetMonitorInfoW`, `AdjustWindowRect` and
`GetClientRect` are behind `orb_api::window` now, and `orb_sim::Windows` is a monitor with two sizes —
3840x2160 that reads as 2560x1440 until the process asks otherwise — and a 6x40 frame charged on the way
in and given back on the way out. Six scenarios in `orb-e2e`'s `the_window` drive it, through the game's
own `CreateWindowExA` call: `Originals` gained a third handed-over function for that, the same argument
[docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) made about the
frame loop's two. What they hold that the arithmetic could not is which side of `SetProcessDPIAware` each
monitor read fell on — `orb_sim::Windows::monitor_reads` writes that down — and the client a window of
the size asked for really comes out with.

**The pacing — done, and it found the thing it was predicted to find.** `frame.rs` is in `orb-core`
and its host calls are behind the seam: the counter, the wait to a frame's deadline, the monitor's rate,
`DwmGetCompositionTimingInfo` as one value, and `DwmFlush`. The simulated compositor holds a refresh
period, a compose time a test may change mid-run, and a `DwmFlush` that returns at the blank the
frame just handed over reached — modelled that way because that is what the real one does and the
whole of how the pacing knows whether a frame made its blank.

Every one of `orb-e2e`'s `pacing`'s sixty-three scenarios drives the loop — `render` itself, called by a
game's own loop — where its own
thirteen tests were all arithmetic and **not one of them drove it**: the whole-multiple cadence, the rates
a display reports and the rate each gets, the fractional 2-2-3-2-3 pattern, the budget rising after a frame
overran, a compositor that spikes and the allowance following it, what that spike costs at each rate, three
stage loads and what they must not buy the compositor, the clock path, a host that cannot make the timer
every wait is on, the
mixed-rate desktop, sixty frames a second for every compose time the pacing has room to cover — which is
the one that found the deadlock in `measure_compose`, now fixed — and the loop's own shape: the update
before the draw, the sounds between them, the chain's two exits becoming the frame's two, and the ways out
that hand the frame back to the game's own loop.

**And a row of a table is a `#[test]`, because that is the unit the harness parallelises.** Measured on
sixteen cores: the file is nearly ten minutes of work and came out at **126 seconds**, of which **121.6 was
one test** looping forty rate-and-compose-time pairs in a process of its own — 96% of the wall clock, and a
floor no number of cores goes below. A test per rate instead, through `a_test_per_rate!`, put the same work
in **54.8 seconds** and the whole suite from 135 to 72. Nothing below about 37 seconds is reachable by
splitting further, that being the work over the cores, so the rows are split as far as the reading of a
failure wants and no further.

The file is **17.5 seconds and the suite 30.3** now, from the same rows, and the split is no longer what
decides it: the exact wait took the file to 76.6, and then charging the spin's `pause` behind the seam —
[docs/adr/0007](docs/adr/0007-the-spins-pause-is-behind-the-seam.md) — took a hundredfold off the spin's
iterations. Which also moves where the floor is: the work is no longer nearly all spin, so what splitting
further would buy has to be measured again before anybody does it.

And one scenario that is a negative: `cFramesLate` reaches no decision. The same frames are run twice against two
hosts that differ in that one answer and in nothing else — `orb_sim::Display::says_every_composition_was_late`
is the second — and every number orb decided comes out the same, the allowance among them. Which is what
the measurement needed: the real one read `0 shown late` through runs where 57 frames of 600 missed their
blank, so the number is reported and nothing rests on it.

**And what orb says about the rate is held against it, which is the half nothing asserted before.** Every
number a scenario reads of the pacing it reads out of the log: the `frame:` line's count of frames, the
interval, how many the compositor could not show when it meant to and the histogram of gaps in refreshes,
and beside them what the compositor is being given. That line is what somebody reading a real run's log
has, so a run whose rate was right and whose account of it was wrong used to pass. It cost the scenarios
one thing — the lines are written once per `profile::INTERVAL` frames and drained on the far side of the
*next* frame's flush, so a scenario that wants one runs frames until it is there.

**The simulator is deliberately non-deterministic, and that is the point.** The OS is, from the
application's side: it wakes a thread when it gets round to it and its compositor is slow now and then.
So the host draws its delays from a seeded stream, a scenario is held against several seeds, and the
seed goes in every assertion so a failure can be replayed. A scenario that holds for one seed and not
another has found something a real machine can do, which is a defect and not a flake.

Which means the assertions are about the *rate* and not about ticks. What they ask is what somebody
playing would ask: **what share of the seconds were sixty frames a second**, judged within half a
frame, from a few seconds in. A display the pacing accepts holds every one of them; a mixed-rate
desktop holds none.

Two things the simulator needed that nothing else has:

- **Spinning has to cost time.** `wait_until` waits most of the way to its deadline and spins the last
  1500µs, so a clock that moved only when something asked it to wait never reached the deadline and the
  frame loop hung. Both halves of that loop are charged for now: the counter read at one tick, which is
  the smallest step it has, and the `pause` at `PAUSE_TICKS` — which is why the spin's instruction went
  behind the seam, and it is the one thing there that is not a call into Windows. See
  [docs/adr/0007](docs/adr/0007-the-spins-pause-is-behind-the-seam.md).
- **The distributions have to be measured, not chosen, and the rate matters more than the range.** The
  wake delay was modelled as spread evenly over its measured range and made a tenth of the frames miss;
  the histogram says 81% of returns are within 100µs and the rest are single excursions. The compose
  spike was set at nine frames in six hundred, taken from a mixed-rate run's own miss accounting — but
  that run was mispaced and those misses were the pacing's, not the compositor's. Half an hour of play
  reaching 3.5ms puts it nearer one frame in a hundred thousand, three orders of magnitude rarer, and
  with the wrong figure the simulated 60Hz display lost two seconds in five that a real one holds.

**The drawing, and the font two halves of the suite disagreed about — closed.** It was a `Device` being a
pointer to a vtable, so `recording.rs` was one and needed no seam; `says` baked each string a second time
through the same font and held the two against each other, and the four UI modules did it on
`C:\Windows\Fonts\arial.ttf` while the scenarios did it on the `font.ttf` beside the game. Both halves are
gone: the device is eighteen slots of the seam and the record is `orb_sim::Recording`, a bake is
[`orb_sim::Glyphs`](crates/orb-sim/src/text.rs) answering from a metric a scenario declares, and a mask
carries which string it came from — so `says` reads the answer out rather than baking a comparison, and no
test depends on which fonts the machine happens to have. See
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).

What is left of it is one thing rather than three: **nothing anywhere holds the real rasterising to
anything.** The suite answers a bake from a declared metric, so what a string comes out as on the machine
is something only a launch says — `overlay: font.ttf loaded, GDI is using …` is that line, and it is in
the list of what only the real game can still answer. The status line's own placement in `window.rs` is
untested for the same reason and by the same route: it is GDI on the window, above the letterbox, handed
over from a hook body rather than driven by one.

**The regions and the copies** — `memtrack`, `snapshot` — where the cases worth having are a page
freed between a snapshot and the restore of it, which a laid-out address space can make and a real
process cannot be asked to.

**Holding the game still** is the one where the argument holds: what is relied on is what
`SuspendThread` does, and that is checked against threads a test makes and registers itself. The
enumeration around it could go behind the seam; the suspending should not.

**What is next, and why each one is behind the one before it.** The end-to-end suite the rest of this
was aimed at exists now — a game that plays the game's part, driving a whole run of each mode through
orb's own hooks — so what is left below is not what makes a run testable but what would let the crates
above the seam be checked on a host that is not Windows. `chapter.rs` is still the prize among them,
its twenty-seven scenarios driving the real `observe` over a game laid out by hand, and it is behind
three modules:

- `memtrack.rs` for the regions a snapshot covers, which is a walk of the process's heaps.
- `threads.rs` for holding the game still, which is `OpenThread`, `SuspendThread` and `ResumeThread`
  over the threads the game made. The rule it exists for is checked against real threads today and
  should stay that way — a test makes threads of its own and registers them — so what goes behind
  the seam is the enumeration, not the suspending.
- `snapshot.rs` for the copies, which needs both of those and `VirtualQuery` besides.

`hook.rs` is behind them in turn — `memtrack` and `threads` both patch the game's import table — and
it needs exactly one seam function that is not there yet: `FlushInstructionCache`. `VirtualProtect`
it already has.

**`joystick.rs` is the one that can move on its own**, and a trap in it is worth writing down before
somebody finds it the hard way. `joystick::calibrate` copies the bytes of a `JOYCAPSA` straight into
the game's memory, and `const _: () = assert!(size_of::<JOYCAPSA>() == 0x194)` is what holds it to
the struct the game passes 0x194 for. A plain-data mirror in `orb-api` has to have that layout
exactly, so it must keep that assert *and* gain a `#[cfg(windows)]` one against `windows-sys`' own —
which is a better check than the one there now, since today nothing says the two agree.

~~**`recording.rs` stayed in `orb`** while the vtable declarations went to `orb-core`.~~ The drawing seam
is cut and it is `crates/orb-sim/src/drawing.rs`, the simulated host's answer to the eighteen slots — no
vtable of Rust functions, no `Screen`, and no second bake to recognise a string by. The declarations went
with it, to `orb-api`. See
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).

**Three surfaces are not functions at all**, and they are still ahead: Direct3D 8 and DirectSound
reached through COM vtables, the window procedure and its message pump, and the game's own code
called through transmuted pointers.

**One thing needed no seam, one looked as though it needed none, and the third only looked that way.**
Holding the game still needed none, because the threads it stops are ones a test can make and register
itself. The Direct3D device *read* as needing none — it is a pointer to a vtable, and a vtable of Rust
functions is a device — and that argument turned out to be why the tests worked rather than why the seam was
unnecessary: the same sentence would retire `orb_api::mem`, and what it left behind was `overlay.rs` calling
Windows fifteen times while naming no `windows-sys`. It is eighteen slots of the seam now, and so are the
eight of the sound buffer and the bake of a string. See
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md). The third was the clock, and the argument for leaving it — that every decision the pacing
makes is already a function of numbers, so a seam would only add the order the waiting calls come in —
does not survive being checked: the order *is* what is untested, `agrees` is decided from two numbers
a sim can declare, and the measurements beside `frame::Pacing::grid` are of frames landing on blanks, which is a different
question. Reading the clock went behind the seam for the log's sake in any case, since a test that
cannot say what time it is cannot assert on a stamped line.

So the lesson for the crate split is not the one this section was started for. The seam is worth
cutting where a *test* cannot otherwise get at the behaviour — not where a Win32 call happens to be,
and not to get off a Windows runner, which was never reachable and would have bought nothing.

**Not yet run in the game.** What the split has been held to is the suite and the lints: **all 228 tests
passing on `i686-pc-windows-gnu`** — every one that passed before it, plus the forty-one scenarios,
with nothing ignored — and with
`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean. Its rustdoc links too, since
the split moved items out from under four doc comments that named them. `orb-api`, `orb-core` and
`orb-sim` also *check*
clean for `i686-unknown-linux-gnu`, which is the only evidence that the boundary holds — the crates
above the seam name nothing that is Windows. Running the suite there is a different matter and not one
worth arranging: it wants a 32-bit Linux toolchain on top of the mingw one, for a DLL Windows is the
only thing that will ever load.

Nobody has launched 東方紅魔郷 with the DLL since. The paths most worth watching when somebody does
are the three the seam rewrote out from under a caller: the log, which now reaches `CreateFileW`
through `orb_api::logfile` from inside `DllMain` while the loader lock is held; `crash.rs`'s
module-and-offset line, which now asks `orb_api::module::path` for a handle rather than
`log::module_path`; and `Game::pad`, which is handed orb's own last joystick sample now instead of
fetching it. None of them has a test that means anything without a real process.

**And the one place a simulated Windows is still the answer** is the window and its message pump:
`IN_HOOK` exists because a Win32 call that moves a window dispatches messages synchronously and the
game draws from its window proc, so a hook can be entered again from inside itself. Nothing tests
that, and nothing can until there is a message pump to drive.

## Draw a frame in a test and say what is on it

**The device is done** — `crates/orb-sim/src/drawing.rs`, which is the simulated host's answer to the
eighteen slots of the game's own. It keeps the quads with their rectangles, colours and which texture each
went through, the clears, and the viewports, and `Quad::covers`/`overlaps` are how "the mark covers the row"
and "it leaves the row below alone" get asked. Five tests hold it to that. It was a vtable of Rust functions
in `orb/recording.rs`, on the argument that a `Device` is a pointer to a vtable — which is why the tests
worked and not why the seam was unnecessary; see
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).

**The font is answered.** `Overlay::new` loads the game's own `font.ttf` through
`AddFontResourceExW`, which wants a real file — but it does not want *that* file. Windows has fonts
of its own, `AddFontResourceExW` takes any of their paths, and `Font::load` already survives GDI
substituting a face, which is what its own log line is about. `GetWindowsDirectoryW` joined with
`Fonts\arial.ttf` builds an overlay in a test, and Arial is not machine-specific in the way a game
directory is: it has shipped with every Windows there has been. What it costs is the glyph metrics —
they are Arial's, not the game's — so a test may ask where the drawing put things and not how wide
a word came out.

`lives_ui` is through it. Two tests over the recorded frame say that something covers every part of
the count's row and that nothing reaches the bombs eight pixels below or the score above, and that
what covers the row is a picture rather than a flat fill — through the white texel the same opaque
ink would be a patch over the row, which is the one thing the mark must not look like. Shifting the
stroke by 24 pixels in `draw` fails both; the five geometry tests beside them do not notice, because
`brush_area` is right in all of them and it is `draw` that is wrong.

**The three menus are through it too**, and so is reading text back: `retry_ui` has ten tests over the
recorded frame, `resume_ui` eleven, `mode_ui` three, and the scenarios over a whole run ask what a quad
*says* — `orb_sim::Recording::says`, which reads which string a texture's mask was baked from, so the
retry menu naming the chapter it is offering is something a test can read. What is left of the drawing is
the rasterising itself: a bake here is a declared metric and the real glyph boxes are the GDI's, so `says`
tells one string from another and nothing tells one metric from a wrong one.

**What nothing has ever read is the status line.** The chapter's name, `RETRY n`, `INPUT LAG`, `COMPOSE`
and the frame rate, in the black beside the game — and every pacing measurement on the machine was read
off that `fps` and the log's own line. It cannot be reached in a test: `window::write_beside` wants a
window it can `GetClientRect` and returns without one. The seam that would open it is at the level of
the *lines* rather than the GDI — orb tells the host to put these lines beside the game, and a simulated
host keeps them — which is small, and would leave the placement arithmetic where it is. Worth knowing
what it is not: `interval_us` is an average over 32 frames held for 30 more, so the number on screen
says a run settled at sixty and cannot see the second that lost four frames. The rate is the clock's to
answer — see [docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).

Two things were learnt on the way and both are traps worth keeping:

- **A `Label` keeps the texture it baked** and hands it to the next frame asking for the same text.
  So a menu drawn against a *second* device draws the first device's textures — a use-after-free
  rather than a wrong picture. In a run there is one device and the overlay is rebuilt with it when
  it is lost, so only a test can arrange this. A harness for these should make one device per test
  and draw every frame of that test on it.
- **A recording and a guard beside it is the wrong shape.** The guard has to be a field of the
  recording, dropped after everything it protects: returned as a second value it drops first, and
  the next test then clears the textures this one is still releasing.

**One thing found while looking for a way to gate a mutation check, worth writing down before
anything else is designed around it:** the tests are Windows binaries run through WSL interop, and
**an environment variable set for `cargo test` does not reach them**. So a test cannot be
switched by the environment here, however natural that is everywhere else in this repository.

## What only the real game can still answer

The suite is 356 tests and **no stubs**, and most of what it now covers used to need a session. So this
is the list that is left — what a run on 東方紅魔郷 1.02h is still the only witness to, and therefore
what a session is for. Nothing here is a test somebody could write.

**And [docs/adr/0010](docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md)
added one file of that kind: `orb-api/src/real/window.rs`.** The status line's whole drawing path moved
into it — the font measured, the bar cleared, the text written and blitted — and what a scenario now holds
is the layout above it: which of the two bars the lines went in, at which em height, where the block
landed, and a shorter stack afterwards clearing the rows the longer one wrote in, all in `orb-e2e`'s
`the_window`. What no test reaches is the blit itself, and **no log line says it happened**: orb writes one
only where the text had to be made smaller or where there was no black to write in, so silence is a bar
written and a bar silently unwritten alike. **All three shapes of that black have now been through a
session** — the bar down the side, the bar under the game, and the client that leaves none — so what is
below is what each one said, and the one sub-case a scenario should take over.

**The bar down the side is confirmed on this machine** — `screen: fullscreen` on the 3840x2160 panel, a
2880x2160 game with 480 pixels of black either side, the lines left-aligned a margin inside the right-hand
one at 30 pixels of em with no line long enough to be reduced (472 pixels of room, and no `writing at Npx`
in the log). Watched drawn and readable through a played stage, chapters and the retry menu.

**And the 4:3 client that leaves no black at all is confirmed**, which is the one shape of this a log line
settles on its own: `screen: 640x480 — window at 1597,820 sized 646x520, client 640x480` and then
`screen: client 640x480, game 640x480 at 0,0 — no black to write in`, once and not per frame. Once that
line is written `write_beside` returns before it reaches the seam, so the line *is* the evidence that
nothing was drawn over the game — the only case here that needs no eye.

**And the bar under the game is confirmed**, which was the third shape: `screen: 1600x1400` — `window at
1117,360 sized 1606x1440, client 1600x1400`, a 1600x1200 game at y 100 and the black the 100 rows from
1300 down. Three lines at the title menu, right-aligned on the game's own right edge at x 1600 with the
lowest one's bottom at 1392, which puts the block at 1302 and leaves **two rows** between it and the game.
Watched: entirely in the black, nothing over the game. No `writing at Npx` either, the strip being the
client's full 1600 wide — what forces a smaller font is a narrow bar and not a short one.

**Three lines and not five, and the five-line case is a scenario rather than a session.** `write_status`
pushes a chapter's name and its retry count only while a run is being chaptered, so the title menu is three
lines — 90 pixels into 92, the tightest the arithmetic gets, and it fits. A run makes it five, which is 150
pixels of stack in 92 rows of black, and `write_beside` holds the block's top at `letterbox.bottom` so the
lines above run off the top of the bitmap and are clipped there rather than drawn over the game. That is
what a run plays with and what the session above did not show, so it is
`the_window`'s `a_stack_taller_than_the_black_under_the_game_starts_at_the_games_edge`: five lines in that
same client, the unclamped top asserted to be inside the game first so the scenario cannot stop being about
the clamp, and `bar.area.top` then the game's own bottom edge.

**The launcher cannot ask for this shape**, which is worth knowing next to it: `launcher/settings.rs`'s
`sizes` offers 16:9, then 4:3, then 640x480 — every one either leaves the black down the sides or leaves
none — so it took `orb.yaml` written by hand with a taller-than-4:3 `screen` **and `ask_at_startup: false`**,
the dialog dropping a size its own list has not got and falling back to fullscreen. Nobody playing can
select it, so no run will find a fault here for us.

**What [docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md)
built and the suite cannot witness.** Everything the scenarios drive is now above the seam, which means the
code that actually talks to Direct3D, DirectSound and the GDI is in three files nothing but a launch
reaches: `orb-api/src/real/d3d8.rs`, `real/dsound.rs` and `real/text.rs`. They are the only callers of a
COM vtable or a GDI call left in the tree, and every call site above them was rewritten — 479 lines of
`overlay.rs` and 468 of `audio.rs` — with no compiler checking the rewrite the way it checks an import
move. **All four have now been through a session**, and what each one's instrument said is here rather than
in a list of what to do again — the point of the list was the first launch after the move, and it happened:

- **The overlay draws and the game's own scene does not draw wrong** — watched, and normal. The state block
  round every draw is what that rests on: `overlay.rs`'s `frame` sets thirteen render states and twelve
  texture stage states, plus the viewport and the FVF, and 紅魔郷 sets its own once and assumes they stay
  set. So the failure to watch for was never orb's own text going missing but the game's scene coming out
  wrong behind it — the blending of bullets, the draw order with `D3DRS_ZENABLE` left off, the scene drawn
  into orb's 640x480 viewport, sprites drawn with orb's texture still bound to stage 0. Every one of those
  is gross enough to see at a glance; the only quiet one would be `D3DTEXF_LINEAR` left on, which softens
  the game's own pixels.
- **The letterbox still works** — `screen: presenting through a letterbox, client 3840x2160`, which is the
  `Present` slot patched through a device read as `orb_api::mem::read(device.0)` rather than dereferenced. A
  stretched picture is what its absence looks like, and the status line beside the game needs that black to
  exist at all, so the two were confirmed together.
- **A chapter's music comes back from where the chapter began** — heard. **The `music:` lines are Verbose**,
  one `detail!` at `audio.rs:279`, so a session at `log_level=normal` has none of them and their absence is
  not a signal: the ear is the instrument at that level.
- **The glyphs are the game's own font again** — `overlay: font.ttf loaded, GDI is using Some("Rounded M+ 2p
  regular")`. The suite answers a bake from a declared metric, so nothing in it has read a real `font.ttf`
  since, and a substituted face is what that line exists to catch.

**The last twelve stubs were un-stubbed by growing the laid-out game and the seam**, the way the
seventeen before them were: an ending object reached through the chain job it registers, with a `.end`
whose waits run out inside one frame and a staff roll it hands over to as the track changes; a replay
manager whose per-stage records a teardown would write over, whose entries the player moves on, and whose
seed a stage reached by moving is drawn from; a screen shake registered as a job of the chain's, with the
game's own `Chain::Cut` handed over to take it down; the front end's own eight items, drawn, with `Extra
Start` among them only where the score file's `clrd` left something to light it from; and a track streamed
through a buffer the seam answers and a real `mmioSeek`/`mmioRead` pair —
[`orb_sim::Sound`](crates/orb-sim/src/sound.rs).

**Two of those carry a claim the laid-out game narrows**, and both say so where they are written:

- `the_ending_and_the_roll_together_come_to_the_waits_in_the_script` asserts the arithmetic over an
  ending whose waits are known — an ending orb can find no script in is run out to the scene change,
  roll and all, and the difference between that and stopping at the roll is the roll's own script. The
  **62 frames** between the real run's 7,892 and `staff00.end`'s 7,830 are still unaccounted for, and a
  scenario that accounted for them would be inventing the account.
- `a_sought_stream_keeps_its_countdown_and_still_takes_its_loop` asserts that the file's position and the
  countdown still name the same loop point after a seek, which is the arithmetic the episode was the
  failure of. **The track has not been re-heard** since the countdown was moved with the file.

And two more things a laid-out game reaches the edge of, which the scenarios name rather than assert:

- **The roll's drawn frames are fewer than its own script's waits**, because a run that ended waits for
  the trip through the ranking and a trip that finds no front end up spends its whole allowance of
  updates inside one frame. Whether that is part of what left the real roll **544 frames short** of its
  7,830 is not settled.
- ~~**A track taken down and started again** is `StopBGM` and `PlayAudio` at their own addresses, and
  there is no code at either in a game laid out by hand.~~ Both are handed over now, the way `Chain::Cut`
  is — see [docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) — so
  `restore: the track has changed since this snapshot` and the two lines after it are
  `a_chapter_whose_track_has_gone_is_restored_by_taking_the_music_down_and_starting_it_again`. What is
  still the real game's is what the *game* does inside those two calls: its allocator seeing the stream
  freed, and `PlayAudio` finding the `.wav` and the `.pos` beside the path it was handed.

**What the calls handed over do inside the game.** Nine of them now — see
[docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) — and every one is
a constant in the shipped build with the handover behind the `sim` feature, so what a scenario drives is
the path and never the code at the far end of it. Three are new since the last launch and worth watching
on the next: `GameWindow::Create`, where the display setting orb overrules is read a few instructions
later; `StopBGM` and `PlayAudio`, where a restore's `music: stopped through the game` and `music:
restarting …` are the lines to look for; and the game's own `joyGetPosEx`, whose entry orb still patches
for real. The `JOYCAPSA` orb writes into 0x69d760 goes through `orb_api::mem` rather than a raw
`copy_nonoverlapping` now, which is the same volatile write in a shipped launch and a different one only
under a simulated host.

**The frame accounting the fake's fidelity pass moved, and the fourteen fixes behind it.** The laid-out game
is faithful to the game orb is injected into now — see
[docs/adr/0008](docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md), which is a description
of the tree — and every one of those fixes was read out of a decompilation of the same binary rather than off
a run. **Two of them move when a frame happens**, and only the real game says whether they moved it the right
way:

- **A scene's first update is on the frame it was built**, and the input word is zeroed there. Every
  transition therefore lands a frame earlier than it used to — the front end's, a stage's, a stage
  transition's, the result screen's. `orb-e2e`'s `the_frame_a_scene_is_built_on` is the claim; what a session
  would say is whether the game's own `f<N> scene=` lines fall where the fake now puts them.
- **A stage begins with 240 updates nothing can kill the player in**, not 120 and not none. That is
  `Player::OnUpdate`'s spawning branch replacing the count `Player::AddedCallback` wrote, read out of the
  decompilation and agreeing with `PLAYER_INVULNERABLE_FRAMES`, which was measured on the real game from the
  other end. A run that stands still into a bullet at a stage's start is what would say so directly.

**And one global orb reads that has no name.** `CURRENT_CARD` at 0x005a5f98 is in no `globals.csv` row, and
the exe touches it only from inside `EclManager::RunEcl` — the interpreter that runs a spell card
declaration, so the provenance agrees with what orb reads it as. That it *is* the current card is not
confirmed by name. A read of the ECL instruction or a run with a card up is what would confirm it.

**A pre-existing intermittent in the suite, and it is not the fidelity pass's.** One scenario of
`orb-e2e`'s `the_music_across_a_restore` fails perhaps one run in eight with `no sound has been installed on
this thread` — `orb_sim::Sound`'s `STREAMING` reading null where the sound was installed on the thread that
asked for it. Held against `HEAD` before any of the fidelity work: it flakes there too, at about the same
rate, so it is older than that work and nothing in it. What is *fixed* is the other one that used to sit
beside it: an allocation landing inside a range the laid-out game claims, which failed two runs in three
until `Sound::of` began asking `Space::has_room` and allocating again — the addresses it was watched at are
written down there. This one is still to find; the panic crosses an `extern` frame and aborts, so what it
needs first is a run that catches it under a debugger or a `STREAMING` that says which thread it was set on.

**Addresses and the bytes at them.** Every offset in `game/th06/` is written into a laid-out space
by the same constant the reader reads it with, so a wrong one is wrong on both sides at once. The
prologue bytes a hook is installed over, the exe's md5, and that the jmp lands where it should are
the same kind of claim. Only the real image says.

**That the hooks hold.** Installing a trampoline over a prologue and having the game carry on
through it is a property of that image and that compiler's code, not of the installation logic.

**Which game a launch matched, and the two log lines that come of it.** `attach_to` is handed its
game, as a scenario's is, so `host_exe()`'s lookup against `orb_core::game::KNOWN` — the build a run's
addresses were read off, and the list of games in a process that is none of them — runs only in a real
launch. **東方紅魔郷 has not been launched with the DLL since that table went in.** The table's own
arithmetic has five tests; the match has none and cannot.

**A chapter picked up in a second process.** One launch writes the file and another reads it, which
is two processes of the game — a scenario is one, and `orb-e2e`'s `pointdevice_run` gives the run up
inside its own launch instead, so what it reads is the file and only the file. That the file survives
the game being gone, and that a launch finds and names it, are the real thing's to show.

**The pacing, as frames on a screen.** The arithmetic is covered; what is not, and cannot be, is
whether frames land on blanks. The 120Hz, 119.88Hz and 144Hz runs are beside `frame::Pacing::grid`
with the probes that took them, and they stay measurements.

**What else on the desktop does to the window.** A borderless tool on this machine acted on
`東方紅魔郷.exe`'s window creation and resized the client to 2880x2160 three and a half seconds after
orb's last say in it. Nothing in the suite has another program in it.

**The launcher's dialog answered on a pad.** `launcher/pad.rs` has the reading and `settings.rs` the
dialog's own arithmetic; a Win32 dialog with a real pad pushed at it is neither.

**The sound, as sound.** A track does play in a laid-out game now — `orb_sim::Sound` is the buffer
`Music::capture` locks and the winmm it seeks through — so which chapters put their music back, the
position a chapter is written down with, and where the countdown lands after a seek are all covered. What
is not, and cannot be, is what any of it sounds like: whether a chapter comes back with its music is a
thing to hear, and the streaming margin is a real thread topping up a real buffer against a real card.

**Anything the screen is the judge of.** The drawing tests say where a quad went and in what colour;
they do not say it looks right. The brush stroke reading as a stroke, the panel's own tile showing
through the strips either side of it, the wash being dark enough to read a menu over and light enough
to see the run under — those are looked at.

**And the front end's own answers.** What its spell card history shows in each mode, and which items the
game's menu lights beyond the one the score file's `clrd` decides — the log cannot see a menu, so what a
scenario reads is what the laid-out front end draws, and it draws eight names and no artwork.

**The driver for such a session is not in the tree, on purpose.** `.gitignore` keeps `/scripts` out
because what wraps the build and the copy is one person's workflow and the last one carried that
person's machine in it. A session driver is the same shape — it needs a game directory, and it would
be asserting against a log nobody else's run produces. `--take-sent-keys` exists so that one can be
written outside the tree: it makes the game read its keyboard the way it does when DirectInput has no
device, which is the only way keys sent with `SendInput` are seen at all.

## Confirm `self_check`

It should report zero saved regions failing to restore, and no untracked region changing outside
the process heap. It pauses the game for as long as fingerprinting every private page takes,
which is why running it is a deliberate session rather than something left on.

## Play a stage with a pad

The read is orb's thread's now and the frame pays a copy, and a pad that turned up mid-run drove
the menus — `joystick.rs` carries both, the timings and the run that watched a pad wake mid-run. What no
run has been through is a stage: shot and bomb under
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
