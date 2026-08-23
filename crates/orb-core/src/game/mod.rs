//! The seam between orb and the game it is running inside.
//!
//! Everything above this module — chapters, snapshots, the retry menu, the
//! overlay — is written against [`Game`] and [`State`] rather than against
//! 東方紅魔郷. The point of the seam is that porting to another Touhou game means
//! writing addresses and offsets, not reworking how chapters or retries behave.
//!
//! Two implement it. [`th06`] answers everything, and [`th07`] answers what has
//! been measured of 妖々夢 — a frame, and nothing about a run — and declines the
//! rest, which is how the list of what a second game still needs is found rather
//! than guessed at. So orb paces 妖々夢 and plays 紅魔郷. Which of them a process
//! is is [`KNOWN`].

/// Where to call one of the game's own functions, which is its address in a real process and whatever a
/// game laid out by hand hands over in its place: a reader, a setter, and the slot behind them.
///
/// **Code is the one thing an address space laid out by hand cannot hold**, so every call a game in here
/// makes into itself that an e2e test reaches has to be answerable — a shake still running at a stage
/// move is taken down through one of 紅魔郷's, and the queue of quads a frame of 妖々夢's drawing fills is
/// emptied and drawn through two more. An e2e test without them would be jumping into memory nothing has
/// mapped. The same answer `window::install_over` and `score::install_over` are — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
///
/// Here rather than in either game because both games have such calls, and a macro apiece is one of them
/// drifting. The slot is behind the same gate the laid-out images are, so the shipped DLL has the
/// constant and no atomic in the path. The setter is not `install_`, which everything wearing that name
/// in this tree does by writing over something — an import table entry or a prologue: this patches
/// nothing and records where a function is.
macro_rules! handed_over {
    ($slot:ident, $at:ident, $set:ident, $real:ident) => {
        #[cfg(any(test, feature = "sim"))]
        static $slot: AtomicUsize = AtomicUsize::new($real);

        #[cfg(any(test, feature = "sim"))]
        fn $at() -> usize {
            $slot.load(Ordering::Relaxed)
        }

        #[cfg(not(any(test, feature = "sim")))]
        fn $at() -> usize {
            $real
        }

        #[cfg(any(test, feature = "sim"))]
        fn $set(address: usize) {
            $slot.store(address, Ordering::Relaxed);
        }
    };
}

pub mod th06;
pub mod th07;

use std::ffi::c_void;
use std::fmt;
use std::ops::Range;

use orb_api::{Device, Hwnd, Texture};

use crate::audio::Music;

/// A game orb knows how to run inside: what its exe is called, the builds of it every address in it
/// was read off, and the [`Game`] those addresses are in.
///
/// Here rather than a constant apiece in the DLL and in the launcher, so that the two cannot
/// disagree about which games exist: the launcher starts nothing it finds no entry for, and orb
/// inside a process no entry names does nothing at all. See `docs/adr/0004`.
pub struct Known {
    /// The exe's own file name, which is what the process orb wakes up inside is recognised by:
    /// nothing else of a game is readable before the game it is has been settled.
    pub exe: &'static str,
    /// What the game keeps its own configuration in, which the launcher reads for one thing — which
    /// pad button the game takes as shoot and which as bomb, so the settings dialog answers to the
    /// same two.
    pub cfg: &'static str,
    /// The builds this entry's addresses hold for, newest first, and a launch is refused any exe that
    /// is none of them.
    pub builds: &'static [Build],
    pub game: &'static dyn Game,
}

/// One build of a game orb has addresses for: the md5 of that exe, and what the build is called.
///
/// **A list rather than the one build, because a translation patch makes another one.** orb reads the
/// game's state through absolute addresses, so what an entry has to hold is every exe those addresses
/// are right for — and an exe with the game's text swapped out is a different file with the same code
/// in it. Each one here is a build somebody has had orb's addresses read against; what is not here is
/// refused, whatever it says about itself.
pub struct Build {
    /// The md5 of the exe.
    ///
    /// Checked by the launcher and not at the attach: a run has no reason to read six hundred
    /// kilobytes back off the disk to learn what it is already inside, and a game started some other
    /// way is one nobody checked the exe of either.
    pub md5: &'static str,
    /// What the build is called where anybody says it out loud, so that a refusal and a log line name
    /// something somebody can go and look for.
    pub version: &'static str,
}

/// Every game orb knows, in the order a directory is searched for one.
///
/// A `const` rather than a `static` because a `static` of this type would want `Game: Sync`, and what
/// this table is read for on either side is one entry copied out of it once.
pub const KNOWN: &[Known] = &[
    Known {
        exe: "東方紅魔郷.exe",
        cfg: "東方紅魔郷.cfg",
        builds: &[Build {
            md5: "fa3d64768b1bfc50703dedc2db92f7fa",
            version: "1.02h",
        }],
        game: &th06::Th06,
    },
    // A game orb paces and does not play: [`th07::Th07`] answers what has been read of one frame and
    // declines everything about a run, so a launch here gets its window sized and orb's own frame in place
    // of the game's, and no overlay, no chapters and its own score file. What version this build is the
    // file name does not say and nothing in it has been read that does, so the md5 is the whole of what
    // pins it — which is what the md5 is for in the other entry too.
    Known {
        exe: "th07.exe",
        cfg: "th07.cfg",
        builds: &[Build {
            md5: "0126afce1e805370d36c3482445e98da",
            version: "the build of md5 0126afce",
        }],
        game: &th07::Th07,
    },
];

/// The game an exe of that name is, or `None` for a name no entry holds.
///
/// The ASCII of it case-insensitively, these being Windows file names: an exe copied out as
/// `TH06.EXE` is the same game. The rest matches as written, a name in kanji having no case to fold.
pub fn known_by_exe(exe: &str) -> Option<&'static Known> {
    KNOWN
        .iter()
        .find(|known| known.exe.eq_ignore_ascii_case(exe))
}

/// The game a process running an exe of that name is, said out loud — and `None` for a name no entry
/// holds, with the refusal that names every build orb has addresses for.
///
/// **The two log lines are here rather than at the call site**, because there are two call sites and
/// what they say has to be the same: `orb::attach` reads the name off the exe it woke up inside, and a
/// game laid out by hand is running under a name of its own. A process orb has no addresses for is one
/// it must leave exactly as it found it, so this is the line before which nothing of the host has been
/// touched — see `docs/adr/0004`.
pub fn found(exe: &str) -> Option<&'static Known> {
    let Some(known) = known_by_exe(exe) else {
        crate::log!(
            "game: nothing orb knows is called {exe}; it knows {}. orb is doing nothing this run",
            known_named(),
        );
        return None;
    };
    crate::log!(
        "game: {exe}, and every address orb has for it was read off {}",
        known.builds_named()
    );
    Some(known)
}

impl Known {
    /// The builds of this game, named the way a refusal and a log line name them.
    ///
    /// `or` between them rather than a comma, because that is what a list of them is: one exe is one
    /// of these builds, and which one the DLL cannot say — the md5 is the launcher's to read, and a
    /// process is past the moment for it.
    pub fn builds_named(&self) -> String {
        self.builds
            .iter()
            .map(|build| build.version)
            .collect::<Vec<_>>()
            .join(" or ")
    }
}

/// The games orb knows, named the way a refusal has to name them: the exe to look for and the builds
/// of it its addresses were read off.
///
/// One spelling for both sides, so that what the launcher prints and what the log says of a process
/// orb does nothing in are the same list.
pub fn known_named() -> String {
    KNOWN
        .iter()
        .map(|known| format!("{} {}", known.exe, known.builds_named()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A function to hook and the bytes expected at it, so a mismatch is caught
/// rather than relocating an instruction that cannot be relocated.
pub struct Patch {
    pub target: usize,
    pub prologue: &'static [u8],
}

/// A function of the game's own that orb calls, and what to call it on.
///
/// `__thiscall` with a single argument is `fastcall` with nothing on the stack, which is an ABI Rust
/// can spell — the note `run_calc_chain` in the `orb` crate already carries.
#[derive(Clone, Copy)]
pub struct Call {
    pub function: usize,
    pub this: usize,
}

/// The two calls into the game that orb's own frame loop makes, the loop it replaced having made
/// them itself.
///
/// Addresses rather than two methods of [`Game`], because that is what the rest of this seam is:
/// [`Game::chain`] is an address, a [`Patch`] carries a target, and the update and draw the frame
/// loop runs are addresses orb was handed and stored. Porting is then a number for these two as it
/// is for everything else, instead of the transmute written again — and it is what lets a game that
/// is not a real process be driven through the frame loop at all, an address space laid out by hand
/// answering reads rather than execution.
pub struct FrameCalls {
    /// Hands queued sound effects to the sound system, as the game does once per frame after its
    /// update. Runs on the game's main thread.
    pub play_sounds: Call,
    /// The game's own present, which also handles a lost device and its own screenshot key. Runs on
    /// the game's main thread with the frame drawn.
    pub present: Call,
}

pub struct Hooks {
    /// One call per logic frame. Not calling through freezes the game.
    pub update: Patch,
    /// Called inside the scene the game draws into, the only place an overlay
    /// may draw.
    pub draw: Patch,
    /// Writes a replay file. `None` if the game has nothing to suppress.
    pub save_replay: Option<Patch>,
    /// Finishes off the record of inputs a run is being recorded into. `None` if the
    /// game does not also run it while playing a replay back, where the record it
    /// would write into is the replay's own.
    pub stop_recording: Option<Patch>,
    /// Runs where the game has just put a stage's numbers in place and before that
    /// stage's first update: the one moment a run's state can be written without
    /// something of the stage having already been decided from the state it replaces.
    ///
    /// `None` for a game with no such point, which leaves a run unable to be picked up
    /// again — see `resume` in the `orb` crate.
    pub stage_begun: Option<Entered>,
    /// Runs after those numbers are in place and before the stage is built out of them,
    /// which is the last moment nothing of the stage has been drawn from the generator
    /// yet: the only place a resumed run's seed can go.
    ///
    /// A separate seam from [`stage_begun`](Hooks::stage_begun) because the two moments are
    /// not the same one, and what the seed has to be in before is this one.
    ///
    /// `None` where the game has no such call, which costs a resumed run the seed and with
    /// it every enemy the seed decides.
    pub stage_building: Option<Patch>,
    /// Runs around the read of the score file that the front end's own unlocks are taken out of —
    /// which stages it offers, whether it offers the Extra stage — and around no other read of it.
    ///
    /// What it buys is one exception to the mode's file. A pointdevice run's ranking and the record
    /// it keeps beside it are the mode's own, but what the front end offers is not a record of
    /// anything: a stage that has been reached has been reached, and a mode whose file is new would
    /// otherwise start with the game locked back to stage 1.
    ///
    /// `None` for a game whose unlocks come out of a read that cannot be told from the rest, which
    /// leaves them following the mode like everything else in the file.
    pub unlocks_read: Option<Entered>,
    /// Runs as a ranking is about to be read out of the score file, before the read.
    ///
    /// Where [`Game::forget_captures`] goes, and nowhere else: the record it clears is the one that
    /// read is about to fill.
    ///
    /// `None` for a game whose ranking read cannot be got in front of, which leaves the captures in
    /// memory carrying from one file's ranking into the next one's.
    pub ranking_read: Option<Entered>,
    /// Runs where the game lays the records its score file holds over the ones in memory, **called
    /// through**: what the file held stands, and [`Game::add_the_cards_a_session_counted`] puts a
    /// session's own counts on top of it.
    ///
    /// The one seam between a save's two halves, which is why it is a seam at all: the save reads the
    /// file, lays its records over memory here, and then writes memory back out — so what memory holds
    /// at *this* moment is the whole of what reaches the file, and a session's counts have nowhere else
    /// to be put. Entered `(the file's image, the array)` in `ecx` and `edx`.
    ///
    /// `None` for a game whose score file is written as its ranking screen goes down, which is 紅魔郷:
    /// there the record in memory is what the write takes and nothing lays a file over it first.
    pub card_records_read: Option<Patch>,
    /// Runs after the config is read and before the window and device exist, which
    /// is the only moment `force_windowed` can still take effect.
    pub create_window: Option<Patch>,
    /// Runs just after the device is created, and again after every reset: the
    /// first moment its vtable can be redirected.
    pub init_device: Option<Patch>,
    /// The game's whole frame: its draw, its update, its pacing and its present.
    /// Replacing it is what lets orb pace the game and drop a frame of input lag.
    pub render: Option<Patch>,
    /// Everything the game reads from the keyboard, once a frame. Needed because
    /// orb keeps updating while the window is not in front, and a game that carried
    /// on reading the keyboard then would act on keys meant for something else.
    pub input: Option<Patch>,
    /// The part of the input read that goes to a joystick, if the game keeps it separate. Hooked and
    /// **not called through**: what every pad does to the word is [`Game::pad_word`], so a launch that
    /// left this out is a launch where no pad does anything. `None` for a game whose pad read has not
    /// been measured, which is a game orb reads no pad in at all.
    pub joystick: Option<Entered>,
    /// The pad's buttons one byte apiece, which is what a game's own key config screen reads to learn
    /// *which* button was pressed rather than what it means. Hooked and **called through**: what the game
    /// found stands, and [`Game::add_pad_buttons`] adds the pads orb read for itself.
    ///
    /// A read of its own because a mapping screen cannot use a mapped word — it is the screen the mapping
    /// is made on. `None` for a game with no such screen, which is 紅魔郷: its buttons are configured by
    /// `custom.exe` beside it and nothing in the game reads a pad by number.
    pub pad_buttons: Option<Patch>,
}

/// A patch, and where the argument of the function under it arrives.
///
/// **Because a hook is entered by its ABI and nothing else**, and MSVC compiled the same call differently
/// in the two games. The pad read is the pair this was first needed for: 紅魔郷's
/// `Controller::GetControllerInput` takes the word the keyboard was read into on the stack — `push edx;
/// call 0x41cfc0; add esp,4` at 0x41dc77 — and 妖々夢's takes it in `ecx`, `mov ecx,[ebp-0x104]; call
/// 0x4303f0` at 0x430f1a. A stage's own beginning is the second: 紅魔郷's added callback is `__cdecl` and
/// 妖々夢's is `__thiscall`, its chain calling every callback as `mov ecx,[elem+0x1c]; call [elem+0x4]`.
/// Entered the wrong way, a hook that calls the original through reads its argument out of whatever the
/// register or the stack happened to hold, which is a crash and not a wrong number.
///
/// Here rather than in a method of [`Game`], because it is one fact about one patch and this is where the
/// patch is — a second answer somewhere else is a second place to keep in step.
pub enum Entered {
    OnTheStack(Patch),
    InEcx(Patch),
}

impl Entered {
    /// The patch either of them carries, for the installer that has already decided which hook goes over
    /// it.
    pub fn patch(&self) -> &Patch {
        match self {
            Self::OnTheStack(patch) | Self::InEcx(patch) => patch,
        }
    }
}

/// What the game's own front end is being asked for.
///
/// Only the two things orb has a question of its own to put over: a run is one thing in pointdevice
/// mode and another in normal mode, and so is a ranking. Everything else the front end can be asked
/// for is nothing to orb, and [`Game::menu_pointed_at`] answers `None` for it.
///
/// What the cursor is on rather than what the game has acted on, so that the question can be asked on
/// the press: every game reads a decide before it acts on one, and none of them can be persuaded to
/// un-act.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Menu {
    /// A run: practice as well as a full run and the Extra stage, each being one thing with chapters
    /// and another without.
    Run,
    /// The ranking, of which there is one per mode.
    Scores,
}

/// One pad as `joyGetPosEx` reports one — which are the numbers the game's own read of a pad works in,
/// and an XInput pad is translated into them rather than into a decision.
///
/// Here beside [`Pad`] rather than with the thread that samples it, because it is what a `Game` is
/// handed: the sampling is orb's own business and the mapping of these numbers onto a decision is
/// the game's, and this is the boundary between them.
///
/// **Both axes**, because a pad the game has no device for is one orb stands in for the whole read of:
/// a menu of orb's is a list and up and down are the whole of what it needs, and a player moves in
/// four directions.
#[derive(Clone, Copy)]
pub struct Reading {
    pub buttons: u32,
    pub x: Axis,
    pub y: Axis,
    /// Where the hat — the d-pad — points, which is its own field and not the axes.
    pub pov: u32,
}

/// Where one axis of a pad is, and the travel it is measured against.
///
/// **The bounds travel with the position** because the pads orb reads are no longer only the one the
/// game measures for itself: `GetControllerInput` places the centre of an axis halfway between the
/// bounds in the `JOYCAPSA` at 0x69d760, which is joystick 0's, so every other pad measured against
/// them would be measured against a device it is not.
#[derive(Clone, Copy)]
pub struct Axis {
    pub at: u32,
    pub min: u32,
    pub max: u32,
}

/// What a pad is doing, in the terms a menu of orb's needs.
///
/// Asked of the game because every one of these is its own mapping: which button decides and which
/// cancels, and how far a stick goes before it counts. Answered from a reading orb took itself,
/// because a menu of orb's has the game frozen and the game is not reading the pad on those frames.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Pad {
    pub up: bool,
    pub down: bool,
    pub decide: bool,
    pub cancel: bool,
    /// The other two directions, and the game's own four buttons. **What a judging pass is driven with**:
    /// somebody watching a stage go by has a controller in their hands, so every key that pass uses has to
    /// be one of these — see `runtime`'s `keys`.
    ///
    /// Named as the game's own buttons rather than as what orb does with them, because that is what they
    /// are: which of them orb reads as a step and which as the key held with one is orb's business and is
    /// written where those keys are.
    pub left: bool,
    pub right: bool,
    pub shot: bool,
    pub bomb: bool,
    pub focus: bool,
    pub skip: bool,
}

impl Pad {
    /// Nothing pushed, which is what a pad orb has not read yet is: `Default` in a `const`.
    pub const fn none() -> Self {
        Self {
            up: false,
            down: false,
            decide: false,
            cancel: false,
            left: false,
            right: false,
            shot: false,
            bomb: false,
            focus: false,
            skip: false,
        }
    }

    /// Two pads' worth of pushing as one. 紅魔郷 is a one-player game, so every pad on the machine is
    /// that one player's and which of them a push came from is nothing a menu has any use for.
    pub fn or(self, other: Self) -> Self {
        Self {
            up: self.up || other.up,
            down: self.down || other.down,
            decide: self.decide || other.decide,
            cancel: self.cancel || other.cancel,
            left: self.left || other.left,
            right: self.right || other.right,
            shot: self.shot || other.shot,
            bomb: self.bomb || other.bomb,
            focus: self.focus || other.focus,
            skip: self.skip || other.skip,
        }
    }
}

/// The tile the game paints its status panel's background with: a texture it has already loaded,
/// the piece of that texture which is one tile, and the grid the tiles are laid on.
///
/// Asked for rather than carried, so that orb ships no art and a modified sheet is honoured —
/// and so that what is left on the screen where orb stops drawing is the panel the game would
/// have painted there anyway.
pub struct PanelTile {
    pub texture: Texture,
    /// The piece of the sheet, as `[left, top, right, bottom]` in texture coordinates.
    pub uv: [f32; 4],
    /// Where the grid starts in the game's output, and how far apart its lines are.
    pub origin: (f32, f32),
    pub pitch: f32,
}

/// Which run this is: everything the game's own front end is asked for before one starts.
///
/// What says whether a chapter written down belongs to the run about to be played. A run of another
/// character is another run — its buttons would play somebody else's shot — which is why 紺珠伝 keeps
/// a pointdevice save per difficulty and character, and why orb keeps one per [`Game::run_slot`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RunStart {
    pub difficulty: i32,
    /// Whose run it is, and which of that character's two shots. Two numbers because that is
    /// how the game keeps them, and every table it indexes by them is `character * 2 + shot`.
    pub character: i32,
    pub shot_type: i32,
    /// One stage rather than a whole run.
    pub practice: bool,
    /// Counted from zero, the way orb counts stages everywhere.
    pub stage: i32,
}

/// **There is no shared shape for the run's numbers.** What a stage has to start with is the game's own —
/// see [`Game::run_state`], which hands them over as that game's bytes, and the two games' own
/// `resume` modules, where each says what its own file holds.
///
/// It was one struct, with 紅魔郷's rank, power items and extra lives beside 妖々夢's cherry, and each game
/// wrote nothing into the other's fields. What that cost was not the six wasted numbers: it was that the file
/// they went into had one version between the two games, so a field 妖々夢 needed could not be added without
/// refusing every 紅魔郷 run on a disk. It was added without the version being raised, a run of 妖々夢 was
/// picked up out of a file that predated it, and the playback parted from the run with nothing in the log
/// able to say where.
/// A face the host is asked for by name, as the game itself asks for one: the family, whether it is wanted
/// bold, the height that ask carries, and how much bigger than the size it is drawn at that comes out.
///
/// What a game that ships no font of its own draws its text with — see [`Game::own_face`].
///
/// **The family alone is not the same text on the screen**, which is what the last two are for. The size orb
/// draws its own lines at is orb's, but *how a glyph of that size is made* is the game's: 妖々夢 asks GDI for
/// a height of `size * 2 - 2` — positive, so the whole cell rather than the em — and bakes into a bitmap twice
/// the size before that is drawn back down. Baked at the size instead, the same family comes out with different
/// stroke weights and edges, which a 妖々夢 session saw as text that did not look like the game's.
#[derive(Clone, Copy)]
pub struct Face {
    pub family: &'static str,
    pub bold: bool,
    /// The height to ask the host for, out of the size orb draws at — GDI's own convention, where a negative
    /// number asks for the em height and a positive one for the whole cell.
    pub height: fn(i32) -> i32,
    /// And what that height comes out at as a multiple of the size it is drawn at, which is what the drawing
    /// divides the baked mask by: 1 where a face is asked for at the size it is used at.
    pub baked_at: u32,
}

impl Face {
    /// A face asked for at the size it is drawn at, which is what a game that names no family of its own
    /// gets: the em height, and no scaling either way.
    pub const AT_ITS_OWN_SIZE: Self = Self {
        family: "",
        bold: false,
        height: |size| -size,
        baked_at: 1,
    };
}

/// A rectangle in the game's own output resolution.
#[derive(Clone, Copy)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn center_x(&self) -> f32 {
        self.left + self.width / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.top + self.height / 2.0
    }
}

pub trait Game {
    fn hooks(&self) -> Hooks;

    /// Whether a run of this game can be rewound at all, which is what pointdevice mode *is*: chapters,
    /// the snapshots they are taken as, and the retry menu a death offers.
    ///
    /// **What it decides is above the seam**, and none of it can be worked out from the methods that
    /// decline the pieces: the mode a launch starts in, whether the front end is asked which, and whether
    /// the score file is forked. Those are settled before the game has been through its own startup —
    /// the fork has to be in place before the front end's first read of the file — so there is no run,
    /// no `RunStart` and no chapter for a `None` to be asked about yet.
    ///
    /// It has to agree with them all the same, and a game that answered `true` here while declining
    /// `midstage_table`, `stage_begun` and the rest would be a launch in a mode it cannot be in: the log
    /// would say pointdevice, the front end would be asked which mode a run is, and the runs it wrote
    /// would go to orb's file having never been rewindable. Which is what 妖々夢 did before this was
    /// asked — a `pointdevice_score.dat` in a 妖々夢 install, holding runs anybody could have played.
    ///
    /// `false` for a game orb has read no run of, which keeps that game's scores in its own file and
    /// orb's question off its menus.
    fn rewinds(&self) -> bool;

    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn read_state(&self) -> State;

    /// Counts one more retry where the game's own records have somewhere to keep it, so that その他の記録 says
    /// what a mode whose runs are rewound cost.
    ///
    /// **The game's own field for it and not a new one**: its pause menu's 「最初からやり直す」 fills it in a
    /// legacy run, and in a mode where that item asks orb's question instead nothing else ever does — so what a
    /// chapter cost goes where somebody reading that screen already looks.
    ///
    /// Nothing for a game whose records have no such field, which is what the default is.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn count_a_retry(&self) {}

    /// Whether the game has been asked to give the run up and begin it again, which is what its pause menu's
    /// own 「最初からやり直す」 asks for.
    ///
    /// **A question rather than a hook**, because the ask is a field the game acts on a scene later: it is set
    /// inside the update orb's frame wraps and read outside it, so a frame is long enough to answer it in.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn asked_to_start_the_run_over(&self) -> bool {
        false
    }

    /// Whether the plate a spell card's name is shown on is being drawn, which is not the same as a card
    /// running: the card starts and the plate is put up after it, so a picture asked for as the card started
    /// came out with the boss's name and the timer in place and nothing between them.
    ///
    /// **A question of its own and not a field of [`State`]**, which every game fills: this is what a recipe
    /// waits for to photograph a card's name, so a game whose plate has not been read answers no and a recipe
    /// that asks for it says so by giving up.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn a_cards_name_is_shown(&self) -> bool {
        false
    }

    /// Whether a line of a conversation is on the screen, which is narrower than [`State::in_dialogue`]: that
    /// one is the *game's* predicate, and 妖々夢's counts the line it hands the screen over on — the one no words
    /// are drawn for. A recipe waiting for a conversation to photograph waited on that in 5面 and 6面 and got the
    /// frame the box was still being drawn on.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn a_line_of_dialogue_is_shown(&self) -> bool {
        false
    }

    /// And the run left where it is instead, the ask taken back off: what a mode whose runs are rewound has to
    /// offer in its place is where to rewind *to*, which is the retry menu.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn keep_the_run_where_it_is(&self) {}

    /// Notices what the game has counted of its own play time since the frame before, so that a span counted
    /// before anybody chose a mode can be carried onto whichever file the sitting turns out to write.
    ///
    /// **Nothing for a game whose score file does not record one**, which is what the default is: 紅魔郷's
    /// その他の記録 has no such total, so there is nothing of it to attribute.
    ///
    /// # Safety
    /// Must run on the game's main thread, once per frame.
    unsafe fn notice_the_time(&self) {}

    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn window(&self) -> Hwnd;

    /// [`Device::NULL`] until the game has finished setting Direct3D up.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn d3d_device(&self) -> Device;

    /// The music stream, if one is playing and its objects look intact.
    fn music(&self) -> Option<Music>;

    /// Identifies the track playing, changing when the game switches tracks. Used to tell a
    /// stage's own music from a boss's, which is what tells a midboss from the boss a stage
    /// ends with: a stage's data names two songs and the game plays the second for that
    /// fight alone.
    fn music_identity(&self) -> Option<u32>;

    /// Whether the song playing is the one the stage's data names for the fight it ends
    /// with, where the game says which of its two songs is on directly.
    ///
    /// `None` where it does not, and then a fight is told from a midboss by
    /// [`music_identity`](Game::music_identity) having changed since the stage began —
    /// which is what a game whose songs cannot be named by name is left with.
    ///
    /// **The two are not the same reading, and 妖々夢 is why.** A stage's first chapter is
    /// taken frames into the stage, so the track at the stage's start can be the song the
    /// stage was come into with rather than the stage's own; then every fight of that stage
    /// is a track change, and its midboss comes out as the boss the stage ends with — its
    /// chapters keeping a music they should rewind and the retry menu offering them under
    /// the wrong name. Where the game names its two songs, the answer does not depend on
    /// when a chapter was taken.
    fn the_fights_own_song_is_playing(&self) -> Option<bool> {
        None
    }

    /// The family the game draws its own text in, where it ships no font and asks the host for
    /// one by name — so that orb's own screens are in the same face as the game's text.
    ///
    /// `None` where the game ships a font instead, which orb loads: 紅魔郷 adds the `font.ttf`
    /// beside its exe. 妖々夢 ships none — the sprites in `th07.dat` are its panel's digits — and
    /// makes every string it draws out of a family it names to GDI: `CreateFontA` at 0x4322e7 with
    /// `ＭＳ ゴシック` and the weight 0x2bc, into a DIB it then uploads. The `bold` beside the name
    /// is that weight, 700 being what Windows calls bold.
    ///
    /// A family the machine does not have is a substituted face, which is why what came back is
    /// said in the log rather than assumed.
    fn own_face(&self) -> Option<Face> {
        None
    }

    /// How much panel there is above and below [`lives_row`](Game::lives_row) that the game
    /// draws nothing else in, and so which a mark of orb's own may reach into.
    ///
    /// **0.0 by default, which is a panel whose rows are as far apart as they are tall.** The
    /// mark is a brush stroke and a stroke is not the shape of a row, so it is drawn taller
    /// than one where there is room — and where there is not, a mark reaching past the row
    /// paints over a neighbour the game repaints no oftener than this row: 妖々夢 puts its
    /// bombs 16 pixels under a row 16 tall, and the stroke's own box hung 7 pixels into them,
    /// which took the tops of the bombs' stars with it.
    fn room_around_the_lives_row(&self) -> f32 {
        0.0
    }

    /// The thread that keeps the music's buffer topped up, which is left running
    /// while the game is held still: its pauses are the ones that are heard.
    fn audio_thread(&self) -> Option<u32>;

    /// Stops the music through the game's own code, so that the stream, its sound
    /// buffer and its streaming thread all go away the way the game expects.
    ///
    /// For a restore that cannot put the sound back: what the snapshot holds of it
    /// went with the track, and the stream playing now was allocated after the
    /// snapshot. Called while the game's memory is still its own, because that is
    /// the only moment freeing that stream is bookkeeping the allocator agrees with.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with nothing suspended.
    unsafe fn stop_music(&self);

    /// Whether the game still has a sound command of its own in flight, so that what
    /// a restore has left to do about the sound waits for the ask it made to have
    /// been carried out.
    ///
    /// Nothing for a game whose calls are done by the time they return; 妖々夢's are
    /// not, and one of its own runs of the sound is not enough to finish one either.
    fn a_sound_command_is_in_flight(&self) -> bool {
        false
    }

    /// Starts the stage's own track again after such a restore, and returns whether
    /// there was one to name. The restored state names a stream that no longer
    /// exists, so that has to be cleared before the game is asked for anything.
    ///
    /// **Asked for even where the chapter had nothing audible.** What a chapter had
    /// none of is a sound, not a stream: these games keep the stream a track is
    /// played through in a field of their sound player, and a take-down empties it —
    /// so a restore that asks for nothing leaves the player holding no stream at all,
    /// which is a state neither game's own code is written for. A chapter that had
    /// nothing audible has that ask taken back down instead, once the frame's own
    /// sound has carried it out.
    ///
    /// # Safety
    /// Must run on the game's main thread, after the memory has been restored.
    unsafe fn restart_stage_music(&self) -> bool;

    /// The memory holding the state of the sound system. A restore that leaves
    /// these alone leaves the music playing, instead of rewinding a stream whose
    /// sound buffer is not being rewound with it.
    fn audio_state(&self) -> Vec<Range<usize>>;

    /// Where the game keeps handles to things that are not its own memory — Direct3D's
    /// objects — which a snapshot cannot copy and a restore therefore must not rewind. Put
    /// back, a handle names something released long ago, and the next release of it is a
    /// use-after-free inside the game.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn live_handles(&self) -> Vec<Range<usize>>;

    fn play_area(&self) -> Rect;

    /// How to put the status panel's own background back over a piece of it — for the parts of a
    /// mark that fall outside a row the game repaints. `None` before the sheet is loaded, and for a
    /// game whose panel is not a tiled one.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn panel_tile(&self) -> Option<PanelTile>;

    /// Asks the game to repaint the row it shows the lives in, this frame — the background and the
    /// count both.
    ///
    /// For a mark drawn over that row, and it does two things at once. A panel that is not
    /// repainted is one where a mark blended over what the last frame left hardens into its own
    /// edges; and the count showing faintly through where the ink of the mark is dry is only true
    /// if the count is drawn again underneath it. Which is worth having: the lives are disabled,
    /// not gone, and one gained still shows.
    ///
    /// # Safety
    /// Must run on the game's main thread, before the game draws.
    unsafe fn repaint_lives_row(&self);

    /// Whether the game is going to paint that row itself this frame.
    ///
    /// Which is not the same question as whether a run is being played, and the difference is the
    /// frames a run's panel outlives the run: the game leaves the run on one frame and its panel
    /// stays on the screen until the front end has drawn over it. Every frame the game paints that
    /// row is a frame a mark has to go over, or the paint that is left standing there is the count.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn draws_lives_row(&self) -> bool;

    /// Where the mark about the run the front end is pointing at goes, in the game's own 640x480 output: the
    /// left edge, and the top of the *lower* of its two lines — the line above it carrying the mark's own title.
    ///
    /// **Its own answer per game, because which corner is free is the game's own layout.** It is drawn over the
    /// shot select while that screen is still being read, so a corner the game writes in is a corner where
    /// orb's two lines land on top of the game's. 紅魔郷 leaves the bottom-left of it empty, which is what this
    /// default is; 妖々夢 writes the difficulty's own name and its English line there, and a session of it read
    /// orb's mark over the top of them.
    /// **The last line that fits, said as one**: a line's own height off the bottom, and four pixels more for a
    /// face that measures taller than the spacing between lines. It was twenty pixels off the bottom while those
    /// lines were fifteen high, and raising that size ran the lower of the mark's two off the screen.
    fn mark_corner(&self) -> (f32, f32) {
        (
            24.0,
            crate::overlay::SCREEN_HEIGHT - crate::menu_ui::LINE_HEIGHT - 4.0,
        )
    }

    /// Where the game shows how many lives are left, in its own 640x480 output: the part of
    /// the status panel holding the count itself, which is what a mark saying the count
    /// decides nothing goes over.
    ///
    /// The count and not the label beside it. This is a rectangle orb paints over every
    /// frame, so it has to be one the game paints too: a count that changes is erased and
    /// redrawn, and the label beside it is drawn when the stage begins and never again.
    fn lives_row(&self) -> Rect;

    /// The size the game renders at, whose aspect ratio a borderless window has
    /// to preserve.
    fn content_size(&self) -> (u32, u32);

    /// False when the game is configured to take the display exclusively.
    fn windowed(&self) -> bool;

    /// Makes the game create a window instead of taking the display, so that
    /// borderless mode has a window to work with.
    ///
    /// # Safety
    /// Must run before the game creates its window.
    unsafe fn force_windowed(&self);

    /// Puts back the per-frame device setup the game itself does, for frames
    /// where its update is being held back.
    ///
    /// # Safety
    /// Must run on the game's main thread with a live device, before the game
    /// draws.
    unsafe fn set_play_viewport(&self, device: Device);

    /// The chain object the update and draw functions are called on.
    fn chain(&self) -> *mut c_void;

    /// Whether the game wipes the back buffer before drawing each frame. What decides
    /// whether anything orb draws in a corner the game does not repaint stays there.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn clears_back_buffer(&self) -> bool;

    /// Puts the game's keyboard back in a state it can be read from, returning
    /// whether it now can be. `true` when the game reads the keyboard in a way that
    /// needs nothing done to it.
    ///
    /// Needed because a game may read the keyboard through a device the system takes
    /// away while the window is behind, and getting it back is not something the
    /// game's own code can be relied on to do.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the window in front.
    unsafe fn acquire_input(&self) -> bool;

    /// Makes the game read its keyboard the way it does when DirectInput has no device to read,
    /// and says whether it had one to let go of.
    ///
    /// For a session driven by another program: a device taken `DISCL_EXCLUSIVE | DISCL_FOREGROUND`
    /// does not see keys sent with `SendInput`, and every screen orb has a question over is the
    /// game's own screen answered on the game's own keyboard. What is left is the path the game
    /// takes when it never got a device, which reads the state the system keeps — and that does see
    /// them. Nothing else about the run changes: the same code turns those keys into the same
    /// buttons.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the game past initialisation.
    unsafe fn take_sent_keys(&self) -> bool;

    /// The device setup the game does before its update: the full-output viewport,
    /// and the background clear its options may ask for.
    ///
    /// # Safety
    /// Must run on the game's main thread with a live device.
    unsafe fn prepare_frame(&self, device: Device);

    /// What the game's own frame does inside its scene *before* its draw chain, and what it does after
    /// the chain and before the scene ends.
    ///
    /// **Two seams rather than more of [`prepare_frame`](Game::prepare_frame)**, because these are a
    /// different place in the frame: that one runs before the wait and before the update, outside any
    /// scene, and what goes here is between `BeginScene` and `EndScene` with the draw chain in the
    /// middle. 妖々夢 empties the queue of quads its drawing fills here and draws it there, and a loop
    /// that put either of those in `prepare_frame` would be emptying a queue the game's own drawing had
    /// already filled or drawing one outside the scene it was queued in.
    ///
    /// Nothing for a game whose frame does nothing there, which is 紅魔郷: its `GameWindow::Render` goes
    /// `BeginScene` at 0x4207d1, `Chain::RunDrawChain` at 0x4207dc, `EndScene` at 0x4207ef with nothing
    /// between any of them. See
    /// [docs/adr/0017](../../../docs/adr/0017-the-frame-loop-has-a-seam-either-side-of-the-draw-chain.md).
    ///
    /// # Safety
    /// Must run on the game's main thread with a live device, inside the scene the game draws into, and
    /// the second of them with the first having run this frame.
    unsafe fn begin_drawing(&self, device: Device);

    /// # Safety
    /// As [`begin_drawing`](Game::begin_drawing), and after that call has run this frame.
    unsafe fn end_drawing(&self, device: Device);

    /// The two functions of the game's own that orb's frame loop calls where the game's own loop
    /// called them — see [`FrameCalls`].
    fn frame_calls(&self) -> FrameCalls;

    /// Starts the replay being watched at another of its stages, the way the game's own
    /// replay menu does: it tears this stage down, builds that one, and the replay's
    /// record of that stage puts the run's state back — the seed, the lives, the power,
    /// the score so far. Nothing of orb's is involved, which is what makes this the way
    /// to move between stages rather than restoring a snapshot of one.
    ///
    /// False where the replay has no such stage, so that nothing is asked for that
    /// would drop the game back to its menu.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn jump_to_stage(&self, stage: i32) -> bool;

    /// Puts the player where nothing can hit them for the update about to run, using
    /// whatever the game itself does for the seconds after a bomb or a respawn.
    ///
    /// For one update, because that state is the game's own and the game takes it back:
    /// a clear that reaches an ending is the only way to see one, and half an hour of
    /// playing well is not how a boundary in the ending gets looked at.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage running.
    unsafe fn make_invulnerable(&self);

    /// Whether the replay held in memory is one being played back, whose record of
    /// inputs nothing may write into.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn replaying(&self) -> bool;

    /// Asks the front end for the screen a ranking is shown on, by writing what its own item writes:
    /// the game builds it from there. orb stays out of the transition, which is what it got wrong.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the front end settled.
    unsafe fn show_ranking(&self);

    /// Whether that screen is built — the scene being it *and* the screen past its own init state.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn showing_ranking(&self) -> bool;

    /// Whether the ranking is still the scene at all, which is a coarser thing than
    /// [`showing_ranking`](Game::showing_ranking) and what says it is all the way down: the screen
    /// answers a request to leave by changing its own state first and putting the scene back a few
    /// updates later, so stopping at the state change let those updates fall into drawn frames — the
    /// ranking, seen for a second on the way out of a run.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn ranking_scene(&self) -> bool;

    /// Puts the front end's cursor back where it was before the ranking was asked for, once the front
    /// end is up again.
    ///
    /// Afterwards rather than as part of leaving the screen: asking for the ranking takes the front end
    /// down and coming back builds a new one, so a cursor written on the way out goes into the object
    /// being discarded — and the new one starts on the item the game thinks you came back from, which
    /// is the ranking.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the front end up.
    unsafe fn restore_menu_cursor(&self);

    /// Where that screen has got to, for the log: the scene, the scene wanted, the screen orb has the
    /// address of, and the state that screen is in. Numbers rather than a verdict, because a verdict
    /// is what has been wrong.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn ranking_state(&self) -> String;

    /// Tells that screen to leave, the way its own menu tells it to: the screen then puts the scene
    /// back itself, and going down is what writes the file.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with
    /// [`showing_ranking`](Game::showing_ranking) true.
    unsafe fn leave_ranking(&self);

    /// Counts one more attempt at the spell card a boss is on, and answers the count. `None` where no
    /// card is up or the record is not one the game has written.
    ///
    /// The game counts an attempt where a card *starts*, and a chapter beginning inside one never
    /// starts it: a card retried ten times is one attempt to the game. Counting it where the game
    /// cannot is the whole of it — the cleaner shape is a snapshot that predates the game's own count,
    /// which needs the moment a card starts to be a seam of its own.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn count_card_attempt(&self) -> Option<u16>;

    /// Puts the name of the spell card that is up back on the plate it is shown on, where the game
    /// baked it into a sprite when the card was declared.
    ///
    /// **A name on the screen is not memory.** The declaration hands the name to the game's own text
    /// drawing, which bakes it into a sprite — a Direct3D texture, which no snapshot holds — so a
    /// chapter put back where a *later* card has been declared since shows that later card's name over
    /// the card the run is now fighting. Which is what going back more than one chapter reaches: from a
    /// boss's spell card to the midboss's, the plate said the boss's until the next card started.
    ///
    /// Nothing where no card is up, and nothing where the game keeps no name of its own to bake.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the memory the name comes out of
    /// already put back.
    unsafe fn redraw_card_name(&self);

    /// The game's own record of which spell cards have been captured, as the bytes it keeps it in.
    ///
    /// Taken and put back rather than read field by field: orb has no use for the shape of it, only
    /// for holding it across something that would replace it. Two such things: the snapshot a chapter
    /// is rewound from, which would take back the attempt the game counted for a try that failed —
    /// and the ranking read, which fills this record out of the file and would overwrite what a
    /// session has counted since the file was last written.
    ///
    /// Empty where the game keeps no such record, which leaves both of those alone.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn captures(&self) -> Vec<u8>;

    /// Puts back what [`captures`](Game::captures) took.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `saved` a `captures` of this same build.
    unsafe fn set_captures(&self, saved: &[u8]);

    /// Puts the card records a run left back over the ones a save's own read of the file has just laid down.
    ///
    /// **Back rather than added**, and that is a reading of the game rather than a choice: 妖々夢 empties every
    /// record as it builds a run (0x42e657) and reads the file straight back into them (0x42e735), so what
    /// stands in them when that run ends is the file's own counts *with* the run's on top. Added to the file's
    /// again, a run's history would be counted twice — and it was, until a laid-out build that reads its file
    /// the way the game's does said so.
    ///
    /// For the one moment a save leaves for it — see [`Hooks::card_records_read`] — and nothing for a game with
    /// no such moment, whose captures are carried in memory instead.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `left` a [`captures`](Game::captures) of this same build.
    unsafe fn put_back_the_cards_a_run_left(&self, left: &[u8]) {
        let _ = left;
    }

    /// Counts one attempt and one capture against a card, as the game's own counting would, and answers
    /// whether it could.
    ///
    /// **For a launch that reads the score file's write back without a run being played to a spell card** —
    /// `Config::counted_a_card`, which is what asks for it. Nothing an ordinary launch reaches: what counts
    /// a card in a run is the game itself.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn count_a_card_for_a_check(&self, card: usize) -> bool {
        let _ = card;
        false
    }

    /// Whether the game writes its own score file, so that nothing above has to build a ranking screen for
    /// it.
    ///
    /// **妖々夢 does and 紅魔郷 does not**, and it is a difference in where the write is: 紅魔郷's runs as its
    /// ranking screen goes down, which is why orb builds one and takes it down; 妖々夢's screen writes at no
    /// phase it is built with, and the one write in that exe is a save the game makes for itself at every run's
    /// build and on its way out. Which is enough — every way a session's records are lost goes through one of
    /// those two — so what orb does for it is put the records back inside that save rather than ask for a save
    /// of its own. Asking is what a launch faulted on: the call runs the screen's own teardown, which frees the
    /// anm files in four of the manager's slots, and orb made it with the front end standing on them.
    fn writes_its_own_records(&self) -> bool {
        false
    }

    /// Whether the read of the score file now running is one a write follows, `screen` being the object that
    /// read is made on.
    ///
    /// **Which is what says a session's records belong back in memory at the parse inside it.** Every read of
    /// 妖々夢's file lays the file's own card records over the ones in memory, and only some of those reads are
    /// part of a save: the phase the object carries is what tells them apart, and the same field orb reads as
    /// [`Game::ranking_state`] carries it.
    ///
    /// `false` for a game whose reads are none of them a save's, which leaves the records in memory alone.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `screen` the object the game's own read was entered with.
    unsafe fn its_records_are_about_to_be_written(&self, screen: *mut c_void) -> bool {
        let _ = screen;
        false
    }

    /// And puts it back the way a run played into place needs it: the counts as they were before the
    /// buttons went in, and whatever the playback named left as it named it.
    ///
    /// **A name is not a count.** Playing a stage in again starts every card the run had passed, so the
    /// playback both counts attempts nobody made this session — which is what has to go back — and writes
    /// each of those cards' *names* into the record, which is the one thing about them the game itself
    /// knows and the session needs. Put back along with the counts, the name goes back to whatever the
    /// record held before the card was ever started, and a landing is inside the card: nothing starts it
    /// again, so nothing names it again for the rest of the session.
    ///
    /// The same as [`set_captures`](Game::set_captures) where the game keeps no name of its own.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `saved` a `captures` of this same build.
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]);

    /// Empties the game's own record of which spell cards have been captured, where a ranking is
    /// about to be read into it.
    ///
    /// The game's parse of that part of the file copies the records the file holds and leaves the
    /// rest as they were — which is how a run's captures survive the reload the game does at every
    /// stage, and with two score files is also how one file's captures reach the other: the record
    /// is written back out of memory, so looking at one ranking and then leaving the other writes
    /// the first one's captures into the second one's file. Cleared here so that a ranking read
    /// defines the history instead of adding to what was already in memory.
    ///
    /// `screen` is what the callback was handed, because whether the game is going to do that parse
    /// at all is a state of that screen: the states it is in on the way out of a run are the ones
    /// where the record in memory is that run's own, and they keep it.
    ///
    /// # Safety
    /// Must run on the game's main thread, before the game's own read of the ranking.
    unsafe fn forget_captures(&self, screen: *mut c_void);

    /// Fills whatever the game keeps of a *record of runs* out of the file the mode now points at, which
    /// is a read of that file the game itself does not make.
    ///
    /// **What a stage select offers is not a record, and the score beside it is.** The game takes both out
    /// of one read — the front end's own, which is pointed at the game's own file so that a mode whose file
    /// is new does not lock what an installation has earned — and the record in that file is the other
    /// mode's. Which left 完全無欠モード showing legacy practice scores on a screen whose runs go in orb's
    /// file. One read cannot land in two files, so the record is read again: the front end's read stands as
    /// it is, and this puts the mode's own numbers over the half of it that is a record rather than an
    /// unlock.
    ///
    /// **Every such record, which for 妖々夢 is two**: the practice scores, and the card records its
    /// ランキング shows and a run counts into. A session that never answered the mode question again showed
    /// the other mode's of both — a run given up back to the practice stage select carried a legacy score
    /// for the stage it had just been played in, and the ランキング counted nothing of the session.
    ///
    /// Called after the front end's own read, and again wherever the mode is settled: the first is every
    /// path that brings that screen up, including a run coming back to it, and the second is the last thing
    /// before a practice stage is chosen.
    ///
    /// Nothing where the game keeps no such record, which leaves whatever its own read found standing.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames: it reads a file through the game's own code
    /// and writes what the game's own front end draws from.
    unsafe fn read_what_the_modes_file_records(&self);

    /// What the game's own title menu is *pointing* at, where that is something orb has a question
    /// about, so the question can go on the press rather than after it.
    ///
    /// `None` on any other screen and on an item orb has nothing to ask about. Whether a press would be
    /// acted on there is [`Game::menu_takes_a_press`] and not part of this: what the cursor is on is
    /// also what the mark on the shot type select is drawn from, and that has nothing to wait for.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn menu_pointed_at(&self) -> Option<Menu>;

    /// Whether the screen the front end is on would act on a decide, on the read after this one.
    ///
    /// Which is what says a press held back there is a press the game would have honoured: these
    /// screens ignore their own decide for their first frames, and one held back over those frames and
    /// handed over afterwards is a keypress the game had thrown away, acted on late.
    ///
    /// The read after rather than this one, because what this decides is the *next* read's holding.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn menu_takes_a_press(&self) -> bool;

    /// What the pad is doing, in the terms a menu of orb's needs: which way it is being pushed,
    /// and whether it is deciding or cancelling.
    ///
    /// Asked of the game rather than worked out from a reading, because *which button is which* is
    /// the game's mapping and *where* a pad is read is the game's business too: the device its own
    /// enumeration found is one orb's sampling never sees — it is taken `DISCL_EXCLUSIVE` — and a menu
    /// driven from the samples alone would ignore the pad the game is being played with. Which is what
    /// it did.
    ///
    /// `pad` is the pad orb's own sampling last saw pushed, and `None` while none is answering. Handed
    /// in rather than fetched, because taking the samples is orb's business and not the game's: a
    /// `Game` that went looking for one would be reaching past the seam for a thread of orb's. Merged
    /// with the game's own device rather than standing behind it — every pad on the machine is the one
    /// player's — and it may be that same device again, which merges to the same answer.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn pad(&self, pad: Option<Reading>) -> Pad;

    /// **The pad half of the game's own input read**, which orb does instead of the game: every pad, as
    /// the bits of the word the game acts on.
    ///
    /// The game's own function that did this is hooked and not called through, so what this answers is
    /// the whole of what a pad does to a frame — the buttons its mapping names, the directions its own
    /// arithmetic makes of an axis, and the hat it reads on no device at all. Which is why it is one
    /// answer and not several: the device the game holds is read once here, where a second question
    /// about the same device would be a second `Poll` and `GetDeviceState` in the frame.
    ///
    /// `pad` is the pad orb's own sampling last saw pushed, and `None` while none is answering — handed
    /// in rather than fetched, because taking the samples is orb's business and not the game's: a `Game`
    /// that went looking for one would be reaching past the seam for a thread of orb's. The device the
    /// game's own enumeration found is not handed in, orb's sampling never seeing it: it is taken
    /// `DISCL_EXCLUSIVE`, so reading it is something only a `Game` can do.
    ///
    /// `hats` is `dpad_moves` out of `orb.yaml`, and it gates the one thing here the game would not have
    /// produced from the same pads: a d-pad. Everything else is what its own read would have made of
    /// them, which is why nothing else is behind a setting.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn pad_word(&self, pad: Option<Reading>, hats: bool) -> u16;

    /// And the same pads written into the array the game's own [`Hooks::pad_buttons`] read just filled, so
    /// that a key config screen sees a button orb read for itself as pressed.
    ///
    /// Added to what the game found rather than written over it: a pad the game *can* see is one that
    /// screen already answered to, and this is for the ones it cannot. Nothing for a game with no such
    /// screen, which is where the hook is `None` and this is never reached.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `into` the array that read answered — the game's own, of
    /// the length its own screen walks.
    unsafe fn add_pad_buttons(&self, into: *mut c_void, pad: Option<Reading>);

    /// Gives the run up, and says whether there was one to give up.
    ///
    /// For the retry menu's third choice. Asked of the game rather than done by hand, because
    /// every game has its own way out of a run and each of them already works: what a pause
    /// menu's quit does is what orb should do, so that whatever a run has to be taken down is
    /// taken down by the code that knows about it.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with a run in progress.
    unsafe fn leave_run(&self) -> bool;

    /// Leaves the game's own idea of the buttons so that the frame it carries on into has no new
    /// press on it, whatever is being held.
    ///
    /// For the frames orb froze the game for: the key that answered orb's question is not a key
    /// the game's own menu should act on, and the keyboard is read globally rather than per
    /// window, so there is nothing else to tell one from the other. Without it, whether the
    /// answer also reaches the game depends on whether the screen behind the question happens to
    /// read input on the frame it resumes on, which is a property of that screen and not
    /// something orb decided.
    ///
    /// Not needed after a retry, where the snapshot restore puts the same state back with
    /// everything else in `.data`.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn swallow_input(&self);

    /// Which buttons the game's own front end reads as *decide*, in the word its input read hands
    /// back.
    ///
    /// For the question orb asks over the screen a run is started from: a press taken out of that
    /// word is a press no screen in the frame ever saw, so the screen is still standing when the
    /// question comes down and answering "neither" is nothing at all rather than a scene put back by
    /// hand. Handed over afterwards the same way, by putting the bits back in for one read, which
    /// leaves starting the run to the screen whose business it is.
    ///
    /// Which bits those are is the game's own: 紅魔郷's shot type select tests
    /// `g_CurFrameInput & 0x1001` against the frame before at 0x436d79.
    fn menu_decide(&self) -> u16;

    /// And which it reads as *back*, for the frames after one of orb's questions was cancelled with
    /// that key: the key is still down, and what the screen underneath would do with it is go back —
    /// which is not what somebody who answered "neither" about *this* screen asked for. So it is kept
    /// from the game until it is let go.
    ///
    /// Held back rather than swallowed. [`Game::swallow_input`] leaves the game's idea of the frame
    /// before as all-ones, and a screen that tests `(cur & mask) != (last & mask)` — which is how these
    /// menus read both of their keys — then takes anything short of the whole mask being down for a
    /// press. One of the two buttons is the ordinary case, so that made the press arrive rather than
    /// stopping it.
    fn menu_cancel(&self) -> u16;

    /// Takes the game off the screen that offers to save a replay of the run just finished, if
    /// that is where it is, and says whether it did.
    ///
    /// For a pointdevice run, where the offer is one nothing could accept: a replay is the
    /// inputs and nothing about the rewinds between them, so one recorded here plays back as a
    /// run that dies where this one carried on. Refusing the write instead would leave somebody
    /// naming a file that never appears.
    ///
    /// Asked every frame, so it must cost nothing on the frames where there is no such screen.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn skip_replay_prompt(&self) -> bool;

    /// The frame the stage now running is on, or `None` where no stage is running.
    ///
    /// Its own accessor rather than a whole [`State`] because the input read asks for it on every
    /// frame there is, including the frames of a menu: what a frame's buttons are written down
    /// against is this number, and reading the rest of the state to get it would put a job-chain
    /// walk inside the game's keyboard read.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn stage_frame(&self) -> Option<u32>;

    /// The generator's own state, which is the whole of what says a stage is playing out the same way twice:
    /// the same seed and the same buttons make the same stage, and one number out of step makes another one.
    ///
    /// **Its own accessor beside [`stage_frame`](Game::stage_frame) and for the same reason**: the input read
    /// asks for it on every frame, one to write down beside that frame's buttons and one to hold against what
    /// was written down — see `resume::fed`. A whole [`State`] to get it would put a job-chain walk inside the
    /// game's keyboard read, and [`Reproduction`] would put two pointer chases there.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn generator(&self) -> u16;

    /// Hands the word a frame's read produced to the run, where the game itself would hand it over too late
    /// for that frame's own update to act on.
    ///
    /// **妖々夢's frame of input lag is this and nothing else.** Its jobs run in the order their chain
    /// priorities give: the read at 0, the player at 8 — which reads the word *the run acts on* and moves on
    /// it — and the copy from the one word into the other at 0x10, inside the recorder. So the player moves on
    /// what the copy left a frame ago. Called from the input hook, which is the read at priority 0, this makes
    /// that copy early and the frame's own keys are what its update acts on.
    ///
    /// Nothing where a game hands the read to its own update in time, which is 紅魔郷: there is one word and
    /// the player reads it. And nothing while a replay is being watched, where what the run acts on is the
    /// recording rather than the keyboard.
    ///
    /// **`moved_on` is whether this read is the first of a frame of the run**, and what it decides is the
    /// *pair*: a run reads a bomb as an edge, so the word it last acted on has to be the frame before's rather
    /// than the read before's. 妖々夢 reads the keyboard again for a frame it did not advance the run on — its
    /// own slowdown answers the manager's 3, which stops the chain walk before both the player's job and the
    /// copy this stands in for — so on such a read the game moves neither word, and a hand-over that moved
    /// them made the pair hold one word twice. The press on that frame is then no edge, and which frames it
    /// happens on is how many updates the game spent on each: a run's own buttons played back in a burst of
    /// them skip elsewhere than the run did.
    ///
    /// # Safety
    /// Must run on the game's main thread, from inside that read.
    unsafe fn hand_the_read_to_the_run(&self, word: u16, moved_on: bool) {
        let _ = (word, moved_on);
    }

    /// Which bits of a frame's input decide how a run goes, and so are the ones worth writing
    /// down and handing back.
    ///
    /// 紅魔郷's own replay records exactly these, which is what says the rest do not change a run.
    /// The pause button is outside them, which is the one that matters: a resume that fed it back
    /// would open the menu instead of playing.
    fn run_input(&self) -> u16;

    /// Which run this is, as the game's own front end left it.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn run_start(&self) -> RunStart;

    /// Which run the front end is *pointing* at, where it is on a screen that knows: the difficulty
    /// and the character already chosen, and the last of the three under the cursor.
    ///
    /// For the mark that says a run of it was left unfinished, which is worth putting where the run is
    /// being chosen rather than only offering it a screen later — nothing on the game's own screens
    /// remembers which run somebody was in the middle of.
    ///
    /// `None` anywhere the answer would be a guess: any other screen, and a practice run, whose stage
    /// is part of which run it is and is not chosen until afterwards.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_pointed_at(&self) -> Option<RunStart>;

    /// Which run this is one of, as a name: what two runs have in common when a chapter of one can
    /// be picked up in the other.
    ///
    /// Asked of the game because the characters are its own. It is also the name of the file that
    /// chapter is kept in, so it has to be one a directory listing can be read — and two runs are
    /// the same run exactly when this is the same, which is what keeps the answer in one place.
    ///
    /// `None` for a run there is no slot for, which is what stops one being kept at all: nothing to
    /// name is nothing to write, nothing to offer and nothing to mark, in one place rather than as a
    /// check three callers have to remember.
    fn run_slot(&self, run: &RunStart) -> Option<String>;

    /// **The file a run left unfinished is written in, which is this game's own.**
    ///
    /// What [`crate::resume`] does with a kept run — when to write it, where the file goes, playing the
    /// buttons back into a stage, holding the landing against what was written down — is the same work for
    /// either game. What a *file* holds is not: 紅魔郷 keeps a rank, a count of power items and a count of the
    /// extra lives its score has paid for, where 妖々夢 keeps a cherry it pays its own with, and each game
    /// wrote nothing into the other's fields for as long as the two shared one shape.
    ///
    /// **And one version number stood in front of both**, which is what settled it: the generator of every
    /// frame — the one thing that can say where a playback stopped reproducing a run — went into the file
    /// without that number being raised, because raising it would have refused every 紅魔郷 run already on a
    /// disk. A 妖々夢 run picked up out of a file from before it parted from itself inside its first thousand
    /// frames and nothing in the log could say where. So the file is the game's, and each game's version is
    /// its own: `th06::resume` is frozen at what shipped, `th07::resume` starts again at 1.
    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8>;

    /// And back, or `None` for bytes that are not a file this game wrote.
    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept>;

    /// **The run's numbers as this stage began, in the game's own bytes.**
    ///
    /// Opaque here, and the whole point of that: what a run *is* differs between the games and the
    /// machinery does not need to know. 紅魔郷 pays for extra lives out of the score and keeps a rank its
    /// enemies read; 妖々夢 pays for them out of a cherry it draws from its own generator as the run is made,
    /// and keeps a baseline every reader of that cherry measures from. They were one struct with both games'
    /// fields in it, each writing nothing into the other's — and with the shape shared, a number 妖々夢 needed
    /// could not be added without touching the game people already have.
    ///
    /// These bytes go into the game's own file and come back out of it — see [`Game::encode_resume`] — so
    /// nothing but the game that wrote them ever looks inside.
    ///
    /// **What goes in is what the game's own replay restore puts back**, field for field, and that is a
    /// reading of the game rather than a judgement: a replay watched from a stage past the first is the game
    /// enumerating what a stage of a run in progress has to be restarted from. A resume builds the stage and
    /// plays the buttons in, so the world is reproducible exactly as far as it is a function of the seed, the
    /// buttons and these bytes — and a field left out is one the replay gets a *build's* value for where the
    /// run had its own. See
    /// [docs/adr/0022](../../../docs/adr/0022-what-a-resume-puts-back-is-what-the-games-own-replay-restore-puts-back.md),
    /// which is also where the alternative that was built and taken back is written down.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn run_state(&self) -> Vec<u8>;

    /// Puts them back, the way the game's own replay puts a stage's numbers back — all but the seed, which
    /// is [`Game::seed_the_stage`]'s and has to be earlier.
    ///
    /// # Safety
    /// Must run on the game's main thread, where the game has a stage built and has not yet
    /// updated it — see [`Hooks::stage_begun`]. `state` must be bytes this game's own
    /// [`Game::run_state`] produced.
    unsafe fn set_run_state(&self, state: &[u8]);

    /// Seeds the generator for the stage the game is about to build, out of those same bytes.
    ///
    /// Apart from the rest of a stage's numbers because it goes in at another moment: building a
    /// stage draws from the generator, so the seed has to be in before that, where the numbers cannot
    /// go in until the stage exists. Both moments were measured wrong before they were measured
    /// right — see the game's own account of the seam, `th06`'s `STAGE_REGISTER_CHAIN`.
    ///
    /// # Safety
    /// Must run on the game's main thread, from the hook over [`Hooks::stage_building`], before the
    /// original.
    unsafe fn seed_the_stage(&self, state: &[u8]);

    /// And what those bytes say, for the log: the game's own numbers in the game's own words, since a line
    /// about a run of 紅魔郷 has a rank in it and one about a run of 妖々夢 has a cherry.
    fn say_run_state(&self, state: &[u8]) -> String;

    /// Whether the game's front end has asked for a run and nothing of that run has been built yet.
    ///
    /// The one frame on which what the run is — its difficulty, its character, its shot — is settled
    /// and none of it has been acted on, which is where the question of picking one up belongs: the
    /// answer costs nothing either way there, and the front end has already taken itself down. Never
    /// true of the attract demo or a replay, which ask for a run the same way.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_chosen(&self) -> bool;

    /// Points the run being built at another of its stages, and says whether the game has one.
    ///
    /// The whole of what picking a run up asks of the front end: the difficulty, character and shot
    /// are the ones just chosen, and [`Hooks::stage_begun`] is where the run's own numbers go in.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, on a frame [`Game::run_chosen`] is true
    /// of.
    unsafe fn start_stage(&self, stage: i32) -> bool;

    /// Whether the run has finished into the game's own result screen.
    ///
    /// What says a run is over rather than left: a run given up does not go through one — see
    /// [`Game::leave_run`] — so this is what tells the chapter worth coming back to from the run
    /// that has none.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_finished(&self) -> bool;

    /// **The line one pass of a stage is held against another's, in the game's own words.**
    ///
    /// Two passes over one stage have to agree on all of it, frame for frame. That is the premise
    /// chapters rest on — a boundary is a frame number, and a step back replays the stage to reach it —
    /// so when they disagree, the first frame they disagree on and which of its numbers it was are the
    /// whole of the diagnosis.
    ///
    /// **The numbers in it are the game's own and so is the type they are held in** —
    /// `th06::Reproduction` and `th07::Reproduction`, the way each game's resume file has its own
    /// `Numbers`. What stood here before was one struct carrying the union of both: the box the player is
    /// held inside, the extra lives the score has paid for, the rank and the subrank are 紅魔郷's, and
    /// 妖々夢 answered zero for all four and printed them on every line. The union of every 東方 game is
    /// not one kind of game to take a union of — a photo game and a versus game have none of the four
    /// either, and no lives to pay for at all.
    ///
    /// A line rather than either type, because the two things done with one are printing it and holding it
    /// against a written one — and the written one is text, a file's own field being the line as the log
    /// prints it so that somebody can hold the two together by eye. [`crate::resume::differs`] is what
    /// holds them, by splitting `key=value` and comparing by name, and it knows no game's names.
    ///
    /// Always a line, `replay_frame=-1` and zeroes included: the frame before a stage exists is a frame two
    /// passes agree about as much as any other. What skips such a frame is [`Game::mid_build`], and only
    /// the one instrument that has nothing to say about it.
    ///
    /// **What a line says cannot be changed without the files that hold one becoming unreadable in the
    /// only way [`crate::resume`] cannot catch**: a chapter is written down with the line of its own
    /// frame, and a file written before the change disagrees at the first field of it while the version
    /// says nothing is wrong. Which is per game, this being per game.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage running.
    unsafe fn reproduction(&self) -> String;

    /// **Where the things a run's world holds are**, folded to one number, and `None` from a game nobody has
    /// read them off.
    ///
    /// **Beside [`Game::reproduction`] and never in it**, which is what the paragraph above forbids: this is a
    /// thing to add to and to change as the next parting needs, and a line the files hold is a line that cannot
    /// be. So it goes in the log next to that one and nowhere else.
    ///
    /// **Which a parting asked for.** A pair of launches of 妖々夢 agreed on every field of a reproduction line
    /// for 992 frames of 2面 and parted at 993 on a point item: the run collected one where the playback passed
    /// through it, with the same word, the same player, the same count of items in the air and the same cherry.
    /// Two worlds that differ in nothing a line carries and differ in what they collect differ in where those
    /// things *are*, and that is the one thing no field of it looks at.
    ///
    /// A fold rather than the places: what a comparison wants is the frame two passes stop agreeing on, and a
    /// game's whole item array printed per frame is a log nobody can read.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage running.
    unsafe fn the_worlds_fingerprint(&self) -> Option<String> {
        None
    }

    /// **The bytes a run's own numbers are held in, said as themselves**, and `None` from a game nobody has read
    /// them off. Said once at a stage's start, where the numbers have just been put in place.
    ///
    /// **Which is what tells a missing field from a wrong one.** A chapter recorded in a stage the run walked
    /// into does not replay: two passes agree on every field a reproduction line carries for hundreds of frames
    /// and then collect different items. The fields a line carries are the ones somebody thought to name, so a
    /// difference in a field nobody named reads as no difference at all — and the game's own per-stage replay
    /// record turned out to name exactly the fields [`Game::run_state`] already puts back, which says the
    /// missing one is not in that record either.
    ///
    /// So the bytes rather than a fold or a list: held against the same stage entered the other way, a dump
    /// names the offset, and an offset is a thing the disassembly can be asked about.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage's numbers in place.
    unsafe fn the_runs_own_bytes(&self) -> Option<String> {
        None
    }

    /// **The bit a recipe's button name is in this game's own word**, or `None` for a name it has nothing for.
    ///
    /// What a bot presses is a name in a file — `decide`, `focus`, `left` — and what one *is* differs from game
    /// to game: `decide` is a shot and a return key in 妖々夢 and something else in the next game. So the names
    /// are resolved here and the recipe that holds them is `orb-bot`'s.
    ///
    /// `None` for everything, from a game nobody has written a bot's vocabulary for.
    fn button_bit(&self, _name: &str) -> Option<u16> {
        None
    }

    /// **Everything of one frame of a run that a bot decides from**, written into a buffer the caller keeps, and
    /// `false` where there is no stage to read one off.
    ///
    /// **What is here is what the game holds, and none of what to do about it.** Which of these things is worth
    /// keeping away from, how far, and where to be instead are the bot's — so a bullet still coming out and a
    /// body with nothing solid on it are both handed over with what the game says about them rather than left
    /// out, a beam is handed over with the frame it starts hitting on rather than judged, and an item is handed
    /// over as which item it is. Reading them is what needs this game's addresses; deciding is what does not,
    /// and `orb-bot` is where it is decided.
    ///
    /// The buffer is the caller's so that a frame of a run costs no allocation once it has run once: the three
    /// lists are cleared and filled, and a stage of 妖々夢 fills the first of them six hundred times.
    ///
    /// Nothing filled and `false` from a game nobody has read a frame of.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn what_a_bot_sees(&self, _into: &mut Frame) -> bool {
        false
    }

    /// **Says what the game's own arrays hold within reach of the player, past every rule that leaves one out.**
    ///
    /// Which is the question [`Game::what_a_bot_sees`] cannot answer for a bot that was hit: what it was given is
    /// what it decided from, so a hazard missing from that is missing from any reading of it. This is the arrays
    /// themselves, in the words this game's own fields are kept in, and the one thing that tells a model whose
    /// boxes are too small from a hazard that is in no array at all.
    ///
    /// Nothing said, from a game nobody has read a frame of.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn say_what_was_near(&self) {}

    /// **Which item the front end's own cursor is on**, counted the way the game counts it, and `None` off a
    /// screen that keeps one — or from a game whose front end nobody has read.
    ///
    /// What a recipe needs it for is naming an item of a list it cannot see the items of: which screen is up
    /// and where that number lives are this game's, so the answer is here.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn front_end_cursor(&self) -> Option<i32> {
        None
    }

    /// **Which of the front end's screens is up**, by whatever number the game keeps for it, and `None` off the
    /// front end — or from a game whose screens nobody has read.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn front_end_screen(&self) -> Option<i32> {
        None
    }

    /// **Whether the game is on the update that runs while a stage is still being built**, where the
    /// replay's own clock does not exist yet and the player and the play field read as zeroes.
    ///
    /// Asked by the one instrument with nothing to say about such a frame — the `sync:` line a `--judge`
    /// pass writes — and by nothing else: a resume writes a line for every frame it plays, that frame
    /// included, because two passes agree about it as much as about any other.
    ///
    /// Which field says so is the game's, this being per game like [`Game::reproduction`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn mid_build(&self) -> bool;

    /// Midstage chapter boundaries per stage.
    fn midstage_table(&self) -> &'static [&'static [Boundary]];

    /// **What the rows of that table are called where `stage N` is not what they are**, in the order the table
    /// holds them: the last rows, and as many of them as the game has.
    ///
    /// The game's because the games differ — 紅魔郷's table ends at its Extra and 妖々夢's carries the Phantasm
    /// after it — and a tuning pass that read the shape off the table instead wrote 妖々夢's Extra down as
    /// `stage 7` and its Phantasm as `extra`, which is the row above the one a reader would paste an Extra
    /// boundary into.
    fn stages_past_the_numbered(&self) -> &'static [&'static str];

    fn midstage(&self, stage: i32) -> &'static [Boundary] {
        usize::try_from(stage)
            .ok()
            .and_then(|stage| self.midstage_table().get(stage).copied())
            .unwrap_or(&[])
    }
}

/// One entry of the midstage table: the frame, and whether somebody put it there.
///
/// Whose hand it was is data and not a comment on the number, because the shortest a chapter may
/// be exempts a boundary somebody placed — see `Chapters::due`. Left as a comment it was an
/// exemption only a tuning pass could reach, so a baked table divided a stage one way in the
/// `--judge` pass that chose its numbers and another way in the run that used them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Boundary {
    /// The enemy-timeline frame the chapter begins on. That clock advances the same way however
    /// the player is doing and on every difficulty, which is what lets one number stand for a
    /// place in the wave pattern.
    pub frame: i32,
    pub by_hand: bool,
}

/// A boundary the detector proposed and a pass kept.
///
/// **`const fn` because the baked table calls it in a const initialiser**, which is also why no test can
/// enter this or [`hand`]: what happens to them is const evaluation and not execution, so a coverage run
/// reports both as never run however many stages are played. Nothing to fix — a run that reported them
/// covered would be reporting something else.
pub const fn proposed(frame: i32) -> Boundary {
    Boundary {
        frame,
        by_hand: false,
    }
}

/// And one somebody put there, which is a number nothing would propose again if it were lost.
pub const fn hand(frame: i32) -> Boundary {
    Boundary {
        frame,
        by_hand: true,
    }
}

/// **One frame of a run as the game holds it**, which is what a bot is given and the whole of what it is given.
///
/// Filled by [`Game::what_a_bot_sees`] and kept by whoever asks, so that asking again costs no allocation. The
/// three lists are what the player is tested against and what there is to pick up; the rest is the field it all
/// happens in.
#[derive(Default)]
pub struct Frame {
    /// The play field: left, top, width, height, in the game's own units.
    pub field: (f32, f32, f32, f32),
    /// Where the player is, and the half of the box the game's own hit test gives it — the two halves being
    /// separate because nothing says a hitbox is square.
    pub at: (f32, f32),
    pub half: (f32, f32),
    /// What one frame of holding a direction moves the player by, across and down, straight and then diagonally.
    /// Both pairs, because nothing says the two axes are scaled alike.
    pub straight: (f32, f32),
    pub diagonal: (f32, f32),
    /// Whether the game says the player has just been hit and is inside whatever window it gives to take that
    /// back — 妖々夢's 決死結界 and whatever the next game calls its own.
    pub hit: bool,
    /// Whether the player is in its ordinary state: neither hit nor in the middle of dying or coming back.
    ///
    /// Which is not `!hit`, there being more than two states — and the difference is what tells a border
    /// deflected from a border tidied away by a respawn.
    pub normal: bool,
    /// Whether the player is carrying something that takes a hit for it, for a game that has such a thing: its
    /// going away while the player is otherwise well is a hit that the count of deaths never sees.
    pub border: bool,
    /// Whether a line of dialogue is up, which is a thing to press past.
    pub dialogue: bool,
    /// Where the boss is, for a fight that has one up.
    pub boss: Option<(f32, f32)>,
    pub hazards: Vec<Hazard>,
    pub beams: Vec<Beam>,
    pub items: Vec<Pickup>,
}

/// **One thing the player's own hit test is run against**, and what the game says about it.
pub struct Hazard {
    pub kind: Kind,
    /// Which slot of its own array it is in, which is what makes it the same thing as last frame's: a hazard
    /// with no step of its own can only be read by the difference from where it was, and that needs an identity.
    pub slot: usize,
    pub at: (f32, f32),
    /// Its step, where the game keeps one on it, and `None` where nothing on it is a step — a body a script
    /// moves, or a place something has already been. The difference between the two is not a judgement: it is
    /// whether the field exists.
    pub step: Option<(f32, f32)>,
    /// Half the box the player is tested against, on each axis, as the game's own test is given it.
    pub half: (f32, f32),
}

/// **Which of the game's arrays a [`Hazard`] came out of**, with what the game says of it there.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A bullet, and whether it is in the state its collision is found in: the others are a bullet still coming
    /// out, or one already spent.
    Bullet { hitting: bool },
    /// A body, and whether the game will run its collision at all — an enemy with nothing solid on it cannot
    /// touch the player, however wide it is.
    Body { solid: bool },
    /// A place a body has already been, which is what a beam that follows one is made of. It does not move and
    /// nothing will move it: only its ends come and go.
    Trail,
}

/// **A beam**, which is a line rather than a box and so is not a [`Hazard`] at all.
pub struct Beam {
    pub slot: usize,
    /// Where it is fired from, and which way it points now. What it will be pointing at in a frame's time is a
    /// thing to work out from how it has been turning, there being no rate on it to read.
    pub origin: (f32, f32),
    pub angle: f32,
    /// How far along it the beam runs, from the near end to the far one.
    pub near: f32,
    pub far: f32,
    /// Half of how wide it is now.
    pub half_across: f32,
    /// How many frames until the game starts testing the player against it, which is nought or less for one
    /// already firing. Before that it is drawn and hits nothing.
    pub fires_in: i32,
    /// Whether it is on its way out. It goes on hitting while it fades, narrowing as it does — so this is worth
    /// having only to say that the width read for it is the width it had.
    pub fading: bool,
}

/// **Something to pick up**, and which of the things a game drops it is.
pub struct Pickup {
    pub worth: Worth,
    pub at: (f32, f32),
}

/// **The kinds of thing 東方 drops**, which is a vocabulary the games share where their own numbering for it is
/// not: what one is called here is what every game of them has one of, and how much any of them is worth going
/// for is the bot's to decide.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Worth {
    Life,
    Bomb,
    BigPower,
    Power,
    Point,
    Cherry,
    /// A kind this vocabulary has no name for, from a game that drops something of its own.
    Other,
}

/// **What orb knows about where a run has got to and a game does not**, for the recipe a launch playing
/// itself is walking — see [`Game::what_a_bot_sees`].
///
/// Three numbers rather than a game's own vocabulary: a stage, a chapter of it, and which of orb's own
/// questions is up are orb's own — `State` and `chapter::Chapters` are where the first two come from, and the
/// third is a menu orb draws itself. Nothing of any game is in it, which is what makes it shared.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reached {
    /// Counted from one, the way the log counts it.
    pub stage: i32,
    pub chapter: u32,
    pub question: Option<Question>,
    /// **Which item the game's own cursor is on**, counted from zero the way the game counts it, and nothing
    /// where the screen up keeps no cursor a recipe could read.
    ///
    /// Which is what makes the front end's screens walkable at all: a recipe cannot see what the items of a
    /// list *are*, so it names the one it wants by where it sits and taps until the cursor is there.
    pub cursor: Option<i32>,
    /// **Which screen of the front end that cursor is a list of**, and nothing off one. The cursor is one field
    /// whatever screen is up, so an item named without this is an item of whichever list reaches the number
    /// first — which put a run on Lunatic once.
    pub screen: Option<i32>,
    /// **Whether the run is over**, which is what makes a `play` step end: there is nothing left to play, the
    /// game is on its own screen for it, and the rules press nothing there — so a step that waited for a stage
    /// would wait out its whole bound saying `still waiting` while nothing happened.
    pub over: bool,
    /// **Whether a boss is up**, which is what the part of a stage before one ends at: a recipe practising that
    /// part had no condition for it and was bounded by a count of frames instead, which is a guess about how
    /// long a stage's midsection runs.
    pub boss: bool,
    /// **Whether a line of dialogue is up**, which is what the fight a stage ends with begins after — and the
    /// one thing in a stage no count of frames reaches, the rules pressing past a line as fast as the game
    /// takes presses.
    pub dialogue: bool,
    /// **Whether a spell card is running**, which is the frames the plate its name is on is drawn over. Which
    /// chapter of a stage that is differs stage by stage, so a step naming one is written against a reading of
    /// one afternoon's logs.
    pub card: bool,
    /// **Whether the plate a card's name is shown on is being drawn**, which is the frame a picture of that name
    /// has to be asked on — and a frame or more after the card itself started. See
    /// [`Game::a_cards_name_is_shown`].
    pub card_name: bool,
}

/// The questions of orb's own a recipe can wait for, by the names it writes them as.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Question {
    /// `mode` — which of the two things a run is.
    Mode,
    /// `resume` — where to start, for a run left unfinished.
    Resume,
    /// `retry` — where to start again, for a run that died. Which is the whole of the mode being played, so a
    /// bot takes it wherever it comes up rather than waiting for a step to say so: a death lands in the middle
    /// of one.
    Retry,
}

/// The game's state for one frame, read all at once so that everything in it
/// describes the same instant.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct State {
    /// The game's own scene identifier. Only for the log and for noticing that
    /// the scene changed.
    pub scene: i32,
    /// The scene the game is on its way to, which is the same number as [`scene`](State::scene)
    /// wherever it has arrived. Read because asking the game for one of its own screens has to
    /// start from a game that has arrived somewhere: asking while a change is still in
    /// flight left the front end built twice, moving its cursor two items to a press.
    pub wanted: i32,
    /// The gameplay scene, whoever is driving it.
    pub playing: bool,
    /// That scene, or the step between two of its stages. Everything a run's
    /// snapshots reach through is still alive here, which is what says whether they
    /// are worth keeping: a stage transition leaves the gameplay scene for a moment
    /// and is not the run ending.
    pub in_run: bool,
    /// A run orb should act on: not a menu, and not the attract demo or a replay,
    /// which look like play but have no player to offer a retry to.
    pub in_game: bool,
    pub in_ending: bool,
    /// Which script the ending is running, as the address of the file it was read from.
    /// `None` when no ending is running.
    ///
    /// An ending is one script per part of itself and moves on by reading the next file over
    /// the one it is running, so this changing is the one thing that marks a part of an ending
    /// ending: the scene does not change with it and no flag says which part is running. What
    /// the ending skip stops on, so that the part it hands over to — the staff roll — is
    /// played instead of run out with the rest.
    pub ending_script: Option<usize>,
    pub demo: bool,
    pub replay: bool,
    pub practice: bool,
    /// A menu is open and the game has stopped updating itself.
    pub paused: bool,
    /// Dying or respawning: no place to start a chapter from, and no place to
    /// return to either.
    pub unsettled: bool,
    /// A bomb is going off, which clears the screen and makes the player
    /// invulnerable for a couple of seconds. Also no place to start a chapter: the
    /// danger has been taken away and comes straight back.
    pub bombing: bool,
    pub in_dialogue: bool,
    pub stage: i32,
    pub difficulty: i32,
    /// Frames since this stage began.
    pub stage_frames: u32,
    /// The clock the stage's enemy script runs on, which is what midstage
    /// boundaries are expressed in: it advances the same way however the player
    /// is doing, and on every difficulty.
    pub script_frames: i32,
    pub random_seed: u16,
    pub deaths: i32,
    pub lives: i8,
    pub bombs: i8,
    pub power: u16,
    pub enemy_count: i32,
    pub bullet_count: i32,
    pub laser_count: i32,
    pub boss_present: bool,
    pub boss_life: Option<i32>,
    /// Frames the boss's current attack has been running. Resetting is what marks
    /// the move to the next attack, spellcard or not.
    pub boss_attack_frames: Option<i32>,
    /// The spellcard being fought, if any.
    pub spellcard: Option<u32>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scene={}->{} stage={} diff={} frames={} script={} seed={:#06x} \
             deaths={} lives={} bombs={} power={} enemies={} bullets={} lasers={} boss={}",
            self.scene,
            self.wanted,
            self.stage,
            self.difficulty,
            self.stage_frames,
            self.script_frames,
            self.random_seed,
            self.deaths,
            self.lives,
            self.bombs,
            self.power,
            self.enemy_count,
            self.bullet_count,
            self.laser_count,
            self.boss_present,
        )?;
        if let (Some(life), Some(frames)) = (self.boss_life, self.boss_attack_frames) {
            write!(f, " boss_life={life} attack_frames={frames}")?;
        }
        if let Some(spellcard) = self.spellcard {
            write!(f, " spell={spellcard}")?;
        }
        for (flag, name) in [
            (self.unsettled, "unsettled"),
            (self.bombing, "bombing"),
            (self.in_dialogue, "dialogue"),
            (self.paused, "paused"),
            (self.practice, "practice"),
            (self.replay, "replay"),
            (self.demo, "demo"),
            (self.in_ending, "ending"),
        ] {
            if flag {
                write!(f, " {name}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Build, KNOWN, Known, known_by_exe, known_named, th06};

    /// Every entry is reachable by the name its exe has, which is the whole of how a process is
    /// recognised: an entry no name finds is a game orb carries addresses for and never uses.
    #[test]
    fn every_game_in_the_table_is_found_by_its_own_exe() {
        for known in KNOWN {
            let found = known_by_exe(known.exe).expect("an entry found by its own exe");
            assert_eq!(found.builds_named(), known.builds_named());
        }
    }

    /// The ASCII of a name folds, since a copied exe is often renamed in capitals, and the kanji of
    /// one is matched as written.
    #[test]
    fn an_exe_renamed_in_capitals_is_the_same_game() {
        assert!(known_by_exe("東方紅魔郷.EXE").is_some());
        assert!(known_by_exe("東方紅魔郷.exe").is_some());
    }

    /// And a name no entry holds is no game, which is the answer that leaves a process untouched.
    ///
    /// The whole name and not a part of it: a game's own data file carries the game's name, and a
    /// process orb attached to on the strength of that would be reading the wrong memory.
    #[test]
    fn a_name_no_entry_holds_is_no_game() {
        assert!(known_by_exe("notepad.exe").is_none());
        assert!(known_by_exe("").is_none());
        for known in KNOWN {
            assert!(known_by_exe(&known.exe.replace(".exe", ".dat")).is_none());
            assert!(known_by_exe(&format!("v{}", known.exe)).is_none());
        }
    }

    /// What a refusal on either side of the table names: the exe to look for and every build its
    /// addresses were read off, one entry apiece.
    #[test]
    fn the_games_are_named_by_exe_and_build() {
        let named = known_named();
        for known in KNOWN {
            for build in known.builds {
                assert!(
                    named.contains(known.exe) && named.contains(build.version),
                    "{named:?} does not name {} {}",
                    known.exe,
                    build.version,
                );
            }
        }
        assert_eq!(named.matches(", ").count(), KNOWN.len() - 1);
    }

    /// A game with more than one build names them all, since which of them an exe is is not something
    /// the DLL can tell: the md5 is read before a process exists, and by the attach there is nothing
    /// left to read it off.
    #[test]
    fn a_game_of_more_than_one_build_names_every_one_of_them() {
        let patched = Known {
            exe: "東方紅魔郷.exe",
            cfg: "東方紅魔郷.cfg",
            builds: &[
                Build {
                    md5: "fa3d64768b1bfc50703dedc2db92f7fa",
                    version: "1.02h",
                },
                Build {
                    md5: "d41d8cd98f00b204e9800998ecf8427e",
                    version: "1.02h with its text swapped out",
                },
            ],
            game: &th06::Th06,
        };
        assert_eq!(
            patched.builds_named(),
            "1.02h or 1.02h with its text swapped out"
        );
    }
}
