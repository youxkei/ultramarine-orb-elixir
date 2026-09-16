# How to work in this repository

Name the project **Ultramarine Orb Elixir** where it is being named in full, and *orb* everywhere
else — that is also the name of its crates and of the files it installs.

## Building and checking

- Run every task that has to name a target through `cargo xtask`: `build`, `test`, `clippy`, `seam`,
  `coverage`, `real`. `cargo xtask` with no task lists them.
- Build what is handed out with `cargo xtask build --release`; the one file is
  `target/i686-pc-windows-gnu/release/orb.exe`. Have `mingw-w64-i686-gcc` from MSYS2's MINGW32
  environment on `PATH` — `.cargo/config.toml` names it as the linker.
- Point the editor at the target: set `rust-analyzer.cargo.target` to `i686-pc-windows-gnu`.
- Test on the real game with `cargo xtask real <copy of the install>`, on a copy made for it, after the
  same cases pass in `orb-e2e`.
- Run `cargo xtask seam` after touching a crate above the seam: it builds those crates for a target with no
  Windows, which is what says no call went past the seam.
- Run `sh .husky/pre-commit` by hand before asking for a commit; jj runs no git hooks.
- Keep the tree at zero warnings. Answer a lint that is wrong about this code with an allow and the
  reason beside it.

## Commit messages

Write commit messages in English.

### The subject line

- Write it as `type(scope): description`
  ([Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)), 50 characters
  including the prefix, 72 at the very most.
- Pick the type from `feat`, `fix`, `docs`, `chore`, `test`, `refactor`. A commit that needs a type
  not on this list adds it to the list.
- Give the scope where the change is one crate's, as that crate's name (`orb-core`, `orb-e2e`, …), and
  leave it out where it is not.
- Write the description as a sentence whose subject is this commit: the head verb is the editing
  action — add, move, rename, remove, … — lowercase and imperative, with the type in front of it. Put
  behaviour in a clause hanging off the thing that was added, so that clause's subject is that thing.
- Say in it which one thing was done to what. The body carries the rest.

### The body

- Open with one bullet per change, and a bullet for every change.
- Write each bullet as a summary of its change: name the file or the item it is about, say what it now
  does, and stop. Use the code's own words for what it touches, unabbreviated, and name a measurable
  property together with the direction it moved. Leave the identifiers, constants, addresses,
  arguments, call sites and test names inside the change to the diff.
- Then write why: the problem the change solves, with the numbers and addresses that make it real, and
  why this way rather than the obvious alternative. Where a commit adds e2e tests beside other changes,
  write here what they now cover.
- Give unrelated changes separate commits.

### Reading the message back

Have the message read back by agents before asking for a commit, and change the line where a reader
and the commit disagree.

1. Hand **the description by itself** — prefix off, together with any bullet it leans on — to an
   agent with an empty context, no sight of the diff and no access to this repository. Ask it: what
   the line claims was changed; whose each thing in it is; which noun phrases it can read more than
   one way; whether the line is true read as a sentence whose subject is this commit; and whether any
   word in it is figurative rather than the plain word. Compare its answer with what the commit did.
2. Hand **the diff and the line** to a second agent. Ask it: what the diff does, in one sentence of
   its own; whether the line names what the commit is *for* or a step on the way to it; whether it is
   true of the whole diff or of a part of it; and whether any word in it is one the diff itself
   deletes.
3. Check every noun phrase against the code yourself: where the code has a name for that thing, use
   that name.

Judge what the readers report rather than obeying it; a reader also names what no one line can
settle. Where two wordings pass equally, use the ordinary one. What settles a wording is this section
and the words the code uses — a subject in the log is a sample of an earlier convention, and settles
nothing.

## The documents

| | |
| --- | --- |
| `README.md` | one sentence: what orb is, where to download it, and which games it runs, the table saying supported or not supported. The one link out is to `README.ja.md` |
| `README.ja.md` | the same in Japanese, edited together with `README.md`. Each names the games and the mode in its own language |
| `crates/orb-core/src/game/<game>/asm.md` | what one game does, read off its exe: the build's md5, the commands to read it again, the functions with every address in them, and at the end the list of what is not established. One file per game, pulled into the game module's doc with `#![doc = include_str!("asm.md")]` |

Where a piece of writing goes:

- what orb itself is and does → the doc comment (`//!`, `///`) of the crate, module or item that does
  it
- the reasoning behind how the code is shaped → the `//!` of the crate or module it constrains, and
  *Shape* below where it is a rule for working here
- a decision still to be taken → the description of the change for it (see *What is left*)
- a reading that spans a whole path through an exe → that game's `asm.md`
- the measurement a constant is the number it is because of → a comment beside that constant, with how
  it was found
- the reason an alternative was rejected → a comment beside the code that would tempt someone back to
  it
- how a release is built → this file

## What is left

- Write work still to do into the description of the change whose work it is:
  - where that change exists — the one being worked on, or another in the line of work — as a
    `TODO: ` line below what the message already says, one line per piece of work;
  - otherwise as a new empty change, one per piece of work, inserted into the line of work with
    `jj new --no-edit -A @ -m "TODO: ..."`, its description starting with `TODO: ` so that `jj log`
    tells work to do from work done.
- Carry in it the problem, everything already known, what was measured, and the order the work is
  worth doing in.
- Put the recipe for a measurement that has to be repeatable into the tree beside the code it is run
  against, as a script or a test.
- Rewrite the TODO as the last step of its work: an empty change's description into the commit message,
  a `TODO: ` line into the bullet and the why its work earns. Move what has to survive to its home
  first — the reason a test or a check cannot be written to a comment beside the code that would tempt
  somebody to write it, the recipe to the code that implements it — and keep in the message the problem
  with the numbers and addresses that made it real.

## Code

- **The how is in the code.** Keep it self-explanatory.
- **The what is in the tests.** Write them so the expected behaviour is clear from the test.
- **The why not is in the comments.** The alternative rejected, the constraint that is not obvious,
  the reason the straightforward approach was avoided.

### Shape

- Put code by what it touches:
  - `orb` holds code whose subject is another module's memory — `DllMain`, the jump written over a
    prologue, the import table entry and the vtable slot swapped, the PE headers those are read out of,
    the crash filter, and the install lists saying which hook goes with which prologue. A hook body makes
    the one call its rewrite consists of and stays beside its patch.
  - `orb-api` is the seam: every call to the host — Win32, COM, GDI, DirectSound, the files orb reads
    and writes, the machine's language, the `pause` instruction — as a facade, with `real/` behind it on
    Windows and `orb-sim` behind it in a test. Mirror the host's calls in it and keep every decision above
    it.
  - `orb-core` holds everything that decides what happens to a run and names no `windows-sys`: the hook
    bodies, `Originals`, `attach_to`.
- Hand every call orb makes into the game over from the `Game` as an address (`FrameCalls`, `Walks`,
  `Originals`, `handed_over!`), so that a game laid out by hand answers with functions of its own.
- Give each game a `Game` of its own under `crates/orb-core/src/game/<game>/`: `mod.rs` for the addresses
  and offsets, `image.rs` for the address space laid out, `chapters.rs` for the midstage table. Choose the
  game once, at the attach, out of `game::KNOWN`, which the launcher reads too.
- Answer what has been measured and decline the rest: a method with no reading behind it answers `None`
  or the emptiest true answer, and a process orb has no addresses for is left as it was found. Read a
  game's resume set out of its own replay restore, field for field.
- Write an e2e test in `orb-e2e`, driven by the fake game in `orb-e2e/src/fake/`, speaking only the host
  and the input — the display, the compositor, the window in front, the keys and the pad — and reading
  everything else back out of the game's memory, the log and the game's records. Keep the code under
  test free of a test-only path, and add nothing to production for a test to read.
- Make the fake game copy the real one wherever orb calls through, and owe nothing where orb replaces or
  never reads. A divergence is a defect where it makes an e2e test pass that the real game would fail.
  Fix fidelity red first: read the game's code, write the e2e test from it, watch it fail against the
  fake, then change the fake.
- Keep the frame loop's e2e tests in `orb-e2e/src/pacing.rs`, one section per subject, with the functions
  that judge a rate at the file's top level taking values — microseconds and the log's lines — and
  nothing of the host.

### Language

- Write prose in English: comments, doc comments, test names, commit messages. Use the English name
  for a term the game community holds in Japanese — *deathbomb* for 決死結界, *focused* for 低速,
  *Pointdevice mode* for 完全無欠モード, *the warning line* for 予告線, *stage 2* for 2面.
- Keep in Japanese what the game itself holds: a string the game draws or orb draws over it (an e2e
  test matches its characters), a file name the game owns, a directory of an install, and
  `README.ja.md`.

### Numbers

- Name a list; where its count is worth having, write the command that answers it.
- Write a number down where the game or Windows fixes it, where a test expects it, and in the commit log,
  which records a moment.
- Make scripts take what varies as an argument or from the environment, and fail with a usage line
  when given neither.

### Measurement

- Establish a claim about the game's behaviour by measurement. Establish where a thing is written
  plainly by a grep over the disassembly, and that a field never changes by watching it.
- Keep each measurement beside the thing it is about, chosen by what the measurement did:
  - it decided how the code is shaped → the `//!` of the crate or module it shaped
  - it is why a constant is the number it is → beside that constant, with how it was found
  - it confirmed the code works on the real game → an e2e test asserting the same thing, which is the
    whole record of it; what did not work → an empty change inserted for it
  - it is ahead of all three → the description of the empty change inserted for that work
