//! **th08's ways**: the cards the rules lost a run on, each with a way of its own — taken up while its card runs and
//! put down when it ends. What each way does and the measurements it holds are `crate::steer`'s, or `crate::cards`'
//! for a way named for its card; what is here is which card wants which, and why that one.

use orb_config::recipe::Dodge;

use crate::Ways;

pub(super) fn normal() -> Ways {
    Ways {
        dodges: vec![
            // **Card 94, stage 4b's `光符「アースライトレイ」`** — the ninth of that boss's attacks, after her Master
            // Spark and before her Last Spell — whose lasers are laid up the field from its floor with an aimed 3-way
            // between them. The strategy wiki starts it at or near an edge (`端あるいは少し端から始めて`) with the
            // 3-way dodged on the outside, and has `中で避けると、ものすごくシビアだが、慣れると安定する` beside it;
            // the rules were placed at `40` and at `96` and died at each of them, and the middle is where they get
            // through.
            (94, Dodge::InTheMiddle),
            // **Card 97, Marisa's `魔砲「ファイナルスパーク」`, stage 4B's Last Spell**, whose laser is aimed where the
            // run is, held there while it warns, fires 240 wide and then turns after the run until it points at it:
            // the strategy wiki's way is to lead it — stand where it aims so that what it turns across is open ground,
            // and keep out from under its root. Read off the enemies every frame, the run stood `83.7` off its middle,
            // clear of the `160` it was read at while it warned and inside the `240` it fired at, and was hit the
            // frame it fired. See `crate::steer::A_FOLLOWING_LASERS_HALF_ACROSS`, and `ahead_of_a_following_laser`
            // beside it for the lead.
            (97, Dodge::OutOfTheFollowingLaser),
            // **Card 105, stage 5's own `狂符「幻視調律（ビジョナリチューニング）」`**, whose lattice below the band it is
            // born in comes down on the run for a hundred frames and is hidden, and harmless, for the next hundred. The
            // strategy wiki's way is down with it and up again while the illusion is on — `どんどん上を意識していこう`,
            // the bottom being where a run is crushed — so the rules climb back up while it is hidden, which they never
            // did waiting high in the middle: see `crate::cards`.
            (105, Dodge::VisionaryTuning),
            // **Card 113, stage 5's own `散符「真実の月（インビジブルフルムーン）」`**, whose rings go out of the boss at
            // the top of the middle, vanish and come back where they have got to — dodged focused through the whole
            // card, the strategy wiki's way through it being small moves down with each ring
            // (`全方位弾が来たらそれに合わせて下がりつつ`) and the rules at their whole step drifting to a wall under it.
            // Waiting high in the middle was tried and lost the card at the bottom-left corner six attempts running,
            // the key never held.
            (113, Dodge::Focused),
            // **Card 120, the last stage's own `天丸「壺中の天地」`**, whose bullets go out from the boss in rings that
            // fill the field and pile against its right: the rules told nothing were walked along the floor to the
            // right wall and died in that corner, so they wait left of the middle.
            (120, Dodge::LeftOfTheMiddle),
            // **Card 128, the last stage's own `蘇活「生命遊戯 －ライフゲーム－」`**, which lays its bullets across the
            // field and then fires them, piled against its left: the strategy wiki has it dodged at the bottom
            // (`最下段で避ける事になる`) and a bomb advised, and the rules told nothing died along the floor, so they
            // wait right of the middle.
            (128, Dodge::RightOfTheMiddle),
            // **Card 144, Eirin's Last Spell `禁薬「蓬莱の薬」`**, which lets point items fall down the column under her
            // that its fire is thickest in: eighteen of the twenty-three whole runs that reached it were hit on it,
            // sixteen at the floor between `x = 143` and `212`, where a Normal clear through its last stretch wanted a
            // point item on 22% of the frames and two practices of it wanted one on 7% and were hit on neither. So the
            // rules go for none there, and wait where their own boxes say.
            (144, Dodge::WithoutTheItems),
            // **And card 152, Kaguya's own `難題「龍の頸の玉　-五色の弾丸-」`**, whose lasers are aimed at the run with
            // fire strewn between them: the rules told nothing were carried to a wall by dodging each laser the same
            // way as the last, and every one of nine clears was hit on it, eight within 47 of a wall. The strategy
            // wiki has it `まともに避けようとすると詰む事もある`, and its way through is side to side between the lasers.
            (152, Dodge::SideToSide),
        ],
        ..Ways::default()
    }
}

pub(super) fn extra() -> Ways {
    Ways {
        dodges: vec![
            // **Mokou's `滅罪「正直者の死」` got through the way it is meant to be**: each of its lasers is drawn off to
            // one side of the run and turned onto where the run stood when it appeared, and goes live there and sweeps
            // on — so the run steps the short way across it towards the side it came from before it goes live, and is
            // behind it for the sweep. See `crate::steer::BEHIND_THE_LASER_BY`; the rules' own dodging ran ahead of it
            // and lost the card 70 times in one session.
            (198, Dodge::BehindTheLaser),
            // **And her `「パゼストバイフェニックス」` gone round**: she leaves the field as a spirit that follows the run
            // and fires from where it is, so the rules waiting under her were waiting where her fire comes from, and
            // one Extra was hit there nine times. The strategy wiki's way through each of its four phases is round the
            // whole field (`画面全体を大回転`).
            (201, Dodge::RoundTheField),
            // **And her `「蓬莱人形」` gone round a small square under her**: its familiars stay on the run and fire from
            // where they are, so a run standing still, or moving at the focused half step, is standing where they fire
            // — three hits of one Extra came with nothing in the model and those familiars seven to fourteen units
            // off. The strategy wiki's way through it is `妹紅の前で小さく時計回りにちょん避け(四角く→↓←↑)`. See
            // `crate::steer::round_a_small_square`.
            (202, Dodge::RoundASmallSquare),
            // **And her Last Spell `「インペリシャブルシューティング」` dodged by its own schedule**, a pattern the same
            // every launch: its eight waves come at the same frames of the card each time, and the wiki's way through
            // the last is from the bottom-right or bottom-left corner. See `crate::cards`.
            (204, Dodge::ImperishableShooting),
        ],
        ..Ways::default()
    }
}
