# 20. The address space a chapter needs is kept when the game hands it back

**Status:** accepted and built. `Snapshot::update` and `Drop for Snapshot` name what a restore of them
would put back through `orb_api::mem::a_restore_needs`, `orb::memtrack`'s `HeapFree` and `VirtualFree`
hooks ask `mem::hold_what_a_restore_needs` for the space after every free, and
`crates/orb-e2e/src/the_address_space_a_chapter_of_th07_needs.rs` holds the naming. Stands on
[0010](0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md), whose two homes
are where the halves of this went.

## Context

A snapshot is a copy of ranges of the game's memory, and a range is **where** it is: `Snapshot::restore`
writes each one back to the address it was read from, committing pages the game has given up in the
meantime so that the restored pointers in the game's own bookkeeping point at memory again.

That works while the *address space* is still the game's. It is not always. A block of a few megabytes is
not inside one of a heap's segments: Windows gives a heap a reservation of its own for each, and freeing
one hands that reservation back to the OS. 妖々夢 frees blocks of that size — a stage's own data — and
what is left behind is address space anybody may be given next.

**Orb does not see the free that does it.** Its hooks are on the exe's import table — `HeapCreate`,
`HeapAlloc`, `HeapReAlloc`, `HeapFree`, `VirtualAlloc` and `VirtualFree` — and the release of a large
block is the heap manager's own `VirtualFree`, called from inside `ntdll` where an import hook does not
reach. So the game's `free` returns, the space is gone, and nothing has told orb.

**Two takers have taken 妖々夢 down**, both in one session of `--judge` stepping back through a stage:

- orb's own copy. A step back faulted inside the *next* chapter's snapshot — `code 0xc0000005 ... reading
  0x00000018`, the stack `render → run_calc_chain → on_update → run_to → Chapters::observe →
  Chapters::begin_chapter → chapter::take`, and the faulting instruction the memcpy loop over `saved`
  with a slice pointer of 8 and a count that was not zero. A restore had written the game's old bytes
  over the `Vec` orb keeps its own buffers in.
- the process heap. With the ranges orb had *named* skipped, the step back after that faulted inside
  `ntdll`'s own heap with `chapter::take` under it — `reading 0x00000007`, `ecx` and `esi` both the heap's
  handle. The heap had been given the space instead, and its bookkeeping was what got written over.

Neither is a list that can be finished. Every allocator in the process is a candidate, and orb cannot
enumerate them.

## Decision

**The address space a kept chapter would put back is orb's to hold from the moment the game gives it up.**

- Each snapshot names the ranges it holds — one entry per snapshot, `mem::a_restore_needs` as the set is
  built and `mem::a_restore_is_done_with` as it is replaced or dropped. What says a range still matters is
  that a snapshot naming it is still there, so a stage beginning or a run ending gives them up on its own.
- After every free the game makes through its imports, `mem::hold_what_a_restore_needs` asks whether the
  space that pointer was in has come free inside one of those ranges, and reserves it where it has. After
  the free rather than before it, because the release nobody can hook is what has to be noticed, and being
  free afterwards is what says it happened.
- **Reserved and not committed.** What it costs while nothing needs it is address space; the pages come
  back through the commit `restore` already makes before its copy, which now cannot fail for want of the
  space and cannot land in somebody else's.

## What follows

**A reservation is never given back while the process lives.** After a restore has committed inside one,
the game is playing out of that memory — the block its restored bookkeeping says it owns — so releasing it
would take memory out from under a run. What that costs is address space in a 32-bit process, and the log
names each one as it is kept, which is where the size of it is read off rather than guessed.

**A refusal stays as the net, and it is one question rather than a list.** Every saved range carries the
allocation its bytes came out of — `mem::allocation_at`, the reservation the host says those pages were
handed out in — and `restore` writes nothing whose pages are *another* allocation now, saying so in the
log. Free is still the game's to have back, which is what the commit before the copy is for; what is
refused is pages somebody else was given. With this decision built that line means the space came free
without the hook seeing it, which is a hole in *this* and the one thing that would say so.

Enumerating who else might hold the pages is what that replaced, and it was tried twice: the ranges orb
had named, and then the process heap's own regions. Each answered one of the crashes below and neither
answered the next one — the second left the headers a heap hands its large blocks out with, and the step
back after it faulted in the heap again.

**A laid-out game cannot answer the keeping**, only the telling. `Sim::hold_what_a_restore_needs` writes
the ask down and keeps nothing, address space being what a simulated Windows does not hand out; the e2e
test asks which ranges are named and for how long, which is the half the decision is in.

### What was rejected

**Not passing the game's `HeapFree` through** for a block a chapter names, which was the first shape of
this and is where it started. The game's allocator would then hold what it believes it freed: the divergence
is invisible to the game, but a stage's frees stay committed, every snapshot after that copies them, and
the copies grow with the leak. Reserving costs address space where this costs memory twice over.

**The refusal on its own**, which is the net above and not a fix: a range skipped is a range the chapter
does not get back while its restored heap bookkeeping says it is allocated, so the game is left owning
memory whose contents are somebody else's. It says a chapter was lost; it does not keep one.

**An arena of orb's own for the snapshot buffers**, high in the address space and reserved at the attach.
It takes orb out of the competition and leaves everybody else in it — the first of the two crashes and not
the second.
