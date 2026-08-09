//! The sound a track is streamed through, as much of one as orb reaches.
//!
//! **The buffer is answered through the seam and winmm's two functions are not**, which is the whole of
//! the shape here. Eight slots of `IDirectSoundBuffer` are `orb_api::dsound`'s, so what stands in for the
//! buffer is this crate answering them out of a `Vec` — no vtable, no object. `mmioSeek` and `mmioRead`
//! are found by name in the game's own copy of winmm and called through a transmuted address, so those
//! two are real functions and this is a real object as far as they are concerned.
//!
//! What the *address space* is told is where the buffer is — see [`Sound::install`] — because the one
//! thing orb reads rather than calls is the pointer at its head: a pointer into the image is what it takes
//! for the difference between a live object and the stale one left in a block the allocator did not scrub.

use std::cell::Cell;

use orb_api::dsound::{DSBPLAY_LOOPING, DSBSTATUS_LOOPING, DSBSTATUS_PLAYING};
use orb_api::{Hresult, Kind, LockedBuffer, SoundBuffer};

use crate::Sim;

/// The buffer the game's music is played out of, as a scenario's game keeps it.
///
/// Any address — orb reads it out of the game's memory and hands it back to the seam — with a word at it
/// holding [`BUFFER_VTABLE`], which is what makes `vtable_in_image` say the buffer is live. See
/// [`Sound::install`].
pub const BUFFER: SoundBuffer = SoundBuffer(0x0d50_0000);

/// And the vtable that word names, which nothing ever calls through: what has to be true of it is only
/// that it lies in a region mapped as [`Kind::Image`], since that is the whole of the difference between a
/// live COM object and a stale pointer in a freed block.
pub const BUFFER_VTABLE: usize = 0x0d51_0000;

/// `mmioSeek`'s origins, of which orb uses two.
const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;

/// What either of the two winmm functions answers where it could not do what it was asked.
const REFUSED: i32 = -1;

/// `S_OK` and `E_FAIL` as the buffer answers them: orb only asks whether the answer is negative.
const OK: i32 = 0;

/// The name orb asks for the two functions under, which is the game's own copy of the library.
const WINMM: &str = "winmm.dll";

/// A track being streamed: the wave file it is read out of, the buffer it is played from, and where each
/// of the two has got to.
pub struct Sound {
    /// The wave file's own sound, which is what `mmioRead` hands back.
    wave: Vec<u8>,
    /// Where the file handle is, which is what `mmioSeek` answers and moves.
    at: Cell<i32>,
    /// The buffer's contents. Boxed and never resized, so the address a `Lock` hands out stays put for as
    /// long as whoever took the lock is writing through it.
    buffer: std::cell::UnsafeCell<Box<[u8]>>,
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

/// # An intermittent that is still to find
///
/// One scenario of `orb-e2e`'s `the_music_across_a_restore` fails perhaps one run in eight on the
/// assertion below, this reading null where the sound was installed on the thread that asked for it.
/// Held against `HEAD` before the fake's fidelity pass: it flakes there too and at about the same rate,
/// so it is older than that work and nothing in it.
///
/// The one that used to sit beside it *is* fixed — an allocation landing inside a range the laid-out game
/// claims, which failed two runs in three until [`Sound::of`] began asking `Space::has_room` and
/// allocating again, and the addresses it was watched at are written down there.
///
/// The panic crosses an `extern` frame and aborts, so what this needs first is a run caught under a
/// debugger, or this saying which thread it was set on.
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
    /// Boxed and owned by whoever asked for it: winmm's two functions find it through a pointer — the
    /// handle the game keeps is this object's own address — so a value moved out of here would leave that
    /// pointer behind.
    ///
    /// **It used to have to dodge the game's own addresses and does not any more.** The buffer was a real
    /// object behind a vtable of Rust functions, and its address and its vtable's were both told to the
    /// address space — so a `Box` landing inside a range the game is laid out at was a mapping that would
    /// shadow the game's own memory, watched at 0x6ce8f0, 0x301df98 and 0x651398. Now the buffer is
    /// answered through the seam and the addresses it is known by are [`BUFFER`] and [`BUFFER_VTABLE`],
    /// which are numbers rather than allocations.
    pub fn of(wave: Vec<u8>, buffer: usize, notify: u32) -> Box<Self> {
        let sound = Box::new(Self {
            wave,
            at: Cell::new(0),
            buffer: std::cell::UnsafeCell::new(vec![0; buffer].into_boxed_slice()),
            size: buffer as u32,
            play: Cell::new(0),
            status: Cell::new(DSBSTATUS_PLAYING | DSBSTATUS_LOOPING),
            notify,
        });
        STREAMING.set(&raw const *sound);
        sound
    }

    /// Tells `sim` where this is: winmm's two functions, and the one word of the buffer that orb reads
    /// rather than asks for.
    ///
    /// **Four bytes and one**, deliberately: what the address space has to answer is the pointer at
    /// [`BUFFER`]'s head and that the vtable it names is in an image. A region no bigger than those cannot
    /// shadow anything else a scenario laid out, which a page-sized one might.
    pub fn install(&self, sim: &Sim) {
        sim.load_module(WINMM);
        sim.set_proc_address(WINMM, "mmioSeek", mmio_seek as *const () as usize);
        sim.set_proc_address(WINMM, "mmioRead", mmio_read as *const () as usize);
        sim.space().map(BUFFER.0, size_of::<usize>(), Kind::Private);
        sim.space().write::<usize>(BUFFER.0, BUFFER_VTABLE);
        sim.space().map(BUFFER_VTABLE, 1, Kind::Image);
    }

    /// The buffer as the game keeps it, which is the address orb reads out of the game's memory and hands
    /// back to the seam.
    pub fn buffer_object(&self) -> usize {
        BUFFER.0
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

    /// Where the play cursor is in the buffer, which is the other thing a restore has to put back:
    /// the bytes alone would be the same sound starting in the wrong place.
    pub fn play_cursor(&self) -> u32 {
        self.play.get()
    }

    /// The bytes in the buffer, which is what a chapter's music mostly *is*: reading these back either
    /// side of a restore is the whole of the question whether the sound came back byte for byte.
    pub fn buffered(&self) -> Vec<u8> {
        unsafe { (*self.buffer.get()).to_vec() }
    }

    /// The play cursor moved on by `bytes`, wrapping at the buffer's end, which is the mixer playing
    /// what is in there.
    ///
    /// A scenario saying so, the way [`heard_at`](Sound::heard_at) is: what puts a distance between the
    /// cursor and the offset the next chunk goes at is a mixer running on its own clock, and that
    /// distance is the whole of what a margin measures.
    pub fn plays_on(&self, bytes: u32) {
        self.play.set((self.play.get() + bytes) % self.size);
    }

    /// `StreamingSound::ServiceBuffer`: the next chunk of the file written into the buffer at `at`,
    /// wrapping at its end, and how many bytes that was.
    ///
    /// The offset is handed in rather than kept here because it is the *game's* — the field
    /// `orb_core::audio::Music::write_offset` names, in the game's own memory — and the same is true of
    /// the countdown this read moves: what a scenario has to move with the position is the game's to
    /// move. See `Fake::services_the_buffer`.
    pub fn tops_the_buffer_up(&self, at: u32) -> u32 {
        let mut chunk = vec![0u8; self.notify as usize];
        let read = self.reads(&mut chunk);
        let buffer = unsafe { &mut *self.buffer.get() };
        let size = buffer.len();
        for (offset, byte) in chunk[..read].iter().enumerate() {
            buffer[(at as usize + offset) % size] = *byte;
        }
        read as u32
    }

    /// What follows the file handle, copied out and the handle moved on: short at the file's end, which
    /// is the case a stream that has been sought past its own sound runs into.
    fn reads(&self, into: &mut [u8]) -> usize {
        let at = self.at.get().max(0) as usize;
        let read = self.wave.len().saturating_sub(at).min(into.len());
        into[..read].copy_from_slice(&self.wave[at..at + read]);
        self.at.set((at + read) as i32);
        read
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

/// The eight slots of `IDirectSoundBuffer`, as this host answers them — see [`crate::Sim`], which is
/// what `orb-api` calls and which calls these.
///
/// Reached through [`streaming`] rather than through a field of the `Sim`, the way winmm's two functions
/// are: the sound is a thread's, because a scenario's game is a thread's. Every one of them refuses a
/// handle that is not [`BUFFER`] — a scenario that had orb reading some other buffer would be a scenario
/// about nothing.
pub(crate) mod buffer {
    use super::{
        BUFFER, DSBPLAY_LOOPING, DSBSTATUS_LOOPING, DSBSTATUS_PLAYING, Hresult, LockedBuffer, OK,
        REFUSED, SoundBuffer, streaming,
    };

    /// Where the mixer is playing from, and where the buffer says the next write goes.
    ///
    /// The same number twice: the offset the game's own streaming thread writes at is one orb reads out
    /// of the game's memory and never asks the buffer for, so what DirectSound reports as its own write
    /// cursor is not that and nothing here has an opinion about it.
    pub fn position(handle: SoundBuffer) -> (Hresult, u32, u32) {
        if handle != BUFFER {
            return (REFUSED, 0, 0);
        }
        let play = streaming().play.get();
        (OK, play, play)
    }

    pub fn status(handle: SoundBuffer) -> (Hresult, u32) {
        if handle != BUFFER {
            return (REFUSED, 0);
        }
        (OK, streaming().status.get())
    }

    /// `Lock` over the whole buffer, which is the only lock orb takes: one starting at zero never wraps,
    /// so there is never a second run to hand back.
    pub fn lock(handle: SoundBuffer, offset: u32, bytes: u32) -> (Hresult, LockedBuffer) {
        let sound = streaming();
        if handle != BUFFER || offset != 0 || bytes != sound.size {
            return (REFUSED, LockedBuffer::default());
        }
        let first = unsafe { (*sound.buffer.get()).as_mut_ptr() } as usize;
        (
            OK,
            LockedBuffer {
                first,
                first_bytes: bytes,
                second: 0,
                second_bytes: 0,
            },
        )
    }

    pub fn play(handle: SoundBuffer, flags: u32) {
        if handle != BUFFER {
            return;
        }
        let looping = if flags & DSBPLAY_LOOPING != 0 {
            DSBSTATUS_LOOPING
        } else {
            0
        };
        streaming().status.set(DSBSTATUS_PLAYING | looping);
    }

    pub fn stop(handle: SoundBuffer) {
        if handle != BUFFER {
            return;
        }
        let sound = streaming();
        sound.status.set(sound.status.get() & !DSBSTATUS_PLAYING);
    }

    pub fn set_position(handle: SoundBuffer, at: u32) {
        if handle == BUFFER {
            streaming().play.set(at);
        }
    }
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
    let into = unsafe { std::slice::from_raw_parts_mut(into, bytes as usize) };
    sound.reads(into) as i32
}
