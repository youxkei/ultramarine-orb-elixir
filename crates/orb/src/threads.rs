//! Noticing the threads the game creates.
//!
//! An import hook on `CreateThread`, because there is no other way to know which of the process's
//! threads are the game's — and only those may be stopped while its memory is copied. Suspending
//! everything, which is what enumerating the process's threads amounts to, also stops DirectSound's
//! mixer and the graphics driver's workers, and stopping those for the length of a copy is audible as
//! the sound breaking up.
//!
//! What is done with one afterwards is `orb_api::thread`: the table of them and the suspending live
//! behind the seam, because a thread handle is a Windows thing and nothing above the seam should be
//! holding one. Here is only the noticing, which is a hook and so orb's.

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{GetThreadId, THREAD_CREATION_FLAGS};

use crate::hook;
use crate::log;

static CREATE_THREAD: AtomicUsize = AtomicUsize::new(0);

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
            hook::address(create_thread as _),
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
        let id = unsafe { GetThreadId(thread) };
        if id != 0 && !orb_api::thread::registered(id) {
            log!("threads: cannot open the game's thread {id}");
        }
    }
    thread
}
