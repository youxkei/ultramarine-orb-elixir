//! **What 完全無欠ボムモード adds to Pointdevice mode**: the bomb button judged at every hit, and lives that
//! a hit taken without it costs a part of.
//!
//! Pointdevice mode takes away the reflex of bombing when about to be hit. A hit costs one press of the retry
//! menu, and a run aimed at a flawless clear has no reason to spend a bomb, so the habit of reaching for the
//! button wears off. This mode keeps the rewind on every hit and judges the hit by whether the button went
//! down just before it — from [`BEFORE_THE_HIT_FRAMES`] before the hit to the end of the game's own deathbomb
//! window. A hit without that press costs the [`Difficulty`]'s part of a life, and the parts running a run
//! out of lives is the game's own game over.
//!
//! **So one press of the button sets no bomb off**, which is the shape of the whole mode: a bomb that went
//! off on the press would take the hit away, and with it the very thing being judged. The press is taken out
//! of the word the game reads — see [`Button`] — and a bomb goes off on a second press within
//! [`DOUBLE_PRESS_FRAMES`] of the first. Mashing the button in a panic therefore spends bombs, which is what
//! keeps mashing from passing for a press in time.
//!
//! What is here is the rules, as functions of frames and buttons, so that a test can drive them. Where they
//! stand in a frame — the input read the button is taken out of, the death the judgement is made on, the
//! restore the lives are carried across — is `runtime`'s.

use std::fmt;

use orb_config::Language;

/// **How many frames apart the two presses of a double press may be**: a quarter of a second at the 60
/// updates a second every game runs at.
///
/// Chosen rather than measured: it has to be short enough that two presses somebody meant as two judged
/// presses — one for each of two hits a second apart — stay two, and long enough that a deliberate double tap
/// lands inside it. A tap of a key is some 6 frames down and 6 up, so two of them come to about 12.
pub const DOUBLE_PRESS_FRAMES: u32 = 15;

/// **How many frames before the hit a press still counts as one in time**, on top of the game's own
/// deathbomb window after it.
///
/// Chosen rather than measured, at the order of the windows themselves — th10's is the 8 frames its player
/// waits in after a hit (`game::th10::PLAYER_STATE`): a press this close before a hit is somebody who saw it
/// coming, and one further back is a press for some other danger. **A fifth of a second**, which is a press
/// made on seeing the bullet that is going to land rather than one made as it lands — the 8 it was left a
/// pair of taps tight for anyone reacting rather than guessing.
pub const BEFORE_THE_HIT_FRAMES: u32 = 12;

/// What a hit taken without the press costs: the part of a life, chosen per run right after the mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Lunatic,
}

/// The four, in the order they are offered — the game's own order for its own difficulties.
pub const DIFFICULTIES: [Difficulty; 4] = [
    Difficulty::Easy,
    Difficulty::Normal,
    Difficulty::Hard,
    Difficulty::Lunatic,
];

/// A life, in the parts the smallest difficulty takes: every part is a whole number of these.
///
/// **Twelfths and not eighths**, since a third is a part a hit costs on Normal: the smallest number both a
/// quarter and a third divide is twelve.
pub const A_LIFE: u8 = 12;

impl Difficulty {
    /// The part of a life a hit without the press costs, in twelfths of one — see [`A_LIFE`] — Easy 3/12,
    /// Normal 4/12, Hard a half and Lunatic a whole life.
    ///
    /// **Chosen for what a hit feels like**, and moved up from what they were: at an eighth on Easy a run
    /// could take eight hits without a life going, which on the panel is a sliver of a star nobody sees and
    /// a risk nobody feels.
    pub fn part(self) -> u8 {
        match self {
            Self::Easy => 3,
            Self::Normal => 4,
            Self::Hard => 6,
            Self::Lunatic => A_LIFE,
        }
    }

    /// What it is called on screen, which is the game's own names for its difficulties in both languages:
    /// every game orb runs prints them in English on its own difficulty select.
    pub fn name(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Lunatic => "Lunatic",
        }
    }

    /// What choosing it means, a line, in whichever language orb's screens are in.
    pub fn aside(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Easy, Language::Japanese) => "押せなかった被弾1回で残機が1/4減ります",
            (Self::Normal, Language::Japanese) => "押せなかった被弾1回で残機が1/3減ります",
            (Self::Hard, Language::Japanese) => "押せなかった被弾1回で残機が1/2減ります",
            (Self::Lunatic, Language::Japanese) => "押せなかった被弾1回で残機が1つ減ります",
            (Self::Easy, Language::English) => "A hit without the press costs 1/4 of a life",
            (Self::Normal, Language::English) => "A hit without the press costs 1/3 of a life",
            (Self::Hard, Language::English) => "A hit without the press costs 1/2 of a life",
            (Self::Lunatic, Language::English) => "A hit without the press costs a whole life",
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
            Self::Lunatic => "lunatic",
        })
    }
}

/// **The bomb button, between the player and the game**: which reads of it the game sees down, and when it
/// was last pressed in a way that counts for a hit.
///
/// A press is the read the button goes down on and not the reads it is held on, so a button held from
/// before the window does not count as pressed in it. The first press of a pair is kept from the game and
/// is the one judged; a second within [`DOUBLE_PRESS_FRAMES`] is handed to the game as the press it reads
/// a bomb off, held for as long as the button is.
#[derive(Clone, Copy, Default, Debug)]
pub struct Button {
    /// Whether it was down on the read before, which is what makes a read a press.
    down: bool,
    /// Whether the game is being handed the button, from a second press until it is let go.
    passing: bool,
    /// The frame of the first press of a pair still waiting for its second.
    waiting: Option<u32>,
    /// The frame of the last press that counts for a hit — the first of a pair.
    judged: Option<u32>,
    /// Whether the read being made is the one a bomb is set off on, which is the second press of a pair
    /// going down.
    a_bomb: bool,
}

impl Button {
    /// Nothing pressed and nothing held, for a static to start from.
    pub const NONE: Self = Self {
        down: false,
        passing: false,
        waiting: None,
        judged: None,
        a_bomb: false,
    };

    /// One read of the button on a stage's frame `frame`, and whether the game is to see it down.
    pub fn read(&mut self, frame: u32, down: bool) -> bool {
        // A frame before one already seen is a restore having put the stage back: nothing pressed in the
        // attempt that was rewound away was pressed in this one.
        if self.waiting.is_some_and(|at| frame < at) || self.judged.is_some_and(|at| frame < at) {
            self.forget();
        }
        self.a_bomb = false;
        if down && !self.down {
            match self.waiting {
                Some(at) if frame - at <= DOUBLE_PRESS_FRAMES => {
                    self.passing = true;
                    self.a_bomb = true;
                    self.waiting = None;
                }
                _ => {
                    self.waiting = Some(frame);
                    self.judged = Some(frame);
                }
            }
        }
        if !down {
            self.passing = false;
        }
        self.down = down;
        self.passing && down
    }

    /// Whether the read just made is the one a bomb goes off on — see [`Button::read`], which is the only
    /// thing that sets it.
    pub fn a_bomb(&self) -> bool {
        self.a_bomb
    }

    /// The frame of the last press that counts for a hit, if there has been one in this attempt.
    pub fn judged(&self) -> Option<u32> {
        self.judged
    }

    /// Forgets every press, for a stage left or put back: what is held stays held, so a button still down
    /// across it is not a press.
    pub fn forget(&mut self) {
        *self = Self {
            down: self.down,
            ..Self::default()
        };
    }
}

/// **A bomb pressed the way this mode sets one off**, for a player that presses the bomb once and expects it
/// to go: the press it makes, let go for a frame, and pressed again, so the second press is within
/// [`DOUBLE_PRESS_FRAMES`] of the first.
///
/// For the bot, which knows nothing of the mode: a bomb it reaches for is a bomb it means, and one press of it
/// would set none off — a real case played by it would never bomb at all.
#[derive(Clone, Copy, Debug)]
pub struct Twice {
    /// Whether the player held the button on the frame before.
    held: bool,
    /// The frame the pair being pressed began on.
    from: Option<u32>,
}

impl Twice {
    pub const NONE: Self = Self {
        held: false,
        from: None,
    };

    /// Whether the button is to be down on the stage's frame `frame`, for a player holding it as `wanted`.
    pub fn down(&mut self, frame: u32, wanted: bool) -> bool {
        // A frame put back by a restore ends whatever pair was being pressed.
        if self.from.is_some_and(|from| frame < from) {
            self.from = None;
        }
        if wanted && !self.held && self.from.is_none() {
            self.from = Some(frame);
        }
        self.held = wanted;
        let Some(from) = self.from else {
            return wanted;
        };
        let down = match frame - from {
            0 | 2 => true,
            1 => false,
            _ => wanted,
        };
        if frame - from >= 2 && !wanted {
            self.from = None;
        }
        down
    }
}

/// Whether a press on frame `pressed` counts for the hit that landed on frame `hit` and was settled on frame
/// `settled`: from [`BEFORE_THE_HIT_FRAMES`] before the hit to the end of the deathbomb window.
pub fn in_time(pressed: Option<u32>, hit: u32, settled: u32) -> bool {
    pressed.is_some_and(|at| at + BEFORE_THE_HIT_FRAMES >= hit && at <= settled)
}

/// **How much of a bomb a bomb with nothing to answer costs**, in quarters of one: the game takes a whole bomb
/// for every one that goes off, and what the mode judges is the bomb that answered nothing — so the game's bomb
/// is put back and the quarter is what the run owes, four of them making a bomb.
///
/// **Because a bomb with nothing to answer is free otherwise**, and the button is then one to bash: the mode's
/// press is worth something only for the hit it answers, and a bomb for nothing is a bomb the mode has no
/// reason to judge.
pub const A_BOMB_IN_PARTS: u8 = 4;

/// **The parts of a life the run owes**, which a whole life is taken out of each time they add up to one.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Owed {
    /// In twelfths of a life — see [`A_LIFE`] — and always fewer than one.
    parts: u8,
}

impl Owed {
    /// What the run already owes, for a run picked up where a launch left it.
    pub fn of(parts: u8) -> Self {
        Self {
            parts: parts % A_LIFE,
        }
    }

    /// A hit taken without the press, on `difficulty`: whether the parts came to a whole life, which is then
    /// the life taken.
    pub fn a_hit(&mut self, difficulty: Difficulty) -> bool {
        self.parts += difficulty.part();
        let whole = self.parts >= A_LIFE;
        self.parts %= A_LIFE;
        whole
    }

    /// The parts owed, in twelfths.
    pub fn parts(self) -> u8 {
        self.parts
    }
}

impl fmt::Display for Owed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{A_LIFE}", self.parts)
    }
}

/// **What the bomb mode keeps of a run across its rewinds**: the lives it has, what it owes towards the next
/// one, and where the last hit landed.
///
/// **The lives orb's and not the snapshot's**, because a restore puts back the lives the chapter began with:
/// read off the game at each chapter's start, a whole life taken off them where the parts come to one, and
/// written back over what each restore put there. Not read off the game after a hit instead, which is the
/// obvious place: the game takes its own life for the death on a frame of its own choosing, before or after
/// the one orb settles the hit on, and a life read there is one taken twice or not at all.
#[derive(Clone, Copy, Default, Debug)]
pub struct Run {
    /// The lives the run has, and `None` until they have been read off a settled player — at the run's
    /// start, and again after the game's own game over has put a continue's back.
    pub lives: Option<i8>,
    pub owed: Owed,
    /// The stage's frame the player was last hit on, which a press is judged against.
    pub hit_at: Option<u32>,
    /// The chapter the lives were last read at the start of, as the stage and the frame it began on.
    pub chapter: Option<(i32, u32)>,
    /// **The bombs the run has**, kept the way the lives are and for the same reason: the game takes a whole
    /// one for every bomb that goes off, and the mode charges a quarter of one only for a bomb that answered
    /// nothing — see [`A_BOMB_IN_PARTS`] and [`Run::the_bomb_is_over`].
    pub bombs: Option<i8>,
    /// The quarters of a bomb the run has spent on bombs that answered nothing.
    pub bomb_parts: u8,
    /// The frame a bomb went off on, and whether a hit landed under it.
    pub bomb_at: Option<(u32, bool)>,
}

/// What came of a bomb's window ending, in the bomb mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BombOver {
    /// Whether a hit landed under the bomb, which is the bomb answering it and costing nothing.
    pub answered: bool,
    /// Whether the quarters this one came to made a whole bomb, which the run has to give.
    pub a_whole_bomb: bool,
}

/// What came of a hit the game settled as a death, in the bomb mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Judged {
    /// The button was pressed in time: the chapter is retried, and nothing is owed.
    Pressed,
    /// It was not, and the parts owed did not come to a life.
    Owed,
    /// It was not, and they came to a life, which the run had to give.
    ALife,
    /// They came to a life the run did not have: the game's own game over.
    GameOver,
}

impl Run {
    /// A hit landing, on the stage's frame `frame`.
    pub fn hit(&mut self, frame: u32) {
        self.hit_at = Some(frame);
        // **A hit under a bomb is that bomb answering it**, whatever order the two are seen in.
        if let Some((_, answered)) = self.bomb_at.as_mut() {
            *answered = true;
        }
    }

    /// **A bomb the button has just set off**, on frame `frame`, which is judged when its window ends — see
    /// [`Run::the_bomb_is_over`]. Nothing is owed for it yet: a bomb that answers a hit costs nothing, and a hit
    /// that has landed and not been judged is one this bomb is answering — the game's own deathbomb takes it
    /// back within the bomb's window.
    pub fn a_bomb(&mut self, frame: u32) {
        let answered = self.hit_at.is_some();
        self.bomb_at = Some((frame, answered));
    }

    /// **The window of the bomb that went off is over**: whether a hit landed under it, and whether the quarter
    /// it cost where none did came to a whole bomb, which is then the bomb the run has to give.
    pub fn the_bomb_is_over(&mut self) -> Option<BombOver> {
        let (_, answered) = self.bomb_at.take()?;
        if answered {
            return Some(BombOver {
                answered,
                a_whole_bomb: false,
            });
        }
        self.bomb_parts += 1;
        let a_whole_bomb = self.bomb_parts >= A_BOMB_IN_PARTS;
        if a_whole_bomb {
            self.bomb_parts = 0;
            if let Some(bombs) = self.bombs.as_mut() {
                *bombs = bombs.saturating_sub(1);
            }
        }
        Some(BombOver {
            answered,
            a_whole_bomb,
        })
    }

    /// The hit the game has just settled as a death on frame `settled`, judged against the button's last
    /// counted press.
    pub fn judge(&mut self, pressed: Option<u32>, settled: u32, difficulty: Difficulty) -> Judged {
        let hit = self.hit_at.take().unwrap_or(settled);
        if in_time(pressed, hit, settled) {
            return Judged::Pressed;
        }
        // **Nought lives is nothing to shave a part off**, whatever the difficulty: the run's next hit without
        // the press is its last. Waiting for the parts to come to a whole life first was a Normal run at nought
        // taking four such hits to end.
        if self.lives == Some(0) {
            self.lives = None;
            return Judged::GameOver;
        }
        if !self.owed.a_hit(difficulty) {
            return Judged::Owed;
        }
        match self.lives {
            Some(lives) if lives > 0 => {
                self.lives = Some(lives - 1);
                Judged::ALife
            }
            // A run whose lives were never read is a run nobody has seen settled: the game's own count is
            // what it has, and nought is what is left of it.
            _ => {
                self.lives = None;
                Judged::GameOver
            }
        }
    }

    /// A settled frame of the run, in chapter `chapter` — its stage and the frame it began on — with the game
    /// saying `lives`: read where a chapter has begun since the last read, or where nothing has been.
    ///
    /// **And where the game's count has gone up**, which is an extend: the parts owed are unchanged, so the
    /// star a part is gone from moves to the one gained. Left as it was, the shaved star stayed where the old
    /// count put it — the last of what the run had — and the new one came after it, a whole star to the right
    /// of a shaved one with a shaved one in the middle of the row.
    pub fn settled(&mut self, chapter: (i32, u32), lives: i8, bombs: i8) {
        // **A hit that has landed and left no death behind is one the game's own bomb took back**, and it is
        // not a hit waiting to be judged any more: left pending, every bomb fired afterwards was free — a
        // deathbomb taken back by the game inside its own window, then a press of the button a second later
        // that cost nothing for the rest of the attempt.
        self.hit_at = None;
        let extended = self.lives.is_some_and(|held| lives > held);
        if self.lives.is_none() || self.chapter != Some(chapter) || extended {
            self.lives = Some(lives);
            // **The bombs with them**, at the same moments and for the same reason: the run's count is what
            // the game is written back to, a bomb it took for one that answered nothing being put back.
            self.bombs = Some(bombs);
            self.chapter = Some(chapter);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        A_LIFE, BEFORE_THE_HIT_FRAMES, BombOver, Button, DIFFICULTIES, DOUBLE_PRESS_FRAMES,
        Difficulty, Judged, Owed, Run, Twice, in_time,
    };
    use orb_config::Language;

    /// Presses the button down on `at` and lets it go two reads later, saying whether the game saw it down
    /// on any of them.
    fn tap(button: &mut Button, at: u32) -> bool {
        let seen = button.read(at, true) | button.read(at + 1, true);
        button.read(at + 2, false);
        seen
    }

    /// One press sets no bomb off: the game never sees the button down, and the press is the one judged.
    #[test]
    fn one_press_is_kept_from_the_game_and_judged() {
        let mut button = Button::default();
        assert!(!tap(&mut button, 100));
        assert_eq!(button.judged(), Some(100));
    }

    /// A second press within the double press's frames is handed to the game for as long as it is held, and
    /// the first of the two stays the one judged.
    #[test]
    fn a_second_press_soon_after_is_the_bomb() {
        let mut button = Button::default();
        tap(&mut button, 100);
        assert!(button.read(100 + DOUBLE_PRESS_FRAMES, true));
        assert!(
            button.read(101 + DOUBLE_PRESS_FRAMES, true),
            "held, it stays down"
        );
        assert!(!button.read(102 + DOUBLE_PRESS_FRAMES, false));
        assert_eq!(button.judged(), Some(100));
    }

    /// A second press later than that is a first press of its own: kept from the game, and judged.
    #[test]
    fn a_second_press_late_is_another_first_press() {
        let mut button = Button::default();
        tap(&mut button, 100);
        assert!(!tap(&mut button, 101 + DOUBLE_PRESS_FRAMES));
        assert_eq!(button.judged(), Some(101 + DOUBLE_PRESS_FRAMES));
    }

    /// Mashing goes off: a third press after a bomb is the first of a pair again, and the fourth is the next
    /// bomb.
    #[test]
    fn mashing_spends_bombs() {
        let mut button = Button::default();
        let seen: Vec<bool> = (0..4)
            .map(|press| tap(&mut button, 100 + press * 4))
            .collect();
        assert_eq!(seen, [false, true, false, true]);
    }

    /// A button held from before is not a press: only the read it goes down on is.
    #[test]
    fn a_button_held_is_not_pressed_again() {
        let mut button = Button::default();
        button.read(100, true);
        for frame in 101..200 {
            assert!(!button.read(frame, true));
        }
        assert_eq!(button.judged(), Some(100));
    }

    /// A restore puts the stage's frame back, and nothing pressed in the attempt rewound away counts in the
    /// one after it.
    #[test]
    fn a_frame_put_back_forgets_the_presses() {
        let mut button = Button::default();
        tap(&mut button, 500);
        button.read(300, false);
        assert_eq!(button.judged(), None);
        assert!(
            !tap(&mut button, 301),
            "and the next press is a first press"
        );
    }

    /// A bomb pressed once, through the doubling, reaches the game as a bomb: the button goes down, up and down
    /// again, and the gate hands the second press over.
    #[test]
    fn a_bomb_pressed_once_is_pressed_twice() {
        let mut twice = Twice::NONE;
        let mut button = Button::default();
        let wanted = [false, true, false, false, false, false];
        let pressed: Vec<bool> = wanted
            .iter()
            .enumerate()
            .map(|(at, wanted)| twice.down(100 + at as u32, *wanted))
            .collect();
        assert_eq!(pressed, [false, true, false, true, false, false]);
        let seen: Vec<bool> = pressed
            .iter()
            .enumerate()
            .map(|(at, down)| button.read(100 + at as u32, *down))
            .collect();
        assert_eq!(seen, [false, false, false, true, false, false]);
    }

    /// Held, the bomb stays held after its second press, and a press put back by a restore starts over.
    #[test]
    fn a_bomb_held_is_held_after_its_second_press() {
        let mut twice = Twice::NONE;
        let pressed: Vec<bool> = (0..5).map(|at| twice.down(100 + at, true)).collect();
        assert_eq!(pressed, [true, false, true, true, true]);
        twice.down(105, false);
        assert!(twice.down(50, true), "a frame put back");
        assert!(!twice.down(51, true));
    }

    /// **And a fifth of a second before the hit is the window**, 12 frames: twelve counts, thirteen does not —
    /// see [`BEFORE_THE_HIT_FRAMES`], which goes from 8 to 12 so that a press made on seeing the bullet that is
    /// going to land counts.
    #[test]
    fn a_press_counts_from_just_before_the_hit_to_the_window_closing() {
        assert_eq!(BEFORE_THE_HIT_FRAMES, 12);
        let hit = 1000;
        let settled = 1008;
        assert!(
            in_time(Some(hit - 12), hit, settled),
            "twelve frames before"
        );
        assert!(
            !in_time(Some(hit - 13), hit, settled),
            "thirteen frames before"
        );
        assert!(in_time(Some(hit), hit, settled));
        assert!(in_time(Some(settled), hit, settled));
        assert!(!in_time(Some(settled + 1), hit, settled));
        assert!(!in_time(None, hit, settled));
    }

    /// **The quarters of a bomb a bomb with nothing to answer costs come to a whole one**, which the run gives:
    /// four bombs with no hit under them, and the fifth costs nothing, the count having gone down.
    #[test]
    fn four_bombs_with_nothing_to_answer_cost_one() {
        let mut run = Run::default();
        run.settled((0, 100), 2, 3);
        let mut over: Vec<BombOver> = Vec::new();
        for frame in 0..4 {
            run.a_bomb(1000 + frame * 100);
            over.push(run.the_bomb_is_over().expect("a bomb to judge"));
        }
        assert!(over.iter().all(|it| !it.answered));
        assert_eq!(
            over.iter().filter(|it| it.a_whole_bomb).count(),
            1,
            "the quarters did not come to one bomb"
        );
        assert!(over.last().unwrap().a_whole_bomb, "the fourth is the one");
        assert_eq!(run.bombs, Some(2), "the bomb the quarters came to");
        // And a bomb a hit landed under costs nothing, whatever the quarters were.
        run.a_bomb(2000);
        run.hit(2002);
        let answered = run.the_bomb_is_over().expect("a bomb to judge");
        assert!(answered.answered);
        assert!(!answered.a_whole_bomb);
        assert_eq!(run.bomb_parts, 0, "an answered bomb spent a quarter");
    }

    /// Each difficulty takes its own part of a life per hit, and a whole life is taken when the parts add up
    /// to one: four hits on Easy, three on Normal, two on Hard, one on Lunatic.
    #[test]
    fn a_life_goes_when_the_parts_add_up_to_one() {
        for (difficulty, hits) in [
            (Difficulty::Easy, 4),
            (Difficulty::Normal, 3),
            (Difficulty::Hard, 2),
            (Difficulty::Lunatic, 1),
        ] {
            let mut owed = Owed::default();
            let taken: Vec<bool> = (0..hits * 2).map(|_| owed.a_hit(difficulty)).collect();
            let whole: Vec<usize> = (0..taken.len()).filter(|at| taken[*at]).collect();
            assert_eq!(whole, [hits - 1, hits * 2 - 1], "{difficulty}");
            assert_eq!(owed.parts(), 0);
        }
    }

    /// The parts owed are what a run picked up starts with, and never a whole life.
    #[test]
    fn what_is_owed_is_kept_below_a_life() {
        assert_eq!(Owed::of(3).parts(), 3);
        assert_eq!(Owed::of(A_LIFE + 1).parts(), 1);
        assert_eq!(Owed::of(6).to_string(), "6/12");
    }

    /// **An extend moves the star a part is gone from to the one gained**, which is what keeps the shaved star
    /// the last of the row: the run's count follows the game's up, and the parts it owes are unchanged.
    #[test]
    fn an_extend_moves_the_shaved_star_to_the_one_gained() {
        let mut run = Run::default();
        run.settled((0, 100), 2, 3);
        run.owed.a_hit(Difficulty::Normal);
        assert_eq!(run.lives, Some(2));
        run.settled((0, 100), 3, 3);
        assert_eq!(run.lives, Some(3), "the life gained was not taken up");
        assert_eq!(run.owed.parts(), 4, "the parts the run owes were swept");
        // And a count that went down is not the game's to say: what a hit took stays taken.
        run.settled((0, 100), 1, 3);
        assert_eq!(run.lives, Some(3), "the game's own count took a life back");
    }

    /// A hit with the press in time costs nothing; one without costs the part, and a life where the parts come
    /// to one.
    #[test]
    fn a_hit_is_judged_by_the_press_before_it() {
        let mut run = Run::default();
        run.settled((0, 100), 2, 3);

        run.hit(1000);
        assert_eq!(
            run.judge(Some(995), 1008, Difficulty::Hard),
            Judged::Pressed
        );
        run.hit(2000);
        assert_eq!(run.judge(Some(995), 2008, Difficulty::Hard), Judged::Owed);
        run.hit(3000);
        assert_eq!(run.judge(None, 3008, Difficulty::Hard), Judged::ALife);
        assert_eq!(run.lives, Some(1));
        assert_eq!(run.owed.parts(), 0);
    }

    /// The parts coming to a life the run does not have is the game over.
    #[test]
    fn a_life_the_run_has_not_got_is_the_game_over() {
        let mut run = Run::default();
        run.settled((0, 100), 0, 3);
        run.hit(1000);
        assert_eq!(run.judge(None, 1008, Difficulty::Lunatic), Judged::GameOver);
        assert_eq!(
            run.lives, None,
            "read again once the game has put a continue's back"
        );
    }

    /// At nought lives the next hit without the press is the game over on every difficulty, and not once the
    /// parts come to a whole life: there is no star left to take a part of.
    #[test]
    fn at_nought_lives_one_hit_without_the_press_is_the_game_over() {
        for difficulty in DIFFICULTIES {
            let mut run = Run::default();
            run.settled((0, 100), 0, 3);
            run.hit(1000);
            assert_eq!(
                run.judge(None, 1008, difficulty),
                Judged::GameOver,
                "{difficulty}"
            );
        }
        // And a press in time is still a hit that costs nothing.
        let mut run = Run::default();
        run.settled((0, 100), 0, 3);
        run.hit(1000);
        assert_eq!(
            run.judge(Some(1000), 1008, Difficulty::Easy),
            Judged::Pressed
        );
    }

    /// The lives are read at a chapter's start and not again inside it, so a life the game took for a death
    /// or put back with a restore is not what the run has.
    #[test]
    fn the_lives_are_read_at_a_chapters_start() {
        let mut run = Run::default();
        run.settled((0, 100), 3, 3);
        run.settled((0, 100), 2, 3);
        assert_eq!(run.lives, Some(3), "the same chapter");
        run.settled((0, 900), 4, 3);
        assert_eq!(run.lives, Some(4), "a chapter begun since");
    }

    /// The four are offered Easy to Lunatic, as the game offers its own, and each says its own part.
    #[test]
    fn the_difficulties_are_offered_easy_to_lunatic() {
        assert_eq!(DIFFICULTIES[0], Difficulty::Easy);
        assert_eq!(DIFFICULTIES[3], Difficulty::Lunatic);
        for language in [Language::Japanese, Language::English] {
            let said: std::collections::BTreeSet<&str> =
                DIFFICULTIES.iter().map(|it| it.aside(language)).collect();
            assert_eq!(said.len(), DIFFICULTIES.len());
        }
        assert!(Difficulty::Easy.aside(Language::Japanese).contains("1/4"));
        assert!(Difficulty::Normal.aside(Language::Japanese).contains("1/3"));
        assert!(Difficulty::Hard.aside(Language::Japanese).contains("1/2"));
        assert!(
            Difficulty::Lunatic
                .aside(Language::Japanese)
                .contains("1つ減ります")
        );
        assert!(
            Difficulty::Lunatic
                .aside(Language::English)
                .contains("whole life")
        );
    }
}
