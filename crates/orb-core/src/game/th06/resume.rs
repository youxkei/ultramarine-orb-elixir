//! **The file a run of th06 left unfinished is written in, which is this game's own.**
//!
//! It was one shape for both games, with one version number between them, and that is what this file exists
//! to end. What it cost is written where it was measured: a field th07 needed could not be added without
//! refusing every th06 file already on a disk, so it went in without the version being raised and the reader
//! learnt to read a file both with the field and without it — see [`Written::generators`]. The two games are
//! two games, and the only fact about them that was ever measured is that.
//!
//! **So this is frozen at what shipped.** th06 is the game people already have, and the bytes below are the
//! bytes a released orb wrote: version 2, the run, the chapter, the thirteen numbers a stage starts with —
//! three of which this game never reads and writes as nothing — the reproduction line, the buttons of every
//! frame they changed on, and the generator of each. What th07 does with its own file is its business now,
//! and nothing here moves when it moves.
//!
//! What the machinery in [`crate::resume`] asks of this is two things: the bytes to write, and a
//! [`Kept`](crate::resume::Kept) out of the bytes it read.

use std::collections::BTreeMap;

use crate::game::RunStart;
use crate::resume::{Kept, MAX_STAGE_FRAMES};

/// **The numbers a stage of th06 starts with, which is what a stage of it needs to be played again.**
///
/// These are the fields the game's own replay keeps per stage — see `SPEC.md` for the record they are read out
/// of — plus the count of deaths, which the replay leaves behind because the result screen is the only thing
/// that reads it. Everything else a stage starts with is either loaded with the stage or set the same way
/// every time, so it is the same on both sides of a resume.
///
/// **This game's own, and no other game's.** It was one struct shared with th07, which meant a cherry
/// th06 has no notion of sitting in the middle of it and this game writing three zeros where that game
/// writes its own numbers. What the sharing actually cost is written where the file's version is — see
/// [`VERSION`] — and it is why these are here now.
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
/// **Two, and it stays two**: the runs of th06 on people's disks are version 2, and a number raised here
/// stops them being offered. There is nothing left to add to this file — a game whose exe cannot change is a
/// game whose run cannot need a field it has not needed for years — so nothing has to be traded against that
/// again.
const VERSION: u32 = 2;

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
            // The three this game never reads, written as nothing: they are in the file because they were in
            // it when th06 shipped — see [`WrittenStart`].
            cherry: 0,
            cherry_target: 0,
            cherry_plus: 0,
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
/// a number that is not one, a version that is not this one.
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
    // And an entry for every frame the buttons reach, or no list at all: the playback holds the two together
    // by frame, so a list that is shorter is one whose frames do not line up, and reading it would be a
    // playback held against another frame's generator. No list at all is a file from before orb wrote them
    // down, which is a run to offer with nothing to say about where a playback of it parts. An entry that
    // holds nothing is a frame of that kind on its own, and costs only its own check.
    if !written.generators.is_empty() && written.generators.len() != at as usize {
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
        // The three the file carries and this game does not read are dropped here, which is where they stop
        // travelling any further.
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
    /// somebody can hold against the log by eye.
    ///
    /// **Two things it cannot say, both worth knowing before changing anything near it.** Changing what
    /// [`crate::game::Reproduction`] puts in this line makes every file written before the change disagree at
    /// its first field, and [`VERSION`] does not catch it: the line is opaque to the reader, which is the
    /// price of holding the log's own text. And a run written down by one build and picked up by another whose
    /// midstage table differs lands on a chapter the second build names and numbers differently, which passes
    /// unnoticed — what is compared here is the reproduction and not the name.
    landing: String,
    /// The buttons held from each frame they changed on, keyed by that frame.
    ///
    /// Only the changes, which is how the game writes its own record: everything between two of them is the
    /// first one's, and a stage's worth of frames comes to a few hundred entries. A map rather than a list of
    /// pairs, so that they are in order because a map of frames is, rather than because something checked; it
    /// is also a byte less each, and a converter prints one per line where a list of pairs is two.
    ///
    /// **The keys being numbers is what stops that converter writing JSON**, which has no way to say a map key
    /// that is not a string: `msgpack2json` on one of these answers `map key is not a string`, and its debug
    /// view — `msgpack2json -d` — is what prints the file.
    buttons: BTreeMap<u32, u16>,
    /// The generator's own state on every one of those frames — see [`Kept::generators`] for what it is for.
    ///
    /// **A list and not a map of the changes**, which is the opposite of the buttons above and for the opposite
    /// reason: a generator moves on nearly every frame, so a map of changes would be every frame with a key in
    /// front of it. A stage's worth is 8500 entries, which is 26KB as an array of words against 43KB as a map
    /// of them, and the index is the frame either way.
    ///
    /// **Absent in a file written before orb wrote them down**, which is what the default is for. It is also
    /// the whole reason this file is th06's own now: the field went in for th07 and the version could not
    /// be raised for it, because the number was one number for both games and raising it would have refused
    /// every th06 run on a disk.
    #[serde(default)]
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
    /// Where the song was in the track's own file, and `null` where the stream would not say — which is a
    /// resume that leaves the music as the stage started it.
    song: Option<i32>,
}

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
    /// **th07's three, which this game never reads and writes as nothing.** They are here because they were
    /// here when th06 shipped: the file was one shape for both games, and a released file has a `cherry` in
    /// it whether or not the game that wrote it has a cherry. Taking them out now would refuse every one of
    /// those files, which is the same trade that put them here.
    cherry: i32,
    cherry_target: i32,
    cherry_plus: i32,
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
                score: 7417420,
                seed: 0x1a2b,
                point_items: 328,
                power: 128,
                lives: 2,
                bombs: 3,
                rank: 21,
                power_items: 44,
                extra_lives: 1,
                deaths: 6,
            }),
            at: 12,
            chapter: 17,
            name: "BOSS SPELL 2".to_owned(),
            retries: 42,
            song: Some(1234567),
            landing: "replay_frame=-1 input=0x0000 randoms=1234".to_owned(),
            buttons: vec![0, 0, 1, 1, 1, 0x41, 0x41, 0, 0, 0, 0x101, 0x101],
            // One per frame like the buttons, and every one different: what the file has to carry through is
            // the frame each of them belongs to, so a list that survived the round trip in the wrong order
            // would read as a playback parting on the first frame it was held against. **And one frame with
            // no word**, which is a frame this run played back out of a file that had none for it.
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

    /// The buttons are written as the frames they changed on and come back one per frame, holding the last
    /// change through to the chapter — which is what a run with a key held down is.
    #[test]
    fn the_buttons_come_back_one_per_frame() {
        let mut held = saved();
        held.at = 6;
        held.buttons = vec![0, 0, 1, 1, 1, 1];
        held.generators.truncate(6);
        // One entry, not six frames.
        assert_eq!(
            super::changes(&held.buttons),
            std::collections::BTreeMap::from([(2, 1)]),
        );
        assert_eq!(decode(&encode(&held)), Some(held));
    }

    /// A stream that would not say where the song was is written as nothing rather than as a number standing
    /// for nothing, and read back as a resume that leaves the music as the stage started it.
    #[test]
    fn a_chapter_with_no_song_position_is_read_as_having_none() {
        let mut quiet = saved();
        quiet.song = None;
        assert_eq!(decode(&encode(&quiet)), Some(quiet));
    }

    /// **A file with no generators in it is still a run to offer**, which is every file written before orb
    /// began writing them down. th06 is the game people already have and this is its own file: a run of it
    /// left unfinished is not one to drop because a launch has learnt to write something more about a stage.
    ///
    /// What that costs is what th07's own file refuses to pay — see `th07::resume`, where the generators are
    /// not optional: a playback of a run with none of them cannot be held to it, and one such th07 run
    /// parted somewhere in the sixty frames after 853, was hit at 1249 and died out at 5900 of the 24119
    /// frames it was aiming at, with nothing in the log able to say which frame it went wrong on.
    #[test]
    fn a_file_from_before_the_generators_is_read_and_offered() {
        let mut older = saved();
        older.generators = Vec::new();
        assert_eq!(decode(&encode(&older)), Some(older));
    }

    /// And one with *some* of them is refused, that being a file whose frames do not line up: the buttons of
    /// a frame and the generator it was pressed with are held together by their index, so a list that runs out
    /// part way through would have a playback held against another frame's number.
    #[test]
    fn a_file_with_some_of_the_generators_is_refused() {
        let mut half = saved();
        half.generators.truncate(half.buttons.len() - 1);
        assert!(decode(&encode(&half)).is_none());
    }

    /// A chapter at the stage's own start has none of them, which is a run resumed by building the stage and
    /// playing nothing.
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

    /// A file this version did not write is refused rather than read with its fields taken for each other,
    /// and so is one that has lost a field, gained one nothing knows, or holds a field of another kind than
    /// it should.
    ///
    /// Written as MessagePack maps here rather than by editing bytes, since what has to be refused is a *file*
    /// of that shape however it came to be one.
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
        let named = |name: &str| Value::String(name.into());
        let replacing = |name: &str, with: Value| {
            let mut fields = whole();
            let at = fields
                .iter()
                .position(|(key, _)| *key == named(name))
                .expect("a field of that name");
            fields[at].1 = with;
            fields
        };

        // The whole of it, untouched, is the file that is read — everything below is that file with one thing
        // wrong with it.
        assert_eq!(decode(&encode(&saved())), Some(saved()));

        refused(replacing("version", Value::from(VERSION + 1)));
        refused(replacing("landing", Value::from(0)));
        refused(
            whole()
                .into_iter()
                .filter(|(key, _)| *key != named("start"))
                .collect(),
        );
        refused({
            let mut fields = whole();
            fields.push((named("elsewhere"), Value::from(1)));
            fields
        });
        // A frame at or past the one the buttons are meant to reach is a record that does not describe the
        // chapter beside it.
        let held_from = |frames: [u32; 2]| {
            Value::Map(
                frames
                    .iter()
                    .map(|frame| (Value::from(*frame), Value::from(1)))
                    .collect(),
            )
        };
        refused(replacing("buttons", held_from([2, 12])));
        refused(replacing("buttons", held_from([2, saved().at])));
    }
}
