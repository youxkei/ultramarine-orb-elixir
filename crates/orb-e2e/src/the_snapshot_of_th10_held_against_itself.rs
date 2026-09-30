//! **`--self-check` over th10: every chapter's snapshot restored into the state it was just taken from.**
//!
//! What `the_snapshot_of_th08_held_against_itself.rs` asks of that game, asked of this one — and the one
//! moment a restore can be held against what it should have produced without disturbing anything: the memory
//! is already what the snapshot holds, so putting it back has to leave every byte where it was. A region named
//! here is a region the snapshot saved and cannot give back, and that is a chapter which would come back wrong
//! the first time somebody died on it.
//!
//! **What this game brings of its own is what a snapshot covers and what it leaves out**: its `.data` and the
//! heap blocks beside it, with the ranges a restore must leave alone taken out — the sound's device, its stream
//! and the stream's buffer, and the sheets of every `.anm` file it has loaded. A skipped range is one the check
//! has to skip as well, so a restore that wrote into one of them, or a check that held one to what the snapshot
//! never kept, is a line here saying a region did not restore. So the stage below is played into its boss's
//! fight, who puts her own track on the stream between two of the snapshots, which is the stream's bookkeeping
//! moving inside the ranges a restore leaves alone.
//!
//! **The untracked half of the check is empty here and that is the host's answer, not a gap** — see the
//! th07 file: what a simulated Windows answers `snapshot::fingerprint_untracked`'s walk with is nothing.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process, the_frame_chapter_began_on};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the laid-out stage lasts: its midboss's two attacks from frame 1200 its second and third chapters,
/// the waves after her its fourth, and the boss's nonspell, card and nonspell — under her own track — its fifth
/// to seventh, she arriving for the last 900 frames.
const A_STAGE: u32 = 3_600;

/// The frames into the boss's card the run is lost at: past the shortest a chapter may be.
const INTO_THE_CARD: u32 = 120;

/// What a check that found nothing says, which is the whole line: the two counts of regions and the count of
/// changes in the process heap.
const NOTHING_WRONG: &str = "self_check: 0 saved region(s) did not restore, 0 untracked region(s) changed, \
     0 change(s) in the process heap";

/// The self-check lines the log holds, and those of them that found something.
fn checks(game: &Fake) -> (usize, Vec<String>) {
    let lines: Vec<String> = game
        .log()
        .lines()
        .into_iter()
        .filter(|line| line.contains("self_check: "))
        .collect();
    let wrong = lines
        .iter()
        .filter(|line| !line.contains(NOTHING_WRONG))
        .cloned()
        .collect();
    (lines.len(), wrong)
}

#[test]
fn every_chapter_of_th10_gives_back_the_state_its_snapshot_was_taken_from() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-self-check",
            Work::wandering(WORK_US),
            |config| {
                config.self_check = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();
        game.its_title_menu_is_up();
        game.is_in_a_run();

        // The stage's own start, its midboss's two attacks, the waves after her and the boss's nonspell under
        // her own track and her card — each of them a snapshot taken and put straight back.
        game.frames_until("the boss's card", A_STAGE, || {
            log.said("stage 1 chapter 6 ")
        });
        let at_the_card = game.state();
        let began = the_frame_chapter_began_on(&log.lines(), 1, 6);

        // A check for each of the six snapshots the run has taken, and every one of them found nothing: a
        // region that would not restore is named on the line under the counts, so a run of these lines all
        // reading zero is every region of every snapshot so far giving back what it was taken from.
        let (taken, wrong) = checks(&game);
        assert!(
            taken >= 6,
            "six snapshots were taken and only {taken} of them were checked",
        );
        assert!(
            wrong.is_empty(),
            "a snapshot did not give back the state it was taken from:\n  {}",
            wrong.join("\n  ")
        );

        // And the restore a run really asks for, which is the one the check above is evidence about: the hit,
        // チャプターをやり直す, and the run back on the frame the card's chapter began on with the card up.
        game.frames(INTO_THE_CARD);
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 6"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        // **The frame and the card and not the count of deaths**, which over this game is orb's own count of
        // the hits and no byte of the game's — see `Th10`'s `DEATHS` — so a restore of the game's memory does
        // not take it back.
        let back = game.state();
        assert!(
            (began..=began + 1).contains(&back.stage_frames)
                && back.spellcard == at_the_card.spellcard,
            "the chapter did not come back where it began: frame {} with {:?}, where it began on frame {began} \
             with {:?}",
            back.stage_frames,
            back.spellcard,
            at_the_card.spellcard,
        );

        // The check keeps saying nothing is wrong after a restore has really happened, which is what says the
        // run of zeroes above is not a snapshot that never held anything: the card played out and the nonspell
        // after it a chapter of its own.
        game.frames_until("the chapter after the card", A_STAGE, || {
            checks(&game).0 > taken
        });
        let (_, wrong) = checks(&game);
        assert!(
            wrong.is_empty(),
            "a snapshot taken after a restore did not give back its own state:\n  {}",
            wrong.join("\n  ")
        );
    });
}
