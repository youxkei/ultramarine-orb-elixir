//! One 完全無欠モード run, from the question that starts it to the file it is left in.
//!
//! Every other scenario in the tree takes one mechanism at a time. This one takes the whole of what
//! the mode *is*, in the order somebody playing meets it, because the parts have to agree with each
//! other and nothing that tests them apart can say whether they do: the chapter a death goes back to
//! is the one the boundary detector named, the numbers it puts back are the ones the retry menu was
//! answered over, and the chapter written down for the next session is the one the run was actually
//! in.
//!
//! In `tests/` rather than in a `#[cfg(test)]` module, and that is the point. `cfg(test)` is false
//! here, so this reaches the simulated Windows the only way the shipped DLL would reach the real
//! one — through the `sim` feature — and nothing it drives can have a test-only path in it.
//!
//! What it cannot reach is `lib.rs`'s frame hook, which is where the death is noticed and the menu
//! put up. Those two lines are named below where the scenario stands in for them, and they are the
//! seam of this test rather than of the code.

use orb::chapter::Chapters;
use orb::retry_ui::{Choice, RetryMenu};
use orb::{resume, score};
use orb_api::Hwnd;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{Image, Playing};
use orb_core::game::{Game, Menu, Pad, RunStart, State};
use orb_core::input::Keyboard;
use orb_core::menu::By;
use orb_core::mode::{Answer, INPUT_GRACE_FRAMES, Mode, Question};
use orb_sim::keys;

/// The window has to be in front for a key to read as down at all — see `input::Keyboard::poll`.
const WINDOW: Hwnd = Hwnd(0x1234);

/// Frames a stage takes to settle before its first chapter is taken, which is `chapter.rs`'s
/// `STAGE_SETTLE_FRAMES` plus the music wait in front of it. Read off the behaviour rather than
/// imported: what this scenario needs is "far enough in that the stage has begun", and it asserts
/// the chapter was taken rather than assuming a number.
const SETTLE_FRAMES: u32 = 300;

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

/// A frame of a stage being played, as the frame hook reads one.
fn playing(frame: u32, lives: i8, deaths: i32) -> State {
    State {
        scene: 2,
        wanted: 2,
        playing: true,
        in_run: true,
        in_game: true,
        in_ending: false,
        ending_script: None,
        demo: false,
        replay: false,
        practice: false,
        paused: false,
        unsettled: false,
        bombing: false,
        in_dialogue: false,
        stage: 0,
        difficulty: 1,
        stage_frames: frame,
        script_frames: frame as i32,
        random_seed: 0x27ad,
        deaths,
        lives,
        bombs: 3,
        power: 128,
        enemy_count: enemies_on(frame),
        bullet_count: 0,
        laser_count: 0,
        boss_present: false,
        boss_life: None,
        boss_attack_frames: None,
        spellcard: None,
    }
}

/// A stage's waves in the only terms the boundary detector reads: two hundred frames of enemies and
/// two hundred without, over and over. A stage with nothing ever on it has no gap between waves to
/// find, and so no chapter but the one its start is.
fn enemies_on(frame: u32) -> i32 {
    if (frame / 200).is_multiple_of(2) {
        3
    } else {
        0
    }
}

/// The numbers the game's memory holds, which is what a chapter is a copy of.
fn in_memory(frame: u32, lives: i8, deaths: i32) -> Playing {
    Playing {
        stage: 0,
        difficulty: 1,
        frames: frame,
        script_frames: frame as i32,
        seed: 0x27ad,
        deaths,
        lives,
        bombs: 3,
        power: 128,
        enemies: enemies_on(frame),
    }
}

/// The whole run, one scenario.
#[test]
fn a_pointdevice_run_is_chosen_lost_retried_and_left_for_the_next_session() {
    let dir = std::env::temp_dir().join("orb-playthrough");
    std::fs::create_dir_all(&dir).unwrap();
    let image = Image::laid_out();
    let sim = image.sim();
    sim.display().set_foreground(WINDOW);

    // ── 1. モードを選ぶ. The question over the game's own title menu, on the press that would have
    // chosen `Game Start`. Somebody who played a pointdevice run last time finds the cursor already
    // on 完全無欠モード, so one press is the whole answer.
    let mode = {
        let _entered = image.enter();
        let mut keyboard = Keyboard::new();
        let mut question = Question::new(Menu::Run, Mode::Pointdevice);
        for _ in 0..INPUT_GRACE_FRAMES {
            keyboard.poll(WINDOW);
            assert!(
                question.update(&keyboard, Pad::default()).is_none(),
                "the press the question went up on answered it",
            );
        }
        sim.keyboard().set(keys::Z, true);
        keyboard.poll(WINDOW);
        let answered = question.update(&keyboard, Pad::default());
        sim.keyboard().set(keys::Z, false);
        match answered {
            Some((Answer::Chosen(mode), By::Keyboard)) => mode,
            _ => panic!("完全無欠モード was not chosen"),
        }
    };
    assert_eq!(mode, Mode::Pointdevice, "the run keeps its chapters");

    // ── 2. The stage begins. `keep` is what `lib.rs` calls with `chapters && resume && pointdevice`,
    // which is the one place the mode reaches the file a run is left in.
    resume::keep(mode == Mode::Pointdevice);
    // And the score file forks with it, which is the other thing the mode decides: a pointdevice run
    // is ranked against pointdevice runs, in `pointdevice_score.dat`, because a run that can be
    // retried until it is won does not belong in the same table as one that cannot.
    score::fork(mode == Mode::Pointdevice);
    let mut chapters = Chapters::new(&Th06, None, false);

    // What the game's memory holds as the stage settles, which is what chapter 1 will be a copy of.
    let started = in_memory(0, 2, 0);
    image.playing(started);
    // The two points `lib.rs` patches the game at as a stage is registered and then built. What a run
    // is written down with is read *there* and nowhere else: a record taken later would put the run
    // back with the lives and the seed of whatever it had reached, and nothing in the file would say so.
    {
        let _entered = image.enter();
        unsafe {
            resume::stage_building(&Th06);
            resume::stage_begun(&Th06);
        }
    }
    for frame in 0..=SETTLE_FRAMES {
        image.playing(in_memory(frame, 2, 0));
        observe(&image, &mut chapters, &playing(frame, 2, 0));
    }
    assert_eq!(chapters.number(), 1, "the stage's first chapter was taken");
    assert!(chapters.can_retry(), "and there is something to go back to");

    // ── 3. The run gets somewhere. A chapter of its own, further in than the stage's start, so that
    // what a retry goes back to is a boundary the detector named rather than the stage.
    let mut at = SETTLE_FRAMES;
    while chapters.number() < 2 && at < SETTLE_FRAMES + 20_000 {
        at += 1;
        image.playing(in_memory(at, 2, 0));
        observe(&image, &mut chapters, &playing(at, 2, 0));
    }
    assert!(
        chapters.number() >= 2,
        "no second chapter in {} frames of stage; reached {}",
        at - SETTLE_FRAMES,
        chapters.number(),
    );

    let lost_in = chapters.number();
    let named = chapters.name().map(|name| name.to_string());
    let at_the_boundary = state(&image);

    // ── 4. 被弾. The game takes a life, counts the death, and the stage runs on for the frames the
    // death animation takes — every one of them writing over the memory the chapter is a copy of.
    //
    // Which frame the death is *noticed* on is `lib.rs`: `state.deaths > previous.deaths` while
    // `in_game`, and only once the death bomb window has closed. Here the scenario is that
    // comparison, because the frame hook it lives in cannot be driven from outside the DLL.
    let before_death = playing(at, 2, 0);
    at += 60;
    let died_on = playing(at, 1, 1);
    image.playing(in_memory(at, 1, 1));
    observe(&image, &mut chapters, &died_on);
    assert!(
        died_on.in_game && died_on.deaths > before_death.deaths,
        "the frame hook would read this as a death",
    );
    assert_ne!(
        state(&image),
        at_the_boundary,
        "the death has to have moved the memory a retry puts back",
    );

    // ── 5. The menu where the chapter was lost, answered on the pad. `lib.rs` puts this up on the
    // frame the death is noticed; what it does with the answer is the three arms below.
    let chose = {
        let _entered = image.enter();
        let mut keyboard = Keyboard::new();
        let mut menu = RetryMenu::new();
        let mut chose = None;
        for _ in 0..INPUT_GRACE_FRAMES * 4 {
            keyboard.poll(WINDOW);
            if let Some((choice, by)) = menu.update(
                &keyboard,
                Pad {
                    decide: true,
                    ..Pad::default()
                },
            ) {
                chose = Some((choice, by));
                break;
            }
            keyboard.poll(WINDOW);
            menu.update(&keyboard, Pad::default());
        }
        chose.expect("the retry menu never answered")
    };
    assert_eq!(chose.0, Choice::Chapter, "チャプターをやり直す");
    assert_eq!(chose.1, By::Pad);

    // ── 6. チャプターの頭に戻る. The whole promise of the mode: the memory is put back, so the run
    // is where it was — the lives it had, the seed it was drawn with, the frame it was on.
    assert!(retry(&image, &mut chapters), "the chapter came back");
    let put_back = state(&image);
    assert_eq!(
        put_back, at_the_boundary,
        "the run is not where the chapter began, field for field",
    );
    // The numbers somebody playing would look at, called out by name so that a future fixture growing
    // its window cannot quietly stop covering one of them.
    assert_eq!(put_back.lives, 2, "the life the death took is back");
    assert_eq!(
        put_back.deaths, 0,
        "and the death is not counted against the run"
    );
    assert_eq!(
        put_back.stage_frames, at_the_boundary.stage_frames,
        "the stage is back at the frame the chapter began on",
    );
    assert_eq!(
        put_back.random_seed, at_the_boundary.random_seed,
        "and drawing from where it was, so the attempt is the same one",
    );
    assert_eq!(
        chapters.number(),
        lost_in,
        "and it is the chapter that was lost, not another",
    );
    assert_eq!(chapters.name().map(|n| n.to_string()), named);
    assert_eq!(chapters.retries(), 1, "one attempt counted");

    // ── 7. タイトルに戻る, with the chapter written down. This is what makes the next session able
    // to pick the run up: the file holds the chapter, the numbers the stage began with, and the
    // buttons that were pressed to get there.
    let written = {
        let _entered = image.enter();
        unsafe {
            resume::write(
                &dir,
                &Th06,
                at_the_boundary.stage_frames,
                lost_in,
                named.as_deref().unwrap_or(""),
                chapters.retries(),
            )
        }
    };
    assert!(written, "nothing was written down for the run");

    // ── 8. And the game is closed. A different `Sim` is a different process as far as anything here
    // can tell — nothing of the first one is in memory — so what the run is picked up from is the
    // file and only the file.
    let next_session = Image::laid_out();
    let saved = {
        let _entered = next_session.enter();
        resume::load(&dir, &Th06, &the_run())
    };
    let saved = saved.expect("the run was not there to pick up");
    assert_eq!(
        saved.chapter, lost_in,
        "the chapter the next session offers is the one the run was in",
    );

    // And it is offered by name, which is what the question after the character select shows.
    assert!(
        resume::left(&dir)
            .iter()
            .any(|slot| slot == "normal-reimu-a"),
        "the run is not among the ones left unfinished: {:?}",
        resume::left(&dir),
    );

    // ── 9. The ranking, on the way to the title. The run's score goes in orb's file, and the one read
    // that does *not* follow the mode is the front end's unlocks — which are what the game offers on
    // its title screen, and are the same whichever way a run was played. Called around that read and
    // nowhere else, which is the order the real log shows it in:
    //
    //   score: pointdevice_score.dat opened in place of the game's own, read
    //   score: score.dat opened as the game's own, read for the front end's unlocks
    // Called the way `lib.rs` calls them, and nothing here asserts which file was opened: what decides
    // that is an import hook on `CreateFileW` that a test cannot install, and the only thing a scenario
    // could hold it to would be an observer written for the scenario. `score.rs`'s own tests cover the
    // decision; what this covers is that the calls happen in this order at all.
    score::reading_unlocks(true);
    score::reading_unlocks(false);

    // ── 10. Given up for good, nothing is left behind: a run somebody finished or abandoned must not
    // be offered again.
    let discarded = {
        let _entered = next_session.enter();
        resume::discard(&dir, &Th06, &the_run())
    };
    assert!(discarded.is_some(), "the file was not taken away");
    assert!(
        resume::load(&dir, &Th06, &the_run()).is_none(),
        "the run is still being offered after being given up",
    );
}

/// One frame through the real `observe`, with the game in front so its reads land in it.
fn observe(image: &Image, chapters: &mut Chapters, state: &State) {
    let _entered = image.enter();
    // What the input hook does every frame of a run being kept: one entry per frame, because a chapter
    // is reached by playing the frames under it and a record that is short is a record that missed
    // some — `resume::write` refuses one. Nothing is pressed here; what is being checked is that the
    // frames were counted, not which buttons they held.
    unsafe { resume::noted(state.stage_frames, 0) };
    unsafe { chapters.observe(&Th06, state, &image.data(), false) };
}

/// The real `retry_chapter`, which is what the menu's first item does.
fn retry(image: &Image, chapters: &mut Chapters) -> bool {
    let _entered = image.enter();
    unsafe { chapters.retry_chapter(&Th06) }
}

/// What the game's memory says the run is, read the way the frame hook reads it — every field parsed
/// back out of the memory a restore landed in, rather than off the addresses this test wrote.
fn state(image: &Image) -> State {
    let _entered = image.enter();
    unsafe { Th06.read_state() }
}
