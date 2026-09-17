//! **The frame orb paces th06c with, and what it reads of one.**
//!
//! A th06c laid out from what has been read of `th06c.exe`, orb attached to `Th06c`, and orb's own frame
//! loop in place of the game's own frame — which is the only frame a launch of this game has, the game
//! handing its two walks and everything around them over. What the e2e tests below ask is that the loop
//! composes th06c's frame: the update, the sound's catch-up, the draw and the present in orb's order, the
//! viewport put in place before the update and the picture finished after the draw, through the calls the
//! game handed over. Beside that, one field at a time: the scene, the stage, the run's own numbers, the
//! player's states, the fight, and the score file the mode forks through `fopen`.
//!
//! **The mode itself is `a_pointdevice_run_of_th06c.rs`**, which plays a run of the same laid-out game
//! through the whole of it by a recipe — the question, a chapter, a death, the retry, the run left and
//! picked up. What orb *draws* over the same game is `the_mode_question_over_th06c.rs`, which reads a
//! question of orb's off the screen it is put on through this game's own Direct3D 11.
//!
//! What these cannot say is that the addresses *are* right: a laid-out th06c is written from the same
//! constants `Th06c` reads. `cargo xtask real` on the copy of the install is what says.

use crate::fake::th06c::Fake;
use crate::fake::{DRAW, Display, Launched, PRESENT, SOUND, UPDATE, Work, in_its_own_process};
use orb_core::game::th06c::Th06c;
use orb_core::game::{CountedDeath, Game};

/// What the game's own frame is declared to take: about what a real run's report line shows.
const WORK_US: i64 = 700;

/// The whole of this game's output, which is what the frame's viewport covers before the update narrows it.
const THE_OUTPUT: [u32; 4] = [0, 0, 640, 480];

/// The game's own score file, as its `fopen` is handed the name: relative, resolving where the game's own
/// open would.
const THE_GAMES_FILE: &str = "score.dat";

/// The run a read-back is laid out with: Normal, the first stage, a shot part way up, a score with digits in
/// it, a generator that has drawn a few times, three hundred frames in.
const DIFFICULTY: i32 = 1;
const LIVES: i8 = 2;
const BOMBS: i8 = 3;
const POWER: u16 = 96;
const SCORE: u32 = 1_234_560;
const POINT_ITEMS: u16 = 12;
const SEED: u16 = 0x1a2b;
const RANDOMS: u32 = 7;
const AT: u32 = 300;

#[test]
fn orb_paces_a_launch_of_th06c_with_no_run_in_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06c-paced",
            Work::wandering(WORK_US),
        );
        let log = game.log();

        // The frame is orb's own, in orb's order: the update first, the draw, and the frame handed to the
        // display through this game's own swapchain.
        //
        // **And nothing of the sound on a frame like this one**, which is a run at its own speed: this
        // game's catch-up is a function of orb's rather than one of the game's — see `FrameCalls::orbs_own`
        // — and what it does is turn walks the music has not heard into a seek, so a frame that walked the
        // calc chain once asks the game for nothing at all. The frame below is one that has.
        //
        // **Not the launch's first frame, which hands the frame over twice.** The game's own composite
        // presents — `Game::end_drawing`, and `fake::th06c::finishes_the_picture` behind it — and the
        // search that takes the swapchain runs in the present call *after* it, so on the first frame the
        // game's own present goes straight through and orb's loop then presents as well. From the second
        // frame the slot holds orb's stand-in, which says the frame is presented and leaves the loop's own
        // call nothing to do. The real game's first frame does the same; what a frame is composed of is
        // what this asks about, so it is asked of a frame that is not the first.
        game.frame();
        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), vec![UPDATE, DRAW, PRESENT]);

        // And the sound's own place in that order, on a frame with music behind it: between the update and
        // the draw, as this game's own frame has it, with the count cleared and the track seeked by the
        // frames' worth of samples the music missed.
        game.image().plays_the_track(3);
        game.image().the_music_is_behind_by(2);
        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), vec![UPDATE, SOUND, DRAW, PRESENT]);
        assert_eq!(
            game.image().the_music_is_behind_by_now(),
            0,
            "the catch-up did not clear the walks it caught up on",
        );
        assert_eq!(
            game.seeks_asked_for(),
            vec![(44_100 / 60 * 2, 3)],
            "the music was not seeked by the two walks it was behind",
        );

        // And the two calls either side of them that are this game's own frame's: the viewport over the whole
        // output before the update, and the picture finished after the draw — one apiece per frame, through
        // the functions the game handed over.
        // Counted from where they stand rather than from nought, so that a frame run above is one this does
        // not have to know about.
        let (viewports, pictures) = (game.viewports_set().len() as u32, game.pictures_finished());
        let frames = 30;
        game.frames(frames);
        assert_eq!(
            game.viewports_set().len() as u32 - viewports,
            frames,
            "the viewport was not set once a frame"
        );
        assert!(
            game.viewports_set()
                .iter()
                .all(|viewport| *viewport == THE_OUTPUT),
            "a viewport was set to something other than the whole output: {:?}",
            game.viewports_set()
        );
        assert_eq!(
            game.pictures_finished() - pictures,
            frames,
            "the picture was not finished once a frame"
        );

        // A game orb can rewind starts in the mode orb is for — and the one read it makes of its score file
        // at startup, the file every screen of it parses afterwards, landed in the game's own all the same:
        // that read is the one the title's unlocks come out of, and nobody has been asked which mode yet.
        assert!(
            log.said("mode: pointdevice to start with; the menu asks which"),
            "the launch did not start in the mode orb is for: {:?}",
            log.lines()
        );
        assert_eq!(
            game.score_file_opens().first(),
            Some(&(THE_GAMES_FILE.to_owned(), false)),
            "the game's own read of its score file did not land in its own: {:?}",
            game.score_file_opens()
        );
        assert!(
            log.said("score: score.dat opened as the game's own, read"),
            "the fork did not say where the read landed: {:?}",
            log.lines()
        );
    });
}

/// Every field of a `State` against what the game's memory holds, one number at a time — which is what a
/// wrong offset would show as, and what the mode is decided from.
#[test]
fn every_number_a_stage_of_th06c_is_played_with_reads_back_as_the_game_put_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06c-read-back",
            Work::wandering(WORK_US),
        );
        let image = game.image();
        image.is_playing(0, DIFFICULTY);
        image.has_lives_and_bombs(LIVES, BOMBS);
        image.shot_is_at_power(POWER);
        image.the_run_has_scored(SCORE, POINT_ITEMS);
        image.the_generator_is(SEED, RANDOMS);
        image.has_been_in_the_stage_for(AT);
        image.the_script_has_run_for(AT as i32);
        image.the_player_is_playing();

        let state = game.state();
        assert!(state.playing && state.in_run && state.in_game);
        assert!(!state.replay && !state.demo && !state.practice && !state.paused);
        assert_eq!(state.stage, 0, "counted from zero above `Game`");
        assert_eq!(state.difficulty, DIFFICULTY);
        assert_eq!(state.stage_frames, AT);
        assert_eq!(state.script_frames, AT as i32);
        assert_eq!(state.random_seed, SEED, "the generator itself");
        assert_eq!(state.lives, LIVES);
        assert_eq!(state.bombs, BOMBS);
        assert_eq!(state.power, POWER);
        assert_eq!(state.deaths, 0);
        assert!(!state.unsettled && !state.bombing && !state.in_dialogue);
        assert!(!state.boss_present);
        assert_eq!(state.spellcard, None);
        assert_eq!(game.state(), state, "reading it twice gave two answers");

        // The run's own numbers as this game's resume carries them, read where its replay record reads them.
        let numbers = unsafe { Th06c.run_state() };
        let numbers = orb_core::game::th06c::resume::decode_state(&numbers)
            .expect("the run's numbers are this game's own");
        assert_eq!(numbers.score, SCORE);
        assert_eq!(numbers.point_items, POINT_ITEMS);
        assert_eq!(numbers.lives, LIVES);
        assert_eq!(numbers.bombs, BOMBS);
        assert_eq!(numbers.power, POWER);

        // A fight, with a card up: the boss's life and the frames its attack has run, and which card.
        image.a_boss_is_fighting(1500, 42);
        image.the_spellcard_is(3);
        let state = game.state();
        assert!(state.boss_present);
        assert_eq!(state.boss_life, Some(1500));
        assert_eq!(state.boss_attack_frames, Some(42));
        assert_eq!(state.spellcard, Some(3));

        // A line of dialogue, and the pause menu.
        image.a_line_of_dialogue_is_up(2);
        assert!(game.state().in_dialogue);
        image.no_dialogue_is_up();
        image.is_paused();
        assert!(game.state().paused);
        image.is_not_paused();

        // The hit, which is orb's own count of deaths moving — this game keeps none — and where it stands:
        // open while a bomb is still taken, taken once the frames run out, and a bomb where the state left
        // the hit's before they did.
        image.the_player_has_been_hit(8);
        let state = game.state();
        assert!(state.unsettled);
        assert_eq!(state.deaths, 1, "the hit was not counted");
        assert_eq!(game.state().deaths, 1, "a held hit was counted twice");
        assert_eq!(
            unsafe { Th06c.where_a_counted_death_stands() },
            CountedDeath::Open
        );
        image.the_frames_a_bomb_is_still_taken_in(0);
        assert_eq!(
            unsafe { Th06c.where_a_counted_death_stands() },
            CountedDeath::Taken
        );
        image.the_player_is_playing();
        assert_eq!(
            unsafe { Th06c.where_a_counted_death_stands() },
            CountedDeath::Bombed
        );
        // A frame of the player playing, read: the count moves on the edge into the hit's state, and an edge
        // is a frame the state was something else on.
        assert!(!game.state().unsettled);
        image.the_player_has_been_hit(8);
        assert_eq!(game.state().deaths, 2, "a second hit was not counted");
        image.the_player_is_playing();

        // A bomb going off, the game holding everything, and a playback: each is a frame no chapter begins on,
        // and a playback is a run orb does not act on.
        image.a_bomb_is_going_off(true);
        assert!(game.state().bombing);
        image.a_bomb_is_going_off(false);
        image.everything_is_held(true);
        let state = game.state();
        assert!(
            !state.playing && state.in_run,
            "a held frame is still a frame of the run"
        );
        image.everything_is_held(false);
        image.a_replay_is_playing(false);
        let state = game.state();
        assert!(state.replay && !state.demo && !state.in_game);
        image.a_replay_is_playing(true);
        let state = game.state();
        assert!(state.demo && !state.replay && !state.in_game);
        image.no_replay_is_playing();
        assert!(game.state().in_game);
    });
}

/// The log goes beside the game, which is the one thing about it a game decides: `orb.yaml` and the launcher
/// are there, so that is where somebody looks for what a run did.
#[test]
fn the_log_of_a_launch_of_th06c_is_written_beside_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th06c-log", Work::wandering(WORK_US));
        assert_eq!(
            game.log().path(),
            Some(game.dir().join("orb.log")),
            "the log is not beside the exe orb was injected into",
        );
        // The line every run opens with. A log without it cannot say whether the run even happened.
        assert!(game.log().said("---- new run ----"));
        // Appended to rather than started over: a run worth looking at is often over by the time anybody
        // looks.
        assert!(!game.log().restarted());
    });
}

/// Before a run is started, nothing in this game reads as one — and the reads that have nothing to read come
/// back as nothing rather than as a fault.
///
/// Which is what the title looks like from orb's side: the supervisor is not in the scene a stage runs in, so
/// none of the flags a run has is set, and the numbers a run keeps read as the zeroes of a game that has not
/// built one. **th06c keeps its run's numbers in its own statics rather than behind a pointer a build
/// allocates**, so what this asserts is the zeroes and not a chase that ends — see `Th06c::read_state`, and
/// `the_run_of_th07_read_back` for the game where five chases have nothing at the end of them.
#[test]
fn nothing_is_a_run_of_th06c_before_one_is_started() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06c-title",
            Work::wandering(WORK_US),
        );
        game.puts_its_title_up();
        game.frame();

        let state = game.state();
        assert!(!state.playing, "the title reads as a stage running");
        assert!(!state.in_run);
        assert!(!state.in_game);
        assert!(!state.replay);
        assert!(!state.demo);
        assert!(!state.practice);
        assert!(!state.paused);
        assert!(!state.in_ending);
        assert_eq!(state.ending_script, None);
        assert!(!state.boss_present);
        assert_eq!(state.boss_life, None);
        assert_eq!(state.boss_attack_frames, None);
        assert_eq!(state.spellcard, None);
        assert!(!state.in_dialogue);
        assert_eq!(state.laser_count, 0);
        assert_eq!((state.lives, state.bombs, state.power), (0, 0, 0));
        assert_eq!(state.deaths, 0);
        assert_eq!(
            game.state(),
            state,
            "reading the title twice gave two answers"
        );
    });
}

/// A replay being watched is a stage played through and no run of this game's own, and the attract demo is
/// neither — which is three answers off two flags and a byte.
///
/// **The same shape th07 has**: `in_run` set and `in_game` clear for a replay, because everything that must
/// leave one alone asks the narrower flag and a pass over one needs the wider. What is this game's own is
/// the third answer: its attract demo runs in the same supervisor state with the same stage as a run, so a
/// step bounded by a run ending waited 34,200 frames of one before the demo was told apart — and
/// `THE_PLAYBACK_IS_THE_DEMO` is what tells it, set where the logo screen starts one and cleared at the
/// hand-over back to the title.
///
/// **Read the other way round once**, and the cost was a pass: `sight::in_a_run` answered
/// `!a_replay_is_playing()`, which covers the demo and the watched replay together, and with it
/// `Chapters::tracking` was false through every playback — no chapters for a judging pass to step between,
/// no `sync:` line and no reader for `Game::mid_build`. See `a_chapter_table_collected_for_th06c`.
///
/// What no reading of it changes is the thing that matters: no chapter is kept over a replay, there being
/// nobody there to answer a question about one.
#[test]
fn a_replay_of_th06c_is_no_run_of_its_own_and_nothing_is_kept_over_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06c-replay",
            Work::wandering(WORK_US),
        );
        let log = game.log();
        let image = game.image();
        image.is_playing(0, DIFFICULTY);
        image.has_lives_and_bombs(LIVES, BOMBS);
        image.the_player_is_playing();
        image.a_replay_is_playing(false);
        game.frame();

        let state = game.state();
        assert!(state.replay, "the run is not a replay being watched");
        // **A stage being played through**, which is what a pass over one has to be able to hold: the
        // chapters of it are the run's own and a judging pass steps between them.
        assert!(
            state.in_run && state.playing,
            "a replay of this game read as no stage being played at all, which is what a pass over one \
             would have nothing to hold: {state}",
        );
        // **And no run orb acts on**, which is the reading everything that must leave a replay alone asks:
        // no death to offer a retry for, no chapter written down, no file of its own.
        assert!(
            !state.in_game,
            "a replay is not a run somebody is playing, which is what decides whether orb acts on it",
        );
        assert!(
            !log.said("chapter 1"),
            "a chapter was kept over a replay nobody is playing:\n  {}",
            log.lines().join("\n  ")
        );

        // **And the attract demo is neither**, which is the one of the two nobody asked for: a step bounded
        // by a run ending waited 34,200 frames of one before this was told apart from a replay.
        image.a_replay_is_playing(true);
        game.frame();
        let state = game.state();
        assert!(state.demo, "the playback did not read as the demo");
        assert!(
            !state.in_run && !state.in_game,
            "the attract demo read as a run, which is what a `play_until` waits out: {state}",
        );

        // The playback over, the same screen is a run again: this is what says it is the playback being
        // asked and not the stage having gone.
        image.no_replay_is_playing();
        let state = game.state();
        assert!(
            state.in_run && state.in_game,
            "the stage stopped being a run when the playback ended"
        );
    });
}
