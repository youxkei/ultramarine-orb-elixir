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
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use orb_api::{clock, fs};
use serde::{Deserialize, Serialize};

use crate::game::{Frame, Kind, Question, State};

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
    /// **The frame as it was drawn, where one was taken on it** — a BMP, the bytes `crate::shot` makes.
    ///
    /// Which is what the fields cannot say: a hit whose dump names nothing near it is answered by what was
    /// on the screen, and a run played out over twenty minutes is answered by a picture every so often
    /// rather than by reading every frame's numbers. `Config::shot_every` is what asks for those, and a
    /// trace of a launch that asked for none carries nothing here.
    ///
    /// **Defaulted, so that a trace written before this field reads back**: what a check does with a trace
    /// is read it, and a reader that refuses last week's file is no reader.
    #[serde(default)]
    pub picture: Option<Vec<u8>>,
    /// **What stood over the player on the frame a hit was seen, where one was** — see [`OverAHit`].
    ///
    /// Defaulted for the reason [`Record::picture`] is.
    #[serde(default)]
    pub over_a_hit: Option<OverAHit>,
    /// **Every bullet on the field, on a frame of the card a launch asked for** — see [`TheField`].
    ///
    /// Defaulted for the reason [`Record::picture`] is.
    #[serde(default)]
    pub the_field: Option<TheField>,
    /// **Where the player was on the frame the rules last read**, and `None` where they read none.
    ///
    /// Which is what two runs of one stretch are held against each other by: the whole Extra and a pick-up of
    /// its chapter 7 parted at stage frame `8385` on the word pressed, the game's own numbers agreeing, and a
    /// trace with the words and the seed alone cannot say where either player went from there.
    ///
    /// Defaulted for the reason [`Record::picture`] is.
    #[serde(default)]
    pub at: Option<(f32, f32)>,
}

/// **Every bullet on the field on one frame of the card a launch named**, with the player beside them — which
/// is what a card is measured by before a way of dodging it is written: where its bullets come out, when they
/// turn to the kind that kills and where the gaps stand as they do. `Config::trace_the_field_in` names the card,
/// and a game that reads its bullets fills this; nothing here for one that does not.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TheField {
    /// The card it was read in, by its number.
    pub card: u32,
    /// Where the player was.
    pub at: (f32, f32),
    /// How far into the card the frame is, as the game counts it.
    pub card_frames: i32,
    pub bullets: Vec<ABulletOverAHit>,
    /// **And every laser**, which a card that sweeps the field with them is measured by and a bullet's box
    /// cannot stand for.
    ///
    /// Defaulted for the reason [`Record::picture`] is, a trace of th08 carrying none.
    #[serde(default)]
    pub beams: Vec<ABeamOnTheField>,
    /// **And where the boss is**, which a card whose way through follows her about is measured by — the
    /// strategy wiki's for `禁忌「レーヴァテイン」` begins `フランが左へ行くので`.
    ///
    /// Defaulted for the reason [`TheField::beams`] is.
    #[serde(default)]
    pub boss: Option<(f32, f32)>,
    /// **And every body on the field, solid or not**, which a card whose enemies come for the player is
    /// measured by — th06c's `秘弾「そして誰もいなくなるか？」` hit a run walking its edge with two bodies
    /// `37` across on it, the boss standing still at `192,80` the whole card.
    ///
    /// Defaulted for the reason [`TheField::beams`] is.
    #[serde(default)]
    pub bodies: Vec<ABodyOnTheField>,
}

/// One body of [`TheField`], as [`Frame`] hands it over.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ABodyOnTheField {
    pub slot: u32,
    pub at: (f32, f32),
    pub half: (f32, f32),
    pub solid: bool,
}

impl TheField {
    /// **The field as a bot is handed it**, for a game whose bullets have no word of their own worth tracing
    /// past what [`Frame`] already carries: each bullet's box and its state as the kind, and every beam as it
    /// goes across.
    pub fn of(frame: &Frame, card: u32, card_frames: i32) -> Self {
        let bullets = frame
            .hazards
            .iter()
            .filter_map(|hazard| match hazard.kind {
                Kind::Bullet { state, .. } => Some(ABulletOverAHit {
                    slot: hazard.slot as u32,
                    at: hazard.at,
                    size: (hazard.half.0 * 2.0, hazard.half.1 * 2.0),
                    kind: state,
                    hidden: 0,
                }),
                _ => None,
            })
            .collect();
        let beams = frame
            .beams
            .iter()
            .map(|beam| ABeamOnTheField {
                slot: beam.slot as u32,
                origin: beam.origin,
                angle: beam.angle,
                frames: beam.frames,
                near: beam.near,
                far: beam.far,
                half_across: beam.half_across,
                fires_in: beam.fires_in,
                fading: beam.fading,
            })
            .collect();
        let bodies = frame
            .hazards
            .iter()
            .filter_map(|hazard| match hazard.kind {
                Kind::Body { solid } => Some(ABodyOnTheField {
                    slot: hazard.slot as u32,
                    at: hazard.at,
                    half: hazard.half,
                    solid,
                }),
                _ => None,
            })
            .collect();
        TheField {
            card,
            at: frame.at,
            card_frames,
            bullets,
            beams,
            boss: frame.boss,
            bodies,
        }
    }
}

/// One laser of [`TheField`], as [`crate::game::Beam`] hands it over.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ABeamOnTheField {
    pub slot: u32,
    pub origin: (f32, f32),
    pub angle: f32,
    pub frames: i32,
    pub near: f32,
    pub far: f32,
    pub half_across: f32,
    pub fires_in: i32,
    pub fading: bool,
}

/// **Every enemy and every bullet within reach of the player on the frame a hit is seen**, with the words the
/// game's own tests of the player are gated on — which a hit dump cannot say, holding only what the model
/// held: a hit that came with nothing in the model is answered by what was there.
///
/// **The words are the game's own**, as its own walks read them, and each field says which of the game's
/// words it is where the game has one. Filled by a game that reads them; nothing here for one that does not.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct OverAHit {
    /// Where the player was.
    pub at: (f32, f32),
    pub enemies: Vec<AnEnemyOverAHit>,
    pub bullets: Vec<ABulletOverAHit>,
}

/// One enemy of [`OverAHit`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnEnemyOverAHit {
    pub slot: u32,
    pub at: (f32, f32),
    /// Its size whole, as the game keeps it.
    pub size: (f32, f32),
    /// The word the enemy walk's gates read.
    pub flags: u32,
    /// The word that says whether it is a familiar, nought where it is not.
    pub familiar: u32,
    /// What its script runs each frame, where it has a script.
    pub behaviour: Option<u32>,
    /// The vector a beam of its script is laid from, where it has a script.
    pub anchor: Option<(f32, f32)>,
    pub angle: f32,
}

/// One bullet of [`OverAHit`] or of [`TheField`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ABulletOverAHit {
    pub slot: u32,
    pub at: (f32, f32),
    /// Its size whole, as the game keeps it.
    pub size: (f32, f32),
    /// Which of the game's kinds it is in.
    pub kind: u16,
    /// The byte that takes it out of the player's tests while its script hides it.
    pub hidden: u8,
}

/// How many encoded bytes are held before they are appended to the file: about a few seconds of frames.
const FLUSH_AT: usize = 64 * 1024;

struct Tracing {
    path: PathBuf,
    encoded: Vec<u8>,
    lines: Vec<String>,
    word: u16,
    /// The picture taken since the last record, where one was — see [`picture`].
    picture: Option<Vec<u8>>,
    /// What stood over a hit seen since the last record, where one was — see [`over_a_hit`].
    over_a_hit: Option<OverAHit>,
    /// The field read since the last record, where the card a launch named was up — see [`the_field`].
    the_field: Option<TheField>,
    /// Where the player was when the rules last read, since the last record — see [`the_player_at`].
    at: Option<(f32, f32)>,
}

/// **The card a launch wants the field of**, `Config::trace_the_field_in`, and -1 for none — set once as the
/// trace opens, and read on every frame a bot sees, which is why it is not behind [`TRACING`]'s lock.
static THE_FIELD_IN: AtomicI64 = AtomicI64::new(-1);

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
        picture: None,
        over_a_hit: None,
        the_field: None,
        at: None,
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
/// **Keeps the frame just drawn for the record the next frame writes**, and drops it where no trace is
/// being written.
///
/// The record it lands in is the one *after* the frame the picture is of, which is the same offset
/// [`Record`]'s own doc gives for the state: a record is taken at the top of an update and holds the frame
/// before it as the game left it. So the picture and the numbers beside it are of one frame.
pub fn picture(bytes: &[u8]) {
    if !on() {
        return;
    }
    let mut held = tracing();
    if let Some(tracing) = held.as_mut() {
        tracing.picture = Some(bytes.to_vec());
    }
}

/// **Keeps what stood over a hit for the next record** — see [`OverAHit`] — and drops it where no trace is
/// being written, the way [`picture`] does.
pub fn over_a_hit(what: OverAHit) {
    if !on() {
        return;
    }
    if let Some(tracing) = tracing().as_mut() {
        tracing.over_a_hit = Some(what);
    }
}

/// Says which card's frames want the whole field, `None` for none — see [`TheField`], and [`NO_CARD`].
pub fn wants_the_field_in(card: Option<u32>) {
    THE_FIELD_IN.store(card.map_or(-1, i64::from), Ordering::Relaxed);
}

/// **The number that names the frames no card is up on**, for a launch that wants the field of a stretch that
/// is not a card — th06c's Extra's midstage, whose stream pinned the run against a wall under either place a
/// recipe held it at. `--trace-the-field-in=none`'s.
pub const NO_CARD: u32 = orb_config::args::NO_CARD;

/// **Whether the field is wanted on a frame `card` is up on**, which is what a game asks before it walks its
/// bullets for it: nothing is wanted where no trace is being written. `None` is a frame no card is up on,
/// wanted where [`NO_CARD`] was named.
pub fn the_field_is_wanted_in(card: Option<u32>) -> bool {
    on() && i64::from(card.unwrap_or(NO_CARD)) == THE_FIELD_IN.load(Ordering::Relaxed)
}

/// **Keeps the field for the next record** — see [`TheField`] — and drops it where no trace is being written.
pub fn the_field(what: TheField) {
    if !on() {
        return;
    }
    if let Some(tracing) = tracing().as_mut() {
        tracing.the_field = Some(what);
    }
}

/// **Keeps where the player was for the next record** — see [`Record::at`] — and drops it where no trace is
/// being written.
pub fn the_player_at(at: (f32, f32)) {
    if !on() {
        return;
    }
    if let Some(tracing) = tracing().as_mut() {
        tracing.at = Some(at);
    }
}

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
            picture: tracing.picture.take(),
            over_a_hit: tracing.over_a_hit.take(),
            the_field: tracing.the_field.take(),
            at: tracing.at.take(),
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
            picture: None,
            over_a_hit: None,
            the_field: None,
            at: Some((192.0, 384.0)),
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

    /// **The frame's own picture goes in and comes back out**, bytes for bytes: what a check does with a
    /// trace is read it, and a picture nothing can read back is a picture nobody took.
    #[test]
    fn the_picture_a_frame_was_drawn_as_is_read_back_whole() {
        let mut record = a_record(4);
        record.picture = Some(vec![0x42, 0x4d, 0, 1, 2, 3]);
        let bytes = rmp_serde::to_vec_named(&record).unwrap();
        assert_eq!(read(&bytes).unwrap(), vec![record]);
    }

    /// **And a trace written before there was a picture to hold reads back with none**, which is what the
    /// field's own default is for: a reader that refuses last week's file is no reader. The record here is
    /// a map with every field of one but that, which is what those traces hold.
    #[test]
    fn a_trace_from_before_the_picture_reads_back_with_none() {
        let without = a_record(5);
        use rmpv::Value;
        let written = rmp_serde::to_vec_named(&without).unwrap();
        let fields = match rmpv::decode::read_value(&mut written.as_slice()).expect("a map") {
            Value::Map(fields) => fields,
            other => panic!("a record is a map and this is {other}"),
        };
        let was = fields.len();
        let fields: Vec<_> = fields
            .into_iter()
            .filter(|(name, _)| name.as_str() != Some("picture"))
            .collect();
        assert_eq!(fields.len() + 1, was, "the field was not written");
        let mut bytes = Vec::new();
        rmpv::encode::write_value(&mut bytes, &Value::Map(fields)).expect("written");
        assert_eq!(read(&bytes).unwrap(), vec![without]);
    }
}
