//! A 東方妖々夢 that is not the real one, and far less of one than [`th06`](super::th06) is.
//!
//! **It plays as much of the game's part as `Th07` reads.** Which is a frame and nothing else: its
//! memory holds the device orb draws through, the window orb asks about, the chain the two frame calls are
//! made at and the queue of quads its drawing fills, and its own frame does what 妖々夢's does in
//! 妖々夢's own draw-then-update order. Everything a run is made of is missing because `Th07` answers all
//! of it without looking — no scene to be in, no stage, no pad, and a score file that is one name and no
//! contents — so there is nothing for this game to be doing.
//!
//! What that makes it good for is the two things worth asking of a second game: that the seam holds, and
//! that the frame orb composes is that game's frame. orb attached to a game it has read a frame of gets
//! in, paces it, and has its update and draw reached inside orb's own frame — and draws nothing, there
//! being no font beside the exe to build an overlay from. **Its draw chain queues a quad**, which is why
//! the second of those can be asked at all: a frame missing what the game's own does around its drawing
//! faults here as it faulted on the machine. Over `Launch`, which is the same half of a launch th06's e2e
//! tests run on. See
//! [docs/adr/0004](../../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) and
//! [docs/adr/0017](../../../../docs/adr/0017-the-frame-loop-has-a-seam-either-side-of-the-draw-chain.md).
//!
//! Where a number here is 妖々夢's it says so. There are two: the answers its chain walk gives, and what
//! its own frame answers to the loop above it.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_void};

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th07::Th07;
use orb_core::game::th07::image::Image;

use super::{
    DRAW, Display, Launch, Launched, PRESENT, Presented, SOUND, UPDATE, WINDOW, Work, scratch,
};

/// What the game's own chain walk answers while the game is running, and what it answers when the game
/// is leaving: nothing, and a walk that failed. All three read off `th07.exe` — 0x4347f1 turns a zero
/// into the frame answering 1, and 0x434801 turns a `-1` into 2.
const CHAIN_CARRIED_ON: i32 = 1;
pub const CHAIN_LEFT: i32 = 0;
pub const CHAIN_FAILED: i32 = -1;

/// The score file 妖々夢 asks for, which is the one name this game passes `CreateFileA`: `score.dat`,
/// beside its exe, opened twice for reading and once for writing in the launch `docs/adr/0004` reports.
///
/// The name the game asks for and not the name an open lands in — which of the two files that is is
/// orb's decision, and a game that named the forked file itself would be answering the question the e2e
/// test over it asks.
const SCORE_FILE: &CStr = c"score.dat";

/// `GENERIC_WRITE`, which is the one bit of `CreateFileA`'s access an open here varies: orb's fork reads
/// it to say which of the game's opens this is.
const GENERIC_WRITE: u32 = 0x4000_0000;

/// What the keyboard's half of the read answers on a frame with nothing held, which is every frame of
/// these e2e tests: nothing of 妖々夢's keyboard has been read, so this is the word the pad's bits are
/// added to — see [`Fake::update`].
const NO_KEY_HELD: u16 = 0;

/// The class 妖々夢 registers its window with, which is 紅魔郷's too: `BASE`, at 0x497bd0 in this exe. What
/// orb recognises the window it plays in by — see `orb_core::window::is_game_class`, and the size the game
/// asks for is [`Th07::content_size`]'s, orb deciding what it gets instead.
const WINDOW_CLASS: &CStr = c"BASE";

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
    /// How many times the game has been made to tell its device the fog is off — see
    /// [`puts_the_fog_out`].
    fog_told: Cell<u32>,
    /// The name every open of this game's score file landed in, in the order they happened — see
    /// [`create_file`].
    opens: RefCell<Vec<String>>,
    /// Every present the game's device was asked for — see [`device_present`].
    presents: RefCell<Vec<Presented>>,
    /// Whether orb stands in front of this game's pad read, and the word that read last answered — see
    /// [`Fake::update`].
    reads_the_pad: bool,
    /// And whether it stands in front of the read that screen makes — see [`Fake::reads_the_pad_buttons`].
    reads_the_pad_by_number: bool,
    word: Cell<u16>,
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
    /// to it the way a launch there is attached: **orb's own frame loop, with the game's update and draw
    /// inside it.**
    ///
    /// `work` is what the game's own frame costs, which is what the clock moves by.
    ///
    /// Boxed and never moved for the reason th06's is: the hooks find it through a pointer.
    pub fn attach(display: Display, name: &str, work: Work) -> Box<Self> {
        Self::attach_configured(display, name, work, |_| ())
    }

    /// And the same launch with a setting of its own answered — the screen, for an e2e test about the
    /// shape a client of that shape gets the game in.
    ///
    /// A closure rather than a `Config`, so that what an e2e test says is the one thing it is about and
    /// everything else is what a launch in a directory with no `orb.yaml` brings.
    pub fn attach_configured(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
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
        settings(&mut config);
        // The font is there for the harness's own device and gone before orb reaches for one: **a 妖々夢
        // install has no `font.ttf` beside its exe.** 紅魔郷 ships one and 妖々夢 keeps its fonts inside
        // `th07.dat`, so orb's overlay cannot be built there — measured on the machine as `overlay:
        // cannot load …\font.ttf` eight times and then `overlay: unavailable`. A launch with a font
        // here would have orb drawing things no launch of this game draws.
        let font = dir.join("font.ttf");
        image.sim().text().install_font(&font);
        let launch = Launch::new(
            image.sim(),
            dir,
            display.seed,
            super::orbs_own_loop(&config, the_game_this_is()),
        );
        image.sim().text().remove_font(&font);
        image.shows_through(launch.device(), WINDOW);
        // The three calls orb's frame makes into this game, which is what a game laid out by hand has to
        // hand over: code is the one thing an address space cannot hold.
        image.hands_over_the_frames_own_calls(
            empties_the_queue as *const () as usize,
            draws_the_queue as *const () as usize,
            puts_the_fog_out as *const () as usize,
        );
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
        // And the monitor the window goes on, before orb is attached and for the reason the rate is: the
        // attach is where orb says this process reads sizes as real pixels. Declared for every launch here
        // rather than by the e2e tests that want one — 妖々夢 makes a window on any machine, and a launch
        // with no monitor is one where orb leaves the window as the game asked for it, which is a th06 e2e
        // test's subject and not one of these.
        sim.windows()
            .set_monitor(orb_sim::Monitor::scaled(), orb_sim::Frame::SOME);

        let fake = Box::new(Self {
            image,
            launch,
            answers: Cell::new(CHAIN_CARRIED_ON),
            fog_told: Cell::new(0),
            opens: RefCell::new(Vec::new()),
            presents: RefCell::new(Vec::new()),
            reads_the_pad: the_game_this_is().hooks().joystick.is_some(),
            reads_the_pad_by_number: the_game_this_is().hooks().pad_buttons.is_some(),
            word: Cell::new(0),
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
                    pad_buttons,
                    save_replay,
                    init_d3d_device,
                },
            )
        };
        fake.frame_takes(work);
        fake
    }

    /// This game's memory, for an e2e test that reads back what a frame of orb's did to it — the queue of
    /// quads and the fog, which are the whole of what a frame here touches.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// The game creating the window it plays in, which is one `CreateWindowExA` on the class orb
    /// recognises.
    ///
    /// Straight through orb's rewrite of that import rather than through a hook over the game's own
    /// creation: `Th07::hooks` declines `create_window`, nothing of 妖々夢's display setting having been
    /// read to overrule, so the import is the whole of what orb stands in front of here. What orb does with
    /// the call is decide the window's shape and place — see `orb_core::window`.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        let (width, height) = Th07.content_size();
        let window = unsafe {
            orb_core::window::create_window_ex_a(
                0,
                WINDOW_CLASS.as_ptr().cast(),
                c"th07".as_ptr().cast(),
                0,
                0,
                0,
                i32::try_from(width).unwrap(),
                i32::try_from(height).unwrap(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        orb_api::Hwnd(window as usize)
    }

    /// And the device it shows through, with the vtable orb redirects the `Present` slot of laid out under
    /// it — then orb's own device setup, where that redirection happens.
    ///
    /// The device object and its vtable are laid out here because orb reaches through them:
    /// `window::hook_device` reads the object for its vtable and swaps the slot, so a device that was only
    /// a pointer is a read of an address nothing has mapped. Which is also why this is a step of its own —
    /// the launch is attached before the game has a device, as a real one is.
    pub fn finds_its_device(&self) {
        let device = self.launch.device();
        let vtable = self.launch.vtable();
        let space = self.image.space();
        space.map(device.0, size_of::<usize>(), orb_api::Kind::Private);
        space.write::<usize>(device.0, vtable);
        space.map(vtable, super::DEVICE_VTABLE_BYTES, orb_api::Kind::Image);
        space.write_bytes(
            vtable,
            &self
                .launch
                .vtable_bytes(device_present as *const () as usize),
        );
        self.image.shows_through(device, WINDOW);
        // orb's own device setup only where a launch installs it over the game's, which is what
        // `Hooks::init_device` says: a game that declines it is one orb never gets in front of, so the
        // `Present` slot stays the one Direct3D put there and the frames are stretched over the client.
        // Calling it anyway would be this game driving a hook production does not install — the same
        // thing `orbs_own_loop` is for.
        if the_game_this_is().hooks().init_device.is_some() {
            orb_core::runtime::init_d3d_device();
        }
    }

    /// Every present the game's device has been asked for, in the order it was asked.
    pub fn presented(&self) -> Vec<Presented> {
        self.presents.borrow().clone()
    }

    pub fn forget_presents(&self) {
        self.presents.borrow_mut().clear();
    }

    /// How many frames have taken the fog off to the device, which is not the same as how many put the
    /// field out: see [`puts_the_fog_out`].
    pub fn fog_told_to_the_device(&self) -> u32 {
        self.fog_told.get()
    }

    /// The game opening its own score file, which is one `CreateFileA` on the name it asks for — and
    /// which 妖々夢 does at its front end and again on the way out, measured as three opens of `score.dat`
    /// in the launch `docs/adr/0004` reports.
    ///
    /// Answers the name the open landed in, which is orb's decision and not this game's.
    ///
    /// Through orb's rewrite where the launch installed one over the game's import entry, and straight
    /// into this game's own where it did not — a launch with nothing to fork patches nothing, and an
    /// unpatched entry is the one thing a game with no import table cannot have.
    pub fn opens_its_score_file(&self, write: bool) -> String {
        let access = if write { GENERIC_WRITE } else { 0 };
        let open = if orb_core::score::installed() {
            orb_core::score::create_file_a
        } else {
            create_file
        };
        let handle = unsafe {
            open(
                SCORE_FILE.as_ptr().cast(),
                access,
                0,
                std::ptr::null(),
                0,
                0,
                std::ptr::null_mut(),
            )
        };
        assert_ne!(handle, std::ptr::null_mut(), "the open was refused");
        self.opens.borrow()[handle as usize - 1].clone()
    }

    /// The game's own key config read, made where that screen makes it: the array's address, with whatever
    /// the game found in it and whatever orb added.
    ///
    /// Through orb's hook only where a launch installs it, for the reason `orbs_own_loop` is: a game whose
    /// `hooks` declines that read is one orb is not in front of, and what its screen then walks is what its
    /// own read left — which here is an empty array.
    pub fn reads_the_pad_buttons(&self) -> usize {
        if self.reads_the_pad_by_number {
            orb_core::runtime::pad_buttons() as usize
        } else {
            pad_buttons() as usize
        }
    }

    /// The word the game's own input read came back with on the frame just run, for an e2e test that
    /// pushed a pad.
    pub fn word_the_input_read_answered(&self) -> u16 {
        self.word.get()
    }

    /// The whole of what this game does in one update: spend the frame's own work.
    ///
    /// Nothing else, because nothing else is read. `Th07::read_state` answers that no run is in
    /// progress without looking at the memory, so a scene written here would be a scene nothing could
    /// see — and inventing one would be this game claiming to know something the exe has not been read
    /// for.
    fn update(&self) {
        // The input read the game's own update makes, which is where orb's pad half is: the keyboard read
        // at 0x4309c0 tail-calls the pad read, and orb answers that whole call. Nothing of this game's own
        // keyboard goes in — nothing of 妖々夢's has been read, `Th07::hooks` declining `input` — so what
        // is asked is the word a frame with no key held has, and what comes back is what the pads did to
        // it.
        //
        // Only where a launch installs that hook, for the reason `orbs_own_loop` is: a game whose pad read
        // orb does not stand in front of is one whose own read answers, and this game has none to answer
        // with.
        if self.reads_the_pad {
            self.word
                .set(orb_core::runtime::get_controller_input_in_ecx(NO_KEY_HELD));
        }
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
/// What orb's frame loop replaces — doing the update first is the frame of input lag removed — and what
/// that loop hands a frame back to on each of the three ways out of it that return: no runtime, no
/// device, and a chain target that is null.
///
/// **Its own drawing is here in full**, because what a frame of 妖々夢 does around its draw chain is the
/// whole subject of `../th07.rs`: the queue of quads emptied at 0x434734, the word at 0x434739, the fog
/// put out at 0x434748, and the queue drawn at 0x43475d. A laid-out frame with those left out would fault
/// on its own first quad, which is what orb's loop did to the real game.
///
/// The chain's two exits become the frame's own two here as they do in the real one, that mapping being
/// 妖々夢's and not orb's. Without the frame-skip loop the real one has around them: what orb's loop
/// replacing that frame does is take the skipping out, so a frame here is one draw and one update.
///
/// No wait in it, for the reason th06's has none: the real one paces itself and this one is called a
/// frame at a time by whatever is driving the game.
extern "fastcall" fn own_render(_window: *mut c_void) -> i32 {
    let fake = running();
    let chain = Th07.chain();
    unsafe { empties_the_queue(0) };
    fake.image.forces_the_fog_call_through();
    unsafe { puts_the_fog_out(0) };
    unsafe { orb_core::runtime::run_draw_chain(chain) };
    unsafe { draws_the_queue(0) };
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
    // And the device's own `Present`, which is what this function does in the game — four nulls through
    // +0x3c at 0x4345df — and the one path orb's letterbox is on. The slot is read out of the vtable
    // rather than remembered, because what is in it is whatever orb last put there.
    //
    // Only where the vtable is laid out, which is a launch whose device has been found: orb's own frame
    // reaches this call from the first frame, and a game that never had a device presents through nothing.
    let device = fake.launch.device();
    if fake
        .image
        .space()
        .read_committed::<usize>(device.0)
        .is_some()
    {
        let slot: usize = fake.image.space().read(fake.launch.present_slot());
        let through: DevicePresent = unsafe { std::mem::transmute(slot) };
        unsafe {
            through(
                device.0 as *mut c_void,
                std::ptr::null(),
                std::ptr::null(),
                0,
                std::ptr::null(),
            )
        };
    }
}

/// The `Present` slot's own signature, which is `orb_core::window`'s private one written out again: a
/// laid-out game calling through that slot has to call it the way Direct3D would.
type DevicePresent = unsafe extern "system" fn(
    *mut c_void,
    *const orb_api::Rect,
    *const orb_api::Rect,
    isize,
    *const c_void,
) -> orb_api::Hresult;

/// What is in that slot until orb redirects it, and what an e2e test reads the redirection back through:
/// the two rectangles the ask carried.
///
/// `S_OK` always. What a driver that refuses to stretch does is th06's e2e tests' subject — the fallback is
/// orb's and not a game's, so one game asking it is enough.
unsafe extern "system" fn device_present(
    _device: *mut c_void,
    source: *const orb_api::Rect,
    destination: *const orb_api::Rect,
    _window_override: isize,
    _dirty: *const c_void,
) -> orb_api::Hresult {
    let fake = running();
    let read = |at: *const orb_api::Rect| (!at.is_null()).then(|| unsafe { *at });
    fake.presents.borrow_mut().push(Presented {
        source: read(source),
        destination: read(destination),
    });
    0
}

/// `Chain::RunDrawChain` at 0x42fe20: one quad queued, which is what drawing anything in 妖々夢 is.
///
/// **The whole of why this game has a queue laid out.** A real draw chain reaches 0x44f690 for every
/// sprite on the screen, and that function writes where `[queue + 0x17e534]` points — so a frame that
/// never emptied the queue writes its first quad to address zero, which is the fault the launch in
/// `docs/adr/0004` took. A laid-out game whose draw chain only said it had been asked would have passed
/// that frame and every frame after it.
extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.image.queues_a_quad();
    CHAIN_CARRIED_ON
}

/// The three the frame makes on that queue and on the fog, `__thiscall` on what the game keeps each of
/// them in: 0x44f580, 0x44f5c0 and 0x43a207.
///
/// What they do to this game's memory is `image`'s, the block being laid out there — see
/// [`Image::empties_the_queue`](orb_core::game::th07::image::Image::empties_the_queue). Handed over
/// rather than reached at their own addresses, for the reason every call out of the seam is: an address
/// space holds no code.
unsafe extern "thiscall" fn empties_the_queue(_queue: usize) {
    running().image.empties_the_queue();
}

unsafe extern "thiscall" fn draws_the_queue(_queue: usize) {
    running().image.draws_the_queue();
}

/// And the fog, which draws the queue first as the game's own does: what was queued under fog is drawn
/// under it, and putting the fog out before drawing it would draw it without.
///
/// **The field is what decides whether the device is told**, which is the game's own shape and the whole
/// of why its frame writes that field first — a call whose field already says the fog is out sets no
/// render state at all. What reaches the device is counted here rather than at the device itself: the
/// `SetRenderState` is not laid out, the device in an e2e test being a real object with a vtable of Rust
/// functions.
unsafe extern "thiscall" fn puts_the_fog_out(_supervisor: usize) {
    let fake = running();
    fake.image.draws_the_queue();
    if fake.image.fog_is_on() {
        fake.image.puts_the_fog_out();
        fake.fog_told.set(fake.fog_told.get() + 1);
    }
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

/// The game's own read of the pad's buttons by number, which its key config screen makes: the array
/// emptied and answered, and nothing put in it.
///
/// Nothing, because this game has no device of its own to fill it from — which is the launch the machine
/// really is, 妖々夢 finding no pad there. What goes in is what orb adds, and that is the whole subject of
/// the e2e test over it.
extern "C" fn pad_buttons() -> *mut c_void {
    let fake = running();
    fake.image.empties_the_pad_buttons();
    std::ptr::without_provenance_mut(fake.image.pad_buttons())
}

/// And its own `joyGetPosEx`, which is never called either: `Th07` declines the pad along with everything
/// else about a run, so nothing here ever reads one.
unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

/// And its own `ReplayManager::SaveReplay` and device setup, neither of which is ever called: this game is a
/// frame and nothing else — see this file's own head — so the whole of what it hands over past the frame's
/// four calls is handed over to be a valid `Originals` and for nothing more.
extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

extern "C" fn init_d3d_device() {}

/// And its `CreateFileA`, which is what the game's own opens of its score file go through.
///
/// It writes down the name the open landed in and answers a handle that is an index into that list: a
/// file this game keeps on no disk, so what an open is for here is which of the two names orb sent it
/// to. Every open succeeds — what a read of a file that is not there answers is nothing this game is
/// asked, `Th07` reading no score file of its own.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_file(
    name: *const u8,
    _access: u32,
    _share: u32,
    _security: *const c_void,
    _disposition: u32,
    _flags: u32,
    _template: *mut c_void,
) -> *mut c_void {
    let fake = running();
    let path = unsafe { CStr::from_ptr(name.cast()) }
        .to_string_lossy()
        .into_owned();
    let mut opens = fake.opens.borrow_mut();
    opens.push(path);
    std::ptr::without_provenance_mut(opens.len())
}

/// And its `CreateWindowExA`, which is what the window the game plays in is made through — with the
/// arguments orb decided rather than the ones this game asked for.
///
/// The host makes the window, so what an e2e test can then read of it is what any window of the simulated
/// Windows has: where it is, how big, and the client area inside it — which is what orb's letterbox is
/// worked out from.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_window(
    _ex_style: u32,
    _class_name: *const u8,
    _window_name: *const u8,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    _parent: *mut c_void,
    _menu: *mut c_void,
    _instance: *mut c_void,
    _param: *const c_void,
) -> *mut c_void {
    let asked = orb_api::Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    running().sim().windows().create_window(asked, style).0 as *mut c_void
}
