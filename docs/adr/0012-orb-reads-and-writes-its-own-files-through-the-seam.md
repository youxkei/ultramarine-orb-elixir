# 12. orb reads and writes its own files through the seam, and no e2e test touches a real filesystem

**Status:** accepted and built. `orb_api::fs` carries six calls — `read`, `read_to_string`, `write`,
`create_dir_all`, `remove_file`, `files_in` — over `std::io::Result`; `orb_sim::Files` answers them out of a
map and refuses on demand; and the eleven `std::fs` call sites above the seam are gone, six in
`orb-core/src/resume.rs`, two in `orb-core/src/tuning.rs` and three in `orb-config/src/lib.rs`.
`fake::scratch` makes no directory, `Fake::attach_finding` puts what an earlier session left into the store,
and the four e2e tests that read a file back read it out of the store.

**It overturns the rule [`orb_api`]'s own module comment gave for what belongs behind the seam.** That rule
was *whether it is the host doing something to orb that no test could otherwise decide*, and it is why files
were left out: a test can put a file in a directory it owns, so files decided themselves. The rule is now
whether orb reaches outside itself for the thing at all.

## Context

**Eleven call sites read and wrote real files, and the e2e tests drove them by making real directories.**
`fake::scratch` built one under the system temp directory per launch, named after the test and the process
id, emptying it first with `remove_dir_all`. `Fake::attach_finding` wrote into it what an earlier session was
to have left. Four e2e tests read files back out of it with `std::fs::read`.

It worked, and three things were wrong with it.

**A directory to empty first is a directory left behind.** `scratch` began with `remove_dir_all` precisely
because the last run's was still there — which is the same thing as saying every failed run leaves one. The
name carried the process id to keep two runs apart, which is a workaround for the store not being per-test in
the first place.

**A failure could only be arranged, never declared.** The two arms of `resume::write` that report a write
that did not happen were reached by putting a *file* where the directory has to go and a *directory* where
the file has to go. Both work. Neither says what it means: an e2e test that arranges a filesystem into a
shape where `std::fs` refuses is asserting about the machine it happens to be on, and the next question —
a disk that is full, a file another process holds open — has no such shape to arrange.

**And it was the one host call orb made outside the seam.** Everything else orb reaches for — the game's
memory, the clock, the display, the window, the fonts, the log, the pad — goes through `orb_api`, which is
what lets `cargo xtask seam` check the boundary by building `orb-core` for a host with no Windows on it. The
files went round it, and the seam check could not see them: `std::fs` builds on Linux, so nothing failed.

## The decision

**Everything orb reaches outside itself for goes through the seam, and the files are no exception.**

`orb_api::fs` is six free functions in the shape every other module here has: a facade with the real answer
behind it and whatever `Win` a test installed in front. Two things about it differ from its neighbours and
both are deliberate.

**`std::io::Result` crosses the seam.** It is neither a `windows-sys` type — which is the rule the seam has
about its signatures — nor a new one. What every call site does with the error is put it in a line of the
log, so an error that could not say what went wrong would be a seam that cost the log its diagnosis:
`resume: cannot write …: {error}` is the line somebody reads when a run was not kept.

**The real implementation is `std::fs` on every host, not `#[cfg(windows)]`.** Every other module here has a
`real/` half under that cfg and a `no_windows` panic beside it; a file is the one thing this seam carries
that Windows is not the only one of. What that buys is the crates above the seam keeping their own
`#[cfg(test)]` tests on a host with no Windows — `orb-config`'s read and write of `orb.yaml` among them.

`orb_sim::Files` is a flat map from path to bytes with a set of directories that have been made, because what
orb does with directories is exactly two things: make the one its files go in, and list it. There are no
permissions, no metadata and no symlinks, none being read above the seam. What it adds that a disk cannot is
three declarations — `refuses_to_read`, `refuses_to_write`, `refuses_to_make` — and one obligation: a write
into a directory nothing has made fails, here as on a real host, so `resume::write` making its own directory
stays a thing the e2e tests hold it to.

## What follows

- **`fake::scratch` is a path and nothing else.** No directory of that name exists anywhere. It is still
  named after the test, so that a path in the log says which one wrote it.
- **Every e2e test's files are its own**, the store being per simulated Windows and so per test. Nothing to
  empty, nothing left behind, and no two tests sharing a directory.
- **The two write failures are declared.** `the_run_left_behind`'s two e2e tests say
  `files().refuses_to_write(path)` and put a file where the directory has to go, rather than arranging a
  disk.
- **`orb-config` gains an `orb-api` dependency** and a `sim` feature that turns `orb-api`'s on;
  `orb-core`'s `sim` names it so the chain is written down rather than left to cargo's feature unification.
- **`cargo xtask seam` now covers the files too**, every path through them being an `orb_api` call.

## What was weighed and refused

- **Leaving it, on the old rule.** The rule is what is wrong. It was written to justify the `pause`
  instruction being behind the seam — a thing with no Win32 call at all — and read the other way it excuses
  anything a test can reach by arranging the machine. That is most of a host.
- **The simulated Windows forwarding to `std::fs`.** It would have made the seam complete with none of the
  190 e2e tests changing, which is exactly what is wrong with it: the tests would still own real directories
  and still be unable to ask what a failure does. The seam would be a shape with nothing behind it.
- **A `#[cfg(windows)] real/fs.rs` for consistency.** Consistency with the other modules is worth less than
  `orb-config`'s own tests running on a host with no Windows, and a `no_windows` panic for a call `std::fs`
  answers everywhere would be a boundary drawn where there is none.
- **Directory objects in the store.** A `HashSet` of paths that have been made answers both questions orb
  asks. Anything more is a filesystem written to pass tests nobody has.
- **A store on `Sim` that outlives one test.** It is per simulated Windows, which is per launch, which is
  per test process — see `fake::in_its_own_process`. That is the property the temporary directories were
  reaching for with a process id in their name.
