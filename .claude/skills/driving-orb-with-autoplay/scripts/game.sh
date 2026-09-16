#!/usr/bin/env bash
# The three questions anybody driving a copy of a game asks Windows about it: whether it is running,
# whether it has gone, and closing it.
#
#   game.sh running <game-exe>   how many of it are running, as a number on stdout
#   game.sh gone <game-exe>      exit 0 where none is, after asking twice
#   game.sh close <game-exe>     close it, and say so in this shell's own words
set -u

what="${1:?usage: game.sh running-or-gone-or-close <game-exe>}"
exe="${2:?usage: game.sh running-or-gone-or-close <game-exe>}"

# Counted by the CSV row and not by the name, and with `wc -l` and not with `grep -c`: an image name comes
# back in the console code page, so a grep for 東方紅魔郷.exe matches no row while the filter handed to
# `tasklist.exe` is Windows' own and does; and `grep -c` prints nothing where nothing matched, which reaches
# `[ -eq ]` as an empty string.
#
# `-a`, because that code page is not this shell's: without it grep reads the row as binary and prints
# `binary file matches` to stderr and no line, so the count is nought while the game is up.
running() { tasklist.exe /FI "IMAGENAME eq $exe" /FO CSV /NH 2>/dev/null | grep -a '^"' | wc -l; }

# Asked twice: `tasklist.exe` is reached through the interop and answers an empty list once in a while with
# the game up, so one nought is not a game that has gone.
gone() {
  [ "$(running)" -eq 0 ] || return 1
  sleep 2
  [ "$(running)" -eq 0 ]
}

case "$what" in
  running) running ;;
  gone) gone ;;
  # Said here rather than passed through: `taskkill.exe` answers in the console code page too, so its own
  # line arrives as mojibake. The exit status is what it is read by.
  close)
    if gone; then
      echo "$exe was not running"
      exit 0
    fi
    if taskkill.exe /F /IM "$exe" >/dev/null 2>&1; then
      echo "$exe closed"
    else
      echo "$exe would not close"
      exit 1
    fi
    ;;
  *)
    echo "usage: game.sh {running|gone|close} <game-exe>"
    exit 2
    ;;
esac
