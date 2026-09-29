//! **`--self-check` over th08: every chapter's snapshot restored into the state it was just taken from.**
//!
//! What `the_snapshot_of_th07_held_against_itself.rs` asks of that game, asked of this one — and the one
//! moment a restore can be held against what it should have produced without disturbing anything: the memory
//! is already what the snapshot holds, so putting it back has to leave every byte where it was. A region named
//! here is a region the snapshot saved and cannot give back, and that is a chapter which would come back wrong
//! the first time somebody died on it.
//!
//! **What this game brings of its own is the size of what a snapshot covers and what it leaves out**: its
//! twenty-one megabytes of `.data` and the five heap regions beside it, with the ranges a restore must leave
//! alone taken out — the sound manager's stream and the sheets of every `.anm` slot it has loaded. A skipped
//! range is one the check has to skip as well, so a restore that wrote into one of them, or a check that held
//! one to what the snapshot never kept, is a line here saying a region did not restore. So the fight below
//! puts its own track on the stream between two of the snapshots, which is the stream's bookkeeping moving
//! inside the ranges a restore leaves alone.
//!
//! **The untracked half of the check is empty here and that is the host's answer, not a gap** — see the
//! th07 file: what a simulated Windows answers `snapshot::fingerprint_untracked`'s walk with is nothing.

use crate::fake::th08::{Fake, THE_FIGHTS_TRACK};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// The card the boss puts up, by the record it has of its own: any card, what the test asks of it being that
/// it is one.
const A_CARD: usize = 20;

/// What a check that found nothing says, which is the whole line: the two counts of regions and the count of
/// changes in the process heap.
const NOTHING_WRONG: &str = "self_check: 0 saved region(s) did not restore, 0 untracked region(s) changed, \
     0 change(s) in the process heap";

/// **One frame of a stage placed by hand**, the run's clock stepped the way the stage's own jobs step it and
/// the boss's attack timer with it where she is up — and the script's clock left standing, so that no row of
/// the stage's table comes up under what this is about.
fn a_frame_of_the_stage(game: &Fake, attacking_for: Option<i32>) {
    let image = game.image();
    image.has_been_in_the_run_for(image.the_frames_the_run_has_been_going() + 1);
    if let Some(frames) = attacking_for {
        image.the_boss_has_attacked_for(0, frames);
    }
    game.frame();
}

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
fn every_chapter_of_th08_gives_back_the_state_its_snapshot_was_taken_from() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-self-check",
            Work::wandering(WORK_US),
            |config| {
                config.self_check = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        for _ in 0..FRAMES_FOR_A_BUILD {
            if log.said("stage 1 chapter 1 (stage start)") {
                break;
            }
            a_frame_of_the_stage(&game, None);
        }
        assert!(
            log.said("stage 1 chapter 1 (stage start)"),
            "the stage's first chapter was never taken:\n  {}",
            log.lines().join("\n  ")
        );

        // The stage's own start and the two chapters of the fight after it — her arrival to her own track, and
        // her card — each of them a snapshot taken and put straight back.
        for _ in 0..APART {
            a_frame_of_the_stage(&game, None);
        }
        game.image().a_boss_is_fighting(0, (192.0, 96.0), 1000);
        game.image().the_stream_reads_the_track(THE_FIGHTS_TRACK);
        for frame in 0..APART {
            a_frame_of_the_stage(&game, Some(frame as i32));
        }
        assert!(
            log.said("stage 1 chapter 2 at frame"),
            "the boss's arrival took no chapter:\n  {}",
            log.lines().join("\n  ")
        );
        game.image().a_card_is_up(A_CARD);
        let mut attacked = 0;
        while !log.said("stage 1 chapter 3 at frame") {
            assert!(
                attacked < APART,
                "the card took no chapter:\n  {}",
                log.lines().join("\n  ")
            );
            a_frame_of_the_stage(&game, Some(attacked as i32));
            attacked += 1;
        }
        let at_the_card = game.state();

        // A check for each of the three snapshots the run has taken, and every one of them found nothing: a
        // region that would not restore is named on the line under the counts, so a run of these lines all
        // reading zero is every region of every snapshot so far giving back what it was taken from.
        let (taken, wrong) = checks(&game);
        assert!(
            taken >= 3,
            "three snapshots were taken and only {taken} of them were checked",
        );
        assert!(
            wrong.is_empty(),
            "a snapshot did not give back the state it was taken from:\n  {}",
            wrong.join("\n  ")
        );

        // And the restore a run really asks for, which is the one the check above is evidence about: the hit,
        // チャプターをやり直す, and the run back on the frame the card's chapter began on with the card up.
        for frame in attacked..attacked + APART {
            a_frame_of_the_stage(&game, Some(frame as i32));
        }
        game.image().has_died(1.0);
        game.image().the_player_has_been_hit();
        game.frames_until("the death seen", 8, || log.said("died in chapter 3"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let back = game.state();
        assert_eq!(
            (back.stage_frames, back.spellcard, back.deaths),
            (
                at_the_card.stage_frames,
                at_the_card.spellcard,
                at_the_card.deaths
            ),
            "the chapter did not come back where it began",
        );

        // The check keeps saying nothing is wrong after a restore has really happened, which is what says the
        // run of zeroes above is not a snapshot that never held anything. The card played on first: the
        // restore put her attack's timer back to where the card's chapter began it, so the nonspell after the
        // card is a timer going back to nought only once the card has run a while again.
        for frame in attacked..attacked + APART {
            a_frame_of_the_stage(&game, Some(frame as i32));
        }
        game.image().no_card_is_up();
        let mut frames = 0;
        while checks(&game).0 <= taken {
            assert!(
                frames < APART * 2,
                "no chapter was taken past the retry:\n  {}",
                log.lines().join("\n  ")
            );
            a_frame_of_the_stage(&game, Some(frames as i32));
            frames += 1;
        }
        let (_, wrong) = checks(&game);
        assert!(
            wrong.is_empty(),
            "a snapshot taken after a restore did not give back its own state:\n  {}",
            wrong.join("\n  ")
        );
    });
}
