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
//!
//! A retry restores a whole memory snapshot rather than asking the game to jump anywhere. Nothing has to
//! know what a boss script was in the middle of.
//!
//! A snapshot covers:
//!
//! - the exe's `.data`;
//! - the regions the game's CRT took from the OS, found by hooking `HeapCreate`, `HeapAlloc`,
//!   `HeapReAlloc`, `HeapFree` and `VirtualAlloc` in the exe's import table and walking the heaps;
//! - the music's stream position.
//!
//! Direct3D and DirectSound objects are not saved. The game loads a stage's resources on entry and creates
//! nothing within a chapter, so restoring the pointers in `.data` leaves them naming the same live objects.
//! Since those addresses differ per launch, a snapshot is only valid inside the process that took it, so
//! nothing of one is written to disk. What is written down instead is what the run *pressed*, and a chapter
//! is reached again by being played to — see [`crate::resume`].
//!
//! Restoring suspends the game's other threads, puts back any page of a saved region that has gone since —
//! the game freeing a few megabytes hands part of a segment back to the OS, and the copy has to have
//! somewhere to write — writes each region back, and resumes. The audio thread is left running; the music
//! is handled separately, rewound for the midstage and the midboss and left playing through a boss-fight
//! restore — see [`crate::audio`].
//!
//! **The name of the spell card being fought is put back by asking the game to write it again.** It is not
//! memory: the declaration hands the name to `AnmManager::DrawStringFormat` through `Gui::SetSpellcardName`
//! (0x417bfd, called at 0x409636), which bakes it into a vm's own sprite and keeps no copy of the string —
//! so a chapter put back where a *later* card has been declared since is fought under that later card's
//! name: going back from a boss's spell card to the midboss's, the plate said the boss's until the next
//! card started. So a restore bakes it again through the same call, out of the copy the game itself made in
//! that card's record — the one copy of the name that is memory, and one a restore already carries across
//! for the counts beside it. Nothing where no card is up, and nothing for a record the game has not named,
//! whose bytes are the generator's.
//!
//! Sound effects cut off on a restore rather than rewinding. Only the music is restored.
//!
//! **`--self-check`** restores every snapshot immediately after taking it and compares the result, and
//! reports memory that changed since a snapshot without being covered by it. The one moment a restore can
//! be held against what it should have produced is the instant it was taken from, which is why it happens
//! there. It fingerprints every private page in the process, so it pauses the game for as long as that
//! takes. **`--stress=N`** restores the current chapter every N frames, a few times per chapter and then
//! moving on, so a replay walks through the midstage, the midboss and every boss attack restoring as it
//! goes.

use std::ops::Range;

use crate::audio;
use crate::memtrack::{Owned, Region};
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

/// **How many of a snapshot's regions a restore writes back**, and nought for all of them, which is what a
/// launch nobody is bisecting anything with leaves it as.
///
/// **What it is for is finding which region's contents the next update cannot survive.** th06nc dies on the
/// frame a chapter of it is put back — not inside the write loop, every region of it being written and the
/// lines after them reached, but on the first update over the restored memory — and the thing no launch can
/// say is which of the eight the game then read something impossible out of. A restore held to the first N
/// of them answers that in as many launches as a bisection of eight takes, where the write loop's own line
/// answers only the order.
///
/// **A launch under it is a launch whose game is deliberately half restored**, so it is `orb_config::args`'
/// fault group's and nothing a run anybody plays should carry.
static PUT_BACK_ONLY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// **And how many of them it leaves alone**, from the front, which is the same bisect from the other end.
///
/// **Both halves exist because the regions are not independent.** `.data` is the first of them and it holds
/// pointers into the heap blocks that follow — the calc chain's head is a static and its elements are
/// `0x40`-byte blocks — so a prefix that puts `.data` back without the heap is a game whose chain names
/// blocks that have moved, and it dies of the flag rather than of the fault. Skipping the first region
/// instead puts the heap back and leaves `.data` as the run had it, which is the turn that says which of
/// the two the fault is in.
static PUT_BACK_SKIPPING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Holds every restore of this launch to the first `regions` of what it saved, or to all of them for
/// nought. Set once, off the launch's own options.
pub fn put_back_only(regions: usize) {
    PUT_BACK_ONLY.store(regions, std::sync::atomic::Ordering::Relaxed);
}

/// Leaves the first `regions` of what a restore saved alone, writing back the rest. Nought writes them all.
pub fn put_back_skipping(regions: usize) {
    PUT_BACK_SKIPPING.store(regions, std::sync::atomic::Ordering::Relaxed);
}

/// **Whether a restore holds back none of the ranges it would**, which is the third of the bisects and the
/// one the other two point at.
///
/// A restore ordinarily leaves a *mixture* behind: every region written back as the snapshot had it except
/// the ranges of [`Snapshot::live`] and the audio's, which stay as the run has them — eleven of them and
/// 195,276 bytes over th06nc. And that game dies of `.data` coming back and lives when `.data` is left
/// alone altogether, so whether the mixture is what its next update cannot survive is the question between
/// the two, and clearing the holes is what asks it. **A restore under this puts back a handle the game has
/// since replaced**, which is what the holes exist to stop — so it is a fault flag and nothing else.
static HOLD_BACK_NOTHING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Has every restore of this launch write back even the ranges it would hold back.
pub fn hold_back_nothing(on: bool) {
    HOLD_BACK_NOTHING.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// **Ranges inside a snapshot's memory a restore is told to hold back as well**, as
/// `(source, offset, len)` with the source named as `orb_api::mem::Source` prints it — `data`,
/// `heap1.3`, `reserve2` — which is how a bisection inside the game's memory is expressed.
///
/// **Offsets and not addresses, because the exe is built with ASLR.** A static of th06nc is written down
/// as `0x140aee0b0` and lives wherever the loader put the image, so an address on a command line names
/// nothing; `.data` is one source, and an offset into it is the same span whatever base a launch came up
/// at. `.data` begins at `0x14036d000`, so the enemy manager is `0x140aee0b0 - 0x14036d000 = 0x7810b0`
/// in.
///
/// **And a source rather than only `.data`, because the fault moved out of it.** th06nc's chapter retries
/// died of one byte of `.data`, found by halving it with this; what its stage 2 dies of is somewhere in
/// the game's heap, whose blocks are at a different address every launch.
///
/// **And a source rather than a place in the region list**, which is what this named first and had to
/// give up: the committed runs of one heap region are as many entries in that list as it has holes, and
/// one launch of th06nc listed the game's heap as 8 entries with `0xf000` in one chapter and 9 with
/// `0x6000` in another — so `2:0+7800` named different pages within the one run it was meant to bisect.
/// A heap's region as `HeapWalk` hands it over is the same segment in the next launch; where it sits and
/// how much of it is committed are the only parts anything else chooses.
///
/// **The offset is from the source's lowest base**, and the span is written back to whichever of that
/// source's regions it falls in — none of them, where the source has grown no further than the offset.
/// For a heap region that base is the region's own, which is what makes the offsets of a bisect mean the
/// same bytes twice: the distances *between* a heap's regions are the OS's, and one launch of th06nc put
/// the segment another had `0x1a670000` above the first `0x180000` below it.
static HOLD_BACK_IN_A_SOURCE: std::sync::Mutex<Vec<(String, usize, usize)>> =
    std::sync::Mutex::new(Vec::new());

/// Tells every restore of this launch to hold back these `(source, offset, len)` spans, the source named
/// as `orb_api::mem::Source` prints it.
pub fn hold_back_in_a_source(spans: &[(String, usize, usize)]) {
    let mut held = HOLD_BACK_IN_A_SOURCE.lock().unwrap();
    held.clear();
    held.extend_from_slice(spans);
}

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
    /// Which of the things the game took these bytes came out of, as the walk named it. What it is for is
    /// [`HOLD_BACK_IN_A_SOURCE`]: a launch bisecting a restore has to be able to name a span of the
    /// game's heap, and this region's place in the list is not that name.
    from: orb_api::mem::Source,
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
        regions: &[Owned],
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
        regions: &[Owned],
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
            // A stream that was read is a stream to hand back, and the memory it lives in is put back with
            // it: what the two say about each other has to agree, and both are this chapter's.
            Music::Rewind(Some(stream)) => (Some(stream), Vec::new()),
            // **And a chapter that read no stream leaves that memory where it stands.** What it holds of the
            // sound is not a description of the sound now playing: the case it most often is is a snapshot
            // taken while the game's own sound had an ask outstanding, where the player's stream field names
            // the object the take-down before that ask deleted — see `Chapters::audio_for`. Bytes naming a
            // released COM object are not bytes to put into a live player, and a restore that put them back
            // is what took th07 down on the real game: `0xc0000005` reading null at `th07.exe+0x5dc04` with
            // `ecx` zero, on the game's own sound thread, one frame after a chapter retry — 40 retries into a
            // chapter a resume had landed in, every one of them restoring over the live sound.
            //
            // The chapter itself is kept either way, its own frame being where a retry has to land. What is
            // left out of it is the sound, and the music then comes from the ask rather than from the bytes.
            Music::Rewind(None) | Music::KeepPlaying => (None, audio_state),
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
                .all(|(saved, owned)| saved.region == owned.region && saved.from == owned.from);
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
                .filter_map(|&Owned { region, from }| {
                    Some(Saved {
                        region,
                        from,
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

        // **Under a suspend of its own**, which is what [`fingerprint_untracked`] has always said it runs
        // under and did not: the guard above is scoped to the copy loop and dropped before this, so the
        // walk went round a process whose every thread was running. What that costs is the fault its own
        // doc names — a region found by the walk and freed before the read of it — and a launch of th06nc
        // with `--self-check` was killed by it at the first chapter of stage 1, the log ending after
        // `memory: a buffer of orb's own at …` for all nine regions with no chapter line after them.
        self.untracked = if with_inventory {
            let _suspended = suspend(audio.thread);
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

    /// **Every region it covers, as `base+len` apiece**, for a log line two launches can be held against
    /// each other by.
    ///
    /// Written because the count and the total are not enough to tell two snapshots apart: th06nc dies on
    /// the frame a chapter of its *stage 1* is put back and lives through the same restore in its Extra,
    /// and the one number that differs between them is this shape — `7 region(s) of 12539488 bytes`
    /// against `11 region(s) of 12449376`, the `.data` region being the same size in both. Which regions
    /// those are is what says what the two runs disagree about, and no line said it.
    pub fn covers(&self) -> String {
        self.saved
            .iter()
            .map(|entry| format!("{:#x}+{:#x}", entry.region.base, entry.region.len))
            .collect::<Vec<_>>()
            .join(" ")
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
            "sound: left playing — {}, track {:?}, {} range(s) held back",
            unsafe { live.how_it_stands() },
            game.music_identity(),
            self.preserve.len(),
        );
        // **And nothing done to it, which a repair that stood here for one launch is the reason to say out
        // loud.** It rebuilt the stream wherever the game's own refill would have refused the next write it
        // held — read off th07's 0x45db60 as the geometry a copy's own suspension leaves behind — and what the
        // frames after it then measured is that the refill *refuses in the ordinary way of things*: the thread
        // wakes at every notify point and writes only where it is safe to, so its steady state is the next
        // write exactly one chunk behind the device's write cursor, which is a refusal and is not a fault.
        //
        // A run of that repair rebuilt the stream on every retry, each from an offset 2.7 seconds further on
        // than the last. Whatever is wrong with the sound after a chapter of a resumed run comes back, this is
        // not it — and a repair that fires where nothing is broken is worse than the fault it was aimed at.
        // What the frames after a restore say instead is [`audio::read_the_stream_back`]'s.
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
        //
        // **And not whether the track is still the one it was taken under, which is a question this could not
        // answer.** A snapshot carrying ranges to hold back is one that captured no stream — both of the
        // policies that fill them leave `self.music` empty — and `music_still_playing` answers true for exactly
        // that, so asking it here decided nothing while reading as though it decided this.
        let mut holes = if !self.preserve.is_empty() {
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
        // **Or none of them, where a launch is asking what the holes cost** — see [`HOLD_BACK_NOTHING`].
        // A restore leaves a mixture behind: most of `.data` as the snapshot had it and these ranges as
        // the run has them, and whether *that* is what the next update cannot survive is a question one
        // launch answers where a bisection of nine megabytes wants twenty.
        if HOLD_BACK_NOTHING.load(std::sync::atomic::Ordering::Relaxed) {
            log!(
                "restore: holding back none of the {} range(s) this launch would have",
                holes.len(),
            );
            holes.clear();
        }
        // **And the spans a launch bisecting the game's memory named** — see [`HOLD_BACK_IN_A_SOURCE`],
        // empty for every launch that named none. Held against the regions of the source it names and
        // not against the whole span from its base: what lies between two of a heap's regions is nobody's,
        // and a restore never wrote it, so a hole over it would hold back nothing while reading as though
        // it held back something.
        for (which, offset, len) in HOLD_BACK_IN_A_SOURCE.lock().unwrap().iter() {
            let named: Vec<&Saved> = self
                .saved
                .iter()
                .filter(|entry| entry.from.to_string() == *which)
                .collect();
            let Some(base) = named.iter().map(|entry| entry.region.base).min() else {
                log!(
                    "restore: this snapshot saved nothing of {which}, so there is no {offset:#x} in it \
                     to hold back",
                );
                continue;
            };
            let wanted = base + offset..base + offset + len;
            let mut held = 0;
            let mut changed = Vec::new();
            for entry in &named {
                let start = wanted.start.max(entry.region.base);
                let end = wanted.end.min(entry.region.end());
                if start < end {
                    held += end - start;
                    unsafe { what_the_run_changed(entry, start..end, base, &mut changed) };
                    holes.push(Region {
                        base: start,
                        len: end - start,
                    });
                }
            }
            log!(
                "restore: holding back {held:#x} bytes of the {:#x} asked for at {offset:#x} into \
                 {which}, which begins at {base:#010x} and is {} region(s) here",
                len,
                named.len(),
            );
            unsafe { self.say_what_points_into(which, &named) };
            // **And which bytes of it the run has changed since the chapter**, which is what a bisect of
            // a span is after rather than the span: a hold-back that turns a death into a survival says
            // the fault is among the bytes the restore would have *written*, and a byte the run has not
            // touched is written with what is already there. So this is the span's own shortlist, and
            // the halvings that would find it one at a time cost a launch each.
            if !changed.is_empty() {
                let bytes: usize = changed.iter().map(|(_, was, _)| was.len()).sum();
                let first = changed.first().map_or(0, |(at, _, _)| *at);
                let last = changed
                    .last()
                    .map_or(0, |(at, was, _)| at + was.len().max(1) - 1);
                // **The source and the extent in front of the runs**, both because of what a bisect does
                // with the line: a launch names more than one span, so a line that began with the runs
                // said nothing about which span's they were; and the runs are cut at
                // [`WHAT_CHANGED_AT_MOST`], so where the next cut goes is a question about the *last* of
                // them as much as the first.
                detail!(
                    "restore: of {which} the run changed {bytes:#x} byte(s) in {} run(s) between \
                     {first:#x} and {last:#x}: {}",
                    changed.len(),
                    changed
                        .iter()
                        .take(WHAT_CHANGED_AT_MOST)
                        .map(|(at, was, now)| format!(
                            "{at:#x}+{:#x} {} -> {}",
                            was.len(),
                            in_hex(was),
                            in_hex(now),
                        ))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        holes.sort_unstable_by_key(|hole| hole.base);
        if !holes.is_empty() {
            let covered: usize = holes.iter().map(|hole| hole.len).sum();
            // **Named and not only counted**, the way the snapshot names the ones it left alone: a count says
            // that something was held back and not *what*, and which of them a restore skipped is the
            // question a session about the sound going wrong asks first. Read off the real game as
            // `skipping 1 range(s), 1056 bytes` where four were wanted, it took a second launch to learn
            // which three were missing.
            log!(
                "restore: skipping {} range(s), {covered} bytes: {}",
                holes.len(),
                holes
                    .iter()
                    .map(|hole| format!("{:#010x}+{:#x}", hole.base, hole.len))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        {
            // **Held to some of them where a launch is bisecting them** — see [`PUT_BACK_ONLY`] and
            // [`PUT_BACK_SKIPPING`], both nought for every launch that is not.
            match PUT_BACK_ONLY.load(std::sync::atomic::Ordering::Relaxed) {
                0 => {}
                only if only < writable.len() => {
                    log!(
                        "restore: putting back {only} of this snapshot's {} region(s) and no more",
                        writable.len(),
                    );
                    writable.truncate(only);
                }
                _ => {}
            }
            match PUT_BACK_SKIPPING.load(std::sync::atomic::Ordering::Relaxed) {
                0 => {}
                skipping if skipping < writable.len() => {
                    log!(
                        "restore: leaving the first {skipping} of this snapshot's {} region(s) alone",
                        writable.len(),
                    );
                    writable.drain(..skipping);
                }
                // Every region skipped is a restore that writes nothing, which is `--no-chapters` said
                // another way and not what anybody means by it.
                skipping => log!(
                    "restore: {skipping} region(s) would be skipped of {}; writing them all instead",
                    writable.len(),
                ),
            }
            let _suspended = suspend(self.audio_thread);
            let regions = writable.len();
            for (at, entry) in writable.into_iter().enumerate() {
                crash::doing(crash::Doing::PuttingARegionBack, at + 1, regions);
                // **Said before the write and not after it**, which is the whole use of it: the marker
                // above is read by `orb::crash`'s filter, and a death that goes through neither that
                // filter nor a Rust panic loses it — which is how th06nc dies, on the frame a chapter of
                // its stage 1 whose music is kept is put back. A line here leaves the last region the
                // process was writing in the log, so one launch names it where bisecting the nine would
                // want nine.
                detail!(
                    "restore: putting back {} of {regions}: {:#x}+{:#x}",
                    at + 1,
                    entry.region.base,
                    entry.region.len,
                );
                unsafe { restore_region(entry, &holes) };
            }
        }
        crash::doing(crash::Doing::PuttingTheSoundBack, 0, 0);
        // **And the frames after it watched, whichever arm below runs.** What a restore leaves the sound as is
        // read on the frame it happens; whether the game's own thread goes on *feeding* it is a question only
        // the frames after it answer, and only on the real game — see [`audio::watch_what_becomes_of_the_stream`].
        audio::watch_what_becomes_of_the_stream();
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

    /// **Which statics of the exe hold a pointer into a source**, said once a launch per source.
    ///
    /// **What it is for is the one thing a bisect cannot answer.** Halving a span says which bytes a
    /// restore must not write and never what they are, where what orb needs in order to hold a block back
    /// for good is the *pointer the game keeps it in* — [`crate::game::Game::live_handles`] is a list of
    /// exactly those, each a static read at a chapter. So the chapter's own `.data` is scanned for words
    /// that land inside the source's regions, and what comes back is `.data + offset`, which `asm.md` and
    /// a disassembly can put a name to.
    ///
    /// **The chapter's copy and not the live memory**, since the two hold the same pointer and this one is
    /// a buffer of orb's own: a nine-megabyte walk of the game's statics is a walk this can do without.
    ///
    /// **Once a launch**, because the answer does not change and the walk is nine megabytes.
    ///
    /// # Safety
    /// Must run on the game's main thread, with no thread suspended.
    unsafe fn say_what_points_into(&self, which: &str, named: &[&Saved]) {
        static ALREADY: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let mut already = ALREADY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if already.iter().any(|said| said == which) {
            return;
        }
        already.push(which.to_owned());
        drop(already);
        let Some(data) = self
            .saved
            .iter()
            .find(|entry| entry.from == orb_api::mem::Source::Data)
        else {
            return log!("restore: this snapshot saved no .data to look for {which} in");
        };
        let mut found = Vec::new();
        let word = size_of::<usize>();
        for at in (0..data.region.len.saturating_sub(word)).step_by(word) {
            let value = usize::from_ne_bytes(
                data.data.bytes[at..at + word]
                    .try_into()
                    .expect("a word of the copy"),
            );
            if named
                .iter()
                .any(|entry| value >= entry.region.base && value < entry.region.end())
            {
                found.push((at, value));
            }
        }
        detail!(
            "restore: {} static(s) of .data point into {which}: {}",
            found.len(),
            found
                .iter()
                .take(WHAT_POINTS_AT_MOST)
                .map(|(at, value)| format!("+{at:#x}={value:#x}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
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
unsafe fn fingerprint_untracked(regions: &[Owned]) -> Vec<Fingerprint> {
    // Nothing in a test build. The walk is a question about the *process*, and what it relies on is
    // the one thing a test binary cannot give it: everything that allocates is held still. Its caller
    // suspends the game's threads for it, so a region the walk found is a region that is still there
    // to be read; in a test binary the harness's other threads are running, and a region freed between
    // the walk and the read is a fault inside the read. Measured — the suite crashed here in parallel
    // and passed with `--test-threads=1`.
    //
    // **And the caller did not always suspend for it.** The guard was scoped to the copy loop and
    // dropped before this, so on a real host the walk went round a live process — which killed a launch
    // of th06nc with `--self-check` at the first chapter of stage 1, exactly the fault named above. See
    // `Snapshot::update`.
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
/// How many of the statics naming a source a line writes out, the rest being counted: a block a table of
/// pointers refers to is named by hundreds of them and the first few are as much as anybody reads.
const WHAT_POINTS_AT_MOST: usize = 24;

/// How many of the changed runs a line names, the rest being counted and not written: a span of a few
/// thousand bytes that the run has rewritten wholesale is a line nobody can read, and the count is what
/// says that is what happened.
const WHAT_CHANGED_AT_MOST: usize = 12;

/// How many bytes of a run are written out either side of it, longer runs being cut short: what names a
/// field is its first few bytes — a pointer, a handle, a count — and a hundred of them in a line is the
/// same as none.
const OF_A_RUN_AT_MOST: usize = 8;

/// **The bytes of `span` the run has changed since this snapshot took them**, as runs of
/// `(offset into the source, what the chapter has, what the run has)` — the shortlist a bisect of a
/// held-back span is looking for.
///
/// **Read before the suspend and so of this frame rather than of the restore.** The restore itself runs
/// with the game's threads stopped and may neither allocate nor log, and reading the live bytes twice to
/// have them at the exact moment of the write would be reading the game's memory for the sake of a log
/// line. A bisect wants to know which fields move at all, and a frame's difference does not change that.
///
/// # Safety
/// `span` must lie inside `entry`'s region, which must be mapped and readable.
unsafe fn what_the_run_changed(
    entry: &Saved,
    span: Range<usize>,
    from: usize,
    out: &mut Vec<(usize, Vec<u8>, Vec<u8>)>,
) {
    let into = span.start - entry.region.base;
    let theirs = &entry.data.bytes[into..into + span.len()];
    let now = unsafe { mem::read_bytes(span.start, span.len()) };
    let mut at = 0;
    while at < theirs.len() {
        if theirs[at] == now[at] {
            at += 1;
            continue;
        }
        let began = at;
        while at < theirs.len() && theirs[at] != now[at] {
            at += 1;
        }
        out.push((
            span.start + began - from,
            theirs[began..at].to_vec(),
            now[began..at].to_vec(),
        ));
    }
}

/// A run of bytes as a line can carry it, cut short past [`OF_A_RUN_AT_MOST`] with an ellipsis so that a
/// cut run is not read as a short one.
fn in_hex(bytes: &[u8]) -> String {
    let said = bytes
        .iter()
        .take(OF_A_RUN_AT_MOST)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");
    match bytes.len() > OF_A_RUN_AT_MOST {
        true => format!("{said}…"),
        false => said,
    }
}

fn overlaps(region: &Region, others: &[Owned]) -> bool {
    others
        .iter()
        .map(|owned| owned.region)
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

// Only on the target th06 and th07 are built for — see `game::th06c::tests`, which is what the other
// target's run of these is for.
#[cfg(all(test, target_arch = "x86"))]
mod tests {
    use std::sync::Arc;

    use super::{Audio, Music, Snapshot};
    use crate::game::th06::Th06;
    use crate::memtrack::{Owned, Region};
    use orb_api::Kind;
    use orb_api::mem::Source;
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

    /// A region and the source it came out of, as a walk of a laid-out space answers: its allocations,
    /// counted as they were mapped — see `orb_api::mem::Source::LaidOut`.
    fn laid_out_region(base: usize, len: usize, allocation: usize) -> Owned {
        Owned {
            region: Region { base, len },
            from: Source::LaidOut(allocation),
        }
    }

    fn regions() -> Vec<Owned> {
        vec![
            Owned {
                region: Region {
                    base: DATA,
                    len: LEN,
                },
                from: Source::Data,
            },
            laid_out_region(HEAP, LEN, 2),
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

    /// **A chapter that read no stream leaves the sound's own memory where it stands**, whatever else it puts
    /// back.
    ///
    /// What such a chapter holds of that memory is not a description of the sound now playing: the case it is
    /// most often is a snapshot taken while the game's own sound had an ask outstanding, where the player's
    /// stream field names the object the take-down before that ask deleted — see `Chapters::audio_for`. Bytes
    /// naming a released COM object are not bytes to put back into a live player, and a restore that put them
    /// back is what took th07 down on the real game: `0xc0000005` reading null at `th07.exe+0x5dc04` with
    /// `ecx` zero, on the game's own sound thread, one frame after a chapter retry. The chapter itself is kept
    /// — its own frame is where a retry has to land — and what is left out of it is the sound.
    ///
    /// **And left out whether or not the track is the one it was taken under**, which is where this differs
    /// from a chapter that keeps its music playing: that one is holding a stream it read and can hand back,
    /// and this one has nothing to hand back at all.
    #[test]
    fn the_sound_of_a_chapter_that_read_no_stream_is_left_where_it_stands() {
        let sim = laid_out();
        let _installed = sim.enter();
        let space = sim.space();
        // Where the sound's own memory is, which is the game's to say — and mapped here, an unmapped range
        // being one the copy would leave out for a different reason.
        let sound = crate::game::Game::audio_state(&Th06)
            .first()
            .cloned()
            .expect("the game says where its sound player is");
        space.map(sound.start, sound.len(), Kind::Private);
        space.fill_bytes(DATA, 0xa1, LEN);
        space.fill_bytes(sound.start, 0xa1, sound.len());
        let mut regions = regions();
        regions.push(laid_out_region(sound.start, sound.len(), 3));

        // A track named, so that the restore below is the harder case: the game answers with no track at all,
        // so this is a chapter whose own track is not the one playing.
        let audio = Audio {
            policy: Music::Rewind(None),
            identity: Some(7),
            state: vec![sound.clone()],
            thread: None,
        };
        let snapshot = unsafe { Snapshot::capture(&regions, audio, &[], false) };

        // The run goes on: the game's own data moves, and so does the sound's — the stream it plays through
        // now is not the one that was there when the chapter was taken.
        space.write::<u32>(DATA + 0x40, 0xdead_beef);
        space.fill_bytes(sound.start, 0xc3, sound.len());

        unsafe { snapshot.restore(&Th06) };

        assert!(
            space.read_bytes(DATA, LEN).iter().all(|byte| *byte == 0xa1),
            "the game's own memory did not come back",
        );
        assert!(
            space
                .read_bytes(sound.start, sound.len())
                .iter()
                .all(|byte| *byte == 0xc3),
            "the sound's own memory was written back out of a chapter that read no stream",
        );
    }
}
