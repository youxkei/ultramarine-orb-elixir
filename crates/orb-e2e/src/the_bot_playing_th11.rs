//! **The rules playing a laid-out th11**: a bullet coming out over the run, a player's path through a card, the
//! ways that hold the run to a part of the field, an enemy shot down out of a wave, the way the autoplay engine
//! names for a card of the Normal, and a conversation skipped.
//!
//! What `the_bot_playing_th10.rs` asks of that game, asked of this one wherever the question is the rules' and not a
//! card's. **The tests of th10's own cards' ways are not asked here** — `宝永四年の赤蛙`, `蛙は口ゆえ蛇に呑まるる`,
//! `洩矢の鉄の輪`, `厭い川の翡翠`, `お天水の奇跡`, `手長足長さま` and `諏訪大戦` — those cards being th10's: th11's own
//! ways are held by `orb_bot`'s tests of `cards.rs` over the fields they were written on, by the real cases, and
//! here by where each waits with nothing on the field to place it by.
//!
//! The field is th10's: `x` in `-184` to `184` and `y` in `32` to `432` (0x4307c5 through 0x430849), the player's
//! place kept in 128ths of a unit (0x435630), and its whole and focused steps `4.5` and `2.0` — so the places and the
//! distances below are th10's own.

use crate::fake::th11::{DIFFICULTY_NORMAL, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// th11's own four directions, as `Th11::button_bit` has them (0x430293 on).
const UP: u32 = 0x010;
const DOWN: u32 = 0x020;
const LEFT: u32 = 0x040;
const RIGHT: u32 = 0x080;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **The place the player is put**: the middle across, the field being counted from its centre line, at the low
/// line the rules wait on.
const MIDDLE: f32 = 0.0;
const LOW: f32 = 368.0;

/// **A card the paths below are written for**, which the autoplay engine has no way for, and where the boss whose
/// card it is stands.
const A_CARD: i32 = 3;
const THE_BOSS_AT: (f32, f32) = (0.0, 100.0);

/// **The player put at `x`, `y`**, in the 128ths its place is kept in.
fn the_player_is_at(game: &Fake, x: f32, y: f32) {
    game.image()
        .the_player_is_at_128ths((x * 128.0) as i32, (y * 128.0) as i32);
}

/// A launch played by `recipe`, logged verbosely, with a run placed in its first stage.
fn playing(name: &str, recipe: &str) -> Box<Fake> {
    let recipe = recipe.to_owned();
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
        },
    );
    game.is_in_a_run();
    game
}

/// **A bullet coming out over the run, six frames from being tested**, laid at an age of `20` so that the frame the
/// move is judged on reads it three from it — most of th11's bullets being tested at `26`, see
/// `orb_core::game::th11::sight`'s `A_BULLET_IS_TESTED_AT`.
const COMING_OUT_FOR: i32 = 20;
/// And a ring of eight active bullets round it at `24`, still, so that every move the run has is hit within some
/// five frames of going: further off than waiting is from the one coming out, nearer than anywhere clear.
const A_RINGS_SLOTS: std::ops::Range<usize> = 20..28;
const THE_RINGS_REACH: f32 = 24.0;
const THE_ONE_COMING_OUT: usize = 30;

/// **The rules do not wait inside a bullet coming out, its being tested read as a hit**: the run on the low line
/// with a bullet coming out on top of it, a few frames from being tested, and every way out a few frames longer —
/// so the move that is hit latest is a step off it, where a bullet coming out read as a warning for good left
/// waiting there the one move nothing hits.
#[test]
fn the_rules_of_th11_step_off_a_bullet_coming_out_over_the_run() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-step-off-a-bullet-coming-out",
            "- play_until: { frames: 400 }\n",
        );
        // Two frames in before anything is laid: the stage's first frame empties the bullet list.
        game.frames(2);
        let image = game.image();
        the_player_is_at(&game, MIDDLE, LOW);
        for (slot, at) in A_RINGS_SLOTS.zip(0..8) {
            let turn = at as f32 * std::f32::consts::FRAC_PI_4;
            let place = (
                MIDDLE + THE_RINGS_REACH * turn.cos(),
                LOW + THE_RINGS_REACH * turn.sin(),
            );
            image.a_bullet_is_at(slot, place, (0.0, 0.0), (4.0, 4.0));
        }
        image.a_bullet_is_at(THE_ONE_COMING_OUT, (MIDDLE, LOW), (0.0, 0.0), (4.0, 4.0));
        image.the_bullet_is_coming_out(THE_ONE_COMING_OUT);
        image.the_bullet_has_been_out_for(THE_ONE_COMING_OUT, COMING_OUT_FOR);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (UP | DOWN | LEFT | RIGHT) != 0,
            "the rules waited inside a bullet a few frames from being tested, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A player's path through a card, as a recipe writes it**: four frames apart, from the low line's middle along it
/// to the right at the whole step's `4.5` a frame, `18` a place, for 40 frames.
const A_PLAYERS_PATH: &str = "[[0.0, 368.0], [18.0, 368.0], [36.0, 368.0], [54.0, 368.0], [72.0, 368.0], [90.0, 368.0], \
                              [108.0, 368.0], [126.0, 368.0], [144.0, 368.0], [162.0, 368.0]]";

/// A recipe following `path` through [`A_CARD`].
fn following(path: &str) -> String {
    format!(
        "- follow: {{ spell: {A_CARD}, every: 4, path: {path} }}\n- play_until: {{ frames: 400 }}\n"
    )
}

/// **Whether the run is where [`A_PLAYERS_PATH`] has it on the card's frame 30**, between its places on 28 and 32 —
/// the card having come up a frame or two into those 30, within a few steps of it.
fn along_the_path(game: &Fake) {
    let (x, y) = game.image().where_the_player_is();
    assert!(
        (x - 117.0).abs() < 4.0 * 4.5 && (y - LOW).abs() < 4.0,
        "the run was at {x:.1},{y:.1} and not along the path, some 117,368:\n  {}",
        game.log().lines().join("\n  "),
    );
}

/// **The rules go along a player's path through the card it is written for**, frame for frame: put on the path's
/// first place as the card starts, the run is where the path has it 30 frames on, at the whole step the path goes
/// at — the frames counted off the card's own timer (`+0x88c`, 0x40c147), which the fake steps as the game does.
#[test]
fn the_rules_of_th11_go_along_a_players_path_through_the_card_a_recipe_follows_it_in() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-go-along-a-players-path",
            &following(A_PLAYERS_PATH),
        );
        let image = game.image();
        game.frames(2);
        the_player_is_at(&game, MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(A_CARD);
        game.frames(30);
        along_the_path(&game);
        assert!(
            game.log()
                .said("wanted the place on the player's path it follows at"),
            "the rules did not say they went along the path:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A player's path that stands still**, on the low line's middle, for 40 frames.
const A_PLAYERS_PATH_THAT_STANDS: &str = "[[0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], \
                                          [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0]]";

/// **And still dodge what was fired before the card**, which the path was never fired at: a bullet out for 500
/// frames falling on the run where the path has it stand, and the rules step aside of it.
#[test]
fn the_rules_of_th11_on_a_players_path_step_aside_of_fire_from_before_the_card() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-on-a-path-step-aside-of-fire-from-before",
            &following(A_PLAYERS_PATH_THAT_STANDS),
        );
        let image = game.image();
        game.frames(2);
        the_player_is_at(&game, MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(A_CARD);
        game.frames(2);
        image.a_bullet_is_at(0, (0.0, LOW - 20.0), (0.0, 2.3), (4.0, 4.0));
        image.the_bullet_has_been_out_for(0, 500);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (LEFT | RIGHT) != 0,
            "the rules stood on the path under fire from before the card, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **But not what was fired before and is going away from the run**: a bullet out for 500 frames `60` behind it
/// and going on away at `2.3` a frame, and the run is where the path has it 30 frames on at the whole step, as it
/// is with nothing behind it.
#[test]
fn the_rules_of_th11_on_a_players_path_keep_to_it_past_fire_from_before_going_away() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-on-a-path-keep-to-it-past-fire-going-away",
            &following(A_PLAYERS_PATH),
        );
        let image = game.image();
        game.frames(2);
        the_player_is_at(&game, MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(A_CARD);
        image.a_bullet_is_at(0, (-60.0, LOW), (-2.3, 0.0), (4.0, 4.0));
        image.the_bullet_has_been_out_for(0, 500);
        game.frames(30);
        along_the_path(&game);
    });
}

/// **And the card's own way once the path is over**, and not the path's last place: a path of three places, eight
/// frames, and 30 frames on the rules want what the recipe's dodge for the card has them want.
#[test]
fn the_rules_of_th11_go_the_cards_own_way_once_a_players_path_through_it_is_over() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-go-the-cards-way-once-the-path-is-over",
            &format!(
                "- dodge: {{ with: in_the_middle, spell: {A_CARD} }}\n\
                 - follow: {{ spell: {A_CARD}, every: 4, path: [[0.0, 368.0], [18.0, 368.0], [36.0, 368.0]] }}\n\
                 - play_until: {{ frames: 400 }}\n"
            ),
        );
        let image = game.image();
        game.frames(2);
        the_player_is_at(&game, MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(A_CARD);
        game.frames(30);
        let last = game
            .log()
            .lines()
            .into_iter()
            .rev()
            .find(|line| line.contains("autoplay: wanted"))
            .unwrap_or_default();
        assert!(
            !last.contains("the place on the player's path"),
            "the rules still went along the path once it was over: {last}\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And what was fired at the run on its way to the path**, which the path was never fired at either: put `30`
/// right of where the path stands as the card starts, the run is on it some five frames later, and a bullet out
/// for 27 of the card's 30 frames falling on it there is stepped aside of.
#[test]
fn the_rules_of_th11_on_a_players_path_step_aside_of_fire_from_before_the_run_was_on_it() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-on-a-path-step-aside-of-fire-from-before-they-were-on-it",
            &following(A_PLAYERS_PATH_THAT_STANDS),
        );
        let image = game.image();
        game.frames(2);
        the_player_is_at(&game, MIDDLE + 30.0, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(A_CARD);
        game.frames(30);
        let (x, y) = image.where_the_player_is();
        assert!(
            x.abs() < 8.0,
            "the run was at {x:.1},{y:.1} and not on the path at 0,368:\n  {}",
            game.log().lines().join("\n  "),
        );
        image.a_bullet_is_at(0, (x, y - 20.0), (0.0, 2.3), (4.0, 4.0));
        image.the_bullet_has_been_out_for(0, 27);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (LEFT | RIGHT) != 0,
            "the rules stood on the path under fire from before they were on it, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A player's path that turns at the whole step**: down the field from the low line's middle, `18` a place four
/// frames apart, and then down and to the right at `12.7` a place each way.
const A_PLAYERS_PATH_THAT_TURNS: &str = "[[0.0, 300.0], [0.0, 318.0], [0.0, 336.0], [0.0, 354.0], [0.0, 372.0], \
                                         [12.7, 384.7], [25.4, 397.4], [38.1, 410.1], [50.8, 422.8]]";

/// **And keep to it through its turn**, at the whole step the turn goes at and not the focused one: 30 frames
/// in, the path is `25` down the diagonal, which the focused step's `1.41` each way a frame is twice as far short of
/// as a path is gone along within — **with a bullet resting on the floor ahead of the diagonal**, which the whole
/// step carried on along it for the frames looked at reaches and the focused one does not.
#[test]
fn the_rules_of_th11_keep_to_a_players_path_at_the_whole_step_through_a_turn() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-keep-to-a-players-path-through-a-turn",
            &following(A_PLAYERS_PATH_THAT_TURNS),
        );
        let image = game.image();
        game.frames(2);
        the_player_is_at(&game, MIDDLE, 300.0);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(A_CARD);
        image.a_bullet_is_at(0, (100.0, 432.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(30);
        // Where the path is on the card's frame 30, between its places on 28 and 32 — the card having come up a frame
        // or two into those 30, within a couple of steps of it.
        let (x, y) = image.where_the_player_is();
        let (there_x, there_y) = (25.4 + 12.7 / 2.0, 397.4 + 12.7 / 2.0);
        assert!(
            (x - there_x).hypot(y - there_y) < 2.0 * 4.5,
            "the run was at {x:.1},{y:.1} and not along the path's turn, some {there_x:.0},{there_y:.0}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The run on the floor `39` right of `x`, with bullets resting `9` to its left and up and to its left and one
/// falling on it from above** — so the steps right are the only ones clear of all three, and land further than `40`
/// from `x`. And the word the rules read a frame on.
fn held_by(game: &Fake, x: f32) -> u32 {
    let image = game.image();
    game.frames(2);
    let floor = 432.0;
    let at = x + 39.0;
    the_player_is_at(game, at, floor);
    image.a_bullet_is_at(0, (at - 9.0, floor), (0.0, 0.0), (4.0, 4.0));
    image.a_bullet_is_at(1, (at, floor - 32.0), (0.0, 2.3), (4.0, 4.0));
    image.a_bullet_is_at(2, (at - 7.0, floor - 12.0), (0.0, 0.0), (4.0, 4.0));
    game.frames(1);
    image.the_words_the_keys_read().0
}

/// **Waited for on the floor by the middle**, `25` right of it, and no move taken further across than `40` from
/// there.
#[test]
fn the_rules_of_th11_keep_on_the_floor_by_the_middle_where_a_dodge_would_take_them_off_it() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-keep-on-the-floor-by-the-middle",
            "- dodge: { with: on_the_floor_by_the_middle, stage: 1, from: 0, to: 9000 }\n\
             - play_until: { frames: 400 }\n",
        );
        let word = held_by(&game, 25.0);
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the floor's place by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A third of the way up th11's field, and no move taken lower than `32` below it**: the run `20` below that
/// height with bullets resting either side of it and up either side, and one falling on it from `40` above at `3` a
/// frame — so standing is hit some eleven frames on, and the steps down, clear of all of them, land lower than that
/// and are not taken.
#[test]
fn the_rules_of_th11_keep_a_third_of_the_way_up_where_a_dodge_would_take_them_lower() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-keep-a-third-of-the-way-up",
            "- dodge: { with: a_third_of_the_way_up_and_no_lower, stage: 1, from: 0, to: 9000 }\n\
             - play_until: { frames: 400 }\n",
        );
        let image = game.image();
        game.frames(2);
        let y = 432.0 - 400.0 / 3.0 + 20.0;
        the_player_is_at(&game, 0.0, y);
        image.a_bullet_is_at(0, (-9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(2, (-9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(3, (9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(4, (0.0, y - 40.0), (0.0, 3.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & DOWN == 0,
            "the rules stepped down, below a third of the way up by more than 32, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And no move taken further across than `40` from the middle** there.
#[test]
fn the_rules_of_th11_keep_a_third_of_the_way_up_by_the_middle_where_a_dodge_would_take_them_off_it()
{
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-keep-a-third-of-the-way-up-by-the-middle",
            "- dodge: { with: a_third_of_the_way_up_and_no_lower, stage: 1, from: 0, to: 9000 }\n\
             - play_until: { frames: 400 }\n",
        );
        let word = held_by(&game, 0.0);
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the middle by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Low in th11's field's middle, and no move taken further across than `40` from it.**
#[test]
fn the_rules_of_th11_keep_low_in_the_middle_where_a_dodge_would_take_them_off_it() {
    in_its_own_process(|| {
        let game = playing(
            "th11-the-rules-keep-low-in-the-middle",
            "- dodge: { with: low_in_the_middle, stage: 1, from: 0, to: 9000 }\n\
             - play_until: { frames: 400 }\n",
        );
        let word = held_by(&game, 0.0);
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the middle by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy of a wave shot down is no motion of the one behind it**: two enemies laid far apart, the first taken
/// off the list, so that the second moves into its place on it — and the run, with nothing near it, keeps every one
/// of its nine moves open. th11's sight keeps an enemy by its address and not its place on the list (`places_kept`),
/// which is what this holds: the step between the two places read as the second enemy's motion is a fear spread
/// over the field.
#[test]
fn an_enemy_of_th11_shot_down_is_no_motion_of_the_one_behind_it() {
    in_its_own_process(|| {
        let game = playing(
            "th11-an-enemy-shot-down-is-no-motion",
            "- play_until: { frames: 400 }\n",
        );
        let image = game.image();
        the_player_is_at(&game, MIDDLE, LOW);
        // After the stage's first frame, whose start empties the list down to `main`.
        game.frames(2);
        image.a_wave_is_up(2);
        // Laid so that the step between the two places carried on from the second lands on the run: `75,216` and on
        // by `-75,152` is `0,368`.
        image.the_waves_enemy_is_at(0, (150.0, 64.0));
        image.the_waves_enemy_is_at(1, (75.0, 216.0));
        game.frames(4);
        let log = game.log();
        let from = log.written();
        image.the_waves_enemy_is_shot_down(0);
        for _ in 0..2 {
            the_player_is_at(&game, MIDDLE, LOW);
            game.frames(1);
        }
        assert!(
            log.said_since(from, "of the nine clear"),
            "the rules said nothing of their moves after the enemy was shot down:\n  {}",
            log.lines()[from..].join("\n  "),
        );
        assert!(
            !log.said_since(from, " 0 of the nine clear"),
            "the enemy behind the one shot down was feared over the step between their places:\n  {}",
            log.lines()[from..].join("\n  "),
        );
    });
}

/// **A card the autoplay engine has a way for in th11's Normal**, stage 6's 143, `核熱「ニュークリアフュージョン」`,
/// which it dodges between the suns — see `orb_bot::ways::th11`.
const A_CARD_OF_THE_NORMAL: i32 = 143;
/// The difficulty above the Normal, as th11 numbers them.
const HARD: i32 = DIFFICULTY_NORMAL + 1;

/// A run of `difficulty` played to `recipe` with that card up, two frames in, and the log it left.
fn the_rules_dodging_a_card_of_the_normal(
    case: &str,
    recipe: &str,
    difficulty: i32,
) -> Vec<String> {
    let game = playing(case, recipe);
    let image = game.image();
    image.the_difficulty_is(difficulty);
    the_player_is_at(&game, MIDDLE, LOW);
    image.a_fight_is_up(THE_BOSS_AT, 1000);
    image.a_card_is_up(A_CARD_OF_THE_NORMAL);
    game.frames(2);
    game.log().lines()
}

/// **A card of a Normal run is dodged the way the autoplay engine names for it**, with no step of the recipe naming
/// one.
#[test]
fn the_rules_of_th11_dodge_a_card_of_a_normal_run_the_way_the_autoplay_engine_names() {
    in_its_own_process(|| {
        let lines = the_rules_dodging_a_card_of_the_normal(
            "th11-the-rules-dodge-the-way-the-engine-names",
            "- play_until: { frames: 400 }\n",
            DIFFICULTY_NORMAL,
        );
        let said =
            "card 143 is up, and the rules dodge it between_the_suns as the autoplay engine names";
        assert!(
            lines.iter().any(|line| line.contains(said)),
            "not said, {said}:\n  {}",
            lines.join("\n  "),
        );
    });
}

/// **And the way a recipe names for the same card is taken in place of the engine's.**
#[test]
fn the_rules_of_th11_dodge_a_card_the_way_the_recipe_names_over_the_autoplay_engines() {
    in_its_own_process(|| {
        let lines = the_rules_dodging_a_card_of_the_normal(
            "th11-the-rules-dodge-the-way-the-recipe-names-over-the-engine",
            "- dodge: { with: in_the_middle, spell: 143 }\n- play_until: { frames: 400 }\n",
            DIFFICULTY_NORMAL,
        );
        let said = "card 143 is up, and the rules dodge it in_the_middle as the recipe names";
        assert!(
            lines.iter().any(|line| line.contains(said)),
            "not said, {said}:\n  {}",
            lines.join("\n  "),
        );
    });
}

/// **And a run of a difficulty the engine has no way written for takes none**, the same card up in a Hard run.
#[test]
fn the_rules_of_th11_dodge_a_card_of_a_hard_run_by_no_way_of_the_autoplay_engines() {
    in_its_own_process(|| {
        let lines = the_rules_dodging_a_card_of_the_normal(
            "th11-the-rules-of-a-hard-run-dodge-no-way-the-engine-names",
            "- play_until: { frames: 400 }\n",
            HARD,
        );
        assert!(
            !lines
                .iter()
                .any(|line| line.contains("card 143 is up, and the rules dodge it")),
            "a way was taken:\n  {}",
            lines.join("\n  "),
        );
    });
}

/// **Where a way of th11's own cards waits with nothing on the field to place it by**, as the log says the rules
/// wanted it: the run put up in the left of the field, nothing else on it, the way named by a recipe over the stage.
fn the_place_a_way_waits_at(name: &str, with: &str) -> Vec<String> {
    let game = playing(
        name,
        &format!(
            "- dodge: {{ with: {with}, stage: 1, from: 0, to: 9000 }}\n- play_until: {{ frames: 400 }}\n"
        ),
    );
    game.frames(2);
    the_player_is_at(&game, -100.0, 150.0);
    game.frames(2);
    game.log().lines()
}

/// Whether the rules said they wanted the place the card's own dodge puts the run at `x`,`y`.
fn wanted_at(lines: &[String], x: i32, y: i32) {
    let wanted = format!("wanted the place the card's own dodge puts it at {x},{y} ");
    assert!(
        lines.iter().any(|line| line.contains(&wanted)),
        "the rules did not want {x},{y}:\n  {}",
        lines.join("\n  "),
    );
}

/// **The Extra's way where the red fairies come out waits on the floor's middle before a body is up** — under the
/// fairies once one is, see `orb_bot::cards::under_the_red_fairies`.
#[test]
fn the_way_of_th11_where_the_red_fairies_come_out_waits_on_the_floors_middle() {
    in_its_own_process(|| {
        let lines = the_place_a_way_waits_at(
            "th11-the-red-fairies-way-waits",
            "where_the_red_fairies_come_out",
        );
        wanted_at(&lines, 0, 432);
    });
}

/// **`「嫌われ者のフィロソフィ」`'s way waits in the middle of the boss's round with no boss read**: `212` down the
/// field.
#[test]
fn the_way_of_th11_against_the_philosophy_of_the_hated_waits_in_the_middle_of_her_round() {
    in_its_own_process(|| {
        let lines =
            the_place_a_way_waits_at("th11-the-hateds-way-waits", "philosophy_of_the_hated");
        wanted_at(&lines, 0, 244);
    });
}

/// **`「サブタレイニアンローズ」`'s way waits half way down with no ring coming**: `248` down the field.
#[test]
fn the_way_of_th11_against_the_subterranean_rose_waits_half_way_down() {
    in_its_own_process(|| {
        let lines = the_place_a_way_waits_at("th11-the-roses-way-waits", "subterranean_rose");
        wanted_at(&lines, 0, 280);
    });
}

/// **`「恋の埋火」`'s way waits in the middle of the low line with no heart out.**
#[test]
fn the_way_of_th11_against_the_embers_of_love_waits_in_the_middle_of_the_low_line() {
    in_its_own_process(|| {
        let lines = the_place_a_way_waits_at("th11-the-embers-way-waits", "embers_of_love");
        wanted_at(&lines, 0, 368);
    });
}

/// **`「深層心理の遺伝子」`'s way waits in the middle of the low line with no boss read.**
#[test]
fn the_way_of_th11_against_the_genetics_of_the_subconscious_waits_in_the_middle_of_the_low_line() {
    in_its_own_process(|| {
        let lines =
            the_place_a_way_waits_at("th11-the-genes-way-waits", "genetics_of_the_subconscious");
        wanted_at(&lines, 0, 368);
    });
}

/// **`「妖怪ポリグラフ」`'s way waits under the boss with no card's frame read**: `252` down the field.
#[test]
fn the_way_of_th11_against_the_youkai_polygraph_waits_under_the_boss() {
    in_its_own_process(|| {
        let lines = the_place_a_way_waits_at("th11-the-polygraphs-way-waits", "youkai_polygraph");
        wanted_at(&lines, 0, 284);
    });
}

/// **Stage 5's way after the yin-yangs waits `300` down the middle** with nothing read.
#[test]
fn the_way_of_th11_after_the_yin_yangs_waits_down_the_middle() {
    in_its_own_process(|| {
        let lines =
            the_place_a_way_waits_at("th11-after-the-yin-yangs-way-waits", "after_the_yin_yangs");
        wanted_at(&lines, 0, 332);
    });
}

/// **And its way across the yin-yangs starts on the floor `24` off the wall on the run's side** — the run left of
/// the middle, so the left wall's, `-184 + 24` — from where it swings across, see
/// `orb_bot::cards::across_the_yin_yangs`.
#[test]
fn the_way_of_th11_across_the_yin_yangs_starts_off_the_wall_on_the_runs_side() {
    in_its_own_process(|| {
        let lines = the_place_a_way_waits_at(
            "th11-across-the-yin-yangs-way-starts",
            "across_the_yin_yangs",
        );
        wanted_at(&lines, -160, 432);
    });
}

/// **A conversation is skipped**, which is th11's own Ctrl — the bit `0x200` of the word, which runs a conversation
/// on where it lets one be (0x41d3ad, 0x41e977) — as th10's bot skips that game's: a bot that sat through every one
/// of them would spend the afternoon it was written to save.
#[test]
fn the_bot_skips_a_conversation_of_th11() {
    const SKIP: u32 = 0x200;
    in_its_own_process(|| {
        let game = playing(
            "th11-the-bot-skips-a-conversation",
            "- play_until: { frames: 400 }\n",
        );
        let image = game.image();
        the_player_is_at(&game, MIDDLE, LOW);
        game.frames(2);
        let (word, _) = image.the_words_the_keys_read();
        assert_eq!(word & SKIP, 0, "the bot skipped with no conversation up");

        image.a_conversation_is_up();
        game.frames(2);
        let (word, _) = image.the_words_the_keys_read();
        assert_eq!(
            word & SKIP,
            SKIP,
            "the bot did not skip the conversation:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
