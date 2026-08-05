//! 完全無欠モード, chosen at the question orb puts over the game's own title menu.
//!
//! The real `mode::Question` reading a real `input::Keyboard` off a simulated Windows: keys a
//! scenario holds down, `GetKeyboardState` answering with them, and what orb makes of that. What the
//! question *draws* is asserted in `mode_ui.rs` against a recording device; what it *decides* was
//! decided by hand until the keyboard went behind the seam, and the decision is the half that turns
//! a run into a pointdevice one.
//!
//! The window has to be in front for any of it. `Keyboard::poll` treats everything as released
//! otherwise — alt-tabbing away must not leave a key stuck down, and must not let typing elsewhere
//! drive the game — so a scenario that forgot to say the game is in front would see nothing pressed
//! and could not tell that from a question that ignores its keys.

use std::sync::Arc;

use orb_api::Hwnd;
use orb_core::game::{Menu, Pad};
use orb_core::input::Keyboard;
use orb_core::mode::{Answer, INPUT_GRACE_FRAMES, Mode, Question};
use orb_sim::{Sim, keys};

/// Any window, so long as it is the one in front: orb compares the game's own against the host's
/// answer and asks for nothing else about it.
const WINDOW: Hwnd = Hwnd(0x1234);

/// One question, its keyboard, and the simulated Windows both read through.
struct Asking {
    sim: Arc<Sim>,
    _installed: orb_api::Installed,
    keyboard: Keyboard,
    question: Question,
}

impl Asking {
    /// Asked over the title menu, with orb in `current` — which is where the cursor starts.
    fn about_a_run(current: Mode) -> Self {
        Self::new(Menu::Run, current)
    }

    fn new(asked: Menu, current: Mode) -> Self {
        let sim = Arc::new(Sim::new());
        let installed = sim.enter();
        sim.display().set_foreground(WINDOW);
        Self {
            sim,
            _installed: installed,
            keyboard: Keyboard::new(),
            question: Question::new(asked, current),
        }
    }

    /// One frame of the question: the keyboard is read, then the question is asked what that says.
    ///
    /// In orb's own loop these are two calls a hook apart — `Keyboard::poll` in `on_update`, the
    /// question's `update` while the game is frozen — and the order is what matters: a question that
    /// read the keyboard itself would see the state a frame late.
    fn frame(&mut self, pad: Pad) -> Option<(Answer, By)> {
        self.keyboard.poll(WINDOW);
        self.question.update(&self.keyboard, pad)
    }

    /// Frames with nothing touched, for running the grace out.
    fn idle(&mut self, frames: u32) -> Option<(Answer, By)> {
        (0..frames).find_map(|_| self.frame(Pad::default()))
    }

    /// A key pressed and let go again, which is what one press is: orb acts on the edge, so a key
    /// left down would be one press however long it is held.
    fn press(&mut self, key: u8) -> Option<(Answer, By)> {
        self.sim.keyboard().set(key, true);
        let answered = self.frame(Pad::default());
        self.sim.keyboard().set(key, false);
        answered
    }

    /// Past the grace, with nothing having been answered on the way.
    fn ready(&mut self) -> &mut Self {
        assert!(
            self.idle(INPUT_GRACE_FRAMES).is_none(),
            "the grace answered something on its own"
        );
        assert_eq!(self.question.held(), 0, "the grace is not run out");
        self
    }
}

use orb_core::menu::By;

fn chosen(answered: Option<(Answer, By)>) -> (Mode, By) {
    match answered {
        Some((Answer::Chosen(mode), by)) => (mode, by),
        Some((Answer::Cancelled, by)) => panic!("cancelled by the {by}, where a mode was expected"),
        None => panic!("nothing was answered"),
    }
}

// ── The answer itself ────────────────────────────────────────────────────────────────────────────

/// The cursor starts on the mode orb is already in, so 完全無欠モード is one press away from
/// somebody who played a pointdevice run last time.
///
/// Which is the whole reason the cursor starts there rather than at the top: the answer most likely
/// to be wanted is the one that was wanted last time.
#[test]
fn a_run_that_was_pointdevice_is_pointdevice_again_on_one_press() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    let (mode, by) = chosen(asking.ready().press(keys::Z));
    assert_eq!(mode, Mode::Pointdevice);
    assert_eq!(by, By::Keyboard);
}

/// And somebody in レガシーモード reaches it by moving the cursor once. Down or up, either of them:
/// with two items each direction is the other one.
#[test]
fn a_run_that_was_legacy_reaches_pointdevice_with_one_move_either_way() {
    for step in [keys::DOWN, keys::UP] {
        let mut asking = Asking::about_a_run(Mode::Normal);
        asking.ready();
        assert!(
            asking.press(step).is_none(),
            "moving the cursor is not answering"
        );
        assert_eq!(chosen(asking.press(keys::Z)).0, Mode::Pointdevice);
    }
}

/// Choosing without moving gives what the cursor was on, which is the other mode. Asserted because
/// the two being in the order they are drawn in is what makes every other test here mean what it
/// says.
#[test]
fn the_cursor_answers_with_the_mode_it_is_on() {
    let mut asking = Asking::about_a_run(Mode::Normal);
    assert_eq!(chosen(asking.ready().press(keys::Z)).0, Mode::Normal);
}

/// Return answers as well as the shot key: this question is over the game's own menu, and the two
/// keys that choose an item there both have to choose here.
#[test]
fn either_key_the_game_decides_with_decides_here() {
    for key in [keys::Z, keys::RETURN] {
        let mut asking = Asking::about_a_run(Mode::Pointdevice);
        assert_eq!(chosen(asking.ready().press(key)).0, Mode::Pointdevice);
    }
}

/// Cancelling leaves neither chosen, on the bomb key the game's own menus read as back and on escape.
///
/// Which is not the same as choosing レガシーモード. The press that would have started a run was held
/// back, so a cancelled question is a run not started at all rather than one started without
/// chapters — see `mode_ui`'s module comment.
#[test]
fn cancelling_chooses_neither_mode() {
    for key in [keys::X, keys::ESCAPE] {
        let mut asking = Asking::about_a_run(Mode::Pointdevice);
        match asking.ready().press(key) {
            Some((Answer::Cancelled, By::Keyboard)) => {}
            other => panic!(
                "{:?} did not cancel: {}",
                key,
                match other {
                    Some((Answer::Chosen(mode), _)) => format!("chose {mode}"),
                    Some((Answer::Cancelled, by)) => format!("cancelled by the {by}"),
                    None => "nothing was answered".to_owned(),
                }
            ),
        }
    }
}

// ── The keys it will not read ────────────────────────────────────────────────────────────────────

/// Nothing is answered while the grace runs, and the key that would answer is down the whole time.
///
/// The press this question went up on is still down — it is the press that was held back — so a
/// question that read its keys on the frame it appeared would answer itself with that press. Held
/// off for [`INPUT_GRACE_FRAMES`] rather than one frame because somebody who pressed it twice meant
/// both presses for the game's menu.
#[test]
fn the_press_the_question_went_up_on_does_not_answer_it() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    asking.sim.keyboard().set(keys::Z, true);
    for frame in 0..INPUT_GRACE_FRAMES {
        assert!(
            asking.frame(Pad::default()).is_none(),
            "answered on frame {frame} of {INPUT_GRACE_FRAMES}, with the key held from the start"
        );
    }

    // And it does not answer the moment the grace ends either: the key never went down again, and
    // what a menu acts on is the edge.
    assert!(
        asking.idle(60).is_none(),
        "a key held from before the question went up became a press when the grace ran out"
    );

    // Let it up and press it again, which is what somebody actually does.
    asking.sim.keyboard().release_all();
    assert!(asking.frame(Pad::default()).is_none());
    assert_eq!(chosen(asking.press(keys::Z)).0, Mode::Pointdevice);
}

/// A key held down answers once and not once a frame. A question that answered every frame would
/// choose, and choose again, and the second answer would land on a run already started.
#[test]
fn a_held_key_answers_once() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    asking.ready();
    asking.sim.keyboard().set(keys::Z, true);
    assert_eq!(chosen(asking.frame(Pad::default())).0, Mode::Pointdevice);
    for frame in 0..30 {
        assert!(
            asking.frame(Pad::default()).is_none(),
            "answered again on frame {frame} with the key still down"
        );
    }
}

/// Alt-tabbed away, nothing is pressed however hard: typing in another window must not choose a mode,
/// and a key held while the game goes to the back must not be a press when it comes forward.
#[test]
fn a_window_that_is_not_in_front_reads_nothing() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    asking.ready();

    asking.sim.display().set_foreground(Hwnd(0x9999));
    asking.sim.keyboard().set(keys::Z, true);
    for frame in 0..30 {
        assert!(
            asking.frame(Pad::default()).is_none(),
            "answered on frame {frame} with another window in front"
        );
    }

    // Back in front with the key still down. That is not a press — the edge happened while orb was
    // reading nothing — so the question is still up.
    asking.sim.display().set_foreground(WINDOW);
    assert!(
        asking.idle(30).is_none(),
        "a key held through an alt-tab answered the question on the way back"
    );
}

/// And a host that will not answer about the keyboard at all reads as nothing down, rather than as
/// whatever was down last.
///
/// `GetKeyboardState` returns zero on a thread with no message queue. Taking the array anyway would
/// leave the last successful read standing, which is a key stuck down on a menu that acts on edges —
/// so the first refused frame would answer and the question would choose by itself.
#[test]
fn a_refused_keyboard_reads_as_nothing_down() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    asking.ready();
    asking.sim.keyboard().set(keys::Z, true);
    asking.sim.keyboard().refuse(true);
    for frame in 0..30 {
        assert!(
            asking.frame(Pad::default()).is_none(),
            "answered on frame {frame} while the host refused to say what was down"
        );
    }
}

// ── The pad ──────────────────────────────────────────────────────────────────────────────────────

/// The pad answers it, and the log says so. The game is frozen on these frames, so its own reading
/// of the pad is not running: without the question reading the sample orb's thread keeps, a pad does
/// nothing at all here while working perfectly on the game's own menu a keypress earlier.
#[test]
fn the_pad_answers_and_is_named_as_the_hand_that_did() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    asking.ready();
    let (mode, by) = chosen(asking.frame(Pad {
        decide: true,
        ..Pad::default()
    }));
    assert_eq!(mode, Mode::Pointdevice);
    assert_eq!(by, By::Pad);
}

/// A pad button held from before the question went up is not a press when the grace ends. The pad is
/// read every frame, grace or not, which is what makes the edge come out right — the shot button was
/// held to choose the item this is asked over.
#[test]
fn a_pad_button_held_from_before_the_question_does_not_answer_it() {
    let mut asking = Asking::about_a_run(Mode::Pointdevice);
    let held = Pad {
        decide: true,
        ..Pad::default()
    };
    for frame in 0..INPUT_GRACE_FRAMES + 30 {
        assert!(
            asking.frame(held).is_none(),
            "answered on frame {frame} with the pad button held from the start"
        );
    }
}

/// The pad moves the cursor too, so 完全無欠モード is reachable without touching the keyboard.
#[test]
fn the_pad_can_reach_pointdevice_from_legacy() {
    let mut asking = Asking::about_a_run(Mode::Normal);
    asking.ready();
    assert!(
        asking
            .frame(Pad {
                down: true,
                ..Pad::default()
            })
            .is_none()
    );
    assert!(asking.frame(Pad::default()).is_none());
    assert_eq!(
        chosen(asking.frame(Pad {
            decide: true,
            ..Pad::default()
        }))
        .0,
        Mode::Pointdevice
    );
}

// ── The other thing the same question asks ───────────────────────────────────────────────────────

/// The ranking is asked with the same keys and answers with the same two modes: one choice for both,
/// because the ranking of pointdevice runs and a pointdevice run are the same file.
#[test]
fn the_ranking_is_asked_the_same_way() {
    let mut asking = Asking::new(Menu::Scores, Mode::Pointdevice);
    assert_eq!(chosen(asking.ready().press(keys::Z)).0, Mode::Pointdevice);
}
