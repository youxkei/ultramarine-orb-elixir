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

use std::ffi::c_void;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Memory::{
    GetProcessHeap, HeapLock, HeapUnlock, HeapWalk, MEM_COMMIT, MEM_PRIVATE, MEM_RELEASE,
    MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
    PROCESS_HEAP_ENTRY, VirtualAlloc, VirtualFree, VirtualProtect, VirtualQuery,
};
use windows_sys::Win32::System::SystemServices::PROCESS_HEAP_REGION;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

use std::ops::Range;

use crate::audio;
use crate::log::log;
use crate::memtrack::Region;
use crate::threads;

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

/// Ranges `orb` itself allocated, so `self_check` does not report its own
/// buffers as game memory that changed behind our back.
static OURS: Mutex<Vec<Region>> = Mutex::new(Vec::new());

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

/// Backing store for a saved region, taken straight from the OS: the game's own
/// heap is what is being copied, and Rust's allocator shares a heap with the
/// libraries `self_check` has to tell apart from the game.
struct Buffer {
    base: *mut u8,
    len: usize,
}

impl Buffer {
    fn new(len: usize) -> Option<Self> {
        let base = unsafe {
            VirtualAlloc(std::ptr::null(), len, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
        };
        if base.is_null() {
            return None;
        }
        if let Ok(mut ours) = OURS.lock() {
            ours.push(Region { base: base as usize, len });
        }
        Some(Self { base: base.cast(), len })
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base, self.len) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if let Ok(mut ours) = OURS.lock() {
            ours.retain(|region| region.base != self.base as usize);
        }
        unsafe { VirtualFree(self.base.cast(), 0, MEM_RELEASE) };
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
        self.live = live.iter().map(|r| Region { base: r.start, len: r.len() }).collect();
        let audio_state: Vec<Region> = audio
            .state
            .iter()
            .map(|range| Region { base: range.start, len: range.len() })
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
            && self.saved.iter().zip(regions).all(|(saved, region)| saved.region == *region);
        if !reusable {
            // Allocated before suspending: VirtualAlloc can block on a lock a
            // suspended thread would then never release.
            self.saved = regions
                .iter()
                .filter_map(|&region| Some(Saved { region, data: Buffer::new(region.len)? }))
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
                        std::ptr::copy_nonoverlapping(
                            entry.region.base as *const u8,
                            entry.data.base,
                            entry.region.len,
                        )
                    };
                }
            }
            match &saved_music {
                Some((music, saved)) if !music.agrees_with_memory(saved) => continue,
                _ => break,
            }
        }
        if let Some((music, saved)) = &saved_music {
            if !music.agrees_with_memory(saved) {
                log!("snapshot: the music stream would not hold still; not restoring it");
                saved_music = None;
            }
        }

        self.untracked =
            if with_inventory { unsafe { fingerprint_untracked(regions) } } else { Vec::new() };
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
        let Some((music, saved)) = &self.music else { return true };
        let Some(live) = game.music() else { return false };
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
        // Sorted here rather than while writing: the write loop runs with threads
        // suspended and must not allocate or do anything it can avoid.
        //
        // Nothing is held back when the sound has been taken down: there is no
        // longer anything live in those ranges to protect.
        let mut holes = if same_track { self.preserve.clone() } else { Vec::new() };
        holes.extend_from_slice(&self.live);
        holes.sort_unstable_by_key(|hole| hole.base);
        if !holes.is_empty() {
            let covered: usize = holes.iter().map(|hole| hole.len).sum();
            log!("restore: skipping {} range(s), {covered} bytes", holes.len());
        }
        {
            let _suspended = suspend(self.audio_thread);
            for entry in &self.saved {
                if !unsafe { restore_region(entry, &holes) } {
                    failed.push(entry.region);
                }
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
            log!("restore: cannot write {:#010x}+{:#x}", region.base, region.len);
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
            let live = unsafe {
                std::slice::from_raw_parts(entry.region.base as *const u8, entry.region.len)
            };
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

/// Returns false if the region could not be written back. Must not allocate or
/// log: it runs with the game's other threads suspended.
///
/// `holes` are ranges to leave at their current values, sorted by address. They
/// are skipped rather than saved and put back, because the one thread allowed to
/// keep running is the one that owns them: writing anything there would be
/// writing a value it had already moved past.
unsafe fn restore_region(entry: &Saved, holes: &[Region]) -> bool {
    let region = entry.region;
    if !unsafe { commit(region) } {
        return false;
    }

    let mut previous: PAGE_PROTECTION_FLAGS = 0;
    let unprotected = unsafe {
        VirtualProtect(region.base as *const c_void, region.len, PAGE_READWRITE, &mut previous)
    } != FALSE;

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

    if unprotected {
        unsafe {
            VirtualProtect(region.base as *const c_void, region.len, previous, &mut previous);
        }
    }
    true
}

/// Puts back any page of `region` that has gone since the snapshot, so that the copy
/// has somewhere to write. The game will reach every one of them again through a
/// restored pointer, so they have to exist at the same addresses.
///
/// Walked run by run rather than tested at the region's first page. `VirtualQuery`
/// describes only the run of pages that begins where it is asked, so a region whose
/// head is still committed answers "committed" while a hole further in has been handed
/// back to the OS — which the game freeing a few megabytes does. That hole was a fault
/// inside the copy's own `memcpy`, and `VirtualProtect` over the whole region had
/// quietly failed for the same reason.
unsafe fn commit(region: Region) -> bool {
    let mut at = region.base;
    while at < region.end() {
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let queried = unsafe {
            VirtualQuery(at as *const c_void, &mut info, size_of::<MEMORY_BASIC_INFORMATION>())
        };
        if queried == 0 {
            return false;
        }
        let run = (info.BaseAddress as usize + info.RegionSize).min(region.end());
        if run <= at {
            return false;
        }
        if info.State != MEM_COMMIT {
            let len = run - at;
            let committed =
                unsafe { VirtualAlloc(at as *const c_void, len, MEM_COMMIT, PAGE_READWRITE) };
            if committed.is_null() {
                let reserved = unsafe {
                    VirtualAlloc(
                        at as *const c_void,
                        len,
                        MEM_COMMIT | MEM_RESERVE,
                        PAGE_READWRITE,
                    )
                };
                if reserved.is_null() {
                    return false;
                }
            }
        }
        at = run;
    }
    true
}

/// Copies one span of a saved region back over the live memory.
unsafe fn write_back(entry: &Saved, span: Range<usize>) {
    let offset = span.start - entry.region.base;
    unsafe {
        std::ptr::copy_nonoverlapping(
            entry.data.base.add(offset),
            span.start as *mut u8,
            span.len(),
        )
    };
}

/// Fingerprints every private committed range that `regions` does not cover, so
/// a later `check` can name game state the snapshot is missing.
unsafe fn fingerprint_untracked(regions: &[Region]) -> Vec<Fingerprint> {
    let ours = OURS.lock().map(|ours| ours.clone()).unwrap_or_default();
    let process_heap = unsafe { process_heap_regions() };

    let mut fingerprints = Vec::new();
    let mut address = 0usize;
    loop {
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let queried = unsafe {
            VirtualQuery(address as *const c_void, &mut info, size_of::<MEMORY_BASIC_INFORMATION>())
        };
        if queried == 0 {
            break;
        }
        let region = Region { base: info.BaseAddress as usize, len: info.RegionSize };
        let next = region.end();

        let interesting = info.State == MEM_COMMIT
            && info.Type == MEM_PRIVATE
            && crate::memtrack::is_readable(info.Protect)
            && !overlaps(&region, regions)
            && !overlaps(&region, &ours);
        if interesting {
            if let Some(hash) = unsafe { hash(region) } {
                fingerprints.push(Fingerprint {
                    region,
                    hash,
                    in_process_heap: overlaps(&region, &process_heap),
                });
            }
        }

        if next <= address {
            break;
        }
        address = next;
    }
    fingerprints
}

/// Rust's allocator and the DirectX runtimes share the process heap, so
/// `self_check` counts changes there instead of listing them.
unsafe fn process_heap_regions() -> Vec<Region> {
    let heap = unsafe { GetProcessHeap() };
    let mut regions = Vec::new();
    if heap.is_null() || unsafe { HeapLock(heap) } == 0 {
        return regions;
    }
    let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
    while unsafe { HeapWalk(heap, &mut entry) } != 0 {
        if entry.wFlags & PROCESS_HEAP_REGION as u16 == 0 {
            continue;
        }
        let region = unsafe { entry.Anonymous.Region };
        regions.push(Region {
            base: entry.lpData as usize,
            len: region.dwCommittedSize as usize + region.dwUnCommittedSize as usize,
        });
    }
    unsafe { HeapUnlock(heap) };
    regions
}

fn overlaps(region: &Region, others: &[Region]) -> bool {
    others.iter().any(|other| region.base < other.end() && other.base < region.end())
}

/// FNV-1a over 8 bytes at a time. A byte at a time is a serial multiply chain,
/// and `self_check` fingerprints every private page in the process twice per
/// snapshot, which made it slow enough to look like a hang.
unsafe fn hash(region: Region) -> Option<u64> {
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let queried = unsafe {
        VirtualQuery(region.base as *const c_void, &mut info, size_of::<MEMORY_BASIC_INFORMATION>())
    };
    if queried == 0 || info.State != MEM_COMMIT || info.RegionSize < region.len {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(region.base as *const u8, region.len) };
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
/// Timed here rather than in `threads`, so the cost of holding the game still is
/// visible next to the cost of the copy it exists for.
fn suspend(audio: Option<u32>) -> threads::Suspended {
    let started = crate::profile::now();
    let suspended = threads::Suspended::all_but(unsafe { GetCurrentThreadId() }, audio);
    unsafe { crate::profile::record(crate::profile::Phase::Threads, started) };
    suspended
}
