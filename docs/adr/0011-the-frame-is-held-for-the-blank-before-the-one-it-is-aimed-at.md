# The frame is held for the blank before the one it is aimed at

**Status:** accepted and built. Stands on
[0006](0006-the-frame-loop-waits-on-a-high-resolution-timer.md), whose wait it makes a second use of.
Overturns nothing; it replaces a bound on `budget_ceiling` that lived for one commit —
`dc56662 cut budget_ceiling to a display refresh` — and is why that bound is gone again.

## Context

The frame loop waits until `budget` before the blank it is aiming at, then does the frame's work,
then hands the frame over. `budget` is a measurement of what the work takes, tracked near the worst of
the recent frames rather than their average, because aiming at the average means missing the handover
on every frame heavier than it.

So how early a frame is really handed over is **`budget` less that frame's own work**, and those two
part company whenever a heavy frame is followed by a light one. The budget rises to a spike at once and
decays a sixty-fourth per frame, so for a couple of seconds after one, every frame is handed over most
of a budget early.

Past a refresh early, that is a fault and not merely lag. The compositor takes a frame at the first
blank it has had time to compose it for, so a frame handed over more than a refresh before the blank it
was aimed at is composed at the blank *before* it. `DwmFlush` returns at the blank the frame reached, so
the anchor every aim is counted from moves a refresh early with it, and the next frame is handed over as
early again. One frame per refresh is one update per refresh: the game runs at double speed.

Measured on a 120Hz desktop, and this is the whole reason the file exists. A frame whose `PLAY_SOUNDS`
ran 8438µs came to 8940µs of work, so the budget went to 11440µs. The frames after it did 250µs of work
apiece and were handed over 11190µs before their blanks; the blank one refresh earlier was 2857µs away
against the 2500µs the compositor wanted, so it composed them there. Five turns came out one refresh
apart — 6587, 9090, 8212, 8501 and 9820µs — about 120 frames a second, and the log said `10 shown a
refresh or more early, so the game ran fast for them`.

## The decision

**Hold the drawn frame until the blank before the one it is aimed at has gone, then hand it over.**
`Pacing::hold_for_the_blank_before`, called from the frame loop between the draw and `Present`.

After the drawing, because that is the only place the question has an answer: before it, the frame's own
work is a prediction, and it is the difference between the prediction and the fact that decides how
early the frame goes.

The target is that earlier blank itself rather than a margin before it — a frame handed over at a blank
cannot have been composed for it, the composing wanting time it did not have. What is left afterwards
is a whole refresh in which to be composed for the aimed blank, and `compose_ceiling` already holds the
compositor's share under three quarters of one, so the hold can never eat the compositor's own time
however long it waits.

It is not counted as the frame's work. A budget that grew to include a wait caused by the budget being
too high would start the next frame earlier, hold it longer, and grow again — the estimate chasing its
own wait to the ceiling.

## What was rejected

**Bounding `budget_ceiling` at one refresh**, which is what shipped in `dc56662` and what this
replaces. It works, and it is one line: the budget can then start a frame at most a refresh before its
blank, so the handover cannot precede the earlier blank whatever the work turns out to be.

What it costs is the other half of the budget's job. The budget exists so that a frame whose work does
not fit the tail of its turn is started earlier; bounded at a refresh, work heavier than a refresh less
the compositor's share cannot be covered at all, and every frame of it misses. Swept at 120Hz with
2500µs allowed: 5500µs a frame held the cadence, and 6000µs ran every frame three refreshes apart — 40
frames a second, from work a fifth of a game frame long. At 240Hz the same arithmetic leaves 1666µs
against the millisecond the game actually takes, so the room there was nearly gone.

Nothing in play reached that edge, which is why the bound was worth taking at the time and why it is not
worth keeping: it is a compromise where the invariant can simply be enforced where it belongs.

**Bounding it at three quarters of a refresh** is a third thing and was a mistake made earlier: at
144Hz that is the same 5208µs as the compositor's share, so the drawing has no allowance at all, every
frame reaches the compositor late, and the input lag and the share read as the same number on screen.

## What follows

- `budget_ceiling` is three quarters of a game frame again and takes no period. Nothing about it varies
  with the display, and `the_whole_budget_is_the_games_frame_and_not_the_displays` is that back.
- `Marks` carries a `held` mark between `drawn` and `presented`, so the pacing line's spans still add up
  to the gap. Folded into `present` the hold would read as a `Present` that took a refresh.
- The frames the hold fires on are shown at their own blank rather than a refresh early, so the input lag
  those frames report goes *up* by a refresh. That is the reading being correct rather than a cost: what
  it is measured against is where the frame was shown, and it was being shown somewhere nobody asked for.
- `orb-e2e`'s `pacing`'s `budget` holds both halves — `work_that_is_heavy_every_frame_is_covered` for the
  headroom, `a_spike_the_ceiling_admits_does_not_hand_the_frames_after_it_over_early` for this. The second
  asserts on the count of frames shown early rather than on the spacing of the handovers: a spike's own
  frame is shown a refresh late whatever is done, the frame after it comes back onto the grid, and the gap
  between those two handovers is a refresh short by arithmetic without either frame being anywhere it
  should not be.
