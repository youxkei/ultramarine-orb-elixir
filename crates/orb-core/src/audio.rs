//! Rewinding the music along with everything else.
//!
//! The game streams BGM from a `.wav` into a looping DirectSound buffer, topped
//! up by a background thread. Three pieces of that live outside the game's own
//! memory and so outside a plain snapshot: the bytes inside the sound buffer,
//! its play cursor, and the file position held by winmm's `HMMIO`. Restoring
//! memory without them leaves the streaming bookkeeping pointing at a buffer
//! that has moved on, which is audible as a short loop repeating forever.

use orb_api::dsound::{
    self, DSBPLAY_LOOPING, DSBSTATUS_BUFFER_LOST, DSBSTATUS_LOOPING, DSBSTATUS_PLAYING,
};
use orb_api::{SoundBuffer, module};

use crate::{detail, log};

/// The game's own `HMMIO`, read out of its memory and handed back to its own winmm. Opaque: orb
/// never asks what is inside one, so a plain word is the whole of what it has to be.
type Mmio = usize;

const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;

/// The game's own copy, which is the one to ask: a second one loaded here would answer about a
/// file this process opened rather than the one the game is streaming from.
const WINMM: &str = "winmm.dll";

/// `mmioSeek` from winmm, which the game already has loaded.
type MmioSeek = unsafe extern "system" fn(Mmio, i32, i32) -> i32;
type MmioRead = unsafe extern "system" fn(Mmio, *mut u8, i32) -> i32;

/// The live BGM stream, located afresh each time: a new stage loads a new track
/// and with it a new buffer and file handle.
#[derive(Clone, Copy)]
pub struct Music {
    /// The streaming object itself, which the game replaces when it changes track.
    pub stream: usize,
    pub buffer: SoundBuffer,
    pub buffer_size: u32,
    /// How much the streaming thread writes each time it is woken.
    pub notify_size: u32,
    pub mmio: Mmio,
    /// Address of the countdown the track's loop is taken on: how many bytes of sound the stream
    /// believes are left before it. The game subtracts every byte it reads from it and starts the
    /// track over when a read comes up short against it, so it and the file's position are a pair —
    /// moving one without the other is a track that loops in the wrong place, or not at all.
    pub bytes_left: usize,
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
        orb_api::mem::read_committed(self.write_offset)
    }

    /// # Safety
    /// Must run with no game thread suspended: the streaming thread can be
    /// inside DirectSound, and `Lock` would then wait for a lock it cannot get.
    pub unsafe fn capture(&self, identity: Option<u32>) -> Option<Saved> {
        // Read either side of everything below, so a capture torn by the
        // streaming thread is rejected rather than saved.
        let token = self.token()?;

        let (asked, status) = dsound::get_status(self.buffer);
        if asked < 0 {
            return None;
        }
        let (asked, play_cursor, _) = dsound::get_current_position(self.buffer);
        if asked < 0 {
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
            && self.buffer == live.buffer
    }

    /// # Safety
    /// Must run after the game's memory has been restored, so the streaming
    /// bookkeeping and what this puts back describe the same instant. No game
    /// thread may be suspended, and `still_current` must hold.
    pub unsafe fn restore(&self, saved: &Saved) {
        dsound::stop(self.buffer);
        unsafe {
            self.with_locked_buffer(|locked| locked.copy_from_slice(&saved.bytes));
            dsound::set_current_position(self.buffer, saved.play_cursor);
            if saved.file_position >= 0 {
                mmio_seek(self.mmio, saved.file_position, SEEK_SET);
            }
        }
        if saved.playing {
            let flags = if saved.looping { DSBPLAY_LOOPING } else { 0 };
            dsound::play(self.buffer, 0, 0, flags);
        }
    }

    /// Where the play cursor is relative to the next write. `None` when nothing
    /// is playing.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub unsafe fn margin(&self) -> Option<Margin> {
        let (asked, status) = dsound::get_status(self.buffer);
        if asked < 0 || status & DSBSTATUS_PLAYING == 0 {
            return None;
        }
        let (asked, play, _) = dsound::get_current_position(self.buffer);
        if asked < 0 {
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

    /// Where in the wav the sound now audible begins, as a file offset [`Music::play_from`] takes
    /// back. `None` where the stream will not say.
    ///
    /// One number rather than the buffer's contents, because this is what crosses a launch: the file
    /// is the same file next time, where the buffer is an object of this process. The streaming
    /// thread reads its next chunk from the file position, so what is audible began that position
    /// less everything sitting in the buffer unplayed.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub unsafe fn audible_offset(&self) -> Option<i32> {
        let (asked, play, _) = dsound::get_current_position(self.buffer);
        if asked < 0 {
            return None;
        }
        let write = self.token()?;
        if self.buffer_size == 0 || play >= self.buffer_size || write >= self.buffer_size {
            return None;
        }
        let unplayed = (write + self.buffer_size - play) % self.buffer_size;
        let position = unsafe { mmio_seek(self.mmio, 0, SEEK_CUR) };
        // Negative is `mmioSeek` refusing to say. Less than a buffer in is a song that has looped
        // since the position was read, and the offset before its own start is no place to seek to.
        u32::try_from(position)
            .ok()
            .filter(|position| *position >= unplayed)
            .map(|position| (position - unplayed) as i32)
    }

    /// The file offset the game will take the track's loop at: where the file is now, plus what the
    /// countdown says is left before it does.
    ///
    /// Asked of the two together rather than of the loop fields in the header, because which of them
    /// the game is going by is its own business — a track with no loop end runs to the end of its
    /// sound instead — and this is the number both answers come out as.
    ///
    /// # Safety
    /// Must run with the buffer stopped, so that neither half of the pair moves between the reads.
    unsafe fn loop_point(&self) -> Option<i32> {
        let position = unsafe { mmio_seek(self.mmio, 0, SEEK_CUR) };
        if position < 0 {
            return None;
        }
        let left = unsafe { orb_api::mem::read::<u32>(self.bytes_left) };
        i32::try_from(left).ok()?.checked_add(position)
    }

    /// Starts the track again from `offset`, the file offset of the sound to be heard next, and says
    /// whether it could.
    ///
    /// The buffer is filled from the file rather than left to the streaming thread, so that nothing
    /// of what it already held is heard first: after a resume that is the song's opening
    /// milliseconds, the stage having been built a frame ago. Everything is then where a freshly
    /// loaded track has it — the buffer holding one buffer's worth from `offset`, the play cursor and
    /// the next write at nothing, the file at what follows, and the countdown to the loop as far off
    /// as `offset` is — so the thread carries on without a seam and loops where the track does.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with no game thread suspended: this locks
    /// the buffer, which the streaming thread can be inside DirectSound holding.
    pub unsafe fn play_from(&self, offset: i32) -> bool {
        let (asked, status) = dsound::get_status(self.buffer);
        if asked < 0 {
            return false;
        }
        // Stopped before the file is touched rather than after it: the streaming thread seeks and
        // reads the same handle and moves the same countdown, on notifications a stopped buffer does
        // not raise. Every read below is of a pair that has to agree, and one of them moving in
        // between is a track that loops in the wrong place.
        dsound::stop(self.buffer);
        let Some(loop_point) = (unsafe { self.loop_point() }) else {
            return false;
        };
        if unsafe { mmio_seek(self.mmio, offset, SEEK_SET) } != offset {
            return false;
        }
        let mut chunk = vec![0u8; self.buffer_size as usize];
        // Short of a whole buffer is the song's end inside it, and the rest of the buffer is left
        // silent: the thread tops it up from wherever the file now is, which is what it does at the
        // end of any pass over the song.
        let read = unsafe { mmio_read(self.mmio, &mut chunk) };
        if read <= 0 {
            return false;
        }
        // The countdown moved to match the file, because what loops the track is that running out and
        // not the file ending. Left where it was, the stream believes it has as much left as it did
        // at `offset` bytes ago, reads past the end of the sound, and the read fails rather than
        // coming up short — so no loop is taken and the buffer is left going round its own contents,
        // for as long as it takes to count off the bytes that were skipped. Which is audible, and
        // was: seconds of the song repeating, once, near the end of a resumed stage.
        let left = (loop_point - offset - read).max(0) as u32;
        unsafe { orb_api::mem::write::<u32>(self.bytes_left, left) };
        detail!("music: the track loops at {loop_point}, so {left} byte(s) left from {offset}");
        if unsafe {
            self.with_locked_buffer(|locked| locked.copy_from_slice(&chunk))
                .is_none()
        } {
            return false;
        }
        dsound::set_current_position(self.buffer, 0);
        unsafe { orb_api::mem::write::<u32>(self.write_offset, 0) };
        let flags = if status & DSBSTATUS_LOOPING != 0 {
            DSBPLAY_LOOPING
        } else {
            0
        };
        dsound::play(self.buffer, 0, 0, flags);
        true
    }

    /// Locks the whole buffer and hands it over as one slice. DirectSound splits
    /// a lock that wraps the end of the buffer in two, which a lock starting at
    /// zero never does.
    unsafe fn with_locked_buffer(&self, body: impl FnOnce(&mut [u8])) -> Option<()> {
        let (mut asked, mut locked) = dsound::lock(self.buffer, 0, self.buffer_size, 0);
        // A buffer DirectSound has taken away, which happens when the device goes: restored once and
        // locked again, and given up on if that does not take. **No scenario reaches this**, and none can
        // as things are — `orb_sim::Sound` answers every call, and one that refused on demand would be a
        // switch with nothing on the other side of it: what this arm does is give up, which is not a claim
        // a scenario can hold orb to. The same is true of every other arm here that answers `None`.
        if asked < 0 {
            let (_, status) = dsound::get_status(self.buffer);
            if status & DSBSTATUS_BUFFER_LOST == 0 {
                return None;
            }
            dsound::restore(self.buffer);
            (asked, locked) = dsound::lock(self.buffer, 0, self.buffer_size, 0);
        }
        if asked < 0 || locked.first == 0 || locked.first_bytes != self.buffer_size {
            if asked >= 0 {
                dsound::unlock(self.buffer, locked);
            }
            return None;
        }

        // The run the lock handed over, as the bytes it is: where it is and how long it is are what the
        // slot answers, and what a capture or a restore does with it is this file's own.
        body(unsafe {
            std::slice::from_raw_parts_mut(locked.first as *mut u8, locked.first_bytes as usize)
        });
        dsound::unlock(self.buffer, locked);
        Some(())
    }
}

/// Returns a negative value if the position could not be read, which `restore`
/// then leaves alone rather than seeking somewhere wrong.
unsafe fn mmio_seek(mmio: Mmio, offset: i32, origin: i32) -> i32 {
    static SEEK: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let seek = *SEEK.get_or_init(|| {
        // Neither of these two is reached by a scenario, and the reason is the same one both ways round: a
        // laid-out stream is reached through `orb_sim::Sound::install`, which is what loads the library and
        // puts both functions in it, and orb needs the stream to get as far as this call at all. So the
        // shape this arm wants is a stream with no sound behind it, which nothing else needs.
        if !module::loaded(WINMM) {
            log!("audio: winmm.dll is not loaded; the music will not rewind exactly");
            return None;
        }
        let address = module::proc_address(WINMM, "mmioSeek");
        if address.is_none() {
            log!("audio: winmm.dll has no mmioSeek");
        }
        address
    });
    let Some(seek) = seek else { return -1 };
    if mmio == 0 {
        return -1;
    }
    let seek: MmioSeek = unsafe { std::mem::transmute(seek) };
    unsafe { seek(mmio, offset, origin) }
}

/// The game's own handle read through the game's own library, so the bytes are the ones its
/// streaming thread would have read next. Negative or zero where nothing was read.
unsafe fn mmio_read(mmio: Mmio, into: &mut [u8]) -> i32 {
    static READ: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let read = *READ.get_or_init(|| {
        if !module::loaded(WINMM) {
            log!("audio: winmm.dll is not loaded; a track cannot be started part way through");
            return None;
        }
        let address = module::proc_address(WINMM, "mmioRead");
        if address.is_none() {
            log!("audio: winmm.dll has no mmioRead");
        }
        address
    });
    let Some(read) = read else { return -1 };
    if mmio == 0 {
        return -1;
    }
    let read: MmioRead = unsafe { std::mem::transmute(read) };
    let Ok(length) = i32::try_from(into.len()) else {
        return -1;
    };
    unsafe { read(mmio, into.as_mut_ptr(), length) }
}
