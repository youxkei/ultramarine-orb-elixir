//! **The score file of th07 as bytes**: `score.dat` as 0x44552c writes it and 0x444c20 reads it back — for the one
//! thing that makes one other than the game, the records `orb-real` lays into a copy for a case that starts from a
//! clear. orb itself never reads it.
//!
//! **th08's file without th08's cipher**: a header of `0x1c` bytes — `+0x1` and `+0x6` random bytes, `+0x2` a word sum
//! of every byte from `+0x4`, `+0x4` the version `0xb`, `+0x8` the offset the chunks start at, `0x1c`, `+0x10` and
//! `+0x14` the length the chunks reach unpacked, `+0x18` the packed length — and from `+0x1c` the chunks packed by
//! 0x45ead0, unpacked by 0x45ef00 (th10's LZSS). Then every byte from `+0x2` on `xor`ed with a key carried along the
//! bytes before it (the end of 0x44552c), which is the whole of what stands between the file and its bytes.
//!
//! **The chunks a file has to hold are `TH7K`, version 1**, without which the read throws the file away and
//! recreates it (`score.dat version mismatch`): `CLRD` and the rest are read where there is one and left as the game
//! made them where there is none.

/// The header and what it holds.
const HEADER: usize = 0x1c;
const SUM_AT: usize = 0x2;
const SUMMED_FROM: usize = 0x4;
const VERSION_AT: usize = 0x4;
const VERSION: u16 = 0xb;
const CHUNKS_AT_AT: usize = 0x8;
const CHUNKS_END_AT: usize = 0x10;
const CHUNKS_END_UNTIL_THE_EXES_SUM_AT: usize = 0x14;
const PACKED_LENGTH_AT: usize = 0x18;
/// Where the `xor` with a carried key starts, the byte before it being the key's start.
const XORED_FROM: usize = 0x2;

/// The window the packing copies out of (0x45ef00).
const WINDOW: usize = 0x2000;
const OFFSET_BITS: u32 = 13;
const LENGTH_BITS: u32 = 4;
const SHORTEST_COPY: usize = 3;

/// A chunk's tag, its length (twice, as words at `+0x4` and `+0x6`) and its version.
const CHUNK_LENGTH_AT: usize = 0x4;
const CHUNK_LENGTH_AGAIN_AT: usize = 0x6;
const CHUNK_VERSION_AT: usize = 0x8;

/// `TH7K`: the chunk that says the file is this game's.
const TH7K: &[u8; 4] = b"TH7K";
const TH7K_BYTES: usize = 0xc;
const TH7K_VERSION: u8 = 1;

/// `CLRD`: a shot's — `character * 2 + shot` — stages reached and clears (0x445192).
const CLRD: &[u8; 4] = b"CLRD";
pub const CLRD_BYTES: usize = 0x1c;
pub const CLRD_RECORDS: usize = 6;
const CLRD_VERSION: u8 = 1;
/// The two arrays of a byte a difficulty, Easy to Phantasm: `+0xc` what a run with no continue reached, `+0x12` what
/// any run did; and the record's own number at `+0x18`.
const NO_CONTINUE_AT: usize = 0xc;
const ANY_RUN_AT: usize = 0x12;
const DIFFICULTIES: usize = 5;
const CLRD_NUMBER_AT: usize = 0x18;

/// What the ending writes for a clear, `99` (0x41e590), and the title offers the Extra to a shot by — `'c'` in any of
/// its no-continue bytes for Easy to Lunatic (0x42f7df).
pub const CLEARED: u8 = 99;

/// **The `CLRD` records a game with no file makes** (0x445192 with no chunk to read): each zeroed, headed, numbered,
/// and every byte of both arrays 1.
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
            record[NO_CONTINUE_AT + difficulty] = 1;
            record[ANY_RUN_AT + difficulty] = 1;
        }
        record
    })
}

/// **A run cleared, as the ending marks it** (0x41e590): [`CLEARED`] into the shot's byte for the
/// difficulty of the array any run reached, and of the no-continue array as well for a run with none.
pub fn a_clear(
    clears: &mut [[u8; CLRD_BYTES]; CLRD_RECORDS],
    shot: usize,
    difficulty: usize,
    continued: bool,
) {
    if !continued {
        clears[shot][NO_CONTINUE_AT + difficulty] = CLEARED;
    }
    clears[shot][ANY_RUN_AT + difficulty] = CLEARED;
}

/// **A file of these records as the game writes one** (0x44552c): the header, `TH7K` and the six `CLRD`s, packed,
/// summed and `xor`ed along — none of the chunks the game reads where it finds one and leaves as it made them where
/// it does not, and no `VRSM`, whose sum of the exe the read checks only where the chunk is there.
///
/// **Packed as literals alone and not by the game's own packing**, which chooses which runs to copy: the read
/// (0x45ef00) unpacks either to the same chunks. The random bytes at `+0x1` and `+0x6` are nought.
pub fn encode(clears: &[[u8; CLRD_BYTES]; CLRD_RECORDS]) -> Vec<u8> {
    let mut chunks = Vec::new();
    let mut th7k = [0; TH7K_BYTES];
    th7k[..4].copy_from_slice(TH7K);
    th7k[CHUNK_LENGTH_AT..CHUNK_LENGTH_AT + 2].copy_from_slice(&(TH7K_BYTES as u16).to_le_bytes());
    th7k[CHUNK_LENGTH_AGAIN_AT..CHUNK_LENGTH_AGAIN_AT + 2]
        .copy_from_slice(&(TH7K_BYTES as u16).to_le_bytes());
    th7k[CHUNK_VERSION_AT] = TH7K_VERSION;
    chunks.extend_from_slice(&th7k);
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
    file
}

/// **The `CLRD` records a file holds, as 0x444c20 and 0x445192 read them**, or `None` for bytes the read throws away:
/// a sum that does not add up, a header of another size or version, or no `TH7K` of version 1.
pub fn decode(file: &[u8]) -> Option<[[u8; CLRD_BYTES]; CLRD_RECORDS]> {
    if file.len() < HEADER {
        return None;
    }
    let mut plain = file.to_vec();
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
    let mut seen_th7k = false;
    let mut at = 0;
    while at + CHUNK_VERSION_AT < chunks.len() {
        let length = usize::from(word(&chunks, at + CHUNK_LENGTH_AT)?);
        if length == 0 {
            return None;
        }
        let chunk = chunks.get(at..at + length)?;
        if &chunk[..4] == TH7K && chunk[CHUNK_VERSION_AT] == TH7K_VERSION {
            seen_th7k = true;
        }
        if &chunk[..4] == CLRD && chunk[CHUNK_VERSION_AT] == CLRD_VERSION && length == CLRD_BYTES {
            let number = usize::from(chunk[CLRD_NUMBER_AT]);
            clears.get_mut(number)?.copy_from_slice(chunk);
        }
        at += length;
    }
    seen_th7k.then_some(clears)
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

/// 0x45ef00: a flag bit, then eight bits of a byte or thirteen of an offset into the window and four of a length,
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

    fn a_normal_clear() -> [[u8; CLRD_BYTES]; CLRD_RECORDS] {
        let mut clears = the_clears_of_a_fresh_install();
        a_clear(&mut clears, 0, 1, false);
        clears
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&a_normal_clear())), Some(a_normal_clear()));
    }

    #[test]
    fn a_normal_clear_with_no_continue_lights_the_byte_the_title_offers_the_extra_by() {
        let clears = a_normal_clear();
        assert_eq!(clears[0][NO_CONTINUE_AT + 1], CLEARED);
        assert_eq!(clears[0][ANY_RUN_AT + 1], CLEARED);
        assert_eq!(clears[1][NO_CONTINUE_AT + 1], 1);
    }

    #[test]
    fn a_run_that_continued_lights_no_byte_of_the_runs_with_none() {
        let mut clears = the_clears_of_a_fresh_install();
        a_clear(&mut clears, 3, 2, true);
        assert_eq!(clears[3][NO_CONTINUE_AT + 2], 1);
        assert_eq!(clears[3][ANY_RUN_AT + 2], CLEARED);
    }

    #[test]
    fn a_file_with_a_byte_turned_is_refused() {
        let mut file = encode(&a_normal_clear());
        let last = file.len() - 1;
        file[last] ^= 0x10;
        assert_eq!(decode(&file), None);
    }
}
