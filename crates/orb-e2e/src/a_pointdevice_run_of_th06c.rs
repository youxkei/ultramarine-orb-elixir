//! Pointdevice mode over a laid-out th06c: the question that starts it, a chapter, a death, a retry, and the run
//! left and picked up again — and what that retry leaves alone.
//!
//! **This is what `Th06c::rewinds` rests on**, the way `a_pointdevice_run_of_th07.rs` is for th07: every
//! method of that game is otherwise one answer over a laid-out address space, and none of those can say the
//! *mode* holds together — that the chapter a snapshot is taken as covers this game's nine megabytes of
//! `.data` and the heap blocks beside it, that a restore puts the stage's clock and the generator back, that
//! the file a run is left in reads back as the run it was.
//!
//! **Everything here is a recipe**, and nothing presses a key: orb notices the decide for this game only
//! where a word is being fed — `runtime::the_frames_word` — so a question of orb's is answered by the
//! recipe or by nobody, whatever is on the screen. Which is also what a launch of the real one is. That the
//! question is *on* the screen is `the_mode_question_over_th06c.rs`.

use std::path::{Path, PathBuf};

use crate::fake::th06c::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06c::score_file::{self, CLEARED, Record};

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file: Normal, and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 30_000;

fn launched(name: &str, recipe: String) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            // Every replay kept without asking, these being about the chapter file.
            config.save_replays = true;
        },
    )
}

/// The recipe run out, the game asked to close by its last step.
fn played_out(game: &Fake) {
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    panic!(
        "the recipe never quit in {FRAMES} frames:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// **The world each frame of a stage was played with, by the frame it was played on**, read off the
/// `resume: frame N played with` lines a verbose launch writes — `marker` choosing those or the
/// `played back with` ones a playback writes.
///
/// The count of updates the frame took is cut off the end: a playback runs a stage's frames as fast as the
/// chain will walk and a run plays one a frame, so that field says how each pass got there and never what
/// it reached.
fn the_world_each_frame_was_played_with(lines: &[String], marker: &str) -> Vec<(u32, String)> {
    lines
        .iter()
        .filter_map(|line| {
            let said = "resume: frame ";
            let rest = &line[line.find(said)? + said.len()..];
            let (frame, rest) = rest.split_once(marker)?;
            Some((
                frame.trim().parse().ok()?,
                rest.split(" updates=").next()?.to_owned(),
            ))
        })
        .collect()
}

/// What a launch left under orb's directory, re-rooted for the next.
fn left_by(game: &Fake, dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            let inside = path.strip_prefix(dir).ok()?.to_path_buf();
            Some((inside, files.get(&path)?))
        })
        .collect()
}

#[test]
fn a_pointdevice_run_of_th06c_is_chosen_chaptered_lost_and_retried() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 400 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched("th06c-pointdevice", recipe);
        let log = game.log();
        played_out(&game);

        // ── 1. The mode, asked at the title's item and answered by the recipe, which is how a launch that
        // plays itself answers it whatever is on the screen.
        assert!(
            log.said("mode: pointdevice to start with; the menu asks which"),
            "the launch did not start in the mode orb is for: {:?}",
            log.lines()
        );
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "the press did not put the question up: {:?}",
            log.lines()
        );
        assert!(
            log.said("mode: answered on the recipe's own ask"),
            "the recipe did not answer the question: {:?}",
            log.lines()
        );

        // ── 2. The run and its first chapter, written down for a later launch with the numbers this game's
        // own replay restarts a stage from.
        assert!(
            log.said("stage 1 chapter 1 (stage start)"),
            "the stage's first chapter was never taken: {:?}",
            log.lines()
        );
        assert!(
            log.said("resume: stage 1 chapter 1 (MIDSTAGE 1) at frame"),
            "the chapter was not written down: {:?}",
            log.lines()
        );

        // ── 3. The death, counted off the hit this game keeps no count of, and the chapter again: the
        // recipe answers the retry, the snapshot goes back, and the stage's clock and the generator with it.
        // Whichever chapter the stage's own fire reached the player in — which is the rules' business and
        // moves as they change, the reason the real recipe bounds this by frames rather than by a chapter.
        assert!(
            log.said("died in chapter"),
            "the hit the recipe took was not read as a death: {:?}",
            log.lines()
        );
        assert!(
            log.said("retry: the chapter again on the pad")
                || log.said("retry: the chapter again on the recipe's own ask"),
            "the retry was not answered with the chapter again: {:?}",
            log.lines()
        );
        assert!(
            log.said("(retry 1)"),
            "the chapter was not put back: {:?}",
            log.lines()
        );

        // ── 4. The run left for the title, which is what a later launch picks up, and its records written
        // through the ranking this game's own Score item builds — into the mode's own file.
        assert!(
            log.said("retry: the run is given up; the game is on its way to the title"),
            "the run was not given up: {:?}",
            log.lines()
        );
        assert!(
            log.said("score: the ranking built and taken down"),
            "the ranking was not built for the records: {:?}",
            log.lines()
        );
        assert!(
            game.score_file_written_as("pointdevice_score.dat"),
            "the records did not land in the mode's own file: {:?}",
            game.score_file_opens()
        );
        assert!(
            !game.score_file_written_as("score.dat")
                || game
                    .score_file_opens()
                    .iter()
                    .all(|(name, write)| !(name == "score.dat" && *write)),
            "the game's own file was written by a run of the mode: {:?}",
            game.score_file_opens()
        );
    });
}

/// The two words the test below tells apart: one inside the sound library's block and one below it, each
/// as it stands when the chapter is taken and as it stands by the time the chapter is put back.
///
/// Words and not a lock's real fields: what orb does with that block is hold the whole of it back from a
/// restore, and it reads no word of it — see `Th06c::live_handles`. What they have to be is two numbers
/// that can be told apart.
const AT_THE_CHAPTER: u32 = 0x1111_1111;
const WRITTEN_SINCE: u32 = 0x2222_2222;

/// How many bytes the sound library's block is, which is how the restore names it in the log.
const THE_BLOCK: usize = 0xe00;

/// **The sound library's own block is left where the restore finds it, and the memory below it comes back.**
///
/// A chapter put back over that block says a thread holds a lock that does not and that a worker has work
/// to do that it has already done, and a restore rewinds neither the threads nor what they are doing. On
/// the real game that cost the frame 35ms to 137ms every time a shot hit an enemy, for the rest of the run
/// after a retry — see `Th06c::live_handles` for the samples that said so.
///
/// **The word below the block is the half that makes the first half say something**: a restore that put
/// nothing back at all would leave the library's word alone by accident.
#[test]
fn the_sound_librarys_block_is_not_put_back_from_a_snapshot_of_th06c() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 400 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let game = launched("th06c-the-sound-librarys-block", recipe);
        let log = game.log();

        // Both words as the game has them for every chapter of the run, so that whichever chapter the
        // stage's own fire reaches the player in is one holding these — which chapter that is being the
        // rules' business and no part of this.
        game.image().the_sound_librarys_word_is(AT_THE_CHAPTER);
        game.image()
            .the_word_below_the_sound_library_is(AT_THE_CHAPTER);
        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });

        // And then what the library's own threads get up to while the retry menu is up, with a write of
        // the game's own below it: a dead run takes no further chapter, so neither word is in one.
        game.image().the_sound_librarys_word_is(WRITTEN_SINCE);
        game.image()
            .the_word_below_the_sound_library_is(WRITTEN_SINCE);
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        // Named among the ranges the restore skipped by its own length, and not by how many there were:
        // what else it holds back is the renderer's and the music's, which are other claims.
        assert!(
            log.said(&format!("+{THE_BLOCK:#x}")),
            "the restore named no block of the sound library's to leave alone:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_sound_librarys_word(),
            WRITTEN_SINCE,
            "the restore put the sound library's block back a chapter:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_word_below_the_sound_library(),
            AT_THE_CHAPTER,
            "the restore put nothing back at all, so leaving the library's block alone means \
             nothing:\n  {}",
            log.lines().join("\n  ")
        );

        played_out(&game);
    });
}

#[test]
fn the_run_of_th06c_left_in_a_chapter_is_offered_and_picked_up() {
    in_its_own_process(|| {
        let left = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        let first = launched("th06c-left", left);
        played_out(&first);
        assert!(
            first.log().said("stage 1 chapter 2"),
            "the run never reached its second chapter: {:?}",
            first.log().lines()
        );
        let played = the_world_each_frame_was_played_with(&first.log().lines(), " played with ");
        let files = left_by(&first, first.launch().dir());
        assert!(
            files.iter().any(|(inside, _)| {
                inside.to_string_lossy().contains(SLOT)
                    && inside.to_string_lossy().contains("pointdevice_resume")
            }),
            "no chapter file of {SLOT} was left: {:?}",
            files.iter().map(|(inside, _)| inside).collect::<Vec<_>>()
        );
        drop(first);

        // The second launch, finding what the first left, offered the run as the shot select decides one
        // and answering つづきから: the run comes back in the chapter it was left in and plays on.
        let picked_up = format!(
            "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ stage: 1, chapter: 3 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        // **Found at the attach and not put in afterwards**: what a launch has to offer it reads as it comes
        // up, so the files go in before orb does.
        let second = Fake::attach_finding(
            Display::agreed(120),
            "th06c-picked-up",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(picked_up);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        played_out(&second);
        let log = second.log();
        assert!(
            log.said(&format!("resume: 1 run(s) left unfinished: {SLOT}")),
            "the run left was not found: {:?}",
            log.lines()
        );
        assert!(
            log.said(&format!("resume: {SLOT} was left; asking where to start")),
            "the run was not offered: {:?}",
            log.lines()
        );
        assert!(
            log.said("resume: from where it stopped"),
            "the offer was not answered with where it stopped: {:?}",
            log.lines()
        );
        // **And the playback put the run back frame for frame**, which is the whole claim of picking a run
        // up and the one thing the assertions above do not reach. They are all about the offer; a playback
        // that answers it and then plays another stage answers them too.
        //
        // **Asked of every frame the playback fed and not of where it got to.** `stage 1 chapter 3` below is
        // reached by a run that diverged as readily as by one that did not — the player dies, the rules take
        // the retry the mode offers, and the run carries on into that chapter all the same — and a real
        // case's trace names the chapters a run reached, so it passes for that same reason. What parts is a
        // frame's own world, and a comparison of the two launches' lines is where it shows.
        let back = the_world_each_frame_was_played_with(&log.lines(), " played back with ");
        let parted: Vec<String> = back
            .iter()
            .filter_map(
                |(frame, world)| match played.iter().rev().find(|(at, _)| at == frame) {
                    Some((_, was)) if was == world => None,
                    Some((_, was)) => {
                        Some(format!("frame {frame}\n    run: {was}\n    back: {world}"))
                    }
                    None => Some(format!("frame {frame} is one the run never played")),
                },
            )
            .take(4)
            .collect();
        assert!(
            !back.is_empty(),
            "the playback fed no frame at all: {:?}",
            log.lines()
        );
        assert!(
            parted.is_empty(),
            "the playback parted from the run it was put back from:\n  {}",
            parted.join("\n  ")
        );
        assert!(
            log.said(
                "resume: the playback stopped because it reached the frame the chapter began on"
            ),
            "the playback stopped short of the chapter it was going to: {:?}",
            log.lines()
        );
        // The run reached every chapter of its own with `deaths: 0`, so a playback of the path to one that
        // takes a hit has come out of step somewhere at or before the hit whatever the lines above agree on.
        assert!(
            !log.said("resume: the player was hit at frame"),
            "the playback was hit on a path the run survived: {:?}",
            log.lines()
        );
        assert!(
            log.said("stage 1 chapter 3"),
            "the picked-up run never went on into its third chapter: {:?}",
            log.lines()
        );
    });
}

/// The walk to the title's Start item and orb's question over it, which is [`TO_A_RUN`] up to the answer.
const TO_THE_MODE_QUESTION: &str = "\
- wait_until: { screen: 1 }
  give_up_after: 7200
- press: [decide]
  frames: 6
- wait_until: { screen: 2 }
  give_up_after: 36000
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
  until: { question: mode }
  give_up_after: 1800
";

/// The records two launches before left: the game's own file with a ranking and a Normal clear, and the mode's
/// own with a ranking of its own and no clear.
fn the_games_own_records() -> Vec<Record> {
    vec![
        score_file::th6k(),
        score_file::record(b"HSCR", &999_u32.to_le_bytes()),
        score_file::clrd(0, [1, CLEARED, 1, 1, 1], [1; 5]),
    ]
}

fn the_modes_own_records() -> Vec<Record> {
    vec![
        score_file::th6k(),
        score_file::record(b"HSCR", &5_u32.to_le_bytes()),
        score_file::clrd(0, [1; 5], [1; 5]),
    ]
}

/// **This game reads its file once, at startup, and every screen parses that copy** — so the fork that sends
/// each read of th06 and th07 to the mode's file has one read to send, before anybody has been asked which
/// mode. What orb does instead: the startup read lands in the game's own file, whose `CLRD` records are what
/// the title lights `Extra Start` from, and at the answer the copy is put together again — the mode's own
/// runs under the game's own unlocks — and the game's next parse reads that.
#[test]
fn the_copy_th06c_parses_is_the_modes_runs_under_the_games_unlocks() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_THE_MODE_QUESTION}\
- answer: {{ question: mode, item: 0 }}
  give_up_after: 300
- wait_until: {{ screen: 7 }}
  give_up_after: 1800
- wait_until: {{ frames: 60 }}
- quit: true
"
        );
        let game = launched("th06c-the-copy", recipe);
        game.its_records_are(&the_games_own_records());
        game.the_modes_file_holds(&the_modes_own_records());
        let log = game.log();
        // Before anything is asked, the copy is the game's own file — the startup read landed there.
        assert_eq!(
            game.the_records_the_game_parses(),
            Some(the_games_own_records()),
            "the copy the startup read made is not the game's own file"
        );
        played_out(&game);

        assert!(
            log.said("mode: answered on the recipe's own ask"),
            "the recipe did not answer the question: {:?}",
            log.lines()
        );
        assert_eq!(
            game.the_records_the_game_parses(),
            Some(vec![
                score_file::th6k(),
                score_file::record(b"HSCR", &5_u32.to_le_bytes()),
                score_file::clrd(0, [1, CLEARED, 1, 1, 1], [1; 5]),
            ]),
            "the copy is not the mode's runs under the game's unlocks: {:?}",
            log.lines()
        );
        assert!(
            log.said(
                "score: the copy the game parses is pointdevice_score.dat's runs under score.dat's unlocks"
            ),
            "the log did not say the copy was put together: {:?}",
            log.lines()
        );
        // And neither file was written: no ranking has been built.
        assert!(
            game.score_file_opens().iter().all(|(_, write)| !write),
            "a file was written by a launch that ran nothing: {:?}",
            game.score_file_opens()
        );
    });
}

/// And the other answer: the copy of a normal-mode launch is the game's own file, whole, since that is the
/// file such a run's records belong in.
#[test]
fn the_copy_th06c_parses_in_normal_mode_is_the_games_own_file() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_THE_MODE_QUESTION}\
- answer: {{ question: mode, item: 1 }}
  give_up_after: 300
- wait_until: {{ screen: 7 }}
  give_up_after: 1800
- wait_until: {{ frames: 60 }}
- quit: true
"
        );
        let game = launched("th06c-the-copy-normal", recipe);
        game.its_records_are(&the_games_own_records());
        game.the_modes_file_holds(&the_modes_own_records());
        let log = game.log();
        played_out(&game);

        assert!(
            log.said("mode: normal, was pointdevice"),
            "the recipe did not answer normal: {:?}",
            log.lines()
        );
        assert_eq!(
            game.the_records_the_game_parses(),
            Some(the_games_own_records()),
            "the copy of a normal-mode launch is not the game's own file: {:?}",
            log.lines()
        );
        assert!(
            log.said(
                "score: the copy the game parses is score.dat's runs under score.dat's unlocks"
            ),
            "the log did not say the copy was put together: {:?}",
            log.lines()
        );
    });
}

/// **The Extra the title offers is lit from the game's own file whichever mode the launch is in**: a launch
/// in pointdevice mode whose mode's file holds no clear still has the item, since the unlocks come out of
/// the game's own — which is what the real case the_extra_a_cleared_run_unlocked walks into.
#[test]
fn the_extra_of_th06c_is_lit_from_the_games_own_records_in_pointdevice_mode() {
    in_its_own_process(|| {
        let recipe = format!(
            "{TO_THE_MODE_QUESTION}\
- answer: {{ question: mode, item: 0 }}
  give_up_after: 300
- wait_until: {{ screen: 7 }}
  give_up_after: 1800
- press: [bomb]
  frames: 6
  until: {{ screen: 2 }}
  give_up_after: 900
- wait_until: {{ frames: 30 }}
- press: [down]
  frames: 6
  until: {{ screen: 2, cursor: 1 }}
  give_up_after: 900
- wait_until: {{ frames: 30 }}
- quit: true
"
        );
        let game = launched("th06c-the-extra-lit", recipe);
        game.its_records_are(&the_games_own_records());
        game.the_modes_file_holds(&the_modes_own_records());
        let log = game.log();
        played_out(&game);
        assert!(
            log.said("front end: screen 2, cursor on item 1"),
            "the title never had its cursor on Extra Start: {:?}",
            log.lines()
        );
    });
}
