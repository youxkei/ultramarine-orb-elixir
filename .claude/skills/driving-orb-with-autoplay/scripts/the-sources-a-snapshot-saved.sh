#!/usr/bin/env bash
# What a launch's snapshots covered, one line per source: how many ranges it was in, how many bytes were
# committed, and the lowest base it sat at — read off the `memory: saving` lines of a verbose log.
#
# Why this exists: a cut of the restore bisect is named by a source (`--hold-back-at=heap1.3:0+200000`),
# and which sources there are to name is a launch's own answer. The line is also printed once per
# snapshot, and the same source is a different number of ranges in different snapshots as its committed
# holes come and go — so the interesting part is which sources appear at all and how big each is, and
# that is what this prints.
#
# usage: the-sources-a-snapshot-saved.sh <orb.log> [<snapshot ordinal>|all]
#        the ordinal counts `memory: saving` lines from 1; the last one is the default, and `all` prints
#        one block per snapshot.
#
# `all` prints a block apiece rather than one folded total, because a fold would be a number no snapshot
# ever held: the same region is more ranges in one snapshot than in another and adding them up says
# neither how big it is nor how big it got.
set -u
log="${1:?usage: the-sources-a-snapshot-saved.sh <orb.log> [<snapshot ordinal>|all]}"
which="${2:-last}"

# The launch and not the file: `orb.log` is one file across launches.
from=$(grep -an "new run ----" "$log" | tail -1 | cut -d: -f1)
saving=$(tail -n +"${from:-1}" "$log" | grep -a "memory: saving")
if [ -z "$saving" ]; then
  echo "no snapshot of this launch said what it covered; give the launch --log=verbose" >&2
  exit 1
fi

count=$(printf '%s\n' "$saving" | wc -l)
case "$which" in
  all) wanted=$(seq "$count") ;;
  last) wanted="$count" ;;
  *) wanted="$which" ;;
esac

echo "$count snapshot(s) in this launch; reading $which"
for at in $wanted; do
  line=$(printf '%s\n' "$saving" | sed -n "${at}p")
  if [ -z "$line" ]; then
    echo "there is no snapshot $at" >&2
    exit 1
  fi
  [ "$which" = "all" ] && echo "-- snapshot $at"
  printf '%s\n' "$line" |
    sed -E 's/^.*memory: saving //' |
    tr ' ' '\n' |
    # Each range arrives as two words — the source and `base+len` — so a source is remembered and spent
    # on the word after it.
    awk '
      /^[a-z]/ { source = $0; next }
      /\+/ {
        split($0, at, "+")
        bytes[source] += strtonum(at[2])
        ranges[source] += 1
        base = strtonum(at[1])
        if (!(source in lowest) || base < lowest[source]) lowest[source] = base
      }
      END {
        for (source in bytes)
          printf "%-12s %2d range(s) %#12x byte(s) from %#x\n", source, ranges[source], bytes[source], lowest[source]
      }
    ' |
    sort
done
