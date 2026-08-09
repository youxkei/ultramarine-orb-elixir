# 10. `orb` is the patched bytes, and everything else has one of two other homes

**Status:** accepted and built. `crates/orb` is nine files and 1346 lines, from 2728: `DllMain` and the
install lists, `hook`, `pe`, `crash`, and one file per set of patched entries holding the write and nothing
else. **Nothing outside it names it** — `crates/orb-e2e` has no dependency on it and
`crates/orb/Cargo.toml` builds a `cdylib` only, which is what
[0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md)'s step 8 predicted and
its status recorded as not having happened. It did not happen because 0009's step 6 drew the line in the
wrong place; this is the same decision finished, and 0009's title is now true of the tree. 356 tests pass,
which is the 351 this was written against, the four *What it buys* names, and a fifth a launch asked for:
the status line's stack held off the game where the black is shorter than the stack is tall, which is the
shape a run plays with and the one a session watching the title menu does not reach.

**The five-line experiment answered *record and return*.** Both filters stayed green and neither got
slower — `pacing::` 15.1s against 15.1s, `orb-e2e` as a whole 16.3s against 16.4s — so a simulated host
writes the milliseconds down and returns at once. Which makes the count of them this machine's clock speed
rather than a cadence, so `orb_sim::Clock::sleeps` collapses a run of equal asks: what a scenario reads back
is *which* numbers were asked for, in order, and that is what a cadence is. The numbers are beside that
function rather than here, being the kind that goes stale.

**Eight things the building found, each of which corrects something below.**

1. **`push_merged` did not stay in `orb-core`.** *The walk goes to the seam's far side* leaves it there and
   the walk below the seam, and that cannot be: the walk is the only caller, and `real/mem.rs` may not name
   `orb-core`. Folding the seam's *answer* through it instead was tried and is wrong — merging is a rule
   about real pages, a heap region and a reservation being able to name the same ones, where two laid-out
   objects that abut are two objects. Measured: **26 of `orb-core`'s own tests** failed on
   `0x03000000 for 61440 bytes is not mapped in this space`, which is two of `Space`'s regions merged into
   one range nothing can read. So `push_merged` and its two tests went below the seam with the walk, and
   `Win::game_regions`'s own contract is that no two entries cover the same pages.
2. **The joystick's one test moved at step 4 and not step 11.** Step 11 says it goes with `calibrate`, and
   `calibrate` and `CALIBRATION` move at step 4 — so `a_calibration_write_stops_where_the_caps_do` had to go
   then, and `orb/src/joystick.rs` was left with no tests of its own eight steps earlier than step 11 says.
3. **`orb` keeps three unit tests and not two.** `create_file_a`'s refusal is `INVALID_HANDLE_VALUE`, which
   *Before starting*'s reading of `score.rs` classified as patched bytes and step 11 moves anyway. Written
   out above the seam it cannot be held against Windows' own by a `const` assert the way the two window
   styles and the two text alignments are: a raw pointer can be neither cast to an integer nor compared
   during const evaluation. So `orb/src/score.rs` has `the_refused_handle_is_windows_own`, and it is the
   one Windows number in the tree the compiler cannot be made to check.
4. **`orb-api`'s `windows-sys` features were missing `Win32_Globalization`,** and the whole-workspace build
   never said so: `orb` names that feature, and cargo unifies features across a workspace build. What found
   it is `cargo xtask test -p orb-core`, one package at a time, which is worth running after any step that
   adds a `windows-sys` import to `orb-api`.
5. **`measure_lines` answers two numbers and the layout uses one.** *The status line's layout goes above the
   seam* asks the seam for the widest of a stack *and one line's height*, which is what
   `GetTextExtentPoint32W` answers. `Bar::height` stays the em height it already was, and the line height
   comes back unused above the seam: `CreateFontW` with a null face name can map to a face whose `tmHeight`
   is not the height asked for, so stacking on the measured height would be a change in where the lines land
   hiding inside a move whose only witness is a launch.
6. **`orb::window::install` hands over last, which is a change in what a failed install does.** With
   `settle` in `orb-core` and reached through `install_over`, the two are one call — so an install that
   cannot find one of the two import entries now settles nothing, where before it had already called
   `SetProcessDPIAware` and written the ratio down. That is the better answer: what `settle` says is said
   because orb is laying the window out, and a launch whose imports are not there is not.
7. **`orb-e2e` stopped naming `windows-sys` at step 10, and its dev-dependency went with the page.** *`mem`
   gains the one call that stands in the way* predicts the page, its loop and its hazard note going; the
   dependency was the fourth thing that went, that page having been the crate's only use of it.
8. **The bar's record is a list of stacks written and not a picture.** `orb_sim::Written` is the window, the
   `Bar` and the lines, which is what the four assertions *What it buys* names read: `bar.height` for the
   height a stack got, `bar.align` and `bar.x` for which of the two bars, `bar.area` for where the block
   landed, and `bar.area` again on a shorter stack for the rows it clears. The cleared rows turned out to be
   the *last stack's own block* and not the widest ever painted — `PAINTED` holds one block — which is
   correct and is what the scenario now says.

## Context

**0009's title is not true of the tree 0009 built, and its own step 6 is why.** Its first section says
what stays in `orb` is *what has no meaning without a process to patch*. Its step 6 then splits the five
mixed files "along the line between a hook and an arithmetic", and names the joystick's halves as "what a
sample means, against the import entry **and the sampling thread**". Those are two different lines. A
hook body that needs Windows but patches nothing came out on `orb`'s side, deliberately, and there is a
lot of it.

**And 0009's own status records the weaker property in place of the one its title claims.** It says
`crates/orb` "names `windows-sys` in every one of them", which is true and is a different sentence: it
says the boundary is where Windows is, not where the patching is. The rule 0009 sets —
*the code a scenario drives cannot reach Windows except through the seam* — does hold, and
`cargo xtask seam` holds it. What does not hold is the other half of the same thought, which is that
everything a scenario *ought* to be able to drive is above the seam.

**What is missing is not reach — it is where the answers stop coming back**, and getting that the wrong way
round would send somebody looking for code a scenario cannot get to. A scenario gets to nearly all of
`orb`, because a game laid out by hand has no import table and so *calls* each rewrite where a real launch
has the entry patched. Seven doors:

```sh
$ grep -rhoE 'orb::[a-z_]+(::[a-z_]+)?' crates/orb-e2e/src | sort -u
```

Everything those reach runs. What happens at the far end is the thing.

`write_beside` runs on every frame of every scenario and gets as
far as `GetDC` of `Hwnd(0x1234)`, which is null, and returns; the sampling thread runs, loops, and calls a
real `Sleep` nobody writes down. So the layout above those calls is *executed and unreadable*: a scenario
cannot say how often the pad is sampled, what orb said about the device it found, which font height the bar
chose, where the block landed, or that a shorter stack of lines clears the rows the last one wrote in. What
moves the boundary is not reaching further but having the simulated host answer where a null handle answers
now.

**Measured.** Every `windows-sys` import left in `orb`, by file:

```sh
$ for f in crates/orb/src/*.rs; do echo "== $f"; awk '/^use windows_sys/,/;$/' $f; done
```

Classified by what its subject is:

| | |
| --- | --- |
| `hook.rs` | `VirtualAlloc`, `VirtualProtect`, `VirtualFree`, `FlushInstructionCache` — writing a jump over a prologue and swapping an entry. **Patched bytes.** |
| `pe.rs` | the DOS and NT headers, the section table, the import directory — where those bytes are. **Patched bytes.** |
| `lib.rs` | `DllMain`'s two reasons, `GetModuleHandleW`, `GetCommandLineW`, `GetCurrentProcessId`. **The injection itself.** |
| `crash.rs` | `SetUnhandledExceptionFilter` and `GetModuleHandleExW`. **A fault in the process orb was injected into**, reported as a module and an offset. |
| `memtrack.rs` | six import hooks — **patched bytes** — and `HeapLock`, `HeapWalk`, `VirtualQuery` over what they noticed, which is **not**. |
| `threads.rs` | one import hook. **Patched bytes.** |
| `score.rs` | one import hook, and `INVALID_HANDLE_VALUE` for the refusal. **Patched bytes.** |
| `window.rs` | two import hooks and the `Present` slot — **patched bytes** — `CreateSolidBrush` inside the rewrite of `RegisterClassA`, and twenty-odd GDI calls for the status line, which are **not**. |
| `joystick.rs` | one import hook — **patched bytes** — and `SetThreadPriority`, `Sleep` and `MultiByteToWideChar`, which are **not**. |

**Three calls and one conversion are the whole of what stands in the joystick's way.** Its 382 lines
reach Windows in four places: `hook::install_import`, which is the patch; `SetThreadPriority` with
`THREAD_PRIORITY_BELOW_NORMAL` and `Sleep`, which are the sampling thread's priority and its cadence; and
`MultiByteToWideChar` for one log line. Everything else — `poll`'s cadence and its
`ATTACHED_MS`/`DETACHED_MS`, the rule that it never waits less than the read itself took, `REPORT_READS`
and why an average must be reset, `describe`'s six arms, `calibrate`'s write through `orb_api::mem` and
the line about where two `JOYCAPSA` first differ, and `answer`'s decision — is logic that reaches the
host only through the seam already.

**The status line's way is the GDI, and its layout is not.** `write_beside` decides which of the two
bars — down the side or under the game — and where in it a stack of lines starts, out of `BAR_MARGIN`,
`BAR_TEXT_HEIGHT` and `BAR_TEXT_MIN`; `union` is what makes a shorter stack clear the rows the last one
wrote in; `report_no_bar` is the judgement that there is nowhere to write at all; `letterbox` caches
`fit`'s answer against the client it was worked out from. None of that is GDI. What is GDI is `paint`,
which clears, writes and blits in one operation so that a refresh cannot land between the clear and the
text, and `fitting_font`, which measures the widest line to choose a height.

**And the line already runs where a scenario can see the first of those and not the rest**, which is what
says where the boundary really is. `orb-e2e`'s `the_window` calls `letterbox` and asserts the rectangle to
the pixel — `(320, 0, 2240, 1440)` on a 16:9 client — and it calls `write_beside` twice and asserts
`report_no_bar`'s judgement both ways: that the `no black to write in` line does *not* appear where there
are 320 pixels of it, and that it appears with the client's own numbers in it where a 4:3 window leaves
none. So the judgement is covered. What is not is everything past `GetDC`, which in a scenario is a null
handle: the height `fitting_font` chose, where the block landed, and `union`'s clearing of the rows before
— which was a bug that was fixed and has never had a test.

**`memtrack`'s walk is the case that shows the rule has a third home.** 0009 left the walk in `orb` and
had `orb-core` reach it through `hands_over_the_walk` — a handover the other way, for code that patches
nothing. There is somewhere better: `orb-api/src/real/mem.rs` already holds `private_regions` and
`process_heap_regions`, which are the same `HeapWalk` and `VirtualQuery` over the same process, and
`Win::game_regions` already declares the answer. So the walk is not homeless; it was put in the wrong one
of the three that exist.

## Decision

**The line is *patched bytes*, and there are three homes rather than two.**

- **`orb`** is code whose subject is another module's memory: a jump written over a prologue, an
  import-table entry swapped, a vtable slot replaced, the PE that says where those are, the lists that say
  which goes with which, `DllMain`, and the filter that names a module and an offset when a fault
  happens in the process this was injected into.
- **`orb-api`'s `real`** is everything else that needs Windows. It is the seam's far side and it is
  already where twelve modules of exactly this kind live.
- **`orb-core`** is everything that needs no Windows, which is everything that decides what happens to a
  run.

**One exception, and it is named rather than left to be discovered: a hook body may make the one call its
rewrite consists of.** `register_class_a` swaps `hbrBackground` for a black brush, and that is a GDI call
inside a body that is otherwise arithmetic — so `CreateSolidBrush` stays with it in `orb`, and a seam
function for one call nothing else makes would be a worse tree.

**Which leaves nothing outside `orb` calling `orb`, and that is the test of the rule rather than a bonus.**
0009's step 8 said `orb-e2e` would stop naming `orb` and the `rlib` could go; its status records that this
did not happen, because step 6 left the rewrite of each patched import in `orb` and a laid-out game has to
call one. Under the line above those rewrites are not `orb`'s: `create_window_ex_a`'s body reaches Windows
only through `orb_api::window::primary_monitor`, `create_file_a`'s not at all, and `answer`'s not at all —
they are hook bodies, and **0009 already settled where a hook body lives**. `run_calc_chain` is
`orb_core::runtime`'s and `orb`'s install list takes its address; an import hook's body is not a different
kind of thing.

The three `install_over`s go the same way, and for the same reason: each one stores an address the game
handed over into the static its own hook body reads, which is what `attach_to` already does for the fifteen
slots of `Originals`. So `attach_to` and `detached` are `orb-core`'s too, and what is left in `orb` is
`DllMain`, `attach`, `install_hooks`, `hook`, `pe` and `crash` — none of which a scenario has ever reached,
`joystick::install`'s own doc saying so of itself: *the one thing in this module no scenario reaches*.

### `Rect` gains `#[repr(C)]`, before anything is moved

`Present`'s signature takes two `*const RECT`, and the replacement that goes in the slot has to have that
signature exactly. `orb_api::Rect` says of itself *the same four fields in the same order as Windows'
`RECT`* — and it is not `#[repr(C)]`, so today that sentence is a claim about a layout Rust does not
promise. Four `i32`s give a compiler no reason to reorder them and it does not, which is why nothing has
gone wrong; it is also why nothing would say if it did. So `Rect` gets the attribute, and below the seam a
`const` assert per field — `offset_of!(Rect, left) == offset_of!(RECT, left)` and the three beside it — the
way `DeviceVtable`'s eighteen slots are asserted and the two window styles are held against Windows' own. A
size assert would pass whatever order the fields came out in, which is the one thing being asked about.

**Only then** does the alias become `*const Rect`, `windows_rect` — which exists to convert one to the
other — go, and the slot's replacement stop naming a `windows-sys` type.

### `mem` gains the one call that stands in the way

**`mem::replace_word(address, value) -> Option<usize>`** — swaps the word at `address`, unprotecting the
page for as long as the write takes, and answers what was there.

It is the whole of what `hook::replace_pointer` does, and it is the last thing keeping a hook body in
`orb`: `init_d3d_device` decides to redirect the device's `Present`, which is `orb-core`'s decision, and the
swap needs `VirtualProtect` round it because the vtable's page is read-only. The read and the write are
`orb_api::mem::read` and `mem::write` already; only the protection is not.

**A page operation and not a decision**, which is why it belongs in `mem` beside `commit` — *puts back any
page that has gone since a snapshot, so a restore has somewhere to write* — which is the same kind of thing
said about the same pages. What stays above it is every judgement `hook_device` makes: whether a launch
wants a letterbox at all, that a second call must not patch twice, and the line that says a client of that
size is being presented into one.

**And it takes `runtime::Patches` with it.** With `hook_device` and `write_beside` both above the seam,
nothing is handed from `orb-core` back into `orb` — `Patches` and `hands_over_the_patches` go, and so does
`memtrack`'s `hands_over_the_walk`, which the walk's move takes. **The split adds no direction of its own.**

It also takes a hack out of the fake. `orb-e2e`'s `vtable_page` asks `VirtualAlloc` for a page at a named
address because `replace_pointer`'s `VirtualProtect` had to have a real one to work on; with the swap
answered by a simulated host out of its own address space, the vtable is laid out like everything else and
that page, its `VTABLE_TRIES` loop and the hazard note above it all go.

### `Win` gains the three the sampling thread needs

- **`clock::sleep(ms)`**, a coarse wait. Apart from `clock::wait(ticks)`, which exists already: that one
  is aimed at a frame's own deadline on a high-resolution timer, and the whole of what it is for is the
  input lag left to win. This one is the gap between two reads of a device, in whole milliseconds, on a
  thread nothing is waiting for.
- **`thread::below_normal()`** — `SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL)`,
  called by the thread on itself. Not a parameter of `thread::spawn`: the priority is set from inside the
  body because that is where the thread is, and `std::thread::Builder` has nowhere to say it.
- **`codepage::text(bytes) -> String`** — `MultiByteToWideChar` with `CP_ACP`, for a device name winmm
  gives in the machine's own code page. A simulated host answers the bytes as UTF-8, lossily, and that is
  not a gap: what a scenario asks of the line is which device it names, and `orb_sim::Joystick` is where
  the name it names was declared.

  **A module of its own, `crates/orb-api/src/codepage.rs`, and neither of the two it could have been put
  in.** `module.rs` opens *the modules loaded into the process orb is injected into* and this is not one;
  `text.rs` opens *the font a string is baked through* and this is not that either. One function in a file
  that says what it is beats a second subject in a file that already has one.

### The status line's layout goes above the seam and its GDI below

`Win` gains two, and they are coarser than the drawing seam's eighteen on purpose:

- **the widest of a stack of lines and one line's height, at an em height** — what `fitting_font` asks
  `GetTextExtentPoint32W` before it chooses a size.
- **a stack of lines written into a rectangle of the window** — the clear, the text and the blit, which
  have to be one call because a refresh landing between the clear and the text is a flicker somebody
  sees at 120Hz.

**`Bar` goes to `orb-api` and not to `orb-core`**, which is the one thing here that cannot be got wrong
quietly: it is `paint`'s argument, `paint` is `real`'s, and `real` may not name `orb-core`. So it joins
`Viewport` and `Locked` at `orb-api`'s root — the rectangle to repaint, the x the text is aligned from,
the bottom of the lowest line, one line's height, and which way it is aligned. Its alignment is
`TA_LEFT` or `TA_RIGHT`, so it is written out as a plain number above the seam and held against
Windows' own by a `const` assert below it, which is what `BORDERLESS_STYLE` and `WINDOWED_STYLE` already
do.

**Why coarser here, when the device's seam is a mirror of fifteen slots.** 0009 rejected a
`Win::draw_text` for the overlay because it would have taken the state-block bracket, the FVF and the
vertex layout below the line — and the failure that bracket exists to prevent is the *game's own scene*
drawing wrong, which no test below the seam could see. Here there is no such bracket. The black beside
the game is orb's alone, Direct3D never touches it, and the one failure below this line is a flicker,
which `paint`'s single `BitBlt` prevents and which no test could have seen either way. What a scenario
*can* have an opinion about is which bar the lines went in and where in it they started, and that is
exactly what stays above.

### The walk goes to the seam's far side, and the handover goes away

`Win` gains **`note_heap`**, **`note_reservation`** and **`forget_reservation`** — the same shape
`register_thread` already has, and for the same reason its doc gives: the noticing is orb's, being an
import hook, and what is done with what was noticed is the host's. `real/mem.rs` then answers
`game_regions` for itself, beside the two walks it already does, and `orb_core::memtrack`'s `Walk`,
`WALK` and `hands_over_the_walk` go.

**`mem::game_regions` stops being an `Option` when it does.** It answers `None` today for a build with no
simulated Windows in it — *there being nothing there to ask*, which is its own comment — and that `None`
is what `orb_core::memtrack::asked` branches on to reach the handover. With the real side answering, both
the `Option` and that branch go, and `regions` is the seam and nothing else.

With that and the two above, **`runtime::Patches` is one field rather than two**: `write_beside` is the
seam's now, and only the device's `Present` slot is still handed the other way.

## What was weighed and rejected

- **Leaving it and recording it.** The cheapest, and it is what the status of 0009 does today. Rejected
  because what it records is that a document's title is wrong about its own tree, and the four things a
  scenario cannot ask stay unaskable — which is the cost, not the untidiness.
- **Moving the sampling thread and leaving the status line.** Half the work for most of the benefit: the
  joystick's is 250 lines against the status line's 330, and the three seam calls it wants are far smaller
  than the two the bar wants. Rejected because it leaves the rule stated and broken in the same tree,
  which is the state this decision exists to end — and because *nothing has ever tested the status line*,
  which makes it the half where a scenario is worth more.
- **A `Win::status_line(lines)` that took the whole thing.** It reads better and it is the mistake 0009
  named: `BAR_TEXT_MIN` is a judgement about what is still readable and `union` is a bug that was fixed,
  and neither would be reachable. What is below the line here is one blit and one measurement, and that is
  the most that can go there.
- **Moving `score::given`.** It walks the path the hook was handed, a byte at a time to `LIMIT`, and it
  names no Windows at all — so by the letter of the rule it could move. Left where it is: its subject is
  the hook's own argument, a raw pointer into the game's memory that the call brought with it, and moving
  it would mean that pointer crossing into `orb-core` to be walked and nothing else changing.
- **Keeping the walk in `orb` and keeping the handover.** It is where 0009 put it and it works. Rejected
  because `real/mem.rs` already walks that process's heaps twice for `private_regions` and
  `process_heap_regions`, `Win::game_regions` already promises this answer, and the handover exists only
  because the walk is on the wrong side — one fewer mechanism for the same behaviour.
- **A crate of its own for the injection**, leaving `orb` as the DLL and nothing else. The nine files are
  one subject and 2728 lines; a crate boundary through them would buy a manifest.

## Before starting

**The sampling thread is a real thread in a scenario, and every step below rests on that.** It is not
laid out and it is not stubbed: `answer` calls `start_polling`, which calls `orb_api::thread::spawn`, whose
whole reason for existing is that it carries the installed simulated Windows onto the thread it makes —
its own doc says what a plain `std::thread::spawn` there would cost, *the joystick poller reading this
machine's own winmm instead of the one a scenario laid out*. So the thread really runs, really loops, and
really calls `Sleep`, in the middle of a scenario. `crates/orb-e2e/src/mode_on_a_winmm_pad.rs` is where it
is driven.

**Two things go first, and the first of them is the last step tried early.** Take `orb` out of
`crates/orb-e2e/Cargo.toml`'s dev-dependencies and `rlib` out of `crates/orb/Cargo.toml`'s `crate-type`,
build, and write down every error:

```sh
$ cargo check -p orb-e2e --target i686-pc-windows-gnu --all-targets
```

**That list is the worklist, and it is worth more than the one in this document.** Everything below was
found by reading, and reading missed one — `create_window_ex_a`'s first line asks `is_game_class`, which
this document did not say moved until somebody looked. The compiler cannot miss one. So the list it prints
is what the steps below are checked against, and **an error naming something this document does not is the
document being wrong rather than the tree**. Put both lines back afterwards; the last step is this same
change kept.

**And the second, because a `no` there changes the shape.** `Win::sleep` under a
simulated host cannot advance the clock: a background thread moving the frame loop's own counter would
break every pacing scenario, each of which asserts about that counter to the microsecond. But a `sleep`
that records the ask and returns immediately leaves `poll` going round as fast as a core will let it for
the length of a scenario. So the question is which of those two a simulated host answers with, and it is a
five-line experiment — `Win::sleep` recording and returning, then:

```sh
$ cargo xtask test -p orb-e2e -- pacing::
$ cargo xtask test -p orb-e2e -- mode_on_a_winmm_pad
```

**Both have to stay green and neither may get materially slower**, which is the whole of the criterion:
the first is the clock the thread must not move, the second is the scenario the thread runs hardest in.
Green and no slower, take it — recording the ask is what makes the cadence assertable and no wait is
needed for that. Slower, and the simulated host waits the milliseconds it was asked for instead, and the
cadence is still asserted off the record rather than off the wait. **Neither, and the joystick's step is a
different shape** — the thread's own loop would have to be something a scenario steps rather than something
it starts, which is a bigger decision than this one and belongs in a document of its own.

**Where it stands, and the commands rather than the numbers**, which are stale the moment the tree moves:

```sh
$ wc -l crates/orb/src/*.rs
$ grep -c 'offset_of!' crates/orb-api/src/d3d8.rs crates/orb-api/src/dsound.rs
$ cargo xtask seam && cargo xtask test
```

**And the suite has one known intermittent**, so *green* below means green or that one: a scenario of
`orb-e2e`'s `the_music_across_a_restore` fails about one run in eight with `no sound has been installed on
this thread`. `TODO.md` carries what is known about it.

## What follows from it

Ordered, and **each step ends with `cargo xtask test` green**.

0. **The last step, tried and put back** — the survey in *Before starting*. What it prints is the worklist
   the steps below are held against, and it costs two lines of a manifest to get.
1. **The five-line experiment**, also in *Before starting*. Nothing else can start until it answers, and
   what it answers decides what `orb_sim`'s `sleep` does rather than whether the rest happens.
2. **`clock::sleep` and `thread::below_normal`.** The two seam functions, `orb-sim` answering them —
   recording the milliseconds asked for, so that a scenario can read the cadence back — and the sampling
   thread calling them where it calls `Sleep` and `SetThreadPriority` now. Nothing moves crates yet, so
   this step is the seam widening and the suite unchanged.
3. **`codepage::text`.** One function in `crates/orb-api/src/codepage.rs`, and `joystick::name` becomes a
   call to it.
4. **The joystick moves.** `poll`, `describe`, `calibrate`, `bytes_of` and the module doc — which is the
   measurement of the 8.7ms call, and so belongs with the thread that measurement is the reason for — to
   `orb-core/src/joystick.rs`, beside what a sample means, which is already there.

   **`answer` stays in `orb` and its decision does not, so say which side each thing it touches is on.**
   The entry is `pub unsafe extern "system" fn answer(device: u32, into: *mut JoyInfo) -> u32` and it stays
   whole: its address is what `install` writes into the import table, and **a game laid out by hand calls it
   too** — `crates/orb-e2e/src/fake/th06.rs` calls `orb::joystick::answer(JOYSTICK_0, &mut info)` where its
   own read would have gone through that entry, so the signature is a scenario's and must not change. What
   is left of it here is three lines: start the sampling if it is not started, ask `orb-core` for an answer,
   and where there is none call through the entry the import held.
   - `orb_core::joystick::sampling()` takes the `POLLING` compare-and-exchange and the
     `orb_api::thread::spawn` — the thread's body is `orb-core`'s now, so the thing that starts it is too.
   - `orb_core::joystick::answered(device, into) -> Option<u32>` takes the whole of the decision:
     `DEVICE`, `describes_a_sample`, `latest`, `is_a_pad` and `calibrate`, with `CALIBRATION` and the
     `REPORTED_DIFFERENCE` line moving with the last of those. `None` is *this is not a read a sample
     answers*, which is the arm that falls through.
   - **`ORIGINAL` stays in `orb`**, and it is the one that could be got wrong either way: it holds the
     address `install` took out of the import table, or the function `install_over` was handed. Both of
     those are `orb`'s, so the static is, and the fall-through is the entry's own last line.

   **The first step a scenario gains something from**: the cadence and the line that names the pad become
   things one can assert.
5. **The bar's two seam functions**, `orb-api`'s `real/window.rs` holding `paint` and `fitting_font`, and
   `orb-sim` recording the lines and the rectangle they went in. That file's own opening — *which real
   window is in front, what the real monitor measures, and the frame a real host puts round a client
   area* — gains the third thing it then does.
6. **The status line moves.** `write_beside`'s layout, `union`, `report_no_bar`, `BAR_MARGIN`,
   `BAR_TEXT_HEIGHT`, `BAR_TEXT_MIN`, `letterbox` and its cache, `client_area` and `client_size` — which
   are `orb_api::window::client_rect` with a `Hwnd` wrapped round it and so already above the seam — and
   `CONTENT`, `SCREEN` and `DESTINATION` with them, to `orb-core/src/window.rs`. `Bar` goes to `orb-api`,
   for the reason the decision gives. `runtime::Patches` loses `write_beside` and `runtime::write_status`
   calls `crate::window` instead.

   **`GAME_WINDOW` is the one static that has a writer on each side, and it moves.** The rewrite of
   `CreateWindowExA` stores the window it made and stays in `orb`; `letterbox` and `write_beside` read it
   and move. Which window the game got is a fact about the run rather than about the patch, so it goes with
   the readers — `orb_core::window::window_created(Hwnd)` is what the rewrite calls, and a scenario reading
   the letterbox back no longer needs a launch to have happened for the address to be there.

   **The riskiest step**, and the one to write a scenario *before* rather than after. It is the shipped
   path for everything orb says about itself, and `cargo xtask test` staying green says almost nothing
   about it: the suite runs `write_beside` on every frame and it returns at `GetDC` of a handle that is
   not a window, so the 330 lines being moved are green today whatever is done to them. The scenario that
   makes the step verifiable is the one the move makes possible — which font height a stack of lines got,
   which of the two bars it went in, where the block landed, and a shorter stack afterwards clearing the
   rows the longer one wrote in.
7. **`note_heap`, `note_reservation` and `forget_reservation`**, `real/mem.rs` answering `game_regions`
   out of them, and `orb::memtrack`'s `walk`, `collect_heap`, `collect_committed`, `is_readable` and
   `TRACKED` moving there. `orb_core::memtrack` loses `Walk`, `WALK` and `hands_over_the_walk`; what is
   left of it is `Region`, `push_merged` and `regions` asking the seam.
8. **`mem::replace_word`**, `real/mem.rs` holding the `VirtualProtect` either side of the write, and
   `orb-sim` answering it out of its own address space. `hook::replace_pointer` then has one caller,
   `hook_device`, which step 10 moves — and none after it, so it goes. That it has exactly that one caller
   is what makes this safe to say:

   ```sh
   $ grep -rn 'replace_pointer(' crates/ --include='*.rs'
   ```

   `hook::install_import` writes its slot itself and does not go through it, which is why `hook.rs`'s two
   tests survive `replace_pointer` going.
9. **`Rect` gains `#[repr(C)]`** and the four `offset_of!` asserts against Windows' own go into `orb`,
   which is the last place that names `RECT`. Its own step because everything after it rests on it and
   because it is the one change here that would compile and be wrong.
10. **`hook_device` moves**, and with it `PRESENT` and the letterbox's own `Present` — the rewrite of the
    slot, which is a hook body like the others. The slot's signature becomes `*const Rect` and
    `windows_rect` goes. `runtime::Patches` and `hands_over_the_patches` go. In
    `orb-e2e`, `vtable_page`, `VTABLE_PAGE`, `VTABLE_STEP`, `VTABLE_TRIES` and `vtable_for` go with them,
    and the vtable is laid out in the space like everything else.
11. **The last four doors.** `create_window_ex_a`, `create_file_a` and `answer` to `orb-core` beside the
    halves of themselves that are already there, with `CREATE_WINDOW_EX_A`, `CREATE_FILE_A`, `ORIGINAL`,
    `CALIBRATION` and `ENABLED` and the three `install_over`s; then `attach_to` and `detached`. `orb`'s
    install lists take each body's address the way they already take `run_calc_chain`'s.

    **`is_game_class` and `GAME_WINDOW_CLASS` move with them, and they are what the exception above does not
    cover.** Both rewrites ask them — `register_class_a`, which stays for its brush, and
    `create_window_ex_a`, which is the first thing it does, *so a window that is not the game's is not the
    reason the monitor was read*. Neither names Windows: it is a `CStr` comparison against `c"BASE"`. So they
    go to `orb-core` and the one rewrite that stays calls across, which is the direction that is already
    allowed.

    **`orb::joystick`'s own test goes with `calibrate`.** `a_calibration_write_stops_where_the_caps_do` lays
    a `JOYCAPSA` and the game's auto-repeat counter next to each other and asserts the write stops at 0x194;
    it is a test of the function being moved, so it moves, and `orb/src/joystick.rs` is left with no tests of
    its own. **A test left behind is a compile error that invites deleting it**, which is the one way this
    step can quietly cost coverage.
12. **The dependency and the `rlib`, kept this time.** The same two lines as step 0, and the survey there is
    what says the list is empty: `crates/orb-e2e/Cargo.toml` loses `orb`, `crates/orb/Cargo.toml` loses
    `rlib` from `crate-type`, and it compiles. **This is the step that proves the rest** — it cannot be made
    to compile while anything a scenario drives is still in `orb` — which is why it is tried at the
    beginning and kept at the end, and why every error it prints at the beginning is a line of the worklist.

    **`orb` keeps unit tests of its own after this, and a `cdylib`-only crate still runs them**, which had
    to be measured rather than assumed: `hook.rs`'s two, over an import table written out by hand, are the
    only ones left, and losing them would be a coverage change this decision is not allowed to make.
    Measured by making the change and running it:

    ```sh
    $ cargo test -p orb --target i686-pc-windows-gnu --lib
    test hook::tests::an_import_a_module_has_not_got_is_named ... ok
    test hook::tests::an_imports_slot_is_swapped_and_what_was_there_comes_back ... ok
    ```

    Cargo builds the test harness from the lib's own source whatever `crate-type` says, so the `rlib` is
    only ever about who may `use` the crate.
13. **The documents.** `crates/orb/src/lib.rs`'s own opening, which lists nine files by what each does;
    `crates/orb/Cargo.toml`'s `[lib]` comment, which says why the `rlib` is there; `crates/orb-e2e`'s, which
    says which four doors it knocks on; `SPEC.md`'s table of where each thing lives; `TODO.md`'s *What is
    still where it was*, whose first three entries this empties; and the status of
    [0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md), whose title this
    makes true, whose step 8 prediction it vindicates, and whose correction #4 it turns from a finding into
    a thing that was true of that step and not of the design.

**What it buys.** Not reach — a scenario runs almost all of this already — but answers where a null handle
answers now. Four things become askable: how often a pad that is answering is read and how often one that
is not, what orb says about the device it found, which bar the status line went in and where in it the
block landed, and that a stack of fewer lines clears the rows the last one wrote in, which is a bug that
was fixed and has never had a test. And one mechanism goes: with the walk and the bar on the seam's far
side, the only thing handed from `orb-core` back into `orb` is the device's `Present` slot.

**And what it does not buy.** Not one line of what a scenario runs changes: the fake calls the same
functions at the same points under `orb_core::` names, so `cargo xtask test` passing at step 12 says the
move was faithful and nothing more. What stops being reachable from a scenario is `DllMain`, `attach`,
`install_hooks`, `hook`, `pe` and `crash` — which no scenario reaches today either, a laid-out game having
neither an import table nor PE headers. **So the coverage is the same before and after**, and the thing that
changed is that the crate boundary now says so.

**What it costs.** Step 5 rewrites a shipped drawing path whose far end no test reaches, so until the
scenario named in that step exists the only witness before and after is a launch: the four lines in the
black beside the game, at both sizes the bar can be, on a window whose letterbox leaves room for one and on
one that does not. That belongs in `TODO.md`'s list of what only the real game can answer until the
scenario stands behind it.
