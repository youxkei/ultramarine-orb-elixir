//! **A launch of th11 pressing through a pad** — `--through-a-pad`, the virtual pad orb's own code plugs in — whose
//! recipe and rules reach the game the way a person's pad does: through the game's own read of a pad, and through
//! orb's menus' read of one, and not by having their word handed to the read.
//!
//! **Both of th11's ways of reading a pad are held here**, the input's set-up deciding between them
//! (`game/th11/asm.md`, *The input*): `joyGetPosEx(0)` (0x4572bf) where its DirectInput enumeration found none — the
//! virtual pad answering as joystick 0, its caps the ones the set-up's probe keeps (0x4571a0) — and the DirectInput
//! pad it found otherwise, whose `GetDeviceState` (0x457476 through 0x45749f) orb stands in front of. **What says a
//! press went through a pad** is what the game's own read made of its pads, with the keyboard's word left out —
//! `Fake::the_pads_share_of_the_read`.

use crate::fake::th11::{DirectInputPad, Fake, PluggedIn, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bits of th11's word the shot, the bomb and down are.
const SHOT: u32 = 0x1;
const BOMB: u32 = 0x2;
const DOWN: u32 = 0x20;

/// Another program's window in front.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// A recipe holding the shot for as long as the test runs.
const HOLDING_THE_SHOT: &str = "- press: [shot]\n  frames: 100000\n";

/// A launch of `recipe`, through the pad or not, with `pad` plugged in as the game starts.
fn launched(name: &str, pad: PluggedIn, through_a_pad: bool, recipe: &str) -> Box<Fake> {
    let recipe = recipe.to_owned();
    Fake::attach_with_a_pad(
        pad,
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        move |config| {
            config.autoplay = Some(recipe);
            config.through_a_pad = through_a_pad;
            config.log_level = LogLevel::Verbose;
        },
    )
}

/// **With no pad on the machine, the virtual one is joystick 0**: the set-up's probe finds it and keeps its caps,
/// and the recipe's shot is a button the game's own winmm read makes the shot of.
#[test]
fn a_recipes_press_reaches_th11s_winmm_read_as_a_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th11-through-a-pad-on-winmm",
            PluggedIn::Nothing,
            true,
            HOLDING_THE_SHOT,
        );
        let log = game.log();
        assert!(
            log.said("joystick: a virtual pad is plugged in"),
            "no virtual pad was plugged in:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_joysticks_axes(),
            (0, 65535, 0, 65535),
            "the set-up's probe did not keep the caps of a joystick 0",
        );
        game.frames(10);
        assert_eq!(
            game.the_pads_share_of_the_read(),
            SHOT,
            "the recipe's shot did not reach the game's own read of its pad",
        );
        assert_eq!(
            game.word_the_input_read_answered() & SHOT,
            SHOT,
            "the shot read off the pad did not reach the word",
        );
    });
}

/// **And without `--through-a-pad`, the same recipe's word is handed to the read as it always was**, the pads'
/// share of it nothing.
#[test]
fn a_recipes_press_is_handed_to_th11s_read_where_the_launch_presses_through_no_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th11-through-no-pad",
            PluggedIn::Nothing,
            false,
            HOLDING_THE_SHOT,
        );
        game.frames(10);
        assert_eq!(game.the_pads_share_of_the_read(), 0, "a pad was pressed");
        assert_eq!(
            game.word_the_input_read_answered() & SHOT,
            SHOT,
            "the recipe's word did not reach the read",
        );
        assert!(
            !game.log().said("joystick: a virtual pad is plugged in"),
            "a virtual pad was plugged in a launch that asked for none",
        );
    });
}

/// **Where the game holds a DirectInput pad, the virtual one reaches it through that device**, orb standing in
/// front of its `GetDeviceState` — and the real pad's own buttons do not, a run pressed through the virtual pad
/// being orb's: see `a_run_of_th10_pressed_through_a_pad`.
#[test]
fn a_recipes_press_reaches_th11s_directinput_read_in_place_of_the_real_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th11-through-a-pad-on-directinput",
            PluggedIn::ThroughDirectInput,
            true,
            "- press: [down]\n  frames: 100000\n",
        );
        let log = game.log();
        game.frames(10);
        assert!(
            log.said("joystick: the game's DirectInput pad answers the virtual pad as well"),
            "orb did not stand in front of the game's DirectInput pad:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.the_pads_share_of_the_read(),
            DOWN,
            "the recipe's down did not reach the game's DirectInput read",
        );
        game.pushes_its_directinput_pad(DirectInputPad {
            buttons: 1 << 1,
            ..DirectInputPad::default()
        });
        game.frames(2);
        assert_eq!(
            game.the_pads_share_of_the_read() & (DOWN | BOMB),
            DOWN,
            "the real pad's bomb reached a run pressed through the virtual pad",
        );
    });
}

/// **With the window behind, the pad is read and the keyboard is not**, which is a launch playing itself in a
/// corner of somebody's screen: the recipe's shot reaches the game, and a key held down for another program does
/// not.
#[test]
fn a_recipes_press_reaches_th11_through_a_pad_with_its_window_behind() {
    in_its_own_process(|| {
        let game = launched(
            "th11-through-a-pad-behind",
            PluggedIn::Nothing,
            true,
            HOLDING_THE_SHOT,
        );
        game.frames(10);
        game.sim().display().set_foreground(ANOTHER);
        game.keyboard().set(keys::X, true);
        game.frames(10);
        assert!(
            game.log().said("input: window behind, keys not read"),
            "the window going behind was not seen",
        );
        let word = game.word_the_input_read_answered();
        assert_eq!(
            word & SHOT,
            SHOT,
            "the pad's shot did not reach the game behind the window"
        );
        assert_eq!(
            word & BOMB,
            0,
            "a key held for another program reached the game"
        );
    });
}

/// **And the whole front end walked through the pad**: the title's screens and selects, orb's own question of the
/// mode answered off what the pad is doing, and the run entered — every press of the recipe a pad's.
#[test]
fn th11s_front_end_and_orbs_question_are_walked_through_a_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th11-the-front-end-through-a-pad",
            PluggedIn::Nothing,
            true,
            &format!("{TO_THE_SHOT}{TO_A_RUN}- play_until: {{ frames: 100000 }}\n"),
        );
        let log = game.log();
        game.frames_until("the run", 20_000, || game.state().in_game);
        assert!(
            log.said("mode: answered on the pad"),
            "orb's question was not answered on the pad:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("autoplay: this game takes no word as a pad"),
            "the word was handed to the read:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And orb's question answered with the game's own mode through the pad at sixteen times the pace**, which is how
/// the real case of a Normal mode run is played: the cursor moved off the mode the question comes up on, and
/// decided.
#[test]
fn orbs_question_over_th11_is_answered_with_its_own_mode_through_a_pad_at_sixteen_times_the_pace() {
    in_its_own_process(|| {
        let recipe = format!(
            "{}{TO_A_RUN}- play_until: {{ frames: 100000 }}\n",
            TO_THE_SHOT.replace(
                "- answer: { question: mode, item: 0 }",
                "- answer: { question: mode, item: 1 }"
            )
        );
        let game = Fake::attach_with_a_pad(
            PluggedIn::Nothing,
            Display::agreed(120),
            "th11-its-own-mode-through-a-pad-paced",
            Work::wandering(WORK_US),
            move |config| {
                config.autoplay = Some(recipe);
                config.through_a_pad = true;
                config.pace = 16;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.frames_until("the run", 20_000, || game.state().in_game);
        assert!(
            log.said("mode: normal, was pointdevice"),
            "orb's question was not answered with the game's own mode:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
