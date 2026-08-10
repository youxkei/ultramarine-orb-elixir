//! **The brush stroke over the count of lives, and the frames either end of a run it has to reach.**
//!
//! What each e2e test holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine and seen on the screen.
//!
//! What the stroke *is* has tests already — `lives_ui.rs`'s `the_stroke_covers_the_count_and_nothing_else`,
//! `the_row_itself_is_left_to_the_game`, `the_stroke_over_the_row_is_a_picture_and_not_a_flat_fill` and the
//! six beside them — and `pointdevice_run.rs` reads `DISABLE` off the screen over a run. What is
//! left, and what these are, is the two edges: the frame a stage transition takes, and the frame after a
//! run has ended.
//!
//! **Every one of them reads the word off the screen.** `DISABLE` is a quad the overlay drew, so the
//! frames are run one at a time with nothing remembered before each — [`Launched::one_frame`] — and what
//! is asserted is which of those frames it was on. Nothing is asked of orb's own bookkeeping: the mark
//! going off is a frame with no such quad on it, which is what somebody watching the screen saw.

use crate::fake::th06::{Fake, PANEL_FRAMES, lives_row, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Scene;
use orb_sim::keys;

/// How long each stage of these runs is, in frames.
///
/// Past the 248 a stage spends settling before its first chapter — a snapshot has to exist for there to
/// be anything to mark — and short of the 400 this game's boss arrives at, so a stage is a stage's start
/// and nothing else. See `fake::th06::BOSS_ARRIVES`.
const STAGE_FRAMES: u32 = 300;

/// How many presses an e2e test makes at one of orb's menus before it gives up on it answering.
///
/// Well past the 24 frames the retry menu holds its keys off for, against the two frames a press is.
/// Counted in presses rather than frames because how long a menu holds them off is the menu's business —
/// the same reason `fake`'s own `press_until` counts them.
const PRESSES: u32 = 30;

/// Whether the mark was on the screen on the frame `reached` first answered true, running the frames one
/// at a time so that what is read is that frame's own screen.
///
/// # Panics
/// After `limit` frames, naming what was being waited for.
fn marked_on_the_frame(game: &Fake, what: &str, limit: u32, reached: impl Fn() -> bool) -> bool {
    for _ in 0..limit {
        game.one_frame();
        if reached() {
            return !game.says("DISABLE").is_empty();
        }
    }
    panic!("{what} did not happen in {limit} frame(s)");
}

/// And the same while a key is being pressed at one of orb's menus, since a menu is answered by
/// somebody pressing rather than by frames going by.
fn marked_on_the_frame_a_press_reaches(
    game: &Fake,
    what: &str,
    key: u8,
    reached: impl Fn() -> bool,
) -> bool {
    for _ in 0..PRESSES {
        game.keyboard().set(key, true);
        game.one_frame();
        game.keyboard().set(key, false);
        if reached() {
            return !game.says("DISABLE").is_empty();
        }
        game.one_frame();
        if reached() {
            return !game.says("DISABLE").is_empty();
        }
    }
    panic!("{what} did not happen in {PRESSES} press(es) of {key:#04x}");
}

/// The mark is drawn on the run rather than on the frame, so a stage transition does not lose it.
///
/// A stage transition leaves the gameplay scene for exactly one frame — `f44096 scene=3 stage=2` and then
/// `f44097 scene=2 stage=3 frames=1` — and that one frame is long, because the game builds the next stage
/// inside it. Every transition of the run went the same way, so the one frame the mark was asked about and
/// said no to was a visible stretch of screen: the count came back for an instant at every stage boundary.
///
/// The frame a chapter is put back on is the same case for a different reason — the update drops what it
/// knows of the frame it froze on, a chapter put back not being a continuation of it. Both are a run's own
/// frames, so what the mark is drawn on is whether the run being tracked is one somebody is playing, taken
/// with the stage's snapshot and dropped when the run is left. Both were seen gone on the screen.
#[test]
fn the_mark_survives_a_stage_transition_and_the_frame_a_chapter_is_put_back_on() {
    in_its_own_process(|| {
        let game = Fake::attach("the-mark-across-a-transition", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.stages_last(STAGE_FRAMES);
        game.in_a_pointdevice_run();

        // The frame the transition is on: the scene is not the gameplay one, and the mark is still there.
        // Which is the claim — the run is what it is drawn on, and this frame is a frame of the run.
        assert!(
            marked_on_the_frame(&game, "the stage transition", STAGE_FRAMES + 60, || {
                game.image().scene() == Scene::Rebuilding
            }),
            "the mark went off on the one frame a stage transition takes, which is a quarter of a \
             second of the count back on the panel",
        );
        // And the stage after it really did come up, or the frame above was not a transition.
        game.frames_until("the stage after the transition", 8, || {
            game.image().scene() == Scene::Playing
        });
        assert_eq!(
            game.state().stage,
            1,
            "the transition built the wrong stage"
        );
        game.frames_until("the second stage's first chapter", 400, || {
            game.log().said("stage 2 chapter 1 (stage start)")
        });

        // The frame a chapter is put back on, which orb reaches by way of its own retry menu: the update
        // that puts it back has dropped what it knew of the frame it froze on, so this is the other frame
        // the mark would go off on if it were drawn on the frame rather than on the run.
        game.hit();
        game.frame();
        assert!(
            game.log().said("died in chapter"),
            "the death was not noticed:\n  {}",
            game.log().lines().join("\n  ")
        );
        game.frames(READS_KEYS_AFTER);
        assert!(
            marked_on_the_frame_a_press_reaches(&game, "the chapter put back", keys::Z, || game
                .log()
                .said("retry chapter")),
            "the mark went off on the frame the chapter was put back on",
        );
        // And the row underneath really is the game's own count being painted over, not an empty panel:
        // the same two things `pointdevice_run.rs` reads of the mark, on this frame.
        assert!(
            game.drawn()
                .quads
                .iter()
                .any(|quad| quad.covers(&lives_row())),
            "nothing covered the row the lives are counted in",
        );
        let word = game.says("DISABLE");
        assert!(
            lives_row().overlaps(&word[0]),
            "the word is not over the count of lives: {:?} against {:?}",
            word[0],
            lives_row(),
        );
    });
}

/// The mark reaches one frame past the end of the run, which is the frame that stays on the screen.
///
/// `esc` and then やめる ends the run on a single frame — `run ended after 8 retries` and `f20724 scene=1`
/// together, with `f20700 scene=2 paused` before them — and the panel stays on the screen after it, so the
/// row the game paints on that frame is the one left standing there for the whole fade to the title. A mark
/// that stopped with the run stopped one frame early, and the stars showed plain.
///
/// What ends it instead is the game's own `Gui` no longer being in the draw chain. `Gui::RegisterChain` at
/// **0x41b252** registers two statics: **0x69bc7c** through `AddToCalcChain` at priority **0xc**, and
/// **0x69bc5c** through `AddToDrawChain` at priority **0xb** with `Gui::OnDraw` (**0x417502**) at +4 and
/// `&g_Gui` at +0x1c. Which of the two `Chain` lists is which came out of the lines they log — "add calc
/// chain (pri = %d)" at 0x46afb8 against "add draw chain (pri = %d)" at 0x46afd4 — and the draw list's head
/// is the calc list's **0x20** further in. So the mark is drawn while 0x69bc5c is in that list, which also
/// means it can never be drawn over a screen that is no longer the panel.
///
/// The log says what it is worth where each run ends: `lives: the mark stayed on the panel for 1 frame(s)
/// after the run ended`.
#[test]
fn the_mark_stays_on_the_panel_for_the_one_frame_the_game_paints_after_the_run() {
    in_its_own_process(|| {
        let game = Fake::attach("the-mark-after-the-run", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.in_a_pointdevice_run();
        assert!(
            game.image().gui_in_the_draw_chain(),
            "the game's own Gui is not in the draw chain while a stage is being played",
        );

        // `esc` and then やめる: the run is over on that frame, and the panel — with `g_Gui`'s job still in
        // the draw chain — stands until the front end is built on the frame after. So the row the game
        // paints on it is the one left on the screen, and the mark has to be on that row.
        game.gives_the_run_up_at_its_own_pause();
        let marked = marked_on_the_frame(&game, "the run ended", 8, || {
            game.log().said("run ended after")
        });
        assert!(
            marked,
            "the mark stopped with the run, one frame before the painting did — which is the stars \
             showing plain for the whole fade to the title:\n  {}",
            game.log().lines().join("\n  ")
        );

        // Said, and said as one frame: that count is the whole of whether the mark stops a frame early,
        // and half a second of fade is not something to judge by eye.
        game.frames_until("the front end taking the panel down", 8, || {
            !game.image().gui_in_the_draw_chain()
        });
        game.frames(2);
        assert!(
            game.log()
                .said("lives: the mark stayed on the panel for 1 frame(s) after the run ended"),
            "the mark did not reach exactly one frame past the run:\n  {}",
            game.log()
                .lines()
                .iter()
                .filter(|line| line.contains("lives:") || line.contains("run ended"))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    });
}

/// Giving the same run up from orb's own retry menu needs none of those frames, the chain being cut
/// already.
///
/// Measured: no such line at all where a run was given up that way. Which is the other half of what the
/// count is worth — a line saying one frame is a fact about how the *game* left the run, not a number orb
/// adds to every ending — and it is why the count is reported rather than assumed.
#[test]
fn a_run_given_up_at_orbs_own_menu_needs_no_frame_after_it() {
    in_its_own_process(|| {
        let game = Fake::attach("the-mark-given-up", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.in_a_pointdevice_run();
        // The mark on, first, because the assertion below is a negative one: a line that is absent because
        // orb never marked anything would pass it and mean nothing.
        game.one_frame();
        assert!(
            !game.says("DISABLE").is_empty(),
            "the mark was never on the panel, so the line this e2e test asks about could not be there \
             for any reason worth reading",
        );

        // タイトルに戻る, which is the third of the retry menu's items and reached by pressing up once from
        // the first: the same walk `pointdevice_run.rs` makes.
        game.hit();
        game.frame();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the give-up asking", || {
            game.log().said("retry: asking about the run given up")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the run given up", || {
            game.log().said("retry: the run is given up")
        });
        game.frames_until("the ranking built and taken down", 60, || {
            game.log().said("score: the ranking built and taken down")
        });
        game.frames(4);

        assert!(
            !game
                .log()
                .lines()
                .iter()
                .any(|line| line.contains("the mark stayed on the panel")),
            "orb counted frames of panel after a run it took down itself:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// The strips either side of the count are painted with the game's own panel sheet where there is one to
/// paint with, and flat where there is not.
///
/// **Which matters because nothing repaints the panel after a stage's first 250 frames.** What orb leaves
/// on the row is what stays there, so it has to be the panel and not a patch: 紅魔郷's is a noise of four
/// shades within seven of each other, and the flat colour is their average — close enough to be worth
/// having and not close enough to be right.
///
/// The sheet is `data/front.anm`'s, in slot 13 of the anm manager's own array — `ANM_FILE_FRONT` — and
/// what orb reads is that slot through a pointer to the manager. Reading it *without* the chase was
/// tried and is what the flat patch was: the offset added to the address of the pointer instead of to
/// the manager reads whatever lies past the pointer, which is no texture at all.
#[test]
fn the_strips_are_painted_with_the_games_own_panel_tile_where_there_is_one() {
    in_its_own_process(|| {
        // With no sheet first, which is what a game whose anm manager orb cannot find has: the strips are
        // a flat fill, and orb says so rather than leaving somebody to wonder why the panel has a patch
        // in it.
        {
            let game = Fake::attach("the-mark-no-tile", the_run(), |config| {
                config.log_level = LogLevel::Verbose;
            });
            game.in_a_pointdevice_run();
            game.one_frame();
            assert!(
                game.log().said(
                    "lives: no panel tile; the strips are painted flat and will show as a patch"
                ),
                "orb did not say the panel was being painted flat:\n  {}",
                game.log().lines().join("\n  ")
            );
            assert!(
                game.drawn()
                    .quads
                    .iter()
                    .all(|quad| quad.texture != FRONT_SHEET),
                "a sheet the game has not loaded was bound to paint the panel with",
            );
        }

        // And with the sheet where the game keeps it: the strips go through that texture, tile by tile.
        let game = Fake::attach("the-mark-the-tile", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.image().loads_the_front_sheet(FRONT_SHEET);
        game.in_a_pointdevice_run();
        game.one_frame();
        assert!(
            game.log()
                .said("lives: the panel's own tile is what the strips are painted with"),
            "orb did not find the sheet the game had loaded:\n  {}",
            game.log().lines().join("\n  ")
        );
        let strips: Vec<_> = game
            .drawn()
            .quads
            .into_iter()
            .filter(|quad| quad.texture == FRONT_SHEET)
            .collect();
        assert!(
            !strips.is_empty(),
            "nothing was painted with the game's own sheet",
        );
        // Tiles rather than one stretched piece: the sprite is 32x32 and it is laid every 32 pixels, so
        // no piece of it is wider than that.
        assert!(
            strips.iter().all(|quad| quad.width <= TILE),
            "the sheet was stretched over the panel rather than laid across it: {strips:?}",
        );
        // And they are beside the row and not over it: the count is the game's to paint, and painting the
        // panel over it would take the count away — the mark is what goes on the row itself.
        assert!(
            strips
                .iter()
                .all(|quad| !lives_row().overlaps(quad) || quad.height <= 0.0),
            "the panel was painted over the row the game repaints for orb: {strips:?}",
        );
    });
}

/// The sheet `data/front.anm` is loaded into, as an address.
///
/// A number and not an object: orb binds it and draws with it — the simulated device writes down which
/// texture each quad went through — and never reads a word out of it. Nothing else in a laid-out game is
/// at this address, which is the whole of what it has to be.
const FRONT_SHEET: usize = 0x0500_0000;

/// How far apart the panel's own tiles go, which is 紅魔郷's sprite 5: 32x32, laid every 32 pixels.
const TILE: f32 = 32.0;

/// Two fields in the ask, and the game's own repaint of a stage's first frames is why one of them decides
/// nothing.
///
/// One was tried first, to leave no repaint standing for the frame after the last marked one, and it is not
/// what put the count back: the panel being laid over a stage's **first 250 frames** sets all five of those
/// fields to 2 itself, at **0x41a2b6**, so during those frames the value orb writes decides nothing.
///
/// What makes that answerable rather than a matter of two writes agreeing is the other end of the same
/// mechanism: `Gui::OnDraw` takes one off each field at 0x41acdb, so a field nothing writes again is
/// nothing two draws later. Over the first 250 frames all five are set, and orb writes only the lowest
/// pair — so the four above it are the game's own repaint and the lowest pair would have been set without
/// orb. Past them only the lowest pair is set, and that is orb's write being the one that decides.
#[test]
fn the_games_own_repaint_of_a_stages_first_frames_overrides_what_orb_writes() {
    in_its_own_process(|| {
        let game = Fake::attach("the-mark-over-the-panel", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        // Long enough that a stage outlasts its own panel, since that is the boundary being read.
        game.stages_last(PANEL_FRAMES * 3);
        game.in_a_pointdevice_run();
        // The first chapter is 248 frames in, so the run is inside the panel's own frames here — with two
        // left, which is what makes the reads below the two sides of one boundary.
        let inside = game.state().stage_frames;
        assert!(
            inside < PANEL_FRAMES,
            "the run is already past the panel's own frames at {inside}",
        );

        // Inside them: every one of the five fields set. Only the lowest pair is orb's, so the other four
        // are the game laying its panel — and they say the row would be repainted whether orb wrote or not.
        let flags = game.image().gui_flags();
        let field = |at: u32| (flags >> (at * 2)) & 0b11;
        assert!(
            (0..5).all(|at| field(at) != 0),
            "the panel is not being laid over the stage's first frames: {flags:#07b}",
        );

        // Past them the script has stopped, and the game's own draw has taken the four it was writing down
        // to nothing. The lowest pair is still set, which is orb writing it every frame it draws the mark:
        // these are the frames that write decides.
        game.frames_until("the panel's own frames over", PANEL_FRAMES * 2, || {
            game.state().stage_frames > PANEL_FRAMES + 4
        });
        let flags = game.image().gui_flags();
        let field = |at: u32| (flags >> (at * 2)) & 0b11;
        assert_ne!(
            field(0),
            0,
            "nothing is keeping the lives' row repainted past the panel's own frames: {flags:#07b}",
        );
        assert!(
            (1..5).all(|at| field(at) == 0),
            "the game is still laying its panel past the frames it lays it over: {flags:#07b}",
        );
    });
}
