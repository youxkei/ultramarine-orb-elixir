//! **The file a run of th10 left unfinished is written in, which is this game's own.**
//!
//! Its own for the reason every game's is: a file is the game's, and one version number over two games'
//! numbers is what let a field go into th06's file without being raised — see `th06::resume`.
//!
//! **What a stage of this game is restarted from is its own replay record** — see *The replay* in `asm.md`: the
//! build fills a stage's `0x1c4`-byte record out of the run as it records (0x4290bf through 0x42910d), the
//! stage's first frame fills the player's part (0x42a4fc through 0x42a58f), and a playback puts every field back
//! at the same two moments (0x4292de through 0x429347, 0x42a5a8 through 0x42a68c). Every field of it is here,
//! and two more.
//!
//! **One field has no name of the game's**: `0x474c98`, which the record keeps at `+0x20`, the enemies' script
//! reads (0x4105eb on), a death takes `0x400` off (0x426caf) and a continue starts at `-0x200` (0x4179e2). It is
//! carried by address and nothing is claimed about it — a field put back into the wrong place is a run that
//! plays out differently and says nothing about why.
//!
//! **And two the record leaves out**: the deaths, which this game keeps nowhere and orb counts itself (see
//! `Th10`'s `DEATHS`), and the high score, which the record does not carry and the build takes out of the score
//! file (0x4178e7 through 0x417912).
//!
//! **The generator of every frame is not optional here**, for th07's reason: a file without them is a run
//! nothing can be held to. Version 1 means they are there.

use std::collections::BTreeMap;

use crate::game::RunStart;
use crate::resume::{Again, Kept, MAX_STAGE_FRAMES};

/// **How many past places the player keeps**, pairs of hundredths the options trail — the `0x108` bytes the
/// record copies at 0x42a512 and back at 0x42a5d2.
pub const PLACES_KEPT: usize = 0x108 / 8;
/// **How many numbers of the four options the record keeps**: four pairs apiece, `+0x32d4` on in steps of
/// `0x98` (0x42a528 through 0x42a56a).
pub const OPTION_NUMBERS: usize = 4 * 4 * 2;

/// **The numbers a stage of th10 starts with, which is what a stage of it needs to be played again.**
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Numbers {
    /// The score over ten, `0x474c44`.
    pub score: u32,
    /// The power, `0x474c48`, a word from nought to 100.
    pub power: u16,
    /// The faith over ten, `0x474c4c`, which the playback puts back through `0x418b80` with the timer beside it
    /// started over.
    pub faith: u32,
    /// The frames the faith stands before it falls, the timer at `0x474c54` whose frame is `0x474c58` — put back
    /// through `0x42a930`.
    pub faith_stands_for: i32,
    /// The lives, `0x474c70`.
    pub lives: i32,
    /// `0x474c98` — see the module's own doc.
    pub the_number_at_474c98: i32,
    /// The continues used, `0x474c90`.
    pub continues: i32,
    /// The extends the score has given, `0x474c9c`.
    pub extends: i32,
    /// What the generator was seeded with as the stage began, the record's `+0x2` — which a playback writes back
    /// with the draws counted from nought (0x4292d2 through 0x4292d8).
    pub seed: u16,
    /// Where the player is, in hundredths — `+0x3cc` and `+0x3d0`, which 0x428e10 turns into the floats beside
    /// them.
    pub at: (i32, i32),
    /// The player's past places, `+0x436c`.
    pub places: Vec<(i32, i32)>,
    /// **Whether the player is focused**, `+0x4474`, which the record keeps straight after the places
    /// (0x42a58f) — the focus key's bit of the word (0x42518a), held off with no enemy up or for the first four
    /// frames of the update timer (0x4251ca). Read by the shot's list (0x428182) and the options' (0x427972).
    /// Read under its old name too, which it was kept under before it was read off the exe.
    #[serde(alias = "newest_place")]
    pub focused: i32,
    /// The options' numbers, as the record keeps them.
    pub options: Vec<i32>,
    /// **orb's own and not the record's**: how many hits the run has taken.
    pub deaths: i32,
    /// **orb's own and not the record's either: whether the stage began at the end of the last one's fade** —
    /// the game object's `0x800` (0x4181f2), which a stage entered from the one before begins under and a stage
    /// built from the menu does not. **What that changes is the player's shot on the stage's first update.**
    /// Measured on a pair of stage 2: the run entered it by the transition and fired its first six shots on the
    /// update its clock went back to 1, `shot_timer=1 shots=6` after it, where the pick-up built it fresh and
    /// fired them an update later — every shot of the playback a frame behind the run's from there, the
    /// generator parting at frame 198.
    ///
    /// **And not the shot timer itself**, which is what was kept first: the numbers are read at the top of the
    /// update that begins the stage and the shot is fired inside it, so every file held -1 there, the run's
    /// and the pick-up's alike. Defaulted to a stage built from the menu for a file written before it.
    #[serde(default)]
    pub began_out_of_a_fade: bool,
    /// **And the frames nothing can hit the player for**, `+0x4310`: 120 out of the player's constructor
    /// (0x424d14), which a stage built from the menu has and a stage entered from the one before has whatever
    /// the last stage left — nought on that same pair, 118 on the pick-up's frame 2.
    #[serde(default)]
    pub unhittable_for: i32,
    /// **And where the ring of the small faith stands**, the item manager's `+0x21ceb8`: the slot the next one
    /// goes in, `0x800` of them after the first `0x96` (0x41bccd), which the manager a run keeps through a
    /// transition carries out of every stage before and a stage built from the menu starts at nought — and what
    /// a small faith does comes out of it, the slot's number taken modulo 4, 8, 16 or 32 as the item is made
    /// (0x41bd03 through 0x41bd51). Measured on a pair of stage 5: the run put one in slot 432 where the
    /// pick-up put it in 150, the two going on to be collected a frame apart from frame 3,806 and the landing
    /// 1 short on the score.
    #[serde(default)]
    pub small_faith_ring_at: i32,
}

/// Those numbers as the bytes the machinery carries about — see `Game::run_state`, which is where the
/// shape stops being anybody else's business.
pub fn encode_state(numbers: &Numbers) -> Vec<u8> {
    rmp_serde::to_vec_named(numbers).unwrap_or_default()
}

/// And back. `None` for bytes another game wrote, which is a file put in the wrong game's directory, and for
/// numbers of the player's the record could not hold.
pub fn decode_state(bytes: &[u8]) -> Option<Numbers> {
    let numbers: Numbers = rmp_serde::from_slice(bytes).ok()?;
    (numbers.places.len() == PLACES_KEPT && numbers.options.len() == OPTION_NUMBERS)
        .then_some(numbers)
}

/// What this file's fields mean, and what a file carrying another number is refused for. th10's own:
/// what tells this game's file from another's is `Game::named` in the directory each is kept in.
const VERSION: u32 = 1;

/// The file, whole. Written every time rather than appended to: it describes one chapter of one run, and
/// the one it describes is the newest.
pub fn encode(saved: &Kept) -> Vec<u8> {
    let Some(numbers) = decode_state(&saved.state) else {
        return Vec::new();
    };
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
        start: numbers,
        landing: saved.landing.clone(),
        buttons: changes(&saved.buttons),
        generators: saved.generators.clone(),
        again: saved.again.clone(),
    };
    // With the names in it, so that the file says what its own fields are — see `resume`'s own `EXTENSION`.
    rmp_serde::to_vec_named(&written).unwrap_or_default()
}

/// Reads it back, and `None` for anything that is not a file this version wrote: a field missing or
/// unknown, a number that is not one, a version that is not this one, or a generator missing for a frame.
pub fn decode(bytes: &[u8]) -> Option<Kept> {
    let written: Written = rmp_serde::from_slice(bytes).ok()?;
    if written.version != VERSION {
        return None;
    }
    let at = written.chapter.frame;
    // Nothing at or past the frame the buttons are meant to reach, that being a record which does not
    // describe the chapter beside it.
    if at > MAX_STAGE_FRAMES
        || written
            .buttons
            .keys()
            .last()
            .is_some_and(|frame| *frame >= at)
    {
        return None;
    }
    // One entry per frame the buttons reach, and no fewer: the playback holds the two together by frame.
    if written.generators.len() != at as usize {
        return None;
    }
    if written.again.iter().any(|again| again.frame >= at) {
        return None;
    }
    let state = encode_state(&written.start);
    decode_state(&state)?;
    Some(Kept {
        run: RunStart {
            difficulty: written.run.difficulty,
            character: written.run.character,
            shot_type: written.run.shot,
            // Not in the file, there being nothing to say: a practice run has no slot to be written in —
            // see `Game::run_slot` — so every file is a full run's.
            practice: false,
            stage: written.run.stage,
            // Not in the file either: this game has one mode of its own to start a run in.
            mode: 0,
        },
        state,
        at,
        chapter: written.chapter.number,
        name: written.chapter.name,
        retries: written.chapter.retries,
        song: written.chapter.song,
        song_identity: written.chapter.song_identity,
        song_sounding: written.chapter.song_sounding,
        landing: written.landing,
        buttons: dense(&written.buttons, at),
        generators: written.generators,
        again: written.again,
    })
}

/// The file's own shape, which is not the game's structs: the names in it are the file's business, so that
/// a field renamed in [`RunStart`] is not a file that reads differently afterwards. The numbers are
/// [`Numbers`] as they are, that one being this file's alone.
///
/// `deny_unknown_fields` throughout, because a field nothing knows is a file from another version or
/// another program, and reading one of those halfway is worse than not reading it.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    version: u32,
    run: WrittenRun,
    chapter: WrittenChapter,
    start: Numbers,
    /// The reproduction line exactly as the log prints it — see `Game::reproduction`.
    landing: String,
    /// The buttons held from each frame they changed on, keyed by that frame — only the changes.
    buttons: BTreeMap<u32, u16>,
    /// The generator's own state on every one of those frames — not optional, for the reason the module doc
    /// gives.
    generators: Vec<Option<u16>>,
    /// And every read of a frame before its last — see `resume::Again`.
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
    /// [`crate::resume::Kept::song`] and its two neighbours; `null` where the game would not say.
    song: Option<i32>,
    song_identity: Option<u32>,
    song_sounding: Option<bool>,
}

/// The frames the buttons changed on, out of one entry per frame.
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
    use super::{
        Numbers, OPTION_NUMBERS, PLACES_KEPT, VERSION, decode, decode_state, encode, encode_state,
    };
    use crate::game::RunStart;
    use crate::resume::{Again, Kept, MAX_STAGE_FRAMES};

    /// A run's numbers with a different number in every field: a reader that takes one for another is a run
    /// put back with somebody else's numbers, and nothing about the file would say so.
    fn numbers() -> Numbers {
        Numbers {
            score: 1_234_567,
            power: 87,
            faith: 54_321,
            faith_stands_for: 23,
            lives: 3,
            the_number_at_474c98: -0x180,
            continues: 1,
            extends: 2,
            seed: 0x1a2b,
            at: (1_200, 38_400),
            places: (0..PLACES_KEPT as i32)
                .map(|place| (place, -place))
                .collect(),
            focused: 1,
            options: (0..OPTION_NUMBERS as i32)
                .map(|number| number * 100 + 7)
                .collect(),
            deaths: 4,
            began_out_of_a_fade: true,
            unhittable_for: 57,
            small_faith_ring_at: 1_234,
        }
    }

    /// A chapter of a run of this game.
    fn saved() -> Kept {
        Kept {
            run: RunStart {
                difficulty: 1,
                character: 1,
                shot_type: 2,
                practice: false,
                stage: 3,
                mode: 0,
            },
            state: encode_state(&numbers()),
            at: 6,
            chapter: 4,
            name: "BOSS SPELL 1".to_owned(),
            retries: 9,
            song: None,
            song_identity: None,
            song_sounding: None,
            landing: "frames=6 seed=0x3274 score=1234567".to_owned(),
            buttons: vec![0, 1, 1, 0x11, 0x11, 0],
            generators: vec![
                Some(0x1000),
                None,
                Some(0x1002),
                Some(0x1003),
                Some(0x1004),
                Some(0x1005),
            ],
            // Out of order on purpose: the playback searches the file by frame, so a round trip that kept them
            // in the order written would say nothing about a file whose own order is the run's.
            again: vec![
                Again {
                    frame: 4,
                    buttons: 0x11,
                    generator: 0x2000,
                },
                Again {
                    frame: 2,
                    buttons: 0x1,
                    generator: 0x2001,
                },
            ],
        }
    }

    /// Every field of it, since a resume is a run started from these numbers.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&saved())), Some(saved()));
    }

    /// Numbers of the player's the record could not hold are no run's.
    #[test]
    fn numbers_the_record_could_not_hold_are_refused() {
        let mut short = numbers();
        short.places.pop();
        assert!(decode_state(&encode_state(&short)).is_none());
        let mut long = numbers();
        long.options.push(0);
        assert!(decode_state(&encode_state(&long)).is_none());
    }

    /// A file with some of the generators is one whose frames do not line up.
    #[test]
    fn a_file_with_some_of_the_generators_is_refused() {
        let mut half = saved();
        half.generators.pop();
        assert!(decode(&encode(&half)).is_none());
    }

    /// A read against a frame the chapter does not reach is a file describing another chapter.
    #[test]
    fn a_read_past_the_chapters_own_frame_is_refused() {
        let mut past = saved();
        past.again.push(Again {
            frame: past.at,
            buttons: 0x1,
            generator: 0x3000,
        });
        assert!(decode(&encode(&past)).is_none());
    }

    /// A frame number no stage reaches is not a file to allocate for.
    #[test]
    fn a_frame_no_stage_reaches_is_refused() {
        let mut far = saved();
        far.at = MAX_STAGE_FRAMES + 1;
        assert!(decode(&encode(&far)).is_none());
    }

    /// A file of another version, or one missing a field or carrying one nothing knows, is refused.
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
        a_field_over.push((key("faith_gauge"), Value::from(0)));
        refused(a_field_over);
    }

    /// Bytes that are not this game's numbers at all are not a run to put back.
    #[test]
    fn bytes_of_another_game_are_no_numbers() {
        assert!(decode_state(b"not a run").is_none());
    }

    /// **A file that kept the focus under its old name** reads it as the focus: every chapter kept before
    /// `+0x4474` was read off the exe carries it as `newest_place`.
    #[test]
    fn a_file_keeping_the_focus_under_its_old_name_reads_it_as_the_focus() {
        use rmpv::Value;

        let mut fields = match rmpv::decode::read_value(&mut encode_state(&numbers()).as_slice()) {
            Ok(Value::Map(fields)) => fields,
            other => panic!("not a map: {other:?}"),
        };
        for (name, _) in &mut fields {
            if *name == Value::String("focused".into()) {
                *name = Value::String("newest_place".into());
            }
        }
        let mut bytes = Vec::new();
        rmpv::encode::write_value(&mut bytes, &Value::Map(fields)).expect("written");
        assert_eq!(
            decode_state(&bytes).expect("read back").focused,
            numbers().focused
        );
    }

    /// **A file written before the fade, the frames of no hit and the small faith's ring were kept** reads them
    /// as a stage built from the menu has them — no fade and noughts — which is what every such file was put
    /// back as.
    #[test]
    fn a_file_written_before_the_fade_was_kept_reads_as_a_stage_built_from_the_menu() {
        use rmpv::Value;

        let mut fields = match rmpv::decode::read_value(&mut encode_state(&numbers()).as_slice()) {
            Ok(Value::Map(fields)) => fields,
            other => panic!("not a map: {other:?}"),
        };
        fields.retain(|(name, _)| {
            *name != Value::String("began_out_of_a_fade".into())
                && *name != Value::String("unhittable_for".into())
                && *name != Value::String("small_faith_ring_at".into())
        });
        let mut bytes = Vec::new();
        rmpv::encode::write_value(&mut bytes, &Value::Map(fields)).expect("written");
        let read = decode_state(&bytes).expect("read back");
        assert_eq!(
            (
                read.began_out_of_a_fade,
                read.unhittable_for,
                read.small_faith_ring_at
            ),
            (false, 0, 0)
        );
    }
}
