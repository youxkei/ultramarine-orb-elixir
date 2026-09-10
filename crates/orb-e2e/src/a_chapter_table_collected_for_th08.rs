//! **`--collect` over th08: a midstage boundary proposed out of this game's own waves.**
//!
//! A stage's waves are a script on a clock, so its midstage boundaries are frame numbers somebody has to
//! choose, and `--collect` proposes them — a second into each gap between waves, with nothing to shoot at and
//! no boss to interrupt — writing `chapters.rs` to paste into the source and `tuning.txt` to read back at
//! startup. th08's table is compiled in already (`game/th08/chapters.rs`, nine rows); what this says is that
//! the pass still works over it — the gap proposed out of *this game's* count of enemies to shoot at and its
//! script clock, the chapter begun there, both files out with the row in them — and that a judging pass over
//! the game's own replay steps onto the boundary and puts it out.
//!
//! **The count is `Th08::enemies_to_shoot_at`**, a walk of the slots for the ones the player's shot can take
//! and not the manager's own count of the live ones — so a wave here is enemies laid out alive and shootable,
//! which is what the image's `enemies_alive` writes, and a gap is none of them.

use orb_config::LogLevel;

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The two files a pass writes.
const TABLE: &str = "chapters.rs";
const STATE: &str = "tuning.txt";

/// How many enemies the wave before the gap has: anything above nothing.
const A_WAVE: usize = 4;

/// How long that wave lasts, in frames: past the shortest a chapter may be, so the gap after it is one the
/// proposal's own floor on the frames since the last chapter is already clear of.
const THE_WAVE_LASTS: u32 = 180;

/// And how long the gap runs, which is past the second the detector waits into one.
const THE_GAP_LASTS: u32 = 120;

/// The frames of nothing to shoot at that a proposal is made a second into, which is
/// `tuning::ENEMY_GAP_FRAMES` — written out here for the reason every other number an e2e test holds is.
const A_SECOND_IN: i32 = 60;

/// How long a stage lasts here: past the wave, the gap, the compiled row at script 790 and the fight this
/// laid-out game scripts, so that the pass reaches a stage's end and writes what it found.
const STAGE_FRAMES: u32 = 2_000;

/// How many frames a walk here is given before it says the pass went nowhere.
const FRAMES: u32 = STAGE_FRAMES * 2;

/// The keys a boundary is stepped onto and judged with, and the one that holds the game and lets it go.
const BACK: u8 = orb_config::keys::LEFT.0;
const DROP: u8 = orb_config::keys::DOWN.0;
const HOLD: u8 = orb_config::keys::SPACE.0;

/// A launch making a pass over this game, standing in its first stage with the stage's own first chapter
/// taken — over a run where nothing is being judged, and over the game's own replay where something is.
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
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    if judging {
        game.image().the_run_reads_its_own_input();
    }
    game.frames_until("the stage's own first chapter", FRAMES, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// A wave of enemies for as long as [`THE_WAVE_LASTS`] and then none, and the script frame the boundary is
/// proposed at a second into that gap.
fn a_wave_and_a_gap(game: &Fake) -> i32 {
    game.image().enemies_alive(A_WAVE);
    game.frames(THE_WAVE_LASTS);
    let wave_ended_at = game.state().script_frames;
    game.image().enemies_alive(0);
    wave_ended_at + A_SECOND_IN
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

/// A collect pass proposes the gap in the stage's waves, begins a chapter there, and writes both files with it.
#[test]
fn a_collect_pass_over_th08_proposes_the_stages_gap_and_writes_the_table_out() {
    in_its_own_process(|| {
        let game = passing("th08-collect", false);
        let log = game.log();

        let proposed_at = a_wave_and_a_gap(&game);
        game.frames_until("the boundary the gap is", FRAMES, || {
            log.said(&format!("(script {proposed_at}): tl {proposed_at}"))
        });
        game.frames(THE_GAP_LASTS);

        // The stage over and the next one begun, which is where a pass writes what it has found — waited for
        // since here, a pass writing at every stage's start.
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

        let table = read(&game, TABLE);
        assert!(
            table.contains(&format!("proposed({proposed_at})")),
            "the boundary the pass proposed is not in the table it wrote:\n{table}",
        );
        assert!(
            table.contains("pub const MIDSTAGE: [&[Boundary]; 9] = ["),
            "the table is not this game's own nine rows:\n{table}",
        );
        let state = read(&game, STATE);
        assert!(
            state.contains(&format!("1 {proposed_at} keep proposed")),
            "the boundary was not written down for the next sitting:\n{state}",
        );
    });
}

/// And a judging pass over the game's own replay steps back onto that boundary and puts it out of the table,
/// which is what a pass is *for*: a proposal nobody looked at is a number the detector chose.
#[test]
fn a_judging_pass_over_th08_steps_onto_the_boundary_and_puts_it_out_of_the_table() {
    in_its_own_process(|| {
        let game = passing("th08-judge", true);
        let log = game.log();

        let proposed_at = a_wave_and_a_gap(&game);
        game.frames_until("the boundary the gap is", FRAMES, || {
            log.said(&format!("(script {proposed_at}): tl {proposed_at}"))
        });

        // A step back, which lands on the chapter that boundary began — strictly either side of where the game
        // is, so a frame or two past it steps back onto it.
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

        // Judged out from there, twice down: `Keep` to `Adjust` to `Rejected`.
        game.press(DROP);
        game.press(DROP);
        assert!(
            log.said(&format!("tuning: tl {proposed_at} ADJUST -> DROP")),
            "the boundary was not judged out:\n  {}",
            log.lines().join("\n  ")
        );

        // And the hold let go of, which is the same key again.
        let from = log.written();
        game.press(HOLD);
        assert!(
            log.said_since(from, "step: playing on at chapter"),
            "the hold was not let go of:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
