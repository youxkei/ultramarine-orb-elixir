//! Whole-region save and restore of the game's state.
//!
//! Returning to a chapter start is done by putting the memory back exactly as it
//! was, rather than by asking the game to jump somewhere. Nothing has to
//! understand what a boss's script was in the middle of, because every byte of
//! it — including the allocator's bookkeeping, so pointers stay valid — comes
//! back unchanged.
//!
//! Deliberately not covered: Direct3D and DirectSound objects. Those cannot be copied,
//! so the memory holding handles to them is left as the restore finds it — see
//! `Game::live_handles`. A handle put back from a snapshot names something that may since
//! have been released, and the game releasing it a second time faults inside itself: which
//! is what a step back across the frame a boss's graphics are loaded on used to do.
//!
//! **What leaving them costs is cosmetic and is the price of not faulting.** For the frames between
//! such a restore and the game loading those graphics again, the sprite tables the snapshot put back
//! describe one set of graphics while the slots hold another, so something may be drawn from the wrong
//! texture until the load catches up. Nothing can be done about it from here: the two cannot be made
//! consistent without copying objects that cannot be copied.
//!
//! Left out for a different reason: `AnmManager`'s surfaces and its vertex buffer. The game releases
//! those when it loses the device, which a stage does not run into, and a range left out of a restore is
//! a range the snapshot no longer describes at all.

use std::ops::Range;

use crate::audio;
use crate::memtrack::Region;
use crate::{crash, detail, log};
use orb_api::{mem, thread};

/// How a snapshot treats the game's sound.
pub struct Audio {
    pub policy: Music,
    /// Which track is playing, so a restore can tell whether the stream it saved
    /// still exists.
    pub identity: Option<u32>,
    /// Where the sound system's state lives, for a restore that has to leave it
    /// alone.
    pub state: Vec<Range<usize>>,
    /// The game's audio thread, which is not suspended while copying.
    pub thread: Option<u32>,
}

/// What a snapshot does about the music.
pub enum Music {
    /// Save the stream and put it back, so the music returns to where it was.
    Rewind(Option<audio::Music>),
    /// Leave the sound system's memory as it is on restore, so the music plays on.
    KeepPlaying,
}

/// How many times to retry a capture the streaming thread interrupted. It tops
/// the buffer up a few times a second, so one retry is almost always enough.
const AUDIO_ATTEMPTS: u32 = 4;

/// What memory is handed out and taken back in, which is the step a walk of a saved range asks the host
/// about: a page is the smallest thing that can have changed hands.
const PAGE: usize = 0x1000;

/// What a restore has left for the frame's own sound to finish: the song put where the chapter had it, or the
/// track it had to ask for taken back down again.
///
/// **Waiting rather than done, because the ask is a command in the game's own sound queue** — see
/// [`Snapshot::leave_the_song_to_be_picked_up`]. What runs that queue is the game's per-frame sound, so both
/// belong after it: [`finish_the_restores_sound`] is called from the frame there, and a restore itself leaves
/// only what to do behind.
///
/// A static because a restore and the frame's sound are two points in one frame with the snapshot's own
/// lifetime in between — the chapters may have dropped it by then — and both are on the game's main thread.
static WAITING: std::sync::Mutex<Option<Waiting>> = std::sync::Mutex::new(None);

/// How many frames that note has been held for, and how many it may be: what is waited for is the game's own
/// sound command, and a game that never finishes one would otherwise be a note held over a song it started
/// itself seconds later.
///
/// The number is a bound rather than a measurement — what says the wait is over is the queue going empty — so it
/// is generous: a stage of th07 whose own song starts seconds into it is the case this exists for, and every
/// frame of the wait says so in the log.
static WAITED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
const FRAMES_A_COMMAND_MAY_TAKE: u32 = 30;

enum Waiting {
    /// The offset in the track's own file the chapter's sound was at, and which track that was.
    Song { identity: Option<u32>, offset: i32 },
    /// Or nothing audible, for a chapter that had a stream and no sound coming out of it. The ask that
    /// filled the player's stream field again is what there is to take back down — see
    /// `Game::restart_stage_music`.
    Silence,
}

/// Finishes what a restore left, the game's own sound having run since.
///
/// A song only where the track playing is the one that chapter's sound was in, since an offset means nothing
/// in another track: the game was asked for the stage's own song, and a chapter taken under any other is one
/// whose position is in a different part of the file.
///
/// # Safety
/// Must run on the game's main thread, between frames, with no thread suspended: both lock the sound buffer,
/// which the streaming thread can be inside DirectSound holding.
pub unsafe fn finish_the_restores_sound(game: &dyn crate::game::Game) {
    let mut held = WAITING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if held.is_none() {
        return;
    }
    // The ask a restore made is a command the game walks over several runs of its own sound, so what is left to
    // do about the sound waits here until that command has gone — see `Game::a_sound_command_is_in_flight`. The
    // wait is given up on rather than held for ever, a note kept past the frames a play takes being one that
    // would silence or seek a song the game started for its own reasons.
    if game.a_sound_command_is_in_flight() {
        let waited = WAITED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if waited < FRAMES_A_COMMAND_MAY_TAKE {
            return;
        }
        log!(
            "restore: the game's own sound is still not done after {waited} frame(s); giving up on it"
        );
    }
    WAITED.store(0, std::sync::atomic::Ordering::Relaxed);
    let Some(waiting) = held.take() else { return };
    drop(held);
    let (identity, offset) = match waiting {
        Waiting::Song { identity, offset } => (identity, Some(offset)),
        Waiting::Silence => (None, None),
    };
    if let Some(offset) = offset {
        if game.music_identity() != identity {
            return log!("restore: the song is in another track than the one that came back");
        }
        let Some(live) = game.music() else {
            return log!("restore: nothing is playing for the song to be put into");
        };
        log!(
            "restore: the song {} at {offset} in the track's own file",
            if unsafe { live.play_from(offset) } {
                "picked up"
            } else {
                "could not be put"
            },
        );
        return;
    }
    let Some(live) = game.music() else {
        return log!("restore: nothing came back for this chapter's silence to be taken off");
    };
    unsafe { live.silence() };
    log!("restore: this chapter had nothing audible; the track asked for is stopped again");
}

pub struct Snapshot {
    saved: Vec<Saved>,
    /// Private committed memory that is *not* saved, fingerprinted so
    /// `self_check` can say what changed without being restored.
    untracked: Vec<Fingerprint>,
    /// The BGM stream, whose state partly lives inside DirectSound and winmm.
    music: Option<(audio::Music, audio::Saved)>,
    /// The game's audio thread, left running while the game is held still.
    audio_thread: Option<u32>,
    /// Which track was playing. A restore that finds another one playing refuses,
    /// because what this snapshot holds of the sound has gone with it.
    identity: Option<u32>,
    /// Memory a restore leaves at its current value. Used to let the music play
    /// on: rewinding a stream whose sound buffer is not being rewound with it is
    /// what makes it loop on a few hundred milliseconds forever.
    preserve: Vec<Region>,
    /// Memory a restore leaves alone whatever else it does: handles to things outside the
    /// game's own memory, which cannot be copied into a snapshot and so must not be put
    /// back from one. Unlike `preserve` this holds even when the sound has been taken down —
    /// it is not about the sound.
    live: Vec<Region>,
}

struct Saved {
    region: Region,
    data: Buffer,
    /// Which allocation the bytes came out of, as the host answered when they were saved. What a
    /// restore holds it against is the answer for the same address then — see `Snapshot::restore`.
    allocation: Option<usize>,
}

struct Fingerprint {
    region: Region,
    hash: u64,
    in_process_heap: bool,
}

/// Backing store for a saved region.
///
/// An allocation of orb's own, with the host *told* about it rather than asked for it: `self_check`
/// finds memory the game changed outside a snapshot by fingerprinting every private page in the
/// process, and Rust's allocator shares the process heap with the libraries it has to tell apart from
/// the game — so the range is named as orb's and left out of that walk.
///
/// Pages from the host instead was tried and is a crash. A buffer taken from a simulated Windows and
/// given back once that Windows has gone is a `VirtualFree` of a heap pointer, which is what the suite
/// did while taking a chapter's snapshots down: the installation is scoped to a call and the chapter
/// outlives it.
///
/// Never grown after it is made, so the pointer the copy is written through does not move.
struct Buffer {
    bytes: Vec<u8>,
}

impl Buffer {
    fn new(len: usize) -> Option<Self> {
        let bytes = vec![0u8; len];
        mem::keep_out_of_private_regions(bytes.as_ptr() as usize, len);
        // Where orb's own copies are, for holding against what a chapter saves: a buffer inside a range a
        // restore writes is orb's own memory being put back to what the game had there.
        detail!(
            "memory: a buffer of orb's own at {:#010x}+{len:#x}",
            bytes.as_ptr() as usize,
        );
        Some(Self { bytes })
    }

    fn base(&self) -> *mut u8 {
        self.bytes.as_ptr() as *mut u8
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        crash::doing(crash::Doing::GivingABufferBack, 0, 0);
        mem::count_private_region_again(self.bytes.as_ptr() as usize);
    }
}

impl Drop for Snapshot {
    /// The address space this would have put back is nobody's to keep once there is no putting it back.
    /// A stage beginning drops every chapter of the one before it, and a run ending drops them all.
    fn drop(&mut self) {
        for entry in &self.saved {
            mem::a_restore_is_done_with(entry.region.base, entry.region.len);
        }
    }
}

#[derive(Default)]
pub struct SelfCheck {
    /// Saved regions whose contents differ from the snapshot after a restore:
    /// the restore itself did not take.
    pub unrestored: Vec<Region>,
    /// Regions that changed since the snapshot and are not restored, i.e. state
    /// the snapshot may be missing.
    pub changed_untracked: Vec<Region>,
    pub changed_in_process_heap: usize,
}

impl Snapshot {
    /// # Safety
    /// Must run on the game's main thread. `regions` must describe currently
    /// mapped memory; ranges that have gone away since are skipped.
    pub unsafe fn capture(
        regions: &[Region],
        audio: Audio,
        live: &[Range<usize>],
        with_inventory: bool,
    ) -> Self {
        let mut snapshot = Self {
            saved: Vec::new(),
            untracked: Vec::new(),
            music: None,
            audio_thread: None,
            identity: None,
            preserve: Vec::new(),
            live: Vec::new(),
        };
        unsafe { snapshot.update(regions, audio, live, with_inventory) };
        snapshot
    }

    /// Re-saves over the buffers this snapshot already owns. Chapter boundaries
    /// come around every few seconds, and allocating megabytes each time costs
    /// more than the copy itself: the pages have to be faulted in fresh.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub unsafe fn update(
        &mut self,
        regions: &[Region],
        audio: Audio,
        live: &[Range<usize>],
        with_inventory: bool,
    ) {
        self.audio_thread = audio.thread;
        self.live = live
            .iter()
            .map(|r| Region {
                base: r.start,
                len: r.len(),
            })
            .collect();
        let audio_state: Vec<Region> = audio
            .state
            .iter()
            .map(|range| Region {
                base: range.start,
                len: range.len(),
            })
            .collect();
        let (music, preserve) = match audio.policy {
            Music::Rewind(stream) => (stream, Vec::new()),
            Music::KeepPlaying => (None, audio_state),
        };
        self.identity = audio.identity;
        self.preserve = preserve;
        if !self.preserve.is_empty() {
            log!(
                "snapshot: leaving {} audio range(s) alone: {}",
                self.preserve.len(),
                self.preserve
                    .iter()
                    .map(|r| format!("{:#010x}+{:#x}", r.base, r.len))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        let reusable = self.saved.len() == regions.len()
            && self
                .saved
                .iter()
                .zip(regions)
                .all(|(saved, region)| saved.region == *region);
        if !reusable {
            // What the set it held named is given up and what the new one names is said instead: the
            // address space a restore of this snapshot needs is the ranges it holds *now*, and the host
            // keeps that space the moment the game hands it back — see `mem::hold_what_a_restore_needs`.
            for entry in &self.saved {
                mem::a_restore_is_done_with(entry.region.base, entry.region.len);
            }
            // Allocated before suspending: VirtualAlloc can block on a lock a
            // suspended thread would then never release.
            self.saved = regions
                .iter()
                .filter_map(|&region| {
                    Some(Saved {
                        region,
                        data: Buffer::new(region.len)?,
                        allocation: mem::allocation_at(region.base),
                    })
                })
                .collect();
            for entry in &self.saved {
                mem::a_restore_needs(entry.region.base, entry.region.len);
            }
        }
        // The sound buffer has to be captured with no thread suspended, because
        // locking it waits on the streaming thread — but that thread also moves
        // the bookkeeping the memory copy is about to record. So the two are
        // taken together and rejected if the stream ran in between, which is what
        // otherwise leaves a chapter's music very slightly wrong.
        //
        // **No e2e test reaches the retry**, and none can: a laid-out game has no streaming thread, and
        // what stands in for one is an e2e test saying the stream ran, between frames. So the loop below
        // goes round once there and the `continue` is the real game's alone.
        let mut saved_music = None;
        for _ in 0..AUDIO_ATTEMPTS {
            crash::doing(crash::Doing::CapturingTheSound, 0, 0);
            saved_music = music.and_then(|music| {
                let saved = unsafe { music.capture(audio.identity) }?;
                Some((music, saved))
            });
            {
                let _suspended = suspend(audio.thread);
                let regions = self.saved.len();
                for (at, entry) in self.saved.iter_mut().enumerate() {
                    // Which copy this is, for a fault inside one to be read by: what a crash in here turns
                    // on is which region and how far through, and the address alone says neither.
                    crash::doing(crash::Doing::CopyingARegion, at + 1, regions);
                    unsafe {
                        mem::copy_out(entry.region.base, entry.data.base(), entry.region.len)
                    };
                }
                crash::doing(crash::Doing::TakingASnapshot, 0, 0);
            }
            match &saved_music {
                Some((music, saved)) if !music.agrees_with_memory(saved) => continue,
                _ => break,
            }
        }
        if let Some((music, saved)) = &saved_music
            && !music.agrees_with_memory(saved)
        {
            log!("snapshot: the music stream would not hold still; not restoring it");
            saved_music = None;
        }

        self.untracked = if with_inventory {
            unsafe { fingerprint_untracked(regions) }
        } else {
            Vec::new()
        };
        self.music = saved_music;
    }

    pub fn has_music(&self) -> bool {
        self.music.is_some()
    }

    pub fn bytes(&self) -> usize {
        self.saved.iter().map(|entry| entry.region.len).sum()
    }

    pub fn regions(&self) -> usize {
        self.saved.len()
    }

    /// Whether the music this snapshot holds is the one playing, and so can be put
    /// back byte for byte.
    ///
    /// Once the game has changed track it has freed the stream and released its
    /// sound buffer, and no amount of memory copying brings a released COM object
    /// back. Neither is the memory restorable around the stream that replaced it:
    /// that one was allocated after the snapshot, so writing the snapshot back
    /// rolls its own object out from under the streaming thread, which is not
    /// suspended — measured as an access violation inside `DSOUND.dll` writing a
    /// buffer it no longer owned. Skipping the live ranges only moves it, since the
    /// heap's bookkeeping still rolls back to before that stream was allocated and
    /// the next track change frees a block the allocator has been told is free.
    ///
    /// So the sound is not restored in that case: it is torn down through the game
    /// first and started again after, which is what [`Snapshot::restore`] does.
    /// What a restore that touched none of the sound left it as: the pair the loop is taken on — the countdown
    /// and where the file is — with the buffer's own state beside them.
    ///
    /// **The pair is the point.** The streaming thread moves both together, and a restore that put one back
    /// without the other is a stream that loops over a stretch of itself: the countdown runs out where the file
    /// is not, so no loop of the *track* is taken and the buffer goes round its own contents. Which is why both
    /// are said here whether or not this arm changed anything — a chapter kept playing had its own memory held
    /// back, and what that leaves is the question.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn say_what_the_sound_was_left_as(&self, game: &dyn crate::game::Game) {
        let Some(live) = game.music() else {
            return detail!("sound: nothing is playing for this chapter to be put back into");
        };
        detail!(
            "sound: left playing — {} byte(s) left, next write at {:?}, track {:?}, \
             {} range(s) held back",
            mem::read_committed::<u32>(live.bytes_left).unwrap_or_default(),
            live.token(),
            game.music_identity(),
            self.preserve.len(),
        );
    }

    fn music_still_playing(&self, game: &dyn crate::game::Game) -> bool {
        let Some((music, saved)) = &self.music else {
            return true;
        };
        let Some(live) = game.music() else {
            return false;
        };
        music.still_current(saved, &live, game.music_identity())
    }

    /// # Safety
    /// Must run on the game's main thread, from a point where the game is
    /// between frames: mid-frame the game holds live pointers on the stack that
    /// the restored `.data` knows nothing about.
    pub unsafe fn restore(&self, game: &dyn crate::game::Game) {
        let same_track = self.music_still_playing(game);
        // Before the copy, while the game's memory is still its own: its allocator
        // has to see the stream being freed, and the streaming thread has to be
        // gone before its object is written over.
        if !same_track {
            log!("restore: the track has changed since this snapshot; taking the music down");
            unsafe { game.stop_music() };
        }
        // Nothing between the suspend and the resume may allocate: a suspended
        // thread can hold the allocator's lock, and would never give it back.
        let mut failed = Vec::with_capacity(self.saved.len());
        // Which is why the pages a saved region has lost since — the game freeing a few megabytes
        // does it — are committed here rather than inside the write loop, where `commit`'s
        // `VirtualAlloc` was exactly the call that rule forbids. It leaves a gap in which a page
        // committed here could go again before it is written: the same gap the region list already
        // lives with, `memtrack::regions` calling its own result a plan rather than a fact. Taken
        // deliberately, because a fault in the copy is a fault and a lock a stopped thread will
        // never give back is a hang with the game's threads frozen.
        let mut writable = Vec::with_capacity(self.saved.len());
        let mut taken = Vec::new();
        for entry in &self.saved {
            // **Whose the pages are, asked before anything is committed.** Free is the game's to have
            // back — that is what the commit below is for, and a range whose whole reservation went with
            // the block is free — but pages that are *another allocation* than the one these bytes came
            // out of belong to whoever was given them, and the game's old bytes over them is a restore
            // corrupting somebody. Which somebody is not asked: enumerating them is what this replaced,
            // and two crashes of th07 were two different answers to it.
            let now = mem::allocation_at(entry.region.base);
            // **Every page of it and not only the first**, which is what a range of this list can hide: a
            // heap region's committed runs and the reservations beside them are merged where they abut, so
            // one entry here can cover more than one allocation and the far end of it can have changed
            // hands while the base has not.
            let mut runs: Vec<(usize, usize, Option<usize>)> = Vec::new();
            let mut at = entry.region.base;
            while at < entry.region.end() {
                let allocation = mem::allocation_at(at);
                match runs.last_mut() {
                    Some(run) if run.2 == allocation => run.1 += PAGE,
                    _ => runs.push((at, PAGE, allocation)),
                }
                at += PAGE;
            }
            let strange: Vec<&(usize, usize, Option<usize>)> = runs
                .iter()
                .filter(|run| run.2.is_some() && run.2 != entry.allocation)
                .collect();
            detail!(
                "memory: {:#010x}+{:#x} came out of {:?}, is in {:?}, {} run(s){}",
                entry.region.base,
                entry.region.len,
                entry.allocation,
                now,
                runs.len(),
                strange
                    .iter()
                    .take(8)
                    .map(|run| format!(" — {:#010x}+{:#x} in {:?}", run.0, run.1, run.2))
                    .collect::<String>(),
            );
            if let Some(allocation) = now
                && Some(allocation) != entry.allocation
            {
                taken.push(entry.region);
                continue;
            }
            if unsafe { mem::commit(entry.region.base, entry.region.len) } {
                writable.push(entry);
            } else {
                failed.push(entry.region);
            }
        }
        if !taken.is_empty() {
            let covered: usize = taken.iter().map(|region| region.len).sum();
            log!(
                "restore: {} range(s) of this chapter are another allocation now, {covered} bytes; \
                 leaving them: {}",
                taken.len(),
                taken
                    .iter()
                    .map(|region| format!("{:#010x}+{:#x}", region.base, region.len))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        // Sorted here rather than while writing: the write loop runs with threads
        // suspended and must not allocate or do anything it can avoid.
        //
        // Nothing is held back when the sound has been taken down: there is no
        // longer anything live in those ranges to protect.
        // **The ranges the sound is in *now*, and not the ones this snapshot wrote down.** A chapter whose music
        // is left playing is one whose sound the restore must not touch — and which objects that is, is a
        // question about the sound the game is playing through at this moment, not the one it was playing
        // through when the chapter was taken. A track change deletes the stream and its source and makes both
        // again somewhere else, so the written-down addresses hold back memory nobody is playing through and
        // leave the live stream to be written over: its countdown, its source, the buffer it names.
        //
        // Measured on th07: retrying a chapter of stage 6's fight, whose second phase had replaced that
        // stream, left the song going round a stretch of itself and still playing at the title.
        //
        // What the snapshot's own list still decides is *whether* the sound is held back at all, which is the
        // chapter's own policy — see `Chapters::audio_for`.
        let mut holes = if same_track && !self.preserve.is_empty() {
            game.audio_state()
                .iter()
                .map(|range| Region {
                    base: range.start,
                    len: range.len(),
                })
                .collect()
        } else {
            Vec::new()
        };
        holes.extend_from_slice(&self.live);
        holes.sort_unstable_by_key(|hole| hole.base);
        if !holes.is_empty() {
            let covered: usize = holes.iter().map(|hole| hole.len).sum();
            log!(
                "restore: skipping {} range(s), {covered} bytes",
                holes.len()
            );
        }
        {
            let _suspended = suspend(self.audio_thread);
            let regions = writable.len();
            for (at, entry) in writable.into_iter().enumerate() {
                crash::doing(crash::Doing::PuttingARegionBack, at + 1, regions);
                unsafe { restore_region(entry, &holes) };
            }
        }
        crash::doing(crash::Doing::PuttingTheSoundBack, 0, 0);
        // After the memory copy, so the streaming bookkeeping this has to agree
        // with is already back in place.
        match &self.music {
            Some((music, saved)) if same_track => unsafe { music.restore(saved) },
            _ if !same_track => {
                unsafe { game.restart_stage_music() };
                self.leave_the_song_to_be_picked_up();
            }
            // **The music left playing, which is what a chapter of a boss's own fight is put back with** — see
            // `Chapters::audio_for`. Nothing is done to the sound and its own memory was held back, so what this
            // says is only what it was left as: a restore whose sound is wrong afterwards has nothing else to be
            // read from, which is what a launch of th07 retrying stage 6's second phase came back with —
            // the song looping over a section of itself, and still playing at the title afterwards.
            _ => unsafe { self.say_what_the_sound_was_left_as(game) },
        }
        for region in failed {
            log!(
                "restore: cannot write {:#010x}+{:#x}",
                region.base,
                region.len
            );
        }
    }

    /// Leaves the song to be put where this chapter had it, the track having just been asked for again.
    ///
    /// The stream that held that place has been freed with the track that was playing, so what is left
    /// of it is the offset in the track's own file — which is the one thing about a chapter's sound
    /// that means anything to another stream. The same seek a resume does for a chapter it lands in,
    /// for the same reason: a chapter reached by going back to it from a boss's own music is the same
    /// place in the stage's song as the chapter it was taken at.
    ///
    /// **Left rather than done, because the track is not playing yet.** The game was asked for it, and on
    /// the backend th07 makes that ask is a command in the sound player's own queue: what carries it out
    /// is the game's own per-frame sound, which orb's frame runs after its update. A seek before that is a
    /// seek of the stream the memory copy has just written back — the one the game had already freed — and
    /// a launch of th07 faulted inside the game's own sound code doing exactly that, at `0x0045d8d5`
    /// reading `4` with `E_INVALIDARG` in two registers. So what to do waits, and
    /// [`finish_the_restores_sound`] is called where the frame's sound has run.
    ///
    /// Only where the track the game has started is the one this chapter's sound was in, since an
    /// offset means nothing in another file: the game restarts the stage's own song, and a chapter
    /// snapshotted under any other is one whose position is in another wav — which is why the identity
    /// is kept beside the offset and checked there rather than here.
    ///
    /// **And nothing audible where the chapter had none**, which is a chapter that held a stream with no sound
    /// coming out of it — see [`audio::Saved::was_playing`]. The ask is made all the same, the player's own
    /// stream field being what it fills; what this leaves is that ask taken back off once it has been carried
    /// out, so a stage's first chapter comes back to the silence it was taken in and the stage's own script is
    /// what starts its song. A launch of th07 heard the other thing: the song from orb's ask, and then the
    /// same song from its beginning where that script reached its own play.
    fn leave_the_song_to_be_picked_up(&self) {
        let Some((music, saved)) = &self.music else {
            return;
        };
        let waiting = if saved.was_playing() {
            Waiting::Song {
                identity: self.identity,
                offset: match music.offset_of(saved) {
                    Some(offset) => offset,
                    None => return,
                },
            }
        } else {
            Waiting::Silence
        };
        WAITED.store(0, std::sync::atomic::Ordering::Relaxed);
        *WAITING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(waiting);
    }

    /// Compares memory against the snapshot without changing anything.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub unsafe fn check(&self) -> SelfCheck {
        let mut check = SelfCheck {
            unrestored: Vec::with_capacity(self.saved.len()),
            changed_untracked: Vec::with_capacity(self.untracked.len()),
            changed_in_process_heap: 0,
        };
        let _suspended = suspend(self.audio_thread);
        for entry in &self.saved {
            let live = unsafe { mem::read_bytes(entry.region.base, entry.region.len) };
            if live != entry.data.as_slice() {
                check.unrestored.push(entry.region);
            }
        }
        for entry in &self.untracked {
            if unsafe { hash(entry.region) } != Some(entry.hash) {
                if entry.in_process_heap {
                    check.changed_in_process_heap += 1;
                } else {
                    check.changed_untracked.push(entry.region);
                }
            }
        }
        check
    }
}

/// Writes one saved region back. Must not allocate or log: it runs with the game's other
/// threads suspended, which is why `commit` is the caller's to have done already.
///
/// `holes` are ranges to leave at their current values, sorted by address. They
/// are skipped rather than saved and put back, because the one thread allowed to
/// keep running is the one that owns them: writing anything there would be
/// writing a value it had already moved past.
unsafe fn restore_region(entry: &Saved, holes: &[Region]) {
    let region = entry.region;
    let previous = unsafe { mem::unprotect(region.base, region.len) };

    let mut cursor = region.base;
    for hole in holes {
        let start = hole.base.clamp(cursor, region.end());
        let end = hole.end().clamp(cursor, region.end());
        if start > cursor {
            unsafe { write_back(entry, cursor..start) };
        }
        cursor = cursor.max(end);
    }
    if cursor < region.end() {
        unsafe { write_back(entry, cursor..region.end()) };
    }

    if let Some(previous) = previous {
        unsafe { mem::reprotect(region.base, region.len, previous) };
    }
}

/// Copies one span of a saved region back over the live memory.
unsafe fn write_back(entry: &Saved, span: Range<usize>) {
    let offset = span.start - entry.region.base;
    unsafe { mem::copy_in(span.start, entry.data.base().add(offset), span.len()) };
}

/// Fingerprints every private committed range that `regions` does not cover, so
/// a later `check` can name game state the snapshot is missing.
unsafe fn fingerprint_untracked(regions: &[Region]) -> Vec<Fingerprint> {
    // Nothing in a test build. The walk is a question about the *process*, and what it relies on is
    // the one thing a test binary cannot give it: everything that allocates is held still. On a real
    // host this runs with the game's threads suspended, so a region the walk found is a region that
    // is still there to be read; in a test binary the harness's other threads are running, and a
    // region freed between the walk and the read is a fault inside the read. Measured — the suite
    // crashed here in parallel and passed with `--test-threads=1`.
    //
    // What `self_check` is for, naming game state a snapshot is missing, needs the real process to
    // mean anything anyway: under a laid-out simulated Windows the regions an e2e test did not hand
    // over are the ones it chose not to.
    if cfg!(test) {
        return Vec::new();
    }
    let process_heap = mem::process_heap_regions();
    mem::private_regions()
        .into_iter()
        .map(|(base, len)| Region { base, len })
        .filter(|region| !overlaps(region, regions))
        .filter_map(|region| {
            Some(Fingerprint {
                region,
                hash: unsafe { hash(region) }?,
                in_process_heap: process_heap
                    .iter()
                    .any(|(base, len)| region.base < base + len && *base < region.end()),
            })
        })
        .collect()
}

/// **Neither this nor [`hash`] is entered by an e2e test**, and the reason is the host rather than the
/// switch: `self_check` is on in `orb-e2e`'s `the_snapshot_held_against_itself`, and what
/// `mem::private_regions` answers a simulated Windows' walk with is nothing — see
/// `orb_sim::Sim::private_regions`, whose comment says why. So the iterator above runs over an empty list
/// and these two, which are what it would call per range, are never reached.
fn overlaps(region: &Region, others: &[Region]) -> bool {
    others
        .iter()
        .any(|other| region.base < other.end() && other.base < region.end())
}

/// FNV-1a over 8 bytes at a time. A byte at a time is a serial multiply chain,
/// and `self_check` fingerprints every private page in the process twice per
/// snapshot, which made it slow enough to look like a hang.
unsafe fn hash(region: Region) -> Option<u64> {
    let bytes = mem::read_committed_bytes(region.base, region.len)?;
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let (chunks, tail) = bytes.as_chunks::<8>();
    for chunk in chunks {
        hash = (hash ^ u64::from_le_bytes(*chunk)).wrapping_mul(0x100_0000_01b3);
    }
    for &byte in tail {
        hash = (hash ^ byte as u64).wrapping_mul(0x100_0000_01b3);
    }
    Some(hash)
}

/// The game's threads, stopped for as long as this lives.
///
/// Timed here rather than behind the seam, so the cost of holding the game still is visible next to
/// the cost of the copy it exists for.
fn suspend(audio: Option<u32>) -> thread::Suspended {
    let started = crate::profile::now();
    let suspended = thread::Suspended::all_but_audio(audio);
    unsafe { crate::profile::record(crate::profile::Phase::Threads, started) };
    suspended
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{Audio, Music, Snapshot};
    use crate::game::th06::Th06;
    use crate::memtrack::Region;
    use orb_api::Kind;
    use orb_sim::Sim;

    /// The game's static data, and a second region standing for a block of its heap.
    const DATA: usize = 0x0069_b000;
    const HEAP: usize = 0x03a0_0000;
    const LEN: usize = 0x1000;

    /// No sound at all, which is what a space with the game's audio structures left zeroed reads
    /// as: `music_still_playing` then finds nothing to have changed and the restore leaves the
    /// sound paths alone.
    fn silent() -> Audio {
        Audio {
            policy: Music::Rewind(None),
            identity: None,
            state: Vec::new(),
            thread: None,
        }
    }

    fn regions() -> Vec<Region> {
        vec![
            Region {
                base: DATA,
                len: LEN,
            },
            Region {
                base: HEAP,
                len: LEN,
            },
        ]
    }

    fn laid_out() -> Arc<Sim> {
        let sim = Arc::new(Sim::new());
        sim.space().map(DATA, LEN, Kind::Private);
        sim.space().map(HEAP, LEN, Kind::Private);
        sim
    }

    /// What a chapter is: every byte of what the game had, back the way it was. Not a summary of it
    /// and not the fields orb knows the names of — the allocator's own bookkeeping comes back too,
    /// which is what keeps the pointers in it pointing at what they pointed at.
    #[test]
    fn a_restore_puts_every_byte_back() {
        let sim = laid_out();
        let _installed = sim.enter();
        let space = sim.space();
        space.fill_bytes(DATA, 0xa1, LEN);
        space.fill_bytes(HEAP, 0xb2, LEN);

        let snapshot = unsafe { Snapshot::capture(&regions(), silent(), &[], false) };

        // A stage's worth of the game running: a value here, a value there, and a range of it
        // rewritten altogether.
        space.write::<u32>(DATA + 0x40, 0xdead_beef);
        space.fill_bytes(DATA + 0x800, 0x00, 0x100);
        space.write::<u32>(HEAP + 4, 1886);

        unsafe { snapshot.restore(&Th06) };

        assert!(space.read_bytes(DATA, LEN).iter().all(|byte| *byte == 0xa1));
        assert!(space.read_bytes(HEAP, LEN).iter().all(|byte| *byte == 0xb2));
        assert!(unsafe { snapshot.check() }.unrestored.is_empty());
    }

    /// Memory the game handed back to the OS between the snapshot and the restore of it. The game
    /// will reach every one of those addresses again through a restored pointer, so the pages have
    /// to exist again at the same addresses — which is what the commit before the copy is for.
    ///
    /// Freeing a few megabytes mid-stage is what does this, and the hole it leaves was a fault
    /// inside the copy's own memcpy.
    #[test]
    fn a_region_that_has_gone_is_put_back_before_the_copy() {
        let sim = laid_out();
        let _installed = sim.enter();
        let space = sim.space();
        space.fill_bytes(HEAP, 0xb2, LEN);

        let snapshot = unsafe { Snapshot::capture(&regions(), silent(), &[], false) };
        space.unmap(HEAP);

        unsafe { snapshot.restore(&Th06) };

        assert!(space.read_bytes(HEAP, LEN).iter().all(|byte| *byte == 0xb2));
    }

    /// Handles to things that are not the game's own memory — Direct3D's objects. A snapshot cannot
    /// copy what they name, so a restore must not put them back: one names something released long
    /// ago, and the game releasing it a second time faults inside itself.
    ///
    /// The hole is left at its current value while everything around it comes back, which is the
    /// part worth pinning: a restore that skipped the whole region instead would lose the chapter.
    #[test]
    fn a_live_handle_is_left_where_the_restore_finds_it() {
        let sim = laid_out();
        let _installed = sim.enter();
        let space = sim.space();
        space.fill_bytes(DATA, 0xa1, LEN);
        let handles = std::slice::from_ref(&(DATA + 0x100..DATA + 0x110));

        let snapshot = unsafe { Snapshot::capture(&regions(), silent(), handles, false) };

        space.fill_bytes(DATA, 0xc3, LEN);
        unsafe { snapshot.restore(&Th06) };

        // The handle is the device the game has now, not the one it had then.
        assert!(
            space
                .read_bytes(DATA + 0x100, 0x10)
                .iter()
                .all(|byte| *byte == 0xc3)
        );
        // And the bytes either side of it are the chapter's.
        assert!(
            space
                .read_bytes(DATA, 0x100)
                .iter()
                .all(|byte| *byte == 0xa1)
        );
        assert!(
            space
                .read_bytes(DATA + 0x110, 0x100)
                .iter()
                .all(|byte| *byte == 0xa1)
        );
    }

    /// Pages that are **another allocation** than the one they were saved from, which a restore must
    /// leave alone: the game handing a block back to the OS puts that address space up for anybody, and
    /// what a restore would write the game's old bytes over is then whatever was given it next.
    ///
    /// **A th07 session is why this is here, and it crashed three times.** A step back at a stage's
    /// end faulted inside the *next* snapshot's own copy — `code 0xc0000005 ... reading 0x00000018`, the
    /// stack naming `chapter::take` under `Chapters::begin_chapter`, and the faulting instruction the
    /// memcpy loop over `saved` whose slice pointer read as 8 with a count that did not: orb's own buffer
    /// had been given those pages and then written over. Skipping the ranges orb had *named* left the
    /// heap's own bookkeeping and everything nobody named, and the step back after that faulted inside
    /// `ntdll`'s own heap. Skipping what the process heap said it held left the headers those blocks are
    /// handed out with, and the step after *that* faulted in the heap again.
    ///
    /// Which is why this asks the one question that does not enumerate anybody: **is this the allocation
    /// the bytes came out of?** `mem::allocation_at` is the host's own answer, and a restore writes
    /// nothing it says no to.
    #[test]
    fn a_region_that_is_another_allocation_now_is_left_where_the_restore_finds_it() {
        let sim = laid_out();
        let _installed = sim.enter();
        let space = sim.space();
        space.fill_bytes(DATA, 0xa1, LEN);
        space.fill_bytes(HEAP, 0xb2, LEN);

        let snapshot = unsafe { Snapshot::capture(&regions(), silent(), &[], false) };

        // The game hands the block back and somebody else is given the same addresses: an allocation of
        // its own, holding its own bytes, which the chapter knows nothing about.
        space.unmap(HEAP);
        space.map(HEAP, LEN, Kind::Private);
        space.fill_bytes(HEAP, 0x7e, LEN);

        unsafe { snapshot.restore(&Th06) };

        assert!(
            space.read_bytes(HEAP, LEN).iter().all(|byte| *byte == 0x7e),
            "the restore wrote the chapter's bytes over an allocation that is not the one it saved",
        );
        // And what is still the allocation it was saved from comes back, which is the chapter.
        assert!(space.read_bytes(DATA, LEN).iter().all(|byte| *byte == 0xa1));
    }

    /// A snapshot taken over the same regions again writes into the buffers it already owns: a
    /// boundary comes around every few seconds, and several megabytes of fresh pages each time
    /// costs more than the copy does. What it must not do is keep the bytes from last time.
    #[test]
    fn a_snapshot_taken_again_holds_the_second_one() {
        let sim = laid_out();
        let _installed = sim.enter();
        let space = sim.space();
        space.fill_bytes(DATA, 0xa1, LEN);

        let mut snapshot = unsafe { Snapshot::capture(&regions(), silent(), &[], false) };
        space.fill_bytes(DATA, 0xb2, LEN);
        unsafe { snapshot.update(&regions(), silent(), &[], false) };

        space.fill_bytes(DATA, 0xc3, LEN);
        unsafe { snapshot.restore(&Th06) };

        assert!(space.read_bytes(DATA, LEN).iter().all(|byte| *byte == 0xb2));
    }
}
