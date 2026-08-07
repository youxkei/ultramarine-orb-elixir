//! **The music put back where a chapter had it, and the stream believing the file it was given.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this machine — two of them
//! by ear, which is why they are here rather than asserted anywhere.
//!
//! What it takes to un-stub them: the laid-out 紅魔郷 has no track. `scenario_pointdevice_run.rs` takes its
//! first chapter at frame 248, `STAGE_SETTLE_FRAMES` plus the whole of `MUSIC_WAIT_FRAMES`, precisely
//! because there is no track for the snapshot to wait for. So these want a streaming sound with a `data`
//! chunk, a loop point, and the countdown `WaveFile::Read` keeps — enough of one that seeking it can be
//! got wrong.

/// A chapter's song position comes back with the chapter, and it is asked of the song rather than the kind.
///
/// Measured over a resumed run: the song was left at the track's opening milliseconds with the position
/// that had been written down (**`song=5900628`**) never read — the restore was asked of the chapter's
/// *kind*, and a midboss is not one of the midstage kinds although the stage's own song plays through it.
/// It is asked of the song now, from the same place a retry asks it, so the two cannot disagree about a
/// chapter.
#[test]
#[ignore = "the laid-out 紅魔郷 has no track for a chapter to hold a position in"]
fn a_chapters_song_position_is_asked_of_the_song_and_not_of_the_chapters_kind() {
    todo!(
        "snapshot a midboss chapter with the stage's song under it and assert the position comes back"
    )
}

/// Seeking the file moves the countdown with it, or the stream reads past the end and takes no loop.
///
/// Heard once near the end of a resumed stage 1's midstage and once near the end of a resumed stage 2's: a
/// section of the track repeating. Read off the game afterwards — `WaveFile::Read` (**0x43c080**) clamps
/// every read to `m_ck.cksize` (**0x43c1aa**) and subtracts what it read from it (**0x43c1be**), and
/// `StreamingSound::ServiceBuffer` takes the track's loop only where a read comes up *short against that
/// countdown* (**0x43b759** → `ResetFile(TRUE)` at **0x43b76f**). The file's own end is never asked.
///
/// So seeking the file **5,900,628** bytes forward and leaving the countdown alone left the stream
/// believing it had that much more sound than the file held: it read past the end of the `data` chunk,
/// where `Read` fails rather than returning short — `mmioAdvance` leaves nothing to copy (**0x43c233**) —
/// so no loop was taken and the buffer went round its own contents. It came right by itself once the
/// skipped bytes had been counted off, which is why it was one episode and not a hang.
///
/// The countdown is moved with the file now, to the loop point less where the file ends up, and the loop
/// point is taken as the position plus the countdown as they stand — the pair the game reads, rather than
/// the header's loop fields, since a track without a loop end runs on `cksize` instead. **Not re-heard.**
#[test]
#[ignore = "not re-heard since the countdown was moved with the file"]
fn a_sought_stream_keeps_its_countdown_and_still_takes_its_loop() {
    todo!(
        "seek a stream past a loop point and assert the countdown and the loop point move together"
    )
}

/// The track is rewound for the chapters that share the stage's theme and left playing through a boss's.
///
/// Verified by ear over a full 1→6 run: rewound for the midstage and the midboss, which share the stage
/// theme, and left playing through a boss-fight restore.
///
/// And where the track has changed since the snapshot, it is asked to start again rather than copied back —
/// measured on a stage 6 run: `restore: the track has changed since this snapshot; taking the music down`,
/// `music: stopped through the game`, `music: restarting bgm/th06_12.mid`, with `StopBGM` and then
/// `PlayAudio` given the path read out of the memory the restore had just put back.
#[test]
#[ignore = "the laid-out 紅魔郷 has no track to rewind or leave playing"]
fn the_track_is_rewound_for_the_chapters_that_share_it_and_left_alone_for_a_boss() {
    todo!("restore a midstage, a midboss and a boss chapter and assert which of the three rewound")
}
