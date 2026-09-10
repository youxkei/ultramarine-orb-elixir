//! **The file a run of th08 left unfinished is written in, which is this game's own.**
//!
//! Its own for the reason th06c's and th07's are: a file is the game's, and one version number over two
//! games' numbers is what let a field go into th06's file without being raised — see `th06::resume`.
//!
//! **What a stage of this game is restarted from is its own replay record**, which is what `CLAUDE.md`
//! asks a resume be shaped from: 0x452b5f through 0x452ce6 fills the record's `0x24`-byte header out of
//! the run as a stage ends, and 0x452ee1 through 0x452fe5 puts it back as a replay plays that stage
//! again. Fourteen fields go back, and `asm.md`'s *The replay* has the table with every address in it.
//!
//! **Four of the fourteen have no name of the game's**, and each is written here by where it is:
//! `[run+0x22]`, `[run+0x1c]`, `0x164d334` and `0x164d0b0`. Guessing a name for one of those would be
//! the one thing a resume must not do — a field put back into the wrong place is a run that plays out
//! differently and says nothing about why — so they are carried by address and nothing is claimed about
//! them.
//!
//! **And two more that the record carries and the restore does not**: the score and the deaths. A replay
//! plays the buttons in from the stage's first frame and arrives at the score by playing, where orb
//! builds the stage and writes the numbers, so a resume that left the score out would come back with the
//! score of a run that had not been played. The record holds the score at `+0x0` — written as the stage
//! *ends*, which is what the replay's own menu shows — and the deaths nowhere at all.
//!
//! **The generator of every frame is not optional here**, for th07's reason: a file without them is a run
//! nothing can be held to. Version 1 means they are there.
//!
//! What the machinery in [`crate::resume`] asks of this is two things: the bytes to write, and a
//! [`Kept`](crate::resume::Kept) out of the bytes it read.

use std::collections::BTreeMap;

use crate::game::RunStart;
use crate::resume::{Again, Kept, MAX_STAGE_FRAMES};

/// **The numbers a stage of th08 starts with, which is what a stage of it needs to be played again.**
///
/// The fourteen the game's own per-stage replay record puts back, plus the two the record leaves behind —
/// see the module's own doc for which are which, and `asm.md` for the address each was read at.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Numbers {
    /// **orb's own and not the record's**: the score as the stage began, where the record keeps the score
    /// the stage *ended* on and the restore puts none back.
    pub score: u32,
    /// What the generator was seeded with as the stage began, which the record keeps at `+0x1a` and
    /// 0x452f94 writes back through 0x453be0. The whole of why the buttons alone are not enough: the game
    /// takes this from wherever the generator had got to, so a stage played again from a different seed
    /// is a different stage.
    pub seed: u16,
    /// How many point items the run has taken — `[run+0x30]`.
    pub point_items: u32,
    /// The graze — `[run+0xc]`.
    pub graze: u32,
    /// Which step of the night's own table of point-item counts the run is on — `[run+0x34]`, which
    /// 0x440495 indexes the table at `0x4c7a84` by.
    pub points_step: u32,
    /// And how many the night asks for at that step — `[run+0x38]`.
    pub points_wanted: u32,
    /// What one point item is worth — `[run+0x24]`, which the item's own collect at 0x440e50 reads.
    pub point_worth: u32,
    /// The power, the lives and the bombs, which the restore writes through the game manager's own three
    /// setters rather than into the run: 0x406fa0, 0x406e20 and 0x406f30.
    pub power: u16,
    pub lives: i8,
    pub bombs: i8,
    /// `[run+0x22]`, a word the record keeps at `+0x18` and 0x452fd4 puts back. Nothing is read of what it
    /// means.
    pub the_word_at_0x22: u16,
    /// `[run+0x1c]`, a byte the record keeps at `+0x21` and 0x452fa6 puts back. Nothing is read of what it
    /// means.
    pub the_byte_at_0x1c: u8,
    /// `[run+0x28]`, a byte the record keeps at `+0x22` and 0x452fe5 puts back through 0x453c60 — the one
    /// 0x439df7 compares against `0xc` to choose which of the run's two endings it is.
    pub the_byte_at_0x28: i8,
    /// `0x164d334`, a byte the record keeps at `+0x1f` and 0x452ef1 puts back. Nothing is read of what it
    /// means.
    pub the_byte_at_164d334: u8,
    /// `0x164d0b0`, a byte the record keeps at `+0x20` and 0x452f59 puts back. Nothing is read of what it
    /// means; it sits immediately below the character at `0x164d0b1`.
    pub the_byte_at_164d0b0: u8,
    /// **orb's own and not the record's**: how many lives the run has lost, which this game keeps as
    /// `[run+0x64]` and no replay puts back.
    pub deaths: i32,
}

/// Those numbers as the bytes the machinery carries about — see `Game::run_state`, which is where the
/// shape stops being anybody else's business.
pub fn encode_state(numbers: &Numbers) -> Vec<u8> {
    rmp_serde::to_vec_named(numbers).unwrap_or_default()
}

/// And back. `None` for bytes another game wrote, which is a file put in the wrong game's directory.
pub fn decode_state(bytes: &[u8]) -> Option<Numbers> {
    rmp_serde::from_slice(bytes).ok()
}

/// What this file's fields mean, and what a file carrying another number is refused for.
///
/// **Two, and th08's own.** What tells this game's file from another's is `Game::named` in the directory
/// each is kept in, and not this number.
///
/// **One is refused rather than read without its `again`.** A version 1 file of this game holds one read
/// per frame, and on this game's dialogue that is a hundred updates of a run written down as one — see
/// `resume::Again`. Read with an empty `again`, such a file plays back exactly as it did before this was
/// kept and nothing in it says why, which is the trade `generators` was already caught in once.
const VERSION: u32 = 2;

/// The file, whole. Written every time rather than appended to: it describes one chapter of one run, and
/// the one it describes is the newest.
pub fn encode(saved: &Kept) -> Vec<u8> {
    // The numbers are this game's own bytes by the time they get here — see [`decode_state`]. Nothing but
    // a file of another game's could fail to read, and one of those is a file in the wrong directory.
    let numbers = decode_state(&saved.state).unwrap_or(EMPTY);
    let written = Written {
        version: VERSION,
        run: WrittenRun {
            difficulty: saved.run.difficulty,
            character: saved.run.character,
            shot: saved.run.shot_type,
            stage: saved.run.stage,
        },
        chapter: WrittenChapter {
            number: saved.chapter,
            frame: saved.at,
            retries: saved.retries,
            name: saved.name.clone(),
            song: saved.song,
            song_identity: saved.song_identity,
            song_sounding: saved.song_sounding,
        },
        start: WrittenStart {
            score: numbers.score,
            seed: numbers.seed,
            points: numbers.point_items,
            graze: numbers.graze,
            points_step: numbers.points_step,
            points_wanted: numbers.points_wanted,
            point_worth: numbers.point_worth,
            power: numbers.power,
            lives: numbers.lives,
            bombs: numbers.bombs,
            word_at_0x22: numbers.the_word_at_0x22,
            byte_at_0x1c: numbers.the_byte_at_0x1c,
            byte_at_0x28: numbers.the_byte_at_0x28,
            byte_at_164d334: numbers.the_byte_at_164d334,
            byte_at_164d0b0: numbers.the_byte_at_164d0b0,
            deaths: numbers.deaths,
        },
        landing: saved.landing.clone(),
        buttons: changes(&saved.buttons),
        generators: saved.generators.clone(),
        again: saved.again.clone(),
    };
    // With the names in it, so that the file says what its own fields are — see `resume`'s own
    // `EXTENSION`. Nothing in it can fail to serialize, so a failure here is a file that would say
    // nothing about the run, which is what no bytes read as.
    rmp_serde::to_vec_named(&written).unwrap_or_default()
}

/// A run of nothing, for the one case [`encode`] cannot read the numbers it was handed.
const EMPTY: Numbers = Numbers {
    score: 0,
    seed: 0,
    point_items: 0,
    graze: 0,
    points_step: 0,
    points_wanted: 0,
    point_worth: 0,
    power: 0,
    lives: 0,
    bombs: 0,
    the_word_at_0x22: 0,
    the_byte_at_0x1c: 0,
    the_byte_at_0x28: 0,
    the_byte_at_164d334: 0,
    the_byte_at_164d0b0: 0,
    deaths: 0,
};

/// Reads it back, and `None` for anything that is not a file this version wrote: a field missing or
/// unknown, a number that is not one, a version that is not this one, or a generator missing for a frame.
pub fn decode(bytes: &[u8]) -> Option<Kept> {
    let written: Written = rmp_serde::from_slice(bytes).ok()?;
    if written.version != VERSION {
        return None;
    }
    let changes = written.buttons;
    let at = written.chapter.frame;
    // Nothing at or past the frame the buttons are meant to reach, that being a record which does not
    // describe the chapter beside it. They are in order already, a map of frames being kept that way.
    if at > MAX_STAGE_FRAMES || changes.keys().last().is_some_and(|frame| *frame >= at) {
        return None;
    }
    // One entry per frame the buttons reach, and no fewer: the playback holds the two together by frame,
    // so a list of another length is one whose frames do not line up.
    if written.generators.len() != at as usize {
        return None;
    }
    // And the same bound on the reads before a frame's last: one against a frame the chapter does not reach
    // is a read nothing would ever ask for, which is a file describing another chapter.
    if written.again.iter().any(|again| again.frame >= at) {
        return None;
    }
    Some(Kept {
        run: RunStart {
            difficulty: written.run.difficulty,
            character: written.run.character,
            shot_type: written.run.shot,
            // Not in the file, there being nothing to say: a practice run has no slot to be written in —
            // see `Game::run_slot` — so every file is a full run's.
            practice: false,
            stage: written.run.stage,
            // Not in the file either: this game has one mode of its own to start a run in — see
            // `RunStart::mode`.
            mode: 0,
        },
        state: encode_state(&Numbers {
            score: written.start.score,
            seed: written.start.seed,
            point_items: written.start.points,
            graze: written.start.graze,
            points_step: written.start.points_step,
            points_wanted: written.start.points_wanted,
            point_worth: written.start.point_worth,
            power: written.start.power,
            lives: written.start.lives,
            bombs: written.start.bombs,
            the_word_at_0x22: written.start.word_at_0x22,
            the_byte_at_0x1c: written.start.byte_at_0x1c,
            the_byte_at_0x28: written.start.byte_at_0x28,
            the_byte_at_164d334: written.start.byte_at_164d334,
            the_byte_at_164d0b0: written.start.byte_at_164d0b0,
            deaths: written.start.deaths,
        }),
        at,
        chapter: written.chapter.number,
        name: written.chapter.name,
        retries: written.chapter.retries,
        song: written.chapter.song,
        song_identity: written.chapter.song_identity,
        song_sounding: written.chapter.song_sounding,
        landing: written.landing,
        buttons: dense(&changes, at),
        generators: written.generators,
        again: written.again,
    })
}

/// The file's own shape, which is not the game's structs: the names in it are the file's business, so that
/// a field renamed in [`RunStart`] is not a file that reads differently afterwards.
///
/// `deny_unknown_fields` throughout, because a field nothing knows is a file from another version or
/// another program, and reading one of those halfway is worse than not reading it.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    version: u32,
    run: WrittenRun,
    chapter: WrittenChapter,
    start: WrittenStart,
    /// The reproduction line exactly as the log prints it, kept as the one string it is: a landing is held
    /// against it by field name, so what is compared is this text either way, and the file shows the line
    /// somebody can hold against the log by eye. See `Game::reproduction` for what changing the line
    /// costs.
    landing: String,
    /// The buttons held from each frame they changed on, keyed by that frame — only the changes, which is
    /// how this game writes its own record too: the input buffer at `record + 0x24` is walked a word at a
    /// time.
    buttons: BTreeMap<u32, u16>,
    /// The generator's own state on every one of those frames, which is what says whether a playback is
    /// still reproducing the run — and not optional, for the reason the module doc gives.
    generators: Vec<Option<u16>>,
    /// **And every read of a frame before its last**, which on this game is what a conversation is made of:
    /// the run clock is put on 1,800 by the instruction that starts one and then stands still while the
    /// line is up, so the frames the run plays through it are a hundred reads against one number — see
    /// `resume::Again`, whose doc has the launch, and `Th08`'s own `STAGE_FRAMES`.
    ///
    /// Written whole rather than as changes: there are a few hundred of these against a stage's fifteen
    /// thousand frames, and what a change map saves on a list that short is less than the frame numbers it
    /// would still have to carry.
    again: Vec<Again>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenRun {
    difficulty: i32,
    character: i32,
    shot: i32,
    stage: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenChapter {
    number: u32,
    frame: u32,
    retries: u32,
    name: String,
    /// Where the song was, which track it was in, and whether it was coming out — see
    /// [`crate::resume::Kept::song`] and its two neighbours. `null` where the game would not say, which
    /// for this game is every chapter until its music is read: the three are kept so that the file says
    /// the same things every other game's does.
    song: Option<i32>,
    song_identity: Option<u32>,
    song_sounding: Option<bool>,
}

/// The numbers a stage of this game starts with — see [`Numbers`].
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenStart {
    score: u32,
    #[serde(with = "hex")]
    seed: u16,
    points: u32,
    graze: u32,
    points_step: u32,
    points_wanted: u32,
    point_worth: u32,
    power: u16,
    lives: i8,
    bombs: i8,
    word_at_0x22: u16,
    byte_at_0x1c: u8,
    byte_at_0x28: i8,
    byte_at_164d334: u8,
    byte_at_164d0b0: u8,
    deaths: i32,
}

/// A number written the way every line of the log that mentions this one writes it. The seed is the
/// generator's state rather than a quantity, and `0x1a2b` is what it is called everywhere else.
mod hex {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(value: &u16, into: S) -> Result<S::Ok, S::Error> {
        into.serialize_str(&format!("{value:#06x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<u16, D::Error> {
        let text = String::deserialize(from)?;
        let text = text.trim();
        let value = match text.strip_prefix("0x") {
            Some(hex) => i64::from_str_radix(hex, 16).ok(),
            None => text.parse().ok(),
        };
        value
            .and_then(|value| u16::try_from(value).ok())
            .ok_or_else(|| D::Error::custom(format!("not a number: {text}")))
    }
}

/// The frames the buttons changed on, out of one entry per frame: everything between two changes is the
/// first one's, which is what a run with a key held down is.
fn changes(buttons: &[u16]) -> BTreeMap<u32, u16> {
    let mut changes = BTreeMap::new();
    let mut held = 0;
    for (frame, pressed) in buttons.iter().enumerate() {
        if *pressed != held {
            changes.insert(frame as u32, *pressed);
            held = *pressed;
        }
    }
    changes
}

/// One entry per frame again, out of the frames the buttons changed on.
fn dense(changes: &BTreeMap<u32, u16>, upto: u32) -> Vec<u16> {
    let mut buttons = vec![0; upto as usize];
    let mut held = 0;
    for frame in 0..upto {
        if let Some(pressed) = changes.get(&frame) {
            held = *pressed;
        }
        buttons[frame as usize] = held;
    }
    buttons
}

#[cfg(test)]
mod tests {
    use super::{Numbers, VERSION, decode, encode, encode_state};
    use crate::game::RunStart;
    use crate::resume::{Again, Kept, MAX_STAGE_FRAMES};

    /// A chapter of a run of this game, with a different number in every field: a reader that takes one
    /// field for another is a run put back with somebody else's numbers, and nothing about the file would
    /// say so.
    fn saved() -> Kept {
        Kept {
            run: RunStart {
                difficulty: 3,
                character: 1,
                shot_type: 0,
                practice: false,
                stage: 3,
                mode: 0,
            },
            state: encode_state(&Numbers {
                score: 12_345_670,
                seed: 0x1a2b,
                point_items: 321,
                graze: 4_567,
                points_step: 2,
                points_wanted: 500,
                point_worth: 12_340,
                power: 128,
                lives: 4,
                bombs: 2,
                the_word_at_0x22: 0x0708,
                the_byte_at_0x1c: 9,
                the_byte_at_0x28: -3,
                the_byte_at_164d334: 11,
                the_byte_at_164d0b0: 13,
                deaths: 3,
            }),
            at: 12,
            chapter: 5,
            name: "MIDBOSS SPELL 1".to_owned(),
            retries: 9,
            song: None,
            song_identity: None,
            song_sounding: None,
            landing: "frames=4472 seed=0x3274 score=12345670 lives=4".to_owned(),
            buttons: vec![0, 0, 1, 1, 1, 0x45, 0x45, 0, 0, 0, 0x105, 0x105],
            // One per frame like the buttons, and every one different, so that a list that survived the
            // round trip in the wrong order reads as a playback parting on its first frame; and one frame
            // with no word, which is a frame a playback fed rather than read a keyboard for.
            generators: vec![
                Some(0x1000),
                Some(0x1001),
                None,
                Some(0x1003),
                Some(0x1004),
                Some(0x1005),
                Some(0x1006),
                Some(0x1007),
                Some(0x1008),
                Some(0x1009),
                Some(0x100a),
                Some(0x100b),
            ],
            // **Three reads of frame 4 and two of frame 9**, out of order in the list on purpose: the
            // file is what the playback searches by frame, so a round trip that kept them in the order
            // written would say nothing about a file whose own order is the run's.
            again: vec![
                Again {
                    frame: 4,
                    buttons: 0x45,
                    generator: 0x2000,
                },
                Again {
                    frame: 4,
                    buttons: 0x41,
                    generator: 0x2001,
                },
                Again {
                    frame: 9,
                    buttons: 0x105,
                    generator: 0x2002,
                },
                Again {
                    frame: 4,
                    buttons: 0x40,
                    generator: 0x2003,
                },
                Again {
                    frame: 9,
                    buttons: 0x101,
                    generator: 0x2004,
                },
            ],
        }
    }

    /// Every field of it, since a resume is a run started from these numbers: one read back as another is
    /// a run that plays out differently and says nothing about why.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&saved())), Some(saved()));
    }

    /// A file with no generators in it is refused, which is what this version is for: a run whose frames
    /// have no word to be held against is a run nothing can say it is reproducing.
    #[test]
    fn a_file_with_no_generators_is_refused() {
        let mut older = saved();
        older.generators = Vec::new();
        assert!(decode(&encode(&older)).is_none());
    }

    /// And one with some of them, which is a file whose frames do not line up: the buttons of a frame and
    /// the generator it was pressed with are held together by their index.
    #[test]
    fn a_file_with_some_of_the_generators_is_refused() {
        let mut half = saved();
        half.generators.truncate(half.buttons.len() - 1);
        assert!(decode(&encode(&half)).is_none());
    }

    /// A chapter at the stage's own start has no buttons under it, which is a run resumed by building the
    /// stage and playing nothing — and no generators either, there being no frame to have one, nor any
    /// read before a frame's last.
    #[test]
    fn a_stage_start_holds_no_buttons() {
        let mut start = saved();
        start.at = 0;
        start.chapter = 1;
        start.name = "MIDSTAGE 1".to_owned();
        start.buttons = Vec::new();
        start.generators = Vec::new();
        start.again = Vec::new();
        assert_eq!(decode(&encode(&start)), Some(start));
    }

    /// And a read against a frame the chapter does not reach is a file describing another chapter, which
    /// is what the buttons' own bound says about the same file.
    #[test]
    fn a_read_past_the_chapters_own_frame_is_refused() {
        let mut past = saved();
        past.again.push(Again {
            frame: past.at,
            buttons: 0x11,
            generator: 0x3000,
        });
        assert!(decode(&encode(&past)).is_none());
    }

    /// A frame number no stage reaches is not a file to allocate for, whatever asked.
    #[test]
    fn a_frame_no_stage_reaches_is_refused() {
        let mut far = saved();
        far.at = MAX_STAGE_FRAMES + 1;
        assert!(decode(&encode(&far)).is_none());
    }

    /// A file this version did not write is refused rather than read with its fields taken for each other,
    /// and so is one that has lost a field, gained one nothing knows, or holds a field of another kind
    /// than it should.
    #[test]
    fn a_file_this_version_did_not_write_is_refused() {
        use rmpv::Value;

        let whole = || match rmpv::decode::read_value(&mut encode(&saved()).as_slice()) {
            Ok(Value::Map(fields)) => fields,
            other => panic!("not a map: {other:?}"),
        };
        let refused = |fields: Vec<(Value, Value)>| {
            let mut bytes = Vec::new();
            rmpv::encode::write_value(&mut bytes, &Value::Map(fields)).expect("written");
            assert!(decode(&bytes).is_none());
        };
        let key = |name: &str| Value::String(name.into());

        let mut another_version = whole();
        for (name, value) in &mut another_version {
            if *name == key("version") {
                *value = Value::from(VERSION + 1);
            }
        }
        refused(another_version);

        let mut a_field_short = whole();
        a_field_short.retain(|(name, _)| *name != key("landing"));
        refused(a_field_short);

        let mut a_field_over = whole();
        a_field_over.push((key("cherry"), Value::from(0)));
        refused(a_field_over);

        let mut a_field_of_another_kind = whole();
        for (name, value) in &mut a_field_of_another_kind {
            if *name == key("landing") {
                *value = Value::from(7);
            }
        }
        refused(a_field_of_another_kind);
    }

    /// Bytes that are not this game's numbers at all are not a run to put back.
    #[test]
    fn bytes_of_another_game_are_no_numbers() {
        assert!(super::decode_state(b"not a run").is_none());
    }
}
