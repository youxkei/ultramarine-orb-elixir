//! The seam between orb and the game it is running inside.
//!
//! Everything above this module — chapters, snapshots, the retry menu, the
//! overlay — is written against [`Game`] and [`State`] rather than against
//! th06. The point of the seam is that porting to another Touhou game means
//! writing addresses and offsets, not reworking how chapters or retries behave.
//!
//! Two implement it. [`th06`] answers everything, and [`th07`] answers what has
//! been measured of th07 — a frame, and nothing about a run — and declines the
//! rest, which is how the list of what a second game still needs is found rather
//! than guessed at. So orb paces th07 and plays th06. Which of them a process
//! is is [`KNOWN`].
//!
//! **One table names every game orb knows, and both halves read it.** [`KNOWN`] holds, per game, the exe's
//! own file name, the file the game keeps its configuration in, the builds the addresses were read off — an
//! md5 and what that build is called, apiece — and the [`Game`] those addresses are in. The launcher finds
//! the game by which entry's exe is in the directory it was pointed at, and refuses an exe that is none of
//! that entry's builds, naming every game and build orb knows; the build it *is* is what the line a launch
//! prints names, since a report of a fault is read against what was played and the file's own name does not
//! say which. The DLL is already inside the process by the time it can ask anything, so it matches on the
//! exe's own file name — case-insensitively over the ASCII of it — and names in the log every build its
//! addresses were read off, which of them this one is being the md5's to say and the md5 being read before
//! the process existed. A process no entry names is one where orb patches nothing, says which games it
//! knows, and does nothing else.
//!
//! **A list of builds rather than the one build, because a translation patch makes another one.** orb reads
//! the game's state through absolute addresses, so what an entry has to hold is every exe those addresses
//! are right for — and an exe with the game's text swapped out is a different file with the same code in it.
//! Each build in the table is one somebody has had orb's addresses read against; what is not in it is
//! refused, whatever it says about itself.
//!
//! The choice is made once, at the attach, and kept in a static the hooks read: a hook is a plain `extern`
//! function with nothing but the ABI's arguments, so where it would be handed a game it reads one — the same
//! reason the frame loop's two calls into the game are statics.
//!
//! One table rather than a constant apiece because two lists cannot be kept in step, and a launcher that
//! starts a game the DLL does not recognise is a launch where orb is loaded and silent.
//!
//! **Every address answers a measurement and none answers a guess**: an address written down because it is
//! where another game keeps the same thing is the one thing that must not happen. The addresses in th07's
//! frame lined up with th06's and the *shape* of the frame did not, and orb's first loop over it took the
//! game down on its first frame.

/// Where to call one of the game's own functions, which is its address in a real process and whatever a
/// game laid out by hand hands over in its place: a reader, a setter, and the slot behind them.
///
/// **Code is the one thing an address space laid out by hand cannot hold**, so every call a game in here
/// makes into itself that an e2e test reaches has to be answerable — a shake still running at a stage
/// move is taken down through one of th06's, and the queue of quads a frame of th07's drawing fills is
/// emptied and drawn through two more. An e2e test without them would be jumping into memory nothing has
/// mapped. The same answer `window::install_over` and `score::install_over` are.
///
/// Here rather than in either game because both games have such calls, and a macro apiece is one of them
/// drifting. The slot is behind the same gate the laid-out images are, so the shipped DLL has the
/// constant and no atomic in the path. The setter is not `install_`, which everything wearing that name
/// in this tree does by writing over something — an import table entry or a prologue: this patches
/// nothing and records where a function is.
// **Allowed rather than gated, because what declares nothing through it is a whole target.** Both of the
// games with such calls are 32-bit builds and this crate is built for `x86_64-pc-windows-gnu` as well,
// where th06c is the only game compiled in and it hands over its frame without an address — see
// `th06c::THE_FRAME`. A `cfg` on the arch here would say that in a place nothing reads it.
#[allow(unused_macros)]
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

// **A game is a module of the architecture its exe is**, and not of every build of orb: th06 and th07 are
// 32-bit processes whose every address, calling convention and struct offset is a 32-bit one — their
// `__thiscall`s do not exist as an ABI anywhere else — where th06c is a 64-bit process of its own. A build
// of orb carries the games it could attach to and no others, which is also what [`KNOWN`] then lists.
#[cfg(target_arch = "x86")]
mod song;
#[cfg(target_arch = "x86")]
pub mod th06;
#[cfg(target_arch = "x86_64")]
pub mod th06c;
#[cfg(target_arch = "x86_64")]
pub mod th06nc;
#[cfg(target_arch = "x86")]
pub mod th07;
#[cfg(target_arch = "x86")]
pub mod th08;
#[cfg(target_arch = "x86")]
pub mod th10;

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
/// inside a process no entry names does nothing at all.
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
#[cfg(target_arch = "x86_64")]
pub const KNOWN: &[Known] = &[
    Known {
        exe: "th06c.exe",
        cfg: "th06.cfg",
        builds: &[Build {
            md5: "3d01e7d946556d3a28a738064a8efdc7",
            version: "the Steam build of md5 3d01e7d9",
        }],
        game: &th06c::Th06c,
    },
    // The other Steam release of the same game, on th06c's engine and with every address its own — see
    // [`th06nc`]. The same `th06.cfg` beside a different exe, which is what tells the two directories apart.
    Known {
        exe: "th06nc.exe",
        cfg: "th06.cfg",
        builds: &[Build {
            md5: "fe47d9c26828e4671a2d2e4bf00af5a4",
            version: "the Steam build of md5 fe47d9c2",
        }],
        game: &th06nc::Th06nc,
    },
];

#[cfg(target_arch = "x86")]
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
    // The second game a run of is chaptered: [`th07::Th07`] answers `rewinds` with `true`, so a launch
    // here gets orb's own frame, its overlay, its chapters and the score file a rewound run's numbers
    // belong in. What version this build is the file name does not say and nothing in it has been read
    // that does, so the md5 is the whole of what pins it — which is what the md5 is for in the other
    // entry too.
    Known {
        exe: "th07.exe",
        cfg: "th07.cfg",
        builds: &[Build {
            md5: "0126afce1e805370d36c3482445e98da",
            version: "the build of md5 0126afce",
        }],
        game: &th07::Th07,
    },
    // The third: [`th08::Th08`] answers `rewinds` with `true` as well, so a launch here gets what one of
    // th07 does. Pinned by the md5 alone for the reason th07's build is, and what of it is still not read
    // is at the end of `th08/asm.md`.
    Known {
        exe: "th08.exe",
        cfg: "th08.cfg",
        builds: &[Build {
            md5: "77b6785e04a3406e50be68714a193650",
            version: "the build of md5 77b6785e",
        }],
        game: &th08::Th08,
    },
    // The fourth: [`th10::Th10`] answers `rewinds` with `true` as well, so a launch here gets what one of
    // th07 and th08 does. Pinned by the md5 alone, and what of it is not read is at the end of
    // `th10/asm.md`.
    Known {
        exe: "th10.exe",
        cfg: "th10.cfg",
        builds: &[Build {
            md5: "7dc488d82c81dd4aee4ba098b8804d83",
            version: "1.00a",
        }],
        game: &th10::Th10,
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
/// touched.
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

/// The name a game's own name entry holds, out of the field it is padded into — see
/// [`Game::the_name_its_own_entry_holds`], which both games answer through this.
///
/// **Spaces off both ends and nothing else touched.** The field is a fixed width the entry fills with
/// spaces, and the entries let a space be chosen as a character too, so a name with one inside it is a name
/// somebody typed and stays as it is.
///
/// **ASCII, which is what both entries offer** — 96 characters walked with a cursor, and letters, digits and
/// punctuation are all of them. A field holding anything else is one nothing typed into, and answering
/// nothing there is what keeps a name orb writes into a file it owns to what the game itself would have
/// written.
pub(crate) fn the_name_held_at(address: usize, bytes: usize) -> Option<String> {
    let held = orb_api::mem::read_committed_bytes(address, bytes)?;
    if held
        .iter()
        .any(|byte| !byte.is_ascii_graphic() && *byte != b' ')
    {
        return None;
    }
    let name = String::from_utf8(held).ok()?.trim().to_owned();
    (!name.is_empty()).then_some(name)
}

/// A name written into the field a game's own entry types into, padded with the spaces that entry pads
/// with — see [`Game::types_its_name`], which both games would answer through this.
///
/// **Refused rather than cut short** where the name does not fit the field or holds a character the entry
/// has no item for: a ranking under half a name is worse than one under the name the entry came up
/// holding, and the step that asked for it says so by never ending.
///
/// **The same ASCII [`the_name_held_at`] reads**, which is the whole of what those grids offer.
pub(crate) fn the_name_typed_into(address: usize, bytes: usize, name: &str) -> bool {
    if name.len() > bytes
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
    {
        return false;
    }
    let mut field = name.as_bytes().to_vec();
    field.resize(bytes, b' ');
    for (offset, byte) in field.into_iter().enumerate() {
        unsafe { orb_api::mem::write(address + offset, byte) };
    }
    true
}

/// **Where a game's own read of the keyboard is**, which is not the same kind of place in every game.
///
/// th06 and th07 call theirs from their own code at an address, so what stands in front of one is a
/// [`Patch`]: the first bytes jump to orb's and the prologue is kept to run. th06c calls nothing of the
/// kind — it asks `GetProcAddress` for its read at startup and keeps the answer in a table of its own —
/// so what stands in front of that one is a [`Lookup`]: orb answers the ask, and the pointer the game
/// keeps is orb's because the game wrote it there itself.
pub enum Read {
    Code(Patch),
    LookedUp(Lookup),
    /// **The pad's half of a read whose keyboard half stores what that half answers**, for a game whose read
    /// writes the word into its own buffer before it returns: th10's `0x44a5f0` stores `ax` at `0x474e30`
    /// itself, so what a hook over it answers reaches nobody — and the word it stores is the one
    /// `0x44a190` hands back, entered with the keyboard's word in `ecx` and the player and the pad pushed
    /// (`push 0; push ebx; mov ecx,edi; call 0x44a190` at 0x44a8b3). See
    /// `runtime::get_input_with_the_keys`. **Called through**, so the pad's buttons and stick are the game's
    /// own, and what [`Game::pad_word`] answers is added to it: over such a game, what its half leaves out.
    PadHalf(Patch),
}

/// **A function the game asks for by name at startup**, which orb answers in place of the host.
///
/// **The ask and not the slot the answer lands in.** That slot is the game's own to write and holds
/// nought until it does, so a pointer written there at the attach is one the game's startup overwrites:
/// measured, th06c's keyboard slot held `0x0` when orb read it. Answering the ask needs no address at
/// all, which is what makes it hold across a build and across where the loader put the exe.
///
/// See `th06c/asm.md`, where the three hundred and forty-two names this kind of game asks for are
/// listed.
pub struct Lookup {
    /// **The name, which is the whole of what identifies it.** Not the module beside it: the handle the
    /// game passes is one it got wherever it liked, and a name like `GetKeyboardState` belongs to one
    /// module in any process.
    pub name: &'static str,
}

/// **The controls of a game's own startup dialog that a launch playing itself clicks**, in the order they
/// are clicked: `None` for a game that puts no dialog up.
///
/// **Clicked rather than the dialog being skipped.** What each click settles is the dialog procedure's
/// own business — th06c reads its radio buttons with `IsDlgButtonChecked` when the start button is
/// pressed — so a dialog orb answered by never showing it is a launch whose renderer was never told
/// which window to make. Clicking is what a person does, and it settles the same things.
pub struct Dialog {
    pub clicks: &'static [i32],
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
/// **The two walks orb's own frame loop calls, handed over rather than patched.**
///
/// **Which is what a game whose whole frame orb replaces needs.** Nothing else calls either walk then —
/// the frame that did is orb's now — so there is no prologue to be in the path of, and the loop calls
/// orb's own two hooks with these two behind them. It is also the only way in for a walk a game has no
/// function of its own for, th06c's draw chain being walked inline in the frame orb replaced, and for one
/// whose answers the loop cannot read as they come: th06c's calc walk answers a byte saying whether the
/// frame is leaving and writes the answer it leaves with through a pointer.
///
/// Addresses for the reason [`FrameCalls`] holds addresses, and the same two statics behind them.
pub struct Walks {
    /// `extern "fastcall" fn(*mut c_void) -> i32`, called where the patched calc chain would be.
    pub update: usize,
    /// The same, for the draw chain.
    pub draw: usize,
}

pub struct FrameCalls {
    /// Hands queued sound effects to the sound system, as the game does once per frame after its
    /// update. Runs on the game's main thread.
    pub play_sounds: Call,
    /// The game's own present, which also handles a lost device and its own screenshot key. Runs on
    /// the game's main thread with the frame drawn.
    pub present: Call,
    /// **Whether these two are functions of orb's own rather than the game's.**
    ///
    /// True for a game whose frame has neither as a function to call — th06c's sound catch-up is inline
    /// in the frame orb replaced and its present is inside one of the draw chain's jobs — so what the two
    /// above name is code in orb and not an address in the exe.
    ///
    /// **What reads it is `runtime::attach_to`**, which is the laid-out game's way in. A game's *own* two
    /// have nothing to be called on there: a laid-out game is an address space and not code, so a call to
    /// an exe address would jump into memory nothing has mapped, and what a test hands over in
    /// `Originals` stands in for them. Where they are orb's they are the same code in a test as in a run,
    /// and standing anything in for them is the divergence — a launch of th06c whose present was a test's
    /// own never reaches `look_for_the_swapchain`, so orb has no swapchain, no canvas and no overlay,
    /// which is a whole screen of orb's that no e2e test could see.
    pub orbs_own: bool,
}

pub struct Hooks {
    /// One call per logic frame. Not calling through freezes the game.
    ///
    /// `None` for a game whose frame orb has not been read far enough into to stand in: a launch there
    /// gets no frame loop of orb's own and no chapters, and what it does get is whatever hooks below
    /// are named. And `None` for a game whose two walks are handed over instead — see [`Hooks::walks`],
    /// and [`th06c`], whose frame is orb's that way.
    pub update: Option<Patch>,
    /// Called inside the scene the game draws into, the only place an overlay
    /// may draw. `None` on the same terms as [`Hooks::update`].
    pub draw: Option<Patch>,
    /// **The two walks handed over instead of patched**, for a game whose frame is orb's — see
    /// [`Walks`], which has why a patch cannot reach either of th06c's.
    ///
    /// `None` for a game whose walks are the two patches above, and where both are named this is what
    /// wins: it is the loop's own two calls, and the patches would be a second answer to the same
    /// question.
    pub walks: Option<Walks>,
    /// **And where the argument of those three arrives** — [`Hooks::update`], [`Hooks::draw`] and
    /// [`Hooks::render`]; see [`FrameEntered`].
    pub frame_entered: FrameEntered,
    /// **One call per frame of the game's own, for a game orb does not own the frame of** — the walk of
    /// the calc chain, which orb is told about here rather than standing in the place of.
    ///
    /// **Which is not [`Hooks::update`] and cannot be it.** That one is the frame loop's: naming it
    /// hands orb the update, the draw, the sound and the present, and a game that names it and not the
    /// three others gets no hook installed at all. This is the seam a game orb is *outside* the frame of
    /// has instead, and all it decides is when the rules are asked for a word and when a recipe's step
    /// is walked.
    ///
    /// **Because the read a game like that is otherwise driven off is not its frame.** th06c is asked
    /// for a word by its own keyboard read, and that read is made more often than the game moves: with
    /// this hook in place a launch counted 683, 692, 736, 761 and 773 reads against every 600 walks of
    /// the calc chain, so between a seventh and a quarter of the words were decided off a world that had
    /// not moved since the word before. See `orb_bot::steer::a_tick_read_again`, which is what answers
    /// those where this is `None`, and `th06c/asm.md` for the frame itself.
    pub runs_its_frame: Option<Patch>,
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
    pub stage_building: Option<Building>,
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
    /// `None` for a game whose score file is written as its ranking screen goes down, which is th06:
    /// there the record in memory is what the write takes and nothing lays a file over it first.
    pub card_records_read: Option<Patch>,
    /// Runs after the config is read and before the window and device exist, which
    /// is the only moment `force_windowed` can still take effect.
    ///
    /// **Entered with the instance on the stack or in `ecx`**: th06's `GameWindow::CreateGameWindow` is pushed
    /// it (0x420404) and th07's and th08's are handed it in `ecx` (`mov ecx,[ebp+0x8]` at 0x4340da and
    /// 0x441996), each testing the answer after.
    pub create_window: Option<Entered>,
    /// Runs just after the device is created, and again after every reset: the
    /// first moment its vtable can be redirected.
    pub init_device: Option<Patch>,
    /// The game's whole frame: its draw, its update, its pacing and its present.
    /// Replacing it is what lets orb pace the game and drop a frame of input lag.
    pub render: Option<Patch>,
    /// Everything the game reads from the keyboard, once a frame. Needed because
    /// orb keeps updating while the window is not in front, and a game that carried
    /// on reading the keyboard then would act on keys meant for something else.
    pub input: Option<Read>,
    /// The part of the input read that goes to a joystick, if the game keeps it separate. Hooked and
    /// **not called through**: what every pad does to the word is [`Game::pad_word`], so a launch that
    /// left this out is a launch where no pad does anything. `None` for a game whose pad read has not
    /// been measured, which is a game orb reads no pad in at all.
    pub joystick: Option<Entered>,
    /// The game's own refill of the music buffer, run on its streaming thread: one chunk of the track's file
    /// into the buffer, at the offset the game keeps for it.
    ///
    /// **The one place a sound becomes audible**, which is what makes it the place a log can be held to. Every
    /// other line about the music is a position, a cursor or a countdown read from outside, and a track looping
    /// over a section of itself changes none of them — the same track goes on playing, its buffer goes on being
    /// fed, and what is coming out is a few seconds of the song over and over. Read at the refill, that is the
    /// offsets it reads from going round a bounded range, which is a thing a log says outright.
    ///
    /// A [`Patch`] and not an [`Entered`]: both games' threads call their refill the same way — `__thiscall`
    /// on the stream with one argument on the stack — so there is one shape for the hook to be entered by.
    ///
    /// `None` for a game whose refill has not been read, which leaves its music with nothing said about what
    /// is being fed into it.
    pub tops_up_the_music: Option<Patch>,
    /// And the game's own stop of it: th07's stream stop at 0x45d910, which clears the playing flag and stops
    /// the buffer, and th06's whole `SoundPlayer::StopBGM`.
    ///
    /// **The other half of where a sound is made.** The refill says what is being fed and a stop says the
    /// feeding is over, and neither can be read off the other: a buffer stopped between two frames is a buffer
    /// every reading from the frame calls stopped without saying when or by whose ask, and the silence this
    /// session chased was three different asks — the pause menu's, a fed frame's, and a restore's.
    ///
    /// **Every stop through that function, whoever asked for it**, orb's own calls of it included: what tells
    /// them apart is orb's own line beside this one, and a stop with no such line is the game's.
    ///
    /// `None` for a game whose stop has not been read.
    pub stops_the_music: Option<Patch>,
    /// The pad's buttons one byte apiece, which is what a game's own key config screen reads to learn
    /// *which* button was pressed rather than what it means. Hooked and **called through**: what the game
    /// found stands, and [`Game::add_pad_buttons`] adds the pads orb read for itself.
    ///
    /// A read of its own because a mapping screen cannot use a mapped word — it is the screen the mapping
    /// is made on. `None` for a game with no such screen, which is th06: its buttons are configured by
    /// `custom.exe` beside it and nothing in the game reads a pad by number.
    pub pad_buttons: Option<Patch>,
    /// The dialog the game puts up before it starts anything, and the controls a launch that plays
    /// itself clicks to get past it — see [`Dialog`].
    ///
    /// Only a launch that plays itself: a person at the keyboard is somebody the dialog is *for*, and
    /// answering it for them would take away the choice the game came up to ask about.
    ///
    /// `None` for a game that puts no dialog up, which is th06 and th07 — 1.02h's own settings are
    /// `custom.exe`'s beside it, and th07's are a screen inside the game.
    pub startup_dialog: Option<Dialog>,
    /// **The name the game asks for to make its window**, for a game that looks that up rather than
    /// importing it — see [`Lookup`], and [`Hooks::create_window`] for the moment a game that imports it
    /// is got in front of instead.
    ///
    /// What it buys is the one thing a launch that plays itself wants of a window: somewhere out of the
    /// way. Only the corner is taken — the size and the style are the game's own, being what its own
    /// settings just chose.
    ///
    /// `None` for a game whose window creation is in its import table, which is th06 and th07.
    pub makes_its_window: Option<Lookup>,
    /// **And the name it asks for to move that window afterwards**, which is the other half of placing
    /// one: a game that centres its own window undoes wherever it was made.
    ///
    /// th06c does — it looks up `MonitorFromWindow`, `GetMonitorInfoW`, `GetSystemMetrics`,
    /// `ClientToScreen` and `MoveWindow` together, and a window orb had made in the corner was found in
    /// the middle of a screen two seconds later. So a launch that plays itself answers this one too, and
    /// what it answers with is the rectangle it already decided.
    ///
    /// `None` for a game that leaves its window where it made it, which is th06 and th07.
    pub moves_its_window: Option<Lookup>,
    /// **And the name it asks for to put that window on the screen**, which is what a window made hidden
    /// has to be answered by: `orb_core::window::create_window_ex_w` strips `WS_VISIBLE` so that no
    /// put-back of the corner is seen, and a game that shows its own window undoes that.
    ///
    /// th06nc does — a launch whose window was made hidden was measured on the screen a frame later with
    /// orb having shown nothing — so what this answers keeps the window off the screen until the corner
    /// has held and lets every later call through.
    ///
    /// `None` for a game whose window orb does not hide, which is th06 and th07.
    pub shows_its_window: Option<Lookup>,
    /// **And the name it asks for to place it**, `SetWindowPos` being the one call that moves a window
    /// and shows it at once — so a game that looks it up can undo both halves.
    ///
    /// `None` for a game that places its window through neither, which is th06 and th07.
    pub places_its_window: Option<Lookup>,
    /// **And the name it asks for to write that window's styles**, a style write being both a frame taken
    /// off and a show: th06c and th06nc write their window mode's extended style — no dialog frame — and
    /// then a style with `WS_VISIBLE` in it, which put the window orb plays in on the screen as an ordinary
    /// window before orb had shown it. See `orb_core::window::set_window_long_w`.
    ///
    /// `None` for a game whose window's styles are its `CreateWindowExA` arguments alone, which is th06 and
    /// th07.
    pub restyles_its_window: Option<Lookup>,
    /// **The name it asks for to read a pad**, which orb answers by being one.
    ///
    /// **A pad rather than the keys, because a pad is read the same way whatever screen is up.** This
    /// game's keyboard comes two ways — a DirectInput device while a run is on and a translation from
    /// virtual keys while a menu is — so a key orb writes reaches a menu and not a run. An XInput pad is
    /// read through one call on every screen, and orb answering that call is a pad nothing had to
    /// install.
    ///
    /// `None` for a game whose pad orb reads for itself and hands over a word of — which is th06 and
    /// th07, where [`Hooks::joystick`] is the read that is stood in front of.
    pub reads_a_pad: Option<Lookup>,
    /// **The name it makes its renderer through**, answered so that the swapchain it then makes is one
    /// orb can present through — see `dxgi`, which is all of what that answer does.
    ///
    /// **Which is what a frame orb owns needs and nothing else does.** The loop presents once per frame,
    /// and a game whose own present is inside its draw chain would present twice — so the game's is
    /// taken over and orb's is the one that reaches the display. `None` for a game whose frame is its
    /// own, and for the two whose present is an address `FrameCalls` already carries.
    pub makes_its_renderer: Option<Lookup>,
    /// **How the game takes its memory from the OS**, which is where a chapter's snapshot learns what a
    /// run is a copy of — see [`Memory`], and `orb::memtrack`, which stands in front of the imports named.
    pub takes_its_memory: Memory,
    /// **And how it makes its threads**, which are the ones a restore stops while it copies — see
    /// [`Threads`], and `orb::threads`.
    pub makes_its_threads: Threads,
    /// **And how it opens a file**, which is the import the score file's fork stands in front of — see
    /// [`FileOpen`], and `orb::score`.
    pub opens_its_files: FileOpen,
}

/// **The imports a game's own allocator reaches the OS through**, which is what says where a run's memory is.
///
/// A snapshot is a copy of every region the game owns and a restore writes them all back, the allocator's own
/// bookkeeping included — which is what keeps every pointer in them valid. So what has to be known is which
/// heap the game's `malloc` lands in, and that is a fact about the CRT the exe was linked against.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Memory {
    /// `HeapCreate`, `HeapAlloc`, `HeapReAlloc`, `HeapFree`, `VirtualAlloc` and `VirtualFree` in the exe's
    /// own `KERNEL32.dll` imports: a CRT linked statically, whose `malloc` goes to a heap it made for itself
    /// and nothing else allocates from. th06 1.02h and th07 are built this way, and the hooks over those six
    /// entries notice the heap and the reservations as the game takes them.
    Kernel32Imports,
    /// `GetProcessHeap`, `HeapReAlloc` and `HeapFree` in the exe's own `KERNEL32.dll` imports and no
    /// `HeapCreate`: the universal CRT linked statically, whose `malloc` is `HeapAlloc` on the heap
    /// `GetProcessHeap` answered it once at startup — which is the *process* heap, the one D3D11, DXGI,
    /// Steam and every other module in the process allocate from too, so a copy of it is a copy of theirs
    /// and a restore of it rewinds them. So what orb answers is that one question, with a heap made for
    /// the game alone, and every allocation the CRT makes after it lands there with no call of it stood in
    /// front of. th06c and th06nc are built this way: th06c asks from one place, `0x1402adf64`, and keeps
    /// the answer at `0x140955178`; th06nc asks at `0x1402e72d4` and keeps it at `0x140abac18`.
    ///
    /// **The question and not the allocations**, because the allocations have no one entry to stand at:
    /// `malloc`, `calloc`, `realloc`, `_aligned_malloc`, the CRT's own stdio buffers and its locale
    /// tables are each a function of the exe's, and what they share is the handle the CRT asked for. The
    /// builds before these imported `malloc`, `free` and `realloc` out of the universal CRT's DLL, and
    /// what stood in front of those went with them.
    ProcessHeapImport,
}

/// **The import a game makes its threads through**, which is how orb learns which threads are the game's own:
/// those are stopped while a chapter's memory is copied, and the graphics driver's and the sound mixer's are
/// not — see `orb::threads`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Threads {
    /// `CreateThread` in the exe's `KERNEL32.dll` imports, which is every game's: th06 1.02h's and th07's
    /// own calls, and the one call inside the `_beginthreadex` th06c's and th06nc's CRT links into the exe.
    CreateThread,
}

/// **The import a game opens a file through**, which is where the score file is sent to the mode's own — see
/// `crate::score`, and `orb::score`, which writes over the entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileOpen {
    /// `CreateFileA` in the exe's `KERNEL32.dll` imports, which both of th06 1.02h's paths to the file end
    /// at, and th07's.
    CreateFileA,
    /// `CreateFileW` in the exe's `KERNEL32.dll` imports, which is where every `fopen` of a UCRT linked
    /// statically ends — the path widened out of the game's code page on the way. th06c's and th06nc's:
    /// the score file, the replays and the data archives alike.
    CreateFileW,
}

/// A patch, and where the argument of the function under it arrives.
///
/// **Because a hook is entered by its ABI and nothing else**, and MSVC compiled the same call differently
/// in the two games. The pad read is the pair this was first needed for: th06's
/// `Controller::GetControllerInput` takes the word the keyboard was read into on the stack — `push edx;
/// call 0x41cfc0; add esp,4` at 0x41dc77 — and th07's takes it in `ecx`, `mov ecx,[ebp-0x104]; call
/// 0x4303f0` at 0x430f1a. A stage's own beginning is the second: th06's added callback is `__cdecl` and
/// th07's is `__thiscall`, its chain calling every callback as `mov ecx,[elem+0x1c]; call [elem+0x4]`.
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

/// **How the call [`Hooks::stage_building`] stands over is entered**, which is [`Entered`]'s question for a call
/// that takes something other than one argument: th06's `Stage::RegisterChain` and the builds of th06c and
/// th06nc take one number and leave the stack to their caller, th07's and th08's take theirs in `ecx` — the
/// supervisor's `mov ecx,[eax+0x95ec]` at 0x42f0ae and the build's `mov ecx,[eax+0x3ddc4]` at 0x43b3dc, with
/// nothing pushed — and th10's `0x429610` takes two and pops them itself (`ret 0x8` at 0x4296b2).
///
/// **th07's and th08's were `Cdecl` once**, which read the stage off a stack nothing had pushed it on and
/// called on with `ecx` written over, so the build kept whatever the register held: nothing read the field it
/// keeps it in, which is how it went unseen.
pub enum Building {
    Cdecl(Patch),
    InEcx(Patch),
    StdcallOfTwo(Patch),
}

impl Building {
    /// The patch any of them carries.
    pub fn patch(&self) -> &Patch {
        match self {
            Self::Cdecl(patch) | Self::InEcx(patch) | Self::StdcallOfTwo(patch) => patch,
        }
    }
}

/// **Where the one argument of the game's two chain walks and of its whole frame arrives** — the three
/// patches [`Hooks::update`], [`Hooks::draw`] and [`Hooks::render`] stand over, which a game compiles one
/// way for all three.
///
/// **Beside [`Entered`] and not one of it per patch**, because the three are not three facts: what enters
/// them is the game's own frame and message pump, and a game whose walks took the chain in `ecx` while its
/// frame took the window on the stack is one nobody has read. th06, th07 and th08 enter all three as
/// `__thiscall`s, the argument in `ecx`; th10 pushes it — `push [0x491be4]; call 0x449c00` and
/// `push 0x4924f0; call 0x439390`, each returning with `ret 0x4` — so a hook entered the other way reads its
/// argument out of whatever `ecx` held and returns four bytes short.
///
/// A 64-bit game has one convention, which is what `InEcx` names there: the argument in `rcx`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameEntered {
    InEcx,
    OnTheStack,
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
    /// The replays, of which there is a kind per mode — and this is the one item where the two answers
    /// do different things rather than the same thing to different files. A normal run's replay is the
    /// game's own, kept in the game's own files and shown on the game's own screen; a pointdevice run's
    /// is orb's, the game having no way to record one — see [`crate::replay`] — so orb's own list is
    /// what comes up for it.
    Replays,
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

    /// Two pads' worth of pushing as one. th06 is a one-player game, so every pad on the machine is
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
/// character is another run — its buttons would play somebody else's shot — which is why th15 keeps
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
    /// **Which of the game's own modes the run is in**, for a game that has more than one: th06nc
    /// offers 標準モード and チャレンジモード and they start a run with different lives, so two runs
    /// alike in every field above are still two runs. **Nought where a build has one mode**, which is
    /// th06, th07 and th06c — see [`Game::run_slot`], which is what the number has to reach, and
    /// `game::th06nc::game_manager` for where this game keeps it.
    pub mode: i32,
}

/// **There is no shared shape for the run's numbers.** What a stage has to start with is the game's own —
/// see [`Game::run_state`], which hands them over as that game's bytes, and the two games' own
/// `resume` modules, where each says what its own file holds.
///
/// It was one struct, with th06's rank, power items and extra lives beside th07's cherry, and each game
/// wrote nothing into the other's fields. What that cost was not the six wasted numbers: it was that the file
/// they went into had one version between the two games, so a field th07 needed could not be added without
/// refusing every th06 run on a disk. It was added without the version being raised, a run of th07 was
/// picked up out of a file that predated it, and the playback parted from the run with nothing in the log
/// able to say where.
/// A face the host is asked for by name, as the game itself asks for one: the family, whether it is wanted
/// bold, the height that ask carries, and how much bigger than the size it is drawn at that comes out.
///
/// What a game that ships no font of its own draws its text with — see [`Game::own_face`].
///
/// **The family alone is not the same text on the screen**, which is what the last two are for. The size orb
/// draws its own lines at is orb's, but *how a glyph of that size is made* is the game's: th07 asks GDI for
/// a height of `size * 2 - 2` — positive, so the whole cell rather than the em — and bakes into a bitmap twice
/// the size before that is drawn back down. Baked at the size instead, the same family comes out with different
/// stroke weights and edges, which a th07 session saw as text that did not look like the game's.
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

/// **What became of a death the game's own count has taken**, which the frame that count moves on does not
/// settle in every game.
///
/// **Because a game may count the hit rather than the death.** th06's `Player::Die` at 0x427770 puts the
/// count up as the hit lands and leaves its own deathbomb window open behind it, so the frames in which a bomb
/// still saves the run are frames whose count has already moved; nothing ever brings that count back down,
/// the bomb included. th07 counts in the death handler instead, once the window has closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountedDeath {
    /// The window is still open: a bomb inside it takes the death back, so what the count moved for is not
    /// decided yet.
    Open,
    /// It stands, and the run has lost the life whatever the panel has drawn of it so far.
    Taken,
    /// A bomb took it back and the run goes on, the count keeping the hit it will not count again.
    Bombed,
}

pub trait Game {
    fn hooks(&self) -> Hooks;

    /// **Where the exe turned out to be loaded**, said once at the attach and before any address of
    /// this game's is used.
    ///
    /// Nothing by default, which is a game whose exe is where it asked to be: th06 and th07 are 32-bit
    /// images the loader puts at the 0x400000 their addresses are written against, and there is nothing
    /// to move. A game built with ASLR on gets the ask honoured only until something else is there
    /// first — th06c came up at a different base on every launch measured — and such a game keeps what
    /// it is told here and puts its own addresses through it.
    fn set_image_base(&self, base: usize) {
        let _ = base;
    }

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
    /// would go to orb's file having never been rewindable. Which is what th07 did before this was
    /// asked — a `pointdevice_score.dat` in a th07 install, holding runs anybody could have played.
    ///
    /// `false` for a game orb has read no run of, which keeps that game's scores in its own file and
    /// orb's question off its menus.
    fn rewinds(&self) -> bool;

    /// **The file the game keeps its scores in, and orb's beside it** — `score.dat` for every game but th10.
    fn score_files(&self) -> crate::score::Files {
        crate::score::SCORE_DAT
    }

    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn read_state(&self) -> State;

    /// **Where a death [`State::deaths`] has just counted stands**, asked every frame from the one the count
    /// moved on until it is answered with anything but [`CountedDeath::Open`].
    ///
    /// [`CountedDeath::Taken`] by default, which is a game whose count moves once its own deathbomb window has
    /// closed: th07's death handler moves it at 0x4410a0, after the frames a bomb could have been pressed in,
    /// so a death it has counted is a death.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn where_a_counted_death_stands(&self) -> CountedDeath {
        CountedDeath::Taken
    }

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
    /// one is the *game's* predicate, and th07's counts the line it hands the screen over on — the one no words
    /// are drawn for. A recipe waiting for a conversation to photograph waited on that in stage 5 and stage 6
    /// and got the frame the box was still being drawn on.
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

    /// **The name the game's own name entry holds**, which is what a replay of orb's own is named by — see
    /// [`crate::replay`].
    ///
    /// Both games ask for one on the screen a played run ends at, before the question about saving a replay
    /// that orb's own offer stands in front of, and both keep it as eight bytes with spaces to the end. So
    /// the name a run's replay carries is the name that run's player typed into the game itself, and orb asks
    /// nobody for one.
    ///
    /// **Whatever the entry holds, rather than what was typed on the frame it was left**: it comes up holding
    /// the name the last run was entered under, so a run whose player passes the screen without touching it
    /// is a run whose name is the one before it — which is what the game's own replay would have carried too.
    ///
    /// Trimmed of the spaces the field is padded with, and `None` where no such screen is up, where the field
    /// holds nothing but spaces, or where the game orb is in has no name entry read yet.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        None
    }

    /// **Types a name into that entry**: the characters written where the screen's own keys write them,
    /// and the entry left on the item that ends it so that one press finishes it.
    ///
    /// **Written rather than pressed for, because the entry cannot be pressed to a name.** Its cursor
    /// walks a grid of characters and a `decide` writes whichever one the cursor is on, so eight taps
    /// register eight of the same letter — which is what every ranking a recipe's clear ever wrote:
    /// `AAAAAAAA`. Walking the grid is thirty-odd presses a name and each of them blind, that cursor
    /// being the screen's own and not the one a recipe's `until` can wait for.
    ///
    /// **What it is not is a name orb asks for.** `the_name_its_own_entry_holds` reads what a run was
    /// entered under and orb asks nobody for one; this writes the name a *recipe* asked for, which is a
    /// launch that plays itself and has nobody at the keyboard to type one.
    ///
    /// False where no such entry is up, where the game has none read, or where the name has more
    /// characters than the field holds — so a step that asked for one waits and gives up rather than
    /// registering half of it.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn types_its_name(&self, name: &str) -> bool {
        let _ = name;
        false
    }

    /// **Whether that entry is up**, which is the one thing a recipe otherwise cannot wait for.
    ///
    /// **Because the entry opens inside the scene a run is played in**, so the step that types a name has to
    /// be the step already running when it opens: a `play` ends where the *run* does and the run's own end is
    /// past the entry, and every frame in between is one the rules are asked for a word on — which is a held
    /// shot writing whichever character the cursor starts on. See `recipe::Until::Name`, which is what a step
    /// waits for it by.
    ///
    /// False for a game whose entry has not been read, the same as [`Game::types_its_name`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn typing_a_name(&self) -> bool {
        false
    }

    /// **A word's keys written into the game's own buffer**, for a game orb is asked for a word by its frame
    /// rather than by its keyboard read.
    ///
    /// **Because that read has already happened by then.** [`Game::press_into`] writes into the snapshot the
    /// read was given and the game translates it; a frame hook runs after that translation, so a word left
    /// there reaches the game a whole frame later — and a frame of that is a bot walking into what it just
    /// decided to walk out of. Measured: a launch of th06c whose rules were moved onto its frame took five
    /// hits in two stages where the launch before it took none, every one of them with a bullet directly
    /// above the player, the move chosen away from it, and the room the model left it clear for all twelve
    /// frames looked at — which is the arithmetic of a player that did not move.
    ///
    /// False for a game that has no such buffer, which is every game orb is inside the frame of: theirs is
    /// the read itself.
    ///
    /// # Safety
    /// Must run on the game's main thread, between its keyboard read and the update that acts on it.
    unsafe fn presses_into_its_own_keys(&self, word: u16) -> bool {
        let _ = word;
        false
    }

    /// Notices what the game has counted of its own play time since the frame before, so that a span counted
    /// before anybody chose a mode can be carried onto whichever file the sitting turns out to write.
    ///
    /// **Nothing for a game whose score file does not record one**, which is what the default is: th06's
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

    /// **The `IDirect3DDevice9` the game draws through**, for a game whose device is that one and not
    /// [`Game::d3d_device`]'s Direct3D 8 — [`Device::NULL`] for every other game and until the game has made
    /// it. What orb draws its overlay with in th10, whose scene the game's own frame opens and closes.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn direct3d_9_device(&self) -> Device {
        Device::NULL
    }

    /// **What the game has queued to draw, drawn ahead of the overlay**, for a game whose drawing fills a
    /// queue of quads that its own frame draws out only after the draw chain. Nothing for a game with no
    /// such queue.
    ///
    /// Here rather than moving [`Game::end_drawing`]'s own draw: that one is where the game's frame has
    /// it, and drawing the queue a second time draws nothing, the draw setting its count to nought.
    ///
    /// # Safety
    /// Must run on the game's main thread, inside the scene the game draws into, after its draw chain.
    unsafe fn draw_what_is_queued(&self) {}

    /// **Whether the game draws every frame at the size its swapchain was made**, whatever size its window
    /// is — which is what `crate::dxgi::resizes_its_buffers` keeps a swapchain's buffers at the size asked
    /// for, while a launch orb plays holds the window small.
    ///
    /// `false` for a game that draws at whatever size its buffers are, which is every game with no
    /// swapchain and th06nc: that one lays its frame out over the size it last made its buffers, and a
    /// launch playing it is made small by its own window mode instead — see `Th06nc::prepare_frame`'s
    /// `320x180 out of the game's own 1280x720`.
    fn draws_at_the_size_it_asked_for(&self) -> bool {
        false
    }

    /// **Whether [`Game::d3d_device`] is a device orb draws through**, and so whether a null one is a
    /// frame to hand back.
    ///
    /// `true` for the two games orb has an overlay in: what it draws goes through that device, the scene
    /// either side of the draw chain is that device's, and a frame before the game has made one is a
    /// frame orb can neither pace nor draw in — see `runtime::render`, which hands those back.
    ///
    /// `false` for a game whose renderer orb has no device of. Its frame is still orb's to run — the
    /// update, the draw, the sound and the present are what that loop is made of — and what it gives up
    /// is the overlay, the picture a `screenshot` step takes and every question orb would put over the
    /// game, none of which has anywhere to be drawn. See [`th06c`], which draws through D3D11 and hands
    /// orb nothing.
    fn draws_through_its_d3d_device(&self) -> bool {
        true
    }

    /// The music stream, if one is playing and its objects look intact.
    fn music(&self) -> Option<Music>;

    /// **Whether a snapshot of this game is worth waiting for the music to come up for**, which is what
    /// `chapter`'s own `MUSIC_WAIT_FRAMES` is the bound on: a stage's start is snapshotted as soon as
    /// [`Game::music`] answers one, and on that bound if it never does.
    ///
    /// `true` for a game whose music a snapshot carries and a restore puts back. What the wait buys there is
    /// in that constant's own doc: a snapshot taken between tracks rewinds the streaming bookkeeping onto a
    /// buffer that has moved on, and the track then plays a fraction of a second over and over.
    ///
    /// `false` for a game whose music orb neither reads nor puts back, which has nothing for the wait to be
    /// about — and waiting it out costs that game the first 249 frames of every stage. Measured on th06c,
    /// whose [`Game::music`] answers `None` on every frame: the stage's own start was snapshotted at frame
    /// 249 rather than 9, a run hit at frame 185 had no chapter to be retried from, and the run carried on
    /// past a death that Pointdevice mode exists to take back — writing chapters whose path holds it, which
    /// a playback cannot put back because the frames a respawn holds everything on are frames of no stage.
    fn a_snapshot_waits_for_its_music(&self) -> bool {
        true
    }

    /// Identifies the track playing, changing when the game switches tracks. Used to tell a
    /// stage's own music from a boss's, which is what tells a midboss from the boss a stage
    /// ends with: a stage's data names two songs and the game plays the second for that
    /// fight alone.
    fn music_identity(&self) -> Option<u32>;

    /// **What the game's own refill is about to feed the music buffer**, as a line: where in the track's file
    /// the chunk it is going to read starts, where in the buffer it goes, and how much of the track is left
    /// before the loop is taken.
    ///
    /// `None` where the stream's numbers cannot be read, and for a game whose refill has not been read at all
    /// — see [`Hooks::tops_up_the_music`], which is where this is called from and the whole of why it exists.
    ///
    /// # Safety
    /// Runs on the game's *streaming* thread, from the hook over that refill and before the original: what it
    /// may do is read the stream it was handed and nothing else.
    unsafe fn the_refill_is_about_to(&self, stream: usize) -> Option<String> {
        let _ = stream;
        None
    }

    /// **Whether a refill the streaming thread was woken for and did not make is never made afterwards**, so
    /// that the stream is a chunk out from then on and orb puts it back — see `audio::Refills` for how a
    /// missed one is told from a late one.
    ///
    /// `true` is a reading of the thread, not a default: th06's is read at `Th06::a_missed_refill_is_never_made_up`.
    /// th07's refill refuses a write that would land inside the chunk the device is writing out of — `Stream
    /// Skip`, 0x45db60 — and what its thread does about a wake that was late, and whether a refusal is a refill
    /// owed or one forgiven, has not been read off the exe or watched on the game. Its missed refills are
    /// counted and said, and the stream is left alone: a repair built on reading th07's ordinary state as a
    /// fault rebuilt the stream on every retry and was taken out — see `crate::audio`.
    fn a_missed_refill_is_never_made_up(&self) -> bool {
        false
    }

    /// And what that track is *called*, where the game keeps the name of the one it is streaming.
    ///
    /// **`None` is not a game with no music**, it is a game that does not keep the name: th06 builds the
    /// `.wav` and `.pos` paths of a track in stack buffers at `Supervisor::PlayAudio`, hands them to
    /// `SoundPlayer::LoadBGM`, and keeps the mmio handle, the length and the loop points of what it opened —
    /// so there is nothing in its memory to read the name off. See `th06/asm.md`. What a line about its
    /// music has instead is [`music_identity`](Game::music_identity), which tells one track from another
    /// without naming either.
    fn the_track_heard(&self) -> Option<String> {
        None
    }

    /// Whether the song playing is the one the stage's data names for the fight it ends
    /// with, where the game says which of its two songs is on directly.
    ///
    /// `None` where it does not, and then a fight is told from a midboss by
    /// [`music_identity`](Game::music_identity) having changed since the stage began —
    /// which is what a game whose songs cannot be named by name is left with.
    ///
    /// **The two are not the same reading, and th07 is why.** A stage's first chapter is
    /// taken frames into the stage, so the track at the stage's start can be the song the
    /// stage was come into with rather than the stage's own; then every fight of that stage
    /// is a track change, and its midboss comes out as the boss the stage ends with — its
    /// chapters keeping a music they should rewind and the retry menu offering them under
    /// the wrong name. Where the game names its two songs, the answer does not depend on
    /// when a chapter was taken.
    fn the_fights_own_song_is_playing(&self) -> Option<bool> {
        None
    }

    /// **Whether the fight stage `stage` ends with is fought in two halves**, its boss leaving for a stretch
    /// of the stage's waves after the first and coming back for the second — so that her going the first
    /// time is a chapter of its own and not the stage won. See `chapter::Cause::StageAfterTheBossLeft`.
    ///
    /// **A reading of the stage and not of the fight**, because nothing on the frame she leaves says which
    /// of her two goings it is: the stage's own end comes some frames after the second, and on the frame
    /// itself both are a boss that was up and is not. False for every stage of a game where none has been
    /// read.
    fn the_stages_boss_comes_back(&self, stage: i32) -> bool {
        let _ = stage;
        false
    }

    /// The family the game draws its own text in, where it ships no font and asks the host for
    /// one by name — so that orb's own screens are in the same face as the game's text.
    ///
    /// `None` where the game ships a font instead, which orb loads: th06 adds the `font.ttf`
    /// beside its exe. th07 ships none — the sprites in `th07.dat` are its panel's digits — and
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
    /// paints over a neighbour the game repaints no oftener than this row: th07 puts its
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
    /// Nothing for a game whose calls are done by the time they return; th07's are
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
    /// **And memory whose meaning is a thread's and not the run's**, for the same reason and with the
    /// same answer: a lock says which thread holds it and a work queue where a worker has got to, and a
    /// restore rewinds neither the threads nor what they are doing. Put back, they describe a moment the
    /// rest of the process has left — see `Th06c::live_handles`, where that cost a frame 130ms.
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
    /// orb's two lines land on top of the game's. th06 leaves the bottom-left of it empty, which is what this
    /// default is; th07 writes the difficulty's own name and its English line there, and a session of it read
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
    /// middle. th07 empties the queue of quads its drawing fills here and draws it there, and a loop
    /// that put either of those in `prepare_frame` would be emptying a queue the game's own drawing had
    /// already filled or drawing one outside the scene it was queued in.
    ///
    /// Nothing for a game whose frame does nothing there, which is th06: its `GameWindow::Render` goes
    /// `BeginScene` at 0x4207d1, `Chain::RunDrawChain` at 0x4207dc, `EndScene` at 0x4207ef with nothing
    /// between any of them.
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

    /// Whether those records hold a count at all, `records` being the bytes [`Game::captures`] has just
    /// answered — which is what says there is anything in them worth keeping across something that would
    /// replace them.
    ///
    /// **An emptied record is not a zeroed one, which is why this cannot be a scan for zero bytes.** th07's
    /// own emptying zeroes every record and then writes each one's `CATK` header and card number back into it
    /// — `th07/asm.md`, under the score file's own phases — so an array a run's build has just emptied
    /// answers full to anything counting bytes. Only the game whose records they are can tell a header from a
    /// count, so the question is here.
    ///
    /// **The bytes are handed over rather than read again**, because reading is what costs: a walk that asked
    /// the memory map per word made a played-back frame of th07 ask it 76 times where 48 is what the game's own
    /// allocations are behind — `the_run_of_th07_written_down_off_the_frame` is the e2e test that says so.
    ///
    /// `false` where the game keeps no such record, which is the same answer as records holding nothing.
    fn these_records_hold_a_count(&self, records: &[u8]) -> bool {
        let _ = records;
        false
    }

    /// Puts back what [`captures`](Game::captures) took.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `saved` a `captures` of this same build.
    unsafe fn set_captures(&self, saved: &[u8]);

    /// Puts the card records a run left back over the ones a save's own read of the file has just laid down.
    ///
    /// **Back rather than added**, and that is a reading of the game rather than a choice: th07 empties every
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

    /// Whether the game's own **run** build lays its score file's card records over the ones in memory, so
    /// that the run before this one loses what it counted the moment another starts.
    ///
    /// **th07 loses them there too and has a hook for it; th06nc has this instead**, and the difference is
    /// only where the read can be bracketed: th07's is inside the save [`Hooks::card_records_read`] stands
    /// over, and th06nc's is `0x14006e310(the parsed file, manager + 0x968)` at `0x14003c5e7` — on the
    /// run-start branch of the stage build `Hooks::stage_begun` *already* stands over, after the loop at
    /// `0x14003c390` has laid all 134 records out. So answering true asks `runtime::the_stage_begun` to
    /// put the note a run's end left back once the original returns, and to spend it there.
    ///
    /// **A question rather than one more `Hooks` entry**, because there is nothing left to patch: the jump
    /// is written, and `0x14006e310` itself could not take one — twelve bytes of prologue stand before its
    /// `jne` where a 64-bit jump needs fourteen.
    fn its_run_build_lays_its_file_over_the_cards(&self) -> bool {
        false
    }

    /// Whether the game writes its own score file, so that nothing above has to build a ranking screen for
    /// it.
    ///
    /// **th07 does and th06 does not**, and it is a difference in where the write is: th06's runs as its
    /// ranking screen goes down, which is why orb builds one and takes it down; th07's screen writes at no
    /// phase it is built with, and the one write in that exe is a save the game makes for itself at every run's
    /// build and on its way out. Which is enough — every way a session's records are lost goes through one of
    /// those two — so what orb does for it is put the records back inside that save rather than ask for a save
    /// of its own. Asking is what a launch faulted on: the call runs the screen's own teardown, which frees the
    /// anm files in four of the manager's slots, and orb made it with the front end standing on them.
    fn writes_its_own_records(&self) -> bool {
        false
    }

    /// Whether the game reads its score file once, at startup, and never again — so that the one read is the
    /// front end's read of its unlocks and there is no callback for [`Hooks::unlocks_read`] to bracket.
    ///
    /// **`true` keeps the bracket up from the attach until the mode is first settled**: every open of the file
    /// until then lands in the game's own, which is where the unlocks the title lights its items from belong,
    /// and the mode's own records are read over it at that first settling and every one after —
    /// [`Game::read_what_the_modes_file_records`], which for such a game is the whole of the fork's second
    /// half. th06c is the game this is `true` for: its read at `0x14002c239` is inline in its startup and not a
    /// function a jump could be written over.
    ///
    /// `false` for a game that opens the file at every screen that wants it, whose bracket is the callback's.
    fn reads_its_file_once_at_startup(&self) -> bool {
        false
    }

    /// Whether the read of the score file now running is one a write follows, `screen` being the object that
    /// read is made on.
    ///
    /// **Which is what says a session's records belong back in memory at the parse inside it.** Every read of
    /// th07's file lays the file's own card records over the ones in memory, and only some of those reads are
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
    /// mode's. Which left Pointdevice mode showing legacy practice scores on a screen whose runs go in orb's
    /// file. One read cannot land in two files, so the record is read again: the front end's read stands as
    /// it is, and this puts the mode's own numbers over the half of it that is a record rather than an
    /// unlock.
    ///
    /// **Every such record, which for th07 is two**: the practice scores, and the card records its
    /// ranking screen shows and a run counts into. A session that never answered the mode question again showed
    /// the other mode's of both — a run given up back to the practice stage select carried a legacy score
    /// for the stage it had just been played in, and the ranking screen counted nothing of the session.
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

    /// **The virtual pad pushed the way `word` is pressed**, for a launch pressing through a pad
    /// (`Config::through_a_pad`): each bit held on the button this game's own mapping names for it, and each
    /// direction as the stick pushed all the way — the inverse of what the game's read makes of a pad. A bit no
    /// button is mapped to is left unpressed, the way a person's pad cannot press it either.
    ///
    /// `None` for a game whose own read of a pad orb has not read, or whose menus of orb's read no pad: a launch
    /// over it hands its word to the game's read as it would without the pad, and says so.
    ///
    /// # Safety
    /// Must run on the game's main thread, the mapping being read out of the game's memory.
    unsafe fn pushed_for(&self, word: u16) -> Option<crate::joystick::Pushed> {
        let _ = word;
        None
    }

    /// **The DirectInput pad this game holds, and the range it set its axes to** — `None` where its enumeration
    /// found none, or where the game reads no pad through DirectInput: what `joystick::stand_in_front_of` is
    /// handed, so that a virtual pad reaches a game reading the device rather than winmm.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn its_pad_device(&self) -> Option<(usize, i32)> {
        None
    }

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

    /// **Whether a run given up waits for a press before it goes**, which is the game's own way out asking
    /// for one: somebody answering orb's retry menu makes it, and a give-up nobody presses for — a recipe's
    /// `leave`, a watched attempt's own end — is a run that plays on under the hand-over it asked for. orb
    /// presses for it then, in the read it answers; see `runtime`'s `the_press_a_hand_over_waits_for`.
    ///
    /// False, which is every game whose way out is the scene written.
    fn a_run_given_up_waits_for_a_press(&self) -> bool {
        false
    }

    /// **Whether a stage's end stands until a key goes down**, which is the tally a person presses past and
    /// the rules, pressing what a stage wants, do not: orb presses for it then, in the word the rules hand
    /// over — see `runtime`'s `what_is_reached`.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn a_stages_end_waits_for_a_press(&self) -> bool {
        false
    }

    /// **Whether the ending stands on a line until a key goes down**, which orb's skip of an ending then makes
    /// the presses for: the skip is the game's own update played over inside one frame, and every read in it
    /// answers the same word, so a line waiting for a press is a line no number of those updates gets past.
    ///
    /// False, which is every game whose ending runs on by itself.
    fn an_ending_waits_for_a_press(&self) -> bool {
        false
    }

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

    /// **A word's own buttons put into the keyboard snapshot the game is about to read**, for a game
    /// whose read is a snapshot rather than a word — see [`Read::Pointer`] and
    /// [`crate::runtime::keyboard_state`].
    ///
    /// Nothing for a game that hands orb a word and takes one back: th06 and th07 read their own keys
    /// and orb answers that read with the word, so there is no snapshot for anything to be written
    /// into. Which is why this is a method here rather than a table in the runtime — which keys a word
    /// is, and whether the game reads keys at all, is the game's own.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's own read was given.
    unsafe fn press_into(&self, state: *mut u8, word: u16);

    /// **And the word a person's own keys in that snapshot say**, which is [`Game::press_into`] read the
    /// other way.
    ///
    /// **What asks is the holding back of a press**, and it is the one thing that makes a question of orb's
    /// answerable at a keyboard: `runtime::held_back` takes a word and takes the decide out of it, and for a
    /// game whose keys orb *answers* rather than reads there is no word until this makes one — a press then
    /// reaches the game's own item and orb never sees it. See [`crate::runtime::keyboard_state`].
    ///
    /// `None` for a game that hands orb a word and takes one back: th06 and th07 read their own keys and
    /// orb stands in front of that read, so the word reaches the holding there and there is nothing here to
    /// do.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's own read was given.
    unsafe fn word_in(&self, _state: *const u8) -> Option<u16> {
        None
    }

    /// **A word's buttons put into the pad state orb answers a game's own pad read with**, for a game
    /// whose input orb reaches that way — see [`Hooks::reads_a_pad`].
    ///
    /// Nothing by default, which is a game orb does not answer a pad read for: th06 and th07 stand in
    /// front of their own pad read instead and hand it a word, so there is no state to fill.
    ///
    /// # Safety
    /// `state` must be the bytes that game's own read was given.
    unsafe fn press_into_pad(&self, state: *mut u8, word: u16) {
        let _ = (state, word);
    }

    /// Which buttons the game's own front end reads as *decide*, in the word its input read hands
    /// back.
    ///
    /// For the question orb asks over the screen a run is started from: a press taken out of that
    /// word is a press no screen in the frame ever saw, so the screen is still standing when the
    /// question comes down and answering "neither" is nothing at all rather than a scene put back by
    /// hand. Handed over afterwards the same way, by putting the bits back in for one read, which
    /// leaves starting the run to the screen whose business it is.
    ///
    /// Which bits those are is the game's own: th06's shot type select tests
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

    /// **Takes the game off the screen a finished run ends at, whatever part of it is up**, and says whether
    /// it did.
    ///
    /// For the run an attempt out of a replay was watched in, where that screen was reached because the
    /// attempt is the path the boss died on: the name entry is where somebody types their own name, and the
    /// stats after it are theirs to read. This run is nobody's — [`Game::skip_replay_prompt`] writes past one
    /// screen of it because a pointdevice run has no replay to offer, and this writes past all of it because
    /// there is no run.
    ///
    /// **What it does not stop is the file being written on the way out.** That is the screen's own
    /// `DeletedCallback` and happens whichever way it goes down, so what keeps a watched run out of the
    /// ranking is `score::watching` and not this.
    ///
    /// Asked only while an attempt's run is being taken down, so it may cost a read.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn leave_the_screen_a_run_ends_at(&self) -> bool;

    /// **Whether the game is on that screen at all**, which is what says the write above has been answered.
    ///
    /// Beside it because it is the other half of taking a watched run out of those screens: the write asks
    /// the screen to leave and the screen plays its own way out over the frames that takes — twenty of them
    /// on th06, every one of which drew a name entry belonging to nobody. So the run-out is a loop over the
    /// game's own update, the way an ending is skipped, and this is its condition.
    ///
    /// Asked only on the frames such a run-out is running, so it may cost a read.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn the_screen_a_run_ends_at_is_up(&self) -> bool;

    /// **Whether the front end is on the screen the game keeps its own replays on**, up and drawing.
    ///
    /// Which is where a list of orb's own replays belongs: whoever answered 完全無欠モード did it at that
    /// item, so the screen behind the list is the one the item leads to. The title menu stood there instead
    /// and the list that came back after an attempt stood over the logo, both of which read as having left
    /// the replays rather than being in them.
    ///
    /// The default is any screen the front end has up, which is what a game orb has not read this screen of
    /// answers: what a list needs behind it is a screen the game has drawn, and which one is a nicety.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn the_screen_the_replays_are_on(&self) -> bool {
        unsafe { self.front_end_screen() }.is_some()
    }

    /// **Asks the front end for that screen**, and says whether it asked.
    ///
    /// Asked on every frame a list of orb's is waiting for it, so an answer of `false` is what says the
    /// screen is already on its way: asked twice, a screen restarts whatever transition it is in the middle
    /// of, and one asked every frame never arrives.
    ///
    /// The default asks for nothing, and then the list goes over whatever the front end has up.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn to_the_screen_the_replays_are_on(&self) -> bool {
        false
    }

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
    /// **th07's frame of input lag is this and nothing else.** Its jobs run in the order their chain
    /// priorities give: the read at 0, the player at 8 — which reads the word *the run acts on* and moves on
    /// it — and the copy from the one word into the other at 0x10, inside the recorder. So the player moves on
    /// what the copy left a frame ago. Called from the input hook, which is the read at priority 0, this makes
    /// that copy early and the frame's own keys are what its update acts on.
    ///
    /// Nothing where a game hands the read to its own update in time, which is th06: there is one word and
    /// the player reads it. And nothing while a replay is being watched, where what the run acts on is the
    /// recording rather than the keyboard.
    ///
    /// **`moved_on` is whether this read is the first of a frame of the run**, and what it decides is the
    /// *pair*: a run reads a bomb as an edge, so the word it last acted on has to be the frame before's rather
    /// than the read before's. th07 reads the keyboard again for a frame it did not advance the run on — its
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

    /// **Makes the game forget when it last read its keyboard and its pad**, so that the next read it
    /// makes is a read and not the answer it kept.
    ///
    /// **What this is for is a playback, where a game's own rate is the wrong one.** th06nc reads at most
    /// once a millisecond — its read holds a stored `timeGetTime` against the clock and skips itself where
    /// the two are equal, and the pad's poll does the same — which for a run somebody plays is one read a
    /// frame and exactly right. A playback drives its updates in a burst inside one frame of the host, so
    /// all but the first of them fall in the same millisecond and compose their word out of the snapshot
    /// the first one left: the word a frame of the run acted on is then the word of whichever update
    /// happened to open a millisecond. Measured over a pair of real launches of one chapter — the same
    /// `chose=` on every frame and the *composed* word apart on seven of 3,474, the first of them a frame
    /// orb fed nothing for whose word held the shot.
    ///
    /// Nothing for a game that reads when it is asked, which is th06 and th07: their reads are orb's hook
    /// and there is nothing kept between two of them.
    ///
    /// # Safety
    /// Must run on the game's main thread, before the update whose word is about to be composed.
    unsafe fn forgets_when_it_read(&self) {}

    /// **Tells the game the line of dialogue up is over**, which is what a playback needs and nothing else
    /// does.
    ///
    /// **What it is for, measured.** A conversation inside a stage stops th08's run entirely: a playback
    /// held at one ran 76,000 updates with `script=4962 seed=0x12c5` — the stage's own script clock and the
    /// generator both standing — so no press it fed could reach anything, the run's own update being the
    /// thing that is not running. What advances a line is outside the calc chain a playback drives, and the
    /// line's own field is what says it is done.
    ///
    /// Nothing from a game whose playback has never been held at one, which is every other: th06's and
    /// th07's conversations are frames of the run like any other.
    ///
    /// # Safety
    /// Must run on the game's main thread, and only from a playback — a run somebody is playing has its own
    /// frames to carry a line with.
    unsafe fn give_up_on_a_line_of_dialogue(&self) {}

    /// **Starts whatever a stage's own first frame started in the game's player** — so that a resumed
    /// stage carries on from where the run's did rather than from where its build left it.
    ///
    /// **What this is for is the one frame a playback never feeds, and the one frame the file holds no
    /// word for.** A run's stage begins on its first frame and the word that frame was played with reaches
    /// the player; a resumed run spends that same frame building the stage and landing in it, and orb
    /// feeds it nought. Feeding the file's own word instead is not open: `Game::stage_frame` answers
    /// `None` at that read, the player's job not being in the chain until the build registers it, so
    /// [`resume::Record::note`](crate::resume) writes nothing down for it — see
    /// [`crate::runtime::the_frames_word`], whose comment carries the launches that measured both.
    ///
    /// **Measured on th06nc, where what it started is the shot's own phase.** A pair of the same chapter of
    /// stage 3 — recorded and picked up minutes apart — agreed on the word, the player, the generator, the
    /// script's clock, the items, the score and the enemies, and parted on **frame 2** in that phase
    /// alone: `shot=2,1` against `shot=1,0`. Of the 3,410 frames the two had in common it parted on
    /// **3,405** — a drift and not a transient — and 216 frames later the enemies' life, the count of
    /// draws and the generator all parted together on frame 218, which is the run's shot landing where the
    /// playback's had not yet. With the phase put back, the pair's first parting moves to the shots
    /// themselves and 2,194 of 3,474 frames part rather than 3,405 of 3,410.
    ///
    /// **`word` is the first the playback is going to feed and not that frame's own**, which the file does
    /// not hold — see
    /// [`resume::the_first_word_a_playback_will_feed`](crate::resume::the_first_word_a_playback_will_feed).
    /// What it is for is deciding whether the run was pressing at all across the stage's start: the rules
    /// hold the shot on fifty-nine frames in sixty (`orb_bot`'s `SHOT_LET_GO_EVERY`), so the frame after
    /// answers for the frame before, and a run that was not firing there gets no cycle it never had. The
    /// laid-out games measure that half: their own first word is the recipe's decide, `0x1000`, and a real
    /// launch's is `0x0011`.
    ///
    /// Nothing for a game whose player orb has read no such state of, which is the other three.
    ///
    /// # Safety
    /// Must run on the game's main thread, inside the frame that built the stage, after the build has
    /// registered the player and before the playback's own updates.
    unsafe fn starts_the_shot_a_stage_began_with(&self, word: u16) {
        let _ = word;
    }

    /// Which bits of a frame's input decide how a run goes, and so are the ones worth writing
    /// down and handing back.
    ///
    /// th06's own replay records exactly these, which is what says the rest do not change a run.
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

    /// What this game calls itself in a file of orb's that is not one of the per-game ones.
    ///
    /// One file is like that, and it is the replay of a run: its shape is orb's — see [`crate::replay`] for
    /// why, the short of it being that the shape is new and has no installed base to be frozen by — and what
    /// each game contributes to it is the opaque bytes of [`Game::run_state`]. So the file carries this, and
    /// one carried into another game's installation is refused rather than read as that game's numbers.
    ///
    /// **Not the exe's name**, which is what [`Known::exe`] is: that is the file a launch is recognised by
    /// and it is the game's, where this is the word orb writes in its own files and its own log. `th06` and
    /// `th07`, which is what everything in this tree calls the two.
    fn named(&self) -> &'static str;

    /// **The file a run left unfinished is written in, which is this game's own.**
    ///
    /// What [`crate::resume`] does with a kept run — when to write it, where the file goes, playing the
    /// buttons back into a stage, holding the landing against what was written down — is the same work for
    /// either game. What a *file* holds is not: th06 keeps a rank, a count of power items and a count of the
    /// extra lives its score has paid for, where th07 keeps a cherry it pays its own with, and each game
    /// wrote nothing into the other's fields for as long as the two shared one shape.
    ///
    /// **And one version number stood in front of both**, which is what settled it: the generator of every
    /// frame — the one thing that can say where a playback stopped reproducing a run — went into the file
    /// without that number being raised, because raising it would have refused every th06 run already on a
    /// disk. A th07 run picked up out of a file from before it parted from itself inside its first thousand
    /// frames and nothing in the log could say where. So the file is the game's, and each game's version is
    /// its own: `th06::resume` is frozen at what shipped, `th07::resume` starts again at 1.
    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8>;

    /// And back, or `None` for bytes that are not a file this game wrote.
    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept>;

    /// **The run's numbers as this stage began, in the game's own bytes.**
    ///
    /// Opaque here, and the whole point of that: what a run *is* differs between the games and the
    /// machinery does not need to know. th06 pays for extra lives out of the score and keeps a rank its
    /// enemies read; th07 pays for them out of a cherry it draws from its own generator as the run is made,
    /// and keeps a baseline every reader of that cherry measures from. They were one struct with both games'
    /// fields in it, each writing nothing into the other's — and with the shape shared, a number th07 needed
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
    /// run had its own. The alternative built and taken back was a file holding a stage's *start* in place of
    /// the chapters inside it: the start replays exactly, and a session interrupted at a boss comes back at the
    /// stage's first frame. A replay that does not reproduce is a field not yet found, and narrowing what the
    /// file holds is no fix for it.
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
    /// about a run of th06 has a rank in it and one about a run of th07 has a cherry.
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

    /// **Starts a run of these numbers from the front end, which is the one run orb chooses itself.**
    ///
    /// Every other run orb puts a chapter back into was chosen on the game's own screens: the difficulty, the
    /// character and the shot are what somebody pressed, and [`Game::start_stage`] writes the one thing left.
    /// An attempt out of a replay is chosen out of a *file* — orb's own list, over a title menu whose press
    /// was never handed over — so the three numbers are this file's and the scene has to be asked for the way
    /// the item that starts a run asks for it.
    ///
    /// `run`'s stage is the stage to start at, written as the menu counts it, so nothing further is needed
    /// before the build: what makes that necessary is when this is called. The item on the game's own screen
    /// writes the scene from a job that runs *after* the supervisor's own copy, which leaves the frame
    /// ending with the two disagreeing — the one frame [`Game::run_chosen`] is true of, and where a chapter
    /// goes in. Called from a menu of orb's the chain is not running at all, so the next frame's supervisor
    /// builds the run before anything of orb's looks again, and that frame is not there.
    ///
    /// False where the game will not take it, which is a run of numbers no game of this one's has: nothing
    /// is written then, a half-written run being worse than none.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the game's own front end up and settled.
    unsafe fn start_a_run(&self, run: &RunStart) -> bool;

    /// **The same run asked for through the front end's own item instead**, where this game's front end can
    /// be put on the screen that item belongs to: the run's numbers go in, the screen goes up past the
    /// frames it ignores a press for, and the press the caller hands over next is the one that item reads.
    ///
    /// **Which is not the same run as [`Game::start_a_run`] builds.** That one writes the scene itself, and a
    /// run written into being is missing whatever else the item does: on th06, `EnemyManager::OnTick` read a
    /// sprite pointer of nothing off the first enemy the stage spawned and took the process with it 129
    /// frames in — where a run the item started spawned that enemy with its sprite and played on. What the
    /// item is handed here it does all of, which is the whole reason to go this way round.
    ///
    /// False where this game has no such screen to put up, and then `start_a_run` is what is left.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the game's own front end up and settled.
    unsafe fn front_end_starts_a_run(&self, _run: &RunStart) -> bool {
        false
    }

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
    /// held inside, the extra lives the score has paid for, the rank and the subrank are th06's, and
    /// th07 answered zero for all four and printed them on every line. The union of every Touhou game is
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
    /// **Which a parting asked for.** A pair of launches of th07 agreed on every field of a reproduction line
    /// for 992 frames of stage 2 and parted at 993 on a point item: the run collected one where the playback passed
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

    /// **The run's world as this game's own fields hold it**, for the trace of a launch that asked — see
    /// [`crate::trace::TheWorld`] — and `None` from a game nobody has read it off.
    ///
    /// **Beside [`Game::the_worlds_fingerprint`] and not instead of it**: the fold is a line any launch's log can
    /// carry and says *that* two passes parted; this is every field, which only a trace asked for can hold, and
    /// says *what* parted.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_world(&self) -> Option<crate::trace::TheWorld> {
        None
    }

    /// **What the game's own list of jobs holds**, said where a frame leaves on its walk answering an exit,
    /// and `None` from a game whose list nobody has read.
    ///
    /// **Because the walk's own answer cannot tell an empty list from a job that asked to stop.** th08's
    /// walk answers the count of jobs it ran, and its switch leaves that walk with nought both for a list
    /// nothing was reached in and for one job answering *stop this frame* — so a launch that ends on
    /// `the game's own walk answered 0` has three readings and no way to choose between them. The list
    /// itself has: a frame with jobs on it and a frame with none are not the same frame.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn the_jobs_a_frame_has(&self) -> Option<String> {
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
    /// to game: `decide` is a shot and a return key in th07 and something else in the next game. So the names
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
    /// lists are cleared and filled, and a stage of th07 fills the first of them six hundred times.
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

    /// **Whether a stage's numbers have come into place since the frame before, off the main thread** —
    /// for a game whose build is a loader thread the main thread sits out, where no function of the
    /// game's returns on the main thread with the numbers in place and the stage not yet updated. The
    /// frame this first answers `true` on is what [`Hooks::stage_begun`] is for a game with such a
    /// function, and the runtime does the same two things on it: the run written down or put back, and
    /// the replay's own record begun.
    ///
    /// `false` from a game with the patch, which says the same thing on the thread the build runs on, and
    /// from a game nothing of this has been read of.
    ///
    /// # Safety
    /// Must run on the game's main thread, once a frame.
    unsafe fn a_stage_has_just_been_built(&self) -> bool {
        false
    }

    /// **An update of a frame the stage is still being built on, run in place of the walk**, for a game whose
    /// loader thread puts a stage's jobs on the chain part way through and goes on building after: the walk on
    /// such a frame runs that stage against a build still going on, with nothing of orb's put back yet. **So
    /// the runtime walks only the job the build leans on**, which is what this runs, and hands on what it
    /// answers as the walk would. `None` where the frame is no such frame, which the runtime walks as ever.
    ///
    /// Asked off the same look as the built frame is — and on an update after the one a build was asked on,
    /// off the build being under way now — so that the frame orb reads the build as over is the first frame
    /// anything of the stage runs on, whatever moment the thread finishes at.
    ///
    /// `None` from a game whose build is the function [`Hooks::stage_begun`] stands over, whose numbers go
    /// in at its return and before any update of the stage.
    ///
    /// # Safety
    /// Must run on the game's main thread, after [`Game::a_stage_has_just_been_built`] this frame, where the
    /// walk would run.
    unsafe fn walks_a_frame_of_its_build(&self) -> Option<i32> {
        None
    }

    /// Midstage chapter boundaries per stage.
    fn midstage_table(&self) -> &'static [&'static [Boundary]];

    /// **What the rows of that table are called where `stage N` is not what they are**, in the order the table
    /// holds them: the last rows, and as many of them as the game has.
    ///
    /// The game's because the games differ — th06's table ends at its Extra and th07's carries the Phantasm
    /// after it — and a tuning pass that read the shape off the table instead wrote th07's Extra down as
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
    /// What one frame of holding a direction moves the player by with the focus key *up*, across and down,
    /// straight and then diagonally. Both pairs, because nothing says the two axes are scaled alike.
    pub straight: (f32, f32),
    pub diagonal: (f32, f32),
    /// **And the same two for a frame with the focus key held**, which is a different pair in every game that
    /// has one — th06c's are `4.00` and `2.83` against `2.00` and `1.41`.
    ///
    /// **Both, because which of them a frame takes is the bot's own answer and not the game's.** The key is
    /// pressed by the rules after the move is chosen, so a projection made with the pair the *last* frame
    /// held is a projection of the other state's step on every frame that answer changes — half what the
    /// move covers or twice it. So what the game hands over is both pairs and which to project with is
    /// `steer`'s, whose own focus rule is what decides it.
    ///
    /// **The same pair as [`straight`](Frame::straight) where a game keeps no second one**, which is th07:
    /// its speeds are one table the game itself writes the fraction into, and what selects that fraction is
    /// not established — see `game::th07::PLAYER_SCALE_ACROSS`.
    pub focused_straight: (f32, f32),
    pub focused_diagonal: (f32, f32),
    /// Whether the game says the player has just been hit and is inside whatever window it gives to take that
    /// back — th07's deathbomb window and whatever the next game calls its own.
    pub hit: bool,
    /// Whether the player is in its ordinary state: neither hit nor in the middle of dying or coming back.
    ///
    /// Which is not `!hit`, there being more than two states — and the difference is what tells a border
    /// deflected from a border tidied away by a respawn.
    pub normal: bool,
    /// Whether the player is carrying something that takes a hit for it, for a game that has such a thing: its
    /// going away while the player is otherwise well is a hit that the count of deaths never sees.
    pub border: bool,
    /// **Whether the shot is at the power the game stops adding to**, which is what makes a power item worth
    /// crossing a field for or worth nothing at all.
    ///
    /// **The game's own, because the number it is compared with is.** 128 is where both games stop, and each
    /// keeps the power in its own type — an integer in th06 and a float in th07 — so what a full shot *is*
    /// is a question about the game and what to do about it is the bot's.
    ///
    /// **And it is th06 the answer matters for.** th07 turns a power item collected at full power into
    /// something else, so a run that goes for one there gets what it went for; th06 leaves it a power item
    /// worth a scrap of score, and a run that crosses the field for one has crossed it for nothing.
    pub full_power: bool,
    /// **How many bombs the run has in hand**, which is what says whether the bomb the rules reach for is
    /// one there is.
    ///
    /// **Because a bomb pressed with none in hand is not a bomb that did nothing**: the rules hold their own
    /// bomb off for `steer::BOMBS_APART` frames afterwards, so a press with the count at nought spends the
    /// wait and leaves the next real corner unbombed. One Extra launch wrote 24 `a bomb spent` lines against
    /// a run that could have had nine — three at the start and what two deaths refilled — and its own two
    /// hits came at corners inside that wait.
    pub bombs: i8,
    /// Whether a line of dialogue is up, which is a thing to press past.
    pub dialogue: bool,
    /// **Whether a bullet out of its hitting state is nothing to keep away from at all**, which is a game's
    /// own answer and not a rule: what it says is that such a bullet cannot take a life *and* that standing
    /// where one comes out is not how a run dies either.
    ///
    /// `true` from th08, measured off its own walk: the one box test that takes a life has a single caller
    /// and it is the hitting kind's, the kinds a bullet comes out in asking only whether it is inside one of
    /// the circles of the player's own that erase a bullet — and a bullet erased as it comes out is left in
    /// the kind whose branch asks nothing, harmless for the rest of its life. See `Th08::BULLET_HITS`, and
    /// what it cost to have this wrong: the rules read a mesh
    /// of 747 spawning bullets as solid, had `nowhere open to it` in every box, and died in the corner they
    /// had backed into — where a person plays that card by standing in the mesh.
    ///
    /// `true` from th10 as well, whose coming-out branch tests nothing (see `Th10::BULLET_STATE`) — until the
    /// age in [`Frame::a_bullet_comes_out_for`], which th08 has no counterpart of.
    ///
    /// `false` from every other game, where a bullet out of that state is left as dangerous as any other: what
    /// th06 calls spent and th07 calls not yet hitting are their own readings, and whether either game hands a
    /// run the same mercy is unmeasured.
    pub a_bullet_not_hitting_is_harmless: bool,
    /// **The age a bullet coming out turns solid at**, where [`Frame::a_bullet_not_hitting_is_harmless`] is
    /// `true` and the game turns every bullet the same frame of its life: the frames such a bullet is ground
    /// to cross are the ones before that age, and from it on it is a hazard like any other — the wait inside
    /// it included.
    ///
    /// `None` where it stays harmless for good, which is th08's: a bullet erased as it comes out is never made
    /// solid at all. `Some(9)` from th10, see `Th10::A_BULLET_COMES_OUT_FOR`.
    pub a_bullet_comes_out_for: Option<i32>,
    /// **Whether a boss is to be shot unfocused**, which is a game's own answer about what its shot earns
    /// and not about how it dodges: the rules hold the focus key on a calm frame of a fight because a focused
    /// shot concentrates the damage, and on a game where the shot's mode is also what a hit is *worth*, that
    /// is the wrong trade.
    ///
    /// `true` from th08, whose human–youkai gauge at `[run+0x22]` is moved every frame one way or the other by
    /// the byte the player's own update reads for the shot's mode (0x44beb6), and whose time orbs come out of
    /// the gauge's two ends: an enemy killed there drops one (0x42da41, behind 0x406d10 and 0x406d70), and
    /// the strategy wiki's own list has the one that pays under a boss — `人間逢魔が時にショットを敵に当てる`, a
    /// shot hitting an enemy with the gauge at its human end, which `高速状態なら人へ、低速状態なら妖へ傾く`
    /// says is the unfocused one's. A run that holds the key on calm frames and lets it go to dodge keeps the
    /// gauge in the middle, where neither pays — and the time orbs are the whole of what a stage's Last Spell
    /// is decided by. See `Th08`'s `TIME_ORBS` and `GAUGE`.
    ///
    /// **And only until the stage's norm is met**, which th08 answers every frame off the count and the norm
    /// its panel draws: past it the orbs have bought the Last Spell and a fight wants the focused shot's
    /// damage back, and a miss that takes the count back under it is the unfocused shot again.
    ///
    /// `false` from every other game, whose shot's mode changes its damage and nothing else that is read.
    pub shoots_a_boss_unfocused: bool,
    /// **Whether the gauge a hit is worth a time orb by is at its human end**, for a game that has one — which
    /// is where `shoots_a_boss_unfocused` means it to be.
    ///
    /// **And what says a card's fire holds it short of there, which the unfocused shot alone does not.** th08
    /// moves its gauge `100` towards the youkai end for every graze (0x44aa81) against the unfocused player's
    /// own lean the other way every frame (0x44bedf), so a card grazed thickly enough holds it in the middle,
    /// where neither end pays. Six runs of stage 2 played from its start said so outright: three were at the
    /// human end on `34` to `51%` of card 22 and took `441` to `611` time orbs through it, and three never
    /// reached it, took `27` to `38`, and each ended short of the norm — the other three meeting it. The rules
    /// play the rest of such a card focused, which the grazes and the player then lean the same way — see
    /// `orb_bot::steer`'s `HELD_OFF_THE_HUMAN_END_FOR`.
    ///
    /// `None` from every other game.
    pub at_the_gauges_human_end: Option<bool>,
    /// **Whether the familiars out are worth more left alive than shot at**, which has the rules hold their fire
    /// while it is so — the shot being the one thing of the player's that kills them.
    ///
    /// `true` from th08 through the last 180 frames of a card, with a familiar alive and the stage's norm not
    /// met, because its familiars pay for being alive when the boss they belong to ends her attack: 0x42adb0
    /// pays twice `[body+0x3380]` in time orbs — a count of her familiars alive, stepped up as each is linked
    /// to her (0x41b0de) and down as each dies (0x42d63a) — and then each familiar still on her chain up to 26
    /// more, `2N + 10` for a chain of `N`; and pays nothing at all where the chain is empty. One launch's boss
    /// ended her second non-spell with 37 counted and none on the chain and was paid nothing, and her card 26
    /// with 157 and 15 and was paid 704. **The last 180 frames and not the whole attack**, because a hit on
    /// anything at the gauge's human end pays too, and the unfocused shot's hits are most of what a stage's
    /// count is made of: held focused through every attack with a familiar out, the three runs a stage 2 was
    /// played over from its start ended card 26 on `4680` to `5366` where they had ended it on `6086` to
    /// `6738`. And 180 because that is where the card's own payout reaches its floor of 100 — see `Th08`'s
    /// `CARD_TIME_ORBS` — so a card taken there by shooting pays no more than one run out.
    ///
    /// `false` from every other game.
    pub familiars_are_worth_sparing: bool,
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
    /// **How many frames the game says the thing in this slot has been there**, which is what says it is that
    /// same thing and not the next one the slot was given — the reason `Beam::frames` gives, for a hazard.
    ///
    /// **Because a slot is an identity only while nothing has been taken out of it.** A game that frees a slot
    /// and fills it the frame after holds something whose place has nothing to do with the last one's, and a
    /// body credited with the distance between the two is feared over everything it could have reached from
    /// there: th08's `蘇活「生命遊戯」` stands a grid of cells up and kills and bears them every few frames, and
    /// five hits of one Normal run read `nearest was an enemy ... half of it 2408.0,4164.9` on a field 384 by
    /// 448, every one of the nine moves `hit first on 0` with `-2367.7 of room`. Those halves are the read
    /// `8.0` grown by `(12.0, 20.8)` a frame for forty frames, and `(12.0, 20.8)` is `24 * (cos 60°, sin 60°)`
    /// — one step of that grid's own lattice, which is the distance between two of its cells and not the
    /// motion of either.
    ///
    /// `None` where nothing on the thing is such a count, which leaves the slot to say it: every game but
    /// th08, whose enemy carries a timer of its own — see `Th08::ENEMY_FRAMES`.
    pub frames: Option<i32>,
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
    ///
    /// **And the game's own number for that state**, which is for the log and nothing else: what a bullet's own
    /// step is scaled by is its state's, so a bullet that moved by something other than what its step said is
    /// one whose state is the question — and th06c has a bullet that moved `-2.25,5.13` against a step of
    /// `-1.60,3.66`, the same `1.4` on both axes, which no scale it reads off that game's five branches is.
    ///
    /// **And which of the changes its script laid on it the bullet is at**, where a game keeps such a list on
    /// the bullet, and `None` where it keeps none — every game but th10, whose bullet carries up to eighteen
    /// (see `th10::BULLET_NEXT_CHANGE`). A card whose fire is aimed twice is told apart by it: one way of
    /// dodging turns back only once every bullet has been aimed at the run the second time.
    Bullet {
        hitting: bool,
        state: u16,
        next_change: Option<u32>,
    },
    /// A body, and whether the game will run its collision at all — an enemy with nothing solid on it cannot
    /// touch the player, however wide it is.
    Body { solid: bool },
    /// A place a body has already been, which is what a beam that follows one is made of. It does not move and
    /// nothing will move it: only its ends come and go.
    Trail,
}

/// **What [`Beam::fires_in`] is for a beam a game has aimed and says nothing more about**, which is every
/// frame the rules will ever look at.
///
/// **Because the two things a countdown says come apart in th08.** That game's beam is a script's, and the
/// field that turns it live turns on the frame the box first tests the player — so what says one is coming
/// is the anchor its script wrote hundreds of frames earlier, and that says *where* and never *when*. A
/// lane read live for as long as it is armed is four lanes at once in stage 4's fight, which is a field
/// with nothing open in it; one read as a warning is ground to be off before it fires and ground that may
/// be crossed to get out of something that already is.
pub const FIRES_WHEN_NOTHING_SAYS: i32 = i32::MAX;

/// **A beam**, which is a line rather than a box and so is not a [`Hazard`] at all.
#[derive(Clone, Copy)]
pub struct Beam {
    pub slot: usize,
    /// Where it is fired from, and which way it points now. Where it will be fired from and what it will be
    /// pointing at in a frame's time are things to work out from how it has been moving and turning, there
    /// being no rate on either to read.
    pub origin: (f32, f32),
    pub angle: f32,
    /// **How many frames it has been in the phase it is in**, which is what says the beam in this slot next
    /// frame is the same beam: a slot the game gives to the next beam of a sweep holds one whose angle, ends
    /// and width all match the last one's, so nothing else about a beam tells the two apart — and a beam
    /// credited with the distance between it and its predecessor is a lane feared the width of the field away
    /// from where it is.
    pub frames: i32,
    /// How far along it the beam runs, from the near end to the far one.
    pub near: f32,
    pub far: f32,
    /// **And how much further the far end reaches each frame**, which is nought for a beam whose length is
    /// settled.
    ///
    /// **Because a beam that grows is a beam a fixed length cannot be dodged by.** th06's stage 2 boss fires
    /// icicles straight down at the player and lets them extend: a launch stood five units under a far end,
    /// where every one of the frames looked at read as clear, and died there nineteen times.
    pub grows: f32,
    /// Half of how wide it is now.
    pub half_across: f32,
    /// How many frames until the game starts testing the player against it, which is nought or less for one
    /// already firing. Before that it is drawn and hits nothing.
    ///
    /// **And [`FIRES_WHEN_NOTHING_SAYS`] where the game has one aimed and no field of its says which frame
    /// it goes live on**, which is not the same as a beam far off: the rules may cross it, and they get no
    /// frame to leave it once it fires.
    pub fires_in: i32,
    /// Whether it is on its way out. It goes on hitting while it fades, and what the fading takes down is a
    /// game's own business — th06c's takes the length and not the width, which is why the ends handed over are
    /// the seam's to work out rather than the fields as they are read. So this says which beams a lane read
    /// wrong would be, and nothing else turns on it.
    pub fading: bool,
}

/// **Something to pick up**, and which of the things a game drops it is.
pub struct Pickup {
    pub worth: Worth,
    pub at: (f32, f32),
}

/// **The kinds of thing Touhou drops**, which is a vocabulary the games share where their own numbering for it is
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
    /// **Whether the game's own name entry is up**, which is what a step that types one has to be running
    /// while — see [`Game::typing_a_name`], and `recipe::Until::Name`.
    pub typing_a_name: bool,
    /// **Which card is running, by its number**, and nothing between cards — see `recipe::Until::Spell`.
    pub spell: Option<u32>,
    /// **How many frames that card has run**, off the boss's own attack clock — `State::boss_attack_frames` —
    /// and nothing between cards or on a game that keeps no such clock. What a way of dodging one card by its
    /// own schedule reads.
    pub card_frames: Option<i32>,
    /// **The stage's own frame**, `State::stage_frames`, and nothing off a run — which the count the shot's
    /// let-go is timed by starts again from wherever it starts or goes back, so that a run picked up at a
    /// chapter lets go on the frames the run it was written by did.
    pub stage_frames: Option<u32>,
    /// **The run's difficulty, as the game counts it**, `State::difficulty`, and nothing off a run — which the
    /// autoplay engine's ways are written for, a chapter of one difficulty's stage being no chapter of another's.
    pub difficulty: Option<i32>,
    /// **Where the cursor of the question of orb's up is**, and nothing where none is up — what a recipe's
    /// `answer` presses its way to where the launch presses through a pad, as somebody looking at the question
    /// would.
    pub on_the_question: Option<OnTheQuestion>,
}

/// **Which of a question's lists is showing, and the item its cursor is on** — counted from nought as a recipe's
/// `answer` counts an item. The list is a number of the question's own: what it says is only whether it is the
/// list the cursor was on a frame before, which is what an answer pressed through a pad is over by once its
/// decide has taken an item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OnTheQuestion {
    pub list: u8,
    pub at: usize,
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
    /// `keep_replay` — whether to keep the replay of a run that is finished with. Unlike the retry, a step
    /// has to say when: the two moments it comes up are ones a recipe walks to on purpose, and what it
    /// answers decides whether a file is there afterwards.
    KeepReplay,
    /// `choose_replay` — the lists a replay is watched out of: the replays, then the stages, chapters and
    /// attempts of the one chosen. One question rather than four, the four being one screen at a time behind
    /// one item on the title menu — what a step waits for is the lists being up, and a tapped decide walks
    /// them.
    ChooseReplay,
}

impl Question {
    /// The name a recipe writes this question as, which is also the name a trace records it under — one
    /// table, so that a check over a trace and a step in a recipe are read against the same words.
    pub fn name(self) -> &'static str {
        match self {
            Question::Mode => "mode",
            Question::Resume => "resume",
            Question::Retry => "retry",
            Question::KeepReplay => "keep_replay",
            Question::ChooseReplay => "choose_replay",
        }
    }
}

/// The game's state for one frame, read all at once so that everything in it
/// describes the same instant.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
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
    /// **The enemies there are to shoot at**, which is what the tuning pass's gap between waves is: a game
    /// whose stage script keeps an enemy alive that nothing can shoot counts it out, th08 being one.
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

// Only on the target th06 and th07 are built for — see `th06c::tests`, which is what the other target's
// run of these is for.
#[cfg(all(test, target_arch = "x86"))]
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
