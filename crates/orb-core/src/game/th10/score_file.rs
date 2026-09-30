//! **The score file of th10 as bytes**: how it is enciphered and packed, and how its records are walked — for
//! the one thing orb does with it that the game does not, which is to read the mode's own file into the score
//! object the game read once at startup.
//!
//! **Because this game reads its file once and never again.** `scoreth10.dat` is read by the loading object as
//! the game starts (0x42ae7d) into the object at `[0x47783c]`, and every screen after reads that object and
//! every scene change leaving a run writes the whole of it back (0x42b1e0 from the game object's teardown,
//! 0x417c8d). So orb's fork of the open reaches one read, made before the mode is asked, and what it read is
//! what both modes' writes are made of. See `Th10::read_what_the_modes_file_records`, which puts what this
//! reads into the object.
//!
//! **The file**, as 0x42b030 parses it: `TH10` and a version word of 3 at `+0x8`, the packed length at `+0x10`
//! and the unpacked at `+0x14`, and from `+0x18` the packed bytes, enciphered by 0x44b0d0 with a key of `0xac`
//! stepping by `0x35` over blocks of `0x10`, and packed by 0x435dc0 in an LZSS of a `0x2000`-byte window
//! — thirteen bits of offset, four of length over three, an offset of nought ending it.
//!
//! **What it unpacks to** is records one after another, each two letters, a version word of nought, a sum of
//! its bytes from `+0x8`, its size and then its own: `CR`, a shot type's record, `0x437c` bytes with its number
//! at `+0xc` (0x42b0a5 through 0x42b109), and `ST`, the settings and the clears, `0x448` bytes (0x42b10f through
//! 0x42b158). A record whose version, sum or size is not those is skipped, as the parse skips it.

/// The file's header: `TH10`, a version word at `+0x8`, the packed and the unpacked lengths.
const MAGIC: &[u8; 4] = b"TH10";
const VERSION_AT: usize = 0x8;
const VERSION: u16 = 3;
const PACKED_LENGTH_AT: usize = 0x10;
const UNPACKED_LENGTH_AT: usize = 0x14;
const HEADER: usize = 0x18;

/// The cipher's key, its step and its block, the arguments 0x42b05b through 0x42b062 hand 0x44b0d0.
const KEY: u8 = 0xac;
const STEP: u8 = 0x35;
const BLOCK: usize = 0x10;

/// The window the packing copies out of, and where it starts writing into it (0x435e04).
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

/// `CR`: one shot type's record, and how many there are — `character * 3 + shot`, and the seventh for all.
pub const SHOT_TYPES_RECORD_BYTES: usize = 0x437c;
pub const SHOT_TYPES_RECORDS: usize = 7;
const SHOT_TYPES_RECORD: &[u8; 2] = b"CR";
/// `ST`: the settings, and the clears the main menu lights the Extra from (`+0x1c`, 0x42c850).
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

/// 0x44b0d0 over the whole of `body`: each block written back to front, its odd places first, each byte
/// `xor`ed with the key and the key stepped. A tail shorter than a quarter of a block, and an odd last byte,
/// are left as they are.
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

/// 0x435dc0: a flag bit, then eight bits of a byte or thirteen of an offset into the window and four of a
/// length, every byte written also written into the window; `length` bytes at the most.
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

/// The packed bytes read a bit at a time from the top, reading nought past their end as 0x435e2c does.
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

/// A file of these records as the game would read it: packed as literals alone and enciphered the other way
/// about — for a laid-out game's directory, and for the tests.
#[cfg(any(test, feature = "sim"))]
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
    let mut file = Vec::with_capacity(HEADER + packed.len());
    file.extend_from_slice(MAGIC);
    file.extend_from_slice(&[0; 4]);
    file.extend_from_slice(&VERSION.to_le_bytes());
    file.extend_from_slice(&[0; 6]);
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
#[cfg(any(test, feature = "sim"))]
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

/// A record of `letters` and `size` bytes with its number, its own bytes being `fill`, summed as the game sums
/// one.
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
    let sum = sum(&record[RECORD_SUMMED_FROM..]);
    record[RECORD_SUM_AT..RECORD_SUM_AT + 4].copy_from_slice(&sum.to_le_bytes());
    record
}

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
    fn a_block_is_enciphered_back_to_front_odd_places_first() {
        let mut body: Vec<u8> = (0..0x10).collect();
        encipher(&mut body);
        assert_eq!(body[0], 0x0f ^ KEY);
        assert_eq!(body[1], 0x0d ^ KEY.wrapping_add(STEP));
        assert_eq!(body[8], 0x0e ^ KEY.wrapping_add(STEP.wrapping_mul(8)));
        decipher(&mut body);
        assert_eq!(body, (0..0x10).collect::<Vec<u8>>());
    }

    #[test]
    fn a_copy_out_of_the_window_repeats_what_was_written() {
        // `a`, then a copy of three from where the window's writing began.
        let bits: Vec<u8> = [1u8]
            .into_iter()
            .chain((0..8).rev().map(|shift| (b'a' >> shift) & 1))
            .chain([0])
            .chain((0..13).rev().map(|shift| (1u16 >> shift) as u8 & 1))
            .chain([0, 0, 0, 0])
            .chain([0; 14])
            .collect();
        let packed: Vec<u8> = bits
            .chunks(8)
            .map(|byte| {
                byte.iter()
                    .enumerate()
                    .fold(0, |value, (i, bit)| value | bit << (7 - i))
            })
            .collect();
        assert_eq!(unpack(&packed, 16), b"aaaa");
    }

    #[test]
    fn a_record_whose_sum_does_not_add_up_is_skipped() {
        let mut records = a_file();
        let mut turned = shot_types_record(3, 0x33);
        turned[0x100] ^= 1;
        records.shot_types[3] = Some(turned);
        let read = decode(&encode(&records)).expect("a file of th10's");
        assert_eq!(read.shot_types[3], None);
        assert_eq!(read.shot_types[6], a_file().shot_types[6]);
    }

    #[test]
    fn a_file_of_another_game_is_refused() {
        let mut file = encode(&a_file());
        file[3] = b'8';
        assert_eq!(decode(&file), None);
    }

    /// **The recipe for reading a file off a real install**: `ORB_TH10_SCORE_FILE=<its path> cargo xtask test
    /// score_file -- --ignored`, which says what it holds.
    #[test]
    #[ignore = "reads a file of a real install, named in ORB_TH10_SCORE_FILE"]
    fn a_file_of_a_real_install_reads() {
        let path = std::env::var("ORB_TH10_SCORE_FILE")
            .expect("usage: ORB_TH10_SCORE_FILE=<path of a scoreth10.dat> cargo xtask test score_file -- --ignored");
        let read = decode(&std::fs::read(path).expect("the file")).expect("a file of th10's");
        let held: Vec<usize> = (0..SHOT_TYPES_RECORDS)
            .filter(|number| read.shot_types[*number].is_some())
            .collect();
        println!("shot types' records {held:?}");
        let settings = read.settings.expect("the settings");
        println!("settings from +0x0c: {:02x?}", &settings[0xc..0x40]);
    }
}
