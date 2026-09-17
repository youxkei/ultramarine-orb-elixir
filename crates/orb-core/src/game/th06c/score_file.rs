//! **The score file of th06c as bytes**: how it is enciphered, how its records are walked, and how one is
//! put together again — for the one thing orb does to it that the game does not, which is to put the copy the
//! game reads once at startup together out of two files.
//!
//! **Because this game reads its file once and never again.** th06 1.02h and th07 open `score.dat` at every
//! screen that wants it, so orb's fork of that open — `crate::score` — sends each read to whichever file the
//! mode points at. th06c opens it at `0x14002c158`, at startup, into a buffer at `0x140b77858` that every
//! later parse decrypts and every write is rebuilt from (`th06c/asm.md`, the score file). So the fork reaches
//! one read, made before anybody has been asked which mode the launch is, and what that read landed in is the
//! file both modes' writes are built on. Which would be the game's own file written back out of orb's or the
//! other way about — the thing the fork exists to stop.
//!
//! So the copy is put together here instead, the way the fork would have put it together read by read: the
//! records of runs — the rankings, the card records, the practice scores — out of the mode's own file, and the
//! `CLRD` records that light the front end's items out of the game's, since a stage reached is a stage reached
//! whichever mode reached it. See `Th06c::read_what_the_modes_file_records`, which installs what this builds.
//!
//! **The cipher is th06 1.02h's own**, read off the parse at `0x14004e5c4` and the write at `0x140055500`:
//! from the third byte on, each byte is `xor`ed with a key that is the key before plus the plain byte before,
//! rotated right by five — the second byte seeding it — and a sixteen-bit sum of the plain bytes from the
//! fifth on sits at the third, under the cipher.

/// The header every file begins with: a byte, the key's seed, the checksum, `0x10`, a byte of noise, the
/// offset of the first record as eight bytes, and the file's length — `0x14` bytes, `0x14` being that offset.
const HEADER: usize = 0x14;
const KEY_SEED: usize = 0x1;
const CHECKSUM: usize = 0x2;
const FIRST_RECORD: usize = 0x8;
const LENGTH: usize = 0x10;
/// Where the cipher and the checksum begin.
const ENCIPHERED_FROM: usize = 2;
const SUMMED_FROM: usize = 4;

/// One record, as the writer at `0x140054aae` lays it: four letters, its size twice as words, a version word
/// of `0x10`, and its own bytes from `+0xc` — walked by the first size at `0x14004e620`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Record {
    pub bytes: Vec<u8>,
}

/// Where a record's own bytes begin, after the four letters, the two sizes and the version.
const RECORD_HEADER: usize = 0xc;
const RECORD_VERSION: u32 = 0x10;

impl Record {
    pub fn magic(&self) -> &[u8] {
        &self.bytes[..4.min(self.bytes.len())]
    }
}

/// A record of `magic` holding `body` after its header.
pub fn record(magic: &[u8; 4], body: &[u8]) -> Record {
    let size = u16::try_from(RECORD_HEADER + body.len()).expect("a record of under 64 KB");
    let mut bytes = Vec::with_capacity(RECORD_HEADER + body.len());
    bytes.extend_from_slice(magic);
    bytes.extend_from_slice(&size.to_le_bytes());
    bytes.extend_from_slice(&size.to_le_bytes());
    bytes.extend_from_slice(&RECORD_VERSION.to_le_bytes());
    bytes.extend_from_slice(body);
    Record { bytes }
}

/// The record every file opens with, `TH6K`: its header and nothing after it.
pub fn th6k() -> Record {
    record(b"TH6K", &[])
}

/// A `CLRD` record for one shot — `character * 2 + shot` — with the stage each difficulty has reached with
/// retries and without, [`CLEARED`] being a clear: `0x18` bytes, the two arrays at `+0xc` and `+0x11`, the
/// shot at `+0x16`. See `th06c/asm.md`.
pub fn clrd(shot: u8, reached_with_retries: [u8; 5], reached_without: [u8; 5]) -> Record {
    let mut body = Vec::with_capacity(12);
    body.extend_from_slice(&reached_with_retries);
    body.extend_from_slice(&reached_without);
    body.push(shot);
    body.push(0);
    record(b"CLRD", &body)
}

/// Whether any `CLRD` among `records` says a run has cleared the game: what the title reads at `0x14002d436`
/// to offer the Extra — see `th06c/asm.md`.
pub fn a_clear_is_among(records: &[Record]) -> bool {
    records.iter().any(|record| {
        record.magic() == b"CLRD"
            && record.bytes.len() >= 0x16
            && record.bytes[0xc..0x16].contains(&CLEARED)
    })
}

/// What a clear is written as in those arrays, `cmp BYTE PTR [...],0x63` at `0x14002d436`.
pub const CLEARED: u8 = 0x63;

/// The records a file holds, or `None` for bytes that are not a file of this game's: too short, a checksum
/// that does not add up, or records that walk off the end.
pub fn decode(bytes: &[u8]) -> Option<Vec<Record>> {
    if bytes.len() < HEADER {
        return None;
    }
    let mut plain = bytes.to_vec();
    let mut key = 0u8;
    let mut previous = plain[KEY_SEED];
    for byte in plain.iter_mut().skip(ENCIPHERED_FROM) {
        key = key.wrapping_add(previous).rotate_right(5);
        *byte ^= key;
        previous = *byte;
    }
    let summed = plain[SUMMED_FROM..]
        .iter()
        .fold(0u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
    if summed != u16::from_le_bytes([plain[CHECKSUM], plain[CHECKSUM + 1]]) {
        return None;
    }
    let first = usize::try_from(i32::from_le_bytes(
        plain[FIRST_RECORD..FIRST_RECORD + 4].try_into().ok()?,
    ))
    .ok()?;
    let length = usize::try_from(i32::from_le_bytes(
        plain[LENGTH..LENGTH + 4].try_into().ok()?,
    ))
    .ok()?;
    if length > plain.len() || first > length {
        return None;
    }
    let mut records = Vec::new();
    let mut at = first;
    while at < length {
        if at + 8 > length {
            return None;
        }
        let size = usize::from(u16::from_le_bytes([plain[at + 4], plain[at + 5]]));
        if size < 8 || at + size > length {
            return None;
        }
        records.push(Record {
            bytes: plain[at..at + size].to_vec(),
        });
        at += size;
    }
    Some(records)
}

/// A file out of records, enciphered as the game writes one: the header, the records one after another, the
/// checksum over them and the cipher from the third byte on. `seed` is what the game draws from its generator
/// for the key's seed and the byte of noise; any value reads back.
pub fn encode(records: &[Record], seed: u8) -> Vec<u8> {
    let mut plain = vec![0u8; HEADER];
    plain[KEY_SEED] = seed;
    plain[4] = 0x10;
    plain[6] = seed;
    plain[FIRST_RECORD..FIRST_RECORD + 8].copy_from_slice(&(HEADER as u64).to_le_bytes());
    for record in records {
        plain.extend_from_slice(&record.bytes);
    }
    let length = u32::try_from(plain.len()).expect("a score file of a few kilobytes");
    plain[LENGTH..LENGTH + 4].copy_from_slice(&length.to_le_bytes());
    let summed = plain[SUMMED_FROM..]
        .iter()
        .fold(0u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
    plain[CHECKSUM..CHECKSUM + 2].copy_from_slice(&summed.to_le_bytes());
    let mut key = 0u8;
    let mut previous = plain[KEY_SEED];
    for byte in plain.iter_mut().skip(ENCIPHERED_FROM) {
        key = key.wrapping_add(previous).rotate_right(5);
        let this = *byte;
        *byte = this ^ key;
        previous = this;
    }
    plain
}

/// **The copy the game is to read as the mode's**: every record of runs out of the mode's own file, and the
/// `CLRD` records out of the game's — see the module doc. `None` for either file is a file that is not there,
/// which for the mode's own is a mode nothing has been written in yet.
///
/// The `TH6K` the file opens with comes first whichever file it is taken from, and is made where neither
/// holds one.
pub fn the_copy_the_mode_reads(own: Option<&[u8]>, modes: Option<&[u8]>) -> Vec<u8> {
    let own = own.and_then(decode).unwrap_or_default();
    let modes = modes.and_then(decode).unwrap_or_default();
    let mut records = vec![th6k()];
    records.extend(
        modes
            .iter()
            .filter(|record| record.magic() != b"TH6K" && record.magic() != b"CLRD")
            .cloned(),
    );
    records.extend(
        own.iter()
            .filter(|record| record.magic() == b"CLRD")
            .cloned(),
    );
    encode(&records, 0x5a)
}

#[cfg(test)]
mod tests {
    use super::{
        CLEARED, Record, a_clear_is_among, clrd, decode, encode, record, th6k,
        the_copy_the_mode_reads,
    };

    fn hscr(score: u32) -> Record {
        let mut body = score.to_le_bytes().to_vec();
        body.extend_from_slice(&[0, 1, 0, b'O', b'R', b'B', 0, 0, 0, 0, 0, 0]);
        record(b"HSCR", &body)
    }

    /// A record is its letters, its size twice, the version and its bytes — `TH6K` being twelve bytes of
    /// header alone, and a `CLRD` the `0x18` the exe walks past.
    #[test]
    fn a_record_is_laid_out_as_the_exe_walks_one() {
        assert_eq!(th6k().bytes, b"TH6K\x0c\x00\x0c\x00\x10\x00\x00\x00");
        let clrd = clrd(3, [1, 2, 3, 4, 5], [1, 1, 1, 1, 1]);
        assert_eq!(clrd.bytes.len(), 0x18);
        assert_eq!(&clrd.bytes[..0xc], b"CLRD\x18\x00\x18\x00\x10\x00\x00\x00");
        assert_eq!(clrd.bytes[0x16], 3);
    }

    /// A clear is a `CLRD` whose stages reached hold the clear's own byte, in either array.
    #[test]
    fn a_clear_is_read_off_the_clrd_records() {
        assert!(!a_clear_is_among(&[th6k(), clrd(0, [1; 5], [1; 5])]));
        assert!(a_clear_is_among(&[
            th6k(),
            clrd(0, [1, CLEARED, 1, 1, 1], [1; 5])
        ]));
        assert!(a_clear_is_among(&[clrd(1, [1; 5], [1, 1, 1, CLEARED, 1])]));
    }

    /// Every byte of every record comes back out of the file it was written into, whichever seed the cipher
    /// started from.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        let records = vec![
            th6k(),
            hscr(1_234_560),
            clrd(0, [1, CLEARED, 1, 1, 1], [1, 1, 1, 1, 1]),
        ];
        for seed in [0x00, 0x5a, 0xff] {
            assert_eq!(decode(&encode(&records, seed)), Some(records.clone()));
        }
    }

    /// A byte turned in the file is a file that is not this game's: the checksum says so.
    #[test]
    fn a_file_with_a_byte_turned_is_refused() {
        let mut file = encode(&[th6k(), hscr(7)], 0x5a);
        file[0x20] ^= 0x01;
        assert_eq!(decode(&file), None);
        assert_eq!(decode(b"TH6K"), None);
    }

    /// **The copy is the mode's runs under the game's own unlocks**: the rankings out of the mode's file, the
    /// `CLRD` records out of the game's, and the header record first.
    #[test]
    fn the_copy_takes_the_runs_from_the_modes_file_and_the_unlocks_from_the_games() {
        let own = encode(
            &[
                th6k(),
                hscr(999),
                clrd(0, [1, CLEARED, 1, 1, 1], [1, 1, 1, 1, 1]),
            ],
            0x11,
        );
        let modes = encode(
            &[th6k(), hscr(5), clrd(0, [1, 1, 1, 1, 1], [1, 1, 1, 1, 1])],
            0x22,
        );
        let copy = decode(&the_copy_the_mode_reads(Some(&own), Some(&modes))).expect("a file");
        assert_eq!(
            copy,
            vec![
                th6k(),
                hscr(5),
                clrd(0, [1, CLEARED, 1, 1, 1], [1, 1, 1, 1, 1])
            ]
        );
    }

    /// A mode nothing has been written in yet is a file of the game's unlocks and no runs; and no game's file
    /// at all is a file of the header alone, which the game reads as a first launch.
    #[test]
    fn a_mode_with_no_file_yet_reads_as_the_unlocks_and_no_runs() {
        let own = encode(&[th6k(), hscr(999), clrd(1, [1; 5], [1; 5])], 0x11);
        assert_eq!(
            decode(&the_copy_the_mode_reads(Some(&own), None)),
            Some(vec![th6k(), clrd(1, [1; 5], [1; 5])])
        );
        assert_eq!(
            decode(&the_copy_the_mode_reads(None, None)),
            Some(vec![th6k()])
        );
    }
}
