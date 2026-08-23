---
name: driving-orb-with-autoplay
description: How to make a game orb supports play itself and watch it — the copy to drive, the one game and one monitor at a time, the state said every 30 seconds, when to close a run that has stopped getting anywhere, and what a recipe's own grammar means. Use whenever a task in ultramarine_orb_elixir needs the real game run without a person at the keyboard: confirming a fix on the real thing, recording a resume file, holding two launches against each other, or measuring anything that takes a run to reach.
---

# Driving orb with autoplay, and watching what it does

`--autoplay=<recipe>` makes orb play a run by its own rules to a point a recipe names, so a measurement that
needs a stage reached does not need somebody at the keyboard for six minutes.

## A copy of the install, and never the install

**Drive a copy of the install, and not the install.** A played game writes its records beside itself —
`score.dat`, the replays, the resume directory, `orb.log` — and a bot in the directory somebody plays in
overwrites them. The copy needs the game's own files and wants none of the `replay*/` directories.

Then `cargo xtask build --release` and `bash scripts/deploy.sh <copy>`. Deploy keeps that copy's `orb.yaml`,
so settings put there survive a redeploy.

## One game, and one monitor

**Nothing in 東方 stops a second copy of itself starting, and two of them fight over the same files.** So
before a launch, ask whether one is running and close it if it is. The exe to ask about is the game's own and
not orb's — `game::KNOWN` in `orb-core` is the list of them — and the filter is what to ask with:

    tasklist.exe /FI "IMAGENAME eq <the game's exe>" 2>/dev/null | grep -c "<the game's exe>"

Empty output means none — **once**. It is a Windows process reached through the interop, and it answered
nothing twice in one afternoon while the game was up and playing, so a watch or a wait that acts on one empty
answer acts on a game that is still running. Ask again before believing it.

**A script that refuses is worth more than remembering to ask.** `orb.exe` returns as soon as it has
injected, so what makes one invocation one run is waiting for the game to appear and then waiting for it to
go. Written out here because `/scripts` is ignored by this repository, and taking what varies as arguments for
the reason `CLAUDE.md` gives:

```sh
#!/usr/bin/env bash
# One launch of a copy that plays itself, and nothing else.
set -u
dir="${1:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml>}"
exe="${2:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml>}"
recipe="${3:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml>}"

running() { tasklist.exe /FI "IMAGENAME eq $exe" 2>/dev/null | grep -c "$exe"; }

# **Asked twice, because one empty task list is not a game that has gone.** `tasklist.exe` is a Windows
# process reached through the interop and it answered nothing once while the game was up and playing, which
# ended a watch on the launch it was watching a minute into a twenty-five minute run.
gone() {
  [ "$(running)" -eq 0 ] || return 1
  sleep 2
  [ "$(running)" -eq 0 ]
}

if ! gone; then
  echo "refusing: $exe is already running"
  exit 1
fi

cd "$dir" || exit 1
./orb.exe "--autoplay=$recipe"
echo "injected, waiting for the game to come up"

for _ in $(seq 60); do
  [ "$(running)" -gt 0 ] && break
  sleep 1
done
if [ "$(running)" -eq 0 ]; then
  echo "the game never came up"
  exit 1
fi
echo "up, waiting for it to go"

while ! gone; do
  sleep 5
done
echo "the run is over"
```

Run it in the background: it does not return until the run is over, and the launcher's own lines —
`orb: thcrap — …`, `orb: started … (pid N)` — land in its output rather than in `orb.log`.

**And close the monitor of a run that is over** — `TaskStop` — before starting the next. Two monitors on one
log both fire, and the older one's filter was written for the launch before this one.

## Watching it: every 30 seconds, and what to say

Watch with `Monitor` over the copy's `orb.log`, and **say the state every 30 seconds** rather than only when
something changes: a run whose chapter has not changed for two minutes is either a long spellcard or a run
that is stuck, and the two are indistinguishable while nothing is said. What is worth one line:

- how many hits and how many deaths the run has had
- which recipe step it is on — `autoplay: step N of M …`
- where the run is — the newest `stage S chapter C at frame F` line
- and, on a pickup, the landing: `resume: the landing is …`

**Cut the log to the launch.** `orb.log` is one file across launches and is sometimes rotated, so the newest
launch is what follows the last `---- new run ----` and no offset written down beforehand can be trusted:

    from=$(grep -an "new run ----" "$log" | tail -1 | cut -d: -f1)
    cut=$(tail -n +"$from" "$log")

**A monitor's own end condition is cut the same way.** Matched against the whole file, the previous launch's
closing line ends the monitor the moment it starts.

## When to close a run

**Stalled** is the log not growing. Fetch `wc -c` on it each tick and count the ticks it has not moved for.

**Not stalled**: a chapter that stays the same while `frames=` climbs, a boss's own attack running thousands
of frames; and `script=` standing still while `frames=` climbs, which is what a boss attack does to the
stage's script clock. The newest `f<N> scene=…` line tells them apart:

    tail -n +"$from" "$log" | grep -aE "^\[.*\] f[0-9]+ scene" | tail -1

**Repeating the same chapter** is a run dying and retrying it, and `died in chapter N` and `hit while` are the
lines for it. A run with neither, whose frames climb, is fine however long it sits in one chapter.

Where it really has stopped: close the game, and say what the log said before it. A recipe's own
`give_up_after` ends a step waiting for something that will never happen — `autoplay: step N of M gave up` —
but nothing bounds a game that has gone quiet.

## The recipes

A recipe is a list of steps read by the game's own process, so it lives **beside that copy's exe** and not in
this repository. `orb-config`'s `recipe` module is the grammar and `Until` is what a step can wait for.

**The front end is walked by pressing toward orb's own question, not by tapping through screens.** A screen
and an item — `{ screen: 8, cursor: 1 }` — are numbers that depend on where the score file left the cursors
and on whether orb's question is standing in front of them; the question is reached whatever the screens did.

    - press: [decide]
      frames: 30
      until: { question: resume }
      give_up_after: 1800

**Held for thirty frames rather than tapped**, because orb's question is answered on an *edge*: a key already
down cannot answer it, and a tap lets go and presses again inside the grace frames, which answers it. Then
`- wait_until: { frames: 20 }` for `mode::INPUT_GRACE_FRAMES`, which swallows a press inside them.

**A tap overshoots its target by one press, so `until` cannot land the cursor on an item.** The press for the
frame the condition became true has already gone in, and no wait afterwards takes it back: a step that walked
the title's cursor onto Run pressed `up` once more onto the item beside it — 終了 — and the `decide` after it
closed the game. Aim a cursor with `until` only where the item after the target is harmless.

**And a question raised *by* the press that would answer it cannot be photographed by a recipe at all.** orb's
mode question comes up on a `decide` at the title's Run: waiting for it presses nothing and it never comes,
tapping toward it answers it the frame it appears, and walking the cursor there is the overshoot above. The
questions a recipe can hold still are the ones something *else* raises — picking a run up, and where to start
again.

**A picture is asked for before the press that answers what it is of.** A question answered is a question gone,
so a shot after the press is a shot of whatever came next.

**And a picture holds what the game drew, not everything on the screen.** `shot` copies the back buffer at the
end of the frame's drawing, so a question standing over the *front end* is in it — orb's own words and all —
and a question standing over a **held stage** is not: the frame written out is the stage, with the panel's
`Player` row marked and the rate collapsed to a few frames a second, and none of the question's words. Waiting
longer does not help; it is where the words are drawn and not when.

**A picture asked for before the game has drawn anything is one flat colour.** A `screenshot` as a recipe's
first step copies a back buffer nothing has presented into — a few hundred bytes of one colour once encoded.
Wait for the screen to be up first, by a count of frames where there is nothing else to wait for.

**What the game itself draws is always in the picture**, which is what makes a translation patch checkable: a
stage, its panel, a card's name plate and the title menu all come out, so the same recipe run with the patch
beside the game and without it is the check that the patch reached them.

**Nothing that only waits belongs in a run that is being played.** A `wait_until` hands the game a word of
nothing, so the rules get no frame and the player stands still until something shoots it — and each death
raises orb's retry menu, which the recipe's own next press then answers. Between two things a recipe does to a
running stage, the step is `play_until: { frames: N }`.

**The run is chosen and not fallen into.** Picking a saved run up is the item that question comes up on, so
starting from nothing is `- press: [down]` for two frames first.

**`boss: true` is the fight the stage ends with, not the midboss.** It is
`Chapters::the_stages_own_boss_is_up`, a fight with a song of its own, and a midboss fights under the stage's
own song. The grammar also refuses `boss` beside a `stage`, so a particular stage's fight is two steps:

    - play_until: { stage: 3, chapter: 1 }
      give_up_after: 72000
    - play_until: { boss: true }
      give_up_after: 36000

**A fight has no chapter for a moment after that flag rises**, `Chapters::due` waiting for the music, so a run
left the instant it rises is written down at the chapter in front of the fight. Fight it for a moment first:
`- play_until: { frames: 600 }`.

**A conversation, a card and a card's name are waited for and never counted** — `dialogue: true`, `card: true`
and `card_name: true`. The rules press past a line of dialogue as fast as the game takes presses, so a
conversation is up for a handful of frames; and which chapter of a stage is a card is the game's own numbering,
differing stage by stage. Six pictures at hundred-frame intervals through a midsection caught the stage every
time and the conversation never, and twelve taken at chapters read off one afternoon's logs caught no card plate
at all.

**And the card and its name are two conditions because they are two frames.** A card starts and the plate its
name is on is put up after that, so a picture asked for on `card: true` has the boss's name and the timer in it
and nothing between them. `card_name: true` is the plate itself — for 妖々夢, the bit the engine's own draw reads
before it draws that vm.

**A picture asked for on the frame a screen is reached is a picture of the screen before it.** The front end
draws a change over several frames, so a `screenshot` straight after `until: { screen: N, cursor: M }` comes out
mid-fade with the outgoing screen still legible over the new one — three shots of three menus each showed the
menu before. **The same holds for anything a run draws with an animation of its own**: a picture on the frame
`card: true` became true came out with the boss's name and the timer in place and the card's own plate not yet
drawn.

**So a state to wait for and the frames a drawing takes are both needed, and neither replaces the other.** The
condition is what gets a step to the right moment of the run without counting a stage's frames; the frames are
what let the thing that moment is about be drawn. Measured on 1面's first card: the plate is put up two or three
frames after `card: true`, `card_name: true` catches it as a shape with nothing readable on it, and thirty frames
on it reads `Cold Sign "Lingering Cold"`.

    - play_until: { card_name: true }
      give_up_after: 3600
    - screenshot: the card's own name
      frames: 30

**`leave: true` is the pause menu walked to the title**, which is how a player leaves and so how a resume file
has to be written. Give the game its own half of leaving before the process goes —
`- wait_until: { frames: 300 }` — or the next launch reads a run this one had not finished leaving.

**Every step that can wait for something that will never happen says `give_up_after`.** A run that dies before
the stage it was told to reach leaves orb's retry menu up, which the rules do not press.

## Measuring across launches

A pair of launches is the shape most measurements take: one that plays to a point and leaves, one that picks
it up. **Keep the first launch's log before the second overwrites or rotates it** —
`cat "$log" > <scratchpad>/first.log` — because what a comparison reads is the two launches' lines side by
side.

Where several stages have to be measured, chain them: each launch picks the run up, answers for the stage it
picked up in, plays on to the next stage's fight and leaves it there.
