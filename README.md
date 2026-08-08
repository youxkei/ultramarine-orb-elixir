# Ultramarine Orb Elixir

Chapter-based retry for 東方紅魔郷 (Touhou 6) 1.02h, in the style of 東方紺珠伝's
Pointdevice mode: the stage is divided into chapters, and dying sends you back to the start
of the chapter you were in rather than costing a life. Dying puts up a menu of three — the
chapter again, the stage again, or the run given up — and the two that cannot be taken back ask
before they act. The count of lives on the game's own panel is painted over with a brush
stroke reading `DISABLE` while such a run is on, the way 紺珠伝 marks its 残機 row, since nothing in
the run can lose one — the stars still show through where the ink is dry.

Shortened to *orb* in prose and in everything it installs.

Returning to a chapter restores a snapshot of the game's memory taken at the chapter's
start, rather than asking the game to jump somewhere. Nothing has to understand what a
boss's script was in the middle of.

**A chapter also survives the game being closed**, which the snapshot cannot: what the run has
pressed is written down at every chapter, and choosing a difficulty and character that a run was left
unfinished on then asks *どこから始める* — *つづきから* builds that stage again and plays those buttons
back into the chapter it was left in, with nothing of it drawn. Whatever ends the session leaves that
chapter — the retry menu's third item, the window closed, a crash — and finishing the run takes it
away. One run per difficulty, character and shot, the way 紺珠伝 keeps them.

Which of the two a run is, is asked where the game asks: choosing *Game Start*, *Extra Start* or
*Practice Start* puts a question over the title menu, pointdevice or normal. Normal is the game
as it was, and its scores go in the game's own `score.dat`; pointdevice runs are ranked among
themselves in `pointdevice_score.dat`, and *Score* asks which of the two to show.

orb does its own window — fullscreen or a size you pick — its own frame pacing, and removes a
frame of input lag by updating before drawing, so nothing else has to be loaded alongside it.

## Which games

**東方紅魔郷 1.02h** (`md5 fa3d64768b1bfc50703dedc2db92f7fa`) is everything above.

**東方妖々夢** (`md5 0126afce1e805370d36c3482445e98da`) is a game orb starts and stays out of. It
gets the window at the right shape and nothing else — not the frame pacing, not the frame of input lag,
nothing drawn, and none of the chapters, the retry menu, the run picked up again or the question that
chooses between the two modes. What has been read of that exe is a frame's worth of addresses and
nothing about a run, and orb answers nothing it has not read. There is no reason to start 妖々夢 through
orb yet.

The launcher looks for whichever of those two exes is in the directory it was pointed at and checks
its md5 before starting it, because every address orb uses was read off that exact build. A build it
does not know is refused with the list of the ones it does.

## Building and installing

The game is a 32-bit process, so both halves target `i686-pc-windows-gnu`, linked with
`i686-w64-mingw32-gcc`. `rust-toolchain.toml` selects the toolchain and the targets: nightly,
for artifact dependencies, which is how cargo builds the DLL first and hands its path to the
launcher, and beside what ships a 32-bit Linux target that nothing is built for — it is what
`cargo xtask seam` checks the crates above the seam against, below.

**The target is named by `cargo xtask` and not by a `[build] target`**, so the commands are
`cargo xtask <task>` rather than bare cargo ones. A task runner built for a 32-bit Windows target
cannot spawn the `cargo` that built it on a host that is not Windows, and a `build.target` would apply
to it too — see `crates/xtask/src/main.rs`. A bare `cargo check` therefore builds for the host and
fails, `orb` being full of `windows-sys`; **point your editor at the target instead** —
rust-analyzer's setting is `rust-analyzer.cargo.target`.

```sh
cargo xtask build --release
```

The tests are Windows binaries too, and this runs them: on Windows directly, and under WSL through
its interop, which runs an `.exe` the same way a shell there does.

```sh
cargo xtask test
```

That is also what installs the hooks. husky-rs points git at `.husky`, whose `pre-commit` is
`cargo fmt --check`, `cargo xtask clippy`, `cargo xtask seam` and `cargo xtask test` — a lint that is
wrong about this code is answered with an allow and the reason beside it, so the tree stays at zero
warnings and the next one to appear is about the change that made it. `NO_HUSKY_HOOKS=1` keeps a
build from touching git's
config at all, and `git commit --no-verify` skips the check for a commit that is not code.

`cargo xtask seam` is the boundary, asked for rather than kept by hand: `orb-core` and `orb-sim`
checked for a host with no Windows on it, so a crate above the seam that reaches past `orb-api` fails
to compile. A grep for `windows-sys` would not do — a COM vtable is Windows reached through a pointer
the game handed over, and no crate's name appears in it.

And `cargo xtask coverage` reports what the scenarios cover, which needs a toolchain of its own —
`cargo xtask` with no task says which and where to get it, and `crates/xtask/src/main.rs` says how to read
a report of it and why it is not the toolchain everything else is built with.

Then copy one file into the directory holding the game's exe:

| | |
| --- | --- |
| `target/i686-pc-windows-gnu/release/orb.exe` | the whole of orb: it carries `orb.dll` inside itself and unpacks it to `%TEMP%\orb` when it runs |

Start the game with `orb.exe`. Everything orb has to say goes to `orb.log` beside
it, the scores of runs it could rewind to `pointdevice_score.dat`, so the game's own
`score.dat` holds only runs anybody could have played, and the chapter each unfinished run was left
in to a file per run under `pointdevice_resume/`. To keep the launcher somewhere else,
`--game-dir=PATH` says where the game is.

Those last ones are MessagePack, with their own field names in them, so any msgpack-to-YAML or
msgpack-to-JSON converter prints one — which is the shape to hold the log's numbers against.

**The first launch asks for the settings** — how much of the screen the game gets, whether the
ending is shown, whether it keeps drawing while something else is in front, whether a chapter
beginning washes the play field — and writes the answers to `orb.yaml` beside the exe. The screen
is fullscreen or one of the window sizes this monitor can show, 4:3 or 16:9. The last of the
questions is whether to ask again next time; answer no and the game starts straight away, and
`orb --settings` asks anyway. Delete `orb.yaml` and everything is back to its default, including
being asked.

What is different every time it is run is an argument instead, and `orb --help` lists
them: `--collect` and `--judge` are the two passes over a replay that build a midstage chapter
table for a game orb does not have one for, `--clear` reaches an ending in a minute rather than
half an hour by letting nothing hit the player, and the rest are for looking into a fault.

## Documentation

| | |
| --- | --- |
| [SPEC.md](SPEC.md) | what orb does and the mechanisms it uses in their final form; configuration, tuning, and how the crates fit together |
| [TODO.md](TODO.md) | what is left, and what is built and still waiting on a run against the real game |
| `docs/todo/` | one file per piece of work too long for a paragraph in TODO.md: what was measured, how to measure it again, and the order to work through it. Deleted as the last step of the work, so the directory is there only while there is such a piece of work in flight — and there is none now |
| [docs/adr/](docs/adr/) | one file per decision about how the code is shaped, in the order they were taken, each opening with a status that says whether the tree looks like it yet |
