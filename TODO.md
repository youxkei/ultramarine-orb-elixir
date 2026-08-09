# To do

## Going back more than one chapter

The retry menu offers this chapter and the stage's start, those being the two `Checkpoint`s
`Chapters` keeps. Why the way out of a bad chapter has to be the chapter *before* it is beside
`chapter.rs`'s `guarded`, which also says that stepping has that way out and the retry menu does
not.

The frames are already kept: `Chapters::starts` holds where every chapter of the stage began,
for the stepping keys. What is missing is their snapshots. So the shape is a stack of those per
stage, dropped from the chapter restored onwards, and a menu listing what is in it rather than
the two fixed places. What it costs is a chapter's snapshot — five or six regions of about four
megabytes — times however deep the stack is allowed to go.

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
holding a `static GAME: Th06`, and `orb-e2e`'s `fake` is a host half, a 紅魔郷 half and a 妖々夢 half. What is left of
that list is `game/th06/chapters.rs`, where `MIDSTAGE` is seven stages because 紅魔郷 has seven — and
a game that declines chapters does not reach it.

Two things to settle before the second game is *played* rather than merely paced: whether `State`
says everything a chapter needs for a game whose scoring or resources work differently, and whether
the midstage table's shape — script frame numbers per stage — holds where stages are not one
script on one clock. A game that declines chapters has neither in its way.

### What 妖々夢 does not get, and what each one would take

[docs/adr/0004](docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) is built to its last step,
and what orb does there is settled — see that decision for the two launches that settled it. It
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

## Two things the pacing has left open

The cause and the fix of the stutter are beside `frame::Pacing::grid` and in `orb-e2e`'s `pacing`'s
`compose` section. What is left of it is these:

- **Where `PLAY_SOUNDS` is called, which is orb's choice and costs the frame a spell card starts on
  its blank.** The game's own call, made between the update and the draw, so it is inside the span the
  frame has to reach its blank in — and on the frame a card starts it runs for a large part of one.
  That frame loses its blank whatever the pacing does — it began against a budget the frames
  before it left, and a budget is a prediction — so what is left is only the *where*. After `Present`
  instead, the frame keeps its blank and the sound starts a frame later: one frame of audio latency
  against a lost refresh, and nothing says which of the two anybody notices.
- **Whether the miss the ratchet trips on is always a real one.** A frame charged for a miss did not
  show up as a broken gap in the same period's `gaps in refreshes`. Either the two counters are a
  frame out of step at a period boundary — `measure_compose` runs at the top of a frame and the gap
  is worked out at the bottom, with the report between them — or the overshoot sat right on the
  half-refresh boundary the two round opposite ways from. Two of the three ways a miss is charged
  wrongly are answered since: `recovering` excuses the frame after a load and `overran` excuses a
  frame whose own drawing outgrew its budget. What is left errs toward giving the compositor longer,
  which is the safe direction, but every wrong step is `MISS_STEP_US` of lag for the rest of the run.

## Move the rest of the suite onto the space

**`chapter.rs`'s scenarios build a `State` by hand and step the detector with it**, where production
reads one out of the game's memory with `read_state`. So what those cannot catch is `observe` reading a
different stage from the one the image holds. The parse itself is covered by `orb-e2e`'s
`the_run_read_back`, and the handover once by `orb-e2e`'s `pointdevice_run`, where nothing hands orb a
`State` at all. What is left is that `chapter.rs`'s own twenty-seven still cannot fail for that reason,
which matters for the three cases only they reach: the shortest a chapter may be, a boundary judged out,
and a bomb swallowing a transition.

**What the game that plays the game's part does not do yet**, each of which is a scenario somebody
could write next:

- **No practice run and no Extra.**
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

**No track plays in a laid-out game unless a scenario asks for one**, and a stage with none takes
`STAGE_SETTLE_FRAMES` *plus* `MUSIC_WAIT_FRAMES` to begin — the 248 frames `STAGE_BEGINS` names, in front
of every test that is not about the music. `Fake::plays_its_songs` is the numbers a track is told apart by
and `Fake::streams_its_song` is the whole of one, buffer and file handle included, which brings that down
to the settle alone.
