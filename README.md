# Ultramarine Orb Elixir

Chapter-based retry for 東方紅魔郷 (Touhou 6) 1.02h, in the style of 東方紺珠伝's
Pointdevice mode: the stage is divided into chapters, and dying sends you back to the start
of the chapter you were in rather than costing a life. Dying puts up a menu of three — the
chapter again, the stage again, or the run given up — and the two that cannot be taken back ask
before they act.

Shortened to *orb* in prose and in everything it installs.

Returning to a chapter restores a snapshot of the game's memory taken at the chapter's
start, rather than asking the game to jump somewhere. Nothing has to understand what a
boss's script was in the middle of.

Which of the two a run is, is asked where the game asks: choosing *Game Start*, *Extra Start* or
*Practice Start* puts a question over the title menu, pointdevice or normal. Normal is the game
as it was, and its scores go in the game's own `score.dat`; pointdevice runs are ranked among
themselves in `pointdevice_score.dat`, and *Score* asks which of the two to show.

orb does its own window — fullscreen or a size you pick — its own frame pacing, and removes a
frame of input lag by updating before drawing, so nothing else has to be loaded alongside it.

Only 1.02h is supported (`md5 fa3d64768b1bfc50703dedc2db92f7fa`). The launcher checks the
exe before starting it, because every address orb uses was read off that exact build.

## Building and installing

The game is a 32-bit process, so both halves target `i686-pc-windows-gnu`, linked with
`i686-w64-mingw32-gcc`. `rust-toolchain.toml` selects the toolchain and the target: nightly,
for artifact dependencies, which is how cargo builds the DLL first and hands its path to the
launcher.

```sh
cargo build --release
```

The tests are Windows binaries too, and `cargo test` runs them: on Windows directly, and under
WSL through its interop, which runs an `.exe` the same way a shell there does.

```sh
cargo test
```

That is also what installs the hooks. husky-rs points git at `.husky`, whose `pre-commit` is
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` — a lint that is
wrong about this code is answered with an allow and the reason beside it, so the tree stays at zero
warnings and the next one to appear is about the change that made it. `NO_HUSKY_HOOKS=1` keeps a
build from touching git's
config at all, and `git commit --no-verify` skips the check for a commit that is not code.

Then copy one file into the directory holding `東方紅魔郷.exe`:

| | |
| --- | --- |
| `target/i686-pc-windows-gnu/release/orb.exe` | the whole of orb: it carries `orb.dll` inside itself and unpacks it to `%TEMP%\orb` when it runs |

Start the game with `orb.exe`. Everything orb has to say goes to `orb.log` beside
it, and the scores of runs it could rewind to `pointdevice_score.dat`, so the game's own
`score.dat` holds only runs anybody could have played. To keep the launcher somewhere else,
`--game-dir=PATH` says where the game is.

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
| [SPEC.md](SPEC.md) | what orb does and the mechanisms it uses, with the measurements behind them; configuration, tuning, and how the crates fit together |
| [DONE.md](DONE.md) | what works, and how it was checked |
| [TODO.md](TODO.md) | what is left |
