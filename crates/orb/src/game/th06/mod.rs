//! 東方紅魔郷 1.02h, as a [`Game`].
//!
//! Globals come from `config/globals.csv` of the GensokyoClub/th06
//! decompilation and field offsets from its struct definitions, both for this
//! exact build. Offsets are spelled out rather than derived from Rust structs
//! because only a handful of the fields matter and mirroring whole structs
//! (`Player` alone is 0x98f0 bytes) would be far more to get wrong.

pub mod chapters;

use std::ops::Range;

use windows_sys::Win32::Foundation::HWND;

use crate::audio::{Music, SoundBuffer};
use crate::d3d8::{D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, Device, Viewport};
use crate::game::{Game, Hooks, Patch, Rect, Reproduction, State};
use crate::log::log;

use crate::mem;

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
/// `u16 g_CurFrameInput`, assigned from `Controller::GetInput` at the top of
/// `Supervisor::OnUpdate` and then, for a replay, overwritten with the record's buttons
/// in place of the ones on the keyboard.
const G_CUR_FRAME_INPUT: usize = 0x0069d904;
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
    pub const LIVES_REMAINING: usize = 0x181a;
    pub const BOMBS_REMAINING: usize = 0x181b;
    pub const EXTRA_LIVES: usize = 0x181c;
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
    /// `m_dwSize`, the length of the wave file.
    pub const SIZE_OF_FILE: usize = 0x30;
    pub const LOOP_START: usize = 0x90;
    pub const LOOP_END: usize = 0x94;
    pub const SIZE: usize = 0x98;
}

/// `th06::SupervisorState`.
const STATE_GAMEMANAGER: i32 = 2;
/// The state between two stages of a run, where the game tears the last stage's
/// managers down and builds the next one's. Not the run ending, which is what makes it
/// worth telling apart.
const STATE_GAMEMANAGER_REINIT: i32 = 3;
const STATE_ENDING: i32 = 10;

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
            update: Patch { target: RUN_CALC_CHAIN, prologue: RUN_CHAIN_PROLOGUE },
            draw: Patch { target: RUN_DRAW_CHAIN, prologue: RUN_CHAIN_PROLOGUE },
            save_replay: Some(Patch { target: SAVE_REPLAY, prologue: SAVE_REPLAY_PROLOGUE }),
            stop_recording: Some(Patch {
                target: STOP_RECORDING,
                prologue: STOP_RECORDING_PROLOGUE,
            }),
            create_window: Some(Patch {
                target: CREATE_GAME_WINDOW,
                prologue: CREATE_GAME_WINDOW_PROLOGUE,
            }),
            init_device: Some(Patch {
                target: INIT_D3D_DEVICE,
                prologue: INIT_D3D_DEVICE_PROLOGUE,
            }),
            render: Some(Patch { target: RENDER, prologue: RENDER_PROLOGUE }),
            input: Some(Patch { target: GET_INPUT, prologue: GET_INPUT_PROLOGUE }),
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

    unsafe fn window(&self) -> HWND {
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
        let mmio = mem::read_committed(wave_file + wave_file::MMIO).unwrap_or(std::ptr::null_mut());
        Some(Music {
            stream: streaming,
            buffer: buffer as *mut SoundBuffer,
            buffer_size,
            notify_size: mem::read_committed(streaming + streaming_sound::NOTIFY_SIZE)?,
            mmio,
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
            mem::write::<usize>(G_SOUND_PLAYER + sound_player::BACKGROUND_MUSIC_THREAD_HANDLE, 0);
        }
        let Some((path, name)) = self.stage_song() else { return false };
        log!("music: restarting {name}");
        // `__thiscall` with an argument, which unlike the one-argument case is not
        // `fastcall`: the argument goes on the stack and the callee takes it off.
        let play: unsafe extern "thiscall" fn(usize, usize) -> i32 =
            unsafe { std::mem::transmute(PLAY_AUDIO) };
        unsafe { play(G_SUPERVISOR, path) };
        true
    }

    fn audio_state(&self) -> Vec<Range<usize>> {
        let mut ranges = vec![G_SOUND_PLAYER..G_SOUND_PLAYER + sound_player::SIZE];
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
                vec![textures..textures + anm_manager::TEXTURE_COUNT * size_of::<usize>()]
            })
            .unwrap_or_default()
    }

    /// From `GAME_REGION_*`, inside the game's 640x480 output.
    fn play_area(&self) -> Rect {
        Rect { left: 32.0, top: 16.0, width: 384.0, height: 448.0 }
    }

    fn joystick_calibration(&self) -> Option<usize> {
        Some(G_JOY_CAPS)
    }

    fn content_size(&self) -> (u32, u32) {
        let present = G_SUPERVISOR + supervisor::PRESENT_PARAMETERS;
        let size = unsafe { (mem::read::<u32>(present), mem::read::<u32>(present + 4)) };
        // Before the device exists the parameters are zero; the game renders at
        // its own size whatever the back buffer ends up being.
        if size.0 == 0 || size.1 == 0 { BACK_BUFFER } else { size }
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
                (vtable.clear)(device, 0, std::ptr::null(), D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER, fog, 1.0, 0);
            }
            mem::write(G_SUPERVISOR + supervisor::VIEWPORT, viewport);
            (vtable.set_viewport)(device, &viewport);
            // One update is one tick: the loop above runs the logic exactly once
            // per frame, however fast the frames are being produced.
            mem::write::<f32>(G_SUPERVISOR + supervisor::FRAMERATE_MULTIPLIER, 1.0);
        }
    }

    unsafe fn play_sounds(&self) {
        let play: unsafe extern "fastcall" fn(usize) =
            unsafe { std::mem::transmute(PLAY_SOUNDS) };
        unsafe { play(G_SOUND_PLAYER) };
    }

    unsafe fn present(&self) {
        let present: unsafe extern "fastcall" fn(usize) = unsafe { std::mem::transmute(PRESENT) };
        unsafe { present(G_GAME_WINDOW) };
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
        let extras = if difficulty < DIFFICULTY_EXTRA { 0 } else { EXTRA_LIVES_IN_EXTRA };
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
            mem::write::<i32>(G_SUPERVISOR + supervisor::CUR_STATE, STATE_GAMEMANAGER_REINIT);
        }
        log!("stage {}: asked the game to start the replay there", stage + 1);
        true
    }

    unsafe fn reproduction(&self) -> Reproduction {
        unsafe {
            Reproduction {
                replay_frame: mem::read_committed::<usize>(G_REPLAY_MANAGER)
                    .filter(|manager| *manager != 0)
                    .and_then(|manager| mem::read_committed::<i32>(manager))
                    .unwrap_or(-1),
                input: mem::read(G_CUR_FRAME_INPUT),
                player: (
                    mem::read(G_PLAYER + player::POSITION_CENTER),
                    mem::read(G_PLAYER + player::POSITION_CENTER + size_of::<f32>()),
                ),
                player_area: (
                    mem::read(G_GAME_MANAGER + game_manager::PLAYER_AREA_TOP_LEFT + size_of::<f32>()),
                    mem::read(G_GAME_MANAGER + game_manager::PLAYER_AREA_SIZE + size_of::<f32>()),
                ),
                randoms: mem::read(G_RNG + RNG_GENERATION_COUNT),
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

    fn midstage_table(&self) -> &'static [&'static [i32]] {
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
        && name.bytes().all(|byte| byte.is_ascii_graphic() || byte == b' ');
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

/// What a chain job's callbacks are called on, found by the callback it was registered for.
/// How the ending's object is reached at all: `Ending::RegisterChain` allocates it, hands it
/// to the element it registers, and puts it nowhere else.
///
/// Every step is asked of the memory map rather than read outright, at a `VirtualQuery` each.
/// The walk is short, it only runs on the frames of an ending, and it is what makes this
/// answerable with no game around it.
fn chain_argument(callback: usize) -> Option<usize> {
    /// Longer than any chain the game builds: it has seventeen jobs to register in all, one
    /// per priority from 0 to 0x10. A chain left in a state where the links do not end is then
    /// a missing answer rather than a walk that does not either.
    const LINKS: usize = 64;

    let mut elem = mem::read_committed::<usize>(G_CHAIN + chain_elem::NEXT)?;
    for _ in 0..LINKS {
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
    use super::ending_script;

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
