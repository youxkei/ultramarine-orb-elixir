//! The launches: a game playing the game's part, and every e2e test that drives orb through one.
//!
//! **An e2e test is a launch with the far side of the hooks replaced.** `orb_core::runtime::attach_to`
//! fills the same statics `orb`'s own install lists fill, so orb's own code is the same code in an e2e
//! test as in a run and the only thing that can differ is what lies past a hook. What lies past them is
//! [`fake`], whose address space, scenes and records are the game's own laid out by hand; what lies under
//! them is `orb-sim`.
//!
//! **Nothing here names `orb`.** A laid-out game has no import table, so where a real launch patches an
//! entry it calls the rewrite itself — `orb_core::window::create_window_ex_a`,
//! `orb_core::score::create_file_a` and `orb_core::joystick::answer` — and every one of those is above the
//! line `orb` draws round the patched bytes. What is left in that crate is `DllMain`, the trampolines, the
//! PE headers and the crash filter, none of which a laid-out game has anything to offer.
//!
//! **The whole crate is `#[cfg(test)]`, and that is what it is for.** `orb_core::game::th06::image` and
//! the seam's install point are reached through `orb-core`'s `sim` feature, and a crate under test
//! cannot turn a feature on for itself — which is why the e2e tests were integration tests for as long
//! as they lived in a crate that is under test. This one is not: it is a consumer, so it asks for
//! `orb-core = { features = ["sim"] }` the ordinary way and its own `#[cfg(test)]` is true.
//!
//! **Which buys the dead-code check back.** As integration tests the fake was compiled once per
//! `scenario_*.rs`, twenty-three times, and carried a blanket `#![allow(dead_code)]`: what one file did
//! not touch was another file's, and `dead_code` is worked out per binary so nothing could see that. One
//! binary holding the fake and every e2e test over it can. [`fake`] is `pub(crate)` for the same reason
//! and the two go together — `dead_code` does not fire on a `pub` item a crate exports, so a fake
//! reachable from outside would take the allow away and find nothing.
//!
//! One module per file, and no `scenario_` on any of them: the prefix was there because two levels of
//! test shared one directory, and the crate's name says the level now.

// Named `pub(crate)` rather than left private to say which visibility is load-bearing: a `pub` item in
// here is not one this crate exports, so `dead_code` reaches all of it.
//
// **And every module is the architecture of the game it lays out.** th06 and th07 are 32-bit processes and
// th06c a 64-bit one, and `orb-core` carries the games of the architecture it is built for and no others —
// see `orb_core::game::KNOWN` — so a test over a laid-out th06 is a test only the 32-bit build has a game
// for, and one over th06c only the 64-bit build. `cargo xtask test` runs this crate for both, each with the
// modules its games' tests are in.
#[cfg(test)]
pub(crate) mod fake;

#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_further_back_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_further_back_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_of_a_fight_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_of_a_fight_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_out_of_th06cs_table;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_out_of_th06ncs_table;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_table_collected_for_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_table_collected_for_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_under_a_bomb_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_chapter_under_a_bomb_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_clear_on_demand_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_clear_on_demand_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_confirmation_declined_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_confirmation_declined_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_continue_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_conversation_of_th06nc_pressed_past;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_death_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_death_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_pointdevice_run_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_pointdevice_run_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_region_a_restore_of_th06c_cannot_write;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_region_a_restore_of_th06nc_cannot_write;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_run_of_th06c_played_twice;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_run_of_th06c_pressed_through_a_pad;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_run_of_th06nc_played_twice;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_run_of_th06nc_pressed_through_a_pad;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_span_of_th06nc_held_back_by_its_source;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_stage_transition_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod a_stage_transition_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod stepping_inside_a_stage_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod stepping_inside_a_stage_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod th06cs_window_going_behind;
#[cfg(all(test, target_arch = "x86_64"))]
mod th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod th06ncs_window_going_behind;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_address_space_a_chapter_of_th06c_needs;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_address_space_a_chapter_of_th06nc_needs;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_bot_playing_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_bot_playing_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_cards_a_run_of_th06c_counted;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_cards_a_run_of_th06nc_counted;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_cases_of_th06c_the_real_game_is_tested_with;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_cases_of_th06nc_the_real_game_is_tested_with;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_end_of_a_playback_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_end_of_a_playback_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_frame_a_run_of_th06c_acts_on;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_frame_a_run_of_th06nc_acts_on;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_frame_a_scene_of_th06c_is_built_on;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_frame_a_scene_of_th06nc_is_built_on;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_front_end_of_th06c_walked;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_front_end_of_th06nc_walked;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_main_menu_of_th06nc_walked;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_mode_question_over_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_mode_question_over_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_numbers_a_run_of_th06c_was_made_with;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_numbers_a_run_of_th06nc_was_made_with;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_pacing_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_pacing_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_player_a_stage_of_th06c_starts;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_player_a_stage_of_th06nc_starts;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_replay_a_run_of_th06c_leaves_behind;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_replay_a_run_of_th06nc_leaves_behind;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_replay_offered_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_replay_offered_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_run_a_retried_chapter_of_th06c_writes_down;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_run_a_retried_chapter_of_th06nc_writes_down;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_run_of_th06c_left_behind;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_run_of_th06c_written_down_off_the_frame;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_run_of_th06nc_left_behind;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_run_of_th06nc_written_down_off_the_frame;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_screen_a_finished_run_of_th06c_ends_at;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_screen_a_finished_run_of_th06nc_ends_at;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_screens_of_th06c_in_english;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_screens_of_th06nc_in_english;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_snapshot_of_th06c_held_against_itself;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_snapshot_of_th06nc_held_against_itself;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_start_of_a_run_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_start_of_a_run_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_title_menu_of_th06c_walked;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_track_heard_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_track_heard_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_updates_a_resume_of_th06c_takes;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_updates_a_resume_of_th06nc_takes;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_way_a_stage_of_th06c_ends;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_window_a_bot_plays_in_of_th06nc;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_word_a_run_of_th06c_is_written_down_as;
#[cfg(all(test, target_arch = "x86_64"))]
mod the_word_a_run_of_th06nc_is_written_down_as;
#[cfg(all(test, target_arch = "x86_64"))]
mod watching_a_pointdevice_replay_of_th06c;
#[cfg(all(test, target_arch = "x86_64"))]
mod watching_a_pointdevice_replay_of_th06nc;

#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_further_back;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_further_back_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_further_back_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_further_back_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_of_a_fight_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_of_a_fight_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_of_a_fight_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_of_the_stages_own_boss;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_out_of_th07s_table;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_out_of_th08s_table;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_out_of_th10s_table;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_out_of_the_table;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_table_collected;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_table_collected_for_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_table_collected_for_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_table_collected_for_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_under_a_bomb;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_under_a_bomb_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_chapter_under_a_bomb_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_clear_on_demand;
#[cfg(all(test, target_arch = "x86"))]
mod a_clear_on_demand_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_clear_on_demand_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_clear_on_demand_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_confirmation_declined;
#[cfg(all(test, target_arch = "x86"))]
mod a_confirmation_declined_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_confirmation_declined_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_confirmation_declined_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_death_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_pad_the_game_has_no_device_for;
#[cfg(all(test, target_arch = "x86"))]
mod a_pointdevice_run_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_pointdevice_run_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_pointdevice_run_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_practice_run;
#[cfg(all(test, target_arch = "x86"))]
mod a_practice_run_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_practice_run_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod a_region_a_restore_cannot_write;
#[cfg(all(test, target_arch = "x86"))]
mod a_region_a_restore_cannot_write_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_region_a_restore_of_th08_cannot_write;
#[cfg(all(test, target_arch = "x86"))]
mod a_region_a_restore_of_th10_cannot_write;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th06_pressed_through_a_pad;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th07_played_twice;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th07_pressed_through_a_pad;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th08_played_twice;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th08_pressed_through_a_pad;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th10_played_twice;
#[cfg(all(test, target_arch = "x86"))]
mod a_run_of_th10_pressed_through_a_pad;
#[cfg(all(test, target_arch = "x86"))]
mod a_stage_transition;
#[cfg(all(test, target_arch = "x86"))]
mod a_stage_transition_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod a_stage_transition_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod a_stage_transition_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod keys_from_another_program;
#[cfg(all(test, target_arch = "x86"))]
mod keys_from_another_program_into_th07;
#[cfg(all(test, target_arch = "x86"))]
mod keys_from_another_program_into_th08;
#[cfg(all(test, target_arch = "x86"))]
mod keys_from_another_program_into_th10;
#[cfg(all(test, target_arch = "x86"))]
mod legacy_run;
#[cfg(all(test, target_arch = "x86"))]
mod legacy_run_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod legacy_run_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod legacy_run_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod mode_on_a_winmm_pad;
#[cfg(all(test, target_arch = "x86"))]
mod mode_on_the_pad;
#[cfg(all(test, target_arch = "x86"))]
mod mode_question;
#[cfg(all(test, target_arch = "x86"))]
mod moving_between_a_replays_stages;
#[cfg(all(test, target_arch = "x86"))]
mod moving_between_a_replays_stages_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod pacing;
#[cfg(all(test, target_arch = "x86"))]
mod pointdevice_run;
#[cfg(all(test, target_arch = "x86"))]
mod stepping_inside_a_stage;
#[cfg(all(test, target_arch = "x86"))]
mod stepping_inside_a_stage_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod stepping_inside_a_stage_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod stepping_inside_a_stage_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod th07;
#[cfg(all(test, target_arch = "x86"))]
mod th07s_window_going_behind;
#[cfg(all(test, target_arch = "x86"))]
mod th08;
#[cfg(all(test, target_arch = "x86"))]
mod th08s_window_going_behind;
#[cfg(all(test, target_arch = "x86"))]
mod th10;
#[cfg(all(test, target_arch = "x86"))]
mod th10s_window_going_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_address_space_a_chapter_needs;
#[cfg(all(test, target_arch = "x86"))]
mod the_address_space_a_chapter_of_th07_needs;
#[cfg(all(test, target_arch = "x86"))]
mod the_address_space_a_chapter_of_th08_needs;
#[cfg(all(test, target_arch = "x86"))]
mod the_address_space_a_chapter_of_th10_needs;
#[cfg(all(test, target_arch = "x86"))]
mod the_bot_playing_th06;
#[cfg(all(test, target_arch = "x86"))]
mod the_bot_playing_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_bot_playing_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_bot_playing_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_card_name_across_a_restore;
#[cfg(all(test, target_arch = "x86"))]
mod the_card_name_across_a_restore_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_cards_a_run_of_th07_counted;
#[cfg(all(test, target_arch = "x86"))]
mod the_cards_a_run_of_th08_counted;
#[cfg(all(test, target_arch = "x86"))]
mod the_cards_a_run_of_th10_counted;
#[cfg(all(test, target_arch = "x86"))]
mod the_cases_of_th08_the_real_game_is_tested_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_cases_of_th10_the_real_game_is_tested_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_cases_the_real_game_is_tested_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_clear_of_th10_kept_through_the_mode_question;
#[cfg(all(test, target_arch = "x86"))]
mod the_conversation_a_run_of_th08_plays_through;
#[cfg(all(test, target_arch = "x86"))]
mod the_count_a_run_of_th07_carries;
#[cfg(all(test, target_arch = "x86"))]
mod the_dpad_moving_the_player;
#[cfg(all(test, target_arch = "x86"))]
mod the_dpad_moving_the_player_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_end_of_a_playback;
#[cfg(all(test, target_arch = "x86"))]
mod the_end_of_a_playback_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_end_of_a_playback_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_end_of_a_playback_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_ending;
#[cfg(all(test, target_arch = "x86"))]
mod the_ending_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_run_of_th06_acts_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_run_of_th07_acts_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_run_of_th08_acts_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_run_of_th10_acts_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_scene_is_built_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_scene_of_th07_is_built_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_scene_of_th08_is_built_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_frame_a_scene_of_th10_is_built_on;
#[cfg(all(test, target_arch = "x86"))]
mod the_front_end_of_th07_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_front_end_of_th08_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_front_end_of_th10_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_front_end_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_handles_a_restore_leaves_alone;
#[cfg(all(test, target_arch = "x86"))]
mod the_handles_a_restore_leaves_alone_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_handles_a_restore_leaves_alone_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_handles_a_restore_leaves_alone_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_launch_before_its_device;
#[cfg(all(test, target_arch = "x86"))]
mod the_launch_of_th07_before_its_device;
#[cfg(all(test, target_arch = "x86"))]
mod the_launch_of_th08_before_its_device;
#[cfg(all(test, target_arch = "x86"))]
mod the_launch_of_th10_before_its_device;
#[cfg(all(test, target_arch = "x86"))]
mod the_letterbox_presented_into;
#[cfg(all(test, target_arch = "x86"))]
mod the_mark_over_the_lives;
#[cfg(all(test, target_arch = "x86"))]
mod the_mark_over_the_lives_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_mode_of_th07_on_the_pad;
#[cfg(all(test, target_arch = "x86"))]
mod the_mode_of_th10_on_the_pad;
#[cfg(all(test, target_arch = "x86"))]
mod the_mode_question_over_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_mode_question_over_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_mode_question_over_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_mouse_pointer_over_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_mouse_pointer_over_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_mouse_pointer_over_the_game;
#[cfg(all(test, target_arch = "x86"))]
mod the_music_across_a_restore;
#[cfg(all(test, target_arch = "x86"))]
mod the_music_across_a_restore_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_numbers_a_run_of_th06_was_made_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_numbers_a_run_of_th07_was_made_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_numbers_a_run_of_th08_was_made_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_numbers_a_run_of_th10_was_made_with;
#[cfg(all(test, target_arch = "x86"))]
mod the_pacing_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_pacing_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_pacing_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_pad_half_of_th07s_input_read;
#[cfg(all(test, target_arch = "x86"))]
mod the_pad_half_of_the_input_read;
#[cfg(all(test, target_arch = "x86"))]
mod the_pad_playing_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_pad_seen_on_both_winmm_and_xinput;
#[cfg(all(test, target_arch = "x86"))]
mod the_player_a_stage_of_th07_starts;
#[cfg(all(test, target_arch = "x86"))]
mod the_player_a_stage_of_th08_starts;
#[cfg(all(test, target_arch = "x86"))]
mod the_player_a_stage_of_th10_starts;
#[cfg(all(test, target_arch = "x86"))]
mod the_player_a_stage_starts;
#[cfg(all(test, target_arch = "x86"))]
mod the_process_orb_woke_up_in;
#[cfg(all(test, target_arch = "x86"))]
mod the_rank_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_recipe_a_bot_plays_to;
#[cfg(all(test, target_arch = "x86"))]
mod the_recipe_a_bot_plays_to_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_recipe_a_bot_plays_to_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_recipe_a_bot_plays_to_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_a_run_leaves_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_a_run_of_th07_leaves_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_a_run_of_th08_leaves_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_a_run_of_th10_leaves_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_offered;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_offered_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_offered_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_replay_offered_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_a_retried_chapter_of_th07_writes_down;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_a_retried_chapter_of_th08_writes_down;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_a_retried_chapter_of_th10_writes_down;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_a_retried_chapter_writes_down;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_left_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th07_left_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th07_read_back;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th07_written_down_off_the_frame;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th08_left_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th08_read_back;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th08_written_down_off_the_frame;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th10_left_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th10_read_back;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_of_th10_written_down_off_the_frame;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_read_back;
#[cfg(all(test, target_arch = "x86"))]
mod the_run_written_down_off_the_frame;
#[cfg(all(test, target_arch = "x86"))]
mod the_score_file;
#[cfg(all(test, target_arch = "x86"))]
mod the_screen_a_finished_run_ends_at;
#[cfg(all(test, target_arch = "x86"))]
mod the_screen_a_finished_run_of_th07_ends_at;
#[cfg(all(test, target_arch = "x86"))]
mod the_screen_a_finished_run_of_th08_ends_at;
#[cfg(all(test, target_arch = "x86"))]
mod the_screen_a_finished_run_of_th10_ends_at;
#[cfg(all(test, target_arch = "x86"))]
mod the_screens_in_english;
#[cfg(all(test, target_arch = "x86"))]
mod the_screens_of_th07_in_english;
#[cfg(all(test, target_arch = "x86"))]
mod the_screens_of_th08_in_english;
#[cfg(all(test, target_arch = "x86"))]
mod the_screens_of_th10_in_english;
#[cfg(all(test, target_arch = "x86"))]
mod the_script_clock_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_seal_of_a_run_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_snapshot_held_against_itself;
#[cfg(all(test, target_arch = "x86"))]
mod the_snapshot_of_th07_held_against_itself;
#[cfg(all(test, target_arch = "x86"))]
mod the_snapshot_of_th08_held_against_itself;
#[cfg(all(test, target_arch = "x86"))]
mod the_snapshot_of_th10_held_against_itself;
#[cfg(all(test, target_arch = "x86"))]
mod the_song_a_resume_of_th07_puts_back;
#[cfg(all(test, target_arch = "x86"))]
mod the_step_th07s_awards_have_climbed;
#[cfg(all(test, target_arch = "x86"))]
mod the_title_menu_of_th07_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_title_menu_of_th08_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_title_menu_of_th10_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_title_menu_walked;
#[cfg(all(test, target_arch = "x86"))]
mod the_track_heard;
#[cfg(all(test, target_arch = "x86"))]
mod the_track_heard_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_track_heard_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_track_heard_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_updates_a_resume_of_th06_takes;
#[cfg(all(test, target_arch = "x86"))]
mod the_updates_a_resume_of_th07_takes;
#[cfg(all(test, target_arch = "x86"))]
mod the_updates_a_resume_of_th08_takes;
#[cfg(all(test, target_arch = "x86"))]
mod the_updates_a_resume_of_th10_takes;
#[cfg(all(test, target_arch = "x86"))]
mod the_window;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_a_bot_plays_in;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_a_bot_plays_in_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_a_bot_plays_in_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_a_bot_plays_in_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_going_behind;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_window_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_windows_orb_leaves_alone;
#[cfg(all(test, target_arch = "x86"))]
mod the_windows_orb_leaves_alone_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod the_windows_orb_leaves_alone_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod the_windows_orb_leaves_alone_of_th10;
#[cfg(all(test, target_arch = "x86"))]
mod the_word_a_run_of_th07_is_written_down_as;
#[cfg(all(test, target_arch = "x86"))]
mod the_word_a_run_of_th08_is_written_down_as;
#[cfg(all(test, target_arch = "x86"))]
mod the_word_a_run_of_th10_is_written_down_as;
#[cfg(all(test, target_arch = "x86"))]
mod watching_a_pointdevice_replay;
#[cfg(all(test, target_arch = "x86"))]
mod watching_a_pointdevice_replay_of_th07;
#[cfg(all(test, target_arch = "x86"))]
mod watching_a_pointdevice_replay_of_th08;
#[cfg(all(test, target_arch = "x86"))]
mod watching_a_pointdevice_replay_of_th10;
