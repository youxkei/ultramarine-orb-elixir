//! **The frame orb paces th11 with, and what it reads of a run of it.**
//!
//! A th11 laid out from what has been read of `th11.exe`, orb attached to `Th11`, and orb's own frame loop in
//! place of the game's own frame — which is what a launch there is. What the e2e tests below ask is that the loop
//! composes th11's frame: the queue of quads drawn out either side of the update and put back at its start as the
//! scene begins, the camera set before the update and the fog inside the scene, in the order the game's own frame
//! at 0x446420 makes them. Beside that, one field at a time: the scene, the stage and the run's numbers, the set a
//! stage is started again from, the pause menu's retry, the two lists a snapshot leaves out, and the stage's
//! background held back over a fade.
//!
//! **The mode is over most of them**, `Th11::rewinds` answering true: a run placed here is a Pointdevice run, with
//! a chapter taken at its stage's start. Where a test is about the game and not the mode it says so and turns the
//! chapters off.
//!
//! What these cannot say is that the addresses *are* right: a laid-out th11 is written from the same constants
//! `Th11` reads, so a wrong one is wrong on both sides at once. Only the real image says, which is `cargo xtask
//! real` and the cases it runs.

use crate::fake::th11::{
    DIFFICULTY_NORMAL, FIRST_STAGE, Fake, QUEUE_DRAWN, R, THE_FRAMES_OWN_ORDER,
};
use crate::fake::{DRAW, Display, Launched, PRESENT, SOUND, UPDATE, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::frame;
use orb_core::game::th11::Th11;
use orb_core::game::th11::image::{Pause, Player, STEP_READING, Scene, Screen};
use orb_core::game::{CountedDeath, Game};
use orb_sim::keys;

/// What the game's own frame is declared to take: about what a real run's report line shows for the
/// drawing, so that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long the run is: a few seconds of it, which is past the frames orb spends trying to build an overlay and
/// well into the frames where nothing is happening.
const FRAMES: u32 = 5 * frame::LOGIC_HZ;

/// How many frames a screen, a state or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// **A frame's calls with orb's draw hook over the draw walk**: the game's own, and the queue drawn once more
/// behind the draw walk and ahead of orb's overlay — `Th11::draw_what_is_queued`, the panel's quads being what the
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

/// **And orb's own frame**, which is that and the call every game's frame of orb's makes for the sounds behind the
/// update — on this game `Th11`'s own `plays_no_sounds`, which calls nothing of the game's: the sound's commands
/// are carried out inside the calc walk (0x429058), and a second carrying out a walk would play a queued command
/// out twice as fast.
fn orbs_own_frame() -> Vec<&'static str> {
    let mut calls = the_frames_own_order_with_orbs_draw_of_the_queue();
    let behind_the_update = calls
        .iter()
        .position(|call| *call == UPDATE)
        .expect("a frame with a calc walk in it")
        + 1;
    calls.insert(behind_the_update, SOUND);
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
fn orb_paces_a_launch_of_th11_with_no_run_in_it() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th11-paced", Work::wandering(WORK_US));

        // The frame is orb's own, in orb's order: the update first, the sounds where orb's frame calls a game's,
        // then the drawing, then the frame handed to the display — the order this game's own frame has, the calc
        // walk at 0x4464e1 and the draw walk at 0x446571, with the sounds' call `Th11`'s own and nothing of the
        // game's (see `orbs_own_frame`).
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
        // **In orb's own mode to start with**, with the question left for the title: what `Th11::rewinds`
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
/// two things this game's own frame does to it that no function of the game's does (0x446540 through 0x446557,
/// and 0x44fd10 at 0x44657c), and which orb's frame has to do in their places.
#[test]
fn the_queue_of_quads_of_th11_is_drawn_before_the_scene_ends_and_emptied_as_it_begins() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-queue",
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
/// the camera and its viewport before the update (0x4464a9, 0x4464cf), and the fog inside the scene before the draw
/// walk — the fog's word forced to `0xff` first (0x446562), so that 0x42a300, which does nothing where that word
/// says nought, draws the queue, puts the word at nought and sets the fog off on every frame (0x42a300 through
/// 0x42a32f).
#[test]
fn the_camera_and_the_fog_of_th11_are_set_every_frame_in_the_games_own_order() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-camera",
            Work::wandering(WORK_US),
        );

        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), orbs_own_frame());

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
/// in the test above, less the sounds' call that is orb's alone: a launch with orb's loop turned off runs 0x446420
/// as the game has it.
#[test]
fn the_games_own_frame_of_th11_makes_its_calls_in_the_order_orbs_frame_makes_them() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-its-own-frame",
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

/// **The scene orb reads is the one this game's supervisor is on**, its `+0x554` at `0x4c37d4`, and the one it
/// wants, its `+0x558` at `0x4c37d8`.
#[test]
fn the_scene_of_th11_orb_reads_is_the_one_its_supervisor_is_on() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-scene",
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
            unsafe { Th11.front_end_screen() },
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
            unsafe { Th11.front_end_screen() },
            None,
            "orb reads a title under a run"
        );
    });
}

/// **The stage, the difficulty and the run's numbers are read out of the game**: the stage from 1 at `0x4a5728`,
/// the difficulty at `0x4a5720`, the lives at `0x4a5718`, and the power at `0x4a56e8` — a bomb being the power one
/// costs, `[0x4a574c]`, which is twenty in the laid-out shot type.
#[test]
fn the_stage_the_difficulty_and_the_numbers_of_a_run_of_th11_are_read_out_of_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-numbers",
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
    });
}

/// **The frames a card has run are its own timer's count**, `[0x4a8d6c]+0x88c`: nought at its start (0x40c674) and
/// a frame on at each of its calc job's (0x40c147) — and nothing between cards.
#[test]
fn the_frames_a_card_of_th11_has_run_are_read_off_its_own_timer() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-cards-frames",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();
        assert_eq!(game.state().boss_attack_frames, None, "no card is up");

        game.image().a_card_is_up(169);
        assert_eq!(game.state().boss_attack_frames, Some(0));
        for _ in 0..3 {
            game.frame();
        }
        let state = game.state();
        assert_eq!(state.spellcard, Some(169));
        assert_eq!(
            state.boss_attack_frames,
            Some(3),
            "the card's frames are not its timer's"
        );

        game.image().no_card_is_up();
        game.frame();
        assert_eq!(game.state().boss_attack_frames, None, "the card is down");
    });
}

/// **And between cards the frames of the boss's own attack**, `+0x1194` of the enemy the boss slot holds: a frame on
/// at the end of each of its updates (0x412793), and back at nought where 0x417b40 hands it its next attack.
#[test]
fn the_frames_a_boss_of_th11_has_attacked_for_between_cards_are_read_off_her_own_timer() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-bosss-frames",
            Work::wandering(WORK_US),
        );
        game.fires_at_the_player(false);
        game.is_in_a_run();
        game.frames_until("the stage's first frames", 120, || {
            game.state().in_run && game.state().stage_frames > 0
        });
        game.image().a_fight_is_up((0.0, 96.0), 10_000);
        for _ in 0..3 {
            game.frame();
        }
        assert_eq!(
            game.state().boss_attack_frames,
            Some(3),
            "the boss's frames between cards are not her attack's timer's",
        );

        game.image().the_fight_attacks_again();
        game.frame();
        assert_eq!(
            game.state().boss_attack_frames,
            Some(1),
            "her next attack did not count from nought",
        );
    });
}

/// **The set a stage is started again from reads back out of the game's own memory and writes back into it** —
/// the round trip `resume`'s own unit tests cannot make: every field of the replay's record through the address
/// it is read at and the one it is put back at, the options made again for Patchouli's element (0x436f02 through
/// 0x436f13), and the seed left to its own seam inside the build.
#[test]
fn the_numbers_a_stage_of_th11_is_started_again_from_are_read_back_out_of_the_game_and_put_back() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-the-resume",
            Work::wandering(WORK_US),
        );
        game.is_in_a_run();
        game.frame();
        let image = game.image();
        image.the_run_has(1_234_560, 64, 3);
        image.the_run_also_has(12_345, 2, 12, 1, 2, 77);
        image.the_generator_is_seeded_with(A_SEED);
        image.the_player_is_at_128ths(1_536, 43_520);
        image.the_players_element_is(AN_ELEMENT);

        let run = &Th11;
        let taken = unsafe { run.run_state() };
        let said = run.say_run_state(&taken);
        assert!(
            said.contains(
                "score=1234560 power=64 point_value=12345 lives=3 life_pieces=2 4a5744=12 continues=1 \
                 extends=2 graze=77"
            ) && said.contains("seed=0x4d2f at=1536,43520 element=2"),
            "the numbers the run was laid out with are not the set that was read: {said}",
        );

        // Every number of it moved, so that a field put back at the wrong address cannot come out right.
        image.the_run_has(7, 0, 0);
        image.the_run_also_has(0, 0, 0, 0, 0, 0);
        image.the_generator_is_seeded_with(A_SEED_OF_ITS_OWN);
        image.the_player_is_at_128ths(0, 0);
        image.the_players_element_is(0);
        assert_ne!(
            unsafe { run.run_state() },
            taken,
            "the numbers read back are the same after every one of them was moved",
        );

        // **The numbers put back without the seed**, which goes in at the build's seam, and the options made again
        // for the power twice where the element is not nought — with the count of them the power's.
        let made = game.times_the_options_were_made();
        unsafe { run.set_run_state(&taken) };
        assert_eq!(
            image.the_generators_word(),
            A_SEED_OF_ITS_OWN,
            "the numbers put back took the stage's seed with them",
        );
        assert_eq!(game.times_the_options_were_made(), made + 2);
        assert_eq!(
            image.the_players_options(),
            64 / A_BOMB_COSTS,
            "the options made again are not the power's count of them",
        );
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
/// An element of Patchouli's other than her first, which is what has the playback make the options again.
const AN_ELEMENT: i32 = 2;
/// What a bomb costs in the laid-out shot type, which is what the power is counted in options of (0x432cee).
const A_BOMB_COSTS: i32 = 20;

/// **A run given up goes to the title**: the scene wanted put at 4, which is what the pause menu's own item for
/// the title writes, and the title comes up on its main menu with nothing pressed.
#[test]
fn a_run_of_th11_given_up_goes_to_its_title() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-given-up",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { frames: 30 }\n- leave: true\n- wait_until: { screen: 1 }\n  \
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
                && unsafe { Th11.front_end_screen() } == Some(Screen::MainMenu as i32)
        });
        assert!(
            !game.image().the_game_object_is_up(),
            "the run given up left its game object standing"
        );
    });
}

/// **The pause menu's own retry is answered by orb's question, and the run is left where it was**: `R` on the
/// menu (0x42cb38) closes it on its start over and twelve frames later wants scene 10 with the game still paused —
/// which in a mode whose runs are rewound is orb's to answer with where to start over from. So the scene wanted
/// goes back to the run's, the pause menu's own resume takes the pause off (0x42c880), and nothing is built again.
#[test]
fn the_pause_menus_own_retry_of_th11_is_answered_by_orbs_question_and_the_run_left_where_it_was() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-the-pause-menus-retry",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the stage's first chapter", FRAMES_FOR_A_SCREEN, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        // Past the thirty frames of a stage the pause does not open inside (0x42bf4a).
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

/// Two of the sprite manager's 32 `.anm` slots with a file in them, each with a count of its own of textures and
/// of sprites, and the thread a played track is streamed on — this test's own numbers, different from each other
/// so that a range answered for one cannot pass for another's.
const A_SLOT: usize = 7;
const ANOTHER_SLOT: usize = 0x1c;
const A_THREAD: u32 = 0x1a2b;

/// **The lists a snapshot leaves out are the sound's state, the `.anm` files' textures and sprites, the
/// supervisor's surfaces and its locks** — `Th11::audio_state` and `Th11::live_handles` — with the emptier answers
/// beside them: a launch whose sound is not up answers the manager alone, and one with no sprite manager the locks
/// and the surfaces alone, those being the supervisor's and not the manager's (0x42b530).
#[test]
fn the_lists_a_snapshot_of_th11_leaves_out_are_the_sound_the_anm_files_the_surfaces_and_the_locks()
{
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th11-what-a-snapshot-leaves-out",
            Work::wandering(WORK_US),
        );
        let image = game.image();
        let manager = image.the_sound_manager_is();
        let locks = image.the_supervisors_locks();
        let surfaces = image.the_supervisors_surfaces();

        image.the_sound_is_not_up();
        image.the_sprite_manager_is_not_made();
        assert_eq!(Th11.audio_state(), vec![manager.clone()]);
        assert_eq!(
            unsafe { Th11.live_handles() },
            vec![locks.clone(), surfaces.clone()]
        );

        image.the_sprite_manager_is_made();
        for slot in 0..32 {
            image.an_anm_slot_is_empty(slot);
        }
        assert_eq!(
            unsafe { Th11.live_handles() },
            vec![locks.clone(), surfaces.clone()],
            "a sprite manager with no file loaded names more than the supervisor's own",
        );

        let sound = image.the_sound_is_up();
        image.the_music_is_streamed_on_thread(A_THREAD);
        let [a_files_textures, a_files_sprites] = image.an_anm_file_is_loaded(A_SLOT, 2, 5);
        let [another_files_textures, another_files_sprites] =
            image.an_anm_file_is_loaded(ANOTHER_SLOT, 1, 3);
        assert_eq!(
            Th11.audio_state(),
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
            unsafe { Th11.live_handles() },
            vec![
                locks,
                surfaces,
                a_files_textures,
                a_files_sprites,
                another_files_textures,
                another_files_sprites,
            ],
        );
        assert_eq!(Th11.audio_thread(), Some(A_THREAD));

        assert!(!Th11.a_sound_command_is_in_flight());
        image.a_sound_command_is_queued(orb_core::game::th11::image::THE_PAUSES_COMMAND);
        assert!(Th11.a_sound_command_is_in_flight());
    });
}

/// **A run chosen at its partner select is built on its loader and begins on its first frame**: the title's screens
/// walked by their own presses, the scene wanted as 7 forty frames after the partner's decide (0x43d5fc), the game
/// object built with its flag for a build set (0x42031b) and the stage's frames at nought for as long as the build
/// runs, the
/// replay's seam reached inside it (0x41fc2b), and the stage begun on the first walk after it — `main` at the head
/// of the enemies' list and the player playing.
#[test]
fn a_run_of_th11_chosen_at_its_partner_select_is_built_on_its_loader_and_begins_on_its_first_frame()
{
    in_its_own_process(|| {
        let game = without_chapters("th11-a-run-chosen");
        let image = game.image();
        let reading = |screen: Screen| {
            let (on, step, _, _) = image.where_the_title_is();
            image.the_title_is_up() && on == screen as i32 && step == STEP_READING
        };
        for screen in [
            Screen::MainMenu,
            Screen::Difficulty,
            Screen::Character,
            Screen::Partner,
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
        assert!(unsafe { Th11.run_chosen() });

        game.frames_until("the build", 4, || image.the_stage_is_being_built());
        let mut built_over = 0;
        while image.the_stage_is_being_built() {
            assert!(unsafe { Th11.mid_build() });
            assert_eq!(
                image.the_stages_frames(),
                0,
                "the stage's frames moved inside its build"
            );
            game.frame();
            built_over += 1;
            assert!(built_over < FRAMES_FOR_A_SCREEN, "the build never ended");
        }
        // The flags `0x4`, the build running, and `0x80`, which the game object's calc job writes on every frame it
        // waits on that (0x4206cb).
        assert_eq!(
            game.what_the_build_seeded_on(),
            vec![(0x84, 0)],
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
        assert_eq!(run.lives, 2, "the build's defaults are not a fresh run's");
        assert_eq!((image.the_character(), image.the_shot()), (0, 0));
        let enemies = image.the_enemies_on_the_list();
        assert_eq!(
            enemies.len(),
            1,
            "the stage began with other than `main` on its list"
        );
        assert_eq!(image.what_the_player_is_doing().0, Player::Playing as i32);
    });
}

/// **A hit is counted once and taken with a life**: 0x432a90 puts the player in state 4, the eight frames a bomb
/// could take it back in stand open, the death takes a life and dies for thirty frames (0x4327d0, 0x4313e6), the
/// player comes back in from the bottom over sixty (0x431095 through 0x431188) — and a hit with no life left ends
/// the run on the lost run's tally (0x4313f5, 0x431411).
#[test]
fn a_hit_the_player_of_th11_takes_is_counted_once_and_taken_with_a_life() {
    in_its_own_process(|| {
        let game = without_chapters("th11-a-hit");
        let image = game.image();
        game.is_in_a_run();
        game.frames(2);
        image.the_run_has(0, 0, 1);

        game.the_player_is_hit();
        game.frame();
        assert_eq!(game.state().deaths, 1, "the hit was not counted");
        assert!(game.state().unsettled);
        assert_eq!(
            unsafe { Th11.where_a_counted_death_stands() },
            CountedDeath::Open
        );

        game.frames_until("the player dying", FRAMES_FOR_A_SCREEN, || {
            image.what_the_player_is_doing().0 == Player::Dying as i32
        });
        assert_eq!(
            unsafe { Th11.where_a_counted_death_stands() },
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
            image.where_the_pause_is().0 == Pause::ALostRunTallies as i32
        });
        assert!(unsafe { Th11.run_finished() });
        assert_eq!(game.state().deaths, 2);
    });
}

/// **Reimu with Yukari is carried out through a side wall and in at the other**: pressed against the left wall, the
/// direction let go and left pressed again, the player goes on left at `0x96` 128ths a frame with nothing pressed, put
/// across the field past `0x6800`, and comes to rest beside the right wall 45 frames on — and not with the shot held
/// through the presses.
#[test]
fn reimu_with_yukari_of_th11_is_carried_out_through_a_side_wall_and_in_at_the_other() {
    in_its_own_process(|| {
        let game = without_chapters("th11-the-gap");
        let image = game.image();
        game.is_in_a_run();
        game.frames(2);
        let floor = 400 * 128;
        let through = |with_the_shot: bool| {
            image.the_player_is_at_128ths(-0x5c00 + 0x200, floor);
            game.keyboard().set(keys::Z, with_the_shot);
            game.keyboard().set(keys::LEFT, true);
            game.frames(10);
            game.keyboard().set(keys::LEFT, false);
            game.frame();
            game.keyboard().set(keys::LEFT, true);
            game.frame();
            game.keyboard().set(keys::LEFT, false);
            game.keyboard().set(keys::Z, false);
            game.frames(50);
            image.where_the_player_is_in_128ths()
        };
        let (x, y) = through(false);
        assert!(
            (0x5c00 - 0x800..=0x5c00).contains(&x) && y == floor,
            "not carried through to beside the right wall: {x:#x},{y:#x}"
        );
        assert_eq!(image.the_players_gap().0, 0, "the gap was not over");
        let (x, _) = through(true);
        assert_eq!(x, -0x5c00, "carried through with the shot held");
    });
}

/// **The background of a stage entered out of the last one's fade steps its script first on the update after the
/// stage begins**: the new `.std` put at a count of 60 with its fade-in bit (0x405215, 0x40522f), the calc stepping no
/// script while that bit stands with the count at 30 or over (0x402b1c through 0x402b36), and the stage begun on the
/// fade's thirtieth frame (0x4205b1) — where a stage built fresh steps it on the update the stage begins, the game
/// object's priority `0xa` ahead of the `.std`'s `0xc`. Which is what `Th11::set_run_state` puts a pick-up of such a
/// stage back to.
#[test]
fn the_background_of_a_stage_of_th11_entered_out_of_a_fade_steps_first_on_the_update_after_it_begins()
 {
    in_its_own_process(|| {
        let game = without_chapters("th11-the-background-held-over-a-fade");
        game.fires_at_the_player(false);
        game.is_in_a_run();
        game.frame();
        assert_eq!(
            (
                game.state().stage_frames,
                game.times_the_background_stepped()
            ),
            (1, 1),
            "a stage built fresh did not step its background on the update it began",
        );

        let before = game.times_the_background_stepped();
        game.moves_on_to_the_next_stage();
        let image = game.image();
        game.frames_until("the next stage built", FRAMES_FOR_A_SCREEN, || {
            !image.the_stage_is_being_built() && image.the_stage_before_fades_out()
        });
        game.frames_until("the next stage begun", FRAMES_FOR_A_SCREEN, || {
            !image.the_stage_before_fades_out()
        });
        let stepped = game.times_the_background_stepped();
        assert_eq!(
            (game.state().stage, game.state().stage_frames, stepped),
            (FIRST_STAGE + 1, 1, before),
            "the background stepped its script over the fade, or the stage is not on its first update",
        );
        game.frame();
        assert_eq!(
            game.times_the_background_stepped(),
            stepped + 1,
            "the background of a stage begun out of a fade did not step on the update after the stage began",
        );
    });
}

/// **A hit of th11's puts a picture of the frame into the trace**, taken out of the game's Direct3D 9 device after
/// its own drawing has ended the scene: a 32-bit BMP, which is what `orb-real picture` writes out.
#[test]
fn a_hit_of_th11_puts_a_picture_of_the_frame_into_the_trace() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-a-hit-puts-a-picture-into-the-trace",
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

/// **What a frame of th11 spent is in the trace, frame by frame**: the spans the pacing marks a frame by, the
/// game's update among them, and orb's own phases inside them — the rules' steering among those, on a run the
/// rules play, and the reading of the field it steers by inside that.
#[test]
fn what_a_frame_of_th11_spent_is_in_the_trace_the_rules_steering_with_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-what-a-frame-spent-is-in-the-trace",
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
