//! **The cases the real game is tested with**: a recipe orb plays a launch to, and the check the trace that
//! launch writes has to pass.
//!
//! An e2e test over a laid-out game can say everything about orb but one thing — that the layout is the
//! game's. A run of the real game is the measurement that says so, and until now it was read by a person off
//! `orb.log`, which is a measurement nobody repeats. `--autoplay` plays the run without anybody at the
//! keyboard and `--trace` writes down what every frame held; this crate is the list of what to play and what
//! the trace then has to show, so that `cargo xtask real <copy of the install>` is the measurement.
//!
//! **A check reads the trace's fields and not the log's sentences wherever a field says it.** The chapter, the
//! retries, the question up and the game's own state are fields; a line is read only for what no field
//! carries — which item answered a question, and how the run was left — and then by its opening words, which
//! are the log's own names for those things.
//!
//! **The same cases are played against the laid-out games in `orb-e2e`**, which is what keeps a recipe
//! honest before it costs six minutes of the real game: a step that never reaches its point, or a check that
//! asks for something no launch shows, fails there first.
//!
//! **Except the two of th06nc's that play a run to its clear**, which the laid-out th06nc cannot yet play:
//! `orb-e2e`'s `the_cases_of_th06nc_the_real_game_is_tested_with` names what it gates and what it does
//! not, and the fake owes an ending, a staff roll, a ranking and an Extra before those two can be played
//! there. So what a wrong step in one of them still costs is a launch of the real thing.
//!
//! **th08's are gated there like every other game's, clears and Extra included**, in
//! `the_cases_of_th08_the_real_game_is_tested_with`, one test a chain: its first two took 35 seconds of the
//! laid-out game where the real one took eight minutes, and the second of them was written wrong once — a
//! check looking for a line orb writes on a hit rather than per frame, which the real game took seven
//! minutes to refuse.
//!
//! **One case can carry on from the one before it.** A pick-up needs a run left unfinished, and the launch
//! that leaves one is a case of its own; [`Case::continues`] says the two share the directory orb keeps its
//! files in **and the game's own records**, so the second finds both what the first left for it and what the
//! game says that run unlocked. Everything else starts from neither — a directory of its own, and the
//! records taken aside — so that which difficulty a select comes up on and which item a title menu is on are
//! the same every time a case is played. Nothing in the copy is deleted for it; see `cargo xtask real`.
//!
//! **A case plays the game the way somebody does**, which is `orb_bot`'s own rule and not this list's: the
//! word goes in where a run's own recorded words go, the game's screens are pressed at rather than written
//! past, and nothing makes the player unhittable. What follows here is the one thing that is this list's —
//! **no case asks for `--clear`**, which suppresses the very records a case reads: it refuses the replay
//! write, turns the resume off, and leaves an ending nobody played to. What reaches an ending is
//! [`checks::a_run_played_to_its_clear`], the run played out.

use orb_core::trace::Record;

/// Which game a case is written for: the recipes name the same buttons and questions in both, and the front
/// end each walks is its own game's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Game {
    Th06,
    Th07,
    /// **And 「東方紅魔郷: Classic」, the Steam release of 東方紅魔郷**, a 64-bit game with the same sixteen cases as the two above:
    /// `orb_core::game::th06c` rewinds, so a run of it is picked up, a chapter of it taken again and its
    /// score file forked the way theirs are. Its front end is walked by its own screen numbers — see its
    /// recipes.
    Th06c,
    /// **And the other Steam release, 「東方紅魔郷: New Classic」**, on th06c's engine with every address its
    /// own — see `orb_core::game::th06nc`, which rewinds now. Its cases are its own list all the same:
    /// every case of that list about a chapter inside a stage wants a boundary its midstage table has none
    /// of yet.
    Th06nc,
    /// **And 東方永夜抄, Imperishable Night**, which `orb_core::game::th08` rewinds: its cases are every
    /// case the rewinding games have under this game's own recipes, with the clears and the Extra its own
    /// chain wants — see [`cases_of`].
    Th08,
}

impl Game {
    /// The directory a game's recipes are under, and the name `cargo xtask real` takes for it.
    pub fn name(self) -> &'static str {
        match self {
            Game::Th06 => "th06",
            Game::Th07 => "th07",
            Game::Th06c => "th06c",
            Game::Th06nc => "th06nc",
            Game::Th08 => "th08",
        }
    }

    /// The exe the game runs as, which is what says a copy of the install is this game's, and what a launch
    /// is waited on by. The same names as `orb_core::game::KNOWN` carries, which is where they are read off
    /// the exe; here because a check has no game in memory to ask.
    pub fn exe(self) -> &'static str {
        match self {
            Game::Th06 => "東方紅魔郷.exe",
            Game::Th07 => "th07.exe",
            Game::Th06c => "th06c.exe",
            Game::Th06nc => "th06nc.exe",
            Game::Th08 => "th08.exe",
        }
    }

    /// **The records the game keeps of its own, by the names it keeps them under**: its score file, the
    /// configuration beside it, and the file orb forks the first into while it is in pointdevice mode.
    ///
    /// What reads them is `cargo xtask real`, which takes them out of the copy before a case that starts
    /// from nothing — so that which difficulty a select comes up on, which item the title menu is on, and
    /// what the game says has been unlocked are the same for that case every time it is played, rather than
    /// whatever the case before it left. The same names as `orb_core::game::KNOWN` and `orb_core::score`
    /// carry, here for the reason [`Game::exe`] is: the session that moves them has no game in memory to
    /// ask.
    pub fn records(self) -> [&'static str; 3] {
        [
            "score.dat",
            match self {
                Game::Th06 => "東方紅魔郷.cfg",
                Game::Th07 => "th07.cfg",
                Game::Th06c | Game::Th06nc => "th06.cfg",
                Game::Th08 => "th08.cfg",
            },
            "pointdevice_score.dat",
        ]
    }

    /// **The target orb has to be built for to attach to this game**, which is the architecture the game's
    /// own exe is: th06 and th07 are 32-bit processes and th06c is a 64-bit one, and a build of orb carries
    /// the games of its own architecture and no others — `orb_core::game::KNOWN` is gated on it.
    ///
    /// **Read by `cargo xtask real`, which builds and installs per game rather than once.** It built one
    /// exe for every case before this, which is right while every game is the same architecture and puts a
    /// 32-bit orb beside a 64-bit game the moment one is not — an injection into nothing.
    ///
    /// The same two strings `xtask`'s own `TARGET` and the 64-bit build use, here for the reason
    /// [`Game::exe`] is: this crate is the one place which games there are is written down.
    pub fn target(self) -> &'static str {
        match self {
            Game::Th06 | Game::Th07 | Game::Th08 => "i686-pc-windows-gnu",
            Game::Th06c | Game::Th06nc => "x86_64-pc-windows-gnu",
        }
    }

    pub const ALL: [Game; 5] = [
        Game::Th06,
        Game::Th07,
        Game::Th06c,
        Game::Th06nc,
        Game::Th08,
    ];
}

/// **How long an ordinary case's launch may take**, in seconds, before `cargo xtask real` gives up on it: a
/// case is a stage or two at sixty frames a second with every wait of its own bounded, so a launch still
/// running past this is one whose step never gave up.
pub const WITHIN: u32 = 20 * 60;

/// **And what a run played to its clear may take**, which is the one case that plays a whole game: six
/// stages by the rules with every death retried, and then the ending and the staff roll. Its own number
/// rather than a ceiling raised for every case, because a ceiling is what says a launch has stopped getting
/// anywhere — raised to this everywhere, a stuck stage would sit for an hour and a half before the session
/// said so.
pub const A_WHOLE_GAME: u32 = 90 * 60;

/// One launch of the real game and what its trace has to show.
pub struct Case {
    pub game: Game,
    /// The name the recipe file and the trace's directory carry.
    pub name: &'static str,
    /// The recipe itself, `orb-config`'s grammar.
    pub recipe: &'static str,
    /// Whether this launch picks up in the directory the case before it left orb's files in — the run it
    /// left, the score file the mode forked — rather than starting in an empty one.
    pub continues: bool,
    /// How long this launch may take, in seconds — [`WITHIN`] for every case but the one that plays a whole
    /// game.
    pub within: u32,
    /// What the trace of that launch has to show, and why not where it does not.
    pub check: fn(&Trace) -> Result<(), String>,
}

impl Case {
    /// **Whether this case's own run leaves a clear in the game's own records**, which is what the front end
    /// lights its Extra out of: a run of the game's own mode played to the end does, and a run of orb's does
    /// not — a pointdevice clear goes to the mode's own file, see `orb_core::score`.
    ///
    /// **What reads it is the order of the cases and not a case on its own.** `xtask real` takes the game's
    /// records aside before every case that does not continue the one before it, so a clear reaches a later
    /// case only through the cases that continue: whether *this* launch starts from records with a clear in
    /// them is therefore a fold over the chain, which is what `orb-e2e`'s readers do with it and what a
    /// laid-out game has to be told, having no records carried across launches of its own.
    ///
    /// **And it is what th06nc's Extra case failed on for a session**: the clear was two cases earlier with
    /// a pointdevice clear between them, so the records the Extra continued held no unlock and the front end
    /// offered no Extra at all. Moving that case to sit immediately before it is the whole of the fix, and a
    /// fold is what says so where a list of names by hand did not.
    pub fn leaves_a_clear(&self) -> bool {
        matches!(
            self.name,
            "a_normal_mode_run_played_to_its_clear" | "a_run_played_to_its_clear_without_a_bomb"
        )
    }

    /// **How many updates a frame this case is played at**, which is four for a case that plays a whole game
    /// and the game's own one for every other.
    ///
    /// **Read off the bound it already declares** rather than said twice: [`A_WHOLE_GAME`] is the case that
    /// plays every stage with every death retried, and those are the ones a fast-forward is for. A new case
    /// of that size declares that bound to be given the time, and is given the speed with it.
    ///
    /// **And one is the answer for the rest because the trace holds a record a frame.** At four updates a
    /// frame a chapter reached and the run left inside the same frame's four are one record, so the check
    /// reads a case as never having reached its chapter: five of th06c's sixteen came back that way at four
    /// and passed at one. Which is why this is not simply four everywhere, and why `--speed=N` on the
    /// command line — a sweep, one speed for every launch — is a thing somebody asks for and not the
    /// default.
    ///
    /// A whole game at the game's own speed is half an hour of wall clock and the two of them together are
    /// most of a session: measured, `a_normal_mode_run_played_to_its_clear` took 1,739s at one and 459s at
    /// four, and `a_run_played_to_its_clear` 511s at four.
    ///
    /// **And th06nc plays its whole games at one, because its front end repeats a held key.** Those menus
    /// take a press as a repeat and not as one edge — which is why every recipe of that game presses for
    /// six frames — and a press of six *frames* at four updates a frame is twenty-four updates of repeat,
    /// which walks a cursor past the item it was aimed at. Measured: the first launch of
    /// `a_run_played_to_its_clear_without_a_bomb` aimed at the difficulty select's item 1 and reached the
    /// character select reading `diff=2`, and each `wait_until` for a screen gave up its whole bound
    /// because the screens went by between two of its own checks. So this game pays the wall clock.
    pub fn speed(&self) -> u32 {
        match (self.within == A_WHOLE_GAME, self.game) {
            (true, Game::Th06nc) => 1,
            (true, _) => 4,
            (false, _) => 1,
        }
    }
}

/// Every case, in the order they are played: a case that continues comes straight after the one it
/// continues.
pub fn cases() -> Vec<Case> {
    let mut all = Vec::new();
    for game in Game::ALL {
        all.extend(cases_of(game));
    }
    all
}

/// **The cases of one game in chains**: each chain a case that continues nothing, followed by every case
/// after it that continues — see [`Case::continues`], which is the field that makes one.
///
/// **A chain is the unit a session or a test can play on its own**, which is why this exists: the cases in
/// one have to be played in their own order, a case that continues being handed the files the one before it
/// left, and two different chains share nothing at all. So an `orb-e2e` test per chain plays the eleven of
/// them at once where one test for all sixteen played them one after another — 181s of a 181s run was that
/// one test.
pub fn chains_of(game: Game) -> Vec<Vec<Case>> {
    let mut chains: Vec<Vec<Case>> = Vec::new();
    for case in cases_of(game) {
        match (case.continues, chains.last_mut()) {
            (true, Some(chain)) => chain.push(case),
            // A case that continues with nothing before it is a list whose first says it continues, which
            // is a chain of its own rather than a case dropped.
            _ => chains.push(vec![case]),
        }
    }
    chains
}

/// The cases of one game, in order.
///
/// **th06nc's are its own list and not the same one yet.** What that list is about is a run
/// picked up, a chapter taken again, a replay watched and a mode forked — and `Th06nc::rewinds` answers
/// true now, so the snapshot and the forked file both of those stand on are here: one launch of it took
/// nineteen chapter restores without a fault, wrote its run to `pointdevice_resume` and gave the run up
/// to the title.
///
/// **What keeps the list apart is `game/th06nc/chapters.rs`**, whose seven rows are empty — and what
/// that costs is narrower than it reads. A stage of this game is one chapter **until its fight starts**:
/// the boundaries a fight has of its own still chapter it, and a launch of a probe recipe that played
/// into stage 1's boss reached `retry chapter 6 (retry 1)` at stage frame 9,542. So what the empty table
/// costs is the boundaries *before* the fight, and a case that wants a second chapter of a stage wants a
/// recipe that plays as far as the midboss — which arrives later than stage frame 1,925, past where
/// every case here asks. That table is the person at the keyboard's to fill, which is a change of its
/// own; what this list grows by until then is the cases that need no chapter past the one a stage begins
/// in.
pub fn cases_of(game: Game) -> Vec<Case> {
    match game {
        Game::Th06 | Game::Th07 | Game::Th06c => the_cases_of_a_game_orb_rewinds(game),
        Game::Th06nc => the_cases_of_th06nc(),
        Game::Th08 => the_cases_of_th08(),
    }
}

/// **The cases th08 has**, one per recipe in `recipes/th08/` — `ls crates/orb-real/recipes/th08 | wc -l`
/// counts them — of which two are about the seam and the rest about the mode.
///
/// **What the two are about is the seam.** A laid-out th08 can say nothing about whether the addresses are
/// this exe's, and everything orb does to this game goes through them: the two chain runners it stands in
/// front of, its own frame in place of the game's, the read of the keyboard a recipe presses through, the
/// front end's screens, and the arrays a bot is told of. Both of those cases walk that whole path.
///
/// **And neither asks for a stage to be cleared**: on the launches the second was written from, the rules
/// cleared stage 1 with no death on one and lost three lives in it on the next, so a check about the stage
/// reached would have been a case that passes by luck. What it is held to instead is what does not vary —
/// the run entered, the player moved by the rules, the bomb never pressed, and no fault. The clears are
/// cases of their own, further down the list.
///
/// **And the rest are the mode's**: a chapter put back by a death in it — that run started over with its
/// replay kept, and the attempt of the replay the death rewound it from watched — and by the ask that costs
/// none, the game's own pause menu item, which orb takes off and answers with its own question; a run left
/// standing in one for a resume to be offered out of, and that run picked up; a chapter of the stage
/// *after* the first taken again and played past, which is what says the chapters are the stage's own, and
/// the run left there picked up in that stage; a run of the other mode orb keeps nothing of, the whole game
/// played to its clear with no bomb, the Extra played to its clear behind the two runs of the game's own
/// mode that light it, and the question's other three ways: the stage from its start, a chapter further
/// back, and the run given up.
///
/// **The chapters they are over are stage 1's midstage table's and its fight's**, but for the clears':
/// the table's first boundary is script 790, so a case asking for stage 1's second chapter asks for a
/// chapter of the midsection, and its third is the midboss's arrival at 2935.
///
/// **Four chains and the rest start from records taken aside** — see [`Case::continues`]: the run that
/// died and was retried is started over with its replay kept and the attempt it was rewound from watched,
/// one directory throughout; the run left in its second chapter is picked up; the run left in stage 2's
/// fight is picked up there; and Eirin's clear is followed by Kaguya's and the Extra. Every other case is
/// its own.
fn the_cases_of_th08() -> Vec<Case> {
    vec![
        Case {
            game: Game::Th08,
            name: "a_run_started_and_given_up",
            recipe: include_str!("../recipes/th08/a_run_started_and_given_up.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_run_of_th08_started_and_given_up,
        },
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_played_without_a_bomb",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_played_without_a_bomb.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_normal_mode_run_of_th08_played_without_a_bomb,
        },
        // **And the mode's own case, which is the one a chapter of this game is put back in.** A launch
        // of this walk by hand took the chapter, died in it on purpose and came back:
        // `restore: skipping 21 range(s), 21728 bytes` over the three of the sound and the eighteen of
        // the `.anm` slots, six regions put back, and the run playing on for two thousand frames after
        // it with no fault. Which is what `Th08::rewinds` answering true rests on.
        Case {
            game: Game::Th08,
            name: "a_death_answered_by_the_chapter_again",
            recipe: include_str!("../recipes/th08/a_death_answered_by_the_chapter_again.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_death_of_th08_answered_by_the_chapter_again,
        },
        // **And the run that case left, started over from its own beginning with its replay kept**, which is
        // the other half of what the question at the title offers — and the replay it keeps is one of a run
        // that died, which is the replay the mode is for.
        Case {
            game: Game::Th08,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: include_str!("../recipes/th08/the_run_started_over_and_its_replay_kept.yaml"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_of_th08_started_over_and_its_replay_kept,
        },
        // **And the attempt of that replay the death rewound the run from, watched**, off orb's own list
        // behind the title's fifth item: the second chapter's first of two, which nothing but orb's replay
        // keeps. Session 13 watched `attempt 1 of 1` of a run that had died nowhere, which says nothing of
        // an attempt a retry threw away.
        Case {
            game: Game::Th08,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: include_str!("../recipes/th08/an_attempt_of_the_kept_replay_watched.yaml"),
            continues: true,
            within: WITHIN,
            check: checks::an_attempt_a_retry_rewound_away_watched,
        },
        // **And the same chapter asked for without dying for it**, which is the game's own pause menu
        // item orb takes off — the one way into the mode's question that is not a hit. The check is the
        // one the rewinding games use: what it reads is the log's own lines and the run's deaths either
        // side of the ask, and neither is a game's own.
        Case {
            game: Game::Th08,
            name: "a_chapter_taken_again_without_a_death",
            recipe: include_str!("../recipes/th08/a_chapter_taken_again_without_a_death.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        // **And a run left in a chapter for a resume to be offered out of**, which is the same check
        // again.
        Case {
            game: Game::Th08,
            name: "a_run_left_in_its_second_chapter",
            recipe: include_str!("../recipes/th08/a_run_left_in_its_second_chapter.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        // **And that run picked up where it was left**, which a launch of the real game measured before
        // this case was written: a chapter 16,066 frames into stage 1 played back through three
        // conversations and read `the landing is the frame that was written down, field for field`.
        Case {
            game: Game::Th08,
            name: "the_run_picked_up_where_it_was_left",
            recipe: include_str!("../recipes/th08/the_run_picked_up_where_it_was_left.yaml"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        // **And a chapter of the stage after the first, taken again and played past** — which is what says
        // the chapters are the stage's own: a snapshot taken in stage 2 puts the run back into stage 2.
        // The recipe plays past it by reaching the boss, a chapter being cut where she arrives, so the
        // check the rewinding games use finds the boundary it looks for.
        Case {
            game: Game::Th08,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: include_str!("../recipes/th08/a_chapter_of_the_stage_after_the_first.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        // **And the run that case left in stage 2's fight picked up in stage 2**, which is the build a
        // resume answers for pointed at the stage the file names rather than at a run's first.
        Case {
            game: Game::Th08,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: include_str!(
                "../recipes/th08/the_run_picked_up_in_the_stage_it_walked_into.yaml"
            ),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        // **And a run of the other mode, which orb keeps nothing of** — the check the rewinding games use, the
        // lines it reads being orb's own.
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_nothing_is_kept_of.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // **And the whole game played to its clear in the mode orb exists for**, with no bomb pressed and a
        // Last Spell taken — last, because it is the case that costs the real game half an hour.
        Case {
            game: Game::Th08,
            name: "a_run_played_to_its_clear_without_a_bomb",
            recipe: include_str!("../recipes/th08/a_run_played_to_its_clear_without_a_bomb.yaml"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th08_played_to_its_clear_without_a_bomb,
        },
        // **And the Extra, played to its clear with the bomb allowed, behind the two runs that light it** —
        // one chain, in one directory: this game lights Extra Start only out of Kaguya's clear, a run goes to
        // Kaguya only where the records hold Eirin's stage passed, and both have to be the game's own mode,
        // a pointdevice run's records going to the mode's own file. See `asm.md`'s *What a clear writes into
        // a `CLRD` record*.
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_played_to_its_clear",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_played_to_its_clear.yaml"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_of_th08_played_to_its_clear,
        },
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_played_to_kaguyas_clear",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_played_to_kaguyas_clear.yaml"),
            continues: true,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_of_th08_played_to_kaguyas_clear,
        },
        Case {
            game: Game::Th08,
            name: "the_extra_played_to_its_clear",
            recipe: include_str!("../recipes/th08/the_extra_played_to_its_clear.yaml"),
            continues: true,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th08_played_to_its_clear,
        },
        // **And the other three ways the question offers**, whose items are `retry_ui::RetryMenu::new`'s
        // own order: the chapter, then 更に前からやり直す where the chapters behind the lost one are not
        // none, then the stage, then the run given up. So a recipe that asks in the stage's first chapter
        // names the stage with **item 1** and the give-up with **item 2**, and one that asks deeper has
        // four items with the screen of chapters at 1 — which is why the two below ask early and the
        // third plays past the midboss first.
        Case {
            game: Game::Th08,
            name: "the_stage_taken_again_from_its_start",
            recipe: include_str!("../recipes/th08/the_stage_taken_again_from_its_start.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game: Game::Th08,
            name: "the_run_given_up_at_the_retry",
            recipe: include_str!("../recipes/th08/the_run_given_up_at_the_retry.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game: Game::Th08,
            name: "a_chapter_further_back_taken_again",
            recipe: include_str!("../recipes/th08/a_chapter_further_back_taken_again.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **And one stage practised on its own**, the title's fourth item, which a copy with nothing cleared
        // draws and whose stage select offers that copy the first stage.
        Case {
            game: Game::Th08,
            name: "a_stage_practised_on_its_own",
            recipe: include_str!("../recipes/th08/a_stage_practised_on_its_own.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
    ]
}

/// **The cases th06nc has**, one per recipe in `recipes/th06nc/` — `ls crates/orb-real/recipes/th06nc |
/// wc -l` counts them — which is every case the games orb rewinds have and one more: the
/// card counted and kept. The two the rewinding list holds under other names are here under this game's —
/// `a_run_played_to_its_clear_without_a_bomb` for its Normal clear and `the_extra_played_to_its_clear`
/// for the Extra that clear unlocked.
///
/// **Each is a reading of its own**: the run ends where the run ends because the stage's own jobs leave
/// the chain, and a clear is known from the ending's own job arriving. Both are `orb-core`'s
/// `game/th06nc/asm.md`'s.
///
/// **And a list of its own rather than the rewinding games', because this game's midstage table is
/// empty.** `Th06nc::rewinds` answers true and a launch has taken nineteen chapters of a run of it, but
/// `game/th06nc/chapters.rs` has no boundary inside a stage — so every case the rewinding list writes
/// about a chapter *of* a stage is written here about **chapter 1** instead, which a stage's own start is
/// and which a death goes back to. What the table's rows would buy is a boundary in the middle of a
/// stage to retry at, and the cases waiting on that are `orb-e2e`'s rather than these. See [`cases_of`].
///
/// **The Extra continues the clear**, in the directory the clear's launch left its records in: this game
/// offers its Extra only once a run has been cleared, and the records that say so are the score file the
/// first launch pressed through to the title to have written. As with th06c, no laid-out th06nc plays either
/// first.
fn the_cases_of_th06nc() -> Vec<Case> {
    vec![
        // **The mode's own case, and the one chapter 1 alone can reach**: a death answered by the chapter
        // again. Every other case about a chapter of a stage wants a boundary inside one and this game has
        // none until its midstage table is filled — but a death in a stage goes back to that stage's
        // start, which is chapter 1, so the retry, the restore and the run carrying on out of it are all
        // here. Its check is this game's own for one step of it: what says the run went on is the stage's
        // own clock and not a chapter after the one it died in, there being none.
        Case {
            game: Game::Th06nc,
            name: "a_death_answered_by_the_chapter_again",
            recipe: include_str!("../recipes/th06nc/a_death_answered_by_the_chapter_again.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_death_of_th06nc_answered_by_the_chapter_again,
        },
        // **And the run that case left, started over rather than picked up**, which is the other answer
        // the resume question takes: the replay of the run being written over is asked about and kept,
        // and the fresh run plays on from the stage's own start. Its check is th06c's less the wait for a
        // second chapter.
        Case {
            game: Game::Th06nc,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: include_str!("../recipes/th06nc/the_run_started_over_and_its_replay_kept.yaml"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_of_th06nc_started_over_and_its_replay_kept,
        },
        // **And one attempt of that replay watched**, off orb's own four-deep list: this game records no
        // replay a rewound run would fit, so the main menu's item 4 — the replays — is a question of
        // orb's and the press that would open the game's own list is held back. The other games ask over
        // their *title*'s item; the check is theirs unchanged, every line of it being orb's own lists.
        Case {
            game: Game::Th06nc,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: include_str!("../recipes/th06nc/an_attempt_of_the_kept_replay_watched.yaml"),
            continues: true,
            within: WITHIN,
            check: checks::an_attempt_of_the_kept_replay_watched,
        },
        // **The other two answers the same question takes**, which need no boundary inside a stage
        // either: the stage again from its own start — which for this game *is* chapter 1, so what it
        // says over the chapter again is that the item asks before it acts and that the confirmation is
        // answered — and the run given up, which leaves the run for the front end.
        Case {
            game: Game::Th06nc,
            name: "a_chapter_taken_again_without_a_death",
            recipe: include_str!("../recipes/th06nc/a_chapter_taken_again_without_a_death.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_of_th06nc_taken_again_without_a_death,
        },
        Case {
            game: Game::Th06nc,
            name: "the_stage_taken_again_from_its_start",
            recipe: include_str!("../recipes/th06nc/the_stage_taken_again_from_its_start.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::the_stage_of_th06nc_taken_again_from_its_start,
        },
        Case {
            game: Game::Th06nc,
            name: "the_run_given_up_at_the_retry",
            recipe: include_str!("../recipes/th06nc/the_run_given_up_at_the_retry.yaml"),
            continues: false,
            within: WITHIN,
            // **th06c's own check, unchanged**: nothing in it asks about a chapter after the one the run
            // was in, so there is nothing of this game's for it to be wrong about.
            check: checks::the_run_given_up_at_the_retry,
        },
        // **And a chapter further back**, which wants two of a stage's own boundaries behind the one the
        // run is in — the fight's, this game's table being empty. th06c's check unchanged: it plays back
        // *up* to the chapter it was asked in rather than past it, so nothing in it is about a boundary
        // this game has not got.
        Case {
            game: Game::Th06nc,
            name: "a_chapter_further_back_taken_again",
            recipe: include_str!("../recipes/th06nc/a_chapter_further_back_taken_again.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **The run left in its second chapter and picked up there** — the pair a stage's *own* boundary
        // is needed for, which this game has once its fight starts: `game/th06nc/chapters.rs` is empty,
        // so a stage of it is one chapter until then and the fight's boundaries chapter it after — see
        // [`cases_of`]. The recipes therefore wait on the boss before asking for a second chapter, and
        // both checks are th06c's unchanged, every line of them being orb's rather than a game's.
        Case {
            game: Game::Th06nc,
            name: "a_run_left_in_its_second_chapter",
            recipe: include_str!("../recipes/th06nc/a_run_left_in_its_second_chapter.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game: Game::Th06nc,
            name: "the_run_picked_up_where_it_was_left",
            recipe: include_str!("../recipes/th06nc/the_run_picked_up_where_it_was_left.yaml"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        // **The stage after the first, and the run left there picked up in it** — the pair of cases about
        // a stage a run *walked into* rather than started at, which want no boundary inside one: each
        // stage of this game is a chapter from its own start, so the second stage's chapter 1 is reached
        // by playing the first stage out. Each check is th06c's less the wait for a chapter after the one
        // retried, which this game has not got.
        Case {
            game: Game::Th06nc,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: include_str!("../recipes/th06nc/a_chapter_of_the_stage_after_the_first.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_of_th06nc_in_the_stage_after_the_first,
        },
        Case {
            game: Game::Th06nc,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: include_str!(
                "../recipes/th06nc/the_run_picked_up_in_the_stage_it_walked_into.yaml"
            ),
            continues: true,
            within: WITHIN,
            check: checks::the_run_of_th06nc_picked_up_in_the_stage_it_walked_into,
        },
        // **The attempt a retry inside a spell card counts, and the run after it keeping that count** —
        // the case both of orb's dealings with this game's card records want, and the one case here that
        // plays two runs in a single launch: the put-back is spent at the build of a run's first stage,
        // which is the read that would otherwise lose the count.
        Case {
            game: Game::Th06nc,
            name: "an_attempt_at_a_card_counted_and_kept",
            recipe: include_str!("../recipes/th06nc/an_attempt_at_a_card_counted_and_kept.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::an_attempt_at_a_card_of_th06nc_counted_and_kept,
        },
        // **The other mode's own run, which wants no chapter of anything**: a run with no chapters is
        // what the normal mode is, so this case is reachable here while every one about a boundary inside
        // a stage waits on the midstage table. th06c's own check, unchanged — every line it reads is
        // orb's and not a game's.
        Case {
            game: Game::Th06nc,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: include_str!("../recipes/th06nc/a_normal_mode_run_nothing_is_kept_of.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // **One stage practised on its own, which is the main menu's item 2 here** — a run orb rewinds
        // and writes nothing down for, the byte the item sets being both what says practice and what
        // leaves `Game::run_slot` with no name to keep it under. The other games reach it from a
        // practice select of their own; this one reaches it from the menu, so the recipe is this
        // game's while the check is the same one the three games before it pass.
        Case {
            game: Game::Th06nc,
            name: "a_stage_practised_on_its_own",
            recipe: include_str!("../recipes/th06nc/a_stage_practised_on_its_own.yaml"),
            continues: false,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
        // **Six stages in the mode orb exists for**, which answers orb's question with 完全無欠モード and
        // hands the rules a chance of dying in each chapter. See
        // `checks::a_run_of_th06nc_played_to_its_clear` for what the deaths are asserted for.
        Case {
            game: Game::Th06nc,
            name: "a_run_played_to_its_clear",
            recipe: include_str!("../recipes/th06nc/a_run_played_to_its_clear.yaml"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th06nc_played_to_its_clear,
        },
        // **And the same six in the game's own mode, which is the clear the Extra is unlocked by**: this
        // one answers orb's question with レガシーモード and takes no bomb, so what it leaves is a clear in
        // `score.dat` — a pointdevice clear goes to the mode's own file and lights nothing, see
        // `orb_core::score`.
        //
        // **Immediately before the Extra's case and for no other reason**, which is th06c's own order:
        // `xtask real` takes the game's records aside before every case that does not continue the one
        // before it, so the unlock only reaches the Extra where that case is the one it continues. It sat
        // two cases earlier while the Extra's launch continued the pointdevice clear instead — and then the
        // front end offered no Extra at all, three of its `wait_until: { screen: 11 }` steps giving up
        // while the walk went on to start a Normal run.
        Case {
            game: Game::Th06nc,
            name: "a_run_played_to_its_clear_without_a_bomb",
            recipe: include_str!("../recipes/th06nc/a_run_played_to_its_clear_without_a_bomb.yaml"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th06nc_played_to_its_clear_without_a_bomb,
        },
        Case {
            game: Game::Th06nc,
            name: "the_extra_played_to_its_clear",
            recipe: include_str!("../recipes/th06nc/the_extra_played_to_its_clear.yaml"),
            continues: true,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th06nc_played_to_its_clear,
        },
    ]
}

/// The cases of a game orb rewinds, which is th06, th07 and th06c — see [`cases_of`].
fn the_cases_of_a_game_orb_rewinds(game: Game) -> Vec<Case> {
    macro_rules! recipe {
        ($game:expr, $name:literal) => {
            match $game {
                Game::Th06 => include_str!(concat!("../recipes/th06/", $name, ".yaml")),
                Game::Th07 => include_str!(concat!("../recipes/th07/", $name, ".yaml")),
                Game::Th06c => include_str!(concat!("../recipes/th06c/", $name, ".yaml")),
                // Not reachable: [`cases_of`] is what chooses between the three lists, and this one is
                // the branch the other two are not.
                Game::Th06nc => unreachable!("th06nc has its own cases"),
                Game::Th08 => unreachable!("th08 has its own cases"),
            }
        };
    }
    vec![
        Case {
            game,
            name: "a_run_left_in_its_second_chapter",
            recipe: recipe!(game, "a_run_left_in_its_second_chapter"),
            continues: false,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game,
            name: "the_run_picked_up_where_it_was_left",
            recipe: recipe!(game, "the_run_picked_up_where_it_was_left"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        Case {
            game,
            name: "a_death_answered_by_the_chapter_again",
            recipe: recipe!(game, "a_death_answered_by_the_chapter_again"),
            continues: false,
            within: WITHIN,
            check: checks::a_death_answered_by_the_chapter_again,
        },
        // The two that watch what that death left: the run it gave up is what はじめから is answered about,
        // and the replay kept there is the one with two attempts at the chapter it died in.
        Case {
            game,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: recipe!(game, "the_run_started_over_and_its_replay_kept"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept,
        },
        Case {
            game,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: recipe!(game, "an_attempt_of_the_kept_replay_watched"),
            continues: true,
            within: WITHIN,
            check: checks::an_attempt_of_the_kept_replay_watched,
        },
        Case {
            game,
            name: "a_chapter_taken_again_without_a_death",
            recipe: recipe!(game, "a_chapter_taken_again_without_a_death"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        Case {
            game,
            name: "a_chapter_further_back_taken_again",
            recipe: recipe!(game, "a_chapter_further_back_taken_again"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        Case {
            game,
            name: "the_stage_taken_again_from_its_start",
            recipe: recipe!(game, "the_stage_taken_again_from_its_start"),
            continues: false,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game,
            name: "the_run_given_up_at_the_retry",
            recipe: recipe!(game, "the_run_given_up_at_the_retry"),
            continues: false,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: recipe!(game, "a_chapter_of_the_stage_after_the_first"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        // And the run that case left in the stage it walked into, picked up in that stage.
        Case {
            game,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: recipe!(game, "the_run_picked_up_in_the_stage_it_walked_into"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        Case {
            game,
            name: "a_stage_practised_on_its_own",
            recipe: recipe!(game, "a_stage_practised_on_its_own"),
            continues: false,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
        Case {
            game,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: recipe!(game, "a_normal_mode_run_nothing_is_kept_of"),
            continues: false,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // And the three that play whole games, last because they are the cases that cost an hour apiece: the
        // run's own end is reachable no other way, and the Extra behind them is reachable only from a clear.
        Case {
            game,
            name: "a_run_played_to_its_clear",
            recipe: recipe!(game, "a_run_played_to_its_clear"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_run_played_to_its_clear,
        },
        // **The other mode's own clear, which is what unlocks the Extra**: a pointdevice run's records go to
        // the mode's own file and the front end lights its items from the game's, so the clear that puts the
        // record where that read finds it is a run of the other mode — see `orb_core::score`.
        Case {
            game,
            name: "a_normal_mode_run_played_to_its_clear",
            recipe: recipe!(game, "a_normal_mode_run_played_to_its_clear"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_played_to_its_clear,
        },
        // And the Extra that clear unlocked. **The one case that needs what the case before it left in the
        // game's own records** rather than in the directory orb keeps its files in — which is what
        // [`Case::continues`] keeps for it: a case that starts from nothing has the game's records taken
        // aside, and a game with no record of a clear does not offer the Extra at all.
        Case {
            game,
            name: "the_extra_a_cleared_run_unlocked",
            recipe: recipe!(game, "the_extra_a_cleared_run_unlocked"),
            continues: true,
            within: WITHIN,
            check: checks::the_extra_a_cleared_run_unlocked,
        },
    ]
}

/// The name of the file a launch's trace is written to, beside the settings file of the directory orb keeps
/// its files in for that launch.
pub const TRACE_FILE: &str = "trace.msgpack";

/// A trace read back, with the questions a check asks of one.
pub struct Trace {
    pub records: Vec<Record>,
}

impl Trace {
    pub fn read(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self {
            records: orb_core::trace::read(bytes)?,
        })
    }

    /// The index of the first record from `from` on whose lines hold `needle`.
    pub fn line_from(&self, from: usize, needle: &str) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| record.lines.iter().any(|line| line.contains(needle)))
            .map(|at| at + from)
    }

    /// **The index of the first record from `from` on that says an ending ran**, off the log's own words as
    /// well as off the scene.
    ///
    /// **The words, because the scene was where the checks were wrong first**: orb runs an ending out inside
    /// the frame it came up on, so a launch that skips one has no frame whose state is in it at all — a
    /// cleared run of th06 said `ending skipped, 25101 frames run` and every record of that trace read
    /// `in_ending` false. Which of the two lines a run gets is the ending's own — the run-out stops at a
    /// staff roll where there is one — and a launch told not to skip plays the ending instead, which is the
    /// frames the scene is in it for.
    pub fn ending_from(&self, from: usize) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record.lines.iter().any(|line| {
                    line.starts_with("ending run out in") || line.starts_with("ending skipped")
                }) || record.state.is_some_and(|state| state.in_ending)
            })
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on under which `question` stands.
    pub fn question_from(&self, from: usize, question: &str) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| record.question.as_deref() == Some(question))
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on that is inside a run: the game's own state says so.
    pub fn in_run_from(&self, from: usize) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| record.state.is_some_and(|state| state.in_run))
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on that is at or past `chapter` of `stage`, the stage
    /// counted the way a recipe counts it — from one — and the chapter the way the log does.
    pub fn chapter_from(&self, from: usize, stage: i32, chapter: u32) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record.state.is_some_and(|state| {
                    state.in_run && (state.stage + 1, record.chapter) >= (stage, chapter)
                })
            })
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on that is in a run at or past `stage`, the stage counted
    /// the way a recipe counts it — **and nothing said about the chapter**, for a run orb keeps none of:
    /// a run of the normal mode is at chapter nought on every frame of it.
    pub fn stage_from(&self, from: usize, stage: i32) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record
                    .state
                    .is_some_and(|state| state.in_run && state.stage + 1 >= stage)
            })
            .map(|at| at + from)
    }

    /// And the same at or past `chapter` of **whatever stage** the run is in, for a case whose stage is the
    /// game's own to choose: which stage a practice run is played in is whichever one its select came up on.
    pub fn a_chapter_from(&self, from: usize, chapter: u32) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record.state.is_some_and(|state| state.in_run) && record.chapter >= chapter
            })
            .map(|at| at + from)
    }

    /// Every record between two indices, the second left out.
    pub fn between(&self, from: usize, to: usize) -> &[Record] {
        &self.records[from.min(self.records.len())..to.min(self.records.len())]
    }

    /// The lines of every record, with the frame each was said on, for a check that failed to say what the
    /// launch did instead.
    ///
    /// **Without the per-frame ones**, which say the same thing sixty times a second and would bury the line a
    /// reader is after: the frame a playback played, the state line, and a step still waiting. A trace keeps
    /// them; what this is for is reading, and the whole file is there for a check that wants one of them.
    /// **What a launch did that the game it was played on has no say in**, in the order it happened: the
    /// questions orb put up, the screens and items its front end was walked through, and the recipe's own
    /// steps with how each of them ended.
    ///
    /// What it leaves out is everything a laid-out game is free to differ in — how long a stage lasts, how
    /// many chapters it holds, what the generator handed out, what the sound did — so that a laid-out game's
    /// own held against the real game's says whether the same presses reached the same places, and not
    /// whether the two are the same program.
    pub fn shape(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut question = None;
        for record in &self.records {
            let now = record.question.as_deref();
            if now != question {
                if let Some(name) = now {
                    out.push(format!("orb asks {name}"));
                }
                question = now;
            }
            for line in &record.lines {
                // The front end's own line and not the one the verbose log writes beside it, which carries
                // the object's address: what is being compared is the screen and the item, and an address is
                // a thing no two launches of one game agree on either.
                let walked = line.starts_with("front end: screen");
                let stepped =
                    line.starts_with("autoplay: step") && !line.contains("is still waiting");
                if walked || stepped {
                    out.push(line.clone());
                }
            }
        }
        out
    }

    pub fn narrative(&self) -> String {
        const PER_FRAME: [&str; 3] = ["resume: frame ", "state: cur=", "autoplay: still waiting"];
        let mut out = String::new();
        for record in &self.records {
            for line in &record.lines {
                let per_frame = PER_FRAME.iter().any(|prefix| line.starts_with(prefix))
                    || line.starts_with('f') && line.contains(" scene=");
                if !per_frame {
                    out.push_str(&format!("f{} {line}\n", record.frame));
                }
            }
        }
        out
    }
}

/// `Err` with the reason where a condition does not hold, so that a check reads as a list of what has to be
/// true and the reason travels with it.
fn expect(holds: bool, why_not: impl FnOnce() -> String) -> Result<(), String> {
    if holds { Ok(()) } else { Err(why_not()) }
}

/// The checks, one per case.
pub mod checks {
    use super::{Trace, expect};

    /// **The bit th06nc's word carries for the bomb**, which is th06 1.02h's own and the one the recipe's
    /// `bomb` is — `orb_core::game::th06nc`'s `button::BOMB`. Written down here because a check has no game
    /// in memory to ask, the way `Game::exe` has the names.
    const TH06NC_BOMB: u16 = 0x0002;

    /// **The bit th08's word carries for the bomb**, which is `orb_core::game::th08`'s `BUTTON_BOMB` and
    /// the one the recipe's `bomb` is. Written down here because a check has no game in memory to ask,
    /// the way [`super::Game::exe`] has the names.
    const TH08_BOMB: u16 = 0x0002;

    /// **And the four th08's word carries for the directions**, `orb_core::game::th08`'s `PAD_UP` and its
    /// three neighbours: up, down, left and right at `0x10`, `0x20`, `0x40` and `0x80`.
    ///
    /// Read by the check below as what says the rules steered the game at all, which is the one thing
    /// about this game a laid-out one cannot say: a `what_a_bot_sees` that answered nothing would play a
    /// run pressing the shot and no direction, and the word is where that shows.
    const TH08_DIRECTIONS: u16 = 0x0010 | 0x0020 | 0x0040 | 0x0080;

    /// **th08's run started by a recipe and handed back to the game's own title.**
    ///
    /// **Which is the case that says orb can take a run of this game down**, and the first attempt at
    /// that faulted: writing the wanted scene took the game out at 0x406a1a on the frame after, walking
    /// the text element the teardown had freed. What the way out is instead — two bits of the run kind
    /// and the scene the run itself wants — is `orb_core::game::th08`'s `leave_run`, and this is the
    /// launch that holds it.
    ///
    /// **Three things in order**: a run in progress, the log's own line for the give-up, and the front
    /// end reading a screen after it. The last of those is read off a line and not a field because
    /// nothing in `State` carries a front end's screen — `Game::front_end_screen` is what the log prints
    /// — and the first off `State::in_run`, which for this game is the scene being a run's.
    pub fn a_run_of_th08_started_and_given_up(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let given_up = trace
            .line_from(started, "retry: the run is given up")
            .ok_or("the run was never given up")?;
        // **And the game still standing afterwards**, which is what the fault this case is about looked
        // like: the front end's own line comes from a frame past the teardown, so a launch that faulted
        // on the frame after has no such line to find.
        trace
            .line_from(given_up, "front end: screen")
            .ok_or_else(|| "the front end never came up after the run was given up".to_owned())?;
        Ok(())
    }

    /// **th08's Normal run played by the rules with the bomb forbidden**, and held to what does not vary
    /// from launch to launch.
    ///
    /// **Four things**: the run entered on Normal, the stage's own clock advancing under it, the player
    /// moved somewhere other than its own start, and no bomb pressed inside the run. The bomb is read off
    /// the word the rules pressed and not off the stock in hand, for the reason
    /// [`a_run_of_th06nc_played_to_its_clear_without_a_bomb`] gives: a death puts the stock back, so a
    /// stock that went down is not a bomb spent.
    ///
    /// **And not the stage reached**, which is the one thing about this game's bot that is not settled —
    /// see [`super::the_cases_of_th08`].
    pub fn a_normal_mode_run_of_th08_played_without_a_bomb(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        // **The clock and not the frames orb counted**, which is what says the stage is running rather
        // than the launch: `script_frames` of this game is the stage's own count, stepped once a frame
        // with a conversation the only thing that stops it.
        let played = trace.records[started..]
            .iter()
            .filter_map(|record| record.state)
            .map(|state| state.script_frames)
            .max()
            .unwrap_or(0);
        expect(played >= A_STAGE_OF_TH08_IS_UNDER_WAY, || {
            format!("the stage's own clock reached {played} frame(s)")
        })?;
        // **And a direction pressed inside the run**, which is the whole of what says the rules reached
        // this game: a launch whose `what_a_bot_sees` answered nothing would play a run holding the shot
        // and standing still at the (192, 384) a stage puts the player at. Read off the word for the
        // reason the bomb below is — it is the word the rules pressed, and nothing else the trace carries
        // says where the player went.
        let steered = trace.records[started..].iter().any(|record| {
            record.state.is_some_and(|state| state.in_run) && record.word & TH08_DIRECTIONS != 0
        });
        expect(steered, || {
            "the rules pressed no direction inside the run".to_owned()
        })?;
        th08_pressed_no_bomb(trace, started)
    }

    /// **No bomb pressed inside a run of th08 from `started` on**, read off the word the rules pressed.
    fn th08_pressed_no_bomb(trace: &Trace, started: usize) -> Result<(), String> {
        let bombed = trace.records[started..].iter().find(|record| {
            record.state.is_some_and(|state| state.in_run) && record.word & TH08_BOMB != 0
        });
        expect(bombed.is_none(), || {
            format!(
                "a bomb was pressed inside the run, on frame {}",
                bombed.map_or(0, |record| record.frame)
            )
        })
    }

    /// **th08's Normal run in 完全無欠モード, played to its ending with no bomb pressed and a Last Spell
    /// taken on the way** — the whole-run case, which is what the goal it was written for asks a run of this
    /// game: `Normal 1〜6 面をノーボムでクリア`, and `ラストスペルもちゃんと取得`.
    ///
    /// **The ending is what says the run was cleared**: this game's scene 9, which the last boss going down
    /// is the one way into — see `Th08::read_state`'s `in_ending` — or orb's own line for running it out,
    /// which a launch that skips it has in place of any frame in it: see [`Trace::ending_from`].
    ///
    /// **And a Last Spell is taken where an attempt at one ends with no death inside it** — see
    /// [`TH08_LAST_SPELLS`] for which cards those are and [`th08_took_a_last_spell`] for how an attempt is
    /// read. A death in this mode is answered by the chapter again, so what is held is the attempt the run
    /// got through, not the first.
    pub fn a_run_of_th08_played_to_its_clear_without_a_bomb(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        th08_pressed_no_bomb(trace, started)?;
        let ending = trace
            .ending_from(started)
            .ok_or("the run never reached its ending")?;
        th08_took_a_last_spell(trace, started, ending).map(|_| ())
    }

    /// **The Last Spells a Normal run of th08 has met, by their records' own numbers** — which is what
    /// `State::spellcard` answers, the index of the card's `CATK` record — and the name the game wrote into
    /// each record the first time the card was declared, read back out of the mode's own score file:
    /// 10 `隠蟲「永夜蟄居」`, 29 `夜雀「真夜中のコーラスマスター」`, 51 `未来「高天原」`, 97
    /// `魔砲「ファイナルスパーク」`, 116 `月眼「月兎遠隔催眠術」` and 144 `禁薬「蓬莱の薬」`. Stage 4a's is not
    /// here, no run having met it — and 144 is Eirin's, whose stage's norm of nought means a run that beats
    /// her always meets it.
    const TH08_LAST_SPELLS: [u32; 6] = [10, 29, 51, 97, 116, 144];

    /// **A Last Spell taken between `from` and `to`**, and which: an attempt at one of [`TH08_LAST_SPELLS`]
    /// that ended — the card gone or another up — with no `died in chapter` inside it and no `retry chapter`
    /// ending it. Answers the card.
    fn th08_took_a_last_spell(trace: &Trace, from: usize, to: usize) -> Result<u32, String> {
        let said = |at: usize, opening: &str| {
            trace.records[at]
                .lines
                .iter()
                .any(|line| line.starts_with(opening))
        };
        let mut attempt: Option<(u32, usize)> = None;
        let mut taken = None;
        for (at, record) in trace.records.iter().enumerate().take(to).skip(from) {
            let card = record
                .state
                .and_then(|state| state.spellcard)
                .filter(|card| TH08_LAST_SPELLS.contains(card));
            let taken_again = said(at, "retry chapter");
            match attempt {
                Some((was, _)) if card == Some(was) && !taken_again => {}
                Some((was, began)) => {
                    let died = (began..at).any(|inside| said(inside, "died in chapter"));
                    if !died && !taken_again {
                        taken = Some(was);
                    }
                    attempt = card.map(|card| (card, at));
                }
                None => attempt = card.map(|card| (card, at)),
            }
        }
        taken.ok_or_else(|| {
            let met: Vec<u32> = trace.records[from..to]
                .iter()
                .filter_map(|record| record.state.and_then(|state| state.spellcard))
                .filter(|card| TH08_LAST_SPELLS.contains(card))
                .collect::<std::collections::BTreeSet<u32>>()
                .into_iter()
                .collect();
            format!("no Last Spell was taken on the way to the ending; the run met {met:?}")
        })
    }

    /// **th08's Normal run in the game's own mode, played to Eirin's clear with its ending pressed through
    /// to the title** — the first of the two runs the Extra is lit by, see [`th08_played_to_a_last_stage`].
    pub fn a_normal_mode_run_of_th08_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        th08_played_to_a_last_stage(trace, TH08_EIRINS_STAGE)
    }

    /// **And the one after it, played to Kaguya's**, which the records the first one left route it to — and
    /// whose ending is the one that lights the Extra.
    pub fn a_normal_mode_run_of_th08_played_to_kaguyas_clear(trace: &Trace) -> Result<(), String> {
        th08_played_to_a_last_stage(trace, TH08_KAGUYAS_STAGE)
    }

    /// The stage words of th08's two last stages, Eirin's and Kaguya's — `stage6` and `stage7` of the nine.
    const TH08_EIRINS_STAGE: i32 = 6;
    const TH08_KAGUYAS_STAGE: i32 = 7;

    /// **A Normal run of th08 in the game's own mode played to `last`'s clear**: the mode question answered for
    /// the normal mode, the run on Normal, that last stage reached and no other, the ending after it, and the
    /// title up again — which is where the ending hands the records over to be written, and what a clear the
    /// Extra waits on is kept by.
    fn th08_played_to_a_last_stage(trace: &Trace, last: i32) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let answered = trace
            .line_from(asked, "mode: normal")
            .ok_or("the mode question was not answered for the normal mode")?;
        let started = trace
            .in_run_from(answered)
            .ok_or("no run was started after the mode question")?;
        let difficulty = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(difficulty == 1, || {
            format!("the run was started on difficulty {difficulty} and not on Normal")
        })?;
        let stages: std::collections::BTreeSet<i32> = trace.records[started..]
            .iter()
            .filter_map(|record| record.state.filter(|state| state.in_run))
            .map(|state| state.stage)
            .collect();
        let other = if last == TH08_EIRINS_STAGE {
            TH08_KAGUYAS_STAGE
        } else {
            TH08_EIRINS_STAGE
        };
        expect(stages.contains(&last) && !stages.contains(&other), || {
            format!("the run went through stage words {stages:?}, where its last is {last}")
        })?;
        let reached = trace.records[started..]
            .iter()
            .position(|record| {
                record
                    .state
                    .is_some_and(|state| state.in_run && state.stage == last)
            })
            .map(|at| at + started)
            .expect("a stage word the run went through is a record of it");
        let ending = trace
            .ending_from(reached)
            .ok_or("nothing says an ending ran, so the run was not cleared")?;
        trace
            .line_from(ending, "front end: screen 0")
            .map(|_| ())
            .ok_or_else(|| "the ending was never pressed through to the title".to_owned())
    }

    /// **th08's Extra, played to its clear in 完全無欠モード with the bomb allowed** — the goal it was
    /// written for being `Extra をボムありでクリアする`.
    ///
    /// **Cleared is the run ending on its own**: in this mode a death is answered by the chapter again, so a
    /// run of it ends where its last boss goes down or where it is given up, and `run ended after` with no
    /// give-up before it is the first. Held to that rather than to an ending, the screen the Extra's own end
    /// goes to being unread.
    pub fn the_extra_of_th08_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that starts the Extra")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question, so the item offered none")?;
        let state = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        expect(
            (state.difficulty, state.stage) == (EXTRA, TH08_THE_EXTRAS_STAGE),
            || {
                format!(
                    "the run that started is at difficulty {} and stage word {}, where the Extra is \
                     {EXTRA} and {TH08_THE_EXTRAS_STAGE}",
                    state.difficulty, state.stage
                )
            },
        )?;
        expect(
            trace
                .line_from(started, "retry: the run is given up")
                .is_none(),
            || "the Extra was given up rather than played out".to_owned(),
        )?;
        let ended = trace
            .line_from(started, "run ended after")
            .ok_or("the Extra never ended, so it was not cleared")?;
        // **And the screen after it walked with the game's own offer of a replay taken off and orb's kept**,
        // which is the one run of this game's cases that reaches that screen in the mode whose replays are
        // orb's: the recipe presses on to the title through it, a decide at orb's question answering the
        // first of its two, and a copy with no setting for it is asked.
        let taken_off = trace
            .line_from(
                ended,
                "replay: the offer to save one was taken off the screen a run ended on",
            )
            .ok_or("the game's own offer of a replay was left on the screen the Extra ended at")?;
        trace
            .line_from(taken_off, "replay: kept")
            .map(|_| ())
            .ok_or_else(|| "the replay of the Extra was not kept".to_owned())
    }

    /// The stage word th08's Extra is played in, which its team select writes as the difficulty and four.
    const TH08_THE_EXTRAS_STAGE: i32 = 8;

    /// **How far the stage's own clock has to get for the run to have been played**, which is what
    /// `a_normal_mode_run_of_th08_played_without_a_bomb` is held to in place of a stage reached.
    ///
    /// Two thousand, which is half a minute of stage at this game's sixty frames a second and a tenth of
    /// what the recipe's own `play_until` asks for: every launch of it so far has passed ten thousand,
    /// and the slowest reached 3,843 with three deaths behind it. So this is a bound on a launch that
    /// went nowhere at all rather than a bound on the bot.
    const A_STAGE_OF_TH08_IS_UNDER_WAY: i32 = 2_000;

    /// **th06nc's Normal run: started, played through six stages to its ending, and not one bomb pressed.**
    ///
    /// **The ending is what says the run was cleared rather than lost**: the stage's jobs leave the chain
    /// either way, and the ending's own job arrives only where the last boss went down — see
    /// `orb_core::game::th06nc`'s `ENDING_UPDATE`, whose arrival is the `run: the ending is running` line
    /// and the frames `in_ending` holds on.
    ///
    /// **And the bomb is read off the word the rules pressed**, not off the stock in hand: a death puts the
    /// stock back to three, so a stock that went down is not a bomb spent. The recipe forbids the rules
    /// the bomb from the frame the run starts, so a word with the bit in it inside the run is the rules
    /// pressing what they were told not to.
    pub fn a_run_of_th06nc_played_to_its_clear_without_a_bomb(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let last = trace
            .stage_from(started, 6)
            .ok_or("the run never reached its sixth stage")?;
        let bombed = trace.records[started..].iter().find(|record| {
            record.state.is_some_and(|state| state.in_run) && record.word & TH06NC_BOMB != 0
        });
        expect(bombed.is_none(), || {
            format!(
                "a bomb was pressed inside the run, on frame {}",
                bombed.map_or(0, |record| record.frame)
            )
        })?;
        let ending = the_ending_ran(trace, last)?;
        // **And the name the ranking was registered under**, for the reason [`a_run_played_to_its_clear`]
        // gives: a tap at that entry registers the character its cursor started on, eight times over.
        the_name_was_typed(trace, ending)
    }

    /// **th06nc's 完全無欠モード run: played through six stages to its ending, dying on the way and coming
    /// back in every chapter it died in.**
    ///
    /// **Which is what this asserts and `a_run_of_th06nc_played_to_its_clear_without_a_bomb` does not.**
    /// That one is the レガシーモード clear and is measured by the bomb never being pressed; this one is
    /// the mode's own, and what it is measured by is the deaths: the recipe hands the rules a chance of
    /// dying in each chapter — see `orb_config::recipe::Step::TakeHits` — so a run that reached the ending
    /// with no death at all is a run that never tested the mode, however green the verdict looks.
    ///
    /// **A death answered by the chapter again, and the run past it**, which is the shape
    /// [`a_death_answered_by_the_chapter_again`] holds for one chapter and this holds for every one that
    /// died: each retry line has to be followed by the run coming back, and the run has to reach the
    /// ending rather than be given up.
    pub fn a_run_of_th06nc_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        let last = trace
            .stage_from(started, 6)
            .ok_or("the run never reached its sixth stage")?;
        expect(
            trace
                .line_from(started, "retry: the run is given up")
                .is_none(),
            || "the run was given up rather than played out".to_owned(),
        )?;
        // **At least one death, which is what says the chance fired at all.** A run of this recipe that
        // read none is either a chance that never landed — which the bound below is loose enough to make
        // unlikely rather than impossible — or `take_hits` not reaching the rules, and the two are worth
        // telling apart by hand rather than passing silently.
        let deaths = trace.records[last]
            .state
            .ok_or("the sixth stage stood over no state")?
            .deaths;
        expect(deaths >= 1, || {
            "the run reached its sixth stage with no death, so the mode was never tested".to_owned()
        })?;
        // **And every retry answered with the chapter again came back**, which is the mode doing its work:
        // a line that is never followed by the run being in one again is a chapter that was put back and
        // then lost.
        let mut from = started;
        let mut retried = 0;
        while let Some(answered) = trace.line_from(from, "retry: the chapter again") {
            trace
                .in_run_from(answered + 1)
                .ok_or_else(|| format!("the run never came back after the retry at {answered}"))?;
            retried += 1;
            from = answered + 1;
        }
        expect(retried >= 1, || {
            format!("{deaths} death(s) were counted and no chapter was taken again")
        })?;
        let ending = the_ending_ran(trace, last)?;
        the_name_was_typed(trace, ending)
    }

    /// **th06nc's Extra: started on the Extra's own difficulty, and its boss beaten.**
    ///
    /// The Extra is stage 7 played on difficulty 4 — a launch of it read `stage 7 on difficulty 4` — so
    /// what says the run was the Extra is the difficulty of the run.
    ///
    /// **And what says it was cleared is the boss's life as she left, there being no ending to read.**
    /// The Extra ends into its own clear screen and the result, not the ending the numbered stages
    /// reach, and a run lost there looks the same way from the chain — the stage's jobs leave and the
    /// result's arrives, cleared or not. What differs is the boss: beaten, she leaves at the end of her
    /// last attack with a few points of life — `5` on the launch that cleared — where a run lost leaves
    /// her standing at whatever her attack held, `1000` for the whole of her survival card on the launch
    /// that died in it. So the last frame a boss stood on has to read her nearly out of life.
    pub fn the_extra_of_th06nc_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let extra = trace.records[started..]
            .iter()
            .position(|record| {
                record
                    .state
                    .is_some_and(|state| state.in_run && state.difficulty == 4)
            })
            .map(|at| at + started)
            .ok_or("the run started was not on the Extra's difficulty")?;
        let last_with_a_boss = trace.records[extra..]
            .iter()
            .rposition(|record| record.state.is_some_and(|state| state.boss_present))
            .map(|at| at + extra)
            .ok_or("no boss was ever up")?;
        let life = trace.records[last_with_a_boss]
            .state
            .and_then(|state| state.boss_life)
            .ok_or("the boss's life was not read on the last frame she stood on")?;
        expect(life <= A_BOSS_ALL_BUT_BEATEN, || {
            format!(
                "the boss left with {life} life, so the run was lost under her rather than cleared"
            )
        })?;
        // **And the name the ranking was registered under**, the Extra reaching the same entry with no
        // ending in front of it.
        the_name_was_typed(trace, last_with_a_boss)
    }

    /// **How much life a boss of th06nc leaves with where she was beaten**: her last attack ends the frame
    /// the shot that empties it lands, so what is read is the life the frame before — `5` on the launch that
    /// cleared the Extra. A bound and not that number, because how much the last shot took is the shot's.
    const A_BOSS_ALL_BUT_BEATEN: i32 = 100;

    /// Whether th06nc's ending ran at or after `from` — see [`a_run_of_th06nc_played_to_its_clear_without_a_bomb`].
    fn the_ending_ran(trace: &Trace, from: usize) -> Result<usize, String> {
        trace.records[from..]
            .iter()
            .position(|record| {
                record.state.is_some_and(|state| state.in_ending)
                    || record
                        .lines
                        .iter()
                        .any(|line| line.starts_with("run: the ending is running"))
            })
            .map(|at| at + from)
            .ok_or_else(|| "nothing says an ending ran, so the run was not cleared".to_owned())
    }

    /// Whether th06nc's own name entry was typed into at or after `from` — the line `Step::Name` writes
    /// once `Game::types_its_name` has answered true, which a tap through the entry never writes.
    fn the_name_was_typed(trace: &Trace, from: usize) -> Result<(), String> {
        trace
            .line_from(from, "types ORB into the name the game asks for")
            .map(|_| ())
            .ok_or_else(|| {
                "the ranking was not registered under the name the recipe typed".to_owned()
            })
    }

    /// **Which difficulty the Extra is**, which is the same number in both games: th06's own
    /// `DIFFICULTY_EXTRA`, and the fifth of the six th07 keeps a record per. What reads it is the case that
    /// starts the Extra, the item being what writes it rather than a select.
    const EXTRA: i32 = 4;

    /// **The mode question stood, was answered for the rewindable mode, a run was started and played into its
    /// second chapter, and then left for the title** — which is what leaves a run for the next case to pick up.
    pub fn a_run_left_in_its_second_chapter(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        expect(
            trace.between(asked, started).iter().any(|record| {
                record
                    .lines
                    .iter()
                    .any(|line| line.starts_with("mode: answered"))
            }),
            || "the mode question was not answered before the run started".to_owned(),
        )?;
        let second = trace
            .chapter_from(started, 1, 2)
            .ok_or("the run never reached stage 1 chapter 2")?;
        expect(
            trace
                .between(started, second)
                .iter()
                .all(|record| record.retries == 0),
            || {
                "a chapter was retried on the way to the second, which a clean run does not do"
                    .to_owned()
            },
        )?;
        let left = trace
            .line_from(second, "retry: the run is given up")
            .ok_or("the run was never given up for the title")?;
        expect(
            trace.records[left..]
                .iter()
                .any(|record| record.state.is_some_and(|state| !state.in_run)),
            || "the game never left the run after it was given up".to_owned(),
        )
    }

    /// **The run the case before left is offered, taken up from where it stopped, and lands in the chapter it
    /// was left in** — then plays on into the next.
    pub fn the_run_picked_up_where_it_was_left(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to pick up")?;
        let taken = trace
            .line_from(asked, "resume: from where it stopped")
            .ok_or("the resume question was not answered with where it stopped")?;
        let landed = trace
            .chapter_from(taken, 1, 2)
            .ok_or("the run never came back up in stage 1 chapter 2, where it was left")?;
        expect(
            trace
                .between(taken, landed)
                .iter()
                .all(|record| record.chapter <= 2),
            || {
                "the run went past the chapter it was left in before landing there, which is a playback \
                 that overshot"
                    .to_owned()
            },
        )?;
        trace
            .chapter_from(landed, 1, 3)
            .map(|_| ())
            .ok_or_else(|| "the picked-up run never went on into stage 1 chapter 3".to_owned())
    }

    /// **The run left is declined for one of this launch's own, and the replay of it is kept**: はじめから
    /// is answered at the question the file raises, the question about that run's replay comes up behind it
    /// and is answered はい, and the run that starts is a fresh one — nothing played back into place.
    pub fn the_run_started_over_and_its_replay_kept(trace: &Trace) -> Result<(), String> {
        let started = the_run_started_over_with_its_replay_kept(trace)?;
        trace
            .chapter_from(started, 1, 2)
            .map(|_| ())
            .ok_or_else(|| {
                "the run started from the beginning never reached its second chapter".to_owned()
            })
    }

    /// **th06nc's own, which is the check above less its last step**: the recipe it is written for stops
    /// before that game's midboss, so no second chapter of the stage is reached and what says the fresh
    /// run played on is the stage's own clock — see [`carried_past`], and [`cases_of`] for why reaching
    /// one is a matter of playing that far rather than of the midstage table.
    pub fn the_run_of_th06nc_started_over_and_its_replay_kept(trace: &Trace) -> Result<(), String> {
        let started = the_run_started_over_with_its_replay_kept(trace)?;
        let from = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        carried_past(trace, started, from.stage_frames)
    }

    /// **Every step of [`the_run_started_over_and_its_replay_kept`] up to the fresh run being on**, which
    /// is all of it that says nothing about a second chapter — and so all of it a game with one chapter a
    /// stage can be held to. Answers where that run starts.
    fn the_run_started_over_with_its_replay_kept(trace: &Trace) -> Result<usize, String> {
        let answered = the_beginning_answered(trace)?;
        let offered = trace
            .question_from(answered, "keep_replay")
            .ok_or("the replay of the run being written over was never asked about")?;
        let kept = trace
            .line_from(offered, "replay: kept")
            .ok_or("the question about the replay was not answered with keeping it")?;
        a_fresh_run_after(trace, kept)
    }

    /// **th08's own, which keeps the replay without asking about it**: this game's team select is no screen
    /// orb holds a press back on — `Th08::menu_takes_a_press` answers the title alone — so the resume
    /// question comes as the run is built, and a はじめから answered there keeps the replay of the run it
    /// writes over with no question of its own, which is `it is kept without asking` in the log. Then the
    /// fresh run plays into its second chapter, stage 1's midstage table beginning one at script 790.
    pub fn the_run_of_th08_started_over_and_its_replay_kept(trace: &Trace) -> Result<(), String> {
        let answered = the_beginning_answered(trace)?;
        let kept = trace
            .line_from(answered, "it is kept without asking")
            .ok_or("the replay of the run being written over was not kept")?;
        let started = a_fresh_run_after(trace, kept)?;
        trace
            .chapter_from(started, 1, 2)
            .map(|_| ())
            .ok_or_else(|| {
                "the run started from the beginning never reached its second chapter".to_owned()
            })
    }

    /// The resume question up and answered with はじめから, which is where every start over begins.
    fn the_beginning_answered(trace: &Trace) -> Result<usize, String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to start over")?;
        trace
            .line_from(asked, "resume: from the beginning")
            .ok_or_else(|| "the resume question was not answered with the beginning".to_owned())
    }

    /// **The run after the replay was kept, and a fresh one**: nothing played back into place, and the stage's
    /// first chapter where it comes up. Answers where it starts.
    fn a_fresh_run_after(trace: &Trace, kept: usize) -> Result<usize, String> {
        let started = trace
            .in_run_from(kept)
            .ok_or("no run was started after the replay was kept")?;
        expect(
            trace.between(kept, started).iter().all(|record| {
                !record
                    .lines
                    .iter()
                    .any(|line| line.contains("resume: the landing is"))
            }),
            || {
                "a chapter was played back into place, where a run from the beginning has none"
                    .to_owned()
            },
        )?;
        expect(trace.records[started].chapter <= 1, || {
            format!(
                "the run came up in chapter {}, where a run from the beginning starts at the stage's \
                 first",
                trace.records[started].chapter
            )
        })?;
        Ok(started)
    }

    /// **The replay kept, watched by stage, chapter and attempt**: the question over the title menu's replay
    /// item is answered for the mode whose replays are orb's own, each of the three lists behind it is
    /// answered in turn, and the attempt chosen is played back into the stage the run made it in — landing
    /// where the run was and then going on a frame of the attempt to a frame of the game.
    pub fn an_attempt_of_the_kept_replay_watched(trace: &Trace) -> Result<(), String> {
        trace
            .line_from(0, "menu: Replays is under the cursor")
            .ok_or("the question orb asked was not the one about which replays to look at")?;
        let listed = trace
            .line_from(0, "replay(s) to watch")
            .ok_or("the replays orb keeps were never listed")?;
        let chosen = trace
            .line_from(listed, "stage(s) in it")
            .ok_or("no replay was chosen out of the list")?;
        let of_a_stage = trace
            .line_from(chosen, "chapter(s) in it")
            .ok_or("no stage of that replay was chosen")?;
        let of_a_chapter = trace
            .line_from(of_a_stage, "attempt(s) at it")
            .ok_or("no chapter of that stage was chosen")?;
        let watching = trace
            .line_from(of_a_chapter, "replay: watching attempt")
            .ok_or("no attempt at that chapter was chosen")?;
        let landed = trace
            .line_from(watching, "resume: the landing is")
            .ok_or("the attempt was never played back into a stage")?;
        let over = trace
            .line_from(landed, "replay: the attempt is over")
            .ok_or("the attempt never ran out")?;
        // The stage's own clock over the frames it was being watched in, which is what says a frame of the
        // attempt went to a frame of the game rather than the run being left standing where it landed.
        let clock: Vec<u32> = trace
            .between(landed, over)
            .iter()
            .filter_map(|record| {
                record
                    .state
                    .filter(|state| state.in_run)
                    .map(|state| state.stage_frames)
            })
            .collect();
        expect(clock.windows(2).all(|pair| pair[1] >= pair[0]), || {
            "the stage's clock went back while the attempt was being watched".to_owned()
        })?;
        expect(
            matches!((clock.first(), clock.last()), (Some(first), Some(last)) if *last >= first + 60),
            || {
                format!(
                    "the attempt was watched over {} frame(s) of the stage, which is not a run being \
                     played on",
                    clock.last().unwrap_or(&0) - clock.first().unwrap_or(&0),
                )
            },
        )?;
        expect(
            trace.records[over..]
                .iter()
                .any(|record| record.state.is_some_and(|state| !state.in_run)),
            || "the run the attempt was watched in was never left".to_owned(),
        )?;
        trace
            .line_from(over, "replay: the attempts at that chapter again")
            .map(|_| ())
            .ok_or_else(|| "the list the attempt was chosen out of never came back up".to_owned())
    }

    /// **[`an_attempt_of_the_kept_replay_watched`], of an attempt a retry threw away**: the chapter chosen
    /// holds more attempts than one and the one watched is not the last of them, which is the attempt that
    /// went on — so what was watched is a stretch of the run the death rewound it from, which the game's
    /// own replay never holds.
    pub fn an_attempt_a_retry_rewound_away_watched(trace: &Trace) -> Result<(), String> {
        let watching = trace
            .line_from(0, "replay: watching attempt")
            .ok_or("no attempt was chosen")?;
        let said = trace.records[watching]
            .lines
            .iter()
            .find(|line| line.contains("replay: watching attempt"))
            .expect("line_from answers a record holding the line");
        let (attempt, of) = said
            .split("replay: watching attempt ")
            .nth(1)
            .and_then(|rest| {
                let mut words = rest.split_whitespace();
                let attempt: u32 = words.next()?.parse().ok()?;
                let of: u32 = words.nth(1)?.parse().ok()?;
                Some((attempt, of))
            })
            .ok_or_else(|| {
                format!("the attempt watched is not one the line says the number of: {said}")
            })?;
        expect(attempt < of, || {
            format!(
                "attempt {attempt} of {of} was watched, which is no attempt a retry threw away: {said}"
            )
        })?;
        an_attempt_of_the_kept_replay_watched(trace)
    }

    /// **A hit taken on purpose raises the retry question, the chapter is taken again, and the run goes on
    /// past it**: the deaths climb, the retries climb, the stage's clock goes back, and the chapter after the
    /// one died in is reached all the same.
    pub fn a_death_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let retry = trace
            .question_from(started, "retry")
            .ok_or("the retry question never stood, so nothing was hit")?;
        let before = trace.records[retry]
            .state
            .ok_or("the retry stood over no state")?;
        let died_in = trace.records[retry].chapter;
        expect(before.deaths >= 1, || {
            format!(
                "the retry stood with the game counting {} deaths",
                before.deaths
            )
        })?;
        let answered = trace
            .line_from(retry, "retry: the chapter again")
            .ok_or("the retry question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == died_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {died_in}, which it died in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        trace
            .chapter_from(back, 1, died_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the run never got past chapter {died_in} after retrying it"))
    }

    /// **th06nc's death answered by the chapter again**, which is [`a_death_answered_by_the_chapter_again`]
    /// up to its last step: the retry stands with a death counted, it is answered with the chapter again,
    /// the run comes back in the chapter it died in with the stage's clock behind it, and the retry is
    /// counted.
    ///
    /// **And then the run carries on past where it died, rather than into the chapter after it.** The
    /// recipe stops before this game's midboss, so no chapter after the one it died in is reached and
    /// what says the run went on is the stage's own clock passing the frame the death was on. Which is
    /// the weaker of the two claims and the only one this recipe can make: a run put back and then stuck
    /// would show as a clock that never got there. See [`cases_of`] for why the stronger one is a matter
    /// of playing as far as the fight.
    pub fn a_death_of_th06nc_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let retry = trace
            .question_from(started, "retry")
            .ok_or("the retry question never stood, so nothing was hit")?;
        let before = trace.records[retry]
            .state
            .ok_or("the retry stood over no state")?;
        let died_in = trace.records[retry].chapter;
        expect(before.deaths >= 1, || {
            format!(
                "the retry stood with the game counting {} deaths",
                before.deaths
            )
        })?;
        let answered = trace
            .line_from(retry, "retry: the chapter again")
            .ok_or("the retry question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == died_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {died_in}, which it died in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        carried_past(trace, back, before.stage_frames)
    }

    /// **th08's death answered by the chapter again**, which is
    /// [`a_death_of_th06nc_answered_by_the_chapter_again`]'s shape for that game's reason: no row of this
    /// game's midstage table is measured either, so a stage is one chapter and no chapter after the one
    /// died in is reached — what says the run went on is its clock passing the frame the death was on.
    ///
    /// **And the clock it is read on is `script_frames` and not `stage_frames`**, which is this game's
    /// own split: the clock the game steps carries the *run's* frames, continuous across every stage —
    /// `stage_frames` is orb's count from the stage's build off it — and `script_frames` is the stage
    /// script's own, which starts again with the stage and waits out a fight. Both go back over a
    /// restore, the run's clock being a field of `.data` a snapshot copies; the script's is the one a
    /// chapter is a number on, so it is the one this reads.
    pub fn a_death_of_th08_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let retry = trace
            .question_from(started, "retry")
            .ok_or("the retry question never stood, so nothing was hit")?;
        let before = trace.records[retry]
            .state
            .ok_or("the retry stood over no state")?;
        let died_in = trace.records[retry].chapter;
        expect(before.deaths >= 1, || {
            format!(
                "the retry stood with the game counting {} deaths",
                before.deaths
            )
        })?;
        let answered = trace
            .line_from(retry, "retry: the chapter again")
            .ok_or("the retry question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == died_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {died_in}, which it died in",
                trace.records[back].chapter
            )
        })?;
        expect(after.script_frames < before.script_frames, || {
            format!(
                "the stage script's clock did not go back: {} before the retry, {} after",
                before.script_frames, after.script_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        // **And the sound left alone rather than rewound**, which is what makes a restore of this game
        // safe: `Th08::audio_state` names the manager, its DirectSound wrapper, the stream a played
        // track has and the source that stream reads through, and a launch read the first three as ranges
        // of the game's own heap.
        //
        // Read from the run's own start and not from the retry, because that is where the line is: a
        // chapter says what it holds back as it is *taken*, and the retry puts back the one already
        // taken rather than taking another.
        trace
            .line_from(started, "snapshot: leaving 4 audio range(s) alone")
            .ok_or_else(|| "the snapshot did not hold this game's sound back".to_owned())?;
        let past = trace.records[back..].iter().any(|record| {
            record
                .state
                .is_some_and(|state| state.in_run && state.script_frames > before.script_frames)
        });
        expect(past, || {
            format!(
                "the run never played past frame {} of the stage script",
                before.script_frames
            )
        })
    }

    /// **An attempt counted at a spell card, and the run after it keeping that count** — the two things
    /// of orb's that touch this game's 134 card records, and the one case of any game here that wants two
    /// runs inside one launch.
    ///
    /// `Th06nc::count_card_attempt` counts an attempt where a chapter inside a card is taken again,
    /// because the game counts one where a card *starts* and a chapter beginning inside one never starts
    /// it. `Th06nc::put_back_the_cards_a_run_left` is what keeps it: the build of a run's *first* stage
    /// reads the score file over all 134 records — `0x14006cc40(the parsed file, manager + 0x968)` at
    /// `0x14003ad68` — so the count is lost at the start of the next run and nowhere else, which is why
    /// the second run is in the recipe at all.
    ///
    /// **Both are read off orb's own lines and not off the state**, no field of a card record being in
    /// the trace: `retry: attempt N at this spell card` is what says one was counted, and
    /// `score: what the run before counted put back` what says the note was spent. What says the attempt
    /// was counted at a card rather than in open stage is the state the question stood over having one up.
    ///
    /// **And the chapter is asked for again rather than lost**, which is the recipe's own doing and the
    /// reason no death is looked for here: orb counts an attempt for any chapter taken again with a card
    /// up, and rules told to take a hit once the card's chapter is running take it in the chapter after.
    pub fn an_attempt_at_a_card_of_th06nc_counted_and_kept(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let card = trace.records[started..]
            .iter()
            .position(|record| {
                record
                    .state
                    .is_some_and(|state| state.in_run && state.spellcard.is_some())
            })
            .map(|at| at + started)
            .ok_or_else(|| {
                // **Says what the trace did hold**, because the two fields this asks for are read on
                // different frames and a message naming neither leaves a launch to guess with: the last
                // frame of the trace, how many of its records are inside a run, and the highest card
                // number any record carried.
                let in_a_run = trace
                    .records
                    .iter()
                    .filter(|record| record.state.is_some_and(|state| state.in_run))
                    .count();
                let any_card = trace
                    .records
                    .iter()
                    .filter_map(|record| record.state.and_then(|state| state.spellcard))
                    .max();
                format!(
                    "no spell card was up on a frame the run was on: {} record(s), {in_a_run} of them \
                     inside a run, the last frame {}, and the cards any record carried at all up to \
                     {any_card:?}",
                    trace.records.len(),
                    trace.records.last().map_or(0, |record| record.frame),
                )
            })?;
        let retry = trace
            .question_from(card, "retry")
            .ok_or("the retry question never stood while a card was up")?;
        expect(
            trace.records[retry]
                .state
                .is_some_and(|state| state.spellcard.is_some()),
            || "the chapter the retry question stood over had no card up".to_owned(),
        )?;
        let counted = trace
            .line_from(retry, "retry: attempt ")
            .ok_or("nothing counted an attempt at the card the chapter was taken again inside")?;

        // **The second run, which is what the put-back is spent at**: the run before it is left for the
        // title, and the note orb took as it ended goes back over the records that run's own build reads
        // the file into.
        let inside = trace
            .in_run_from(counted)
            .ok_or("the count was made outside a run")?;
        let left = trace.records[inside..]
            .iter()
            .position(|record| record.state.is_none_or(|state| !state.in_run))
            .map(|at| at + inside)
            .ok_or("the run that counted was never left, so no second run could start")?;
        trace
            .in_run_from(left)
            .ok_or("no second run was started in the launch, so the put-back was never spent")?;
        // **Read from the frame the run was left on** and not from the frame the second is in: the
        // put-back is made inside the build, which the state does not read as a run yet.
        trace
            .line_from(left, "score: what the run before counted put back")
            .map(|_| ())
            .ok_or_else(|| {
                "the run after the one that counted put nothing back over the records its build read \
                 the file into"
                    .to_owned()
            })
    }

    /// **th06nc's chapter taken again without a death**, which is
    /// [`a_chapter_taken_again_without_a_death`] up to its last step — and that step is the stage's own
    /// clock passing the frame the run was asked to start over at, for
    /// [`a_death_of_th06nc_answered_by_the_chapter_again`]'s reason: its recipe stops before the fight,
    /// so no chapter after the one asked about is reached.
    pub fn a_chapter_of_th06nc_taken_again_without_a_death(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(after.deaths == before.deaths, || {
            format!(
                "the run died on the way to the chapter again: {} death(s) when it was asked to \
                 start over and {} when it came back",
                before.deaths, after.deaths
            )
        })?;
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked \
                 to start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        carried_past(trace, back, before.stage_frames)
    }

    /// **th06nc's stage taken again from its start**, which is [`the_stage_taken_again_from_its_start`]
    /// up to its last step — the stage's own clock passing the frame the run was asked at, for the same
    /// reason: a chapter after the stage's first is what this game has none of.
    pub fn the_stage_of_th06nc_taken_again_from_its_start(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        trace
            .line_from(up, "retry: asking about the stage again")
            .ok_or("the stage was taken again without being asked about first")?;
        let answered = trace
            .line_from(up, "retry: the stage again")
            .ok_or("the question was not answered with the stage again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the stage was taken again")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == 1, || {
            format!(
                "the run came back in chapter {} rather than in the stage's first, having been asked \
                 to start over in chapter {asked_in}",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        // **Past the frame it came back at and not past the frame it was asked at**, which is what the
        // stage's own start means: the clock goes back to the beginning of the stage, so a run that
        // played its way up to where it was asked would have to play the whole stage again — and this
        // game has no chapter after the stage's first to say it that way instead.
        carried_past(trace, back, after.stage_frames)
    }

    /// **The run played on past a frame of the stage it had already reached**, which is the weaker of the
    /// two ways a case says the run carried on and the only one a game with a single chapter a stage can
    /// make: a run put back and then stuck would show as a clock that never got there.
    fn carried_past(trace: &Trace, from: usize, frames: u32) -> Result<(), String> {
        let carried_on = trace.records[from..].iter().any(|record| {
            record
                .state
                .is_some_and(|state| state.in_run && state.stage_frames > frames)
        });
        expect(carried_on, || {
            format!("the run never played past frame {frames} of the stage")
        })
    }

    /// **The run asked to start over raises the same question a death does, and the chapter is taken again
    /// with nothing lost**: the chapter comes back, the stage's clock goes back, the retries climb — and the
    /// deaths do not, which is what tells this retry from the one a death asks for.
    pub fn a_chapter_taken_again_without_a_death(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(after.deaths == before.deaths, || {
            format!(
                "the run died on the way to the chapter again: {} death(s) when it was asked to \
                 start over and {} when it came back",
                before.deaths, after.deaths
            )
        })?;
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked \
                 to start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        trace
            .chapter_from(back, 1, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| {
                format!("the run never got past chapter {asked_in} after taking it again")
            })
    }

    /// **The chapters behind the one the run was in are a list of their own, and one of them puts the run
    /// back there**: the list comes up, a chapter is chosen out of it and confirmed, and the run comes back
    /// in a chapter earlier than the one it was asked to start over in — then plays through to it again.
    pub fn a_chapter_further_back_taken_again(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        trace
            .line_from(up, "retry: asking which chapter further back")
            .ok_or("the chapters behind the one the run was in were never listed")?;
        let answered = trace
            .line_from(up, "retry: a chapter further back")
            .ok_or("no chapter behind the one the run was in was chosen")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the chapter behind was chosen")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        let came_back_in = trace.records[back].chapter;
        expect(came_back_in < asked_in, || {
            format!(
                "the run came back in chapter {came_back_in} rather than behind chapter {asked_in}, \
                 which it was asked to start over in"
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 1, asked_in)
            .map(|_| ())
            .ok_or_else(|| {
                format!(
                    "the run never played back up to chapter {asked_in} from the chapter behind it"
                )
            })
    }

    /// **The stage's own start is a way on of its own, and it asks before it acts**: the question is
    /// answered with the stage, はい is answered at the confirmation behind it, and the run comes back at
    /// the stage's first chapter with its clock at the start of the stage.
    pub fn the_stage_taken_again_from_its_start(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        trace
            .line_from(up, "retry: asking about the stage again")
            .ok_or("the stage was taken again without being asked about first")?;
        let answered = trace
            .line_from(up, "retry: the stage again")
            .ok_or("the question was not answered with the stage again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the stage was taken again")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == 1, || {
            format!(
                "the run came back in chapter {} rather than in the stage's first, having been asked \
                 to start over in chapter {asked_in}",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 1, 2)
            .map(|_| ())
            .ok_or_else(|| "the run never played out of the stage's first chapter again".to_owned())
    }

    /// **The run given up at that question, which is the other way a session is interrupted**: the item
    /// asks first, はい is answered at it, and the game leaves the run for the title.
    pub fn the_run_given_up_at_the_retry(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        trace
            .line_from(up, "retry: asking about the run given up")
            .ok_or("the run was given up without being asked about first")?;
        let answered = trace
            .line_from(up, "retry: the run given up")
            .ok_or("the question was not answered with the run given up")?;
        let left = trace
            .line_from(answered, "retry: the run is given up")
            .ok_or("nothing gave the run up, the answer notwithstanding")?;
        expect(
            trace.records[left..]
                .iter()
                .any(|record| record.state.is_some_and(|state| !state.in_run)),
            || "the game never left the run after it was given up".to_owned(),
        )
    }

    /// **The run played into the stage after its first, and a chapter of that one taken again**: the second
    /// stage begins with its own clock and the numbers the run carried into it, and the chapter it is asked
    /// to start over in comes back the way a chapter of the first stage does.
    pub fn a_chapter_of_the_stage_after_the_first(trace: &Trace) -> Result<(), String> {
        let (back, asked_in) = the_stage_after_the_first_and_its_chapter_again(trace)?;
        trace
            .chapter_from(back, 2, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the run never got past chapter {asked_in} of the second stage"))
    }

    /// **Every step of [`a_chapter_of_the_stage_after_the_first`] up to the retry coming back**, which is
    /// all of it that says nothing about a chapter after the one retried — and so all of it a recipe that
    /// stops before the fight can be held to. Answers where the run came back and the chapter it was
    /// asked to start over in.
    ///
    /// A function of its own rather than a flag on that check, so that neither caller can read the other's
    /// tail: what a game whose midstage table is empty has to say instead is the stage's own clock
    /// carrying on — see [`carried_past`].
    fn the_stage_after_the_first_and_its_chapter_again(
        trace: &Trace,
    ) -> Result<(usize, u32), String> {
        let first = trace
            .chapter_from(0, 1, 1)
            .ok_or("the run never reached the first stage's first chapter")?;
        let carried = trace.records[first]
            .state
            .expect("chapter_from answers a record with a state");
        let second = trace
            .chapter_from(first, 2, 1)
            .ok_or("the run never reached the stage after the first")?;
        let begun = trace.records[second]
            .state
            .expect("chapter_from answers a record with a state");
        // Where the first stage had got to, which is what the second beginning is held against: its own
        // first chapter is taken as many frames in as the first stage's was, so the two chapters say
        // nothing about a clock starting again and the frame before the transition says it.
        let ended = trace
            .between(first, second)
            .iter()
            .rev()
            .find_map(|record| {
                record
                    .state
                    .filter(|state| state.in_run && state.stage == carried.stage)
            })
            .ok_or("the run was never in the first stage")?;
        expect(begun.stage_frames < ended.stage_frames, || {
            format!(
                "the second stage's clock is at {} where the first stage's had reached {}, so nothing \
                 began again",
                begun.stage_frames, ended.stage_frames
            )
        })?;
        // And what only the start of a run puts in place, held against what a stage walked into carries:
        // the deaths are the run's own count, so a stage that had reset them would be one built as a run's
        // first rather than out of the stage before it.
        expect(begun.deaths >= ended.deaths, || {
            format!(
                "the run reached the second stage with {} death(s) against the {} the first stage ended \
                 with",
                begun.deaths, ended.deaths
            )
        })?;
        let asked = trace
            .line_from(second, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over in the second stage")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(after.stage == before.stage, || {
            format!(
                "the run came back in stage {} rather than in stage {}, which it was retried in",
                after.stage + 1,
                before.stage + 1
            )
        })?;
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked to \
                 start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        Ok((back, asked_in))
    }

    /// **th06nc's own, which is the check above less its last step**: its recipe stops before the second
    /// stage's fight, so there is no chapter after the one retried to wait for and what says the run went
    /// on is the stage's own clock — see [`carried_past`].
    pub fn a_chapter_of_th06nc_in_the_stage_after_the_first(trace: &Trace) -> Result<(), String> {
        let (back, _) = the_stage_after_the_first_and_its_chapter_again(trace)?;
        let after = trace.records[back]
            .state
            .expect("the run came back with a state");
        carried_past(trace, back, after.stage_frames)
    }

    /// **th06nc's own pick-up in the stage it walked into**, which is
    /// [`the_run_picked_up_in_the_stage_it_walked_into`] with both of its chapter waits taken out: the
    /// chapter the run is left in and picked up in is the second stage's first, that being the only one
    /// this game has, and what says the run played on afterwards is the stage's own clock.
    pub fn the_run_of_th06nc_picked_up_in_the_stage_it_walked_into(
        trace: &Trace,
    ) -> Result<(), String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to pick up")?;
        let taken = trace
            .line_from(asked, "resume: from where it stopped")
            .ok_or("the resume question was not answered with where it stopped")?;
        let landed = trace
            .chapter_from(taken, 2, 1)
            .ok_or("the run never came back up in a chapter of the second stage")?;
        // **And the stage held to exactly and not to *at or past***, which is what
        // `Trace::chapter_from` asks and what let a launch pass with the run one stage too far: it read
        // `resume: not put back — stage 3 came up where stage 2 was asked for` and this check said
        // nothing, a run in stage 3 being past the stage named. The stage the run comes back in is the
        // stage the file named or the pick-up is wrong, whichever side it is wrong on — a first stage
        // here would be a run started from the beginning with a chapter of the second to put back, and a
        // third is the build having counted the stage on over what orb wrote.
        let landed_in = trace.records[landed]
            .state
            .expect("chapter_from answers a record with a state");
        expect(landed_in.stage == 1, || {
            format!(
                "the run came back up in stage {} where the file named stage 2",
                landed_in.stage + 1
            )
        })?;
        // And nothing between the answer and the landing was in another stage either, which is what says
        // the stage above is the one the build came up in rather than one the run walked on into.
        let elsewhere = trace.between(taken, landed).iter().find(|record| {
            record
                .state
                .is_some_and(|state| state.in_run && state.stage != 1)
        });
        expect(elsewhere.is_none(), || {
            let record = elsewhere.expect("a record in another stage");
            format!(
                "frame {} of the pick-up is in stage {}, where the file named stage 2",
                record.frame,
                record.state.expect("a record with a state").stage + 1
            )
        })?;
        let landed_at = trace.records[landed]
            .state
            .expect("chapter_from answers a record with a state");
        carried_past(trace, landed, landed_at.stage_frames)
    }

    /// **The run left in the stage it walked into is picked up in that stage**: the build a resume answers
    /// for is pointed at the stage the file names, so the run comes up there rather than at a run's first —
    /// then lands in the chapter it was left in and plays on.
    pub fn the_run_picked_up_in_the_stage_it_walked_into(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to pick up")?;
        let taken = trace
            .line_from(asked, "resume: from where it stopped")
            .ok_or("the resume question was not answered with where it stopped")?;
        let landed = trace
            .chapter_from(taken, 2, 2)
            .ok_or("the run never came back up in a chapter of the second stage")?;
        // **And it came up in that stage rather than at a run's own first**, which is the other way this can
        // go wrong: the build a resume answers for is pointed at the stage the file names, and a first stage
        // coming up here would be a run started from the beginning with a chapter of the second to put back.
        let at_the_first = trace.between(taken, landed).iter().find(|record| {
            record
                .state
                .is_some_and(|state| state.in_run && state.stage == 0)
        });
        expect(at_the_first.is_none(), || {
            format!(
                "frame {} of the pick-up is in the first stage, where the file named the second",
                at_the_first.expect("a record in the first stage").frame
            )
        })?;
        trace.chapter_from(landed, 2, 3).map(|_| ()).ok_or_else(|| {
            "the run picked up in the second stage never went on into its next chapter".to_owned()
        })
    }

    /// **One stage practised on its own, which orb rewinds and writes nothing down for**: the run says it is
    /// a practice one, its chapters are a stage's like any other's and one of them is taken again — and
    /// nothing of it was looked for on the disk or left there, a run of this kind having no name.
    pub fn a_stage_practised_on_its_own(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that practises a stage")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        let state = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        expect(state.practice, || {
            "the run that started reads as a full run rather than as one stage practised".to_owned()
        })?;
        let in_a_chapter = trace
            .a_chapter_from(started, 1)
            .ok_or("the practice run never reached a chapter of its own")?;
        trace
            .line_from(
                started,
                "of a run orb is not keeping; nothing of it is written down",
            )
            .ok_or("nothing said the practice run was one orb writes nothing down for")?;
        expect(
            trace.line_from(0, "resume: nothing was left of").is_none(),
            || {
                "a run left unfinished was looked for, where a practice run has no name to look one \
                 up by"
                    .to_owned()
            },
        )?;
        // And rewound all the same, which is the other half of a practice run being a run.
        let asked_over = trace
            .line_from(in_a_chapter, "retry: the run was asked to start over")
            .ok_or("the practice run was never asked to start over")?;
        let up = trace
            .question_from(asked_over, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the practice run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked \
                 to start over in",
                trace.records[back].chapter
            )
        })?;
        // **Not past where the question stood, rather than behind it**, and the claim is still the one the
        // case makes: a `start_over` that did nothing fails this, the run carrying on and coming back past
        // where the question stood. With the chapter held above it, the pair says the ask rewound and
        // rewound into the chapter it was asked in.
        //
        // **What it cannot be is strictly behind, because a death can land on the ask's own frame.** The
        // laid-out game reads `step 23 of 28 asks the run to start over` at f6145 and `died in chapter 3`
        // at f6146, the two retries resolving together, and the clock is then at the chapter's start on
        // both sides of it — 501 either side. **And forbidding the rules the hit does not fix that**,
        // which was tried: a chance of nought is a chance of being hit *deliberately*, where that
        // chapter is one the rules cannot dodge at all. The launch with a `take_hits` step in still read
        // `deaths=9`, every one of them before the step, `step 22 of 28 done` for the `play_until` that
        // did reach its 600 frames, and then the same death one frame past the ask. So the recipe's own
        // comment carries the reading and this stays `<=`.
        expect(after.stage_frames <= before.stage_frames, || {
            format!(
                "the stage's clock went on past where the retry was asked: {} before the retry, {} \
                 after",
                before.stage_frames, after.stage_frames
            )
        })
    }

    /// **A run played from the title to its clear, which is the whole of what the mode is for**: it goes
    /// through every stage and is never given up, the ending runs out to where its staff roll begins — which
    /// is what says the game was cleared rather than left — and the replay of a run nobody will add to again
    /// is offered at the screen that run ends at and kept.
    pub fn a_run_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        // **The fifth stage and not the last**, because how many stages a run has is the game's: th06's Easy
        // ends after the fifth, where every other difficulty of either game has a sixth. What says the run
        // was played *out* is the ending below and not a count of stages.
        let last = trace
            .chapter_from(started, 5, 1)
            .ok_or("the run never reached a fifth stage")?;
        expect(
            trace
                .line_from(started, "retry: the run is given up")
                .is_none(),
            || "the run was given up rather than played out".to_owned(),
        )?;
        trace
            .line_from(last, "run ended after")
            .ok_or("the run never ended, so it was neither cleared nor left")?;
        // **And the ending is what says it was cleared rather than left**: a run given up reaches none.
        let ending = trace
            .ending_from(last)
            .ok_or("nothing says an ending ran, so the run was not cleared")?;
        // **And the name the ranking was registered under.** The entry a run ends at is a grid of
        // characters with a cursor of its own, so the tap that walks these screens registers eight of
        // whichever character it started on — `AAAAAAAA`, on every clear this case played before
        // `Step::Name` existed.
        trace
            .line_from(ending, "types ORB into the name the game asks for")
            .ok_or("the ranking was not registered under the name the recipe typed")?;
        let offered = trace
            .question_from(ending, "keep_replay")
            .ok_or("the replay of the run that was finished with was never asked about")?;
        trace
            .line_from(offered, "replay: kept")
            .map(|_| ())
            .ok_or_else(|| {
                "the question about that replay was not answered with keeping it".to_owned()
            })
    }

    /// **A run of the other mode played to its clear, which is the one thing that unlocks the Extra**: the
    /// question is answered for the normal mode, no chapter is counted in any frame of it, the run goes
    /// through the stages on the lives the game gave it, and an ending comes — the record of which lands in
    /// the game's own score file, where the front end reads the items it offers from.
    pub fn a_normal_mode_run_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let answered = trace
            .line_from(asked, "mode: normal")
            .ok_or("the mode question was not answered for the normal mode")?;
        let started = trace
            .in_run_from(answered)
            .ok_or("no run was started after the mode question")?;
        // The fifth stage and by the stage alone: a run of this mode is at chapter nought on every frame,
        // orb keeping none of it, so there is no chapter to wait for.
        let last = trace
            .stage_from(started, 5)
            .ok_or("the run never reached a fifth stage")?;
        let kept = trace.records[started..]
            .iter()
            .find(|record| record.chapter != 0);
        expect(kept.is_none(), || {
            let record = kept.expect("a record with a chapter of its own");
            format!(
                "frame {} of the run is at chapter {}, where a run of this mode has none",
                record.frame, record.chapter
            )
        })?;
        // **And the ending off the log's own words as well as off the scene** — see [`Trace::ending_from`],
        // which holds of a run of either mode: the run cleared here said `ending skipped, 7993 frames run,
        // scene 10 -> 7` with every record of its trace reading `in_ending` false.
        let ending = trace
            .ending_from(last)
            .ok_or("nothing says an ending ran, so the run was not cleared")?;
        // **And the name the ranking was registered under**, for the reason
        // [`a_run_played_to_its_clear`] gives: a tap at that entry registers the character its cursor
        // started on, eight times over.
        trace
            .line_from(ending, "types ORB into the name the game asks for")
            .map(|_| ())
            .ok_or_else(|| {
                "the ranking was not registered under the name the recipe typed".to_owned()
            })
    }

    /// **The Extra, which the run cleared in the other mode unlocked**: the run that starts is at the
    /// Extra's own difficulty, its chapters are a stage's like any other's, and one of them is taken again.
    ///
    /// What says the unlock happened is that there was a run to start at all: the item is on the title menu
    /// only where the game's own score file holds the record, and the case before this one is what put it
    /// there.
    pub fn the_extra_a_cleared_run_unlocked(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that starts the Extra")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question, so the item offered none")?;
        let state = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        expect(state.difficulty == EXTRA, || {
            format!(
                "the run that started is at difficulty {}, where the Extra is {EXTRA}",
                state.difficulty
            )
        })?;
        let in_a_chapter = trace
            .chapter_from(started, 7, 1)
            .ok_or("the Extra never reached a chapter of its own")?;
        let asked_over = trace
            .line_from(in_a_chapter, "retry: the run was asked to start over")
            .ok_or("the Extra was never asked to start over")?;
        let up = trace
            .question_from(asked_over, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the Extra never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked to \
                 start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 7, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the Extra never got past chapter {asked_in} after retrying it"))
    }

    /// **A run of the other mode is the game as it comes**: the question is answered for the normal mode,
    /// the run is played, and orb keeps nothing of it — no chapter counted, no retry, and the hook that
    /// writes a chapter down saying it wrote nothing.
    pub fn a_normal_mode_run_nothing_is_kept_of(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let answered = trace
            .line_from(asked, "mode: normal")
            .ok_or("the mode question was not answered for the normal mode")?;
        let started = trace
            .in_run_from(answered)
            .ok_or("no run was started after the mode question")?;
        trace
            .line_from(
                answered,
                "of a run orb is not keeping; nothing of it is written down",
            )
            .ok_or("nothing said the run was one orb keeps nothing of")?;
        let kept = trace.records[started..]
            .iter()
            .find(|record| record.chapter != 0 || record.retries != 0);
        expect(kept.is_none(), || {
            let record = kept.expect("a record with a chapter of its own");
            format!(
                "frame {} of the run is at chapter {} with {} retr(ies), where a run of this mode \
                 has neither",
                record.frame, record.chapter, record.retries
            )
        })?;
        let over = trace.records[started..]
            .iter()
            .find(|record| record.question.is_some());
        expect(over.is_none(), || {
            let record = over.expect("a record with a question over it");
            format!(
                "orb put its {} question up on frame {} of a run of this mode",
                record.question.as_deref().unwrap_or_default(),
                record.frame
            )
        })
    }
}

#[cfg(test)]
mod tests {
    /// **Every recipe this crate carries is one orb can read**, which is the one thing about a case that
    /// nothing else here would find until a launch.
    ///
    /// **`orb-e2e` plays most of them and would**, but not all: the cases that play a whole game are named
    /// in `the_cases_of_th06nc_the_real_game_is_tested_with`'s own `NOT_GATED_YET` because the laid-out
    /// game cannot reach their ending, and a recipe of one of those goes to the real game unparsed. What
    /// that costs is the launch: `cargo xtask real` writes the recipe out, orb reads it, and a step it
    /// cannot read is a game that comes up and plays nothing.
    #[test]
    fn every_recipe_reads() {
        for case in super::cases() {
            let steps = orb_config::recipe::read(case.recipe).unwrap_or_else(|why| {
                panic!(
                    "{} {}'s recipe does not read: {why}",
                    case.game.name(),
                    case.name
                )
            });
            assert!(
                !steps.is_empty(),
                "{} {}'s recipe has no steps in it",
                case.game.name(),
                case.name,
            );
        }
    }
}
