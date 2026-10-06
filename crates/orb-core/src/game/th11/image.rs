//! A th11 process image laid out in an address space, so that the real [`Th11`](super::Th11) has something to
//! read in a test with no game running.
//!
//! **What a frame of orb's own reads out of this space** is the device the game draws through, the window it
//! asks about, the chain the two walks are called on, the camera whose viewport it sets, the sprite manager
//! whose queue of quads it resets and draws around the draw walk and the word it forces the fog's call through
//! with — and the queue's draw, the camera's set, the fog's set, the options made for the power, the pause
//! menu's resume, a shot type's record made empty and the shot's step and move are functions an e2e test hands
//! over rather than code that would have to be here. See [`Image::hands_over_the_frames_own_calls`].
//!
//! **And a run, as far as [`Th11::read_state`](super::Th11) and the numbers a stage starts from read one**: the
//! supervisor's scenes, the title's screens, the game object with its flags and its frames, the run's numbers,
//! the generator, the player with its `.sht` record, the enemy manager's list with the stage's `main` on it, the
//! stage's `.std` with its fade, the pause object's states, the managers a stage builds and the sound manager's
//! objects. Each is laid out where the game itself keeps it, so a test can say what orb answers for a game in a
//! state the game can really be in.
//!
//! **The addresses and the offsets are the game's own, written from the same constants the reads use**, so what
//! this cannot catch is one of those that is wrong: a wrong one is wrong on both sides at once, and only the
//! real game says otherwise — every one of them was read against `th11.exe` of md5 `6af1e3b3`, see
//! `th11/asm.md`.
//!
//! **The values are this file's own, and taking one from [`Th11`](super::Th11) is what is refused here** — th07's
//! image has why: a scene number, a screen number, a state or a mask shared between the writer and the reader is
//! silent when wrong, and what it costs is a piece of the mode never happening. So each value below carries its
//! own literal and where in the exe it was read, and so does every offset `Th11` has no constant for.
//!
//! **The heap blocks' bases are this file's own**, the game taking each of those objects off its allocator:
//! what a read of one has to find is an address inside the space, and the length each block is laid out at is
//! the one its own allocation asks for, written beside it.
//!
//! **Nothing here goes through a slide**: this game is a 32-bit image with relocations stripped
//! (`Characteristics 0x103`), so the loader puts it at the `0x400000` its addresses are written against.

use std::ops::Range;
use std::sync::Arc;

use orb_api::{Device, Hwnd, Kind, Viewport};
use orb_sim::{Sim, Space};

/// The game's static data: `.data` from `0x4a2000`, `0x28e40` bytes by its section header to `0x4cae40`, of which the
/// file holds the first `0x6400` and the loader zeroes the rest — laid out here to the page the loader maps it to,
/// `.rsrc`'s start at `0x4cb000`, every static `Th11` names being short of it (the input's second entry and the
/// configuration menu's raw buttons at `0x4c98a0` the furthest).
const DATA: Range<usize> = 0x004a_2000..0x004c_b000;

/// **The sprite manager**, the `0x7bd894` bytes WinMain allocates at 0x445762 — which the queue of quads and the
/// 32 `.anm` slots are inside.
const SPRITES: Range<usize> = 0x0300_0000..0x0300_0000 + 0x7b_d894;
/// **The `.anm` files and the arrays they hang off**, each a `new(0x134)` of the loader's (0x454208) and the
/// entries it allocates behind it: one page per file here, the file at its start and its two arrays after it.
const ANM_FILES: Range<usize> = 0x0380_0000..0x0380_0000 + ANM_SLOTS * A_FILES_OWN_SPACE;
const ANM_SLOTS: usize = 0x20;
const A_FILES_OWN_SPACE: usize = 0x1_0000;
/// Where in a file's page its texture entries and its sprites are laid out — numbers of this file's own, far
/// enough apart that neither array reaches the other.
const TEXTURES_IN_THE_PAGE: usize = 0x200;
const SPRITES_IN_THE_PAGE: usize = 0x4000;
/// **What a loaded file's first texture reads as**, of this file's choosing: something rather than nothing,
/// and different for every entry.
const A_TEXTURE: u32 = 0x0420_0000;

/// **The objects a stage's build makes**, each at the length its own allocation asks for: the bullets
/// `0x46d678` (0x408710), the items `0x265e70` (0x4231e0), the player `0x8d24` (0x4301b0), the lasers `0x47c`
/// (0x424940), the pause object `0x2e0` (0x42bea1), the enemy manager `0x78` (0x411070), the card `0x924`
/// (0x40bcc0) and the bomb `0x4a0` (0x406180).
const BULLETS: Range<usize> = 0x0400_0000..0x0400_0000 + 0x46_d678;
const ITEMS: Range<usize> = 0x0450_0000..0x0450_0000 + 0x26_5e70;
const PLAYER: Range<usize> = 0x0480_0000..0x0480_0000 + 0x8d24;
const LASERS: Range<usize> = 0x0486_0000..0x0486_0000 + 0x47c;
const PAUSE: Range<usize> = 0x0487_0000..0x0487_0000 + 0x2e0;
const ENEMY_MANAGER: Range<usize> = 0x0482_0000..0x0482_0000 + 0x78;
const CARD: Range<usize> = 0x0489_0000..0x0489_0000 + 0x924;
const BOMB: Range<usize> = 0x048e_0000..0x048e_0000 + 0x4a0;
/// **The object a stage's conversations are run from**, `[0x4a8d84]`, `0x4458` bytes (`push 0x4458` at 0x41b2c2, its
/// constructor's `memset` at 0x41a381).
const PANEL: Range<usize> = 0x048a_0000..0x048a_0000 + 0x4458;
/// **Where a conversation the panel runs is**, an `0xac`-byte object of its own (0x41d2a0) at an address of this
/// file's own past the panel — see [`Image::a_conversation_is_up`].
const A_CONVERSATION: Range<usize> = 0x048a_c000..0x048a_c000 + 0xac;
/// **The shot type's record of the `.sht` the player keeps at `+0x92c`**, a pointer into the `.sht`'s own bytes
/// on the heap: as far as `+0xc`, the last of the three the build copies (0x42fae9 through 0x42fb15).
const SHT_RECORD: Range<usize> = 0x0480_c000..0x0480_c000 + 0x10;
/// **The DirectInput pad the input's set-up made, and its vtable**, where an e2e test has the enumeration find
/// one (0x447cb0): addresses of this file's own, the object a word and the vtable every slot up to `Poll`'s.
const PAD_DEVICE: Range<usize> = 0x0493_0000..0x0493_0000 + 0x10;
const PAD_DEVICE_VTABLE: Range<usize> = 0x0493_1000..0x0493_1000 + 26 * 4;
/// **And the keyboard it makes**, which it makes on the settings the game comes with (0x447ab0): the same
/// shape, addresses of this file's own.
const KEYBOARD_DEVICE: Range<usize> = 0x0493_2000..0x0493_2000 + 0x10;
const KEYBOARD_DEVICE_VTABLE: Range<usize> = 0x0493_3000..0x0493_3000 + 26 * 4;
/// **The replay**, `new(0x2dc)` (0x435e00).
const REPLAY_MANAGER: Range<usize> = 0x0492_0000..0x0492_0000 + 0x2dc;
/// **The game object**, `0x78` bytes (0x4202d0), one per build.
const GAME: Range<usize> = 0x0481_0000..0x0481_0000 + 0x78;
/// **The enemies a stage has on its list**, `0x2678` bytes apiece as the enemy the spawner allocates, each with
/// its own element of the list inside it at `+0x11a4` (0x4111c0, 0x410b60): as many as this file lays out.
const ENEMIES: Range<usize> = 0x0483_0000..0x0483_0000 + ENEMIES_LAID_OUT * ENEMY_BYTES;
const ENEMIES_LAID_OUT: usize = 8;
const ENEMY_BYTES: usize = 0x2678;
const ENEMY_ELEMENT: usize = 0x11a4;
/// **The element before**, the third word of an element (0x4111c0), and the list's tail at the manager's `+0x6c`.
const ELEMENT_BEFORE: usize = 0x8;
const ENEMY_LIST_TAIL: usize = 0x6c;
/// **The enemy's life as it started**, `+0x2530` (0x4108f0).
const ENEMY_LIFE_AT_THE_START: usize = 0x2530;
/// **The rest of the timer the frames of an enemy's attack are the count of**: the count as a float after it, and
/// the count of the frame before it — the timer stepping by the game's speed, `0x4a7948` at its `+0x119c`.
const ENEMY_ATTACK_FRAMES_AS_A_FLOAT: usize = 0x1198;
const ENEMY_ATTACK_FRAMES_BEFORE: usize = 0x1190;
/// **Two instructions of the stage's `.ecl` for its own scripts to stand on**: the `0x154` that waits while the boss
/// slot is set, and after it an instruction that does not wait — an address of this file's own, which orb reads the
/// opcode at `+0x4` of (see `Th11::the_stage_scripts_are_held`).
const SCRIPTS_STAND_ON: Range<usize> = 0x0494_0000..0x0494_0000 + 2 * AN_INSTRUCTION;
const AN_INSTRUCTION: usize = 0x10;
const WAITS_FOR_THE_BOSS: u16 = 0x154;
const WAITS_FOR_NOTHING: u16 = 0x0;
/// **The title**, `0x5b94` bytes (`push 0x5b94` at 0x439902).
const TITLE: Range<usize> = 0x0488_0000..0x0488_0000 + 0x5b94;
/// **The score object**, `0x2e21c` bytes (0x437822), read out of `scoreth11.dat` once as the game starts.
const SCORE: Range<usize> = 0x048b_0000..0x048b_0000 + 0x2_e21c;
/// **The chain**, the `0x4c` bytes WinMain allocates at 0x44565c.
const CHAIN: Range<usize> = 0x048f_0000..0x048f_0000 + 0x4c;
/// **The stage's `.std`**, `new(0x3158)` (0x4029c0), and the one a stage after one won keeps of the stage before
/// at `[0x4a8d5c]` (0x4200bc through 0x4200c7) — two blocks, the teardown copying the one into the other where
/// the game keeps the same allocation's address.
const STD: Range<usize> = 0x0491_4000..0x0491_4000 + STD_BYTES;
const THE_STAGE_BEFORE: Range<usize> = 0x0491_0000..0x0491_0000 + STD_BYTES;
const STD_BYTES: usize = 0x3158;
/// **The sound's objects off the heap**: the `new(4)` DirectSound is kept in (0x4491a6), the music's stream,
/// `new(0x7c)` (0x45a0fb), the `new(4)` its buffer is held in (0x45a46d), the source it reads through, `new(0x9c)`
/// (0x459f28), and `thbgm.fmt`'s table of records, `0x34` bytes apiece (0x428ab3).
const SOUND_DEVICE: Range<usize> = 0x0490_0000..0x0490_0000 + 4;
const STREAM: Range<usize> = 0x0490_1000..0x0490_1000 + 0x7c;
const STREAM_BUFFER: Range<usize> = 0x0490_2000..0x0490_2000 + 4;
const SOURCE: Range<usize> = 0x0490_3000..0x0490_3000 + 0x9c;
const TRACKS: Range<usize> = 0x0490_4000..0x0490_4000 + TRACKS_LAID_OUT * TRACK_RECORD_BYTES;
const TRACKS_LAID_OUT: usize = 4;
const TRACK_RECORD_BYTES: usize = 0x34;
/// **The names those records carry**, which are the names a play has to find: the lookup cuts the path past its
/// last `/` and holds what is left against each record's start, answering the first record where none matches
/// (0x4496b0) — and the paths the exe asks for are `bgm/th11_NN.wav`, the title's `th11_00` (`0x495f34`) and stage
/// 1's two, `th11_01` and `th11_02` (`0x495f24`, `0x495ed0`). The order the real `thbgm.fmt` keeps them in is in
/// that file and not the exe.
const TRACK_NAMES: [&std::ffi::CStr; TRACKS_LAID_OUT] = [
    c"th11_00.wav",
    c"th11_01.wav",
    c"th11_02.wav",
    c"th11_03.wav",
];
/// **Where the sound manager keeps that table**, `+0x1f84`: the loading thread stores `thbgm.fmt` at `0x4c5e04`
/// (0x428ab3), which is `0x4c3e80 + 0x1f84`.
const SOUND_TRACKS: usize = 0x1f84;

/// **The sound manager's own extent**, the `0x52f4` bytes `0x448ff0` zeroes: `0x4c3e80` to `0x4c9174`, inside
/// [`DATA`] and so a block of nothing of its own. This file's own literals, for the reason the module doc gives.
const SOUND_MANAGER: Range<usize> = 0x004c_3e80..0x004c_3e80 + 0x52f4;
/// **The queue of the sound's commands**: 31 of `0x10c` bytes from `+0x1f88`, each `{command, argument, step,
/// name}` — queued by 0x44a9c0 and carried out at the head and shifted by 0x44a340.
const COMMANDS: usize = 0x1f88;
const COMMAND_BYTES: usize = 0x10c;
const COMMAND_SLOTS: usize = 31;
const COMMAND_ARGUMENT: usize = 0x4;
const COMMAND_STEP: usize = 0x8;
const COMMAND_NAME: usize = 0xc;
const COMMAND_NAME_BYTES: usize = COMMAND_BYTES - COMMAND_NAME;
/// **The commands**: 1 a song's path into its slot, the one that lets the next run in the same call (0x44a3ca,
/// 0x44a402); 2 a play, its steps one a call, the stream's source put on the record at step 2 (0x44a690) and off
/// the queue at 7 (0x44a517); 3 a stop, off at 1 (0x44a584); and 4 the stream closed, off at 10 (0x44a525).
pub const A_SONGS_PATH: u32 = 1;
pub const A_PLAY: u32 = 2;
pub const A_STOP: u32 = 3;
pub const A_CLOSE: u32 = 4;
const A_PLAY_PUTS_THE_SOURCE_ON_AT: u32 = 2;
const A_PLAY_IS_DONE_AT: u32 = 7;
const A_STOP_IS_DONE_AT: u32 = 1;
const A_CLOSE_IS_DONE_AT: u32 = 10;
/// **The two commands the pause object sends**, 6 and 7, whose handlers are the stream's pause and its going on
/// (0x44a5e7, 0x44a605): its opening sends 6 (0x42c719) and its resume 7 (0x42c89e). The game sends neither in stage
/// 6's first 300 frames (0x42c6ff, 0x42c889), which nothing here lays out.
pub const THE_PAUSES_COMMAND: u32 = 6;
pub const THE_RESUMES_COMMAND: u32 = 7;

/// **The supervisor's twelve locks**, `CRITICAL_SECTION`s of `0x18` from `0x4c3a90` with a byte of depth apiece
/// after them to `0x4c3bbb` (0x44556f). This file's own literals.
const LOCKS: Range<usize> = 0x004c_3a90..0x004c_3bbc;
/// **The supervisor's three surfaces**, `0x4c342c` to `0x4c3438` (0x42b530). This file's own literals.
const SURFACES: Range<usize> = 0x004c_342c..0x004c_3438;

/// **The queue of quads**: how many times it has been drawn, `[esi+0xac]` one higher on each draw that had a
/// quad in it (0x44fd1d through 0x44fd94); and how many bytes a quad takes, six vertices of `0x1c` —
/// `DrawPrimitiveUP(4, count * 2, …, 0x1c)` at 0x44fd80.
const QUEUE_DRAWS: usize = 0xac;
const QUAD_BYTES: usize = 6 * 0x1c;
/// **The viewport the frame's camera `0x4c359c` keeps**: the whole of the game's 640x480 output, which its init gives
/// it (0x42ac34 through 0x42ac3e) — the play area's `32,16,384,448` being the camera at `0x4c3484`'s (0x42acb4
/// through 0x42acd2), which the frame does not set.
const THE_CAMERAS_VIEWPORT: Viewport = Viewport {
    x: 0,
    y: 0,
    width: 640,
    height: 480,
    min_z: 0.0,
    max_z: 1.0,
};

/// **The two words the camera set copies out of the camera into the sprite manager** (0x42a94c through 0x42a95e).
const CAMERA_WORDS_INTO_THE_SPRITES: [(usize, usize); 2] = [(0xe8, 0xb0), (0xec, 0xb4)];

/// **The supervisor's flags as its static initialiser leaves them**, `0x4240` (0x42ba06), with WinMain's `0x8000`
/// that the walks take their lock under (0x44555b).
const THE_SUPERVISORS_FLAGS: u32 = 0x4240 | 0x8000;
/// **The supervisor's word the title comes up by**, `[0x4c3250]`: 0 the first title, 1 back to the main menu, 2
/// the replays, 3 after the ending (screen 0's set-up, 0x439a6e on) — 3 written by scene 16 (0x42b214).
const G_HOW_THE_TITLE_COMES_UP: usize = 0x004c_3250;
/// **The character and the partner the selects remember**, `[0x4c325c]` and `[0x4c3260]` (0x43cfbc, 0x43d036).
const G_THE_CHARACTER_REMEMBERED: usize = 0x004c_325c;
const G_THE_PARTNER_REMEMBERED: usize = 0x004c_3260;
/// **The difficulty the title keeps**, `[0x4a3824]`, which the difficulty's decide writes (0x43cb23) and the
/// practice's item starts from (0x43a5b2). `.data`'s file part holds 1 there, Normal, which the image starts it at.
const G_THE_TITLES_DIFFICULTY: usize = 0x004a_3824;
/// **The supervisor's `+0x564`, the run's numbers started afresh** (`0x4c37e4`): 1 for scenes 7, 10, 11, 13 and 14
/// and nought for 12 (0x42b220 through 0x42b30e) — what the build puts the run's numbers in place on (0x41f932).
const G_A_FRESH_RUN: usize = 0x004c_37e4;
/// **The high score**, `0x4a56e0`, raised to the score where it is lower (0x42ba33).
const G_HIGH_SCORE: usize = 0x004a_56e0;
/// **The point items' value's start**, `0x4a5768`, which a death takes the value no lower than (0x433d10).
const G_POINT_VALUE_START: usize = 0x004a_5768;
/// **The frames since the last conversation of a new number began**, `0x4a5738` (0x41d339, 0x420474).
const G_FRAMES_SINCE_A_CONVERSATION: usize = 0x004a_5738;

/// **The title's count of items**, `+0x2c`, beside the cursor and the cursor the frame before at `+0x28`.
const TITLE_ITEMS: usize = 0x2c;
/// **The title's timer**, whose count is `Th11`'s `TITLE_STEPS_FRAMES`: its count before at `+0x2b0` and its float at
/// `+0x2b8` (0x4390e0, 0x439140).
const TITLE_TIMER_BEFORE: usize = 0x2b0;
const TITLE_TIMER_AS_A_FLOAT: usize = 0x2b8;
/// **The difficulty the Extra's item keeps**, `+0x58e8` (0x43a52e).
const TITLE_DIFFICULTY_KEPT: usize = 0x58e8;
/// **The bytes that say the Extra is open**, the endings of a run with no continue, six from `+0x2ddf0` of the score
/// object, each open where its bit `0x10` stands (0x439350).
const THE_EXTRA_IS_OPEN_FOR: Range<usize> = 0x2_ddf0..0x2_ddf6;
const AN_ENDING_OPENS_THE_EXTRA: u8 = 0x10;
/// **The endings seen**, a byte for each of the twelve from `+0x2ddea`: the ending's set-up writes 1, and `| 0x10`
/// where the difficulty is Normal or above (0x40ede8 through 0x40edfa); and the byte `+0x2ddf6` once an ending of six
/// or more is seen.
const THE_ENDINGS_SEEN: usize = 0x2_ddea;
const AN_ENDING_OF_SIX_OR_MORE_SEEN: usize = 0x2_ddf6;
const A_GOOD_ENDING_IS_ON_BY: i32 = 6;
/// **The settings and the clears**, `0x448` bytes from `+0x2ddd4` of the score object (0x4379e7).
const THE_SETTINGS: usize = 0x2_ddd4;
const SETTINGS_BYTES: usize = 0x448;
/// **The name the last entry was typed under**, the settings' `+0xc` — `+0x2dde0` of the score object.
const THE_LAST_NAME: usize = 0x2_dde0;
/// **A blank name**, eight spaces — what the entry's start is held against. The string itself was not read for
/// th11; th10's is eight spaces at `0x46e354`.
pub const A_BLANK_NAME: [u8; 8] = *b"        ";

/// **A shot type's record, as 0x437550 makes one empty**: its length, the ranking's five difficulties of ten
/// places in entries of `0x1c` from `+0x10`, and the cards' entries of `0x90` from `+0x664` each numbered at
/// `+0x88`. **What the ranking's defaults are was not read** — 0x437550 writes "the ranking's ten defaults per
/// difficulty" — so the scores and the name are th10's, 1,000,000 down by 100,000 under `--------`. This file's
/// own literals, for the reason the module doc gives.
const RECORD_BYTES: usize = 0x68d4;
const RANKED_DIFFICULTIES: usize = 5;
const RANKED_PLACES: usize = 10;
const RANKING_AT: usize = 0x10;
const RANKED_BYTES: usize = 0x1c;
const A_RANKINGS_TOP: u32 = 1_000_000;
const A_RANKINGS_STEP: u32 = 100_000;
const A_RANKINGS_NAME: &[u8; 8] = b"--------";
const CARDS_AT: usize = 0x664;
const CARDS: usize = 175;
const CARD_RECORD_BYTES: usize = 0x90;
const CARD_NUMBERED_AT: usize = 0x88;
/// **The plays a shot type's record counts**, `+0x588`, one more at every build with `+0x564` set and not for a
/// replay, held at 99,999 (0x41facd through 0x41fae1).
const RECORD_PLAYS: usize = 0x588;
const A_COUNT_STOPS_AT: u32 = 99_999;
/// **Each shot type's practice**, 25 entries of 8 bytes from `+0x59c` — `stage + difficulty * 6`, the stage from 1
/// — whose `+0x4` and `+0x5` a stage won sets (0x41eb92, 0x41eb61).
const PRACTICE_AT: usize = 0x59c;
const PRACTICE_BYTES: usize = 8;

/// **The game object's timer**, whose count is `+0x14`: its count before at `+0x10` and its float at `+0x18`.
const GAME_TIMER_BEFORE: usize = 0x10;
const GAME_TIMER_AS_A_FLOAT: usize = 0x18;

/// **The input's words past the two `Th11` names**, every one a `u32`: the repeat at `+0x8`, the buttons gone down
/// at `+0xc` and let go at `+0x10`, thirty-two counters of the live word from `+0x14` and the held-eight-frames
/// word at `+0x12c` (0x459b50).
const INPUT_REPEATED: usize = 0x8;
const INPUT_PRESSED: usize = 0xc;
const INPUT_RELEASED: usize = 0x10;
const INPUT_HELD_FOR: usize = 0x14;
const INPUT_HELD_EIGHT: usize = 0x12c;
/// **And the run's copy's**: its counters from `+0x94`, the word at `+0x118`, the frame before's at `+0x11c`, its
/// repeat, gone down and let go at `+0x120`, `+0x124` and `+0x128`, and its held-eight-frames word at `+0x130`
/// (0x40de10).
const RUN_HELD_FOR: usize = 0x94;
const RUN_WORD: usize = 0x118;
const RUN_WORD_BEFORE: usize = 0x11c;
const RUN_REPEATED: usize = 0x120;
const RUN_PRESSED: usize = 0x124;
const RUN_RELEASED: usize = 0x128;
const RUN_HELD_EIGHT: usize = 0x130;
/// **A bit's repeat**: on the 26th frame it is held (`cmp [eax],0x1a` at 0x459b82) and every eighth after, the
/// counter taken back by eight (0x459b8a); and the held-eight-frames word from its counter's eighth on (0x459b77).
const A_BIT_REPEATS_AT: u32 = 0x1a;
const A_REPEAT_TAKES_OFF: u32 = 8;
const HELD_EIGHT_FROM: u32 = 8;
/// **What the run's copy keeps of the word**, `& 0xffff` (0x43600d) — and the bit the shot and Shift pressed
/// together put in, where each one's counter of the copy was under 5 the frame before (0x43605a through 0x436076).
const THE_RUN_KEEPS: u32 = 0xffff;
const THE_SHOT: u32 = 0x1;
const SHIFT: u32 = 0x8;
const SHOT_AND_SHIFT_TOGETHER: u32 = 0x400;
const PRESSED_TOGETHER_WITHIN: u32 = 5;

/// **The rest of the run's numbers a fresh build writes**: the lives, 2 for a run and 9 for a practice (0x41fa12
/// through 0x41fa56); the point items' value's start, `[0x4a79e8 + difficulty * 4] * 100` — 25,000, 50,000,
/// 100,000, 200,000 and 200,000, times 100 (0x41f9eb, 0x41fa25); the word at `0x4a5744`, `-0x200` for a continued
/// run (0x41fae3 through 0x41fb01); and the continues at most 9 (0x41feec).
const A_FRESH_RUNS_LIVES: i32 = 2;
const A_PRACTICES_LIVES: i32 = 9;
const THE_POINT_VALUES_START: [i32; 5] = [25_000, 50_000, 100_000, 200_000, 200_000];
const THE_POINT_VALUE_IS_OVER: i32 = 100;
const A_CONTINUED_RUNS_4A5744: i32 = -0x200;
const CONTINUES_AT_MOST: i32 = 9;
/// **The run's flags the image writes**: `0x1` torn down on the stage the run started at, `0x2` the stage after one
/// won with its managers kept, `0x8` a continued run (0x41fe73, 0x41fefd, 0x41ff65), `0x10` a practice, `0x20` the
/// attract demo — and the `0xb` the build takes off as it ends (0x41fdc2).
const RUN_TORN_DOWN_ON_ITS_FIRST_STAGE: u32 = 0x1;
const RUN_KEEPS_ITS_MANAGERS: u32 = 0x2;
const RUN_WAS_CONTINUED: u32 = 0x8;
const RUN_IS_A_PRACTICE: u32 = 0x10;
const RUN_IS_THE_DEMO: u32 = 0x20;
const THE_BUILD_CLEARS: u32 = 0xb;
/// **The game's own speed**: the death puts `1.0` there (0x4314b8). What `.data` holds at `0x4a7948` before any run
/// was not read; the image starts it at the same.
const THE_GAMES_OWN_SPEED: f32 = 1.0;

/// **The power the `.sht` says a bomb costs and the most the shot type holds** — `[sht+0x24]` and `[sht+0x24] *
/// [sht+0x20]` (0x42fa66 through 0x42fa7c). The `.sht`'s numbers are in `th11.dat` and not in the exe, so these are
/// this file's own: what they have to be is a cost that leaves a run with a bomb or more.
const A_BOMB_COSTS: i32 = 20;
const THE_POWER_AT_MOST: i32 = 80;

/// **The player's own timers**: the state's, whose count is `Th11`'s `PLAYER_STATE_TIMER`, and the one every update
/// steps, `PLAYER_UPDATE_TIMER`; and its flags at `+0x8bc4`, `0x2` the player every test refuses (0x431e90).
const PLAYER_FLAGS: usize = 0x8bc4;
const PLAYER_IS_UNTOUCHABLE: u32 = 0x2;
/// **The player's options**, eight of `0xe4` from `+0x7570`, each's `+0xd4` the flag the options made for the power
/// sets (0x433623).
const PLAYER_OPTIONS: usize = 0x7570;
const PLAYER_OPTION_BYTES: usize = 0xe4;
const PLAYER_OPTIONS_LAID_OUT: usize = 8;
const OPTION_PUT_ON_ITS_PLACE: usize = 0xd4;
/// **Where the player is built**, in 128ths: across the middle and 400 down. **The constructor's own place was not
/// read** — this is the place state 0 walks the player up to (0x431095 through 0x4310d9) — and where a death puts
/// it back, 480 down, `0xf000` (0x4314b2 on).
const THE_PLAYER_IS_BUILT_AT: (i32, i32) = (0, 400 * 128);
const A_DEATH_PUTS_THE_PLAYER_AT: (i32, i32) = (0, 0xf000);
/// **The walk back in**, `0xf000 - frame * 0x2800 / 60` (0x431095 through 0x4310d9).
const COMING_BACK_IN_FROM: i32 = 0xf000;
const COMING_BACK_IN_BY: i32 = 0x2800;
const COMING_BACK_IN_FOR: i32 = 60;
/// **The player's half box**, half of `[0x4a7aa8 + character * 4]`: `1.0` for Reimu; and that table's own `2.0`,
/// the whole hitbox the `.sht` record keeps at `+0x4` (0x42fadc, 0x42fae9).
const REIMUS_HALF_BOX: f32 = 1.0;
const REIMUS_WHOLE_HITBOX: f32 = 2.0;
/// **What a 128th is as a float**, the `0.0078125` at `0x497ed0` (0x430859).
const A_128TH: f32 = 1.0 / 128.0;
/// **The player's speeds as the loader keeps them**, the `.sht`'s floats times the `128.0` at `0x497ed8` and
/// truncated: straight, focused straight, diagonal, focused diagonal. The floats are in `th11.dat` and not in the
/// exe, so these are this file's own — 4.5 and 2.0 straight, and each diagonal the straight times `sin(π/4)`.
const THE_PLAYERS_SPEEDS: [i32; 4] = [576, 256, 407, 181];
/// **The rectangle the player's 128ths are held inside**: `-0x5c00` to `0x5c00` across and `0x1000` to `0xd800`
/// down (0x4307c5 through 0x430849).
const THE_PLAYER_IS_HELD_ACROSS: (i32, i32) = (-0x5c00, 0x5c00);
/// Yukari's gap's states against the left and the right wall (0x4307de, 0x43080f).
pub const GAP_AGAINST_THE_LEFT: i32 = 1;
pub const GAP_AGAINST_THE_RIGHT: i32 = 3;
const THE_PLAYER_IS_HELD_DOWN: (i32, i32) = (0x1000, 0xd800);
/// **What a hit and a death write**: six frames nothing hits it for (0x432c29), 180 at a death (0x4328ac) and 280
/// coming back in (0x431506); the point items' value less 10,000,000 (0x4327d0); `0x400` off the word at `0x4a5744`,
/// held within ±`0x400` (0x432a52).
const A_HIT_LEAVES_IT_UNHITTABLE_FOR: i32 = 6;
const A_DEATH_LEAVES_IT_UNHITTABLE_FOR: i32 = 180;
const COMING_BACK_LEAVES_IT_UNHITTABLE_FOR: i32 = 280;
const A_DEATH_TAKES_OFF_THE_POINT_VALUE: i32 = 10_000_000;
const A_DEATH_TAKES_OFF_4A5744: i32 = 0x400;
/// **The enemy manager's count of the misses and the one beside it a death puts at nought** (0x431238 through
/// 0x43124a).
const ENEMY_MANAGERS_MISSES: usize = 0x10;
const ENEMY_MANAGERS_AFTER_A_MISS: usize = 0x18;
/// **The shot's step**: started on the frame the run's word holds the shot, and past 13 taken back 14 while it is
/// held and put at -1 where it is not (0x4343a7 through 0x43440b).
const THE_SHOTS_ROUND: i32 = 14;

/// **A bullet's own fields past the four `Th11` names**: its flag for the test, the states — 1 active, 0 a free slot,
/// 2 coming out and 3 cancelled (0x408916 through 0x408933, 0x408bda).
const A_BULLET_MAY_HIT: u32 = 0x2;
const A_BULLET_IS_ACTIVE: u16 = 1;
const A_FREE_SLOT: u16 = 0;
const A_BULLET_COMING_OUT: u16 = 2;
const A_BULLET_CANCELLED: u16 = 3;

/// **Where a fight's enemy is laid out**, the list's second place and the second of the enemies this file
/// lays out, behind `main` in the first.
const THE_FIGHTS_PLACE: usize = 1;
/// **Its body**, a size of this file's own: what the boss's own `.ecl` gives it is in `th11.dat`.
const A_FIGHTS_BODY: f32 = 32.0;
/// **Its flags**, the boss's `0x80000` (0x414ed0 through 0x414ee4).
const A_BOSSS_FLAGS: u32 = 0x80000;
/// **Where a wave's enemies are laid out**, the places behind the fight's — see [`Image::a_wave_is_up`].
const A_WAVES_PLACES: Range<usize> = THE_FIGHTS_PLACE + 1..ENEMIES_LAID_OUT;
/// **The flags of an enemy with a body**, `0x1000`, seen on the field — of this file's own: no launch read a wave's
/// flags off th11, and what they have to have is neither of the two bits that skip the body.
const A_WAVES_FLAGS: u32 = 0x1000;
/// And its body and its life, numbers of this file's own: what a wave's `.ecl` gives them is in `th11.dat`.
const A_WAVES_BODY: f32 = 24.0;
const A_WAVES_LIFE: i32 = 100;
/// **The flags `main` stands with**: `0x20`, one of the two bits that skip the body (0x411df9) — spawned with none (its
/// settings block zeroed at 0x42048d and its flags with the rest at 0x4111e9), the bit its own `.ecl`'s, and read off
/// a launch as `0x1020` on stage 1 (the_world's enemies table, 2026-10-07).
const MAINS_FLAGS: u32 = 0x20;
/// **What a card's start writes into its flags**, `& ~0x18 | 3` (0x40c6d8, 0x40c6db), and the bit its end takes off:
/// bit 0, the card running. And its bonus, `(difficulty + stage) * 1,000,000` (0x40c650), and its timer from nought.
const A_CARD_TAKES_OFF: u32 = 0x18;
const A_CARD_STARTS: u32 = 0x3;
const A_CARD_RUNS: u32 = 0x1;
const CARD_BONUS: usize = 0x8e4;
const A_CARDS_BONUS_STEP: u32 = 1_000_000;

/// **The `.std`'s fade**: its jobs' bit 1, fading out 2, fading in 4 and faded out 8 (0x405030, 0x405130, 0x4051d0,
/// 0x403874); and the counts the two fades start at, 30 and 60 (0x405130, 0x405215).
const STD_IS_ON: u32 = 0x1;
const STD_FADING_OUT: u32 = 0x2;
const STD_FADING_IN: u32 = 0x4;
const STD_FADED_OUT: u32 = 0x8;
const A_FADE_OUT_STARTS_AT: i32 = 30;
const A_FADE_IN_STARTS_AT: i32 = 60;
/// **Where the fade's calc steps no script**: bit 8 set, and bit 4 with the count at 30 or over (0x402b1c through
/// 0x402b36) — the calc's own return at a count of 60 or over (0x402ab4 through 0x402acf) being inside the second.
const THE_SCRIPT_IS_HELD_FROM: i32 = 30;
/// **The draw at priority 2**, which with bit 4 and a count under 30 puts the count at 1 (0x403348 through
/// 0x40336d).
const THE_FADE_IN_IS_CUT_TO: i32 = 1;

/// **The items' array the stage's first frame zeroes**, `0x265e50` bytes from `+0x14` — short of the ring's index
/// at `+0x265e68` (0x42044b through 0x42045b).
const THE_ITEMS_A_STAGE_ZEROES: usize = 0x26_5e50;
/// **The last stage the run's numbers count to**, 7 the Extra, and the stage word a cleared Extra is moved past to,
/// 8 (0x42d856 through 0x42d87b).
const THE_LAST_STAGE_WORD: i32 = 7;
const PAST_THE_EXTRA: i32 = 8;
/// **The record of the stages' files**, `0x4a3828` on by `0x40` a stage (0x42b2c1).
const STAGE_RECORDS: usize = 0x004a_3828;
const STAGE_RECORD_BYTES: usize = 0x40;

/// **The pause object's own fields past the `Th11` names**: its count of items at `+0x40`, the list of the items
/// left out from `+0xc8` with its count at `+0x10c`, the run cleared at `+0x1f4` (1 at 0x42d6b0, nought at
/// 0x42d560), the score not placed at `+0x1f8` (0x42d850), a replay's own pause at `+0x1fc`, the game's speed it held
/// back at `+0x2d8`, and the grid's length at `+0x118`. This file's own literals, for the reason the module doc gives.
const PAUSE_ITEMS: usize = 0x40;
const PAUSE_LEFT_OUT: usize = 0xc8;
const PAUSE_HOW_MANY_LEFT_OUT: usize = 0x10c;
const PAUSE_THE_RUN_WAS_CLEARED: usize = 0x1f4;
const PAUSE_OUT_OF_THE_RANKING: usize = 0x1f8;
const PAUSE_A_REPLAYS_OWN: usize = 0x1fc;
const PAUSE_SPEED_HELD: usize = 0x2d8;
const PAUSE_NAME_GRID_LENGTH: usize = 0x118;

/// **The scenes the supervisor's switch moves between** (0x42b0e0, the table at 0x42b35c), and the one it is on
/// before any of them, `-2`, which 0x4298c0 writes from WinMain (0x4298cd, called at 0x4457de) with the scene wanted
/// at 0 (0x4298d3). This file's own literals, for the reason the module doc gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    BeforeAny = -2,
    Startup = 0,
    Loading = 1,
    /// The game leaving (0x42b177), which the title's screen 2 asks for after the main menu's last item.
    TheGameLeaves = 3,
    Title = 4,
    Run = 7,
    StartOver = 10,
    AReplayStartedOver = 11,
    /// The stage after the one won (0x42b269), which a stage's end asks for (0x41eda3).
    NextStage = 12,
    AReplayFromTheTitle = 13,
    /// A continue (0x42b30e), the lost run's item 0 (0x42e5a7 through 0x42e5bb).
    Continue = 14,
    /// The ending (0x42b32f), which the panel asks for 380 frames after the last stage's end (0x41b3fc through
    /// 0x41b430).
    Ending = 15,
    /// The title again after the ending (0x42b1e9), built to come up on the screen after the ending.
    AfterTheEnding = 16,
}

/// **The title's screens this file plays** (the table at 0x439e38): its set-up, the main menu, the screen it leaves
/// on, the three selects a run is chosen through, the stage a practice is of, and the two screens after the ending.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Entry = 0,
    MainMenu = 1,
    Leaving = 2,
    Difficulty = 5,
    Character = 6,
    Partner = 7,
    Practice = 8,
    /// **The clear's own name entry** (0x43f6b0) — see [`Image::the_titles_entry`].
    AfterTheEnding = 14,
    /// **And the replay's slots after it** (0x43ffa0).
    TheReplaysSlots = 15,
}

/// What a name entry holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TheTitlesEntry {
    pub name: [u8; 8],
    pub typed: i32,
    pub not_asked: bool,
    pub grid: i32,
}

/// **The steps every one of those screens goes through**: the set-up, waiting out the frames it ignores a key for,
/// reading one, leaving forward and leaving back (asm.md's *The front end*).
pub const STEP_BUILT: i32 = 0;
pub const STEP_WAITING: i32 = 1;
pub const STEP_READING: i32 = 2;
pub const STEP_DECIDED: i32 = 3;
pub const STEP_LEFT: i32 = 4;

/// **The pause object's states this file plays** (0x42bf30, the tables at 0x42bfb4 and 0x42bfc4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pause {
    Nothing = 0,
    Opening = 1,
    /// A replay's own pause (0x42c760).
    AReplaysOwn = 2,
    Menu = 3,
    /// The item carried out twelve frames on (0x42d460).
    Closing = 4,
    /// The yes or no the title's item and the start-over's ask (0x42cc1b), which this file does not play.
    Confirming = 5,
    /// The run lost: its tally (0x42da94), its menu (0x42dbaf), the replay's slots, the ranking's name entry and
    /// the item carried out (0x42e582).
    ALostRunTallies = 13,
    ALostRunsMenu = 14,
    ALostRunSavesAReplay = 16,
    ALostRunNamesTheRanking = 18,
    ALostRunLeaves = 19,
    /// The run won: its tally (0x42e774), its menu (0x42e8e7), the replay's slots, the ranking's name entry and
    /// the item carried out (0x42f2b2).
    AWonRunTallies = 21,
    AWonRunsMenu = 22,
    AWonRunSavesAReplay = 23,
    AWonRunNamesTheRanking = 25,
    AWonRunLeaves = 26,
    /// Once 19 or 26 has carried its item out (0x42e592, 0x42f2bf).
    Done = 27,
}

/// **The player's states** (the switch at 0x43108e, its table `0x431ba4`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    ComingBackIn = 0,
    Playing = 1,
    Dying = 2,
    Hit = 4,
}

/// **The flags of the game object this file writes**: `0x4` while its stage is built (0x41f903, 0x41fdbe), `0x10`
/// paused (0x42c684, 0x42d601, 0x42d73f), `0x80` written while it waits on the build (0x4206cb) and `0x800` while the
/// stage before fades out under it (0x420424, 0x42043a).
const GAME_IS_BEING_BUILT: u32 = 0x4;
const GAME_IS_PAUSED: u32 = 0x10;
const GAME_WAITS_ON_ITS_BUILD: u32 = 0x80;
const GAME_FADES_THE_STAGE_BEFORE: u32 = 0x800;

/// **The slots of `IDirectInputDevice8` the read and the set-up call**: `Acquire` (`[vtbl+0x1c]`, 0x45741d),
/// `GetDeviceState` and `Poll` (`[vtbl+0x64]`, 0x457407).
pub const DEVICE_ACQUIRE: usize = 7;
pub const DEVICE_GET_STATE: usize = 9;
pub const DEVICE_POLL: usize = 25;
/// **The `JOYCAPSA` of joystick 0 the set-up's probe fills** (`joyGetDevCapsA(0, 0x4c2f08, 0x194)` at 0x4571a0).
const G_JOYSTICK_CAPS: usize = 0x004c_2f08;
/// **The configuration's two thresholds' default**, 600 apiece (0x420d93 through 0x420dad).
const A_THRESHOLDS_DEFAULT: i16 = 600;

pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    /// In a host whose non-determinism is drawn from `seed`, which an e2e test names in its assertions so that a
    /// failure can be replayed. The one constructor, for the reason th06's image says.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        let space = sim.space();
        for block in [
            &DATA,
            &SPRITES,
            &ANM_FILES,
            &BULLETS,
            &ITEMS,
            &PLAYER,
            &SHT_RECORD,
            &LASERS,
            &PAUSE,
            &ENEMY_MANAGER,
            &CARD,
            &PANEL,
            &A_CONVERSATION,
            &BOMB,
            &GAME,
            &ENEMIES,
            &TITLE,
            &SCORE,
            &CHAIN,
            &STD,
            &THE_STAGE_BEFORE,
            &SOUND_DEVICE,
            &STREAM,
            &STREAM_BUFFER,
            &SOURCE,
            &TRACKS,
            &REPLAY_MANAGER,
            &PAD_DEVICE,
            &PAD_DEVICE_VTABLE,
            &KEYBOARD_DEVICE,
            &KEYBOARD_DEVICE_VTABLE,
            &SCRIPTS_STAND_ON,
        ] {
            space.map(block.start, block.len(), Kind::Private);
        }
        space.write::<u16>(
            SCRIPTS_STAND_ON.start + super::INSTRUCTION_OPCODE,
            WAITS_FOR_THE_BOSS,
        );
        space.write::<u16>(
            SCRIPTS_STAND_ON.start + AN_INSTRUCTION + super::INSTRUCTION_OPCODE,
            WAITS_FOR_NOTHING,
        );
        // **What WinMain has made before its first frame**: the chain (0x44565c), the sprite manager (0x445762), and
        // the supervisor's flags.
        space.write::<usize>(super::G_CHAIN, CHAIN.start);
        space.write::<usize>(super::G_SPRITES, SPRITES.start);
        space.write::<u32>(super::G_INPUT_FLAGS, THE_SUPERVISORS_FLAGS);
        space.write::<f32>(super::G_GAME_SPEED, THE_GAMES_OWN_SPEED);
        // **The difficulty the selects come up on**, 1 — Normal — in `.data`'s file part at both `0x4a5720` and
        // `0x4a3824`.
        space.write::<i32>(super::G_DIFFICULTY, 1);
        space.write::<i32>(G_THE_TITLES_DIFFICULTY, 1);
        // **And the pad's mapping the static initialiser writes**, `0, 1, 2, 4, -1, -1, -1, -1, 3` (0x48abf2 through
        // 0x48ac44) — what a launch with no `th11.cfg` read over it plays with.
        space.write::<[i16; 9]>(super::G_PAD_MAPPING, [0, 1, 2, 4, -1, -1, -1, -1, 3]);
        // **The viewport the camera keeps**, which the frame sets after the camera — see [`THE_CAMERAS_VIEWPORT`].
        space.write::<Viewport>(
            super::THE_CAMERA + super::CAMERA_VIEWPORT,
            THE_CAMERAS_VIEWPORT,
        );
        space.write::<i32>(super::G_SCENE, Scene::BeforeAny as i32);
        space.write::<i32>(super::G_SCENE_WANTED, Scene::Startup as i32);
        let image = Self { sim };
        image.the_sound_is_up();
        image
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is what makes
    /// `Th11`'s reads land in it.
    pub fn enter(&self) -> orb_api::Installed {
        self.sim.enter()
    }

    pub fn space(&self) -> &Space {
        self.sim.space()
    }

    /// The host this game is laid out in, for an e2e test that needs more of it than the memory.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }

    /// The one range this game's statics are in, which is what a laid-out launch hands
    /// `orb_core::runtime::attach_to` in place of the section read out of the PE — see [`DATA`].
    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// The device the game shows through and the window its frame is called on, where the game's own
    /// `CreateDevice` (0x446f5d) and window (0x446cc3) left them.
    pub fn shows_through(&self, device: Device, window: Hwnd) {
        let space = self.space();
        space.write::<usize>(super::G_D3D_DEVICE, device.0);
        space.write::<Hwnd>(super::G_GAME_WINDOW, window);
    }

    /// The object the game calls its own whole frame on, which orb's loop is handed.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// **The configuration's window size**, `th11.cfg`'s `+0x1d` — nought in a launch laid out here, which is the
    /// whole screen.
    pub fn the_window_size(&self) -> u8 {
        self.space().read::<u8>(super::CFG_WINDOWED)
    }

    /// **The calls the game's own frame and its resume make that a laid-out game has to answer**, because code is
    /// the one thing an address space cannot hold: the queue drawn, the camera set, the fog set, the options made
    /// for the power and the pause menu's resume — in place of the addresses [`Th11`](super::Th11) holds.
    ///
    /// All of them together, because orb's frame and a resume make all of them: a game that handed over fewer
    /// would be jumping into memory nothing has mapped part way through one.
    pub fn hands_over_the_frames_own_calls(
        &self,
        draws_the_queue: usize,
        sets_the_camera: usize,
        sets_the_fog: usize,
        makes_the_options: usize,
        unpauses: usize,
    ) {
        super::set_the_queues_draw(draws_the_queue);
        super::set_the_cameras_set(sets_the_camera);
        super::set_the_fogs_set(sets_the_fog);
        super::set_the_options_made(makes_the_options);
        super::set_the_unpause(unpauses);
    }

    /// **And the two calls a pick-up makes as it lands in a stage begun out of a fade** — the shot's step and the
    /// shots' move — in place of 0x434380 and 0x434440. th10's third, the background's calc, has no counterpart: the
    /// landing writes the `.std`'s fade where the run's draw left it, and calls nothing of the background's.
    pub fn hands_over_the_landings_own_calls(&self, steps_the_shot: usize, moves_the_shots: usize) {
        super::set_the_shot_step(steps_the_shot);
        super::set_the_shots_moved(moves_the_shots);
    }

    fn sprites(&self) -> usize {
        SPRITES.start
    }

    /// **The queue drawn**, which is what 0x44fd10 does to the manager — see [`QUEUE_DRAWS`]: nothing where the
    /// count is nought, and otherwise the drawn-from cursor up to the write cursor, the draws one higher and the count
    /// to nought. The `DrawPrimitiveUP` and the states are a device's and not an address space's.
    pub fn draws_the_queue(&self) {
        let space = self.space();
        let sprites = self.sprites();
        if space.read::<u32>(sprites + super::SPRITES_QUEUED) == 0 {
            return;
        }
        let at = space.read::<usize>(sprites + super::SPRITES_WRITE_AT);
        space.write::<usize>(sprites + super::SPRITES_DRAWN_FROM, at);
        let draws = space.read::<u32>(sprites + QUEUE_DRAWS);
        space.write::<u32>(sprites + QUEUE_DRAWS, draws + 1);
        space.write::<u32>(sprites + super::SPRITES_QUEUED, 0);
    }

    /// A quad put in it, which is what the game's own drawing does between its frame's two draws of it.
    pub fn queues_a_quad(&self) {
        let space = self.space();
        let sprites = self.sprites();
        let at = space.read::<usize>(sprites + super::SPRITES_WRITE_AT);
        space.write::<[u8; QUAD_BYTES]>(at, [0xcd; QUAD_BYTES]);
        space.write::<usize>(sprites + super::SPRITES_WRITE_AT, at + QUAD_BYTES);
        let queued = space.read::<u32>(sprites + super::SPRITES_QUEUED);
        space.write::<u32>(sprites + super::SPRITES_QUEUED, queued + 1);
    }

    /// **And the queue put back at its start**, which is the frame's own and not a function's (0x446540 through
    /// 0x446557) — what a laid-out game's own frame writes where its frame writes it.
    pub fn the_queue_is_put_back_at_its_start(&self) {
        let space = self.space();
        let sprites = self.sprites();
        let start = sprites + super::SPRITES_QUEUE_STARTS;
        space.write::<u32>(sprites + super::SPRITES_QUEUED, 0);
        space.write::<usize>(sprites + super::SPRITES_WRITE_AT, start);
        space.write::<usize>(sprites + super::SPRITES_DRAWN_FROM, start);
    }

    /// How many quads are queued, how many times the queue has been drawn, where the next quad goes, where the
    /// queue starts and how far one quad moves the cursor — what an e2e test over orb's own frame reads back.
    pub fn queued(&self) -> u32 {
        self.space()
            .read::<u32>(self.sprites() + super::SPRITES_QUEUED)
    }

    pub fn queue_draws(&self) -> u32 {
        self.space().read::<u32>(self.sprites() + QUEUE_DRAWS)
    }

    pub fn queue_writes_at(&self) -> usize {
        self.space()
            .read::<usize>(self.sprites() + super::SPRITES_WRITE_AT)
    }

    pub fn queue_starts_at(&self) -> usize {
        self.sprites() + super::SPRITES_QUEUE_STARTS
    }

    pub fn queue_bytes_per_quad(&self) -> usize {
        QUAD_BYTES
    }

    /// **The camera set**, 0x42a810, entered with the camera in `edi`: its two transforms, the view into `+0x4c` and the
    /// projection into `+0x8c` and both to the device, and the camera's `+0xe8` and `+0xec` copied into the sprite
    /// manager's `+0xb0` and `+0xb4` (0x42a94c through 0x42a95e) — the last of which is what this lays out, the
    /// matrices being nothing a test reads.
    pub fn sets_the_camera(&self, camera: usize) {
        let space = self.space();
        for (from, to) in CAMERA_WORDS_INTO_THE_SPRITES {
            let word = space.read::<u32>(camera + from);
            space.write::<u32>(SPRITES.start + to, word);
        }
    }

    /// The viewport the camera keeps, which orb's frame sets after it.
    pub fn the_cameras_viewport(&self) -> Viewport {
        THE_CAMERAS_VIEWPORT
    }

    /// **The camera the frame sets**, named where the frame names it before the call (0x4464a9), and the word the
    /// frame writes `1` into after its viewport (0x4464d7).
    pub fn the_camera(&self) -> usize {
        super::THE_CAMERA
    }

    pub fn the_frame_names_its_camera(&self) {
        self.space()
            .write::<usize>(super::G_THE_CAMERA, super::THE_CAMERA);
    }

    pub fn the_frame_has_set_its_camera(&self) {
        self.space().write::<i32>(super::G_A_CAMERA_IS_SET, 1);
    }

    /// The supervisor, which the fog is set on, and the sound manager.
    pub fn the_supervisor(&self) -> usize {
        super::G_SUPERVISOR
    }

    pub fn the_sound_manager(&self) -> usize {
        super::G_SOUND_MANAGER
    }

    /// **The fog set**, which is 0x42a300: nothing where the supervisor's dword at `+0x990` is nought, and otherwise
    /// the queue drawn, the dword to nought and render state `0x1c` put to 0 (0x42a300 through 0x42a32f). Answers
    /// whether the device was told, which is the render state this laid-out game counts.
    pub fn sets_the_fog(&self) -> bool {
        let space = self.space();
        if space.read::<u32>(super::G_FOG_FORCED) == 0 {
            return false;
        }
        self.draws_the_queue();
        space.write::<u32>(super::G_FOG_FORCED, 0);
        true
    }

    /// **And the dword forced**, `0xff`, which the game's own frame writes before the call (0x446562).
    pub fn forces_the_fog_call_through(&self) {
        self.space().write::<u32>(super::G_FOG_FORCED, 0xff);
    }

    pub fn the_fog_word(&self) -> u32 {
        self.space().read::<u32>(super::G_FOG_FORCED)
    }

    /// The scene the supervisor is on and the one it wants, which is the pair `Th11::read_state` reads.
    pub fn is_in_scene(&self, scene: Scene, wanted: Scene) {
        let space = self.space();
        space.write::<i32>(super::G_SCENE, scene as i32);
        space.write::<i32>(super::G_SCENE_WANTED, wanted as i32);
    }

    /// The scene wanted, with the one now left as it stands — what a screen of the game writes.
    pub fn asks_for_the_scene(&self, wanted: Scene) {
        self.space()
            .write::<i32>(super::G_SCENE_WANTED, wanted as i32);
    }

    pub fn the_scene_now(&self) -> i32 {
        self.space().read::<i32>(super::G_SCENE)
    }

    pub fn the_scene_wanted(&self) -> i32 {
        self.space().read::<i32>(super::G_SCENE_WANTED)
    }

    /// **What the switch does to the scene once its case has run**: the scene it is on becomes the one it wanted
    /// (0x42b1bb, 0x42b1c1).
    pub fn the_switch_is_on_the_scene_it_wanted(&self) {
        let space = self.space();
        let wanted = space.read::<i32>(super::G_SCENE_WANTED);
        space.write::<i32>(super::G_SCENE, wanted);
    }

    /// **The supervisor's `+0x564`**, which the switch writes for the run it builds — see [`G_A_FRESH_RUN`].
    pub fn the_runs_numbers_are_started_afresh(&self, afresh: bool) {
        self.space().write::<i32>(G_A_FRESH_RUN, i32::from(afresh));
    }

    pub fn the_run_is_fresh(&self) -> bool {
        self.space().read::<i32>(G_A_FRESH_RUN) != 0
    }

    /// **How the title comes up** — see [`G_HOW_THE_TITLE_COMES_UP`].
    pub fn how_the_title_comes_up(&self) -> i32 {
        self.space().read::<i32>(G_HOW_THE_TITLE_COMES_UP)
    }

    pub fn the_title_is_to_come_up(&self, how: i32) {
        self.space().write::<i32>(G_HOW_THE_TITLE_COMES_UP, how);
    }

    /// **The title built**, the object zeroed where its constructor leaves it and stored.
    pub fn the_title_is_built(&self) {
        let space = self.space();
        space.fill_bytes(TITLE.start, 0, TITLE.len());
        space.write::<usize>(super::G_TITLE, TITLE.start);
    }

    /// **And freed**, its pointer at nought.
    pub fn the_title_is_freed(&self) {
        self.space().write::<usize>(super::G_TITLE, 0);
    }

    pub fn the_title_is_up(&self) -> bool {
        self.space().read::<usize>(super::G_TITLE) != 0
    }

    /// **A screen set**, 0x4390e0: the screen, its step at nought and its timer at nought.
    pub fn the_title_is_on(&self, screen: Screen) {
        let space = self.space();
        space.write::<i32>(TITLE.start + super::TITLE_SCREEN, screen as i32);
        self.the_titles_step_is(STEP_BUILT);
    }

    /// **A step set**, 0x439140: the step, and its timer at nought.
    pub fn the_titles_step_is(&self, step: i32) {
        let space = self.space();
        space.write::<i32>(TITLE.start + super::TITLE_STEP, step);
        space.write::<i32>(TITLE.start + TITLE_TIMER_BEFORE, -1);
        space.write::<i32>(TITLE.start + super::TITLE_STEPS_FRAMES, 0);
        space.write::<f32>(TITLE.start + TITLE_TIMER_AS_A_FLOAT, 0.0);
    }

    /// **One more frame of the screen**, the title's timer stepped.
    pub fn the_title_counts_a_frame(&self) {
        let space = self.space();
        let frames = space.read::<i32>(TITLE.start + super::TITLE_STEPS_FRAMES);
        space.write::<i32>(TITLE.start + TITLE_TIMER_BEFORE, frames);
        space.write::<i32>(TITLE.start + super::TITLE_STEPS_FRAMES, frames + 1);
        space.write::<f32>(TITLE.start + TITLE_TIMER_AS_A_FLOAT, (frames + 1) as f32);
    }

    /// Which screen of the title is up, its step, its cursor and its frames.
    pub fn where_the_title_is(&self) -> (i32, i32, i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(TITLE.start + super::TITLE_SCREEN),
            space.read::<i32>(TITLE.start + super::TITLE_STEP),
            space.read::<i32>(TITLE.start + super::TITLE_CURSOR),
            space.read::<i32>(TITLE.start + super::TITLE_STEPS_FRAMES),
        )
    }

    /// **The cursor and the count of items it moves over**, the count at `+0x2c`.
    pub fn the_titles_cursor_is(&self, cursor: i32, items: i32) {
        let space = self.space();
        space.write::<i32>(TITLE.start + super::TITLE_CURSOR, cursor);
        space.write::<i32>(TITLE.start + TITLE_ITEMS, items);
    }

    pub fn the_titles_cursor(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(TITLE.start + super::TITLE_CURSOR),
            space.read::<i32>(TITLE.start + TITLE_ITEMS),
        )
    }

    /// **The difficulty the Extra's item keeps** at the title's `+0x58e8` (0x43a52e).
    pub fn the_title_keeps_the_difficulty(&self) {
        let space = self.space();
        let difficulty = space.read::<i32>(super::G_DIFFICULTY);
        space.write::<i32>(TITLE.start + TITLE_DIFFICULTY_KEPT, difficulty);
    }

    /// **Whether the Extra is open**, any of the score object's bytes for it — see [`THE_EXTRA_IS_OPEN_FOR`].
    pub fn the_extra_is_open(&self) -> bool {
        let space = self.space();
        THE_EXTRA_IS_OPEN_FOR
            .clone()
            .any(|at| space.read::<u8>(SCORE.start + at) & AN_ENDING_OPENS_THE_EXTRA != 0)
    }

    /// **And whether it is open for one shot type**, `character * 3 + partner` — its own byte of the six, which the
    /// selects test the same way (0x439340). Which byte the selects test was read off the shape of the call and not
    /// followed into it.
    pub fn the_extra_is_open_for(&self, shot_type: i32) -> bool {
        self.space()
            .read::<u8>(SCORE.start + THE_EXTRA_IS_OPEN_FOR.start + shot_type.max(0) as usize)
            & AN_ENDING_OPENS_THE_EXTRA
            != 0
    }

    /// **The pad's mapping**, a button's number for the shot, the bomb, the focus, the pause, four unused and the
    /// skip, negative for none — see `Th11`'s `G_PAD_MAPPING`.
    pub fn the_pads_mapping(&self) -> [i16; 9] {
        self.space().read::<[i16; 9]>(super::G_PAD_MAPPING)
    }

    /// And mapped otherwise, as a `th11.cfg` read over the initialiser's own maps it (0x429f41 through 0x429f76).
    pub fn the_pad_is_mapped(&self, mapping: [i16; 9]) {
        self.space()
            .write::<[i16; 9]>(super::G_PAD_MAPPING, mapping);
    }

    /// **The caps of joystick 0 the input's set-up keeps** (0x4571a0): `joyGetDevCapsA(0)` into the `JOYCAPSA` at
    /// `0x4c2f08`, which the winmm branch of the read measures each axis against.
    pub fn the_joysticks_caps_are(&self, caps: &orb_api::JoyCaps) {
        self.space()
            .write::<orb_api::JoyCaps>(G_JOYSTICK_CAPS, *caps);
    }

    /// Those caps' two axes' bounds, `(x_min, x_max, y_min, y_max)` — nought where the probe found no joystick.
    pub fn the_joysticks_axes(&self) -> (u32, u32, u32, u32) {
        let caps = self.space().read::<orb_api::JoyCaps>(G_JOYSTICK_CAPS);
        (caps.x_min, caps.x_max, caps.y_min, caps.y_max)
    }

    /// **A DirectInput pad found by the input's set-up**: the device at `[0x4c32a4]` with its vtable's `Acquire`,
    /// `GetDeviceState` and `Poll` the functions given, and bit `0x800` of the input's flags set (0x4456c8 through
    /// 0x445702). The configuration's two thresholds stand at the 600 its defaults write.
    pub fn a_pad_is_found_through_directinput(
        &self,
        acquire: usize,
        get_state: usize,
        poll: usize,
    ) {
        let space = self.space();
        space.write::<usize>(super::G_PAD_DEVICE, PAD_DEVICE.start);
        space.write::<usize>(PAD_DEVICE.start, PAD_DEVICE_VTABLE.start);
        for (slot, function) in [
            (DEVICE_ACQUIRE, acquire),
            (DEVICE_GET_STATE, get_state),
            (DEVICE_POLL, poll),
        ] {
            space.write::<usize>(PAD_DEVICE_VTABLE.start + slot * 4, function);
        }
        let flags = space.read::<u32>(super::G_INPUT_FLAGS);
        space.write::<u32>(super::G_INPUT_FLAGS, flags | super::PAD_THROUGH_DIRECTINPUT);
    }

    /// **The configuration's defaults' two thresholds**, which 0x420d80 writes where no `th11.cfg` was read.
    pub fn the_configurations_defaults_are_in(&self) {
        let space = self.space();
        space.write::<i16>(super::CFG_PAD_ACROSS, A_THRESHOLDS_DEFAULT);
        space.write::<i16>(super::CFG_PAD_DOWN, A_THRESHOLDS_DEFAULT);
    }

    /// **The keyboard the input's set-up makes**, on the settings the game comes with: the device at `[0x4c32a0]`
    /// with its vtable's `Acquire` and `Unacquire` the functions given, and bit `0x400` of the input's flags set
    /// (0x4456c8 through 0x445702) — which is what sends the read through it rather than `GetKeyboardState`.
    pub fn the_input_set_up_makes_a_keyboard(&self, acquire: usize, unacquire: usize) {
        let space = self.space();
        space.write::<usize>(super::G_KEYBOARD_DEVICE, KEYBOARD_DEVICE.start);
        space.write::<usize>(KEYBOARD_DEVICE.start, KEYBOARD_DEVICE_VTABLE.start);
        for (slot, function) in [
            (DEVICE_ACQUIRE, acquire),
            (super::DEVICE_UNACQUIRE, unacquire),
        ] {
            space.write::<usize>(KEYBOARD_DEVICE_VTABLE.start + slot * 4, function);
        }
        let flags = space.read::<u32>(super::G_INPUT_FLAGS);
        space.write::<u32>(
            super::G_INPUT_FLAGS,
            flags | super::INPUT_THROUGH_DIRECTINPUT,
        );
    }

    /// Whether the read takes the keyboard through that device, bit `0x400` of the input's flags.
    pub fn reads_its_keyboard_through_directinput(&self) -> bool {
        self.space().read::<u32>(super::G_INPUT_FLAGS) & super::INPUT_THROUGH_DIRECTINPUT != 0
    }

    /// **The DirectInput pad the read reads, and the two thresholds its axes are read against** — `None` where the
    /// input's flags say there is none.
    pub fn the_directinput_pad(&self) -> Option<(usize, i16, i16)> {
        let space = self.space();
        (space.read::<u32>(super::G_INPUT_FLAGS) & super::PAD_THROUGH_DIRECTINPUT != 0).then(|| {
            (
                space.read::<usize>(super::G_PAD_DEVICE),
                space.read::<i16>(super::CFG_PAD_ACROSS),
                space.read::<i16>(super::CFG_PAD_DOWN),
            )
        })
    }

    /// **The word the read stores and what 0x459b50 works out of it**: the frame before's, this frame's, the buttons
    /// gone down and those let go, the auto-repeat off a counter a bit, and the held-eight-frames word.
    pub fn the_keys_read(&self, word: u32) {
        let space = self.space();
        let before = space.read::<u32>(super::G_INPUT);
        space.write::<u32>(super::G_INPUT_LAST, before);
        space.write::<u32>(super::G_INPUT, word);
        let (repeated, eight) = self.counts_its_bits(super::G_INPUT + INPUT_HELD_FOR, word);
        space.write::<u32>(super::G_INPUT + INPUT_REPEATED, repeated);
        space.write::<u32>(super::G_INPUT + INPUT_HELD_EIGHT, eight);
        space.write::<u32>(super::G_INPUT + INPUT_PRESSED, word & (word ^ before));
        space.write::<u32>(super::G_INPUT + INPUT_RELEASED, !word & (word ^ before));
    }

    /// **The thirty-two counters from `counters` stepped by `word`** (0x459b6d through 0x459b8f, and 0x40de1d through
    /// 0x40de72 for the run's copy): each bit's held frames, nought where it is not held, its repeat on the 26th and
    /// every eighth after, and its held-eight-frames bit. Answers the repeat and that word.
    fn counts_its_bits(&self, counters: usize, word: u32) -> (u32, u32) {
        let space = self.space();
        let mut repeated = 0;
        let mut eight = 0;
        for bit in 0..32 {
            let at = counters + bit * size_of::<u32>();
            let mut held_for = match word & (1 << bit) {
                0 => 0,
                _ => space.read::<u32>(at) + 1,
            };
            if held_for >= HELD_EIGHT_FROM {
                eight |= 1 << bit;
            }
            if held_for >= A_BIT_REPEATS_AT {
                repeated |= 1 << bit;
                held_for -= A_REPEAT_TAKES_OFF;
            }
            space.write::<u32>(at, held_for);
        }
        (repeated, eight)
    }

    /// The buttons a menu moves its cursor on, which are those gone down this frame and those repeating —
    /// `[0x4c92b4] | [0x4c92b0]` (0x42c9fb, 0x42ca04).
    pub fn the_buttons_moving_the_cursor(&self) -> u32 {
        let space = self.space();
        space.read::<u32>(super::G_INPUT + INPUT_PRESSED)
            | space.read::<u32>(super::G_INPUT + INPUT_REPEATED)
    }

    /// The buttons gone down this frame, which every menu takes decide and cancel off.
    pub fn the_buttons_gone_down(&self) -> u32 {
        self.space().read::<u32>(super::G_INPUT + INPUT_PRESSED)
    }

    /// **The run's copy of the word**, which the replay's calc job makes as it records (0x435fe0): the frame before's
    /// copy kept, the live word's low half, the bit the shot and Shift pressed together put in where bit `0x200` of
    /// the configuration's `+0x38` is clear — which the defaults leave it — and 0x40de10's edges, repeat and
    /// held-eight-frames word.
    pub fn the_run_copies_the_word(&self) {
        let space = self.space();
        let input = super::G_INPUT;
        let before = space.read::<u32>(input + RUN_WORD);
        let mut word = space.read::<u32>(input) & THE_RUN_KEEPS;
        let counted = |bit: usize| space.read::<u32>(input + RUN_HELD_FOR + bit * 4);
        if word & THE_SHOT != 0
            && word & SHIFT != 0
            && counted(0) < PRESSED_TOGETHER_WITHIN
            && counted(3) < PRESSED_TOGETHER_WITHIN
        {
            word |= SHOT_AND_SHIFT_TOGETHER;
        }
        space.write::<u32>(input + RUN_WORD_BEFORE, before);
        space.write::<u32>(input + RUN_WORD, word);
        let (repeated, eight) = self.counts_its_bits(input + RUN_HELD_FOR, word);
        space.write::<u32>(input + RUN_REPEATED, repeated);
        space.write::<u32>(input + RUN_HELD_EIGHT, eight);
        space.write::<u32>(input + RUN_PRESSED, word & (word ^ before));
        space.write::<u32>(input + RUN_RELEASED, !word & (word ^ before));
    }

    pub fn the_run_word(&self) -> u32 {
        self.space().read::<u32>(super::G_INPUT + RUN_WORD)
    }

    /// And the copy the frame before made.
    pub fn the_run_word_before(&self) -> u32 {
        self.space().read::<u32>(super::G_INPUT + RUN_WORD_BEFORE)
    }

    /// And the copy's buttons gone down, which the player's own bits that act on a press read (0x4308fe).
    pub fn the_runs_buttons_gone_down(&self) -> u32 {
        self.space().read::<u32>(super::G_INPUT + RUN_PRESSED)
    }

    /// The input's word this frame and the frame before's.
    pub fn the_words_the_keys_read(&self) -> (u32, u32) {
        let space = self.space();
        (
            space.read::<u32>(super::G_INPUT),
            space.read::<u32>(super::G_INPUT_LAST),
        )
    }

    /// **What a run's choice writes**: the difficulty, the character and the partner (0x43cb11, 0x43cfb6, 0x43d58d).
    pub fn the_difficulty_is(&self, difficulty: i32) {
        self.space().write::<i32>(super::G_DIFFICULTY, difficulty);
    }

    pub fn the_difficulty(&self) -> i32 {
        self.space().read::<i32>(super::G_DIFFICULTY)
    }

    /// **The difficulty the title keeps** — see [`G_THE_TITLES_DIFFICULTY`].
    pub fn the_titles_difficulty_is(&self, difficulty: i32) {
        self.space()
            .write::<i32>(G_THE_TITLES_DIFFICULTY, difficulty);
    }

    pub fn the_titles_difficulty(&self) -> i32 {
        self.space().read::<i32>(G_THE_TITLES_DIFFICULTY)
    }

    pub fn the_character_is(&self, character: i32) {
        self.space().write::<i32>(super::G_CHARACTER, character);
    }

    pub fn the_character(&self) -> i32 {
        self.space().read::<i32>(super::G_CHARACTER)
    }

    pub fn the_shot_is(&self, shot: i32) {
        self.space().write::<i32>(super::G_SHOT_TYPE, shot);
    }

    pub fn the_shot(&self) -> i32 {
        self.space().read::<i32>(super::G_SHOT_TYPE)
    }

    /// **The character and the partner the selects remember** — see [`G_THE_CHARACTER_REMEMBERED`].
    pub fn the_selects_remember(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(G_THE_CHARACTER_REMEMBERED),
            space.read::<i32>(G_THE_PARTNER_REMEMBERED),
        )
    }

    pub fn the_character_select_remembers(&self, character: i32) {
        self.space()
            .write::<i32>(G_THE_CHARACTER_REMEMBERED, character);
    }

    pub fn the_partner_select_remembers(&self, partner: i32) {
        self.space().write::<i32>(G_THE_PARTNER_REMEMBERED, partner);
    }

    /// **And the stage a run starts at**: the stage and the stage the run started at, both 1, and the record the
    /// build loads the stage's files through, `0x4a3868` — 0x43d5b3 through 0x43d5ec.
    pub fn the_run_starts_at_stage_1(&self) {
        self.the_practice_starts_at(1);
    }

    /// **And the Extra's**, which the same decide writes for difficulty 4: both stage words 7 and the record
    /// `0x4a39e8`.
    pub fn the_run_starts_at_the_extra(&self) {
        self.the_practice_starts_at(THE_LAST_STAGE_WORD);
    }

    /// **A cleared Extra moved past**, which 0x42d850 writes before it places the score where the stage word is 7
    /// and the run was cleared: both stage words 8 and the record past the Extra's, `0x4a3a28` (0x42d856 through
    /// 0x42d87b) — put back only where the stage still reads 7, which it does not. See `asm.md`'s *The pause and the
    /// end of a run*.
    pub fn a_cleared_extra_is_moved_past(&self) {
        let space = self.space();
        if space.read::<i32>(super::G_STAGE) != THE_LAST_STAGE_WORD || !self.the_run_was_cleared() {
            return;
        }
        self.the_practice_starts_at(PAST_THE_EXTRA);
    }

    pub fn the_stage_word(&self) -> i32 {
        self.space().read::<i32>(super::G_STAGE)
    }

    /// **The stage the run started at**, which scene 10 builds again (0x42b2b4 through 0x42b2c8).
    pub fn the_starting_stage(&self) -> i32 {
        self.space().read::<i32>(super::G_STARTING_STAGE)
    }

    /// **The game object built**, 0x4202d0: zeroed, its argument at `+0x74` (0x420325), its flag for a build set
    /// (0x42031b) and stored.
    pub fn the_game_object_is_built(&self, replaying: bool) {
        let space = self.space();
        space.fill_bytes(GAME.start, 0, GAME.len());
        space.write::<i32>(GAME.start + super::GAME_REPLAYING, i32::from(replaying));
        space.write::<u32>(GAME.start + super::GAME_FLAGS, GAME_IS_BEING_BUILT);
        space.write::<usize>(super::G_GAME, GAME.start);
    }

    /// The game object's flags and the frames it has run, read back.
    pub fn the_games_flags(&self) -> u32 {
        self.space().read::<u32>(GAME.start + super::GAME_FLAGS)
    }

    pub fn the_games_frames(&self) -> i32 {
        self.space().read::<i32>(GAME.start + super::GAME_FRAMES)
    }

    pub fn the_game_object_is_up(&self) -> bool {
        self.space().read::<usize>(super::G_GAME) != 0
    }

    /// **Whether the game object's flag for a build stands** — see [`GAME_IS_BEING_BUILT`].
    pub fn the_stage_is_being_built(&self) -> bool {
        self.the_games_flags() & GAME_IS_BEING_BUILT != 0
    }

    /// **The flag the game object's calc job writes while it waits on the build** (0x4206cb).
    pub fn the_game_object_waits_on_its_build(&self) {
        let space = self.space();
        let flags = space.read::<u32>(GAME.start + super::GAME_FLAGS);
        space.write::<u32>(
            GAME.start + super::GAME_FLAGS,
            flags | GAME_WAITS_ON_ITS_BUILD,
        );
    }

    /// **The run a replay being played**, the game object's `+0x74` — and the replay made to play one back, its
    /// mode at `+0x10` (0x4356ea), which is what the card start's count is gated on (0x40c6e4).
    pub fn the_run_is_a_replay_being_played(&self) {
        let space = self.space();
        space.write::<i32>(GAME.start + super::GAME_REPLAYING, 1);
        self.the_replay_is_built(true);
    }

    /// **The replay built**, 0x435e00 into 0x4356e0: `new(0x2dc)` zeroed, its mode at `+0x10` and stored — and where
    /// it records, the generator's draws counted from nought (0x43582a).
    pub fn the_replay_is_built(&self, playing_back: bool) {
        let space = self.space();
        space.fill_bytes(REPLAY_MANAGER.start, 0, REPLAY_MANAGER.len());
        space.write::<i32>(
            REPLAY_MANAGER.start + super::REPLAY_MANAGER_PLAYS_BACK,
            i32::from(playing_back),
        );
        space.write::<usize>(super::G_REPLAY_MANAGER, REPLAY_MANAGER.start);
        if !playing_back {
            self.the_generators_draws_start_over();
        }
    }

    /// **Whether the run is a replay being played**, the game object's `+0x74`.
    pub fn the_run_plays_a_replay(&self) -> bool {
        self.space().read::<i32>(GAME.start + super::GAME_REPLAYING) == 1
    }

    /// **The run's flags**, and the bits of them the image's writers set and take off — see [`RUN_IS_A_PRACTICE`] and
    /// the five beside it.
    pub fn the_runs_flags(&self) -> u32 {
        self.space().read::<u32>(super::G_RUN_FLAGS)
    }

    fn the_runs_flags_are(&self, on: u32, off: u32) {
        let space = self.space();
        let flags = space.read::<u32>(super::G_RUN_FLAGS);
        space.write::<u32>(super::G_RUN_FLAGS, flags & !off | on);
    }

    /// **Whether the run is the attract demo**, bit `0x20` of the run's flags — which no pause opens over
    /// (0x42bf4a).
    pub fn the_run_is_the_demo(&self) -> bool {
        self.the_runs_flags() & RUN_IS_THE_DEMO != 0
    }

    /// **The pause object up**, which is a stage built.
    pub fn the_pause_is_up(&self) -> bool {
        self.space().read::<usize>(super::G_PAUSE) != 0
    }

    /// The pause object itself.
    pub fn the_pause_object(&self) -> usize {
        PAUSE.start
    }

    /// And the player, which the options are made on.
    pub fn the_player_object(&self) -> usize {
        PLAYER.start
    }

    /// **The build's run's numbers for a run started afresh**, 0x41f932 through 0x41fb01 where `+0x564` is set: the
    /// difficulty 4 for stage 7, the high score raised, the continues at nought unless the run was continued, the
    /// point items' value at its start, the score, the graze and the life pieces at nought, the lives, the stage's
    /// frames at nought, the player built, the power by the stage, the plays the shot type's record counts, the
    /// extends at nought and the word at `0x4a5744` by the continue.
    pub fn the_builds_defaults_are_in(&self) {
        let space = self.space();
        let stage = space.read::<i32>(super::G_STAGE);
        if stage == THE_LAST_STAGE_WORD {
            space.write::<i32>(super::G_DIFFICULTY, 4);
        }
        let flags = self.the_runs_flags();
        let practice = flags & RUN_IS_A_PRACTICE != 0;
        let continued = flags & RUN_WAS_CONTINUED != 0;
        if !continued {
            space.write::<i32>(super::G_CONTINUES, 0);
        }
        let difficulty = space.read::<i32>(super::G_DIFFICULTY).clamp(0, 4) as usize;
        let start = THE_POINT_VALUES_START[difficulty] * THE_POINT_VALUE_IS_OVER;
        space.write::<i32>(G_POINT_VALUE_START, start);
        space.write::<i32>(super::G_POINT_VALUE, start);
        space.write::<u32>(super::G_SCORE, 0);
        space.write::<u32>(super::G_GRAZE, 0);
        space.write::<i32>(super::G_LIFE_PIECES, 0);
        space.write::<i32>(
            super::G_LIVES,
            match practice {
                true => A_PRACTICES_LIVES,
                false => A_FRESH_RUNS_LIVES,
            },
        );
        space.write::<u32>(super::G_STAGE_FRAMES, 0);
        self.the_player_is_built();
        let power = match (practice, stage) {
            (true, 2..=6) => THE_POWER_AT_MOST,
            _ => 0,
        };
        space.write::<i32>(super::G_POWER, power);
        let shot_type = (space.read::<i32>(super::G_CHARACTER) * 3
            + space.read::<i32>(super::G_SHOT_TYPE))
        .max(0) as usize;
        if !self.the_run_plays_a_replay() {
            let at = SCORE.start + super::RECORDS_AT + shot_type * RECORD_BYTES + RECORD_PLAYS;
            let plays = space.read::<u32>(at);
            space.write::<u32>(at, (plays + 1).min(A_COUNT_STOPS_AT));
        }
        space.write::<i32>(super::G_EXTENDS, 0);
        space.write::<i32>(
            super::G_4A5744,
            match continued {
                true => A_CONTINUED_RUNS_4A5744,
                false => 0,
            },
        );
    }

    /// **The player built**, 0x4301b0 and 0x42f8a0 from the build (0x41fa5b): zeroed and stored, where it is, its box,
    /// its `.sht` record and its four speeds; and the shot type's cost of a bomb and its most power out of the
    /// `.sht` (0x42fa66 through 0x42fa7c).
    fn the_player_is_built(&self) {
        let space = self.space();
        space.fill_bytes(PLAYER.start, 0, PLAYER.len());
        space.write::<usize>(super::G_PLAYER, PLAYER.start);
        let (x, y) = THE_PLAYER_IS_BUILT_AT;
        self.the_player_is_at_128ths(x, y);
        space.write::<f32>(PLAYER.start + super::PLAYER_HALF, REIMUS_HALF_BOX);
        space.write::<f32>(PLAYER.start + super::PLAYER_HALF + 4, REIMUS_HALF_BOX);
        space.fill_bytes(SHT_RECORD.start, 0, SHT_RECORD.len());
        space.write::<f32>(
            SHT_RECORD.start + super::SHT_WHOLE_HITBOX,
            REIMUS_WHOLE_HITBOX,
        );
        space.write::<usize>(PLAYER.start + super::PLAYER_SHT, SHT_RECORD.start);
        for (at, speed) in THE_PLAYERS_SPEEDS.into_iter().enumerate() {
            space.write::<i32>(PLAYER.start + super::PLAYER_SPEEDS + at * 4, speed);
        }
        space.write::<i32>(super::G_BOMB_COSTS, A_BOMB_COSTS);
        space.write::<i32>(super::G_POWER_AT_MOST, THE_POWER_AT_MOST);
    }

    /// **The stage's objects made after the replay** (0x41fc42 through 0x41fc91): the `.std`, the enemy manager with
    /// the `.ecl`, and every other manager of the stage, each stored where its own constructor stores it and zeroed
    /// where that constructor leaves it.
    pub fn the_stages_objects_are_made(&self) {
        let space = self.space();
        for (block, at) in [
            (&PANEL, super::G_PANEL),
            (&BULLETS, super::G_BULLET_MANAGER),
            (&ITEMS, super::G_ITEM_MANAGER),
            (&LASERS, super::G_LASER_MANAGER),
            (&PAUSE, super::G_PAUSE),
            (&CARD, super::G_CARD),
            (&BOMB, super::G_BOMB),
        ] {
            space.fill_bytes(block.start, 0, block.len());
            space.write::<usize>(at, block.start);
        }
        self.the_std_and_the_ecl_are_loaded();
    }

    /// **The objects a stage after the one won makes again**, the rest being kept for it (`[0x4a5758] & 2`, 0x41fc98):
    /// the `.std` and the enemy manager with the `.ecl`, which the build loads either way, and the bomb and the card,
    /// which the teardown freed (0x42016c through 0x4201b5) and the kept path joins the fresh one to make again
    /// (0x406440, 0x40bf90).
    pub fn the_next_stages_objects_are_made(&self) {
        let space = self.space();
        for (block, at) in [(&CARD, super::G_CARD), (&BOMB, super::G_BOMB)] {
            space.fill_bytes(block.start, 0, block.len());
            space.write::<usize>(at, block.start);
        }
        self.the_std_and_the_ecl_are_loaded();
    }

    /// **The `.std` loaded** (`0x4029c0([rec+0x4])`, 0x41fc42) **and the `.ecl`** (`0x411070([rec+0xc])`, which builds
    /// the enemy manager).
    fn the_std_and_the_ecl_are_loaded(&self) {
        let space = self.space();
        space.fill_bytes(STD.start, 0, STD.len());
        space.write::<usize>(super::G_STD, STD.start);
        space.fill_bytes(ENEMY_MANAGER.start, 0, ENEMY_MANAGER.len());
        space.write::<usize>(super::G_ENEMY_MANAGER, ENEMY_MANAGER.start);
    }

    /// **The game object's teardown on the way to the stage after the one won**, 0x41fe10 for scene 12: the run's
    /// flag for its managers kept (0x41fe73), the stage's `.std` kept for the next to fade from (0x4200bc through
    /// 0x4200c7), and the enemy manager, the bomb, the card and the game object freed (0x42016c through 0x4201b5).
    pub fn the_run_is_taken_down_for_the_next_stage(&self) {
        let space = self.space();
        self.the_runs_flags_are(RUN_KEEPS_ITS_MANAGERS, 0);
        let std = space.read_bytes(STD.start, STD.len());
        space.write_bytes(THE_STAGE_BEFORE.start, &std);
        space.write::<usize>(super::G_KEPT_STD, THE_STAGE_BEFORE.start);
        for at in [
            super::G_ENEMY_MANAGER,
            super::G_STD,
            super::G_BOMB,
            super::G_CARD,
            super::G_GAME,
        ] {
            space.write::<usize>(at, 0);
        }
    }

    /// **The game object's teardown for a continue**, 0x41fe10 for scene 14: on a stage past the one the run started
    /// at, one more continue used, at most 9, and the run's flag `0x8`; on the stage it started at, flag `0x1` and
    /// nothing counted (0x41fedb through 0x41ff65).
    pub fn the_run_is_taken_down_for_a_continue(&self) {
        let space = self.space();
        if space.read::<i32>(super::G_STARTING_STAGE) != space.read::<i32>(super::G_STAGE) {
            let continues = space.read::<i32>(super::G_CONTINUES);
            space.write::<i32>(super::G_CONTINUES, (continues + 1).min(CONTINUES_AT_MOST));
            self.the_runs_flags_are(RUN_WAS_CONTINUED, 0);
        } else {
            self.the_runs_flags_are(RUN_TORN_DOWN_ON_ITS_FIRST_STAGE, 0);
        }
        self.the_run_is_taken_down();
    }

    /// **The game object's teardown for a start-over**, scene 10: flag `0x1` on the stage the run started at, and the
    /// stage and its record put back to the stage the run started at (0x42b2b4 through 0x42b2c8).
    pub fn the_run_is_taken_down_to_start_over(&self) {
        let space = self.space();
        let starting = space.read::<i32>(super::G_STARTING_STAGE);
        if starting == space.read::<i32>(super::G_STAGE) {
            self.the_runs_flags_are(RUN_TORN_DOWN_ON_ITS_FIRST_STAGE, 0);
        }
        self.the_run_is_taken_down();
        self.the_stage_word_is(starting);
    }

    /// Whether the stage before is kept to fade from.
    pub fn the_stage_before_is_kept(&self) -> bool {
        self.space().read::<usize>(super::G_KEPT_STD) != 0
    }

    /// **The game object's flag while that `.std` fades out under the new stage** — see
    /// [`GAME_FADES_THE_STAGE_BEFORE`].
    pub fn the_stage_before_fades(&self, fading: bool) {
        let space = self.space();
        let flags = space.read::<u32>(GAME.start + super::GAME_FLAGS);
        let flags = match fading {
            true => flags | GAME_FADES_THE_STAGE_BEFORE,
            false => flags & !GAME_FADES_THE_STAGE_BEFORE,
        };
        space.write::<u32>(GAME.start + super::GAME_FLAGS, flags);
    }

    pub fn the_stage_before_fades_out(&self) -> bool {
        self.the_games_flags() & GAME_FADES_THE_STAGE_BEFORE != 0
    }

    /// **The kept `.std` freed once its fade-out is over**, `+0x2ff0 & 8` up (0x42069e, 0x4206a8).
    pub fn the_stage_before_is_freed_once_faded(&self) {
        let space = self.space();
        let kept = space.read::<usize>(super::G_KEPT_STD);
        if kept != 0 && space.read::<u32>(kept + super::STD_FADE_FLAGS) & STD_FADED_OUT != 0 {
            space.write::<usize>(super::G_KEPT_STD, 0);
        }
    }

    /// **The game object's timer started over**, 0x406100 on `+0x10` with nought (0x420686).
    pub fn the_games_timer_starts_over(&self) {
        let space = self.space();
        space.write::<i32>(GAME.start + GAME_TIMER_BEFORE, -1);
        space.write::<i32>(GAME.start + super::GAME_FRAMES, 0);
        space.write::<f32>(GAME.start + GAME_TIMER_AS_A_FLOAT, 0.0);
    }

    /// **A `.std` timer set**, 0x406100 on the fade's `+0x2ff4` with `count`: the count before one less, the count,
    /// its float, the game's speed it steps by and its set-up bit.
    fn the_fade_is_at(&self, std: usize, count: i32) {
        let space = self.space();
        space.write::<i32>(std + super::STD_FADE_BEFORE, count - 1);
        space.write::<i32>(std + super::STD_FADE_COUNT, count);
        space.write::<f32>(std + super::STD_FADE_AS_A_FLOAT, count as f32);
        space.write::<usize>(std + super::STD_FADE_STEPS_BY, super::G_GAME_SPEED);
        let flags = space.read::<u32>(std + super::STD_FADE_TIMER_FLAGS);
        space.write::<u32>(
            std + super::STD_FADE_TIMER_FLAGS,
            flags | super::TIMER_IS_SET_UP,
        );
    }

    /// **The new `.std`'s three jobs switched on**, 0x405030 on the game object's frame 0 (0x420408).
    pub fn the_background_is_switched_on(&self) {
        let space = self.space();
        let flags = space.read::<u32>(STD.start + super::STD_FADE_FLAGS);
        space.write::<u32>(STD.start + super::STD_FADE_FLAGS, flags | STD_IS_ON);
    }

    /// **The two fades a stage begun out of the last one's starts**: the old `.std` fading out from 30 (0x405130) and
    /// the new one fading in from 60 (0x4051d0, 0x405215 through 0x40522f).
    pub fn the_backgrounds_fade(&self) {
        let space = self.space();
        let kept = space.read::<usize>(super::G_KEPT_STD);
        if kept != 0 {
            let flags = space.read::<u32>(kept + super::STD_FADE_FLAGS);
            space.write::<u32>(kept + super::STD_FADE_FLAGS, flags | STD_FADING_OUT);
            self.the_fade_is_at(kept, A_FADE_OUT_STARTS_AT);
        }
        let flags = space.read::<u32>(STD.start + super::STD_FADE_FLAGS);
        space.write::<u32>(STD.start + super::STD_FADE_FLAGS, flags | STD_FADING_IN);
        self.the_fade_is_at(STD.start, A_FADE_IN_STARTS_AT);
    }

    /// **Whether the `.std`'s calc steps its script this frame** (0x402ab4 through 0x402b36): its jobs on, not faded
    /// out, and not inside the fade in's first thirty frames.
    pub fn the_background_steps_its_script(&self) -> bool {
        let space = self.space();
        if space.read::<usize>(super::G_STD) == 0 {
            return false;
        }
        let flags = space.read::<u32>(STD.start + super::STD_FADE_FLAGS);
        let count = space.read::<i32>(STD.start + super::STD_FADE_COUNT);
        let fading_in = flags & STD_FADING_IN != 0;
        flags & STD_IS_ON != 0
            && flags & STD_FADED_OUT == 0
            && !(fading_in && count >= THE_SCRIPT_IS_HELD_FROM)
    }

    /// **The `.std`s' two draws**, each over the `.std` it is the job of — the current one and the kept one: the draw at
    /// 2 cutting a fade in under 30 to 1 (0x403348 through 0x40336d), and the draw at 5 counting either fade down a
    /// frame and, at nought, taking both bits off and putting 8 on where it faded out (0x40383d through 0x403874).
    pub fn the_backgrounds_draw(&self) {
        let space = self.space();
        let kept = space.read::<usize>(super::G_KEPT_STD);
        let current = space.read::<usize>(super::G_STD);
        for std in [current, kept].into_iter().filter(|it| *it != 0) {
            let flags = space.read::<u32>(std + super::STD_FADE_FLAGS);
            if flags & STD_IS_ON == 0 && std == current {
                continue;
            }
            let count = space.read::<i32>(std + super::STD_FADE_COUNT);
            if flags & STD_FADING_IN != 0 && count < THE_SCRIPT_IS_HELD_FROM {
                self.the_fade_is_at(std, THE_FADE_IN_IS_CUT_TO);
            }
            if flags & (STD_FADING_IN | STD_FADING_OUT) == 0 {
                continue;
            }
            let count = space.read::<i32>(std + super::STD_FADE_COUNT) - 1;
            space.write::<i32>(std + super::STD_FADE_BEFORE, count + 1);
            space.write::<i32>(std + super::STD_FADE_COUNT, count.max(0));
            space.write::<f32>(std + super::STD_FADE_AS_A_FLOAT, count.max(0) as f32);
            if count <= 0 {
                let faded = match flags & STD_FADING_OUT {
                    0 => 0,
                    _ => STD_FADED_OUT,
                };
                space.write::<u32>(
                    std + super::STD_FADE_FLAGS,
                    flags & !(STD_FADING_IN | STD_FADING_OUT) | faded,
                );
            }
        }
    }

    /// **The `.std`'s fade as it stands**: its flags and its count.
    pub fn the_backgrounds_fade_stands_at(&self) -> (u32, i32) {
        let space = self.space();
        (
            space.read::<u32>(STD.start + super::STD_FADE_FLAGS),
            space.read::<i32>(STD.start + super::STD_FADE_COUNT),
        )
    }

    /// **The stage stepped on and its record with it**: the stage word one on under 7. **Where th11 steps the stage
    /// word between two stages was not read** — th10's 0x4175e0 did it at the stage's end — so this is that.
    pub fn the_run_moves_on_a_stage(&self) {
        let stage = self.the_stage_word();
        let stage = match stage < THE_LAST_STAGE_WORD {
            true => stage + 1,
            false => stage,
        };
        self.the_stage_word_is(stage);
    }

    /// **The stage word and its record put at `stage`**, counted from 1.
    pub fn the_stage_word_is(&self, stage: i32) {
        let space = self.space();
        space.write::<i32>(super::G_STAGE, stage);
        space.write::<usize>(
            super::G_STAGE_LOADS,
            STAGE_RECORDS + stage as usize * STAGE_RECORD_BYTES,
        );
    }

    /// **The run's flag for a practice set**, which the main menu's practice item writes (0x43a57e) where its item
    /// for a run takes it off (0x43a4f3).
    pub fn the_run_becomes_a_practice(&self) {
        self.the_runs_flags_are(RUN_IS_A_PRACTICE, 0);
    }

    pub fn the_run_is_no_practice(&self) {
        self.the_runs_flags_are(0, RUN_IS_A_PRACTICE);
    }

    /// **The stage a practice starts at**, counted from 1: the stage and the stage the run started at, and the
    /// record the build loads the stage's files through (0x43da11 through 0x43da3a).
    pub fn the_practice_starts_at(&self, stage: i32) {
        self.the_stage_word_is(stage);
        self.space().write::<i32>(super::G_STARTING_STAGE, stage);
    }

    /// **Whether the practice under the select's cursor is offered**, the byte at `+0x5` of the entry for `cursor + 1
    /// + difficulty * 6` from `+0x59c` of the shot type's record — `cmp byte [ecx+edx*8+0x5b1],0` at 0x43d80a.
    pub fn the_practice_is_offered(&self, difficulty: i32, cursor: i32) -> bool {
        let space = self.space();
        let shot_type =
            space.read::<i32>(super::G_CHARACTER) * 3 + space.read::<i32>(super::G_SHOT_TYPE);
        let record = SCORE.start + super::RECORDS_AT + shot_type.max(0) as usize * RECORD_BYTES;
        let entry = (difficulty * 6 + cursor + 1).max(0) as usize;
        space.read::<u8>(record + PRACTICE_AT + entry * PRACTICE_BYTES + 0x5) != 0
    }

    /// **A stage won offers its practice**: `+0x4` and `+0x5` of the entry `stage + difficulty * 6` set in the shot
    /// type's record, not for a replay or the Extra (0x41eb39 through 0x41eb92).
    pub fn a_stage_won_offers_its_practice(&self) {
        let space = self.space();
        let difficulty = space.read::<i32>(super::G_DIFFICULTY);
        if self.the_run_plays_a_replay() || difficulty >= 4 {
            return;
        }
        let shot_type =
            space.read::<i32>(super::G_CHARACTER) * 3 + space.read::<i32>(super::G_SHOT_TYPE);
        let record = SCORE.start + super::RECORDS_AT + shot_type.max(0) as usize * RECORD_BYTES;
        let entry = (difficulty * 6 + space.read::<i32>(super::G_STAGE)).max(0) as usize;
        let at = record + PRACTICE_AT + entry * PRACTICE_BYTES;
        space.write::<u8>(at + 0x4, 1);
        space.write::<u8>(at + 0x5, 1);
    }

    /// **The run's flag for the attract demo set**, which the title's demo writes as it starts one (0x439aa3).
    pub fn the_run_becomes_the_demo(&self) {
        self.the_runs_flags_are(RUN_IS_THE_DEMO, 0);
    }

    /// The game's speed, which every step of a frame is scaled by.
    pub fn the_games_speed(&self) -> f32 {
        self.space().read::<f32>(super::G_GAME_SPEED)
    }

    /// **The player's four speeds**, as the loader keeps them.
    pub fn the_players_speeds(&self) -> [i32; 4] {
        let space = self.space();
        [0, 1, 2, 3].map(|at| space.read::<i32>(PLAYER.start + super::PLAYER_SPEEDS + at * 4))
    }

    /// **The player's step added to where it is and held inside the field** (0x4307c5 through 0x430870).
    pub fn the_player_steps_by(&self, across: i32, down: i32) {
        self.the_player_moves_by(across, down);
        self.the_player_is_held_inside_the_field();
    }

    /// **The player's step added to where it is**, held nowhere (0x43073a, 0x430740).
    pub fn the_player_moves_by(&self, across: i32, down: i32) {
        let (x, y) = self.where_the_player_is_in_128ths();
        self.the_player_is_at_128ths(x + across, y + down);
    }

    /// **The player held inside the field** (0x4307b0 through 0x430849), and put against the wall it was held off in
    /// Yukari's gap's state 1 or 3 where nothing of the gap is up and the shot type is 0 — and not held at all while
    /// it is carried through the gap.
    pub fn the_player_is_held_inside_the_field(&self) {
        let (state, _) = self.the_players_gap();
        if state >= super::GAP_STATE_THROUGH {
            return;
        }
        let (x, y) = self.where_the_player_is_in_128ths();
        let (left, right) = THE_PLAYER_IS_HELD_ACROSS;
        let (top, bottom) = THE_PLAYER_IS_HELD_DOWN;
        let the_gaps = state == 0
            && self.the_character() * super::SHOT_TYPES_A_CHARACTER + self.the_shot()
                == super::SHOT_TYPE_WITH_A_GAP;
        if the_gaps && x < left {
            self.the_players_gap_is(GAP_AGAINST_THE_LEFT, 0);
        }
        if the_gaps && x > right {
            self.the_players_gap_is(GAP_AGAINST_THE_RIGHT, 0);
        }
        self.the_player_is_at_128ths(x.clamp(left, right), y.clamp(top, bottom));
    }

    /// **Where Yukari's gap is, and its count of frames**, `+0x920` and `+0x924` — see `Th11`'s `PLAYER_GAP_STATE`.
    pub fn the_players_gap(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(PLAYER.start + super::PLAYER_GAP_STATE),
            space.read::<i32>(PLAYER.start + super::PLAYER_GAP_STATE + 4),
        )
    }

    pub fn the_players_gap_is(&self, state: i32, frames: i32) {
        let space = self.space();
        space.write::<i32>(PLAYER.start + super::PLAYER_GAP_STATE, state);
        space.write::<i32>(PLAYER.start + super::PLAYER_GAP_STATE + 4, frames);
    }

    /// **Whether the player is focused**, `+0x8d20` — the run's word's `0x8` while an enemy is up and the update's
    /// timer is 4 or more, nought otherwise (0x430373, 0x43037b).
    pub fn the_player_is_focused(&self, focused: bool) {
        self.space()
            .write::<i32>(PLAYER.start + super::PLAYER_FOCUSED, i32::from(focused));
    }

    pub fn the_player_stands_focused(&self) -> bool {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_FOCUSED)
            != 0
    }

    /// The update's timer's count, which holds the focus off for its first four frames.
    pub fn the_players_update_timer(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_UPDATE_TIMER)
    }

    /// The half of the player's box on each axis, and the whole hitbox the `.sht` record keeps for the circle test.
    pub fn the_players_half_box(&self) -> f32 {
        self.space().read::<f32>(PLAYER.start + super::PLAYER_HALF)
    }

    pub fn the_players_whole_hitbox(&self) -> f32 {
        self.space()
            .read::<f32>(SHT_RECORD.start + super::SHT_WHOLE_HITBOX)
    }

    /// **Whether every test refuses the player**, bit `0x2` of its `+0x8bc4` (0x431e90).
    pub fn the_player_is_untouchable(&self) -> bool {
        self.space().read::<u32>(PLAYER.start + PLAYER_FLAGS) & PLAYER_IS_UNTOUCHABLE != 0
    }

    fn bullet_at(&self, slot: usize) -> usize {
        BULLETS.start + super::BULLETS_AT + slot * super::BULLET_BYTES
    }

    /// **A bullet in a slot of the manager's array**, as the spawn leaves one that is out at once: its flag for the
    /// test, active, where it is, its step a frame and its box — and its age from nought.
    pub fn a_bullet_is_at(&self, slot: usize, at: (f32, f32), step: (f32, f32), size: (f32, f32)) {
        let space = self.space();
        let bullet = self.bullet_at(slot);
        space.write::<u32>(bullet + super::BULLET_FLAGS, A_BULLET_MAY_HIT);
        space.write::<u16>(bullet + super::BULLET_STATE, A_BULLET_IS_ACTIVE);
        space.write::<[f32; 3]>(bullet + super::BULLET_POSITION, [at.0, at.1, 0.0]);
        space.write::<[f32; 3]>(bullet + super::BULLET_STEP, [step.0, step.1, 0.0]);
        space.write::<[f32; 2]>(bullet + super::BULLET_SIZE, [size.0, size.1]);
        space.write::<i32>(bullet + super::BULLET_AGE, 0);
    }

    /// **A bullet still coming out**, its state the one the spawn leaves a bullet in until its sprite's variable
    /// says it is out (0x4089be on).
    pub fn the_bullet_is_coming_out(&self, slot: usize) {
        self.space().write::<u16>(
            self.bullet_at(slot) + super::BULLET_STATE,
            A_BULLET_COMING_OUT,
        );
    }

    /// **A bullet that has been out for so many frames**, the age the manager's update steps.
    pub fn the_bullet_has_been_out_for(&self, slot: usize, frames: i32) {
        self.space()
            .write::<i32>(self.bullet_at(slot) + super::BULLET_AGE, frames);
    }

    /// **A bullet one frame on**, its step times the game's speed added where it is (0x408b4c through 0x408b97)
    /// and its age a frame more.
    pub fn the_bullet_moves(&self, slot: usize) {
        let space = self.space();
        let bullet = self.bullet_at(slot);
        let speed = space.read::<f32>(super::G_GAME_SPEED);
        let [x, y, z] = space.read::<[f32; 3]>(bullet + super::BULLET_POSITION);
        let [dx, dy, _] = space.read::<[f32; 3]>(bullet + super::BULLET_STEP);
        space.write::<[f32; 3]>(
            bullet + super::BULLET_POSITION,
            [x + dx * speed, y + dy * speed, z],
        );
        let age = space.read::<i32>(bullet + super::BULLET_AGE);
        space.write::<i32>(bullet + super::BULLET_AGE, age + 1);
    }

    /// Where a bullet is and its box, and whether its slot holds one that is active or cancelled.
    pub fn where_the_bullet_is(&self, slot: usize) -> ((f32, f32), (f32, f32)) {
        let space = self.space();
        let bullet = self.bullet_at(slot);
        let [x, y, _] = space.read::<[f32; 3]>(bullet + super::BULLET_POSITION);
        let [w, h] = space.read::<[f32; 2]>(bullet + super::BULLET_SIZE);
        ((x, y), (w, h))
    }

    pub fn the_bullet_is_active(&self, slot: usize) -> bool {
        self.space()
            .read::<u16>(self.bullet_at(slot) + super::BULLET_STATE)
            == A_BULLET_IS_ACTIVE
    }

    pub fn the_bullet_is_cancelled(&self, slot: usize) -> bool {
        self.space()
            .read::<u16>(self.bullet_at(slot) + super::BULLET_STATE)
            == A_BULLET_CANCELLED
    }

    /// **A bullet cancelled**, its state at 3 — what the box test answering 1 leaves of one that touched the player
    /// (0x408bda).
    pub fn the_bullet_is_cancelled_now(&self, slot: usize) {
        self.space().write::<u16>(
            self.bullet_at(slot) + super::BULLET_STATE,
            A_BULLET_CANCELLED,
        );
    }

    /// **A slot freed**, which is what the walk leaves of a bullet gone off the field.
    pub fn the_bullet_is_gone(&self, slot: usize) {
        self.space()
            .write::<u16>(self.bullet_at(slot) + super::BULLET_STATE, A_FREE_SLOT);
    }

    /// **The manager's count of the bullets its last walk left**, `+0x5c` (0x408d0b).
    pub fn the_bullet_manager_counts(&self, bullets: i32) {
        self.space()
            .write::<i32>(BULLETS.start + super::BULLETS_ALIVE, bullets);
    }

    fn enemy_at(place: usize) -> usize {
        ENEMIES.start + place * ENEMY_BYTES
    }

    fn the_fight(&self) -> usize {
        Self::enemy_at(THE_FIGHTS_PLACE)
    }

    /// **A fight up**: an enemy behind `main` on the list with its body, where it is and its life, and the enemy
    /// manager's boss slot holding it — which the `.ecl` writes (0x4114d4).
    pub fn a_fight_is_up(&self, at: (f32, f32), life: i32) {
        let space = self.space();
        let enemy = self.the_fight();
        space.fill_bytes(enemy, 0, ENEMY_BYTES);
        space.write::<u32>(enemy + super::ENEMY_FLAGS, A_BOSSS_FLAGS);
        space.write::<[f32; 3]>(enemy + super::ENEMY_POSITION, [at.0, at.1, 0.0]);
        space.write::<f32>(enemy + super::ENEMY_BODY, A_FIGHTS_BODY);
        space.write::<i32>(enemy + super::ENEMY_LIFE, life);
        space.write::<i32>(enemy + ENEMY_LIFE_AT_THE_START, life);
        self.onto_the_list(THE_FIGHTS_PLACE);
        space.write::<usize>(ENEMY_MANAGER.start + super::ENEMY_BOSS, enemy);
    }

    /// **The enemy laid out at `place` put on the end of the list**, its own element pointing at it, and the
    /// manager's count one on — which is what the spawn does with an enemy it has made (0x410ae1 through 0x410b06).
    fn onto_the_list(&self, place: usize) {
        let space = self.space();
        let enemy = Self::enemy_at(place);
        let element = enemy + ENEMY_ELEMENT;
        let manager = ENEMY_MANAGER.start;
        space.write::<usize>(element, enemy);
        space.write::<usize>(element + super::ELEMENT_NEXT, 0);
        let tail = space.read::<usize>(manager + ENEMY_LIST_TAIL);
        space.write::<usize>(element + ELEMENT_BEFORE, tail);
        match tail {
            0 => space.write::<usize>(manager + super::ENEMY_LIST, element),
            tail => space.write::<usize>(tail + super::ELEMENT_NEXT, element),
        }
        space.write::<usize>(manager + ENEMY_LIST_TAIL, element);
        let count = space.read::<i32>(manager + super::ENEMY_COUNT);
        space.write::<i32>(manager + super::ENEMY_COUNT, count + 1);
    }

    /// **And taken off it**, the count one back, where it is on it at all (0x410b60).
    fn off_the_list(&self, place: usize) {
        let space = self.space();
        let manager = ENEMY_MANAGER.start;
        let element = Self::enemy_at(place) + ENEMY_ELEMENT;
        let mut at = space.read::<usize>(manager + super::ENEMY_LIST);
        while at != 0 && at != element {
            at = space.read::<usize>(at + super::ELEMENT_NEXT);
        }
        if at == 0 {
            return;
        }
        let next = space.read::<usize>(element + super::ELEMENT_NEXT);
        let before = space.read::<usize>(element + ELEMENT_BEFORE);
        match before {
            0 => space.write::<usize>(manager + super::ENEMY_LIST, next),
            before => space.write::<usize>(before + super::ELEMENT_NEXT, next),
        }
        match next {
            0 => space.write::<usize>(manager + ENEMY_LIST_TAIL, before),
            next => space.write::<usize>(next + ELEMENT_BEFORE, before),
        }
        space.write::<usize>(element + super::ELEMENT_NEXT, 0);
        space.write::<usize>(element + ELEMENT_BEFORE, 0);
        let count = space.read::<i32>(manager + super::ENEMY_COUNT);
        space.write::<i32>(manager + super::ENEMY_COUNT, count - 1);
    }

    /// **A wave of `enemies` up, the wave before it gone**: each an enemy with a body on the list behind `main` and
    /// any fight, somewhere of its own in the field and a life, with the manager's count of them.
    ///
    /// # Panics
    /// Past the places this file lays out behind the fight's.
    pub fn a_wave_is_up(&self, enemies: usize) {
        let places = A_WAVES_PLACES;
        assert!(
            enemies <= places.len(),
            "a wave of {enemies} is more than the {} places laid out for one",
            places.len(),
        );
        for place in places.clone() {
            self.off_the_list(place);
        }
        let space = self.space();
        for (along, place) in places.take(enemies).enumerate() {
            let enemy = Self::enemy_at(place);
            space.fill_bytes(enemy, 0, ENEMY_BYTES);
            space.write::<u32>(enemy + super::ENEMY_FLAGS, A_WAVES_FLAGS);
            space.write::<[f32; 3]>(
                enemy + super::ENEMY_POSITION,
                [-96.0 + 32.0 * along as f32, 64.0, 0.0],
            );
            space.write::<f32>(enemy + super::ENEMY_BODY, A_WAVES_BODY);
            space.write::<i32>(enemy + super::ENEMY_LIFE, A_WAVES_LIFE);
            space.write::<i32>(enemy + ENEMY_LIFE_AT_THE_START, A_WAVES_LIFE);
            self.onto_the_list(place);
        }
    }

    /// **The wave's enemy `along` laid at `at`.**
    pub fn the_waves_enemy_is_at(&self, along: usize, at: (f32, f32)) {
        let enemy = Self::enemy_at(A_WAVES_PLACES.start + along);
        self.space()
            .write::<[f32; 3]>(enemy + super::ENEMY_POSITION, [at.0, at.1, 0.0]);
    }

    /// **The wave's enemy `along` shot down**, taken off the list.
    pub fn the_waves_enemy_is_shot_down(&self, along: usize) {
        self.off_the_list(A_WAVES_PLACES.start + along);
    }

    /// **The fight's next attack**, the frames of its attack back at nought as 0x417b40 puts them (0x417bf8 through
    /// 0x417c04).
    pub fn the_fight_attacks_again(&self) {
        let space = self.space();
        let enemy = self.the_fight();
        space.write::<i32>(enemy + super::ENEMY_ATTACK_FRAMES, 0);
        space.write::<f32>(enemy + ENEMY_ATTACK_FRAMES_AS_A_FLOAT, 0.0);
        space.write::<i32>(enemy + ENEMY_ATTACK_FRAMES_BEFORE, -1);
    }

    /// **Every enemy on the list a frame further into its attack**, the end of its update (0x412756 through
    /// 0x4127c5): the count kept as the one before, then one on where the game's speed is 1 and the float stepped by
    /// the speed and taken whole otherwise.
    pub fn the_enemies_attacks_go_on(&self) {
        let space = self.space();
        let speed = space.read::<f32>(super::G_GAME_SPEED);
        for enemy in self.the_enemies_on_the_list() {
            let frames = space.read::<i32>(enemy + super::ENEMY_ATTACK_FRAMES);
            space.write::<i32>(enemy + ENEMY_ATTACK_FRAMES_BEFORE, frames);
            let float = space.read::<f32>(enemy + ENEMY_ATTACK_FRAMES_AS_A_FLOAT);
            let (frames, float) = match speed == 1.0 {
                true => (frames + 1, float + 1.0),
                false => ((float + speed) as i32, float + speed),
            };
            space.write::<i32>(enemy + super::ENEMY_ATTACK_FRAMES, frames);
            space.write::<f32>(enemy + ENEMY_ATTACK_FRAMES_AS_A_FLOAT, float);
        }
    }

    /// **The fight over**: its enemy off the list and the boss's slot at nought.
    pub fn the_fight_is_over(&self) {
        self.off_the_list(THE_FIGHTS_PLACE);
        self.space()
            .write::<usize>(ENEMY_MANAGER.start + super::ENEMY_BOSS, 0);
    }

    pub fn a_fight_stands(&self) -> bool {
        self.space()
            .read::<usize>(ENEMY_MANAGER.start + super::ENEMY_BOSS)
            != 0
    }

    /// **A card up**, 0x40c650: its number, its timer from nought, its bonus, and its flags `& ~0x18 | 3`.
    pub fn a_card_is_up(&self, number: i32) {
        let space = self.space();
        space.write::<i32>(CARD.start + super::CARD_NUMBER, number);
        space.write::<i32>(CARD.start + super::CARD_TIMER, 0);
        let bonus = (space.read::<i32>(super::G_DIFFICULTY) + space.read::<i32>(super::G_STAGE))
            .max(0) as u32
            * A_CARDS_BONUS_STEP;
        space.write::<u32>(CARD.start + CARD_BONUS, bonus);
        let flags = space.read::<u32>(CARD.start + super::CARD_FLAGS);
        space.write::<u32>(
            CARD.start + super::CARD_FLAGS,
            flags & !A_CARD_TAKES_OFF | A_CARD_STARTS,
        );
    }

    /// **The card's calc job**, 0x40c630 into 0x40c0a0: its timer a frame on (0x40c147) while it runs, and nothing
    /// otherwise (0x40c0a9).
    pub fn the_card_runs_a_frame(&self) {
        let space = self.space();
        if space.read::<u32>(CARD.start + super::CARD_FLAGS) & A_CARD_RUNS != 0 {
            let timer = space.read::<i32>(CARD.start + super::CARD_TIMER);
            space.write::<i32>(CARD.start + super::CARD_TIMER, timer + 1);
        }
    }

    /// And down, its running bit off.
    pub fn no_card_is_up(&self) {
        let space = self.space();
        let flags = space.read::<u32>(CARD.start + super::CARD_FLAGS);
        space.write::<u32>(CARD.start + super::CARD_FLAGS, flags & !A_CARD_RUNS);
    }

    /// **A conversation running**, the object the panel keeps at `+0x4438` while one is (0x41d2a0).
    pub fn a_conversation_is_up(&self) {
        self.space().write::<usize>(
            PANEL.start + super::PANEL_CONVERSATION,
            A_CONVERSATION.start,
        );
    }

    /// And over, the panel's word at nought.
    pub fn no_conversation_is_up(&self) {
        self.space()
            .write::<usize>(PANEL.start + super::PANEL_CONVERSATION, 0);
    }

    pub fn a_conversation_stands(&self) -> bool {
        self.space()
            .read::<usize>(PANEL.start + super::PANEL_CONVERSATION)
            != 0
    }

    /// **The panel's flag `0x10`**, which a stage 6 won sets (0x41ecb0) and which holds the shot's step back
    /// (0x431b0b through 0x431b15).
    pub fn the_panel_holds_the_shot(&self) {
        let space = self.space();
        let flags = space.read::<u32>(PANEL.start + super::PANEL_FLAGS);
        space.write::<u32>(
            PANEL.start + super::PANEL_FLAGS,
            flags | super::PANEL_HOLDS_THE_SHOT,
        );
    }

    pub fn the_panel_stands_holding_the_shot(&self) -> bool {
        self.space().read::<u32>(PANEL.start + super::PANEL_FLAGS) & super::PANEL_HOLDS_THE_SHOT
            != 0
    }

    /// **The build over**, as its last acts leave it: the game object's flag for a build taken off (0x41fdbe) and
    /// the run's flags a build clears (0x41fdc2).
    pub fn the_build_is_over(&self) {
        let space = self.space();
        let flags = space.read::<u32>(GAME.start + super::GAME_FLAGS);
        space.write::<u32>(GAME.start + super::GAME_FLAGS, flags & !GAME_IS_BEING_BUILT);
        self.the_runs_flags_are(0, THE_BUILD_CLEARS);
    }

    /// **The stage begun**, the stage's first frame (0x420480 on): the player reset — its state playing, its shot's
    /// timer at -1, its state's and its update's timers at nought, its flags `& ~7` (0x42fec0) — the items zeroed,
    /// every enemy destroyed and `main` spawned onto an emptied list, and both of the stage's frame counters at
    /// nought.
    pub fn the_stage_begins(&self) {
        let space = self.space();
        self.the_player_is_in(Player::Playing);
        self.the_shot_timer_is(-1);
        space.write::<i32>(PLAYER.start + super::PLAYER_UPDATE_TIMER, 0);
        let flags = space.read::<u32>(PLAYER.start + PLAYER_FLAGS);
        space.write::<u32>(PLAYER.start + PLAYER_FLAGS, flags & !7);
        space.fill_bytes(ITEMS.start + super::ITEMS_AT, 0, THE_ITEMS_A_STAGE_ZEROES);
        space.fill_bytes(ENEMIES.start, 0, ENEMIES.len());
        let manager = ENEMY_MANAGER.start;
        space.write::<usize>(manager + super::ENEMY_LIST, 0);
        space.write::<usize>(manager + ENEMY_LIST_TAIL, 0);
        space.write::<i32>(manager + super::ENEMY_COUNT, 0);
        space.write::<usize>(manager + super::ENEMY_BOSS, 0);
        space.write::<u32>(super::G_STAGE_FRAMES, 0);
        space.write::<u32>(G_FRAMES_SINCE_A_CONVERSATION, 0);
        let main = Self::enemy_at(0);
        space.write::<u32>(main + super::ENEMY_FLAGS, MAINS_FLAGS);
        space.write::<f32>(main + super::ENEMY_SCRIPT_TIME, 0.0);
        self.onto_the_list(0);
    }

    /// **One frame of the stage**, the game object's calc past its pause: both of the stage's frame counters
    /// (0x4207ee, 0x4207f4) and the game object's own timer (0x4207fd) each one on.
    pub fn a_frame_of_the_stage_goes_by(&self) {
        let space = self.space();
        let frames = space.read::<u32>(super::G_STAGE_FRAMES);
        space.write::<u32>(super::G_STAGE_FRAMES, frames + 1);
        let since = space.read::<u32>(G_FRAMES_SINCE_A_CONVERSATION);
        space.write::<u32>(G_FRAMES_SINCE_A_CONVERSATION, since + 1);
        let before = space.read::<i32>(GAME.start + super::GAME_FRAMES);
        space.write::<i32>(GAME.start + GAME_TIMER_BEFORE, before);
        space.write::<i32>(GAME.start + super::GAME_FRAMES, before + 1);
        space.write::<f32>(GAME.start + GAME_TIMER_AS_A_FLOAT, (before + 1) as f32);
    }

    pub fn the_stages_frames(&self) -> u32 {
        self.space().read::<u32>(super::G_STAGE_FRAMES)
    }

    /// **Every live enemy's scripts one step on**, the game's speed added to the time (0x411aa1 through 0x411abb).
    ///
    /// **Except the stage's own scripts while they wait for a boss**: instruction 340 holds a script where it is while
    /// the boss slot is set (0x415b1e through 0x415b2d) — and each of them is left standing on it then, and on an
    /// instruction that waits for nothing otherwise.
    pub fn the_enemies_scripts_step(&self, the_stage_waits_for_its_boss: bool) {
        let space = self.space();
        let speed = space.read::<f32>(super::G_GAME_SPEED);
        for enemy in self.the_enemies_on_the_list() {
            if space.read::<u32>(enemy + super::ENEMY_FLAGS) & super::ENEMY_NO_BODY != 0 {
                let stands_on = match the_stage_waits_for_its_boss {
                    true => SCRIPTS_STAND_ON.start,
                    false => SCRIPTS_STAND_ON.start + AN_INSTRUCTION,
                };
                space.write::<usize>(enemy + super::ENEMY_SCRIPT_AT, stands_on);
                if the_stage_waits_for_its_boss {
                    continue;
                }
            }
            let time = space.read::<f32>(enemy + super::ENEMY_SCRIPT_TIME);
            space.write::<f32>(enemy + super::ENEMY_SCRIPT_TIME, time + speed);
        }
    }

    /// The enemies on the list, in its order.
    pub fn the_enemies_on_the_list(&self) -> Vec<usize> {
        let space = self.space();
        let mut enemies = Vec::new();
        let mut element = space.read::<usize>(ENEMY_MANAGER.start + super::ENEMY_LIST);
        while element != 0 && enemies.len() < ENEMIES_LAID_OUT {
            enemies.push(space.read::<usize>(element));
            element = space.read::<usize>(element + super::ELEMENT_NEXT);
        }
        enemies
    }

    /// **Whether an enemy is up**, the manager there and its count not nought — what the shot and the focus ask
    /// (0x431af9 through 0x431b15).
    pub fn an_enemy_is_up(&self) -> bool {
        let space = self.space();
        space.read::<usize>(super::G_ENEMY_MANAGER) != 0
            && space.read::<i32>(ENEMY_MANAGER.start + super::ENEMY_COUNT) != 0
    }

    /// **The body an enemy is tested against the player with**, its position and its radius, half its body's size —
    /// `None` where its flags `0x22` skip the test or the frames its body is off for have not run out (0x411df9
    /// through 0x411e48).
    pub fn the_body_of(&self, enemy: usize) -> Option<((f32, f32), f32)> {
        let space = self.space();
        if space.read::<u32>(enemy + super::ENEMY_FLAGS) & super::ENEMY_NO_BODY != 0
            || space.read::<i32>(enemy + super::ENEMY_BODY_OFF_FOR) > 0
        {
            return None;
        }
        let [x, y, _] = space.read::<[f32; 3]>(enemy + super::ENEMY_POSITION);
        let body = space.read::<f32>(enemy + super::ENEMY_BODY);
        Some(((x, y), body * 0.5))
    }

    /// And an enemy's flags.
    pub fn the_flags_of(&self, enemy: usize) -> u32 {
        self.space().read::<u32>(enemy + super::ENEMY_FLAGS)
    }

    /// **The game object's pause flag**, as the pause object's openers and its resume write it.
    pub fn the_game_is_paused(&self, paused: bool) {
        let space = self.space();
        let flags = space.read::<u32>(GAME.start + super::GAME_FLAGS);
        let flags = match paused {
            true => flags | GAME_IS_PAUSED,
            false => flags & !GAME_IS_PAUSED,
        };
        space.write::<u32>(GAME.start + super::GAME_FLAGS, flags);
    }

    pub fn the_game_stands_paused(&self) -> bool {
        self.the_games_flags() & GAME_IS_PAUSED != 0
    }

    /// **The pause object in a state**, its timer started over as every change of state starts it.
    pub fn the_pause_is_in(&self, state: Pause) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + super::PAUSE_STATE, state as i32);
        space.write::<i32>(PAUSE.start + super::PAUSE_FRAMES - 4, -1);
        space.write::<i32>(PAUSE.start + super::PAUSE_FRAMES, 0);
        space.write::<f32>(PAUSE.start + super::PAUSE_FRAMES + 4, 0.0);
    }

    /// And its state, its timer and its cursor read back.
    pub fn where_the_pause_is(&self) -> (i32, i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(PAUSE.start + super::PAUSE_STATE),
            space.read::<i32>(PAUSE.start + super::PAUSE_FRAMES),
            space.read::<i32>(PAUSE.start + super::PAUSE_CURSOR),
        )
    }

    /// One more frame of its state, the timer every state reads.
    pub fn the_pause_counts_a_frame(&self) {
        let space = self.space();
        let frames = space.read::<i32>(PAUSE.start + super::PAUSE_FRAMES);
        space.write::<i32>(PAUSE.start + super::PAUSE_FRAMES - 4, frames);
        space.write::<i32>(PAUSE.start + super::PAUSE_FRAMES, frames + 1);
        space.write::<f32>(PAUSE.start + super::PAUSE_FRAMES + 4, (frames + 1) as f32);
    }

    pub fn the_pauses_cursor_is(&self, cursor: i32, items: i32) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + super::PAUSE_CURSOR, cursor);
        space.write::<i32>(PAUSE.start + PAUSE_ITEMS, items);
    }

    pub fn the_pauses_items(&self) -> i32 {
        self.space().read::<i32>(PAUSE.start + PAUSE_ITEMS)
    }

    /// **An item the pause menu leaves out**, put on its list at `+0xc8` (0x42c91f, 0x42c957), and the list emptied.
    pub fn the_pause_leaves_out(&self, item: i32) {
        let space = self.space();
        let count = space.read::<i32>(PAUSE.start + PAUSE_HOW_MANY_LEFT_OUT);
        space.write::<i32>(
            PAUSE.start + PAUSE_LEFT_OUT + count.max(0) as usize * 4,
            item,
        );
        space.write::<i32>(PAUSE.start + PAUSE_HOW_MANY_LEFT_OUT, count + 1);
    }

    pub fn the_pause_leaves_nothing_out(&self) {
        self.space()
            .write::<i32>(PAUSE.start + PAUSE_HOW_MANY_LEFT_OUT, 0);
    }

    pub fn the_items_the_pause_leaves_out(&self) -> Vec<i32> {
        let space = self.space();
        let room = (PAUSE_HOW_MANY_LEFT_OUT - PAUSE_LEFT_OUT) / 4;
        let count = space.read::<i32>(PAUSE.start + PAUSE_HOW_MANY_LEFT_OUT);
        (0..(count.max(0) as usize).min(room))
            .map(|at| space.read::<i32>(PAUSE.start + PAUSE_LEFT_OUT + at * 4))
            .collect()
    }

    /// **Whether the pause is a replay's own** — see [`PAUSE_A_REPLAYS_OWN`].
    pub fn the_pause_is_a_replays_own(&self) -> bool {
        self.space().read::<i32>(PAUSE.start + PAUSE_A_REPLAYS_OWN) != 0
    }

    /// **The game's speed held back as the pause opens and put back by its resume**: kept in `+0x2d8` and 1.0 put in
    /// its place.
    pub fn the_pause_holds_the_speed_back(&self) {
        let space = self.space();
        let speed = space.read::<u32>(super::G_GAME_SPEED);
        space.write::<u32>(PAUSE.start + PAUSE_SPEED_HELD, speed);
        space.write::<f32>(super::G_GAME_SPEED, THE_GAMES_OWN_SPEED);
    }

    pub fn the_pause_puts_the_speed_back(&self) {
        let space = self.space();
        let speed = space.read::<u32>(PAUSE.start + PAUSE_SPEED_HELD);
        space.write::<u32>(super::G_GAME_SPEED, speed);
    }

    /// **A run over by a game over**, 0x42d560: the pause object on the lost run's tally — or the won run's for a
    /// practice (`lea edx,[edx*8+0xd]` at 0x42d5b0) — with the run not cleared, the game object paused and the speed
    /// held back.
    pub fn the_run_is_over_with_a_game_over(&self) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + PAUSE_THE_RUN_WAS_CLEARED, 0);
        let state = match self.the_runs_flags() & RUN_IS_A_PRACTICE {
            0 => Pause::ALostRunTallies,
            _ => Pause::AWonRunTallies,
        };
        self.the_pause_is_in(state);
        self.the_game_is_paused(true);
        self.the_pause_holds_the_speed_back();
    }

    /// **A run over by its clear**, 0x42d6b0 — a practice's stage won, or the Extra's: the won run's tally with the
    /// run cleared, the game object paused and the speed held back.
    pub fn the_run_is_over_cleared(&self) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + PAUSE_THE_RUN_WAS_CLEARED, 1);
        self.the_pause_is_in(Pause::AWonRunTallies);
        self.the_game_is_paused(true);
        self.the_pause_holds_the_speed_back();
    }

    /// **The score into the high score**, 0x42ba20 (0x42ba33).
    pub fn the_high_score_is_raised(&self) {
        let space = self.space();
        let score = space.read::<u32>(super::G_SCORE);
        if space.read::<u32>(G_HIGH_SCORE) < score {
            space.write::<u32>(G_HIGH_SCORE, score);
        }
    }

    /// **The tally's placing of the run's score**, 0x42d850 past its Extra's stage words: out of the ranking where it
    /// ranks in none of the ten (`+0x1f8` at 1), and where it ranks, the place under the pause's cursor, the grid's
    /// length, and the name started from the last one typed with its trailing spaces off the count.
    pub fn the_pause_places_the_run(&self, grid: i32) {
        let space = self.space();
        let Some(place) = self.the_ranking_places_the_run() else {
            space.write::<i32>(PAUSE.start + PAUSE_OUT_OF_THE_RANKING, 1);
            return;
        };
        let items = self.the_pauses_items();
        self.the_pauses_cursor_is(place as i32, items);
        space.write::<i32>(PAUSE.start + PAUSE_NAME_GRID_LENGTH, grid);
        let name = self.the_last_name_typed();
        self.the_pauses_entry_is(&TheTitlesEntry {
            name,
            typed: name.iter().rposition(|c| *c != b' ').map_or(0, |at| at + 1) as i32,
            not_asked: false,
            grid: if name == A_BLANK_NAME { 0 } else { grid - 1 },
        });
        space.write::<i32>(PAUSE.start + PAUSE_OUT_OF_THE_RANKING, 0);
    }

    /// Whether the tally found the run's score no place in the ranking.
    pub fn the_run_is_out_of_the_ranking(&self) -> bool {
        self.space()
            .read::<i32>(PAUSE.start + PAUSE_OUT_OF_THE_RANKING)
            != 0
    }

    /// **What the pause object's name entry holds**: the name at `+0x2cc`, how many are typed at `+0x1f0` and the
    /// grid's place at `+0x110`. `not_asked` is the title's field and none of the pause's: it reads false.
    pub fn the_pauses_entry(&self) -> TheTitlesEntry {
        let space = self.space();
        let mut name = [0; 8];
        name.copy_from_slice(&space.read_bytes(PAUSE.start + super::PAUSE_NAME, 8));
        TheTitlesEntry {
            name,
            typed: space.read::<i32>(PAUSE.start + super::PAUSE_NAME_TYPED),
            not_asked: false,
            grid: space.read::<i32>(PAUSE.start + super::PAUSE_NAME_GRID),
        }
    }

    pub fn the_pauses_entry_is(&self, entry: &TheTitlesEntry) {
        let space = self.space();
        space.write_bytes(PAUSE.start + super::PAUSE_NAME, &entry.name);
        space.write::<u8>(PAUSE.start + super::PAUSE_NAME + 8, 0);
        space.write::<i32>(PAUSE.start + super::PAUSE_NAME_TYPED, entry.typed);
        space.write::<i32>(PAUSE.start + super::PAUSE_NAME_GRID, entry.grid);
    }

    /// Whether the run the end is for was cleared — see [`PAUSE_THE_RUN_WAS_CLEARED`].
    pub fn the_run_was_cleared(&self) -> bool {
        self.space()
            .read::<i32>(PAUSE.start + PAUSE_THE_RUN_WAS_CLEARED)
            != 0
    }

    /// **The sound's command queued**, 0x44a9c0: into the first slot of the queue that holds none.
    pub fn a_sound_command_is_queued(&self, command: u32) {
        self.a_sound_command_is_queued_with(command, 0, b"");
    }

    /// And one with its argument — the slot a play is of — its name, and its step at nought.
    pub fn a_sound_command_is_queued_with(&self, command: u32, argument: u32, name: &[u8]) {
        let space = self.space();
        let queue = super::G_SOUND_MANAGER + COMMANDS;
        if let Some(slot) = (0..COMMAND_SLOTS)
            .map(|slot| queue + slot * COMMAND_BYTES)
            .find(|at| space.read::<u32>(*at) == 0)
        {
            space.write::<u32>(slot, command);
            space.write::<u32>(slot + COMMAND_ARGUMENT, argument);
            space.write::<u32>(slot + COMMAND_STEP, 0);
            space.fill_bytes(slot + COMMAND_NAME, 0, COMMAND_NAME_BYTES);
            space.write_bytes(
                slot + COMMAND_NAME,
                &name[..name.len().min(COMMAND_NAME_BYTES - 1)],
            );
        }
    }

    /// **The head carried out**, 0x44a340 once: nothing where there is no DirectSound (0x44a365); otherwise the head's
    /// step, and the head shifted off where its command is done (0x44a622 through 0x44a651) — **command 1 letting the
    /// next one run in the same call** (0x44a402, 0x44a655). A play's step 2 puts the stream's source on the record
    /// its slot's path names (0x44a690 through 0x44a6c3), which is what this answers: that record's place among the
    /// records, the first where none matches (0x4496b0).
    pub fn the_head_of_the_sound_queue_is_carried_out(&self) -> Option<usize> {
        let space = self.space();
        if space.read::<usize>(super::G_SOUND_MANAGER + super::SOUND_DEVICE) == 0 {
            return None;
        }
        let queue = super::G_SOUND_MANAGER + COMMANDS;
        let mut heard = None;
        loop {
            let head = space.read::<u32>(queue);
            if head == 0 {
                return heard;
            }
            let step = space.read::<u32>(queue + COMMAND_STEP);
            let argument = space.read::<u32>(queue + COMMAND_ARGUMENT);
            let done = match head {
                A_SONGS_PATH => {
                    let name = space.read_bytes(queue + COMMAND_NAME, COMMAND_NAME_BYTES);
                    let at = super::G_SOUND_MANAGER
                        + super::SOUND_SONGS
                        + argument as usize * super::SONG_PATH_BYTES;
                    let path: Vec<u8> = name.into_iter().take_while(|byte| *byte != 0).collect();
                    space.fill_bytes(at, 0, super::SONG_PATH_BYTES);
                    space.write_bytes(at, &path[..path.len().min(super::SONG_PATH_BYTES - 1)]);
                    true
                }
                A_PLAY => {
                    if step == A_PLAY_PUTS_THE_SOURCE_ON_AT {
                        heard = Some(self.the_record_a_slot_names(argument as usize));
                    }
                    step >= A_PLAY_IS_DONE_AT
                }
                A_STOP => step >= A_STOP_IS_DONE_AT,
                A_CLOSE => step >= A_CLOSE_IS_DONE_AT,
                _ => true,
            };
            if !done {
                space.write::<u32>(queue + COMMAND_STEP, step + 1);
                return heard;
            }
            let rest = space.read_bytes(queue + COMMAND_BYTES, (COMMAND_SLOTS - 1) * COMMAND_BYTES);
            space.write_bytes(queue, &rest);
            space.fill_bytes(
                queue + (COMMAND_SLOTS - 1) * COMMAND_BYTES,
                0,
                COMMAND_BYTES,
            );
            if head != A_SONGS_PATH {
                return heard;
            }
        }
    }

    /// **The record of `thbgm.fmt` a slot's path names**, by its file name past its last `/` against each record's
    /// own, and the first where none matches (0x4496b0).
    fn the_record_a_slot_names(&self, slot: usize) -> usize {
        let space = self.space();
        let at = super::G_SOUND_MANAGER + super::SOUND_SONGS + slot * super::SONG_PATH_BYTES;
        let path: Vec<u8> = space
            .read_bytes(at, super::SONG_PATH_BYTES)
            .into_iter()
            .take_while(|byte| *byte != 0)
            .collect();
        let name = match path
            .iter()
            .rposition(|byte| *byte == b'/' || *byte == b'\\')
        {
            Some(cut) => &path[cut + 1..],
            None => &path[..],
        };
        TRACK_NAMES
            .iter()
            .position(|track| track.to_bytes() == name)
            .unwrap_or(0)
    }

    /// **The sound up, as the game's start-up leaves it**: DirectSound's wrapper (0x4491a6), and the one stream a
    /// session opens over `thbgm.dat` (0x449b30) — its buffer's holder, its source and the source on the first of
    /// `thbgm.fmt`'s records. Answers the four ranges beside the manager's own.
    pub fn the_sound_is_up(&self) -> [Range<usize>; 4] {
        let space = self.space();
        let manager = super::G_SOUND_MANAGER;
        space.write::<usize>(manager + super::SOUND_DEVICE, SOUND_DEVICE.start);
        space.write::<usize>(manager + super::MUSIC_STREAM, STREAM.start);
        space.write::<usize>(STREAM.start + super::STREAM_BUFFER, STREAM_BUFFER.start);
        space.write::<usize>(STREAM.start + super::STREAM_SOURCE, SOURCE.start);
        space.write::<usize>(manager + SOUND_TRACKS, TRACKS.start);
        // **The records' own numbers are this file's**, distinct for each: where in `thbgm.dat` a track is and
        // how long, which is what `Th11::music_identity` tells two apart by.
        for (track, name) in TRACK_NAMES.iter().enumerate() {
            let record = TRACKS.start + track * TRACK_RECORD_BYTES;
            space.fill_bytes(record + super::RECORD_NAME, 0, super::RECORD_NAME_BYTES);
            space.write_bytes(record + super::RECORD_NAME, name.to_bytes());
            space.write::<u32>(record + super::RECORD_AT, 0x0010_0000 * (track as u32 + 1));
            space.write::<u32>(record + super::RECORD_BYTES, 0x0008_0000 + track as u32);
        }
        self.the_stream_plays_the_track(0);
        [
            SOUND_DEVICE.clone(),
            STREAM.clone(),
            STREAM_BUFFER.clone(),
            SOURCE.clone(),
        ]
    }

    /// **The path of the song a slot is to hold**, as 0x42a0d0 makes it: `bgm/` and the record's name — which goes
    /// into the command it queues, and into the slot once that command is carried out.
    pub fn the_path_of_the_track(&self, track: usize) -> Vec<u8> {
        [b"bgm/".as_slice(), TRACK_NAMES[track].to_bytes()].concat()
    }

    /// **The stage's two songs as the slots hold them once loaded**, for an e2e test that is not about the queue the
    /// build loads them through — see [`Image::the_path_of_the_track`].
    pub fn the_stages_songs_are_loaded(&self, stage: usize, fight: usize) {
        let space = self.space();
        for (slot, track) in [stage, fight].into_iter().enumerate() {
            let at = super::G_SOUND_MANAGER + super::SOUND_SONGS + slot * super::SONG_PATH_BYTES;
            space.fill_bytes(at, 0, super::SONG_PATH_BYTES);
            space.write_bytes(at, &self.the_path_of_the_track(track));
        }
    }

    /// **The source put on a track**, the record's address written into the source's `+0x90` (0x45b31d).
    pub fn the_stream_plays_the_track(&self, track: usize) {
        self.space().write::<usize>(
            SOURCE.start + super::SOURCE_RECORD,
            TRACKS.start + track * TRACK_RECORD_BYTES,
        );
    }

    /// And the track the source is on, by its place among the records — `None` where it is on none of them.
    pub fn the_track_the_stream_is_on(&self) -> Option<usize> {
        let record = self
            .space()
            .read::<usize>(SOURCE.start + super::SOURCE_RECORD);
        record
            .checked_sub(TRACKS.start)
            .filter(|at| at % TRACK_RECORD_BYTES == 0)
            .map(|at| at / TRACK_RECORD_BYTES)
            .filter(|track| *track < TRACK_NAMES.len())
    }

    /// Whether the sound's queue holds a command at its head.
    pub fn the_sound_queue_holds_a_command(&self) -> bool {
        self.space().read::<u32>(super::G_SOUND_MANAGER + COMMANDS) != 0
    }

    /// **And none of it**, which is the manager as `0x448ff0` zeroes it — the launch before its sound start-up has run.
    pub fn the_sound_is_not_up(&self) {
        let space = self.space();
        space.write::<usize>(super::G_SOUND_MANAGER + super::SOUND_DEVICE, 0);
        space.write::<usize>(super::G_SOUND_MANAGER + super::MUSIC_STREAM, 0);
    }

    /// **The manager's own range** — see [`SOUND_MANAGER`].
    pub fn the_sound_manager_is(&self) -> Range<usize> {
        SOUND_MANAGER
    }

    /// The music's thread, whose id the stream's opening writes (0x449b9f).
    pub fn the_music_is_streamed_on_thread(&self, thread: u32) {
        self.space()
            .write::<u32>(super::G_SOUND_MANAGER + super::SOUND_MUSIC_THREAD, thread);
    }

    /// **The supervisor's locks**, which a snapshot leaves alone — see [`LOCKS`].
    pub fn the_supervisors_locks(&self) -> Range<usize> {
        LOCKS
    }

    /// **The supervisor's three surfaces**, which a snapshot leaves alone — see [`SURFACES`].
    pub fn the_supervisors_surfaces(&self) -> Range<usize> {
        SURFACES
    }

    /// **An `.anm` file in its slot**, with that many textures and that many sprites: the file stored in the manager's
    /// slot (0x45423d), its counts and its two arrays, each texture's `IDirect3DTexture9` first in its entry and each
    /// sprite holding at `+0x8` the address of its texture's entry (0x454787 through 0x454797). Answers the two ranges
    /// a restore has to leave alone.
    pub fn an_anm_file_is_loaded(
        &self,
        slot: usize,
        textures: usize,
        sprites: usize,
    ) -> [Range<usize>; 2] {
        let space = self.space();
        let file = ANM_FILES.start + slot * A_FILES_OWN_SPACE;
        let textures_at = file + TEXTURES_IN_THE_PAGE;
        let sprites_at = file + SPRITES_IN_THE_PAGE;
        space.write::<usize>(self.sprites() + super::ANM_FILES_AT + slot * 4, file);
        space.write::<u32>(file, slot as u32);
        space.write::<u32>(file + super::ANM_TEXTURES, textures as u32);
        space.write::<usize>(file + super::ANM_TEXTURES_AT, textures_at);
        space.write::<u32>(file + super::ANM_SPRITES, sprites as u32);
        space.write::<usize>(file + super::ANM_SPRITES_AT, sprites_at);
        for texture in 0..textures {
            space.write::<u32>(
                textures_at + texture * super::ANM_TEXTURE_BYTES,
                A_TEXTURE + texture as u32,
            );
        }
        for sprite in 0..sprites {
            let entry = sprites_at + sprite * super::ANM_SPRITE_BYTES;
            space.write::<u32>(entry, slot as u32);
            space.write::<u32>(entry + 4, sprite as u32);
            space.write::<usize>(
                entry + 8,
                textures_at + (sprite % textures.max(1)) * super::ANM_TEXTURE_BYTES,
            );
        }
        [
            textures_at..textures_at + textures * super::ANM_TEXTURE_BYTES,
            sprites_at..sprites_at + sprites * super::ANM_SPRITE_BYTES,
        ]
    }

    /// And a slot with nothing in it, which is what all but a handful of the 32 are.
    pub fn an_anm_slot_is_empty(&self, slot: usize) {
        self.space()
            .write::<usize>(self.sprites() + super::ANM_FILES_AT + slot * 4, 0);
    }

    /// And the sprite manager not made yet, which is a launch before WinMain's 0x445762.
    pub fn the_sprite_manager_is_not_made(&self) {
        self.space().write::<usize>(super::G_SPRITES, 0);
    }

    pub fn the_sprite_manager_is_made(&self) {
        self.space().write::<usize>(super::G_SPRITES, SPRITES.start);
    }

    /// **The run's numbers**, as the stage the run is in holds them.
    pub fn the_run_has(&self, score: u32, power: i32, lives: i32) {
        let space = self.space();
        space.write::<u32>(super::G_SCORE, score);
        space.write::<i32>(super::G_POWER, power);
        space.write::<i32>(super::G_LIVES, lives);
    }

    /// **A bomb let go**, as the player's own bomb lets one go: the bomb's `+0x3c` raised, which is what it asks to be
    /// nought first (0x40651b), and the power one bomb costs taken off (0x4311cd through 0x4311f1).
    pub fn a_bomb_is_going_off(&self) {
        let space = self.space();
        let power = space.read::<i32>(super::G_POWER);
        let costs = space.read::<i32>(super::G_BOMB_COSTS);
        space.write::<i32>(super::G_POWER, (power - costs).max(0));
        space.write::<i32>(BOMB.start + super::BOMB_GOING_OFF, 1);
    }

    /// And the bomb over, that word lowered.
    pub fn no_bomb_is_going_off(&self) {
        self.space()
            .write::<i32>(BOMB.start + super::BOMB_GOING_OFF, 0);
    }

    pub fn the_runs_numbers(&self) -> (u32, i32, i32) {
        let space = self.space();
        (
            space.read::<u32>(super::G_SCORE),
            space.read::<i32>(super::G_POWER),
            space.read::<i32>(super::G_LIVES),
        )
    }

    /// **The rest of what a stage's replay record keeps of the run** — see `resume::Numbers`: the point items'
    /// value, the life pieces, the word at `0x4a5744`, the continues, the extends and the graze.
    #[allow(clippy::too_many_arguments)]
    pub fn the_run_also_has(
        &self,
        point_value: i32,
        life_pieces: i32,
        the_number_at_4a5744: i32,
        continues: i32,
        extends: i32,
        graze: u32,
    ) {
        let space = self.space();
        space.write::<i32>(super::G_POINT_VALUE, point_value);
        space.write::<i32>(super::G_LIFE_PIECES, life_pieces);
        space.write::<i32>(super::G_4A5744, the_number_at_4a5744);
        space.write::<i32>(super::G_CONTINUES, continues);
        space.write::<i32>(super::G_EXTENDS, extends);
        space.write::<u32>(super::G_GRAZE, graze);
    }

    /// The point items' value and the number at `0x4a5744` the run has now.
    pub fn the_point_value_and_the_number_at_4a5744(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(super::G_POINT_VALUE),
            space.read::<i32>(super::G_4A5744),
        )
    }

    /// The continues the run has used and the extends its score has given now.
    pub fn the_continues_and_the_extends(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(super::G_CONTINUES),
            space.read::<i32>(super::G_EXTENDS),
        )
    }

    /// The generator's word, with its draws counted from nought, as a playback's seed leaves it (0x435a8d through
    /// 0x435a98).
    pub fn the_generator_is_seeded_with(&self, seed: u16) {
        let space = self.space();
        space.write::<u16>(super::G_RANDOM, seed);
        space.write::<u32>(super::G_RANDOM_DRAWS, 0);
    }

    pub fn the_generators_word(&self) -> u16 {
        self.space().read::<u16>(super::G_RANDOM)
    }

    /// **The generator's draws counted from nought again**, which the replay's recording does as a stage is built —
    /// a fresh one's (0x43582a) and the stage's after one won (0x436f8b) — the word left as it stands.
    pub fn the_generators_draws_start_over(&self) {
        self.space().write::<u32>(super::G_RANDOM_DRAWS, 0);
    }

    /// **One draw of the generator**, 0x458bc0: the count of draws one on, then the word `xor`ed with `0x9630`,
    /// `0x6553` taken off and turned two to the left (0x458bdd through 0x458c01).
    pub fn the_generator_draws(&self) -> u16 {
        let space = self.space();
        let draws = space.read::<u32>(super::G_RANDOM_DRAWS);
        space.write::<u32>(super::G_RANDOM_DRAWS, draws.wrapping_add(1));
        let word = space.read::<u16>(super::G_RANDOM);
        let word = (word ^ 0x9630).wrapping_sub(0x6553).rotate_left(2);
        space.write::<u16>(super::G_RANDOM, word);
        word
    }

    /// **Where the player is**, in the 128ths it is kept in and the floats made of them — each times the `0.0078125`
    /// at `0x497ed0` (0x430859 through 0x430870).
    pub fn the_player_is_at_128ths(&self, x: i32, y: i32) {
        let space = self.space();
        space.write::<i32>(PLAYER.start + super::PLAYER_128THS, x);
        space.write::<i32>(PLAYER.start + super::PLAYER_128THS + 4, y);
        space.write::<f32>(PLAYER.start + super::PLAYER_POSITION, x as f32 * A_128TH);
        space.write::<f32>(
            PLAYER.start + super::PLAYER_POSITION + 4,
            y as f32 * A_128TH,
        );
    }

    pub fn where_the_player_is_in_128ths(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(PLAYER.start + super::PLAYER_128THS),
            space.read::<i32>(PLAYER.start + super::PLAYER_128THS + 4),
        )
    }

    pub fn where_the_player_is(&self) -> (f32, f32) {
        let space = self.space();
        (
            space.read::<f32>(PLAYER.start + super::PLAYER_POSITION),
            space.read::<f32>(PLAYER.start + super::PLAYER_POSITION + 4),
        )
    }

    /// **The element Patchouli's options are on**, `+0x8bac`, which shot type 4's `0x400` gone down steps on modulo 5
    /// (0x4308fe through 0x430921).
    pub fn the_players_element_is(&self, element: i32) {
        self.space()
            .write::<i32>(PLAYER.start + super::PLAYER_ELEMENT, element);
    }

    pub fn the_players_element(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_ELEMENT)
    }

    /// **The options made for the power**, what 0x432cc0 leaves in memory of the options: the count, the power over a
    /// bomb's cost (0x432cee, 0x43361d), and 1 in each of the eight's `+0xd4` (0x433623).
    pub fn the_options_are_made(&self) {
        let space = self.space();
        let power = space.read::<i32>(super::G_POWER);
        let costs = space.read::<i32>(super::G_BOMB_COSTS);
        let count = match costs {
            1.. => power / costs,
            _ => 0,
        };
        space.write::<i32>(PLAYER.start + super::PLAYER_OPTION_COUNT, count);
        for option in 0..PLAYER_OPTIONS_LAID_OUT {
            space.write::<u32>(
                PLAYER.start
                    + PLAYER_OPTIONS
                    + option * PLAYER_OPTION_BYTES
                    + OPTION_PUT_ON_ITS_PLACE,
                1,
            );
        }
    }

    /// How many options the player has, `+0x7c90`.
    pub fn the_players_options(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_OPTION_COUNT)
    }

    /// **The shot's step, what 0x434380 leaves of the player's shot timer**: nothing outside state 1; held at -1
    /// until the run's word holds the shot, started on the frame it does; past 13 taken back 14 while the shot is held
    /// and put at -1 where it is not, and one on otherwise. The shots it fires at each count are not laid out —
    /// nothing of orb's reads them.
    pub fn the_shot_steps(&self) {
        let space = self.space();
        if space.read::<i32>(PLAYER.start + super::PLAYER_STATE) != Player::Playing as i32 {
            return;
        }
        let at = PLAYER.start + super::PLAYER_SHOT_TIMER;
        let held = space.read::<u32>(super::G_RUN_INPUT) & THE_SHOT != 0;
        let timer = space.read::<i32>(at);
        if timer < 0 && !held {
            return;
        }
        let timer = timer.max(0);
        space.write::<i32>(
            at,
            match timer {
                THE_SHOTS_ROUND.. if held => timer - THE_SHOTS_ROUND,
                THE_SHOTS_ROUND.. => -1,
                _ => timer + 1,
            },
        );
    }

    /// **The shot as the player's update takes it on**, 0x431af9 through 0x431b90: stepped where no conversation is,
    /// an enemy is up and the panel does not hold the shot, and held at -1 otherwise.
    pub fn the_players_shot_goes_on(&self) {
        match !self.a_conversation_stands()
            && self.an_enemy_is_up()
            && !self.the_panel_stands_holding_the_shot()
        {
            true => self.the_shot_steps(),
            false => self.the_shot_timer_is(-1),
        }
    }

    /// **The slot of the small items' ring the next one goes in**, put at `at` — which the stage's first frame leaves
    /// where it is, its zeroing ending short of it.
    pub fn the_small_items_ring_is_at(&self, at: i32) {
        self.space()
            .write::<i32>(ITEMS.start + super::SMALL_ITEMS_RING_AT, at);
    }

    pub fn where_the_small_items_ring_is(&self) -> i32 {
        self.space()
            .read::<i32>(ITEMS.start + super::SMALL_ITEMS_RING_AT)
    }

    /// The player's shot timer, as [`Image::the_shot_steps`] keeps it.
    pub fn the_shot_timer(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_SHOT_TIMER)
    }

    /// The player's shot timer put at a count, as the reset 0x42fec0 puts it at -1.
    pub fn the_shot_timer_is(&self, timer: i32) {
        self.space()
            .write::<i32>(PLAYER.start + super::PLAYER_SHOT_TIMER, timer);
    }

    /// **The player in a state**, with that state's timer from nought, as every writer of the state starts it.
    pub fn the_player_is_in(&self, state: Player) {
        let space = self.space();
        space.write::<i32>(PLAYER.start + super::PLAYER_STATE, state as i32);
        space.write::<i32>(PLAYER.start + super::PLAYER_STATE_TIMER, 0);
    }

    /// The state the player is in and the frames it has been in it.
    pub fn what_the_player_is_doing(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(PLAYER.start + super::PLAYER_STATE),
            space.read::<i32>(PLAYER.start + super::PLAYER_STATE_TIMER),
        )
    }

    /// **The frames nothing can hit the player for**, its timer at `+0x8bb0` set — the count before, the count and its
    /// float.
    pub fn nothing_hits_the_player_for(&self, frames: i32) {
        let space = self.space();
        space.write::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_BEFORE, frames - 1);
        space.write::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR, frames);
        space.write::<f32>(
            PLAYER.start + super::PLAYER_UNHITTABLE_AS_A_FLOAT,
            frames as f32,
        );
    }

    /// **One more frame of the player's state and of its update, and one less of nothing hitting it** (0x431658
    /// through 0x4316c2).
    pub fn the_player_counts_a_frame(&self) {
        let space = self.space();
        for at in [super::PLAYER_STATE_TIMER, super::PLAYER_UPDATE_TIMER] {
            let frames = space.read::<i32>(PLAYER.start + at);
            space.write::<i32>(PLAYER.start + at, frames + 1);
        }
        let unhittable = space.read::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR);
        if unhittable > 0 {
            self.nothing_hits_the_player_for(unhittable - 1);
        }
    }

    pub fn the_frames_nothing_hits_the_player_for(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR)
    }

    /// **The hit taken**, 0x432a90: the state at 4 with its timer at nought, and six frames nothing hits it for
    /// (0x432c29). The card's capture lost with it is not laid.
    pub fn the_player_takes_the_hit(&self) {
        self.the_player_is_in(Player::Hit);
        self.nothing_hits_the_player_for(A_HIT_LEAVES_IT_UNHITTABLE_FOR);
    }

    /// **A death**, 0x4327d0 and the hit state's eighth frame around it (0x431238 through 0x43124a): the point items'
    /// value less 10,000,000 and no lower than its start, a life gone, the player dying with nothing hitting it for
    /// 180 frames, every option let go, `0x400` off the word at `0x4a5744` held within ±`0x400`, and the enemy
    /// manager's misses one on with the word after them at nought.
    pub fn the_player_dies(&self) {
        let space = self.space();
        let value = space.read::<i32>(super::G_POINT_VALUE) - A_DEATH_TAKES_OFF_THE_POINT_VALUE;
        space.write::<i32>(
            super::G_POINT_VALUE,
            value.max(space.read::<i32>(G_POINT_VALUE_START)),
        );
        let lives = space.read::<i32>(super::G_LIVES);
        space.write::<i32>(super::G_LIVES, lives - 1);
        self.the_player_is_in(Player::Dying);
        self.nothing_hits_the_player_for(A_DEATH_LEAVES_IT_UNHITTABLE_FOR);
        space.write::<i32>(PLAYER.start + super::PLAYER_OPTION_COUNT, 0);
        let word = space.read::<i32>(super::G_4A5744) - A_DEATH_TAKES_OFF_4A5744;
        space.write::<i32>(
            super::G_4A5744,
            word.clamp(-A_DEATH_TAKES_OFF_4A5744, A_DEATH_TAKES_OFF_4A5744),
        );
        let manager = space.read::<usize>(super::G_ENEMY_MANAGER);
        if manager != 0 {
            let misses = space.read::<i32>(manager + ENEMY_MANAGERS_MISSES);
            space.write::<i32>(manager + ENEMY_MANAGERS_MISSES, misses + 1);
            space.write::<i32>(manager + ENEMY_MANAGERS_AFTER_A_MISS, 0);
        }
    }

    /// **The power the dying state takes**, on its third frame: the shot type's most, `[0x4a5748]`, off, held at
    /// nought (0x4312c6 through 0x4312ee).
    pub fn the_dying_takes_the_power(&self) {
        let space = self.space();
        let power = space.read::<i32>(super::G_POWER) - space.read::<i32>(super::G_POWER_AT_MOST);
        space.write::<i32>(super::G_POWER, power.max(0));
    }

    /// **The player put back after a death with a life left** (0x4314b2 through 0x431506): at the bottom of the field,
    /// coming back in, the game's speed at 1.0, with nothing hitting it for 280 frames.
    pub fn the_player_comes_back_in(&self) {
        let (x, y) = A_DEATH_PUTS_THE_PLAYER_AT;
        self.the_player_is_at_128ths(x, y);
        self.the_player_is_in(Player::ComingBackIn);
        self.space()
            .write::<f32>(super::G_GAME_SPEED, THE_GAMES_OWN_SPEED);
        self.nothing_hits_the_player_for(COMING_BACK_LEAVES_IT_UNHITTABLE_FOR);
    }

    /// **The player coming back in**, state 0's walk from the bottom: `0xf000 - t * 0x2800 / 60` down at frame `t` of
    /// it (0x431095 through 0x4310d9).
    pub fn the_player_comes_up_to(&self, frames: i32) {
        let y = COMING_BACK_IN_FROM - frames * COMING_BACK_IN_BY / COMING_BACK_IN_FOR;
        let (x, _) = self.where_the_player_is_in_128ths();
        self.the_player_is_at_128ths(x, y);
    }

    /// **The stage's objects taken down with the game object**, which a run's teardown on the way to the title does:
    /// every one of them written nought over where it is kept.
    pub fn the_run_is_taken_down(&self) {
        let space = self.space();
        for at in [
            super::G_BULLET_MANAGER,
            super::G_CARD,
            super::G_ENEMY_MANAGER,
            super::G_ITEM_MANAGER,
            super::G_LASER_MANAGER,
            super::G_PLAYER,
            super::G_PAUSE,
            super::G_BOMB,
            super::G_PANEL,
            super::G_STD,
            super::G_KEPT_STD,
            super::G_GAME,
        ] {
            space.write::<usize>(at, 0);
        }
    }

    /// The chain the two walks are called on, as WinMain made it.
    pub fn chain_object(&self) -> usize {
        CHAIN.start
    }

    /// **The call the score object's own code makes that a laid-out game has to answer**: a shot type's record made
    /// empty, 0x437550, in place of the address [`Th11`](super::Th11) holds.
    pub fn hands_over_the_score_objects_own_call(&self, empties_a_record: usize) {
        super::set_a_record_emptied(empties_a_record);
    }

    /// **A shot type's record made empty**, 0x437550 on `record`: its `0x68d4` bytes zeroed, headed `CR` with a version
    /// of nought and its size, each of the five difficulties' ten places — see [`RECORD_BYTES`] for whose defaults —
    /// and each of the 175 cards numbered at `+0x88` of its entry. The difficulty each card is written with at `+0x8c`
    /// is not laid: nothing of orb's reads it.
    pub fn a_record_is_emptied(&self, record: usize) {
        let space = self.space();
        space.fill_bytes(record, 0, RECORD_BYTES);
        space.write_bytes(record, b"CR");
        space.write::<u16>(record + 0x2, 0);
        space.write::<u32>(record + 0x8, RECORD_BYTES as u32);
        for difficulty in 0..RANKED_DIFFICULTIES {
            for place in 0..RANKED_PLACES {
                let entry =
                    record + RANKING_AT + (difficulty * RANKED_PLACES + place) * RANKED_BYTES;
                space.write::<u32>(entry, A_RANKINGS_TOP - place as u32 * A_RANKINGS_STEP);
                space.write::<u8>(entry + 0x4, 1);
                space.write::<u8>(entry + 0x5, 0);
                space.write_bytes(entry + 0x6, A_RANKINGS_NAME);
                space.write::<u8>(entry + 0xe, 0);
            }
        }
        for card in 0..CARDS {
            let entry = record + CARDS_AT + card * CARD_RECORD_BYTES;
            space.write::<u32>(entry + CARD_NUMBERED_AT, card as u32);
        }
    }

    /// **The score object as the startup read leaves it**, 0x437800 on `file`: the object zeroed, the settings' head
    /// and the blank name as the last one typed, each of its seven records made empty (0x437793 through 0x4377bc), and
    /// the parse of the file copying each of its sound records over the record of its number and its settings over the
    /// object's. **0x437660's `0x200` draws of the generator are not laid** — th10's image has why.
    pub fn the_score_object_is_read_from(&self, file: &super::score_file::Records) {
        let space = self.space();
        space.fill_bytes(SCORE.start, 0, SCORE.len());
        space.write::<usize>(super::G_SCORE_OBJECT, SCORE.start);
        let settings = SCORE.start + THE_SETTINGS;
        space.write_bytes(settings, b"ST");
        space.write::<u16>(settings + 0x2, 0);
        space.write::<u32>(settings + 0x8, SETTINGS_BYTES as u32);
        self.the_last_name_typed_is(&A_BLANK_NAME);
        for (number, held) in file.shot_types.iter().enumerate() {
            let record = SCORE.start + super::RECORDS_AT + number * RECORD_BYTES;
            self.a_record_is_emptied(record);
            if let Some(bytes) = held {
                space.write_bytes(record, bytes);
            }
        }
        if let Some(held) = &file.settings {
            space.write_bytes(settings, held);
        }
    }

    /// **The score object packed for its write**, 0x437a70 up to the pack: each record headed `CR` numbered at `+0xc`
    /// and summed from `+0x8` into `+0x4` (0x437ad0 through 0x437ae1, 0x437500), a record with any other head left
    /// out; and the settings summed the same way.
    pub fn the_score_object_is_packed_for_its_write(&self) -> super::score_file::Records {
        let space = self.space();
        let summed = |at: usize, bytes: usize| {
            let sum = space
                .read_bytes(at + 0x8, bytes - 0x8)
                .iter()
                .fold(0u32, |sum, byte| sum.wrapping_add(u32::from(*byte)));
            space.write::<u32>(at + 0x4, sum);
            space.read_bytes(at, bytes)
        };
        let mut records = super::score_file::Records::default();
        for (number, slot) in records.shot_types.iter_mut().enumerate() {
            let record = SCORE.start + super::RECORDS_AT + number * RECORD_BYTES;
            if space.read_bytes(record, 2) != b"CR" {
                continue;
            }
            space.write::<u32>(record + 0xc, number as u32);
            *slot = Some(summed(record, RECORD_BYTES));
        }
        records.settings = Some(summed(SCORE.start + THE_SETTINGS, SETTINGS_BYTES));
        records
    }

    /// **The ending the run reached, written into the endings seen as the ending is set up** (0x40edb5 through
    /// 0x40edfa): `character * 3 + partner`, on by six where the run took no continue, its byte at 1 and `| 0x10`
    /// where the difficulty is Normal or above — and the byte after the twelve where the ending is six or more.
    pub fn the_ending_writes_its_clear(&self) {
        let space = self.space();
        let mut ending =
            space.read::<i32>(super::G_CHARACTER) * 3 + space.read::<i32>(super::G_SHOT_TYPE);
        if space.read::<i32>(super::G_CONTINUES) == 0 {
            ending += A_GOOD_ENDING_IS_ON_BY;
        }
        let ending = ending.max(0) as usize;
        let mark = match space.read::<i32>(super::G_DIFFICULTY) >= 1 {
            true => 1 | AN_ENDING_OPENS_THE_EXTRA,
            false => 1,
        };
        space.write::<u8>(SCORE.start + THE_ENDINGS_SEEN + ending, mark);
        if ending >= A_GOOD_ENDING_IS_ON_BY as usize {
            space.write::<u8>(SCORE.start + AN_ENDING_OF_SIX_OR_MORE_SEEN, 1);
        }
    }

    fn the_ranking_entry(&self, place: usize) -> usize {
        let space = self.space();
        let shot_type = (space.read::<i32>(super::G_CHARACTER) * 3
            + space.read::<i32>(super::G_SHOT_TYPE))
        .max(0) as usize;
        let difficulty = space.read::<i32>(super::G_DIFFICULTY).clamp(0, 4) as usize;
        SCORE.start
            + super::RECORDS_AT
            + shot_type * RECORD_BYTES
            + RANKING_AT
            + (difficulty * RANKED_PLACES + place) * RANKED_BYTES
    }

    /// **The run's score placed in its shot type's ranking**, 0x42ba80: the first of the difficulty's ten places whose
    /// score the run's is not below, the places under it moved down one, and the place written with the score, the
    /// stage, the continues and a blank name. Answers the place, and nothing where the score ranks in none of the ten.
    pub fn the_ranking_places_the_run(&self) -> Option<usize> {
        let space = self.space();
        let score = space.read::<u32>(super::G_SCORE);
        let place = (0..RANKED_PLACES)
            .find(|place| space.read::<u32>(self.the_ranking_entry(*place)) <= score)?;
        for below in (place + 1..RANKED_PLACES).rev() {
            let moved = space.read_bytes(self.the_ranking_entry(below - 1), RANKED_BYTES);
            space.write_bytes(self.the_ranking_entry(below), &moved);
        }
        let at = self.the_ranking_entry(place);
        space.write::<u32>(at, score);
        space.write::<u8>(at + 0x4, space.read::<i32>(super::G_STAGE) as u8);
        space.write::<u8>(at + 0x5, space.read::<i32>(super::G_CONTINUES) as u8);
        space.write_bytes(at + 0x6, &A_BLANK_NAME);
        space.write::<u8>(at + 0xe, 0);
        Some(place)
    }

    /// **The name a ranked place holds**, the nine bytes at `+0x6` of its entry (0x42e48e, 0x42f1be).
    pub fn the_ranked_place_is_named(&self, place: usize, name: &[u8; 8]) {
        let space = self.space();
        let at = self.the_ranking_entry(place);
        space.write_bytes(at + 0x6, name);
        space.write::<u8>(at + 0xe, 0);
    }

    pub fn the_name_a_ranked_place_holds(&self, place: usize) -> Vec<u8> {
        self.space()
            .read_bytes(self.the_ranking_entry(place) + 0x6, 8)
    }

    /// The name the last entry was typed under — see [`THE_LAST_NAME`].
    pub fn the_last_name_typed(&self) -> [u8; 8] {
        let mut name = [0; 8];
        name.copy_from_slice(&self.space().read_bytes(SCORE.start + THE_LAST_NAME, 8));
        name
    }

    pub fn the_last_name_typed_is(&self, name: &[u8; 8]) {
        let space = self.space();
        space.write_bytes(SCORE.start + THE_LAST_NAME, name);
        space.write::<u8>(SCORE.start + THE_LAST_NAME + 8, 0);
    }

    /// What the title's name entry holds — the title's `+0x58d4`, `+0x58e0`, `+0x58e4` and `+0x58ec`.
    pub fn the_titles_entry(&self) -> TheTitlesEntry {
        let space = self.space();
        let mut name = [0; 8];
        name.copy_from_slice(&space.read_bytes(TITLE.start + super::TITLE_NAME, 8));
        TheTitlesEntry {
            name,
            typed: space.read::<i32>(TITLE.start + super::TITLE_NAME_TYPED),
            not_asked: space.read::<i32>(TITLE.start + super::TITLE_NAME_NOT_ASKED) != 0,
            grid: space.read::<i32>(TITLE.start + super::TITLE_NAME_GRID),
        }
    }

    pub fn the_titles_entry_is(&self, entry: &TheTitlesEntry) {
        let space = self.space();
        space.write_bytes(TITLE.start + super::TITLE_NAME, &entry.name);
        space.write::<u8>(TITLE.start + super::TITLE_NAME + 8, 0);
        space.write::<i32>(TITLE.start + super::TITLE_NAME_TYPED, entry.typed);
        space.write::<i32>(
            TITLE.start + super::TITLE_NAME_NOT_ASKED,
            i32::from(entry.not_asked),
        );
        space.write::<i32>(TITLE.start + super::TITLE_NAME_GRID, entry.grid);
    }

    /// **The stage put at 8 for the ranking after the ending and back at nought after it**, 0x43f6b0's own (0x43f7f8,
    /// 0x43f802 and after) — the record put back to `0x4a3828` unconditionally.
    pub fn the_title_places_the_clear(&self) -> Option<usize> {
        self.the_stage_word_is(PAST_THE_EXTRA);
        let place = self.the_ranking_places_the_run();
        self.the_stage_word_is(0);
        place
    }

    /// **A shot type's record of the score object**, the `0x68d4` bytes at `+0x8` on by `0x68d4`.
    pub fn the_score_objects_record(&self, number: usize) -> Vec<u8> {
        self.space().read_bytes(
            SCORE.start + super::RECORDS_AT + number * RECORD_BYTES,
            RECORD_BYTES,
        )
    }

    /// **And its settings and the endings seen**, the `0x448` bytes at `+0x2ddd4`.
    pub fn the_score_objects_settings(&self) -> Vec<u8> {
        self.space()
            .read_bytes(SCORE.start + THE_SETTINGS, SETTINGS_BYTES)
    }

    /// **A card's entry in a record**, `0x90` bytes from `+0x664` of the record, with its name in the first `0x80`, the
    /// captures at `+0x80` and the attempts at `+0x84`.
    fn a_cards_entry(&self, record: usize, card: usize) -> usize {
        SCORE.start
            + super::RECORDS_AT
            + record * RECORD_BYTES
            + CARDS_AT
            + card * CARD_RECORD_BYTES
    }

    /// The name the card's start copies into its entry, ended by a nought.
    pub fn a_cards_entry_is_named(&self, record: usize, card: usize, name: &[u8]) {
        let space = self.space();
        let entry = self.a_cards_entry(record, card);
        space.write_bytes(entry, name);
        space.write::<u8>(entry + name.len(), 0);
    }

    pub fn the_name_a_cards_entry_carries(&self, record: usize, card: usize) -> Vec<u8> {
        let entry = self.a_cards_entry(record, card);
        (0..0x80)
            .map(|at| self.space().read::<u8>(entry + at))
            .take_while(|byte| *byte != 0)
            .collect()
    }

    pub fn a_card_has_been_captured(&self, record: usize, card: usize, captures: u32) {
        self.space().write::<u32>(
            self.a_cards_entry(record, card) + super::CARD_CAPTURES,
            captures,
        );
    }

    pub fn the_captures_of_a_card(&self, record: usize, card: usize) -> u32 {
        self.space()
            .read::<u32>(self.a_cards_entry(record, card) + super::CARD_CAPTURES)
    }

    pub fn a_card_has_been_attempted(&self, record: usize, card: usize, attempts: u32) {
        self.space().write::<u32>(
            self.a_cards_entry(record, card) + super::CARD_ATTEMPTS,
            attempts,
        );
    }

    /// **The attempt a card's start counts** (0x40c6e4 on): one more in the run's shot type's entry and in the seventh
    /// record's, each held at 99,999, and none while a replay is played back.
    pub fn a_cards_start_counts_an_attempt(&self, card: usize) {
        let space = self.space();
        let manager = space.read::<usize>(super::G_REPLAY_MANAGER);
        if manager != 0 && space.read::<i32>(manager + super::REPLAY_MANAGER_PLAYS_BACK) == 1 {
            return;
        }
        let shot_type = space.read::<i32>(super::G_CHARACTER) * super::SHOT_TYPES_A_CHARACTER
            + space.read::<i32>(super::G_SHOT_TYPE);
        for record in [shot_type.max(0) as usize, super::RECORD_OF_EVERY_SHOT_TYPE] {
            let attempts = self.the_attempts_at_a_card(record, card);
            self.a_card_has_been_attempted(
                record,
                card,
                (attempts + 1).min(super::A_CARDS_COUNT_STOPS_AT),
            );
        }
    }

    pub fn the_attempts_at_a_card(&self, record: usize, card: usize) -> u32 {
        self.space()
            .read::<u32>(self.a_cards_entry(record, card) + super::CARD_ATTEMPTS)
    }
}

#[cfg(test)]
mod tests {
    use super::{DATA, Image, Pause, Player, Scene};
    use crate::game::Game;

    /// Every static this game keeps is in the one range a chapter copies: a static laid out outside it is one a
    /// snapshot would not carry.
    #[test]
    fn every_static_this_game_keeps_is_in_the_range_a_chapter_copies() {
        for address in [
            super::super::G_SCENE,
            super::super::G_GAME,
            super::super::G_PLAYER,
            super::super::G_PAUSE,
            super::super::G_ENEMY_MANAGER,
            super::super::G_SPRITES,
            super::super::G_CHAIN,
            super::super::G_RANDOM,
            super::super::G_STAGE_FRAMES,
            super::super::G_SOUND_MANAGER,
            super::super::G_LOCKS,
            super::super::G_INPUT,
            super::super::G_FOG_FORCED,
            super::super::G_STD,
        ] {
            assert!(
                DATA.contains(&address),
                "{address:#x} is outside the range a chapter copies"
            );
        }
    }

    /// And the image answers what was written into it, which is what says the layout and the reads agree on where
    /// each thing is.
    #[test]
    fn what_is_laid_out_is_what_the_game_reads_back() {
        let image = Image::laid_out_seeded(1);
        let _held = image.enter();
        image.is_in_scene(Scene::Run, Scene::Run);
        image.the_run_starts_at_stage_1();
        image.the_game_object_is_built(false);
        image.the_runs_numbers_are_started_afresh(true);
        image.the_builds_defaults_are_in();
        image.the_replay_is_built(false);
        image.the_stages_objects_are_made();
        image.the_build_is_over();
        image.the_stage_begins();
        image.a_frame_of_the_stage_goes_by();
        image.the_run_has(123_456, 64, 3);
        image.the_generator_is_seeded_with(0x1a2b);
        image.the_pause_is_in(Pause::Nothing);

        let th11 = super::super::Th11;
        let state = unsafe { th11.read_state() };
        assert_eq!(state.scene, Scene::Run as i32);
        assert!(state.playing && state.in_game, "{state}");
        assert_eq!(state.stage, 0);
        assert_eq!(state.difficulty, 1);
        assert_eq!(state.stage_frames, 1);
        // Three bombs of 64 power at the cost this file's `.sht` gives one.
        assert_eq!((state.lives, state.bombs, state.power), (3, 3, 64));
        assert_eq!(state.random_seed, 0x1a2b);
        // `main` alone on the list, which is the stage's script and no enemy to count.
        assert_eq!(state.enemy_count, 0);
        assert!(!state.paused && !state.unsettled);

        image.the_player_is_in(Player::Hit);
        assert!(unsafe { th11.read_state() }.unsettled);
    }

    /// **A fight's song is the one the stage loads into slot 1**, held against the record the stream reads — and the
    /// stage's own song, or any other, is not. Nothing before a stage has loaded its songs, and nothing in the
    /// attract demo.
    #[test]
    fn a_fights_song_is_the_one_the_stage_loads_into_its_second_slot() {
        let image = Image::laid_out_seeded(1);
        let _held = image.enter();
        let th11 = super::super::Th11;
        image.the_stream_plays_the_track(2);
        assert_eq!(
            th11.the_fights_own_song_is_playing(),
            None,
            "a launch whose stage has loaded no songs read one as the fight's",
        );

        image.the_stages_songs_are_loaded(1, 2);
        let heard: Vec<Option<bool>> = (0..4)
            .map(|track| {
                image.the_stream_plays_the_track(track);
                th11.the_fights_own_song_is_playing()
            })
            .collect();
        assert_eq!(heard, [Some(false), Some(false), Some(true), Some(false)]);

        image.the_run_becomes_the_demo();
        assert_eq!(th11.the_fights_own_song_is_playing(), None);
    }
}
