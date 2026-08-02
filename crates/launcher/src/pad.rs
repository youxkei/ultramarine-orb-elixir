//! The pad, for the settings dialog.
//!
//! A dialog takes no notice of a pad, and the person this one is put in front of is about to play a
//! game with one in their hands — reaching for the keyboard to answer it is the one awkward moment
//! in starting orb. So the pad is read here and turned into the messages the dialog manager already
//! understands, which is also what keeps the dialog a dialog: nothing about its controls is aware
//! of any of this.
//!
//! **On a thread of its own**, because `joyGetPosEx` is slow in a way that a message loop cannot
//! absorb: 15ms with a pad awake and 33ms with none on the machine this was written for, measured by
//! the other half of orb, which moved the same call off the game's frame for the same reason. The
//! thread posts what it sees and the dialog never waits for it.
//!
//! What the buttons mean is read out of the game's own configuration file rather than decided here,
//! so that the pad answers this dialog with the same buttons it will play the game with.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Media::Multimedia::{
    JOYCAPSA, JOYERR_NOERROR, JOYINFOEX, joyGetDevCapsA, joyGetPosEx,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

/// `JOY_RETURNALL`, which is what the game asks for and so what this asks for. Not in
/// `windows-sys`; the other half of orb spells it out for the same reason.
const RETURN_ALL: u32 = 0xff;

/// The message the thread posts, with a [`Push`] as its `wParam`.
pub const WM_PAD: u32 = WM_APP;

/// What the pad just did, in the terms the dialog is driven in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Push {
    /// The control before this one, and the one after: a pad moves through a dialog the way it
    /// moves through a game's menu, one item at a time.
    Previous,
    Next,
    /// Change what the control under the cursor says, which for the list of sizes is the size and
    /// for a switch is whether it is on.
    Less,
    More,
    /// The two buttons, whatever the cursor is on: the game's own shoot key decides everywhere and
    /// its bomb key goes back everywhere, and a dialog where that is not true is one to think about.
    Decide,
    Cancel,
}

impl Push {
    pub fn from_wparam(wparam: usize) -> Option<Self> {
        [
            Self::Previous,
            Self::Next,
            Self::Less,
            Self::More,
            Self::Decide,
            Self::Cancel,
        ]
        .into_iter()
        .nth(wparam)
    }
}

/// Which pad button is which, as the game has it.
///
/// Read out of the file the game keeps its configuration in — the first 18 bytes of that file are
/// its `ControllerMapping`, nine `i16`: shoot, bomb, focus, menu, up, down, left, right, skip — so
/// that this dialog answers to the buttons the game will answer to. An unmapped one is 0xffff
/// there, which as an `i16` is negative and names no button; the directions usually are, a stick
/// being how a pad is pushed.
///
/// **Shoot and menu decide; bomb cancels.** The game's own menus take
/// `TH_BUTTON_RETURNMENU = TH_BUTTON_MENU | TH_BUTTON_BOMB` as back, and following that put the menu
/// button on cancel — which on the pad this was written for is button 0, where a thumb rests. A
/// dialog that closes on the most obvious button is a dialog nobody can answer, and it took three
/// launches to see why. So the menu button decides here instead: orb's own menus have no pause for
/// it to open, and the button most easily reached should not be the destructive one.
pub struct Mapping {
    decide: [Option<u32>; 2],
    cancel: Option<u32>,
}

/// What the game falls back to when nobody has configured a pad, which is what its own defaults
/// are: the first button shoots and the second bombs.
const DEFAULT_DECIDE: u32 = 0;
const DEFAULT_CANCEL: u32 = 1;

impl Mapping {
    /// The mapping in `path`, and the game's own defaults where there is no readable file — a pad
    /// that answers the wrong way round is better than one that answers nothing.
    pub fn read(path: &Path) -> Self {
        let button = |bytes: &[u8], at: usize| {
            let value = i16::from_le_bytes([bytes[at], bytes[at + 1]]);
            u32::try_from(value)
                .ok()
                .filter(|button| *button < u32::BITS)
        };
        match std::fs::read(path) {
            Ok(bytes) if bytes.len() >= 18 => Self {
                // shoot and menu, and then bomb.
                decide: [button(&bytes, 0), button(&bytes, 6)],
                cancel: button(&bytes, 2),
            },
            _ => Self {
                decide: [Some(DEFAULT_DECIDE), None],
                cancel: Some(DEFAULT_CANCEL),
            },
        }
    }

    /// What it reads as, for the line the launcher prints: a pad that answers the wrong way round
    /// is a mapping to look at, and that is where it is written down.
    pub fn describe(&self) -> String {
        let name = |button: Option<u32>| match button {
            Some(button) => button.to_string(),
            None => "none".to_owned(),
        };
        format!(
            "decide {} or {}, cancel {}",
            name(self.decide[0]),
            name(self.decide[1]),
            name(self.cancel),
        )
    }

    fn decides(&self, buttons: u32) -> bool {
        self.decide.iter().any(|button| held(*button, buttons))
    }

    fn cancels(&self, buttons: u32) -> bool {
        held(self.cancel, buttons)
    }
}

fn held(button: Option<u32>, buttons: u32) -> bool {
    button.is_some_and(|button| buttons & (1 << button) != 0)
}

/// How long the thread waits between reads.
///
/// Short, because what is acted on is the press and a press is only ever seen if a read lands while
/// the button is down. The read itself costs 15 to 33ms, so a cycle is that plus this — and at the
/// 120ms this started as, a cycle could be 155ms and a quick tap fell between two of them and was
/// never seen at all. Which is what a pad that answers sometimes looks like.
const POLL: std::time::Duration = std::time::Duration::from_millis(8);

/// The one device the game reads, so the one this reads.
const DEVICE: u32 = 0;

static STOP: AtomicBool = AtomicBool::new(false);
/// Whether a pad ever answered, and how many pushes were posted. For the line the launcher prints:
/// without them, a pad that did nothing is indistinguishable from a pad that was never there, and
/// the two want opposite things done about them.
static SEEN: AtomicBool = AtomicBool::new(false);
static PUSHES: AtomicUsize = AtomicUsize::new(0);

/// What the watch came to, for the launcher to print.
pub fn report() -> String {
    match (SEEN.load(Ordering::Relaxed), PUSHES.load(Ordering::Relaxed)) {
        (false, _) => "no pad answered".to_owned(),
        (true, 0) => "a pad answered but was never pushed".to_owned(),
        (true, pushes) => format!("a pad, pushed {pushes} time(s)"),
    }
}

/// Reads the pad until [`stop`] is called, posting to `dialog` what it sees.
pub fn watch(dialog: HWND, mapping: Mapping) {
    let dialog = dialog as isize;
    STOP.store(false, Ordering::Relaxed);
    SEEN.store(false, Ordering::Relaxed);
    PUSHES.store(0, Ordering::Relaxed);
    std::thread::spawn(move || {
        let mut before = Pushed::default();
        let mut caps = None;
        while !STOP.load(Ordering::Relaxed) {
            let answered = read(&mapping, &mut caps);
            if answered.is_some() {
                SEEN.store(true, Ordering::Relaxed);
            }
            let now = answered.unwrap_or_default();
            for push in now.since(before) {
                PUSHES.fetch_add(1, Ordering::Relaxed);
                // Posted rather than sent: this is not the dialog's thread, and a dialog being
                // driven must not be waited on by whoever is driving it.
                unsafe { PostMessageW(dialog as HWND, WM_PAD, push as usize, 0) };
            }
            before = now;
            std::thread::sleep(POLL);
        }
    });
}

/// Ends the watch. The thread is asked rather than killed, and the dialog it posts to going away
/// is not a fault: a post to a window that has gone simply fails.
pub fn stop() {
    STOP.store(true, Ordering::Relaxed);
}

/// Which of the pushes are being held now, so that what is acted on is the press.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Pushed([bool; 6]);

impl Pushed {
    fn since(self, before: Self) -> Vec<Push> {
        self.0
            .iter()
            .zip(before.0)
            .enumerate()
            .filter(|(_, (now, before))| **now && !before)
            .filter_map(|(index, _)| Push::from_wparam(index))
            .collect()
    }
}

/// The pad as it is now, and `None` when none answers — which is also what a pad being unplugged
/// mid-dialog looks like, and is why a failed read is every push released rather than nothing at
/// all.
fn read(mapping: &Mapping, caps: &mut Option<JOYCAPSA>) -> Option<Pushed> {
    let mut info: JOYINFOEX = unsafe { std::mem::zeroed() };
    info.dwSize = size_of::<JOYINFOEX>() as u32;
    info.dwFlags = RETURN_ALL;
    if unsafe { joyGetPosEx(DEVICE, &mut info) } != JOYERR_NOERROR {
        *caps = None;
        return None;
    }
    // The device's own travel, which is what says where the middle of a stick is. Asked once per
    // device rather than beside every position: it belongs to the device and not to the read.
    if caps.is_none() {
        let mut asked: JOYCAPSA = unsafe { std::mem::zeroed() };
        if unsafe { joyGetDevCapsA(DEVICE as usize, &mut asked, size_of::<JOYCAPSA>() as u32) }
            == JOYERR_NOERROR
        {
            *caps = Some(asked);
        }
    }
    // The Y axis is measured downwards, so its low side is up — which `axis` reporting the low side
    // first is exactly what says.
    let (stick_up, stick_down) = caps.map_or((false, false), |caps| {
        axis(caps.wYmin, caps.wYmax, info.dwYpos)
    });
    let (stick_left, stick_right) = caps.map_or((false, false), |caps| {
        axis(caps.wXmin, caps.wXmax, info.dwXpos)
    });
    let (hat_up, hat_down, hat_left, hat_right) = hat(info.dwPOV);
    Some(Pushed([
        stick_up || hat_up,
        stick_down || hat_down,
        stick_left || hat_left,
        stick_right || hat_right,
        mapping.decides(info.dwButtons),
        mapping.cancels(info.dwButtons),
    ]))
}

/// Which way a hat — a d-pad — is pushed, as `(up, down, left, right)`.
///
/// Its own field rather than the axes, because that is where a d-pad reports: hundredths of a degree
/// clockwise from straight up, and `JOY_POVCENTERED` — 0xffff, which is past a full circle — for
/// pushed nowhere. A diagonal counts as both of its two, which is what a menu wants: a hat held
/// up-and-left should still move up.
fn hat(pov: u32) -> (bool, bool, bool, bool) {
    /// A full circle, and an eighth of one either side of each direction.
    const CIRCLE: u32 = 36000;
    const EIGHTH: u32 = CIRCLE / 8;

    if pov > CIRCLE {
        return (false, false, false, false);
    }
    (
        pov <= EIGHTH || pov >= CIRCLE - EIGHTH,
        (CIRCLE / 2 - EIGHTH..=CIRCLE / 2 + EIGHTH).contains(&pov),
        (CIRCLE * 3 / 4 - EIGHTH..=CIRCLE * 3 / 4 + EIGHTH).contains(&pov),
        (CIRCLE / 4 - EIGHTH..=CIRCLE / 4 + EIGHTH).contains(&pov),
    )
}

/// Whether an axis is pushed past its dead zone, as `(low, high)` in the position it reports —
/// which for the Y axis, measured downwards, is `(up, down)`.
///
/// The centre is halfway between the device's bounds and the dead zone a quarter of the travel
/// either side, which is what the game does with the same two numbers.
fn axis(low: u32, high: u32, position: u32) -> (bool, bool) {
    if high <= low {
        return (false, false);
    }
    let centre = low + (high - low) / 2;
    let dead = (high - low) / 4;
    (position + dead < centre, position > centre + dead)
}

#[cfg(test)]
mod tests {
    use super::{Mapping, Push, Pushed, axis, hat};

    /// Up is up: the Y axis is measured downwards, so its low side is the stick pushed up, and the
    /// dialog moving the wrong way is what getting this backwards looks like.
    #[test]
    fn the_low_side_of_the_y_axis_is_up() {
        let (up, down) = axis(0, 65535, 0);
        assert!(up && !down);
        let (up, down) = axis(0, 65535, 65535);
        assert!(down && !up);
    }

    /// A hat reports where it points, in hundredths of a degree clockwise from up.
    #[test]
    fn a_hat_points_where_it_says() {
        assert_eq!(hat(0), (true, false, false, false));
        assert_eq!(hat(9000), (false, false, false, true));
        assert_eq!(hat(18000), (false, true, false, false));
        assert_eq!(hat(27000), (false, false, true, false));
        // A diagonal is both of its two, so a hat held up-and-left still moves up.
        assert_eq!(hat(31500), (true, false, true, false));
        // 0xffff is pushed nowhere, which is past a full circle.
        assert_eq!(hat(0xffff), (false, false, false, false));
    }

    /// A stick has to leave the middle quarter either way, and the numbers are the device's own
    /// rather than assumed: this is the same arithmetic the game does.
    #[test]
    fn a_stick_has_a_dead_zone_a_quarter_of_its_travel() {
        assert_eq!(axis(0, 65535, 32767), (false, false));
        assert_eq!(axis(0, 65535, 0), (true, false));
        assert_eq!(axis(0, 65535, 65535), (false, true));
        // Just inside the dead zone either way.
        assert_eq!(axis(0, 65535, 32767 - 16000), (false, false));
        assert_eq!(axis(0, 65535, 32767 + 16000), (false, false));
    }

    /// A device that says its travel is nothing has no middle to be off.
    #[test]
    fn an_axis_with_no_travel_is_never_pushed() {
        assert_eq!(axis(0, 0, 0), (false, false));
        assert_eq!(axis(100, 100, 100), (false, false));
    }

    /// Only the press, since holding a button is how somebody arrives at this dialog: they have
    /// just launched orb with it.
    #[test]
    fn a_held_button_is_one_push() {
        let held = Pushed([false, false, false, false, true, false]);
        assert_eq!(held.since(Pushed::default()), vec![Push::Decide]);
        assert_eq!(held.since(held), vec![]);
    }

    /// No file to read is the game's own defaults rather than a pad that does nothing.
    #[test]
    fn a_missing_configuration_leaves_the_pad_working() {
        let mapping = Mapping::read(std::path::Path::new("no such file"));
        assert!(mapping.decides(1 << 0));
        assert!(mapping.cancels(1 << 1));
        assert!(!mapping.decides(1 << 4));
    }

    /// The menu button decides rather than cancelling. It is button 0 on the pad this was written
    /// for, where a thumb rests, and a dialog that closes on that button cannot be answered at all.
    #[test]
    fn the_menu_button_is_not_a_cancel() {
        let dir = std::env::temp_dir().join(format!("orb-pad-menu-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("game.cfg");
        let mut bytes = Vec::new();
        for button in [2i16, 5, 4, 0, -1, -1, -1, -1, 1] {
            bytes.extend_from_slice(&button.to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();

        let mapping = Mapping::read(&path);
        // Menu is 0 here, and so is shoot's opposite number: both decide.
        assert!(mapping.decides(1 << 0));
        assert!(mapping.decides(1 << 2));
        assert!(!mapping.cancels(1 << 0));
        // Bomb, and nothing else, cancels.
        assert!(mapping.cancels(1 << 5));
        assert!(!mapping.cancels(1 << 4));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The mapping as the game keeps it: nine `i16` at the front of the file, and 0xffff for a
    /// button nobody assigned.
    #[test]
    fn the_configuration_says_which_button_is_which() {
        let dir = std::env::temp_dir().join(format!("orb-pad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("game.cfg");
        // shoot 2, bomb 5, focus 4, menu 0, and the directions unmapped.
        let mut bytes = Vec::new();
        for button in [2i16, 5, 4, 0, -1, -1, -1, -1, 1] {
            bytes.extend_from_slice(&button.to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();

        let mapping = Mapping::read(&path);
        assert!(mapping.decides(1 << 2));
        assert!(mapping.cancels(1 << 5));
        // focus and skip are nobody's business here.
        assert!(!mapping.decides(1 << 4));
        assert!(!mapping.cancels(1 << 1));

        std::fs::remove_dir_all(&dir).ok();
    }
}
