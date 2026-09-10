//! **The names th07 and th08 keep their songs by**, which is one sound engine's: one music file with a record
//! per track, looked up by a name with its directory cut off, and a stage's own data naming its tracks by the
//! `.mid` names the load writes `wav` over.

use orb_api::mem;

/// A name out of the game's own memory, up to the first zero byte and no further than the room it is given —
/// which is what every name this game keeps is: a fixed buffer with a terminated string in it.
///
/// `None` where the memory cannot be read, so a pointer chase that has gone stale answers nothing rather than
/// a name made of whatever was there.
pub(crate) fn name_at(address: usize, room: usize) -> Option<Vec<u8>> {
    let held = mem::read_committed_bytes(address, room)?;
    let end = held
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(held.len());
    Some(held[..end].to_vec())
}

/// The file's own name out of a path, which is everything past the last separator: the player's own lookup
/// walks past both kinds — th07's at 0x44bb06, th08's at 0x45c6c6 — so both are cut here.
pub(crate) fn file_name(path: &[u8]) -> &[u8] {
    let cut = path
        .iter()
        .rposition(|byte| *byte == b'/' || *byte == b'\\')
        .map_or(0, |at| at + 1);
    &path[cut..]
}

/// And the name without its extension, which is what two names for one song have in common: a stage's data
/// carries `bgm/th07_12.mid` where a slot holds `th07_12.wav` and a record holds `th07_12.wav` — so the stem
/// is the whole of what they agree on.
pub(crate) fn file_stem(path: &[u8]) -> &[u8] {
    let name = file_name(path);
    match name.iter().rposition(|byte| *byte == b'.') {
        Some(at) => &name[..at],
        None => name,
    }
}

/// Whether two names are the same song, which is their stems held against each other — see [`file_stem`].
pub(crate) fn same_song(one: &[u8], other: &[u8]) -> bool {
    file_stem(one).eq_ignore_ascii_case(file_stem(other))
}
