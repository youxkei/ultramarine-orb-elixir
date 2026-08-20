# 21. Which fight is on is the music, and the stage's record where it has one

**Status:** accepted and built. `Chapters::due` in `crates/orb-core/src/chapter.rs` reads
`Game::the_fights_own_song_is_playing` and `Game::music_identity` for a fight it is seeing for the first
time and `Chapters::recorded_fight` for one whose frame the stage has been through before;
`chapter::tests::a_fight_replayed_is_the_fight_the_stage_recorded` is the rule, and
`the_fights_own_song_is_playing` in `crates/orb-core/src/game/mod.rs` is where the two games differ.

## Context

A stage has two kinds of fight, and which one a boss belongs to decides two things. It decides what the
chapters of it are called — `MIDBOSS SPELL 2` against `BOSS SPELL 2`, which is how a chapter worth grinding
is named to somebody else. And it decides whether the defeat is a boundary: a midboss going down hands the
stage back, and the waves after it are the only part of a stage with nothing else to begin a chapter for,
while the stage's own boss going down ends the stage, where a chapter whose first act is that the fight is
already won has nothing to retry.

**Neither game marks a boss as the one its stage ends with.** 妖々夢's registration at 0x413d70 writes the
slot at `[enemy+0x2e17]` and sets 0x40 at `[enemy+0x2e29]` the same way for a midboss as for the fight the
stage ends with, and the instruction that ends a stage — `[gamemanager+0x209bc]`, read at 0x427ea5 — fires
after the fight rather than saying anything about it. There is no field to read.

**The script says it once, by changing the song**, and both games say it the same way. A stage's `.std`
carries `char songPaths[4][128]` at `+0x290`, the stage's own track first and the boss's second, with the
names beside them at `+0x90`.

- 紅魔郷: the ECL dispatcher at 0x418d73 takes the argument of the instruction it is on, `[instr+0x4]`, and
  plays `stdData->songPaths[arg]` — `stdData + 0x290 + arg*128` — through `Supervisor::PlayAudio` at
  0x424b5d, which is the call a restore already restarts a chapter's music through. The same argument
  indexes the names for the line it draws at 0x418d3c.
- 妖々夢: the stage's build loads `songPaths[0]` into the player's first slot and `songPaths[1]` into its
  second (0x42f1d5 and 0x42f1ec, through 0x439dd0), stage 6 a third into the one after (0x42f233), and plays
  the first at 0x42f216. The ECL plays the slot its instruction names, at 0x42a251 through 0x439ec1.

**And neither keeps what it was asked for.** 紅魔郷's `PlayAudio` copies the path into two stack buffers
(0x424bd0 through 0x424c8e) and nothing outlives the call — its `CWaveFile` holds the `HMMIO`, the sizes and
the loop points, and no name. 妖々夢's 0x439ec1 either hands the slot straight to the backend's own object
(0x436790) or queues command 2 with it for `PLAY_SOUNDS` to walk (0x44d2f0), and that queue is walked empty.
So the song that is playing is the whole of what is left of the script's statement, and reading the music is
reading that instruction rather than guessing from it.

**What broke was when it was read, not what was read.** A step back replays part of a stage over frames the
game's own sound never runs on — `runtime::run_to` walks the calc chain and nothing else — and the restore
has asked the game for the track the chapter it put back was taken under. So the music over a replayed fight
is the stage's own whatever fight it is. A launch of 妖々夢 stepping about in its second stage had the whole
of that stage's fight as `a midboss nonspell` and `a midboss spellcard`, and the boss going down took a
chapter of its own.

## Decision

**A fight the pass is seeing for the first time is read off the music, and a fight over a frame the stage has
already been through is the fight that stage recorded there.**

The record is `Chapters::starts`, the frames this stage's chapters began at with the cause of each — the same
list the chapter names are counted out of and the stepping keys move between. `recorded_fight` answers from
the newest of them at or before the frame, a chapter running until the next one begins.

**It can only raise the answer, the same way the music can.** A boss recorded as a midboss is this
misreading itself, and a fight the stage has no boundary under the frame of — a midboss's defeat lost to a
death, and the boss after it read against the midboss's last spellcard — is one the music is still right
about.

Where the game can name the track, the music is asked by name: 妖々夢's `the_fights_own_song_is_playing`
holds the record's own name against `songPaths[1]`'s stem. 紅魔郷 keeps no name anywhere, as above, so it is
left with `music_identity` — the file's length and loop points — having changed since the stage began.

## What follows

Both consequences of the reading come out of one place. `Cause::MidbossNonspell` and its three fellows carry
the fight into the record, so what the record answers with next time is what was decided the first time; and
`Chapters::won`, which is what a judging pass is walled by at the end of a stage, is the defeat of a fight
read as the stage's own.

The record is a stage's and goes when the stage does — `the_stage_was_left` clears it — so a stage arrived at
is read off the music again, which is what a stage nobody has played yet needs.

## Alternatives, and why not

**The timeline's wait.** 妖々夢 parks its timeline on `RunEclTimeline`'s `case 0xc` for a boss, and that is
also how stage 3 waits for its *midboss*, which called that fight the stage's own.

**The slot number the script asked for.** It is the script's statement itself and it is fresh on a replayed
frame, the ECL running in the calc chain. But nothing keeps it, as measured above: on 妖々夢 it sits in the
player's queue until `PLAY_SOUNDS` walks it and on 紅魔郷 it never leaves the stack, while what has to be
answered is every frame of a fight that lasts thousands of them.

**Waiting to see whether the stage ends.** The stage's own boss going down is followed by the stage ending
and a midboss's is not, which decides it with nothing read off the sound at all. It answers the wrong half:
a chapter has to begin on the frame the midboss went down, since its snapshot is that frame's memory, so the
boundary would have to be taken and then retracted — and the name a chapter of the fight is offered under
while it is being fought would still be unanswered.
