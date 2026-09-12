//! What a fault is written down as, so that a crash in the process orb was injected into is evidence
//! rather than a place.
//!
//! **The filter runs inside the faulting process**, which is what makes this worth having at all: the
//! registers are in the exception's own context, the page the fault touched can still be asked about, and
//! the stack can be walked — both games are MSVC6 code whose every function opens `push ebp; mov ebp,esp`,
//! so the chain at `ebp` names the callers. A crash dump would answer the same questions and would have to
//! be written out, moved and parsed first; these are lines in the log beside everything else about the run.
//!
//! **What the callers are for**: a fault inside a call orb made and a fault in the game's own update look
//! identical from the address alone, and they are opposite problems. The chain says which, and where orb's
//! own frames are in it says how far in.

use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::mem;

/// How far the walk goes before it gives up, a stack that is not a stack being the case this exists for:
/// a frame pointer holding a number that happens to be readable would otherwise walk for as long as the
/// numbers held out.
const FRAMES: usize = 16;

/// What the two tables a module's symbols are read out of are made of: a section header is 40 bytes and a
/// symbol is 18, both fixed by the COFF format.
const SECTION_HEADER: usize = 40;
const SYMBOL: usize = 18;

/// What the fault was doing to the address it faulted on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Touching {
    Reading,
    Writing,
    Executing,
}

impl Touching {
    fn as_str(self) -> &'static str {
        match self {
            Self::Reading => "reading",
            Self::Writing => "writing",
            Self::Executing => "executing",
        }
    }
}

/// The registers the exception's own context carries, in the order they are written down.
///
/// All of them rather than the ones this fault happens to need: which register held the pointer is the
/// thing being worked out, so a line that had already chosen would be the answer assumed.
#[derive(Clone, Copy, Default)]
pub struct Registers {
    pub eax: usize,
    pub ebx: usize,
    pub ecx: usize,
    pub edx: usize,
    pub esi: usize,
    pub edi: usize,
    pub esp: usize,
    pub ebp: usize,
}

/// What an address turns out to be part of, in the words orb uses for that memory.
pub struct Whose {
    pub what: String,
    pub base: usize,
    pub len: usize,
}

/// What only the host can answer about a fault: which module an address is in, whose the memory at one is,
/// and whether the heaps are still sound.
///
/// A trait rather than three closures because they are one question — *what was this process doing and to
/// whose memory* — and because the answers come from the same place: see `orb::crash`.
pub trait Around {
    /// The module an address is in and how far into it, for a code address.
    fn name(&self, address: usize) -> String;

    /// What the memory at a *data* address is part of, and nothing for an address orb knows nothing about.
    fn whose(&self, address: usize) -> Option<Whose>;

    /// Every heap this process has that orb knows of, and whether it still validates.
    fn heaps(&self) -> Vec<(usize, bool)>;
}

/// What orb was in the middle of, for the crash to say.
///
/// **Kept as a number rather than a string** because it is written on the way into work a fault happens
/// inside of: a `String` is an allocation, and a fault while the heap is being written to is exactly the
/// case where allocating is what hangs or faults again.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Doing {
    Nothing,
    TakingASnapshot,
    CopyingARegion,
    PuttingARegionBack,
    GivingABufferBack,
    CapturingTheSound,
    PuttingTheSoundBack,
    WalkingTheHeaps,
}

impl Doing {
    fn as_str(self) -> &'static str {
        match self {
            Self::Nothing => "doing nothing it says",
            Self::TakingASnapshot => "taking a snapshot",
            Self::CopyingARegion => "copying a region",
            Self::PuttingARegionBack => "putting a region back",
            Self::GivingABufferBack => "giving a buffer back",
            Self::CapturingTheSound => "capturing the sound",
            Self::PuttingTheSoundBack => "putting the sound back",
            Self::WalkingTheHeaps => "walking the heaps",
        }
    }

    fn from_tag(tag: usize) -> Self {
        match tag {
            1 => Self::TakingASnapshot,
            2 => Self::CopyingARegion,
            3 => Self::PuttingARegionBack,
            4 => Self::GivingABufferBack,
            5 => Self::CapturingTheSound,
            6 => Self::PuttingTheSoundBack,
            7 => Self::WalkingTheHeaps,
            _ => Self::Nothing,
        }
    }

    fn tag(self) -> usize {
        match self {
            Self::Nothing => 0,
            Self::TakingASnapshot => 1,
            Self::CopyingARegion => 2,
            Self::PuttingARegionBack => 3,
            Self::GivingABufferBack => 4,
            Self::CapturingTheSound => 5,
            Self::PuttingTheSoundBack => 6,
            Self::WalkingTheHeaps => 7,
        }
    }
}

/// What orb is in the middle of, and the two numbers that say which piece of it: the region being copied
/// out of how many, the bytes of a buffer, whatever the work counts.
///
/// Three words of atomics and no lock: a crash filter that had to take one could find it held by the thread
/// that faulted holding it.
static DOING: [AtomicUsize; 3] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];

/// Says what orb is in the middle of, for a fault inside it to be read by.
pub fn doing(what: Doing, a: usize, b: usize) {
    DOING[1].store(a, Ordering::Relaxed);
    DOING[2].store(b, Ordering::Relaxed);
    DOING[0].store(what.tag(), Ordering::Relaxed);
}

/// What it says orb is in the middle of, as one reading.
fn was_doing() -> (Doing, usize, usize) {
    (
        Doing::from_tag(DOING[0].load(Ordering::Relaxed)),
        DOING[1].load(Ordering::Relaxed),
        DOING[2].load(Ordering::Relaxed),
    )
}

/// The symbols a module's own file holds, as `(rva, name)` sorted by address.
///
/// **Read out of the file rather than out of the loaded image**, because the symbol table is not part of
/// what the loader maps: `PointerToSymbolTable` is a file offset, and so is the string table after it. orb
/// is running out of a file it unpacked itself, so the one it needs is the one it can open.
///
/// A symbol's address is its section's base plus its own value, which is what a walk of the sections is
/// for. Names come as they are — Rust's `v0` mangling carries the path in fragments a reader can read, and
/// undoing it would be a parser of its own.
pub fn symbols(image: &[u8]) -> Vec<(usize, String)> {
    let at = |at: usize| -> Option<u32> {
        Some(u32::from_le_bytes(image.get(at..at + 4)?.try_into().ok()?))
    };
    let short = |from: usize| -> Option<u16> {
        Some(u16::from_le_bytes(
            image.get(from..from + 2)?.try_into().ok()?,
        ))
    };
    let Some(pe) = at(0x3c).map(|pe| pe as usize) else {
        return Vec::new();
    };
    if image.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Vec::new();
    }
    let Some(sections) = short(pe + 6) else {
        return Vec::new();
    };
    let (Some(table), Some(count), Some(optional)) = (at(pe + 12), at(pe + 16), short(pe + 20))
    else {
        return Vec::new();
    };
    let table = table as usize;
    if table == 0 {
        return Vec::new();
    }
    let headers = pe + 24 + optional as usize;
    let bases: Vec<usize> = (0..sections as usize)
        .filter_map(|section| at(headers + section * SECTION_HEADER + 12).map(|it| it as usize))
        .collect();
    let strings = table + count as usize * SYMBOL;
    let mut symbols = Vec::new();
    let mut entry = 0;
    while entry < count as usize {
        let symbol = table + entry * SYMBOL;
        let (Some(value), Some(section)) = (at(symbol + 8), short(symbol + 12)) else {
            break;
        };
        // The auxiliary entries a symbol carries are entries of the table too, and stepping over them is
        // what keeps the walk on the symbols themselves.
        let aux = image.get(symbol + 17).copied().unwrap_or(0) as usize;
        if let Some(base) = (section as usize)
            .checked_sub(1)
            .and_then(|section| bases.get(section))
            && let Some(name) = name_of(image, symbol, strings)
        {
            symbols.push((base + value as usize, name));
        }
        entry += 1 + aux;
    }
    symbols.sort_by_key(|(rva, _)| *rva);
    symbols
}

/// The symbol an address is inside, as its name and how far into it, and nothing for an address below the
/// first of them.
pub fn nearest(symbols: &[(usize, String)], rva: usize) -> Option<(&str, usize)> {
    let at = symbols
        .partition_point(|(base, _)| *base <= rva)
        .checked_sub(1)?;
    let (base, name) = symbols.get(at)?;
    Some((name.as_str(), rva - base))
}

/// A symbol's name: the eight bytes of the entry where it fits in them, and an offset into the string
/// table where the first four are nothing.
fn name_of(image: &[u8], symbol: usize, strings: usize) -> Option<String> {
    let inline = image.get(symbol..symbol + 8)?;
    if inline[..4] != [0, 0, 0, 0] {
        let end = inline.iter().position(|byte| *byte == 0).unwrap_or(8);
        return Some(String::from_utf8_lossy(&inline[..end]).into_owned());
    }
    let offset = u32::from_le_bytes(inline[4..8].try_into().ok()?) as usize;
    let from = image.get(strings + offset..)?;
    let end = from
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(from.len());
    Some(String::from_utf8_lossy(&from[..end]).into_owned())
}

/// A fault as the filter found it.
pub struct Fault {
    /// Windows' own, which is signed where it is declared: `0xc0000005` is a negative `i32`.
    pub code: i32,
    /// Where the faulting instruction is.
    pub at: usize,
    /// What it was doing and to which address, for the access violations that say so.
    pub touching: Option<(Touching, usize)>,
    pub registers: Registers,
}

/// The lines a fault is written down as, `name` being what turns an address into a module and an offset.
///
/// Every line begins `crash:` and stands on its own, because a log is read by grepping it and a fault is
/// the one thing in there somebody comes looking for.
pub fn lines(fault: &Fault, around: &dyn Around) -> Vec<String> {
    written_down(fault, around, was_doing())
}

/// And with what orb was in the middle of handed in rather than read out of [`DOING`].
///
/// **Which is what makes it testable at all.** [`DOING`] is one word for the whole process and the code that
/// writes it is the snapshot's and the chapter's, so a test that set it and then read the message back was
/// racing every other test in the binary that took a snapshot — and lost, naming whatever that one was doing.
fn written_down(
    fault: &Fault,
    around: &dyn Around,
    was_doing: (Doing, usize, usize),
) -> Vec<String> {
    let name = |address| around.name(address);
    let mut lines = vec![format!(
        "crash: code {:#010x} at {:#010x} in {}",
        fault.code,
        fault.at,
        name(fault.at)
    )];
    if let Some((touching, address)) = fault.touching {
        // Whether the page is there *now*, which is the difference between a pointer that was never
        // anything and one whose block has been freed and given back — the second is the shape a restore
        // leaving a stale pointer behind has.
        lines.push(format!(
            "crash: {} {address:#010x}, which {}",
            touching.as_str(),
            if mem::read_committed::<u8>(address).is_some() {
                "the process can read"
            } else {
                "is not memory the process can read"
            },
        ));
        // **And whose that memory is**, which is the question the address on its own cannot answer: the same
        // number is a field of the game's, a byte of a chapter's own copy or a piece of orb's bookkeeping,
        // and which of the three it is is the whole of what a fault is read for.
        lines.push(match around.whose(address) {
            Some(whose) => format!(
                "crash: {address:#010x} is {}+{:#x}, which runs {:#010x}+{:#x}",
                whose.what,
                address - whose.base,
                whose.base,
                whose.len,
            ),
            None => format!("crash: {address:#010x} is in nothing orb knows the name of"),
        });
    }
    let Registers {
        eax,
        ebx,
        ecx,
        edx,
        esi,
        edi,
        esp,
        ebp,
    } = fault.registers;
    lines.push(format!(
        "crash: eax={eax:#010x} ebx={ebx:#010x} ecx={ecx:#010x} edx={edx:#010x} \
         esi={esi:#010x} edi={edi:#010x} esp={esp:#010x} ebp={ebp:#010x}"
    ));
    // Every register that names memory orb knows, said as what it names. Which register held the pointer is
    // what a fault is worked out from, and a register that names nothing is left out rather than written as
    // a line saying so — eight of those would bury the one that matters.
    for (register, value) in [
        ("eax", eax),
        ("ebx", ebx),
        ("ecx", ecx),
        ("edx", edx),
        ("esi", esi),
        ("edi", edi),
    ] {
        if let Some(whose) = around.whose(value) {
            lines.push(format!(
                "crash: {register} is {}+{:#x}",
                whose.what,
                value - whose.base,
            ));
        }
    }
    // What orb was in the middle of, and whether the heaps are still sound: a fault on a pointer that was
    // never anything and a fault inside a heap somebody had written over look the same from the address, and
    // they are opposite problems.
    let (what, a, b) = was_doing;
    if what != Doing::Nothing {
        // The two numbers only where the work counts something, which is what makes them mean anything:
        // a region being copied is one of five and a buffer being given back is not one of anything.
        lines.push(if b == 0 {
            format!("crash: orb was {}", what.as_str())
        } else {
            format!("crash: orb was {}, {a} of {b}", what.as_str())
        });
    }
    for (heap, sound) in around.heaps() {
        if !sound {
            lines.push(format!("crash: heap {heap:#010x} does not validate"));
        }
    }
    for (frame, at) in callers(ebp).into_iter().enumerate() {
        lines.push(format!("crash: called from {} (frame {frame})", name(at)));
    }
    lines
}

/// The return addresses up the frame-pointer chain: `[ebp]` is the caller's frame and `[ebp+4]` its return
/// address, which is the frame every function of either game opens.
///
/// Stops at a frame pointer that is not readable, one that does not climb — a stack grows downwards, so a
/// chain that goes the other way is not a chain — or at [`FRAMES`].
fn callers(mut ebp: usize) -> Vec<usize> {
    let mut callers = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        let Some(next) = mem::read_committed::<usize>(ebp) else {
            break;
        };
        let Some(at) = mem::read_committed::<usize>(ebp + size_of::<usize>()) else {
            break;
        };
        if at == 0 {
            break;
        }
        callers.push(at);
        if next <= ebp {
            break;
        }
        ebp = next;
    }
    callers
}

#[cfg(test)]
mod tests {
    use super::{
        Around, Doing, Fault, Registers, Touching, Whose, lines, nearest, symbols, written_down,
    };
    use orb_api::Kind;

    /// A host that names modules the way the real one does and knows two ranges: what a chapter saved, and
    /// a buffer of orb's own.
    struct Host;

    /// The range a chapter of this test saved, and the buffer of orb's own that holds the copy of it.
    const SAVED: usize = 0x0aa8_0000;
    const SAVED_BYTES: usize = 0x2000;
    const OURS: usize = 0x1ab8_d020;
    const OURS_BYTES: usize = 0x2000;

    impl Around for Host {
        fn name(&self, address: usize) -> String {
            format!("a module+{address:#x}")
        }

        fn whose(&self, address: usize) -> Option<Whose> {
            let ranges = [
                ("the region a chapter saved", SAVED, SAVED_BYTES),
                ("a buffer of orb's own", OURS, OURS_BYTES),
            ];
            ranges
                .into_iter()
                .find(|(_, base, len)| address >= *base && address < base + len)
                .map(|(what, base, len)| Whose {
                    what: what.to_owned(),
                    base,
                    len,
                })
        }

        fn heaps(&self) -> Vec<(usize, bool)> {
            vec![(0x0015_0000, true), (0x0aa0_0000, false)]
        }
    }

    /// A stack of this test's own, and the two frames on it: the one that faulted and the one that called
    /// it, as a frame-pointer chain — `[ebp]` the next frame up and the word after it the return address.
    ///
    /// **A word and not four bytes**, which is what [`callers`] steps by: this crate is built for a 64-bit
    /// target as well as a 32-bit one, and a chain laid out four bytes apart is one the walk reads the
    /// wrong halves of there. Written down here because the games these faults come out of are both 32-bit
    /// and their own frames really are four apart, so the literal reads as right until the other target
    /// runs — which is what `xtask`'s `ALSO` is for.
    const STACK: usize = 0x0020_0000;
    const STACK_BYTES: usize = 0x1000;
    const INNER: usize = STACK + 0x100;
    const OUTER: usize = STACK + 0x200;
    const RETURNS_TO: usize = 0x0045_1234;
    const CALLED_FROM: usize = 0x1000_5678;

    /// And a block that has been freed and given back, which is the shape a fault on a stale pointer has:
    /// an address in no mapping at all.
    const GONE: usize = 0x0aa8_016c;

    fn faulted(sim: &orb_sim::Sim) -> Vec<String> {
        sim.space().map(STACK, STACK_BYTES, Kind::Private);
        sim.space().write::<usize>(INNER, OUTER);
        sim.space()
            .write::<usize>(INNER + size_of::<usize>(), RETURNS_TO);
        sim.space().write::<usize>(OUTER, 0);
        sim.space()
            .write::<usize>(OUTER + size_of::<usize>(), CALLED_FROM);
        let fault = Fault {
            code: 0xc000_0005u32 as i32,
            at: 0x0045_0db1,
            touching: Some((Touching::Reading, GONE)),
            registers: Registers {
                ecx: GONE - 4,
                ebp: INNER,
                esp: INNER - 0x40,
                ..Registers::default()
            },
        };
        lines(&fault, &Host)
    }

    /// **Whose the memory is, for the address the fault touched and for every register that names one.**
    ///
    /// Which is the question a fault is read to answer and the one an address on its own cannot: `0x18`,
    /// `0x1f` and `0xb7f` were three faults of one th07 session, and what each of them turned on was
    /// whether the pointer they came off belonged to the game, to a chapter's own copy or to orb — hours of
    /// reading a dll with `nm` and a disassembler to say what the process could have said itself.
    #[test]
    fn a_fault_says_whose_the_memory_it_touched_and_holds_is() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        sim.space().map(STACK, STACK_BYTES, Kind::Private);
        sim.space().map(SAVED, SAVED_BYTES, Kind::Private);
        let fault = Fault {
            code: 0xc000_0005u32 as i32,
            at: 0x0045_0db1,
            // Writing into the region a chapter saved, off a pointer held in a buffer of orb's own: the
            // two halves of every question this session asked.
            touching: Some((Touching::Writing, SAVED + 0x40)),
            registers: Registers {
                esi: OURS + 0x1c,
                edi: SAVED + 0x40,
                ..Registers::default()
            },
        };
        let said = lines(&fault, &Host).join("\n");

        assert!(
            said.contains(&format!(
                "crash: {:#010x} is the region a chapter saved+0x40, which runs {SAVED:#010x}+{SAVED_BYTES:#x}",
                SAVED + 0x40
            )),
            "the address the fault touched is not said to be the chapter's own memory:\n{said}",
        );
        assert!(
            said.contains("crash: esi is a buffer of orb's own+0x1c")
                && said.contains("crash: edi is the region a chapter saved+0x40"),
            "the registers holding known memory are not said to:\n{said}",
        );
    }

    /// **What orb was doing**, which is the other half: a fault inside `chapter::take` is one thing while it
    /// copies a region and another while it gives a buffer back, and the address alone says neither.
    #[test]
    fn a_fault_says_what_orb_was_doing_when_it_happened() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        let fault = Fault {
            code: 0xc000_0005u32 as i32,
            at: 0x0045_0db1,
            touching: None,
            registers: Registers::default(),
        };
        let said = written_down(&fault, &Host, (Doing::CopyingARegion, 2, 5)).join("\n");
        assert!(
            said.contains("crash: orb was copying a region, 2 of 5"),
            "the fault does not say what orb was in the middle of:\n{said}",
        );
    }

    /// And says nothing about it where orb is in the middle of nothing, which is every fault that is not
    /// inside one of the four pieces of work [`doing`] is called around.
    #[test]
    fn a_fault_outside_orbs_own_work_says_nothing_about_it() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        let fault = Fault {
            code: 0xc000_0005u32 as i32,
            at: 0x0045_0db1,
            touching: None,
            registers: Registers::default(),
        };
        let said = written_down(&fault, &Host, (Doing::Nothing, 0, 0)).join("\n");
        assert!(
            !said.contains("crash: orb was"),
            "the fault said orb was in the middle of something it was not:\n{said}",
        );
    }

    /// **And orb's own frames by name.** A module and an offset is where the manual work began every time
    /// this session: the dll out of the temp directory it was unpacked into, `nm` over it, the offsets
    /// matched by hand against the sorted symbols, and a disassembler for the faulting instruction. The
    /// symbols are in the file orb is running out of, so orb can say the name itself.
    ///
    /// The names as the linker left them, which for Rust is `v0` mangling: `_RNvNtCs..._8orb_core7chapter4take`
    /// carries the path in plain fragments and demangling it would be a parser of its own for what a reader
    /// can already read.
    #[test]
    fn the_symbols_of_a_module_are_read_out_of_its_own_file() {
        let image = a_module();
        let symbols = symbols(&image);

        assert_eq!(
            symbols,
            [
                (0x1000, "_start".to_owned()),
                (0x1100, "_RNvNtC8orb_core7chapter4take".to_owned()),
            ],
            "the symbol table was not read as the addresses and names it holds",
        );
        // And the nearest one below an address, which is what a fault's own address turns into: the offset
        // into the function and not the module.
        assert_eq!(
            nearest(&symbols, 0x1100 + 0x970),
            Some(("_RNvNtC8orb_core7chapter4take", 0x970)),
        );
        assert_eq!(nearest(&symbols, 0x1000), Some(("_start", 0)));
        assert_eq!(
            nearest(&symbols, 0x100),
            None,
            "nothing lies below the first"
        );
    }

    /// A module of the shape the linker leaves: a `PE` at the offset the DOS header names, one section, and a
    /// symbol table with a name inline and a name in the string table after it.
    fn a_module() -> Vec<u8> {
        const PE: usize = 0x80;
        const OPT: u16 = 0xe0;
        let sections = PE + 24 + OPT as usize;
        let symbols = sections + 40;
        let strings = symbols + 2 * 18;

        let mut image = vec![0u8; strings + 0x40];
        image[0x3c..0x40].copy_from_slice(&u32::try_from(PE).unwrap().to_le_bytes());
        image[PE..PE + 4].copy_from_slice(b"PE\0\0");
        // The COFF header: one section, where the symbols are and how many, and the optional header's size.
        image[PE + 6..PE + 8].copy_from_slice(&1u16.to_le_bytes());
        image[PE + 12..PE + 16].copy_from_slice(&u32::try_from(symbols).unwrap().to_le_bytes());
        image[PE + 16..PE + 20].copy_from_slice(&2u32.to_le_bytes());
        image[PE + 20..PE + 22].copy_from_slice(&OPT.to_le_bytes());
        // One section, `.text`, based where the linker puts it.
        image[sections..sections + 5].copy_from_slice(b".text");
        image[sections + 12..sections + 16].copy_from_slice(&0x1000u32.to_le_bytes());
        // A symbol whose name is short enough to sit in the entry, at the section's own start.
        image[symbols..symbols + 6].copy_from_slice(b"_start");
        image[symbols + 8..symbols + 12].copy_from_slice(&0u32.to_le_bytes());
        image[symbols + 12..symbols + 14].copy_from_slice(&1i16.to_le_bytes());
        // And one whose name is in the string table, 0x100 into the section.
        let name = b"_RNvNtC8orb_core7chapter4take\0";
        image[symbols + 18 + 4..symbols + 18 + 8].copy_from_slice(&4u32.to_le_bytes());
        image[symbols + 18 + 8..symbols + 18 + 12].copy_from_slice(&0x100u32.to_le_bytes());
        image[symbols + 18 + 12..symbols + 18 + 14].copy_from_slice(&1i16.to_le_bytes());
        image[strings..strings + 4]
            .copy_from_slice(&u32::try_from(4 + name.len()).unwrap().to_le_bytes());
        image[strings + 4..strings + 4 + name.len()].copy_from_slice(name);
        image
    }

    /// **And whether the heaps were sound**, which is the difference between a pointer that was never
    /// anything and a heap somebody had already written over: the second is read off the heap and not off
    /// the fault, and every fault of that th07 session was one of the two.
    #[test]
    fn a_fault_says_which_heaps_no_longer_validate() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        let fault = Fault {
            code: 0xc000_0005u32 as i32,
            at: 0x0045_0db1,
            touching: None,
            registers: Registers::default(),
        };
        let said = lines(&fault, &Host).join("\n");
        assert!(
            said.contains("crash: heap 0x0aa00000 does not validate"),
            "the heap that had been written over is not named:\n{said}",
        );
        assert!(
            !said.contains("crash: heap 0x00150000 does not validate"),
            "a heap that validates is named as one that does not:\n{said}",
        );
    }

    /// A fault is written down as where it was, what it touched and whether that is memory at all, the
    /// registers, and the callers up the frame-pointer chain.
    #[test]
    fn a_fault_is_written_down_with_its_callers_and_whether_it_touched_memory_at_all() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        let lines = faulted(&sim);

        let said = lines.join("\n");
        assert_eq!(
            lines[0],
            format!(
                "crash: code 0xc0000005 at 0x00450db1 in a module+{:#x}",
                0x0045_0db1
            ),
        );
        assert!(
            said.contains(&format!(
                "crash: reading {GONE:#010x}, which is not memory the process can read"
            )),
            "the line does not say the block the pointer named has gone:\n{said}",
        );
        assert!(
            said.contains(&format!("ecx={:#010x}", GONE - 4))
                && said.contains(&format!("ebp={INNER:#010x}")),
            "the registers are not written down:\n{said}",
        );
        // By what it says rather than by where the line is: a fault says as much about itself as it can,
        // and what a reader looks for is the line and not its number.
        assert!(
            said.contains(&format!(
                "crash: called from a module+{RETURNS_TO:#x} (frame 0)"
            )) && said.contains(&format!(
                "crash: called from a module+{CALLED_FROM:#x} (frame 1)"
            )),
            "the frame-pointer chain was not walked:\n{said}",
        );
    }

    /// And an address that *is* memory says so, which is what tells a stale pointer from a bad one.
    #[test]
    fn a_fault_on_memory_the_process_still_has_says_so() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        sim.space().map(STACK, STACK_BYTES, Kind::Private);
        let fault = Fault {
            code: 0xc000_0005u32 as i32,
            touching: Some((Touching::Writing, STACK)),
            at: 0x0045_0db1,
            registers: Registers::default(),
        };
        let said = lines(&fault, &Host).join("\n");
        assert!(
            said.contains(&format!(
                "crash: writing {STACK:#010x}, which the process can read"
            )),
            "{said}",
        );
        // And no frames at all, a frame pointer of nothing being no chain: nothing is written rather than
        // a walk that found one frame at address zero.
        assert!(!said.contains("called from"), "{said}");
    }
}
