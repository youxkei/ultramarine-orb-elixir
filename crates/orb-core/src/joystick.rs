//! What a sample of the pad means.
//!
//! **The sampling itself is `orb::joystick`**, which stands in front of the game's own `joyGetPosEx`
//! and takes a sample every few milliseconds on a thread of its own. This is the half that says what
//! one of those samples *is*: whether what answered is a pad at all, whether a read the game is making
//! is one a sample can answer, and what orb's own menus read off it.

use std::sync::Mutex;

use orb_api::{JoyCaps, JoyInfo, joyerr};

use crate::game::Reading;

/// `JOY_RETURNALL`, which is what a sample is taken with and what 紅魔郷 asks a read for.
pub const RETURN_ALL: u32 = 0xff;

/// The last sample, which is what the game's own read and orb's own menus are both answered out of.
static SAMPLE: Mutex<Option<Sample>> = Mutex::new(None);

#[derive(Clone, Copy)]
pub struct Sample {
    pub result: u32,
    pub info: JoyInfo,
    /// The device's own, taken when one starts answering and kept for as long as it does.
    ///
    /// Not read again beside every position: the caps belong to the device and not to the
    /// read, and what the game is handed must not change under it from one read to the next
    /// — that is a write into the game's memory each time it does. A pad swapped for another
    /// without one failing read in between would keep the first one's, which at a read every
    /// four milliseconds is not a way pads are swapped.
    pub caps: Option<JoyCaps>,
}

impl Sample {
    /// Whether what answered is a pad, which is not the same as something having answered.
    ///
    /// A device with no buttons and no axes has nothing to say, and joystick 0 is one of those on
    /// this machine whenever the pad is in XInput's second slot: `mid=413d pid=2104`, answering
    /// `joyGetPosEx` with every field zero. Measured with all three interfaces asked at once — winmm
    /// reports 16 devices, index 0 being that one and 1 to 15 `JOYERR_UNPLUGGED` at 13µs each;
    /// DirectInput enumerates `Controller (Xbox 360 Controller)`; XInput has it in slot 1 with slot 0
    /// empty. Believing it
    /// costs a line in the log claiming a pad answered, and the game's axis calibration written
    /// from a device that has no axes.
    pub fn is_a_pad(&self) -> bool {
        self.result == joyerr::NOERROR
            && self
                .caps
                .is_some_and(|caps| caps.buttons > 0 || caps.axes > 0)
    }
}

/// Writes down what the sampling thread just read, which is what everything below answers out of.
pub fn sampled(sample: Sample) {
    if let Ok(mut held) = SAMPLE.lock() {
        *held = Some(sample);
    }
}

/// Whether a sample taken with `RETURN_ALL` says everything this caller asked for.
pub fn describes_a_sample(asked: &JoyInfo) -> bool {
    asked.size as usize == size_of::<JoyInfo>() && asked.flags & !RETURN_ALL == 0
}

pub fn latest() -> Option<Sample> {
    *SAMPLE.lock().ok()?
}

/// The pad as it was last sampled, and `None` while none is answering.
///
/// For the menus orb puts up itself. Those freeze the game, so the game's own reading of the pad
/// is not running either and a pad would do nothing at all on them; the sample this thread already
/// takes every few milliseconds is there to be read.
pub fn reading() -> Option<Reading> {
    let sample = latest().filter(Sample::is_a_pad)?;
    Some(Reading {
        buttons: sample.info.buttons,
        y: sample.info.y,
        pov: sample.info.pov,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A device answering with no buttons and no axes is not a pad, and what makes that worth
    /// testing is that Windows leaves exactly one of those on joystick 0 while the pad it has
    /// sits in XInput's second slot.
    #[test]
    fn a_device_with_nothing_on_it_is_not_a_pad() {
        let mut caps = JoyCaps {
            x_max: 65535,
            ..JoyCaps::default()
        };
        let phantom = Sample {
            result: joyerr::NOERROR,
            info: JoyInfo::default(),
            caps: Some(caps),
        };
        assert!(!phantom.is_a_pad());

        // A stick with axes and no buttons is still a pad, and so is a wheel with buttons and
        // one axis: either can say something.
        caps.axes = 2;
        let stick = Sample {
            caps: Some(caps),
            ..phantom
        };
        assert!(stick.is_a_pad());

        // And nothing at all answering is not one either, whatever it left in the caps.
        let nothing = Sample {
            result: joyerr::UNPLUGGED,
            ..stick
        };
        assert!(!nothing.is_a_pad());
    }

    /// A sample is taken with `JOY_RETURNALL` into the current `JOYINFOEX`, which is what
    /// 紅魔郷 asks for. Anything asking for more than that has to go to winmm.
    #[test]
    fn a_sample_answers_what_the_game_asks_and_no_more() {
        let mut asked = JoyInfo {
            size: size_of::<JoyInfo>() as u32,
            flags: RETURN_ALL,
            ..JoyInfo::default()
        };
        assert!(describes_a_sample(&asked));

        // `JOY_RETURNRAWDATA`, which a sample does not carry.
        asked.flags = RETURN_ALL | 0x100;
        assert!(!describes_a_sample(&asked));

        // The struct before `JOYINFOEX` grew, which is not the one a sample fills.
        asked.flags = RETURN_ALL;
        asked.size -= 4;
        assert!(!describes_a_sample(&asked));
    }
}
