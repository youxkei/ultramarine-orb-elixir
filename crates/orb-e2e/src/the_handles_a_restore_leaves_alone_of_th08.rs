//! **The handles a restore of a chapter of th08 leaves where it finds them, and everything beside them that
//! it puts back.**
//!
//! What `the_handles_a_restore_leaves_alone_of_th07.rs` asks of that game, asked of this one. A chapter comes
//! back by putting every byte of the game's memory back, so that nothing has to understand what a boss's
//! script was in the middle of — and **Direct3D objects are the exception**: a handle put back from a snapshot
//! names something that may since have been released, and the game releasing it a second time faults inside
//! itself.
//!
//! **th08's are the sheets of every `.anm` slot it has loaded**: an array per slot of one texture and one
//! block apiece, which `Th08::live_handles` reads off the sprite manager's slots and a restore skips. **The
//! arrays only**: they are allocations of the game's own heap, and the rest of that heap is the game's state
//! and comes back like any other memory — which is the half of the claim that makes the first half say
//! something. `th08.rs` holds the lists themselves; what this holds is a restore keeping to them.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// The slot a stage's graphics are loaded into, and how many sheets it has: any slot and any count, what the
/// test asks of them being that there is an array to leave alone.
const A_SLOT: usize = 5;
const SHEETS: usize = 3;

/// The texture each sheet holds by the time the chapter is put back — the slot loaded again since, which
/// releases what was in it and puts new handles there. Addresses and not objects: orb reads no word through
/// either, so what they have to be is numbers that can be told from the ones the chapter was taken with.
const LOADED_SINCE: u32 = 0x0660_0000;

/// And a word of the heap beside that array, which orb reads nothing of and a restore therefore puts back:
/// two numbers again, for the same reason.
const WORD_AT_THE_CHAPTER: u32 = 0x1111_1111;
const WORD_WRITTEN_SINCE: u32 = 0x2222_2222;

/// One frame of a stage placed by hand, the run's clock stepped the way the stage's own jobs step it.
fn a_frame_of_the_stage(game: &Fake) {
    let image = game.image();
    image.has_been_in_the_run_for(image.the_frames_the_run_has_been_going() + 1);
    game.frame();
}

/// A texture handle the game holds is left where the restore finds it, and the heap around it comes back.
#[test]
fn a_texture_handle_of_th08_is_not_put_back_from_a_snapshot_and_the_rest_of_its_heap_is() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-handles",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();

        // The slot as the game has it when the chapter is taken: its sheets, and a word of the heap beside
        // them.
        let sheets = game.image().an_anm_slot_is_loaded(A_SLOT, SHEETS);
        let entry_bytes = sheets.len() / SHEETS;
        let beside = sheets.end;
        let space = game.image().space();
        space.write::<u32>(beside, WORD_AT_THE_CHAPTER);
        let at_the_chapter: Vec<u32> = (0..SHEETS)
            .map(|sheet| space.read::<u32>(sheets.start + sheet * entry_bytes))
            .collect();
        for _ in 0..FRAMES_FOR_A_BUILD {
            if log.said("stage 1 chapter 1 (stage start)") {
                break;
            }
            a_frame_of_the_stage(&game);
        }
        assert!(
            log.said("stage 1 chapter 1 (stage start)"),
            "the stage's first chapter was never taken:\n  {}",
            log.lines().join("\n  ")
        );

        // What a run does between two chapters: the slot loaded again, a new handle in each sheet, and a write
        // of the game's own beside them.
        for sheet in 0..SHEETS {
            space.write::<u32>(
                sheets.start + sheet * entry_bytes,
                LOADED_SINCE + sheet as u32,
            );
        }
        space.write::<u32>(beside, WORD_WRITTEN_SINCE);

        // The hit and チャプターをやり直す.
        game.image().has_died(1.0);
        game.image().the_player_has_been_hit();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert!(
            log.said(&format!("{:#010x}+{:#x}", sheets.start, sheets.len())),
            "the restore named no range of the slot's sheets to leave alone:\n  {}",
            log.lines().join("\n  ")
        );

        // The handles are the live ones, not the ones the chapter was taken with: putting those back would hand
        // the game textures it has already released, and the second release is a fault inside the game.
        let after: Vec<u32> = (0..SHEETS)
            .map(|sheet| space.read::<u32>(sheets.start + sheet * entry_bytes))
            .collect();
        assert_eq!(
            after,
            (0..SHEETS)
                .map(|sheet| LOADED_SINCE + sheet as u32)
                .collect::<Vec<_>>(),
            "the restore put back handles to textures the game had released, where the chapter had {at_the_chapter:x?}",
        );
        // And the word beside them is the chapter's, which is what says the heap was restored at all: the
        // array is skipped and nothing else is.
        assert_eq!(
            space.read::<u32>(beside),
            WORD_AT_THE_CHAPTER,
            "the whole of the heap was left alone, not the handles in it",
        );
    });
}
