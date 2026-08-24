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
}

impl TakingHits {
    /// What a `take_hits` step says, from the step it is written at onwards.
    pub fn told(&mut self, told: Told) {
        self.told = Some(told);
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
        self.was_offered = offered;
        let Some(told) = self.told else {
            return Hazards::Dodged;
        };
        // **And nothing off a chapter of a run.** A question of orb's own is up over a stage the game has
        // stopped updating; a run that is over is a screen with no player on it; and chapter nought is orb's
        // own answer for a run that has not reached one — which is every frame of the front end a recipe walks
        // in through.
        if offered || reached.over || reached.chapter == 0 {
            return Hazards::Dodged;
        }
        let at = (reached.stage, reached.chapter);
        if self.at != Some(at) || self.again {
            self.retried = match self.at {
                Some(was) if was == at && self.again => self.retried.saturating_add(1),
                // A chapter reached rather than started again is one nobody has died in yet.
                _ => 0,
            };
            self.again = false;
            self.at = Some(at);
            self.frames = 0;
            let chance = told.chance * told.times_each_retry.powi(self.retried as i32);
            self.drawn = drawn(at, self.retried) < chance;
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

/// **The number one chapter attempt is drawn**, in nought to one.
///
/// A hash of which attempt it is and not a step of a sequence, which is what makes two launches of one recipe
/// the same run: a sequence hands out its numbers in the order they are asked for, so a chapter that lasted a
/// frame longer than last time would move every draw after it along one.
///
/// splitmix64's own finaliser, which is the cheapest mix that leaves neighbouring chapters unrelated — the
/// three numbers packed into one word are all small and differ in their low bits, which is exactly what a
/// weaker one carries through into the answer.
fn drawn((stage, chapter): (i32, u32), retried: u32) -> f32 {
    let mut mixed = ((stage as u32 as u64) << 48) | ((chapter as u64) << 32) | (retried as u64);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^= mixed >> 31;
    // The top 24 bits over how many of them there are, which is a number an f32 holds every one of.
    (mixed >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::drawn;

    /// **Every attempt draws its own number, and the same one every time it is asked.** Which is what the run
    /// being repeatable rests on: the chapters a launch dies in are the chapters the next launch of the same
    /// recipe dies in.
    #[test]
    fn an_attempt_draws_the_same_number_every_time_it_is_asked() {
        assert_eq!(drawn((3, 5), 0), drawn((3, 5), 0));
        assert_ne!(drawn((3, 5), 0), drawn((3, 5), 1));
        assert_ne!(drawn((3, 5), 0), drawn((3, 6), 0));
        assert_ne!(drawn((3, 5), 0), drawn((4, 5), 0));
    }

    /// **And the numbers are spread over nought to one**, which is what makes comparing one with a chance a
    /// chance at all: a draw that came out of the bottom of the range would be a chapter hit in whatever the
    /// step said.
    #[test]
    fn the_numbers_drawn_are_spread_over_nought_to_one() {
        let all: Vec<f32> = (1..=6)
            .flat_map(|stage| (1..=30).map(move |chapter| drawn((stage, chapter), 0)))
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
