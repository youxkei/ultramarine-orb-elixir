//! The launches: a game playing the game's part, and every scenario that drives orb through one.
//!
//! **A scenario is a launch with the far side of the hooks replaced.** `orb_core::runtime::attach_to`
//! fills the same statics `orb`'s own install lists fill, so orb's own code is the same code in a scenario as
//! in a run and the only thing that can differ is what lies past a hook. What lies past them here is
//! [`fake`], whose address space, scenes and records are the game's own laid out by hand; what lies under
//! them is `orb-sim`.
//!
//! **Nothing here names `orb`.** A laid-out game has no import table, so where a real launch patches an
//! entry it calls the rewrite itself — `orb_core::window::create_window_ex_a`,
//! `orb_core::score::create_file_a` and `orb_core::joystick::answer` — and every one of those is above the
//! line `orb` draws round the patched bytes. What is left in that crate is `DllMain`, the trampolines, the
//! PE headers and the crash filter, none of which a laid-out game has anything to offer. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).
//!
//! **The whole crate is `#[cfg(test)]`, and that is what it is for.** `orb_core::game::th06::image` and
//! the seam's install point are reached through `orb-core`'s `sim` feature, and a crate under test
//! cannot turn a feature on for itself — which is why the scenarios were integration tests for as long
//! as they lived in a crate that is under test. This one is not: it is a consumer, so it asks for
//! `orb-core = { features = ["sim"] }` the ordinary way and its own `#[cfg(test)]` is true. See
//! [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
//!
//! **Which buys the dead-code check back.** As integration tests the fake was compiled once per
//! `scenario_*.rs`, twenty-three times, and carried a blanket `#![allow(dead_code)]`: what one file did
//! not touch was another file's, and `dead_code` is worked out per binary so nothing could see that. One
//! binary holding the fake and every scenario over it can. [`fake`] is `pub(crate)` for the same reason
//! and the two go together — `dead_code` does not fire on a `pub` item a crate exports, so a fake
//! reachable from outside would take the allow away and find nothing.
//!
//! One module per file, and no `scenario_` on any of them: the prefix was there because two levels of
//! test shared one directory, and the crate's name says the level now.

// Named `pub(crate)` rather than left private to say which visibility is load-bearing: a `pub` item in
// here is not one this crate exports, so `dead_code` reaches all of it.
#[cfg(test)]
pub(crate) mod fake;

#[cfg(test)]
mod a_chapter_out_of_the_table;
#[cfg(test)]
mod a_chapter_table_collected;
#[cfg(test)]
mod a_clear_on_demand;
#[cfg(test)]
mod a_stage_transition;
#[cfg(test)]
mod keys_from_another_program;
#[cfg(test)]
mod legacy_run;
#[cfg(test)]
mod mode_on_a_winmm_pad;
#[cfg(test)]
mod mode_on_the_pad;
#[cfg(test)]
mod mode_question;
#[cfg(test)]
mod moving_between_a_replays_stages;
#[cfg(test)]
mod pacing;
#[cfg(test)]
mod pointdevice_run;
#[cfg(test)]
mod th07;
#[cfg(test)]
mod the_ending;
#[cfg(test)]
mod the_frame_a_scene_is_built_on;
#[cfg(test)]
mod the_handles_a_restore_leaves_alone;
#[cfg(test)]
mod the_launch_before_its_device;
#[cfg(test)]
mod the_mark_over_the_lives;
#[cfg(test)]
mod the_music_across_a_restore;
#[cfg(test)]
mod the_player_a_stage_starts;
#[cfg(test)]
mod the_run_read_back;
#[cfg(test)]
mod the_score_file;
#[cfg(test)]
mod the_window;
#[cfg(test)]
mod the_window_going_behind;
