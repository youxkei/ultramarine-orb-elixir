//! One 完全無欠モード run, from the question that starts it to the chapter it is picked up at.
//!
//! Every other scenario in the tree takes one mechanism at a time. This one takes the whole of what
//! the mode *is*, in the order somebody playing meets it, because the parts have to agree with each
//! other and nothing that tests them apart can say whether they do: the chapter a death goes back to
//! is the one the boundary detector named, the numbers it puts back are the ones the retry menu was
//! answered over, and the chapter written down for the next run is the one the run was actually in.
//!
//! **Nothing here calls orb to move the run along, and nothing hands it an opinion about the state.**
//! A game plays the game's part — see `fake` — and this presses keys, runs frames, and reads back
//! three things: the game's own memory, the game's own record of the card, and what orb put in the
//! log.
//!
//! In a `tests/` rather than in a `#[cfg(test)]` module, and that is the point. `cfg(test)` is false
//! here, so this reaches the simulated Windows the only way the shipped DLL would reach the real
//! one — through the `sim` feature — and nothing it drives can have a test-only path in it. Which
//! crate's `tests/` is then free, and it is this one's because that is where the simulated Windows
//! lives — see
//! [docs/adr/0005](../../../docs/adr/0005-every-scenario-lives-in-orb-sims-tests.md).
//!
//! One `#[test]` in the file because there is one game in a process: orb's runtime, the record of what
//! a run has pressed and which file its score goes to are one apiece, the way they are in the game.

use crate::fake::th06::{ATTACK_CHANGES, CARD, CARD_STARTS, Fake, lives_row};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen, item};
use orb_core::game::{Menu, RunStart};
use orb_core::menu_ui::{LINE_HEIGHT, NORMAL, SELECTED};
use orb_core::mode::{Mode, aside, title};
use orb_sim::keys;

/// The run this scenario plays: Normal, Reimu A, from stage one.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

#[test]
fn a_pointdevice_run_is_chosen_lost_retried_given_up_and_picked_up_again() {
    in_its_own_process(|| {
        // `verbose`, because half of what this reads back is written at that level: the file a chapter is
        // left in says so in a `detail!`, being one line every few seconds of play.
        let game = Fake::attach("pointdevice", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();

        // ── 1. モードを選ぶ. The question goes over the game's own title menu, on the press that would
        // have chosen `Game Start` — and only once there is an overlay to draw it with, which is orb's own
        // rule about asking anything.
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.frames_until("the title menu ready to act on a press", 60, || {
            game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "the press did not put the mode question up: {:?}",
            log.lines(),
        );
        // The game underneath is frozen on it, and its own menu never saw that press: the cursor is still
        // on the item the question is about.
        assert!(
            game.image().front_end_now().screen == Screen::Title,
            "the title menu acted on the press orb held back",
        );

        // And what the question put on the screen, read off it: one frame, so each label is one quad.
        // The two modes a line apart, 完全無欠モード the one under the cursor — which is a colour and not a
        // position, `menu_ui` drawing the chosen item in `SELECTED` — with the cursor's own mark to its
        // left on the same line, and under them the lines that say what that choice means.
        game.forget();
        game.frame();
        let asked = game.says(title(Menu::Run));
        let pointdevice = game.says("完全無欠モード");
        let legacy = game.says("レガシーモード");
        let cursor = game.says("▶");
        assert_eq!(
            (asked.len(), pointdevice.len(), legacy.len(), cursor.len()),
            (1, 1, 1, 1),
            "the question is not the four lines it draws",
        );
        assert_eq!(
            pointdevice[0].color, SELECTED,
            "the cursor is not on 完全無欠モード",
        );
        assert_eq!(legacy[0].color, NORMAL);
        assert_eq!(legacy[0].y - pointdevice[0].y, LINE_HEIGHT);
        assert_eq!(
            cursor[0].y, pointdevice[0].y,
            "the mark is not on that line"
        );
        assert!(cursor[0].right() <= pointdevice[0].x, "and not beside it");
        assert_eq!(
            game.says(aside(Menu::Run, Mode::Pointdevice)[0]).len(),
            1,
            "the choice under the cursor does not say what it means",
        );

        // Answered on the keyboard, which is the hand this run has: a pad answers these too, and what
        // says so is `mode_on_the_pad.rs`, where the game owns a controller and orb asks the
        // game for it — here it would have to come from a winmm device this host has not got.
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });

        // ── 2. The run. The press orb held back is handed over, so the title menu chooses the item
        // itself, and the run is answered for at the game's own shot type select.
        game.frames_until("the shot type select ready to act on a press", 90, || {
            game.image().front_end_now().screen == Screen::ShotType
                && game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        // Nothing was left of this run, so nothing was asked and the press went on to start it.
        assert!(
            log.said("resume: nothing was left of normal-reimu-a"),
            "the run was not looked for: {:?}",
            log.lines(),
        );
        game.frames_until("the stage built", 8, || game.state().playing);

        // ── 3. The stage settles and takes its first chapter, which is the stage's own start. Two hundred
        // and forty-eight frames in rather than on the stage's first: a snapshot waits for the music to
        // come up, and a laid-out game has no sound for it to find, so what the wait costs is spent in full
        // — `STAGE_SETTLE_FRAMES` and then `MUSIC_WAIT_FRAMES`.
        game.frames_until("the stage's first chapter", 400, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        assert_eq!(game.state().lives, 2, "the lives a run starts with");

        // And with something to go back to, the count of those lives is painted over: a brush stroke with
        // `DISABLE` on it across the row the game counts them in, because nothing in this run can lose
        // one. What the questions drew on the way in is forgotten first — those cover the whole output.
        game.forget();
        game.frame();
        assert!(
            game.drawn()
                .quads
                .iter()
                .any(|quad| quad.covers(&lives_row())),
            "the count of lives was left standing in a run that cannot lose one",
        );
        let word = game.says("DISABLE");
        assert_eq!(word.len(), 1, "the mark does not say what it means");
        assert!(
            lives_row().overlaps(&word[0]),
            "the word is not over the count of lives: {:?} against {:?}",
            word[0],
            lives_row(),
        );
        assert!(log.said("lives: the brush is"));

        // ── 4. A fight, and a card. Each is a chapter of the fight's own rather than one out of the
        // table: a boss's boundaries are found as it is fought, which is the half of the detector no table
        // could hold.
        game.frames_until("the card's chapter", CARD_STARTS + 60, || {
            log.said("chapter 3 at frame 500 (script 500): a midboss spellcard")
        });
        assert!(
            log.said("chapter 2 at frame 400 (script 400): a midboss nonspell"),
            "the fight arriving was not a chapter of its own: {:?}",
            log.lines(),
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            1,
            "the game counted its own attempt where the card started",
        );
        // The chapter is written down as it begins, so that whatever ends the session leaves it.
        assert!(
            log.said("chapter 3 (MIDBOSS SPELL 1) at frame 500, 500 frame(s) of buttons"),
            "the chapter was not written down: {:?}",
            log.lines(),
        );

        // The frame the chapter begins on, read out of the game's memory: this is what a retry has to put
        // back, field for field.
        let at_the_card = game.state();
        assert_eq!(at_the_card.stage_frames, CARD_STARTS);
        assert_eq!(at_the_card.spellcard, Some(CARD as u32));

        // ── 5. 被弾. The game takes a life and counts the death, and orb freezes it on the frame it
        // notices — the retry menu up over a game whose own clock has stopped.
        game.hit();
        game.frame();
        assert!(
            log.said("died in chapter 3"),
            "the death was not noticed: {:?}",
            log.lines(),
        );
        assert_eq!(game.state().lives, 1, "the life the death took");
        assert_ne!(
            game.state(),
            at_the_card,
            "the death has to have moved the memory a retry puts back",
        );
        let frozen = game.state().stage_frames;
        game.frames(10);
        assert_eq!(
            game.state().stage_frames,
            frozen,
            "the game went on updating under the retry menu",
        );

        // The menu itself, read off the screen: the chapter it is offering to put back — by the name the
        // detector gave it — the rewinds this run has cost so far, and the three ways on a line apart with
        // the cursor on the first.
        game.forget();
        game.frame();
        let chapter = game.says("MIDBOSS SPELL 1");
        let retries = game.says("RETRY 0");
        let again = game.says("チャプターをやり直す");
        let stage = game.says("ステージをやり直す");
        let quit = game.says("タイトルに戻る");
        assert_eq!(
            (
                chapter.len(),
                retries.len(),
                again.len(),
                stage.len(),
                quit.len()
            ),
            (1, 1, 1, 1, 1),
            "the menu over the frozen game is not the one the mode puts there",
        );
        assert_eq!(
            again[0].color, SELECTED,
            "the cursor is not on チャプターをやり直す",
        );
        assert_eq!((stage[0].color, quit[0].color), (NORMAL, NORMAL));
        assert_eq!(stage[0].y - again[0].y, LINE_HEIGHT);
        assert_eq!(quit[0].y - stage[0].y, LINE_HEIGHT);
        assert!(
            chapter[0].y < again[0].y,
            "the chapter it names is not above the items",
        );

        // ── 6. チャプターをやり直す, which is the menu's first item and the whole promise of the mode: the
        // memory is put back, so the run is where it was — the lives it had, the seed it was drawn with,
        // the frame it was on.
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the run is not where the chapter began, field for field",
        );
        assert!(
            log.said("retry chapter 3 (retry 1)"),
            "the chapter that came back is not the one that was lost: {:?}",
            log.lines(),
        );
        // The number the 完全無欠 ranking screen shows against that card, out of the game's own record: the
        // game counts an attempt where a card *starts*, and a chapter that begins inside one never starts
        // it — so this attempt is one only orb can have counted.
        assert_eq!(
            game.image().card_attempts(CARD),
            2,
            "the attempt this retry is was not counted against the card",
        );
        assert!(log.said("retry: attempt 2 at this spell card"));

        // ── 7. The run gets somewhere, and the file follows it: the chapter written down is the one the
        // run is in, with the rewinds it has already cost.
        game.frames_until("the chapter after the card", ATTACK_CHANGES + 60, || {
            log.said("chapter 4 at frame 700 (script 700): a midboss nonspell")
        });
        assert!(
            log.said("chapter 4 (MIDBOSS NONSPELL 2) at frame 700, 700 frame(s) of buttons"),
            "the chapter the run reached was not written down: {:?}",
            log.lines(),
        );
        let at_the_nonspell = game.state();

        // ── 8. やめる. Dying again and giving the run up, which is the one way out of one that leaves the
        // chapter behind — the run did not finish, so nothing takes the file away.
        game.hit();
        game.frame();
        assert!(log.said("died in chapter 4"));
        // Up once, which from the first of three items is the last of them: タイトルに戻る. After the
        // frames the menu reads nothing over, since a direction pressed inside those is one nothing moved
        // on — see `READS_KEYS_AFTER`.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the give-up asking", || {
            log.said("retry: asking about the run given up")
        });
        // And the question it asks, whose cursor starts on いいえ — the answer that costs nothing.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the run given up", || {
            log.said("retry: the run is given up")
        });
        // What a run counted about spell cards is written on the way out, which orb reaches by taking the
        // game through the screen that ranking is shown on, with nothing drawn: that is the one place the
        // game writes the records its score file holds.
        game.frames_until("the trip through the ranking", 30, || {
            log.said("score: taken through the ranking")
        });
        // **The number the 完全無欠 ranking shows against that card.** The screen's own read is what fills
        // this record — and orb empties it first, so that a ranking read defines the history rather than
        // adding to what was in memory — and the file is written from the record as it stands when the
        // screen goes down. So what has to be here afterwards is this session's count, which is the game's
        // own attempt at the card plus the one the retry was.
        assert!(
            log.said("score: the captures in memory cleared for the ranking about to be read"),
            "the ranking's read did not clear the record it fills: {:?}",
            log.lines(),
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            2,
            "the ranking is written from a record that lost what the run counted",
        );
        assert!(
            log.said("run ended after 1 retries"),
            "the run's rewinds were not reported: {:?}",
            log.lines(),
        );

        // ── 9. 完全無欠のスコア画面, which is where that count is somebody's to read. Asked for the way
        // anybody asks — the title menu's `Score`, and then orb's question about which of the two rankings
        // — and read off the screen rather than out of the record: the game draws its rows from
        // `CARD_HISTORY`, which is the record orb put back on the way out of the run.
        game.frames_until("the title menu ready to act on a press", 90, || {
            game.image().front_end_now().screen == Screen::Title
                && game.image().front_end_now().acts_on_a_press()
        });
        for _ in 0..item::SCORE {
            game.press(keys::DOWN);
        }
        assert_eq!(
            game.image().front_end_now().cursor,
            item::SCORE,
            "the cursor is not on the item the ranking is behind",
        );
        game.press(keys::Z);
        // The same question, asked about a ranking rather than about a run — which is a different question
        // and says so.
        assert!(
            log.said("menu: Scores is under the cursor, asking which mode"),
            "the press did not put the ranking's question up: {:?}",
            log.lines(),
        );
        game.forget();
        game.frame();
        assert_eq!(
            game.says(title(Menu::Scores)).len(),
            1,
            "the question over a ranking is the one asked about a run",
        );
        assert!(
            game.says(title(Menu::Run)).is_empty(),
            "the question over a ranking is the one asked about a run",
        );
        game.press_until(keys::Z, "完全無欠のスコア画面", || {
            game.image().scene() == Scene::Ranking
        });

        // What it shows: one row per card the game holds a record of, and against this run's card the two
        // attempts — the one the game counted where the card started, and the one the retry was.
        game.forget();
        game.frame();
        let card = game.says(&format!("CARD {CARD}"));
        let attempts = game.says("2");
        assert_eq!(
            (card.len(), attempts.len()),
            (1, 1),
            "the ranking is not one row for the one card there is a record of",
        );
        assert_eq!(
            attempts[0].y, card[0].y,
            "the count is not on that card's own row",
        );
        assert!(
            card[0].right() < attempts[0].x,
            "the count is not across the row from the card",
        );
        // And the row is the record, not a number of the screen's own: nothing says 1, which is what the
        // card would show if the retry had not been counted or if the trip through the ranking had lost it.
        assert!(
            game.says("1").is_empty(),
            "the ranking shows a count nothing in this run reached",
        );

        // Back to the title, and the cursor back where the trip through the ranking left it.
        game.press(keys::X);
        game.frames_until("the title menu again", 30, || {
            game.image().scene() == Scene::FrontEnd
                && game.image().front_end_now().screen == Screen::Title
        });
        for _ in 0..item::SCORE {
            game.press(keys::UP);
        }
        assert_eq!(game.image().front_end_now().cursor, item::GAME_START);

        // ── 10. And the same run started again, which is a run picked up rather than one carried on: ending
        // a run drops the record of everything it pressed, so what the playback below is fed is what
        // `resume::load` reads back off the disk.
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec!["normal-reimu-a".to_owned()],
            "the run is not among the ones left unfinished",
        );
        game.frames_until("the title menu ready to act on a press", 90, || {
            game.image().front_end_now().screen == Screen::Title
                && game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        // Answered the same way as the first time, and asked for by what it does rather than by counting
        // lines in the log: the press orb hands back is what takes the title menu to the shot type select.
        game.press_until(keys::Z, "the mode question answered again", || {
            game.image().front_end_now().screen == Screen::ShotType
        });
        game.frames_until("the shot type select ready to act on a press", 90, || {
            game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        assert!(
            log.said("resume: normal-reimu-a was left; asking where to start"),
            "the question after the character select was not asked: {:?}",
            log.lines(),
        );

        // ── 11. つづきから. The stage is built again and the buttons the run pressed are played back into
        // it, with nothing of that drawn, and it lands on the chapter it was left in.
        game.press_until(keys::Z, "つづきから answered", || {
            log.said("resume: from where it stopped, answered on the keyboard")
        });
        game.frames_until("the run played back into place", 30, || {
            log.said("resume: the landing is")
        });
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run did not land where it was written down: {:?}",
            log.lines()
                .iter()
                .filter(|line| line.contains("resume:"))
                .collect::<Vec<_>>(),
        );
        // Which is the frame the chapter began on, read back out of the game's memory the same way the
        // frame it was written down at was.
        let landed = game.state();
        assert_eq!(landed.stage_frames, ATTACK_CHANGES);
        assert_eq!(landed, at_the_nonspell, "the run landed in another frame");
        // And one attempt at the card for the landing itself, which is an attempt at that chapter the same
        // way a retry is — with the ones the playback started on its way there taken back off, or a run
        // picked up would arrive having counted every card it passed.
        assert_eq!(
            game.image().card_attempts(CARD),
            3,
            "the playback counted its own way through the card",
        );
        assert!(log.said("resume: attempt 3 at this spell card"));
        // And it is the chapter the file named, by name and by frame: what orb holds the landing against is
        // what was written down, not what the run happens to be standing in.
        assert!(
            log.said("written down as chapter 4 (MIDBOSS NONSPELL 2) at frame 700"),
            "the chapter it landed in is not the one written down: {:?}",
            log.lines(),
        );
    });
}
