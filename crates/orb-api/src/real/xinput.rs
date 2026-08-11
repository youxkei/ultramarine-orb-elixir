//! The real XInput, out of whichever of its libraries this machine has.
//!
//! **Loaded by name rather than imported**, because which of the three is present is a property of the
//! machine and a load-time import of one that is not there is a process that does not start at all. The
//! launcher does the same for its settings dialog and says so beside its own copy.
//!
//! The load happens on the first read and not at the attach: orb's `DllMain` runs with the loader lock
//! held and the game suspended, and a `LoadLibrary` there is the one thing that cannot be done from it.
//! The first read is the sampling thread's, long after both.

use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

use crate::XinputPad;

/// The three libraries XInput ships as, newest first. `xinput1_4.dll` is Windows 8 and later's,
/// `xinput1_3.dll` comes with the DirectX end-user runtime, and `xinput9_1_0.dll` is on everything
/// since Windows Vista.
const LIBRARIES: [&str; 3] = ["xinput1_4.dll", "xinput1_3.dll", "xinput9_1_0.dll"];

/// `XINPUT_GAMEPAD`, the whole of it, because that is what `XInputGetState` fills. What crosses the
/// seam is the three fields orb reads — see [`XinputPad`].
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Gamepad {
    buttons: u16,
    left_trigger: u8,
    right_trigger: u8,
    left_x: i16,
    left_y: i16,
    right_x: i16,
    right_y: i16,
}

/// `XINPUT_STATE`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct State {
    packet: u32,
    pad: Gamepad,
}

type XInputGetState = unsafe extern "system" fn(u32, *mut State) -> u32;

/// What the call answers for a slot that has a pad in it. Anything else is a slot with nobody in it —
/// 1167, `ERROR_DEVICE_NOT_CONNECTED`, for the empty ones.
const SUCCESS: u32 = 0;

pub fn state(slot: u32) -> Option<XinputPad> {
    let get = (*loaded())?;
    let mut state = State::default();
    if unsafe { get(slot, &mut state) } != SUCCESS {
        return None;
    }
    Some(XinputPad {
        buttons: state.pad.buttons,
        left_x: state.pad.left_x,
        left_y: state.pad.left_y,
    })
}

/// `XInputGetState` out of the first of [`LIBRARIES`] this machine has, looked for once. `None` where it
/// has none, which leaves winmm as the only interface — the way it was before orb read this one.
fn loaded() -> &'static Option<XInputGetState> {
    static FOUND: OnceLock<Option<XInputGetState>> = OnceLock::new();
    FOUND.get_or_init(|| {
        for name in LIBRARIES {
            let with_nul: Vec<u8> = name.bytes().chain([0]).collect();
            let library = unsafe { LoadLibraryA(with_nul.as_ptr()) };
            if library.is_null() {
                continue;
            }
            // The library is left loaded for the rest of the process, which is what a function pointer
            // into it needs.
            if let Some(symbol) =
                unsafe { GetProcAddress(library, c"XInputGetState".as_ptr().cast()) }
            {
                return Some(unsafe {
                    std::mem::transmute::<unsafe extern "system" fn() -> isize, XInputGetState>(
                        symbol,
                    )
                });
            }
        }
        None
    })
}
