//! The write over the one of the exe's imports the game asks for the mouse pointer through.
//!
//! `ShowCursor`, and patching that entry catches every ask the game makes and nothing else's — orb's own
//! step of the display counter goes through the seam rather than through this table.
//!
//! Everything the rewrite decides is [`orb_core::mouse`]'s: whether the pointer is on the screen, how
//! long the mouse has to have been still for it to go, and why the game's ask is answered rather than
//! passed on. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use crate::hook;

/// # Safety
/// `module` must be the exe.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    // Nothing keeps what was in the entry. Every one of the game's own asks is answered rather than
    // passed on, so there is nothing for the rewrite to call through to — which is the whole of what
    // leaves the display counter orb's to step over the edge Windows draws the pointer by.
    unsafe {
        hook::install_import(
            module,
            "USER32.dll",
            "ShowCursor",
            hook::address(orb_core::mouse::show_cursor as _),
        )?
    };
    orb_core::mouse::install(true);
    Ok(())
}
