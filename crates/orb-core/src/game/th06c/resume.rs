//! **The file a run of th06c left unfinished is written in, which is this game's own.**
//!
//! Its own for the reason th07's is: a file is the game's, and one version number over two games' numbers is
//! what let a field go into th06's file without being raised — see `th06::resume`. th06c is the same game
//! played and another program playing it, and what a stage of it is restarted from is the eight fields its
//! own replay record carries — `ReplayManager::AddedCallbackDemo` at 0x14004da40 puts them back, field for
//! field, out of the record at 0x14004d8a0 — plus the two orb counts for itself. Those eight are th06 1.02h's
//! eight, which is why this file reads like th06's and is not it: th06's is frozen at what shipped and reads
//! two versions, and this one starts at 1 with the generators required.
//!
//! **The generator of every frame is not optional here**, for th07's reason: a file without them is a run
//! nothing can be held to. Version 1 means they are there.
//!
//! What the machinery in [`crate::resume`] asks of this is two things: the bytes to write, and a
//! [`Kept`](crate::resume::Kept) out of the bytes it read.

use std::collections::BTreeMap;

use crate::game::RunStart;
use crate::resume::{Kept, MAX_STAGE_FRAMES};

/// **The numbers a stage of th06c starts with, which is what a stage of it needs to be played again.**
///
/// The eight the game's own per-stage replay record holds — `0x14004d8a0` writes them out of the statics
/// and `0x14004da40` writes them back: the score, the seed, the point items, the power, the lives, the bombs,
/// the rank and the count of power items towards the next step of the score — and the two the record leaves
/// behind because only the result screen and orb read them: how many extra lives the score has paid for, and
/// the deaths.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Numbers {
    pub score: u32,
    /// What the generator was seeded with as the stage began. The whole of why the buttons alone are not
    /// enough: the game takes this from wherever the generator had got to, so a stage played again from a
    /// different seed is a different stage.
    pub seed: u16,
    pub point_items: u16,
    pub power: u16,
    pub lives: i8,
    pub bombs: i8,
    /// What the game makes of how the run is going, which the enemies read.
    pub rank: i32,
    pub power_items: i8,
    /// How many extra lives the score has already paid for, so the next one is not paid twice.
    pub extra_lives: i8,
    /// **orb's own count and not the game's**: this exe keeps no count of deaths — see `Th06c::read_state`,
    /// which counts them off the player's own state — so what goes back in is what orb had counted.
    pub deaths: i32,
}

/// Those numbers as the bytes the machinery carries about — see `Game::run_state`, which is where the shape
/// stops being anybody else's business.
pub fn encode_state(numbers: &Numbers) -> Vec<u8> {
    rmp_serde::to_vec_named(numbers).unwrap_or_default()
}

/// And back. `None` for bytes another game wrote, which is a file put in the wrong game's directory.
pub fn decode_state(bytes: &[u8]) -> Option<Numbers> {
    rmp_serde::from_slice(bytes).ok()
}

/// What this file's fields mean, and what a file carrying another number is refused for.
///
/// **One, and th06c's own.** th06's file carries 1 and 2 as well, and a th06 file put in this game's
/// directory would read as far as its fields go — which is the same ten names — and then be a run of another
/// exe's put back into this one. What tells the two apart is `Game::named` in the directory each is kept in,
/// and not this number.
const VERSION: u32 = 1;

/// The file, whole. Written every time rather than appended to: it describes one chapter of one run, and the
/// one it describes is the newest.
pub fn encode(saved: &Kept) -> Vec<u8> {
    // The numbers are this game's own bytes by the time they get here — see [`decode_state`]. Nothing but a
    // file of another game's could fail to read, and one of those is a file in the wrong directory.
    let numbers = decode_state(&saved.state).unwrap_or(Numbers {
        score: 0,
        seed: 0,
        point_items: 0,
        power: 0,
        lives: 0,
        bombs: 0,
        rank: 0,
        power_items: 0,
        extra_lives: 0,
        deaths: 0,
    });
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
            power: numbers.power,
            lives: numbers.lives,
            bombs: numbers.bombs,
            rank: numbers.rank,
            power_items: numbers.power_items,
            extra_lives: numbers.extra_lives,
            deaths: numbers.deaths,
        },
        landing: saved.landing.clone(),
        buttons: changes(&saved.buttons),
        generators: saved.generators.clone(),
    };
    // With the names in it, so that the file says what its own fields are — see `resume`'s own `EXTENSION`.
    // Nothing in it can fail to serialize — every field is a number, a string or a pair of numbers — so a
    // failure here is a file that would say nothing about the run, which is what no bytes read as.
    rmp_serde::to_vec_named(&written).unwrap_or_default()
}

/// Reads it back, and `None` for anything that is not a file this version wrote: a field missing or unknown,
/// a number that is not one, a version that is not this one, or a generator missing for a frame.
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
    // One entry per frame the buttons reach, and no fewer: the playback holds the two together by frame, so a
    // list of another length is one whose frames do not line up.
    if written.generators.len() != at as usize {
        return None;
    }
    Some(Kept {
        run: RunStart {
            difficulty: written.run.difficulty,
            character: written.run.character,
            shot_type: written.run.shot,
            // Not in the file, there being nothing to say: a practice run has no slot to be written in — see
            // `Game::run_slot` — so every file is a full run's.
            practice: false,
            stage: written.run.stage,
        },
        state: encode_state(&Numbers {
            score: written.start.score,
            seed: written.start.seed,
            point_items: written.start.points,
            power: written.start.power,
            lives: written.start.lives,
            bombs: written.start.bombs,
            rank: written.start.rank,
            power_items: written.start.power_items,
            extra_lives: written.start.extra_lives,
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
    })
}

/// The file's own shape, which is not the game's structs: the names in it are the file's business, so that a
/// field renamed in [`RunStart`] is not a file that reads differently afterwards.
///
/// `deny_unknown_fields` throughout, because a field nothing knows is a file from another version or another
/// program, and reading one of those halfway is worse than not reading it.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    version: u32,
    run: WrittenRun,
    chapter: WrittenChapter,
    start: WrittenStart,
    /// The reproduction line exactly as the log prints it, kept as the one string it is: a landing is held
    /// against it by field name, so what is compared is this text either way, and the file shows the line
    /// somebody can hold against the log by eye. See `Game::reproduction` for what changing the line costs.
    landing: String,
    /// The buttons held from each frame they changed on, keyed by that frame — only the changes, which is how
    /// the game writes its own record too: `0x14004d760` writes an entry where the word differs from the last
    /// one's and nowhere else.
    buttons: BTreeMap<u32, u16>,
    /// The generator's own state on every one of those frames, which is what says whether a playback is still
    /// reproducing the run — and not optional, for the reason the module doc gives.
    generators: Vec<Option<u16>>,
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
    /// [`crate::resume::Kept::song`] and its two neighbours. `null` where the game would not say, which for
    /// this game is every chapter until its music is read: the three are kept so that the file says the same
    /// things every other game's does.
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
    points: u16,
    power: u16,
    lives: i8,
    bombs: i8,
    rank: i32,
    power_items: i8,
    extra_lives: i8,
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
    use crate::resume::{Kept, MAX_STAGE_FRAMES};

    /// A chapter of a run of this game, with a number in every field: a reader that takes one field for
    /// another is a run put back with somebody else's numbers, and nothing about the file would say so.
    fn saved() -> Kept {
        Kept {
            run: RunStart {
                difficulty: 3,
                character: 1,
                shot_type: 0,
                practice: false,
                stage: 3,
            },
            state: encode_state(&Numbers {
                score: 12_345_670,
                seed: 0x1a2b,
                point_items: 321,
                power: 128,
                lives: 4,
                bombs: 2,
                rank: 20,
                power_items: 7,
                extra_lives: 2,
                deaths: 3,
            }),
            at: 12,
            chapter: 5,
            name: "MIDBOSS SPELL 1".to_owned(),
            retries: 9,
            song: None,
            song_identity: None,
            song_sounding: None,
            landing: "replay_frame=4472 input=0x0014 randoms=0 rng=0x3274".to_owned(),
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
        }
    }

    /// Every field of it, since a resume is a run started from these numbers: one read back as another is a
    /// run that plays out differently and says nothing about why.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&saved())), Some(saved()));
    }

    /// A file with no generators in it is refused, which is what this version is for: a run whose frames have
    /// no word to be held against is a run nothing can say it is reproducing.
    #[test]
    fn a_file_with_no_generators_is_refused() {
        let mut older = saved();
        older.generators = Vec::new();
        assert!(decode(&encode(&older)).is_none());
    }

    /// And one with some of them, which is a file whose frames do not line up: the buttons of a frame and the
    /// generator it was pressed with are held together by their index.
    #[test]
    fn a_file_with_some_of_the_generators_is_refused() {
        let mut half = saved();
        half.generators.truncate(half.buttons.len() - 1);
        assert!(decode(&encode(&half)).is_none());
    }

    /// A chapter at the stage's own start has no buttons under it, which is a run resumed by building the
    /// stage and playing nothing — and no generators either, there being no frame to have one.
    #[test]
    fn a_stage_start_holds_no_buttons() {
        let mut start = saved();
        start.at = 0;
        start.chapter = 1;
        start.name = "MIDSTAGE 1".to_owned();
        start.buttons = Vec::new();
        start.generators = Vec::new();
        assert_eq!(decode(&encode(&start)), Some(start));
    }

    /// A frame number no stage reaches is not a file to allocate for, whatever asked.
    #[test]
    fn a_frame_no_stage_reaches_is_refused() {
        let mut far = saved();
        far.at = MAX_STAGE_FRAMES + 1;
        assert!(decode(&encode(&far)).is_none());
    }

    /// A file this version did not write is refused rather than read with its fields taken for each other, and
    /// so is one that has lost a field, gained one nothing knows, or holds a field of another kind than it
    /// should.
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
