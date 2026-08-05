//! Which real thread is running.

use windows_sys::Win32::System::Threading::GetCurrentThreadId;

pub fn current_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}
