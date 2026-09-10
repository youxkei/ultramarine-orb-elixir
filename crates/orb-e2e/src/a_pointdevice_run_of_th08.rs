//! Pointdevice mode over a laid-out th08: the question that starts it, a chapter, a death and a retry.
//!
//! **This is what `Th08::rewinds` rests on**, the same way `a_pointdevice_run_of_th07` is what that
//! game's rests on. Everything else about this game is a unit test or an e2e test over one field at a
//! time, and none of those can say that the *mode* holds together here: the chapter a snapshot is taken
//! as covers this game's own twenty-one megabytes of `.data` and the five heap regions beside it, the
//! ranges a restore must leave alone are this game's sound manager and the sheets of every `.anm` slot it
//! has loaded, the retry menu is drawn in a face the machine lent — th08 ships no font either — and the
//! numbers have to come back through this game's own replay record.
//!
//! **And the real game has answered the same question**, which is the other half of that method being
//! `true`: a launch of `a-chapter-taken-and-put-back.yaml` at eight times the rate took a chapter of
//! stage 1, died in it on purpose, answered the retry on the pad and came back —
//! `restore: skipping 21 range(s), 21728 bytes` over the three of the sound and eighteen of the slots,
//! six regions put back, and the run playing on afterwards.
//!
//! **The game plays the game's part and this presses keys**, as th07's own does: what moves the run
//! along is the laid-out game, and what this file does is press, run frames, and read back the game's
//! own memory and what orb put in the log.

use std::path::PathBuf;

use crate::fake::th08::{
    BOMBS, DIFFICULTY_NORMAL, EXE, FIRST_STAGE, Fake, LIVES, THE_FIGHTS_TRACK, TITLE_ITEM_REPLAYS,
};
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th08::image::{Scene, Screen};
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a screen or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 120;

/// And the frames the team select is given to come up before its press, past the eight 0x46a92d counts.
const FRAMES_THE_TEAM_SELECT_COMES_UP_OVER: u32 = 10;

/// The item of the title menu a run is behind, which is where a launch of the real game reads the cursor
/// before anything has been left.
const TITLE_ITEM_START: i32 = 0;

/// Two of the retry menu's own ways on, in the language a launch on this machine comes out in, and the
/// chapter it is offering: a stage with no midstage boundary measured has its own start and no other.
const AGAIN: &str = "チャプターをやり直す";
const GIVEN_UP: &str = "タイトルに戻る";
const CHAPTER: &str = "MIDSTAGE 1";

/// The name orb gives the run this plays, which is what names the file it is left in: the difficulty, the
/// team's index and the `.sht` that index loads — see `Th08::run_slot`.
///
/// **Written out rather than asked of `run_slot`**, which would be this agreeing with the code it is about:
/// what the name has to be is what somebody reading the directory sees, and the walk above chooses Normal
/// and the first team.
const SLOT: &str = "normal-0-ply00a";

/// How long the laid-out stage lasts here, which is longer than this test plays: what is being watched is
/// a chapter and a death inside one stage, so the stage must not end under it.
const A_STAGE: u32 = 100_000;

#[test]
fn a_pointdevice_run_of_th08_is_chosen_chaptered_lost_and_retried() {
    in_its_own_process(|| {
        // `verbose`, because half of what this reads back is written at that level.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-pointdevice",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();

        // ── 1. The mode. A game orb can rewind starts in Pointdevice mode and is asked which at its own
        // title menu, which is what `Th08::rewinds` answering `true` turns on.
        assert!(
            log.said(&format!(
                "mode: {} to start with; the menu asks which",
                Mode::Pointdevice
            )),
            "the launch did not start in the mode orb is for: {:?}",
            log.lines(),
        );

        // The overlay first, which is orb's own rule about asking anything — and here it is a face the
        // machine lent, this game shipping no `font.ttf` of its own.
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));

        // The title menu up with the cursor on the item a run is behind, left to come up by itself rather
        // than placed: this game holds a press back over a screen that has just come up — the gate at
        // 0x467cac — and its own walk is what settles that.
        game.frames_until("the title menu came up", FRAMES_FOR_A_SCREEN, || {
            game.image().the_screen_the_front_end_is_on() == Some(Screen::Title as i32)
                && game.image().the_item_the_cursor_is_on() == Some(TITLE_ITEM_START)
        });
        game.frames_until(
            "the title menu taking a press",
            FRAMES_FOR_A_SCREEN,
            || unsafe { orb_core::game::Game::menu_takes_a_press(&orb_core::game::th08::Th08) },
        );
        for _ in 0..4 {
            game.press(keys::Z);
        }
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "the press did not put the question up: {:#?}",
            log.lines(),
        );

        // What the question drew, read off the screen: the title of it, in the face the machine lent.
        game.one_frame();
        assert_eq!(
            game.says(title(Menu::Run, LANGUAGE)).len(),
            1,
            "the mode question is not on the screen",
        );

        // Answered on the keyboard, which is the hand this run has.
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        assert!(
            log.said(&format!("mode: {}", Mode::Pointdevice)),
            "the mode was not settled: {:?}",
            log.lines(),
        );

        // ── 2. The run, walked into rather than placed: the difficulty select, the team select and the
        // two presses that screen takes. **Which is what a chapter needs** — orb is told a stage has begun
        // by the patch over this game's own build, and a run put in place by writing the scene goes
        // through no build at all.
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();
        // **One press per screen and then the frames that screen takes to come up**, which is the order
        // the gate at 0x467cac wants: a screen this game has just put up refuses a press for ten frames,
        // so pressing again the moment it arrives is pressing at nothing.
        for screen in [Screen::Difficulty, Screen::Team] {
            let wanted = screen as i32;
            game.press(keys::Z);
            game.frames_until(&format!("screen {wanted}"), FRAMES_FOR_A_SCREEN, || {
                game.image().the_screen_the_front_end_is_on() == Some(wanted)
            });
        }
        // And the team select's own press, once it has come up — see `Th08`'s `TEAM_PHASE_CHOOSING`, which
        // is why a press the moment it arrives is none: a launch of the real game measured that one too.
        for _ in 0..FRAMES_THE_TEAM_SELECT_COMES_UP_OVER {
            game.frame();
        }
        game.press(keys::Z);
        game.frames_until("the run", FRAMES_FOR_A_SCREEN, || game.state().playing);
        // **And the stage built**, which is a frame later than the run's scene: this game builds a
        // stage on a thread of its supervisor's, and the run's numbers are what that thread writes — see
        // `Th08::a_stage_has_just_been_built`. Read on the scene's own first frame they are bytes nothing
        // has set, on the real game and here alike.
        game.frames_until("the stage built", FRAMES_FOR_A_SCREEN, || {
            !game.image().the_loader_is_running()
        });

        // And the run's own numbers, which its build put in place — see `Fake::the_numbers_a_run_starts_with_are_in_place`.
        let run = game.state();
        assert_eq!(
            (run.stage, run.difficulty, run.lives, run.bombs),
            (FIRST_STAGE, DIFFICULTY_NORMAL, LIVES as i8, BOMBS as i8),
            "the run the walk started is not the one this game's own front end starts",
        );

        // ── 3. The stage's first chapter, which is its own start: no row of this game's midstage table is
        // measured yet, so every chapter of it is a stage's start until a tuning run says otherwise.
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        // And that chapter written down, which is what a run of this mode is kept for: the file the
        // launch beside this one picks the run up out of. **Written off the frame the supervisor's loader
        // finished on**, this game building a stage on a thread — see `Th08::a_stage_has_just_been_built`
        // — and a laid-out game that built on the switch itself was a run written down where the real
        // one wrote bytes nothing had set.
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("chapter 1 ({CHAPTER}) at frame"))
        });
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "the run was not left behind under the name of its own difficulty and shot",
        );
        // And the music played on past what the chapter holds of it.
        game.refills_its_music();
        game.refills_its_music();
        let (_, chunk, before_the_retry) = game.image().where_the_stream_is();

        // ── 4. The death. The game counts one, takes the life and puts the player in the state a hit
        // leaves — and it is the *count* orb reads a death off, which is what makes a deathbomb no death.
        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_has_been_hit();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));

        // The retry menu, drawn over the frozen game and read off the screen: its own two ways on and the
        // chapter the run was in under them.
        game.one_frame();
        assert_eq!(
            (
                game.says(AGAIN).len(),
                game.says(GIVEN_UP).len(),
                game.says(CHAPTER).len(),
                game.says("RETRY 0").len(),
            ),
            (1, 1, 1, 1),
            "the menu over the frozen game is not the one the mode puts there: {:?}",
            log.lines(),
        );

        // ── 5. The retry. Answered on the keyboard, and what comes back is the chapter's own numbers,
        // written through this game's own replay record — see `Th08::put_the_numbers_back`.
        game.press_until(keys::Z, "the chapter put back", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let state = game.state();
        assert_eq!(
            (state.lives, state.bombs),
            (LIVES as i8, BOMBS as i8),
            "the chapter did not put the run's numbers back",
        );
        assert!(state.playing, "the run was not left playing");
        // **And the music left playing where it had got to**: the refill after the retry takes up from the
        // one before it and not from where the chapter's snapshot had the stream, which is what the real
        // game did over a chapter taken again — `19,865,432` bytes of the track left on the refill before
        // the restore and `19,821,332` on the one after, a chunk of `44,100` apart, the same track and no
        // stretch of it gone round again.
        game.refills_its_music();
        let (.., after_the_retry) = game.image().where_the_stream_is();
        assert_eq!(
            after_the_retry,
            before_the_retry - chunk,
            "the retry put the music back to the chapter's own place in it rather than leaving it playing",
        );

        // ── 6. The next stage, and a chapter of its own at its start. **Which this game's clock alone
        // would never give**: the frames the game counts are the run's and carry across a build, and the
        // chapters take a stage as begun where the count reads nought — so a whole Normal run on the real
        // game took every chapter of stage 1 and none of stage 2, and lost there. `Th08::read_state` counts
        // a stage's frames from its build for that.
        game.moves_on_to_the_stage(FIRST_STAGE + 1);
        game.frames_until("the next stage's own first chapter", 600, || {
            log.said("stage 2 chapter 1 (stage start)")
        });
        // **And that chapter written down with the buttons of its own frames**, which is what a launch that
        // picks the run up there plays back in. The same clock is why: counted off the run's frames, a whole
        // Normal run wrote `20585 frame(s) of buttons and no generator for any of them` for a chapter of stage
        // 2 — every frame under the chapter an empty one — and wrote nothing at all from stage 6 on, the
        // run's clock being past the half hour a record is bounded by.
        let written = || {
            log.lines()
                .into_iter()
                .find(|line| line.contains("resume: stage 2 chapter 1 ("))
        };
        game.frames_until_a_thread("the next stage's first chapter written down", || {
            written().is_some()
        });
        let written = written().unwrap_or_default();
        assert!(
            written.contains("and the generator each was played with"),
            "the next stage's first chapter was written down without the frames it was played with: {written}",
        );
    });
}

/// **A replay started off the game's own list is asked nothing and kept nothing of**, with a run of the
/// same difficulty and team left in a chapter for a question to have been about.
///
/// **What the real game did in its place**: a `--collect` launch that decided on the title's fifth item
/// and then on the list it opened read `resume: normal-0-ply00a was left; asking where to start` the frame
/// the replay asked for its run — and nothing answered, so the replay stood behind orb's question until
/// the launch was closed. The list's decide sets bit 3 of the run kind and nothing else of it (0x46ed24),
/// where every screen that starts a run somebody plays writes that bit as nothing first (0x46ac1f,
/// 0x46b0be, 0x46ca8d), and the attract demo sets it with bit 1 beside it (0x467b65).
///
/// **A pass over a replay, because that is the launch that meets it**: orb asks no mode question there and
/// stays in Pointdevice mode, still keeping what a run plays. Where the question is asked, the item's own
/// answer for the game's replays is the other mode, which keeps nothing and asks nothing either way.
#[test]
fn a_replay_of_th08_off_its_own_list_is_asked_nothing_and_kept_nothing_of() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-replay-off-its-list",
            Work::wandering(WORK_US),
            |config| {
                config.chapter_tuning = true;
                config.during_replay = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();

        // A run walked into off the title's first item and left in its first chapter, which is what the
        // replay below would have been asked about: the same difficulty and the same team.
        game.frames_until("the title menu came up", FRAMES_FOR_A_SCREEN, || {
            game.image().the_screen_the_front_end_is_on() == Some(Screen::Title as i32)
                && game.image().the_item_the_cursor_is_on() == Some(TITLE_ITEM_START)
        });
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();
        for screen in [Screen::Difficulty, Screen::Team] {
            let wanted = screen as i32;
            game.press_until(keys::Z, &format!("screen {wanted}"), || {
                game.image().the_screen_the_front_end_is_on() == Some(wanted)
            });
        }
        game.press_until(keys::Z, "the run", || game.state().playing);
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("chapter 1 ({CHAPTER}) at frame"))
        });
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "no run was left for a question to be about",
        );

        // The title again, which is where the list is — placed rather than walked back to, what this is
        // about being the list and not the way out of a run.
        game.its_title_menu_is_up();
        let from = log.written();
        game.press_until(keys::DOWN, "the cursor on the replays", || {
            game.image().the_item_the_cursor_is_on() == Some(TITLE_ITEM_REPLAYS)
        });
        game.press_until(keys::Z, "the list of replays", || {
            game.image().the_screen_the_front_end_is_on() == Some(Screen::Replays as i32)
        });
        game.press_until(keys::Z, "the replay's run asked for", || {
            game.state().wanted == Scene::Run as i32
        });
        game.frames(FRAMES_FOR_A_SCREEN);
        assert!(
            !log.said_since(from, "asking where to start"),
            "a question about the run left was put up over a replay nobody is playing:\n  {}",
            log.lines().join("\n  "),
        );

        let state = game.state();
        assert!(
            state.replay && !state.demo,
            "the run the list started is not a replay being played back: {state}",
        );
        assert!(
            state.playing && !state.in_game,
            "a replay is a stage played through and no run somebody is playing: {state}",
        );
        assert!(
            !log.said_since(from, "chapter 1"),
            "a chapter was kept over a replay nobody is playing:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A run picked up whose playback is hit short of its chapter is the rules' to play after the retry**, and
/// not the record's.
///
/// The playback stops at the hit and the death is answered as any is, with the chapter again — which is one the
/// playback took on its way in. **What came of that on the real game was the record fed again from there**, the
/// feeding being bounded by the chapter the run was picked up at and the retry being a frame below it: an Extra
/// picked up at its fifth boss card parted at frame 539, was hit at 569 and went back to chapter 1 forty times,
/// the player standing where the record had it whatever the rules pressed.
///
/// **The file is the one a run left in its second chapter, with its buttons taken out from a frame on**: a record
/// the laid-out stage's bullets, aimed at where the player stands, find standing still — so its playback is hit
/// short of the chapter where the run it was written from dodged.
#[test]
fn a_run_of_th08_whose_playback_is_hit_short_of_its_chapter_is_the_rules_to_play_after_the_retry() {
    in_its_own_process(|| {
        let first = launched(
            "th08-a-run-left-for-its-playback-to-be-hit",
            include_str!("../../orb-real/recipes/th08/a_run_left_in_its_second_chapter.yaml"),
        );
        runs_out(&first);
        let dir = first.dir().to_path_buf();
        let files = first.sim().files();
        let left: Vec<(PathBuf, Vec<u8>)> = files
            .paths()
            .into_iter()
            .filter_map(|path| {
                Some((
                    path.strip_prefix(&dir).ok()?.to_path_buf(),
                    files.get(&path)?,
                ))
            })
            .collect();

        let kept = PathBuf::from("pointdevice_resume").join(format!("{SLOT}.msgpack"));
        let game = orb_core::game::found(EXE)
            .expect("th08 is a game orb knows")
            .game;
        let bytes = left
            .iter()
            .find(|(inside, _)| *inside == kept)
            .map(|(_, bytes)| bytes)
            .expect("the run was left behind");
        let mut saved = game
            .decode_resume(bytes)
            .expect("the run left behind reads back");
        assert!(
            saved.buttons.len() > STANDING_FROM,
            "the run was left before the frame its buttons are taken out from: {} frame(s)",
            saved.buttons.len(),
        );
        for word in saved.buttons.iter_mut().skip(STANDING_FROM) {
            *word = 0;
        }
        let standing = game.encode_resume(&saved);

        let second = launched(
            "th08-a-run-picked-up-to-be-hit-in-its-playback",
            include_str!("../../orb-real/recipes/th08/the_run_picked_up_where_it_was_left.yaml"),
        );
        for (inside, bytes) in &left {
            let bytes = if *inside == kept { &standing } else { bytes };
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let log = second.log();
        assert!(
            log.said("resume: the playback stopped because the player was hit at frame"),
            "the playback was not hit short of its chapter:\n  {}",
            log.lines().join("\n  "),
        );
        let said: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| {
                ["died in chapter", "retry", "resume: the", "chapter "]
                    .iter()
                    .any(|it| line.contains(it))
                    && !line.contains("resume: frame ")
            })
            .collect();
        let deaths = said
            .iter()
            .filter(|line| line.contains("died in chapter"))
            .count();
        assert_eq!(
            deaths,
            1,
            "the chapter taken again was played from the record and not by the rules:\n  {}",
            said.join("\n  "),
        );
        assert!(
            log.said("stage 1 chapter 3"),
            "the rules did not play on past the chapter the run was picked up at:\n  {}",
            said.join("\n  "),
        );
    });
}

/// **The frame the file's buttons are taken out from**: past the stage's opening, whose frames the rules spend
/// walking to their place, and before the laid-out stage's first bullet at 480.
const STANDING_FROM: usize = 400;

/// **The fourth stage's boss going the first time is a chapter of its own, and the stage is not won by it** —
/// the stretch of waves she leaves for being what a death there goes back to, where before it was her last
/// card's chapter and a stage walled as over. **The second time is the stage's end** and takes no chapter, as
/// every stage's own boss going down does not; and a stage whose boss does not come back is the same.
///
/// **What the real game did**: session 13's no-bomb clear fought 4B's boss through chapters 10 to 13, her
/// going leaving card 82's chapter open for the 440-odd frames to the table's boundary at `5404`.
///
/// **A run placed and its stage's clock stepped by hand**, the boss put up and taken down between: what this
/// is about is the chapters, and a stage firing at a player nobody moves is a death and a retry offered.
#[test]
fn the_first_half_of_th08s_fourth_stages_fight_ends_in_a_chapter_of_its_own() {
    in_its_own_process(|| {
        let game = in_the_stage("th08-a-boss-that-comes-back", THE_POWERFUL_FOURTH);
        let log = game.log();
        let said = |chapter: &str| log.said(&format!("stage 5 chapter {chapter}"));

        the_stage_goes_on_for(&game, A_HALF);
        the_boss_fights_for(&game, A_HALF);
        assert!(
            said("2 (BOSS NONSPELL 1)"),
            "the boss's arrival took no chapter of her fight:\n  {}",
            log.lines().join("\n  ")
        );
        the_stage_goes_on_for(&game, A_HALF);
        assert!(
            said("3 (MIDSTAGE 2)"),
            "her going the first time took no chapter for the waves she left:\n  {}",
            log.lines().join("\n  ")
        );
        the_boss_fights_for(&game, A_HALF);
        assert!(
            said("4 (BOSS NONSPELL 2)"),
            "her coming back took no chapter of her fight:\n  {}",
            log.lines().join("\n  ")
        );
        the_stage_goes_on_for(&game, A_HALF);
        assert!(
            !said("5 "),
            "her going the second time, which is the stage's end, took a chapter:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// And the same two goings in a stage whose boss does not come back — the fifth — are the stage's end the
/// first time, with no chapter for either.
#[test]
fn a_th08_stages_own_boss_going_is_no_chapter_where_she_does_not_come_back() {
    in_its_own_process(|| {
        let game = in_the_stage("th08-a-boss-that-does-not-come-back", THE_FIFTH);
        let log = game.log();
        the_stage_goes_on_for(&game, A_HALF);
        the_boss_fights_for(&game, A_HALF);
        the_stage_goes_on_for(&game, A_HALF);
        assert!(
            log.said("stage 6 chapter 2 (BOSS NONSPELL 1)") && !log.said("stage 6 chapter 3 "),
            "the fifth stage's own boss going took a chapter:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// The two stage words those are, the powerful fourth stage and the fifth — see `fake::th08`'s
/// `A_RUNS_STAGES`.
const THE_POWERFUL_FOURTH: i32 = 4;
const THE_FIFTH: i32 = 5;

/// **How long each half of the fight and each stretch of waves lasts**, past the shortest a chapter may be —
/// the stage's own opening among them, a boss arriving on the frame after the stage's first chapter being a
/// boundary too soon after it to take.
const A_HALF: u32 = 120;

/// **A run placed and walked into stage word `stage`**, with the stage's first chapter taken.
fn in_the_stage(name: &str, stage: i32) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.moves_on_to_the_stage(stage);
    game.frames_until("the stage built", FRAMES_FOR_A_BUILD, || {
        game.state().stage == stage && !game.image().the_loader_is_running()
    });
    let log = game.log();
    for _ in 0..FRAMES_FOR_A_BUILD {
        if log.said(&format!("stage {} chapter 1 (stage start)", stage + 1)) {
            return game;
        }
        a_frame_of_the_stage(&game);
    }
    panic!(
        "the stage's first chapter was never taken:\n  {}",
        log.lines().join("\n  ")
    );
}

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// **One frame of a stage placed by hand**, the run's clock stepped the way the stage's own jobs step it —
/// which nothing does for a run an e2e test placed — and its script's left standing, so that no boundary
/// of the stage's table comes up under what this is about.
fn a_frame_of_the_stage(game: &Fake) {
    let image = game.image();
    image.has_been_in_the_run_for(image.the_frames_the_run_has_been_going() + 1);
    game.frame();
}

/// **The stage's own boss up for `frames`**, to her own track — which is what tells her from a midboss,
/// see `Th08::the_fights_own_song_is_playing`.
fn the_boss_fights_for(game: &Fake, frames: u32) {
    game.image().a_boss_is_fighting(0, (192.0, 96.0), 1000);
    game.image().the_stream_reads_the_track(THE_FIGHTS_TRACK);
    for _ in 0..frames {
        a_frame_of_the_stage(game);
    }
}

/// And gone for `frames`, the stage going on.
fn the_stage_goes_on_for(game: &Fake, frames: u32) {
    game.image().no_boss_is_fighting();
    for _ in 0..frames {
        a_frame_of_the_stage(game);
    }
}

/// **A chapter that begins on the frame the run's clock jumped to is kept, and picked up there.** The real
/// game's clock goes 1,800 on in one update as a boss's card ends, so the frames between are ones no update
/// plays, and the boss's next attack begins on the frame the clock lands on. **On the real game every such
/// chapter went unwritten**: session 11's no-bomb clear read `resume: not written — 27113 frame(s) of buttons
/// for a chapter at 28913` and six more like it, each a boss's attack after one of its cards, so a death there
/// went back to the card before it.
///
/// **The laid-out stage's clock jumps on the first boundary of the stage's table**, which is where a run left
/// in its second chapter is left.
#[test]
fn a_chapter_of_th08_begun_where_its_clock_jumped_to_is_kept_and_picked_up_there() {
    in_its_own_process(|| {
        let first = launched(
            "th08-a-run-left-in-a-chapter-its-clock-jumped-to",
            include_str!("../../orb-real/recipes/th08/a_run_left_in_its_second_chapter.yaml"),
        );
        first.skips_its_clock_at(THE_SECOND_CHAPTER_AT);
        runs_out(&first);
        let landed_on = THE_SECOND_CHAPTER_AT + THE_CLOCK_JUMPS_BY;
        let log = first.log();
        let kept = format!("resume: stage 1 chapter 2 (MIDSTAGE 2) at frame {landed_on}");
        assert!(
            log.said(&kept) && !log.said("resume: not written"),
            "the chapter the clock jumped to was not kept at frame {landed_on}:\n  {}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("resume: ") && !line.contains("resume: frame "))
                .collect::<Vec<_>>()
                .join("\n  "),
        );
        let dir = first.dir().to_path_buf();
        let files = first.sim().files();
        let left: Vec<(PathBuf, Vec<u8>)> = files
            .paths()
            .into_iter()
            .filter_map(|path| {
                Some((
                    path.strip_prefix(&dir).ok()?.to_path_buf(),
                    files.get(&path)?,
                ))
            })
            .collect();

        let second = launched(
            "th08-a-chapter-a-conversation-began-picked-up",
            include_str!("../../orb-real/recipes/th08/the_run_picked_up_where_it_was_left.yaml"),
        );
        second.skips_its_clock_at(THE_SECOND_CHAPTER_AT);
        for (inside, bytes) in &left {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let log = second.log();
        let said: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("resume: ") && !line.contains("resume: frame "))
            .collect();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the chapter was not picked up on the frame it was written at:\n  {}",
            said.join("\n  "),
        );
        assert!(
            log.said("stage 1 chapter 3"),
            "the run picked up did not play on into the chapter after:\n  {}",
            said.join("\n  "),
        );
    });
}

/// **A run picked up in a stage after its first plays the run it was written from back into place**, where
/// its numbers are not the ones a build starts a run with. What the numbers put back break is the run's seal
/// — see `Th08`'s `RUN_SEAL_KEY` — which the game's own setters keep and a write from outside does not, and
/// the enemy manager stops the chain behind it from the stage's first frame. **On the real game the pick-up
/// of stage 2 never moved**: its playback stood at `192,384` through all 7,318 frames, the bullets and the
/// card's clock with it, and read `the landing is not the frame written down`.
///
/// **The laid-out stage has no items, so the power a person's run takes is put in by hand** part way through
/// stage 1, through the setter that seals it the way collecting one does.
#[test]
fn a_run_of_th08_picked_up_after_its_first_stage_lands_where_it_was_written() {
    in_its_own_process(|| {
        let first = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-run-left-after-its-first-stage",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    include_str!(
                        "../../orb-real/recipes/th08/a_chapter_of_the_stage_after_the_first.yaml"
                    )
                    .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        first.plays_its_own_stage();
        first.stages_last(A_SHORT_STAGE);
        first.frames_until("stage 1 played part way", 40_000, || {
            let state = first.state();
            state.in_run && state.stage == FIRST_STAGE && state.stage_frames >= A_POWER_ITEM_AT
        });
        first.image().shot_is_at_power(THE_POWER_ITS_ITEMS_TOOK);
        runs_out(&first);
        let dir = first.dir().to_path_buf();
        let files = first.sim().files();
        let left: Vec<(PathBuf, Vec<u8>)> = files
            .paths()
            .into_iter()
            .filter_map(|path| {
                Some((
                    path.strip_prefix(&dir).ok()?.to_path_buf(),
                    files.get(&path)?,
                ))
            })
            .collect();

        let second = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-run-picked-up-after-its-first-stage",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    include_str!(
                        "../../orb-real/recipes/th08/the_run_picked_up_in_the_stage_it_walked_into.yaml"
                    )
                    .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        second.plays_its_own_stage();
        second.stages_last(A_SHORT_STAGE);
        for (inside, bytes) in &left {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let log = second.log();
        let said: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("resume: ") && !line.contains("resume: frame "))
            .collect();
        assert!(
            said.iter()
                .any(|line| line.contains("the run is being built at that stage"))
                && !said.iter().any(|line| line.contains("stage 1 chapter")),
            "the run was not picked up in a stage after its first:\n  {}",
            said.join("\n  "),
        );
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run picked up after its first stage did not land where it was written:\n  {}",
            said.join("\n  "),
        );
    });
}

/// **A run picked up whose stage's build goes on after the stage's jobs are on the chain lands where it was
/// written** — the supervisor's thread putting the bullets' and the enemies' jobs up straight after the seed
/// and asleep in its last wait for frames after the count it waits for is up, which the walk runs the stage
/// on. On the real game that took two of three pick-ups of one file off their run on the first frame, `the
/// playback parted from the run at frame 0: the generator is 0xeede where the run had 0xe08d`, their stage at
/// `frames=5` with `script=7` and `8`.
#[test]
fn a_run_of_th08_picked_up_whose_build_goes_on_after_its_jobs_are_up_lands_where_it_was_written() {
    in_its_own_process(|| {
        let first = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-run-left-in-a-stage-after-its-first",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    include_str!(
                        "../../orb-real/recipes/th08/a_chapter_of_the_stage_after_the_first.yaml"
                    )
                    .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        first.plays_its_own_stage();
        first.stages_last(A_SHORT_STAGE);
        runs_out(&first);
        let dir = first.dir().to_path_buf();
        let files = first.sim().files();
        let left: Vec<(PathBuf, Vec<u8>)> = files
            .paths()
            .into_iter()
            .filter_map(|path| {
                Some((
                    path.strip_prefix(&dir).ok()?.to_path_buf(),
                    files.get(&path)?,
                ))
            })
            .collect();

        let second = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-run-picked-up-whose-build-goes-on",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    include_str!(
                        "../../orb-real/recipes/th08/the_run_picked_up_in_the_stage_it_walked_into.yaml"
                    )
                    .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        second.plays_its_own_stage();
        second.stages_last(A_SHORT_STAGE);
        second.goes_on_building_for(A_BUILD_GOES_ON_FOR);
        for (inside, bytes) in &left {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let log = second.log();
        let said: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("resume: ") && !line.contains("resume: frame "))
            .collect();
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run picked up did not land where it was written:\n  {}",
            said.join("\n  "),
        );
    });
}

/// **How many frames that build sleeps on past its count**, which is the most the real one's stage was ahead of
/// its clock by: `script=8` at `frames=5`.
const A_BUILD_GOES_ON_FOR: u32 = 3;

/// How long a stage of the laid-out game lasts for that pair, which is what the chains of the cases give one:
/// past stage 1's own midstage table, whose last boundary is at script 4086.
const A_SHORT_STAGE: u32 = 4_500;
/// Where in stage 1 the power goes up, and to what: the real run's stage 2 began at `72` where the build that
/// picked it up had `0`, and a build here has `96` — what has to hold is that the two differ.
const A_POWER_ITEM_AT: u32 = 1_000;
const THE_POWER_ITS_ITEMS_TOOK: f32 = 72.0;

/// **Where the laid-out stage's clock jumps**: the first boundary of the stage's table, script 790, which the
/// stage's own clock is on too until the jump puts it past — see `chapters::MIDSTAGE`.
const THE_SECOND_CHAPTER_AT: u32 = 790;
/// And by how much, which is the real game's own `0x708`.
const THE_CLOCK_JUMPS_BY: u32 = 1800;

/// A launch of this game played by a recipe, over a stage of its own — see
/// `the_cases_of_th08_the_real_game_is_tested_with`, whose launches these are.
fn launched(name: &str, recipe: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_STAGE);
    game
}

/// **Played until its recipe quits it**, and failed where it never does.
fn runs_out(game: &Fake) {
    const FRAMES: u32 = 40_000;
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the launch never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  "),
    );
}
