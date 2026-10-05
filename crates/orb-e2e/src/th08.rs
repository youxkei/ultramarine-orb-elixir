//! **The frame orb paces th08 with, and what it reads of one.**
//!
//! A th08 laid out from what has been read of `th08.exe`, orb attached to `Th08`, and orb's own frame loop
//! in place of the game's own frame — which is what a launch there is. What the e2e tests below ask is
//! that the loop composes th08's frame: the update before the draw, the queue of quads emptied before the
//! drawing fills it and drawn before the scene ends, the fog a stage left on put out. Beside that, one
//! field at a time: the scene, the front end's screens, the stage and the run's own numbers, the
//! letterbox.
//!
//! **No mode over any of them, which is this file's own shape and not the game's**: `Th08::rewinds`
//! answers true, and the mode over a run of this game is
//! [`a_pointdevice_run_of_th08`](crate::a_pointdevice_run_of_th08)'s to hold together end to end. What
//! each test here asks is one field or one call at a time, and
//! `the_cases_of_th08_the_real_game_is_tested_with.rs` plays every case the real game is tested with
//! against this same laid-out game.
//!
//! **The queue is what makes these more than a smoke test.** orb's first loop in th07's frame took that
//! game down on its first frame, writing a quad through a pointer the frame had not reset, and every
//! address it used was right. So this laid-out game's draw chain queues a quad the way a real one does,
//! through the field the game's own frame resets, and a loop that leaves that reset out faults here as it
//! faulted there.
//!
//! What these cannot say is that the addresses *are* right: a laid-out th08 is written from the same
//! constants `Th08` reads, so a wrong one is wrong on both sides at once. Only the real image says, which
//! is `cargo xtask real` and the cases it runs.

use crate::fake::th08::{
    BOMBS, DIFFICULTY_NORMAL, FIRST_STAGE, Fake, LIVES, PLAYER_STARTS_AT, POINT_ITEMS, POWER,
    SCORE, THE_FIGHTS_TRACK,
};
use crate::fake::{
    DRAW, Display, Launched, PRESENT, Presented, SOUND, UPDATE, Work, in_its_own_process,
};
use orb_config::LogLevel;
use orb_config::Screen as Shape;
use orb_core::frame;
use orb_core::game::th08::Th08;
use orb_core::game::th08::image::{Scene, Screen};
use orb_core::game::{Game, RunStart};
use orb_core::window::letterbox;

/// What the game's own frame is declared to take here: about what a real run's report line shows for the
/// drawing, so that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long the run is: a few seconds of it, which is past the frames orb spends trying to build an
/// overlay and well into the frames where nothing is happening.
const FRAMES: u32 = 5 * frame::LOGIC_HZ;

#[test]
fn orb_paces_a_launch_of_th08_with_no_run_in_it() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th08-paced", Work::wandering(WORK_US));

        // The frame is orb's own, in orb's order: the update first — which is the frame of input lag
        // removed — the sounds where the game's own frame hands them over, then the drawing, then the
        // frame handed to the display.
        //
        // **Which for this game is the same order its own frame has and not the reverse**, th08's
        // `GameWindow::Render` calling its calc chain at 0x441f4d and its draw chain at 0x441fec. So what
        // this asserts about th08 is that the loop kept that order rather than that it turned one round.
        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), vec![UPDATE, SOUND, DRAW, PRESENT]);

        game.frames(FRAMES);
        let log = game.log().lines().join("\n");

        // orb got in, and got in through both of the patches `Th08::hooks` asks for: an update hook and a
        // draw hook reached is what the order above is evidence of.
        assert!(
            log.contains("attached to a game laid out in this process"),
            "orb did not attach:\n  {log}"
        );

        // And in orb's own mode to start with, with the question left standing for the menu: that is what
        // `Th08::rewinds` answering true puts a launch of this game in, and the title is where it is asked
        // — see `a_pointdevice_run_of_th08`, which answers it.
        assert!(
            log.contains("mode: pointdevice to start with; the menu asks which"),
            "orb did not start this launch in its own mode:\n  {log}"
        );
        // **And nothing of a chapter over a launch with no run in it**, which is the same list as before
        // the mode: a question standing at a title is not a chapter taken, a death or a run picked up.
        for absent in ["chapter ", "retry", "resume: stage", "died in"] {
            assert!(
                !log.contains(absent),
                "orb did {absent:?} to a launch with no run in it:\n  {log}"
            );
        }

        // And it did not take the game down, which is the one thing an injected DLL must not do to
        // somebody's play session.
        for fatal in ["panic:", "crash:"] {
            assert!(
                !log.contains(fatal),
                "orb wrote a {fatal:?} line over {FRAMES} frames:\n  {log}"
            );
        }

        // Every frame handed over by orb's loop, which is what paces it: one present a frame, and the
        // `frame:` line is orb saying what cadence it settled on.
        assert_eq!(game.handovers_us().len(), FRAMES as usize + 1);
    });
}

#[test]
fn the_queue_of_quads_of_th08_is_emptied_before_the_drawing_and_drawn_before_the_scene_ends() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-queue",
            Work::wandering(WORK_US),
        );

        game.frames(FRAMES);

        // Where the next quad goes is one quad past the start of the buffer, however many frames have
        // run: emptied at the top of every frame's drawing, filled by the draw chain, and drawn before
        // the scene ended. A loop that emptied it and never drew it would leave the pointer FRAMES quads
        // along; one that never emptied it walks off the end of the buffer.
        let image = game.image();
        assert_eq!(
            image.queue_writes_at(),
            image.queue_buffer() + image.queue_bytes_per_quad(),
            "the queue of quads is not one quad along after a frame that queued one and drew it",
        );
        assert_eq!(
            image.queued(),
            0,
            "quads were left in the queue at the end of a frame",
        );
        // And the game's own count of how many times it drew that queue, which is the frames: one drawing
        // a frame, and orb's loop skips none.
        assert_eq!(image.queue_draws(), FRAMES);
    });
}

/// **The fog is put out on every frame orb draws and not only on the frame after a stage turned it on.**
///
/// The call at 0x448229 does nothing where `[supervisor+0x350]` already says the fog is out, and this
/// game's own frame writes `0xff` into that field at 0x441fd3 before making the call — so a loop that left
/// the write out would tell the device once and never again, and everything orb draws after that would be
/// drawn under whatever fog the last stage set.
#[test]
fn the_fog_of_th08_is_taken_off_to_the_device_every_frame() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-fog",
            Work::wandering(WORK_US),
        );

        game.frames(FRAMES);

        // One telling a frame, and the flag left out at the end of every one of them.
        assert_eq!(
            game.times_the_fog_was_put_out(),
            FRAMES,
            "the device was not told the fog is out once a frame",
        );
        assert_eq!(
            game.image().the_fog_flag(),
            0,
            "the fog flag was left on at the end of a frame",
        );
    });
}

/// A client wider than the game's own ratio, so that the rectangle a frame reaches the device in is plainly
/// not the client: th08 renders 640x480, and what 4:3 gets in 16:9 is bars down the sides.
const WIDER_THAN_THE_GAME: Shape = Shape::Window {
    width: 1280,
    height: 720,
};

#[test]
fn the_frames_of_th08_reach_the_device_inside_a_letterbox_of_the_games_own_shape() {
    in_its_own_process(|| {
        let game = Fake::attach_before_its_device(
            Display::agreed(120),
            "th08-the-letterbox",
            Work::wandering(WORK_US),
            |config| config.screen = WIDER_THAN_THE_GAME,
        );
        // The window first and the device after it, which is the order a launch has: orb redirects the
        // `Present` slot from the game's own device setup at 0x4424c0, and what it works the rectangle out
        // from is the client area of a window that exists.
        game.creates_its_window();
        game.finds_its_device();

        game.forget_presents();
        game.frame();

        let presents = game.presented();
        assert_eq!(presents.len(), 1, "a frame presented other than once");
        let letterbox = unsafe { letterbox() }.expect("a launch whose window orb has measured");
        assert_eq!(
            presents[0].destination,
            Some(letterbox),
            "the frame was presented over the whole client rather than into the game's own shape",
        );
        // And the whole back buffer into it. This game's own present asks for none of its surface in
        // particular — four nulls at 0x44207f — so what orb has to keep is that.
        assert_eq!(presents[0].source, None);
    });
}

/// And a driver that will not stretch: the game's own call again, said once, and the run still presented.
#[test]
fn a_driver_that_refuses_to_stretch_is_fallen_back_from_over_th08() {
    in_its_own_process(|| {
        let game = Fake::attach_before_its_device(
            Display::agreed(120),
            "th08-the-letterbox-refused",
            Work::wandering(WORK_US),
            |config| {
                config.screen = WIDER_THAN_THE_GAME;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.creates_its_window();
        game.finds_its_device();
        let letterbox = unsafe { letterbox() }.expect("a launch whose window orb has measured");
        game.refuses_to_stretch_on_a_present();

        game.forget_presents();
        game.frame();
        // Two presents for the one frame: the letterbox, which the driver refused, and the game's own ask
        // again — its whole back buffer over the whole client, which is what every launch before orb had.
        assert_eq!(
            game.presented(),
            vec![
                Presented {
                    source: None,
                    destination: Some(letterbox),
                },
                Presented {
                    source: None,
                    destination: None,
                },
            ],
            "the refusal was not fallen back from with the ask the game made",
        );
        assert!(
            log.said("screen: Present into a letterbox failed"),
            "orb did not say the driver refused:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **The scene orb reads is the one this game's own supervisor is in**, which is the pair
/// `Th08::read_state` reads out of `[0x17ce758+0x28]` and `+0x2c`: the scene now and the scene wanted.
#[test]
fn the_scene_of_th08_orb_reads_is_the_one_the_games_own_supervisor_is_in() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-scene",
            Work::wandering(WORK_US),
        );
        game.its_title_menu_is_up();
        game.frame();

        let front = game.state();
        assert_eq!(
            (front.scene, front.wanted),
            (Scene::FrontEnd as i32, Scene::FrontEnd as i32),
            "the scene orb reads at the title menu is not the front end's",
        );
        assert!(
            !front.in_run && !front.playing && !front.in_game,
            "orb reads a run at the title menu: {front}",
        );
        // And which screen of the front end is up, which is what a recipe of this game presses at.
        assert_eq!(
            unsafe { Th08.front_end_screen() },
            Some(Screen::Title as i32),
            "orb does not read the title menu's own screen number",
        );

        // And a run placed under it read as a run: the same two fields, the scene a run is played in.
        game.is_in_a_run();
        game.frame();
        let run = game.state();
        assert_eq!(
            (run.scene, run.wanted),
            (Scene::Run as i32, Scene::Run as i32),
            "the scene orb reads in a run is not the run's",
        );
        assert!(
            run.in_run && run.playing && run.in_game,
            "orb does not read a run in the scene a run is played in: {run}",
        );
    });
}

/// **The stage, the difficulty and the run's own numbers are read out of the game**, which for this game
/// means read *through a pointer its build allocates*: `[0x164d1b0]`, whose freed value one launch read
/// `lives=-128 bombs=-128` through.
#[test]
fn the_stage_the_difficulty_and_the_numbers_of_a_run_of_th08_are_read_out_of_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-numbers",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();

        let state = game.state();
        assert_eq!(
            state.stage, FIRST_STAGE,
            "the stage orb reads is not the run's"
        );
        assert_eq!(
            state.difficulty, DIFFICULTY_NORMAL,
            "the difficulty orb reads is not the one the run was started on",
        );
        assert_eq!(
            (state.lives, state.bombs, state.power),
            (LIVES as i8, BOMBS as i8, POWER as u16),
            "the numbers orb reads are not the ones behind the run's own pointer",
        );
        assert_eq!(
            state.deaths, 0,
            "orb reads a death in a run that has had none"
        );
        // **And the player where a stage of this game starts one**, which is the middle of the play area
        // across and 64 up from its bottom: what a bot is told is where it is, and a stage that put it
        // somewhere else would be a bot steering from the wrong place.
        assert_eq!(
            game.image().where_the_player_is(),
            PLAYER_STARTS_AT,
            "the stage did not start its player where this game's build puts one",
        );

        // **And nothing at all where that pointer has been freed**, which is the state a launch read
        // through: the whole numbers answered as nothing rather than as whatever the allocator left.
        game.image().its_runs_numbers_are_freed();
        let freed = game.state();
        assert_eq!(
            (freed.lives, freed.bombs, freed.power, freed.deaths),
            (0, 0, 0, 0),
            "orb read a run's numbers through a pointer the build had freed: {freed}",
        );
    });
}

/// **The set a stage of a run of th08 would be started again from reads back out of the game's own
/// memory, and writes back into it** — which is the round trip `resume`'s own unit tests cannot make, a
/// `Numbers` held against itself saying nothing about where each field was read.
///
/// Sixteen fields through two addresses apiece: the run's own pointer for the score, the power and the
/// rest, and four statics for the ones the record carries that the run's block does not. What this catches
/// is a field read at one address and written at another — which is silent in either direction on its own.
#[test]
fn the_numbers_a_stage_of_th08_is_started_again_from_read_back_out_of_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-resume",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();

        let run = &Th08;
        let taken = unsafe { run.run_state() };
        assert!(
            run.say_run_state(&taken)
                .contains(&format!("score: {SCORE}")),
            "the score the run was laid out with is not in the set that was read: {}",
            run.say_run_state(&taken),
        );

        // Every number of it moved, so that a field put back at the wrong address cannot come out right:
        // the ones behind the run's own pointer, and the stage's seed, which is one of the four statics.
        game.image().the_run_has_scored(SCORE + 7, POINT_ITEMS + 3);
        game.image().has_lives_and_bombs(0.0, 0.0);
        game.image().shot_is_at_power(0.0);
        game.image().has_died(4.0);
        game.image()
            .the_stage_began_with_the_seed(A_SEED_OF_ITS_OWN);
        assert_ne!(
            unsafe { run.run_state() },
            taken,
            "the numbers read back are the same after every one of them was moved",
        );

        // **And the numbers go back without the seed**, which is the whole of what `set_run_state`
        // writes: the seed is put back inside the build, through `seed_the_stage` at the seam
        // `STAGE_SEEDED` is, and the numbers at the build's own end through `STAGE_BEGUN`. So the two
        // halves of a resume of this game land at two moments, and a `set_run_state` that had put the
        // seed back would be one that put it back before the build drew from it.
        unsafe { run.set_run_state(&taken) };
        assert_eq!(
            game.image().the_seed_the_stage_began_with(),
            A_SEED_OF_ITS_OWN,
            "the numbers put back took the stage's seed with them",
        );
        let back = game.state();
        assert_eq!(
            (back.lives, back.bombs, back.power, back.deaths),
            (LIVES as i8, BOMBS as i8, POWER as u16, 0),
            "the run's own numbers are not the ones the set was taken with: {back}",
        );

        // And with the seed put back at its own seam, the whole set reads back as the set that was taken.
        unsafe { run.seed_the_stage(&taken) };
        assert_eq!(
            unsafe { run.run_state() },
            taken,
            "the set put back does not read back as the set that was taken",
        );
    });
}

/// A seed the numbers taken above were not read with, so that a `set_run_state` which wrote one cannot
/// write the same one.
const A_SEED_OF_ITS_OWN: u16 = 0x1234;

/// How far into its thirty frames a hit is put before orb is asked to make the player invulnerable: past
/// halfway, so that a countdown left where it was would reach the life going in under a second.
const THE_HIT_HAS_COUNTED: i32 = 20;

/// **The player put where nothing can hit them, which for this game is the hit's own window with its
/// countdown put back.**
///
/// All four of th08's hit tests decline while `[player+0x0]` is not nothing and there is no window beside
/// that one — so what makes the player unhittable is the state a hit leaves, and what keeps the life is
/// the count going back to nothing every update. A write of the state alone reaches `0x1e` and the life
/// goes, which is what this holds `Th08::make_invulnerable` to.
#[test]
fn the_player_of_th08_is_made_invulnerable_without_the_life_going() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-invulnerable",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();

        // A player that can be hit, which is the state a stage starts one in.
        assert_eq!(
            game.image().what_the_player_is_doing(),
            0,
            "a stage did not start its player in the state a hit can land on",
        );
        assert!(
            !game.state().unsettled,
            "orb reads a player that has been hit in a stage that just started one",
        );

        // A hit two thirds of the way through its window, and then orb asked for invulnerability: the
        // state stands and the count goes back to nothing, which is the frame the life never arrives on.
        game.image().the_player_has_been_hit();
        game.image().the_hit_has_counted(THE_HIT_HAS_COUNTED);
        unsafe { Th08.make_invulnerable() };

        assert_ne!(
            game.image().what_the_player_is_doing(),
            0,
            "the player is in the one state every hit test of this game lands on",
        );
        assert_eq!(
            game.image().the_frames_the_hit_has_counted(),
            0,
            "the countdown the life goes at the end of was left where it was",
        );
        assert!(
            game.state().unsettled,
            "orb does not read the player it just made invulnerable as unsettled",
        );
    });
}

/// **And a card held where it stands is let go of with that same ask**, which is the handler's own first
/// act and not its last: a card's update does nothing at all while `[gm+0x2c]` stands, and inside a stage
/// the hit handler is the only thing that clears it, a stage's build at 0x43b7fe being the other — so a
/// run made unhittable without it is a fight that can
/// stop for good, and one did. A `--clear` launch held the last stage's Last Spell for eleven minutes of
/// game time with `script=4264`, `boss_life=20000` and not a bullet on the screen.
#[test]
fn a_card_of_th08_held_where_it_stands_is_let_go_of_by_the_invulnerable_ask() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-a-card-held",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();
        game.image().a_card_is_held();
        assert_ne!(
            game.image().the_card_is_held(),
            0,
            "the laid-out game did not hold its card where the write says it does",
        );

        unsafe { Th08.make_invulnerable() };

        assert_eq!(
            game.image().the_card_is_held(),
            0,
            "the ask left the byte a card's own update stops on standing",
        );
    });
}

/// **And the held card holds the player with it**: 0x44c390 returns at once while `[gm+0x2c]` stands — no move,
/// no step of its state, no countdown — so a direction held under it moves nothing.
#[test]
fn a_card_of_th08_held_holds_the_player_still() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-a-card-held-holds-the-player",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frames(5);
        let at = game.image().where_the_player_is();
        game.image().a_card_is_held();
        game.keyboard().set(orb_sim::keys::LEFT, true);
        game.frames(10);
        game.keyboard().set(orb_sim::keys::LEFT, false);
        assert_eq!(
            game.image().where_the_player_is(),
            at,
            "the player moved under a held card",
        );
    });
}

/// **A run of th08 given up goes to the title with nothing pressed after it** — the hand-over the run manager
/// asks for at 0x439f3c waiting for a press, a give-up nobody presses for is orb's to press: a recipe's
/// `leave` answered outright, and a watched attempt's own end, which orb said on every frame for two hundred
/// frames of the laid-out game while the run it was watched in played on.
///
/// **Given up after the rules have played**, which is what every case does: they hold the shot on fifty-nine
/// frames in sixty, so the frame the run is left on carries no edge of its own, where a `leave` on a run's
/// first frame would have one from the shot starting.
#[test]
fn a_run_of_th08_given_up_goes_to_the_title_with_nothing_pressed() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-given-up-with-nothing-pressed",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { frames: 30 }\n- leave: true\n- wait_until: { screen: 0 }\n  \
                     give_up_after: 600\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        // The title up first and the run placed after it, a launch's own title coming up sixty frames in
        // whatever else is laid out.
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the run given up", 120, || {
            game.log().said("the game is on its way to the title")
        });
        game.frames_until("the title", 120, || {
            game.state().scene == Scene::FrontEnd as i32
        });
    });
}

/// **And the hand-over takes only the bits of the run's end out of the kind word** (`& 0xffffff9f`,
/// 0x439f3c through 0x439fb6): a bit the start screen wrote — here the one that starts a run with eight, the
/// practice's — is still there as the front end comes back.
#[test]
fn a_run_of_th08_given_up_keeps_the_kind_its_start_screen_wrote() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-given-up-keeps-its-kind",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { frames: 30 }\n- leave: true\n- wait_until: { screen: 0 }\n  \
                     give_up_after: 600\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().the_run_kind_is(A_START_SCREENS_BIT);
        game.frames_until("the title", 240, || {
            game.state().scene == Scene::FrontEnd as i32
        });
        assert_eq!(
            game.image().the_run_kind() & A_START_SCREENS_BIT,
            A_START_SCREENS_BIT,
            "the hand-over took a bit of the kind word the start screen wrote",
        );
    });
}

/// The bit the stage practice's start screen writes, which starts a run with eight lives and bombs.
const A_START_SCREENS_BIT: u32 = 0x1;

/// **A recipe's `boss` of th08 is the boss fought to a song of her own, and not a midboss fought to the
/// stage's** — the song the conversation before her plays out of the stage's info block (0x434dce), the
/// stage's own being the first of its three names.
///
/// Which a launch of the real game could not reach while the song was unread: a `play_until: { boss: true }`
/// asked in stage 2 played on through that stage's boss and stage 3's into stage 4, and gave up its whole
/// bound of 36,000 frames.
#[test]
fn a_recipes_boss_of_th08_is_the_one_fought_to_a_song_of_her_own() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-a-boss-with-a-song-of-her-own",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { boss: true }\n  give_up_after: 600\n- quit: true\n".to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.its_title_menu_is_up();
        game.is_in_a_run();
        let image = game.image();
        image.a_boss_is_fighting(0, (192.0, 96.0), 1000);
        for _ in 0..30 {
            game.frame();
        }
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "a boss fought to the stage's own song ended a step waiting for the stage's own boss",
        );

        image.the_stream_reads_the_track(THE_FIGHTS_TRACK);
        game.frames_until("the step waiting for the boss done", 30, || {
            game.log().said("autoplay: step 1 of 2 done")
        });
    });
}

/// **What th08's own refill is feeding its music, said once a refill** — the chunk, where it goes and the
/// countdown to the loop, which is the number a restore that left the music going round a stretch of itself
/// would turn over a narrow range: see `Th08`'s `TOPS_UP_THE_MUSIC`.
#[test]
fn the_refill_of_th08_says_what_it_is_feeding_the_buffer() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-the-refill",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frame();
        let (_, chunk, before) = game.image().where_the_stream_is();

        game.refills_its_music();
        game.refills_its_music();

        let said: Vec<String> = game
            .log()
            .lines()
            .into_iter()
            .filter(|line| line.contains("sound: the refill"))
            .collect();
        assert_eq!(said.len(), 2, "not a line per refill: {said:?}");
        for (line, left) in said.iter().zip([before, before - chunk]) {
            assert!(
                line.contains(&format!("puts {chunk} byte(s) in"))
                    && line.contains(&format!("{left} byte(s) of the track left before it loops")),
                "a refill said something other than its own numbers: {line}",
            );
        }
    });
}

/// **A run of th08 orb asks for itself goes through the team select's own press, which takes the front end
/// off** — and a run written into being with the front end left on is one its title starts the attract demo
/// over, 1,500 updates of nothing pressed later (0x467ac4).
///
/// Which is what an attempt watched out of orb's own list was, on the real game, twice: asked for over the
/// title with the scene written, played on with the title still counting under it, and taken by the demo —
/// `diff=0` 133 frames into the watch, the demo's own Easy, and the game gone with its calc list empty.
#[test]
fn a_run_of_th08_orb_asks_for_is_started_by_the_team_select_and_no_demo_takes_it() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-a-run-its-team-select-starts",
            Work::wandering(WORK_US),
        );
        game.its_title_menu_is_up();
        game.frame();
        assert!(
            unsafe { Th08.front_end_starts_a_run(&A_RUN_ORB_ASKS_FOR) },
            "orb put no screen of this game's up to start a run from",
        );
        game.press(orb_sim::keys::Z);
        game.frames_until("the run started", 120, || game.state().in_game);
        for _ in 0..THE_TITLE_WAITS_AND_MORE {
            game.frame();
        }
        let state = game.state();
        assert!(!state.demo, "the demo took a run its team select started");
        assert_eq!(state.difficulty, DIFFICULTY_NORMAL);
    });
}

/// **And the run written into being instead, which is the one the demo takes** — see the test above.
#[test]
fn a_run_of_th08_written_into_being_is_one_its_title_starts_the_demo_over() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-a-run-written-into-being",
            Work::wandering(WORK_US),
        );
        game.its_title_menu_is_up();
        game.frame();
        assert!(unsafe { Th08.start_a_run(&A_RUN_ORB_ASKS_FOR) });
        game.frames_until("the run started", 120, || game.state().in_game);
        for _ in 0..THE_TITLE_WAITS_AND_MORE {
            game.frame();
        }
        assert!(
            game.state().demo,
            "the title left on under a run written into being never started its demo",
        );
    });
}

/// A Normal run of the first team from the first stage, which is what either test asks for.
const A_RUN_ORB_ASKS_FOR: RunStart = RunStart {
    difficulty: DIFFICULTY_NORMAL,
    character: 0,
    shot_type: 0,
    practice: false,
    stage: FIRST_STAGE,
    mode: 0,
};

/// More updates than the title counts before its demo — `0x5dc` at 0x467ac4 — with room past it.
const THE_TITLE_WAITS_AND_MORE: u32 = 0x5dc + 60;

/// **The screen a run of th08 ends at, and the one write that leaves it** — which is what tells a run
/// that played itself out from a run given up: `Th08::leave_run` hands the run straight to the front end
/// and never reaches this scene.
#[test]
fn the_screen_a_run_of_th08_ends_at_is_read_and_left_for_the_front_end() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-screen-a-run-ends-at",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();

        // A run under way is not a run finished, and the screen is not up.
        assert!(
            !unsafe { Th08.run_finished() },
            "orb reads a run under way as finished",
        );
        assert!(
            !unsafe { Th08.the_screen_a_run_ends_at_is_up() },
            "orb reads the screen a run ends at as up inside a run",
        );

        // The run played itself out, which is the scene a launch read on the frame the third life went.
        game.image().is_in_scene(Scene::RunOver, Scene::RunOver);
        assert!(
            unsafe { Th08.run_finished() },
            "orb does not read the scene a finished run of this game is in",
        );
        assert!(
            unsafe { Th08.the_screen_a_run_ends_at_is_up() },
            "orb does not read the screen a run of this game ends at",
        );

        // And the one thing that screen's own transition takes: the front end asked for.
        assert!(
            unsafe { Th08.leave_the_screen_a_run_ends_at() },
            "orb declined to leave the screen a run ends at",
        );
        assert_eq!(
            game.image().the_scene_wanted(),
            Scene::FrontEnd as i32,
            "leaving that screen did not ask the supervisor for the front end",
        );
        // The scene it is *on* is left where it was, the supervisor's own update being what moves it.
        assert_eq!(
            game.image().the_scene_now(),
            Scene::RunOver as i32,
            "leaving that screen moved the scene the game is in rather than asking for one",
        );
    });
}

/// Two `.anm` slots of the sprite manager's `0x100` with a file in them, and how many sheets each has —
/// both of this test's own choosing, and different from each other so that a range answered for one
/// cannot pass for the other's.
const A_SLOT: usize = 3;
const ANOTHER_SLOT: usize = 0x2d;
const A_SLOTS_SHEETS: usize = 4;
const ANOTHER_SLOTS_SHEETS: usize = 1;
/// And the thread a played track is streamed on, which is a number `PlayBGM` would have got from Windows.
const A_THREAD: u32 = 0x1a2b;

/// **The two lists a snapshot of th08 has to leave out**, which is what `Th08::rewinds` was waiting on:
/// the sound's own state and the handles a restore must not write over.
///
/// **What each is, in the laid-out game's own terms.** `audio_state` is the manager's whole `0x5224`
/// bytes — the event a refill waits on and the streaming thread's handle are fields of it — plus the
/// wrapper it keeps DirectSound in, the stream a played track has and the source that stream reads the
/// track through, each a heap object here as it is on the real game. `live_handles` is a range per loaded `.anm` slot over its sheets, where the texture
/// DirectX made and the block the game's allocator gave sit side by side.
///
/// **And the emptier answers are half of what this asserts**: a launch that has played no track answers
/// the manager and nothing else, and a slot with no file in it is no range at all — which is the
/// difference between a chapter that leaves the sound alone and one that holds back a page of whatever
/// the allocator has since put there.
#[test]
fn the_lists_a_snapshot_of_th08_leaves_out_are_the_sounds_own_state_and_the_slots_it_has_loaded() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-what-a-snapshot-leaves-out",
            Work::wandering(WORK_US),
        );
        let image = game.image();

        // Nothing loaded and nothing playing, which is the launch before its first frame.
        for slot in 0..2 {
            image.an_anm_slot_is_empty(slot);
        }
        image.no_track_is_playing();
        assert_eq!(
            unsafe { Th08.live_handles() },
            Vec::new(),
            "orb names a handle of a game that has loaded nothing",
        );
        let manager = image.the_sound_manager_is();
        assert_eq!(
            Th08.audio_state(),
            vec![manager.clone()],
            "orb names more of the sound than the manager itself with no track playing",
        );

        // And the two together: a track playing and two slots loaded.
        let sound = image.a_track_is_playing();
        image.the_music_is_streamed_on_thread(A_THREAD);
        let sheets = [
            image.an_anm_slot_is_loaded(A_SLOT, A_SLOTS_SHEETS),
            image.an_anm_slot_is_loaded(ANOTHER_SLOT, ANOTHER_SLOTS_SHEETS),
        ];

        assert_eq!(
            Th08.audio_state(),
            vec![
                manager,
                sound[0].clone(),
                sound[1].clone(),
                sound[2].clone()
            ],
            "the sound's own state is not the manager, its device, the stream a track has and the source \
             that stream reads through",
        );
        assert_eq!(
            unsafe { Th08.live_handles() },
            sheets.to_vec(),
            "the handles a restore has to leave alone are not the sheets of the loaded slots",
        );
        // And the thread the music is streamed on, which is the one thread a restore leaves running.
        assert_eq!(
            Th08.audio_thread(),
            Some(A_THREAD),
            "orb does not read the thread this game streams its music on",
        );

        // **And whether an ask of the sound is still in flight**, which is the manager's own queue: a
        // restore that ran while one was would be writing over the object the ask is about.
        image.a_sound_command_is_waiting(false);
        assert!(
            !Th08.a_sound_command_is_in_flight(),
            "orb reads a command in flight with the queue's first slot empty",
        );
        image.a_sound_command_is_waiting(true);
        assert!(
            Th08.a_sound_command_is_in_flight(),
            "orb reads no command in flight with a kind in the queue's first slot",
        );
    });
}

/// **The title of th08 reads its records out of the game's own file in either mode, and every other open follows
/// the mode.** The title's own load — the front end's added callback, `Th08`'s `FRONT_END_ADDED` — is what the
/// title's items are lit from and what the save at every run's start writes back, so left on the mode's file it
/// wrote a Pointdevice session's records over the clears in `score.dat`: three Normal clears came to nothing
/// that way, and the Extra is lit from those same records.
#[test]
fn the_title_of_th08_reads_its_records_out_of_the_games_own_file() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-titles-own-read",
            Work::wandering(WORK_US),
        );

        // The mode orb starts a launch of this game in, whose ranking and captures are its own file's.
        for write in [false, true] {
            assert_eq!(
                game.opens_its_score_file(write),
                "pointdevice_score.dat",
                "an open of th08's score file in Pointdevice mode landed in the game's own",
            );
        }

        // **And the title's own read, which is the game's whatever the mode.**
        assert_eq!(
            game.reads_what_its_title_offers(),
            "score.dat",
            "the title's own read did not land in the game's file",
        );
    });
}

/// **The stage a build hands the seed's job reaches that job through orb's hook over it**: the build passes it
/// in `ecx` (0x43b3dc) and the job keeps it at `[0x4e4030 + 0x820]` (0x409b27), so a hook that took it off the
/// stack, or wrote over `ecx` before calling on, leaves whatever the register held there instead.
#[test]
fn the_stage_a_build_of_th08_hands_on_reaches_the_seeds_job() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th08-the-stage-handed-on",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        assert_eq!(
            game.image().the_stage_the_seeds_job_kept(),
            game.image().the_games_stage_word(),
            "the seed's job kept another stage than the one its build was of",
        );
    });
}
