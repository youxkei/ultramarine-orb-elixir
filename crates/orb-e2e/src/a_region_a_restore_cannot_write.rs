//! **A region of a chapter's snapshot that cannot be made writable: named, and the rest of the restore
//! still happens.**
//!
//! A snapshot holds the pages a stage was in, and by the time it is put back some of them may not be there:
//! the game freeing a few megabytes does it. So the pages are committed before the write loop — outside it,
//! because that loop runs with the game's other threads suspended and `VirtualAlloc` is exactly the call
//! that must not happen there.
//!
//! What is left is a commit that comes back refusing. **The refusal cannot stop the restore**: a chapter put
//! back except for one region is a run to carry on from, and a restore that gave up at the first refusal
//! would leave the game with half of one chapter and half of another. So the region goes on a list, every
//! other one is written, and the list is logged afterwards — after the copy, since nothing between the
//! suspend and the resume may allocate and a `log!` does.

use crate::fake::th06::{CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

#[test]
fn a_region_that_cannot_be_committed_is_named_and_the_chapter_still_comes_back() {
    in_its_own_process(|| {
        let game = Fake::attach("a-region-refused", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.in_a_pointdevice_run();
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });
        let at_the_card = game.state().stage_frames;

        // One of the regions a snapshot of this game covers, declared un-committable — the last of them
        // rather than the first, the first being the game's own data and a run that lost that would be no
        // run at all. Which region it is is the game's to say: what the claim is about is that *a* region
        // refusing is survivable, and the list comes out of the same walk the snapshot's does.
        let regions = game.sim().space().game_regions(&game.image().data());
        let (base, len) = *regions
            .last()
            .expect("a game laid out in more than one region");
        assert_ne!(
            regions.len(),
            1,
            "this game is one region, so the one refused would be its data",
        );
        game.sim().space().refuses_to_commit(base);

        // 被弾 and チャプターをやり直す, which is the restore.
        game.hit();
        game.frame();
        assert!(log.said("died in chapter 3"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });

        // The region is named, with the address and the length, so that whoever reads the log can go and
        // look at what is there.
        assert!(
            log.said(&format!("restore: cannot write {base:#010x}+{len:#x}")),
            "the region that could not be made writable was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // And the chapter came back all the same: the run is on the frame it began at, and the game is
        // still being played.
        assert_eq!(
            game.state().stage_frames,
            at_the_card,
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
