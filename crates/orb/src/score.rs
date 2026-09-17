//! The write over the exe's import of `CreateFileA`.
//!
//! Done at the import rather than at the game's own score code, which is the seam `memtrack` uses and for
//! the same reason: the game's statically linked CRT reaches the OS through it. Both of th06's own paths
//! to the file end there — `orb_core::score`'s doc has the four places it names it and the call graph down
//! to the import. So nothing about this patch is per-game: no address, no offset, and nothing about the file's
//! format or the encryption over it. d3d8's and dsound's own opens go through their own imports and are
//! not in the path.
//!
//! Only the open is redirected. A game that truncated a file by deleting it first — which is
//! what one of th06's two file paths does, and not the one the score file takes — would have
//! its own file deleted while orb's was written, and would need `DeleteFileA` redirected
//! alongside this.
//!
//! Which file the open lands in, and the walk of the path the game handed over, are
//! [`orb_core::score`]'s.

use orb_core::game::FileOpen;
use orb_core::score::{CreateFileA, Fopen, create_file_a, fopen, install_over, install_over_fopen};

use crate::hook;

/// # Safety
/// Must run before the game's entry point, or the game's first open of its score file is
/// the game's own. Patches `module`'s imports, so nothing else may be executing it.
pub unsafe fn install(module: usize, opens: FileOpen) -> Result<(), hook::Error> {
    match opens {
        FileOpen::CreateFileA => {
            let previous = unsafe {
                hook::install_import(
                    module,
                    "KERNEL32.dll",
                    "CreateFileA",
                    hook::address(create_file_a as _),
                )?
            };
            unsafe { install_over(std::mem::transmute::<usize, CreateFileA>(previous)) };
        }
        // **The universal CRT's `fopen`, for a game whose every open is that** — see
        // `orb_core::game::FileOpen::Fopen`. The `CreateFileW` under it is `ucrtbase.dll`'s own import
        // and out of reach; the exe's entry is the one the game calls.
        FileOpen::Fopen => {
            let previous = unsafe {
                hook::install_import(
                    module,
                    "api-ms-win-crt-stdio-l1-1-0.dll",
                    "fopen",
                    hook::address(fopen as _),
                )?
            };
            unsafe { install_over_fopen(std::mem::transmute::<usize, Fopen>(previous)) };
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// The refusal `orb_core::score::create_file_a` answers a write with, held against Windows' own.
    ///
    /// A test rather than a `const` assert beside the number, which is what the two window styles and the
    /// two text alignments get: a raw pointer can be neither cast to an integer nor compared during const
    /// evaluation, so this is the one Windows number in the tree the compiler cannot be made to check.
    #[test]
    fn the_refused_handle_is_windows_own() {
        assert_eq!(
            orb_core::score::INVALID_HANDLE as *mut std::ffi::c_void,
            windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE,
        );
    }
}
