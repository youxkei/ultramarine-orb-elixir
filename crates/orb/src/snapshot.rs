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

use std::ops::Range;

use crate::audio;
use crate::log;
use crate::memtrack::Region;
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
        mem::count_private_region_again(self.bytes.as_ptr() as usize);
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
            // Allocated before suspending: VirtualAlloc can block on a lock a
            // suspended thread would then never release.
            self.saved = regions
                .iter()
                .filter_map(|&region| {
                    Some(Saved {
                        region,
                        data: Buffer::new(region.len)?,
                    })
                })
                .collect();
        }
        // The sound buffer has to be captured with no thread suspended, because
        // locking it waits on the streaming thread — but that thread also moves
        // the bookkeeping the memory copy is about to record. So the two are
        // taken together and rejected if the stream ran in between, which is what
        // otherwise leaves a chapter's music very slightly wrong.
        let mut saved_music = None;
        for _ in 0..AUDIO_ATTEMPTS {
            saved_music = music.and_then(|music| {
                let saved = unsafe { music.capture(audio.identity) }?;
                Some((music, saved))
            });
            {
                let _suspended = suspend(audio.thread);
                for entry in &mut self.saved {
                    unsafe {
                        mem::copy_out(entry.region.base, entry.data.base(), entry.region.len)
                    };
                }
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
        for entry in &self.saved {
            if unsafe { mem::commit(entry.region.base, entry.region.len) } {
                writable.push(entry);
            } else {
                failed.push(entry.region);
            }
        }
        // Sorted here rather than while writing: the write loop runs with threads
        // suspended and must not allocate or do anything it can avoid.
        //
        // Nothing is held back when the sound has been taken down: there is no
        // longer anything live in those ranges to protect.
        let mut holes = if same_track {
            self.preserve.clone()
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
            for entry in writable {
                unsafe { restore_region(entry, &holes) };
            }
        }
        // After the memory copy, so the streaming bookkeeping this has to agree
        // with is already back in place.
        match &self.music {
            Some((music, saved)) if same_track => unsafe { music.restore(saved) },
            _ if !same_track => {
                unsafe { game.restart_stage_music() };
            }
            _ => {}
        }
        for region in failed {
            log!(
                "restore: cannot write {:#010x}+{:#x}",
                region.base,
                region.len
            );
        }
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
    // mean anything anyway: under a laid-out simulated Windows the regions a scenario did not hand
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
