//! **The brush stroke over the count of lives, and the frames either end of a run it has to reach.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine and seen on the
//! screen.
//!
//! What the stroke *is* has tests already — `lives_ui.rs`'s `the_stroke_covers_the_count_and_nothing_else`,
//! `the_row_itself_is_left_to_the_game`, `the_stroke_over_the_row_is_a_picture_and_not_a_flat_fill` and the
//! six beside them — and `scenario_pointdevice_run.rs` reads `DISABLE` off the screen over a run. What is
//! left is the two edges: the frame a stage transition takes, and the frame after a run has ended.
//!
//! What it takes to un-stub them: the laid-out 紅魔郷 has one stage, so it has no transition, and its `Gui`
//! is not registered in and out of a draw chain the way the real one's is.

/// The mark is drawn on the run rather than on the frame, so a stage transition does not lose it.
///
/// A stage transition leaves the gameplay scene for exactly one frame — `f44096 scene=3 stage=2` and then
/// `f44097 scene=2 stage=3 frames=1` — and the log's timestamps put those **265ms** apart, because the game
/// builds the next stage inside that frame. Every transition of the run went the same way, **250ms to
/// 265ms** each, so the one frame the mark was asked about and said no to was a quarter of a second of
/// screen: the count came back for an instant at every stage boundary.
///
/// The frame a chapter is put back on is the same case for a different reason — the update drops what it
/// knows of the frame it froze on, a chapter put back not being a continuation of it. Both are a run's own
/// frames, so what the mark is drawn on is whether the run being tracked is one somebody is playing, taken
/// with the stage's snapshot and dropped when the run is left. Both were seen gone on the screen.
#[test]
#[ignore = "the laid-out 紅魔郷 has one stage and so no transition to cross"]
fn the_mark_survives_a_stage_transition_and_the_frame_a_chapter_is_put_back_on() {
    todo!(
        "cross a stage boundary and a chapter restore, and assert the mark is drawn on both frames"
    )
}

/// The mark reaches one frame past the end of the run, which is the frame that stays on the screen.
///
/// `esc` and then やめる ends the run on a single frame — `run ended after 8 retries` and `f20724 scene=1`
/// together, with `f20700 scene=2 paused` before them — and the panel stays on the screen after it, so the
/// row the game paints on that frame is the one left standing there for the whole fade to the title. A mark
/// that stopped with the run stopped one frame early, and the stars showed plain.
///
/// What ends it instead is the game's own `Gui` no longer being in the draw chain. `Gui::RegisterChain` at
/// **0x41b252** registers two statics: **0x69bc7c** through `AddToCalcChain` at priority **0xc**, and
/// **0x69bc5c** through `AddToDrawChain` at priority **0xb** with `Gui::OnDraw` (**0x417502**) at +4 and
/// `&g_Gui` at +0x1c. Which of the two `Chain` lists is which came out of the lines they log — "add calc
/// chain (pri = %d)" at 0x46afb8 against "add draw chain (pri = %d)" at 0x46afd4 — and the draw list's head
/// is the calc list's **0x20** further in. So the mark is drawn while 0x69bc5c is in that list, which also
/// means it can never be drawn over a screen that is no longer the panel.
///
/// The log says what it is worth where each run ends: `lives: the mark stayed on the panel for 1 frame(s)
/// after the run ended`. Giving the same run up from orb's own retry menu needed none — no line, the chain
/// already cut on the frame the run ended. Both were seen on the screen keeping the mark to the end.
#[test]
#[ignore = "the laid-out 紅魔郷 does not register its Gui in and out of a draw chain"]
fn the_mark_stays_on_the_panel_for_the_one_frame_the_game_paints_after_the_run() {
    todo!(
        "end a run through the game's own pause and assert one more marked frame, then through orb's and assert none"
    )
}

/// Two fields in the ask, and the game's own repaint of a stage's first frames is why one of them decides
/// nothing.
///
/// One was tried first, to leave no repaint standing for the frame after the last marked one, and it is not
/// what put the count back: the panel being laid over a stage's **first 250 frames** sets all five of those
/// fields to 2 itself, at **0x41a2b6**, so during those frames the value orb writes decides nothing.
#[test]
#[ignore = "the laid-out 紅魔郷 does not lay its panel over a stage's first 250 frames"]
fn the_games_own_repaint_of_a_stages_first_frames_overrides_what_orb_writes() {
    todo!(
        "run a stage's first 250 frames and assert the field orb writes is overwritten by the game"
    )
}
