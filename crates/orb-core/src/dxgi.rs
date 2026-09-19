//! **The swapchain a game draws through, taken as it is made, so that the present is orb's to call.**
//!
//! **Nothing of any game's is read here.** What `CreateDXGIFactory2` answers, what a factory's own
//! `CreateSwapChainForHwnd` answers and where `Present` sits in a swapchain's vtable are Windows' own
//! shapes — so this is the same wiring for any game that draws through DXGI, and the game module's part
//! of it is one name to answer and one call per frame. See `game::th06c`, which is the one that asks.
//!
//! **Why it is taken at all**: orb's own frame loop calls the game's draw chain and then presents, and
//! the present a game like this has is inside one of that chain's own jobs — so a loop that walked the
//! chain and then presented would present twice. The slot is written over with [`presents_nothing`] and
//! the original kept here: the job's call does nothing and [`present`] is the one that reaches the
//! swapchain.
//!
//! **And a game that asks for no factory keeps its own present.** Every function here does nothing at
//! all until something hands it a factory or a swapchain, and [`present`] then presents nothing — which
//! is a frame handed over by whatever the game does inside its own draw, exactly as it was before.

use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::mem;

use crate::log;

/// `IID_IDXGISwapChain`, `310d36a0-d2e7-4c0a-aa04-6a9d23b8886a`, as the sixteen bytes a `GUID` is: its
/// first three fields little-endian and the last eight in the order they are written.
const IID_IDXGISWAPCHAIN: [u8; 16] = [
    0xa0, 0x36, 0x0d, 0x31, 0xe7, 0xd2, 0x0a, 0x4c, 0xaa, 0x04, 0x6a, 0x9d, 0x23, 0xb8, 0x88, 0x6a,
];

/// The slots of the two vtables this reaches through, counted in pointers.
///
/// `IUnknown`'s three come first in every one of them, which is what makes [`RELEASE`] the same number
/// for a factory and for a swapchain. The rest are each interface's own, in the order they are declared:
/// `IDXGIObject`'s four and `IDXGIDeviceSubObject`'s one before a swapchain's [`PRESENT`], and
/// `IDXGIObject`'s four, `IDXGIFactory`'s five and `IDXGIFactory1`'s two before
/// [`CREATE_SWAP_CHAIN_FOR_HWND`].
const QUERY_INTERFACE: usize = 0;
const RELEASE: usize = 2;
const PRESENT: usize = 8;
/// `IDXGISwapChain::ResizeBuffers`, after `GetBuffer`, `SetFullscreenState`, `GetFullscreenState` and
/// `GetDesc` — see [`resizes_its_buffers`].
const RESIZE_BUFFERS: usize = 13;
const CREATE_SWAP_CHAIN_FOR_HWND: usize = 15;
/// **And `IDXGISwapChain1::Present1`, which is the one a game built this decade uses.** It comes after
/// that interface's own four — `GetDesc1`, `GetFullscreenDesc`, `GetHwnd`, `GetCoreWindow` — and a
/// swapchain made by `CreateSwapChainForHwnd` has it whether the game asked for the newer interface or
/// not, the object being the same one either way.
///
/// **Both are taken and the one the game used is the one orb calls.** Measured: with only `Present`
/// taken, the window stood at the frame it held when orb took the slot while the game walked three
/// screens of its front end, and the stand-in in that slot was never called once — so the game presents
/// through this one, and `Present` on its own suppressed nothing and presented nothing.
const PRESENT1: usize = 22;

/// What the host's `CreateDXGIFactory2` is, once the game has asked for it — see [`answers_the_factory`].
static THE_HOSTS_FACTORY: AtomicUsize = AtomicUsize::new(0);
/// And what a factory's own `CreateSwapChainForHwnd` was before orb wrote over that slot.
static ITS_CREATE_SWAP_CHAIN: AtomicUsize = AtomicUsize::new(0);
/// The swapchain itself, and its own two presents as they were before orb wrote over those slots.
static SWAPCHAIN: AtomicUsize = AtomicUsize::new(0);
static ITS_PRESENT: AtomicUsize = AtomicUsize::new(0);
static ITS_PRESENT1: AtomicUsize = AtomicUsize::new(0);
/// **Which of the two the game's own call went through**, which is the one orb's own call goes through:
/// `Present1` takes a structure of its own that a call made up here would have to invent, so what orb
/// hands over is the pointer the game's own call handed over — nought where it was `Present`, and a
/// `DXGI_PRESENT_PARAMETERS` of the game's where it was `Present1`. See [`PRESENT1`].
static ITS_PARAMETERS: AtomicUsize = AtomicUsize::new(0);
static PRESENTS_THROUGH_PRESENT1: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Says what the host answered the game's ask for `CreateDXGIFactory2` with, which is what
/// [`creates_a_factory`] calls through to.
///
/// # Safety
/// Must run before the game has called the answer.
pub unsafe fn answers_the_factory(host: usize) {
    THE_HOSTS_FACTORY.store(host, Ordering::Relaxed);
}

/// **What the game gets when it asks for `CreateDXGIFactory2`**: the host's own factory, with its
/// `CreateSwapChainForHwnd` written over so that the swapchain it makes is a swapchain orb has.
///
/// **Only that one name of the three.** A DXGI renderer asks for `CreateDXGIFactory2` first and falls
/// back to `CreateDXGIFactory1` and `CreateDXGIFactory` where it is not there — which is Windows 7 and
/// older — so answering the newest is answering every launch this runs on, and the two older names are
/// left to the host. What a launch that fell back would lose is the present, and nothing else: see the
/// module's own note on a game that asks for no factory.
///
/// **And the slot is written once.** A game may ask for a factory more than once, and every factory of
/// the same interface shares one vtable inside `dxgi.dll` — so a second write would put orb's own
/// pointer in as the original and orb would call itself.
///
/// # Safety
/// The arguments are `CreateDXGIFactory2`'s own.
pub unsafe extern "system" fn creates_a_factory(
    flags: u32,
    iid: *const u8,
    out: *mut usize,
) -> i32 {
    let host = THE_HOSTS_FACTORY.load(Ordering::Relaxed);
    if host == 0 {
        return E_NOINTERFACE;
    }
    let host: unsafe extern "system" fn(u32, *const u8, *mut usize) -> i32 =
        unsafe { std::mem::transmute(host) };
    let answered = unsafe { host(flags, iid, out) };
    if answered < 0 {
        return answered;
    }
    let factory = unsafe { out.read() };
    if factory != 0 && ITS_CREATE_SWAP_CHAIN.load(Ordering::Relaxed) == 0 {
        unsafe { takes_the_swapchains_of(factory) };
    }
    answered
}

/// Writes over that factory's `CreateSwapChainForHwnd`, keeping what was there.
///
/// # Safety
/// `factory` must be a live `IDXGIFactory2`, on the thread that made it.
unsafe fn takes_the_swapchains_of(factory: usize) {
    let Some(vtable) = mem::read_committed::<usize>(factory).filter(|vtable| *vtable != 0) else {
        return;
    };
    let slot = vtable + CREATE_SWAP_CHAIN_FOR_HWND * size_of::<usize>();
    let replacement = creates_a_swapchain as *const () as usize;
    match unsafe { mem::replace_word(slot, replacement) } {
        Some(original) => {
            ITS_CREATE_SWAP_CHAIN.store(original, Ordering::Relaxed);
            log!("frame: the swapchain this game makes will be orb's to present through");
        }
        None => {
            log!("frame: cannot reach the factory's own swapchains: the slot cannot be written")
        }
    }
}

/// **What the game's own call to `CreateSwapChainForHwnd` goes through**: the factory's own, and then the
/// swapchain it answered taken.
///
/// # Safety
/// The arguments are `IDXGIFactory2::CreateSwapChainForHwnd`'s own.
unsafe extern "system" fn creates_a_swapchain(
    factory: usize,
    device: usize,
    window: usize,
    description: *const u8,
    fullscreen: *const u8,
    restrict: usize,
    out: *mut usize,
) -> i32 {
    let original = ITS_CREATE_SWAP_CHAIN.load(Ordering::Relaxed);
    if original == 0 {
        return E_NOINTERFACE;
    }
    let original: unsafe extern "system" fn(
        usize,
        usize,
        usize,
        *const u8,
        *const u8,
        usize,
        *mut usize,
    ) -> i32 = unsafe { std::mem::transmute(original) };
    if let Some(asked) = unsafe { the_description_asked(description) } {
        THE_SIZE_ASKED.store(
            (u64::from(asked.width) << 32) | u64::from(asked.height),
            Ordering::Relaxed,
        );
        log!(
            "frame: the game asks for a swapchain of {}x{} with {} buffer(s), scaling {} and swap \
             effect {}",
            asked.width,
            asked.height,
            asked.buffers,
            asked.scaling,
            asked.swap_effect,
        );
    }
    let answered = unsafe {
        original(
            factory,
            device,
            window,
            description,
            fullscreen,
            restrict,
            out,
        )
    };
    if answered < 0 {
        return answered;
    }
    let made = unsafe { out.read() };
    if made != 0 {
        unsafe { take(made) };
    }
    answered
}

/// **The five numbers of a `DXGI_SWAP_CHAIN_DESC1` a launch says**, of the eleven the structure has.
struct Asked {
    width: u32,
    height: u32,
    buffers: u32,
    scaling: u32,
    swap_effect: u32,
}

/// **Where `DXGI_SWAP_CHAIN_DESC1` keeps each of those**, in bytes from its start: its width, height,
/// format and stereo flag come to 16, `DXGI_SAMPLE_DESC`'s two to 24, and the buffers' usage, their count,
/// the scaling and the swap effect follow a dword apiece. Forty-eight bytes the whole of it.
const ITS_BUFFER_COUNT: usize = 28;
const ITS_SCALING: usize = 32;
const ITS_SWAP_EFFECT: usize = 36;
const A_DESCRIPTION1: usize = 48;

/// **What a `CreateSwapChainForHwnd` asked for**, read off the description it was handed, and `None` for a
/// call handed none.
///
/// # Safety
/// `description` must be null or a `DXGI_SWAP_CHAIN_DESC1`.
unsafe fn the_description_asked(description: *const u8) -> Option<Asked> {
    if description.is_null() {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(description, A_DESCRIPTION1) };
    let dword = |at: usize| {
        let mut word = [0u8; size_of::<u32>()];
        word.copy_from_slice(&bytes[at..at + size_of::<u32>()]);
        u32::from_le_bytes(word)
    };
    Some(Asked {
        width: dword(0),
        height: dword(size_of::<u32>()),
        buffers: dword(ITS_BUFFER_COUNT),
        scaling: dword(ITS_SCALING),
        swap_effect: dword(ITS_SWAP_EFFECT),
    })
}

/// **The swapchain kept and its `Present` taken over**, whichever way it was found.
///
/// Answers whether the present is orb's now. Once: a launch that made a second swapchain would otherwise
/// put orb's own stand-in in as the original, and every frame after it would present nothing.
///
/// # Safety
/// `swapchain` must be a live `IDXGISwapChain`, on the game's own thread.
pub unsafe fn take(swapchain: usize) -> bool {
    if ITS_PRESENT.load(Ordering::Relaxed) != 0 {
        return true;
    }
    let Some(vtable) = mem::read_committed::<usize>(swapchain).filter(|vtable| *vtable != 0) else {
        return false;
    };
    let present = unsafe {
        mem::replace_word(
            vtable + PRESENT * size_of::<usize>(),
            presents_nothing as *const () as usize,
        )
    };
    let present1 = unsafe {
        mem::replace_word(
            vtable + PRESENT1 * size_of::<usize>(),
            presents_nothing1 as *const () as usize,
        )
    };
    let resize = unsafe {
        mem::replace_word(
            vtable + RESIZE_BUFFERS * size_of::<usize>(),
            resizes_its_buffers as *const () as usize,
        )
    };
    if let Some(resize) = resize {
        ITS_RESIZE_BUFFERS.store(resize, Ordering::Relaxed);
    }
    match (present, present1) {
        (Some(present), Some(present1)) => {
            SWAPCHAIN.store(swapchain, Ordering::Relaxed);
            ITS_PRESENT.store(present, Ordering::Relaxed);
            ITS_PRESENT1.store(present1, Ordering::Relaxed);
            log!("frame: both of the presents are orb's to call now, the game's own doing nothing");
            true
        }
        _ => {
            log!("frame: cannot take the presents: a slot's page cannot be made writable");
            false
        }
    }
}

/// The swapchain's own `ResizeBuffers`, as it was before orb wrote over that slot.
static ITS_RESIZE_BUFFERS: AtomicUsize = AtomicUsize::new(0);

/// **The size the game asked its swapchain for**, width above height, and nought where it asked none —
/// see [`resizes_its_buffers`].
static THE_SIZE_ASKED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// **What sits in the swapchain's `ResizeBuffers` once orb has it**: nothing at all where the window is one
/// orb plays in, its game draws at the size it asked for, and the buffers are that size already — and the
/// game's own call otherwise.
///
/// **Because the game makes its buffers the window's size again when the window is resized**, and orb holds
/// the window it plays in at a small client — see `crate::window::keeps_its_corner`. For a game that draws
/// at the size it asked for — see `Game::draws_at_the_size_it_asked_for` — buffers made that small are a
/// picture cut to the window's: a launch held at a `320x240` client showed the top-left of th06c's title
/// and nothing else. Kept at the size asked, the swapchain stretches the whole picture onto the window it
/// presents into — it was asked for with `scaling 0`, `DXGI_SCALING_STRETCH`.
///
/// **And th06nc is the other kind**, which lays its frame out over the size its buffers are: left at
/// `1280x720` under a `320x240` window its title was a quarter of the window's width in its top-left
/// corner, the game asking for `320x240` again on every frame. Its window is made small by its own mode.
///
/// # Safety
/// The arguments are `IDXGISwapChain::ResizeBuffers`'s own.
unsafe extern "system" fn resizes_its_buffers(
    swapchain: usize,
    buffers: u32,
    width: u32,
    height: u32,
    format: u32,
    flags: u32,
) -> i32 {
    let original = ITS_RESIZE_BUFFERS.load(Ordering::Relaxed);
    if original == 0 {
        return S_OK;
    }
    let original: unsafe extern "system" fn(usize, u32, u32, u32, u32, u32) -> i32 =
        unsafe { std::mem::transmute(original) };
    let asked = THE_SIZE_ASKED.load(Ordering::Relaxed);
    let held = (asked >> 32) as u32 != 0
        && unsafe { crate::window::plays_out_of_the_way() }
        && unsafe { crate::runtime::the_game_draws_at_the_size_it_asked_for() };
    // **And not made again at all where they are that size already**, which is every such call: made again
    // at the same size, the call is one the swapchain refuses while anything holds a view of its buffers,
    // and a launch whose buffers were made the asked size again closed on it. The game has let go of its
    // own views to ask, and takes the same buffers back.
    let (into_width, into_height) = ((asked >> 32) as u32, asked as u32);
    if held && unsafe { the_size_of(swapchain) } == Some((into_width, into_height)) {
        crate::detail!(
            "frame: the game makes its swapchain's buffers {width}x{height} again, which are left \
             {into_width}x{into_height}"
        );
        return S_OK;
    }
    unsafe { original(swapchain, buffers, width, height, format, flags) }
}

/// **Whether an object is a swapchain**, asked of the object itself.
///
/// For a game whose swapchain orb goes looking for among the pointers it keeps rather than taking as it
/// is made: `QueryInterface` says what an object is where no reading of the exe does. The reference the
/// ask returns is given straight back — what orb keeps is the pointer the game keeps, and a second
/// reference would hold the swapchain alive past the game's own release of it.
///
/// # Safety
/// `object` must be a live COM object whose vtable is a mapped image's — see `mem::vtable_in_image`,
/// which is what tells one from a field that merely looks like a pointer.
pub unsafe fn is_a_swapchain(object: usize) -> bool {
    let Some(vtable) = mem::read_committed::<usize>(object).filter(|vtable| *vtable != 0) else {
        return false;
    };
    let query = mem::read_committed::<usize>(vtable + QUERY_INTERFACE * size_of::<usize>());
    let Some(query) = query.filter(|query| *query != 0) else {
        return false;
    };
    let query: unsafe extern "system" fn(usize, *const u8, *mut usize) -> i32 =
        unsafe { std::mem::transmute(query) };
    let mut answered = 0usize;
    if unsafe { query(object, IID_IDXGISWAPCHAIN.as_ptr(), &mut answered) } < 0 || answered == 0 {
        return false;
    }
    let release = mem::read_committed::<usize>(vtable + RELEASE * size_of::<usize>());
    if let Some(release) = release.filter(|release| *release != 0) {
        let release: unsafe extern "system" fn(usize) -> u32 =
            unsafe { std::mem::transmute(release) };
        unsafe { release(answered) };
    }
    true
}

/// What sits in the swapchain's own slot once orb has it: nothing, and the answer a present that worked
/// gives.
///
/// **And the game's own two numbers taken as they go past, once.** What interval a game asks its present
/// for is the game asking the display to pace it, and orb's own present is made with the interval this
/// saw rather than a number written down here: a frame handed over with an interval the game never used
/// is a frame the display may do something else with — see [`THE_GAMES_INTERVAL`].
///
/// # Safety
/// The arguments are `IDXGISwapChain::Present`'s own.
unsafe extern "system" fn presents_nothing(swapchain: usize, interval: u32, flags: u32) -> i32 {
    if !THE_INTERVAL_IS_KNOWN.swap(true, Ordering::Relaxed) {
        THE_GAMES_INTERVAL.store(interval as usize, Ordering::Relaxed);
        THE_GAMES_FLAGS.store(flags as usize, Ordering::Relaxed);
        log!("frame: the game's own Present asks for interval {interval} and flags {flags:#x}");
    }
    // **And the present made here rather than at the end of orb's frame.** This is the place the game's
    // renderer is in the state a present wants: the job that reaches this has just drawn, and what it
    // does either side of the call is that renderer's own bookkeeping about which buffer comes next.
    // Measured: a present made after the draw chain instead answered `S_OK` and put nothing on the
    // screen — the window stood at the title while the log walked three more of the front end's screens
    // — so the call is made where the game makes it and orb's loop is what reaches it, that chain being
    // orb's own walk. See `present`, which then has nothing left to do.
    PRESENTED_THIS_FRAME.store(true, Ordering::Relaxed);
    // **And what orb draws over the frame, here and nowhere earlier.** The back buffer holds the finished
    // picture exactly once a frame, and this is that moment: the draw chain is walked, the renderer has
    // composited, and the surface has not yet been handed over. See
    // `crate::runtime::the_overlay_over_a_swapchain`, and what drawing it earlier cost — th06c's
    // `Game::end_drawing` is the composite *and* the hand-over, so a quad drawn before it was a quad the
    // composite wrote over. The log said the mode question was up and the screen showed the title with
    // nothing on it.
    unsafe { crate::runtime::the_overlay_over_a_swapchain() };
    unsafe { the_original_present(swapchain, the_interval(interval), flags) }
}

/// Whether the game's own call has already presented this frame — cleared by [`starts_a_frame`].
static PRESENTED_THIS_FRAME: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Says the frame is beginning, so that [`present`] can tell a frame the game presented from one it did
/// not.
///
/// # Safety
/// Must run once per frame of the game's own, on its own thread.
pub unsafe fn starts_a_frame() {
    PRESENTED_THIS_FRAME.store(false, Ordering::Relaxed);
}

/// The swapchain's own `Present`, as it was before orb wrote over that slot.
///
/// # Safety
/// `swapchain` must be the swapchain that slot belonged to.
unsafe fn the_original_present(swapchain: usize, interval: u32, flags: u32) -> i32 {
    let present = ITS_PRESENT.load(Ordering::Relaxed);
    if present == 0 {
        return S_OK;
    }
    let present: unsafe extern "system" fn(usize, u32, u32) -> i32 =
        unsafe { std::mem::transmute(present) };
    unsafe { present(swapchain, interval, flags) }
}

/// And what sits in the other slot, which is the one this game uses — see [`PRESENT1`]. The parameters
/// are kept as they come: orb's own call hands over the same pointer rather than a structure of its own.
///
/// # Safety
/// The arguments are `IDXGISwapChain1::Present1`'s own.
unsafe extern "system" fn presents_nothing1(
    swapchain: usize,
    interval: u32,
    flags: u32,
    parameters: usize,
) -> i32 {
    if !THE_INTERVAL_IS_KNOWN.swap(true, Ordering::Relaxed) {
        THE_GAMES_INTERVAL.store(interval as usize, Ordering::Relaxed);
        THE_GAMES_FLAGS.store(flags as usize, Ordering::Relaxed);
        ITS_PARAMETERS.store(parameters, Ordering::Relaxed);
        PRESENTS_THROUGH_PRESENT1.store(true, Ordering::Relaxed);
        log!("frame: the game's own Present1 asks for interval {interval} and flags {flags:#x}");
    }
    PRESENTED_THIS_FRAME.store(true, Ordering::Relaxed);
    let present1 = ITS_PRESENT1.load(Ordering::Relaxed);
    if present1 == 0 {
        return S_OK;
    }
    let present1: unsafe extern "system" fn(usize, u32, u32, usize) -> i32 =
        unsafe { std::mem::transmute(present1) };
    unsafe { present1(swapchain, the_interval(interval), flags, parameters) }
}

/// **What the game's own present asked for**, which is what orb's asks for too — see
/// [`presents_nothing`], which is where they are read off the call orb took over.
///
/// The interval starts at one because that is what a game that paces itself by the display asks for, and
/// it is the safe way round: a frame handed over with a wait is a frame shown, where one handed over
/// without may be a frame the compositor drops.
static THE_GAMES_INTERVAL: AtomicUsize = AtomicUsize::new(1);
static THE_GAMES_FLAGS: AtomicUsize = AtomicUsize::new(0);
static THE_INTERVAL_IS_KNOWN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// **Whether a frame is handed over without waiting for the display**, which is the whole of what a run
/// paced faster than one needs here: the interval a game asks its present for *is* the display pacing
/// it, so a launch asking for frames faster than the display and a present that waits for a blank are
/// two answers to one question, and the blank wins.
///
/// **Measured**: `--pace=4` and `--pace=8` both held 8.3ms a frame on a 120Hz display — one refresh —
/// and what moved between them was where the waiting was, `Pacing`'s own hold at four and the present
/// itself at eight (`wait 3869us` and `draw 3512us` against `wait 199us` and `draw 7421us`). th06 and
/// th07 reach several times sixty at the same ask because a D3D8 present has no such wait to give.
///
/// **`0` and not the game's own where this is set**, that being DXGI's own word for do not wait. Off by
/// default and set only from [`crate::frame::Pacing::paces_faster`], so an ordinary launch hands its
/// frames over exactly as the game asked.
static HANDS_OVER_WITHOUT_WAITING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Says whether a frame is to be handed over without waiting — see [`HANDS_OVER_WITHOUT_WAITING`].
/// **The swapchain this launch presents through**, and nought for a game that makes none — see
/// [`take`], which is what puts one here.
///
/// What asks is the drawing: a game whose frames go through one of these is a game orb draws over with
/// `crate::canvas` rather than with the D3D8 overlay, and the canvas is made out of this.
pub fn the_swapchain() -> usize {
    SWAPCHAIN.load(Ordering::Relaxed)
}

/// `IDXGISwapChain::GetDesc`, after [`PRESENT`] and `IDXGISwapChain`'s own `GetBuffer`,
/// `SetFullscreenState` and `GetFullscreenState`.
const GET_DESC: usize = 12;

/// **Where `DXGI_SWAP_CHAIN_DESC` keeps the window the swapchain presents into**, in bytes from its start:
/// `DXGI_MODE_DESC`'s two sizes, its refresh rate's two, its format, its scanline order and its scaling
/// come to 28; `DXGI_SAMPLE_DESC`'s two make 36; the buffer's usage and count make 44; and the `HWND`
/// after them is eight-aligned, so it starts here.
const OUTPUT_WINDOW: usize = 48;

/// How many bytes the whole of that description is: the window, the windowed flag, the swap effect and the
/// flags after it, with the struct's own eight-byte alignment at the end.
const A_DESCRIPTION: usize = 72;

/// **The window this launch's swapchain presents into**, asked of the swapchain itself.
///
/// **Which is the window a game that makes one plays in**, and the reading a game with no field of its own
/// for the handle has: th06c looks its window creation up and where it keeps what that answered is not in
/// its `asm.md`, so `Th06c::window` answers out of here instead. What the handle is for is orb's own menus
/// — `runtime::on_update` polls the keyboard against it, and `input::Keyboard::poll` reads nothing at all
/// for a null one, so a question of orb's over such a game went up and could never be answered.
///
/// Asked every time rather than kept: a swapchain outlives the window it was made for only by being
/// released, and what a stale handle would cost is orb's menus reading a keyboard the player is not at.
///
/// `NULL` where no swapchain was taken, where the slot is empty, or where the ask fails.
///
/// # Safety
/// Must run on the game's own thread, with the swapchain live.
pub unsafe fn the_window() -> orb_api::Hwnd {
    let Some(description) = (unsafe { its_own_description(SWAPCHAIN.load(Ordering::Relaxed)) })
    else {
        return orb_api::Hwnd::NULL;
    };
    let mut handle = [0u8; size_of::<usize>()];
    handle.copy_from_slice(&description[OUTPUT_WINDOW..OUTPUT_WINDOW + size_of::<usize>()]);
    orb_api::Hwnd(usize::from_le_bytes(handle))
}

/// **How big a swapchain's back buffer is**, asked of the swapchain itself: the first two dwords of
/// `DXGI_SWAP_CHAIN_DESC` are `DXGI_MODE_DESC`'s width and height — see [`OUTPUT_WINDOW`], whose offset
/// is counted from the same two.
///
/// **What it is for is the size orb's own drawing is stretched onto.** Everything orb draws is placed in
/// `overlay`'s own 640x480 and the canvas maps those coordinates onto the viewport it sets; a viewport of
/// 640x480 pixels puts the whole overlay in the top-left corner of a larger window, which is what a
/// launch at `1280x720` showed. So the canvas asks this every frame and sets the viewport to it.
///
/// **Asked every frame rather than kept**, for the reason [`the_window`] is: a window that is resized
/// gives the swapchain new buffers, and what a stale size would cost is the overlay drawn at the size the
/// window used to be.
///
/// `None` where no swapchain was taken, where the slot is empty, or where the ask fails.
///
/// # Safety
/// Must run on the game's own thread, with the swapchain live.
pub unsafe fn the_size_of(swapchain: usize) -> Option<(u32, u32)> {
    let description = unsafe { its_own_description(swapchain) }?;
    let dword = |at: usize| {
        let mut bytes = [0u8; size_of::<u32>()];
        bytes.copy_from_slice(&description[at..at + size_of::<u32>()]);
        u32::from_le_bytes(bytes)
    };
    match (dword(0), dword(size_of::<u32>())) {
        (0, _) | (_, 0) => None,
        size => Some(size),
    }
}

/// **What the swapchain says of itself**, through its own `GetDesc`. `None` where no swapchain was taken,
/// where the slot is empty, or where the ask fails.
///
/// # Safety
/// Must run on the game's own thread, with `swapchain` live or nought.
unsafe fn its_own_description(swapchain: usize) -> Option<[u8; A_DESCRIPTION]> {
    if swapchain == 0 {
        return None;
    }
    let vtable = mem::read_committed::<usize>(swapchain).filter(|vtable| *vtable != 0)?;
    let describes = mem::read_committed::<usize>(vtable + GET_DESC * size_of::<usize>())
        .filter(|describes| *describes != 0)?;
    let describes: unsafe extern "system" fn(usize, *mut u8) -> i32 =
        unsafe { std::mem::transmute(describes) };
    let mut description = [0u8; A_DESCRIPTION];
    if unsafe { describes(swapchain, description.as_mut_ptr()) } < 0 {
        return None;
    }
    Some(description)
}

pub fn hands_over_without_waiting(without: bool) {
    HANDS_OVER_WITHOUT_WAITING.store(without, Ordering::Relaxed);
}

/// The interval a present is made with: nought where the frames are paced faster than the display, and
/// what the caller asked for otherwise.
fn the_interval(asked: u32) -> u32 {
    match HANDS_OVER_WITHOUT_WAITING.load(Ordering::Relaxed) {
        true => 0,
        false => asked,
    }
}

/// **The present orb's own frame loop makes**, which is the game's own through the pointer taken above
/// and with the two numbers the game's own call asks for — see [`THE_GAMES_INTERVAL`].
///
/// Nothing at all where no swapchain was found, which is a frame handed over by the game's own draw.
///
/// # Safety
/// Must run on the game's main thread, with everything of the frame drawn.
pub unsafe fn present() {
    let swapchain = SWAPCHAIN.load(Ordering::Relaxed);
    // **Nothing where this frame has already been presented**, which is every frame of a game that
    // presents from inside its own draw: the call was made from orb's own loop all the same, that draw
    // being orb's walk of the chain and the call itself orb's stand-in — see [`presents_nothing`], which
    // has why it is made there and not here. What is left for this is a game whose frame presents
    // nowhere of its own.
    if swapchain == 0 || PRESENTED_THIS_FRAME.load(Ordering::Relaxed) {
        return;
    }
    let interval = the_interval(THE_GAMES_INTERVAL.load(Ordering::Relaxed) as u32);
    let flags = THE_GAMES_FLAGS.load(Ordering::Relaxed) as u32;
    // **Whichever of the two the game's own call went through**, that being the one its swapchain is
    // made for: a `Present` on a swapchain a game only ever presented with `Present1` is a call DXGI may
    // answer and show nothing for, which is a window standing at the frame it held — see [`PRESENT1`].
    let answered = match PRESENTS_THROUGH_PRESENT1.load(Ordering::Relaxed) {
        true => {
            let present1 = ITS_PRESENT1.load(Ordering::Relaxed);
            if present1 == 0 {
                return;
            }
            let present1: unsafe extern "system" fn(usize, u32, u32, usize) -> i32 =
                unsafe { std::mem::transmute(present1) };
            unsafe {
                present1(
                    swapchain,
                    interval,
                    flags,
                    ITS_PARAMETERS.load(Ordering::Relaxed),
                )
            }
        }
        false => {
            let present = ITS_PRESENT.load(Ordering::Relaxed);
            if present == 0 {
                return;
            }
            let present: unsafe extern "system" fn(usize, u32, u32) -> i32 =
                unsafe { std::mem::transmute(present) };
            unsafe { present(swapchain, interval, flags) }
        }
    };
    // **What it answered, said once for each answer it gives** — and the picture handed back where it
    // refused. A present orb makes that DXGI will not carry out is a window standing at whatever frame
    // it last showed while the run goes on behind it, which is a launch nobody can watch: better the
    // game's own call than a frame that never arrives. See [`hands_it_back`].
    if !THE_ANSWER_IS_KNOWN.swap(true, Ordering::Relaxed) {
        log!("frame: orb's own present answers {answered:#x}");
    }
    if answered < 0 {
        unsafe { hands_it_back() };
    }
}

/// So the answer above is said once rather than once a frame.
static THE_ANSWER_IS_KNOWN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// **The presents put back as they were**, for a present of orb's that the swapchain refused: the game's
/// own call is then the one that reaches the display again, and orb's own does nothing.
///
/// # Safety
/// Must run on the thread the swapchain was made on.
unsafe fn hands_it_back() {
    let swapchain = SWAPCHAIN.swap(0, Ordering::Relaxed);
    let Some(vtable) = mem::read_committed::<usize>(swapchain).filter(|vtable| *vtable != 0) else {
        return;
    };
    for (slot, original) in [
        (PRESENT, ITS_PRESENT.swap(0, Ordering::Relaxed)),
        (PRESENT1, ITS_PRESENT1.swap(0, Ordering::Relaxed)),
    ] {
        if original != 0 {
            unsafe { mem::replace_word(vtable + slot * size_of::<usize>(), original) };
        }
    }
    log!("frame: the present is the game's own again, orb's having been refused");
}

/// The two `HRESULT`s this answers with: one for a present that worked, and one for an ask orb cannot
/// answer at all because nothing handed it the host's own function.
const S_OK: i32 = 0;
const E_NOINTERFACE: i32 = -0x7FFF_BFFE;
