//! **The score file of th08 as bytes**: `score.dat` as 0x453d0d writes it and 0x45a5e0 reads it back — for the one
//! thing that makes one other than the game, the records `orb-real` lays into a copy for a case that starts from a
//! clear. orb itself never reads it; see `asm.md`'s *The score file, and the records it is read into*.
//!
//! **The file**: a header of `0x1c` bytes — `+0x1` and `+0x6` random bytes, `+0x2` a word sum of every byte from
//! `+0x4`, `+0x4` the version 1, `+0x8` the offset the chunks start at, `0x1c`, `+0x10` and `+0x14` the length the
//! chunks reach unpacked, `+0x18` the packed length — and from `+0x1c` the chunks packed by 0x473cb0, th10's LZSS.
//! Then every byte from `+0x2` on `xor`ed with a key carried along the bytes before it (the end of 0x453d0d), and
//! the whole enciphered by 0x43e4a0 with a key of `0x59` stepping by `0x79` over blocks of `0x100`, the first
//! `0xc00` bytes of it.
//!
//! **The chunks a file has to hold are `TH8K`, version 1**, without which the read throws the file away and
//! recreates it (0x45a5e0's `score.dat version mismatch`): `CLRD` and the rest are read where there is one and left
//! as the game made them where there is none.

/// The header and what it holds.
const HEADER: usize = 0x1c;
const SUM_AT: usize = 0x2;
const SUMMED_FROM: usize = 0x4;
const VERSION_AT: usize = 0x4;
const VERSION: u16 = 1;
const CHUNKS_AT_AT: usize = 0x8;
const CHUNKS_END_AT: usize = 0x10;
const CHUNKS_END_UNTIL_THE_EXES_SUM_AT: usize = 0x14;
const PACKED_LENGTH_AT: usize = 0x18;
/// Where the `xor` with a carried key starts, the byte before it being the key's start.
const XORED_FROM: usize = 0x2;

/// The cipher's key, its step, its block and how much of a file it covers (0x45422b through 0x454248).
const KEY: u8 = 0x59;
const STEP: u8 = 0x79;
const BLOCK: usize = 0x100;
const ENCIPHERED_AT_MOST: usize = 0xc00;

/// The window the packing copies out of (0x4740e0).
const WINDOW: usize = 0x2000;
const OFFSET_BITS: u32 = 13;
const LENGTH_BITS: u32 = 4;
const SHORTEST_COPY: usize = 3;

/// A chunk's tag, its length (twice, as words at `+0x4` and `+0x6`) and its version.
const CHUNK_LENGTH_AT: usize = 0x4;
const CHUNK_LENGTH_AGAIN_AT: usize = 0x6;
const CHUNK_VERSION_AT: usize = 0x8;

/// `TH8K`: the chunk that says the file is this game's (0x453d0d).
const TH8K: &[u8; 4] = b"TH8K";
const TH8K_BYTES: usize = 0xc;
const TH8K_VERSION: u8 = 1;

/// `CLRD`: a character's — and the thirteenth, every character's — stages passed and clears (0x45ac90).
const CLRD: &[u8; 4] = b"CLRD";
pub const CLRD_BYTES: usize = 0x24;
pub const CLRD_RECORDS: usize = 13;
const CLRD_VERSION: u8 = 4;
/// The two arrays of five words, Easy to Extra: `+0xc` what a run with no continue passed and cleared, `+0x16`
/// what any run did; and the record's own number at `+0x21`.
const NO_CONTINUE_AT: usize = 0xc;
const ANY_RUN_AT: usize = 0x16;
const DIFFICULTIES: usize = 5;
const CLRD_NUMBER_AT: usize = 0x21;
/// The thirteenth record, every character's.
const EVERY_CHARACTERS: usize = 12;

/// The bit a clear writes, at the ending's start (0x429a00 through 0x429c07): Kaguya's for a run that ended on stage
/// word 7, Eirin's for any other.
const KAGUYAS_CLEAR: u16 = 0x4000;
const EIRINS_CLEAR: u16 = 0x8000;

/// **The `CLRD` records a game with no file makes** (0x45ac90 with no chunk to read): each zeroed, headed, numbered,
/// and every word of both arrays 1.
pub fn the_clears_of_a_fresh_install() -> [[u8; CLRD_BYTES]; CLRD_RECORDS] {
    std::array::from_fn(|number| {
        let mut record = [0; CLRD_BYTES];
        record[..4].copy_from_slice(CLRD);
        record[CHUNK_LENGTH_AT..CHUNK_LENGTH_AT + 2]
            .copy_from_slice(&(CLRD_BYTES as u16).to_le_bytes());
        record[CHUNK_LENGTH_AGAIN_AT..CHUNK_LENGTH_AGAIN_AT + 2]
            .copy_from_slice(&(CLRD_BYTES as u16).to_le_bytes());
        record[CHUNK_VERSION_AT] = CLRD_VERSION;
        record[CLRD_NUMBER_AT] = number as u8;
        for difficulty in 0..DIFFICULTIES {
            put_word(&mut record, NO_CONTINUE_AT + difficulty * 2, 1);
            put_word(&mut record, ANY_RUN_AT + difficulty * 2, 1);
        }
        record
    })
}

/// **A stage passed, as the run manager marks it at the stage's end** (0x439c50 through 0x439d02): `1 << stage` into
/// the difficulty's word of the character's `+0x16` array and of the thirteenth record's, and into their `+0xc`
/// arrays as well for a run with no continue.
pub fn a_stage_passed(
    clears: &mut [[u8; CLRD_BYTES]; CLRD_RECORDS],
    character: usize,
    difficulty: usize,
    stage: u32,
    continued: bool,
) {
    marked(clears, character, difficulty, 1 << stage, continued);
}

/// **A run cleared, as the ending's start marks it** (0x429a00 through 0x429c07): Kaguya's bit or Eirin's into the
/// same words a stage's end marks.
pub fn a_clear(
    clears: &mut [[u8; CLRD_BYTES]; CLRD_RECORDS],
    character: usize,
    difficulty: usize,
    kaguyas: bool,
    continued: bool,
) {
    let bit = match kaguyas {
        true => KAGUYAS_CLEAR,
        false => EIRINS_CLEAR,
    };
    marked(clears, character, difficulty, bit, continued);
}

fn marked(
    clears: &mut [[u8; CLRD_BYTES]; CLRD_RECORDS],
    character: usize,
    difficulty: usize,
    bit: u16,
    continued: bool,
) {
    for record in [character, EVERY_CHARACTERS] {
        let mut arrays = vec![ANY_RUN_AT];
        if !continued {
            arrays.push(NO_CONTINUE_AT);
        }
        for array in arrays {
            let at = array + difficulty * 2;
            let word = u16::from_le_bytes([clears[record][at], clears[record][at + 1]]);
            put_word(&mut clears[record], at, word | bit);
        }
    }
}

/// **A file of these records as the game writes one** (0x453d0d): the header, `TH8K` and the thirteen `CLRD`s, packed,
/// summed, `xor`ed along and enciphered — none of the chunks the game reads where it finds one and leaves as it made
/// them where it does not, and no `VRSM`, whose sum of the exe the read checks only where the chunk is there.
///
/// **Packed as literals alone and not by the game's own packing**, which chooses which runs to copy: the read
/// (0x4740e0) unpacks either to the same chunks. The random bytes at `+0x1` and `+0x6` are nought.
pub fn encode(clears: &[[u8; CLRD_BYTES]; CLRD_RECORDS]) -> Vec<u8> {
    let mut chunks = Vec::new();
    let mut th8k = [0; TH8K_BYTES];
    th8k[..4].copy_from_slice(TH8K);
    th8k[CHUNK_LENGTH_AT..CHUNK_LENGTH_AT + 2].copy_from_slice(&(TH8K_BYTES as u16).to_le_bytes());
    th8k[CHUNK_LENGTH_AGAIN_AT..CHUNK_LENGTH_AGAIN_AT + 2]
        .copy_from_slice(&(TH8K_BYTES as u16).to_le_bytes());
    th8k[CHUNK_VERSION_AT] = TH8K_VERSION;
    chunks.extend_from_slice(&th8k);
    for record in clears {
        chunks.extend_from_slice(record);
    }
    let packed = packed(&chunks);
    let mut file = vec![0; HEADER];
    put_word(&mut file, VERSION_AT, VERSION);
    put_dword(&mut file, CHUNKS_AT_AT, HEADER as u32);
    put_dword(&mut file, CHUNKS_END_AT, (HEADER + chunks.len()) as u32);
    put_dword(
        &mut file,
        CHUNKS_END_UNTIL_THE_EXES_SUM_AT,
        (HEADER + chunks.len()) as u32,
    );
    put_dword(&mut file, PACKED_LENGTH_AT, packed.len() as u32);
    file.extend_from_slice(&packed);
    let sum = file[SUMMED_FROM..]
        .iter()
        .fold(0u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
    put_word(&mut file, SUM_AT, sum);
    let mut key = file[XORED_FROM - 1];
    for byte in &mut file[XORED_FROM..] {
        key = key.rotate_left(3);
        let plain = *byte;
        *byte ^= key;
        key = key.wrapping_add(plain);
    }
    enciphered(&file)
}

/// **The `CLRD` records a file holds, as 0x45a5e0 and 0x45ac90 read them**, or `None` for bytes the read throws away:
/// a sum that does not add up, a header of another size or version, or no `TH8K` of version 1.
pub fn decode(file: &[u8]) -> Option<[[u8; CLRD_BYTES]; CLRD_RECORDS]> {
    if file.len() < HEADER {
        return None;
    }
    let mut plain = deciphered(file);
    let mut key = 0u8;
    for at in XORED_FROM..plain.len() {
        key = key.wrapping_add(plain[at - 1]).rotate_left(3);
        plain[at] ^= key;
    }
    let sum = plain[SUMMED_FROM..]
        .iter()
        .fold(0u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
    if word(&plain, SUM_AT)? != sum
        || dword(&plain, CHUNKS_AT_AT)? != HEADER as u32
        || word(&plain, VERSION_AT)? != VERSION
    {
        return None;
    }
    let packed_length = dword(&plain, PACKED_LENGTH_AT)? as usize;
    let end = dword(&plain, CHUNKS_END_AT)? as usize;
    let chunks = unpacked(
        plain.get(HEADER..HEADER + packed_length)?,
        end.checked_sub(HEADER)?,
    );
    let mut clears = the_clears_of_a_fresh_install();
    let mut seen_th8k = false;
    let mut at = 0;
    while at + CHUNK_VERSION_AT < chunks.len() {
        let length = usize::from(word(&chunks, at + CHUNK_LENGTH_AT)?);
        if length == 0 {
            return None;
        }
        let chunk = chunks.get(at..at + length)?;
        if &chunk[..4] == TH8K && chunk[CHUNK_VERSION_AT] == TH8K_VERSION {
            seen_th8k = true;
        }
        if &chunk[..4] == CLRD && chunk[CHUNK_VERSION_AT] == CLRD_VERSION && length == CLRD_BYTES {
            let number = usize::from(chunk[CLRD_NUMBER_AT]);
            clears.get_mut(number)?.copy_from_slice(chunk);
        }
        at += length;
    }
    seen_th8k.then_some(clears)
}

/// 0x43e4a0: each block of `0x100` written back to front, its odd places first, each byte `xor`ed with the key and
/// the key stepped, over the first `0xc00` bytes; a short tail and an odd last byte left as they are.
fn enciphered(plain: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(plain.len());
    let short_tail = match plain.len() % BLOCK {
        tail if tail < BLOCK / 4 => tail,
        _ => 0,
    } + (plain.len() & 1);
    let mut left = plain.len() - short_tail;
    let mut covered = ENCIPHERED_AT_MOST as isize;
    let mut key = KEY;
    let mut start = 0;
    while left > 0 && covered > 0 {
        let block = left.min(BLOCK);
        let end = start + block;
        for place in (0..block.div_ceil(2))
            .map(|i| end - 1 - 2 * i)
            .chain((0..block / 2).map(|i| end - 2 - 2 * i))
        {
            out.push(plain[place] ^ key);
            key = key.wrapping_add(STEP);
        }
        start = end;
        left -= block;
        covered -= BLOCK as isize;
    }
    out.extend_from_slice(&plain[start..]);
    out
}

/// [`enciphered`] undone, as 0x43e1d0 undoes it: each place read in the order it was written.
fn deciphered(enciphered: &[u8]) -> Vec<u8> {
    let mut plain = enciphered.to_vec();
    let short_tail = match enciphered.len() % BLOCK {
        tail if tail < BLOCK / 4 => tail,
        _ => 0,
    } + (enciphered.len() & 1);
    let mut left = enciphered.len() - short_tail;
    let mut covered = ENCIPHERED_AT_MOST as isize;
    let mut key = KEY;
    let (mut read, mut start) = (0, 0);
    while left > 0 && covered > 0 {
        let block = left.min(BLOCK);
        let end = start + block;
        for place in (0..block.div_ceil(2))
            .map(|i| end - 1 - 2 * i)
            .chain((0..block / 2).map(|i| end - 2 - 2 * i))
        {
            plain[place] = enciphered[read] ^ key;
            key = key.wrapping_add(STEP);
            read += 1;
        }
        start = end;
        left -= block;
        covered -= BLOCK as isize;
    }
    plain
}

/// Every byte a literal — a set bit and its eight — and an offset of nought ending it.
fn packed(plain: &[u8]) -> Vec<u8> {
    let mut bits = Vec::new();
    for byte in plain {
        bits.push(1u8);
        bits.extend((0..8).rev().map(|shift| (byte >> shift) & 1));
    }
    bits.extend(std::iter::repeat_n(0, 1 + OFFSET_BITS as usize));
    bits.chunks(8)
        .map(|byte| {
            byte.iter()
                .enumerate()
                .fold(0, |value, (i, bit)| value | bit << (7 - i))
        })
        .collect()
}

/// 0x4740e0: a flag bit, then eight bits of a byte or thirteen of an offset into the window and four of a length,
/// every byte written also written into the window, an offset of nought ending it; `length` bytes at the most.
fn unpacked(packed: &[u8], length: usize) -> Vec<u8> {
    let mut at = 0usize;
    let mut take = |count: u32| {
        (0..count).fold(0usize, |value, _| {
            let bit = packed
                .get(at / 8)
                .map_or(0, |byte| (byte >> (7 - at % 8)) & 1);
            at += 1;
            value << 1 | usize::from(bit)
        })
    };
    let mut window = [0u8; WINDOW];
    let mut head = 1;
    let mut plain = Vec::with_capacity(length);
    while plain.len() < length {
        if take(1) == 1 {
            let byte = take(8) as u8;
            plain.push(byte);
            window[head] = byte;
            head = (head + 1) % WINDOW;
            continue;
        }
        let offset = take(OFFSET_BITS);
        if offset == 0 {
            break;
        }
        for i in 0..take(LENGTH_BITS) + SHORTEST_COPY {
            let byte = window[(offset + i) % WINDOW];
            plain.push(byte);
            window[head] = byte;
            head = (head + 1) % WINDOW;
        }
    }
    plain
}

fn word(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn dword(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn put_word(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_dword(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eirins_and_kaguyas() -> [[u8; CLRD_BYTES]; CLRD_RECORDS] {
        let mut clears = the_clears_of_a_fresh_install();
        for stage in [0, 1, 2, 4, 5, 6] {
            a_stage_passed(&mut clears, 0, 1, stage, false);
        }
        a_clear(&mut clears, 0, 1, false, false);
        a_stage_passed(&mut clears, 0, 1, 7, false);
        a_clear(&mut clears, 0, 1, true, false);
        clears
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(
            decode(&encode(&eirins_and_kaguyas())),
            Some(eirins_and_kaguyas())
        );
    }

    #[test]
    fn a_fresh_installs_clears_hold_one_in_every_word() {
        let clears = the_clears_of_a_fresh_install();
        assert_eq!(&clears[7][..4], b"CLRD");
        assert_eq!(clears[7][CLRD_NUMBER_AT], 7);
        assert_eq!(word(&clears[7], NO_CONTINUE_AT + 4 * 2), Some(1));
        assert_eq!(word(&clears[7], ANY_RUN_AT), Some(1));
    }

    #[test]
    fn kaguyas_clear_lights_the_bit_the_title_offers_the_extra_by_and_eirins_the_route_to_her() {
        let clears = eirins_and_kaguyas();
        let normal = NO_CONTINUE_AT + 2;
        assert_ne!(word(&clears[0], normal).unwrap() & KAGUYAS_CLEAR, 0);
        assert_ne!(word(&clears[0], ANY_RUN_AT + 2).unwrap() & 1 << 6, 0);
        assert_ne!(
            word(&clears[EVERY_CHARACTERS], normal).unwrap() & KAGUYAS_CLEAR,
            0
        );
        assert_eq!(word(&clears[1], normal), Some(1));
    }

    #[test]
    fn a_run_that_continued_marks_no_word_of_the_runs_with_none() {
        let mut clears = the_clears_of_a_fresh_install();
        a_clear(&mut clears, 2, 1, true, true);
        assert_eq!(word(&clears[2], NO_CONTINUE_AT + 2), Some(1));
        assert_eq!(word(&clears[2], ANY_RUN_AT + 2), Some(1 | KAGUYAS_CLEAR));
    }

    #[test]
    fn a_file_with_a_byte_turned_is_refused() {
        let mut file = encode(&eirins_and_kaguyas());
        let last = file.len() - 1;
        file[last] ^= 0x10;
        assert_eq!(decode(&file), None);
    }
}
