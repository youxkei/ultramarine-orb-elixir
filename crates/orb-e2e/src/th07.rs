//! **The frame orb paces th07 with, and what it reads of one.**
//!
//! A th07 laid out from what has been read of `th07.exe`, orb attached to `Th07`, and orb's own frame
//! loop in place of the game's own frame — which is what a launch there is. What the e2e tests below ask
//! is that the loop composes th07's frame rather than th06's: the update before the draw, the queue of
//! quads emptied before the drawing fills it and drawn before the scene ends, the fog a stage left on put
//! out. Beside that, one field at a time: the scene, the stage, the run's own numbers, the pad, the
//! letterbox, the score file the mode forks.
//!
//! **The mode itself is `a_pointdevice_run_of_th07.rs`**, which plays a run of the same laid-out game
//! through the whole of it — the question, a chapter, a death, the retry. The launches here have no run in
//! them, so nothing of a chapter is reached from any of them.
//!
//! **The queue is what makes these more than a smoke test.** orb's first loop in that frame took the game
//! down on its first frame, writing a quad through a null pointer, and every address it used was right —
//! see [docs/adr/0004](../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) and
//! [docs/adr/0017](../../../docs/adr/0017-the-frame-loop-has-a-seam-either-side-of-the-draw-chain.md).
//! So the laid-out game's draw chain queues a quad the way a real one does, through the field the game's
//! own frame resets, and a loop that leaves that reset out faults here as it faulted there.
//!
//! What these cannot say is that the addresses *are* right: a laid-out th07 is written from the same
//! constants `Th07` reads, so a wrong one is wrong on both sides at once. Only the real image says.

use crate::fake::th07::Fake;
use crate::fake::{
    DRAW, Display, Launched, PRESENT, Presented, SOUND, UPDATE, Work, in_its_own_process,
};
use orb_config::LogLevel;
use orb_config::Screen;
use orb_core::frame;
use orb_core::overlay::{FONT_HEIGHT, MARK_FONT_HEIGHT};
use orb_core::window::letterbox;
use orb_sim::Asked;

/// What the game's own frame is declared to take here: about what a real run's report line shows for the
/// drawing, so that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long the run is: a few seconds of it, which is past the frames orb spends trying to build an
/// overlay and well into the frames where nothing is happening.
const FRAMES: u32 = 5 * frame::LOGIC_HZ;

/// The family th07 draws its own text in, which is the one orb's screens are to be drawn in: the name it
/// hands `CreateFontA` at 0x4322e7, at 0x497cbc in Shift-JIS.
const FACE: &str = "ＭＳ ゴシック";

/// A card of the game's own and the name its record carries, for the run that has to have counted something
/// before its ranking is worth building: th07 counts an attempt where a card starts.
const A_CARD: usize = 96;
const A_CARDS_NAME: &str = "幽夢「猫の見た夢」";

#[test]
fn orb_paces_a_launch_of_th07_with_no_run_in_it() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th07-paced", Work::wandering(WORK_US));

        // The frame is orb's own, in orb's order and not th07's: the update first — which is the frame
        // of input lag removed — the sounds where the game's own frame hands them over, then the drawing,
        // then the frame handed to the display.
        game.forget_asked();
        game.frame();
        assert_eq!(game.asked(), vec![UPDATE, SOUND, DRAW, PRESENT]);

        game.frames(FRAMES);
        let log = game.log().lines().join("\n");

        // orb got in, and got in through both of the patches `Th07::hooks` asks for: an update hook and
        // a draw hook reached is what the order above is evidence of, and the state line is orb's own
        // per-frame work having run at all.
        assert!(
            log.contains("attached to a game laid out in this process"),
            "orb did not attach:\n  {log}"
        );

        // And an overlay all the same, in a directory with no `font.ttf` in it: th07 ships none — its
        // fonts are inside `th07.dat` — and orb's own screens are drawn with the face the machine has
        // rather than not drawn at all. See `docs/adr/0019`.
        assert!(
            log.contains("overlay: no font beside the game; using the machine's own face"),
            "orb did not fall back to a face of the machine's:\n  {log}"
        );
        assert!(
            log.contains("overlay: ready"),
            "orb built no overlay in a directory with no font in it:\n  {log}"
        );

        // **And the face it asked the machine for is the one this game asks for itself, asked for the way the
        // game asks**: th07 ships no font and makes its own text out of a family it names to GDI —
        // `CreateFontA` at 0x4322e7 with `ＭＳ ゴシック` and the weight 0x2bc, which is bold — so asking for
        // anything else is orb's screens in a face the game's own text is not in.
        //
        // **And the height is the game's own number**: that call asks for `size * 2 - 2` as a *positive* height,
        // which is the whole cell rather than the em, and the call bakes into a bitmap twice as wide and
        // `size * 2 + 6` tall before the result is drawn back down to size. Glyphs baked at twice the size and
        // scaled down are not the same glyphs as ones baked at the size — a th07 session said the text did not
        // look like the game's — so what orb asks for is the same number and its own drawing does the same
        // halving. Both of the overlay's sizes, the status lines' and the mark's.
        let asked = game.sim().text().machine_faces_asked_for();
        assert_eq!(
            asked,
            vec![
                Asked {
                    family: FACE.to_owned(),
                    bold: true,
                    height: FONT_HEIGHT * 2 - 2,
                },
                Asked {
                    family: FACE.to_owned(),
                    bold: true,
                    height: MARK_FONT_HEIGHT * 2 - 2,
                },
            ],
            "orb did not ask the machine for the face th07 asks for, the way it asks for it",
        );
        // And the log says which one came back, a family a machine does not have being a substituted face
        // and the glyphs then not the game's.
        assert!(
            log.contains("overlay: the machine's own face is"),
            "the log does not say which face the machine gave:\n  {log}"
        );

        // And nothing of what a run gets, there being no run in this launch: the scene never leaves the front
        // end's, so nothing is chaptered, nothing dies and nothing is offered. What a run of it gets is
        // `a_pointdevice_run_of_th07.rs`, which is the same game laid out with one in progress.
        for absent in ["chapter ", "retry", "resume: stage", "died in", "wash"] {
            assert!(
                !log.contains(absent),
                "orb did {absent:?} to a launch with no run in it:\n  {log}"
            );
        }

        // And it did not take the game down, which is the one thing an injected DLL must not do to
        // somebody's play session: a method that panicked would be a `panic:` line here and a game gone
        // in a real process.
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

/// The two play times その他の記録 is seeded with, one per file: hours, minutes and seconds, the way the game
/// keeps them. Different in every field, so a read that landed in the wrong file cannot come out right.
const THE_GAMES_PLAY_TIME: (u32, u32, u32) = (1, 42, 32);
const THE_MODES_PLAY_TIME: (u32, u32, u32) = (7, 13, 51);
/// And 総起動時間 in each, which the same read replaces and which belongs to neither mode: the game has been up
/// for as long as it has been up whichever of them a sitting is spent in.
const THE_GAMES_RUNNING_TIME: (u32, u32, u32) = (3, 8, 19);
const THE_MODES_RUNNING_TIME: (u32, u32, u32) = (11, 27, 4);

/// A span of time the game counted while no mode had been chosen: long enough to carry into minutes, so that a
/// test cannot pass on seconds alone.
const A_SPAN_MS: u32 = 95_000;

/// That span added to a time the game keeps as hours, minutes and seconds.
fn plus(time: (u32, u32, u32), milliseconds: u32) -> (u32, u32, u32) {
    let (hours, minutes, seconds) = time;
    let total = ((hours * 60 + minutes) * 60 + seconds) * 1000 + milliseconds;
    (total / 3_600_000, total / 60_000 % 60, total / 1000 % 60)
}

/// **Time spent before a mode was chosen belongs to both files**, which is what the title screen and the replay
/// menu are: nobody has said yet whether the runs of this sitting are ones a rewind can be had in. So the span
/// counted there is carried onto whichever file the sitting turns out to write.
///
/// Left alone it is worse than lost. The block the game keeps その他の記録 in is replaced wholesale when the
/// mode's own file is read over it — see `Th07::read_what_the_modes_file_records` — so the span counted before
/// that read is dropped, and before orb read that file at all it was written into the *other* mode's file.
#[test]
fn the_time_of_th07_before_a_mode_was_chosen_is_counted_in_the_modes_own_file() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-play-time",
            Work::wandering(WORK_US),
        );
        game.image().its_score_file_records_times(
            "score.dat",
            THE_GAMES_RUNNING_TIME,
            THE_GAMES_PLAY_TIME,
        );
        game.image().its_score_file_records_times(
            "pointdevice_score.dat",
            THE_MODES_RUNNING_TIME,
            THE_MODES_PLAY_TIME,
        );

        // A span of each counted with the front end up, which is where no mode has been chosen. The frame either
        // side of it is what orb sees it in: what a span *is* is the block having gone up since the last look.
        game.frame();
        game.image().counts_a_play_span(A_SPAN_MS);
        game.image().counts_a_running_span(A_SPAN_MS);
        game.frame();

        // And the mode settled, which is the read that lays that file's own records over the block.
        game.reads_what_its_menus_offer();
        assert_eq!(
            game.image().the_times_it_records(),
            (
                plus(THE_MODES_RUNNING_TIME, A_SPAN_MS),
                plus(THE_MODES_PLAY_TIME, A_SPAN_MS),
            ),
            "the time counted before a mode was chosen is not in the mode's own file",
        );
    });
}

#[test]
fn the_score_file_of_a_run_that_can_be_rewound_is_orbs_own() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-score-file",
            Work::wandering(WORK_US),
        );

        // The front end's own reads and the write on the way out, which is what a launch there really
        // does — and every one of them orb's file, this launch being in the mode a run of it can be rewound
        // in. A rewound run is not a run anybody played, so its score does not belong in the ranking the
        // game keeps: see `orb_core::score`, and `a_pointdevice_run_of_th07.rs` for the mode itself.
        for write in [false, false, true] {
            assert_eq!(
                game.opens_its_score_file(write),
                "pointdevice_score.dat",
                "an open of th07's score file landed in the game's own",
            );
        }

        // **With one exception, and it is a read.** What the front end's own menus offer — which practice
        // stages are open, whether the Extra is — is not a record of a run, so that one read is answered out
        // of the game's own file whichever mode orb is in: a first Pointdevice session answering it out of a
        // new file of its own would lock every stage a `score.dat` had already earned. orb brackets exactly
        // that callback, which in this game is 0x45c4c8 — see `Th07::hooks`.
        // その他の記録 in each of the two files, and not the same in both: what a play time is is time spent in
        // a mode, so which file a read of it lands in is the whole question below.
        game.image().its_score_file_records_times(
            "score.dat",
            THE_GAMES_RUNNING_TIME,
            THE_GAMES_PLAY_TIME,
        );
        game.image().its_score_file_records_times(
            "pointdevice_score.dat",
            THE_MODES_RUNNING_TIME,
            THE_MODES_PLAY_TIME,
        );

        assert_eq!(
            game.reads_what_its_menus_offer(),
            "score.dat",
            "the front end's own read did not land in the game's file",
        );

        // **And what that read carries of the mode's own is read again out of the mode's own file**, which is
        // the other half of the exception: that read is one call, and what it fills is the unlocks *and* a
        // record of runs — the practice scores a stage is offered with, and this game's card records. Left as
        // the game's own file has them, a session that never answered the mode question again shows the other
        // mode's: a run given up back to the practice stage select showed a legacy score for the stage it had
        // just been played in, and the ranking screen counted nothing of the session.
        assert!(
            game.log()
                .said("score: what the mode's own file records read out of it again"),
            "the mode's own records were left as the game's own file had them:\n  {}",
            game.log().lines().join("\n  ")
        );

        // **And その他の記録 is one of those records**, which is the screen headed by the two totals: a play time
        // is time spent in a mode, and the counts under it are counts of that mode's runs. Left as the game's
        // own file has them, the block the game writes carries the other mode's totals into this mode's file —
        // and it is written whole, out of the block, by the game's own save.
        assert_eq!(
            game.image().the_times_it_records(),
            (THE_MODES_RUNNING_TIME, THE_MODES_PLAY_TIME),
            "その他の記録 came back as the game's own file has them, not the mode's",
        );

        // And the game's file is never written in this mode: the one write th07 makes of it is the screen a
        // run ends on going down, which is outside that bracket and so follows the mode like every other open.
        let log = game.log().lines().join("\n");
        assert!(
            log.contains("mode: pointdevice to start with"),
            "orb started in normal mode in a game whose runs it can rewind:\n  {log}"
        );
    });
}

/// **No ranking is built for a run of th07 that ended, and no update of the game's own is asked for.**
///
/// Which is a finding about this game rather than a preference: the screen orb used to build and take down here
/// writes nothing at either phase it is built with — 0xa and 0x12 both skip the one write in the exe, which is
/// gated on 0x13 — so every frame spent driving it was spent for nothing, and the file was left as it was. What
/// carries a session's counts instead is the game's own save, and `the_cards_a_run_of_th07_counted.rs` is what
/// says the counts reach the file through it.
///
/// **And a launch is why the frames matter as much as the file.** orb's pump faulted inside one of those updates,
/// `th07.exe+0x50db1` reading a vm the screen it had just asked to leave no longer had, with orb's own frames in
/// the stack between the game's loop and its calc chain. A run that ends asking the game for no update at all
/// cannot fault there.
///
/// **Nor is a save of the game's own asked for here**, which a second launch taught at the same address: that
/// call is the score screen's added callback and its deleted one behind it, and the deleted one frees the anm
/// files four of the manager's slots hold. What carries the counts instead is the save the game makes for itself
/// — see `the_cards_a_run_of_th07_counted.rs`.
#[test]
fn no_ranking_is_built_for_a_run_of_th07_that_ended() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-no-ranking",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // A card attempted, so the run has counted something: a run with nothing counted is one there would be
        // nothing to write for either way.
        game.the_boss_arrives();
        game.a_card_starts(A_CARD, A_CARDS_NAME);
        game.frames(30);
        game.gives_the_run_up();

        assert!(
            log.said("score: this game writes its own file"),
            "the run's end was not left to the save the game makes itself:\n  {}",
            log.lines().join("\n  ")
        );
        for said in [
            "score: the ranking is up",
            "score: asked it to leave",
            "score: update 1 answered",
        ] {
            assert!(
                !log.said(said),
                "the ranking screen was driven for a run that ended — {said}:\n  {}",
                log.lines().join("\n  ")
            );
        }
    });
}

#[test]
fn the_queue_of_quads_is_emptied_before_the_drawing_and_drawn_before_the_scene_ends() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-queue",
            Work::wandering(WORK_US),
        );

        game.frames(FRAMES);

        // Where the next quad goes is one quad past the start of the buffer, however many frames have
        // run: emptied at the top of every frame's drawing, filled by the draw chain, and drawn before
        // the scene ended. A loop that emptied it and never drew it would leave the pointer FRAMES quads
        // along and eventually walk off the end of a two-thousand-quad buffer; one that never emptied it
        // writes to address zero on its first frame, which is what the real th07 did.
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
        // And the game's own count of how many times it drew that queue, which is the frames: one
        // drawing a frame, and orb's loop skips none.
        assert_eq!(image.queue_draws(), FRAMES);
    });
}

/// A client wider than the game's own ratio, so that the rectangle a frame reaches the device in is plainly
/// not the client: th07 renders 640x480, and what 4:3 gets in 16:9 is bars down the sides.
const WIDER_THAN_THE_GAME: Screen = Screen::Window {
    width: 1280,
    height: 720,
};

#[test]
fn the_frames_reach_the_device_inside_a_letterbox_of_the_games_own_shape() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-letterbox",
            Work::wandering(WORK_US),
            |config| config.screen = WIDER_THAN_THE_GAME,
        );
        // The window first and the device after it, which is the order a launch has: orb redirects the
        // `Present` slot from the game's own device setup, and what it works the rectangle out from is the
        // client area of a window that exists.
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
        // And the whole back buffer into it. th07's own present asks for none of its surface in
        // particular — four nulls at 0x4345df — so what orb has to keep is that: a part of the surface
        // scaled into a rectangle worked out for the whole of it would be the wrong part, wrongly.
        assert_eq!(presents[0].source, None);
    });
}

/// And a driver that will not stretch: the game's own call again, said once, and the run still presented.
///
/// The fallback is orb's and not a game's, and `the_letterbox_presented_into.rs` is where it is asked of th06.
/// What asking it of this game adds is the ask th07 makes on the way back — four nulls at 0x4345df, so the call
/// that goes through is one with neither rectangle in it, where th06's e2e tests hand over a source to see kept.
#[test]
fn a_driver_that_refuses_to_stretch_is_fallen_back_from_over_th07_and_said_once() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-letterbox-refused",
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

        // Said once and not once a frame: a run at sixty frames a second would otherwise write a line for
        // every one of them, which is a log nobody can read the rest of.
        let said = || {
            log.lines()
                .iter()
                .filter(|line| line.contains("Present into a letterbox failed"))
                .count()
        };
        assert_eq!(said(), 1);
        game.frames(30);
        assert_eq!(
            said(),
            1,
            "orb said the driver refused once a frame rather than once",
        );

        // And every one of those frames still reached the device, which is the whole point of falling back.
        assert!(
            game.presented().len() > 30,
            "frames stopped being presented once the driver refused",
        );
    });
}

/// A pad on the host, as an e2e test declares one: buttons enough for the game's own mapping to name any
/// of them, and an axis over a whole 16-bit travel.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// The bits of th07's own input word this asks about: what the game's default mapping calls shoot, and
/// the two the stick and the hat decide. Read off the pad read at 0x430457 and 0x4305ac — see
/// `Th07::pad_word`.
const SHOOT: u16 = 0x1;
const UP: u16 = 0x10;
const DOWN: u16 = 0x20;
/// And which button shoot is with nobody's `th07.cfg` in the directory, which is the game's own default:
/// the first. `ANOTHER_BUTTON` is one the default mapping names nothing, so a key config screen finding it
/// held is finding the pad and not the mapping.
const SHOOT_BUTTON: u32 = 0;
const ANOTHER_BUTTON: u32 = 7;

fn centred() -> u32 {
    (PAD_TRAVEL.0 + PAD_TRAVEL.1) / 2
}

#[test]
fn a_pad_the_game_has_no_device_for_is_in_the_word_the_game_reads() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-pad",
            Work::wandering(WORK_US),
        );
        // A pad the game itself finds nothing of: th07 looks for a DirectInput device of its own and
        // falls back to one winmm joystick, and orb reads every pad the machine has instead — which is
        // the whole of why a pad works here at all.
        game.sim()
            .joystick()
            .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);

        // Shoot, on the button the game's own mapping calls shoot. The sample is taken on a thread of
        // orb's own, so what an e2e test waits for is that thread — every frame of the waiting is an
        // ordinary frame of the run.
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.frames_until_a_thread("the pad's button reaching the game's word", || {
            game.word_the_input_read_answered() & SHOOT != 0
        });

        // Down on the stick, past the dead zone the pad's own travel makes: a quarter of it either side of
        // the middle, which is what the game's own read does with the same two numbers.
        game.sim()
            .joystick()
            .pushes(0, centred() + PAD_TRAVEL.1 / 4 + 1);
        game.frames_until_a_thread("the stick reaching the game's word", || {
            game.word_the_input_read_answered() == DOWN
        });

        // And up on the hat, which is where a d-pad reports and where an XInput pad's arrives: the game's
        // own read has no look at that field, so this is `dpad_moves` — on by default — doing what it says.
        game.sim().joystick().pushes(0, centred());
        game.sim().joystick().pushes_the_hat(0);
        game.frames_until_a_thread("the hat reaching the game's word", || {
            game.word_the_input_read_answered() == UP
        });
    });
}

#[test]
fn a_pad_the_game_has_no_device_for_is_in_the_buttons_its_key_config_reads() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-key-config",
            Work::wandering(WORK_US),
        );
        game.sim()
            .joystick()
            .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
        // Two buttons: the one the game's default mapping calls shoot, and one it names nothing. The first
        // is what says the sample has arrived — the word is the only thing an e2e test can wait on, the
        // sampling being a thread's — and the second is the whole subject, a button no mapping can account
        // for reaching the screen a mapping is made on.
        game.sim()
            .joystick()
            .pushes((1 << SHOOT_BUTTON) | (1 << ANOTHER_BUTTON), centred());
        game.frames_until_a_thread("the pad being sampled", || {
            game.word_the_input_read_answered() & SHOOT != 0
        });

        // What the key config screen reads, which is the pad's buttons *by number* and not the word: a
        // mapping cannot be made from a mapped word. The game's own read fills nothing here — th07 finds
        // no device of its own on the machine this was found on — so every byte set in it is orb's.
        let array = game.reads_the_pad_buttons();
        assert_eq!(
            array,
            game.image().pad_buttons(),
            "the read answered somewhere other than the game's own array",
        );
        for button in [SHOOT_BUTTON, ANOTHER_BUTTON] {
            assert!(
                game.image().pad_button_held(button as usize),
                "button {button} was pushed and the key config screen would not have found it",
            );
        }
        // And nothing else, since nothing else is pushed: that screen assigns the first byte it finds set,
        // so one left over from an earlier read would be the button it assigned.
        assert!(
            !game.image().pad_button_held(1),
            "a button nobody pushed reads as held",
        );
    });
}

#[test]
fn the_scene_orb_reads_is_the_one_the_games_own_supervisor_is_in() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-scene",
            Work::wandering(WORK_US),
        );

        // A run in progress, as the game's own transition would have left its supervisor: orb writes the
        // pair into its state line every frame, and that line is what somebody reading a launch's log has
        // to be able to believe.
        let run = game.image().scene_of_a_run();
        game.image().is_in_scene(run, run);
        game.frames(FRAMES);

        let log = game.log().lines().join("\n");
        assert!(
            log.contains(&format!("scene={run}->{run}")),
            "orb's state line does not say the scene the supervisor is in:\n  {log}"
        );
        // And nothing offered over a run with no numbers behind it: the pointer the build allocates is null
        // here, so every one of them reads as nothing — and a retry over that would be a retry to a chapter
        // whose snapshot is of a run that was never in progress. What a run with its numbers in place gets is
        // `a_pointdevice_run_of_th07.rs`.
        for absent in ["retry", "died in"] {
            assert!(
                !log.contains(absent),
                "orb did {absent:?} over a run with no numbers behind it:\n  {log}"
            );
        }
    });
}

/// A stage and a difficulty a laid-out run is on: the fifth stage of six on Lunatic, which is a pair
/// nothing else in these e2e tests would leave lying around.
const STAGE: i32 = 4;
const DIFFICULTY: i32 = 3;

/// And the numbers that run is at, each different from the others so that a line printing one of them in
/// another's place is a failure rather than a pass. The game keeps all four as floats and the lives with a
/// fraction is what a run that has just taken an extend looks like.
const LIVES: f32 = 3.5;
const BOMBS: f32 = 2.0;
const DEATHS: f32 = 1.0;
const POWER: f32 = 62.5;

/// And the state the game's own generator is in, which is a word of its own rather than one of the run's
/// numbers — a run resumed from a chapter is reproduced by putting this back.
const SEED: u16 = 0x1234;

/// How long the stage has been running, which the game counts in the manager and not in the run's numbers:
/// a number no other field of this test could be mistaken for.
const STAGE_FRAMES: u32 = 1234;

/// And how many enemies are alive, which their own manager counts.
const ENEMIES: i32 = 17;

/// And how long the stage's own script has run, which is the clock a chapter's boundaries are written in —
/// a different number from the stage's frames, because the two are different clocks.
const SCRIPT_FRAMES: i32 = 987;

#[test]
fn the_stage_the_difficulty_and_the_runs_own_numbers_are_read_out_of_the_game() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-run-read",
            Work::wandering(WORK_US),
        );

        // A run in progress, as the game's own build of one leaves it: the scene, the stage, the difficulty,
        // and the numbers behind the pointer that build allocates.
        game.image().is_playing(STAGE, DIFFICULTY);
        game.image().has_a_runs_numbers();
        game.image().has_lives_and_bombs(LIVES, BOMBS);
        game.image().has_died(DEATHS);
        game.image().shot_is_at_power(POWER);
        game.image().generator_is_at(SEED);
        game.image().has_been_in_the_stage_for(STAGE_FRAMES);
        game.image().enemies_alive(ENEMIES);
        game.image().the_script_has_run_for(SCRIPT_FRAMES);
        game.frames(FRAMES);
        let log = game.log().lines().join("\n");
        assert!(
            log.contains(&format!(
                "stage={STAGE} diff={DIFFICULTY} frames={STAGE_FRAMES} script={SCRIPT_FRAMES}"
            )),
            "orb's state line does not say the stage, the difficulty and its two clocks:\n  {log}"
        );
        // Whole numbers of what the game keeps as floats, and the generator's word beside them: all five in
        // one assertion because the log's line is one place to get any of them wrong, and a run at 3.5 lives
        // has 3.
        assert!(
            log.contains(&format!(
                "seed={SEED:#06x} deaths={} lives={} bombs={} power={} enemies={ENEMIES}",
                DEATHS as i32, LIVES as i8, BOMBS as i8, POWER as u16
            )),
            "orb's state line does not say the run's own numbers:\n  {log}"
        );
    });
}

#[test]
fn the_fog_is_taken_off_to_the_device_every_frame_and_not_only_where_a_stage_left_it_on() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-fog",
            Work::wandering(WORK_US),
        );

        // A stage's background drawing having turned it on, which is where th07's own fog comes from.
        game.image().turns_the_fog_on();
        game.frames(FRAMES);

        assert!(
            !game.image().fog_is_on(),
            "the fog a stage left on was still on after a frame of orb's",
        );
        // And `D3DRS_FOGENABLE` set false on every frame rather than on the one after the stage turned it
        // on. The game's own call reaches the device only where the field says the fog is on, so the frame
        // writes that field first — a frame of orb's that left the write out would take the fog off to the
        // device once in a run and leave every frame after it to whatever the device was last told.
        assert_eq!(
            game.fog_told_to_the_device(),
            FRAMES,
            "the fog was not taken off to the device on every frame",
        );
    });
}
