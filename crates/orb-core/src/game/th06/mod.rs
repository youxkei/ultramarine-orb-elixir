//! 東方紅魔郷 1.02h, as a [`Game`].
//!
//! Globals come from `config/globals.csv` of the GensokyoClub/th06
//! decompilation and field offsets from its struct definitions, both for this
//! exact build. Offsets are spelled out rather than derived from Rust structs
//! because only a handful of the fields matter and mirroring whole structs
//! (`Player` alone is 0x98f0 bytes) would be far more to get wrong.

pub mod chapters;
/// Reachable from this crate's own tests and, through the `sim` feature, from the tests of the
/// crates that drive it — which is where the scenarios live.
#[cfg(any(test, feature = "sim"))]
pub mod image;

use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::Hwnd;

use crate::audio::{Music, SoundBuffer};
use crate::d3d8::{D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, Device, Texture, Viewport};
use crate::game::{
    Boundary, Call, FrameCalls, Game, Hooks, Menu, Pad, PanelTile, Patch, Reading, Rect,
    Reproduction, RunStart, RunState, State,
};
use crate::log;

use orb_api::mem;

pub struct Th06;

/// `th06::Chain::RunCalcChain`, one call per logic frame from
/// `GameWindow::Render`, with drawing done by `RunDrawChain` just before it.
const RUN_CALC_CHAIN: usize = 0x0041ca10;
const RUN_DRAW_CHAIN: usize = 0x0041cad0;
/// `push ebp; mov ebp,esp; sub esp,0x14` — position-independent, 6 bytes. Both
/// chain functions start with the same MSVC prologue.
const RUN_CHAIN_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x14];

/// `th06::ReplayManager::SaveReplay`, `__cdecl`.
///
/// Writes the replay file only when given a path. `Supervisor::OnUpdate` also
/// calls it with nulls at every scene change to tear the recording down, so
/// suppressing replay saving has to skip the write specifically — stubbing the
/// whole function leaves the replay state behind and the game crashes later.
const SAVE_REPLAY: usize = 0x0042ab30;
/// `push ebp; mov ebp,esp; sub esp,0xa8` — position-independent, 9 bytes.
const SAVE_REPLAY_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x81, 0xec, 0xa8, 0x00, 0x00, 0x00];

/// `th06::Chain::Cut`, `__thiscall` with the element on the stack. Unlinks a job and,
/// through its deleted callback, lets whatever registered it take itself down — which is
/// how the game removes one, so it is how orb removes one. Identified by its two searches,
/// the first from `this` and the second from `this + 0x20`: the calc chain's root and the
/// draw chain's, which also fixes `Chain`'s layout.
const CHAIN_CUT: usize = 0x0041cde0;
/// `th06::ScreenEffect::ShakeScreen`, matched against a job's callback rather than called.
/// Its `isTimeStopped` branch writes 32.0, 16.0, 384.0 and 448.0 to 0x69d6dc onwards — the
/// arcade region — which is what says this is the right function.
const SHAKE_SCREEN: usize = 0x0042ffc0;
/// `th06::Ending::OnUpdate`, matched against a job's callback rather than called: the
/// ending's own object is nowhere in a global, and the job it registers carries a pointer to
/// it. `Ending::RegisterChain` at 0x4107b0 is what identifies it — it hands this address to
/// `Chain::CreateElem` and writes the object it allocated into the element that comes back.
const ENDING_ON_UPDATE: usize = 0x004109c0;
/// `th06::ResultScreen::OnUpdate`, found the same way and for the same reason: the result
/// screen is allocated by `ResultScreen::RegisterChain` at 0x42d773, which puts it nowhere but
/// into the element it registers.
const RESULT_SCREEN_ON_UPDATE: usize = 0x0042d98e;

/// `th06::ReplayManager::StopRecording`, `__cdecl`. Writes the last two entries of an
/// input record: a blank one at the frame the run stopped, and a terminator at frame
/// 9999999. `GameManager::DeletedCallback` calls it — the `call 0x42aab0` at 0x41c24c,
/// after cutting the player and the stage — so it runs at every stage teardown, and
/// during playback `replayInputs` points into the replay that was loaded.
const STOP_RECORDING: usize = 0x0042aab0;
/// `push ebp; mov ebp,esp; push ecx; mov eax,[g_ReplayManager]` — 9 bytes, and the
/// absolute load reading 0x6d3f18 is both position-independent and what identifies the
/// function.
const STOP_RECORDING_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x51, 0xa1, 0x18, 0x3f, 0x6d, 0x00];

/// `th06::GameManager::AddedCallback`, `__cdecl` taking the manager. `GameManager::RegisterChain`
/// at 0x41ba6a registers it as the added callback of the element at 0x69d720, priority 4, so
/// `Chain::AddToCalcChain` runs it at the moment a stage is registered — before that stage's own
/// first update, which comes later in the same walk of the chain.
///
/// This is where the game itself puts a stage's numbers in place: for a replay it calls
/// `ReplayManager::RegisterChain` from inside itself, whose `AddedCallbackDemo` writes the eight
/// fields of that stage's record. It is also where rank comes from a table indexed by difficulty
/// and where practice mode overwrites the power, which is why orb writes a resumed run's numbers
/// after it returns rather than before the transition into the stage.
const GAME_MANAGER_ADDED: usize = 0x0041bb02;
/// `push ebp; mov ebp,esp; sub esp,0x28` — position-independent, 6 bytes.
const GAME_MANAGER_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x28];

/// `th06::MainMenu::AddedCallback`, `__cdecl` taking the menu. Registered at 0x43a3c4 as the `+0x8`
/// of the chain element at `menu+0x8234`, and it is the callback that starts the title theme —
/// `bgm/th06_01.mid` at 0x46c3a4, pushed at 0x43a475.
///
/// Hooked to bracket one read of the score file out of the three. All three go through the helper
/// at 0x42b0d9: this one at 0x43a5c0, `GameManager::AddedCallback`'s at 0x41bcdc, and the ranking
/// screen's at 0x42f47f. What each does with what it read is what tells them apart — `hscr` at
/// 0x42b280, `catk` at 0x42b466, `clrd` at 0x42b502, `pscr` at 0x42b65e:
///
/// | read | hscr | catk | clrd | pscr |
/// | --- | --- | --- | --- | --- |
/// | this one | | | into `g_GameManager` at 0x69ccd0 | at 0x69cd30 |
/// | `GameManager::AddedCallback`, once per stage | ✓ | ✓ | ✓ | ✓ |
/// | the ranking screen's added callback at 0x42f060 | into a 5×4 table of difficulty by shot at screen+0x3ab0 | ✓ | ✓ | ✓ |
///
/// The two globals this one fills are what the front end lights `Extra Start` and its practice
/// stages from, and it is the only read that fills them — which is what makes this the one read
/// that has to stay pointed at the game's own file. See [`crate::score`].
///
/// The write is not bracketed and does not need to be: the whole exe reaches it from one place,
/// 0x42f5cd in the ranking screen's deleted callback (0x42f5bc, the `+0xc` beside its added).
const MAIN_MENU_ADDED: usize = 0x0043a464;
/// `push ebp; mov ebp,esp; sub esp,0x10` — position-independent, 6 bytes.
const MAIN_MENU_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x10];

/// The added callback of the screen a ranking is shown on, `__cdecl` taking the screen. Registered
/// at 0x42d7ec as the `+0x8` of the chain element that screen makes, and the read at 0x42f47f is
/// the ranking's own — see [`MAIN_MENU_ADDED`] for what each of the three reads is for.
///
/// Hooked to get in front of that read, which is the one moment the record of captures in memory
/// can be cleared without losing anything: see [`Th06::forget_captures`].
const RANKING_ADDED: usize = 0x0042f060;
/// `push ebp; mov ebp,esp; sub esp,0x40` — position-independent, 6 bytes.
const RANKING_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x40];

/// The `curState` a ranking is shown in, measured as `scene=6->1` while a player had one open.
const STATE_SCORE: i32 = 6;
/// What the title menu's `Score` item writes into [`main_menu::GAME_STATE`]: its handler at 0x437f56,
/// the fifth entry of the table at 0x4381cc, writing 0xa at 0x437f84.
const MENU_STATE_SCORE: i32 = 0xa;

/// `RESULT_SCREEN_STATE_EXITING` of the decompilation's `ResultScreenState`, which is how that screen
/// leaves: its own `case RESULT_SCREEN_STATE_EXITING` writes `g_Supervisor.curState =
/// SUPERVISOR_STATE_MAINMENU` itself (ResultScreen.cpp:1527-1535). Writing that `curState` from
/// outside instead is what left a screen up with nothing behind it and a front end built twice.
const RESULT_SCREEN_STATE_EXITING: i32 = 2;
/// The states the screen is in while it is showing something — `CHOOSING_DIFFICULTY`, the five
/// `BEST_SCORES_*`, and `SPELLCARDS` of the decompilation's `ResultScreenState`. Looked for by name
/// rather than by "not `INIT`": a screen whose added callback has just run holds a value that is
/// neither, and taking that for a built screen is what left one standing.
const RESULT_SCREEN_SHOWING: [i32; 7] = [1, 3, 4, 5, 6, 7, 8];
/// The screen itself, as the added callback was handed it: it is allocated into the chain element and
/// nowhere else, so this is where orb learns the address.
static RESULT_SCREEN: AtomicUsize = AtomicUsize::new(0);
/// What the front end's own state was before orb asked it for the ranking, plus one so that zero means
/// nothing was asked. Put back afterwards: the field is the item the front end is acting on, so leaving
/// orb's request in it left the cursor sitting on `Score` and the *next* run's end acting on it again —
/// a give-up that went to the ranking instead of the title.
static MENU_STATE_BEFORE: AtomicUsize = AtomicUsize::new(0);
/// And the item its cursor was on, plus one the same way. The state alone was not enough: the cursor
/// stayed on `Score` after a trip, and the next run given up was read as that item being chosen again.
static MENU_CURSOR_BEFORE: AtomicUsize = AtomicUsize::new(0);

/// `ResultScreen+0x8`, the state the screen a ranking is shown on is in.
const RANKING_STATE: usize = 0x8;
/// The two of those states the screen does not parse the score file it read in — compared at
/// 0x42f4e5 and 0x42f4f1, jumping over the `catk`, `clrd` and `pscr` parses at 0x42f4f7 onwards.
/// They are the states on the way out of a run, where what is in memory is that run's own record and
/// the file is about to be written from it.
const RANKING_STATES_KEEPING_THE_RECORD: [i32; 2] = [0x9, 0x11];
/// `g_GameManager.cardHistory`: the `GameManager` at 0x69bca0 plus the 0x30 its own read parses
/// `catk` into (0x41bd1e), which is the same address the ranking screen parses it into (0x42f4f7)
/// and the same one the write copies it out of (0x42b9ed).
const CARD_HISTORY: usize = 0x0069bcd0;
/// `Catk::numAttempts` of the decompilation's 0x40-byte record — the field after `unk_38`, and one of
/// the two words `GameManager::AddedCallback` clears at 0x41bcd2 and 0x41bcd5. A `u16`.
const CATK_ATTEMPTS: usize = 0x3c;
/// `Catk::base.magic`, which says a slot holds a record at all.
const CATK_MAGIC: u32 = u32::from_le_bytes(*b"CATK");
/// The card a boss is on, as both of the places the game counts one index by: `ds:0x5a5f98`, shifted
/// into [`CARD_HISTORY`] at 0x4096df and 0x409889.
const CURRENT_CARD: usize = 0x005a5f98;

/// 64 records of 0x40, which is the count the write walks and the size it copies each at.
const CARD_HISTORY_BYTES: usize = 64 * 0x40;

/// `th06::Stage::RegisterChain`, `__cdecl` taking the stage number. Called from one place in the
/// whole exe — 0x41c00d, inside the callback above — which makes it the seam between the numbers
/// being put in place and the stage being built out of them.
///
/// That is the only window a resumed run's seed can go in at. The callback above **draws from the
/// generator 2048 times before it copies the seed**: 0x41bc4f fills 64 records of 32 `u16` keys at
/// `manager+0x30` from `g_Rng`, one call to `Rng::GetRandomU16` (0x41e780) each, and every one of
/// them rewrites `g_Rng.seed` — the whole block is skipped when `curState` is
/// [`STATE_GAMEMANAGER_REINIT`] at 0x41bb1e, which is how a stage reached
/// by playing gets none of it and a stage reached from the menu gets all of it. So a seed written on
/// the way into the callback comes out 2048 draws later, and a seed written after the callback is two
/// draws late: `Stage::RegisterChain` is what draws those two, building the stage. Written here, in
/// between, is where the game's own replay effectively writes it — `AddedCallbackDemo` from 0x41bf7e,
/// after the 2048 and before `g_Rng.generationCount = 0` at 0x41bfec.
const STAGE_REGISTER_CHAIN: usize = 0x004044c0;
/// `push ebp; mov ebp,esp; sub esp,0x8; push edi` — position-independent, 7 bytes.
const STAGE_REGISTER_CHAIN_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x08, 0x57];

/// `th06::GameWindow::CreateGameWindow`, `__cdecl`. `WinMain` calls it after
/// reading the config and before creating the device, so it is the last moment
/// `cfg.windowed` can be changed and still be obeyed.
const CREATE_GAME_WINDOW: usize = 0x00420c10;
/// `push ebp; mov ebp,esp; sub esp,0x30` — position-independent, 6 bytes.
const CREATE_GAME_WINDOW_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x30];

/// `th06::GameWindow::InitD3dDevice`, `__cdecl`. Called once the device exists and
/// after every reset, before anything is presented.
const INIT_D3D_DEVICE: usize = 0x00421420;
/// `push ebp; mov ebp,esp; sub esp,0x18` — position-independent, 6 bytes.
const INIT_D3D_DEVICE_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x18];

/// `th06::GameWindow::Render`, `__thiscall`. One whole frame: draw, update, wait,
/// present.
const RENDER: usize = 0x004206e0;
/// `push ebp; mov ebp,esp; sub esp,0x64` — position-independent, 6 bytes.
const RENDER_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x64];
/// `th06::GameWindow::Present`, which resets a lost device and takes the game's
/// own screenshots.
const PRESENT: usize = 0x00420b50;
/// `th06::SoundPlayer::PlaySounds`, `__thiscall`.
const PLAY_SOUNDS: usize = 0x00431270;

/// `th06::SoundPlayer::StopBGM`, `__thiscall`. The whole teardown of a track: stops
/// the buffer, posts `WM_QUIT` to the streaming thread and joins it, closes its
/// handles, deletes the stream — which releases the sound buffer — and clears the
/// pointer. Called rather than any of that being done by hand, because the game's own
/// free is the one its allocator agrees with. Its prologue reads `backgroundMusic` at
/// +0x62c, which is what identifies it.
const STOP_BGM: usize = 0x00430f80;
/// `th06::Supervisor::PlayAudio`, `__thiscall`, taking the path the stage names its
/// track by. It swaps the extension for `.wav` and `.pos` itself, loads them, and
/// plays looping or not depending on whether the `.pos` was there — and takes the
/// MIDI branch instead when the game is configured for MIDI, which is why this is
/// the call to make rather than the three below it.
const PLAY_AUDIO: usize = 0x00424b5d;

/// `th06::Controller::GetInput`, `__stdcall`, no arguments, returns the buttons
/// held this frame. `Supervisor::OnUpdate` calls it once a frame, so it is the one
/// place every key the game sees passes through.
const GET_INPUT: usize = 0x0041d820;
/// `push ebp; mov ebp,esp; sub esp,0x110` — position-independent, 9 bytes.
const GET_INPUT_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x81, 0xec, 0x10, 0x01, 0x00, 0x00];

/// `th06::Controller::GetControllerInput`, `__cdecl`, taking the buttons read from the
/// keyboard and returning them with the joystick's added. `GetInput` ends in a call to
/// it, so its cost is inside the keyboard read as measured from outside.
const GET_CONTROLLER_INPUT: usize = 0x0041cfc0;
/// `push ebp; mov ebp,esp; sub esp,0x15c` — position-independent, 9 bytes.
const GET_CONTROLLER_INPUT_PROLOGUE: &[u8] =
    &[0x55, 0x8b, 0xec, 0x81, 0xec, 0x5c, 0x01, 0x00, 0x00];

const G_CHAIN: usize = 0x0069d918;
const G_GAME_WINDOW: usize = 0x006c6bd4;
const G_STAGE: usize = 0x00487b10;
const G_GAME_MANAGER: usize = 0x0069bca0;
const G_SUPERVISOR: usize = 0x006c6d18;
const G_ENEMY_MANAGER: usize = 0x004b79c8;
const G_BULLET_MANAGER: usize = 0x005a5ff8;
const G_PLAYER: usize = 0x006ca628;
const G_GUI: usize = 0x0069bc30;
const G_SOUND_PLAYER: usize = 0x006d3f50;
/// `ReplayManager *g_ReplayManager` — a pointer to it, not the thing. `RegisterChain`
/// tests this address against zero and then writes what `new` gave it, and a struct
/// here would run over `g_SoundPlayer` at 0x6d3f50.
const G_REPLAY_MANAGER: usize = 0x006d3f18;
/// `AnmManager *g_AnmManager` — a pointer to it, allocated once at startup.
const G_ANM_MANAGER: usize = 0x006d4588;
/// `ANM_FILE_FRONT`, the slot `data/front.anm` — the panel, its border and their labels — is
/// loaded into.
const FRONT_TEXTURE: usize = 13;
/// `MainMenu g_MainMenu`, the whole front end: the title menu, the difficulty and character
/// selects, the options, the replay list. `MainMenu::RegisterChain` memsets it and sets its
/// state afresh every time the front end is entered, so nothing in it is left over from the
/// last time round.
const G_MAIN_MENU: usize = 0x006d46c0;
/// `u16 g_CurFrameInput`, assigned from `Controller::GetInput` at the top of
/// `Supervisor::OnUpdate` and then, for a replay, overwritten with the record's buttons
/// in place of the ones on the keyboard.
const G_CUR_FRAME_INPUT: usize = 0x0069d904;
/// The buttons a run is made of, which `ReplayManager::OnUpdate` masks `g_CurFrameInput` with
/// before it writes an entry — and `OnUpdateDemo` puts back under the same mask, keeping the rest
/// of the word as it was read. Every button but 0x8, which is the one `GameManager::OnUpdate`
/// reads at 0x41b72c to open the pause menu.
const RUN_INPUT: u16 = 0x01f7;
/// The buttons the front end reads as *decide*: the shot button and return. Every one of its screens
/// tests them the same way — `g_CurFrameInput & 0x1001` against `g_LastFrameInput` under the same
/// mask, the shot type select's own at 0x436d79 and the title menu's at 0x437c1b — so a change in
/// these two, with one of them down, is what moves it on.
const MENU_DECIDE: u16 = 0x1001;
/// And as *back*: the bomb and the menu button, tested the same way at 0x436c1b and 0x438188. What it
/// does differs per screen — the shot type select goes to the character select, the title menu puts
/// its cursor on `Quit` — and both are things orb has to keep from happening on the frame one of its
/// own questions was cancelled with that very key.
const MENU_CANCEL: u16 = 0x000a;
/// The bounds inside [`G_JOY_CAPS`] that an axis is measured against, which is where the game
/// takes the centre of one and its dead zone from.
mod joy_caps {
    pub const Y_MIN: usize = 0x2c;
    pub const Y_MAX: usize = 0x30;
}

/// `JOYCAPSA g_JoyCaps`, the 0x194 bytes `joyGetDevCapsA` fills. `GetControllerInput` reads
/// `wXmin`/`wXmax` (+0x24, +0x28) and `wYmin`/`wYmax` (+0x2c, +0x30) out of it every frame to
/// place the centre of each axis and a dead zone of a quarter of its travel. This address
/// appears once in the whole exe — the one `joyGetDevCapsA` call, at startup, and only where
/// a joystick answered `joyGetPosEx` first — so nothing the game does fills it later.
const G_JOY_CAPS: usize = 0x0069d760;
/// `ItemManager g_ItemManager`, whose `Item items[513]` of 0x144 bytes each are followed
/// by `nextIndex` and then `itemCount` — 513 * 0x144 = 0x28944, and the struct is
/// 0x2894c. Nothing on the way into a stage puts it back: the bullet manager is built
/// fresh per stage and this is not part of it.
const G_ITEM_MANAGER: usize = 0x0069e268;
const ITEM_COUNT: usize = 0x28948;
/// `Rng g_Rng`: `u16 seed`, then `u32 generationCount` at +0x4.
const G_RNG: usize = 0x0069d8f8;
const RNG_GENERATION_COUNT: usize = 0x4;
// The game's own frame-rate counter is left alone: orb's numbers go in the black beside
// the game, not over it, so the two do not collide. Worth recording where its switch is,
// in case that changes: the counter is drawn unless `g_Supervisor.isInEnding` is set —
// which is the same flag orb reads to know when to run an ending out, so writing it to
// hide the counter tells the game it is in the ending for good.

mod game_manager {
    /// The score as the player sees it, which chases `score` rather than being it.
    pub const GUI_SCORE: usize = 0x0;
    pub const SCORE: usize = 0x4;
    pub const NEXT_SCORE_INCREMENT: usize = 0x8;
    pub const DIFFICULTY: usize = 0x10;
    pub const IS_IN_REPLAY: usize = 0x1c;
    pub const DEATHS: usize = 0x20;
    pub const CURRENT_POWER: usize = 0x1810;
    /// `pointItemsCollected`, which is what the end of a stage is scored on.
    pub const POINT_ITEMS: usize = 0x1816;
    /// `powerItemCountForScore`, the count a power item collected at full power is worth,
    /// which the item code holds at 30.
    pub const POWER_ITEMS: usize = 0x1819;
    pub const LIVES_REMAINING: usize = 0x181a;
    pub const BOMBS_REMAINING: usize = 0x181b;
    pub const EXTRA_LIVES: usize = 0x181c;
    /// Whose run it is and which of that character's two shots, as the character select left
    /// them: `AddedCallbackDemo` writes these two out of a replay's header — `character` from
    /// `shottypeChara / 2` and `shotType` from its low bit — and every table the game indexes by
    /// them is `character * 2 + shotType`.
    pub const CHARACTER: usize = 0x181d;
    pub const SHOT_TYPE: usize = 0x181e;
    pub const IS_IN_GAME_MENU: usize = 0x181f;
    pub const IS_IN_RETRY_MENU: usize = 0x1820;
    pub const IS_IN_PRACTICE_MODE: usize = 0x1823;
    pub const DEMO_MODE: usize = 0x1824;
    pub const RANDOM_SEED: usize = 0x1a2c;
    pub const GAME_FRAMES: usize = 0x1a30;
    /// Counts from one, not from zero: the game holds the menu's choice here — 0 for
    /// a full run, the 0-based stage for practice — and raises it as each stage
    /// starts, so while a stage is running the value is its 1-based number. Measured
    /// on stage 4 practice (`stage=3 frames=0` entering the scene, `stage=4` from
    /// frame 1 on) and on a 1→6 replay (0, then 1 through 6). `read_state` subtracts
    /// one, which also makes the frames before the first stage read as no stage at
    /// all rather than as stage 1.
    pub const CURRENT_STAGE: usize = 0x1a34;
    pub const ARCADE_REGION_TOP_LEFT: usize = 0x1a3c;
    pub const ARCADE_REGION_SIZE: usize = 0x1a44;
    /// `playerMovementAreaTopLeftPos` and `playerMovementAreaSize`, the two
    /// `D3DXVECTOR2` after the arcade region's pair.
    pub const PLAYER_AREA_TOP_LEFT: usize = 0x1a4c;
    pub const PLAYER_AREA_SIZE: usize = 0x1a54;
    /// The last four fields of the 0x1a80-byte struct — `counat`, then rank, its two
    /// bounds and the fraction between steps. `isInMenu` at 0x1821, which
    /// `GameManager::DeletedCallback` clears with the `andb $0x0,0x1821(%eax)` at
    /// 0x41c254, is what fixes the layout these are counted from.
    pub const RANK: usize = 0x1a70;
    pub const SUB_RANK: usize = 0x1a7c;
}

mod stage {
    /// `skyFog.color`, the colour the background clears to.
    pub const SKY_FOG_COLOR: usize = 0x48;
    /// `RawStageHeader *stdData`, the head of the stage's own `.std` file.
    pub const STD_DATA: usize = 0x4;
}

/// `th06::RawStageHeader`: two `i16`, three `i32`, `char stageName[128]`, then the
/// names and the paths of the tracks the stage carries.
mod stage_header {
    /// `char songPaths[4][128]`, the stage's own track first and the boss's second.
    /// Inline arrays, so the address of an entry is the string itself.
    pub const SONG_PATHS: usize = 0x290;
}

/// `LPDIRECTINPUTDEVICE8A g_Supervisor.controller`, null where the game's own
/// `EnumDevices(DI8DEVCLASS_GAMECTRL, DIEDFL_ATTACHEDONLY)` found nothing attached. Where it
/// found something, `Supervisor::RegisterChain` gives it `c_dfDIJoystick2`, takes it
/// `DISCL_EXCLUSIVE | DISCL_FOREGROUND`, and sets every axis' range to ±1000 — so a state read
/// off it is in those units and needs no calibration of orb's.
const G_CONTROLLER: usize = 0x006c6d2c;

/// The slots of `IDirectInputDevice8A`'s vtable that orb calls. Slot 7 is the one the keyboard's
/// re-acquire uses, and the game's own `*0x1c(%eax)` for it is what settled the numbering.
mod dinput_device {
    /// `IUnknown`'s third, which every COM interface begins with.
    pub const RELEASE: usize = 2;
    pub const ACQUIRE: usize = 7;
    pub const UNACQUIRE: usize = 8;
    pub const GET_DEVICE_STATE: usize = 9;
    pub const POLL: usize = 25;
}

/// `lY` — the second of the six axes, and the one a menu is driven by.
const AXIS_Y: usize = 1;

/// `DIJOYSTATE2`, the format the game set on its controller. All of it, because its size is what
/// `GetDeviceState` has to be told; the tail is the velocity, acceleration and force halves of
/// every axis, which neither the game nor orb reads.
#[repr(C)]
#[derive(Clone, Copy)]
struct JoyState {
    axes: [i32; 6],
    sliders: [i32; 2],
    hats: [u32; 4],
    buttons: [u8; 128],
    velocity: [i32; 8],
    acceleration: [i32; 8],
    force: [i32; 8],
}

/// What the game passes as the size of its state, which is the whole of that struct.
const _: () = assert!(size_of::<JoyState>() == 272);

mod supervisor {
    pub const D3D_DEVICE: usize = 0x8;
    /// `LPDIRECTINPUTDEVICE8A keyboard`, null when the game falls back to
    /// `GetKeyboardState`. Read out of `Controller::GetInput`, which tests the pointer
    /// at 0x6c6d28 before choosing which way to read the keyboard.
    pub const KEYBOARD: usize = 0x10;
    pub const HWND_GAME_WINDOW: usize = 0x44;
    pub const VIEWPORT: usize = 0xc8;
    /// `D3DPRESENT_PARAMETERS`, starting with `BackBufferWidth` and `Height`.
    pub const PRESENT_PARAMETERS: usize = 0xe0;
    /// `cfg.windowed`, inside the `GameConfiguration` at 0x114.
    pub const CFG_WINDOWED: usize = 0x132;
    /// `cfg.controllerMapping`, which is that configuration's first member, so these are its `i16`
    /// in order: shoot, bomb, focus, menu, up, down, left, right, skip. What a pad's buttons mean
    /// to the game, and the copy `Controller::GetControllerInput` itself reads every frame.
    pub const CFG_SHOOT_BUTTON: usize = 0x114;
    pub const CFG_BOMB_BUTTON: usize = 0x116;
    pub const CFG_MENU_BUTTON: usize = 0x11a;
    pub const CFG_UP_BUTTON: usize = 0x11c;
    pub const CFG_DOWN_BUTTON: usize = 0x11e;
    /// `cfg.padYAxis`, how far a stick has to go before the game counts it as pushed — in the
    /// ±1000 it gave the controller's axes. Past the mapping, the version, and the eight bytes of
    /// counts and switches: `windowed` at 0x132 is the seventh of those, which is what fixes the
    /// two axis thresholds at 0x134 and 0x136.
    pub const CFG_PAD_Y_AXIS: usize = 0x136;
    /// `wantedState`, the field before `curState`. Assigned from `curState` at the end of every
    /// `Supervisor::OnUpdate`, so the two differing is a scene change that has been asked for and
    /// not yet acted on.
    pub const WANTED_STATE: usize = 0x188;
    /// `effectiveFramerateMultiplier`, how far the game's timers advance in one
    /// update.
    pub const FRAMERATE_MULTIPLIER: usize = 0x1a8;
    pub const CUR_STATE: usize = 0x18c;
    pub const IS_IN_ENDING: usize = 0x19c;
}

mod enemy_manager {
    pub const BOSSES: usize = 0xee598;
    pub const ENEMY_COUNT: usize = 0xee5bc;
    pub const SPELLCARD_IS_ACTIVE: usize = 0xee5c8;
    pub const SPELLCARD_IDX: usize = 0xee5d0;
    pub const TIMELINE_TIME_CURRENT: usize = 0xee5e8;
}

mod enemy {
    pub const LIFE: usize = 0xce4;
    /// `bossTimer.current`, reset to zero whenever the script moves the boss on
    /// to its next attack.
    pub const BOSS_TIMER_CURRENT: usize = 0xcf8;
}

mod bullet_manager {
    pub const LASERS: usize = 0xec000;
    pub const LASER_STRIDE: usize = 0x270;
    pub const LASER_COUNT: usize = 64;
    pub const LASER_IN_USE: usize = 0x258;
    pub const BULLET_COUNT: usize = 0xf5c04;
}

mod player {
    /// `D3DXVECTOR3 positionCenter`, which `unk_44c` — the field after it, named for its
    /// offset — puts at 0x440.
    pub const POSITION_CENTER: usize = 0x440;
    pub const PLAYER_STATE: usize = 0x9e0;
    /// `invulnerabilityTimer.current`, the frames of invulnerability left. `Player::OnUpdate`
    /// ticks it down while the state is invulnerable and puts the state back to normal the
    /// moment it reaches nothing, so the state alone does not last: the timer under it is what
    /// makes it last. The respawn sets it to 240 at 0x428f81, over the `Timer` at +0x75b4 whose
    /// `current` this is.
    pub const INVULNERABLE_FRAMES: usize = 0x75bc;
    /// `bombInfo.isInUse`. `PlayerBombInfo` is 0x231c bytes and sits three pointers
    /// from the end of the 0x98f0-byte `Player`, and `isInUse` is its first field.
    /// Four places in the exe read this address as a flag — `ItemManager::OnUpdate`,
    /// `Player::ScoreGraze`, `Player::UpdateFireBulletsTimer` and one ECL
    /// instruction — which is what a bomb suppresses, so the arithmetic is not the
    /// only thing saying it is the right address.
    pub const BOMB_IN_USE: usize = 0x75c8;
}

mod gui {
    /// `GuiFlags`, five two-bit fields in one word and the first member of `Gui`. The lowest pair
    /// is the count of lives' row: while it is not zero, `Gui::OnDraw` erases that row and draws
    /// the stars again.
    pub const FLAGS: usize = 0x0;
    pub const IMPL: usize = 0x4;
    pub const BOSS_PRESENT: usize = 0x20;
    /// Inside `GuiImpl`, at `msg.currentMsgIdx`.
    pub const IMPL_CURRENT_MSG_IDX: usize = 0x253c;
}

/// `th06::ChainElem`, 0x20 bytes: the priority and the heap flag, then the three
/// callbacks, then the links. `Chain::Cut` walking `+0x14` is what says which link is
/// which.
/// `th06::AnmManager`, whose `IDirect3DTexture8 *textures[264]` is the one array of
/// Direct3D's own objects that the game replaces while a stage runs: an ECL instruction
/// loads a boss's graphics part way through, and loading releases what the slot held.
///
/// The offset is out of the crash it caused — `AnmManager::ReleaseTexture` reads
/// `0x1c110(%eax,%edx,4)` and calls the third entry of its vtable — and the count out of the
/// struct, where `imageDataArray` follows at 0x1c530, which is 264 pointers later.
mod anm_manager {
    pub const TEXTURES: usize = 0x1c110;
    pub const TEXTURE_COUNT: usize = 264;
}

mod chain_elem {
    pub const CALLBACK: usize = 0x4;
    pub const NEXT: usize = 0x14;
    /// What the callbacks are called on, written by whoever registered the job.
    pub const ARG: usize = 0x1c;
}

/// How much further into a `Chain` its draw list's head is than its calc list's. The two lists are
/// the same shape 0x20 apart: `AddToCalcChain` at 0x41c860 links from `chain + 0x14` and
/// `AddToDrawChain` at 0x41c940 from `chain + 0x20 + 0x14`, and which is which is out of the line
/// each of them logs — "add calc chain (pri = %d)" at 0x46afb8 against "add draw chain (pri = %d)"
/// at 0x46afd4. A walk from `G_CHAIN` without this is a walk of the calc list.
const CHAIN_DRAW_LIST: usize = 0x20;

/// `g_Gui`'s own element in the draw chain, a static rather than a heap job: `Gui::RegisterChain`
/// at 0x41b252 writes `Gui::OnDraw` into 0x69bc60 and `&g_Gui` into 0x69bc78, then hands 0x69bc5c to
/// `AddToDrawChain` at priority 0xb. Element + 4 and element + 0x1c, which is `chain_elem` checking
/// out against a second registration.
const GUI_DRAW_ELEM: usize = 0x0069bc5c;

/// `th06::MainMenu`, whose own fields come after an `AnmVm vm[122]` of 0x110 bytes each: 122 *
/// 0x110 is 0x81a0, and four bytes of cursor and 0x40 of padding after that lands exactly on the
/// field the decompilation names `unk_81e4` for its offset, which is the arithmetic checking out.
mod main_menu {
    /// `i32 cursor`, the item every one of the front end's screens has under its own cursor. On the
    /// shot type select that is the shot itself: 0x436d07 takes `shotType` from it on the way out and
    /// 0x436dae on the way into a run, and nothing writes `shotType` while the screen is up — so this
    /// is the only place the shot being pointed at is to be read.
    pub const CURSOR: usize = 0x81a0;
    /// `GameState gameState`, three fields further on, and the whole of where the front end is.
    pub const GAME_STATE: usize = 0x81f0;
    /// `stateTimer`, the frames the front end has been in that state. Every state that walks
    /// itself out counts this up to a number of its own.
    pub const STATE_TIMER: usize = 0x81f4;
}

/// `th06::ResultScreen`, 0x56b0 bytes on the heap, reached through the job it registers. Its
/// `unk_3c` and `unk_40` are named for their offsets, which is what fixes this one.
mod result_screen {
    /// `i32 resultScreenState`, after the score file it opened and its frame timer.
    pub const STATE: usize = 0x8;
}

/// `th06::Ending`, 0x1170 bytes on the heap, reached through the job it registers.
mod ending {
    /// The `.end` script as it was read. `Ending::LoadEndingFile` at 0x4106d0 reads the new
    /// file before freeing the one it replaces, so the address is a different one every time
    /// a script hands over to another.
    pub const SCRIPT: usize = 0x1114;
}

mod replay_manager {
    /// `ReplayData *replayData`, the file as it was read.
    pub const REPLAY_DATA: usize = 0x4;
    /// `i32 isDemo`, set when the manager is playing a replay back rather than
    /// recording one. The game's own name for it: the attract demo and a replay the
    /// player chose are the same thing to it.
    pub const IS_DEMO: usize = 0x8;
}

/// `th06::ReplayData`, whose `StageReplayData *stageReplayData[7]` is its last field:
/// seven pointers at the end of 0x50 bytes puts them at 0x34. A null one is a stage the
/// replay does not cover.
mod replay_data {
    pub const STAGE_DATA: usize = 0x34;
    pub const STAGES: i32 = 7;
}

mod sound_player {
    /// `CStreamingSound *backgroundMusic`.
    pub const BACKGROUND_MUSIC: usize = 0x62c;
    pub const BACKGROUND_MUSIC_THREAD_ID: usize = 0x614;
    /// `HANDLE backgroundMusicThreadHandle`. What `StopBGM` tests before it goes near
    /// the thread and its handles, so clearing it is what makes that call a no-op.
    pub const BACKGROUND_MUSIC_THREAD_HANDLE: usize = 0x618;
    pub const SIZE: usize = 0x638;
}

/// `CStreamingSound`, laid out as `CSound` then its own fields, after the vtable
/// pointer its virtual destructor puts first.
mod streaming_sound {
    pub const BUFFERS: usize = 0x04;
    pub const BUFFER_SIZE: usize = 0x08;
    pub const WAVE_FILE: usize = 0x0c;
    pub const BUFFER_COUNT: usize = 0x10;
    pub const NOTIFY_SIZE: usize = 0x28;
    pub const NEXT_WRITE_OFFSET: usize = 0x2c;
    pub const SIZE: usize = 0x34;
}

mod wave_file {
    pub const MMIO: usize = 0x04;
    /// `m_ck.cksize`, the size of the `data` chunk as `mmioDescend` left it — which
    /// `WaveFile::Read` (0x43c080) uses as how much is left to read, clamping each read to it and
    /// subtracting what it read. The stream loops when a read comes up short against it, so it is
    /// the countdown to the track's loop and not a size at all after the first read.
    pub const BYTES_LEFT: usize = 0x0c;
    /// `m_dwSize`, the length of the wave file.
    pub const SIZE_OF_FILE: usize = 0x30;
    pub const LOOP_START: usize = 0x90;
    pub const LOOP_END: usize = 0x94;
    pub const SIZE: usize = 0x98;
}

/// `th06::SupervisorState`.
const STATE_MAINMENU: i32 = 1;
const STATE_GAMEMANAGER: i32 = 2;
/// The state between two stages of a run, where the game tears the last stage's
/// managers down and builds the next one's. Not the run ending, which is what makes it
/// worth telling apart.
const STATE_GAMEMANAGER_REINIT: i32 = 3;
/// The result screen a run has just ended into, which is the only one of the two that offers to
/// save a replay. The other — 6, reached from the ranking on the title menu — has no run behind
/// it to record.
const STATE_RESULTSCREEN_FROMGAME: i32 = 7;
const STATE_ENDING: i32 = 10;

/// `th06::GameState`, the front end's own state, of which orb watches two.
///
/// `MainMenu::OnUpdate` switches on it through the table at 0x4374d0, one entry per state, and the
/// two below are entry 2 and entry 11 of it.
///
/// The title menu with its eight items live. Its own handler is a function of its own, 0x437b41,
/// called from the state's block at 0x435972.
const MENU_STATE_TITLE: i32 = 2;
/// The shot type select, where a run's third and last question is answered and so the first screen
/// that knows which run is about to be played. 0x436a7c takes the shot from the cursor at 0x436d07
/// and 0x436dae; the character select — entry 9, 0x4364e0 — is what sets this state, at 0x4368fd,
/// right after writing the character it was asked for.
const MENU_STATE_SHOT_TYPE: i32 = 0xb;

/// Which item of the title menu is which, as its cursor counts them: `MainMenu+0x81a0` bounded to
/// 0..=7 at 0x437c5c and jumped through the table at 0x4381cc.
///
/// The three that start a run set `gameState` to 6, the difficulty load — 0x437c9e, 0x437d9d and
/// 0x437e4b, one per item — and the ranking sets 10 at 0x437f84. The four not named here are the
/// replay, the music room, the options and quitting, none of which orb has anything to ask about.
const TITLE_ITEM_START: i32 = 0;
const TITLE_ITEM_EXTRA: i32 = 1;
const TITLE_ITEM_PRACTICE: i32 = 2;
const TITLE_ITEM_SCORE: i32 = 4;

/// The frames each of the two screens ignores its own decide for, counted in `stateTimer`: the title
/// menu at 0x437c0e and the shot type select at 0x436c0a.
///
/// What they are needed for is that orb holds the decide back on both — a press it read where the
/// screen would have ignored it is a press the screen then acts on when it is handed back, which is
/// a run started by a keypress the game had thrown away.
const MENU_TITLE_GRACE_FRAMES: i32 = 0x14;
const MENU_SHOT_TYPE_GRACE_FRAMES: i32 = 0x1e;
/// And how far ahead of either of them orb starts holding: one frame.
///
/// What decides the holding is read after the game's update and applies to the *next* read, so a screen
/// tested for the frame it is on leaves the frame its grace runs out on unheld — and a press on that
/// frame is one the game acts on and orb never saw. The frame the other way costs nothing: answering a
/// question takes ten frames of its own grace at least, so the press handed back lands well past
/// either.
const MENU_GRACE_LOOKAHEAD: i32 = 1;

/// `th06::ResultScreenState`: the screen that asks whether to save a replay, and the state that
/// leaves for the title menu without any of that.
///
/// `RESULT_STATE_EXIT` is the game's own way out — it is what a practice run's result screen is
/// registered in, and its `OnUpdate` case sets the supervisor back to the title menu and takes
/// the job out. So the score file is still written on the way, by `DeletedCallback`.
const RESULT_STATE_SAVE_REPLAY_QUESTION: i32 = 10;
const RESULT_STATE_EXIT: i32 = 17;

/// `th06::PlayerState`.
const PLAYER_NORMAL: i8 = 0;
const PLAYER_SPAWNING: i8 = 1;
const PLAYER_DEAD: i8 = 2;
/// The state a bomb and the end of a respawn put the player in — written by all four bombs
/// (0x405577, 0x4065a2, 0x406ac7, 0x407174) and at 0x428f38 once the respawn has faded in.
///
/// Written rather than the hit test being patched out, because the game already has a state
/// for this and everything around it agrees on what it means: `Player::Kill` at 0x427770 is
/// reached from both collision tests only where the state is 0, while firing (0x4299e4) and
/// collecting (0x426fe5) take 0 and this alike. Patching the test would leave a state nothing
/// else in the game expects.
const PLAYER_INVULNERABLE: i8 = 3;
/// What the game gives a respawn, and what orb writes under the state above every update.
///
/// It has to be written and not only the state: `Player::OnUpdate` runs at chain priority 7 and
/// the bullets are checked at 11, so frames left at nothing — where the last respawn left
/// them — is a state put back to normal *before* the hit test, in the same update it was written
/// for. Measured with the state alone: `died in chapter 1` 235ms after `stage 1 chapter 1 (stage
/// start)`, and again after each of the two retries.
///
/// 240 rather than something that could not run out, because the frames left are also the blink:
/// 0x42905d draws the player dark where `current & 7` is under 2, and a value refreshed to 240
/// every update is seen as 239 there, which is not.
const PLAYER_INVULNERABLE_FRAMES: i32 = 0xf0;

/// `th06::Difficulty`, whose last value the Extra stage runs at.
const DIFFICULTY_EXTRA: i32 = 4;
/// What the game counts as already paid for there, so that the four scores an extra life
/// costs in a run are not paid again in a stage that is not one.
const EXTRA_LIVES_IN_EXTRA: i8 = 4;

impl Game for Th06 {
    fn hooks(&self) -> Hooks {
        Hooks {
            update: Patch {
                target: RUN_CALC_CHAIN,
                prologue: RUN_CHAIN_PROLOGUE,
            },
            draw: Patch {
                target: RUN_DRAW_CHAIN,
                prologue: RUN_CHAIN_PROLOGUE,
            },
            save_replay: Some(Patch {
                target: SAVE_REPLAY,
                prologue: SAVE_REPLAY_PROLOGUE,
            }),
            stop_recording: Some(Patch {
                target: STOP_RECORDING,
                prologue: STOP_RECORDING_PROLOGUE,
            }),
            stage_begun: Some(Patch {
                target: GAME_MANAGER_ADDED,
                prologue: GAME_MANAGER_ADDED_PROLOGUE,
            }),
            stage_building: Some(Patch {
                target: STAGE_REGISTER_CHAIN,
                prologue: STAGE_REGISTER_CHAIN_PROLOGUE,
            }),
            unlocks_read: Some(Patch {
                target: MAIN_MENU_ADDED,
                prologue: MAIN_MENU_ADDED_PROLOGUE,
            }),
            ranking_read: Some(Patch {
                target: RANKING_ADDED,
                prologue: RANKING_ADDED_PROLOGUE,
            }),
            create_window: Some(Patch {
                target: CREATE_GAME_WINDOW,
                prologue: CREATE_GAME_WINDOW_PROLOGUE,
            }),
            init_device: Some(Patch {
                target: INIT_D3D_DEVICE,
                prologue: INIT_D3D_DEVICE_PROLOGUE,
            }),
            render: Some(Patch {
                target: RENDER,
                prologue: RENDER_PROLOGUE,
            }),
            input: Some(Patch {
                target: GET_INPUT,
                prologue: GET_INPUT_PROLOGUE,
            }),
            joystick: Some(Patch {
                target: GET_CONTROLLER_INPUT,
                prologue: GET_CONTROLLER_INPUT_PROLOGUE,
            }),
        }
    }

    unsafe fn read_state(&self) -> State {
        unsafe {
            let scene: i32 = mem::read(G_SUPERVISOR + supervisor::CUR_STATE);
            let demo = mem::read::<u8>(G_GAME_MANAGER + game_manager::DEMO_MODE) != 0;
            let replay = mem::read::<u32>(G_GAME_MANAGER + game_manager::IS_IN_REPLAY) != 0;
            let player_state: i8 = mem::read(G_PLAYER + player::PLAYER_STATE);
            let boss = mem::read::<usize>(G_ENEMY_MANAGER + enemy_manager::BOSSES);
            let spellcard_active =
                mem::read::<i32>(G_ENEMY_MANAGER + enemy_manager::SPELLCARD_IS_ACTIVE) != 0;
            let in_ending = scene == STATE_ENDING
                || mem::read::<i32>(G_SUPERVISOR + supervisor::IS_IN_ENDING) != 0;

            State {
                scene,
                wanted: mem::read(G_SUPERVISOR + supervisor::WANTED_STATE),
                playing: scene == STATE_GAMEMANAGER,
                in_run: scene == STATE_GAMEMANAGER || scene == STATE_GAMEMANAGER_REINIT,
                in_game: scene == STATE_GAMEMANAGER && !demo && !replay,
                in_ending,
                // Asked for only while an ending is running: it is a walk of the job chain,
                // and there is no ending to find one in on any other frame.
                ending_script: in_ending.then(ending_script).flatten(),
                demo,
                replay,
                practice: mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_PRACTICE_MODE) != 0,
                // `isInMenu` is set while the game is *running*; the pause and
                // retry menus are the two flags that mean it has stopped.
                paused: mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_GAME_MENU) != 0
                    || mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_RETRY_MENU) != 0,
                // Being invulnerable after a bomb or a respawn does not count.
                unsettled: player_state == PLAYER_DEAD || player_state == PLAYER_SPAWNING,
                bombing: mem::read::<u32>(G_PLAYER + player::BOMB_IN_USE) != 0,
                in_dialogue: dialogue_msg_idx() >= 0,
                stage: mem::read::<i32>(G_GAME_MANAGER + game_manager::CURRENT_STAGE) - 1,
                difficulty: mem::read(G_GAME_MANAGER + game_manager::DIFFICULTY),
                stage_frames: mem::read(G_GAME_MANAGER + game_manager::GAME_FRAMES),
                script_frames: mem::read(G_ENEMY_MANAGER + enemy_manager::TIMELINE_TIME_CURRENT),
                random_seed: mem::read(G_GAME_MANAGER + game_manager::RANDOM_SEED),
                deaths: mem::read(G_GAME_MANAGER + game_manager::DEATHS),
                lives: mem::read(G_GAME_MANAGER + game_manager::LIVES_REMAINING),
                bombs: mem::read(G_GAME_MANAGER + game_manager::BOMBS_REMAINING),
                power: mem::read(G_GAME_MANAGER + game_manager::CURRENT_POWER),
                enemy_count: mem::read(G_ENEMY_MANAGER + enemy_manager::ENEMY_COUNT),
                bullet_count: mem::read(G_BULLET_MANAGER + bullet_manager::BULLET_COUNT),
                laser_count: laser_count(),
                boss_present: mem::read::<u8>(G_GUI + gui::BOSS_PRESENT) != 0,
                boss_life: (boss != 0).then(|| mem::read(boss + enemy::LIFE)),
                boss_attack_frames: (boss != 0)
                    .then(|| mem::read(boss + enemy::BOSS_TIMER_CURRENT)),
                spellcard: spellcard_active
                    .then(|| mem::read(G_ENEMY_MANAGER + enemy_manager::SPELLCARD_IDX)),
            }
        }
    }

    unsafe fn window(&self) -> Hwnd {
        unsafe { mem::read(G_SUPERVISOR + supervisor::HWND_GAME_WINDOW) }
    }

    unsafe fn d3d_device(&self) -> *mut Device {
        unsafe { mem::read(G_SUPERVISOR + supervisor::D3D_DEVICE) }
    }

    /// Every step is a pointer out of a structure the game rebuilds between
    /// stages, so each one is checked before being followed: the allocator does
    /// not scrub freed blocks, and following a stale pointer into DirectSound
    /// crashes.
    fn music(&self) -> Option<Music> {
        /// A streaming buffer holds a fraction of a second of audio; anything
        /// near this is not a buffer size.
        const MAX_BUFFER_SIZE: u32 = 8 << 20;

        let streaming = self.streaming_sound()?;
        let buffers = mem::read_committed::<usize>(streaming + streaming_sound::BUFFERS)?;
        let buffer_size = mem::read_committed::<u32>(streaming + streaming_sound::BUFFER_SIZE)?;
        let count = mem::read_committed::<u32>(streaming + streaming_sound::BUFFER_COUNT)?;
        if count != 1 || buffer_size == 0 || buffer_size > MAX_BUFFER_SIZE {
            return None;
        }
        let buffer = mem::read_committed::<usize>(buffers)?;
        if !mem::vtable_in_image(buffer) {
            return None;
        }
        let wave_file = mem::read_committed::<usize>(streaming + streaming_sound::WAVE_FILE)?;
        let mmio = mem::read_committed(wave_file + wave_file::MMIO).unwrap_or(0);
        Some(Music {
            stream: streaming,
            buffer: buffer as *mut SoundBuffer,
            buffer_size,
            notify_size: mem::read_committed(streaming + streaming_sound::NOTIFY_SIZE)?,
            mmio,
            bytes_left: wave_file + wave_file::BYTES_LEFT,
            write_offset: streaming + streaming_sound::NEXT_WRITE_OFFSET,
        })
    }

    /// The track's length and loop points together, which differ between tracks
    /// and stay put while one is playing.
    fn music_identity(&self) -> Option<u32> {
        let wave_file = self.wave_file()?;
        let length = mem::read_committed::<u32>(wave_file + wave_file::SIZE_OF_FILE)?;
        let start = mem::read_committed::<u32>(wave_file + wave_file::LOOP_START)?;
        let end = mem::read_committed::<u32>(wave_file + wave_file::LOOP_END)?;
        Some(length ^ start.rotate_left(11) ^ end.rotate_left(22))
    }

    fn audio_thread(&self) -> Option<u32> {
        let id =
            unsafe { mem::read::<u32>(G_SOUND_PLAYER + sound_player::BACKGROUND_MUSIC_THREAD_ID) };
        (id != 0).then_some(id)
    }

    unsafe fn stop_music(&self) {
        let stop: unsafe extern "fastcall" fn(usize) = unsafe { std::mem::transmute(STOP_BGM) };
        unsafe { stop(G_SOUND_PLAYER) };
        log!("music: stopped through the game");
    }

    unsafe fn restart_stage_music(&self) -> bool {
        // Cleared before the game is asked for anything, because the first thing
        // `LoadWav` does is stop whatever is playing — and what the restored state
        // says is playing was deleted long ago, sound buffer and all.
        unsafe {
            mem::write::<usize>(G_SOUND_PLAYER + sound_player::BACKGROUND_MUSIC, 0);
            mem::write::<usize>(
                G_SOUND_PLAYER + sound_player::BACKGROUND_MUSIC_THREAD_HANDLE,
                0,
            );
        }
        let Some((path, name)) = self.stage_song() else {
            return false;
        };
        log!("music: restarting {name}");
        // `__thiscall` with an argument, which unlike the one-argument case is not
        // `fastcall`: the argument goes on the stack and the callee takes it off.
        let play: unsafe extern "thiscall" fn(usize, usize) -> i32 =
            unsafe { std::mem::transmute(PLAY_AUDIO) };
        unsafe { play(G_SUPERVISOR, path) };
        true
    }

    fn audio_state(&self) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        ranges.push(G_SOUND_PLAYER..G_SOUND_PLAYER + sound_player::SIZE);
        if let Some(streaming) = self.streaming_sound() {
            ranges.push(streaming..streaming + streaming_sound::SIZE);
        }
        if let Some(wave_file) = self.wave_file() {
            ranges.push(wave_file..wave_file + wave_file::SIZE);
        }
        ranges
    }

    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        // The surfaces and the vertex buffer beside it are left out: the game releases those
        // when it loses the device, which is not something a stage runs into, and a range
        // left out of a restore is a range the snapshot no longer describes.
        mem::read_committed::<usize>(G_ANM_MANAGER)
            .filter(|manager| *manager != 0)
            .map(|manager| {
                let textures = manager + anm_manager::TEXTURES;
                // One range, said as one: the textures are contiguous, and everything else the
                // manager holds is deliberately left out — see above.
                std::iter::once(
                    textures..textures + anm_manager::TEXTURE_COUNT * size_of::<usize>(),
                )
                .collect()
            })
            .unwrap_or_default()
    }

    /// From `GAME_REGION_*`, inside the game's 640x480 output.
    fn play_area(&self) -> Rect {
        Rect {
            left: 32.0,
            top: 16.0,
            width: 384.0,
            height: 448.0,
        }
    }

    /// `front.anm`'s sprite 5 — 32x32 at (0, 224) of a 256x256 sheet — which `Gui::OnDraw` lays
    /// from (416, 0) every 32 pixels over the whole panel and border. The sheet is the texture
    /// `LoadAnm` put in slot 13, `ANM_FILE_FRONT`, of the manager's own array.
    ///
    /// It stops laying them 250 frames into a stage, where the vm's script reaches `ExitHide`, and
    /// after that nothing repaints the panel — which is exactly why orb has to be able to paint it
    /// itself, and why painting it with the game's own tile rather than a colour of orb's own
    /// matters: what is left behind is then the panel and not a patch.
    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        // A pointer to the manager, not the manager: the same chase `live_handles` makes, and
        // adding the offset without it reads whatever lies past the pointer instead — which is a
        // texture orb does not have, so the strips fell back to a flat colour and looked like the
        // patch this exists to avoid.
        let manager = mem::read_committed::<usize>(G_ANM_MANAGER).filter(|it| *it != 0)?;
        let at = manager + anm_manager::TEXTURES + FRONT_TEXTURE * size_of::<usize>();
        let texture = mem::read_committed::<usize>(at).filter(|texture| *texture != 0)?;
        let sheet = 256.0;
        Some(PanelTile {
            texture: texture as *mut Texture,
            uv: [0.0, 224.0 / sheet, 32.0 / sheet, 256.0 / sheet],
            origin: (416.0, 0.0),
            pitch: 32.0,
        })
    }

    /// `Gui::OnDraw` erases the count's row and draws the stars again only while `Gui`'s own
    /// `flags.flag0` is not zero — the test at 0x41a3cb for the bar and 0x41a5eb for the stars — and
    /// takes one off it at the end of the same draw, at 0x41acdb. Whatever changes the count sets it
    /// to 2: a death, an extend, an item. So two is what orb writes, every frame it draws the mark:
    /// the game then repaints that row for orb, background and stars, and the two bits cost nothing
    /// else.
    ///
    /// One was tried, so that no repaint is left standing for the frame after the last marked one.
    /// It is not what leaves the count on the panel of a run being left — see `draws_lives_row`,
    /// which is — and a stage's first 250 frames make it moot anyway: the panel being laid sets
    /// every one of these fields to 2 itself, at 0x41a2b6.
    ///
    /// The rest of the flags are left alone: they are the other rows of the panel, two bits each,
    /// in one word at `Gui + 0`.
    unsafe fn repaint_lives_row(&self) {
        let flags = unsafe { mem::read::<u32>(G_GUI + gui::FLAGS) };
        unsafe { mem::write::<u32>(G_GUI + gui::FLAGS, flags & !0b11 | 0b10) };
    }

    /// Whether the game will paint the row the lives are in this frame, which is its `Gui`'s own
    /// draw job being registered: the stars are drawn in one place in the whole exe, the loop at
    /// 0x41a622 inside what that job calls.
    ///
    /// What this is for: a run being left is not a run whose panel has gone. `esc` and then やめる
    /// has `StageMenu::OnUpdateGameMenu` write `curState = MAINMENU`, and orb's own bookkeeping ends
    /// the run on that frame — `run ended after 8 retries` and `f20724 scene=1` together in the log —
    /// while the panel stays on the screen until the front end has drawn its own. The game painting
    /// that row once more with the mark stopped is the count back on the panel, which is what was
    /// seen: the stars, plain, for as long as the fade to the title took.
    unsafe fn draws_lives_row(&self) -> bool {
        chain_holds(CHAIN_DRAW_LIST, GUI_DRAW_ELEM)
    }

    /// `Gui::OnDraw` draws one 16x16 star per life from (496, 122) rightwards, 16 pixels
    /// apart — the loop at 0x41a622, against the constants 496.0 at 0x46ac50, 122.0 at
    /// 0x46ac44 and the 16.0 step at 0x46a2b4. Every sprite of the panel runs
    /// `AnchorTopLeft` in `front.anm`, so those are corners and not middles.
    ///
    /// 144 wide, which is 496 to the right edge of the output: that is the bar the game
    /// erases the row with before it redraws the stars, `front.anm`'s 144x16 sprite drawn at
    /// the row's own position, so it is exactly the part of the panel the game paints itself.
    fn lives_row(&self) -> Rect {
        Rect {
            left: 496.0,
            top: 122.0,
            width: 144.0,
            height: 16.0,
        }
    }

    /// `Unacquire` then `Release` then the pointer cleared, which is what `Supervisor::RegisterChain`
    /// itself does where the device it just created cannot be set up — so it is a state the game
    /// already knows how to be in. `Controller::GetInput` then takes its `GetKeyboardState` branch,
    /// and the two places that would touch the device again — the shutdown's `Unacquire` and
    /// `Release` — test the pointer first.
    unsafe fn take_sent_keys(&self) -> bool {
        let device: usize = unsafe { mem::read(G_SUPERVISOR + supervisor::KEYBOARD) };
        if device == 0 {
            return false;
        }
        let vtable: usize = unsafe { mem::read(device) };
        let slot = |index: usize| unsafe {
            let address: usize = mem::read(vtable + index * size_of::<usize>());
            std::mem::transmute::<usize, unsafe extern "system" fn(usize) -> i32>(address)
        };
        unsafe {
            slot(dinput_device::UNACQUIRE)(device);
            slot(dinput_device::RELEASE)(device);
            mem::write::<usize>(G_SUPERVISOR + supervisor::KEYBOARD, 0);
        }
        true
    }

    fn joystick_calibration(&self) -> Option<usize> {
        Some(G_JOY_CAPS)
    }

    fn content_size(&self) -> (u32, u32) {
        let present = G_SUPERVISOR + supervisor::PRESENT_PARAMETERS;
        let size = unsafe { (mem::read::<u32>(present), mem::read::<u32>(present + 4)) };
        // Before the device exists the parameters are zero; the game renders at
        // its own size whatever the back buffer ends up being.
        if size.0 == 0 || size.1 == 0 {
            BACK_BUFFER
        } else {
            size
        }
    }

    fn windowed(&self) -> bool {
        unsafe { mem::read::<u8>(G_SUPERVISOR + supervisor::CFG_WINDOWED) != 0 }
    }

    unsafe fn force_windowed(&self) {
        unsafe { mem::write::<u8>(G_SUPERVISOR + supervisor::CFG_WINDOWED, 1) };
    }

    /// What `GameManager::OnUpdate` does at the top of every frame. Without it,
    /// a frame drawn while the update is held back goes through whatever viewport
    /// was left over — the full 640x480 — so bullets appear outside the play area
    /// and stay there, because nothing repaints the border.
    unsafe fn set_play_viewport(&self, device: *mut Device) {
        unsafe {
            let top_left =
                mem::read::<[f32; 2]>(G_GAME_MANAGER + game_manager::ARCADE_REGION_TOP_LEFT);
            let size = mem::read::<[f32; 2]>(G_GAME_MANAGER + game_manager::ARCADE_REGION_SIZE);
            let viewport = Viewport {
                x: top_left[0] as u32,
                y: top_left[1] as u32,
                width: size[0] as u32,
                height: size[1] as u32,
                min_z: 0.5,
                max_z: 1.0,
            };
            mem::write(G_SUPERVISOR + supervisor::VIEWPORT, viewport);

            let vtable = &*(*device).vtable;
            (vtable.set_viewport)(device, &viewport);
            (vtable.clear)(device, 0, std::ptr::null(), D3DCLEAR_ZBUFFER, 0, 1.0, 0);
        }
    }

    fn chain(&self) -> *mut std::ffi::c_void {
        G_CHAIN as *mut std::ffi::c_void
    }

    unsafe fn clears_back_buffer(&self) -> bool {
        self.clears_background()
    }

    unsafe fn acquire_input(&self) -> bool {
        let device: usize = unsafe { mem::read(G_SUPERVISOR + supervisor::KEYBOARD) };
        // No DirectInput device: the game is reading the keyboard through
        // `GetKeyboardState`, which the system never takes away.
        if device == 0 {
            return true;
        }
        let vtable: usize = unsafe { mem::read(device) };
        // `IDirectInputDevice8::Acquire`, slot 7 — the same slot the game's own
        // device-lost branch calls as `*0x1c(%eax)`.
        let acquire: unsafe extern "system" fn(usize) -> i32 =
            unsafe { std::mem::transmute(mem::read::<usize>(vtable + 7 * size_of::<usize>())) };
        let acquired = unsafe { acquire(device) };
        // `S_FALSE` for a device that was already acquired, which is a success.
        acquired >= 0
    }

    /// What the game does before its update chain: the viewport over the whole
    /// output, and, for the options that ask for it, a clear of the background.
    unsafe fn prepare_frame(&self, device: *mut Device) {
        unsafe {
            let vtable = &*(*device).vtable;
            let viewport = Viewport {
                x: 0,
                y: 0,
                width: BACK_BUFFER.0,
                height: BACK_BUFFER.1,
                min_z: 0.0,
                max_z: 1.0,
            };
            if self.clears_background() {
                (vtable.set_viewport)(device, &viewport);
                let fog = mem::read::<u32>(G_STAGE + stage::SKY_FOG_COLOR);
                (vtable.clear)(
                    device,
                    0,
                    std::ptr::null(),
                    D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER,
                    fog,
                    1.0,
                    0,
                );
            }
            mem::write(G_SUPERVISOR + supervisor::VIEWPORT, viewport);
            (vtable.set_viewport)(device, &viewport);
            // One update is one tick: the loop above runs the logic exactly once
            // per frame, however fast the frames are being produced.
            mem::write::<f32>(G_SUPERVISOR + supervisor::FRAMERATE_MULTIPLIER, 1.0);
        }
    }

    fn frame_calls(&self) -> FrameCalls {
        FrameCalls {
            play_sounds: Call {
                function: PLAY_SOUNDS,
                this: G_SOUND_PLAYER,
            },
            present: Call {
                function: PRESENT,
                this: G_GAME_WINDOW,
            },
        }
    }

    unsafe fn jump_to_stage(&self, stage: i32) -> bool {
        if !(0..replay_data::STAGES).contains(&stage) {
            return false;
        }
        // Asked of the replay first. `GameManager::RegisterChain` reads the stage's
        // record to put the run back, and a stage the replay does not cover fails that
        // load — which drops the game to its main menu rather than saying so.
        let entry = replay_data::STAGE_DATA + stage as usize * size_of::<usize>();
        let recorded = mem::read_committed::<usize>(G_REPLAY_MANAGER)
            .filter(|manager| *manager != 0)
            .and_then(|manager| mem::read_committed::<usize>(manager + replay_manager::REPLAY_DATA))
            .filter(|data| *data != 0)
            .and_then(|data| mem::read_committed::<usize>(data + entry));
        if recorded.unwrap_or(0) == 0 {
            return false;
        }
        // The run's score goes back to nothing first, which is the one thing a stage
        // move has to do that a stage transition does not. `GameManager::RegisterChain`
        // zeroes the score only when it is *not* reinitialising, and reinitialising is
        // the path a stage move takes; `ReplayManager::AddedCallbackDemo` then puts the
        // score back from the stage before the one being started, which the first stage
        // does not have. So stage 1 would otherwise begin with the score the run was
        // left on — 7417420 in the log at 303310937ms — and cross the first extra life's
        // 10000000 part way through it: a life the recording never got, and with it the
        // `IncreaseSubrank(200)` that took rank from 21 to 23. Rank is what the enemies
        // read, so from there the stage was a different one, and the recorded inputs
        // walked the player into it.
        //
        // `extraLives` goes with it, and to nothing rather than to what the score says:
        // the count only ever rises, so the stage's own `while` loop can raise it from
        // zero to what the restored score has paid for, and cannot lower it from what a
        // later stage had reached.
        let difficulty = unsafe { mem::read::<i32>(G_GAME_MANAGER + game_manager::DIFFICULTY) };
        let extras = if difficulty < DIFFICULTY_EXTRA {
            0
        } else {
            EXTRA_LIVES_IN_EXTRA
        };
        unsafe { self.cut_screen_shake() };
        // What the replay menu writes, and nothing more: the stage counted as the menu
        // counts it — `GameManager::RegisterChain` raises the number by one, so it is
        // handed the stage before the one meant — and the state that makes the
        // supervisor cut this stage's chain and register the next.
        unsafe {
            mem::write::<u32>(G_GAME_MANAGER + game_manager::GUI_SCORE, 0);
            mem::write::<u32>(G_GAME_MANAGER + game_manager::SCORE, 0);
            mem::write::<u32>(G_GAME_MANAGER + game_manager::NEXT_SCORE_INCREMENT, 0);
            mem::write::<i8>(G_GAME_MANAGER + game_manager::EXTRA_LIVES, extras);
            mem::write::<i32>(G_GAME_MANAGER + game_manager::CURRENT_STAGE, stage);
            mem::write::<i32>(
                G_SUPERVISOR + supervisor::CUR_STATE,
                STATE_GAMEMANAGER_REINIT,
            );
        }
        log!(
            "stage {}: asked the game to start the replay there",
            stage + 1
        );
        true
    }

    /// The reinit state counts too: a stage move is where the frame counter is about to be put
    /// back to nothing, and the frames either side of it belong to the stage whose counter this
    /// still is.
    unsafe fn stage_frame(&self) -> Option<u32> {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + supervisor::CUR_STATE) };
        (scene == STATE_GAMEMANAGER || scene == STATE_GAMEMANAGER_REINIT)
            .then(|| unsafe { mem::read(G_GAME_MANAGER + game_manager::GAME_FRAMES) })
    }

    fn run_input(&self) -> u16 {
        RUN_INPUT
    }

    fn menu_decide(&self) -> u16 {
        MENU_DECIDE
    }

    fn menu_cancel(&self) -> u16 {
        MENU_CANCEL
    }

    unsafe fn menu_takes_a_press(&self) -> bool {
        if unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) } != STATE_MAINMENU {
            return false;
        }
        let grace = match unsafe { mem::read::<i32>(G_MAIN_MENU + main_menu::GAME_STATE) } {
            MENU_STATE_TITLE => MENU_TITLE_GRACE_FRAMES,
            MENU_STATE_SHOT_TYPE => MENU_SHOT_TYPE_GRACE_FRAMES,
            // Every other state either has no decide to read or is not one orb asks anything over.
            _ => return false,
        };
        let timer: i32 = unsafe { mem::read(G_MAIN_MENU + main_menu::STATE_TIMER) };
        timer >= grace - MENU_GRACE_LOOKAHEAD
    }

    unsafe fn run_start(&self) -> RunStart {
        unsafe {
            RunStart {
                difficulty: mem::read(G_GAME_MANAGER + game_manager::DIFFICULTY),
                character: mem::read::<u8>(G_GAME_MANAGER + game_manager::CHARACTER).into(),
                shot_type: mem::read::<u8>(G_GAME_MANAGER + game_manager::SHOT_TYPE).into(),
                practice: mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_PRACTICE_MODE) != 0,
                // The game counts it from one while a stage runs — see `CURRENT_STAGE` — and orb
                // counts from zero, which is also what it has to be written back as.
                stage: mem::read::<i32>(G_GAME_MANAGER + game_manager::CURRENT_STAGE) - 1,
            }
        }
    }

    /// Both of the supervisor's states, for the reason [`Th06::menu_pointed_at`] gives: the frame a
    /// run leaves for the front end ends with `curState` already saying front end and `gameState` still
    /// saying whatever screen that run was entered from, which for every run is this one. Without
    /// `wantedState` the mark flashes on that frame and the shot button is taken out of the next read.
    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        let settled = unsafe {
            mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) == STATE_MAINMENU
                && mem::read::<i32>(G_SUPERVISOR + supervisor::WANTED_STATE) == STATE_MAINMENU
                && mem::read::<i32>(G_MAIN_MENU + main_menu::GAME_STATE) == MENU_STATE_SHOT_TYPE
        };
        if !settled {
            return None;
        }
        let practice =
            unsafe { mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_PRACTICE_MODE) } != 0;
        if practice {
            return None;
        }
        Some(RunStart {
            difficulty: unsafe { mem::read(G_GAME_MANAGER + game_manager::DIFFICULTY) },
            character: unsafe { mem::read::<u8>(G_GAME_MANAGER + game_manager::CHARACTER) }.into(),
            shot_type: unsafe { mem::read::<i32>(G_MAIN_MENU + main_menu::CURSOR) },
            practice: false,
            // A full run's chapter can be in any of them, and which one is what the mark says rather
            // than part of naming it.
            stage: 0,
        })
    }

    /// `th06::Difficulty` and the four shots, named as the game's own screens name them and written
    /// as a file can be. Anything outside them is said as the number it is rather than guessed at,
    /// since a run nothing recognises is one somebody should be able to see is wrong.
    ///
    /// **A practice run has none**, which is the whole of why none is kept: nothing to name is nothing
    /// to write, offer or mark. It is one stage played on its own, started again from the game's own
    /// menu in less time than a playback takes — and a name for it would have to carry the stage, or
    /// it is the file the full run of that shot keeps.
    fn run_slot(&self, run: &RunStart) -> Option<String> {
        if run.practice {
            return None;
        }
        const DIFFICULTIES: [&str; 5] = ["easy", "normal", "hard", "lunatic", "extra"];
        // `character * 2 + shotType`, which is the order every table the game indexes by them is
        // in — see `game_manager::CHARACTER`.
        const SHOTS: [&str; 4] = ["reimu-a", "reimu-b", "marisa-a", "marisa-b"];

        let difficulty = usize::try_from(run.difficulty)
            .ok()
            .and_then(|at| DIFFICULTIES.get(at));
        let shot = usize::try_from(run.character * 2 + run.shot_type)
            .ok()
            .and_then(|at| SHOTS.get(at));
        Some(match (difficulty, shot) {
            (Some(difficulty), Some(shot)) => format!("{difficulty}-{shot}"),
            _ => format!(
                "difficulty{}-character{}-shot{}",
                run.difficulty, run.character, run.shot_type
            ),
        })
    }

    /// Two of these are not the field they look like, and both are because of where this is read:
    /// just after `GameManager::AddedCallback`, which is the moment the game's own replay reads
    /// them at.
    ///
    /// **The score comes out of `guiScore`, not `score`.** The last thing that callback does is set
    /// `score = 0`, and `GameManager::OnUpdate` raises it back — `if (score < guiScore) score =
    /// guiScore` — on the stage's first update. So on this frame `score` is nothing at all and
    /// `guiScore` is the run's total. The game's own recording reads `score` for its record and
    /// gets away with it because it is registered from inside that callback, before the zeroing.
    ///
    /// **The seed comes out of `randomSeed`, not the generator.** That field is where the same
    /// callback copies the generator's seed as a stage begins, so it still says what the stage
    /// started from once the stage has drawn from it.
    unsafe fn run_state(&self) -> RunState {
        unsafe {
            RunState {
                score: mem::read(G_GAME_MANAGER + game_manager::GUI_SCORE),
                seed: mem::read(G_GAME_MANAGER + game_manager::RANDOM_SEED),
                point_items: mem::read(G_GAME_MANAGER + game_manager::POINT_ITEMS),
                power: mem::read(G_GAME_MANAGER + game_manager::CURRENT_POWER),
                lives: mem::read(G_GAME_MANAGER + game_manager::LIVES_REMAINING),
                bombs: mem::read(G_GAME_MANAGER + game_manager::BOMBS_REMAINING),
                rank: mem::read(G_GAME_MANAGER + game_manager::RANK),
                power_items: mem::read(G_GAME_MANAGER + game_manager::POWER_ITEMS),
                extra_lives: mem::read(G_GAME_MANAGER + game_manager::EXTRA_LIVES),
                deaths: mem::read(G_GAME_MANAGER + game_manager::DEATHS),
            }
        }
    }

    /// The same writes `ReplayManager::AddedCallbackDemo` makes, in the same places, plus the two
    /// it has no need of.
    ///
    /// `extraLives` because that function does not write it: the replay branch of
    /// `GameManager::AddedCallback` raises the count with a loop against the score thresholds
    /// instead, which can only ever raise it — and a resume writing the score without it would
    /// have the game hand out every extra life the run has already had. Which is the fault
    /// `jump_to_stage` describes: a life the recording never got, and the subrank that came with
    /// it.
    ///
    /// `deaths` because the result screen shows it and a resumed run's count should be the run's.
    /// The two counters beside it — the ones a fresh stage zeroes — are left alone: the game's own
    /// replay does not carry them either, and nothing but that screen reads them.
    unsafe fn set_run_state(&self, state: &RunState) {
        unsafe {
            // Both, and the increment between them cleared: `guiScore` is the number on the panel
            // and it chases `score`, so leaving it behind would show the score counting up from
            // wherever the front end left it.
            mem::write::<u32>(G_GAME_MANAGER + game_manager::GUI_SCORE, state.score);
            mem::write::<u32>(G_GAME_MANAGER + game_manager::SCORE, state.score);
            mem::write::<u32>(G_GAME_MANAGER + game_manager::NEXT_SCORE_INCREMENT, 0);
            // The seed is not here: it goes in before the stage is built — see `set_run_seed` — and
            // by this point the loading has drawn from the generator, as it did in the run that was
            // written down.
            mem::write::<u16>(
                G_GAME_MANAGER + game_manager::POINT_ITEMS,
                state.point_items,
            );
            mem::write::<u16>(G_GAME_MANAGER + game_manager::CURRENT_POWER, state.power);
            mem::write::<i8>(G_GAME_MANAGER + game_manager::LIVES_REMAINING, state.lives);
            mem::write::<i8>(G_GAME_MANAGER + game_manager::BOMBS_REMAINING, state.bombs);
            mem::write::<i32>(G_GAME_MANAGER + game_manager::RANK, state.rank);
            mem::write::<i8>(
                G_GAME_MANAGER + game_manager::POWER_ITEMS,
                state.power_items,
            );
            mem::write::<i8>(
                G_GAME_MANAGER + game_manager::EXTRA_LIVES,
                state.extra_lives,
            );
            mem::write::<i32>(G_GAME_MANAGER + game_manager::DEATHS, state.deaths);
        }
    }

    /// The generator, and the `GameManager` copy of it beside it.
    ///
    /// Both, because this runs where `GameManager::AddedCallback` has already taken its copy — see
    /// `STAGE_REGISTER_CHAIN` for why it cannot run any earlier — and the copy is what a stage
    /// written down reads its seed from, orb's own record included. Left as the game set it, the
    /// generator would be right and the number written down for the next resume of that stage wrong.
    /// The count of numbers drawn is not touched: the callback has just zeroed it, which is what a
    /// stage starts from.
    unsafe fn set_run_seed(&self, seed: u16) {
        unsafe {
            mem::write::<u16>(G_RNG, seed);
            mem::write::<u16>(G_GAME_MANAGER + game_manager::RANDOM_SEED, seed);
        }
    }

    /// `curState` already the game manager while `wantedState` is still the front end, which is the
    /// one frame between the two.
    ///
    /// `Supervisor::OnUpdate` is chain priority 0 and copies `wantedState = curState` as its last
    /// act, and the front end is priority 2 — so an item that starts a run writes `curState` after
    /// that copy, and the frame that follows is the one where the two disagree and the supervisor has
    /// not yet built anything. The front end has already removed its own job by then, its update
    /// being what does that.
    ///
    /// The demo and a replay are refused rather than left to the caller: both ask for a run through
    /// the same two states, the demo from the title screen itself.
    unsafe fn run_chosen(&self) -> bool {
        unsafe {
            mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) == STATE_GAMEMANAGER
                && mem::read::<i32>(G_SUPERVISOR + supervisor::WANTED_STATE) == STATE_MAINMENU
                && mem::read::<u32>(G_GAME_MANAGER + game_manager::IS_IN_REPLAY) == 0
                && mem::read::<u8>(G_GAME_MANAGER + game_manager::DEMO_MODE) == 0
        }
    }

    /// The stage as the menu counts it: `GameManager::AddedCallback` raises the number by one, so it
    /// is handed the stage before the one meant — which is what the character select writes too.
    ///
    /// Checked against what the game has, because the number comes out of a text file beside the
    /// game and one that is nonsense is `AddedCallback` indexing its stage table past the end:
    /// `[eax*8+0x4764ec]` is where it picks a stage's data out of.
    unsafe fn start_stage(&self, stage: i32) -> bool {
        if !(0..replay_data::STAGES).contains(&stage) {
            log!("resume: there is no stage {}", stage + 1);
            return false;
        }
        unsafe { mem::write::<i32>(G_GAME_MANAGER + game_manager::CURRENT_STAGE, stage) };
        true
    }

    unsafe fn run_finished(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + supervisor::CUR_STATE) };
        scene == STATE_RESULTSCREEN_FROMGAME
    }

    unsafe fn reproduction(&self) -> Reproduction {
        unsafe {
            Reproduction {
                replay_frame: mem::read_committed::<usize>(G_REPLAY_MANAGER)
                    .filter(|manager| *manager != 0)
                    .and_then(mem::read_committed::<i32>)
                    .unwrap_or(-1),
                input: mem::read(G_CUR_FRAME_INPUT),
                player: (
                    mem::read(G_PLAYER + player::POSITION_CENTER),
                    mem::read(G_PLAYER + player::POSITION_CENTER + size_of::<f32>()),
                ),
                player_area: (
                    mem::read(
                        G_GAME_MANAGER + game_manager::PLAYER_AREA_TOP_LEFT + size_of::<f32>(),
                    ),
                    mem::read(G_GAME_MANAGER + game_manager::PLAYER_AREA_SIZE + size_of::<f32>()),
                ),
                randoms: mem::read(G_RNG + RNG_GENERATION_COUNT),
                seed: mem::read(G_RNG),
                items: mem::read(G_ITEM_MANAGER + ITEM_COUNT),
                score: mem::read(G_GAME_MANAGER + game_manager::GUI_SCORE),
                extra_lives: mem::read(G_GAME_MANAGER + game_manager::EXTRA_LIVES),
                rank: mem::read(G_GAME_MANAGER + game_manager::RANK),
                sub_rank: mem::read(G_GAME_MANAGER + game_manager::SUB_RANK),
            }
        }
    }

    /// The state and the frames left under it, both, and only from the two states the player can
    /// be hit in or is already invulnerable in.
    ///
    /// Spawning and dying are left alone: nothing can hit the player in either, and writing over
    /// a spawn would skip what sets the player's colour and scale on the way in — 0x428ee1
    /// onwards, and the 0xffffffff at 0x428f56 — so a player nobody can see is worse than one
    /// nothing can hit. An invulnerable player is written over rather than left, because leaving
    /// them means the frames run out and the one update they run out in is an update the hit test
    /// sees a player it can kill.
    unsafe fn make_invulnerable(&self) {
        let state = G_PLAYER + player::PLAYER_STATE;
        let current: i8 = unsafe { mem::read(state) };
        if current == PLAYER_NORMAL || current == PLAYER_INVULNERABLE {
            unsafe {
                mem::write::<i8>(state, PLAYER_INVULNERABLE);
                mem::write::<i32>(
                    G_PLAYER + player::INVULNERABLE_FRAMES,
                    PLAYER_INVULNERABLE_FRAMES,
                );
            }
        }
    }

    unsafe fn replaying(&self) -> bool {
        mem::read_committed::<usize>(G_REPLAY_MANAGER)
            .filter(|manager| *manager != 0)
            .and_then(|manager| mem::read_committed::<i32>(manager + replay_manager::IS_DEMO))
            .is_some_and(|demo| demo != 0)
    }

    unsafe fn count_card_attempt(&self) -> Option<u16> {
        let card = unsafe { mem::read::<i32>(CURRENT_CARD) };
        let record = CARD_HISTORY + usize::try_from(card).ok()? * 0x40;
        if record + 0x40 > CARD_HISTORY + CARD_HISTORY_BYTES
            || unsafe { mem::read::<u32>(record) } != CATK_MAGIC
        {
            return None;
        }
        let attempts = unsafe { mem::read::<u16>(record + CATK_ATTEMPTS) }.saturating_add(1);
        unsafe { mem::write::<u16>(record + CATK_ATTEMPTS, attempts) };
        Some(attempts)
    }

    unsafe fn captures(&self) -> Vec<u8> {
        unsafe { mem::read_bytes(CARD_HISTORY, CARD_HISTORY_BYTES) }
    }

    unsafe fn set_captures(&self, saved: &[u8]) {
        if saved.len() != CARD_HISTORY_BYTES {
            return log!(
                "score: {} bytes of captures is not this build's {CARD_HISTORY_BYTES}; left alone",
                saved.len()
            );
        }
        unsafe { mem::write_bytes(CARD_HISTORY, saved) };
    }

    unsafe fn show_ranking(&self) {
        let before: i32 = unsafe { mem::read(G_MAIN_MENU + main_menu::GAME_STATE) };
        MENU_STATE_BEFORE.store(before as usize + 1, Ordering::Relaxed);
        let cursor: i32 = unsafe { mem::read(G_MAIN_MENU + main_menu::CURSOR) };
        MENU_CURSOR_BEFORE.store(cursor as usize + 1, Ordering::Relaxed);
        unsafe { mem::write::<i32>(G_MAIN_MENU + main_menu::GAME_STATE, MENU_STATE_SCORE) };
    }

    unsafe fn showing_ranking(&self) -> bool {
        let screen = RESULT_SCREEN.load(Ordering::Relaxed);
        screen != 0
            && unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) } == STATE_SCORE
            && RESULT_SCREEN_SHOWING.contains(&unsafe { mem::read::<i32>(screen + RANKING_STATE) })
    }

    unsafe fn ranking_scene(&self) -> bool {
        unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) == STATE_SCORE }
    }

    unsafe fn ranking_state(&self) -> String {
        let screen = RESULT_SCREEN.load(Ordering::Relaxed);
        format!(
            "cur={} wanted={} screen={screen:#x} state={}",
            unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) },
            unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::WANTED_STATE) },
            if screen == 0 {
                -1
            } else {
                unsafe { mem::read::<i32>(screen + RANKING_STATE) }
            },
        )
    }

    unsafe fn leave_ranking(&self) {
        let screen = RESULT_SCREEN.load(Ordering::Relaxed);
        if screen != 0 {
            unsafe { mem::write::<i32>(screen + RANKING_STATE, RESULT_SCREEN_STATE_EXITING) };
        }
        // The front end's own state back as it was, before it is the thing acting on orb's request.
        // The item list rather than whatever was found there: after somebody has looked at the
        // ranking themselves the field still holds that item, and putting *that* back is the front end
        // acting on it again — a give-up that landed on the score screen instead of the title, and a
        // trip that therefore never wrote.
        if MENU_STATE_BEFORE.swap(0, Ordering::Relaxed) != 0 {
            unsafe { mem::write::<i32>(G_MAIN_MENU + main_menu::GAME_STATE, MENU_STATE_TITLE) };
        }
    }

    unsafe fn restore_menu_cursor(&self) {
        if let Some(cursor) = MENU_CURSOR_BEFORE.swap(0, Ordering::Relaxed).checked_sub(1) {
            unsafe { mem::write::<i32>(G_MAIN_MENU + main_menu::CURSOR, cursor as i32) };
        }
    }

    unsafe fn forget_captures(&self, screen: *mut std::ffi::c_void) {
        RESULT_SCREEN.store(screen as usize, Ordering::Relaxed);
        let state: i32 = unsafe { mem::read(screen as usize + RANKING_STATE) };
        if RANKING_STATES_KEEPING_THE_RECORD.contains(&state) {
            return;
        }
        // Written as bytes rather than record by record: what the game holds here is 64 of a
        // structure orb has no use for the shape of, and all that is being said is that it holds
        // none of them.
        unsafe { mem::fill_bytes(CARD_HISTORY, 0, CARD_HISTORY_BYTES) };
        log!("score: the captures in memory cleared for the ranking about to be read");
    }

    /// The item under the title menu's cursor, and only while the front end is what the game is
    /// running and has been since before this frame: `g_MainMenu` is a global that keeps whatever it
    /// was left holding, and what says it is being used is the supervisor.
    ///
    /// Both of the supervisor's states, because one of them is not enough. `Supervisor::OnUpdate`
    /// is chain priority 0 and assigns `wantedState = curState` as its last act, so a screen that
    /// sets `curState` to the front end — which is how the ranking leaves — is a frame ending with
    /// `curState` already saying front end while `wantedState` still says where the game was, and
    /// the front end not yet rebuilt. On that one frame `gameState` is whatever the screen being left
    /// was entered from, and `MainMenu::RegisterChain` only memsets that stale state away the frame
    /// after.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        let settled = unsafe {
            mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) == STATE_MAINMENU
                && mem::read::<i32>(G_SUPERVISOR + supervisor::WANTED_STATE) == STATE_MAINMENU
                && mem::read::<i32>(G_MAIN_MENU + main_menu::GAME_STATE) == MENU_STATE_TITLE
        };
        if !settled {
            return None;
        }
        match unsafe { mem::read::<i32>(G_MAIN_MENU + main_menu::CURSOR) } {
            TITLE_ITEM_START | TITLE_ITEM_EXTRA | TITLE_ITEM_PRACTICE => Some(Menu::Run),
            TITLE_ITEM_SCORE => Some(Menu::Scores),
            _ => None,
        }
    }

    /// Both of the ways the game reads a pad, in the order it tries them.
    ///
    /// `Controller::GetControllerInput` asks winmm for joystick 0 only where its own enumeration
    /// found no game controller; where it found one it polls that through DirectInput and never
    /// asks winmm at all. So a menu of orb's has to read the same device, or it answers to a pad
    /// the game has not got — which is what a pad in XInput's second slot does here: DirectInput
    /// has it, and winmm's joystick 0 is a phantom reporting no buttons and no axes — `mid=413d
    /// pid=2104`, every field zero. Measured on this machine with all three interfaces asked; the numbers
    /// are beside `orb`'s `joystick::Sample::is_a_pad`.
    unsafe fn pad(&self, winmm: Option<Reading>) -> Pad {
        unsafe { self.controller_pad() }
            .or_else(|| winmm.map(|reading| self.winmm_pad(reading)))
            .unwrap_or_default()
    }

    /// What `StageMenu::OnUpdateGameMenu` writes where its own quit is answered yes: the two
    /// menu flags cleared and `g_Supervisor.curState = MAINMENU`. `Supervisor::OnUpdate` then
    /// cuts the run's chain — the game manager, the player, the stage, the recording — and
    /// registers the front end, so there is nothing else for orb to take down.
    ///
    /// The flags are written although neither should be set. orb's menu is not one of the
    /// game's, and it opens on the frame `Player::Die` runs, where the flag a run out of lives
    /// gets is written 30 frames later by a respawn the freeze never reaches. What makes it
    /// worth two bytes anyway is what a stale one does rather than how it would get there:
    /// `AsciiManager`'s job is registered by the supervisor for the whole process, so
    /// `isInRetryMenu` left set runs `StageMenu::OnUpdateRetryMenu` on the title screen, and its
    /// first three branches write `curState` themselves — one of them to the result screen.
    unsafe fn leave_run(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + supervisor::CUR_STATE) };
        if scene != STATE_GAMEMANAGER && scene != STATE_GAMEMANAGER_REINIT {
            return false;
        }
        unsafe {
            mem::write::<u8>(G_GAME_MANAGER + game_manager::IS_IN_GAME_MENU, 0);
            mem::write::<u8>(G_GAME_MANAGER + game_manager::IS_IN_RETRY_MENU, 0);
            mem::write::<i32>(G_SUPERVISOR + supervisor::CUR_STATE, STATE_MAINMENU);
        }
        true
    }

    /// Every button, so that the `held & ~held-last-frame` every one of the game's own
    /// `WAS_PRESSED` is finds nothing on the frame it carries on into. `Supervisor::OnUpdate`
    /// runs first in the calc chain — priority 0 against the main menu's 2 — and its first act is
    /// `g_LastFrameInput = g_CurFrameInput; g_CurFrameInput = GetInput()`, so this is read once,
    /// by that, and by nothing else. What is genuinely held still reads as held, since that comes
    /// from the fresh read; only the edge is gone, and only for the one frame.
    ///
    /// Not zero, which is what the game itself writes into these three at a scene change — see
    /// the end of `Supervisor::OnUpdate`'s state switch. Zero leaves `g_LastFrameInput` empty and
    /// so turns every button still down into a fresh press, which is the opposite of what is
    /// wanted here. The game gets away with it because the screens it changes to guard their own
    /// first frames with timers instead.
    unsafe fn swallow_input(&self) {
        unsafe { mem::write::<u16>(G_CUR_FRAME_INPUT, u16::MAX) };
    }

    /// The state is written rather than the screen being answered for the player, because
    /// answering it means writing the interrupt each of its 38 sprites is to run next and then
    /// waiting out the fade they play — where this is one number, and the one the game itself
    /// puts a practice run's result screen into.
    ///
    /// Written the frame the state is reached and before the frame timer gets to the 60 that
    /// starts the question's own animation, so no part of the screen is ever drawn.
    unsafe fn skip_replay_prompt(&self) -> bool {
        // The one read on the frames there is no such screen, which is every frame of a run.
        if unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) }
            != STATE_RESULTSCREEN_FROMGAME
        {
            return false;
        }
        let Some(screen) = chain_argument(RESULT_SCREEN_ON_UPDATE) else {
            return false;
        };
        let state = screen + result_screen::STATE;
        if mem::read_committed::<i32>(state) != Some(RESULT_STATE_SAVE_REPLAY_QUESTION) {
            return false;
        }
        unsafe { mem::write::<i32>(state, RESULT_STATE_EXIT) };
        true
    }

    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &chapters::MIDSTAGE
    }
}

/// The size the game renders at, which its viewports are expressed in.
const BACK_BUFFER: (u32, u32) = (640, 480);

/// `GCOS_CLEAR_BACKBUFFER_ON_REFRESH` and `GCOS_DISPLAY_MINIMUM_GRAPHICS`, the two
/// options behind `Supervisor::IsUnknown`.
const CFG_OPTS: usize = 0x148;
const GCOS_CLEAR_BACKBUFFER_ON_REFRESH: u32 = 3;
const GCOS_DISPLAY_MINIMUM_GRAPHICS: u32 = 4;

impl Th06 {
    /// The game's own mapping, read where `Controller::GetControllerInput` reads it, and the axis
    /// read the way that function reads it.
    ///
    /// **Shoot decides; bomb and menu cancel**, which is what the game's own menus do:
    /// `TH_BUTTON_SELECTMENU` is `TH_BUTTON_ENTER | TH_BUTTON_SHOOT` and `TH_BUTTON_RETURNMENU` is
    /// `TH_BUTTON_MENU | TH_BUTTON_BOMB`, so either of those two is back there.
    ///
    /// The menu button decided here for a while instead, because on the pad orb was first run with
    /// it was button 0 — where a thumb rests — and the most obvious button on the pad closing a
    /// question rather than answering it took three launches to find. That is the thing to look for
    /// if a menu of orb's starts cancelling itself, and the launcher printing the mapping it read is
    /// where to look: on the pad this is written against, shoot is button 0 and menu is button 1.
    ///
    /// An unmapped button is 0xffff in that mapping, which as an `i16` is negative and names no bit
    /// — which is what the directions usually are, a stick being how a pad is pushed.
    fn winmm_pad(&self, reading: Reading) -> Pad {
        // The low side of the Y axis is up, it being measured downwards; and a d-pad reports in the
        // hat rather than on the axes at all.
        let (stick_up, stick_down) = axis(joy_caps::Y_MIN, joy_caps::Y_MAX, reading.y);
        let (hat_up, hat_down) = hat(reading.pov);
        self.pad_from(reading.buttons, stick_up || hat_up, stick_down || hat_down)
    }

    /// The pad the game itself polls, where its own enumeration found one: `Poll` and then
    /// `GetDeviceState` into the `DIJOYSTATE2` the format it set fills, which is what
    /// `Controller::GetControllerInput` does on those frames. `None` where there is no such device
    /// or the read did not come off, and then the winmm sample is what is left.
    ///
    /// The acquire after a lost device is orb's to do here, because the frames this runs on are
    /// exactly the frames the game's own read is frozen out of: the device is taken
    /// `DISCL_EXCLUSIVE | DISCL_FOREGROUND`, so anything that took the foreground away — which is
    /// how somebody arrives at a menu of orb's with the window behind — leaves it unacquired. Asked
    /// for once and the frame given up, the way the game asks: it is a menu, and the next frame is
    /// a sixtieth of a second away.
    unsafe fn controller_pad(&self) -> Option<Pad> {
        let device = mem::read_committed::<usize>(G_CONTROLLER).filter(|device| *device != 0)?;
        let vtable = mem::read_committed::<usize>(device)?;
        let slot = |index: usize| {
            mem::read_committed::<usize>(vtable + index * size_of::<usize>()).filter(|it| *it != 0)
        };
        let poll: unsafe extern "system" fn(usize) -> i32 =
            unsafe { std::mem::transmute(slot(dinput_device::POLL)?) };
        if unsafe { poll(device) } < 0 {
            let acquire: unsafe extern "system" fn(usize) -> i32 =
                unsafe { std::mem::transmute(slot(dinput_device::ACQUIRE)?) };
            unsafe { acquire(device) };
            return None;
        }
        let read: unsafe extern "system" fn(usize, u32, *mut JoyState) -> i32 =
            unsafe { std::mem::transmute(slot(dinput_device::GET_DEVICE_STATE)?) };
        let mut state: JoyState = unsafe { std::mem::zeroed() };
        if unsafe { read(device, size_of::<JoyState>() as u32, &mut state) } < 0 {
            return None;
        }
        // Every button as one mask, in the numbering the mapping names them in: DirectInput gives a
        // byte each with the top bit set, and `SetButtonFromDirectInputJoystate` indexes that array
        // with the very same number `SetButtonFromControllerInputs` shifts a winmm mask by.
        let buttons = state
            .buttons
            .iter()
            .take(u32::BITS as usize)
            .enumerate()
            .filter(|(_, held)| **held & 0x80 != 0)
            .fold(0, |mask, (button, _)| mask | 1 << button);
        // The axes are the ±1000 the game gave every one of them, against the threshold it keeps
        // beside that mapping — not the winmm caps, which describe another device entirely.
        let threshold =
            i32::from(unsafe { mem::read::<i16>(G_SUPERVISOR + supervisor::CFG_PAD_Y_AXIS) });
        let (hat_up, hat_down) = hat(state.hats[0]);
        let y = state.axes[AXIS_Y];
        Some(self.pad_from(buttons, y < -threshold || hat_up, y > threshold || hat_down))
    }

    /// What a frame's buttons and directions mean to a menu, whichever of the two devices they
    /// were read from. Only the directions differ between them: a button is a button by the same
    /// number either way.
    fn pad_from(&self, buttons: u32, up: bool, down: bool) -> Pad {
        let held = |at: usize| {
            u32::try_from(unsafe { mem::read::<i16>(G_SUPERVISOR + at) })
                .ok()
                .filter(|button| *button < u32::BITS)
                .is_some_and(|button| buttons & (1 << button) != 0)
        };
        Pad {
            up: up || held(supervisor::CFG_UP_BUTTON),
            down: down || held(supervisor::CFG_DOWN_BUTTON),
            decide: held(supervisor::CFG_SHOOT_BUTTON),
            cancel: held(supervisor::CFG_BOMB_BUTTON) || held(supervisor::CFG_MENU_BUTTON),
        }
    }

    /// Takes down any screen shake still running, and puts the play field's rectangle back
    /// where the shake would have put it when it finished.
    ///
    /// Screen effects outlive the stage that started them, which is deliberate — a fade
    /// between two stages belongs to neither, and `GameManager::DeletedCallback` leaves
    /// them alone where it cuts everything else. A shake is not only drawing, though: it
    /// writes the arcade region from two numbers out of the generator every frame, and
    /// `Player::AddedCallback` measures where the player starts from that region —
    /// `arcadeRegionSize.x / 2` across and `arcadeRegionSize.y - 64` down. So a bomb
    /// within a shake's 80 frames of a stage move gave the next stage a player 3.13
    /// pixels high, at `192.00,380.87` where the stage starts one at `192.00,384.00`, and
    /// four numbers a frame going out of a stream the replay has to match. Measured at
    /// 312597765ms in the log: stage 2 left at its frame 642 with `bombs=2`.
    ///
    /// The region is written here rather than left to the shake, because the shake only
    /// restores it on the frame it removes itself on and it is being removed early.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn cut_screen_shake(&self) {
        let cut: unsafe extern "thiscall" fn(usize, usize) =
            unsafe { std::mem::transmute(CHAIN_CUT) };
        let mut elem = unsafe { mem::read::<usize>(G_CHAIN + chain_elem::NEXT) };
        while elem != 0 {
            // Read before the cut: the element is freed by it.
            let next = unsafe { mem::read::<usize>(elem + chain_elem::NEXT) };
            if unsafe { mem::read::<usize>(elem + chain_elem::CALLBACK) } == SHAKE_SCREEN {
                unsafe { cut(G_CHAIN, elem) };
                let area = self.play_area();
                unsafe {
                    mem::write::<[f32; 2]>(
                        G_GAME_MANAGER + game_manager::ARCADE_REGION_TOP_LEFT,
                        [area.left, area.top],
                    );
                    mem::write::<[f32; 2]>(
                        G_GAME_MANAGER + game_manager::ARCADE_REGION_SIZE,
                        [area.width, area.height],
                    );
                }
                log!("stage move: a screen shake was still running, and is taken down");
            }
            elem = next;
        }
    }

    fn clears_background(&self) -> bool {
        let opts = unsafe { mem::read::<u32>(G_SUPERVISOR + CFG_OPTS) };
        opts >> GCOS_CLEAR_BACKBUFFER_ON_REFRESH & 1 != 0
            || opts >> GCOS_DISPLAY_MINIMUM_GRAPHICS & 1 != 0
    }

    /// The live `CStreamingSound`, checked before being believed: the allocator
    /// does not scrub freed blocks, so a stale pointer reads back as its old self.
    fn streaming_sound(&self) -> Option<usize> {
        // Asked of the memory map rather than read outright, which is the only read in
        // this path that assumed the address was there. It costs one `VirtualQuery` and
        // makes every `Game` method that leads here answerable with no game around it —
        // which is what lets the rules about where chapters begin be tested at all, since
        // deciding whether a boss was a midboss asks what music is playing.
        let streaming =
            mem::read_committed::<usize>(G_SOUND_PLAYER + sound_player::BACKGROUND_MUSIC)?;
        mem::vtable_in_image(streaming).then_some(streaming)
    }

    /// `g_Stage.stdData->songPaths[0]`, an inline array so the address is the string.
    fn stage_song(&self) -> Option<(usize, String)> {
        let std_data = mem::read_committed::<usize>(G_STAGE + stage::STD_DATA)?;
        path_at(std_data + stage_header::SONG_PATHS)
    }

    fn wave_file(&self) -> Option<usize> {
        let streaming = self.streaming_sound()?;
        let wave_file = mem::read_committed::<usize>(streaming + streaming_sound::WAVE_FILE)?;
        mem::read_committed::<u32>(wave_file + wave_file::SIZE).map(|_| wave_file)
    }
}

/// Whether an axis is pushed past its dead zone, as `(low, high)` in the position it reports —
/// which for the Y axis, measured downwards, is `(up, down)`.
///
/// The centre is halfway between the caps' bounds and the dead zone is a quarter of the travel
/// either side of it, which is what `Controller::GetControllerInput` does with the same two
/// numbers. Nothing where the bounds say nothing, since a device whose travel is zero has no
/// middle to be off.
fn axis(low_at: usize, high_at: usize, position: u32) -> (bool, bool) {
    let (low, high) = unsafe {
        (
            mem::read::<u32>(G_JOY_CAPS + low_at),
            mem::read::<u32>(G_JOY_CAPS + high_at),
        )
    };
    if high <= low {
        return (false, false);
    }
    let centre = low + (high - low) / 2;
    let dead = (high - low) / 4;
    (position + dead < centre, position > centre + dead)
}

/// Whether a hat — a d-pad — is pushed up or down.
///
/// Its own field rather than the axes, because that is where a d-pad reports: hundredths of a degree
/// clockwise from straight up, and `JOY_POVCENTERED` — 0xffff, past a full circle — for pushed
/// nowhere. A diagonal counts as its two, so a hat held up-and-left still moves up. The game itself
/// reads none of this; orb's own menus are the only thing here a hat drives.
fn hat(pov: u32) -> (bool, bool) {
    /// A full circle, and an eighth of one either side of each direction.
    const CIRCLE: u32 = 36000;
    const EIGHTH: u32 = CIRCLE / 8;

    if pov > CIRCLE {
        return (false, false);
    }
    (
        pov <= EIGHTH || pov >= CIRCLE - EIGHTH,
        (CIRCLE / 2 - EIGHTH..=CIRCLE / 2 + EIGHTH).contains(&pov),
    )
}

/// A path out of the game's own memory, and what it says, checked before either is
/// used. These are handed back to the game as path arguments — `PlayAudio` looks for
/// the last `.` in one without checking that there is one, and `LoadAnm` opens what it
/// is given — so a wrong offset is a line in the log rather than a crash inside the
/// game.
fn path_at(address: usize) -> Option<(usize, String)> {
    /// Longer than any path the game carries; `songPaths` entries are this size.
    const LIMIT: usize = 128;

    let bytes = mem::read_committed::<[u8; LIMIT]>(address)?;
    let end = bytes.iter().position(|byte| *byte == 0)?;
    let name = std::str::from_utf8(&bytes[..end]).ok()?;
    let usable = !name.is_empty()
        && name.contains('.')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ');
    if !usable {
        log!("{address:#010x} does not hold a path");
        return None;
    }
    Some((address, name.to_owned()))
}

/// The `.end` script the ending is reading, out of the ending's own object.
///
/// 紅魔郷's six endings all finish on `@Fdata/staff00.end`, the staff roll: the script
/// interpreter's `F` instruction, at 0x40fc06, reads that file over the one running and
/// carries straight on into it. Which file is loaded is therefore what tells the ending from
/// the roll, and nothing else does — the scene stays 10 across the two and `isInEnding` stays
/// set through both.
fn ending_script() -> Option<usize> {
    let ending = chain_argument(ENDING_ON_UPDATE)?;
    mem::read_committed::<usize>(ending + ending::SCRIPT).filter(|script| *script != 0)
}

/// Longer than any chain the game builds: it has seventeen jobs to register in all, one per
/// priority from 0 to 0x10. A chain left in a state where the links do not end is then a walk that
/// answers nothing rather than one that does not end either.
const CHAIN_LINKS: usize = 64;

/// Whether `elem` is one of the jobs in a chain's `list`, which for a job registered from a static
/// is the whole of whether the game is still running it.
///
/// Every step is asked of the memory map rather than read outright, the same way and for the same
/// reason as `chain_argument`.
fn chain_holds(list: usize, elem: usize) -> bool {
    let mut at = mem::read_committed::<usize>(G_CHAIN + list + chain_elem::NEXT);
    for _ in 0..CHAIN_LINKS {
        match at {
            None | Some(0) => return false,
            Some(job) if job == elem => return true,
            Some(job) => at = mem::read_committed::<usize>(job + chain_elem::NEXT),
        }
    }
    false
}

/// What a chain job's callbacks are called on, found by the callback it was registered for.
/// How the ending's object is reached at all: `Ending::RegisterChain` allocates it, hands it
/// to the element it registers, and puts it nowhere else.
///
/// Every step is asked of the memory map rather than read outright, at a `VirtualQuery` each.
/// The walk is short, it only runs on the frames of an ending, and it is what makes this
/// answerable with no game around it.
fn chain_argument(callback: usize) -> Option<usize> {
    let mut elem = mem::read_committed::<usize>(G_CHAIN + chain_elem::NEXT)?;
    for _ in 0..CHAIN_LINKS {
        if elem == 0 {
            return None;
        }
        if mem::read_committed::<usize>(elem + chain_elem::CALLBACK)? == callback {
            return mem::read_committed::<usize>(elem + chain_elem::ARG).filter(|arg| *arg != 0);
        }
        elem = mem::read_committed::<usize>(elem + chain_elem::NEXT)?;
    }
    None
}

unsafe fn laser_count() -> i32 {
    let lasers = G_BULLET_MANAGER + bullet_manager::LASERS;
    (0..bullet_manager::LASER_COUNT)
        .filter(|index| {
            let laser = lasers + index * bullet_manager::LASER_STRIDE;
            unsafe { mem::read::<i32>(laser + bullet_manager::LASER_IN_USE) != 0 }
        })
        .count() as i32
}

/// `GuiImpl` lives on the heap, so this is a null-checked pointer chase.
/// Negative when no dialogue is running.
unsafe fn dialogue_msg_idx() -> i32 {
    unsafe {
        let implementation = mem::read::<usize>(G_GUI + gui::IMPL);
        if implementation == 0 {
            return -1;
        }
        mem::read(implementation + gui::IMPL_CURRENT_MSG_IDX)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        CARD_HISTORY, CARD_HISTORY_BYTES, G_GAME_MANAGER, G_REPLAY_MANAGER, G_SUPERVISOR,
        RANKING_STATE, RANKING_STATES_KEEPING_THE_RECORD, STATE_GAMEMANAGER,
        STATE_GAMEMANAGER_REINIT, STATE_MAINMENU, ending_script, game_manager, replay_manager,
        supervisor,
    };
    use crate::game::{Game, RunStart, th06::Th06};
    use orb_api::Kind;
    use orb_sim::Sim;

    /// Somewhere the game's allocator would have put a structure — outside its static data, so
    /// that a pointer into it is a pointer and not an offset from a global.
    const ALLOCATED: usize = 0x03a0_0000;

    /// The globals these tests read, with nothing in them. One region over the game manager,
    /// which is also where the spell card history sits 0x30 into it, and one over the
    /// supervisor; the replay manager is a pointer, so only the pointer is laid out here and
    /// what it points at is each test's business.
    fn image() -> Arc<Sim> {
        let sim = Arc::new(Sim::new());
        let space = sim.space();
        space.map(G_GAME_MANAGER, 0x2000, Kind::Private);
        space.map(G_SUPERVISOR, 0x1000, Kind::Private);
        space.map(G_REPLAY_MANAGER, size_of::<usize>(), Kind::Private);
        sim
    }

    /// The frame a stage is on is the game manager's own count, and it is only that while a stage
    /// is the scene: the manager is a global that keeps whatever it was left holding, so the count
    /// reads as a number at every moment of a run and as a stale one at every moment that is not.
    #[test]
    fn the_stage_frame_is_read_only_while_a_stage_is_the_scene() {
        let sim = image();
        let _installed = sim.enter();
        let space = sim.space();
        space.write::<u32>(G_GAME_MANAGER + game_manager::GAME_FRAMES, 1886);

        for scene in [STATE_GAMEMANAGER, STATE_GAMEMANAGER_REINIT] {
            space.write::<i32>(G_SUPERVISOR + supervisor::CUR_STATE, scene);
            assert_eq!(unsafe { Th06.stage_frame() }, Some(1886));
        }

        // The front end, where the count left over from the run before it is not this run's.
        space.write::<i32>(G_SUPERVISOR + supervisor::CUR_STATE, STATE_MAINMENU);
        assert_eq!(unsafe { Th06.stage_frame() }, None);
    }

    /// A replay being watched is reached by chasing a pointer the game may not have written yet,
    /// and each step of the chase is a read that has to come back rather than fault: no manager,
    /// a manager the game has not allocated, and a manager that is there and says it is not a
    /// demo, all read as a run somebody is playing.
    #[test]
    fn a_replay_is_read_through_a_pointer_that_may_not_be_there_yet() {
        let sim = image();
        let _installed = sim.enter();
        let space = sim.space();

        // Never written, which is the zero a global starts at.
        assert!(!unsafe { Th06.replaying() });

        // Written, and pointing at memory the game has not allocated. This is the read that takes
        // the process down if it is not asked for as `read_committed`.
        space.write::<usize>(G_REPLAY_MANAGER, ALLOCATED);
        assert!(!unsafe { Th06.replaying() });

        space.map(ALLOCATED, 0x100, Kind::Private);
        assert!(!unsafe { Th06.replaying() });

        space.write::<i32>(ALLOCATED + replay_manager::IS_DEMO, 1);
        assert!(unsafe { Th06.replaying() });
    }

    /// The game is asked to make a window rather than take the display by writing its own option,
    /// which is a byte it reads back the same way.
    #[test]
    fn forcing_a_window_is_read_back_as_windowed() {
        let sim = image();
        let _installed = sim.enter();

        assert!(!Th06.windowed());
        unsafe { Th06.force_windowed() };
        assert!(Th06.windowed());
    }

    /// The spell card record is taken and put back as the bytes the game keeps it in, so what
    /// comes back is what went in — the whole of it, byte for byte, with orb having no use for the
    /// shape of the sixty-four records inside.
    #[test]
    fn the_captures_that_are_put_back_are_the_ones_that_were_taken() {
        let sim = image();
        let _installed = sim.enter();
        let space = sim.space();
        space.write::<u32>(CARD_HISTORY + 0x40, 0xabcd_1234);

        let saved = unsafe { Th06.captures() };
        assert_eq!(saved.len(), CARD_HISTORY_BYTES);

        space.fill_bytes(CARD_HISTORY, 0xff, CARD_HISTORY_BYTES);
        assert_ne!(unsafe { Th06.captures() }, saved);

        unsafe { Th06.set_captures(&saved) };
        assert_eq!(unsafe { Th06.captures() }, saved);
    }

    /// A record of the wrong length is not this build's, and half of one written in would be worse
    /// than none: the file is left to say what it says and memory is left alone.
    #[test]
    fn captures_of_another_length_are_left_alone() {
        let sim = image();
        let _installed = sim.enter();
        let space = sim.space();
        space.fill_bytes(CARD_HISTORY, 0x5a, CARD_HISTORY_BYTES);
        let before = unsafe { Th06.captures() };

        unsafe { Th06.set_captures(&[0u8; 8]) };
        assert_eq!(unsafe { Th06.captures() }, before);
    }

    /// The record in memory is cleared before the game reads a ranking into it, so that the read
    /// defines the history rather than adding to what a session had already counted — except in
    /// the states the screen is in on the way out of a run, where the record in memory *is* that
    /// run's own and the file has not got it yet.
    #[test]
    fn the_captures_are_cleared_for_a_ranking_read_but_not_on_the_way_out_of_a_run() {
        let sim = image();
        let _installed = sim.enter();
        let space = sim.space();
        space.map(ALLOCATED, 0x100, Kind::Private);
        let screen = ALLOCATED as *mut std::ffi::c_void;

        for state in RANKING_STATES_KEEPING_THE_RECORD {
            space.fill_bytes(CARD_HISTORY, 0x5a, CARD_HISTORY_BYTES);
            space.write::<i32>(ALLOCATED + RANKING_STATE, state);
            unsafe { Th06.forget_captures(screen) };
            assert!(unsafe { Th06.captures() }.iter().all(|byte| *byte == 0x5a));
        }

        space.write::<i32>(ALLOCATED + RANKING_STATE, 0);
        unsafe { Th06.forget_captures(screen) };
        assert!(unsafe { Th06.captures() }.iter().all(|byte| *byte == 0));
    }

    /// Which runs a chapter can be picked up in, as the name of the file it is kept in: the
    /// difficulty, the character and the shot, and the stage not among them — a full run's stage is
    /// what a resume moves through.
    #[test]
    fn a_slot_names_the_run_a_chapter_belongs_to() {
        let run = RunStart {
            difficulty: 3,
            character: 1,
            shot_type: 1,
            practice: false,
            stage: 3,
        };
        assert_eq!(Th06.run_slot(&run).as_deref(), Some("lunatic-marisa-b"));
        assert_eq!(
            Th06.run_slot(&RunStart { stage: 5, ..run }).as_deref(),
            Some("lunatic-marisa-b"),
        );
        // Another difficulty, character or shot is another run, and none of the three may be lost
        // in the name: a chapter picked up in the wrong one plays somebody else's shot.
        for other in [
            RunStart {
                difficulty: 1,
                ..run
            },
            RunStart {
                character: 0,
                ..run
            },
            RunStart {
                shot_type: 0,
                ..run
            },
        ] {
            assert_ne!(Th06.run_slot(&other), Th06.run_slot(&run));
        }
        // A run this game has not got is said as the numbers it is rather than guessed at.
        assert_eq!(
            Th06.run_slot(&RunStart {
                character: 7,
                ..run
            })
            .as_deref(),
            Some("difficulty3-character7-shot1"),
        );
    }

    /// A practice run has no slot at all, which is what keeps one from being written down: it is one
    /// stage played on its own, and the game's own menu starts that stage again in a moment.
    #[test]
    fn a_practice_run_has_no_slot() {
        for stage in 0..6 {
            assert_eq!(
                Th06.run_slot(&RunStart {
                    difficulty: 3,
                    character: 1,
                    shot_type: 1,
                    practice: true,
                    stage,
                }),
                None,
            );
        }
    }

    /// With no game around it — which is not the same as with nothing there. This binary's own
    /// image is 10.5MB from 0x400000, so the game's globals are addresses inside it, and what the
    /// walk reads at them is whatever this binary keeps there. `None` is then the walk finding no
    /// job registered for the ending's callback among those bytes, and the reads getting that far
    /// without faulting is what `mem::read_committed` is for.
    #[test]
    fn there_is_no_ending_script_without_a_game() {
        assert_eq!(ending_script(), None);
    }
}
