//! The write over the one of the exe's imports the game asks for its *other* functions through.
//!
//! `GetProcAddress`, and patching that entry catches every name the game looks up — which for a game
//! built this way is almost everything it calls: th06c imports eighteen names from `USER32.dll` and asks
//! for three hundred and forty-two more by string, its window, its renderer and its keyboard read among
//! them. See `orb-core`'s `game/th06c/asm.md`.
//!
//! Everything the rewrite decides is [`orb_core::runtime::get_proc_address`]'s: which name is answered
//! with orb's own, what the host answered, and that every other name is handed back untouched.

use std::sync::atomic::Ordering;

use orb_core::runtime::{GET_PROC_ADDRESS, Lookups, answers_the_lookup_of, get_proc_address};

use crate::hook;

/// **One installation for every name orb answers**, which is what the entry is: there is one
/// `GetProcAddress` in the import table whatever is being answered through it, so a second write over it
/// would be orb calling through to itself.
///
/// # Safety
/// `module` must be the exe, and this must run before the game's own startup has looked anything up.
pub unsafe fn install(module: usize, names: Lookups) -> Result<(), hook::Error> {
    // The names first, so that a lookup answered between the two writes is one orb already knows what to
    // do about: the entry is what puts the rewrite in the path, and there is nothing to be gained from
    // being in it a moment before knowing which names it is for.
    unsafe { answers_the_lookup_of(names) };
    match unsafe {
        hook::install_import(
            module,
            "KERNEL32.dll",
            "GetProcAddress",
            hook::address(get_proc_address as _),
        )
    } {
        Ok(previous) => {
            GET_PROC_ADDRESS.store(previous, Ordering::Relaxed);
            Ok(())
        }
        Err(error) => {
            // Taken back, so that a launch whose entry was not there has orb answering no lookup at all
            // rather than holding names nothing will ever ask it about.
            unsafe { answers_the_lookup_of(Lookups::default()) };
            Err(error)
        }
    }
}
