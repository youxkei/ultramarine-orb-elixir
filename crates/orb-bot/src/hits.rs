//! **Being hit on purpose**, which is the one thing rules that dodge everything cannot show.
//!
//! Pointdevice mode's whole content is what happens *after* a death — the chapter put back, the run carried on
//! from it — and a run that is never hit reaches none of that. Every measurement of a retry so far has been a
//! chapter asked to start over, which is the pause menu's own ask and not a death: nothing was written down,
//! nothing was lost, and the question was answered from a screen the run had chosen to be on.
//!
//! **So a chapter is drawn for**: `take_hits` hands the rules a chance, the draw is made as the chapter starts,
//! and an attempt drawn is one the hazards are ignored in — see [`Hazards`]. What the run does then is what
//! somebody who has stopped paying attention does, and where it ends up is a death in the middle of a pattern.
//!
//! **And the chance is lowered each retry, so that the run gets past the chapter it died in.** At the same
//! chance every attempt a chapter is retried until the fire happens to miss, which is an afternoon spent on one
//! chapter; lowered, the attempts after the first are played more carefully than the one before, which is what
//! somebody who has just died there plays like.
//!
//! **The draws are a hash and not a sequence**, so that the same recipe dies in the same chapters every launch:
//! two launches are comparable because they are the same run, and a sequence would move every draw along as
//! soon as anything before it differed.
//!
//! **The one draw that carries a number of the launch's own is the giving up** — see [`A_LAUNCHS_OWN`], and the
//! eight launches that handed one run to each other.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use orb_core::game::{Question, Reached};
use orb_core::{detail, summary};

/// **What the rules do about the hazards on a frame**, which is the whole of what a `take_hits` step changes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Hazards {
    /// Dodged, which is every frame of a run nobody asked to be hit in.
    Dodged,
    /// **Ignored**: the move is chosen by where the rules want to be and none of the hazards decides any of
    /// it. Which is what somebody who has stopped paying attention plays like, and the death it ends in lands
    /// where theirs does — in the middle of a pattern, rather than against one bullet singled out to be run
    /// into.
    Ignored,
    /// **And gone to**: the nearest thing that can hit the player is where the rules want to be, which is a
    /// chapter drawn to be hit in that has gone [`GOES_TO_IT_AFTER`] frames without being hit.
    GoneTo,
}

/// **How long an attempt drawn to be hit in ignores the hazards before it goes to the nearest of them**, in
/// frames.
///
/// **A second, because a chapter is short.** The 114 chapters two launches of th07 played through ran from 47
/// frames to 3510 with a median of 237 — four seconds — and a quarter of them were under 112. So a wait is a
/// thing most chapters have no room for: at three seconds, 37 in every hundred of them would end before the
/// going-to began, and the chance a step was given would be the chance of something else.
///
/// **And not nought, which is a death nobody has ever died.** A run that walks at the nearest bullet on the
/// frame its chapter starts is hit at the top of every chapter it is drawn in, against whatever happened to be
/// nearest — where what these runs are for is the death somebody playing takes.
const GOES_TO_IT_AFTER: u32 = 60;

/// What a `take_hits` step told the rules, which stands until another one tells them otherwise.
#[derive(Clone, Copy)]
pub struct Told {
    pub chance: f32,
    pub times_each_retry: f32,
    pub gives_up: f32,
}

/// **The chapter attempt being played and whether it is one to be hit in**, which is what a frame is answered
/// out of.
#[derive(Default)]
pub struct TakingHits {
    told: Option<Told>,
    /// The chapter the attempt in flight is at, and nothing before the first frame of one.
    at: Option<(i32, u32)>,
    /// How many times that chapter has been retried, which is what its chance is lowered by.
    retried: u32,
    /// Frames of this attempt at it.
    frames: u32,
    /// Whether this attempt was drawn to be hit in.
    drawn: bool,
    /// Whether orb's own offer to retry was up on the frame before — see [`TakingHits::this_frame`].
    was_offered: bool,
    /// And whether the chapter is to be drawn for again on the next frame of one, which taking that offer
    /// makes true.
    again: bool,
    /// Whether the attempt in flight is still to be drawn for, which a chapter starting and a `take_hits` step
    /// being reached both make true.
    undrawn: bool,
}

impl TakingHits {
    /// What a `take_hits` step says, from the step it is written at onwards — **the attempt in flight included**,
    /// which is drawn for again on its next frame.
    ///
    /// **Not only at a chapter's start**, which is where the draw was before every attempt was counted: an
    /// attempt is now counted from the first frame of its chapter whether or not a step asked for hits, so a
    /// draw made only there is made before a step written inside the chapter is reached — th06c's own run
    /// asked for hits in its stage's first chapter and was drawn for in its second, and took none.
    pub fn told(&mut self, told: Told) {
        self.told = Some(told);
        self.undrawn = true;
    }

    /// How many times the chapter in flight has been retried: nought on the attempt that reached it.
    pub fn attempt(&self) -> u32 {
        self.retried
    }

    /// Whether the attempt in flight was drawn to be hit in — and so, at an offer to retry, whether the death it
    /// answers is one the rules walked into.
    pub fn drawn_to_be_hit_in(&self) -> bool {
        self.drawn
    }

    /// **What the rules do about the hazards this frame**, with the draw for a chapter made as it starts.
    ///
    /// **The offer to retry going down is the chapter being started again.** A retry puts back the chapter it
    /// was asked from, so its number is the same on either side of the death and nothing else a bot is given
    /// says an attempt ended: what marks one is orb's own question having been up and being gone.
    pub fn this_frame(&mut self, reached: Reached) -> Hazards {
        let offered = reached.question == Some(Question::Retry);
        if self.was_offered && !offered {
            self.again = true;
        }
        // **And the offer coming up is where a run is given up rather than carried on**, which is the other
        // thing somebody who has just died does — and the one that leaves the run in a file for the next
        // launch to pick up. Drawn on the frame it comes up, so that one offer is one draw.
        if let Some(told) = self.told
            && offered
            && !self.was_offered
        {
            let chance = told.gives_up;
            let at = self.at.unwrap_or_default();
            if drawn(at, self.retried, GIVING_UP ^ a_launchs_own()) < chance {
                orb_core::runtime::give_the_run_up_at_the_retry();
                let (stage, chapter) = at;
                summary!(
                    "autoplay: the offer to retry stage {stage} chapter {chapter} is one to give the \
                     run up at, its chance being {chance:.3}",
                );
            } else {
                detail!(
                    "autoplay: the offer to retry is one to take the chapter again at, the chance of \
                     giving up being {chance:.3}",
                );
            }
        }
        self.was_offered = offered;
        // **And nothing off a chapter of a run.** A question of orb's own is up over a stage the game has
        // stopped updating; a run that is over is a screen with no player on it; and chapter nought is orb's
        // own answer for a run that has not reached one — which is every frame of the front end a recipe walks
        // in through.
        if offered || reached.over || reached.chapter == 0 {
            return Hazards::Dodged;
        }
        let at = (reached.stage, reached.chapter);
        // **The attempt is counted whether or not a recipe asked for hits**, the rules waiting somewhere
        // else on each one — see `steer::this_attempt`.
        if self.at != Some(at) || self.again {
            self.retried = match self.at {
                Some(was) if was == at && self.again => self.retried.saturating_add(1),
                // A chapter reached rather than started again is one nobody has died in yet.
                _ => 0,
            };
            self.again = false;
            self.at = Some(at);
            self.undrawn = true;
        }
        if self.undrawn {
            self.undrawn = false;
            self.frames = 0;
            self.drawn = false;
            let Some(told) = self.told else {
                return Hazards::Dodged;
            };
            let chance = told.chance * told.times_each_retry.powi(self.retried as i32);
            self.drawn = drawn(at, self.retried, BEING_HIT_IN) < chance;
            let (stage, chapter) = at;
            // **Said where it was drawn and only asked for where it was not.** A run being watched is watched
            // for the deaths in it, and a line per chapter that is to be played out is a line per chapter.
            if self.drawn {
                summary!(
                    "autoplay: stage {stage} chapter {chapter} is one to be hit in, its chance \
                     being {chance:.3} after {} retr(ies)",
                    self.retried,
                );
            } else {
                detail!(
                    "autoplay: stage {stage} chapter {chapter} is not one to be hit in, its chance \
                     being {chance:.3} after {} retr(ies)",
                    self.retried,
                );
            }
        }
        self.frames += 1;
        if !self.drawn {
            return Hazards::Dodged;
        }
        if self.frames <= GOES_TO_IT_AFTER {
            return Hazards::Ignored;
        }
        if self.frames == GOES_TO_IT_AFTER + 1 {
            detail!(
                "autoplay: nothing has hit it in {GOES_TO_IT_AFTER} frame(s), so it goes to the \
                 nearest thing that can",
            );
        }
        Hazards::GoneTo
    }
}

/// The three things an attempt is drawn for, which are three draws and not one — see [`drawn`].
const BEING_HIT_IN: u64 = 0;
const GIVING_UP: u64 = 1;
/// Where the rules wait — see `steer::aside_on`.
pub(crate) const WAITING_ASIDE: u64 = 2;

/// **A number of this launch's own, which the giving up is drawn with and the being hit in is not.**
///
/// **Because a run given up is picked up at the chapter it was given up in.** The draws are a hash of the
/// attempt, so the launch that picks that run up meets the same chapter with the same retry count and draws the
/// same number — and gives the run up again. Measured: eight launches of th06 in a row gave the run up at
/// stage 2 chapter 3, each one handing it to the next, and the run never reached stage 3.
///
/// **What it costs is that two launches of one recipe give up at different deaths**, and what it keeps is the
/// half that makes a run comparable: which chapters are *hit in* is the attempt's own hash still, so the deaths
/// a launch takes are the deaths the launch before it took.
///
/// The clock, because a launch has nothing else of its own: the recipe, the game and the run are all the same
/// as the last launch's, which is the whole problem.
static A_LAUNCHS_OWN: AtomicU64 = AtomicU64::new(0);

/// Settled once per launch, on the first draw that wants it.
fn a_launchs_own() -> u64 {
    match A_LAUNCHS_OWN.load(Ordering::Relaxed) {
        0 => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(1, |since| since.as_nanos() as u64 | 1);
            A_LAUNCHS_OWN.store(now, Ordering::Relaxed);
            now
        }
        settled => settled,
    }
}

/// **The number one chapter attempt is drawn**, in nought to one.
///
/// A hash of which attempt it is and not a step of a sequence, which is what makes two launches of one recipe
/// the same run: a sequence hands out its numbers in the order they are asked for, so a chapter that lasted a
/// frame longer than last time would move every draw after it along one.
///
/// splitmix64's own finaliser, which is the cheapest mix that leaves neighbouring chapters unrelated — the
/// three numbers packed into one word are all small and differ in their low bits, which is exactly what a
/// weaker one carries through into the answer.
///
/// `about` is which question is being asked of that attempt, so that the two draws an attempt takes are
/// unrelated: at one salt a chapter drawn to be hit in would be a chapter drawn to be given up at.
pub(crate) fn drawn((stage, chapter): (i32, u32), retried: u32, about: u64) -> f32 {
    let mut mixed =
        ((stage as u32 as u64) << 48) | ((chapter as u64) << 32) | ((retried as u64) << 8) | about;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^= mixed >> 31;
    // The top 24 bits over how many of them there are, which is a number an f32 holds every one of.
    (mixed >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use orb_core::game::{Question, Reached};

    use super::{BEING_HIT_IN, GIVING_UP, Hazards, TakingHits, Told, drawn};
    use crate::steer::aside_on;

    /// A frame of stage 1's chapter, with orb's offer to retry up or not.
    fn in_chapter(chapter: u32, offered: bool) -> Reached {
        Reached {
            stage: 1,
            chapter,
            question: offered.then_some(Question::Retry),
            cursor: None,
            screen: None,
            over: false,
            boss: false,
            dialogue: false,
            card: false,
            card_name: false,
            typing_a_name: false,
            spell: None,
            card_frames: None,
            stage_frames: None,
            difficulty: None,
            on_the_question: None,
        }
    }

    /// **A chapter taken again is its next attempt and a chapter reached is its first**, with no `take_hits`
    /// written — the attempt being what the rules wait somewhere else on, whether or not they are hit.
    #[test]
    fn each_retry_of_a_chapter_is_its_next_attempt_and_the_chapter_after_it_starts_at_the_first() {
        let mut hits = TakingHits::default();
        hits.this_frame(in_chapter(3, false));
        assert_eq!(hits.attempt(), 0);
        for retry in 1..=2 {
            hits.this_frame(in_chapter(3, true));
            hits.this_frame(in_chapter(3, false));
            assert_eq!(hits.attempt(), retry);
        }
        hits.this_frame(in_chapter(4, false));
        assert_eq!(hits.attempt(), 0);
    }

    /// **A `take_hits` step reached inside a chapter is drawn for in that chapter**, and not in the next one:
    /// the attempt was counted from the chapter's first frame with nothing told, and a hit asked for there is
    /// asked for there.
    #[test]
    fn a_take_hits_step_reached_inside_a_chapter_is_drawn_for_in_that_chapter() {
        let mut hits = TakingHits::default();
        assert!(hits.this_frame(in_chapter(1, false)) == Hazards::Dodged);
        hits.told(Told {
            chance: 1.0,
            times_each_retry: 0.0,
            gives_up: 0.0,
        });
        assert!(
            hits.this_frame(in_chapter(1, false)) == Hazards::Ignored,
            "the chapter in flight was not drawn to be hit in",
        );
        assert_eq!(hits.attempt(), 0, "being told is no retry");
    }

    /// **The first attempt waits where the rules always have and each retry somewhere of its own**, no
    /// further than a quarter of the field either way.
    #[test]
    fn the_first_attempt_waits_where_it_always_has_and_each_retry_somewhere_else() {
        assert_eq!(aside_on(0), 0.0);
        let asides: Vec<f32> = (1..=8).map(aside_on).collect();
        assert!(
            asides.iter().all(|aside| (-96.0..=96.0).contains(aside)),
            "{asides:?} strays past a quarter of the field",
        );
        assert!(
            asides
                .windows(2)
                .all(|pair| (pair[0] - pair[1]).abs() > 1.0),
            "{asides:?} has two retries in a row waiting in the same place",
        );
        assert!(
            asides.iter().any(|aside| *aside < 0.0) && asides.iter().any(|aside| *aside > 0.0),
            "{asides:?} waits on one side of the boss alone",
        );
    }

    /// **Every attempt draws its own number, and the same one every time it is asked.** Which is what the run
    /// being repeatable rests on: the chapters a launch dies in are the chapters the next launch of the same
    /// recipe dies in.
    #[test]
    fn an_attempt_draws_the_same_number_every_time_it_is_asked() {
        assert_eq!(
            drawn((3, 5), 0, BEING_HIT_IN),
            drawn((3, 5), 0, BEING_HIT_IN)
        );
        assert_ne!(
            drawn((3, 5), 0, BEING_HIT_IN),
            drawn((3, 5), 1, BEING_HIT_IN)
        );
        assert_ne!(
            drawn((3, 5), 0, BEING_HIT_IN),
            drawn((3, 6), 0, BEING_HIT_IN)
        );
        assert_ne!(
            drawn((3, 5), 0, BEING_HIT_IN),
            drawn((4, 5), 0, BEING_HIT_IN)
        );
        // And the two questions an attempt is asked are drawn for apart, which is what stops every chapter
        // that is hit in being one that is also given up at.
        assert_ne!(drawn((3, 5), 0, BEING_HIT_IN), drawn((3, 5), 0, GIVING_UP));
    }

    /// **And the numbers are spread over nought to one**, which is what makes comparing one with a chance a
    /// chance at all: a draw that came out of the bottom of the range would be a chapter hit in whatever the
    /// step said.
    #[test]
    fn the_numbers_drawn_are_spread_over_nought_to_one() {
        let all: Vec<f32> = (1..=6)
            .flat_map(|stage| (1..=30).map(move |chapter| drawn((stage, chapter), 0, BEING_HIT_IN)))
            .collect();
        assert!(
            all.iter().all(|drawn| (0.0..1.0).contains(drawn)),
            "a draw came out from outside nought to one",
        );
        // A tenth of a chance drawn by about a tenth of them, over the 180 chapters six stages of th07 have.
        let tenth = all.iter().filter(|drawn| **drawn < 0.1).count();
        assert!(
            (9..=27).contains(&tenth),
            "{tenth} of 180 draws were under a tenth, which is not about a tenth of them",
        );
    }
}
