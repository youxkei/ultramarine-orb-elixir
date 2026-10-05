//! Pointdevice mode over a laid-out th06nc: the question that starts it, a chapter, a death, a retry, and
//! the run left and picked up again — and what that retry leaves alone.
//!
//! **This is what `Th06nc::rewinds` rests on**, the way `a_pointdevice_run_of_th06c.rs` is for th06c:
//! every method of that game is otherwise one answer over a laid-out address space, and none of those can
//! say the *mode* holds together — that the chapter a snapshot is taken as covers this game's nine
//! megabytes of `.data` and the heap blocks beside it, that a restore puts the stage's clock and the
//! generator back, that the file a run is left in reads back as the run it was.
//!
//! **Everything here is a recipe, and nothing presses a key** — which is this file's choice and not a
//! limit of the game's: a recipe's own answer lands whatever is on the screen, so a walk of the front end
//! needs no press to be timed against it, and a launch of the real game under `orb-real` is the same
//! thing. That orb *does* see a press of this game is `a_confirmation_declined_of_th06nc`, which answers
//! a question of orb's on the keyboard — the frame's word goes through `runtime::the_frames_word`, which
//! this game's calc walk once went around, and a launch of the real game read
//! `retry: the chapter again on the pad` off it. That the question is *on* the screen is
//! `the_mode_question_over_th06nc.rs`.
//!
//! **And every wait is bounded in frames rather than in chapters**, where th06c's module plays to its
//! second and third: this game's `chapters.rs` holds seven rows with nothing in them, so a stage of it is
//! one chapter **until its fight starts** — the laid-out game's midboss arrives at frame 400 and the
//! chapters after that are the fight's own, which is a thing of where this module's stage puts a boss
//! rather than of the table. A frame is the bound that says the same thing either way.
//!
//! **The run is left by quitting with it on and not by `leave`**, which is what a person who wants a run
//! back does: a run to pick up is a run the process ended during. Either would leave the file — the state
//! `Th06nc::leave_run` asks for builds no screen a run ends at, so `Game::run_finished` answers false for
//! it and nothing discards the slot — and quitting is the one of the two that needs no answer to a
//! question first.

use std::path::{Path, PathBuf};

use crate::fake::th06nc::{Fake, TO_A_RUN, TO_THE_LAST_SCREEN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// The name orb gives the run these play, which names its file: Normal and Reimu's first shot.
const SLOT: &str = "normal-reimu-a";

/// How many frames a launch is given to run its recipe out and quit in.
const FRAMES: u32 = 30_000;

/// How long a stage of this laid-out game lasts, so that a run left in one is left mid-stage rather than
/// walked out of it.
const A_STAGE: u32 = 1_200;

/// **The bits of this game's own word that orb drives a run with** — the shot `0x1`, the bomb `0x2`, the
/// focus `0x4`, the four directions `0x10` to `0x80` and the decide `0x100`, as `game/th06nc/asm.md` reads
/// them off the player's walk at `0x14006a790` and the binding at `0x140013499`.
///
/// **The game's own numbers and not orb's**, which are a different set — orb's decide is `0x1000` — and
/// the field this masks is the game's. Written out here rather than asked of the game, the way every other
/// number an e2e test charges is.
const THE_RUNS_OWN_BUTTONS: u32 =
    0x0100 | 0x0080 | 0x0040 | 0x0020 | 0x0010 | 0x0004 | 0x0002 | 0x0001;

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
/// **The line is cut at `hooked=`, which takes the last two fields off it**, and both for the same reason:
/// they say how each pass got to the frame and never what it reached — `updates=` the count of updates the
/// frame took, where a playback runs a stage's frames as fast as the chain will walk and a run plays one a
/// frame, and `hooked=` the generator as the *input hook* saw it, which th06c's own copy of this test
/// measured as not reproducing. See that one, and this change's description for why it matters to
/// `resume::Record`.
///
/// **And `input=` is held to the bits a run is written down with**, [`THE_RUNS_OWN_BUTTONS`]: that field is
/// the game's whole composed word and the bits above those are not a run's — the `0x8000` the frame loop
/// reads as a level and the `0x100` the front end writes for a screen going by, which two launches that
/// walked different screens hold different values of. Measured: a playback whose world agreed field for
/// field — player, randoms, generator, bullets and every fingerprint — read `input=0x0041` where the run
/// had `0x8141`.
fn the_world_each_frame_was_played_with(lines: &[String], marker: &str) -> Vec<(u32, String)> {
    lines
        .iter()
        .filter_map(|line| {
            let said = "resume: frame ";
            let rest = &line[line.find(said)? + said.len()..];
            let (frame, rest) = rest.split_once(marker)?;
            let world = rest.split(" hooked=").next()?;
            Some((frame.trim().parse().ok()?, the_runs_own_word_in(world)))
        })
        .collect()
}

/// **The stage's own first frame**, which is the frame its build happens in — and the one frame of a run
/// being picked up that goes through its own path rather than through the playback's feed.
const THE_STAGES_FIRST_FRAME: u32 = 1;

/// The last world a launch wrote down for `frame`, out of what [`the_world_each_frame_was_played_with`]
/// collected: the last, a stage retried writing its frames over again and the attempt that reached the
/// chapter being the one the file holds.
fn the_world_of(frames: &[(u32, String)], frame: u32) -> Option<&str> {
    frames
        .iter()
        .rev()
        .find(|(at, _)| *at == frame)
        .map(|(_, world)| world.as_str())
}

/// The word orb chose on `frame`, out of what [`the_world_each_frame_was_played_with`] collected —
/// `chose=` as the line spells it, and the last write for that frame, a stage retried writing its frames
/// over again.
fn the_word_orb_chose_on(frames: &[(u32, String)], frame: u32) -> Option<&str> {
    frames
        .iter()
        .rev()
        .find(|(at, _)| *at == frame)?
        .1
        .split(" chose=")
        .nth(1)?
        .split(' ')
        .next()
}

/// That world with `input=`'s own field cut down to [`THE_RUNS_OWN_BUTTONS`], and every other field of it
/// as it was written.
fn the_runs_own_word_in(world: &str) -> String {
    let Some(rest) = world.strip_prefix("input=0x") else {
        return world.to_owned();
    };
    let Some((word, after)) = rest.split_once(' ') else {
        return world.to_owned();
    };
    match u32::from_str_radix(word, 16) {
        Ok(word) => format!("input={:#06x} {after}", word & THE_RUNS_OWN_BUTTONS),
        Err(_) => world.to_owned(),
    }
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

/// The walk to a run and the death the rules bring, with the run given up after it.
fn a_run_that_dies_and_retries() -> String {
    format!(
        "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- take_hits: {{ chance: 1.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 400 }}
- take_hits: {{ chance: 0.0, times_each_retry: 0.0, gives_up: 0.0 }}
- play_until: {{ frames: 600 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
    )
}

#[test]
fn a_pointdevice_run_of_th06nc_is_chosen_chaptered_lost_and_retried() {
    in_its_own_process(|| {
        let game = launched("th06nc-pointdevice", a_run_that_dies_and_retries());
        let log = game.log();
        played_out(&game);

        // ── 1. The mode, asked at the menu's first item and answered by the recipe, which is how a launch
        // that plays itself answers it whatever is on the screen.
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

        // ── 2. The run and its first chapter, written down for a later launch with the numbers this
        // game's own stage build leaves.
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
        // recipe answers the retry, the snapshot goes back, and the stage's clock and the generator with
        // it.
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

        // ── 4. The run left for the front end, with what it counted held for a ranking.
        assert!(
            log.said("retry: the run is given up; the game is on its way to the title")
                || log.said("run ended"),
            "the run was not given up: {:?}",
            log.lines()
        );
        assert!(
            log.said(
                "score: a run ended; what it counted waits for the ranking to be built and taken \
                      down"
            ),
            "what the run counted was not noted at the run's end: {:?}",
            log.lines()
        );
        // **And no ranking is built for it**, which is what `Th06nc::writes_its_own_records` answering
        // true is for: the game writes its own score file — as the screen a run ends at goes down, as a
        // run is left for the front end and as one is started again — and the ranking the main menu's
        // item 5 builds is one that writes nothing, so a screen orb built and took down would leave the
        // file as it was.
        //
        // **Read as the ranking never being driven rather than as the line that says so**, which is this
        // game's own gap and worth naming here: `runtime::commit_records` is reached only where
        // `State::scene` and `State::wanted` agree, and `Th06nc::read_state` answers the front end's own
        // screen number against a hardcoded `-1` — a pair that cannot agree unless the front end is out
        // of the chain, which is to say unless a run is playing. So the line that function writes is a
        // line no run of this game reaches, and what stands instead is that none of the four the ranking
        // would have written is there. This change's own `TODO:` carries it.
        for said in [
            "score: the ranking is up",
            "score: asked it to leave",
            "score: update 1 answered",
        ] {
            assert!(
                !log.said(said),
                "a ranking was driven for a run of this game — {said}: {:?}",
                log.lines()
            );
        }

        // ── 5. **And the game's own file is not written by a run of the mode**, which is the whole of
        // what the fork is for: a run orb rewound is not a run to rank a person's own file by.
        assert!(
            game.score_file_opens()
                .iter()
                .all(|(name, write)| !(name == "score.dat" && *write)),
            "the game's own file was written by a run of the mode: {:?}",
            game.score_file_opens()
        );
        // **And what the run counted lands in the mode's own file**, the game writing it on its way out:
        // orb gives a run up the way the pause menu's own give-up does, asking for the front end (0x14000b48e
        // under `[0x14053d3dc]` clear), whose 0x14007b770 writes the score file before it builds the title.
        assert!(
            game.score_file_written_as("pointdevice_score.dat"),
            "the run was given up without the game writing what it counted: {:?}",
            game.score_file_opens()
        );
    });
}

/// The two words the test below tells apart: one inside the block the frame's sound effects are queued
/// through and one below it, each as it stands when the chapter is taken and as it stands by the time the
/// chapter is put back.
///
/// Words and not a lock's real fields: what orb does with that block is hold the whole of it back from a
/// restore, and it reads no word of it — see `Th06nc::live_handles`. What they have to be is two numbers
/// that can be told apart.
const AT_THE_CHAPTER: u32 = 0x1111_1111;
const WRITTEN_SINCE: u32 = 0x2222_2222;

/// How many bytes that block is, which is how the restore names it in the log.
const THE_BLOCK: usize = 0xd2c;

/// **The five queues the frame's sound effects are flushed through are left where the restore finds them,
/// and the memory below them comes back.**
///
/// A chapter put back over that block says a thread holds a queue's lock that does not and that a worker
/// has work to do that it has already done, and a restore rewinds neither the threads nor what they are
/// doing. On the real game a chapter put back over it stopped the frame's own thread inside
/// `EnterCriticalSection` eight frames later and it never came out — orb's other threads went on logging
/// for seconds past the last frame. See `Th06nc::live_handles` for the sampler's stack that said so.
///
/// **The word below the block is the half that makes the first half say something**: a restore that put
/// nothing back at all would leave the block's word alone by accident.
#[test]
fn the_medias_queues_are_not_put_back_from_a_snapshot_of_th06nc() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-medias-queues", a_run_that_dies_and_retries());
        let log = game.log();

        // Both words as the game has them for every chapter of the run, so that whichever chapter the
        // stage's own fire reaches the player in is one holding these.
        game.image().the_medias_word_is(AT_THE_CHAPTER);
        game.image()
            .the_word_below_the_medias_queues_is(AT_THE_CHAPTER);
        game.frames_until("the run lost a life", FRAMES, || {
            log.said("died in chapter")
        });

        // And then what the library's own threads get up to while the retry menu is up, with a write of
        // the game's own below it: a dead run takes no further chapter, so neither word is in one.
        game.image().the_medias_word_is(WRITTEN_SINCE);
        game.image()
            .the_word_below_the_medias_queues_is(WRITTEN_SINCE);
        game.frames_until("the chapter was put back", FRAMES, || log.said("(retry 1)"));

        // Named among the ranges the restore skipped by its own length, and not by how many there were:
        // what else it holds back is the renderer's, the sound layer's and the music's, which are other
        // claims.
        assert!(
            log.said(&format!("+{THE_BLOCK:#x}")),
            "the restore named no block of the media's to leave alone:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_medias_word(),
            WRITTEN_SINCE,
            "the restore put the media's queues back a chapter:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_word_below_the_medias_queues(),
            AT_THE_CHAPTER,
            "the restore put nothing back at all, so leaving the media's queues alone means \
             nothing:\n  {}",
            log.lines().join("\n  ")
        );

        played_out(&game);
    });
}

/// **A run left mid-stage, found by the next launch, and put back frame for frame.**
///
/// Which is the claim picking a run up rests on and the one
/// `the_numbers_a_run_of_th06nc_was_made_with` does not make: that one asks what the run *carried*, and
/// this one asks that every frame the playback fed is the frame the run played.
#[test]
fn the_run_of_th06nc_left_in_a_chapter_is_offered_and_picked_up() {
    in_its_own_process(|| {
        let left = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 3600
- play_until: {{ frames: 600 }}
  give_up_after: 6000
- quit: true
"
        );
        let first = launched("th06nc-left", left);
        first.stages_last(A_STAGE);
        played_out(&first);
        assert!(
            first
                .log()
                .said("resume: stage 1 chapter 1 (MIDSTAGE 1) at frame"),
            "the run never had a chapter written down: {:?}",
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

        // The second launch, finding what the first left, offered the run at the last screen before a
        // stage and answering つづきから: the run comes back in the chapter it was left in and plays on.
        let picked_up = format!(
            "{TO_A_RUN}{TO_THE_LAST_SCREEN}\
- press: [decide]
  frames: 6
  until: {{ question: resume }}
  give_up_after: 1800
- answer: {{ question: resume, item: 0 }}
  give_up_after: 600
- play_until: {{ frames: 300 }}
  give_up_after: 6000
- leave: true
- wait_until: {{ frames: 300 }}
- quit: true
"
        );
        // **Found at the attach and not put in afterwards**: what a launch has to offer it reads as it
        // comes up, so the files go in before orb does.
        let second = Fake::attach_finding(
            Display::agreed(120),
            "th06nc-picked-up",
            &files,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(picked_up);
                config.log_level = LogLevel::Verbose;
                config.save_replays = true;
            },
        );
        second.stages_last(A_STAGE);
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
        // **And the playback's own series begins one frame later than the run's**, which is the frame the
        // stage's build and the landing spend: the run wrote its first line on [`THE_STAGES_FIRST_FRAME`]
        // and nothing of the playback reads the keyboard inside the build, so the first frame it feeds is
        // the one after. Held because that offset is what every accounting of a playback against a run
        // rests on — `resume: N of M frame(s) played back, K update(s) in` reads two apart for it, which
        // is a constant of the landing and not a count coming out uneven — and because it is the whole of
        // what the two passes do differently at a stage's start.
        assert_eq!(
            back.first().map(|(frame, _)| *frame),
            Some(THE_STAGES_FIRST_FRAME + 1),
            "the playback's first fed frame is not the one after the stage's own first:\n    {:?}",
            back.first(),
        );
        // **And the frame the stage is built in was played with the word the run's own file holds for
        // it**, which is the one frame a playback does not feed through its own loop: the build and the
        // landing happen inside it, so the word goes in through the frame's own read — see
        // `runtime::the_frames_word`, whose comment carries the pair of real launches, and
        // `resume::the_word_the_file_holds_for`, which is keyed by that read's own frame.
        //
        // **And the frame the stage is built in is played with nothing**, which is the one frame of a
        // resumed stage no word reproduces — not because the run's own moved nobody, which is refuted,
        // but because the file holds none for it: the record is keyed by `Game::stage_frame`, and that
        // read has no stage to be keyed under. See `runtime::the_frames_word`, whose comment carries the
        // launches that measured both, and `resume`'s own note where the lookup was taken out.
        let the_pick_ups_own = the_world_each_frame_was_played_with(&log.lines(), " played with ");
        assert_eq!(
            the_word_orb_chose_on(&the_pick_ups_own, THE_STAGES_FIRST_FRAME),
            Some("0x0000"),
            "the frame the stage was built in was played with a word of somebody's:\n    {:?}\n    {:?}",
            the_world_of(&the_pick_ups_own, THE_STAGES_FIRST_FRAME),
            the_world_of(&played, THE_STAGES_FIRST_FRAME),
        );
        // **And what that frame started was started instead**, which is the seam that stands in for the
        // word — `Game::starts_the_shot_a_stage_began_with`. Any of its three lines is it having had its
        // say: the cycle put back, a pair already running that it left alone, or a run that was not firing
        // across the stage's start and gets no cycle. **This fake is the third**, its own first word being
        // the recipe's decide; the real game's is the first.
        assert!(
            log.said("resume: the stage's shot cycle begins on phase")
                || log.said("resume: the stage's shot is already on phase")
                || log.said("resume: the run was not firing as the stage began"),
            "nothing says the stage's shot cycle was looked at as the run landed: {:?}",
            log.lines()
                .into_iter()
                .filter(|line| line.contains("resume: the stage's shot"))
                .collect::<Vec<_>>()
        );
        assert!(
            log.said(
                "resume: the playback stopped because it reached the frame the chapter began on"
            ),
            "the playback stopped short of the chapter it was going to: {:?}",
            log.lines()
        );
        // The run reached its chapter with `deaths: 0`, so a playback of the path to it that takes a hit
        // has come out of step somewhere at or before the hit whatever the lines above agree on.
        assert!(
            !log.said("resume: the player was hit at frame"),
            "the playback was hit on a path the run survived: {:?}",
            log.lines()
        );
    });
}
