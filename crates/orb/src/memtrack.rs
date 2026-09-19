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
//! **A game whose CRT is the universal one linked into the exe asks the process for its heap**, and gets
//! one made for it instead — see [`install_process_heap`]. That CRT's `malloc` is `HeapAlloc` on the
//! handle `GetProcessHeap` answered it at startup, which is the *process* heap: the one every module in
//! the process shares, so a copy of it is a copy of D3D11's, DXGI's and Steam's own bookkeeping and a
//! restore of it rewinds theirs along with the game's. Answering that one question with a heap
//! `HeapCreate` made for the game alone puts the game's memory where th06's own CRT put it: in a heap
//! nothing else allocates from, whose regions the walk finds and whose bookkeeping a restore puts back
//! whole. Which of the two a game is, is `Hooks::takes_its_memory`.

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{BOOL, HANDLE};
use windows_sys::Win32::System::Memory::{
    HEAP_FLAGS, HeapCreate, MEM_RELEASE, VIRTUAL_ALLOCATION_TYPE, VIRTUAL_FREE_TYPE,
};

use crate::hook;

static HEAP_CREATE: AtomicUsize = AtomicUsize::new(0);
static HEAP_ALLOC: AtomicUsize = AtomicUsize::new(0);
static HEAP_REALLOC: AtomicUsize = AtomicUsize::new(0);
static HEAP_FREE: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_ALLOC: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_FREE: AtomicUsize = AtomicUsize::new(0);

/// The heap made for a game whose own CRT asks for the process's — see [`install_process_heap`].
static THE_GAMES_HEAP: AtomicUsize = AtomicUsize::new(0);

/// **The hooks for a game whose UCRT is linked into the exe** — see
/// `orb_core::game::Memory::ProcessHeapImport`: `GetProcessHeap` answered with a heap made for the game
/// alone, which is the one handle that CRT allocates from, and `HeapReAlloc` and `HeapFree` stood in front
/// of for the space a block leaves — see [`keep_the_space`].
///
/// **Growable and serialised, and with nothing said about the low-fragmentation heap.** `HeapCreate` with a
/// maximum of nought is a heap that grows, which a stage's `malloc(0x9e338)` per stage record wants; and
/// the default serialisation is what a `malloc` made from the game's streaming thread wants. Whether
/// `HeapWalk` answers regions for a heap the LFH has been turned on for is what the line `attach` writes
/// after this says — `memory: the game's heap walks to N region(s)` — and is measured there rather than
/// decided here.
///
/// # Safety
/// Must run before the game's entry point, which is where that CRT asks for the heap and keeps the answer:
/// allocations made before it are the process heap's and are freed through a heap that never handed them
/// out. Patches `module`'s imports, so nothing else may be executing it.
pub unsafe fn install_process_heap(module: usize) -> Result<(), hook::Error> {
    let heap = unsafe { HeapCreate(0, 0, 0) };
    if heap.is_null() {
        return Err(hook::Error::OutOfMemory);
    }
    THE_GAMES_HEAP.store(heap as usize, Ordering::Relaxed);
    orb_api::mem::note_heap(heap as usize);
    unsafe {
        hook::install_import(
            module,
            "KERNEL32.dll",
            "GetProcessHeap",
            hook::address(get_process_heap as _),
        )?;
        for (function, replacement, original) in [
            (
                "HeapReAlloc",
                hook::address(heap_realloc as _),
                &HEAP_REALLOC,
            ),
            ("HeapFree", hook::address(heap_free as _), &HEAP_FREE),
        ] {
            let previous = hook::install_import(module, "KERNEL32.dll", function, replacement)?;
            original.store(previous, Ordering::Relaxed);
        }
    }
    Ok(())
}

/// The exe's `GetProcessHeap`, answered with the game's own.
unsafe extern "system" fn get_process_heap() -> HANDLE {
    THE_GAMES_HEAP.load(Ordering::Relaxed) as HANDLE
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
