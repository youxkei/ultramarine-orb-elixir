//! **The address space a chapter of th06 would put back is held for as long as that chapter is kept.**
//!
//! A snapshot is a copy of ranges of memory, named by where they are. What a copy cannot hold is the
//! *address space*: a block of a few megabytes is a reservation of its own inside the game's heap, so the
//! game freeing one hands those pages back to the OS — and the free that does it is the heap manager's
//! own, inside `ntdll`, which no hook on the exe's imports sees. Whoever is given that space next is who
//! a restore of the older snapshot writes over.
//!
//! What took th07 down twice is in `the_address_space_a_chapter_of_th07_needs.rs`, and the keeping is one
//! mechanism over both games: a snapshot of this game names its ranges through the same call, so what is
//! asked here is the same telling — which ranges, and for how long — over this game's chapters. The
//! keeping itself is a reservation over pages a laid-out game does not have: `Sim::hold_what_a_restore_needs`
//! says so, and `orb::memtrack` is where the one call that asks for it lives, beside the import hooks it
//! belongs with.

use crate::fake::th06::{BOSS_ARRIVES, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// The hit and then タイトルに戻る, which is the way out of a run that ends it where it stands: the run's
/// chapters go with it, and the file it was written down in is left for the next launch to offer.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.dies();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
}

#[test]
fn the_ranges_a_chapter_would_put_back_are_named_while_it_is_kept() {
    in_its_own_process(|| {
        let game = Fake::attach("the-address-space-a-chapter-needs", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.in_a_pointdevice_run();

        // The stage's own start is a chapter, and a chapter is a snapshot: what it would put back is every
        // range of the game this launch laid out.
        let named = game.sim().what_a_restore_needs();
        assert!(
            !named.is_empty(),
            "the chapter kept at the stage's start named no range a restore of it would need:\n  {}",
            log.lines().join("\n  ")
        );

        // A second chapter is a second snapshot, and it names them again: what says a range still matters
        // is that a snapshot naming it is still there, so one going must not take another's word with it.
        game.frames_until("the midboss's chapter", BOSS_ARRIVES + 400, || {
            log.said(&format!("chapter 2 at frame {BOSS_ARRIVES}"))
        });
        let two = game.sim().what_a_restore_needs();
        assert!(
            two.len() > named.len(),
            "the second chapter named nothing of its own: {} range(s) before it, {} after:\n  {}",
            named.len(),
            two.len(),
            log.lines().join("\n  ")
        );

        // And a run that is over needs none of it: every chapter of it has gone, so the address space is
        // the game's to hand back and nobody's to keep. The run's own way out here goes through the game's
        // ranking, and what the chapters are dropped with is the run ending rather than that screen — so
        // this is waited for as the run ending and not as the screen coming down.
        gives_the_run_up(&game);
        game.frames_until("the run ended", 60, || log.said("run ended after"));
        assert!(
            game.sim().what_a_restore_needs().is_empty(),
            "a run that has ended still needs the address space of its chapters:\n  {:?}\n  {}",
            game.sim().what_a_restore_needs(),
            log.lines().join("\n  ")
        );
    });
}
