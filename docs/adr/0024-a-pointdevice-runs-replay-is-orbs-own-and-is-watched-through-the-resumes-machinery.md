# 24. A pointdevice run's replay is orb's own and is watched through the resume's machinery

**Status:** accepted and built. `crates/orb-core/src/replay.rs` is the record and the file,
`crates/orb-core/src/replay_ui.rs` the question and the lists, `resume::watch` and `Record::until` in
`crates/orb-core/src/resume.rs` are the playback, and `Game::front_end_starts_a_run` is the run it is
watched in. `Game::to_the_screen_the_replays_are_on` and `Game::the_screen_the_replays_are_on` are the
screen the lists stand over, and `Game::the_screen_a_run_ends_at_is_up` is what the run-out of the screens a
finished run ends at is bounded by.
`watching_a_pointdevice_replay.rs`, `the_replay_a_run_leaves_behind.rs` and `the_replay_offered.rs` in
`crates/orb-e2e/src/` drive it. It stands on
[0022](0022-what-a-resume-puts-back-is-what-the-games-own-replay-restore-puts-back.md), which is what makes
a stretch of a stage reproducible at all.

## Context

**The game cannot keep a replay of a run that was rewound, and what it would keep is the reason.**
`ReplayManager` is `new`ed at 0x42a27f and each stage's record allocated per stage, in the heap a chapter's
snapshot covers — so a retry rewinds the record along with the run, and each attempt writes over the last
from the chapter's own frame. What `SaveReplay` would write is the path that survived, with nothing in it of
the attempts that did not: a run nobody could tell from a flawless one. That is why the game's own offer is
written past, and it has been written past since the mode existed.

**But the attempts are the whole of what somebody grinding a chapter has done.** A pointdevice run is a
history of them — eleven tries at one card, the one that worked last — and none of it survives in any file
the game writes. The chapter file already keeps the *path*, for the resume, and throws the rest away for the
same reason the game does: what it needs is where the run is standing.

**Three ways of keeping the attempts were open.**

- **The game's own format, written by orb.** `.rpy` is watched on the game's own screen, which is the whole
  attraction — nothing of orb's to draw, and the file readable by anything else that reads them. What it
  cannot hold is the shape: a stage's record is one list of inputs, so the attempts would have to be
  separate replays, one file per try, and a chapter played eleven times is eleven files whose relation to
  each other nothing in them says. Watching one would also be the game's own playback, which is a different
  thing to the game in every place `isInReplay` is read — the score, the result screen, and what a stage's
  start puts back.
- **The game's own replay manager, driven.** Its playback reaches a stage and the ending skip's loop reaches
  a frame, so the machinery exists. What it costs is written where the resume declined the same offer: the
  takeover afterwards, two ways of allocating a record in one heap, and a run that is a recording being
  watched for as long as the playback lasts.
- **orb's own file, played back the way a run is picked up.** Which is the third, and it is what the resume
  already is.

## Decision

**A pointdevice run's replay is orb's own file, and watching an attempt out of it is a run being picked up
with a second end.**

The record is the path a stage was left with plus, per chapter, the attempts that were rewound away — so the
one that survived is held nowhere of its own, being that path from the chapter's frame to the frame the
chapter after it began on. One file per
run there can be one of, named for the slot the chapter file is named for, `.pending` while the run is being
played and dated on the day it is kept.

The playback hands `resume` a chapter of a run — the same numbers, the same buttons, the same landing held
against the same reproduction line — and one number more: the frame the feeding stops on. For a run being
picked up that frame is the chapter, which is where the keyboard becomes the player's; for an attempt being
watched it is the end of the attempt, and the frames between are fed one to a frame of the game. So the two
differ in `Record::until` and in nothing else.

## What follows

**The file is one shape for both games where the chapter file is per-game.** That one is per-game because it
shipped that way and a field th07 needed could not be added without refusing every th06 file on a disk. This
one has no installed base, so what each game contributes travels as what it already is — the game's own
bytes from `Game::run_state`, opaque to the rest — and `Game::named` in the file is what refuses one carried
between two installations.

**The run is asked of the game's own front end, and orb does not write one into being.** Every other run orb
puts a chapter into was chosen on the game's own screens; this one came out of a file, so
`Game::front_end_starts_a_run` puts the screen whose item starts a run back up with this run's numbers under
its cursor, and the press that chose the attempt is handed over rather than swallowed — the item reads it and
does the rest.

**Which is not what this started as, and the difference is a crash.** `Game::start_a_run` writes the numbers
and the scene itself, which reads like the same thing: every write the item makes is one it makes too, held
against the item's own code address by address, and the run it built came up on the recorded seed and played
its buttons back. Then, 129 frames in, th06 went — `EnemyManager::OnTick` reading a sprite pointer of nothing
off the first enemy the stage's timeline spawned, at 0x412499. The stage's own data was identical to a
working run's, field for field: the ECL loaded at the same offsets with the same 24 subs, the timeline
waiting on the same entry, the same sub id spawned, the same template copied over the slot. What the item
does that a write cannot is still not known — and it does not need to be, because the item is what runs now.
A run the front end started spawns that enemy with its sprite and watches an attempt through to its end,
measured on the real game.

`start_a_run` is what is left where a game has no such screen to put up, and the frame `run_chosen` is true
of is where the chapter goes in on either path.

**Nothing about that run is written down**, which needed a second flag beside the one that says a run is
being kept: `resume::WATCHING` puts the numbers back and feeds the buttons, and `KEEPING` does that *and*
records. A playback that recorded would be the run it came out of overwritten by a recording of itself.

**And the run ends where the attempt does.** What follows the frame an attempt ran to is whatever a keyboard
does to a stage nobody is playing, so orb leaves the run there. Which is also what makes an attempt the unit
of watching: running on past it would mean a rewind between attempts, and that is the retry menu — a replay
retried by whoever is watching it is two runs.

**So the list the attempt came out of is what the run is left *into*.** The unit being an attempt is only
worth having if the next one is a keypress away, and the walk down to it — the replay, the stage, the chapter
— is one whoever is watching has already made: the menu keeps where its cursor was on each of the four, so
the same one put back up is that walk not made again. Everything under one of these lists is frozen for as
long as it is up, so a list that came back too early stood over nothing for as long as somebody left it
there.

**And the screen it stands over is the game's own replay screen, asked for rather than pressed for.** A list
of replays over the title menu — or, coming back, over the logo a left run goes to — reads as having left the
replays rather than as being in them, and the press that would have taken the game to that screen is the one
orb held back for its question. So orb writes the state the item writes, with that state's own sixty frames
already spent: those frames are the item's fade and what they draw is the screen being left, which is the
title menu it exists to avoid. The screen then arrives on the next update, and what the list waits for after
that is its drawing — a couple of frames going up the first time, the front end's own sixty coming back,
where the front end itself was built on the frame the run went. Which makes the two ways in one wait
rather than two — the first list and the one that comes back after an attempt both wait for that screen —
and leaves a cancel out of the first list on the screen whoever chose that item would have been on.

**The screens a finished run ends at are run out inside the frame they came up in**, where the attempt was
the path the boss died on: the write that takes the game out of them is the screen's own way out, and the
screen plays that out over the twenty frames it takes. Since drawing happens once per frame, calling the
game's update over until the screen is gone is what keeps a name entry belonging to nobody off the screen —
the same trick the ending skip is.

**What this does not buy** is anything readable by another program: nobody else reads these files, and the
game will not. The trade is the shape — the attempts, in one file, watched from any chapter of any stage —
against a format somebody else could open, and the shape is the thing the game's own replay cannot have.
