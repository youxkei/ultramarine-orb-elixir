//! The files orb reads and writes of its own: `orb.yaml`, the run left unfinished, and the two a
//! tuning pass produces.
//!
//! Not the game's own files. The score file is the game's `CreateFileA` with orb's hook over it, and the
//! log is [`crate::logfile`] because its first lines go out of `DllMain` with the loader lock held. What
//! is here is the rest — every path orb decides on its own account.
//!
//! **Behind the seam so that no test touches a real filesystem.** A test that writes into a temporary
//! directory is a test with a `remove_dir_all` between it and the next run, litter left behind whenever it
//! fails, and no way at all to say *the write failed* — the only way to ask that question of `std::fs` is
//! to arrange the disk into a shape that makes it fail, which is a test asserting about the machine it
//! happens to be on. A simulated Windows keeps the bytes instead and refuses on demand, which is what
//! every other host call here already does. See
//! [docs/adr/0012](../../../docs/adr/0012-orb-reads-and-writes-its-own-files-through-the-seam.md).
//!
//! **`std::io::Result` across the seam**, which is neither a `windows-sys` type nor a new one: what every
//! call site does with the error is put it in a line of the log, so an error that could not say what went
//! wrong would be a seam that costs the log its diagnosis.

use std::io;
use std::path::{Path, PathBuf};

/// The whole of a file.
pub fn read(path: &Path) -> io::Result<Vec<u8>> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.read_file(path);
    }
    host::read(path)
}

/// And as text, which is what the two files somebody edits by hand are read as.
///
/// Its own call rather than [`read`] and a decode at the call site, because what a decode that failed has
/// to answer with is the same `io::Error` the read would have — `InvalidData` — and writing that
/// conversion at each of the two call sites is writing it twice.
pub fn read_to_string(path: &Path) -> io::Result<String> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.read_file_to_string(path);
    }
    host::read_to_string(path)
}

/// Writes a file, replacing whatever was there.
pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.write_file(path, bytes);
    }
    host::write(path, bytes)
}

/// Makes a directory and every directory above it, and answers `Ok` for one that is already there.
pub fn create_dir_all(path: &Path) -> io::Result<()> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.create_dir_all(path);
    }
    host::create_dir_all(path)
}

pub fn remove_file(path: &Path) -> io::Result<()> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.remove_file(path);
    }
    host::remove_file(path)
}

/// The files directly inside a directory, as their whole paths, in no order the caller may rely on.
///
/// Files and not entries: a `DirEntry` is a handle with a lifetime and a metadata call behind it, and what
/// the one caller of this wants is the names in a directory it knows holds nothing else. A directory that
/// is not there is an error like any other — the caller has a run to carry on with either way.
pub fn files_in(path: &Path) -> io::Result<Vec<PathBuf>> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.files_in(path);
    }
    host::files_in(path)
}

/// The host's own answer, which is `std::fs` on **every** host.
///
/// So there is no `real/fs.rs` and no `no_windows` arm, where every other module here has both: a file is
/// the one thing this seam carries that Windows is not the only one of. What that buys is the crates over
/// the seam keeping their own `#[cfg(test)]` tests on a host that has no Windows — `orb-config`'s read and
/// write of `orb.yaml` among them — and it costs nothing, since the DLL installs no simulated Windows and
/// reaches these directly.
mod host {
    use std::io;
    use std::path::{Path, PathBuf};

    pub fn read(path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    pub fn read_to_string(path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
        std::fs::write(path, bytes)
    }

    pub fn create_dir_all(path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    pub fn remove_file(path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    pub fn files_in(path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                files.push(entry.path());
            }
        }
        Ok(files)
    }
}
