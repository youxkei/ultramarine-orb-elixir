# 16. The game is handed to a thcrap installed beside it, and orb's DLL goes in after it

**Status:** accepted and built. `launcher::thcrap` reads a thcrap out of the launcher its own
configuration tool left in the game's directory, brings its patch stack up to date through
`stack_update_wrapper`, and hands the suspended game to `WaitUntilEntryPoint` and
`thcrap_inject_into_running` before `inject::load_library` puts orb's own DLL in. `thcrap: false` in
`orb.yaml` turns the whole of it off. What orb draws in either language is
[0014](0014-a-language-is-threaded-into-every-screen-and-the-machines-comes-through-the-seam.md), and what
thcrap's presence did to orb's own text is [0015](0015-orbs-own-text-leaves-through-gdi32s-exports.md).

## Context

**紅魔郷 in English is thcrap, and thcrap is a launcher.** It patches a game by injecting into it, and its
configuration tool installs a wrapper exe per patch stack in the game's directory —
`th06 (en).exe` — which runs `thcrap_loader.exe` with a run configuration and a game id. orb is a launcher
too, for the same reason: it creates the process suspended so that its DLL is in before the game's entry
point.

**So somebody who installs both has two launchers for one game**, and every arrangement they can make by
hand is bad:

- Start the game through thcrap and there is no orb in the process — no chapters, no retries, no frame
  pacing.
- Start it through orb and there is no translation.
- Edit thcrap's `games.js` to name `orb.exe` instead of the game, and thcrap injects into orb's launcher
  rather than into the game, which is not what either of them meant.

**And it is not a rare pair.** orb's screens are in English because the machine is (0014); a machine whose
Windows is in English is a machine whose 紅魔郷 is very likely under thcrap.

## Decision

**A launch finds the thcrap beside the game and does thcrap's work itself, through what thcrap publishes
about itself.** Nothing here reproduces a step of thcrap's own installation, and nothing asks the player
to arrange the two.

**Discovery is the wrapper exe's string resources.** Resource 0 is the `bin` thcrap runs from, 1 is the
command line, 2 is the exe it runs; resource 2 naming `thcrap_loader.exe` is what tells one of those
wrappers from the game's exe and from orb's, which carry no string resources at all. The patch stack's run
configuration and the game id come out of resource 1, so a directory with two stacks in it is read the same
way twice — the names sort, and a launch that picked a different stack each time would be a translation
that changed by itself.

**And the game's directory is the only place asked.** thcrap's wizard also writes a shortcut instead of a
wrapper exe, and to the desktop or the start menu instead of beside the game, and none of those is looked
for: a `.lnk` would be `IShellLink` to resolve, the desktop and the start menu are folders holding every
game's launcher rather than this one's, and an installation that left nothing beside the game would need a
path in `orb.yaml` — where a path is the one thing that file holds none of, since it is written on the
machine it is read on. What is beside the game is what somebody installed to play *this* game with, which
is the question a launch is asking, and it is the arrangement the wizard offers first.

**The injection is two of thcrap's exports, in the order thcrap's own loader calls them.**
`WaitUntilEntryPoint` runs the suspended process to its entry point and stops it there;
`thcrap_inject_into_running` then takes the process and the run configuration. Both are cdecl.
Measured, on 紅魔郷 1.02h with thcrap 2025-12-02:

- Without `WaitUntilEntryPoint`, a process created suspended and handed straight to
  `thcrap_inject_into_running` ends up with thcrap's DLLs in it and no window ever created. Its export list
  carries the reason in a comment of its own — "Yes, these are necessary for injection chaining".
- Declared `extern "system"` — stdcall on this target — the stack comes back short by the arguments and orb
  dies a few instructions later with the game standing at its entry point. `THCRAP_API` is
  `__declspec(dllexport)` and names no convention, and thcrap's own C# bindings declare every one of these
  `CallingConvention.Cdecl`.

**orb's DLL goes in second**, because both rewrite `CreateWindowExA` in the game's import table and the
second one there owns it: orb's rewrite calls through to whatever was in the entry and thcrap's does not.
Measured as a launch with `screen: fullscreen` that came up in the game's own 640x480 with orb's letterbox
holding no client, which is what orb-first looks like.

**The update is thcrap's own sequence**, `runconfig_thcrap_dir_set` and the run configuration and then
`stack_update_wrapper` under each of thcrap's two filters, with `update_at_exit` deciding whether it
happens before the game starts or after it is running. Two things measured here:

- Without `runconfig_thcrap_dir_set` the updater resolves no repository and fetches nothing.
- The wrapper in `thcrap.dll` loads `thcrap_update.dll` by name, which searches orb's directory rather than
  thcrap's, and answers with a fallback that updates nothing when it cannot find it: a launch fetched
  neither of two files deleted by hand. orb loads that DLL itself first, with `LOAD_WITH_ALTERED_SEARCH_PATH`,
  so the module is already in the process under that name when the wrapper looks.

**Nothing is passed where thcrap's progress callback goes.** A `stack_update` given one corrupted the
process's heap: first exception `0xc0000374`, then the same access violation 477 times and a stack overflow
to finish, on two runs out of two that had files to fetch. The same two calls with nothing there fetched the
same thirty files and came back, three more launches with nothing to fetch came back, and the launch after
them had the game up in English and fullscreen at 60.00fps with orb's own `INPUT LAG 3.3ms`. The callback
was only ever there to count files for the line a launch prints, so the line says the stack was brought up
to date and no longer says how much of it moved.

**A failure anywhere in this is not a failure to launch.** The game is orb's and the words in it are
thcrap's: a patch that cannot be found, loaded or injected leaves somebody playing the game they had before
installing either, and the line the launcher prints says which step it was.

## What follows from it

**orb calls into a DLL that was written to be injected into a game, from its own process.** That is the
shape of the thing and not a detail: `thcrap.dll` is loaded into `orb.exe`, its updater runs there, and its
own `log_init` is never called, so its log goes nowhere for the duration. The heap corruption above is what
that costs when one of the paths through it is exercised in a process it did not expect, and the answer was
to stop exercising that path rather than to fix thcrap from the outside.

**A thcrap dialog can appear over orb's launch.** thcrap reports a patch it cannot find with a message box,
and with the update running inside orb's process that box is orb's launch waiting on somebody's `OK`. Seen
once, from a patch directory removed by hand.

**Somebody whose thcrap is installed the other ways plays in Japanese and is told so** — `none installed
where the game is`, which is the line for a search that had one place to look — and has nothing to answer:
`thcrap:` is a `bool` because one place to look is all a switch has to turn on. Moving one of the wizard's
wrapper exes into the game's directory is what makes such an installation orb's, and it is what the wizard
would have written there itself.
