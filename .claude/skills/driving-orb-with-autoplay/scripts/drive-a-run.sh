#!/usr/bin/env bash
# One launch of a copy that plays itself, and nothing else: refuses while the game is running, launches
# orb with the recipe, waits for the game to come up and then to go.
#
# What it asks Windows about the game is `game.sh`'s.
set -u
dir="${1:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml> [orb option...]}"
exe="${2:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml> [orb option...]}"
recipe="${3:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml> [orb option...]}"
shift 3

here="$(cd "$(dirname "$0")" && pwd)"
game() { bash "$here/game.sh" "$1" "$exe"; }

if ! game gone >/dev/null; then
  echo "refusing: $exe is already running"
  exit 1
fi

# **And a trace, always**, unless one was named: a launch is watched and read afterwards through its own trace
# (`orb-real watching`, `orb-real show`), and a launch without one leaves only `orb.log`, which the next launch
# starts over. Beside the copy under `traces/`, by the time and the recipe, spelt the Windows way — `--trace`
# is a path orb opens from inside the game.
trace=""
for arg in "$@"; do
  case "$arg" in --trace=*) trace="${arg#--trace=}" ;; esac
done
if [ -z "$trace" ]; then
  mkdir -p "$dir/traces"
  trace="$(wslpath -w "$dir/traces")\\$(date +%Y%m%d-%H%M%S)-$(basename "$recipe" .yaml).msgpack"
  set -- "$@" "--trace=$trace"
fi
echo "trace: $trace"

cd "$dir" || exit 1
./orb.exe "--autoplay=$recipe" "$@"
echo "injected, waiting for the game to come up"

# `orb.exe` returns as soon as it has injected, so the game's own process is what one launch is waited on by.
for _ in $(seq 60); do
  [ "$(game running)" -gt 0 ] && break
  sleep 1
done
if [ "$(game running)" -eq 0 ]; then
  echo "the game never came up"
  exit 1
fi
echo "up, waiting for it to go"

while ! game gone >/dev/null; do
  sleep 5
done
echo "the run is over"
