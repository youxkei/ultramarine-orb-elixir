//! Finding the midstage chapter boundaries that get baked into `chapters.rs`.
//!
//! Boss boundaries are detected from the game at runtime, but a stage's waves are
//! just a script on a clock, so those boundaries are frame numbers someone has to
//! choose. This mode proposes them — a second into each gap between waves — lets
//! them be judged and corrected by hand while playing, and writes the result out as
//! source.
//!
//! What it has found and what has been decided about each of them also goes to
//! `tuning.txt`, which is read back at startup. A stage is more than one sitting's
//! work: the file is what lets it be picked up again rather than started over.

use std::path::{Path, PathBuf};

use crate::game::{Game, State};
use crate::{detail, log};

/// How long with nothing left to shoot at before the stage counts as being between
/// waves.
///
/// Bullets in the air are not part of the test. The game was not written around
/// chapters, so a frame with none of them is rare — asking for one found three
/// places in a stage where a retry unit wants a dozen — and a snapshot restores
/// whatever is in the air exactly, so a boundary does not need the screen to be
/// clear. What it needs is a gap in the script, and enemies are what say that.
const ENEMY_GAP_FRAMES: u32 = 60;

/// The shortest a chapter may be. A gap that comes sooner than this is passed over
/// rather than saved up: the next one will do.
const MIN_GAP_FRAMES: u32 = 120;

/// The shortest gap the log bothers to mention. What `ENEMY_GAP_FRAMES` should be is
/// a question about this game's waves, so every gap long enough to be a candidate is
/// reported whether or not it was taken — a pass over a run is then the measurement,
/// rather than the number being guessed at and the table coming out short.
const GAP_WORTH_REPORTING: u32 = 10;

/// The table, as Rust to paste into the source.
const TABLE_FILE: &str = "chapters.rs";
/// The same boundaries and what has been decided about them, in a form this reads
/// back.
const STATE_FILE: &str = "tuning.txt";

pub struct Tuning {
    /// Per stage, the pass over it. `None` for a stage neither visited nor read back,
    /// which keeps whatever the compiled-in table has for it.
    passes: Vec<Option<Pass>>,
    /// Where both files live: beside the launcher, which is beside the game.
    dir: PathBuf,
}

/// What has been decided about a boundary while looking at it.
#[derive(Clone, Copy, PartialEq)]
pub enum Verdict {
    /// Goes into the table as it stands.
    Keep,
    /// Goes in, marked in the written source for someone to move by hand.
    Adjust,
    /// Kept out of the table. Remembered rather than deleted, so that the decision
    /// survives the stage being played over — the detector would otherwise propose it
    /// again — and so that it can be taken back.
    Rejected,
}

/// One boundary of one stage.
#[derive(Clone, Copy)]
pub struct Boundary {
    frame: i32,
    verdict: Verdict,
    /// Whether a person put it there rather than the detector. Worth telling apart:
    /// the detector's proposals come back by playing the stage again, and this one
    /// does not, so it is the one that would be quietly lost.
    by_hand: bool,
}

/// What one stage's pass holds, and how much of the stage it has seen this session.
struct Pass {
    /// Sorted by frame, and holding the ones judged out as well as the ones in the
    /// table.
    boundaries: Vec<Boundary>,
    /// The furthest script frame the pass has reached this session.
    covered: i32,
    /// How long there has been nothing to shoot at. Counted over new ground only,
    /// along with `covered`, so that replaying part of a stage leaves it holding
    /// what it held there the first time.
    without_enemies: u32,
}

/// What is known about a boundary, for whoever is looking at it.
#[derive(Clone, Copy)]
pub struct Judged {
    /// Which boundary this is about, as the script frame it would be written down as.
    /// Not always the frame on screen — a chapter is judged from anywhere inside it — and
    /// what the judging keys are about to change is not a thing to have to work out.
    pub frame: i32,
    pub verdict: Verdict,
    pub by_hand: bool,
}

impl Tuning {
    pub fn new(game: &dyn Game, dir: PathBuf) -> Self {
        let mut tuning = Self {
            passes: (0..game.midstage_table().len()).map(|_| None).collect(),
            dir,
        };
        tuning.load();
        tuning
    }

    /// Starts a fresh look at `stage`, keeping everything already known about it —
    /// which is the point of writing it down — and forgetting only how far the
    /// detector had got, since that is about this pass and not about the stage.
    pub fn begin_stage(&mut self, stage: i32) {
        if let Some(slot) = self.slot(stage) {
            match slot {
                Some(pass) => {
                    pass.covered = i32::MIN;
                    pass.without_enemies = 0;
                }
                None => *slot = Some(Pass::new()),
            }
        }
    }

    /// Proposes a boundary a second into a gap between waves: nothing to shoot at
    /// and no boss fight to interrupt. Nothing here is specific to a difficulty,
    /// which is why one table covers all of them.
    ///
    /// Only on ground the pass has not covered yet. Stepping back through the
    /// chapters plays part of the stage a second time, and a second look at the same
    /// frames must neither propose a boundary twice nor bring back one that was
    /// judged out.
    pub fn propose(&mut self, state: &State, since_last: u32) {
        let nothing_to_shoot = state.enemy_count == 0 && !state.boss_present;
        let frame = state.script_frames;
        let Some(pass) = self.pass_mut(state.stage) else {
            return;
        };
        if frame <= pass.covered {
            return;
        }
        pass.covered = frame;
        if !nothing_to_shoot && pass.without_enemies >= GAP_WORTH_REPORTING {
            detail!(
                "tuning: gap of {} frames, script {}..{frame}",
                pass.without_enemies,
                frame - pass.without_enemies as i32,
            );
        }
        pass.without_enemies = if nothing_to_shoot {
            pass.without_enemies + 1
        } else {
            0
        };
        // The frame the gap becomes one, and not every frame after it: a lull of ten
        // seconds is one boundary, where testing the two floors separately would put
        // one every time the shorter of them came round again.
        if pass.without_enemies == ENEMY_GAP_FRAMES && since_last >= MIN_GAP_FRAMES {
            pass.propose(frame);
        }
    }

    /// Puts a boundary at `frame` by hand, for a gap the detector misses, and reports
    /// whether the stage has one there now. An explicit hand is allowed to bring back
    /// one that was judged out, which asking again for the same frame is the way to do.
    pub fn add(&mut self, stage: i32, frame: i32) -> bool {
        let Some(pass) = self.pass_mut(stage) else {
            return false;
        };
        let known = pass.at(frame).map(|boundary| boundary.verdict);
        pass.put(Boundary {
            frame,
            verdict: Verdict::Keep,
            by_hand: true,
        });
        match known {
            None => log!("tuning: added tl {frame} by hand"),
            Some(Verdict::Keep) => log!("tuning: tl {frame} is already there"),
            Some(verdict) => log!("tuning: tl {frame} {} -> KEEP by hand", verdict.label()),
        }
        true
    }

    /// Judges one boundary of `stage`, one step better or one step worse: `Rejected`
    /// to `Adjust` to `Keep`, and back. Nothing wraps — pressing on past `Keep` must
    /// not throw the boundary away.
    ///
    /// Which boundary is the caller's to say, and it says the chapter's own: the
    /// nearest one behind would be some other chapter's whenever a boss began this
    /// one, and judging a boundary that is not the one on screen is worse than
    /// judging nothing.
    pub fn judge_up(&mut self, stage: i32, boundary: i32) {
        self.judge(stage, boundary, |verdict| match verdict {
            Verdict::Rejected => Verdict::Adjust,
            _ => Verdict::Keep,
        });
    }

    pub fn judge_down(&mut self, stage: i32, boundary: i32) {
        self.judge(stage, boundary, |verdict| match verdict {
            Verdict::Keep => Verdict::Adjust,
            _ => Verdict::Rejected,
        });
    }

    /// Puts a boundary out of the table, which is where `judge_down` twice would also
    /// leave it.
    pub fn reject(&mut self, stage: i32, boundary: i32) {
        self.judge(stage, boundary, |_| Verdict::Rejected);
    }

    fn judge(&mut self, stage: i32, boundary: i32, step: fn(Verdict) -> Verdict) {
        let Some(pass) = self.pass_mut(stage) else {
            return;
        };
        let Some(before) = pass.at(boundary) else {
            return log!("tuning: tl {boundary} is not a boundary of this stage");
        };
        let verdict = step(before.verdict);
        if verdict == before.verdict {
            return;
        }
        // One put there by hand and judged out goes altogether. Nothing would propose
        // it again, so a line saying it was refused would only be in the way — and
        // `tuning_add_key` is how it came to be there, which is how it comes back.
        //
        // The detector's own are kept as refused instead, because forgetting one is
        // exactly what has it proposed again the next time the stage is played.
        if verdict == Verdict::Rejected && before.by_hand {
            pass.forget(boundary);
            return log!("tuning: took out tl {boundary}, which was put there by hand");
        }
        pass.put(Boundary { verdict, ..before });
        log!(
            "tuning: tl {boundary} {} -> {}",
            before.verdict.label(),
            verdict.label()
        );
    }

    /// What is known about one boundary, or `None` for a frame this stage has never
    /// had one at.
    pub fn judged(&self, stage: i32, boundary: i32) -> Option<Judged> {
        let found = self.pass(stage)?.at(boundary)?;
        Some(Judged {
            frame: found.frame,
            verdict: found.verdict,
            by_hand: found.by_hand,
        })
    }

    /// How many boundaries of `stage` the table would have.
    pub fn count(&self, stage: i32) -> usize {
        self.pass(stage).map_or(0, |pass| pass.kept().count())
    }

    /// The frames of `stage`'s boundaries that are judged out of the table, in order.
    /// They begin no chapter, so nothing else in orb knows where they are.
    pub fn rejected(&self, stage: i32) -> impl Iterator<Item = i32> {
        self.pass(stage)
            .into_iter()
            .flat_map(|pass| pass.boundaries.iter())
            .filter(|boundary| boundary.verdict == Verdict::Rejected)
            .map(|boundary| boundary.frame)
    }

    /// The newest boundary in `(after, upto]` that the table would have, which is what
    /// says a chapter begins here.
    pub fn passed(&self, stage: i32, after: i32, upto: i32) -> Option<i32> {
        let pass = self.pass(stage)?;
        pass.kept()
            .map(|boundary| boundary.frame)
            .filter(|frame| *frame > after && *frame <= upto)
            .max()
    }

    fn pass(&self, stage: i32) -> Option<&Pass> {
        self.passes.get(usize::try_from(stage).ok()?)?.as_ref()
    }

    fn pass_mut(&mut self, stage: i32) -> Option<&mut Pass> {
        self.slot(stage)?.as_mut()
    }

    fn slot(&mut self, stage: i32) -> Option<&mut Option<Pass>> {
        self.passes.get_mut(usize::try_from(stage).ok()?)
    }

    /// Writes both files: the table to paste into the source, and everything this
    /// knows so that the next session starts where this one stopped.
    pub fn write(&self, game: &dyn Game) {
        self.put(&self.dir.join(TABLE_FILE), &self.table(game));
        self.put(&self.dir.join(STATE_FILE), &self.state(game));
    }

    fn put(&self, path: &Path, contents: &str) {
        match orb_api::fs::write(path, contents.as_bytes()) {
            Ok(()) => log!("tuning: wrote {}", path.display()),
            Err(error) => log!("tuning: cannot write {}: {error}", path.display()),
        }
    }

    /// The table as Rust, ready to replace the one in `chapters.rs`. Stages this
    /// knows nothing about keep the values already compiled in, so a stage can be
    /// tuned, baked and left alone while the next one is done.
    fn table(&self, game: &dyn Game) -> String {
        let table = game.midstage_table();
        let mut source = format!("pub const MIDSTAGE: [&[Boundary]; {}] = [\n", table.len());
        for (stage, built_in) in table.iter().enumerate() {
            let frames: Vec<String> = match self.pass(stage as i32) {
                Some(pass) => pass
                    .kept()
                    .map(|boundary| {
                        entry(
                            boundary.frame,
                            boundary.by_hand,
                            boundary.verdict == Verdict::Adjust,
                        )
                    })
                    .collect(),
                None => built_in
                    .iter()
                    .map(|built| entry(built.frame, built.by_hand, false))
                    .collect(),
            };
            let label = if stage + 1 == table.len() {
                "extra  ".to_owned()
            } else {
                format!("stage {}", stage + 1)
            };
            source += &format!("    /* {label} */ &[{}],\n", frames.join(", "));
        }
        source + "];\n"
    }

    fn state(&self, game: &dyn Game) -> String {
        let stages = game.midstage_table().len();
        let mut text = String::from(
            "# What the tuning pass has found, and what has been decided about it.\n\
             # Read back when orb starts, so that a stage can be judged over more than\n\
             # one session. Written whole every time: edit it while nothing is running.\n\
             #\n\
             # stage  script frame  keep|adjust|drop  proposed|hand\n",
        );
        for stage in 0..stages {
            let Some(pass) = self.pass(stage as i32) else {
                continue;
            };
            let label = if stage + 1 == stages {
                "extra".to_owned()
            } else {
                format!("stage {}", stage + 1)
            };
            text += &format!("\n# {label}\n");
            for boundary in &pass.boundaries {
                text += &format!(
                    "{} {} {} {}\n",
                    stage + 1,
                    boundary.frame,
                    boundary.verdict.name(),
                    if boundary.by_hand { "hand" } else { "proposed" },
                );
            }
        }
        text
    }

    /// Reads back what earlier sessions found. A missing file is the ordinary case
    /// the first time a stage is looked at.
    fn load(&mut self) {
        let path = self.dir.join(STATE_FILE);
        let text = match orb_api::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            // A file that is there and will not read, which no e2e test reaches: what an e2e test can hand
            // a launch is *bytes* — `Fake::attach_finding` — and this arm wants something else at the path
            // altogether. The two ways a line can be wrong below are covered.
            Err(error) => return log!("tuning: cannot read {}: {error}", path.display()),
        };
        let mut read = 0;
        for (number, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            match parse(line) {
                // The file's stages are counted from one, the way they are named.
                Some((stage, boundary)) => match self.slot(stage - 1) {
                    Some(slot) => {
                        slot.get_or_insert_with(Pass::new).put(boundary);
                        read += 1;
                    }
                    None => log!(
                        "tuning: {}:{}: no stage {stage}",
                        path.display(),
                        number + 1
                    ),
                },
                None => log!(
                    "tuning: {}:{}: cannot read `{line}`",
                    path.display(),
                    number + 1
                ),
            }
        }
        log!("tuning: read {read} boundary(s) from {}", path.display());
    }
}

/// One entry of the generated table.
///
/// Which hand it came from is the call and not a comment on the number, because the shortest a
/// chapter may be reads it: a boundary somebody placed is exempt and a proposal is not, so a
/// table that only remarked on it divided a stage differently in play than in the pass that
/// chose it. `adjust` stays a comment — nothing reads that, and what it means is that a boundary
/// is in roughly the right place while somebody works out where exactly.
fn entry(frame: i32, by_hand: bool, adjust: bool) -> String {
    let placed = if by_hand { "hand" } else { "proposed" };
    let note = if adjust { " /* adjust */" } else { "" };
    format!("{placed}({frame}){note}")
}

/// One line of the state file: the stage counted from one, then the boundary.
fn parse(line: &str) -> Option<(i32, Boundary)> {
    let mut fields = line.split_whitespace();
    // Held to being a stage number here and not where `stage - 1` indexes with it: the file
    // invites hand-editing, and a number below one overflows that subtraction. One malformed
    // line is then the `cannot read` every other malformed field here gets.
    let stage = fields.next()?.parse().ok().filter(|stage| *stage >= 1)?;
    let frame = fields.next()?.parse().ok()?;
    let verdict = Verdict::parse(fields.next()?)?;
    let by_hand = match fields.next() {
        Some("hand") => true,
        Some("proposed") | None => false,
        Some(_) => return None,
    };
    fields.next().is_none().then_some((
        stage,
        Boundary {
            frame,
            verdict,
            by_hand,
        },
    ))
}

impl Verdict {
    /// For the status line, which is the ASCII strip in the black beside the game.
    pub fn label(self) -> &'static str {
        match self {
            Self::Keep => "KEEP",
            Self::Adjust => "ADJUST",
            Self::Rejected => "DROP",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Adjust => "adjust",
            Self::Rejected => "drop",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name {
            "keep" => Some(Self::Keep),
            "adjust" => Some(Self::Adjust),
            "drop" => Some(Self::Rejected),
            _ => None,
        }
    }
}

impl Pass {
    fn new() -> Self {
        Self {
            boundaries: Vec::new(),
            covered: i32::MIN,
            without_enemies: 0,
        }
    }

    /// The ones the table would carry, in order.
    fn kept(&self) -> impl Iterator<Item = &Boundary> {
        self.boundaries
            .iter()
            .filter(|boundary| boundary.verdict != Verdict::Rejected)
    }

    fn at(&self, frame: i32) -> Option<Boundary> {
        let at = self
            .boundaries
            .binary_search_by_key(&frame, |boundary| boundary.frame)
            .ok()?;
        Some(self.boundaries[at])
    }

    fn forget(&mut self, frame: i32) {
        if let Ok(at) = self
            .boundaries
            .binary_search_by_key(&frame, |entry| entry.frame)
        {
            self.boundaries.remove(at);
        }
    }

    /// Adds or replaces, keeping the list sorted so that it can be searched and
    /// written in order.
    fn put(&mut self, boundary: Boundary) {
        match self
            .boundaries
            .binary_search_by_key(&boundary.frame, |entry| entry.frame)
        {
            Ok(at) => self.boundaries[at] = boundary,
            Err(at) => self.boundaries.insert(at, boundary),
        }
    }

    /// What the detector found. Anything already known keeps the verdict it has: what
    /// a proposal says is where a boundary might go, and a decision outranks it.
    fn propose(&mut self, frame: i32) {
        if self.at(frame).is_none() {
            self.put(Boundary {
                frame,
                verdict: Verdict::Keep,
                by_hand: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tuning;
    use crate::game::th06::Th06;

    /// What a second sitting has to find: every boundary, every verdict, and which of
    /// them nothing would propose again.
    #[test]
    fn what_a_session_decided_comes_back_in_the_next_one() {
        let dir = std::env::temp_dir().join("orb-tuning-round-trip");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join(super::STATE_FILE)).ok();

        let mut first = Tuning::new(&Th06, dir.clone());
        first.begin_stage(0);
        first.begin_stage(3);
        for frame in [1886, 2500, 5144] {
            first.add(0, frame);
        }
        first.add(3, 900);
        first.judge_down(0, 2500);
        first.reject(0, 5144);
        first.write(&Th06);

        let next = Tuning::new(&Th06, dir);
        assert_eq!(next.count(0), 2);
        assert_eq!(next.judged(0, 1886).unwrap().verdict.label(), "KEEP");
        assert_eq!(next.judged(0, 2500).unwrap().verdict.label(), "ADJUST");
        assert!(next.judged(0, 1886).unwrap().by_hand);
        // Refusing one that was put there by hand takes it out altogether, so there is
        // nothing of it to come back.
        assert!(next.judged(0, 5144).is_none());
        // Stages are counted from one in the file and from zero in the table.
        assert_eq!(next.count(3), 1);
        assert!(next.judged(3, 900).is_some());
        assert!(next.judged(1, 900).is_none());
    }

    /// The table this writes carries which hand each boundary came from as data, because the
    /// shortest a chapter may be reads it: a table that only said it in a comment divided a stage
    /// differently in play than in the pass that chose the numbers.
    #[test]
    fn the_written_table_says_which_hand_each_boundary_came_from() {
        let dir = std::env::temp_dir().join("orb-tuning-table");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join(super::STATE_FILE)).ok();

        let mut tuning = Tuning::new(&Th06, dir);
        tuning.begin_stage(0);
        tuning.add(0, 1886);
        let source = tuning.table(&Th06);
        assert!(source.contains("hand(1886)"), "{source}");
        // And a stage this knows nothing about keeps what is compiled in, hands and all.
        assert!(source.contains("hand(4597)"), "{source}");
        assert!(source.contains("proposed(880)"), "{source}");
    }

    /// A line whose stage is not a stage number is one that cannot be read, however far out it
    /// is: the number is counted from one and indexed with one taken off it, so nothing below
    /// one may reach that subtraction.
    #[test]
    fn a_stage_below_one_cannot_be_read() {
        assert!(super::parse("1 900 keep hand").is_some());
        for line in [
            "0 900 keep hand",
            "-1 900 keep",
            "-2147483648 900 keep hand",
        ] {
            assert!(super::parse(line).is_none(), "{line}");
        }
    }
}
