# Conventions for this repository

The project is **Ultramarine Orb Elixir**. Written in full where it is being named, and
shortened to *orb* everywhere else, which is also what its crates and the files it installs
are called.

## Building and checking

`cargo xtask build --release` is what installs: it puts the one file to hand out,
`target/i686-pc-windows-gnu/release/orb.exe`, which carries `orb.dll` inside itself. Building needs
the 32-bit mingw linker `.cargo/config.toml` names, `mingw-w64-i686-gcc` from MSYS2's MINGW32
environment, on `PATH`. **The README says none of this** — it tells whoever is going to play to
download that exe from the latest release, so what a release is built with is written down here and
nowhere else.

Everything that has to name a target goes through `cargo xtask` — `build`, `test`, `clippy`,
`seam`, `coverage` — because there is no `[build] target` naming it for them; `cargo xtask` with no
task lists them, and `.cargo/config.toml` says why there is no such key. A bare `cargo check`
therefore builds for the host and fails on `windows-sys`, so **point the editor at the target
instead**: rust-analyzer's setting is `rust-analyzer.cargo.target`, and the target is
`i686-pc-windows-gnu`.

`cargo xtask test` is also what installs the git hooks, husky-rs pointing `core.hooksPath` at
`.husky`. What its `pre-commit` runs and why is written in that file, `NO_HUSKY_HOOKS=1` and
`git commit --no-verify` included. The tree is held at zero warnings, so a lint that is wrong about
this code is answered with an allow and the reason beside it rather than left in the output.

## Commit messages

**The subject line is a summary in 50 characters**, 72 at the very most. The subject of it is the
commit, and the line says what that commit does to this repository: the head verb is the editing
action, lowercase and imperative, with no type prefix and no noun phrase naming the feature. At that
length one line cannot hold a commit that did seven things, and it is not asked to — it says which
thing was done to what, and the body carries the rest.

**The body opens with what the commit does**: one bullet per change, and a bullet for every change, so
that a reader learns from the list which changes happened without opening the diff. Picking out the
largest of them and leaving the others unwritten is what this exists to stop. Where a diff really is
unrelated changes, separate commits are better than one long list.

**Each bullet is a summary of its change and not an inventory of it**, in the words the code uses for
the things it touches: unabbreviated, with no metaphor standing in for a name, no periphrasis where the
code has a name, and no abstract gesture at an *area* — *the docs*, *the e2e tests*, *the score file*,
none of which leaves a reader anything to reconstruct. It is the area that makes those bad and not the
shortness: a measurable property with one direction to move in is no such area, and naming it with the
direction it went does say what happened.
Name the file or the item the bullet is about, say what it now does, and stop: the identifiers, the
constants, the addresses, the arguments, the call sites and the e2e test names inside that change are
the diff's, and a bullet that needs a sub-item per name is too detailed rather than too short.

**Then why the change was made**, concretely: the problem it solves, named and with whatever numbers or
addresses make it real, and why this way rather than the obvious alternative. Not a list of what
changed, which the bullets and the diff carry between them, and not a summary abstract enough to fit
some other change. What a commit's e2e tests now cover goes here, unless the e2e tests are the whole of
the diff.

**The read-back is not yours to do.** Having written the message you cannot read it as the reader it is
for, who has not seen the diff: you supply the owner a noun phrase left out, so the line reads as
finished to you and to nobody else. Before committing, hand **the subject line by itself** to an agent
with an empty context, no sight of the diff and no access to this repository, and ask it three things —
what the line claims was changed, whose each thing in it is, and which noun phrases it can read more than
one way. Then compare its answer with what the commit did. Where the two differ the line is wrong and the
reader is right. Where the subject leans on a bullet, hand that bullet over with it.

What it reports is judged rather than obeyed: it will also name what no subject line could settle. `cap a commit's subject line at fifty characters in CLAUDE.md` came back read correctly and still
flagged *a commit* for being any commit rather than one — which is the rule being general — and asked
which CLAUDE.md, there being a global one as well. Both stay. What does not stay is a difference in what
was changed: `cut CLAUDE.md's subject line to fifty characters` came back as a subject line **belonging to
the file**, and `cut a commit's subject line to fifty characters in CLAUDE.md` as an example inside the
file cut down to fifty characters as much as a limit set at fifty — one word each time, and the fifty
characters had room for it.

The check the reader cannot make is the one against the code: for every noun phrase, ask whether the code
has a name for that thing, and where it does, a paraphrase in its place is a defect.

**A rule above refuses a wording only where three things hold**: it is the rule for that part of the
message, whatever would replace the wording fails it too, and refusing serves the reason the rule gives.
Where two candidates come out level the ordinary phrasing wins, a technicality being no tiebreak — and a
line defended over two turns was defended from the letter.

## The documents

| | |
| --- | --- |
| `README.md` | one sentence saying what orb is, where to download it, and which games it runs — and nothing else. **Not a feature list**: the window, the frame pacing and the frame of input lag are not in it, and the table of games says supported or not supported and nothing more. Nor is how any of it works — no build, no target, no linker, no file format. Nor is there a section pointing at any of the documents below: the only link out is to the other language. Whoever is going to play knows 完全無欠モード already, is installing one file, and reads past everything that is neither of those two things |
| `README.ja.md` | the same in Japanese, and the two are edited together. The game and everything orb puts on the screen are Japanese, so it is the language most of the people it is for read. Each names the games and the mode in its own language — 東方紺珠伝's 完全無欠モード here, Legacy of Lunatic Kingdom's Pointdevice mode in `README.md`, where a Japanese title is a title the reader cannot read |
| `SPEC.md` | the final form only. No history of what was tried, no record of what a mechanism used to be |
| the repository's issues | what is left, with what is already known about each: one issue per piece of work, and none of it in the tree. There is no `TODO.md` — a worklist in the tree is one every branch carries its own copy of, and what is left to do is not a fact about a commit |
| `docs/todo/` | one file per piece of work too long to sit in an issue as a paragraph: what was measured, how to measure it again, and the order the work is worth doing in. A file belongs here rather than in an issue when the measurement behind it has to be repeatable — the recipe goes in the tree it is run against, since a worklist nobody can re-derive is one nobody can tell is still true |
| `docs/adr/` | one file per decision about how the code is *shaped*, numbered, in the order they were taken. A **status** at the top, then context, the decision, what follows from it. A decision belongs here rather than in `SPEC.md` when what has to survive is the reasoning — `SPEC.md` carries the final form and would have to throw the reasoning away — and rather than in an issue when it is settled rather than pending |

The status is the first thing because a reader knows from it which document this is: `accepted, not
built` is a plan and the code does not look like it yet, `accepted and built` is a description of the
tree. It carries where to read the rest of the story too — what the thing that got built stands on,
and which later decision overturned something this one said, so the reasoning can be followed from
either end. A
decision nobody took is not a file here: an alternative that was weighed and rejected is recorded
inside the decision that rejected it, beside what was chosen instead.

**A file in `docs/todo/` is deleted when its work is done**, and that deletion is the last step of the work
rather than tidying afterwards. It is one piece of work and not a permanent document, and a worklist with
nothing left on it costs whoever finds it a read to learn that.

It is not a plain `rm`, though. Everything in the file that has to survive goes first, and where each thing
goes is what the rules below already say — the reason an e2e test or a check *cannot* be written to a comment
beside the code that would otherwise tempt somebody to write it, the recipe for a measurement to the code
that implements it, a decision still to be taken to `docs/adr/`. **Not back into the issue**, which is going
the other way: what is in one belongs in files here, one per piece of work, and moving anything back into it
is work to undo later. What needs no home at all is the measurement itself, wherever the recipe it went with
can be run again — a number in a document is stale the moment the tree moves, and the command that produces
it is not.

Reasons an alternative was rejected go in a comment beside the code that would otherwise tempt
someone back to it, not into `SPEC.md`.

## Code

- **The how is in the code.** Keep it self-explanatory; do not write it again in comments.
- **The what is in the tests.** Write them so the expected behaviour is clear from the test.
- **The why not is in the comments.** The alternative rejected, the constraint that is not obvious,
  the reason the straightforward approach was avoided.

Nothing machine-specific in anything committed. Scripts take what varies as an argument or
from the environment, and fail with a usage line when given neither.

Claims about the game's behaviour are established by measurement. A plausible explanation is not a
finding. **Every measurement is kept beside the thing it is about, and what it did says which thing
that is:**

- **It decided how the code is shaped** — the file in `docs/adr/` holding that decision, beside the
  reasoning it settled.
- **It is why a constant is the number it is** — beside that constant, with how it was found, so that
  whoever changes the number reads what it was measured against first.
- **It confirmed the code works on the real game** — an e2e test asserting the same thing, which is then
  the record. A confirmation written as an e2e test is one anybody can repeat on demand, so the e2e test
  is the work and the write-up comes free.
- **It is ahead of all three** — the issue for that work, named and carried in it. A measurement still
  waiting for a constant or an e2e test to stand beside belongs there.

**A claim beside the thing it is about moves when that thing moves**, which is what the list above is
for: the measurement goes where the reader of that thing is already standing, and a document collecting
measurements of its own gives that up.
