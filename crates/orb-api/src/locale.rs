//! The language the machine puts its own windows up in, which is the one orb writes its screens in
//! where `orb.yaml` does not say.
//!
//! **The display language and not the user locale.** `GetUserDefaultLocaleName` answers the regional
//! formats — which decimal separator, which date order — and those are set apart from the language
//! Windows itself is in: a machine whose windows are English with Japanese formats behind them is a
//! machine somebody reads English on. What this decides is words, so what it asks about is the words
//! Windows is already using.
//!
//! **A module of its own beside [`crate::codepage`]** rather than a second function in it: a code page
//! is which bytes a `-A` call answered in, and this is which language a person reads. Windows keeps
//! the two apart and orb reads them for different things — the code page for a pad's name, this for
//! every screen orb puts up.

/// `LANG_JAPANESE`, held against Windows' own by a `const` assert below the seam — the way [`Bar`]'s
/// two alignments are, and for the same reason: which language a LANGID is is part of deciding which
/// words orb writes, and that decision is `orb_config::Language`'s, above here.
///
/// [`Bar`]: crate::Bar
pub const JAPANESE: u16 = 0x11;

/// What takes a LANGID down to the language in it, without the region: Windows' `PRIMARYLANGID`.
///
/// A macro over there and so the one number here that no assert can hold against its own. Japanese as
/// spoken in Japan is the only Japanese Windows has a sublanguage for, so what this drops for the
/// comparison [`JAPANESE`] is in is every *other* language's region.
pub const PRIMARY_LANGUAGE: u16 = 0x3ff;

/// `GetUserDefaultUILanguage` — the LANGID of the language Windows shows its own windows in.
///
/// The whole LANGID rather than the primary language alone: cutting it down is a read of the number
/// rather than the asking of it, and both halves of that read are above the seam.
pub fn ui_language() -> u16 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.ui_language();
    }
    host::ui_language()
}

#[cfg(windows)]
use crate::real::locale as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn ui_language() -> u16 {
        no_windows("locale::ui_language")
    }
}
