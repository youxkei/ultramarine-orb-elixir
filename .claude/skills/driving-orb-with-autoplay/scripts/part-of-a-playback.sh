#!/usr/bin/env bash
# Holds a run's own series against the playback that put it back, and names the first frame and the first
# field the two disagree about.
#
# A run written down writes `resume: frame N played with <fields>` on every frame of it and the playback
# that picks it up writes `resume: frame N played back with <fields>` on every frame of its own — the same
# fields either way. So the two are one table each, keyed by the frame, and what says why a playback
# stopped reproducing the run is the field that parted first. orb's own report names the generator and
# nothing else, which is one field of the twelve and says nothing about which of them moved first.
#
# Usage: part-of-a-playback.sh <the recording launch's log> <the playback launch's log> [<field>]
#
# **With a field named, every frame that field parts on is printed instead of the first frame's report.**
# The first parting is what says which field went first; it says nothing about a field that parts on one
# frame and agrees again three frames later, which is the shape a shot reaching an enemy late has — and
# following one field across a whole chapter is how a transient is told from a drift. The name is matched
# against the `key` of `key=value`, so `shot`, `lives`, `randoms` and `shot_at` each name one.
#
# `orb.log` is one file across launches, so copy the first launch's out before driving the second and hand
# this the two copies.
#
# **Only the last attempt of each log is read, and that is not a nicety.** A frame number here is the
# *stage's* own, so one launch writes the series over again for every stage it plays and for every attempt
# a retry rewinds — a recording of three stages with retries wrote 32,157 lines for a chapter 3,475 frames
# deep. Keyed by the frame alone, the table then holds whichever attempt of whichever stage came last, and
# holding *that* against a playback names a parting between two runs that were never the same run: it read
# `frame 9 parts: the run had input=0x0000, the playback has input=0x8101` where the attempt the chapter
# was actually written from agreed with the playback field for field. So each log is cut to its final
# block of ascending frames — the attempt that reached the chapter — before anything is compared.
set -eu

recorded=${1:?usage: part-of-a-playback.sh <recording log> <playback log> [<field>]}
playback=${2:?usage: part-of-a-playback.sh <recording log> <playback log> [<field>]}
field=${3:-}

fields() {
    # One line per frame: the frame number, then the fields with their names, from the newest launch in
    # the file. A launch is `---- new run ----`, and a log holding several is read from the last.
    local log=$1 what=$2 at
    at=$(grep -an -- "---- new run ----" "$log" | tail -1 | cut -d: -f1)
    tail -n +"${at:-1}" "$log" |
        grep -a -- "resume: frame .* $what with " |
        sed -E "s/^.*resume: frame ([0-9]+) $what with /\1 /" |
        # And only the last block of ascending frames, which is the attempt that reached the chapter: a
        # frame number that does not climb is a new stage or a retry starting the count over, and the
        # lines before it belong to a run this one is not.
        awk '
            $1 <= last { held = "" }
            { held = held $0 "\n"; last = $1 }
            END { printf "%s", held }
        '
}

fields "$recorded" "played" > /tmp/part-of-a-playback-recorded.$$
fields "$playback" "played back" > /tmp/part-of-a-playback-playback.$$
trap 'rm -f /tmp/part-of-a-playback-recorded.$$ /tmp/part-of-a-playback-playback.$$' EXIT

echo "$(wc -l < /tmp/part-of-a-playback-recorded.$$) frame(s) recorded, \
$(wc -l < /tmp/part-of-a-playback-playback.$$) played back"

# One field followed across the whole comparison, where one was named: every frame it parts on with both
# values, and the frames counted so that a transient reads as one and a drift as the rest of the chapter.
if [ -n "$field" ]; then
    awk -v want="$field" '
        function value(line, pairs, i, key) {
            split(line, pairs, " ")
            for (i in pairs) {
                key = pairs[i]
                sub(/=.*/, "", key)
                if (key == want) { sub(/^[^=]*=/, "", pairs[i]); return pairs[i] }
            }
            return "(none)"
        }
        NR == FNR { was[$1] = value(substr($0, length($1) + 2)); next }
        {
            frame = $1
            if (!(frame in was)) { next }
            now = value(substr($0, length($1) + 2))
            held++
            if (now == was[frame]) { next }
            parted++
            printf "frame %s: the run had %s=%s, the playback has %s=%s\n",
                frame, want, was[frame], want, now
        }
        END {
            printf "%s of %s frame(s) held in common part in %s\n",
                parted + 0, held + 0, want
        }
    ' /tmp/part-of-a-playback-recorded.$$ /tmp/part-of-a-playback-playback.$$
    exit 0
fi

# Joined on the frame, and the first frame whose fields differ is reported field by field. A field that is
# in one line and not the other is reported too, a game's answer having grown or shrunk between launches.
awk '
    NR == FNR { was[$1] = substr($0, length($1) + 2); next }
    {
        frame = $1
        now = substr($0, length($1) + 2)
        if (!(frame in was)) { next }
        if (now == was[frame]) { next }
        split(was[frame], before, " ")
        split(now, after, " ")
        printf "frame %s parts:\n", frame
        for (i = 1; i in before || i in after; i++) {
            if (before[i] != after[i]) {
                printf "  the run had %-28s the playback has %s\n", before[i], after[i]
            }
        }
        exit
    }
    END { if (!printed) { } }
' /tmp/part-of-a-playback-recorded.$$ /tmp/part-of-a-playback-playback.$$

# And where a field parts, whether the playback holds the run's own value on the frame either side of it:
# a playback drawing the same numbers a frame out of step is a different fault from one drawing different
# numbers, and orb's report cannot tell them apart.
awk '
    NR == FNR { was[$1] = substr($0, length($1) + 2); next }
    { now[$1] = substr($0, length($1) + 2) }
    END {
        for (frame in was) {
            if (!(frame in now) || now[frame] == was[frame]) { continue }
            if (now[frame + 1] == was[frame] || now[frame - 1] == was[frame]) {
                printf "frame %s: the playback holds the run\x27s whole line one frame %s\n",
                    frame, (now[frame + 1] == was[frame] ? "later" : "earlier")
            }
            break
        }
    }
' /tmp/part-of-a-playback-recorded.$$ /tmp/part-of-a-playback-playback.$$
