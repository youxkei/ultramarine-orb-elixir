//! Rewinding the music along with everything else.
//!
//! The game streams BGM from a `.wav` into a looping DirectSound buffer, topped
//! up by a background thread. Three pieces of that live outside the game's own
//! memory and so outside a plain snapshot: the bytes inside the sound buffer,
//! its play cursor, and the file position held by winmm's `HMMIO`. Restoring
//! memory without them leaves the streaming bookkeeping pointing at a buffer
//! that has moved on, which is audible as a short loop repeating forever.

use std::ffi::c_void;
use std::mem::offset_of;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

use crate::log::log;

const DSBSTATUS_PLAYING: u32 = 0x1;
const DSBSTATUS_BUFFER_LOST: u32 = 0x2;
const DSBSTATUS_LOOPING: u32 = 0x4;
const DSBPLAY_LOOPING: u32 = 0x1;

const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;

/// `mmioSeek` from winmm, which the game already has loaded.
type MmioSeek = unsafe extern "system" fn(HANDLE, i32, i32) -> i32;

#[repr(C)]
pub struct SoundBuffer {
    vtable: *const SoundBufferVtable,
}

#[repr(C)]
struct SoundBufferVtable {
    _iunknown: [usize; 3],
    _get_caps: usize,
    get_current_position: unsafe extern "system" fn(*mut SoundBuffer, *mut u32, *mut u32) -> i32,
    _slot_5_to_8: [usize; 4],
    get_status: unsafe extern "system" fn(*mut SoundBuffer, *mut u32) -> i32,
    _initialize: usize,
    lock: unsafe extern "system" fn(
        *mut SoundBuffer,
        u32,
        u32,
        *mut *mut c_void,
        *mut u32,
        *mut *mut c_void,
        *mut u32,
        u32,
    ) -> i32,
    play: unsafe extern "system" fn(*mut SoundBuffer, u32, u32, u32) -> i32,
    set_current_position: unsafe extern "system" fn(*mut SoundBuffer, u32) -> i32,
    _slot_14_to_17: [usize; 4],
    stop: unsafe extern "system" fn(*mut SoundBuffer) -> i32,
    unlock: unsafe extern "system" fn(*mut SoundBuffer, *mut c_void, u32, *mut c_void, u32) -> i32,
    restore: unsafe extern "system" fn(*mut SoundBuffer) -> i32,
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

/// The live BGM stream, located afresh each time: a new stage loads a new track
/// and with it a new buffer and file handle.
#[derive(Clone, Copy)]
pub struct Music {
    /// The streaming object itself, which the game replaces when it changes track.
    pub stream: usize,
    pub buffer: *mut SoundBuffer,
    pub buffer_size: u32,
    /// How much the streaming thread writes each time it is woken.
    pub notify_size: u32,
    pub mmio: HANDLE,
    /// Address of the offset the streaming thread will write the next chunk at.
    /// It moves exactly when that thread tops the buffer up, which makes it the
    /// token for "has the stream moved since I looked".
    pub write_offset: usize,
}

/// Where the play cursor sits relative to what the game has written.
///
/// The game refills the space the play cursor has just left, so in normal running
/// the next write goes a little *behind* the cursor. `behind` is that distance,
/// and it growing past a chunk is the streaming falling behind.
#[derive(Clone, Copy)]
pub struct Margin {
    pub behind: u32,
    pub notify_size: u32,
}

pub struct Saved {
    bytes: Vec<u8>,
    play_cursor: u32,
    playing: bool,
    looping: bool,
    file_position: i32,
    /// What `write_offset` held while all of the above was read.
    token: u32,
    /// Which track this was. The game deletes the stream when it changes tracks,
    /// and putting these bytes back into a freed buffer is audible as the music
    /// breaking up — the boss's own music starts a moment after the boss appears,
    /// so a chapter that began in between saves a stream that is about to go.
    identity: Option<u32>,
}

impl Music {
    /// Where the stream is up to. Comparing this before and after tells whether
    /// the streaming thread has run in between.
    pub fn token(&self) -> Option<u32> {
        crate::mem::read_committed(self.write_offset)
    }

    /// # Safety
    /// Must run with no game thread suspended: the streaming thread can be
    /// inside DirectSound, and `Lock` would then wait for a lock it cannot get.
    pub unsafe fn capture(&self, identity: Option<u32>) -> Option<Saved> {
        let vtable = unsafe { &*(*self.buffer).vtable };
        // Read either side of everything below, so a capture torn by the
        // streaming thread is rejected rather than saved.
        let token = self.token()?;

        let mut status = 0;
        if unsafe { (vtable.get_status)(self.buffer, &mut status) } < 0 {
            return None;
        }
        let mut play_cursor = 0;
        if unsafe { (vtable.get_current_position)(self.buffer, &mut play_cursor, &mut 0) } < 0 {
            return None;
        }

        let mut bytes = vec![0u8; self.buffer_size as usize];
        unsafe { self.with_locked_buffer(|locked| bytes.copy_from_slice(locked)) }?;

        let file_position = unsafe { mmio_seek(self.mmio, 0, SEEK_CUR) };
        if self.token()? != token {
            return None;
        }
        Some(Saved {
            bytes,
            play_cursor,
            playing: status & DSBSTATUS_PLAYING != 0,
            looping: status & DSBSTATUS_LOOPING != 0,
            file_position,
            token,
            identity,
        })
    }

    /// Whether `saved` still describes the stream that is playing: the same track,
    /// through the same objects.
    ///
    /// The track alone is not enough. A track can be taken down and started again —
    /// which is what a restore under another track has to do — and the one that comes
    /// back is the same wave file through a *new* stream and a *new* sound buffer.
    /// Going by the track there called `Stop` and `Lock` on a buffer DirectSound had
    /// freed, and the game stopped answering: the main thread inside a released
    /// object's lock while the new stream's thread held the real one.
    pub fn still_current(&self, saved: &Saved, live: &Music, identity: Option<u32>) -> bool {
        saved.identity.is_some()
            && saved.identity == identity
            && self.stream == live.stream
            && std::ptr::eq(self.buffer, live.buffer)
    }

    /// # Safety
    /// Must run after the game's memory has been restored, so the streaming
    /// bookkeeping and what this puts back describe the same instant. No game
    /// thread may be suspended, and `still_current` must hold.
    pub unsafe fn restore(&self, saved: &Saved) {
        let vtable = unsafe { &*(*self.buffer).vtable };
        unsafe {
            (vtable.stop)(self.buffer);
            self.with_locked_buffer(|locked| locked.copy_from_slice(&saved.bytes));
            (vtable.set_current_position)(self.buffer, saved.play_cursor);
            if saved.file_position >= 0 {
                mmio_seek(self.mmio, saved.file_position, SEEK_SET);
            }
            if saved.playing {
                let flags = if saved.looping { DSBPLAY_LOOPING } else { 0 };
                (vtable.play)(self.buffer, 0, 0, flags);
            }
        }
    }

    /// Where the play cursor is relative to the next write. `None` when nothing
    /// is playing.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub unsafe fn margin(&self) -> Option<Margin> {
        let vtable = unsafe { &*(*self.buffer).vtable };
        let mut status = 0;
        if unsafe { (vtable.get_status)(self.buffer, &mut status) } < 0
            || status & DSBSTATUS_PLAYING == 0
        {
            return None;
        }
        let mut play = 0;
        if unsafe { (vtable.get_current_position)(self.buffer, &mut play, &mut 0) } < 0 {
            return None;
        }
        let write = self.token()?;
        if self.buffer_size == 0 || play >= self.buffer_size || write >= self.buffer_size {
            return None;
        }
        Some(Margin {
            behind: (play + self.buffer_size - write) % self.buffer_size,
            notify_size: self.notify_size,
        })
    }

    /// Whether the game's memory, as copied into a snapshot, still describes the
    /// stream this was captured from.
    pub fn agrees_with_memory(&self, saved: &Saved) -> bool {
        self.token() == Some(saved.token)
    }

    /// Locks the whole buffer and hands it over as one slice. DirectSound splits
    /// a lock that wraps the end of the buffer in two, which a lock starting at
    /// zero never does.
    unsafe fn with_locked_buffer(&self, body: impl FnOnce(&mut [u8])) -> Option<()> {
        let vtable = unsafe { &*(*self.buffer).vtable };
        let mut first = std::ptr::null_mut();
        let mut first_bytes = 0;
        let mut second = std::ptr::null_mut();
        let mut second_bytes = 0;

        let mut locked = unsafe {
            (vtable.lock)(
                self.buffer,
                0,
                self.buffer_size,
                &mut first,
                &mut first_bytes,
                &mut second,
                &mut second_bytes,
                0,
            )
        };
        if locked < 0 {
            let mut status = 0;
            unsafe { (vtable.get_status)(self.buffer, &mut status) };
            if status & DSBSTATUS_BUFFER_LOST == 0 {
                return None;
            }
            unsafe { (vtable.restore)(self.buffer) };
            locked = unsafe {
                (vtable.lock)(
                    self.buffer,
                    0,
                    self.buffer_size,
                    &mut first,
                    &mut first_bytes,
                    &mut second,
                    &mut second_bytes,
                    0,
                )
            };
        }
        if locked < 0 || first.is_null() || first_bytes != self.buffer_size {
            if locked >= 0 {
                unsafe { (vtable.unlock)(self.buffer, first, first_bytes, second, second_bytes) };
            }
            return None;
        }

        body(unsafe { std::slice::from_raw_parts_mut(first.cast(), first_bytes as usize) });
        unsafe { (vtable.unlock)(self.buffer, first, first_bytes, second, second_bytes) };
        Some(())
    }
}

/// Returns a negative value if the position could not be read, which `restore`
/// then leaves alone rather than seeking somewhere wrong.
unsafe fn mmio_seek(mmio: HANDLE, offset: i32, origin: i32) -> i32 {
    static SEEK: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let seek = *SEEK.get_or_init(|| {
        let winmm: Vec<u16> = "winmm.dll".encode_utf16().chain([0]).collect();
        let module = unsafe { GetModuleHandleW(winmm.as_ptr()) };
        if module.is_null() {
            log!("audio: winmm.dll is not loaded; the music will not rewind exactly");
            return None;
        }
        let address = unsafe { GetProcAddress(module, c"mmioSeek".as_ptr().cast()) };
        if address.is_none() {
            log!("audio: winmm.dll has no mmioSeek");
        }
        address.map(|address| address as usize)
    });
    let Some(seek) = seek else { return -1 };
    if mmio.is_null() {
        return -1;
    }
    let seek: MmioSeek = unsafe { std::mem::transmute(seek) };
    unsafe { seek(mmio, offset, origin) }
}
