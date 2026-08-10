//! A 東方妖々夢 that is not the real one, and far less of one than [`th06`](super::th06) is.
//!
//! **It plays as much of the game's part as `Th07` reads.** Which is a frame and nothing else: its
//! memory holds the device orb draws through, the window orb asks about and the chain the two frame
//! calls are made at, and its own loop calls them in 妖々夢's own draw-then-update order. Everything a
//! run is made of is missing because `Th07` answers all of it without looking — no scene to be in, no
//! stage, no score file, no pad — so there is nothing for this game to be doing.
//!
//! What that makes it good for is the one thing worth asking of a second game: that the seam holds. orb
//! attached to a game it has read a frame of gets in, has its two hooks reached inside that game's own
//! frame, and writes its own account of the run — and paces nothing and draws nothing, there being no
//! frame loop of orb's here and no font beside the exe to build an overlay from. Over `Launch`, which is
//! the same half of a launch th06's e2e tests run on. See
//! [docs/adr/0004](../../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).
//!
//! Where a number here is 妖々夢's it says so. There are two: the answers its chain walk gives, and what
//! its own frame answers to the loop above it.

use std::cell::Cell;
use std::ffi::c_void;

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th07::Th07;
use orb_core::game::th07::image::Image;

use super::{DRAW, Display, Launch, Launched, PRESENT, SOUND, UPDATE, WINDOW, Work, scratch};

/// What the game's own chain walk answers while the game is running, and what it answers when the game
/// is leaving: nothing, and a walk that failed. All three read off `th07.exe` — 0x4347f1 turns a zero
/// into the frame answering 1, and 0x434801 turns a `-1` into 2.
const CHAIN_CARRIED_ON: i32 = 1;
pub const CHAIN_LEFT: i32 = 0;
pub const CHAIN_FAILED: i32 = -1;

/// And what a whole frame answers, which the message pump reads: zero to be called again, and the two
/// above zero to leave. 妖々夢's own, at 0x4346f2, 0x4347f7 and 0x434807.
pub const FRAME_KEPT_RUNNING: i32 = 0;
pub const FRAME_LEFT: i32 = 1;
pub const FRAME_FAILED: i32 = 2;

/// A 妖々夢 laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// What this game's chain walk answers. [`CHAIN_CARRIED_ON`] for the whole of a launch here: no
    /// e2e test over this game asks it to leave, 妖々夢's half being a frame and nothing else. A `Cell`
    /// all the same, because what reads it is a bare `extern` function with the ABI's arguments and
    /// nothing else — the same shape 紅魔郷's is, where an e2e test does move it.
    answers: Cell<i32>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory.
    _installed: orb_api::Installed,
}

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into — plain `extern` functions
    /// with nothing but the ABI's arguments, so where the real game would reach its own globals this
    /// reaches the game.
    static RUNNING: Cell<*const Fake> = const { Cell::new(std::ptr::null()) };
}

fn running() -> &'static Fake {
    let fake = RUNNING.get();
    assert!(!fake.is_null(), "no game has been attached on this thread");
    unsafe { &*fake }
}

impl Launched for Fake {
    fn frame_window(&self) -> *mut c_void {
        self.image.game_window_object() as *mut c_void
    }

    fn own_render(&self) -> i32 {
        own_render(self.frame_window())
    }

    fn sim(&self) -> &orb_sim::Sim {
        self.image.sim()
    }

    fn launch(&self) -> &Launch {
        &self.launch
    }
}

impl Fake {
    /// Lays the game out on a display an e2e test says the whole of, gives it a device, and attaches orb
    /// to it the way a launch there is attached: **the game's own frame loop, with orb's update and draw
    /// hooks inside it.**
    ///
    /// Not orb's own loop, because `Th07::hooks` declines `render` — a run measured orb's loop taking
    /// 妖々夢 down on its first frame, and an e2e test driving a loop production does not install would be
    /// asserting about a configuration nobody gets. `work` is what the game's own frame costs, which is
    /// what the clock moves by.
    ///
    /// Boxed and never moved for the reason th06's is: the hooks find it through a pointer.
    pub fn attach(display: Display, name: &str, work: Work) -> Box<Self> {
        let dir = scratch(name);
        // The simulated Windows first, `orb.yaml` being a file of orb's own and so read through the file
        // seam — the same order 紅魔郷's has, and for the same reason.
        let image = Image::laid_out_seeded(display.seed);
        let installed = image.enter();
        image.sim().files().make(&dir);
        let mut config =
            Config::load_beside(&dir.join(EXE)).expect("a directory with no orb.yaml in it");
        // The memory hooks patch an import table there is none of.
        config.track_memory = false;
        // As `Th07::hooks` leaves it: the game's own frame runs and orb's two hooks are inside it.
        config.own_frame_loop = false;
        // The font is there for the harness's own device and gone before orb reaches for one: **a 妖々夢
        // install has no `font.ttf` beside its exe.** 紅魔郷 ships one and 妖々夢 keeps its fonts inside
        // `th07.dat`, so orb's overlay cannot be built there — measured on the machine as `overlay:
        // cannot load …\font.ttf` eight times and then `overlay: unavailable`. A launch with a font
        // here would have orb drawing things no launch of this game draws.
        let font = dir.join("font.ttf");
        image.sim().text().install_font(&font);
        let launch = Launch::new(image.sim(), dir, display.seed, config.own_frame_loop);
        image.sim().text().remove_font(&font);
        image.shows_through(launch.device(), WINDOW);
        // Where the game is installed, which is where orb writes its log.
        image.sim().set_host_exe(launch.dir().join(EXE));
        // The display, in front of orb being attached: `configure` reads the desktop's own rate before
        // there is a window to ask about, and a rate written down after that would be read a second
        // late — the first second of the run paced against nothing.
        let sim = image.sim();
        sim.display().set_monitor_hz(display.monitor_hz);
        sim.display().set_desktop_hz(display.monitor_hz);
        sim.display().set_foreground(WINDOW);
        if let Some(hz) = display.compositor_hz {
            sim.display().attach_compositor(
                sim.clock().peek(),
                sim.clock().frequency() / i64::from(hz),
                display.compose,
            );
        }
        if display.metronome {
            sim.display().as_a_metronome();
        }

        let fake = Box::new(Self {
            image,
            launch,
            answers: Cell::new(CHAIN_CARRIED_ON),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        unsafe {
            orb_core::runtime::attach_to(
                the_game_this_is(),
                config,
                fake.image.data(),
                orb_core::runtime::Originals {
                    update,
                    draw,
                    input,
                    stage_building,
                    stage_begun,
                    unlocks_read,
                    ranking_read,
                    render: own_render,
                    play_sounds,
                    present,
                    create_window,
                    create_file,
                    stop_recording,
                    create_game_window,
                    joystick_position,
                    get_controller_input,
                    save_replay,
                    init_d3d_device,
                },
            )
        };
        fake.frame_takes(work);
        fake
    }

    /// The whole of what this game does in one update: spend the frame's own work.
    ///
    /// Nothing else, because nothing else is read. `Th07::read_state` answers that no run is in
    /// progress without looking at the memory, so a scene written here would be a scene nothing could
    /// see — and inventing one would be this game claiming to know something the exe has not been read
    /// for.
    fn update(&self) {
        self.sim().clock().advance_micros(self.work_this_frame());
    }
}

impl Drop for Fake {
    /// The game closing: the runtime first, so orb's overlay is released through a device that is still
    /// there, then the fields in the order they are declared.
    fn drop(&mut self) {
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The exe this game is running as, which is the file name 妖々夢 ships under, and the `Game` orb is
/// attached to found out of the table by it.
///
/// Asked rather than named for the same reason 紅魔郷's is — see `fake::th06::the_game_this_is`: what says
/// which game a process is, in a launch and here, is the exe's own name.
///
/// # Panics
/// Where no entry holds that name.
pub const EXE: &str = "th07.exe";

fn the_game_this_is() -> &'static dyn orb_core::game::Game {
    orb_core::game::found(EXE)
        .unwrap_or_else(|| panic!("{EXE} is a game orb knows"))
        .game
}

extern "fastcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.update();
    fake.answers.get()
}

/// `GameWindow::Render` at 0x4346e0: the game's own whole frame, in 妖々夢's own draw-then-update order.
///
/// What orb's frame loop replaced — doing the update first is the frame of input lag removed — and what
/// that loop hands a frame back to on each of the three ways out of it that return: no runtime, no
/// device, and a chain target that is null.
///
/// The chain's two exits become the frame's own two here as they do in the real one, that mapping being
/// 妖々夢's and not orb's. Without the frame-skip loop the real one has around them: what orb's loop
/// replacing that frame does is take the skipping out, so a frame here is one draw and one update.
///
/// No wait in it, for the reason th06's has none: the real one paces itself and this one is called a
/// frame at a time by whatever is driving the game.
extern "fastcall" fn own_render(_window: *mut c_void) -> i32 {
    let chain = Th07.chain();
    unsafe { orb_core::runtime::run_draw_chain(chain) };
    let walked = unsafe { orb_core::runtime::run_calc_chain(chain) };
    unsafe { play_sounds(0) };
    if walked == CHAIN_LEFT {
        return FRAME_LEFT;
    }
    if walked == CHAIN_FAILED {
        return FRAME_FAILED;
    }
    unsafe { present(0) };
    FRAME_KEPT_RUNNING
}

/// `SoundPlayer::PlaySounds` at 0x44c9c0: the time an e2e test says this frame's sounds cost, a laid-out
/// game having no sound system to spend it in.
///
/// Here rather than left out because the frame loop calls it where the game's own loop did, and a frame
/// that skipped it would be one span of the pacing's breakdown short. What it costs is spent here and
/// not with the rest of the frame's work, as in 紅魔郷's — see [`Work::sound_us`].
unsafe extern "fastcall" fn play_sounds(_player: usize) {
    let fake = running();
    fake.launch.asked_for(SOUND);
    fake.sim().clock().advance_micros(fake.sound_this_frame());
}

/// `GameWindow::Present` at 0x4345c0: the frame handed over, which from here is the compositor's.
///
/// Where an e2e test counts a frame — the tick it was handed over at, which is what a rate is read off —
/// and where the host is told, since the next flush is what waits for this frame to be composed.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
}

extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    running().launch.asked_for(DRAW);
    CHAIN_CARRIED_ON
}

/// The keys the game sees. None: `Th07::hooks` leaves `input` `None`, so orb never stands in front of
/// this read and nothing here is ever called through it.
extern "system" fn input() -> u16 {
    0
}

/// The four seams `Th07` declines, as functions that exist so [`orb_core::runtime::Originals`] can be
/// filled.
///
/// Every one of them is `None` in `Th07::hooks`, so nothing patches them and nothing calls them — the
/// recording's own teardown among them, this game having no record to end. They answer what the real
/// callbacks answer for success, which is what makes them harmless rather than merely unreached.
extern "C" fn stage_building(_stage: i32) -> i32 {
    0
}

extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    0
}

extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    0
}

extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    0
}

extern "C" fn stop_recording() {}

/// And its own `GameWindow::Create`, which is never called: nothing of 妖々夢 is laid out but a frame, and
/// this game asks for no window.
extern "C" fn create_game_window(_instance: *mut c_void) {}

/// And its own `joyGetPosEx`, which is never called either: `Th07` declines the pad along with everything
/// else about a run, so nothing here ever reads one.
unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

/// And its own `Controller::GetControllerInput`, `ReplayManager::SaveReplay` and device setup, none of which
/// is ever called: this game is a frame and nothing else — see this file's own head — so the whole of what it
/// hands over past the frame's four calls is handed over to be a valid `Originals` and for nothing more.
extern "C" fn get_controller_input(buttons: u32) -> u16 {
    buttons as u16
}

extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

extern "C" fn init_d3d_device() {}

/// And its `CreateFileA`, which is never called either: `Th07` declines the score file the same way it
/// declines everything else about a run, so nothing of 妖々夢's is ever opened through orb.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_file(
    _name: *const u8,
    _access: u32,
    _share: u32,
    _security: *const c_void,
    _disposition: u32,
    _flags: u32,
    _template: *mut c_void,
) -> *mut c_void {
    std::ptr::null_mut()
}

/// And its `CreateWindowExA`, which is never called for a different reason: **this game never creates a
/// window.** `../th07.rs` is a launch of 妖々夢 and nothing of its window has been read, so the ask
/// is not here to be rewritten — orb's rewrite waits for a call that does not come, which is the same
/// thing a launch with no window in it does.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_window(
    _ex_style: u32,
    _class_name: *const u8,
    _window_name: *const u8,
    _style: u32,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
    _parent: *mut c_void,
    _menu: *mut c_void,
    _instance: *mut c_void,
    _param: *const c_void,
) -> *mut c_void {
    std::ptr::null_mut()
}
