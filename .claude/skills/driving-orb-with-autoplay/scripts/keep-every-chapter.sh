#!/usr/bin/env bash
# **Every chapter a launch writes into `pointdevice_resume/`, kept as it is written**, so that a later launch can
# pick a run up at any chapter it passed and not only at the one it ended in.
#
#   keep-every-chapter.sh <directory the runs are kept under> <directory to keep the chapters in> <orb.log>
#
# orb keeps one file a run, `pointdevice_resume/<slot>.msgpack`, and writes it over at each chapter it writes
# down: what a launch leaves behind is the chapter it ended in, or nothing where the run was cleared. A card
# that wants measuring was then reached by playing the whole run to it again — three quarters of an hour of a
# session for the Extra's fifth card — where a pick-up of the chapter before it plays that stage's own frames
# back in and is at the card in a minute.
#
# So this copies the file out each time it changes, named for the newest chapter `orb.log` says was written:
# `<slot>/s<stage>-c<chapter>-<the chapter's name>-f<frame>.msgpack` under the directory named, or
# `s<stage>-begins` for the write a stage's build makes before its first chapter is taken. A launch that
# picks one up is given a `--keep-in=` of its own with the file put in its `pointdevice_resume/`, so the kept
# one is never written over.
set -u

usage="usage: keep-every-chapter.sh <directory the runs are kept under> <directory to keep the chapters in> <orb.log>"
watch="${1:?$usage}"
keep="${2:?$usage}"
log="${3:?$usage}"
[ -d "$watch" ] || { echo "$usage — $watch is not a directory"; exit 2; }
[ -f "$log" ] || { echo "$usage — $log is not a file"; exit 2; }
mkdir -p "$keep"

declare -A seen
while true; do
  while IFS= read -r file; do
    stamp=$(stat -c '%s %Y' "$file" 2>/dev/null) || continue
    [ "${seen[$file]:-}" = "$stamp" ] && continue
    seen[$file]=$stamp
    slot=$(basename "$file" .msgpack)
    # Eight megabytes, since a verbose log writes a `resume: frame N played with …` line every frame: at the
    # rate a whole-run case plays, the two seconds between two looks are some two hundred kilobytes of them.
    chapter=$(tail -c 8000000 "$log" \
      | grep -a -o -E "resume: stage [0-9]+ (chapter [0-9]+ \([^)]*\) at frame [0-9]+|begins)" \
      | tail -1 \
      | sed -E 's/^resume: stage ([0-9]+) chapter ([0-9]+) \(([^)]*)\) at frame ([0-9]+)$/s\1-c\2-\3-f\4/;
                s/^resume: stage ([0-9]+) begins$/s\1-begins/; s/ /-/g')
    mkdir -p "$keep/$slot"
    cp "$file" "$keep/$slot/${chapter:-unnamed-$(date +%s)}.msgpack"
    echo "$(date +%T) kept $slot ${chapter:-unnamed}"
  done < <(find "$watch" -path '*pointdevice_resume/*.msgpack' 2>/dev/null)
  sleep 2
done
