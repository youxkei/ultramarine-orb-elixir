//! th06c — the Steam release of 東方紅魔郷 — as a [`Game`].
//!
//! **A game of its own and not a build of th06.** It is a 64-bit process built two decades later, whose
//! every address, struct and calling convention is its own, and `th06c/asm.md` is where what has
//! been read off its exe is written down. Nothing of `game::th06` means anything here.
//!
//! **Two seams, and a stage read through them.** The dialog is got past by clicking what a person would,
//! and the keyboard read is this game's one moment per tick, so a recipe's steps are walked there and the
//! word they press is put into the snapshot the game is about to read — see
//! [`crate::runtime::keyboard_state`]. Off that one call a stage is read as well: [`sight`] answers
//! [`Game::what_a_bot_sees`] out of this game's own statics, so the rules play it.
//!
//! **And the frame is orb's.** [`Th06c::hooks`] hands over this game's two chain walks and the whole
//! frame around them, so the pacing, the present and the moment the rules are asked are orb's own — see
//! [`THE_FRAME`]. What a run is made of past a stage is read as well: the supervisor's states, the game
//! manager's own numbers, the two seams a stage's numbers go in at and the record this game's own
//! replay restarts a stage from — so [`Th06c::rewinds`] answers true, and a run here has chapters, a
//! snapshot to rewind to and a file of its own to be picked up out of ([`resume`]). **And what orb draws
//! over it goes through the game's own Direct3D 11**: there is no Direct3D 8 device here for the overlay
//! to draw on, so it draws through a canvas over the swapchain this game presents its frames through —
//! see `crate::canvas` and `Overlay::over_a_swapchain`, and [`Game::draws_through_its_d3d_device`], which
//! is still false because that is the *device*'s question.
//!
//! **And a person can answer one of those screens**, which took a read of its own: this game's keys reach
//! it through the lookup orb *answers* rather than one it stands in front of, so a press has to be read
//! back out of the snapshot orb is filling before [`Game::word_in`] can take the decide out of it. Without
//! that, the question is seen and the press goes to the game's own item —
//! `the_mode_question_over_th06c`'s last two tests are the pair that press keys the way somebody at the
//! keyboard does.
//!
//! **Neither seam is an address**, which is what makes them hold at all: this exe is built with ASLR on
//! and every launch measured came up at a different base, and the pointers it calls through are ones it
//! looked up itself. So orb answers the name the game asks for and clicks the controls of the dialog it
//! puts up, and neither needs an address of the game's.
//!
//! **The addresses below are the reading of a stage, and every one of them goes through [`at`]**, which
//! adds the distance [`Game::set_image_base`] was told.
//!
//! **And every call into this game is `extern "win64"`**, which is the same ABI as the `fastcall` the
//! frame loop's own statics are declared with and the name this target has for it: `fastcall` is an x86
//! convention, a 64-bit build falls back to the C one for it, and it warns each time. The loop's own
//! signatures stay `fastcall` because that code is compiled for both architectures, where `win64` is not
//! a name at all — and on this one the two are the same four registers.
#![doc = include_str!("asm.md")]

use std::ffi::c_void;
use std::ops::Range;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering};

use orb_api::mem;

use crate::game::{
    Boundary, Building, Call, CountedDeath, Dialog, Entered, FileOpen, FrameCalls, FrameEntered,
    Game, Hooks, Lookup, Memory, Menu, Pad, PanelTile, Patch, Read, Reading, Rect, RunStart, State,
    Threads, Walks,
};
use crate::log;
use crate::score;

pub mod chapters;
/// Reachable from this crate's own tests and, through the `sim` feature, from the tests of the crates that
/// drive it — which is where the scenarios live.
#[cfg(any(test, feature = "sim"))]
pub mod image;
pub mod resume;
pub mod score_file;
mod sight;

/// Where the exe asks to be loaded, which is what the addresses below are written against.
const IMAGE_BASE: usize = 0x1_4000_0000;

/// How far it turned out to be from that ask — see [`Game::set_image_base`], which writes it, and
/// [`at`], which is the only place it is read. Nought until told, which is a game laid out at the
/// addresses it is written down with.
static SLIDE: AtomicUsize = AtomicUsize::new(0);

/// An address of this game's own where its exe actually is.
///
/// **Wrapping, because the slide is a difference and can be either way about**: an exe below its own
/// ask is a slide that borrows, and the sum is the address all the same.
fn at(address: usize) -> usize {
    address.wrapping_add(SLIDE.load(Ordering::Relaxed))
}

/// **A value in the exe's own image**, at the address the exe's own numbering gives it — read without asking
/// the host first whether it is there, which it is from the moment the exe is loaded until it exits.
///
/// **And not through `mem::read_committed`**, which asks with a `VirtualQuery` on every read: the state read
/// made about a hundred and fifty of them, the sixty-four laser slots among them, and a playback, which reads
/// the state on every update, spent `0.9ms` of each update's `1.8` on them — a pick-up of the Extra's chapter
/// 26 took a minute where th07's playback runs at `0.05ms` an update.
///
/// # Safety
/// `address` must be one of the exe's own, and the read must fit inside the image.
unsafe fn in_the_image<T: Copy>(address: usize) -> T {
    unsafe { mem::read::<T>(at(address)) }
}

/// **A call orb makes into this game, as a slot a laid-out game can fill** — the parent module's
/// `handed_over!` for a game whose addresses go through [`at`].
///
/// That macro's slot starts out holding the address itself, which is a constant there; here the address is
/// the constant plus the slide the loader chose, which no static initialiser can hold. So the slot starts at
/// nought, nought reads as the game's own function at [`at`] of the constant, and a laid-out game — which is
/// code the address space cannot hold, the reason the parent macro exists — writes its own function over it.
/// Outside a build with the simulator in it the slot does not exist and the read is [`at`] of the constant.
macro_rules! handed_over_at {
    ($slot:ident, $at:ident, $set:ident, $real:ident) => {
        #[cfg(any(test, feature = "sim"))]
        static $slot: AtomicUsize = AtomicUsize::new(0);

        #[cfg(any(test, feature = "sim"))]
        fn $at() -> usize {
            match $slot.load(Ordering::Relaxed) {
                0 => at($real),
                handed => handed,
            }
        }

        #[cfg(not(any(test, feature = "sim")))]
        fn $at() -> usize {
            at($real)
        }

        #[cfg(any(test, feature = "sim"))]
        fn $set(address: usize) {
            $slot.store(address, Ordering::Relaxed);
        }
    };
}

// **Every call orb makes into this game's code, one slot apiece.** A laid-out th06c fills them with
// functions of its own through `image`, the way th06's and th07's fill theirs; a real launch reads each as
// the game's own address.
handed_over_at!(
    RUN_CALC_CHAIN_AT,
    run_calc_chain,
    set_run_calc_chain,
    RUN_CALC_CHAIN
);
handed_over_at!(
    TAKES_AN_ELEMENT_OUT_AT,
    takes_an_element_out,
    set_takes_an_element_out,
    TAKES_AN_ELEMENT_OUT
);
handed_over_at!(
    SETS_THE_FRAMES_VIEW_AT,
    sets_the_frames_view,
    set_sets_the_frames_view,
    SETS_THE_FRAMES_VIEW
);
handed_over_at!(
    FLUSHES_THE_SPRITES_AT,
    flushes_the_sprites,
    set_flushes_the_sprites,
    FLUSHES_THE_SPRITES
);
handed_over_at!(MALLOC_AT, mallocs, set_mallocs, MALLOC);
handed_over_at!(FREE_AT, frees, set_frees, FREE);
handed_over_at!(
    THE_SOUNDS_RATE_AT,
    the_sounds_rate,
    set_the_sounds_rate,
    THE_SOUNDS_RATE
);
handed_over_at!(
    WHERE_THE_MUSIC_IS_AT,
    where_the_music_is,
    set_where_the_music_is,
    WHERE_THE_MUSIC_IS
);
handed_over_at!(
    HOW_LONG_THE_TRACK_IS_AT,
    how_long_the_track_is,
    set_how_long_the_track_is,
    HOW_LONG_THE_TRACK_IS
);
handed_over_at!(
    SEEKS_THE_MUSIC_AT,
    seeks_the_music,
    set_seeks_the_music,
    SEEKS_THE_MUSIC
);
handed_over_at!(
    PUTS_THE_RATE_TOGETHER_AT,
    puts_the_rate_together,
    set_puts_the_rate_together,
    PUTS_THE_RATE_TOGETHER
);
handed_over_at!(
    FINISHES_THE_PICTURE_AT,
    finishes_the_picture,
    set_finishes_the_picture,
    FINISHES_THE_PICTURE
);
handed_over_at!(RUN_START_AT, run_start, set_run_start, RUN_START);
handed_over_at!(
    CAPTURES_THE_FIELD_AT,
    captures_the_field,
    set_captures_the_field,
    CAPTURES_THE_FIELD
);

/// **Where the game keeps a virtual key in the 256 it plays a run off**, out of its own table — see
/// [`THE_TABLE_OF_KEYS`], and `Th06c::presses_into_its_own_keys`, which is the one thing that asks.
///
/// Nothing for a key the table has no entry for, and nothing for an index past the buffer: both are a
/// table read wrongly rather than a key nobody pressed, and writing outside those 256 bytes is writing
/// over whatever the game keeps after them.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told — see [`at`].
unsafe fn the_index_of_the_key(key: u8) -> Option<usize> {
    let table = at(THE_TABLE_OF_KEYS);
    for entry in 0..KEYS_IN_THE_TABLE {
        let entry = table + entry * A_KEY_IN_THE_TABLE;
        let wanted = mem::read_committed::<u16>(entry)?;
        // **The end of the table ends the walk**, and does not merely skip an entry: a walk that carried
        // on read whatever follows the table as entries, matched a key against one of them and wrote a
        // `0x80` at the index that garbage held. What that looked like was a launch whose title cursor
        // walked itself from item 0 to item 5 and opened whatever was there.
        if wanted == NO_MORE_KEYS {
            return None;
        }
        if wanted != u16::from(key) {
            continue;
        }
        let index = usize::from(mem::read_committed::<u16>(entry + size_of::<u16>())?);
        return (index < KEYS_IN_A_SNAPSHOT).then_some(index);
    }
    None
}

/// **The DirectInput keyboard the game reads a run with**, which orb takes away.
///
/// **Because a run's own input does not come through the keyboard at all while this is here.** The read
/// at `0x14008e0f0` takes the device where there is one — `GetDeviceState` straight into the game's own
/// key buffer at `0x14042893c`, called at `0x14008e1c8` — and the translation from virtual keys that
/// orb's answer feeds is on the *other* branch, reached only where `[0x140441fa4]` or `[0x140441fe4]` is
/// 1, or this is nought, or `[0x140441fa8]` is nought with the build number at `0x140452438` at `0x108`
/// or above — and then only on the thread whose id is at `0x140443af0`, the main one. So a menu, which
/// is up while one of those flags is, took orb's presses, and a run did not: measured with
/// the shot and the right held for six hundred frames, the player still in the middle of the field and
/// the score still `000000000` with the fairies of stage 1 in formation in front of it.
///
/// Nought here is what `Config::sent_keys` is for th06 and th07 — the game reading its keyboard the way
/// it does when DirectInput has no device.
const DEVICE: usize = 0x1_4044_1fd0;

/// **The read the game asks for by name**, which is the one thing orb stands in front of here: it is
/// resolved with `GetProcAddress` at startup into a slot of the game's own, so orb answers the ask and
/// the game keeps orb's pointer itself — see [`Read::LookedUp`], which has why the slot is the wrong
/// place to write.
const KEYBOARD_READ: &str = "GetKeyboardState";

/// **And the name it asks for to make its window**, which orb answers for the same reason: there is no
/// `CreateWindowEx` of either width in this exe's import table, only the string in its `.rdata` — see
/// `th06c/asm.md`. Its answer lands in the slot at `0x140442588` and is called at `0x140094419`.
const WINDOW_MADE: &str = "CreateWindowExW";

/// **And the name it reads a pad with**, which orb answers by being the pad — see
/// [`Game::press_into_pad`], and `Hooks::reads_a_pad` for why a pad rather than the keys.
///
/// This exe walks `xinput1_4.dll`, `xinput1_3.dll` and `xinput9_1_0.dll` for whichever is there and
/// keeps the handle at `0x140441f88`, then asks that module for this by name — see `th06c/asm.md`.
const PAD_READ: &str = "XInputGetState";

/// **The copy of the score file the game reads once, at startup, and every screen parses**: the pointer to
/// the block, written at `0x14002c2b2` and read by the parse at `0x14004e63f`, and its length beside it.
/// The result screen's deinit builds a new one from what its screens hold, frees this and points here at
/// its own (`0x140055570`..`0x140055583`) — see `th06c/asm.md`, the score file, and [`score_file`] for
/// why orb puts one here itself.
const SCORE_FILE_COPY: usize = 0x1_40bc_2ad8;
const SCORE_FILE_COPY_LENGTH: usize = 0x1_40bb_89ec;

/// **The two functions the game takes that block through**, its own CRT's `malloc` and `free`:
/// `0x14004e62e` and `0x140055555` call the first, `0x140055577` the second. Called rather than stood in
/// front of, a CRT linked into the exe having no import of them to stand at — see
/// `Memory::ProcessHeapImport`, which is why a block of theirs is a block of the game's own heap.
const MALLOC: usize = 0x1_4028_f790;
const FREE: usize = 0x1_4028_f780;

/// **And the name it makes its renderer through**, which orb answers to take the present — see
/// [`crate::dxgi`], and `Hooks::makes_its_renderer`.
///
/// `0x1402522e9` asks for it by this string at `0x1402c9860`, of the module kept at `0x1408971a0`, and
/// `0x140252358` puts what the call answered at `0x1408971d8`. The two older names beside it —
/// `CreateDXGIFactory1` at `0x14025238b` and `CreateDXGIFactory` at `0x14025242b` — are the fallback
/// this exe walks where the newest is not there, and `dxgi` has why orb answers only the newest.
const RENDERER_MADE: &str = "CreateDXGIFactory2";

/// **Which XInput button each of orb's own is**, which is what orb's answer to that read is filled from.
///
/// **The conventional mapping and not a reading of this build's own**: the directions are the d-pad's
/// four bits, the shot is `A` and the bomb is `B`, which is how a pad has been held to play this game
/// since it was sold with one. What this game's own pad screen maps them to is `th06.cfg`'s business and
/// has not been read, so a machine whose pad was configured otherwise is one where a bot presses the
/// wrong buttons — which is a thing the first launch that plays itself says out loud.
mod pad {
    pub const UP: u16 = 0x0001;
    pub const DOWN: u16 = 0x0002;
    pub const LEFT: u16 = 0x0004;
    pub const RIGHT: u16 = 0x0008;
    /// The bit number the menus take their decide off where `th06.cfg` swaps nothing: `A`'s.
    pub const DECIDE: u16 = 12;
    /// And `th06.cfg`'s own pad bindings where it is not there to be read — `0x1403e0290`'s five, see
    /// `super::supervisor::CONFIG_PAD_SHOT`.
    pub const DEFAULTS: [(usize, i16); 4] = [
        (super::supervisor::CONFIG_PAD_SHOT, 12),
        (super::supervisor::CONFIG_PAD_BOMB, 13),
        (super::supervisor::CONFIG_PAD_FOCUS, 9),
        (super::supervisor::CONFIG_PAD_PAUSE, 4),
    ];

    /// The `wButtons` bit a bit number is, nothing for one past the sixteen.
    pub const fn bit(number: u16) -> u16 {
        match number {
            0..16 => 1 << number,
            _ => 0,
        }
    }
}

/// **The `wButtons` bit the `th06.cfg` at `config` binds `field` bytes into it**, or the game's default where
/// the field cannot be read; nothing for a binding the file left negative, which is no button.
fn the_button_bound_in(config: usize, field: usize) -> u16 {
    let bound = mem::read_committed::<i16>(config + field).or_else(|| {
        pad::DEFAULTS
            .iter()
            .find(|(at, _)| *at - supervisor::CONFIG_PAD_SHOT == field)
            .map(|(_, default)| *default)
    });
    match bound {
        Some(number @ 0..) => pad::bit(number.unsigned_abs()),
        _ => 0,
    }
}

/// **The `wButtons` a word presses on a pad bound by the `th06.cfg` at `config`** — the shot, the bomb, the
/// focus and the pause on the buttons the file binds, the decide on `A` or, where the file swaps the two, on
/// `B`, and the four directions on the d-pad. `pub(crate)` for th06nc, which keeps the same file at another
/// address and binds its pad the same way.
///
/// **The game's own bindings and not a mapping of orb's**: the focus is the right shoulder by default and
/// `X` is the conversation skip, so a focus pressed on `X` skipped every conversation it was held through and
/// never slowed the player.
///
/// `swap` is the byte that swaps the decide and the cancel, which the two builds keep at different places of
/// the file: th06c's `+0x15`, th06nc's `+0x16`.
pub(crate) fn the_buttons_pressed_through(config: usize, swap: usize, word: u16) -> u16 {
    let swapped = mem::read_committed::<u8>(swap).is_some_and(|swapped| swapped != 0);
    let bound = |field: usize| the_button_bound_in(config, field - supervisor::CONFIG_PAD_SHOT);
    [
        (button::SHOT, bound(supervisor::CONFIG_PAD_SHOT)),
        (button::DECIDE, pad::bit(pad::DECIDE + u16::from(swapped))),
        (button::BOMB, bound(supervisor::CONFIG_PAD_BOMB)),
        (button::FOCUS, bound(supervisor::CONFIG_PAD_FOCUS)),
        (button::MENU, bound(supervisor::CONFIG_PAD_PAUSE)),
        (button::UP, pad::UP),
        (button::DOWN, pad::DOWN),
        (button::LEFT, pad::LEFT),
        (button::RIGHT, pad::RIGHT),
    ]
    .iter()
    .filter(|(bit, _)| word & bit != 0)
    .fold(0, |buttons, (_, button)| buttons | button)
}

/// **The same buttons in the numbering orb's sampling gives an XInput pad's** — `A` 0, `B` 1, the right shoulder
/// 5 and `Start` 7, DirectInput's order (see `joystick`'s own table) — each with the bits of the word it presses.
/// `A` both the shot and the decide, as [`Game::press_into_pad`] presses both on it; **the focus on the right
/// shoulder**, which is where `th06.cfg` binds it by default in both builds (`0x1403e0290`'s 9, th06nc's
/// `0x140429210`'s), `X` being the conversation skip there and a button orb has no bit for.
const SAMPLED_BUTTONS: [(i16, u16); 4] = [
    (0, button::SHOT | button::DECIDE),
    (1, button::BOMB),
    (5, button::FOCUS),
    (7, button::MENU),
];

/// **The word a pad orb sampled presses**, its buttons by [`SAMPLED_BUTTONS`] and the four directions off the
/// d-pad and the left stick both, which [`Game::press_into_pad`] fills together.
///
/// `pub(crate)` for th06nc, whose word and pad are this game's bit for bit.
pub(crate) fn the_word_of_a_sampled_pad(reading: Reading) -> u16 {
    let buttons = SAMPLED_BUTTONS
        .iter()
        .filter(|(button, _)| reading.buttons & 1 << button != 0)
        .fold(0, |word, (_, bits)| word | bits);
    let (up, down, left, right) = crate::joystick::hat(reading.pov);
    let (stick_up, stick_down) = crate::joystick::axis(reading.y);
    let (stick_left, stick_right) = crate::joystick::axis(reading.x);
    [
        (up || stick_up, button::UP),
        (down || stick_down, button::DOWN),
        (left || stick_left, button::LEFT),
        (right || stick_right, button::RIGHT),
    ]
    .into_iter()
    .filter(|(held, _)| *held)
    .fold(buttons, |word, (_, bit)| word | bit)
}

/// And the virtual pad pushed the way `word` is pressed, which that reads back.
pub(crate) fn pushed_as(word: u16) -> crate::joystick::Pushed {
    crate::joystick::pushed_by(
        word,
        SAMPLED_BUTTONS.map(|(button, bits)| (bits, button)),
        (button::UP, button::DOWN, button::LEFT, button::RIGHT),
    )
}

/// And that word as a menu of orb's reads a pad: the decide `A`, the cancel `B`.
pub(crate) fn the_pad_of(word: u16) -> Pad {
    let held = |bit: u16| word & bit != 0;
    Pad {
        up: held(button::UP),
        down: held(button::DOWN),
        decide: held(button::DECIDE),
        cancel: held(button::BOMB),
        left: held(button::LEFT),
        right: held(button::RIGHT),
        shot: held(button::SHOT),
        bomb: held(button::BOMB),
        focus: held(button::FOCUS),
        skip: false,
    }
}

/// **And the name it moves that window with afterwards**, which is what put a window orb had made in
/// the corner back in the middle of a screen: this exe looks up `MonitorFromWindow`, `GetMonitorInfoW`,
/// `GetSystemMetrics`, `ClientToScreen` and this together, which between them are a window centred on
/// the monitor it is on.
///
/// **And not `SetWindowPos`, which it also calls.** Both of its calls of that one pass `0x27` —
/// `SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED`, at `0x14009cbc8` and `0x14009da7b` — so
/// neither moves anything.
const WINDOW_MOVED: &str = "MoveWindow";

/// **And the two it puts that window on the screen with**, named for the reason th06nc's are — see
/// `game::th06nc`, whose launch log has both looked up before the window is made. This exe's own reading
/// is the same shape: `SetWindowPos` is in its `USER32` import table, and a window orb made in the
/// corner was measured in the middle of the primary monitor with both of the `SetWindowPos` calls orb
/// could see carrying `SWP_NOMOVE` — so the call that moves it is one of these, reached by a lookup
/// rather than through the table.
const WINDOW_SHOWN: &str = "ShowWindow";
const WINDOW_PLACED: &str = "SetWindowPos";
const WINDOW_RESTYLED: &str = "SetWindowLongW";

/// **The startup dialog's own controls**, read off the dialog while it was up: `1` is 「ゲーム起動」,
/// `200` the ボーダーレスウィンドウ it comes up on, `201`–`205` the フルスクリーン sizes, `206`–`210`
/// the ウィンドウ ones — largest first, so `210` is the 640x480 at the bottom of the list — and `400`
/// the VSync it comes up with ticked.
///
/// The dialog is `DialogBoxIndirectParamW`'s, built in memory rather than from a resource, and it comes
/// up **every launch**: measured, a launch with the `th06.cfg` and `th06.env` of an earlier one beside
/// the exe put the same dialog up again.
mod dialog {
    /// The smallest window it offers, which is what a launch nobody is watching asks for.
    pub const A_640_BY_480_WINDOW: i32 = 210;
    pub const START: i32 = 1;
}

/// **Which virtual key each of this game's buttons is pressed with**, which is what orb puts into the
/// snapshot for a word — see [`Th06c::press_into`].
///
/// **The keys 東方紅魔郷 has always been played with** and not a reading of this build's own
/// configuration: `th06.cfg` beside the exe is where that would be, and nothing has been read out of it.
/// A machine whose keys were changed there is one where a bot presses the wrong ones, which is a thing
/// the first launch that plays itself says out loud.
mod key {
    pub const SHOT: u8 = 0x5a; // Z
    pub const BOMB: u8 = 0x58; // X
    pub const FOCUS: u8 = 0x10; // Shift
    pub const MENU: u8 = 0x1b; // Escape
    pub const DECIDE: u8 = 0x0d; // Enter
    pub const UP: u8 = 0x26;
    pub const DOWN: u8 = 0x28;
    pub const LEFT: u8 = 0x25;
    pub const RIGHT: u8 = 0x27;

    /// **And the focus again as this build's own table holds it**: `VK_LSHIFT`, which the table at
    /// `0x1402bc0b0` keeps at scan `0x2a` beside `VK_RSHIFT` at `0x36` — and it holds no [`FOCUS`] at all.
    /// See [`the_key_the_games_table_holds`](super::the_key_the_games_table_holds).
    pub const FOCUS_IN_THE_GAMES_TABLE: u8 = 0xa0;
}

/// **The key this game's own table holds for a button orb presses**, which is the key itself for eight of
/// the nine.
///
/// **The focus is the ninth.** The table has `VK_LSHIFT` and `VK_RSHIFT` and no `VK_SHIFT`, so a focus
/// looked up by [`key::FOCUS`] finds no entry, and a snapshot holding `VK_SHIFT` alone is one the game's read
/// copies no focus out of. And [`key::FOCUS`] still stands for what a person presses: `GetKeyboardState`
/// fills its 256 bytes by virtual key and sets `VK_SHIFT` beside the side that is down, which is the read
/// [`Th06c::word_in`] makes. th06nc's table is the same, and so is this.
const fn the_key_the_games_table_holds(key: u8) -> u8 {
    match key {
        key::FOCUS => key::FOCUS_IN_THE_GAMES_TABLE,
        key => key,
    }
}

/// **The bits orb's own word carries for this game**, which are orb's and not the game's: what this
/// game's own read answers is a snapshot of keys, so there is no word of its own to match — see
/// [`Th06c::press_into`], which is the only place these are turned back into anything.
///
/// The same shape as th06 1.02h's own word all the same, so that a recipe written for one game reads the
/// same for the other: a `press: [decide]` is a decide in both.
mod button {
    pub const SHOT: u16 = 0x0001;
    pub const BOMB: u16 = 0x0002;
    pub const FOCUS: u16 = 0x0004;
    pub const MENU: u16 = 0x0008;
    pub const UP: u16 = 0x0010;
    pub const DOWN: u16 = 0x0020;
    pub const LEFT: u16 = 0x0040;
    pub const RIGHT: u16 = 0x0080;
    pub const DECIDE: u16 = 0x1000;
}

/// **The word the game's own read leaves behind, which a run is played with** — `0x140955394` is the
/// frame before it and the pair is what an edge is taken from, at `0x14004a683`.
///
/// The bits are [`button`]'s, which is what makes that module's numbers the game's here as well as
/// orb's: `0x14004ad29` onwards tests `0x10`, `0x20`, `0x40` and the sign bit for the four directions
/// and `0x14004ad5d` tests `0x04` for the focus.
const INPUT_THE_RUN_ACTS_ON: usize = 0x1_4095_5390;

/// **The byte that says everything is held**, which `0x14000db9d` and six other managers answer 1 and
/// return on. A frame with it set is a frame nothing of the game moved in.
const EVERYTHING_HELD: usize = 0x1_403e_d7d2;

/// **The walk of the calc chain, which is this game's frame** — every job of a tick, in priority
/// order, and the frame's own queued sound effects at the end of it.
///
/// **The one function of the frame there is to stand in.** `0x1400258a0` calls this once and then up to
/// seven times more, as many as the sound's own position says the run is behind by, and walks the *draw*
/// chain inlined in its own code — so the draw side has no function of its own and this side has one.
/// `th06c/asm.md` has the frame, the return codes the walk acts on, and how both were read.
/// **Called rather than patched**, nothing of the game's calling it once the frame around it is orb's:
/// see [`walks_the_calc_chain`], which is what orb's own loop calls and where the answer this gives is
/// turned into one that loop reads.
const RUN_CALC_CHAIN: usize = 0x1_4002_56b0;

/// **The whole frame, which orb replaces to own this game's own.** The loop at `0x14002c5b7` calls it
/// once round and nothing else calls it at all, and what it holds is the viewport, the calc chain, the
/// sound's own catch-up, the draw chain walked inline and the frame's own timing — every one of which
/// orb's own loop now calls, in that order. `th06c/asm.md` has the frame.
const THE_FRAME: usize = 0x1_4002_58a0;
/// `mov [rsp+0x10],rbx; mov [rsp+0x18],rbp; mov [rsp+0x20],rsi` — fifteen bytes to the end of the third
/// instruction, none of them rip-relative, where a 64-bit jump needs fourteen.
const THE_FRAME_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24, 0x20,
];

/// **The draw chain's head**, which the frame walks inline at `0x140025a2d` — so there is no function of
/// the game's to patch and the walk is orb's own, see [`walks_the_draw_chain`].
const THE_DRAW_CHAIN: usize = 0x1_403e_c5b0;

/// **What takes an element out of either chain**, for the walk whose callback answered nought:
/// `0x14000f620` searches the calc chain's head and then the draw chain's for it, unlinks it, calls its
/// own taken-out callback and frees it where it was heap-allocated. Only `rdx` is its argument — the
/// first thing it does is test it — and the frame passes the element there.
const TAKES_AN_ELEMENT_OUT: usize = 0x1_4000_f620;

/// The fields of a `ChainElem` a walk reads. `th06c/asm.md` has the whole `0x40` bytes of one.
mod chain {
    /// **The callback a frame runs**, called with [`ON`] in `rcx`.
    pub const CALLBACK: usize = 0x8;
    /// The link a walk follows.
    pub const NEXT: usize = 0x28;
    /// What that callback is called on.
    pub const ON: usize = 0x38;
    /// How much of an element holds the three, [`ON`] being the last of them.
    pub const A_LINK_S_SPAN: usize = ON + size_of::<usize>();
}

/// **What a chain's callback answers, and what the walk does about it** — read off `0x140025a4c`
/// onwards, which is the draw chain's own walk.
mod walked {
    /// Call it again where it stands.
    pub const AGAIN: i32 = 2;
    /// Take the element out of the chain and carry on with the next.
    pub const TAKE_IT_OUT: i32 = 0;
    /// The walk is over. Three answers with one meaning for the draw chain, whose walk ends on each of
    /// them; the calc chain's own frame reads which one it was.
    pub const OVER: [i32; 3] = [3, 4, 5];
}

/// **The view the frame sets before anything of it runs**, which is one call of the game's: `0x1400591d0`
/// flushes whatever sprites the renderer's batch still holds, sets the viewport out of the block below —
/// the corner at [`WHAT_THE_VIEWPORT_WAS_ASKED_FOR`], and the corner plus the size — with the near and far
/// planes beside it, and puts the matrix in place after it. So what the frame does is write the block and
/// call it: nought for the corner at `0x1400258c5`, and the whole 640 by 480 output at `0x1400258bb` and
/// `0x1400258cc`.
const SETS_THE_FRAMES_VIEW: usize = 0x1_4005_91d0;
const WHAT_THE_VIEWPORT_WAS_ASKED_FOR: usize = 0x1_40bc_28fc;
const THE_VIEWPORTS_WIDTH: usize = 0x1_40bc_2904;
const THE_VIEWPORTS_HEIGHT: usize = 0x1_40bc_2908;
/// The output's own size, which the frame writes as a pair of integers.
const THE_OUTPUT: (u32, u32) = (0x280, 0x1e0);

/// **What draws the sprites the renderer's batch holds** — see [`renderer`]: `0x140062330` with the batch,
/// its count, four, the word beside the count and one, which is the call the draw chain's walk makes after
/// every job at `0x140025a51`–`0x140025a88`, and zeroes the count after.
const FLUSHES_THE_SPRITES: usize = 0x1_4006_2330;

/// **The batch every draw of this game queues its sprites into**, a part of the renderer's own block past
/// the fields it had before — `0x140001970`, its constructor, writes those at the same offsets as ever.
/// Nothing is drawn until [`FLUSHES_THE_SPRITES`] is called on it, which is what the walk of the draw chain
/// does as each job is done and what the frame's own view does before it moves the viewport.
mod renderer {
    /// The sprites themselves.
    pub const SPRITES: usize = 0x2_2c3c;
    /// How many vertices of them are queued, read at `0x140025a5d` and zeroed at `0x140025a88`.
    pub const QUEUED: usize = 0x3_2b9c;
    /// And the word the flush is handed beside the count, `0x140025a67`.
    pub const FLUSHED_AS: usize = 0x3_2ba0;
}

/// **The sound's own catch-up, which the frame does between the two chains** — `0x140025992` onwards.
/// What it is for is a run that has gone by faster than the music: the frame counts the extra walks of
/// the calc chain it made into [`THE_EXTRA_WALKS`], and this is where that count is turned into a seek
/// and cleared.
const THE_SOUNDS_RATE: usize = 0x1_400a_4700;
const WHERE_THE_MUSIC_IS: usize = 0x1_400a_4600;
const HOW_LONG_THE_TRACK_IS: usize = 0x1_400a_4ce0;
const SEEKS_THE_MUSIC: usize = 0x1_400a_9660;
/// What the rate answers where it answers nothing at all, which is 44,100 — the frame's own fallback.
const SAMPLES_A_SECOND: i64 = 0xac44;
/// How many frames of them a second is, which is what the frame divides the rate by.
const FRAMES_A_SECOND: i64 = 60;
/// **How many walks of the calc chain have gone by that the music has not.** The frame increments it per
/// extra walk and this is what clears it.
const THE_EXTRA_WALKS: usize = 0x1_403e_ea60;
/// Which track is playing, and `-1` for none: the seek's second argument.
const THE_TRACK_PLAYING: usize = 0x1_403e_ea3c;
/// Whether that track is playing at all, and whether it loops — the two bytes the catch-up reads before
/// it decides which bound to wrap the seek inside.
const THE_TRACK_IS_PLAYING: usize = 0x1_403e_ea4c;
const THE_TRACK_LOOPS: usize = 0x1_403e_ea4d;
/// Where that loop starts and ends, in the same samples the seek is in.
const THE_LOOP_STARTS: usize = 0x1_403e_ea50;
const THE_LOOP_ENDS: usize = 0x1_403e_ea58;
/// **The block all of those are in**, which a restore leaves alone — see [`Game::audio_state`]. From the
/// track's own number at `0x1403eea3c`, rounded down to the sixteen, to past the extra walks at
/// `0x1403eea60`: the run's own start writes `0x1403eea38` to `0x1403eea60` together at `0x14002cc9e`.
///
/// **And not the sound player's table after it** — `0x1403eea70` to `0x1403eec60`, which the player's init
/// at `0x140055f30` fills with `-1`s and reaches from `0x1403eea64` by index. It was tried: with the block
/// run out to `0x1403eec70`, a_death_answered_by_the_chapter_again's frames after its retry were as slow as
/// before (a worst of 150ms in every 600), so the table is not what a restore costs frames by — see
/// [`THE_RENDERERS_BLOCK`] for what is.
const THE_AUDIO_STATICS: Range<usize> = 0x1_403e_ea30..0x1_403e_ea70;

/// **What the frame does after the draw chain and before it leaves**: a `-1` into the renderer's own
/// field at `0x1409552d0`'s `+0x1cb50`, read off `0x140025aaa`.
const THE_RENDERER: usize = 0x1_4095_52d0;
const WHAT_THE_FRAME_LEAVES_IN_IT: usize = 0x1_cb50;
/// **How big the renderer is**: `0x32ba8` bytes off the heap, `mov ecx,0x32ba8; call 0x140286c70` at
/// `0x14002c571`, zeroed and built before its pointer goes into [`THE_RENDERER`] at `0x14002c596` — the
/// last `0xff6c` of them the batch of sprites, see [`renderer`], which a restore leaves alone with the rest.
///
/// **A restore leaves the whole of it alone**, not the pointer to it only — see [`Th06c::live_handles`].
/// What it holds is the drawing's own side: the pointers to the D3D11 objects, which are the same kind of
/// thing as the statics in [`THE_RENDERERS_OBJECTS`] and cannot be copied into a snapshot, and the cursors
/// the game writes its vertices through them by. A chapter put back over either is a chapter describing a
/// moment the driver has left.
///
/// **And it is not what the frame rate after a chapter turned on**, which it was held back on the guess
/// that it was: the frames after a retry were as slow with the whole block out of the restore as with it
/// in. What they turned on is [`THE_SOUND_LIBRARYS_BLOCK`]. Nothing has measured a fault from putting this
/// one back either, so what keeps it here is the reason above and not a session it fixed.
pub(crate) const THE_RENDERERS_BLOCK: usize = 0x3_2ba8;

/// **And the two calls at the head of the frame's own timing, which are what finish its picture.**
/// `0x140025af0` begins with `0x1400592d0` on `0x140bc2580` — the thing that puts the frame's own
/// `60.00fps` together, off the renderer's `+0x1cb57` — and then `0x1400755c0`, which tails into the
/// renderer at `0x140072a70`. Only after those two does it read a clock and wait.
///
/// **So the two are called and the wait is not**, which took a launch to tell apart: with the whole
/// function left out, everything of the game ran and nothing new ever reached the window — the picture
/// stood at the startup splash while the log walked the front end and played a stage, and a launch of
/// the same build with `--no-frame-loop` drew every screen. The draw chain was reaching six elements
/// and five jobs in both.
const PUTS_THE_RATE_TOGETHER: usize = 0x1_4005_92d0;
const WHAT_IT_PUTS_IT_ON: usize = 0x1_40bc_2580;
const FINISHES_THE_PICTURE: usize = 0x1_4007_55c0;

/// **And the two things the wait does after it has waited**, which are the game's and not the wait's: the
/// field captured where something asked for it, and a count of the supervisor's taken down a frame.
///
/// The capture is asked for in the renderer's `+0x1c608`, nought or more being a request and `-1` none —
/// the pause menu writes `3` there with the field's rectangle in the four words after it (0x1400093b8 through
/// 0x1400093e0, where `[0x140bc2914]` says the pause shows the field behind it) — and 0x140025da4 onwards
/// serves it: 0x140007620 handed the renderer, the request and the rectangle, and `-1` written back
/// (0x140025dd8). Left out with the wait, a pause that asked for the field behind it was never given it.
const CAPTURES_THE_FIELD: usize = 0x1_4000_7620;
/// The renderer's request, and the rectangle after it — see [`CAPTURES_THE_FIELD`].
const THE_CAPTURE_ASKED_FOR: usize = 0x1_c608;
const NO_CAPTURE_ASKED_FOR: i32 = -1;
/// **The count the wait's last act takes one off** where it is not nought (0x140025de2 through 0x140025dee).
const THE_WAITS_OWN_COUNT: usize = 0x1_40bc_258c;

/// **The game's own run start**, called by the shot select's decide at `0x140035958` and the practice
/// stage select's at `0x14003629b` once their screens have written the run's numbers: the lives and the
/// bombs out of the configuration — `0x302` over the pair where the difficulty is the Extra's or the run
/// is a practice one, `0x14002cc30` — the supervisor's state to two, the decide's own sound queued, the
/// playback flag at `0x1403ed7b8` cleared, and then by the configuration byte at `0x140bc28c6`: the
/// music stopped and its track put to `-1` where it is 1 (`0x14002cc9e` on), `0x1400381d0` called on the
/// sound object instead where it is 2. No arguments; see `th06c/asm.md`. What orb calls where it writes a
/// run into being — `Th06c::start_a_run`.
const RUN_START: usize = 0x1_4002_cc00;
/// **How far into that object [`the_swapchain`] looks for one**, which is as far as the frame itself
/// reaches into it and a little over: what is being searched is the renderer's own fields, and the
/// furthest of them anything here names is the one above.
const THE_RENDERERS_OWN_FIELDS: usize = WHAT_THE_FRAME_LEAVES_IN_IT + 0x100;

/// **Where the renderer keeps its COM objects**, which `0x140251a24` onwards releases one after another:
/// `0x140897170` and `0x1408971a8` are two of them, `0x1408971b8` to `0x1408971f0` are the rest, and
/// `0x1408971a0` between them is a module handle rather than an object at all.
///
/// **A range walked and not a list of the six the release names**, because which one is the swapchain is
/// a question orb asks the objects themselves — see [`the_swapchain`] — and asking one more costs a
/// `QueryInterface`. What makes walking past a module handle safe is the vtable check there.
const THE_RENDERERS_OBJECTS: Range<usize> = 0x1_4089_7170..0x1_4089_71f8;

/// **The renderer's table of windows**, 256 entries of `0x118` — `imul rcx,rax,0x118` off `0x1407e3898` at
/// `0x1402355f4`, the count of them open at `0x1407e3890` just below, compared with `0x100` at
/// `0x1402348fd` — each holding the window's handle at `+0x8`, its swapchain at `+0x18`, the back buffer at
/// `+0xd0` and that buffer's size at `+0xd8`, and the two views made of the buffer at `+0x108` and `+0x110`.
/// It ends where the index of the window being drawn to begins, `0x1407f5098`.
///
/// **A restore must leave it alone, because the views in it are remade while a chapter is played.** Every
/// frame `0x14023b630` asks the window's client size and holds it against the size at `+0xd8`, which
/// `0x1402354d0` read off the back buffer itself; where the two differ it releases both views
/// (`0x140235310`), resizes the buffers and makes the views again. A launch that plays itself is held at a
/// `320x240` client with its buffers left `640x480` — see `crate::dxgi` — so that is every frame, and a
/// chapter put back names views released hundreds of frames since. On the real game that was the Extra's
/// chapter 13 retried: the first frame after the restore released the view the snapshot held, and
/// `0x140252ab9`'s `jmp [rax+0x10]` read a vtable of nought (`cdb`, `Attempt to read from address
/// 0000000000000010`), under `0x14023534d`, `0x140235446` and `0x14023b78a`.
const THE_WINDOWS: Range<usize> = 0x1_407e_3898..0x1_407f_5098;

/// **The CRT's own locks**, which a restore must leave alone for the reason the sound library's are: a lock
/// put back from a snapshot says a thread is somewhere it is not. This build's CRT is linked into the exe,
/// so its locks are statics of `.data` where the build before kept them in `ucrtbase.dll`, whose memory no
/// snapshot copies:
///
/// - fifteen of `0x28` at `0x140954730`, the UCRT's own table, each initialised with a spin count of
///   `0xfa0` in the loop at `0x14029ba60`..`0x14029ba88`, and the count of them at `0x140954988` after;
/// - one at `0x140954668`, the loop at `0x14028c288` stopping at one;
/// - and the three standard streams at `0x1403dd2e0`, `0x58` bytes apiece, each with its lock at `+0x30`
///   initialised at `0x140292025` — held back whole, nothing of the game's own writing to them.
///
/// **What this does not reach is the CRT's objects off the heap** — an open file's own lock and the table of
/// its handles — which are the game's heap now and put back with it. A file opened or closed between a
/// chapter being taken and put back is the case that would read differently; see `th06c/asm.md`'s
/// *What is not established*.
const THE_CRTS_LOCKS: [Range<usize>; 3] = [
    0x1_4095_4730..0x1_4095_4988,
    0x1_4095_4668..0x1_4095_4690,
    0x1_403d_d2e0..0x1_403d_d3e8,
];

/// **The block the sound library keeps its own state in**, which a restore must leave alone: five
/// `CRITICAL_SECTION`s with the pool each guards after it, and the library's flags and counters past
/// them. Each lock is the `0x230` bytes its own `InitializeCriticalSection` wrapper at `0x140244460`
/// zeroes — the section, its name at `+0x28`, the holder's thread id at `+0x22c` — and the five are
/// initialised together at `0x1400a5126` onwards: `0x140497a80`, `0x1404984b0`, `0x140497ce0`,
/// `0x140497f50`, `0x1404981c0`. The pool each guards sits at the lock's own end, which is where
/// `0x1400a8036` and `0x1400ab7b9` reach for `0x140497f10` and `0x140497cb0`.
///
/// **The bounds are where the references stop.** Every address anything in the exe reaches in
/// `0x140496f80`–`0x140498f80` lies in `0x1404979b4`–`0x1404987a8`, and every instruction that reaches one
/// is at `0x1400a14xx` or past it — the sound library's own code and the runtime helpers at
/// `0x14024xxxx`. Not one of the game's own functions touches a byte of it, so no part of a run is in
/// here and the whole of it can be held back.
///
/// **And holding it back is what the frame rate after a chapter turned on.** A shot hitting an enemy
/// queues an effect into the three words at `0x1403eea40`; the calc walk hands each to `0x1400a7e60` at
/// `0x140025783`, and that takes `0x140497f50` at `0x1400a8014`. On the real game, sampled from a thread
/// of orb's while the frame's own thread sat in the game's update — see [`crate::sampler`] — 909 of the
/// 917 samples a_death_answered_by_the_chapter_again gave up were `ntdll!ZwWaitForAlertByThreadId+0x14`,
/// every one of them under `RtlEnterCriticalSection+0xf2`, under the wrapper at `0x140244536`, under
/// `0x1400a8019`, under `0x140025783`. So the frame blocked 35ms to 137ms on that one lock, on every hit.
///
/// **And only from the retry on**: of the 236 updates that ran over two frames long, the first was the
/// startup's own and every one of the rest came after `retry chapter 1`. The 600 frames before the retry
/// had a worst of 50ms; the six windows after it, 99ms, 147ms, 153ms, 148ms, 150ms and 148ms.
///
/// **Two narrower answers were tried first and neither was it**: the sound player's effect table
/// (`0x1403eea70`–`0x1403eec60`, see [`THE_AUDIO_STATICS`]) and the renderer's own block off the heap
/// (see [`THE_RENDERERS_BLOCK`]), each held back on its own, left the same 150ms worst in every 600
/// frames. So did the five locks held back without the pools they guard: a live section whose queue was
/// rewound is a worker with nothing to release it for.
const THE_SOUND_LIBRARYS_BLOCK: Range<usize> = 0x1_4049_79b0..0x1_4049_87b0;

/// Whether the search below has been made, so that it is made once a launch rather than once a frame.
static LOOKED_FOR_A_SWAPCHAIN: AtomicBool = AtomicBool::new(false);

/// **The present orb's own frame loop makes**, which is `dxgi`'s — and, where the swapchain was not
/// taken as it was made, a look for it among the pointers this game keeps first.
///
/// **That look is the second way in and not the first.** `Hooks::makes_its_renderer` is the first: the
/// swapchain a factory of orb's makes is taken there and nothing has to be searched for at all. What the
/// search is for is a launch whose factory came some other way — the two older names orb does not
/// answer — and it is made once rather than once a frame.
///
/// # Safety
/// Must run on the game's main thread, with everything of the frame drawn.
unsafe extern "win64" fn presents(_: usize) {
    if !LOOKED_FOR_A_SWAPCHAIN.swap(true, Ordering::Relaxed) {
        unsafe { look_for_the_swapchain() };
    }
    unsafe { crate::dxgi::present() };
}

/// **Which of the pointers this game keeps is a swapchain, asked of each of them in turn.**
///
/// `QueryInterface` and nothing read off the exe, which is what makes it hold across a build: what an
/// object *is* is a thing the object answers — see `dxgi::is_a_swapchain`.
///
/// **Where it looks is the renderer's own object as well as the block of statics beside it**, every
/// eight bytes of both: what `0x140251a24` releases are the objects kept in `.data`, and the one that
/// presents may be a field of the object at [`THE_RENDERER`] instead, which is `0x1cb54` bytes at least
/// — `0x140025aaa` writes that far into it.
///
/// **And what makes searching safe is the vtable check**: a field that is not a pointer to a mapped
/// image is never called on, which is what tells an object from the module handle the same block holds.
/// That handle reads as a pointer and what it points at is a `MZ` header, so a `QueryInterface` off it
/// would be a call into the letters of that header.
///
/// # Safety
/// Must run on the game's main thread, with the game's renderer already made.
unsafe fn look_for_the_swapchain() {
    let renderer = mem::read_committed::<usize>(at(THE_RENDERER)).unwrap_or(0);
    let fields = match renderer {
        0 => 0..0,
        renderer => renderer..renderer + THE_RENDERERS_OWN_FIELDS,
    };
    let mut asked = 0;
    for slot in THE_RENDERERS_OBJECTS
        .step_by(size_of::<usize>())
        .map(at)
        .chain(fields.step_by(size_of::<usize>()))
    {
        let object = mem::read_committed::<usize>(slot).unwrap_or(0);
        if object == 0 || !mem::vtable_in_image(object) {
            continue;
        }
        asked += 1;
        if unsafe { crate::dxgi::is_a_swapchain(object) } {
            crate::log!("frame: the swapchain is the object kept at {slot:#x}");
            unsafe { crate::dxgi::take(object) };
            return;
        }
    }
    // **Said with how many objects there were to ask**, which is the half of it worth having: nought of
    // them is a search that happened before the renderer existed, and a hundred is one that happened and
    // found no swapchain.
    crate::log!(
        "frame: none of the {asked} object(s) this game keeps answers to a swapchain, so the present \
         is whatever took it"
    );
}

/// **The calc chain walked, and its answer turned into one the frame loop reads.**
///
/// The walk answers a byte — nought to carry the frame on and anything else to leave it — and writes the
/// answer the frame leaves *with* through the pointer it is handed, which the game's own frame kept on
/// its stack. So the loop's two ways out are the other way about from this game's: nought here means
/// carry on, and what the loop reads as carry on is anything that is neither of its own two.
///
/// **And the word put into the game's own keys here.** This is the last moment before the walk that reads
/// them, and the rules were asked for it on the frame around this one — see
/// [`Game::presses_into_its_own_keys`], which has what a word left for the keyboard's own read costs.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn walks_the_calc_chain(_: *mut c_void) -> i32 {
    // **Every walk of a frame after the first counted where the game counts its own**, which is what
    // puts the music back in step with a run that has gone by faster than it: `--speed` runs this more
    // than once per frame and the sound is played by something else at its own rate, so the frames
    // between them are exactly what [`tops_up_the_sound`] seeks past. The game's own fast-forward
    // counts them in this same field.
    if WALKS_THIS_FRAME.fetch_add(1, Ordering::Relaxed) > 0 {
        let walks = mem::read_committed::<i32>(at(THE_EXTRA_WALKS)).unwrap_or(0);
        unsafe { mem::write::<i32>(at(THE_EXTRA_WALKS), walks.saturating_add(1)) };
    }
    // **The frame's own word, decided and written down by the runtime**: a chapter being played back is
    // fed its recorded buttons here, a bot's word goes through the hold a question of orb's is raised on,
    // and either is noted against the frame — see `runtime::the_frames_word`, which is this game's
    // `get_input`.
    if let Some(word) = unsafe { crate::runtime::the_frames_word(&Th06c) } {
        unsafe { Th06c.presses_into_its_own_keys(word) };
    }
    let walk: unsafe extern "win64" fn(*mut i32) -> u8 =
        unsafe { std::mem::transmute(run_calc_chain()) };
    let mut leaving_with = 0i32;
    match unsafe { walk(&mut leaving_with) } {
        0 => CARRY_THE_FRAME_ON,
        _ => crate::runtime::CHAIN_EXIT_SUCCESS,
    }
}

/// What this game's calc walk answers where the frame is to carry on, in the frame loop's own terms:
/// anything that is neither `CHAIN_EXIT_SUCCESS` nor `CHAIN_EXIT_ERROR`.
const CARRY_THE_FRAME_ON: i32 = 1;

/// **How many walks of the calc chain this frame has had**, which [`Game::prepare_frame`] clears at the
/// top of one: what it decides is which of them are extra, and so how far the music has to be seeked to
/// be where the run is — see [`walks_the_calc_chain`].
static WALKS_THIS_FRAME: AtomicU32 = AtomicU32::new(0);

/// **The draw chain walked, which this game's own frame does inline and orb's loop calls here.**
///
/// The same walk as `0x140025a2d`: every element's callback with the element's own argument, and what it
/// answers says what the walk does next — see [`walked`]. Nothing it answers leaves the frame, that being
/// the calc chain's own business, so this always answers the loop's carry on.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn walks_the_draw_chain(_: *mut c_void) -> i32 {
    let mut element = at(THE_DRAW_CHAIN);
    // **How many elements this walk reaches and how many of them it runs, said once.** A walk that
    // reaches nothing is a frame with nothing drawn in it, which is a window standing at whatever it
    // last showed while everything else about the run goes on — and that is a thing no other line says.
    let mut reached = 0;
    let mut ran = 0;
    'walk: while element != 0 {
        reached += 1;
        let callback = mem::read_committed::<usize>(element + chain::CALLBACK).unwrap_or(0);
        if callback == 0 {
            element = mem::read_committed::<usize>(element + chain::NEXT).unwrap_or(0);
            continue;
        }
        let on = mem::read_committed::<usize>(element + chain::ON).unwrap_or(0);
        let run: unsafe extern "win64" fn(usize) -> i32 = unsafe { std::mem::transmute(callback) };
        ran += 1;
        loop {
            let answered = unsafe { run(on) };
            if answered == walked::AGAIN {
                continue;
            }
            unsafe { flushes_what_the_job_queued() };
            let next = mem::read_committed::<usize>(element + chain::NEXT).unwrap_or(0);
            if answered == walked::TAKE_IT_OUT {
                let out: unsafe extern "win64" fn(usize, usize) =
                    unsafe { std::mem::transmute(takes_an_element_out()) };
                unsafe { out(0, element) };
                element = next;
                continue 'walk;
            }
            if walked::OVER.contains(&answered) {
                break 'walk;
            }
            element = next;
            continue 'walk;
        }
    }
    if !SAID_WHAT_THE_DRAW_WALKS.swap(true, Ordering::Relaxed) {
        crate::log!("frame: the draw chain reaches {reached} element(s), {ran} of them with a job");
    }
    CARRY_THE_FRAME_ON
}

/// So the line above is said once rather than once a frame.
static SAID_WHAT_THE_DRAW_WALKS: AtomicBool = AtomicBool::new(false);

/// **The sprites a job of the draw chain queued, drawn as the job is done** — what the game's own walk does
/// after every job and before it acts on the answer, `0x140025a51`–`0x140025a88`: the renderer's batch
/// flushed where it holds anything, and its count put back to nought.
///
/// **After every job and not once after the walk**, because the next job sets state of its own before it
/// queues: a batch carried over is drawn under the blend, the texture or the viewport of whichever job
/// comes next, and one left after the last is not drawn in this frame at all but by the next frame's
/// [`SETS_THE_FRAMES_VIEW`], a frame late.
///
/// # Safety
/// Must run on the game's main thread, inside a walk of the draw chain.
unsafe fn flushes_what_the_job_queued() {
    let Some(renderer) =
        mem::read_committed::<usize>(at(THE_RENDERER)).filter(|renderer| *renderer != 0)
    else {
        return;
    };
    let queued = mem::read_committed::<i32>(renderer + renderer::QUEUED).unwrap_or(0);
    if queued <= 0 {
        return;
    }
    let flushed_as = mem::read_committed::<i32>(renderer + renderer::FLUSHED_AS).unwrap_or(0);
    let flush: unsafe extern "win64" fn(usize, i32, i32, i32, i32) =
        unsafe { std::mem::transmute(flushes_the_sprites()) };
    unsafe { flush(renderer + renderer::SPRITES, queued, 4, flushed_as, 1) };
    unsafe { mem::write::<i32>(renderer + renderer::QUEUED, 0) };
}

/// **The sound's own catch-up, as the frame between the two chains has it.**
///
/// What it is about is a run that has gone by faster than the music: [`THE_EXTRA_WALKS`] counts the walks
/// of the calc chain the music has not heard, and this turns that count into a seek of the track by as
/// many frames' worth of samples and clears it. Nothing at all where the count is nought, which is every
/// frame of a run going by at its own speed.
///
/// **Held inside the track's own loop where it has one**, which is what the two bounds are for:
/// `0x1400259e0` onwards takes the seek back by the loop's length until it is inside it, and a track
/// with no loop is held against its own length instead.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn tops_up_the_sound(_: usize) {
    let walks = i64::from(mem::read_committed::<i32>(at(THE_EXTRA_WALKS)).unwrap_or(0));
    if walks <= 0 {
        return;
    }
    unsafe { mem::write::<i32>(at(THE_EXTRA_WALKS), 0) };
    let track = mem::read_committed::<i32>(at(THE_TRACK_PLAYING)).unwrap_or(-1);
    if track == -1 || mem::read_committed::<u8>(at(THE_TRACK_IS_PLAYING)) == Some(0) {
        return;
    }
    let rate: unsafe extern "win64" fn() -> i32 = unsafe { std::mem::transmute(the_sounds_rate()) };
    let rate = match i64::from(unsafe { rate() }) {
        rate if rate > 0 => rate,
        _ => SAMPLES_A_SECOND,
    };
    let playing: unsafe extern "win64" fn() -> i64 =
        unsafe { std::mem::transmute(where_the_music_is()) };
    let playing = unsafe { playing() };
    if playing < 0 {
        return;
    }
    let mut seek = rate / FRAMES_A_SECOND * walks + playing;
    match mem::read_committed::<u8>(at(THE_TRACK_LOOPS)) {
        Some(0) | None => {
            let length: unsafe extern "win64" fn(i32) -> i64 =
                unsafe { std::mem::transmute(how_long_the_track_is()) };
            let length = unsafe { length(track) };
            if length < 0 || seek >= length {
                return;
            }
        }
        Some(_) => {
            let ends = mem::read_committed::<i64>(at(THE_LOOP_ENDS)).unwrap_or(0);
            let starts = mem::read_committed::<i64>(at(THE_LOOP_STARTS)).unwrap_or(0);
            if ends > starts {
                while seek >= ends {
                    seek -= ends - starts;
                }
            }
        }
    }
    let seeks: unsafe extern "win64" fn(i64, i32) =
        unsafe { std::mem::transmute(seeks_the_music()) };
    unsafe { seeks(seek, track) };
}

/// **The player**, a static object of `0x9b88` bytes: `0x140049bb0` clears it in two pieces and hands
/// it to the three `ChainElem`s it builds. `th06c/asm.md` has how it was found.
const PLAYER: usize = 0x1_40bb_89f0;

mod player {
    /// Three floats, and the third is a depth — `0x140049f53` writes `0.49` into it.
    pub const POSITION: usize = 0x7248;
    /// **The two corners the game's own hit test is given**, at `0x14004c752`–`0x14004c788`, rather than
    /// the half they are worked out from: those are the numbers being compared.
    ///
    /// **And not the four floats at `+0x7274`**, which is what a reading of `0x14004c690`'s own prologue
    /// took them for: that address is the first of *sixteen* boxes the test walks *before* it reaches
    /// the player's own, each `0x10` bytes of width, height and a centre, and `0x14004a59b` onwards
    /// clears all sixteen every frame — so the `ucomiss` against nought at `0x14004c6fa` is that walk
    /// finding an empty slot and not a player who cannot be hit. `th06c/asm.md` has both.
    pub const BOX_MIN: usize = 0x410;
    pub const BOX_MAX: usize = 0x41c;
    /// **What one frame of holding a direction moves the player by**, which is four numbers: the switch
    /// at `0x14004ad7f` picks the pair by direction and the focus bit picks which of the pair.
    pub const SPEED_STRAIGHT: usize = 0x7380;
    pub const SPEED_STRAIGHT_FOCUSED: usize = 0x7384;
    pub const SPEED_DIAGONAL: usize = 0x7388;
    pub const SPEED_DIAGONAL_FOCUSED: usize = 0x738c;
    /// And what that is scaled by on each axis before it reaches the position — `mulss` at
    /// `0x14004afd4` and `0x14004afe4`. A pair, because nothing says the two axes are scaled alike.
    pub const SCALE_ACROSS: usize = 0x7228;
    pub const SCALE_DOWN: usize = 0x722c;
    /// **Which of four states the player is in**, one byte.
    pub const STATE: usize = 0x73b8;
    /// The state a hit can land in, and the only one: `0x14004c78e` gates the hit on this byte being
    /// nought and `0x14004c7f4` is what writes [`HIT`] over it.
    pub const NORMAL: u8 = 0;
    /// The two states nothing can hit the player in and a chapter cannot start in: coming back after a
    /// death, and the seconds after a bomb or that respawn.
    pub const SPAWNING: u8 = 1;
    pub const HIT: u8 = 2;
    pub const INVULNERABLE: u8 = 3;
    /// **What the game gives a respawn**, and what orb writes under [`INVULNERABLE`] for one update — the
    /// same `0xf0` th06 1.02h gives its own.
    pub const INVULNERABLE_FRAMES: i32 = 0xf0;
    /// **Whether a bomb is going off**, a dword deep in the player's own bomb block.
    pub const BOMB_IN_USE: usize = 0x97d0;
    /// **The frames a bomb still takes the hit back in**, one off a frame at `0x14004a7ae`, and the frame
    /// it reaches nought is the frame the death is made final — `0x14004a7bd` onwards reads the lives.
    /// The player's register writes it 8 (`0x14004a45d`), the walk-in nought (`0x14004ab32`) and the
    /// walk-in's end 6 (`0x14004abb4`).
    ///
    /// **Which is what says a hit was taken at all**, where the state above says what the player is in
    /// the middle of: read beside it in the line a hit writes, because a run kept reporting hits with
    /// nothing within sixteen of the player and the two readings tell those apart.
    pub const A_BOMB_STILL_TAKES_IT_BACK: usize = 0x7254;
    /// And the frames of invulnerability the state above counts down — `0x14004ac4e`, whose pair is a
    /// `Timer`. In every other state it counts up, and states 1 and 2 read it (`0x14004aaeb`,
    /// `0x14004a93e`).
    pub const UNTOUCHABLE: usize = 0x7378;
    /// **The phase of the shot the player is firing, and the phase the update before it had** — the pair
    /// `0x14004b8aa` steps every update: `[+0x40c]` up one and `[+0x408]` keeping what it was, with the
    /// walk of the eighty slots before it skipping altogether where the two are equal (`0x14004b7e7`).
    ///
    /// **What starts one is the word's own shot bit**: `0x14004b340`'s `test r14b,cl` fires where no
    /// conversation is up (`0x14004b34c`) and the pair is idle (`0x14004b354`), writing `0` into
    /// [`SHOT_PHASE`] and `-999` into [`SHOT_PHASE_BEFORE`]. `0x14004b8be` ends the cycle at thirty and
    /// `0x14004b8d3` puts `-999` and `-1` back, which is idle, and the stage's own register writes the same
    /// qword at `0x14004a3ca`. The same pair at the same offsets as th06nc's, the two being one engine.
    pub const SHOT_PHASE: usize = 0x40c;
    pub const SHOT_PHASE_BEFORE: usize = 0x408;
}

/// **The pair one update into a firing cycle**, which is what a stage's own first frame leaves where the
/// word it was played with held the shot — and the value
/// [`Game::starts_the_shot_a_stage_began_with`] puts back for this game. Not the pair the press writes,
/// for th06nc's reason: `0x14004b8aa`'s advance runs in the same update as `0x14004b35c`'s press, so what a
/// frame ends on is one step past `(0, -999)`.
///
/// **Measured**: a pick-up of the Extra at its chapter 14 parted from the run at stage frame `558`, where the
/// playback's shot took the first of the stage's enemies down two frames before the run's —
/// `the generator is 0x88a1 where the run had 0x79c1`, `enemies` `11` against `12`, `items=1` against `0`
/// — with every earlier frame agreeing, and the run's first frame played with `input=0x0101` where the
/// pick-up's build frame was played with `0x0000`. The chapter before it had parted at the same `558` on a
/// run of another seed.
const ONE_UPDATE_INTO_A_CYCLE: (i32, i32) = (1, 0);

/// **The field the player may move in**, four floats — left, top, width, height — which
/// `0x14004b006`–`0x14004b05f` puts the position back inside.
///
/// **They initialise to nought in `.data`, and the stage callback writes them as a run starts**: as the
/// game manager's `+0x17e4`..`+0x17f0` off `rcx`, so with no rip-relative address naming them — `8`, `16`,
/// `368` and `416` at `0x140024345`, `0x14002434f`, `0x140024359` and `0x140024363`. So the player is held
/// to x from 8 to 376 and y from 16 to 432. Before that first stage they are still nought, which is why
/// [`sight::field`] does not trust them alone.
const PLAYER_AREA: usize = 0x1_403e_e7f4;

/// **The field's own size**, two floats initialised in `.data` to `384` and `448`, and what every
/// comparison a coordinate is held against reads — `0x140049f09` onwards, where a life starts at half
/// the first and 64 up from the second.
const FIELD_SIZE: usize = 0x1_403e_d024;

/// The run's own numbers, each a static of its own — `th06c/asm.md` says what each was read off.
const LIVES: usize = 0x1_403e_e804;
const BOMBS: usize = 0x1_403e_e805;
/// A word, and `0x14002ab8e` holds the cap it is written back at.
const POWER: usize = 0x1_403e_d034;
const POWER_FULL: u16 = 128;
/// Counted from one: `0x140027255` multiplies it by a thousand into `Stage * 1000 = %5d`, and
/// `0x14002731e` gives the lives and bombs their bonus only where it has reached six.
const STAGE: usize = 0x1_403e_d030;
/// Nought for Easy up to four for the Extra, which is the order the five `… Rank * …` lines at
/// `0x14030c680` onwards are picked in at `0x140027393`.
const DIFFICULTY: usize = 0x1_403e_d7d8;

/// **Whether the stage on the screen is a replay being played back**, which is the byte both of this
/// game's playbacks set and a run of somebody's own clears: `0x14003079b` sets it as the replay list's
/// own decide copies the file it names into [`THE_REPLAY_BEING_PLAYED`], `0x14002f9fd` sets it where the
/// logo screen starts the attract demo, and `0x14002cc77` clears it inside the same function that writes
/// `0x302` over the lives and the bombs — a run starting. What reads it is `0x140024e02`: set, the path
/// at [`THE_REPLAY_BEING_PLAYED`] goes to `0x14004d170` to be *read*, where clear sends
/// `replay/th6_00.rpy` to that same call to be written.
const A_REPLAY_IS_PLAYING: usize = 0x1_403e_d7b8;

/// **And whether that playback is the attract demo**, the one of the two nobody asked for.
/// `0x14002f9fd` is the only writer that sets it and `0x140037930` the one that clears it — the run's
/// own hand-over back to the title, which starts `bgm/th06_01.mid` for a run that ended and does
/// nothing but clear these two bytes for a demo that did. What reads it is the panel: `0x140029b3f`
/// draws ` DEMO PLAY` at `0x14030ca90` in place of the stage's own name, which is what a picture of one
/// said while every line of the log still read it as a stage.
const THE_PLAYBACK_IS_THE_DEMO: usize = 0x1_403e_d7d1;

/// **The path of the replay being played**, `0x104` bytes written two ways: `0x14002fa11` copies
/// `data/demo/demo00.rpy` over it where the demo starts, and `0x1400307c0` copies the name out of the
/// record the replay list's cursor is on. Read for the log alone — which of the two a playback is, is
/// [`THE_PLAYBACK_IS_THE_DEMO`]'s answer and not this one's.
const THE_REPLAY_BEING_PLAYED: usize = 0x1_403e_e86c;
/// **The front end**, a static of `0x11628` bytes whose frame callback is `0x14002ccf0` — the logo, the
/// title, the difficulty, the character and the shot selects, the practice stage select, the replay
/// list and the options, one state apiece.
///
/// **The front end and not the game manager**, which is what it was first taken for: the run's own
/// numbers are none of them in it, and what is are the screens a run is chosen on. The game manager is
/// the static at [`GAME_MANAGER`], whose stage callback `0x1400242d0` is handed it.
const FRONT_END: usize = 0x1_40ba_73c0;

mod front_end {
    /// The dword `0x14002cd2b` reads and `0x14002cd48` jumps through a table of twenty-one at
    /// `0x140036414` on. Twelve to seventeen are a stage's own six phases, which `0x14002fa4d` takes
    /// twelve off and jumps through a second table of six.
    pub const STATE: usize = 0x11608;
    /// **Which item of the front end the cursor is on**, counted from nought: `0x140030740` steps it up
    /// and wraps it at seven, and the decide at `0x140030785` reads the record of the item it names.
    ///
    /// One field whatever screen is up, which is what the front end here is — a menu whose items carry
    /// their own records, and the decide follows the one at `[+0x11610]` into the next list. Which is
    /// why a recipe aiming with it names the screen too.
    pub const CURSOR: usize = 0x28;
    /// **How many frames the screen that is up has been up**, which every screen zeroes as it arrives
    /// (`mov DWORD PTR [rdi+0x9450],esi` at `0x14002d9f5` and a dozen more) and reads its decide only
    /// past a count of — see [`TITLE_GRACE_FRAMES`] and [`SHOT_SELECT_GRACE_FRAMES`].
    pub const SCREEN_FRAMES: usize = 0x9450;
    /// **The title menu's own state**, which is the second of the twenty-one: its case at `0x14002d509`
    /// is where the cursor wraps and the items are read.
    pub const TITLE: i32 = 2;
    /// **And the shot type select's**, the last screen before a run, whose case at `0x1400353b3` writes
    /// the shot at `0x140035901` — the byte at `0x1403ed02d` — off the cursor.
    pub const SHOT_SELECT: i32 = 11;
    /// How many frames each of the two ignores a decide for on arrival: `cmp DWORD PTR [rdi+0x9450],0x14`
    /// at `0x14002d614` inside the title's case, and `…,0x1e` at `0x1400354d8` inside the shot select's.
    pub const TITLE_GRACE_FRAMES: i32 = 0x14;
    pub const SHOT_SELECT_GRACE_FRAMES: i32 = 0x1e;
    /// **The title's items, as the cursor counts them** — the same eight th06 1.02h has and in the same
    /// order, which is what `0x14002d592` steps the cursor over the second of where no clear has
    /// unlocked the Extra: `inc eax; mov [rdi+0x28],eax`.
    pub const ITEM_START: i32 = 0;
    pub const ITEM_EXTRA_START: i32 = 1;
    pub const ITEM_PRACTICE_START: i32 = 2;
    pub const ITEM_REPLAY: i32 = 3;
    pub const ITEM_SCORE: i32 = 4;
    /// **The state the Score item writes**, `mov DWORD PTR [rdi+0x11608],0xa` at `0x14002e9bd`, whose
    /// case at `0x1400362c4` puts the supervisor into [`super::STATE_SCORE`] once the screen has been up
    /// sixty frames — which is how the ranking is built, and how orb has the game write its file for a
    /// run that reached no screen of its own: see `Game::show_ranking`.
    pub const SCORE: i32 = 10;
    /// How many frames that state waits before it asks the supervisor: `cmp DWORD PTR [rdi+0x9450],0x3c`
    /// at `0x1400362c4`.
    pub const SCORE_ASKS_AFTER: i32 = 0x3c;
}

/// **The supervisor's state the ranking is shown in**, th06 1.02h's own six: the front end's Score state
/// writes it at `0x1400362d7`, the supervisor builds the result screen for it, and the screen's own
/// leaving puts one back at `0x140050d79` — and its deinit at `0x140054ab0` is what writes the score
/// file. So a run given up, which reaches no screen of its own, has its records written by orb asking
/// for this screen and taking it down again, the way th06's are.
const STATE_SCORE: i32 = 6;

/// The result screen's states the ranking is drawn in — every one from the first past its build to the
/// last before the way out, th06 1.02h's own set, this being its screen with two states added.
const RANKING_SHOWING: Range<i32> = 1..result_screen::EXIT;

/// Where the front end's state and cursor were before the ranking was asked for, plus one — nought
/// being none asked — so that they can go back once the front end is up again: see
/// `Game::restore_menu_cursor`, which has why afterwards and not on the way out.
static FRONT_END_BEFORE_THE_RANKING: AtomicUsize = AtomicUsize::new(0);
static CURSOR_BEFORE_THE_RANKING: AtomicUsize = AtomicUsize::new(0);

/// **The game manager**, the static the stage callback at [`GAME_MANAGER_ADDED`] is handed in `rcx` —
/// `lea rdi,[rip+0x37f411]` at `0x140024148`, registered as the callback's argument at `0x14002413a` —
/// and `0x1980` bytes at least, its last field read being the sub-rank at `+0x197c`. The run's own
/// numbers are its fields, and the statics named on their own above —
/// [`LIVES`], [`BOMBS`], [`POWER`], [`STAGE`], [`DIFFICULTY`], [`PLAYER_AREA`] — are fields of it too,
/// written down by address because they were read before the struct was.
const GAME_MANAGER: usize = 0x1_403e_d010;

mod game_manager {
    /// **The character and the shot**, one byte each: `0x14004d997` puts `character * 2 + shot` into the
    /// replay's header, and the character select writes the first at `0x140035329`, the shot select the
    /// second at `0x140035901`. `+0x8` before them counts the run's captures (`inc DWORD PTR [0x1403ed018]`
    /// at `0x140014e99`) and `+0x20` after them is the stage — see [`super::STAGE`].
    pub const CHARACTER: usize = 0x1c;
    pub const SHOT: usize = 0x1d;
    /// **The score**, the field the recording init saves into the previous stage's record at
    /// `0x14004da37` as a stage begins, and the capture adds its bonus to at `0x140014e1f`. The field
    /// before it is what the panel draws, which the update raises to this.
    pub const GUI_SCORE: usize = 0x7ac;
    pub const SCORE: usize = 0x7b0;
    /// **The two menus a run stops for**: the pause menu, and the menu a run out of lives is asked to
    /// continue from. Bytes, and a frame with either set is a frame nothing of the run moved in. The
    /// byte after them, `+0x7be`, says the run is *running* and gates the replay's own recording.
    pub const IS_IN_GAME_MENU: usize = 0x7bc;
    pub const IS_IN_RETRY_MENU: usize = 0x7bd;
    pub const IS_IN_PRACTICE: usize = 0x7c0;
    /// The point items collected, a word: `0x14004db64` puts the replay record's back here.
    pub const POINT_ITEMS: usize = 0x7c6;
    /// **The sixty-four spell card records**, `0x40` bytes each — see [`super::CARD_HISTORY`].
    pub const CARD_HISTORY: usize = 0x7cc;
    /// How many times the run was started over from the pause menu, a byte the result screen draws.
    pub const NUM_RETRIES: usize = 0x17e0;
    /// The power items counted towards the next step of the score, a byte the replay record restores at
    /// `0x14004dbba`.
    pub const POWER_ITEMS: usize = 0x17e1;
    /// How many extra lives the score has paid for, a byte after the lives and the bombs.
    pub const EXTRA_LIVES: usize = 0x17f6;
    /// **The seed the stage started from**, which `0x140024e9c` copies out of the generator as the stage
    /// begins and the replay's record carries — see [`super::RNG_SEED`], which is the generator itself.
    pub const RANDOM_SEED: usize = 0x195c;
    /// **The frames the stage has run**, which the game's own recording counts its entries by.
    pub const GAME_FRAMES: usize = 0x196c;
    /// **The rank**, which the enemies read. `0x1400243cf` writes eight, and a run's first stage then
    /// takes it and the bounds either side of it out of the table by difficulty at `0x1403e0010`
    /// (`0x140024cac` on) — sixteen on every difficulty; a playback takes the bounds again on every stage
    /// at `0x140024e65`–`0x140024e86`. The sub-rank counts towards a step.
    pub const RANK: usize = 0x1970;
    pub const SUB_RANK: usize = 0x197c;
}

/// **The supervisor**, the static at `0x140bc2580` whose first field is the pointer to the sound object
/// and whose state machine every screen of the game writes into — see [`supervisor`].
const SUPERVISOR: usize = 0x1_40bc_2580;

mod supervisor {
    /// **The state the supervisor is in**, which every screen writes to leave itself: the run's start at
    /// `0x14002cc40` writes two, the result screen leaving writes one at `0x140050d79`, an ending's end
    /// writes seven at `0x140018f05`. The numbers are th06 1.02h's own, this being its supervisor.
    pub const CUR_STATE: usize = 0x32c;
    /// **And the state the supervisor last acted on**, the dword before it, which its update writes from
    /// the one above as its last act — so the two disagreeing is a change asked for and not yet made.
    pub const LAST_STATE: usize = 0x328;
    /// How many lives and bombs a run starts with, out of `th06.cfg`'s `+0x10` and `+0x11`: the run's
    /// start at `0x14002cc0b` copies them over the lives and the bombs, and writes two and three over
    /// both where the difficulty is the Extra or the run is a practice one (`0x14002cc30`).
    ///
    /// **Allowed rather than `cfg`ed**, the same answer `super::STATE_THE_RUNS_OWN_END` got: a `cfg`
    /// takes the constant out of the build this module's doc links are resolved in, and what is wrong is
    /// the lint's reading of the crate rather than the constant — the item does have a reader.
    #[allow(
        dead_code,
        reason = "read by `image`, which is behind `feature = \"sim\"`"
    )]
    pub const CONFIG_LIVES: usize = 0x344;
    #[allow(
        dead_code,
        reason = "read by `image`, which is behind `feature = \"sim\"`"
    )]
    pub const CONFIG_BOMBS: usize = 0x345;
    /// **How the music is played**, 1 for the wav tracks by default (`0x01010302` at 0x140059935) and 2 for
    /// MIDI: a run's start stops the track only where it is 1 (0x14002cc70).
    #[allow(
        dead_code,
        reason = "read by `image`, which is behind `feature = \"sim\"`"
    )]
    pub const CONFIG_MUSIC: usize = 0x346;
    /// **The pad's buttons, as `th06.cfg`'s `+0x0` to `+0x9` hold them**: five `i16` bit numbers into
    /// `XINPUT_GAMEPAD`'s `wButtons` for the shot, the bomb, the focus, the conversation skip and the pause,
    /// a negative one being no button. `0x14000f710` reads them over the pad it polled (0x140bc28b4 to
    /// 0x140bc28bc), and the defaults it falls back on, `0x1403e0290`, are `12, 13, 9, 14, 4` — `A`, `B`,
    /// the right shoulder, `X` and `Start`.
    pub const CONFIG_PAD_SHOT: usize = 0x334;
    pub const CONFIG_PAD_BOMB: usize = 0x336;
    pub const CONFIG_PAD_FOCUS: usize = 0x338;
    #[allow(
        dead_code,
        reason = "read by `image`, which is behind `feature = \"sim\"`: orb presses no conversation skip"
    )]
    pub const CONFIG_PAD_SKIP: usize = 0x33a;
    pub const CONFIG_PAD_PAUSE: usize = 0x33c;
    /// And `th06.cfg`'s `+0x15`, which swaps the menus' decide and cancel: `0x14000f710` takes the decide
    /// `0x100` off bit `12 + this` and the cancel `0x200` off `13 - this`.
    pub const CONFIG_DECIDE_SWAPPED: usize = 0x349;
}

/// The supervisor's states, th06 1.02h's numbering — see [`supervisor::CUR_STATE`].
const STATE_MAINMENU: i32 = 1;
const STATE_GAMEMANAGER: i32 = 2;
/// The state a stage moves to the next through, in which the run's chain is cut and the next stage's
/// registered: `0x1400242f9` reads it as the stage callback begins to know a stage from a run's start.
const STATE_GAMEMANAGER_REINIT: i32 = 3;
const STATE_RESULTSCREEN_FROMGAME: i32 = 7;
const STATE_ENDING: i32 = 10;

/// **The generator**, one word: every draw in the exe is `xor cx,[0x14095538c]; sub cx,0x6553; … ; mov
/// [0x14095538c],ax`, and the stage callback fills its sixty-four card records from it 2048 draws deep
/// before the file is parsed over them — see `th06c/asm.md`.
const RNG_SEED: usize = 0x1_4095_538c;
/// And how many draws have been made of it since the stage seeded it: `inc DWORD PTR [0x140955388]`
/// beside every draw, cleared at `0x140024e93` as the stage begins.
const RNG_COUNT: usize = 0x1_4095_5388;

/// **The replay manager**, a pointer to an object of `0x78` bytes the run makes at `0x14004d170` — for a
/// recording as much as for a playback — and null between runs.
const REPLAY_MANAGER: usize = 0x1_4095_5380;

mod replay_manager {
    /// **The frames of the stage the record is at**, which is what its entries are written against:
    /// cleared at `0x14004d950` as a stage's record begins and stepped at `0x14004d861` once a frame.
    pub const FRAME: usize = 0x30;
}

/// **The panel's own block**, a pointer to a heap object the game manager makes — `0x1400251f0` writes it
/// — and the two fields of it a run is read by.
const GUI_IMPL: usize = 0x1_4095_5358;

mod gui_impl {
    /// **Which line of a conversation is up**, and negative for none: the enemy manager gates the boss's
    /// life bar on it at `0x140021232`, and the playback's fast-forward at `0x14004d870` on the same.
    pub const MSG_IDX: usize = 0x23a0;
}

/// **Whether the panel is drawing a boss**, a byte the enemy that carries the boss flag sets at
/// `0x140014592` and clears as it dies at `0x140020b7a`.
const BOSS_PRESENT: usize = 0x1_4095_5364;

/// **The sixty-four spell card records**, `0x40` bytes each, which the score file's `CATK` records are
/// parsed into and written out of — see `th06c/asm.md`, whose section 4 has every field.
const CARD_HISTORY: usize = GAME_MANAGER + game_manager::CARD_HISTORY;
const CARD_HISTORY_BYTES: usize = CARDS * CARD_BYTES;
const CARDS: usize = 64;
const CARD_BYTES: usize = 0x40;
/// `'CATK'`, which the stage callback writes into every record at `0x1400249cb` and the file's writer
/// looks for at `0x140054eb0`.
const CATK_MAGIC: u32 = u32::from_le_bytes(*b"CATK");
/// The card's name, copied in as the card starts by the `strcpy` at `0x140014cd0`.
const CATK_NAME: usize = 0x18;
const CATK_NAME_BYTES: usize = 0x3c - CATK_NAME;
/// The low byte of the sum of the name's own bytes, compared at `0x140014d10`: a name whose sum is
/// another's is another card in this slot, and both counts go back to nought.
const CATK_NAME_SUM: usize = 0x12;
/// The attempts, a word capped at 9999 at `0x140014d4a`.
const CATK_ATTEMPTS: usize = 0x3c;
/// **Which card a boss is on**, the dword the card's own start indexes the records by: `mov r9d,
/// [0x140b7c1d8]; shl r9,0x6` at `0x140014c99`. The enemy manager's, at [`enemy_manager::SPELLCARD_IDX`].
const CURRENT_CARD: usize = ENEMY_MANAGER + enemy_manager::SPELLCARD_IDX;

/// **The stage callback of the game manager**, `0x1400242d0`: the numbers a stage starts with go in
/// here — the rank from a table by difficulty on a run's first stage, the seed copied out of the
/// generator at `0x140024e9c` —
/// and the stage is built out of them at `0x140024ea3`. Answers nought where it built one and `-1`
/// where it could not. Registered as the game manager's element's own callback and called with the
/// manager in `rcx`, which is what makes it [`Entered::InEcx`].
const GAME_MANAGER_ADDED: usize = 0x1_4002_42d0;
/// `mov [rsp+0x18],rbx; mov [rsp+8],rcx; push rbp; push rsi; push rdi; push r12` — fifteen bytes, none of
/// them rip-relative, where a 64-bit jump needs fourteen.
const GAME_MANAGER_ADDED_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x18, 0x48, 0x89, 0x4c, 0x24, 0x08, 0x55, 0x56, 0x57, 0x41, 0x54,
];

/// **The stage's own build**, `0x1400572d0`, called once in the exe — at `0x140024ea3`, from the callback
/// above with the stage in `ecx` — after the seed has been copied and before anything of the stage has
/// drawn from the generator. Which is the one place a resumed run's seed can go in, and why it is a
/// seam of its own — see [`Hooks::stage_building`].
const STAGE_REGISTER_CHAIN: usize = 0x1_4005_72d0;
/// `mov [rsp+8],rbx; mov [rsp+0x10],rsi; push rdi; sub rsp,0x20` — fifteen bytes.
const STAGE_REGISTER_CHAIN_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x74, 0x24, 0x10, 0x57, 0x48, 0x83, 0xec, 0x20,
];

/// **The calc chain's own head**, the other of the two — see [`THE_DRAW_CHAIN`], and
/// `th06c/asm.md` for why both are sentinels rather than pointers.
const THE_CALC_CHAIN: usize = 0x1_403e_c5f0;

/// Longer than any chain this game builds, so that a chain caught in a state whose links do not end is a
/// walk that answers nothing rather than one that does not return.
const CHAIN_LINKS: usize = 64;

/// **The job the screen a run ends at registers**, which is the one way to that screen: it is built on
/// the heap, nothing holds a pointer to it, and `0x14004fdcd` is where its own constructor puts it into
/// the element it registers — see [`what_a_job_runs_on`].
const RESULT_SCREEN_UPDATE: usize = 0x1_4004_ff40;

/// **The job an ending runs as**, which is what says one is running: nothing else does.
///
/// **Because an ending of this game is a job and not a screen.** It runs inside the scene the stage it
/// followed had — one launch's frame lines read `scene=11 stage=5 enemies=0 bullets=0 boss=false` for the
/// whole of it — so the game manager's own state says a stage is still up, and `State::in_ending` had
/// nowhere to come from. The chain is where it is: this callback arrives on the frame the stage's own ten
/// jobs go and stands alone until the screen a run ends at registers.
///
/// **Found by walking the chain rather than by the file it reads.** `th06ED.dat`'s string sits at
/// `0x14030bf08` and is named once in `.text`, by the `lea r8` at `0x140018c62` — inside the ending's own
/// code, which says what the ending loads and not which callback of the chain is the ending. So
/// [`say_the_jobs_when_they_change`] wrote the chain's callbacks down whenever the set of them
/// changed, and one launch of a Normal clear named this one outright: the stage's jobs at `f98220`, this
/// one alone from `f98220` to `f106680` — `8,460` frames, which is the ending's own length — and
/// [`RESULT_SCREEN_UPDATE`] registered on the frame after it went.
///
/// **And the staff roll goes with it.** `State::ending_script` is what th06 1.02h's skip watches to stop
/// where the roll begins, and this game's own script is unread, so the skip runs to where this job leaves
/// — which is the roll's end and not its start.
const ENDING_UPDATE: usize = 0x1_4001_8110;

mod result_screen {
    /// **The screen's own state**, which its update reads at `0x14004ff71` and jumps through a table of
    /// eighteen at `0x1400535c8` on.
    pub const STATE: usize = 0x5194;

    /// **The one of those eighteen the name is typed in.** `0x14004fde9` writes it as the screen is
    /// built — nine where `[0x1403ed7d0]` is clear and seventeen where it is not, which is the run that
    /// is asked for no name at all.
    pub const TYPING_A_NAME: i32 = 9;

    /// **The field the entry types into**, eight bytes: `0x140052044` writes the character at
    /// `[this + 0x5173 + index]`, the index being [`TYPED`] capped at seven by the `cmovge` at
    /// `0x140052020`.
    ///
    /// **The same shape th06 1.02h has**, whose own is `[this+0x5193]` with the cursor at `+0x20` — this
    /// build has the cursor at the front of the object instead and the field `0x20` earlier.
    pub const NAME: usize = 0x5173;
    pub const NAME_BYTES: usize = 8;

    /// **How many characters have been typed**, which `0x140052058` steps up and `0x140052051` stops at
    /// eight — and reaching eight is what `0x140052063` puts the cursor on [`THE_END_ITEM`] for.
    pub const TYPED: usize = 0x517c;

    /// **Which character of the grid the cursor is on**, the first dword of the screen's own object:
    /// `0x140052023` reads it, `0x14005202b` takes the character at `0x14030d500 + it`, and
    /// `0x14005203c` writes a space where it is `0x5e`.
    ///
    /// **The grid is reached off the image base in a register** and not rip-relative —
    /// `[rax + r11 + 0x2c8af0]`, r11 being the base — which is why no search of the exe for a reference
    /// to it as an operand finds one. `th06c/asm.md` has the sweep that said so.
    pub const CURSOR: usize = 0x0;

    /// **The item that ends the entry**, which is the one past the space at `0x5e`: `0x140052036` leaves
    /// the typing for anything past that space, and this is what `0x140052063` puts the cursor on once
    /// eight characters have been typed.
    pub const THE_END_ITEM: i32 = 0x5f;

    /// **The state the screen asks whether to save a replay in**, which is the one a run of orb's is taken
    /// past: the question's own case reads the name entry's answer and calls the save at `0x1400528cc`.
    pub const SAVE_REPLAY_QUESTION: i32 = 10;
    /// **And the state the screen leaves in**, which every way out of it writes and whose case puts the
    /// supervisor back to the front end at `0x140050d79`. Written over whatever state the screen is in,
    /// which is the game's own way out — the one its own 「いいえ」 takes.
    pub const EXIT: i32 = 17;
}

/// **The object a job of the calc chain is run on**, found by the callback it registered.
///
/// **Which is the only way to a screen this game keeps on the heap**: nothing holds a pointer to one, and
/// the element the screen registers carries both — the callback at [`chain::CALLBACK`] and the object at
/// [`chain::ON`].
///
/// Every step asked of the memory map rather than read outright, a chain being a list the game is free to
/// be halfway through rebuilding.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn what_a_job_runs_on(callback: usize) -> Option<usize> {
    unsafe { the_jobs() }.runs_on(callback)
}

/// **Every job the calc chain holds, in the order it holds them**, walked once — what the chain's own
/// questions are asked of: which object a job runs on ([`what_a_job_runs_on`]), whether a job is there at all
/// ([`Jobs::hold`]), and the set of them ([`say_the_jobs_when_they_change`]).
///
/// **Once for the state read's three and not once apiece**, a playback reading the state on every update and
/// the three walks being what was left of that read's cost.
struct Jobs {
    /// Each job's callback and what it is called on.
    held: [(usize, usize); CHAIN_LINKS],
    count: usize,
}

impl Jobs {
    fn runs_on(&self, callback: usize) -> Option<usize> {
        self.held[..self.count]
            .iter()
            .find(|(held, _)| *held == at(callback))
            .map(|(_, on)| *on)
            .filter(|on| *on != 0)
    }

    /// **Whether the chain holds a job of its own**, which is what a thing that neither moves nor is pointed
    /// at is read by — see [`ENDING_UPDATE`]. Apart from [`Jobs::runs_on`] because the two ask different
    /// things: that one wants the object a job runs on and answers nothing for a job whose object is null,
    /// where this one is about the job being there at all.
    fn hold(&self, callback: usize) -> bool {
        self.held[..self.count]
            .iter()
            .any(|(held, _)| *held == at(callback))
    }
}

/// The calc chain walked, every step asked of the memory map rather than read outright — see
/// [`what_a_job_runs_on`].
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_jobs() -> Jobs {
    let mut jobs = Jobs {
        held: [(0, 0); CHAIN_LINKS],
        count: 0,
    };
    let mut element = unsafe { in_the_image::<usize>(THE_CALC_CHAIN + chain::NEXT) };
    while element != 0 && jobs.count < CHAIN_LINKS {
        let Some(link) = the_link(element) else {
            break;
        };
        jobs.held[jobs.count] = (link.callback, link.on);
        jobs.count += 1;
        element = link.next;
    }
    jobs
}

/// One element of a chain, as a walk reads it.
struct Link {
    callback: usize,
    next: usize,
    on: usize,
}

/// **The element at `element`, read in one ask of the host** — its callback, the link on and what it is called
/// on all lie in its first [`chain::A_LINK_S_SPAN`] bytes.
///
/// **One and not a read apiece**, because a playback reads the state on every update and walks the chain on
/// each: at two and three asks a link the walks were most of what was left of the state read's `0.58ms` an
/// update once the image's own reads had stopped asking.
fn the_link(element: usize) -> Option<Link> {
    let bytes = mem::read_committed_bytes(element, chain::A_LINK_S_SPAN)?;
    let word = |at: usize| {
        bytes
            .get(at..at + size_of::<usize>())
            .and_then(|word| word.try_into().ok())
            .map(usize::from_le_bytes)
    };
    Some(Link {
        callback: word(chain::CALLBACK)?,
        next: word(chain::NEXT)?,
        on: word(chain::ON)?,
    })
}

/// **Every job the calc chain holds, said the frame the set of them changes and on no other frame** —
/// which is how the job that draws something nothing points at is found.
///
/// **Because a screen this game keeps on the heap is reached through the job that registers it**, and
/// which job that is, for a thing no string of the exe is referenced from, is what a launch says rather
/// than what a listing does — see [`what_a_job_runs_on`], and `th06c/asm.md`'s note on the name
/// entry's own grid, whose address appears in `.text` as no operand at all. So the chain is walked and
/// the callbacks are written down whenever one arrives or leaves: the job added on the frame the last
/// boss dies is the ending's own.
///
/// **On the change and not every frame**, the walk being `CHAIN_LINKS` reads of the game's memory: what
/// is interesting is the arrival, and a line a frame would be a page a second.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
fn say_the_jobs_when_they_change(jobs: &Jobs) {
    static WAS: AtomicUsize = AtomicUsize::new(0);
    let held = &jobs.held[..jobs.count];
    // A number the set of them makes, so that the walk is compared without keeping the last one: the
    // callbacks added up, each turned by where it sits so that two jobs swapping is a change as well.
    let fingerprint = held
        .iter()
        .enumerate()
        .fold(0usize, |sum, (link, (callback, _))| {
            sum.wrapping_add(callback.rotate_left(link as u32 % 64))
        });
    if WAS.swap(fingerprint, Ordering::Relaxed) == fingerprint {
        return;
    }
    let slide = SLIDE.load(Ordering::Relaxed);
    let said = held
        .iter()
        .map(|(callback, _)| format!("{:#x}", callback.wrapping_sub(slide)))
        .collect::<Vec<_>>()
        .join(" ");
    crate::detail!("chain: {} job(s): {said}", jobs.count);
}

/// **The result screen's state, said whenever it changes** — the same way the chain's jobs are said, and
/// for the same reader: a recipe walking that screen has nothing else to read which of its eighteen states
/// (`0x1400535c8`) a press landed on, and a launch that sat in one for the rest of its bound said nothing
/// about which.
///
/// `-1` is no screen, which is how the line marks the screen going down.
///
/// # Safety
/// Must run on the game's main thread.
fn say_the_result_screens_state_when_it_changes(jobs: &Jobs) {
    static WAS: AtomicI32 = AtomicI32::new(-1);
    let state = match jobs.runs_on(RESULT_SCREEN_UPDATE) {
        Some(screen) => mem::read_committed::<i32>(screen + result_screen::STATE).unwrap_or(-1),
        None => -1,
    };
    if WAS.swap(state, Ordering::Relaxed) != state {
        crate::detail!("result: the screen a run ends at is in state {state}");
    }
}

/// **The screen a name is being typed into, or nothing where none is** — the result screen while its own
/// state is [`result_screen::TYPING_A_NAME`].
///
/// **The state as well as the screen, so that the ranking the *Score* item opens answers nothing**: that is
/// the same object with no run behind it, and a name written there would be one nobody's run was ranked
/// under.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_entry_a_name_is_typed_into() -> Option<usize> {
    let screen = unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }?;
    (mem::read_committed::<i32>(screen + result_screen::STATE)
        == Some(result_screen::TYPING_A_NAME))
    .then_some(screen)
}

/// **The bullets**, a static of `0xf7c20` bytes — see `th06c/asm.md`, where the array was read off
/// the update's own loop at `0x14000db90`.
const BULLET_MANAGER: usize = 0x1_4095_5410;

mod bullet_manager {
    pub const BULLETS: usize = 0x8;
    pub const SLOTS: usize = 0x280;
    pub const BYTES: usize = 0x5d0;
    /// Counted up per live one at `0x14000dccb`.
    pub const LIVE: usize = 0xe8808;
    pub const STEP: usize = 0x08;
    pub const POSITION: usize = 0x30;
    /// **Which of five states it is in**, a word the update jumps on at `0x14000dcd2`, and nought is a
    /// slot nothing is in.
    pub const STATE: usize = 0x44;
    /// **And the only one the player is tested against.** State 1's branch is the one whose path reaches
    /// `0x14000e3fb`, and the manager's one call of the player's own test `0x14004c690`, at `0x14000e490`,
    /// is on it. States 2 to 4 fall into that branch on the frame they become 1 (`0x14000ddee`) and are
    /// tested then; state 5 ends at the manager's own loop tail with no test made.
    pub const FLYING: u16 = 1;
    /// **The size the game's own hit test is given**, which `0x14000e47e` hands over beside the position
    /// at `0x14000e490`.
    pub const SIZE: usize = 0x5a4;

    /// **What a bullet's own step is scaled by before it reaches its position**, which is its state's and
    /// not one number — five integrations, one per branch of `0x14000dcd2`'s jump:
    ///
    /// | state | where | what it multiplies by |
    /// | --- | --- | --- |
    /// | 1 | `0x14000e372` | nothing at all: `[+0x30] += [+0x8]` |
    /// | 2 | `0x14000dd57` | a half — `1.0 / 2.0` |
    /// | 3 | `0x14000dd6d` | `1.0 / 2.5` |
    /// | 4 | `0x14000dd7e` | `1.0 / 3.0` |
    /// | 5 | `0x14000dcf9` | a half — `0x14030dfc8`'s `0.5` in `xmm13` (`0x14000dc4f`) |
    ///
    /// **The jump is a chain of subtractions and not a table**: `0x14000dcd6` onwards takes `r15d` — one —
    /// off the state four times over, sending 1 to `0x14000de04`, 2, 3 and 4 to the divisor chain at
    /// `0x14000dd52`, and letting 5 fall through to `0x14000dcf9`.
    ///
    /// **State 5's half is the bullet's own step**: `rsi` and `rdi` are the bullet's `+0x8` and `+0x30`
    /// (`lea` at 0x14000dc4b and 0x14000dc58, stepped a slot at a time at 0x14000e4e1 and 0x14000e4e8), and the
    /// `rbx+0x490` the `lea` between them takes is the argument of the animation call after. A launch that read
    /// `the widest of them moved 3.30,5.38 saying 1.65,2.69, in the game's own state 5` was a bullet the hit
    /// test or the graze test had just put in state 5 (0x14000e499, 0x14000e464): it moved its whole step in
    /// state 1 on that frame, before the test, and is read in state 5 after it.
    ///
    /// **And getting this wrong is what a flat half cost**: a launch that scaled every state by `0.5`
    /// had the rules judge ten of ten bullets as moved by something other than their own step, so every
    /// one of them was feared over the whole box it had just swept and none was projected — the model
    /// could only dodge what was already on the player, and that run was down to its last life by
    /// stage 3's boss.
    pub const fn a_steps_scale(state: u16) -> f32 {
        match state {
            2 | 5 => 1.0 / 2.0,
            3 => 1.0 / 2.5,
            4 => 1.0 / 3.0,
            _ => 1.0,
        }
    }
}

/// **The lasers**, which are in that same static: `0x14000e50d` puts the array at `+0xe8818` of the
/// bullet manager and walks sixty-four of them, right after the bullets and in the same update.
const LASERS: usize = 0x1_40a3_dc28;

mod laser {
    /// `sub rdi,r15` from `0x40` at `0x14000e697`.
    pub const SLOTS: usize = 0x40;
    /// `add rbx,0x278` at `0x14000e690`.
    pub const BYTES: usize = 0x278;
    /// A byte, and nought is a slot nothing is in — `0x14000e570`. `0x14000e66e` clears it once the
    /// near end has run off the far edge.
    pub const IN_USE: usize = 0x254;
    /// **How wide it is**, and the test is given half of it: `0x14000e5c2` reads it and `0x14000e5dc`
    /// halves it into the size handed to `0x14004c890`.
    pub const WIDTH: usize = 0x4;
    /// **What a hit on it is, to either side of its own line**: a quarter of that width, which is two
    /// halvings read off the exe.
    ///
    /// `0x14000e5dc` multiplies the width by `xmm13` into the size the test is handed — `[rsp+0x34]`, the
    /// second of the pair at `0x14000e5e7`, which `0x14004c890` compares on the axis *across* the beam —
    /// and that test multiplies it by `xmm6` again at `0x14004c96c`. Both registers hold
    /// `[0x14030dfc8]`, which is `0.5`, and the same `xmm6` is what the bullet test halves the bullet's
    /// own size with — where the model reads the half as the size over two and is right.
    ///
    /// **A half was here, set against a run, and the run had another explanation.** A laser `11.7` off a
    /// player, modelled `6.0` wide by this arithmetic, hit it — and a laser's origin is carried by the
    /// script that fired it, which nothing read at the time: th06c's Extra boss fires one twice this
    /// width from where she stands and walks it `6.4` a frame, so the lane that hit was `6.4` from where
    /// the judgement had it and not twice as wide. What a half cost meanwhile is in `th06c/asm.md`:
    /// the gaps of a fan of lasers all read as shut, and two hits of an Extra run with `nowhere open to
    /// it` on every one of the nine moves.
    pub const HALF_OF_ITS_WIDTH: f32 = 4.0;
    /// How far along it runs, from the near end to the far one. The far end is what grows and the near
    /// end follows it once they are [`LENGTH`] apart — `0x14000e57d`–`0x14000e5af`.
    pub const NEAR: usize = 0x10;
    pub const FAR: usize = 0x14;
    /// And how much further the far end reaches each frame, added straight on with no scale.
    pub const GROWS: usize = 0x24c;
    /// **Where it is fired from and which way it points**, which is the pivot `0x14004c890` turns the
    /// player about: `xmm8`/`xmm9` from `[r9]`, and the angle in `xmm0`.
    pub const ORIGIN: usize = 0x258;
    pub const ANGLE: usize = 0x240;
    /// The frames it has been in its phase, and the phase — `0x14000e5d5` reads the phase and
    /// `0x14000e683` is the pair `+0x8` and `+0xc` are.
    pub const FRAMES: usize = 0xc;
    pub const PHASE: usize = 0x270;
    /// **The frame of its first phase it starts hitting on**: `0x14000e7a1` runs the player's own test
    /// only once the frames have reached it, so a laser before that is drawn and hits nothing.
    pub const HITS_FROM: usize = 0x26c;
    /// In this phase the test is run every frame — `0x14000e847` — and in the next while its frames are
    /// under the slot's own `+0x0` (`0x14000e8fb`, the call at `0x14000e93d`).
    pub const FIRING: u8 = 1;
    /// And from this one it is going out, `0x14000e8bd` onwards taking `width × (1 − frames ÷`
    /// [`NARROWS_OVER`]`)`, halving it and writing it where the length goes.
    pub const FADING: u8 = 2;
    /// **How many frames it goes out over**, which is what that fraction is of.
    ///
    /// **And what it takes down is the length**, not the width: `0x14000e8ea` writes that number into
    /// `[rsp+0x30]`, the slot `0x14004c95a` compares *along* the beam, where `0x14000e5e7` put the width
    /// in `[rsp+0x34]` for the axis across it. So a fading laser is a stub about the middle of its own
    /// line at the width it always had, and `0x14000e8bb` skips the whole of it where this is nought.
    pub const NARROWS_OVER: usize = 0x18;
}

/// **The enemies**, a static of `0xff020` bytes whose frame callback is `0x140020780` and whose array
/// that function's own loop says the shape of.
const ENEMY_MANAGER: usize = 0x1_40a7_e1e0;

mod enemy_manager {
    pub const ENEMIES: usize = 0x8;
    /// `cmp r15d,0x100; jl` at `0x140021365`.
    pub const SLOTS: usize = 0x100;
    /// `add rbx,0xfd0` at `0x140021357`.
    pub const BYTES: usize = 0xfd0;
    /// Counted up per live one at `0x14002085d`, into the manager's first dword.
    pub const LIVE: usize = 0x0;
    /// **The clock the stage's script runs on**, which `0x1400201c2` writes as the timeline steps — the
    /// one clock a midstage boundary can be written in, since it moves the same way however the run
    /// goes.
    pub const TIMELINE_TIME: usize = 0xfdfdc;
    /// **Whether a spell card is up**, `0x140b7c1d0`: the capture's branch is gated on it at
    /// `0x140014d64`.
    pub const SPELLCARD_IS_ACTIVE: usize = 0xfdff0;
    /// **And which**, `0x140b7c1d8`, the index the card's start shifts into the records by.
    ///
    /// The bosses themselves are pointers at `+0xfe008` — `mov [rcx+rax*8+0xfe008],r14` at `0x14001458a`
    /// puts an enemy there as the script names it one — and are not read: the slot the fight is with is
    /// found by its flag instead, see `sight::the_boss_the_fights_own_slot`.
    pub const SPELLCARD_IDX: usize = 0xfdff8;
}

mod enemy {
    /// `lea rbp,[rbx+0xa8]` at `0x140020860`, and `+0xfa4` onwards is what the walk adds to it.
    pub const POSITION: usize = 0xa8;
    /// **The size the player's own hit test is given**, which `0x140020db5` onwards multiplies by two
    /// thirds before handing over — and that test halves what it is given, so what a hit is is a third
    /// of this on each side.
    pub const SIZE: usize = 0x2a4;
    /// **What a hit is, on each side of it**: a third of the size above, which is that size times two
    /// thirds at `0x140020db5` and then halved by the test at `0x14004c690`.
    ///
    /// **A third and not a sixth.** The half was worked out as this over two — the size over three and
    /// then halved again — which is every enemy's body read at half the reach the game gives it.
    pub const A_THIRD_OF_ITS_SIZE: f32 = 3.0;
    /// The flags byte the walk itself gates on: `cmp BYTE PTR [rbx+0xb4],0; jge` at `0x140020850`
    /// skips a slot whose top bit is clear.
    pub const FLAGS: usize = 0xb4;
    pub const IN_USE: u8 = 0x80;
    /// **And whether the game runs its body against the player at all.** `0x140020d82` requires `0x4`,
    /// `0x140020d9c` refuses where `+0xb6`'s `0x8` is set, and `0x140020daf` runs the test only where
    /// the low two bits are both up.
    pub const SOLIDITY: usize = 0xb5;
    pub const TESTED: u8 = 0x4;
    pub const SOLID_BITS: u8 = 0x3;
    /// **And which one the fight is with.** `0x140021222` tests this bit of the same byte and, for the
    /// one that has it, works `+0x21c / +0x220` out into `0x14095534c` — which is the life bar the panel
    /// draws, so the enemy carrying it is the boss.
    pub const IS_THE_BOSS: u8 = 0x8;
    /// Its life, which `+0x220` beside it is the most of.
    pub const LIFE: usize = 0x21c;
    pub const MORE_FLAGS: usize = 0xb6;
    pub const PASSES_THROUGH: u8 = 0x8;
    /// **The frames its attack has run**, the dword the panel's own countdown is worked out from:
    /// `0x1400202df` takes it off the limit at `+0xbb4` and divides by sixty into `0x14095536c`, and
    /// `0x14002030c` ends the attack where it reaches that limit. Put back to nought as the script moves
    /// the boss on, which is what marks the next attack.
    pub const TIMER: usize = 0x4;
}

/// **The items**, a static of `0x2a160` bytes whose walk is `0x14002aa70`.
const ITEM_MANAGER: usize = 0x1_40b7_d260;

mod item {
    pub const ITEMS: usize = 0x8;
    /// `cmp r9d,0x200; jl` at `0x14002ba56`.
    pub const SLOTS: usize = 0x200;
    /// `add r14,0x150` at `0x14002ba32`.
    pub const BYTES: usize = 0x150;
    /// A byte, and nought is a slot nothing is in — `cmp BYTE PTR [r14],bpl; je` at `0x14002abe0`.
    pub const ALIVE: usize = 0x0;
    pub const POSITION: usize = 0x10;
    /// **Which of seven kinds it is**, a signed byte the walk jumps through a table of seven at
    /// `0x14002bb1c` on — `0x14002ae97`. What each number is is what its own case does:
    pub const KIND: usize = 0x34;
    /// `inc r15w` into the power at `0x14002af2e`.
    pub const POWER: i8 = 0;
    /// The point items counted up at `0x14002b2d8`.
    pub const POINT: i8 = 1;
    /// `add r15w,0x8` into the power at `0x14002b39e`.
    pub const BIG_POWER: i8 = 2;
    /// The bombs, stopped at eight and stepped up at `0x14002b609`.
    pub const BOMB: i8 = 3;
    /// The power written to its cap at `0x14002b6eb` on.
    pub const FULL_POWER: i8 = 4;
    /// The lives, the same shape at `0x14002b638`.
    pub const LIFE: i8 = 5;
    /// The one a bomb leaves behind, worth a score off the graze — `0x14002b8db`, and the bullet
    /// manager spawns it as kind 6 at `0x14000e46d`.
    pub const CANCELLED: i8 = 6;
}

/// How much of the 256-byte snapshot is a key: every index, which is what `GetKeyboardState` fills.
const KEYS_IN_A_SNAPSHOT: usize = 256;

/// **The 256 bytes the game plays a run off**, which its own read fills through the table below — see
/// `th06c/asm.md`, and [`Th06c::presses_into_its_own_keys`], which writes into it for a frame that
/// is past the read.
const THE_KEYS_THE_GAME_READS: usize = 0x1_4042_893c;

/// **When the game last read the keyboard, in `timeGetTime`'s own milliseconds**, the same guard th06nc's
/// read has: both paths of the read ask `timeGetTime` through `0x1404421c0` — `0x14008e16a` and
/// `0x14008e257` — hold this against the answer at `0x14008e170` and `0x14008e25d`, and leave the read
/// out where the two are equal, `0x14008e20a` and `0x14008e269` writing the new time where they are not.
///
/// **Which cost a pick-up of the Extra its chapter.** Fed `chose=0x0041` on frame 692 after `0x0081` on 691,
/// the playback's run acted on `input=0x01c1` — the left fed and the right of the frame before — where the
/// run it was put back from had `0x0141` — `144.00` against `136.00` across — and the playback took a hit at
/// frame 797 that the run never took.
const WHEN_THE_KEYBOARD_WAS_READ: usize = 0x1_4042_8938;

/// **And when the pad was**, the same shape one device along, as th06nc has it: `0x14008db48` asks the
/// same clock, `0x14008db4e` holds the device record's first dword against it and `0x14008db50` leaves the
/// poll out where they are equal — the poll being the one that calls `XInputGetState` through
/// `0x140441f90` at `0x14008db58`. The address is the first device's record: `0x14008d2aa` onwards takes
/// `0x140428930 + 0x2b78` and `0x530` bytes a device.
///
/// **What a keyboard forgotten alone did**: the same pick-up parted *earlier*, at frame 527 on the same
/// `input=0x01c1` against the run's `0x0141` — a word of this frame's keys over the pad's word of the frame
/// before, the pad's half being ORed into the keyboard's.
const WHEN_THE_PAD_WAS_READ: usize = 0x1_4042_b4a8;

/// What orb writes into it to make the next read a real one: a time `timeGetTime` cannot answer, the clock
/// being milliseconds since the machine started.
const NO_READ_YET: u32 = 0;
/// **What a key that is down reads as there**, the same `0x80` a `GetKeyboardState` snapshot carries.
const DOWN_IN_ITS_OWN_KEYS: u8 = 0x80;
/// And what one that is not reads as, which is what the game's own read leaves for every key nobody is
/// holding — see `Th06c::presses_into_its_own_keys`, which writes it for the keys a word does not hold.
const UP_IN_ITS_OWN_KEYS: u8 = 0x00;
/// **The game's own table from a virtual key to the index it keeps that key at**: six bytes an entry —
/// the virtual key, the index, and two bytes of nought — walked at `0x14008e2c0`, and 110 entries before
/// the key of `0xffff` that ends it, the walk loading the first key, `0x41`, at `0x14008e2a2` rather
/// than reading it. `A` is `0x41 -> 0x1e`, which is `DIK_A`, so the buffer is indexed by DirectInput's
/// own scan codes.
///
/// **Read rather than written down**, because a scan code per button is nine numbers to get right and
/// this is the mapping the game itself uses.
const THE_TABLE_OF_KEYS: usize = 0x1_402b_c0b0;
/// How far apart its entries are, and how many of them there can be before the walk gives up: the table
/// ends at a key of `0xffff` after 110 entries, so `0x82`, 130, is a bound above that which cannot loop
/// on a table orb has misread.
const A_KEY_IN_THE_TABLE: usize = 6;
const KEYS_IN_THE_TABLE: usize = 0x82;
/// What the table's own end is.
const NO_MORE_KEYS: u16 = 0xffff;

/// What a key that is down reads as in that snapshot, which is what `GetKeyboardState` writes: the high
/// bit of the byte.
const DOWN: u8 = 0x80;
/// And what a key that is not reads as in one, which is what [`Game::press_into`] writes for every button
/// the word does not hold — see there for why they are written rather than left as the host's read left
/// them.
const UP: u8 = 0x00;

/// The pad's own packet number, which moves on every state orb hands over — see
/// [`Game::press_into_pad`].
static PACKETS: AtomicU32 = AtomicU32::new(1);

/// What the log's own `run:` line carries: the stage, the difficulty, the scene, the lives and the bombs.
type Said = (i32, i32, i32, i8, i8);

/// What that line last said, so that [`Game::read_state`] says it once each time any of it changes rather
/// than once a frame.
static SAID: std::sync::Mutex<Option<Said>> = std::sync::Mutex::new(None);

/// Whether [`A_REPLAY_IS_PLAYING`] was set the last time [`Game::read_state`] looked, so that a
/// playback starting and a playback ending are each one line.
static PLAYING_BACK: AtomicBool = AtomicBool::new(false);

/// **How many hits the run has taken, which is orb's count and not the game's** — see
/// [`Th06c::read_state`], which moves it, and [`resume::Numbers::deaths`], which is where it is put back
/// from. This exe keeps no such count of its own: th06 1.02h's `Player::Die` steps a field of the game
/// manager and nothing in this one's game manager is stepped by a hit.
static DEATHS: AtomicI32 = AtomicI32::new(0);
/// Whether the player's state was [`player::HIT`] on the frame before, which is what makes the count above
/// move once per hit rather than once per frame of one.
static WAS_HIT: AtomicBool = AtomicBool::new(false);
/// The stage word the last frame in a stage's own state read — see [`Th06c::read_state`]'s frame between
/// the stages.
static THE_WORD_PLAYED: AtomicI32 = AtomicI32::new(0);

/// **Which line of a conversation is up**, and negative for none — the panel's own field, behind a pointer
/// the game manager makes and nulls, so a null is no conversation either.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn dialogue_msg_idx() -> i32 {
    mem::read_committed::<usize>(at(GUI_IMPL))
        .filter(|panel| *panel != 0)
        .and_then(|panel| mem::read_committed::<i32>(panel + gui_impl::MSG_IDX))
        .unwrap_or(-1)
}

/// The low byte of the sum of a name's own bytes, which is what a record holds beside it at
/// [`CATK_NAME_SUM`] — the game's own is `movsx` into a 32-bit accumulator at `0x140014d00`, and the two
/// agree on the one byte either reads.
fn name_sum(name: &[u8]) -> u8 {
    name.iter().fold(0, |sum, byte| sum.wrapping_add(*byte))
}

/// The name inside a record's [`CATK_NAME`] span: the bytes up to the first `0`, or `None` where there is
/// no `0` in the span at all — which a name the game copied there always has and one the generator left
/// only has by chance.
fn name_in(span: &[u8]) -> Option<&[u8]> {
    let end = span.iter().position(|byte| *byte == 0)?;
    Some(&span[..end])
}

/// The record the game keeps for a card, and `None` for a number that is not one of the 64 or a slot
/// with no record in it at all.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn card_record(card: u32) -> Option<usize> {
    let card = usize::try_from(card).ok()?;
    if card >= CARDS {
        return None;
    }
    let record = at(CARD_HISTORY) + card * CARD_BYTES;
    (mem::read_committed::<u32>(record) == Some(CATK_MAGIC)).then_some(record)
}

/// Whether the game has named the card a record is for, asked the way the game asks it at `0x140014d10`:
/// the sum the record holds against the name it holds.
///
/// # Safety
/// `record` must be one of [`CARD_HISTORY`]'s.
unsafe fn named(record: usize) -> bool {
    let span = mem::read_committed_bytes(record + CATK_NAME, CATK_NAME_BYTES).unwrap_or_default();
    name_in(&span).is_some_and(|name| {
        !name.is_empty()
            && mem::read_committed::<u8>(record + CATK_NAME_SUM) == Some(name_sum(name))
    })
}

/// **The line one pass of a stage is held against another's** — see [`Game::reproduction`], and
/// `th06::Reproduction`, whose fields these are: the two games are one game to the frame, and a line that
/// reads the same is a line somebody can hold one game's log against the other's by.
struct Reproduction {
    /// The replay manager's own count of the stage's frames, or `-1` for a stage still being built.
    replay_frame: i32,
    /// The buttons the update acted on, in the ten bits the record keeps.
    input: u16,
    player: (f32, f32),
    /// The top and the height of the box the player is held inside.
    player_area: (f32, f32),
    /// How many numbers have come out of the generator since the stage seeded it.
    randoms: u32,
    /// The generator itself.
    seed: u16,
    /// Items in the air.
    items: u32,
    score: u32,
    extra_lives: i8,
    rank: i32,
    sub_rank: i32,
    /// **The phase of the shot the player is firing, and the phase the update before it had** — see
    /// [`player::SHOT_PHASE`]. **Here because what parted a pick-up of the Extra from its run was when the
    /// player's shots took an enemy down**, and the player's shots are in nothing else the line holds.
    shot_phase: (i32, i32),
    /// **Which of its four states the player is in** — see [`player::STATE`] — which decides whether a
    /// press of the shot starts a cycle at all: `0x14004ad0e` and `0x14004ad17` skip it in states 2 and 1.
    player_state: u8,
    /// **Which line of a conversation is up**, `-1` for none — [`gui_impl::MSG_IDX`], which `0x14004b34c`
    /// gates the shot's press on. **Here because a pick-up of the Extra parted inside the midboss's
    /// conversation**, the playback's shot cycle beginning on a frame the run's stayed idle on, both runs
    /// reading `in_dialogue=True` on every frame either side.
    line: i32,
}

impl std::fmt::Display for Reproduction {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            out,
            "replay_frame={} input={:#06x} player={:.2},{:.2} area={:.2}+{:.2} randoms={} \
             rng={:#06x} items={} score={} extras={} rank={} subrank={} shot={},{} state={} line={}",
            self.replay_frame,
            self.input,
            self.player.0,
            self.player.1,
            self.player_area.0,
            self.player_area.1,
            self.randoms,
            self.seed,
            self.items,
            self.score,
            self.extra_lives,
            self.rank,
            self.sub_rank,
            self.shot_phase.0,
            self.shot_phase.1,
            self.player_state,
            self.line,
        )
    }
}

/// **The ten bits of the word a run acts on that the game's own record keeps**, `and r10d,0x3ff` at
/// `0x14004d80f`: the four directions, the shot, the bomb, the focus and the three below them.
const RUN_INPUT_BITS: u16 = 0x3ff;

/// The stages a run of this game has, the Extra counted — see `sight::STAGES`, which is the same seven.
const STAGES: i32 = 7;
/// Nought for Easy up to four for the Extra — see [`DIFFICULTY`].
const DIFFICULTY_EXTRA: i32 = 4;
/// Two characters of two shots each, indexed `character * 2 + shot` everywhere the game indexes them.
const SHOT_COMBINATIONS: i32 = 4;

/// **The file the playback is reading**, off [`THE_REPLAY_BEING_PLAYED`] and cut at the NUL the copy
/// that wrote it left.
///
/// `0x40` bytes is a bound and not a reading: nothing in the exe says how long that buffer is, and both
/// of the paths written into it are this game's own — `data/demo/demo00.rpy` and `replay/th6_NN.rpy`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_replays_path() -> String {
    let bytes = mem::read_committed_bytes(at(THE_REPLAY_BEING_PLAYED), 0x40).unwrap_or_default();
    let path = &bytes[..bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())];
    String::from_utf8_lossy(path).into_owned()
}

pub struct Th06c;

impl Th06c {
    /// The buttons orb's word has and the key each is pressed with, in one place so that
    /// [`Game::button_bit`] and [`Game::press_into`] cannot disagree about one.
    const BUTTONS: &'static [(&'static str, u16, u8)] = &[
        ("shot", button::SHOT, key::SHOT),
        ("bomb", button::BOMB, key::BOMB),
        ("focus", button::FOCUS, key::FOCUS),
        ("menu", button::MENU, key::MENU),
        ("decide", button::DECIDE, key::DECIDE),
        ("up", button::UP, key::UP),
        ("down", button::DOWN, key::DOWN),
        ("left", button::LEFT, key::LEFT),
        ("right", button::RIGHT, key::RIGHT),
    ];
}

impl Game for Th06c {
    /// Kept as the distance from [`IMAGE_BASE`] — see [`at`].
    fn set_image_base(&self, base: usize) {
        SLIDE.store(base.wrapping_sub(IMAGE_BASE), Ordering::Relaxed);
    }

    /// **The DirectInput keyboard taken away, so that the keys orb writes are the keys a run is played
    /// with** — see [`DEVICE`], which has the measurement.
    ///
    /// Asked every frame rather than once, because the game makes that device during its own startup
    /// and remakes it whenever the window is activated again: a launch that nulled it at the attach
    /// would be a launch whose run took the device back.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn acquire_input(&self) -> bool {
        if orb_api::mem::read_committed::<usize>(at(DEVICE)) == Some(0) {
            return true;
        }
        unsafe { orb_api::mem::write::<usize>(at(DEVICE), 0) };
        true
    }

    /// **The keyboard read and the startup dialog**, which is as far as this game has been read: the read
    /// stood in front of by answering the name the game asks for rather than by patching code, and the
    /// dialog got past by clicking the controls a person would — see [`crate::game::Read::LookedUp`] and
    /// [`Dialog`].
    ///
    /// Every other hook is `None` because the address it would need is unread, and the two the seam
    /// takes outright are `None` for the same reason — a launch here has no frame of orb's own, which is
    /// what [`Hooks::update`] says.
    fn hooks(&self) -> Hooks {
        Hooks {
            // **Neither, and not for want of a frame loop: this game's two walks are handed over.**
            // Nothing of the game's calls either of them once the frame is orb's — that frame was the
            // only caller — so there is no prologue to be in the path of, and a patch could not reach
            // the draw side at all, its walk being inline in that frame. See [`walks`](Hooks::walks).
            update: None,
            draw: None,
            walks: Some(Walks {
                update: walks_the_calc_chain as *const () as usize,
                draw: walks_the_draw_chain as *const () as usize,
            }),
            // **Nothing, because orb is inside this game's frame now rather than outside it.** What that
            // hook was is one call per walk of the calc chain for a game whose frame orb had not been
            // read far enough into to replace; the walk is orb's own to call now, and the rules are
            // asked where every other game's are.
            // The one convention a 64-bit game has, and nothing patched for it to matter to: the walks are
            // handed over above.
            frame_entered: FrameEntered::InEcx,
            runs_its_frame: None,
            save_replay: None,
            stop_recording: None,
            // **The two moments a stage's numbers go in at**, which are th06 1.02h's two under other
            // addresses: the game manager's stage callback, entered with the manager in `rcx` the way
            // its chain calls every callback, and the stage's own build it calls once the seed is
            // copied — see [`GAME_MANAGER_ADDED`] and [`STAGE_REGISTER_CHAIN`].
            stage_begun: Some(Entered::InEcx(Patch {
                target: at(GAME_MANAGER_ADDED),
                prologue: GAME_MANAGER_ADDED_PROLOGUE,
            })),
            stage_building: Some(Building::Cdecl(Patch {
                target: at(STAGE_REGISTER_CHAIN),
                prologue: STAGE_REGISTER_CHAIN_PROLOGUE,
            })),
            // **None, and the bracket is up all the same**: the one read this game makes is inline in its
            // startup, so it is bracketed from the attach to the first settling of the mode instead — see
            // [`Game::reads_its_file_once_at_startup`] and [`score_file`].
            unlocks_read: None,
            ranking_read: None,
            card_records_read: None,
            create_window: None,
            init_device: None,
            // **The whole frame, which is what makes it orb's**: the loop at `0x14002c5b7` calls this
            // once round and orb's own loop stands in its place, calling every piece of it — see
            // [`THE_FRAME`], and `Game::prepare_frame`, `Game::end_drawing` and `Th06c::frame_calls`,
            // which are where the pieces that are not the two walks went.
            render: Some(Patch {
                target: at(THE_FRAME),
                prologue: THE_FRAME_PROLOGUE,
            }),
            input: Some(Read::LookedUp(Lookup {
                name: KEYBOARD_READ,
            })),
            joystick: None,
            tops_up_the_music: None,
            stops_the_music: None,
            pad_buttons: None,
            startup_dialog: Some(Dialog {
                clicks: &[dialog::A_640_BY_480_WINDOW, dialog::START],
            }),
            makes_its_window: Some(Lookup { name: WINDOW_MADE }),
            moves_its_window: Some(Lookup { name: WINDOW_MOVED }),
            shows_its_window: Some(Lookup { name: WINDOW_SHOWN }),
            places_its_window: Some(Lookup {
                name: WINDOW_PLACED,
            }),
            restyles_its_window: Some(Lookup {
                name: WINDOW_RESTYLED,
            }),
            reads_a_pad: Some(Lookup { name: PAD_READ }),
            makes_its_renderer: Some(Lookup {
                name: RENDERER_MADE,
            }),
            // **What the CRT linked into this exe reaches the OS through, all three.** `objdump -p` lists
            // `steam_api64.dll`, `KERNEL32.dll`, `USER32.dll` and `ole32.dll` and nothing of the universal
            // CRT's; of `KERNEL32.dll` it takes `GetProcessHeap` and no `HeapCreate` or `VirtualAlloc`,
            // `CreateThread`, and `CreateFileW` — see `th06c/asm.md`. What that decides is which entries orb
            // stands in front of: the build of md5 acadbc0c imported `malloc`, `_beginthreadex` and `fopen`
            // outright, and an install looking for those finds none of them here.
            takes_its_memory: Memory::ProcessHeapImport,
            makes_its_threads: Threads::CreateThread,
            opens_its_files: FileOpen::CreateFileW,
        }
    }

    /// **Yes**: the midstage table, the two seams a stage's numbers go in at, the snapshot a chapter
    /// is — the game's `.data` and the heap orb answers its `GetProcessHeap` with, see
    /// `Memory::ProcessHeapImport` —
    /// and the retry a death is offered are all here, and [`resume`] is the file a run is picked up
    /// from. And the questions those raise are drawn, through a canvas over this game's own swapchain —
    /// what a launch here still cannot do is answer one at a keyboard, see the module doc.
    fn rewinds(&self) -> bool {
        true
    }

    /// **A word's buttons put into the snapshot the game is about to read**, which is how a bot presses
    /// anything here: the game reads its keys once a tick and turns them into its own word itself, so
    /// what orb writes is the keys and not the word — see [`crate::runtime::keyboard_state`].
    ///
    /// Written into the snapshot the read was given rather than into the game's own key buffer at
    /// `0x14042893c`, because the translation between the two is the game's: a key written here reaches
    /// that buffer through the table at `0x1402bc0b0` the way the keyboard's own would. See
    /// `th06c/asm.md`.
    ///
    /// **And the keys the word does not hold are written up, not left alone**, which is what makes this
    /// the whole of what the game reads rather than an addition to it. Two things need that. A word held
    /// back has the decide taken out of it — see `runtime::held_back` — and a decide left down in the
    /// snapshot is a press the game acts on after all, which is orb's question up *and* the game's own item
    /// chosen. And a launch that plays itself is played by its word alone: a key somebody leans on at the
    /// keyboard would otherwise be added to every word the rules chose. `Th06c::presses_into_its_own_keys`
    /// has the measurement for what a union of two words does to a run.
    ///
    /// **Each button at its own virtual key and at the one this game's table holds for it**, which is what
    /// carries the focus: that key goes in at `VK_SHIFT`, and the table holds `0xa0` and `0xa1` at the shift
    /// scans and no `0x10` — see [`key::FOCUS_IN_THE_GAMES_TABLE`]. Both keys is what a real
    /// `GetKeyboardState` answers, and for the other eight buttons the two are one key written twice.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's read was given.
    unsafe fn press_into(&self, state: *mut u8, word: u16) {
        for (_, bit, key) in Self::BUTTONS {
            let held = match word & bit != 0 {
                true => DOWN,
                false => UP,
            };
            for key in [*key, the_key_the_games_table_holds(*key)] {
                if usize::from(key) < KEYS_IN_A_SNAPSHOT {
                    unsafe { state.add(usize::from(key)).write(held) };
                }
            }
        }
    }

    /// **The word this game's own keys in that snapshot say**, which is [`Game::press_into`] read the other
    /// way and over the same [`Th06c::BUTTONS`], so the two cannot disagree about which key a button is.
    ///
    /// The high bit and not the whole byte: what `GetKeyboardState` puts there is that bit for a key that
    /// is down and the low bit for one that has been toggled, and a caps lock left on is not a button.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's read was given.
    unsafe fn word_in(&self, state: *const u8) -> Option<u16> {
        let mut word = 0;
        for (_, bit, key) in Self::BUTTONS {
            if usize::from(*key) < KEYS_IN_A_SNAPSHOT
                && unsafe { state.add(usize::from(*key)).read() } & DOWN != 0
            {
                word |= bit;
            }
        }
        Some(word)
    }

    /// **A word's keys written straight into the buffer this game plays a run off**, which is what a word
    /// decided on the game's own frame needs: that frame is past the read `press_into` writes into, so a
    /// word left there is acted on a frame later — see `Game::presses_into_its_own_keys`, which has what
    /// a frame of that cost.
    ///
    /// **Through the game's own table from a virtual key to its index**, rather than nine scan codes
    /// written down here: what the buffer is indexed by is DirectInput's own, and the mapping is the one
    /// the game's read uses — see [`THE_TABLE_OF_KEYS`].
    ///
    /// **And the keys the word does not hold are written up, not left alone.** That buffer is filled by
    /// the game's own read as well — `0x14008e297` memsets the 256 bytes and the walk at `0x14008e2c0`
    /// puts orb's snapshot into them — and that read is made at its own rate rather than once a frame,
    /// so a word written on top of one already carrying the *previous* word leaves the run acting on the
    /// union of the two. Which is not a word the rules ever chose, and the direction decode at
    /// `0x14004ad29` makes the union worse than a coin toss: it tests `0x10` before `0x20` and the sign
    /// bit last, so up beats down and right beats left whenever both are down. Measured — a stage 1 hit
    /// whose dump says `the word the run acts on is 0x81f1`, all four directions at once, where the
    /// rules had chosen `0x050`: the player went up and right out of a move chosen up and left, and six
    /// hits of one run were that same reversal.
    /// **The two times this game keeps put back to nought**, which makes its next read of the keyboard and
    /// of the pad real reads — see [`WHEN_THE_KEYBOARD_WAS_READ`] and [`WHEN_THE_PAD_WAS_READ`] for the
    /// instructions that skip a read and the pick-ups that cost, and `Game::forgets_when_it_read` for what a
    /// playback wants it for.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the exe's base already told.
    unsafe fn forgets_when_it_read(&self) {
        unsafe { mem::write::<u32>(at(WHEN_THE_KEYBOARD_WAS_READ), NO_READ_YET) };
        unsafe { mem::write::<u32>(at(WHEN_THE_PAD_WAS_READ), NO_READ_YET) };
    }

    unsafe fn presses_into_its_own_keys(&self, word: u16) -> bool {
        let mut pressed = false;
        for (_, bit, key) in Self::BUTTONS {
            let index = unsafe { the_index_of_the_key(the_key_the_games_table_holds(*key)) };
            let Some(index) = index else {
                continue;
            };
            let down = word & bit != 0;
            let held = match down {
                true => DOWN_IN_ITS_OWN_KEYS,
                false => UP_IN_ITS_OWN_KEYS,
            };
            unsafe { mem::write::<u8>(at(THE_KEYS_THE_GAME_READS) + index, held) };
            pressed |= down;
        }
        pressed
    }

    /// **A word's buttons put into the pad state orb answers this game's own read with**, which is how a
    /// bot presses anything that a run acts on — see `Hooks::reads_a_pad` and [`pad`].
    ///
    /// **The stick as well as the d-pad**, both at once: which of the two a game reads its directions
    /// from is the game's business, and a pad held to the corner says the same thing either way. Full
    /// deflection, since a stick half over is a game's own dead zone away from meaning nothing.
    ///
    /// **On the buttons the game's own `th06.cfg` binds**, read where its read reads them — see
    /// [`the_buttons_pressed_through`] and [`supervisor::CONFIG_PAD_SHOT`].
    ///
    /// # Safety
    /// `state` must be the sixteen bytes of an `XINPUT_STATE`.
    unsafe fn press_into_pad(&self, state: *mut u8, word: u16) {
        let buttons = the_buttons_pressed_through(
            at(SUPERVISOR + supervisor::CONFIG_PAD_SHOT),
            at(SUPERVISOR + supervisor::CONFIG_DECIDE_SWAPPED),
            word,
        );
        let across = match (word & button::RIGHT != 0, word & button::LEFT != 0) {
            (true, false) => i16::MAX,
            (false, true) => i16::MIN,
            _ => 0,
        };
        // **Up is positive on a stick and down is negative**, which is the other way about from a
        // screen's own rows.
        let down = match (word & button::UP != 0, word & button::DOWN != 0) {
            (true, false) => i16::MAX,
            (false, true) => i16::MIN,
            _ => 0,
        };
        unsafe {
            // `XINPUT_STATE`: the packet number, then the gamepad's buttons, its two triggers and its
            // four thumb axes. The packet number moves every frame, a game that watched it for a change
            // being one that ignores a state it has already seen.
            state
                .cast::<u32>()
                .write(PACKETS.fetch_add(1, Ordering::Relaxed));
            state.add(4).cast::<u16>().write(buttons);
            state.add(6).write(0);
            state.add(7).write(0);
            state.add(8).cast::<i16>().write(across);
            state.add(10).cast::<i16>().write(down);
            state.add(12).cast::<i16>().write(0);
            state.add(14).cast::<i16>().write(0);
        }
    }

    /// The bits a recipe's button names are, which is what makes `press: [decide]` a decide here — see
    /// [`Th06c::BUTTONS`].
    fn button_bit(&self, name: &str) -> Option<u16> {
        Self::BUTTONS
            .iter()
            .find(|(button, _, _)| *button == name)
            .map(|(_, bit, _)| *bit)
    }

    /// **Which item of the front end the cursor is on**, and `None` while a stage runs — see
    /// [`front_end::CURSOR`], which is one field for whatever list is up.
    unsafe fn front_end_cursor(&self) -> Option<i32> {
        match unsafe { sight::in_a_stage() } {
            true => None,
            false => mem::read_committed::<i32>(at(FRONT_END + front_end::CURSOR)),
        }
    }

    /// **And which screen that is**, by the front end's own number for it — see
    /// [`front_end::STATE`]. `None` while a stage runs, which is what makes a
    /// `until: { screen: N, cursor: M }` a list of the front end's and not a stage.
    unsafe fn front_end_screen(&self) -> Option<i32> {
        match unsafe { sight::in_a_stage() } {
            true => None,
            false => mem::read_committed::<i32>(at(FRONT_END + front_end::STATE)),
        }
    }

    /// One frame of a stage as this game holds it — see [`sight`], which is where every address it
    /// reads is written down.
    unsafe fn what_a_bot_sees(&self, into: &mut crate::game::Frame) -> bool {
        unsafe { sight::what_a_bot_sees(into) }
    }

    unsafe fn say_what_was_near(&self) {
        unsafe { sight::say_what_was_near() };
    }

    fn named(&self) -> &'static str {
        "th06c"
    }

    /// Which of orb's own bits the front end's two answers are — see [`button`], and
    /// `th06c/asm.md` for what of this game's own front end has been read, which is nothing.
    fn menu_decide(&self) -> u16 {
        button::DECIDE | button::SHOT
    }

    fn menu_cancel(&self) -> u16 {
        button::BOMB
    }

    /// The bits a run is written down with, which is every button orb has for this game: nothing of its
    /// own record has been read, so nothing here is left out of one.
    fn run_input(&self) -> u16 {
        let mut word = 0;
        for (_, bit, _) in Self::BUTTONS {
            word |= bit;
        }
        word
    }

    /// **The run's own numbers, out of the supervisor, the game manager, the player and the enemies.**
    ///
    /// `scene` is the supervisor's state and not the front end's, which is what the `run:` line said
    /// before this game's run was read: the front end's number is [`Game::front_end_screen`]'s and stays
    /// what it was through a stage, where the supervisor's says the stage itself — see
    /// [`supervisor::CUR_STATE`]. `playing` is still [`sight::in_a_stage`]'s, which has the byte that
    /// holds everything in it and the reasoning beside it.
    ///
    /// **The deaths are orb's own count**, this exe keeping none: `Player::Die`'s count of th06 1.02h is
    /// no field of this game manager. So the count moves here, on the frame the player's state becomes
    /// [`player::HIT`], and [`Th06c::where_a_counted_death_stands`] says what became of it — a count
    /// that started at the hit so that the deathbomb window can be watched the way th06's is.
    unsafe fn read_state(&self) -> State {
        let jobs = unsafe { the_jobs() };
        say_the_jobs_when_they_change(&jobs);
        say_the_result_screens_state_when_it_changes(&jobs);
        let scene = unsafe { in_the_image::<i32>(SUPERVISOR + supervisor::CUR_STATE) };
        let wanted = unsafe { in_the_image::<i32>(SUPERVISOR + supervisor::LAST_STATE) };
        // **The game's own number less one**, this game counting stages from one where `State::stage`
        // is counted from nought the way orb counts them everywhere — see `RunStart::stage`.
        //
        // **And no stage between a stage's end and the next one's build**: the end asks for the rebuild
        // (0x140028410's case 0xb writing 3 over the state) and the supervisor's next update builds it, so the
        // frame between is the old stage over and the new one not there. The word itself says nothing of it —
        // nothing writes it nought, and the callback raises it by one — so it is the state asked for and not
        // yet acted on that says it, which is what `runtime`'s wall at a stage's end reads as no stage.
        //
        // **And only while the word is still the stage played**: the stage callback runs inside that same
        // state, the supervisor putting it back only once the build is done, and a stage's start read there
        // is of the stage it has just raised the word to.
        let word = unsafe { in_the_image::<i32>(STAGE) };
        if scene == STATE_GAMEMANAGER {
            THE_WORD_PLAYED.store(word, Ordering::Relaxed);
        }
        let between_the_stages = scene == STATE_GAMEMANAGER_REINIT
            && wanted == STATE_GAMEMANAGER
            && word == THE_WORD_PLAYED.load(Ordering::Relaxed);
        let stage = match between_the_stages {
            true => -1,
            false => word - 1,
        };
        let playing = unsafe { sight::in_a_stage() };
        // **Wider than `playing`, the way th06's is**: a frame the game holds everything on is a frame
        // of a run, and what reads this is whether a `play_until` is over — see [`sight::in_a_run`].
        let in_run = unsafe { sight::in_a_run() };
        // **Which of the two playbacks the game is holding**, and whether it is the one nobody asked for.
        let playback = unsafe { sight::a_replay_is_playing() };
        let demo = playback && unsafe { sight::the_playback_is_the_demo() };
        // **And a run of somebody's own**, which is [`sight::in_a_run`] without the replay somebody is
        // watching: that is a stage being played and not a run orb acts on — no death to offer a retry for,
        // no chapter to write down and no file of its own.
        let in_game = in_run && !playback;
        let boss = unsafe { sight::the_boss_the_fights_own_slot() };
        let difficulty = unsafe { in_the_image::<i32>(DIFFICULTY) };
        let player_state = unsafe { in_the_image::<u8>(PLAYER + player::STATE) };
        // **Counted on the frame the state becomes the hit's and on no other**, a state being held for
        // frames: the frame before is what says this is the edge. Put back to nought where the run is
        // over, so that a launch's second run starts its count where the game starts its own.
        let hit = player_state == player::HIT;
        // **Counted for a run of somebody's own and not for a playback**, which is the narrower reading's
        // whole point here: the hits of a replay are the hits of the run it was recorded from.
        if in_game && hit && !WAS_HIT.swap(hit, Ordering::Relaxed) {
            DEATHS.fetch_add(1, Ordering::Relaxed);
        } else if !in_game {
            WAS_HIT.store(false, Ordering::Relaxed);
            if scene == STATE_MAINMENU {
                DEATHS.store(0, Ordering::Relaxed);
            }
        } else {
            WAS_HIT.store(hit, Ordering::Relaxed);
        }
        let spellcard_active =
            unsafe { in_the_image::<i32>(ENEMY_MANAGER + enemy_manager::SPELLCARD_IS_ACTIVE) } != 0;
        // **What run this is, said once each time any of it changes.** Which difficulty a launch played
        // is otherwise in no line of the log and on no part of the screen the panel draws, and a recipe
        // that walks this game's front end by taps cannot say which item it landed on — so the run orb
        // played is a thing only this says.
        //
        // **The lives and the bombs are in what makes a line and not only in what a line carries**,
        // because a death is the thing a reader is looking for and it happens inside a stage: keyed on
        // the stage alone, a launch that lost three lives on stage 5 said `5 live` at the stage's start
        // and nothing after it, and how many deaths the run had could only be read off the pictures.
        let lives = unsafe { in_the_image::<i8>(LIVES) };
        let bombs = unsafe { in_the_image::<i8>(BOMBS) };
        // **Which of the two playbacks the game is holding**, and the file it is playing said the frame
        // one starts. Said on its own line rather than keyed into the one below, because what the line
        // below reads of a playback is a run's own numbers and nothing else: 34,200 frames of the demo
        // went by with every line of one launch's log saying `stage 4 on difficulty 1, scene 11`.
        //
        // **The byte itself and not the byte with a stage's scene beside it**, which was tried and says
        // nothing about the demo a launch actually meets: a demo started off the logo screen plays
        // through *that* screen's own state — `scene 1` for the whole of it, with the run's own lives
        // and bombs moving underneath — so a reading that waited for `scene 11` said nothing at all.
        if PLAYING_BACK.swap(playback, Ordering::Relaxed) != playback {
            match playback {
                true => crate::log!(
                    "run: {} is playing back {}",
                    if demo { "the demo" } else { "a replay" },
                    unsafe { the_replays_path() },
                ),
                false => crate::log!("run: the playback is over"),
            }
        }
        let said = (
            if playing { stage + 1 } else { 0 },
            difficulty,
            scene,
            lives,
            bombs,
        );
        let mut last = SAID
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if last.replace(said) != Some(said) {
            crate::log!(
                "run: stage {} on difficulty {difficulty}, scene {scene}, {lives} live and \
                 {bombs} bomb(s)",
                said.0,
            );
        }
        drop(last);
        State {
            scene,
            wanted,
            playing,
            in_run,
            // **Neither playback**, where `in_run` above is only not the demo: a replay somebody asked to
            // watch is a stage being played through and no run for orb to act on, which is the reading th07
            // has of its own and the one a judging pass over this game needs — see [`sight::in_a_run`].
            in_game,
            // **The job an ending runs as being in the chain**, or the supervisor's own state for one:
            // see [`ENDING_UPDATE`], and `runtime`'s own skip, which runs the chain on until this goes
            // out.
            in_ending: scene == STATE_ENDING || jobs.hold(ENDING_UPDATE),
            // **And no script of it read**, which is what th06 1.02h's skip stops at the staff roll by:
            // this game's is unread, so the skip runs the roll out with the rest of it.
            ending_script: None,
            demo,
            replay: playback && !demo,
            practice: unsafe { in_the_image::<u8>(GAME_MANAGER + game_manager::IS_IN_PRACTICE) }
                != 0,
            // The two menus a run stops for, the way th06 1.02h's are read: the byte between them says a
            // run is *running*.
            paused: unsafe { in_the_image::<u8>(GAME_MANAGER + game_manager::IS_IN_GAME_MENU) }
                != 0
                || unsafe { in_the_image::<u8>(GAME_MANAGER + game_manager::IS_IN_RETRY_MENU) }
                    != 0,
            unsettled: hit || player_state == player::SPAWNING,
            bombing: unsafe { in_the_image::<u32>(PLAYER + player::BOMB_IN_USE) } != 0,
            in_dialogue: (unsafe { dialogue_msg_idx() }) >= 0,
            stage,
            difficulty,
            stage_frames: unsafe { in_the_image::<u32>(GAME_MANAGER + game_manager::GAME_FRAMES) },
            script_frames: unsafe {
                in_the_image::<i32>(ENEMY_MANAGER + enemy_manager::TIMELINE_TIME)
            },
            random_seed: unsafe { in_the_image::<u16>(RNG_SEED) },
            deaths: DEATHS.load(Ordering::Relaxed),
            lives,
            bombs,
            power: unsafe { in_the_image::<u16>(POWER) },
            enemy_count: unsafe { in_the_image::<i32>(ENEMY_MANAGER + enemy_manager::LIVE) },
            bullet_count: unsafe { in_the_image::<i32>(BULLET_MANAGER + bullet_manager::LIVE) },
            laser_count: (0..laser::SLOTS)
                .filter(|slot| {
                    (unsafe { in_the_image::<u8>(LASERS + slot * laser::BYTES + laser::IN_USE) })
                        != 0
                })
                .count() as i32,
            // **The panel's own byte and the slot both**: the byte is what th06 1.02h is read by and is
            // set as the script names a boss, where the slot is what [`sight`] steers by. Either says a
            // fight is on.
            boss_present: boss.is_some() || unsafe { in_the_image::<u8>(BOSS_PRESENT) } != 0,
            boss_life: boss.map(|body| unsafe { mem::read::<i32>(body + enemy::LIFE) }),
            boss_attack_frames: boss.map(|body| unsafe { mem::read::<i32>(body + enemy::TIMER) }),
            spellcard: spellcard_active.then(|| unsafe { in_the_image::<u32>(CURRENT_CARD) }),
        }
    }

    /// **This game counts the hit**, the way th06 1.02h does — orb's count moves as the state becomes
    /// [`player::HIT`] — so what became of it is read off the two fields the game decides it with: the
    /// state, and the frames a bomb is still taken in at [`player::A_BOMB_STILL_TAKES_IT_BACK`]. A state
    /// no longer the hit's with the window still open is a death a bomb took back.
    unsafe fn where_a_counted_death_stands(&self) -> CountedDeath {
        let state = mem::read_committed::<u8>(at(PLAYER + player::STATE)).unwrap_or(player::NORMAL);
        let window = mem::read_committed::<i32>(at(PLAYER + player::A_BOMB_STILL_TAKES_IT_BACK))
            .unwrap_or(0);
        match state == player::HIT {
            true if window > 0 => CountedDeath::Open,
            true => CountedDeath::Taken,
            false => CountedDeath::Bombed,
        }
    }

    /// **One more into the game manager's own count of retries**, the byte the result screen draws and
    /// the pause menu's 「最初からやり直す」 steps up — see [`game_manager::NUM_RETRIES`].
    unsafe fn count_a_retry(&self) {
        let at_ = at(GAME_MANAGER + game_manager::NUM_RETRIES);
        let standing = mem::read_committed::<u8>(at_).unwrap_or(0);
        unsafe { mem::write::<u8>(at_, standing.saturating_add(1)) };
    }

    /// The same index `State::in_dialogue` is read from: this game, like th06 1.02h, has no line of its own
    /// for handing the screen over, so a line being up and a dialogue running are one thing.
    unsafe fn a_line_of_dialogue_is_shown(&self) -> bool {
        (unsafe { dialogue_msg_idx() }) >= 0
    }

    /// **The window the swapchain presents into**, which is this game's own — see
    /// `crate::dxgi::the_window`, and nought before a swapchain has been taken.
    ///
    /// **Asked of the swapchain and not of the game**, because the game makes its window through a pointer
    /// it looked up and where it keeps the handle that call answered is not in `th06c/asm.md`. The
    /// swapchain was built for that window and says so, so there is a reading here without one.
    ///
    /// **What it is for is orb's own menus.** `runtime::on_update` polls the keyboard against this handle
    /// and `input::Keyboard::poll` reads nothing for a null one, so a question of orb's over this game went
    /// up on a press and could never be answered: the cursor did not move and the decide did nothing.
    unsafe fn window(&self) -> orb_api::Hwnd {
        unsafe { crate::dxgi::the_window() }
    }

    /// Unread: what this game draws through is a swapchain it built, and nothing of orb's is in front of
    /// it — see `th06c/asm.md`, where the device's own creation is the next reading.
    unsafe fn d3d_device(&self) -> orb_api::Device {
        orb_api::Device(0)
    }

    /// **Yes: every frame is laid out in the 640x480 its swapchain was asked for**, whatever its window
    /// is. A launch whose window was held at a `320x240` client and whose buffers were made that size
    /// again showed the top-left of the title and nothing else; left at `640x480`, the whole of it.
    fn draws_at_the_size_it_asked_for(&self) -> bool {
        true
    }

    /// **No, and what follows from that is the scene and not the overlay.** What this game draws through
    /// is a D3D11 renderer of its own, which orb has no `IDirect3DDevice8` of — so the frame loop opens no
    /// scene of orb's around the draw chain, sets no render state either side of it, and the picture a
    /// `screenshot` step takes has nowhere to be read out of.
    ///
    /// **The overlay is drawn all the same**, through a canvas over this game's own swapchain rather than
    /// on a device — see `crate::canvas` and `Overlay::over_a_swapchain`. So the questions orb puts over
    /// the game are on the screen here; what is not is the mark over the lives, which needs the row the
    /// game repaints and [`Th06c::draws_lives_row`] answers no to for its own reasons.
    ///
    /// The frame is still orb's: see [`Th06c::hooks`], whose `render` is the whole of this game's own.
    fn draws_through_its_d3d_device(&self) -> bool {
        false
    }

    /// Unread: what this game streams its music through is an object of its own behind the pointer at
    /// [`SUPERVISOR`], and nothing of a DirectSound stream is in it.
    fn music(&self) -> Option<crate::audio::Music> {
        None
    }

    /// **Nothing to wait for**: the line above answers `None` on every frame, and the block the track is
    /// played out of is in [`Th06c::live_handles`], which a restore leaves where it finds it.
    fn a_snapshot_waits_for_its_music(&self) -> bool {
        false
    }

    /// **Which track is playing**, out of the static the frame's own catch-up reads — see
    /// [`THE_TRACK_PLAYING`] — and `None` for the `-1` it holds between tracks. What it tells apart is a
    /// stage's own song from its boss's, which is what tells a midboss from the boss.
    fn music_identity(&self) -> Option<u32> {
        mem::read_committed::<i32>(at(THE_TRACK_PLAYING))
            .filter(|track| *track >= 0)
            .map(|track| track as u32)
    }

    fn audio_thread(&self) -> Option<u32> {
        None
    }

    unsafe fn stop_music(&self) {}

    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    /// **The block of statics the music is played out of**, which a restore leaves alone so that the track
    /// goes on where it is: which track, whether it plays, whether it loops, the loop's bounds and the extra
    /// walks — [`THE_TRACK_PLAYING`] to [`THE_EXTRA_WALKS`], one `0x40`-byte block at `0x1403eea30`.
    fn audio_state(&self) -> Vec<Range<usize>> {
        // One block, collected rather than written as a `vec!` of one range: a `vec![a..b]` reads as a range
        // of numbers to a lint and to a reader alike.
        std::iter::once(at(THE_AUDIO_STATICS.start)..at(THE_AUDIO_STATICS.end)).collect()
    }

    /// **The renderer's COM objects and the pointer to the renderer itself**, which a restore must not
    /// rewind: what they name is D3D11's and DXGI's, made in a heap no snapshot copies, and a pointer put
    /// back names an object released since — see [`THE_RENDERERS_OBJECTS`], [`THE_RENDERER`] and the table
    /// of windows whose views are remade inside a chapter, [`THE_WINDOWS`]. **And the sound library's own
    /// block**, whose locks and work queues say where the library's own threads are,
    /// which no snapshot rewinds with them — see [`THE_SOUND_LIBRARYS_BLOCK`] for what putting it back cost.
    ///
    /// **And not the textures the game's own sprites hold**, whose slots have not been read: a chapter is
    /// inside one stage and the stage's own sprites are loaded as it begins, so nothing between two
    /// chapters of it releases one. **Which the real cases have now measured**: the sixteen put chapters
    /// back inside every stage from the first to the Extra — a death's chapter again, a chapter further
    /// back, a stage from its start, a run picked up mid-stage and an attempt watched — and a pointer put
    /// back to a texture released since would have faulted on the next draw; none did, over every session
    /// from real6 on. A release inside a stage, should a later build make one, is what would move those
    /// slots into this list.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        let mut live = vec![
            at(THE_RENDERERS_OBJECTS.start)..at(THE_RENDERERS_OBJECTS.end),
            at(THE_RENDERER)..at(THE_RENDERER) + size_of::<usize>(),
            at(THE_WINDOWS.start)..at(THE_WINDOWS.end),
        ];
        // And the renderer itself, a block off the heap the snapshot otherwise copies whole — see
        // [`THE_RENDERERS_BLOCK`] for what putting it back cost. Nothing where the game has not made it yet.
        if let Some(renderer) = mem::read_committed::<usize>(at(THE_RENDERER)).filter(|it| *it != 0)
        {
            live.push(renderer..renderer + THE_RENDERERS_BLOCK);
        }
        live.push(at(THE_SOUND_LIBRARYS_BLOCK.start)..at(THE_SOUND_LIBRARYS_BLOCK.end));
        // And the CRT's own locks, which this build keeps in `.data` — see [`THE_CRTS_LOCKS`].
        live.extend(
            THE_CRTS_LOCKS
                .iter()
                .map(|locks| at(locks.start)..at(locks.end)),
        );
        live
    }

    /// Unread, and what asks is the overlay and the panel: neither is drawn here.
    fn play_area(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        None
    }

    unsafe fn repaint_lives_row(&self) {}

    unsafe fn draws_lives_row(&self) -> bool {
        false
    }

    fn lives_row(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Unread: this game sizes its own window and orb does not rewrite the ask.
    fn content_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn windowed(&self) -> bool {
        false
    }

    unsafe fn force_windowed(&self) {}

    unsafe fn set_play_viewport(&self, _device: orb_api::Device) {}

    fn chain(&self) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    unsafe fn take_sent_keys(&self) -> bool {
        false
    }

    /// **The viewport and the matrix the frame put in place before anything else of it ran**, which is
    /// where orb's own loop has this call too — `0x14002572e`–`0x1400592a8`, in that order.
    ///
    /// **Before the calc chain and not after it**, which is the frame's own order and the reason the loop
    /// calls this where it does: the viewport this sets covers the whole output and what narrows it to the
    /// playfield is the update, so a frame that set it afterwards would draw the field's contents outside
    /// the field.
    ///
    /// The device is nothing here: what this game draws through is a D3D11 renderer of its own and orb
    /// has no device of it — see [`Game::draws_through_its_d3d_device`].
    unsafe fn prepare_frame(&self, _device: orb_api::Device) {
        WALKS_THIS_FRAME.store(0, Ordering::Relaxed);
        unsafe { crate::dxgi::starts_a_frame() };
        unsafe {
            mem::write::<u64>(at(WHAT_THE_VIEWPORT_WAS_ASKED_FOR), 0);
            mem::write::<u32>(at(THE_VIEWPORTS_WIDTH), THE_OUTPUT.0);
            mem::write::<u32>(at(THE_VIEWPORTS_HEIGHT), THE_OUTPUT.1);
        }
        let sets: unsafe extern "win64" fn() =
            unsafe { std::mem::transmute(sets_the_frames_view()) };
        unsafe { sets() };
    }

    /// Nothing: this game's frame does nothing of its own between the sound and the draw chain.
    unsafe fn begin_drawing(&self, _device: orb_api::Device) {}

    /// **What the frame did after its draw chain**: the `-1` into the renderer's own field at
    /// `0x140025aaa`, and then the two calls its own timing begins with — see
    /// [`FINISHES_THE_PICTURE`], which is the one that puts the frame on the screen.
    ///
    /// **And not the wait that timing ends with, which is the one piece of the frame orb does not
    /// call.** `0x140025c00` onwards works out how far off its own deadline the frame is, sleeps through
    /// the pointer at `0x140bc2a90` for as much of it as it can, and then spins on the clock at
    /// `0x140bc2a80` until the deadline itself — and the frame's timing is `frame::Pacing`'s now. Two
    /// pacings is one too many and this is the one that goes: measured, it was `14.7ms` of a `16.6ms`
    /// frame, so a launch at `--pace=2` ran at sixty a second like every other.
    unsafe fn end_drawing(&self, _device: orb_api::Device) {
        if let Some(renderer) =
            mem::read_committed::<usize>(at(THE_RENDERER)).filter(|renderer| *renderer != 0)
        {
            unsafe { mem::write::<i32>(renderer + WHAT_THE_FRAME_LEAVES_IN_IT, -1) };
        }
        let rate: unsafe extern "win64" fn(usize) =
            unsafe { std::mem::transmute(puts_the_rate_together()) };
        unsafe { rate(at(WHAT_IT_PUTS_IT_ON)) };
        let finish: unsafe extern "win64" fn() =
            unsafe { std::mem::transmute(finishes_the_picture()) };
        unsafe { finish() };
        // **And what the wait does once it has waited**, in its order — see [`CAPTURES_THE_FIELD`].
        if let Some(renderer) =
            mem::read_committed::<usize>(at(THE_RENDERER)).filter(|renderer| *renderer != 0)
        {
            let asked = renderer + THE_CAPTURE_ASKED_FOR;
            let request = mem::read_committed::<i32>(asked).unwrap_or(NO_CAPTURE_ASKED_FOR);
            if request >= 0 {
                let word = |at: usize| mem::read_committed::<i32>(asked + at).unwrap_or(0);
                let capture: unsafe extern "win64" fn(usize, i32, i32, i32, i32, i32) =
                    unsafe { std::mem::transmute(captures_the_field()) };
                unsafe {
                    capture(
                        renderer,
                        request,
                        word(0x4),
                        word(0x8),
                        word(0xc),
                        word(0x10),
                    )
                };
                unsafe { mem::write::<i32>(asked, NO_CAPTURE_ASKED_FOR) };
            }
        }
        if let Some(left) =
            mem::read_committed::<i32>(at(THE_WAITS_OWN_COUNT)).filter(|left| *left != 0)
        {
            unsafe { mem::write::<i32>(at(THE_WAITS_OWN_COUNT), left - 1) };
        }
    }

    /// **Both of them orb's own, for a game whose frame has neither as a function to call.** The sound's
    /// catch-up is inline in the frame orb replaced and the present is inside one of the draw chain's
    /// jobs — see [`tops_up_the_sound`] and [`presents`], which is also where the game's own present is
    /// taken over.
    fn frame_calls(&self) -> FrameCalls {
        FrameCalls {
            play_sounds: Call {
                function: tops_up_the_sound as *const () as usize,
                this: 0,
            },
            present: Call {
                function: presents as *const () as usize,
                this: 0,
            },
            orbs_own: true,
        }
    }

    /// Unread: which of this game's replay list's writes starts a playback at a stage has not been read,
    /// and nothing of orb's watches a replay of this game's own.
    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    /// The state and the frames left under it, both, and only from the two states the player can be hit
    /// in or is already invulnerable in — th06 1.02h's own reasoning, this being its player: spawning and
    /// dying are left alone, since nothing can hit the player in either and writing over a spawn skips
    /// what fades the player in.
    unsafe fn make_invulnerable(&self) {
        let state = at(PLAYER + player::STATE);
        let current = mem::read_committed::<u8>(state).unwrap_or(player::NORMAL);
        if current == player::NORMAL || current == player::INVULNERABLE {
            unsafe {
                mem::write::<u8>(state, player::INVULNERABLE);
                mem::write::<i32>(
                    at(PLAYER + player::UNTOUCHABLE),
                    player::INVULNERABLE_FRAMES,
                );
            }
        }
    }

    /// Whether the stage on the screen is a playback — see [`sight::a_replay_is_playing`], and
    /// [`Self::read_state`] for which of the two kinds it is.
    unsafe fn replaying(&self) -> bool {
        unsafe { sight::a_replay_is_playing() }
    }

    /// **What the entry on the screen a run ends at holds** — see [`result_screen::NAME`].
    ///
    /// **Only while that entry is the state the screen is in**, so that the ranking the *Score* item opens
    /// answers nothing: it is the same object with no run behind it.
    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        let screen = unsafe { the_entry_a_name_is_typed_into() }?;
        crate::game::the_name_held_at(screen + result_screen::NAME, result_screen::NAME_BYTES)
    }

    /// Whether that entry is up at all — see [`the_entry_a_name_is_typed_into`].
    unsafe fn typing_a_name(&self) -> bool {
        unsafe { the_entry_a_name_is_typed_into() }.is_some()
    }

    /// **Into the field that entry's own keys write**, with its count of characters typed at the full eight
    /// and its cursor left on the item that ends it — which is the state the game itself is in once somebody
    /// has typed eight, so the press after it leaves the screen the way theirs would.
    ///
    /// **And it is the rules' own held shot that makes this worth having here.** The entry comes up while the
    /// game manager's state is still a stage's, so the frame it opens on is a frame the rules are being asked
    /// for a word on — and the word they hold is the shot, which writes whichever character the cursor starts
    /// on, eight times over. Every ranking a clear of this game registered was `AAAAAAAA`, read back out of
    /// the replay the same run wrote.
    unsafe fn types_its_name(&self, name: &str) -> bool {
        let Some(screen) = (unsafe { the_entry_a_name_is_typed_into() }) else {
            return false;
        };
        if !crate::game::the_name_typed_into(
            screen + result_screen::NAME,
            result_screen::NAME_BYTES,
            name,
        ) {
            return false;
        }
        unsafe {
            mem::write::<i32>(
                screen + result_screen::TYPED,
                result_screen::NAME_BYTES as i32,
            );
            mem::write::<i32>(screen + result_screen::CURSOR, result_screen::THE_END_ITEM);
        }
        true
    }

    /// **What the title's Score item writes**, and the frames it waits already spent: the front end's
    /// state to the one whose case asks the supervisor for the ranking, with the count it asks past —
    /// see [`front_end::SCORE`]. The state and the cursor before it are kept so that they can go back.
    unsafe fn show_ranking(&self) {
        let state = mem::read_committed::<i32>(at(FRONT_END + front_end::STATE)).unwrap_or(0);
        FRONT_END_BEFORE_THE_RANKING.store(state as usize + 1, Ordering::Relaxed);
        let cursor = mem::read_committed::<i32>(at(FRONT_END + front_end::CURSOR)).unwrap_or(0);
        CURSOR_BEFORE_THE_RANKING.store(cursor as usize + 1, Ordering::Relaxed);
        unsafe {
            mem::write::<i32>(at(FRONT_END + front_end::STATE), front_end::SCORE);
            mem::write::<i32>(
                at(FRONT_END + front_end::SCREEN_FRAMES),
                front_end::SCORE_ASKS_AFTER,
            );
        }
    }

    /// The scene being the ranking's *and* the screen past its own build — see [`RANKING_SHOWING`].
    unsafe fn showing_ranking(&self) -> bool {
        if mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE)) != Some(STATE_SCORE) {
            return false;
        }
        let Some(screen) = (unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }) else {
            return false;
        };
        mem::read_committed::<i32>(screen + result_screen::STATE)
            .is_some_and(|state| RANKING_SHOWING.contains(&state))
    }

    unsafe fn ranking_scene(&self) -> bool {
        mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE)) == Some(STATE_SCORE)
    }

    /// The front end's cursor back where it was, once the front end is up again — see
    /// `Game::restore_menu_cursor` for why afterwards.
    unsafe fn restore_menu_cursor(&self) {
        if let Some(cursor) = CURSOR_BEFORE_THE_RANKING
            .swap(0, Ordering::Relaxed)
            .checked_sub(1)
        {
            unsafe { mem::write::<i32>(at(FRONT_END + front_end::CURSOR), cursor as i32) };
        }
    }

    unsafe fn ranking_state(&self) -> String {
        let screen = unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }.unwrap_or(0);
        format!(
            "cur={} wanted={} screen={screen:#x} state={}",
            mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE)).unwrap_or(-1),
            mem::read_committed::<i32>(at(SUPERVISOR + supervisor::LAST_STATE)).unwrap_or(-1),
            match screen {
                0 => -1,
                screen => mem::read_committed::<i32>(screen + result_screen::STATE).unwrap_or(-1),
            },
        )
    }

    /// The screen's own way out written over the ranking, where one is up — th06's reasoning, this being
    /// its screen reached two ways: written unconditionally it sent a finished run's own screen away.
    /// And the front end's state back to the title, the way th06's is put back, before it is the thing
    /// acting on orb's request again.
    unsafe fn leave_ranking(&self) {
        if unsafe { self.showing_ranking() }
            && let Some(screen) = unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }
        {
            unsafe { mem::write::<i32>(screen + result_screen::STATE, result_screen::EXIT) };
        }
        if FRONT_END_BEFORE_THE_RANKING.swap(0, Ordering::Relaxed) != 0 {
            unsafe { mem::write::<i32>(at(FRONT_END + front_end::STATE), front_end::TITLE) };
        }
    }

    /// **One more attempt at the card that is up, in the game's own record of it** — th06 1.02h's
    /// reasoning, this being its records: only where a card is up at all, since [`CURRENT_CARD`] holds the
    /// last card a boss was on and nothing clears it; and only against a record the game has named, the
    /// bytes of one it has not being the generator's.
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        if mem::read_committed::<i32>(at(ENEMY_MANAGER + enemy_manager::SPELLCARD_IS_ACTIVE))
            .unwrap_or(0)
            == 0
        {
            log!("score: no spell card is up; no attempt counted");
            return None;
        }
        let card = mem::read_committed::<u32>(at(CURRENT_CARD)).unwrap_or(u32::MAX);
        let record = unsafe { card_record(card) }?;
        if !unsafe { named(record) } {
            log!("score: card {card} is not one the game has named; no attempt counted");
            return None;
        }
        let attempts = mem::read_committed::<u16>(record + CATK_ATTEMPTS)
            .unwrap_or(0)
            .saturating_add(1);
        unsafe { mem::write::<u16>(record + CATK_ATTEMPTS, attempts) };
        Some(attempts)
    }

    /// Nothing: what bakes a card's name onto its plate here is a D3D11 draw of the game's own that has not
    /// been read, so a chapter put back under a later card's plate keeps that plate until the next card.
    unsafe fn redraw_card_name(&self) {}

    unsafe fn captures(&self) -> Vec<u8> {
        mem::read_committed_bytes(at(CARD_HISTORY), CARD_HISTORY_BYTES).unwrap_or_default()
    }

    unsafe fn set_captures(&self, saved: &[u8]) {
        if saved.len() != CARD_HISTORY_BYTES {
            return log!(
                "score: {} bytes of captures is not this build's {CARD_HISTORY_BYTES}; left alone",
                saved.len()
            );
        }
        unsafe { mem::write_bytes(at(CARD_HISTORY), saved) };
    }

    /// The name and the sum beside it together, never one without the other — see th06's, whose records
    /// these are the shape of.
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]) {
        let mut going_back = saved.to_vec();
        for (card, record) in going_back
            .as_chunks_mut::<CARD_BYTES>()
            .0
            .iter_mut()
            .enumerate()
        {
            let standing = at(CARD_HISTORY) + card * CARD_BYTES;
            let name = mem::read_committed_bytes(standing + CATK_NAME, CATK_NAME_BYTES)
                .unwrap_or_default();
            if name.len() == CATK_NAME_BYTES {
                record[CATK_NAME..CATK_NAME + CATK_NAME_BYTES].copy_from_slice(&name);
            }
            if let Some(sum) = mem::read_committed::<u8>(standing + CATK_NAME_SUM) {
                record[CATK_NAME_SUM] = sum;
            }
        }
        unsafe { self.set_captures(&going_back) };
    }

    /// **Nothing, because this game's own parse already leaves a run's records alone where they matter.**
    /// The read this would go in front of is the result screen's parse of the copy at `0x14005497c`, which
    /// lays the file's `CATK` records over the array a session counts into — and the screen's init skips
    /// it for the two states a run of its own arrives in, nine and seventeen (`0x14005495c` and
    /// `0x140054965`), so a run that finished keeps what it counted. The one build of that screen where the
    /// parse does run over a session's records is the ranking orb asks for after a run given up, and there
    /// the runtime takes the records before asking and puts them back once the screen is up — see
    /// `commit_records`. A bracket over the init at `0x1400544a0` (fifteen bytes of prologue, none of them
    /// rip-relative) would clear what the runtime is about to put back anyway, so `Hooks::ranking_read` is
    /// `None` here rather than a seam that changes nothing.
    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// **The copy every screen parses, put together again out of both files**: the records of runs out of the
    /// mode's own, the `CLRD` records the title lights its items from out of the game's — [`score_file`] has
    /// why it is a copy rather than a read. Installed the way the result screen installs its own: a block
    /// off the game's `malloc`, the block there before freed through its `free`, and the pointer and the
    /// length written where the parse reads them.
    ///
    /// Both files are asked for by their own names, relative — which is how the game asks for its own at
    /// `0x14030d628`, so each resolves where the game's open would.
    ///
    /// A `malloc` that answers nought leaves the copy as it was, said in the log: a parse of what was there is
    /// a screen showing the other mode's records, where a parse of nothing would be a first launch.
    unsafe fn read_what_the_modes_file_records(&self) {
        let own = orb_api::fs::read(Path::new(score::THEIRS_NAME)).ok();
        let modes_name = score::the_file_now_opened();
        let modes = orb_api::fs::read(Path::new(modes_name)).ok();
        let copy = score_file::the_copy_the_mode_reads(own.as_deref(), modes.as_deref());
        let malloc: unsafe extern "win64" fn(usize) -> usize =
            unsafe { std::mem::transmute(mallocs()) };
        let free: unsafe extern "win64" fn(usize) = unsafe { std::mem::transmute(frees()) };
        let block = unsafe { malloc(copy.len()) };
        if block == 0 {
            log!(
                "score: the game's malloc answered nothing for {} bytes; its copy of the file stands",
                copy.len()
            );
            return;
        }
        unsafe { mem::write_bytes(block, &copy) };
        let before = unsafe { mem::read::<usize>(at(SCORE_FILE_COPY)) };
        if before != 0 {
            unsafe { free(before) };
        }
        unsafe { mem::write::<usize>(at(SCORE_FILE_COPY), block) };
        unsafe {
            mem::write::<i32>(
                at(SCORE_FILE_COPY_LENGTH),
                i32::try_from(copy.len()).expect("a score file of a few kilobytes"),
            )
        };
        log!(
            "score: the copy the game parses is {modes_name}'s runs under {}'s unlocks, {} bytes",
            score::THEIRS_NAME,
            copy.len()
        );
    }

    /// **Yes**: its one read is at `0x14002c239`, inline in its startup, and the front end's unlocks come out
    /// of it — see [`score_file`], and `Hooks::unlocks_read`, which is `None` here because there is no
    /// callback to write a jump over.
    fn reads_its_file_once_at_startup(&self) -> bool {
        true
    }

    /// The item under the title menu's cursor, and only while the front end is what the supervisor is
    /// running and has been since before this frame — both of its states, for the reason th06's gives: the
    /// frame a run leaves for the front end ends with the one already saying front end and the front end not
    /// yet rebuilt.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        if !unsafe { self.front_end_settled_on(front_end::TITLE) } {
            return None;
        }
        match mem::read_committed::<i32>(at(FRONT_END + front_end::CURSOR))? {
            front_end::ITEM_START
            | front_end::ITEM_EXTRA_START
            | front_end::ITEM_PRACTICE_START => Some(Menu::Run),
            front_end::ITEM_REPLAY => Some(Menu::Replays),
            front_end::ITEM_SCORE => Some(Menu::Scores),
            _ => None,
        }
    }

    /// Whether the screen up would act on a decide on the read after this one: the two screens orb asks
    /// anything over each ignore their decide for their first frames — see [`front_end::TITLE_GRACE_FRAMES`]
    /// — and a press held back over those and handed over afterwards is a press the game had thrown away.
    unsafe fn menu_takes_a_press(&self) -> bool {
        if mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            != Some(STATE_MAINMENU)
        {
            return false;
        }
        let grace = match mem::read_committed::<i32>(at(FRONT_END + front_end::STATE)) {
            Some(front_end::TITLE) => front_end::TITLE_GRACE_FRAMES,
            Some(front_end::SHOT_SELECT) => front_end::SHOT_SELECT_GRACE_FRAMES,
            _ => return false,
        };
        let frames =
            mem::read_committed::<i32>(at(FRONT_END + front_end::SCREEN_FRAMES)).unwrap_or(0);
        // The read after this one is a frame on, so a count one short of the grace is a press the next
        // read takes.
        frames >= grace - 1
    }

    /// **The pads orb samples, for a menu of orb's** — the word [`Th06c::pad_word`] reads off them, the decide
    /// being `A` and the cancel `B`, as every screen of the game takes them.
    unsafe fn pad(&self, pad: Option<Reading>) -> Pad {
        the_pad_of(unsafe { self.pad_word(pad, true) })
    }

    /// **The pads orb samples, as the word [`Game::press_into_pad`] would have pressed** — read for a launch
    /// pressing through a pad, whose word comes off the pads rather than the bot: see `runtime::the_frames_word`.
    unsafe fn pad_word(&self, pad: Option<Reading>, _hats: bool) -> u16 {
        pad.map_or(0, the_word_of_a_sampled_pad)
    }

    /// **The word pressed on a pad, as [`Game::press_into_pad`] presses it** — the same buttons, numbered the way
    /// orb's sampling numbers an XInput pad's.
    unsafe fn pushed_for(&self, word: u16) -> Option<crate::joystick::Pushed> {
        Some(pushed_as(word))
    }

    unsafe fn add_pad_buttons(&self, _into: *mut c_void, _pad: Option<Reading>) {}

    /// **What the pause menu's own quit writes**: the two menu flags cleared and the supervisor put back to
    /// the front end, whose update then cuts the run's chain and registers the front end — th06 1.02h's
    /// own way out, this being its supervisor. The flags are written although neither should be set, for
    /// the reason th06's gives: a stale retry flag runs the retry menu's update on the title.
    unsafe fn leave_run(&self) -> bool {
        let scene =
            mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE)).unwrap_or(-1);
        if scene != STATE_GAMEMANAGER && scene != STATE_GAMEMANAGER_REINIT {
            return false;
        }
        unsafe {
            mem::write::<u8>(at(GAME_MANAGER + game_manager::IS_IN_GAME_MENU), 0);
            mem::write::<u8>(at(GAME_MANAGER + game_manager::IS_IN_RETRY_MENU), 0);
            mem::write::<i32>(at(SUPERVISOR + supervisor::CUR_STATE), STATE_MAINMENU);
        }
        true
    }

    /// Every button into the word the run acts on, so that the edge every screen reads — this word
    /// against the frame before's — finds nothing on the frame the game carries on into. The supervisor's
    /// update copies this word into the frame before's as its first act and reads the keyboard over it,
    /// so what is genuinely held still reads as held and only the edge is gone.
    unsafe fn swallow_input(&self) {
        unsafe { mem::write::<u32>(at(INPUT_THE_RUN_ACTS_ON), u32::MAX) };
    }

    /// The screen's state written to its own way out, where the state is the question about saving a
    /// replay — see [`result_screen::SAVE_REPLAY_QUESTION`] — and nothing on any other frame.
    unsafe fn skip_replay_prompt(&self) -> bool {
        if mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            != Some(STATE_RESULTSCREEN_FROMGAME)
        {
            return false;
        }
        let Some(screen) = (unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }) else {
            return false;
        };
        let state = screen + result_screen::STATE;
        if mem::read_committed::<i32>(state) != Some(result_screen::SAVE_REPLAY_QUESTION) {
            return false;
        }
        unsafe { mem::write::<i32>(state, result_screen::EXIT) };
        true
    }

    /// The same write over whatever state the screen is in, [`result_screen::EXIT`] being the game's own way
    /// out of it — the name entry included, which is the one a played run starts in.
    unsafe fn leave_the_screen_a_run_ends_at(&self) -> bool {
        if mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            != Some(STATE_RESULTSCREEN_FROMGAME)
        {
            return false;
        }
        let Some(screen) = (unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }) else {
            return false;
        };
        let state = screen + result_screen::STATE;
        if mem::read_committed::<i32>(state) == Some(result_screen::EXIT) {
            return false;
        }
        unsafe { mem::write::<i32>(state, result_screen::EXIT) };
        true
    }

    /// The supervisor's state alone: whichever state the screen is in, it is up while the supervisor runs
    /// it, and what takes it down is the screen's own leaving putting the state back.
    unsafe fn the_screen_a_run_ends_at_is_up(&self) -> bool {
        mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            == Some(STATE_RESULTSCREEN_FROMGAME)
    }

    /// The reinit state counts too, the way th06's does: a stage move is where the frame counter is about
    /// to be put back to nothing, and the frames either side of it belong to the stage whose counter this
    /// still is.
    unsafe fn stage_frame(&self) -> Option<u32> {
        let scene = mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))?;
        (scene == STATE_GAMEMANAGER || scene == STATE_GAMEMANAGER_REINIT)
            .then(|| mem::read_committed::<u32>(at(GAME_MANAGER + game_manager::GAME_FRAMES)))
            .flatten()
    }

    /// The generator itself — see [`RNG_SEED`] — which is the same word [`State::random_seed`] reads.
    unsafe fn generator(&self) -> u16 {
        mem::read_committed::<u16>(at(RNG_SEED)).unwrap_or(0)
    }

    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: mem::read_committed::<i32>(at(DIFFICULTY)).unwrap_or(-1),
            character: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::CHARACTER))
                .map_or(-1, i32::from),
            shot_type: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::SHOT))
                .map_or(-1, i32::from),
            practice: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::IS_IN_PRACTICE))
                .is_some_and(|practice| practice != 0),
            // Counted from one while a stage runs — see [`STAGE`] — and orb counts from nought.
            stage: mem::read_committed::<i32>(at(STAGE)).unwrap_or(0) - 1,
            // **This build has one mode**, where th06nc — the other 紅魔郷 in the library — has two.
            // See `RunStart::mode`.
            mode: 0,
        }
    }

    /// **On the shot type select, with the difficulty and the character already written by the two
    /// screens before it** and the shot being the item under the cursor — see [`front_end::SHOT_SELECT`].
    /// `None` for a practice run, whose stage is not chosen until afterwards.
    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        if !unsafe { self.front_end_settled_on(front_end::SHOT_SELECT) } {
            return None;
        }
        if mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::IS_IN_PRACTICE))? != 0 {
            return None;
        }
        Some(RunStart {
            difficulty: mem::read_committed::<i32>(at(DIFFICULTY))?,
            character: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::CHARACTER))?
                .into(),
            shot_type: mem::read_committed::<i32>(at(FRONT_END + front_end::CURSOR))?,
            practice: false,
            // A full run's chapter can be in any of them, and which one is what the mark says rather
            // than part of naming it.
            stage: 0,
            mode: 0,
        })
    }

    /// The five difficulties and the four shots, named as th06 1.02h's are — the two games being one game
    /// to a player, a file of one reads as a run of the same shot in the other's log. A practice run has
    /// none, for th06's reason: nothing to name is nothing to write, offer or mark.
    fn run_slot(&self, run: &RunStart) -> Option<String> {
        if run.practice {
            return None;
        }
        const DIFFICULTIES: [&str; 5] = ["easy", "normal", "hard", "lunatic", "extra"];
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

    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8> {
        resume::encode(saved)
    }

    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept> {
        resume::decode(bytes)
    }

    /// The eight fields this game's own stage records carry — the score in the previous stage's, the
    /// other seven in the stage's own — and the two orb keeps beside them — see
    /// [`resume::Numbers`], and [`Th06c::numbers`] for where each is read.
    unsafe fn run_state(&self) -> Vec<u8> {
        resume::encode_state(&unsafe { self.numbers() })
    }

    unsafe fn set_run_state(&self, state: &[u8]) {
        let Some(state) = resume::decode_state(state) else {
            return log!("resume: the numbers put back are not a run of this game's");
        };
        unsafe { self.put_the_numbers_back(&state) };
    }

    unsafe fn seed_the_stage(&self, state: &[u8]) {
        let Some(state) = resume::decode_state(state) else {
            return log!("resume: the seed put back is not a run of this game's");
        };
        unsafe { self.seed(state.seed) };
    }

    /// Every field of them, out of the struct, with the seed said again in front as the `0x1a2b` every other
    /// line writes it as.
    fn say_run_state(&self, state: &[u8]) -> String {
        match resume::decode_state(state) {
            Some(state) => format!("seed={:#06x} {state:?}", state.seed),
            None => "numbers of another game's run".to_owned(),
        }
    }

    /// The supervisor's state already the game manager while the state it last acted on is still the front
    /// end, which is the one frame between the two — th06 1.02h's own shape, see `Th06::run_chosen`. The
    /// demo and a replay are refused rather than left to the caller, both asking for a run the same way.
    unsafe fn run_chosen(&self) -> bool {
        mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            == Some(STATE_GAMEMANAGER)
            && mem::read_committed::<i32>(at(SUPERVISOR + supervisor::LAST_STATE))
                == Some(STATE_MAINMENU)
            && mem::read_committed::<u8>(at(A_REPLAY_IS_PLAYING)) == Some(0)
            && mem::read_committed::<u8>(at(THE_PLAYBACK_IS_THE_DEMO)) == Some(0)
    }

    /// The stage as the menu counts it: the stage callback raises the number by one, so it is handed the
    /// stage before the one meant — which is what the character select writes too. Checked against the
    /// seven the game has, the number coming out of a file beside the game.
    unsafe fn start_stage(&self, stage: i32) -> bool {
        if !(0..STAGES).contains(&stage) {
            log!("resume: there is no stage {}", stage + 1);
            return false;
        }
        unsafe { mem::write::<i32>(at(STAGE), stage) };
        true
    }

    /// **What the item that starts a run leaves behind**: the run's three numbers and the stage written
    /// where the front end's own screens write them, and then the game's own run start called — [`RUN_START`],
    /// which is what both of those screens call once their numbers are in.
    ///
    /// **Called rather than copied**, which is what this once did: the lives, the bombs, the playback flag and
    /// the supervisor's state written from here, and the music left as the title had it. A watched attempt's
    /// run started that way died before its stage had drawn — 128 frames of its playback with the generator
    /// on its seed and the player where it spawns, and the process gone with no line after — where a run
    /// the shot select started played through; the one thing between the two was what that routine does and
    /// this did not, the track stopped and put to `-1`.
    unsafe fn start_a_run(&self, run: &RunStart) -> bool {
        if !(0..STAGES).contains(&run.stage) {
            log!("replay: there is no stage {}", run.stage + 1);
            return false;
        }
        if !(0..=DIFFICULTY_EXTRA).contains(&run.difficulty) {
            log!("replay: there is no difficulty {}", run.difficulty);
            return false;
        }
        if !(0..SHOT_COMBINATIONS).contains(&(run.character * 2 + run.shot_type)) {
            log!(
                "replay: there is no character {} shot {}",
                run.character,
                run.shot_type
            );
            return false;
        }
        unsafe {
            mem::write::<i32>(at(DIFFICULTY), run.difficulty);
            mem::write::<u8>(
                at(GAME_MANAGER + game_manager::CHARACTER),
                run.character as u8,
            );
            mem::write::<u8>(at(GAME_MANAGER + game_manager::SHOT), run.shot_type as u8);
            mem::write::<u8>(at(GAME_MANAGER + game_manager::IS_IN_PRACTICE), 0);
            // The demo's own flag is not the run start's to clear — the logo's hand-over sets it and the
            // demo's end clears it — and a run written into being from the title has the demo behind it.
            mem::write::<u8>(at(THE_PLAYBACK_IS_THE_DEMO), 0);
            mem::write::<i32>(at(STAGE), run.stage);
        }
        let start: unsafe extern "win64" fn() -> i32 = unsafe { std::mem::transmute(run_start()) };
        unsafe { start() };
        true
    }

    /// **The run asked for through the shot select's own decide**, the way th06's is: the difficulty and the
    /// character written where the two selects before it write them, the front end put on the shot select
    /// with its cursor on the shot and its frames past the `0x1e` it ignores a decide for, and the press the
    /// caller hands over next is the one `0x14003592d` reads — which writes the shot off the cursor, the
    /// stage for a full run, and calls [`RUN_START`].
    ///
    /// **Rather than a run written into being from the replay list**, which [`Th06c::start_a_run`] is: an
    /// attempt watched from a run started that way died 128 frames into its playback, the process gone with
    /// no line after it, twice over — with the run start copied and with it called — and a run the item
    /// started played through. The trait's doc has th06 dying the same way at the same frame.
    unsafe fn front_end_starts_a_run(&self, run: &RunStart) -> bool {
        if !(0..=DIFFICULTY_EXTRA).contains(&run.difficulty)
            || !(0..SHOT_COMBINATIONS).contains(&(run.character * 2 + run.shot_type))
        {
            log!(
                "replay: there is no difficulty {} character {} shot {}",
                run.difficulty,
                run.character,
                run.shot_type
            );
            return false;
        }
        if mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            != Some(STATE_MAINMENU)
        {
            log!("replay: the front end is not what is running; no run is asked of it");
            return false;
        }
        unsafe {
            mem::write::<i32>(at(DIFFICULTY), run.difficulty);
            mem::write::<u8>(
                at(GAME_MANAGER + game_manager::CHARACTER),
                run.character as u8,
            );
            mem::write::<u8>(at(GAME_MANAGER + game_manager::IS_IN_PRACTICE), 0);
            mem::write::<u8>(at(THE_PLAYBACK_IS_THE_DEMO), 0);
            mem::write::<i32>(at(FRONT_END + front_end::CURSOR), run.shot_type);
            mem::write::<i32>(at(FRONT_END + front_end::STATE), front_end::SHOT_SELECT);
            mem::write::<i32>(
                at(FRONT_END + front_end::SCREEN_FRAMES),
                front_end::SHOT_SELECT_GRACE_FRAMES,
            );
        }
        true
    }

    unsafe fn run_finished(&self) -> bool {
        mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE))
            == Some(STATE_RESULTSCREEN_FROMGAME)
    }

    unsafe fn reproduction(&self) -> String {
        unsafe { self.reproduced() }.to_string()
    }

    /// **The shot's own phase put where the stage's first frame left it** — see [`player::SHOT_PHASE`] for
    /// the pair and [`ONE_UPDATE_INTO_A_CYCLE`] for why the value is not the one the press writes, and for
    /// the pick-up of the Extra that parted for want of it.
    ///
    /// **Only where the pair is idle**, which is what a resumed stage's build leaves — the register writes
    /// the idle pair at `0x14004a3ca` and orb feeds the build frame nought — and **only where the run was
    /// firing across the stage's start**, which `word` answers, th06nc's reasons for both.
    ///
    /// **And the decide is firing on this game**, which is what the first word of a run started from the
    /// shot select is: `Enter` reaches the game's word as its shot bit as well as its own. Measured on a real
    /// recording of the Extra — `chose=0x1000` on the stage's first four frames, and `input=0x0101
    /// shot=1,0`, `shot=2,1` on them — where a check of the shot's own bit left the pick-up's cycle idle, and
    /// the playback one update behind from frame 2 on.
    unsafe fn starts_the_shot_a_stage_began_with(&self, word: u16) {
        if word & (button::SHOT | button::DECIDE) == 0 {
            crate::detail!(
                "resume: the run was not firing as the stage began; its shot cycle stays idle"
            );
            return;
        }
        let phase = at(PLAYER + player::SHOT_PHASE);
        let was = unsafe { mem::read::<i32>(phase) };
        if was >= 0 {
            crate::detail!(
                "resume: the stage's shot is already on phase {was}; the cycle stays as it is"
            );
            return;
        }
        unsafe { mem::write::<i32>(phase, ONE_UPDATE_INTO_A_CYCLE.0) };
        unsafe {
            mem::write::<i32>(
                at(PLAYER + player::SHOT_PHASE_BEFORE),
                ONE_UPDATE_INTO_A_CYCLE.1,
            )
        };
        crate::detail!(
            "resume: the stage's shot cycle begins on phase {} where the run's did",
            ONE_UPDATE_INTO_A_CYCLE.0,
        );
    }

    /// **The bullet manager's own array, folded three ways**: which slots are in use and in which state,
    /// where each of them is, and what each moves by.
    ///
    /// **Three folds and not one**, because what a parting wants said is which of the three parted: two
    /// passes whose slots agree and whose positions do not have moved the same bullets differently, and two
    /// whose slots disagree have fired or dropped a different bullet — and one number over all of it says
    /// only that something did.
    ///
    /// **The state, the position and the step of a slot, and nothing else of its `0x5d0` bytes**, for
    /// th07's reason spelled out in its own fold: a slot holds a sprite with clocks of its own that a
    /// playback never draws, so a fold over the whole slot reports a parting on every frame. These three
    /// are what the game's own update at `0x14000db90` reads and writes, and they are what the rules are
    /// handed — see [`bullet_manager`].
    ///
    /// **Which a parting asked for.** A pick-up of a th06c run agreed with the run on every field of
    /// [`Th06c::reproduction`] — `player`, `input`, `randoms` and `rng` to the digit — and reported
    /// `the player was hit at frame 185` all the same, of a record whose chapters were written with
    /// `deaths: 0`. Where the bullets are is the one thing about a hit that no field of that line looks at.
    unsafe fn the_worlds_fingerprint(&self) -> Option<String> {
        // The fold every other one in this tree is: multiply by a prime and add, which two arrays
        // differing anywhere differ under.
        let eat = |fold: &mut u32, bits: u32| *fold = fold.wrapping_mul(31).wrapping_add(bits);
        let (mut states, mut places, mut steps) = (0u32, 0u32, 0u32);
        let mut live = 0usize;
        for slot in 0..bullet_manager::SLOTS {
            let bullet =
                at(BULLET_MANAGER) + bullet_manager::BULLETS + slot * bullet_manager::BYTES;
            let state = unsafe { mem::read::<u16>(bullet + bullet_manager::STATE) };
            // The state of every slot, in use or not: a slot going out is a thing two passes can differ
            // about on its own — the walk's own gate at `0x14000dcc0` reads nought as a slot holding
            // nothing.
            eat(&mut states, u32::from(state));
            if state == 0 {
                continue;
            }
            live += 1;
            for (fold, of) in [
                (&mut places, bullet_manager::POSITION),
                (&mut steps, bullet_manager::STEP),
            ] {
                for axis in 0..2 {
                    let word = unsafe { mem::read::<f32>(bullet + of + axis * size_of::<f32>()) };
                    eat(fold, word.to_bits());
                }
            }
        }
        Some(format!(
            "bullets={live} states={states:08x} places={places:08x} steps={steps:08x}"
        ))
    }

    /// The replay manager's own frame count being nothing yet — the manager is made as the stage's
    /// numbers go in and its count cleared as the record begins.
    unsafe fn mid_build(&self) -> bool {
        unsafe { self.reproduced() }.replay_frame < 0
    }

    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &chapters::MIDSTAGE
    }

    /// The Extra, and nothing after it: this game's stage switch ends there too.
    fn stages_past_the_numbered(&self) -> &'static [&'static str] {
        &["extra"]
    }
}

impl Th06c {
    /// Whether the front end is what the supervisor is running, has been since before this frame, and is
    /// on `screen` — see [`Th06c::menu_pointed_at`] for why both of the supervisor's states are asked.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the exe's base already told.
    unsafe fn front_end_settled_on(&self, screen: i32) -> bool {
        mem::read_committed::<i32>(at(SUPERVISOR + supervisor::CUR_STATE)) == Some(STATE_MAINMENU)
            && mem::read_committed::<i32>(at(SUPERVISOR + supervisor::LAST_STATE))
                == Some(STATE_MAINMENU)
            && mem::read_committed::<i32>(at(FRONT_END + front_end::STATE)) == Some(screen)
    }

    /// **The numbers a stage starts with, read where the game's own recording reads them** — the score,
    /// which `0x14004da37` saves into the previous stage's record, and `0x14004da71` onwards for the new
    /// stage's own, out of the same statics: the seed the stage was seeded with, the point items, the
    /// power, the lives, the bombs, the rank and the power items — and orb's own two beside them.
    ///
    /// The seed out of the game manager's copy and not the generator, for th06's reason: the copy is what
    /// the stage started from once the stage has drawn from the generator.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn numbers(&self) -> resume::Numbers {
        let manager = at(GAME_MANAGER);
        resume::Numbers {
            // **The panel's field and not the run's**, which is where the score stands when a stage is written
            // down: the stage callback copies the run's into the panel's (`0x140024cf5`) and leaves the run's
            // at nought in its tail (`0x14002548d`), and only the stage's first update raises it back
            // (`0x140023f71`). Read off the run's, every stage after the first was written down with no
            // score — a Normal run's stage 6 as nought where its panel held 55,221,580 (2026-10-05) — and a
            // chapter of it played back without the extra lives that score had paid for.
            score: mem::read_committed::<u32>(manager + game_manager::GUI_SCORE).unwrap_or(0),
            seed: mem::read_committed::<u16>(manager + game_manager::RANDOM_SEED).unwrap_or(0),
            point_items: mem::read_committed::<u16>(manager + game_manager::POINT_ITEMS)
                .unwrap_or(0),
            power: mem::read_committed::<u16>(at(POWER)).unwrap_or(0),
            lives: mem::read_committed::<i8>(at(LIVES)).unwrap_or(0),
            bombs: mem::read_committed::<i8>(at(BOMBS)).unwrap_or(0),
            rank: mem::read_committed::<i32>(manager + game_manager::RANK).unwrap_or(0),
            power_items: mem::read_committed::<i8>(manager + game_manager::POWER_ITEMS)
                .unwrap_or(0),
            extra_lives: mem::read_committed::<i8>(manager + game_manager::EXTRA_LIVES)
                .unwrap_or(0),
            deaths: DEATHS.load(Ordering::Relaxed),
        }
    }

    /// **And back, field for field, the way the playback's own init puts them at `0x14004db3a`
    /// onwards** — all but the seed, which is [`Th06c::seed`]'s and has to be earlier. The score into both
    /// of the game's two, the one the panel draws and the one the run counts, as that init puts the
    /// previous record's into both from stage 2 on (`0x14004dbe1`, `0x14004dbe7`).
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage built and not yet updated.
    unsafe fn put_the_numbers_back(&self, numbers: &resume::Numbers) {
        let manager = at(GAME_MANAGER);
        unsafe {
            mem::write::<u32>(manager + game_manager::GUI_SCORE, numbers.score);
            mem::write::<u32>(manager + game_manager::SCORE, numbers.score);
            mem::write::<u16>(manager + game_manager::POINT_ITEMS, numbers.point_items);
            mem::write::<u16>(at(POWER), numbers.power);
            mem::write::<i8>(at(LIVES), numbers.lives);
            mem::write::<i8>(at(BOMBS), numbers.bombs);
            mem::write::<i32>(manager + game_manager::RANK, numbers.rank);
            mem::write::<i8>(manager + game_manager::POWER_ITEMS, numbers.power_items);
            mem::write::<i8>(manager + game_manager::EXTRA_LIVES, numbers.extra_lives);
        }
        DEATHS.store(numbers.deaths, Ordering::Relaxed);
    }

    /// **The generator and the game manager's copy of it both, and the count of draws back to nought** — the
    /// three writes the playback's own init makes at `0x14004db70` and `0x14004db77`, made here before the
    /// stage is built out of them.
    ///
    /// # Safety
    /// Must run on the game's main thread, from the hook over [`Hooks::stage_building`], before the original.
    unsafe fn seed(&self, seed: u16) {
        unsafe {
            mem::write::<u16>(at(RNG_SEED), seed);
            mem::write::<u16>(at(GAME_MANAGER + game_manager::RANDOM_SEED), seed);
            mem::write::<u32>(at(RNG_COUNT), 0);
        }
    }

    /// The line one pass is held against another's — see [`Reproduction`].
    ///
    /// # Safety
    /// Must run on the game's main thread, with the exe's base already told.
    unsafe fn reproduced(&self) -> Reproduction {
        // **The exe's own image read without asking**, for `in_the_image`'s reason: this is every frame's own
        // line of a playback, and with every item slot asked of the host it was `0.4ms` of each update — `3.2`
        // of the `10.7` seconds a pick-up of the Extra at chapter 7 took. The replay manager is a pointer
        // into the heap, and that is asked as before.
        let replay_frame = unsafe { in_the_image::<usize>(REPLAY_MANAGER) };
        let replay_frame = match replay_frame {
            0 => -1,
            replay => mem::read_committed::<i32>(replay + replay_manager::FRAME).unwrap_or(-1),
        };
        let player = PLAYER + player::POSITION;
        let items = (0..item::SLOTS)
            .filter(|slot| {
                (unsafe {
                    in_the_image::<u8>(
                        ITEM_MANAGER + item::ITEMS + slot * item::BYTES + item::ALIVE,
                    )
                }) != 0
            })
            .count() as u32;
        Reproduction {
            replay_frame,
            input: (unsafe { in_the_image::<u32>(INPUT_THE_RUN_ACTS_ON) } as u16) & RUN_INPUT_BITS,
            player: unsafe {
                (
                    in_the_image::<f32>(player),
                    in_the_image::<f32>(player + size_of::<f32>()),
                )
            },
            player_area: unsafe {
                (
                    in_the_image::<f32>(PLAYER_AREA + size_of::<f32>()),
                    in_the_image::<f32>(PLAYER_AREA + 3 * size_of::<f32>()),
                )
            },
            randoms: unsafe { in_the_image::<u32>(RNG_COUNT) },
            seed: unsafe { in_the_image::<u16>(RNG_SEED) },
            items,
            score: unsafe { in_the_image::<u32>(GAME_MANAGER + game_manager::GUI_SCORE) },
            extra_lives: unsafe { in_the_image::<i8>(GAME_MANAGER + game_manager::EXTRA_LIVES) },
            rank: unsafe { in_the_image::<i32>(GAME_MANAGER + game_manager::RANK) },
            sub_rank: unsafe { in_the_image::<i32>(GAME_MANAGER + game_manager::SUB_RANK) },
            shot_phase: unsafe {
                (
                    in_the_image::<i32>(PLAYER + player::SHOT_PHASE),
                    in_the_image::<i32>(PLAYER + player::SHOT_PHASE_BEFORE),
                )
            },
            player_state: unsafe { in_the_image::<u8>(PLAYER + player::STATE) },
            line: unsafe { dialogue_msg_idx() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Th06c, bullet_manager, button, dialog, key};
    use crate::game::{Game, Read};

    /// **What each of the five bullet states multiplies its own step by**, which is the whole of what the
    /// model does with the state it reads: a bullet moves `[+0x30] += [+0x8] × this`, and a scale that is
    /// wrong makes the movement disagree with the step, which is the test the rules use to tell a bullet a
    /// script is moving from one that is moving itself.
    ///
    /// State 5 is the one a launch corrected — read as a half off `0x14030dfc8` and measured as whole,
    /// `moved 3.30,5.38 saying 1.65,2.69` — so it is here beside the four that were right.
    #[test]
    fn each_bullet_state_scales_its_own_step_by_what_the_game_divides_it_by() {
        for (state, scale) in [(1, 1.0), (2, 0.5), (3, 0.4), (4, 1.0 / 3.0), (5, 0.5)] {
            assert_eq!(
                bullet_manager::a_steps_scale(state),
                scale,
                "the step of a bullet in state {state} is scaled by something else",
            );
        }
    }

    /// **What orb stands in front of here is a name and not a place**, which is what makes the read
    /// reachable at all: the slot the game keeps its answer in holds nought until the game's own startup
    /// writes it, so there is nothing at the attach to write over.
    #[test]
    fn the_keyboard_read_is_stood_in_front_of_by_the_name_the_game_asks_for() {
        let Some(Read::LookedUp(lookup)) = Th06c.hooks().input else {
            panic!("th06c's input is not a name the game asks for");
        };
        assert_eq!(lookup.name, "GetKeyboardState");
    }

    /// **And the startup dialog is got past by clicking a window and then the start button**, in that
    /// order: the start button is what makes the dialog procedure read the radio buttons, so a launch
    /// that clicked it first would come up in whatever the dialog opened on.
    #[test]
    fn the_startup_dialog_is_answered_with_a_window_and_then_the_start_button() {
        let Some(answer) = Th06c.hooks().startup_dialog else {
            panic!("th06c puts a startup dialog up and orb answers none");
        };
        assert_eq!(answer.clicks, &[dialog::A_640_BY_480_WINDOW, dialog::START],);
        assert_eq!(
            answer.clicks.last(),
            Some(&dialog::START),
            "the start button is clicked before the window is chosen",
        );
    }

    /// **Every button orb's word has is a key it presses**, which is what stops a recipe naming one that
    /// reaches nothing: the two tables are one table, and this is what says so.
    #[test]
    fn every_button_a_recipe_can_name_is_a_key_this_game_is_played_with() {
        for (name, bit, pressed) in Th06c::BUTTONS {
            assert_eq!(
                Th06c.button_bit(name),
                Some(*bit),
                "{name} is not a button a recipe can name",
            );
            assert_ne!(*pressed, 0, "{name} is pressed with no key at all");
        }
        assert_eq!(
            Th06c.button_bit("nothing of the kind"),
            None,
            "a name this game has no button for answered a bit",
        );
    }

    /// **And a word's keys are the ones it names and no others**, which is what a snapshot orb writes
    /// into has to be: a key nobody pressed is a key the game reads as pressed.
    #[test]
    fn a_word_presses_the_keys_its_own_buttons_are() {
        let mut state = [0u8; 256];
        unsafe { Th06c.press_into(state.as_mut_ptr(), button::DECIDE | button::UP) };
        for (at, byte) in state.iter().enumerate() {
            let expected = u8::from(at == usize::from(key::DECIDE) || at == usize::from(key::UP));
            assert_eq!(
                u8::from(*byte != 0),
                expected,
                "the key at {at:#04x} reads {byte:#04x} where a decide and an up were pressed",
            );
        }
    }
}
