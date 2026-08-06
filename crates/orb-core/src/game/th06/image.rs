//! A th06 process image laid out in an address space, so that the real
//! [`Th06`] has something to read in a test with no game running.
//!
//! The space itself is `orb_sim::Space`, which this crate cannot link to: it is a dev-dependency and
//! the seam runs the other way round — the simulator implements `orb_api::Win` and knows nothing about
//! this.
//!
//! The regions are the game's own, at the game's own addresses, and they are laid out from the
//! same constants the reads use — so what this cannot catch is a constant that is wrong. A wrong
//! offset makes the writer and the reader wrong together, and only the real game says otherwise;
//! that is what the measurements in `DONE.md` are for. What it does catch is everything built on
//! top of the offsets, which is where the code is.
//!
//! Zeroed, not filled with plausible values. A field the game has not written is zero in a
//! freshly committed page too, and a `Game` method that has to survive reading one is a method
//! whose pointer chases go through `read_committed` — which is the property worth holding it to.

use std::ops::Range;
use std::sync::Arc;

use orb_api::{Hwnd, Kind};
use orb_sim::{Sim, Space};

use crate::d3d8::Device;
use crate::game::RunStart;

/// The game's static data, as the real one is: **one range**, `0x00476000..0x006e79fc`, which is what
/// orb reads out of the PE and writes in the log every run — `.data 0x00476000..0x006e79fc (2562556
/// bytes)`. This is the range a chapter is a copy of.
///
/// A set of windows around each global was tried and is why this is one range. Every structure the
/// reads reach — the enemy manager at 0x004b79c8, the cards at 0x005a5ff8, the lasers, the game
/// manager at 0x0069bca0, the supervisor, the player, the managers — is *inside* this on the machine,
/// so a snapshot there covers all of them. Windows covering each one separately left the rest outside
/// what `data()` reported, and a chapter restored in a scenario put back less than the same chapter
/// restores on the machine: measured as the script's clock coming back on hardware and not in a test,
/// which is a simulator disagreeing with the thing it stands for.
const DATA: Range<usize> = 0x0047_6000..0x006e_79fc;

/// Stands for the game's own code, so that a pointer into it reads as a live COM object's vtable
/// and one anywhere else reads as the stale pointer left in a block the allocator did not scrub.
const CODE: Range<usize> = 0x0040_1000..0x0040_2000;

/// Two of the blocks the game takes from its allocator: the boss of the fight on now, and the screen
/// a ranking is shown on. Both are reached only through a pointer the game keeps in [`DATA`], which
/// is what makes "no boss" the enemy manager holding none rather than the memory going away.
///
/// Outside `DATA` because that is where they are in the real game, and a snapshot of a laid-out game
/// should have the same work to do as one of a running game: `game_regions` finds these for itself
/// and copies them beside the static data, rather than getting them for free as fields of a global.
const BOSS: Range<usize> = 0x0300_0000..0x0300_1000;
const RANKING_SCREEN: Range<usize> = 0x0300_1000..0x0300_2000;

/// A stage in progress, as a scenario says it.
///
/// Every field is one `read_state` parses back out of the game's memory, which is what makes a test
/// that writes these and reads a `State` a test of the parse rather than of itself.
#[derive(Clone, Copy)]
pub struct Playing {
    /// Counted from zero, as everything above `Game` counts them.
    pub stage: i32,
    pub difficulty: i32,
    pub frames: u32,
    pub script_frames: i32,
    pub seed: u16,
    pub deaths: i32,
    pub lives: i8,
    pub bombs: i8,
    pub power: u16,
    pub enemies: i32,
}

/// The numbers a stage is played with that no [`State`](crate::game::State) holds: what the run has
/// scored, where its generator has got to, and where the player is standing.
///
/// Apart from [`Playing`] because these are not what a frame is judged by — they are the fields of
/// [`Reproduction`](crate::game::Reproduction), which is the line a run played back into a chapter is
/// held against. A resume that arrives with any of them different is a resume that has come out of
/// step, and these are what let a scenario say so.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Reproducing {
    pub score: u32,
    /// The generator itself, which is not [`Playing::seed`]: that one is the `GameManager`'s copy of
    /// what the stage was seeded with, and this is what the next number out of it will be made from.
    pub seed: u16,
    /// How many numbers have come out of it since the stage seeded it.
    pub randoms: u32,
    pub player: (f32, f32),
}

/// One of the game's own screens, as its front end's `gameState` names them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    /// The title menu, whose cursor is one of [`item`]'s.
    Title,
    /// The shot type select, where the cursor is the shot itself.
    ShotType,
    /// The ranking, which is what orb asks the front end for on the way out of a run so that what
    /// the run counted is written — see [`Game::show_ranking`](crate::game::Game::show_ranking).
    Ranking,
    /// Any other of its screens, as the number it is: the difficulty and the character select are
    /// two, and orb has nothing to ask over either.
    Other(i32),
}

/// Which item of the title menu a cursor is on, for the two orb has a question about.
pub mod item {
    pub const GAME_START: i32 = super::super::TITLE_ITEM_START;
    pub const SCORE: i32 = super::super::TITLE_ITEM_SCORE;
}

/// Where the game's own front end is: which screen, what its cursor is on, and how many frames it
/// has been there.
///
/// The last of those is `stateTimer`, and it is here because it is what decides whether the screen
/// would act on a press at all — see [`acts_on_a_press`](FrontEnd::acts_on_a_press).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrontEnd {
    pub screen: Screen,
    pub cursor: i32,
    pub frames: i32,
}

impl FrontEnd {
    /// Whether this screen is past the frames it ignores its own decide for.
    ///
    /// Here rather than in whatever is driving the game, because the numbers are the game's own —
    /// see `MENU_TITLE_GRACE_FRAMES` — and orb holds a press back over exactly these frames.
    pub fn acts_on_a_press(&self) -> bool {
        self.frames
            >= match self.screen {
                Screen::Title => super::MENU_TITLE_GRACE_FRAMES,
                Screen::ShotType => super::MENU_SHOT_TYPE_GRACE_FRAMES,
                // Neither has a decide of its own that orb has anything to say about.
                _ => return false,
            }
    }
}

/// Which of its own scenes the game is running.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    FrontEnd,
    /// A stage being played.
    Playing,
    /// The screen a run ends into, which is what tells a run that finished from one that was left —
    /// see [`Game::run_finished`](crate::game::Game::run_finished).
    Result,
    /// The ranking, which is a scene of the supervisor's as well as a state of the front end's:
    /// asking for it is one and building it is the other.
    Ranking,
    Other(i32),
}

/// Its supervisor's two states: what is running, and what has been asked for.
///
/// Both, because every one-frame window orb watches for is the two disagreeing —
/// [`Game::run_chosen`](crate::game::Game::run_chosen) is that frame and nothing else is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Supervising {
    pub running: Scene,
    pub wanted: Scene,
}

/// The boss a fight is against: what is left of it, and how long the attack it is on has been
/// running.
///
/// That second count going back to nothing is what says the fight has moved to its next attack,
/// spell card or not, which is where a chapter begins.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Boss {
    pub life: i32,
    pub attack_frames: i32,
}

/// What the player is doing, which is what says whether a frame is one a chapter may begin on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Normal,
    /// Coming back after a death.
    Spawning,
    /// Hit, and the frames of dying that follow.
    Dying,
    /// The seconds after a bomb or a respawn, where nothing can hit the player.
    Invulnerable,
}

impl Default for Playing {
    /// Stage one on Normal, on its first frame, with the lives and bombs a run starts with.
    fn default() -> Self {
        Self {
            stage: 0,
            difficulty: 1,
            frames: 0,
            script_frames: 0,
            seed: 0,
            deaths: 0,
            lives: 2,
            bombs: 3,
            power: 0,
            enemies: 0,
        }
    }
}

/// The game's memory, laid out. Reading it goes through [`enter`](Self::enter).
pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    pub fn laid_out() -> Self {
        let sim = Arc::new(Sim::new());
        sim.space().map(DATA.start, DATA.len(), Kind::Private);
        sim.space().map(CODE.start, CODE.len(), Kind::Image);
        sim.space().map(BOSS.start, BOSS.len(), Kind::Private);
        sim.space()
            .map(RANKING_SCREEN.start, RANKING_SCREEN.len(), Kind::Private);
        Self { sim }
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which
    /// is what makes `Th06`'s reads land in it.
    ///
    /// Scoped rather than held for the image's whole life, so that a test can lay out two games and
    /// each read the one it is asking about: the one entered is the one in front.
    pub fn enter(&self) -> orb_api::Installed {
        self.sim.enter()
    }

    pub fn space(&self) -> &Space {
        self.sim.space()
    }

    /// Lays a stage in progress over the game's globals.
    ///
    /// In the game's own terms rather than as addresses, so that a scenario says what the game is
    /// doing and this file stays the only place that knows where any of it is kept. The offsets are
    /// the same constants [`Th06`](super::Th06) reads through, reached as a child module of it —
    /// which is what keeps the writer and the reader from drifting apart.
    pub fn playing(&self, run: Playing) {
        use super::{game_manager, supervisor};
        let space = self.space();
        space.write::<i32>(
            super::G_SUPERVISOR + supervisor::CUR_STATE,
            super::STATE_GAMEMANAGER,
        );
        space.write::<i32>(
            super::G_SUPERVISOR + supervisor::WANTED_STATE,
            super::STATE_GAMEMANAGER,
        );
        // The game counts stages from one while one is running; `read_state` takes the one back off.
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::CURRENT_STAGE,
            run.stage + 1,
        );
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::DIFFICULTY,
            run.difficulty,
        );
        space.write::<u32>(
            super::G_GAME_MANAGER + game_manager::GAME_FRAMES,
            run.frames,
        );
        space.write::<u16>(super::G_GAME_MANAGER + game_manager::RANDOM_SEED, run.seed);
        space.write::<i32>(super::G_GAME_MANAGER + game_manager::DEATHS, run.deaths);
        space.write::<i8>(
            super::G_GAME_MANAGER + game_manager::LIVES_REMAINING,
            run.lives,
        );
        space.write::<i8>(
            super::G_GAME_MANAGER + game_manager::BOMBS_REMAINING,
            run.bombs,
        );
        space.write::<u16>(
            super::G_GAME_MANAGER + game_manager::CURRENT_POWER,
            run.power,
        );
        space.write::<i32>(
            super::G_ENEMY_MANAGER + super::enemy_manager::TIMELINE_TIME_CURRENT,
            run.script_frames,
        );
        space.write::<i32>(
            super::G_ENEMY_MANAGER + super::enemy_manager::ENEMY_COUNT,
            run.enemies,
        );
    }

    /// The numbers of [`Reproducing`], which a stage moves as it is played.
    pub fn reproducing(&self, run: Reproducing) {
        use super::game_manager;
        let space = self.space();
        // Both, because the panel's number chases the run's and a score put in one alone would show
        // counting up to it.
        space.write::<u32>(super::G_GAME_MANAGER + game_manager::GUI_SCORE, run.score);
        space.write::<u32>(super::G_GAME_MANAGER + game_manager::SCORE, run.score);
        space.write::<u16>(super::G_RNG, run.seed);
        space.write::<u32>(super::G_RNG + super::RNG_GENERATION_COUNT, run.randoms);
        space.write::<f32>(
            super::G_PLAYER + super::player::POSITION_CENTER,
            run.player.0,
        );
        space.write::<f32>(
            super::G_PLAYER + super::player::POSITION_CENTER + size_of::<f32>(),
            run.player.1,
        );
    }

    /// What they are now, which is what the next frame moves on from.
    pub fn reproducing_now(&self) -> Reproducing {
        use super::game_manager;
        let space = self.space();
        Reproducing {
            score: space.read(super::G_GAME_MANAGER + game_manager::GUI_SCORE),
            seed: space.read(super::G_RNG),
            randoms: space.read(super::G_RNG + super::RNG_GENERATION_COUNT),
            player: (
                space.read(super::G_PLAYER + super::player::POSITION_CENTER),
                space.read(super::G_PLAYER + super::player::POSITION_CENTER + size_of::<f32>()),
            ),
        }
    }

    /// The box the player is held inside, which a stage's build puts in place and nothing else moves.
    pub fn play_field(&self, top: f32, height: f32) {
        use super::game_manager;
        let space = self.space();
        let y = size_of::<f32>();
        space.write::<f32>(
            super::G_GAME_MANAGER + game_manager::PLAYER_AREA_TOP_LEFT + y,
            top,
        );
        space.write::<f32>(
            super::G_GAME_MANAGER + game_manager::PLAYER_AREA_SIZE + y,
            height,
        );
    }

    /// The numbers of [`Playing`] as they are now.
    ///
    /// What makes a game driving itself out of this image a game with one state rather than two: the
    /// frame it plays next is worked out from the memory it is in, so a chapter put back underneath it
    /// takes the run back with it.
    pub fn playing_now(&self) -> Playing {
        use super::{enemy_manager, game_manager};
        let space = self.space();
        Playing {
            stage: space.read::<i32>(super::G_GAME_MANAGER + game_manager::CURRENT_STAGE) - 1,
            difficulty: space.read(super::G_GAME_MANAGER + game_manager::DIFFICULTY),
            frames: space.read(super::G_GAME_MANAGER + game_manager::GAME_FRAMES),
            script_frames: space
                .read(super::G_ENEMY_MANAGER + enemy_manager::TIMELINE_TIME_CURRENT),
            seed: space.read(super::G_GAME_MANAGER + game_manager::RANDOM_SEED),
            deaths: space.read(super::G_GAME_MANAGER + game_manager::DEATHS),
            lives: space.read(super::G_GAME_MANAGER + game_manager::LIVES_REMAINING),
            bombs: space.read(super::G_GAME_MANAGER + game_manager::BOMBS_REMAINING),
            power: space.read(super::G_GAME_MANAGER + game_manager::CURRENT_POWER),
            enemies: space.read(super::G_ENEMY_MANAGER + enemy_manager::ENEMY_COUNT),
        }
    }

    /// Which run the front end has been answered for: the difficulty, the character and the shot, and
    /// the stage as its own screens count it — one less than a stage in progress, since building one
    /// is what raises the number.
    pub fn chose(&self, run: &RunStart) {
        use super::game_manager;
        let space = self.space();
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::DIFFICULTY,
            run.difficulty,
        );
        space.write::<u8>(
            super::G_GAME_MANAGER + game_manager::CHARACTER,
            run.character as u8,
        );
        space.write::<u8>(
            super::G_GAME_MANAGER + game_manager::SHOT_TYPE,
            run.shot_type as u8,
        );
        space.write::<u8>(
            super::G_GAME_MANAGER + game_manager::IS_IN_PRACTICE_MODE,
            u8::from(run.practice),
        );
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::CURRENT_STAGE,
            run.stage,
        );
    }

    /// Raises the stage number the way the callback that puts a stage's numbers in place does, and
    /// answers the stage that came up.
    ///
    /// Its own step because orb reads a run's stage back through the same `-1` — see
    /// `game_manager::CURRENT_STAGE` — so a game that built a stage without doing this would be a
    /// stage off in everything above it.
    pub fn stage_built(&self) -> i32 {
        use super::game_manager;
        let at = super::G_GAME_MANAGER + game_manager::CURRENT_STAGE;
        let stage: i32 = self.space().read(at);
        self.space().write::<i32>(at, stage + 1);
        stage
    }

    /// Puts the front end's own three fields where `front` says.
    ///
    /// Not the supervisor with them: whether the front end is what is *running* is the supervisor's to
    /// say — see [`supervising`](Self::supervising) — and the frames field is a timer any screen of the
    /// game can be counting, since a game laid out by hand has one screen at a time.
    pub fn front_end(&self, front: FrontEnd) {
        use super::main_menu;
        let space = self.space();
        space.write::<i32>(
            super::G_MAIN_MENU + main_menu::GAME_STATE,
            screen_of(front.screen),
        );
        space.write::<i32>(super::G_MAIN_MENU + main_menu::CURSOR, front.cursor);
        space.write::<i32>(super::G_MAIN_MENU + main_menu::STATE_TIMER, front.frames);
    }

    /// What the front end's own three fields hold, whatever the supervisor says is running.
    ///
    /// Not `None` off the front end, because the game's is a global that keeps whatever it was left
    /// holding — which is the very thing `menu_pointed_at` guards against and so has to be readable
    /// here. [`scene`](Self::scene) is what says whether the front end is what is running.
    pub fn front_end_now(&self) -> FrontEnd {
        use super::main_menu;
        let space = self.space();
        FrontEnd {
            screen: screen_from(space.read(super::G_MAIN_MENU + main_menu::GAME_STATE)),
            cursor: space.read(super::G_MAIN_MENU + main_menu::CURSOR),
            frames: space.read(super::G_MAIN_MENU + main_menu::STATE_TIMER),
        }
    }

    pub fn supervising(&self, supervisor: Supervising) {
        let space = self.space();
        space.write::<i32>(
            super::G_SUPERVISOR + super::supervisor::CUR_STATE,
            scene_of(supervisor.running),
        );
        space.write::<i32>(
            super::G_SUPERVISOR + super::supervisor::WANTED_STATE,
            scene_of(supervisor.wanted),
        );
    }

    pub fn supervising_now(&self) -> Supervising {
        let space = self.space();
        Supervising {
            running: scene_from(space.read(super::G_SUPERVISOR + super::supervisor::CUR_STATE)),
            wanted: scene_from(space.read(super::G_SUPERVISOR + super::supervisor::WANTED_STATE)),
        }
    }

    /// What is running, which is the half of [`supervising_now`](Self::supervising_now) that most
    /// questions are about.
    pub fn scene(&self) -> Scene {
        self.supervising_now().running
    }

    /// The word the game's own read handed to this frame's update, which is what every one of its
    /// `WAS_PRESSED` tests is worked out against — see
    /// [`Game::swallow_input`](crate::game::Game::swallow_input).
    pub fn input(&self, word: u16) {
        self.space().write::<u16>(super::G_CUR_FRAME_INPUT, word);
    }

    /// And what the frame before was handed, which is what it still holds until the next update
    /// assigns over it.
    pub fn input_now(&self) -> u16 {
        self.space().read(super::G_CUR_FRAME_INPUT)
    }

    /// The boss of the fight on now, or none.
    ///
    /// Both halves of what orb reads: the pointer the enemy manager keeps, which is where the life
    /// and the attack's clock are read through, and the panel's own flag, which is what
    /// `boss_present` is.
    pub fn boss(&self, boss: Option<Boss>) {
        use super::{enemy, enemy_manager, gui};
        let space = self.space();
        space.write::<u8>(super::G_GUI + gui::BOSS_PRESENT, u8::from(boss.is_some()));
        space.write::<usize>(
            super::G_ENEMY_MANAGER + enemy_manager::BOSSES,
            boss.map_or(0, |_| BOSS.start),
        );
        if let Some(boss) = boss {
            space.write::<i32>(BOSS.start + enemy::LIFE, boss.life);
            space.write::<i32>(BOSS.start + enemy::BOSS_TIMER_CURRENT, boss.attack_frames);
        }
    }

    /// The boss of that fight as the game's memory holds it now, or none.
    ///
    /// Read back rather than kept beside the memory, so that a game moving an attack's clock on moves
    /// the clock a chapter restored underneath it takes back.
    pub fn boss_now(&self) -> Option<Boss> {
        use super::{enemy, enemy_manager};
        let space = self.space();
        let at = space.read::<usize>(super::G_ENEMY_MANAGER + enemy_manager::BOSSES);
        (at != 0).then(|| Boss {
            life: space.read(at + enemy::LIFE),
            attack_frames: space.read(at + enemy::BOSS_TIMER_CURRENT),
        })
    }

    /// The spell card that boss is on now, or none.
    ///
    /// Which card it is goes in both of the places the game keeps it: the enemy manager's, which is
    /// what a `State` reads, and the one the count of attempts is indexed by — see
    /// [`card_record`](Self::card_record).
    pub fn card(&self, card: Option<i32>) {
        use super::enemy_manager;
        let space = self.space();
        space.write::<i32>(
            super::G_ENEMY_MANAGER + enemy_manager::SPELLCARD_IS_ACTIVE,
            i32::from(card.is_some()),
        );
        if let Some(card) = card {
            space.write::<u32>(
                super::G_ENEMY_MANAGER + enemy_manager::SPELLCARD_IDX,
                card as u32,
            );
            space.write::<i32>(super::CURRENT_CARD, card);
        }
    }

    /// Lays out the game's own record of a spell card.
    ///
    /// A record the game has written, which means the `CATK` magic at its head: `count_card_attempt`
    /// refuses one without it, because a zeroed record is a card this build does not have rather than
    /// a card nobody has tried. So a scenario that wants the attempt counted has to say the game had
    /// got as far as writing the record, which is what this says.
    pub fn card_record(&self, card: i32, attempts: u16) {
        let space = self.space();
        let record = super::CARD_HISTORY + card as usize * 0x40;
        space.write::<u32>(record, super::CATK_MAGIC);
        space.write::<u16>(record + super::CATK_ATTEMPTS, attempts);
    }

    /// What the player is doing.
    pub fn player(&self, player: Player) {
        self.space().write::<i8>(
            super::G_PLAYER + super::player::PLAYER_STATE,
            match player {
                Player::Normal => super::PLAYER_NORMAL,
                Player::Spawning => super::PLAYER_SPAWNING,
                Player::Dying => super::PLAYER_DEAD,
                Player::Invulnerable => super::PLAYER_INVULNERABLE,
            },
        );
    }

    /// The device orb draws its overlay through and the window it reads the keyboard against, which
    /// are what the game has once it has finished setting Direct3D up.
    pub fn shows_through(&self, device: *mut Device, window: Hwnd) {
        let space = self.space();
        space.write::<*mut Device>(super::G_SUPERVISOR + super::supervisor::D3D_DEVICE, device);
        space.write::<Hwnd>(
            super::G_SUPERVISOR + super::supervisor::HWND_GAME_WINDOW,
            window,
        );
    }

    /// The three objects the game's own chain hands to a callback, which is what orb's hooks over
    /// those callbacks are given: the front end, the game manager, and the screen a ranking is shown
    /// on.
    ///
    /// Here rather than as addresses in whatever is driving the game, for the same reason everything
    /// else in this file is: the offsets are th06's, and this is the one place that knows them.
    pub fn front_end_object(&self) -> usize {
        super::G_MAIN_MENU
    }

    pub fn game_manager_object(&self) -> usize {
        super::G_GAME_MANAGER
    }

    pub fn ranking_screen(&self) -> usize {
        RANKING_SCREEN.start
    }

    /// Says that screen is up with its records read in, which is the state orb waits for before it
    /// puts back what this session counted.
    pub fn ranking_screen_shown(&self) {
        self.space()
            .write::<i32>(RANKING_SCREEN.start + super::RANKING_STATE, RANKING_SHOWN);
    }

    /// Whether it has been told to leave, which is what asks the game to put the scene back.
    pub fn ranking_screen_leaving(&self) -> bool {
        self.space()
            .read::<i32>(RANKING_SCREEN.start + super::RANKING_STATE)
            == super::RESULT_SCREEN_STATE_EXITING
    }

    /// Every card the game holds a record of, and the attempts against each — which is what its
    /// ranking screen has to show and what its score file is written from.
    ///
    /// A record with no `CATK` at its head is not a card this build has, so it is not a row: that is
    /// the same test `count_card_attempt` refuses on.
    pub fn card_records(&self) -> Vec<(i32, u16)> {
        let cards = super::CARD_HISTORY_BYTES / 0x40;
        (0..cards as i32)
            .filter(|card| {
                self.space()
                    .read::<u32>(super::CARD_HISTORY + *card as usize * 0x40)
                    == super::CATK_MAGIC
            })
            .map(|card| (card, self.card_attempts(card)))
            .collect()
    }

    /// How many attempts the game's record holds for `card`, which is the number the 完全無欠 ranking
    /// screen shows against it.
    pub fn card_attempts(&self, card: i32) -> u16 {
        self.space()
            .read::<u16>(super::CARD_HISTORY + card as usize * 0x40 + super::CATK_ATTEMPTS)
    }

    /// Says the run on screen is a replay being watched rather than one somebody is playing.
    ///
    /// Its own method rather than a field of [`Playing`], because it is not a number a stage is
    /// played with: it is what decides whether orb acts on the run at all.
    pub fn watching_a_replay(&self) {
        self.space()
            .write::<u32>(super::G_GAME_MANAGER + super::game_manager::IS_IN_REPLAY, 1);
    }

    /// What a snapshot covers, which orb reads out of the PE in a real game.
    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// The host this game is laid out in, for a scenario that needs more of it than the memory — the
    /// keyboard somebody presses at orb's own menus, or which window is in front.
    ///
    /// The same one, not another: a scenario pressing keys against one host while the chapters read
    /// the game through a second would be two processes agreeing about nothing.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }
}

/// The state the ranking screen is in once it is up with its records read in. The first of the states
/// orb reads as "showing", so a game laid out here and orb cannot disagree about which those are.
const RANKING_SHOWN: i32 = super::RESULT_SCREEN_SHOWING[0];

fn scene_of(scene: Scene) -> i32 {
    match scene {
        Scene::FrontEnd => super::STATE_MAINMENU,
        Scene::Playing => super::STATE_GAMEMANAGER,
        Scene::Result => super::STATE_RESULTSCREEN_FROMGAME,
        Scene::Ranking => super::STATE_SCORE,
        Scene::Other(state) => state,
    }
}

fn scene_from(state: i32) -> Scene {
    match state {
        super::STATE_MAINMENU => Scene::FrontEnd,
        super::STATE_GAMEMANAGER => Scene::Playing,
        super::STATE_RESULTSCREEN_FROMGAME => Scene::Result,
        super::STATE_SCORE => Scene::Ranking,
        state => Scene::Other(state),
    }
}

fn screen_of(screen: Screen) -> i32 {
    match screen {
        Screen::Title => super::MENU_STATE_TITLE,
        Screen::ShotType => super::MENU_STATE_SHOT_TYPE,
        Screen::Ranking => super::MENU_STATE_SCORE,
        Screen::Other(state) => state,
    }
}

fn screen_from(state: i32) -> Screen {
    match state {
        super::MENU_STATE_TITLE => Screen::Title,
        super::MENU_STATE_SHOT_TYPE => Screen::ShotType,
        super::MENU_STATE_SCORE => Screen::Ranking,
        state => Screen::Other(state),
    }
}
