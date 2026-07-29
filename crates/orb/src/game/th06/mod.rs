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
use crate::game::{Game, Hooks, Patch, Rect, State};

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
// The game's own frame-rate counter is left alone: orb's numbers go in the black beside
// the game, not over it, so the two do not collide. Worth recording where its switch is,
// in case that changes: the counter is drawn unless `g_Supervisor.isInEnding` is set —
// which is the same flag orb reads to know when to run an ending out, so writing it to
// hide the counter tells the game it is in the ending for good.

mod game_manager {
    pub const DIFFICULTY: usize = 0x10;
    pub const IS_IN_REPLAY: usize = 0x1c;
    pub const DEATHS: usize = 0x20;
    pub const CURRENT_POWER: usize = 0x1810;
    pub const LIVES_REMAINING: usize = 0x181a;
    pub const BOMBS_REMAINING: usize = 0x181b;
    pub const IS_IN_GAME_MENU: usize = 0x181f;
    pub const IS_IN_RETRY_MENU: usize = 0x1820;
    pub const IS_IN_PRACTICE_MODE: usize = 0x1823;
    pub const DEMO_MODE: usize = 0x1824;
    pub const RANDOM_SEED: usize = 0x1a2c;
    pub const GAME_FRAMES: usize = 0x1a30;
    pub const CURRENT_STAGE: usize = 0x1a34;
    pub const ARCADE_REGION_TOP_LEFT: usize = 0x1a3c;
    pub const ARCADE_REGION_SIZE: usize = 0x1a44;
}

mod stage {
    /// `skyFog.color`, the colour the background clears to.
    pub const SKY_FOG_COLOR: usize = 0x48;
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
    pub const PLAYER_STATE: usize = 0x9e0;
}

mod gui {
    pub const IMPL: usize = 0x4;
    pub const BOSS_PRESENT: usize = 0x20;
    /// Inside `GuiImpl`, at `msg.currentMsgIdx`.
    pub const IMPL_CURRENT_MSG_IDX: usize = 0x253c;
}

mod sound_player {
    /// `CStreamingSound *backgroundMusic`.
    pub const BACKGROUND_MUSIC: usize = 0x62c;
    pub const BACKGROUND_MUSIC_THREAD_ID: usize = 0x614;
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
const STATE_ENDING: i32 = 10;

/// `th06::PlayerState`.
const PLAYER_SPAWNING: i8 = 1;
const PLAYER_DEAD: i8 = 2;

impl Game for Th06 {
    fn hooks(&self) -> Hooks {
        Hooks {
            update: Patch { target: RUN_CALC_CHAIN, prologue: RUN_CHAIN_PROLOGUE },
            draw: Patch { target: RUN_DRAW_CHAIN, prologue: RUN_CHAIN_PROLOGUE },
            save_replay: Some(Patch { target: SAVE_REPLAY, prologue: SAVE_REPLAY_PROLOGUE }),
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

            State {
                scene,
                playing: scene == STATE_GAMEMANAGER,
                in_game: scene == STATE_GAMEMANAGER && !demo && !replay,
                in_ending: scene == STATE_ENDING
                    || mem::read::<i32>(G_SUPERVISOR + supervisor::IS_IN_ENDING) != 0,
                demo,
                replay,
                practice: mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_PRACTICE_MODE) != 0,
                // `isInMenu` is set while the game is *running*; the pause and
                // retry menus are the two flags that mean it has stopped.
                paused: mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_GAME_MENU) != 0
                    || mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_RETRY_MENU) != 0,
                // Being invulnerable after a bomb or a respawn does not count.
                unsettled: player_state == PLAYER_DEAD || player_state == PLAYER_SPAWNING,
                in_dialogue: dialogue_msg_idx() >= 0,
                stage: mem::read(G_GAME_MANAGER + game_manager::CURRENT_STAGE),
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

    /// From `GAME_REGION_*`, inside the game's 640x480 output.
    fn play_area(&self) -> Rect {
        Rect { left: 32.0, top: 16.0, width: 384.0, height: 448.0 }
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
    fn clears_background(&self) -> bool {
        let opts = unsafe { mem::read::<u32>(G_SUPERVISOR + CFG_OPTS) };
        opts >> GCOS_CLEAR_BACKBUFFER_ON_REFRESH & 1 != 0
            || opts >> GCOS_DISPLAY_MINIMUM_GRAPHICS & 1 != 0
    }

    /// The live `CStreamingSound`, checked before being believed: the allocator
    /// does not scrub freed blocks, so a stale pointer reads back as its old self.
    fn streaming_sound(&self) -> Option<usize> {
        let streaming =
            unsafe { mem::read::<usize>(G_SOUND_PLAYER + sound_player::BACKGROUND_MUSIC) };
        mem::vtable_in_image(streaming).then_some(streaming)
    }

    fn wave_file(&self) -> Option<usize> {
        let streaming = self.streaming_sound()?;
        let wave_file = mem::read_committed::<usize>(streaming + streaming_sound::WAVE_FILE)?;
        mem::read_committed::<u32>(wave_file + wave_file::SIZE).map(|_| wave_file)
    }
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
