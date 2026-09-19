//! **A span of the game's memory named by the source it is in, held back by a restore that puts the rest
//! of that source back.**
//!
//! `--hold-back-at=SOURCE:OFFSET+LEN` is the innermost of the fault flags — `--put-back-only` and
//! `--put-back-skipping` name a count of regions and this names a span inside one source — and what it is
//! for is bisecting a restore that kills the game: halve the span, launch, and read off which half the
//! game lives without. th06nc's own chapter retries were traced to one byte of `.data` that way, at
//! `.data + 0x911f5`, and the fix is the 153,898 bytes of `data/ascii.anm` the exe is built around it.
//!
//! **What this test is about is the name and not the halving.** The flag named a place in the snapshot's
//! region list first — `2:0+7800` for the second of them — and that is not a name: a heap's committed runs
//! are as many regions as its manager currently holds them in, and one launch of th06nc listed the game's
//! heap as 8 regions of `0xf000` in one chapter and 9 of `0x6000` in another, so the same `2:` named
//! different pages within the one run it was meant to bisect. A source — `.data`, the game's `n`th heap,
//! its `n`th reservation — is the same thing in the next launch, since the order the game takes them in is
//! the game's own and only their addresses are the OS's.
//!
//! So the two claims here are that a span named by its source is the span held back, and that a name no
//! source of this snapshot has is said in the log rather than held back silently — which is the answer a
//! bisect gets for a misspelling, and a bisect reading a launch as evidence of the span is a bisect
//! reading it of nothing.

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// How many frames a recipe is given to run out, and how long a wait inside one is given.
const FRAMES: u32 = 30_000;

/// **Two bytes of `.data` past everything this game has an address for**, whose highest static is
/// `0x140c6e368` where the range runs to `0x140c75884` — so what is written here is read by nothing and
/// what a restore does to them is the whole of what changes them.
///
/// Offsets and not addresses, because that is what the flag takes: `.data` begins at `0x14036d000`, and
/// the span held back is the first of them alone.
const HELD_BACK: usize = 0x90_0000;
const PUT_BACK: usize = 0x90_0100;

/// How long the span held back is, which covers the first of those two bytes and not the second.
const HELD_BACK_BYTES: usize = 0x10;

/// What the run has in them when the chapter is taken, and what it has in them by the time the chapter is
/// put back. Neither is a byte a zeroed page holds, so a read of one says which of the two it is.
const AT_THE_CHAPTER: u8 = 0x11;
const AFTER_IT: u8 = 0x22;

/// A source no walk of a laid-out game answers with: it holds the objects an e2e test put in it and has
/// neither heaps nor reservations, so this is the name a real bisect mistypes rather than a name this
/// snapshot could have.
const A_SOURCE_IT_HAS_NOT: &str = "heap9.1";

/// **The line that says the chapter's own copy has been made**, which is what both tests have to wait for
/// before they change a byte the chapter is supposed to hold.
///
/// **And not the `resume:` line naming the same chapter**, which is the one to reach for and is wrong: the
/// resume file for this chapter is written seven frames before the copy is taken, so a test that changed a
/// byte on that line changed it before the snapshot read it — which is a restore putting the run's own
/// value back and reading as though nothing was held back.
const THE_CHAPTER_TAKEN: &str = "stage 1 chapter 1 (stage start) at frame";

fn launched(name: &str, held: Vec<(String, usize, usize)>) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        move |config| {
            config.autoplay = Some(a_chapter_then_a_retry());
            config.log_level = LogLevel::Verbose;
            config.hold_back_at = held.clone();
        },
    )
}

/// **A chapter reached, then a hit taken and the chapter retried**: the recipe's own rules do the dying
/// and the answering, as a launch of the real one does.
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

/// **The span the flag named comes back as the run has it and the rest of its source as the chapter had
/// it**, which is the whole of what a bisect reads off a launch: the half of `.data` it held back is the
/// half the game was allowed to keep.
#[test]
fn a_span_named_by_its_source_is_the_span_a_restore_of_th06nc_holds_back() {
    in_its_own_process(|| {
        let game = launched(
            "th06nc-a-span-held-back",
            vec![("data".to_owned(), HELD_BACK, HELD_BACK_BYTES)],
        );
        let log = game.log();
        let space = game.sim().space();
        let data = game.image().data();

        // Nothing of the game is in either of them, which is what makes the two reads at the end about the
        // restore and not about whatever else wrote there.
        assert_eq!(space.read::<u8>(data.start + HELD_BACK), 0);
        assert_eq!(space.read::<u8>(data.start + PUT_BACK), 0);
        space.write::<u8>(data.start + HELD_BACK, AT_THE_CHAPTER);
        space.write::<u8>(data.start + PUT_BACK, AT_THE_CHAPTER);

        // The chapter taken, which is what holds those two bytes from here on.
        game.frames_until("a chapter of the run", FRAMES, || {
            log.said(THE_CHAPTER_TAKEN)
        });
        space.write::<u8>(data.start + HELD_BACK, AFTER_IT);
        space.write::<u8>(data.start + PUT_BACK, AFTER_IT);

        // The death the rules bring, and the chapter put back after it.
        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        assert_eq!(
            space.read::<u8>(data.start + HELD_BACK),
            AFTER_IT,
            "the span the launch held back was written back anyway:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            space.read::<u8>(data.start + PUT_BACK),
            AT_THE_CHAPTER,
            "the rest of the source did not come back, so the launch held back more than its span:\n  {}",
            log.lines().join("\n  ")
        );

        // And the launch says what it held back, with the source and the offset it was asked for: a bisect
        // that cannot tell from the log which span a launch ran with has nothing to hold the crash against.
        assert!(
            log.said(&format!(
                "restore: holding back {HELD_BACK_BYTES:#x} bytes of the {HELD_BACK_BYTES:#x} asked for \
                 at {HELD_BACK:#x} into data"
            )),
            "the span held back was not named:\n  {}",
            log.lines().join("\n  ")
        );
        played_out(&game);
    });
}

/// **A source this snapshot has nothing of is said and held back nothing**, which is what a bisect given a
/// name that is not one has to be able to read: a launch that lived is evidence about the span it held
/// back, and a launch that held back nothing is evidence about nothing.
#[test]
fn a_source_th06ncs_snapshot_has_nothing_of_is_named_in_the_log() {
    in_its_own_process(|| {
        let game = launched(
            "th06nc-a-source-it-has-not",
            vec![(A_SOURCE_IT_HAS_NOT.to_owned(), 0, HELD_BACK_BYTES)],
        );
        let log = game.log();
        let space = game.sim().space();
        let data = game.image().data();

        space.write::<u8>(data.start + HELD_BACK, AT_THE_CHAPTER);
        game.frames_until("a chapter of the run", FRAMES, || {
            log.said(THE_CHAPTER_TAKEN)
        });
        space.write::<u8>(data.start + HELD_BACK, AFTER_IT);

        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        assert!(
            log.said(&format!(
                "restore: this snapshot saved nothing of {A_SOURCE_IT_HAS_NOT}"
            )),
            "a source the snapshot has nothing of was not said:\n  {}",
            log.lines().join("\n  ")
        );
        // And the restore was the ordinary one: what the flag named was not there to hold back, so the
        // chapter's own bytes came back everywhere.
        assert_eq!(
            space.read::<u8>(data.start + HELD_BACK),
            AT_THE_CHAPTER,
            "a span nothing was held back for was held back all the same:\n  {}",
            log.lines().join("\n  ")
        );
        played_out(&game);
    });
}
