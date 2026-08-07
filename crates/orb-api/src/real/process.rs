//! Ending the real process.

use windows_sys::Win32::System::Threading::ExitProcess;

pub fn exit(code: u32) {
    unsafe { ExitProcess(code) }
}
