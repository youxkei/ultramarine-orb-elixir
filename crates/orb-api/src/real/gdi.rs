//! The GDI calls that carry orb's own text, taken out of gdi32's export table rather than called
//! through this DLL's import table.
//!
//! **Why not the imports, which is what every other call here uses.** A translation patch injected into
//! the same game rewrites the import table of every module in the process, orb's DLL included, and the
//! GDI text calls are among the entries it rewrites — that being how it puts a language pack's words
//! where the game's own were. orb's text then leaves through a replacement written for the game's `-A`
//! calls, which takes a `-W` call's count of characters for a count of bytes: a line cut short where the
//! count falls inside the string, and whatever is next in the process drawn as glyphs where it falls
//! outside. See
//! [docs/adr/0015](../../../docs/adr/0015-orbs-own-text-leaves-through-gdi32s-exports.md).
//!
//! **Read out of the export table and not asked of `GetProcAddress`**, which is itself one of the calls
//! such a patcher replaces — it hands out its own wrappers through it by design. The export directory is
//! what the loader itself reads and nothing rewrites it.
//!
//! Only the calls that carry a string or hand back a font are taken this way. The rest of the drawing —
//! the memory DC, the bitmap, the blit — carries no text for a count to be wrong about, and taking those
//! too would be a longer table for nothing.

use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{BOOL, RECT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    self, ETO_OPTIONS, FONT_RESOURCE_CHARACTERISTICS, HDC, HFONT, LOGFONTW,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

pub type ExtTextOutW = unsafe extern "system" fn(
    HDC,
    i32,
    i32,
    ETO_OPTIONS,
    *const RECT,
    *const u16,
    u32,
    *const i32,
) -> BOOL;
pub type TextOutW = unsafe extern "system" fn(HDC, i32, i32, *const u16, i32) -> BOOL;
pub type GetTextExtentPoint32W = unsafe extern "system" fn(HDC, *const u16, i32, *mut SIZE) -> BOOL;
pub type GetTextFaceW = unsafe extern "system" fn(HDC, i32, *mut u16) -> i32;
pub type CreateFontIndirectW = unsafe extern "system" fn(*const LOGFONTW) -> HFONT;
#[allow(clippy::type_complexity)]
pub type CreateFontW = unsafe extern "system" fn(
    i32,
    i32,
    i32,
    i32,
    i32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    *const u16,
) -> HFONT;
pub type AddFontResourceExW =
    unsafe extern "system" fn(*const u16, FONT_RESOURCE_CHARACTERISTICS, *const c_void) -> i32;

use std::ffi::c_void;

/// The calls, resolved once.
///
/// Each falls back to the one this DLL imports where the export cannot be found, so a host whose gdi32
/// this cannot be read out of draws its text the way orb drew it before — which is text on the screen
/// rather than none.
pub struct Text {
    pub ext_text_out: ExtTextOutW,
    pub text_out: TextOutW,
    pub text_extent: GetTextExtentPoint32W,
    pub text_face: GetTextFaceW,
    pub create_font_indirect: CreateFontIndirectW,
    pub create_font: CreateFontW,
    pub add_font_resource: AddFontResourceExW,
}

pub fn text() -> &'static Text {
    static TEXT: OnceLock<Text> = OnceLock::new();
    TEXT.get_or_init(|| {
        let gdi32 = module("gdi32.dll");
        // # Safety
        // Each name is resolved out of gdi32's own export directory, so the address is that export's;
        // every type above is the signature `windows-sys` declares for that same export.
        unsafe {
            Text {
                ext_text_out: found(
                    gdi32,
                    b"ExtTextOutW",
                    Gdi::ExtTextOutW as *const () as usize,
                ),
                text_out: found(gdi32, b"TextOutW", Gdi::TextOutW as *const () as usize),
                text_extent: found(
                    gdi32,
                    b"GetTextExtentPoint32W",
                    Gdi::GetTextExtentPoint32W as *const () as usize,
                ),
                text_face: found(
                    gdi32,
                    b"GetTextFaceW",
                    Gdi::GetTextFaceW as *const () as usize,
                ),
                create_font_indirect: found(
                    gdi32,
                    b"CreateFontIndirectW",
                    Gdi::CreateFontIndirectW as *const () as usize,
                ),
                create_font: found(
                    gdi32,
                    b"CreateFontW",
                    Gdi::CreateFontW as *const () as usize,
                ),
                add_font_resource: found(
                    gdi32,
                    b"AddFontResourceExW",
                    Gdi::AddFontResourceExW as *const () as usize,
                ),
            }
        }
    })
}

/// # Safety
/// `imported` must be the address of the very export `name` names, which is what the caller passes as
/// the fallback: the two are the same function and `F` is that function's signature.
unsafe fn found<F: Copy>(module: usize, name: &[u8], imported: usize) -> F {
    let address = if module == 0 {
        None
    } else {
        unsafe { export(module, name) }
    };
    let address = address.unwrap_or(imported);
    // # Safety
    // `F` is a function pointer type and `address` a function of that signature, so this is the cast
    // `GetProcAddress`'s answer needs at every call site — done once here.
    unsafe { *(&address as *const usize as *const F) }
}

fn module(name: &str) -> usize {
    let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
    unsafe { GetModuleHandleW(wide.as_ptr()) as usize }
}

/// The address a module exports under `name`, read out of its export directory.
///
/// Names only, and no forwarders followed: what is asked for here are gdi32's own functions, none of which
/// gdi32 forwards. A name that is not found — or one that is a forwarder, which reads as an address inside
/// the export directory itself — answers `None` and the caller keeps what it imported, which is the call
/// this module exists to go around. So a Windows that forwards one of them is caught by
/// `every_call_is_found_inside_gdi32` below rather than by somebody reading the screen.
///
/// # Safety
/// `module` must be the base address of a module mapped into this process.
unsafe fn export(module: usize, name: &[u8]) -> Option<usize> {
    unsafe {
        let directory = export_directory(module)?;
        let table = Exports::at(module, directory.clone())?;
        let names = std::slice::from_raw_parts(
            (module + table.names as usize) as *const u32,
            table.name_count as usize,
        );
        let ordinals = std::slice::from_raw_parts(
            (module + table.ordinals as usize) as *const u16,
            table.name_count as usize,
        );
        let functions = std::slice::from_raw_parts(
            (module + table.functions as usize) as *const u32,
            table.function_count as usize,
        );
        let at = names
            .iter()
            .position(|offset| exported_name(module + *offset as usize) == name)?;
        let function = *functions.get(*ordinals.get(at)? as usize)?;
        // A forwarder's slot points inside the export directory rather than at code. Nothing asked for
        // here is one, and answering `None` leaves the caller with the address it imported.
        if directory.contains(&function) {
            return None;
        }
        Some(module + function as usize)
    }
}

/// The `IMAGE_EXPORT_DIRECTORY` fields this reads, which is every field it needs and no more.
struct Exports {
    functions: u32,
    names: u32,
    ordinals: u32,
    function_count: u32,
    name_count: u32,
}

impl Exports {
    /// # Safety
    /// `module` must be a mapped module and `directory` its export directory's range of RVAs.
    unsafe fn at(module: usize, directory: std::ops::Range<u32>) -> Option<Self> {
        // The five fields at the offsets `IMAGE_EXPORT_DIRECTORY` puts them: the counts at 0x14 and
        // 0x18, then the three tables. Read as words out of the mapped image rather than through a
        // `windows-sys` struct, which does not carry this one.
        let base = module + directory.start as usize;
        let word = |offset: usize| unsafe { *((base + offset) as *const u32) };
        Some(Self {
            function_count: word(0x14),
            name_count: word(0x18),
            functions: word(0x1c),
            names: word(0x20),
            ordinals: word(0x24),
        })
    }
}

/// The bytes of a name in the export table, up to its terminator.
///
/// # Safety
/// `at` must be inside a mapped module, at a NUL-terminated name.
unsafe fn exported_name(at: usize) -> &'static [u8] {
    unsafe {
        let mut end = at;
        // Bounded by nothing but the terminator, which is what an export name has: the table this walks
        // is the loader's own and its names are strings.
        while *(end as *const u8) != 0 {
            end += 1;
        }
        std::slice::from_raw_parts(at as *const u8, end - at)
    }
}

/// Where a module's exports are, as a range of RVAs — the first entry of the data directory.
///
/// # Safety
/// `module` must be the base address of a module mapped into this process.
unsafe fn export_directory(module: usize) -> Option<std::ops::Range<u32>> {
    unsafe {
        // `IMAGE_DOS_HEADER::e_lfanew` at 0x3c, and the PE signature it points at.
        if *(module as *const u16) != 0x5a4d {
            return None;
        }
        let nt = module + *((module + 0x3c) as *const u32) as usize;
        if *(nt as *const u32) != 0x0000_4550 {
            return None;
        }
        // The optional header follows the four-byte signature and the 20-byte file header; its data
        // directory begins at 0x60 in the 32-bit one, which is the only one this process has.
        let address = *((nt + 0x18 + 0x60) as *const u32);
        let size = *((nt + 0x18 + 0x64) as *const u32);
        (address != 0 && size != 0).then(|| address..address + size)
    }
}

#[cfg(test)]
mod tests {
    use super::{export, module, text};

    /// Every call is found, and found inside gdi32 — which is the whole of what the walk above has to get
    /// right. An offset wrong by a field answers some other export, or a name table read as an ordinal
    /// table answers an address that is not a function at all, and either one is a call into whatever
    /// happens to be there.
    ///
    /// Held against gdi32's own image rather than against the addresses this DLL imports, which are not
    /// the same numbers: an import is a stub that jumps through the table, so the two are equal in no
    /// build.
    #[test]
    fn every_call_is_found_inside_gdi32() {
        let gdi32 = module("gdi32.dll");
        assert_ne!(gdi32, 0, "gdi32 is not loaded in this process");
        let image = unsafe { image_range(gdi32) }.expect("gdi32 has PE headers");

        let mut found = Vec::new();
        for name in [
            b"ExtTextOutW".as_slice(),
            b"TextOutW",
            b"GetTextExtentPoint32W",
            b"GetTextFaceW",
            b"CreateFontIndirectW",
            b"CreateFontW",
            b"AddFontResourceExW",
        ] {
            let at = unsafe { export(gdi32, name) }
                .unwrap_or_else(|| panic!("gdi32 exports no {}", String::from_utf8_lossy(name)));
            assert!(
                image.contains(&at),
                "{} came out at {at:#x}, outside gdi32's {image:#x?}",
                String::from_utf8_lossy(name),
            );
            found.push(at);
        }
        // And each name answered a different address: one table read for another would answer the same
        // function over and over, which lands inside gdi32 too.
        let mut sorted = found.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), found.len(), "two names answered one address");
    }

    /// And what they answer is what the imported call answers, asked of the one call here whose answer is
    /// a number: the same string measured through both comes out the same size.
    ///
    /// Which is what says the address is the *right* export rather than merely one of gdi32's.
    #[test]
    fn a_measurement_through_the_export_is_the_measurement_through_the_import() {
        use windows_sys::Win32::Foundation::SIZE;
        use windows_sys::Win32::Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, GetTextExtentPoint32W,
        };

        let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        assert!(!dc.is_null(), "no device context to measure against");
        let wide: Vec<u16> = "INPUT LAG 3.0ms".encode_utf16().collect();
        let mut theirs = SIZE { cx: 0, cy: 0 };
        let mut ours = SIZE { cx: 0, cy: 0 };
        let measured = unsafe {
            (
                GetTextExtentPoint32W(dc, wide.as_ptr(), wide.len() as i32, &mut theirs),
                (text().text_extent)(dc, wide.as_ptr(), wide.len() as i32, &mut ours),
            )
        };
        unsafe { DeleteDC(dc) };

        assert_eq!(measured.0 != 0, measured.1 != 0, "one of the two refused");
        assert_eq!((theirs.cx, theirs.cy), (ours.cx, ours.cy));
        // And a string of that length measures to something, or the two agreeing says nothing.
        assert!(theirs.cx > 0 && theirs.cy > 0);
    }

    /// Where a module is mapped, as the range its own headers say.
    ///
    /// # Safety
    /// `module` must be the base address of a module mapped into this process.
    unsafe fn image_range(module: usize) -> Option<std::ops::Range<usize>> {
        unsafe {
            if *(module as *const u16) != 0x5a4d {
                return None;
            }
            let nt = module + *((module + 0x3c) as *const u32) as usize;
            if *(nt as *const u32) != 0x0000_4550 {
                return None;
            }
            // `SizeOfImage`, which in the 32-bit optional header follows the signature and the file
            // header at 0x38.
            let size = *((nt + 0x18 + 0x38) as *const u32) as usize;
            (size != 0).then(|| module..module + size)
        }
    }
}
