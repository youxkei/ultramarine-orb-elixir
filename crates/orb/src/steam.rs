//! The writes over the exe's imports of `steam_api64.dll`, so that a launch nobody is watching reaches
//! Steam not at all.
//!
//! **What it is for is somebody's account.** A run the machine plays is not a run they played, and a
//! session that starts the game a hundred times would put a hundred launches of playtime on it, unlock
//! whatever a stage reached unlocks, and hand Steam a save to sync. None of that belongs to a
//! measurement.
//!
//! **And it is what makes a launch need no Steam client at all**, which is the other half: this game
//! calls `SteamInternal_SteamAPI_Init` at `0x1400488a1` and shows
//! `SteamAPI_Init() failed. Steam must be running to play this game.` and exits where it answers
//! anything but nought — measured, its own `log.txt` ending at the 122 bytes of its header. Answered
//! here, the game goes on to its own startup dialog with nothing running beside it.
//!
//! **Two of the nine are answered and the rest are left alone.** What the game asks that DLL for is
//! `SteamUtils`, `STEAMUSERSTATS_INTERFACE_VERSION013` and `STEAMAPPS_INTERFACE_VERSION...`, each
//! through `SteamInternal_FindOrCreateUserInterface` at `0x140048790` onwards — and with no client
//! there those answer nothing of their own accord. There is no `ISteamRemoteStorage` among them, so the
//! save file is not this API's business either way.
//!
//! Everything these decide is a constant, so there is nothing above the seam for them to decide with.

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::hook;

/// `bool SteamAPI_RestartAppIfNecessary(uint32 unOwnAppID)`, answered **false**: the game asks Steam to
/// start it again and exits where this is true, so a launch orb injected into would be replaced by one
/// it did not.
///
/// The app id the game passes is its own — `0x48ce48` at `0x140048889` — and is not read here.
unsafe extern "system" fn restart_app_if_necessary(_app: u32) -> u8 {
    0
}

/// `ESteamAPIInitResult SteamInternal_SteamAPI_Init(const char *version, SteamErrMsg *error)`, answered
/// **nought**, which is the value the game tests for: `test eax,eax; je` at `0x1400488a7`.
///
/// The error message is left as the game's own buffer had it: what reads it is the failure path, and
/// this is the answer that says there was none.
unsafe extern "system" fn steam_api_init(_version: *const u8, _error: *mut u8) -> i32 {
    0
}

/// `void SteamAPI_RunCallbacks()`, answered with nothing: there is no client to have called anything
/// back, and the real one dispatches against an API this launch never initialised.
unsafe extern "system" fn run_callbacks() {}

/// `void SteamAPI_Shutdown()`, the same.
unsafe extern "system" fn shutdown() {}

/// `HSteamUser SteamAPI_GetHSteamUser()`, answered **nought**: it is the handle the interface fetches
/// are made with, and what they are answered with here is the same whatever it is.
unsafe extern "system" fn get_h_steam_user() -> i32 {
    0
}

/// `void SteamAPI_RegisterCallback(CCallbackBase*, int)` and its unregister, answered with nothing:
/// what would call one back is a client, and there is none.
unsafe extern "system" fn register_callback(_callback: *mut c_void, _kind: i32) {}
unsafe extern "system" fn unregister_callback(_callback: *mut c_void) {}

/// **How many slots orb's own interface has.**
///
/// A bound rather than a count of anything: what the game calls on one of these is whatever slot its
/// own header had, and the real interfaces are tens of methods long. Every one of them answers nought —
/// see [`nothing_at_all`] — so the only cost of a slot nobody calls is eight bytes.
const SLOTS: usize = 128;

/// **Which slot the game's own language comes out of**: `ISteamApps::GetCurrentGameLanguage`, called as
/// `[vtable+0x20]` at `0x14004899f` and its answer compared against `japanese`, `schinese` and the rest
/// at `0x1402c82c8` onwards. An interface that answered nought there is a null string the game compares,
/// which is a fault rather than a language.
const LANGUAGE: usize = 4;

/// And what it answers, which is this build's own: the strings it is held against are Steam's own names
/// for a language, and `japanese` is the one this game was read in.
const JAPANESE: &[u8] = b"japanese\0";

/// Every method of every interface orb hands over, which answers nought.
///
/// **Nought and not a crash**, because what a run needs of Steam is nothing: the interfaces this game
/// asks for are `SteamUtils`, `STEAMUSERSTATS_INTERFACE_VERSION013` and `STEAMAPPS_INTERFACE_VERSION…`,
/// and between them they say what somebody has unlocked and which language they bought — neither of
/// which a run the machine played is adding to.
unsafe extern "system" fn nothing_at_all() -> usize {
    0
}

/// Except the language, which is a string or a fault.
unsafe extern "system" fn the_language(_this: *mut c_void) -> *const u8 {
    JAPANESE.as_ptr()
}

/// The interface orb hands over, as a pointer to a pointer to [`SLOTS`] of [`nothing_at_all`] — which is
/// what an interface *is* to the game: one indirection to a vtable.
static ORBS_OWN: AtomicUsize = AtomicUsize::new(0);

/// Makes it, once.
fn orbs_own_interface() -> usize {
    let made = ORBS_OWN.load(Ordering::Relaxed);
    if made != 0 {
        return made;
    }
    let mut vtable = vec![hook::address(nothing_at_all as _); SLOTS];
    vtable[LANGUAGE] = hook::address(the_language as _);
    let vtable = Box::leak(vtable.into_boxed_slice()).as_ptr() as usize;
    let interface = Box::leak(Box::new(vtable)) as *mut usize as usize;
    ORBS_OWN.store(interface, Ordering::Relaxed);
    interface
}

/// `void *SteamInternal_FindOrCreateUserInterface(HSteamUser, const char *version)`, answered with
/// orb's own — see [`orbs_own_interface`].
///
/// **The same one for every name asked for.** Which interface a name is decides what its slots mean,
/// and every slot of orb's answers nought whatever it means; the one exception is the language, and no
/// other interface of these three is asked for a string at all.
unsafe extern "system" fn find_or_create_user_interface(
    _user: i32,
    _version: *const u8,
) -> *mut c_void {
    orbs_own_interface() as *mut c_void
}

/// `void *SteamInternal_ContextInit(void *data)`, done the way the SDK's own does it: the data is
/// `{ void (*fill)(void *context); uintptr_t counter; void *context }`, and what comes back is the
/// address of that context.
///
/// **Answered rather than called through, and it is the one that has to be.** It is where this game
/// reaches every interface — `0x14004897c` and `0x140048993`, each followed by `mov rcx,[rax]` and a
/// call through the vtable that lands on — and the real one hands back a context the client filled,
/// which with no client is nothing for those two instructions to walk.
///
/// The `fill` it calls is the game's own accessor, which calls
/// [`find_or_create_user_interface`] — so the interface the game ends up holding is orb's because the
/// game put it there.
unsafe extern "system" fn context_init(data: *mut usize) -> *mut usize {
    let context = unsafe { data.add(2) };
    if unsafe { data.add(1).read() } == 0 {
        let fill: unsafe extern "system" fn(*mut usize) =
            unsafe { std::mem::transmute(data.read()) };
        unsafe { fill(context) };
        unsafe { data.add(1).write(1) };
    }
    context
}

/// # Safety
/// `module` must be the exe, and this must run before the game's own startup has reached Steam.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    // **The two that decide whether the game runs at all go in first**, and a failure on either is the
    // whole installation's: a launch that answered the restart and not the init is one that reaches the
    // init with no client and exits, which is worse than not having tried.
    for (name, replacement) in [
        (
            "SteamAPI_RestartAppIfNecessary",
            hook::address(restart_app_if_necessary as _),
        ),
        (
            "SteamInternal_SteamAPI_Init",
            hook::address(steam_api_init as _),
        ),
    ] {
        unsafe { hook::install_import(module, "steam_api64.dll", name, replacement)? };
    }
    // And the two the game reaches its interfaces through, which are the other pair it cannot run
    // without: `context_init` says why.
    for (name, replacement) in [
        (
            "SteamInternal_ContextInit",
            hook::address(context_init as _),
        ),
        (
            "SteamInternal_FindOrCreateUserInterface",
            hook::address(find_or_create_user_interface as _),
        ),
    ] {
        unsafe { hook::install_import(module, "steam_api64.dll", name, replacement)? };
    }
    // And the rest, which would run against an API this launch never initialised. Not fatal: what they
    // do with no client is nothing, and a launch that got the four above is a launch that plays.
    for (name, replacement) in [
        ("SteamAPI_RunCallbacks", hook::address(run_callbacks as _)),
        ("SteamAPI_Shutdown", hook::address(shutdown as _)),
        (
            "SteamAPI_GetHSteamUser",
            hook::address(get_h_steam_user as _),
        ),
        (
            "SteamAPI_RegisterCallback",
            hook::address(register_callback as _),
        ),
        (
            "SteamAPI_UnregisterCallback",
            hook::address(unregister_callback as _),
        ),
    ] {
        if let Err(error) =
            unsafe { hook::install_import(module, "steam_api64.dll", name, replacement) }
        {
            crate::log::line(&format!("steam: {error}; that one stays the DLL's own"));
        }
    }
    Ok(())
}
