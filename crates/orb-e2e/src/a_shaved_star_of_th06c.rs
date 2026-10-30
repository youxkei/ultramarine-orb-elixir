//! **The star a part of a life is gone from, on th06c** — and the two things that were wrong with it on the
//! real game: the panel never repainted the row the count is in, so a part taken was a count nothing on the
//! screen moved for, and the part itself was painted with a colour of orb's own, because this game's panel is a
//! noise orb had no tile of.
//!
//! **Where the stars are is half of it.** The panel's draw puts them at 496 by 122 in this game's own
//! 640x480 picture — and that picture is four by three inside a buffer that is not, with black either side.
//! Orb's output stretched over the whole buffer put a shave at the draw's 496 beside stars that a picture of
//! the frame showed at 452, which is what a launch of this game showed: a rectangle beside the stars. See
//! `orb_core::canvas::Canvas::the_picture`.
//!
//! The pixels the cut is painted with cannot be read here — a laid-out frame is no picture — so what is held to
//! is where they came from: the copy this game's overlay takes out of its own frame at the slot beside the star,
//! and the quads drawn from what that copy went into.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::{WHAT_THE_FRAME_HOLDS, keys};

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// **The panel's own word of which rows to draw again**, the lives' pair its lowest — see
/// `orb_core::game::th06c`'s `THE_GUIS_FLAGS`, which is not public because no game but this one has it.
const THE_GUIS_FLAGS: usize = 0x1_4095_5368;
const THE_LIVES_ROW: u32 = 0b11;
const PAINT_FOR_TWO_FRAMES: u32 = 0b10;

/// And where the panel draws the star of the run's first life, in the picture this game's own frame is made
/// in — see the same game's `life_star`.
const FIRST_STAR: (f32, f32) = (496.0, 122.0);
const A_STARS_WIDTH: f32 = 16.0;
const A_STARS_HEIGHT: f32 = 16.0;

/// Frames a launch is given to reach the run it is asked for.
const TO_THE_RUN: u32 = 30_000;

/// **The output the overlay is placed in**, which is what the game's panels are laid out in and what the
/// canvas places on the game's own picture inside its buffers — see `orb_core::canvas::Canvas::the_picture`.
const OUTPUT: (f32, f32) = (640.0, 480.0);

/// **Where a point of the output lands in the laid-out game's buffers**: the game's picture is four by three
/// and the buffer is `BACK_BUFFER`, so the picture is the largest four-by-three box in it, centred.
fn in_the_buffer(x: f32, y: f32) -> (f32, f32) {
    let (width, height) = (
        crate::fake::BACK_BUFFER.0 as f32,
        crate::fake::BACK_BUFFER.1 as f32,
    );
    let scale = (width / OUTPUT.0).min(height / OUTPUT.1);
    let (picture_width, picture_height) = (OUTPUT.0 * scale, OUTPUT.1 * scale);
    (
        (width - picture_width) / 2.0 + x * scale,
        (height - picture_height) / 2.0 + y * scale,
    )
}

/// **`TO_A_RUN`'s walk with the mode answered as the bomb mode and its difficulty after it** — the third item
/// of the question's list and Easy, the first of the page that asks which.
fn the_recipe() -> String {
    "\
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
- answer: { question: mode, item: 2 }
  give_up_after: 300
- answer: { question: difficulty, item: 0 }
  give_up_after: 300
- bombs: false
- wait_until: { screen: 7 }
  give_up_after: 1800
- press: [down]
  frames: 6
  until: { screen: 7, cursor: 1 }
  give_up_after: 900
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
- wait_until: { screen: 9 }
  give_up_after: 1800
- press: [up]
  frames: 6
  until: { screen: 9, cursor: 0 }
  give_up_after: 900
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
- wait_until: { screen: 11 }
  give_up_after: 1800
- wait_until: { frames: 120 }
- press: [decide]
  frames: 6
  until: { stage: 1, chapter: 1 }
  give_up_after: 1800
- wait_until: { frames: 3600 }
  give_up_after: 7200
"
    .to_owned()
}

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(the_recipe());
            config.log_level = LogLevel::Verbose;
        },
    )
}

#[test]
fn a_part_of_a_life_of_th06c_is_cut_out_of_the_star_the_run_lost_it_from() {
    in_its_own_process(|| {
        let game = launched("th06c-a-shaved-star");
        let log = game.log();
        // ── The run, in the bomb mode on Easy, with its first chapter taken.
        game.frames_until("the run's first chapter", TO_THE_RUN, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        assert!(
            log.said("bomb: the difficulty is easy"),
            "the mode was not the bomb mode on Easy: {:?}",
            log.lines()
        );
        assert_eq!(
            game.sim().space().read::<u32>(THE_GUIS_FLAGS) & THE_LIVES_ROW,
            0,
            "the panel's row was asked for before any part of a life was owed",
        );
        // **And a frame of the run copies nothing out of itself**: the picture the retry screen is drawn over
        // is taken once, on the frame the question goes up — see `orb_core::runtime`'s `Runtime::last_frame` —
        // so a copy before the hit would be one nothing asked for.
        assert!(
            game.sim().drawing().drawn().copies.is_empty(),
            "a piece of the frame was copied out before anything was shaved: {:?}",
            game.sim().drawing().drawn().copies,
        );

        // ── A hit with the button never pressed, and the retry menu answered as somebody at the keyboard
        // answers it. The frames first, the player being still in its spawn's invulnerability for the first
        // of a stage — see `a_death_of_th06c`'s own `in_a_run`.
        game.frames(300);
        game.hit();
        game.frames_until("the death judged", 240, || {
            log.said("not pressed in time; 3/12 owed, lives 2")
        });
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        game.frames(6);

        // ── 1. **The row the count is in is asked for**, which is what makes the count follow: this game paints
        // it only while its own word says so.
        assert_eq!(
            game.sim().space().read::<u32>(THE_GUIS_FLAGS) & THE_LIVES_ROW,
            PAINT_FOR_TWO_FRAMES,
            "the panel was not asked to paint the lives' row again, so the count cannot follow the part",
        );

        // ── 2. **And the part is painted with the game's own pixels**, copied out of the frame at the bare slot
        // beside the star — not with a colour of orb's.
        let drawn = game.sim().drawing();
        let copies = drawn.drawn().copies;
        assert!(
            !copies.is_empty(),
            "no piece of the frame was copied out for the shave: {:?}",
            log.lines()
        );
        let bare = FIRST_STAR.0 + A_STARS_WIDTH * 2.0;
        // **In the buffer's own pixels and not the output's**, which is what a copy and a quad are placed in —
        // see [`in_the_buffer`].
        let (bare_in_the_buffer, _) = in_the_buffer(bare, FIRST_STAR.1);
        assert!(
            copies
                .iter()
                .any(|copy| copy.left as f32 <= bare_in_the_buffer
                    && bare_in_the_buffer < (copy.left + copy.width) as f32),
            "the copy was not taken at the bare slot beside the second star: {copies:?}",
        );
        let pieces: Vec<_> = drawn
            .drawn()
            .quads
            .into_iter()
            .filter(|quad| {
                drawn.pixels_of(quad.texture).is_some_and(|pixels| {
                    !pixels.is_empty() && pixels.iter().all(|p| *p == WHAT_THE_FRAME_HOLDS)
                })
            })
            .collect();
        assert!(
            !pieces.is_empty(),
            "nothing was drawn with what was copied out of the frame: {:?}",
            log.lines()
        );
        // The star the part is gone from is the second of the run's two lives, and what is drawn over it is
        // inside its own box.
        let (left, top) = in_the_buffer(FIRST_STAR.0 + A_STARS_WIDTH, FIRST_STAR.1);
        let (right, bottom) = in_the_buffer(
            FIRST_STAR.0 + A_STARS_WIDTH * 2.0,
            FIRST_STAR.1 + A_STARS_HEIGHT,
        );
        assert!(
            pieces.iter().any(|piece| piece.x >= left
                && piece.y >= top
                && piece.right() <= right
                && piece.bottom() <= bottom),
            "no piece of the frame was drawn inside the star the part is gone from: {pieces:?}",
        );
    });
}
