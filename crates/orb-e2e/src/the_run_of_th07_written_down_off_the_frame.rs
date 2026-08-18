//! **The file a chapter of 妖々夢 is written down in is written from a thread of orb's own, not in the frame.**
//!
//! One is written every time a chapter begins — see `resume.rs`, and *Picking a run up again* — which is a few
//! tens of kilobytes at every boundary. A `WriteFile` takes what it takes: on one machine the boundaries
//! stopped the game for about a second each, and a write in the frame's own thread is a write the frame waits
//! for, whatever the disk, the antivirus watching it or the folder being synced decide to do about it.
//!
//! So the frame reads the game and hands the chapter over, and a thread of orb's does the encoding and the
//! write. What a test can say about *where* a write happened is the thread it came from, which the simulated
//! Windows records per file: the frames run on the test's own thread, so a write from any other thread is one
//! that is not in the frame.
//!
//! And what 妖々夢 brings of its own is the name that file has. Its runs are told apart by the difficulty, the
//! character and the shot — `Th07::run_slot` — so a chapter of one run is offered back to that run and to no
//! other, and picking it up again is this game's own `start_stage`, `set_run_state` and the added callback the
//! numbers go in on.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between the stage's own start and the card: past the shortest a chapter may be.
const APART: u32 = 120;

/// What the chapter the card begins is called, which is the name the file carries.
const THE_CARD: &str = "MIDBOSS SPELL 1";

/// The name orb gives the run this plays, which is what names its file: `DIFFICULTY`, `CHARACTER` and `SHOT` are
/// 1, 0 and 0 — Normal, and 霊夢's first shot. Written out rather than asked of `Th07::run_slot`, which would be
/// this agreeing with the code it is about.
const SLOT: &str = "normal-reimu-a";

/// The file this run's chapter is written down in, under the directory `resume.rs` keeps them in.
fn written_down_in(game: &Fake) -> std::path::PathBuf {
    game.dir()
        .join("pointdevice_resume")
        .join(format!("{SLOT}.msgpack"))
}

/// A write that took longer than a frame is said at the level a run is ordinarily logged at, not the one
/// somebody has to know to ask for.
///
/// **This is the line the whole arrangement is for.** A session on a machine where the boundaries stopped the
/// game for about a second was played through with the write off the frame and nothing in the log said what the
/// write had cost — it was a `verbose` line, and nobody knew to ask for `verbose`. The evidence for a slow disk
/// has to be in the log the run already writes.
#[test]
fn a_write_of_a_chapter_of_th07_that_took_longer_than_a_frame_is_said_at_the_ordinary_level() {
    in_its_own_process(|| {
        // `normal`, which is what a run is logged at when nobody is looking into anything.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-written-slowly",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Normal,
        );
        let log = game.log();
        // A disk that takes longer over this file than a frame has: what the machine this is about does.
        game.sim()
            .files()
            .writes_slowly(written_down_in(&game), TAKES);
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        game.frames_until_a_thread("the slow write said out loud", || {
            log.said("resume: writing")
        });
        let said = log
            .lines()
            .into_iter()
            .find(|line| line.contains("resume: writing"))
            .expect("the slow write was said");
        assert!(
            said.contains("took longer than a frame"),
            "the line does not say what is wrong with it: {said}",
        );
        // With the microseconds in it, since what the line is for is saying how bad the disk is.
        let us: u64 = said
            .rsplit(' ')
            .next()
            .and_then(|last| last.strip_suffix("us"))
            .and_then(|us| us.parse().ok())
            .unwrap_or_else(|| panic!("no microseconds at the end of {said}"));
        assert!(
            us >= TAKES.as_micros() as u64,
            "the write took at least {:?} and the line says {us}us",
            TAKES,
        );
        // And the run is still being played, which is the other half of taking the write off the frame: the
        // frames went by while the disk was busy.
        let from = game.state().stage_frames;
        game.frames(30);
        assert_eq!(
            game.state().stage_frames,
            from + 30,
            "the game waited for a disk that is not in its frame",
        );
    });
}

/// How long the disk in that e2e test takes over the file: longer than a frame, and short enough that a suite
/// waiting for it is not something anybody notices.
const TAKES: std::time::Duration = std::time::Duration::from_millis(40);

/// How many times a frame of a run being played back may ask the memory map about an address.
///
/// What it covers is the reads that go *through* something the game's allocator handed orb — the stream the
/// music is held quiet through, the record that says which track that is, the stage's dialogue, the panel's own
/// tile — several of them more than once, the state being read more than once a frame.
///
/// **What it refuses is a walk of one of the game's own arrays made through the map.** The laser slots are 0x40
/// on their own, so a state read that asked about each was over this bound by itself — which is what a launch of
/// 妖々夢 was doing, at 196 asks a frame, and what it cost is above.
const ASKS_A_FED_FRAME_MAY_MAKE: u64 = 48;

#[test]
fn the_chapter_of_th07_written_down_is_written_from_a_thread_of_orbs_own() {
    in_its_own_process(|| {
        // `verbose`, because the line saying the file was written is one of those: it is written once a
        // chapter, and this is what says which thread wrote it and how long the write took.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-written-off-the-frame",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The chapter the midboss's card begins, which is a chapter deep enough to have buttons behind it: the
        // file holds every frame of them.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        // Read on the frame the chapter began on and not after waiting for the write: what the file holds is
        // that frame, and one more frame of the run is another.
        let at_the_card = game.state();
        let at = at_the_card.stage_frames;
        // The write is not the frame's, so what a test waits for is the line the writer says it with rather
        // than the frame that handed it over.
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!(
                "chapter 2 ({THE_CARD}) at frame {at}, {at} frame(s) of buttons"
            ))
        });

        let path = written_down_in(&game);
        let wrote = game
            .sim()
            .files()
            .written_from(&path)
            .expect("the chapter was written down");
        assert_ne!(
            wrote,
            orb_api::thread::current_id(),
            "the file was written in the frame's own thread, which is where a slow disk stops the game",
        );
        // And what it holds is the chapter, not a file the writer left half written: the run is picked up out
        // of it below.
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "the run is not among the ones left unfinished",
        );

        // The whole of it read back the way a later launch reads it, which is what says the bytes a thread of
        // orb's wrote are the bytes the frame handed over: the run is started again and picked up at that
        // chapter.
        game.gives_the_run_up();
        let asked_before = game.sim().space().asked_about_an_address();
        game.picks_the_run_up();
        // **And what each of those frames cost in asks of the memory map**, which is the seconds a launch waits
        // before the run lands: every ask is a `VirtualQuery` there, a syscall apiece, and the playback makes
        // one frame's worth of reads per frame of buttons. A 妖々夢 session waited **3.2 seconds** for 2667 of
        // them — 1.2ms an update, against 紅魔郷 running 13675 in 0.8s — and `perf: update=1206us/frame` said
        // whose those milliseconds were: orb's own per-frame work, 18us a frame on 紅魔郷.
        //
        // The count and not the time, because the time is the host's: how long a syscall takes is nothing a
        // suite can hold anything against, and how many of them a frame makes is orb's own.
        let asked = game.sim().space().asked_about_an_address() - asked_before;
        let a_frame = asked / u64::from(at.max(1));
        assert!(
            a_frame <= ASKS_A_FED_FRAME_MAY_MAKE,
            "each of the {at} frames played back asked the memory map {a_frame} time(s), which is more \
             than the {ASKS_A_FED_FRAME_MAY_MAKE} the game's own allocations are behind",
        );
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  ")
        );
        // **And it was quiet while it came back.** The stage's own song is started by the build a moment before
        // the buttons go in, so a playback that leaves the sound alone plays the stage's opening for as long as
        // the buttons take and then jumps to the chapter's place as the run lands — which is what a 妖々夢
        // session heard, seconds of it, on a run picked up in the first stage.
        //
        // **One frame of it and no more**, which is the shape of where orb quietens it: in the game's own input
        // read, every frame a button is fed. The frame the playback begins on has the game's own sound work
        // after that read, and a play standing in its queue is carried out there — so that one frame can sound,
        // and the next read quietens it again. Where the song goes at the landing is the line above.
        let heard = game.frames_the_sound_was_heard_on_while_it_played_back();
        assert!(
            heard <= 1,
            "the stage's own song played on {heard} frames of the run being played back into place:\n  {}",
            log.lines().join("\n  ")
        );
        // The frame it landed on and the numbers it landed with, read out of the game's own memory: the stage's
        // two clocks put the run at the frame the file names, and the numbers are the ones that stage began
        // with, written back by `Th07::set_run_state`.
        //
        // Those and not the whole `State`, because the rest of what the chapter was inside is the stage's own
        // script's to put up again — the fight, its card, the enemies — and a laid-out 妖々夢 has no script to
        // run. What says the landing is right field for field is the line above, which is orb's own comparison
        // against what it wrote down.
        let landed = game.state();
        assert_eq!(
            (landed.stage_frames, landed.script_frames),
            (at_the_card.stage_frames, at_the_card.script_frames),
            "the run landed on another frame than the one written down off the frame",
        );
        assert_eq!(
            (landed.lives, landed.bombs, landed.power, landed.deaths),
            (
                at_the_card.lives,
                at_the_card.bombs,
                at_the_card.power,
                at_the_card.deaths
            ),
            "the run landed with numbers other than the ones the stage it was written down in began with",
        );
    });
}
