//! **A region of a chapter of th06nc that cannot be made writable: named, and the rest of the restore
//! still happens.**
//!
//! The same claim `a_region_a_restore_cannot_write.rs` makes of th06, `..._of_th07.rs` of th07 and
//! `a_region_a_restore_of_th06c_cannot_write.rs` of th06c, over the regions *this* game is in — and this
//! game's `.data` alone is `0x906660` bytes, nine megabytes before a heap is counted, so one region
//! refusing is a large share of a run to lose and a restore that gave up at the first refusal would leave
//! the game with half of one chapter and half of another.
//!
//! Why the commit happens outside the write loop at all is beside that loop: it runs with the game's
//! other threads suspended, and `VirtualAlloc` is exactly the call that must not happen there.
//!
//! **Driven by a recipe**, as every e2e test over a run of this game is: the retry is answered by the
//! recipe's own rules rather than by a key.
//!
//! **And the second test here is the shape a launch of the real game dies on.** Four launches of
//! `an_attempt_at_a_card_counted_and_kept`, and one of a probe that came at it through a death instead,
//! end on the frame a chapter inside a boss spellcard of *stage 1* is put back — and the launch that
//! retried the Extra's own `music=keep` boss spellcard thirty-one times, and the one that asked for the
//! Extra's chapter 8 back the same way the dying ones do, both lived. Every part either pair can be held
//! against the other matches: the kind of chapter, the music policy, the ask, `restore: skipping 11
//! range(s), 195276 bytes` to the byte, and the sound line.
//!
//! **And the snapshot's own shape is not it either**, which a launch with `Snapshot::covers` in it
//! settled: over the five chapters of stage 1 that launch took — the three `music=rewind` ones a gated
//! case retries and passes on, and the two `music=keep` ones a retry kills it at — every
//! `snapshot: covering` line is the *same* line, `0x7ff746d23000+0x906660` of `.data` and eight heap
//! regions after it, `9 region(s) of 12445280 bytes` apiece. So the shape is a property of the launch and
//! not of the chapter, and the counts that differ between launches — `7`, `8`, `11` — differ because the
//! heap does, not because a dying chapter holds anything a surviving one does not.
//!
//! So what the second test asks is not that difference but the claim orb can be held to whatever it turns
//! out to be: **a region that appeared after the chapter was taken is not in the snapshot, and a restore
//! of that chapter leaves it alone rather than writing over it.** A block the game allocated after the
//! snapshot is exactly such a region, and a restore that reverted the segment it is in would be reverting
//! a header the allocator still owns.

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

/// **Where a region that appears after a chapter was taken is put**, and how long it is: clear of
/// everything the laid-out game is in — its heap is `0x150040000` and `0x100000` long — so that mapping it
/// is a region appearing and not a region being changed.
const A_BLOCK_AFTER_THE_CHAPTER: usize = 0x1_6000_0000;
const ITS_BYTES: usize = 0x1_0000;

/// What is written into it, which is what says a restore left it alone: a byte no zeroed page holds.
const ITS_OWN_BYTE: u8 = 0xa5;

/// **How long the allocator's own state is**, which is what the third test looks for in the restore's own
/// skip line: `Th06nc`'s `THE_ALLOCATORS_OWN_BLOCK`, `0x14079e240`..`0x1407a6660`. Written out here
/// because a test expects it, the constant itself being the game module's own.
const THE_ALLOCATORS_OWN_BYTES: usize = 0x8420;

/// **The length of the object pointer a draw releases through, and the count beside it** — the qword at
/// `0x14093d728` and the dword at `0x14093d730`, which is `game::th06nc`'s `THE_DRAWS_OWN_OBJECT`.
/// Written out here for this file's own reason: a number read off the code under test is one the test
/// agrees with whatever either says.
const THE_DRAWS_OWN_OBJECTS_BYTES: usize = 0xc;

/// **And how long the object a conversation runs in is**, for the test below: `Th06nc`'s
/// `THE_CONVERSATIONS_BYTES`, the size `0x140041ce7` asks the CRT's `malloc` for. Written out here for
/// the same reason as the length above.
const THE_CONVERSATIONS_BYTES: usize = 0x4228;

fn launched(name: &str, recipe: String) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    )
}

/// **A chapter reached, then a hit taken and the chapter retried**: the recipe's own rules do the dying
/// and the answering, which is what a launch of the real one does.
///
/// **The frames after the retry are counted rather than a chapter**, because what both tests here are
/// about is the frames the restore is judged over and not where the run got to next.
fn a_chapter_then_a_retry() -> String {
    format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 400 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 600 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    )
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

#[test]
fn a_region_of_th06nc_that_cannot_be_committed_is_named_and_the_chapter_still_comes_back() {
    in_its_own_process(|| {
        let game = launched("th06nc-a-region-refused", a_chapter_then_a_retry());
        let log = game.log();

        // The chapter taken, which is what there is to put back.
        game.frames_until("a chapter of the run", FRAMES, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });

        // One of the regions a snapshot of this game covers, declared un-committable — the last of them
        // rather than the first, the first being the game's own data and a run that lost that would be no
        // run at all. Which region it is is the game's to say: what the claim is about is that *a* region
        // refusing is survivable, and the list comes out of the same walk the snapshot's does.
        let regions = game.sim().space().game_regions(&game.image().data());
        assert!(
            regions.len() > 1,
            "this game is one region, so the one refused would be its data",
        );
        let (base, len, _) = *regions
            .last()
            .expect("a game laid out in more than one region");
        game.sim().space().refuses_to_commit(base);

        // The death the rules bring, and the chapter put back after it.
        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        // The region is named, with the address and the length, so that whoever reads the log can go and
        // look at what is there.
        assert!(
            log.said(&format!("restore: cannot write {base:#010x}+{len:#x}")),
            "the region that could not be made writable was not named:\n  {}",
            log.lines().join("\n  ")
        );
        // And the run carried on: the frames of the stage move after the restore, which is what says the
        // refusal cost the region and not the game.
        //
        // The frames and not every field, because the region refused is one of the game's own and may be
        // the one this run's numbers are in — which is the whole point of the claim. What a refusal costs
        // is whatever was in that region; what it must not cost is the restore.
        let from = game.state().stage_frames;
        for _ in 0..30 {
            game.frame();
        }
        assert!(
            game.state().stage_frames > from,
            "the game stopped being played after a region refused to commit:\n  {}",
            log.lines().join("\n  ")
        );
        for fatal in ["panic:", "crash:"] {
            assert!(
                !log.said(fatal),
                "a region that could not be committed took the game down:\n  {}",
                log.lines().join("\n  ")
            );
        }
        played_out(&game);
    });
}

/// **A region that appeared after the chapter was taken is left alone by the restore**, which is the half
/// of the real game's own death that orb can be held to — see the module's own note for what the two pairs
/// of launches ruled out and what the snapshot's shape is.
#[test]
fn a_region_of_th06nc_that_appeared_after_the_chapter_is_not_written_back_over() {
    in_its_own_process(|| {
        let game = launched("th06nc-a-region-that-appeared", a_chapter_then_a_retry());
        let log = game.log();
        let space = game.sim().space();

        // The chapter taken, which fixes what the snapshot covers.
        game.frames_until("a chapter of the run", FRAMES, || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });
        let covered = space.game_regions(&game.image().data()).len();

        // And a region that was not there when it was taken, with a byte of its own written all through it:
        // a block the game allocated after the snapshot is a region of exactly this standing.
        space.map(A_BLOCK_AFTER_THE_CHAPTER, ITS_BYTES, orb_api::Kind::Private);
        for at in A_BLOCK_AFTER_THE_CHAPTER..A_BLOCK_AFTER_THE_CHAPTER + ITS_BYTES {
            space.write::<u8>(at, ITS_OWN_BYTE);
        }
        assert_eq!(
            space.game_regions(&game.image().data()).len(),
            covered + 1,
            "the region that appeared is not one a snapshot of this game would cover",
        );

        // The death the rules bring, and the chapter put back after it.
        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        // Every byte of it still its own, which is what says the restore wrote back what it had saved and
        // nothing beside it.
        let first = space.read::<u8>(A_BLOCK_AFTER_THE_CHAPTER);
        let last = space.read::<u8>(A_BLOCK_AFTER_THE_CHAPTER + ITS_BYTES - 1);
        assert_eq!(
            (first, last),
            (ITS_OWN_BYTE, ITS_OWN_BYTE),
            "the restore wrote over a region that appeared after the chapter it was putting back",
        );

        // And the run carried on out of it.
        let from = game.state().stage_frames;
        for _ in 0..30 {
            game.frame();
        }
        assert!(
            game.state().stage_frames > from,
            "the game stopped being played after the chapter was put back:\n  {}",
            log.lines().join("\n  ")
        );
        played_out(&game);
    });
}

/// **And the object a conversation runs in is another of them**, which is what stage 2's own death turned
/// out to be: two of its fields are pointers to blocks the stage's asset load frees and refills, so a
/// chapter of one stage holds pointers into memory a later stage has handed back — see
/// `Th06nc::live_handles`, `game::th06nc::THE_CONVERSATIONS_BYTES` and `asm.md`'s *The conversation*.
///
/// **Held to the whole allocation's length**, `0x4228`, because that is the decision the fix makes: the
/// span a bisect measured was the object's first `0x2000`, and what goes into `live_handles` is the
/// allocation `0x140041ce7` asks `malloc` for. A launch of the real game with this range held back
/// retried stage 2's first chapter twenty-nine times where every launch before it died there.
#[test]
fn the_conversation_of_th06nc_is_not_written_back_over() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-conversation", a_chapter_then_a_retry());
        let log = game.log();
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));
        // The block orb found, said before the restore reaches it — the line that would say so where the
        // pointer read as nought, which is a range a restore writes back.
        assert!(
            log.said(&format!(
                "memory: the conversation is {:#x} and {THE_CONVERSATIONS_BYTES:#x} bytes",
                game.image().the_conversation(),
            )),
            "orb did not find the conversation object where the game keeps it:\n  {}",
            log.lines().join("\n  ")
        );
        let skipped: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("restore: skipping"))
            .collect();
        assert!(
            skipped.iter().any(|line| line.contains(&format!(
                "{:#010x}+{THE_CONVERSATIONS_BYTES:#x}",
                game.image().the_conversation()
            ))),
            "the conversation object is not among the ranges the restore held back: {skipped:?}",
        );
        played_out(&game);
    });
}

/// **And the allocator's own state is one of the ranges a restore holds back**, which is what the real
/// game's death turned out to be: its TLSF checks the structure on every allocation, and a structure put
/// back from a snapshot describes an arena the heap has moved on from — see `Th06nc::live_handles` and
/// `th06nc/asm.md`'s *The allocator, which checks itself and kills the process*.
///
/// **The span is looked for in the restore's own skip line** rather than in anything of the laid-out
/// game's: the block is a static of the real one's, and what orb can be held to here is that a restore
/// names it among the ranges it does not write.
#[test]
fn the_allocators_own_state_of_th06nc_is_not_written_back_over() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-allocators-own", a_chapter_then_a_retry());
        let log = game.log();
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));
        let skipped: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("restore: skipping"))
            .collect();
        assert!(
            !skipped.is_empty(),
            "the restore held nothing back at all:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            skipped
                .iter()
                .any(|line| line.contains(&format!("+{THE_ALLOCATORS_OWN_BYTES:#x}"))),
            "no range of the allocator's own length is among the ones the restore held back: {skipped:?}",
        );
        played_out(&game);
    });
}

/// **And the one object a draw releases from outside the renderer's own ranges**, which is the third
/// restore fault of this game and the one cdb had to catch: with the renderer's objects and its counts
/// both held back, a launch still died at `th06nc+0x270e69` — `jmp qword ptr [rax+10h]` with `rax`
/// reading `0xa78`, the vtable of an object whose pointer a restore had put back. The static that held it
/// is `0x14093d728`, twelve bytes with the count that gates the release beside it. See
/// `Th06nc::live_handles`, `game::th06nc`'s `THE_DRAWS_OWN_OBJECT` and this change's own description.
///
/// **Held to its length in the restore's own skip line**, the way the allocator's is: the span is a
/// static of the real game's and what orb can be held to here is that a restore names it among the ranges
/// it does not write.
#[test]
fn the_object_a_draw_of_th06nc_releases_is_not_written_back_over() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-draws-own-object", a_chapter_then_a_retry());
        let log = game.log();
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));
        let skipped: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("restore: skipping"))
            .collect();
        assert!(
            !skipped.is_empty(),
            "the restore held nothing back at all:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            skipped
                .iter()
                .any(|line| line.contains(&format!("+{THE_DRAWS_OWN_OBJECTS_BYTES:#x}"))),
            "no range of that object's own length is among the ones the restore held back: {skipped:?}",
        );
        played_out(&game);
    });
}
