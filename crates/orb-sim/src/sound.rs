//! The sound a track is streamed through, as much of one as orb reaches.
//!
//! **Both halves of it are real objects rather than laid-out memory**, and that is not a shortcut: orb
//! *dereferences* the buffer's pointer — a DirectSound buffer is a COM object and its vtable is called,
//! not read — and it asks winmm for the two functions it moves the file with. So what stands in for them
//! is a buffer of this crate's own behind a vtable of Rust functions, which is the same answer the
//! Direct3D device orb draws through is, and a `mmioSeek`/`mmioRead` pair over a wave file kept in a
//! `Vec`.
//!
//! What the *address space* is told is where that object is — see [`Sound::install`] — because the one
//! thing orb reads rather than calls is the pointer at its head: a pointer into the image is what it takes
//! for the difference between a live object and the stale one left in a block the allocator did not scrub.

use std::cell::{Cell, UnsafeCell};
use std::ffi::c_void;

use orb_api::Kind;

use crate::Sim;

/// Which slot of the buffer's vtable is which, as `orb_core::audio` lays `IDirectSoundBuffer` out: the
/// three `IUnknown` slots first and then the interface's own.
mod slot {
    pub const GET_CURRENT_POSITION: usize = 4;
    pub const GET_STATUS: usize = 9;
    pub const LOCK: usize = 11;
    pub const PLAY: usize = 12;
    pub const SET_CURRENT_POSITION: usize = 13;
    pub const STOP: usize = 18;
    pub const UNLOCK: usize = 19;
    pub const RESTORE: usize = 20;
    /// One past the last of them, which is how big the vtable has to be.
    pub const COUNT: usize = 21;
}

/// The two status bits orb reads, and the flag it plays with.
const DSBSTATUS_PLAYING: u32 = 0x1;
const DSBSTATUS_LOOPING: u32 = 0x4;
const DSBPLAY_LOOPING: u32 = 0x1;

/// `mmioSeek`'s origins, of which orb uses two.
const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;

/// What either of the two winmm functions answers where it could not do what it was asked.
const REFUSED: i32 = -1;

/// `S_OK` and `E_FAIL` as the buffer answers them: orb only asks whether the answer is negative.
const OK: i32 = 0;

/// The name orb asks for the two functions under, which is the game's own copy of the library.
const WINMM: &str = "winmm.dll";

/// The object orb is handed, which is a pointer to a pointer to functions.
#[repr(C)]
struct Object {
    vtable: *const usize,
}

/// A track being streamed: the wave file it is read out of, the buffer it is played from, and where each
/// of the two has got to.
pub struct Sound {
    /// The vtable first, because the object holds its address.
    vtable: Box<[usize; slot::COUNT]>,
    object: UnsafeCell<Object>,
    /// The wave file's own sound, which is what `mmioRead` hands back.
    wave: Vec<u8>,
    /// Where the file handle is, which is what `mmioSeek` answers and moves.
    at: Cell<i32>,
    /// The buffer's contents. Its length never changes, so the pointer `Lock` hands out stays put.
    buffer: UnsafeCell<Box<[u8]>>,
    /// How long that is, kept beside it: asking the boxed slice would mean a reference into the cell for
    /// nothing, and the length is what every `Lock` is checked against.
    size: u32,
    play: Cell<u32>,
    status: Cell<u32>,
    /// How much the streaming thread writes each time it is woken, which orb reads back but does not act
    /// on: it is the unit a margin is judged in.
    notify: u32,
}

thread_local! {
    /// The sound this thread is streaming, for the functions orb calls back into. Those are plain
    /// `extern` functions with nothing but the ABI's arguments, so where a real DirectSound would reach
    /// its own object this reaches the one that was installed.
    static STREAMING: Cell<*const Sound> = const { Cell::new(std::ptr::null()) };
}

fn streaming() -> &'static Sound {
    let sound = STREAMING.get();
    assert!(
        !sound.is_null(),
        "no sound has been installed on this thread"
    );
    unsafe { &*sound }
}

impl Sound {
    /// A track of `wave` bytes, played through a buffer of `buffer` bytes topped up `notify` at a time.
    ///
    /// Boxed and owned by whoever asked for it: what goes into the game's memory is the address of the
    /// object inside this, so a value moved out of here would leave that address behind.
    pub fn of(wave: Vec<u8>, buffer: usize, notify: u32) -> Box<Self> {
        let vtable = Box::new(filled_vtable());
        let sound = Box::new(Self {
            object: UnsafeCell::new(Object {
                vtable: vtable.as_ptr(),
            }),
            vtable,
            wave,
            at: Cell::new(0),
            buffer: UnsafeCell::new(vec![0; buffer].into_boxed_slice()),
            size: buffer as u32,
            play: Cell::new(0),
            status: Cell::new(DSBSTATUS_PLAYING | DSBSTATUS_LOOPING),
            notify,
        });
        STREAMING.set(&raw const *sound);
        sound
    }

    /// Tells `sim` where this is: winmm's two functions, and the two words of it that orb reads rather
    /// than calls.
    ///
    /// **Four bytes and one**, deliberately: what the address space has to answer is the pointer at the
    /// object's head and that the vtable it names is in the image. A region no bigger than those cannot
    /// shadow anything else a scenario laid out, which a page-sized one over a real allocation might.
    ///
    /// # Panics
    /// Through `Space::map`, where this process's heap put either of those two inside an address the game
    /// is laid out at — which is the one way this can go wrong, and the panic names both addresses. The
    /// game's own ranges are its 0x476000 data and the blocks from 0x3000000 up; a small allocation in a
    /// binary based at 0x400000 with a 10MB image is nowhere near either.
    pub fn install(&self, sim: &Sim) {
        sim.load_module(WINMM);
        sim.set_proc_address(WINMM, "mmioSeek", mmio_seek as *const () as usize);
        sim.set_proc_address(WINMM, "mmioRead", mmio_read as *const () as usize);
        sim.space()
            .map(self.buffer_object(), size_of::<usize>(), Kind::Private);
        sim.space()
            .write::<usize>(self.buffer_object(), self.vtable.as_ptr() as usize);
        sim.space()
            .map(self.vtable.as_ptr() as usize, 1, Kind::Image);
    }

    /// The buffer as the game keeps it: the address of the object, which is what orb dereferences.
    pub fn buffer_object(&self) -> usize {
        self.object.get() as usize
    }

    /// The file handle, as the game keeps one. Its own address, which is a number no other handle is.
    pub fn mmio(&self) -> usize {
        std::ptr::from_ref(self) as usize
    }

    pub fn buffer_size(&self) -> u32 {
        self.size
    }

    pub fn notify_size(&self) -> u32 {
        self.notify
    }

    /// Where the file handle is now, which is half of the pair a loop is taken on.
    pub fn position(&self) -> i32 {
        self.at.get()
    }

    /// Puts the stream where a track that has been playing for a while is: the file that far in, and the
    /// play cursor at the start of a buffer nothing has been played out of yet.
    ///
    /// Which is what makes the position orb writes down a number worth seeking to. A scenario saying so,
    /// the way it says the player was hit: the streaming thread is a thread, and what it would have done
    /// over minutes of a track is not something a scenario can wait for.
    pub fn heard_at(&self, offset: i32) {
        self.at.set(offset);
        self.play.set(0);
    }

    pub fn playing(&self) -> bool {
        self.status.get() & DSBSTATUS_PLAYING != 0
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        STREAMING.set(std::ptr::null());
    }
}

/// The vtable, with a function in every slot orb calls and nothing in the rest.
///
/// The empty ones are never reached — orb calls eight of the twenty-one — and a null in them is what says
/// so: a call through one would fault where a plausible function would quietly answer.
fn filled_vtable() -> [usize; slot::COUNT] {
    let mut slots = [0usize; slot::COUNT];
    slots[slot::GET_CURRENT_POSITION] = get_current_position as *const () as usize;
    slots[slot::GET_STATUS] = get_status as *const () as usize;
    slots[slot::LOCK] = lock as *const () as usize;
    slots[slot::PLAY] = play as *const () as usize;
    slots[slot::SET_CURRENT_POSITION] = set_current_position as *const () as usize;
    slots[slot::STOP] = stop as *const () as usize;
    slots[slot::UNLOCK] = unlock as *const () as usize;
    slots[slot::RESTORE] = restore as *const () as usize;
    slots
}

unsafe extern "system" fn get_current_position(
    _buffer: usize,
    play: *mut u32,
    write: *mut u32,
) -> i32 {
    let sound = streaming();
    unsafe {
        if !play.is_null() {
            play.write(sound.play.get());
        }
        // The write cursor DirectSound reports, which is not the offset the game's own streaming thread
        // keeps: orb reads that one out of the game's memory and never asks the buffer for it.
        if !write.is_null() {
            write.write(sound.play.get());
        }
    }
    OK
}

unsafe extern "system" fn get_status(_buffer: usize, status: *mut u32) -> i32 {
    unsafe { status.write(streaming().status.get()) };
    OK
}

/// `Lock` over the whole buffer, which is the only lock orb takes: one starting at zero never wraps, so
/// there is never a second part to hand back.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn lock(
    _buffer: usize,
    offset: u32,
    bytes: u32,
    first: *mut *mut c_void,
    first_bytes: *mut u32,
    second: *mut *mut c_void,
    second_bytes: *mut u32,
    _flags: u32,
) -> i32 {
    let sound = streaming();
    if offset != 0 || bytes != sound.buffer_size() {
        return REFUSED;
    }
    unsafe {
        first.write((*sound.buffer.get()).as_mut_ptr().cast());
        first_bytes.write(bytes);
        second.write(std::ptr::null_mut());
        second_bytes.write(0);
    }
    OK
}

unsafe extern "system" fn unlock(
    _buffer: usize,
    _first: *mut c_void,
    _first_bytes: u32,
    _second: *mut c_void,
    _second_bytes: u32,
) -> i32 {
    OK
}

unsafe extern "system" fn play(_buffer: usize, _reserved: u32, _priority: u32, flags: u32) -> i32 {
    let sound = streaming();
    let looping = if flags & DSBPLAY_LOOPING != 0 {
        DSBSTATUS_LOOPING
    } else {
        0
    };
    sound.status.set(DSBSTATUS_PLAYING | looping);
    OK
}

unsafe extern "system" fn stop(_buffer: usize) -> i32 {
    let sound = streaming();
    sound.status.set(sound.status.get() & !DSBSTATUS_PLAYING);
    OK
}

unsafe extern "system" fn set_current_position(_buffer: usize, at: u32) -> i32 {
    streaming().play.set(at);
    OK
}

/// Buffers are never lost here: what `Restore` is for is a device that has been taken away, which no
/// scenario does.
unsafe extern "system" fn restore(_buffer: usize) -> i32 {
    OK
}

/// `mmioSeek` over the wave file's own bytes, which answers where the handle ended up.
unsafe extern "system" fn mmio_seek(mmio: usize, offset: i32, origin: i32) -> i32 {
    let sound = streaming();
    if mmio != sound.mmio() {
        return REFUSED;
    }
    let at = match origin {
        SEEK_CUR => sound.at.get().checked_add(offset),
        SEEK_SET => Some(offset),
        _ => None,
    };
    match at.filter(|at| (0..=sound.wave.len() as i32).contains(at)) {
        Some(at) => {
            sound.at.set(at);
            at
        }
        None => REFUSED,
    }
}

/// And `mmioRead`, which hands back what follows the handle and moves it on — short at the file's end,
/// which is the case a stream that has been sought past its own sound runs into.
unsafe extern "system" fn mmio_read(mmio: usize, into: *mut u8, bytes: i32) -> i32 {
    let sound = streaming();
    if mmio != sound.mmio() || into.is_null() || bytes < 0 {
        return REFUSED;
    }
    let at = sound.at.get().max(0) as usize;
    let read = sound.wave.len().saturating_sub(at).min(bytes as usize);
    unsafe { std::ptr::copy_nonoverlapping(sound.wave[at..].as_ptr(), into, read) };
    sound.at.set((at + read) as i32);
    read as i32
}
