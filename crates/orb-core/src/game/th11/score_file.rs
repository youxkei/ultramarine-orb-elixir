//! **The score file of th11 as bytes**: how it is enciphered and packed, and how its records are walked — for the
//! one thing orb does with it that the game does not, which is to read the mode's own file into the score object
//! the game read once at startup.
//!
//! **Because this game reads its file once and never again**, as th10 does: `scoreth11.dat` is read by the loading
//! object as the game starts (0x437800, called last at 0x428b28) into the object at `[0x4a8ebc]`, and every screen
//! after reads that object and the game object's teardown writes the whole of it back (0x437a70 from 0x41fe17).
//!
//! **The file**, as 0x4378f0 parses it: `TH11` (`cmp [eax],0x31314854` at 0x437900) and a version word of 4 at
//! `+0x8` (0x43790c), the packed length at `+0x10` and the unpacked at `+0x14`, and from `+0x18` the packed bytes,
//! enciphered by 0x4581c0 with a key of `0xac` stepping by `0x35` over blocks of `0x10` (0x43791d through 0x437926),
//! and packed by 0x4426c0 — th10's LZSS, which unpacks the install's file of 2026-10-07 to exactly the length its
//! header names with every record's sum sound.
//!
//! **What it unpacks to** is th10's shape of records: `CR`, a shot type's record, `0x68d4` bytes with its number at
//! `+0xc`, and `ST`, the settings and the clears, `0x448` bytes.

/// The file's header: `TH11`, a version word at `+0x8`, the packed and the unpacked lengths.
const MAGIC: &[u8; 4] = b"TH11";
const VERSION_AT: usize = 0x8;
const VERSION: u16 = 4;
const PACKED_LENGTH_AT: usize = 0x10;
const UNPACKED_LENGTH_AT: usize = 0x14;
const HEADER: usize = 0x18;

/// The cipher's key, its step and its block (0x43791d through 0x437926).
const KEY: u8 = 0xac;
const STEP: u8 = 0x35;
const BLOCK: usize = 0x10;

/// The window the packing copies out of.
const WINDOW: usize = 0x2000;
const OFFSET_BITS: u32 = 13;
const LENGTH_BITS: u32 = 4;
const SHORTEST_COPY: usize = 3;

/// A record's two letters, its version, its sum, its size and — a `CR` — its number.
const RECORD_VERSION_AT: usize = 0x2;
const RECORD_SUM_AT: usize = 0x4;
const RECORD_SIZE_AT: usize = 0x8;
const RECORD_SUMMED_FROM: usize = 0x8;
const RECORD_NUMBER_AT: usize = 0xc;

/// `CR`: one shot type's record, and how many there are — `character * 3 + partner`, and the seventh for all.
pub const SHOT_TYPES_RECORD_BYTES: usize = 0x68d4;
pub const SHOT_TYPES_RECORDS: usize = 7;
const SHOT_TYPES_RECORD: &[u8; 2] = b"CR";
/// `ST`: the settings, the endings seen and the clears.
pub const SETTINGS_BYTES: usize = 0x448;
const SETTINGS: &[u8; 2] = b"ST";

/// What one file holds: each shot type's record by its number, and the settings.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Records {
    pub shot_types: [Option<Vec<u8>>; SHOT_TYPES_RECORDS],
    pub settings: Option<Vec<u8>>,
}

/// The records a file holds, or `None` for bytes that are not a file of this game's.
pub fn decode(file: &[u8]) -> Option<Records> {
    if file.len() < HEADER || &file[..MAGIC.len()] != MAGIC || word(file, VERSION_AT)? != VERSION {
        return None;
    }
    let packed = usize::try_from(dword(file, PACKED_LENGTH_AT)?).ok()?;
    let unpacked = usize::try_from(dword(file, UNPACKED_LENGTH_AT)?).ok()?;
    let mut body = file.get(HEADER..HEADER.checked_add(packed)?)?.to_vec();
    decipher(&mut body);
    let plain = unpack(&body, unpacked);
    let mut records = Records::default();
    let mut at = 0;
    while at + RECORD_NUMBER_AT <= plain.len() {
        let size = usize::try_from(dword(&plain, at + RECORD_SIZE_AT)?).ok()?;
        let Some(record) = plain.get(at..at.checked_add(size)?) else {
            break;
        };
        match (&record[..2], size) {
            (letters, SHOT_TYPES_RECORD_BYTES) if letters == SHOT_TYPES_RECORD && sound(record) => {
                let number = usize::try_from(dword(record, RECORD_NUMBER_AT)?).ok()?;
                // **Bounded here, where the game's own parse is not** (0x43799e through 0x4379b2): a number past
                // the seven is a file somebody edited, and the game's copy of it lands past the object.
                if let Some(slot) = records.shot_types.get_mut(number) {
                    *slot = Some(record.to_vec());
                }
            }
            (letters, SETTINGS_BYTES) if letters == SETTINGS && sound(record) => {
                records.settings = Some(record.to_vec());
            }
            _ => {}
        }
        if size == 0 {
            break;
        }
        at += size;
    }
    Some(records)
}

/// Whether a record's version is nought and its sum is the sum of its bytes from `+0x8`.
fn sound(record: &[u8]) -> bool {
    word(record, RECORD_VERSION_AT) == Some(0)
        && dword(record, RECORD_SUM_AT) == Some(sum(&record[RECORD_SUMMED_FROM..]))
}

fn sum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |sum, byte| sum.wrapping_add(u32::from(*byte)))
}

fn word(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn dword(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// 0x4581c0 over the whole of `body`: each block written back to front, its odd places first, each byte `xor`ed
/// with the key and the key stepped — th10's cipher.
fn decipher(body: &mut [u8]) {
    let enciphered = body.to_vec();
    let short_tail = match body.len() % BLOCK {
        tail if tail < BLOCK / 4 => tail,
        _ => 0,
    };
    let mut left = body.len() - short_tail - (body.len() & 1);
    let mut key = KEY;
    let (mut read, mut start) = (0, 0);
    while left > 0 {
        let block = left.min(BLOCK);
        let end = start + block;
        for place in (0..block.div_ceil(2)).map(|i| end - 1 - 2 * i) {
            body[place] = enciphered[read] ^ key;
            key = key.wrapping_add(STEP);
            read += 1;
        }
        for place in (0..block / 2).map(|i| end - 2 - 2 * i) {
            body[place] = enciphered[read] ^ key;
            key = key.wrapping_add(STEP);
            read += 1;
        }
        start = end;
        left -= block;
    }
}

/// 0x4426c0: a flag bit, then eight bits of a byte or thirteen of an offset into the window and four of a length,
/// every byte written also written into the window; `length` bytes at the most.
fn unpack(packed: &[u8], length: usize) -> Vec<u8> {
    let mut bits = Bits {
        bytes: packed,
        at: 0,
    };
    let mut window = [0u8; WINDOW];
    let mut head = 1;
    let mut plain = Vec::with_capacity(length);
    let mut put = |byte: u8, plain: &mut Vec<u8>, window: &mut [u8; WINDOW]| {
        plain.push(byte);
        window[head] = byte;
        head = (head + 1) % WINDOW;
    };
    while plain.len() < length {
        if bits.take(1) == 1 {
            let byte = bits.take(8) as u8;
            put(byte, &mut plain, &mut window);
            continue;
        }
        let offset = bits.take(OFFSET_BITS) as usize;
        if offset == 0 {
            break;
        }
        let copy = bits.take(LENGTH_BITS) as usize + SHORTEST_COPY;
        for i in 0..copy {
            if plain.len() == length {
                break;
            }
            let byte = window[(offset + i) % WINDOW];
            put(byte, &mut plain, &mut window);
        }
    }
    plain
}

/// The packed bytes read a bit at a time from the top, reading nought past their end.
struct Bits<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Bits<'_> {
    fn take(&mut self, count: u32) -> u32 {
        (0..count).fold(0, |value, _| {
            let bit = self
                .bytes
                .get(self.at / 8)
                .map_or(0, |byte| (byte >> (7 - self.at % 8)) & 1);
            self.at += 1;
            value << 1 | u32::from(bit)
        })
    }
}

/// A file of these records as the game writes one (0x437a70), the records as they are given, packed and enciphered
/// behind the header a file of its own carries — for a laid-out game's directory, for the tests, and for the score
/// file `orb-real` lays into a copy.
///
/// **Packed as literals alone and not by the game's own packing (0x4423a0)**, which chooses which runs to copy: the
/// game's parse (0x4378f0) unpacks either to the same records, and what a launch reads is the records.
#[cfg(any(test, feature = "sim", feature = "score-files"))]
pub fn encode(records: &Records) -> Vec<u8> {
    let plain: Vec<u8> = records
        .shot_types
        .iter()
        .flatten()
        .chain(records.settings.iter())
        .flatten()
        .copied()
        .collect();
    let mut bits = Vec::new();
    for byte in &plain {
        bits.push(1);
        bits.extend((0..8).rev().map(|shift| (byte >> shift) & 1));
    }
    bits.extend(std::iter::repeat_n(0, 1 + OFFSET_BITS as usize));
    let mut packed: Vec<u8> = bits
        .chunks(8)
        .map(|byte| {
            byte.iter()
                .enumerate()
                .fold(0, |value, (i, bit)| value | bit << (7 - i))
        })
        .collect();
    encipher(&mut packed);
    // The header 0x4378f0 makes where it reads no file, `+0xc` its `0x100`, and the file's length at `+0x4` as the
    // writer puts it (0x437a70).
    let mut file = Vec::with_capacity(HEADER + packed.len());
    file.extend_from_slice(MAGIC);
    file.extend_from_slice(
        &u32::try_from(HEADER + packed.len())
            .expect("a file of kilobytes")
            .to_le_bytes(),
    );
    file.extend_from_slice(&VERSION.to_le_bytes());
    file.extend_from_slice(&[0; 2]);
    file.extend_from_slice(&FRESH_HEADERS_WORD.to_le_bytes());
    file.extend_from_slice(
        &u32::try_from(packed.len())
            .expect("a file of kilobytes")
            .to_le_bytes(),
    );
    file.extend_from_slice(
        &u32::try_from(plain.len())
            .expect("a file of kilobytes")
            .to_le_bytes(),
    );
    file.extend_from_slice(&packed);
    file
}

/// [`decipher`] run backwards: each place read in the order it writes them.
#[cfg(any(test, feature = "sim", feature = "score-files"))]
fn encipher(body: &mut [u8]) {
    let plain = body.to_vec();
    let short_tail = match body.len() % BLOCK {
        tail if tail < BLOCK / 4 => tail,
        _ => 0,
    };
    let mut left = body.len() - short_tail - (body.len() & 1);
    let mut key = KEY;
    let (mut write, mut start) = (0, 0);
    while left > 0 {
        let block = left.min(BLOCK);
        let end = start + block;
        let places: Vec<usize> = (0..block.div_ceil(2))
            .map(|i| end - 1 - 2 * i)
            .chain((0..block / 2).map(|i| end - 2 - 2 * i))
            .collect();
        for place in places {
            body[write] = plain[place] ^ key;
            key = key.wrapping_add(STEP);
            write += 1;
        }
        start = end;
        left -= block;
    }
}

/// A record of `letters` and `size` bytes with its number, its own bytes being `fill`, summed as the game sums one.
#[cfg(any(test, feature = "sim"))]
pub fn record(letters: &[u8; 2], size: usize, number: u32, fill: u8) -> Vec<u8> {
    let mut record = vec![fill; size];
    record[..2].copy_from_slice(letters);
    record[RECORD_VERSION_AT..RECORD_VERSION_AT + 2].copy_from_slice(&0u16.to_le_bytes());
    record[RECORD_SIZE_AT..RECORD_SIZE_AT + 4].copy_from_slice(
        &u32::try_from(size)
            .expect("a record of kilobytes")
            .to_le_bytes(),
    );
    record[RECORD_NUMBER_AT..RECORD_NUMBER_AT + 4].copy_from_slice(&number.to_le_bytes());
    summed(record)
}

/// The record with its sum written in, the bytes from `+0x8` added up as 0x437500 adds them.
#[cfg(any(test, feature = "sim", feature = "score-files"))]
fn summed(mut record: Vec<u8>) -> Vec<u8> {
    let sum = sum(&record[RECORD_SUMMED_FROM..]);
    record[RECORD_SUM_AT..RECORD_SUM_AT + 4].copy_from_slice(&sum.to_le_bytes());
    record
}

/// The header word at `+0xc` of the file 0x4378f0 makes where it reads none, which the writer carries on.
#[cfg(any(test, feature = "sim", feature = "score-files"))]
const FRESH_HEADERS_WORD: u32 = 0x100;

/// **The records a game with no file of its own makes** (0x437740): seven shot types' records as 0x437550 empties
/// one and the settings as 0x437660 does, each numbered and summed as the writer numbers and sums them (0x437a70) —
/// what a fresh install's first teardown writes. `generator` is the word 0x437660 draws the settings' `0x200` words
/// from, `0x4c2f00` as `timeGetTime` seeded it.
#[cfg(any(test, feature = "score-files"))]
pub fn the_records_of_a_fresh_install(generator: u16) -> Records {
    let mut records = Records::default();
    for (number, slot) in records.shot_types.iter_mut().enumerate() {
        *slot = Some(an_empty_shot_types_record(number));
    }
    records.settings = Some(empty_settings(generator));
    records
}

/// 0x437550: nought, `CR` and its size, five difficulties' rankings of ten from 1,000,000 down by 100,000 under the
/// name at `0x4957ac`, and every card numbered with its difficulty out of `0x4a3e10`.
#[cfg(any(test, feature = "score-files"))]
fn an_empty_shot_types_record(number: usize) -> Vec<u8> {
    let mut record = vec![0; SHOT_TYPES_RECORD_BYTES];
    record[..2].copy_from_slice(SHOT_TYPES_RECORD);
    put_dword(&mut record, RECORD_SIZE_AT, SHOT_TYPES_RECORD_BYTES as u32);
    for difficulty in 0..RANKED_DIFFICULTIES {
        for place in 0..RANKED_PLACES {
            let at = RANKING_AT + (difficulty * RANKED_PLACES + place) * RANKED_ENTRY_BYTES;
            put_dword(
                &mut record,
                at,
                RANKED_SCORE_AT_THE_TOP - RANKED_SCORE_STEP * place as u32,
            );
            record[at + RANKED_STAGE_AT] = 1;
            record[at + RANKED_NAME_AT..at + RANKED_NAME_AT + RANKED_NAME.len()]
                .copy_from_slice(RANKED_NAME);
        }
    }
    for (card, difficulty) in CARD_DIFFICULTIES.iter().enumerate() {
        let at = CARDS_AT + card * CARD_BYTES;
        put_dword(&mut record, at + CARD_NUMBER_AT, card as u32);
        put_dword(&mut record, at + CARD_DIFFICULTY_AT, u32::from(*difficulty));
    }
    put_dword(&mut record, RECORD_NUMBER_AT, number as u32);
    summed(record)
}

/// 0x437660: nought, `ST` and its size, the name at `0x495244`, and `0x200` words drawn from the generator.
#[cfg(any(test, feature = "score-files"))]
fn empty_settings(mut generator: u16) -> Vec<u8> {
    let mut record = vec![0; SETTINGS_BYTES];
    record[..2].copy_from_slice(SETTINGS);
    put_dword(&mut record, RECORD_SIZE_AT, SETTINGS_BYTES as u32);
    record[SETTINGS_NAME_AT..SETTINGS_NAME_AT + SETTINGS_NAME.len()].copy_from_slice(SETTINGS_NAME);
    for word in 0..SETTINGS_DRAWS {
        // One draw of 0x458bc0, inlined at 0x4376d9 through 0x437721.
        generator = (generator ^ 0x9630).wrapping_sub(0x6553).rotate_left(2);
        let at = SETTINGS_DRAWS_AT + word * 2;
        record[at..at + 2].copy_from_slice(&generator.to_le_bytes());
    }
    summed(record)
}

/// **An ending seen, as the ending's set-up writes it** (0x40ec90): the byte of ending `character * 3 + partner`,
/// six more for a run with no continue, set to 1 and `| 0x10` above Easy, and the flag of an ending of six or more
/// seen — the bytes the main menu offers the Extra by (0x439350) and the selects offer a shot type by (0x439340).
#[cfg(any(test, feature = "score-files"))]
pub fn an_ending_seen(
    records: &mut Records,
    character: usize,
    partner: usize,
    continues: u32,
    difficulty: u32,
) {
    let Some(settings) = records.settings.as_mut() else {
        return;
    };
    let ending = character * 3 + partner + if continues == 0 { 6 } else { 0 };
    settings[ENDINGS_SEEN_AT + ending] = 1;
    if difficulty > 0 {
        settings[ENDINGS_SEEN_AT + ending] |= ENDING_SEEN_ABOVE_EASY;
    }
    if ending > 5 {
        settings[AN_ENDING_OF_SIX_OR_MORE_SEEN_AT] = 1;
    }
    *settings = summed(settings.clone());
}

/// **A stage won, as a run not replayed and not in the Extra marks it** (0x41eb39 through 0x41eb92): the practice's
/// entry `stage + difficulty * 6` of the shot type's record, the stage counted from 1, its `+0x4` and `+0x5` set to
/// 1 — the latter the byte the practice's stage select offers a stage by (0x43d80a).
#[cfg(any(test, feature = "score-files"))]
pub fn a_stage_won(
    records: &mut Records,
    character: usize,
    partner: usize,
    difficulty: usize,
    stage: usize,
) {
    let Some(Some(record)) = records.shot_types.get_mut(character * 3 + partner) else {
        return;
    };
    let at = PRACTICE_AT + (stage + difficulty * PRACTICE_STAGES) * PRACTICE_ENTRY_BYTES;
    record[at + PRACTICE_WON_AT] = 1;
    record[at + PRACTICE_OFFERED_AT] = 1;
    *record = summed(record.clone());
}

/// The practice (0x437550's record, read at 0x43d80a): 25 entries of 8 bytes from `+0x59c`, `stage + difficulty * 6`.
#[cfg(any(test, feature = "score-files"))]
const PRACTICE_AT: usize = 0x59c;
#[cfg(any(test, feature = "score-files"))]
const PRACTICE_STAGES: usize = 6;
#[cfg(any(test, feature = "score-files"))]
const PRACTICE_ENTRY_BYTES: usize = 8;
#[cfg(any(test, feature = "score-files"))]
const PRACTICE_WON_AT: usize = 0x4;
#[cfg(any(test, feature = "score-files"))]
const PRACTICE_OFFERED_AT: usize = 0x5;

#[cfg(any(test, feature = "score-files"))]
fn put_dword(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

/// A shot type's ranking (0x437550): from `+0x10`, ten places of `0x1c` bytes for each of five difficulties, the
/// score at `+0x0`, the stage reached at `+0x4` and the name at `+0x6`.
#[cfg(any(test, feature = "score-files"))]
const RANKING_AT: usize = 0x10;
#[cfg(any(test, feature = "score-files"))]
const RANKED_DIFFICULTIES: usize = 5;
#[cfg(any(test, feature = "score-files"))]
const RANKED_PLACES: usize = 10;
#[cfg(any(test, feature = "score-files"))]
const RANKED_ENTRY_BYTES: usize = 0x1c;
#[cfg(any(test, feature = "score-files"))]
const RANKED_STAGE_AT: usize = 0x4;
#[cfg(any(test, feature = "score-files"))]
const RANKED_NAME_AT: usize = 0x6;
#[cfg(any(test, feature = "score-files"))]
const RANKED_SCORE_AT_THE_TOP: u32 = 1_000_000;
#[cfg(any(test, feature = "score-files"))]
const RANKED_SCORE_STEP: u32 = 100_000;
/// The name a fresh ranking is under: `0x4957ac`'s eight and the nought at `0x4957b4`.
#[cfg(any(test, feature = "score-files"))]
const RANKED_NAME: &[u8; 9] = b"--------\0";

/// The cards (0x437550): 175 of `0x90` bytes from `+0x664`, each numbered at `+0x88` and its difficulty at `+0x8c`.
#[cfg(any(test, feature = "score-files"))]
const CARDS_AT: usize = 0x664;
#[cfg(any(test, feature = "score-files"))]
const CARD_BYTES: usize = 0x90;
#[cfg(any(test, feature = "score-files"))]
const CARD_NUMBER_AT: usize = 0x88;
#[cfg(any(test, feature = "score-files"))]
const CARD_DIFFICULTY_AT: usize = 0x8c;
/// **Each card's difficulty**, the 175 bytes at `0x4a3e10` (`objdump -s --start-address=0x4a3e10
/// --stop-address=0x4a3ebf th11.exe`): 0 to 3 round and round from Hard, and the Extra's thirteen at the end.
#[cfg(any(test, feature = "score-files"))]
const CARD_DIFFICULTIES: [u8; 175] = {
    let mut difficulties = [4u8; 175];
    let mut card = 0;
    while card < 162 {
        difficulties[card] = [2, 3, 0, 1][card % 4];
        card += 1;
    }
    difficulties
};

/// The settings (0x437660): the name at `+0xc`, `0x495244`'s eight spaces and the nought at `0x49524c`, and the
/// generator's `0x200` words from `+0x46`.
#[cfg(any(test, feature = "score-files"))]
const SETTINGS_NAME_AT: usize = 0xc;
#[cfg(any(test, feature = "score-files"))]
const SETTINGS_NAME: &[u8; 9] = b"        \0";
#[cfg(any(test, feature = "score-files"))]
const SETTINGS_DRAWS_AT: usize = 0x46;
#[cfg(any(test, feature = "score-files"))]
const SETTINGS_DRAWS: usize = 0x200;
/// The endings seen (0x40ec90): a byte an ending from `+0x16`, `| 0x10` above Easy, and the flag at `+0x22`.
#[cfg(any(test, feature = "score-files"))]
const ENDINGS_SEEN_AT: usize = 0x16;
#[cfg(any(test, feature = "score-files"))]
const ENDING_SEEN_ABOVE_EASY: u8 = 0x10;
#[cfg(any(test, feature = "score-files"))]
const AN_ENDING_OF_SIX_OR_MORE_SEEN_AT: usize = 0x22;

/// A shot type's record, numbered.
#[cfg(any(test, feature = "sim"))]
pub fn shot_types_record(number: u32, fill: u8) -> Vec<u8> {
    record(SHOT_TYPES_RECORD, SHOT_TYPES_RECORD_BYTES, number, fill)
}

/// The settings.
#[cfg(any(test, feature = "sim"))]
pub fn settings(fill: u8) -> Vec<u8> {
    record(SETTINGS, SETTINGS_BYTES, 0, fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_file() -> Records {
        let mut records = Records::default();
        records.shot_types[0] = Some(shot_types_record(0, 0x11));
        records.shot_types[6] = Some(shot_types_record(6, 0x66));
        records.settings = Some(settings(0x5a));
        records
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&a_file())), Some(a_file()));
    }

    #[test]
    fn a_record_whose_sum_does_not_add_up_is_skipped() {
        let mut records = a_file();
        let mut turned = shot_types_record(3, 0x33);
        turned[0x100] ^= 1;
        records.shot_types[3] = Some(turned);
        let read = decode(&encode(&records)).expect("a file of th11's");
        assert_eq!(read.shot_types[3], None);
        assert_eq!(read.shot_types[6], a_file().shot_types[6]);
    }

    #[test]
    fn a_fresh_installs_records_are_read_back_whole_and_sound() {
        let records = the_records_of_a_fresh_install(0x1234);
        assert_eq!(decode(&encode(&records)), Some(records));
    }

    #[test]
    fn a_fresh_installs_rankings_go_down_from_a_million_and_its_cards_are_numbered() {
        let records = the_records_of_a_fresh_install(0);
        let record = records.shot_types[2]
            .as_ref()
            .expect("a shot type's record");
        assert_eq!(dword(record, RECORD_NUMBER_AT), Some(2));
        assert_eq!(dword(record, RANKING_AT), Some(1_000_000));
        assert_eq!(
            dword(record, RANKING_AT + 9 * RANKED_ENTRY_BYTES),
            Some(100_000)
        );
        assert_eq!(
            &record[RANKING_AT + RANKED_NAME_AT..RANKING_AT + RANKED_NAME_AT + 9],
            b"--------\0"
        );
        let the_last_card = CARDS_AT + 174 * CARD_BYTES;
        assert_eq!(dword(record, the_last_card + CARD_NUMBER_AT), Some(174));
        assert_eq!(dword(record, the_last_card + CARD_DIFFICULTY_AT), Some(4));
    }

    #[test]
    fn a_normal_clear_with_no_continue_lights_the_ending_the_extra_is_offered_by() {
        let mut records = the_records_of_a_fresh_install(0);
        an_ending_seen(&mut records, 0, 0, 0, 1);
        let read = decode(&encode(&records)).expect("a file of th11's");
        let settings = read.settings.expect("the settings");
        assert_eq!(settings[ENDINGS_SEEN_AT + 6], 0x11);
        assert_eq!(settings[AN_ENDING_OF_SIX_OR_MORE_SEEN_AT], 1);
    }

    #[test]
    fn a_stage_won_offers_it_to_the_practice_of_its_shot_type_and_difficulty_alone() {
        let mut records = the_records_of_a_fresh_install(0);
        a_stage_won(&mut records, 1, 2, 1, 3);
        let offered = |records: &Records, number: usize, entry: usize| {
            records.shot_types[number]
                .as_ref()
                .expect("a shot type's record")
                [PRACTICE_AT + entry * PRACTICE_ENTRY_BYTES + PRACTICE_OFFERED_AT]
        };
        assert_eq!(offered(&records, 5, 3 + 6), 1);
        assert_eq!(offered(&records, 5, 3), 0);
        assert_eq!(offered(&records, 0, 3 + 6), 0);
        assert_eq!(decode(&encode(&records)), Some(records));
    }

    #[test]
    fn an_easy_clear_lights_no_ending_above_easy() {
        let mut records = the_records_of_a_fresh_install(0);
        an_ending_seen(&mut records, 1, 2, 0, 0);
        let settings = records.settings.expect("the settings");
        assert_eq!(settings[ENDINGS_SEEN_AT + 11], 1);
    }

    #[test]
    fn a_file_of_another_game_is_refused() {
        let mut file = encode(&a_file());
        file[3] = b'0';
        assert_eq!(decode(&file), None);
    }

    /// **The recipe for reading a file off a real install**: `ORB_TH11_SCORE_FILE=<its path> cargo xtask test
    /// score_file -- --ignored`, which says what it holds.
    #[test]
    #[ignore = "reads a file of a real install, named in ORB_TH11_SCORE_FILE"]
    fn a_file_of_a_real_install_reads() {
        let path = std::env::var("ORB_TH11_SCORE_FILE").expect(
            "usage: ORB_TH11_SCORE_FILE=<path of a scoreth11.dat> cargo xtask test score_file -- --ignored",
        );
        let read = decode(&std::fs::read(path).expect("the file")).expect("a file of th11's");
        let held: Vec<usize> = (0..SHOT_TYPES_RECORDS)
            .filter(|number| read.shot_types[*number].is_some())
            .collect();
        println!("shot types' records {held:?}");
        assert!(read.settings.is_some(), "the settings");
    }
}
