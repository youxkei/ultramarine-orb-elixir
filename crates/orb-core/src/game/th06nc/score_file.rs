//! **The score file of th06nc as bytes**: how it is enciphered, how its records are walked, and how one
//! is put together again — for the one thing orb does to it that the game does not, which is to put the
//! copy the game reads once at startup together out of two files.
//!
//! **Because this game reads its file once and never again**, as th06c does. `0x14006ca60` answers the
//! file as one parsed block and five places ask it, but what it parses each time is the raw bytes at
//! `[0x140c22148]` with their length in `[0x140c21f6c]` — read once — so the fork of the game's own
//! `fopen` (`crate::score`) reaches one read, made before anybody has been asked which mode the launch
//! is. What that read landed in is the file both modes' writes would be built on, which is the thing the
//! fork exists to stop.
//!
//! So the copy is put together here instead, the way the fork would have put it together read by read:
//! the records of runs out of the mode's own file, and the `CLRD` records that light the front end's
//! items out of the game's, since a stage reached is a stage reached whichever mode reached it.
//!
//! **The header and the cipher are th06c's byte for byte**, read off `0x14006ca60`: the key is seeded
//! from the byte at `+1`, every byte from `+2` on is `xor`ed with the key and the key is then the key
//! plus the plain byte just decrypted rotated right by five (`add r9b,cl; ror r9b,5` at `0x14006cad0`),
//! a sixteen-bit sum of the plain bytes from `+4` on sits in the word at `+2`, `+8` is the offset the
//! first record is at and `+0x10` the file's length, and the first record must be `TH6K`
//! (`0x14006cb20`). What differs is the record version — `0x15` where th06c's is `0x10` — and the
//! bodies.

/// The header every file begins with: a byte, the key's seed, the checksum, `0x10`, a byte of noise, the
/// offset of the first record as eight bytes, and the file's length — `0x14` bytes, `0x14` being that
/// offset.
const HEADER: usize = 0x14;
const KEY_SEED: usize = 0x1;
const CHECKSUM: usize = 0x2;
const FIRST_RECORD: usize = 0x8;
const LENGTH: usize = 0x10;
/// Where the cipher and the checksum begin.
const ENCIPHERED_FROM: usize = 2;
const SUMMED_FROM: usize = 4;

/// One record: four letters, its size twice as words, the version, and its own bytes from `+0xc` —
/// walked by the first size, `0x14006cb28` advancing by the word at `+0x4`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Record {
    pub bytes: Vec<u8>,
}

/// Where a record's own bytes begin, after the four letters, the two sizes and the version.
const RECORD_HEADER: usize = 0xc;

/// **The version, `0x15`.** Written as the one byte at `+0x8` — `mov BYTE PTR [rsi+0x8],0x15` at
/// `0x14003ab5f` for a `CATK` and `0x14003adb4` for a `CLRD` — and what the three bytes after it hold
/// is not read: a `CLRD` is memset before its header goes in, so they are nought there, and a `CATK` is
/// laid over `0xc0` words of the generator's, so they are noise. Nothing of the exe reads them; a record
/// built here writes nought, which is what a `CLRD` of the game's own holds.
const RECORD_VERSION: u32 = 0x15;

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

/// The record every file opens with, `TH6K`: its header and nothing after it — the one `0x14006cb20`
/// looks for before it walks any of the rest.
pub fn th6k() -> Record {
    record(b"TH6K", &[])
}

/// A `CLRD` record for one difficulty, with the stage each shot has reached with retries and without,
/// [`CLEARED`] being a clear: `0x18` bytes, the two arrays of five at `+0xc` and `+0x11` and the
/// difficulty at `+0x16`.
///
/// **Read off both ends.** The game builds four of them in memory at `0x14003ad90` — `0x18` bytes, the
/// version at `+0x8`, `0x01010101` into `+0xc` and `+0x11` and `1` into `+0x10` and `+0x15` — lays the
/// file's own over them by that `+0x16` at `0x14003adfc`, writes a stage into the two arrays at
/// `0x14003afaa` and `0x14003afda` as a stage is counted on, and writes [`CLEARED`] into them at
/// `0x14002ca5d` and `0x14002ca9e`.
pub fn clrd(difficulty: u8, reached_with_retries: [u8; 5], reached_without: [u8; 5]) -> Record {
    let mut body = Vec::with_capacity(RECORD_HEADER);
    body.extend_from_slice(&reached_with_retries);
    body.extend_from_slice(&reached_without);
    body.push(difficulty);
    body.push(0);
    record(b"CLRD", &body)
}

/// What a clear is written as in those arrays, `cmp BYTE PTR [...],0x63` at `0x14002ca3f`.
pub const CLEARED: u8 = 0x63;

/// One card's record: `0x180` bytes, and where in a record's body the four fields the file's own read
/// lays down sit — the record's `+0x10`, `+0x12`, `+0x18` and `+0x3c` less [`RECORD_HEADER`].
pub const CARD_BYTES: usize = 0x180;
const CATK_INDEX: usize = 0x10 - RECORD_HEADER;
const CATK_NAME_SUM: usize = 0x12 - RECORD_HEADER;
const CATK_NAME: usize = 0x18 - RECORD_HEADER;
const CATK_ATTEMPTS: usize = 0x3c - RECORD_HEADER;
const CATK_NAME_BYTES: usize = CATK_ATTEMPTS - CATK_NAME;

/// A `CATK` record for one card: the card's own index at `+0x10` as a word, the low byte of the name's
/// bytes' sum at `+0x12`, the name from `+0x18` with its terminator, and the attempts at `+0x3c`.
///
/// **Read off the read and not off a write, this build serialising none of them** — see
/// `Th06nc::writes_its_own_records`. The layout at `0x14003ab00` writes the magic, `0x180`/`0x180`, the
/// version and the index into all 134 records in memory, and `0x14006cc40` lays a file's own over them:
/// `0x180` bytes off the magic into the record the word at `+0x10` names. So what a file holding a count
/// gives the game is one of these, and it is `Image::the_files_card_records_are_laid_over` that lays it.
///
/// **The name is in it because the count is only read beside one**: `0x140027af0` holds the sum at
/// `+0x12` against the name at `+0x18`, and where the two disagree `0x140027afa` puts the count back to
/// nought — so a record carrying attempts and no name is a record the game clears.
///
/// # Panics
/// If the name does not fit [`CATK_NAME_BYTES`] with its terminator.
pub fn catk(card: u16, name: &str, attempts: u16) -> Record {
    let mut body = vec![0u8; CARD_BYTES - RECORD_HEADER];
    body[CATK_INDEX..CATK_INDEX + 2].copy_from_slice(&card.to_le_bytes());
    let named = name.as_bytes();
    assert!(
        named.len() < CATK_NAME_BYTES,
        "a name of {} bytes does not fit a record's {CATK_NAME_BYTES} with its terminator",
        named.len()
    );
    body[CATK_NAME..CATK_NAME + named.len()].copy_from_slice(named);
    body[CATK_NAME_SUM] = named.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
    body[CATK_ATTEMPTS..CATK_ATTEMPTS + 2].copy_from_slice(&attempts.to_le_bytes());
    record(b"CATK", &body)
}

/// Whether any `CLRD` among `records` says a run has cleared the game, which is what the front end reads
/// to offer the Extra: `0x14002bfc0` onwards walks the four records in memory at stride `0x18` from
/// `0x1404ff100` — the first one's `+0xc` — and asks `0x63` of both arrays of each.
pub fn a_clear_is_among(records: &[Record]) -> bool {
    records.iter().any(|record| {
        record.magic() == b"CLRD"
            && record.bytes.len() >= 0x16
            && record.bytes[0xc..0x16].contains(&CLEARED)
    })
}

/// The records a file holds, or `None` for bytes that are not a file of this game's: too short, a
/// checksum that does not add up, or records that walk off the end.
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

/// A file out of records, enciphered as the game reads one: the header, the records one after another,
/// the checksum over them and the cipher from the third byte on. `seed` is what the key and the byte of
/// noise start from; any value reads back.
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

/// **The copy the game is to read as the mode's**: every record of runs out of the mode's own file, and
/// the `CLRD` records out of the game's — see the module doc. `None` for either file is a file that is
/// not there, which for the mode's own is a mode nothing has been written in yet.
///
/// The `TH6K` the file opens with comes first whichever file it is taken from, and is made where neither
/// holds one — `0x14006cb20` walking nothing without it.
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
        CARD_BYTES, CLEARED, RECORD_VERSION, Record, a_clear_is_among, catk, clrd, decode, encode,
        record, th6k, the_copy_the_mode_reads,
    };

    /// A ranking record with the four fields of its body that are read: the score as a qword at `+0xc`,
    /// the shot at `+0x14`, the difficulty at `+0x15` and the mode at `+0x16` — the four the stage's own
    /// build matches on at `0x14003aca0` onwards. The `0x78` bytes are what `0x14006c52d` builds one as;
    /// what the rest of them hold is not read, so they are nought here.
    fn hscr(score: u64, shot: u8, difficulty: u8, mode: u8) -> Record {
        let mut body = score.to_le_bytes().to_vec();
        body.extend_from_slice(&[shot, difficulty, mode]);
        body.resize(0x78 - 0xc, 0);
        record(b"HSCR", &body)
    }

    /// A record is its letters, its size twice, the version and its bytes — `TH6K` being twelve bytes of
    /// header alone, and a `CLRD` the `0x18` the exe walks past.
    #[test]
    fn a_record_is_laid_out_as_the_exe_walks_one() {
        assert_eq!(th6k().bytes, b"TH6K\x0c\x00\x0c\x00\x15\x00\x00\x00");
        let clrd = clrd(3, [1, 2, 3, 4, 5], [1, 1, 1, 1, 1]);
        assert_eq!(clrd.bytes.len(), 0x18);
        assert_eq!(&clrd.bytes[..0xc], b"CLRD\x18\x00\x18\x00\x15\x00\x00\x00");
        assert_eq!(clrd.bytes[0x16], 3);
        // And a ranking record is the `0x78` the writer builds, with its score in eight bytes.
        assert_eq!(hscr(5_000_000_000, 1, 2, 0).bytes.len(), 0x78);
    }

    /// **A card's record is the `0x180` the read lays down**, with its four fields where the record's own
    /// offsets put them: the index at `+0x10`, the name's sum at `+0x12`, the name at `+0x18` and the
    /// attempts at `+0x3c`.
    #[test]
    fn a_card_record_is_laid_out_as_the_read_lays_one_down() {
        let card = catk(17, "ABC", 7);
        assert_eq!(card.bytes.len(), CARD_BYTES);
        assert_eq!(&card.bytes[..0xc], b"CATK\x80\x01\x80\x01\x15\x00\x00\x00");
        assert_eq!(u16::from_le_bytes([card.bytes[0x10], card.bytes[0x11]]), 17);
        assert_eq!(card.bytes[0x12], b'A' + b'B' + b'C');
        assert_eq!(&card.bytes[0x18..0x1c], b"ABC\0");
        assert_eq!(u16::from_le_bytes([card.bytes[0x3c], card.bytes[0x3d]]), 7);
    }

    /// And it goes into a file and comes back out of one whole, which is what a laid-out game reads it
    /// back through.
    #[test]
    fn a_card_record_reads_back_out_of_a_file() {
        let records = vec![th6k(), catk(3, "「ムーンライトレイ」", 9999)];
        assert_eq!(decode(&encode(&records, 0x5a)), Some(records));
    }

    /// **The version is this build's `0x15` and not th06c's `0x10`**, which is the byte every record of
    /// the file carries and the one the exe checks before it takes a record for its own: `0x14003aca8`
    /// asks it of an `HSCR` and `0x14003adee` of a `CLRD`.
    #[test]
    fn the_version_in_a_record_is_this_builds_own() {
        assert_eq!(RECORD_VERSION, 0x15);
        assert_eq!(th6k().bytes[8], 0x15);
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

    /// Every byte of every record comes back out of the file it was written into, whichever seed the
    /// cipher started from.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        let records = vec![
            th6k(),
            hscr(5_000_000_000, 1, 2, 0),
            clrd(0, [1, CLEARED, 1, 1, 1], [1, 1, 1, 1, 1]),
        ];
        for seed in [0x00, 0x5a, 0xff] {
            assert_eq!(decode(&encode(&records, seed)), Some(records.clone()));
        }
    }

    /// A byte turned in the file is a file that is not this game's: the checksum says so.
    #[test]
    fn a_file_with_a_byte_turned_is_refused() {
        let mut file = encode(&[th6k(), hscr(7, 0, 0, 0)], 0x5a);
        file[0x20] ^= 0x01;
        assert_eq!(decode(&file), None);
        assert_eq!(decode(b"TH6K"), None);
    }

    /// **The copy is the mode's runs under the game's own unlocks**: the rankings out of the mode's file,
    /// the `CLRD` records out of the game's, and the header record first.
    #[test]
    fn the_copy_takes_the_runs_from_the_modes_file_and_the_unlocks_from_the_games() {
        let own = encode(
            &[
                th6k(),
                hscr(999, 0, 1, 0),
                clrd(0, [1, CLEARED, 1, 1, 1], [1, 1, 1, 1, 1]),
            ],
            0x11,
        );
        let modes = encode(
            &[
                th6k(),
                hscr(5, 0, 1, 1),
                clrd(0, [1, 1, 1, 1, 1], [1, 1, 1, 1, 1]),
            ],
            0x22,
        );
        let copy = decode(&the_copy_the_mode_reads(Some(&own), Some(&modes))).expect("a file");
        assert_eq!(
            copy,
            vec![
                th6k(),
                hscr(5, 0, 1, 1),
                clrd(0, [1, CLEARED, 1, 1, 1], [1, 1, 1, 1, 1])
            ]
        );
    }

    /// A mode nothing has been written in yet is a file of the game's unlocks and no runs; and no game's
    /// file at all is a file of the header alone, which the game reads as a first launch.
    #[test]
    fn a_mode_with_no_file_yet_reads_as_the_unlocks_and_no_runs() {
        let own = encode(&[th6k(), hscr(999, 0, 1, 0), clrd(1, [1; 5], [1; 5])], 0x11);
        assert_eq!(
            decode(&the_copy_the_mode_reads(Some(&own), None)),
            Some(vec![th6k(), clrd(1, [1; 5], [1; 5])])
        );
        assert_eq!(
            decode(&the_copy_the_mode_reads(None, None)),
            Some(vec![th6k()])
        );
    }

    /// **A file of th06c's is not a file of this game's and the records say so**, the version being the
    /// byte every record carries: a `TH6K` of that build's reads as `0x10` where this one reads `0x15`.
    #[test]
    fn a_record_of_th06cs_file_carries_another_version() {
        let theirs = crate::game::th06c::score_file::th6k();
        assert_eq!(theirs.bytes[8], 0x10);
        assert_ne!(theirs.bytes, th6k().bytes);
    }
}
