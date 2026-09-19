//! Noticing where the game's own allocations are.
//!
//! The game keeps most of its state in `.data`, but not all of it: `malloc` in
//! its statically linked MSVC6 CRT goes to a private heap, and a few things come
//! straight from `VirtualAlloc`. Both are reached through the exe's imports, so
//! hooking those imports catches every region the game owns without catching
//! d3d8's or dsound's, whose allocations must be left alone.
//!
//! Heap contents are saved together with the allocator's own bookkeeping, which
//! is what makes a restored snapshot hand back the identical addresses.
//!
//! **Import hooks and nothing else.** What a region is and how two of them are held apart is
//! [`orb_core::memtrack`], where a snapshot reads them; the walk of what these noticed is behind the
//! seam, `HeapLock`, `HeapWalk` and `VirtualQuery` being the host's and a laid-out game answering the
//! same question out of its own address space. So each hook hands the handle or the range over as it
//! sees it — see [`orb_api::mem::note_heap`].
//!
//! **A game linked against the universal CRT has no heap of its own to notice**, and gets one made for it
//! instead — see [`install_ucrt`]. Its `malloc` is `ucrtbase.dll`'s, which allocates from the *process*
//! heap: the one every module in the process shares, so a copy of it is a copy of D3D11's, DXGI's and
//! Steam's own bookkeeping and a restore of it rewinds theirs along with the game's. The imports the
//! exe reaches that allocator through are `malloc`, `free` and — where it has one — `realloc`, and every
//! `new` in the exe goes through the exe's own `operator new` to `malloc` — so standing in front of those
//! and answering out of a heap `HeapCreate` made for the game alone puts the game's memory where th06's
//! own CRT put it: in a heap nothing else allocates from, whose regions the walk finds and whose
//! bookkeeping a restore puts back whole. Which of the two a game is, is `Hooks::takes_its_memory`.
//!
//! **And every one of those imports has to be stood in front of**, which is what [`realloc`] is for: a
//! block `malloc` answered out of the game's own heap and `realloc` then handed to `ucrtbase`'s is a
//! block reallocated through a heap that never handed it out, and Windows ends the process for it
//! without the unhandled filter ever running. th06nc imports the three where th06c imports two, and
//! that third one had nothing in front of it.

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{BOOL, HANDLE};
use windows_sys::Win32::System::Memory::{
    HEAP_FLAGS, HeapAlloc, HeapCreate, HeapFree, HeapReAlloc, HeapValidate, MEM_RELEASE,
    VIRTUAL_ALLOCATION_TYPE, VIRTUAL_FREE_TYPE,
};

use crate::hook;

static HEAP_CREATE: AtomicUsize = AtomicUsize::new(0);
static HEAP_ALLOC: AtomicUsize = AtomicUsize::new(0);
static HEAP_REALLOC: AtomicUsize = AtomicUsize::new(0);
static HEAP_FREE: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_ALLOC: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_FREE: AtomicUsize = AtomicUsize::new(0);

/// The heap made for a game whose own CRT has none — see [`install_ucrt`] — and the CRT's own `free` and
/// `realloc`, for a block that heap never handed out.
static THE_GAMES_HEAP: AtomicUsize = AtomicUsize::new(0);
static FREE: AtomicUsize = AtomicUsize::new(0);
static REALLOC: AtomicUsize = AtomicUsize::new(0);

/// The UCRT's DLL the imports are of. `api-ms-win-crt-heap-l1-1-0.dll` is an API set name and not a
/// file: the loader resolves it to `ucrtbase.dll`, and the import descriptor in the exe carries this name.
const UCRT_HEAP: &str = "api-ms-win-crt-heap-l1-1-0.dll";

/// **The hooks for a game whose allocator is the universal CRT's** — see the module doc, and
/// `orb_core::game::Memory::UcrtImports`.
///
/// **Growable and serialised, and with nothing said about the low-fragmentation heap.** `HeapCreate` with a
/// maximum of nought is a heap that grows, which a stage's `malloc(0x9e338)` per stage record wants; and
/// the default serialisation is what a `malloc` made from the game's streaming thread wants. Whether
/// `HeapWalk` answers regions for a heap the LFH has been turned on for is what the line `attach` writes
/// after this says — `memory: the game's heap walks to N region(s)` — and is measured there rather than
/// decided here.
///
/// # Safety
/// Must run before the game's entry point, or allocations made in between are the process heap's and are
/// freed through a heap that never handed them out. Patches `module`'s imports, so nothing else may be
/// executing it.
pub unsafe fn install_ucrt(module: usize) -> Result<(), hook::Error> {
    let heap = unsafe { HeapCreate(0, 0, 0) };
    if heap.is_null() {
        return Err(hook::Error::OutOfMemory);
    }
    THE_GAMES_HEAP.store(heap as usize, Ordering::Relaxed);
    orb_api::mem::note_heap(heap as usize);
    unsafe {
        // `malloc` first: what was in that slot is not called through, its answers being the process
        // heap's, so nothing of it is kept.
        hook::install_import(module, UCRT_HEAP, "malloc", hook::address(malloc as _))?;
        let previous = hook::install_import(module, UCRT_HEAP, "free", hook::address(free as _))?;
        FREE.store(previous, Ordering::Relaxed);
        // **And `realloc` where the exe has one** — see [`realloc`], and the three launches that say why.
        // An exe without one needs nothing standing in front of it, so a missing import is not an error
        // here: th06nc imports all three and th06c only the two.
        match hook::install_import(module, UCRT_HEAP, "realloc", hook::address(realloc as _)) {
            Ok(previous) => REALLOC.store(previous, Ordering::Relaxed),
            Err(hook::Error::ImportNotFound { .. }) => {}
            Err(why) => return Err(why),
        }
    }
    Ok(())
}

type Free = unsafe extern "C" fn(*mut c_void);
type Realloc = unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void;

/// The exe's `malloc`, answered out of the game's own heap.
///
/// `malloc(0)` answers a block the way the CRT's does: `HeapAlloc` with nought bytes is a block of its own
/// too, so a `free` of it lands in the heap that made it.
unsafe extern "C" fn malloc(bytes: usize) -> *mut c_void {
    let started = orb_core::profile::now();
    let heap = THE_GAMES_HEAP.load(Ordering::Relaxed) as HANDLE;
    let block = unsafe { HeapAlloc(heap, 0, bytes) };
    orb_core::profile::record_heap(started);
    block
}

/// And its `free`, for the blocks that heap made — and the CRT's own for any other.
///
/// **Asked first rather than freed and seen**: `HeapFree` handed a block of another heap's is a corruption
/// of this one's bookkeeping and not an error it answers, so the block is held against the heap before
/// anything is done with it. What else the exe could be handed to free is a block a DLL of the CRT's
/// allocated for it — nothing of the game's own does, every `new` being the exe's, but the CRT's `free` is
/// what such a block wants and is what it gets.
unsafe extern "C" fn free(block: *mut c_void) {
    if block.is_null() {
        return;
    }
    let started = orb_core::profile::now();
    let heap = THE_GAMES_HEAP.load(Ordering::Relaxed) as HANDLE;
    if unsafe { HeapValidate(heap, 0, block) } != 0 {
        if unsafe { HeapFree(heap, 0, block) } != 0 {
            keep_the_space(block as usize);
        }
        orb_core::profile::record_heap(started);
        return;
    }
    let original: Free = unsafe { std::mem::transmute(FREE.load(Ordering::Relaxed)) };
    unsafe { original(block) };
    orb_core::profile::record_heap(started);
}

/// And its `realloc`, for the blocks that heap made — and the CRT's own for any other.
///
/// **Which is a hook th06nc needs and th06c does not**: `objdump -p` puts `realloc` in th06nc's imports
/// beside `malloc` and `free` where th06c imports only the two, and both of its call sites are one array
/// being grown — `0x140056e8b` mallocs the buffer where the object holds none and reallocs it where it
/// holds one, and `0x1400571b5` does the same over an array of `0x10`-byte items. So the pointer handed
/// here came out of [`malloc`] above, which is *this* heap and not the process heap `ucrtbase`'s own
/// `realloc` would look it up in. A block reallocated through a heap that never handed it out is the
/// corruption Windows ends a process for on the spot, and it ends it without the unhandled filter
/// [`crate::crash`] installs ever running — which is exactly how this was found and why it took a
/// comparison of two games to see.
///
/// **Read off the exe and not off a launch.** What is measured is the import and the two call sites; no
/// launch has been seen going through either — a session driven with this hook in wrote
/// `memory: kept` not once — so what this closes is a hazard the exe carries and not a fault anybody has
/// watched. **It is not what killed the th06nc launches it was written for**: the same case died the same
/// way with it in place, which is that change's own TODO.
///
/// The block is held against the heap first, for the reason [`free`] gives; `realloc(null, n)` is
/// `malloc(n)` and `realloc(p, 0)` frees and answers null, which is what the UCRT's own does.
unsafe extern "C" fn realloc(block: *mut c_void, bytes: usize) -> *mut c_void {
    if block.is_null() {
        return unsafe { malloc(bytes) };
    }
    if bytes == 0 {
        unsafe { free(block) };
        return std::ptr::null_mut();
    }
    let started = orb_core::profile::now();
    let heap = THE_GAMES_HEAP.load(Ordering::Relaxed) as HANDLE;
    if unsafe { HeapValidate(heap, 0, block) } != 0 {
        let moved = unsafe { HeapReAlloc(heap, 0, block, bytes) };
        // **The space the block left, where it moved**, which is the same note [`free`] makes: a
        // reallocation that moves frees the old one, and a chapter of this stage would put it back.
        if !moved.is_null() && moved != block {
            keep_the_space(block as usize);
        }
        orb_core::profile::record_heap(started);
        return moved;
    }
    let original: Realloc = unsafe { std::mem::transmute(REALLOC.load(Ordering::Relaxed)) };
    let answer = unsafe { original(block, bytes) };
    orb_core::profile::record_heap(started);
    answer
}

/// # Safety
/// Must run before the game's entry point, or allocations made in between go
/// unrecorded. Patches `module`'s imports, so nothing else may be executing it.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    unsafe {
        for (function, replacement, original) in [
            ("HeapCreate", hook::address(heap_create as _), &HEAP_CREATE),
            ("HeapAlloc", hook::address(heap_alloc as _), &HEAP_ALLOC),
            (
                "HeapReAlloc",
                hook::address(heap_realloc as _),
                &HEAP_REALLOC,
            ),
            ("HeapFree", hook::address(heap_free as _), &HEAP_FREE),
            (
                "VirtualAlloc",
                hook::address(virtual_alloc as _),
                &VIRTUAL_ALLOC,
            ),
            (
                "VirtualFree",
                hook::address(virtual_free as _),
                &VIRTUAL_FREE,
            ),
        ] {
            let previous = hook::install_import(module, "KERNEL32.dll", function, replacement)?;
            original.store(previous, Ordering::Relaxed);
        }
    }
    Ok(())
}

type HeapCreate = unsafe extern "system" fn(HEAP_FLAGS, usize, usize) -> HANDLE;
type HeapAlloc = unsafe extern "system" fn(HANDLE, HEAP_FLAGS, usize) -> *mut c_void;
type HeapReAlloc =
    unsafe extern "system" fn(HANDLE, HEAP_FLAGS, *const c_void, usize) -> *mut c_void;
type HeapFree = unsafe extern "system" fn(HANDLE, HEAP_FLAGS, *const c_void) -> BOOL;
type VirtualAlloc =
    unsafe extern "system" fn(*const c_void, usize, VIRTUAL_ALLOCATION_TYPE, u32) -> *mut c_void;
type VirtualFree = unsafe extern "system" fn(*mut c_void, usize, VIRTUAL_FREE_TYPE) -> BOOL;

unsafe extern "system" fn heap_create(flags: HEAP_FLAGS, initial: usize, max: usize) -> HANDLE {
    let original: HeapCreate = unsafe { std::mem::transmute(HEAP_CREATE.load(Ordering::Relaxed)) };
    let heap = unsafe { original(flags, initial, max) };
    if !heap.is_null() {
        orb_api::mem::note_heap(heap as usize);
    }
    heap
}

unsafe extern "system" fn heap_alloc(heap: HANDLE, flags: HEAP_FLAGS, bytes: usize) -> *mut c_void {
    let original: HeapAlloc = unsafe { std::mem::transmute(HEAP_ALLOC.load(Ordering::Relaxed)) };
    orb_api::mem::note_heap(heap as usize);
    unsafe { original(heap, flags, bytes) }
}

unsafe extern "system" fn heap_realloc(
    heap: HANDLE,
    flags: HEAP_FLAGS,
    memory: *const c_void,
    bytes: usize,
) -> *mut c_void {
    let original: HeapReAlloc =
        unsafe { std::mem::transmute(HEAP_REALLOC.load(Ordering::Relaxed)) };
    orb_api::mem::note_heap(heap as usize);
    unsafe { original(heap, flags, memory, bytes) }
}

unsafe extern "system" fn heap_free(
    heap: HANDLE,
    flags: HEAP_FLAGS,
    memory: *const c_void,
) -> BOOL {
    let original: HeapFree = unsafe { std::mem::transmute(HEAP_FREE.load(Ordering::Relaxed)) };
    orb_api::mem::note_heap(heap as usize);
    let freed = unsafe { original(heap, flags, memory) };
    if freed != 0 {
        keep_the_space(memory as usize);
    }
    freed
}

/// The address space of a block the game has just handed back, where a chapter would put it back.
///
/// **After the free and not before it**, which is the whole of what this catches: a block of a few
/// megabytes is a reservation of its own inside the heap, so freeing it is the heap manager's own
/// `VirtualFree` — inside `ntdll`, where no hook on the exe's imports reaches — and what says it happened
/// is the space being free afterwards. Whoever is given it next is who a restore would write over.
fn keep_the_space(address: usize) {
    if let Some((base, len)) = orb_api::mem::hold_what_a_restore_needs(address) {
        crate::log::line(&format!(
            "memory: kept {base:#010x}+{len:#x}, which a chapter of this stage would put back"
        ));
    }
}

unsafe extern "system" fn virtual_alloc(
    address: *const c_void,
    size: usize,
    allocation_type: VIRTUAL_ALLOCATION_TYPE,
    protection: u32,
) -> *mut c_void {
    let original: VirtualAlloc =
        unsafe { std::mem::transmute(VIRTUAL_ALLOC.load(Ordering::Relaxed)) };
    let allocated = unsafe { original(address, size, allocation_type, protection) };
    if !allocated.is_null() {
        orb_api::mem::note_reservation(allocated as usize, size);
    }
    allocated
}

unsafe extern "system" fn virtual_free(
    address: *mut c_void,
    size: usize,
    free_type: VIRTUAL_FREE_TYPE,
) -> BOOL {
    let original: VirtualFree =
        unsafe { std::mem::transmute(VIRTUAL_FREE.load(Ordering::Relaxed)) };
    let freed = unsafe { original(address, size, free_type) };
    if freed != 0 && free_type & MEM_RELEASE != 0 {
        orb_api::mem::forget_reservation(address as usize);
        keep_the_space(address as usize);
    }
    freed
}
