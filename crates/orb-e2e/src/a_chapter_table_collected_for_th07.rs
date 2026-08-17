//! **`--collect` and `--judge` over 妖々夢: the midstage chapter table built out of a replay.**
//!
//! A fight's boundaries are found from the game as it is fought, but a stage's waves are a script on a clock, so
//! those boundaries are frame numbers somebody has to choose. `--collect` proposes them — a second into each gap
//! between waves, with nothing to shoot at and no boss to interrupt — and `--judge` holds the game on each one so
//! it can be looked at and corrected. Both write two files beside the game: `chapters.rs`, to paste into the
//! source, and `tuning.txt`, which is read back at startup so that a stage can be judged over more than one
//! sitting.
//!
//! **This is the pass 妖々夢 is waiting for.** `Th07::midstage_table` is eight empty rows, so a chapter of this
//! game falls at a stage's start and nowhere inside one until a run with `--chapter-tuning` fills them — which is
//! the one piece of work in `docs/todo/th07-the-run.md` that cannot be read out of the exe. What this e2e test
//! says is that the pass works over this game: the gap is proposed out of *its* enemy count and *its* script
//! clock, the chapter begins there, and both files come out with the row in them.
//!
//! The waves are driven from here rather than laid out as a script, which is what a laid-out 妖々夢 has: what the
//! detector reads is the count of enemies alive and the boss's slot, so a wave is that count going up and a gap is
//! it going back to nothing.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The two files a pass writes.
const TABLE: &str = "chapters.rs";
const STATE: &str = "tuning.txt";

/// The two keys a boundary is stepped onto and judged with. orb's own names for them are in its `keys` module;
/// these are the numbers, which is what an e2e test presses.
const BACK: u8 = orb_config::keys::LEFT.0;
const DROP: u8 = orb_config::keys::DOWN.0;
/// And the one that holds the game and lets it go again, which is a toggle: a pass judges while standing on a
/// boundary and carries on with the same key.
const HOLD: u8 = orb_config::keys::SPACE.0;

/// How many enemies the wave before the gap has: anything above nothing, the detector asking only whether there
/// is something to shoot at.
const A_WAVE: i32 = 4;

/// How long that wave lasts, in frames: past the shortest a chapter may be, so the gap after it is one the
/// proposal's own floor on the frames since the last chapter is already clear of.
const THE_WAVE_LASTS: u32 = 180;

/// And how long the gap runs before the wave after it, which is past the second the detector waits into one.
const THE_GAP_LASTS: u32 = 120;

/// The frames of nothing to shoot at that a proposal is made a second into, which is `tuning::ENEMY_GAP_FRAMES`
/// — written out here for the reason every other number an e2e test holds is: a test that read it off the code
/// under test would agree with it whatever either said.
const A_SECOND_IN: i32 = 60;

/// A launch making a pass over a replay of its own stages.
///
/// **The song is laid out because of what waiting for it costs**: a stage's first chapter is taken once the stage
/// has settled *and* the music has come up, and a laid-out game with no sound spends the whole of that wait —
/// which is frames the gap below would be measured from the middle of.
fn passing(name: &str, judging: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_tuning = true;
            config.during_replay = true;
            config.chapter_stepping = judging;
            config.log_level = LogLevel::Verbose;
        },
    );
    let log = game.log();
    game.streams_its_song(0);
    game.image().a_replay_covers_the_stage(0);
    game.image().a_replay_covers_the_stage(1);
    game.watches_a_replay();
    game.frames_until("the stage's own first chapter", 600, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// What a pass wrote into one of its two files.
///
/// # Panics
/// Where the file is not there, which is a pass that wrote nothing.
fn read(game: &Fake, file: &str) -> String {
    let path = game.dir().join(file);
    game.sim()
        .files()
        .text(&path)
        .unwrap_or_else(|| panic!("{} was not written", path.display()))
}

/// A collect pass proposes the gap in the stage's waves, begins a chapter there, and writes both files with it.
#[test]
fn a_collect_pass_over_th07_proposes_the_stages_gap_and_writes_the_table_out() {
    in_its_own_process(|| {
        let game = passing("th07-collect", false);
        let log = game.log();

        // A wave, and then nothing to shoot at: what the detector reads is this game's own count of enemies
        // alive, which its manager's walk of its slots leaves.
        game.image().enemies_alive(A_WAVE);
        game.frames(THE_WAVE_LASTS);
        let wave_ended_at = game.state().script_frames;
        game.image().enemies_alive(0);

        // The boundary, proposed a second into that gap and acted on at once: a proposal is a boundary of the
        // table, so the chapter begins there on the same update.
        let proposed_at = wave_ended_at + A_SECOND_IN;
        game.frames_until("the boundary the gap is", 600, || {
            log.said(&format!("(script {proposed_at}): tl {proposed_at}"))
        });

        // And the gap itself is reported when the next wave arrives, whether or not a boundary was taken in it:
        // what the second a proposal waits should be is a question about this game's waves, so a pass over a run
        // is the measurement rather than the number being guessed at.
        //
        // The frames of it are counted from the first with nothing to shoot at, which is the frame after the wave
        // was taken away.
        game.frames(THE_GAP_LASTS);
        game.image().enemies_alive(A_WAVE);
        let gap_began_at = wave_ended_at + 1;
        game.frames_until("the wave after the gap", 60, || {
            log.said(&format!("frames, script {gap_began_at}.."))
        });

        // The stage over and the next one begun, which is where a pass writes what it has found: the stage just
        // finished is worth keeping even if the pass stops there.
        //
        // Waited for since here rather than looked for, because a pass writes at every stage's start: the write
        // this is about is the one that has the boundary in it, and the first one went out before the stage had
        // been played at all.
        let from = log.written();
        game.moves_on_to_the_stage(1);
        game.frames_until("the table written", 600, || {
            log.said_since(
                from,
                &format!("tuning: wrote {}", game.dir().join(TABLE).display()),
            )
        });

        // The table as Rust to paste into `chapters.rs`, with this boundary in stage one's row and marked as the
        // detector's rather than as somebody's hand — which the shortest a chapter may be reads, so a table that
        // only said it in a comment would divide the stage differently in play than in the pass that chose it.
        let table = read(&game, TABLE);
        assert!(
            table.contains(&format!("proposed({proposed_at})")),
            "the boundary the pass proposed is not in the table it wrote:\n{table}",
        );
        // And the row is the one whose stage this is, in a table of this game's own eight: what 妖々夢 has
        // compiled in is nothing at all, so every other row comes out empty rather than carrying somebody else's
        // numbers.
        assert!(
            table.contains("pub const MIDSTAGE: [&[Boundary]; 8] = ["),
            "the table is not this game's own eight rows:\n{table}",
        );
        assert_eq!(
            table.matches("proposed(").count(),
            1,
            "a stage this pass never reached came out with a boundary in it:\n{table}",
        );

        // And the same boundary in the form the next sitting reads back.
        let state = read(&game, STATE);
        assert!(
            state.contains(&format!("1 {proposed_at} keep proposed")),
            "the boundary was not written down for the next sitting:\n{state}",
        );
    });
}

/// And a judging pass steps back onto that boundary and puts it out of the table, which is what a pass is *for*:
/// a proposal nobody looked at is a number the detector chose.
///
/// The keys are orb's own and `a_chapter_table_collected.rs` holds them over 紅魔郷; what this asks of 妖々夢 is
/// the half that is the game's — the step landing on the frame out of this game's own script clock, and the
/// boundary that is judged out no longer beginning a chapter of it.
#[test]
fn a_judging_pass_over_th07_steps_onto_the_boundary_and_puts_it_out_of_the_table() {
    in_its_own_process(|| {
        let game = passing("th07-judge", true);
        let log = game.log();

        game.image().enemies_alive(A_WAVE);
        game.frames(THE_WAVE_LASTS);
        let wave_ended_at = game.state().script_frames;
        game.image().enemies_alive(0);
        let proposed_at = wave_ended_at + A_SECOND_IN;
        game.frames_until("the boundary the gap is", 600, || {
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

        // And judged out from there, twice down: `Keep` to `Adjust` to `Rejected`, which is what a boundary
        // somebody looked at and did not want comes to.
        game.press(DROP);
        game.press(DROP);
        assert!(
            log.said(&format!("tuning: tl {proposed_at} ADJUST -> DROP")),
            "the boundary was not judged out:\n  {}",
            log.lines().join("\n  ")
        );

        // The hold let go of, which is the same key again: a pass that judged a boundary carries on from it, and
        // the frames after it are what reach the end of the stage.
        let from = log.written();
        game.press(HOLD);
        assert!(
            log.said_since(from, "step: playing on at chapter"),
            "the hold was not let go of:\n  {}",
            log.lines().join("\n  ")
        );

        // The stage over, and the file says so: a boundary judged out is written down as one, so the next sitting
        // does not have to look at it again. Since here, a pass writing at every stage's start.
        let from = log.written();
        game.moves_on_to_the_stage(1);
        game.frames_until("the state written", 600, || {
            log.said_since(
                from,
                &format!("tuning: wrote {}", game.dir().join(STATE).display()),
            )
        });
        let state = read(&game, STATE);
        assert!(
            state.contains(&format!("1 {proposed_at} drop proposed")),
            "the boundary judged out was not written down as one:\n{state}",
        );
        // And it is out of the table as well, which is what the pass is writing: the row it wrote has nothing in
        // it, the one boundary it found having been judged out.
        let table = read(&game, TABLE);
        assert!(
            !table.contains(&format!("proposed({proposed_at})")),
            "a boundary judged out is still in the table:\n{table}",
        );
    });
}
