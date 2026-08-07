//! **`--sent-keys`: the game reading keys another program pressed, which is what drives an unwatched run.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine.
//!
//! What it takes to un-stub them: DirectInput's device is the game's, and what `DISCL_EXCLUSIVE |
//! DISCL_FOREGROUND` does to an injected key is the host's answer rather than orb's. The laid-out 紅魔郷
//! has neither, so the fake game needs a controller device that refuses injected keys the way a real
//! exclusive foreground acquire does, and orb's release of it has to be visible to the same host.
//!
//! Why it matters past automation: every measurement of a front end that nobody was there to press keys
//! at was taken through this, so a launch that cannot be driven is a launch that cannot be measured.

/// Keys another program sends reach the system and do not reach the game, until orb lets the device go.
///
/// Measured: keys injected with `SendInput` — tried carrying the virtual key with its scancode, and as
/// the scancode alone with `KEYEVENTF_SCANCODE` — are accepted by the system (`SendInput` returns **1**)
/// and not seen by the game, which sat idle into its attract demo twice. `Controller::GetInput` takes the
/// keyboard `DISCL_EXCLUSIVE | DISCL_FOREGROUND` and such a device does not see them.
///
/// `--sent-keys` has orb let that device go — `Unacquire`, `Release`, the pointer cleared, which is what
/// `Supervisor::RegisterChain` does with a device it cannot set up — and the game then reads
/// `GetKeyboardState`, its own other way, which does see them. The first press after that ended the
/// attract demo, which is what proved it.
#[test]
#[ignore = "the fake 紅魔郷 has no exclusive foreground keyboard for an injected key to be refused by"]
fn an_injected_key_reaches_the_game_only_once_orb_has_released_its_device() {
    todo!(
        "give the fake game a device that refuses injected keys, assert the game reads nothing, then \
         release it under --sent-keys and assert GetKeyboardState answers"
    )
}

/// A press has to be repeated, because two moments in the front end spend one on nothing.
///
/// Measured, two things about the timing: a press inside the title's own opening animation is spent on
/// nothing, and the attract demo eats one to leave. So what works is **a press every 1.1 seconds until
/// the log says the screen moved**, rather than one press per screen.
#[test]
#[ignore = "the fake 紅魔郷 has no opening animation and no attract demo to eat a press"]
fn a_press_is_repeated_until_the_screen_moves_because_two_moments_swallow_one() {
    todo!("run the opening animation and the attract demo and assert a single press moves neither")
}
