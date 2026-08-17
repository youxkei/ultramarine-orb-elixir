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

use orb_api::mem;

/// How far the walk goes before it gives up, a stack that is not a stack being the case this exists for:
/// a frame pointer holding a number that happens to be readable would otherwise walk for as long as the
/// numbers held out.
const FRAMES: usize = 16;

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
pub fn lines(fault: &Fault, name: &dyn Fn(usize) -> String) -> Vec<String> {
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
    use super::{Fault, Registers, Touching, lines};
    use orb_api::Kind;

    /// A stack of this test's own, and the two frames on it: the one that faulted and the one that called
    /// it, as a frame-pointer chain — `[ebp]` the next frame up and `[ebp+4]` the return address.
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
        sim.space().write::<usize>(INNER + 4, RETURNS_TO);
        sim.space().write::<usize>(OUTER, 0);
        sim.space().write::<usize>(OUTER + 4, CALLED_FROM);
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
        lines(&fault, &|address| format!("a module+{address:#x}"))
    }

    /// A fault is written down as where it was, what it touched and whether that is memory at all, the
    /// registers, and the callers up the frame-pointer chain.
    #[test]
    fn a_fault_is_written_down_with_its_callers_and_whether_it_touched_memory_at_all() {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let _installed = sim.enter();
        let lines = faulted(&sim);

        assert_eq!(
            lines[0],
            format!(
                "crash: code 0xc0000005 at 0x00450db1 in a module+{:#x}",
                0x0045_0db1
            ),
        );
        assert_eq!(
            lines[1],
            format!("crash: reading {GONE:#010x}, which is not memory the process can read"),
            "the line does not say the block the pointer named has gone",
        );
        assert!(
            lines[2].contains(&format!("ecx={:#010x}", GONE - 4))
                && lines[2].contains(&format!("ebp={INNER:#010x}")),
            "the registers are not written down: {}",
            lines[2],
        );
        assert_eq!(
            &lines[3..],
            [
                format!("crash: called from a module+{RETURNS_TO:#x} (frame 0)"),
                format!("crash: called from a module+{CALLED_FROM:#x} (frame 1)"),
            ],
            "the frame-pointer chain was not walked",
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
        let lines = lines(&fault, &|_| "a module".to_owned());
        assert_eq!(
            lines[1],
            format!("crash: writing {STACK:#010x}, which the process can read"),
        );
        // And no frames at all, a frame pointer of nothing being no chain: the line is left out rather
        // than written as a walk that found one frame at address zero.
        assert_eq!(lines.len(), 3, "{lines:?}");
    }
}
