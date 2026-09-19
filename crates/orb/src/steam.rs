//! The writes over the exe's imports of `steam_api64.dll`, so that a launch asked for it reaches Steam
//! not at all.
//!
//! **Which launches those are is `Config::without_steam_wanted`**: what `without_steam` in `orb.yaml` or
//! `--without-steam` says, and otherwise whether the launch plays itself.
//!
//! **What it is for is somebody's account.** A run the machine plays is not a run they played, and a
//! session that starts the game a hundred times would put a hundred launches of playtime on it, unlock
//! whatever a stage reached unlocks, and hand Steam a save to sync. None of that belongs to a
//! measurement.
//!
//! **And it is what makes a launch need no Steam client at all**, which is the other half: this game
//! calls `SteamInternal_SteamAPI_Init` at `0x140048961` and shows
//! `SteamAPI_Init() failed. Steam must be running to play this game.` and exits where it answers
//! anything but nought — measured, its own `log.txt` ending at the 122 bytes of its header. Answered
//! here, the game goes on to its own startup dialog with nothing running beside it.
//!
//! **All nine imports are answered**, the four the game cannot run without first. What it asks that DLL
//! for is four interfaces, each through a getter of its own reaching
//! `SteamInternal_FindOrCreateUserInterface`: `SteamUtils010` at `0x140048850`,
//! `STEAMUSERSTATS_INTERFACE_VERSION013` at `0x140048880`, `STEAMAPPS_INTERFACE_VERSION009` at
//! `0x1400488b0` and `SteamInput006` at `0x1400488e0`. There is no `ISteamRemoteStorage` among them, so
//! the save file is not this API's business either way.
//!
//! **And eight methods over seven slots of those four are called**, which is what says a vtable of
//! noughts is an answer and not a hope. Each name was read off the DLL that ships beside the exe — see
//! `th06c/asm.md`, which has the two commands it takes:
//!
//! | interface | slot | method | what the game does with the answer |
//! | --- | --- | --- | --- |
//! | Utils | `+0x110` | `IsSteamRunningOnSteamDeck` | nought is a desktop, which is what this is |
//! | UserStats | `+0x30` | `SetAchievement` | nought is "not set", and there is no account to set it on. Read off th06nc, whose `0x1400693c0` is the one function both of its achievements go through — `Achievement01` as a run starts and `Achievement21` as the name entry ends, see `th06nc/asm.md`'s *Steam* — and it is the reason a driven run wants this module and not only a client that is closed |
//! | UserStats | `+0x48` | `StoreStats` | nought is "not stored", and there is nothing to store to; the site is in the frame loop behind a flag the answer would clear, so it is asked again every frame and costs a call into this module. `0x1400693c0` tail-calls it after the set above |
//! | Apps | `+0x20` | `GetCurrentGameLanguage` | a string the game compares, which is why [`LANGUAGE`] is the one slot that answers something |
//! | Input | `+0x00` | `Init` | nought, and the game ignores it |
//! | Input | `+0x08` | `Shutdown` | the same |
//! | Input | `+0x30` | `GetConnectedControllers` | nought is no Steam Input controller, and the game passes over the rest of that path |
//! | Input | `+0x128` | `GetInputTypeForHandle` | out of reach behind the line above, no handle having been answered |
//!
//! Everything these decide is a constant, so there is nothing above the seam for them to decide with.
//!
//! **And th06nc is the same DLL asked the same four questions**, which is why this module names no game:
//! every write here is over an *import by name* and not over an address, so what makes it a game's is
//! only that the game imports `steam_api64.dll`. The interface versions are the reading that would have
//! moved a slot, and they do not —
//!
//! ```sh
//! strings -a th06nc.exe | grep -E 'SteamUtils[0-9]+|STEAMUSERSTATS_INTERFACE_VERSION[0-9]+|STEAMAPPS_INTERFACE_VERSION[0-9]+|SteamInput[0-9]+' | sort -u
//! ```
//!
//! — answers `SteamUtils010`, `STEAMUSERSTATS_INTERFACE_VERSION013`, `STEAMAPPS_INTERFACE_VERSION009`
//! and `SteamInput006`, the four th06c asks for, so the seven slots above are that build's too.
//!
//! **Driven on it, with no client running at all**: five launches of `cargo xtask real` over th06nc each
//! said `steam: this launch reaches Steam not at all` and played their case out — the game's own startup
//! dialog and then a run — where the same exe with Steam closed and this not installed exits at
//! `SteamAPI_Init() failed.`

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::hook;

/// `bool SteamAPI_RestartAppIfNecessary(uint32 unOwnAppID)`, answered **false**: the game asks Steam to
/// start it again and exits where this is true, so a launch orb injected into would be replaced by one
/// it did not.
///
/// The app id the game passes is its own — `0x48ce48` at `0x140048949` — and is not read here.
unsafe extern "system" fn restart_app_if_necessary(_app: u32) -> u8 {
    0
}

/// `ESteamAPIInitResult SteamInternal_SteamAPI_Init(const char *version, SteamErrMsg *error)`, answered
/// **nought**, which is the value the game tests for: `test eax,eax; je` at `0x140048967`.
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
///
/// **The furthest one this game reaches is 37**, `ISteamInput::GetInputTypeForHandle` at `+0x128`, and
/// the table in this module's own doc is every slot it calls — so a hundred and twenty-eight is three
/// times what has been measured and the room is what an interface version nobody has read yet would
/// need. `tests::CALLED` is that table as indices, held against this.
const SLOTS: usize = 128;

/// **Which slot the game's own language comes out of**: `ISteamApps::GetCurrentGameLanguage`, called as
/// `[vtable+0x20]` at `0x140048a5e` and its answer compared against `japanese`, `schinese` and the rest
/// at `0x14030ccd8` onwards. An interface that answered nought there is a null string the game compares,
/// which is a fault rather than a language.
///
/// **A slot index and not the offset**, which is what the table above carries: `0x20 / 8`.
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
/// reaches every interface — `0x140048a3b` and `0x140048a52`, each followed by `mov rcx,[rax]` and a
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

#[cfg(test)]
mod tests {
    use super::{LANGUAGE, SLOTS, context_init};

    /// **The slots this game is measured to call, as indices** — the table in this module's own doc, by
    /// the offsets it names: `0x110`, `0x48`, `0x20`, `0x00`, `0x08`, `0x30` and `0x128`. Seven rather
    /// than the table's eight rows, `0x30` being two methods of two interfaces at the one slot.
    ///
    /// Here rather than beside [`SLOTS`] because nothing but a test reads it: what the interface answers
    /// a slot with does not depend on which slot it is — every one but the language's is nought — so the
    /// reading is a claim to hold that bound against and not a table the code walks.
    const CALLED: [usize; 7] = [34, 9, LANGUAGE, 0, 1, 6, 37];

    /// **Every slot this game calls is one the interface orb hands over has.**
    ///
    /// A test rather than a `const` assert, so that a failure names the slot: what it is about is the
    /// reading in this module's own doc, and a version of an interface that reached further than
    /// [`SLOTS`] would be a call off the end of a `Box` orb leaked.
    #[test]
    fn every_slot_the_game_calls_is_one_the_interface_has() {
        for slot in CALLED {
            assert!(
                slot < SLOTS,
                "slot {slot} is past the {SLOTS} the interface has",
            );
        }
    }

    /// And the one that answers something is the language's, which is the only slot whose nought would
    /// be a fault rather than an answer.
    #[test]
    fn the_language_is_the_only_slot_answered_with_anything() {
        assert!(CALLED.contains(&LANGUAGE));
        assert_eq!(LANGUAGE, 0x20 / 8, "the language is not the slot read");
    }

    /// **The context is filled once and its own slot is what comes back**, which is what the game's two
    /// instructions after the call walk: `mov rcx,[rax]` and then the vtable out of that.
    ///
    /// The fill is a function of this test's own rather than the game's accessor: what is under test is
    /// the SDK's own shape — `{ fill, counter, context }` — and that orb answers the third word's
    /// address, which is the whole of why this is answered rather than called through.
    #[test]
    fn the_context_is_filled_once_and_answered_as_its_own_slot() {
        static FILLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        const INTERFACE: usize = 0x1234_5678;

        unsafe extern "system" fn fill(context: *mut usize) {
            FILLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            unsafe { context.write(INTERFACE) };
        }

        let mut block = [fill as *const () as usize, 0, 0];
        let answered = unsafe { context_init(block.as_mut_ptr()) };
        assert_eq!(answered, unsafe { block.as_mut_ptr().add(2) });
        assert_eq!(unsafe { answered.read() }, INTERFACE);
        assert_eq!(FILLS.load(std::sync::atomic::Ordering::Relaxed), 1);

        // And again, which is what every frame of the game does: the counter beside the fill is what
        // says it has been made, so the second call is the same address and no second fill.
        let again = unsafe { context_init(block.as_mut_ptr()) };
        assert_eq!(again, answered);
        assert_eq!(FILLS.load(std::sync::atomic::Ordering::Relaxed), 1);
    }
}
