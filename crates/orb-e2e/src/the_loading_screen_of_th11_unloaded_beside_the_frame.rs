//! **th11's title thread unloading the loading screen's file while orb's frames go on**, and the frame that file
//! is read in left whole.
//!
//! The title's thread (0x4394f0) waits for the loading screen's counter to reach 180 and unloads slot 1 through
//! 0x4549e0 (0x439564), freeing the scripts and releasing the textures of the loading screen's two VMs under no
//! lock while the main thread runs and draws them. A real launch that lost that race stood spinning at the title
//! or crashed inside the driver, every one on f215 — so what is asserted here is the title still answering a key
//! past the unload, and the unload having waited for the frame it was asked inside: the laid-out game lets its
//! thread run inside the walk it asks on, and stands spinning where the file goes there.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **Frames enough for the unload and the main menu past its build**: the thread unloads at the laid-out game's
/// 180th, and the main menu reads a key from its 83rd update, the title coming up on the 61st.
const PAST_THE_UNLOAD: u32 = 300;

#[test]
fn th11s_title_answers_a_key_past_the_unload_of_the_loading_screens_file() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-loading-screen-unloaded",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.frames(PAST_THE_UNLOAD);
        let before = game.image().the_titles_cursor().0;
        game.press(keys::DOWN);
        assert_ne!(
            game.image().the_titles_cursor().0,
            before,
            "the main menu took no key past the loading screen's unload:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            game.log().said(
                "frame: an unload off the frame's thread waits for the frame to be presented"
            ),
            "the unload did not wait for the frame it was asked inside:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
