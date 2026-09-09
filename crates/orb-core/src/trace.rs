//! **What every frame held, written down for a check after the run.**
//!
//! A run on the real game is the one measurement nothing in the tree can repeat: it takes the game, the
//! machine and the minutes, and what is left of it afterwards is `orb.log`. A log is written for a person
//! reading it, so a check written against one is a check written against the wording of its lines — which
//! is what `docs/todo/` used to hold as recipes for reading a log by hand. This is the other record: one
//! [`Record`] per frame, holding what the game's own memory said, the word the run was played with, which of
//! orb's own questions was up, and every line the log said in between — as MessagePack, the same encoding
//! as the file a run is left in, so that a check reads fields and not sentences.
//!
//! **Every frame and not the ones something happened on**, because what a check asks is as often about the
//! frames between two events as about the events: that the chapter did not change while a question stood,
//! that the deaths did not climb during a restore, that the track was the same one every frame after the
//! retry as before it. A record per frame answers all of those; a record per event answers none.
//!
//! **The log's lines are in it too**, and not instead of the fields. A line is where orb says what it
//! *decided* — which item answered a question, where a resume landed, what a recipe's step reached — and
//! the game's memory says none of that. Carrying them per frame is also what puts each beside the state it
//! was said in, where the log has only a stamp.
//!
//! **Written from the frame's own thread, in chunks, and that is a cost this instrument accepts.** The log
//! hands its lines to a thread of its own because a `WriteFile` can take what a disk decides, and the
//! files a run keeps go the same way — see [`crate::log`] and [`crate::writer`]. A trace is asked for by a
//! launch that is being *checked* rather than played, and what its check reads is the sequence, so a hitch
//! in the frames around a chunk's write costs it nothing it asks about. One append every [`FLUSH_AT`] bytes,
//! which at a few hundred bytes a frame is once every few seconds.
//!
//! **Off unless `--trace` names the file**, and nothing here costs a frame anything while it is off: a load
//! of one flag on the log's path and on the input read's, and no lock taken.
//!
//! **The reader is here too**, [`read`], so that the check over a trace of the real game and the e2e test
//! over a trace of a laid-out one read the same bytes the same way.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use orb_api::{clock, fs};
use serde::{Deserialize, Serialize};

use crate::game::{Question, State};

/// One frame of a launch, as orb's update saw it.
///
/// **What it holds is what was there when the update began**, which is the frame before it as the game left
/// it: the state is the one the last update produced, the word is the one the last input read handed the
/// run, and the lines are what was said since the last record. Taken at the top of the update rather than
/// the bottom because orb's update leaves by several doors — a question up, a chapter being restored — and
/// a record at the top is one record however the frame went.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Record {
    /// orb's own count of frames since it attached.
    pub frame: u32,
    /// The host's clock as the record was taken, in milliseconds — the same stamp the log's lines carry,
    /// so that a line in the log can be put beside its frame here.
    pub ms: u32,
    /// What the game's memory said after the last update, and `None` before orb has read it once.
    pub state: Option<State>,
    /// The chapter the run is in, counted the way the log counts it, and nought outside a run.
    pub chapter: u32,
    /// How many times the chapter has been retried.
    pub retries: u32,
    /// Which of orb's own questions is up, by the name a recipe uses for it — `mode`, `resume`, `retry`,
    /// `keep_replay`, `choose_replay` — and `None` where the game has the screen.
    pub question: Option<String>,
    /// The word the input read last handed the run, the game's own buttons in its own bits.
    pub word: u16,
    /// Every line the log said since the last record, without its stamp: the message as `log!` was given
    /// it.
    pub lines: Vec<String>,
}

/// How many encoded bytes are held before they are appended to the file: about a few seconds of frames.
const FLUSH_AT: usize = 64 * 1024;

struct Tracing {
    path: PathBuf,
    encoded: Vec<u8>,
    lines: Vec<String>,
    word: u16,
}

static TRACING: Mutex<Option<Tracing>> = Mutex::new(None);
/// Whether a trace is being written, for the two places on a frame's path that would otherwise take the
/// lock to find out.
static ON: AtomicBool = AtomicBool::new(false);

fn tracing() -> std::sync::MutexGuard<'static, Option<Tracing>> {
    TRACING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Starts a trace at `path`, over whatever was there: a trace is one launch's, and a check reading a file
/// two launches appended to would be reading two runs as one.
///
/// A path that cannot be started is a launch with no trace, said in the log and carried on with: the run is
/// the thing being checked and a trace that could not be opened is a finding about the machine, not a reason
/// to stop the run.
pub fn open(path: &Path) {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(why) = fs::create_dir_all(parent)
    {
        crate::log!(
            "trace: {} cannot be made, so nothing is traced: {why}",
            parent.display()
        );
        return;
    }
    if let Err(why) = fs::write(path, &[]) {
        crate::log!(
            "trace: {} cannot be started, so nothing is traced: {why}",
            path.display()
        );
        return;
    }
    *tracing() = Some(Tracing {
        path: path.to_path_buf(),
        encoded: Vec::new(),
        lines: Vec::new(),
        word: 0,
    });
    ON.store(true, Ordering::Release);
    crate::log!("trace: every frame written to {}", path.display());
}

/// Whether a trace is being written.
pub fn on() -> bool {
    ON.load(Ordering::Acquire)
}

/// A line the log said, kept for the next record. Called by [`crate::log::line`] with the message as it
/// was given, and before the log has taken any lock of its own.
pub fn said(message: &str) {
    if !on() {
        return;
    }
    if let Some(tracing) = tracing().as_mut() {
        tracing.lines.push(message.to_owned());
    }
}

/// The word the input read handed the run, kept for the next record.
pub fn the_word(word: u16) {
    if !on() {
        return;
    }
    if let Some(tracing) = tracing().as_mut() {
        tracing.word = word;
    }
}

/// One record, taken at the top of orb's update with what the frame before left.
pub fn frame(
    frame: u32,
    state: Option<State>,
    chapter: u32,
    retries: u32,
    question: Option<Question>,
) {
    if !on() {
        return;
    }
    let failed = {
        let mut held = tracing();
        let Some(tracing) = held.as_mut() else {
            return;
        };
        let record = Record {
            frame,
            ms: clock::ticks(),
            state,
            chapter,
            retries,
            question: question.map(|question| question.name().to_owned()),
            word: tracing.word,
            lines: std::mem::take(&mut tracing.lines),
        };
        // Named, so that any MessagePack reader prints a record back without being told the shape —
        // the same choice `resume` made for the file a run is left in.
        match rmp_serde::to_vec_named(&record) {
            Ok(bytes) => tracing.encoded.extend_from_slice(&bytes),
            Err(why) => {
                crate::log!("trace: frame {frame} could not be encoded: {why}");
            }
        }
        if tracing.encoded.len() >= FLUSH_AT {
            flush(tracing)
        } else {
            None
        }
    };
    // Said with the lock let go of: the log's line comes back here through `said`.
    if let Some(why) = failed {
        crate::log!("trace: a chunk could not be written, and the trace is short of it: {why}");
    }
}

/// Appends what is held and empties it, answering what went wrong where the append failed.
fn flush(tracing: &mut Tracing) -> Option<String> {
    if tracing.encoded.is_empty() {
        return None;
    }
    let bytes = std::mem::take(&mut tracing.encoded);
    fs::append(&tracing.path, &bytes)
        .err()
        .map(|why| why.to_string())
}

/// The last chunk written and the trace closed. Called as orb detaches, which is the last moment there is.
pub fn close() {
    let failed = {
        let mut held = tracing();
        let Some(tracing) = held.as_mut() else {
            return;
        };
        let failed = flush(tracing);
        *held = None;
        ON.store(false, Ordering::Release);
        failed
    };
    if let Some(why) = failed {
        crate::log!("trace: the last chunk could not be written: {why}");
    }
}

/// Every record in a trace, in the order they were taken.
///
/// **Refuses the whole file on a record it cannot read**, rather than answering the ones before it: a
/// trace that ends mid-record is a launch that was killed with a chunk in memory, and a check that passed
/// over the frames it did have would be passing a run whose end nobody saw. The error names how many
/// records were read before it.
pub fn read(bytes: &[u8]) -> Result<Vec<Record>, String> {
    let mut records = Vec::new();
    let mut cursor = Cursor::new(bytes);
    while (cursor.position() as usize) < bytes.len() {
        let mut reader = rmp_serde::Deserializer::new(&mut cursor);
        match Record::deserialize(&mut reader) {
            Ok(record) => records.push(record),
            Err(why) => {
                return Err(format!(
                    "record {} cannot be read: {why}",
                    records.len() + 1
                ));
            }
        }
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_record(frame: u32) -> Record {
        Record {
            frame,
            ms: frame * 16,
            state: None,
            chapter: 3,
            retries: 1,
            question: Some("retry".to_owned()),
            word: 0x0011,
            lines: vec![format!("frame {frame} said this")],
        }
    }

    #[test]
    fn records_written_one_after_another_are_read_back_in_order() {
        let mut bytes = Vec::new();
        for frame in 1..=3 {
            bytes.extend(rmp_serde::to_vec_named(&a_record(frame)).unwrap());
        }
        let records = read(&bytes).unwrap();
        assert_eq!(records, vec![a_record(1), a_record(2), a_record(3)]);
    }

    #[test]
    fn a_trace_cut_short_inside_a_record_is_refused_and_says_where() {
        let mut bytes = rmp_serde::to_vec_named(&a_record(1)).unwrap();
        let whole = rmp_serde::to_vec_named(&a_record(2)).unwrap();
        bytes.extend(&whole[..whole.len() / 2]);
        let refused = read(&bytes).unwrap_err();
        assert!(
            refused.starts_with("record 2 cannot be read"),
            "the refusal does not name the record: {refused}"
        );
    }

    #[test]
    fn an_empty_trace_is_no_records() {
        assert_eq!(read(&[]).unwrap(), Vec::<Record>::new());
    }
}
