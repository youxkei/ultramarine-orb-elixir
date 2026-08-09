//! **The ending run out inside one frame, and its staff roll left to play.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! **What an ending is made of**, read out of `紅魔郷ED.DAT`: 33 entries, unpacked the way the game
//! does — a table whose every number is two bits of length and then that many bytes, and LZSS over an
//! 8kB window with 13-bit offsets, 4-bit lengths and the window written from 1. An entry runs to the
//! next one and the archive keeps the sum of those bytes, so the table having been read right is checked
//! rather than assumed. An ending is one file per part of it: `end00`, `end01`, `end10` and `end11` for
//! Reimu and Marisa with each shot, `end00b` and `end10b` for a clear on Easy or with a continue, and
//! `staff00.end` for the roll. **All six end on `@Fdata/staff00.end`**, so the roll is the ending's last
//! script and nothing else marks where it begins; `@mbgm/th06_16.mid` is the one track an ending plays,
//! and `staff00.end` starts `bgm/th06_17` for the roll. The waits in each add up to 23,340 frames for
//! Reimu A, 26,940 for Reimu B, 32,940 for Marisa A, 34,140 for Marisa B, 6,540 for either bad ending,
//! and 7,830 — a little over two minutes — for the roll.
//!
//! **What the laid-out game reproduces is the two signals and the arithmetic between them**, not the
//! wall clock: an ending laid out with those frames of waits, the script it hands over to, and the track
//! each part plays — see `Fake::lays_out_an_ending`. The seconds and the milliseconds in the
//! measurements below are the real game's.

use crate::fake::th06::{Fake, STAGES, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Scene;

/// How long each stage of these runs is, in frames, and how many frames a whole run of six needs — the
/// same shape `a_clear_on_demand.rs` uses, since `--clear` is how an ending is reached at all.
const STAGE_FRAMES: u32 = 300;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES as u32 * 2;

/// How many updates the ending's own script takes before it hands over.
///
/// Measured: **29,040** on a stage 6 clear, which is what `Fake::lays_out_an_ending` is given so that
/// the count a scenario reads off the log is the count the real one wrote.
const ENDING_UPDATES: i32 = 29_040;

/// And the frames of waits in the roll's own script, `staff00.end`: **7,830**, a little over two
/// minutes, counted out of the script itself rather than off a run.
const ROLL_FRAMES: i32 = 7_830;

/// The two lines orb writes about running an ending out, one per way it can end: at the roll, where the
/// script it was reading handed over, or at the scene, where there was no script to compare.
const RAN_OUT: &str = "ending run out in";
const SKIPPED: &str = "ending skipped,";

/// A `--clear` launch with the ending laid out, run up to and including the frame orb runs the ending out
/// in — and holding every frame before it to never having found the ending running.
///
/// **Which is what "inside the frame it begins on" means**, and it is asserted here rather than in one
/// scenario because both of them rest on it: drawing happens once a frame, so a frame of the loop that
/// found the scene already the ending's would be a frame of the ending that reached the screen.
///
/// `--clear` because that is how an ending is reached without half an hour of playing well, and it is
/// what every one of the measurements below was taken on.
fn clearing_into_the_ending(name: &str, lay_out: impl FnOnce(&Fake), ran_out: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.fast_clear = true;
        config.log_level = LogLevel::Verbose;
    });
    game.stages_last(STAGE_FRAMES);
    lay_out(&game);
    game.in_a_run_nobody_was_asked_about();
    game.puts_a_bullet_on_the_player();
    game.frames_until("the ending run out", A_WHOLE_RUN, || {
        if game.log().said(ran_out) {
            return true;
        }
        assert_ne!(
            game.image().scene(),
            Scene::Ending,
            "a frame of the loop found the ending still running, so part of it reached the screen",
        );
        false
    });
    game
}

/// The line orb wrote about running the ending out, whichever of the two it was.
///
/// # Panics
/// Where it wrote neither, with the whole log: everything below reads a number out of this line.
fn the_line(game: &Fake, holding: &str) -> String {
    game.log()
        .lines()
        .iter()
        .find(|line| line.contains(holding))
        .unwrap_or_else(|| {
            panic!(
                "orb wrote no {holding:?} line:\n  {}",
                game.log().lines().join("\n  ")
            )
        })
        .clone()
}

/// How many updates a line of either kind says were run.
///
/// Read out of the line rather than counted here, because the line is what somebody reading a real run's
/// log has: a scenario that counted its own updates would be asserting about itself.
fn updates_in(line: &str) -> i32 {
    line.split_whitespace()
        .find_map(|word| word.parse::<i32>().ok())
        .unwrap_or_else(|| panic!("no count of updates in {line:?}"))
}

/// The two track identities a `track A -> B` line names, in the order it names them.
fn tracks_in(line: &str) -> (String, String) {
    let (_, tracks) = line
        .split_once("track ")
        .unwrap_or_else(|| panic!("no tracks in {line:?}"));
    let (before, after) = tracks
        .split_once(" -> ")
        .unwrap_or_else(|| panic!("only one track in {line:?}"));
    (before.trim().to_owned(), after.trim().to_owned())
}

/// The ending runs out inside the frame it begins on, and stops where it hands over to the roll.
///
/// Measured on a stage 6 clear: **29,040 updates inside the frame it began on**, stopping at
/// `ending run out in 29040 update(s), where its staff roll begins, track Some(1727006158) ->
/// Some(3570673472)` — the script and the track changing on the same update, which is the two signals
/// agreeing on the boundary. Nothing of the ending reached the screen.
#[test]
fn the_ending_runs_out_inside_one_frame_and_stops_at_the_roll() {
    in_its_own_process(|| {
        let game = clearing_into_the_ending(
            "the-ending-run-out",
            |game| game.lays_out_an_ending(ENDING_UPDATES, ROLL_FRAMES),
            RAN_OUT,
        );
        let line = the_line(&game, RAN_OUT);
        assert_eq!(
            updates_in(&line),
            ENDING_UPDATES,
            "the ending's script did not run out inside one frame: {line}",
        );

        // Stopped where it handed over rather than at the scene change, which is what keeps the roll: the
        // scene is still the ending's and there are the roll's own frames left to play.
        assert_eq!(
            game.image().scene(),
            Scene::Ending,
            "the skip ran past the roll to the scene change: {line}",
        );
        assert!(
            !game.log().said(SKIPPED),
            "orb ran the scene out as well as the script: {line}",
        );

        // And the track changed on the same update, which is the second signal: the ending plays
        // `@mbgm/th06_16.mid` and the roll's script starts one of its own, so two things say the boundary
        // is here rather than one.
        let (before, after) = tracks_in(&line);
        assert_ne!(
            before, after,
            "the track did not change where the script handed over: {line}",
        );
        assert!(
            before.starts_with("Some(") && after.starts_with("Some("),
            "one of the two updates had no track to read: {line}",
        );
    });
}

/// The roll plays on its own afterwards, at the rate everything else is paced at.
///
/// Measured over the same clear: **7,286 drawn frames over 122.0 seconds**, 16.74ms each, with
/// `0 shown late` and the audio never behind, and the scene after it was 7, the result screen.
///
/// The roll's own length is the 7,830 frames of waits in `staff00.end`, and it is played out an update a
/// frame — the skip does not start again over it, the ending's flag staying set and the scene staying 10
/// through both. The rate is what the loop paces every other frame at. The audio half is the real run's:
/// a laid-out game streams no sound.
///
/// **Its updates and the frames it was drawn in are not the same count**, which is why the length is read
/// off the roll and the rate off the frames. The run ended when the ending began, and what a run that
/// ended waits for is the trip through the ranking — a trip that finds no front end up spends its whole
/// allowance of updates saying so, `score: the ranking was not built after 240 update(s)`, and every one
/// of those is an update of the roll inside a single frame. Whether that is part of what left the real
/// roll **544 frames short** of its 7,830 is not settled here or anywhere — see
/// `the_ending_and_the_roll_together_come_to_the_waits_in_the_script` below, which carries that number
/// and the 62 frames beside it.
#[test]
fn the_staff_roll_plays_at_sixty_and_the_result_screen_follows_it() {
    in_its_own_process(|| {
        let game = clearing_into_the_ending(
            "the-ending-the-roll",
            |game| game.lays_out_an_ending(ENDING_UPDATES, ROLL_FRAMES),
            RAN_OUT,
        );
        // Some of the roll is already behind: the run ended when the ending began, and the trip through the
        // ranking it waits for spends its whole allowance of updates on a frame with no front end up to
        // build one. Named rather than left to arithmetic, because it is what the counts below differ by.
        assert!(
            game.log().said("score: the ranking was not built after"),
            "the trip through the ranking did not happen where the run ended:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The roll played out, a frame of the loop at a time.
        let handovers_before = game.handovers_us().len();
        game.frames_until(
            "the roll played out",
            A_WHOLE_RUN + ROLL_FRAMES as u32,
            || game.image().scene() != Scene::Ending,
        );
        assert_eq!(
            game.image().front_end_now().frames,
            ENDING_UPDATES + ROLL_FRAMES,
            "the roll did not play out the frames of waits in its own script",
        );
        // And the result screen after it, which is the scene a run's own ending hands over to.
        assert_eq!(
            game.image().scene(),
            Scene::Result,
            "the scene after the roll is not the one a run's result is shown on",
        );

        // At the rate everything else is paced at: the frames it was drawn in, a refresh apart. The mean
        // rather than each turn — the host wakes a waiting thread when it gets round to it — with the count
        // orb itself reported shown late as the other half of the reading.
        let handovers = game.handovers_us();
        let rolled = &handovers[handovers_before..];
        assert!(
            rolled.len() > ROLL_FRAMES as usize / 2,
            "the roll was drawn in {} frame(s), which is not a roll that played",
            rolled.len(),
        );
        let spent = rolled.last().expect("a frame of the roll") - rolled[0];
        let apart = spent / (rolled.len() - 1) as i64;
        let refresh = game.refresh_period_us();
        assert!(
            (apart - refresh).abs() * 100 < refresh,
            "the roll's frames were {apart}us apart against a refresh of {refresh}us",
        );
        let late: Vec<String> = game
            .log()
            .lines()
            .iter()
            .filter(|line| line.contains(" shown late") && !line.contains("0 shown late"))
            .cloned()
            .collect();
        assert!(
            late.is_empty(),
            "frames of the roll were shown late:\n  {}",
            late.join("\n  ")
        );
    });
}

/// The two ways of measuring an ending agree, and the arithmetic between them is what says so.
///
/// The clear above ran the ending alone at 29,040 updates. An earlier clear measured the ending and the
/// roll together at **36,932 updates** — `ending skipped, 7200 frames run, scene 10 -> 10` five times
/// and then `932 frames run, scene 10 -> 7`, 484ms of wall clock, 13µs an update, with the scene after
/// it opening the score file 47ms later. 36,932 − 29,040 = **7,892**, against the **7,830** frames of
/// waits in `staff00.end`.
///
/// **What the two ways are.** The skip stops where the script hands over, and it can only do that where
/// there is a script to compare: an ending orb finds no script in — one whose job is not in the chain —
/// is run out to the scene change instead, roll and all. That is what the earlier measurement was, the
/// skip having stopped at the scene in that build, and it is why both readings are of the same ending.
///
/// **The 62 frames between 7,892 and 7,830 are still unaccounted for**, and no laid-out game can account
/// for them: the roll ran 544 frames short of those 7,830 on that clear, the only wait in it that input
/// can cut short is one `@w1200` whose second argument is 4, and nobody was watching the keyboard for it.
/// So what is asserted here is the arithmetic over an ending whose waits are known, and the gap stays in
/// [TODO.md](../../../TODO.md) for a run against the real game to close.
#[test]
fn the_ending_and_the_roll_together_come_to_the_waits_in_the_script() {
    in_its_own_process(|| {
        // The ending alone, which is where the skip stops when it has a script to compare.
        let alone = {
            let game = clearing_into_the_ending(
                "the-ending-alone",
                |game| game.lays_out_an_ending(ENDING_UPDATES, ROLL_FRAMES),
                RAN_OUT,
            );
            updates_in(&the_line(&game, RAN_OUT))
        };
        assert_eq!(alone, ENDING_UPDATES);

        // And the same ending with nothing for orb to find the script in, which is run out to the scene
        // change: the roll goes with it, and the scene after is the result screen either way.
        let game = clearing_into_the_ending(
            "the-ending-and-the-roll",
            |game| game.lays_out_an_ending_orb_cannot_find(ENDING_UPDATES, ROLL_FRAMES),
            SKIPPED,
        );
        let line = the_line(&game, SKIPPED);
        let together = updates_in(&line);
        assert!(
            !game.log().said(RAN_OUT),
            "orb stopped at a roll it had no script to find: {line}",
        );
        assert!(
            line.contains("scene 10 -> 7"),
            "the scene the run-out ended at is not the result screen: {line}",
        );

        // The arithmetic, which is what says the two readings are of one ending: together is the ending's
        // own updates and the roll's script after them, and nothing else.
        assert_eq!(
            together - alone,
            ROLL_FRAMES,
            "the difference between the two ways of measuring is not the roll's own script: \
             {together} - {alone}",
        );
    });
}
