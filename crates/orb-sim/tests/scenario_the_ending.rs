//! **The ending run out inside one frame, and its staff roll left to play.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine, so that
//! writing the scenario is a matter of making the laid-out game reach the same numbers rather than
//! deciding what the numbers are.
//!
//! What it takes to un-stub them: the fake 紅魔郷 has no ending. It needs the ending's script — a `.end`
//! of one-character instructions — the scene the roll plays in, and the two signals the boundary is read
//! off, which are the script handing over and the track changing on the same update.
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

/// The ending runs out inside the frame it begins on, and stops where it hands over to the roll.
///
/// Measured on a stage 6 clear: **29,040 updates inside the frame it began on**, stopping at
/// `ending run out in 29040 update(s), where its staff roll begins, track Some(1727006158) ->
/// Some(3570673472)` — the script and the track changing on the same update, which is the two signals
/// agreeing on the boundary. Nothing of the ending reached the screen.
#[test]
#[ignore = "the fake 紅魔郷 has no ending script to run out"]
fn the_ending_runs_out_inside_one_frame_and_stops_at_the_roll() {
    todo!(
        "lay out an ending script and its track, run the frame it begins on, and assert 29040 updates \
         and the script and track changing together"
    )
}

/// The roll plays on its own afterwards, at the rate everything else is paced at.
///
/// Measured over the same clear: **7,286 drawn frames over 122.0 seconds**, 16.74ms each, with
/// `0 shown late` and the audio never behind, and the scene after it was 7, the result screen.
#[test]
#[ignore = "the fake 紅魔郷 has no staff roll to play"]
fn the_staff_roll_plays_at_sixty_and_the_result_screen_follows_it() {
    todo!("run the roll and assert its frames, its rate and the scene that follows")
}

/// The two ways of measuring an ending agree, and the arithmetic between them is what says so.
///
/// The clear above ran the ending alone at 29,040 updates. An earlier clear measured the ending and the
/// roll together at **36,932 updates** — `ending skipped, 7200 frames run, scene 10 -> 10` five times
/// and then `932 frames run, scene 10 -> 7`, 484ms of wall clock, 13µs an update, with the scene after
/// it opening the score file 47ms later. 36,932 − 29,040 = **7,892**, against the **7,830** frames of
/// waits in `staff00.end`.
///
/// **Two things that measurement leaves open**, and they are why this is three scenarios rather than
/// one: the roll ran 544 frames short of those 7,830, and the only wait in it that input can cut short
/// is one `@w1200` whose second argument is 4, which nobody was watching the keyboard for. And the
/// frame the skip runs in is only known to have taken 5 or more refreshes, which is where the log
/// line's buckets stop.
#[test]
#[ignore = "the 62-frame gap between 7,892 and 7,830 is unaccounted for"]
fn the_ending_and_the_roll_together_come_to_the_waits_in_the_script() {
    todo!(
        "run an ending and its roll in one pass, assert 36932 against 29040 plus staff00.end's 7830, \
         and account for the 62 frames between 7892 and 7830"
    )
}
