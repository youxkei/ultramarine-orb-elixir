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
#[cfg(test)]
pub(crate) mod fake;

#[cfg(test)]
mod a_chapter_further_back;
#[cfg(test)]
mod a_chapter_further_back_of_th07;
#[cfg(test)]
mod a_chapter_of_a_fight_of_th07;
#[cfg(test)]
mod a_chapter_of_the_stages_own_boss;
#[cfg(test)]
mod a_chapter_out_of_th07s_table;
#[cfg(test)]
mod a_chapter_out_of_the_table;
#[cfg(test)]
mod a_chapter_table_collected;
#[cfg(test)]
mod a_chapter_table_collected_for_th07;
#[cfg(test)]
mod a_chapter_under_a_bomb;
#[cfg(test)]
mod a_chapter_under_a_bomb_of_th07;
#[cfg(test)]
mod a_clear_on_demand;
#[cfg(test)]
mod a_clear_on_demand_of_th07;
#[cfg(test)]
mod a_confirmation_declined;
#[cfg(test)]
mod a_confirmation_declined_of_th07;
#[cfg(test)]
mod a_pad_the_game_has_no_device_for;
#[cfg(test)]
mod a_pointdevice_run_of_th07;
#[cfg(test)]
mod a_practice_run;
#[cfg(test)]
mod a_practice_run_of_th07;
#[cfg(test)]
mod a_region_a_restore_cannot_write;
#[cfg(test)]
mod a_region_a_restore_cannot_write_of_th07;
#[cfg(test)]
mod a_stage_transition;
#[cfg(test)]
mod a_stage_transition_of_th07;
#[cfg(test)]
mod keys_from_another_program;
#[cfg(test)]
mod keys_from_another_program_into_th07;
#[cfg(test)]
mod legacy_run;
#[cfg(test)]
mod legacy_run_of_th07;
#[cfg(test)]
mod mode_on_a_winmm_pad;
#[cfg(test)]
mod mode_on_the_pad;
#[cfg(test)]
mod mode_question;
#[cfg(test)]
mod moving_between_a_replays_stages;
#[cfg(test)]
mod moving_between_a_replays_stages_of_th07;
#[cfg(test)]
mod pacing;
#[cfg(test)]
mod pointdevice_run;
#[cfg(test)]
mod stepping_inside_a_stage;
#[cfg(test)]
mod stepping_inside_a_stage_of_th07;
#[cfg(test)]
mod th07;
#[cfg(test)]
mod th07s_window_going_behind;
#[cfg(test)]
mod the_address_space_a_chapter_needs;
#[cfg(test)]
mod the_address_space_a_chapter_of_th07_needs;
#[cfg(test)]
mod the_bot_playing_th06;
#[cfg(test)]
mod the_bot_playing_th07;
#[cfg(test)]
mod the_card_name_across_a_restore;
#[cfg(test)]
mod the_card_name_across_a_restore_of_th07;
#[cfg(test)]
mod the_cards_a_run_of_th07_counted;
#[cfg(test)]
mod the_cases_the_real_game_is_tested_with;
#[cfg(test)]
mod the_count_a_run_of_th07_carries;
#[cfg(test)]
mod the_dpad_moving_the_player;
#[cfg(test)]
mod the_dpad_moving_the_player_of_th07;
#[cfg(test)]
mod the_end_of_a_playback;
#[cfg(test)]
mod the_end_of_a_playback_of_th07;
#[cfg(test)]
mod the_ending;
#[cfg(test)]
mod the_ending_of_th07;
#[cfg(test)]
mod the_frame_a_run_of_th07_acts_on;
#[cfg(test)]
mod the_frame_a_scene_is_built_on;
#[cfg(test)]
mod the_frame_a_scene_of_th07_is_built_on;
#[cfg(test)]
mod the_front_end_of_th07_walked;
#[cfg(test)]
mod the_front_end_walked;
#[cfg(test)]
mod the_handles_a_restore_leaves_alone;
#[cfg(test)]
mod the_handles_a_restore_leaves_alone_of_th07;
#[cfg(test)]
mod the_launch_before_its_device;
#[cfg(test)]
mod the_launch_of_th07_before_its_device;
#[cfg(test)]
mod the_letterbox_presented_into;
#[cfg(test)]
mod the_mark_over_the_lives;
#[cfg(test)]
mod the_mark_over_the_lives_of_th07;
#[cfg(test)]
mod the_mode_of_th07_on_the_pad;
#[cfg(test)]
mod the_mode_question_over_th07;
#[cfg(test)]
mod the_mouse_pointer_over_th07;
#[cfg(test)]
mod the_mouse_pointer_over_the_game;
#[cfg(test)]
mod the_music_across_a_restore;
#[cfg(test)]
mod the_music_across_a_restore_of_th07;
#[cfg(test)]
mod the_number_th07s_shot_is_scaled_by;
#[cfg(test)]
mod the_numbers_a_run_of_th07_was_made_with;
#[cfg(test)]
mod the_pacing_of_th07;
#[cfg(test)]
mod the_pad_half_of_th07s_input_read;
#[cfg(test)]
mod the_pad_half_of_the_input_read;
#[cfg(test)]
mod the_pad_seen_on_both_winmm_and_xinput;
#[cfg(test)]
mod the_player_a_stage_of_th07_starts;
#[cfg(test)]
mod the_player_a_stage_starts;
#[cfg(test)]
mod the_process_orb_woke_up_in;
#[cfg(test)]
mod the_recipe_a_bot_plays_to;
#[cfg(test)]
mod the_recipe_a_bot_plays_to_of_th07;
#[cfg(test)]
mod the_replay_a_run_leaves_behind;
#[cfg(test)]
mod the_replay_a_run_of_th07_leaves_behind;
#[cfg(test)]
mod the_replay_offered;
#[cfg(test)]
mod the_replay_offered_of_th07;
#[cfg(test)]
mod the_run_a_retried_chapter_of_th07_writes_down;
#[cfg(test)]
mod the_run_a_retried_chapter_writes_down;
#[cfg(test)]
mod the_run_left_behind;
#[cfg(test)]
mod the_run_of_th07_left_behind;
#[cfg(test)]
mod the_run_of_th07_read_back;
#[cfg(test)]
mod the_run_of_th07_written_down_off_the_frame;
#[cfg(test)]
mod the_run_read_back;
#[cfg(test)]
mod the_run_written_down_off_the_frame;
#[cfg(test)]
mod the_score_file;
#[cfg(test)]
mod the_screen_a_finished_run_ends_at;
#[cfg(test)]
mod the_screen_a_finished_run_of_th07_ends_at;
#[cfg(test)]
mod the_screens_in_english;
#[cfg(test)]
mod the_screens_of_th07_in_english;
#[cfg(test)]
mod the_snapshot_held_against_itself;
#[cfg(test)]
mod the_snapshot_of_th07_held_against_itself;
#[cfg(test)]
mod the_song_a_resume_of_th07_puts_back;
#[cfg(test)]
mod the_step_th07s_awards_have_climbed;
#[cfg(test)]
mod the_title_menu_of_th07_walked;
#[cfg(test)]
mod the_title_menu_walked;
#[cfg(test)]
mod the_track_heard;
#[cfg(test)]
mod the_track_heard_of_th07;
#[cfg(test)]
mod the_updates_a_resume_of_th07_takes;
#[cfg(test)]
mod the_window;
#[cfg(test)]
mod the_window_a_bot_plays_in;
#[cfg(test)]
mod the_window_a_bot_plays_in_of_th07;
#[cfg(test)]
mod the_window_going_behind;
#[cfg(test)]
mod the_window_of_th07;
#[cfg(test)]
mod the_windows_orb_leaves_alone;
#[cfg(test)]
mod the_windows_orb_leaves_alone_of_th07;
#[cfg(test)]
mod the_word_a_run_of_th07_is_written_down_as;
#[cfg(test)]
mod watching_a_pointdevice_replay;
#[cfg(test)]
mod watching_a_pointdevice_replay_of_th07;
