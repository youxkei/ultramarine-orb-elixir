//! **A replay of a pointdevice run of th07 kept and watched again.**
//!
//! What the machinery is and why it exists is `watching_a_pointdevice_replay.rs` and
//! [docs/adr/0024](../../../docs/adr/0024-a-pointdevice-runs-replay-is-orbs-own-and-is-watched-through-the-resumes-machinery.md);
//! all of it is one shape for both games, the record and the file included. **What is this game's own is the
//! run the attempt is watched in.** Every other run orb puts a chapter back into was chosen on the game's own
//! screens; this one is chosen out of a file, so orb writes the three numbers those screens write, the
//! combined byte every table of either is indexed by, and the scene — and this game's supervisor keeps
//! `wantedState` and `curState` the other way round from th06's, so a run asked for here is the *first* of
//! them written.
//!
//! Which is what this drives: the item on the title menu, the mode over it, the lists, and a run built out of
//! a file playing an attempt back into the stage it was made in.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th07::Th07;
use orb_core::menu_ui::SELECTED;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stretch of the fight this run plays is, which is what the e2e tests about this game's
/// chapters drive it in.
const APART: u32 = 120;

/// The card the midboss of this run's stage runs, and the name the game puts on the plate for it — the same
/// pair `a_chapter_further_back_of_th07.rs` drives its own fight with.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// And what the boundary detector calls the chapter that card began, which is what the list of them shows.
const THE_CARDS_CHAPTER: &str = "MIDBOSS SPELL 1";

/// The name orb gives the run this plays — Normal, and Reimu's first shot — which is what names both of its
/// files.
const SLOT: &str = "normal-reimu-a";
/// The date the simulated host's calendar is set to, which a kept replay's name carries.
const DATED: &str = "20040815-214233";
/// And whose runs these are, as `orb.yaml` says — which is what the list says about a run that was played
/// out. The run here is one nobody finished, so its own line says that instead.
const PLAYER: &str = "ゆうか";
const INTERRUPTED: &str = "中断";

/// A launch of this game whose replays are kept without asking, so that what this drives is the watching.
fn keeping_every_replay(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
            config.player_name = PLAYER.to_owned();
        },
    );
    game.streams_its_song(0);
    game
}

/// A run played into its midboss's card, hit there, the chapter retried, and played on to the chapter after
/// it — one chapter with two attempts at it and three with one apiece.
///
/// Returns the frame the attempt that was rewound away ended on: the retry menu freezes the game, so the
/// frame it is standing on is the frame the death happened on.
fn a_run_worth_watching(game: &Fake) -> u32 {
    let log = game.log();
    game.in_a_pointdevice_run();
    game.frames(APART);
    game.the_boss_arrives();
    game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
    game.frames(APART);
    game.a_card_starts(CARD, CARD_NAME);
    game.frames_until("the card's chapter", APART, || log.said("chapter 3"));

    // A stretch of that chapter played before the death, so that the attempt it makes is one long enough
    // for a playback of it to be watched *going*: an attempt two frames long is over before anything can
    // say it was being played a frame at a time.
    game.frames(APART);
    game.dies();
    game.one_frame();
    assert!(
        log.said("died in chapter 3"),
        "the death was not noticed:\n  {}",
        log.lines().join("\n  ")
    );
    let died_at = game.state().stage_frames;
    let from = log.written();
    game.press_until(keys::Z, "the chapter retried", || {
        log.said_since(from, "retry: the chapter again on the keyboard")
    });

    game.frames(APART);
    game.the_card_is_over();
    game.the_attack_changes();
    game.frames_until("the chapter after the card", APART, || {
        log.said_since(from, "chapter 4")
    });
    died_at
}

/// **The whole of it over a run of th07**: the run recorded with its attempts, the replay kept where the run
/// left unfinished is written over, and one attempt watched out of the lists.
#[test]
fn a_replay_of_a_run_of_th07_is_chosen_by_stage_chapter_and_attempt_and_watched() {
    in_its_own_process(|| {
        let game = keeping_every_replay("th07-watching-a-replay");
        let log = game.log();
        let died_at = a_run_worth_watching(&game);

        // ── 1. The run given up, which leaves both files behind — and the write that puts the stretch the
        // run ended in into the replay.
        game.gives_the_run_up();
        game.frames_until_a_thread("the whole run written down", || {
            log.said(&format!(
                "replay: {SLOT}, 1 stage(s), 4 chapter(s), 5 attempt(s) written"
            ))
        });
        let left = orb_core::replay::read_being_played(game.dir(), SLOT, &Th07)
            .expect("the replay of the run that was left");
        assert_eq!(left.name, PLAYER);
        assert_eq!(
            left.stages[0].tries(&left.stages[0].chapters[2]),
            2,
            "the chapter that was retried is not the one with two attempts at it",
        );

        // ── 2. はじめから, which is where the replay of the run being written over is kept: the same run
        // chosen again and the second of the two answers given.
        let from = log.written();
        game.at_the_shot_select();
        game.press_until(keys::Z, "the question about the run left behind", || {
            log.said_since(from, "asking where to start")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said_since(from, "resume: from the beginning, answered on the keyboard")
        });
        let kept = format!("{SLOT}-{DATED}");
        game.frames_until_a_thread("the replay kept", || {
            orb_core::replay::saved(game.dir()) == vec![kept.clone()]
        });

        // And that run given up in turn, so the game is standing at its title menu with nothing playing.
        game.image().a_run_has_been_chosen();
        game.builds_a_stage();
        game.frames_until("its own first chapter", 600, || {
            log.said_since(from, "stage 1 chapter 1 (stage start)")
        });
        game.gives_the_run_up();

        // ── 3. The replay item, and the question orb asks over it.
        let from = log.written();
        game.image().the_title_menu_points_at_the_replays();
        // A frame with the cursor there before the press: what orb holds a press back for is read after the
        // game's update and applies to the *next* read — see `held_back` — so a press on the frame the
        // cursor arrives on is one the game acts on itself.
        game.frames(2);
        game.press(keys::Z);
        assert!(
            log.said_since(from, "menu: Replays is under the cursor, asking which mode"),
            "the press did not put the question up:\n  {}",
            log.lines().join("\n  ")
        );

        // ── 4. 完全無欠モード, and the list of replays with whose run this was and when on it.
        game.press_until(keys::Z, "the mode question answered", || {
            log.said_since(from, "mode: answered on the keyboard")
        });
        game.frames_until("the list of replays", 60, || {
            log.said_since(from, "replay: 1 replay(s) to watch")
        });
        let listed = game.a_frame_that_says(&format!("{INTERRUPTED}  {SLOT}  {DATED}"));
        assert_eq!(
            listed.len(),
            1,
            "the list does not say this replay is of a run nobody finished, and when it was played",
        );
        assert_eq!(listed[0].color, SELECTED, "the cursor is not on it");

        // ── 5. The stages, the chapters and the attempts, each list chosen out of in turn.
        game.press_until(keys::Z, "the replay chosen", || {
            log.said_since(from, &format!("replay: {kept} chosen; 1 stage(s) in it"))
        });
        game.press_until(keys::Z, "the stage chosen", || {
            log.said_since(from, "replay: stage 1 chosen; 4 chapter(s) in it")
        });
        // Two presses down from the chapter the cursor starts on, which is the card's.
        for _ in 0..2 {
            game.frames(READS_KEYS_AFTER);
            game.press(keys::DOWN);
        }
        game.press_until(keys::Z, "the chapter chosen", || {
            log.said_since(
                from,
                &format!("replay: chapter 3 ({THE_CARDS_CHAPTER}) chosen; 2 attempt(s) at it"),
            )
        });

        // ── 6. The attempt that ended in the death, watched: the run is built out of the file, the frames
        // below the chapter are played in, and the landing is the frame that was written down.
        let chapter_file = orb_core::resume::left(game.dir());
        let being_played = orb_core::replay::read_being_played(game.dir(), SLOT, &Th07);
        game.press_until(keys::Z, "the attempt chosen", || {
            log.said_since(
                from,
                "replay: watching attempt 1 of 2 at chapter 3 of stage 1",
            )
        });
        game.frames_until("the attempt put in the way of the run", 600, || {
            log.said_since(from, "frame(s) of it to watch from there")
        });
        // The build orb asked for by writing the scene the item that starts a run writes — which is what
        // `Th07::start_a_run` does, and this is the game doing what that ask means.
        game.builds_a_stage();
        game.frames_until("the buttons played in", 600, || {
            log.said_since(
                from,
                "resume: the landing is the frame that was written down, field for field",
            )
        });

        // ── 7. And it is watched a frame at a time, with nothing of the run somebody is in the middle of
        // touched by any of it.
        let landed = game.state().stage_frames;
        game.frames(10);
        assert_eq!(
            game.state().stage_frames,
            landed + 10,
            "the attempt is not being played a frame at a time",
        );
        assert_eq!(
            orb_core::resume::left(game.dir()),
            chapter_file,
            "watching a replay wrote over the chapter a run was left in",
        );
        assert_eq!(
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th07),
            being_played,
            "watching a replay wrote over the replay of the run being played",
        );

        // ── 8. The attempt runs out, and orb takes the game out of the run it built for it.
        game.frames_until("the attempt over", died_at + 120, || {
            log.said_since(
                from,
                "replay: the attempt is over; the run it was watched in is left",
            )
        });
        game.frames_until("the run gone", 600, || !game.state().playing);
    });
}
