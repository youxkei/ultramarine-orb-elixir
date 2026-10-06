//! **th11's title thread decoding `title.anm` while the loading's thread decodes the score file**, and the main
//! menu's VM made out of what the first decoded.
//!
//! th11's LZSS decode (0x4426c0) and encode (0x4423a0) keep one window for the whole process, the 8KB at `0x4c0ee8`,
//! and take no lock around it: the archive's reads are inside `0x4c3ac0` and the score file's decode (0x437952) is
//! not. A launch held for a debugger logged the score's decode still running on the loading object's thread as the
//! title's thread began on `title.anm`, and a launch that stood spinning at the title held every script of
//! `title.anm` with its lengths nought — a window written over under a decode — the main menu's VM of script 0 then
//! running an instruction of size nought for ever. So what is asserted here is the main menu answering a key past
//! the frame it makes that VM on, and a decode having waited for the other: the laid-out game's two decodes overlap
//! unless something keeps them apart, and it stands spinning where `title.anm` came out wrong.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **Frames enough for the main menu past its VM and its build**: the title comes up on the laid-out game's 61st
/// frame, makes the VM on the main menu's frame `0x46` and reads a key from its 83rd update.
const PAST_THE_MAIN_MENUS_VM: u32 = 160;

#[test]
fn th11s_main_menu_answers_a_key_past_title_anm_decoded_beside_the_score_file() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-title-decoded-beside-the-score",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.frames(PAST_THE_MAIN_MENUS_VM);
        let before = game.image().the_titles_cursor().0;
        game.press(keys::DOWN);
        assert_ne!(
            game.image().the_titles_cursor().0,
            before,
            "the main menu took no key past the VM it makes out of title.anm:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            game.log().said(
                "lzss: a decode or an encode waits for another thread's to be done with the window"
            ),
            "no decode waited for the other:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
