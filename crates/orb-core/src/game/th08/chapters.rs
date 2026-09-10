//! Midstage chapter boundaries, as script frame numbers.
//!
//! **Six rows off one `--collect` pass over the game's own replay** — `replay/th8_01.rpy`, the Normal run that
//! cleared Kaguya's stage, walked Stage 1, 2, 3, 4b, 5 and 6b — every boundary the detector proposed and none
//! moved: a gap of sixty frames with nothing to shoot at and no boss up, at least 120 frames into the chapter
//! before it. See `orb_core::tuning` for the pass, and `collect.yaml` beside this file for the walk that
//! reaches the replay — driven as `--autoplay=collect.yaml --collect --keep-in=<an empty directory>`, the
//! last so that no `tuning.txt` of an earlier pass is read back into this one.
//!
//! **And each of the rows that replay never went through is off a replay of its own**, the pass reading one
//! replay a launch: stage 6a, Eirin's, which a run reaches only before the route to Kaguya is open, off
//! `replay/th8_02.rpy`, a Normal run that cleared her stage — the pass proposed rows for the stages before it
//! as well, and the ones above are kept, being the first replay's and just as measured; and the Extra off
//! `replay/th8_03.rpy`, an Extra cleared with bombs. **Its last, `9558`, is inside Mokou's fight**: the script
//! clock stands at `9498` while she is up and runs on for the frames between two of her cards, when she is
//! not, which is a gap the detector reads like any other and a place a chapter begins at a card's edge.
//! **And stage 4a**, the other of the fourth stage's two, which only the second and third teams' runs go
//! through, off a Normal run of the second team played for it and cleared — its replay the copy's first slot
//! once more, which is where a cleared run writes.
//!
//! **And a pass made before the enemies nothing can shoot were counted out proposed nothing before stage 5**
//! — see `ENEMY_UNTESTED` in `th08/mod.rs`: stage 1 keeps one alive through its whole midsection, so a count
//! of the live enemies never reached nothing there. Every row here is a pass's made after that, into a
//! directory of its own.
//!
//! **And not th07's rows copied across.** A boundary is a measurement of a run and not a reading of the
//! exe, so a number written here that nobody measured against `th08.exe` is a number a tuning pass
//! would have to find wrong before it could find its own. th08's stages are not th07's stages either:
//! the waves differ, and so does what the player is doing between them.
//!
//! **Nine rows, which is what this game's own tables hold** — see the stage table in `asm.md`: the two
//! `.std` tables at `0x4c731c` and `0x4c7340` and the `.ecl` one at `0x4c73cc` each have nine entries,
//! indexed by `0x164d2cc` counted from zero, and the name table beside them names the same nine. The
//! Last Words are the tenth name and none of those nine, loading `ecldata_yk.ecl` and its five siblings
//! out of a table of their own, so they are not a row here.

use crate::game::{Boundary, proposed};

/// Indexed by the stage counted from zero, which is what [`Th08::read_state`](super::Th08) answers:
/// `0x164d2cc` as it stands — see `G_STAGE` in `th08/mod.rs`, which this game already counts from
/// nothing.
///
/// Nine rows: the six stages a Normal run plays, the two halves the fourth and the last are each
/// split into, and the Extra.
pub const MIDSTAGE: [&[Boundary]; 9] = [
    /* stage 1        */ &[proposed(790), proposed(4086)],
    /* stage 2        */ &[proposed(4303), proposed(4463), proposed(4762)],
    /* stage 3        */
    &[
        proposed(1224),
        proposed(1542),
        proposed(2178),
        proposed(2979),
        proposed(3142),
        proposed(3413),
        proposed(4607),
    ],
    /* stage 4 uncanny  */ &[proposed(1214), proposed(4612), proposed(5412)],
    /* stage 4 powerful */
    &[
        proposed(512),
        proposed(883),
        proposed(1189),
        proposed(2244),
        proposed(3145),
        proposed(3530),
        proposed(4612),
        proposed(5404),
    ],
    /* stage 5        */
    &[
        proposed(612),
        proposed(2850),
        proposed(4530),
        proposed(5917),
    ],
    /* stage 6 Eirin  */
    &[
        proposed(818),
        proposed(1424),
        proposed(1938),
        proposed(2317),
        proposed(2689),
        proposed(3215),
        proposed(4083),
    ],
    /* stage 6 Kaguya */
    &[
        proposed(857),
        proposed(1613),
        proposed(2243),
        proposed(2833),
        proposed(3435),
    ],
    /* extra          */
    &[
        proposed(1011),
        proposed(2393),
        proposed(2939),
        proposed(3178),
        proposed(3498),
        proposed(4137),
        proposed(4284),
        proposed(4442),
        proposed(4955),
        proposed(5723),
        proposed(6776),
        proposed(9558),
    ],
];
