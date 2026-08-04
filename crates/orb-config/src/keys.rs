//! The Win32 virtual-key codes orb names.
//!
//! Codes and not names read out of `orb.yaml`: the keys are fixed in the code — see `keys` in
//! the DLL, which is the only thing that binds them — so there is nothing to turn a name into
//! one. What is left is a number wanting a name, which is what these are.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtualKey(pub u8);

pub const A: VirtualKey = VirtualKey(0x41);
pub const D: VirtualKey = VirtualKey(0x44);
pub const SHIFT: VirtualKey = VirtualKey(0x10);
pub const CTRL: VirtualKey = VirtualKey(0x11);
pub const SPACE: VirtualKey = VirtualKey(0x20);
pub const LEFT: VirtualKey = VirtualKey(0x25);
pub const UP: VirtualKey = VirtualKey(0x26);
pub const RIGHT: VirtualKey = VirtualKey(0x27);
pub const DOWN: VirtualKey = VirtualKey(0x28);
