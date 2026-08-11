//! The joysticks winmm has, as a test plugs them in.
//!
//! **The host's and not the game's.** 紅魔郷 asks winmm for joystick 0 only where its own DirectInput
//! enumeration found no game controller — the controller it did find is laid out in the game's own memory,
//! `Image::controller` — so this is the device on the other branch, and the one orb's own menus have to
//! read to answer to the same pad the game does.
//!
//! **Every index and not only joystick 0**, which is the one the game asks about. A pad the game has no
//! device for is one orb reads for itself, and winmm has room for [`SOCKETS`] of them — so a second pad is
//! at an index the game never asks about, and an e2e test can put one there.
//!
//! What an e2e test says is what the two calls answer: where the stick is, and what the device is. Which is
//! the state orb would have read rather than a press it has interpreted — whether a stick pushed past its
//! dead zone is a direction is orb's answer to give, out of the caps below, and not the simulator's.

use std::sync::Mutex;

use orb_api::{JoyCaps, JoyInfo, joyerr};

/// The sockets winmm has, one per index it will answer about.
///
/// A socket with nothing in it is what each starts as, and there is nothing here to take a device back
/// out with: an e2e test about the empty socket is one that never plugs anything in, and a pad *unplugged*
/// mid-run is a third thing again — `JOYERR_UNPLUGGED` rather than `JOYERR_PARMS` — that nothing asks for
/// yet.
pub struct Joystick {
    /// What is plugged into each socket, and `None` for one with nothing in it — which is what
    /// `JOYERR_PARMS` says and the case that is slow on a real machine.
    devices: Mutex<Vec<Option<Device>>>,
}

/// How many devices `joyGetNumDevs` answers with, which is how many sockets a simulated winmm has to
/// have: sixteen, because that is what the real one answers — see `Sample::is_a_pad`, where what a
/// machine reported for every index stands beside the phantom it found on the first.
pub const SOCKETS: u32 = 16;

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
            devices: Mutex::new((0..SOCKETS).map(|_| None).collect()),
        }
    }

    /// Plugs a pad into joystick 0, which is the one socket the game itself asks about: this many
    /// buttons, and a Y axis running from `y_min` to `y_max`.
    ///
    /// The axis' bounds and not a direction, because the bounds are what orb reads to work out where
    /// the middle of the travel is: `GetControllerInput` puts the centre halfway between them with a
    /// dead zone of a quarter of the travel either side, and a pad measured against zeros is a game
    /// spending the whole run with two directions held.
    pub fn attach(&self, buttons: u32, y_min: u32, y_max: u32) {
        self.attach_at(JOYSTICK_0, buttons, y_min, y_max);
    }

    /// And one into any of the sockets, for the pads the game has no device for.
    ///
    /// # Panics
    /// Where `device` is not one of winmm's [`SOCKETS`], there being no socket to plug into.
    pub fn attach_at(&self, device: u32, buttons: u32, y_min: u32, y_max: u32) {
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
        let centre = |low: u32, high: u32| low + (high - low) / 2;
        *socket(&mut self.devices.lock().unwrap(), device) = Some(Device {
            caps,
            // Centred, which is where a stick nobody is touching sits.
            position: JoyInfo {
                x: centre(caps.x_min, caps.x_max),
                y: centre(y_min, y_max),
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
        *socket(&mut self.devices.lock().unwrap(), JOYSTICK_0) = Some(Device {
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

    /// Pushes joystick 0's buttons whose bits are set in `buttons`, and its Y axis to `y`. The X axis
    /// is left where it is, that being the axis nothing on a menu reads.
    ///
    /// # Panics
    /// Where nothing is plugged in, there being nothing to push.
    pub fn pushes(&self, buttons: u32, y: u32) {
        let mut devices = self.devices.lock().unwrap();
        let device = plugged_in(&mut devices, JOYSTICK_0);
        device.position.buttons = buttons;
        device.position.y = y;
    }

    /// And any socket's buttons and both of its axes, which is what a pad the game has no device for
    /// needs: a player moves in four directions where a menu is a list.
    ///
    /// # Panics
    /// Where nothing is plugged into that socket.
    pub fn pushes_at(&self, device: u32, buttons: u32, x: u32, y: u32) {
        let mut devices = self.devices.lock().unwrap();
        let device = plugged_in(&mut devices, device);
        device.position.buttons = buttons;
        device.position.x = x;
        device.position.y = y;
    }

    /// And the hat — a d-pad — pushed this many hundredths of a degree clockwise from straight up, or
    /// [`POV_CENTERED`] for pushed nowhere.
    ///
    /// # Panics
    /// Where nothing is plugged in.
    pub fn pushes_the_hat(&self, pov: u32) {
        self.pushes_the_hat_at(JOYSTICK_0, pov);
    }

    /// And any socket's.
    ///
    /// # Panics
    /// Where nothing is plugged into that socket.
    pub fn pushes_the_hat_at(&self, device: u32, pov: u32) {
        let mut devices = self.devices.lock().unwrap();
        plugged_in(&mut devices, device).position.pov = pov;
    }

    /// `joyGetPosEx`.
    pub fn position(&self, device: u32, flags: u32) -> (u32, JoyInfo) {
        let held = self.devices.lock().unwrap();
        match held.get(device as usize).and_then(Option::as_ref) {
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
        let held = self.devices.lock().unwrap();
        held.get(device as usize)
            .and_then(Option::as_ref)
            .filter(|device| device.describes_itself)
            .map(|device| device.caps)
    }

    /// `joyGetNumDevs` — how many sockets there are, which is not how many have anything in them.
    pub fn count(&self) -> u32 {
        SOCKETS
    }
}

/// The socket at that index.
///
/// # Panics
/// Where `device` is not one of winmm's [`SOCKETS`], there being no socket to reach.
fn socket(devices: &mut [Option<Device>], device: u32) -> &mut Option<Device> {
    let sockets = devices.len();
    devices
        .get_mut(device as usize)
        .unwrap_or_else(|| panic!("winmm has no joystick {device}: it has {sockets} sockets"))
}

/// And the device in it, which is what an e2e test pushes.
///
/// # Panics
/// Where the socket is empty.
fn plugged_in(devices: &mut [Option<Device>], device: u32) -> &mut Device {
    socket(devices, device)
        .as_mut()
        .expect("a joystick this e2e test plugged in")
}

/// The one joystick the game ever asks about, and the one every e2e test that says only *the* pad means.
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
