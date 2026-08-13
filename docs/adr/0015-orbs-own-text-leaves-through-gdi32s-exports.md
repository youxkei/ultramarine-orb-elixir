# 15. orb's own text leaves through gdi32's exports, not through orb's imports

**Status:** accepted and built. `orb_api::real::gdi` resolves the seven GDI calls that carry a string or
hand back a font — `ExtTextOutW`, `TextOutW`, `GetTextExtentPoint32W`, `GetTextFaceW`,
`CreateFontIndirectW`, `CreateFontW`, `AddFontResourceExW` — out of gdi32's export directory, once, and
`real::window` (the status line) and `real::text` (the overlay's glyphs) call through those. Everything
else about the drawing is still called through this DLL's imports. Where an export cannot be read the
call falls back to the imported one. That orb and thcrap are in one process by a launch's own doing, and
not only when a player arranges it, is
[0016](0016-the-game-is-handed-to-a-thcrap-installed-beside-it.md).

## Context

**orb is not the only thing injected into the game.** 紅魔郷 is played in English through thcrap, which
injects its own DLLs and rewrites import tables to put a language pack's words where the game's own were.
It rewrites them **per module, orb's DLL included** — its own log names the file it did that to — and what
it puts in the GDI text entries is a replacement written for the game's `-A` calls. Handed a `-W` call it
takes the count of characters for a count of bytes.

**What that did to the screen**, measured on 紅魔郷 1.02h with orb and thcrap 2025-12-02 in one process
and thcrap's `lang_en` installed:

- Beside the game, orb's status line came out `INPUT LAG 3.0ms` → `INPUT LA` and `COMPOSE 2.6ms` →
  `COMPOSE`: the first 15 and 13 *bytes* of those strings as UTF-16, with the NULs dropped.
- In the bar *under* the game — the shape a client taller than 4:3 gets, which a tiling window manager
  makes of any window — the same arithmetic reads past the string instead, and the bar filled with rows of
  glyphs from whatever is next in the process, a COM GUID legible among them.
- The game's own text was correctly translated throughout, and orb alone in either window shape wrote its
  status line correctly. So what was wrong was orb's route to the screen and not the patch.

**Two things orb cannot do about it.** It cannot ask the patcher not to: nothing in that direction is
orb's to arrange, and a player installs the two separately. And it cannot fix the count from its side —
the call orb makes is already correct.

## The decision

**Take the address out of gdi32's export directory and call that.** An import table is per module and is
what a patcher rewrites; the export directory is the loader's own and nothing rewrites it, so an address
read from it is the function Windows exports.

**Not through `GetProcAddress`**, which is the other obvious way to the same address and is itself one of
the calls such a patcher replaces — thcrap's `win32_utf8` hands out its own wrappers through it by design,
which is the whole of what it is for.

**Only the calls that carry text.** A memory DC, a bitmap, a blit and a `SelectObject` carry no string for
a count to be wrong about. Taking those out too would be a longer table, more `unsafe`, and nothing bought.

## What follows

- **orb's text is orb's again whatever else is in the process**, which is what a status line is for: it is
  read to find out what a run did, and a run under a translation patch is a run whose numbers have to be
  readable.
- **A host this cannot be read on keeps what it had.** The fallback is the imported call, so the worst case
  is the behaviour before this decision rather than a screen with no text.
- **A forwarded export would be such a host, and the test is what says none of the seven is one.** gdi32
  forwards a good deal to gdi32full, and a forwarder's slot points inside the export directory rather than
  at code, which this reads as *not found* — the fallback then quietly puts the imported call back and this
  decision buys nothing. `every_call_is_found_inside_gdi32` fails there rather than passing quietly, which
  is what makes the tripwire a tripwire: a Windows that moves one of the seven is a build that stops here
  and not a status line somebody cannot read.
- **The export walk is orb-api's, below the seam**, beside the other things only the real Windows answers.
  `orb-core` reaches none of it, and a game laid out by hand draws through the simulated host's own
  bookkeeping, so nothing above the seam changes.
- **No e2e test covers the swap.** What an e2e scenario drives is above the seam, and this is a decision
  about which of two identical functions is called below it: a simulated host answers `write_lines` out of
  its own state either way. What the scenarios do still cover is that the status line says what it should;
  which door that text left through is not something a laid-out game can be asked.

## What was weighed and refused

- **Leaving it and telling whoever plays in English to accept it.** The status line is where a run's own
  numbers are read, and this is a repository whose subject is those numbers.
- **Restoring orb's own import entries after the patcher has rewritten them.** It would have to be done
  after every module load, it takes orb's imports back out from under something that is entitled to have
  patched them, and the order it happens in is not orb's to know.
- **Asking `GetProcAddress` for the addresses** — refused above: it is replaced by the same patcher.
- **Baking orb's own glyphs without GDI.** A rasteriser of orb's own would answer this and much else, and
  it is a font engine for a status line and three menus.
- **Taking every GDI call this way.** Refused above: the ones that carry no text have nothing to get wrong.
