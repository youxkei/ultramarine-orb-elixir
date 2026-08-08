//! The real joystick, through winmm.
//!
//! **The two neutral types are winmm's own layouts**, so what crosses the seam is a transmute and not a
//! field-by-field copy: `JoyInfo` is handed to the game as the `JOYINFOEX` it asked to have filled, and
//! `JoyCaps` is written into the game's own `JOYCAPSA` at 0x69d760. The asserts below are what holds
//! that true — a field added on either side that moved anything would fail here rather than in the
//! game's memory.

use std::mem::offset_of;

use windows_sys::Win32::Media::Multimedia::{
    JOYCAPSA, JOYERR_NOERROR, JOYERR_PARMS, JOYERR_UNPLUGGED, JOYINFOEX, joyGetDevCapsA,
    joyGetPosEx,
};

use crate::{JoyCaps, JoyInfo, joyerr};

/// The sizes and every field orb reads or writes, held against winmm's own declarations.
///
/// **Sizes and offsets and not alignment**: `windows-sys` declares both of these packed and the neutral
/// pair is not, so the two disagree about the alignment of the struct as a whole and about nothing else —
/// every field of either is a word or an array of bytes at a word-aligned offset. Which is what these
/// assert, since it is what the transmutes below rest on. Nothing under-aligned ever reaches them: the
/// game's own `JOYINFOEX` is a local and its `JOYCAPSA` is at 0x69d760.
const _: () = {
    assert!(size_of::<JoyInfo>() == size_of::<JOYINFOEX>());
    assert!(offset_of!(JoyInfo, y) == offset_of!(JOYINFOEX, dwYpos));
    assert!(offset_of!(JoyInfo, buttons) == offset_of!(JOYINFOEX, dwButtons));
    assert!(offset_of!(JoyInfo, pov) == offset_of!(JOYINFOEX, dwPOV));
    assert!(size_of::<JoyCaps>() == size_of::<JOYCAPSA>());
    // The game passes 0x194 to `joyGetDevCapsA`, so this is the struct it means by that.
    assert!(size_of::<JoyCaps>() == 0x194);
    assert!(offset_of!(JoyCaps, name) == offset_of!(JOYCAPSA, szPname));
    assert!(offset_of!(JoyCaps, x_min) == offset_of!(JOYCAPSA, wXmin));
    assert!(offset_of!(JoyCaps, y_min) == offset_of!(JOYCAPSA, wYmin));
    assert!(offset_of!(JoyCaps, y_max) == offset_of!(JOYCAPSA, wYmax));
    assert!(offset_of!(JoyCaps, buttons) == offset_of!(JOYCAPSA, wNumButtons));
    assert!(offset_of!(JoyCaps, axes) == offset_of!(JOYCAPSA, wNumAxes));
    assert!(joyerr::NOERROR == JOYERR_NOERROR);
    assert!(joyerr::PARMS == JOYERR_PARMS);
    assert!(joyerr::UNPLUGGED == JOYERR_UNPLUGGED);
};

pub fn position(device: u32, flags: u32) -> (u32, JoyInfo) {
    let mut info: JOYINFOEX = unsafe { std::mem::zeroed() };
    info.dwSize = size_of::<JOYINFOEX>() as u32;
    info.dwFlags = flags;
    let result = unsafe { joyGetPosEx(device, &mut info) };
    (result, unsafe {
        std::mem::transmute::<JOYINFOEX, JoyInfo>(info)
    })
}

pub fn caps(device: u32) -> Option<JoyCaps> {
    let mut caps: JOYCAPSA = unsafe { std::mem::zeroed() };
    let read = unsafe { joyGetDevCapsA(device as usize, &mut caps, size_of::<JOYCAPSA>() as u32) };
    (read == JOYERR_NOERROR).then(|| unsafe { std::mem::transmute::<JOYCAPSA, JoyCaps>(caps) })
}
