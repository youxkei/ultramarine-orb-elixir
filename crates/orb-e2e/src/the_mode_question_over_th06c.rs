//! **Orb's own screens drawn through a game's own Direct3D 11**: the mode question over th06c's title,
//! read off the screen the way it is read for th06 and th07.
//!
//! What is under test is the whole of the drawing for a game that has no Direct3D 8 device — the swapchain
//! found among the objects the game keeps, the canvas made over its back buffer, the shaders compiled, the
//! glyphs uploaded as textures and one quad drawn per line. `overlay::Backend` is what makes everything
//! above that the same code as th06's, so what these say is that the backend under it answers: a line
//! reaching the screen here reached it through `canvas::Canvas`.
//!
//! **And answered at a keyboard**, which is the other half and was the harder one: orb notices the decide
//! for this game through `runtime::keyboard_state`, the read it *answers* rather than stands in front of,
//! so a person's keys have to be read back out of the snapshot before `held_back` can take the decide out
//! of them — see `Game::word_in`. The two tests at the end of this file press keys the way somebody at the
//! keyboard does, and the recipe-driven ones above them read the screen with the question standing.

use crate::fake::th06c::Fake;
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames the walk to the question is given. The recipe's own bounds are longer than this between
/// them; what this is is the wall the test puts up rather than sitting in a loop.
const FRAMES: u32 = 4000;

/// **The walk to the title's Start item and orb's question over it, and no answer after it** — the same
/// steps `a_pointdevice_run_of_th06c.rs` walks, whose own doc has why each is shaped so. It ends at the
/// question on purpose: a recipe with nothing left to do presses nothing, so the question stands while the
/// screen is read.
const TO_THE_MODE_QUESTION: &str = "\
- wait_until: { screen: 1 }
  give_up_after: 7200
- press: [decide]
  frames: 6
- wait_until: { screen: 2 }
  give_up_after: 36000
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
  until: { question: mode }
  give_up_after: 1800
";

/// A launch walked to the question, with an overlay of orb's over it.
///
/// # Panics
/// If the overlay does not come up or the question never goes up, since every assertion below is about what
/// it drew.
fn asking(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(TO_THE_MODE_QUESTION.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    let log = game.log();
    // **Both lines, because either alone is half the drawing.** The canvas is what says the swapchain was
    // found and the shaders compiled; the overlay is what says a font was loaded and a white texel uploaded
    // through it. A launch of this game had neither before.
    game.frames_until("a canvas over this game's swapchain", FRAMES, || {
        log.said("canvas: drawing through this game's own Direct3D 11, over 640x480")
    });
    assert!(
        log.said("overlay: ready"),
        "the canvas came up and the overlay did not:\n  {}",
        log.lines().join("\n  ")
    );
    game.frames_until("the mode question", FRAMES, || {
        log.said("menu: Run is under the cursor, asking which mode")
    });
    game
}

/// Which mode the question would answer with — the one drawn in `SELECTED`.
///
/// # Panics
/// If the two are not both on the screen with exactly one of them lit, since a question showing anything
/// else is not one anybody could answer.
fn under_the_cursor(game: &Fake) -> Mode {
    // One frame for both, so that the two are read off the same screen rather than off two.
    assert!(
        !game
            .a_frame_that_says(Mode::Pointdevice.name(LANGUAGE))
            .is_empty(),
        "no frame of the question drew a mode at all:\n  {}",
        game.log().lines().join("\n  ")
    );
    let mut on = None;
    for mode in orb_core::mode::CHOICES {
        let text = mode.name(LANGUAGE);
        let drawn = game.says(text);
        assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
        if drawn[0].color == SELECTED {
            assert!(on.replace(mode).is_none(), "both modes are lit");
        } else {
            assert_eq!(drawn[0].color, NORMAL, "{text} is neither lit nor not");
        }
    }
    on.expect("neither mode is lit")
}

/// **The question is on the screen of a game that draws through Direct3D 11**: its own title, both modes
/// under it, and the one orb is in lit.
///
/// Which is the same assertion `the_mode_question_over_th07.rs` makes of a game with a Direct3D 8 device,
/// and it holds here through a backend that has no such device at all.
#[test]
fn the_mode_question_of_th06c_is_drawn_through_the_games_own_direct3d_11() {
    in_its_own_process(|| {
        let game = asking("th06c-question-drawn");
        assert!(
            !game
                .a_frame_that_says(title(Menu::Run, LANGUAGE))
                .is_empty(),
            "the question's own title is not on the screen:\n  {}",
            game.log().lines().join("\n  ")
        );
        // The cursor starts on the mode orb is already in, which for a launch of this game is Pointdevice.
        assert_eq!(
            under_the_cursor(&game),
            Mode::Pointdevice,
            "the cursor is not on the mode orb is in",
        );
    });
}

/// **And the drawing is the canvas's own**, which is the half no log line says: the quads went through
/// textures the canvas handed out, and the viewport each was drawn under is the overlay's own 640x480.
///
/// That viewport is what a canvas turning pixels into clip space against anything else would show as — the
/// lines landing somewhere the Direct3D 8 overlay does not put them — and the swapchain's own buffer
/// measuring something else does not come into it: the viewport is what maps the one onto the other.
#[test]
fn every_quad_of_th06cs_question_goes_through_the_canvas_over_its_swapchain() {
    in_its_own_process(|| {
        let game = asking("th06c-question-canvas");
        // One frame of the question and nothing before it, so that what is counted is a frame's own quads.
        assert!(
            !game
                .a_frame_that_says(title(Menu::Run, LANGUAGE))
                .is_empty(),
            "the question's own title is not on the screen:\n  {}",
            game.log().lines().join("\n  ")
        );
        let drawn = game.drawn();
        assert!(!drawn.quads.is_empty(), "the question drew no quads at all");
        assert!(
            drawn
                .viewports
                .iter()
                .all(|viewport| (viewport.width, viewport.height) == (640, 480)),
            "a quad was drawn under a viewport that is not the overlay's own output: {:?}",
            drawn.viewports,
        );
        // **One viewport per quad**, which is what the canvas setting its own state on every draw means: a
        // count short of that is a quad drawn under whatever the game's renderer had left set.
        assert_eq!(
            drawn.viewports.len(),
            drawn.quads.len(),
            "the canvas did not set its viewport for every quad it drew",
        );
        // And the swapchain orb asked about is one it holds no reference to: the ask that said the object is
        // a swapchain added one, and orb gave it straight back.
        assert_eq!(
            game.swapchain_references_given_back(),
            1,
            "orb did not give back the reference its ask about the swapchain added",
        );
    });
}

/// **The state the game's context was in is the state it is in after orb has drawn.**
///
/// Which is the one thing a black window is: Direct3D 11 has no state block, so a drawing that sets a
/// render target, a blend state, a topology and two shaders and puts none of them back leaves the game
/// drawing the rest of its frame under orb's — and a launch of the real th06c showed exactly that, a black
/// window with the run advancing behind it. See `canvas::Saved`.
///
/// **Read whole rather than field by field**, so that a piece added to the drawing is a piece this covers
/// without being edited — `orb_sim::State` is the set, and the comparison is of the two together.
#[test]
fn th06cs_own_drawing_state_is_what_it_was_before_orb_drew_over_it() {
    in_its_own_process(|| {
        let game = asking("th06c-question-state");
        let sim = game.sim();
        // The state this game's own renderer left the context in — nothing like the one orb draws under,
        // so that a restore putting back the wrong piece shows as a difference rather than as a match.
        let before = orb_sim::State {
            targets: vec![0x7101],
            depth_view: 0x7102,
            blend: 0x7103,
            blend_factor: [0.25, 0.5, 0.75, 1.0],
            blend_mask: 0xf0,
            depth_stencil: 0x7104,
            stencil_reference: 7,
            rasterizer: 0x7105,
            viewports: vec![orb_api::d3d11::Viewport {
                top_left_x: 8.0,
                top_left_y: 16.0,
                width: 320.0,
                height: 240.0,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
            layout: 0x7106,
            vertex_buffer: 0x7107,
            stride: 48,
            offset: 16,
            topology: 4,
            vertex_shader: 0x7108,
            pixel_shader: 0x7109,
            resource_view: 0x710a,
            sampler: 0x710b,
        };
        sim.drawing().the_context_is_in(before.clone());

        // A frame of the question, which is several quads of orb's through that same context.
        assert!(
            !game
                .a_frame_that_says(title(Menu::Run, LANGUAGE))
                .is_empty(),
            "the question was not drawn, so nothing changed the state:\n  {}",
            game.log().lines().join("\n  ")
        );

        assert_eq!(
            sim.drawing().canvas_state(),
            before,
            "orb's drawing left this game's context in a state it did not find it in",
        );
    });
}

// ---- and the same question met at a keyboard ----------------------------------

/// A launch with no recipe at all, walked to the title menu with an overlay of orb's over it.
///
/// # Panics
/// If the overlay does not come up or the title is never reached.
fn at_the_title(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.frames_until("a canvas over this game's swapchain", FRAMES, || {
        log.said("canvas: drawing through this game's own Direct3D 11, over 640x480")
    });
    game.frames_until("the title menu", FRAMES, || {
        game.the_title_is_up_and_takes_a_press()
    });
    game
}

/// How many presses the question is given to come up. A press is two frames and the title ignores one for
/// its first twenty, so this is well past what a screen that is taking presses needs.
const PRESSES: u32 = 30;

/// The question put up by a press, and not by a recipe.
///
/// **The loop is written out rather than `press_until`'s**, because what a failure here has to say is what
/// orb decided: the lines about the cursor, the screen and the mode are the whole diagnosis, and a panic
/// that names only the key leaves the next reader pressing keys by hand.
///
/// # Panics
/// If the question does not come up, or is not on the screen afterwards.
fn ask_with_a_key(game: &Fake, key: u8) {
    let log = game.log();
    let from = log.written();
    let asked = |log: &orb_sim::Log| {
        log.said_since(from, "menu: Run is under the cursor, asking which mode")
    };
    for _ in 0..PRESSES {
        game.press(key);
        if asked(log) {
            break;
        }
    }
    assert!(
        asked(log),
        "{PRESSES} press(es) of {key:#04x} raised no question; what orb said of the screen:\n  {}",
        said_of_the_menus(game).join("\n  ")
    );
    assert!(
        !game
            .a_frame_that_says(title(Menu::Run, LANGUAGE))
            .is_empty(),
        "the press did not put the question on the screen:\n  {}",
        said_of_the_menus(game).join("\n  ")
    );
    // And past the frames the question reads nothing over, which is what the press it went up on is held
    // off by: a direction pressed inside those is one nothing moved on. See `READS_KEYS_AFTER`, and th07's
    // own question test, which waits the same out.
    game.frames(READS_KEYS_AFTER);
}

/// The lines about orb's own menus and the screen under them, which is what a question that did not come
/// up is diagnosed from — see `runtime`'s own line about the item under the cursor and whether the screen
/// will take a press.
fn said_of_the_menus(game: &Fake) -> Vec<String> {
    game.log()
        .lines()
        .into_iter()
        .filter(|line| {
            line.contains("menu:") || line.contains("mode:") || line.contains("front end:")
        })
        .collect()
}

/// How many times orb has reported an outcome for this question — a mode chosen, or neither.
///
/// Counted rather than looked for, for the reason th07's own question test gives: a log line once written
/// is written for good, so a condition that waits for one to appear is already true the second time round.
fn outcomes(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("mode: answered on the") || line.contains("mode: not chosen on the")
        })
        .count()
}

/// **One press at the title puts the question up rather than starting a run** — which is the whole of what
/// `Game::word_in` buys, and what four launches of the real game did not do: every one of them pressed at
/// the title and every one walked into the game's own item with `menu:` never in the log.
///
/// Both keys this game's own menus decide with, which is why this asks twice: the question stands over the
/// game's screen, so a key that chooses an item there has to choose here.
#[test]
fn a_key_pressed_at_th06cs_title_puts_orbs_question_up_instead_of_starting_a_run() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-question-pressed");
        for key in [keys::Z, keys::RETURN] {
            ask_with_a_key(&game, key);
            assert_eq!(
                under_the_cursor(&game),
                Mode::Pointdevice,
                "the cursor is not on the mode orb is in",
            );
            // Answered, and the mode it was on is the mode orb reports.
            let before = outcomes(&game);
            game.press_until(key, "the question answered", || outcomes(&game) > before);
            assert!(
                game.log().said("mode: pointdevice, was pointdevice"),
                "the mode is not what was chosen:\n  {}",
                game.log().lines().join("\n  ")
            );
            // And the press orb held back, handed on to the game's own read: the item it was asked about is
            // the one the game acts on, which walks the title off its own screen. So the second time round
            // is given its title back rather than waiting for a run to be played out and the front end
            // rebuilt — see `Fake::puts_its_title_up`, and th07's own question test, which says the same.
            //
            // **A frame before the title is put up**, which is where that press orb handed on is spent: the
            // title acts on a decide, so landing it on the frame *before* the title is back walks nothing.
            game.frame();
            game.puts_its_title_up();
            game.one_frame();
            game.one_frame_to_drain_the_log();
        }
    });
}

/// And the cursor moves, so somebody in Pointdevice mode reaches normal mode with one step — which is also
/// what says the two lines are drawn in two colours rather than in one.
#[test]
fn the_cursor_of_th06cs_question_moves_at_a_keyboard_and_the_mode_it_lands_on_is_chosen() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-question-moved");
        ask_with_a_key(&game, keys::Z);
        game.press(keys::DOWN);
        assert_eq!(under_the_cursor(&game), Mode::Normal);
        let before = outcomes(&game);
        game.press_until(keys::Z, "the question answered", || {
            outcomes(&game) > before
        });
        assert!(
            game.log().said("mode: normal, was pointdevice"),
            "normal mode was not chosen:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
