//! Bakes the brush stroke that goes over the lives out of the picture of one beside this file.
//!
//! Here rather than in a script run by hand, and that is the whole reason it is here: the picture
//! is the source and the coverage is a build artefact, so a script would mean the artefact
//! committed beside the picture it came from — two copies of one stroke, and the editable one the
//! wrong one. This way `brush.png` is the only copy in the tree and nothing has to be re-run by
//! anybody.
//!
//! What it does: the picture is ink on paper, so the coverage is how far a pixel is from the paper
//! towards the ink — the paper being the commonest value in it. Cropped to the ink, scaled by area,
//! which keeps the amount of ink right where sampling a pixel here and there would lose a hair
//! entirely, and then put through a contrast curve, because at a thirtieth of the original height
//! the hairs and the dry slivers average into grey.

use std::fmt::Write as _;
use std::path::PathBuf;

/// The size the stroke is drawn at, which is what it is baked to: as wide as the row the game
/// draws its count of lives in, and taller than that row because a stroke is not the shape of a
/// row. The picture's ink is 4:1 and this is 4.8:1 — the squash is what fits a stroke between the
/// score's row and the bombs'.
const WIDTH: usize = 144;
const HEIGHT: usize = 30;
/// How many times the smoothstep is applied to what the averaging gave. Grey that came from a hair
/// goes towards ink and grey that came from a sliver towards paper; without it the whole stroke
/// comes out of the scaling soft.
const CONTRAST_PASSES: usize = 2;

/// Every fifth pixel each way, for finding the paper. The paper is most of the picture, so a
/// twenty-fifth of it says which green that is and reading all of it says the same thing slower.
const PAPER_STRIDE: usize = 5;
/// Half coverage is the ink's edge, which is what the crop is measured to.
const EDGE: f32 = 0.5;

fn main() {
    let picture = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("brush.png");
    println!("cargo::rerun-if-changed={}", picture.display());
    println!("cargo::rerun-if-changed=build.rs");

    let bytes =
        std::fs::read(&picture).unwrap_or_else(|error| panic!("{}: {error}", picture.display()));
    let picture = Png::read(&bytes);
    let (ink, box_) = picture.ink();
    let coverage = scaled(&ink, picture.width, box_);

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("brush.rs");
    std::fs::write(&out, rust(&coverage))
        .unwrap_or_else(|error| panic!("{}: {error}", out.display()));
}

/// The picture, decoded far enough for this one job: eight bits a channel, not interlaced, and grey
/// or truecolour with or without an alpha channel. Anything else is this build failing rather than a
/// stroke coming out wrong, since the picture is one this repository carries.
struct Png {
    width: usize,
    height: usize,
    channels: usize,
    rows: Vec<Vec<u8>>,
}

impl Png {
    fn read(bytes: &[u8]) -> Self {
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "brush.png is not a PNG");
        let (mut width, mut height, mut channels) = (0usize, 0usize, 0usize);
        let mut deflated = Vec::new();
        let mut at = 8;
        while at + 8 <= bytes.len() {
            let length = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let kind = &bytes[at + 4..at + 8];
            let data = &bytes[at + 8..at + 8 + length];
            match kind {
                b"IHDR" => {
                    width = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
                    height = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
                    let (depth, colour, interlace) = (data[8], data[9], data[12]);
                    assert_eq!(depth, 8, "brush.png is not eight bits a channel");
                    assert_eq!(interlace, 0, "brush.png is interlaced");
                    channels = match colour {
                        0 => 1,
                        2 => 3,
                        6 => 4,
                        other => panic!("brush.png is colour type {other}, not grey or truecolour"),
                    };
                }
                b"IDAT" => deflated.extend_from_slice(data),
                _ => {}
            }
            at += 12 + length;
        }
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&deflated)
            .expect("brush.png's pixels do not inflate");

        // Undo the per-row filters, which is the whole of what a PNG's rows are wrapped in.
        let stride = width * channels;
        let mut rows: Vec<Vec<u8>> = Vec::with_capacity(height);
        let mut at = 0;
        for y in 0..height {
            let filter = raw[at];
            at += 1;
            let mut row = raw[at..at + stride].to_vec();
            at += stride;
            let previous: &[u8] = if y == 0 { &[] } else { &rows[y - 1] };
            let above = |i: usize| if y == 0 { 0 } else { previous[i] };
            for i in 0..stride {
                let left = if i >= channels { row[i - channels] } else { 0 };
                let up = above(i);
                let corner = if i >= channels && y > 0 {
                    previous[i - channels]
                } else {
                    0
                };
                row[i] = match filter {
                    0 => row[i],
                    1 => row[i].wrapping_add(left),
                    2 => row[i].wrapping_add(up),
                    3 => row[i].wrapping_add(((u16::from(left) + u16::from(up)) / 2) as u8),
                    4 => row[i].wrapping_add(paeth(left, up, corner)),
                    other => panic!("brush.png row {y} has filter {other}"),
                };
            }
            rows.push(row);
        }
        Self {
            width,
            height,
            channels,
            rows,
        }
    }

    /// What the ink is measured in: the grey of a grey picture, and the green of a colour one —
    /// green because ink on green paper is what these were, and green is the channel that loses
    /// most of itself to it. A colour picture bakes to the same bytes as its own grey, this being
    /// the only channel either of them is read for.
    fn value(&self, x: usize, y: usize) -> u8 {
        let channel = if self.channels == 1 { 0 } else { 1 };
        self.rows[y][x * self.channels + channel]
    }

    /// The ink, 0..1 a pixel, and the box it lies inside.
    fn ink(&self) -> (Vec<f32>, [usize; 4]) {
        let mut counts = [0u32; 256];
        for y in (0..self.height).step_by(PAPER_STRIDE) {
            for x in (0..self.width).step_by(PAPER_STRIDE) {
                counts[usize::from(self.value(x, y))] += 1;
            }
        }
        let paper = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map(|(green, _)| green)
            .unwrap();
        let darkest = counts.iter().position(|count| *count > 0).unwrap();
        let span = (paper.saturating_sub(darkest)).max(1) as f32;

        let mut ink = vec![0.0; self.width * self.height];
        let (mut left, mut top, mut right, mut bottom) = (self.width, self.height, 0, 0);
        for y in 0..self.height {
            for x in 0..self.width {
                let a = ((paper as f32 - f32::from(self.value(x, y))) / span).clamp(0.0, 1.0);
                ink[y * self.width + x] = a;
                if a > EDGE {
                    left = left.min(x);
                    right = right.max(x);
                    top = top.min(y);
                    bottom = bottom.max(y);
                }
            }
        }
        assert!(left <= right && top <= bottom, "brush.png has no ink in it");
        (ink, [left, top, right, bottom])
    }
}

fn paeth(left: u8, up: u8, corner: u8) -> u8 {
    let (a, b, c) = (i32::from(left), i32::from(up), i32::from(corner));
    let guess = a + b - c;
    let (da, db, dc) = ((guess - a).abs(), (guess - b).abs(), (guess - c).abs());
    if da <= db && da <= dc {
        left
    } else if db <= dc {
        up
    } else {
        corner
    }
}

/// Averages the ink inside `box_` into `WIDTH` x `HEIGHT`, by area, and curves the result.
fn scaled(ink: &[f32], stride: usize, box_: [usize; 4]) -> Vec<u8> {
    let [left, top, right, bottom] = box_;
    let (source_width, source_height) = ((right - left + 1) as f32, (bottom - top + 1) as f32);
    let mut out = Vec::with_capacity(WIDTH * HEIGHT);
    for y in 0..HEIGHT {
        let y0 = top as f32 + source_height * y as f32 / HEIGHT as f32;
        let y1 = top as f32 + source_height * (y + 1) as f32 / HEIGHT as f32;
        for x in 0..WIDTH {
            let x0 = left as f32 + source_width * x as f32 / WIDTH as f32;
            let x1 = left as f32 + source_width * (x + 1) as f32 / WIDTH as f32;
            let (mut total, mut weight) = (0.0, 0.0);
            for sy in y0 as usize..=y1 as usize {
                let covered_y = y1.min(sy as f32 + 1.0) - y0.max(sy as f32);
                if covered_y <= 0.0 {
                    continue;
                }
                for sx in x0 as usize..=x1 as usize {
                    let covered_x = x1.min(sx as f32 + 1.0) - x0.max(sx as f32);
                    if covered_x <= 0.0 {
                        continue;
                    }
                    let area = covered_x * covered_y;
                    total += ink[sy * stride + sx] * area;
                    weight += area;
                }
            }
            let mut a = if weight > 0.0 { total / weight } else { 0.0 };
            for _ in 0..CONTRAST_PASSES {
                a = a * a * (3.0 - 2.0 * a);
            }
            out.push((255.0 * a + 0.5) as u8);
        }
    }
    out
}

fn rust(coverage: &[u8]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Generated by build.rs from brush.png. What it is for, and why the stroke is a picture\n\
         // rather than an algorithm, is in lives_ui.rs.\n\
         pub const WIDTH: u32 = {WIDTH};\n\
         pub const HEIGHT: u32 = {HEIGHT};\n\
         /// Row by row from the top, one byte of coverage per pixel.\n\
         pub const COVERAGE: [u8; {}] = [",
        WIDTH * HEIGHT
    );
    for row in coverage.chunks(WIDTH) {
        let _ = writeln!(
            out,
            "    {}",
            row.iter().fold(String::new(), |mut line, a| {
                let _ = write!(line, "{a},");
                line
            })
        );
    }
    let _ = writeln!(out, "];");
    out
}
