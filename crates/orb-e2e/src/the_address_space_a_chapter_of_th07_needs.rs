//! **The address space a chapter of th07 would put back is held for as long as that chapter is kept.**
//!
//! A snapshot is a copy of ranges of memory, named by where they are. What a copy cannot hold is the
//! *address space*: a block of a few megabytes is a reservation of its own inside the game's heap, so the
//! game freeing one hands those pages back to the OS — and the free that does it is the heap manager's
//! own, inside `ntdll`, which no hook on the exe's imports sees. Whoever is given that space next is who
//! a restore of the older snapshot writes over.
//!
//! **Both of the two things that took it have taken th07 down.** A step back at a stage's end faulted
//! inside the next snapshot's own copy, `chapter::take`'s memcpy loop walking a `saved` list whose slice
//! pointer read as 8 — orb's own buffer had been given the game's freed pages and then written over. The
//! step back after that faulted inside `ntdll`'s heap with `take` under it, the process heap having taken
//! them instead. Neither is a list orb can finish: what is left to do about it is to stop the space from
//! coming free at all.
//!
//! So orb tells the host, per snapshot, which ranges a restore of it still needs, and the host keeps the
//! space the moment the game gives it up. **What is asked here is the telling** — which ranges, and for
//! how long — that being the half the decision is in. The keeping itself is a reservation over pages a
//! laid-out game does not have: `Sim::hold_what_a_restore_needs` says so, and `orb::memtrack` is where the
//! one call that asks for it lives, beside the import hooks it belongs with.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames between one part of a fight and the next: past the shortest a chapter may be, so the second
/// one is a chapter of its own.
const APART: u32 = 120;

/// A Pointdevice mode run of this game with its stage's own track streaming, played to its first chapter.
fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game.frames_until("the stage's own first chapter", 600, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

#[test]
fn the_ranges_a_chapter_of_th07_would_put_back_are_named_while_it_is_kept() {
    in_its_own_process(|| {
        let game = in_a_run("th07-the-address-space-a-chapter-needs");
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
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the fight's own chapter", APART, || log.said("chapter 2"));
        let two = game.sim().what_a_restore_needs();
        assert!(
            two.len() > named.len(),
            "the second chapter named nothing of its own: {} range(s) before it, {} after",
            named.len(),
            two.len(),
        );

        // And a run that is over needs none of it: every chapter of it has gone, so the address space is
        // the game's to hand back and nobody's to keep.
        game.gives_the_run_up();
        assert!(
            game.sim().what_a_restore_needs().is_empty(),
            "a run that has ended still needs the address space of its chapters:\n  {:?}",
            game.sim().what_a_restore_needs(),
        );
    });
}
