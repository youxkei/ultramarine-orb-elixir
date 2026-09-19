#!/usr/bin/env bash
# **A picture of a game orb has no drawing hook in, taken once per pause a recipe holds** — th06c, th06nc.
#
#   shoot-the-pauses.sh <copy dir> <game-exe> <first wait step> <steps apart> <label>...
#
# A recipe that wants a picture of such a game at some point of a run pauses it there with the game's own
# `menu`, waits a `wait_until: { frames: N }` long enough for a look, and lets it go again. This watches the
# launch's trace and, the first time that wait is under way, takes the window with `shot.ps1` into
# `<copy dir>/shots/<label>.png` — the first wait being step <first wait step> and each one after it
# <steps apart> further on, one label apiece. The trace and not the log, for the reason `watch-a-run.sh` gives.
#
# Run it in the background beside the launch; it exits once every label has a picture.
set -u
usage="usage: shoot-the-pauses.sh <copy dir> <game-exe> <first wait step> <steps apart> <label>..."
dir="${1:?$usage}"
exe="${2:?$usage}"
first="${3:?$usage}"
apart="${4:?$usage}"
shift 4
if [ "$#" -eq 0 ]; then
  echo "$usage"
  exit 2
fi
labels=("$@")
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
reader="$repo/target/i686-pc-windows-gnu/release/orb-real.exe"
mkdir -p "$dir/shots"
out="$(wslpath -w "$dir/shots")"
declare -A taken
count=0
while [ "$count" -lt "${#labels[@]}" ]; do
  trace="$(find "$dir/traces" -name '*.msgpack' -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
  if [ -n "$trace" ]; then
    line="$("$reader" watching "$(wslpath -w "$trace")" 2>&1 | grep -aoE 'autoplay: step [0-9]+ of [0-9]+ [a-z]+' | head -1)"
    n="$(printf '%s' "$line" | awk '{print $3}')"
    # The wait is under way once the press before it is done, or while it says it is still waiting.
    if [ -n "$n" ]; then
      if [ "$(printf '%s' "$line" | awk '{print $6}')" = "done" ]; then at=$((n + 1)); else at=$n; fi
      i=$(((at - first) / apart))
      if [ $(((at - first) % apart)) -eq 0 ] && [ "$i" -ge 0 ] && [ "$i" -lt "${#labels[@]}" ] && [ -z "${taken[$i]:-}" ]; then
        # A moment for the pause menu to finish coming up, which is a fade of its own.
        sleep 2
        powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w "$here/shot.ps1")" \
          -Exe "$exe" -Out "$out\\${labels[$i]}.png" 2>&1 | tr -d '\r'
        taken[$i]=1
        count=$((count + 1))
      fi
    fi
  fi
  sleep 1
done
echo "all $count taken"
