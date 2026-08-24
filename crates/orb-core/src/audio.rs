//! Rewinding the music along with everything else.
//!
//! The game streams BGM out of a file into a looping DirectSound buffer, topped
//! up by a background thread. Three pieces of that live outside the game's own
//! memory and so outside a plain snapshot: the bytes inside the sound buffer,
//! its play cursor, and where the game has read that file up to. Restoring
//! memory without them leaves the streaming bookkeeping pointing at a buffer
//! that has moved on, which is audible as a short loop repeating forever.
//!
//! **Which file, and how it is moved, is the one thing the two games do not share** — see [`Track`].

use orb_api::dsound::{self, DSBPLAY_LOOPING, DSBSTATUS_BUFFER_LOST, DSBSTATUS_PLAYING};
use orb_api::{SoundBuffer, module};

use crate::{detail, log};

const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;

/// The game's own copy, which is the one to ask: a second one loaded here would answer about a
/// file this process opened rather than the one the game is streaming from.
const WINMM: &str = "winmm.dll";
/// And the same of the library the other kind of stream is read through, which every process has.
const KERNEL32: &str = "kernel32.dll";

/// `mmioSeek` from winmm, which the game already has loaded.
type MmioSeek = unsafe extern "system" fn(usize, i32, i32) -> i32;
type MmioRead = unsafe extern "system" fn(usize, *mut u8, i32) -> i32;
/// `SetFilePointer` and `ReadFile` from kernel32, for the game that reads its own file.
///
/// The seek's origins are `FILE_BEGIN` and `FILE_CURRENT`, 0 and 1 — which are [`SEEK_SET`] and
/// [`SEEK_CUR`]'s own numbers, so the two kinds of stream take the same two arguments. It answers
/// `INVALID_SET_FILE_POINTER` where it will not say, and that is 0xffffffff, which is the -1 the other kind
/// answers with.
type SetFilePointer = unsafe extern "system" fn(usize, i32, *mut i32, u32) -> u32;
type ReadFile = unsafe extern "system" fn(usize, *mut u8, u32, *mut u32, *mut u8) -> i32;

/// Where a track's sound is and how the game's place in it is read and moved.
///
/// **Two shapes because the two games stream differently, and the difference is not orb's to hide.**
/// th06 opens a `.wav` per track through winmm and keeps the `HMMIO` it got back; th07 opens
/// `thbgm.dat` itself, keeps the Win32 handle, and reads every track out of that one file — so a position
/// in it is an absolute offset in the archive rather than an offset in the track's own sound. Everything
/// above works in differences of positions, which is what lets one shape serve both: the only place the
/// difference shows is that an offset [`Music::audible_offset`] answers for th07 means nothing in another
/// game's file, and a resume already keeps those per track.
#[derive(Clone, Copy)]
pub enum Track {
    /// th06's, seeked and read through the game's own winmm.
    Mmio(usize),
    /// th07's, seeked and read through kernel32 on the handle the game opened.
    OpenFile(usize),
}

impl Track {
    /// Returns a negative value if the position could not be read, which `restore` then leaves alone
    /// rather than seeking somewhere wrong.
    ///
    /// # Safety
    /// The handle must be the game's own, as read out of its memory.
    unsafe fn seek(self, offset: i32, origin: i32) -> i32 {
        match self {
            Self::Mmio(mmio) => unsafe { mmio_seek(mmio, offset, origin) },
            Self::OpenFile(file) => unsafe { file_seek(file, offset, origin) },
        }
    }

    /// The game's own file read the game's own way, so the bytes are the ones its streaming thread would
    /// have read next. Negative or zero where nothing was read.
    ///
    /// # Safety
    /// The handle must be the game's own, as read out of its memory.
    unsafe fn read(self, into: &mut [u8]) -> i32 {
        match self {
            Self::Mmio(mmio) => unsafe { mmio_read(mmio, into) },
            Self::OpenFile(file) => unsafe { file_read(file, into) },
        }
    }
}

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
    pub track: Track,
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
    file_position: i32,
    /// What `write_offset` held while all of the above was read.
    token: u32,
    /// Which track this was. The game deletes the stream when it changes tracks,
    /// and putting these bytes back into a freed buffer is audible as the music
    /// breaking up — the boss's own music starts a moment after the boss appears,
    /// so a chapter that began in between saves a stream that is about to go.
    identity: Option<u32>,
}

impl Saved {
    /// Whether anything was **audible** when this was captured, which is not the same as there having been a
    /// stream to capture: a game between one song and the next has stopped the buffer and not yet asked for
    /// the next track, and the stream, its source and the record it is on are all still there.
    ///
    /// A state chapters are taken in rather than a rare one: a stage's first chapter falls frames into the
    /// stage, where the scene before it has had its song taken down and the stage's own has not started — see
    /// [`crate::snapshot::Snapshot::restore`] for what a restore does about it.
    pub fn was_playing(&self) -> bool {
        self.playing
    }
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

        let file_position = unsafe { self.track.seek(0, SEEK_CUR) };
        if self.token()? != token {
            return None;
        }
        Some(Saved {
            bytes,
            play_cursor,
            // **And not whether it was going round**, which a played stream always is: see
            // [`Music::restore`], where a capture that found that bit clear was heard as the music stopping.
            playing: status & DSBSTATUS_PLAYING != 0,
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
    /// **Going round whatever the capture found**, which is the one thing here that is not put back as it was:
    /// a streaming buffer *is* a ring — the game's own thread writes into it ahead of the mixer and the mixer
    /// wraps at its end for as long as the track lasts — so `DSBPLAY_LOOPING` is what a played stream is and not
    /// a state a chapter was taken in. A launch of th07 retrying a resumed stage's first chapter had it back
    /// as `status 0x1`, playing with that bit clear: every number the restore asked for came back and the music
    /// stopped where the buffer ended, with the game's own thread still filling one nobody was playing.
    pub unsafe fn restore(&self, saved: &Saved) {
        dsound::stop(self.buffer);
        let locked = unsafe {
            self.with_locked_buffer(|locked| locked.copy_from_slice(&saved.bytes))
                .is_some()
        };
        dsound::set_current_position(self.buffer, saved.play_cursor);
        let seeked = if saved.file_position >= 0 {
            unsafe { self.track.seek(saved.file_position, SEEK_SET) }
        } else {
            saved.file_position
        };
        if saved.playing {
            dsound::play(self.buffer, 0, 0, DSBPLAY_LOOPING);
        }
        // **What was put back, and what the game's own bookkeeping says beside it.** A chapter whose track has
        // not changed is put back through this and nothing is written down about it, which is a restore whose
        // sound can only be reported as "the music stopped" — a launch of th07 retrying stage 4's first
        // chapter had exactly that to go on. The pair to watch is the file and the countdown the loop is taken
        // on: they are moved by the streaming thread together, and this puts one of them back while the memory
        // copy puts the other.
        let (asked, status) = dsound::get_status(self.buffer);
        let (_, cursor, _) = dsound::get_current_position(self.buffer);
        detail!(
            "sound: put back at {} in the file (asked {}), cursor {} (asked {}), \
             {} byte(s) left, next write at {:?}, {} locked, status {status:#x} (asked {asked}), \
             {}",
            seeked,
            saved.file_position,
            cursor,
            saved.play_cursor,
            orb_api::mem::read_committed::<u32>(self.bytes_left).unwrap_or_default(),
            self.token(),
            if locked { "buffer" } else { "nothing" },
            if saved.playing { "playing" } else { "silent" },
        );
    }

    /// The mixer stopped and nothing else touched, which is what a chapter that had a stream and no sound
    /// coming out of it comes back to.
    ///
    /// The buffer rather than the game's own call, that one taking the stream down with it: what has to be
    /// left standing is the stream the restore has just had the game make, and what has to go is only the
    /// sound the making of it started.
    ///
    /// # Safety
    /// Must run on the game's main thread with no thread suspended.
    pub unsafe fn silence(&self) {
        dsound::stop(self.buffer);
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
        let position = unsafe { self.track.seek(0, SEEK_CUR) };
        self.audible(position, write, play)
    }

    /// And the same of a capture rather than of the stream now playing, which is what is left of a
    /// chapter's sound once the stream that held it has been freed: the buffer and the cursor in it
    /// belong to an object that has gone, and the file is the same file.
    pub fn offset_of(&self, saved: &Saved) -> Option<i32> {
        self.audible(saved.file_position, saved.token, saved.play_cursor)
    }

    /// Where in the wav the sound audible at that instant begins, out of the three numbers that say
    /// it: how far the file has been read, where the next chunk goes in the buffer, and where the
    /// mixer is playing from.
    ///
    /// One function for the live stream and for a capture, so the two cannot work it out differently:
    /// a resume seeks to the second and a restore under another track to the first, and a chapter
    /// reached either way is the same place in the song.
    fn audible(&self, position: i32, write: u32, play: u32) -> Option<i32> {
        if self.buffer_size == 0 || play >= self.buffer_size || write >= self.buffer_size {
            return None;
        }
        let unplayed = (write + self.buffer_size - play) % self.buffer_size;
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
        let position = unsafe { self.track.seek(0, SEEK_CUR) };
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
        // Asked so that a buffer nobody can ask about is left alone, and the status itself is not read: what
        // this plays it back as is a ring, for [`Music::restore`]'s reason.
        let (asked, _) = dsound::get_status(self.buffer);
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
        if unsafe { self.track.seek(offset, SEEK_SET) } != offset {
            return false;
        }
        let mut chunk = vec![0u8; self.buffer_size as usize];
        // Short of a whole buffer is the song's end inside it, and the rest of the buffer is left
        // silent: the thread tops it up from wherever the file now is, which is what it does at the
        // end of any pass over the song.
        let read = unsafe { self.track.read(&mut chunk) };
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
        dsound::play(self.buffer, 0, 0, DSBPLAY_LOOPING);
        true
    }

    /// Locks the whole buffer and hands it over as one slice. DirectSound splits
    /// a lock that wraps the end of the buffer in two, which a lock starting at
    /// zero never does.
    unsafe fn with_locked_buffer(&self, body: impl FnOnce(&mut [u8])) -> Option<()> {
        let (mut asked, mut locked) = dsound::lock(self.buffer, 0, self.buffer_size, 0);
        // A buffer DirectSound has taken away, which happens when the device goes: restored once and
        // locked again, and given up on if that does not take. **No e2e test reaches this**, and none can
        // as things are — `orb_sim::Sound` answers every call, and one that refused on demand would be a
        // switch with nothing on the other side of it: what this arm does is give up, which is not a claim
        // an e2e test can hold orb to. The same is true of every other arm here that answers `None`.
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
unsafe fn mmio_seek(mmio: usize, offset: i32, origin: i32) -> i32 {
    static SEEK: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let seek = *SEEK.get_or_init(|| {
        // Neither of these two is reached by an e2e test, and the reason is the same one both ways round: a
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
unsafe fn mmio_read(mmio: usize, into: &mut [u8]) -> i32 {
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

/// And the other kind of stream's two, on the handle the game opened its own file with.
///
/// `SetFilePointer`'s answer is where the file now is, and `INVALID_SET_FILE_POINTER` where it refused —
/// 0xffffffff, which as a signed word is the -1 the winmm pair answers with, so every caller above reads
/// both kinds the same way. A position past 2GB would read as a refusal, and `thbgm.dat` is 23MB.
unsafe fn file_seek(file: usize, offset: i32, origin: i32) -> i32 {
    static SEEK: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let seek = *SEEK.get_or_init(|| {
        let address = module::proc_address(KERNEL32, "SetFilePointer");
        if address.is_none() {
            log!("audio: kernel32.dll has no SetFilePointer; the music will not rewind exactly");
        }
        address
    });
    let Some(seek) = seek else { return -1 };
    if file == 0 || file == usize::MAX {
        return -1;
    }
    let seek: SetFilePointer = unsafe { std::mem::transmute(seek) };
    let Ok(origin) = u32::try_from(origin) else {
        return -1;
    };
    let at = unsafe { seek(file, offset, std::ptr::null_mut(), origin) };
    at as i32
}

/// The same read, through the same handle: what the game's own streaming thread would have read next.
unsafe fn file_read(file: usize, into: &mut [u8]) -> i32 {
    static READ: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let read = *READ.get_or_init(|| {
        let address = module::proc_address(KERNEL32, "ReadFile");
        if address.is_none() {
            log!("audio: kernel32.dll has no ReadFile; a track cannot be started part way through");
        }
        address
    });
    let Some(read) = read else { return -1 };
    if file == 0 || file == usize::MAX {
        return -1;
    }
    let read: ReadFile = unsafe { std::mem::transmute(read) };
    let Ok(length) = u32::try_from(into.len()) else {
        return -1;
    };
    let mut got = 0u32;
    // Nothing read is nothing read, whether the call refused or the file had ended: what the caller does
    // with either is give up, and telling the two apart would be a `GetLastError` nobody reads.
    if unsafe {
        read(
            file,
            into.as_mut_ptr(),
            length,
            &raw mut got,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return -1;
    }
    i32::try_from(got).unwrap_or(-1)
}
