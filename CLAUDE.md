# Conventions for this repository

The project is **Ultramarine Orb Elixir**. Written in full where it is being named, and
shortened to *orb* everywhere else, which is also what its crates and the files it installs
are called.

## Commit messages

Subject lines start lowercase with a bare verb. No type prefix, and not a noun phrase naming
the feature. The verb is what the code does once the diff is applied, not what the commit
does to the repository: the line is the predicate of an implied *"the code now"*, so `take a
key held through an alt-tab as already held` rather than `add tests for the mode menu`.
Somebody who never saw the diff should learn from the line what orb does differently now, in the
words the code uses for those things, without abbreviating them and without a metaphor standing in
for one: `hand Th06::present and Th06::play_sounds over as addresses` rather than `hand the frame
loop's two calls over`.

**A line names everything the code now does differently, and not the largest of them.** Where the diff
does three things the line says three, joined with `and`: `count the cadence in the compositor's
refresh period, flush for a window that is behind, and hand Th06::present and Th06::play_sounds over as
addresses`. **There is no limit on the number of clauses and none on the length** — the log already runs
past a hundred characters, and `take the mode from a menu over the game's title menu and the settings
from a dialog, not from orb.yaml` is one line. Picking the most consequential change and leaving the
rest to the body fails the condition above: a reader of that line comes away not knowing the others
happened, which is exactly what the line exists to prevent.

**The subject is a thing and never somebody** — the implied `the code`, or a test or a file where the
diff is only those — so the verb is one a thing does. A thing performs no speech act: nothing in orb
says, tells, asks, answers, reports, refuses, offers, claims, requires or plans anything, every one of
which wants somebody doing the speaking and somebody hearing it. Name what it does instead:

- `draw three lines under 完全無欠モード in ですます`, not `say what a mode does to a run`
- `put the question up before the screen has moved`, not `ask before the screen has moved`
- `fill the game's joystick read from the last sample a thread of orb's own took`, not `answer the
  game's joystick read`
- `hold a tuning.txt line's stage to being a stage`, not `refuse` one
- `take a key that moved to the command line as any other unknown key`, not `report` it

A frame does not say where its time went either — the `--pacing` line breaks it down — and つづきから
is offered *in* no launch after a `--clear` run rather than offered *to* one, a launch being nobody to
offer anything to. **No verb is an exception for being ordinary in computing**: a menu does not ask, a
function does not answer, a parser does not refuse.

A commit whose diff is only tests or only documents takes the same form about them — what the test
now asserts, and what a file now holds — and **there the subject is written out**, since the implied
one is `the code` and these are not it. A file is not somebody either, so the verb is one a file can
take: it does not say, plan or require anything, it *has* a rule, it *lists* an entry, it *is* a plan.
`CLAUDE.md now has a rule that every docs/adr file starts with a status line`, `TODO.md now lists
window::write_beside instead of retry_ui`, `docs/adr/0002 is the plan for Game to hand Th06::present
and Th06::play_sounds over as addresses`.

A diff that is code *and* tests is the code's, however much of it is the tests: the implied subject is
`the code`, and a scenario is how the change is known rather than something the program does
differently. What the scenarios now cover goes in the body.

The body is why the change was made, concretely: the problem it solves, named and with
whatever numbers or addresses make it real, and why this way rather than the obvious
alternative. Not a list of what changed, which the diff carries, and not a summary abstract
enough to fit some other change.

## The documents

| | |
| --- | --- |
| `README.md` | what orb is, how to build and install it, and where to read further |
| `SPEC.md` | the final form only. No history of what was tried, no record of what a mechanism used to be |
| `DONE.md` | what works, and how it was checked |
| `TODO.md` | what is left, with what is already known about each |
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

Claims about the game's behaviour are established by measurement and the measurement is kept,
in the log or in a test. A plausible explanation is not a finding.
