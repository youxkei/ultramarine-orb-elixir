//! **The address space a chapter of th10 would put back is held for as long as that chapter is kept.**
//!
//! What `the_address_space_a_chapter_of_th08_needs.rs` asks of that game — why a copy of memory cannot hold
//! the address space it was taken out of is written in th07's — asked of this one. A snapshot of this game
//! covers its `.data` and the heap blocks beside it, so the ranges a restore needs are the blocks the game
//! allocates its stage's objects in, and every one of them is a range the game's own frees could hand back to
//! the OS while a chapter still names it.
//!
//! So orb tells the host, per snapshot, which ranges a restore of it still needs, and the host keeps the space
//! the moment the game gives it up. **What is asked here is the telling** — which ranges, and for how long —
//! that being the half the decision is in; the keeping is a reservation over pages a laid-out game does not
//! have, `Sim::hold_what_a_restore_needs` says so.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the laid-out stage lasts, which is past its midboss's arrival at frame 1200: her first attack is the
/// stage's second chapter.
const A_STAGE: u32 = 4_500;

/// A Pointdevice mode run of this game placed in its first stage, played to its first chapter, with nothing
/// fired at the player — the one death here is the one taken to give the run up.
fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.stages_last(A_STAGE);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    let log = game.log();
    game.frames_until("the stage's first chapter", A_STAGE, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The hit and then タイトルに戻る, which is the way out of a run that ends it where it stands: the run's
/// chapters go with it.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.the_player_is_hit();
    game.frames_until("the death seen", 16, || log.said("died in chapter"));
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

/// How many frames a run given up is given to have ended: the way out is the game's own, which the supervisor
/// walks over frames of its own.
const LEAVES_WITHIN: u32 = 1_800;

#[test]
fn the_ranges_a_chapter_of_th10_would_put_back_are_named_while_it_is_kept() {
    in_its_own_process(|| {
        let game = in_a_run("th10-the-address-space-a-chapter-needs");
        let log = game.log();

        // The stage's own start is a chapter, and a chapter is a snapshot: what it would put back is every
        // range of the game this launch laid out.
        let named = game.sim().what_a_restore_needs();
        assert!(
            !named.is_empty(),
            "the chapter kept at the stage's start named no range a restore of it would need",
        );

        // A second chapter is a second snapshot, and it names them again: what says a range still matters
        // is that a snapshot naming it is still there, so one going must not take another's word with it.
        game.frames_until("the midboss's first attack", A_STAGE, || {
            log.said("stage 1 chapter 2 ")
        });
        let two = game.sim().what_a_restore_needs();
        assert!(
            two.len() > named.len(),
            "the second chapter named nothing of its own: {} range(s) before it, {} after",
            named.len(),
            two.len(),
        );

        // And a run that is over needs none of it: every chapter of it has gone, so the address space is the
        // game's to hand back and nobody's to keep.
        gives_the_run_up(&game);
        game.frames_until("the run ended", LEAVES_WITHIN, || {
            log.said("run ended after")
        });
        assert!(
            game.sim().what_a_restore_needs().is_empty(),
            "a run that has ended still needs the address space of its chapters:\n  {:?}\n  {}",
            game.sim().what_a_restore_needs(),
            log.lines().join("\n  ")
        );
    });
}
