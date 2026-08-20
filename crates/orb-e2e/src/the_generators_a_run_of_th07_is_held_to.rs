//! **The generator's own word on every frame a run of 妖々夢 played, which is what says a playback has come
//! out of step with the run it is reproducing — and what a file from before orb wrote them down gets instead.**
//!
//! The buttons cannot say it. A playback that has parted plays the same presses onto a different stage and
//! carries on: what shows up is a death thousands of frames later, or the landing check at the end, and
//! neither says which frame it went wrong on. The generator does — one word a frame, held against what the
//! game's generator is on that frame as the buttons go in.
//!
//! **And a file may have no word for a frame**, which is what this is about. `VERSION` stayed at 2 when the
//! generators were added, so that a run left unfinished before orb wrote them down is still offered rather
//! than refused — such a file has nothing to hold its playback to, and a playback of it is fed its buttons and
//! says nothing about where it parts. What must not follow is that the *next* file says nothing either: the
//! list was one that had to be whole, so a 妖々夢 run picked up out of one of those wrote `generators: []` at
//! every chapter after it, and the run was played for a session with the one check that names a parting frame
//! switched off and nothing in the log saying so. Both ends of that are here — what the file the run leaves
//! carries, and the line each end of a file says it with.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between the stage's own start and the card, and between that card and the attack after it: past
/// the shortest a chapter may be, so each is a chapter of its own with frames of buttons behind it.
const APART: u32 = 120;

/// The name orb gives the run this plays, which is what names its file: `DIFFICULTY`, `CHARACTER` and `SHOT`
/// are 1, 0 and 0 — Normal, and 霊夢's first shot. Written out rather than asked of `Th07::run_slot`, which
/// would be this agreeing with the code it is about.
const SLOT: &str = "normal-reimu-a";

/// What the line a chapter is written down with says about the words, at both ends: the writer says it of the
/// file it is handing over, and a launch says it of the file it picked up.
const WITH_THEM: &str = "and the generator each was played with";
const WITHOUT_THEM: &str = "and no generator for any of them";

/// A run picked up out of a file from before the generators plays its frames back with nothing to hold them
/// to — and the file it leaves at the next chapter says what the generator was on every frame the player
/// played after that.
///
/// **The fed frames keep having no word, and that is the point.** What a file holds is the run's own
/// generator, and a playback's is what is held *against* it: a word written down for a fed frame would go into
/// every file after this one as the run's, and the next playback of those frames would be held against
/// whatever this one did rather than against the run. What must not follow from a frame with no word is a file
/// with nothing in it at all, which is what the list being all-or-none made of one — the parting check off for
/// every chapter the run went on to write.
#[test]
fn a_run_of_th07_picked_up_out_of_a_file_from_before_the_generators_writes_them_down() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-generators-from-before",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // A chapter with frames behind it, so that the file it is written in holds buttons to be played back.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!(
                "at frame {at}, {at} frame(s) of buttons {WITH_THEM}"
            ))
        });

        // And that file made into the file every disk held when orb learnt to write the words down.
        assert!(
            orb_core::resume::take_the_generators_out(game.dir(), SLOT),
            "the file was not rewritten as one from before the generators",
        );
        game.gives_the_run_up();
        game.picks_the_run_up();
        assert!(
            log.said(WITHOUT_THEM),
            "the run was not picked up out of a file from before the generators:\n  {}",
            log.lines().join("\n  "),
        );

        // The chapter after the one the run came back at, which is the file the next launch would pick up.
        let from = log.written();
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the chapter after the landing", APART, || {
            log.said_since(from, "a midboss spellcard")
        });
        let then = game.state().stage_frames;
        let written = format!("at frame {then}, {then} frame(s) of buttons");
        game.frames_until_a_thread("the chapter after the landing written down", || {
            log.said_since(from, &written)
        });
        // Every frame the player played, and none of the ones that were fed: the fed frames are the file's own
        // and it had no word for them, and this playback's generator is not the run's to write down.
        assert!(
            log.said_since(
                from,
                &format!("{written} and the generator on {} of them", then - at),
            ),
            "the file this run leaves does not say what the generator was on the {} frame(s) it played:\n  {}",
            then - at,
            log.lines().join("\n  "),
        );
    });
}
