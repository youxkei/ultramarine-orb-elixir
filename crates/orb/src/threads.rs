//! Holding the game still while its memory is copied.
//!
//! Only the threads the game created itself can touch the memory a snapshot
//! covers, so only those are stopped. Suspending everything — which is what
//! enumerating the process's threads amounts to — also stops DirectSound's mixer
//! and the graphics driver's workers, and stopping those for the length of a copy
//! is audible as the sound breaking up.
//!
//! The game's audio thread is left running as well, even though it is the game's:
//! it is the one thread whose pauses are heard, and the only game memory it
//! touches is the streaming bookkeeping, which the snapshot checks separately.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::System::Threading::{
    GetThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_CREATION_FLAGS,
    THREAD_SUSPEND_RESUME,
};

use crate::hook;
use crate::log::log;

/// The threads the game has created, with a handle each so that suspending one
/// needs no lookup. A thread that has exited fails to suspend and is dropped.
static GAME_THREADS: Mutex<Vec<Thread>> = Mutex::new(Vec::new());

static CREATE_THREAD: AtomicUsize = AtomicUsize::new(0);

struct Thread {
    id: u32,
    handle: isize,
}

#[allow(clippy::type_complexity)]
type CreateThread = unsafe extern "system" fn(
    *const c_void,
    usize,
    Option<unsafe extern "system" fn(*mut c_void) -> u32>,
    *const c_void,
    THREAD_CREATION_FLAGS,
    *mut u32,
) -> HANDLE;

/// # Safety
/// Must run before the game creates any thread, and `module` must be the exe.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "KERNEL32.dll",
            "CreateThread",
            create_thread as usize,
        )
    }?;
    CREATE_THREAD.store(previous, Ordering::Relaxed);
    Ok(())
}

unsafe extern "system" fn create_thread(
    attributes: *const c_void,
    stack_size: usize,
    start: Option<unsafe extern "system" fn(*mut c_void) -> u32>,
    parameter: *const c_void,
    flags: THREAD_CREATION_FLAGS,
    id_out: *mut u32,
) -> HANDLE {
    let original: CreateThread =
        unsafe { std::mem::transmute(CREATE_THREAD.load(Ordering::Relaxed)) };
    let thread = unsafe { original(attributes, stack_size, start, parameter, flags, id_out) };
    if !thread.is_null() {
        remember(unsafe { GetThreadId(thread) });
    }
    thread
}

fn remember(id: u32) {
    if id == 0 {
        return;
    }
    // A handle of our own, so suspending never depends on one the game may close.
    let handle = unsafe { OpenThread(THREAD_SUSPEND_RESUME, FALSE, id) };
    if handle.is_null() {
        return log!("threads: cannot open the game's thread {id}");
    }
    if let Ok(mut threads) = GAME_THREADS.lock() {
        threads.push(Thread {
            id,
            handle: handle as isize,
        });
    }
}

/// The game's threads, stopped for as long as this lives.
pub struct Suspended(Vec<HANDLE>);

impl Suspended {
    /// Stops every thread the game made except the caller's and `audio`.
    pub fn all_but(current: u32, audio: Option<u32>) -> Self {
        let Ok(mut threads) = GAME_THREADS.lock() else {
            return Self(Vec::new());
        };
        let mut suspended = Vec::with_capacity(threads.len());
        threads.retain(|thread| {
            if thread.id == current || Some(thread.id) == audio {
                return true;
            }
            let handle = thread.handle as HANDLE;
            if unsafe { SuspendThread(handle) } == u32::MAX {
                // Already gone: nothing to resume, and nothing to keep.
                unsafe { CloseHandle(handle) };
                return false;
            }
            suspended.push(handle);
            true
        });
        Self(suspended)
    }
}

impl Drop for Suspended {
    fn drop(&mut self) {
        for &handle in &self.0 {
            unsafe { ResumeThread(handle) };
        }
    }
}
