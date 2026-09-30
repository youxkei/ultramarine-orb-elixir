//! A th10 process image laid out in an address space, so that the real [`Th10`](super::Th10) has something to
//! read in a test with no game running.
//!
//! **What a frame of orb's own reads out of this space** is the device the game draws through, the window it
//! asks about, the chain the two walks are called on, the camera whose viewport it sets, the sprite manager
//! whose queue of quads it resets and draws around the draw walk and the word it forces the fog's call through
//! with — and the queue's draw, the camera's set, the fog's set, the options made for the power and the pause
//! menu's resume are functions an e2e test hands over rather than code that would have to be here. See
//! [`Image::hands_over_the_frames_own_calls`].
//!
//! **And a run, as far as [`Th10::read_state`](super::Th10) and the numbers a stage starts from read one**: the
//! supervisor's scenes, the title's screens, the game object with its flags and its frames, the run's numbers,
//! the generator, the player, the enemy manager's list with the stage's `main` on it, the pause object's states,
//! the managers a stage builds and the sound manager's objects. Each is laid out where the game itself keeps
//! it, so a test can say what orb answers for a game in a state the game can really be in.
//!
//! **The addresses and the offsets are the game's own, written from the same constants the reads use**, so what
//! this cannot catch is one of those that is wrong: a wrong one is wrong on both sides at once, and only the
//! real game says otherwise — every one of them was read against `th10.exe` of md5 `7dc488d8`, see
//! `th10/asm.md`.
//!
//! **The values are this file's own, and taking one from [`Th10`](super::Th10) is what is refused here** — th07's
//! image has why: a scene number, a screen number, a state or a mask shared between the writer and the reader is
//! silent when wrong, and what it costs is a piece of the mode never happening. So each value below carries its
//! own literal and where in the exe it was read, and so does every offset `Th10` has no constant for.
//!
//! **The heap blocks' bases are this file's own**, the game taking each of those objects off its allocator:
//! what a read of one has to find is an address inside the space, and the length each block is laid out at is
//! the one its own allocation asks for, written beside it.
//!
//! **Nothing here goes through a slide**: this game is a 32-bit image with relocations stripped
//! (`Characteristics 0x10f`), so the loader puts it at the `0x400000` its addresses are written against.

use std::ops::Range;
use std::sync::Arc;

use orb_api::{Device, Hwnd, Kind, Viewport};
use orb_sim::{Sim, Space};

/// The game's static data, as the real one is: **one range**, `.data`'s own virtual size from its section header
/// — `0x2634c` bytes from `0x473000`, of which the file holds the first `0x4200` and the loader zeroes the rest.
/// Read with the section table of the exe, whose `.rsrc` starts at `0x49a000`.
const DATA: Range<usize> = 0x0047_3000..0x0049_934c;

/// **The sprite manager**, the `0x732460` bytes WinMain allocates at 0x438c48 — which the queue of quads and the
/// 33 `.anm` slots are inside.
const SPRITES: Range<usize> = 0x0300_0000..0x0300_0000 + 0x73_2460;
/// **The `.anm` files and the arrays they hang off**, each a `new(0x130)` of the loader's (0x44712f) and the
/// entries it allocates behind it: one page per file here, the file at its start and its two arrays after it.
const ANM_FILES: Range<usize> = 0x0380_0000..0x0380_0000 + ANM_SLOTS * A_FILES_OWN_SPACE;
const ANM_SLOTS: usize = 0x21;
const A_FILES_OWN_SPACE: usize = 0x1_0000;
/// Where in a file's page its texture entries and its sprites are laid out — numbers of this file's own, far
/// enough apart that neither array reaches the other.
const TEXTURES_IN_THE_PAGE: usize = 0x200;
const SPRITES_IN_THE_PAGE: usize = 0x2000;
/// **What a loaded file's first texture reads as**, of this file's choosing: something rather than nothing,
/// and different for every entry.
const A_TEXTURE: u32 = 0x0420_0000;

/// **The objects a stage's build makes**, each at the length its own `push …; call 0x452493` asks for: the
/// bullets `0x3e0b54` (0x406060), the items `0x21cec0` (0x41aed0), the player `0x4478` (0x425020), the lasers
/// `0x45c` (0x41c290), the pause object `0x2c8` (0x422360), the enemy manager `0x68` (0x40d6b0), the card
/// `0x37b0` (0x408c90), the panel `0x9ed0` (0x414830) and the bomb `0x48` (0x4056b1).
const BULLETS: Range<usize> = 0x0400_0000..0x0400_0000 + 0x3e_0b54;
const ITEMS: Range<usize> = 0x0450_0000..0x0450_0000 + 0x21_cec0;
const PLAYER: Range<usize> = 0x0480_0000..0x0480_0000 + 0x4478;
const LASERS: Range<usize> = 0x0486_0000..0x0486_0000 + 0x45c;
const PAUSE: Range<usize> = 0x0487_0000..0x0487_0000 + 0x2c8;
/// **The replay manager**, `0xb5` words its constructor zeroes (0x428f4d through 0x428f54).
const REPLAY_MANAGER: Range<usize> = 0x0492_0000..0x0492_0000 + 0xb5 * 4;
const ENEMY_MANAGER: Range<usize> = 0x0482_0000..0x0482_0000 + 0x68;
const CARD: Range<usize> = 0x0489_0000..0x0489_0000 + 0x37b0;
const PANEL: Range<usize> = 0x048a_0000..0x048a_0000 + 0x9ed0;
/// **Where a conversation the panel runs is**, an address of this file's own past the panel — see
/// [`Image::a_conversation_is_up`].
const A_CONVERSATION: usize = 0x048a_c000;
const BOMB: Range<usize> = 0x048e_0000..0x048e_0000 + 0x48;
/// **The game object**, `new(0x60)` at 0x4180e4, one per build.
const GAME: Range<usize> = 0x0481_0000..0x0481_0000 + 0x60;
/// **The enemies a stage has on its list**, `0x2518` bytes apiece as the enemy the spawner allocates, and the
/// two-word elements of the list they are on (0x40d751 through 0x40d79f): as many as this file lays out.
const ENEMIES: Range<usize> = 0x0483_0000..0x0483_0000 + ENEMIES_LAID_OUT * ENEMY_BYTES;
const ELEMENTS: Range<usize> = 0x0485_0000..0x0485_0000 + ENEMIES_LAID_OUT * ELEMENT_BYTES;
const ENEMIES_LAID_OUT: usize = 8;
const ENEMY_BYTES: usize = 0x2518;
const ELEMENT_BYTES: usize = 8;
/// **The title**, `0x5acc` bytes (0x42c920).
const TITLE: Range<usize> = 0x0488_0000..0x0488_0000 + 0x5acc;
/// **The score object**, `0x1dcb4` bytes (0x42af36), read out of `scoreth10.dat` once as the game starts.
const SCORE: Range<usize> = 0x048b_0000..0x048b_0000 + 0x1_dcb4;
/// **The chain**, the `0x48` bytes WinMain allocates at 0x438ba5.
const CHAIN: Range<usize> = 0x048f_0000..0x048f_0000 + 0x48;
/// **The sound's objects off the heap**: the `new(4)` DirectSound is kept in (0x43cda4), the music's stream,
/// `new(0x78)` (0x44cb2d), the `new(4)` its buffer is held in (0x44d6a3), the source it reads through, `new(0x94)`
/// (0x44c969), and `thbgm.fmt`'s table of records, `0x34` bytes apiece (0x43d684).
const SOUND_DEVICE: Range<usize> = 0x0490_0000..0x0490_0000 + 4;
const STREAM: Range<usize> = 0x0490_1000..0x0490_1000 + 0x78;
const STREAM_BUFFER: Range<usize> = 0x0490_2000..0x0490_2000 + 4;
const SOURCE: Range<usize> = 0x0490_3000..0x0490_3000 + 0x94;
const TRACKS: Range<usize> = 0x0490_4000..0x0490_4000 + TRACKS_LAID_OUT * TRACK_RECORD_BYTES;
const TRACKS_LAID_OUT: usize = 4;
const TRACK_RECORD_BYTES: usize = 0x34;
/// **The names those records carry**, which are the names a play has to find: the lookup holds the path's file
/// name against each record's start and plays the first record where none matches (0x43d2a0) — and the paths the
/// exe asks for are `bgm/th10_NN.wav`, stage 1's two at 0x46ecb0 and 0x46eca0 (its record's `+0x10` and `+0x14`,
/// `0x4747c8` and `0x4747cc`), the title's at 0x46f018 and stage 2's first at 0x46ec18. The order the real
/// `thbgm.fmt` keeps them in is in that file and not the exe.
const TRACK_NAMES: [&std::ffi::CStr; TRACKS_LAID_OUT] = [
    c"th10_00.wav",
    c"th10_01.wav",
    c"th10_02.wav",
    c"th10_03.wav",
];
/// **Where the sound manager keeps that table**, `+0x1f84`: the start-up loads `thbgm.fmt` (0x41fa28) and stores it
/// at `0x494514` (0x41fa34), which is `0x492590 + 0x1f84` — the store 0x43d684 makes, inlined.
const SOUND_TRACKS: usize = 0x1f84;

/// **The sound manager's own extent**, the `0x14b4` dwords its constructor zeroes (0x43cbf3): `0x492590` to
/// `0x497860`, inside [`DATA`] and so a block of nothing of its own. This file's own literals, for the reason the
/// module doc gives: an extent an e2e test asserts and orb answers from is one number written twice.
const SOUND_MANAGER: Range<usize> = 0x0049_2590..0x0049_2590 + 0x14b4 * 4;
/// **The queue of the sound's commands**: 31 of `0x10c` bytes from `+0x1f88`, each `{command, argument, phase,
/// name}` — enqueued by 0x43e460 through 0x43e4c5 and carried out at the head and shifted by 0x43ddf0.
const COMMANDS: usize = 0x1f88;
const COMMAND_BYTES: usize = 0x10c;
const COMMAND_SLOTS: usize = 31;
/// **The two commands the pause object sends**: 6 as it opens (`push 6` at 0x422bd3) and 7 as its resume takes
/// the pause off (`push 7` at 0x422c40).
pub const THE_PAUSES_COMMAND: u32 = 6;
pub const THE_RESUMES_COMMAND: u32 = 7;

/// **The supervisor's seven locks**, `CRITICAL_SECTION`s of `0x18` from `0x492274` with a byte of depth apiece
/// after them to `0x492323` (0x421420, 0x421c50 through 0x421c88). This file's own literals.
const LOCKS: Range<usize> = 0x0049_2274..0x0049_2324;
/// **The surfaces the screen's captures are made in**, `spr+0x3ad4e0` to `spr+0x3ad6e0` (0x4484d6 through
/// 0x4485be). This file's own literals.
const SURFACES: Range<usize> = 0x003a_d4e0..0x003a_d6e0;

/// **How far apart the jobs a chain walk runs are is none of this file's business**: the jobs are the fake's
/// own code, and what an address space holds of the chain is the object the walks are called on.
///
/// **The queue of quads**: how many times it has been drawn, `[esi+0x58]` one higher on each draw that had a
/// quad in it (0x442fd1); and how many bytes a quad takes, six vertices of `0x1c` — `DrawPrimitiveUP` of
/// `count * 2` triangles of a list with a stride of `0x1c` at 0x442fb1.
const QUEUE_DRAWS: usize = 0x58;
const QUAD_BYTES: usize = 6 * 0x1c;
/// **The word the camera leaves in the sprite manager**, `[spr+0x5c] = [camera+0xe8]` (0x421589 through
/// 0x42158f).
const SPRITES_CAMERAS_WORD: usize = 0x5c;
const CAMERAS_WORD: usize = 0xe8;
/// **The viewport the camera keeps**, of this file's own: the play area in the game's 640x480 output. What the
/// real camera holds there has not been read; what a test holds orb's viewport against is the one the camera
/// keeps, whatever it is.
const THE_CAMERAS_VIEWPORT: Viewport = Viewport {
    x: 32,
    y: 16,
    width: 384,
    height: 448,
    min_z: 0.0,
    max_z: 1.0,
};

/// **The supervisor's word the title comes up by**, `[0x491c00]`: nought on the game's first title, which goes
/// through the screen that waits for a key, and 1 once that screen has been passed (0x42cff2), which puts every
/// title after on the main menu (0x42cf90 through 0x42d137).
const G_HOW_THE_TITLE_COMES_UP: usize = 0x0049_1c00;
/// **The practice select's cursor, kept from one time it is up to the next** (0x431019, 0x43110b, 0x431187).
const G_THE_PRACTICE_SELECTS_CURSOR: usize = 0x0049_1c08;
/// **Whether the run the switch builds is a fresh one**, `[0x491fc4]`: 1 for scene 7 (0x421a0a) and 0 for the
/// stage after one won (0x421a5a) — which is what the build's defaults are put in place on (0x4178ab).
const G_A_FRESH_RUN: usize = 0x0049_1fc4;

/// **The title's count of items and its list of the ones it leaves out**: the count at `+0x2c` (0x42d43b), and
/// the items left out from `+0xb4` with how many of them at `+0xf8` (0x42d44b through 0x42d45c).
const TITLE_ITEMS: usize = 0x2c;
const TITLE_LEFT_OUT: usize = 0xb4;
const TITLE_HOW_MANY_LEFT_OUT: usize = 0xf8;
/// **The bytes that say the Extra is open**, one per shot type from `+0x1d888` of the score object (0x42c850,
/// 0x4306d5 through 0x43073c).
const THE_EXTRA_IS_OPEN_FOR: Range<usize> = 0x1_d888..0x1_d88e;
/// **The settings and the clears**, `0x448` bytes from `+0x1d86c` of the score object (0x42ae82, 0x42add6).
const THE_SETTINGS: usize = 0x1_d86c;
const SETTINGS_BYTES: usize = 0x448;
/// **The endings seen**, a byte for each of the twelve from `+0x1d882` — the shot type's number, on by six for its
/// good ending — and the byte after them for any good ending (0x40b6ba through 0x40b6e2).
const THE_ENDINGS_SEEN: usize = 0x1_d882;
const A_GOOD_ENDING_SEEN: usize = 0x1_d88e;
const A_GOOD_ENDING_IS_ON_BY: i32 = 6;

/// **A shot type's record, as 0x42acb0 makes one empty**: its length, the ranking's five difficulties of ten
/// places in entries of `0x18` from `+0x10`, their scores from 1,000,000 down by 100,000 and the name at
/// `0x46e858`, and the cards' entries of `0x90` from `+0x59c`. This file's own literals, for the reason the module
/// doc gives.
const RECORD_BYTES: usize = 0x437c;
const RANKED_DIFFICULTIES: usize = 5;
const RANKED_PLACES: usize = 10;
const RANKING_AT: usize = 0x10;
const RANKED_BYTES: usize = 0x18;
const A_RANKINGS_TOP: u32 = 1_000_000;
const A_RANKINGS_STEP: u32 = 100_000;
const A_RANKINGS_NAME: &[u8; 8] = b"--------";
const CARDS_AT: usize = 0x59c;
const CARD_RECORD_BYTES: usize = 0x90;
/// **Each card's difficulty**, the 110 bytes at `0x4743c0` that 0x42acb0 copies into every card's `+0x8c` —
/// Easy to Lunatic over and over, and the Extra's thirteen at the end. Read with
/// `objdump -s --start-address=0x4743c0 --stop-address=0x47442e th10.exe`.
const THE_CARDS_DIFFICULTIES: [u8; 110] = [
    2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1,
    2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2,
    3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2,
    3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
];

/// **The game object's timer**, whose frame is `+0x14`: its frame before at `+0x10` and its float at `+0x18`,
/// which 0x405410 starts at `-1`, nought and nought (0x40543c through 0x405442).
const GAME_TIMER_BEFORE: usize = 0x10;
const GAME_TIMER_AS_A_FLOAT: usize = 0x18;

/// **The input's words past the two `Th10` names**: the buttons gone down this frame at `+0x6` and let go at
/// `+0x8` (0x44a90c through 0x44a926), and the run's copy at `+0x2c` with its word before at `+0x2e` and its own
/// gone-down at `+0x32` (0x4297e9 through 0x429809, 0x40ac7f).
const INPUT_PRESSED: usize = 0x6;
const INPUT_RELEASED: usize = 0x8;
const RUN_WORD: usize = 0x2c;
const RUN_WORD_BEFORE: usize = 0x2e;
const RUN_PRESSED: usize = 0x32;
/// **What the run's copy keeps of the word**, `& 0x1f7` at 0x4297e9.
const THE_RUN_KEEPS: u16 = 0x01f7;

/// **The rest of the run's numbers a fresh build writes**, past the ones `Th10` names: the frames since the last
/// conversation boundary (0x4178c1), the faith's fall a frame, 9 (0x417a00), and the hidden number the enemies'
/// script reads, nought on a run's start (0x4179e2).
const G_FRAMES_SINCE_A_CONVERSATION: usize = 0x0047_4c8c;
const G_THE_FAITHS_FALL: usize = 0x0047_4c50;
/// **The defaults a fresh build puts in** (0x4178ab through 0x4179e2): the faith `0xc350` handed to 0x418b80,
/// which keeps it over ten (0x418b80 through 0x418b93); two lives (0x41794f); no power on stage 1 and `0x50` on
/// any other (0x417976 through 0x41798c); and 1.0 for the game's speed (0x4178b1).
const A_FRESH_FAITH: u32 = 0xc350 / 10;
const A_FRESH_RUNS_LIVES: i32 = 2;
const A_FRESH_RUNS_POWER_PAST_STAGE_1: u16 = 0x50;
const THE_FAITHS_FALL: i32 = 9;
const THE_GAMES_OWN_SPEED: f32 = 1.0;

/// **The player's own timer for the state it is in**, whose frame is `+0x478` (0x424e1b, `cmp [player+0x478],8`
/// at 0x425a96).
const PLAYER_STATE_FRAMES: usize = 0x478;
/// **Where the player is built**, in hundredths: across the middle and `0x9c40` down (0x4248cc, 0x4248d2) — `400.0`
/// as its float (0x4248c2). And where a death puts it back, `0xbb80` down (0x425cfc) — `480.0` (0x425d08).
const THE_PLAYER_IS_BUILT_AT: (i32, i32) = (0, 0x9c40);
const A_DEATH_PUTS_THE_PLAYER_AT: (i32, i32) = (0, 0xbb80);
/// **The player's half box**, half of `[0x476fb8 + character * 4]` (0x4249eb through 0x424a0b): `1.0` for Reimu,
/// as `asm.md`'s table of the player has it.
const REIMUS_HALF_BOX: f32 = 1.0;
/// The float a place in hundredths is multiplied by, `0.01` at `0x470b00`.
const A_HUNDREDTH: f32 = 0.01;
/// **The player's speeds as the loader keeps them**, the `.sht` file's floats times the `100.0` at `0x470b44` and
/// truncated (0x4248dc through 0x424926): straight, focused straight, diagonal, focused diagonal. The floats are
/// in `th10.dat` and not in the exe, so these are this file's own — 4.5 and 2.0 straight, and each diagonal the
/// straight times `sin(π/4)`, which is what the load works them out as (0x426537 through 0x42656e).
const THE_PLAYERS_SPEEDS: [i32; 4] = [450, 200, 318, 141];
/// **The rectangle the player's hundredths are held inside**: `-0x47e0` to `0x47e0` across and `0xc80` to
/// `0xa8c0` down (0x425446 through 0x425494).
const THE_PLAYER_IS_HELD_ACROSS: (i32, i32) = (-0x47e0, 0x47e0);
const THE_PLAYER_IS_HELD_DOWN: (i32, i32) = (0xc80, 0xa8c0);

/// **A bullet's own fields past the four `Th10` names**: its flag the spawn sets for the test (bit 1, 0x406b8b),
/// the states the switch at 0x40625f has — 1 active, 0 a free slot — and the timer its age is counted on.
const A_BULLET_MAY_HIT: u32 = 1 << 1;
const A_BULLET_IS_ACTIVE: u16 = 1;
const A_FREE_SLOT: u16 = 0;
/// **The state of a bullet coming out**, which moves and is tested by nothing until it turns active — see `Th10`'s
/// `BULLET_STATE`.
const A_BULLET_COMING_OUT: u16 = 2;

/// **Where a fight's enemy is laid out**, the list's second place and the second of the enemies this file
/// lays out, behind `main` in the first.
const THE_FIGHTS_PLACE: usize = 1;
/// **Its body**, a size of this file's own: what the boss's own `.ecl` gives it is in `th10.dat`.
const A_FIGHTS_BODY: (f32, f32) = (32.0, 32.0);
/// **Where a wave's enemies are laid out**, the places behind the fight's — see [`Image::a_wave_is_up`].
const A_WAVES_PLACES: Range<usize> = THE_FIGHTS_PLACE + 1..ENEMIES_LAID_OUT;
/// **The flags of an enemy with a body**, `0x1120`, which a launch read off the enemies that came on in stage 2
/// after its two scripts — see `Th10`'s `ENEMY_HAS_NO_BODY`.
const A_WAVES_FLAGS: u32 = 0x1120;
/// And its body and its life, numbers of this file's own: what a wave's `.ecl` gives them is in `th10.dat`.
const A_WAVES_BODY: f32 = 24.0;
const A_WAVES_LIFE: i32 = 100;
/// **What a card's start writes into its flags**, `|= 3` at 0x409280, and what its end leaves of them: bit 0,
/// which is the card running, off.
const A_CARD_STARTS: u32 = 3;
const A_CARD_RUNS: u32 = 1;

/// **The `.std` a stage after one won keeps of the stage before**, at a base of this file's choosing: orb reads
/// nothing of it but that it is there (`[0x4776e4]`, 0x417f0b).
const THE_STAGE_BEFORE: Range<usize> = 0x0491_0000..0x0491_1000;
/// **The game object's flag while that `.std` fades out under the new stage**, `|= 0x800` at 0x418201 and off
/// at 0x418426.
const GAME_FADES_THE_STAGE_BEFORE: u32 = 0x800;
/// **The items' array the stage's first frame zeroes**, `0x873a8` dwords from `+0x14` (0x41824c through
/// 0x418255).
const THE_ITEMS_A_STAGE_ZEROES: usize = 0x873a8 * 4;
/// **The last stage the run's numbers count to**, 7 the Extra: 0x4175e0 steps the stage on while it is under 7.
const THE_LAST_STAGE_WORD: i32 = 7;
/// **The record of the stages' files**, `0x474788` on by `0x30` a stage (0x4175ef through 0x4175fa).
const STAGE_RECORDS: usize = 0x0047_4788;
const STAGE_RECORD_BYTES: usize = 0x30;

/// **The pause object's own fields past the three `Th10` names**: its count of items at `+0x2c` (0x422cb5), the
/// run cleared at `+0x1e4` (1 at 0x4234ed, nought at 0x423349), the score out of the ranking at `+0x1e8`
/// (0x4236db), and the game's speed it held back at `+0x2c0` (0x422c19).
const PAUSE_ITEMS: usize = 0x2c;
const PAUSE_THE_RUN_WAS_CLEARED: usize = 0x1e4;
const PAUSE_OUT_OF_THE_RANKING: usize = 0x1e8;
const PAUSE_SPEED_HELD: usize = 0x2c0;
/// **The ranking's name entry over the pause object**, state 12: the name, eight characters and a nought at
/// `+0x2b4`; the place in it at `+0x1e0`; the grid's cursor at `+0xfc` and the grid's length at `+0x104`
/// (0x42362d through 0x4236c5); and the count the cursor holding the ranked place is set over, 25 (0x423601).
/// This file's own literals, for the reason the module doc gives.
const PAUSE_NAME: usize = 0x2b4;
const PAUSE_NAME_TYPED: usize = 0x1e0;
const PAUSE_NAME_GRID: usize = 0xfc;
const PAUSE_NAME_GRID_LENGTH: usize = 0x104;
const PAUSE_RANKED_PLACES: i32 = 0x19;

/// **The scenes the supervisor's switch moves between** (0x4218d0, the table at 0x421b0c), and the one it starts
/// on before any of them (`-2` at 0x42047b). This file's own literals, for the reason the module doc gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    BeforeAny = -2,
    Startup = 0,
    Loading = 1,
    /// The game leaving (0x42194e), which the title's screen 3 asks for after the main menu's last item
    /// (0x42d15d).
    TheGameLeaves = 3,
    Title = 4,
    Run = 7,
    StartOver = 10,
    /// The stage after the one won (0x421a4e), which the conversation after a stage's boss asks for (0x416ed5).
    NextStage = 11,
    /// The ending (0x40b940), which the panel asks for `0x78` frames after the last stage's end (0x414937).
    Ending = 14,
    /// The title again after the ending (0x4219bb), built to come up on the screen after the ending.
    AfterTheEnding = 15,
}

/// **The title's screens this file plays** (the table at 0x42d214): its entry, the screen that waits for a key,
/// the main menu, the screen it leaves on, the three selects a run is chosen through, and the stage a practice is
/// of (0x430ff0).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Entry = 0,
    PressAKey = 1,
    MainMenu = 2,
    Leaving = 3,
    Difficulty = 6,
    Character = 7,
    Shot = 8,
    Practice = 9,
    /// **The screen a title after the ending comes up on** (0x42cfc4), the name entry for the run that was cleared
    /// (0x432cb0) — see [`Image::the_title_names_the_clear`].
    AfterTheEnding = 15,
    /// **And the replay's slots after it** (0x433570), the twenty-five a replay of the clear can be saved in.
    TheReplaysSlots = 16,
}

/// **The title's name entry for a clear and the replay's slots after it** (0x432cb0, 0x433570): the name, eight
/// characters and a nought at `+0x58dc`; the place in it at `+0x58e8`; nought where the clear ranks and 1 where it
/// does not at `+0x58ec` (0x432ea4, 0x432ed0); the grid's cursor at `+0x58f4`, the cursor before at `+0x58f8` and
/// the grid's length at `+0x58fc` (0x432e39); and the title's own count of items at `+0x2c`. This file's own
/// literals, for the reason the module doc gives.
const TITLE_NAME: usize = 0x58dc;
const TITLE_NAME_TYPED: usize = 0x58e8;
const TITLE_NAME_NOT_ASKED: usize = 0x58ec;
const TITLE_NAME_GRID: usize = 0x58f4;
const TITLE_NAME_GRID_LENGTH: usize = 0x58fc;
/// **The name the last clear was typed under**, the settings' `+0xc` — `+0x1d878` of the score object — which
/// the entry starts from (0x432e45) and an entry's end writes (0x4330fa).
const THE_LAST_NAME: usize = 0x1_d878;
/// **A blank name**, the eight spaces at `0x46e354` the entry compares its start against (0x432e6a) and the
/// ranking's placing writes into a place it opens (0x42208b).
pub const A_BLANK_NAME: [u8; 8] = *b"        ";

/// What the title's name entry holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TheTitlesEntry {
    pub name: [u8; 8],
    pub typed: i32,
    pub not_asked: bool,
    pub grid: i32,
}

/// **The steps every one of those screens goes through** (0x42c620 writes them): built, waiting out the frames it
/// ignores a key for, reading one, and the two a decide and a cancel put it in — step 3 and step 4 on the three
/// selects (0x43056b, 0x4304ac), step 4 for the main menu's decide (0x42d665). Screen 1's are 1, 2 and 4 of its
/// own.
pub const STEP_BUILT: i32 = 0;
pub const STEP_WAITING: i32 = 1;
pub const STEP_READING: i32 = 2;
pub const STEP_DECIDED: i32 = 3;
pub const STEP_LEFT: i32 = 4;

/// **The pause object's states this file plays** (0x4223f0, the tables at 0x4224b0 and 0x42438c).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pause {
    Nothing = 0,
    Opening = 1,
    Menu = 2,
    Closing = 3,
    Confirming = 4,
    TheRunIsOver = 6,
    Tallying = 7,
    TheEndsMenu = 8,
    /// The replay's slots, which state 8's item 1 goes to (0x423991).
    SavingAReplay = 10,
    /// The ranking's name entry, which the tally opens where the run's score ranks (0x42386d) and which hands
    /// back to the menu (0x4241bb).
    NamingTheRanking = 12,
    Leaving = 13,
}

/// **The player's states** (the switch at 0x425739, the table at 0x426344).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    ComingBackIn = 0,
    Playing = 1,
    Dying = 2,
    Hit = 4,
}

/// **The flags of the game object this file writes**: `0x4` while its stage is built (0x41787a, 0x417c30) and
/// `0x10` while the pause menu is up (0x422af4, 0x422c38).
const GAME_IS_BEING_BUILT: u32 = 0x4;
const GAME_IS_PAUSED: u32 = 0x10;
/// **And the flags of the run the build clears as it ends**, `& ~0xb` at 0x417c34.
const THE_BUILD_CLEARS: u32 = 0xb;

/// **The flags `main` is spawned with**: `0x10`, which a launch read off stage 2's `main` — see `Th10`'s
/// `ENEMY_HAS_NO_BODY`.
const MAINS_FLAGS: u32 = 0x10;

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
            &LASERS,
            &PAUSE,
            &ENEMY_MANAGER,
            &CARD,
            &PANEL,
            &BOMB,
            &GAME,
            &ENEMIES,
            &ELEMENTS,
            &TITLE,
            &SCORE,
            &CHAIN,
            &SOUND_DEVICE,
            &STREAM,
            &STREAM_BUFFER,
            &SOURCE,
            &TRACKS,
            &THE_STAGE_BEFORE,
            &REPLAY_MANAGER,
        ] {
            space.map(block.start, block.len(), Kind::Private);
        }
        // **What WinMain has made before its first frame**: the chain (0x438ba5), the sprite manager (0x438c48)
        // and the score object read at start-up (0x42af57).
        space.write::<usize>(super::G_CHAIN, CHAIN.start);
        space.write::<usize>(super::G_SPRITES, SPRITES.start);
        space.write::<usize>(super::G_SCORE_OBJECT, SCORE.start);
        // **The difficulty `.data`'s file part holds**, `1` at `0x474c74` — what the difficulty select's cursor
        // comes up on before any run has been chosen (0x42d814 hands it to the cursor).
        space.write::<i32>(super::G_DIFFICULTY, 1);
        // **And the pad's mapping it holds**, `0, 1, 2, 3, -1, -1, -1, -1, 4` at `0x474e88` — what a launch with no
        // `th10.cfg` read over it plays with (0x418c7a).
        space.write::<[i16; 9]>(super::G_PAD_MAPPING, [0, 1, 2, 3, -1, -1, -1, -1, 4]);
        // **The viewport the camera keeps**, which the frame sets after the camera — see [`THE_CAMERAS_VIEWPORT`].
        space.write::<Viewport>(
            super::THE_CAMERA + super::CAMERA_VIEWPORT,
            THE_CAMERAS_VIEWPORT,
        );
        // **The scene before any**, `-2` and nought wanted (0x42047b, 0x420485).
        space.write::<i32>(super::G_SCENE, Scene::BeforeAny as i32);
        space.write::<i32>(super::G_SCENE_WANTED, Scene::Startup as i32);
        let image = Self { sim };
        image.the_sound_is_up();
        image
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is what makes
    /// `Th10`'s reads land in it.
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
    /// `CreateDevice` (0x438e2f) and window left them.
    pub fn shows_through(&self, device: Device, window: Hwnd) {
        let space = self.space();
        space.write::<usize>(super::G_D3D_DEVICE, device.0);
        space.write::<Hwnd>(super::G_GAME_WINDOW, window);
    }

    /// The object the game calls its own whole frame on, which orb's loop is handed.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// **Whether `th10.cfg` says a window**, the byte 0x439730 reads (0x4397ad) to choose which window it asks
    /// for — nought in a launch laid out here, which is the whole screen.
    pub fn is_windowed(&self) -> bool {
        self.space().read::<u8>(super::CFG_WINDOWED) != 0
    }

    /// **The calls the game's own frame makes that a laid-out game has to answer**, because code is the one thing
    /// an address space cannot hold: the queue drawn, the camera set, the fog set, the options made for the power
    /// and the pause menu's resume — in place of the addresses [`Th10`](super::Th10) holds.
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

    /// **And the three calls a pick-up makes as it lands in a stage begun out of a fade** — the shot's step, the
    /// shots' move and the background's calc — in place of 0x4281d0, 0x428280 and 0x402720.
    pub fn hands_over_the_landings_own_calls(
        &self,
        steps_the_shot: usize,
        moves_the_shots: usize,
        runs_the_background: usize,
    ) {
        super::set_the_shot_step(steps_the_shot);
        super::set_the_shots_moved(moves_the_shots);
        super::set_the_backgrounds_calc(runs_the_background);
    }

    fn sprites(&self) -> usize {
        SPRITES.start
    }

    /// **The queue drawn**, which is what 0x442f50 does to the manager — see [`QUEUE_DRAWS`]: nothing where the
    /// count is nought, and otherwise the drawn-from cursor up to the write cursor, the count to nought and the
    /// draws one higher. The `DrawPrimitiveUP` and the states are a device's and not an address space's.
    pub fn draws_the_queue(&self) {
        let space = self.space();
        let sprites = self.sprites();
        if space.read::<u32>(sprites + super::SPRITES_QUEUED) == 0 {
            return;
        }
        let at = space.read::<usize>(sprites + super::SPRITES_WRITE_AT);
        space.write::<usize>(sprites + super::SPRITES_DRAWN_FROM, at);
        space.write::<u32>(sprites + super::SPRITES_QUEUED, 0);
        let draws = space.read::<u32>(sprites + QUEUE_DRAWS);
        space.write::<u32>(sprites + QUEUE_DRAWS, draws + 1);
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

    /// **And the queue put back at its start**, which is the frame's own and not a function's (0x4394a7 through
    /// 0x4394b7) — what a laid-out game's own frame writes where its frame writes it.
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

    /// **The camera set**, which is what 0x421480 does to memory — the queue drawn first where there is a manager
    /// (0x42148f) and the camera's word into the manager (0x421589 through 0x42158f). The two transforms are a
    /// device's.
    pub fn sets_the_camera(&self, camera: usize) {
        self.draws_the_queue();
        let space = self.space();
        let word = space.read::<u32>(camera + CAMERAS_WORD);
        space.write::<u32>(self.sprites() + SPRITES_CAMERAS_WORD, word);
    }

    /// The viewport the camera keeps, which orb's frame sets after it.
    pub fn the_cameras_viewport(&self) -> Viewport {
        THE_CAMERAS_VIEWPORT
    }

    /// **The camera the frame sets**, named where the frame names it before the call (0x4393f4 through
    /// 0x439411), and the word the frame writes `1` into after its viewport (0x439425).
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

    /// The supervisor, which the fog is set on, and the sound manager, which the frame's sounds are.
    pub fn the_supervisor(&self) -> usize {
        super::G_SUPERVISOR
    }

    pub fn the_sound_manager(&self) -> usize {
        super::G_SOUND_MANAGER
    }

    /// **The fog set**, which is 0x420cd0: nothing where the supervisor's word at `+0x750` is nought, and otherwise
    /// the queue drawn, the word to nought and the fog's render state put off (0x420cd0 through 0x420cfa). Answers
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

    /// **And the word forced**, `0xff`, which the game's own frame writes before the call (0x4394c2) — the call
    /// doing nothing where the word already says nought.
    pub fn forces_the_fog_call_through(&self) {
        self.space().write::<u32>(super::G_FOG_FORCED, 0xff);
    }

    pub fn the_fog_word(&self) -> u32 {
        self.space().read::<u32>(super::G_FOG_FORCED)
    }

    /// The scene the supervisor is on and the one it wants, which is the pair `Th10::read_state` reads.
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
    /// (0x42198b through 0x421992).
    pub fn the_switch_is_on_the_scene_it_wanted(&self) {
        let space = self.space();
        let wanted = space.read::<i32>(super::G_SCENE_WANTED);
        space.write::<i32>(super::G_SCENE, wanted);
    }

    /// **Whether the title comes up on its main menu**, the supervisor's word the title's entry reads — see
    /// [`G_HOW_THE_TITLE_COMES_UP`] — and it written as the first title passes its screen that waits for a key.
    pub fn the_title_comes_up_on_its_main_menu(&self) -> bool {
        self.space().read::<i32>(G_HOW_THE_TITLE_COMES_UP) == 1
    }

    pub fn the_title_has_been_passed(&self) {
        self.space().write::<i32>(G_HOW_THE_TITLE_COMES_UP, 1);
    }

    /// **And 3, the title after the ending**, which the switch's scene 15 writes (0x4219ec) and the title's entry
    /// answers with the screen after the ending (0x42cf90 through 0x42cfcb).
    pub fn the_title_is_to_come_up_after_the_ending(&self) {
        self.space().write::<i32>(G_HOW_THE_TITLE_COMES_UP, 3);
    }

    pub fn the_title_comes_up_after_the_ending(&self) -> bool {
        self.space().read::<i32>(G_HOW_THE_TITLE_COMES_UP) == 3
    }

    /// **The title built**, the object zeroed where its constructor leaves it and stored (0x42c9e2).
    pub fn the_title_is_built(&self) {
        let space = self.space();
        space.fill_bytes(TITLE.start, 0, TITLE.len());
        space.write::<usize>(super::G_TITLE, TITLE.start);
    }

    /// **And freed**, which its destructor writes nought over (0x42cd25).
    pub fn the_title_is_freed(&self) {
        self.space().write::<usize>(super::G_TITLE, 0);
    }

    pub fn the_title_is_up(&self) -> bool {
        self.space().read::<usize>(super::G_TITLE) != 0
    }

    /// **A screen set**, 0x42c5c0: the screen, its step at nought and its frames at nought.
    pub fn the_title_is_on(&self, screen: Screen) {
        let space = self.space();
        space.write::<i32>(TITLE.start + super::TITLE_SCREEN, screen as i32);
        space.write::<i32>(TITLE.start + super::TITLE_STEP, STEP_BUILT);
        space.write::<i32>(TITLE.start + super::TITLE_FRAMES, 0);
    }

    /// **A step set**, 0x42c620: the step, and its frames at nought.
    pub fn the_titles_step_is(&self, step: i32) {
        let space = self.space();
        space.write::<i32>(TITLE.start + super::TITLE_STEP, step);
        space.write::<i32>(TITLE.start + super::TITLE_FRAMES, 0);
    }

    /// **One more frame of the screen**, the count the title's calc steps at its tail (0x42d1dc).
    pub fn the_title_counts_a_frame(&self) {
        let space = self.space();
        let frames = space.read::<i32>(TITLE.start + super::TITLE_FRAMES);
        space.write::<i32>(TITLE.start + super::TITLE_FRAMES, frames + 1);
    }

    /// Which screen of the title is up, its step, its cursor and its frames.
    pub fn where_the_title_is(&self) -> (i32, i32, i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(TITLE.start + super::TITLE_SCREEN),
            space.read::<i32>(TITLE.start + super::TITLE_STEP),
            space.read::<i32>(TITLE.start + super::TITLE_CURSOR),
            space.read::<i32>(TITLE.start + super::TITLE_FRAMES),
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

    /// **An item the title leaves out**, put on its list (0x42d44b through 0x42d45c), and the list read.
    pub fn the_title_leaves_out(&self, item: i32) {
        let space = self.space();
        let count = space.read::<i32>(TITLE.start + TITLE_HOW_MANY_LEFT_OUT);
        space.write::<i32>(TITLE.start + TITLE_LEFT_OUT + count as usize * 4, item);
        space.write::<i32>(TITLE.start + TITLE_HOW_MANY_LEFT_OUT, count + 1);
    }

    pub fn the_title_leaves_nothing_out(&self) {
        self.space()
            .write::<i32>(TITLE.start + TITLE_HOW_MANY_LEFT_OUT, 0);
    }

    pub fn the_items_the_title_leaves_out(&self) -> Vec<i32> {
        let space = self.space();
        // Held to the room between the list and its count, a count read off a title the game has not built
        // being a word of anything.
        let room = (TITLE_HOW_MANY_LEFT_OUT - TITLE_LEFT_OUT) / 4;
        let count = space.read::<i32>(TITLE.start + TITLE_HOW_MANY_LEFT_OUT);
        (0..(count.max(0) as usize).min(room))
            .map(|at| space.read::<i32>(TITLE.start + TITLE_LEFT_OUT + at * 4))
            .collect()
    }

    /// **Whether the Extra is open**, any of the score object's bytes for it — see [`THE_EXTRA_IS_OPEN_FOR`].
    pub fn the_extra_is_open(&self) -> bool {
        let space = self.space();
        THE_EXTRA_IS_OPEN_FOR
            .clone()
            .any(|at| space.read::<u8>(SCORE.start + at) != 0)
    }

    /// **And whether it is open for one shot type**, `character * 3 + shot` — its own byte of the six, which the
    /// character and shot selects over the Extra leave an item out by (0x4306d5 through 0x43073c, 0x430a9d through
    /// 0x430b9a).
    pub fn the_extra_is_open_for(&self, shot_type: i32) -> bool {
        self.space()
            .read::<u8>(SCORE.start + THE_EXTRA_IS_OPEN_FOR.start + shot_type.max(0) as usize)
            != 0
    }

    /// The input's words as the read leaves them (0x44a8bd through 0x44a926): this frame's, the frame before's,
    /// those gone down and those let go.
    /// **The pad's mapping**, a button's number for the shot, the bomb, the focus, the pause, four unused and the
    /// skip, negative for none — see `Th10`'s `G_PAD_MAPPING`.
    pub fn the_pads_mapping(&self) -> [i16; 9] {
        self.space().read::<[i16; 9]>(super::G_PAD_MAPPING)
    }

    pub fn the_keys_read(&self, word: u16) {
        let space = self.space();
        let before = space.read::<u16>(super::G_INPUT);
        space.write::<u16>(super::G_INPUT_LAST, before);
        space.write::<u16>(super::G_INPUT, word);
        space.write::<u16>(super::G_INPUT + INPUT_PRESSED, (word ^ before) & word);
        space.write::<u16>(super::G_INPUT + INPUT_RELEASED, (word ^ before) & !word);
    }

    /// The buttons gone down this frame, which is what every menu reads.
    pub fn the_buttons_gone_down(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT + INPUT_PRESSED)
    }

    /// **The run's copy of the word**, which the replay's job makes as it records (0x4297e9 through 0x429809).
    pub fn the_run_copies_the_word(&self) {
        let space = self.space();
        let before = space.read::<u16>(super::G_INPUT + RUN_WORD);
        let word = space.read::<u16>(super::G_INPUT) & THE_RUN_KEEPS;
        space.write::<u16>(super::G_INPUT + RUN_WORD_BEFORE, before);
        space.write::<u16>(super::G_INPUT + RUN_WORD, word);
        space.write::<u16>(super::G_INPUT + RUN_PRESSED, (word ^ before) & word);
    }

    pub fn the_run_word(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT + RUN_WORD)
    }

    /// And the copy the frame before made, which the copy's own buttons gone down are taken against.
    pub fn the_run_word_before(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT + RUN_WORD_BEFORE)
    }

    /// The input's word this frame and the frame before's, as the read leaves them (0x44a8bd through 0x44a926).
    pub fn the_words_the_keys_read(&self) -> (u16, u16) {
        let space = self.space();
        (
            space.read::<u16>(super::G_INPUT),
            space.read::<u16>(super::G_INPUT_LAST),
        )
    }

    /// **What a run's choice writes**: the difficulty, the character and the shot (0x4305cb, 0x430997, 0x430ef6).
    pub fn the_difficulty_is(&self, difficulty: i32) {
        self.space().write::<i32>(super::G_DIFFICULTY, difficulty);
    }

    pub fn the_difficulty(&self) -> i32 {
        self.space().read::<i32>(super::G_DIFFICULTY)
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

    /// **And the stage a run starts at**: the stage and the stage the run started at, both 1, and the record the
    /// build loads the stage's files through, `0x4747b8` — 0x430f24 through 0x430f38.
    pub fn the_run_starts_at_stage_1(&self) {
        let space = self.space();
        space.write::<i32>(super::G_STAGE, 1);
        space.write::<i32>(super::G_STARTING_STAGE, 1);
        space.write::<usize>(super::G_STAGE_LOADS, 0x0047_47b8);
    }

    /// **And the Extra's**, which the same decide writes where the difficulty is 4 or more: both stage words 7 and
    /// the record `0x4748d8` — 0x430f3f through 0x430f4f.
    pub fn the_run_starts_at_the_extra(&self) {
        let space = self.space();
        space.write::<i32>(super::G_STAGE, THE_LAST_STAGE_WORD);
        space.write::<i32>(super::G_STARTING_STAGE, THE_LAST_STAGE_WORD);
        space.write::<usize>(super::G_STAGE_LOADS, 0x0047_48d8);
    }

    /// **A cleared Extra moved past**, which 0x423570 writes before it places the score where the stage word is 7
    /// and the run was cleared: both stage words 8 and the record after the Extra's, `0x474908` (0x42358f through
    /// 0x4235a3) — put back to 7 after the placing only where the stage still reads 7 (0x4235cc), which it does
    /// not. See `asm.md`'s *The pause and the end of a run*.
    pub fn a_cleared_extra_is_moved_past(&self) {
        let space = self.space();
        if space.read::<i32>(super::G_STAGE) != THE_LAST_STAGE_WORD || !self.the_run_was_cleared() {
            return;
        }
        space.write::<i32>(super::G_STAGE, THE_LAST_STAGE_WORD + 1);
        space.write::<i32>(super::G_STARTING_STAGE, THE_LAST_STAGE_WORD + 1);
        space.write::<usize>(super::G_STAGE_LOADS, 0x0047_4908);
    }

    pub fn the_stage_word(&self) -> i32 {
        self.space().read::<i32>(super::G_STAGE)
    }

    /// **The game object built for a run**, 0x4180e0: zeroed (0x41810c), the replay's flag at `+0x5c` (0x418121),
    /// stored (0x418136) and its flag for a build set before the thread is started (0x418124 through 0x41813c) —
    /// with the switch's word for a fresh run (0x421a0a).
    pub fn the_game_object_is_built_for_a_fresh_run(&self) {
        let space = self.space();
        space.fill_bytes(GAME.start, 0, GAME.len());
        space.write::<i32>(GAME.start + super::GAME_REPLAYING, 0);
        space.write::<usize>(super::G_GAME, GAME.start);
        space.write::<u32>(GAME.start + super::GAME_FLAGS, GAME_IS_BEING_BUILT);
        space.write::<i32>(G_A_FRESH_RUN, 1);
    }

    /// **Whether the run the build is for is a fresh one** — see [`G_A_FRESH_RUN`].
    pub fn the_run_is_fresh(&self) -> bool {
        self.space().read::<i32>(G_A_FRESH_RUN) != 0
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

    /// **The run a replay being played**, the flag the build's argument writes at `+0x5c` of the game object
    /// (0x41811d through 0x418121) — and the replay manager made to play one back, its mode at `+0x10` (0x428f67)
    /// and stored (0x429251), which is what the card start's count is gated on (0x409312).
    pub fn the_run_is_a_replay_being_played(&self) {
        let space = self.space();
        space.write::<i32>(GAME.start + super::GAME_REPLAYING, 1);
        space.fill_bytes(REPLAY_MANAGER.start, 0, REPLAY_MANAGER.len());
        space.write::<i32>(REPLAY_MANAGER.start + super::REPLAY_MANAGER_PLAYS_BACK, 1);
        space.write::<usize>(super::G_REPLAY_MANAGER, REPLAY_MANAGER.start);
    }

    /// **Whether the run is a replay being played**, the game object's `+0x5c` — which is what the pause menu's
    /// count of items is read off (0x422cb5).
    pub fn the_run_plays_a_replay(&self) -> bool {
        self.space().read::<i32>(GAME.start + super::GAME_REPLAYING) == 1
    }

    /// **Whether the run is the attract demo**, bit `0x20` of the run's flags (0x42ce58) — which no pause opens
    /// over (0x422406).
    pub fn the_run_is_the_demo(&self) -> bool {
        self.space().read::<u32>(super::G_RUN_FLAGS) & 0x20 != 0
    }

    /// **The pause object up**, which is a stage built.
    pub fn the_pause_is_up(&self) -> bool {
        self.space().read::<usize>(super::G_PAUSE) != 0
    }

    /// The pause object itself, which its resume is entered with in `esi`.
    pub fn the_pause_object(&self) -> usize {
        PAUSE.start
    }

    /// And the player, which the options are made on.
    pub fn the_player_object(&self) -> usize {
        PLAYER.start
    }

    /// **The build's defaults for a fresh run**, 0x4178ab through 0x4179e2: the game's speed, the stage's frames at
    /// nought, no continues, no score, the faith, two lives, the power by the stage, no extends and the hidden
    /// number at nought — see [`A_FRESH_FAITH`] and the three beside it.
    pub fn the_builds_defaults_are_in(&self) {
        let space = self.space();
        space.write::<f32>(super::G_GAME_SPEED, THE_GAMES_OWN_SPEED);
        space.write::<u32>(super::G_STAGE_FRAMES, 0);
        space.write::<u32>(G_FRAMES_SINCE_A_CONVERSATION, 0);
        space.write::<i32>(super::G_CONTINUES, 0);
        space.write::<u32>(super::G_SCORE, 0);
        space.write::<u32>(super::G_FAITH, A_FRESH_FAITH);
        space.write::<i32>(super::G_FAITH_TIMER_BEFORE, -1);
        space.write::<i32>(super::G_FAITH_TIMER, 0);
        space.write::<f32>(super::G_FAITH_TIMER_AS_A_FLOAT, 0.0);
        space.write::<i32>(super::G_LIVES, A_FRESH_RUNS_LIVES);
        let power = match space.read::<i32>(super::G_STAGE) {
            1 => 0,
            _ => A_FRESH_RUNS_POWER_PAST_STAGE_1,
        };
        space.write::<u16>(super::G_POWER, power);
        space.write::<i32>(super::G_EXTENDS, 0);
        space.write::<i32>(super::G_474C98, 0);
    }

    /// **The rest of the build before the replay's seam** — the faith's fall (0x417a00) — **and the stage's objects
    /// made after it** (0x417aab through 0x417ba2): each stored where its own constructor stores it, and zeroed
    /// where that constructor leaves it.
    pub fn the_stages_objects_are_made(&self) {
        let space = self.space();
        space.write::<i32>(G_THE_FAITHS_FALL, THE_FAITHS_FALL);
        for (block, at) in [
            (&PANEL, super::G_PANEL),
            (&PLAYER, super::G_PLAYER),
            (&BULLETS, super::G_BULLET_MANAGER),
            (&ITEMS, super::G_ITEM_MANAGER),
            (&LASERS, super::G_LASER_MANAGER),
            (&PAUSE, super::G_PAUSE),
            (&ENEMY_MANAGER, super::G_ENEMY_MANAGER),
            (&CARD, super::G_CARD),
            (&BOMB, super::G_BOMB),
        ] {
            space.fill_bytes(block.start, 0, block.len());
            space.write::<usize>(at, block.start);
        }
        // **The player as its constructor leaves it** (0x4248bc through 0x4248d2, 0x424d14): across the middle,
        // `400.0` down, with its box and nothing to hit it for the `0x78` frames it is built with.
        let (x, y) = THE_PLAYER_IS_BUILT_AT;
        self.the_player_is_at_hundredths(x, y);
        space.write::<f32>(PLAYER.start + super::PLAYER_HALF, REIMUS_HALF_BOX);
        space.write::<f32>(PLAYER.start + super::PLAYER_HALF + 4, REIMUS_HALF_BOX);
        space.write::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR, 0x78);
        for (at, speed) in THE_PLAYERS_SPEEDS.into_iter().enumerate() {
            space.write::<i32>(PLAYER.start + super::PLAYER_SPEEDS + at * 4, speed);
        }
    }

    /// **The objects a stage after the one won builds again**, the rest being kept by the teardown before it
    /// (0x417ee7 through 0x417f6b): the enemies, the card and the bomb (0x417b33 through 0x417ba2).
    pub fn the_next_stages_objects_are_made(&self) {
        let space = self.space();
        space.write::<i32>(G_THE_FAITHS_FALL, THE_FAITHS_FALL);
        for (block, at) in [
            (&ENEMY_MANAGER, super::G_ENEMY_MANAGER),
            (&CARD, super::G_CARD),
            (&BOMB, super::G_BOMB),
        ] {
            space.fill_bytes(block.start, 0, block.len());
            space.write::<usize>(at, block.start);
        }
    }

    /// **The run's teardown on the way to the stage after the one won** (0x417c80 with the run's flag `0x2`): the
    /// stage's `.std` kept for the next to fade from (0x417f0b), the enemies, the card and the bomb freed
    /// (0x40d644, 0x408c1f, 0x40569b) and the game object with them (0x41807a) — the player, the bullets, the
    /// items, the lasers, the pause object and the panel kept.
    pub fn the_run_is_taken_down_for_the_next_stage(&self) {
        let space = self.space();
        space.write::<usize>(super::G_KEPT_STD, THE_STAGE_BEFORE.start);
        for at in [
            super::G_ENEMY_MANAGER,
            super::G_CARD,
            super::G_BOMB,
            super::G_GAME,
        ] {
            space.write::<usize>(at, 0);
        }
    }

    /// **The game object built for the stage after the one won**, as for a fresh run but with the switch's word at
    /// nought (0x421a5a) — and the replay's flag the old game object had (0x421a6e).
    pub fn the_game_object_is_built_for_the_next_stage(&self, replaying: bool) {
        self.the_game_object_is_built_for_a_fresh_run();
        let space = self.space();
        space.write::<i32>(GAME.start + super::GAME_REPLAYING, i32::from(replaying));
        space.write::<i32>(G_A_FRESH_RUN, 0);
    }

    /// Whether the stage before is kept to fade from, and the fade: the flag set on the game object's frame 0
    /// (0x418201), taken off on its thirtieth with the stage begun (0x418426), and the `.std` freed once faded
    /// (0x418622 through 0x418645).
    pub fn the_stage_before_is_kept(&self) -> bool {
        self.space().read::<usize>(super::G_KEPT_STD) != 0
    }

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

    pub fn the_stage_before_is_freed(&self) {
        self.space().write::<usize>(super::G_KEPT_STD, 0);
    }

    /// **The game object's timer started over**, 0x405410 on `+0x10`, which the fade's own beginning of the stage
    /// makes (0x418619).
    pub fn the_games_timer_starts_over(&self) {
        let space = self.space();
        space.write::<i32>(GAME.start + GAME_TIMER_BEFORE, -1);
        space.write::<i32>(GAME.start + super::GAME_FRAMES, 0);
        space.write::<f32>(GAME.start + GAME_TIMER_AS_A_FLOAT, 0.0);
    }

    /// **The stage stepped on and its record with it**, 0x4175e0: the stage word one on under 7, and the record the
    /// build loads its files through the stage's own.
    pub fn the_run_moves_on_a_stage(&self) {
        let space = self.space();
        let stage = space.read::<i32>(super::G_STAGE);
        let stage = match stage < THE_LAST_STAGE_WORD {
            true => stage + 1,
            false => stage,
        };
        space.write::<i32>(super::G_STAGE, stage);
        space.write::<usize>(
            super::G_STAGE_LOADS,
            STAGE_RECORDS + stage as usize * STAGE_RECORD_BYTES,
        );
    }

    /// **The stage word and its record put at `stage`**, counted from 1 — what the start of a run at that stage
    /// writes (0x43138f, 0x431389).
    pub fn the_stage_word_is(&self, stage: i32) {
        let space = self.space();
        space.write::<i32>(super::G_STAGE, stage);
        space.write::<usize>(
            super::G_STAGE_LOADS,
            STAGE_RECORDS + stage as usize * STAGE_RECORD_BYTES,
        );
    }

    /// **The run's flag for a practice set**, which the main menu's practice item writes (0x42d7ce through
    /// 0x42d7de) where its item for a run takes it off (0x42d749 through 0x42d767).
    pub fn the_run_becomes_a_practice(&self) {
        let space = self.space();
        let flags = space.read::<u32>(super::G_RUN_FLAGS);
        space.write::<u32>(super::G_RUN_FLAGS, flags | 0x10);
    }

    pub fn the_run_is_no_practice(&self) {
        let space = self.space();
        let flags = space.read::<u32>(super::G_RUN_FLAGS);
        space.write::<u32>(super::G_RUN_FLAGS, flags & !0x10);
    }

    /// **The stage a practice starts at**, counted from 1: the stage and the stage the run started at, and the
    /// record the build loads the stage's files through (0x431375 through 0x431394).
    pub fn the_practice_starts_at(&self, stage: i32) {
        self.the_stage_word_is(stage);
        self.space().write::<i32>(super::G_STARTING_STAGE, stage);
    }

    /// **The stage the practice select's cursor came up on last**, `[0x491c08]`, which the select starts its
    /// cursor on (0x431032) and a decide or a cancel keeps (0x43110b, 0x431187).
    pub fn the_practice_select_remembers(&self) -> i32 {
        self.space().read::<i32>(G_THE_PRACTICE_SELECTS_CURSOR)
    }

    pub fn the_practice_select_keeps(&self, cursor: i32) {
        self.space()
            .write::<i32>(G_THE_PRACTICE_SELECTS_CURSOR, cursor);
    }

    /// **Whether the practice of a stage is offered**, the byte at `+0x5` of the entry for `difficulty * 6 +
    /// stage` from `+0x4dc` of the shot type's record — the shot type `character * 3 + shot` — which the select's
    /// decide reads (0x431124 through 0x431156), counted from nought.
    pub fn the_practice_is_offered(&self, difficulty: i32, stage: i32) -> bool {
        let space = self.space();
        let shot_type =
            space.read::<i32>(super::G_CHARACTER) * 3 + space.read::<i32>(super::G_SHOT_TYPE);
        let record = SCORE.start + 0x8 + shot_type.max(0) as usize * RECORD_BYTES;
        let entry = (difficulty * 6 + stage).max(0) as usize;
        space.read::<u8>(record + 0x4dc + entry * 8 + 0x5) != 0
    }

    /// **The run's flag for the attract demo set**, which the title's demo writes as it starts one (0x42ce58).
    pub fn the_run_becomes_the_demo(&self) {
        let space = self.space();
        let flags = space.read::<u32>(super::G_RUN_FLAGS);
        space.write::<u32>(super::G_RUN_FLAGS, flags | 0x20);
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

    /// **The player's step added to where it is and held inside the field** (0x425436 through 0x4254c2).
    pub fn the_player_steps_by(&self, across: i32, down: i32) {
        let (x, y) = self.where_the_player_is_in_hundredths();
        let (left, right) = THE_PLAYER_IS_HELD_ACROSS;
        let (top, bottom) = THE_PLAYER_IS_HELD_DOWN;
        self.the_player_is_at_hundredths(
            (x + across).clamp(left, right),
            (y + down).clamp(top, bottom),
        );
    }

    /// The half of the player's box on each axis, which both of its tests build the box from.
    pub fn the_players_half_box(&self) -> f32 {
        self.space().read::<f32>(PLAYER.start + super::PLAYER_HALF)
    }

    fn bullet_at(&self, slot: usize) -> usize {
        BULLETS.start + super::BULLETS_AT + slot * super::BULLET_BYTES
    }

    /// **A bullet in a slot of the manager's array**, as the spawn leaves one that is out at once (0x406a83): its
    /// flag for the test, active, where it is, its step a frame and its box — and its age from nought.
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

    /// **A bullet still coming out**, its state the one the spawn leaves a bullet in until its sprite's script says
    /// it is out (0x40635e).
    pub fn the_bullet_is_coming_out(&self, slot: usize) {
        self.space().write::<u16>(
            self.bullet_at(slot) + super::BULLET_STATE,
            A_BULLET_COMING_OUT,
        );
    }

    /// **A bullet that has been out for so many frames**, the age the manager's update steps (0x40664f).
    pub fn the_bullet_has_been_out_for(&self, slot: usize, frames: i32) {
        self.space()
            .write::<i32>(self.bullet_at(slot) + super::BULLET_AGE, frames);
    }

    /// **A bullet one frame on**, its step times the game's speed added where it is (0x40643e through 0x406480)
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

    /// Where a bullet is and its box, and whether its slot holds one that is active.
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

    /// **A slot freed**, which is what the walk leaves of a bullet gone off the field or into the player.
    pub fn the_bullet_is_gone(&self, slot: usize) {
        self.space()
            .write::<u16>(self.bullet_at(slot) + super::BULLET_STATE, A_FREE_SLOT);
    }

    /// **The manager's count of the bullets its last walk left**, `+0x5c` (0x4065c9, 0x40664c).
    pub fn the_bullet_manager_counts(&self, bullets: i32) {
        self.space()
            .write::<i32>(BULLETS.start + super::BULLETS_ALIVE, bullets);
    }

    fn the_fight(&self) -> usize {
        ENEMIES.start + THE_FIGHTS_PLACE * ENEMY_BYTES
    }

    /// **A fight up**: an enemy behind `main` on the list with its body, where it is and its life, its attack's
    /// timer from nought, and the enemy manager's slot for the boss the fight is with holding it — which ECL
    /// instruction 332 writes (0x40fffe).
    pub fn a_fight_is_up(&self, at: (f32, f32), life: i32) {
        let space = self.space();
        let enemy = self.the_fight();
        space.fill_bytes(enemy, 0, ENEMY_BYTES);
        space.write::<[f32; 3]>(enemy + super::ENEMY_POSITION, [at.0, at.1, 0.0]);
        space.write::<[f32; 2]>(
            enemy + super::ENEMY_BODY,
            [A_FIGHTS_BODY.0, A_FIGHTS_BODY.1],
        );
        space.write::<i32>(enemy + super::ENEMY_LIFE, life);
        self.onto_the_list(THE_FIGHTS_PLACE);
        space.write::<usize>(ENEMY_MANAGER.start + super::ENEMY_BOSS, enemy);
        space.write::<i32>(enemy + super::ENEMY_ATTACK_FRAMES, 0);
    }

    /// **The enemy laid out at `place` put on the end of the list**, its element pointing at it, and the manager's
    /// count one on — which is what the spawn does with an enemy it has made (0x40d751 through 0x40d79f).
    fn onto_the_list(&self, place: usize) {
        let space = self.space();
        let element = ELEMENTS.start + place * ELEMENT_BYTES;
        space.write::<usize>(element, ENEMIES.start + place * ENEMY_BYTES);
        space.write::<usize>(element + super::ELEMENT_NEXT, 0);
        let mut last = space.read::<usize>(ENEMY_MANAGER.start + super::ENEMY_LIST);
        while last != 0 && space.read::<usize>(last + super::ELEMENT_NEXT) != 0 {
            last = space.read::<usize>(last + super::ELEMENT_NEXT);
        }
        match last {
            0 => space.write::<usize>(ENEMY_MANAGER.start + super::ENEMY_LIST, element),
            last => space.write::<usize>(last + super::ELEMENT_NEXT, element),
        }
        let count = space.read::<i32>(ENEMY_MANAGER.start + super::ENEMY_COUNT);
        space.write::<i32>(ENEMY_MANAGER.start + super::ENEMY_COUNT, count + 1);
    }

    /// **And taken off it**, the count one back, where it is on it at all.
    fn off_the_list(&self, place: usize) {
        let space = self.space();
        let element = ELEMENTS.start + place * ELEMENT_BYTES;
        let mut at = ENEMY_MANAGER.start + super::ENEMY_LIST - super::ELEMENT_NEXT;
        loop {
            let next = space.read::<usize>(at + super::ELEMENT_NEXT);
            if next == 0 {
                break;
            }
            if next == element {
                let after = space.read::<usize>(element + super::ELEMENT_NEXT);
                space.write::<usize>(at + super::ELEMENT_NEXT, after);
                let count = space.read::<i32>(ENEMY_MANAGER.start + super::ENEMY_COUNT);
                space.write::<i32>(ENEMY_MANAGER.start + super::ENEMY_COUNT, count - 1);
                break;
            }
            at = next;
        }
    }

    /// **A wave of `enemies` up, the wave before it gone**: each an enemy with a body on the list behind `main`
    /// and any fight — the flags a launch read off one of stage 2's (see `Th10`'s `ENEMY_HAS_NO_BODY`),
    /// somewhere of its own in the field and a life — with the manager's count of them. Nought is a stage with
    /// nothing up to shoot at but its fight.
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
            let enemy = ENEMIES.start + place * ENEMY_BYTES;
            space.fill_bytes(enemy, 0, ENEMY_BYTES);
            space.write::<u32>(enemy + super::ENEMY_FLAGS, A_WAVES_FLAGS);
            space.write::<[f32; 3]>(
                enemy + super::ENEMY_POSITION,
                [-96.0 + 32.0 * along as f32, 64.0, 0.0],
            );
            space.write::<[f32; 2]>(enemy + super::ENEMY_BODY, [A_WAVES_BODY, A_WAVES_BODY]);
            space.write::<i32>(enemy + super::ENEMY_LIFE, A_WAVES_LIFE);
            self.onto_the_list(place);
        }
    }

    /// **The wave's enemy `along` laid at `at`.**
    pub fn the_waves_enemy_is_at(&self, along: usize, at: (f32, f32)) {
        let enemy = ENEMIES.start + (A_WAVES_PLACES.start + along) * ENEMY_BYTES;
        self.space()
            .write::<[f32; 3]>(enemy + super::ENEMY_POSITION, [at.0, at.1, 0.0]);
    }

    /// **Every enemy of the wave a frame into its attack**, the timer each one's update steps (0x40e572) — the same
    /// count on every one of them, a wave's enemies being laid on the one frame.
    pub fn the_waves_attacks_go_on(&self) {
        let space = self.space();
        for place in A_WAVES_PLACES {
            let at = ENEMIES.start + place * ENEMY_BYTES + super::ENEMY_ATTACK_FRAMES;
            let frames = space.read::<i32>(at);
            space.write::<i32>(at, frames + 1);
        }
    }

    /// **The wave's enemy `along` shot down**, taken off the list — which moves every one behind it a place on.
    pub fn the_waves_enemy_is_shot_down(&self, along: usize) {
        self.off_the_list(A_WAVES_PLACES.start + along);
    }

    /// **The fight's next attack**, its timer at nought (0x41284c).
    pub fn the_fight_attacks_again(&self) {
        self.space()
            .write::<i32>(self.the_fight() + super::ENEMY_ATTACK_FRAMES, 0);
    }

    /// **The fight's attack `frames` in**, the timer the enemy's update steps (0x40e572) set there at once.
    pub fn the_fight_has_attacked_for(&self, frames: i32) {
        self.space()
            .write::<i32>(self.the_fight() + super::ENEMY_ATTACK_FRAMES, frames);
    }

    /// **The fight's attack a frame on**, the timer the enemy's update steps (0x40e572).
    pub fn the_fight_goes_on(&self) {
        let space = self.space();
        let at = self.the_fight() + super::ENEMY_ATTACK_FRAMES;
        let frames = space.read::<i32>(at);
        space.write::<i32>(at, frames + 1);
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

    /// **A card up**, 0x409280: its number and its flags (`|= 3`).
    pub fn a_card_is_up(&self, number: i32) {
        let space = self.space();
        space.write::<i32>(CARD.start + super::CARD_NUMBER, number);
        let flags = space.read::<u32>(CARD.start + super::CARD_FLAGS);
        space.write::<u32>(CARD.start + super::CARD_FLAGS, flags | A_CARD_STARTS);
    }

    /// **A conversation running**, the object the panel keeps at `+0x9eb8` while one is (0x415df9) — an address of
    /// this file's own, orb reading nothing through it but whether it is there.
    pub fn a_conversation_is_up(&self) {
        self.space()
            .write::<usize>(PANEL.start + super::PANEL_CONVERSATION, A_CONVERSATION);
    }

    /// And over, the panel's word at nought (0x415490).
    pub fn no_conversation_is_up(&self) {
        self.space()
            .write::<usize>(PANEL.start + super::PANEL_CONVERSATION, 0);
    }

    /// And down, its running bit off.
    pub fn no_card_is_up(&self) {
        let space = self.space();
        let flags = space.read::<u32>(CARD.start + super::CARD_FLAGS);
        space.write::<u32>(CARD.start + super::CARD_FLAGS, flags & !A_CARD_RUNS);
    }

    /// **The build over**, as its last acts leave it: the game object's timer started (0x405410 at 0x417bf5), its
    /// flag for a build taken off (0x417c30) and the run's flags a build clears (0x417c34).
    pub fn the_build_is_over(&self) {
        let space = self.space();
        space.write::<i32>(GAME.start + GAME_TIMER_BEFORE, -1);
        space.write::<i32>(GAME.start + super::GAME_FRAMES, 0);
        space.write::<f32>(GAME.start + GAME_TIMER_AS_A_FLOAT, 0.0);
        let flags = space.read::<u32>(GAME.start + super::GAME_FLAGS);
        space.write::<u32>(GAME.start + super::GAME_FLAGS, flags & !GAME_IS_BEING_BUILT);
        let run = space.read::<u32>(super::G_RUN_FLAGS);
        space.write::<u32>(super::G_RUN_FLAGS, run & !THE_BUILD_CLEARS);
    }

    /// **The stage begun**, which the game object's calc does on its frame 0 (0x41821c through 0x418299): the
    /// stage's frames at nought, the player reset — its state playing and its state's timer at nought (0x424d90,
    /// 0x424e1b) — and the enemy list emptied with `main` spawned onto it.
    pub fn the_stage_begins(&self) {
        let space = self.space();
        space.write::<u32>(super::G_STAGE_FRAMES, 0);
        space.write::<u32>(G_FRAMES_SINCE_A_CONVERSATION, 0);
        self.the_player_is_in(Player::Playing);
        space.fill_bytes(ITEMS.start + super::ITEMS_AT, 0, THE_ITEMS_A_STAGE_ZEROES);
        space.fill_bytes(ENEMIES.start, 0, ENEMIES.len());
        let main = ENEMIES.start;
        space.write::<u32>(main + super::ENEMY_FLAGS, MAINS_FLAGS);
        space.write::<f32>(main + super::ENEMY_SCRIPT_TIME, 0.0);
        let element = ELEMENTS.start;
        space.write::<usize>(element, main);
        space.write::<usize>(element + super::ELEMENT_NEXT, 0);
        space.write::<usize>(ENEMY_MANAGER.start + super::ENEMY_LIST, element);
        space.write::<i32>(ENEMY_MANAGER.start + super::ENEMY_COUNT, 1);
        space.write::<usize>(ENEMY_MANAGER.start + super::ENEMY_BOSS, 0);
    }

    /// **One frame of the stage**, the game object's calc past its pause: the stage's frames (0x418774), the frames
    /// since a conversation (0x41877f) and the game object's own timer (0x418784) each one on.
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

    /// **Every live enemy's scripts one step on**, the float the enemy update hands each of them — the game's
    /// speed — added to the time (0x40e0e9 through 0x40e0fe, 0x44fb96).
    pub fn the_enemies_scripts_step(&self) {
        let space = self.space();
        let speed = space.read::<f32>(super::G_GAME_SPEED);
        let mut element = space.read::<usize>(ENEMY_MANAGER.start + super::ENEMY_LIST);
        while element != 0 {
            let enemy = space.read::<usize>(element);
            let time = space.read::<f32>(enemy + super::ENEMY_SCRIPT_TIME);
            space.write::<f32>(enemy + super::ENEMY_SCRIPT_TIME, time + speed);
            element = space.read::<usize>(element + super::ELEMENT_NEXT);
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

    /// And `main`'s flags, which is the first of them.
    pub fn the_flags_of(&self, enemy: usize) -> u32 {
        self.space().read::<u32>(enemy + super::ENEMY_FLAGS)
    }

    /// **The game object's pause flag**, as the pause object's opening (0x422af4) and its resume (0x422c38) write
    /// it.
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

    /// **The pause object in a state**, its timer started over as every change of state starts it (0x405410 on
    /// `+0x10`).
    pub fn the_pause_is_in(&self, state: Pause) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + super::PAUSE_STATE, state as i32);
        space.write::<i32>(PAUSE.start + super::PAUSE_FRAMES - 4, -1);
        space.write::<i32>(PAUSE.start + super::PAUSE_FRAMES, 0);
        space.write::<f32>(PAUSE.start + super::PAUSE_FRAMES + 4, 0.0);
    }

    /// And its state, its timer, its cursor and its count read back.
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

    /// **The game's speed held back as the pause opens and put back by its resume** (0x422c11 through 0x422c20,
    /// 0x422c6d through 0x422c73).
    pub fn the_pause_holds_the_speed_back(&self) {
        let space = self.space();
        let speed = space.read::<u32>(super::G_GAME_SPEED);
        space.write::<u32>(PAUSE.start + PAUSE_SPEED_HELD, speed);
        space.write::<f32>(super::G_GAME_SPEED, THE_GAMES_OWN_SPEED);
    }

    pub fn the_pause_puts_the_speed_back(&self, pause: usize) {
        let space = self.space();
        let speed = space.read::<u32>(pause + PAUSE_SPEED_HELD);
        space.write::<u32>(super::G_GAME_SPEED, speed);
    }

    /// **A run over by a game over**, 0x4231d0 outside a replay: the pause object on the end of the run, with the
    /// run not cleared (0x423349). Whether its score ranks is the tally's to say (0x423570 from 0x42384a).
    pub fn the_run_is_over_with_a_game_over(&self) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + PAUSE_THE_RUN_WAS_CLEARED, 0);
        self.the_pause_is_in(Pause::TheRunIsOver);
    }

    /// **A run over by its clear**, 0x423370 outside a replay — a practice's stage won, or the Extra's: the pause
    /// object on the end of the run with the run cleared (0x4234ed).
    pub fn the_run_is_over_cleared(&self) {
        let space = self.space();
        space.write::<i32>(PAUSE.start + PAUSE_THE_RUN_WAS_CLEARED, 1);
        self.the_pause_is_in(Pause::TheRunIsOver);
    }

    /// **The tally's placing of the run's score, 0x423570 past its Extra's stage words**: out of the ranking where
    /// it ranks in none of the ten (`+0x1e8` at 1, 0x4236db); and where it ranks, the place under the pause's
    /// cursor out of 25 (0x423601 through 0x423623), the grid's cursor on its first cell and its length the grid's
    /// string's (0x42362d through 0x423659), the name the last one typed (0x42366b through 0x423688), the grid's
    /// cursor stepped back round to the end where that is a name at all (0x4236a0 through 0x4236a8), and the
    /// place in it after its last character that is not a space (0x4236ad through 0x4236c5).
    pub fn the_pause_places_the_run(&self, grid: i32) {
        let space = self.space();
        let Some(place) = self.the_ranking_places_the_run() else {
            space.write::<i32>(PAUSE.start + PAUSE_OUT_OF_THE_RANKING, 1);
            return;
        };
        self.the_pauses_cursor_is(place as i32, PAUSE_RANKED_PLACES);
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

    /// Whether the tally found the run's score no place in the ranking — see [`Image::the_pause_places_the_run`].
    pub fn the_run_is_out_of_the_ranking(&self) -> bool {
        self.space()
            .read::<i32>(PAUSE.start + PAUSE_OUT_OF_THE_RANKING)
            != 0
    }

    /// **What the ranking's name entry holds**, state 12 over the pause object's fields: the name at `+0x2b4`, the
    /// place in it at `+0x1e0` and the grid's cursor at `+0xfc` (0x423f93 through 0x424204). `not_asked` is the
    /// title's field and none of the pause's: it reads false.
    pub fn the_pauses_entry(&self) -> TheTitlesEntry {
        let space = self.space();
        let mut name = [0; 8];
        name.copy_from_slice(&space.read_bytes(PAUSE.start + PAUSE_NAME, 8));
        TheTitlesEntry {
            name,
            typed: space.read::<i32>(PAUSE.start + PAUSE_NAME_TYPED),
            not_asked: false,
            grid: space.read::<i32>(PAUSE.start + PAUSE_NAME_GRID),
        }
    }

    pub fn the_pauses_entry_is(&self, entry: &TheTitlesEntry) {
        let space = self.space();
        space.write_bytes(PAUSE.start + PAUSE_NAME, &entry.name);
        space.write::<u8>(PAUSE.start + PAUSE_NAME + 8, 0);
        space.write::<i32>(PAUSE.start + PAUSE_NAME_TYPED, entry.typed);
        space.write::<i32>(PAUSE.start + PAUSE_NAME_GRID, entry.grid);
    }

    /// Whether the run the end is for was cleared, which is where state 13's item 0 goes (0x42431d through
    /// 0x424370).
    pub fn the_run_was_cleared(&self) -> bool {
        self.space()
            .read::<i32>(PAUSE.start + PAUSE_THE_RUN_WAS_CLEARED)
            != 0
    }

    /// **The sound's command queued**, 0x43e460: into the first slot of the queue that holds none.
    pub fn a_sound_command_is_queued(&self, command: u32) {
        let space = self.space();
        let queue = super::G_SOUND_MANAGER + COMMANDS;
        if let Some(slot) = (0..COMMAND_SLOTS)
            .map(|slot| queue + slot * COMMAND_BYTES)
            .find(|at| space.read::<u32>(*at) == 0)
        {
            space.write::<u32>(slot, command);
        }
    }

    /// **And the head carried out**, what 0x43ddf0 does once a frame where there is DirectSound (0x43de00): the
    /// command done and the queue shifted up by one (0x43e0e5 through 0x43e110). Answers the command.
    pub fn the_head_of_the_sound_queue_is_carried_out(&self) -> Option<u32> {
        let space = self.space();
        if space.read::<usize>(super::G_SOUND_MANAGER + super::SOUND_DEVICE) == 0 {
            return None;
        }
        let queue = super::G_SOUND_MANAGER + COMMANDS;
        let head = space.read::<u32>(queue);
        if head == 0 {
            return None;
        }
        let rest = space.read_bytes(queue + COMMAND_BYTES, (COMMAND_SLOTS - 1) * COMMAND_BYTES);
        space.write_bytes(queue, &rest);
        space.fill_bytes(
            queue + (COMMAND_SLOTS - 1) * COMMAND_BYTES,
            0,
            COMMAND_BYTES,
        );
        Some(head)
    }

    /// **The sound up, as the game's start-up leaves it**: DirectSound's wrapper (0x43cda4 through 0x43cdd9), and the
    /// one stream a session opens over `thbgm.dat` (0x41fa7a) — its buffer's holder, its source and the source on
    /// the first of `thbgm.fmt`'s records. Answers the four ranges beside the manager's own.
    pub fn the_sound_is_up(&self) -> [Range<usize>; 4] {
        let space = self.space();
        let manager = super::G_SOUND_MANAGER;
        space.write::<usize>(manager + super::SOUND_DEVICE, SOUND_DEVICE.start);
        space.write::<usize>(manager + super::MUSIC_STREAM, STREAM.start);
        space.write::<usize>(STREAM.start + super::STREAM_BUFFER, STREAM_BUFFER.start);
        space.write::<usize>(STREAM.start + super::STREAM_SOURCE, SOURCE.start);
        space.write::<usize>(manager + SOUND_TRACKS, TRACKS.start);
        // **The records' own numbers are this file's**, distinct for each: where in `thbgm.dat` a track is and
        // how long, which is what `Th10::music_identity` tells two apart by.
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

    /// **The stage's two songs loaded**, which the build asks for after the stage's objects unless the run is the
    /// attract demo (0x417ba6 through 0x417bd1) — the record `stage` names the stage's own song and `fight` the
    /// fight's, each slot's path the `bgm/` one the stage record holds for that record's name.
    pub fn the_stages_songs_are_loaded(&self, stage: usize, fight: usize) {
        let space = self.space();
        for (slot, track) in [stage, fight].into_iter().enumerate() {
            let at = super::G_SOUND_MANAGER + super::SOUND_SONGS + slot * super::SONG_PATH_BYTES;
            let path = [b"bgm/".as_slice(), TRACK_NAMES[track].to_bytes()].concat();
            space.fill_bytes(at, 0, super::SONG_PATH_BYTES);
            space.write_bytes(at, &path);
        }
    }

    /// **The source put on a track**, which a play retargets it to (0x44dca0 at 0x43e170).
    pub fn the_stream_plays_the_track(&self, track: usize) {
        self.space().write::<usize>(
            SOURCE.start + super::SOURCE_RECORD,
            TRACKS.start + track * TRACK_RECORD_BYTES,
        );
    }

    /// **And none of it**, which is the manager as its constructor zeroes it — the launch before its sound
    /// start-up has run.
    pub fn the_sound_is_not_up(&self) {
        let space = self.space();
        space.write::<usize>(super::G_SOUND_MANAGER + super::SOUND_DEVICE, 0);
        space.write::<usize>(super::G_SOUND_MANAGER + super::MUSIC_STREAM, 0);
    }

    /// **The manager's own range**, the one range of the sound a launch with no sound up answers — see
    /// [`SOUND_MANAGER`].
    pub fn the_sound_manager_is(&self) -> Range<usize> {
        SOUND_MANAGER
    }

    /// The music's thread, which the stream's opening writes the id of (0x43d6fa).
    pub fn the_music_is_streamed_on_thread(&self, thread: u32) {
        self.space()
            .write::<u32>(super::G_SOUND_MANAGER + super::SOUND_MUSIC_THREAD, thread);
    }

    /// **The supervisor's locks**, which a snapshot leaves alone — see [`LOCKS`].
    pub fn the_supervisors_locks(&self) -> Range<usize> {
        LOCKS
    }

    /// **The capture surfaces of the sprite manager**, which a snapshot leaves alone — see [`SURFACES`].
    pub fn the_capture_surfaces(&self) -> Range<usize> {
        self.sprites() + SURFACES.start..self.sprites() + SURFACES.end
    }

    /// **An `.anm` file in its slot**, with that many textures and that many sprites: the file stored in the
    /// manager's slot (0x44712f through 0x447168), its counts and its two arrays, each texture's
    /// `IDirect3DTexture9` first in its entry and each sprite with its texture's pointer at `+0x4` (0x4471b8
    /// through 0x4471dc, 0x4475f0 through 0x447602). Answers the two ranges a restore has to leave alone.
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
            space.write::<u32>(
                sprites_at + sprite * super::ANM_SPRITE_BYTES + 4,
                A_TEXTURE + (sprite % textures.max(1)) as u32,
            );
        }
        [
            textures_at..textures_at + textures * super::ANM_TEXTURE_BYTES,
            sprites_at..sprites_at + sprites * super::ANM_SPRITE_BYTES,
        ]
    }

    /// And a slot with nothing in it, which is what all but a handful of the 33 are.
    pub fn an_anm_slot_is_empty(&self, slot: usize) {
        self.space()
            .write::<usize>(self.sprites() + super::ANM_FILES_AT + slot * 4, 0);
    }

    /// And the sprite manager not made yet, which is a launch before WinMain's 0x438c48.
    pub fn the_sprite_manager_is_not_made(&self) {
        self.space().write::<usize>(super::G_SPRITES, 0);
    }

    pub fn the_sprite_manager_is_made(&self) {
        self.space().write::<usize>(super::G_SPRITES, SPRITES.start);
    }

    /// **The run's numbers**, as the stage the run is in holds them.
    pub fn the_run_has(&self, score: u32, power: u16, lives: i32) {
        let space = self.space();
        space.write::<u32>(super::G_SCORE, score);
        space.write::<u16>(super::G_POWER, power);
        space.write::<i32>(super::G_LIVES, lives);
    }

    /// **A bomb let go**, as the player's own bomb lets one go: the bomb's `+0x28` raised, which is what it asks
    /// to be nought first (0x425998), and the power it costs taken off (0x4259d4).
    pub fn a_bomb_is_going_off(&self) {
        let space = self.space();
        let power = space.read::<u16>(super::G_POWER);
        space.write::<u16>(super::G_POWER, power - super::POWER_A_BOMB_COSTS);
        space.write::<i32>(BOMB.start + super::BOMB_GOING_OFF, 1);
    }

    /// And the bomb over, that word lowered: the power stays where the bomb left it.
    pub fn no_bomb_is_going_off(&self) {
        self.space()
            .write::<i32>(BOMB.start + super::BOMB_GOING_OFF, 0);
    }

    pub fn the_runs_numbers(&self) -> (u32, u16, i32) {
        let space = self.space();
        (
            space.read::<u32>(super::G_SCORE),
            space.read::<u16>(super::G_POWER),
            space.read::<i32>(super::G_LIVES),
        )
    }

    /// **The rest of what a stage's replay record keeps of the run** — see `resume::Numbers`: the faith, the
    /// frames it stands for, the hidden number, the continues and the extends.
    pub fn the_run_also_has(
        &self,
        faith: u32,
        stands_for: i32,
        the_number_at_474c98: i32,
        continues: i32,
        extends: i32,
    ) {
        let space = self.space();
        space.write::<u32>(super::G_FAITH, faith);
        space.write::<i32>(super::G_FAITH_TIMER, stands_for);
        space.write::<i32>(super::G_474C98, the_number_at_474c98);
        space.write::<i32>(super::G_CONTINUES, continues);
        space.write::<i32>(super::G_EXTENDS, extends);
    }

    /// The faith and the number at `0x474c98` the run has now.
    pub fn the_faith_and_the_number_at_474c98(&self) -> (u32, i32) {
        let space = self.space();
        (
            space.read::<u32>(super::G_FAITH),
            space.read::<i32>(super::G_474C98),
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

    /// The generator's word, with its draws counted from nought, as a playback's seed leaves it (0x4292d2 through
    /// 0x4292d8).
    pub fn the_generator_is_seeded_with(&self, seed: u16) {
        let space = self.space();
        space.write::<u16>(super::G_RANDOM, seed);
        space.write::<u32>(super::G_RANDOM_DRAWS, 0);
    }

    pub fn the_generators_word(&self) -> u16 {
        self.space().read::<u16>(super::G_RANDOM)
    }

    /// **The generator's draws counted from nought again**, which the replay's recording does as a stage is built —
    /// a fresh one's (0x429092) and the stage's after one won (0x42a6ef) — the word left as it stands.
    pub fn the_generators_draws_start_over(&self) {
        self.space().write::<u32>(super::G_RANDOM_DRAWS, 0);
    }

    /// **One draw of the generator**, 0x44b9b0: the word `xor`ed with `0x9630`, `0x6553` taken off, turned two to
    /// the left, and the count of draws one on (0x44b9b0 through 0x44b9d6).
    pub fn the_generator_draws(&self) -> u16 {
        let space = self.space();
        let word = space.read::<u16>(super::G_RANDOM);
        let word = (word ^ 0x9630).wrapping_sub(0x6553).rotate_left(2);
        space.write::<u16>(super::G_RANDOM, word);
        let draws = space.read::<u32>(super::G_RANDOM_DRAWS);
        space.write::<u32>(super::G_RANDOM_DRAWS, draws.wrapping_add(1));
        word
    }

    /// **Where the player is**, in the hundredths it is kept in and the floats made of them — each times the
    /// `0.01` at `0x470b00` (0x42549e through 0x4254c2).
    pub fn the_player_is_at_hundredths(&self, x: i32, y: i32) {
        let space = self.space();
        space.write::<i32>(PLAYER.start + super::PLAYER_HUNDREDTHS, x);
        space.write::<i32>(PLAYER.start + super::PLAYER_HUNDREDTHS + 4, y);
        space.write::<f32>(
            PLAYER.start + super::PLAYER_POSITION,
            x as f32 * A_HUNDREDTH,
        );
        space.write::<f32>(
            PLAYER.start + super::PLAYER_POSITION + 4,
            y as f32 * A_HUNDREDTH,
        );
    }

    pub fn where_the_player_is_in_hundredths(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(PLAYER.start + super::PLAYER_HUNDREDTHS),
            space.read::<i32>(PLAYER.start + super::PLAYER_HUNDREDTHS + 4),
        )
    }

    pub fn where_the_player_is(&self) -> (f32, f32) {
        let space = self.space();
        (
            space.read::<f32>(PLAYER.start + super::PLAYER_POSITION),
            space.read::<f32>(PLAYER.start + super::PLAYER_POSITION + 4),
        )
    }

    /// **The player's past places, whether it is focused, and the options' numbers**, what the replay's record
    /// keeps of the player — each place and each number of this call's own, `from` on in steps of one, and the
    /// focus `from`'s lowest bit.
    pub fn the_players_trail_is(&self, from: i32) {
        let space = self.space();
        for place in 0..super::resume::PLACES_KEPT {
            let at = PLAYER.start + super::PLAYER_PLACES + place * 8;
            space.write::<i32>(at, from + place as i32 * 2);
            space.write::<i32>(at + 4, from + place as i32 * 2 + 1);
        }
        space.write::<i32>(PLAYER.start + super::PLAYER_FOCUSED, from & 1);
        for number in 0..super::resume::OPTION_NUMBERS {
            space.write::<i32>(
                PLAYER.start
                    + super::PLAYER_OPTIONS
                    + number / 8 * super::PLAYER_OPTION_BYTES
                    + number % 8 * 4,
                from + 1000 + number as i32,
            );
        }
    }

    /// **The four options' flags the playback puts at nought**, one to the `+0x58` of each (0x42a66f through
    /// 0x42a681), read back.
    pub fn the_options_moved(&self) -> [u32; 4] {
        let space = self.space();
        [0, 1, 2, 3].map(|option| {
            space.read::<u32>(
                PLAYER.start
                    + super::PLAYER_OPTIONS
                    + option * super::PLAYER_OPTION_BYTES
                    + super::OPTION_MOVED,
            )
        })
    }

    /// **The options made for the power**, what 0x426f70 leaves in memory of the options: each of the four's flag
    /// set, which the options' own rebuild does and the playback then takes off.
    pub fn the_options_are_made(&self) {
        let space = self.space();
        for option in 0..4 {
            space.write::<u32>(
                PLAYER.start
                    + super::PLAYER_OPTIONS
                    + option * super::PLAYER_OPTION_BYTES
                    + super::OPTION_MOVED,
                1,
            );
        }
    }

    /// **The shot's step, what 0x4281d0 leaves of the player's shot timer**: nothing outside state 1; held at -1
    /// until the run's word holds the shot, started at nought on the frame it does; past fifteen taken back
    /// fifteen while the shot is held and put at -1 where it is not, and one on otherwise. The shots it fires
    /// at each count are not laid out — nothing of orb's reads them.
    pub fn the_shot_steps(&self) {
        let space = self.space();
        if space.read::<i32>(PLAYER.start + super::PLAYER_STATE) != Player::Playing as i32 {
            return;
        }
        let at = PLAYER.start + super::PLAYER_SHOT_TIMER;
        let held = space.read::<u16>(super::G_INPUT + RUN_WORD) & super::BUTTON_SHOT != 0;
        let timer = space.read::<i32>(at);
        if timer < 0 && !held {
            return;
        }
        let timer = timer.max(0);
        space.write::<i32>(
            at,
            match timer {
                15.. if held => timer - 15,
                15.. => -1,
                _ => timer + 1,
            },
        );
    }

    /// **The shot as the player's calc takes it on**, 0x4262b4 through 0x42631f: stepped where an enemy is up and
    /// no conversation is, and held at -1 otherwise.
    pub fn the_players_shot_goes_on(&self) {
        let talking = self
            .space()
            .read::<usize>(PANEL.start + super::PANEL_CONVERSATION)
            != 0;
        match self.the_enemies_on_the_list().is_empty() || talking {
            true => self.the_shot_timer_is(-1),
            false => self.the_shot_steps(),
        }
    }

    /// **The slot of the small faith's ring the next one goes in**, put at `at` — which the stage's first frame
    /// leaves where it is, its zeroing ending short of it.
    pub fn the_small_faith_ring_is_at(&self, at: i32) {
        self.space()
            .write::<i32>(ITEMS.start + super::SMALL_FAITH_RING_AT, at);
    }

    /// And where it is.
    pub fn where_the_small_faith_ring_is(&self) -> i32 {
        self.space()
            .read::<i32>(ITEMS.start + super::SMALL_FAITH_RING_AT)
    }

    /// The player's shot timer, as [`Image::the_shot_steps`] keeps it.
    pub fn the_shot_timer(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_SHOT_TIMER)
    }

    /// The player's shot timer put at a count, as the reset 0x424d90 puts it at -1.
    pub fn the_shot_timer_is(&self, timer: i32) {
        self.space()
            .write::<i32>(PLAYER.start + super::PLAYER_SHOT_TIMER, timer);
    }

    /// **The player in a state**, with that state's timer from nought, as every writer of the state starts it
    /// (0x424e1b, 0x426cff, 0x426a44).
    pub fn the_player_is_in(&self, state: Player) {
        let space = self.space();
        space.write::<i32>(PLAYER.start + super::PLAYER_STATE, state as i32);
        space.write::<i32>(PLAYER.start + PLAYER_STATE_FRAMES, 0);
    }

    /// The state the player is in and the frames it has been in it.
    pub fn what_the_player_is_doing(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(PLAYER.start + super::PLAYER_STATE),
            space.read::<i32>(PLAYER.start + PLAYER_STATE_FRAMES),
        )
    }

    /// **One more frame of the player's state, and one less of nothing hitting it** (0x42618e, 0x425e46 through
    /// 0x425eaf).
    pub fn the_player_counts_a_frame(&self) {
        let space = self.space();
        let frames = space.read::<i32>(PLAYER.start + PLAYER_STATE_FRAMES);
        space.write::<i32>(PLAYER.start + PLAYER_STATE_FRAMES, frames + 1);
        let unhittable = space.read::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR);
        if unhittable > 0 {
            space.write::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR, unhittable - 1);
        }
    }

    pub fn the_frames_nothing_hits_the_player_for(&self) -> i32 {
        self.space()
            .read::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR)
    }

    /// **A death**, 0x4269d0: a life gone (0x426a15), the player dying (0x426a44) and nothing hitting it for `180`
    /// frames (0x426ac4).
    pub fn the_player_dies(&self) {
        let space = self.space();
        let lives = space.read::<i32>(super::G_LIVES);
        space.write::<i32>(super::G_LIVES, lives - 1);
        self.the_player_is_in(Player::Dying);
        space.write::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR, 180);
    }

    /// **The player put back after a death with a life left** (0x425cf6 through 0x425d1e): at the bottom of the
    /// field, coming back in, with nothing hitting it for `280` frames (0x425ceb).
    pub fn the_player_comes_back_in(&self) {
        let (x, y) = A_DEATH_PUTS_THE_PLAYER_AT;
        self.the_player_is_at_hundredths(x, y);
        self.the_player_is_in(Player::ComingBackIn);
        self.space()
            .write::<i32>(PLAYER.start + super::PLAYER_UNHITTABLE_FOR, 280);
    }

    /// **The player coming back in**, state 0's walk from the bottom: `48000 - t * 8000 / 60` down at frame `t` of
    /// it (0x425751 through 0x42577b).
    pub fn the_player_comes_up_to(&self, frames: i32) {
        let y = 48000 - frames * 8000 / 60;
        self.the_player_is_at_hundredths(0, y);
    }

    /// **The stage's objects taken down with the game object**, which a run's teardown on the way to the title
    /// does (0x417dc1 through 0x417ffc): each destructor writing nought over the pointer it is kept at — the
    /// bullets 0x406038, the card 0x408c1f, the enemies 0x40d644, the items 0x41aea4, the lasers 0x41c275, the
    /// player 0x424f6a, the pause object 0x42234e, the bomb 0x40569b and the game object 0x41807a.
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
            super::G_GAME,
        ] {
            space.write::<usize>(at, 0);
        }
    }

    /// The chain the two walks are called on, as WinMain made it.
    pub fn chain_object(&self) -> usize {
        CHAIN.start
    }

    /// **The call the score object's own code makes that a laid-out game has to answer**: a shot type's record
    /// made empty, 0x42acb0, in place of the address [`Th10`](super::Th10) holds.
    pub fn hands_over_the_score_objects_own_call(&self, empties_a_record: usize) {
        super::set_a_record_emptied(empties_a_record);
    }

    /// **A shot type's record made empty**, 0x42acb0 on `record`: its `0x437c` bytes zeroed (0x42acb8 through
    /// 0x42acc1), headed `CR` with a version of nought and its size (0x42acc5 through 0x42accf); each of the five
    /// difficulties' ten names `--------` (the string at `0x46e858`) with a score from 1,000,000 down by 100,000,
    /// a stage of 1 and nothing after the name, in entries of `0x18` from `+0x10` (0x42ace0 through 0x42ad27); and
    /// each of the 110 cards numbered at `+0x88` of its entry, with its own byte of the table at `0x4743c0` at
    /// `+0x8c` (0x42ad29 through 0x42adb9).
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
                space.write::<u32>(entry + 0x10, 0);
            }
        }
        for (card, difficulty) in THE_CARDS_DIFFICULTIES.iter().enumerate() {
            let entry = record + CARDS_AT + card * CARD_RECORD_BYTES;
            space.write::<u32>(entry + 0x88, card as u32);
            space.write::<i32>(entry + 0x8c, i32::from(*difficulty as i8));
        }
    }

    /// **The score object as the startup read leaves it**, 0x42ae60 on `file`: the object zeroed (0x42ae66
    /// through 0x42ae6f), the settings' defaults put in (0x42ae8a), each of its seven records made empty (0x42aeb0
    /// through 0x42aed6), and the parse of the file at 0x42b030 copying each of its sound records over the record
    /// of its number and its settings over the object's.
    ///
    /// **The settings' defaults are 0x42add0's head and name and nothing after them**: the `0x448` bytes zeroed,
    /// `ST` with a version of nought and its size, and the blank name at `0x46e354` as the last one typed
    /// (0x42add0 through 0x42ae07). The `0x200` words it then draws from the generator into `+0x46` (0x42ae20
    /// through 0x42ae59) are not laid — nothing orb reads is in them, and the draws would move the generator every
    /// launch's first run starts on, which is this file's own seed. What the head is for is the clears: a file
    /// with no `ST` in it leaves the head where the defaults put it, and the write of the object (0x42b1e0) sums
    /// the settings but writes no head of its own, so without it the clear an ending writes would go out in a
    /// record the next read skips.
    pub fn the_score_object_is_read_from(&self, file: &super::score_file::Records) {
        let space = self.space();
        space.fill_bytes(SCORE.start, 0, SCORE.len());
        let settings = SCORE.start + THE_SETTINGS;
        space.write_bytes(settings, b"ST");
        space.write::<u16>(settings + 0x2, 0);
        space.write::<u32>(settings + 0x8, SETTINGS_BYTES as u32);
        self.the_last_name_typed_is(&A_BLANK_NAME);
        for (number, held) in file.shot_types.iter().enumerate() {
            let record = SCORE.start + 0x8 + number * RECORD_BYTES;
            self.a_record_is_emptied(record);
            if let Some(bytes) = held {
                space.write_bytes(record, bytes);
            }
        }
        if let Some(held) = &file.settings {
            space.write_bytes(settings, held);
        }
    }

    /// **The score object packed for its write**, 0x42b1e0 up to the pack: each record headed `CR` numbered at
    /// `+0xc` and summed from `+0x8` into `+0x4` (0x42b240 through 0x42b28c) and taken, a record with any other
    /// head left out (0x42b246); and the settings summed the same way into their own `+0x4` (0x42b2b5 through
    /// 0x42b2de) and taken whatever their head — the sums written into the object as the write writes them.
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
            let record = SCORE.start + 0x8 + number * RECORD_BYTES;
            if space.read_bytes(record, 2) != b"CR" {
                continue;
            }
            space.write::<u32>(record + 0xc, number as u32);
            *slot = Some(summed(record, RECORD_BYTES));
        }
        records.settings = Some(summed(SCORE.start + THE_SETTINGS, SETTINGS_BYTES));
        records
    }

    /// **The ending the run reached, written into the clears as the ending is built** (0x40b684 through
    /// 0x40b6e2): the shot type's number, on by six where the run took no continue and was played on Normal or
    /// above — its good ending — and the byte of that ending set among the twelve from `+0x1d882`, the six good
    /// ones being the bytes the Extra is open by ([`THE_EXTRA_IS_OPEN_FOR`]); and the byte after them where the
    /// ending was a good one.
    pub fn the_ending_writes_its_clear(&self) {
        let space = self.space();
        let mut ending =
            space.read::<i32>(super::G_CHARACTER) * 3 + space.read::<i32>(super::G_SHOT_TYPE);
        if space.read::<i32>(super::G_CONTINUES) == 0 && space.read::<i32>(super::G_DIFFICULTY) >= 1
        {
            ending += A_GOOD_ENDING_IS_ON_BY;
        }
        let ending = ending.max(0) as usize;
        space.write::<u8>(SCORE.start + THE_ENDINGS_SEEN + ending, 1);
        if ending >= A_GOOD_ENDING_IS_ON_BY as usize {
            space.write::<u8>(SCORE.start + A_GOOD_ENDING_SEEN, 1);
        }
    }

    /// **The run's score placed in its shot type's ranking**, 0x421fa0 over the record of `character * 3 + shot`
    /// (0x432d96 through 0x432dab): the first of the difficulty's ten places whose score the run's is not below
    /// (0x421fc0), the places under it moved down one (0x421fe7 through 0x422023), and the place written with the
    /// score, the stage, the byte at `0x474c90` and a blank name (0x422034 through 0x4220ad). Answers the place,
    /// and nothing where the score ranks in none of the ten (0x421fcd).
    pub fn the_ranking_places_the_run(&self) -> Option<usize> {
        let space = self.space();
        let shot_type = (space.read::<i32>(super::G_CHARACTER) * 3
            + space.read::<i32>(super::G_SHOT_TYPE))
        .max(0) as usize;
        let difficulty = space.read::<i32>(super::G_DIFFICULTY).max(0) as usize;
        let score = space.read::<u32>(super::G_SCORE);
        let ranking = SCORE.start + 0x8 + shot_type * RECORD_BYTES + RANKING_AT;
        let entry = |place: usize| ranking + (difficulty * RANKED_PLACES + place) * RANKED_BYTES;
        let place = (0..RANKED_PLACES).find(|place| space.read::<u32>(entry(*place)) <= score)?;
        for below in (place + 1..RANKED_PLACES).rev() {
            let moved = space.read_bytes(entry(below - 1), RANKED_BYTES);
            space.write_bytes(entry(below), &moved);
        }
        let at = entry(place);
        space.write::<u32>(at, score);
        space.write::<u8>(at + 0x4, space.read::<i32>(super::G_STAGE) as u8);
        space.write::<u8>(at + 0x5, space.read::<u8>(0x0047_4c90));
        space.write_bytes(at + 0x6, &A_BLANK_NAME);
        space.write::<u8>(at + 0xe, 0);
        Some(place)
    }

    /// **The name a ranked place holds**, the nine bytes at `+0x6` of its entry the entry's end copies the name
    /// into (0x4330b1 through 0x4330f8).
    pub fn the_ranked_place_is_named(&self, place: usize, name: &[u8; 8]) {
        let space = self.space();
        let shot_type = (space.read::<i32>(super::G_CHARACTER) * 3
            + space.read::<i32>(super::G_SHOT_TYPE))
        .max(0) as usize;
        let difficulty = space.read::<i32>(super::G_DIFFICULTY).max(0) as usize;
        let at = SCORE.start
            + 0x8
            + shot_type * RECORD_BYTES
            + RANKING_AT
            + (difficulty * RANKED_PLACES + place) * RANKED_BYTES;
        space.write_bytes(at + 0x6, name);
        space.write::<u8>(at + 0xe, 0);
    }

    pub fn the_name_a_ranked_place_holds(&self, place: usize) -> Vec<u8> {
        let space = self.space();
        let shot_type = (space.read::<i32>(super::G_CHARACTER) * 3
            + space.read::<i32>(super::G_SHOT_TYPE))
        .max(0) as usize;
        let difficulty = space.read::<i32>(super::G_DIFFICULTY).max(0) as usize;
        let at = SCORE.start
            + 0x8
            + shot_type * RECORD_BYTES
            + RANKING_AT
            + (difficulty * RANKED_PLACES + place) * RANKED_BYTES;
        space.read_bytes(at + 0x6, 8)
    }

    /// The name the last clear was typed under — see [`THE_LAST_NAME`].
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

    /// What the title's name entry holds — see [`TITLE_NAME`].
    pub fn the_titles_entry(&self) -> TheTitlesEntry {
        let space = self.space();
        let mut name = [0; 8];
        name.copy_from_slice(&space.read_bytes(TITLE.start + TITLE_NAME, 8));
        TheTitlesEntry {
            name,
            typed: space.read::<i32>(TITLE.start + TITLE_NAME_TYPED),
            not_asked: space.read::<i32>(TITLE.start + TITLE_NAME_NOT_ASKED) != 0,
            grid: space.read::<i32>(TITLE.start + TITLE_NAME_GRID),
        }
    }

    pub fn the_titles_entry_is(&self, entry: &TheTitlesEntry) {
        let space = self.space();
        space.write_bytes(TITLE.start + TITLE_NAME, &entry.name);
        space.write::<u8>(TITLE.start + TITLE_NAME + 8, 0);
        space.write::<i32>(TITLE.start + TITLE_NAME_TYPED, entry.typed);
        space.write::<i32>(
            TITLE.start + TITLE_NAME_NOT_ASKED,
            i32::from(entry.not_asked),
        );
        space.write::<i32>(TITLE.start + TITLE_NAME_GRID, entry.grid);
    }

    /// **And the grid's length**, which the entry's first step writes from the grid's string (0x432e21 through
    /// 0x432e39).
    pub fn the_titles_grid_is_this_long(&self, length: i32) {
        self.space()
            .write::<i32>(TITLE.start + TITLE_NAME_GRID_LENGTH, length);
    }

    /// **A shot type's record of the score object**, the `0x437c` bytes at `+0x8` on by `0x437c` (0x42aeb1).
    pub fn the_score_objects_record(&self, number: usize) -> Vec<u8> {
        self.space()
            .read_bytes(SCORE.start + 0x8 + number * RECORD_BYTES, RECORD_BYTES)
    }

    /// **And its settings and clears**, the `0x448` bytes at `+0x1d86c` (0x42b152).
    pub fn the_score_objects_settings(&self) -> Vec<u8> {
        self.space().read_bytes(SCORE.start + 0x1_d86c, 0x448)
    }

    /// **A card's entry in a record**, `0x90` bytes from `+0x59c` of the record — the shot type's at `+0x5a4` of
    /// the object on by `0x437c` (0x409343) and the seventh's at `+0x19a8c` (0x40938f) — with its name in the first
    /// `0x80`, the captures at `+0x80` and the attempts at `+0x84` (0x40937a, 0x4093aa).
    fn a_cards_entry(&self, record: usize, card: usize) -> usize {
        SCORE.start + 0x8 + record * RECORD_BYTES + CARDS_AT + card * CARD_RECORD_BYTES
    }

    /// The name the card's start copies into its entry (0x409350 through 0x409358), ended by a nought.
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
        self.space()
            .write::<u32>(self.a_cards_entry(record, card) + 0x80, captures);
    }

    pub fn the_captures_of_a_card(&self, record: usize, card: usize) -> u32 {
        self.space()
            .read::<u32>(self.a_cards_entry(record, card) + 0x80)
    }

    pub fn a_card_has_been_attempted(&self, record: usize, card: usize, attempts: u32) {
        self.space()
            .write::<u32>(self.a_cards_entry(record, card) + 0x84, attempts);
    }

    /// **The attempt a card's start counts** (0x409312 through 0x4093b8): one more in the run's shot type's
    /// entry and in the seventh record's, each held at 99,999, and none while a replay is played back.
    pub fn a_cards_start_counts_an_attempt(&self, card: usize) {
        let space = self.space();
        let manager = space.read::<usize>(super::G_REPLAY_MANAGER);
        if manager != 0 && space.read::<i32>(manager + super::REPLAY_MANAGER_PLAYS_BACK) == 1 {
            return;
        }
        let shot_type = space.read::<i32>(super::G_CHARACTER) * super::SHOT_TYPES_A_CHARACTER
            + space.read::<i32>(super::G_SHOT_TYPE);
        for record in [shot_type as usize, super::RECORD_OF_EVERY_SHOT_TYPE] {
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
            .read::<u32>(self.a_cards_entry(record, card) + 0x84)
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
        image.the_game_object_is_built_for_a_fresh_run();
        image.the_builds_defaults_are_in();
        image.the_stages_objects_are_made();
        image.the_build_is_over();
        image.the_stage_begins();
        image.a_frame_of_the_stage_goes_by();
        image.the_run_has(123_456, 64, 3);
        image.the_generator_is_seeded_with(0x1a2b);
        image.the_pause_is_in(Pause::Nothing);

        let th10 = super::super::Th10;
        let state = unsafe { th10.read_state() };
        assert_eq!(state.scene, Scene::Run as i32);
        assert!(state.playing && state.in_game, "{state}");
        assert_eq!(state.stage, 0);
        assert_eq!(state.difficulty, 1);
        assert_eq!(state.stage_frames, 1);
        assert_eq!((state.lives, state.bombs, state.power), (3, 3, 64));
        assert_eq!(state.random_seed, 0x1a2b);
        // `main` alone on the list, which is the stage's script and no enemy to count.
        assert_eq!(state.enemy_count, 0);
        assert!(!state.paused && !state.unsettled);

        image.the_player_is_in(Player::Hit);
        assert!(unsafe { th10.read_state() }.unsettled);
    }

    /// **A fight's song is the one the stage loads into the slot the conversation before its boss plays**, held
    /// against the record the stream reads — and the stage's own song, or any other, is not. Nothing before a stage
    /// has loaded its songs, and nothing in the attract demo, whose build loads none.
    #[test]
    fn a_fights_song_is_the_one_the_stage_loads_for_the_conversation_before_its_boss() {
        let image = Image::laid_out_seeded(1);
        let _held = image.enter();
        let th10 = super::super::Th10;
        image.the_stream_plays_the_track(1);
        assert_eq!(
            th10.the_fights_own_song_is_playing(),
            None,
            "a launch whose stage has loaded no songs read one as the fight's",
        );

        image.the_stages_songs_are_loaded(0, 1);
        let heard: Vec<Option<bool>> = (0..4)
            .map(|track| {
                image.the_stream_plays_the_track(track);
                th10.the_fights_own_song_is_playing()
            })
            .collect();
        assert_eq!(heard, [Some(false), Some(true), Some(false), Some(false)]);

        image.the_run_becomes_the_demo();
        assert_eq!(th10.the_fights_own_song_is_playing(), None);
    }
}
