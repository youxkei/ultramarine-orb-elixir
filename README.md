# Ultramarine Orb Elixir

Chapter-based retry for 東方紅魔郷 (Touhou 6) 1.02h, in the style of 東方紺珠伝's
Pointdevice mode: the stage is divided into chapters, and dying sends you back to the start
of the chapter you were in rather than costing a life.

Shortened to *orb* in prose and in everything it installs.

Returning to a chapter restores a snapshot of the game's memory taken at the chapter's
start, rather than asking the game to jump somewhere. Nothing has to understand what a
boss's script was in the middle of.

orb does its own borderless fullscreen, its own frame pacing, and removes a frame of input lag
by updating before drawing, so nothing else has to be loaded alongside it.

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
`cargo fmt --check` and `cargo test` — `NO_HUSKY_HOOKS=1` keeps a build from touching git's
config at all, and `git commit --no-verify` skips the check for a commit that is not code.

Then copy one file into the directory holding `東方紅魔郷.exe`:

| | |
| --- | --- |
| `target/i686-pc-windows-gnu/release/orb.exe` | the whole of orb: it carries `orb.dll` inside itself and unpacks it to `%TEMP%\orb` when it runs |

Start the game with `orb.exe`. Everything orb has to say goes to `orb.log` beside
it, and the scores of runs it could rewind to `orb_score.dat`, so the game's own `score.dat`
is left as it was. To keep the launcher somewhere else, `--game-dir=PATH` says where the game
is.

With no `orb.yaml` beside it, every setting is its default: borderless fullscreen, the ending
run out without being shown, no replay written, and the scores in orb's own file. Copy the
`orb.yaml` in this repository there to change one of those — the file is a list of the defaults
with what each is for, so a key left as it stands says the same thing as no key at all.

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
