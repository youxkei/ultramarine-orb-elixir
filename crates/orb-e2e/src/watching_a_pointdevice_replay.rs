//! **A replay of a pointdevice run watched: the item on the title menu, the mode asked over it, and the
//! stage, the chapter and the attempt chosen out of one.**
//!
//! The title menu's replay item is the fifth screen orb asks the mode question over, and it is the one where
//! the two answers do different things rather than the same thing to different files: **レガシーモード hands
//! the press over** and the game shows the replays it saved itself, where **完全無欠モード** is a list of
//! orb's own — the game has no screen for these, its own replay of such a run being the path that survived
//! with nothing in it of the attempts that did not.
//!
//! Then three lists, because that is what one of these holds: a run is stages, a stage is chapters, and a
//! chapter is the attempts made at it. Choosing one plays it back the way a run is picked up — the frames
//! below the chapter fed inside the frame that built the stage, the landing held against the reproduction
//! line written down — and then goes on feeding, one frame of the attempt to one frame of the game, which is
//! the part somebody is watching.
//!
//! **And nothing of the run it came out of is touched.** The chapter that run was left in and the replay
//! still being written into are both files of a run somebody is in the middle of; what is being watched is
//! a copy that was finished with. A playback that wrote either would be a run overwritten by a recording of
//! itself.

use crate::fake::th06::{
    A_BOMB_IS_TAKEN_FOR, ATTACK_CHANGES, CARD_STARTS, Fake, MAPPING, STAGES, the_run,
};
use crate::fake::{LANGUAGE, Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{Pushed, Scene, Screen, item};
use orb_core::menu::By;
use orb_core::menu_ui::SELECTED;
use orb_core::mode::title;
use orb_sim::keys;

const SLOT: &str = "normal-reimu-a";

/// The frames of an attempt watched one at a time before it is asked about.
const A_FEW_FRAMES: u32 = 4;
/// The date the simulated host's calendar is set to, which a kept replay's name and the line the list shows
/// it on both carry.
const DATED: &str = "20040815-214233";
/// And the name whoever plays these runs has set in `orb.yaml`, which is the other half of that line — for a
/// run that was played out.
const PLAYER: &str = "ゆうか";
/// And what the line says in that name's place for a run nobody finished, which is every run these tests give
/// up or write over.
const INTERRUPTED: &str = "中断";
/// How long a stage of a run played to its clear here lasts, and the frames the whole of one needs — the
/// shape `the_screen_a_finished_run_ends_at.rs` uses, a clear being the only way a run of this mode reaches
/// the screens it ends at.
const STAGE_FRAMES: u32 = 300;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES as u32 * 2;

/// A launch whose replays are kept without asking, so that what these e2e tests drive is the watching.
fn keeping_every_replay(name: &str) -> Box<Fake> {
    Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
        config.save_replays = true;
        config.player_name = PLAYER.to_owned();
    })
}

/// A run played to its midboss's card, hit there, the chapter retried, played on to the chapter after it,
/// and then given up — which leaves a replay of one stage with four chapters, the third of them with two
/// attempts at it.
///
/// Returns the frame the attempt that was rewound away ended on, which is how long it is.
fn a_run_worth_watching(game: &Fake) -> u32 {
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames_until("the card's chapter", CARD_STARTS + 400, || {
        log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
    });
    game.hit();
    game.frames_until(
        "the death the hit stands for",
        A_BOMB_IS_TAKEN_FOR as u32 + 2,
        || log.said("died in chapter 3"),
    );
    let died_at = game.state().stage_frames;
    game.press_until(keys::Z, "the chapter retried", || {
        log.said("retry: the chapter again on the keyboard")
    });
    game.frames_until("the chapter after the card", ATTACK_CHANGES + 400, || {
        log.said(&format!("chapter 4 at frame {ATTACK_CHANGES}"))
    });
    game.dies();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said("retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said("retry: the run is given up")
    });
    game.frames_until_a_thread("the whole run written down", || {
        log.said(&format!(
            "replay: {SLOT}, 1 stage(s), 4 chapter(s), 5 attempt(s) written"
        ))
    });
    died_at
}

/// And that run's replay kept, which is what はじめから does to the one being written over: the run is
/// started again from the beginning and the replay of what was played becomes one of the saved.
fn its_replay_kept(game: &Fake) -> String {
    let log = game.log();
    game.frames_until("the title menu ready to act on a press", 300, || {
        let front = game.image().front_end_now();
        front.screen == Screen::Title && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered again", || {
        game.image().front_end_now().screen == Screen::ShotType
    });
    game.frames_until("the shot type select ready to act on a press", 90, || {
        game.image().front_end_now().acts_on_a_press()
    });
    game.press(keys::Z);
    game.frames(READS_KEYS_AFTER);
    game.press(keys::DOWN);
    game.press_until(keys::Z, "はじめから answered", || {
        log.said("resume: from the beginning, answered on the keyboard")
    });
    let kept = format!("{SLOT}-{DATED}");
    game.frames_until_a_thread("the replay kept", || {
        orb_core::replay::saved(game.dir()) == vec![kept.clone()]
    });
    // And that run given up in turn, so the game is back at the title menu with nothing being played.
    //
    // Waited for from *here* rather than by what the log has ever said: the run before this one reached its
    // own first chapter too, and a launch's log holds both.
    // The chapter with a snapshot behind it and not the stage's own start, which is written down before the
    // stage has settled: dying inside that wait costs a life rather than a chapter, so there is no menu to
    // give the run up on.
    let from = log.lines().len();
    game.frames_until("its own first chapter", 800, || {
        log.said_since(from, "stage 1 chapter 1 (stage start)")
    });
    game.dies();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said_since(from, "retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up again", || {
        log.said_since(from, "retry: the run is given up")
    });
    kept
}

/// The list waited for, and the screen it came up over asserted: the one the game keeps its own replays on,
/// which orb asks for and which takes its own frames to arrive.
///
/// **What is behind the list is what this is about.** The item somebody chose to get here leads to that
/// screen, so a list of replays over the title menu — or, coming back after an attempt, over th06's logo —
/// reads as having left the replays rather than as being in them.
fn the_list_over_the_games_own_replay_screen(game: &Fake, from: usize) {
    let log = game.log();
    game.frames_until("the screen the game keeps its own replays on", 300, || {
        log.said_since(
            from,
            "replay: the replays to watch, over the screen the game keeps its own on",
        )
    });
    assert_eq!(
        game.image().front_end_now().screen,
        Screen::Replays,
        "the list is not standing over the screen the game keeps its own replays on",
    );
}

/// The title menu with its cursor on the replay item, past the frames it ignores a press for.
fn at_the_replay_item(game: &Fake) {
    game.frames_until("the title menu ready to act on a press", 400, || {
        let front = game.image().front_end_now();
        game.image().scene() == Scene::FrontEnd
            && front.screen == Screen::Title
            && front.acts_on_a_press()
    });
    while game.image().front_end_now().cursor < item::REPLAY {
        game.press(keys::DOWN);
    }
    assert_eq!(
        game.image().front_end_now().cursor,
        item::REPLAY,
        "the cursor is not on the item the replays are behind",
    );
}

/// **完全無欠モード at the replay item, and the three lists behind it.** The attempt that was rewound away
/// is chosen and watched, and nothing of the run still being played is touched.
#[test]
fn a_replay_is_chosen_by_stage_chapter_and_attempt_and_watched() {
    in_its_own_process(|| {
        let game = keeping_every_replay("watching-a-replay");
        let log = game.log();
        let died_at = a_run_worth_watching(&game);
        let kept = its_replay_kept(&game);

        // ── 1. The item, and the question orb asks over it: which of the two kinds of replay to look at.
        //
        // Everything from here is read out of the log **since this line**: the runs above answered the mode
        // question twice between them, and a launch's log holds every line it ever wrote.
        let from = log.lines().len();
        at_the_replay_item(&game);
        game.press(keys::Z);
        assert!(
            log.said_since(from, "menu: Replays is under the cursor, asking which mode"),
            "the press did not put the question up:\n  {}",
            log.lines().join("\n  ")
        );
        game.forget();
        game.frame();
        assert_eq!(
            game.says(title(Menu::Replays, LANGUAGE)).len(),
            1,
            "the question over the replays is not the one asked about them",
        );

        // ── 2. 完全無欠モード, which is the cursor's own item: orb's own list, with whose run each replay
        // is and when it was played on it.
        game.press_until(keys::Z, "the mode question answered", || {
            log.said_since(from, "mode: answered on the keyboard")
        });
        game.frames_until("the list of replays", 60, || {
            log.said_since(from, "replay: 1 replay(s) to watch")
        });
        // **And it waits for the screen it stands over**, which is the one the game keeps its own replays on:
        // the item somebody chose leads there, so that is what is behind a list of replays.
        the_list_over_the_games_own_replay_screen(&game, from);
        game.forget();
        game.frame();
        assert_eq!(
            game.says("リプレイを選ぶ").len(),
            1,
            "the list has no title"
        );
        // **`中断` where a name would be**, this being the replay of a run written over by はじめから: a name
        // is what a run *ends* with in this game, so the name in `orb.yaml` is said about a run that was
        // played out and this line says instead that nobody finished this one.
        let listed = game.says(&format!("{INTERRUPTED}  {SLOT}  {DATED}"));
        assert_eq!(
            listed.len(),
            1,
            "the list does not say this replay is of a run nobody finished, and when it was played",
        );
        assert_eq!(listed[0].color, SELECTED, "the cursor is not on it");
        assert!(
            game.says(PLAYER).is_empty(),
            "a run nobody finished is listed under somebody's name",
        );

        // ── 3. The stages of it, which for this run is the one it played.
        game.press_until(keys::Z, "the replay chosen", || {
            log.said(&format!("replay: {kept} chosen; 1 stage(s) in it"))
        });
        game.forget();
        game.frame();
        assert_eq!(game.says("ステージを選ぶ").len(), 1);
        assert_eq!(game.says("STAGE 1").len(), 1, "the stage it played");

        // ── 4. The chapters of that stage, in the order the stage plays them.
        game.press_until(keys::Z, "the stage chosen", || {
            log.said_since(from, "replay: stage 1 chosen; 4 chapter(s) in it")
        });
        game.forget();
        game.frame();
        assert_eq!(game.says("チャプターを選ぶ").len(), 1);
        assert_eq!(game.says("MIDBOSS SPELL 1").len(), 1);
        assert_eq!(game.says("MIDSTAGE 1").len(), 1);

        // ── 5. The attempts at the third of them: the one that was rewound away, and the one that
        // survived. Two presses down from the chapter the cursor starts on.
        for _ in 0..2 {
            game.frames(READS_KEYS_AFTER);
            game.press(keys::DOWN);
        }
        game.press_until(keys::Z, "the chapter chosen", || {
            log.said_since(
                from,
                "replay: chapter 3 (MIDBOSS SPELL 1) chosen; 2 attempt(s) at it",
            )
        });
        game.forget();
        game.frame();
        assert_eq!(game.says("トライを選ぶ").len(), 1);
        assert_eq!(game.says("トライ 1").len(), 1);
        assert_eq!(game.says("トライ 2").len(), 1);
        // **And what was chosen to get here, above it**: three lists deep, a column of numbered attempts
        // says nothing about which chapter of which stage of whose run they are attempts at.
        assert_eq!(
            game.says(&format!("{INTERRUPTED}  {SLOT}  {DATED}")).len(),
            1,
            "the replay the attempts are out of is not on the screen",
        );
        assert_eq!(
            game.says("STAGE 1").len(),
            1,
            "the stage the attempts are out of is not on the screen",
        );
        assert_eq!(
            game.says("MIDBOSS SPELL 1").len(),
            1,
            "the chapter the attempts are at is not on the screen",
        );

        // ── 6. The first of them, which is the attempt that ended in the death: the stage is built, the
        // frames below the chapter are played in, and the landing is the frame that was written down.
        let left_behind = orb_core::replay::read_being_played(game.dir(), SLOT, &Th06)
            .expect("the run still being played");
        let chapter_file = orb_core::resume::left(game.dir());
        // What the log says from *here* is what watching an attempt did, which is what the last of these
        // assertions is about: the run given up on the way to this screen built a ranking of its own.
        let watching_from = log.lines().len();
        game.press_until(keys::Z, "the attempt chosen", || {
            log.said(&format!(
                "replay: watching attempt 1 of 2 at chapter 3 of stage 1, {} frame(s) of it",
                died_at - CARD_STARTS,
            ))
        });
        // **The run it is watched in is the front end's own, asked for and not written into being.** Which
        // is a line in the log and not a shape on the screen, and it is asserted because the other way round
        // is what took th06 down 129 frames into an attempt — see `Game::front_end_starts_a_run`.
        assert!(
            log.said_since(
                watching_from,
                "replay: the front end is asked for a run of stage 1"
            ),
            "the run was not asked of the front end:\n  {}",
            log.lines()[watching_from..].join("\n  "),
        );
        // **The frames that item takes to start it are in this wait**: the press handed over, the run
        // registered, the stage built and the buttons played in.
        game.frames_until("the stage built and the buttons played in", 400, || {
            log.said_since(
                from,
                "resume: the landing is the frame that was written down, field for field",
            )
        });
        assert_eq!(
            game.state().stage_frames,
            CARD_STARTS,
            "the playback did not land on the frame the chapter began on",
        );

        // ── 7. And then it is watched: the game plays on, a frame of the attempt to a frame of the game,
        // with the buttons the attempt was made with rather than the keyboard's.
        // **Fewer frames than the attempt holds**, which is the frames from the card to the death its hit
        // stands for — the window a bomb is taken in, eight frames on this laid-out game.
        assert!(
            died_at - CARD_STARTS > A_FEW_FRAMES,
            "the attempt is shorter than the frames it is watched over",
        );
        game.frames(A_FEW_FRAMES);
        assert_eq!(
            game.state().stage_frames,
            CARD_STARTS + A_FEW_FRAMES,
            "the attempt is not being played a frame at a time",
        );

        // ── 8. Nothing of the run somebody is in the middle of is touched by any of it.
        assert_eq!(
            orb_core::resume::left(game.dir()),
            chapter_file,
            "watching a replay wrote over the chapter a run was left in",
        );
        assert_eq!(
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th06),
            Some(left_behind),
            "watching a replay wrote over the replay of the run being played",
        );

        // ── 9. The attempt runs out, and orb takes the game out of the run it built for it: what is
        // watched is an attempt and not a run somebody carries on playing.
        game.frames_until("the attempt over", died_at - CARD_STARTS + 60, || {
            log.said_since(
                from,
                "replay: the attempt is over; the run it was watched in is left",
            )
        });
        game.frames_until("the front end again", 300, || {
            game.image().scene() == Scene::FrontEnd
        });
        assert!(
            !game.state().playing,
            "the run orb built to watch an attempt in is still being played",
        );

        // ── 10. And the list it was chosen out of is put back up, on the attempt it just watched: what
        // somebody who has watched one wants next is another attempt at that same chapter, and the walk down
        // to it is one they have already made.
        //
        // **And the screen a left run goes back to is not what the frames on the way show**, which is counted
        // rather than waited for: the screen the list stands over is asked for with the frames its own
        // coming-up state waits already spent, so it arrives on the update after the ask. One frame of
        // whatever was there is what the ask itself costs — the frame it is written on has already drawn —
        // and asking with those frames unspent cost a second of it.
        let mut coming_up = 0;
        for _ in 0..300 {
            if log.said_since(watching_from, "replay: the attempts at that chapter again") {
                break;
            }
            game.frame();
            if game.image().front_end_now().screen == Screen::ReplaysComingUp {
                coming_up += 1;
            }
        }
        assert!(
            coming_up <= 1,
            "the screen was coming up for {coming_up} frame(s), which are frames of the screen before it",
        );
        assert!(
            log.said_since(watching_from, "replay: the attempts at that chapter again"),
            "the list never came back:\n  {}",
            log.lines()[watching_from..].join("\n  "),
        );
        game.forget();
        game.frame();
        assert_eq!(
            game.says("トライを選ぶ").len(),
            1,
            "the list the attempt came out of is not back up",
        );
        let listed = game.says("トライ 1");
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].color, SELECTED,
            "the cursor is not on the attempt that was watched",
        );
        // And the game under it is the screen it keeps its own replays on, which orb asked the front end for
        // on the way out of the run: a list of replays over the logo a left run goes back to reads as having
        // left them.
        assert_eq!(game.image().front_end_now().screen, Screen::Replays);

        // ── 11. And it was not a run anybody played, which is the other half of that: the death the attempt
        // ended in offered no retry — a chapter cannot be gone back to in a replay of one — and what the
        // run counted was written into no ranking.
        assert!(
            !log.said_since(watching_from, "died in chapter"),
            "the death an attempt ends in was offered a retry:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said_since(watching_from, "score: the ranking built and taken down"),
            "the run an attempt was watched in was written into a ranking:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// The attempts at the card's chapter of the run [`a_run_worth_watching`] played, with the cursor on the
/// first of the two: the mode question answered, the replay chosen, its one stage chosen, and that chapter
/// chosen two items down the list of four.
///
/// `from` is where in the log this walk's own lines begin — the runs above it answered the mode question
/// twice between them, and a launch's log holds every line it ever wrote.
fn at_the_attempts_of_the_cards_chapter(game: &Fake, kept: &str, from: usize) {
    let log = game.log();
    at_the_replay_item(game);
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered", || {
        log.said_since(from, "mode: answered on the keyboard")
    });
    the_list_over_the_games_own_replay_screen(game, from);
    game.press_until(keys::Z, "the replay chosen", || {
        log.said(&format!("replay: {kept} chosen; 1 stage(s) in it"))
    });
    game.press_until(keys::Z, "the stage chosen", || {
        log.said_since(from, "replay: stage 1 chosen; 4 chapter(s) in it")
    });
    for _ in 0..2 {
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
    }
    game.press_until(keys::Z, "the chapter chosen", || {
        log.said_since(
            from,
            "replay: chapter 3 (MIDBOSS SPELL 1) chosen; 2 attempt(s) at it",
        )
    });
}

/// **The attempt that survived a chapter is that chapter and stops where it ended.** It is the path the run
/// went on with, so the frames after the chapter are the chapters that follow it — and what was chosen here
/// came out of a list of one chapter's attempts. Watched to the end of what was played instead, choosing the
/// third chapter of a stage played the twelve after it as well, and the list came back only at the end of
/// that.
#[test]
fn the_attempt_that_survived_stops_where_its_chapter_ended() {
    in_its_own_process(|| {
        let game = keeping_every_replay("watching-the-attempt-that-survived");
        let log = game.log();
        a_run_worth_watching(&game);
        let kept = its_replay_kept(&game);

        let from = log.lines().len();
        at_the_attempts_of_the_cards_chapter(&game, &kept, from);
        // The second of the two attempts, which is the one the run went on from.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        let watching_from = log.lines().len();
        game.press_until(keys::Z, "the attempt that survived chosen", || {
            log.said(&format!(
                "replay: watching attempt 2 of 2 at chapter 3 of stage 1, {} frame(s) of it",
                ATTACK_CHANGES - CARD_STARTS,
            ))
        });
        game.frames_until("the stage built and the buttons played in", 400, || {
            log.said_since(
                watching_from,
                "resume: the landing is the frame that was written down, field for field",
            )
        });

        // And it is over inside the frames of that chapter: what the run it was watched in has played by
        // then is this chapter and none of the chapter after it.
        game.frames_until(
            "the attempt over where the chapter ended",
            ATTACK_CHANGES - CARD_STARTS + 60,
            || {
                log.said_since(
                    watching_from,
                    "replay: the attempt is over; the run it was watched in is left",
                )
            },
        );
        game.frames_until("the attempts again", 300, || {
            log.said_since(watching_from, "replay: the attempts at that chapter again")
        });
    });
}

/// **The pad stops a watched attempt too, on the button the game's own menus take a cancel on.** Somebody
/// watching one has a controller in their hands, and the run's own pause is not theirs to open — the buttons
/// the game acts on are the attempt's — so a stop that only the keyboard could ask for left a pad with no way
/// out of a chapter four minutes long. Which is how it was: escape worked and the pad's menu button did
/// nothing.
#[test]
fn the_pads_own_cancel_stops_a_watched_attempt() {
    in_its_own_process(|| {
        let game = keeping_every_replay("stopping-a-watch-on-the-pad");
        let log = game.log();
        a_run_worth_watching(&game);
        let kept = its_replay_kept(&game);

        let from = log.lines().len();
        at_the_attempts_of_the_cards_chapter(&game, &kept, from);
        // The attempt that survived, which is the chapter's own frames: the one that was rewound away ends
        // in the death nine frames later, and a stop has to be asked for while there is still an attempt to
        // stop.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        let watching_from = log.lines().len();
        game.press_until(keys::Z, "the attempt chosen", || {
            log.said("replay: watching attempt 2 of 2 at chapter 3 of stage 1")
        });
        game.frames_until("the stage built and the buttons played in", 400, || {
            log.said_since(
                watching_from,
                "resume: the landing is the frame that was written down, field for field",
            )
        });

        // Past the frames the attempt reads no stop over, which are there for a button still down from the
        // list it was chosen out of — and well inside the chapter, so that what ends the attempt is the
        // button and not its own last frame.
        game.frames(30);
        game.push(Pushed::button(MAPPING.menu));
        game.frames_until("the attempt stopped on the pad", 8, || {
            log.said_since(
                watching_from,
                &format!(
                    "replay: the attempt is over; the run it was watched in is left, asked for on the {}",
                    By::Pad,
                ),
            )
        });
        // And the list comes back the same way it does when an attempt runs out on its own.
        game.frames_until("the attempts again", 300, || {
            log.said_since(watching_from, "replay: the attempts at that chapter again")
        });
    });
}

/// **レガシーモード at the same item hands the press over**, and the game opens the replays it saved itself:
/// those are the game's own screens and orb has nothing to put on them.
#[test]
fn the_games_own_replays_are_the_games_own_screen() {
    in_its_own_process(|| {
        let game = keeping_every_replay("watching-the-games-own");
        let log = game.log();
        at_the_replay_item(&game);
        game.press(keys::Z);
        assert!(log.said("menu: Replays is under the cursor, asking which mode"));

        // One press down from 完全無欠モード, after the frames the question reads nothing over.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "レガシーモード answered", || {
            log.said("mode: normal, was pointdevice")
        });
        assert!(
            !log.said("replay(s) to watch"),
            "a list of orb's own went up for the game's own replays:\n  {}",
            log.lines().join("\n  ")
        );
        // The press held back for the question is handed over, so the item is chosen by the game's own
        // menu — which is the whole of what orb owes that answer, the screen behind it being the game's.
        game.frames_until("the press handed over", 60, || {
            game.asked_for_its_own_replays() == 1
        });
    });
}

/// A run of two stages, hit at a chapter of each: what it leaves is a stage list of two, and a chapter of
/// the second stage with two attempts at it.
fn two_stages_worth_watching(game: &Fake) {
    let log = game.log();
    // Long enough for the four chapters a stage of this laid-out game has and the frames a retry at one of
    // them takes, and short enough that two stages fit in an e2e test.
    game.stages_last(ATTACK_CHANGES + 900);
    game.in_a_pointdevice_run();
    // Stage 1, hit at the card's chapter and retried: two attempts at chapter 3 of it.
    game.frames_until("the card's chapter", CARD_STARTS + 400, || {
        log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
    });
    game.hit();
    game.frames_until(
        "the death the hit stands for",
        A_BOMB_IS_TAKEN_FOR as u32 + 2,
        || log.said("died in chapter 3"),
    );
    game.press_until(keys::Z, "the chapter retried", || {
        log.said("retry: the chapter again on the keyboard")
    });
    // And the stage run out, which is where the next one begins.
    //
    // **Every wait from here is read off the log from where the wait before it stopped**, and the chapter
    // ones name the stage: the retry above puts the same chapter up again, so a stage's own lines are said
    // twice over and a wait on the whole log is a wait that was over before it started — which is a death
    // taken in a stage's first frames, where there is no chapter to be sent back to.
    let stage_two = log.lines().len();
    game.frames_until("the stage after it", ATTACK_CHANGES + 900, || {
        log.said_since(stage_two, "stage 2 chapter 1 (stage start)")
    });
    // Stage 2, the same again: to its card's chapter, hit, retried.
    let its_card = log.lines().len();
    game.frames_until("the card's chapter of stage 2", CARD_STARTS + 400, || {
        log.said_since(its_card, "stage 2 chapter 3 (")
    });
    let hit_at = log.lines().len();
    game.hit();
    game.frames_until(
        "the death the hit stands for",
        A_BOMB_IS_TAKEN_FOR as u32 + 2,
        || log.said_since(hit_at, "died in chapter 3"),
    );
    let retried = log.lines().len();
    game.press_until(keys::Z, "the chapter of stage 2 retried", || {
        log.said_since(retried, "retry: the chapter again on the keyboard")
    });
    // Given up past the chapter after the card, so that stage 2 has four chapters of its own — and the
    // retry is behind it, a rewind being the snapshot put back rather than frames played in.
    let after_the_card = log.lines().len();
    game.frames_until("the chapter after the card", ATTACK_CHANGES + 400, || {
        log.said_since(after_the_card, "stage 2 chapter 4 (")
    });
    let given_up = log.lines().len();
    game.dies();
    game.frame();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the give-up asking", || {
        log.said_since(given_up, "retry: asking about the run given up")
    });
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, "the run given up", || {
        log.said_since(given_up, "retry: the run is given up")
    });
    game.frames_until_a_thread("the whole run written down", || {
        log.said(&format!("replay: {SLOT}, 2 stage(s), "))
    });
}

/// **The second stage's own chapter, and the second attempt at it.** Every list here has more than one item
/// and none of the three answers is its first: the stage is the one after the run's first, the chapter is
/// the third of that stage's, and the attempt is the one that came after the death.
///
/// **Which is what says a particular attempt is what gets watched**, and the stage is the half of it that
/// nothing else covers: the run the attempt is watched in is started by the game's own item, and that item
/// starts every run at the stage the menu would have — so a stage of any other number has to be put in after
/// it. A watch that came up in stage 1 for an attempt made in stage 2 would play the right buttons into the
/// wrong stage.
#[test]
fn the_second_stage_and_the_second_attempt_are_what_the_lists_reach() {
    in_its_own_process(|| {
        let game = keeping_every_replay("watching-a-later-attempt");
        let log = game.log();
        two_stages_worth_watching(&game);
        let kept = its_replay_kept(&game);

        let from = log.lines().len();
        at_the_replay_item(&game);
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered", || {
            log.said_since(from, "mode: answered on the keyboard")
        });
        the_list_over_the_games_own_replay_screen(&game, from);
        // The one replay, and the two stages in it.
        game.press_until(keys::Z, "the replay chosen", || {
            log.said(&format!("replay: {kept} chosen; 2 stage(s) in it"))
        });
        game.forget();
        game.frame();
        assert_eq!(game.says("STAGE 1").len(), 1);
        assert_eq!(game.says("STAGE 2").len(), 1, "the stage it went on to");

        // ── The second stage: one press down the list of two.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the second stage chosen", || {
            log.said_since(from, "replay: stage 2 chosen; 4 chapter(s) in it")
        });
        // ── Its third chapter: two presses down from the one the cursor starts on.
        for _ in 0..2 {
            game.frames(READS_KEYS_AFTER);
            game.press(keys::DOWN);
        }
        game.press_until(keys::Z, "the chapter chosen", || {
            log.said_since(
                from,
                "replay: chapter 3 (MIDBOSS SPELL 1) chosen; 2 attempt(s) at it",
            )
        });
        // ── And the second attempt at it: one press down the list of two.
        game.forget();
        game.frame();
        assert_eq!(game.says("トライ 2").len(), 1);
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        let watching_from = log.lines().len();
        game.press_until(keys::Z, "the second attempt chosen", || {
            log.said("replay: watching attempt 2 of 2 at chapter 3 of stage 2")
        });
        // **The run comes up in the stage the attempt was made in**, which the front end's own item does not
        // do by itself.
        game.frames_until("the stage built and the buttons played in", 400, || {
            log.said_since(
                watching_from,
                "resume: the landing is the frame that was written down, field for field",
            )
        });
        assert_eq!(
            game.state().stage,
            1,
            "the attempt is being watched in another stage than the one it was made in",
        );
        assert_eq!(
            game.state().stage_frames,
            CARD_STARTS,
            "the playback did not land on the frame the chapter began on",
        );
        // And it plays on from there, a frame of the attempt to a frame of the game.
        game.frames(10);
        assert_eq!(game.state().stage_frames, CARD_STARTS + 10);
        assert_eq!(game.state().stage, 1);
    });
}

/// A run cleared, whose replay is therefore one whose last chapter's last attempt is the path that *finished*
/// it: the boss's death is in those frames.
///
/// Kept without asking, this launch's `save_replays` being on, and the screens whoever cleared it reads are
/// pressed through the way they are by whoever cleared it — the name entry is theirs to type at.
fn a_run_cleared(game: &Fake) -> String {
    let log = game.log();
    game.stages_last(STAGE_FRAMES);
    game.in_a_pointdevice_run();
    game.frames_until("the run cleared", A_WHOLE_RUN, || {
        game.image().scene() == Scene::Result
    });
    let kept = format!("{SLOT}-{DATED}");
    game.frames_until_a_thread("the replay of it kept", || {
        orb_core::replay::saved(game.dir()) == vec![kept.clone()]
    });
    game.press_until(keys::Z, "the front end after the run", || {
        game.image().scene() == Scene::FrontEnd
    });
    log.lines();
    kept
}

/// **An attempt that finished the run is watched out of the run all the same.** The last attempt at the last
/// chapter of a cleared run's last stage is the path the boss died on, so playing it back kills the boss
/// again and the game asks for the screens a finished run ends at — the name entry, and the score written out
/// of what the run counted.
///
/// **Neither of which is this run's to have.** It is not a run anybody played: nobody is going to type a name
/// into it, and what it counted was counted once already by the run it came out of. So orb takes the game out
/// of those screens on the frame they come up, and the list the attempt was chosen out of comes back — with
/// nothing pressed at all, which is what this asserts by pressing nothing.
#[test]
fn an_attempt_that_finished_the_run_is_taken_out_of_the_screens_it_ends_at() {
    in_its_own_process(|| {
        let game = keeping_every_replay("watching-the-attempt-that-cleared");
        let log = game.log();
        let kept = a_run_cleared(&game);

        let from = log.lines().len();
        at_the_replay_item(&game);
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered", || {
            log.said_since(from, "mode: answered on the keyboard")
        });
        the_list_over_the_games_own_replay_screen(&game, from);
        // **And this one is listed under a name**, the run having been played out: the other side of what the
        // interrupted runs above are listed as.
        game.forget();
        game.frame();
        assert_eq!(
            game.says(&format!("{PLAYER}  {SLOT}  {DATED}")).len(),
            1,
            "a run that was played out is not listed under whose orb this is",
        );
        game.press_until(keys::Z, "the replay chosen", || {
            log.said(&format!("replay: {kept} chosen; {STAGES} stage(s) in it"))
        });
        // **The last of each list, which is one press *up* from the first**: a cursor is a thing that goes
        // round, so the item before the first is the last however many there are — and how many stages a run
        // reached, or chapters a stage had, is not something this test should have to count.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the last stage chosen", || {
            log.said_since(from, &format!("replay: stage {STAGES} chosen;"))
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "its last chapter chosen", || {
            log.said_since(from, "attempt(s) at it")
        });
        let watching_from = log.lines().len();
        game.press_until(keys::Z, "the attempt chosen", || {
            log.said_since(watching_from, "replay: watching attempt")
        });

        // ── The attempt plays out, the boss dies in it, and the run ends itself — so there is no "the
        // attempt is over" line here, the frame it would have been said on being one the run had already
        // gone on. From here **nothing is pressed**: what the game does with the screens a finished run ends
        // at is orb's to answer for a run nobody played, and the list coming back is the answer.
        //
        // **And not one frame of those screens is drawn**, which is counted rather than waited for: they are
        // run out inside the frame they came up in, the way an ending is, so a run nobody played never puts
        // its name entry or its stats on the screen at all.
        let mut on_the_screen = 0;
        for _ in 0..A_WHOLE_RUN {
            if log.said_since(watching_from, "replay: the attempts at that chapter again") {
                break;
            }
            game.frame();
            if game.image().scene() == Scene::Result {
                on_the_screen += 1;
            }
        }
        assert_eq!(
            on_the_screen,
            0,
            "the screens a finished run ends at were drawn for {on_the_screen} frame(s):\n  {}",
            log.lines()[watching_from..].join("\n  "),
        );
        assert!(
            log.said_since(watching_from, "replay: the attempts at that chapter again"),
            "the list never came back:\n  {}",
            log.lines()[watching_from..].join("\n  "),
        );
        game.forget();
        game.frame();
        assert_eq!(
            game.says("トライを選ぶ").len(),
            1,
            "the list the attempt came out of is not back up",
        );
        // And the name entry never stood: the state the screen was left in is the one orb writes, which is
        // the game's own way out of it.
        assert!(
            !log.said_since(
                watching_from,
                "score: pointdevice_score.dat opened in place of the game's own, write"
            ),
            "what the watched run counted was written into the ranking:\n  {}",
            log.lines()[watching_from..].join("\n  "),
        );
    });
}

/// And a launch with no saved replays: the list says so rather than nothing appearing, which is what a press
/// held back for a screen that never came would be.
#[test]
fn a_launch_with_no_saved_replays_says_so() {
    in_its_own_process(|| {
        let game = keeping_every_replay("watching-with-none");
        let log = game.log();
        at_the_replay_item(&game);
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        game.frames_until("the list of replays", 60, || {
            log.said("replay: 0 replay(s) to watch")
        });
        the_list_over_the_games_own_replay_screen(&game, 0);
        game.forget();
        game.frame();
        assert_eq!(
            game.says("保存されたリプレイがありません").len(),
            1,
            "a list with nothing in it does not say so",
        );
        // And a cancel is the way out of it, onto the screen the game keeps its own replays on: that screen
        // was asked for so the list could stand over it, and it carries on where it was — which is the
        // screen whoever chose that item would have been on, with its own cancel as the way back.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::X);
        game.frames_until("the list closed", 60, || {
            log.said("replay: nothing chosen; the menu is where it was")
        });
        assert_eq!(game.image().front_end_now().screen, Screen::Replays);
        game.frames(READS_KEYS_AFTER);
        game.press(keys::X);
        game.frames_until("the game's own screen left as well", 60, || {
            game.image().front_end_now().screen == Screen::Title
        });
    });
}

/// **A launch playing itself walks to the item and answers the question over it**, which is the whole of the
/// way in reached without a keyboard: the cursor to the replays, orb's question raised by the recipe's own
/// press, and 完全無欠モード taken by its place in the list.
///
/// **The press a bot makes goes through the same holding a keyboard's does.** Four launches on the real game
/// pressed at this item and every one of them opened the game's own replay list: the word a recipe presses is
/// handed to the game where the read begins, and orb's question is raised by the read that takes the decide
/// *out* — so a bot pressing at the title reached no question of orb's at all, and the mode question, which is
/// the way into every one of these lists, could not be reached by a launch nobody was at the keyboard for.
#[test]
fn a_recipe_reaches_the_question_over_the_replay_item_and_answers_it() {
    in_its_own_process(|| {
        let game = Fake::attach("watching-by-a-recipe", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
            config.player_name = PLAYER.to_owned();
            // The cursor walked to the replays and the press made at it, both tapped: the title menu ignores
            // a press for its own first frames, and a key already down when it stops ignoring them is a key
            // no read of orb's sees go down.
            config.autoplay = Some(format!(
                "- press: [down]\n  frames: 6\n  until: {{ cursor: {} }}\n  give_up_after: 900\n\
                 - press: [decide]\n  frames: 30\n  until: {{ question: mode }}\n  give_up_after: 900\n\
                 - answer: {{ question: mode, item: 0 }}\n  give_up_after: 120\n",
                item::REPLAY,
            ));
        });
        let log = game.log();

        game.frames_until("the mode question over the replay item", 900, || {
            log.said("menu: Replays is under the cursor, asking which mode")
        });
        game.frames_until("完全無欠モード answered by its item", 300, || {
            log.said("mode: answered on the recipe's own ask")
        });
        // And what that answer opens is orb's own list, not the game's: nothing was kept by this launch, so
        // what it holds is the line that says so.
        game.frames_until("the list of replays", 60, || {
            log.said("replay: 0 replay(s) to watch")
        });
        the_list_over_the_games_own_replay_screen(&game, 0);
        game.forget();
        game.frame();
        assert_eq!(
            game.says("保存されたリプレイがありません").len(),
            1,
            "the list the recipe opened is not orb's own",
        );
        assert_eq!(
            game.asked_for_its_own_replays(),
            0,
            "the press reached the game's own replay list",
        );
    });
}
