//! **A launch of th08 pressing through a pad** — `--through-a-pad` — whose recipe reaches the game through the
//! game's own read of a pad (0x43d120) and not by having its word handed to the read.
//!
//! **Both of th08's ways of reading a pad are held here**, as th10's are (`game/th08/asm.md`, *The pad*):
//! `joyGetPosEx(0)` where the input's set-up found no DirectInput pad — the virtual pad answering as joystick 0,
//! its caps the ones the startup check keeps — and the DirectInput pad it found otherwise, whose `GetDeviceState`
//! orb stands in front of. **What says a press went through a pad** is what the game's own read made of its pads,
//! with the keyboard's word left out — `Fake::the_pads_share_of_the_read`. The front end walked through the pad is
//! `the_cases_of_th08_the_real_game_is_tested_with`'s, every chain there pressing through it.

use crate::fake::th08::{DirectInputPad, Fake, PluggedIn};
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bits of th08's word the shot, the bomb and down are.
const SHOT: u16 = 0x1;
const BOMB: u16 = 0x2;
const DOWN: u16 = 0x20;

/// The button `th08.cfg`'s mapping has the bomb on.
const THE_BOMBS_BUTTON: u32 = 5;

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

/// **With no pad on the machine, the virtual one is joystick 0**: the startup check finds it and keeps its caps,
/// and the recipe's shot is a button the game's own winmm read makes the shot of.
#[test]
fn a_recipes_press_reaches_th08s_winmm_read_as_a_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th08-through-a-pad-on-winmm",
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
            "the startup check did not keep the caps of a joystick 0",
        );
        game.frames(10);
        assert!(
            log.said("autoplay: the run is pressed through the virtual pad"),
            "the run was not pressed through the pad:\n  {}",
            log.lines().join("\n  ")
        );
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
fn a_recipes_press_is_handed_to_th08s_read_where_the_launch_presses_through_no_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th08-through-no-pad",
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
fn a_recipes_press_reaches_th08s_directinput_read_in_place_of_the_real_pad() {
    in_its_own_process(|| {
        let game = launched(
            "th08-through-a-pad-on-directinput",
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
            buttons: 1 << THE_BOMBS_BUTTON,
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
fn a_recipes_press_reaches_th08_through_a_pad_with_its_window_behind() {
    in_its_own_process(|| {
        let game = launched(
            "th08-through-a-pad-behind",
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
