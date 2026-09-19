//! **The frame orb paces th06nc with, and what it reads of one.**
//!
//! A th06nc laid out from what has been read of `th06nc.exe`, orb attached to `Th06nc`, and orb's own
//! frame loop in place of the frame `main`'s loop calls — which is the only frame a launch of this game
//! has, the game handing its walk, its draw's own calls, its timing and the calls around them over. What the e2e tests below ask is that the loop composes th06nc's frame in orb's order,
//! and beside that one field at a time: the scene, the stage, the run's own numbers, the player's hit, the
//! fight, and the score file the mode forks.
//!
//! **What this game answers that th06c does not**, and the reason the two modules are not one test each:
//!
//! - **The stage is the front end's own absence** and not a state of it, so a run reads as `scene` of
//!   `-1` where th06c's reads as its own state — see `game::th06nc::a_stage_is_the_scene`.
//! - **Its sound's catch-up is between the update and the draw** the way th06c's is, but the count it
//!   works off is this game's own `THE_EXTRA_WALKS`, which the game's loop steps per extra walk and orb's
//!   frame clears — so the seek is asked for in this game's own samples a walk.
//! - **The picture is sized rather than a viewport set**: this game's frame calls `0x14003db50` whole,
//!   which is the present and the deadline together, and what a frame is asked for is the size.
//! - **No card and no line of dialogue is read**, so `State::spellcard` and `State::in_dialogue` answer
//!   nothing here however the game is laid out — see `Th06nc::read_state`, which declines both.
//!
//! **The mode itself is the real game's**, `the_cases_of_th06nc_the_real_game_is_tested_with`; what orb
//! *draws* over this game is `the_mode_question_over_th06nc.rs`, and the walk to its menu is
//! `the_main_menu_of_th06nc_walked.rs`.
//!
//! What these cannot say is that the addresses *are* right: a laid-out th06nc is written from the same
//! constants `Th06nc` reads. `cargo xtask real` on the copy of the install is what says.

use crate::fake::th06nc::Fake;
use crate::fake::{
    DRAW, Display, FLUSH, Launched, PRESENT, SOUND, UPDATE, Work, in_its_own_process,
};
use orb_core::game::th06nc::Th06nc;
use orb_core::game::{Game, State};

/// What the game's own frame is declared to take: about what a real run's report line shows.
const WORK_US: i64 = 700;

/// The game's own score file, as its `fopen` is handed the name.
const THE_GAMES_FILE: &str = "score.dat";

/// **The samples one walk of this game's own is worth**, which is the seek a walk the music missed turns
/// into — 48,000 a second at sixty frames, and `game::th06nc::SAMPLES_A_WALK` is where it is read.
const SAMPLES_A_WALK: i64 = 0x320;

/// The run a read-back is laid out with: Normal, the first stage, a shot part way up, a score with digits
/// in it, a generator that has drawn a few times, three hundred frames in.
const DIFFICULTY: i32 = 1;
/// **The screen a run is pointed at from**, which is the last of the front end's own that takes a press —
/// `Th06nc::run_pointed_at` answers nothing anywhere else. The number is the game's own, read off the
/// `front end: screen N` lines of a launch walking into a run.
const THE_LAST_SCREEN_BEFORE_A_RUN: i32 = 24;
/// How long the laid-out game is given to build its front end and put its logo up.
const UNTIL_THE_FRONT_END: u32 = 300;
const LIVES: i8 = 2;
const BOMBS: i8 = 3;
const POWER: u16 = 96;
const SCORE: u64 = 1_234_560;
const POINT_ITEMS: u16 = 12;
const EXTRA_LIVES: i8 = 1;
const SEED: u16 = 0x1a2b;
const RANDOMS: u32 = 7;
const AT: u32 = 300;

/// The track a catch-up is asked about, and how many walks it is behind by.
const TRACK: i32 = 3;
const BEHIND_BY: i32 = 2;

/// **The lines of a conversation a test raises**, and the number that says none is up: the dialogue's own
/// start writes the line it was asked for and its two ends write `-1` — see
/// `a_line_of_dialogue_of_th06nc_is_read_off_the_line_its_object_is_on`.
const NO_LINE: i32 = -1;
const THE_FIRST_LINE: i32 = 0;
const A_LATER_LINE: i32 = 3;

#[test]
fn orb_paces_a_launch_of_th06nc_with_no_run_in_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-paced",
            Work::wandering(WORK_US),
        );
        let log = game.log();

        // The frame is orb's own, in orb's order: the update first, the draw, and the frame handed to the
        // display through this game's own swapchain.
        //
        // **And nothing of the sound on a frame like this one**, which is a run at its own speed: this
        // game's catch-up turns walks the music has not heard into a seek, so a frame that walked the calc
        // chain once asks the game for nothing at all.
        //
        // **Not the launch's first frame, which hands the frame over twice** — the game's own timing call
        // presents, and the search that takes the swapchain runs in the present after it, so the loop's
        // own present has nothing to stand in for until the second frame. The real game's first frame does
        // the same.
        game.frame();
        game.forget_asked();
        game.frame();
        // **The draw's sprites flushed as the job is done and before the picture is finished**, which is
        // what this game's own walk of the draw chain does after every job — see th06c's twin of this
        // assertion, the same batch in the same engine.
        assert_eq!(game.asked(), vec![UPDATE, DRAW, FLUSH, PRESENT]);

        // And the sound's own place in that order, on a frame with music behind it: between the update and
        // the draw, as this game's own loop has it, with the count cleared and the track seeked by the
        // walks' worth of samples the music missed.
        game.image().plays_the_track(TRACK);
        game.image().the_extra_walks_are(BEHIND_BY);
        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), vec![UPDATE, SOUND, DRAW, FLUSH, PRESENT]);
        assert_eq!(
            game.image().the_extra_walks(),
            0,
            "the catch-up did not clear the walks it caught up on",
        );
        assert_eq!(
            game.seeks_asked_for(),
            vec![(SAMPLES_A_WALK * i64::from(BEHIND_BY), TRACK)],
            "the music was not seeked by the two walks it was behind",
        );

        // And the call that is this game's own frame's rather than orb's: the picture finished once a
        // frame, through the function the game handed over. Counted from where it stands rather than from
        // nought, so that a frame run above is one this does not have to know about.
        let pictures = game.pictures_finished();
        let frames = 30;
        game.frames(frames);
        assert_eq!(
            game.pictures_finished() - pictures,
            frames,
            "the picture was not finished once a frame"
        );

        // A game orb can rewind starts in the mode orb is for — and the one read it makes of its score
        // file at startup, the file every screen of it parses afterwards, landed in the game's own all the
        // same: that read is the one the menu's unlocks come out of, and nobody has been asked which mode
        // yet.
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
fn every_number_a_stage_of_th06nc_is_played_with_reads_back_as_the_game_put_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-read-back",
            Work::wandering(WORK_US),
        );
        // **Counted from nought, which is what `Fake::in_a_run_in_the_stage` is handed**: the stage's own
        // build counts the number in the manager's field on — `0x14003c7c0`, on both of its paths — so
        // the first stage of a run is nought here, 1 in that field once the build has run, and nought
        // again in a `State`. See `Th06nc::read_state` and `Th06nc::start_stage`.
        let in_the_first_stage = 0;
        game.in_a_run_in_the_stage(in_the_first_stage);
        let image = game.image();
        image.the_difficulty_is(DIFFICULTY);
        image.has_lives_and_bombs(LIVES, BOMBS);
        image.shot_is_at_power(POWER);
        image.the_run_has_scored(SCORE, POINT_ITEMS);
        image.the_score_has_paid_for(EXTRA_LIVES);
        image.the_generator_is(SEED, RANDOMS);
        // **And the copy the stage's own build takes of it**, which is the seed a resume of this game
        // carries rather than the generator's own word — see `Th06nc::seed`.
        image.the_stage_began_with_the_seed(SEED);
        image.has_been_in_the_stage_for(AT);
        image.the_script_has_run_for(AT as i32);
        image.the_player_is_playing();

        let state = game.state();
        assert!(state.playing && state.in_run && state.in_game);
        assert!(!state.replay && !state.demo && !state.practice && !state.paused);
        assert_eq!(
            state.scene, -1,
            "a stage of this game is the front end's absence and not a state of it"
        );
        assert_eq!(state.stage, 0, "counted from zero above `Game`");
        assert_eq!(state.difficulty, DIFFICULTY);
        assert_eq!(state.stage_frames, AT);
        assert_eq!(
            state.script_frames, AT as i32,
            "the clock the stage's own script runs on"
        );
        assert_eq!(state.random_seed, SEED, "the generator itself");
        assert_eq!(state.lives, LIVES);
        assert_eq!(state.bombs, BOMBS);
        assert_eq!(state.power, POWER);
        assert_eq!(state.deaths, 0);
        assert!(!state.boss_present);
        assert_eq!(state.boss_life, None);
        // And the two this stage has neither of: no card is up and no line of a conversation stands — see
        // `the_cards_a_run_of_th06nc_counted` for the first and
        // [`a_line_of_dialogue_of_th06nc_is_read_off_the_line_its_object_is_on`] for the second.
        assert_eq!(state.spellcard, None);
        assert!(!state.in_dialogue);
        assert_eq!(game.state(), state, "reading it twice gave two answers");

        // The run's own numbers as this game's resume carries them, read where its stage's build leaves
        // them.
        let numbers = unsafe { Th06nc.run_state() };
        let numbers = orb_core::game::th06nc::resume::decode_state(&numbers)
            .expect("the run's numbers are this game's own");
        assert_eq!(numbers.score, SCORE);
        assert_eq!(numbers.point_items, POINT_ITEMS);
        assert_eq!(numbers.lives, LIVES);
        assert_eq!(numbers.bombs, BOMBS);
        assert_eq!(numbers.power, POWER);
        assert_eq!(numbers.extra_lives, EXTRA_LIVES);
        assert_eq!(numbers.seed, SEED);

        // A fight: the boss's life, off the slot the fight's own flag says it is.
        image.a_boss_is_fighting(1500, 42);
        let state = game.state();
        assert!(state.boss_present);
        assert_eq!(state.boss_life, Some(1500));
        // **And the frames its attack has run**, off the dword the panel's countdown is worked out from.
        assert_eq!(state.boss_attack_frames, Some(42));
        image.no_boss_is_fighting();

        // A bomb in the air, which is a frame no chapter begins on — the byte the bomb's own start
        // writes and each of the game's four bomb ends clears.
        image.a_bomb_is_going_off(true);
        assert!(game.state().bombing);
        image.a_bomb_is_going_off(false);
        assert!(!game.state().bombing);

        // The pause menu, which is one of the two menus a run stops for.
        image.is_paused();
        assert!(game.state().paused);
        image.is_not_paused();
        assert!(!game.state().paused);

        // The hit, which is orb's own count of deaths moving — this game keeps none of its own — counted
        // on the edge into the hit's state and on no other.
        image.the_player_has_been_hit();
        assert_eq!(game.state().deaths, 1, "the hit was not counted");
        assert_eq!(game.state().deaths, 1, "a held hit was counted twice");
        // A frame of the player playing, read: the count moves on the edge into the hit's state, and an
        // edge is a frame the state was something else on.
        image.the_player_is_playing();
        assert!(game.state().playing, "the stage stopped running");
        image.the_player_has_been_hit();
        assert_eq!(game.state().deaths, 2, "a second hit was not counted");
        image.the_player_is_playing();

        // The game holding everything: a frame no chapter begins on, and still a frame of the run.
        image.everything_is_held(true);
        let held = game.state();
        assert!(
            !held.playing && held.in_run,
            "a held frame is not a frame of the run: {held}"
        );
        image.everything_is_held(false);
    });
}

/// **A line of a conversation is up where the dialogue object says so**, and negative for none.
///
/// **What it is for**: a chapter must not begin inside a line — the game holds the run there and a frame
/// held is no boundary — and a `play_until: { dialogue: true }` is how a recipe stops at one. `Th06nc`
/// answers it twice over the same reading, `State::in_dialogue` and
/// `Game::a_line_of_dialogue_is_shown`, and the second is what the frame's own fast-forward asks.
///
/// **Read off what the game itself asks**, since this build's fast-forward gate is no handle on it: the
/// line at `+0x36b0` of the object behind `0x140abae28`, which the dialogue's own start writes at
/// `0x1400406c4` and its two ends put back to `-1` — and which the bomb press is refused on at
/// `0x14006a0d6` while it is nought or more. th06c's own handle is a panel field its fast-forward gates
/// on, and this build's gate reads four other things; see `Th06nc::the_line_of_dialogue`.
#[test]
fn a_line_of_dialogue_of_th06nc_is_read_off_the_line_its_object_is_on() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-dialogue",
            Work::wandering(WORK_US),
        );
        game.in_a_run_in_the_stage(0);
        let image = game.image();
        image.the_player_is_playing();

        // No line to start with, which is what the object holds from the frame a stage's assets are
        // loaded.
        assert_eq!(image.the_line_of_dialogue_up(), NO_LINE);
        assert!(
            !game.state().in_dialogue,
            "a line read as up with the object saying none: {}",
            game.state(),
        );
        assert!(!unsafe { Th06nc.a_line_of_dialogue_is_shown() });

        // The first line of the conversation, which is the number the dialogue's own start writes.
        image.a_line_of_dialogue_is_up(THE_FIRST_LINE);
        assert!(
            game.state().in_dialogue,
            "the line up read as no conversation: {}",
            game.state(),
        );
        assert!(unsafe { Th06nc.a_line_of_dialogue_is_shown() });

        // And a line further into it, which is the same answer: what is read is whether the number is
        // negative and not which line it is.
        image.a_line_of_dialogue_is_up(A_LATER_LINE);
        assert!(game.state().in_dialogue);
        assert!(unsafe { Th06nc.a_line_of_dialogue_is_shown() });

        // And the conversation over, which is the `-1` its ends write.
        image.no_line_of_dialogue_is_up();
        assert!(
            !game.state().in_dialogue,
            "the conversation ended and a line still read as up: {}",
            game.state(),
        );
        assert!(!unsafe { Th06nc.a_line_of_dialogue_is_shown() });
    });
}

/// **And a card practised on its own reads as a practice too, which is what keeps orb out of it.**
///
/// **The main menu's item 3 is this build's Spell Practice**, settled by a picture: that item walks
/// screens 27 and 28, and 28 reads `Spell Practice` over a list of the cards stage by stage. A run of it
/// is played under `0x14053d405` — the byte `0x14004ee53` sets for a scene out of the table at
/// `0x140c6e3e0` and `0x14004f35c` clears as an ordinary run starts — and the card's own start steps a
/// count of that mode's own instead of the attempts while it is set.
///
/// **The two modes of this game are two runs, and the slot says which.** 標準モード and チャレンジモード
/// are screen 14's own two items, and they start a run with different lives — `0x14004f373` writes two
/// where the mode byte is set — so a chapter of one put back into the other is a run given the wrong
/// numbers to begin with. `Th06nc::run_slot` therefore names a Challenge run's slot with a word in front
/// of the name a Standard one's has, which leaves every Standard chapter written so far where it is.
///
/// **And the byte is read at both moments a slot is asked for**: `Game::run_start` reads the run's own,
/// `0x14053d414`, which screen 25 writes out of the configuration; `Game::run_pointed_at` reads the
/// configuration's, `[0x140c6dfdb]`, which screen 14 wrote on the way there. See
/// `Th06nc::the_mode_a_run_is_in` and `Th06nc::the_mode_the_next_run_is_in`.
#[test]
fn the_two_modes_of_th06nc_are_two_slots() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-the-two-modes",
            Work::wandering(WORK_US),
        );
        game.in_a_run_in_the_stage(0);
        let image = game.image();
        image.the_player_is_playing();
        // Normal, so that the name the two are held to is written out rather than the laid-out game's
        // own default.
        image.the_difficulty_is(DIFFICULTY);

        image.the_run_is_in_challenge_mode(false);
        let standard = unsafe { Th06nc.run_start() };
        let standards = Th06nc
            .run_slot(&standard)
            .expect("a run of the whole game to have a slot");

        image.the_run_is_in_challenge_mode(true);
        let challenge = unsafe { Th06nc.run_start() };
        let challenges = Th06nc
            .run_slot(&challenge)
            .expect("a run of the whole game to have a slot");

        assert_ne!(
            standards, challenges,
            "the two modes of this game share one slot, so a chapter of either is offered to both",
        );
        assert_eq!(
            standards, "normal-reimu-a",
            "the mode every chapter written so far was in stopped being the bare name",
        );
        assert_eq!(
            challenges, "challenge-normal-reimu-a",
            "a Challenge run's slot is not the Standard name with the mode in front of it",
        );

        // And the four numbers beside it are the same run, which is what says the mode is what parted
        // the two names.
        assert_eq!(
            (
                standard.difficulty,
                standard.character,
                standard.shot_type,
                standard.practice
            ),
            (
                challenge.difficulty,
                challenge.character,
                challenge.shot_type,
                challenge.practice
            ),
        );
    });
}

/// **And the run *pointed at* is named the same way, off the configuration's own byte** — which is the
/// moment orb asks about picking a chapter up, so a Challenge run pointed at has to be offered the
/// Challenge slot and not the Standard one.
///
/// **A launch of its own and not the test above's**, because `Game::run_pointed_at` wants the front end:
/// it reads the screen out of the object `Th06nc::the_front_end` finds by walking the calc chain, and a
/// game already in a run has taken that job out of it. Asked after `in_a_run_in_the_stage`, this answered
/// `None` — which passed alone and failed in the loaded suite.
#[test]
fn the_run_of_th06nc_pointed_at_is_named_by_the_mode_last_chosen() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-pointing-at-a-mode",
            Work::wandering(WORK_US),
        );
        // The front end up and in the chain, which is what the walk needs, and then the screen a run is
        // pointed at from.
        game.frames_until("the front end", UNTIL_THE_FRONT_END, || {
            game.image().the_screen_the_front_end_is_on() > 0
        });
        let image = game.image();
        image.the_front_end_is_on(THE_LAST_SCREEN_BEFORE_A_RUN, 0);
        image.the_difficulty_is(DIFFICULTY);

        image.the_mode_last_chosen_is_challenge(false);
        let pointed =
            unsafe { Th06nc.run_pointed_at() }.expect("the screen before a run to point at a run");
        assert_eq!(Th06nc.run_slot(&pointed).as_deref(), Some("normal-reimu-a"));

        image.the_mode_last_chosen_is_challenge(true);
        let pointed =
            unsafe { Th06nc.run_pointed_at() }.expect("the screen before a run to point at a run");
        assert_eq!(
            Th06nc.run_slot(&pointed).as_deref(),
            Some("challenge-normal-reimu-a"),
            "the run pointed at was named off the run's own byte and not the configuration's",
        );
    });
}

/// **So `Th06nc` answers `practice` off both bytes**, item 2's and item 3's, and `Game::run_slot` names
/// neither: a card practised on its own is no run whose chapters a run of the whole game could be put back
/// into, and a slot shared with one would hand a full run the practice's own file. See
/// `Th06nc::a_run_is_practice`.
#[test]
fn a_scene_of_th06nc_reads_as_a_practice_and_is_given_no_slot() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-a-scene",
            Work::wandering(WORK_US),
        );
        game.in_a_run_in_the_stage(0);
        let image = game.image();
        image.the_player_is_playing();

        // A run of the whole game to start with: neither byte is set and orb has a slot to keep it under.
        assert!(
            !game.state().practice,
            "a run of the whole game read as a practice: {}",
            game.state(),
        );
        let run = unsafe { Th06nc.run_start() };
        assert!(
            Th06nc.run_slot(&run).is_some(),
            "a run of the whole game was given no slot",
        );

        // And the scene's own byte, which the Spell Practice's start writes.
        image.a_scene_is_being_played(true);
        assert!(
            game.state().practice,
            "a card practised on its own did not read as a practice: {}",
            game.state(),
        );
        let scene = unsafe { Th06nc.run_start() };
        assert_eq!(
            Th06nc.run_slot(&scene),
            None,
            "a card practised on its own was given a slot to keep chapters under",
        );

        // And the byte cleared, which is what an ordinary run's own start does with it.
        image.a_scene_is_being_played(false);
        assert!(!game.state().practice);
    });
}

/// The log goes beside the game, which is the one thing about it a game decides: `orb.yaml` and the
/// launcher are there, so that is where somebody looks for what a run did.
#[test]
fn the_log_of_a_launch_of_th06nc_is_written_beside_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th06nc-log", Work::wandering(WORK_US));
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

/// Before a run is started, nothing in this game reads as one — and the reads that have nothing to read
/// come back as nothing rather than as a fault.
///
/// Which is what the menu looks like from orb's side: the player's job is not in the calc chain, so none
/// of the flags a run has is set, and the numbers a run keeps read as the zeroes of a game that has not
/// built one. **th06nc keeps its run's numbers in one static the stage's build writes** — see
/// `game::th06nc::GAME_MANAGER` — so what this asserts is those zeroes and not a chase that ends.
#[test]
fn nothing_is_a_run_of_th06nc_before_one_is_started() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-menu",
            Work::wandering(WORK_US),
        );
        game.puts_its_menu_up();
        game.frame();

        let state = game.state();
        assert!(!state.playing, "the menu reads as a stage running");
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
            "reading the menu twice gave two answers"
        );
    });
}

/// A playback of this game is a stage being played through and no run of its own, and the same screen is a
/// run again once the playback is over — which is two answers off one flag.
///
/// **The same shape th06c and th07 have, less the third answer neither of them could do without**: for
/// those two the split is `in_run` set and `in_game` clear for a replay, with the *attract demo* a third
/// state that is neither — and th06c paid for reading it any other way with a step that waited 34,200
/// frames of a demo for a run that was over. **This build plays no demo at all**, so there is no third
/// state to tell apart: both callers of the replay's own start `0x14004ede0` are the front end's replay
/// list, and a launch of `the-front-end-after-a-run.yaml` left a run, watched the front end walk `16, 17,
/// 18` and sat on 18 for 21,600 frames with nothing pressed and no playback of any kind — see
/// `Th06nc::in_a_run`, where both readings are, and `th06nc/asm.md`'s table of the screens.
///
/// **And the wider reading is what a judging pass needs**: `Chapters::tracking` is
/// `in_game || (during_replay && playing && !demo)` and `runtime::step` asks for `state.in_run` as well,
/// so a reading that called a playback no stage at all left a pass with nothing to hold — see
/// `stepping_inside_a_stage_of_th06nc`, which is the module that waited on this.
///
/// What no reading of it changes is the thing that matters: no chapter is kept over a replay, there being
/// nobody there to answer a question about one.
#[test]
fn a_replay_of_th06nc_is_no_run_of_its_own_and_nothing_is_kept_over_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th06nc-replay",
            Work::wandering(WORK_US),
        );
        let log = game.log();
        // Counted from nought, as `Fake::in_a_run_in_the_stage` is — see the read-back test above.
        game.watches_a_replay_of_the_stage(0);
        let image = game.image();
        image.has_lives_and_bombs(LIVES, BOMBS);
        image.the_player_is_playing();
        game.frame();

        let state = game.state();
        assert!(state.replay, "the run is not a replay being played back");
        // **A stage being played through**, which is what a pass over one has to be able to hold: the
        // chapters of it are the run's own and a judging pass steps between them.
        assert!(
            state.in_run && state.playing,
            "a playback of this game read as no stage being played at all, which is a pass over one with \
             nothing to hold: {state}",
        );
        // **And no run orb acts on**, which is the reading everything that must leave a playback alone
        // asks: no death to offer a retry for, no chapter written down, no file of its own.
        assert!(
            !state.in_game,
            "a playback is not a run somebody is playing, which is what decides whether orb acts on it: \
             {state}",
        );
        assert!(
            !log.said("chapter 1"),
            "a chapter was kept over a replay nobody is playing:\n  {}",
            log.lines().join("\n  ")
        );

        // The playback over, the same screen is a run again: this is what says it is the playback being
        // asked and not the stage having gone.
        image.a_replay_is_playing(false);
        let state: State = game.state();
        assert!(
            state.in_run && state.in_game,
            "the stage stopped being a run when the playback ended: {state}"
        );
    });
}
