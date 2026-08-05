# Conventions for this repository

The project is **Ultramarine Orb Elixir**. Written in full where it is being named, and
shortened to *orb* everywhere else, which is also what its crates and the files it installs
are called.

## Commit messages

Subject lines start lowercase with a bare verb. No type prefix, and not a noun phrase naming
the feature. The verb says what the code does once the diff is applied, not what the commit
does to the repository: the line is the predicate of an implied *"the code now"*, so `take a
key held through an alt-tab as already held` rather than `add tests for the mode menu`.
Somebody who never saw the diff should learn from the line what orb does differently now. A
commit whose diff is only tests or only documents takes the same form about them — what the
test now asserts, what the document now says.

The body explains why the change was made, concretely: the problem it solves, named and with
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

Reasons an alternative was rejected go in a comment beside the code that would otherwise tempt
someone back to it, not into `SPEC.md`.

## Code

- **Code says how.** Keep it self-explanatory; do not restate it in comments.
- **Tests say what.** Write them so the expected behaviour is clear from the test.
- **Comments say why not.** The alternative rejected, the constraint that is not obvious, the
  reason the straightforward approach was avoided.

Nothing machine-specific in anything committed. Scripts take what varies as an argument or
from the environment, and fail with a usage line when given neither.

Claims about the game's behaviour are established by measurement and the measurement is kept,
in the log or in a test. A plausible explanation is not a finding.
