# 23. A launcher the wizard put somewhere else is read through the shell, and matched to the game by thcrap's own games.js

**Status:** accepted and built. `launcher::thcrap::installed` asks the game's directory first and then the
desktop and thcrap's own folder in the start menu; `read_link` resolves a `.lnk` through `IShellLinkW` and
`IPersistFile`, and `names_this_game` reads `config/games.js` to say which game a launcher in one of those
folders is for. This overturns the last section of
[0016](0016-the-game-is-handed-to-a-thcrap-installed-beside-it.md), which asked the game's directory and
nowhere else; everything else that decision settled stands.

## Context

thcrap's configuration tool asks where to write the launcher and takes four answers, which are not
exclusive — its wizard is four checkboxes, and each one that is ticked gets a launcher of its own. Read off
thcrap's source at `thcrap/src/shelllink.cpp`, with `<id>` the game id and `<cfg>` the run configuration:

| the answer | what it writes, and where |
| --- | --- |
| on the desktop | `<id> (<cfg>).lnk` in the desktop folder — a wrapper exe instead, on Wine |
| in the start menu | `<id> (<cfg>).lnk` under `Programs\thcrap`, which it creates |
| in the games' folder | `<id> (<cfg>).exe` beside the game's own exe |
| in the thcrap folder | `<id> (<cfg>).exe` in the directory thcrap is installed in |

The first two are the ones it ticks by default, and 0016 looked in neither: what it looked in was the
third, which is the one somebody has to ask for. So a player whose thcrap was installed the way the wizard
offers it started the game in Japanese and read `orb: thcrap — none installed where the game is` while
thcrap sat on their desktop.

0016 gave three reasons for asking only the game's directory. Two of them are the work rather than an
obstacle — a `.lnk` has to be resolved through COM, and the folders above hold a launcher per game rather
than this game's. The third is about a case this decision does not reach: an installation that left
nothing in any of these folders would need a path in `orb.yaml`, and a path is the one thing that file
holds none of.

## Decision

**The three folders a launcher can be found in are asked, in that order.** The game's directory first,
because a launcher there was installed to play *this* game with; then the desktop, then the start menu.
The fourth answer — a launcher inside thcrap's own directory — cannot be asked for: it is a launcher
inside the directory this is trying to find.

**A `.lnk` is resolved by the shell that wrote it.** `IShellLinkW::GetPath` is the loader it runs and
`GetArguments` is the command line, which is resources 0 and 1 of a wrapper exe in another form — minus
the loader's own name, which neither the run configuration nor the game id is read from anyway. The
alternative was reading the bytes of the file: a `.lnk` holds a target id list, a link info block and the
string data, and which of those wins is a question the program that wrote the file is never wrong about.
`windows-sys` carries Windows' functions and constants and not its COM interfaces, so the two vtables are
declared in `launcher::thcrap` in the shape `orb-api` declares Direct3D's — every slot in the interface's
own order, the ones nothing calls holding their places as `usize`.

**Which game a launcher is for is answered out of `config/games.js`.** The command line carries the game
id, that file maps the id to an exe, and the answer is whether that exe is the one orb is about to start —
compared through the file system rather than as two strings, since one of them has been through `..` and a
wizard's idea of a drive letter. This is the answer to 0016's objection that these folders hold every
game's launcher: they do, and thcrap says which is which.

**Only in those folders.** A launcher beside the game is not checked against `games.js`, which keeps the
behaviour 0016 built exactly as it was, and rests on the same argument: what somebody installed beside
this game is what they installed to play this game with.

**`games.js` is read for the one id rather than parsed.** What is wanted is one string out of a file of
`"th07": "../th07.exe"` lines. The `\u` escapes are taken off because the writer puts them there — the
configuration tool serialises with .NET's own encoder, which escapes everything outside ASCII, so
紅魔郷's exe is five of them in the file.

## What follows

- A thcrap installed on another drive from the game is found, and this is the arrangement the wizard
  produces by default: the path baked into a wrapper exe is relative only where a relative path can
  reach, and `games.js` holds an absolute path under the same condition, so both forms arrive at
  discovery and both are resolved.
- Two patch stacks in one folder are still read in the order the names sort, for the reason 0016 gave:
  a launch that picked a different one each time would be a translation that changed by itself.
- The desktop is read on Wine as well, where thcrap writes a wrapper exe there rather than a shortcut —
  its own comment says shortcuts do not work well there. Nothing extra was needed for it: the exe is read
  the way one beside the game is.
- What a launcher in thcrap's own directory would need is a path in `orb.yaml`, which is
  [0016](0016-the-game-is-handed-to-a-thcrap-installed-beside-it.md)'s third reason and still stands.
