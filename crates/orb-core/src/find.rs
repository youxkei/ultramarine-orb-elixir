//! **A field of a game nobody has read one of, found by what it does rather than by where it is.**
//!
//! What this is for is th06c: a 64-bit reimplementation whose every address is its own, and in which the
//! only things orb can already do are press a key and count a frame. So the finding is driven from those
//! two — a recipe holds a direction for a while and then the other one, and what is left is the
//! addresses whose own value went the way the player's would have.
//!
//! **Two ways and not one, because one way is passed by everything that only goes up.** A frame counter
//! read as a float rises on every frame of a pass that wants rising; nothing that is not being steered
//! rises and then falls on the frames the steering changed. That is the whole of why this works, and it
//! is why the recipe that drives it presses one direction and then the other rather than moving about.
//!
//! **And the two readings are the ends of a press and not two frames**, which is the second thing this
//! took: a player held against a direction reaches the field's edge and stops there, so a rule that
//! wanted it to move on *every* frame threw it away the moment it arrived — measured, sixty-six
//! thousand places differed between two frames and thirty-five thousand of them rose, and by the
//! twenty-third frame the nine left were all in the system's own clock page. Compared end to end, a
//! value that moved and then sat against a wall still moved.
//!
//! **The exe's own `.data` and every private region**, which took three searches to get right: the first
//! watched `.data` alone, and the ones after it watched what [`orb_api::mem::private_regions`] answers —
//! which is every *private* committed region, and the exe's sections are `MEM_IMAGE` rather than private.
//! So the one place a static object of the game's can be was the one place no search had looked.
//!
//! **And only until the candidates are few.** The first pass reads everything; from then on a candidate
//! carries the value it had last frame, so the frames after it read four bytes apiece and nothing else.

use std::ops::Range;
use std::sync::Mutex;

use crate::log;

/// How much of the exe's own `.data` is read at a time, which is Windows' own page: a range is readable
/// only where all of it is, and that section's is not.
const A_PAGE: usize = 0x1000;

/// Which way the value being looked for is expected to be going this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Way {
    Up,
    Down,
    /// Nothing is being pressed, so nothing is expected of anything: a frame like this is skipped rather
    /// than used, the player standing still being a thing the whole of memory also does.
    Still,
}

/// How many survivors are few enough to be worth reading, which is what ends a search.
const FEW: usize = 64;

/// And how many frames one is followed for before it is given up on and says so: a search that narrows
/// to nothing has been driven by a recipe that did not press what it was written to press, and a launch
/// that says nothing about that is a launch nobody can tell from one that found nothing.
const AT_MOST: u32 = 1800;

/// Four bytes and aligned to four, which is what a float the game keeps is: a candidate at an odd
/// offset is two halves of two other numbers.
const ALIGNMENT: usize = 4;

/// **How wide the number being looked for is**, both being tried at once.
///
/// **Because a double read as a single is not monotonic.** A search over four-byte floats alone found
/// nothing in a live stage twice — nothing at all rose twelve frames running in two hundred megabytes,
/// which a stage's own timers do — and this exe is built with `movsd` in fourteen hundred places. Read
/// as a single, the top half of a rising double goes up and down with its exponent, so the one thing
/// this was looking for was the one thing it could not see.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Wide {
    /// `f32`, at every four bytes.
    Single,
    /// `f64`, at every eight.
    Double,
}

/// One address still in the running, the value it held last frame, and which width it is being read as.
#[derive(Clone, Copy)]
struct Candidate {
    at: usize,
    was: f64,
    wide: Wide,
}

/// The largest region worth watching.
///
/// **A bound and not a completeness**: what is being looked for is one number a key moves, and a region
/// of this size is a texture, a sound buffer or a swapchain — none of which a key moves, and all of
/// which cost the pass its whole budget. Said in the log, so a search that found nothing says how much
/// it did not look at.
const AT_MOST_A_REGION: usize = 512 * 1024 * 1024;

/// What one search holds between frames.
struct Looking {
    /// Everything being watched as it stood on the frame before, one reading per region — kept only
    /// until there are candidates, after which each carries its own last value.
    was: Vec<(usize, Vec<u8>)>,
    /// The addresses still in the running, or `None` before the first frame a direction was held on.
    candidates: Option<Vec<Candidate>>,
    /// How many frames have been spent, and which way the frame before was going — what the way is for
    /// is knowing whether this frame's reading is comparable with the last.
    frames: u32,
    was_going: Way,
    /// The last list that had anything in it, and how many frames in it was.
    ///
    /// **Because a search that ends with nothing is the one that needs reading.** What went wrong is
    /// always in the list the frame before: a player that was there and then was not had died, hit a
    /// wall or was never there, and the values say which. Kept rather than logged per frame, a pass
    /// being hundreds of frames long.
    was_left: (Vec<Candidate>, u32),
    /// Whether a pass each way has narrowed at all.
    ///
    /// **A search is not over until both have**, which is the whole of why it presses two ways: the
    /// first search here was declared finished four frames in, on seventeen values that had risen four
    /// times running and were never asked to come back down. Counters pass one way; nothing but what is
    /// being steered passes both.
    narrowed_up: bool,
    narrowed_down: bool,
}

static LOOKING: Mutex<Option<Looking>> = Mutex::new(None);

/// Whether a launch is looking for anything, which is what makes this cost nothing in one that is not.
static ASKED: Mutex<bool> = Mutex::new(false);

/// Says a launch is looking, or that it is not.
///
/// # Safety
/// Must run at the attach, before the game has been asked for a frame.
pub unsafe fn looks_for_what_moves(asked: bool) {
    *ASKED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = asked;
    if asked {
        log!(
            "find: looking for the field that moves with the direction pressed; \
             hold one way and then the other"
        );
    }
}

/// One frame of a search: what is watched read, the candidates narrowed, and the survivors said once
/// there are few enough of them.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn a_frame(data: &Range<usize>, going: Way) {
    if !*ASKED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    {
        return;
    }
    let mut held = LOOKING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Nothing on a frame with nothing pressed, and nothing before one: the first reading is taken on
    // the first frame a direction is held, so that the pair it is compared against is a pair the
    // pressing is the whole difference between.
    if going == Way::Still && held.is_none() {
        return;
    }
    let Some(looking) = held.as_mut() else {
        let was = reading(data);
        log!(
            "find: {} region(s) and {} bytes taken as they stand, with {going:?} pressed",
            was.len(),
            was.iter().map(|(_, bytes)| bytes.len()).sum::<usize>(),
        );
        *held = Some(Looking {
            was,
            candidates: None,
            frames: 0,
            was_going: going,
            was_left: (Vec::new(), 0),
            narrowed_up: false,
            narrowed_down: false,
        });
        return;
    };
    looking.frames += 1;
    // **Nothing happens until the pressing changes**, which is what makes the comparison end to end:
    // every frame of one direction is inside one press, and what a press did is the difference between
    // the reading taken when it began and the reading taken when it ended.
    if going == looking.was_going {
        return;
    }
    let pressed = std::mem::replace(&mut looking.was_going, going);
    if pressed == Way::Still {
        // Nothing was being pressed, so there is nothing to have moved: the reading is taken now
        // instead, this being where the next press begins.
        if looking.candidates.is_none() {
            looking.was = reading(data);
        } else if let Some(candidates) = &mut looking.candidates {
            *candidates = as_they_stand(candidates);
        }
        return;
    }
    let left = match looking.candidates.take() {
        None => {
            let left = what_moved(&looking.was, pressed);
            looking.was = Vec::new();
            left
        }
        Some(candidates) => still_moving(&candidates, pressed),
    };
    match pressed {
        Way::Up => looking.narrowed_up = true,
        Way::Down => looking.narrowed_down = true,
        Way::Still => {}
    }
    log!(
        "find: {} candidate(s) after {} frame(s), over a press of {pressed:?}",
        left.len(),
        looking.frames,
    );
    // **Both ways, then few enough.** A pass one way keeps every count that only rises, so a search
    // that stopped before the pressing turned round would be reporting those.
    let both = looking.narrowed_up && looking.narrowed_down;
    let done = (both && left.len() <= FEW) || left.is_empty() || looking.frames >= AT_MOST;
    if !left.is_empty() && left.len() <= FEW {
        looking.was_left = (left.clone(), looking.frames);
    }
    looking.candidates = Some(left);
    if done {
        said(data, looking);
        *held = None;
        unsafe { looks_for_what_moves(false) };
    }
}

/// Everything worth watching, as it stands.
fn reading(data: &Range<usize>) -> Vec<(usize, Vec<u8>)> {
    let mut skipped = Vec::new();
    let mut read = Vec::new();
    // **The exe's own `.data` first, and page by page.** It is the one region a static object of the
    // game's is in, and `private_regions` answers none of it — the sections of an image are `MEM_IMAGE`
    // and that walk keeps `MEM_PRIVATE`. Page by page because a range is readable only where all of it
    // is, and asking for th06c's nine megabytes in one piece answers nothing at all; a page that cannot
    // be read is left as nought, which is a page nothing moved in and so no candidate.
    let mut bytes = vec![0u8; data.len()];
    for at in (0..data.len()).step_by(A_PAGE) {
        let len = A_PAGE.min(data.len() - at);
        if let Some(page) = orb_api::mem::read_committed_bytes(data.start + at, len) {
            bytes[at..at + len].copy_from_slice(&page);
        }
    }
    read.push((data.start, bytes));
    for (base, len) in orb_api::mem::private_regions() {
        if len > AT_MOST_A_REGION {
            skipped.push((base, len, "too big"));
            continue;
        }
        match orb_api::mem::read_committed_bytes(base, len) {
            Some(bytes) => read.push((base, bytes)),
            None => skipped.push((base, len, "unreadable")),
        }
    }
    // **Each of them named**, because what is skipped is what a search cannot find: a game that keeps
    // its objects in one big arena is a game whose player is in exactly the region a bound like this
    // leaves out, and a count says nothing about whether that happened.
    for (base, len, why) in &skipped {
        log!("find: {base:#x} for {len} bytes is {why}, so nothing there is looked at");
    }
    read
}

/// **Every address whose number went the way it was meant to**, out of a reading taken now against the
/// one taken last frame — read as a single at every four bytes and as a double at every eight. The
/// first pass, which is the only one that reads everything.
fn what_moved(was: &[(usize, Vec<u8>)], going: Way) -> Vec<Candidate> {
    let mut left = Vec::new();
    // **How much moved at all, said once.** What this answers is whether the search's own premise
    // holds: two readings a frame apart in a running game differ in thousands of places, and a pass
    // that says nothing rose is a pass whose two readings are of the same frame. Told apart from a
    // pass that saw plenty move and kept none of it, which is a rule to look at rather than a premise.
    let mut differed = 0usize;
    let mut rose = 0usize;
    let mut fell = 0usize;
    for (base, before) in was {
        let Some(now) = orb_api::mem::read_committed_bytes(*base, before.len()) else {
            continue;
        };
        for at in (0..before.len().saturating_sub(size_of::<f32>())).step_by(ALIGNMENT) {
            if let (Some(then), Some(after)) = (single_at(before, at), single_at(&now, at)) {
                if then.to_bits() != after.to_bits() {
                    differed += 1;
                    if went(then, after, Way::Up) {
                        rose += 1;
                    }
                    if went(then, after, Way::Down) {
                        fell += 1;
                    }
                }
                if went(then, after, going) {
                    left.push(Candidate {
                        at: base + at,
                        was: after,
                        wide: Wide::Single,
                    });
                }
            }
            if !at.is_multiple_of(size_of::<f64>()) {
                continue;
            }
            if let (Some(then), Some(after)) = (double_at(before, at), double_at(&now, at))
                && went(then, after, going)
            {
                left.push(Candidate {
                    at: base + at,
                    was: after,
                    wide: Wide::Double,
                });
            }
        }
    }
    log!(
        "find: {differed} four-byte place(s) differed between the two readings, {rose} up and \
         {fell} down"
    );
    left
}

/// The candidates with the values they hold now, which is where the next press begins from.
fn as_they_stand(candidates: &[Candidate]) -> Vec<Candidate> {
    candidates
        .iter()
        .filter_map(|candidate| {
            Some(Candidate {
                was: now_at(candidate)?,
                ..*candidate
            })
        })
        .collect()
}

/// What one candidate reads as this moment, at its own width.
fn now_at(candidate: &Candidate) -> Option<f64> {
    match candidate.wide {
        Wide::Single => orb_api::mem::read_committed::<f32>(candidate.at).map(f64::from),
        Wide::Double => orb_api::mem::read_committed::<f64>(candidate.at),
    }
}

/// And the passes after it, which read one number per candidate and nothing else.
fn still_moving(candidates: &[Candidate], going: Way) -> Vec<Candidate> {
    candidates
        .iter()
        .filter_map(|candidate| {
            let after = now_at(candidate)?;
            went(candidate.was, after, going).then_some(Candidate {
                was: after,
                ..*candidate
            })
        })
        .collect()
}

/// Whether one reading to the next is a move the way asked for.
///
/// A value that did not move at all is out as well as one that moved the wrong way: what is being
/// looked for is steered on every frame a direction is held, so a frame it stood still on is a frame it
/// was not the thing being steered.
///
/// **And a reading that is not a number is out**, which is most of what a process holds: a pointer or a
/// string read as a float is an infinity or a NaN far more often than it is a coordinate, and neither
/// compares greater or less than anything.
pub fn went(before: f64, after: f64, going: Way) -> bool {
    if !before.is_finite() || !after.is_finite() {
        return false;
    }
    match going {
        Way::Up => after > before,
        Way::Down => after < before,
        Way::Still => false,
    }
}

/// The single four bytes of a reading hold, as a double so that both widths compare alike.
fn single_at(bytes: &[u8], at: usize) -> Option<f64> {
    let four: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
    Some(f64::from(f32::from_le_bytes(four)))
}

/// And the double eight of them hold, or nothing where there are not eight to be had.
fn double_at(bytes: &[u8], at: usize) -> Option<f64> {
    let eight: [u8; 8] = bytes.get(at..at + 8)?.try_into().ok()?;
    Some(f64::from_le_bytes(eight))
}

/// Says what is left, as an address and both readings of it: the float it is being looked for as, and
/// the same four bytes as an integer, because a count that happens to rise and fall is told from a
/// coordinate by which of the two reads as a number somebody would recognise.
fn said(data: &Range<usize>, looking: &Looking) {
    let Some(candidates) = &looking.candidates else {
        return log!("find: nothing was narrowed at all");
    };
    if candidates.is_empty() {
        log!(
            "find: nothing moved both ways over {} frame(s); the recipe pressed neither, or what \
             moves is not a number where orb looked",
            looking.frames,
        );
        // And what was there the last time anything was, which is what says whether the thing being
        // looked for was ever a candidate at all.
        let (was_left, when) = &looking.was_left;
        if was_left.is_empty() {
            return log!("find: and nothing was ever narrowed to a few");
        }
        log!("find: what was left at {when} frame(s) was");
        return said_each(was_left);
    }
    log!(
        "find: {} left after {} frame(s)",
        candidates.len(),
        looking.frames,
    );
    said_each(candidates);
    // And the path to each, without which a heap address is one launch's own.
    who_points_at(data, candidates);
}

/// **How far above a candidate a pointer is looked for**, which is how big an object of the game's is
/// taken to be: a field is somewhere inside one and the pointer that is kept is to its start.
///
/// **Sixty-four kilobytes, because eight was too few.** th06c's own startup news objects of 0x2c58
/// bytes and more and keeps each in a static of its own — `0x14090a0d8` and its neighbours — so a
/// window of 0x2000 found nothing at all for a field deeper in than that.
const AN_OBJECT: usize = 0x1_0000;

/// **What in the exe's own `.data` points at a candidate**, and how far into what it points the
/// candidate is.
///
/// **Because a candidate is a heap address and a heap address is one launch's.** What orb can read
/// twice is a static: the pointer the game keeps its object in lives in `.data`, at an address that is
/// the same every launch once the loader's own base is taken off. So the finding is not over at the
/// value — it is over at the path to it.
///
/// Said as the `.data` address, what it holds, and the difference: `P holds O, and the candidate is
/// O+N`, which is a field of a struct at an offset and the two numbers a reading needs.
fn who_points_at(data: &Range<usize>, candidates: &[Candidate]) {
    let watched = reading(data);
    for candidate in candidates.iter().take(FEW) {
        // **Two levels and not one**, because what a static holds is often a manager and what the
        // manager holds is the object: `.data` → manager → the field is the shape this game keeps its
        // bullets in, and a search that only looked one step out reported nothing at all.
        let mut said = 0;
        for (base, delta) in points_into(&watched, candidate.at) {
            let from = if data.contains(&base) {
                ".data"
            } else {
                "a heap"
            };
            log!(
                "find:   {from} {base:#x} holds the object {candidate:#x} is {delta:#x} into",
                candidate = candidate.at,
            );
            said += 1;
            // And who holds *that*, where the first step landed on the heap.
            if !data.contains(&base) {
                for (outer, step) in points_into(&watched, base) {
                    let whose = if data.contains(&outer) {
                        ".data"
                    } else {
                        "a heap"
                    };
                    log!("find:     and {whose} {outer:#x} holds it, {step:#x} in");
                    said += 1;
                    if said > AS_MANY_AS {
                        break;
                    }
                }
            }
            if said > AS_MANY_AS {
                log!("find:   and more, which is a candidate nothing points at in particular");
                break;
            }
        }
    }
}

/// How many holders are worth saying about one candidate: a value pointed at from everywhere is one
/// nothing in particular owns, and the list is what would bury the one that does.
const AS_MANY_AS: usize = 6;

/// Every address in what is watched holding a pointer to the object `wanted` is inside, as that
/// pointer's own address and how far into the object it points.
fn points_into(watched: &[(usize, Vec<u8>)], wanted: usize) -> Vec<(usize, usize)> {
    let mut holders = Vec::new();
    for (base, bytes) in watched {
        for at in (0..bytes.len().saturating_sub(size_of::<u64>())).step_by(size_of::<u64>()) {
            let Ok(eight) = <[u8; 8]>::try_from(&bytes[at..at + 8]) else {
                continue;
            };
            let held = u64::from_le_bytes(eight) as usize;
            if held != 0 && held <= wanted && wanted - held < AN_OBJECT {
                holders.push((base + at, wanted - held));
            }
        }
    }
    holders
}

/// One line per candidate: the address, what it was read as, and the same bytes as a whole number —
/// because a count that happens to rise and fall is told from a coordinate by which of the two reads as
/// a number somebody would recognise.
fn said_each(candidates: &[Candidate]) {
    for candidate in candidates.iter().take(FEW) {
        let whole = orb_api::mem::read_committed::<u32>(candidate.at).unwrap_or(0);
        let now = match candidate.wide {
            Wide::Single => orb_api::mem::read_committed::<f32>(candidate.at).map(f64::from),
            Wide::Double => orb_api::mem::read_committed::<f64>(candidate.at),
        };
        log!(
            "find:   {:#x} was {} as a {:?}, is {} now, {whole} as a whole number",
            candidate.at,
            candidate.was,
            candidate.wide,
            now.unwrap_or(f64::NAN),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{Way, went};

    /// **A value steered both ways is the one that survives both passes**, which is the whole of the
    /// narrowing: the first keeps everything that rose — a count of frames among them — and the second
    /// takes out everything that went on rising.
    #[test]
    fn a_value_steered_both_ways_survives_both_passes() {
        for (before, after, up, down) in [
            (192.0, 196.0, true, false),
            (196.0, 192.0, false, true),
            (1.0, 2.0, true, false),
        ] {
            assert_eq!(went(before, after, Way::Up), up, "{before} to {after}, Up");
            assert_eq!(
                went(before, after, Way::Down),
                down,
                "{before} to {after}, Down",
            );
        }
    }

    /// And a value that did not move is out of the pass it did not move on, a field being steered on
    /// every frame a direction is held.
    #[test]
    fn a_value_that_stood_still_is_no_candidate() {
        assert!(!went(192.0, 192.0, Way::Up));
        assert!(!went(192.0, 192.0, Way::Down));
    }

    /// And what is not a number is out: what a process holds is mostly pointers and strings, and either
    /// read as a float is an infinity or a NaN far more often than a coordinate.
    #[test]
    fn bytes_that_are_no_number_are_no_candidate() {
        assert!(!went(f64::NAN, 1.0, Way::Up));
        assert!(!went(1.0, f64::INFINITY, Way::Up));
        assert!(!went(f64::INFINITY, 1.0, Way::Down));
    }

    /// And nothing at all is expected of a frame with nothing pressed.
    #[test]
    fn a_frame_with_nothing_pressed_expects_nothing() {
        assert!(!went(1.0, 2.0, Way::Still));
        assert!(!went(2.0, 1.0, Way::Still));
    }
}
