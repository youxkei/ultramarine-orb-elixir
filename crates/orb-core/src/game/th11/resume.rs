//! **The file a run of th11 left unfinished is written in, which is this game's own.**
//!
//! Its own for the reason every game's is: a file is the game's, and one version number over two games' numbers
//! is what let a field go into th06's file without being raised — see `th06::resume`.
//!
//! **What a stage of this game is restarted from is its own replay record**, `0x90` bytes a stage (0x435e00): the
//! run's first stage fills all of it as the replay is made in the build (0x4356e0 recording, at 0x41fc2b), the
//! stage's first frame fills the player's part (0x436da0, from 0x420480 and 0x4205bd), and a playback puts every
//! field back at the same two moments. Every field of it is here, and four more of orb's.
//!
//! **Fields with no name of the game's** are carried by address and nothing is claimed about them — a field put
//! back into the wrong place is a run that plays out differently and says nothing about why: `0x4a5744`, which a
//! death takes `0x400` off (0x432a52) and a continue starts at `-0x200` (0x41fb01), and `0x4a5750`.
//!
//! **The generator of every frame is not optional here**, for th07's reason: a file without them is a run nothing
//! can be held to. Version 1 means they are there.

use std::collections::BTreeMap;

use crate::game::RunStart;
use crate::resume::{Again, Kept, MAX_STAGE_FRAMES};

/// **The numbers a stage of th11 starts with, which is what a stage of it needs to be played again.**
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Numbers {
    /// The score over ten, `0x4a56e4`, the record's `+0xc`.
    pub score: u32,
    /// The power, `0x4a56e8`, the record's `+0x10`.
    pub power: i32,
    /// The point items' value, `0x4a56f0`, which the record keeps over a hundred (`+0x14`).
    pub point_value: i32,
    /// The lives, `0x4a5718`, and the pieces of the next, `0x4a571c` — the record's `+0x18` and `+0x1a`.
    pub lives: i32,
    pub life_pieces: i32,
    /// `0x4a5744` — see the module's own doc — the record's `+0x1c`.
    pub the_number_at_4a5744: i32,
    /// The continues used, `0x4a573c`, the record's `+0x28`.
    pub continues: i32,
    /// `0x4a5750`, the record's `+0x2c` — the extends the score has given (0x41efd8 through 0x41f006).
    pub extends: i32,
    /// The graze, `0x4a5754`, the record's `+0x34`.
    pub graze: u32,
    /// What the generator was seeded with as the stage began, the record's `+0x2`, which a playback writes back
    /// with the draws counted from nought.
    pub seed: u16,
    /// Where the player is, in 128ths — `+0x888` and `+0x88c` of the player, the record's `+0x20` and `+0x24`.
    pub at: (i32, i32),
    /// Whether the player is focused, `+0x8d20`, the record's `+0x30`.
    pub focused: i32,
    /// The element Patchouli's options are on, `+0x8bac`, the record's `+0x38`.
    pub element: i32,
    /// **orb's own and not the record's**: how many hits the run has taken.
    pub deaths: i32,
    /// **orb's own and not the record's either: whether the stage began at the end of the last one's fade** — the
    /// game object's `0x800` (0x420424), which a stage entered from the one before begins under and a stage built
    /// from the menu does not.
    pub began_out_of_a_fade: bool,
    /// **And the frames nothing can hit the player for**, `+0x8bb4`, which a stage entered from the one before has
    /// whatever the last stage left.
    pub unhittable_for: i32,
    /// **And where the ring of the small items stands**, the item manager's `+0x265e68`: the slot the next one fired
    /// off a bullet goes in, which a manager a run keeps through a transition carries out of every stage before.
    pub small_items_ring_at: i32,
}

/// Those numbers as the bytes the machinery carries about — see `Game::run_state`.
pub fn encode_state(numbers: &Numbers) -> Vec<u8> {
    rmp_serde::to_vec_named(numbers).unwrap_or_default()
}

/// And back. `None` for bytes another game wrote, which is a file put in the wrong game's directory.
pub fn decode_state(bytes: &[u8]) -> Option<Numbers> {
    rmp_serde::from_slice(bytes).ok()
}

/// What this file's fields mean, and what a file carrying another number is refused for.
const VERSION: u32 = 1;

/// The file, whole. Written every time rather than appended to: it describes one chapter of one run, and the one
/// it describes is the newest.
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
    rmp_serde::to_vec_named(&written).unwrap_or_default()
}

/// Reads it back, and `None` for anything that is not a file this version wrote.
pub fn decode(bytes: &[u8]) -> Option<Kept> {
    let written: Written = rmp_serde::from_slice(bytes).ok()?;
    if written.version != VERSION {
        return None;
    }
    let at = written.chapter.frame;
    if at > MAX_STAGE_FRAMES
        || written
            .buttons
            .keys()
            .last()
            .is_some_and(|frame| *frame >= at)
    {
        return None;
    }
    if written.generators.len() != at as usize {
        return None;
    }
    if written.again.iter().any(|again| again.frame >= at) {
        return None;
    }
    let state = encode_state(&written.start);
    Some(Kept {
        run: RunStart {
            difficulty: written.run.difficulty,
            character: written.run.character,
            shot_type: written.run.shot,
            // Not in the file: a practice run has no slot to be written in — see `Game::run_slot`.
            practice: false,
            stage: written.run.stage,
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

/// The file's own shape, which is not the game's structs — th10's reasons.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    version: u32,
    run: WrittenRun,
    chapter: WrittenChapter,
    start: Numbers,
    landing: String,
    buttons: BTreeMap<u32, u16>,
    generators: Vec<Option<u16>>,
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
    use super::{Numbers, VERSION, decode, decode_state, encode, encode_state};
    use crate::game::RunStart;
    use crate::resume::{Again, Kept, MAX_STAGE_FRAMES};

    /// A run's numbers with a different number in every field.
    fn numbers() -> Numbers {
        Numbers {
            score: 1_234_567,
            power: 61,
            point_value: 26_000_000,
            lives: 3,
            life_pieces: 2,
            the_number_at_4a5744: -0x180,
            continues: 1,
            extends: 2,
            graze: 456,
            seed: 0x1a2b,
            at: (1_280, 51_200),
            focused: 1,
            element: 3,
            deaths: 4,
            began_out_of_a_fade: true,
            unhittable_for: 57,
            small_items_ring_at: 1_234,
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

    /// A file with some of the generators is one whose frames do not line up.
    #[test]
    fn a_file_with_some_of_the_generators_is_refused() {
        let mut half = saved();
        half.generators.pop();
        assert!(decode(&encode(&half)).is_none());
    }

    /// A frame number no stage reaches is not a file to allocate for.
    #[test]
    fn a_frame_no_stage_reaches_is_refused() {
        let mut far = saved();
        far.at = MAX_STAGE_FRAMES + 1;
        assert!(decode(&encode(&far)).is_none());
    }

    /// A file of another version is refused.
    #[test]
    fn a_file_of_another_version_is_refused() {
        use rmpv::Value;

        let mut fields = match rmpv::decode::read_value(&mut encode(&saved()).as_slice()) {
            Ok(Value::Map(fields)) => fields,
            other => panic!("not a map: {other:?}"),
        };
        for (name, value) in &mut fields {
            if *name == Value::String("version".into()) {
                *value = Value::from(VERSION + 1);
            }
        }
        let mut bytes = Vec::new();
        rmpv::encode::write_value(&mut bytes, &Value::Map(fields)).expect("written");
        assert!(decode(&bytes).is_none());
    }

    /// Bytes that are not this game's numbers at all are not a run to put back.
    #[test]
    fn bytes_of_another_game_are_no_numbers() {
        assert!(decode_state(b"not a run").is_none());
    }
}
