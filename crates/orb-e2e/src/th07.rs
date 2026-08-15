//! **A game orb paces and does nothing else to.**
//!
//! A 妖々夢 laid out from what has been read of `th07.exe`, orb attached to `Th07`, and orb's own frame
//! loop in place of the game's own frame — which is what a launch there is. What the e2e tests below ask
//! is that the loop composes 妖々夢's frame rather than 紅魔郷's: the update before the draw, the queue of
//! quads emptied before the drawing fills it and drawn before the scene ends, the fog a stage left on put
//! out. And that orb did none of what it does to 紅魔郷 — no chapter, no snapshot, no menu, no mark, no
//! overlay, and its score file its own, each because a method of `Th07` answered nothing rather than a
//! number.
//!
//! **The queue is what makes these more than a smoke test.** orb's first loop in that frame took the game
//! down on its first frame, writing a quad through a null pointer, and every address it used was right —
//! see [docs/adr/0004](../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) and
//! [docs/adr/0017](../../../docs/adr/0017-the-frame-loop-has-a-seam-either-side-of-the-draw-chain.md).
//! So the laid-out game's draw chain queues a quad the way a real one does, through the field the game's
//! own frame resets, and a loop that leaves that reset out faults here as it faulted there.
//!
//! What these cannot say is that the addresses *are* right: a laid-out 妖々夢 is written from the same
//! constants `Th07` reads, so a wrong one is wrong on both sides at once. Only the real image says.

use crate::fake::th07::Fake;
use crate::fake::{DRAW, Display, Launched, PRESENT, SOUND, UPDATE, Work, in_its_own_process};
use orb_config::Screen;
use orb_core::frame;
use orb_core::window::letterbox;

/// What the game's own frame is declared to take here: about what a real run's report line shows for the
/// drawing, so that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long the run is: a few seconds of it, which is past the frames orb spends trying to build an
/// overlay and well into the frames where nothing is happening.
const FRAMES: u32 = 5 * frame::LOGIC_HZ;

#[test]
fn orb_paces_a_game_it_declines_everything_about_a_run_of() {
    in_its_own_process(|| {
        let game = Fake::attach(Display::agreed(120), "th07-paced", Work::wandering(WORK_US));

        // The frame is orb's own, in orb's order and not 妖々夢's: the update first — which is the frame
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

        // No overlay, a 妖々夢 install having no `font.ttf` beside its exe — so nothing orb draws is
        // drawn, which is measured of the real game too.
        assert!(
            log.contains("overlay: unavailable"),
            "orb built an overlay in a directory with no font in it:\n  {log}"
        );

        // And none of what orb does to 紅魔郷. Each of these is a method of `Th07` answering nothing:
        // `read_state` says no run, `run_slot` says no run is kept, `midstage_table` is empty, and
        // `menu_pointed_at` never has anything under the cursor.
        for absent in [
            "chapter ",
            "retry",
            "resume: stage",
            "died in",
            "wash",
            "mode: asking",
        ] {
            assert!(
                !log.contains(absent),
                "orb did {absent:?} to a game it has read no run of:\n  {log}"
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

#[test]
fn the_score_file_a_game_orb_can_rewind_nothing_in_writes_is_the_games_own() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-score-file",
            Work::wandering(WORK_US),
        );

        // The front end's own reads and the write on the way out, which is what a launch there really
        // does — and every one of them the game's own file. orb's is for runs that can be rewound, and
        // nothing here can: a run played in 妖々夢 with orb in it is a run anybody could have played, so
        // it belongs in the ranking the game keeps.
        for write in [false, false, true] {
            assert_eq!(
                game.opens_its_score_file(write),
                "score.dat",
                "an open of 妖々夢's score file landed in orb's",
            );
        }

        // And the mode says so rather than leaving it to be worked out from the file: pointdevice mode is
        // chapters, and a launch that said it was in one it can never be in is a launch whose log means
        // something else.
        let log = game.log().lines().join("\n");
        assert!(
            log.contains("mode: normal to start with"),
            "orb started in pointdevice mode in a game it can rewind nothing in:\n  {log}"
        );
        assert!(
            !log.contains("pointdevice_score.dat"),
            "orb opened a file of its own for a game whose runs are the game's:\n  {log}"
        );
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
        // writes to address zero on its first frame, which is what the real 妖々夢 did.
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
/// not the client: 妖々夢 renders 640x480, and what 4:3 gets in 16:9 is bars down the sides.
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
        // And the whole back buffer into it. 妖々夢's own present asks for none of its surface in
        // particular — four nulls at 0x4345df — so what orb has to keep is that: a part of the surface
        // scaled into a rectangle worked out for the whole of it would be the wrong part, wrongly.
        assert_eq!(presents[0].source, None);
    });
}

/// A pad on the host, as an e2e test declares one: buttons enough for the game's own mapping to name any
/// of them, and an axis over a whole 16-bit travel.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// The bits of 妖々夢's own input word this asks about: what the game's default mapping calls shoot, and
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
        // A pad the game itself finds nothing of: 妖々夢 looks for a DirectInput device of its own and
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
        // mapping cannot be made from a mapped word. The game's own read fills nothing here — 妖々夢 finds
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
fn the_fog_is_taken_off_to_the_device_every_frame_and_not_only_where_a_stage_left_it_on() {
    in_its_own_process(|| {
        let game = Fake::attach(
            Display::agreed(120),
            "th07-the-fog",
            Work::wandering(WORK_US),
        );

        // A stage's background drawing having turned it on, which is where 妖々夢's own fog comes from.
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
