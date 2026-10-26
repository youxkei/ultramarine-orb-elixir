//! A short tone orb makes for itself, apart from the game's own sound.
//!
//! **Orb's own and not the game's**, because what this reports is a decision orb made about the run
//! rather than anything the game did: the game is streaming its music and its effects through
//! DirectSound in a buffer orb does not own, and the one sound the game could be asked for by name
//! is one of its own, whose ids and queue are a reading per game. A tone of orb's own is one thing
//! for every game, and it is heard over the game's music because it goes through winmm rather than
//! through the game's buffer.
//!
//! **A wave of orb's own rather than a system sound.** `MessageBeep` is the nearest call there is,
//! and what it plays is the desktop's notification sound, which a machine is free to have set to
//! none — and then the one moment this exists for is silent with nothing to say why. A wave orb
//! carries is the same on every machine.
//!
//! It is started and left to play: the caller is a frame of the game's, and waiting for a sound
//! would be waiting out a frame's deadline on the host's mixer.

/// The short rise that says a judgement of orb's went the run's way.
pub fn a_tone() {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.a_tone();
    }
    host::a_tone()
}

#[cfg(windows)]
use crate::real::sound as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn a_tone() {
        no_windows("sound::a_tone")
    }
}
