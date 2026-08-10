//! The files orb reads and writes of its own, kept rather than written.
//!
//! The same trade as [`crate::log`], and for the stronger reason: a test that put these on a real disk
//! would need a temporary directory to be emptied before it and would leave one behind whenever it failed,
//! and it could not ask the one question that matters most about a write — *what happens when it fails* —
//! except by arranging the disk into a shape that makes `std::fs` refuse. That is a test asserting about
//! the machine it happens to be on. Here a refusal is declared.
//!
//! **A flat map from path to bytes, with no directories in it except the ones that were made.** What orb
//! does with directories is exactly two things — make the one its files go in, and list it — so those are
//! what a directory is here: a set of paths that have been made, and a prefix match for the listing. There
//! are no permissions, no metadata and no symlinks, because nothing above the seam reads any.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What a path answers instead of the bytes it holds, where a test has declared one.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Refuses {
    Read,
    Write,
    Make,
}

#[derive(Default)]
struct State {
    files: HashMap<PathBuf, Vec<u8>>,
    directories: HashSet<PathBuf>,
    refused: Vec<(PathBuf, Refuses)>,
}

#[derive(Default)]
pub struct Files {
    state: Mutex<State>,
}

impl Files {
    /// Puts a file there, as an earlier session or an installer would have left it.
    ///
    /// What a test says it finds, which is the other half of what it later reads back: every directory above
    /// it counts as made, since a file cannot be somewhere that is not.
    pub fn put(&self, path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
        let path = path.as_ref().to_path_buf();
        let mut state = self.state.lock().unwrap();
        for parent in ancestors(&path) {
            state.directories.insert(parent);
        }
        state.files.insert(path, bytes.as_ref().to_vec());
    }

    /// Says a directory is there, with nothing in it.
    ///
    /// What every e2e test declares of the one the game is installed in, because orb writes straight into
    /// it — `chapters.rs` and `tuning.txt` — and a write wants its directory to be there, here as on a real
    /// host. The runs left unfinished need no such declaration: `resume::write` makes that directory itself,
    /// which is the behaviour worth keeping honest.
    pub fn make(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        let mut state = self.state.lock().unwrap();
        for parent in ancestors(&path) {
            state.directories.insert(parent);
        }
        state.directories.insert(path);
    }

    /// What a path holds now, or `None` for one nothing has written.
    ///
    /// Which is how a test reads back what orb wrote: the bytes are the bytes, so a file written and read
    /// here is the same comparison it would be on a disk and none of the waiting.
    pub fn get(&self, path: impl AsRef<Path>) -> Option<Vec<u8>> {
        self.state.lock().unwrap().files.get(path.as_ref()).cloned()
    }

    /// And as text, for the two files somebody edits by hand.
    ///
    /// # Panics
    /// On bytes that are not UTF-8, which for a file orb wrote as text is orb having written something else.
    pub fn text(&self, path: impl AsRef<Path>) -> Option<String> {
        let path = path.as_ref();
        let bytes = self.get(path)?;
        Some(
            String::from_utf8(bytes)
                .unwrap_or_else(|error| panic!("{} is not text: {error}", path.display())),
        )
    }

    /// Whether a path is there at all, which is what a test asks of one orb was to have removed.
    pub fn holds(&self, path: impl AsRef<Path>) -> bool {
        self.state.lock().unwrap().files.contains_key(path.as_ref())
    }

    /// Every path there is, for a test saying nothing was left behind.
    pub fn paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.state.lock().unwrap().files.keys().cloned().collect();
        paths.sort();
        paths
    }

    /// Says that reading this path fails, where it would otherwise answer.
    ///
    /// A test saying so, the way it says the host refuses `SetProcessDPIAware`. **The one thing a real
    /// filesystem cannot be asked**: the only way to make `std::fs::read` fail on demand is to put
    /// something other than a readable file at the path, which is a test that has arranged the machine
    /// rather than declared the case.
    pub fn refuses_to_read(&self, path: impl AsRef<Path>) {
        self.refuse(path, Refuses::Read);
    }

    /// And that writing it fails, which is a disk that is full or a file somebody else holds open.
    pub fn refuses_to_write(&self, path: impl AsRef<Path>) {
        self.refuse(path, Refuses::Write);
    }

    /// And that the directory cannot be made, which is what a path with a file already at it does.
    pub fn refuses_to_make(&self, path: impl AsRef<Path>) {
        self.refuse(path, Refuses::Make);
    }

    fn refuse(&self, path: impl AsRef<Path>, how: Refuses) {
        self.state
            .lock()
            .unwrap()
            .refused
            .push((path.as_ref().to_path_buf(), how));
    }

    // --- what the seam asks -------------------------------------------------------

    pub fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let state = self.state.lock().unwrap();
        if refuses(&state, path, Refuses::Read) {
            return Err(refusal(path));
        }
        match state.files.get(path) {
            Some(bytes) => Ok(bytes.clone()),
            // The error a caller branches on: `resume::load` keeps "there is none" apart from "it could
            // not be read", and a store that answered one for the other would take that apart.
            None if state.directories.contains(path) => Err(io::Error::new(
                io::ErrorKind::IsADirectory,
                format!("{} is a directory", path.display()),
            )),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not there", path.display()),
            )),
        }
    }

    pub fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} is not UTF-8: {error}", path.display()),
            )
        })
    }

    pub fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        if refuses(&state, path, Refuses::Write) {
            return Err(refusal(path));
        }
        // The directory has to be there, which is what `std::fs::write` asks too: a run that wrote into
        // one it had not made would be a run whose `create_dir_all` nothing needed.
        let parent = path.parent().map(Path::to_path_buf);
        if let Some(parent) = parent
            && !parent.as_os_str().is_empty()
            && !state.directories.contains(&parent)
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not there", parent.display()),
            ));
        }
        state.files.insert(path.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    pub fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        if refuses(&state, path, Refuses::Make) {
            return Err(refusal(path));
        }
        // A file already at that path, which is the ordinary way this fails.
        if state.files.contains_key(path) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} is a file", path.display()),
            ));
        }
        state.directories.insert(path.to_path_buf());
        for parent in ancestors(path) {
            state.directories.insert(parent);
        }
        Ok(())
    }

    pub fn remove_file(&self, path: &Path) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        match state.files.remove(path) {
            Some(_) => Ok(()),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not there", path.display()),
            )),
        }
    }

    pub fn files_in(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let state = self.state.lock().unwrap();
        if !state.directories.contains(path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not there", path.display()),
            ));
        }
        // Directly inside, which is what the parent being this directory says: a store with no directory
        // objects in it has nothing else to go on, and nothing above the seam asks for a walk.
        Ok(state
            .files
            .keys()
            .filter(|file| file.parent() == Some(path))
            .cloned()
            .collect())
    }
}

/// Every directory above a path, which is what having it there implies.
fn ancestors(path: &Path) -> Vec<PathBuf> {
    path.ancestors()
        .skip(1)
        .filter(|at| !at.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .collect()
}

fn refuses(state: &State, path: &Path, how: Refuses) -> bool {
    state
        .refused
        .iter()
        .any(|(at, refused)| at == path && *refused == how)
}

/// What a declared refusal answers with.
///
/// `PermissionDenied`, which is what the two real cases look like — a file somebody else holds open, a
/// directory that cannot be written — and the kind is nothing any caller branches on: what they all do with
/// the error is put it in a line of the log.
fn refusal(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("{} was declared to refuse", path.display()),
    )
}
