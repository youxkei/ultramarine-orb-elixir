//! The buffer the game's music is played out of.
//!
//! **A mirror of the slots and not an abstraction over them**, for the reason [`crate::d3d8`] is: what
//! must not cross is a decision. Which of the buffer's status bits mean what, that a lock starting at
//! zero never wraps, and how a capture is rejected where the streaming thread moved under it are all
//! `orb-core`'s, and are what an e2e test over the music is about.
//!
//! **Eight of them, which is every slot this file types**, out of `IDirectSoundBuffer`'s twenty-one:
//!
//! ```sh
//! $ grep -c 'offset_of!' crates/orb-api/src/dsound.rs
//! 8
//! ```
//!
//! Counted off the slot asserts, for the reason [`crate::d3d8`] gives: the padding fields are fields too.
//!
//! Each one's index is asserted at compile time against the one it has in `dsound.h`. The rest are
//! pointer-sized padding, and a call through one of those would be a call into an unrelated method with
//! the wrong signature.

use std::ffi::c_void;
use std::mem::offset_of;

use crate::{Hresult, LockedBuffer, SoundBuffer};

/// The status bits orb reads and the flag it plays with. `DSBSTATUS_*` and `DSBPLAY_LOOPING`, which are
/// DirectSound's own numbers.
pub const DSBSTATUS_PLAYING: u32 = 0x1;
pub const DSBSTATUS_BUFFER_LOST: u32 = 0x2;
pub const DSBSTATUS_LOOPING: u32 = 0x4;
pub const DSBPLAY_LOOPING: u32 = 0x1;

/// `GetCurrentPosition` — where the mixer is playing from and where the buffer says the next write goes,
/// with the result beside them.
///
/// Both cursors, though orb reads only the first: the slot fills in two and what a mirror hands back is
/// what the slot answers. Where the next write goes orb takes out of the game's own memory instead, that
/// being the number the streaming thread moves.
pub fn get_current_position(buffer: SoundBuffer) -> (Hresult, u32, u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.buffer_position(buffer);
    }
    host::get_current_position(buffer)
}

/// `GetStatus`.
pub fn get_status(buffer: SoundBuffer) -> (Hresult, u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.buffer_status(buffer);
    }
    host::get_status(buffer)
}

/// `Lock` over `bytes` from `offset`, and what it handed back.
///
/// Both halves of it, because that is what the slot answers: a lock that reaches the end of the buffer
/// comes back as two runs, and one starting at zero for the whole buffer never does. Which of those the
/// caller asks for and what it makes of a second half is above this line.
pub fn lock(buffer: SoundBuffer, offset: u32, bytes: u32, flags: u32) -> (Hresult, LockedBuffer) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.lock_buffer(buffer, offset, bytes, flags);
    }
    host::lock(buffer, offset, bytes, flags)
}

/// `Unlock`, handed back the two runs the lock gave out.
pub fn unlock(buffer: SoundBuffer, locked: LockedBuffer) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.unlock_buffer(buffer, locked);
    }
    host::unlock(buffer, locked);
}

/// `Play`, with the two reserved arguments the slot takes and the flags.
pub fn play(buffer: SoundBuffer, reserved: u32, priority: u32, flags: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.play_buffer(buffer, reserved, priority, flags);
    }
    host::play(buffer, reserved, priority, flags);
}

/// `Stop`.
pub fn stop(buffer: SoundBuffer) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.stop_buffer(buffer);
    }
    host::stop(buffer);
}

/// `SetCurrentPosition` — where the mixer is to play from next.
pub fn set_current_position(buffer: SoundBuffer, position: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_buffer_position(buffer, position);
    }
    host::set_current_position(buffer, position);
}

/// `Restore`, for a buffer the device going away has taken the memory of.
pub fn restore(buffer: SoundBuffer) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.restore_buffer(buffer);
    }
    host::restore(buffer);
}

/// `IDirectSoundBuffer`'s vtable, as far as the slots orb calls are concerned: the three `IUnknown` slots
/// first and then the interface's own.
///
/// The object each slot is called on is a `*mut c_void`. What it really is is the buffer the game's
/// streaming sound holds, and nothing above the seam looks inside one — the one word of it orb reads
/// rather than calls, the vtable pointer at its head, it reads through [`crate::mem`], because whether
/// that pointer lands in a mapped image is how a live buffer is told from the stale one left in a block
/// the game's allocator did not scrub.
#[repr(C)]
pub struct SoundBufferVtable {
    pub _iunknown: [usize; 3],
    pub _get_caps: usize,
    pub get_current_position: unsafe extern "system" fn(*mut c_void, *mut u32, *mut u32) -> Hresult,
    pub _slot_5_to_8: [usize; 4],
    pub get_status: unsafe extern "system" fn(*mut c_void, *mut u32) -> Hresult,
    pub _initialize: usize,
    pub lock: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        *mut *mut c_void,
        *mut u32,
        *mut *mut c_void,
        *mut u32,
        u32,
    ) -> Hresult,
    pub play: unsafe extern "system" fn(*mut c_void, u32, u32, u32) -> Hresult,
    pub set_current_position: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
    pub _slot_14_to_17: [usize; 4],
    pub stop: unsafe extern "system" fn(*mut c_void) -> Hresult,
    pub unlock:
        unsafe extern "system" fn(*mut c_void, *mut c_void, u32, *mut c_void, u32) -> Hresult,
    pub restore: unsafe extern "system" fn(*mut c_void) -> Hresult,
}

const fn slot(index: usize) -> usize {
    index * size_of::<usize>()
}

const _: () = {
    assert!(offset_of!(SoundBufferVtable, get_current_position) == slot(4));
    assert!(offset_of!(SoundBufferVtable, get_status) == slot(9));
    assert!(offset_of!(SoundBufferVtable, lock) == slot(11));
    assert!(offset_of!(SoundBufferVtable, play) == slot(12));
    assert!(offset_of!(SoundBufferVtable, set_current_position) == slot(13));
    assert!(offset_of!(SoundBufferVtable, stop) == slot(18));
    assert!(offset_of!(SoundBufferVtable, unlock) == slot(19));
    assert!(offset_of!(SoundBufferVtable, restore) == slot(20));
};

#[cfg(windows)]
use crate::real::dsound as host;

#[cfg(not(windows))]
mod host {
    use crate::{Hresult, LockedBuffer, SoundBuffer, no_windows};

    pub fn get_current_position(_buffer: SoundBuffer) -> (Hresult, u32, u32) {
        no_windows("dsound::get_current_position")
    }
    pub fn get_status(_buffer: SoundBuffer) -> (Hresult, u32) {
        no_windows("dsound::get_status")
    }
    pub fn lock(
        _buffer: SoundBuffer,
        _offset: u32,
        _bytes: u32,
        _flags: u32,
    ) -> (Hresult, LockedBuffer) {
        no_windows("dsound::lock")
    }
    pub fn unlock(_buffer: SoundBuffer, _locked: LockedBuffer) {
        no_windows("dsound::unlock")
    }
    pub fn play(_buffer: SoundBuffer, _reserved: u32, _priority: u32, _flags: u32) {
        no_windows("dsound::play")
    }
    pub fn stop(_buffer: SoundBuffer) {
        no_windows("dsound::stop")
    }
    pub fn set_current_position(_buffer: SoundBuffer, _position: u32) {
        no_windows("dsound::set_current_position")
    }
    pub fn restore(_buffer: SoundBuffer) {
        no_windows("dsound::restore")
    }
}
