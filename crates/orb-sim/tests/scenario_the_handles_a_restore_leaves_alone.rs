//! **The handles a chapter's restore leaves where it finds them, and everything beside them that it puts
//! back.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! A chapter comes back by putting every byte of the game's memory back, allocator bookkeeping included,
//! so that nothing has to understand what a boss's script was in the middle of. **Direct3D and
//! DirectSound objects are the exception**: they cannot be copied into a snapshot, so a handle to one put
//! back from a snapshot names something that may since have been released — and the game releasing it a
//! second time faults inside itself. Which is what a step back across the frame a boss's graphics are
//! loaded on used to do.
//!
//! So `Th06::live_handles` names the anm manager's array of 264 texture pointers, and a restore skips it.
//! The array only: everything else in the same block is the game's own state and comes back like any
//! other memory, which is the half of the claim that makes the first half say something.

mod fake;

use fake::th06::{Fake, the_run};
use fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// The sheet the manager holds when the chapter is taken, and the one it holds by the time the chapter is
/// put back — a boss's graphics loaded over it, which is what a run really does between two chapters.
///
/// Addresses and not objects: orb binds a texture and hands the array's bounds to a snapshot, and reads
/// no word through either. What they have to be is two numbers that can be told apart.
const AT_THE_CHAPTER: usize = 0x0500_0000;
const LOADED_SINCE: usize = 0x0600_0000;

/// And a word of the manager outside that array, which orb reads nothing of and a restore therefore puts
/// back: two numbers again, for the same reason.
const WORD_AT_THE_CHAPTER: usize = 0x1111_1111;
const WORD_WRITTEN_SINCE: usize = 0x2222_2222;

/// How many bytes the skipped range is: the manager's 264 texture pointers, four bytes each.
const HANDLES: usize = 264 * size_of::<usize>();

/// A texture handle the game holds is left where the restore finds it, and the manager around it comes
/// back.
#[test]
fn a_texture_handle_is_not_put_back_from_a_snapshot_and_the_rest_of_the_manager_is() {
    in_its_own_process(|| {
        let game = Fake::attach("the-handles", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // The manager as the game has it when the chapter is taken: a sheet in the slot the panel is
        // painted from, and a word of its own beside it.
        game.image().loads_the_front_sheet(AT_THE_CHAPTER);
        game.image().set_anm_manager_word(WORD_AT_THE_CHAPTER);
        game.in_a_pointdevice_run();

        // The range was named as one to leave alone, and it is the array's own size: a snapshot that
        // skipped nothing would put the handle back and pass the first half of this by accident.
        let log = game.log();

        // What a run does between two chapters: the next fight's graphics loaded, which releases what was
        // in the slot and puts a new handle there. And a write of the game's own beside it.
        game.image().loads_the_front_sheet(LOADED_SINCE);
        game.image().set_anm_manager_word(WORD_WRITTEN_SINCE);

        // 被弾 and チャプターをやり直す.
        game.hit();
        game.frame();
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert!(
            log.said(&format!("restore: skipping 1 range(s), {HANDLES} bytes")),
            "the restore named no range of handles to leave alone:\n  {}",
            log.lines().join("\n  ")
        );

        // The handle is the live one, not the one the chapter was taken with: putting that back would
        // hand the game a texture it has already released, and the second release is a fault inside the
        // game.
        assert_eq!(
            game.image().front_sheet(),
            LOADED_SINCE,
            "the restore put back a handle to a texture the game had released",
        );
        // And the word beside it is the chapter's, which is what says the block was restored at all: the
        // array is skipped and nothing else is.
        assert_eq!(
            game.image().anm_manager_word(),
            WORD_AT_THE_CHAPTER,
            "the whole of the manager's block was left alone, not the handles in it",
        );
    });
}
