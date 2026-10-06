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
//! **Every game's are gated there, clears and Extra included** — th06nc's whole-game cases over a field
//! with no bullets, which is the one thing about them the laid-out game does not play: see
//! `the_cases_of_th06nc_the_real_game_is_tested_with`'s `WITHOUT_BULLETS`. th08's are in
//! `the_cases_of_th08_the_real_game_is_tested_with`, one test a chain: its first two took 35 seconds of the
//! laid-out game where the real one took eight minutes, and the second of them was written wrong once — a
//! check looking for a line orb writes on a hit rather than per frame, which the real game took seven
//! minutes to refuse. th10's are in `the_cases_of_th10_the_real_game_is_tested_with`, one test a chain the
//! same way.
//!
//! **No case depends on another.** Every case starts from a directory of its own and the copy's records taken
//! aside, so that which difficulty a select comes up on, which item a title menu is on and what the game says has
//! been unlocked are the same every time it is played, whichever cases were played before it or none. A case that
//! needs a run left unfinished or a replay kept plays the launches that leave one as its own first, [`Case::before`];
//! a case that needs the Extra or a stage's practice offered starts from a score file the game's own logic makes for
//! the clears that offer them, [`Case::starts_from`] — and whether a clear does unlock them is the case that plays
//! one.
//! Nothing in the copy is deleted for it; see `cargo xtask real`.
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
    /// **And 「東方紅魔郷: Classic」, the Steam release of 東方紅魔郷**, a 64-bit game with the same cases as
    /// th06, and its two clears with no hit and no bomb after them: `orb_core::game::th06c` rewinds, so a
    /// run of it is picked up, a chapter of it taken again and its score file forked the way theirs are. Its
    /// front end is walked by its own screen numbers — see its recipes.
    Th06c,
    /// **And the other Steam release, 「東方紅魔郷: New Classic」**, on th06c's engine with every address its
    /// own — see `orb_core::game::th06nc`, which rewinds. Its cases are its own list, for what its front
    /// end and its records are: its checks of a chapter inside a stage are th06c's, over recipes walking
    /// this game's own screens.
    Th06nc,
    /// **And 東方永夜抄, Imperishable Night**, which `orb_core::game::th08` rewinds: its cases are every
    /// case the rewinding games have under this game's own recipes, with the clears and the Extra its own
    /// chain wants — see [`cases_of`].
    Th08,
    /// **And 東方風神録, Mountain of Faith**, which `orb_core::game::th10` rewinds, its first stage holding no row
    /// of its midstage table: its cases are th08's, under this game's own recipes — see [`cases_of`].
    Th10,
    /// **And 東方地霊殿, Subterranean Animism**, which `orb_core::game::th11` rewinds: its cases are th10's under this
    /// game's own recipes, less the one its fights cut no chapter for — see [`the_cases_of_th11`].
    Th11,
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
            Game::Th10 => "th10",
            Game::Th11 => "th11",
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
            Game::Th10 => "th10.exe",
            Game::Th11 => "th11.exe",
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
    ///
    /// **th10 names its score file after itself**, `scoreth10.dat` where every game before it has
    /// `score.dat` — `Th10::score_files` — so the fork is `pointdevice_scoreth10.dat`.
    ///
    /// **And th10's configuration is left in the copy**, `th10.cfg` not being one of these: without it the game
    /// stops at start-up on a dialog of its own asking whether to run in a window, which nothing answers — the
    /// first case played without it stood at that dialog with its trace holding no frame. Whether its 52 bytes
    /// hold anything a case's start turns on — a cursor the title or a select comes up on — is not established.
    pub fn records(self) -> &'static [&'static str] {
        match self {
            Game::Th06 => &["score.dat", "東方紅魔郷.cfg", "pointdevice_score.dat"],
            Game::Th07 => &["score.dat", "th07.cfg", "pointdevice_score.dat"],
            Game::Th06c | Game::Th06nc => &["score.dat", "th06.cfg", "pointdevice_score.dat"],
            Game::Th08 => &["score.dat", "th08.cfg", "pointdevice_score.dat"],
            Game::Th10 => &["scoreth10.dat", "pointdevice_scoreth10.dat"],
            // **And th11's configuration left in the copy for th10's reason**: without it the game's defaults put
            // up the startup dialog (0x420d8c), which orb answers, but with them come the defaults' pad mapping and
            // window size rather than the copy's own.
            Game::Th11 => &["scoreth11.dat", "pointdevice_scoreth11.dat"],
        }
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
            Game::Th06 | Game::Th07 | Game::Th08 | Game::Th10 | Game::Th11 => "i686-pc-windows-gnu",
            Game::Th06c | Game::Th06nc => "x86_64-pc-windows-gnu",
        }
    }

    /// **The game's records a case starts from**, by the names [`Game::records`] gives them, made with the game's own
    /// logic: the records it makes with no file of its own, each stage won and each ending seen marked as the game
    /// marks them, written as the game writes its file. Nothing for [`Start::Nothing`].
    ///
    /// **Made and not copied from a launch that played one**, so that a case starting from them depends on nothing
    /// but the exe they were read off: whether a clear does light them is the clear's own case.
    pub fn the_records_of(self, start: Start) -> Vec<(&'static str, Vec<u8>)> {
        match (self, start) {
            (_, Start::Nothing) => Vec::new(),
            (Game::Th11, Start::ANormalClear | Start::TheExtraOffered) => {
                use orb_core::game::th11::score_file;
                // Reimu and Yukari, the first character and partner, which every Extra recipe here picks.
                let (character, partner, normal) = (0, 0, 1);
                let mut records = score_file::the_records_of_a_fresh_install(0);
                for stage in 1..=6 {
                    score_file::a_stage_won(&mut records, character, partner, normal, stage);
                }
                score_file::an_ending_seen(&mut records, character, partner, 0, normal as u32);
                vec![("scoreth11.dat", score_file::encode(&records))]
            }
            (Game::Th10, Start::ANormalClear | Start::TheExtraOffered) => {
                use orb_core::game::th10::score_file;
                // Reimu A, the first character and shot, which every Extra recipe here picks.
                let (character, shot, normal) = (0, 0, 1);
                let mut records = score_file::the_records_of_a_fresh_install(0);
                for stage in 1..=6 {
                    score_file::a_stage_cleared(&mut records, character, shot, normal, stage);
                }
                score_file::an_ending_seen(&mut records, character, shot, 0, normal as u32);
                vec![("scoreth10.dat", score_file::encode(&records))]
            }
            (Game::Th08, Start::ANormalClear | Start::TheExtraOffered) => {
                use orb_core::game::th08::score_file;
                // Reimu and Yukari, the first team, on Normal with no continue: Eirin's route, its stage words
                // 0, 1, 2, 4, 5 and 6 passed and her clear, and for the Extra Kaguya's stage word 7 and hers.
                let (team, normal) = (0, 1);
                let mut clears = score_file::the_clears_of_a_fresh_install();
                for stage in [0, 1, 2, 4, 5, 6] {
                    score_file::a_stage_passed(&mut clears, team, normal, stage, false);
                }
                score_file::a_clear(&mut clears, team, normal, false, false);
                if start == Start::TheExtraOffered {
                    score_file::a_stage_passed(&mut clears, team, normal, 7, false);
                    score_file::a_clear(&mut clears, team, normal, true, false);
                }
                vec![("score.dat", score_file::encode(&clears))]
            }
            (Game::Th07, Start::ANormalClear | Start::TheExtraOffered) => {
                use orb_core::game::th07::score_file;
                // Reimu A, the first shot, on Normal with no continue.
                let mut clears = score_file::the_clears_of_a_fresh_install();
                score_file::a_clear(&mut clears, 0, 1, false);
                vec![("score.dat", score_file::encode(&clears))]
            }
            // **th06's file is th06c's**, header for header and record for record — the Steam release keeps the
            // 1.02h's format, its `TH6K` and its four `CLRD`s of version `0x10`, and its read checks a clear the same
            // way: `0x63` in Normal, Hard or Lunatic of a shot's no-retry array (th06's 0x43a736, th06c's
            // `0x14002d526`). The four `CLRD`s are what each game makes with no file, `1` in every byte of both
            // arrays (th06's 0x42b502), and a Normal clear of Reimu A with no retry raises Normal to `0x63` in both.
            (Game::Th06 | Game::Th06c, Start::ANormalClear | Start::TheExtraOffered) => {
                use orb_core::game::th06c::score_file;
                let normal_cleared = [1, score_file::CLEARED, 1, 1, 1];
                let mut records = vec![
                    score_file::th6k(),
                    score_file::clrd(0, normal_cleared, normal_cleared),
                ];
                records.extend((1..4).map(|shot| score_file::clrd(shot, [1; 5], [1; 5])));
                vec![("score.dat", score_file::encode(&records, 0))]
            }
            (Game::Th06nc, Start::ANormalClear | Start::TheExtraOffered) => {
                use orb_core::game::th06nc::score_file;
                let normal_cleared = [1, score_file::CLEARED, 1, 1, 1];
                let mut records = vec![
                    score_file::th6k(),
                    score_file::clrd(0, normal_cleared, normal_cleared),
                ];
                records.extend((1..4).map(|shot| score_file::clrd(shot, [1; 5], [1; 5])));
                vec![("score.dat", score_file::encode(&records, 0))]
            }
        }
    }

    pub const ALL: [Game; 7] = [
        Game::Th06,
        Game::Th07,
        Game::Th06c,
        Game::Th06nc,
        Game::Th08,
        Game::Th10,
        Game::Th11,
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

/// **What a case's first launch finds of the game's own records in the copy** — see [`Game::the_records_of`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Start {
    /// None, as a fresh install has.
    Nothing,
    /// A run of the game's own mode played to the clear on Normal with no continue: every stage of the main game
    /// offered by the practice, and in th08 Eirin's stage passed, which a run has to have to go to Kaguya.
    ANormalClear,
    /// The clears the main menu offers the Extra out of: [`Start::ANormalClear`]'s, and in th08 Kaguya's as well.
    TheExtraOffered,
}

/// One launch of the real game and what its trace has to show.
pub struct Case {
    pub game: Game,
    /// The name the recipe file and the trace's directory carry.
    pub name: &'static str,
    /// The recipe itself, `orb-config`'s grammar.
    pub recipe: &'static str,
    /// **The cases whose recipes are played first, as launches of this case's own**, in its directory and with the
    /// records they leave: the run left for a pick-up, the replay kept for one to be watched. Only this case's own
    /// launch is checked; theirs are checked as the cases they are.
    pub before: &'static [&'static str],
    /// What the first of its launches finds of the game's own records.
    pub starts_from: Start,
    /// How long this launch may take, in seconds — [`WITHIN`] for every case but the one that plays a whole
    /// game.
    pub within: u32,
    /// What the trace of that launch has to show, and why not where it does not.
    pub check: fn(&Trace) -> Result<(), String>,
}

impl Case {
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
    ///
    /// **And th11 plays its whole games at one too, paced to sixteen instead** — see [`Case::pace`].
    pub fn speed(&self) -> u32 {
        match (self.within == A_WHOLE_GAME, self.game) {
            (true, Game::Th06nc) => 1,
            (true, Game::Th11) if self.pace() > 1 => 1,
            (true, _) => 4,
            (false, _) => 1,
        }
    }

    /// **How many times its own rate this case is paced to**, `--pace`: sixteen for a case of th11's that plays a
    /// whole game, and one for every other — where [`Case::speed`] is the fast-forward instead.
    ///
    /// **Paced and not sped up, for a game whose frames orb is inside**, which th11's are: at four updates a
    /// frame orb steered the run once every four, and wrote the chapter it reached down only where the chapter's
    /// first frame was a drawn one — so a hit was retried from a chapter further back, and the hitless Extra
    /// was hit two to four times on each of four launches (real1 to real4, 2026-10-08) in cards that took no hit
    /// on the chapters kept for them, which were picked up at `--pace=4` with orb at every frame.
    ///
    /// **The normal mode run that unlocks the Extra as well**, which was left at four updates a frame once: its
    /// recipe's answer of item 1 to orb's own mode question gave up its whole bound at `--pace=4` with the game
    /// held on frame 274 (real5, 2026-10-08) — and the same answer through the pad came off at `--pace=4` and at
    /// one, 91 and 368 milliseconds after the question came up (`tune-mode-paced.yaml`, 2026-10-08 21:24 and 21:30).
    ///
    /// **Sixteen and not four**, which it was: card 171's nineteen kept chapters were hit the same at 16 as at 4 (0 0 1
    /// 0 0 0 0 0 0 0 0 0 1 1 0 0 0 0 0), and whole Pointdevice Extra 50 went through at 16 with no hit in 113 seconds
    /// where at 4 the Extras before it took some 320 (2026-10-09). A frame came every 1.3 to 3.1 ms through its cards
    /// rather than the 1.04 sixteen asks for, the rules' steering 2 ms of it through cards 173 and 174 — so the pace is
    /// a ceiling the steering sets the frame under, and nothing a hit depends on.
    pub fn pace(&self) -> u32 {
        match (self.within == A_WHOLE_GAME, self.game) {
            (true, Game::Th11) => 16,
            _ => 1,
        }
    }
}

/// Every case, in the order they are played.
pub fn cases() -> Vec<Case> {
    let mut all = Vec::new();
    for game in Game::ALL {
        all.extend(cases_of(game));
    }
    all
}

/// **The case of `game` named `name`**, which is how [`Case::before`] names the cases whose recipes a case plays
/// first.
pub fn case_of(game: Game, name: &str) -> Option<Case> {
    cases_of(game).into_iter().find(|case| case.name == name)
}

/// The cases of one game, in order.
///
/// **th06nc's are its own list and not the shared one.** What that list is about is a run picked up, a
/// chapter taken again, a replay watched and a mode forked — `Th06nc::rewinds` answering true, one launch
/// of it having taken nineteen chapter restores without a fault, written its run to `pointdevice_resume`
/// and given the run up to the title — and every check of a chapter in it is th06c's.
///
/// **What keeps the list apart is the game's own front end and records**: recipes walking this game's
/// screen numbers, the attempt at a card its run build loses, and clears with no ranking of the game's own.
/// **And what kept its checks apart once was `game/th06nc/chapters.rs`**, whose seven rows were empty
/// when these cases were written: six of them were th06c's less the wait for a chapter after the one
/// retried, the recipes stopping inside the stage's first chapter. The table holds th06c's rows now, moved
/// onto this game's clock, and the recipes play on to the chapters th06c's play to.
pub fn cases_of(game: Game) -> Vec<Case> {
    match game {
        Game::Th06 => the_cases_of_a_game_orb_rewinds(game),
        Game::Th07 => {
            let mut cases = the_cases_of_a_game_orb_rewinds(game);
            cases.extend(the_card_cases_of_th07());
            cases
        }
        Game::Th06c => {
            let mut cases = the_cases_of_a_game_orb_rewinds(game);
            cases.extend(the_no_hit_cases_of_th06c());
            cases
        }
        Game::Th06nc => the_cases_of_th06nc(),
        Game::Th08 => the_cases_of_th08(),
        Game::Th10 => the_cases_of_th10(),
        Game::Th11 => the_cases_of_th11(),
    }
}

/// **The cases th11 has**, one per recipe in `recipes/th11/` — `ls crates/orb-real/recipes/th11 | wc -l` counts
/// them — which are th10's under this game's own recipes, in th10's order and with th10's starts: the two about
/// the seam, then the mode's over this game's first and second stages, the whole game played to its clear in the
/// mode, the Extra behind a clear of the game's own mode, the card counted and kept, and the Normal run with no
/// hit and no bomb last.
///
/// **The chapters they are over are the first stage's start, its one midstage row and its fight's**: a stage is cut
/// where it begins, at the rows of its midstage table (`orb_core::game::th11::chapters`), and where a boss arrives,
/// declares a card, or is beaten. Stage 1's chapter 2 is its row at frame 1005, chapter 3 the midboss's arrival at
/// 2111, chapter 4 the stage after she is beaten — 3947 on one launch of the real game and 3361 on another, the
/// frame being how quickly she went down — and chapter 5 the boss's arrival, 6086 and 5749 on those two; stage 2's
/// chapter 2 is its row at 957, chapter 3 its midboss's arrival at 2521 and chapter 4 her card, 11, at 3052
/// (2026-10-07, the rows 2026-10-10). Stage 1's boss's first card began chapter 6 at frame 6463 and her second at
/// 8223 on the launch that played the stage through, 2026-10-07, when the nonspell between them was no chapter —
/// `Th11::read_state` reads the boss's own attack clock between cards now, which makes it chapter 7.
fn the_cases_of_th11() -> Vec<Case> {
    vec![
        Case {
            game: Game::Th11,
            name: "a_run_started_and_given_up",
            recipe: include_str!("../recipes/th11/a_run_started_and_given_up.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_of_th11_started_and_given_up,
        },
        Case {
            game: Game::Th11,
            name: "a_normal_mode_run_played_without_a_bomb",
            recipe: include_str!("../recipes/th11/a_normal_mode_run_played_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_normal_mode_run_of_th11_played_without_a_bomb,
        },
        // **And one stage practised on its own**, the main menu's item 2 — **from records of a Normal clear**, for
        // th10's reason: the practice's stage select takes a decide only for a stage the score file marks offered
        // (0x43d80a), the mark is made where a run not replayed wins that stage (0x41eb61, 0x41eb92), and the mode's
        // question reads the marks out of the game's own file, which only a run of the game's own mode writes.
        Case {
            game: Game::Th11,
            name: "a_stage_practised_on_its_own",
            recipe: include_str!("../recipes/th11/a_stage_practised_on_its_own.yaml"),
            before: &[],
            starts_from: Start::ANormalClear,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
        // **And the mode's own case**, which a launch of this walk by hand played, 2026-10-07: hit in chapter 1 at
        // frame 199, `restore` putting back 14 regions, the run back on frame 9 of the chapter that began on 8, and on
        // into chapter 2 at frame 2111.
        Case {
            game: Game::Th11,
            name: "a_death_answered_by_the_chapter_again",
            recipe: include_str!("../recipes/th11/a_death_answered_by_the_chapter_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_of_th11_answered_by_the_chapter_again,
        },
        // **And the run that case left, started over from its own beginning with its replay kept**, and the attempt
        // of that replay the death rewound the run from watched, off orb's own list behind the main menu's item 3 —
        // the first chapter's first of two, which nothing but orb's replay keeps.
        Case {
            game: Game::Th11,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: include_str!("../recipes/th11/the_run_started_over_and_its_replay_kept.yaml"),
            before: &["a_death_answered_by_the_chapter_again"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept_without_asking,
        },
        Case {
            game: Game::Th11,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: include_str!("../recipes/th11/an_attempt_of_the_kept_replay_watched.yaml"),
            before: &[
                "a_death_answered_by_the_chapter_again",
                "the_run_started_over_and_its_replay_kept",
            ],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_a_retry_rewound_away_watched,
        },
        // **And the same chapter asked for without dying for it**, the game's own pause menu item orb takes off —
        // the rewinding games' check, whose lines are the log's own.
        Case {
            game: Game::Th11,
            name: "a_chapter_taken_again_without_a_death",
            recipe: include_str!("../recipes/th11/a_chapter_taken_again_without_a_death.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        // **And a run left in a chapter for a resume to be offered out of, and that run picked up**, which a launch
        // of the real game measured before these cases were written: a run left in stage 1's third chapter, picked
        // up, read `the landing is the frame that was written down, field for field`, 2026-10-07.
        Case {
            game: Game::Th11,
            name: "a_run_left_in_its_second_chapter",
            recipe: include_str!("../recipes/th11/a_run_left_in_its_second_chapter.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game: Game::Th11,
            name: "the_run_picked_up_where_it_was_left",
            recipe: include_str!("../recipes/th11/the_run_picked_up_where_it_was_left.yaml"),
            before: &["a_run_left_in_its_second_chapter"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        // **And a chapter of the stage after the first, taken again and played past**, and the run left there picked
        // up in that stage — which for this game is a stage begun out of the fade the stage before it ends on, a
        // pick-up of it putting that back too (`resume::Numbers::began_out_of_a_fade`).
        Case {
            game: Game::Th11,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: include_str!("../recipes/th11/a_chapter_of_the_stage_after_the_first.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        Case {
            game: Game::Th11,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: include_str!(
                "../recipes/th11/the_run_picked_up_in_the_stage_it_walked_into.yaml"
            ),
            before: &["a_chapter_of_the_stage_after_the_first"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        // **And a run of the other mode, which orb keeps nothing of** — the check the rewinding games use, the lines
        // it reads being orb's own.
        Case {
            game: Game::Th11,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: include_str!("../recipes/th11/a_normal_mode_run_nothing_is_kept_of.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // **And the other three ways the question offers**, whose items are `retry_ui::RetryMenu::new`'s own order —
        // see `the_cases_of_th08`, whose recipes ask at the same depths.
        Case {
            game: Game::Th11,
            name: "the_stage_taken_again_from_its_start",
            recipe: include_str!("../recipes/th11/the_stage_taken_again_from_its_start.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game: Game::Th11,
            name: "the_run_given_up_at_the_retry",
            recipe: include_str!("../recipes/th11/the_run_given_up_at_the_retry.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game: Game::Th11,
            name: "a_chapter_further_back_taken_again",
            recipe: include_str!("../recipes/th11/a_chapter_further_back_taken_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **And the same list over a death**, and a death in the nonspell after a card — the cases the rewinding
        // games have of these names, with their checks.
        Case {
            game: Game::Th11,
            name: "a_death_answered_by_a_chapter_further_back",
            recipe: include_str!("../recipes/th11/a_death_answered_by_a_chapter_further_back.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_a_chapter_further_back,
        },
        Case {
            game: Game::Th11,
            name: "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
            recipe: include_str!(
                "../recipes/th11/a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell,
        },
        // **And the whole game played to its clear in the mode orb exists for**, with no bomb pressed.
        Case {
            game: Game::Th11,
            name: "a_run_played_to_its_clear_without_a_bomb",
            recipe: include_str!("../recipes/th11/a_run_played_to_its_clear_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th11_played_to_its_clear_without_a_bomb,
        },
        // **And the Normal clear of the game's own mode, and the Extra it lights** — the main menu offers the Extra
        // off the endings the score object has seen (0x439350 at 0x43a0c3), and only a run of the game's own mode
        // writes the game's own file. The Extra's cases start from records of that clear and not from the clear's
        // launch: whether the clear lights the Extra is the clear's own check.
        Case {
            game: Game::Th11,
            name: "a_normal_mode_run_played_to_its_clear",
            recipe: include_str!("../recipes/th11/a_normal_mode_run_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_of_th11_played_to_its_clear,
        },
        Case {
            game: Game::Th11,
            name: "the_extra_played_to_its_clear",
            recipe: include_str!("../recipes/th11/the_extra_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th11_played_to_its_clear,
        },
        Case {
            game: Game::Th11,
            name: "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th11/the_extra_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th11_played_to_its_clear_with_no_hit_and_no_bomb,
        },
        Case {
            game: Game::Th11,
            name: "a_chapter_of_the_extra_taken_again",
            recipe: include_str!("../recipes/th11/a_chapter_of_the_extra_taken_again.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: WITHIN,
            check: checks::the_extra_a_cleared_run_unlocked,
        },
        // **An attempt at a card counted and kept**, th10's case over this game's score object.
        Case {
            game: Game::Th11,
            name: "an_attempt_at_a_card_counted_and_kept",
            recipe: include_str!("../recipes/th11/an_attempt_at_a_card_counted_and_kept.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_at_a_card_of_th11_counted_and_kept,
        },
        // **And the Normal run cleared with no hit and no bomb.**
        Case {
            game: Game::Th11,
            name: "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th11/a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_run_of_th11_played_to_its_clear_with_no_hit_and_no_bomb,
        },
    ]
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
/// the table's one boundary of stage 1 is script 1352, so a case asking for stage 1's second chapter asks for
/// a chapter of the midsection, and its third is the midboss's arrival at 2935.
///
/// **Each case its own** — see [`Case::before`] and [`Case::starts_from`]: the run started over with its replay
/// kept plays the death that leaves one first, and the attempt watched plays both; each pick-up plays the launch
/// that leaves its run; Kaguya's clear starts from records holding Eirin's, and the Extra from records holding both.
fn the_cases_of_th08() -> Vec<Case> {
    vec![
        Case {
            game: Game::Th08,
            name: "a_run_started_and_given_up",
            recipe: include_str!("../recipes/th08/a_run_started_and_given_up.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_of_th08_started_and_given_up,
        },
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_played_without_a_bomb",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_played_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
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
            before: &[],
            starts_from: Start::Nothing,
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
            before: &["a_death_answered_by_the_chapter_again"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept_without_asking,
        },
        // **And the attempt of that replay the death rewound the run from, watched**, off orb's own list
        // behind the title's fifth item: the second chapter's first of two, which nothing but orb's replay
        // keeps. Session 13 watched `attempt 1 of 1` of a run that had died nowhere, which says nothing of
        // an attempt a retry threw away.
        Case {
            game: Game::Th08,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: include_str!("../recipes/th08/an_attempt_of_the_kept_replay_watched.yaml"),
            before: &[
                "a_death_answered_by_the_chapter_again",
                "the_run_started_over_and_its_replay_kept",
            ],
            starts_from: Start::Nothing,
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
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        // **And a run left in a chapter for a resume to be offered out of**, which is the same check
        // again.
        Case {
            game: Game::Th08,
            name: "a_run_left_in_its_second_chapter",
            recipe: include_str!("../recipes/th08/a_run_left_in_its_second_chapter.yaml"),
            before: &[],
            starts_from: Start::Nothing,
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
            before: &["a_run_left_in_its_second_chapter"],
            starts_from: Start::Nothing,
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
            before: &[],
            starts_from: Start::Nothing,
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
            before: &["a_chapter_of_the_stage_after_the_first"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        // **And a run of the other mode, which orb keeps nothing of** — the check the rewinding games use, the
        // lines it reads being orb's own.
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_nothing_is_kept_of.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // **And the whole game played to its clear in the mode orb exists for**, with no bomb pressed and a
        // Last Spell taken — last, because it is the case that costs the real game half an hour.
        Case {
            game: Game::Th08,
            name: "a_run_played_to_its_clear_without_a_bomb",
            recipe: include_str!("../recipes/th08/a_run_played_to_its_clear_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th08_played_to_its_clear_without_a_bomb,
        },
        // **And the two clears that light the Extra, and the Extra played to its clear with the bomb allowed** —
        // this game lights Extra Start only out of Kaguya's clear, a run goes to Kaguya only where the records hold
        // Eirin's stage passed, and both have to be the game's own mode, a pointdevice run's records going to the
        // mode's own file. See `asm.md`'s *What a clear writes into a `CLRD` record*. Kaguya's starts from records
        // of Eirin's, and the Extra's from records of both.
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_played_to_its_clear",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_of_th08_played_to_its_clear,
        },
        Case {
            game: Game::Th08,
            name: "a_normal_mode_run_played_to_kaguyas_clear",
            recipe: include_str!("../recipes/th08/a_normal_mode_run_played_to_kaguyas_clear.yaml"),
            before: &[],
            starts_from: Start::ANormalClear,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_of_th08_played_to_kaguyas_clear,
        },
        Case {
            game: Game::Th08,
            name: "the_extra_played_to_its_clear",
            recipe: include_str!("../recipes/th08/the_extra_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th08_played_to_its_clear,
        },
        // **And a chapter of the Extra taken again**, which is what the rewinding games' Extra case holds
        // and the whole-run one above does not: a case of its own rather than an ask inside that run — whose
        // check is the clear, and an ask there would be one more thing between the run and its end.
        Case {
            game: Game::Th08,
            name: "a_chapter_of_the_extra_taken_again",
            recipe: include_str!("../recipes/th08/a_chapter_of_the_extra_taken_again.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: WITHIN,
            check: checks::a_chapter_of_th08s_extra_taken_again,
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
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game: Game::Th08,
            name: "the_run_given_up_at_the_retry",
            recipe: include_str!("../recipes/th08/the_run_given_up_at_the_retry.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game: Game::Th08,
            name: "a_chapter_further_back_taken_again",
            recipe: include_str!("../recipes/th08/a_chapter_further_back_taken_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **And the same list over a death**, and a death in the nonspell after a card — the cases the
        // rewinding games have of these names, with their checks.
        Case {
            game: Game::Th08,
            name: "a_death_answered_by_a_chapter_further_back",
            recipe: include_str!("../recipes/th08/a_death_answered_by_a_chapter_further_back.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_a_chapter_further_back,
        },
        Case {
            game: Game::Th08,
            name: "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
            recipe: include_str!(
                "../recipes/th08/a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell,
        },
        // **And one stage practised on its own**, the title's fourth item, which a copy with nothing cleared
        // draws and whose stage select offers that copy the first stage.
        Case {
            game: Game::Th08,
            name: "a_stage_practised_on_its_own",
            recipe: include_str!("../recipes/th08/a_stage_practised_on_its_own.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
    ]
}

/// **The cases th10 has**, one per recipe in `recipes/th10/` — `ls crates/orb-real/recipes/th10 | wc -l`
/// counts them — which are th08's first two, about the seam, then the mode's over this game's first and
/// second stages, the whole game played to its clear in the mode, and the Extra behind a clear of the game's
/// own mode last.
///
/// **The chapters they are over are the first stage's start and its fight's**: a stage is cut where it begins,
/// at the rows of its midstage table (`orb_core::game::th10::chapters`) and at each attack of a fight as it is
/// fought. Stage 1's chapter 2 is the midboss's first attack and chapter 3 the attack after it, which launches
/// of the real game read at frames 2521 and 3247 of the stage (scripts 2201 and 2927), 2026-10-01.
///
/// **Each case its own** — see [`Case::before`] and [`Case::starts_from`]: the stage practised starts from records
/// of a Normal clear, which offer it; the run started over with its replay kept plays the death that leaves one
/// first, and the attempt watched plays both; each pick-up plays the launch that leaves its run; and the Extra's
/// cases start from records of the clear that offers it.
fn the_cases_of_th10() -> Vec<Case> {
    vec![
        Case {
            game: Game::Th10,
            name: "a_run_started_and_given_up",
            recipe: include_str!("../recipes/th10/a_run_started_and_given_up.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_of_th10_started_and_given_up,
        },
        Case {
            game: Game::Th10,
            name: "a_normal_mode_run_played_without_a_bomb",
            recipe: include_str!("../recipes/th10/a_normal_mode_run_played_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_normal_mode_run_of_th10_played_without_a_bomb,
        },
        // **And one stage practised on its own**, the main menu's item 2 — **from records of a Normal clear**,
        // because th10 offers a stage to practise only once a run of that shot type and difficulty has cleared
        // it: the stage's end marks it (0x416c08) and the practice select draws nothing it has not marked
        // (0x4314be). From the score files taken aside the stage screen stood with nothing to choose, `decide`
        // doing nothing for the 900 frames its step waits (2026-10-02, `no run was started after the mode
        // question`), and so it did after a run in the mode quit in stage 1 — the mode's question taking the
        // offers out of the game's own file, which only a run of the game's own mode writes.
        Case {
            game: Game::Th10,
            name: "a_stage_practised_on_its_own",
            recipe: include_str!("../recipes/th10/a_stage_practised_on_its_own.yaml"),
            before: &[],
            starts_from: Start::ANormalClear,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
        // **And the mode's own case**, which a launch of this walk by hand played, 2026-10-01: hit in
        // chapter 1 at script 561, `restore: skipping 27 range(s), 90744 bytes`, six regions put back, and
        // the run playing on into chapter 2 at frame 2521.
        Case {
            game: Game::Th10,
            name: "a_death_answered_by_the_chapter_again",
            recipe: include_str!("../recipes/th10/a_death_answered_by_the_chapter_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_of_th10_answered_by_the_chapter_again,
        },
        // **And the run that case left, started over from its own beginning with its replay kept**, and the
        // attempt of that replay the death rewound the run from watched, off orb's own list behind the main
        // menu's item 3 — the first chapter's first of two, which nothing but orb's replay keeps. Both are
        // played over a laid-out th10 by `orb-e2e`'s `watching_a_pointdevice_replay_of_th10`, from these
        // recipes.
        Case {
            game: Game::Th10,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: include_str!("../recipes/th10/the_run_started_over_and_its_replay_kept.yaml"),
            before: &["a_death_answered_by_the_chapter_again"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept_without_asking,
        },
        Case {
            game: Game::Th10,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: include_str!("../recipes/th10/an_attempt_of_the_kept_replay_watched.yaml"),
            before: &[
                "a_death_answered_by_the_chapter_again",
                "the_run_started_over_and_its_replay_kept",
            ],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_a_retry_rewound_away_watched,
        },
        // **And the same chapter asked for without dying for it**, the game's own pause menu item orb takes
        // off. The check is the one the rewinding games use: what it reads is the log's own lines and the
        // run's deaths either side of the ask, and neither is a game's own.
        Case {
            game: Game::Th10,
            name: "a_chapter_taken_again_without_a_death",
            recipe: include_str!("../recipes/th10/a_chapter_taken_again_without_a_death.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        // **And a run left in a chapter for a resume to be offered out of, and that run picked up**, which a
        // launch of the real game measured before these cases were written: a run left in stage 1's third
        // chapter, picked up, read `the landing is the frame that was written down, field for field`.
        Case {
            game: Game::Th10,
            name: "a_run_left_in_its_second_chapter",
            recipe: include_str!("../recipes/th10/a_run_left_in_its_second_chapter.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game: Game::Th10,
            name: "the_run_picked_up_where_it_was_left",
            recipe: include_str!("../recipes/th10/the_run_picked_up_where_it_was_left.yaml"),
            before: &["a_run_left_in_its_second_chapter"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        // **And a chapter of the stage after the first, taken again and played past** — which is what says
        // the chapters are the stage's own: a snapshot taken in stage 2 puts the run back into stage 2 — and
        // the run left there picked up in that stage, the build a resume answers for pointed at the stage the
        // file names, which for this game is a stage after one won built with the numbers carried (0x4179e9).
        Case {
            game: Game::Th10,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: include_str!("../recipes/th10/a_chapter_of_the_stage_after_the_first.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        Case {
            game: Game::Th10,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: include_str!(
                "../recipes/th10/the_run_picked_up_in_the_stage_it_walked_into.yaml"
            ),
            before: &["a_chapter_of_the_stage_after_the_first"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        // **And a run of the other mode, which orb keeps nothing of** — the check the rewinding games use,
        // the lines it reads being orb's own.
        Case {
            game: Game::Th10,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: include_str!("../recipes/th10/a_normal_mode_run_nothing_is_kept_of.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // **And the other three ways the question offers**, whose items are `retry_ui::RetryMenu::new`'s own
        // order — see `the_cases_of_th08`, whose recipes ask at the same depths.
        Case {
            game: Game::Th10,
            name: "the_stage_taken_again_from_its_start",
            recipe: include_str!("../recipes/th10/the_stage_taken_again_from_its_start.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game: Game::Th10,
            name: "the_run_given_up_at_the_retry",
            recipe: include_str!("../recipes/th10/the_run_given_up_at_the_retry.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game: Game::Th10,
            name: "a_chapter_further_back_taken_again",
            recipe: include_str!("../recipes/th10/a_chapter_further_back_taken_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **And the same list over a death**, and a death in the nonspell after a card — the cases the
        // rewinding games have of these names, with their checks.
        Case {
            game: Game::Th10,
            name: "a_death_answered_by_a_chapter_further_back",
            recipe: include_str!("../recipes/th10/a_death_answered_by_a_chapter_further_back.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_a_chapter_further_back,
        },
        Case {
            game: Game::Th10,
            name: "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
            recipe: include_str!(
                "../recipes/th10/a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell,
        },
        // **And the whole game played to its clear in the mode orb exists for**, with no bomb pressed — last,
        // because it is the case that costs the real game the most.
        Case {
            game: Game::Th10,
            name: "a_run_played_to_its_clear_without_a_bomb",
            recipe: include_str!("../recipes/th10/a_run_played_to_its_clear_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th10_played_to_its_clear_without_a_bomb,
        },
        // **And the Normal clear of the game's own mode, and the Extra it lights** — the main menu takes its Extra
        // off the score object's clears (0x42c850 at 0x42d442), and only a run of the game's own mode writes the
        // game's own file. The Extra's cases start from records of that clear.
        Case {
            game: Game::Th10,
            name: "a_normal_mode_run_played_to_its_clear",
            recipe: include_str!("../recipes/th10/a_normal_mode_run_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_of_th10_played_to_its_clear,
        },
        Case {
            game: Game::Th10,
            name: "the_extra_played_to_its_clear",
            recipe: include_str!("../recipes/th10/the_extra_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th10_played_to_its_clear,
        },
        // **The Extra cleared with no hit and no bomb**, as th06c's and th06nc's is, in the directory the unlock is
        // in — and ahead of the chapter taken again, which leaves its run in the Extra's fourth chapter: behind it,
        // the Extra's item puts up orb's question of picking that run up, and no step here answers it.
        Case {
            game: Game::Th10,
            name: "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th10/the_extra_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th10_played_to_its_clear_with_no_hit_and_no_bomb,
        },
        Case {
            game: Game::Th10,
            name: "a_chapter_of_the_extra_taken_again",
            recipe: include_str!("../recipes/th10/a_chapter_of_the_extra_taken_again.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: WITHIN,
            check: checks::the_extra_a_cleared_run_unlocked,
        },
        // **An attempt at a card counted and kept**, th07's and th06nc's case over this game's score object.
        Case {
            game: Game::Th10,
            name: "an_attempt_at_a_card_counted_and_kept",
            recipe: include_str!("../recipes/th10/an_attempt_at_a_card_counted_and_kept.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_at_a_card_of_th10_counted_and_kept,
        },
        // **And the Normal run cleared with no hit and no bomb**, as th06c's and th06nc's is.
        Case {
            game: Game::Th10,
            name: "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th10/a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_run_of_th10_played_to_its_clear_with_no_hit_and_no_bomb,
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
/// **And a list of its own rather than the rewinding games'**, for this game's own front end and records
/// — see [`cases_of`], which also has why six of its checks were once shorter than th06c's.
///
/// **The Extra's cases start from records of a clear** ([`Start::TheExtraOffered`]): this game offers its Extra only
/// once a run has been cleared, and the records that say so are its score file.
fn the_cases_of_th06nc() -> Vec<Case> {
    vec![
        // **The mode's own case**: a death answered by the chapter again, and the run played on into the
        // chapters of the fight after it — th06c's check, which waits for a chapter past the one died in.
        Case {
            game: Game::Th06nc,
            name: "a_death_answered_by_the_chapter_again",
            recipe: include_str!("../recipes/th06nc/a_death_answered_by_the_chapter_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_the_chapter_again,
        },
        // **And the run that case left, started over rather than picked up**, which is the other answer
        // the resume question takes: the replay of the run being written over is asked about and kept,
        // and the fresh run plays on from the stage's own start into its second chapter.
        Case {
            game: Game::Th06nc,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: include_str!("../recipes/th06nc/the_run_started_over_and_its_replay_kept.yaml"),
            before: &["a_death_answered_by_the_chapter_again"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept,
        },
        // **And one attempt of that replay watched**, off orb's own four-deep list: this game records no
        // replay a rewound run would fit, so the main menu's item 4 — the replays — is a question of
        // orb's and the press that would open the game's own list is held back. The other games ask over
        // their *title*'s item; the check is theirs unchanged, every line of it being orb's own lists.
        Case {
            game: Game::Th06nc,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: include_str!("../recipes/th06nc/an_attempt_of_the_kept_replay_watched.yaml"),
            before: &[
                "a_death_answered_by_the_chapter_again",
                "the_run_started_over_and_its_replay_kept",
            ],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_of_the_kept_replay_watched,
        },
        // **A death in a fight's nonspell after a card, answered by that nonspell again** — the mode's promise
        // over a fight, each of a boss's attacks a chapter, which none of the cases before this one held.
        Case {
            game: Game::Th06nc,
            name: "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
            recipe: include_str!(
                "../recipes/th06nc/a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell,
        },
        // **The other three answers the same question takes**, each th06c's case over this game's own walk:
        // the chapter again asked for without a death, the stage again from its own start — asked in the
        // midboss's card, so the stage is the third of four ways on — and the run given up, which leaves the
        // run for the front end.
        Case {
            game: Game::Th06nc,
            name: "a_chapter_taken_again_without_a_death",
            recipe: include_str!("../recipes/th06nc/a_chapter_taken_again_without_a_death.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        Case {
            game: Game::Th06nc,
            name: "the_stage_taken_again_from_its_start",
            recipe: include_str!("../recipes/th06nc/the_stage_taken_again_from_its_start.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game: Game::Th06nc,
            name: "the_run_given_up_at_the_retry",
            recipe: include_str!("../recipes/th06nc/the_run_given_up_at_the_retry.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            // **th06c's own check, unchanged**: nothing in it asks about a chapter after the one the run
            // was in, so there is nothing of this game's for it to be wrong about.
            check: checks::the_run_given_up_at_the_retry,
        },
        // **And a chapter further back**, which wants two of a stage's own boundaries behind the one the
        // run is in — the fight's. th06c's check unchanged.
        Case {
            game: Game::Th06nc,
            name: "a_chapter_further_back_taken_again",
            recipe: include_str!("../recipes/th06nc/a_chapter_further_back_taken_again.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **And the same list over a death**, which is where somebody playing meets it — the case the
        // rewinding games have of this name.
        Case {
            game: Game::Th06nc,
            name: "a_death_answered_by_a_chapter_further_back",
            recipe: include_str!(
                "../recipes/th06nc/a_death_answered_by_a_chapter_further_back.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_a_chapter_further_back,
        },
        // **The run left in its second chapter and picked up there** — the pair a stage's *own* boundary
        // is needed for, which stage 1 of this game has first where its midboss arrives, its one row of
        // `game/th06nc/chapters.rs` coming after the midboss. Both checks are th06c's unchanged, every line
        // of them being orb's rather than a game's.
        Case {
            game: Game::Th06nc,
            name: "a_run_left_in_its_second_chapter",
            recipe: include_str!("../recipes/th06nc/a_run_left_in_its_second_chapter.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game: Game::Th06nc,
            name: "the_run_picked_up_where_it_was_left",
            recipe: include_str!("../recipes/th06nc/the_run_picked_up_where_it_was_left.yaml"),
            before: &["a_run_left_in_its_second_chapter"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        // **The stage after the first, and the run left there picked up in it** — the pair of cases about
        // a stage a run *walked into* rather than started at: the second stage's chapter 1 is reached by
        // playing the first stage out, and its second is that stage's first row of the table. Both checks
        // are th06c's.
        Case {
            game: Game::Th06nc,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: include_str!("../recipes/th06nc/a_chapter_of_the_stage_after_the_first.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        Case {
            game: Game::Th06nc,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: include_str!(
                "../recipes/th06nc/the_run_picked_up_in_the_stage_it_walked_into.yaml"
            ),
            before: &["a_chapter_of_the_stage_after_the_first"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        // **The attempt a retry inside a spell card counts, and the run after it keeping that count** —
        // the case both of orb's dealings with this game's card records want, and the one case here that
        // plays two runs in a single launch: the put-back is spent at the build of a run's first stage,
        // which is the read that would otherwise lose the count.
        Case {
            game: Game::Th06nc,
            name: "an_attempt_at_a_card_counted_and_kept",
            recipe: include_str!("../recipes/th06nc/an_attempt_at_a_card_counted_and_kept.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_at_a_card_of_th06nc_counted_and_kept,
        },
        // **The other mode's own run, which wants no chapter of anything**: a run with no chapters is
        // what the normal mode is. th06c's own check, unchanged — every line it reads is orb's and not a
        // game's.
        Case {
            game: Game::Th06nc,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: include_str!("../recipes/th06nc/a_normal_mode_run_nothing_is_kept_of.yaml"),
            before: &[],
            starts_from: Start::Nothing,
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
            before: &[],
            starts_from: Start::Nothing,
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
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th06nc_played_to_its_clear,
        },
        // **And the same six in the game's own mode, which is the clear the Extra is unlocked by**: this
        // one answers orb's question with レガシーモード and takes no bomb, so what it leaves is a clear in
        // `score.dat` — a pointdevice clear goes to the mode's own file and lights nothing, see
        // `orb_core::score`.
        Case {
            game: Game::Th06nc,
            name: "a_run_played_to_its_clear_without_a_bomb",
            recipe: include_str!("../recipes/th06nc/a_run_played_to_its_clear_without_a_bomb.yaml"),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_run_of_th06nc_played_to_its_clear_without_a_bomb,
        },
        Case {
            game: Game::Th06nc,
            name: "the_extra_played_to_its_clear",
            recipe: include_str!("../recipes/th06nc/the_extra_played_to_its_clear.yaml"),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th06nc_played_to_its_clear,
        },
        // **The two clears with no hit and no bomb**, as th06c's are: the Extra's, from records of a clear, and the
        // Normal run.
        Case {
            game: Game::Th06nc,
            name: "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th06nc/the_extra_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: A_WHOLE_GAME,
            check: checks::the_extra_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb,
        },
        Case {
            game: Game::Th06nc,
            name: "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th06nc/a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_run_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb,
        },
    ]
}

/// **th06c's two clears with no hit and no bomb**, after the rest of its cases: the Extra's, from records of a
/// clear — the Extra being offered only where the game's own records hold one — and the Normal run.
///
/// **Played at the game's own speed and not [`A_WHOLE_GAME`]'s four**, which is what that bound would give
/// the Normal run: every measurement the two recipes' ways were read off was a launch at one, and the Extra
/// fits [`WITHIN`] at it.
fn the_no_hit_cases_of_th06c() -> Vec<Case> {
    vec![
        Case {
            game: Game::Th06c,
            name: "the_extra_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th06c/the_extra_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::TheExtraOffered,
            within: WITHIN,
            check: checks::the_extra_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb,
        },
        Case {
            game: Game::Th06c,
            name: "a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb",
            recipe: include_str!(
                "../recipes/th06c/a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb.yaml"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: A_NO_HIT_RUN,
            check: checks::a_normal_run_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb,
        },
    ]
}

/// **th07's attempt at a card counted and kept**, after the rest of its cases: the two runs are one launch, and
/// the records it counts into are the game's own taken aside, so what the second run's build reads the file back
/// over is this launch's count and nothing another case left.
///
/// **th07's alone of the four rewinding games' lists**, because it is the one whose count is lost at a read
/// orb brackets: th06 and th06c carry their captures in memory, and th06nc's case of this name is its own
/// list's.
fn the_card_cases_of_th07() -> Vec<Case> {
    vec![Case {
        game: Game::Th07,
        name: "an_attempt_at_a_card_counted_and_kept",
        recipe: include_str!("../recipes/th07/an_attempt_at_a_card_counted_and_kept.yaml"),
        before: &[],
        starts_from: Start::Nothing,
        within: WITHIN,
        check: checks::an_attempt_at_a_card_of_th07_counted_and_kept,
    }]
}

/// **What th06c's Normal run with no hit may take**, a whole game at the game's own speed — its own number,
/// because [`Case::speed`] reads [`A_WHOLE_GAME`] as a case to play at four updates a frame, which none of
/// what this run's ways were measured by was. Six stages at sixty frames a second are forty minutes and a
/// little, and every death retried on top.
pub const A_NO_HIT_RUN: u32 = 100 * 60;

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
                Game::Th10 => unreachable!("th10 has its own cases"),
                Game::Th11 => unreachable!("th11 has its own cases"),
            }
        };
    }
    vec![
        Case {
            game,
            name: "a_run_left_in_its_second_chapter",
            recipe: recipe!(game, "a_run_left_in_its_second_chapter"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game,
            name: "the_run_picked_up_where_it_was_left",
            recipe: recipe!(game, "the_run_picked_up_where_it_was_left"),
            before: &["a_run_left_in_its_second_chapter"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        Case {
            game,
            name: "a_death_answered_by_the_chapter_again",
            recipe: recipe!(game, "a_death_answered_by_the_chapter_again"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_the_chapter_again,
        },
        // The two that watch what that death left: the run it gave up is what はじめから is answered about,
        // and the replay kept there is the one with two attempts at the chapter it died in.
        Case {
            game,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: recipe!(game, "the_run_started_over_and_its_replay_kept"),
            before: &["a_death_answered_by_the_chapter_again"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept,
        },
        Case {
            game,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: recipe!(game, "an_attempt_of_the_kept_replay_watched"),
            before: &[
                "a_death_answered_by_the_chapter_again",
                "the_run_started_over_and_its_replay_kept",
            ],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::an_attempt_of_the_kept_replay_watched,
        },
        Case {
            game,
            name: "a_chapter_taken_again_without_a_death",
            recipe: recipe!(game, "a_chapter_taken_again_without_a_death"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        Case {
            game,
            name: "a_chapter_further_back_taken_again",
            recipe: recipe!(game, "a_chapter_further_back_taken_again"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        // **And the same list over a death**, which is where somebody playing meets it: the one above asks
        // for it over a run asked to start over, which is nobody's death.
        Case {
            game,
            name: "a_death_answered_by_a_chapter_further_back",
            recipe: recipe!(game, "a_death_answered_by_a_chapter_further_back"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_answered_by_a_chapter_further_back,
        },
        // **A death in the attack a boss moves to after a card**, answered by that attack again — th06nc's
        // case of this name, where a fight whose nonspells were no chapters of their own was first missed.
        Case {
            game,
            name: "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell",
            recipe: recipe!(
                game,
                "a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell"
            ),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell,
        },
        Case {
            game,
            name: "the_stage_taken_again_from_its_start",
            recipe: recipe!(game, "the_stage_taken_again_from_its_start"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game,
            name: "the_run_given_up_at_the_retry",
            recipe: recipe!(game, "the_run_given_up_at_the_retry"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: recipe!(game, "a_chapter_of_the_stage_after_the_first"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        // And the run that case left in the stage it walked into, picked up in that stage.
        Case {
            game,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: recipe!(game, "the_run_picked_up_in_the_stage_it_walked_into"),
            before: &["a_chapter_of_the_stage_after_the_first"],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        Case {
            game,
            name: "a_stage_practised_on_its_own",
            recipe: recipe!(game, "a_stage_practised_on_its_own"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
        Case {
            game,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: recipe!(game, "a_normal_mode_run_nothing_is_kept_of"),
            before: &[],
            starts_from: Start::Nothing,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // And the three that play whole games, last because they are the cases that cost an hour apiece: the
        // run's own end is reachable no other way, and the Extra behind them is reachable only from a clear.
        Case {
            game,
            name: "a_run_played_to_its_clear",
            recipe: recipe!(game, "a_run_played_to_its_clear"),
            before: &[],
            starts_from: Start::Nothing,
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
            before: &[],
            starts_from: Start::Nothing,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_played_to_its_clear,
        },
        // And the Extra a clear unlocks, **started from records of a clear** ([`Start::TheExtraOffered`]): a game
        // with no record of one does not offer the Extra at all.
        Case {
            game,
            name: "the_extra_a_cleared_run_unlocked",
            recipe: recipe!(game, "the_extra_a_cleared_run_unlocked"),
            before: &[],
            starts_from: Start::TheExtraOffered,
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

    /// **th10's bomb and its four directions in the word a run is played with** — `orb_core::game::th10`'s
    /// `BUTTON_BOMB` and `PAD_UP` with its three neighbours, the same bits as th08's, written out for the
    /// reason [`TH08_BOMB`] is.
    const TH10_BOMB: u16 = 0x0002;
    const TH10_DIRECTIONS: u16 = 0x0010 | 0x0020 | 0x0040 | 0x0080;

    /// **th08's run started by a recipe and handed back to the game's own title.**
    ///
    /// **Which is the case that says orb can take a run of this game down**, and the first attempt at
    /// that faulted: writing the wanted scene took the game out at 0x406a1a on the frame after, walking
    /// the text element the teardown had freed. What the way out is instead — two bits of the run kind
    /// and the scene the run itself wants — is `orb_core::game::th08`'s `leave_run`, and this is the
    /// launch that holds it.
    ///
    /// See [`started_and_given_up`] for what it is held to.
    pub fn a_run_of_th08_started_and_given_up(trace: &Trace) -> Result<(), String> {
        started_and_given_up(trace)
    }

    /// **th10's run started by a recipe and handed back to the game's own title**, which is the case that
    /// says `orb_core::game::th10`'s `leave_run` takes a run of this game down: the scene wanted put at the
    /// title, the write the pause menu's own item for the title makes, answered by the supervisor taking the
    /// game down and building the title. See [`started_and_given_up`] for what it is held to.
    pub fn a_run_of_th10_started_and_given_up(trace: &Trace) -> Result<(), String> {
        started_and_given_up(trace)
    }

    /// **th11's run started by a recipe and handed back to the game's own title**, the scene wanted put at the
    /// title as the pause menu's own item for it writes (`0x40dea0` with 4 at 0x42d4ec). See
    /// [`started_and_given_up`] for what it is held to.
    pub fn a_run_of_th11_started_and_given_up(trace: &Trace) -> Result<(), String> {
        started_and_given_up(trace)
    }

    /// **th11's bomb and its four directions in the word a run is played with** — `orb_core::game::th11`'s
    /// `BUTTON_BOMB` and `PAD_UP` with its three neighbours, th10's bits.
    const TH11_BOMB: u16 = 0x0002;
    const TH11_DIRECTIONS: u16 = 0x0010 | 0x0020 | 0x0040 | 0x0080;

    /// **th11's Normal run played to its clear with no hit and no bomb**, in the game's own mode: the run started on
    /// Normal, its ending reached, and no hit and no bomb between — [`took_no_hit_and_no_bomb`].
    pub fn a_normal_run_of_th11_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        let ending = trace
            .ending_from(started)
            .ok_or("the run never reached its ending")?;
        took_no_hit_and_no_bomb(trace, started, ending, TH11_BOMB)
    }

    /// **Three things in order**: a run in progress, the log's own line for the give-up, and the front end
    /// reading a screen after it. The last of those is read off a line and not a field because nothing in
    /// `State` carries a front end's screen — `Game::front_end_screen` is what the log prints — and the first
    /// off `State::in_run`, which for either game is the scene being a run's.
    fn started_and_given_up(trace: &Trace) -> Result<(), String> {
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
        played_without_a_bomb(trace, TH08_BOMB, TH08_DIRECTIONS, the_stage_scripts_clock)
    }

    /// **th10's Normal run played by the rules with the bomb forbidden**, held to the four things th08's
    /// is — see [`a_normal_mode_run_of_th08_played_without_a_bomb`] — and not to the stage reached either,
    /// for th08's reason: how far the rules get is the bot's to vary, where this case is about the seam.
    pub fn a_normal_mode_run_of_th10_played_without_a_bomb(trace: &Trace) -> Result<(), String> {
        played_without_a_bomb(trace, TH10_BOMB, TH10_DIRECTIONS, the_stage_scripts_clock)
    }

    /// **th11's Normal run played by the rules with the bomb forbidden**, held to th10's four things — **with the
    /// stage's own frames for its clock and not the stage scripts'**. orb's clock of this game's stage scripts stood
    /// at 60 from frame 129 of stage 1 to frame 4,305 of it on a launch of the real game, 2026-10-07, and read 360 at
    /// frame 5,105 — nowhere near the bound below on a stage played for minutes — where the stage's frames counted
    /// every one of them.
    pub fn a_normal_mode_run_of_th11_played_without_a_bomb(trace: &Trace) -> Result<(), String> {
        played_without_a_bomb(trace, TH11_BOMB, TH11_DIRECTIONS, |state| {
            i64::from(state.stage_frames)
        })
    }

    /// The stage script's own count — th08's, and th10's clock of its scripts — which a conversation stops.
    fn the_stage_scripts_clock(state: &orb_core::game::State) -> i64 {
        i64::from(state.script_frames)
    }

    /// The run entered on Normal, the stage's `clock` advancing under it, a direction in `directions`
    /// pressed inside it and never `bomb`.
    fn played_without_a_bomb(
        trace: &Trace,
        bomb: u16,
        directions: u16,
        clock: fn(&orb_core::game::State) -> i64,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        // **The clock and not the frames orb counted**, which is what says the stage is running rather
        // than the launch: `script_frames` is the stage script's own count, which a conversation stops —
        // and th10's, orb's clock of the stage's scripts, a boss up as well.
        let played = trace.records[started..]
            .iter()
            .filter_map(|record| record.state)
            .map(|state| clock(&state))
            .max()
            .unwrap_or(0);
        expect(played >= i64::from(A_STAGE_IS_UNDER_WAY), || {
            format!("the stage's own clock reached {played} frame(s)")
        })?;
        // **And a direction pressed inside the run**, which is the whole of what says the rules reached
        // the game: a launch whose `what_a_bot_sees` answered nothing would play a run holding the shot
        // and standing still where a stage puts the player. Read off the word for the reason the bomb below
        // is — it is the word the rules pressed, and nothing else the trace carries says where the player
        // went.
        let steered = trace.records[started..].iter().any(|record| {
            record.state.is_some_and(|state| state.in_run) && record.word & directions != 0
        });
        expect(steered, || {
            "the rules pressed no direction inside the run".to_owned()
        })?;
        pressed_no_bomb(trace, started, bomb)
    }

    /// **No `bomb` pressed inside a run from `started` on**, read off the word the rules pressed.
    fn pressed_no_bomb(trace: &Trace, started: usize, bomb: u16) -> Result<(), String> {
        let bombed = trace.records[started..].iter().find(|record| {
            record.state.is_some_and(|state| state.in_run) && record.word & bomb != 0
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
    ///
    /// **And the screen the run ends at, which is this game's one Pointdevice clear reaching it**: the name
    /// typed into its entry and orb's replay of the run kept in place of the game's own offer — what every
    /// other rewinding game's clear is held to, and what nothing of this game's asked before.
    pub fn a_run_of_th08_played_to_its_clear_without_a_bomb(trace: &Trace) -> Result<(), String> {
        let (started, ending) = played_to_its_clear_without_a_bomb(trace, TH08_BOMB)?;
        th08_took_a_last_spell(trace, started, ending).map(|_| ())
    }

    /// **th10's Normal run in Pointdevice mode, played to its ending with no bomb pressed** — th08's case of
    /// this name less its Last Spell, this game having none: the ending, which is this game's scene 14 or
    /// orb's own line for running it out, and the screen after it.
    pub fn a_run_of_th10_played_to_its_clear_without_a_bomb(trace: &Trace) -> Result<(), String> {
        played_to_its_clear_without_a_bomb(trace, TH10_BOMB).map(|_| ())
    }

    /// **th11's Normal run in Pointdevice mode, played to its ending with no bomb pressed** — th10's case of this
    /// name: the ending, this game's scene 15 or orb's own line for running it out, the name typed into the clear's
    /// own entry, the title's screen 14, and orb's question about the replay put up where the game's own offer of
    /// one stood, the title's screen 15.
    pub fn a_run_of_th11_played_to_its_clear_without_a_bomb(trace: &Trace) -> Result<(), String> {
        played_to_its_clear_without_a_bomb(trace, TH11_BOMB).map(|_| ())
    }

    /// **A Normal run played to its ending with `bomb` never pressed, and the screen it ends at walked**, as
    /// [`a_run_played_to_its_clear`] holds the rewinding games' clears to it: the name the ranking was
    /// registered under, and the replay of the run kept. Answers where the run started and where its ending
    /// is.
    fn played_to_its_clear_without_a_bomb(
        trace: &Trace,
        bomb: u16,
    ) -> Result<(usize, usize), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        pressed_no_bomb(trace, started, bomb)?;
        let ending = trace
            .ending_from(started)
            .ok_or("the run never reached its ending")?;
        trace
            .line_from(ending, "types ORB into the name the game asks for")
            .ok_or("the ranking was not registered under the name the recipe typed")?;
        let offered = trace
            .question_from(ending, "keep_replay")
            .ok_or("the replay of the run that was finished with was never asked about")?;
        trace
            .line_from(offered, "replay: kept")
            .ok_or("the question about that replay was not answered with keeping it")?;
        Ok((started, ending))
    }

    /// **th06c's bomb in the word a run is played with** — `orb_core`'s `th06c::button::BOMB`, written out for
    /// the reason [`TH08_BOMB`] is.
    const TH06C_BOMB: u16 = 0x0002;

    /// **A run from `started` to its clear at `cleared` with no hit and no bomb**, `bomb` being the game's
    /// bomb in the word: no `died in chapter` said between — every death a Pointdevice run takes being
    /// answered by the chapter again, so a death a run went on from is still one said — and no record
    /// inside the run whose word pressed the bomb.
    fn took_no_hit_and_no_bomb(
        trace: &Trace,
        started: usize,
        cleared: usize,
        bomb: u16,
    ) -> Result<(), String> {
        let run = &trace.records[started..cleared];
        let hit = run.iter().find(|record| {
            record
                .lines
                .iter()
                .any(|line| line.starts_with("died in chapter"))
        });
        expect(hit.is_none(), || {
            format!(
                "the run was hit, on frame {}",
                hit.map_or(0, |record| record.frame)
            )
        })?;
        let bombed = run.iter().find(|record| {
            record.state.is_some_and(|state| state.in_run) && record.word & bomb != 0
        });
        expect(bombed.is_none(), || {
            format!(
                "a bomb was pressed inside the run, on frame {}",
                bombed.map_or(0, |record| record.frame)
            )
        })
    }

    /// **th06c's Normal run in Pointdevice mode, played to its ending with no hit and no bomb** — the
    /// goal it was written for: `th06cのautoplayで被弾なしでボムを使わずにクリア`.
    pub fn a_normal_run_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        let ending = trace
            .ending_from(started)
            .ok_or("the run never reached its ending")?;
        took_no_hit_and_no_bomb(trace, started, ending, TH06C_BOMB)
    }

    /// **And th06c's Extra, played the same way** — its three cards with ways of their own, `禁忌「レーヴァテイン」`,
    /// `禁弾「過去を刻む時計」` and `秘弾「そして誰もいなくなるか？」`, and chapter 7 and 8's wait, all in its
    /// recipe.
    ///
    /// **Cleared where its boss left all but out of life and the run then ended with nothing leaving it**,
    /// the Extra having no ending to reach — see [`the_last_boss_was_beaten`]. The boss alone is not enough:
    /// a pick-up of chapter 7 the recipe left after the midboss read her beaten too, at 11,386 frames, and
    /// what it did not have is a `run ended` said before its recipe `leaves the run`.
    pub fn the_extra_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let extra = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(extra == EXTRA, || {
            format!("the run was started on difficulty {extra} and not on the Extra's {EXTRA}")
        })?;
        let cleared = the_last_boss_was_beaten(trace, started)?;
        let ended = trace
            .line_from(cleared, "run ended after")
            .ok_or("the run never ended after its last boss")?;
        let left = trace.records[started..ended].iter().find(|record| {
            record
                .lines
                .iter()
                .any(|line| line.ends_with("leaves the run"))
        });
        expect(left.is_none(), || {
            format!(
                "the recipe left the run on frame {}, so it ended without being cleared",
                left.map_or(0, |record| record.frame)
            )
        })?;
        took_no_hit_and_no_bomb(trace, started, cleared, TH06C_BOMB)
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
        let title = trace
            .line_from(ending, "front end: screen 0")
            .ok_or("the ending was never pressed through to the title")?;
        // **And Kaguya's lighting Extra Start**, the title's item 1, which it steps over where no Kaguya clear is in
        // the records (0x46799c): her clear's case is the one that says so, the Extra's own starting from records of
        // one (`Start::TheExtraOffered`).
        match last {
            TH08_KAGUYAS_STAGE => trace
                .line_from(title, "front end: screen 0, cursor on item 1")
                .map(|_| ())
                .ok_or_else(|| {
                    "the title after Kaguya's clear never had its cursor on Extra Start, so her clear lit no Extra"
                        .to_owned()
                }),
            _ => Ok(()),
        }
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

    /// **th10's Normal run in the game's own mode, played to its clear with the screen after its ending walked
    /// to the main menu** — the run the Extra is lit by: the mode question answered for the normal mode, the run
    /// on Normal, the ending, the name the ranking is registered under typed into the clear's own entry, and
    /// the main menu up again, which is where the run's records have been written.
    pub fn a_normal_mode_run_of_th10_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        a_normal_mode_run_played_to_its_clear_and_on_to(trace, "front end: screen 2,")
    }

    /// **th11's Normal run in the game's own mode, played to its clear with the screen after its ending walked to the
    /// main menu** — th10's case of this name over this game's screens: the name typed into the title's screen 14
    /// and the main menu, screen 1, up again.
    ///
    /// **And the run held to no continue by what lights the Extra, not here**: the main menu offers it off endings
    /// 6 to 11, which only a run with no continue reaches (0x40edb5 through 0x40edc0), so a run that continued and
    /// was cleared passes this and leaves the Extra's case after it with no Extra to choose.
    pub fn a_normal_mode_run_of_th11_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        a_normal_mode_run_played_to_its_clear_and_on_to(trace, "front end: screen 1,")
    }

    /// The mode question answered for the normal mode, the run on Normal, the ending, the name typed, and then the
    /// front end's line `main_menu` opens with — **with the comma**, which is what keeps th11's `screen 1,` from
    /// matching the `screen 14` and `screen 15` the ending walks through first — **and its cursor then on the Extra's
    /// item**, which the menu steps over where the records do not light it: the clear's case is the one that says a
    /// clear lights the Extra, the Extra's own starting from records of one (`Start::TheExtraOffered`).
    fn a_normal_mode_run_played_to_its_clear_and_on_to(
        trace: &Trace,
        main_menu: &str,
    ) -> Result<(), String> {
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
        let ending = trace
            .ending_from(started)
            .ok_or("nothing says an ending ran, so the run was not cleared")?;
        let named = trace
            .line_from(ending, "types ORB into the name the game asks for")
            .ok_or("the ranking was not registered under the name the recipe typed")?;
        let menu = trace
            .line_from(named, main_menu)
            .ok_or("the screen after the ending was never walked to the main menu")?;
        trace
            .line_from(menu, &format!("{main_menu} cursor on item {THE_EXTRAS_ITEM}"))
            .map(|_| ())
            .ok_or_else(|| {
                "the main menu after the clear never had its cursor on the Extra, so the clear lit no Extra"
                    .to_owned()
            })
    }

    /// The main menu's item the Extra is, th10's and th11's alike: item 1.
    const THE_EXTRAS_ITEM: i32 = 1;

    /// **th10's Extra, played to its clear in Pointdevice mode** — th08's case of this name over this game's
    /// Extra, which its shot select writes as stage 7, the stage word 6 the state reads: the run at the Extra's
    /// own difficulty, ended on its own and not given up, and the screen after it walked with the game's own menu
    /// taken off and orb's question about the replay answered with keeping it.
    pub fn the_extra_of_th10_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        the_extra_played_to_its_clear_in(trace, TH10_THE_EXTRAS_STAGE)
    }

    /// **th11's Extra, played to its clear in Pointdevice mode** — th10's case of this name over this game's Extra,
    /// whose partner select writes the stage as 7 (0x43d5b3 through 0x43d5ec), the stage word 6 the state reads.
    ///
    /// **The screen after it is the pause object's**, where th10's is too: a won run's end opens on state 21
    /// (0x42d6b0), its ranking's name entry is state 25 where the score places, and the menu after it state 22 —
    /// or, where it does not place, the lost run's menu, state 14 — and orb takes either off for its own question
    /// (`Th11::skip_replay_prompt`). The run is read at the stage it started in and not where it ended: a cleared
    /// Extra is moved on to stage 8 by its own ranking (0x42d850) and left there.
    pub fn the_extra_of_th11_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        the_extra_played_to_its_clear_in(trace, TH11_THE_EXTRAS_STAGE)
    }

    /// The run started after the mode question at the Extra's difficulty and in its stage word `stage`, ended on its
    /// own and not given up, and the screen after it walked with the game's own menu taken off and orb's question
    /// about the replay answered with keeping it.
    fn the_extra_played_to_its_clear_in(trace: &Trace, stage: i32) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that starts the Extra")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question, so the item offered none")?;
        let state = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        expect((state.difficulty, state.stage) == (EXTRA, stage), || {
            format!(
                "the run that started is at difficulty {} and stage word {}, where the Extra is \
                     {EXTRA} and {stage}",
                state.difficulty, state.stage
            )
        })?;
        expect(
            trace
                .line_from(started, "retry: the run is given up")
                .is_none(),
            || "the Extra was given up rather than played out".to_owned(),
        )?;
        let ended = trace
            .line_from(started, "run ended after")
            .ok_or("the Extra never ended, so it was not cleared")?;
        let taken_off = trace
            .line_from(
                ended,
                "replay: the offer to save one was taken off the screen a run ended on",
            )
            .ok_or("the game's own menu was left on the screen the Extra ended at")?;
        trace
            .line_from(taken_off, "replay: kept")
            .map(|_| ())
            .ok_or_else(|| "the replay of the Extra was not kept".to_owned())
    }

    /// **th10's Extra played to its clear with no hit and no bomb, in Pointdevice mode** — the clear
    /// [`the_extra_of_th10_played_to_its_clear`] holds a launch to, from the run's start to its end taking no hit
    /// and pressing no bomb: th06c's and th06nc's case of this name over this game.
    pub fn the_extra_of_th10_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        the_extra_of_th10_played_to_its_clear(trace)?;
        the_extra_took_no_hit_and_no_bomb(trace, TH10_BOMB)
    }

    /// **th11's Extra played to its clear with no hit and no bomb, in Pointdevice mode** — th10's case of this name:
    /// [`the_extra_of_th11_played_to_its_clear`], and no hit taken and no bomb pressed from the run's start to its
    /// end.
    pub fn the_extra_of_th11_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        the_extra_of_th11_played_to_its_clear(trace)?;
        the_extra_took_no_hit_and_no_bomb(trace, TH11_BOMB)
    }

    /// The Extra started after the mode question, from its start to `run ended after`, taking no hit and pressing no
    /// `bomb`.
    fn the_extra_took_no_hit_and_no_bomb(trace: &Trace, bomb: u16) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that starts the Extra")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        let ended = trace
            .line_from(started, "run ended after")
            .ok_or("the Extra never ended, so it was not cleared")?;
        took_no_hit_and_no_bomb(trace, started, ended, bomb)
    }

    /// **th10's Normal run played to its ending with no hit and no bomb, in Pointdevice mode** — the goal its
    /// ways were written for, `normalの1〜6面とextraを無被弾ボムなしでクリアする`: th06c's and th06nc's case of
    /// this name over this game.
    pub fn a_normal_run_of_th10_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        let ending = trace
            .ending_from(started)
            .ok_or("the run never reached its ending")?;
        took_no_hit_and_no_bomb(trace, started, ending, TH10_BOMB)
    }

    /// **th10's attempt at a card counted and kept**, th07's two runs over this game's own score object: an
    /// attempt counted where the first run's chapter is taken again inside a card, and the second run's retry
    /// inside the same card counting at least two more — the card's own start and the retry — which is what
    /// says nothing between the runs laid the score file back over the count.
    pub fn an_attempt_at_a_card_of_th10_counted_and_kept(trace: &Trace) -> Result<(), String> {
        an_attempt_at_a_card_counted_and_kept_in_the_score_object(trace)
    }

    /// **th11's attempt at a card counted and kept**, th10's two runs over this game's score object — which this game
    /// treats as th10 treats its own: read from `scoreth11.dat` once, at the startup (0x437800 at 0x428b28), and
    /// written whole at each run's teardown (0x437a70 at 0x41fe17), so nothing between the two runs reads the file
    /// back over the count `Th11::count_card_attempt` adds.
    pub fn an_attempt_at_a_card_of_th11_counted_and_kept(trace: &Trace) -> Result<(), String> {
        an_attempt_at_a_card_counted_and_kept_in_the_score_object(trace)
    }

    /// The first run's count at a card, the run left, and the second run's count at least two more.
    fn an_attempt_at_a_card_counted_and_kept_in_the_score_object(
        trace: &Trace,
    ) -> Result<(), String> {
        const COUNTED: &str = "retry: attempt ";
        let attempt = |at: usize| -> Option<u32> {
            trace.records[at].lines.iter().find_map(|line| {
                line.split_once(COUNTED)?
                    .1
                    .split_once(" at this spell card")?
                    .0
                    .parse()
                    .ok()
            })
        };
        let first = trace.line_from(0, COUNTED).ok_or(
            "nothing counted an attempt at the card the first run's chapter was taken again inside",
        )?;
        let left = trace.records[first..]
            .iter()
            .position(|record| record.state.is_none_or(|state| !state.in_run))
            .map(|at| at + first)
            .ok_or("the run that counted was never left, so no second run could start")?;
        let second_run = trace
            .in_run_from(left)
            .ok_or("no second run was started in the launch")?;
        let second = trace
            .line_from(second_run, COUNTED)
            .ok_or("nothing counted an attempt at the card in the second run")?;
        let (first, second) = (
            attempt(first).ok_or("the first count said no number")?,
            attempt(second).ok_or("the second count said no number")?,
        );
        expect(second >= first + 2, || {
            format!(
                "the second run's retry counted attempt {second} after the first's {first}, which is not the \
                 card's start and a retry on top of it: the first run's count was not kept"
            )
        })
    }

    /// The stage word th10's Extra is played in, the stage from nought that `[0x474c7c]` counts from 1 as 7.
    const TH10_THE_EXTRAS_STAGE: i32 = 6;

    /// And th11's, the stage from nought that `[0x4a5728]` counts from 1 as 7.
    const TH11_THE_EXTRAS_STAGE: i32 = 6;

    /// **How far the stage's own clock has to get for the run to have been played**, which is what
    /// `a_normal_mode_run_of_th08_played_without_a_bomb` and th10's of that name are held to in place of a
    /// stage reached.
    ///
    /// Two thousand, which is half a minute of stage at sixty frames a second and a tenth of what th08's
    /// recipe's own `play_until` asks for: every launch of it so far has passed ten thousand, and the
    /// slowest reached 3,843 with three deaths behind it. So this is a bound on a launch that went nowhere
    /// at all rather than a bound on the bot — th10's first stage alone reads 2201 on its clock where its
    /// midboss arrives.
    const A_STAGE_IS_UNDER_WAY: i32 = 2_000;

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
        the_name_was_typed(trace, ending)?;
        // **And the main menu's cursor on the Extra after it**, this being the clear that lights it — see
        // [`the_extra_offered_after`]: this game's menu is screen 3.
        let menu = trace
            .line_from(ending, "front end: screen 3,")
            .ok_or("the title after the clear was never walked to the main menu")?;
        the_extra_offered_after(trace, menu)
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
        let last_with_a_boss = the_last_boss_was_beaten(trace, extra)?;
        // **And the name the ranking was registered under**, the Extra reaching the same entry with no
        // ending in front of it.
        the_name_was_typed(trace, last_with_a_boss)
    }

    /// **th06nc's 完全無欠モード run on Normal, played to its ending with no hit and no bomb** — th06c's
    /// case of that name, over this game's walk and its ending.
    pub fn a_normal_run_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let normal = trace.records[started]
            .state
            .ok_or("the run stood over no state")?
            .difficulty;
        expect(normal == 1, || {
            format!("the run was started on difficulty {normal} and not on Normal")
        })?;
        let last = trace
            .stage_from(started, 6)
            .ok_or("the run never reached its sixth stage")?;
        let ending = the_ending_ran(trace, last)?;
        took_no_hit_and_no_bomb(trace, started, ending, TH06NC_BOMB)?;
        the_name_was_typed(trace, ending)
    }

    /// **And th06nc's Extra, played the same way** — th06c's ways through its three cards and after its
    /// midboss, under this game's numbers, all in its recipe; cleared where its boss left all but out of
    /// life, as [`the_extra_of_th06nc_played_to_its_clear`] reads it.
    pub fn the_extra_of_th06nc_played_to_its_clear_with_no_hit_and_no_bomb(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was ever started")?;
        let extra = trace.records[started..]
            .iter()
            .position(|record| {
                record
                    .state
                    .is_some_and(|state| state.in_run && state.difficulty == EXTRA)
            })
            .map(|at| at + started)
            .ok_or("the run started was not on the Extra's difficulty")?;
        let cleared = the_last_boss_was_beaten(trace, extra)?;
        took_no_hit_and_no_bomb(trace, extra, cleared, TH06NC_BOMB)?;
        the_name_was_typed(trace, cleared)
    }

    /// **The last frame from `from` on that a boss stood on, where she left it all but out of life** — see
    /// [`the_extra_of_th06nc_played_to_its_clear`] for why that and not an ending says an Extra was cleared.
    fn the_last_boss_was_beaten(trace: &Trace, from: usize) -> Result<usize, String> {
        let last_with_a_boss = trace.records[from..]
            .iter()
            .rposition(|record| record.state.is_some_and(|state| state.boss_present))
            .map(|at| at + from)
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
        Ok(last_with_a_boss)
    }

    /// **How much life a boss leaves with where she was beaten**: her last attack ends the frame the shot
    /// that empties it lands, so what is read is the life the frame before — `5` on the launch that cleared
    /// th06nc's Extra. A bound and not that number, because how much the last shot took is the shot's.
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

    /// **The run the launch before left is offered, taken up from where it stopped, and lands in the chapter it
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
        let answered = the_beginning_answered(trace)?;
        let offered = trace
            .question_from(answered, "keep_replay")
            .ok_or("the replay of the run being written over was never asked about")?;
        let kept = trace
            .line_from(offered, "replay: kept")
            .ok_or("the question about the replay was not answered with keeping it")?;
        let started = a_fresh_run_after(trace, kept)?;
        trace
            .chapter_from(started, 1, 2)
            .map(|_| ())
            .ok_or_else(|| {
                "the run started from the beginning never reached its second chapter".to_owned()
            })
    }

    /// **th08's and th10's, which keep the replay without asking about it**: neither game's last screen
    /// before a run is one orb holds a press back on — `Th08::menu_takes_a_press` and `Th10::menu_takes_a_press`
    /// answer the main menu alone — so the resume question comes as the run is chosen, and a はじめから
    /// answered there keeps the replay of the run it writes over with no question of its own, which is `it is
    /// kept without asking` in the log. Then the fresh run plays into its second chapter: th08's stage 1's
    /// midstage table beginning one at script 1352, th10's the midboss's first attack.
    pub fn the_run_started_over_and_its_replay_kept_without_asking(
        trace: &Trace,
    ) -> Result<(), String> {
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
        put_back_where_its_chapter_began(trace, back)?;
        trace
            .chapter_from(back, 1, died_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the run never got past chapter {died_in} after retrying it"))
    }

    /// **A death in the attack a boss moves to after a card, answered by that attack again** — th06nc's and
    /// th06c's case of this name. The chapter the run died in began as a nonspell and the one before it as a
    /// card, and the run the chapter again put back is in that same chapter with no card up: a fight whose
    /// nonspells were not chapters of their own read the death as one in the card and put the run back at
    /// the card's start.
    ///
    /// **The death looked for among the run's, and not the first of them**: the recipe hands the rules the
    /// chance while the card is up, and a chance handed over counts for the attempt already under way — see
    /// `orb_bot::hits::Hits::told` — so the card is died in first, its retry played clean, and the nonspell
    /// after it is where the next death lands.
    pub fn a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell(
        trace: &Trace,
    ) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        // The line each chapter of the stage began with, the newest before `until` of each number.
        let began = |chapter: u32, until: usize| {
            let said = format!(" chapter {chapter} at frame");
            trace.records[started..until]
                .iter()
                .rev()
                .flat_map(|record| record.lines.iter())
                .find(|line| line.contains(&said) && line.contains(": a "))
                .cloned()
        };
        let mut from = started;
        let mut deaths = Vec::new();
        let (retry, died_in) = loop {
            let retry = trace.question_from(from, "retry").ok_or_else(|| {
                format!(
                    "no death was in a nonspell after a card; the run died in {}",
                    match deaths.is_empty() {
                        true => "no chapter at all".to_owned(),
                        false => deaths.join(", "),
                    }
                )
            })?;
            let died_in = trace.records[retry].chapter;
            let its_start = began(died_in, retry).unwrap_or_default();
            let before = began(died_in.saturating_sub(1), retry).unwrap_or_default();
            if its_start.contains("nonspell") && before.contains("spellcard") {
                break (retry, died_in);
            }
            deaths.push(format!("chapter {died_in}"));
            // Past the frames the question stands on, to the next death: it stands for several.
            from = trace.records[retry..]
                .iter()
                .position(|record| record.question.as_deref() != Some("retry"))
                .map(|after| retry + after)
                .ok_or("the retry question never went down")?;
        };
        let answered = trace
            .line_from(retry, "retry: the chapter again")
            .ok_or("the retry question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        expect(trace.records[back].chapter == died_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {died_in}, which it died in",
                trace.records[back].chapter
            )
        })?;
        put_back_where_its_chapter_began(trace, back)?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(after.spellcard.is_none(), || {
            format!(
                "the run came back with card {:?} up, so it was put back at the card and not at the nonspell",
                after.spellcard
            )
        })
    }

    /// **A death answered by a chapter further back**: the retry question a death raised is answered with
    /// the chapters behind the one died in, the nearest of them is chosen and confirmed, and the run comes
    /// back on the frame that chapter began on — then plays up to the one it died in again.
    ///
    /// **The question the death raised, and not the pause menu's**: [`a_chapter_further_back_taken_again`]
    /// asks the same list over a run asked to start over, which is nobody's death. The question is found
    /// back from where the list was asked for, so a death the rules answered with the chapter again before
    /// the recipe turned its hits on is not the one read.
    pub fn a_death_answered_by_a_chapter_further_back(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let listed = trace
            .line_from(started, "retry: asking which chapter further back")
            .ok_or("the chapters behind the one the run died in were never listed")?;
        expect(
            trace
                .line_from(started, "retry: the run was asked to start over")
                .is_none_or(|asked| asked > listed),
            || "the list was asked for over a run asked to start over, not over a death".to_owned(),
        )?;
        let up = trace.records[..listed]
            .iter()
            .rposition(|record| record.question.as_deref() != Some("retry"))
            .map_or(0, |at| at + 1);
        let before = trace.records[up]
            .state
            .ok_or("the retry stood over no state")?;
        let died_in = trace.records[up].chapter;
        expect(before.deaths >= 1, || {
            format!(
                "the retry stood with the game counting {} deaths",
                before.deaths
            )
        })?;
        let answered = trace
            .line_from(listed, "retry: a chapter further back")
            .ok_or("no chapter behind the one the run died in was chosen")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the chapter behind was chosen")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        let came_back_in = trace.records[back].chapter;
        expect(came_back_in < died_in, || {
            format!(
                "the run came back in chapter {came_back_in} rather than behind chapter {died_in}, which it \
                 died in"
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        put_back_where_its_chapter_began(trace, back)?;
        trace
            .chapter_from(back, 1, died_in)
            .map(|_| ())
            .ok_or_else(|| {
                format!(
                    "the run never played back up to chapter {died_in} from the chapter behind it"
                )
            })
    }

    /// **th08's death answered by the chapter again**, which is [`a_death_answered_by_the_chapter_again`]
    /// up to its last step, written while no row of this game's midstage table was measured: what says the
    /// run went on is its clock passing the frame the death was on, rather than a chapter after the one
    /// died in.
    ///
    /// **And the clock it is read on is `script_frames` and not `stage_frames`**, which is this game's
    /// own split: the clock the game steps carries the *run's* frames, continuous across every stage —
    /// `stage_frames` is orb's count from the stage's build off it — and `script_frames` is the stage
    /// script's own, which starts again with the stage and waits out a fight. Both go back over a
    /// restore, the run's clock being a field of `.data` a snapshot copies; the script's is the one a
    /// chapter is a number on, so it is the one this reads.
    ///
    /// **And the sound left alone in four ranges**: `Th08::audio_state` names the manager, its DirectSound
    /// wrapper, the stream a played track has and the source that stream reads through, and a launch read
    /// the first three as ranges of the game's own heap.
    pub fn a_death_of_th08_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        a_death_answered_by_the_chapter_again_on_the_scripts_clock(trace, 4)
    }

    /// **th10's death answered by the chapter again**, which is th08's check — see
    /// [`a_death_of_th08_answered_by_the_chapter_again`] — for the same reason: this game's first stage holds no
    /// row of its midstage table, and a chapter after the one died in is a boss's next attack, which is further
    /// than what says the run went on.
    ///
    /// **Its `script_frames` is orb's own clock of the stage's scripts**, the latest time any of them has
    /// reached, which stands while a boss is up — see `orb_core::game::th10`'s `SCRIPT_CLOCK`. It is kept by
    /// the stage's frame, so a frame restored to reads the count it was given then, and the clock goes back
    /// with the chapter. The recipe's hit lands in the stage's first chapter, before the midboss, where it
    /// climbs.
    ///
    /// **And the sound left alone in five ranges**: `Th10::audio_state` names the manager, its DirectSound
    /// wrapper, the stream, the block the stream's buffer is held in and the source the stream reads
    /// through — `snapshot: leaving 5 audio range(s) alone` on every chapter a launch of this walk took,
    /// 2026-10-01.
    pub fn a_death_of_th10_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        a_death_answered_by_the_chapter_again_on_the_scripts_clock(trace, 5)
    }

    /// **th11's death answered by the chapter again**, which is [`a_death_answered_by_the_chapter_again`] — the
    /// stage's own frames going back to where the chapter began, and the chapter after the one died in reached —
    /// with th10's sound held back on top of it.
    ///
    /// **Not th10's check, whose clock is the stage scripts'**: orb's clock of this game's scripts stood at 60 over
    /// both sides of the retry on the launch this walk was played by hand in, 2026-10-07 — `script=60` on the frame
    /// the question stood on, 199 frames into the stage — so it says nothing went back. The chapter after is near
    /// enough for th10's reason not to hold here: the midboss's arrival at frame 2111, well inside what the recipe
    /// plays.
    ///
    /// **And the sound left alone in five ranges**, `Th11::audio_state`'s manager, its DirectSound wrapper, the
    /// stream, the holder of its buffer and the source — `snapshot: leaving 5 audio range(s) alone` on every chapter
    /// that launch took.
    pub fn a_death_of_th11_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        a_death_answered_by_the_chapter_again(trace)?;
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        trace
            .line_from(started, "snapshot: leaving 5 audio range(s) alone")
            .map(|_| ())
            .ok_or_else(|| "the snapshot did not hold this game's sound back".to_owned())
    }

    /// A hit raising the retry question, the chapter again answered, the run back in the chapter it died in
    /// with the stage script's clock behind where the death was and the retry counted, the sound held back
    /// in `audio_ranges` ranges, and the clock past the death afterwards.
    fn a_death_answered_by_the_chapter_again_on_the_scripts_clock(
        trace: &Trace,
        audio_ranges: usize,
    ) -> Result<(), String> {
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
        // **And the sound left alone rather than rewound**, which is what makes a restore of either game
        // safe: the ranges are each game's `audio_state`.
        //
        // Read from the run's own start and not from the retry, because that is where the line is: a
        // chapter says what it holds back as it is *taken*, and the retry puts back the one already
        // taken rather than taking another.
        trace
            .line_from(
                started,
                &format!("snapshot: leaving {audio_ranges} audio range(s) alone"),
            )
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
    /// reads the score file over all 134 records — `0x14006e310(the parsed file, manager + 0x968)` at
    /// `0x14003c5e7` — so the count is lost at the start of the next run and nowhere else, which is why
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
        an_attempt_at_a_card_counted_and_kept(trace, "score: what the run before counted put back")
    }

    /// **th07's attempt at a card counted and kept**, which is th06nc's two runs over this game's own
    /// read: the save th07 makes at every run's build empties every record (0x42e657) and reads the file
    /// straight back into them (0x42e735), and `Th07::put_back_the_cards_a_run_left` is the put-back
    /// inside that save — `score: the records this run left put back over N the file's own differ in`,
    /// said once the read is done.
    ///
    /// **And a card put back with more attempts than the file had**, which is what says the put-back
    /// carried the attempt — read off the line it says per record it moved, `score: card N now A
    /// attempt(s) … where the file had B and …`. Not off the count of records moved: the real game's
    /// put-back moved 141 of them, every one but the card retried reading `now 0 attempt(s)` where the file
    /// had 0, the records differing from the file's in more than their counts.
    pub fn an_attempt_at_a_card_of_th07_counted_and_kept(trace: &Trace) -> Result<(), String> {
        an_attempt_at_a_card_counted_and_kept(
            trace,
            "score: the records this run left put back over ",
        )?;
        let gained = |line: &str| -> Option<(u32, u32)> {
            let (_, now) = line.split_once("score: card ")?.1.split_once(" now ")?;
            let (attempts, rest) = now.split_once(" attempt(s)")?;
            let had = rest
                .split_once("where the file had ")?
                .1
                .split(' ')
                .next()?;
            Some((attempts.parse().ok()?, had.parse().ok()?))
        };
        let kept = trace
            .records
            .iter()
            .flat_map(|record| record.lines.iter())
            .filter_map(|line| gained(line))
            .any(|(now, had)| now > had);
        expect(kept, || {
            "no card was put back with more attempts than the file had, so the attempt was not in it"
                .to_owned()
        })
    }

    /// The two runs both of those are: a chapter taken again inside a card counting an attempt, the run
    /// left, a second run started, and `put_back` said from the frame the first was left on.
    fn an_attempt_at_a_card_counted_and_kept(trace: &Trace, put_back: &str) -> Result<(), String> {
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
        trace.line_from(left, put_back).map(|_| ()).ok_or_else(|| {
            "the run after the one that counted put nothing back over the records its build read \
                 the file into"
                .to_owned()
        })
    }

    /// **The frame of the stage `chapter` of `stage` began on, as the chapter's own line said it** — the
    /// newest before `until`, the stage counted from one — out of `stage S chapter N (stage start) at frame F`
    /// and `stage S chapter N at frame F (script X): …` alike.
    fn the_frame_a_chapter_began_on(
        trace: &Trace,
        stage: i32,
        chapter: u32,
        until: usize,
    ) -> Option<u32> {
        let said = format!("stage {stage} chapter {chapter} ");
        trace.records[..until.min(trace.records.len())]
            .iter()
            .rev()
            .flat_map(|record| record.lines.iter())
            .filter(|line| line.starts_with(&said))
            .find_map(|line| {
                let (_, after) = line.split_once(" at frame ")?;
                after
                    .split(|c: char| !c.is_ascii_digit())
                    .next()?
                    .parse()
                    .ok()
            })
    }

    /// **The run put back on the frame the chapter it came back in began on**, and not merely somewhere
    /// behind where it was: the stage's clock at `back` is the one the chapter's own line wrote down, or the
    /// frame after it.
    ///
    /// **The game's clock and not orb's count of chapters**, because the count is orb's own and agrees with
    /// itself whatever went wrong: a fight whose nonspells were no chapters of their own read a death in a
    /// nonspell as one in the card before it, put the run back at the card, and said it came back in the
    /// chapter it died in — which every check that compared chapter numbers passed.
    ///
    /// **And one frame late allowed**, measured and not guessed: judged again over the sessions each game
    /// was last played in, 2026-09-27, the first record in the run after a restore read the chapter's frame
    /// plus one in every case of th06, th07, th06c and th06nc — 10 against 9 at a stage's start, 7329
    /// against 7328 at th06nc's nonspell, 2010 against 2009 at th06's midboss — and the chapter's frame
    /// itself in th08's death, 790, the record being written after the update the restored frame ran or
    /// before it by the path the retry took. A chapter put back anywhere else is the fault this is for.
    fn put_back_where_its_chapter_began(trace: &Trace, back: usize) -> Result<(), String> {
        let after = trace.records[back]
            .state
            .ok_or("the run came back to a record with no state")?;
        let chapter = trace.records[back].chapter;
        let began = the_frame_a_chapter_began_on(trace, after.stage + 1, chapter, back)
            .ok_or_else(|| format!("no line said where chapter {chapter} began"))?;
        expect((began..=began + 1).contains(&after.stage_frames), || {
            format!(
                "the run came back on frame {} of the stage, where chapter {chapter} began on frame {began}",
                after.stage_frames
            )
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
        put_back_where_its_chapter_began(trace, back)?;
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

    /// **Every step of [`a_chapter_of_the_stage_after_the_first`] up to the retry coming back**: the second
    /// stage reached with the numbers the first ended with, a chapter of it asked for again, and the run back
    /// in that chapter with the clock behind where it was asked. Answers where the run came back and the
    /// chapter it was asked to start over in.
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
        // the stage above is the one the build came up in rather than one the run walked on into. **A stage
        // of -1 is no stage yet and not another one**: th06c's build reads as a run for frames before it
        // names its stage — the laid-out game's frame 449 of this pick-up read `stage=-1` — so only a stage
        // the state names is held to it.
        let elsewhere = trace.between(taken, landed).iter().find(|record| {
            record
                .state
                .is_some_and(|state| state.in_run && state.stage >= 0 && state.stage != 1)
        });
        expect(elsewhere.is_none(), || {
            let record = elsewhere.expect("a record in another stage");
            format!(
                "frame {} of the pick-up is in stage {}, where the file named stage 2",
                record.frame,
                record.state.expect("a record with a state").stage + 1
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
    /// question is answered for the normal mode, no chapter is retried in any frame of it, the run goes
    /// through the stages on the lives the game gave it, and an ending comes — the record of which lands in
    /// the game's own score file, where the front end reads the items it offers from. Its chapters are
    /// counted all the same, for the reason [`a_normal_mode_run_nothing_is_kept_of`] gives.
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
        // The fifth stage and by the stage alone: a game whose table has no row in a stage counts no chapter in it,
        // so a chapter is nothing to wait for.
        let last = trace
            .stage_from(started, 5)
            .ok_or("the run never reached a fifth stage")?;
        let kept = trace.records[started..]
            .iter()
            .find(|record| record.retries != 0);
        expect(kept.is_none(), || {
            let record = kept.expect("a record with a retry of its own");
            format!(
                "frame {} of the run is at chapter {} with {} retr(ies), where a run of this mode has none",
                record.frame, record.chapter, record.retries
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
        let named = trace
            .line_from(ending, "types ORB into the name the game asks for")
            .ok_or("the ranking was not registered under the name the recipe typed")?;
        the_extra_offered_after(trace, named)
    }

    /// **The title's cursor on Extra Start, item 1, after the clear**, which the title will not rest its cursor on
    /// where it is not offering it: the clear's case is the one that says a clear lights the Extra, the Extra's own
    /// starting from records of one (`Start::TheExtraOffered`).
    fn the_extra_offered_after(trace: &Trace, from: usize) -> Result<(), String> {
        trace
            .line_from(from, ", cursor on item 1")
            .map(|_| ())
            .ok_or_else(|| {
                "the title after the clear never had its cursor on Extra Start, so the clear lit no Extra"
                    .to_owned()
            })
    }

    /// **The Extra, which the run cleared in the other mode unlocked**: the run that starts is at the
    /// Extra's own difficulty, its chapters are a stage's like any other's, and one of them is taken again.
    ///
    /// What says the records offer it is that there was a run to start at all: the item is on the title menu
    /// only where the game's own score file holds the record.
    pub fn the_extra_a_cleared_run_unlocked(trace: &Trace) -> Result<(), String> {
        a_chapter_of_the_extra_taken_again(trace, 7)
    }

    /// **th08's Extra, one of its chapters taken again** — [`the_extra_a_cleared_run_unlocked`] over this
    /// game's Extra, which is its ninth stage word.
    pub fn a_chapter_of_th08s_extra_taken_again(trace: &Trace) -> Result<(), String> {
        a_chapter_of_the_extra_taken_again(trace, TH08_THE_EXTRAS_STAGE + 1)
    }

    /// The Extra started, `stage` being the number the chapters count it as, and one of its chapters taken
    /// again: the run back in the chapter it was asked in, its clock behind where the question stood, and the
    /// chapter after it reached.
    fn a_chapter_of_the_extra_taken_again(trace: &Trace, stage: i32) -> Result<(), String> {
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
            .chapter_from(started, stage, 1)
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
            .chapter_from(back, stage, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the Extra never got past chapter {asked_in} after retrying it"))
    }

    /// **A run of the other mode is the game as it comes**: the question is answered for the normal mode,
    /// the run is played, and orb keeps nothing of it — no retry, and the hook that writes a chapter down
    /// saying it wrote nothing.
    ///
    /// **A chapter counted is not one kept**: a run orb plays counts its chapters in this mode too, for the
    /// autoplay engine's ways named by them (`orb_core::chapter::Chapters::counts_only`), and the trace's record
    /// carries the number — so what says nothing was kept is that no chapter was ever retried.
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
            .find(|record| record.retries != 0);
        expect(kept.is_none(), || {
            let record = kept.expect("a record with a retry of its own");
            format!(
                "frame {} of the run is at chapter {} with {} retr(ies), where a run of this mode \
                 has none",
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
    /// **`orb-e2e` plays every one of them and would too**, but a gate that one day leaves a case out —
    /// as th06nc's did while its laid-out game could not reach an ending — sends that recipe to the real
    /// game unparsed. What that costs is the launch: `cargo xtask real` writes the recipe out, orb reads
    /// it, and a step it cannot read is a game that comes up and plays nothing.
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
