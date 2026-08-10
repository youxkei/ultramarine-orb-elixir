//! The joystick winmm has, as a test plugs one in.
//!
//! **The host's and not the game's.** 紅魔郷 asks winmm for joystick 0 only where its own DirectInput
//! enumeration found no game controller — the controller it did find is laid out in the game's own memory,
//! `Image::controller` — so this is the device on the other branch, and the one orb's own menus have to
//! read to answer to the same pad the game does.
//!
//! What an e2e test says is what the two calls answer: where the stick is, and what the device is. Which is
//! the state orb would have read rather than a press it has interpreted — whether a stick pushed past its
//! dead zone is a direction is orb's answer to give, out of the caps below, and not the simulator's.

use std::sync::Mutex;

use orb_api::{JoyCaps, JoyInfo, joyerr};

/// A joystick 0, or none.
///
/// A socket with nothing in it is what one starts as, and there is nothing here to take a device back
/// out with: an e2e test about the empty socket is one that never plugs anything in, and a pad *unplugged*
/// mid-run is a third thing again — `JOYERR_UNPLUGGED` rather than `JOYERR_PARMS` — that nothing asks for
/// yet.
pub struct Joystick {
    /// What is plugged in, and `None` for a socket with nothing in it — which is what
    /// `JOYERR_PARMS` says and the case that is slow on a real machine.
    device: Mutex<Option<Device>>,
}

/// One device: what it is, and where it is being pushed.
struct Device {
    caps: JoyCaps,
    position: JoyInfo,
    /// Whether its caps read at all. A device that answers `joyGetPosEx` and whose
    /// `joyGetDevCapsA` fails is a device orb cannot describe and must not write the game's axis
    /// calibration from, which is a branch worth being able to ask for.
    describes_itself: bool,
    /// What `joyGetPosEx` answers about it: `JOYERR_NOERROR` for one that is there, and
    /// `JOYERR_UNPLUGGED` for one whose socket is still named but whose device has gone.
    result: u32,
}

impl Default for Joystick {
    fn default() -> Self {
        Self::new()
    }
}

impl Joystick {
    pub fn new() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }

    /// Plugs a pad in: this many buttons, and a Y axis running from `y_min` to `y_max`.
    ///
    /// The axis' bounds and not a direction, because the bounds are what orb reads to work out where
    /// the middle of the travel is: `GetControllerInput` puts the centre halfway between them with a
    /// dead zone of a quarter of the travel either side, and a pad measured against zeros is a game
    /// spending the whole run with two directions held.
    pub fn attach(&self, buttons: u32, y_min: u32, y_max: u32) {
        let caps = JoyCaps {
            manufacturer: 0x045e,
            product: 0x02ff,
            name: name(b"Controller (Xbox One For Windows)"),
            x_min: y_min,
            x_max: y_max,
            y_min,
            y_max,
            buttons,
            axes: 5,
            ..JoyCaps::default()
        };
        *self.device.lock().unwrap() = Some(Device {
            caps,
            // Centred, which is where a stick nobody is touching sits.
            position: JoyInfo {
                y: y_min + (y_max - y_min) / 2,
                pov: POV_CENTERED,
                ..JoyInfo::default()
            },
            describes_itself: true,
            result: joyerr::NOERROR,
        });
    }

    /// And a device that answers with no buttons and no axes, which is not a pad however much it
    /// answers.
    ///
    /// Windows leaves exactly one of these on joystick 0 while the pad it has sits in XInput's second
    /// slot: `mid=413d pid=2104`, every field zero. Measured with all three interfaces asked at once.
    pub fn attach_a_phantom(&self) {
        *self.device.lock().unwrap() = Some(Device {
            caps: JoyCaps {
                manufacturer: 0x413d,
                product: 0x2104,
                name: name(b"USB Gaming Controller"),
                ..JoyCaps::default()
            },
            position: JoyInfo::default(),
            describes_itself: true,
            result: joyerr::NOERROR,
        });
    }

    /// Pushes the buttons whose bits are set in `buttons`, and the Y axis to `y`.
    ///
    /// # Panics
    /// Where nothing is plugged in, there being nothing to push.
    pub fn pushes(&self, buttons: u32, y: u32) {
        let mut device = self.device.lock().unwrap();
        let device = device
            .as_mut()
            .expect("a joystick this e2e test plugged in");
        device.position.buttons = buttons;
        device.position.y = y;
    }

    /// And the hat — a d-pad — pushed this many hundredths of a degree clockwise from straight up, or
    /// [`POV_CENTERED`] for pushed nowhere.
    ///
    /// # Panics
    /// Where nothing is plugged in.
    pub fn pushes_the_hat(&self, pov: u32) {
        let mut device = self.device.lock().unwrap();
        let device = device
            .as_mut()
            .expect("a joystick this e2e test plugged in");
        device.position.pov = pov;
    }

    /// `joyGetPosEx`.
    pub fn position(&self, device: u32, flags: u32) -> (u32, JoyInfo) {
        let held = self.device.lock().unwrap();
        match held.as_ref().filter(|_| device == JOYSTICK_0) {
            Some(device) => (
                device.result,
                JoyInfo {
                    size: size_of::<JoyInfo>() as u32,
                    flags,
                    ..device.position
                },
            ),
            None => (joyerr::PARMS, JoyInfo::default()),
        }
    }

    /// `joyGetDevCapsA`.
    pub fn caps(&self, device: u32) -> Option<JoyCaps> {
        let held = self.device.lock().unwrap();
        held.as_ref()
            .filter(|device| device.describes_itself)
            .filter(|_| device == JOYSTICK_0)
            .map(|device| device.caps)
    }
}

/// The one joystick the game ever asks about.
const JOYSTICK_0: u32 = 0;

/// `JOY_POVCENTERED` — past a full circle, which is how winmm says a hat is pushed nowhere.
pub const POV_CENTERED: u32 = 0xffff;

/// A device's name as winmm answers one: the machine's own code page, terminated inside 32 bytes.
fn name(from: &[u8]) -> [u8; 32] {
    let mut field = [0u8; 32];
    let end = from.len().min(field.len() - 1);
    field[..end].copy_from_slice(&from[..end]);
    field
}
