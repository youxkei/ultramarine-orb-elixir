//! 東方妖々夢, as a [`Game`] that answers what has been measured and declines the rest.
//!
//! `th07.exe`, 650752 bytes, `md5 0126afce1e805370d36c3482445e98da`. Every address below was read off
//! that exe with `i686-w64-mingw32-objdump -d -M intel` and nothing else: no table anybody published
//! and nothing carried across from 紅魔郷's addresses, which are where the same things happen to live
//! in another game. Each constant says what identifies it, because that is the whole of the evidence
//! there is — there is no decompilation of this build to cross-check against, as [`th06`](super::th06)
//! has.
//!
//! **What is measured is a frame, and nothing about a run.** The window, the device, the chain and its
//! two walks, the two calls a frame makes into the game, the play area and the size it renders at. So
//! orb *paces* 妖々夢 and does not *play* it: every method below that answers `None` or nothing is a
//! feature 完全無欠 does not have here — no chapters, no retry menu, no run picked up again, no card
//! counted — and the trait already says what each of those costs. See `docs/adr/0004`.
//!
//! **Nothing here panics.** An `unimplemented!()` in a method a frame reaches would take somebody's
//! game down at the moment orb ran out of measurements, which is the one thing an injected DLL must
//! not do. Where the trait offers no way to decline, the answer is the emptiest one that is *true* —
//! no regions, no music, no run in progress, no button worth writing down — and no method answers a
//! guess.

/// Reachable from this crate's own tests and, through the `sim` feature, from the tests of the
/// crates that drive it — which is where the scenarios live.
#[cfg(any(test, feature = "sim"))]
pub mod image;

use std::ffi::c_void;
use std::ops::Range;

use orb_api::Hwnd;
use orb_api::mem;

use crate::audio::Music;
use crate::d3d8::{Device, Viewport};
use crate::game::{
    Boundary, Call, FrameCalls, Game, Hooks, Menu, Pad, PanelTile, Patch, Reading, Rect,
    Reproduction, RunStart, RunState, State,
};

pub struct Th07;

/// `IDirect3DDevice8*`, and the `IDirect3D8*` it was made on.
///
/// Both fall out of one call: `IDirect3D8::CreateDevice` at 0x434d94, reached through the interface's
/// own vtable at +0x3c, with its seven arguments pushed from 0x434d70 down. The last pushed is
/// `ppReturnedDeviceInterface`, which is this address, and the `this` it is called on is the other.
///
/// A better witness than either would have been alone: the one call fixes the device, the interface it
/// came from and the window below, so none of the three rests on recognising a global by what is done
/// with it.
const G_D3D_DEVICE: usize = 0x00575958;

/// The game's window, and the object a frame is called on.
///
/// The `hFocusWindow` argument of that same `CreateDevice`, pushed at 0x434d7b — and also the object
/// [`RENDER`] is called on, at `mov ecx,0x575c20` in the message pump. So the handle is that object's
/// first field, as 紅魔郷's window object's is.
const G_GAME_WINDOW: usize = 0x00575c20;

/// `th07::Chain`, the object both walks below are called on.
///
/// The `this` of the two calls at 0x43474d and 0x4347da. What says it is a `Chain` of the shape orb
/// already knows is that the two walks start 0x20 apart — [`RUN_CALC_CHAIN`] from `this` and
/// [`RUN_DRAW_CHAIN`] from `this + 0x20` — which is 紅魔郷's layout arrived at from this exe rather
/// than assumed from that one.
const G_CHAIN: usize = 0x00626218;

/// `th07::Chain::RunCalcChain`, `__thiscall`. One call per logic frame, from [`RENDER`] at 0x4347da,
/// after the drawing.
///
/// Its answer is what the frame's own is made of, which is what identifies it: at 0x4347f1 a zero
/// makes the frame return 1 and at 0x434801 a `-1` makes it return 2, so nothing and a failed walk are
/// the game asking to stop. 紅魔郷's chain answers the same two.
const RUN_CALC_CHAIN: usize = 0x0042fd60;
/// `th07::Chain::RunDrawChain`, `__thiscall`. Called from [`RENDER`] at 0x43474d, between
/// `IDirect3DDevice8::BeginScene` (+0x88, at 0x434728) and `EndScene` (+0x8c, at 0x434788) — which is
/// what makes it the draw, and the only place an overlay may draw.
const RUN_DRAW_CHAIN: usize = 0x0042fe20;
/// `push ebp; mov ebp,esp; sub esp,0x14` — position-independent, 6 bytes, read out of the exe at both
/// addresses. The same prologue as 紅魔郷's two, which is MSVC6 and not a coincidence worth resting
/// anything on.
const RUN_CHAIN_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x14];

/// `th07::SoundPlayer::PlaySounds`, `__thiscall` on [`G_SOUND_PLAYER`]: the queued sound effects
/// handed to the sound system.
///
/// Called from the game's own frame at 0x4347e7, immediately after the calc chain, which is where
/// 紅魔郷 hands its own over. What it does with the object agrees: it answers nothing at once where
/// `[this + 0x610]` is zero and otherwise walks a list at `this + 0x73c`.
const PLAY_SOUNDS: usize = 0x0044c9c0;
const G_SOUND_PLAYER: usize = 0x004ba0d8;

/// `th07::GameWindow::Present`, taking nothing: the frame handed to the display.
///
/// `IDirect3DDevice8::Present(NULL, NULL, NULL, NULL)` through +0x3c at 0x4345df, the lost-device
/// `Reset` beside it at 0x434604, and the game's own screenshot key after it at 0x434621 — the three
/// things 紅魔郷's present does, in one function. The game's own frame calls it once per frame, at
/// 0x434a0e and 0x434a18, which are the two ways out of its frame-skip loop.
///
/// No `this`: it reads `ecx` nowhere, so the [`Call`] below carries nothing to call it on. `__thiscall`
/// with one argument is `fastcall` with nothing on the stack — see [`Call`] — and a `fastcall` whose
/// `ecx` the callee ignores is a `__cdecl` of no arguments called correctly.
const PRESENT: usize = 0x004345c0;

/// The size the game renders at.
///
/// `0x280` and `0x1e0` written into the shared `D3DVIEWPORT8` at [`VIEWPORT`] over the whole output,
/// at 0x4347ad inside [`RENDER`] and again at 0x448d70, and set through +0xa0 each time.
const BACK_BUFFER: (u32, u32) = (640, 480);

/// The `D3DVIEWPORT8` the game builds its viewports in, which is one struct reused: the whole output
/// at 0x4347ad, and whatever else asks. Written here as well as handed to the device, so that a
/// viewport orb sets is one the game's own next read of it agrees with.
const VIEWPORT: usize = 0x00575a18;

/// The play field, in the game's own 640x480 output.
///
/// The `D3DVIEWPORT8` built on the stack at 0x406ace — `0x20`, `0x10`, `0x180`, `0x1c0` — set through
/// +0xa0 at 0x406afc and cleared through +0x90 straight after it. The same rectangle 紅魔郷 has, read
/// off this exe rather than taken from that one.
const PLAY_AREA: Rect = Rect {
    left: 32.0,
    top: 16.0,
    width: 384.0,
    height: 448.0,
};

impl Game for Th07 {
    /// The two a frame is made of, and the frame itself. Everything else is `None`: eleven seams
    /// 紅魔郷 has and 妖々夢 has not been read for, each costing what [`Hooks`] says it costs.
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
            // **Declined, and a run is why.** 妖々夢's own whole frame is 0x4346e0, `__thiscall` on
            // 0x575c20, reached at `mov ecx,0x575c20; call 0x4346e0` from the message pump's idle
            // branch — the same shape 紅魔郷's `Render` has, which is what made replacing it look like
            // the same job. It is not. orb's loop in its place took the game down on the first frame:
            // `crash: code 0xc0000005 at 0x0044f6aa in th07.exe+0x4f6aa`, `writing 0x00000000` — a
            // `rep movsd` copying 28 bytes to `[this + 0x17e534]`, null, which is a render-state block
            // 妖々夢's frame sets up around its own drawing through two calls on 0x4b9e44 at 0x43472e
            // and 0x434757 that orb's loop does not make. The same launch under `--no-frame-loop`, which
            // leaves that frame alone with orb's update and draw hooks inside it, ran 600 frames and
            // left cleanly. So it is the loop and not the two patches above. See DONE.md.
            //
            // What replacing it would take is reading the rest of that frame rather than one more
            // address: those two calls, 0x43478e on the same object, 0x43a207 on 0x575950, and the
            // frame-skip loop at `[0x575c20 + 0x10]` against `[0x575a8b]` — measured running the update
            // twice a frame, `calls=1200` against `draw … calls=570` over 600 frames. None of it has
            // been read. Until it is, 妖々夢 gets its own cadence and keeps its frame of input lag.
            render: None,
            // A replay is neither written nor watched here: nothing has been read of where 妖々夢 keeps
            // its record of inputs, so there is nothing orb could suppress or protect.
            save_replay: None,
            stop_recording: None,
            // No stage of a run is written over, which is what leaves a run unable to be picked up
            // again — and with `run_slot` answering `None` there is nothing to write one into either.
            stage_begun: None,
            stage_building: None,
            // One score file, the game's own, whichever mode a launch says it is in. Both of these
            // exist to keep two files apart, and 妖々夢 has no second one until orb has a mode here.
            unlocks_read: None,
            ranking_read: None,
            // The game makes whatever window its own configuration asks for. `force_windowed` has
            // nowhere measured to write, so there would be nothing for this hook to do.
            create_window: None,
            // The device's `Present` is not redirected, so nothing of orb's is presented around the
            // game's own frame.
            init_device: None,
            // Every key the game reads reaches it. A run whose buttons are written down is a run that
            // can be picked up, and this game has none.
            input: None,
            joystick: None,
        }
    }

    /// No run, which is what orb knows of 妖々夢 rather than a claim about the game.
    ///
    /// The trait has no way to decline a `State`, and this is the emptiest one that is true of what has
    /// been measured: nothing is playing, so `Chapters::observe` sees no run, no snapshot is taken, no
    /// boundary is judged and no mark is drawn. A field filled from a th06 offset would be a number out
    /// of another game's memory, which is worse than nothing by exactly the amount somebody would trust
    /// it.
    unsafe fn read_state(&self) -> State {
        State {
            scene: 0,
            wanted: 0,
            playing: false,
            in_run: false,
            in_game: false,
            in_ending: false,
            ending_script: None,
            demo: false,
            replay: false,
            practice: false,
            paused: false,
            unsettled: false,
            bombing: false,
            in_dialogue: false,
            stage: 0,
            difficulty: 0,
            stage_frames: 0,
            script_frames: 0,
            random_seed: 0,
            deaths: 0,
            lives: 0,
            bombs: 0,
            power: 0,
            enemy_count: 0,
            bullet_count: 0,
            laser_count: 0,
            boss_present: false,
            boss_life: None,
            boss_attack_frames: None,
            spellcard: None,
        }
    }

    unsafe fn window(&self) -> Hwnd {
        unsafe { mem::read(G_GAME_WINDOW) }
    }

    unsafe fn d3d_device(&self) -> *mut Device {
        unsafe { mem::read(G_D3D_DEVICE) }
    }

    /// No music, which costs a chapter its track and there being no chapters costs nothing.
    fn music(&self) -> Option<Music> {
        None
    }

    fn music_identity(&self) -> Option<u32> {
        None
    }

    fn audio_thread(&self) -> Option<u32> {
        None
    }

    unsafe fn stop_music(&self) {}

    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    fn audio_state(&self) -> Vec<Range<usize>> {
        Vec::new()
    }

    /// No range of the game's memory holds a handle orb knows about, which is only reached by a restore
    /// and there is nothing here to restore.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        Vec::new()
    }

    fn play_area(&self) -> Rect {
        PLAY_AREA
    }

    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        None
    }

    unsafe fn repaint_lives_row(&self) {}

    /// Nothing of orb goes over the lives, so nothing has to be repainted under it.
    unsafe fn draws_lives_row(&self) -> bool {
        false
    }

    /// Nowhere, which is true of a game whose panel orb has not read: a rectangle of no size is one no
    /// quad covers, and with `draws_lives_row` false nothing is drawn against it anyway.
    fn lives_row(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn content_size(&self) -> (u32, u32) {
        BACK_BUFFER
    }

    /// Not windowed, which is the answer that leaves the game's own configuration alone: with
    /// [`Hooks::create_window`] `None` there is no hook to overrule it from, so this is read nowhere.
    fn windowed(&self) -> bool {
        false
    }

    unsafe fn force_windowed(&self) {}

    /// The play field's own viewport, and no clear.
    ///
    /// What the game does at 0x406afc is set this viewport and then clear it to opaque black through
    /// +0x90 — `D3DCLEAR_TARGET`, Z 1.0, no stencil, read off the pushes at 0x406b02. That clear is the
    /// stage wiping its own field rather than device setup being put back, and no depth clear beside it
    /// has been read, so putting one here would be orb inventing one. Reached only on the frames orb
    /// freezes the game for, of which 妖々夢 has none: it asks no question and offers no menu here.
    unsafe fn set_play_viewport(&self, device: *mut Device) {
        unsafe {
            let viewport = Viewport {
                x: PLAY_AREA.left as u32,
                y: PLAY_AREA.top as u32,
                width: PLAY_AREA.width as u32,
                height: PLAY_AREA.height as u32,
                min_z: 0.5,
                max_z: 1.0,
            };
            mem::write(VIEWPORT, viewport);
            let vtable = &*(*device).vtable;
            (vtable.set_viewport)(device, &viewport);
        }
    }

    fn chain(&self) -> *mut c_void {
        G_CHAIN as *mut c_void
    }

    /// Taken as not wiping it, so that orb cleans up after whatever it draws rather than leaving it to
    /// a wipe that may not come. Which of 妖々夢's options wipes the back buffer has not been read, and
    /// the wrong way round here is orb's own drawing hardening into the corners of a frame.
    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    /// The keyboard needs nothing done to it, which is what `true` says. With [`Hooks::input`] `None`
    /// the game's read is never stood in front of, so it is never lost and never re-acquired.
    unsafe fn acquire_input(&self) -> bool {
        true
    }

    /// Nothing was let go of. `--take-sent-keys` is for driving the game's own menus from another
    /// program, and orb puts no question over 妖々夢's.
    unsafe fn take_sent_keys(&self) -> bool {
        false
    }

    /// Nothing is written into the game's own copy of a joystick's caps. Where 妖々夢 keeps one, or
    /// whether it asks the device every time, has not been read — and writing 404 bytes into an address
    /// that is not that is how a game's memory gets scribbled on.
    fn joystick_calibration(&self) -> Option<usize> {
        None
    }

    /// The whole-output viewport the game sets at this point in its own frame.
    ///
    /// 0,0,640,480 written into the game's own `D3DVIEWPORT8` at 0x575a18 at 0x4347ad and set through
    /// +0xa0 at 0x4347d4, which is where its own frame does it: after the scene it drew into ends and
    /// before the update. No background
    /// clear, that being an option of the game's that has not been read, and no framerate multiplier,
    /// 妖々夢 not having been read for one.
    unsafe fn prepare_frame(&self, device: *mut Device) {
        unsafe {
            let viewport = Viewport {
                x: 0,
                y: 0,
                width: BACK_BUFFER.0,
                height: BACK_BUFFER.1,
                min_z: 0.0,
                max_z: 1.0,
            };
            mem::write(VIEWPORT, viewport);
            let vtable = &*(*device).vtable;
            (vtable.set_viewport)(device, &viewport);
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
                this: 0,
            },
        }
    }

    /// No stage to jump to: stepping is a replay's, and no replay is watched here.
    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn make_invulnerable(&self) {}

    /// Never a replay, which means orb offers a retry to every run it thinks it sees — and it sees
    /// none, `read_state` answering that nothing is playing.
    unsafe fn replaying(&self) -> bool {
        false
    }

    unsafe fn show_ranking(&self) {}

    unsafe fn showing_ranking(&self) -> bool {
        false
    }

    unsafe fn ranking_scene(&self) -> bool {
        false
    }

    unsafe fn restore_menu_cursor(&self) {}

    /// Said rather than numbered, there being no numbers: the line exists so that a trip through the
    /// ranking can be read afterwards, and no trip is taken here.
    unsafe fn ranking_state(&self) -> String {
        "nothing of 妖々夢's ranking screen has been measured".to_owned()
    }

    unsafe fn leave_ranking(&self) {}

    /// No card is counted. Where 妖々夢 keeps its record of spell cards has not been read, and a count
    /// written into an address that is not that would be a number in somebody's score file.
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        None
    }

    /// No record to hold across anything, which leaves the game's own alone through every snapshot and
    /// every ranking read — there being neither.
    unsafe fn captures(&self) -> Vec<u8> {
        Vec::new()
    }

    unsafe fn set_captures(&self, _saved: &[u8]) {}

    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// Nothing orb has a question about is ever under the cursor, so no press is ever held back and
    /// every one of 妖々夢's own menus works as it does without orb.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        None
    }

    unsafe fn menu_takes_a_press(&self) -> bool {
        false
    }

    /// A pad that is doing nothing, which drives the menus orb has here: none.
    unsafe fn pad(&self, _winmm: Option<Reading>) -> Pad {
        Pad::default()
    }

    /// No run to give up, the retry menu that would ask being one orb does not put up here.
    unsafe fn leave_run(&self) -> bool {
        false
    }

    unsafe fn swallow_input(&self) {}

    /// No bit is the decide, so nothing is taken out of the word the game reads and nothing is handed
    /// back for one read. Which is what a game orb asks no question of wants — see
    /// [`Game::menu_decide`] for what the bits are for.
    fn menu_decide(&self) -> u16 {
        0
    }

    fn menu_cancel(&self) -> u16 {
        0
    }

    unsafe fn skip_replay_prompt(&self) -> bool {
        false
    }

    /// No stage is running as far as orb knows, so no frame's buttons are written down against one.
    unsafe fn stage_frame(&self) -> Option<u32> {
        None
    }

    /// No bit of a frame's input is worth writing down, there being nothing to write it into: the word
    /// a run is recorded with is masked with this, so nothing is recorded.
    fn run_input(&self) -> u16 {
        0
    }

    /// Nothing, which is only asked with a run in progress and orb sees none.
    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: 0,
            character: 0,
            shot_type: 0,
            practice: false,
            stage: 0,
        }
    }

    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        None
    }

    /// No slot, which is the one answer that stops a run being kept at all: nothing to name is nothing
    /// to write, nothing to offer and nothing to mark. 妖々夢's characters and shots have not been read,
    /// and a file named from another game's numbering would be a chapter offered to the wrong run.
    fn run_slot(&self, _run: &RunStart) -> Option<String> {
        None
    }

    unsafe fn run_state(&self) -> RunState {
        RunState {
            score: 0,
            seed: 0,
            point_items: 0,
            power: 0,
            lives: 0,
            bombs: 0,
            rank: 0,
            power_items: 0,
            extra_lives: 0,
            deaths: 0,
        }
    }

    unsafe fn set_run_state(&self, _state: &RunState) {}

    unsafe fn set_run_seed(&self, _seed: u16) {}

    /// No run is ever chosen, so the frame a chapter would be put back on never arrives.
    unsafe fn run_chosen(&self) -> bool {
        false
    }

    unsafe fn start_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn run_finished(&self) -> bool {
        false
    }

    unsafe fn reproduction(&self) -> Reproduction {
        Reproduction {
            replay_frame: 0,
            input: 0,
            player: (0.0, 0.0),
            player_area: (0.0, 0.0),
            randoms: 0,
            seed: 0,
            items: 0,
            score: 0,
            extra_lives: 0,
            rank: 0,
            sub_rank: 0,
        }
    }

    /// No table, so no stage has a midstage boundary and the shape of one — script frame numbers on one
    /// clock — is a question that does not come due until 妖々夢 is a game orb plays. With
    /// `stage_begun` and `stage_building` `None` beside it, nothing reaches a chapter here at all.
    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &[]
    }
}
