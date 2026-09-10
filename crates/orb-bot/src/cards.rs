//! **Ways of dodging one card of one game, each named for its card** — a card whose pattern is the same every
//! launch, so that what gets through it is its own schedule and nothing any other card shares. The ways that
//! are a mechanism a card may want, whichever card it is, are `steer`'s.

use orb_core::game::Frame;

/// **Where th08's `「インペリシャブルシューティング」` is waited at, by how far into the card it is** — and nothing
/// before its last wave, where the rules' own errand is the way.
///
/// **Its waves come at the same frames of the card every launch**, read off the frames its bullet count
/// jumped on in three Extras, each within a frame of the others: `124`, `575`, three at `1067`, `1095` and
/// `1127`, five `132` apart from `1559` to `2087`, five `8` to `12` apart from `2539` and from `2959`, one
/// from about `3363`, and the last from `4003`, which fills the bullet pool to its `1536` by `4123` and runs out at
/// about `4700`. The strategy wiki's line for that last one is `画面の右下端or左下端で避けると楽`.
///
/// **The rules' own through the seven before it**, which is under Mokou in the middle at the low line: they
/// took those waves with one hit in the three Extras, in the fifth, and the one thing a told place would add
/// there is the crossing to it. Through the last, one of those Extras found the bottom-right corner on its
/// own — at `331` to `370` along the floor for its last `300` frames, and was not hit.
pub(crate) fn imperishable_shooting(frame: &Frame, card_frames: i32) -> Option<(f32, f32)> {
    let field = frame.field;
    (card_frames >= IMPERISHABLE_SHOOTINGS_LAST_WAVE).then_some((
        field.0 + field.2 - IN_FROM_THE_RIGHT_WALL,
        field.1 + field.3,
    ))
}

/// The frame of `「インペリシャブルシューティング」` its last wave starts on — see [`imperishable_shooting`].
const IMPERISHABLE_SHOOTINGS_LAST_WAVE: i32 = 4000;

/// **How far in from the right wall that wave is waited at** — `352` on a field whose right is `376`, the middle
/// of the `331` to `370` the run that found the corner stood at.
const IN_FROM_THE_RIGHT_WALL: f32 = 24.0;

/// **Whether th08's `狂符「幻視調律（ビジョナリチューニング）」` has its lattice where none of it can hit the run**, by how
/// far into the card it is: hidden, and not shown again for more than [`BEFORE_IT_IS_SHOWN`] frames.
///
/// **Its lattice is hidden on the same frames of the card every launch**, read off every bullet of a Normal
/// practice of it frame by frame (`--trace-the-field-in=105`): shown from its first bullets at `91` to `340`, then
/// hidden for a hundred frames and shown for a hundred to the card's end, the first hidden frame `341` and the
/// first shown `441`. Shown, a bullet goes straight up or straight down at `1.3` a frame — born in pairs in a band
/// from about `y = 190` to `250`, one of each pair going each way — so that below the band the lattice is one body
/// going down: at the card's `1060`th frame, `306` of the `307` bullets between `250` and `450` were. Hidden, every
/// one of them goes sideways at `0.6` a frame, either way, and none can hit — see `th08::BULLET_UNTESTED` — where
/// `sight` hands them over as hitting all the same.
///
/// **So while it is hidden the run climbs, which the rules alone never do** — the strategy wiki's
/// `どんどん上を意識していこう`. A body of bullets coming down is not climbed against: every move up meets it faster
/// than standing does, and the moves that give way to it give way downwards, so the rules waiting at `260` were
/// carried to the floor — the practice reached it on the card's `681`st frame and was at the right wall's foot by
/// its `941`st, and two clears of one session spent a bomb twice apiece in it on stretches with nothing open,
/// three of the four at `y = 413` and below. And a lattice that cannot hit is no more climbed through, the rules
/// dodging it as though it could.
///
/// **And not for the last [`BEFORE_IT_IS_SHOWN`] frames**, where the rules' own dodging is what puts the run clear
/// of every bullet by the frame they can hit again: a hidden bullet is shown where it has got to.
pub(crate) fn visionary_tunings_lattice_cannot_hit(card_frames: i32) -> bool {
    let into = card_frames - VISIONARY_TUNINGS_FIRST_HIDDEN;
    into >= 0 && into % VISIONARY_TUNINGS_CYCLE < VISIONARY_TUNINGS_HIDDEN_FOR - BEFORE_IT_IS_SHOWN
}

/// The frame of `「幻視調律」` its lattice is first hidden on, and the frames of a hide and a show together and
/// of the hide alone — see [`visionary_tunings_lattice_cannot_hit`].
const VISIONARY_TUNINGS_FIRST_HIDDEN: i32 = 341;
const VISIONARY_TUNINGS_CYCLE: i32 = 200;
const VISIONARY_TUNINGS_HIDDEN_FOR: i32 = 100;

/// **How many frames before `「幻視調律」`'s lattice is shown again the rules dodge it again** — twenty, which is
/// room for the climb and room after it.
///
/// **The climb to [`VISIONARY_TUNING_ABOVE_THE_FLOOR`] is `36` frames from the floor** at the rules' whole step of
/// `4`, so the eighty frames of a hide this leaves are twice that; and a hidden bullet drifts `0.6` a frame,
/// so twenty frames of the rules' own dodging is a run that has five steps of its own to be out of any box it
/// is standing in by the frame the box can hit.
const BEFORE_IT_IS_SHOWN: i32 = 20;

/// **How far above the field's floor `「幻視調律」` is waited at** — `290` on a field whose floor is `432` — and in
/// the middle.
///
/// **Because the lattice carries the run down with it for a hundred frames at a time**, `130` of the game's
/// units at its `1.3` a frame: from here that ends at `420` and not against the floor, where the rules have
/// no move left to give way with. **And no higher**, the band from `190` to `250` being where every bullet of
/// the card is born: forty below it is thirty frames of a bullet born at its foot. **Nor at the low line**,
/// under which twenty-two attempts told nothing were shut in, `nowhere open to it` at `376.0,405.4` and
/// `200.4,401.7`.
///
/// **And in the middle**, which is the x the launches before the climb measured: two passed the card on one
/// death with a height told and the x left to the errand, and a third died fourteen times at `8.0,432.0`, the
/// field's own bottom-left corner.
///
/// **And for this card and not for rings that close on the run, which a wait this high was tried for.** A card
/// that sends rings out of a boss at the top of the middle and hides and shows them again was waited for at
/// `260` on one no-bomb clear, the strategy wiki's way through it being down with each ring and back up once it
/// is gone — and the run was hit at `14.0,423.5`, the bottom-left corner, on six attempts running, where waiting
/// at the low line it had taken the card with no death at all. `Dodge::Focused` is that card's.
pub(crate) const VISIONARY_TUNING_ABOVE_THE_FLOOR: f32 = 142.0;
