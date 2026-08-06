//! The joystick read, off the game's thread.
//!
//! `Controller::GetControllerInput` asks winmm for joystick 0 once a frame, and where no
//! joystick answers, that one call takes 8.7ms and spends nearly all of it on the CPU:
//! winmm looking for a device, not waiting on one. Against a 16.67ms frame it was the
//! whole of the frame-pacing trouble.
//!
//! **Measured, both branches, outside the game by a program that asks the way the game asks** —
//! `joyGetPosEx` with `JOY_RETURNALL`, and `EnumDevices` for `DI8DEVCLASS_GAMECTRL` with
//! `DIEDFL_ATTACHEDONLY` then `Poll`, `Acquire`, `GetDeviceState` behind a foreground exclusive
//! acquire — around `QueryPerformanceCounter`, with `GetThreadTimes` across the hundred calls for the
//! split between work and waiting:
//!
//! | | |
//! | --- | --- |
//! | with nothing plugged in | `joyGetPosEx(0, JOY_RETURNALL)` answered `JOYERR_PARMS` in 8.7ms, min 7.8 max 10.8 over 100 calls; 100 back to back took 917ms of wall clock against 906ms of CPU — 703 kernel, 203 user |
//! | with an Xbox One pad attached | the same call returns in under a microsecond |
//! | the branch the game takes where `EnumDevices` found a controller | DirectInput, about a microsecond a frame: `GetDeviceState` 1µs average and 9µs worst over 100 reads, `Poll` answering `DI_NOEFFECT` in under a microsecond, and one 2.1ms `Acquire` on the first read |
//! | what the frames it ran on cost | `joystick=8562us/frame worst=13227us` in the log, against about 1µs for the keyboard |
//!
//! So the setting that turned the joystick off was never the answer to this and is gone: the call is
//! only expensive where no device answers, which is also the only case DirectInput leaves the game in
//! this branch for.
//!
//! Because it is work rather than waiting, moving it to a quieter part of the frame buys
//! nothing. So a thread of orb's own takes the samples, and the game's call is answered
//! out of the last one. With nothing plugged in that costs the frame `input=1us/frame worst=4us
//! calls=600` and `joystick=0us/frame worst=1us calls=600` over 600 frames, the frame itself
//! `16763us worst=23817us`, while the thread's own reads were 32ms for the first in the process —
//! winmm's joystick support coming up — and 8.7ms once a second after it. What a sample means is left
//! to the game: which button is shot,
//! where an axis becomes a direction, the auto-repeat behind holding one — none of that
//! is orb's to reimplement, and all of it is downstream of the call this replaces.
//!
//! One thing the game cannot be left to do for itself. `GetControllerInput` places the centre of each
//! axis at `(wXmin + wXmax) / 2` with a dead zone of a quarter of the travel, read out of the
//! `JOYCAPSA` at 0x69d760 — an address that appears exactly once in the whole exe, in the
//! `joyGetDevCapsA` call the startup check makes, and that check only reads it where a joystick
//! answered `joyGetPosEx` first. A pad that turns up after that
//! is measured against zeros — its centred axes, 32767 of a 65535 travel, both read as far
//! over, and the game spends the rest of the run with two directions held. So the
//! calibration is handed over with the sample it belongs to.
//!
//! Watched doing it: a run that started with the pad asleep (`there is no joystick 0, read in
//! 33785us`), had it wake mid-run (`mid=045e pid=02ff ... 16 buttons, 5 axes, X 0..65535, read in
//! 2494us`), took the calibration on the next frame (`the game's axis calibration was not this
//! device's`), and was then driven through the menus with nothing drifting — reads settled at 2µs on
//! the 4ms cadence, the frame at `input=2us/frame worst=180us calls=600`.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows_sys::Win32::Globalization::{CP_ACP, MultiByteToWideChar};
use windows_sys::Win32::Media::Multimedia::{
    JOYCAPSA, JOYERR_NOERROR, JOYERR_PARMS, JOYERR_UNPLUGGED, JOYINFOEX, joyGetDevCapsA,
    joyGetPosEx,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, Sleep, THREAD_PRIORITY_BELOW_NORMAL,
};

use crate::game::Reading;
use crate::{detail, log};
use crate::{hook, profile};

/// The joystick the game asks about. It only ever asks about this one.
const DEVICE: u32 = 0;
/// `JOY_RETURNALL`, which is what the game asks for, so a sample taken with it answers
/// anything the game can ask. Not in `windows-sys`.
const RETURN_ALL: u32 = 0xff;

/// Between reads while a joystick is answering. Four samples a frame, so what the game
/// is handed is never a frame behind what the stick is doing.
const ATTACHED_MS: u32 = 4;
/// And while none is. Asking every frame to be told again that there is nothing there is
/// what this module exists to stop, and doing it on another thread would only move the
/// cost onto another core. A joystick plugged in mid-session is picked up within this.
const DETACHED_MS: u32 = 1000;
/// How many reads a `verbose` line covers: a second's worth while a joystick answers, and
/// far rarer than that while none does, which is when there is least to say about it.
const REPORT_READS: u32 = 250;

/// The last read, and what the call that took it returned. `None` until the thread has
/// been round once.
///
/// A lock between orb's thread and the game's frame is only safe because a snapshot
/// suspends the threads the game made and this is not one of them: they are remembered
/// through the exe's `CreateThread` import, and orb's own calls do not go through it. One
/// suspended holding this would stop the game's next read for the length of a snapshot.
static SAMPLE: Mutex<Option<Sample>> = Mutex::new(None);
/// winmm's own `joyGetPosEx`, which the game's import table pointed at.
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
/// Where the game keeps the caps it measures axes against. Zero for a game that keeps none.
static CALIBRATION: AtomicUsize = AtomicUsize::new(0);
/// Whether the thread has been asked for. Never cleared: a failed spawn leaves every
/// call going to winmm, which is the behaviour orb replaced and not a broken one.
static POLLING: AtomicBool = AtomicBool::new(false);
/// Where the game's calibration last differed from the device's, plus one so that zero means
/// nothing yet.
static REPORTED_DIFFERENCE: AtomicUsize = AtomicUsize::new(0);

type JoyGetPosEx = unsafe extern "system" fn(u32, *mut JOYINFOEX) -> u32;

/// The game passes 0x194 to `joyGetDevCapsA`, so this is the struct it means by that.
const _: () = assert!(size_of::<JOYCAPSA>() == 0x194);

#[derive(Clone, Copy)]
struct Sample {
    result: u32,
    info: JOYINFOEX,
    /// The device's own, taken when one starts answering and kept for as long as it does.
    ///
    /// Not read again beside every position: the caps belong to the device and not to the
    /// read, and what the game is handed must not change under it from one read to the next
    /// — that is a write into the game's memory each time it does. A pad swapped for another
    /// without one failing read in between would keep the first one's, which at a read every
    /// four milliseconds is not a way pads are swapped.
    caps: Option<JOYCAPSA>,
}

impl Sample {
    /// Whether what answered is a pad, which is not the same as something having answered.
    ///
    /// A device with no buttons and no axes has nothing to say, and joystick 0 is one of those on
    /// this machine whenever the pad is in XInput's second slot: `mid=413d pid=2104`, answering
    /// `joyGetPosEx` with every field zero. Measured with all three interfaces asked at once — winmm
    /// reports 16 devices, index 0 being that one and 1 to 15 `JOYERR_UNPLUGGED` at 13µs each;
    /// DirectInput enumerates `Controller (Xbox 360 Controller)`; XInput has it in slot 1 with slot 0
    /// empty. Believing it
    /// costs a line in the log claiming a pad answered, and the game's axis calibration written
    /// from a device that has no axes.
    fn is_a_pad(&self) -> bool {
        self.result == JOYERR_NOERROR
            && self
                .caps
                .is_some_and(|caps| caps.wNumButtons > 0 || caps.wNumAxes > 0)
    }
}

/// Points the game's `joyGetPosEx` at orb, and takes the address of the caps it measures
/// axes against so that a device arriving mid-run can be described to it.
///
/// # Safety
/// `module` must be the game exe, and nothing may be executing its import table.
pub unsafe fn install(module: usize, calibration: Option<usize>) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "WINMM.dll",
            "joyGetPosEx",
            hook::address(answer as _),
        )
    }?;
    ORIGINAL.store(previous, Ordering::Relaxed);
    CALIBRATION.store(calibration.unwrap_or(0), Ordering::Relaxed);
    Ok(())
}

/// Replaces `joyGetPosEx` for the game's three callers of it: the "is there a pad"
/// check at startup, the caps read behind it, and the per-frame one.
unsafe extern "system" fn answer(device: u32, into: *mut JOYINFOEX) -> u32 {
    start_polling();
    if device == DEVICE
        && !into.is_null()
        && unsafe { describes_a_sample(&*into) }
        && let Some(sample) = latest()
    {
        // Only a device that is a pad writes the game's calibration: the axes of one that has
        // none describe nothing, and this write lands in the game's own memory.
        if let Some(caps) = sample.caps.filter(|_| sample.is_a_pad()) {
            unsafe { calibrate(&caps) };
        }
        unsafe { *into = sample.info };
        return sample.result;
    }
    // Another joystick, a struct this does not describe, or the very first call — which
    // is the game's startup check, and arrives before the thread has anything. Those go
    // to winmm as they always did rather than being answered wrongly.
    let original: JoyGetPosEx = unsafe { std::mem::transmute(ORIGINAL.load(Ordering::Relaxed)) };
    unsafe { original(device, into) }
}

/// Whether a sample taken with `RETURN_ALL` says everything this caller asked for.
fn describes_a_sample(asked: &JOYINFOEX) -> bool {
    asked.dwSize as usize == size_of::<JOYINFOEX>() && asked.dwFlags & !RETURN_ALL == 0
}

/// Puts the answering device's caps where the game reads them, so the axes it is about to
/// place the centre of are the axes it has.
///
/// Every read rather than once when the device appeared: a chapter restored from before it
/// appeared rewinds the game's `.data`, and the caps are in there.
///
/// # Safety
/// Must run on the thread that reads them, which is the game's: the write is not atomic and
/// the game reads it a few instructions after this returns.
unsafe fn calibrate(caps: &JOYCAPSA) {
    let at = CALIBRATION.load(Ordering::Relaxed);
    if at == 0 {
        return;
    }
    let ours = std::ptr::from_ref(caps).cast::<u8>();
    let ours = unsafe { std::slice::from_raw_parts(ours, size_of::<JOYCAPSA>()) };
    let theirs = unsafe { std::slice::from_raw_parts(at as *const u8, size_of::<JOYCAPSA>()) };
    let Some(differs) = ours
        .iter()
        .zip(theirs)
        .position(|(ours, theirs)| ours != theirs)
    else {
        return;
    };
    unsafe { std::ptr::copy_nonoverlapping(ours.as_ptr(), at as *mut u8, ours.len()) };
    // Where it differed, and only when that is somewhere new. A write that has to happen
    // again says something is putting the game's copy back — a restored chapter is the
    // expected way, anything else is a finding — and one line says it as well as a line a
    // frame would.
    if REPORTED_DIFFERENCE.swap(differs + 1, Ordering::Relaxed) != differs + 1 {
        log!(
            "joystick: the game's axis calibration was not this device's from +{differs:#x}; set from its caps"
        );
    }
}

fn latest() -> Option<Sample> {
    *SAMPLE.lock().ok()?
}

/// The pad as it was last sampled, and `None` while none is answering.
///
/// For the menus orb puts up itself. Those freeze the game, so the game's own reading of the pad
/// is not running either and a pad would do nothing at all on them; the sample this thread already
/// takes every few milliseconds is there to be read.
pub fn reading() -> Option<Reading> {
    let sample = latest().filter(Sample::is_a_pad)?;
    Some(Reading {
        buttons: sample.info.dwButtons,
        y: sample.info.dwYpos,
        pov: sample.info.dwPOV,
    })
}

fn start_polling() {
    if POLLING
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    // Not from `DllMain`, where the rest of orb's hooks go in: a thread created while the
    // loader lock is held cannot run its own attach notifications, and the game is
    // suspended there, so nothing would be answered anyway. The first call the game makes
    // is the startup check, which is a fine place to start from.
    if std::thread::Builder::new().spawn(poll).is_err() {
        log!("joystick: cannot start a thread; the read stays in the game's frame");
    }
}

fn poll() -> ! {
    // Below the game's, because a sample a millisecond old costs nobody anything and a
    // late frame costs the thing orb was written for.
    unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL) };
    let mut reported = None;
    let mut polls = 0;
    let mut total = 0;
    let mut caps = None;
    loop {
        let mut info: JOYINFOEX = unsafe { std::mem::zeroed() };
        info.dwSize = size_of::<JOYINFOEX>() as u32;
        info.dwFlags = RETURN_ALL;
        let started = profile::now();
        // orb's own import rather than the one `install` redirected: this is winmm's, and
        // the cost this module exists to move is right here.
        let result = unsafe { joyGetPosEx(DEVICE, &mut info) };
        // Only where one answered, and only where the last read did not: the caps of a
        // device that is not there cost what its position costs, and a device that is there
        // has the caps it had a moment ago.
        caps = match (result, caps) {
            (JOYERR_NOERROR, None) => read_caps(),
            (JOYERR_NOERROR, known) => known,
            _ => None,
        };
        let cost = profile::since(started);
        let sample = Sample { result, info, caps };
        if let Ok(mut held) = SAMPLE.lock() {
            *held = Some(sample);
        }

        if reported != Some(result) {
            reported = Some(result);
            (polls, total) = (0, 0);
            log!("joystick: {}, read in {cost}us", describe(&sample));
        }
        polls += 1;
        total += cost;
        // Each line covers its own reads and no others: one that averaged everything since
        // the thread started would spend a minute walking the first read down — 12ms of it,
        // where winmm found the device — and read as though the reads were getting faster.
        // What device they are of stands in the line above rather than being asked again,
        // since naming it means another read of the thing this stops reading often.
        if polls == REPORT_READS {
            detail!(
                "joystick: {polls} reads, {}us each",
                total / i64::from(polls)
            );
            (polls, total) = (0, 0);
        }

        let wait = if result == JOYERR_NOERROR {
            ATTACHED_MS
        } else {
            DETACHED_MS
        };
        // Never sooner than the read itself took, so a device slow enough to cost what no
        // device at all costs cannot hold a core of its own.
        unsafe { Sleep(wait.max((cost.max(0) / 1000) as u32)) };
    }
}

fn read_caps() -> Option<JOYCAPSA> {
    let mut caps: JOYCAPSA = unsafe { std::mem::zeroed() };
    let read = unsafe { joyGetDevCapsA(DEVICE as usize, &mut caps, size_of::<JOYCAPSA>() as u32) };
    (read == JOYERR_NOERROR).then_some(caps)
}

/// What a sample says, with the device named where one answered.
fn describe(sample: &Sample) -> String {
    match (sample.result, &sample.caps) {
        // Named all the same, since which phantom it is is the thing to look up: 413d:2104 is
        // what Windows leaves on joystick 0 while the pad it has is in XInput's second slot.
        (JOYERR_NOERROR, Some(caps)) if !sample.is_a_pad() => {
            let (mid, pid) = (caps.wMid, caps.wPid);
            format!(
                "joystick 0 is mid={mid:04x} pid={pid:04x} \"{}\" with no buttons and no axes, \
                 which is no pad; orb's own menus will not be driven from it",
                name(caps.szPname),
            )
        }
        (JOYERR_NOERROR, Some(caps)) => {
            // Copied out one at a time: `JOYCAPSA` is packed, so its fields cannot be
            // borrowed, which is what passing one to `format!` would do.
            let (mid, pid) = (caps.wMid, caps.wPid);
            let (buttons, axes) = (caps.wNumButtons, caps.wNumAxes);
            let (low, high) = (caps.wXmin, caps.wXmax);
            format!(
                "mid={mid:04x} pid={pid:04x} \"{}\", {buttons} buttons, {axes} axes, X {low}..{high}",
                name(caps.szPname),
            )
        }
        (JOYERR_NOERROR, None) => "a joystick whose caps do not read".to_owned(),
        (JOYERR_PARMS, _) => "there is no joystick 0".to_owned(),
        (JOYERR_UNPLUGGED, _) => "joystick 0 is unplugged".to_owned(),
        (other, _) => format!("joyGetPosEx returns {other}"),
    }
}

/// `szPname`, which winmm gives in the machine's code page and the log is written in UTF-8:
/// the name here read `Microsoft PC �W���C�X�e�B�b�N` before this converted it, and a line
/// nobody can read is no answer to which device it is.
fn name(field: [i8; 32]) -> String {
    let ansi: Vec<u8> = field
        .iter()
        .take_while(|byte| **byte != 0)
        .map(|byte| *byte as u8)
        .collect();
    let mut wide = [0u16; 32];
    let written = unsafe {
        MultiByteToWideChar(
            CP_ACP,
            0,
            ansi.as_ptr(),
            ansi.len() as i32,
            wide.as_mut_ptr(),
            wide.len() as i32,
        )
    };
    if written > 0 {
        String::from_utf16_lossy(&wide[..written as usize])
    } else {
        String::from_utf8_lossy(&ansi).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The caps are 0x194 bytes and the game keeps its auto-repeat counter in the four
    /// immediately after them — `G_JOY_CAPS` + 0x194 is 0x69d8f4, which
    /// `GetControllerInput` counts a held direction in. A write that runs over lands there.
    #[test]
    fn a_calibration_write_stops_where_the_caps_do() {
        #[repr(C)]
        struct AsTheGameHasThem {
            caps: JOYCAPSA,
            auto_repeat: u32,
        }
        let mut theirs = AsTheGameHasThem {
            caps: unsafe { std::mem::zeroed() },
            auto_repeat: 0x0102_0304,
        };
        let mut ours: JOYCAPSA = unsafe { std::mem::zeroed() };
        ours.wXmax = 65535;
        ours.wNumButtons = 16;

        CALIBRATION.store(
            std::ptr::from_mut(&mut theirs.caps) as usize,
            Ordering::Relaxed,
        );
        unsafe { calibrate(&ours) };

        let (travel, buttons, beyond) = (
            theirs.caps.wXmax,
            theirs.caps.wNumButtons,
            theirs.auto_repeat,
        );
        assert_eq!((travel, buttons), (65535, 16));
        assert_eq!(beyond, 0x0102_0304);

        // What the caps are put back over is a restored chapter's, which is why the handover
        // happens with every read and not once when the device turned up.
        theirs.caps.wXmax = 0;
        unsafe { calibrate(&ours) };
        let travel = theirs.caps.wXmax;
        assert_eq!(travel, 65535);
    }

    /// A device answering with no buttons and no axes is not a pad, and what makes that worth
    /// testing is that Windows leaves exactly one of those on joystick 0 while the pad it has
    /// sits in XInput's second slot.
    #[test]
    fn a_device_with_nothing_on_it_is_not_a_pad() {
        let mut caps: JOYCAPSA = unsafe { std::mem::zeroed() };
        caps.wXmax = 65535;
        let phantom = Sample {
            result: JOYERR_NOERROR,
            info: unsafe { std::mem::zeroed() },
            caps: Some(caps),
        };
        assert!(!phantom.is_a_pad());

        // A stick with axes and no buttons is still a pad, and so is a wheel with buttons and
        // one axis: either can say something.
        caps.wNumAxes = 2;
        let stick = Sample {
            caps: Some(caps),
            ..phantom
        };
        assert!(stick.is_a_pad());

        // And nothing at all answering is not one either, whatever it left in the caps.
        let nothing = Sample {
            result: JOYERR_UNPLUGGED,
            ..stick
        };
        assert!(!nothing.is_a_pad());
    }

    /// A sample is taken with `JOY_RETURNALL` into the current `JOYINFOEX`, which is what
    /// 紅魔郷 asks for. Anything asking for more than that has to go to winmm.
    #[test]
    fn a_sample_answers_what_the_game_asks_and_no_more() {
        let mut asked: JOYINFOEX = unsafe { std::mem::zeroed() };
        asked.dwSize = size_of::<JOYINFOEX>() as u32;
        asked.dwFlags = RETURN_ALL;
        assert!(describes_a_sample(&asked));

        // `JOY_RETURNRAWDATA`, which a sample does not carry.
        asked.dwFlags = RETURN_ALL | 0x100;
        assert!(!describes_a_sample(&asked));

        // The struct before `JOYINFOEX` grew, which is not the one a sample fills.
        asked.dwFlags = RETURN_ALL;
        asked.dwSize -= 4;
        assert!(!describes_a_sample(&asked));
    }
}
