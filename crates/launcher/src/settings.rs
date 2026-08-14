//! The settings orb asks for before it starts the game.
//!
//! They are asked here rather than left to somebody editing `orb.yaml` because every one of
//! them is about the machine the game is being played on — how much of which screen it gets,
//! whether it keeps drawing while something else is in front — and a person who has just
//! installed one file has nothing to edit. What is answered is written back to `orb.yaml`, so
//! the next launch can start the game without asking; the last of the questions is whether to
//! ask again.
//!
//! **A real dialog**, class `#32770`, from a template built in memory rather than from a
//! resource: a resource means a `.rc` and a resource compiler in the build, for a list of controls
//! short enough to write out here. Being one rather than merely looking like one is what matters — a window manager
//! decides whether to leave a window alone by asking what it is, and the dialog class is the
//! answer it looks for. One built out of `CreateWindowExW` with a dialog's styles carries its own
//! class whatever it looks like, and gets tiled.
//!
//! Its measurements are therefore in dialog units, which are the font's own, so the dialog scales
//! with whatever the system gives it and nothing here is in pixels.

use std::error::Error;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use orb_config::{Config, Language, Screen};
use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, LOGPIXELSY, ReleaseDC};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetFocus, VIRTUAL_KEY, VK_DOWN, VK_UP};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BM_CLICK, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, CB_ADDSTRING,
    CB_GETCOUNT, CB_GETCURSEL, CB_GETDROPPEDSTATE, CB_SETCURSEL, CB_SHOWDROPDOWN, CBS_DROPDOWNLIST,
    DLGTEMPLATE, DialogBoxIndirectParamW, EndDialog, GetDlgCtrlID, GetDlgItem, GetSystemMetrics,
    IDCANCEL, IDOK, NONCLIENTMETRICSW, SM_CXSCREEN, SM_CYSCREEN, SPI_GETNONCLIENTMETRICS,
    SendMessageW, SetProcessDPIAware, SystemParametersInfoW, WM_COMMAND, WM_INITDIALOG, WM_KEYDOWN,
    WM_KEYUP, WM_NEXTDLGCTL, WS_CAPTION, WS_CHILD, WS_GROUP, WS_POPUP, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE, WS_VSCROLL,
};

use crate::pad::{self, Mapping, Push};

/// `BST_CHECKED`, which `BM_GETCHECK` answers with and `BM_SETCHECK` is given. Spelled out
/// because windows-sys does not carry it: it is a plain `#define` rather than anything with a
/// type of its own.
const CHECKED: usize = 1;

/// The dialog styles, which windows-sys does not carry either. `DS_SETFONT` is what makes the
/// template's font the dialog's — and with it the dialog unit; `DS_MODALFRAME` is the thin fixed
/// frame; `DS_CENTER` puts it in the middle of the screen, there being no owner to centre on.
const DS_SETFONT: u32 = 0x40;
const DS_MODALFRAME: u32 = 0x80;
const DS_CENTER: u32 = 0x0800;

/// The window classes a template names by ordinal rather than by string.
const ATOM_BUTTON: u16 = 0x0080;
const ATOM_STATIC: u16 = 0x0082;
const ATOM_COMBOBOX: u16 = 0x0085;

/// What the settings were answered with, which is every key of `orb.yaml`.
pub struct Answers {
    pub screen: Screen,
    /// `None` for the machine's own, which is what the row offers first.
    pub language: Option<Language>,
    pub thcrap: bool,
    pub always_draw: bool,
    pub boundary_flash: bool,
    pub skip_ending: bool,
    pub hide_mouse: bool,
    pub dpad_moves: bool,
    pub ask_at_startup: bool,
}

impl Answers {
    pub fn apply(&self, config: &mut Config) {
        config.screen = self.screen;
        config.language = self.language;
        // By name, where the file may have left it to the machine's language: a box that was in front of
        // somebody and came back either way is an answer, and the next launch is not to go looking for
        // one again. Which is the row above's difference too — a language row offers the machine's own as
        // an item, and a box has no room for a third state.
        config.thcrap = Some(self.thcrap);
        config.always_draw = self.always_draw;
        config.boundary_flash = self.boundary_flash;
        config.skip_ending = self.skip_ending;
        config.hide_mouse = self.hide_mouse;
        config.dpad_moves = self.dpad_moves;
        config.ask_at_startup = self.ask_at_startup;
    }
}

/// The size the game renders at, which is offered whatever the monitor is: it is the one size
/// that scales nothing at all, and a monitor too small for it is one the game cannot be played
/// on.
const OWN_SIZE: (u32, u32) = (640, 480);

/// The heights a window is offered at, which are the ones monitors have. The width comes from
/// the aspect ratio rather than being listed, so every size offered is exactly its ratio.
const HEIGHTS: [u32; 4] = [720, 1080, 1440, 2160];

/// What a window's frame adds to what is inside it, near enough to keep a size that could not
/// be put on the screen whole out of the list. Guessed rather than measured, because the window
/// this is about is the game's: it does not exist yet, and it is created by the other half of
/// orb inside a process that has not started.
const FRAME: (u32, u32) = (16, 40);

/// The window sizes worth offering on a monitor this size, biggest first and 16:9 before 4:3.
///
/// 16:9 above because that is the ratio that leaves black down the sides for orb's own numbers,
/// and biggest first because the size somebody is looking for on a large monitor is the large
/// one — a list that starts at 640x480 is a list to scroll past.
///
/// Filtered by the monitor because a window bigger than the screen is a window with part of
/// itself off it, and these settings are being asked on the machine the game will be played on.
///
/// **Every size here leaves the black down the sides or leaves none**, the game being 4:3, so the third
/// shape orb's status line can land in — a bar *under* the game, from a client taller than 4:3 — is one
/// this list cannot produce. Reaching it takes `orb.yaml` written by hand with such a `screen` and
/// `ask_at_startup: false`, since the dialog drops a size its own list has not got and falls back to
/// fullscreen. So nobody playing can select it, and no run of theirs will find a fault in it.
pub fn sizes(monitor: (u32, u32)) -> Vec<(u32, u32)> {
    let fits = |(width, height): &(u32, u32)| {
        width + FRAME.0 <= monitor.0 && height + FRAME.1 <= monitor.1
    };
    let by_ratio = |(width, height): (u32, u32)| {
        HEIGHTS
            .iter()
            .rev()
            .map(move |tall| (tall * width / height, *tall))
            .filter(fits)
    };
    by_ratio((16, 9))
        .chain(by_ratio((4, 3)))
        // Last, since it is the smallest of the 4:3 sizes and the only one no monitor is too
        // small for.
        .chain(std::iter::once(OWN_SIZE))
        .collect()
}

/// The primary monitor, in real pixels.
///
/// Real because the process has asked to be told about display scaling — see [`ignore_scaling`] —
/// so a monitor at 150% reports the 3840x2160 it is rather than the 2560x1440 it pretends to be,
/// and the sizes offered here are sizes the game's window will actually come out at.
pub fn primary_monitor() -> (u32, u32) {
    let size = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    (size.0.max(0) as u32, size.1.max(0) as u32)
}

/// Takes the process out of display scaling, so that every pixel this side counts is a pixel on
/// the screen.
///
/// The other half of orb does the same inside the game — see `orb/window.rs` — because a size
/// offered here has to be a size the game's window can be given. The dialog is unharmed by it:
/// its measurements are in dialog units and the font it is given is the one the system wants at
/// this dpi, so the two scale together.
fn ignore_scaling() {
    unsafe { SetProcessDPIAware() };
}

/// Which hand the dialog was answered with, for the line the launcher prints about it.
pub fn answered_with() -> &'static str {
    if BY_PAD.load(Ordering::Relaxed) {
        "pad"
    } else {
        "keyboard or mouse"
    }
}

/// One switch: what it says, where it starts from, and where its answer goes.
///
/// The answer travels with the label rather than being read back by position, so that moving a
/// switch up the dialog cannot quietly move somebody's answer to another key.
struct Switch {
    /// What the key does rather than what it is called, since nobody reading this dialog has read
    /// the file — and in the language the person answering reads.
    text: fn(Language) -> &'static str,
    /// Where the box starts from, which for the one switch whose default the machine's own language
    /// decides is that answer rather than the file's — a box shown unticked and a launch that patches
    /// the game anyway would be the dialog lying about what pressing start does.
    shown: fn(&Config, Language) -> bool,
    answered: fn(&mut Answers, bool),
}

/// The switches, in the order they are stacked. Each is one key of `orb.yaml`.
const SWITCHES: [Switch; 7] = [
    Switch {
        text: |language| match language {
            Language::Japanese => "インストールされている翻訳パッチを使う",
            Language::English => "Use the translation patch installed beside the game",
        },
        shown: |config, language| config.thcrap_wanted(language),
        answered: |answers, on| answers.thcrap = on,
    },
    Switch {
        text: |language| match language {
            Language::Japanese => "エンディングをスキップする",
            Language::English => "Skip the ending",
        },
        shown: |config, _| config.skip_ending,
        answered: |answers, on| answers.skip_ending = on,
    },
    Switch {
        text: |language| match language {
            Language::Japanese => "背面でもゲームを止めない",
            Language::English => "Keep playing while another window is in front",
        },
        shown: |config, _| config.always_draw,
        answered: |answers, on| answers.always_draw = on,
    },
    Switch {
        text: |language| match language {
            Language::Japanese => "チャプターの切り替わりにフラッシュする",
            Language::English => "Flash the play field where a chapter begins",
        },
        shown: |config, _| config.boundary_flash,
        answered: |answers, on| answers.boundary_flash = on,
    },
    Switch {
        text: |language| match language {
            Language::Japanese => "時間経過でマウスカーソルを消す",
            Language::English => "Hide the mouse pointer once the mouse is still",
        },
        shown: |config, _| config.hide_mouse,
        answered: |answers, on| answers.hide_mouse = on,
    },
    Switch {
        text: |language| match language {
            Language::Japanese => "ゲームパッドの十字キーでも移動する",
            Language::English => "Move on a gamepad's d-pad as well as its stick",
        },
        shown: |config, _| config.dpad_moves,
        answered: |answers, on| answers.dpad_moves = on,
    },
    Switch {
        text: |language| match language {
            Language::Japanese => "起動時に毎回訊ねる",
            Language::English => "Ask all of this at every launch",
        },
        shown: |config, _| config.ask_at_startup,
        answered: |answers, on| answers.ask_at_startup = on,
    },
];

/// The three the language row offers, in the order they are stacked.
///
/// The machine's own first, that being what a file with nothing in it says and what nobody has to
/// choose.
const LANGUAGES: [Option<Language>; 3] = [None, Some(Language::Japanese), Some(Language::English)];

/// What each item of that row says.
///
/// **The two languages are named in themselves and not translated**, the way every language picker
/// names them: somebody looking for English is looking for the word `English`, whichever language the
/// dialog they are reading is in. Only the machine's own is a word this dialog has to say.
fn language_text(item: Option<Language>, language: Language) -> &'static str {
    match (item, language) {
        (None, Language::Japanese) => "自動",
        (None, Language::English) => "Automatic",
        (Some(Language::Japanese), _) => "日本語",
        (Some(Language::English), _) => "English",
    }
}

/// What the two labels down the left say.
fn screen_label(language: Language) -> &'static str {
    match language {
        Language::Japanese => "画面",
        Language::English => "Screen",
    }
}

fn language_label(language: Language) -> &'static str {
    match language {
        Language::Japanese => "言語",
        Language::English => "Language",
    }
}

/// What one item of the list of sizes says.
///
/// The ratio is said after the size so that a list of them says which of the two shapes each is
/// without anybody dividing it out — see [`ratio`].
fn screen_text(choice: Screen, language: Language) -> String {
    match (choice, language) {
        (Screen::Fullscreen, Language::Japanese) => "フルスクリーン".to_owned(),
        (Screen::Fullscreen, Language::English) => "Fullscreen".to_owned(),
        (Screen::Window { width, height }, Language::Japanese) => {
            format!("ウィンドウ {width}x{height} ({})", ratio(width, height))
        }
        (Screen::Window { width, height }, Language::English) => {
            format!("Window {width}x{height} ({})", ratio(width, height))
        }
    }
}

/// The line about the pad.
///
/// The two buttons are named by what they are in the game rather than by a letter on a pad, because
/// that is what they are: which physical button decides is read out of the game's own configuration —
/// see [`Mapping`] — so a line naming one cannot be right for two pads.
fn pad_hint(language: Language) -> &'static str {
    match language {
        Language::Japanese => "パッド: 上下で移動  ショットで決定  ボムでやめる",
        Language::English => "Pad: up and down to move, shoot to decide, bomb to quit",
    }
}

/// What the two buttons say: the one that starts the game and the one that starts none.
fn start_text(language: Language) -> &'static str {
    match language {
        Language::Japanese => "はじめる",
        Language::English => "Start",
    }
}

fn quit_text(language: Language) -> &'static str {
    match language {
        Language::Japanese => "やめる",
        Language::English => "Quit",
    }
}

const SCREEN_ID: u16 = 100;
const LANGUAGE_ID: u16 = 150;
const SWITCH_ID: u16 = 200;

/// The dialog, in dialog units — a quarter of the font's average character width across and an
/// eighth of its height down — so all of it scales with the font the system gives it.
///
/// **As tall as what is stacked in it**, rather than a number of its own: the switches are one row each
/// and the pad's line and the buttons are two more, so a switch added to [`SWITCHES`] takes the dialog
/// with it. Written out as a number it was 136, which is what this comes to for the four switches it had
/// then; a fifth would have put the pad's line through the buttons.
const DIALOG: (i16, i16) = (268, HINT_TOP + TEXT + 4 + BUTTON.1 + MARGIN.1);
const MARGIN: (i16, i16) = (10, 8);
/// The pitch of a row, which is a line of text and the space under it.
const LINE: i16 = 16;
/// And one line of text: a label, a switch's own box, the line about the pad.
const TEXT: i16 = 10;
const BUTTON: (i16, i16) = (56, 16);
/// The whole of a combo box including its dropped list, which is what its height measures — not
/// the box shown while it is closed.
const COMBO: i16 = 90;
/// Where the language row is, which is under the row the sizes are on.
const LANGUAGE_TOP: i16 = MARGIN.1 + LINE;
/// And where the switches start, under both of the rows with a list on them.
const SWITCHES_TOP: i16 = LANGUAGE_TOP + LINE + 6;
/// And the line about the pad, under the last of them.
const HINT_TOP: i16 = SWITCHES_TOP + LINE * SWITCHES.len() as i16 + 4;
/// The top of the two buttons, which are the last row.
const BUTTONS_TOP: i16 = DIALOG.1 - MARGIN.1 - BUTTON.1;

/// The column the two labels are written in, and so where the lists beside them start.
///
/// As wide as the longer label in whichever language it is: `画面` is two characters where `Language`
/// is eight, and a dialog unit across is a quarter of the font's average character width — so the
/// English column is the wider one. A number for both would be a column with `Language` running into
/// the list beside it, or a Japanese dialog with a hand's width of nothing down its left.
fn label_width(language: Language) -> i16 {
    match language {
        Language::Japanese => 26,
        Language::English => 44,
    }
}

// The rows held against the dialog they are stacked in: every switch above the line about the pad, that
// line above the two buttons, and the buttons inside the dialog. A `const` block rather than a test
// because [`DIALOG`]'s own height is worked out from these — so a switch added to [`SWITCHES`] with a
// height that did not follow it is a build that stops, rather than a dialog with that line drawn through
// its buttons.
const _: () = {
    assert!(SWITCHES_TOP + LINE * (SWITCHES.len() as i16 - 1) + TEXT <= HINT_TOP);
    assert!(HINT_TOP + TEXT <= BUTTONS_TOP);
    assert!(BUTTONS_TOP + BUTTON.1 <= DIALOG.1);
};

/// What the dialog is shown with and what it comes back with. One dialog per launch, and its
/// procedure runs on the thread that put it up, so there is nothing here to share with anyone.
static SETUP: Mutex<Option<Setup>> = Mutex::new(None);
static ANSWERS: Mutex<Option<Answers>> = Mutex::new(None);
/// Whether what closed the dialog came from the pad.
///
/// Because a pad here is orb's own doing — a dialog answers to none by itself — whether one worked
/// is a question about orb, and a launcher that does not say which hand answered cannot settle it.
static BY_PAD: AtomicBool = AtomicBool::new(false);

struct Setup {
    choices: Vec<Screen>,
    screen: Screen,
    /// Which language the dialog's own words are in — what `orb.yaml` says, or the machine's where it
    /// says nothing — and which item of the language row is therefore shown as chosen.
    ///
    /// **The words are settled when the dialog is built and do not follow the row being answered.**
    /// Every label is in the template, which the dialog manager has already read by the time anybody
    /// can choose anything; rebuilding the dialog under the hand that is answering it would be a
    /// window that vanishes and comes back mid-answer. What the row asks for is what the game and the
    /// next launch are in.
    language: Language,
    chosen_language: Option<Language>,
    switches: [bool; SWITCHES.len()],
    /// The game's own configuration file, which is where the pad's buttons are read from so that
    /// this dialog answers to the ones the game will.
    game_cfg: PathBuf,
}

/// Shows the settings and returns what was answered, or `None` when the dialog was closed
/// without starting the game.
///
/// `game_cfg` is the game's own configuration file, read for what its pad buttons mean.
pub fn ask(config: &Config, game_cfg: &Path) -> Result<Option<Answers>, Box<dyn Error>> {
    // Before anything is asked of the screen or put on it.
    ignore_scaling();
    BY_PAD.store(false, Ordering::Relaxed);
    let choices: Vec<Screen> = std::iter::once(Screen::Fullscreen)
        .chain(
            sizes(primary_monitor())
                .into_iter()
                .map(|(width, height)| Screen::Window { width, height }),
        )
        .collect();
    // Settled before the boxes, one of which starts from it.
    let language = config.language.unwrap_or_else(Language::of_the_machine);
    let mut switches = [false; SWITCHES.len()];
    for (shown, switch) in switches.iter_mut().zip(&SWITCHES) {
        *shown = (switch.shown)(config, language);
    }
    *SETUP.lock().unwrap() = Some(Setup {
        choices,
        screen: config.screen,
        language,
        chosen_language: config.language,
        switches,
        game_cfg: game_cfg.to_owned(),
    });
    *ANSWERS.lock().unwrap() = None;

    let font = unsafe { message_font() };
    let template = template(&font, language);
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    // Modal, and its own message loop: what a dialog is for. The value is whatever `EndDialog`
    // was given, and -1 is the dialog not having been created at all.
    let answered = unsafe {
        DialogBoxIndirectParamW(
            instance,
            template.as_ptr() as *const DLGTEMPLATE,
            std::ptr::null_mut(),
            Some(procedure),
            0,
        )
    };
    pad::stop();
    if answered < 0 {
        return Err("cannot create the settings dialog".into());
    }
    Ok(ANSWERS.lock().unwrap().take())
}

/// The dialog's own procedure, which unlike a window's says whether it handled the message
/// rather than what the answer was.
unsafe extern "system" fn procedure(
    dialog: HWND,
    message: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> isize {
    match message {
        WM_INITDIALOG => {
            let setup = SETUP.lock().unwrap();
            let Some(setup) = setup.as_ref() else {
                return 0;
            };
            // Started here rather than before the dialog, since there is no dialog to post to until
            // now.
            pad::watch(dialog, Mapping::read(&setup.game_cfg));
            let screen = unsafe { GetDlgItem(dialog, SCREEN_ID as i32) };
            for choice in &setup.choices {
                let text = screen_text(*choice, setup.language);
                unsafe { SendMessageW(screen, CB_ADDSTRING, 0, wide(&text).as_ptr() as LPARAM) };
            }
            // What the file says, if it is still a size this monitor can show; fullscreen
            // otherwise, which is the one choice always in the list.
            let selected = setup
                .choices
                .iter()
                .position(|choice| *choice == setup.screen)
                .unwrap_or(0);
            unsafe { SendMessageW(screen, CB_SETCURSEL, selected, 0) };

            let languages = unsafe { GetDlgItem(dialog, LANGUAGE_ID as i32) };
            for item in LANGUAGES {
                let text = wide(language_text(item, setup.language));
                unsafe { SendMessageW(languages, CB_ADDSTRING, 0, text.as_ptr() as LPARAM) };
            }
            // What the file says, which for a file that says nothing is the machine's own — the first
            // item, and the one every install starts on.
            let selected = LANGUAGES
                .iter()
                .position(|item| *item == setup.chosen_language)
                .unwrap_or(0);
            unsafe { SendMessageW(languages, CB_SETCURSEL, selected, 0) };
            for (index, on) in setup.switches.iter().enumerate() {
                if *on {
                    let switch = unsafe { GetDlgItem(dialog, (SWITCH_ID + index as u16) as i32) };
                    unsafe { SendMessageW(switch, BM_SETCHECK, CHECKED, 0) };
                }
            }
            // The dialog manager puts the focus on the first tab stop, which is the list of
            // sizes: the one thing here somebody is most likely to have come to change.
            1
        }
        // The low word is which control it came from, which for the two buttons is the answer.
        // Return and escape arrive here as those two, the dialog manager having turned them into
        // them, so there is nothing to do about either key.
        WM_COMMAND => {
            let id = (wparam & 0xffff) as i32;
            if id == IDOK {
                unsafe { take_answers(dialog) };
                unsafe { EndDialog(dialog, 1) };
                return 1;
            }
            if id == IDCANCEL {
                unsafe { EndDialog(dialog, 0) };
                return 1;
            }
            0
        }
        // The pad, turned into what the dialog manager and the controls already answer to. Nothing
        // about the controls knows a pad exists.
        pad::WM_PAD => {
            if let Some(push) = Push::from_wparam(wparam) {
                unsafe { pushed(dialog, push) };
            }
            1
        }
        _ => 0,
    }
}

/// Where the cursor is, which is not quite which control has the focus: the two buttons are one row
/// and left and right choose between them, the way a menu with two answers on one line works.
///
/// The row is worked out from whatever has the focus rather than remembered, so that a pad picks up
/// wherever a mouse or the tab key left off.
fn row(id: u16) -> usize {
    match id {
        SCREEN_ID => SCREEN_ROW,
        LANGUAGE_ID => LANGUAGE_ROW,
        _ if (SWITCH_ID..SWITCH_ID + SWITCHES.len() as u16).contains(&id) => {
            SWITCHES_ROW + (id - SWITCH_ID) as usize
        }
        _ => BUTTON_ROW,
    }
}

/// The rows, in the order the pad walks them: the two with a list on them, then one per switch, and
/// the two buttons sharing the last.
const SCREEN_ROW: usize = 0;
const LANGUAGE_ROW: usize = 1;
const SWITCHES_ROW: usize = 2;
const BUTTON_ROW: usize = SWITCHES_ROW + SWITCHES.len();

/// The list a row holds, where it holds one — the sizes and the languages — and `None` for a row that
/// is a switch or the buttons.
///
/// # Safety
/// `dialog` must be the settings dialog, on its own thread.
unsafe fn list_on(dialog: HWND, at: usize) -> Option<HWND> {
    let id = match at {
        SCREEN_ROW => SCREEN_ID,
        LANGUAGE_ROW => LANGUAGE_ID,
        _ => return None,
    };
    let list = unsafe { GetDlgItem(dialog, id as i32) };
    (!list.is_null()).then_some(list)
}

/// Whichever list is open, where one is.
///
/// # Safety
/// `dialog` must be the settings dialog, on its own thread.
unsafe fn dropped(dialog: HWND) -> Option<HWND> {
    [SCREEN_ROW, LANGUAGE_ROW].into_iter().find_map(|at| {
        let list = unsafe { list_on(dialog, at) }?;
        (unsafe { SendMessageW(list, CB_GETDROPPEDSTATE, 0, 0) } != 0).then_some(list)
    })
}

/// # Safety
/// `dialog` must be the settings dialog, on its own thread — which is where its messages arrive.
unsafe fn pushed(dialog: HWND, push: Push) {
    // A dropped list has the whole pad while it is open: up and down move inside it, and either
    // button closes it on whatever it is showing — which for a dropdown list is already the
    // selection, since moving inside one changes it.
    if let Some(open) = unsafe { dropped(dialog) } {
        match push {
            Push::Previous => unsafe { key(open, VK_UP) },
            Push::Next => unsafe { key(open, VK_DOWN) },
            Push::Decide | Push::Cancel => unsafe {
                SendMessageW(open, CB_SHOWDROPDOWN, 0, 0);
            },
            _ => {}
        }
        return;
    }

    let focused = unsafe { GetFocus() };
    let at = row(unsafe { GetDlgCtrlID(focused) } as u16);
    match push {
        Push::Previous | Push::Next => {
            let step = if push == Push::Next { 1 } else { -1 };
            let wanted = (at as isize + step).clamp(0, BUTTON_ROW as isize) as usize;
            unsafe { focus(dialog, wanted, focused) };
        }
        // Sideways is what a row holds more than one of: the two lists, and the two buttons. A switch
        // is on or off, and turning it over is what the decide button is for.
        Push::Less | Push::More => {
            let forward = push == Push::More;
            if let Some(list) = unsafe { list_on(dialog, at) } {
                unsafe { pick(list, forward) };
            } else if at == BUTTON_ROW {
                let other = if forward { IDCANCEL } else { IDOK };
                unsafe { focus_control(dialog, GetDlgItem(dialog, other)) };
            }
        }
        // The decide button does what the row it is on has to be done to it: a list is opened, a
        // switch is turned over, and a button is pressed.
        Push::Decide => match unsafe { list_on(dialog, at) } {
            Some(list) => unsafe {
                SendMessageW(list, CB_SHOWDROPDOWN, 1, 0);
            },
            None => unsafe {
                // Noted before the click, since a button's is what ends the dialog.
                BY_PAD.store(true, Ordering::Relaxed);
                SendMessageW(focused, BM_CLICK, 0, 0);
            },
        },
        Push::Cancel => unsafe {
            BY_PAD.store(true, Ordering::Relaxed);
            SendMessageW(dialog, WM_COMMAND, IDCANCEL as WPARAM, 0);
        },
    }
}

/// Puts the focus on a row, keeping whichever of the two buttons it was on if it is that row.
///
/// # Safety
/// `dialog` must be the settings dialog and `focused` what has the focus now.
unsafe fn focus(dialog: HWND, wanted: usize, focused: HWND) {
    let id = match wanted {
        SCREEN_ROW => SCREEN_ID as i32,
        LANGUAGE_ROW => LANGUAGE_ID as i32,
        row if row < BUTTON_ROW => (SWITCH_ID + (row - SWITCHES_ROW) as u16) as i32,
        // Arriving at the buttons lands on the one that starts the game, which is the answer this
        // dialog is usually given; going sideways from there reaches the other.
        _ if row(unsafe { GetDlgCtrlID(focused) } as u16) == BUTTON_ROW => {
            return;
        }
        _ => IDOK,
    };
    unsafe { focus_control(dialog, GetDlgItem(dialog, id)) };
}

/// # Safety
/// `dialog` must be the settings dialog and `control` one of its controls.
unsafe fn focus_control(dialog: HWND, control: HWND) {
    if control.is_null() {
        return;
    }
    // Through the dialog rather than `SetFocus`, so that the dialog's own idea of where it is —
    // which button is the default one, what tab does next — goes with it.
    unsafe { SendMessageW(dialog, WM_NEXTDLGCTL, control as WPARAM, 1) };
}

/// The next size along in the list, without opening it.
///
/// # Safety
/// `screen` must be the list of sizes.
unsafe fn pick(screen: HWND, forward: bool) {
    let count = unsafe { SendMessageW(screen, CB_GETCOUNT, 0, 0) };
    let at = unsafe { SendMessageW(screen, CB_GETCURSEL, 0, 0) };
    let step = if forward { 1 } else { -1 };
    let wanted = (at + step).clamp(0, (count - 1).max(0));
    unsafe { SendMessageW(screen, CB_SETCURSEL, wanted as WPARAM, 0) };
}

/// A key press and release, for a control that is listening for one — a dropped list moving through
/// itself is the game's own behaviour and not something to reimplement.
///
/// # Safety
/// `control` must be a live control on this thread.
unsafe fn key(control: HWND, key: VIRTUAL_KEY) {
    unsafe {
        SendMessageW(control, WM_KEYDOWN, key as WPARAM, 0);
        SendMessageW(control, WM_KEYUP, key as WPARAM, 0);
    }
}

/// Reads the controls into [`ANSWERS`] while they are still there to read: `EndDialog` takes the
/// dialog and its controls with it.
///
/// # Safety
/// `dialog` must be the settings dialog, with its controls alive.
unsafe fn take_answers(dialog: HWND) {
    let setup = SETUP.lock().unwrap();
    let Some(setup) = setup.as_ref() else {
        return;
    };
    let screen = unsafe { GetDlgItem(dialog, SCREEN_ID as i32) };
    let chosen = unsafe { SendMessageW(screen, CB_GETCURSEL, 0, 0) };
    let languages = unsafe { GetDlgItem(dialog, LANGUAGE_ID as i32) };
    let chosen_language = unsafe { SendMessageW(languages, CB_GETCURSEL, 0, 0) };
    let mut answers = Answers {
        screen: setup
            .choices
            .get(chosen.max(0) as usize)
            .copied()
            .unwrap_or(Screen::Fullscreen),
        // Read back off its switch like every other, which is what the pair in `Switch` is for.
        thcrap: false,
        // A selection this row does not hold is the machine's own, which is the item nobody has to
        // choose and the one a file that says nothing means.
        language: LANGUAGES
            .get(chosen_language.max(0) as usize)
            .copied()
            .unwrap_or(None),
        always_draw: false,
        boundary_flash: false,
        skip_ending: false,
        hide_mouse: false,
        dpad_moves: false,
        ask_at_startup: false,
    };
    for (index, switch) in SWITCHES.iter().enumerate() {
        let control = unsafe { GetDlgItem(dialog, (SWITCH_ID + index as u16) as i32) };
        let state = unsafe { SendMessageW(control, BM_GETCHECK, 0, 0) };
        (switch.answered)(&mut answers, state == CHECKED as isize);
    }
    *ANSWERS.lock().unwrap() = Some(answers);
}

/// The face and point size this host writes windows in, asked of it rather than chosen here: a face
/// named in the source is a face missing on somebody's machine, and this text is Japanese.
struct Font {
    face: String,
    points: u16,
}

/// # Safety
/// Nothing; it only asks the system.
unsafe fn message_font() -> Font {
    /// What the dialog manager falls back to, and what every Windows dialog was written in before
    /// there was anything to ask. The system maps it to the face this locale wants.
    const FALLBACK: (&str, u16) = ("MS Shell Dlg", 9);

    let mut metrics: NONCLIENTMETRICSW = unsafe { std::mem::zeroed() };
    metrics.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    let told = unsafe {
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            &mut metrics as *mut NONCLIENTMETRICSW as *mut c_void,
            0,
        )
    };
    if told == 0 {
        return Font {
            face: FALLBACK.0.to_owned(),
            points: FALLBACK.1,
        };
    }
    let face = &metrics.lfMessageFont.lfFaceName;
    let end = face
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(face.len());
    let face = String::from_utf16_lossy(&face[..end]);
    // A template says the size in points, and `lfHeight` is in this dpi's pixels — negative for
    // the character height, which is what a point size means.
    let height = metrics.lfMessageFont.lfHeight.unsigned_abs();
    let points = (height * 72 / system_dpi().max(1) as u32).clamp(6, 72) as u16;
    if face.is_empty() || height == 0 {
        return Font {
            face: FALLBACK.0.to_owned(),
            points: FALLBACK.1,
        };
    }
    Font { face, points }
}

/// How many pixels the system puts in an inch, which is what turns the font's height into the
/// points a template says it in. Asked of a screen device context rather than through
/// `GetDpiForSystem`, which is Windows 10 and later; this has been there throughout.
fn system_dpi() -> i32 {
    let screen = unsafe { GetDC(std::ptr::null_mut()) };
    if screen.is_null() {
        return 96;
    }
    let dpi = unsafe { GetDeviceCaps(screen, LOGPIXELSY as i32) };
    unsafe { ReleaseDC(std::ptr::null_mut(), screen) };
    if dpi > 0 { dpi } else { 96 }
}

/// The dialog template, as the bytes `DialogBoxIndirectParamW` reads.
///
/// A `Vec<u32>` rather than a `Vec<u8>` because the template has to start on a four-byte boundary
/// and so does every control in it, and a byte vector's buffer promises neither.
fn template(font: &Font, language: Language) -> Vec<u32> {
    let label = label_width(language);
    let mut bytes = Vec::new();
    let items: Vec<Item> = std::iter::once(Item {
        class: ATOM_STATIC,
        style: WS_CHILD | WS_VISIBLE | WS_GROUP,
        id: 0,
        at: (MARGIN.0, MARGIN.1 + 2, label, TEXT),
        text: screen_label(language).to_owned(),
    })
    .chain(std::iter::once(Item {
        class: ATOM_STATIC,
        style: WS_CHILD | WS_VISIBLE,
        id: 0,
        at: (MARGIN.0, LANGUAGE_TOP + 2, label, TEXT),
        text: language_label(language).to_owned(),
    }))
    // Said, because a dialog that answers to a pad is not something anybody expects one to do, and
    // what its buttons do here is worth a line even to somebody who guesses that they do anything.
    // The line used to say `A で決定`, and on the pad this was written for decide is button 0 or 1 —
    // see [`pad_hint`] for why it names neither.
    //
    // Left and right are left out although they do something on three of the rows — the two lists, and
    // moving between the two buttons — because they do nothing on any of the switches, and a line
    // that is wrong for every switch in the dialog is worse than one thing fewer to read. The
    // decide button is what turns a switch over, and that is said.
    .chain(std::iter::once(Item {
        class: ATOM_STATIC,
        style: WS_CHILD | WS_VISIBLE,
        id: 0,
        at: (MARGIN.0, HINT_TOP, DIALOG.0 - MARGIN.0 * 2, TEXT),
        text: pad_hint(language).to_owned(),
    }))
    .chain([SCREEN_ID, LANGUAGE_ID].into_iter().map(move |id| Item {
        class: ATOM_COMBOBOX,
        style: WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST as u32,
        id,
        at: (
            MARGIN.0 + label,
            if id == SCREEN_ID {
                MARGIN.1
            } else {
                LANGUAGE_TOP
            },
            DIALOG.0 - MARGIN.0 * 2 - label,
            COMBO,
        ),
        text: String::new(),
    }))
    .chain(SWITCHES.iter().enumerate().map(|(index, switch)| Item {
        class: ATOM_BUTTON,
        style: WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX as u32,
        id: SWITCH_ID + index as u16,
        at: (
            MARGIN.0,
            SWITCHES_TOP + LINE * index as i16,
            DIALOG.0 - MARGIN.0 * 2,
            TEXT,
        ),
        text: (switch.text)(language).to_owned(),
    }))
    .chain([
        Item {
            class: ATOM_BUTTON,
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON as u32,
            id: IDOK as u16,
            at: (
                DIALOG.0 - MARGIN.0 - BUTTON.0 * 2 - 4,
                BUTTONS_TOP,
                BUTTON.0,
                BUTTON.1,
            ),
            text: start_text(language).to_owned(),
        },
        Item {
            class: ATOM_BUTTON,
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            id: IDCANCEL as u16,
            at: (
                DIALOG.0 - MARGIN.0 - BUTTON.0,
                BUTTONS_TOP,
                BUTTON.0,
                BUTTON.1,
            ),
            text: quit_text(language).to_owned(),
        },
    ])
    .collect();

    push32(
        &mut bytes,
        DS_SETFONT | DS_MODALFRAME | DS_CENTER | WS_POPUP | WS_CAPTION | WS_SYSMENU,
    );
    push32(&mut bytes, 0);
    push16(&mut bytes, items.len() as u16);
    for value in [0, 0, DIALOG.0, DIALOG.1] {
        push16(&mut bytes, value as u16);
    }
    // No menu and no class of its own, which is what makes it `#32770`.
    push16(&mut bytes, 0);
    push16(&mut bytes, 0);
    push_text(&mut bytes, "Ultramarine Orb Elixir");
    push16(&mut bytes, font.points);
    push_text(&mut bytes, &font.face);

    for item in &items {
        align(&mut bytes);
        push32(&mut bytes, item.style);
        push32(&mut bytes, 0);
        for value in [item.at.0, item.at.1, item.at.2, item.at.3] {
            push16(&mut bytes, value as u16);
        }
        push16(&mut bytes, item.id);
        // The class by ordinal, which is the shorter of the two ways a template may name it.
        push16(&mut bytes, 0xffff);
        push16(&mut bytes, item.class);
        push_text(&mut bytes, &item.text);
        // No creation data.
        push16(&mut bytes, 0);
    }

    align(&mut bytes);
    bytes
        .chunks(4)
        .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
        .collect()
}

/// One control of the template: everything about it that differs from the others.
struct Item {
    class: u16,
    style: u32,
    id: u16,
    /// Left, top, width and height, in dialog units.
    at: (i16, i16, i16, i16),
    text: String,
}

fn push16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// A template's strings are UTF-16 and terminated, and an empty one is the terminator alone.
fn push_text(bytes: &mut Vec<u8>, text: &str) {
    for unit in text.encode_utf16().chain(std::iter::once(0)) {
        push16(bytes, unit);
    }
}

fn align(bytes: &mut Vec<u8>) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
}

/// `4:3` or `16:9`, so that a size in the list says which of the two it is without anybody
/// dividing it out.
fn ratio(width: u32, height: u32) -> String {
    let divisor = gcd(width, height);
    format!("{}:{}", width / divisor, height / divisor)
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a.max(1) } else { gcd(b, a % b) }
}

/// A NUL-terminated wide string for Win32, alive as long as the caller keeps it.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ATOM_BUTTON, Answers, DIALOG, Font, LANGUAGES, SWITCHES, language_text, ratio, sizes,
        template,
    };
    use orb_config::{Config, Language};

    /// The language the dialog below is built in. Both, wherever what is being asked could come out
    /// differently in one of them — the template is laid out from the words in it.
    const LANGUAGES_TO_READ: [Language; 2] = [Language::Japanese, Language::English];

    /// Every key at what a file that is not there gives it, which is on for every switch — and no
    /// directory is made, the read of a file that is not there being the whole of what this needs.
    fn defaults() -> Config {
        let missing = std::env::temp_dir().join(format!("orb-no-settings-{}", std::process::id()));
        Config::load_beside(&missing.join("orb.exe")).expect("the defaults")
    }

    /// Each switch answers the key it shows.
    ///
    /// Which is what the pair in `Switch` is for: a `shown` and an `answered` naming two different keys
    /// would be a dialog showing one setting and writing another, and nothing about the dialog itself
    /// would look wrong.
    #[test]
    fn each_switch_answers_the_key_it_shows() {
        /// The language the switches are named in for what this test says about them — and the one the
        /// patch switch's own default is read against, that being the switch whose default the
        /// machine's language decides. English is the side of it that starts on, which is what lets
        /// every box below be asserted the same way.
        const IN: Language = Language::English;

        let shown = defaults();
        for switch in &SWITCHES {
            assert!(
                (switch.shown)(&shown, IN),
                "{}: not shown as on with every key at its default",
                (switch.text)(IN),
            );
        }
        for (index, switch) in SWITCHES.iter().enumerate() {
            // Answered the way the dialog reads its controls: an `Answers` of nothing, and one call
            // per switch — this one off and every other on.
            let mut answers = Answers {
                screen: shown.screen,
                language: shown.language,
                thcrap: false,
                always_draw: false,
                boundary_flash: false,
                skip_ending: false,
                hide_mouse: false,
                dpad_moves: false,
                ask_at_startup: false,
            };
            for (other, answering) in SWITCHES.iter().enumerate() {
                (answering.answered)(&mut answers, other != index);
            }
            let mut config = defaults();
            answers.apply(&mut config);

            let off: Vec<&str> = SWITCHES
                .iter()
                .filter(|switch| !(switch.shown)(&config, IN))
                .map(|switch| (switch.text)(IN))
                .collect();
            assert_eq!(
                off,
                [(switch.text)(IN)],
                "{}: the key that came out off is not the one this switch shows",
                (switch.text)(IN),
            );
        }
    }

    /// The language row offers the machine's own first and then the two languages, each named in
    /// itself: somebody looking for English is looking for the word `English`, whichever language the
    /// dialog they are reading is in.
    #[test]
    fn the_language_row_offers_the_machines_own_first_and_names_each_language_in_itself() {
        assert_eq!(LANGUAGES[0], None);
        for language in LANGUAGES_TO_READ {
            assert_eq!(language_text(Some(Language::Japanese), language), "日本語");
            assert_eq!(language_text(Some(Language::English), language), "English");
            // And the machine's own is the one item this dialog says in its own words.
            assert_ne!(
                language_text(None, Language::Japanese),
                language_text(None, Language::English),
            );
        }
    }

    /// Every answer the dialog can be given is a value `orb.yaml` holds, the language included: an
    /// answer this row could give that the file could not keep would be a setting somebody chose and
    /// does not get.
    #[test]
    fn every_language_the_row_offers_is_one_the_file_keeps() {
        for item in LANGUAGES {
            let mut config = defaults();
            let answers = Answers {
                screen: config.screen,
                language: item,
                thcrap: true,
                always_draw: true,
                boundary_flash: true,
                skip_ending: true,
                hide_mouse: true,
                dpad_moves: true,
                ask_at_startup: true,
            };
            answers.apply(&mut config);
            assert_eq!(config.language, item);
        }
    }

    /// 16:9 above 4:3 and the biggest of each first, since that is the order somebody reads a
    /// list of sizes in. A 16:9 window is the one that leaves black down the sides for orb's
    /// numbers; a 4:3 one leaves none.
    #[test]
    fn a_full_hd_monitor_is_offered_what_fits_on_it() {
        assert_eq!(
            sizes((1920, 1080)),
            vec![(1280, 720), (960, 720), (640, 480)]
        );
    }

    /// Biggest first within each ratio, and the game's own size last of all. A 4K monitor is
    /// offered no 4K window, of either ratio: a window as tall as the screen has nowhere to put
    /// its caption, which is what fullscreen is for.
    #[test]
    fn the_list_runs_from_the_biggest_down() {
        assert_eq!(
            sizes((3840, 2160)),
            vec![
                (2560, 1440),
                (1920, 1080),
                (1280, 720),
                (1920, 1440),
                (1440, 1080),
                (960, 720),
                (640, 480),
            ]
        );
    }

    /// Every monitor is offered the size the game renders at, which is the one that scales
    /// nothing.
    #[test]
    fn every_monitor_is_offered_the_games_own_size() {
        assert_eq!(sizes((640, 480)), vec![(640, 480)]);
        assert_eq!(sizes((3840, 2160)).last(), Some(&(640, 480)));
    }

    #[test]
    fn a_size_says_which_ratio_it_is() {
        assert_eq!(ratio(1280, 720), "16:9");
        assert_eq!(ratio(640, 480), "4:3");
        assert_eq!(ratio(2880, 2160), "4:3");
    }

    /// The header says how many controls follow and each of them starts on a four-byte boundary,
    /// which is what the dialog manager walks the template by: get either wrong and it reads a
    /// control out of the middle of the one before it.
    #[test]
    fn the_template_says_what_it_holds() {
        for language in LANGUAGES_TO_READ {
            let built = template(
                &Font {
                    face: "Segoe UI".to_owned(),
                    points: 9,
                },
                language,
            );
            let bytes: Vec<u8> = built.iter().flat_map(|word| word.to_le_bytes()).collect();
            // style, exstyle, then the count.
            let count = u16::from_le_bytes([bytes[8], bytes[9]]);
            // Three labels, the two lists, the switches, and the two buttons.
            assert_eq!(count as usize, SWITCHES.len() + 7);
            // The size the header asks for, which is in dialog units and not pixels. The same either
            // way: what the language changes is the column the labels are written in, inside it.
            let cx = i16::from_le_bytes([bytes[14], bytes[15]]);
            assert_eq!(cx, DIALOG.0);
            // Every control names its class by ordinal, and one of them is a button.
            assert!(
                bytes
                    .windows(4)
                    .any(|four| four[0..2] == 0xffff_u16.to_le_bytes()
                        && four[2..4] == ATOM_BUTTON.to_le_bytes())
            );
        }

        // Everything the pad walks through is on a row of its own, and the two buttons share the
        // last one: `row` maps the controls onto that, and nothing may fall off the end.
        assert_eq!(super::row(super::SCREEN_ID), super::SCREEN_ROW);
        assert_eq!(super::row(super::LANGUAGE_ID), super::LANGUAGE_ROW);
        assert_eq!(super::row(super::SWITCH_ID), super::SWITCHES_ROW);
        assert_eq!(
            super::row(super::SWITCH_ID + SWITCHES.len() as u16 - 1),
            super::BUTTON_ROW - 1
        );
        assert_eq!(super::row(super::IDOK as u16), super::BUTTON_ROW);
        assert_eq!(super::row(super::IDCANCEL as u16), super::BUTTON_ROW);
    }
}
