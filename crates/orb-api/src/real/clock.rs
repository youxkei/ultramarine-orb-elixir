//! The real host's clock.

use windows_sys::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::System::Threading::Sleep;

pub fn counter() -> i64 {
    let mut counter = 0;
    unsafe { QueryPerformanceCounter(&mut counter) };
    counter
}

pub fn frequency() -> i64 {
    let mut frequency = 0;
    unsafe { QueryPerformanceFrequency(&mut frequency) };
    frequency
}

pub fn ticks() -> u32 {
    unsafe { GetTickCount() }
}

pub fn sleep_millis(millis: u32) {
    unsafe { Sleep(millis) };
}

pub fn begin_period(millis: u32) -> u32 {
    unsafe { timeBeginPeriod(millis) }
}

pub fn end_period(millis: u32) {
    unsafe { timeEndPeriod(millis) };
}
