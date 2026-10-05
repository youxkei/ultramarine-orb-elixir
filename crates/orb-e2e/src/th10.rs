//! **The frame orb paces th10 with, and what it reads of a run of it.**
//!
//! A th10 laid out from what has been read of `th10.exe`, orb attached to `Th10`, and orb's own frame loop in
//! place of the game's own frame — which is what a launch there is. What the e2e tests below ask is that the loop
//! composes th10's frame: the queue of quads drawn out either side of the update and put back at its start as the
//! scene begins, the camera set before the update and the fog inside the scene, in the order the game's own frame
//! makes them. Beside that, one field at a time: the scene, the stage and the run's numbers, the set a stage is
//! started again from, the pause menu's retry and the end of a run's menu, and the two lists a snapshot leaves out.
//!
//! **The mode is over most of them**, `Th10::rewinds` answering true: a run placed here is a Pointdevice run, with
//! a chapter taken at its stage's start. Where a test is about the game and not the mode it says so and turns the
//! chapters off.
//!
//! What these cannot say is that the addresses *are* right: a laid-out th10 is written from the same constants
//! `Th10` reads, so a wrong one is wrong on both sides at once. Only the real image says, which is `cargo xtask
//! real` and the cases it runs.

use crate::fake::th10::{
    DIFFICULTY_NORMAL, FIRST_STAGE, Fake, QUEUE_DRAWN, R, THE_FRAMES_OWN_ORDER,
};
use crate::fake::{DRAW, Display, Launched, PRESENT, SOUND, UPDATE, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::frame;
use orb_core::game::th10::Th10;
use orb_core::game::th10::image::{Pause, Player, STEP_READING, Scene, Screen};
use orb_core::game::{CountedDeath, Game};
use orb_sim::keys;

/// What the game's own frame is declared to take here: about what a real run's report line shows for the
/// drawing, so that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long the run is: a few seconds of it, which is past the frames orb spends trying to build an overlay and
/// well into the frames where nothing is happening.
const FRAMES: u32 = 5 * frame::LOGIC_HZ;

/// How many frames a screen, a state or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// **A frame's calls with orb's draw hook over the draw walk**: the game's own, and the queue drawn once more
/// behind the draw walk and ahead of orb's overlay — `Th10::draw_what_is_queued`, the panel's quads being what the
/// overlay has to go over. Which is a frame of either loop, the hook standing over the walk in both.
fn the_frames_own_order_with_orbs_draw_of_the_queue() -> Vec<&'static str> {
    let mut calls = THE_FRAMES_OWN_ORDER.to_vec();
    let behind_the_draw = calls
        .iter()
        .position(|call| *call == DRAW)
        .expect("a frame with a draw walk in it")
        + 1;
    calls.insert(behind_the_draw, QUEUE_DRAWN);
    calls
}

/// A launch of the game with the chapters off, for a test about the game and not about the mode.
fn without_chapters(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapters = false;
        },
    )
}

#[test]
fn orb_paces_a_launch_of_th10_with_no_run_in_it() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th10-paced", Work::wandering(WORK_US));

        // The frame is orb's own, in orb's order: the update first, the sounds where the game's own frame hands
        // them over, then the drawing, then the frame handed to the display — which for this game is the order its
        // own frame has, the calc walk at 0x43942f and the draw walk at 0x4394d8.
        game.forget_asked();
        game.frame();
        let asked: Vec<&str> = game
            .asked()
            .into_iter()
            .filter(|call| [UPDATE, SOUND, DRAW, PRESENT].contains(call))
            .collect();
        assert_eq!(asked, vec![UPDATE, SOUND, DRAW, PRESENT]);

        game.frames(FRAMES);
        let log = game.log().lines().join("\n");
        assert!(
            log.contains("attached to a game laid out in this process"),
            "orb did not attach:\n  {log}"
        );
        // **In orb's own mode to start with**, with the question left for the title: what `Th10::rewinds`
        // answering true puts a launch of this game in.
        assert!(
            log.contains("mode: pointdevice to start with; the menu asks which"),
            "orb did not start this launch in its own mode:\n  {log}"
        );
        for absent in ["chapter ", "retry", "resume: stage", "died in"] {
            assert!(
                !log.contains(absent),
                "orb did {absent:?} to a launch with no run in it:\n  {log}"
            );
        }
        for fatal in ["panic:", "crash:"] {
            assert!(
                !log.contains(fatal),
                "orb wrote a {fatal:?} line over {FRAMES} frames:\n  {log}"
            );
        }
        // Every frame handed over by orb's loop, which is what paces it.
        assert_eq!(game.handovers_us().len(), FRAMES as usize + 1);
    });
}

/// **The queue of quads is drawn out before the scene ends and put back at its start as the scene begins** — the
/// two things this game's own frame does to it that no function of the game's does (0x4394a7 through 0x4394b7,
/// and 0x442f50 at 0x4394e8), and which orb's frame has to do in their places.
#[test]
fn the_queue_of_quads_of_th10_is_drawn_before_the_scene_ends_and_emptied_as_it_begins() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-the-queue",
            Work::wandering(WORK_US),
        );

        game.frames(FRAMES);

        // One quad past the queue's start however many frames have run: put back at the start of every scene,
        // filled by the draw walk, drawn before the scene ended. A loop that never put it back leaves the cursor
        // FRAMES quads along; one that never drew it leaves the quad queued.
        let image = game.image();
        assert_eq!(
            image.queue_writes_at(),
            image.queue_starts_at() + image.queue_bytes_per_quad(),
            "the queue of quads is not one quad along after a frame that queued one and drew it",
        );
        assert_eq!(
            image.queued(),
            0,
            "a quad was left in the queue at the end of a frame"
        );
        // And the game's own count of the draws that had a quad in them, which is one a frame.
        assert_eq!(image.queue_draws(), FRAMES);
    });
}

/// **The camera and the fog are set every frame, in the order the game's own frame sets them**: the queue drawn,
/// the camera and its viewport before the update, and the fog inside the scene before the draw walk — the fog's
/// word forced to `0xff` first (0x4394c2), so that 0x420cd0, which does nothing where that word says nought,
/// draws the queue, puts the word at nought and sets the fog off on every frame (0x420cd6 through 0x420cfa).
#[test]
fn the_camera_and_the_fog_of_th10_are_set_every_frame_in_the_games_own_order() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-the-camera",
            Work::wandering(WORK_US),
        );

        game.forget_asked();
        game.frame();
        assert_eq!(
            game.asked(),
            the_frames_own_order_with_orbs_draw_of_the_queue()
        );

        game.forget();
        let told = game.times_the_fog_was_put_out();
        game.frames(FRAMES);
        assert_eq!(
            game.times_the_fog_was_put_out() - told,
            FRAMES,
            "the device was not told the fog is out once a frame",
        );
        assert_eq!(
            game.image().the_fog_word(),
            0,
            "the fog's word was left forced"
        );
        let camera = game.image().the_cameras_viewport();
        assert_eq!(
            game.drawn()
                .viewports
                .iter()
                .filter(|viewport| **viewport == camera)
                .count(),
            FRAMES as usize,
            "the camera's viewport was not set once a frame",
        );
    });
}

/// **And the game's own frame makes the same calls in the same order**, which is what orb's frame is held against
/// in the test above: a launch with orb's loop turned off runs 0x439390 as the game has it.
#[test]
fn the_games_own_frame_of_th10_makes_its_calls_in_the_order_orbs_frame_makes_them() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-its-own-frame",
            Work::wandering(WORK_US),
            |config| config.own_frame_loop = false,
        );

        game.forget_asked();
        game.frame();
        assert_eq!(
            game.asked(),
            the_frames_own_order_with_orbs_draw_of_the_queue()
        );
    });
}

/// **The scene orb reads is the one this game's supervisor is on**, `[0x491fb4]` and the one it wants at
/// `[0x491fb8]`.
#[test]
fn the_scene_of_th10_orb_reads_is_the_one_its_supervisor_is_on() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-the-scene",
            Work::wandering(WORK_US),
        );
        game.its_title_menu_is_up();
        game.frame();

        let front = game.state();
        assert_eq!(
            (front.scene, front.wanted),
            (Scene::Title as i32, Scene::Title as i32),
            "the scene orb reads at the title is not the title's",
        );
        assert!(
            !front.in_run && !front.playing && !front.in_game,
            "orb reads a run at the title: {front}",
        );
        assert_eq!(
            unsafe { Th10.front_end_screen() },
            Some(Screen::MainMenu as i32),
            "orb does not read the title's own screen number",
        );

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
        assert_eq!(
            unsafe { Th10.front_end_screen() },
            None,
            "orb reads a title under a run"
        );
    });
}

/// **The stage, the difficulty and the run's numbers are read out of the game**: the stage from 1 at `0x474c7c`,
/// the difficulty at `0x474c74`, the lives, and the power — twenty of which is a bomb.
#[test]
fn the_stage_the_difficulty_and_the_numbers_of_a_run_of_th10_are_read_out_of_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-the-numbers",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();
        game.image().the_run_has(1_234_560, 64, 3);

        let state = game.state();
        assert_eq!(
            state.stage, FIRST_STAGE,
            "the stage orb reads is not the run's"
        );
        assert_eq!(state.difficulty, DIFFICULTY_NORMAL);
        assert_eq!(
            (state.lives, state.bombs, state.power),
            (3, 3, 64),
            "the numbers orb reads are not the run's own",
        );
        assert_eq!(
            state.stage_frames, 1,
            "the stage's clock is not on its first frame"
        );
        assert_eq!(
            state.deaths, 0,
            "orb reads a death in a run that has had none"
        );
        // **And the player where the game's constructor builds one**, across the middle and 400 down.
        assert_eq!(game.image().where_the_player_is(), (0.0, 400.0));
    });
}

/// **The set a stage is started again from reads back out of the game's own memory and writes back into it** —
/// the round trip `resume`'s own unit tests cannot make: every field of the replay's record through the address
/// it is read at and the one it is put back at, the options made again for the power between the places and the
/// options' own numbers, and the seed left to its own seam inside the build.
#[test]
fn the_numbers_a_stage_of_th10_is_started_again_from_are_read_back_out_of_the_game_and_put_back() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-the-resume",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();
        let image = game.image();
        image.the_run_has(1_234_560, 64, 3);
        image.the_run_also_has(12_345, 77, 12, 1, 2);
        image.the_generator_is_seeded_with(A_SEED);
        image.the_player_is_at_hundredths(1_200, 34_000);
        image.the_players_trail_is(10);

        let run = &Th10;
        let taken = unsafe { run.run_state() };
        let said = run.say_run_state(&taken);
        assert!(
            said.contains(
                "score=1234560 power=64 faith=12345 faith_stands_for=77 lives=3 474c98=12"
            ) && said.contains("seed=0x4d2f at=1200,34000"),
            "the numbers the run was laid out with are not the set that was read: {said}",
        );

        // Every number of it moved, so that a field put back at the wrong address cannot come out right.
        image.the_run_has(7, 0, 0);
        image.the_run_also_has(0, 0, 0, 0, 0);
        image.the_generator_is_seeded_with(A_SEED_OF_ITS_OWN);
        image.the_player_is_at_hundredths(0, 0);
        image.the_players_trail_is(500);
        assert_ne!(
            unsafe { run.run_state() },
            taken,
            "the numbers read back are the same after every one of them was moved",
        );

        // **The numbers put back without the seed**, which goes in at the build's seam, and the options made for
        // the power once with their flags put at nought after it.
        let made = game.times_the_options_were_made();
        unsafe { run.set_run_state(&taken) };
        assert_eq!(
            image.the_generators_word(),
            A_SEED_OF_ITS_OWN,
            "the numbers put back took the stage's seed with them",
        );
        assert_eq!(game.times_the_options_were_made(), made + 1);
        assert_eq!(image.the_options_moved(), [0; 4]);
        assert_eq!(image.where_the_player_is(), (12.0, 340.0));

        unsafe { run.seed_the_stage(&taken) };
        assert_eq!(
            unsafe { run.run_state() },
            taken,
            "the set put back does not read back as the set that was taken",
        );
    });
}

/// The seed the numbers above are read with, and one they were not.
const A_SEED: u16 = 0x4d2f;
const A_SEED_OF_ITS_OWN: u16 = 0x1234;

/// **A run given up goes to the title**: the scene wanted put at 4, which is what the pause menu's own item for
/// the title writes (0x40ac90 with 4, at 0x42319a), and the title comes up on its main menu with nothing pressed.
#[test]
fn a_run_of_th10_given_up_goes_to_its_title() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-given-up",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { frames: 30 }\n- leave: true\n- wait_until: { screen: 2 }\n  \
                     give_up_after: 600\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the run given up", FRAMES_FOR_A_SCREEN, || {
            game.log().said("the game is on its way to the title")
        });
        game.frames_until("the title's main menu", FRAMES_FOR_A_SCREEN, || {
            game.state().scene == Scene::Title as i32
                && unsafe { Th10.front_end_screen() } == Some(Screen::MainMenu as i32)
        });
        assert!(
            !game.image().the_game_object_is_up(),
            "the run given up left its game object standing"
        );
    });
}

/// **The pause menu's own retry is answered by orb's question, and the run is left where it was**: `R` on the
/// menu closes it on its third item (0x422e28) and twelve frames later wants scene 10 (0x42316c) with the game still
/// paused — which in a mode whose runs are rewound is orb's to answer with where to start over from. So the scene
/// wanted goes back to the run's, the pause menu's own resume takes the pause off (0x422c30), and nothing is built
/// again.
#[test]
fn the_pause_menus_own_retry_of_th10_is_answered_by_orbs_question_and_the_run_left_where_it_was() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-pause-menus-retry",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the stage's first chapter", FRAMES_FOR_A_SCREEN, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        // Past the thirty frames of a stage the pause does not open inside (0x422406).
        game.frames_until("the stage thirty frames in", FRAMES_FOR_A_SCREEN, || {
            game.image().the_games_frames() > 30
        });

        game.press(keys::ESCAPE);
        game.frames_until("the pause menu", FRAMES_FOR_A_SCREEN, || {
            game.image().where_the_pause_is().0 == Pause::Menu as i32
        });
        assert!(
            game.state().paused,
            "orb does not read the pause the menu is up over"
        );
        let built = game.what_the_build_seeded_on().len();

        game.press(R);
        game.frames_until("orb's question", FRAMES_FOR_A_SCREEN, || {
            log.said("retry: the run was asked to start over in chapter 1")
        });
        let image = game.image();
        assert_eq!(
            (image.the_scene_now(), image.the_scene_wanted()),
            (Scene::Run as i32, Scene::Run as i32),
            "the retry the menu asked for was left standing",
        );
        assert!(
            !image.the_game_stands_paused(),
            "the run was left paused under orb's question"
        );
        assert_eq!(game.times_the_run_was_unpaused(), 1);
        game.frames(4);
        assert_eq!(
            game.what_the_build_seeded_on().len(),
            built,
            "the stage was built again under orb's question",
        );
        assert!(!image.the_stage_is_being_built());
        assert!(
            !game.a_frame_that_says(AGAIN).is_empty(),
            "orb's question is not on the screen",
        );
    });
}

/// The retry menu's item for the chapter again, in the language a launch on this machine comes out in.
const AGAIN: &str = "チャプターをやり直す";

/// **The end of a run's menu is taken to the title**: a run over puts the pause object on its end (0x4231d0), a
/// decide and ten frames later on its three-item menu (0x42377e), and orb writes that menu's last item as decided
/// (`Th10::skip_replay_prompt`), which twelve frames later wants the title (0x42431d through 0x424370).
#[test]
fn the_end_of_a_run_of_th10s_menu_is_taken_to_the_title() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-end-of-a-run",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.its_title_menu_is_up();
        // The score object as a startup with no `scoreth10.dat` leaves it, its rankings the game's own empty ones,
        // in which a run that scored nothing has no place (0x421fcd) — so the tally puts up the menu.
        game.image()
            .the_score_object_is_read_from(&Default::default());
        game.is_in_a_run();
        game.frames(2);
        game.image().the_run_is_over_with_a_game_over();
        game.frames(12);
        assert!(
            unsafe { Th10.run_finished() },
            "orb does not read the run as over"
        );
        assert!(
            !game.state().playing,
            "orb reads the end of a run as a run played"
        );

        game.press(keys::Z);
        game.frames_until(
            "the replay's offer written past",
            FRAMES_FOR_A_SCREEN,
            || log.said("result: no replay of the game's own is offered for a run with chapters"),
        );
        assert_eq!(
            game.image().where_the_pause_is().2,
            2,
            "the item orb took is not the one for the title",
        );
        game.frames_until("the title", FRAMES_FOR_A_SCREEN, || {
            game.state().scene == Scene::Title as i32
        });
    });
}

/// Two of the sprite manager's 33 `.anm` slots with a file in them, each with a count of its own of textures and
/// of sprites, and the thread a played track is streamed on — this test's own numbers, different from each other
/// so that a range answered for one cannot pass for another's.
const A_SLOT: usize = 7;
const ANOTHER_SLOT: usize = 0x1c;
const A_THREAD: u32 = 0x1a2b;

/// **The lists a snapshot leaves out are the sound's state, the `.anm` files' textures and sprites, the capture
/// surfaces and the supervisor's locks** — `Th10::audio_state` and `Th10::live_handles` — with the emptier answers
/// beside them: a launch whose sound is not up answers the manager alone, and one with no sprite manager the locks
/// alone.
#[test]
fn the_lists_a_snapshot_of_th10_leaves_out_are_the_sound_the_anm_files_the_surfaces_and_the_locks()
{
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th10-what-a-snapshot-leaves-out",
            Work::wandering(WORK_US),
        );
        let image = game.image();
        let manager = image.the_sound_manager_is();
        let locks = image.the_supervisors_locks();

        image.the_sound_is_not_up();
        image.the_sprite_manager_is_not_made();
        assert_eq!(Th10.audio_state(), vec![manager.clone()]);
        assert_eq!(unsafe { Th10.live_handles() }, vec![locks.clone()]);

        image.the_sprite_manager_is_made();
        assert_eq!(
            unsafe { Th10.live_handles() },
            vec![locks.clone(), image.the_capture_surfaces()],
            "a sprite manager with no file loaded names more than its surfaces",
        );

        let sound = image.the_sound_is_up();
        image.the_music_is_streamed_on_thread(A_THREAD);
        let [a_files_textures, a_files_sprites] = image.an_anm_file_is_loaded(A_SLOT, 2, 5);
        let [another_files_textures, another_files_sprites] =
            image.an_anm_file_is_loaded(ANOTHER_SLOT, 1, 3);
        assert_eq!(
            Th10.audio_state(),
            vec![
                manager,
                sound[0].clone(),
                sound[1].clone(),
                sound[2].clone(),
                sound[3].clone()
            ],
            "the sound's state is not the manager, its DirectSound, the stream, its buffer and its source",
        );
        assert_eq!(
            unsafe { Th10.live_handles() },
            vec![
                locks,
                image.the_capture_surfaces(),
                a_files_textures,
                a_files_sprites,
                another_files_textures,
                another_files_sprites,
            ],
        );
        assert_eq!(Th10.audio_thread(), Some(A_THREAD));

        assert!(!Th10.a_sound_command_is_in_flight());
        image.a_sound_command_is_queued(orb_core::game::th10::image::THE_PAUSES_COMMAND);
        assert!(Th10.a_sound_command_is_in_flight());
    });
}

/// **A run chosen at its shot select is built on its loader and begins on its first frame**: the title's screens
/// walked by their own presses, the scene wanted as 7 forty frames after the shot's decide (0x430f5a), the game
/// object built with its flag for a build set and the stage's frames at nought for as long as the build runs, the
/// replay's seam reached inside it (0x417a86), and the stage begun on the first walk after it — `main` at the head
/// of the enemies' list and the player playing where its constructor put it.
#[test]
fn a_run_of_th10_chosen_at_its_shot_select_is_built_on_its_loader_and_begins_on_its_first_frame() {
    in_its_own_process(|| {
        let game = without_chapters("th10-a-run-chosen");
        let image = game.image();
        let reading = |screen: Screen| {
            let (on, step, _, _) = image.where_the_title_is();
            image.the_title_is_up() && on == screen as i32 && step == STEP_READING
        };
        for screen in [
            Screen::PressAKey,
            Screen::MainMenu,
            Screen::Difficulty,
            Screen::Character,
            Screen::Shot,
        ] {
            game.frames_until(
                &format!("{screen:?} reading a key"),
                FRAMES_FOR_A_SCREEN,
                || reading(screen),
            );
            game.press(keys::Z);
        }
        game.frames_until("the run wanted", FRAMES_FOR_A_SCREEN, || {
            image.the_scene_wanted() == Scene::Run as i32
        });
        let chosen = game.state();
        assert_eq!(
            chosen.scene,
            Scene::Title as i32,
            "the run was built on the frame it was wanted"
        );
        assert!(unsafe { Th10.run_chosen() });

        game.frames_until("the build", 4, || image.the_stage_is_being_built());
        let mut built_over = 0;
        while image.the_stage_is_being_built() {
            assert!(unsafe { Th10.mid_build() });
            assert_eq!(
                image.the_stages_frames(),
                0,
                "the stage's frames moved inside its build"
            );
            game.frame();
            built_over += 1;
            assert!(built_over < FRAMES_FOR_A_SCREEN, "the build never ended");
        }
        assert_eq!(
            game.what_the_build_seeded_on(),
            vec![(0x4, 0)],
            "the replay's seam was not reached inside the build with the stage at its frame 0",
        );
        assert_eq!(
            image.the_games_frames(),
            0,
            "the game object ran a frame of its build"
        );

        game.frame();
        let run = game.state();
        assert!(run.in_game, "no run was begun after its build: {run}");
        assert_eq!(
            (run.stage, run.difficulty, run.stage_frames),
            (FIRST_STAGE, DIFFICULTY_NORMAL, 1)
        );
        assert_eq!(
            (run.lives, run.power),
            (2, 0),
            "the build's defaults are not a fresh run's"
        );
        assert_eq!((image.the_character(), image.the_shot()), (0, 0));
        let enemies = image.the_enemies_on_the_list();
        assert_eq!(
            enemies.len(),
            1,
            "the stage began with other than `main` on its list"
        );
        assert_eq!(image.the_flags_of(enemies[0]), 0x10);
        assert_eq!(image.what_the_player_is_doing().0, Player::Playing as i32);
        assert_eq!(image.where_the_player_is(), (0.0, 400.0));
    });
}

/// **A hit is counted once and taken with a life**: 0x426cf0 puts the player in state 4, the eight frames a bomb
/// could take it back in stand open, the death takes a life and dies for thirty frames (0x4269d0, 0x425b9d), the
/// player comes back in from the bottom over sixty (0x425751 through 0x42590f) — and a hit with no life left ends
/// the run on the end of a run's screen (0x4231d0).
#[test]
fn a_hit_the_player_of_th10_takes_is_counted_once_and_taken_with_a_life() {
    in_its_own_process(|| {
        let game = without_chapters("th10-a-hit");
        let image = game.image();
        game.is_in_a_run();
        game.frames(2);
        image.the_run_has(0, 0, 1);

        game.the_player_is_hit();
        game.frame();
        assert_eq!(game.state().deaths, 1, "the hit was not counted");
        assert!(game.state().unsettled);
        assert_eq!(
            unsafe { Th10.where_a_counted_death_stands() },
            CountedDeath::Open
        );

        game.frames_until("the player dying", FRAMES_FOR_A_SCREEN, || {
            image.what_the_player_is_doing().0 == Player::Dying as i32
        });
        assert_eq!(
            unsafe { Th10.where_a_counted_death_stands() },
            CountedDeath::Taken
        );
        assert_eq!(image.the_runs_numbers().2, 0, "the death took no life");

        game.frames_until("the player coming back in", FRAMES_FOR_A_SCREEN, || {
            image.what_the_player_is_doing().0 == Player::ComingBackIn as i32
        });
        assert_eq!(image.where_the_player_is(), (0.0, 480.0));
        game.frames_until("the player playing again", FRAMES_FOR_A_SCREEN, || {
            image.what_the_player_is_doing().0 == Player::Playing as i32
        });
        assert_eq!(image.where_the_player_is(), (0.0, 400.0));
        assert_eq!(game.state().deaths, 1, "one hit was counted more than once");

        game.the_player_is_hit();
        game.frames_until("the end of the run", FRAMES_FOR_A_SCREEN, || {
            image.where_the_pause_is().0 == Pause::TheRunIsOver as i32
        });
        assert!(unsafe { Th10.run_finished() });
        assert_eq!(game.state().deaths, 2);
    });
}

/// **The last stage won goes to the ending, which orb skips to the title after it**: the panel's count of `0x78`
/// frames after the stage's end (0x414918) and the scene wanted at 14 (0x414937); orb playing the ending's frames
/// over inside one frame of its own until the scene moves on; and the title after it, scene 15's switch wanting
/// the title with its word at 3 (0x4219e2 through 0x4219ec) and the title's entry putting the screen after the
/// ending up on it (0x42cfc4).
#[test]
fn the_last_stage_of_a_run_of_th10_won_goes_through_its_ending_to_the_title_after_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-ending",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.fires_at_the_player(false);
        game.stages_last(THE_LAST_STAGE_LASTS);
        game.plays_its_own_stage();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().the_stage_word_is(THE_LAST_STAGE);
        let log = game.log();
        game.frames_until("the ending skipped", THE_LAST_STAGE_LASTS * 2, || {
            log.said("ending skipped")
        });
        game.frames_until("the title after the ending", FRAMES_FOR_A_SCREEN, || {
            game.state().scene == Scene::Title as i32
                && unsafe { Th10.front_end_screen() } == Some(Screen::AfterTheEnding as i32)
        });
        assert!(
            !log.said("ending skipped, 54000 frames run, scene 14 -> 14"),
            "orb's skip of the ending ran out without the ending ending:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **The title after the ending takes the clear's name at its grid and leaves the replay's slots on a cancel**:
/// the name entry of screen 15 (0x432cb0) walked by the arrows and decided on — a character written at the
/// place, the end cell copying the name into the ranked place and the last name typed (0x4330b1 through
/// 0x43310c) — then the slots of screen 16 (0x4331fd), whose cancel's step 4 puts the main menu up (0x433a7e).
/// With the chapters off, so that what answers the screens is the keys and nothing of orb's.
#[test]
fn the_title_after_th10s_ending_names_the_clear_and_leaves_the_replays_slots_on_a_cancel() {
    in_its_own_process(|| {
        let game = without_chapters("th10-the-title-after-the-ending");
        game.fires_at_the_player(false);
        game.stages_last(THE_LAST_STAGE_LASTS);
        game.plays_its_own_stage();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.image().the_stage_word_is(THE_LAST_STAGE);
        let reading = |screen: Screen| {
            let (on, step, _, _) = game.image().where_the_title_is();
            game.state().scene == Scene::Title as i32 && on == screen as i32 && step == STEP_READING
        };
        game.frames_until(
            "the name entry after the ending",
            THE_LAST_STAGE_LASTS * 2,
            || reading(Screen::AfterTheEnding),
        );
        let entry = game.image().the_titles_entry();
        assert!(
            !entry.not_asked,
            "the clear was not asked for a name, its score having no place in the ranking",
        );

        // Every character there was stepped back over, then the grid's cell a row down from the end — `M`, the
        // thirteenth — decided on, and the end a row up from it decided on.
        for _ in 0..8 {
            game.press(keys::X);
        }
        assert_eq!(
            game.image().the_titles_entry().typed,
            0,
            "the cancels did not step back over the name"
        );
        game.press(keys::DOWN);
        game.press(keys::Z);
        game.press(keys::UP);
        game.press(keys::Z);
        let named = *b"M       ";
        assert_eq!(
            (
                game.image().the_name_a_ranked_place_holds(0),
                game.image().the_last_name_typed()
            ),
            (named.to_vec(), named),
            "the name typed is not the ranked place's and the last one typed",
        );

        game.frames_until("the replay's slots", FRAMES_FOR_A_SCREEN, || {
            reading(Screen::TheReplaysSlots)
        });
        game.press(keys::X);
        game.frames_until("the main menu", FRAMES_FOR_A_SCREEN, || {
            reading(Screen::MainMenu)
        });
    });
}

/// The last stage, counted from 1, and how long the laid-out one lasts here: no midboss, and the boss for its last
/// 900 frames.
const THE_LAST_STAGE: i32 = 6;
const THE_LAST_STAGE_LASTS: u32 = 1_500;

/// **A hit of th10's puts a picture of the frame into the trace**, taken out of the game's Direct3D 9 device after
/// its own drawing has ended the scene: a 32-bit BMP, which is what `orb-real picture` writes out.
#[test]
fn a_hit_of_th10_puts_a_picture_of_the_frame_into_the_trace() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-a-hit-puts-a-picture-into-the-trace",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- play_until: { frames: 400 }\n".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
            },
        );
        game.is_in_a_run();
        game.frames(2);
        game.the_player_is_hit();
        game.frames(4);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let pictures: Vec<&Vec<u8>> = records
            .iter()
            .filter_map(|record| record.picture.as_ref())
            .collect();
        let [picture] = pictures.as_slice() else {
            panic!(
                "one picture is not what the trace holds: {} of them\n  {}",
                pictures.len(),
                game.log().lines().join("\n  ")
            );
        };
        assert_eq!(&picture[..2], b"BM", "the trace's picture is no BMP");
    });
}

/// **What a frame of th10 spent is in the trace, frame by frame**: the spans the pacing marks a frame by, the
/// game's update among them, and orb's own phases inside them — the rules' steering among those, on a run the
/// rules play, and the reading of the field it steers by inside that.
#[test]
fn what_a_frame_of_th10_spent_is_in_the_trace_the_rules_steering_with_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-what-a-frame-spent-is-in-the-trace",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- play_until: { frames: 400 }\n".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
            },
        );
        game.is_in_a_run();
        game.frames(30);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let named =
            |spans: &[(String, u32)], name: &str| spans.iter().any(|(span, _)| span == name);
        let spent = |name: &str| {
            records
                .iter()
                .filter(|record| named(&record.spans, name))
                .count()
        };
        let phased = |name: &str| {
            records
                .iter()
                .filter(|record| named(&record.phases, name))
                .count()
        };
        for span in ["gap", "wait", "update", "draw", "present"] {
            assert!(
                spent(span) >= 20,
                "{} of {} records hold the frame's {span}",
                spent(span),
                records.len()
            );
        }
        for phase in ["steering", "sight"] {
            assert!(
                phased(phase) >= 20,
                "{} of {} records hold the rules' {phase}",
                phased(phase),
                records.len()
            );
        }
    });
}

/// **The world is traced on the frames of the card a launch names the field of, and on no others, where it asks
/// for both**: three frames of another card and three of the named one, and the world in records of the named
/// card's field alone.
#[test]
fn the_world_of_th10_is_traced_through_the_card_its_field_is_traced_in_alone() {
    in_its_own_process(|| {
        const NAMED: i32 = 107;
        const ANOTHER: i32 = 106;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-world-in-a-card",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
                config.trace_the_field_in = Some(NAMED as u32);
                config.trace_the_world = true;
            },
        );
        game.is_in_a_run();
        game.image().a_card_is_up(ANOTHER);
        game.frames(3);
        game.image().a_card_is_up(NAMED);
        game.frames(3);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let with_the_world: Vec<_> = records
            .iter()
            .filter(|record| record.the_world.is_some())
            .collect();
        assert!(!with_the_world.is_empty(), "no record holds the world");
        assert!(
            with_the_world.iter().all(|record| record
                .the_field
                .as_ref()
                .is_some_and(|field| field.card == NAMED as u32)),
            "the world was traced on a frame of no field of the named card: {:?}",
            with_the_world
                .iter()
                .map(|record| record.the_field.as_ref().map(|field| field.card))
                .collect::<Vec<_>>(),
        );
    });
}
