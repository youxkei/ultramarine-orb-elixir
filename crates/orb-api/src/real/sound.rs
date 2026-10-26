//! The tone through winmm's `PlaySound`, out of a wave orb makes for itself.
//!
//! **Made here and not carried in the tree**, which is the trade this file is: the wave is two
//! notes of a few hundred milliseconds, and a binary beside the code would be a file nothing in
//! the source says the shape of. What the tone is, then, is the arithmetic below — and a note an e2e
//! test hears is that same arithmetic in `orb-sim`, which does not play it.
//!
//! **Asynchronous**, because the caller is a frame of the game's: `PlaySound` with `SND_ASYNC`
//! starts the mixer and returns, and the buffer it reads from has to outlive the call — which is
//! what the `OnceLock` is for, a wave made once and kept for the process rather than a `Vec` whose
//! drop would take the samples away under the mixer.

use std::sync::OnceLock;

use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};

/// The samples a second every wave here is made at, and the one WAV's header is written with.
const RATE: u32 = 44_100;

/// How loud a note is at its first frame, as a part of full scale: loud enough to be heard over the
/// game's music, quiet enough not to be a noise somebody flinches at.
const AMPLITUDE: f32 = 0.22;

/// The notes, as `(frequency, seconds)` — a rising fifth, which is the shortest thing that sounds
/// finished rather than cut off.
const NOTES: [(f32, f32); 2] = [(880.0, 0.07), (1174.66, 0.11)];

/// And how long a note takes to come up, which is what keeps its first sample from being a click.
const ATTACK: f32 = 0.004;

pub fn a_tone() {
    let wave = wave();
    unsafe {
        PlaySoundW(
            wave.as_ptr().cast(),
            std::ptr::null_mut(),
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        )
    };
}

/// The wave, made once and kept for the process — see the module's note on the mixer reading it.
fn wave() -> &'static [u8] {
    static WAVE: OnceLock<Vec<u8>> = OnceLock::new();
    WAVE.get_or_init(|| {
        let mut samples: Vec<i16> = Vec::new();
        for (frequency, seconds) in NOTES {
            let frames = (RATE as f32 * seconds) as u32;
            let attack = ATTACK * RATE as f32;
            for at in 0..frames {
                let time = at as f32 / RATE as f32;
                // Up over the attack and down to nothing across the note, so neither end is a step.
                let envelope = (at as f32 / attack).min(1.0) * (1.0 - at as f32 / frames as f32);
                let sample =
                    (time * frequency * std::f32::consts::TAU).sin() * envelope * AMPLITUDE;
                samples.push((sample * f32::from(i16::MAX)) as i16);
            }
        }
        let data: Vec<u8> = samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect();
        let mut wave = Vec::with_capacity(44 + data.len());
        wave.extend_from_slice(b"RIFF");
        wave.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        wave.extend_from_slice(b"WAVEfmt ");
        wave.extend_from_slice(&16u32.to_le_bytes());
        wave.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wave.extend_from_slice(&1u16.to_le_bytes()); // one channel
        wave.extend_from_slice(&RATE.to_le_bytes());
        wave.extend_from_slice(&(RATE * 2).to_le_bytes()); // bytes a second
        wave.extend_from_slice(&2u16.to_le_bytes()); // bytes a frame
        wave.extend_from_slice(&16u16.to_le_bytes()); // bits a sample
        wave.extend_from_slice(b"data");
        wave.extend_from_slice(&(data.len() as u32).to_le_bytes());
        wave.extend_from_slice(&data);
        wave
    })
}
