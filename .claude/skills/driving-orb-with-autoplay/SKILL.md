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

**And again after the fix the run is meant to check.** The exe in the copy is whatever was deployed last: a
pick-up driven against an exe deployed four hours earlier read the file the way the build *without* the fix
reads it, said so, and the run it then started wrote over the file the measurement was about.

**What says which build a launch is, is the commit in its first line**:

    orb 0.11.0-dev attached to pid 45320, built from 90b82bc364b3

`cargo xtask` puts the working copy's own commit there as it builds, so the id moves with the tree and
answers *is this the build with the fix* — hold it against `jj log -r @ -T commit_id.short(12)` before
reading anything else off the run. The version alone answers nothing: `0.11.0-dev` is every build between two
releases. A launch saying `built outside cargo xtask, so no commit is named` was built some other way and
there is nothing to hold it against.

## One game, and one monitor

**Nothing in a Touhou game stops a second copy of itself starting, and two of them fight over the same files.** So
before a launch, ask whether one is running and close it if it is. The exe to ask about is the game's own and
not orb's — `game::KNOWN` in `orb-core` is the list of them.

**Every one of those three questions is `scripts/game.sh`'s, and none of them is to be typed out again:**

    bash scripts/game.sh running <the game's exe>   # how many are running, as a number
    bash scripts/game.sh gone <the game's exe>      # exit 0 where none is
    bash scripts/game.sh close <the game's exe>     # close it, and say so readably

**Written down as a script because the same three traps were walked into session after session**, each of
them a `tasklist.exe` invocation that answers *wrongly* rather than failing:

- **A row per process, counted, and never a grep for the name**: an image name comes back in the console
  code page, so a grep for 東方紅魔郷.exe finds a running game never — the filter the command is given is
  Windows' own and matches. And `grep -c` answers nothing at all where nothing matched, which reaches
  `[ -eq ]` as an empty string and reads as a game still running.
- **`grep -a`, or the count is nought while the game is up.** That code page is not this shell's, so the
  row comes back with bytes grep reads as binary and it prints `binary file matches` to stderr rather than
  the line. Measured: a launch that had already injected, whose window was on the screen, was watched for
  sixty seconds and then declared never to have come up.
- **Nought means none once and not twice.** It is a Windows process reached through the interop, and it
  answered nothing twice in one afternoon while the game was up and playing, so a watch or a wait that acts
  on one answer of nought acts on a game that is still running. `gone` asks again before believing it.

**And `taskkill.exe`'s own answer is unreadable here** — `���: �v���Z�X "th06c.exe"` — so `close` says what
it did in this shell's words instead. It is also a thing to ask about rather than to do to a game somebody
is playing: what it is for is a launch of orb's own that has finished or gone wrong.

**A script that refuses is worth more than remembering to ask.** `orb.exe` returns as soon as it has
injected, so what makes one invocation one run is waiting for the game to appear and then waiting for it to
go. That is `scripts/drive-a-run.sh`, which asks `game.sh` the questions above:

    bash scripts/drive-a-run.sh <game-dir> <game-exe> <recipe.yaml> [orb option...]

Both are written out below as well, because `/scripts` is ignored by this repository and a copy that is in
no commit is a copy one `rm` away from being gone:

```sh
#!/usr/bin/env bash
# One launch of a copy that plays itself, and nothing else.
set -u
dir="${1:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml>}"
exe="${2:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml>}"
recipe="${3:?usage: drive-a-run.sh <game-dir> <game-exe> <recipe.yaml>}"

# **Counted by the CSV row and not by the name, and with `wc -l` and not with `grep -c`.** th06's image name
# comes back in the console code page, so a grep for 東方紅魔郷.exe finds a running game never — the filter
# itself is Windows' own and matches. And `grep -c` answered nothing at all where nothing matched, which
# reaches `[ -eq ]` as an empty string and reads as a game still running.
#
# **And `-a`, which is the whole of whether this works at all.** That code page is not this shell's, so the
# row comes back with bytes grep reads as binary: without it grep prints `binary file matches` to stderr and
# no line, so the count is nought while the game is up. Measured — a launch that had already injected, whose
# window was on the screen, was watched for sixty seconds and then declared never to have come up.
running() { tasklist.exe /FI "IMAGENAME eq $exe" /FO CSV /NH 2>/dev/null | grep -a '^"' | wc -l; }

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

## And `--pace=8`, which is what a launch costs

**A launch of a whole run is twenty-eight minutes of game time**, and `--pace=N` is what turns that into
minutes of wall clock: N game frames to a sixtieth of a second, with everything orb does per frame left
exactly as it is. Eight held `2.1ms` a frame on this machine, which is a Normal run of th06c in four
minutes and an Extra attempt in two.

**It was two for a long while and the cap was the present and not the pacing.** `--pace=4` and `--pace=8`
both came out at `8.3ms` a frame — one refresh of a 120Hz display — and what moved between them was only
where the waiting was: `wait 3869us` with `draw 3512us` at four, and `wait 199us` with `draw 7421us` at
eight. The interval a game asks its present for *is* the display pacing it, so a launch asking for frames
faster than the display and a present that waits for a blank are two answers to one question.
`orb_core::dxgi` drops the interval while `frame::Pacing` is pacing faster than one, which is what makes
the ask reachable on a game that presents through DXGI at all.

**What a paced launch does not change is the run**, the frames being the same frames; what it does change
is that a monitor polling every thirty seconds is polling every two minutes of game time. A screen that
stands for a few seconds — a ranking, the frame a run hands over on — wants a two-second poll, and the
state line every fifteen of those.

**And nothing after the run belongs in the recipe.** A `play_until` ends where the run does, which is past
both the ending and the name the game asks for, so a `wait_until` after it is waiting for nothing: one
recipe's 10,800 frames there spent ninety seconds at the logo screen, which starts the attract demo at 720
of them and filled the log with a playback nobody had asked for.

## Every launch says `--log=verbose`

**A launch is verbose, whether it is a recipe's or somebody's own game** — `./orb.exe --log=verbose`, and for
the script above, `./orb.exe "--autoplay=$recipe" --log=verbose`. What `normal` leaves out is the per-frame
lines, and those are the ones that say what orb did *to* the run: the word handed to the read each frame, the
frames a pick-up plays its buttons back in, the chapters `take_hits` passed over, and the sound's geometry
either side of a restore.

**The reason is that a symptom is not something to be reproduced on request.** A run of th07 picked up and
played to stage 6 was seen to move on its own after one chapter retry, as if another replay were playing;
the launch had been left at `normal`, and what its log could then answer was only which explanations were
*not* it — no `replay` or `demo` in the frame lines' flags, so not the game's own playback, and no `resume:`
line after the retry, so not orb starting one. What was left needed the frame lines, the run was over, and
the route to it is twenty-five minutes of play.

**And the cost of verbose is the log's size and nothing else.** It is written beside the exe, a run of a few
minutes is megabytes rather than hundreds of kilobytes, and `orb.log` is one file the next launch appends to
— so a measurement worth keeping is copied out to the scratchpad while the game is still up, which
`## Measuring across launches` says anyway.

## Watching it: every 30 seconds, and what to say

Watch with `Monitor` over the copy's `orb.log`, and **say the state every 30 seconds** rather than only when
something changes: a run whose chapter has not changed for two minutes is either a long spellcard or a run
that is stuck, and the two are indistinguishable while nothing is said.

**And nothing a monitor says arrives while a tool call is blocking**, which is the whole of whether it is
being read. Its events land between calls; a `TaskOutput` waiting ten minutes on the launch, or any other
long wait, holds every one of them back and then delivers the lot at once when the turn resumes — so the
run is unwatched for exactly as long as the wait. Measured: a launch whose game over sat behind a continue
prompt ran on for ten minutes with the monitor printing all of it into a queue nobody could see, and what
ended the wait was somebody asking why the game was stuck. **So check with short commands between
actions** — `wc -c` on the log, the newest `run:` line — and never block on the launch itself; there is
always other work while a run plays, and the events arrive in the gaps.

**That monitor is armed with the launch and not when somebody asks for it**, and a monitor that only greps
for the lines a measurement is about is not it: a run reported on every thirty seconds is one whose stall,
whose deaths and whose recipe step are all visible while it happens, where an event filter says nothing for
minutes and nothing is what a stuck run says too. Both together is fine — the filter for the lines the
measurement turns on, the poll for the state — but the poll is the one that cannot be left out.

**And a filter whose last stage is `cut` or `head` delivers nothing**: only `grep --line-buffered` flushes per
line, so a `... | grep --line-buffered <pattern> | cut -c1-170` sits in `cut`'s buffer and the monitor stays
silent through the very lines it was armed for. Trim inside the loop with `sed`'s own substitution, or put the
`cut` before the grep, or leave the lines untrimmed.

**Nor does `tail -f` on this log fire at all.** It is on a Windows drive through the interop, where inotify
gets nothing, so `tail -n 0 -f "$log" | grep --line-buffered …` reports not one line however much is
appended — measured on a hand-played run whose log grew by 2,590 lines with that monitor armed and silent
throughout. The poll above is the only thing that sees a Windows-side file grow: `wc -c` each tick, and
`tail -n +"$from"` for what is new.

What is worth one line:

- **the game's own state, which is the newest `f<N> scene=A->B stage=S frames=F … deaths=D lives=L` line** —
  the scene, the stage, both clocks, the deaths and the flags, all of it read off the game rather than
  inferred. This is the one thing that cannot be left out: see below.
- which recipe step it is on — `autoplay: step N of M …`
- and the newest thing that *happened*: `run ended after N retries`, `ending run out in N update(s)` or
  `ending skipped`, `died in chapter N`, `retry: …`, `replay: kept`, `front end: screen N`.

**A step number is not a state, and reading one as the other is how a stall was reported as progress.** A
launch playing a run to its clear had its run *end* at stage 5 — th06's Easy has no sixth stage — and the
step waiting for `{ stage: 6, chapter: 1 }` gave up the moment the run was over, as a `play` step does. The
step after it then sat at the game's own name entry for its whole bound, and the watch, which was reporting
`autoplay: step 7 of 10 is still waiting, 13200 frame(s) in` beside the newest chapter line, was read as a
run still being played in its last stage. The log had said `run ended after 0 retries` and
`ending skipped, 7993 frames run, scene 10 -> 7` a hundred seconds earlier, and the state line says the same
thing every frame: no run, scene 7, the clocks standing still.

So the poll's line carries the state line verbatim, and a watch that reports which step a recipe is on
without it is a watch that cannot tell a run from a screen the run has left.

The poll that says all three, one line a tick:

```sh
recent=$(tail -c 2000000 "$log")          # the newest lines, which is what a state is read off
state=$(printf '%s\n' "$recent" | grep -aE "^\[[0-9]+ms\] f[0-9]+ scene=" | tail -1 \
  | sed -E 's/^\[[0-9]+ms\] //; s/ seed=[^ ]*//' | cut -c1-120)
step=$(printf '%s\n' "$recent" | grep -a "autoplay: step" | tail -1 | sed -E 's/^.*autoplay: //' | cut -c1-48)
event=$(printf '%s\n' "$recent" | grep -aE \
  "run ended|ending (skipped|run out)|died in chapter|retry: |replay: (kept|declined)|front end: screen" \
  | tail -1 | sed -E 's/^\[[0-9]+ms\] //' | cut -c1-90)
echo "${step:-no step} | ${state:-no state yet} | ${event:-nothing yet}"
```

**A bounded `tail -c` and not a grep over the file**: a verbose launch of twenty minutes is hundreds of
megabytes, and a tick that reads all of it is a tick that costs more than the interval it is polled at. Two
megabytes is seconds of frames at a few hundred bytes each, which is what a state is.

**Cut the log to the launch.** `orb.log` is one file across launches and is sometimes rotated, so the newest
launch is what follows the last `---- new run ----` and no offset written down beforehand can be trusted:

    from=$(grep -an "new run ----" "$log" | tail -1 | cut -d: -f1)
    cut=$(tail -n +"$from" "$log")

**A monitor's own end condition is cut the same way.** Matched against the whole file, the previous launch's
closing line ends the monitor the moment it starts.

## When to close a run

**Stalled** is the log not growing. Fetch `wc -c` on it each tick and count the ticks it has not moved for.

**And a run can be over while nothing is stalled at all.** A game with no lives left puts its continue
prompt over the stage and waits for somebody; the log grows, the frames climb, `Game::read_state` still
says a run is on — and a `play_until` then waits out its whole bound, which for th06c's Extra is half an
hour of frames. So the state a watch says has to carry the **lives**: nought of them with the play step
still waiting is a game over and not a fight, and it is the one shape that looks healthy in every other
field. Nothing in a recipe ends it, so closing the game is the answer.

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

A recipe is a list of steps read by the game's own process, so one written for a measurement of the day lives
**beside that copy's exe** and not in this repository. `orb-config`'s `recipe` module is the grammar and
`Until` is what a step can wait for.

**The recipes that are tests live in the repository**, under `crates/orb-real/recipes/<game>/`, each with a
check over the trace its launch writes — `--trace=<file>` writes one MessagePack record per frame: the game's
state, the word the run was played with, the question of orb's own that was up, and the log's lines. `cargo
xtask real <copy>` plays them all in order and checks each, keeping orb's own files for the session under
`<copy>/orb-real/<stamp>/` through `--keep-in=<dir>` so nothing already in the copy is touched or answers a
question — measured: without it, three runs an earlier session had left beside the game were offered on the
first case's launch, and a pad answered the offer before the recipe reached it. A case there is
also played against the laid-out games by `orb-e2e`'s `the_cases_the_real_game_is_tested_with`, which is
where a step that reaches nothing is found first.

**A launch may come up in the game's own demo, and what leaves one is `decide`.** th06c's front end goes
straight into it — a replay of the game's own played at the title, which is the game manager's state 1 and
stands for as long as that replay does. Three launches were spent on this: one waited 23,400 frames for a
title that was not coming, one tapped `down` at the demo for 6,000 frames and stayed in it, and a third
worked only because a `decide` meant for a menu happened to land there first. So the first step of a recipe
for such a game is that press, waiting for the title, and the press it takes is the demo's own and reaches no
menu:

    - press: [decide]
      frames: 30
      until: { screen: 2, cursor: 0 }
      give_up_after: 36000

**And a screen on its own is what a step waiting for one waits for**: `wait_until: { screen: 6 }`, whatever
its cursor is on. Which is how a front end is walked where the screens do not arrive in the same order every
launch — a step per screen, each waiting for the screen it is about to press at — and where the cursor is no
use for it, being one field whatever screen is up and left wherever the score file remembered.

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

**A tap can overshoot its target by one press.** The press for the frame the condition became true has already
gone in, and no wait afterwards takes it back: a step that walked the title's cursor onto Run pressed `up` once
more onto the item beside it — Quit — and the `decide` after it closed the game.

**So a cursor is aimed with `until` only where the item after the target is harmless**, and where it is, that
is how to aim one:

    - press: [down]
      frames: 6
      until: { cursor: 3 }
      give_up_after: 600

Six launches walking th06's title cursor onto the replays that way landed on it every time, `Score` being the
item after it and nothing to be afraid of. Counting the presses instead needs the item the cursor starts on,
which is the score file's business and not a recipe's — `MainMenu::RegisterChain` memsets it to the first item
where a run was just left, and a launch that only started the game finds it wherever the last session left it.

**And the cursor a step can wait for is the game's own**, which is `Game::front_end_cursor` — a game whose
front end nobody has read that off answers nothing, and `until: { cursor: N }` then waits for something that
never comes.

**A question raised *by* a press is reached by the press step that waits for it**, and the step ends the frame
it is up — so the words are on the screen with nothing pressing at them. That is what makes orb's mode question
reachable at all: waiting for it presses nothing and it never comes, and walking the cursor there is the
overshoot above.

    - press: [decide]
      frames: 30
      until: { question: mode }
      give_up_after: 900
    - answer: { question: mode, item: 0 }
      give_up_after: 300

**And the answer goes straight after that press, with nothing between.** A launch that put a `wait_until` and a
`screenshot` between them came back `resume: from where it stopped, answered on the pad` — thirty-three frames
after the question went up and before the `answer` step was reached, so every one of its five answers gave up.
Nothing a recipe pressed can have done it (`wait_until` hands over a word of nothing, and off a run the rules
say nothing either); what the same launch logged at startup is `joystick: xinput 0 has a pad, read in 24353us`,
and a pad orb reads answers its questions whoever is holding it. A picture of one of these is worth having and
costs that risk: take it *after* the answer, of the screen the answer opened.

**An answer names the item, and a press never walks to one.** Every question orb asks keeps a cursor of its own
and a recipe's presses go to the game underneath it, so what a press answers is the item the question came up
on: three launches written to press `down` before `decide` at どこから始める all answered つづきから. So the item
is named instead, counted from nought:

    - answer: { question: resume, item: 1 }
      give_up_after: 600

`question` is one of the names `until: { question: … }` takes — `Question` in `orb-core`'s `game` module is the
list of them — and what a list holds in what order is the module that draws it. The step waits for that question
to be up, answers it, and is over once it has gone, so one that never comes up ends at `give_up_after`.

**Which is what reaches the questions a press cannot.** A pointdevice replay is chosen out of four lists of
twelve items and there is no other way into one; and the mode question, raised by the press at the title's own
item, is answered on the frame a tap toward it lets go — item nought, whatever was meant.

**The name a run's ranking is registered under is written and not pressed for**, and the step that writes it
has to be the step already running when the entry opens:

    - play_until: { name: true }
      give_up_after: 216000
    - name: ORB
      give_up_after: 1800
    - press: [decide]
      frames: 6

**Because the run's own end is past that entry.** It opens inside the scene a run is played in — after the
last boss and after the ending, with the field empty — so a `play` step waiting for the run to be *over* runs
straight past it. And every frame in between is a frame the rules are asked for a word on, whose held shot
writes whichever character the entry's cursor starts on: every ranking a clear registered before
`play_until: { name: true }` existed was `AAAAAAAA`, read back out of the replay the same run wrote.

**`menu` is the game's own pause menu, and it is a press like any other.** The button both games call the menu
button, held for a few frames and let go — the manager takes its *edge*, so a word held for ever opens one menu
and closes none:

    - press: [menu]
      frames: 4

What is behind it is that menu's own items, and 「最初からやり直す」 is what asks orb where to start over — the
one road into that question, and the only way to reach what the pause did to the music on the way. **Its items
are the game's own sprites and its cursor is the game's**, so they are pressed down blind: `answer` is what
orb's own questions have instead, and a menu of the game's has no such thing. Nothing waits for it either —
`until` has no state for a menu of the game's — so what a step after it has is frames.

**And the word it lands in is the one the game's own read left**, not the word a run is written down as: that
is what the manager watches, and it is why a resume feeding a run's frames back opens no menu.

**And an attempt watched to its end puts that same list back up**, on the attempt just watched, a second after
the front end comes back — so a recipe watching several attempts of one chapter is one `answer` step per
attempt and nothing in between:

    - answer: { question: choose_replay, item: 0 }
      give_up_after: 600
    - wait_until: { frames: 900 }          # the attempt itself
    - answer: { question: choose_replay, item: 1 }
      give_up_after: 900

**The list that comes back is the *attempts* one, and an item is an item of whichever list is up.** A step
written for the chapter list after a watch names an item that list has not got: `replay: the attempt list has
1 item(s), so there is no item 14` is what the log said, and the step then answered nought and watched the
same attempt over. Another chapter is a walk from the title again — every list of it, in order.

**And the four lists wait for the screen they stand over**, which orb asks the front end for: `replay: the
replays to watch, over the screen the game keeps its own on` is the line saying they are up, about a second
after the mode question is answered. A step that presses in that second is pressing at the game.

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

**And a `screenshot` step asks for nothing at all on a game that draws through D3D11.** `orb_core::shot`
copies the D3D8 back buffer at the end of `d3d8::end_scene`, which th06c never reaches — so its recipes'
`screenshot` steps log the ask, log `done`, and write no file. What takes a picture there is
`scripts/shot.ps1` from outside, run from the monitor on the line that says the screen is up, and **the
recipe has to hold that screen long enough for a poll to catch it**: two launches took their shot after the
game had closed, the recipe's 300 frames being under a second at `--pace=8`.

**What the game itself draws is always in the picture**, which is what makes a translation patch checkable: a
stage, its panel, a card's name plate and the title menu all come out, so the same recipe run with the patch
beside the game and without it is the check that the patch reached them.

**Nothing that only waits belongs in a run that is being played.** A `wait_until` hands the game a word of
nothing, so the rules get no frame and the player stands still until something shoots it — and each death
raises orb's retry menu, which the recipe's own next press then answers. Between two things a recipe does to a
running stage, the step is `play_until: { frames: N }`.

**And the other way round while an attempt out of a replay is being watched, which plays itself**: there
`wait_until` is the step, and `play_until` is what must not be there — the rules press the bomb button, which
is one of the two this game's menus take a cancel on, and a cancel stops the watch. `press: [cancel]` is how a
recipe stops one on purpose, and the log says `asked for on the pad`, a recipe's word arriving as one.

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
and nothing between them. `card_name: true` is the plate itself — for th07, the bit the engine's own draw reads
before it draws that vm.

**A picture asked for on the frame a screen is reached is a picture of the screen before it.** The front end
draws a change over several frames, so a `screenshot` straight after `until: { screen: N, cursor: M }` comes out
mid-fade with the outgoing screen still legible over the new one — three shots of three menus each showed the
menu before. **The same holds for anything a run draws with an animation of its own**: a picture on the frame
`card: true` became true came out with the boss's name and the timer in place and the card's own plate not yet
drawn.

**So a state to wait for and the frames a drawing takes are both needed, and neither replaces the other.** The
condition is what gets a step to the right moment of the run without counting a stage's frames; the frames are
what let the thing that moment is about be drawn. Measured on stage 1's first card: the plate is put up two or three
frames after `card: true`, `card_name: true` catches it as a shape with nothing readable on it, and thirty frames
on it reads `Cold Sign "Lingering Cold"`.

    - play_until: { card_name: true }
      give_up_after: 3600
    - screenshot: the card's own name
      frames: 30

**A run that is never hit reaches nothing that happens after a death**, and `take_hits` is what a run that has
to takes:

    - take_hits: { chance: 0.4, times_each_retry: 0.4, gives_up: 0.1 }

`chance` is per **chapter attempt**: the draw is made as the chapter starts and an attempt drawn is one the
rules ignore the hazards in — no dodging and no deathbomb — so the death lands in the middle of a pattern the way
a player's does. A drawn attempt that nothing has hit for a second goes to the nearest thing that can, so that
the chance is a chance of being *hit* rather than of playing carelessly.

`times_each_retry` is what that chance is multiplied by each time the same chapter is retried, which is what
gets the run past the chapter it died in: at the same chance every attempt a chapter is retried until the fire
happens to miss. And `gives_up` is the chance that a death is answered by going back to the title instead of by
taking the chapter again — asked for outright rather than pressed for, orb's own offer keeping a cursor a bot
cannot read.

**The step is where the chance starts**, not the whole recipe: put it after the front end and a launch plays its
menus clean, and a second one later changes the chance for the rest of the run.

**The draws are a hash of the attempt and not a sequence**, so the same recipe dies in the same chapters every
launch. What the log says: `stage S chapter C is one to be hit in, its chance being X after N retr(ies)` at
`normal`, the ones it passed over at `verbose`, and `retry: the run given up on the recipe's own ask` where a
death was answered by leaving.

**A run given up needs a launch of its own to carry on in.** The recipe's `play_until` ends where the run does,
so the launch closes with the run left in a file — which is the interruption a resume is for, and picking it up
is the next launch's first steps:

    - press: [decide]
      frames: 10
      until: { question: resume }
      give_up_after: 1800
    - wait_until: { frames: 20 }
    - press: [decide]                  # 続きから, the item that question comes up on
      frames: 2
    - play_until: { frames: 120 }      # the frames the pick-up plays its buttons back in

So a run to a clear with hits in it is launch after launch: one recipe that starts a run, one that picks it up,
and a loop that chooses between them by whether `pointdevice_resume/*.msgpack` is there. What says the game was
cleared rather than given up is `ending run out in N update(s)` — a run given up has `run ended` and no ending.

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

**And `quit` reaches no save of the game's own**, which is what a measurement of anything the game writes at
its own exit has to be built around: `quit: true` is `process::exit`, and th07's write on the way out is after
the message pump. What a recipe can reach instead is the save a run's *build* makes, so the shape is to play
the run, `leave` it, and start a second one — that build is where the first run's records are written.

**And a rewindable launch has two score files in it, so a line naming a count does not say which file's.**
Every open of one goes to the mode's own file except the front end's unlocks read, which stays the game's — so
`score: card N now A attempt(s) and C capture(s) …, where the file had a and c` can hold the two files'
numbers at once. `--no-chapters` leaves one file and is what makes such a line readable; without it, what
settles a number is the pair of launches rather than the line.

**A file the *released* orb wrote is one only a hand-played run produces, so keep the one it produces.**
Nothing here can drive that build: v0.10.0 answers `--autoplay` with `error: unexpected argument`, the bot
having come after it. And a file this build wrote with two keys edited out of it is a file no orb ever wrote,
which is the one thing a compatibility measurement must not be built on — so what makes one is somebody
playing the released exe for the forty seconds a stage's midsection takes.

**Copy it out of `pointdevice_resume/` before any launch is driven at that slot.** A launch whose question
that file does not answer starts a run of its own in the same slot and writes over it within a second of the
stage starting, which is how the first one was lost and a second play had to be asked for. `resume-fixtures/`
beside the copy is where the kept ones live, and one of them is worth more than the run it came from: every
later measurement of what a released file does is that file read again.
