//! **The handles a restore of a chapter of th11 leaves where it finds them, and everything beside them that
//! it puts back.**
//!
//! What `the_handles_a_restore_leaves_alone_of_th10.rs` asks of that game, asked of this one. A chapter comes
//! back by putting every byte of the game's memory back, so that nothing has to understand what a boss's
//! script was in the middle of — and **Direct3D objects are the exception**: a handle put back from a snapshot
//! names something that may since have been released, and the game releasing it a second time faults inside
//! itself.
//!
//! **th11's are in the two arrays of every `.anm` file it has loaded**: its textures, `0x14` bytes each with its
//! `IDirect3DTexture9` first, and its sprites, `0x48` bytes each with the address of its texture's entry at `+0x8`
//! (0x454787 through 0x454797), which `Th11::live_handles` reads off the sprite manager's 32 slots and a restore
//! skips. **The arrays only**: they are allocations of the game's own heap, and the rest of that heap is the game's
//! state and comes back like any other memory — which is the half of the claim that makes the first half say
//! something. `th11.rs` holds the lists themselves; what this holds is a restore keeping to them.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// The slot a stage's graphics are loaded into, and how many textures and sprites it has: any slot and any
/// counts, what the test asks of them being that there are arrays to leave alone.
const A_SLOT: usize = 5;
const TEXTURES: usize = 3;
const SPRITES: usize = 4;

/// Where in a sprite's entry the address of its texture's entry is (0x454797).
const A_SPRITES_TEXTURE: usize = 8;

/// The texture each entry names by the time the chapter is put back — the file loaded again since, which
/// releases what was in it and puts new handles there. Addresses and not objects: orb reads no word through
/// any of them, so what they have to be is numbers that can be told from the ones the chapter was taken with.
const LOADED_SINCE: u32 = 0x0660_0000;

/// And a word of the heap beside those arrays, which orb reads nothing of and a restore therefore puts back:
/// two numbers again, for the same reason.
const WORD_AT_THE_CHAPTER: u32 = 0x1111_1111;
const WORD_WRITTEN_SINCE: u32 = 0x2222_2222;

/// The handle at the head of each entry of the textures and the texture's address in each sprite's.
fn the_handles(
    game: &Fake,
    textures: &std::ops::Range<usize>,
    sprites: &std::ops::Range<usize>,
) -> Vec<u32> {
    let space = game.image().space();
    let texture_bytes = textures.len() / TEXTURES;
    let sprite_bytes = sprites.len() / SPRITES;
    (0..TEXTURES)
        .map(|texture| space.read::<u32>(textures.start + texture * texture_bytes))
        .chain((0..SPRITES).map(|sprite| {
            space.read::<u32>(sprites.start + sprite * sprite_bytes + A_SPRITES_TEXTURE)
        }))
        .collect()
}

/// A texture handle the game holds is left where the restore finds it, and the heap around it comes back.
#[test]
fn a_texture_handle_of_th11_is_not_put_back_from_a_snapshot_and_the_rest_of_its_heap_is() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-handles",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();

        // The file as the game has it when the chapter is taken: its two arrays, and a word of the heap beside
        // them.
        let [textures, sprites] = game
            .image()
            .an_anm_file_is_loaded(A_SLOT, TEXTURES, SPRITES);
        let beside = textures.end;
        let space = game.image().space();
        space.write::<u32>(beside, WORD_AT_THE_CHAPTER);
        let at_the_chapter = the_handles(&game, &textures, &sprites);
        game.frames_until("the stage's first chapter", FRAMES_FOR_A_BUILD, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // What a run does between two chapters: the file loaded again, a new handle in each entry, and a write
        // of the game's own beside them.
        let texture_bytes = textures.len() / TEXTURES;
        let sprite_bytes = sprites.len() / SPRITES;
        for texture in 0..TEXTURES {
            space.write::<u32>(
                textures.start + texture * texture_bytes,
                LOADED_SINCE + texture as u32,
            );
        }
        for sprite in 0..SPRITES {
            space.write::<u32>(
                sprites.start + sprite * sprite_bytes + A_SPRITES_TEXTURE,
                LOADED_SINCE + (sprite % TEXTURES) as u32,
            );
        }
        let loaded_since = the_handles(&game, &textures, &sprites);
        space.write::<u32>(beside, WORD_WRITTEN_SINCE);

        // The hit and チャプターをやり直す.
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        for range in [&textures, &sprites] {
            assert!(
                log.said(&format!("{:#010x}+{:#x}", range.start, range.len())),
                "the restore named no range of the file's {:#010x} to leave alone:\n  {}",
                range.start,
                log.lines().join("\n  ")
            );
        }

        // The handles are the live ones, not the ones the chapter was taken with: putting those back would hand
        // the game textures it has already released, and the second release is a fault inside the game.
        assert_eq!(
            the_handles(&game, &textures, &sprites),
            loaded_since,
            "the restore put back handles to textures the game had released, where the chapter had {at_the_chapter:x?}",
        );
        // And the word beside them is the chapter's, which is what says the heap was restored at all: the
        // arrays are skipped and nothing else is.
        assert_eq!(
            space.read::<u32>(beside),
            WORD_AT_THE_CHAPTER,
            "the whole of the heap was left alone, not the handles in it",
        );
    });
}
