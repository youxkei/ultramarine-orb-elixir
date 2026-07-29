//! Config key names to Win32 virtual-key codes.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtualKey(pub u8);

pub const A: VirtualKey = VirtualKey(0x41);
pub const C: VirtualKey = VirtualKey(0x43);
pub const D: VirtualKey = VirtualKey(0x44);
pub const S: VirtualKey = VirtualKey(0x53);
pub const V: VirtualKey = VirtualKey(0x56);
pub const SHIFT: VirtualKey = VirtualKey(0x10);
pub const CTRL: VirtualKey = VirtualKey(0x11);
pub const ALT: VirtualKey = VirtualKey(0x12);

const NAMED: &[(&str, u8)] = &[
    ("shift", 0x10),
    ("ctrl", 0x11),
    ("control", 0x11),
    ("alt", 0x12),
    ("esc", 0x1b),
    ("escape", 0x1b),
    ("space", 0x20),
    ("enter", 0x0d),
    ("return", 0x0d),
    ("tab", 0x09),
    ("backspace", 0x08),
    ("left", 0x25),
    ("up", 0x26),
    ("right", 0x27),
    ("down", 0x28),
];

pub fn parse(name: &str) -> Option<VirtualKey> {
    let name = name.trim().to_ascii_lowercase();
    if let Some(&(_, code)) = NAMED.iter().find(|(known, _)| *known == name) {
        return Some(VirtualKey(code));
    }
    if let Some(index) = name.strip_prefix('f').and_then(|n| n.parse::<u8>().ok())
        && (1..=12).contains(&index)
    {
        return Some(VirtualKey(0x70 + index - 1));
    }
    match name.as_bytes() {
        [single @ (b'a'..=b'z' | b'0'..=b'9')] => Some(VirtualKey(single.to_ascii_uppercase())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CTRL, VirtualKey, parse};

    #[test]
    fn accepts_names_letters_digits_and_function_keys() {
        assert_eq!(parse("Ctrl"), Some(CTRL));
        assert_eq!(parse("z"), Some(VirtualKey(0x5a)));
        assert_eq!(parse("5"), Some(VirtualKey(0x35)));
        assert_eq!(parse("f1"), Some(VirtualKey(0x70)));
        assert_eq!(parse("f12"), Some(VirtualKey(0x7b)));
    }

    #[test]
    fn rejects_unknown_names() {
        assert_eq!(parse("f13"), None);
        assert_eq!(parse("f0"), None);
        assert_eq!(parse("hyper"), None);
        assert_eq!(parse(""), None);
    }
}
