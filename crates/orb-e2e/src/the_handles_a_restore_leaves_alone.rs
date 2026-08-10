//! **The handles a chapter's restore leaves where it finds them, and everything beside them that it puts
//! back.**
//!
//! What each e2e test holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
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

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
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

/// And the chain's own linked list comes back with the memory, which nothing had ever asked.
///
/// `g_Chain` and the static `ChainElem`s the scenes register are in `.data`, so a chapter is a copy of the
/// list as well as of the run: the `next` pointers, the callbacks and the elements themselves are all inside
/// the range a snapshot takes. Which means a job registered *after* the chapter is a job the restore takes
/// out of the chain, and one that was in it at the chapter is one the restore puts back — neither of which is
/// anything orb does on purpose.
///
/// A bomb's screen shake is the job to read it off: `ScreenEffect::RegisterChain` links its element into the
/// calc chain, and the chapter this run is put back to was taken before the bomb went off. `g_Gui`'s own draw
/// job is the other half — it *was* in the chain at the chapter — because a restore that emptied both lists
/// would pass the first assertion and mean the opposite.
#[test]
fn a_chapters_restore_puts_the_chains_own_list_back() {
    in_its_own_process(|| {
        let game = Fake::attach("the-handles-the-chain", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.in_a_pointdevice_run();
        // The chain as it stood when the chapter was taken: the panel's draw job in it, and no shake.
        assert!(
            game.image().gui_in_the_draw_chain(),
            "the stage registered no draw job for its panel, so there is nothing to put back",
        );
        assert!(
            !game.image().shaking_the_screen(),
            "a shake was already running when the chapter was taken",
        );

        // ボム, which links a job of its own into the calc chain, and then 被弾 straight after it: soon
        // enough that no boundary of the stage's own has gone by, so the chapter put back is the one taken
        // before the bomb.
        game.bombs();
        game.frame();
        assert!(
            game.image().shaking_the_screen(),
            "the bomb registered no job of its own, so there is nothing for the restore to take out",
        );

        // チャプターをやり直す.
        game.hit();
        game.frame();
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert!(
            log.said("stage 1 chapter 1"),
            "the chapter put back is not the one taken before the bomb:\n  {}",
            log.lines().join("\n  ")
        );

        assert!(
            !game.image().shaking_the_screen(),
            "the restore left a job in the chain that was not in it when the chapter was taken",
        );
        assert!(
            game.image().gui_in_the_draw_chain(),
            "the restore emptied the chain rather than putting back the list the chapter held",
        );
    });
}
