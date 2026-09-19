//! **`--collect` over th06c: the midstage chapter table proposed out of this game's own waves.**
//!
//! A fight's boundaries are found from the game as it is fought, but a stage's waves are a script on a clock,
//! so those boundaries are frame numbers somebody has to choose. `--collect` proposes them — a second into
//! each gap between waves, with nothing to shoot at and no boss to interrupt — and writes two files beside
//! the game: `chapters.rs`, to paste into the source, and `tuning.txt`, which is read back at startup so a
//! stage can be judged over more than one sitting.
//!
//! **This is the pass a table of th06c's own would be found by**, and the one piece of this game nothing can
//! be read out of the exe for. What `game::th06c::chapters` holds is th06 1.02h's rows carried across rather
//! than found again — that module says why — so what this says is that the pass works over this game at all:
//! the gap is proposed out of *its* count of enemies alive and *its* script clock, the chapter begins there,
//! and both files come out with the row in them and the rows it never reached as they are compiled in.
//!
//! **Over a run and not over a replay**, which is where this parts from
//! `a_chapter_table_collected_for_th07.rs`. `Chapters::tracking` is `state.in_game || (during_replay &&
//! state.playing && !demo)`, and for this game both terms are false while its own replay is playing:
//! `sight::in_a_run` excludes a playback outright, because th06c runs its attract demo through the same
//! supervisor state and the same stage as a run. So `--replay` buys a pass over this game nothing, and the
//! run somebody — or the bot — plays is what proposes the boundaries.
//!
//! **And the judging half is over a replay**, which is where the stepping keys work at all: `runtime::step`
//! asks for `during_replay && state.replay && state.in_run` together. That those three can be true on this
//! game is a reading of its own — `sight::in_a_run` excludes the attract demo and nothing else, where it
//! once excluded every playback — and `stepping_inside_a_stage_of_th06c` is what the keys themselves are
//! held to.
//!
//! The waves are driven from the test rather than laid out as a script, which is what the laid-out game has:
//! what the detector reads is the count of enemies alive, so a wave is that count going up and a gap is it
//! going back to nothing.

use orb_config::LogLevel;
use orb_core::game::Boundary;
use orb_core::game::th06c::chapters::MIDSTAGE;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The two files a pass writes.
const TABLE: &str = "chapters.rs";
const STATE: &str = "tuning.txt";

/// A stage no pass here reaches, by its row of the table: the third, counted from zero as the table is
/// indexed. The pass plays the first and writes its files as the second is built.
const THE_THIRD_STAGE: usize = 2;

/// A row of the compiled table as a pass writes it out: each boundary under the hand that placed it, with its
/// frame, in the order the row has them.
fn as_written(row: &[Boundary]) -> String {
    let boundaries: Vec<String> = row
        .iter()
        .map(|boundary| {
            let placed = if boundary.by_hand { "hand" } else { "proposed" };
            format!("{placed}({})", boundary.frame)
        })
        .collect();
    format!("&[{}]", boundaries.join(", "))
}

/// How many enemies the wave before the gap has: anything above nothing, the detector asking only whether
/// there is something to shoot at.
const A_WAVE: i32 = 4;

/// How long that wave lasts, in frames: past the shortest a chapter may be, so the gap after it is one the
/// proposal's own floor on the frames since the last chapter is already clear of.
const THE_WAVE_LASTS: u32 = 180;

/// And how long the gap runs, which is past the second the detector waits into one.
const THE_GAP_LASTS: u32 = 120;

/// The frames of nothing to shoot at that a proposal is made a second into, which is
/// `tuning::ENEMY_GAP_FRAMES` — written out here for the reason every other number an e2e test holds is: a
/// test that read it off the code under test would agree with it whatever either said.
const A_SECOND_IN: i32 = 60;

/// How long a stage lasts here: past the wave, the gap and the fight this laid-out game scripts, so that the
/// pass reaches a stage's end and writes what it found.
const STAGE_FRAMES: u32 = 1_200;

/// How many frames a walk here is given before it says the pass went nowhere.
const FRAMES: u32 = STAGE_FRAMES * 2;

/// The keys a boundary is stepped onto and judged with. orb's own names for them are in its `keys` module;
/// these are the numbers, which is what an e2e test presses.
const BACK: u8 = orb_config::keys::LEFT.0;
const DROP: u8 = orb_config::keys::DOWN.0;
/// And the one that holds the game and lets it go again, which is a toggle: a pass judges while standing on
/// a boundary and carries on with the same key.
const HOLD: u8 = orb_config::keys::SPACE.0;

/// A launch making a pass over this game, standing in its first stage with a chapter of it already taken —
/// over a run where nothing is being judged, and over the game's own replay where something is.
fn passing(name: &str, judging: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_tuning = true;
            config.during_replay = judging;
            config.chapter_stepping = judging;
            config.log_level = LogLevel::Verbose;
        },
    );
    let log = game.log();
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    if judging {
        game.watches_a_replay_of_the_stage(0);
    } else {
        game.in_a_run_in_the_stage(0);
    }
    game.frames_until("the stage's own first chapter", FRAMES, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// What a pass wrote into one of its two files.
///
/// # Panics
/// Where the file is not there, which is a pass that wrote nothing.
fn read(game: &Fake, file: &str) -> String {
    let path = game.launch().dir().join(file);
    game.sim()
        .files()
        .text(&path)
        .unwrap_or_else(|| panic!("{} was not written", path.display()))
}

/// A collect pass proposes the gap in the stage's waves, begins a chapter there, and writes both files with
/// it.
#[test]
fn a_collect_pass_over_th06c_proposes_the_stages_gap_and_writes_the_table_out() {
    in_its_own_process(|| {
        let game = passing("th06c-collect", false);
        let log = game.log();

        // A wave, and then nothing to shoot at: what the detector reads is this game's own count of enemies
        // alive, which its manager's walk of its slots leaves.
        game.the_stage_has_enemies(A_WAVE);
        game.frames(THE_WAVE_LASTS);
        let wave_ended_at = game.state().script_frames;
        game.the_stage_has_enemies(0);

        // The boundary, proposed a second into that gap and acted on at once: a proposal is a boundary of the
        // table, so the chapter begins there on the same update.
        let proposed_at = wave_ended_at + A_SECOND_IN;
        game.frames_until("the boundary the gap is", FRAMES, || {
            log.said(&format!("(script {proposed_at}): tl {proposed_at}"))
        });
        game.frames(THE_GAP_LASTS);

        // The stage over and the next one begun, which is where a pass writes what it has found: the stage
        // just finished is worth keeping even if the pass stops there.
        //
        // Waited for since here rather than looked for, because a pass writes at every stage's start: the
        // write this is about is the one that has the boundary in it, and the first went out before the stage
        // had been played at all.
        let from = log.written();
        game.frames_until("the stage after the first", FRAMES, || {
            game.state().stage == 1
        });
        game.frames_until("the table written", FRAMES, || {
            log.said_since(
                from,
                &format!(
                    "tuning: wrote {}",
                    game.launch().dir().join(TABLE).display()
                ),
            )
        });

        // The table as Rust to paste into `chapters.rs`, with this boundary in stage one's row and marked as
        // the detector's rather than as somebody's hand — which the shortest a chapter may be reads, so a
        // table that only said it in a comment would divide the stage differently in play than in the pass
        // that chose it.
        let table = read(&game, TABLE);
        assert!(
            table.contains(&format!("proposed({proposed_at})")),
            "the boundary the pass proposed is not in the table it wrote:\n{table}",
        );
        // And the row is the one whose stage this is, in a table of this game's own seven.
        assert!(
            table.contains("pub const MIDSTAGE: [&[Boundary]; 7] = ["),
            "the table is not this game's own seven rows:\n{table}",
        );
        // And a stage the pass never reached keeps what is compiled in, so that a stage can be tuned, baked
        // and left alone while the next one is done — stage three's row, out of the table itself.
        let compiled = as_written(MIDSTAGE[THE_THIRD_STAGE]);
        assert!(
            table.contains(&compiled),
            "a stage this pass knows nothing about lost what the built-in table has for it, {compiled}:\n\
             {table}",
        );

        // And the same boundary in the form the next sitting reads back.
        let state = read(&game, STATE);
        assert!(
            state.contains(&format!("1 {proposed_at} keep proposed")),
            "the boundary was not written down for the next sitting:\n{state}",
        );
    });
}

/// And a judging pass steps back onto that boundary and puts it out of the table, which is what a pass is
/// *for*: a proposal nobody looked at is a number the detector chose.
///
/// The keys are orb's own and `a_chapter_table_collected.rs` holds them over th06; what this asks of th06c is
/// the half that is the game's — the step landing on the frame out of this game's own script clock, and the
/// boundary that is judged out no longer beginning a chapter of it.
#[test]
fn a_judging_pass_over_th06c_steps_onto_the_boundary_and_puts_it_out_of_the_table() {
    in_its_own_process(|| {
        let game = passing("th06c-judge", true);
        let log = game.log();

        game.the_stage_has_enemies(A_WAVE);
        game.frames(THE_WAVE_LASTS);
        let wave_ended_at = game.state().script_frames;
        game.the_stage_has_enemies(0);
        let proposed_at = wave_ended_at + A_SECOND_IN;
        game.frames_until("the boundary the gap is", FRAMES, || {
            log.said(&format!("(script {proposed_at}): tl {proposed_at}"))
        });

        // A step back, which lands on the chapter that boundary began — strictly either side of where the
        // game is, so a frame or two past it steps back onto it.
        game.frames(A_SECOND_IN as u32);
        game.press(BACK);
        assert_eq!(
            game.state().script_frames,
            proposed_at,
            "the step did not land on the frame the boundary was proposed at",
        );
        assert!(
            log.said("step: held in stage 1 chapter 2"),
            "the step did not hold the game on the boundary's own chapter:\n  {}",
            log.lines().join("\n  ")
        );

        // And judged out from there, twice down: `Keep` to `Adjust` to `Rejected`, which is what a boundary
        // somebody looked at and did not want comes to.
        game.press(DROP);
        game.press(DROP);
        assert!(
            log.said(&format!("tuning: tl {proposed_at} ADJUST -> DROP")),
            "the boundary was not judged out:\n  {}",
            log.lines().join("\n  ")
        );

        // The hold let go of, which is the same key again: a pass that judged a boundary carries on from it,
        // and the frames after it are what reach the end of the stage.
        let from = log.written();
        game.press(HOLD);
        assert!(
            log.said_since(from, "step: playing on at chapter"),
            "the hold was not let go of:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
