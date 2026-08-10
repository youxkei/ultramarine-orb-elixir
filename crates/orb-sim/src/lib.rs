//! A simulated Windows, for the tests that run with no Windows under them.
//!
//! What orb asks of the host is answered here out of state a test laid out, rather than by the
//! OS. The point of it is not to avoid Windows for its own sake: it is that the suite then runs
//! on any host, so a Linux runner can say whether orb's logic is right and the Windows runner is
//! left with only the questions that are really about Windows.
//!
//! The simulated game is a client of this rather than a part of it — it takes its memory, and in
//! time its audio thread and its device, from here — so that a snapshot walks these pages the way
//! it walks the real ones, and the failures that only happen around a free or a suspended thread
//! stay reachable.

use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use orb_api::{
    Composition, Device, Face, Hresult, Hwnd, Locked, LockedBuffer, LogFile, Mask, Rect,
    SoundBuffer, Texture, Viewport, Win,
};

mod clock;
mod display;
mod drawing;
mod files;
mod joystick;
mod keyboard;
mod log;
mod mouse;
mod noise;
mod sound;
mod space;
mod text;
mod window;
pub use clock::{Clock, FREQUENCY};
pub use display::{Compose, Display, SPIKE_PERCENT, SPIKE_US, USUAL_US};
pub use drawing::{DEVICE, Drawn, Quad, Recording};
pub use files::Files;
pub use joystick::{Joystick, POV_CENTERED};
pub use keyboard::{Keyboard, keys};
pub use log::Log;
pub use mouse::Mouse;
/// The seeded stream the host's own unevenness is drawn from, for an e2e test that has unevenness of its
/// own to declare: how long the game's frame takes is the game's business rather than the host's, and a
/// run whose every draw comes from one seed is a run that replays.
pub use noise::Noise;
pub use sound::{BUFFER, BUFFER_VTABLE, Sound};
pub use space::Space;
pub use text::{Glyphs, Metric};
pub use window::{Frame, Made, Monitor, Windows, Written};

/// Where a test's laid-out game is installed. The log and `orb.yaml` are read as siblings of the
/// exe, so what matters is that it has a directory and a file name and that both come back
/// unchanged.
///
/// Joined rather than written out as one `C:\game\th06.exe`: a path is host-shaped, and a
/// Windows-spelled string is a single component to a `PathBuf` on Linux — `with_file_name` would
/// then replace the directory along with the file, and orb's log would come out somewhere no test
/// asked for. Joining leaves the separator to the host and the structure the same on both.
fn host_exe() -> PathBuf {
    PathBuf::from("game").join("th06.exe")
}

/// Which thread this is, as far as the seam can tell.
///
/// Handed out on first ask rather than derived from the real thread id, so it is the same number
/// on every host and never zero — which is what orb relies on to claim the frame's own thread with
/// nothing but a compare-and-swap.
fn thread_id() -> u32 {
    static NEXT: AtomicU32 = AtomicU32::new(1);
    thread_local! {
        static ID: u32 = NEXT.fetch_add(1, Ordering::Relaxed);
    }
    ID.with(|id| *id)
}

/// What orb has told this host the game allocated: the heap handles, and the ranges reserved straight
/// from the OS as `(base, len)`.
#[derive(Default)]
struct Noticed {
    heaps: Vec<usize>,
    reservations: Vec<(usize, usize)>,
}

/// The host a test puts in front of the real one.
///
/// Holds the game's memory, the clock and the log. The threads and the device belong here too as
/// they go behind the seam; a test reaches whichever of them it needs off the one value, because
/// they have to agree with each other — an audio thread that could be suspended without the memory
/// it was writing being the memory a snapshot reads would answer nothing worth asking.
pub struct Sim {
    space: Space,
    clock: Clock,
    display: Display,
    /// The panel and the frame it puts round a window, which the layout reads and the pacing does
    /// not — see [`Windows`].
    windows: Windows,
    keyboard: Keyboard,
    /// The mouse, and whether its pointer is being drawn — which is the game's window that orb hides
    /// one over, so it is the layout's neighbour rather than the pacing's.
    mouse: Mouse,
    /// The joystick winmm has, which is not the controller DirectInput has: that one is laid out in the
    /// game's own memory, and this is the device on the other branch of the game's own read.
    joystick: Joystick,
    /// The fonts an e2e test says are beside the game, and every string baked through one.
    glyphs: Glyphs,
    /// The device the game shows its frames through, keeping what it was asked to draw.
    drawing: Recording,
    log: Log,
    /// The files orb reads and writes of its own, kept rather than written — see [`Files`].
    files: Files,
    /// The ranges orb has said are its own — where it keeps the copies a snapshot holds. Nothing is
    /// ever excluded from anything here, `private_regions` answering with none; what the list is for
    /// is an e2e test asking whether the copies a chapter took were given back with it.
    ours: Mutex<HashMap<usize, usize>>,
    /// The threads the game has said it created. Nothing is ever stopped; what the list is for is an
    /// e2e test asking whether orb noticed one.
    threads: Mutex<Vec<u32>>,
    /// The heaps and the reservations orb has said the game took. Nothing is ever walked — laid-out
    /// memory answers `game_regions` on its own — and what the lists are for is the same as `threads`'.
    noticed: Mutex<Noticed>,
    /// The threads that have put themselves below the game's priority, by [`thread_id`]. Nothing is
    /// scheduled differently for it; what the list is for is an e2e test asking that a thread of orb's
    /// own said so, and that the frame's did not.
    below_normal: Mutex<Vec<u32>>,
    host_exe: Mutex<PathBuf>,
    /// What `module::proc_address` finds, keyed by module and name. Empty by default: a test that
    /// has not said `mmioSeek` is there is a test of what orb does without it, which is a case orb
    /// has to have an answer for.
    procs: Mutex<HashMap<(String, String), usize>>,
    /// Which modules are loaded. Kept apart from `procs` because orb tells a module that is not
    /// there from one that is there without the symbol, and says so in the log.
    modules: Mutex<Vec<String>>,
    /// What orb has put up as a modal, title and text, in the order it did.
    dialogs: Mutex<Vec<(String, String)>>,
    /// The code orb asked the host to end the process with, where it has. Written down instead of
    /// happening: a suite that really exited would take the harness's child with it, so the giving
    /// up is a thing an e2e test reads back rather than a thing it survives.
    exited: Mutex<Option<u32>>,
}

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

impl Sim {
    pub fn new() -> Self {
        Self::seeded(0)
    }

    /// With the host's non-determinism drawn from `seed`, which an e2e test names in its assertions so
    /// that a failure can be replayed.
    pub fn seeded(seed: u64) -> Self {
        Self {
            space: Space::new(),
            clock: Clock::new(),
            display: Display::new(seed),
            windows: Windows::new(),
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            joystick: Joystick::new(),
            glyphs: Glyphs::new(),
            drawing: Recording::new(),
            log: Log::new(),
            files: Files::default(),
            ours: Mutex::new(HashMap::new()),
            threads: Mutex::new(Vec::new()),
            noticed: Mutex::new(Noticed::default()),
            below_normal: Mutex::new(Vec::new()),
            host_exe: Mutex::new(host_exe()),
            procs: Mutex::new(HashMap::new()),
            modules: Mutex::new(Vec::new()),
            dialogs: Mutex::new(Vec::new()),
            exited: Mutex::new(None),
        }
    }

    /// The address space, for a test laying the game out or reading back what orb wrote.
    pub fn space(&self) -> &Space {
        &self.space
    }

    /// Puts this simulated Windows in front of the real host for as long as the answer is held,
    /// which is what makes orb's own reads land in it.
    ///
    /// Scoped rather than held for the sim's whole life, so that a test can lay out two games and
    /// each read the one it is asking about: the one entered is the one in front.
    ///
    /// Takes an `Arc` because the threads a simulated game runs on are given the same sim, not one
    /// each — an audio thread reading its own copy of the game would agree with nothing.
    pub fn enter(self: &std::sync::Arc<Self>) -> orb_api::Installed {
        let win: std::sync::Arc<dyn Win> = self.clone();
        orb_api::install(&win)
    }

    /// The clock, for a test that moves time on rather than waiting for it.
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// The display and its compositor, for a test that says what the pacing is pacing against.
    pub fn display(&self) -> &Display {
        &self.display
    }

    /// The panel and the frame round a window, for a test that says what orb's layout is laid out
    /// against — and reads back the windows it was asked to make.
    pub fn windows(&self) -> &Windows {
        &self.windows
    }

    /// The keyboard, for a test that presses a key at one of orb's own menus.
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// The mouse, for a test that moves the pointer over the game and reads back whether the host is
    /// drawing it.
    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }

    /// And the joystick winmm has, for a test that plugs a pad in — which is the branch the game reads
    /// a pad through where its own enumeration found no controller.
    pub fn joystick(&self) -> &Joystick {
        &self.joystick
    }

    /// The fonts and the strings baked through them, for an e2e test that says a font is beside the game
    /// or asks which string went into a texture.
    pub fn text(&self) -> &Glyphs {
        &self.glyphs
    }

    /// The device the game shows through, for an e2e test reading back what was drawn on a frame.
    pub fn drawing(&self) -> &Recording {
        &self.drawing
    }

    /// The quads that drew `text`, which is [`Recording::says`] with the bake this host answered it
    /// from — the two being one question and held apart only because a record of the drawing knows
    /// nothing about fonts.
    pub fn says(&self, text: &str) -> Vec<Quad> {
        self.drawing.says(&self.glyphs, text)
    }

    /// How many ranges orb is holding copies of the game's memory in, for an e2e test asking whether
    /// the ones a chapter took were given back with it.
    pub fn copies_held(&self) -> usize {
        self.ours.lock().unwrap().len()
    }

    /// The threads orb has said the game created.
    pub fn threads(&self) -> Vec<u32> {
        self.threads.lock().unwrap().clone()
    }

    /// And the ones that have put themselves below the game's priority, which is a thread of orb's own
    /// saying it is not the one to schedule first.
    pub fn threads_below_normal(&self) -> Vec<u32> {
        self.below_normal.lock().unwrap().clone()
    }

    /// The heaps orb has said the game allocated from, and the ranges it reserved straight from the OS
    /// as `(base, len)` — a range released again having been forgotten.
    pub fn noticed_allocations(&self) -> (Vec<usize>, Vec<(usize, usize)>) {
        let noticed = self.noticed.lock().unwrap();
        (noticed.heaps.clone(), noticed.reservations.clone())
    }

    /// The modals orb has put up, as title and text.
    pub fn dialogs(&self) -> Vec<(String, String)> {
        self.dialogs.lock().unwrap().clone()
    }

    /// The code orb asked the host to end the process with, and `None` where it has not asked.
    pub fn exited(&self) -> Option<u32> {
        *self.exited.lock().unwrap()
    }

    /// Puts a compositor there timing `hz`, with the monitor reporting the same, and the window in
    /// front — a display the pacing has no reason to refuse.
    ///
    /// `compose` is what the compositor takes over a frame, which is what the pacing's own allowance
    /// has to find.
    pub fn attach_display(&self, window: Hwnd, hz: u32, compose: Compose) {
        self.display.set_monitor_hz(Some(hz));
        self.display.set_desktop_hz(Some(hz));
        self.display.set_foreground(window);
        self.display.attach_compositor(
            self.clock.counter(),
            self.clock.frequency() / i64::from(hz),
            compose,
        );
    }

    /// Says the game has handed a frame over, which is what the next flush waits to be composed.
    pub fn presented(&self) {
        self.display.presented(self.clock.counter());
    }

    /// The log, for a test that asserts on what orb said.
    pub fn log(&self) -> &Log {
        &self.log
    }

    /// The files orb reads and writes of its own: what an e2e test says it finds there, and what it reads
    /// back of what orb wrote.
    pub fn files(&self) -> &Files {
        &self.files
    }

    /// Where the game is installed, which is the directory orb reads `orb.yaml` and writes the log
    /// in.
    pub fn set_host_exe(&self, path: impl Into<PathBuf>) {
        *self.host_exe.lock().unwrap() = path.into();
    }

    /// Says a module is loaded, without saying what it exports — which is the case orb tells apart
    /// from the module being absent.
    pub fn load_module(&self, module: &str) {
        let mut modules = self.modules.lock().unwrap();
        if !modules.iter().any(|loaded| loaded == module) {
            modules.push(module.to_string());
        }
    }

    /// Says a module exports a function at an address, for the winmm entry points orb looks up in
    /// the game's own copy. Loads the module too, since a module that exports something is one that
    /// is there.
    pub fn set_proc_address(&self, module: &str, name: &str, address: usize) {
        self.load_module(module);
        self.procs
            .lock()
            .unwrap()
            .insert((module.to_string(), name.to_string()), address);
    }
}

impl Win for Sim {
    fn read_bytes(&self, address: usize, len: usize) -> Vec<u8> {
        self.space.read_bytes(address, len)
    }

    fn write_bytes(&self, address: usize, source: &[u8]) {
        self.space.write_bytes(address, source);
    }

    fn fill_bytes(&self, address: usize, byte: u8, len: usize) {
        self.space.fill_bytes(address, byte, len);
    }

    fn read_committed_bytes(&self, address: usize, len: usize) -> Option<Vec<u8>> {
        self.space.read_committed_bytes(address, len)
    }

    /// Out of the laid-out space, with nothing to unprotect: what a page's protection is is the real
    /// host's business, and a space that modelled one would be refusing writes an e2e test asked for.
    ///
    /// Always answers, an address nothing is mapped at panicking the way every other read here does.
    fn replace_word(&self, address: usize, value: usize) -> Option<usize> {
        let original = self.space.read::<usize>(address);
        self.space.write::<usize>(address, value);
        Some(original)
    }

    fn commit(&self, address: usize, len: usize) -> bool {
        self.space.commit(address, len)
    }

    fn vtable_in_image(&self, address: usize) -> bool {
        self.space.vtable_in_image(address)
    }

    /// Remembered, and the walk answers without them: a laid-out game's memory *is* the game's, so
    /// there are no heaps to walk and no reservations to have been told about. What the lists are for is
    /// an e2e test asking whether orb noticed one.
    fn note_heap(&self, heap: usize) {
        let mut noticed = self.noticed.lock().unwrap();
        if !noticed.heaps.contains(&heap) {
            noticed.heaps.push(heap);
        }
    }

    fn note_reservation(&self, base: usize, len: usize) {
        let mut noticed = self.noticed.lock().unwrap();
        if !noticed
            .reservations
            .iter()
            .any(|(at, len)| *at <= base && base < at + len)
        {
            noticed.reservations.push((base, len));
        }
    }

    fn forget_reservation(&self, base: usize) {
        self.noticed
            .lock()
            .unwrap()
            .reservations
            .retain(|(at, _)| *at != base);
    }

    fn game_regions(&self, data: &Range<usize>) -> Vec<(usize, usize)> {
        self.space.game_regions(data)
    }

    fn keep_out_of_private_regions(&self, base: usize, len: usize) {
        self.ours.lock().unwrap().insert(base, len);
    }

    fn count_private_region_again(&self, base: usize) {
        self.ours.lock().unwrap().remove(&base);
    }

    /// Nothing. The walk this answers is `self_check`'s hunt for memory a snapshot did not cover, and
    /// what it would find here is the test binary's own pages — every allocation the harness made,
    /// reported as the game having changed memory behind orb's back. The check still says what it
    /// found in the regions it *did* cover, which is the half a laid-out game can speak to.
    fn private_regions(&self) -> Vec<(usize, usize)> {
        Vec::new()
    }

    fn process_heap_regions(&self) -> Vec<(usize, usize)> {
        Vec::new()
    }

    fn counter(&self) -> i64 {
        self.clock.counter()
    }

    fn frequency(&self) -> i64 {
        self.clock.frequency()
    }

    fn wait(&self, ticks: i64) -> bool {
        self.clock.wait(ticks)
    }

    fn spin_once(&self) {
        self.clock.spin_once();
    }

    fn sleep(&self, ms: u32) {
        self.clock.sleep(ms);
    }

    fn message_box(&self, title: &str, text: &str) {
        self.dialogs
            .lock()
            .unwrap()
            .push((title.to_string(), text.to_string()));
    }

    fn exit_process(&self, code: u32) {
        *self.exited.lock().unwrap() = Some(code);
    }

    fn monitor_refresh(&self, window: Hwnd) -> Option<u32> {
        self.display.monitor_refresh(window)
    }

    fn desktop_refresh(&self) -> Option<u32> {
        self.display.desktop_refresh()
    }

    fn composition(&self) -> Option<Composition> {
        self.display.composition(self.clock.counter())
    }

    fn flush(&self) -> bool {
        match self.display.flush(self.clock.counter()) {
            Some(blank) => {
                self.clock.advance_to(blank);
                true
            }
            None => false,
        }
    }

    fn foreground_window(&self) -> Hwnd {
        self.display.foreground()
    }

    fn set_process_dpi_aware(&self) -> bool {
        self.windows.set_process_dpi_aware()
    }

    fn primary_monitor(&self) -> Option<Rect> {
        self.windows.monitor_now()
    }

    fn adjust_window_rect(&self, area: Rect, style: u32, _menu: bool) -> Option<Rect> {
        Some(self.windows.adjust(area, style))
    }

    /// Out of the same declared metric a baked string is measured by — one answer per em height,
    /// whichever rasteriser asks. Two knobs for the two would be two numbers an e2e test had to keep in
    /// step for no question it asks; the bar is written at 30 pixels of em and the overlay at 15 and 19,
    /// so nothing declares one of these and moves the other by accident.
    fn measure_lines(&self, lines: &[String], em: i32) -> (i32, i32) {
        let metric = self.glyphs.metric(em);
        let widest = lines
            .iter()
            .map(|line| metric.advance * line.chars().count() as u32)
            .max()
            .unwrap_or(0);
        (widest as i32, metric.line as i32)
    }

    fn write_lines(&self, window: Hwnd, bar: orb_api::Bar, lines: &[String]) -> bool {
        self.windows.write_lines(window, bar, lines)
    }

    fn client_rect(&self, window: Hwnd) -> Option<Rect> {
        self.windows.client(window)
    }

    fn keyboard_state(&self) -> Option<[u8; 256]> {
        self.keyboard.state()
    }

    fn mouse_position(&self) -> Option<(i32, i32)> {
        self.mouse.position()
    }

    fn show_mouse(&self, showing: bool) -> i32 {
        self.mouse.show(showing)
    }

    fn joystick_position(&self, device: u32, flags: u32) -> (u32, orb_api::JoyInfo) {
        self.joystick.position(device, flags)
    }

    fn joystick_caps(&self, device: u32) -> Option<orb_api::JoyCaps> {
        self.joystick.caps(device)
    }

    /// UTF-8, lossily. A code page is a property of the machine, so a simulated one would be a table
    /// nobody could check — and the device names an e2e test declares are the names it then reads back.
    fn codepage_text(&self, bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    fn current_thread_id(&self) -> u32 {
        thread_id()
    }

    /// Remembered, and nothing is ever stopped. A simulated game runs on the thread the e2e test is
    /// on, so there is nothing else touching the memory a copy covers — and what `SuspendThread` does
    /// is checked against real threads a test makes and registers itself, in `orb-api`, rather than
    /// against a model of it here.
    fn register_thread(&self, id: u32) -> bool {
        self.threads.lock().unwrap().push(id);
        true
    }

    /// Written down and nothing scheduled differently, for the same reason: how a real host schedules a
    /// thread it has been told about is not something a model of one can answer.
    fn below_normal(&self) {
        self.below_normal.lock().unwrap().push(thread_id());
    }

    fn suspend_game_threads(&self, _audio: Option<u32>) -> Vec<u32> {
        Vec::new()
    }

    fn resume_threads(&self, _ids: &[u32]) {}

    fn open_log(&self, path: &Path, max_bytes: u64) -> Option<LogFile> {
        self.log.open(path, max_bytes)
    }

    fn write_log(&self, file: LogFile, bytes: &[u8]) {
        self.log.write(file, bytes);
    }

    fn close_log(&self, file: LogFile) {
        self.log.close(file);
    }

    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        self.files.read(path)
    }

    fn read_file_to_string(&self, path: &Path) -> std::io::Result<String> {
        self.files.read_to_string(path)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        self.files.write(path, bytes)
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        self.files.create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        self.files.remove_file(path)
    }

    fn files_in(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.files.files_in(path)
    }

    fn module_path(&self, module: Option<usize>) -> Option<PathBuf> {
        // Only the exe is asked for by anything a test drives. A module handle is a number a test
        // never has a real one of, so answering `None` for it is the truth rather than a gap: the
        // crash handler's `module+offset` line is about a real process's modules.
        match module {
            None => Some(self.host_exe.lock().unwrap().clone()),
            Some(_) => None,
        }
    }

    fn module_loaded(&self, module: &str) -> bool {
        self.modules
            .lock()
            .unwrap()
            .iter()
            .any(|loaded| loaded == module)
    }

    fn proc_address(&self, module: &str, name: &str) -> Option<usize> {
        self.procs
            .lock()
            .unwrap()
            .get(&(module.to_string(), name.to_string()))
            .copied()
    }

    fn load_face(&self, path: &Path, height: i32) -> Option<Face> {
        self.glyphs.load_face(path, height)
    }

    fn face_name(&self, face: Face) -> Option<String> {
        self.glyphs.face_name(face)
    }

    fn bake(&self, face: Face, text: &str) -> Option<Mask> {
        self.glyphs.bake(face, text)
    }

    /// Nothing. A face is left where it is so that a mask baked through it is still readable, which is
    /// the whole of what one is here for — there is no font loaded to take back out.
    fn drop_face(&self, _face: Face) {}

    // --- the device -------------------------------------------------------------
    //
    // Which device is nothing to any of these: a simulated host has the one an e2e test's game shows
    // through, and the handle is there because a real one has several and the game says which.
    // `Recording` is where the record is; these are the eighteen slots reaching it.

    fn create_texture(
        &self,
        _device: Device,
        width: u32,
        height: u32,
        _levels: u32,
        _usage: u32,
        _format: u32,
        _pool: u32,
    ) -> (Hresult, Option<Texture>) {
        (0, Some(self.drawing.create_texture(width, height)))
    }

    /// A token that is not zero, which is what the drawing gives up on a device for not answering.
    fn create_state_block(&self, _device: Device, _kind: u32) -> (Hresult, u32) {
        (0, 1)
    }

    fn capture_state_block(&self, _device: Device, _token: u32) -> Hresult {
        0
    }

    fn apply_state_block(&self, _device: Device, _token: u32) {}

    fn delete_state_block(&self, _device: Device, _token: u32) {}

    /// Nothing. Which states the drawing sets is above the seam and has its own tests there; what an
    /// e2e test reads back is the quads, and a record of every state written would be a record of the
    /// drawing's own source.
    fn set_render_state(&self, _device: Device, _state: u32, _value: u32) {}

    fn set_texture_stage_state(&self, _device: Device, _stage: u32, _kind: u32, _value: u32) {}

    fn set_texture(&self, _device: Device, _stage: u32, texture: Option<Texture>) {
        self.drawing.set_texture(texture);
    }

    fn set_vertex_shader(&self, _device: Device, _shader: u32) {}

    fn set_viewport(&self, _device: Device, viewport: Viewport) {
        self.drawing.viewport_set(viewport);
    }

    fn get_viewport(&self, _device: Device) -> Viewport {
        self.drawing.viewport()
    }

    fn draw_primitive_up(
        &self,
        _device: Device,
        _kind: u32,
        count: u32,
        vertices: &[u8],
        stride: u32,
    ) {
        self.drawing.drew(count, vertices, stride);
    }

    fn begin_scene(&self, _device: Device) {
        self.drawing.scene_began();
    }

    fn end_scene(&self, _device: Device) {}

    fn clear(&self, _device: Device, _flags: u32, color: u32, _z: f32, _stencil: u32) {
        self.drawing.cleared(color);
    }

    fn lock_rect(&self, texture: Texture, _level: u32, _flags: u32) -> Option<Locked> {
        self.drawing.lock_rect(texture)
    }

    fn unlock_rect(&self, _texture: Texture, _level: u32) {}

    /// Nothing, and the storage stays. The drawing may release a texture twice — a `Label` re-baked
    /// releases the one before — and what an e2e test asks after a frame is what went into it, so a
    /// release that freed the rows would take the answer with it.
    fn release_texture(&self, _texture: Texture) {}

    // --- the buffer the game's music is played out of ----------------------------
    //
    // The sound an e2e test installed, which is a thread's rather than this value's — see
    // `crate::sound`, and `Sound::install`, which is what tells this host where the buffer is.

    fn buffer_position(&self, buffer: SoundBuffer) -> (Hresult, u32, u32) {
        sound::buffer::position(buffer)
    }

    fn buffer_status(&self, buffer: SoundBuffer) -> (Hresult, u32) {
        sound::buffer::status(buffer)
    }

    fn lock_buffer(
        &self,
        buffer: SoundBuffer,
        offset: u32,
        bytes: u32,
        _flags: u32,
    ) -> (Hresult, LockedBuffer) {
        sound::buffer::lock(buffer, offset, bytes)
    }

    /// Nothing. The rows stay where they are for as long as the sound does, so there is nothing to give
    /// back — and what an e2e test reads afterwards is those same bytes.
    fn unlock_buffer(&self, _buffer: SoundBuffer, _locked: LockedBuffer) {}

    fn play_buffer(&self, buffer: SoundBuffer, _reserved: u32, _priority: u32, flags: u32) {
        sound::buffer::play(buffer, flags);
    }

    fn stop_buffer(&self, buffer: SoundBuffer) {
        sound::buffer::stop(buffer);
    }

    fn set_buffer_position(&self, buffer: SoundBuffer, position: u32) {
        sound::buffer::set_position(buffer, position);
    }

    /// Nothing. Buffers are never lost here: what `Restore` is for is a device that has been taken away,
    /// which no e2e test does.
    fn restore_buffer(&self, _buffer: SoundBuffer) {}
}

#[cfg(test)]
mod tests {
    use super::Sim;
    use orb_api::{Kind, Win};
    use std::sync::Arc;

    /// Inside the game's static data, and inside a test binary's own image as well — which is the
    /// point. Every address orb reads the game at is one a binary already keeps something at, so a
    /// read that is not answered by the sim is answered by whatever is there.
    const DATA: usize = 0x0069_b000;
    const CODE: usize = 0x0040_1000;

    /// What the space is for: the game's own address answers with the game's own number, in a
    /// process where no game is running.
    #[test]
    fn a_mapped_address_reads_back_what_was_put_there() {
        let sim = Sim::new();
        sim.space().map(DATA, 0x100, Kind::Private);
        sim.space().write::<u32>(DATA + 0x40, 0x1234_5678);
        assert_eq!(sim.space().read::<u32>(DATA + 0x40), 0x1234_5678);
        // The bytes either side are the zeroes a freshly committed page reads as, and not this
        // binary's.
        assert_eq!(sim.space().read::<u32>(DATA + 0x3c), 0);
        assert_eq!(sim.space().read::<u32>(DATA + 0x44), 0);
    }

    /// Reserved and never committed is where every pointer into a structure the game has not
    /// built yet lands, and the answer is that there is nothing to read — which is the answer
    /// orb's pointer chases are written against.
    #[test]
    fn an_uncommitted_region_is_not_read() {
        let sim = Sim::new();
        sim.space().reserve(DATA, 0x100);
        assert_eq!(sim.space().read_committed::<u32>(DATA), None);
    }

    /// The page every thread's stack ends with. Refused separately from what is uncommitted
    /// because reading one in a real process does not merely fail: the guard comes off and the
    /// thread owning that stack stops growing it.
    #[test]
    fn a_guard_page_is_not_read() {
        let sim = Sim::new();
        sim.space().guard(DATA, 0x1000);
        assert_eq!(sim.space().read_committed::<u32>(DATA), None);
    }

    /// The alignment rule holds in the sim as well, so that a test cannot pass on a read the real
    /// one would have refused. Every one of these addresses is a field of a structure the game
    /// built, so an unaligned one is not one of them.
    #[test]
    fn an_unaligned_address_is_not_read() {
        let sim = Sim::new();
        sim.space().map(DATA, 0x100, Kind::Private);
        assert_eq!(sim.space().read_committed::<u32>(DATA + 4), Some(0));
        assert_eq!(sim.space().read_committed::<u32>(DATA + 5), None);
    }

    /// A read that would have taken the process down is a test with a wrong address in it, so it
    /// says so rather than answering. The alternative — falling through to the real address space
    /// — is the failure this whole crate exists to remove: it would read the test binary's image
    /// and call it the game's.
    #[test]
    #[should_panic(expected = "is not mapped in this space")]
    fn reading_where_nothing_is_mapped_says_so() {
        let sim = Sim::new();
        sim.space().map(DATA, 0x100, Kind::Private);
        sim.space().read::<u32>(DATA + 0x200);
    }

    /// How orb tells a live COM object from the stale pointer left in a block the game's
    /// allocator did not scrub: the live one's vtable is in a mapped image.
    #[test]
    fn a_vtable_is_live_only_where_it_points_into_an_image() {
        let sim = Sim::new();
        sim.space().map(CODE, 0x100, Kind::Image);
        sim.space().map(DATA, 0x100, Kind::Private);

        sim.space().write::<usize>(DATA, CODE + 0x10);
        assert!(sim.space().vtable_in_image(DATA));

        // Into the game's own data rather than its code: allocated, readable, and not a vtable.
        sim.space().write::<usize>(DATA, DATA + 0x80);
        assert!(!sim.space().vtable_in_image(DATA));

        // And nowhere at all, which is the freed object whose block still holds its old bytes.
        sim.space().write::<usize>(DATA, 0x0dead000);
        assert!(!sim.space().vtable_in_image(DATA));
    }

    /// What a snapshot is handed, and what it is handed after a free: a region that has gone away
    /// between the snapshot and the restore of it is the case a restore has to survive, so the
    /// sim has to be able to stop having one.
    #[test]
    fn what_is_mapped_is_what_has_not_been_unmapped() {
        let sim = Sim::new();
        sim.space().map(DATA, 0x100, Kind::Private);
        sim.space().map(CODE, 0x40, Kind::Image);
        // Reserved without being committed is not memory anything can be read out of, so it is not
        // memory a snapshot has anything to save.
        sim.space().reserve(DATA + 0x1000, 0x100);

        let mut mapped = sim.space().mapped();
        mapped.sort_unstable();
        assert_eq!(mapped, vec![(CODE, 0x40), (DATA, 0x100)]);

        sim.space().unmap(DATA);
        assert_eq!(sim.space().mapped(), vec![(CODE, 0x40)]);
    }

    /// The harness hands its threads out again, so an installation left behind would be the next
    /// test on that thread reading a game it did not lay out.
    #[test]
    fn a_sim_comes_off_the_thread_when_its_installation_goes() {
        assert!(orb_api::installed().is_none());
        let sim: Arc<dyn Win> = Arc::new(Sim::new());
        {
            let _installed = orb_api::install(&sim);
            assert!(orb_api::installed().is_some());
        }
        assert!(orb_api::installed().is_none());
    }

    /// The whole point of the seam: `orb_api::mem`'s own functions — the ones every structure walk
    /// in orb goes through — answer out of the sim, on a host that has no Windows to fall back to.
    #[test]
    fn the_memory_seam_reads_the_installed_sim() {
        let sim = Sim::new();
        sim.space().map(DATA, 0x100, Kind::Private);
        let sim: Arc<dyn Win> = Arc::new(sim);
        let _installed = orb_api::install(&sim);

        unsafe { orb_api::mem::write::<u32>(DATA + 8, 0xdead_beef) };
        assert_eq!(unsafe { orb_api::mem::read::<u32>(DATA + 8) }, 0xdead_beef);
        assert_eq!(
            orb_api::mem::read_committed::<u32>(DATA + 8),
            Some(0xdead_beef)
        );
        // Off the end of what was laid out, which is where every pointer into a structure the game
        // has not built yet lands.
        assert_eq!(orb_api::mem::read_committed::<u32>(DATA + 0x100), None);
    }
}
