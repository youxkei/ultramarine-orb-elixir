# To do

## What a played run still has to show

A `--clear` run has been through stages 1 to 6, the ending and the result screen with chapters
taken and no replay offered — see [DONE.md](DONE.md). What that run cannot show is anything that
needs somebody to be hittable, or a mode nobody chose.

**The retry menu on the keyboard.** Every item of it, both confirmations, and both ways of refusing
one are measured — see [DONE.md](DONE.md) — and every one of them was answered on the pad. The
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
being cancelled without the screen moving are in [DONE.md](DONE.md), on the pad. What that sitting did
not reach:

- **`はじめから` through this path.** `つづきから` is in [DONE.md](DONE.md) — the press handed back, the
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

**The rest of what the pad now reaches.** The mode question answers on it — see
[DONE.md](DONE.md) — which leaves three things that go through the same reading and have not been
pushed. The retry menu: up and down on the stick and on the d-pad, shoot deciding, bomb or the menu
button cancelling. The settings dialog, which is the launcher's own reading and not the game's — its
line should say `a pad on XInput, pushed N time(s)` rather than `no pad answered`, and that is where
XInput's buttons being put into the order the game's mapping names them gets tested. And a pad that
winmm *does* have, which is the path that used to work and the one that must not have been broken by
this.

**A life gained under the brush over the lives.** The mark itself is on the screen — see
[DONE.md](DONE.md) — and what it has not been through is the count changing under it: an extend at
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
pad — see [DONE.md](DONE.md) — but nothing says which part of it moved the cursor, and a d-pad and
a stick are read from different fields. What is left is watching the axis: the dead zone is a
quarter of the travel either side of centre, taken from `g_JoyCaps`, and this pad's caps had to be
written there by orb before the game would have believed them.

**Two more about the question itself**, neither of which a menu session reaches:

- that the demo does not start while it is up. The idle counter is in an update that is not running,
  which is the argument, but 720 frames of it has never been sat through.
- what the key that answers it does to the difficulty select. `g_CurFrameInput` is left as every
  button so that no edge exists on the frame the game carries on into: the cursor should not move,
  and a direction genuinely held across the answer should still move it on the frame after.

**What the front end offers with the bracket in.** Which file each open goes to is measured — see
[DONE.md](DONE.md) — and what is left is the half no log can show: `Extra Start` and the practice
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

Both halves work and are measured — see [DONE.md](DONE.md): a session that stops keeps what it counted,
and a chapter retried counts an attempt.

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
asked for — see [DONE.md](DONE.md) — but on the machine it was run on a borderless tool then took
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

## What picking a run up has not been through

A chapter has been written down, the game closed, and the chapter played back into place — the
landing agreeing with what was written down field for field. See [DONE.md](DONE.md) for that session
and for the fault it caught on the way. What it was, though, is one chapter of stage 1 nine frames
in, driven by keys sent from another program. What is left is everything that is more than that.

**A chapter worth grinding, landing on every field — including the seed.** Chapters of 2009, 4597 and
3394 frames have been picked up now, and the landing check has moved the seed's write twice: out of the
frame before the stage, and out of the callback's own entry, which turned out to be 2048 draws early
because that callback fills a key table from the generator first. See [DONE.md](DONE.md) for both and
for the arithmetic. The check itself now carries `rng=`, having once agreed field for field with a seed
2048 draws out. **Nothing has been resumed since either fix**, so what is owed is one landing that
agrees with `rng=` among the fields it agrees on. The song's position, ignored in that same session and now decided by the song
rather than by the chapter's kind, is unwatched for the same reason: the chapter to hear it on is a
midboss's or a midstage one, and what should be heard is the phrase that was playing rather than the
track's first bar.

**The track's loop, on a resumed stage.** Putting the song back where the chapter had it worked and
then looped a section of itself near the end of the stage, which was the countdown the loop is taken
on being left where the file's old position had put it — moved with the file now, see
[DONE.md](DONE.md). What is left is to hear a resumed stage's track run past that point and take its
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
- **Whether the grid's own way of running the game fast is gone**, which only a display that is
  not a whole multiple of 60 can reach — see [DONE.md](DONE.md) for the same fault in the work
  estimate, which is the one that was found and measured. A grid moment left behind the blank in
  hand made the aim come out at one refresh a frame until the difference had been made up, and
  each of those frames is an update. It is dropped now and the arithmetic has a test, but no
  144Hz session has been through a stall shorter than four game frames, which is the only way
  in: beyond that the phase guard was already resetting the grid. What to watch is the same `1x`
  bucket, in a session whose log says `144Hz monitor is not a multiple of 60Hz`.
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

## Move the rest of the suite onto the space

The mechanism is in — see *Reaching the game's memory with no game there* in [SPEC.md](SPEC.md) —
and `chapter.rs`'s tests are through it: every one drives the real `observe`, over a laid-out
game, with the snapshot really taken and `retry_chapter` really putting the memory back. The six
helpers that used to set `mark` and insert into `starts` in their own words are gone, and with
them the reason none of those tests could fail when `observe` changed. Breaking the production
`starts` insert now fails four of them; before, it failed none.

**What the space does not answer yet.** `State` is still built by hand rather than read by
`Th06::read_state()` out of a laid-out image, so the parse of the game's structures is exercised
only as far as the accessors the chapter path calls — `music`, `music_identity`, `audio_state`,
`audio_thread`, `live_handles`, `captures`, `count_card_attempt`. Reading a whole `State` out of
the image is the next piece, and it is what would let the tests hand over a stage rather than a
struct.

**No track plays in a laid-out game, and that is a limit rather than a choice.** With the sound
structures zeroed, `music()` reads as nothing, so a stage takes `STAGE_SETTLE_FRAMES` *plus*
`MUSIC_WAIT_FRAMES` to begin — the 248 frames `STAGE_BEGINS` names, in front of every test.
Laying out a stream instead would bring it down to the settle alone, but `Music::capture` reaches
into DirectSound through the buffer's vtable, so it needs the COM calls simulated first. Until
then nothing about the music in a snapshot is covered here, and the frames a chapter's music is
judged by are the real game's to answer for.

**`self_check`'s inventory is skipped under a space** — see `fingerprint_untracked`. It is a walk
of every private page in the process, which is a question about the process and not about the
game.

## Put Windows itself behind a seam, and run the suite on any host

The memory seam took the game out of the tests. What is left holding them to Windows is Windows:
the suite is a set of `i686-pc-windows-gnu` binaries because the crate cannot be built without
`windows-sys`, so CI needs a Windows runner and a 32-bit mingw to get anything run at all.

**Measured, so the size of it is not a guess.** Twenty of the thirty-nine files under
`crates/orb/src` name `windows_sys`, and between them they call **sixty-four** distinct Win32
functions: the memory ones the seam already covers, and then `QueryPerformanceCounter`,
`DwmGetCompositionTimingInfo` and `DwmFlush` for the pacing; `SuspendThread`, `ResumeThread`,
`OpenThread` and `GetThreadId` for holding the game still; twenty-odd GDI calls for the status
line; `GetKeyboardState` and DirectInput's `EnumDevices` for the input; `MonitorFromWindow`,
`EnumDisplaySettingsW` and `AdjustWindowRect` for the window. On top of the functions there are
three surfaces that are not functions at all: Direct3D 8 and DirectSound reached through COM
vtables, the window procedure and its message pump, and the game's own code called through
transmuted pointers.

**There is no small first slice.** Only six of `Game`'s sixty-three methods name a Win32 or COM
type — `window`, `d3d_device`, `set_play_viewport`, `chain`, `prepare_frame`, `forget_captures` —
so making the trait traffic in plain data is a morning's work. It buys nothing on its own: an
opaque device handle is still handed to `overlay.rs` and `d3d8.rs`, which make the real calls, so
the host requirement does not move until those calls go behind the seam too. The value arrives
whole or not at all.

**The shape to build, when it is built.** `orb-core` for the logic with no `windows-sys` in it;
`orb-api` for the seam traits and the `#[cfg(windows)]` real implementations; `orb-sim` for the
simulated Windows, with the scenarios in its own `tests/` — a cdylib cannot host them, and the
sim is where they belong anyway. The simulated game becomes a client of the simulated Windows
rather than part of it: it takes its memory, its audio thread and its device from the sim, so that
a snapshot walks the sim's pages the way it walks the real ones and the failures that only happen
around a free or a suspended thread stay reachable. The seam traffics in plain data — no
`windows-sys` types in any signature — which is the whole reason the result builds on a Linux
runner.

**Three things looked like they needed a seam and none of them did.** The drawing, because a
Direct3D device is a pointer to a vtable and a vtable of Rust functions is a device. Holding the
game still, because the threads it stops are ones a test can make and register itself. And the
clock, because every decision the pacing makes is already a function of numbers — `grid_aim`,
`next_budget`, `whole_multiple`, `on_cadence`, `refreshes_this_frame`, the three ceilings — so what
a seam would add is the order the waiting calls come in, and what matters about the waiting is
whether frames land on blanks, which is a measurement of real hardware. `DONE.md` keeps those.

So the lesson for the crate split is that the seam is worth cutting where the *host* is in the way,
not where a call is: what is left needing one is Windows itself, for a suite that runs anywhere.

**The Linux runner is already there, with the host-independent half of the tree on it.** `orb-config`
names no Windows at all — it reads `orb.yaml` and the command line and depends on nothing but clap
and serde — so its twenty-three tests pass on `x86_64-unknown-linux-gnu` as they stand, and CI has a
job that runs them and clippy there. That is the path the rest is meant to move onto, and a crate
that stops building on it is a crate that has taken a dependency on the host — worth finding out
about when it happens rather than at the end of the work.

What is left is the moving: `orb`'s twenty files that name `windows_sys` and the sixty-four Win32
functions behind them, which is the weeks and not the wiring.

**And the one place a simulated Windows is still the answer** is the window and its message pump:
`IN_HOOK` exists because a Win32 call that moves a window dispatches messages synchronously and the
game draws from its window proc, so a hook can be entered again from inside itself. Nothing tests
that, and nothing can until there is a message pump to drive.

## Draw a frame in a test and say what is on it

**The device is done** — `d3d8/recording.rs` — and it needed none of the above. A `Device` is a
pointer to a vtable, so a vtable of Rust functions *is* a device: nothing in the drawing code
branches on anything, because the COM ABI is already the seam. It keeps the quads with their
rectangles, colours and which texture each went through, the clears, and the viewports, and
`Quad::covers`/`overlaps` are how "the mark covers the row" and "it leaves the row below alone" get
asked. Five tests hold it to that.

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

**Still to move:** `retry_ui` (6), `mode_ui` (4), `resume_ui` (8), and what `overlay` and `text` do
with a label. The pattern is the one in `lives_ui`'s tests — a `Recording`, an overlay on a system
font, `clear()` so the assertion is about one frame, then the real drawing call.

**`retry_ui` was tried and taken back out, and what stopped it is not understood yet.** Drawing the
menu across two frames on one device kills the process, with no panic and no message — exit 5, part
way through the test. What is *not* the cause, each ruled out by a probe: the overlay itself builds
on a system font; a label of Japanese renders in Arial (110x14) as does the cursor's arrow (8x14);
and one `RetryMenu::draw` against a fresh device and a fresh menu is fine. It is the second frame,
or the harness around it, and finding out means bisecting inside `Frame`'s own drawing rather than
around it.

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

The suite went from 138 tests to 182 without a game, and most of what it now covers used to need a
session. So this is the list that is left — what a run on 東方紅魔郷 1.02h is still the only witness
to, and therefore what a session is for.

**Addresses and the bytes at them.** Every offset in `game/th06/` is written into a laid-out space
by the same constant the reader reads it with, so a wrong one is wrong on both sides at once. The
prologue bytes a hook is installed over, the exe's md5, and that the jmp lands where it should are
the same kind of claim. Only the real image says.

**That the hooks hold.** Installing a trampoline over a prologue and having the game carry on
through it is a property of that image and that compiler's code, not of the installation logic.

**The pacing, as frames on a screen.** The arithmetic is covered; what is not, and cannot be, is
whether frames land on blanks. The 120Hz and 144Hz numbers in [DONE.md](DONE.md) are the record and
they stay measurements.

**The sound.** No track plays in a laid-out game — `Music::capture` reaches into DirectSound through
the buffer's vtable — so nothing about the music in a snapshot, the streaming margin, or what a
restore does to a stream is covered here. Whether a chapter comes back with its music is a thing to
hear.

**Anything the screen is the judge of.** The drawing tests say where a quad went and in what colour;
they do not say it looks right. The brush stroke reading as a stroke, the panel's own tile showing
through the strips either side of it, the wash being dark enough to read a menu over and light enough
to see the run under — those are looked at.

**And the front end's own answers.** Which items the game's menu lights after a run, what its spell
card history shows in each mode, whether `Extra Start` is offered — the log cannot see a menu.

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
