//! Finding the midstage chapter boundaries that get baked into `chapters.rs`.
//!
//! Boss boundaries are detected from the game at runtime, but a stage's waves are
//! just a script on a clock, so those boundaries are frame numbers someone has to
//! choose. This mode proposes them — a second into each gap between waves — lets
//! them be judged and corrected by hand while playing, and writes the result out as
//! source.
//!
//! What it has found and what has been decided about each of them also goes to
//! `tuning.txt`, which is read back at startup. A stage is more than one sitting's
//! work: the file is what lets it be picked up again rather than started over.
//!
//! Boss boundaries need no table; a stage's waves are a script on a clock, so those boundaries are frame
//! numbers someone has to pick. Under `--tune`, boundaries are proposed as the stage is played, a second
//! into each gap between waves: nothing left to shoot at, no boss fight to interrupt, and two seconds at
//! least into the chapter now running. One per gap, on the frame the gap becomes one, so a long lull does
//! not fill up with them.
//!
//! Bullets in the air are not part of the test. The game was not written around chapters, so a frame with
//! none of them is rare — asking for one found three places in a stage where a retry unit wants a dozen —
//! and a snapshot restores whatever is in the air exactly, so a boundary does not need the screen to be
//! clear. What it needs is a gap in the script, and enemies are what say that.
//!
//! **One table for every difficulty**, because the clock it is written in does not move with the play. The
//! enemy timeline runs through the midstage and through a midboss fight, and stands still only while the
//! stage's own boss is fought: on stage 1 the chapters at script 2009, 2666 and 3180 track the stage's own
//! clock exactly across the midboss, while every chapter of the boss fight reads 5282. So a frame number
//! always names the same point of the script, however fast the midboss was killed and whatever a difficulty
//! adds to it.
//!
//! What differs is what is on screen at that point: a midboss still alive because this run was slow, the
//! leftovers of a spellcard a harder difficulty adds. A boundary worth keeping therefore sits in a gap the
//! script leaves rather than one a fast clear happened to open, and the `tuning: gap of N frames` line is
//! what tells them apart — a few hundred frames is the script's, a few dozen is that run's. It is also why
//! the difficulty to tune on is the hardest: a gap on Lunatic is a gap on every difficulty below it, and
//! the reverse does not hold.
//!
//! What the pass has proposed is the list chapters are then counted from, exactly as the compiled-in table
//! is, so a stage played twice divides the same way both times. A boundary is proposed only on ground the
//! pass has not covered yet: replaying part of a stage neither writes one down twice nor brings back one
//! that was removed by hand.
//!
//! **Two files, and what is decided is kept.** `chapters.rs` is the table as Rust, to paste over the
//! `chapters.rs` of the game the pass was run on — `crates/orb-core/src/game/th06/` or `th07/`, each holding
//! a table of its own; `tuning.txt` beside it is the same boundaries with what has been decided about each —
//! `keep`, `adjust` or `drop`, and whether a person put it there — and is read back when orb starts. So a
//! stage is not one sitting's work: it can be looked at, judged in part, left, and picked up where it was.
//! Both are written whole as soon as anything is decided, and at every stage's end — looking at a few
//! boundaries and then closing the game is how a sitting ends, and nothing about it should have to be
//! remembered. So edit `tuning.txt` only while nothing is running.
//!
//! Entering a stage keeps everything already known about it and forgets only how far the detector had got,
//! which is about the pass rather than about the stage. A stage neither visited nor read back keeps whatever
//! is compiled in, so stages can still be done one at a time: tune a stage, paste the table in, rebuild,
//! and go on to the next.
//!
//! `--replay` puts a replay in charge of the playing, and the two passes over one are a word each, each of
//! which is `--tune --replay` and one more decision:
//!
//! - **`--collect`**, which nobody watches: the replay runs to the end of the run, nothing stops, and every
//!   boundary is proposed and written to `chapters.rs` beside the launcher as each stage ends. The frames
//!   still come at the display's cadence and each carries 64 updates, so twenty minutes of run go by in
//!   twenty seconds — with one frame in 64 drawn, which is nothing to judge by eye.
//! - **`--judge`**, which somebody does: one update a frame, and stepping between the boundaries. What says
//!   whether one is in the right place is the frames around it in motion, since a still frame shows no
//!   bullets on screen and that is the exact thing the detector already tested. Stepping is instant whatever
//!   the speed, so above 1 buys nothing here.
//!
//! The keys, read while a run is being tracked, which under `--replay` includes a replay. They are fixed in
//! the code rather than settings, since whoever is building a table is the only person who presses them,
//! and a setting nobody changes is one more thing that can be wrong. The stepping keys are a replay's alone
//! — holding a run someone is playing still is what the retry menu is for:
//!
//! | | |
//! | --- | --- |
//! | `right` | runs updates with nothing drawn until the chapter changes, and holds the game there — the ending skip's mechanism, aimed at a boundary instead of the end of a scene |
//! | `left` | restores the stage's start, which rewinds the replay along with everything else, and runs forward to the chapter before this frame. A stage is a few thousand updates and an update is tens of microseconds, so it costs a visible pause |
//! | `space` | holds the game where it is, or lets it carry on. Anywhere, not only on a boundary |
//! | `shift`, held | held with next or back to move between stages, which the game does by starting the replay at one |
//! | `ctrl`, held | held with next or back to move between the boundaries judged out of the table, which the stepping otherwise passes over. The way one is reached, and so the way a `Rejected` is taken back |
//! | `up`, `down` | judge the boundary the chapter being looked at began at, one step per press: `Rejected` → `Adjust` → `Keep` and back. Neither end wraps, so pressing on past `Keep` cannot throw a boundary away |
//! | `a` | puts a boundary at the frame the game is on, for a gap the detector misses, and begins its chapter there. Judged like any other afterwards, so it can be taken out again |
//! | `d` | writes both files, which each stage's end does anyway |
//!
//! **Neither key leaves the stage, and neither does playing on.** What lies outside a stage is the game's to
//! load rather than a snapshot's to put back, so a step stops at the stage's own start going back, and the
//! replay is held at the stage's end however it got there. That end is the game leaving the gameplay scene
//! to build the next stage — not the boss going down, which is well before the stage is done with itself.
//!
//! Moving between stages is asked of the game instead, with `chapter_across_key` held: it starts the replay
//! at another of its stages, the way its own menu does. `GameManager::RegisterChain` raises `currentStage`
//! by one and `ReplayManager::RegisterChain` reads that stage's record — the seed, the rank, the lives, the
//! bombs, the power, the score so far — so writing the stage before the one meant and `GAMEMANAGER_REINIT`
//! into `curState` is the whole of it. The supervisor cuts this stage's chain and registers the next, and
//! nothing of orb's has to be kept whole. A stage the replay does not cover is refused rather than asked
//! for, since the game's own answer to that is to drop to its main menu.
//!
//! The run's score goes back to nothing on the way, along with the count of extra lives it has already paid
//! for, because those are the one thing the game leaves to whichever path it took in.
//! `GameManager::RegisterChain` clears them only when it is starting a run rather than reinitialising, and a
//! stage move reinitialises; the replay then puts the score back from the record of the stage before the one
//! being started, which the first stage does not have. Left alone, stage 1 begins with the score the run was
//! left on, crosses the first extra life's 10,000,000 part way through, and gains a life the recording never
//! had — and with it the rank the extra life carries, which the enemies read. The count of extra lives goes
//! back to none rather than to what the score says, since the stage's own loop only ever raises it: from
//! none it can reach what the restored score has paid for, and from a later stage's count it cannot come
//! down.
//!
//! One thing has to be held back for that to be repeatable. Tearing a stage down ends the run's recording —
//! `GameManager::DeletedCallback` calls `ReplayManager::StopRecording`, which closes the record of inputs
//! off with a blank entry and a frame number no run reaches — and the game does it whether the run was
//! recorded or is a replay being watched. Played back, the record it writes into is the replay's own, at the
//! entry playback has reached, so leaving a stage part way terminates that stage where it was left: play it
//! again and the player takes no input from that frame on, standing still until it is hit. So while a replay
//! is being played back that function does nothing, which costs a recording nothing — the game's own replay
//! writing calls it only for a run it recorded.
//!
//! **Every chapter boundary, not only the midstage ones.** A boss's attacks are chapters too, and a step that
//! skipped them would put a fight out of reach. They are in no table — they are found as the fight runs —
//! and the script clock the table is written in stands still during a fight, so what the stepping moves
//! between is the stage frame each chapter of this stage has begun at.
//!
//! **A boundary the fight has is taken wherever it falls.** Dying and bombing are both seconds long and a
//! fight ends or moves on under them, so neither holds a boundary back: a chapter that begins mid-death
//! kills whoever restores it and one that begins mid-bomb hands them a cleared screen, and both are better
//! than a boundary that is not where the attack began. The way out of a start like that is the chapter
//! before it. What is held back is dialogue and a pause, where the run is not getting anywhere at all — and
//! held rather than dropped, so what the fight did under one is taken on the first frame that can carry a
//! chapter, which for the dialogue a boss arrives with is the frame the fight starts on.
//!
//! **A fight underway outranks the table.** The enemy timeline runs on through a midboss, so a fight that
//! drags reaches the frames the waves after it are divided at, and a chapter beginning half way through a
//! fight is a retry point for neither the fight nor the waves. The entry is spent where it falls rather than
//! held back: the fight's own end is the boundary those waves want, and an entry held over it would fire a
//! frame later as that boundary's double. The same frame in a run that killed the midboss quickly is past
//! the fight and is a boundary — that is the clock's doing, and the table cannot say which run it is in.
//!
//! Both keys are relative to the frame the game is on rather than to the chapter it is in, and strictly so:
//! a boundary a frame or two behind is what back reaches, and forward from there moves on rather than
//! staying put. Neither stops at a boundary judged out of the table: it begins no chapter, so stopping there
//! would be stopping at nothing. It is judged out and not gone, though, so `chapter_dropped_key` with either
//! key goes between exactly those — in the frames of the script clock, since nothing has a stage frame for a
//! boundary no chapter ever began at — and that is how a `Rejected` is reached and taken back.
//!
//! **Why the chapter changed is on the status line, not what the chapter is.** A pass is deciding whether a
//! boundary belongs in the table, and what answers that is which signal produced it — which is also what
//! says where to look when one turns up where none belongs. So the line under `CH 05` is one of:
//!
//! | | |
//! | --- | --- |
//! | `STAGE`, `STAGE_AFTER_MIDBOSS`, `MIDBOSS_NONSPELL`, `MIDBOSS_SPELL`, `BOSS_NONSPELL`, `BOSS_SPELL` | the game's own, settled as the run goes. In no table, and nothing to judge |
//! | `AUTO 1886 KEEP` | the table's, as the detector proposed it, with what has been decided about it |
//! | `HAND 1886 KEEP` | the table's, put there by hand — the number nothing would find again |
//!
//! Told apart that finely because which fight it is and whether the attack has a name is what says where in
//! a stage the game is standing. A boss arriving is its first attack starting, so there is nothing else to
//! call that; and what follows a midboss is the stage carrying on, so it is named for that rather than for
//! the defeat behind it. `HOLD` says only that the game's update is being held, which is a different
//! question and is on its own line. A run shows the chapter's name in place of both lines — see
//! [`crate::window`] — because there nothing is being judged and where dying will send the player is the
//! whole of what the line is for.
//!
//! **The judging keys work only with the game held on a boundary.** A frame is a sixtieth of a second, so a
//! key pressed while the game runs lands on whichever frame it reached, which is no way to aim at one; and a
//! chapter judged from anywhere inside it is a boundary edited from where it cannot be seen. So what a key
//! acts on is always the frame on screen, and the status line is always naming it.
//!
//! A boundary is `Keep` from the moment it is proposed. `Adjust` keeps it and writes it out marked —
//! `proposed(1886) /* adjust */` — for a gap that is real where the frame is not quite right. `Rejected`
//! keeps it out of the table while remembering it, because a decision outranks the detector: the same stage
//! played again would otherwise propose it back, and taking a rejection back means finding it again.
//!
//! Whose hand a boundary came from is written out as the entry itself — `hand(1886)` against
//! `proposed(1886)` — since that is the number nothing would propose again if it were lost, and refusing one
//! put there by hand takes it out altogether rather than remembering it as refused. There is nothing for a
//! refusal to hold back there, and `a` is the way back.
//!
//! **The shortest a chapter may be does not apply to one put there by hand.** That floor is there to stop a
//! boss's opening flurry of script transitions carving out chapters a fraction of a second long, and a hand
//! is not that: stage 5's 2363 was added 54 frames after the boundary at 2309, and being dropped on every
//! pass but the one it was added on would lose what somebody wrote down while leaving it in the table to
//! look at. Which is why `hand` is an entry of the table and not a comment on a number: the exemption is
//! read in play, where a comment could not be, and a table that only remarked on it divided a stage
//! differently from the pass that chose it.
//!
//! **A boundary and the start of a chapter are one thing**, and adding one by hand begins its chapter on its
//! own frame. Crossing a boundary is otherwise noticed on the update where the script clock has reached it,
//! and for a frame the game is already standing on that is the update after — so the chapter would begin one
//! frame past the number written down, while the same table read on a later pass began it on the number
//! itself. That frame is what the status line shows, what the flash goes off on and what a step lands on:
//! three places for one thing to be, and the one on screen the wrong one. Beginning it there also keeps the
//! detector from offering one of its own a frame or two later: both are reading the same lull — a hand goes
//! down when the screen empties, the detector a second into it — so a boundary added just before the
//! detector would have spoken leaves two a frame apart, and the shortest a chapter may be is the rule that
//! already says no to that.
//!
//! The frame a boundary falls on is one frame — a sixtieth of a second under `--judge`, and one of the ones
//! that go undrawn above it — so a step holds the game on it. What is held back is the game's update; the
//! drawing carries on, so that frame stays on screen and the status line's `SCRIPT` is the number that would
//! be written down.
//!
//! **A judging pass washes brighter and longer than a run does**, and whichever way `boundary_flash` is
//! left: the setting is about a run somebody is playing, and a pass with no wash has nothing to say a
//! boundary has been reached by. A boundary is one frame among a stage's thousands, and nothing is being
//! dodged on the frame underneath, so it can take the field for the sixth of a second it has — one frame in
//! twenty is all that frame gets — and still be gone before the next boundary. It marks one the game runs
//! past under `space` as well as one a step stops on.
//!
//! Every gap long enough to have been a candidate goes to the log at `verbose`, taken or not, with its
//! length and how far into the chapter it fell. What `ENEMY_GAP_FRAMES` should be is a question about this
//! game's waves, and that line is how a pass answers it.

use std::path::{Path, PathBuf};

use crate::game::{Game, State};
use crate::{detail, log};

/// How long with nothing left to shoot at before the stage counts as being between
/// waves.
///
/// Bullets in the air are not part of the test. The game was not written around
/// chapters, so a frame with none of them is rare — asking for one found three
/// places in a stage where a retry unit wants a dozen — and a snapshot restores
/// whatever is in the air exactly, so a boundary does not need the screen to be
/// clear. What it needs is a gap in the script, and enemies are what say that.
const ENEMY_GAP_FRAMES: u32 = 60;

/// The shortest a chapter may be. A gap that comes sooner than this is passed over
/// rather than saved up: the next one will do.
const MIN_GAP_FRAMES: u32 = 120;

/// The shortest gap the log bothers to mention. What `ENEMY_GAP_FRAMES` should be is
/// a question about this game's waves, so every gap long enough to be a candidate is
/// reported whether or not it was taken — a pass over a run is then the measurement,
/// rather than the number being guessed at and the table coming out short.
const GAP_WORTH_REPORTING: u32 = 10;

/// The table, as Rust to paste into the source.
const TABLE_FILE: &str = "chapters.rs";
/// The same boundaries and what has been decided about them, in a form this reads
/// back.
const STATE_FILE: &str = "tuning.txt";

pub struct Tuning {
    /// Per stage, the pass over it. `None` for a stage neither visited nor read back,
    /// which keeps whatever the compiled-in table has for it.
    passes: Vec<Option<Pass>>,
    /// Where both files live: beside the launcher, which is beside the game.
    dir: PathBuf,
}

/// What has been decided about a boundary while looking at it.
#[derive(Clone, Copy, PartialEq)]
pub enum Verdict {
    /// Goes into the table as it stands.
    Keep,
    /// Goes in, marked in the written source for someone to move by hand.
    Adjust,
    /// Kept out of the table. Remembered rather than deleted, so that the decision
    /// survives the stage being played over — the detector would otherwise propose it
    /// again — and so that it can be taken back.
    Rejected,
}

/// One boundary of one stage.
#[derive(Clone, Copy)]
pub struct Boundary {
    frame: i32,
    verdict: Verdict,
    /// Whether a person put it there rather than the detector. Worth telling apart:
    /// the detector's proposals come back by playing the stage again, and this one
    /// does not, so it is the one that would be quietly lost.
    by_hand: bool,
}

/// What one stage's pass holds, and how much of the stage it has seen this session.
struct Pass {
    /// Sorted by frame, and holding the ones judged out as well as the ones in the
    /// table.
    boundaries: Vec<Boundary>,
    /// The furthest script frame the pass has reached this session.
    covered: i32,
    /// How long there has been nothing to shoot at. Counted over new ground only,
    /// along with `covered`, so that replaying part of a stage leaves it holding
    /// what it held there the first time.
    without_enemies: u32,
}

/// What is known about a boundary, for whoever is looking at it.
#[derive(Clone, Copy)]
pub struct Judged {
    /// Which boundary this is about, as the script frame it would be written down as.
    /// Not always the frame on screen — a chapter is judged from anywhere inside it — and
    /// what the judging keys are about to change is not a thing to have to work out.
    pub frame: i32,
    pub verdict: Verdict,
    pub by_hand: bool,
}

impl Tuning {
    pub fn new(game: &dyn Game, dir: PathBuf) -> Self {
        let mut tuning = Self {
            passes: (0..game.midstage_table().len()).map(|_| None).collect(),
            dir,
        };
        tuning.load();
        tuning
    }

    /// Starts a fresh look at `stage`, keeping everything already known about it —
    /// which is the point of writing it down — and forgetting only how far the
    /// detector had got, since that is about this pass and not about the stage.
    pub fn begin_stage(&mut self, stage: i32) {
        if let Some(slot) = self.slot(stage) {
            match slot {
                Some(pass) => {
                    pass.covered = i32::MIN;
                    pass.without_enemies = 0;
                }
                None => *slot = Some(Pass::new()),
            }
        }
    }

    /// Proposes a boundary a second into a gap between waves: nothing to shoot at
    /// and no boss fight to interrupt. Nothing here is specific to a difficulty,
    /// which is why one table covers all of them.
    ///
    /// Only on ground the pass has not covered yet. Stepping back through the
    /// chapters plays part of the stage a second time, and a second look at the same
    /// frames must neither propose a boundary twice nor bring back one that was
    /// judged out.
    pub fn propose(&mut self, state: &State, since_last: u32) {
        let nothing_to_shoot = state.enemy_count == 0 && !state.boss_present;
        let frame = state.script_frames;
        let Some(pass) = self.pass_mut(state.stage) else {
            return;
        };
        if frame <= pass.covered {
            return;
        }
        pass.covered = frame;
        if !nothing_to_shoot && pass.without_enemies >= GAP_WORTH_REPORTING {
            detail!(
                "tuning: gap of {} frames, script {}..{frame}",
                pass.without_enemies,
                frame - pass.without_enemies as i32,
            );
        }
        pass.without_enemies = if nothing_to_shoot {
            pass.without_enemies + 1
        } else {
            0
        };
        // The frame the gap becomes one, and not every frame after it: a lull of ten
        // seconds is one boundary, where testing the two floors separately would put
        // one every time the shorter of them came round again.
        if pass.without_enemies == ENEMY_GAP_FRAMES && since_last >= MIN_GAP_FRAMES {
            pass.propose(frame);
        }
    }

    /// Puts a boundary at `frame` by hand, for a gap the detector misses, and reports
    /// whether the stage has one there now. An explicit hand is allowed to bring back
    /// one that was judged out, which asking again for the same frame is the way to do.
    pub fn add(&mut self, stage: i32, frame: i32) -> bool {
        let Some(pass) = self.pass_mut(stage) else {
            return false;
        };
        let known = pass.at(frame).map(|boundary| boundary.verdict);
        pass.put(Boundary {
            frame,
            verdict: Verdict::Keep,
            by_hand: true,
        });
        match known {
            None => log!("tuning: added tl {frame} by hand"),
            Some(Verdict::Keep) => log!("tuning: tl {frame} is already there"),
            Some(verdict) => log!("tuning: tl {frame} {} -> KEEP by hand", verdict.label()),
        }
        true
    }

    /// Judges one boundary of `stage`, one step better or one step worse: `Rejected`
    /// to `Adjust` to `Keep`, and back. Nothing wraps — pressing on past `Keep` must
    /// not throw the boundary away.
    ///
    /// Which boundary is the caller's to say, and it says the chapter's own: the
    /// nearest one behind would be some other chapter's whenever a boss began this
    /// one, and judging a boundary that is not the one on screen is worse than
    /// judging nothing.
    pub fn judge_up(&mut self, stage: i32, boundary: i32) {
        self.judge(stage, boundary, |verdict| match verdict {
            Verdict::Rejected => Verdict::Adjust,
            _ => Verdict::Keep,
        });
    }

    pub fn judge_down(&mut self, stage: i32, boundary: i32) {
        self.judge(stage, boundary, |verdict| match verdict {
            Verdict::Keep => Verdict::Adjust,
            _ => Verdict::Rejected,
        });
    }

    /// Puts a boundary out of the table, which is where `judge_down` twice would also
    /// leave it.
    pub fn reject(&mut self, stage: i32, boundary: i32) {
        self.judge(stage, boundary, |_| Verdict::Rejected);
    }

    fn judge(&mut self, stage: i32, boundary: i32, step: fn(Verdict) -> Verdict) {
        let Some(pass) = self.pass_mut(stage) else {
            return;
        };
        let Some(before) = pass.at(boundary) else {
            return log!("tuning: tl {boundary} is not a boundary of this stage");
        };
        let verdict = step(before.verdict);
        if verdict == before.verdict {
            return;
        }
        // One put there by hand and judged out goes altogether. Nothing would propose
        // it again, so a line saying it was refused would only be in the way — and
        // `tuning_add_key` is how it came to be there, which is how it comes back.
        //
        // The detector's own are kept as refused instead, because forgetting one is
        // exactly what has it proposed again the next time the stage is played.
        if verdict == Verdict::Rejected && before.by_hand {
            pass.forget(boundary);
            return log!("tuning: took out tl {boundary}, which was put there by hand");
        }
        pass.put(Boundary { verdict, ..before });
        log!(
            "tuning: tl {boundary} {} -> {}",
            before.verdict.label(),
            verdict.label()
        );
    }

    /// What is known about one boundary, or `None` for a frame this stage has never
    /// had one at.
    pub fn judged(&self, stage: i32, boundary: i32) -> Option<Judged> {
        let found = self.pass(stage)?.at(boundary)?;
        Some(Judged {
            frame: found.frame,
            verdict: found.verdict,
            by_hand: found.by_hand,
        })
    }

    /// How many boundaries of `stage` the table would have.
    pub fn count(&self, stage: i32) -> usize {
        self.pass(stage).map_or(0, |pass| pass.kept().count())
    }

    /// The frames of `stage`'s boundaries that are judged out of the table, in order.
    /// They begin no chapter, so nothing else in orb knows where they are.
    pub fn rejected(&self, stage: i32) -> impl Iterator<Item = i32> {
        self.pass(stage)
            .into_iter()
            .flat_map(|pass| pass.boundaries.iter())
            .filter(|boundary| boundary.verdict == Verdict::Rejected)
            .map(|boundary| boundary.frame)
    }

    /// The newest boundary in `(after, upto]` that the table would have, which is what
    /// says a chapter begins here.
    pub fn passed(&self, stage: i32, after: i32, upto: i32) -> Option<i32> {
        let pass = self.pass(stage)?;
        pass.kept()
            .map(|boundary| boundary.frame)
            .filter(|frame| *frame > after && *frame <= upto)
            .max()
    }

    fn pass(&self, stage: i32) -> Option<&Pass> {
        self.passes.get(usize::try_from(stage).ok()?)?.as_ref()
    }

    fn pass_mut(&mut self, stage: i32) -> Option<&mut Pass> {
        self.slot(stage)?.as_mut()
    }

    fn slot(&mut self, stage: i32) -> Option<&mut Option<Pass>> {
        self.passes.get_mut(usize::try_from(stage).ok()?)
    }

    /// Writes both files: the table to paste into the source, and everything this
    /// knows so that the next session starts where this one stopped.
    pub fn write(&self, game: &dyn Game) {
        self.put(&self.dir.join(TABLE_FILE), &self.table(game));
        self.put(&self.dir.join(STATE_FILE), &self.state(game));
    }

    fn put(&self, path: &Path, contents: &str) {
        match orb_api::fs::write(path, contents.as_bytes()) {
            Ok(()) => log!("tuning: wrote {}", path.display()),
            Err(error) => log!("tuning: cannot write {}: {error}", path.display()),
        }
    }

    /// The table as Rust, ready to replace the one in `chapters.rs`. Stages this
    /// knows nothing about keep the values already compiled in, so a stage can be
    /// tuned, baked and left alone while the next one is done.
    fn table(&self, game: &dyn Game) -> String {
        let table = game.midstage_table();
        let mut source = format!("pub const MIDSTAGE: [&[Boundary]; {}] = [\n", table.len());
        for (stage, built_in) in table.iter().enumerate() {
            let frames: Vec<String> = match self.pass(stage as i32) {
                Some(pass) => pass
                    .kept()
                    .map(|boundary| {
                        entry(
                            boundary.frame,
                            boundary.by_hand,
                            boundary.verdict == Verdict::Adjust,
                        )
                    })
                    .collect(),
                None => built_in
                    .iter()
                    .map(|built| entry(built.frame, built.by_hand, false))
                    .collect(),
            };
            let label = row_label(game, stage);
            source += &format!(
                "    /* {label:width$} */ &[{}],\n",
                frames.join(", "),
                width = row_label_width(game),
            );
        }
        source + "];\n"
    }

    fn state(&self, game: &dyn Game) -> String {
        let stages = game.midstage_table().len();
        let mut text = String::from(
            "# What the tuning pass has found, and what has been decided about it.\n\
             # Read back when orb starts, so that a stage can be judged over more than\n\
             # one session. Written whole every time: edit it while nothing is running.\n\
             #\n\
             # stage  script frame  keep|adjust|drop  proposed|hand\n",
        );
        for stage in 0..stages {
            let Some(pass) = self.pass(stage as i32) else {
                continue;
            };
            text += &format!("\n# {}\n", row_label(game, stage));
            for boundary in &pass.boundaries {
                text += &format!(
                    "{} {} {} {}\n",
                    stage + 1,
                    boundary.frame,
                    boundary.verdict.name(),
                    if boundary.by_hand { "hand" } else { "proposed" },
                );
            }
        }
        text
    }

    /// Reads back what earlier sessions found. A missing file is the ordinary case
    /// the first time a stage is looked at.
    fn load(&mut self) {
        let path = self.dir.join(STATE_FILE);
        let text = match orb_api::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            // A file that is there and will not read, which no e2e test reaches: what an e2e test can hand
            // a launch is *bytes* — `Fake::attach_finding` — and this arm wants something else at the path
            // altogether. The two ways a line can be wrong below are covered.
            Err(error) => return log!("tuning: cannot read {}: {error}", path.display()),
        };
        let mut read = 0;
        for (number, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            match parse(line) {
                // The file's stages are counted from one, the way they are named.
                Some((stage, boundary)) => match self.slot(stage - 1) {
                    Some(slot) => {
                        slot.get_or_insert_with(Pass::new).put(boundary);
                        read += 1;
                    }
                    None => log!(
                        "tuning: {}:{}: no stage {stage}",
                        path.display(),
                        number + 1
                    ),
                },
                None => log!(
                    "tuning: {}:{}: cannot read `{line}`",
                    path.display(),
                    number + 1
                ),
            }
        }
        log!("tuning: read {read} boundary(s) from {}", path.display());
    }
}

/// What a row of the table is called: its stage number, or the game's own name for a row it does not number —
/// see [`Game::stages_past_the_numbered`].
///
/// **Asked of the game and not read off the table's length.** Taking the last row for the Extra is right for
/// th06 and wrong for th07, whose Phantasm sits after it: a pass over th07's Extra wrote the row out as
/// `stage 7` and the Phantasm as `extra`, so the file said Extra of the row above the Extra's.
fn row_label(game: &dyn Game, stage: usize) -> String {
    let named = game.stages_past_the_numbered();
    let numbered = game.midstage_table().len().saturating_sub(named.len());
    match stage.checked_sub(numbered).and_then(|past| named.get(past)) {
        Some(name) => (*name).to_owned(),
        None => format!("stage {}", stage + 1),
    }
}

/// How wide those labels are, so the rows of the generated table line up: whichever is longest.
fn row_label_width(game: &dyn Game) -> usize {
    (0..game.midstage_table().len())
        .map(|stage| row_label(game, stage).len())
        .max()
        .unwrap_or_default()
}

/// One entry of the generated table.
///
/// Which hand it came from is the call and not a comment on the number, because the shortest a
/// chapter may be reads it: a boundary somebody placed is exempt and a proposal is not, so a
/// table that only remarked on it divided a stage differently in play than in the pass that
/// chose it. `adjust` stays a comment — nothing reads that, and what it means is that a boundary
/// is in roughly the right place while somebody works out where exactly.
fn entry(frame: i32, by_hand: bool, adjust: bool) -> String {
    let placed = if by_hand { "hand" } else { "proposed" };
    let note = if adjust { " /* adjust */" } else { "" };
    format!("{placed}({frame}){note}")
}

/// One line of the state file: the stage counted from one, then the boundary.
fn parse(line: &str) -> Option<(i32, Boundary)> {
    let mut fields = line.split_whitespace();
    // Held to being a stage number here and not where `stage - 1` indexes with it: the file
    // invites hand-editing, and a number below one overflows that subtraction. One malformed
    // line is then the `cannot read` every other malformed field here gets.
    let stage = fields.next()?.parse().ok().filter(|stage| *stage >= 1)?;
    let frame = fields.next()?.parse().ok()?;
    let verdict = Verdict::parse(fields.next()?)?;
    let by_hand = match fields.next() {
        Some("hand") => true,
        Some("proposed") | None => false,
        Some(_) => return None,
    };
    fields.next().is_none().then_some((
        stage,
        Boundary {
            frame,
            verdict,
            by_hand,
        },
    ))
}

impl Verdict {
    /// For the status line, which is the ASCII strip in the black beside the game.
    pub fn label(self) -> &'static str {
        match self {
            Self::Keep => "KEEP",
            Self::Adjust => "ADJUST",
            Self::Rejected => "DROP",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Adjust => "adjust",
            Self::Rejected => "drop",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name {
            "keep" => Some(Self::Keep),
            "adjust" => Some(Self::Adjust),
            "drop" => Some(Self::Rejected),
            _ => None,
        }
    }
}

impl Pass {
    fn new() -> Self {
        Self {
            boundaries: Vec::new(),
            covered: i32::MIN,
            without_enemies: 0,
        }
    }

    /// The ones the table would carry, in order.
    fn kept(&self) -> impl Iterator<Item = &Boundary> {
        self.boundaries
            .iter()
            .filter(|boundary| boundary.verdict != Verdict::Rejected)
    }

    fn at(&self, frame: i32) -> Option<Boundary> {
        let at = self
            .boundaries
            .binary_search_by_key(&frame, |boundary| boundary.frame)
            .ok()?;
        Some(self.boundaries[at])
    }

    fn forget(&mut self, frame: i32) {
        if let Ok(at) = self
            .boundaries
            .binary_search_by_key(&frame, |entry| entry.frame)
        {
            self.boundaries.remove(at);
        }
    }

    /// Adds or replaces, keeping the list sorted so that it can be searched and
    /// written in order.
    fn put(&mut self, boundary: Boundary) {
        match self
            .boundaries
            .binary_search_by_key(&boundary.frame, |entry| entry.frame)
        {
            Ok(at) => self.boundaries[at] = boundary,
            Err(at) => self.boundaries.insert(at, boundary),
        }
    }

    /// What the detector found. Anything already known keeps the verdict it has: what
    /// a proposal says is where a boundary might go, and a decision outranks it.
    fn propose(&mut self, frame: i32) {
        if self.at(frame).is_none() {
            self.put(Boundary {
                frame,
                verdict: Verdict::Keep,
                by_hand: false,
            });
        }
    }
}

// Only on the target th06 and th07 are built for — see `game::th06c::tests`, which is what the other
// target's run of these is for.
#[cfg(all(test, target_arch = "x86"))]
mod tests {
    use super::Tuning;
    use crate::game::th06::Th06;

    /// What a second sitting has to find: every boundary, every verdict, and which of
    /// them nothing would propose again.
    #[test]
    fn what_a_session_decided_comes_back_in_the_next_one() {
        let dir = std::env::temp_dir().join("orb-tuning-round-trip");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join(super::STATE_FILE)).ok();

        let mut first = Tuning::new(&Th06, dir.clone());
        first.begin_stage(0);
        first.begin_stage(3);
        for frame in [1886, 2500, 5144] {
            first.add(0, frame);
        }
        first.add(3, 900);
        first.judge_down(0, 2500);
        first.reject(0, 5144);
        first.write(&Th06);

        let next = Tuning::new(&Th06, dir);
        assert_eq!(next.count(0), 2);
        assert_eq!(next.judged(0, 1886).unwrap().verdict.label(), "KEEP");
        assert_eq!(next.judged(0, 2500).unwrap().verdict.label(), "ADJUST");
        assert!(next.judged(0, 1886).unwrap().by_hand);
        // Refusing one that was put there by hand takes it out altogether, so there is
        // nothing of it to come back.
        assert!(next.judged(0, 5144).is_none());
        // Stages are counted from one in the file and from zero in the table.
        assert_eq!(next.count(3), 1);
        assert!(next.judged(3, 900).is_some());
        assert!(next.judged(1, 900).is_none());
    }

    /// The table this writes carries which hand each boundary came from as data, because the
    /// shortest a chapter may be reads it: a table that only said it in a comment divided a stage
    /// differently in play than in the pass that chose the numbers.
    #[test]
    fn the_written_table_says_which_hand_each_boundary_came_from() {
        let dir = std::env::temp_dir().join("orb-tuning-table");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join(super::STATE_FILE)).ok();

        let mut tuning = Tuning::new(&Th06, dir);
        tuning.begin_stage(0);
        tuning.add(0, 1886);
        let source = tuning.table(&Th06);
        assert!(source.contains("hand(1886)"), "{source}");
        // And a stage this knows nothing about keeps what is compiled in, hands and all.
        assert!(source.contains("hand(4597)"), "{source}");
        assert!(source.contains("proposed(880)"), "{source}");
    }

    /// **And it names the rows the game does not number.** th07's table is its six stages, its Extra and
    /// the Phantasm after it, so the row to paste an Extra boundary into is not the last one — and a writer
    /// that took the last for the Extra called the Extra `stage 7` and the Phantasm `extra`, which is the row
    /// above the one a reader would put it in.
    #[test]
    fn the_written_table_names_the_stages_the_game_does_not_number() {
        let dir = std::env::temp_dir().join("orb-tuning-named-rows");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join(super::STATE_FILE)).ok();

        let tuning = Tuning::new(&crate::game::th07::Th07, dir.clone());
        let source = tuning.table(&crate::game::th07::Th07);
        assert!(source.contains("/* extra"), "{source}");
        assert!(source.contains("/* phantasm"), "{source}");
        assert!(!source.contains("/* stage 7"), "{source}");
        assert!(!source.contains("/* stage 8"), "{source}");

        // And th06's Extra is its last row, which is the case the shape used to be read off.
        let of_th06 = Tuning::new(&Th06, dir).table(&Th06);
        assert!(of_th06.contains("/* extra"), "{of_th06}");
        assert!(!of_th06.contains("/* stage 7"), "{of_th06}");
    }

    /// A line whose stage is not a stage number is one that cannot be read, however far out it
    /// is: the number is counted from one and indexed with one taken off it, so nothing below
    /// one may reach that subtraction.
    #[test]
    fn a_stage_below_one_cannot_be_read() {
        assert!(super::parse("1 900 keep hand").is_some());
        for line in [
            "0 900 keep hand",
            "-1 900 keep",
            "-2147483648 900 keep hand",
        ] {
            assert!(super::parse(line).is_none(), "{line}");
        }
    }
}
