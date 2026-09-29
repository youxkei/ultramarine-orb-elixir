//! **What a playback of th06 says about itself, frame by frame, and where it is played.**
//!
//! orb drives the updates itself to put a run back: it calls the game's own calc chain again and again inside
//! one frame of its own, feeding a frame's buttons to each, until the stage's clock reaches the frame the
//! chapter began on — see `runtime`'s playback. What has to hold of that is not visible from the run landing
//! in the right place, and the other games' own files say it of theirs: `the_updates_a_resume_of_th07_takes`,
//! `…_of_th06c_takes` and `…_of_th08_takes`. This is the same three questions asked of this game.
//!
//! - **Every frame says its own line where the log is verbose**, played and played back alike, which is what
//!   holds a playback to the run it is reproducing: the first frame the two series differ on is where it
//!   parted, and the field that differs says what parted.
//! - **The whole playback is one frame of the host**: a pick-up is asked to be back at once and draw nothing
//!   on the way, where one that ran a few hundred updates to a frame was the run fast-forwarded on the screen.
//! - **A second playback in one session is watched from its own first frame**, the frame last fed and
//!   whether it has parted being a playback's own.
//! - **A run picked up is played on by the rules the way the run it was cut from was**, frame for frame from
//!   the landing — th06c's fourth question, asked of the recipes this game's real cases are played with.
//!
//! **What th07's file asks and this one does not is the playback bounded on the stage clock rather than on a
//! count of updates**, and the reason is this game's: the two counts are the same number here. th07's case
//! rests on the game slowing its own run down — 0x42e186 answering 3 to one update in five, which steps
//! neither of its clocks — and no update of th06 that leaves `GAME_FRAMES` standing inside a run has been
//! read. `asm.md` names none; the one that stops short of the stage, `GameManager::OnUpdate`'s early break
//! while `isInGameMenu` or `isInRetryMenu` is set, is a menu of the game's own, and the button that opens
//! it is kept out of a run's record on purpose (the fake's own `MAP` is held to that), so a playback never
//! feeds it; and a conversation is a frame of the run like any other on this game — see
//! `Game::give_up_on_a_line_of_dialogue`'s own note. A case laid out here would be the two bounds agreeing, which says
//! nothing about either.

use std::path::{Path, PathBuf};

use crate::fake::th06::{CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen};
use orb_sim::keys;

/// How long each of this game's stages runs here: past the frame the card chapter starts on, so the run is
/// left in a chapter deep enough for its playback to be hundreds of updates.
const STAGE_ENDS: u32 = CARD_STARTS + 300;

/// Which chapter the card the midboss puts up is, counting the stage's own start as the first: the two
/// before it are the stage's start and the midboss's arrival — see `pointdevice_run.rs`.
const CARD_CHAPTER: u32 = 3;

/// A pointdevice run played as far as the card's chapter, and that chapter written down, with whatever
/// `settings` asks of the launch beside the verbose log every one of these reads.
fn played_to_the_cards_chapter(
    name: &str,
    settings: impl FnOnce(&mut orb_config::Config),
) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
        settings(config);
    });
    game.streams_its_song(0);
    game.stages_last(STAGE_ENDS);
    game.in_a_pointdevice_run();
    let log = game.log();
    game.frames_until("the card's chapter", CARD_STARTS + 60, || {
        log.said(&format!("chapter {CARD_CHAPTER} at frame {CARD_STARTS}"))
    });
    game.frames_until_a_thread("the chapter written down", || {
        log.said(&format!(
            "at frame {CARD_STARTS}, {CARD_STARTS} frame(s) of buttons"
        ))
    });
    game
}

/// The hit and then タイトルに戻る, which leaves the chapter behind for the run to be picked up from.
fn gives_the_run_up(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.dies();
    game.frames_until("the death seen", READS_KEYS_AFTER, || {
        log.said_since(from, "died in chapter")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said_since(from, "retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said_since(from, "retry: the run is given up")
    });
}

/// **The same walk as `Fake::picks_the_run_up`, waited out on what it says from here on**: that one waits on
/// lines the log holds at all, which the pick-up before this one already wrote — so a second walk by it is one
/// press at the resume question and no playback waited for.
fn picks_it_up_again(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.frames_until("the title menu ready to act on a press", 300, || {
        let front = game.image().front_end_now();
        game.image().scene() == Scene::FrontEnd
            && front.screen == Screen::Title
            && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered again", || {
        game.image().front_end_now().screen == Screen::ShotType
    });
    game.frames_until("the shot type select ready to act on a press", 90, || {
        game.image().front_end_now().acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "つづきから answered", || {
        log.said_since(
            from,
            "resume: from where it stopped, answered on the keyboard",
        )
    });
    game.frames_until("the run played back into place", 60, || {
        log.said_since(from, "resume: the landing is")
    });
}

/// The lines the log holds with `said` in them.
fn lines_saying(game: &Fake, said: &str) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| line.contains(said))
        .collect()
}

#[test]
fn a_playback_of_th06_says_every_frames_own_line_where_the_log_is_verbose() {
    in_its_own_process(|| {
        let game = played_to_the_cards_chapter("th06-playback-lines", |_| {});
        let log = game.log();

        // The frames somebody played, said the same way: the two series are what hold a playback to the run it
        // is reproducing.
        let played = lines_saying(&game, "played with replay_frame=").len();
        assert!(
            played >= CARD_STARTS as usize,
            "the run said its own line on {played} of the {CARD_STARTS} frames it was played for:\n  {}",
            log.lines().join("\n  "),
        );

        gives_the_run_up(&game);
        game.picks_the_run_up();
        // One line per frame the playback moved the run onto, which is every frame of it but the one it began
        // standing on — the stage's own start, whose generator the chapter line already carries.
        let played_back = lines_saying(&game, "played back with replay_frame=");
        assert_eq!(
            played_back.len(),
            CARD_STARTS as usize - 1,
            "the playback said its own line on {} of the {CARD_STARTS} frames it played:\n  {}",
            played_back.len(),
            log.lines().join("\n  "),
        );
        // And each of them the run's own world beside the frame, and how many updates the frame took: the
        // generator alone says only that a playback parted, and the rest say what parted.
        for line in &played_back {
            for named in ["player=", "randoms=", "rng=", "score=", "rank=", "updates="] {
                assert!(
                    line.contains(named),
                    "a frame of the playback said nothing about {named}: {line}",
                );
            }
        }
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A playback of th06 puts its run back inside the one frame of the host it began on**, nothing of the run
/// being played forward drawn: every record the trace takes of it carries that frame's own number.
#[test]
fn a_playback_of_th06_puts_its_run_back_inside_one_frame_of_the_host() {
    in_its_own_process(|| {
        let game = played_to_the_cards_chapter("th06-playback-one-frame", |config| {
            config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
        });
        gives_the_run_up(&game);
        game.picks_the_run_up();
        let log = game.log();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where the file said:\n  {}",
            log.lines().join("\n  "),
        );

        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        // The frames of the host a record of the stage's frames between its start and the landing carries,
        // after the give-up — the run played the first time through is on a host frame apiece.
        let given_up = records
            .iter()
            .rposition(|record| {
                record
                    .state
                    .is_some_and(|state| !state.playing && !state.in_run)
            })
            .expect("the trace holds the front end the run was given up to");
        let played_back: std::collections::BTreeSet<u32> = records[given_up..]
            .iter()
            .filter(|record| {
                record.state.is_some_and(|state| {
                    state.playing && state.stage_frames > 1 && state.stage_frames < CARD_STARTS
                })
            })
            .map(|record| record.frame)
            .collect();
        assert_eq!(
            played_back.len(),
            1,
            "the playback went on over the host's frames {played_back:?}:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A second playback in one session is watched from its own first frame.**
///
/// A run given up and picked up again is a second playback — one that starts at frame 0 again, so the frame
/// last fed by the one before it is a jump backwards that nothing jumped.
#[test]
fn a_second_playback_of_th06_in_one_session_is_watched_from_its_own_first_frame() {
    in_its_own_process(|| {
        let game = played_to_the_cards_chapter("th06-picked-up-twice", |_| {});
        let log = game.log();

        // The run picked up, given up again and picked up a second time: the same file both times, so the two
        // playbacks are the same frames in the same order.
        gives_the_run_up(&game);
        game.picks_the_run_up();
        gives_the_run_up(&game);
        let from = log.written();
        picks_it_up_again(&game);
        assert!(
            log.said_since(
                from,
                "resume: the landing is the frame that was written down, field for field"
            ),
            "the second pick-up did not land where the file said:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            !log.said_since(from, "resume: the playback was fed frame"),
            "the second playback was held against the first one's frames:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **The walk this game's real cases start a run with**, `a_run_left_in_its_second_chapter.yaml`'s: orb's
/// mode question answered, Normal aimed for by the cursor, and the decide that starts the run — ending on the
/// stage's first chapter.
const TO_A_RUN: &str = "
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { frames: 60 }
- press: [down]
  frames: 6
  until: { screen: 7, cursor: 1 }
  give_up_after: 600
";

/// How long a stage lasts for the two launches below, as the real cases give one: past every frame they play.
const A_RECIPES_STAGE: u32 = 6_000;

/// How many frames a recipe is given to run out.
const FRAMES_FOR_A_RECIPE: u32 = 30_000;

/// A launch of this game played by `recipe`, set up the way `the_cases_the_real_game_is_tested_with` sets up
/// one of the real cases: a stage that shoots and ends, the screens between the title and a run, and records
/// with no clear in them — and the trace, which is what the words are read back out of.
fn launched_by(name: &str, recipe: String) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.autoplay = Some(recipe);
        config.trace = Some(PathBuf::from(orb_real::TRACE_FILE));
        config.log_level = LogLevel::Verbose;
    });
    game.fires_at_the_player();
    game.walks_the_screens_between_its_title_and_a_run();
    game.its_records_hold_no_clear();
    game.stages_last(A_RECIPES_STAGE);
    game
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES_FOR_A_RECIPE {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES_FOR_A_RECIPE} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// The records of a launch's trace, once it has closed.
fn the_records_of(game: &Fake) -> Vec<orb_core::trace::Record> {
    orb_core::trace::close();
    let bytes = game
        .sim()
        .files()
        .get(game.dir().join(orb_real::TRACE_FILE))
        .expect("the launch wrote a trace");
    orb_core::trace::read(&bytes).expect("the trace reads back")
}

/// The word each stage frame of a run was first played with, from `from` on — the records of the frames the
/// rules were asked for, which carry where the player was.
fn the_words_from(records: &[orb_core::trace::Record], from: u32) -> Vec<(u32, u16)> {
    let mut seen = std::collections::BTreeMap::new();
    for record in records {
        if let Some(state) = record.state
            && state.in_run
            && state.stage_frames >= from
            && record.at.is_some()
        {
            seen.entry(state.stage_frames).or_insert(record.word);
        }
    }
    seen.into_iter().collect()
}

/// **A run picked up is played on by the rules the way the run it was cut from was**, frame for frame from the
/// landing: what the rules do on a frame depends on the frames before it, and a playback that showed them none
/// has them go on from what they last saw before it — which on th06c was a pick-up of the real Extra's chapter
/// 7 landing field for field and pressed down on the next frame where the run had been pressed up and right.
#[test]
fn a_run_of_th06_picked_up_is_played_on_as_the_run_it_was_cut_from_was() {
    in_its_own_process(|| {
        const PLAYED_ON: u32 = 600;
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 6000
- play_until: {{ frames: {} }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
",
            PLAYED_ON * 2
        );
        let run = launched_by("th06-played-on-as-cut-left", recipe);
        played_out(&run);
        let dir = run.launch().dir().to_path_buf();
        let files = run.sim().files();
        let left: Vec<(PathBuf, Vec<u8>)> = files
            .paths()
            .into_iter()
            .filter_map(|path| {
                let inside = path.strip_prefix(&dir).ok()?.to_path_buf();
                (inside != Path::new(orb_real::TRACE_FILE)).then_some(())?;
                Some((inside, files.get(&path)?))
            })
            .collect();
        let the_run = the_records_of(&run);
        drop(run);

        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 30
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ frames: {PLAYED_ON} }}
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched_by("th06-played-on-as-cut", recipe);
        for (inside, bytes) in &left {
            game.sim().files().put(game.dir().join(inside), bytes);
        }
        played_out(&game);
        let log = game.log();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run was not picked up where the file said:\n  {}",
            log.lines().join("\n  "),
        );
        let landed_at = log
            .lines()
            .iter()
            .find_map(|line| {
                let (_, after) = line.split_once("resume: stage 1 chapter ")?;
                let (_, after) = after.split_once(" at frame ")?;
                after.split(',').next()?.parse::<u32>().ok()
            })
            .expect("the log says where the chapter the run was left in begins");
        let picked_up = the_words_from(&the_records_of(&game), landed_at + 1);
        let the_run = the_words_from(&the_run, landed_at + 1);
        let both = picked_up.len().min(the_run.len());
        assert!(
            both > PLAYED_ON as usize / 2,
            "the two runs have {both} frames after the landing at {landed_at} to hold against each other",
        );
        assert_eq!(
            picked_up[..both],
            the_run[..both],
            "the run picked up at frame {landed_at} was played otherwise than the run it was cut from:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
