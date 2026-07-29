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

Then copy two files into the directory holding `東方紅魔郷.exe`:

| | |
| --- | --- |
| `target/i686-pc-windows-gnu/release/orb-launcher.exe` | the whole of orb: it carries `orb.dll` inside itself and unpacks it to `%TEMP%\orb` when it runs |
| `orb.yaml` | only if there is not one there already, since it is where local settings live |

Start the game with `orb-launcher.exe`. Everything orb has to say goes to `orb.log` beside
it.

## Documentation

| | |
| --- | --- |
| [SPEC.md](SPEC.md) | what orb does and the mechanisms it uses, with the measurements behind them; configuration, tuning, and how the crates fit together |
| [DONE.md](DONE.md) | what works, and how it was checked |
| [TODO.md](TODO.md) | what is left |
