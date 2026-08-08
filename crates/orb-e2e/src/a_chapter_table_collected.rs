//! **`--collect` and `--judge`: the midstage chapter table built out of a replay.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! A boss's boundaries are found from the game as it is fought, but a stage's waves are a script on a
//! clock, so those boundaries are frame numbers somebody has to choose. `--collect` proposes them — a
//! second into each gap between waves, with nothing to shoot at and no boss to interrupt — and `--judge`
//! holds the game on each one so it can be looked at and corrected. Both write two files beside the game:
//! `chapters.rs`, to paste into the source, and `tuning.txt`, which is read back at startup so that a
//! stage can be judged over more than one sitting.
//!
//! **A pass is driven by a replay**, which is what makes it possible at all: the alternative is playing
//! the whole game by hand for every one. So what all of this rests on is a replay playing out the same way
//! after being stepped and moved through, which is
//! `moving_between_a_replays_stages.rs`.

use crate::fake::th06::{BOSS_ARRIVES, Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;

/// The keys a boundary is looked at and decided with. orb's own names for them are in its `keys` module;
/// these are the numbers, which is what a scenario presses.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const PREVIOUS: u8 = orb_config::keys::LEFT.0;
/// Held with a stepping key, which is what aims at a boundary judged *out* of the table: that one begins
/// no chapter, so the ordinary stepping between chapter starts cannot reach it.
const DROPPED: u8 = orb_config::keys::CTRL.0;
const ADD: u8 = orb_config::keys::A.0;
const WRITE: u8 = orb_config::keys::D.0;
const KEEP: u8 = orb_config::keys::UP.0;
const DROP: u8 = orb_config::keys::DOWN.0;

/// The script frame the detector proposes for this game's first stage, and where that number comes from.
///
/// Its waves are two hundred frames of enemies and two hundred without — `fake::th06::waves` — so the gap
/// begins at script 200, and a proposal goes a second into one: `ENEMY_GAP_FRAMES` is 60, counted from the
/// first frame with nothing to shoot at, which lands on 259. The other floor is passed by then as well —
/// `MIN_GAP_FRAMES` is 120 and the stage's own start was chapter one at frame 8.
const PROPOSED_AT: i32 = 259;

/// And the whole of the gap that boundary sits in, which the pass reports when the next wave arrives: the
/// two hundred frames from script 200 to the boss at [`BOSS_ARRIVES`].
const GAP: i32 = BOSS_ARRIVES as i32 - 200;

/// The two files a pass writes.
const TABLE: &str = "chapters.rs";
const STATE: &str = "tuning.txt";

/// How long each stage of the collect pass's run is, in frames. Past the gap and the wave after it, so the
/// stage that ends is one the detector has had its say about.
const STAGE_ENDS: u32 = 500;

/// A launch making a pass over a replay of its own stages.
///
/// **The song is laid out because of what waiting for it costs**: a stage's first chapter is taken once the
/// stage has settled *and* the music has come up, and a laid-out game with no sound spends the whole of
/// that wait — 248 frames, which is far enough into the gap that the detector's floor on how short a
/// chapter may be would swallow the proposal. A stage with a song under it takes its first chapter at
/// frame 8, which is where a real one takes it.
fn passing(name: &str, judging: bool) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.chapter_tuning = true;
        config.during_replay = true;
        config.chapter_stepping = judging;
        config.log_level = LogLevel::Verbose;
    });
    game.streams_its_song(0);
    game.watches_a_replay_of_its_stages();
    game
}

/// A collect pass proposes the gap in the stage's waves, begins a chapter there, and writes both files
/// with it.
#[test]
fn a_collect_pass_proposes_the_stages_gap_and_writes_the_table_out() {
    in_its_own_process(|| {
        let game = passing("collect", false);
        // A stage that ends, because the end of one is where a collect pass writes: nothing stops that pass
        // — the whole point of it is that the replay runs to the end of the run — so what it hands its
        // findings over at is the next stage beginning.
        game.stages_last(STAGE_ENDS);
        let log = game.log();

        // The boundary, proposed as the stage is played through the gap and acted on at once: a proposal is
        // a boundary of the table, so the chapter begins there on the same update.
        game.frames_until("the boundary the gap is", 600, || {
            log.said(&format!(
                "stage 1 chapter 2 at frame {PROPOSED_AT} (script {PROPOSED_AT}): tl {PROPOSED_AT}"
            ))
        });

        // And the gap itself is reported when the next wave arrives, whether or not a boundary was taken in
        // it: what `ENEMY_GAP_FRAMES` should be is a question about this game's waves, so a pass over a run
        // is the measurement rather than the number being guessed at.
        game.frames_until("the wave after the gap", 600, || {
            log.said(&format!(
                "tuning: gap of {GAP} frames, script 200..{BOSS_ARRIVES}"
            ))
        });

        // The stage over and the next one begun, which is where a pass writes what it has found: the stage
        // just finished is worth keeping even if the pass stops there.
        game.frames_until("stage 2 built", STAGE_ENDS, || game.state().stage == 1);
        assert!(
            log.said(&format!(
                "tuning: wrote {}",
                game.dir().join(TABLE).display()
            )),
            "the table was not written when the stage was left:\n  {}",
            log.lines().join("\n  ")
        );

        // The table as Rust to paste into `chapters.rs`, with this boundary in stage one's row and marked as
        // the detector's rather than as somebody's hand — which the shortest a chapter may be reads, so a
        // table that only said it in a comment would divide the stage differently in play than in the pass
        // that chose it.
        let table = read(&game, TABLE);
        assert!(
            table.contains(&format!("proposed({PROPOSED_AT})")),
            "the boundary the pass proposed is not in the table it wrote:\n{table}",
        );
        // And the stages the pass never reached keep whatever is compiled in, so that a stage can be tuned,
        // baked and left alone while the next one is done.
        assert!(
            table.contains("hand(4597)"),
            "a stage this pass knows nothing about lost what the built-in table has for it:\n{table}",
        );

        // And the same boundary in the form the next sitting reads back.
        let state = read(&game, STATE);
        assert!(
            state.contains(&format!("1 {PROPOSED_AT} keep proposed")),
            "the boundary was not written down for the next sitting:\n{state}",
        );
    });
}

/// A judging pass holds the game on that boundary, and the keys move it: out of the table, back into it,
/// and one more put there by hand.
#[test]
fn a_judging_pass_decides_the_boundary_it_is_held_on_and_takes_a_hand_placed_one() {
    in_its_own_process(|| {
        let game = passing("judge", true);
        let log = game.log();
        game.frames_until("the boundary the gap is", 600, || {
            log.said(&format!("stage 1 chapter 2 at frame {PROPOSED_AT}"))
        });

        // Held on the very frame the boundary is, which is the only frame the judging keys act on: a
        // boundary judged from anywhere inside its chapter is one edited from where it cannot be seen, and
        // a key pressed while the game runs lands on whichever frame it happened to reach.
        game.press(HOLD);
        assert!(
            log.said(&format!("step: held at chapter 2 (script {PROPOSED_AT})")),
            "the game was not held on the boundary:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().script_frames,
            PROPOSED_AT,
            "the hold let the game past the boundary it was aimed at",
        );

        // One step worse, and one more: a proposal goes KEEP → ADJUST → DROP, and nothing wraps past
        // either end — pressing on past DROP must not bring it back.
        game.press(DROP);
        assert!(
            log.said(&format!("tuning: tl {PROPOSED_AT} KEEP -> ADJUST")),
            "the first press did not judge the boundary:\n  {}",
            log.lines().join("\n  ")
        );
        game.press(DROP);
        assert!(
            log.said(&format!("tuning: tl {PROPOSED_AT} ADJUST -> DROP")),
            "the second press did not take the boundary out of the table:\n  {}",
            log.lines().join("\n  ")
        );

        // Written out as soon as anything is decided rather than only at the end of the stage: a sitting
        // that looks at a few boundaries and stops would otherwise lose everything it decided.
        let refused = read(&game, STATE);
        assert!(
            refused.contains(&format!("1 {PROPOSED_AT} drop proposed")),
            "the boundary judged out was not written down as refused:\n{refused}",
        );
        assert!(
            !read(&game, TABLE).contains(&format!("({PROPOSED_AT})")),
            "a boundary judged out of the table is still in it:\n{}",
            read(&game, TABLE),
        );

        // **A boundary judged out is nowhere in the chapter starts the ordinary stepping moves between**,
        // beginning no chapter — so the dropped key with a stepping key is the only way back to one, and it
        // aims by the script clock, which is the only thing that names it. Forward from the frame it is on
        // there is none, and orb says so rather than moving.
        game.keyboard().set(DROPPED, true);
        game.press(NEXT);
        assert!(
            log.said(&format!(
                "step: no boundary judged out after script {PROPOSED_AT}"
            )),
            "the dropped key found a boundary past the only one there is:\n  {}",
            log.lines().join("\n  ")
        );
        game.keyboard().set(DROPPED, false);

        // And back to it from further on, which is a restore of the stage's start and a run forward: the
        // replay comes back with it, so the frame it lands on is the frame the boundary is.
        game.press(HOLD);
        game.frames(GAP as u32 / 4);
        game.keyboard().set(DROPPED, true);
        game.press(PREVIOUS);
        game.keyboard().set(DROPPED, false);
        assert_eq!(
            game.state().script_frames,
            PROPOSED_AT,
            "the dropped key did not land on the boundary judged out:\n  {}",
            log.lines().join("\n  ")
        );

        // And back: one step better is ADJUST, which goes into the table marked for somebody to move by
        // hand. Refused is remembered rather than deleted for exactly this — the decision survives the
        // stage being played again, and it can be taken back.
        game.press(KEEP);
        assert!(
            log.said(&format!("tuning: tl {PROPOSED_AT} DROP -> ADJUST")),
            "a boundary judged out could not be brought back:\n  {}",
            log.lines().join("\n  ")
        );
        let table = read(&game, TABLE);
        assert!(
            table.contains(&format!("proposed({PROPOSED_AT}) /* adjust */")),
            "the boundary came back without the note that says it wants moving:\n{table}",
        );

        // A gap the detector missed, caught by holding the game where it looks like one and pressing: the
        // boundary goes in at the frame on screen, by hand, and a chapter begins there.
        game.press(HOLD);
        game.frames(GAP as u32 / 4);
        game.press(HOLD);
        let at = game.state().script_frames;
        game.press(ADD);
        assert!(
            log.said(&format!("tuning: added tl {at} by hand")),
            "the key put no boundary at the frame the game was held on:\n  {}",
            log.lines().join("\n  ")
        );
        // By hand and not as a proposal, which is the difference that decides how the stage is divided:
        // a hand-placed boundary is exempt from the shortest a chapter may be and a proposal is not.
        let table = read(&game, TABLE);
        assert!(
            table.contains(&format!("hand({at})")),
            "the boundary somebody placed reads as the detector's:\n{table}",
        );

        // And judging *that* one out takes it away altogether rather than keeping it as refused: nothing
        // would propose it again, so a line saying it was refused would only be in the way — and the key
        // that put it there is how it comes back. Which is the difference from the detector's own, whose
        // refusal has to be remembered or the next pass over the stage proposes it again.
        game.press(DROP);
        game.press(DROP);
        assert!(
            log.said(&format!(
                "tuning: took out tl {at}, which was put there by hand"
            )),
            "the boundary somebody placed was kept as refused:\n  {}",
            log.lines().join("\n  ")
        );
        let state = read(&game, STATE);
        assert!(
            !state.contains(&format!("1 {at} ")),
            "the hand-placed boundary is still written down after being judged out:\n{state}",
        );

        // And the write key alone writes both files, for a pass that has looked and decided nothing.
        game.press(WRITE);
        assert!(
            log.said(&format!(
                "tuning: wrote {}",
                game.dir().join(STATE).display()
            )),
            "the write key wrote nothing:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And what one sitting decided is what the next one finds, which is the whole of what `tuning.txt` is for.
#[test]
fn what_a_sitting_decided_is_read_back_by_the_one_after_it() {
    in_its_own_process(|| {
        // A sitting that judged the proposal out and put one of its own in, written down and then closed.
        let (state, at) = {
            let game = passing("collect-first", true);
            let log = game.log();
            game.frames_until("the boundary the gap is", 600, || {
                log.said(&format!("stage 1 chapter 2 at frame {PROPOSED_AT}"))
            });
            game.press(HOLD);
            game.press(DROP);
            game.press(DROP);
            game.press(HOLD);
            game.frames(GAP as u32 / 4);
            game.press(HOLD);
            let at = game.state().script_frames;
            game.press(ADD);
            (read(&game, STATE), at)
        };

        // And the sitting after it, which finds that file where the first left it.
        let game = Fake::attach_finding("collect-again", &[(STATE, &state)], the_run(), |config| {
            config.chapter_tuning = true;
            config.during_replay = true;
            config.log_level = LogLevel::Verbose;
        });
        assert!(
            game.log().said("tuning: read 2 boundary(s) from"),
            "the pass did not read back what the sitting before it wrote:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The verdicts came with them: the boundary judged out is still out — nothing proposes it again,
        // which is why a refusal is remembered rather than deleted — and the one placed by hand is still
        // there and still says which hand put it there.
        game.streams_its_song(0);
        game.watches_a_replay_of_its_stages();
        game.frames_until("the stage played past the gap", 900, || {
            game.state().stage_frames > BOSS_ARRIVES
        });
        game.press(WRITE);
        let table = read(&game, TABLE);
        assert!(
            !table.contains(&format!("({PROPOSED_AT})")),
            "the boundary the sitting before judged out was proposed again:\n{table}",
        );
        assert!(
            table.contains(&format!("hand({at})")),
            "the boundary the sitting before placed by hand did not come back:\n{table}",
        );
    });
}

/// A state file somebody has edited by hand is read line by line, and the lines that will not read say so
/// and cost nothing else.
///
/// **The file invites hand-editing** — it is what a sitting is picked up from, and its own header says to
/// edit it while nothing is running — so a line nobody can read has to be a line named in the log rather
/// than a pass that starts over. Two of the three ways one can be wrong are here: a field that will not
/// parse, and a stage number the table has no row for. The stage number is held to being one or more where
/// it is *read* rather than where `stage - 1` indexes with it, because a number below one would overflow
/// that subtraction.
#[test]
fn a_hand_edited_state_file_is_read_line_by_line_and_names_the_lines_it_cannot() {
    in_its_own_process(|| {
        // A comment, a blank line, two boundaries that read, and two that do not.
        let left = "\
# stage  script frame  keep|adjust|drop  proposed|hand

1 900 keep hand
1 nonsense keep hand
99 900 keep hand
1 1200 adjust proposed
";
        let game = Fake::attach_finding(
            "collect-hand-edited",
            &[(STATE, left)],
            the_run(),
            |config| {
                config.chapter_tuning = true;
                config.during_replay = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let at = game.dir().join(STATE);
        let log = game.log();
        assert!(
            log.said(&format!("tuning: read 2 boundary(s) from {}", at.display())),
            "the two lines that read were not both read:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said(&format!(
                "tuning: {}:4: cannot read `1 nonsense keep hand`",
                at.display()
            )),
            "the line with a frame that is not a number was not named:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said(&format!("tuning: {}:5: no stage 99", at.display())),
            "the line naming a stage the table has no row for was not named:\n  {}",
            log.lines().join("\n  ")
        );

        // And what did read is in the table, which is what says the two that did not cost nothing else.
        // Written where a pass writes it — the stage beginning — rather than on a key: what this reads is
        // the boundaries `load` kept, and a pass writes those out before it looks at the stage at all.
        game.streams_its_song(0);
        game.watches_a_replay_of_its_stages();
        game.frames_until("the pass writing what it read back", 60, || {
            log.said(&format!(
                "tuning: wrote {}",
                game.dir().join(TABLE).display()
            ))
        });
        let table = read(&game, TABLE);
        assert!(
            table.contains("hand(900)") && table.contains("proposed(1200) /* adjust */"),
            "the lines that read did not come back as the boundaries they name:\n{table}",
        );
    });
}

/// One of the two files a pass writes, read out of the directory the game is installed in.
///
/// # Panics
/// Where it is not there, naming it: a pass that wrote nothing is a pass with nothing to assert about.
fn read(game: &Fake, file: &str) -> String {
    let at = game.dir().join(file);
    std::fs::read_to_string(&at).unwrap_or_else(|error| panic!("{}: {error}", at.display()))
}
