//! **The handles a restore of a chapter of 妖々夢 leaves where it finds them, and everything beside them that
//! it puts back.**
//!
//! A chapter comes back by putting every byte of the game's memory back, allocator bookkeeping included, so
//! that nothing has to understand what a boss's script was in the middle of. **Direct3D and DirectSound objects
//! are the exception**: they cannot be copied into a snapshot, so a handle to one put back from a snapshot names
//! something that may since have been released — and the game releasing it a second time faults inside itself.
//!
//! 妖々夢's own array is the sprite manager's, which is the block the frame queues its quads in: `Th07::
//! live_handles` names it and a restore skips it. **The array only**: everything else in that block is the
//! game's own state and comes back like any other memory, which is the half of the claim that makes the first
//! half say something.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The sheet the manager holds when the chapter is taken, and the one it holds by the time the chapter is put
/// back — a fight's graphics loaded over it, which is what a run really does between two chapters.
///
/// Addresses and not objects: orb binds a texture and hands the array's bounds to a snapshot, and reads no word
/// through either. What they have to be is two numbers that can be told apart.
const AT_THE_CHAPTER: usize = 0x0500_0000;
const LOADED_SINCE: usize = 0x0600_0000;

/// And a word of the manager outside that array, which orb reads nothing of and a restore therefore puts back:
/// two numbers again, for the same reason.
const WORD_AT_THE_CHAPTER: u32 = 0x1111_1111;
const WORD_WRITTEN_SINCE: u32 = 0x2222_2222;

/// A texture handle the game holds is left where the restore finds it, and the manager around it comes back.
#[test]
fn a_texture_handle_of_th07_is_not_put_back_from_a_snapshot_and_the_rest_of_the_manager_is() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-handles",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        // The manager as the game has it when the chapter is taken: a sheet in the slot the panel is painted
        // from, and a word of its own beside it.
        game.image().loads_the_panels_sheet(AT_THE_CHAPTER);
        game.image().set_sprite_manager_word(WORD_AT_THE_CHAPTER);
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // What a run does between two chapters: the next fight's graphics loaded, which releases what was in
        // the slot and puts a new handle there. And a write of the game's own beside it.
        game.image().loads_the_panels_sheet(LOADED_SINCE);
        game.image().set_sprite_manager_word(WORD_WRITTEN_SINCE);

        // 被弾 and チャプターをやり直す.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let handles = game.image().sprite_textures().len();
        assert!(
            log.said(&format!("restore: skipping 1 range(s), {handles} bytes")),
            "the restore named no range of handles to leave alone:\n  {}",
            log.lines().join("\n  ")
        );

        // The handle is the live one, not the one the chapter was taken with: putting that back would hand the
        // game a texture it has already released, and the second release is a fault inside the game.
        assert_eq!(
            game.image().the_panels_sheet(),
            LOADED_SINCE,
            "the restore put back a handle to a texture the game had released",
        );
        // And the word beside it is the chapter's, which is what says the block was restored at all: the array
        // is skipped and nothing else is.
        assert_eq!(
            game.image().sprite_manager_word(),
            WORD_AT_THE_CHAPTER,
            "the whole of the manager's block was left alone, not the handles in it",
        );
    });
}

/// And the chain's own linked list comes back with the memory, which nothing had ever asked.
///
/// 妖々夢's `g_Chain` and the `ChainElem`s its scenes register are in `.data`, so a chapter is a copy of the
/// list as well as of the run: the `next` pointers, the callbacks and the elements themselves are all inside
/// the range a snapshot takes. Which means a job cut *after* the chapter is a job the restore puts back, and
/// none of that is anything orb does on purpose.
///
/// The panel's own draw job is the one to read it off, that list being where orb finds it — see
/// `Th07::panel_draw`. Its being in the chain at the chapter is the other half of the claim: a restore that
/// emptied the list would pass an assertion about the cut and mean the opposite.
#[test]
fn a_restore_of_a_chapter_of_th07_puts_the_chains_own_list_back() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-handles-the-chain",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        // The chain as it stood when the chapter was taken: the panel's draw job in it.
        assert!(
            game.image().paints_its_panel_now(),
            "the stage registered no draw job for its panel, so there is nothing to put back",
        );

        // The job cut, which is what the teardown of the object it belongs to does — and then 被弾 straight
        // after it, soon enough that no boundary of the stage's own has gone by.
        game.image().paints_no_panel();
        game.one_frame();
        assert!(
            !game.image().paints_its_panel_now(),
            "the job was not cut, so there is nothing for the restore to put back",
        );

        // チャプターをやり直す.
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert!(
            log.said("stage 1 chapter 1"),
            "the chapter put back is not the stage's own start:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.image().paints_its_panel_now(),
            "the restore left the list as the cut had it rather than as the chapter held it",
        );
    });
}
