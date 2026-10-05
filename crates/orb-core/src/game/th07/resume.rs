//! **The file a run of th07 left unfinished is written in, which is this game's own.**
//!
//! It was th06's file with th07's numbers added to it, and one version number stood in front of both. What
//! that cost is the whole reason this file exists: the cherry went in as three more fields of a shape th06
//! shares, the generator of every frame went in without the version being raised at all — because raising it
//! would have refused every th06 run on a disk — and a th07 run picked up out of a file from before that
//! parted from itself inside its first thousand frames with nothing in the log able to say where.
//!
//! **So this is th07's, from version 1.** th06 is released and its file is frozen at what shipped; this one
//! is not released and can say what a run of *this* game is. What it does not carry is th06's vocabulary:
//! no count of power items, and th06's rank and extra lives only in this game's own shapes — the rank is a
//! static of its own, and the extra lives are given by the point items, counted against thresholds of
//! their own.
//!
//! **And the generator of every frame is not optional here.** A file without them is a run nothing can be held
//! to: the buttons play back into a stage that is no longer the same stage and what shows up is a death
//! thousands of frames later. Version 1 means they are there, so a file that has none is a file this reader
//! does not know.
//!
//! What the machinery in [`crate::resume`] asks of this is two things: the bytes to write, and a
//! [`Kept`](crate::resume::Kept) out of the bytes it read.

use std::collections::BTreeMap;

use crate::game::RunStart;
use crate::resume::{Kept, MAX_STAGE_FRAMES};

/// **The numbers a stage of th07 starts with, which is what a stage of it needs to be played again.**
///
/// This game's own, and nothing of another game's: there is no count of power items and no count of extra
/// lives the score has paid for, because th07 reads neither — its own extra lives are given by the point
/// items, which the two below the score count — and its rank is the static at the end.
///
/// **The cherry is three numbers and a baseline they are counted from**, and the baseline is the run's rather
/// than the stage's: 0x42e3da draws it as the run is made. Each of the three is kept here as a count from it,
/// which is how every reader of them in the game reads them, so a resumed stage's own baseline is what they
/// are put back against.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Numbers {
    pub score: u32,
    /// The score the panel is drawing, which the game's own teardown and the continue menu's ways out of a run
    /// catch up with the one above: a stage resumed without it draws the number the front end left and counts up to
    /// the run's from there.
    pub score_shown: u32,
    /// How many extends the point items have given and how many items the next one wants — th07's own extend.
    /// A stage resumed with a first stage's thresholds is a stage whose items give an extend where a first
    /// stage's would.
    ///
    /// `alias` for the names they were written under while they were read as a point value.
    #[serde(alias = "point_value_step")]
    pub extends_given: u32,
    #[serde(alias = "point_value_next")]
    pub items_for_the_next_extend: u32,
    /// What the generator was seeded with as the stage began. The whole of why the buttons alone are not
    /// enough: the game takes this from wherever the generator had got to, so a stage played again from a
    /// different seed is a different stage.
    pub seed: u16,
    pub point_items: u16,
    pub power: u16,
    pub lives: i8,
    pub bombs: i8,
    pub deaths: i32,
    pub cherry: i32,
    pub cherry_target: i32,
    pub cherry_plus: i32,
    /// What an item's cherry is worked out from, which the run's own build puts back to nothing: a stage
    /// resumed without it fills its cherry at a first stage's rate. See `Th07::run_state`.
    pub counted: i32,
    /// And the cards this run has captured, which the game's own replay restore carries for the same reason.
    pub captured: i32,
    /// The step this run's awards have climbed to, which is what a collection is worth: a stage resumed at the
    /// bottom of it takes 30 where the run took 2030. See `Th07::run_state`.
    pub award_step: u8,
    /// **Whether the run was carrying a border as the stage began**, which the cherry+ beside it cannot say: a
    /// border is *earned* at a figure and then lasts its own 540 frames, so a run crosses into the next stage
    /// with whatever is left of one and a plus nowhere near that figure.
    ///
    /// **And a border is the one thing in a stage that takes items off the screen.** It absorbs every one of
    /// them, deflects a bullet, and its absorbing is counted in the run's own count — so a stage put back
    /// without the border it had is a stage that collects what the run did not. Five pairs of launches parted
    /// exactly there: an item taken on one pass and passed through on the other, the count up by one, with
    /// every field of a frame's line agreeing and the plus between 5790 and 8780.
    ///
    /// `default` because this game's file is read from version 1 and a file written before this field is a file
    /// whose runs had a border or not with nothing said: false is what such a file has always been read as, and
    /// it is what the numbers already meant.
    #[serde(default)]
    pub border: bool,
    /// **The rank**, `[0x62f8a4]` — see `Th07::RANK`.
    ///
    /// A run's build fills it out of the table by difficulty at 0x42e3a4, inside the block 0x42e89b jumps over,
    /// so a stage the run walked into carries whatever the stage before it left and a stage a resume builds gets
    /// the table's starting figure. What moves it while a run is played is 0x42f4aa and 0x42f526, through
    /// `[ecx+0x9634]` — which is why no absolute store in the exe mentions it.
    ///
    /// And the ECL runner 0x410520 reads it as an enemy fires: 0x412ad4 and 0x412b3c scale the bullet counts,
    /// and 0x412bab and 0x412c04, with `fimul`, the speeds, each as `low + (high - low) * rank / 32` added to
    /// what the instruction asked for. So a higher rank is more bullets and faster ones, and the same stage 2
    /// held 0x20 entered by a transition against 0x10 built by a resume.
    ///
    /// `default` for the reason the border above is: a file written before this field is one whose runs had
    /// whatever a build left, and nothing is what such a file has always been read as.
    ///
    /// `alias` for the name it was written under while it was read as a number of the shot's.
    #[serde(default, alias = "shots_own_number")]
    pub rank: u32,
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
/// **One, and th07's own.** The number th06's file carries is that file's; a run of this game written down
/// before this file was its own says 2 and is refused, which is what it deserves: those are the files whose
/// runs cannot be reproduced.
///
/// **And it stays one while this game is unreleased, however the fields change.** What a version is for is the
/// files already on somebody's disk, and there are none of those to keep readable: th07 has not shipped, so
/// every file is one this tree wrote and can be thrown away. A field added or dropped is refused by
/// `deny_unknown_fields` on the way in anyway — which is what refuses the files version 1 wrote with the eight
/// numbers of [`super::Th07::set_run_state`]'s own story in them — so raising the number would buy nothing but
/// a second reason for the same refusal.
const VERSION: u32 = 1;

/// The file, whole. Written every time rather than appended to: it describes one chapter of one run, and the
/// one it describes is the newest.
pub fn encode(saved: &Kept) -> Vec<u8> {
    // The numbers are this game's own bytes by the time they get here — see [`decode_state`]. Nothing but a
    // file of another game's could fail to read, and one of those is a file in the wrong directory.
    let numbers = decode_state(&saved.state).unwrap_or(Numbers {
        score: 0,
        score_shown: 0,
        extends_given: 0,
        items_for_the_next_extend: 0,
        seed: 0,
        point_items: 0,
        power: 0,
        lives: 0,
        bombs: 0,
        deaths: 0,
        cherry: 0,
        cherry_target: 0,
        cherry_plus: 0,
        counted: 0,
        captured: 0,
        award_step: 0,
        border: false,
        rank: 0,
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
            shown: numbers.score_shown,
            extends_given: numbers.extends_given,
            items_for_the_next_extend: numbers.items_for_the_next_extend,
            seed: numbers.seed,
            points: numbers.point_items,
            power: numbers.power,
            lives: numbers.lives,
            bombs: numbers.bombs,
            deaths: numbers.deaths,
            cherry: numbers.cherry,
            cherry_target: numbers.cherry_target,
            cherry_plus: numbers.cherry_plus,
            counted: numbers.counted,
            captured: numbers.captured,
            award_step: numbers.award_step,
            border: numbers.border,
            rank: numbers.rank,
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
    // list of another length is one whose frames do not line up. An entry may hold nothing — those are the
    // frames a playback fed rather than read, which have no generator of their own to write down — and each
    // costs only its own check.
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
            // Not in the file either, and for a plainer reason: this build has one mode — see
            // `RunStart::mode`.
            mode: 0,
        },
        state: encode_state(&Numbers {
            score: written.start.score,
            score_shown: written.start.shown,
            extends_given: written.start.extends_given,
            items_for_the_next_extend: written.start.items_for_the_next_extend,
            seed: written.start.seed,
            point_items: written.start.points,
            power: written.start.power,
            lives: written.start.lives,
            bombs: written.start.bombs,
            deaths: written.start.deaths,
            cherry: written.start.cherry,
            cherry_target: written.start.cherry_target,
            cherry_plus: written.start.cherry_plus,
            counted: written.start.counted,
            captured: written.start.captured,
            award_step: written.start.award_step,
            border: written.start.border,
            rank: written.start.rank,
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
        // **Nothing, and this game's file carries none.** A read of a frame before its last is what a clock
        // that stands while the run plays on makes — th08's dialogue, where `resume::Again` was measured —
        // and this game's own frame count is stepped by its slowdown rather than held: a frame read twice
        // here is a frame its run was not advanced on, which is the case the one entry a frame was always
        // right for. So a playback of this game is fed a frame's last read for every update it spends on
        // it, which is what it was fed before.
        again: Vec::new(),
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
    /// first one's, and a stage's worth of frames comes to a few hundred entries — 1837 of them in a run of
    /// stage 6 taken to its last boss card. A map rather than a list of pairs, so that they are in order
    /// because a map of frames is, rather than because something checked.
    ///
    /// **The keys being numbers is what stops `msgpack2json` writing JSON**, which has no way to say a map key
    /// that is not a string: its debug view — `msgpack2json -d` — is what prints the file.
    buttons: BTreeMap<u32, u16>,
    /// The generator's own state on every one of those frames, which is what says whether a playback is still
    /// reproducing the run — and **not optional**, unlike th06's, whose file had to keep reading without them.
    ///
    /// **A list and not a map of the changes**, which is the opposite of the buttons above and for the opposite
    /// reason: a generator moves on nearly every frame, so a map of changes would be every frame with a key in
    /// front of it. A stage's worth is 24119 entries in that same run, which is 72KB as an array of words
    /// against 120KB as a map of them, and the index is the frame either way.
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
    /// Where the song was in the file the game streams from, and `null` where the stream would not say —
    /// which is a resume that leaves the music as the stage started it.
    song: Option<i32>,
    /// And which track of that file the position is in — see [`crate::resume::Kept::song_identity`]. `null`
    /// where the stream would not say, and a file without the key at all is refused with everything else this
    /// reader cannot vouch for: a position with no track to hold it to is one nothing can seek a stream by.
    song_identity: Option<u32>,
    /// And whether any of it was coming out — see [`crate::resume::Kept::song_sounding`]. A chapter taken with
    /// its buffer stopped is a chapter whose sound was nothing, and a position put back and played would make
    /// that silence audible.
    ///
    /// **Not optional in this game's own file**, unlike th06's: what a launch of this build writes it always
    /// says, and a file from before it is refused with the rest of what this reader cannot vouch for.
    song_sounding: Option<bool>,
}

/// The numbers a stage of *this* game starts with, and none of another game's.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenStart {
    score: u32,
    /// The one the panel is drawing, and the two behind the count of point items: how many extends they have
    /// given and how many items the next one wants.
    shown: u32,
    #[serde(alias = "point_value_step")]
    extends_given: u32,
    #[serde(alias = "point_value_next")]
    items_for_the_next_extend: u32,
    #[serde(with = "hex")]
    seed: u16,
    points: u16,
    power: u16,
    lives: i8,
    bombs: i8,
    deaths: i32,
    /// The cherry, what it is filling towards, and the cherry+ — each a count from the run's own baseline,
    /// which is how every reader of them in the game reads them.
    cherry: i32,
    cherry_target: i32,
    cherry_plus: i32,
    /// And what an item's cherry is worked out from, and the cards captured beside it: counts of the run's
    /// rather than the stage's, which the game's own replay record carries for the same reason.
    counted: i32,
    captured: i32,
    /// And the step this run's awards have climbed to, which is what a collection is worth.
    award_step: u8,
    /// And whether the run was carrying a border, which the cherry+ beside it cannot say — see
    /// [`Numbers::border`]. `default` so that a file written before this field is read as a run that had none,
    /// which is what those files have always been put back as.
    #[serde(default)]
    border: bool,
    /// And the rank — see [`Numbers::rank`]. `default` for the same reason the border above has one, and
    /// `alias` for the name a file written before it was read as the rank holds it under.
    #[serde(default, alias = "shots_own_number")]
    rank: u32,
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

    /// A chapter of a run of this game, with a number in every field it has — and nothing in th06's three,
    /// which this game answers nothing for and this file does not carry.
    fn saved() -> Kept {
        Kept {
            run: RunStart {
                difficulty: 1,
                character: 0,
                shot_type: 1,
                practice: false,
                stage: 5,
                mode: 0,
            },
            state: encode_state(&Numbers {
                score: 36_662_377,
                score_shown: 36_662_370,
                extends_given: 12,
                items_for_the_next_extend: 5_320,
                seed: 0x4ccc,
                point_items: 993,
                power: 128,
                lives: 8,
                bombs: 8,
                deaths: 0,
                cherry: 282_330,
                cherry_target: 282_330,
                cherry_plus: 25_200,
                counted: 9_614,
                captured: 11,
                award_step: 19,
                border: true,
                rank: 0x20,
            }),
            at: 12,
            chapter: 14,
            name: "BOSS SPELL 6".to_owned(),
            retries: 13,
            song: Some(269_385_428),
            song_identity: Some(0x0089_abcd),
            song_sounding: Some(true),
            landing: "replay_frame=24119 input=0x0014 randoms=0 rng=0x3274".to_owned(),
            buttons: vec![0, 0, 1, 1, 1, 0x45, 0x45, 0, 0, 0, 0x105, 0x105],
            // One per frame like the buttons, and every one different: what the file has to carry through is
            // the frame each of them belongs to, so a list that survived the round trip in the wrong order
            // would read as a playback parting on the first frame it was held against. **And one frame with
            // no word**, which is a frame this run played back rather than read a keyboard for.
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
            // None: this game's file carries no read of a frame before its last — see [`decode`].
            again: Vec::new(),
        }
    }

    /// Every field of it, since a resume is a run started from these numbers: one read back as another is a
    /// run that plays out differently and says nothing about why.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&saved())), Some(saved()));
    }

    /// **A file with no generators in it is refused**, which is what this version is for: a run whose frames
    /// have no word to be held against is a run nothing can say it is reproducing. One of those was picked up
    /// in stage 6 and parted somewhere in the sixty frames after 853 — the player was hit at 1249 and the run
    /// died out at 5900 of the 24119 frames it was aiming at — with nothing in the log able to name the frame.
    ///
    /// th06's own file reads them, because that game is released and its files are on disks — see
    /// `th06::resume`. This one is not, so nothing has to be traded.
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
    ///
    /// **Which is what a run left before this file was this game's own runs into**: those say version 2 and
    /// carry th06's rank, power items and extra lives, so they are refused twice over.
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
