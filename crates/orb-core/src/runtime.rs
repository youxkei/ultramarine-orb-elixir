//! What happens to a run: the hooks the game reaches orb through, and the state they carry between
//! frames.
//!
//! **Every one of these is a hook body and none of them is a hook.** What gets in front of the game's
//! own code needs a process to patch and is `orb`'s — `DllMain`, the trampolines, the import table, the
//! install lists. What a hook then *does* is here: it reads a `State`, asks `chapter` or `resume` for a
//! decision, and calls through a function pointer out of a static. None of that is Windows, and it is
//! precisely what an e2e test drives. See
//! [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
//!
//! **The install lists fill the statics below**, which is why the ones a hook calls through are `pub`:
//! `orb::attach` stores a trampoline in each as it patches, and [`attach_to`] stores a laid-out game's
//! own functions there instead — see [`Originals`], which is that list as one value.
//!
//! **Nothing here is handed the other way.** Every hook body is this crate's, and what each of them needs
//! of the process it is in — the game's memory read and written, a vtable slot swapped, the black beside
//! the game painted — goes through `orb_api`. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use std::ffi::c_void;
use std::ops::Range;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU16, AtomicU32, AtomicUsize, Ordering};

use orb_config::{Config, Language};

use crate::find;
use crate::game::{CountedDeath, Game, Menu, Pad, Question, Reached, RunStart, State};
use crate::input::{Keyboard, Pads};
use crate::lives_ui::LivesMark;
use crate::menu_ui::{By, Keys};
use crate::mode_ui::{Answer, Mode, ModeMenu};
use crate::mouse::Mouse;
use crate::overlay::{FONT_HEIGHT, MARK_FONT_HEIGHT, Overlay};
use crate::retry_ui::{self, Choice, RetryMenu};
use crate::sync::MainThread;
use orb_api::d3d8;

use crate::chapter::{Cause, Chapters, Judgement};
use crate::{
    detail, frame, game, joystick, log, pacing, profile, replay, replay_ui, resume, resume_ui,
    score, summary, trace, writer,
};

/// The frame loop's state, made on the first ask.
///
/// Self-initialising rather than set up in `attach`, so that there is no order in which a caller
/// can reach it before it exists — it reads the counter's frequency when it is made, and a pacing
/// that does not know that yet can answer nothing.
///
/// # Safety
/// Must run on the game's main thread, and the caller must not already hold a reference this
/// returned.
#[allow(clippy::mut_from_ref)]
pub unsafe fn pacing() -> &'static mut frame::Pacing {
    unsafe { PACING.get() }.get_or_insert_with(frame::Pacing::new)
}

/// How often the state line goes to the log while a run is in progress.
const STATE_LOG_INTERVAL: u32 = 60;
/// How often the numbers on the HUD are refreshed. Often enough to watch, seldom enough
/// to read.
const HUD_NUMBER_INTERVAL: u32 = 30;

/// How many frames to keep trying to build the overlay for.
///
/// More than one because what one failure costs is the whole run: no retry menu and none of the
/// questions orb asks, each of them being skipped where there is no overlay. Not forever, because
/// the failure to expect is `font.ttf` missing, and that answer does not change however often it
/// is asked — so this is a handful of tries and then the line saying it is unavailable.
const OVERLAY_ATTEMPTS: u32 = 8;

/// `CHAIN_CALLBACK_RESULT_BREAK`: what `RunCalcChain` returns when a job asks it
/// to stop for this frame. Returning it instead of calling the original is how
/// the retry menu freezes the game while drawing carries on.
const CHAIN_BREAK: i32 = 1;
/// `th06::RenderResult`, which the loop the game calls this from expects back.
const RENDER_KEEP_RUNNING: i32 = 0;
const RENDER_EXIT_SUCCESS: i32 = 1;
const RENDER_EXIT_ERROR: i32 = 2;

/// `RunCalcChain`'s answers that mean the game wants to stop running.
///
/// `pub(crate)` because a game whose calc walk answers something else has to turn its own answers into
/// these before the frame loop reads them — see `game::Walks`, and `game::th06c`, whose walk answers a
/// byte saying whether the frame is leaving.
pub(crate) const CHAIN_EXIT_SUCCESS: i32 = 0;
pub(crate) const CHAIN_EXIT_ERROR: i32 = -1;

/// A stop on the ending skip, in case an ending ever waits for something that
/// running the update alone will not deliver.
///
/// Above a whole ending, because the point is that no frame of one is drawn and this is a
/// limit per frame: the loop stops here, the frame it is in goes on to draw whatever the
/// ending is showing by then, and the next frame picks the skip up again. At 7200 — two
/// minutes of game time, which was taken for longer than any ending — stage 6's ending hit
/// it five times and put five frames of itself on the screen.
///
/// Fifteen minutes, against the 36,932 updates that ending took, staff roll included. An
/// update of it costs 13µs — 484ms for all 36,932 — so an ending that never ends is a
/// pause under a second rather than a game that never comes back.
const ENDING_SKIP_LIMIT: u32 = 60 * 60 * 15;

/// How long a list of replays waits for the screen it is to stand over before going up over whatever the
/// front end has instead — see [`ToPutUp::waited`].
///
/// Five seconds against the one that screen takes, which is what a stop on this has to be: a screen asked
/// for that never arrives is a game somewhere orb did not expect, and a list nobody can see is a launch
/// somebody has to close.
const A_SCREEN_IS_WAITED_FOR: u32 = 300;

/// And a stop on the same run-out for the screens a finished run ends at — see
/// [`Game::the_screen_a_run_ends_at_is_up`].
///
/// Ten seconds of them against the twenty frames th06's result screen takes to leave: what it is a stop on
/// is a screen that answers the write by standing there, which is a screen that would otherwise hang the
/// frame it is in rather than one that shows for a moment.
const RESULT_SKIP_LIMIT: u32 = 600;

/// A stop on building the ranking and taking it down, which is what writes what a run counted about
/// spell cards: room for the front end's wait on the item and the screen building itself, in updates
/// run inside one frame with nothing drawn.
///
/// **Inside one frame on purpose, and the frame is long enough to feel.** Spreading those updates over
/// frames is spreading something that would then be *drawn*, which is the ranking screen appearing for
/// a second on the way out of a run — the thing this shape was arrived at to stop. So the cost is a
/// pause where a run is given up, and the alternative is a second of a screen nobody asked for.
const COMMIT_FRAME_LIMIT: u32 = 240;

/// How many times the stress mode restores the same chapter before letting the
/// run carry on. Without a limit it would rewind the first chapter for ever and
/// never reach a boss, which is where the paths worth exercising are.
const STRESS_PER_CHAPTER: u32 = 4;

/// Green rather than white, because the game itself flashes white — a bomb, a boss going
/// down — and a mark that means something orb decided should not look like something the
/// game did. Nothing in th06 fills the play field with green.
const FLASH_COLOR: u32 = 0x0040_ff40;

/// The wash over the play field the moment a boundary is reached: how far it goes, how long
/// it stays at full before it starts to go, and how many drawn frames it lasts altogether.
///
/// It holds before it fades because a wash that starts fading at once reads as dim however
/// bright its first frame is.
#[derive(Clone, Copy)]
struct Flash {
    alpha: u32,
    hold: u32,
    frames: u32,
}

/// A judging pass, where the wash is the instrument the pass is run with: a boundary is a
/// frame among a stage's thousands, and what says one has been reached should not be a
/// number to read. Nobody is playing, so it can take the field for the sixth of a second it
/// has — one frame in twenty is all the frame underneath gets — and still be gone before the
/// next boundary.
const FLASH_JUDGING: Flash = Flash {
    alpha: 0xc0,
    hold: 5,
    frames: 16,
};

/// A run somebody is playing, where the same wash would be in the way rather than useful: a
/// chapter begins every few seconds through a fight, and the frame one lands on is a frame
/// being dodged on. So it says a chapter began — which is where dying now sends you back
/// to — through bullets that stay readable, and is gone inside a quarter of a second.
///
/// Dimmer and shorter than a judging pass's, but not by as much as it was: 0x40 held two
/// frames of ten went unnoticed in play, where attention is on the player and not on the
/// field. Somebody watching a pass is looking straight at the frame the wash is on.
///
/// **If a wash bright enough to notice turns out to be a wash in the way, the answer is a different
/// shape and not another number**: the field's *edge* rather than over it, which says a chapter began
/// without taking any of the field away. Turning these two numbers down again only trades one of the
/// two faults for the other.
const FLASH_PLAYING: Flash = Flash {
    alpha: 0x70,
    hold: 3,
    frames: 14,
};

mod keys {
    use orb_config::VirtualKey;
    use orb_config::keys::*;

    /// Puts a boundary at the frame the game is on, and writes both files.
    pub const ADD: VirtualKey = A;
    /// Steps to the boundary either side of this frame, and holds or lets go.
    pub const NEXT: VirtualKey = RIGHT;
    pub const PREVIOUS: VirtualKey = LEFT;
    pub const HOLD: VirtualKey = SPACE;
    /// Judges the boundary the game is standing on, one step per press.
    pub const KEEP: VirtualKey = UP;
    pub const DROP: VirtualKey = DOWN;
    /// Held with a stepping key: across stages, or between the boundaries judged out.
    pub const ACROSS: VirtualKey = SHIFT;
    pub const DROPPED: VirtualKey = CTRL;
}

/// How long to wait for the game to build the stage a resume asked for before giving up on it.
///
/// Ten seconds of frames, against the second or so a stage takes to load: what this is a stop on
/// is the game having gone somewhere else with the run — back to its own menu, which is what a
/// stage it cannot load does.
const RESUME_START_FRAMES: u32 = 600;

/// The frames the screen a list of replays stands over is left to draw itself in before the list goes over
/// it.
///
/// **A second of them, because what is under one of these lists is frozen for as long as it is up.** A list
/// put up on the frame the front end became what was running stood over a screen the game had not drawn yet,
/// and went on standing over it — the chain being broken from then on, nothing ever drew the screen behind.
/// Thirty frames caught th06's logo mid-fade; sixty has it up. What these frames are for is the picture
/// behind the list, and nothing waits on them but that.
///
/// **A screen orb asked the front end for is drawn in over these frames as much as a front end just built
/// is**, which two frames of them was not enough for: th06's replay screen came up and drew its own title
/// and the row its fifteen slots are headed by over the frames after that, and a list put up on the third
/// froze it half drawn.
const THE_SCREEN_DRAWS_FOR: u32 = 60;

/// The frames a watched attempt reads no stop over, counted from the frame its playback landed on.
///
/// **For a button still down from the list the attempt was chosen out of.** A pad is read as an edge — see
/// [`menu::Keys::read`] — and the edge is against what the pad was doing the frame before, which for the
/// first frame of a watch is a `Keys` that has seen no frame at all: a button held then reads as a press.
/// The same frames the lists themselves hold their keys off for.
const STOP_GRACE_FRAMES: u32 = 12;

/// How many updates in a row a playback may run without the stage's own clock moving before it gives up on
/// that clock — and **not** how many updates a stage's frames may take, which is the game's own business.
///
/// **What a count of updates against the frame being aimed at spends is the difference on nothing.** A run
/// picked up in stage 6 ran `24179 update(s)` — the frame it was aiming at plus this — and reached frame
/// 5900: 5959 of them played the run in, and the other eighteen thousand were spent after it had died out
/// with the clock standing still. Bounded on the clock, the same playback gives up at 5959 and the game is
/// back in the player's hands fifteen seconds sooner.
///
/// And the other way the two counts differ, which no session has shown yet: th07 skips the run on one
/// update in five while it is slowing itself down (0x42e186), so a stage played through that takes as many
/// updates again — stage 5 has it and stage 6 does not.
const STALLED_FRAMES: u32 = 60;

/// How far a playback goes between saying so: ten seconds of the run it is putting back, which for the deepest
/// chapter of a stage is a line every few seconds of waiting and forty of them in all.
const SAID_EVERY: u32 = 600;

/// A stop on a chapter step, for a target the run never reaches — a replay that
/// has ended, or a stage that has no further boundary.
///
/// **Half an hour of game time, because a step back replays a stage from its start and the stop has to be above
/// a whole one.** Ten minutes was written down as longer than any stage and the Extra is not one: a replay of it
/// fought its boss to frame 42127, and a Phantasm fight with deaths in it runs longer again. What ten minutes
/// cost was silent — a step back to any chapter past it ran the stop out and held the game wherever that fell,
/// which is inside whichever attack was running: a pass stepping around 式神「橙」 came back into the middle of
/// it every time.
///
/// The cost of a stop this high is paid only where a target really is unreachable, and an update is tens of
/// microseconds: the pass this was measured on ran 36000 of them in 3.5 seconds.
const STEP_LIMIT_FRAMES: u32 = 60 * 60 * 30;

pub struct Runtime {
    game: &'static dyn Game,
    config: Config,
    data: Range<usize>,
    frames: u32,
    previous: Option<State>,
    keyboard: Keyboard,
    /// The pads, as the judging hand reads them: a controller is what somebody watching a stage go by has
    /// in their hands, so every key that pass uses is one of the game's own buttons too.
    pads: Pads,
    /// Where the mouse pointer was, for the frame loop that takes it off the screen once nothing has
    /// moved it.
    mouse: Mouse,
    /// Created on the first frame that has a Direct3D device.
    overlay: Option<Overlay>,
    /// Frames left to try building it on, so that a broken overlay is not retried for the whole
    /// run and a busy first frame is not the whole of the answer either.
    overlay_attempts: u32,
    /// The lag and the frame interval as the status line last showed them, in
    /// microseconds. Held between refreshes so the numbers can be read.
    shown: (i64, i64, i64),
    chapters: Chapters,
    /// How far the play cursor has run past the next write, at its best and worst
    /// over a reporting interval. Judging the music by ear needs someone listening
    /// at the moment it slips; these numbers do not.
    margin_worst: u32,
    margin_best: u32,
    /// Frames left to report the margin every frame, set by a restore so that what
    /// a restore does to the streaming is visible rather than averaged away.
    margin_trace: u32,
    /// The count of refills the streaming thread owes the mixer, for the stream in `stream` — see
    /// [`audio::Refills`]. `None` between streams and after anything that moved the cursor or the next write
    /// behind the count's back, and started again from the next reading.
    refills: Option<crate::audio::Refills>,
    /// **Whose the replays this launch writes are**: the name the game's own name entry was last seen holding
    /// — see [`Game::the_name_its_own_entry_holds`] — and, until one has been seen, the one the settings file
    /// carries, which is where the last launch to see one wrote it.
    ///
    /// Read from the game rather than asked for, because the game asks: a name typed on the screen a run ends
    /// at is the name that run's own replay would have been saved under, and a second place to type one is a
    /// second answer to keep in step with the first.
    name: String,
    /// What the last line about the music coming out said it was: which track, and whether its buffer was
    /// playing — see [`notice_the_track`], which says a line where this changes and nothing where it does not.
    heard: Option<(Option<u32>, bool)>,
    /// The flash now running — its numbers and the drawn frames of it left — and the
    /// boundary it was started for.
    flash: Option<(Flash, u32)>,
    flashed: Option<(i32, u32)>,
    /// The stream the music was last seen playing through. The game replaces it
    /// when it changes track, which would leave a snapshot pointing at freed
    /// memory, so it is worth knowing exactly when that happens.
    stream: (usize, Option<u32>),
    /// Restores the stress mode has done in the chapter it is on.
    stressed: u32,
    stressing: (i32, u32),
    /// `Some` while the game is frozen on the retry menu.
    retry: Option<RetryMenu>,
    /// Whether a death the count has taken is still being decided, which is a game's own deathbomb window
    /// standing between the count moving and the life being gone — see [`Game::where_a_counted_death_stands`].
    deciding_a_death: bool,
    /// The mark over the game's own count of lives, in a run where dying costs a chapter
    /// rather than one of them.
    lives: LivesMark,
    /// Whether that mark is being drawn, as `before_draw` decided at the top of this frame: read
    /// again by the drawing itself, so that the row the game was asked to repaint and the row a
    /// mark goes over are the same row — and read by the next frame's decision, which is what
    /// keeps the mark on a panel the game is still painting after the run it belonged to has ended.
    marked: bool,
    /// How many frames it has been kept for on that second count, for the line that says so.
    marked_after: u32,
    /// Which of the two things a run is, and which of the two rankings is being looked at.
    mode: Mode,
    /// Whether anybody is there to be asked which. A pass over a replay is not, and a menu
    /// frozen on a question nobody answers is a pass that never ends.
    asks_mode: bool,
    /// `Some` while the game is frozen on that question.
    asking: Option<ModeMenu>,
    /// Whether the game is being held on a chapter boundary stepped to during a
    /// replay. Its update does not run while it is; the drawing carries on, which
    /// is what makes the frame the boundary falls on something to look at.
    held: bool,
    /// Whether the end of this stage has already held the run once. Once is enough:
    /// the hold says the stage is over and going on loses the way back into it, and
    /// after that the choice has been made.
    walled: bool,
    /// Whether this run has been in a stage at all, which is what tells a stage that has
    /// ended from one that has not begun. th07 reads as a run before its first is built —
    /// its supervisor's scene lags that build by the rest of an update — and a wall there is
    /// one nothing lifts, the update it stops being the update that would build the stage.
    played: bool,
    /// Whether the ending skip has reached the staff roll, which is where it stops and
    /// leaves the game to play the roll a frame at a time.
    rolling: bool,
    /// Whether a run was given up, the one way out of a run that writes nothing.
    given_up: bool,
    /// `Some` while the game is frozen on the question of where to start a run that has one left
    /// unfinished, holding the run that would be picked up.
    asking_resume: Option<(resume_ui::ResumeMenu, resume::Kept)>,
    /// `Some` while the game is frozen on the question of whether to keep the replay of a run that is
    /// finished with, holding the slot that run's replay is being written under.
    /// The offer to keep the replay of a run finished with: the question, which run's it is, and how that run
    /// ended — the two moments this is asked at being a run that reached the screens it ends at and one being
    /// written over, which is what the kept file has to say for itself afterwards.
    asking_replay: Option<(replay_ui::SaveMenu, String, replay::Ended)>,
    /// `Some` while the game is frozen on the lists a replay is chosen out of — the replays, then the
    /// stages, chapters and attempts of the one picked.
    choosing_replay: Option<replay_ui::WatchMenu>,
    /// `Some` while an attempt out of one is being watched, holding what is being watched for the line that
    /// says the attempt is over.
    ///
    /// What it also says is that this run is nobody's: while it is set, nothing about the run is written
    /// down, no chapter of it is kept, dying does not offer a retry, and what it counted goes in no ranking
    /// — see [`Runtime::chaptering`], which is where the first three of those come from.
    watching: Option<WatchedAttempt>,
    /// The list orb's replays are chosen out of, waiting for the screen it is to stand over — see
    /// [`ToPutUp`].
    to_put_up: Option<ToPutUp>,
    /// The line drawn on the shot type select where the run under the cursor has a chapter written
    /// down. Kept across frames because it holds what it read, the screen being one somebody sits on.
    mark: resume_ui::Mark,
    /// What the question over the shot type select was answered with, waiting for the frame the run it
    /// was about is registered on. That frame is where a chapter can be put back — see
    /// [`Game::start_stage`] — and is also where the question would otherwise be asked a second time.
    started: Option<Started>,
    /// Which chapter this run's file describes — the stage and the frame it began on — so that it is
    /// written once per chapter rather than once per frame of one.
    kept: Option<(i32, u32)>,
    /// Frames spent waiting for the game to build the stage a resume asked for.
    starting: u32,
    /// Whether the game has been asked to read its keyboard the way it does without DirectInput,
    /// which is asked once and cannot be asked before the device exists.
    sent_keys: bool,
    /// Which language every screen of orb's own is written in, settled at the attach: `orb.yaml` says,
    /// or the machine does where it says nothing. Kept here rather than read per screen because it
    /// cannot change while a process runs and the machine's own is a call to the host.
    language: Language,
}

/// An attempt out of a replay being watched: how far it runs, which is where the run it is watched in ends.
struct WatchedAttempt {
    /// The frame of the stage the attempt ran to, which is where the run it is being watched in is left.
    until: u32,
    /// **The list it was chosen out of, kept for when it is over.** What somebody who has watched an attempt
    /// wants next is another attempt at that same chapter, and the walk down to it — the replay, its stage,
    /// the chapter — is one they have already made; the menu holds where its cursor was on each of the four,
    /// so putting this same one back up is that walk not made again.
    list: replay_ui::WatchMenu,
    /// And the keys the one way out of it is asked for on, read the way every menu of orb's reads its own:
    /// the keyboard's cancel keys and the pad's cancel button, each as a press rather than as a button that
    /// is down. Kept across the frames of the attempt because an edge is against the frame before.
    keys: Keys,
}

/// A list of orb's own replays waiting for the screen it is to stand over.
///
/// **The same wait whether the list is going up for the first time or coming back after an attempt**, which
/// is what makes it one thing: the screen the game keeps its own replays on is asked for and arrives a
/// second later — sixty frames of a fade on th06 — and until it is there the list would be standing over a
/// screen nobody chose. It is where a list of replays belongs, that item being what was chosen to get here.
struct ToPutUp {
    list: replay_ui::WatchMenu,
    /// Whether this is a list coming back after an attempt rather than one going up for the first time. Which
    /// is two things: the line said when it goes up, and whether the screen behind it needs drawing frames —
    /// the front end a run left is one built that frame.
    came_back: bool,
    /// And whether the game took the ask for the screen the replays are on, which is the other way a screen
    /// needs them: it came up on this list's account and draws itself in over the frames after that.
    asked: bool,
    /// The frames the screen it stands over has been up for, counted from the frame it arrived — against
    /// [`THE_SCREEN_DRAWS_FOR`].
    drawn: u32,
    /// And the frames spent waiting for a screen that never came, against [`A_SCREEN_IS_WAITED_FOR`]: a list
    /// that never goes up is worse than one over the wrong screen.
    waited: u32,
}

/// A run the resume question has been answered for, waiting for the game to get to it.
struct Started {
    /// Which run it was answered about. A run the game then registers that is not this one is a run
    /// nothing was answered about — the cursor moved between the answer and the press being handed
    /// over, which is not something the front end lets happen, but starting somebody else's chapter is
    /// not a thing to leave resting on that.
    slot: String,
    /// The chapter to put back, where that is what was asked for.
    saved: Option<resume::Kept>,
}

/// The game this process is, as the attach settled it out of [`game::KNOWN`].
///
/// A static, and not something the hooks below are handed, for the reason
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md)
/// keeps the frame loop's two calls in statics: a hook is a plain `extern` function with nothing but
/// the ABI's arguments, so where it would be handed a game it reads one. Filled by [`attach`] before
/// anything is patched and by [`attach_to`] from what an e2e test hands it — so `None` is a process
/// orb chose no game for, where none of the readers below was installed and each of them does
/// nothing of orb's if it was.
pub static GAME: MainThread<Option<&'static dyn Game>> = MainThread::new(None);

/// The game a hook is running inside, or `None` in a process orb recognised no game in.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn chosen() -> Option<&'static dyn Game> {
    *unsafe { GAME.get() }
}

pub static RUNTIME: MainThread<Option<Runtime>> = MainThread::new(None);
/// Everything the frame loop keeps between frames.
///
/// Beside the runtime rather than inside it, because the pacing is configured out of `DllMain` and
/// the runtime does not exist until the game has a device.
pub static PACING: MainThread<Option<frame::Pacing>> = MainThread::new(None);
/// Set while orb's own per-frame work is running.
///
/// Win32 calls that move a window dispatch messages synchronously, and the game
/// draws from its window proc, so a hook can be entered again from inside itself.
/// Nested entries run the game and nothing of ours: `Runtime` is handed out as
/// `&mut`, and two of those at once is not a thing that can be reasoned about.
///
/// **Nothing tests this and nothing can**, which is the one thing a simulated Windows is still the
/// answer to and does not have: the re-entry comes from a message pump, and there is no pump for an
/// e2e test to drive. So the flag is reasoned about rather than asserted.
pub static IN_HOOK: AtomicBool = AtomicBool::new(false);

pub static RUN_CALC_CHAIN: AtomicUsize = AtomicUsize::new(0);
pub static RUN_DRAW_CHAIN: AtomicUsize = AtomicUsize::new(0);
pub static UNLOCKS_READ: AtomicUsize = AtomicUsize::new(0);
pub static RANKING_READ: AtomicUsize = AtomicUsize::new(0);
pub static CARD_RECORDS_READ: AtomicUsize = AtomicUsize::new(0);
pub static STAGE_BEGUN: AtomicUsize = AtomicUsize::new(0);
pub static STAGE_BUILDING: AtomicUsize = AtomicUsize::new(0);
pub static SAVE_REPLAY: AtomicUsize = AtomicUsize::new(0);
pub static STOP_RECORDING: AtomicUsize = AtomicUsize::new(0);
pub static CREATE_GAME_WINDOW: AtomicUsize = AtomicUsize::new(0);
pub static INIT_D3D_DEVICE: AtomicUsize = AtomicUsize::new(0);
pub static RENDER: AtomicUsize = AtomicUsize::new(0);
pub static GET_INPUT: AtomicUsize = AtomicUsize::new(0);
/// And the read of a game that looks its own up: the host's own `GetKeyboardState`, which is the answer
/// orb took the game's lookup of it away from — see [`keyboard_state`], [`get_proc_address`] and
/// `game::Read::LookedUp`.
pub static GET_KEYBOARD_STATE: AtomicUsize = AtomicUsize::new(0);
/// And the lookup itself, which orb stands in front of to be the one that answers: the exe's own import
/// of `GetProcAddress` — see [`get_proc_address`].
pub static GET_PROC_ADDRESS: AtomicUsize = AtomicUsize::new(0);
/// Where the original of the one hook orb does **not** call through goes: `GetControllerInput`, whose
/// prologue is patched and whose body never runs again — see [`get_controller_input`], which answers for
/// every pad itself.
///
/// Kept because a patched prologue has to be relocated somewhere and the log names where: an offset in a
/// crash report lands in that trampoline as readily as anywhere else. Nothing reads it.
pub static GET_CONTROLLER_INPUT: AtomicUsize = AtomicUsize::new(0);
/// And the game's own read of the pad's buttons by number, which [`pad_buttons`] *does* call through — see
/// [`Hooks::pad_buttons`].
pub static PAD_BUTTONS: AtomicUsize = AtomicUsize::new(0);
/// And the game's own refill of the music buffer, which [`tops_up_the_music`] calls through — see
/// [`Hooks::tops_up_the_music`].
pub static TOPS_UP_THE_MUSIC: AtomicUsize = AtomicUsize::new(0);
/// And the game's own stop of it, which [`stops_the_music`] calls through — see
/// [`Hooks::stops_the_music`].
pub static STOPS_THE_MUSIC: AtomicUsize = AtomicUsize::new(0);
/// Whether the one hook [`attach`] installs **conditionally** is to act.
///
/// In a real launch the installation is the gate: no `block_replay_save` and the save is not hooked at all.
/// A game that hands its own functions over cannot be gated that way — it has one call site whichever way
/// the launch was configured, and a laid-out game whose saves were dropped without being asked would be orb
/// blocking a write nobody turned off. So the gate moves here, set by [`attach_to`] where [`attach`] decides
/// whether to install.
///
/// `true` to begin with, which is what leaves a real launch's behaviour exactly as it was: there the hook is
/// only ever reached because it was installed, and being installed is the decision already taken.
pub static BLOCK_REPLAY_SAVE: AtomicBool = AtomicBool::new(true);

/// Whether orb is playing the game itself — `Config::autoplay`, kept here for [`get_input`]'s reason: that
/// hook is reached from the game's own read and has no `Runtime` to ask, the way [`BLOCK_REPLAY_SAVE`] has
/// none either.
///
/// **The slot whatever is playing this run goes in**, and nothing about how it plays.
///
/// A bot is a harness rather than any part of what orb does for somebody playing, so what walks a recipe is
/// `orb-bot` and what is here is the seam it is installed through — the same shape
/// [`crate::window::install_over`], [`crate::score::install_over`] and [`crate::joystick::install_over`]
/// have, where this crate keeps the slot and something else decides what goes in it.
///
/// Empty to begin with, which is a launch nobody asked to play itself: the read hook then answers out of the
/// keyboard, the way it always has.
static PLAYING_IT: std::sync::Mutex<Option<Playing>> = std::sync::Mutex::new(None);

/// **Whatever is playing the run, asked for a word.** The game to work one out of, and where the run has got
/// to — which is what orb knows and no game does. `None` is a word the keyboard is to answer instead.
pub type PlaysTheRun = fn(&'static dyn Game, Reached) -> Option<u16>;

/// What is installed there: that, and where the run had got to when the frame hook last looked.
struct Playing {
    word: PlaysTheRun,
    reached: Reached,
    /// What it is pressing this frame, asked for once where the frame begins — see [`what_is_reached`].
    pressing: Option<u16>,
    /// The screen and the item the front end was last said to be on, so that the line saying so is written
    /// where either changes rather than every frame.
    ///
    /// **Held by the launch and not in a static of the game's**, which is what it was: a static outlives an
    /// attach, and `orb-e2e` attaches to sixteen laid-out games in one process where the real game is one
    /// attach per process — so the second launch onwards said nothing of the screen it came up on, its
    /// number being the one the launch before had left behind. What that cost is the one thing holding a
    /// laid-out launch against a real one is for: the first place every case walks read as a difference.
    said: Option<(i32, i32)>,
}

/// The game closed, for whatever is playing the run having finished with it.
///
/// Here rather than reached for directly, so that what asks for a process to end goes through the crate that
/// owns the process's own life — and through the seam, which is what a laid-out host answers instead of
/// ending a test binary.
pub fn close_the_game() {
    // Whatever the run has said put in the file first: the log is written by a thread of orb's own, so a
    // line handed over and then exited out from under is a line lost — and the last lines of a run are the
    // ones somebody closing it deliberately still wants. See `log::stop`.
    log::stop();
    // And the trace's last chunk, for the same reason: a recipe ends with `quit`, and the frames before it
    // are the ones its check is about.
    trace::close();
    orb_api::process::exit(0);
}

/// **The run given up for the title**, for whatever is playing the run wanting to leave it rather than end the
/// process. Says whether there was a run to leave.
///
/// The same thing the retry menu's own third choice does, and by the same call — see [`give_up`]. Here so that
/// a recipe can ask for it: closing the process leaves a run nobody left, and the interruption a resume is for
/// is somebody going back to the title.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn give_the_run_up(game: &dyn Game) -> bool {
    unsafe { give_up(game) }
}

/// **The run asked to start over, the way the game's own pause menu asks it** — for whatever is playing the
/// run wanting a chapter again without dying for it.
///
/// A flag rather than the question raised here, because raising it is the frame hook's: what answers the ask
/// is one block, and a recipe going another way round to the same question would be a second answer to keep
/// in step with the first. So this says somebody asked, and the block that already reads the pause menu's own
/// ask reads this beside it.
///
/// **It waits where there is no run to ask about.** A recipe that asks before a stage can retry is a recipe
/// whose ask stands until one can, which is what the pause menu's own ask does to a player who opens it too
/// early: nothing, until there is something.
pub fn start_the_run_over() {
    ASKED_TO_START_OVER.store(true, Ordering::Relaxed);
}

/// And the ask taken, which is what makes it one ask rather than one a frame.
fn asked_by_a_recipe() -> bool {
    ASKED_TO_START_OVER.swap(false, Ordering::Relaxed)
}

static ASKED_TO_START_OVER: AtomicBool = AtomicBool::new(false);

/// **The offer to retry a death answered by going back to the title**, which is that menu's own last item —
/// for whatever is playing the run wanting to give a run up the way somebody who has just died does.
///
/// **A flag and not a press, because that item is the one thing a bot cannot press its way to.** The menu is
/// orb's own and keeps its own cursor; nothing of it reaches a bot, and how many items it has is how many
/// chapters the stage has kept behind the one that was lost — so a bot walking down it would be walking a list
/// it cannot see the length of, and the item it stopped on would depend on the stage.
///
/// It waits where no offer is up, the same as [`start_the_run_over`]: what takes it is the block that answers
/// that menu, on the next frame one is up.
pub fn give_the_run_up_at_the_retry() {
    ASKED_TO_GIVE_THE_RUN_UP.store(true, Ordering::Relaxed);
}

/// And whether a bot is waiting for that answer to be taken, which is what stops it pressing at a menu it has
/// already answered: a press that lands after the menu has gone reaches the front end, and the front end
/// starts a run with it.
pub fn giving_the_run_up_at_the_retry() -> bool {
    ASKED_TO_GIVE_THE_RUN_UP.load(Ordering::Relaxed)
}

fn asked_to_give_the_run_up() -> bool {
    ASKED_TO_GIVE_THE_RUN_UP.swap(false, Ordering::Relaxed)
}

static ASKED_TO_GIVE_THE_RUN_UP: AtomicBool = AtomicBool::new(false);

/// **An item of whichever question of orb's is up, taken outright**, for whatever is playing the run.
///
/// **The generalisation of the flag above, and it is here for the reason that one is.** A question of orb's
/// keeps its own cursor and nothing of it reaches a bot, so a press can only ever take the item the cursor
/// came up on — the first of them. Every second item is therefore unreachable to a recipe: `はじめから`,
/// `いいえ` at the offer about a replay, and any stage, chapter or attempt but the first of each.
///
/// **Which three launches on the real game measured.** Each was written to answer `はじめから` by pressing
/// `down` and then `decide`; each came back `resume: from where it stopped` — the item the cursor was on —
/// and one of them then reported keeping a replay it went on writing into. A recipe that cannot say which
/// item it means cannot drive the screens this exists for.
///
/// It waits where no question is up, the same as [`give_the_run_up_at_the_retry`]: what takes it is the block
/// that answers whichever question is up, on the next frame one is.
///
/// The number is the item's place in the list as it is on the screen, counted from nought — which is what a
/// recipe can say about a list it cannot see.
pub fn answer_the_question(item: usize) {
    ANSWERED_OUTRIGHT.store(item as i64, Ordering::Relaxed);
}

/// And whether one is waiting to be taken, which is what stops a bot pressing at a question it has already
/// answered — see [`giving_the_run_up_at_the_retry`], which is the same guard on the same hazard.
pub fn answering_a_question() -> bool {
    ANSWERED_OUTRIGHT.load(Ordering::Relaxed) >= 0
}

/// The item asked for, taken: `None` where nothing has asked.
fn asked_outright() -> Option<usize> {
    match ANSWERED_OUTRIGHT.swap(NOTHING_ASKED, Ordering::Relaxed) {
        NOTHING_ASKED => None,
        item => usize::try_from(item).ok(),
    }
}

/// Which item of the question up now has been asked for, and [`NOTHING_ASKED`] where none has.
///
/// An `i64` and not an `Option<usize>` because it is a static every one of the question blocks reads: what
/// it holds is one number, and a number outside the items is a question the item is not in — said where it
/// is answered rather than checked here, the lists being as long as the file they came out of.
static ANSWERED_OUTRIGHT: AtomicI64 = AtomicI64::new(NOTHING_ASKED);
const NOTHING_ASKED: i64 = -1;

/// Installs one, or takes what is there out.
///
/// # Safety
/// Must run at the attach, before the game has been asked for a frame.
pub unsafe fn plays_the_run(word: Option<PlaysTheRun>) {
    *PLAYING_IT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = word.map(|word| Playing {
        word,
        reached: Reached {
            stage: 0,
            chapter: 0,
            question: None,
            cursor: None,
            screen: None,
            over: false,
            boss: false,
            dialogue: false,
            card: false,
            card_name: false,
            typing_a_name: false,
        },
        pressing: None,
        said: None,
    });
}
/// The game's chain functions, which orb has hooked, so calling them from its own
/// frame loop still runs everything orb does per frame.
pub static RUN_CALC_CHAIN_TARGET: AtomicUsize = AtomicUsize::new(0);
pub static RUN_DRAW_CHAIN_TARGET: AtomicUsize = AtomicUsize::new(0);
/// The two functions of the game's own that orb's frame loop calls rather than reaching through a
/// hook, as [`Game::frame_calls`] answered: the sounds handed over after the update, and the present.
///
/// Stored beside the chain targets and for the same reason — a frame reads them rather than asking the
/// game again, and a game that is not a real process fills them with functions of its own. See
/// [`attach_to`].
pub static PLAY_SOUNDS: FrameCall = FrameCall::none();
pub static PRESENT: FrameCall = FrameCall::none();
/// Whether the window-creation hook should override the game's display setting.
pub static FORCE_WINDOWED: AtomicBool = AtomicBool::new(false);
/// Whether the d-pad is to move the player, which is `dpad_moves` out of `orb.yaml`.
///
/// A static rather than the `Config` the runtime holds, for the reason every flag around it is one:
/// the input hook is entered from inside the game's own update, where a frame of orb's is already
/// holding the `Runtime` as `&mut`. Written by [`attached`], which is the one place both a real launch
/// and a game laid out by hand pass through — so the default here is only what a process has before
/// orb has read a setting, which is a process with no game in it yet.
pub static DPAD_MOVES: AtomicBool = AtomicBool::new(false);
/// Whether the record of captures in memory is emptied before a read of the score file that is about to
/// define it, which is a launch with two files to keep apart — see [`ranking_read`] and
/// `Game::forget_captures`.
///
/// A static beside [`DPAD_MOVES`] and for its reason: the hook is entered from inside the game's own update,
/// and what decides this is a setting rather than an installation. **Both of that hook's jobs used to be
/// decided by whether it went in at all**, and one of them is every launch's: what a session counted belongs
/// in whichever file the mode points at, a legacy session's as much as a rewound one's.
pub static EMPTIES_CAPTURES: AtomicBool = AtomicBool::new(false);
/// Whether the game was last given the keys, so the change can be logged once
/// rather than every frame.
pub static INPUT_ACTIVE: AtomicBool = AtomicBool::new(true);

/// **How many updates the stage's current frame has taken**, and which frame that is.
///
/// The game answers 3 to one update in five when the screen fills, which takes the chain walk with it and moves
/// neither of the run's clocks — so a frame of a stage is one update or several, and *which* frames get the
/// extra ones is the one thing a playback cannot reproduce: it drives its updates in a burst, and the count
/// that decides the skipping is the manager's own.
///
/// **Which a parting asked for.** A pair of launches of th07 agreed on every field of a frame's line — the
/// word, the player, the generator, the score, the cherry, and the fold over every item slot — for 852 frames of
/// stage 2, and on 853 the run collected a point item the playback passed through. The same state and the same word
/// cannot come out differently, so the count of updates is what differed, and nothing was saying it.
static UPDATES_THIS_FRAME: AtomicU32 = AtomicU32::new(0);
static THE_FRAME_THEY_ARE_ON: AtomicU32 = AtomicU32::new(u32::MAX);

/// Counts this read against the stage frame it is for, and says how many the frame has taken including it.
fn updates_of(frame: Option<u32>) -> u32 {
    let frame = frame.unwrap_or(u32::MAX);
    if THE_FRAME_THEY_ARE_ON.swap(frame, Ordering::Relaxed) == frame {
        UPDATES_THIS_FRAME.fetch_add(1, Ordering::Relaxed) + 1
    } else {
        UPDATES_THIS_FRAME.store(1, Ordering::Relaxed);
        1
    }
}

/// And how many the frame being logged took, without counting one: the line is written from the frame loop and
/// the counting is done at the read.
fn updates_taken() -> u32 {
    UPDATES_THIS_FRAME.load(Ordering::Relaxed)
}
/// Whether the keyboard has been out of reach since it was last read, and so needs
/// getting back before the next read.
pub static INPUT_LOST: AtomicBool = AtomicBool::new(false);
/// Which frame of a run the word it acts on was last handed over for, and [`NO_FRAME`] where that was a read
/// outside a stage — see [`the_first_read_of`], and `Game::hand_the_read_to_the_run` for what it decides.
static HANDED_OVER_FOR: AtomicI64 = AtomicI64::new(NO_FRAME);
const NO_FRAME: i64 = -1;
/// Whether the front end's own decide is being kept from the game — see [`held_back`]. Set for as
/// long as the screen orb has a question about is up with a run of a chapter under the cursor, and
/// left alone while the question itself is up: the frame that writes it ends in the question's own
/// branch and never reaches the write.
pub static HOLD_DECIDE: AtomicBool = AtomicBool::new(false);
/// Whether one of those presses has arrived and has not been asked about yet.
pub static DECIDE_PRESSED: AtomicBool = AtomicBool::new(false);
/// Whether it was down on the read before. What is held back is the press and not the holding, and
/// the game's own `g_LastFrameInput` cannot tell orb which this is: the bits were taken out of the
/// word it is assigned from.
pub static DECIDE_WAS_DOWN: AtomicBool = AtomicBool::new(false);
/// Whether the press held back is to be handed over on the next read, the question having been
/// answered with a run to start.
pub static FEED_DECIDE: AtomicBool = AtomicBool::new(false);
/// Whether the key a question was cancelled with is being kept from the game until it is let go. What
/// the screen underneath would do with it is go back — see [`Game::menu_cancel`].
pub static HOLD_CANCEL: AtomicBool = AtomicBool::new(false);
/// Whether the next read is the first after one of orb's questions came down, on which nothing the
/// game was not already holding is let through — see [`held_back`].
pub static SETTLE_KEYS: AtomicBool = AtomicBool::new(false);
/// The word the game was handed on the read before, which is what the game's own idea of the frame
/// before still says while a question of orb's is up: no frame of one runs its chain, so nothing
/// assigns `g_LastFrameInput`.
pub static LAST_WORD: AtomicU16 = AtomicU16::new(0);

/// One of the frame loop's two calls into the game, as a static: the address, and what to call it on.
pub struct FrameCall {
    function: AtomicUsize,
    this: AtomicUsize,
}

impl FrameCall {
    /// **`const fn` because both of these are statics**, which is also why no e2e test can enter it: what
    /// happens here is const evaluation and not execution. See `sync::MainThread::new`, which is the same
    /// zero for the same reason.
    pub const fn none() -> Self {
        Self {
            function: AtomicUsize::new(0),
            this: AtomicUsize::new(0),
        }
    }

    pub fn store(&self, call: game::Call) {
        self.function.store(call.function, Ordering::Relaxed);
        self.this.store(call.this, Ordering::Relaxed);
    }

    /// # Safety
    /// Must run on the game's main thread with whatever that call itself wants of the frame — see
    /// [`game::FrameCalls`] — and after the attach that filled this one.
    unsafe fn call(&self) {
        let function: unsafe extern "fastcall" fn(usize) =
            unsafe { std::mem::transmute(self.function.load(Ordering::Relaxed)) };
        unsafe { function(self.this.load(Ordering::Relaxed)) };
    }
}

/// The game's own `CreateWindowExA`, whose arguments the window's own rewrite decides.
///
/// Windows' own signature with `*mut c_void` where it says `HWND` and `HMENU`, which is what those are:
/// this crate names no `windows-sys` type, and a handle is a pointer whichever header declares it.
/// `orb::window` holds its own hook to this alias, so the two cannot drift.
#[allow(clippy::type_complexity)]
pub type CreateWindowExA = unsafe extern "system" fn(
    u32,
    *const u8,
    *const u8,
    u32,
    i32,
    i32,
    i32,
    i32,
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *const c_void,
) -> *mut c_void;

/// And its own `CreateFileA`, whose name the score file's fork decides. `*mut c_void` for `HANDLE`, the
/// same way.
pub type CreateFileA = unsafe extern "system" fn(
    *const u8,
    u32,
    u32,
    *const c_void,
    u32,
    u32,
    *mut c_void,
) -> *mut c_void;

/// And its own `joyGetPosEx`, as its import table held it. What it fills is [`orb_api::JoyInfo`], which
/// is `JOYINFOEX`'s own layout, so this is the signature winmm exports rather than a shape of orb's.
pub type JoyGetPosEx = unsafe extern "system" fn(u32, *mut orb_api::JoyInfo) -> u32;

/// The game's own functions orb's hooks call through, which in a real process are the trampolines
/// [`hook::install`] leaves behind and for a game that is not one are its own.
///
/// One per hook a game reaches orb through, which is what a frame of one is made of: its update and
/// its draw, the read every key it acts on passes through, the two moments a stage's numbers are put
/// in place, and the two reads of the score file orb has something to say about. Then the frame loop:
/// the loop of the game's own that [`render`] replaced and hands the frame back to, and the two calls
/// [`Game::frame_calls`] answers with, which in a real process are the exe's code at its own addresses
/// and here are functions a laid-out game brings.
pub struct Originals {
    pub update: extern "fastcall" fn(*mut c_void) -> i32,
    pub draw: extern "fastcall" fn(*mut c_void) -> i32,
    pub input: extern "system" fn() -> u16,
    pub stage_building: extern "C" fn(i32) -> i32,
    pub stage_begun: extern "C" fn(*mut c_void) -> i32,
    pub unlocks_read: extern "C" fn(*mut c_void) -> i32,
    pub ranking_read: extern "C" fn(*mut c_void) -> i32,
    /// The game's own parse of the records its score file holds, which [`card_records_read`] calls through
    /// to: two registers, the file's image and the array it lays that file's records into.
    pub card_records_read: unsafe extern "fastcall" fn(usize, usize) -> i32,
    /// The game's own whole frame, which is what [`render`] hands one back to on the three ways out
    /// of it that return: no runtime, no device, and a chain target that is null.
    pub render: extern "fastcall" fn(*mut c_void) -> i32,
    /// Its sound and its present. Called on nothing, a function of a laid-out game's own reaching
    /// that game the way the hooks above it do rather than through a `this` it was handed.
    pub play_sounds: unsafe extern "fastcall" fn(usize),
    pub present: unsafe extern "fastcall" fn(usize),
    /// Its own `CreateWindowExA`, which the rewrite calls through with the arguments it decided —
    /// where a real launch reaches the exe's import table for it. A laid-out game hands it over for
    /// the same reason it hands over the two above: there is no import table to patch.
    pub create_window: CreateWindowExA,
    /// And its own `CreateFileA`, which the score file's fork calls through with the name it decided.
    /// The same story, at the same import table there is none of.
    pub create_file: CreateFileA,
    /// And its own `ReplayManager::StopRecording`, which [`stop_recording`] calls through where the
    /// record being written is a run's own rather than a replay's. Reached in a real launch by a patch
    /// over that function's prologue, and here by the game calling it where its own teardown does.
    pub stop_recording: extern "C" fn(),
    /// And its own `GameWindow::Create`, which [`create_game_window`] calls through once the display
    /// setting it found there has been overruled. A patch over that function's prologue in a real launch;
    /// here the game calls the hook where it would have created its window, and this is what the hook
    /// calls back into.
    pub create_game_window: extern "C" fn(*mut c_void),
    /// And its own `joyGetPosEx`, as its import table held it: what [`joystick::answer`] calls through on
    /// the reads it has no sample for. The import entry is what a real launch patches; a laid-out game
    /// hands the entry over and calls the replacement where its own read would have gone through it.
    pub joystick_position: JoyGetPosEx,
    /// And its own read of the pad's buttons by number, which [`pad_buttons`] calls through to — the one
    /// hook of this list orb does not answer the whole of.
    pub pad_buttons: extern "C" fn() -> *mut c_void,
    // And **not** its own `Controller::GetControllerInput`, which is the one hook orb does not call
    // through: what a pad does to the word is [`get_controller_input`]'s own answer now, so a laid-out
    // game has nothing to hand over for it. It calls that hook where its keyboard read tail-calls the
    // function, and that is the whole of the wiring.
    /// And its own `ReplayManager::SaveReplay`, which [`save_replay`] drops the write of while leaving the
    /// teardown the game does through the same function.
    ///
    /// Handed over always, where a real launch installs the hook only under `block_replay_save`: the gate
    /// cannot be the installation here, since a laid-out game has one call site whichever way the launch was
    /// configured. It is [`BLOCK_REPLAY_SAVE`] instead, set where the patch would have been installed.
    pub save_replay: extern "C" fn(*const u8, *const u8),
    /// And its own device setup, which [`init_d3d_device`] gets in front of to redirect `Present` before
    /// anything is presented through it.
    ///
    /// Not something a real launch hands over — the hook is always installed there, and orb attaches before
    /// the device exists — so a laid-out game that wrote its device before orb was attached would leave this
    /// unreachable. See `Fake::attach_before_its_device`.
    pub init_d3d_device: extern "C" fn(),
    /// And its own refill of the music buffer, which [`tops_up_the_music`] says what is being fed through and
    /// then calls: `__thiscall` on the stream with the one argument both games' own threads pass it.
    /// **Of the 32-bit games and no other**, which is what the ABI settles: `__thiscall` is an x86
    /// calling convention and there is no such thing to spell anywhere else — see `game::th06`, whose
    /// module is the same architecture's. A game whose music orb does not feed has neither of these.
    #[cfg(target_arch = "x86")]
    pub tops_up_the_music: unsafe extern "thiscall" fn(usize, i32) -> i32,
    /// And its own stop of that music, which [`stops_the_music`] says is happening and then calls:
    /// `__thiscall` on the stream for th07 and on the sound player for th06, neither taking an argument.
    #[cfg(target_arch = "x86")]
    pub stops_the_music: unsafe extern "thiscall" fn(usize) -> i32,
}

/// Attaches orb to a game that is not a real process: `originals` in place of the trampolines, and
/// then the same runtime [`attach`] leaves behind.
///
/// What [`attach`] does above this and an e2e test cannot: read a `.data` section out of the PE, load
/// `orb.yaml` from beside an exe, and patch a call site in the game's code. A game laid out by hand
/// has none of those — it hands over its own memory's bounds, a `Config` written in the open, and its
/// own functions where the patches would have pointed.
///
/// # Safety
/// Must run on the thread the game's frames will run on, with a simulated Windows installed there,
/// and every function in `originals` must outlive the last frame orb's hooks are reached on.
pub unsafe fn attach_to(
    game: &'static dyn Game,
    config: Config,
    data: Range<usize>,
    originals: Originals,
) {
    log::open();
    log!(
        "orb {} attached to a game laid out in this process",
        env!("CARGO_PKG_VERSION")
    );
    log::set_level(config.log_level);
    log::set_pacing(config.pacing_log);
    // Where [`attach`] settles this off the exe's own name, a game laid out by hand is handed over as
    // itself: there is no file to read the name of, and an e2e test that had to name one would be
    // choosing its game twice.
    unsafe { *GAME.get() = Some(game) };
    // The flags a fresh process would have brought, since a game that is not one does not bring it: a
    // launch is the moment nothing is being held back, nobody has pressed anything, and the keyboard
    // has not been lost. Left standing, one game's would be the next game's opening state — a press
    // held back on the screen a game before ended on, and a question up over the one that follows.
    IN_HOOK.store(false, Ordering::Relaxed);
    INPUT_ACTIVE.store(true, Ordering::Relaxed);
    INPUT_LOST.store(false, Ordering::Relaxed);
    HOLD_DECIDE.store(false, Ordering::Relaxed);
    DECIDE_PRESSED.store(false, Ordering::Relaxed);
    DECIDE_WAS_DOWN.store(false, Ordering::Relaxed);
    FEED_DECIDE.store(false, Ordering::Relaxed);
    HOLD_CANCEL.store(false, Ordering::Relaxed);
    SETTLE_KEYS.store(false, Ordering::Relaxed);
    LAST_WORD.store(0, Ordering::Relaxed);
    // And nothing of a run written down, which is the one piece of that state a `Runtime` does not
    // hold: the record lives beside it, for the two hooks that reach it from inside the game's update.
    unsafe { resume::forget() };
    // The decision [`attach`] takes by installing a hook or not — see [`BLOCK_REPLAY_SAVE`].
    BLOCK_REPLAY_SAVE.store(config.block_replay_save, Ordering::Relaxed);
    for (slot, original) in [
        (&RUN_CALC_CHAIN, originals.update as usize),
        (&RUN_DRAW_CHAIN, originals.draw as usize),
        (&GET_INPUT, originals.input as usize),
        (&STAGE_BUILDING, originals.stage_building as usize),
        (&STAGE_BEGUN, originals.stage_begun as usize),
        (&UNLOCKS_READ, originals.unlocks_read as usize),
        (&RANKING_READ, originals.ranking_read as usize),
        (&CARD_RECORDS_READ, originals.card_records_read as usize),
        (&RENDER, originals.render as usize),
        (&STOP_RECORDING, originals.stop_recording as usize),
        (&CREATE_GAME_WINDOW, originals.create_game_window as usize),
        (&SAVE_REPLAY, originals.save_replay as usize),
        (&INIT_D3D_DEVICE, originals.init_d3d_device as usize),
        (&PAD_BUTTONS, originals.pad_buttons as usize),
        // What the patched call sites would be. In a real process these are the game's own two chain
        // functions with a jump to orb's hooks written over their prologues, so the frame loop calling
        // the address it was handed runs everything orb does per frame; here the hooks are what there
        // is, and calling them straight is the same path.
        (&RUN_CALC_CHAIN_TARGET, run_calc_chain as *const () as usize),
        (&RUN_DRAW_CHAIN_TARGET, run_draw_chain as *const () as usize),
    ] {
        slot.store(original, Ordering::Relaxed);
    }
    // And the music's own two apart from the rest, the `__thiscall` they are being an x86 ABI and so
    // an x86 field — see [`Originals::tops_up_the_music`]. Outside the list above because a `cfg` on an
    // element of an array is not a thing to write.
    #[cfg(target_arch = "x86")]
    {
        TOPS_UP_THE_MUSIC.store(originals.tops_up_the_music as usize, Ordering::Relaxed);
        STOPS_THE_MUSIC.store(originals.stops_the_music as usize, Ordering::Relaxed);
    }
    // And the two the frame loop calls in the game rather than reaching through a hook. Nothing to
    // call them on: see [`Originals`].
    PLAY_SOUNDS.store(game::Call {
        function: originals.play_sounds as usize,
        this: 0,
    });
    PRESENT.store(game::Call {
        function: originals.present as usize,
        this: 0,
    });
    // And the window, which is the same story: what the game gets is decided by orb either way, and a
    // laid-out game reaches the rewrite by calling it rather than by having its imports patched. Always,
    // as [`attach`] installs it always — the answer is orb's whichever of the two the settings say.
    unsafe {
        crate::window::install_over(
            originals.create_window,
            game.content_size(),
            config.screen,
            config.autoplay.is_some(),
        )
    };
    // And the pointer over that window, whose rewrite is reached the same way: a laid-out game calls it
    // where the real one's patched `ShowCursor` entry would have taken it. With the setting handed over
    // rather than the call gated on it, so that a launch told not to hide the pointer is one that says so
    // here — where [`attach`] answers the same setting by leaving the entry alone.
    crate::mouse::install(config.hide_mouse);
    // And the display setting that window is made under, which is overruled once and where [`attach`]
    // overrules it: a game that has taken the display exclusively has no window to resize, and by the
    // time anything of orb's runs per frame the device already exists. Set here rather than in the reset
    // above because it is the one flag a fresh process brings *set*.
    FORCE_WINDOWED.store(true, Ordering::Relaxed);
    // And the joystick, which is the same story again: the read moves onto a thread of orb's own either
    // way, and what differs is whether the entry it stands in front of was patched or handed over.
    crate::joystick::install_over(originals.joystick_position);
    log!("joystick: read on a thread of orb's, out of the game's frame");
    // The score file's fork, on the same gate [`attach`] puts it behind: only where a run can be rewound,
    // and always for a clear — there the fork is not what it is for, and being in the path of the write is.
    if (config.chapters && game.rewinds()) || config.fast_clear {
        if config.fast_clear {
            crate::score::refuse_writes();
        }
        unsafe { crate::score::install_over(originals.create_file) };
        if config.fast_clear {
            log!("score: no file is written this run");
        } else {
            log!("score: score.dat is forked while orb is in pointdevice mode");
        }
    }
    // The frame loop from nothing, and set up as [`attach`] sets it up: the cadence off the desktop's
    // own rate before there is a window to ask about, and the compositor's drawing time pinned where
    // the launch pinned it. Thrown away first rather than configured again, because what a `Pacing`
    // carries is a run's own — the frames it has counted and the gaps it has put them in are what the
    // `frame:` line is written from, and a game before this one's would be added to this one's.
    unsafe { *PACING.get() = None };
    unsafe { pacing() }.configure();
    unsafe { pacing() }.pin_compose(config.compose_us);
    // And the frames asked for sooner than the game's own rate where the launch asked — see
    // [`frame::Pacing::paces_faster`], which is what a walk to some stage of a run is spent on.
    unsafe { pacing() }.paces_faster(config.pace);
    unsafe { attached(game, config, data) };
}

/// Takes the runtime down, which is what closing the game does to it.
///
/// For an e2e test that plays one game and then another in the same process: a runtime left standing
/// would hold the first game's chapters and draw its overlay through a device that has gone. Dropped
/// here rather than left to `DllMain`'s own detach, where the process is going away and an overlay
/// released through a device Direct3D has already torn down is a fault on the way out.
///
/// # Safety
/// Must run on the game's main thread, with no frame in progress and the game it was attached to —
/// its memory and the device it draws through — still there.
pub unsafe fn detached() {
    unsafe { *RUNTIME.get() = None };
    // Before the log, since the writer says what it wrote: a line from it afterwards would go into
    // whatever log this process opens next. See `writer::stop` for why waiting for it is this path's to
    // do and not `DllMain`'s.
    writer::stop();
    // And the trace after the writer, so that the writer's own lines are in its last chunk.
    trace::close();
    // And the log, which a real launch closes from `DllMain`'s `DLL_PROCESS_DETACH` and which this is
    // the whole of the way out of for anything else. Left open it is a handle onto a game that has
    // gone: the next `log::line` writes through it, and where the next thing along is another game in
    // the same process — which is an e2e test file with two launches in it — that write lands in *its*
    // log, before it has opened one. Three counter reads a line, so it moves that launch's clock too.
    log::close();
}

/// Which of the two a launch starts in and what that decides, and the runtime the hooks then find.
///
/// Apart from [`attach`] because it is the whole of what a game has to have done to it, with nothing
/// in it about a process: see [`attach_to`].
///
/// # Safety
/// Must run on the game's main thread, before any of orb's hooks are reached.
pub unsafe fn attached(game: &'static dyn Game, config: Config, data: Range<usize>) {
    // Which mode a launch starts in, before anybody has been asked: pointdevice, since that is
    // what orb is for, and normal where `--no-chapters` has left it nothing to be — or where the game
    // itself has nothing orb can rewind, which is a game orb has read no run of. See [`Game::rewinds`]
    // for the file that came of not asking.
    let rewinds = game.rewinds();
    let chapters = config.chapters && rewinds;
    let mode = if chapters {
        Mode::Pointdevice
    } else {
        Mode::Normal
    };
    // The question is only put to somebody who is there to answer it. A pass over a replay has
    // nobody at the keyboard, and neither has a clear: those take the mode they are given.
    let asks_mode =
        chapters && !config.during_replay && !config.chapter_tuning && !config.fast_clear;
    log!(
        "mode: {mode} to start with; {}",
        if asks_mode {
            "the menu asks which"
        } else if !rewinds {
            "there is no other here, nothing of this game's runs having been read"
        } else {
            "nobody is asked"
        }
    );

    // Whether this launch keeps what it plays, and which runs an earlier one left unfinished. The
    // names only: which of them is offered depends on what is chosen at the character select, and
    // this is the line that says whether there was anything there to offer at all.
    let keeping = config.chapters && config.resume && mode == Mode::Pointdevice;
    resume::keep(keeping);
    // And the replay of the whole run beside it, which is the same answer and one thing more: a launch that
    // refuses to write a replay refuses this one — see [`Runtime::keeping_the_replay`], where the reason is.
    replay::keep(keeping && !config.block_replay_save);
    if config.resume {
        let left = resume::left(&config.base_dir);
        log!(
            "resume: {} run(s) left unfinished{}{}",
            left.len(),
            if left.is_empty() { "" } else { ": " },
            left.join(", "),
        );
    }
    // Which of the two rankings this run is headed for, said whether or not the hook that acts on it
    // went in: with nothing able to rewind, the mode is normal and this is the game's own file, which
    // is where it was anyway.
    score::fork(mode == Mode::Pointdevice);
    // And whether the read of that file empties the record of captures first, which is the same question:
    // two files to keep apart is a launch that can rewind a run. A setting rather than an installation
    // because the hook has a second job that is every launch's — see [`EMPTIES_CAPTURES`].
    EMPTIES_CAPTURES.store(config.chapters, Ordering::Relaxed);
    // What the input hook adds to the word the game read, which is a setting rather than an
    // installation: the hook goes in whatever it says, there being every other reason for it to.
    DPAD_MOVES.store(config.dpad_moves, Ordering::Relaxed);
    log!(
        "joystick: the d-pad {}",
        if config.dpad_moves {
            "moves the player, which the game's own read of a pad does not"
        } else {
            "is left as the game has it, which is doing nothing"
        }
    );

    // Which language every screen of orb's own is in, settled once here: the file says, or the machine
    // does where it says nothing. Said in the log because a screenshot of a menu somebody could not
    // read is otherwise the only evidence of which it came out as.
    let language = config.language.unwrap_or_else(Language::of_the_machine);
    log!(
        "language: {language}, {}",
        match config.language {
            None => "which is what this machine's own windows are in",
            Some(_) => "which is what orb.yaml asks for",
        }
    );

    // Where a launch being checked writes down what every frame held — see [`trace`]. Beside the game, as
    // every file orb keeps is, where the path names no directory of its own.
    if let Some(path) = &config.trace {
        trace::open(&config.base_dir.join(path));
    }
    // And whether this launch is looking for a field of the game's own rather than playing it — see
    // [`find`], whose pass is nine megabytes a frame and so belongs to no other launch.
    unsafe { find::looks_for_what_moves(config.find) };

    let tuning = config.chapter_tuning.then(|| config.base_dir.clone());
    let during_replay = config.during_replay;
    // What the settings file carries, which is what the last launch to see the game's own name entry wrote
    // there — see [`Runtime::name`] and [`notice_the_name`].
    let name = config.player_name.clone();
    unsafe {
        *RUNTIME.get() = Some(Runtime {
            game,
            config,
            data,
            frames: 0,
            previous: None,
            keyboard: Keyboard::new(),
            pads: Pads::new(),
            mouse: Mouse::new(),
            overlay: None,
            overlay_attempts: OVERLAY_ATTEMPTS,
            shown: (0, 0, 0),
            chapters: Chapters::new(game, tuning, during_replay),
            margin_worst: 0,
            margin_best: u32::MAX,
            margin_trace: 0,
            refills: None,
            name,
            heard: None,
            flash: None,
            flashed: None,
            stream: (0, None),
            stressed: 0,
            stressing: (-1, 0),
            retry: None,
            deciding_a_death: false,
            lives: LivesMark::new(),
            marked: false,
            marked_after: 0,
            mode,
            asks_mode,
            asking: None,
            held: false,
            walled: false,
            played: false,
            rolling: false,
            given_up: false,
            asking_resume: None,
            asking_replay: None,
            choosing_replay: None,
            watching: None,
            to_put_up: None,
            mark: resume_ui::Mark::new(language),
            started: None,
            kept: None,
            starting: 0,
            sent_keys: false,
            language,
        });
    }
}

impl Runtime {
    /// Whether this run has chapters at all: the mode says so, `--no-chapters` says not, and an attempt out
    /// of a replay being watched is not a run anybody is playing.
    ///
    /// **That last one turns off everything a run of somebody's own gets**, and each of them would be wrong
    /// here: a snapshot to send the player back to, an offer to retry a death that already happened, a
    /// chapter written down over the file of the run this attempt came out of, and a replay of a replay.
    fn chaptering(&self) -> bool {
        self.config.chapters && self.mode == Mode::Pointdevice && self.watching.is_none()
    }

    /// And whether what it presses is being written down, so that the chapter it is left in can be
    /// played again in a later launch.
    fn keeping(&self) -> bool {
        self.chaptering() && self.config.resume
    }

    /// And whether a replay of the whole of it is being kept, which is that and one thing more: a launch
    /// that refuses to write a replay refuses this one too.
    ///
    /// `--clear` is what sets that, and the reason is written beside it: what such a run recorded is the
    /// buttons of a player nothing could hit, so a replay of it plays back as a run that dies where this one
    /// did not. Which is the same reason that flag takes the chapter file away — and the chapter file has
    /// `resume` to be turned off by where this has nothing of its own, orb's own replay being newer than
    /// either.
    fn keeping_the_replay(&self) -> bool {
        self.keeping() && !self.config.block_replay_save
    }
}

/// What the pads are doing, for a menu of orb's own.
///
/// Asked of the game, and read as *its* buttons, because the frames these menus are up are frames
/// the game's own input is not running on — so a pad does nothing on them unless orb does this.
/// Which of a game's ways of reaching a pad it is read through is the game's business too: see
/// `Game::pad`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn pad(game: &dyn Game) -> Pad {
    let mut pad = unsafe { game.pad(joystick::reading()) };
    // **And what a launch playing itself is pressing, as though it were pushing the pad.** orb's own screens —
    // the mode question, the offer of a run left unfinished, the offer to retry a death — read the keyboard and
    // the pad, and a bot's word goes to the *game*: without this there is no way for one to answer them, and a
    // launch that died sat on that offer until somebody looked at the screen. The pad rather than the keyboard
    // because a pad is already a set of named buttons, which is what a word is.
    if let Some(word) = pressing() {
        let held = |name| game.button_bit(name).is_some_and(|bit| word & bit != 0);
        pad.shot |= held("shot");
        pad.bomb |= held("bomb");
        pad.focus |= held("focus");
        pad.skip |= held("skip");
        pad.up |= held("up");
        pad.down |= held("down");
        pad.left |= held("left");
        pad.right |= held("right");
        // **And the two the menus actually read** — `Menu::read` takes its answer off `decide` and `cancel` and
        // off nothing else, so a fold that set the buttons and left these two is a bot whose every press went
        // nowhere. Which is what one did: it died, was offered the retry, and sat on it.
        pad.decide |= held("decide");
        pad.cancel |= held("cancel");
    }
    pad
}

/// **Where the run has got to and what the bot presses for it**, worked out once where the frame begins.
///
/// Once because the walker counts the frames it has been on a step, so a second ask would be a second frame;
/// and *here* because the blocks after this one hold the frame back and return — orb's own offer to retry a
/// death is one of them, so a fill any later never says a retry is up at all, which is a bot that cannot be told
/// to answer it.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn what_is_reached(runtime: &Runtime) {
    let mut held = PLAYING_IT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(playing) = held.as_mut() else {
        return;
    };
    playing.reached.chapter = runtime.chapters.number();
    playing.reached.cursor = unsafe { runtime.game.front_end_cursor() };
    playing.reached.screen = unsafe { runtime.game.front_end_screen() };
    // **Which screen is up and which item its cursor is on, said whenever either changes.** That is what a
    // recipe naming an item by where it sits is written against: the cursor is one field whatever screen is
    // up, so a step waiting for a number cannot tell the title's fourth item from a difficulty select's, and
    // which screen is which number is read off these lines rather than off either exe.
    //
    // A screen from a game that answers none is said as -1, which is the number those lines have always
    // carried for it: th07 reads the field behind the same walk as the cursor and answers nothing where the
    // walk ends short.
    if let Some(cursor) = playing.reached.cursor {
        let screen = playing.reached.screen.unwrap_or(-1);
        if playing.said != Some((screen, cursor)) {
            playing.said = Some((screen, cursor));
            detail!("front end: screen {screen}, cursor on item {cursor}");
        }
    }
    playing.reached.question = question_up(runtime);
    // **And the stage and whether the run is over, for a game orb stands in no frame of.** Everything a
    // stage is read for is filled where the frame ends — see the block after `watch_music`, which is
    // where the state is read anyway — and a game whose `Hooks::update` is `None` never reaches it: the
    // stage stayed nought and the run read as never over, so a `play_until` could wait for neither. What
    // that cost was a launch of th06c whose run lost its last life, whose front end the rules then went
    // on pressing at, and which started a second run the same recipe step was still counting frames of.
    //
    // **Read here rather than left to the frame**, because here is the only call a game like that has:
    // this runs once a tick, off its own read of its keyboard.
    if runtime.game.hooks().update.is_none() {
        let state = unsafe { runtime.game.read_state() };
        playing.reached.stage = state.stage + 1;
        playing.reached.over = !state.in_run;
        playing.reached.dialogue = state.in_dialogue;
        playing.reached.boss = state.boss_present;
        // And whether the game's own name entry is up, which a `play` step runs straight past otherwise —
        // see `recipe::Until::Name`.
        playing.reached.typing_a_name = unsafe { runtime.game.typing_a_name() };
    }
    playing.pressing = (playing.word)(runtime.game, playing.reached);
}

/// Which of orb's own questions is up, if one is: what a recipe's step waits for, and what a trace records
/// the frame under.
fn question_up(runtime: &Runtime) -> Option<Question> {
    if runtime.asking.is_some() {
        Some(Question::Mode)
    } else if runtime.asking_resume.is_some() {
        Some(Question::Resume)
    } else if runtime.retry.is_some() {
        Some(Question::Retry)
    } else if runtime.asking_replay.is_some() {
        Some(Question::KeepReplay)
    } else if runtime.choosing_replay.is_some() {
        Some(Question::ChooseReplay)
    } else {
        None
    }
}

/// What the bot is pressing this frame, or nothing where no launch is playing itself.
///
/// `pub(crate)` for the game that has to put that word somewhere itself: th06c's calc walk writes it
/// into the game's own keys, the read those would otherwise come through being made at its own rate
/// rather than once a frame — see `game::Game::presses_into_its_own_keys`.
pub(crate) fn pressing() -> Option<u16> {
    PLAYING_IT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .and_then(|playing| playing.pressing)
}

/// Takes the answer to the mode question: what a run does, and which of the two files the
/// scores are kept in.
///
/// One answer for both, because they are one thing: the ranking of pointdevice runs is the file
/// pointdevice runs are written to. What the game has unlocked is not part of that answer — the
/// menu is lit from `score.dat` whichever mode is chosen, since a stage reached is a stage
/// reached.
///
/// **What that file records is part of it**, and it came out of `score.dat` with the unlocks: the front
/// end reads both in one go, before anybody has been asked. So it is read again here, out of the file
/// this answer just chose — which is the moment before the screen those numbers are drawn on, a practice
/// stage being chosen after this question. See [`Game::read_what_the_modes_file_records`], which the
/// front end's own read is bracketed by as well: this is the moment the *mode* changes, and that one is
/// every path that brings the screen up.
///
/// # Safety
/// Must run on the game's main thread, between frames: that read goes through the game's own code.
unsafe fn choose(runtime: &mut Runtime, mode: Mode) {
    let was = std::mem::replace(&mut runtime.mode, mode);
    score::fork(mode == Mode::Pointdevice);
    unsafe { the_modes_own_records_again() };
    resume::keep(runtime.keeping());
    // The replay of a run follows the same two things the chapter it is left in does: the mode, and orb
    // keeping what the run pressed at all. What each of them then *does* with it is its own — see [`replay`].
    replay::keep(runtime.keeping_the_replay());
    // Nothing kept of a run that will not be rewound: a stage's snapshots are several megabytes a
    // chapter and it keeps `chapter::KEPT_CHAPTERS` of them, and normal mode is the game as it was.
    // Its buttons go with them — a run that cannot be rewound has nothing a chapter would be
    // resumed into.
    if mode == Mode::Normal {
        runtime.chapters.forget();
        unsafe { resume::forget() };
        unsafe { replay::forget() };
    }
    log!("mode: {mode}, was {was}");
}

/// Has the game build the screen a ranking is shown on and take it down again, with nothing drawn, that
/// being the one place it writes the records its score file holds.
///
/// A run given up writes nothing, so what it counted about spell cards would go with the process. The
/// screen is asked for the way the front end's item asks, the updates that brings run inside this one
/// frame, and then the screen is told to leave the way its own menu tells it to — it puts the scene
/// back itself, and going down is what writes. Nothing is seen: drawing happens once a frame, after.
///
/// **The record goes back in the middle**: building that screen is also the game loading the record
/// out of the file, which is what it held before this session counted anything.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread, with no run in progress and the
/// game settled where it is.
unsafe fn commit_records(runtime: &mut Runtime, chain: *mut c_void, result: &mut i32) {
    // The game's own save first, where it has one that writes: th07's ranking screen writes nothing at either
    // phase it is built with, so the pump below would spend frames on a screen and leave the file as it was —
    // see `Game::save_its_records`. What the run counted goes on top of the file's own records inside that call,
    // at the seam `card_records_read` is hooked over.
    if runtime.game.writes_its_own_records() {
        return log!(
            "score: this game writes its own file; what the run counted waits for the save it makes itself"
        );
    }
    let captures = unsafe { runtime.game.captures() };
    if captures.is_empty() {
        return log!("score: this game keeps no record of captures; nothing to write");
    }
    let mut frames = 0;
    // **The walk giving up stops this, and the frame limit stops it otherwise.** `CHAIN_EXIT_ERROR` is
    // what the game's own frame returns on — th07 tests the answer at 0x434801 and returns at 0x43480c,
    // and th06's frame does the same — so a walk asked for after it is a walk no frame of the game's own
    // makes, over a chain whose scene is being taken apart. A launch of th07 faulted in exactly that:
    // three more walks after the answer, inside the pump that takes the ranking down.
    //
    // `CHAIN_EXIT_SUCCESS` is not that answer, though the game's frame returns on it too: it is 0, which
    // is also what a walk answers when it simply carried on, so guarding on it ended both loops after one
    // update — the screen was left standing where the player then met it, and the write came from whatever
    // the screen itself did later.
    let mut gave_up = false;
    // The game itself rather than the runtime, so that what the walk left behind can be said per update
    // without the closure holding everything: a fault inside this loop is a fault on one of these updates,
    // and which one it was and what the screen's own state was then is the whole of what a log can say about
    // it. Verbose, because it is one line per update and there are up to [`COMMIT_FRAME_LIMIT`] of them.
    let game = runtime.game;
    let mut running = |result: &mut i32, frames: &mut u32| {
        if gave_up {
            return false;
        }
        *result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        *frames += 1;
        detail!("score: update {frames} answered {result} — {}", unsafe {
            game.ranking_state()
        });
        if *result == CHAIN_EXIT_ERROR {
            gave_up = true;
            log!("score: the walk gave up after {frames} update(s); nothing more is asked of it");
            return false;
        }
        *frames < COMMIT_FRAME_LIMIT
    };
    unsafe { runtime.game.show_ranking() };
    while !unsafe { runtime.game.showing_ranking() } && running(result, &mut frames) {}
    if !unsafe { runtime.game.showing_ranking() } {
        // The request undone as well: asking is a reservation the front end acts on later, so leaving
        // one behind is a ranking that comes up by itself — which is what a give-up landing on the
        // score screen was.
        unsafe { runtime.game.leave_ranking() };
        return log!("score: the ranking was not built after {frames} update(s); nothing written");
    }
    log!("score: the ranking is up — {}", unsafe {
        runtime.game.ranking_state()
    });
    unsafe { runtime.game.set_captures(&captures) };
    unsafe { runtime.game.leave_ranking() };
    log!("score: asked it to leave — {}", unsafe {
        runtime.game.ranking_state()
    });
    // Until the scene is no longer the ranking's, not until the screen's own state changes: the state
    // changes on the update it is asked, and the scene goes back several updates later. Stopping at the
    // first left those updates to be drawn — the ranking, visible for a second.
    while unsafe { runtime.game.ranking_scene() } && running(result, &mut frames) {}
    // A couple more so the front end is built before its cursor is put back, and then the cursor: the
    // one that comes back is on the item the game thinks was left, which is the ranking orb asked for.
    //
    // Both of them behind the same answer as the loops above, which is why this is not two bare calls: the
    // pair is worth having where the walk is still walking and is a walk of a chain being taken apart where
    // it is not.
    for _ in 0..2 {
        if !running(result, &mut frames) {
            break;
        }
    }
    unsafe { runtime.game.restore_menu_cursor() };
    log!(
        "score: the ranking built and taken down in {frames} update(s) — {}",
        unsafe { runtime.game.ranking_state() }
    );
}

/// Gives the run up, which is the retry menu's third choice: the game is put on its way to the
/// title menu, and reports whether it had a run to leave.
///
/// Nothing of orb's is dropped here. The snapshots describe a stage the game is about to tear
/// down, and what notices that is the run leaving `in_run` a frame or two later — the same path
/// that ends any other run, which is also what writes the line saying how many retries it took.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn give_up(game: &dyn Game) -> bool {
    if !unsafe { game.leave_run() } {
        log!("retry: there is no run to give up");
        return false;
    }
    // The key that answered orb's question is not one the game's own screens should act on.
    // Needed here where it is not needed after a retry: a retry puts the whole of `.data` back
    // from a snapshot, and this leaves the game to build the title menu — which reads the same
    // keyboard, and would take the `z` still held from this answer as an item chosen.
    unsafe { game.swallow_input() };
    log!("retry: the run is given up; the game is on its way to the title");
    true
}

/// The chapters the retry menu lists behind the one that was lost, in the order it lists them: the
/// ones the stage has a snapshot of, each by the name the status line calls it.
///
/// The chapter being played is left out, which is the first of what `offers` answers: that one is a way
/// on of its own and needs no place in the list behind it.
///
/// By number in a pass building the table, where every number on screen is one to hold against the
/// log — and where a name is settled by boundaries the pass is still judging, so it would move under
/// whoever is reading it.
fn offered(runtime: &Runtime) -> Vec<retry_ui::Chapter> {
    runtime
        .chapters
        .offers()
        .into_iter()
        .skip(1)
        .map(|offer| retry_ui::Chapter {
            at: offer.at,
            name: if runtime.config.chapter_tuning {
                format!("CHAPTER {}", offer.number)
            } else {
                offer.name.to_string()
            },
            stage_start: offer.stage_start,
        })
        .collect()
}

/// Replaces `th06::Chain::RunCalcChain`. `__thiscall` with a single argument is
/// `fastcall` with nothing on the stack, which is an ABI Rust can spell.
///
/// # Safety
/// Must run on the game's main thread, and `chain` must be the chain object the game calls this on:
/// it is handed to the game's own update, which reads the frame's jobs out of it.
pub unsafe extern "fastcall" fn run_calc_chain(chain: *mut c_void) -> i32 {
    if IN_HOOK.swap(true, Ordering::Relaxed) {
        note_reentry();
        return unsafe { call_original(&RUN_CALC_CHAIN, chain) };
    }
    let result = unsafe { on_update(chain) };
    IN_HOOK.store(false, Ordering::Relaxed);
    result
}

/// Once, so a hook that turns out to nest every frame does not fill the log.
///
/// **No e2e test reaches it**, and that is what the guard above is for rather than a gap: a laid-out game
/// calls this hook where the real one's patched prologue is reached, on one thread and never from inside
/// itself. What would nest it is the game's own code re-entering the chain walk, which is a thing the real
/// game does and this one is written not to.
fn note_reentry() {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        log!("hook re-entered from inside itself; the nested frame runs the game only");
    }
}

/// Replaces `th06::Chain::RunDrawChain`, so the overlay draws after the game's
/// own drawing and inside the same scene.
///
/// # Safety
/// As [`run_calc_chain`], and inside the scene the game draws into: what orb draws here goes through
/// the device it is already drawing with.
pub unsafe extern "fastcall" fn run_draw_chain(chain: *mut c_void) -> i32 {
    if IN_HOOK.swap(true, Ordering::Relaxed) {
        note_reentry();
        return unsafe { call_original(&RUN_DRAW_CHAIN, chain) };
    }
    unsafe { before_draw() };
    let result = unsafe { call_original(&RUN_DRAW_CHAIN, chain) };
    unsafe { after_draw() };
    IN_HOOK.store(false, Ordering::Relaxed);
    result
}

/// Replaces the game's window creation, to overrule its display setting first.
/// Borderless mode needs a window; a game that has taken the display exclusively
/// has none to resize, and by the time anything of ours runs per frame the device
/// already exists.
pub extern "C" fn create_game_window(instance: *mut c_void) {
    if FORCE_WINDOWED.swap(false, Ordering::Relaxed)
        && let Some(game) = unsafe { chosen() }
        && !game.windowed()
    {
        unsafe { game.force_windowed() };
        log!("borderless: overrode the game's fullscreen setting");
    }
    let original: extern "C" fn(*mut c_void) =
        unsafe { std::mem::transmute(CREATE_GAME_WINDOW.load(Ordering::Relaxed)) };
    original(instance)
}

/// Replaces `th06::GameWindow::Render` with orb's own frame: update first, then
/// draw, then present on the display's cadence.
///
/// The game's own order is draw-then-update, which puts everything on screen one
/// update behind the input that produced it. Doing the update first is the whole of
/// that fix; the pacing around it is what keeps the result smooth.
///
/// # Safety
/// Must run on the game's main thread, and `game_window` must be the object the game calls its own
/// whole frame on: the three ways out of this that return hand the frame back to that frame, which
/// reads it.
pub unsafe extern "fastcall" fn render(game_window: *mut c_void) -> i32 {
    let runtime = unsafe { RUNTIME.get() }.as_mut();
    let Some(runtime) = runtime else {
        return unsafe { call_render(game_window) };
    };
    let game = runtime.game;
    let device = unsafe { game.d3d_device() };
    // Nothing to pace or draw with; let the game have its own loop back.
    //
    // **A game orb has no device of at all is not that**, and its frame is still orb's to run: what a
    // device is wanted for here is the scene orb's overlay is drawn inside and the picture a `screenshot`
    // step copies, where the *pacing* is the compositor's and wants only the window. So the scene either
    // side of the draw chain is skipped and the four calls the loop is made of stay — see
    // `Game::draws_through_its_d3d_device`, and `game::th06c`, which draws through D3D11.
    let scene = !device.is_null();
    if !scene && game.draws_through_its_d3d_device() {
        return unsafe { call_render(game_window) };
    }
    // The pointer, above the check below rather than beside the keyboard's own read: whether the mouse
    // has moved is nothing to do with the game's update, and a frame that draws nothing is still a frame
    // somebody may be reaching for the mouse on.
    runtime.mouse.poll(unsafe { game.window() });

    // The game does nothing at all while its window is behind, and orb carries on: that is what
    // makes coming back to it instant instead of a stale frame, and what keeps a replay or a
    // stress run going while attention is elsewhere. The keys are dealt with in the input hook,
    // not by stopping. `always_draw: false` is for somebody who wants the game's own behaviour
    // back, and below is where they get it.
    let window = unsafe { game.window() };
    if !runtime.config.always_draw && orb_api::window::foreground() != window {
        // Paced even with nothing drawn. The wait this frame loop runs on is inside this
        // function, and the game's own loop calls it straight back, so a return that waits for
        // nothing spins a core for as long as the window stays behind. On the compositor's blanks
        // like any other frame — a window behind, one covered and one minimised all flush at the
        // compositor's own rate with every gap one refresh, which is measured rather than assumed.
        unsafe { pacing() }.wait_for_slot(window);
        return RENDER_KEEP_RUNNING;
    }

    let chain = game.chain();
    let (update, draw) = (
        RUN_CALC_CHAIN_TARGET.load(Ordering::Relaxed),
        RUN_DRAW_CHAIN_TARGET.load(Ordering::Relaxed),
    );
    // Calling a null function pointer is undefined, and the compiler turns it into
    // an instruction that only crashes. Handing the frame back is the honest answer.
    //
    // **The one way out of this loop no e2e test reaches, and it can have none**: `attach` and
    // `attach_to` both fill these two statics as they install, and nothing outside orb can empty
    // them — so a game would have to be laid out with a hole in orb rather than in itself. The other
    // three are the `frame_loop` section of `orb-e2e`'s `pacing`: the frame handed back for want of a
    // runtime or of a device, the chain's two exits becoming the frame's two, and the turn a frame
    // whose window is behind still takes.
    if update == 0 || draw == 0 {
        return unsafe { call_render(game_window) };
    }
    let update: extern "fastcall" fn(*mut c_void) -> i32 = unsafe { std::mem::transmute(update) };
    let draw: extern "fastcall" fn(*mut c_void) -> i32 = unsafe { std::mem::transmute(draw) };

    // Before the update as the game does it: it leaves the viewport covering the whole
    // output, and the update is what narrows it to the playfield for the drawing that
    // follows. Doing it after the update overwrites that and the bullets end up outside
    // the playfield.
    //
    // Before the wait, too, because it ends in a `Clear` — a driver call that blocks
    // for as long as the display still holds a buffer of ours. Waiting here costs
    // nothing, since waiting is what comes next anyway; on the far side of the wait it
    // would push the rest of the frame past a blank, and the frame after that would sit
    // out an extra one.
    let started = frame::now();
    unsafe { game.prepare_frame(device) };
    let cleared = frame::now();
    // The frame's turn. What follows runs in one go, so the input the update reads is
    // as recent as the frame it appears in.
    unsafe { pacing() }.wait_for_slot(window);
    let waited = frame::now();
    let updated = update(chain);
    let ran = frame::now();
    // Where the game's own loop makes it, and inside the span this frame has to reach its blank in. On
    // the frame a spell card starts, this call is most of that span — 8438µs of it on the run
    // [docs/adr/0011](../../../docs/adr/0011-the-frame-is-held-for-the-blank-before-the-one-it-is-aimed-at.md)
    // was measured on — so that frame is handed over after its blank has gone and loses a refresh.
    //
    // **Past `PRESENT.call()` was tried and buys nothing**, which is why the frame that loses the
    // refresh is this one and not the next. `orb-e2e`'s `pacing`'s `sound` section is the measurement,
    // and moving the call is how to take it again: the card's own frame then keeps its blank, and the
    // frame after it loses one instead, the sounds having spent the whole of the tail between this
    // handover and the blank it was aimed at. Which is arithmetic rather than a near miss — that tail
    // is `compose_us`, a couple of milliseconds against a sound of nine — so the next frame reaches
    // the flush past that blank, and a frame that arrives there has lost its refresh before doing
    // anything.
    //
    // It is the worse of the two, and not merely a wash: the miss lands on a frame whose own drawing
    // did not overrun, so `measure_compose` reads it as the compositor being short and climbs the
    // allowance — 2500µs to 2600µs per card, never shaved back, which is input lag for the rest of the
    // run. The sound also starts a frame later than the update that asked for it.
    unsafe { PLAY_SOUNDS.call() };
    // And what a restore in that update left waiting, now that the sound has run: the play it asked for was
    // a command in the game's own queue, and this is where that queue has been carried out — see
    // `snapshot::finish_the_restores_sound`. Before the two exits below, because a frame that is leaving is
    // still a frame whose restore has already happened.
    if let Some(game) = unsafe { chosen() } {
        unsafe { crate::snapshot::finish_the_restores_sound(game) };
    }
    if updated == CHAIN_EXIT_SUCCESS {
        return RENDER_EXIT_SUCCESS;
    }
    if updated == CHAIN_EXIT_ERROR {
        return RENDER_EXIT_ERROR;
    }
    let sounded = frame::now();

    let (drawn, held) = unsafe {
        // **The scene only where there is a device to open one on.** A game orb draws nothing in has no
        // scene of orb's around its draw chain — its own renderer opens whatever it draws inside — and
        // the same goes for the picture below and the texture unbound after it.
        if scene {
            d3d8::begin_scene(device);
        }
        // What the game's own frame does around its own draw chain, inside its own scene. Nothing at all
        // for th06, whose frame has nothing there; th07's queue of quads is emptied by the first and
        // drawn by the second, and a frame missing either is a frame that writes a quad through a null
        // pointer or draws one outside the scene it was queued in. See `Game::begin_drawing`.
        game.begin_drawing(device);
        draw(chain);
        // On the far side of `draw`, which is the hook: orb's own overlay is drawn inside that call, so
        // what a game queues and has drawn here is drawn *over* the overlay. Nothing today — th07 is
        // the game with a queue and it has no font beside its exe to build an overlay from — and the
        // day it has one this is the line that decides it: the flush moves into the draw hook, ahead of
        // `after_draw`, which puts it where the game's own frame has it relative to everything drawn
        // inside the chain. Left here until then, because that is where the game's frame has it relative
        // to the chain and nothing yet says which of the two the overlay wants.
        game.end_drawing(device);
        if scene {
            d3d8::end_scene(device);
            // Whatever asked for a picture of this frame, now that everything of the frame is in the
            // back buffer and before the swap makes what is in it undefined. Nothing on a frame nobody
            // asked on.
            crate::shot::taken(device);
            d3d8::set_texture(device, 0, None);
        }
        let drawn = frame::now();
        // Between the drawing and the handover, which is the one place the frame's own work is a
        // number rather than a prediction — see `Pacing::hold_for_the_blank_before` for what goes
        // wrong when the frame is handed over as early as the budget started it.
        pacing().hold_for_the_blank_before();
        let held = frame::now();
        PRESENT.call();
        (drawn, held)
    };
    unsafe { pacing() }.finished(frame::Marks {
        started,
        cleared,
        waited,
        updated: ran,
        sounded,
        drawn,
        held,
        presented: frame::now(),
    });
    RENDER_KEEP_RUNNING
}

unsafe fn call_render(window: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(RENDER.load(Ordering::Relaxed)) };
    original(window)
}

/// Replaces the game's device setup, to redirect the device's `Present` before
/// anything is presented through it. Runs again after every device reset.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// setup would have been reached, there being no prologue to patch — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md). A
/// laid-out game that had its device before orb was attached would never reach it at all, which is what
/// `Fake::attach_before_its_device` exists for.
pub extern "C" fn init_d3d_device() {
    if let Some(game) = unsafe { chosen() } {
        let device = unsafe { game.d3d_device() };
        if !device.is_null() {
            unsafe { crate::window::hook_device(device) };
        }
    }
    let original: extern "C" fn() =
        unsafe { std::mem::transmute(INIT_D3D_DEVICE.load(Ordering::Relaxed)) };
    original()
}

/// What the game sees on the keyboard this frame — nothing, while its window is not
/// the one in front, and the buttons written down for this frame while a run is being
/// played back into the chapter it was left in.
///
/// The keyboard is read globally, not per window, so a game that keeps updating in
/// the background would otherwise act on whatever is being typed elsewhere. Dropping
/// the buttons here rather than skipping the update is what lets a background window
/// go on being drawn.
///
/// This is also the one place every button the game acts on passes through, which is what makes it
/// where a run's own buttons are written down: against the frame of the stage about to run, since
/// the read happens before the counter moves. See [`resume`].
pub extern "system" fn get_input() -> u16 {
    let original: extern "system" fn() -> u16 =
        unsafe { std::mem::transmute(GET_INPUT.load(Ordering::Relaxed)) };
    // Nothing of orb's in a process the attach settled on no game in: every line below reads one, and
    // the game's own read is what this hook stands in front of. See [`GAME`].
    let Some(game) = (unsafe { chosen() }) else {
        return original();
    };
    // Which frame of the stage this read is for. Before anything else, and before the foreground
    // check: a run being played back into place is being played with the window wherever it is,
    // and the frames it runs through are not frames anybody is at the keyboard for.
    let frame = unsafe { game.stage_frame() };
    // Counted here because this is the one place every update of a run passes through, fed or played or typed:
    // what it is for is the frames a stage spends more than one update on — see [`UPDATES_THIS_FRAME`].
    updates_of(frame);
    if let Some(buttons) = frame.and_then(|frame| unsafe { resume::fed(frame, game.generator()) }) {
        // **The music held quiet on every frame that is fed**, wherever those frames come from: `play_in` runs
        // its whole loop inside one of the game's frames and every update in it comes through this read, and a
        // game whose frames orb does not drive feeds one per frame instead. Either way the song the stage's build
        // started would otherwise play its opening for the seconds the buttons take and then jump where it
        // belongs as the run lands — which is what a th07 session heard on a run picked up in the first stage.
        //
        // Stopped and not taken down: the landing puts the song back with a seek — see the `play_from` there —
        // and a stream that had been deleted is one nothing could seek. A stop on a stopped buffer costs the call
        // and nothing else, which is what makes doing it per frame the simple answer to a queued play carried out
        // in between.
        if let Some(music) = game.music() {
            unsafe { music.silence() };
        }
        // Through the same hand-over the keyboard's own word goes through: a run being played back into place
        // is a run whose frames have to land where they landed, and a frame played on last frame's buttons is
        // not that frame — see `Game::hand_the_read_to_the_run`. Which is also where the frame this read is
        // for matters most: a playback drives its updates in a burst, so the game skips the run on other
        // frames than it did while somebody was playing it.
        unsafe { game.hand_the_read_to_the_run(buttons, the_first_read_of(frame)) };
        return buttons;
    }
    // Asked of the system rather than read from the game's own `WM_ACTIVATEAPP`
    // flag, which only says what the game was last told; this is the same question
    // orb asks for its own keys, so the two cannot disagree.
    let window = unsafe { game.window() };
    let active = !window.is_null() && orb_api::window::foreground() == window;
    // **Said before the bot's word is handed over and not after it.** None of what the window is doing applies
    // to a bot — its word is not the keyboard's and goes in whichever window is in front, which is the block
    // below — but a launch that says nothing about the window is a launch nobody can ask. Somebody watching one
    // play itself in a corner of their screen saw its presses come and go with the focus, and the log had no
    // line either way: this one was written after that block, so a launch playing itself never reached it.
    if INPUT_ACTIVE.swap(active, Ordering::Relaxed) != active {
        log!(
            "input: window {}",
            if active {
                "in front"
            } else {
                "behind, keys not read"
            }
        );
    }
    // **And the word orb plays the game with where it is playing it itself**, in the same place and through the
    // same hand-over: a bot's frame is a frame of a run in every way a recorded frame is. After the playback
    // above, so that a run being put back is put back with what it was played with and not with what a bot
    // would do instead — see `Game::what_a_bot_sees` and `Config::autoplay`.
    // **Every read and not only a stage's**, because the steps of a recipe are screens: the mode question, the
    // title, the difficulty and the shot are all read through here and none of them is a frame of a stage.
    // Asked for where the frame began rather than here, so that a walker counting the frames it has been on a
    // step counts each of them once — this read happens more than once on some screens, and orb's own menus want
    // the same word out of [`pad`]. See [`what_is_reached`].
    // **Through [`noted`] like every other way out of this hook**, which is what writes a frame's buttons down
    // for the chapter it belongs to. A path of its own here is a run orb kept nothing of — and a run to pick up
    // out of a file is the one thing a bot exists to make: the first launch that played its way into stage 2 left
    // `resume: not written — 0 frame(s) of buttons` behind it, and there was nothing to measure.
    // **And through [`held_back`] as well, which is what lets a bot reach a question of orb's at all.** The
    // decide is taken out of the word on the reads a question is to be asked on, and the question is raised by
    // that read — so a word handed over in front of it is a press that reaches the item and never the question.
    // What that cost is four launches on the real game: every one of them pressed at the title menu's replay
    // item and every one opened the game's own replay list, `menu:` never once in the log, while the same press
    // made at a keyboard put orb's question up.
    if let Some(word) = pressing() {
        return unsafe { noted(game, frame, held_back(game, word)) };
    }

    // Not read at all while behind, rather than read and thrown away. The game's
    // keyboard is a foreground DirectInput device, so the system unacquires it the
    // moment the window goes behind and every read then fails. The game checks only
    // for `DIERR_INPUTLOST` and treats anything else as a success, so a failed read
    // hands it an uninitialised stack buffer as the key state.
    if !active {
        INPUT_LOST.store(true, Ordering::Relaxed);
        // Nothing is being held back on the frames the game is being given nothing, and what it was
        // holding is forgotten with them: a button let go of behind the window would otherwise still
        // read as held on the frame it comes back on, and a press then is a press swallowed rather than
        // held back. A button still down on that frame reads as a fresh press instead, which puts the
        // question up — the same trade the game makes across its own scene changes, and the harmless
        // way round. See [`held_back`].
        DECIDE_WAS_DOWN.store(false, Ordering::Relaxed);
        return unsafe { noted(game, frame, 0) };
    }
    // Back in front: get the device back before anyone reads it. Left to itself the
    // game would only try on the one `DIERR_INPUTLOST` that reports the loss, and
    // whether it ever sees that report depends on exactly when the frames fell —
    // which is not something to leave the whole keyboard resting on.
    if INPUT_LOST.load(Ordering::Relaxed) {
        if !unsafe { game.acquire_input() } {
            return unsafe { noted(game, frame, 0) };
        }
        INPUT_LOST.store(false, Ordering::Relaxed);
        log!("input: keyboard re-acquired");
    }

    // Timed because it is the largest single thing in a frame — most of a refresh at
    // 120Hz — and none of it is orb's.
    let started = profile::now();
    let buttons = original();
    unsafe { profile::record(profile::Phase::Input, started) };
    // The pads are inside that read rather than added to what it answered: `Controller::GetInput`
    // tail-calls the pad half, and orb stands in front of that half — see [`get_controller_input`]. So
    // the word arriving here already has every pad in it, and it has them before it is written down,
    // which is what makes a direction pushed on a pad a direction the run recorded.
    unsafe { noted(game, frame, held_back(game, buttons)) }
}

/// **The keyboard read of a game that looks its own up**, which the game itself put in place: orb
/// answered the lookup with this — see [`get_proc_address`], [`crate::game::Read::LookedUp`] and
/// [`GET_KEYBOARD_STATE`].
///
/// **And the frame's own moment, for a game orb has no frame loop in.** th06c reads the keyboard nowhere
/// else, so this is where a recipe's step is walked and where the word that step presses is put into the
/// snapshot the game is about to read. A game orb *is* inside the frame of walks its steps in
/// [`on_update`] instead, and its own read answers a word rather than filling a snapshot.
///
/// **It is not that game's tick, though, and nothing here makes it one.** The read is made more often
/// than the game moves — 66 times a second against a game that moves 60, measured over a whole stage
/// and written down in `docs/th06c/asm.md` — so a recipe's count of frames is that many reads and not
/// that many of the game's own, and the rules are asked for a word on readings that are the reading
/// before over again. What answers those is `orb_bot::steer::a_tick_read_again`, off the reading itself,
/// rather than a tick read here: nothing in this exe has been found that counts one.
/// # Safety
/// `state` must be the 256 bytes the host's own read is given, which is what the game passes.
pub unsafe extern "system" fn keyboard_state(state: *mut u8) -> i32 {
    let original: extern "system" fn(*mut u8) -> i32 =
        unsafe { std::mem::transmute(GET_KEYBOARD_STATE.load(Ordering::Relaxed)) };
    let answer = original(state);
    // Nothing of orb's in a process the attach settled on no game in, the same as [`get_input`].
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return answer;
    };
    // **The device taken away before the word is written**, for a game whose run reads one instead of
    // the keys: what `Game::acquire_input` answers here is that the game is to read its keyboard, which
    // is the only read orb is in front of. Nothing for a game that has no such device — see th06c's,
    // which is the measurement.
    unsafe { runtime.game.acquire_input() };
    // **And the word asked for here only where orb has no frame of its own.** This read is made more
    // often than such a game moves — 683 to 773 of them against every 600 walks of th06c's calc chain —
    // so where orb runs the frame the rules are asked there and what this read writes is the word they
    // left. See [`THE_FRAME_IS_THE_SEAM`].
    if !THE_FRAME_IS_THE_SEAM.load(Ordering::Relaxed) {
        unsafe { what_is_reached(runtime) };
    }
    let word = pressing();
    if let Some(word) = word {
        unsafe { runtime.game.press_into(state, word) };
    }
    // **And the search a launch that is looking runs** — see [`find`], and `Config::find`, which is off
    // in every other launch. Here rather than in the frame hook because this is the frame a game orb
    // stands in none of has: the word is the one just written into the snapshot, so the reading taken
    // next frame is the one this press moved.
    if runtime.config.find {
        unsafe { find::a_frame(&runtime.data, the_way(runtime.game, word)) };
    }
    // And the corner the window orb plays in is held in, for a game whose own window something else
    // moves — see [`crate::window::keeps_its_corner`], which does nothing in every other launch.
    unsafe { crate::window::keeps_its_corner() };
    answer
}

/// Whether orb's own frame is what the rules are asked from — see [`on_update`], which is the only
/// thing that sets it, and [`keyboard_state`], which stops asking where it is set.
static THE_FRAME_IS_THE_SEAM: AtomicBool = AtomicBool::new(false);

/// Which way what is being looked for is expected to be going, out of the word being pressed.
///
/// **Across and not down**, which is the axis a game clamps least: a field is wider than the player's
/// step on both, but the bottom of one is where the rules park a run and the top is where nothing goes.
fn the_way(game: &dyn Game, word: Option<u16>) -> find::Way {
    let Some(word) = word else {
        return find::Way::Still;
    };
    let bit = |name| game.button_bit(name).unwrap_or(0);
    match (word & bit("right") != 0, word & bit("left") != 0) {
        (true, false) => find::Way::Up,
        (false, true) => find::Way::Down,
        _ => find::Way::Still,
    }
}

/// **The pad orb is, for a game that reads one rather than the keys.**
///
/// `DWORD XInputGetState(DWORD dwUserIndex, XINPUT_STATE *pState)`, answered for the first pad and
/// refused for the rest: what the game finds is one pad, always there, pressing whatever the run is
/// being played with.
///
/// **Answered and not called through, which is the point of it.** A real pad on this machine is
/// somebody's and is not playing this run — and what a run needs is a read that works on every screen,
/// where this game's keyboard works only on some. See `game::Hooks::reads_a_pad`.
///
/// # Safety
/// `state` must be what `XInputGetState` takes: the bytes of an `XINPUT_STATE`.
pub unsafe extern "system" fn xinput_get_state(user: u32, state: *mut u8) -> u32 {
    /// `ERROR_DEVICE_NOT_CONNECTED`, which is what a pad that is not there answers.
    const NOT_CONNECTED: u32 = 1167;
    if user != 0 || state.is_null() {
        return NOT_CONNECTED;
    }
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return NOT_CONNECTED;
    };
    unsafe { runtime.game.press_into_pad(state, pressing().unwrap_or(0)) };
    0
}

/// **The names orb answers a lookup for in the host's place**, set at the attach out of `game::Hooks` —
/// the game's own keyboard read, the pad it reads, and the making and moving of the window a launch that
/// plays itself puts in a corner. See [`crate::window::create_window_ex_w`] and
/// [`crate::window::move_window`].
static LOOKED_UP: Mutex<Lookups> = Mutex::new(Lookups {
    read: None,
    pad: None,
    window_made: None,
    window_moved: None,
    millis: None,
    ticks: None,
    counter: None,
    renderer: None,
});

/// **The names a game looks up that orb answers**, out of `game::Hooks`.
///
/// A field apiece and not a list, because each has an answer of its own and nothing chooses between
/// them: what would be in a list is the pair of a name and a function, and the function is this crate's
/// to know.
#[derive(Default, Clone, Copy)]
pub struct Lookups {
    pub read: Option<&'static str>,
    pub pad: Option<&'static str>,
    pub window_made: Option<&'static str>,
    pub window_moved: Option<&'static str>,
    /// **The three clocks a game paces itself by**, answered running faster than the host's where the
    /// launch asked for the game to — see [`crate::faster`], which has why all three go together and
    /// why the frequency beside them does not. Nothing in an ordinary launch: a clock that answers what
    /// the host answers is a name there is no reason to stand in front of.
    pub millis: Option<&'static str>,
    pub ticks: Option<&'static str>,
    pub counter: Option<&'static str>,
    /// **The name a DXGI renderer is made through**, answered so that the swapchain the game makes is
    /// one orb can present through — see [`crate::dxgi`], which has why it is that name and not the two
    /// older ones beside it. Nothing for a game that draws some other way, and nothing for one whose
    /// frame is its own: the present is only orb's business where the frame is.
    pub renderer: Option<&'static str>,
}

/// Says which names those are, or takes them away.
///
/// # Safety
/// Must run at the attach, before the game has been asked for a frame.
pub unsafe fn answers_the_lookup_of(names: Lookups) {
    *LOOKED_UP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = names;
}

/// **The game's own `GetProcAddress`, answered with orb's read where the name is the one orb wants.**
///
/// This is what stands in front of a read the game looks up rather than imports: the slot the answer
/// lands in is the game's own to write, so writing there at the attach is writing what the game's own
/// startup overwrites — measured, th06c's keyboard slot held `0x0` when orb read it. Answered here, the
/// pointer the game keeps is orb's because the game put it there, and no address of the game's is needed
/// for it at all.
///
/// **Called through first**, always: what the host answers is what orb calls through to, and every name
/// but the one is handed back exactly as it came.
///
/// # Safety
/// `name` must be what `GetProcAddress` takes: a null-terminated string, or an ordinal in the pointer
/// itself.
pub unsafe extern "system" fn get_proc_address(module: usize, name: *const u8) -> usize {
    let original: extern "system" fn(usize, *const u8) -> usize =
        unsafe { std::mem::transmute(GET_PROC_ADDRESS.load(Ordering::Relaxed)) };
    let answer = original(module, name);
    // **An ordinal is not a name.** `GetProcAddress` takes either, an ordinal being a small integer in
    // the pointer itself rather than a string to read, so a lookup by ordinal has nothing to compare and
    // is one orb answers for never.
    if (name as usize) >> 16 == 0 {
        return answer;
    }
    let wanted = *LOOKED_UP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // **Every answer is orb's own function, and every one of them keeps the host's**, which is what
    // `orb::lookup` exists for: what the game stores is orb's pointer because the game stored it, and
    // what orb calls through to is what the host would have given it.
    //
    // Through a pointer and not straight to an integer, a cast of a function item being a lint of its
    // own — the same spelling `orb::crash` takes orb's own address with.
    if let Some(read) = wanted.read
        && unsafe { asked_for(name, read) }
    {
        GET_KEYBOARD_STATE.store(answer, Ordering::Relaxed);
        log!("hook: the game asked for {read} and got orb's, the host's being at {answer:#x}");
        return keyboard_state as *const () as usize;
    }
    if let Some(pad) = wanted.pad
        && unsafe { asked_for(name, pad) }
    {
        log!(
            "hook: the game asked for {pad} and got orb's own pad, the host's being at {answer:#x}"
        );
        return xinput_get_state as *const () as usize;
    }
    if let Some(made) = wanted.window_made
        && unsafe { asked_for(name, made) }
    {
        unsafe { crate::window::answers_the_window_made(answer, playing_it()) };
        log!("hook: the game asked for {made} and got orb's, the host's being at {answer:#x}");
        return crate::window::create_window_ex_w as *const () as usize;
    }
    if let Some(moved) = wanted.window_moved
        && unsafe { asked_for(name, moved) }
    {
        unsafe { crate::window::answers_the_window_moved(answer) };
        log!("hook: the game asked for {moved} and got orb's, the host's being at {answer:#x}");
        return crate::window::move_window as *const () as usize;
    }
    // **And the clocks, for a launch asked to run the game faster than it runs itself** — see
    // [`crate::faster`]. Which of the three a game paces by is the game's own business and this says
    // which ones it asked for: th06c holds the strings of all four and orb answers whichever it is asked
    // for, the frequency among them left to the host.
    if let Some(millis) = wanted.millis
        && unsafe { asked_for(name, millis) }
    {
        unsafe { crate::faster::the_millis_are(answer) };
        log!(
            "hook: the game asked for {millis} and got orb's faster one, the host's being at {answer:#x}"
        );
        return crate::faster::millis as *const () as usize;
    }
    if let Some(ticks) = wanted.ticks
        && unsafe { asked_for(name, ticks) }
    {
        unsafe { crate::faster::the_ticks_are(answer) };
        log!(
            "hook: the game asked for {ticks} and got orb's faster one, the host's being at {answer:#x}"
        );
        return crate::faster::ticks as *const () as usize;
    }
    if let Some(counter) = wanted.counter
        && unsafe { asked_for(name, counter) }
    {
        unsafe { crate::faster::the_counter_is(answer) };
        log!(
            "hook: the game asked for {counter} and got orb's faster one, the host's being at {answer:#x}"
        );
        return crate::faster::counter as *const () as usize;
    }
    // **And the renderer, for the game whose frame orb hands over itself** — the factory this answers is
    // the host's own with the swapchains it makes taken on the way past, which is what puts the present
    // in orb's hands. See [`crate::dxgi`].
    if let Some(renderer) = wanted.renderer
        && unsafe { asked_for(name, renderer) }
    {
        unsafe { crate::dxgi::answers_the_factory(answer) };
        log!("hook: the game asked for {renderer} and got orb's, the host's being at {answer:#x}");
        return crate::dxgi::creates_a_factory as *const () as usize;
    }
    answer
}

/// Whether a launch is one orb plays the game in, which is what decides whether the window it is about
/// to make goes in a corner — asked of the bot's own slot rather than of the config, that slot being
/// what `Config::autoplay` installed.
fn playing_it() -> bool {
    PLAYING_IT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some()
}

/// Whether the name a lookup asked for is this one, read off the `char *` the host was handed.
///
/// Compared byte by byte rather than copied into a `String`: this runs on every lookup the game's own
/// startup makes — three hundred and forty-four of them in th06c — and what is being asked of each is
/// whether it is the one name orb answers for.
///
/// # Safety
/// `name` must be a null-terminated string.
unsafe fn asked_for(name: *const u8, wanted: &str) -> bool {
    for (at, byte) in wanted.as_bytes().iter().enumerate() {
        if unsafe { name.add(at).read() } != *byte {
            return false;
        }
    }
    // And ends there, or it is a longer name this one is the start of.
    unsafe { name.add(wanted.len()).read() == 0 }
}

/// The word the game reads, with the front end's own decide taken out of it while orb has a question
/// to ask on that press, and put back for the one read that hands the press over.
///
/// Done to the word the read hands back rather than to `g_CurFrameInput` afterwards, because that is
/// what `Supervisor::OnUpdate` assigns from and what every `WAS_PRESSED` in the frame is worked out
/// against — see [`Game::swallow_input`]. A press taken out here is one no screen in that frame ever
/// saw, so the screen orb asked over is still standing, on the item it was asked about, and answering
/// "neither" is nothing at all rather than a scene put back by hand.
///
/// The edge is orb's own to keep, for the same reason: the game's copy of the frame before has the
/// bits missing too, so it would call a press that has been held back for ten frames a fresh one on
/// the frame the holding stopped.
///
/// And with nothing new in it at all on the first read after a question came down, which is what
/// [`SETTLE_KEYS`] asks for.
fn held_back(game: &dyn Game, buttons: u16) -> u16 {
    let decide = game.menu_decide();
    let cancel = game.menu_cancel();

    // The key a question was cancelled with, until it is let go: it is still down on the frame the
    // game carries on into, and going back is what the screen underneath does with it.
    //
    // Or until the screen it was cancelled on is gone, which is the same holding as the decide's. A
    // hold that outlived it would take the bomb and the pause button out of a run: answer a question
    // with the cancel key still down — a run is started with the other hand — and every read of the
    // stage would be short of them until it was let go, including the reads a resumed run writes down
    // as its own.
    let buttons = if !HOLD_CANCEL.load(Ordering::Relaxed) {
        buttons
    } else if buttons & cancel == 0 || !HOLD_DECIDE.load(Ordering::Relaxed) {
        HOLD_CANCEL.store(false, Ordering::Relaxed);
        buttons
    } else {
        buttons & !cancel
    };

    // And every other key that was pressed while the question was up, on the one read the game
    // carries on with. Its own idea of the frame before is the frame the question went up on — no
    // frame of one runs the chain, so nothing assigns `g_LastFrameInput` — so a key held on orb's
    // menu is a key the screen underneath reads as a fresh press. The directions are what that costs:
    // orb's menus and the game's read the same ones, and both of these screens move their cursor
    // before they read their decide, so the item handed the press would not be the item asked about.
    //
    // The decide is left in, being the one bit whose edge orb keeps itself: masked out here it would
    // read as let go, and the read after as pressed again — which is the question coming straight back
    // up on the frame it was cancelled.
    let buttons = if SETTLE_KEYS.swap(false, Ordering::Relaxed) {
        buttons & (LAST_WORD.load(Ordering::Relaxed) | decide)
    } else {
        buttons
    };

    let word = if FEED_DECIDE.swap(false, Ordering::Relaxed) {
        buttons | decide
    } else if !HOLD_DECIDE.load(Ordering::Relaxed) {
        // Written down on these reads too, so that what is already down as the holding starts is not
        // taken for a press made under it. It is the opposite: the press that reached the screen before
        // is the one that got the game *to* this screen, and the key is still down on its first frames.
        // Read as a press, that put the question up before anybody had asked for a run.
        DECIDE_WAS_DOWN.store(buttons & decide != 0, Ordering::Relaxed);
        buttons
    } else {
        let down = buttons & decide != 0;
        let was = DECIDE_WAS_DOWN.swap(down, Ordering::Relaxed);
        if down && !was {
            DECIDE_PRESSED.store(true, Ordering::Relaxed);
        }
        buttons & !decide
    };
    LAST_WORD.store(word, Ordering::Relaxed);
    word
}

/// Writes down what the update about to run will act on, and hands it back to the game.
///
/// The frames the window was behind are written down as well, as the nothing the game is given
/// there: what has to be reproduced is what the update saw, and a run left with the window behind
/// has frames of exactly that.
///
/// # Safety
/// Only ever called from the input hook, on the game's main thread.
unsafe fn noted(game: &dyn Game, frame: Option<u32>, buttons: u16) -> u16 {
    trace::the_word(buttons);
    if let Some(frame) = frame {
        let word = buttons & game.run_input();
        let generator = unsafe { game.generator() };
        unsafe {
            resume::noted(frame, word, generator);
            // And into the run's own replay, which keeps the whole of it rather than the stage the chapter
            // is in — see [`replay`]. The same word: what a replay of this run plays back is what the run
            // was played with, and that is what the update is about to act on.
            replay::noted(frame, word, generator);
        }
    }
    // And handed to the run here, which is the read's own moment: a game whose own hand-over comes after the
    // update that acts on it is a game played a frame late — see `Game::hand_the_read_to_the_run`. Every way
    // out of this hook comes through here, so what the run is given is what the game was given.
    unsafe { game.hand_the_read_to_the_run(buttons, the_first_read_of(frame)) };
    buttons
}

/// Whether this read is the first one of a frame of the run, which is the half of the hand-over that a run
/// reads its edges off — see [`Game::hand_the_read_to_the_run`], where what it costs to get wrong is written.
///
/// Every read outside a stage is one of its own: what a frame of the front end hands the run is read by
/// nothing, and holding two of them together would be an edge kept across a run being started.
fn the_first_read_of(frame: Option<u32>) -> bool {
    let Some(frame) = frame else {
        HANDED_OVER_FOR.store(NO_FRAME, Ordering::Relaxed);
        return true;
    };
    HANDED_OVER_FOR.swap(i64::from(frame), Ordering::Relaxed) != i64::from(frame)
}

/// Runs where the game's own streaming thread is about to put one chunk of the track's file into the music
/// buffer, and says what that chunk is: where in the file it starts, where in the buffer it goes, and how much
/// of the track is left before the loop.
///
/// **This is the one place a sound becomes audible, which is why the line is said here and not from the
/// frame.** A track looping over a section of itself changes nothing a reading from outside can see — the same
/// track is playing, its buffer is being fed, and the position is wherever the last refill left it — so a log
/// written from the frame says a run's music is fine while a few seconds of the song go round and round. Read
/// here, that is the offsets going round a bounded range instead of climbing, and a launch can be held to it.
///
/// At the verbose level: a refill is a chunk of a second, so this is tens of lines a second while music plays.
///
/// **Not the frame's thread**, which is the whole of what this may do: it reads the stream it was handed
/// through [`Game::the_refill_is_about_to`] and says a line. Nothing of `Runtime` is touched — the frame is
/// running beside it — and the log takes lines from any thread, counting what they cost per thread.
#[cfg(target_arch = "x86")]
pub extern "thiscall" fn tops_up_the_music(stream: usize, argument: i32) -> i32 {
    let original: unsafe extern "thiscall" fn(usize, i32) -> i32 =
        unsafe { std::mem::transmute(TOPS_UP_THE_MUSIC.load(Ordering::Relaxed)) };
    // **The level is asked before the stream is read**, which `detail!` on its own would leave until after:
    // every field of that reading is a `VirtualQuery` — see `orb_api::mem::read_committed` — and the line is
    // an allocation, on a thread woken tens of times a second. Asked first, a launch that is not being read
    // pays an atomic load and the call below.
    if crate::log::wanted(crate::log::VERBOSE)
        && let Some(game) = unsafe { chosen() }
        && let Some(about_to) = unsafe { game.the_refill_is_about_to(stream) }
    {
        detail!("sound: the refill {about_to}");
    }
    unsafe { original(stream, argument) }
}

/// Runs where the game's own stop of the music is about to happen, and says what is being stopped.
///
/// **The other half of where a sound is made**, the refill being the first: a stop is what makes a stream that
/// was being fed silent, and it leaves nothing behind for a reading from the frame to find — the buffer is
/// simply stopped by the time anything asks, with no when and no whose ask. Three different asks silenced a
/// launch this session and none of them said so: the pause menu's, a frame that was being fed a playback's
/// buttons, and a restore.
///
/// **Whoever asked for it, orb included.** Its own calls of the game's stop come through here as well — the
/// take-down a restore makes, and `Game::stop_music` — and what tells those apart from the game's own is orb's
/// line beside this one. A stop with nothing beside it is the game's.
///
/// At the level every line is written at: a run stops its music a handful of times, and this is the line that
/// says which of them the silence began at.
///
/// # Safety
/// Only ever called from the hook over the game's own stop, on the thread the game calls it from.
#[cfg(target_arch = "x86")]
pub extern "thiscall" fn stops_the_music(this: usize) -> i32 {
    let original: unsafe extern "thiscall" fn(usize) -> i32 =
        unsafe { std::mem::transmute(STOPS_THE_MUSIC.load(Ordering::Relaxed)) };
    if let Some(game) = unsafe { chosen() } {
        log!("sound: stopped through the game — {}", unsafe {
            crate::audio::what_is_heard(game)
        });
    }
    unsafe { original(this) }
}

/// Runs after `th06::GameManager::AddedCallback`, where the game has just put a stage's numbers in
/// place and that stage has not been updated yet.
///
/// Nothing of `Runtime` is touched here: this is called from inside the game's own update, where
/// the frame hook is already holding it. What it needs to know — whether this run is being kept —
/// is [`resume`]'s own flag.
pub extern "C" fn stage_begun(manager: *mut c_void) -> i32 {
    let original: extern "C" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(STAGE_BEGUN.load(Ordering::Relaxed)) };
    let result = original(manager);
    // Its own answer first: anything but nothing is a stage it could not build, and the game is
    // on its way back to its menu with no stage for any of this to be about.
    if result == 0
        && let Some(game) = unsafe { chosen() }
    {
        unsafe { resume::stage_begun(game) };
        // After it, because a resumed stage's numbers go back in there and what a replay writes down is the
        // numbers the stage is going to be played with — see [`replay::stage_begun`].
        unsafe { replay::stage_begun(game) };
    }
    result
}

/// Runs around the callback that takes what the front end offers out of the score file, which is the
/// one read of that file whose answer is the game's own whichever mode is on.
///
/// Bracketed rather than the mode's file being chosen once: everything else the file holds is a
/// record of runs played one way or the other, and what the front end offers is not a record —
/// see [`score`].
///
/// **And what that read brings back of a record is read again once it is over**, out of the file the mode
/// points at: one read cannot land in two files, so the game's own stands and the mode's own goes over the
/// half of it that is a record. Here rather than only where the mode is chosen, because a screen brought up
/// again asks nothing — a run given up back to the practice stage select showed a legacy score for the stage
/// it had just been played in.
pub extern "C" fn unlocks_read(menu: *mut c_void) -> i32 {
    let original: extern "C" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(UNLOCKS_READ.load(Ordering::Relaxed)) };
    score::reading_unlocks(true);
    let result = original(menu);
    score::reading_unlocks(false);
    unsafe { the_modes_own_records_again() };
    result
}

/// Reads what the mode's own file records over the half of the front end's read that is a record of runs —
/// see [`Game::read_what_the_modes_file_records`], and [`unlocks_read`] for why it is here.
///
/// # Safety
/// Must run on the game's main thread, between frames, and after that read rather than inside it: the read
/// this makes is the mode's own file, which is the bracket being over.
unsafe fn the_modes_own_records_again() {
    let Some(game) = (unsafe { chosen() }) else {
        return;
    };
    unsafe { game.read_what_the_modes_file_records() };
    // **And what a run has counted back on top of it.** That read replaces the card records wholesale out of the
    // mode's own file, so a run still playing has its counting taken out of memory here by a read of *orb's*
    // rather than the game's — the same loss [`card_records_read`] puts back after the game's own read, and the
    // reason a run left through the game's pause menu reached its file with nothing of itself in it.
    //
    // The note is not let go of: nothing is written here, and what lets go of it is the save.
    //
    // **And the read above happens before this lock is taken, which is not a tidiness.** On a real game that
    // read enters [`card_records_read`], which locks this same mutex, and a `std::sync::Mutex` held across it
    // would be the game stopped dead. A laid-out game would not show it: its read does not go through the hook.
    let waiting = COUNTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(session) = waiting.as_deref() {
        unsafe { game.put_back_the_cards_a_run_left(session) };
    }
}

/// Runs before the callback that reads a ranking out of the score file, which is where the record of
/// captures that read is about to fill is emptied. See [`Game::forget_captures`] for why it has to be
/// emptied at all, and why here.
///
/// **Two jobs on two conditions**, which is why neither is decided by this hook being installed: the emptying
/// is only where there are two files to keep apart, so it asks the fork rather than the launch; and the arm
/// below is every launch's, a session's counts belonging in whichever file the mode points at.
///
/// # Safety
/// Must run on the game's main thread, and `screen` must be the ranking screen's own object as the
/// callback this replaces is handed it: the state of that screen is read through it, and it is what
/// decides whether the record in memory is that read's to fill.
pub unsafe extern "C" fn ranking_read(screen: *mut c_void) -> i32 {
    let original: extern "C" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(RANKING_READ.load(Ordering::Relaxed)) };
    if let Some(game) = unsafe { chosen() }.filter(|_| EMPTIES_CAPTURES.load(Ordering::Relaxed)) {
        unsafe { game.forget_captures(screen) };
    }
    let armed = unsafe { a_write_follows_this_read(screen) };
    let answered = original(screen);
    if armed {
        A_WRITE_FOLLOWS.store(false, Ordering::Relaxed);
    }
    answered
}

/// Arms the put-back where this read of the file is one a write follows, and answers whether it did — see
/// [`card_records_read`], which is called from inside the read this brackets.
///
/// Here rather than in that hook because what says a write follows is the object the *read* was made on, which
/// the parse inside it is handed nothing of.
///
/// # Safety
/// Must run on the game's main thread, with `screen` the object the game's own read was entered with.
unsafe fn a_write_follows_this_read(screen: *mut c_void) -> bool {
    let armed = unsafe { chosen() }
        .is_some_and(|game| unsafe { game.its_records_are_about_to_be_written(screen) });
    if armed {
        A_WRITE_FOLLOWS.store(true, Ordering::Relaxed);
    }
    armed
}

/// Runs where the game lays the records its file holds over the ones in memory, **after** that parse: what
/// stands then is the file's own, and what a run counted goes back over them — see `Hooks::card_records_read`
/// and `Game::put_back_the_cards_a_run_left`.
///
/// **At every one of those reads, and let go of at one**: the game reads its file for the front end's own menus
/// (0x45bff3), inside every save (0x44a059) and at a run's build (0x42e735), and each of them leaves the records
/// in memory as the file has them. Which is what somebody looking at the ranking screen sees — that screen walks the
/// records in memory into itself (0x44a145) — so a session's counts belong back in memory at all of them. The
/// note is only let go of where a write follows, [`A_WRITE_FOLLOWS`] saying which: let go of at a read that
/// writes nothing, what the run counted would be in memory and nowhere else, and gone at the next read.
///
/// # Safety
/// Must run on the game's main thread. `image` and `records` are the two the parse is entered with, and are
/// passed through untouched.
pub unsafe extern "fastcall" fn card_records_read(image: usize, records: usize) -> i32 {
    let original: unsafe extern "fastcall" fn(usize, usize) -> i32 =
        unsafe { std::mem::transmute(CARD_RECORDS_READ.load(Ordering::Relaxed)) };
    // **And the note is taken here, before the parse, where a run was playing on the last frame orb saw.**
    // This read is the last moment that run's own counting is in memory: the parse below lays the file's
    // records over it. The frame hook asking once it sees the run has ended is too late for the way out
    // through the game's own pause menu — th07 builds its front end inside that transition, so the unlocks
    // read its build makes runs while `in_run` still answers true, and what the hook would note is the file's
    // own records. A run whose two attempts at stage 1's cards reached the file as `put back over 0 the
    // file's own differ in` is what says so.
    if let Some(game) = (unsafe { chosen() }).filter(|_| A_RUN_WAS_PLAYING.load(Ordering::Relaxed))
    {
        unsafe { note_the_records_as_they_stand(game) };
    }
    let answered = unsafe { original(image, records) };
    let mut waiting = COUNTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(session) = waiting.as_deref() else {
        return answered;
    };
    let Some(game) = (unsafe { chosen() }) else {
        return answered;
    };
    unsafe { game.put_back_the_cards_a_run_left(session) };
    if A_WRITE_FOLLOWS.load(Ordering::Relaxed) {
        log!("score: and the save after it is what writes them down");
        *waiting = None;
    }
    answered
}

/// Writes down what the run now ending counted, for the save that will carry it — see [`COUNTED`].
///
/// **On the frame the run ends and not where the save is made**, which is what makes it its own step: the game
/// empties every card record as it builds a run and the front end coming up over one reads the file's own back,
/// so the frames between are frames in which what that run counted stops being in memory. Both ways out of a run
/// reach it — orb's own menu, where the run is still standing, and the game's own quit.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn note_what_the_run_counted(game: &dyn game::Game) {
    let mut held = COUNTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // **A note already held is left alone**, because this is asked once the run is over and by then the records
    // may be the file's own: th07's front end is registered inside the transition a run is left through, so its
    // unlocks read lands before orb sees the run end and
    // [`note_the_records_as_they_stand`] is what has the run's numbers. Overwriting here is what put a
    // run's counting nowhere at all.
    if held.is_some() {
        return;
    }
    let counted = unsafe { game.captures() };
    if counted.is_empty() {
        return;
    }
    *held = Some(counted);
}

/// And what the game's records hold **as they stand**, which where a read is about to lay the file's own over
/// them is the last moment a run's counting is anywhere.
///
/// **Replaces whatever is held rather than deferring to it**, which is the difference from
/// [`note_what_the_run_counted`]: the reads this is called at come in an order, and the last of them while a run
/// is playing is the one worth keeping.
///
/// **And nothing is noted where the records hold no count**, [`Game::these_records_hold_a_count`] being what
/// answers that: a run's own build reads the file at the moment it has emptied every record, and a note of no
/// counts put back over the file's own would be the file emptied by the save after it. The question is the
/// game's because an emptied record is not a zeroed one — it keeps its header and its card number, so a scan
/// for zero bytes answers that an emptied array is full, which is a guard that cannot fire.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn note_the_records_as_they_stand(game: &dyn game::Game) {
    let counted = unsafe { game.captures() };
    if !game.these_records_hold_a_count(&counted) {
        return;
    }
    log!("score: what a run has counted taken before this read lays the file's own over it");
    *COUNTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(counted);
}

/// What a run counted, waiting for the save that will write it.
///
/// **Read where the run ended rather than where the save is made**, which is the whole reason it waits here: the
/// records are emptied for every run the game builds and the front end's own read lays the file's over them, so
/// the frames between a run ending and orb asking for the save are frames in which what that run counted stops
/// being in memory at all.
static COUNTED: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

/// Whether the read of the file now running is one a write follows — see [`card_records_read`], which puts the
/// records back at no other. Set around the game's own read where its object says so, and false everywhere else:
/// that read lays the file's records over the ones in memory whether a write follows or not, and a put-back at
/// one that writes nothing is records stranded in memory with the note spent.
static A_WRITE_FOLLOWS: AtomicBool = AtomicBool::new(false);

/// Whether the last frame orb saw had a run of somebody's in progress, which is what says the card records in
/// memory at a read of the file are that run's own — see [`card_records_read`], which is where it is read.
///
/// A static beside the two above rather than [`Runtime::previous`], which the parse hook has no way to reach.
static A_RUN_WAS_PLAYING: AtomicBool = AtomicBool::new(false);

/// The same read where the game's own callback takes the screen in `ecx` — th07's, at 0x449b05, its chain
/// calling every callback that way. See [`game::Entered`].
///
/// # Safety
/// The same as [`ranking_read`]: the game's main thread, and `screen` the ranking screen's own object as the
/// callback this replaces is handed it.
pub unsafe extern "fastcall" fn ranking_read_in_ecx(screen: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(RANKING_READ.load(Ordering::Relaxed)) };
    if let Some(game) = unsafe { chosen() }.filter(|_| EMPTIES_CAPTURES.load(Ordering::Relaxed)) {
        unsafe { game.forget_captures(screen) };
    }
    let armed = unsafe { a_write_follows_this_read(screen) };
    let answered = original(screen);
    if armed {
        A_WRITE_FOLLOWS.store(false, Ordering::Relaxed);
    }
    answered
}

/// The same bracket where the game's own callback takes the front end in `ecx` — th07's, at 0x45c4c8, its
/// chain calling every callback that way. See [`game::Entered`].
pub extern "fastcall" fn unlocks_read_in_ecx(menu: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(UNLOCKS_READ.load(Ordering::Relaxed)) };
    score::reading_unlocks(true);
    let result = original(menu);
    score::reading_unlocks(false);
    unsafe { the_modes_own_records_again() };
    result
}

/// The same moment where the game's own callback takes the manager in `ecx` — th07's added callback, at
/// 0x42e83e, its chain calling every callback as `mov ecx,[elem+0x1c]; call [elem+0x4]`. See
/// [`game::Entered`], and [`get_controller_input_in_ecx`] for why this is an entry point of its own rather
/// than one function told which.
pub extern "fastcall" fn stage_begun_in_ecx(manager: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(STAGE_BEGUN.load(Ordering::Relaxed)) };
    let result = original(manager);
    if result == 0
        && let Some(game) = unsafe { chosen() }
    {
        unsafe { resume::stage_begun(game) };
        unsafe { replay::stage_begun(game) };
    }
    result
}

/// Runs before `th06::Stage::RegisterChain`, where the stage's numbers are in place and the stage
/// itself is about to be built out of them — including out of the generator, which is why a resumed
/// run's seed goes in here and nowhere else.
pub extern "C" fn stage_building(stage: i32) -> i32 {
    let original: extern "C" fn(i32) -> i32 =
        unsafe { std::mem::transmute(STAGE_BUILDING.load(Ordering::Relaxed)) };
    if let Some(game) = unsafe { chosen() } {
        unsafe { resume::stage_building(game) };
    }
    original(stage)
}

/// **The pad half of the input read, which is orb's and not the game's.** A tail call inside the keyboard
/// half, so it cannot be told apart from outside it — and it is not called through: what the buttons and
/// the axes of every pad mean to this frame is [`Game::pad_word`], for the device the game holds as much
/// as for the pads it has none of.
///
/// Replacing it rather than adding to what it answered is what puts one thing in one place, and one thing
/// is why: where the mapping puts focus on the shot button, the game holds the player still off a count of
/// the frames that button has been held, and the count is fed by whichever pad the read looked at. Fed
/// from the game's read of its own device, a pad orb read for itself never reaches it — and orb cannot
/// raise it from outside either, that read having already brought it down for this frame.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// keyboard read tail-calls that function, there being no prologue to patch — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
pub extern "C" fn get_controller_input(buttons: u32) -> u16 {
    with_the_pad_added(buttons as u16)
}

/// The game's own read of the pad's buttons by number, with the pads orb read for itself added to what it
/// found: a key config screen reads that array to learn which button was pressed, so a pad the game has no
/// device for is one nothing could be mapped from.
///
/// **Called through, unlike the pad word above**: what the game's own device says is already right, and
/// what is missing is only the pads it has none of — see [`Game::add_pad_buttons`]. The cost of the call
/// through is nothing to a run: one screen reaches it, and never per frame.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// screen calls that function.
pub extern "C" fn pad_buttons() -> *mut c_void {
    let original: extern "C" fn() -> *mut c_void =
        unsafe { std::mem::transmute(PAD_BUTTONS.load(Ordering::Relaxed)) };
    let buttons = original();
    if let Some(game) = unsafe { chosen() } {
        unsafe { game.add_pad_buttons(buttons, joystick::reading()) };
    }
    buttons
}

/// The same read where the game takes that word in `ecx` — th07's, at 0x4303f0. See
/// [`game::Entered`]: which of the two a game has is what the seam says, and this is the other entry
/// point it says it with.
///
/// An entry point apiece rather than one function told which, because a hook is entered by its ABI: what
/// installs one writes a jump over a prologue, and the convention is settled by then.
pub extern "fastcall" fn get_controller_input_in_ecx(buttons: u16) -> u16 {
    with_the_pad_added(buttons)
}

fn with_the_pad_added(buttons: u16) -> u16 {
    // Nothing of orb's in a process the attach settled on no game in, the same as [`get_input`]: what a
    // pad means is a `Game`'s answer, and the keyboard's word is the whole of the read without one.
    let Some(game) = (unsafe { chosen() }) else {
        return buttons;
    };
    // Timed because it is the read the game paid nine milliseconds a frame for, and what it costs now is
    // the thing to watch: the `joystick=` span of the perf line, inside the `input=` it is part of.
    let started = profile::now();
    let word =
        buttons | unsafe { game.pad_word(joystick::reading(), DPAD_MOVES.load(Ordering::Relaxed)) };
    unsafe { profile::record(profile::Phase::Joystick, started) };
    word
}

/// Replaces `th06::ReplayManager::SaveReplay`, dropping the write while leaving
/// the teardown the game does through the same function.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// code calls that function, there being no prologue to patch — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
pub extern "C" fn save_replay(path: *const u8, name: *const u8) {
    if !path.is_null() && BLOCK_REPLAY_SAVE.load(Ordering::Relaxed) {
        log!("replay save blocked");
        return;
    }
    let original: extern "C" fn(*const u8, *const u8) =
        unsafe { std::mem::transmute(SAVE_REPLAY.load(Ordering::Relaxed)) };
    original(path, name)
}

/// Replaces `th06::ReplayManager::StopRecording`, which finishes an input record off
/// with a blank entry and a frame number no run reaches. Right for a recording, and
/// wrong for a replay: the game runs it from `GameManager::DeletedCallback` at every
/// stage teardown, and during playback the record it writes into is the replay's own,
/// at wherever playback has reached.
///
/// So leaving a stage part way — which is what moving between a replay's stages
/// does — leaves that stage terminated at the frame it was left on. Play it again and
/// the player takes no input from there: it stands still and is hit. Measured by jumping out of
/// stage 1 around script frame 250 and straight back — three lives gone by frame 1027.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its
/// own teardown calls that function, there being no prologue to patch — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
pub extern "C" fn stop_recording() {
    if let Some(game) = unsafe { chosen() }
        && unsafe { game.replaying() }
    {
        detail!("replay: the record is being watched, not written; its terminator dropped");
        return;
    }
    let original: extern "C" fn() =
        unsafe { std::mem::transmute(STOP_RECORDING.load(Ordering::Relaxed)) };
    original()
}

unsafe fn call_original(trampoline: &AtomicUsize, chain: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(trampoline.load(Ordering::Relaxed)) };
    original(chain)
}

/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn on_update(chain: *mut c_void) -> i32 {
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return unsafe { call_original(&RUN_CALC_CHAIN, chain) };
    };
    runtime.keyboard.poll(unsafe { runtime.game.window() });
    // **And this is the frame the rules are asked on, whichever read a game's own word comes through.**
    // A game that looks its keyboard read up reaches [`keyboard_state`], and that read is made more often
    // than the game moves — so where orb has a frame of its own the rules are asked here and that read
    // only writes the word down. Measured: an Extra run whose frame was orb's and whose read still asked
    // said `52447 of the 93711 frame(s) asked for were a tick read again`, which is
    // `orb_bot::steer::a_tick_read_again` catching more than half of the asks as a world that had not
    // moved since the answer before.
    THE_FRAME_IS_THE_SEAM.store(true, Ordering::Relaxed);
    unsafe { what_is_reached(runtime) };
    // Here, at the top, for the reason `trace::Record` gives: this update leaves by several doors, and a
    // record taken before any of them is one record however the frame went.
    trace::frame(
        runtime.frames,
        runtime.previous,
        runtime.chapters.number(),
        runtime.chapters.retries(),
        question_up(runtime),
    );
    // Before anything that can return early, because what this watches is the game's own counting of time and
    // the frames it counts on include the ones orb holds the chain back for — the mode question among them, which
    // is exactly the screen where nobody has chosen yet whose time it is.
    unsafe { runtime.game.notice_the_time() };
    // Beside it because it is the same kind of thing: a field of the game's that orb has to have read *before*
    // the moment it matters. The name entry is two screens in front of the question about keeping a replay, and
    // the screen it is on is gone by the time that file is sealed.
    unsafe { notice_the_name(runtime) };
    // And what the music coming out is, which is the one thing about the sound no other line says.
    unsafe { notice_the_track(runtime) };
    // And what became of a stream a restore has been through, over the frames after it — nothing while none
    // is being watched, which is every frame but the few after a chapter came back.
    unsafe { crate::audio::read_the_stream_back(runtime.game) };

    // Asked once, and here rather than at startup because the device does not exist until the game
    // has made one — which it does after `DllMain` has been and gone.
    if runtime.config.sent_keys && !runtime.sent_keys {
        runtime.sent_keys = true;
        log!(
            "input: {}",
            if unsafe { runtime.game.take_sent_keys() } {
                "the game's own keyboard device let go of; keys sent to it are read the other way"
            } else {
                "the game had no keyboard device to let go of; it was already reading the other way"
            }
        );
    }

    if let Some(menu) = &mut runtime.retry {
        unsafe { hold_frame(runtime.game) };
        let pad = unsafe { pad(runtime.game) };
        // **Or the answer a recipe asked for**, which is the last item of this menu and the one nothing can
        // press its way to — see [`give_the_run_up_at_the_retry`]. Asked before the keys are read, because a
        // bot that has asked presses nothing while it waits.
        // And the item a recipe named, which is any of them: what a press can reach is the cursor's own item,
        // so a launch playing itself could take the chapter it lost and nothing else — see
        // [`retry_ui::RetryMenu::answered_outright`].
        let answered = if asked_to_give_the_run_up() {
            Some((Choice::Quit, By::Recipe))
        } else if let Some(item) = asked_outright() {
            menu.answered_outright(item)
                .map(|choice| (choice, By::Recipe))
        } else {
            menu.update(&runtime.keyboard, pad)
        };
        if let Some((choice, by)) = answered {
            log!("retry: {} on the {by}", choice.label());
            let acted = match choice {
                Choice::Chapter => unsafe { runtime.chapters.retry_chapter(runtime.game) },
                Choice::Further(at) => unsafe { runtime.chapters.retry_kept(runtime.game, at) },
                Choice::Stage => unsafe { runtime.chapters.retry_stage(runtime.game) },
                Choice::Quit => {
                    let left = unsafe { give_up(runtime.game) };
                    if left {
                        unsafe { note_what_the_run_counted(runtime.game) };
                    }
                    runtime.given_up |= left;
                    left
                }
            };
            if acted {
                runtime.retry = None;
                // The frame the attempt that has just been rewound away ended on, taken before the
                // comparison below is dropped: the menu froze the game, so the frame orb last played is the
                // frame the death was on. Nothing for the item that gives the run up — that attempt was not
                // rewound, it is where the run stopped.
                let ended = match choice {
                    Choice::Quit => None,
                    _ => runtime.previous.map(|previous| previous.stage_frames),
                };
                // What the game carries on into is not a continuation of the frame we
                // froze on — a chapter put back, or the front end being built where the
                // run was given up — so nothing about it should be compared against that
                // frame.
                runtime.previous = None;
                // The chapter written down follows the run back, so that a session closed after going
                // back offers the chapter the run is standing in rather than the one it left. Here
                // because this is the chapter's own frame: the restore has just put the game on it, and
                // the song position and the reproduction line that go into the file with it are only
                // that frame's — by the update after this one the stage has moved on and
                // `keep_chapter` refuses. Nothing is written where the run has not moved, the file
                // already describing the chapter it was in.
                let state = unsafe { runtime.game.read_state() };
                // The attempt into the run's own replay first, and from the frame the restore has just put
                // the run on: the frames from there are about to be played again and written over, so this
                // is the one moment the attempt that was rewound away is still readable. Filed under the
                // chapter the run went back to, which is where somebody watching it will start it from.
                if let Some(ended) = ended {
                    unsafe {
                        replay::rewound(
                            &runtime.config.base_dir,
                            runtime.game,
                            runtime.chapters.started_at(),
                            ended,
                        )
                    };
                }
                unsafe { keep_chapter(runtime, &state) };
            }
        }
        return CHAIN_BREAK;
    }

    // No `hold_frame` with this one: it is over the game's own menu, and the frame that menu
    // draws into wants the whole output — which `prepare_frame` has already given it — rather
    // than the play field's viewport.
    if let Some(asking) = &mut runtime.asking {
        let pad = unsafe { pad(runtime.game) };
        let asked = asking.asked();
        let answered = match asked_outright() {
            Some(item) => asking
                .answered_outright(item)
                .map(|answer| (answer, By::Recipe)),
            None => asking.update(&runtime.keyboard, pad),
        };
        if let Some((answer, by)) = answered {
            runtime.asking = None;
            // Whichever way it was answered: the keys that answered it are the title menu's own keys
            // too, and the frame it carries on into is one where nothing of orb's has moved its
            // cursor. See `held_back`.
            SETTLE_KEYS.store(true, Ordering::Relaxed);
            match answer {
                // The mode goes in before the item is acted on, and then the press is handed over so
                // that the menu chooses the item itself.
                Answer::Chosen(mode) => {
                    log!("mode: answered on the {by}");
                    unsafe { choose(runtime, mode) };
                    // **The replays are the one item where the press is not always handed over.** The
                    // game's own replays are on the game's own screen, so that answer is the item being
                    // chosen; a pointdevice run's are orb's own — the game has no screen for them — so
                    // that answer is a list of orb's over an item the game never acted on.
                    match (asked, mode) {
                        (Menu::Replays, Mode::Pointdevice) => {
                            the_replays_to_watch(runtime);
                        }
                        _ => FEED_DECIDE.store(true, Ordering::Relaxed),
                    }
                }
                // The item is not chosen: the press that would have chosen it was held back, so the
                // menu never left the item it is on and there is no animation to put back. The mode is
                // left as it was, nothing having been answered.
                //
                // The key that cancelled is held back until it is let go: it is still down, and what
                // the title menu does with it is put its own cursor on `Quit`.
                Answer::Cancelled => {
                    log!("mode: not chosen on the {by}; the menu is where it was");
                    HOLD_CANCEL.store(true, Ordering::Relaxed);
                }
            }
        }
        return CHAIN_BREAK;
    }

    // The lists a replay is chosen out of, over the game's own title menu — whose press was held back for
    // the question this came from, so no `hold_frame` here either: what is underneath is a menu, and
    // `prepare_frame` has given the frame the whole output.
    if let Some(choosing) = &mut runtime.choosing_replay {
        let pad = unsafe { pad(runtime.game) };
        // **Or the item a recipe asked for outright**, which is the only way a launch playing itself reaches
        // anything but the first of each list — see [`answer_the_question`]. Asked before the keys are read,
        // because a bot that has asked presses nothing while it waits.
        let answered = match asked_outright() {
            Some(item) => {
                let (list, count) = (choosing.list_now(), choosing.items_now());
                if item >= count {
                    log!("replay: the {list} list has {count} item(s), so there is no item {item}");
                }
                choosing
                    .answered_outright(item)
                    .map(|chosen| (chosen, By::Recipe))
            }
            None => choosing.update(&runtime.keyboard, pad),
        };
        if let Some((chosen, by)) = answered {
            // Taken rather than dropped: the list an attempt was chosen out of is put back up when that
            // attempt is over — see [`WatchedAttempt::list`].
            let list = runtime.choosing_replay.take();
            SETTLE_KEYS.store(true, Ordering::Relaxed);
            match chosen {
                replay_ui::Watched::Chosen { .. } => {
                    log!("replay: chosen on the {by}");
                    unsafe { watch_an_attempt(runtime, &chosen, list) };
                }
                // The item is not chosen: the press that would have chosen it was held back, so the menu
                // never left the item it is on. The key that cancelled is held back until it is let go, the
                // way the mode question's is — what the title menu does with it is put its cursor on `Quit`.
                replay_ui::Watched::Cancelled => {
                    log!("replay: nothing chosen; the menu is where it was");
                    HOLD_CANCEL.store(true, Ordering::Relaxed);
                }
            }
        }
        return CHAIN_BREAK;
    }

    // Whether to keep the replay of a run that is finished with. Over the screen the run ended at, or over
    // the shot type select where a run is about to be started over — both of them the game's own screens, so
    // no `hold_frame` here: what is underneath wants the whole output, which `prepare_frame` has given it.
    if let Some((menu, _, _)) = &mut runtime.asking_replay {
        let pad = unsafe { pad(runtime.game) };
        // Or the answer a recipe asked for outright, `いいえ` being an item no press walks to — see
        // [`answer_the_question`].
        let answered = match asked_outright() {
            Some(item) => menu
                .answered_outright(item)
                .map(|answer| (answer, By::Recipe)),
            None => menu.update(&runtime.keyboard, pad),
        };
        if let Some((answer, by)) = answered {
            let (_, slot, ended) = runtime.asking_replay.take().expect("the question is up");
            SETTLE_KEYS.store(true, Ordering::Relaxed);
            match answer {
                replay_ui::Answer::Keep => {
                    log!("replay: kept on the {by}");
                    replay::keep_what_was_being_played(
                        &runtime.config.base_dir,
                        runtime.game,
                        &slot,
                        &runtime.name,
                        orb_api::clock::local_time(),
                        ended,
                    );
                }
                replay_ui::Answer::Decline => {
                    log!("replay: declined on the {by}");
                    replay::declined(&runtime.config.base_dir, &slot);
                }
            }
            // The press that answered is handed on where one was being held back for this, which is the
            // question asked on the way into a run: the run is still to be started. Nothing is held back at
            // the screen a run ended at, and nothing is handed over there either.
            if HELD_FOR_THE_REPLAY.swap(false, Ordering::Relaxed) {
                FEED_DECIDE.store(true, Ordering::Relaxed);
            }
        }
        return CHAIN_BREAK;
    }

    // Where to start a run that has a chapter of its own left unfinished. Over the game's own shot
    // type select, whose decide is being held back for exactly this — so no `hold_frame` here either.
    if let Some((menu, ..)) = &mut runtime.asking_resume {
        let pad = unsafe { pad(runtime.game) };
        // Or the answer a recipe asked for outright, `はじめから` being the item no press walks to — see
        // [`answer_the_question`], where three launches on the real game are what says so.
        let taken = match asked_outright() {
            Some(item) => menu
                .answered_outright(item)
                .map(|answer| (answer, By::Recipe)),
            None => menu.update(&runtime.keyboard, pad),
        };
        if let Some((answer, by)) = taken {
            let (_, saved) = runtime.asking_resume.take().expect("the menu is up");
            SETTLE_KEYS.store(true, Ordering::Relaxed);
            unsafe { answered(runtime, answer, saved, by) };
        }
        return CHAIN_BREAK;
    }

    // A replay can be run faster than it was recorded, which is what makes a pass
    // over a full run to collect chapter boundaries take minutes. Every update is
    // still observed, so nothing is missed by going quickly.
    //
    // A run being cleared to reach an ending goes the same way. It is somebody at the
    // keyboard rather than a replay, but they are not dodging anything — nothing can hit
    // them — so the frames are there to see the run go by and not to play on.
    let fast_forward = runtime.previous.is_some_and(|previous| {
        previous.replay && previous.playing && !previous.demo
            || runtime.config.fast_clear && previous.in_game
    });
    let mut repeats = if fast_forward {
        runtime.config.speed
    } else {
        1
    };

    let mut result = CHAIN_BREAK;
    let mut state = runtime
        .previous
        .unwrap_or(unsafe { runtime.game.read_state() });
    // Stepping between chapter boundaries, and the hold that lets the frame one
    // falls on be looked at. Before the frame's own updates, and instead of them.
    if let Step::Held { reached, ran } = unsafe { step(runtime, chain, &state) } {
        state = reached;
        result = ran;
        // The game's own frame setup is part of the chain being held back, so the
        // viewport it would have set has to be set here instead.
        unsafe { hold_frame(runtime.game) };
        repeats = 0;
    }

    // Timed around orb's own work only: the game's update runs in between, and
    // counting it made orb look like it was costing milliseconds a frame.
    let mut started = profile::now();
    for _ in 0..repeats {
        // Before the update, because what reads it is the hit test inside the update. Not
        // during a replay or the attract demo, where the run is a record of one somebody
        // played and the player not dying would be a different run from the one recorded.
        if runtime.config.fast_clear && state.in_game {
            unsafe { runtime.game.make_invulnerable() };
        }
        unsafe { profile::record(profile::Phase::Update, started) };
        result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        started = profile::now();
        state = unsafe { runtime.game.read_state() };
        if runtime.chaptering() {
            unsafe {
                runtime.chapters.observe(
                    runtime.game,
                    &state,
                    &runtime.data,
                    runtime.config.self_check,
                )
            };
        }
        unsafe { reproduced(runtime, &state) };
        // **And the generator of every frame somebody plays, where something is being diagnosed.** The same
        // line a playback writes — see [`play_in`] — so a run played by hand and a run picked up out of the
        // file it left hold their generators against each other frame for frame, and the first frame the two
        // differ is where the playback stopped reproducing the run. Held against the state line a run writes
        // anyway, the comparison is only as fine as that line is often, which is once every sixty frames: a
        // th07 run picked up in stage 6 parted somewhere in the sixty frames after 853 and nothing narrowed
        // it further.
        //
        // **The whole reproduction line and not the generator alone**, because which *field* parts first is
        // the answer: the generator says a playback has stopped reproducing the run and nothing about why, and
        // a run of stage 2 held against its own playback parted at frame 802 with the boundaries at 1549 and
        // 2827 still landing on the same frames — so the world drew differently while the script did not, and
        // what has to be told apart is a player somewhere else from an item nobody picked up. Every field of
        // it is in this line: see `Game::reproduction`.
        //
        // At the verbose level, which costs log and nothing else — and only while a stage is running, a frame
        // of the front end having no run to be about.
        //
        // **And only where the game answers any of it.** A game whose stage clock and generator are unread
        // has nothing for two runs to be held against each other by, and the line is then its own frame
        // number of nought and two empty fields: measured, a th06c launch wrote 3,713 of them in four
        // minutes, one per frame, saying `resume: frame 0 played with  updates=0`.
        let reproduction = unsafe { runtime.game.reproduction() };
        if log::wanted(log::VERBOSE) && state.in_run && !state.paused && !reproduction.is_empty() {
            log!(
                "resume: frame {} played with {}{} updates={}",
                state.stage_frames,
                reproduction,
                the_worlds_fingerprint(runtime.game),
                updates_taken(),
            );
        }
        if result == CHAIN_EXIT_SUCCESS || result == CHAIN_EXIT_ERROR {
            break;
        }
    }

    // A run being picked up where an earlier session left it: the buttons it has already pressed,
    // played into the stage the game has just built. Inside the frame that built it, so nothing of
    // the run being played forward is ever drawn.
    if let Some(at) = unsafe { resume::landing() }
        && state.playing
    {
        let (reached, ran) = unsafe { play_in(runtime, chain, at) };
        state = reached;
        result = ran;
    }
    // The stage a watched attempt is in, which the run the front end just started has none of: see
    // [`resume::the_stage_of_a_watch`], and [`Game::front_end_starts_a_run`] for why that item is what starts
    // it. Before the block below, so that a stage which does not come up is one this had its say about.
    if unsafe { resume::starting() } && unsafe { runtime.game.run_chosen() } {
        unsafe { resume::the_stage_of_a_watch(runtime.game) };
    }
    // Waiting for the stage one asked for. A stage takes a second to build; ten seconds of frames
    // is the game having gone somewhere else with the run, which is what a stage it cannot load
    // does.
    if unsafe { resume::starting() } {
        runtime.starting += 1;
        if runtime.starting > RESUME_START_FRAMES {
            unsafe { resume::abandon("the stage never came up") };
        }
    }

    // Runs the ending out inside the frame it starts on, which is what keeps it off the screen
    // entirely: drawing happens once per frame, and by the next one the game is on the frame
    // the skip stopped at. Jumping the scene instead would be simpler, but the ending is also
    // where the game sets the clear flag and enters the score, and those have to happen.
    //
    // Stops where the ending hands over to its staff roll rather than at the scene change,
    // which is what keeps the roll: the roll is the last of an ending's scripts and the scene
    // is the same across all of them, so stopping at the scene ran the roll out too.
    //
    // Never during a demo or a replay. Only a run someone played reaches an ending worth
    // skipping, and a demo that looked like one is what turned this into a hundred
    // thousand updates a frame.
    // **And always where an attempt out of a replay was what reached it**, whatever the setting says: an
    // ending is what a run of somebody's own earned, and the run an attempt is watched in is nobody's — it
    // played the frames the boss died on again. Somebody who asked for the ending asked for it about their
    // own clear.
    if (runtime.config.skip_ending || runtime.watching.is_some())
        && state.in_ending
        && !state.demo
        && !state.replay
        // Not once the roll is what is running: the ending's flag stays set through it and the
        // scene stays 10, so this would start again on the next frame and run out the one part
        // of an ending that was worth keeping.
        && !runtime.rolling
    {
        let scene = state.scene;
        let script = state.ending_script;
        let track = runtime.game.music_identity();
        let mut frames = 0;
        let mut rolling = false;
        // Stops at the scene change as well as at the flag, so that whatever follows the
        // ending is reached and then left alone.
        while state.in_ending
            && state.scene == scene
            && !rolling
            && frames < ENDING_SKIP_LIMIT
            && result != CHAIN_EXIT_SUCCESS
            && result != CHAIN_EXIT_ERROR
        {
            result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
            state = unsafe { runtime.game.read_state() };
            frames += 1;
            rolling = moved_on(script, state.ending_script);
        }
        runtime.rolling = rolling;
        if rolling {
            // The track with it, because the roll's script starts one of its own — the ending
            // plays `bgm/th06_16` and the roll `bgm/th06_17` — so both changing on the same
            // update is a second thing saying this is where one ends and the other begins.
            log!(
                "ending run out in {frames} update(s), where its staff roll begins, \
                 track {track:?} -> {:?}",
                runtime.game.music_identity(),
            );
        } else {
            log!(
                "ending skipped, {frames} frames run, scene {scene} -> {}",
                state.scene
            );
        }
    }
    // The roll is inside the ending as far as the game is concerned, so what says there is no
    // longer one to keep out of the way of is the ending's flag going out.
    if !state.in_ending {
        runtime.rolling = false;
    }

    // The screen that offers to save a replay of the run just finished, which a pointdevice run has
    // nothing the *game* could put in one — see `Game::skip_replay_prompt`. Costs one read on every other
    // frame there is.
    //
    // **And it is where the offer to keep orb's own goes**, that being what the game's question is written
    // past for: this is the frame the game would have asked on, which is after the name entry and the stats
    // screen — the two screens whoever just finished the run reads and orb has no business over. Asked here
    // rather than where the chapter file is taken away, which is the frame the result screen comes up on:
    // a question there covers the name entry.
    if runtime.mode == Mode::Pointdevice && unsafe { runtime.game.skip_replay_prompt() } {
        log!("result: no replay of the game's own is offered for a run with chapters");
        let run = unsafe { runtime.game.run_start() };
        if let Some(slot) = runtime.game.run_slot(&run) {
            unsafe { the_replay_of_a_run_finished_with(runtime, &slot) };
        }
    }

    // Over a replay as well as over a run someone is playing, because a replay is
    // what plays a whole run for a tuning pass.
    if runtime.chaptering() && runtime.chapters.tracking(&state) {
        tune(runtime, &state);
        boundary_reached(runtime, &state);
        unsafe { keep_chapter(runtime, &state) };
    }
    // **An attempt out of a replay running out, which is where the run orb built to watch it in is left.**
    //
    // The attempt is a stretch of a stage and not a run: what follows the frame it ended on is whatever the
    // player's own keyboard does to a stage nobody is playing, so there is nothing to carry on into. Ended
    // here rather than where the buttons stop being fed, because the frame they stop on is one the game
    // still has to run — the last of the attempt.
    // **And not while the run it is to be watched in is still being started**, which is the frames between
    // the front end being asked for one and the game registering it — see
    // [`Game::front_end_starts_a_run`]. A run that has not come up yet is not a run that has gone, and
    // taking it for one ended every watch on the frame after the attempt was chosen. What bounds the wait is
    // [`RESUME_START_FRAMES`], which the playback holds itself to.
    let starting = unsafe { resume::starting() };
    if let Some(watching) = runtime.watching.as_mut().filter(|_| !starting) {
        let over = !state.in_run || state.stage_frames > watching.until;
        // And the way out somebody watching one has, which is the only one they have: the buttons the game
        // acts on are the attempt's, so its own pause is not theirs to open. A cancel, on either hand — the
        // keyboard's two keys and whichever button this game's menus take one on, which is what somebody
        // with a controller in their hands has. Read through [`Keys`] and not off the keyboard, because a
        // stop the pad could not ask for left a chapter minutes long with escape as the only way out of it.
        let stopped = watching
            .keys
            .read(&runtime.keyboard, unsafe { pad(runtime.game) })
            .and_then(|pressed| pressed.cancel);
        if state.in_run && (over || stopped.is_some()) {
            let asked = stopped.map_or_else(String::new, |by| format!(", asked for on the {by}"));
            log!("replay: the attempt is over; the run it was watched in is left{asked}");
            unsafe { give_up(runtime.game) };
        }
        if !state.in_run {
            // The list it was chosen out of comes back with it, once the screen it stands over is up — see
            // [`ToPutUp`].
            runtime.to_put_up = runtime.watching.take().map(|watched| ToPutUp {
                list: watched.list,
                came_back: true,
                asked: false,
                drawn: 0,
                waited: 0,
            });
            unsafe { resume::forget() };
            // What this run was is nothing, so nothing about the frame before it applies to the one after —
            // which is also what keeps the block below from taking it for a run somebody gave up and
            // building a ranking out of what it counted.
            runtime.previous = None;
            resume::keep(runtime.keeping());
            replay::keep(runtime.keeping_the_replay());
            log!("replay: the run it was watched in has gone");
        }
    }
    // **The run taken out of the screens a finished run ends at**, where the attempt just watched was the
    // path the boss died on and the game has gone into them. The name entry is where somebody types their own
    // name and the stats after it are theirs to read; this run is nobody's — see
    // [`Game::leave_the_screen_a_run_ends_at`], and `score::watching` for the file, which the screen writes
    // on its way down whichever way it goes.
    // **And run out inside the frame they came up in, the way an ending is.** Drawing happens once per
    // frame, so a screen taken down before this frame's draw is a screen nobody sees. The write on its own
    // only asks the screen to leave, and it plays its own way out over the frames that takes — twenty of
    // them on th06 — so a run nobody played put its name entry on the screen on the way past.
    // **The write asked for inside the loop and not once before it**, because the frame the scene becomes
    // that screen's is a frame the screen is not on the chain yet: asked once, the write missed that frame
    // and the frame drew a name entry.
    if runtime.to_put_up.is_some() && unsafe { runtime.game.the_screen_a_run_ends_at_is_up() } {
        let mut frames = 0;
        while unsafe { runtime.game.the_screen_a_run_ends_at_is_up() }
            && frames < RESULT_SKIP_LIMIT
            && result != CHAIN_EXIT_SUCCESS
            && result != CHAIN_EXIT_ERROR
        {
            unsafe { runtime.game.leave_the_screen_a_run_ends_at() };
            result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
            state = unsafe { runtime.game.read_state() };
            frames += 1;
        }
        log!(
            "replay: the screens a finished run ends at are none of the watched run's; the game is taken \
             out of them in {frames} update(s)"
        );
    }
    // **And the list put up over the screen the game keeps its own replays on**, which is where a list of
    // replays belongs: that item is what was chosen to get here, and a list over the title menu — or, coming
    // back after an attempt, over th06's logo — reads as having left the replays rather than as being in
    // them. Asked for on every frame the front end is up and not on it yet, a run being taken down being no
    // front end to ask; the game answers `false` while it is already on its way there, which is what keeps a
    // transition from being restarted every frame.
    //
    // **And the frames the front end is left to draw itself in where the game has no such screen**, because
    // the list freezes what is under it: put up the moment the front end was what was running, the list
    // stood over a screen the game had not drawn yet and went on standing over it, the chain being broken
    // from then on. Measured on the real game, where that was a list on a flat dark screen.
    if let Some(mut waiting) = runtime.to_put_up.take() {
        let front = !state.in_run && unsafe { runtime.game.front_end_screen() }.is_some();
        let there = front && unsafe { runtime.game.the_screen_the_replays_are_on() };
        if front && !there {
            waiting.asked |= unsafe { runtime.game.to_the_screen_the_replays_are_on() };
            waiting.waited += 1;
        }
        if there {
            waiting.drawn += 1;
        }
        // How long that screen is left to draw itself in: a screen that came up on this list's account draws
        // itself in over those frames, and so does a front end built on the frame a run was taken down — the
        // one screen that needs none of them is the one somebody was already sitting on, drawn every frame
        // since they walked a cursor to that item.
        let draws_in = match waiting.asked || waiting.came_back {
            true => THE_SCREEN_DRAWS_FOR,
            false => 0,
        };
        let drawn = there && waiting.drawn > draws_in;
        // And a screen that never came: the list goes over whatever the front end has rather than not at all.
        let given_up = front && waiting.waited > A_SCREEN_IS_WAITED_FOR;
        if drawn || given_up {
            if given_up && !there {
                log!(
                    "replay: the screen the game keeps its own replays on never came up; the list goes over \
                     the screen that did"
                );
            }
            runtime.choosing_replay = Some(waiting.list);
            if let Some(list) = &mut runtime.choosing_replay {
                // The keys held off, the way they are on every list this menu opens: the press that answered
                // the mode question — or the one that ended the attempt — is not a press at this list.
                list.opened_again();
                // And the ranking is a run's own again from here: the screens the watched run ended at are
                // down, and their write is behind them.
                score::watching(false);
            }
            if waiting.came_back {
                log!(
                    "replay: the attempts at that chapter again, the one that was watched under the cursor"
                );
            } else {
                log!("replay: the replays to watch, over the screen the game keeps its own on");
            }
        } else {
            runtime.to_put_up = Some(waiting);
        }
    }

    // **The run's own replay written down and the record of it dropped, on the first frame there is no run
    // any more.** This is what leaves the stretch the run ended in watchable: what otherwise hands the file
    // over is a chapter beginning, and on the frame a chapter begins the run has played none of it yet.
    //
    // Asked of this frame alone and not of the change from the frame before, which is how the block below
    // notices a run ending: the retry menu drops the comparison as it acts, so a run given up through it has
    // no frame before to be held against — and that is the way out this is most for.
    if !state.in_run && unsafe { replay::keeping() } {
        unsafe { replay::left(&runtime.config.base_dir, runtime.game) };
    }
    // The run over, whichever way: what was written down is a stage the game is taking down, and
    // the file beside it is left holding the chapter unless the run *finished* — the result screen
    // is what a run reaches and what giving one up does not go through.
    if runtime.previous.is_some_and(|previous| previous.in_run) && !state.in_run {
        unsafe { resume::forget() };
        // Any run that ends, not only one given up through orb's own menu: legacy mode has no such
        // menu, so the game's own quit is the only way out of one and it writes nothing either. A demo
        // is nobody's run and has nothing to write.
        if runtime
            .previous
            .is_some_and(|previous| !previous.demo && !previous.replay)
        {
            runtime.given_up = true;
            unsafe { note_what_the_run_counted(runtime.game) };
            log!(
                "score: a run ended; what it counted waits for the ranking to be built and taken down"
            );
        }
    }
    // Unless the run finished, where the screen it finished at is what writes: `ResultScreen`'s deleted
    // callback is the one caller of the write in the whole exe, and a run that reached that screen is
    // already on its way through it. So there is nothing to build a ranking for, and building one there
    // is not harmless either — the front end it is asked of is not up, so the updates would all be spent
    // on the result screen and the request then undone, which used to be a write into the name entry
    // somebody was standing at.
    if runtime.given_up && unsafe { runtime.game.run_finished() } {
        runtime.given_up = false;
        log!("score: the run finished, and the screen it finished at is what writes");
    }
    // The first frame after a run was given up on which the game has arrived somewhere: asking for a
    // screen while a change was in flight is what broke the front end twice. Not while an ending is
    // running, which is a cleared run's own way to that screen: the roll is played out an update a frame,
    // and a ranking asked for over one is asked of a front end that cannot come up, so the whole
    // allowance went on frames of the roll — see `the_ending.rs`.
    if runtime.given_up && !state.in_run && !state.in_ending && state.scene == state.wanted {
        runtime.given_up = false;
        unsafe { commit_records(runtime, chain, &mut result) };
        state = unsafe { runtime.game.read_state() };
    }
    // The mode as well as the chapter, because the result screen is a screen every run ends at: a
    // normal run's game over would otherwise take away the chapter a pointdevice run was left in.
    // And this run's own file, which is the one the run just finished was written to.
    if runtime.kept.is_some() && runtime.keeping() && unsafe { runtime.game.run_finished() } {
        let run = unsafe { runtime.game.run_start() };
        resume::discard(&runtime.config.base_dir, runtime.game, &run);
        runtime.kept = None;
    }
    unsafe { stress(runtime, &state) };
    unsafe { watch_music(runtime, &state) };

    // The stage a run has got to, which is the one part of [`Reached`] that needs a stage read: the rest of it
    // is filled where the frame begins — see [`what_is_reached`], and the reason it cannot wait until here.
    if let Some(playing) = PLAYING_IT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_mut()
    {
        playing.reached.stage = state.stage + 1;
        // **And whether there is a run at all**, which is what makes a `play` step end: a run that lost its last
        // life leaves the game on its own screen, the rules press nothing there, and a step waiting for a stage
        // would sit out its whole bound. Here rather than where the rest of [`Reached`] is filled, because this
        // is where the state is read.
        playing.reached.over = !state.in_run;
        // And whether the stage's own boss is up, which is what a midsection ends at — read here for the same
        // reason, this being where the state is read. **Not the boss slot being filled**, which a midboss fills
        // as well: a step written to play the whole of stage 4's midsection ended at its midboss a third of the way
        // in.
        playing.reached.boss = runtime
            .chapters
            .the_stages_own_boss_is_up(runtime.game, &state);
        // And whether a line of dialogue is up, which is the frame a picture of a boss's conversation has to be
        // asked on: the rules press past a line as fast as the game takes presses, so a step bounded by frames
        // instead walks either side of one.
        // **A line and not the game's own predicate.** `in_dialogue` counts th07's hand-over line, which is a
        // frame with no words drawn on it: a recipe photographing a conversation waited on that in stage 5 and
        // stage 6 and got the frame the box was still being drawn on.
        playing.reached.dialogue = unsafe { runtime.game.a_line_of_dialogue_is_shown() };
        // And whether a card is running, and whether the plate its name is on is being drawn — which is a frame
        // or more later, and the frame a picture of that name has to be asked on.
        playing.reached.card = state.spellcard.is_some();
        playing.reached.card_name = unsafe { runtime.game.a_cards_name_is_shown() };
        // And the game's own name entry, which the run's end is past — see `recipe::Until::Name`.
        playing.reached.typing_a_name = unsafe { runtime.game.typing_a_name() };
    }

    // **The count moving is where a death is noticed and not where it is settled.** th06 counts the *hit*:
    // `Player::Die` puts the count up as the hit lands and leaves the frames a deathbomb is still taken in
    // behind it, where th07 counts in its death handler once its own window has closed. So the count is
    // held here while the game says the window is open, and a bomb inside it leaves the run playing with no
    // offer ever made — which is the whole of the window: an offer raised inside it freezes the game on the
    // frame the player would have bombed on, and takes the death th06 was going to give back.
    match state.in_game {
        // A death being decided is this run's: a stage left while the window was open — a run given up, a
        // game closed — leaves nothing to decide, and a decision kept across that would be answered against
        // whatever the next run's player is doing.
        false => runtime.deciding_a_death = false,
        true if runtime
            .previous
            .is_some_and(|previous| state.deaths > previous.deaths) =>
        {
            runtime.deciding_a_death = true
        }
        true => {}
    }
    let died = runtime.deciding_a_death
        && match unsafe { runtime.game.where_a_counted_death_stands() } {
            CountedDeath::Open => false,
            settled => {
                runtime.deciding_a_death = false;
                settled == CountedDeath::Taken
            }
        };
    if died && runtime.chaptering() && runtime.chapters.can_retry() {
        log!("died in chapter {}", runtime.chapters.number());
        let chapters = offered(runtime);
        runtime.retry = Some(RetryMenu::new(runtime.language, chapters));
    }

    // **And the same menu where the game was asked to give the run up and begin it again**, which is its pause
    // menu's own 「最初からやり直す」. In a mode whose runs are rewound that ask has nowhere to land: nothing
    // here is lost, so starting the *run* over is the one way to throw a stage's chapters away — and it is the
    // only way to reach a chapter again without dying first, which is what this mode is for.
    //
    // So the ask is taken back off and what it opens is the question of where to start over from, which already
    // holds every answer the item could have meant: this chapter, one further back, the stage, or the title.
    // **And a recipe's own ask read beside the menu's**, first so that it is taken whether or not the menu
    // asked too: a flag left standing would raise the question again on a later frame, where what was asked
    // for was one.
    if runtime.chaptering()
        && runtime.retry.is_none()
        && runtime.chapters.can_retry()
        && (asked_by_a_recipe() || unsafe { runtime.game.asked_to_start_the_run_over() })
    {
        unsafe { runtime.game.keep_the_run_where_it_is() };
        log!(
            "retry: the run was asked to start over in chapter {}; where from is the question",
            runtime.chapters.number(),
        );
        let chapters = offered(runtime);
        runtime.retry = Some(RetryMenu::new(runtime.language, chapters));
    }

    // The question after the character select, where a run has been chosen and a chapter of that
    // same run — the same difficulty, character and shot — was left unfinished. Read off the disk
    // here rather than kept from startup: what it offers is the chapter the last session stopped at,
    // which moves with every chapter a run reaches.
    //
    // Before the frame that builds the run, which is what makes both answers cost nothing: the game
    // has settled what the run is and has not acted on it.
    if runtime.keeping() && runtime.asking_resume.is_none() && unsafe { runtime.game.run_chosen() }
    {
        let run = unsafe { runtime.game.run_start() };
        // A run the game keeps no slot for is a practice run: nothing was written down for it and
        // nothing is asked about it, and that ends here rather than in a line of the log.
        if let Some(slot) = runtime.game.run_slot(&run) {
            // Whether a question about this run is standing by the end of this frame, which is what the
            // replay below must not be settled over.
            let mut asking = false;
            match runtime.started.take() {
                // Answered a screen earlier, on the press that started this run. The chapter goes in
                // here and not there because this is the one frame the game will take a stage for a
                // run it has not built yet — see [`Game::start_stage`].
                Some(started) => {
                    if started.slot != slot {
                        log!(
                            "resume: {} was answered for, and {slot} is what the game registered",
                            started.slot
                        );
                    }
                    if let Some(saved) = started.saved.filter(|_| started.slot == slot) {
                        runtime.starting = 0;
                        unsafe { resume::begin(runtime.game, saved) };
                    }
                }
                // Nothing was answered a screen earlier, which is the one frame left to ask on — see
                // [`resume_ui::Cancels::Nothing`].
                None => {
                    asking = ask_where_to_start(runtime, &run, &slot, resume_ui::Cancels::Nothing);
                }
            }
            // **And the replay this run writes, where nothing has settled one yet.** What settles it is the
            // press on the shot type select and the answer that press puts up — see
            // [`the_replay_this_run_writes`] — and this is the frame that catches a run neither reached: a
            // launch with no overlay holds no press back, and a run started by anything but that press is
            // still a run whose replay is worth keeping. The slot here is the one the game gives, which for
            // th07 is a build's away from being this run's; `replay::stage_begun` is where that is put right.
            //
            // **And not while a question about this very run is standing**, which a launch on the real game
            // caught: the frame `どこから始める` went up here, this settled a fresh replay over the one being
            // asked about — sealing it and taking its file away — and the つづきから answered twenty-two
            // frames later found nothing to carry on. What the answer decides is which replay this run
            // writes, so a settling that runs before it is a settling of the wrong one.
            if !asking && !unsafe { replay::keeping() } {
                unsafe {
                    the_replay_this_run_writes(
                        runtime,
                        &run,
                        &slot,
                        None,
                        replay::TheOneBefore::Kept,
                    )
                };
            }
        }
    }

    // And the same thing said a screen earlier, over the game's own shot type select, where it costs
    // nothing and freezes nothing: which run is under the cursor, and whether that one has a chapter
    // written down. Every frame, because the cursor moves between the two shots on the screen — the
    // file is only read where the answer would be about another run.
    let keeping = runtime.keeping();
    let pointing = keeping
        .then(|| unsafe { runtime.game.run_pointed_at() })
        .flatten();
    let pointed = pointing.as_ref().and_then(|run| runtime.game.run_slot(run));
    let base = &runtime.config.base_dir;
    runtime.mark.pointing(pointed.as_deref(), |slot| {
        resume::peek(base, runtime.game, slot)
    });

    // And the press that would choose an item is kept from the game on both of the screens orb has a
    // question about — the title menu and the shot type select — so that a question goes on the press
    // rather than after it. See `held_back`, and `Game::menu_pointed_at` for what that buys: a screen
    // whose press never arrived is a screen a cancel leaves exactly where it was, with no state to put
    // back and no animation to sit through.
    //
    // Whichever item the cursor is on, and not only the ones orb would ask about, although that is a
    // press held back on every run started and one file read on each. The frame the cursor arrives on
    // the item is why: this is written after the game's update, so it is the *next* frame's read the
    // holding starts on, and both screens move their cursor before they read their decide — 0x436a88
    // against 0x436d79 on the one, 0x437b5c against 0x437c1b on the other. So a direction and the shot
    // button on one frame choose the item the cursor has just reached, and holding only under the
    // cursor's own answer would leave that one frame able to start a run nobody was asked about.
    //
    // Not without an overlay to draw the question with: a press held back for a question nobody can
    // see is a screen that has stopped working.
    // Flattened, and not left as the `Option<Option<_>>` `then` hands back: the outer one says only
    // that somebody is there to be asked, which is a property of the launch and not of the screen —
    // read for the hold it would have been on every frame of every run, with the shot button taken
    // out of the word the game reads.
    let asked_about = runtime
        .asks_mode
        .then(|| unsafe { runtime.game.menu_pointed_at() })
        .flatten();
    HOLD_DECIDE.store(
        (pointing.is_some() || asked_about.is_some())
            && runtime.overlay.is_some()
            && unsafe { runtime.game.menu_takes_a_press() },
        Ordering::Relaxed,
    );
    if DECIDE_PRESSED.swap(false, Ordering::Relaxed) {
        match (pointing, pointed, asked_about) {
            (Some(run), Some(slot), _) => {
                // Nothing left of this run is nothing answered, so this press is where the replay it writes
                // is settled: see [`the_replay_this_run_writes`], and [`answered`] for the other two ways.
                if !ask_where_to_start(runtime, &run, &slot, resume_ui::Cancels::TheRun) {
                    unsafe {
                        the_replay_this_run_writes(
                            runtime,
                            &run,
                            &slot,
                            None,
                            replay::TheOneBefore::Kept,
                        )
                    };
                }
            }
            (_, _, Some(menu)) => {
                log!("menu: {menu:?} is under the cursor, asking which mode");
                runtime.asking = Some(ModeMenu::new(menu, runtime.mode, runtime.language));
            }
            // An item orb has nothing to ask about, which is four of the title menu's eight and every
            // frame the cursor has moved off one it does ask about. The press goes to the front end
            // unanswered, a frame late and otherwise as it was made.
            _ => {
                detail!(
                    "menu: nothing to ask about the item under the cursor; the press is handed over"
                );
                FEED_DECIDE.store(true, Ordering::Relaxed);
            }
        }
    }

    // Every frame while nothing is being played, not only where the scene changed: what building the
    // game's own ranking has to fit into is the frames *inside* a transition, and a line per
    // change showed neither how many there are nor what the two fields do across them.
    if !state.in_run {
        detail!("state: cur={} wanted={}", state.scene, state.wanted);
    }
    let scene_changed = runtime
        .previous
        .is_none_or(|previous| previous.scene != state.scene);
    let due = state.in_game && runtime.frames % STATE_LOG_INTERVAL == 0;
    if scene_changed || due {
        summary!("f{} {state} clears={}", runtime.frames, unsafe {
            runtime.game.clears_back_buffer()
        },);
    }
    // `quiet` writes none of the above, and the compose time the pacing settles at is a property of
    // what the game was doing while it settled — a stage being played and a menu are not the
    // same load. Without this a pacing run's numbers cannot be attributed to anything, which
    // is how a sweep came to be written up against a scene nobody had recorded.
    //
    // Once per report rather than per second, so there is one of these against each set of
    // numbers and no more.
    if log::pacing_wanted() && (scene_changed || runtime.frames % profile::INTERVAL == 0) {
        pacing!("f{} {state}", runtime.frames);
    }

    // Beside the state itself, because what reads it is the parse hook and not this function — a read of the
    // file made while this is set is one whose card records in memory are the run's own.
    A_RUN_WAS_PLAYING.store(
        state.in_run && !state.demo && !state.replay,
        Ordering::Relaxed,
    );
    runtime.previous = Some(state);
    runtime.frames += 1;
    // Here rather than in the draw hook: this writes on the window itself, which is not
    // something to do in the middle of a scene the game is drawing into.
    unsafe { write_status(runtime) };
    unsafe {
        profile::record(profile::Phase::Update, started);
        if profile::frame() {
            // All of it where the pacing is what is being watched, and `report` alone where it is
            // not: the buckets and the interval say as much about the pacing as the worsts do, and
            // `quiet` — the level a sweep is read at — has nothing else to write them.
            if log::pacing_wanted() {
                pacing!("{}", pacing().report());
                pacing!("{}", pacing().worst());
                pacing!("{}", pacing().shown());
            } else {
                summary!("{}", pacing().report());
            }
            summary!(
                "audio: behind {}..{} bytes",
                if runtime.margin_best == u32::MAX {
                    0
                } else {
                    runtime.margin_best
                },
                runtime.margin_worst,
            );
            runtime.margin_best = u32::MAX;
            runtime.margin_worst = 0;
        }
    }
    result
}

/// Whether an ending has gone from one of its scripts to the next, which is where the ending
/// itself ends and its staff roll begins.
///
/// Both sides have to be known for a change to say that. With no script to compare — a game
/// whose ending orb cannot find, or an ending already torn down — the answer is no, and the
/// skip runs the scene out the way it did before there was a script to read.
fn moved_on(from: Option<usize>, to: Option<usize>) -> bool {
    matches!((from, to), (Some(from), Some(to)) if from != to)
}

/// Puts the replays somebody has kept on the screen, which is what 完全無欠モード at the game's own replay
/// item answers to.
///
/// **Every one of them read here**, which is a file read apiece on the frame the question was answered: the
/// game is frozen and what the list has to show is whose run each was and what is inside it, and nothing but
/// the file says either. Each is a few tens of kilobytes, and there are as many of them as somebody has kept.
///
/// What is left out is a replay with nothing to watch in it: a stage whose chapters have no attempt, and a
/// chapter with none, are a list that leads nowhere — and a chapter has none where the last write of the file
/// caught it on the frame it began on.
fn the_replays_to_watch(runtime: &mut Runtime) {
    let base = &runtime.config.base_dir;
    let mut replays = Vec::new();
    for name in replay::saved(base) {
        let Some(replay) = replay::read_saved(base, &name, runtime.game) else {
            continue;
        };
        let stages: Vec<replay_ui::ListedStage> = replay
            .stages
            .iter()
            .map(|stage| replay_ui::ListedStage {
                // As somebody counts them, which is one more than the run does.
                stage: stage.stage + 1,
                chapters: stage
                    .chapters
                    .iter()
                    .map(|chapter| replay_ui::ListedChapter {
                        number: chapter.number,
                        name: chapter.name.clone(),
                        attempts: (0..stage.tries(chapter))
                            .filter_map(|which| {
                                let attempt = stage.attempt(chapter, which)?;
                                Some(replay_ui::ListedAttempt {
                                    frames: attempt.buttons.len() as u32,
                                    // The last of a chapter's is the one the run went on from, every other
                                    // having been rewound away — see `replay::Stage::tries`.
                                    survived: which + 1 == stage.tries(chapter),
                                })
                            })
                            .collect(),
                    })
                    .filter(|chapter| !chapter.attempts.is_empty())
                    .collect(),
            })
            .filter(|stage| !stage.chapters.is_empty())
            .collect();
        if stages.is_empty() {
            continue;
        }
        // Which run it was, as the game names one — and not the file's own name, which is that same slot with
        // the day it was kept on after it: the line already says the day.
        let slot = runtime
            .game
            .run_slot(&replay.run)
            .unwrap_or_else(|| name.clone());
        // **What the first column says**: whose orb this is for a run that was played out, and that it was
        // interrupted for one that was not — a name being what a run *ends* with in this game, so a name
        // against a run somebody walked away from is a name from nowhere. A file written before orb recorded
        // which of the two it was says neither, and keeps the name it always showed.
        let whose = match replay.ended {
            Some(replay::Ended::Interrupted) => replay_ui::interrupted(runtime.language),
            Some(replay::Ended::Finished) | None => &replay.name,
        };
        replays.push(replay_ui::Listed {
            said: replay.describe(&slot, whose),
            started: replay.started,
            name,
            stages,
        });
    }
    log!("replay: {} replay(s) to watch", replays.len());
    // **Not up yet: it waits for the screen it is to stand over**, which is the game's own — see [`ToPutUp`].
    // The press that would have taken the game there is the one orb held back for its question, so the screen
    // is asked for instead, and the list goes up when it is there.
    runtime.to_put_up = Some(ToPutUp {
        list: replay_ui::WatchMenu::new(replays, runtime.language),
        came_back: false,
        asked: false,
        drawn: 0,
        waited: 0,
    });
}

/// Starts the run an attempt out of a replay is watched in, and puts that attempt in the way of it.
///
/// **The run's own three numbers are orb's to write here**, which they are nowhere else: every other run orb
/// puts a chapter back into was chosen on the game's own screens, and this one was chosen out of a file — so
/// the difficulty, the character and the shot come from the replay and the scene is written the way the shot
/// type select writes it. See [`Game::start_a_run`].
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread, between frames.
unsafe fn watch_an_attempt(
    runtime: &mut Runtime,
    chosen: &replay_ui::Watched,
    list: Option<replay_ui::WatchMenu>,
) {
    let replay_ui::Watched::Chosen {
        replay: name,
        stage: which_stage,
        chapter: which_chapter,
        attempt: which_attempt,
    } = chosen
    else {
        return;
    };
    let base = &runtime.config.base_dir;
    let Some(replay) = replay::read_saved(base, name, runtime.game) else {
        return log!("replay: {name} cannot be read now; nothing is watched");
    };
    // The three indices are the ones the list was built from, and the list was built from this same file a
    // moment ago — so a stage, a chapter or an attempt that is not there is a file somebody replaced while
    // the question was up.
    let Some(stage) = replay.stages.get(*which_stage) else {
        return log!("replay: {name} has no stage {which_stage}; nothing is watched");
    };
    let Some(chapter) = stage.chapters.get(*which_chapter) else {
        return log!("replay: that stage has no chapter {which_chapter}; nothing is watched");
    };
    let Some(attempt) = stage.attempt(chapter, *which_attempt) else {
        return log!("replay: that chapter has no attempt {which_attempt}; nothing is watched");
    };
    log!(
        "replay: watching attempt {} of {} at chapter {} of stage {}, {} frame(s) of it",
        which_attempt + 1,
        stage.tries(chapter),
        chapter.number,
        stage.stage + 1,
        attempt.buttons.len(),
    );
    // What the playback is handed is a chapter of a run, which is what [`resume`] plays back: the frames
    // below the attempt's own first are the path that reached it, and the numbers are the stage's own.
    let run = RunStart {
        difficulty: replay.run.difficulty,
        character: replay.run.character,
        shot_type: replay.run.shot_type,
        practice: false,
        stage: stage.stage,
    };
    let saved = resume::Kept {
        run,
        state: stage.state.clone(),
        at: attempt.at,
        chapter: chapter.number,
        name: chapter.name.clone(),
        // Nothing of this run is written down, so the count of rewinds is the one the status line shows
        // while the attempt is watched: what it was when the attempt was made.
        retries: 0,
        song: chapter.song,
        song_identity: chapter.song_identity,
        song_sounding: chapter.song_sounding,
        landing: chapter.landing.clone(),
        // **The path that reaches the attempt and then the attempt itself, as one list of frames.** The
        // playback reads its buttons out of exactly this, so the frames past the landing are the attempt's
        // and need nothing else put anywhere — see [`resume::watch`].
        buttons: stage.played[..upto(attempt.at, stage.played.len())]
            .iter()
            .copied()
            .chain(attempt.buttons.iter().copied())
            .collect(),
        // And the generator of the frames below the landing alone, which is as far as the playback is held
        // to the run: an attempt is watched rather than checked, so there is nothing to compare its own
        // frames against.
        generators: stage.generators[..upto(attempt.at, stage.generators.len())].to_vec(),
    };
    let frames = attempt.buttons.len() as u32;
    // **The front end's own item asked for the run where this game has one to ask**, and the press it reads
    // handed over with it: what a run written into being was missing is whatever else that item does — see
    // [`Game::front_end_starts_a_run`]. The run is registered a few frames later, which the playback waits
    // out the way a picked-up run's does.
    if unsafe { runtime.game.front_end_starts_a_run(&run) } {
        log!(
            "replay: the front end is asked for a run of stage {}",
            saved.run.stage + 1
        );
        // **And the press handed over rather than swallowed**, this one being the press that item reads: what
        // is fed is a fresh edge, which is what the front end acts on — `swallow_input` writes every button
        // down, so a press fed after it reads as a button that was already held and the item never fires.
        FEED_DECIDE.store(true, Ordering::Relaxed);
    } else {
        if !unsafe { runtime.game.start_a_run(&run) } {
            return log!(
                "replay: the game did not take a run of {}",
                saved.run.stage + 1
            );
        }
        // Nothing of the press is handed over: the item the lists came up over is one the game never acted
        // on, and what follows is a run orb wrote into being.
        unsafe { runtime.game.swallow_input() };
    }
    let until = saved.at + frames;
    if !unsafe { resume::watch(runtime.game, saved, frames) } {
        return;
    }
    // The list is what the attempt was chosen out of, so there is always one here; a watch asked for by
    // anything but that list would be one nothing puts back up.
    let Some(list) = list else {
        return log!("replay: nothing chose this attempt; there is no list to come back to");
    };
    runtime.watching = Some(WatchedAttempt {
        until,
        list,
        keys: Keys::new(STOP_GRACE_FRAMES),
    });
    runtime.starting = 0;
    // Nothing of this run written down and no chapter of it kept, which is what the two flags say: see
    // `Runtime::chaptering`, and `resume::WATCHING` for the one thing that is still done — the numbers put
    // back and the buttons fed.
    resume::keep(false);
    replay::keep(false);
    // And out of the ranking, for as long as this lasts: what the run counts was counted once already by the
    // run the attempt came out of — see [`score::watching`]. Cleared where the list comes back, the write
    // being the last thing the screens a finished run ends at do on their way down.
    score::watching(true);
}

/// A frame number as an index into a list that may be shorter than it, which is a file saying a chapter is
/// further into a stage than the path it carries.
///
/// Held rather than refused, because what a short path costs is the frames it has not got: the playback
/// lands somewhere else and the line held against the reproduction written down says so, which is the check
/// that exists for exactly this.
fn upto(frame: u32, held: usize) -> usize {
    (frame as usize).min(held)
}

/// Puts the resume question up, where the run has a chapter of its own to ask about.
///
/// `cancels` says what a cancel does, which is what differs between the two frames this is asked on.
/// [`resume_ui::Cancels::TheRun`] is also what says a press is being held back for the question, so
/// where there turns out to be nothing to ask it is that press which has to be handed over: one held
/// back for a question that never appears is a screen that has stopped working.
fn ask_where_to_start(
    runtime: &mut Runtime,
    run: &RunStart,
    slot: &str,
    cancels: resume_ui::Cancels,
) -> bool {
    let asked = match resume::load(&runtime.config.base_dir, runtime.game, run) {
        // Not without the overlay, which is what would draw it: a frozen game with an invisible
        // question over it is a game that looks broken, and what is lost by not asking is the run
        // starting where the game was going to start it anyway.
        Some(_) if runtime.overlay.is_none() => {
            log!("resume: {slot} was left, but there is no overlay to ask over the game with");
            false
        }
        Some(saved) => {
            log!("resume: {slot} was left; asking where to start");
            let menu = resume_ui::ResumeMenu::new(saved.describe(), cancels, runtime.language);
            runtime.asking_resume = Some((menu, saved));
            true
        }
        None => {
            detail!("resume: nothing was left of {slot}");
            false
        }
    };
    if !asked && cancels == resume_ui::Cancels::TheRun {
        FEED_DECIDE.store(true, Ordering::Relaxed);
    }
    asked
}

/// Which replay this run is written into: the one it is picking up where an earlier session left it, or a
/// new one over whatever was being written before.
///
/// `picked_up` is the chapter the run was picked up at, and `None` is a run starting from the beginning —
/// which is where the replay of the run being written over is finished with, that being the moment nothing
/// more will ever be added to it. `before` says whether that is this call's to do or the answer to a question
/// already up; see [`replay::begun`].
///
/// **Settled where the resume question is asked or declined and nowhere else**, which is the press on the
/// game's own shot type select and the answer to what that press put up. Two things make it that frame. It
/// is the last one before a run is built, so a replay begun there is begun before anything of the run
/// happens; and it is the one frame *both* games say which run this is — the frame the run is registered on
/// does not, th07 keeping the difficulty a build is about to write where a run in progress keeps it, so a
/// replay settled there came out `easy-reimu-a` for a Normal run.
///
/// Which means a launch with no overlay keeps no replay: the press is not held back without one, so this is
/// not reached. That is the honest answer rather than a gap — without an overlay there is no screen to offer
/// a replay on and no list to watch one from, so what would be written is a file nobody could ever be asked
/// about or shown.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread, between frames.
unsafe fn the_replay_this_run_writes(
    runtime: &Runtime,
    run: &RunStart,
    slot: &str,
    picked_up: Option<&resume::Kept>,
    before: replay::TheOneBefore,
) {
    let base = &runtime.config.base_dir;
    let name = &runtime.name;
    let now = orb_api::clock::local_time();
    match picked_up {
        Some(saved) => unsafe {
            replay::picked_up(base, runtime.game, slot, name, saved, now);
        },
        None => unsafe { replay::begun(base, runtime.game, run, slot, name, now, before) },
    }
}

/// What answering it does, which is not the same at the two frames it can be asked on. The game is
/// what says which of them this is: on the frame a run was chosen a chapter can go in at once, and on
/// the press there is no run to put one into yet.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn answered(runtime: &mut Runtime, answer: resume_ui::Answer, saved: resume::Kept, by: By) {
    let slot = runtime.game.run_slot(&saved.run);
    // The run this was asked about, which is the one the file was found under: what the game says on this
    // frame is not it — see [`the_replay_this_run_writes`].
    let run = saved.run;
    let picked_up = match answer {
        resume_ui::Answer::Continue => {
            log!("resume: from where it stopped, answered on the {by}");
            Some(saved)
        }
        // The file is left where it is, and the first chapter this run reaches writes over it.
        resume_ui::Answer::Beginning => {
            log!("resume: from the beginning, answered on the {by}; {saved} is left behind");
            None
        }
        // Nothing to hand over and nothing to put back: the press that would have started the run is
        // the one this was asked on, and holding it back was the whole of it. What carries on
        // underneath is the shot type select, on the shot the question was about.
        //
        // Except for the key that cancelled, which is held back until it is let go: it is still down,
        // and what the shot type select does with it is go back to the character select — which is not
        // what somebody answering "neither" about this screen asked for.
        resume_ui::Answer::Cancelled => {
            log!("resume: neither, answered on the {by}; no run is started");
            HOLD_CANCEL.store(true, Ordering::Relaxed);
            return;
        }
    };
    if unsafe { runtime.game.run_chosen() } {
        if let Some(slot) = &slot {
            // **Kept without asking here, whatever the setting says.** This is the frame the run was
            // chosen on: the front end has taken its own job out and the next thing to run is the build,
            // so there is no screen left to put a question over and no press held back to hand on after
            // one. What the alternative would cost is a replay thrown away by a launch that could not ask
            // — see `the_replay_of_a_run_finished_with` for where the question does go.
            //
            // **Said only where something is being written over**, which is はじめから: a run picked up
            // carries its replay on and nothing is kept or lost. Left unqualified it was a launch on the
            // real game announcing it had kept, unasked, a file the very next line went on writing into.
            if picked_up.is_none()
                && !runtime.config.save_replays
                && replay::is_being_played(&runtime.config.base_dir, slot)
            {
                log!(
                    "replay: {slot} is being written over by a run chosen on this frame; it is kept \
                     without asking"
                );
            }
            unsafe {
                the_replay_this_run_writes(
                    runtime,
                    &run,
                    slot,
                    picked_up.as_ref(),
                    replay::TheOneBefore::Kept,
                )
            };
        }
        if let Some(saved) = picked_up {
            runtime.starting = 0;
            unsafe { resume::begin(runtime.game, saved) };
        }
        // The key that answered is not one the stage about to be built should act on — the same reason
        // as the mode question's, and here the frame the game carries on into is the one that
        // registers the run.
        unsafe { runtime.game.swallow_input() };
        return;
    }
    match slot {
        Some(slot) => {
            // **はじめから, which is where the replay of the run being written over is finished with**: the
            // chapter file this run's own first chapter writes over describes a run somebody played, and
            // nothing will ever be added to the replay of it. So it is asked about here, on the frame the
            // answer was given and while the press that starts the run is still held back — the question
            // that follows is answered over the same screen, and handing that press on is what it ends
            // with. Where nobody is asked, this run's own beginning is what keeps it.
            let before = match picked_up.is_none() && the_replay_being_written_over(runtime, &slot)
            {
                true => replay::TheOneBefore::Asked,
                false => replay::TheOneBefore::Kept,
            };
            // And the record this run writes into, started here rather than on the frame the run is
            // registered: this is the frame that says which run it is — see
            // [`the_replay_this_run_writes`]. Nothing of it is written until this run's first chapter, so
            // a question still up over the one before is a question with its own file still there.
            unsafe { the_replay_this_run_writes(runtime, &run, &slot, picked_up.as_ref(), before) };
            let asked = before == replay::TheOneBefore::Asked;
            runtime.started = Some(Started {
                slot,
                saved: picked_up,
            });
            if asked {
                return;
            }
        }
        // Not reachable through the question, `resume::load` having found the file under that very
        // name; said rather than unwrapped, an answer dropped being a run that starts from the
        // beginning.
        None => log!("resume: the run answered about has no slot; the answer is dropped"),
    }
    FEED_DECIDE.store(true, Ordering::Relaxed);
}

/// Puts the question about a replay being written over up, and says whether it went up.
///
/// Nothing where there is no replay of that run being written — the ordinary case, a run started over having
/// usually been started over before its replay was kept — and nothing where the setting says to keep every
/// one without asking, which is where [`replay::begun`] keeps this one as the run begins.
fn the_replay_being_written_over(runtime: &mut Runtime, slot: &str) -> bool {
    if runtime.config.save_replays || !replay::is_being_played(&runtime.config.base_dir, slot) {
        return false;
    }
    // Not without the overlay, which is what would draw it: a frozen game with an invisible question over
    // it is a game that looks broken, and what is lost by not asking is a replay kept that somebody may
    // not have wanted — against a launch that cannot start a run at all.
    if runtime.overlay.is_none() {
        log!(
            "replay: {slot} is being written over, and there is no overlay to ask over the game with"
        );
        return false;
    }
    log!("replay: {slot} is being written over; asking whether to keep it");
    runtime.asking_replay = Some((
        replay_ui::SaveMenu::new(runtime.language),
        slot.to_owned(),
        replay::Ended::Interrupted,
    ));
    HELD_FOR_THE_REPLAY.store(true, Ordering::Relaxed);
    true
}

/// Whether the press that starts a run is being held back for the question about a replay, which is what
/// says the answer has to hand it on.
///
/// The question is asked at two moments and only one of them has a press behind it: the screen a run ended
/// at has none — the run is over and the game is walking its own way out — where the answer given on the way
/// into a run is the one that lets that run start.
static HELD_FOR_THE_REPLAY: AtomicBool = AtomicBool::new(false);

/// Where a step left the game, when one happened.
enum Step {
    /// Not stepping: the frame runs as it would have.
    Carry,
    /// Held on a boundary. The game's update does not run this frame, and the draw
    /// puts up the frame it stopped on again.
    Held { reached: State, ran: i32 },
}

/// Stepping between chapter boundaries while a replay plays back.
///
/// What a midstage boundary has to be judged on is the frame it falls on, and at
/// eight updates to a drawn frame nothing drawn is within eight updates of one. A
/// step runs the updates with nothing drawn and holds the game on the frame it
/// arrives at, so that frame is the one on screen.
///
/// Only over a replay: a replay is what plays a whole run for a tuning pass, and
/// holding a run someone is playing is what the retry menu is for.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn step(runtime: &mut Runtime, chain: *mut c_void, state: &State) -> Step {
    // The run rather than the gameplay scene: a stage's end leaves that scene while the
    // game builds the next stage, and stopping there is the point.
    let watching = runtime.config.chapters
        && runtime.config.chapter_stepping
        && runtime.config.during_replay
        && state.replay
        && state.in_run
        && !state.demo
        && !state.paused;
    if !watching {
        runtime.held = false;
        runtime.walled = false;
        // The run being over is what puts this back, and not a pause or a menu: what it answers is
        // whether *this* run has been in a stage, which a run stopped inside one has.
        runtime.played &= state.in_run;
        runtime.pads.forget();
        return Step::Carry;
    }
    runtime.played |= state.playing;
    // The pads, on the frames a pass is watching and on no others: reaching the device the game holds costs
    // the game's own poll, a hundred microseconds of it, and nothing outside a pass reads these buttons.
    runtime.pads.poll(unsafe { pad(runtime.game) });
    // **Either hand**, the keys and the game's own buttons: a judging pass is watched with a controller in
    // hand, so the four directions step and judge and the buttons are the keys held with a step. Which
    // button is which is where the keys are named.
    let next = runtime.keyboard.pressed(keys::NEXT.0) || runtime.pads.pressed(|pad| pad.right);
    let previous =
        runtime.keyboard.pressed(keys::PREVIOUS.0) || runtime.pads.pressed(|pad| pad.left);
    let hold = runtime.keyboard.pressed(keys::HOLD.0) || runtime.pads.pressed(|pad| pad.skip);
    // Held rather than pressed: they say what the stepping keys mean, like a shift.
    let across = runtime.keyboard.held(keys::ACROSS.0) || runtime.pads.held(|pad| pad.focus);
    let dropped = runtime.keyboard.held(keys::DROPPED.0) || runtime.pads.held(|pad| pad.bomb);
    if next || previous {
        detail!(
            "step: {} pressed, across {}, dropped {}",
            if next { "next" } else { "back" },
            if across { "held" } else { "not held" },
            if dropped { "held" } else { "not held" },
        );
    }

    // The end of the stage: the game leaving the gameplay scene to build the next one,
    // which is well after the boss went down. Held there and kept held — carrying on is
    // what loses the stage, so neither the hold key nor a step forward moves — while a
    // step back still goes where it always goes. Nothing has been torn down yet: the
    // hold is what stops the update that would do it.
    //
    // **A stage this run has not been in is not one it has left**, which is the other way a
    // run reads as playing nothing: th07's is a run from the frame its front end asked for
    // one, and a wall on the way *in* is the same trap `leave` avoids on the way across — the
    // update it stops is the one that would build the stage it is waiting for.
    //
    // **And the fight a stage ends with being won is that stage's end as well**, which is the
    // only one of the two th07 ever gives: its supervisor does a whole transition inside one
    // update, so no frame of one is a frame outside the scene a run is played in and the wall
    // above has nothing to fire on — a pass over a replay of it played six stages through
    // without stopping. This one is inside the stage, where every chapter of it is still a
    // snapshot to go back into; once the next stage has loaded, what a snapshot of this one
    // names has been released and the way back is the game's to load.
    let ended = runtime.played && (!state.playing || runtime.chapters.stage_won());
    if ended {
        runtime.held = true;
        if !runtime.walled {
            runtime.walled = true;
            log!(
                "step: stage {} has ended; the across key with next or back leaves it",
                state.stage + 1,
            );
        }
    } else {
        runtime.walled = false;
    }

    // Moving between stages, which nothing else does.
    if across && (next || previous) {
        if let Some(step) = unsafe { leave(runtime, state, next) } {
            return step;
        }
        return if runtime.held {
            Step::Held {
                reached: *state,
                ran: CHAIN_BREAK,
            }
        } else {
            Step::Carry
        };
    }

    // A toggle rather than a resume, and not only on boundaries: stopping wherever
    // something looks like a gap is what puts `tuning_add_key` on an exact frame.
    if hold && !ended {
        runtime.held = !runtime.held;
        log!(
            "step: {} at chapter {} (script {})",
            if runtime.held { "held" } else { "playing on" },
            runtime.chapters.number(),
            state.script_frames,
        );
        if !runtime.held {
            return Step::Carry;
        }
    }
    // A boundary judged out of the table, which begins no chapter and so is nowhere in
    // the chapter starts the ordinary stepping moves between. Aimed at by the script
    // clock, the only thing that names it.
    if dropped && (next || previous) && !ended {
        let script = if next {
            runtime.chapters.next_dropped(state)
        } else {
            runtime.chapters.previous_dropped(state)
        };
        let Some(script) = script else {
            log!(
                "step: no boundary judged out {} script {}",
                if next { "after" } else { "before" },
                state.script_frames,
            );
            return if runtime.held {
                Step::Held {
                    reached: *state,
                    ran: CHAIN_BREAK,
                }
            } else {
                Step::Carry
            };
        };
        // Back the same way as any other back: the stage's start comes back and the
        // replay comes back with it, and the run forward from there lands on the frame.
        if previous && !unsafe { runtime.chapters.rewind_stage(runtime.game) } {
            log!("step: cannot go back — no stage start kept");
            return Step::Carry;
        }
        return unsafe {
            run_to(
                runtime,
                chain,
                Aim {
                    script: Some(script),
                    ..Aim::none()
                },
            )
        };
    }
    if next && !ended {
        let from = runtime.chapters.number();
        let at = runtime.chapters.next_start(state);
        return unsafe {
            run_to(
                runtime,
                chain,
                Aim {
                    at,
                    from: Some(from),
                    ..Aim::none()
                },
            )
        };
    }
    // Back is a restore and then the same thing again: a restore rewinds the replay
    // along with everything else, so the stage's start and a run forward from it land
    // exactly on the boundary asked for.
    if previous {
        // Read before the restore, which is what takes the stage's clock back.
        let behind = runtime.chapters.previous_start(state);
        if !unsafe { runtime.chapters.rewind_stage(runtime.game) } {
            log!("step: cannot go back — no stage start kept");
        } else if let Some(frame) = behind {
            let aim = Aim {
                at: Some(frame),
                ..Aim::none()
            };
            return unsafe { run_to(runtime, chain, aim) };
        } else {
            // The stage's own start is what lies before its first boundary, and the
            // restore has already arrived there.
            let reached = unsafe { runtime.game.read_state() };
            runtime.held = true;
            log!(
                "step: held at the stage's start (script {})",
                reached.script_frames
            );
            return Step::Held {
                reached,
                ran: CHAIN_BREAK,
            };
        }
    }
    if runtime.held {
        Step::Held {
            reached: *state,
            ran: CHAIN_BREAK,
        }
    } else {
        Step::Carry
    }
}

/// Asks the game to start the replay at the stage either side of this one, and reports
/// the frame to run when it will. `None` when there is no such stage, which leaves
/// whatever was going on alone.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn leave(runtime: &mut Runtime, state: &State, forward: bool) -> Option<Step> {
    let stage = if forward {
        state.stage + 1
    } else {
        state.stage - 1
    };
    if !unsafe { runtime.game.jump_to_stage(stage) } {
        log!("step: the replay has no stage {}", stage + 1);
        return None;
    }
    // Everything kept of the stage being left goes with the ask: what a snapshot of it names is released
    // by the load, and the chapters of a stage are none of the next one's.
    runtime.chapters.the_stage_was_left();
    // The game has to run for it to build that stage, so nothing is held.
    runtime.held = false;
    runtime.walled = false;
    Some(Step::Carry)
}

/// What ends a step: whichever of these comes first.
struct Aim {
    /// A chapter this stage has begun once already, by the stage frame it began at.
    /// Set going either way, so that a boundary judged out of the table — which no
    /// longer begins a chapter, and so no longer moves the number — is still a place
    /// both keys stop at.
    at: Option<u32>,
    /// The chapter the step began in, for a boundary nothing has recorded yet: a stage
    /// the first time through has none of them, and a boss's are never in a table.
    /// `None` going back, which has to run past every boundary between the stage's
    /// start and the one asked for.
    from: Option<u32>,
    /// A frame of the enemy timeline, for a boundary of the table that begins no chapter:
    /// one judged out is in no chapter start, and the clock it is written in is the only
    /// thing that names it.
    script: Option<i32>,
}

impl Aim {
    /// Nothing asked for, to be filled in with the one thing that is.
    fn none() -> Self {
        Self {
            at: None,
            from: None,
            script: None,
        }
    }

    fn reached(&self, state: &State, number: u32) -> bool {
        self.at.is_some_and(|frame| state.stage_frames >= frame)
            || self
                .script
                .is_some_and(|frame| state.script_frames >= frame)
            || self.from.is_some_and(|from| number != from)
    }
}

/// Runs updates until the aim is reached, then holds the game there. The ending
/// skip's mechanism aimed at a boundary rather than at the end of a scene: drawing
/// happens once per frame, so however many updates this runs, not one of them is
/// seen.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn run_to(runtime: &mut Runtime, chain: *mut c_void, aim: Aim) -> Step {
    let mut reached = unsafe { runtime.game.read_state() };
    let mut ran = CHAIN_BREAK;
    let mut frames = 0;
    while !aim.reached(&reached, runtime.chapters.number())
        && reached.playing
        // **And the stage's own fight not yet won**, which is where this stage ends and so where a step
        // inside it has to stop. `playing` is what stops it on th06 — the game leaves the gameplay scene
        // for the frames a transition takes — and th07 leaves it for none of them: its supervisor tears
        // the stage down, builds the next and puts the scene back inside one update, so a loop watching the
        // scene runs on into the stage after. A launch found it: `next` at the end of a fight came to rest
        // in stage 2, with the stage it was judging gone and no chapter of it left to go back into.
        && !runtime.chapters.stage_won()
        && !reached.paused
        && frames < STEP_LIMIT_FRAMES
        && ran != CHAIN_EXIT_SUCCESS
        && ran != CHAIN_EXIT_ERROR
    {
        ran = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        reached = unsafe { runtime.game.read_state() };
        unsafe {
            runtime.chapters.observe(
                runtime.game,
                &reached,
                &runtime.data,
                runtime.config.self_check,
            )
        };
        unsafe { reproduced(runtime, &reached) };
        frames += 1;
    }
    runtime.held = true;
    log!(
        "step: held in stage {} chapter {} at frame {} (script {}), {frames} update(s) run{}",
        reached.stage + 1,
        runtime.chapters.number(),
        reached.stage_frames,
        reached.script_frames,
        // **Said where the stop is what ended the run and not the target.** Without this the line reads as a
        // landing: a pass whose stop was below the Extra's length held the game in the middle of an attack and
        // said the same words it says on a boundary, so what looked like a step that worked was a step that ran
        // out of room.
        if frames >= STEP_LIMIT_FRAMES {
            " — the stop, not the chapter asked for"
        } else {
            ""
        },
    );
    Step::Held { reached, ran }
}

/// Plays a saved run's buttons into the stage the game has just built, until the frame the chapter
/// it was left in began on, and reports where that left the game.
///
/// The ending skip's mechanism aimed at a chapter: drawing happens once per frame, so however many
/// updates this runs, not one of them is seen. Every one of them is observed, though, which is
/// what leaves the run with the chapter's own snapshot to be sent back to and the stage divided
/// the way it was.
///
/// What the run lands on is then held against what was written down for that frame. Both lines go
/// to the log whatever comes of it: what a resume rests on is that the numbers a stage reads at its
/// start are all written down, and this is the instrument for that — see [`game::Reproduction`].
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
/// A track as a line says it, out of what [`Game::music_identity`] answers for it: a number its own record
/// makes, and nothing where the stream would not say — which a file written before the identity was kept has
/// as well.
fn the_track(identity: Option<u32>) -> String {
    match identity {
        Some(identity) => format!("{identity:#010x}"),
        None => "one the stream would not name".to_owned(),
    }
}

unsafe fn play_in(runtime: &mut Runtime, chain: *mut c_void, at: u32) -> (State, i32) {
    // **The music stopped for as long as this takes.** The stage's own song was started by the build a moment
    // ago and the buffer plays on its own thread, so the seconds of wall clock these updates take are seconds of
    // the stage's opening — and then the song jumps where it belongs as the run lands. A th07 session heard
    // exactly that on a run picked up in the first stage.
    //
    // Stopped and not taken down, because the landing is what puts it back: `play_from` below seeks the stream
    // this leaves alive, and a stream that had been deleted is one nothing could seek. Once here rather than per
    // update, this whole loop being one frame's work — nothing calls the game's own sound inside it, so nothing
    // starts a track again until it is over.
    if let Some(music) = runtime.game.music() {
        unsafe { music.silence() };
    }
    let mut reached = unsafe { runtime.game.read_state() };
    let mut ran = CHAIN_BREAK;
    let mut frames = 0;
    let deaths = reached.deaths;
    // The first frame the player was hit, where one was. The buttons written down are a path that
    // survived — every death a run had was rewound away by the retry menu, and the frames of the
    // attempt that did not survive were written over by the one that did — so a death here is a
    // playback that has come out of step, and the frame it happened on is where to look.
    let mut hit = None;
    // A bound on a clock that has stopped, which is the updates since it last moved and **not** a count
    // of them against the frame being aimed at.
    //
    // **An update of a stage is not a frame of it**, for two reasons that are not the same. The one a
    // session has shown: a playback whose run dies out stops advancing the clock and every update after
    // that is spent on nothing — `24179 update(s) run; stage 6 frame 5900`, of which 5959 played the run in
    // and eighteen thousand went by with the clock where it was. The one the exe shows: th07 slows its own
    // run down when the screen fills, 0x42e186 answering one update in five with the 3 that skips both
    // clocks, so a stage played through that takes as many updates again.
    let mut stalled = 0;
    while reached.stage_frames < at
        && reached.playing
        && stalled < STALLED_FRAMES
        && ran != CHAIN_EXIT_SUCCESS
        && ran != CHAIN_EXIT_ERROR
    {
        let was = reached.stage_frames;
        ran = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        reached = unsafe { runtime.game.read_state() };
        stalled = if reached.stage_frames > was {
            0
        } else {
            stalled + 1
        };
        // **How far it has got, every so often.** The whole playback is one frame of the game: nothing is drawn
        // and the window does not answer for as long as it takes, which for a chapter deep in a stage is
        // minutes — and from outside, a game that has stopped answering is a game that has hung. This is the
        // one thing that tells the two apart, so it is said at the level a run is ordinarily logged at and
        // said as it happens rather than in the frame's slack, there being no slack until it is over.
        if reached.stage_frames / SAID_EVERY != was / SAID_EVERY {
            log!(
                "resume: {} of {at} frame(s) played back, {frames} update(s) in, the generator at \
                 {:#06x}",
                reached.stage_frames,
                reached.random_seed,
            );
        }
        // **And every frame of it where something is being diagnosed**, which is the only way to hold a
        // playback to a run whose file has no generator of its own: the periodic state line a run writes
        // carries the generator too, so the log of the session that recorded the run holds one series and
        // this holds the other — and the frames those lines fall on are orb's own count rather than the
        // stage's, so a series taken every so often here would line up with that one on no frame at all.
        //
        // A write apiece, which for the deepest chapter of a stage is a few seconds added to a playback that
        // already takes tens of them. **At the verbose level and not behind `self_check`**, which is where it
        // was: that flag also has a chapter fingerprint every region it saved, and a snapshot that costs 2ms
        // costs 204ms with it — a stutter at every boundary, on a run somebody is playing to leave a file
        // worth picking up. What this costs is log and nothing else.
        if log::wanted(log::VERBOSE) && reached.stage_frames > was {
            log!(
                "resume: frame {} played back with {}{} updates={}",
                reached.stage_frames,
                unsafe { runtime.game.reproduction() },
                the_worlds_fingerprint(runtime.game),
                updates_taken(),
            );
        }
        unsafe {
            runtime.chapters.observe(
                runtime.game,
                &reached,
                &runtime.data,
                runtime.config.self_check,
            )
        };
        if hit.is_none() && reached.deaths > deaths {
            hit = Some(reached.stage_frames);
        }
        frames += 1;
    }
    // **Which of the four ways out it took**, because they are four different faults and the line below says
    // only where the run got to. A playback that stopped after one update was read as a stall, as a run that
    // died, and as a chain that had given up, and none of the three was written down anywhere — the frame it
    // reached is the same number whichever of them happened.
    log!(
        "resume: the playback stopped because {}",
        if reached.stage_frames >= at {
            "it reached the frame the chapter began on".to_owned()
        } else if !reached.playing {
            "the run stopped being one: the game is no longer playing a stage".to_owned()
        } else if stalled >= STALLED_FRAMES {
            format!("the stage's clock stood still for {stalled} update(s)")
        } else {
            format!("the game's own chain gave up on the frame: it answered {ran}")
        },
    );
    if let Some(frame) = hit {
        log!(
            "resume: the player was hit at frame {frame}, which a path that survived does not do; \
             the playback has come out of step there or before it"
        );
    }
    let arrived = unsafe { resume::landed() };
    // The captures back as they were before the buttons went in: playing a stage again starts every
    // card the run had passed, and the game counts an attempt — and a capture — at each of them. The
    // names those starts wrote stay, a name being what the playback learned rather than what it counted.
    if arrived.is_some() {
        let bytes = unsafe { resume::landed_captures(runtime.game) };
        if bytes != 0 {
            log!(
                "resume: {bytes} byte(s) of captures put back; the playback counts none of them and \
                 keeps the names it wrote"
            );
        }
        // And one attempt at the card the chapter is on, because picking a run up where it was left is
        // an attempt at that chapter — the same as the retry menu's, and the game counts neither: it
        // counts where a card starts, and this landing is inside one.
        if let Some(attempts) = unsafe { runtime.game.count_card_attempt() } {
            log!("resume: attempt {attempts} at this spell card");
        }
    }
    let Some(landing) = arrived else {
        return (reached, ran);
    };
    // The rewinds the run had already cost are part of what it is, and the status line's count
    // carries on from them rather than starting again at none.
    runtime.chapters.carry_retries(landing.retries);
    // The file already describes the chapter the run is standing in, so nothing writes it again —
    // and a run that goes on to finish from here still has that file taken away.
    runtime.kept = Some((reached.stage, landing.at));
    // Nothing about the frame the question was frozen on is worth comparing against this one. The
    // count of deaths above all: the run's own is in place now, and against the count the front end
    // was left holding it reads as a death on the frame the resume landed.
    runtime.previous = None;
    let landed = unsafe { runtime.game.reproduction() };
    // The score is not on this line, the reproduction lines under it carrying it: what a game keeps a score
    // in is that game's own field and this is every game's summary — see `Game::reproduction`.
    log!(
        "resume: {frames} update(s) run; stage {} frame {} (script {}) in chapter {} ({}), \
         written down as chapter {} ({}) at frame {}; lives={} bombs={} power={}",
        reached.stage + 1,
        reached.stage_frames,
        reached.script_frames,
        runtime.chapters.number(),
        runtime
            .chapters
            .name()
            .map_or_else(|| "none yet".to_owned(), |name| name.to_string()),
        landing.chapter,
        landing.name,
        landing.at,
        reached.lives,
        reached.bombs,
        reached.power,
    );
    match resume::differs(&landing.reproduction, &landed) {
        None => log!("resume: the landing is the frame that was written down, field for field"),
        Some(field) => {
            log!("resume: the landing is not the frame written down: {field}");
            log!("resume:   written {}", landing.reproduction);
            log!("resume:   landed  {landed}");
        }
    }

    // The song put where that chapter had it, which is what a retry of a chapter of the stage does
    // to the music — so a resume, which is the same chapter arrived at another way, owes the same.
    // What is playing at the landing is otherwise the track's opening milliseconds, the stage having
    // been built a frame ago.
    //
    // A boss's own theme is left alone, the way a fight's retry leaves it: starting it over every
    // attempt is worse than the jump in it. Which chapters those are is the same question the retry
    // asks, asked the same way — of the song playing and not of the chapter's kind, a midboss being a
    // fight the stage's song plays through.
    // **And only into the track that position was read from.** The file the game streams holds every track, so
    // the number written down points at the sound it was taken from and at nothing else once another track is
    // on the stream — which is what a stage's first chapter is picked up into: it falls frames into the stage,
    // the build has loaded the stage's two slots and its script has asked for neither, so the record under the
    // stream is still the front end's. A th07 session picked stage 6's first chapter up that way and heard the
    // fifth stage's boss theme, its file holding `song: 242508064` and the stream a track ending 16 MB in.
    let playing = runtime.game.music_identity();
    let its_own_track = landing.song_identity.is_some() && landing.song_identity == playing;
    // **And a chapter whose sound was nothing comes back to that nothing.** The position of a stopped stream
    // is where its sound *would* start, so putting it back and playing it makes a silence audible: a launch
    // picked stage 6's first chapter up — taken frames into the stage, the scene before it taken down and the
    // stage's own song not yet asked for — and heard the title screen's theme for the five seconds until that
    // script started its own. Which is the same thing the restore holds a chapter to, see `Saved::was_playing`.
    if runtime.chapters.stage_song_playing(runtime.game, &reached) {
        let moved = landing
            .song
            .zip(runtime.game.music())
            .is_some_and(|(song, music)| {
                // **A file that says nothing is put back the way the launch that wrote it put it back**,
                // which is every th06 file written before a chapter carried either of these: those runs have
                // had their midstage music picked up where it was left since orb shipped, and a guard that
                // refused what it cannot vouch for would take that away from every one of them on a disk.
                // What a file cannot say, orb cannot know better than the build that wrote it.
                //
                // So the two below hold a chapter that *does* say, and nothing else.
                let says = landing.song_sounding.is_some() || landing.song_identity.is_some();
                if !says {
                    log!(
                        "resume: the song at {song} is being put back with nothing said about its track or \
                         whether it was coming out, this file being from before either was kept"
                    );
                }
                if landing.song_sounding == Some(false) {
                    log!(
                        "resume: the song was left alone at {song}: nothing was coming out of it when the \
                         chapter was taken"
                    );
                    return false;
                }
                if says && !its_own_track {
                    log!(
                        "resume: the song was left alone at {song}: the track playing is {} and that \
                         position was read from {}",
                        the_track(playing),
                        the_track(landing.song_identity),
                    );
                    return false;
                }
                let moved = unsafe { music.play_from(song) };
                log!(
                    "resume: the song {} at {song} in the track's own file",
                    if moved {
                        "picked up"
                    } else {
                        "could not be put"
                    },
                );
                moved
            });
        // The chapter's snapshot was taken during the playback, with the song where the playback had
        // it — nothing, near enough. Left at that, the first death in this chapter would rewind the
        // music to the top and undo what was just put right, so the snapshot is taken again now that
        // the sound is where it belongs. It costs one copy of a chapter, on a frame that has just
        // run a stage's worth of updates.
        if moved {
            unsafe {
                runtime.chapters.retake(
                    runtime.game,
                    &reached,
                    &runtime.data,
                    runtime.config.self_check,
                )
            };
        }
    }
    (reached, ran)
}

/// **What a game says of where the things in its world are**, as the tail of a frame's own line — and nothing at
/// all from a game that has not been read for it.
///
/// Beside the reproduction line rather than inside it, for the reason `Game::the_worlds_fingerprint` gives: the
/// files hold that line and cannot hold this one.
///
/// # Safety
/// Must run on the game's main thread, with a stage running.
fn the_worlds_fingerprint(game: &dyn Game) -> String {
    match unsafe { game.the_worlds_fingerprint() } {
        Some(said) => format!(" {said}"),
        None => String::new(),
    }
}

/// Writes down where the run is at every chapter it reaches, which is what a later launch offers to
/// pick up.
///
/// At every one of them rather than where a session ends, because there is no moment a session ends
/// at: closing the window, the game being killed and a crash all leave nothing to write from. What
/// this costs is one file of a few tens of kilobytes per chapter, beside a snapshot of several
/// megabytes taken on the same frame.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn keep_chapter(runtime: &mut Runtime, state: &State) {
    // A run somebody is playing, and not one being played back into place: the chapters a playback
    // passes through are behind the one it is going to, and writing one of those down would move
    // the resume point backwards.
    if !runtime.keeping() || !state.in_game || unsafe { resume::landing() }.is_some() {
        return;
    }
    let Some(name) = runtime.chapters.name() else {
        return unsafe { keep_the_stages_start(runtime, state) };
    };
    let at = runtime.chapters.started_at();
    let chapter = (state.stage, at);
    if runtime.kept == Some(chapter) {
        return;
    }
    // Only from the frame the chapter began on, because what goes in the file with it is the
    // reproduction line for that frame and a frame further on is not that frame. A run at more
    // than one update to a drawn frame — a clear — passes chapters without ever standing on one.
    if state.stage_frames != at {
        return detail!(
            "resume: chapter {} began at frame {at} and the game is at {}; not written",
            runtime.chapters.number(),
            state.stage_frames,
        );
    }
    // Set whether or not the write comes off: what it would do again next frame is fail again,
    // and the line saying why has been written once.
    runtime.kept = Some(chapter);
    let written = unsafe {
        resume::write(
            &runtime.config.base_dir,
            runtime.game,
            at,
            runtime.chapters.number(),
            &name.to_string(),
            runtime.chapters.retries(),
        )
    };
    unsafe { the_replay_reaches(runtime, written.as_ref()) };
}

/// What becomes of the replay of a run that is finished with: the screen a run ends at, and the answer that
/// writes over the chapter a run was left in, are the two moments nothing more will ever be added to one.
///
/// Kept without asking where the setting says so, and otherwise offered — see [`replay_ui::SaveMenu`]. Off
/// is not "no replays": it is orb asking, which is where somebody says which of their runs were worth
/// keeping.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn the_replay_of_a_run_finished_with(runtime: &mut Runtime, slot: &str) {
    // The file and not the record in memory: by the time a run has reached this screen it is over, and the
    // record of it was written down and dropped where it ended — see [`replay::left`]. Which is the same
    // gate the question on the way into a run reads, that one having only ever had a file to go on.
    if !replay::is_being_played(&runtime.config.base_dir, slot) {
        return;
    }
    if runtime.config.save_replays {
        replay::keep_what_was_being_played(
            &runtime.config.base_dir,
            runtime.game,
            slot,
            &runtime.name,
            orb_api::clock::local_time(),
            replay::Ended::Finished,
        );
        return;
    }
    log!("replay: {slot} is finished with; asking whether to keep it");
    runtime.asking_replay = Some((
        replay_ui::SaveMenu::new(runtime.language),
        slot.to_owned(),
        replay::Ended::Finished,
    ));
}

/// And the same chapter into the run's own replay, which is what somebody watches this run from.
///
/// Handed what the chapter file wrote rather than reading the game again: the song's position and the
/// reproduction line are the chapter's own frame's, and a second read of them is a read off a game that has
/// moved on. Nothing where nothing was written, the two files being consistent about a run or neither being.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread, on the frame the chapter began on.
unsafe fn the_replay_reaches(runtime: &Runtime, written: Option<&resume::Kept>) {
    let Some(saved) = written else {
        return;
    };
    unsafe { replay::chapter(&runtime.config.base_dir, runtime.game, saved) };
}

/// **Writes the stage's own start down, before its first chapter has been announced.**
///
/// A stage's first chapter waits for the stage to settle and for the music to be ready — see `Chapters::due` —
/// and that wait is a second of a run in which nothing is written down at all. A session interrupted inside it
/// is picked up at the stage *before*, which is a whole stage of a run thrown away; and where the stage before
/// ended on a chapter of its own, the run **cannot get past the boundary at all**.
///
/// Which two launches of th07 measured. One played into the way into stage 2 and left the run for the title with
/// `resume: stage 2 begins with … power: 81, counted: 231` in its log and a file holding `stage 1 chapter 9`;
/// the next picked that up, played stage 1's boss again, reached the same window and left the same file — not one
/// write between the landing and the end of it, that chapter being stage 1's last. So the numbers are there and
/// nothing was writing them.
///
/// **The stage's first frame, and its first chapter's own name.** A stage's start is chapter 1 at frame 0 with
/// no frames to play back, whatever the stage holds — so nothing here is a guess about what the chapter model
/// will say once it settles: it is the same answer, written before the settle rather than after it. See
/// [`Chapters::what_a_stages_start_is_called`].
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn keep_the_stages_start(runtime: &mut Runtime, state: &State) {
    if !runtime.chapters.settling() {
        return;
    }
    // **Written once for the stage and not once for every frame of the wait**, which is what the frame this
    // marks it with says: nought is a frame no chapter of a stage ever begins on, the settle being frames long,
    // so it is the mark for *this* write and the chapter's own is still to come.
    let written = (state.stage, 0);
    if runtime.kept == Some(written) {
        return;
    }
    // **Tried on every frame of the wait until it takes**, and the earliest one it can is what a stage's start
    // is written from. What a chapter carries is the reproduction line of the frame it began on and the buttons
    // of the frames before it, and on the frame a stage is *noticed* the record holds neither — it was emptied
    // when the stage was built, and the recorder notes a frame after it has run. Measured: a launch's first
    // attempt came back `resume: not written — 0 frame(s) of buttons for a chapter at 1`. So the frames of the
    // wait are attempted in turn and the first that is self-consistent is the one that lands, which is a stage's
    // start a frame or two in rather than a file written from numbers that do not go together.
    let wrote = unsafe {
        resume::write(
            &runtime.config.base_dir,
            runtime.game,
            state.stage_frames,
            1,
            &Chapters::what_a_stages_start_is_called().to_string(),
            runtime.chapters.retries(),
        )
    };
    if wrote.is_some() {
        runtime.kept = Some(written);
    }
    unsafe { the_replay_reaches(runtime, wrote.as_ref()) };
}

/// Starts the flash when the game comes to rest on a boundary it was not on before.
///
/// The boundary rather than the chapter number, because numbering starts again at every
/// stage: moving between stages lands on chapter 1 of the next one, which is a boundary
/// reached and would otherwise pass unmarked. Set here rather than where a step ends, so
/// that a boundary the game runs past while it is being held under `space` is marked as
/// well as one a step stops on.
fn boundary_reached(runtime: &mut Runtime, state: &State) {
    let boundary = (state.stage, runtime.chapters.started_at());
    if runtime.flashed == Some(boundary) {
        return;
    }
    runtime.flashed = Some(boundary);
    let watching = if runtime.config.chapter_stepping {
        Watching::Judge
    } else if state.in_game {
        Watching::Player
    } else {
        Watching::Nobody
    };
    if let Some(flash) = flash_for(
        watching,
        runtime.config.boundary_flash,
        runtime.chapters.cause(),
    ) {
        runtime.flash = Some((flash, flash.frames));
    }
}

/// Who is waiting to see a boundary land, which is what decides the wash it gets.
#[derive(Clone, Copy, PartialEq)]
enum Watching {
    /// A judging pass, where the wash is what the pass is run with.
    Judge,
    /// A run somebody is playing, where a boundary says where dying will send them.
    Player,
    /// A collecting pass, or a replay being tracked for anything else: sixty-four updates to
    /// a drawn frame and nobody at the keyboard.
    Nobody,
}

/// Which wash a boundary gets, and whether it gets one at all.
fn flash_for(watching: Watching, wanted: bool, cause: Cause) -> Option<Flash> {
    // Not the stage's own start. A stage beginning is already unmistakable — the title, the
    // music, the field empty — and a wash over the first frame of one says nothing that was
    // not already obvious.
    if cause == Cause::StageStart {
        return None;
    }
    match watching {
        // Whichever way the setting is left: it is about a run somebody is playing, and a
        // pass with no wash is a pass with nothing to judge a boundary by.
        Watching::Judge => Some(FLASH_JUDGING),
        Watching::Player => wanted.then_some(FLASH_PLAYING),
        // A collecting pass crosses a boundary every few drawn frames, so a wash there is a
        // green field for twenty minutes and a mark to nobody.
        Watching::Nobody => None,
    }
}

/// A line per update of a replay's stage, so that two passes over one stage can be held
/// against each other frame for frame. What a desynchronised replay looks like is the
/// player being hit where the recording was not, and the first frame these numbers
/// differ on says which of the replay's clock, its inputs, the player and the game's
/// generator went first.
///
/// Only while stepping, since that is the pass with somebody watching: a collecting pass
/// over a whole run would write a line per update of it.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn reproduced(runtime: &Runtime, state: &State) {
    // Nothing while the game is not moving: a menu holds one frame for as long as it is
    // open, and a line per drawn frame of it would be the same line hundreds of times —
    // which is not only noise but pushes everything after it out of step with the pass
    // being compared against.
    if !runtime.config.chapter_stepping
        || !state.replay
        || state.demo
        || !state.in_run
        || state.paused
        || !log::wanted(log::VERBOSE)
    {
        return;
    }
    let game = runtime.game;
    // The update that runs while the game is building a stage, where the replay's own
    // clock does not exist yet and the player and the play field read as zeroes. There is
    // nothing there to hold against another pass, and which field says so is the game's —
    // see `Game::mid_build`.
    if unsafe { game.mid_build() } {
        return;
    }
    let reproduction = unsafe { game.reproduction() };
    detail!(
        "sync: stage={} frame={} script={} {} lives={} bombs={} power={} deaths={} \
         enemies={} bullets={} lasers={} attack={} spell={}{}{}",
        state.stage + 1,
        state.stage_frames,
        state.script_frames,
        reproduction,
        state.lives,
        state.bombs,
        state.power,
        state.deaths,
        state.enemy_count,
        state.bullet_count,
        state.laser_count,
        // What a chapter of a fight is derived from, next to the two things that stop one
        // being started: whether a bomb moves either of these is the question of whether a
        // bomb has to stop one at all.
        state.boss_attack_frames.unwrap_or(-1),
        state.spellcard.map_or(-1, |spell| spell as i32),
        if state.bombing { " bombing" } else { "" },
        if state.unsettled { " unsettled" } else { "" },
    );
}

/// The device setup the game does inside its own update, for a frame where that
/// update is being held back.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn hold_frame(game: &dyn Game) {
    let device = unsafe { game.d3d_device() };
    if !device.is_null() {
        unsafe { game.set_play_viewport(device) };
    }
}

/// **Notices the name the game's own entry holds**, which is the name the replays this launch writes carry.
///
/// Read every frame rather than on the frame that screen is left: what the entry holds is what somebody has
/// typed so far, and the screen goes down on a press orb does not see coming. The read costs a field of the
/// game's own on the frames that screen is up and one comparison on every other — see
/// [`Game::the_name_its_own_entry_holds`], which answers nothing off it.
///
/// **And kept in the settings file, which is what carries it to the next launch.** A run given up at orb's own
/// menu never reaches a name entry, so a launch spent grinding one chapter would otherwise write its replays
/// under no name at all; what the game itself does there is come up holding the name the last run was entered
/// under, and this is the same thing said in a file.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn notice_the_name(runtime: &mut Runtime) {
    let held = unsafe { runtime.game.the_name_its_own_entry_holds() };
    let Some(name) = held else {
        return;
    };
    if name == runtime.name {
        return;
    }
    log!("replay: the game's own name entry says {name:?}; that is whose the replays are now",);
    runtime.name = name;
    let written = orb_config::remember_the_name(&runtime.config.base_dir, &runtime.name);
    if let Err(error) = written {
        log!(
            "replay: cannot keep {:?} in orb.yaml: {error}",
            runtime.name
        );
    }
}

/// **What is coming out of the speakers, said once per change of it**: which track, whether its buffer is
/// playing, and where in the file the sound audible now begins.
///
/// **Which track is sounding is what no line about the music said.** Every one of them was a position, a
/// cursor or a countdown, so a launch that put one track's position into a stream playing another said so in
/// numbers indistinguishable from the right ones — a th07 session picked stage 6's first chapter up, heard the
/// fifth stage's boss theme, and its log had `music: the track loops at 16545584, so 0 byte(s) left from
/// 242508064` and nothing else. A track named, or told apart by its identity, is what makes that a line a
/// reader can hold the sound to.
///
/// **At `normal`, because a recipe is checked against it**: an autoplay launch that walks a run into the
/// stage after a pick-up is only worth running if what it heard is in the log, and `verbose` is where the
/// per-frame sound lines already are. One line per change of the track is a handful of lines a stage.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn notice_the_track(runtime: &mut Runtime) {
    let identity = runtime.game.music_identity();
    let music = runtime.game.music();
    let sounding = music
        .as_ref()
        .is_some_and(|music| unsafe { music.sounding() });
    if runtime.heard == Some((identity, sounding)) {
        return;
    }
    runtime.heard = Some((identity, sounding));
    log!("sound: {}", unsafe {
        crate::audio::what_is_heard(runtime.game)
    });
}

/// Watches the stream being fed: the margin, which is what a listener hears as the music breaking up once
/// it runs out, and the refills the streaming thread owes the mixer, which is what says a chunk was missed
/// for good before anything is heard.
///
/// **Every frame of a run, and not only while something is being diagnosed**, which is where the margin used
/// to be read: a stream a chunk out of step stays out of step for as long as it lives, and the run it was
/// found in had `audio: behind 0..0 bytes` — no reading at all — in its log for the whole of the twenty minutes
/// it played on. Each frame's cost is one `GetCurrentPosition` and a word read out of the game's memory.
///
/// **The stream is put back before the fault is heard, and only where the game is known to leave it.** th06's
/// ring holds four chunks, so the first missed refill is inaudible and the fourth writes under the mixer; up
/// to the third, [`Music::audible_offset`] is still exact, since the chunks between the cursor and the next
/// write are the file's in order, and [`Music::play_from`] from there is a fresh stream with a full buffer
/// ahead of the cursor and a gap of the call's own length in the sound. At the fourth the three numbers that
/// offset is made of have been moved by what was lost, and the song would jump by up to that. So the first
/// missed refill is acted on, where the repair is exact, rather than the fault that can be heard, where it is
/// not — and `Game::a_missed_refill_is_never_made_up` says which game's thread leaves a missed refill missed.
unsafe fn watch_music(runtime: &mut Runtime, state: &State) {
    if !state.playing {
        return;
    }
    let Some(music) = runtime.game.music() else {
        runtime.refills = None;
        return;
    };
    let identity = runtime.game.music_identity();
    if runtime.stream != (music.stream, identity) {
        log!(
            "audio: stream {:#010x} track {:?} (was {:#010x} track {:?})",
            music.stream,
            identity,
            runtime.stream.0,
            runtime.stream.1,
        );
        runtime.stream = (music.stream, identity);
        runtime.refills = None;
    }
    if crate::audio::was_repositioned() {
        runtime.refills = None;
    }
    let Some((play, write)) = (unsafe { music.cursor_and_next_write() }) else {
        runtime.refills = None;
        return;
    };
    let behind = (play + music.buffer_size - write) % music.buffer_size;
    runtime.margin_worst = runtime.margin_worst.max(behind);
    runtime.margin_best = runtime.margin_best.min(behind);
    if runtime.margin_trace > 0 {
        runtime.margin_trace -= 1;
        log!(
            "audio: behind={behind} of chunk {} buffer {}",
            music.notify_size,
            music.buffer_size,
        );
    }
    let Some(refills) = runtime.refills.as_mut() else {
        runtime.refills = crate::audio::Refills::from(&music, play, write);
        return;
    };
    let Some(missed) = refills.note(play, write) else {
        return;
    };
    log!(
        "audio: {missed} refill(s) missed for good: the next write is {behind} byte(s) behind the cursor, \
         chunk {} of a {}-byte buffer",
        music.notify_size,
        music.buffer_size,
    );
    if !runtime.game.a_missed_refill_is_never_made_up() {
        return;
    }
    let Some(offset) = (unsafe { music.audible_offset() }) else {
        return log!(
            "audio: the stream was left alone: it will not say where the sound audible begins"
        );
    };
    log!(
        "audio: the stream {} at {offset} in the track's own file",
        if unsafe { music.play_from(offset) } {
            "put back"
        } else {
            "could not be put back"
        },
    );
    runtime.refills = None;
}

/// Restores the current chapter on a timer, so the snapshot and the music get
/// exercised as many times as a long session would without anyone playing one.
unsafe fn stress(runtime: &mut Runtime, state: &State) {
    let interval = runtime.config.stress_restore_frames;
    // Not while the game is held on a boundary: a restore on a timer would take it
    // off the frame someone is looking at.
    if interval == 0 || !state.playing || state.unsettled || state.paused || runtime.held {
        return;
    }
    // Each chapter gets a few goes and is then left alone, so the run walks on
    // through the midstage, the midboss and every one of a boss's attacks.
    let chapter = (state.stage, runtime.chapters.number());
    if runtime.stressing != chapter {
        runtime.stressing = chapter;
        runtime.stressed = 0;
    }
    if runtime.stressed >= STRESS_PER_CHAPTER || !runtime.frames.is_multiple_of(interval) {
        return;
    }
    runtime.stressed += 1;
    if unsafe { runtime.chapters.retry_chapter(runtime.game) } {
        runtime.previous = None;
        // Watch what the restore did to the streaming, frame by frame.
        runtime.margin_trace = 12;
    }
}

/// Hand corrections to the midstage table while playing. The table is written out
/// by itself at the end of every stage; these are for a boundary the detector puts
/// in the wrong place, or misses.
fn tune(runtime: &mut Runtime, state: &State) {
    let add = runtime.keyboard.pressed(keys::ADD.0) || runtime.pads.pressed(|pad| pad.shot);
    let keep = runtime.keyboard.pressed(keys::KEEP.0) || runtime.pads.pressed(|pad| pad.up);
    let drop = runtime.keyboard.pressed(keys::DROP.0) || runtime.pads.pressed(|pad| pad.down);

    // These go through `Chapters`, which is what knows whether the frame the game is
    // standing on has a boundary of the table's: a boss's boundaries are the game's own
    // and there is nothing about them to write down.
    //
    // The hold goes with them because judging is only allowed while the game is standing
    // on the boundary. Adding is not: a gap the detector missed is caught by pressing at
    // the moment the stage passes through it, which is with the game running.
    let held = runtime.held;
    if add {
        unsafe {
            runtime.chapters.add_boundary(
                runtime.game,
                state,
                &runtime.data,
                runtime.config.self_check,
            )
        };
    }
    if keep {
        runtime.chapters.judge(state, held, Judgement::Better);
    }
    if drop {
        runtime.chapters.judge(state, held, Judgement::Worse);
    }

    // Written out as soon as anything is decided, not only at the end of the stage:
    // a session that is closed in the middle of one — which is how looking at a few
    // boundaries and stopping goes — would otherwise lose everything it decided.
    // Two files of a few hundred bytes, and only on a press.
    //
    // **On every decision and on nothing else.** A key of its own for writing them was here and went: what
    // it could write that a decision does not is the detector's own proposals for the stage in hand, which
    // a stage's end writes anyway — so all it was ever for was a pass about to be closed mid-stage, and it
    // was never once pressed.
    if (add || keep || drop)
        && let Some(tuning) = runtime.chapters.tuning()
    {
        tuning.write(runtime.game);
    }
}

/// What has to be asked of the game before it draws, rather than after: the row the mark over the
/// lives goes on, repainted with its count, so that the stars show through where the ink is dry and
/// nothing of orb's accumulates on a panel the game otherwise leaves alone.
///
/// # Safety
/// Only ever called from the draw hook, before the game's own drawing, on its main thread.
unsafe fn before_draw() {
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return;
    };
    // Decided here and kept, so the ask and the mark cannot disagree about a frame — and so that
    // the answer is the one the next frame's decision is carried past the end of a run by.
    let marking = marking(runtime);
    runtime.marked = marking != Marking::No;
    if runtime.marked {
        unsafe { runtime.game.repaint_lives_row() };
    }
    // How long a panel outlives its run, said where it ends: that number is the whole of whether
    // the mark stops a frame before the painting does, and half a second of fade to the title is
    // not something to count by eye.
    if marking == Marking::PanelLeft {
        runtime.marked_after += 1;
    } else if runtime.marked_after > 0 {
        log!(
            "lives: the mark stayed on the panel for {} frame(s) after the run ended",
            std::mem::replace(&mut runtime.marked_after, 0)
        );
    }
}

/// Whether the lives are being marked as disabled this frame.
///
/// Three things: the mode, a chapter there is a snapshot to go back to, and a run somebody is
/// playing — not a demo or a replay, which have no menu offered to them. On the frames before the
/// stage's own snapshot exists a death does cost a life, and a mark saying otherwise would be
/// wrong about exactly the frames it matters.
///
/// **Asked of the run and not of the frame**, which is `runtime.previous`, and which was wrong
/// about the mark twice over. It is the gameplay scene alone, so it does not hold the frame a stage
/// transition is built in — one frame of the log and a visible stretch of screen, `f44096
/// scene=3 stage=2` and then `f44097 scene=2 stage=3 frames=1`, the game building the next stage
/// inside it and every transition of that run the same. And it is dropped outright wherever an update has put the game
/// somewhere that has nothing to do with the frame it froze on — a chapter put back, a resume
/// landed — which are frames in the middle of a run and drawn like any other. On both the mark went
/// off, and the game paints that row from its own "this row changed" bits without asking orb, so
/// what stood there instead was the count.
///
/// **And then the frames the run's panel outlives the run**, for as long as the game is still
/// painting that row: leaving a run ends it on one frame and leaves its panel on the screen until
/// the front end has drawn its own, and the row the game paints last is the one left standing
/// there. `Game::draws_lives_row` is that question, and it is what ends the mark rather than the
/// run ending — so the mark cannot stop a frame before the painting does, and cannot go on to be
/// drawn over a screen that is no longer the panel.
///
/// The death itself is still answered on `in_game`, which is where it belongs: it is a comparison
/// against the frame before, and nothing can be hit on a frame the game spends building a stage or
/// walking back to its title.
fn marking(runtime: &Runtime) -> Marking {
    if runtime.chaptering() && runtime.chapters.can_retry() && runtime.chapters.somebody_playing() {
        return Marking::Run;
    }
    if runtime.marked && unsafe { runtime.game.draws_lives_row() } {
        return Marking::PanelLeft;
    }
    Marking::No
}

/// Which of the two counts the mark over the lives is drawn on, where it is drawn at all.
#[derive(Clone, Copy, PartialEq)]
enum Marking {
    /// A frame of the run itself.
    Run,
    /// A frame after the run has ended, with the game still painting that row: the panel a run
    /// leaves on the screen for as long as the front end takes to draw over it.
    PanelLeft,
    No,
}

/// # Safety
/// Only ever called from the draw hook, between the game's `BeginScene` and
/// `EndScene`, on the game's main thread.
unsafe fn after_draw() {
    let started = profile::now();
    unsafe { draw_overlay() };
    unsafe { profile::record(profile::Phase::Draw, started) };
}

/// # Safety
/// Only ever called from the draw hook, between the game's `BeginScene` and
/// `EndScene`, on the game's main thread.
unsafe fn draw_overlay() {
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return;
    };
    if runtime.overlay.is_none() && runtime.overlay_attempts > 0 {
        let device = unsafe { runtime.game.d3d_device() };
        if device.is_null() {
            return;
        }
        runtime.overlay_attempts -= 1;
        runtime.overlay = unsafe {
            Overlay::new(
                device,
                &runtime.config.game_dir.join("font.ttf"),
                runtime.game.own_face(),
                FONT_HEIGHT,
                MARK_FONT_HEIGHT,
            )
        };
        // Said when it is there and when the last try has gone, and nothing in between: what
        // each try itself could not do is already a line of its own.
        match (runtime.overlay.is_some(), runtime.overlay_attempts) {
            (true, _) => log!("overlay: ready"),
            (false, 0) => log!("overlay: unavailable"),
            (false, _) => {}
        }
    }
    let Some(overlay) = &runtime.overlay else {
        return;
    };

    // The game's count of lives painted over, where dying costs the chapter and not one of them.
    // Before the menus, because the mark belongs to the panel and a menu goes up over that.
    //
    // What `before_draw` decided for this frame, not the question asked again: the ask it made of
    // the game has already been spent by the game's own drawing, and a frame where one of the two
    // happened without the other is either a count with nothing over it or ink over a row nobody
    // repainted.
    if runtime.marked {
        let row = runtime.game.lives_row();
        let room = runtime.game.room_around_the_lives_row();
        let panel = unsafe { runtime.game.panel_tile() };
        unsafe { runtime.lives.draw(overlay, row, room, panel) };
    }

    if let Some(menu) = &mut runtime.retry {
        let area = runtime.game.play_area();
        // The chapter by name, which is what the menu is offering to put the player back at —
        // and by number in a pass building the table, where every number on screen is one to
        // hold against the log.
        //
        // The chapters behind it were settled where the menu went up: the game is frozen under it, so
        // what the stage has cannot change while it is being read — see `RetryMenu::new`.
        let chapter = match runtime
            .chapters
            .name()
            .filter(|_| !runtime.config.chapter_tuning)
        {
            Some(name) => name.to_string(),
            None => format!("CHAPTER {}", runtime.chapters.number()),
        };
        unsafe { menu.draw(overlay, area, &chapter, runtime.chapters.retries()) };
        return;
    }

    if let Some(asking) = &mut runtime.asking {
        unsafe { asking.draw(overlay) };
        return;
    }

    if let Some((asking, _)) = &mut runtime.asking_resume {
        unsafe { asking.draw(overlay) };
        return;
    }

    if let Some((asking, _, _)) = &mut runtime.asking_replay {
        unsafe { asking.draw(overlay) };
        return;
    }

    if let Some(choosing) = &mut runtime.choosing_replay {
        unsafe { choosing.draw(overlay) };
        return;
    }

    // Over the game's own screen and not instead of it: what it says is about the item the game's
    // cursor is on, and the screen carries on running underneath.
    let corner = runtime.game.mark_corner();
    unsafe { runtime.mark.draw(overlay, corner) };

    // Over the play field and no further. The rest of the game's output — the panel beside
    // it, the border around it — is not repainted every frame, so a wash drawn there is
    // not drawn over again and stays on the screen for good.
    //
    // Counted down in drawn frames rather than in the game's, because a step holds the game
    // on the frame a boundary falls on: its clock stops there and the flash would stop with
    // it.
    if let Some((flash, left)) = runtime.flash.take() {
        let area = runtime.game.play_area();
        let fading = flash.frames - flash.hold;
        let alpha = if left > fading {
            flash.alpha
        } else {
            flash.alpha * left / fading
        };
        if let Some(frame) = unsafe { overlay.frame() } {
            frame.fill(
                area.left,
                area.top,
                area.width,
                area.height,
                alpha << 24 | FLASH_COLOR,
            );
        }
        if left > 1 {
            runtime.flash = Some((flash, left - 1));
        }
    }
}

/// The line of numbers written below the game, in the black beside it.
///
/// Not part of the overlay, which draws into the game's own 640x480 back buffer — all of
/// that is shown inside the letterbox, so anything put there is over the game, and the
/// game does not clear it between frames so it also stays there. The black around the
/// game belongs to the window, and to orb.
///
/// # Safety
/// Must run on the game's main thread, outside a scene.
unsafe fn write_status(runtime: &mut Runtime) {
    let Some(state) = runtime.previous else {
        return;
    };
    // The lag is what the pacing costs, and it is only meaningful next to the rate it
    // buys: a low number with an uneven rate is not an improvement.
    //
    // Held still between refreshes so the numbers can be read at all.
    if runtime.frames.is_multiple_of(HUD_NUMBER_INTERVAL) {
        runtime.shown = unsafe { pacing() }.status();
    }
    let (lag, interval, compose) = runtime.shown;
    // A line each, because the black is usually bars down the sides of a widescreen
    // monitor rather than a strip under the game, and a strip that narrow takes one
    // short line at a time.
    //
    // Two different questions, and which is being asked depends on what the session is for.
    //
    // A run wants the chapter by name — which part of the stage it is, rather than a count of
    // the chapters gone by — since that is what says where dying will send the player and
    // whether the flash that just went off was expected.
    //
    // A pass building the table wants why the chapter changed *here*: the signal that produced
    // the boundary, and for the table's own the frame it is written down as and what has been
    // decided about it. Which is a name to nobody, and is what says whether the boundary
    // belongs in the table at all.
    let tuning = runtime.config.chapter_tuning;
    let mut lines = Vec::new();
    // Neither in normal mode, where there are no chapters: an empty name and a retry count that
    // cannot move are two lines saying that the run is not the one they describe.
    if runtime.chaptering() {
        if tuning {
            lines.push(format!(
                "CH {:02}  RETRY {}",
                runtime.chapters.number(),
                runtime.chapters.retries(),
            ));
        } else {
            if let Some(name) = runtime.chapters.name() {
                lines.push(name.to_string());
            }
            lines.push(format!("RETRY {}", runtime.chapters.retries()));
        }
    }
    lines.push(format!("INPUT LAG {}.{}ms", lag / 1000, lag % 1000 / 100));
    // Beside the lag rather than folded into it: this is the part of the lag orb chose, it is
    // the part that moves while the game runs, and watching it settle is watching the pacing
    // find how near the blank this display will take a frame.
    lines.push(format!(
        "COMPOSE {}.{}ms",
        compose / 1000,
        compose % 1000 / 100
    ));
    lines.push(format!(
        "{}.{}fps",
        1_000_000 / interval.max(1),
        10_000_000 / interval.max(1) % 10,
    ));
    // `HAND` marks the boundaries a person put there, those being the numbers nothing would
    // find again.
    //
    // The boundary a key would change comes first, and the chapter's own after it: the two
    // are the same thing wherever a step has stopped, and where they are not, what a key
    // is about to do outranks what is being played.
    if tuning {
        let judged = runtime
            .chapters
            .judged(&state, runtime.held)
            .or_else(|| runtime.chapters.chapter_boundary());
        lines.push(match judged {
            Some(judged) => format!(
                "{} {} {}",
                if judged.by_hand { "HAND" } else { "AUTO" },
                judged.frame,
                judged.verdict.label(),
            ),
            None => runtime.chapters.cause().label().to_owned(),
        });
    }
    // The script frame it would be written down as, and how many the stage's table has.
    if let Some(tuning) = runtime.chapters.tuning() {
        lines.push(format!("SCRIPT {}", state.script_frames));
        lines.push(format!("TABLE {}", tuning.count(state.stage)));
    }
    if runtime.held {
        lines.push("HOLD".to_owned());
    }
    // Held rather than painted: working the lines out is where the numbers are and costs nothing,
    // and painting them is GDI on the window and costs milliseconds. See `window::HELD`.
    unsafe { crate::window::hold_beside(lines) };
}

// Only on the target th06 and th07 are built for — see `game::th06c::tests`, which is what the other
// target's run of these is for.
#[cfg(all(test, target_arch = "x86"))]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::game::th06::Th06;

    use super::{
        Cause, DECIDE_PRESSED, DECIDE_WAS_DOWN, FEED_DECIDE, FLASH_JUDGING, FLASH_PLAYING, Game,
        HOLD_CANCEL, HOLD_DECIDE, SETTLE_KEYS, Watching, asked_for, flash_for, held_back, moved_on,
    };

    /// **Which name a lookup asked for**, which is what decides whether orb answers it: the one name it
    /// stands in front of and nothing that merely starts with it or is spelled another way.
    ///
    /// The one that matters is the third: `GetKeyboardState` is the start of no other name in `USER32`,
    /// but the comparison walks the wanted name's bytes and would say yes to any name beginning with
    /// them — so what says the name ends where it should is the byte after it.
    #[test]
    fn a_lookup_is_answered_by_orb_for_one_name_and_no_other() {
        for (asked, wanted, answered) in [
            (c"GetKeyboardState", "GetKeyboardState", true),
            (c"GetKeyboardLayout", "GetKeyboardState", false),
            (c"GetKeyboardStateEx", "GetKeyboardState", false),
            (c"GetKeyboard", "GetKeyboardState", false),
            (c"", "GetKeyboardState", false),
        ] {
            assert_eq!(
                unsafe { asked_for(asked.as_ptr().cast(), wanted) },
                answered,
                "a lookup of {asked:?} against {wanted}",
            );
        }
    }

    /// What the game reads while orb is holding a press of its own back: the buttons it does read are
    /// its own, the decide is gone from every frame of the holding, it is the press that is reported
    /// and not each frame the button is down, the key a question was cancelled with reaches nothing
    /// until it is let go, and no other key held on the question reaches the read the game carries on
    /// with.
    ///
    /// One test for the sequence rather than one per step. Each read's answer depends on the reads
    /// before it, and what they are kept in is statics of the process — so two tests of this would be
    /// two tests writing each other's state as the harness ran them side by side. Every one of them is
    /// put where this test wants it first, for the same reason.
    #[test]
    fn a_press_held_back_is_reported_once_and_reaches_the_game_only_when_handed_over() {
        // th06's own bits, this being about which of them a read hands over and which are kept: the
        // holding is any game's and the words it is done to are one game's.
        let game = &Th06;
        let decide = game.menu_decide();
        let other = 0x20;
        HOLD_DECIDE.store(false, Ordering::Relaxed);
        DECIDE_PRESSED.store(false, Ordering::Relaxed);
        DECIDE_WAS_DOWN.store(false, Ordering::Relaxed);
        FEED_DECIDE.store(false, Ordering::Relaxed);
        HOLD_CANCEL.store(false, Ordering::Relaxed);
        SETTLE_KEYS.store(false, Ordering::Relaxed);

        // Nothing held back: the word is the game's own, whatever is in it.
        assert_eq!(held_back(game, decide | other), decide | other);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // Held back, with the button still down from the press that got the game to this screen. Not a
        // press: nobody has asked for anything here yet, and reading it as one puts the question up on
        // the screen's own first frames.
        HOLD_DECIDE.store(true, Ordering::Relaxed);
        assert_eq!(held_back(game, decide), 0);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // Still down, and still nothing.
        assert_eq!(held_back(game, decide | other), other);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // Let go and pressed: that is the press the question goes on.
        assert_eq!(held_back(game, 0), 0);
        assert_eq!(held_back(game, decide), 0);
        assert!(DECIDE_PRESSED.swap(false, Ordering::Relaxed));

        // Let go and pressed again, which is a second question.
        assert_eq!(held_back(game, 0), 0);
        assert_eq!(held_back(game, decide), 0);
        assert!(DECIDE_PRESSED.swap(false, Ordering::Relaxed));

        // Handed over on one read, and on that read only: the screen decides on the edge, and a second
        // read with the bits in would be a second decide.
        FEED_DECIDE.store(true, Ordering::Relaxed);
        assert_eq!(held_back(game, other), decide | other);
        assert_eq!(held_back(game, other), other);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // A key pushed while the question was up does not reach the read the game carries on with: the
        // game's own frame before is the frame the question went up on, so a direction still down there
        // is a direction it reads as pressed — and it moves its cursor before it reads its decide.
        let direction = 0x40;
        assert_eq!(held_back(game, decide | other), other);
        assert!(DECIDE_PRESSED.swap(false, Ordering::Relaxed));
        SETTLE_KEYS.store(true, Ordering::Relaxed);
        assert_eq!(held_back(game, decide | other | direction), other);
        // And the decide is not settled with the rest of them: read as let go there, it would be a
        // press again on the read after, which is the question coming back up on its own cancel.
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));
        // One read only. What is still down after it is down against a frame the game was given, so it
        // is the game's own to read — by which time the item it asked about has already had the press.
        assert_eq!(
            held_back(game, decide | other | direction),
            other | direction
        );

        // And the key a question was cancelled with: kept from the screen underneath, whose own back it
        // is, for as long as it is down. One of the two buttons rather than both, that being the
        // ordinary case and the one a faked previous frame got wrong.
        let cancel = game.menu_cancel();
        let one = cancel.isolate_lowest_one();
        HOLD_CANCEL.store(true, Ordering::Relaxed);
        assert_eq!(held_back(game, one | other), other);
        assert_eq!(held_back(game, cancel | other), other);

        // Let go, and the next press of it is the screen's own again: cancelling a question and asking
        // to go back are two different things to ask for.
        assert_eq!(held_back(game, other), other);
        assert!(!HOLD_CANCEL.load(Ordering::Relaxed));
        assert_eq!(held_back(game, one | other), one | other);

        // And it ends with the screen it was cancelled on whether or not the key has been let go: held
        // past that, it would be taking the bomb out of whatever was started next.
        HOLD_CANCEL.store(true, Ordering::Relaxed);
        assert_eq!(held_back(game, one | other), other);
        HOLD_DECIDE.store(false, Ordering::Relaxed);
        assert_eq!(held_back(game, one | other), one | other);
        assert!(!HOLD_CANCEL.load(Ordering::Relaxed));
    }

    /// The address is the file the script was read from, so the same address is the same part
    /// of the ending still running and any other address is the next part of it.
    #[test]
    fn an_ending_moves_on_when_its_script_does() {
        assert!(!moved_on(Some(0x1234), Some(0x1234)));
        assert!(moved_on(Some(0x1234), Some(0x5678)));
    }

    /// A script on one side only says nothing about the other: an ending that has just been
    /// torn down has no script, and reading one is a walk of the game's job chain that can
    /// come back with nothing.
    #[test]
    fn a_script_that_is_not_there_is_not_an_ending_moving_on() {
        assert!(!moved_on(None, Some(0x1234)));
        assert!(!moved_on(Some(0x1234), None));
        assert!(!moved_on(None, None));
    }

    #[test]
    fn a_stage_start_is_never_washed() {
        for watching in [Watching::Judge, Watching::Player, Watching::Nobody] {
            assert!(flash_for(watching, true, Cause::StageStart).is_none());
        }
    }

    #[test]
    fn a_run_is_washed_unless_the_setting_says_not_to() {
        let flash = flash_for(Watching::Player, true, Cause::BossSpell);
        assert_eq!(flash.map(|flash| flash.alpha), Some(FLASH_PLAYING.alpha));
        assert!(flash_for(Watching::Player, false, Cause::BossSpell).is_none());
    }

    /// The wash is the instrument a judging pass is run with, so the setting — which is
    /// about a run being played — does not reach it.
    #[test]
    fn a_judging_pass_is_washed_whatever_the_setting_says() {
        for wanted in [false, true] {
            let flash = flash_for(Watching::Judge, wanted, Cause::Boundary(1886));
            assert_eq!(flash.map(|flash| flash.alpha), Some(FLASH_JUDGING.alpha));
        }
    }

    /// A collecting pass runs a replay with nobody at the keyboard, and crosses a boundary
    /// every few drawn frames.
    #[test]
    fn a_pass_nobody_is_watching_is_not_washed() {
        assert!(flash_for(Watching::Nobody, true, Cause::Boundary(1886)).is_none());
    }

    /// Brighter and longer for the pass nobody is playing, since there nothing is being
    /// dodged on the frame underneath it.
    // Constants either side of every assertion here, which is the point of it: what is being
    // pinned is the relationship between two of them, and a run cannot tell you whether that has
    // been edited apart.
    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn a_judging_pass_is_washed_harder_than_a_run() {
        assert!(FLASH_JUDGING.alpha > FLASH_PLAYING.alpha);
        assert!(FLASH_JUDGING.frames > FLASH_PLAYING.frames);
        // Both fade rather than cutting out, which is what the hold being short of the
        // whole is: `frames - hold` is the divisor the fade is worked out over.
        for flash in [FLASH_JUDGING, FLASH_PLAYING] {
            assert!(flash.hold < flash.frames);
        }
    }
}
