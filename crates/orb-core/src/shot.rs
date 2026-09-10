//! A picture of the frame, written beside the exe.
//!
//! **What a number cannot say.** A hit says where the player was, what it pressed, how much room the model
//! left and everything in the three arrays whose box reaches it — and a hit whose dump names nothing has
//! exhausted what the arrays can be asked. The picture is the other half: whatever hit the player was drawn,
//! so a frame of it settles what the thing looks like and where, against a model that says there was nothing
//! there.
//!
//! **Written at the end of the frame's drawing and not when the hit is noticed.** The state a hit is noticed by
//! is written inside the game's own update, which runs after orb reads the input, so the frame this lands on is
//! one on from the collision — the same frame the arrays are walked on, which is what makes the two readable
//! together. Asking earlier buys nothing: a back buffer handed back after `Present` holds whatever the swap
//! left in it, which for a discarding swap chain is undefined.
//!
//! **Copied into a surface of orb's own before its rows are read.** A back buffer can be locked only where the
//! device was created for it, and orb does not create the device — the game does, and long before orb is
//! attached. So the copy is what makes the frame readable, and `CopyRects` out of a render target is the one
//! direction Direct3D 8 allows it in.
//!
//! **A BMP, because the rows a lock hands over are already one.** A 32-bit bottom-up BMP is its pixels in the
//! order `D3DFMT_X8R8G8B8` keeps them and a 54-byte header in front, so nothing here encodes anything; a PNG
//! would put a compressor in orb for the sake of a file that is read once and thrown away.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::d3d8::{D3DBACKBUFFER_TYPE_MONO, D3DFMT_R5G6B5, D3DLOCK_READONLY};
use orb_api::{Device, Surface, clock, d3d8, fs, module};

/// The directory beside the exe the frames are written into.
const HELD_IN: &str = "snapshot";

/// How many one run writes.
///
/// Each is the whole screen at four bytes a pixel, so they are megabytes rather than kilobytes, and they go
/// beside the game where whoever is playing keeps their replays. A run that is hit twenty times has already
/// said what the first few would say, and the ones after that are a directory nobody asked for.
const AT_MOST: usize = 20;

/// What the next frame is to be written out as, and how many have been written.
static WANTED: Mutex<Option<String>> = Mutex::new(None);
static TAKEN: AtomicUsize = AtomicUsize::new(0);

/// **The label a picture asked for by the clock is taken under**, which is the one label whose picture goes
/// into the trace and nowhere else — see [`taken`].
pub const INTO_THE_TRACE: &str = "trace";

/// Ask for the frame being drawn to be written out, named after what asked for it.
///
/// **And [`AT_MOST`] is a bound on the files and not on the trace**: a picture asked for under
/// [`INTO_THE_TRACE`] is written into the trace and nowhere else, so what the bound is there to stop —
/// a directory of megabytes nobody asked for — cannot happen to it. A launch that took the bound as its
/// own kept the first twenty and then stopped: `--shot-every=10` over a half-hour run left every picture
/// inside the first two stages, and the screens a reader wants are the ones late in it.
pub fn wanted(label: &str) {
    if label != INTO_THE_TRACE && TAKEN.load(Ordering::Relaxed) >= AT_MOST {
        return;
    }
    *WANTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(label.to_owned());
}

/// Write out whatever was asked for, at the end of a frame's drawing.
///
/// # Safety
/// Must run on the thread the device belongs to, inside a frame whose scene has ended.
pub unsafe fn taken(device: Device) {
    let label = WANTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let Some(label) = label else {
        return;
    };
    let (_, back) = d3d8::get_back_buffer(device, 0, D3DBACKBUFFER_TYPE_MONO);
    let Some(back) = back else {
        crate::log!("shot: the device would not hand over its back buffer");
        return;
    };
    let taken = unsafe { off(device, back) };
    d3d8::release_surface(back);
    let Some(bytes) = taken else {
        return;
    };
    // **And into the trace where one is being written**, which is where a picture belongs: a trace already
    // holds the state, the word and every line the log said on the frame it was said on, so a frame's
    // picture beside those is the whole of what that frame looked like in the one file a check reads. See
    // [`crate::trace::picture`], and [`INTO_THE_TRACE`] for the label a periodic one is asked for under —
    // that one is not written beside the exe at all, a run's worth of them being a directory nobody asked
    // for.
    crate::trace::picture(&bytes);
    if label == INTO_THE_TRACE {
        return;
    }
    // Counted here and not above, [`AT_MOST`] being the bound on the files — see [`wanted`].
    TAKEN.fetch_add(1, Ordering::Relaxed);
    let Some(exe) = module::host_exe() else {
        return;
    };
    // **A directory of its own, made if it is not there.** Beside the exe is where the replays and the score
    // file are, and a run hit twenty times puts twenty megabytes of frames wherever it writes them; the one
    // place that should not be is among the files whoever is playing keeps.
    let held = exe.with_file_name(HELD_IN);
    if let Err(error) = fs::create_dir_all(&held) {
        crate::log!("shot: {} was not made: {error}", held.display());
        return;
    }
    // **The stamp the log puts on its own lines and not a count.** A count starts over at nought every launch,
    // so three practices put their first hits under one name and which picture went with which run stopped being
    // answerable; the stamp is the number in front of every line the log wrote, which is what makes a picture and
    // the dump of the hit it came out of findable from one another.
    let path = held.join(format!("{label}-{}.bmp", clock::ticks()));
    match fs::write(&path, &bytes) {
        Ok(()) => crate::log!("shot: {} holds the frame", path.display()),
        Err(error) => crate::log!("shot: {} was not written: {error}", path.display()),
    }
}

/// The frame's rows as a BMP, out of a copy of the back buffer.
///
/// # Safety
/// Must run on the thread the device belongs to.
unsafe fn off(device: Device, back: Surface) -> Option<Vec<u8>> {
    let desc = d3d8::surface_desc(back)?;
    let (_, copy) = d3d8::create_image_surface(device, desc.width, desc.height, desc.format);
    let copy = copy?;
    let made = d3d8::copy_rects(device, back, copy);
    let bytes = if made < 0 {
        crate::log!("shot: the frame would not copy, {made:#x}");
        None
    } else {
        d3d8::lock_surface(copy, D3DLOCK_READONLY).map(|locked| {
            let rows = unsafe { read(locked.bits, locked.pitch, &desc) };
            d3d8::unlock_surface(copy);
            bmp(desc.width, desc.height, &rows)
        })
    };
    d3d8::release_surface(copy);
    bytes
}

/// The locked rows as tightly packed blue-green-red-and-a-spare, top row first.
///
/// # Safety
/// `bits` must be the address a lock of a surface `desc` describes handed back.
unsafe fn read(bits: usize, pitch: i32, desc: &orb_api::SurfaceDesc) -> Vec<u8> {
    let across = desc.width as usize;
    let down = desc.height as usize;
    let mut rows = Vec::with_capacity(across * down * 4);
    let wide = desc.format == D3DFMT_R5G6B5;
    for y in 0..down {
        let row = bits + y * pitch.unsigned_abs() as usize;
        for x in 0..across {
            if wide {
                let held = unsafe { *((row + x * 2) as *const u16) };
                // Each channel back up to eight bits by repeating its own top bits, so that white stays
                // white — a plain shift left leaves the brightest 5-bit value at 0xf8 and greys the screen.
                let red = ((held >> 11) & 0x1f) as u8;
                let green = ((held >> 5) & 0x3f) as u8;
                let blue = (held & 0x1f) as u8;
                rows.push(blue << 3 | blue >> 2);
                rows.push(green << 2 | green >> 4);
                rows.push(red << 3 | red >> 2);
                rows.push(0xff);
            } else {
                let held = unsafe { *((row + x * 4) as *const u32) };
                rows.extend_from_slice(&held.to_le_bytes());
            }
        }
    }
    rows
}

/// A 32-bit bottom-up BMP over rows given top-first.
fn bmp(width: u32, height: u32, rows: &[u8]) -> Vec<u8> {
    const HEADER: u32 = 14 + 40;
    let stride = width as usize * 4;
    let pixels = stride * height as usize;
    let mut out = Vec::with_capacity(HEADER as usize + pixels);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(HEADER + pixels as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&HEADER.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&(height as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(pixels as u32).to_le_bytes());
    for _ in 0..4 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    for y in (0..height as usize).rev() {
        let row = y * stride;
        match rows.get(row..row + stride) {
            Some(row) => out.extend_from_slice(row),
            None => out.resize(out.len() + stride, 0),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bmp_says_its_size_and_holds_its_rows_bottom_first() {
        let top = [1u8, 2, 3, 4];
        let bottom = [5u8, 6, 7, 8];
        let out = bmp(1, 2, &[top, bottom].concat());
        assert_eq!(&out[..2], b"BM");
        assert_eq!(u32::from_le_bytes(out[2..6].try_into().unwrap()), 54 + 8);
        assert_eq!(u32::from_le_bytes(out[10..14].try_into().unwrap()), 54);
        assert_eq!(i32::from_le_bytes(out[18..22].try_into().unwrap()), 1);
        assert_eq!(i32::from_le_bytes(out[22..26].try_into().unwrap()), 2);
        assert_eq!(u16::from_le_bytes(out[28..30].try_into().unwrap()), 32);
        assert_eq!(&out[54..58], &bottom);
        assert_eq!(&out[58..62], &top);
    }

    #[test]
    fn a_bmp_of_rows_that_ran_short_is_still_the_size_it_says() {
        let out = bmp(2, 2, &[9u8; 8][..4]);
        assert_eq!(out.len(), 54 + 16);
        assert_eq!(&out[54..58], &[0, 0, 0, 0]);
    }
}
