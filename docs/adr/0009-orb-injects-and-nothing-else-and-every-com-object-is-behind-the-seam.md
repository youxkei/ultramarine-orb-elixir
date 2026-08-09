# 9. `orb` injects and nothing else, every COM object orb calls is behind the seam, and the scenarios drive `orb-core` from a crate of their own

**Status:** accepted and built, and its title made true by
[0010](0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md). Everything that
decides what happens to a run is `orb-core`, whose `runtime.rs` holds the eleven hook bodies, `Runtime`
and `Originals`; `cargo xtask seam` checks it and `orb-sim` for a host with no Windows, so a
`windows-sys` import there fails to compile. Direct3D, DirectSound and the GDI's glyphs are behind the
seam — eighteen slots, eight slots and four calls — and the 137 scenarios are `#[cfg(test)]` modules of
`crates/orb-e2e/src/`. 356 tests pass, which is the 347 this was written against plus four the declared
metric brought and five 0010 added.

**What this document built and 0010 finished.** As built, `crates/orb` was nine files and 2728 lines and
named `windows-sys` in every one of them — which is true and is the weaker claim than the title's: it says
the boundary is where Windows is rather than where the patching is. Step 6 below splits the five mixed
files "along the line between a hook and an arithmetic", and names the joystick's halves as the import
entry *and the sampling thread*, which is a different line from the one this document's first section draws.
So a hook body that needed Windows and patched nothing came out on `orb`'s side: `joystick.rs`'s sampling
thread and `window.rs`'s status line, about 250 and 330 lines, with the heap walk a third of the same kind
reached through a handover. 0010 moves all three and `crates/orb` is nine files and 1346 lines. The rule
this document sets held throughout and `cargo xtask seam` holds it; what did not hold until 0010 is that
everything a scenario *ought* to be able to drive is above the seam.

**Six things the building found, each of which corrects something below.**

1. **The steps were done 0, 1, 2, then the glyph seam, then the drawing seam**, which is *What follows from
   it*'s 4 before its 3. Step 3 as written cannot be done first: it deletes `Screen`, and `says` has no
   baker until the glyph seam has landed — `Recording` in `orb-sim` cannot rasterise. Swapping them costs
   nothing and throws nothing away, because with the bake behind the seam both sides of `says`'s pixel
   comparison come from the same simulated host and are equal by construction; the string the mask carries
   then replaces the comparison in the step after.
2. **The drawing seam is eighteen slots and not fifteen.** Fifteen is what `overlay.rs` calls. `clear` was
   already `orb-core`'s — `Th06::set_play_viewport` and `Th06::prepare_frame` call it, at
   `game/th06/mod.rs:1214` and `:1259` — and `begin_scene` and `end_scene` became `orb-core`'s with the
   frame loop. The honest count is *every slot `d3d8.rs` types*, that file having only ever typed what gets
   called, and it is asserted where the trait declares them.
3. **Four of the `*_ui` modules drew through `Screen`, not five.** `menu_ui` names only `Frame` and `Label`
   and has no fixture of its own, so the four are `lives_ui`, `mode_ui`, `resume_ui` and `retry_ui`.
4. **`orb-e2e` does not stop naming `orb` and the `rlib` stays — because of step 6 and not because of the
   design.** Step 6 leaves the rewrite of each patched import in `orb` —
   `window::create_window_ex_a`, `score::create_file_a`, `joystick::answer` — and a game laid out by hand
   has no import table, so it *calls* those where a real launch has them patched in. `orb::attach_to` stays
   with them, being an install list. What the fake stopped naming is the eleven hook bodies, which are
   `orb_core::runtime`'s.

   **[0010](0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md) has undone
   this one, so it was true of step 6 and not of the design**: step 8's prediction was right, and wrong
   only about which step would make it so. Those three are hook bodies, and this document already settled
   where a hook body lives — `run_calc_chain` is `orb_core::runtime`'s and the install list takes its
   address. `attach_to` went to `orb_core::runtime` beside `detached`. The one thing that really had to stay
   was the `Present` slot's `VirtualProtect`, and 0010 put that behind `mem::replace_word`, which took the
   last handover with it. `crates/orb-e2e` names no `orb` and `crates/orb/Cargo.toml` builds a `cdylib`
   only.
5. **Two handovers the other way, not three.** `save_replay` and `get_controller_input` turned out to need
   none, and `render`'s bail-outs reach the seam's own `window::foreground`. What was handed over is the
   device's `Present` slot and the lines written in the black beside the game — `runtime::Patches`, set by
   both attaches. The region walk was a third of the same kind, in `memtrack`: the arithmetic was
   `orb-core`'s and `HeapWalk` was `orb`'s, so `orb::memtrack::install` handed the walk over as it patched
   the imports it walked the results of.

   **All three are gone with 0010**, and there is no handover the other way at all: the bar's GDI and the
   walk went to `orb-api`'s `real`, the slot's swap to `mem::replace_word`, and `Patches`,
   `hands_over_the_patches` and `hands_over_the_walk` with them.
6. **`orb::detached` did not close the log, and that was a defect.** A real launch closes it from
   `DllMain`'s `DLL_PROCESS_DETACH`; `detached` is the fake's game-closing and the only way out of a
   scenario. Left open, the next `log::line` wrote through a handle onto a game that had gone — and where
   the next thing along was another game in the same process, that write landed in *its* log before it had
   opened one, three counter reads at a time. Found by `pacing::counters`'s two runs coming out one
   microsecond apart.

It stands on
[0005](0005-every-scenario-lives-in-orb-sims-tests.md), which merged the two `tests/` directories and
chose `orb-sim` for a reason this replaces, and on
[0008](0008-the-fake-game-copies-the-game-orb-is-injected-into.md), which is why the fake game is worth
this much care in the first place. It overturns part of
[0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) — `Originals` becomes `orb-core`'s,
and the exception 0002 records stops being one — takes [0003](0003-the-frame-loops-scenarios-are-one-file.md)'s
reason for existing away, and replaces `recording.rs`'s own claim that the drawing needs no seam.

## Context

**The boundary between `orb` and `orb-core` is stated in terms of a property nothing checks.**
`orb-core/src/lib.rs` opens with *orb's logic, with no Windows in it*, and `crates/orb-core/Cargo.toml`
gives the reason:

> Everything it asks of the host goes through `orb-api`, which is what lets it be built and tested on a
> Linux runner — and a build that stops working there is a build that has taken a dependency on the host,
> which is worth finding out about when it happens rather than at the end of the work.

`.github/workflows/ci.yml` has one job. `runs-on: windows-latest`, target `i686-pc-windows-gnu`, and the
three steps are `cargo fmt --check`, `cargo xtask clippy` and `cargo xtask test` — all of which name that
target. **Nothing in this repository builds `orb-core` for a host that is not Windows.** The tripwire the
manifest describes is not armed, so the rule it is there to enforce is kept by hand.

**And the rule does not describe the tree.** Of `orb`'s twenty-one files, eleven name `windows_sys`:
`pe`, `score`, `joystick`, `recording`, `lib`, `window`, `hook`, `threads`, `text`, `crash`, `memtrack`.
The other ten name none — `chapter`, `snapshot`, `resume`, `tuning`, `overlay` and the five `*_ui` — so
whatever puts those on `orb`'s side of the line, it is not the line. Following their imports, **two leaves
hold all ten up**: `memtrack.rs`, which hooks the exe's imports for the heap, and `text.rs`, which
rasterises glyphs through GDI. Everything else they reach is `orb-core`'s already, or `orb_api`'s.

**What the rule should have said.** The property worth having is not that `orb-core` builds on Linux. It
is that **the code a scenario drives cannot reach Windows except through the seam** — because that is the
whole of what makes a scenario evidence about a launch. `attach_to` fills the same statics `hook::install`
fills, so orb's own code is identical in both and the only thing that can differ is what lies past a
hook; a scenario is then a launch with the far side replaced. Today that property is false of the near
side: `run_calc_chain`, `get_input`, `stage_begun` and the eight beside them live in `orb`, which may name
`windows_sys` in any function. A hook body that grew a direct `GetAsyncKeyState` would make every scenario
over it diverge from a launch, and nothing in the workspace would say so.

**And `windows_sys` is not the test the rule can be operated by, which is the hole this decision closes
second.** `overlay.rs` names `windows_sys` nowhere and calls **fifteen slots of `IDirect3DDevice8`** —
`create_texture`, `lock_rect`, `unlock_rect`, `release`, `create_state_block`, `capture_state_block`,
`apply_state_block`, `delete_state_block`, `set_render_state`, `set_texture_stage_state`, `set_texture`,
`set_vertex_shader`, `set_viewport`, `get_viewport` and `draw_primitive_up`. `audio.rs`, already in
`orb-core`, calls **eight of `IDirectSoundBuffer`**. Every one of those is d3d8.dll's or dsound.dll's code,
reached through a pointer the game handed over rather than through a crate a grep would find. So a rule kept
by looking for `windows_sys` passes a file that is calling Windows fifteen times.

`recording.rs` argues that no seam is needed there, and the argument is weaker than it reads:

> No seam and no `#[cfg(test)]` branch anywhere in the drawing code, because none is needed: a `Device` is a
> pointer to a vtable, so a vtable of Rust functions *is* a device as far as everything that calls one is
> concerned.

That is why the *tests* work, and it is not why the seam is unnecessary. The same sentence would retire
`orb_api::mem` — memory is an address, so a test could hand addresses over and read them — and what `mem`
buys is not the ability to answer but that `orb-core` cannot reach past it. A COM vtable is an interface
already; what it is not is *this repository's* interface, and what the rule is about is which crate may name
the far side of one.

**The scenarios live in the simulator's `tests/`, and 0005 gives one reason:**

> **`orb-sim` is where the thing every one of them installs lives.** A scenario's first act is to put a
> `Sim` in front of `orb_api` and its last is to drop it.

Every consumer of a library installs something of it, and that is not a reason to host that library's
users' tests. 0005 weighed three alternatives — a `tests/scenario/` subdirectory, two-letter names in
place of the word, and renaming without moving — and a crate of their own was not among them.

**What their being integration tests of `orb-sim` costs is paid twice, and both payments are written
down.** `fake/mod.rs` carries a blanket allow over the whole of the fake game:

> Shared by every `scenario_*.rs` beside this module, and each drives the part of a game it is about — so
> what one file does not touch is not dead code, it is another file's. Nothing can see that: `dead_code`
> is worked out per binary, and this module is compiled into one per `scenario_*.rs`.
>
> `#![allow(dead_code)]`

Twenty-three binaries, so three thousand lines of fake game compiled twenty-three times and dead code in
it invisible. And 0003 put sixty-three scenarios in one file for the same reason, in its own words:

> One file rather than twelve is `fake` compiled once instead of twelve times, and the judging below with
> no `dead_code` allow over it: a helper nothing calls reads as dead, which twelve binaries could not see.

So the tree already pays a design cost — one file holding sixty-three scenarios over one subject — to buy
back what a crate would give for nothing.

**And the cycle.** `orb-sim --dev--> orb --> orb-core --(sim)--> orb-sim` closes, which cargo takes
because a dev-dependency edge is outside the normal build graph. 0005 records what it leaves behind:
*`cargo test -p orb-sim` now builds the `orb` crate, so the simulator's own four tests wait on it.*

## Before starting

**Measured while this was written, because three of them settle a step and one of them corrected it.** The
numbers are stale the moment the tree moves; the commands are not.

**`orb-core` does not build for a 64-bit non-Windows host, and the manifest's claim that it does is
false rather than merely unchecked.**

```sh
$ cargo check -p orb-core --target x86_64-unknown-linux-gnu
error[E0570]: "thiscall" is not a supported ABI for the current target
    --> crates/orb-core/src/game/th06/mod.rs:1977:32
error: could not compile `orb-core` (lib) due to 3 previous errors
```

The three are the `handed_over!` calls, which transmute to `extern "thiscall"` — an ABI that exists on x86
and nowhere else. **On a 32-bit Linux target it builds**, and that target is already installed:

```sh
$ cargo check -p orb-core --target i686-unknown-linux-gnu
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.64s
```

So the tripwire step below names `i686-unknown-linux-gnu`. A step naming a 64-bit one would fail on
arrival and read as this decision being wrong about the boundary, when what it would be is wrong about the
ABI.

**`lib.rs` reaches all five of the mixed files**, which is what fixes the order of the last two steps:
`window::` nine times, `score::` nine, `joystick::` five, `memtrack::` and `threads::` once each. The hook
bodies cannot move to a crate the arithmetic they call has not reached yet.

**The four tests no game drives name no `orb`**, so `crates/orb-sim/Cargo.toml` can lose its
dev-dependency at the `orb-e2e` step rather than at the last one:

```sh
$ grep -l 'orb::' crates/orb-sim/tests/log_*.rs crates/orb-sim/tests/pacing_no_timer.rs
```

**The declared metric reaches three assertions, and that is the whole of what the glyph seam has to get
right.** `Recording::says` is called from 46 places in `crates/orb-sim/tests/`. Twenty-six read nothing but
whether the label is there — `.len()` or `is_empty()`. Ten read `Quad::color`, which a metric cannot move.
**Three read the geometry**, and all three are the same claim: `DISABLE` overlapping `Th06::lives_row` —
`scenario_the_mark_over_the_lives.rs:148`, `scenario_pointdevice_run.rs:154` and
`scenario_legacy_run.rs:138`. So the metric has to make that word the size that overlaps that row, and
nothing else in the suite has an opinion about how wide a string is.

**The two seams are fifteen slots and eight**, which is what says a mirror of them is a trait somebody can
read and an abstraction over them is a temptation. Counted off the two files with `grep -oE 'vtable\)?\.[a-z_]+'`
over `crates/orb/src/overlay.rs` and `crates/orb-core/src/audio.rs`.

**One thing to try before the crate is built out.** `in_its_own_process` spawns `current_exe` with
`--exact` and the current thread's name, which for a `#[cfg(test)]` module is a module path rather than a
bare function name. The whole of the `orb-e2e` shape rests on that still selecting one test, and it has not
been tried. It is a five-line experiment and it goes first, because a `no` there changes the shape rather
than the plan.

**And the suite has one known intermittent**, so *green* below means green or that one: a scenario of
`scenario_the_music_across_a_restore.rs` fails about one run in eight with `no sound has been installed on
this thread`, reproduced at `HEAD` before any of this work. `TODO.md` carries what is known about it. A
step that fails only that way has not broken anything.

## Decision

**Three parts. They are separable, and the order they are done in matters** — see *What follows from it*.

### `orb` injects `orb-core` into the exe, and does nothing else

What stays is what has no meaning without a process to patch: `DllMain`, `hook` (the trampolines, the
import table, the vtable slots), `pe` (the headers those are read out of), `crash` (the handler that names
a module and an offset), and the install lists in `lib.rs` — which prologue goes with which hook, and
which of them a `Config` asks for.

What moves to `orb-core` is everything that decides what happens to a run: the ten files that already name
no `windows_sys`, and out of `lib.rs` the **eleven hook bodies, `Runtime` and `Originals`**. A hook body is
logic — it reads a `State`, asks `chapter` or `resume` for a decision, and calls through a function pointer
out of a static. None of that is Windows, and it is precisely what a scenario drives.

Five files are a hook and an arithmetic sharing a file, and each is split along that line: `window` (the
rewrite of `CreateWindowExA`'s arguments and the letterbox rectangle, against the hooks that reach them),
`score` (the fork's choice of name, whose arithmetic already has its own tests, against the `CreateFileA`
hook), `joystick` (what a sample means, against the import entry and the sampling thread), `memtrack` (the
set of regions the game owns, against the heap import hooks) and `threads` (which threads are the game's,
against the suspend and resume that `orb_api::Win` already answers).

**Three of the eleven hook bodies do injection themselves, and those need a handover the other way.**
`init_d3d_device` calls `window::hook_device`, which is `hook::replace_pointer` over the device's `Present`
slot; `save_replay` and `get_controller_input` are pure, but `render`'s own bail-outs and the window
rewrite reach the same file. A body that patches memory cannot live in `orb-core`, and the answer is the
shape `orb-core` already uses for the three calls it makes *out* into the game: `orb` writes the address of
its own *patch this slot* into a static behind the same `cfg(any(test, feature = "sim"))` the
`handed_over!` macro puts one behind, and `orb-core` calls it. `Originals` carries game → `orb-core`; this
carries `orb-core` → `orb`, and it is the one direction the split adds.

`Originals` becomes `orb-core`'s. 0002 records that three of the calls handed over had already crossed
that way — `Chain::Cut`, `SoundPlayer::StopBGM` and `Supervisor::PlayAudio`, kept in `orb-core` behind the
`handed_over!` macro — and calls it *the one place the list has crossed out of `orb` into `orb-core`*.
Under this decision it is not an exception: the whole list is `orb-core`'s, and what distinguished the
three was only that they are calls *out* rather than hooks, which is not a difference in where they belong.

### Every COM object orb calls goes behind the seam, and the seam is a mirror rather than an abstraction

`orb-core` reaches Direct3D and DirectSound by dereferencing a pointer the game handed over and calling a
vtable. That is Windows, and it is Windows a grep for `windows_sys` does not find. So `orb_api::Win` gains
the slots those two calls go through — **fifteen of `IDirect3DDevice8`, eight of `IDirectSoundBuffer`** — with
`orb-api`'s `real` holding the vtable calls and `orb-sim` answering them.

**A mirror of the slots and not an abstraction over them**, which is the whole of the design and the one way
it can be got wrong. `orb_api::mem` is `read`, `write` and `fill` — the operations themselves, with no opinion
about memory — and the drawing seam is the same: `set_render_state`, `capture_state_block`,
`draw_primitive_up` and the twelve beside them, taking what the vtable takes. What must not cross is a
*decision*. `overlay.rs` opens with the reason:

> Every draw is bracketed by a state block capture and apply. The game sets render states once and assumes
> they stay set, so leaving so much as the vertex shader changed shows up as the whole scene drawing wrong.

Which render states, how the bracket is made, the FVF and the vertex layout are `orb-core`'s and are what a
scenario over the drawing is about. A seam that said *draw this text here* would take all of it below the
line into `orb-api`'s `real`, where no scenario reaches — and the failure it exists to prevent is a scene of
the game's own drawing wrong, which no test would then be able to see.

**`d3d8.rs` goes with the seam rather than into a crate of its own.** Its 178 lines are `#[repr(C)]`
declarations and constants and no logic, and once the only code that calls a vtable is `orb-api`'s `real`,
`orb-core` names none of them. `Game::d3d_device` answers a handle the way `Win::open_log` answers a
`LogFile`. And `orb-sim/src/sound.rs`'s `mod slot` — eight slot numbers written out again because the
simulator cannot name `orb_core::audio`'s layout — goes with it, which is the duplication this arrangement
has already been paying for.

**Glyph rasterisation is the third of those objects and not a part of its own.** `text.rs` loads the game's
`font.ttf` process-private and bakes a coverage mask through GDI; `Win` gains *bake this string at this
height*, `orb-api`'s `real` keeps the GDI, and `orb-sim` answers by **recording the string it was asked for
and returning a mask built from a metric a scenario declares**. Two heights, `FONT_HEIGHT` and
`MARK_FONT_HEIGHT`, so the ask names which; a mask and its own dimensions, because those are what the quad
is sized from; and `Font::load`'s `AddFontResourceExW` goes with the baking or the simulator is left holding
a font it cannot open.

**A declared metric rather than the real rasteriser, and the reason is that the real one is already not the
game's.** `fake::scratch` copies Windows' own `arial.ttf` into the scratch directory under the name the
game's font is installed as, and says why:

> Windows' own, under the name the game's is installed as — GDI substituting a face is something
> `Font::load` already survives, and a path that is not a font is not.

So the pixels a scenario matches against today are Arial's, not 紅魔郷's, and *keeping them real* buys a
fidelity that was never there. A metric a scenario declares is the shape everything else `orb-sim` answers
with already has — `Panel::measured()`'s two sizes, `Work::flat`'s microseconds, `Display::agreed`'s hertz —
and it takes away the one thing those exist to take away: an answer that depends on which fonts the machine
happens to have. It also retires what `Screen`'s own doc records as the cost of the substituted face, *a test
may ask where the drawing put something and not how wide it came out*, since a declared metric is a width a
test may ask about.

**And it takes `Recording` and `Screen` with it.** `Recording` becomes `orb-sim`'s implementation of the
drawing seam rather than a device of its own, and belongs there: it names `orb-api`'s declarations and
nothing of `orb-core`, so the cycle that would have closed on `orb-core --(sim)--> orb-sim --> orb-core`
does not arise. `Quad` and `Drawn` go with it as what the implementation records.

**`Screen` is deleted.** It exists to hold an `Overlay` and the device it was built on together, because
`says` had to bake a string a second time through the same font at the same size to recognise it — which is
why a scenario carries *two* overlays on one device, orb's own that draws and `Screen`'s that never draws.
With the simulator recording the string it baked and the quad it was then asked to draw with that mask,
`says` asks it. No second overlay, no bake held against itself, and no matching by pixel equality that
works because two strings do not happen to rasterise alike. What `Screen::writes` did — the game drawing a
screen of its own — is the fake asking the seam for a mask and drawing it, the same way `Overlay` does.

What replaces `Screen` for `orb`'s own unit tests of the drawing is what every other test of `orb-core`
already does: install a `Sim`, build an `Overlay` through it, draw, and ask the simulator what it was asked
for. `Screen`'s three traps go with it — the drop order, one device per test, and the substituted face — none
of which a `Sim` has.

### `orb-e2e` holds the fakes and the scenarios

A crate whose `src/` is the fake games — `mod.rs`'s half of any launch, `th06.rs`, `th07.rs` — and the
recording device beside them, compiled **once**, with the scenarios as `#[cfg(test)]` modules over it.

`#[cfg(test)]` is available here where it is not in `orb` or `orb-core`, and that is the whole of why the
crate is worth making. 0001 and 0005 both give the reason the scenarios had to be integration tests: *a
crate under test cannot turn a feature on for itself*, so `orb-core`'s `sim` feature — which is how
`game::th06::image` and the seam's install point are reached — cannot be enabled by `orb-core`'s own
`#[cfg(test)]`. `orb-e2e` is not under test. It is a consumer, so it depends on
`orb-core = { features = ["sim"] }` the ordinary way, and its own `#[cfg(test)]` is true.

**The fake is `pub(crate)`, and that is the condition on the whole thing.** `dead_code` does not fire on
a `pub` item of a library, so a fake made `pub` to be reachable from `tests/` would delete the blanket
allow and find nothing — the allow would go and what it was hiding would stay. The scenarios being
`#[cfg(test)]` modules of the same crate is what makes `pub(crate)` enough, and the two go together: either
both, or neither buys the check back.

`orb-e2e` **may name `windows_sys`**, and that is not a hole: the rule is about `orb-core` and `orb-sim`,
which are the code under test and the host it is tested against. A game that plays the game's part is
allowed to be a real object where a laid-out one will not do — which `orb_sim::Sound` and the recording
device already are.

**The four tests no game drives stay in `crates/orb-sim/tests/`**: `log_writes.rs`, `log_off_thread.rs`,
`log_overflow.rs` and `pacing_no_timer.rs`, which drive `orb_core::log` and `frame::Pacing` with a `Sim`
installed and nothing of a game anywhere in them. That leaves two directories again, and this time along a
boundary that is one: *a game drives it* against *no game does*. 0005's complaint was a split along a
boundary that was not a boundary, and it is not made twice by drawing a real one.

The `scenario_` prefix goes with the move. It was there because two levels shared a directory; the crate
name says the level now.

**`recording.rs` is not `orb-e2e`'s, and this is why the seams come before the moves.** It becomes
`orb-sim`'s implementation of the drawing seam — see the part above — so it goes there and not here, and
`Screen` is deleted rather than moved. Until that step it stays in `orb`, which `orb-e2e` names anyway. Its
one use of `windows_sys`, a `GetWindowsDirectoryW` for the system font `Screen::new` bakes from, goes with
`Screen`. And the `#[cfg(test)]` tests of `mode_ui`, `resume_ui`, `retry_ui` and `lives_ui` **stay where
their files are**: what they need in place of a `Screen` is a `Sim`, which `orb-core` already dev-depends on.

## What was weighed and rejected

- **Arming the tripwire instead: a CI job that runs `cargo check -p orb-core --target
  x86_64-unknown-linux-gnu`.** Two lines, and it does check the property the manifest claims. It is worth
  having either way and is not a substitute: what matters is that the code a scenario drives cannot reach
  Windows, and a Linux check of `orb-core` says nothing at all about the eleven hook bodies in `orb`. It is
  in *What follows from it* as the first step, because it is cheap and it makes the boundary that arrives
  later self-enforcing.
- **Moving the ten seam-clean files down and leaving the hook bodies where they are.** Half the point.
  The hook bodies are what the fake game calls — `orb::get_input`, `orb::run_calc_chain`,
  `orb::stage_begun` — so leaving them in a crate that may name Windows leaves the hole exactly where it
  costs the most.
- **Putting the whole overlay behind the seam.** Written above: `Win::draw_text` takes the shadow, the
  shared bake, the geometry and the colour out of the suite with it.
- **Abstracting the device rather than mirroring it** — a `Win::draw_text`, or a seam that took a rectangle
  and a colour. It reads better and it moves the render-state bracketing, the FVF and the vertex layout into
  `orb-api`'s `real`, where nothing a scenario drives reaches them. The failure `overlay.rs`'s state block
  exists to prevent is the game's own scene drawing wrong, and below the seam no test could see it. Fifteen
  slots is a large trait; it is also the only one that keeps every decision above the line.
- **Extracting `d3d8`'s declarations into a crate both `orb-core` and `orb-sim` could name.** It was the
  answer while `orb-core` still called the vtable, and once only `orb-api`'s `real` does, `orb-core` names
  none of those types and the crate has one consumer. Fewer crates for the same property.
- **Keeping `recording.rs`'s position that no seam is needed for the drawing.** Its reason — a vtable of Rust
  functions is a device — is why the tests work rather than why the seam is unnecessary, and the same
  sentence would retire `orb_api::mem`. What it left behind is `overlay.rs` calling Windows fifteen times
  while naming no `windows_sys`, which is the rule passing a file it should have stopped.
- **Leaving DirectSound alone and seaming only the drawing.** `audio.rs` is already in `orb-core` and already
  calls eight slots of a COM object, so the hole is there whether the drawing is fixed or not — and
  `orb-sim/src/sound.rs` is already paying for it in eight slot numbers written out a second time.
- **`orb-sim` answering the glyph seam by delegating to the real GDI through `orb-api`.** It keeps the
  dimensions a real rasteriser's, which is the one thing a declared metric gives up. Rejected because the
  glyphs are Arial's already — see the `scratch` comment above — so what is kept is realism about a font
  the game does not ship, at the price of an answer that varies with the machine and of numbers nobody can
  read. It would also need `orb-api::real::text` made public purely so that a simulator which declares it
  names no `windows-sys` can reach Windows through the front door.
- **A `tests/scenario/` subdirectory, or renaming without moving.** 0005 weighed both and its reasons
  still hold; neither addresses the per-binary compilation, which is the cost this is about.
- **One `orb-e2e` per game.** `fake/mod.rs` is already the half of a launch that both games share, and
  `th07.rs` is one scenario. Two crates to keep in step for that is a manifest, not a boundary.
- **Leaving 0003's sixty-three scenarios in one file after the move.** Not rejected — freed. Splitting them
  is then a choice about what a file is for rather than a way of buying dead-code detection back, and 0003
  says so once its reason has gone.

## What follows from it

Ordered, and **each step ends with `cargo xtask test` green** — 347 tests at the time of writing, of which
133 are scenarios, and *green* meaning green or the one intermittent *Before starting* names.

**The order is structure, then seams, then moves**, and it is that way round because the seams rewrite call
sites in code that ships while the rest moves files that do not. Doing the structural move first means every
seam step is verified by the suite already in its final home; doing a seam first would mean verifying it
twice.

0. **The five-line experiment.** `in_its_own_process` under a `#[cfg(test)]` module, to see whether
   `--exact` still selects one test when the name is a module path. Nothing else can start until it
   answers, because a `no` there is a different shape rather than a different order.
1. **Arm the tripwire.** A CI step running `cargo check -p orb-core --target i686-unknown-linux-gnu`.
   Independent of everything below, and it fails loudly the day the rule is broken by hand. That target and
   not a 64-bit one, for the reason *Before starting* measures.
2. **`orb-e2e`, with the fake as its `src/`.** The crate, the workspace member, `fake/` moved in and made a
   library and `pub(crate)`, the scenarios moved to `#[cfg(test)]` modules, `#![allow(dead_code)]` deleted
   and whatever it was hiding dealt with. `crates/orb-sim/Cargo.toml` loses its dev-dependency on `orb`,
   which is the cycle gone. `recording.rs` stays in `orb` through this step — `orb-e2e` names `orb` until
   the last of the moves anyway, and step 3 is where that file stops being a device of its own.
3. **The drawing seam.** `Win` gains the fifteen slots, `d3d8.rs` moves to `orb-api` and its `real` holds
   the vtable calls, `Game::d3d_device` answers a handle, `overlay.rs`'s call sites are rewritten,
   `Recording` becomes `orb-sim`'s implementation and moves there with `Quad` and `Drawn`, `Screen` is
   deleted, and the drawing tests of `lives_ui`, `menu_ui`, `mode_ui`, `resume_ui` and `retry_ui` install a
   `Sim` in its place. **The riskiest step**: it is the shipped drawing path, and the thing to watch is that
   every render state and the state block bracket stay above the line.
4. **The glyph seam.** `Win` gains *bake this string at this height*, `orb-api`'s `real` keeps the GDI,
   `orb-sim` records the ask and answers from a declared metric, and `says` asks the simulator rather than
   baking a second time. Smaller than it looks once step 3 has landed, and *Before starting* names the three
   assertions the metric has to satisfy out of `says`'s 46 callers.
5. **The audio seam.** The eight slots of `IDirectSoundBuffer`, `audio.rs`'s call sites, and
   `orb-sim/src/sound.rs`'s `mod slot` deleted for the layout it can now name.
6. **The five mixed files split**, along the line named in the decision — `window`, `score`, `joystick`,
   `memtrack`, `threads`.
7. **The ten seam-clean files to `orb-core`.** **After the split and not before it**: `chapter.rs` and
   `snapshot.rs` both name `crate::memtrack`, whose region-set half only exists once step 6 has run, so
   moving the ten first would take two of them to a crate the file they call has not reached.
8. **`lib.rs` split.** The eleven hook bodies, `Runtime` and `Originals` to `orb-core`, with the handover
   the other way for the bodies that patch memory; `attach`, the install lists and `DllMain` stay. Also
   after step 6, and for the same kind of reason: `lib.rs` reaches every one of the five — nine references
   to `window::`, nine to `score::`, five to `joystick::`. `orb-e2e` stops naming `orb` here, and
   `crates/orb/Cargo.toml`'s `crate-type = ["cdylib", "rlib"]` can drop the `rlib` — its comment says the
   `rlib` is there so that the scenarios can drive this crate, and after this they do not.
9. **The documents.** `orb-core/src/lib.rs`'s own opening and `orb-core/Cargo.toml`'s reason rewritten to
   the rule this decision sets rather than the Linux one — and the manifest's *built and tested on a Linux
   runner* corrected, since it was not true of the target it implies. `recording.rs`'s own opening, which
   argues the drawing needs no seam. `SPEC.md`'s table of where each thing lives, and its *Running the game
   with no game there*. `TODO.md`'s counts of which crate holds what and which files still name
   `windows_sys`. And the statuses of 0002, 0003 and 0005, each carrying a pointer here, so that the story
   can be followed from either end.

**What it buys, and one thing it does not.** No test that cannot be written today passes because of the
moves: every file that moves is already driven by scenarios through `orb`'s `rlib`, and the fake game
already calls the hook bodies where the real game's code calls them. The seams are different — they make
`orb-core` unable to reach Windows at all rather than merely unlikely to, which is the property the whole
decision is for, and they take three things out of the tree on the way: the second overlay a scenario
carries so that `says` can bake a comparison, the eight slot numbers `orb-sim/src/sound.rs` writes out
because it cannot name the layout, and the note in `Screen`'s own doc that a test may ask where the drawing
put something and not how wide it came out. What also arrives is that dead code in three thousand lines of
fake game becomes visible, that `orb-sim` becomes what its own header says it is, and that `orb`'s name
stops being larger than what it does.

**What it costs, said plainly.** Two of the nine steps rewrite call sites in code that ships —
`overlay.rs`'s 479 lines and `audio.rs`'s 468 — and neither is a move that a compiler checks the way an
import rewrite is checked. That is a materially bigger piece of work than the file moves around it, and it
is the reason the order puts them where the suite is already in its final shape.
