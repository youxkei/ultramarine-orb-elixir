//! The real machine's display language.

use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
use windows_sys::Win32::System::SystemServices::LANG_JAPANESE;

/// The one number above the seam that Windows has a name for, held against it here.
const _: () = assert!(crate::locale::JAPANESE as u32 == LANG_JAPANESE);

/// Nothing to fail: the call answers the language Windows is installed in where a user has chosen
/// none, so there is no failure for it to report and none to answer here.
pub fn ui_language() -> u16 {
    unsafe { GetUserDefaultUILanguage() }
}
