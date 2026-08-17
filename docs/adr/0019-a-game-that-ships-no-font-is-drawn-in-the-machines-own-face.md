# 19. A game that ships no font is drawn in the machine's own face

**Status:** accepted and built. `overlay::loaded` in `crates/orb-core/src/overlay.rs` falls back to
`orb_api::text::Font::of_the_machine`, and a launch of 妖々夢 says so in its log — asserted in
`crates/orb-e2e/src/th07.rs`, where the same launch used to assert `overlay: unavailable`.

**And the face asked for is the game's own family, which this document first said it could not be.** The
walk below was of the *file* the game ships and stopped there; 妖々夢 ships none and **asks Windows for
one by name**, which the exe says outright — `CreateFontA` at 0x4322e7 with `ＭＳ ゴシック` and the weight
0x2bc. So the family is a fact about the game after all, and `Game::own_face` is where each game answers
it. What is below stands except where it says no name is asked for; *What follows* carries the correction.

## Context

**Everything orb draws, it draws with one font, and it took that font from the game.** 紅魔郷 ships
`font.ttf` beside its exe — the whole reason `overlay::loaded` takes a path at all — and orb loads it
process-private through `AddFontResourceExW`, asks GDI for `Rounded M+ 2p regular` at two heights, and
writes into the log which face GDI really selected. Everything of orb's own is drawn in it: the status
line, the retry menu, the wash a chapter gets, the mode question, and the word on the mark over the
lives.

**妖々夢 ships none.** Its fonts are inside `th07.dat` as sprites, and its directory holds no `.ttf` at
all. So `Overlay::new` answered `None`, the log said `overlay: unavailable`, and *nothing of orb's was
drawn* — which is not a degraded 完全無欠モード but no 完全無欠モード at all: the retry menu a death
offers is the mode, and a menu nobody can see is not a menu.

That was written down as a question rather than a gap — `docs/todo/th07-the-run.md` had it as "the font,
which is a question rather than a measurement" — because no walk of the exe settles it. It is a choice
about what somebody sees.

## Decision

**Where the game ships no font, ask the host for a face of its own and draw in that.**

`orb_api::text::Font::of_the_machine` is that ask: the same `LOGFONTW` the game's own font is asked for
through — the height, `SHIFT_JIS` for its charset, antialiased — with **no font resource added**, and with
the family the game itself names where it names one. Where it names none the name is left empty and which
face comes back is the machine's answer to "something that can draw Shift-JIS at this size".
`overlay::loaded` tries the game's file first and falls back, and the log says which of the two this launch
is and which face came back, because what a substituted face changes is the glyphs on the screen and that
is worth a line either way.

Nothing changes for 紅魔郷: its file is there, so the fallback is never reached.

## What follows

- A machine with no Japanese face draws orb's own screens in boxes. The line in the log names the face,
  which is what turns that into something somebody can read rather than guess at.
- ~~The face is not the game's~~ — **it is the family the game itself asks for**, which the reading above
  missed by looking only for a file. A game that ships no font still names a face: 妖々夢 hands
  `CreateFontA` `ＭＳ ゴシック` at weight 700 and bakes its strings through `TextOutA` into a DIB, and
  `ascii.anm` inside `th07.dat` is its panel's digits rather than a face anything is written in. So
  `Game::own_face` answers a family and a weight, `Font::of_the_machine` takes them, and orb's screens are
  in the same face as the game's own text. What is left of the price is a machine that does not have that
  family, which GDI substitutes for — and the log now names what came back beside what was asked for.
- `Face` now covers both kinds, and the drop tells them apart by the path it remembers: a face with no
  path had no `AddFontResourceExW` behind it, and removing a resource by an empty name would be a call
  about somebody else's font.
- **What a thcrap launch draws its text in is not measured**, and that is the one open thing here: thcrap
  patches the strings a game draws and can put a face of its own behind them, and then the game's text is in
  one face and orb's screens in the family the exe names. The answer that would need no measurement at all
  is to ask through the game's own `CreateFontA` import — a patch has already hooked it, so whatever the
  game gets is what orb would get — against which the cost is orb asking the *game's* GDI rather than the
  host's, which is a seam this repository draws the other way round
  ([0009](0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md)). Left until
  somebody plays a patched launch and sees the two faces side by side.

## What was weighed and not taken

- **Ship a font inside `orb.exe`.** It would give both games the same glyphs and needs no machine to have
  anything. What it costs is a licence decision that is the user's rather than this repository's, and
  megabytes in the one file a release hands out — and the machine's own face can be replaced by a shipped
  one later without changing anything above `overlay::loaded`.
- **Read a face out of `th07.dat`.** There is no face in it to read: 妖々夢's fonts are sprite sheets, so
  this would be orb rasterising from bitmaps of the game's own text at the game's own sizes, which is a
  renderer rather than a font.
- **Leave it as it was.** Then the midstage table landing would still leave 妖々夢 with a mode whose every
  screen is invisible, and the first launch of it would report a bug that is this decision unmade.
