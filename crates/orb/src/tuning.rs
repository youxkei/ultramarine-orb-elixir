//! Finding the midstage chapter boundaries that get baked into `chapters.rs`.
//!
//! Boss boundaries are detected from the game at runtime, but a stage's waves are
//! just a script on a clock, so those boundaries are frame numbers someone has to
//! choose. This mode proposes them — the quiet moments between waves — lets them
//! be corrected by hand while playing, and writes the result out as source.

use std::path::PathBuf;

use crate::game::{Game, State};
use crate::log::log;

/// The shortest gap the automatic detector will propose. Between waves there can
/// be several quiet frames in a row, and each would otherwise be a boundary.
const MIN_GAP_FRAMES: u32 = 120;

pub struct Tuning {
    /// Per stage, the boundaries found this session. `None` for a stage not
    /// visited yet, which keeps whatever the compiled-in table has for it.
    tuned: Vec<Option<Vec<i32>>>,
    /// Where the table is written, so finishing a stage is enough to save it and
    /// a pass driven by a replay needs nobody at the keyboard.
    path: PathBuf,
}

impl Tuning {
    pub fn new(game: &dyn Game, path: PathBuf) -> Self {
        Self { tuned: vec![None; game.midstage_table().len()], path }
    }

    /// Starts a fresh pass over `stage`, discarding anything found for it before.
    pub fn begin_stage(&mut self, stage: i32) {
        if let Some(slot) = self.slot(stage) {
            *slot = Some(Vec::new());
        }
    }

    /// A gap in the action: no enemies, nothing in the air, and no boss fight to
    /// interrupt. Nothing here is specific to a difficulty, which is why one
    /// table covers all of them.
    pub fn is_boundary(&mut self, state: &State, frames_since_last: u32) -> bool {
        let quiet = state.enemy_count == 0
            && state.bullet_count == 0
            && state.laser_count == 0
            && !state.boss_present;
        if !quiet || frames_since_last < MIN_GAP_FRAMES {
            return false;
        }
        self.record(state)
    }

    /// Adds the current moment as a boundary, for a spot the detector misses.
    pub fn add(&mut self, state: &State) -> bool {
        let added = self.record(state);
        log!("tuning: {} at tl {}", if added { "added" } else { "already have" }, state.script_frames);
        added
    }

    pub fn remove_last(&mut self, state: &State) {
        let Some(Some(boundaries)) = self.slot(state.stage).map(|slot| slot.as_mut()) else { return };
        match boundaries.pop() {
            Some(frame) => log!("tuning: removed tl {frame}"),
            None => log!("tuning: nothing to remove"),
        }
    }

    pub fn count(&self, stage: i32) -> usize {
        self.boundaries(stage).map_or(0, Vec::len)
    }

    fn record(&mut self, state: &State) -> bool {
        let Some(slot) = self.slot(state.stage) else { return false };
        let boundaries = slot.get_or_insert_with(Vec::new);
        if boundaries.last() == Some(&state.script_frames) {
            return false;
        }
        boundaries.push(state.script_frames);
        boundaries.sort_unstable();
        true
    }

    fn slot(&mut self, stage: i32) -> Option<&mut Option<Vec<i32>>> {
        self.tuned.get_mut(usize::try_from(stage).ok()?)
    }

    fn boundaries(&self, stage: i32) -> Option<&Vec<i32>> {
        self.tuned.get(usize::try_from(stage).ok()?)?.as_ref()
    }

    /// Writes the table as Rust, ready to replace the one in `chapters.rs`.
    /// Stages not tuned this session keep the values already compiled in, so a
    /// stage can be tuned, baked and left alone while the next one is done.
    pub fn write(&self, game: &dyn Game) {
        let path = &self.path;
        let table = game.midstage_table();
        let mut source = format!("pub const MIDSTAGE: [&[i32]; {}] = [\n", table.len());
        for (stage, built_in) in table.iter().enumerate() {
            let boundaries = self.boundaries(stage as i32).map_or(*built_in, Vec::as_slice);
            let label =
                if stage + 1 == table.len() { "extra  ".to_owned() } else { format!("stage {}", stage + 1) };
            let frames: Vec<String> = boundaries.iter().map(i32::to_string).collect();
            source += &format!("    /* {label} */ &[{}],\n", frames.join(", "));
        }
        source += "];\n";

        match std::fs::write(path, &source) {
            Ok(()) => log!("tuning: wrote {}", path.display()),
            Err(error) => log!("tuning: cannot write {}: {error}", path.display()),
        }
    }
}
