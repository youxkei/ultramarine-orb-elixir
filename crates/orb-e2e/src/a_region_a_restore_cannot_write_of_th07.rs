//! **A region of a chapter of th07 that cannot be made writable: named, and the rest of the restore still
//! happens.**
//!
//! The same claim `a_region_a_restore_cannot_write.rs` makes of th06, over the regions *this* game is in — a
//! snapshot of th07 is its `.data` and the blocks off the allocator beside it, and a chapter put back except
//! for one of them is a run to carry on from, where a restore that gave up at the first refusal would leave the
//! game with half of one chapter and half of another.
//!
//! Why the commit happens outside the write loop at all is beside that loop: it runs with the game's other
//! threads suspended, and `VirtualAlloc` is exactly the call that must not happen there.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between the stage's own start and the card: past the shortest a chapter may be.
const APART: u32 = 120;

#[test]
fn a_region_of_th07_that_cannot_be_committed_is_named_and_the_chapter_still_comes_back() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-region-refused",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || {
            log.said("chapter 2") && log.said("a midboss spellcard")
        });
        let at_the_card = game.state();

        // One of the regions a snapshot of this game covers, declared un-committable — the last of them rather
        // than the first, the first being the game's own data and a run that lost that would be no run at all.
        // Which region it is is the game's to say: what the claim is about is that *a* region refusing is
        // survivable, and the list comes out of the same walk the snapshot's does.
        let regions = game.sim().space().game_regions(&game.image().data());
        let (base, len, _) = *regions
            .last()
            .expect("a game laid out in more than one region");
        assert_ne!(
            regions.len(),
            1,
            "this game is one region, so the one refused would be its data",
        );
        game.sim().space().refuses_to_commit(base);

        // The hit and チャプターをやり直す, which is the restore.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 2"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // The region is named, with the address and the length, so that whoever reads the log can go and look
        // at what is there.
        assert!(
            log.said(&format!("restore: cannot write {base:#010x}+{len:#x}")),
            "the region that could not be made writable was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // And the chapter came back all the same: the run is on the frame it began at, and the game is still
        // being played.
        //
        // The frame and not every field, because the region refused is one of the game's own and may be the one
        // this run's numbers are in — which is the whole point of the claim. What a refusal costs is whatever was
        // in that region; what it must not cost is the restore.
        assert_eq!(
            game.state().stage_frames,
            at_the_card.stage_frames,
            "the restore gave up at the region it could not write",
        );
        let from = game.state().stage_frames;
        game.frames(30);
        assert_eq!(
            game.state().stage_frames,
            from + 30,
            "the game stopped being played after a region refused to commit",
        );
        for fatal in ["panic:", "crash:"] {
            assert!(
                !log.said(fatal),
                "a region that could not be committed took the game down:\n  {}",
                log.lines().join("\n  ")
            );
        }
    });
}
