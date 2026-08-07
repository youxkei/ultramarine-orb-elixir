# Conventions for this repository

The project is **Ultramarine Orb Elixir**. Written in full where it is being named, and
shortened to *orb* everywhere else, which is also what its crates and the files it installs
are called.

## Commit messages

**The subject of the subject line is the commit**, and the line says what that commit does to this
repository: the head verb is the editing action, lowercase and imperative, with no type prefix and no
noun phrase naming the feature.

**In the words the code uses for the things it touches**, unabbreviated, with no metaphor standing in
for a name and no periphrasis where the code has a name. This is the rule that decides whether the line
is worth reading and the one easiest to lose. Before committing, read the line back as though you had
not seen the diff, and for every noun phrase in it ask whether the code has a name for that thing; where
it does, a paraphrase in its place is a defect. Having seen the diff makes you the worst judge of this,
so it is a deliberate pass and not an impression.

**Name the files and the items rather than the area they are in.** An abstract gesture at what was
touched costs the reader the diff, the same way a paraphrase does.

**A line names everything the commit does, and not the largest of them**, joined with `and`. **There is
no limit on the number of clauses and none on the length** — the log already runs past a hundred
characters. Picking the most consequential change and leaving the rest to the body is what this exists
to stop: a reader of that line comes away not knowing the others happened. Where a diff really is
unrelated changes, separate commits are better than one long line — but a line that has to be short is
never the reason to leave a change unnamed.

The body is why the change was made, concretely: the problem it solves, named and with
whatever numbers or addresses make it real, and why this way rather than the obvious
alternative. Not a list of what changed, which the diff carries, and not a summary abstract
enough to fit some other change. What a commit's scenarios now cover goes here rather than in the
subject, unless the scenarios are the whole of the diff.

## The documents

| | |
| --- | --- |
| `README.md` | what orb is, how to build and install it, and where to read further |
| `SPEC.md` | the final form only. No history of what was tried, no record of what a mechanism used to be |
| `TODO.md` | what is left, with what is already known about each — and what is built and still waiting on a run against the real game |
| `docs/adr/` | one file per decision about how the code is *shaped*, numbered, in the order they were taken. A **status** at the top, then context, the decision, what follows from it. A decision belongs here rather than in `SPEC.md` when what has to survive is the reasoning — `SPEC.md` carries the final form and would have to throw the reasoning away — and rather than in `TODO.md` when it is settled rather than pending |

The status is the first thing because a reader knows from it which document this is: `accepted, not
built` is a plan and the code does not look like it yet, `accepted and built` is a description of the
tree. It carries where to read the rest of the story too — what the thing that got built stands on,
and which later decision overturned something this one said, so the reasoning can be followed from
either end. A
decision nobody took is not a file here: an alternative that was weighed and rejected is recorded
inside the decision that rejected it, beside what was chosen instead.

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
- **It confirmed the code works on the real game** — a scenario asserting the same thing, which is then
  the record. A confirmation written as a scenario is one anybody can repeat on demand, so the scenario
  is the work and the write-up comes free.
- **It is ahead of all three** — `TODO.md`, named and carried. A measurement still waiting for a
  constant or a scenario to stand beside belongs there, and so does everything built that still waits
  on a run against the real game.

**A claim beside the thing it is about moves when that thing moves**, which is what the list above is
for: the measurement goes where the reader of that thing is already standing, and a document collecting
measurements of its own gives that up.
