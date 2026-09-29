//! **The address space a chapter of th06nc would put back is held for as long as that chapter is kept.**
//!
//! What `the_address_space_a_chapter_of_th06c_needs.rs` asks of that game, asked of this one: a snapshot is a
//! copy of ranges named by where they are, and what a copy cannot hold is the address space — a block freed
//! inside the game's heap is handed back to the OS by `ntdll`, which no hook on the exe's imports sees, and
//! whoever is given that space next is who a restore of the older snapshot writes over. The keeping is one
//! mechanism over every game; what this game brings is its own chapters and its own heap regions, which are
//! more than th06c's — `a_span_of_th06nc_held_back_by_its_source.rs` names them — so a range left unnamed here
//! is a restore writing into memory the game no longer owns.

use crate::fake::th06nc::{Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a run given up is given to have left its stage.
const LEAVES_WITHIN: u32 = 1_800;

/// A Pointdevice mode run of this game in its first stage, played to its first chapter, with nothing fired at
/// the player — the one death here is the one taken to give the run up.
fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game.frames_until("the stage's own first chapter", MIDBOSS_ARRIVES, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The hit and then タイトルに戻る, which is the way out of a run that ends it where it stands.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    game.hit();
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

#[test]
fn the_ranges_a_chapter_of_th06nc_would_put_back_are_named_while_it_is_kept() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-the-address-space-a-chapter-needs");
        let log = game.log();

        // The stage's own start is a chapter, and a chapter is a snapshot: what it would put back is every
        // range of the game this launch laid out.
        let named = game.sim().what_a_restore_needs();
        assert!(
            !named.is_empty(),
            "the chapter kept at the stage's start named no range a restore of it would need:\n  {}",
            log.lines().join("\n  ")
        );

        // A second chapter is a second snapshot, and it names them again: one going must not take another's
        // word with it.
        game.frames_until("the midboss's chapter", MIDBOSS_BEATEN, || {
            log.said("chapter 2 at frame")
        });
        let two = game.sim().what_a_restore_needs();
        assert!(
            two.len() > named.len(),
            "the second chapter named nothing of its own: {} range(s) before it, {} after:\n  {}",
            named.len(),
            two.len(),
            log.lines().join("\n  ")
        );

        // And a run that is over needs none of it: every chapter of it has gone.
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
