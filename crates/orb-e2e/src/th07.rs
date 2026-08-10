//! **A game that declines everything is a game orb does nothing to.**
//!
//! One e2e test, because one is what says it: a 妖々夢 laid out from what has been read of `th07.exe`,
//! orb attached to `Th07`, and the game's own frame run with orb's update and draw hooks inside it —
//! which is what a launch there really is, `Th07::hooks` declining `render`. What it asks is that orb
//! got in, left the game's own frame alone, and did none of what it does to 紅魔郷: no chapter, no
//! snapshot, no menu, no mark, and no overlay, each because a method of `Th07` answered nothing rather
//! than a number.
//!
//! It is the smallest thing that says the seam holds for a second game, and it fails loudly the day one
//! of those addresses is wrong — see
//! [docs/adr/0004](../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).
//!
//! What it cannot say is that the addresses *are* right: a laid-out 妖々夢 is written from the same
//! constants `Th07` reads, so a wrong one is wrong on both sides at once. Only the real image says, and
//! the two launches that asked it are in
//! [docs/adr/0004](../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).

use crate::fake::th07::Fake;
use crate::fake::{DRAW, Display, Launched, PRESENT, SOUND, UPDATE, Work, in_its_own_process};
use orb_core::frame;

/// What the game's own frame is declared to take here: about what a real run's report line shows for the
/// drawing, so that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long the run is: a few seconds of it, which is past the frames orb spends trying to build an
/// overlay and well into the frames where nothing is happening.
const FRAMES: u32 = 5 * frame::LOGIC_HZ;

#[test]
fn orb_leaves_a_game_it_declines_everything_about_alone() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-declined",
            Work::wandering(WORK_US),
        );

        // The frame is the game's own, in 妖々夢's own draw-then-update order and not orb's: the draw
        // first, the update after it, the sounds handed over and the frame presented. Which is the
        // difference `Hooks::render` being `None` makes, and the whole of what a launch there has.
        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), vec![DRAW, UPDATE, SOUND, PRESENT]);

        game.frames(FRAMES);
        let log = game.log().lines().join("\n");

        // orb got in, and got in through both of the patches `Th07::hooks` asks for: an update hook and
        // a draw hook reached is what the order above is evidence of, and the state line is orb's own
        // per-frame work having run at all.
        assert!(
            log.contains("attached to a game laid out in this process"),
            "orb did not attach:\n  {log}"
        );

        // No overlay, a 妖々夢 install having no `font.ttf` beside its exe — so nothing orb draws is
        // drawn, which is measured of the real game too.
        assert!(
            log.contains("overlay: unavailable"),
            "orb built an overlay in a directory with no font in it:\n  {log}"
        );

        // And none of what orb does to 紅魔郷. Each of these is a method of `Th07` answering nothing:
        // `read_state` says no run, `run_slot` says no run is kept, `midstage_table` is empty, and
        // `menu_pointed_at` never has anything under the cursor.
        for absent in [
            "chapter ",
            "retry",
            "resume: stage",
            "died in",
            "wash",
            "mode: asking",
        ] {
            assert!(
                !log.contains(absent),
                "orb did {absent:?} to a game it has read no run of:\n  {log}"
            );
        }

        // And it did not take the game down, which is the one thing an injected DLL must not do to
        // somebody's play session: a method that panicked would be a `panic:` line here and a game gone
        // in a real process.
        for fatal in ["panic:", "crash:"] {
            assert!(
                !log.contains(fatal),
                "orb wrote a {fatal:?} line over {FRAMES} frames:\n  {log}"
            );
        }

        // The run is the game's own from beginning to end: every frame handed over, and none of them by
        // orb — the pacing counts the frames *its* loop paces, and here it paces none.
        assert_eq!(game.handovers_us().len(), FRAMES as usize + 1);
    });
}
