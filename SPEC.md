# Ultramarine Orb Elixir — specification

What orb does, and the facts about 東方紅魔郷 1.02h and about Windows that shape how. The README
says where to download it and which games it runs, and nothing more than that; everything anybody
would use it with — the settings, the arguments, the files it writes and every screen it puts up — is
here.

**This document describes the final form only.** No history of what was tried and rejected,
and no record of what a mechanism used to be — only what it is, and the facts it rests on. The
reasons an alternative was rejected belong in a comment beside the code that would otherwise
tempt someone back to it, which is where they are. What was measured to settle something is beside the
thing it settled — the decision in [docs/adr/](docs/adr/), the constant it is the reason for, or the
e2e test that asserts it; what is left is in the repository's issues.

## Which game a launch is

**One table names every game orb knows, and both halves read it.** `orb_core::game::KNOWN` holds,
per game, the exe's own file name, the file the game keeps its configuration in, the builds the
addresses were read off — an md5 and what that build is called, apiece — and the [`Game`] those
addresses are in. The launcher finds the game by which entry's exe is in the directory it was pointed
at, and refuses an exe that is none of that entry's builds, naming every game and build orb knows; the
build it *is* is what the line a launch prints names, since a report of a fault is read against what
was played and the file's own name does not say which. The DLL is already inside the process by the
time it can ask anything, so it matches on the exe's own file name — case-insensitively over the ASCII
of it — and names in the log every build its addresses were read off, which of them this one is being
the md5's to say and the md5 being read before the process existed. A process no entry names is one
where orb patches nothing, says which games it knows, and does nothing else.

**A list of builds rather than the one build, because a translation patch makes another one.** orb
reads the game's state through absolute addresses, so what an entry has to hold is every exe those
addresses are right for — and an exe with the game's text swapped out is a different file with the same
code in it. Each build in the table is one somebody has had orb's addresses read against; what is not
in it is refused, whatever it says about itself. Today every entry holds one build, so the exe orb
accepts is the exe it always accepted.

The choice is made once, at the attach, and kept in a static the hooks read: a hook is a plain
`extern` function with nothing but the ABI's arguments, so where it would be handed a game it reads
one. The same reason the frame loop's two calls into the game are statics — see
[docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md) — and the
shape [docs/adr/0004](docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) settled the
choice of game into.

One table rather than a constant apiece because two lists cannot be kept in step, and a launcher
that starts a game the DLL does not recognise is a launch where orb is loaded and silent.

Two entries, and they are not the same kind of game.

`東方紅魔郷.exe`, 1.02h, `md5 fa3d64768b1bfc50703dedc2db92f7fa` — everything this document describes.
Every address below was read off that build and cross-checked against the
[GensokyoClub/th06](https://github.com/GensokyoClub/th06) decompilation.

`th07.exe`, `md5 0126afce1e805370d36c3482445e98da` — 東方妖々夢, and **a game orb is in rather than one
it does anything to.** What has been read of that exe is a frame's worth of addresses, each with the
evidence in `orb-core/src/game/th07/mod.rs`, and what a launch there gets is: the window sized to the
4:3 its 640x480 output has, and orb's update and draw hooks inside the game's own frame. Nothing else.
Not orb's own frame loop — replacing 妖々夢's frame took the game down, and `Hooks::render` is `None`
until the rest of that frame has been read — so no cadence of orb's and no frame of input lag removed.
Not anything drawn either, there being no `font.ttf` beside that exe to build an overlay from. And none
of what the rest of this document is about: no chapters, no retry menu, no run picked up again, no card
counted, no mode question.

Every one of those is a method of `Th07` answering `None` or nothing rather than a branch anywhere above
the seam, which is what a seam is for — and none of them answers a guess, an address written down
because it is where 紅魔郷 keeps the same thing being the one thing that must not happen. That rule is
what the frame loop cost: the addresses in 妖々夢's frame lined up with 紅魔郷's and the *shape* of the
frame did not. See
[docs/adr/0004](docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).

## Chapters and retries

A chapter begins at a stage's start, at each attack of a fight — the boss's timer being reset,
which a spellcard beginning does too — and when a midboss is beaten, since that hands the stage
back and the waves after it want a chapter of their own. Not when a spellcard ends: either the
fight is moving on, which the timer says on the same frame, or the boss has just been beaten.
Not when the stage's own boss is beaten either: the stage is over, and a chapter whose first act
is that the fight is already won has nothing to retry.

Dialogue and a pause hold a boundary back, since neither is the run getting anywhere; a bomb and
a death do not, though a chapter beginning under either is a poor place to restart from. A
boundary the fight really has is worth more than one that is always comfortable, and the way out
of a bad start is the chapter before it. What the fight did while a boundary was held back is
taken as soon as one is allowed, so nothing is lost to the seconds a bomb or a death lasts.

The log names what began each chapter, so one that turns up where none belongs says which signal
produced it. Midstage boundaries come from a compiled-in table of script frame numbers, because
a stage's waves run on a clock and are reproducible. Boss boundaries are detected as the game
runs, so a difficulty with an extra attack gets an extra chapter with no table.

**A chapter beginning washes the play field green**, so that where dying will send you back to
is something seen rather than a number to read off the status line. Dim enough to read the
bullets through and gone inside a sixth of a second, because a chapter begins every few seconds
through a fight and the frame one lands on is a frame being dodged on. Not a stage's own start:
that is already unmistakable from the title, the music and an empty field, and a wash over its
first frame says nothing that was not obvious. Green because the game flashes white itself — a
bomb, a boss going down — and a mark that means something orb decided should not look like
something the game did; nothing in 紅魔郷 fills the play field with green. It holds a moment
before it fades, since a wash that starts fading at once reads as dim however bright its first
frame is. The play field and no further, because that is the part the game repaints every frame:
the panel beside it and the border around it are not, so a wash drawn there would never be drawn
over and would stay for good. The fade is counted in frames drawn rather than in the game's,
which is stopped wherever a step is holding the game still.

`boundary_flash: false` leaves the screen with nothing of orb's over the game. Only a run
somebody is playing is marked, and a judging pass — see *Building the midstage table* — never a
replay running for anything else: a collecting pass crosses a boundary every few drawn frames,
and a green field for twenty minutes is a mark to nobody.

A retry restores a whole memory snapshot rather than asking the game to jump anywhere.
Nothing has to know what a boss script was in the middle of.

A snapshot covers:

- the exe's `.data`;
- the regions the game's CRT took from the OS, found by hooking `HeapCreate`,
  `HeapAlloc`, `HeapReAlloc`, `HeapFree` and `VirtualAlloc` in the exe's import table and
  walking the heaps;
- the music's stream position.

Direct3D and DirectSound objects are not saved. The game loads a stage's resources on entry
and creates nothing within a chapter, so restoring the pointers in `.data` leaves them
naming the same live objects. Since those addresses differ per launch, a snapshot is only
valid inside the process that took it, so nothing of one is written to disk. What is written down
instead is what the run *pressed*, and a chapter is reached again by being played to — see
*Picking a run up again*.

Restoring suspends the game's other threads, puts back any page of a saved region that has
gone since — the game freeing a few megabytes hands part of a segment back to the OS, and the
copy has to have somewhere to write — writes each region back, and resumes. The audio thread
is left running; the music is handled separately, rewound for the midstage and the midboss and
left playing through a boss-fight restore.

**A track change is handled by the game, not by the copy.** Restoring a snapshot taken under
another track — the stage's start, once a boss has brought its own music — cannot put that
track back: the game freed its stream and released its sound buffer, and no memory copy brings a
released COM object back. Neither is the memory restorable around the stream that replaced it,
since that one was allocated after the snapshot: writing the snapshot back rolls its object out
from under the streaming thread, which is not suspended, and that was measured as an access
violation inside `DSOUND.dll` writing a buffer it no longer owned. Skipping the live ranges only
moves the problem, because the heap's bookkeeping still rolls back to before that stream existed
and the next track change then frees a block the allocator has been told is free.

So the game is asked to do it:

- `SoundPlayer::StopBGM` (0x430f80) before the copy, while the game's memory is still its own —
  it stops the buffer, quits and joins the streaming thread, and deletes the stream, which is
  the one free its allocator agrees with;
- the copy, with nothing held back;
- `backgroundMusic` and `backgroundMusicThreadHandle` cleared, because the restored state names
  the stream that was playing when the snapshot was taken and that one is long gone;
- `Supervisor::PlayAudio` (0x424b5d) with `g_Stage.stdData->songPaths[0]`, the stage's own
  track, read out of the memory that was just restored.

The track therefore starts again from its beginning rather than from where it was, which is what
a midstage restore does to the music anyway.

Two generations are kept: the current chapter and the stage's start, matching the two items of
the retry menu that put anything back. Of this stage only — a snapshot of an earlier one would name Direct3D
textures the game released when it loaded this stage, and reloading them is not enough to make
it whole: an `AnmVm` holds its script as a raw pointer into the file's buffer, so a file
loaded again at another address leaves every live one of them pointing at nothing. Measured as
a jump to 0x00000000 on the frame after such a restore.

It is kept across a stage transition, which is not the run ending however much it looks like
one from outside: the game leaves the gameplay scene for `GAMEMANAGER_REINIT` while it tears
the last stage's managers down and builds the next one's. Only leaving the run for good takes
it.

## Where the chapter was lost

The menu goes up on the frame the miss becomes certain, which is `deaths` moving rather than the
player being hit: the death bomb window closes first, and a successful one never gets here. The
game is frozen underneath it — `RunCalcChain` returning `CHAIN_BREAK` with the drawing carrying
on — so the frame the player died on stays on the screen behind the menu, and the play field's
viewport has to be set by orb because the game's own frame setup is part of the update being held
back.

Three ways on, and the chapter is the first of them — each in whichever language orb's screens are in;
see *The language orb writes its screens in*:

| | |
| --- | --- |
| チャプターをやり直す / `Retry the chapter` | the snapshot taken where this chapter began |
| ステージをやり直す / `Retry the stage` | the snapshot taken where the stage began, which is chapter 1 |
| タイトルに戻る / `Back to the title screen` | the run given up, and the game on its way to the title menu |

The third is named for where it ends up rather than for what it gives up, that being the half of
it somebody reading the item does not already know: that the run is over is the obvious part, and
that the game carries on into its own front end is not.

**The chapter acts on the press; the other two ask first.** A fight worth grinding loses a
chapter every few seconds, so the first item is answered hundreds of times in a session — a
question in front of it would be answered without being read, and that trains the hand which then
answers the other two. Those two are one press away from that hand and neither can be taken back:
the stage's start throws away everything the stage has gained since it, and giving up throws away
the run. So each of them puts up a second question naming what it is about to do, with the cursor
on いいえ / `No`, which is where the game's own quit question puts it.

The question is the whole of what is said there. A line under it spelling out what would be lost
was tried and taken out: the question already names what is about to happen, and a screen somebody
arrives at by dying is not the place to be read at.

`x`, escape or the pad's cancel takes a confirmation back to the three items.

**Neither of orb's menus writes its keys on the screen.** Both had a `Z 決定    X 戻る` line under
them and both lost it: the keys are the game's own — `z` shoots and `x` bombs, which is what its own
menus take as decide and back — so the line was telling somebody playing 紅魔郷 the one thing they
already know. Where a way out is worth pointing at, the screen does it with an item rather than with
a key: a confirmation's cursor sits on its no, and the retry menu's third item says where it goes.

**Both graces are keys being held off, for two different reasons.** The menu itself waits 24
frames because the player was holding a direction and the shot key when they died, and those
presses belong to the run. A confirmation waits 12, and not for that: the press that opened it is
an edge and so already spent, which is why this is a few frames rather than a fifth of a second —
what it buys is that a question cannot be answered on the frame it appeared on, since an answer
that fast is one nobody read. The cursor starting on いいえ is what makes such a press cost
nothing but the question closing.

**Giving up is what the game's own quit does**: `isInGameMenu` and `isInRetryMenu` cleared and
`g_Supervisor.curState = MAINMENU`, which is `StageMenu::OnUpdateGameMenu`'s answer to yes.
`Supervisor::OnUpdate` then cuts the run's chain — the game manager, the player, the stage, the
recording — and registers the front end, so nothing of the run is orb's to take down. What
notices the run is gone is the state leaving `in_run` a frame or two later, which is the same
path that ends any other run and is what drops the snapshots and writes the line saying how many
retries it took.

The two flags are written although neither can be set here: orb's menu is not one of the game's,
and it goes up on the frame `Player::Die` runs, where the flag a run out of lives gets is written
30 frames later by a respawn the freeze never reaches. They are worth two bytes for what a stale
one *does*, not for how it would get there — `AsciiManager`'s job is registered by the supervisor
for the whole process rather than per run, so `isInRetryMenu` left set has
`StageMenu::OnUpdateRetryMenu` running on the title screen, and its first three branches write
`curState` themselves, one of them to the result screen.

The key that answered has to be swallowed here, where a retry needs nothing of the sort: a retry
puts the whole of `.data` back from a snapshot, and this leaves the game to build the title menu
— which reads the same keyboard, and would take the `z` still held as an item chosen. No score is
entered on the way out, the same as the game's own quit: that happens at the result screen, and
giving up does not go through one.

## Picking a run up again

A snapshot is only good inside the process that took it, so closing the game would lose where the
run was — which for the one thing orb exists to do, grinding a late chapter, is the wrong place to
lose it. What survives a launch is what the run *pressed*, and the chapter is reached again by
being played to.

`pointdevice_resume/` beside the game holds one `.msgpack` per run there is a chapter of, and each
holds: which run it is, the run's numbers as the stage it is in began, and the buttons of every frame
from that stage's first to the frame the chapter began on. One is written **every time a chapter
begins**, because there is no moment a session ends at — the window closed, the game killed and a
crash all leave nothing to write from.

**MessagePack, through the same serde the settings go through, and with the field names in it** —
`to_vec_named`, not the positional arrays rmp-serde writes by default. Packed because the file is the
machine's own: written as text a chapter deep in a stage came to 30KB against 11KB here, and nobody
reads a button mask by eye. Named because a file that carries its own field names needs nothing but
itself to be read.

**Reading one takes a MessagePack-to-YAML converter and nothing more**, which is the reading half of
that decision: reading one of these beside the log is what has caught every fault in this
machinery — the seed written 2048 draws early, the song position ignored, a landing that agreed on
every field it had. Because the file carries its own field names, that converter needs to know nothing
about the format and so cannot fall out of step with what wrote it — any of the msgpack CLIs will do,
and the tree this was written in keeps one of its own under `scripts/`, which the repository does not
carry. Not a flag on the launcher: it is for whoever is working on orb, not for anybody playing.

The buttons are a map keyed by the frame they changed on, rather than a list of pairs: a map of frames
is in order because it is a map, not because something checked, it is a byte smaller each, and the
dump prints one change per line where pairs are two. Everything anyone reads by eye is a string
either way — the seed as `0x1a2b`, since that is how every line of the log writes it, and the
reproduction line as the log's own text.

A chapter is never written down part-way through: the resume point is a chapter's own first frame,
which is where dying sends the player anyway, and the frame a run was abandoned on is as likely as
not the frame they were hit on.

**One per run there can be one of**, the way 紺珠伝 keeps a pointdevice save per difficulty and
character. A run of another character is another run and its buttons would play somebody else's
shot, so what two runs have in common when a chapter of one can be picked up in the other is the
difficulty, the character and the shot. That answer is one string, `lunatic-marisa-b`, which is both
the name of the file and the test for whether two runs are the same run: a listing of the directory
then reads as the runs somebody has left unfinished. A directory rather than one file holding them
all, because a file is written whole at every chapter and with every run in it that write would be
every run's buttons a few seconds apart.

**A practice run has no such name, and that is the whole of why none is kept.** It is one stage
played on its own — nothing carried in, nothing after it — so what a resume would save is the walk
back through a menu that starts the same stage again in a moment. Having no slot is also what keeps
it from taking the full run's: a practice run of the same shot would name that same file unless the
stage went into the name too. Nothing is written for one, nothing is offered, and nothing is marked,
by there being nothing to call it rather than by three separate checks.

**The record is one entry per stage frame, kept by frame number**, where the game's own replay
keeps a `(frame, buttons)` pair per change. That is what makes a retried chapter cost nothing: an
attempt that is rewound plays the same frames over and writes over them, so the entries below any
frame are always the ones that survived to reach it, and a restore has nothing to drop. What goes
in the file is the frames the buttons changed on, which is a few hundred lines for a stage.

The buttons are written down and handed back in `Controller::GetInput` (0x41d820), which
`Supervisor::OnUpdate` calls once a frame and is the one place every button the game acts on passes
through. They are keyed to `GameManager.gameFrames`, read in that same call: the supervisor is
chain priority 0 and `GameManager::OnUpdate` — which is what advances that counter, at its very
end — is priority 4, so the number read there is the frame the update about to run *is*. That
counter is also the clock a chapter's start is expressed in, which is what lets a chapter name a
place in the record.

Only the buttons the game's own replay records: `g_CurFrameInput & 0x1f7`, which is every button
but 0x8. That one opens the pause menu, and a resume that fed it back would put up a menu instead
of playing. A frame with a menu up is not in the record either, and does not have to be kept out
of it by hand: `GameManager::OnUpdate` returns `CHAIN_CALLBACK_RESULT_BREAK` there before it
reaches the counter, so the frame does not advance and the same entry is written again by the frame
that eventually runs.

**What the run's numbers are, and where they go back.** The state a stage needs to be played again
is what the game's own replay keeps per stage, which is `StageReplayData`'s eight fields:

| | |
| --- | --- |
| score | `guiScore` at `GameManager+0x0` and `score` at `+0x4`, both, as the replay writes both |
| `randomSeed` | `g_Rng.seed` at 0x69d8f8, with the generation count cleared |
| `pointItemsCollected` | `+0x1816` |
| power | `+0x1810` |
| `livesRemaining`, `bombsRemaining` | `+0x181a`, `+0x181b` |
| rank | `+0x1a70` |
| `powerItemCountForScore` | `+0x1819` |

and beside them the difficulty at `+0x10` and the character and shot at `+0x181d` and `+0x181e`,
which the replay keeps in its own header. orb writes two more. `extraLives` (`+0x181c`), because
the replay's path does not: `GameManager::AddedCallback` raises that count with a loop against the
score thresholds instead, and a count that can only rise means a resume writing the score without
it hands out every extra life the run has already had — a life the run never got, and with it the
`IncreaseSubrank(200)` that came with the last one. And the count of deaths (`+0x20`), which
nothing but the result screen reads, so that a resumed run's is the run's. The two counters beside
that one are left as a fresh stage leaves them, the game's own replay not carrying them either.

**The seed is the whole of why the buttons alone are not enough.** `GameManager::AddedCallback`
copies the generator's seed into `randomSeed` as each stage begins and clears the generation count
— it does not reseed — so a stage played again from a different seed is a different stage however
the buttons fall.

**And it goes in at one instant, which is neither end of that callback.** It is written on the way into
`Stage::RegisterChain` (0x4044c0) — called from one place in the whole exe, 0x41c00d, between the
numbers being put in place and the stage being built out of them. Both ends of the callback are wrong,
and each was measured wrong in turn:

- **After it is two draws late.** Building a stage draws from the generator — two numbers on stage 1 —
  and `Stage::RegisterChain` is what draws them. The landing check said so in the one field it
  disagreed on: `randoms=2 against randoms=0`.
- **On the way in is 2048 draws early.** Before the callback copies the seed anywhere it fills a table
  of keys out of the generator: 0x41bc4f, 64 records of 32 `u16` at `manager+0x30`, one
  `Rng::GetRandomU16` (0x41e780) each, and every draw rewrites `g_Rng.seed`. The whole block is
  skipped when `curState` is 3, the state between two stages of a run — so a stage reached by playing
  draws none of them and a stage reached from the menu, which is every resume, draws all 2048.
  Measured as a stage resumed with `seed=0x789c` where `0xc381` was written down, and 2048 draws of
  that generator from `0xc381` are `0x789c` exactly.

Written where the game's own replay effectively writes it, in other words: `AddedCallbackDemo` is
called from 0x41bf7e, after the 2048 and before `g_Rng.generationCount = 0` at 0x41bfec. The
`GameManager` copy of the seed goes in beside the generator, since the callback has already taken it
by then and that copy is what the *next* chapter of the resumed stage writes down.

**The seed is in the landing line too**, and not only in the header, because it is the one number a
stage can start wrong in while every other field of that line is right: a resumed stage 2 agreed with
what was written down field for field with its seed 2048 draws out — the player's place being the
player's own inputs, and the bullets being the stage's script, which is deterministic. A landing that
agrees now agrees about the generator as well.

**Which score is the run's depends on the frame it is read on.** The last thing that callback does
is `score = 0`, and `GameManager::OnUpdate` raises it back on the stage's first update — `if (score
< guiScore) score = guiScore`, which is what carries a run's total across a stage at all. So on the
frame orb reads, `score` is nothing and `guiScore` is the total, and it is `guiScore` that is read.
The game's own recording reads `score` there and is right to: it is registered from inside that
callback, before the zeroing. Reading the field the game reads, on a different frame from the game,
is where this would have gone wrong silently — every stage written down with a score of nothing.

**They go back where the game puts them back**, which is that same callback: 0x41bb02, registered
by `GameManager::RegisterChain` as the added callback of the chain element at 0x69d720 and so run
by `Chain::AddToCalcChain` at the moment a stage is registered, before that stage's first update.
It is where `ReplayManager::AddedCallbackDemo` writes a replay's stage record from — called from
inside it — and it has to be after rather than before, because that callback is also what sets rank
from a table indexed by difficulty and what overwrites the power in practice mode. orb hooks it and
writes the run's numbers as it returns; on any other stage of any other run, that is where they are
*read*, which is the only moment they are the numbers the stage started with.

**Starting the run is one write, because the game's own front end has done the rest.** The question
is put on the frame after the character select has taken its item, which is the one frame where
`g_Supervisor.curState` is already `GAMEMANAGER` while `wantedState` is still the front end:
`Supervisor::OnUpdate` is chain priority 0 and copies `wantedState = curState` as its last act, and
the front end is priority 2, so an item that starts a run writes the state after that copy and the
supervisor has not yet acted on it. By then the difficulty, the character and the shot are the ones
just chosen, the front end has removed its own job — its update returns
`CHAIN_CALLBACK_RESULT_CONTINUE_AND_REMOVE_JOB`, whose deleted callback releases the title's
graphics and cuts its drawing — and nothing of the run exists. So orb writes `currentStage`, as the
menu counts it: `AddedCallback` raises the number by one, so it is handed the stage before the one
meant, which is what the character select writes too.

The number is checked against what the game has, since it comes out of a text file and one that is
nonsense is `AddedCallback` indexing its stage table past the end — `[eax*8+0x4764ec]` is where it
picks a stage's data out of. Nothing else has to be checked or written: what the run *is* was chosen
on the game's own screens, and the file is only read at all when its name says it is that same run.
The demo and a replay reach those two states the same way, so both are refused: the demo starts from
the title screen itself.

**The playback runs inside the frame that built the stage**, updating with nothing drawn until the
frame the chapter began on — the ending skip's mechanism aimed at a chapter. An update is tens of
microseconds and a stage is thousands of them. Every update is observed, so the run lands with the
stage divided into the same chapters and with the chapter's own snapshot to be sent back to, and
the retry count carries on from what the run had already spent.

**A chapter the stage's own song plays through lands with that song where the chapter had it**, which
is what a retry of one does to the music — and a resume is the same chapter arrived at another way, so
it owes the same. Otherwise what plays is the track's opening milliseconds, the stage having been
built a frame ago.

Which chapters those are is decided by the song and not by the chapter's kind: a midboss is a fight
with the stage's song still playing, so its chapters want the same thing done to them as the
midstage's, and it is the same question the retry answers from the same place. Asked of the kind
instead, a resume into `MIDBOSS NONSPELL 1` left the track at its opening milliseconds with the
position that had been written down ignored.

What is written down for it is one number: the file offset of the sound that was audible when the
chapter began, which is where the streaming thread's next read was less everything sitting in the
buffer unplayed. The buffer's own contents are what a snapshot keeps for a rewind and are no use
across a launch — the buffer is an object of that process — where the wave file is the same file
next time. Putting it back seeks the file there, fills the buffer from it, and starts the buffer at
nothing, which leaves the stream exactly as a freshly loaded track leaves it: one buffer's worth
from that offset, the play cursor and the next write at nothing, the file at what follows. Filled
rather than left to the streaming thread so that nothing of the opening it already held is heard
first.

**And the countdown to the loop with it, which is the second half of that seek.** The game's wave
file is the DirectX SDK's `CWaveFile` with loop points added, and it keeps `m_ck.cksize` — the size
`mmioDescend` left, or the loop end where the track has one — as how much sound is left: every read
is clamped to it and subtracted from it, and the track is looped when a read comes up *short against
it*. The file's own end is never consulted. So the position and that countdown are a pair, and a seek
that moves one alone is a track that loops in the wrong place: measured as seconds of the song
repeating, once, near the end of a resumed stage, because the stream believed it had as much left as
it did before the seek, read past the end of the sound, and got a failed read rather than a short one
— no loop taken, and the buffer left going round its own contents until the skipped bytes had been
counted off. It is put back as the loop point less where the file now is, and the loop point is taken
from the pair as it stands before the seek rather than from the header's fields, since which of them
the track is going by is the game's business. The buffer is stopped for both reads: the streaming
thread moves both halves, on notifications a stopped buffer does not raise.

The chapter's own snapshot is then taken again, because the one from the playback holds the sound as
the playback had it: left alone, the first death in that chapter would rewind the music to the top
and undo this. It costs one copy of a chapter on a frame that has just run a stage's worth of
updates. A chapter of a boss's own theme is left alone in both respects, the way that fight's retry
leaves it: starting a long theme over every attempt is worse than the jump in it.

`song=-1` in the file is a stream that would not say — `mmioSeek` refusing, or a song that had
looped within a buffer's length of the reading — and a resume then leaves the music as the stage
started it. So is a file written before the field existed.

**What is asked, and where.** `どこから始める` / `Where to start`, with `つづきから` / `Continue` and
`はじめから` / `From the beginning`, on the game's own shot
type select — on the press that would have started the run, which orb holds back. Asked there rather
than beside the mode question because which run this is includes the character and the shot, and the
mode question is two screens too early to know them; that screen knows all three, the first two being
settled and the third under its cursor at `MainMenu+0x81a0`. Only where a run of that same difficulty,
character and shot was left, so the question never appears for a run there is nothing of.

**The press is held back in the input read**, `GetInput`'s answer having the front end's decide taken
out of it — `0x1001`, the shot button and return, which is what every one of its screens tests against
the frame before. That word is what `Supervisor::OnUpdate` assigns `g_CurFrameInput` from, so a press
taken out there is one no screen in the frame ever saw. Which is what `キャンセル` is: the press is
never handed over, and the screen carries on where it was, on the shot it was asked about. Nothing is
put back, and nothing has to be — beyond holding the key that cancelled back too, until it is let go:
it is still down on the frame the game carries on into, and what the shot type select does with it is
go back to the character select.

Handed over on one read, by putting those bits back in, where the answer is a run to start. The screen
then decides as it would have — `shotType` from its own cursor, `curState` to the run — rather than orb
writing the run's three numbers and the scene by hand. Two frames later the run is registered, and the
chapter asked for goes in there, that being the one frame the game will take a stage for a run it has
not built.

Not asked after the run is chosen. The shot type select does not fade or wait: 0x436dae writes the shot
and the scene in one go, and by the frame the run is chosen the front end has taken its own job out, so
a question asked there has nothing behind it to go back to. That frame is kept as the one place a run
whose press could not be held back is still asked about, and there the cancel it cannot act on is left
unread.

The line under `つづきから` says where that run stopped — the stage, the chapter by name, the
retries — and not which run it is, that having just been chosen. Under `はじめから` it says what
starting again costs, which is that the chapter left behind is written over as soon as the new run
reaches one; this is the last moment anybody is told so. The cursor starts on `つづきから`, not
because it is the likelier answer but because of what the two mistakes cost: a run picked up by
accident is a run put back where it was, while a fresh run started by accident is the file gone.

**And said before the press, as a mark rather than a question.** `中断データあり` /
`A run left unfinished`, with the
stage, the chapter and the retries under it, in the bottom left of that same screen while the cursor is
on a shot a run was left in. Nothing is frozen and nothing is asked for it: the screen carries on
running underneath and the line follows the cursor between the two shots.

The press is held back on that screen whichever shot the cursor is on, and not only where the line is
up. What the line says is read one frame after the cursor arrives — orb's own frame work runs after the
game's update — while the screen moves its cursor before it reads its decide, 0x436a88 against 0x436d79.
So a direction and the shot button on one frame start a run on the shot the cursor has just reached, and
holding the press back only under the line would leave that one frame able to lose a chapter. Where there
turns out to be nothing to ask, the press is handed over on the next read, which is a frame nobody
sees.

It is there because the choice that quietly loses a run is made on those screens, not on the question
after them: `MainMenu::RegisterChain` memsets the cursor, so somebody who left a ReimuB run and picks
ReimuA out of habit is offered nothing, gets a fresh run, and that run writes its own file —
measured, by doing it, and nothing on the game's own screens says otherwise. A practice run is not
marked, its stage being part of which run it is and not chosen until afterwards. The file is read when
the cursor moves onto another run rather than every frame, and it is a few hundred lines to read.
The file is read where the question is put rather than at startup, so what is offered is the chapter the
last session stopped at.

**The file goes when the run finishes**, which is the result screen: a run given up does not go
through one, so what is left holding a chapter is exactly a run that was left. A cleared run's file
is taken away — that run's own, which is the one it was being written to.

**What settles whether this works is the landing.** The reproduction line — the replay clock, the
buttons, where the player is, how many numbers the generator has given out, the score, the rank —
is written into the file for the chapter's frame and read again at the frame the playback stops on,
and the log says whether the two agree field for field and which field they first differ on. That
is the instrument for the one thing the mechanism rests on: that the numbers a stage reads at its
start are all written down. A player hit during the playback is reported too, and at which frame: a
path that survived does not do that, every death a run had having been rewound away, so a hit is a
playback that has come out of step at or before that frame.

`--no-resume` takes the whole of it out — the two hooks and the file — while leaving the chapters
it is built on, which is what a fault in a run that chapters alone do not explain wants.

## Pointdevice and normal

**Which of the two a run is, is asked where the run is started**, over the game's own title
menu, because that is where it belongs: a run with chapters and a run without are two different
things to start, and 紺珠伝 asks it in the same place. Not a key in `orb.yaml`, which is for
what somebody sets once.

**On the press, not after it.** The title menu's decide is held back the same way the shot type
select's is — `g_CurFrameInput & 0x1001`, tested at 0x437c1b against the frame before — and what the
cursor is on is what says whether orb has a question: `MainMenu+0x81a0` bounded to 0..=7 and jumped
through the table at 0x4381cc, of which items 0, 1 and 2 start a run and item 4 is the ranking.
Answering picks the mode and hands the press over, so the menu chooses its own item; cancelling hands it
over to nobody, and the menu is still on the item, with its cursor where it was and nothing of it
redrawn.

**Only where the screen would have acted on the press.** Each ignores its own decide for its first
frames — 20 for the title menu at 0x437c0e, 30 for the shot type select at 0x436c0a — and a press held
back over those and handed over afterwards is a keypress the game had thrown away, acted on late. Held
from a frame before either grace runs out, because what decides the holding is read after the game's
update and applies to the next read: tested for the frame it is on, the frame the grace expires on would
go unheld and a press there would start a run nobody was asked about. What the cursor is on is asked
apart from this, the mark on the shot type select being drawn from the same reading and having nothing to
wait for.

**The key a question was cancelled with is held back as well**, until it is let go or until the screen it
was cancelled on is gone, whichever comes first, on both screens. It
is still down on the frame the game carries on into, and going back is what each of them does with it —
the title menu puts its cursor on `Quit` (0x4381b0), the shot type select returns to the character
select (0x436c49) — where what was answered is about the screen the question was on. The press after it
is the screen's own again: cancelling a question and asking to go back are two different things to ask
for. It ends with that screen because a hold that outlived it would be taking the bomb and the pause
button out of the run started next, and out of what a resumed run writes down as its own buttons.

| | |
| --- | --- |
| 完全無欠モード / `Pointdevice Mode` | chapters, snapshots, the retry menu, the wash a chapter gets, the lives painted over, the retry count on the status line, and `pointdevice_score.dat` |
| レガシーモード / `Legacy Mode` | the game as it was: dying costs a life, a replay can be saved, and the score goes in the game's own `score.dat` |

On screen they are 紺珠伝's own two names, in whichever of the two languages that game itself calls
them, since that is where the mode comes from and those are
the names somebody who wants it knows. In the code, the log and the file it writes they are
pointdevice and normal — the English of the first, and what the second actually is.

**The game's own count of lives is painted over with a brush stroke in a pointdevice run**, with
`DISABLE` written on it. Dying there costs the chapter and not a life — the menu goes up, and the
snapshot that puts the chapter back puts the count back with it — so the row is the one thing left
on the screen still describing the game as it was. 紺珠伝 says it the same way across its own 残機
row, which is where somebody who wants the mode will look for it. Drawn on three conditions: the
mode, a run of somebody's own rather than a demo or a replay, and a chapter with a snapshot to go
back to — that last being the frames at a stage's start before its own snapshot has been taken,
where a death does still cost a life.

Asked of the run and not of the frame, unlike the death, which is a comparison against the frame
before and belongs to one. A run passes through frames that are not gameplay frames and are drawn
like any other: the single frame a stage transition is built in, which is a quarter of a second of
screen because the game builds the next stage inside it, and the frame a chapter is put back on,
where what orb knows of the frame before has deliberately been dropped. So what decides is whether
the run being tracked is one somebody is playing — settled where that stage's snapshot was taken —
and, past the end of the run, whether the game is still painting that row at all. A mark that
stopped on any of those frames would leave whatever the game paints in that row standing there
instead, which is the count: the game paints it from its own "this row changed" bits, and does not
ask orb first.

**The stroke is a picture of one and not a generated one.** Generating one was tried first, and
every version read as a smear rather than as a brush: a spine with a taper has the ends wrong, and
noise along an edge does not carry ink the way a hair does. So the stroke is a real one, and what
the tree carries is the picture of it: `crates/orb-core/brush.png`. `crates/orb-core/build.rs` bakes that into
the 144x30 of coverage the drawing wants, one byte a pixel — how far each pixel of ink on paper is
from the paper, cropped to the ink, averaged by area and put through two smoothstep passes.

The picture is grey, and that is a size and not a taste: only one channel of it is ever read, so the
colour one it was baked from gave the same 4320 bytes at three times the file. WebP was measured
against it and is the wrong trade — lossless came out *larger* than the grey PNG, 884KB against
460KB, and both of its modes want a decoder that cannot be sixty lines. What made the difference was
never the format.

In the build rather than in a script anybody runs, and that is the whole of why: a script means its
output committed beside the picture it came from, which is two copies of one stroke with the
editable one the wrong one. Only the inflate comes from a crate — walking a PNG's chunks and undoing
its row filters is sixty lines that say what the picture is, against eight crates that would say it
for every picture there has ever been. The picture's ink is 4:1 and the squash to 144x30 is what
fits a stroke between the score's row and the bombs'.

**The count is not painted out; the stroke goes over it.** Where the ink is dry, the stars show
faintly through — which is what they are, disabled rather than gone, and one gained still shows.
That needs the count drawn again underneath every frame, and the game is asked to do it: `Gui`'s
`flags.flag0`, the two bits set by whatever changes the count and taken one off by each draw, is
written back to 2 before the game draws. So the game erases that row and draws its own stars again
for orb, background and all.

**Which means every frame the game paints that row is a frame the mark has to go over.** The row it
paints last is the one left standing on the screen, and a run's panel outlives the run: leaving one
ends it on a single frame and the panel stays there until the front end has drawn its own screen. So
what ends the mark is the game no longer painting that row — its `Gui`'s own draw job gone from the
draw chain — and not the run ending. Ending it with the run left the count back on the panel for the
whole fade to the title.

The two strips the stroke reaches past that row are orb's to paint, and painted first: a panel that
is not repainted is one where a mark blended over what the last frame left hardens into its own
edges within a second. They are painted with the game's own panel tile — `front.anm`'s 32x32 at (0,
224), laid on the grid it lays it on, from the texture the game has loaded in slot 13 — rather than
with a colour of orb's, because the panel is a noise and a flat rectangle inside it reads as a
patch. What that also buys: what is left where orb stops drawing is the panel the game would have
painted there itself.

**Over the count and not over the `Player` beside it.** The count is where the stars are — from
(496, 122) rightwards, 16 apart, and eight of them at the most, which is 496 to the right edge of
the output — and it is a row the game will repaint on being asked. The label is drawn when the
stage begins and never again, so a mark over that one would still be on the screen after the run it
belonged to. What repaints what, and how seldom, is a fact about the game: `Gui::OnDraw` erases and
redraws a row of the panel only while that row's two bits are set, and the background behind the
whole panel only for the first 250 frames of a stage, that being where its own script reaches
`ExitHide`. Nothing repaints any of it after that unless the game is told to clear the back buffer,
which is the setting orb reads for `clears_back_buffer`.

**Neither is an answer too.** `x` — the game's own bomb key, which its menus read as back — escape,
or the pad's cancel, and the front end goes back to the title the way its own back button does:
`gameState` to `STATE_CHARACTER_LOAD` and its timer to zero, which 36 frames later reaches
`STATE_STARTUP` and falls through to the menu. Copied from the difficulty select's `RETURNMENU`
branch rather than undoing what the chosen item did, because the sprites are already running the
fade that branch would set and the cursor is already on the item that was chosen.

**A menu of orb's has to read the pad itself.** Freezing the game stops its input read, so on those
frames a pad drives nothing — which looks like the pad being broken, since it worked on the game's
own menu one keypress earlier. So both of orb's menus ask the game what the pad is doing, and the
answer is read the way the game reads it: its own DirectInput controller where it has one, the
winmm sample orb's thread keeps where it has not, and either way through
`g_Supervisor.cfg.controllerMapping` — the same copy `Controller::GetControllerInput` reads. Which
of the two matters more than it sounds; see *Input*.

**Shoot decides; bomb and menu cancel.** Which is what the game's own menus do:
`TH_BUTTON_SELECTMENU` is `TH_BUTTON_ENTER | TH_BUTTON_SHOOT` and `TH_BUTTON_RETURNMENU` is
`TH_BUTTON_MENU | TH_BUTTON_BOMB`, so either of those two is a back there. The settings dialog reads
it the same way, out of the game's configuration file rather than the game's memory, because a
button that answers a question before the game starts and cancels one inside it is worse than
either.

The menu button decided for a while instead. On the pad orb was first run with it was button 0 —
where a thumb rests — so the most obvious button on the pad closed the question instead of answering
it, and three launches went by before the mapping said why. That is what the launcher printing the
mapping it read is for, that being the only place it is written down in a form anybody can look at:
on the pad these were last run with, shoot is button 0 and menu is button 1. A pad where the two are
the other way round is a pad on which orb's menus will look like they cancel themselves, and the
printed line is where to see it.

Up and
down come from that mapping too, and from the Y axis read the way that function reads it — the centre
halfway between `g_JoyCaps`' bounds and a dead zone of a quarter of the travel either side, its low
side being up since the axis is measured downwards — and from the hat, which is where a d-pad reports
and which the game itself does not read at all. All of it on the press
rather than the holding, and the previous frame's state is kept through the grace frames as well —
a button held from before the menu opened must not become a press the moment the grace ends.

The question goes over three of the title menu's items — a full run, the Extra stage, practice —
because each of them starts a run, and over `Score`, because the two modes have a ranking each.
One answer for both: orb is in one mode at a time, and the ranking of pointdevice runs *is* the
file pointdevice runs are written to. So looking at the other ranking puts orb in the other mode,
and the next run asks again — which is why the mode left behind cannot be what a screen reads from:
every item that opens the file, the ranking included, asks first.

**How the moment is caught.** `g_MainMenu.gameState` at 0x6dc8b0 — 0x81f0 into the 0x10f34-byte
`MainMenu` at 0x6d46c0, past its `AnmVm vm[122]` of 0x110 each — is read once a frame while the
supervisor says the front end is what is running. All three of the items that start a run set it to
`STATE_DIFFICULTY_LOAD`, and `Score` sets it to `STATE_SCORE`; each then waits 60 frames before the
game acts on it, so there is a second to spare. It is the *change* that is acted on, and coming
back from the difficulty select goes through `STATE_CHARACTER_LOAD` rather than through either, so
a choice is caught once. `MainMenu::RegisterChain` memsets the whole struct and writes the state
afresh, so nothing there is left over from the last time round.

**Both of the supervisor's states have to say front end**, not just `curState`.
`Supervisor::OnUpdate` is chain priority 0 and assigns `wantedState = curState` as its last act, so
a screen that leaves by setting `curState` itself — which is how the ranking leaves — produces one
frame ending with `curState` already saying front end, `wantedState` still saying where the game
was, and the front end not yet rebuilt. `gameState` on that frame is whatever the screen being left
was entered from, and for the ranking that is `STATE_SCORE`: leaving the ranking read as choosing
it, and the question was asked again for nothing. Seen in the log as
`menu: Scores chosen, asking which mode` on the same millisecond as the score file being written on
the way out. Requiring `wantedState` too excludes exactly that frame, and the frame after it the
memset has taken the stale state away.

Nothing has to be undone by the answer, which is why the question can be asked after the game has
acted on the keypress rather than instead of it: what the game has done by then is start a fade
and set a state, and both of them are wanted whichever mode is chosen. The game is frozen the
frame after — `RunCalcChain` returning `CHAIN_BREAK` while the drawing carries on, the mechanism
the retry menu uses — so one update of the front end runs after the keypress and none of the
second that follows it. The play field's viewport is *not* set for these frames, unlike a retry:
what is underneath is a menu, and `prepare_frame` has already given the frame the whole output.

**The key that answers it does not also reach the game.** `g_CurFrameInput` is left as every
button, so that the `held & ~held-last-frame` every one of the game's own `WAS_PRESSED` is finds
nothing on the frame the game carries on into: `Supervisor::OnUpdate` runs first in the calc chain
— priority 0 against the main menu's 2 — and its first act is
`g_LastFrameInput = g_CurFrameInput; g_CurFrameInput = GetInput()`, so that is the one thing which
reads it. What is genuinely held still reads as held, from the fresh read; only the edge goes, and
only for that frame.

Not zero, which is what the game itself writes into those three at a scene change: zero leaves
`g_LastFrameInput` empty and turns every button still down into a fresh press, which is the
opposite of what is wanted. The game gets away with it because the screens it changes to guard
their own first frames with timers — which is also why doing nothing here would have worked, and
is exactly the kind of reason not to rely on: it is a property of the screen behind the question
rather than something orb decided. A retry needs none of this, since the snapshot restore puts the
same state back with the rest of `.data`.

**The fade goes on underneath it**, because `MainMenu::OnDraw` is what advances it —
`numFramesSinceActive` against the 60 the chosen item set `framesActive` to — and the drawing is
the half that is not frozen. So the title screen darkens to the black the item's fade was heading
for and stays there while the question is up, which is a background for it rather than a problem
with it. The vms' own scripts do stop, since those are stepped by `ExecuteScript` at the end of
`MainMenu::OnUpdate`.

**No replay is offered for a pointdevice run.** What one recorded here would play back as is a run
nobody could tell from a flawless one: the game keeps its record of inputs in the heap a snapshot
covers — `ReplayManager` is `new`ed at 0x42a27f and each stage's record allocated per stage — so a
retry rewinds the record along with the run, and each attempt writes over the last from the
chapter's own frame. What would be saved is therefore the path that survived, with nothing in it of
the attempts that did not. That is a better reason not to offer one than a broken replay would be,
and it is the same property orb's own record of a run leans on — see *Picking a run up again*.
The screen that offers to save one is `ResultScreen`, reached through the job it registers —
`ResultScreen::OnUpdate` at 0x42d98e, the same walk the ending's object is found by — and its
`resultScreenState` at +0x8 is written from `SAVE_REPLAY_QUESTION` to `EXIT` on the frame it
arrives there, before the frame timer reaches the 60 that starts the question's own animation. So
no part of it is ever drawn. `EXIT` is the game's own way out and not something invented for
this: it is the state a practice run's result screen is registered in, and its `OnUpdate` case
sets the supervisor back to the title menu and takes the job out — which runs
`DeletedCallback`, so the score file is still written on the way. The alternative was answering
the question for the player, which means writing the interrupt each of the screen's 38 sprites is
to run next and then waiting out the fade they play.

**That question is the only state of that screen orb writes.** `RegisterChain` starts a finished run's
screen in `WRITING_HIGHSCORE_NAME` — `EXIT` for a practice run, which is why both are the states the
game skips its own parses in — and the name entry and the stats screen after it are screens whoever
just finished the run reads. Nothing of orb's is written into either, and what used to write into the
first of them is above, under the ranking orb has built to write.

Refusing the write instead — which is what `--clear` still does, since a cleared run *does* reach
that screen — would leave somebody naming a replay file that never appears.

**Nobody is asked** where there is nobody to ask: a pass over a replay (`--collect`, `--judge`,
`--replay`), a tuning session (`--tune`) and a clear (`--clear`) take the mode they are given,
which is pointdevice. A menu frozen on a question nobody answers is a pass that never ends. Nor
is anything asked with `--no-chapters`, where the mode is normal because there is nothing for it
to be: orb is loaded with none of its own work happening. And nothing is asked without the
overlay, which is what would draw the question — a frozen game with an invisible question over it
looks broken, and what is lost by not asking is the mode orb is in already.

## The frame loop

```
prepare_frame          the game's full-output viewport, and its background clear
DwmFlush()             returns just after a blank
wait                   until (blank + one frame) − (our drawing + the compositor's)
update(chain)          the game's logic; reads the keyboard as its first act
draw(chain)            between BeginScene and EndScene
hold                   until the blank before the aimed one has gone, if it has not
Present()              handed over, not waited on
```

The update runs before the draw, which is the opposite of the game's own order and removes a
frame of lag. The work is done at the end of the frame's turn rather than the start, so the
input it reads is as recent as it can be and still reach that frame.

`Present` does not wait for anything: windowed with `D3DSWAPEFFECT_COPY` it queues the frame
and returns. `DwmFlush` is the call that waits for the compositor, and it sits at the top of
the next frame — so a frame reaches the screen when the *following* frame's flush comes back,
and that is where input-to-screen is measured from and to. It is also what says whether the
frame made the blank it was aimed at, since what the flush waits for is that frame being
composed: see *The compositor's drawing time*.

**Cadence.** One game frame is a whole number of the compositor's refreshes, counted in the
period `DwmGetCompositionTimingInfo` reports: two at 120Hz, one at 60Hz.

**The compositor's, because that is the grid a frame can be put on.** `DwmFlush` returns at its
blanks and at nobody else's, whatever monitor the window is on, while `EnumDisplaySettingsW` answers
about the window's own panel — which on a mixed-rate desktop are two different rates, measured.
And the frames really are composed against that grid: handed over too near a blank they miss it and
handed over far enough ahead they make it, with the same threshold for a window covered by a
full-screen one as for a window in front.
`MonitorFromWindow` and `EnumDisplaySettingsW` are then only the rate to count in where the compositor
will not say, and otherwise what the log calls the desktop: a frame shown on the compositor's blank
still has the window's own panel to reach, which is worth a line and decides nothing.

Counting in the monitor's rate while flushing on the compositor's blanks is what ran a 120Hz window at
72 frames a second on a desktop the compositor timed at 144 — the frames went on 6944µs blanks while
the cadence asked for two of the monitor's 8333µs ones. It is not a case to refuse: it is the ordinary
fractional cadence at the compositor's rate.

**The rate is rounded and the rounding may not decide anything.** A period in whole microseconds puts
an NTSC-derived display at 119 or 59 rather than 120 or 60, and a rate within two per cent of a
multiple of 60 *is* that multiple and gets its constant count. Read as fractional instead, 119 sends
the grid chasing an exact sixtieth against a rate 0.8% away from one, which it settles by putting a
one-refresh frame in about once a second — a live fault until a 119.88Hz display was actually run. The
spacing itself is the measured one rather than the nominal multiple's, that being the truer of the two:
a 119.88Hz refresh is 8341µs, which 120 would put at 8333 and 119 at 8403.

A rate that is not a whole multiple of 60 has no one cadence to keep — 144Hz is 2.4 refreshes a
frame — so there each frame goes on whichever blank is nearest where a sixtieth-of-a-second grid
has got to: two refreshes, three, two, three, two. Measured over 600 frames at 144Hz, `gaps in
refreshes 2x360 3x240`, which is 2.4 exactly and 60.00 frames a second. Every frame is still shown
at a blank rather than at a moment on a clock that lands wherever it likes among them.

Individual frames there are unequal by a refresh — 13.9ms against 20.8 — which is what 144 over 60
comes to and not something pacing can undo.

**Both the grid and the phase it is measured against are absolute**, and that is the whole of what
makes it come out even. The aim is the blank nearest the grid, counted from a kept blank; and only
a frame that landed where it was aimed moves that phase on. Measuring the next aim from the last
*landing* instead cannot correct itself — a frame that lands a refresh late becomes the reference,
so the following aim asks for one refresh fewer and the lateness is absorbed rather than undone.
That settles: at a fractional rate the aim comes out averaging *less* than the display wants while
some of the frames land a refresh late, and the two add back to the right average — so the rate
reads correct while those frames are shown somewhere nobody asked for. Anchoring the aim instead
leaves none of them.

A grid moment the blank in hand has already passed is a frame that has been missed, and it is
dropped: the grid starts again one frame from that blank. Left where it was, the aim comes out at
one refresh and stays there until the difference is made up — and making it up means an update per
refresh, so the frames that were missed are paid for by running the game fast, which brings none of
them back.

A rate that *does* divide is given the same count every frame instead of following that grid.
A display sold as 120Hz is often 119.88, and chasing an exact sixtieth there would spend three
refreshes on a frame every few minutes to make the difference up. Two every time is 59.94fps —
a tenth of a percent on a clock nobody can see, against a hitch anybody can.

Two things are paced by the clock and nothing else is: a desktop with no compositor to ask, where
nothing reports a rate at all, and a display under 60Hz, which has no blank to put a sixtieth of a
second on — one frame per blank there would run a 50Hz display's game at 50, seventeen per cent slow
with the music to match, and the clock at least keeps the game's own speed.

**The window being behind is not one of them.** It was, and measurement is what took it off the list: a
window behind, one covered by a full-screen window and one minimised all flush at the compositor's own
rate with every gap one refresh, `Present` never answers that there is nobody to show the frame to, and
the lead a frame needs to make its blank is the same whether anybody can see it. `always_draw` is on by
default, so this is every alt-tab away from the game.

A replay being run fast, or a run being cleared fast, keeps the cadence like anything else: `--speed`
is updates per drawn frame, so the frames come one per turn and only carry more of the game with them.

`DWM_TIMING_INFO.cRefresh` counts compositions of the window rather than refreshes of the
display, and is not used.

**The compositor's drawing time.** The window between `Present` and the blank is not idle: it is
where the compositor composes the desktop and gets it onto the screen for that blank. So a
frame's turn holds two drawing times, the game's and the compositor's, and both have to finish
before the blank or the frame is shown at the one after.

How long the compositor wants is not something it will say, so it is measured, and what measures
it is `DwmFlush`'s own return.

The flush waits for the compositor to compose the next frame rather than for the next blank as
such, so it returns at the blank the frame just handed over *reached*. Its overshoot against the
blank that frame was aimed at is therefore a per-frame answer to whether the frame made it, and
the two cases do not overlap: a frame that made its blank comes back near it and one that missed
comes back most of a refresh late, so the two are a refresh apart while the aim is good to a few
hundred microseconds. Half a refresh is the boundary and one frame decides it — nothing to average,
and no waiting for a fault that happens a few times in six hundred frames.

Raised 100µs the moment a frame misses, and **never lowered**. Every microsecond of it is input
lag on every frame, so the least that works is what is wanted — but the only way to learn that a
value is too little is a frame missing its blank at it, so a value that comes down is a value
being wagered again, and a lost wager is a stutter in the middle of a run. It starts at 2500µs
and climbs from there; a display wanting less pays the difference in lag and nothing else.

Shaving it back was tried and is why it is not done. A walk downward passes the real edge without
dwelling anywhere long enough to catch a value that only fails sometimes, and each step it takes
below that edge is paid for by a frame missing its blank on the way back up — a stutter a step,
for as long as the walk lasts.

Raised with it, by the same 100µs, is the budget: the budget is the drawing's time and the
compositor's together, so a share that grew without it starts the frame the step was taken for
exactly as late as the frame that missed and hands it over exactly as near its blank.

**What a step may be taken for is decided by what a step can do**, which is hand the frame over
earlier. Four kinds of miss are counted and only the first of them climbs:

| | |
| --- | --- |
| a frame exactly one refresh past the blank it was aimed at | the share was short, and a step is what answers it |
| more than one refresh past it | past anything a step could reach: the share is held under a refresh, so a composition longer than one puts its frame there at every share there is |
| the frame after one of those, and one frame only | still picking itself up, and not the compositor's to answer for |
| a frame whose own drawing outgrew its budget | late whatever the compositor had been given |

The three excused ones are there because a run found them. A stage load stops the game long enough
that the frame it lands on lands many refreshes out, and the frame after it is aimed off the blank
grid — the aim is measured from where the last flush returned, and a flush called after its blank
has gone returns at once rather than at a blank — so climbing for either raises the lag for the rest
of the run to answer for something the compositor never did. The drawing's own overrun is worse: a
heavy frame climbed the share to its ceiling, and with the budget capped at the same figure the
drawing had no allowance left, so every frame afterwards reached the compositor late and asked to
climb again — a run that never recovers, from one frame.

`orb-e2e`'s `pacing`'s `compose` section holds each of those: a compositor with room to spare never
moves the share at any rate a display reports, a composition longer than a refresh never moves it,
and every frame that is a refresh late moves it by exactly one step.

**Two ceilings, and only one of them is a refresh.** The compositor's share has to stay inside one
refresh, because the frame is handed over that far before the blank it is aimed at: hand it over
earlier than the blank before that one and the compositor takes it at the earlier blank. The share is
three quarters of a refresh. Getting it wrong is invisible at 120Hz, where half a game frame is
exactly one refresh; at 144Hz a refresh is 6944µs and half a game frame is 8333, and a share that
size collapses the gaps to about one refresh apiece — a hundred frames a second, the game running
fast.

The budget is a whole game frame less a quarter, and does not shrink as the display gets faster.
That is what makes work heavy on every frame coverable: the budget's whole job is starting such a
frame earlier, and a game whose update and draw come to most of a frame still makes its blank. Tying
it to three quarters of a *refresh* is the mistake in the other direction — at 144Hz that is the same
5208µs as the share, so the drawing has no allowance at all and every frame reaches the compositor
late.

**Neither ceiling can keep the handover off the blank before the aimed one, and the hold is what
does.** How early a frame really goes is the budget less that frame's own work, and the work is not
a number until the drawing is done: the budget is tracked near the worst of the recent frames, so the
frame after a heavy one does almost none of the work the budget was set from and goes nearly the
whole of it early. So between the drawing and `Present` the frame is held until the blank before the
one it is aimed at has gone. The target is that blank itself rather than a margin before it — a frame
handed over at a blank cannot have been composed for it — and what is left afterwards is a whole
refresh, which the share is already held under three quarters of, so the hold can never eat the
compositor's own time. `hold` is its own span in the pacing line, and it is zero on every frame whose
drawing finished after that blank, which is nearly all of them. It is not counted as the frame's work
either: a budget that grew to include a wait caused by the budget being too high would start the next
frame earlier, hold it longer and grow again.

What that answers is a run where one heavy frame — a spell card starting, and its sounds with it —
set the budget high enough that the frames after it, doing almost no work, were handed over further
ahead than a refresh. The blank one refresh earlier was then nearer than the compositor's share, so it
composed them *there*, the flush came back at that earlier blank, the anchor moved a refresh with it,
and the next frame went as early again. The turns came out a refresh apart instead of two — the game
running fast — and the log said so: `shown a refresh or more early, so the game ran fast for them`.

**Bounding the budget instead was tried and is why the hold exists.** Held under a refresh, the most
the budget could start a frame was a refresh before its blank, so work heavier than a refresh less the
share could not be covered at all — the cadence held up to that point and broke to a third of the rate
past it, from work a fraction of a game frame long. See
[docs/adr/0011](docs/adr/0011-the-frame-is-held-for-the-blank-before-the-one-it-is-aimed-at.md).

`orb-e2e`'s `pacing`'s `budget` section holds both halves: `work_that_is_heavy_every_frame_is_covered`
sweeps 4000 to 9000µs a frame and asks for the cadence at each, and
`a_spike_the_ceiling_admits_does_not_hand_the_frames_after_it_over_early` drives a 9000µs frame one in
three hundred and asks that no frame is shown early. A spike's own frame is shown a refresh late either
way — it started against the budget the frames before it left — and the frame after it comes back onto
the grid, so the gap between those two handovers is a refresh short by arithmetic and is not the game
running fast. What says the game ran fast is the count of frames shown early, and it is zero.

`--compose=N` pins the share, which is how it is swept: pinned small enough that frames are known
to miss, then walked up until they stop. A pinned value is also the floor the work estimate is
clamped to, or the clamp would hold the sweep at `COMPOSE_FLOOR_US` and every reading below it
would be the same reading — held under the ceiling as well, since `clamp` panics when they cross
and this runs inside the frame loop.

**What the compositor will not answer.** `cFramesLate` reads zero through runs where 57 frames
of every 600 missed their blank, so it is reported and never acted on. `qpcFrameDisplayed`,
`cFrameDisplayed`, `cFramesDropped`, `cFramesMissed` and `cRefreshesDisplayed` are all zero,
while `cFrameSubmitted` and `cFrameConfirmed` in the same read move — so the call works and that
family is not populated for the desktop composition, which is the only thing
`DwmGetCompositionTimingInfo` will report on: it takes an `HWND` and accepts only null.

orb's own present-to-present gaps cannot stand in for it either. They say when a frame was
handed over, not which blank showed it, and they only wobble at all because the pacing is
anchored to the flush — a frame loop paced purely by the clock would show a perfect 16.67ms
while frames slipped a refresh apiece.

**The work estimate.** How long the frame's work takes is measured and tracked near the worst
of the recent frames rather than their average, because aiming at the average means missing
the handover on every frame heavier than it.

A frame that wanted more than the whole budget is left out of that. It is a scene being built rather
than a heavy frame — `RunCalcChain` takes a quarter of a second where a run ends and the next one is
set up — and it says nothing about what the frame after it will take. Believed, it pinned the estimate
to the ceiling, and the frames that followed, doing almost no work, were then started further ahead
than a refresh: handed over that early they were composed for the blank *before* the one they were
aimed at, `DwmFlush` returned there with them, so the anchor the next aim is counted from moved a
refresh early and the frame after was handed over just as early again. One frame per refresh is one
update per refresh, so the game and everything in it ran fast for as long as the estimate took to
decay back — after every stage load, and with nothing in the log saying so, since the buckets take
`max(0)` of the overshoot and a frame a refresh early read as one that landed exactly where it asked
to. Those are counted now.

**What the wait to a frame's own deadline is.** A waitable timer created with
`CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`, made on first use from the frame hook and kept for the run,
with the last `SPIN_US = 1500µs` spun rather than waited out. Nothing is asked of the system timer
resolution: the timer is not tied to the system tick, and `SPIN_US` is what the wait's own overshoot
is measured at.

A host that cannot create it is a host orb does not run on, and there is no second wait behind it. The
launcher asks for one before it starts anything, and where it cannot it says so in a modal and prints
a line and starts nothing; the DLL asks the same on its first wait, for the case the launcher was not
the way in, and says so and ends the process. See
[docs/adr/0006](docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md).

**What a late frame says.** The pacing writes a line per frame whose gap was not the cadence,
and the line accounts for the whole gap in spans that add up to it:

```
after present   what orb did once the last frame was handed over
loop            the game's own frame loop, between orb returning and being called back
clear           prepare_frame
pace            the frame's turn worked out, settle's display query inside it
flush           DwmFlush, and how far its anchor sits after the compositor's own qpcVBlank
wait            the rest of the turn
update sound draw
hold            waiting out the blank before the aimed one, zero on nearly every frame
present
```

The one that decides is not in that list. `DwmFlush` returns at the *next* blank, so a frame
must reach it before the blank that is its own turn — and the frame is handed over some
`COMPOSE_US` before that blank, a couple of milliseconds, which is all `after present`,
`loop`, `clear` and `pace` have between them. A frame that arrives after its blank has gone
waits out another whole refresh, and nothing it does afterwards wins that refresh back. So the
line leads with how far before or after its blank the frame reached the flush, measured there
rather than worked out from the gap afterwards: a gap of the wrong size says only that
something went wrong somewhere.

`after present` and `loop` are in the line because they used to be the only part of a frame
nothing measured, while being the part that time belongs to.

The anchor the arrival is measured against is when the last flush came back, which is taken to
be a blank and is not checked anywhere else. So the line says how far it sits after the blank
the compositor reports as its last: a flush that overshoots a real blank and a flush that
returns on time against an anchor a refresh early are otherwise the same line. That query is
made only while the pacing is being written, and after the flush, so it is spent out of the
slack rather than out of the compositor's own.

Per period, alongside the gap buckets, the worst arrival against the blank and how many frames
were past it — the rate being what says whether a stutter is one cause or the weather. Arrivals
beyond a whole turn are counted apart and left out of the worst: the frame after a stage load
is that late through no fault of the compositor, and one of those would otherwise be the
whole of the worst.

**What a run says while it is being watched is not written where it was worked out.** A
`WriteFile` takes what it takes, and one in the millisecond before a handover costs that frame a
refresh — a run stuttering because it was being watched. So everything a level or a switch can
turn off is held and written on the far side of the flush, where what is left of the turn is
slack: the pacing's own lines, the summary a second brings and the per-frame detail alike. What
is written where it stands is what every level keeps — startup, the faults, and where a run got
to — because a run that ends in a crash or a hang has to have its last lines in the file, and
the crash filter closing the log writes out whatever the faulting thread was still holding.

**The log is an instrument and is weighed as one all the same.** What writing it cost is written
down beside what it is reporting, per frame and per period, the frame's own thread and orb's kept
apart: the appends serialise on one handle, so a line from orb's own thread can still hold up the
frame's slack write, and a run whose stutters line up with what was written cannot say so unless
what the writing cost is written down too.

## Input

`Controller::GetInput` (0x41d820, `__stdcall`) is the one place every key the game sees passes
through: `Supervisor::OnUpdate` calls it once a frame, and the only other writer of
`g_CurFrameInput` is replay playback.

**Not read while the window is behind.** The game's keyboard is a foreground DirectInput
device — `DISCL_NONEXCLUSIVE | DISCL_FOREGROUND`, acquired once at startup — so the system
unacquires it whenever the window goes behind and every read fails. The game checks only for
`DIERR_INPUTLOST` and treats anything else as a success, which hands it an uninitialised stack
buffer as key state. Reading while behind also spends that single `DIERR_INPUTLOST` report on
a moment when re-acquiring cannot succeed.

**Re-acquired on the way back.** orb calls `Acquire` itself — `g_Supervisor.keyboard` at
`G_SUPERVISOR + 0x10` (0x6c6d28), vtable slot 7 — and holds the keys back until it succeeds.

**orb's own questions read the keyboard for themselves**, with `GetKeyboardState`, because each of
them is up on frames the game is frozen on and the game's own input handling is frozen with it. Every
key at once rather than a key at a time, that being what the call answers, and a call that fails reads
as nothing down rather than as whatever was down last — the state it left standing would be a key
stuck down on menus that act on edges.

**Nothing is down while the window is behind, and the way back is not a press.** Reading while behind
would let typing elsewhere drive the game, so everything reads as released — but zeroing alone makes
the return an edge, everything read as up and then read as down, which is a press by the rule those
menus use. So whatever is down on the first frame orb is reading again counts as already held. The
first read of all goes the same way: orb starting while a key is down is the same thing as coming back
to one.

**The pad half of that read is orb's.** `Controller::GetControllerInput` (0x41cfc0, `__cdecl`) is a tail
call inside `GetInput` that added a joystick's buttons to the keyboard's — reached from that function's two
exits, 0x41dc78 and 0x41e09d, and from nowhere else in the exe — and orb hooks it and does not call through.
What every pad does to the word the game acts on is orb's answer, for the device the game's own enumeration
found as much as for the pads it has none of. See
[docs/adr/0013](docs/adr/0013-the-pad-half-of-the-input-read-is-orbs.md), and the paragraphs below for each
of the rules that function had.

Where it got a pad was `joyGetPosEx(0, JOY_RETURNALL)`, winmm's, through the exe's import
table. The DirectInput branch beside it was only entered where the game's `EnumDevices` found
an attached game controller at startup — where none was, `g_Supervisor.controller` (0x6c6d2c)
stays null and every frame went to winmm.

**Which is settled at startup and never again.** That pointer is written in one place in the whole
exe: the enumeration's own callback at 0x423da0, which calls `CreateDevice` only while it is still
null. The enumeration is `EnumDevices(DI8DEVCLASS_GAMECTRL, …, DIEDFL_ATTACHEDONLY)` at 0x423d12,
reached from the one call at 0x423a1f, in the startup that also creates the keyboard device. So a pad
attached before the game starts is one the game holds a DirectInput device for all run, and a pad
plugged in afterwards is one nothing ever creates a device for — the run reads winmm for it instead,
and so does everything of orb's that asks the game where a pad is. Which decides nothing on a machine
whose winmm has the pad, and decides everything on one where winmm has only the phantom a pad in
XInput's second slot leaves behind.

Where nothing answers, that call was measured taking a large fraction of a frame and spending nearly
all of it on the CPU, so being work rather than waiting there is nowhere cheap in the frame to put it.
Where a joystick
does answer it is fast, so what it charges for is the looking and not the reading. **So the reads are on a
thread of orb's own** and every frame is answered out of the last sample it took: every 4ms while a pad
answers, once a second while none does, and never sooner than the read itself took, so no device can hold a
core of its own.

The exe's import of that call is redirected all the same, for the one read of a joystick the game still
makes for itself: the startup check that asks whether there is one at all, which is the slow call on a
machine with nothing plugged in.

Both of those reads — the position and the caps behind it — go through `orb_api::joystick`, and XInput's
through `orb_api::xinput`; the thread is spawned through `orb_api::thread::spawn`, which carries onto it
whatever host the caller reads through: the installation is per thread, so a thread spawned any other way
would read the machine's own winmm whatever was installed. Which is what lets an e2e test plug a pad in — see
*Running the game with no game there*.

The device the game's own enumeration found is not sampled on that thread and cannot be: it is taken
`DISCL_EXCLUSIVE | DISCL_FOREGROUND`, so it is read where the game read it — once a frame, inside the read
orb answers — and its `Poll` and `GetDeviceState` are the fast case, a device that is there.

**Every pad the machine has is read, and not the one device the game asks about.** The game holds
exactly one controller and asks winmm about joystick 0 alone, so a second pad, a pad plugged in after
the game started, and — on this machine — the pad itself are pads it can do nothing with. 紅魔郷 is a
one-player game, so every pad on the machine is that one player's. orb's own thread reads all of them:
every index `joyGetNumDevs` reports, and each of XInput's four slots, through
`orb_api::joystick` and `orb_api::xinput`. Which of XInput's three libraries a machine has is a property
of the machine, so the one that is there is loaded by name — the same three the launcher's dialog loads.

**A socket with nothing in it is the expensive read**, which is what decides how often each is read: a
socket with a pad in it is read every 4ms, and every socket is looked at once a second. That second is
also what bounds how late a pad plugged in mid-run is picked up, and it is the wait the thread was
already taking while nothing answered.

**The pad the run is played with is the one last pushed.** Two pads plugged into a one-player game is one
in somebody's hands and one on the floor, and the one on the floor is the one whose stick may be resting
past its own dead zone: merged, that is a direction held down for the rest of the run. So one pad is
handed over — whichever was last done something to — and it stops being read the moment the other is
touched. What counts as touching one is a button or the hat changing, or a stick moving a sixteenth of
its travel from where it was last resting: a stick reports a few counts of noise while nobody is holding
it, and a pad whose noise counted would be the last-pushed pad for ever. A sixteenth is well inside the
quarter of the travel the game's own read needs before it calls an axis a direction, so nothing that
would move the player fails to count.

**Which is not a nicety, and here is what it is instead:** on this machine, with the pad in XInput's
second slot, winmm has no pad at all. `joyGetNumDevs` says 16, index 0 answers `joyGetPosEx` with
`JOYERR_NOERROR` and every field zero — `mid=413d pid=2104`, no buttons and no axes, which is what
Windows leaves there while the slot the pad is in is not the first — and 1 to 15 are all
`JOYERR_UNPLUGGED`. DirectInput has the pad, the game therefore has it, and everything of orb's that read
winmm had nothing. Those numbers are the measurement and they are beside `orb-core/joystick.rs`'s
`Sample::is_a_pad`; what took them is `scripts/joystick-scan.c`, which asks all three interfaces at once
and is what to run again the next time a pad works in one half of orb and not the other.

XInput reports its buttons in its own order and the game's configuration names them in DirectInput's — A,
B, X, Y, the two shoulders, Back, Start, the two thumbs — so an XInput pad's mask is translated into that
numbering rather than the mapping being second-guessed, and *shoot decides* stays true whatever the
player mapped. Its stick and its d-pad are translated the same way, into a winmm axis' travel and into
the angle a hat reports, so that where an axis becomes a direction stays the game's own arithmetic and is
not written twice.

**Both devices go into the word, and into a menu of orb's.** The one the game enumerated is read by asking
the game — `Poll`, then `GetDeviceState` into the `DIJOYSTATE2` the format it set fills, the buttons by the
same numbers the mapping names, since `rgbButtons` is indexed by the very number a winmm mask is shifted by,
and each axis against its own threshold in the ±1000 the game gave every axis — and the pad orb sampled goes
in beside it. Either of the two may be the same physical pad, and the same push twice is the same push.

A menu of orb's reads both for the same reason the word does, and one more: those frames freeze the game, so
nothing of the game's is reading anything on them.

The acquire after a lost device is orb's. The game takes its controller
`DISCL_EXCLUSIVE | DISCL_FOREGROUND`, so anything that took the foreground away leaves it
unacquired — and the read that used to ask for it back is the read orb answers. Asked for once and the frame
given up, the way the game asks: its own loop tries 400 times, at 0x41d2a5 to 0x41d2f0, and a frame spent in
that loop is the thing orb exists to protect.

**A device with no buttons and no axes is not a pad.** What it answered is what the game's own startup check
is handed, that being what the game would have read for itself, but it drives nothing of orb's: the axes of
a device that has none describe nothing. The log names it rather than reporting a pad.

What the log says of the sockets is one line each and only where what a socket answers changes: a socket
that has never had anything in it says nothing, sixteen of those being sixteen lines saying nothing, and
a machine with no pad anywhere gets one line for the machine instead. A line also says which socket the
pad the run is being played with is in, whenever that becomes a different one.

**Every pad is measured against the bounds its own caps report**, which the sample carries beside the
position. `GetControllerInput` placed the centre of an axis at `(wXmin + wXmax) / 2` with a dead zone of a
quarter of the travel, out of the `JOYCAPSA` at 0x69d760 — one struct, which the game filled once in its
startup check and only where a joystick answered there. A pad plugged in later was measured against zeros,
its centred axes reading as held over, and the run spent the rest of itself with two directions held. Watched
happening on this machine.

orb used to write the answering device's caps into that struct with every sample. It does not any more, and
nothing does: every read of that struct's fields was in the branch of the read orb now answers — 0x41d18b to
0x41d22f, the address's only other mention in the exe being the `joyGetDevCapsA` call that fills it — so it
has no reader left. Which is a write into the game's memory gone, and one less thing for a chapter's restore
to put back.

There is no setting for the read. There was one — it cost most of a frame and turning it off was the
only way out — and now that a thread of orb's own takes the samples there is nothing left for it to be off
for. What that read costs is the `joystick=` span of the `perf:` line, inside the `input=` it is part of.

**The d-pad moves the player, which the game itself does not.** `GetControllerInput` reads a pad's two axes
and neither of the two fields a d-pad reports in: not winmm's `dwPOV`, at +0x28 of the `JOYINFOEX` it
fills on its own stack, and not DirectInput's `rgdwPOV[0]`, at +0x20 of the `DIJOYSTATE2` its other
branch fills. The whole function was read for both and reads neither, so a d-pad does nothing in the
game at all — while driving the launcher's settings dialog and orb's own menus, both of which read
one, which is a pad that works everywhere except where it is being played with.

So orb puts the direction the hat is pushed into the word, in the bits that word names the four directions
by: 0x10 up, 0x20 down, 0x40 left, 0x80 right, read off `Controller::GetInput`'s `GetKeyboardState` branch at
0x41d86d, 0x41d899, 0x41d8c7 and 0x41d8f4 and off both of `GetControllerInput`'s own branches. `dpad_moves`
in `orb.yaml` is the switch, on by default, and `false` leaves the word what the game's own read would have
made of the same pads.

Every pad's hat, which is both of the devices there are to read — and it costs no extra read of either:
they are read once for everything, in the read orb answers. The word is made there and so before it is
written down, which is what makes a direction pushed on a d-pad a direction the run recorded; added
afterwards it would be a resumed run playing itself back without it.

**And the rest of a pad goes into that word by the game's own rules.** Which bit each of the nine mapped
buttons is was read off both of `GetControllerInput`'s branches — shoot 0x1, bomb 0x2, focus 0x4, menu 0x8,
the four directions 0x10 to 0x80, skip 0x100, set by `SetButtonFromControllerInputs` (0x41d600) at 0x41d01b,
0x41d0d5, 0x41d04a, 0x41d0ef, 0x41d109, 0x41d123, 0x41d13d, 0x41d15a and 0x41d177 and by
`SetButtonFromDirectInputJoystate` (0x41d580) at 0x41d34e, 0x41d40d, 0x41d37f, 0x41d42a, 0x41d447, 0x41d464,
0x41d481, 0x41d4a1 and 0x41d4c1. A mapping entry below zero names no button.

**Nothing but the hat is behind a setting**, and that is the line: the hat is what the game reads on no
device at all, and everything else is what its own read would have made of the same pads.

Focus is the one of the nine that is not simply a button becoming a bit. Where the mapping gives it a
button of its own the game sets 0x4 from that button, and so does orb. Where focus and shoot are the
*same* button there is no focus button to read: the game holds the player still off
`g_IsEigthFrameOfHeldInput` at 0x69d8f4 instead — the count of frames that button has been held, raised to
16 (0x41d06e, 0x41d3a4) and read against 8 (0x41d08e, 0x41d3c3), and brought back down by 8 a frame while
it is above 8 and to nothing below that (0x41d0ac, 0x41d3e3) — which is what makes holding the shot button
a way to move slowly, and the whole reason anybody maps the two together.

**That count is the whole reason the read came into orb.** It is fed by whichever pad the read looked at, so
with the read the game's own it followed the device the game holds and nothing else — a pad it has no device
for could shoot and never hold the player still, and orb could not raise the count from outside either: the
game's read runs first in the frame and has already brought it down by the time a hook over `GetInput` is
entered. With every pad read in one place there is one count of one thing, and orb keeps it where the game
kept it, in that field, so a chapter restored rewinds it with the rest of `.data`.

## How much of the screen the game gets

The arguments of the game's own `CreateWindowExA` are rewritten on the way through, so the
window is the size it is going to be from the moment it exists — no frame to remove afterwards
and nothing to flash first. Its window class gets a black background brush, and that is the
letterbox.

`screen: fullscreen` is that window borderless — `WS_POPUP | WS_VISIBLE`, and nothing else,
since a caption or a frame is exactly what puts a border on one — covering the monitor.
`screen: 1280x720` is that window with a caption to move it by and a system menu to close it
with, centred on the monitor, and nothing to resize it with: the size is one of the settings, so
dragging the edge would be a second place to say it and the one that is not written down. The
size is of what is *inside* the window, `AdjustWindowRect` being what turns it into the window to
ask for, so `1280x720` is 1280x720 of game however thick the host's frames are. A window too
big for the monitor goes against its top-left corner rather than half off the top, since a caption
above the screen cannot be dragged back onto it.

**Always a window, either way.** The game's own fullscreen setting is overruled before it creates
anything: a game that has taken the display exclusively has no window to size, and orb needs one
to write its numbers in the black beside the game.

**Display scaling is off**, said with `SetProcessDPIAware` at the same moment — before the window
exists, which is the only moment it can be said. So a size in `orb.yaml` is that many pixels of
screen, and the monitor a fullscreen window is measured against is the whole of it. The size the
window came out with is logged beside the size that was asked for, and again when the device is
created, because those are two different numbers whenever anything in between has an opinion.

The back buffer stays 640x480. Windowed, the game asks for `D3DSWAPEFFECT_COPY`, the swap
effect that honours a destination rectangle on `Present`, so the back buffer goes into a
centred rectangle of the game's aspect ratio. Which is why a 16:9 window is worth offering at
all: the black it leaves down the sides is where the status line goes.

**Anything outside the game that squares windows up wins**, and there is nothing orb can do about
it: such a tool acts on the window after it exists, which is after every moment orb has. It takes
the black beside the game with it, and with that the numbers written there. Seen happening: a window
created with exactly the client asked for, still that at the device, and squared up to 4:3 filling the
monitor's height some seconds later. The client the window came out with is logged next to the size
asked for, and again at the device, so a run where this happens says so rather than looking like orb
getting the size wrong. vpatch's `[Window]` section is the same hazard from inside.

A monitor-sized back buffer would not scale the game: its 2D drawing uses `D3DFVF_XYZRHW`,
whose coordinates are already screen space, so a viewport does not transform them. Scaling
that way would need a render target and a full-screen quad.

## The mouse pointer

**It goes once the mouse has been still for three seconds, and comes back on the frame it moves.**
Neither game is played with the mouse and orb's answer to every display setting is a window, which is
where Windows draws a pointer — so without this the arrow sits over the playfield for as long as nobody
moves it away. Where the pointer is is read once a frame with `GetCursorPos`, on every frame the loop
takes and the ones that draw nothing included. A host that will not say where it is has it left alone:
a pointer orb cannot follow is one it cannot tell has stopped, and one taken off the screen by a launch
that has lost sight of the mouse is one nothing puts back.

**Only while the game's window is the one in front**, and a window going behind puts the pointer back —
with the wait then running from the frame it comes forward rather than from the last movement of the
mouse. Which is the question orb's own keys are read behind as well, asked for a different reason: keys
read with another window in front are keys somebody typed at that window, while no movement of a mouse is
ever the game's to act on. What it answers here is how far the counter below reaches — the pointer over
another program's window is not something orb has measured itself into, and one over a window that is not
in front is in nobody's way.

**Whether the pointer is drawn is one counter the whole process shares**, and it is a counter rather
than a flag: `ShowCursor` moves it one step per call, and Windows draws the pointer while it is not
negative. **The game moves it too.** 紅魔郷 answers `WM_SETCURSOR` itself — its window procedure at
0x420d40 sends `0x20` to the case at 0x420dc0 — and with the game in a window, which is every launch of
orb's, that case loads the arrow, sets it, asks for the pointer to be drawn, and answers 1 so
`DefWindowProcA` never sees the message. One of those arrives per movement of the pointer over the
window, so the counter climbs by one for each and nothing in a windowed run ever lowers it: a single
`ShowCursor(FALSE)` from orb would leave the pointer exactly where it was, and what it would take
instead is one call per mouse movement since the window came up.

**So the exe's `ShowCursor` import is patched and the game's asks are answered rather than passed on**,
which leaves the counter orb's alone — one step takes the pointer off the screen and one puts it back.
All of them are answered and not only the ones that would fight a pointer orb has taken off: a call let
through while the pointer is drawn changes nothing on the screen and moves the counter all the same, and
a step taken afterwards cannot cross an edge that has moved. What the game gets back is the counter orb
last read, which it looks at in none of its six calls. A launch where that entry could not be patched
says so in the log and leaves the pointer as the game has it, there being no counter left to win.

Three seconds because nothing in either game wants the pointer at all: the wait is there to keep it from
being chased away between two movements of a hand that is still using it, and no longer.

**`hide_mouse` in `orb.yaml` is what asks for it**, on by default and one of the switches the launcher
offers — 時間経過でマウスカーソルを消す. Off leaves the pointer to the game, and leaves the `ShowCursor`
import unpatched with it: what orb does about the pointer is own that counter, and there is nothing to own
where the pointer is not being taken off. The log line says which of the two a launch got.

## The status line

The chapter, `RETRY`, `INPUT LAG` and the frame rate are drawn with GDI onto the window, in the
black beside the game, stacked in whichever letterbox bar is wider and lined up with the game's
edge. `SCRIPT` and `TABLE` join them while a table is being built, and `HOLD` while the game's
update is held. On a 16:9 monitor a 4:3 game leaves that black down the sides. Shown whatever
the game is doing, including demos and menus, and redrawn only when the text changes.

**In a run the chapter is named for the part of the stage it is in, and numbered inside it**:
`MIDSTAGE 3`, `MIDBOSS SPELL 1`, `BOSS NONSPELL 2`. A count of the chapters gone by says
nothing about where the game is standing, while the second spellcard of a fight is how a fight
is talked about and how a chapter worth grinding is named to somebody else. *Midstage* rather
than "stage" because "STAGE 3" beside a stage number reads as the third stage. The waves are
one run of chapters through the whole stage — its start, each midstage boundary, and the waves
handed back when a midboss goes down — since a midboss interrupts them rather than starting a
new set: the chapter after the midboss of stage 4 is `MIDSTAGE 3`, not `MIDSTAGE 1` again.
Where there is no chapter, in a menu, that line is not there at all.

**Neither it nor `RETRY` is there in a normal run**, where there are no chapters: an empty name
and a retry count that cannot move are two lines saying the run is not the one they describe.
What is left is the lag, the compositor's share and the frame rate, which are about the machine
and are the same in both modes.

The count comes out of the frames this stage's chapters began at rather than a counter per
kind, because a retry puts the mark back and the run then reaches the same chapters again. A
counter would name the same chapter one further along every time it was played.

**A pass building the table shows `CH 05` and why the chapter changed instead** — see *Building
the midstage table*. That is a different question from what the chapter is, and the one worth
having while what is being decided is whether the boundary belongs in the table: the name would
say the same thing about a boundary to keep and a boundary to throw away.

**The GDI it is written with is gdi32's own and not this DLL's import of it.** A translation patch
injected into the same game rewrites the import table of every module in the process, orb's included, and
puts its own replacement in the text calls — one written for the game's `-A` calls, which takes a `-W`
call's count of characters for a count of bytes. So the calls that carry a string or hand back a
font are read out of gdi32's export directory instead, which is the loader's own and is not rewritten;
the overlay's glyphs go the same way. See
[docs/adr/0015](docs/adr/0015-orbs-own-text-leaves-through-gdi32s-exports.md).

**Each draw clears what it is going to write in, and everything the last draw wrote in, to
black first.** A stack with fewer lines than the one before it, or a smaller font, does not
reach every row that already had text in it. Clearing and writing are two operations, so both
go into a bitmap of their own and reach the window as one `BitBlt`: done straight on the
window, a refresh landing between them shows a bar with nothing in it, and at 120Hz that is a
flicker. A stack taller than the black there is runs off the edge of that bitmap and is
clipped there, rather than being drawn over the game.

**The size fits the widest line into the black there is.** What goes here runs from three
characters to twenty — a chapter named for the part of a stage it belongs to is the long one — and
a line clipped at the bar's edge cannot be read at all, which is worse than small. The line is
measured at the full size and the height scaled by what it overran by, once per change of text,
which lands inside a pixel or two because character widths go with the em height. Nothing shrinks
below the point where the text stops being readable anyway; past that, clipping the odd long line
is the better trade.

The game's back buffer is the wrong place for it twice over: all 640x480 of it is shown inside
the letterbox, so anything there is over the game; and the game does not clear it between
frames — it draws the static background once and redraws only the moving parts — so anything
in a corner it does not repaint stays there and accumulates. The black is the window's own
background and Direct3D never touches it.

The retry menu is drawn with the Direct3D overlay instead, because it belongs over the game.

`INPUT LAG` is measured, from the keyboard read to that frame being on screen, across the two
frames those ends fall in.

## The ending

With `skip_ending`, the ending is run out inside the frame it starts on, so no frame of it is
drawn, rather than jumped over — the ending is also where the game sets the clear flag and
enters the score. Never entered during a demo or a replay.

**The staff roll is kept**, which the scene cannot say: an ending and its roll are all one
scene, so what the skip stops on is the script the ending is reading. 紅魔郷 runs an ending
from a `.end` file of one-character instructions, one file per part of it, and the last
instruction of every one of the six — `end00`, `end01`, `end10`, `end11`, and the
`end00b`/`end10b` a clear on Easy or with a continue gets — is `@Fdata/staff00.end`, the roll.
`F`, at 0x40fc06 in the interpreter at 0x40f7c0, reads that file over the one running and
carries straight on into it, so nothing else marks the handover: the scene stays 10 and
`isInEnding` stays set across both. The skip stops on the update the file changes on and the
roll plays from there a frame at a time, with the track it starts for itself — the ending plays
`bgm/th06_16`, `staff00.end` starts `bgm/th06_17` three instructions after the handover and in
the same update as it, so the track changing is a second signal for the same boundary.

The script is read out of the `Ending` object, which is on the heap and in no global: it is
reached by walking the calc chain for the job whose callback is `Ending::OnUpdate` at 0x4109c0
and taking that element's argument at +0x1c, the same field `RunCalcChain` passes to the
callback. The file it loaded is at `Ending + 0x1114`. `Ending::LoadEndingFile` at 0x4106d0 reads
the new file before it frees the one it replaces, so the address is always a different one
after a handover — which is what makes comparing addresses enough. With no script to compare,
the skip runs the scene out the way it did before there was one to read.

**A stage 6 ending is 29,040 updates** on its own and 36,932 with the roll, after which the scene
is 7, the result screen. So the limit on the loop is above a whole ending — fifteen minutes of
game time — because it is a limit per frame rather than on the skip: the frame the loop stops in
goes on to draw whatever the ending is showing by then, which at two minutes put five frames of
it on the screen. What the frame the skip runs in costs is not measured, only that it took five
refreshes or more — the pacing is what would say, its `gap at worst` being that frame.

**Reaching one at all** takes clearing the game: `GameManager`'s end-of-run branch at 0x418f4e
sends a replay to state 8 and practice to the result screen, and only a run somebody played gets
state 10. So `--clear` — see *Configuration* — is how the ending is looked at without half an
hour of playing well first.

`g_Supervisor.isInEnding` is at `G_SUPERVISOR + 0x19c` = 0x6c6eb4. The same flag gates the
game's own frame-rate counter (`cmpl $0x0, 0x6c6eb4` at 0x4240bb, jumping over the
`AsciiManager::AddString` at 0x4240e9), so it must not be written to hide that counter. The
counter is left alone; orb's numbers are outside the game's output.

## The score file

A run with chapter retries is not a run anyone played, so its score does not belong in the
game's ranking — the same reason no replay is offered for one. Refusing the write would lose
the record altogether, so the file is forked: while orb is in pointdevice mode an open of
`score.dat` becomes an open of `pointdevice_score.dat`, those runs are ranked against each other
in the game's own format and on its own screen, and `score.dat` comes out of such a run unchanged
because it is never opened.

**Which file is open is a runtime switch, not a setting.** The mode is chosen inside the game —
see *Pointdevice and normal* — and the fork follows it, because a normal run and the ranking of
normal runs are the game's own file: that is where a run anybody could have played belongs. A
launch starts in pointdevice, which is what orb is for, and in normal with `--no-chapters`, where
there is nothing to fork because nothing can rewind.

**A rewind does not take back what the game counted about a spell card.** The record lives in the
memory a snapshot covers, so restoring a chapter would undo the attempt the game counted when the
card started — a chapter retried ten times would show one attempt — and undo a capture, which in a
chapter that can be played again is that chapter cleared. So the block is taken before the restore
and put back after it, in both of the paths that restore one. Both counters, not the attempt alone:
capturing the card is what clearing the chapter *is*, so a rewind that took one back and left the other
would say a chapter had been cleared none of the tries at it cleared.

**A chapter is an attempt at a spell card only where one is up.** The card orb's own count goes against
comes out of `ds:0x5a5f98`, which holds the last card a boss was on and which **nothing clears** — not the
card ending, not the stage, not the run. So a chapter with no card in it reads whichever card came before,
and the nonspell that follows a spell reads the spell it follows: counted there, it would be an attempt at
a card nobody was fighting. Whether there is one to count is asked of `g_EnemyManager.spellcardIsActive`
instead, and asked *after* the snapshot is back rather than before — a spell chapter's snapshot was taken at
the card's own start, so what the restore puts the game back into is that card. Where no card is up the log
says so: `score: no spell card is up; no attempt counted`, said rather than left as a missing line, since a
line only written when the count was made would put the meaning in the absence.

**And only against a card the game has named.** 紅魔郷 fills all 64 records from `Rng::GetRandomU16` before
it reads the file — 0x41bc87, the whole 0x40 of each of them — and writes back only the magic, the two
lengths, the version, the card's number and the two counts, 0x41bca0 to 0x41bcd5. So the name a record
carries is the generator's until the card itself starts and copies one in (0x409720), and `catk`'s parse
leaves every record the file has no entry for exactly as the fill left it. The ranking screen draws that name
for any card whose attempts are not zero and 「？？？？？」 for the ones whose are (0x42e265 and 0x42e26e),
which makes a count added to a record nobody named the generator's own bytes on the screen. Refused by the
game's own test for the same thing: the byte a record holds beside its name against the sum of that name,
which is the comparison at 0x4097e8. **A row already written wrong heals itself**, and only by the card
starting for real: that comparison then disagrees, both counts go, and the name is written. A retry cannot do
it — a chapter's snapshot is taken after the name was copied in, so putting it back never starts the card
again.

**A run picked up keeps the names its playback learned.** The playback starts every card the run had passed,
which is why the counts are put back as they were before the buttons went in — a run picked up would
otherwise arrive having counted every card it passed. The names those starts wrote are not put back with
them: a name is what the playback *learned* about a card and not something it counted, and it is the one
thing about the card only the game knows. Putting it back leaves the record carrying the fill's own bytes,
which is a card orb then refuses to count against and a row the ranking never draws a name for again — the
landing being *inside* the card, nothing starts it a second time. So the block goes back with each record's
name, and the sum beside it, left as the playback left them. Which makes picking a run up the second thing
that heals a row already written wrong.

**Where a session stops without the game writing, orb has the ranking built and taken down.** The
write is reached from one place, so a run given up or quit with `ESC` leaves what it counted about
spell cards in memory and nowhere else — 紅魔郷 loses it, and orb does not. On the first frame the
front end is up after a run that ended anywhere else, orb writes `MainMenu.gameState` with what the
`Score` item writes — 0xa, from the handler at 0x437f56 — runs the updates that brings inside that one
frame with nothing drawn, and writes `curState` back to the front end the way the ranking leaves
itself. The deleted callback then writes the file. Bounded at 240 updates, against the 60 the front
end waits and the one each scene change costs; at 30µs an update the whole of it is under a frame's
worth of time and none of it is seen, the draw chain running once a frame after this.

**None is built for a run that finished, or for one still inside its ending.** The screen a run
finishes at writes for itself, that deleted callback being the same one — so a run that reached
`STATE_RESULTSCREEN_FROMGAME` has nothing left waiting, and orb drops the request there. That a
ranking cannot be built from either place is what makes it worth naming rather than leaving to the
front end never coming up: there is no front end to act on 0xa on the result screen or over a staff
roll, so all 240 updates went on whichever of those was running and the request was then undone — and
undoing it wrote `RESULT_SCREEN_STATE_EXITING` into the screen that *was* up. One `ResultScreen`
reached two ways is the same field in the same object either way, so what that write sent away was the
high-score name entry a finished run had just arrived on: two frames from the run ending to the title
menu, with the name entry and the stats screen inside them and nobody having seen either. Over an
ending it took 240 frames off the staff roll instead, and wrote into a screen already freed. So the
write is made only where a ranking orb asked for is up, and the request is dropped where the run
finished.

**The record is put back between the two.** The added callback fills the captures out of the file it
read, which is what they were before the session counted anything, so orb puts back what it took
before asking the screen to leave. The ranking is the file's own and has to be: the write takes it out
of the table that read just filled — which is also why the record goes out through the game's own
screen rather than orb calling the write itself.

Whether a run wrote on its own is asked rather than assumed: every open for writing is noted, and a
run that ended at the result screen has one. Both modes build one. A legacy session stopping
partway keeping its record is the one place orb is deliberately not the game as it was.

**The captures in memory are emptied before a ranking is read.** 紅魔郷 keeps the record of which
spell cards have been captured in one place, `g_GameManager.cardHistory` at 0x69bcd0 — 64 records of
0x40 — and both reads that load it copy the records the file holds while leaving the rest as they
were: 0x42b466 has no clear of its own, unlike `clrd`'s parse at 0x42b502, which memsets four
records before it looks. That is how a run's captures survive the reload the game does at every
stage, and it is what the file is written from: the write at 0x42b9ed copies `catk` out of that
global, not out of what the screen read.

With one file that is consistent. With two it is how one file's captures reach the other: a session
that looks at one ranking has that file's records in memory, and leaving the other one's screen
writes them into the other file. So orb empties the block before the ranking screen's read — at
0x42f060, before the callback — and the read then defines the history rather than adding to it. Not
in the two states that screen is in on the way out of a run, 9 and 0x11 at `screen+0x8`, which are
the ones the game itself skips its parses in: there the record in memory is that run's own and the
file is about to be written from it. And not with `--no-chapters`, where there is one file and
carrying the captures in memory is right.

**Every open of the file goes in the log, with which of the two it landed in and what it was for**:
`opened in place of the game's own` or `opened as the game's own`, and `write`, `read`, or `read for
the front end's unlocks`. Both sides rather than the redirected ones alone, because a line that is
only written when the file was swapped makes the absence of a line carry the meaning — and an
absence is equally what a read that never happened looks like. The three reads cannot be told apart
past the one that is bracketed, so the line says which of them it is and no more.

**One read is not the mode's, and it is the one that is not a record.** 紅魔郷 keeps four things in
the one file: `hscr`, the ranking; `catk`, which spell cards have been captured; `clrd`, what has
been cleared; `pscr`, which stages may be practised. The ranking and the captures are the mode's
own — a card captured in a chapter that can be played again is not the capture the game's record is
a record of, for the same reason the score is not that score. What the front end *offers* is not a
record of anything, though: a stage that has been reached has been reached, and answering that
question out of a new file locks the game back to stage 1 for want of anything in it. So that one
read is left pointed at `score.dat` whatever the mode. The three reads are told apart by which
callback is running:

| read | what it is for | which file |
| --- | --- | --- |
| `MainMenu::AddedCallback`, 0x43a5c0 | `clrd` and `pscr` into `g_GameManager` at 0x69ccd0 and 0x69cd30, the only place the front end's `Extra Start` and practice stages are lit from | the game's own, always |
| `GameManager::AddedCallback`, 0x41bcdc, once per stage | the run's own record: the score to beat, and the spell cards it can add a capture to | the mode's |
| the ranking screen's added callback, 0x42f47f | the ranking on screen, and the record a score is entered into | the mode's |

The one write needs no telling apart: the exe reaches it from a single place, the ranking screen's
deleted callback on its way out, so a write while pointdevice mode is on is that screen's — a score
entered into it, or a ranking that was only looked at and written back as it was read, which is
what brings `pointdevice_score.dat` into existence before any run has finished. It carries `clrd`
along with the rest, because the game writes the file whole — so a pointdevice clear is recorded in
orb's file and **unlocks nothing**, the front end being lit from `score.dat`. That is the shape and not
a gap in it: what the front end offers is what runs somebody could have played have earned, and a run
orb could rewind is not one of those. Making a clear in the mode count would mean the union of the two
files' `clrd`, and with it orb knowing a format and an encryption it stays out of.

**orb's file is a new one, not a copy of the game's.** Nothing seeds it: until a pointdevice run
has entered a score there is no `pointdevice_score.dat` at all, the open of one that is not there
fails, and the game takes that the way it takes a first launch. A copy would put the game's whole
record at the top of a ranking none of it belongs to — what a `score.dat` says has been cleared
was cleared by runs nobody could rewind, which is the one thing keeping these two files apart.
Where a copy was what kept a pointdevice session's unlocks, the reads above are: the file being new
costs a ranking its history and nothing else.

**At the exe's import of `CreateFileA`**, not at the game's own score code, because that
import is where both of the game's own paths to the file end up. `score.dat` is one string, at
0x46af94, pushed at four places: three reads through the helper at 0x42b0d9, which decrypts
what it read (from 0x41bcdc, 0x42f47f and 0x43a5c0), and one write through
`FileSystem::WriteDataToFile` at 0x41e460, which encrypts before it writes (from 0x42bc15).
Both of those open with the CRT's `fopen` at 0x45ca0b — `"rb"` and `"wb"` — and the CRT is
statically linked, so the open reaches the OS at the exe's own import, IAT slot 0x46a150. That
slot is called from exactly two places: the CRT's open at 0x4677fa, and a file class at
0x43ceea that the score file never goes through.

So the redirect itself knows nothing about the game: no offset, and nothing about the format or the
encryption. It is the seam `memtrack` hooks for the heap calls, and d3d8's and dsound's own
opens go through their own imports and are not in the path.

**One address is needed, and only to tell the front end's read from the others**:
`MainMenu::AddedCallback` at 0x43a464, hooked so that the game's own file answers the read made
while it runs and the mode's file answers the rest. It is registered at 0x43a3c4 as the `+0x8` of
the chain element at `menu+0x8234`, and it is the callback that starts the title theme:
`bgm/th06_01.mid` at 0x46c3a4, pushed at 0x43a475, which is what identifies it. The parses identify
each read: `hscr` at 0x42b280, `catk` at 0x42b466, `clrd` at 0x42b502, `pscr` at 0x42b65e; this
callback calls only the last two, and the ranking screen picks `hscr` into a table of five
difficulties by four shots at screen+0x3ab0. Nothing but which file an open lands in rides on that
address, and a game orb has no such address for lights its front end from whichever file the mode
points at — which for a mode whose file is new is nothing at all, and is the reason this is one
address rather than none.

**Only the open is redirected, which is enough here and would not be everywhere.** That other
file class truncates a `"w"` by calling `DeleteFileA` on the name before creating it — at
0x43cea9, its only call site in the exe — so a game whose score write went that way would have
its own file deleted while orb's was written. 紅魔郷's does not go that way.

The whole file name is compared, ignoring case, and the directory the game named is kept. So
`pointdevice_score.dat` is not itself taken for the game's file and forked again — a second pass
over it would open `pointdevice_pointdevice_score.dat` — and a relative name resolves where the
game's own open would have resolved it. The paths are handled as the bytes the game gave and
handed to the game's own `CreateFileA` as those bytes: a directory name in the game's code page is
not necessarily UTF-8, so nothing in the path converts one. They are converted to text in one
place, the log.

With `--no-chapters` the hook is not installed at all: nothing can rewind that run, and its score
belongs in the game's own file.

**With `--clear` nothing is written**, whichever mode the run is in — there the hook is in the
path for the refusing rather than for the forking, and the file that must not be written is
whichever this run would have written. A cheated clear is not a score: orb's file is where
runs that cannot be compared with the game's are kept, and a clear nobody could have played at
the top of *that* ranking is the same mistake one file further on. The refusal is decided before
the fork is, so it covers both files. The open for writing is
refused rather than the write being sent elsewhere, which the game takes cleanly:
`WriteDataToFile` checks its `fopen`, returns -1, and its one caller — the `call` at 0x42bc1a —
drops that and frees the buffer either way. Reads go through, so the ranking screen and what the
file has unlocked are what they were.

## A latent crash in the game

`ResultScreen` passes `g_CharacterList[charUsed * 2]` as a *format string* to
`DrawStringFormat2`. `g_CharacterList` is `char*[6]` at 0x4784d8; `charUsed = 5` reads
0x478500, which holds the float 2.5 — the address `0x40200000`. `charUsed` is the shared
result-screen cursor, and erratic input can leave it outside the 0..2 this screen assumes.
Not orb's bug, but a reason orb must never feed the game stray input.

## How it is installed

`orb.exe` is the only file. It carries `orb.dll` inside itself — cargo's artifact
dependencies (`-Z bindeps`, which is why the toolchain is nightly) build the cdylib first and
hand over its path — and unpacks it to `%TEMP%\orb` before injecting, since `LoadLibrary`
needs a path.

The unpacked file is named for its own checksum rather than reused, because a mapped image
cannot be replaced while it is loaded. Stale copies are removed by the next launch that finds
them unloaded.

`orb.yaml` and `orb.log` are therefore found in the directory of the exe this is running
inside — the game's, which is where the launcher is installed too — and not relative to the
DLL.

Where the game and the DLL are is the launcher's own question, and `--game-dir=PATH` and
`--orb-dll=PATH` are how it is answered when they are not where they normally are. The DLL is
inside the game, so its answer is where it is; neither is handed on to it, and neither is a key
in `orb.yaml`, which therefore holds no paths and nothing belonging to one machine.

`--game-dir` therefore names the `orb.yaml` as well, since that directory is where the DLL will
read it from. The launcher reading the one beside itself instead put every answer the settings
dialog wrote into a file the game never opens — a size and an ending answered, and the game
starting on the defaults.

One DLL covers every game it is taught, and is not split per game the way vpatch's is. vpatch
patches per-game code; orb's per-game part is a `Game` implementation, which is a table of
addresses and a handful of accessors. Splitting would copy the snapshot, chapter, retry and
frame-loop code into every DLL and make the launcher carry several payloads.

## Configuration

**Three places, split by who sets them and when.** `orb.yaml` holds what somebody playing sets
and leaves set; a launch's arguments hold what is different every time it is run; and the mode a
run is in is asked inside the game, where the run is started — see *Pointdevice and normal*.

`orb.yaml`'s keys are `screen`, `language`, `thcrap`, `skip_ending`, `always_draw`,
`boundary_flash`, `hide_mouse`, `dpad_moves` and `ask_at_startup`. YAML read with serde; every one but
the first three is a switch written `true` or
`false`, `screen` is written `fullscreen` or a size like `1280x720`, `language` is written `japanese`,
`english` or `auto`, and `thcrap` is a switch that takes `auto` as well — the two whose answer the
machine can give. `deny_unknown_fields`, so a key nobody
reads is an error naming it — including one that used to be a key, which is a file to edit rather
than one to pass over quietly. A setting that is not read is a setting somebody thinks is on.

**The launcher asks for every one of them before it starts the game**, and writes back what it is told.
Which is why they are the keys they are: each is about the machine the game is being played on,
and somebody who has just installed one file has nothing to edit. It is orb's own window rather
than a dialog resource — a resource means a `.rc` and a resource compiler in the build, for a list of
controls this short — with the system's own message font asked for rather than a face named, since a face
named here is one that is missing on somebody's machine and the text may be Japanese.

**A real dialog, class `#32770`**, from a `DLGTEMPLATE` built in memory: a resource would mean a
`.rc` and a resource compiler in the build, and the controls are few enough to write out. It is as tall as what is
stacked in it rather than a height of its own, so a switch added to the list takes the dialog with
it instead of being drawn over the buttons. Being one rather than looking
like one is the point — a window manager decides whether to leave a window alone by asking what it
is, and the dialog class is the answer it looks for. A window of orb's own class with a dialog's
styles was tiled by a window manager that leaves real dialogs alone, which is how the difference
showed up.
Its measurements are therefore in dialog units, a quarter of the font's average character width
across and an eighth of its height down, so the whole of it scales with the font the system gives
it and nothing in it is in pixels — which is also why display scaling costs the dialog nothing. The
one measurement the language changes is the column the two labels are written in, `画面` being two
characters where `Language` is eight. The
dialog manager brings the rest: tab between the controls, return on `はじめる` / `Start`, escape on
`やめる` / `Quit`. The window sizes
offered are 16:9 and then 4:3 at the heights monitors have, biggest of each first and the game's
own 640x480 last, filtered to those that fit the primary monitor with a window frame's worth of
room to spare. 16:9 above because that is the ratio that leaves black for the status line, and
biggest first because on a large monitor the size wanted is the large one. The frame is allowed
for by a fixed 16x40 rather than measured, because the window being filtered for is the game's: it
does not exist yet, and it is created by the other half of orb inside a process that has not
started. A monitor is therefore never offered a window as tall as itself — that is what
fullscreen is for.

**The dialog answers to a pad**, which no dialog does by itself and which matters because the person
it is put in front of is about to play a game with one in their hands. `joyGetPosEx` on a thread of
its own — the same call the game makes, and slow for the same reason, the worse case being the one
where there is no pad to find, so a message loop cannot be made to wait for it — read again as soon
as each read finishes, because a press is only ever seen if a read lands while the button is down. A
cycle that waited between reads was longer than a quick tap and lost taps that fell between two of
them, which is what a pad that answers *sometimes* looks like; the launcher now also prints whether a
pad answered at all
and how many pushes it sent, since a pad that was never there and one that was pushed and ignored
want opposite things done about them. The thread posts what it sees to the dialog,
which turns it into what the dialog manager and the controls already answer to.

**What the pad does is a menu's, not a dialog's.** Up and down move a row at a time, and the two
buttons are one row — left and right choose between them, the way a menu with two answers on one
line works. Left and right otherwise change what the row holds more than one of, which is either of
the two lists — the sizes and the languages; a switch is on or off, so turning one over is the decide
button's job rather than a
sideways push, which does nothing there. The decide button does whatever the row it is on needs
doing to it: a list is opened, a switch is turned over, a button is pressed. While a list is open
it has the whole pad — up and down move inside it and either button closes it, a dropped list's
selection already being whatever it is showing. The row is worked out from whatever has the focus
rather than remembered, so the pad picks up wherever a mouse or the tab key left off, and the focus
is moved through the dialog with `WM_NEXTDLGCTL` so that its own idea of where it is goes along.

A stick and a d-pad both drive it:
a hat reports in `dwPOV` — hundredths of a degree clockwise from up, and 0xffff for pushed
nowhere — and not on the axes at all, so reading only the axes leaves a d-pad dead. Up is the *low*
side of the Y axis, that axis being measured downwards; getting it the other way round is a dialog
that moves the wrong way, which is what it did once. Nothing about the controls is aware
that a pad exists. Which button is which comes out of the first 18 bytes of the game's own
configuration file — its `ControllerMapping`, nine `i16` — so the dialog answers to the buttons the
game will, and falls back to the game's own defaults where there is no file to read.

**Display scaling is ignored on both sides.** `SetProcessDPIAware` before either process puts
anything on the screen: the launcher before it measures the monitor, and orb inside the game
before the window is created. Without it every size is a size Windows quietly multiplies — a
1280x720 window asked for on a monitor at 150% covers 1920x1080 of screen, and the monitor a
fullscreen window is measured against reads as two thirds of itself. What it costs is that the
settings window's own layout is then in real pixels and has to be scaled by hand, from
`GetDeviceCaps(LOGPIXELSY)` against the 96 the numbers are written in; the font does not, since
what the system hands back for its own windows is already the size it wants at that dpi.

`ask_at_startup` is the last of the questions. Answer no and the next launch starts the game
straight away; `--settings` asks whichever way it is set, which is the way back. Closing the
window rather than answering it starts no game and writes nothing.

**No file is every default**, which is what leaves the launcher the one file to install — and
what makes the default `ask_at_startup: true`, so a first launch asks and there is a file
afterwards. A file that `--config=PATH` names and does not find is an error even so, since a path
somebody typed is one they meant, and answering it with the defaults would leave them watching for
a setting nothing read.

**Nothing but the keys and their values is written.** What each of them is for is said by the dialog
that asked all of them, in the language the person answering reads — so a comment over each key would
be the one thing orb installs that is in one language whoever gets it may not have. A file somebody
has written comments into keeps them until the dialog writes that file back, which is the same moment
every value in it is replaced by what was just answered. It is written by hand rather than through a
serialiser even so: `screen` is a word or a size and `language` is a word or `auto`, and both are
written exactly as they are read back. There is no copy of the file in this repository to install: it
is written by the thing that asks, and a second hand-kept copy is one that goes stale.

Everything to do with building the midstage table, reaching an ending, or looking into a fault
is an argument to `orb.exe` instead — `--help` lists them — because a file is the wrong
place for something that is different every time it is run. The two passes over a replay are one
word each:

| | |
| --- | --- |
| `--collect` | propose boundaries over the whole replay, at 64 updates a frame, with nothing stopping and nobody at the keyboard |
| `--judge` | step between them at one update a frame and decide about each: the pass somebody watches |

and beside them `--tune`, `--replay`, `--speed=N`, `--log=quiet|normal|verbose`, `--compose=N`,
`--self-check`, `--stress=N`, `--sent-keys`, and `--no-pacing`, `--no-chapters`, `--no-memory`,
`--no-resume`, `--no-frame-loop` and `--no-hooks` for taking orb apart until a fault stops happening.

**`--sent-keys`** has the game read its keyboard the way it does when DirectInput gave it no device:
orb lets the device go — `Unacquire`, `Release`, the pointer cleared, which is what
`Supervisor::RegisterChain` itself does with one it cannot set up — and `Controller::GetInput` then
takes its `GetKeyboardState` branch. What that buys is a session another program can drive, which is
otherwise impossible: a device held `DISCL_EXCLUSIVE | DISCL_FOREGROUND` does not see keys injected
with `SendInput`, and every screen orb has a question over is the game's own screen answered on the
game's own keyboard. Nothing else about a run changes — the same code turns those keys into the same
buttons — and what it made possible is the whole of picking a run up again, watched twice through the
front end without a hand on the keyboard. `orb-e2e`'s `keys_from_another_program` drives both sides of
it: the game holding the device and refusing a sent key, and the game reading `GetKeyboardState` once orb
has let the device go and being driven by one.

`--game-dir=PATH`, `--orb-dll=PATH`, `--config=PATH` and `--settings` are the launcher's own,
and the only ones it does not hand on: each answers a question the DLL, being inside a game
that has already started, never has to ask. Keeping them here is also what keeps a path with a
space in it off the line the DLL reads.

**`--no-frame-loop`** leaves the frame to the game: its own order, draw before update, and its
own pacing, with the update and the draw still hooked so chapters carry on. The frame of input
lag comes back with it and frames are doubled and dropped again, which is why it is asked for at
a launch and not somewhere it could be left set.

One clap definition for all of them, read by both halves: the launcher out of its own
arguments, the DLL out of the command line the launcher wrote. A value goes onto its option,
`--speed=64` rather than `--speed 64`, and clap is told to require the `=` — the DLL picks the
options out of that whole command line by the two dashes they begin with, since everything
before them is a path that may hold anything, so a value standing in a word of its own is a
line the two halves would read differently.

**The pacing is written unless a launch says otherwise**, and at whatever `--log` says rather than
as a tier of it — see *What a late frame says*. Its own switch because it answers a different
question from how the run is going, and on by default because a run whose frames came out unevenly
has the account of it without having been launched by somebody who expected that. `--log=quiet`
leaves nothing in the file but the startup lines and these, which is what a sweep is read off.

**`--no-pacing`** stops it, and with it the one question asked once a frame that costs a call to
answer: whether the anchor is a blank. That call is made after the flush and the lines are written
there too, so what it takes back is the call and the lines rather than any of a frame's own time.

**`--compose=N`** pins the time left for the compositor to draw in, which is otherwise found
while running — see *The compositor's drawing time*. For sweeping it, `N` being what the sweep
steps in.

**`--clear`** is the one that is about neither: nothing can hit the player and the run goes at
`--speed`'s 64 updates a drawn frame, so half an hour of playing well is a minute of holding the
shot key. It is there because the ending is reached by clearing the game and by nothing else —
see *The ending* — and a boundary inside one cannot be looked at twice at half an hour a look.
What it writes, before each update, is the player's state — to the one the game's own bombs
write — and the frames of invulnerability left under it. Both, because the state on its own does
not last: `Player::OnUpdate` is chain priority 7 and the bullets are checked at 11, so a state
whose frames have run out is a state put back to normal before the hit test in the same update,
and the player dies with it written. Only from the states the player can be hit in or is already
invulnerable in, so a spawn or a death still plays out as itself.
It leaves no record: no score file is written at all — see *The score file* — because such a run
has no score to keep in either ranking, and no replay either. `--clear` is the one thing that
still refuses the write of one, rather than not being offered it: a cleared run does reach the
screen that offers to save a replay, since that screen is only kept from a pointdevice run and
this run is not one. A replay holds the inputs and nothing about the player having been
unhittable, so playing one back is a run that dies where this one did not.

Nor is the chapter it is in written down — see *Picking a run up again* — for the same reason and
with the same consequence: what is kept is the buttons, so a later launch offered that chapter
would play them into a player who can be hit, and land somewhere else. A clear takes the third
record out along with the other two, and is never asked the question either.

The launcher reads them, refuses to start on anything it cannot, and hands them on the game's
own command line — which the game never looks at, `lpCmdLine` appearing once in the whole of its
`WinMain` as the parameter it ignores. orb reads them back off that inside the game, so the two
sides cannot disagree and nothing has to be written down between them.

**The keys are fixed in the code**: `a` adds a boundary, `d` writes the files, the arrow keys
step and judge, `space` holds, `shift` and `ctrl` change what stepping means. Whoever is
building a table is the only person who presses any of them, and a setting nobody changes is one
more thing that can be wrong.

`orb.log` is appended to rather than started over, because a run worth looking at is usually
over before anyone looks. A crash adds a line naming the faulting module and offset.

## The language orb writes its screens in

**`language` decides every word orb puts on the screen, and `auto` — what a file with nothing in it
says — is the language Windows itself is in.** `GetUserDefaultUILanguage` answers that, and orb reads
the primary language out of the LANGID it gives: `LANG_JAPANESE` is Japanese and every other language
is English. Which is the honest answer rather than a fallback — a machine in German is one orb has no
German for, and whoever is playing 紅魔郷 on it is reading either the game's own Japanese or an English
patch over it. The user locale is *not* what is read: that one carries the regional formats, which are
set apart from the language Windows shows its own windows in.

**The answer is settled once, at the attach, and said in the log** — `language: english, which is what
this machine's own windows are in` — because a screenshot of a menu somebody could not read is
otherwise the only evidence of which language it came out as.

Every screen orb puts up is in it: the question over the title menu with both modes and the lines
under them, the menu where a chapter was lost with the two questions its items ask, the question about
a run left unfinished with the mark that goes up before it, and the launcher's settings dialog. What
each screen says in either language is beside the screen itself — `mode.rs`, `retry_ui.rs`,
`resume_ui.rs`, `launcher/settings.rs` — as a `match` over the language, so a screen added without a
wording in both is a build that stops rather than a line nobody can read. 完全無欠モード and
レガシーモード are 紺珠伝's own names for the two modes, and `Pointdevice Mode` and `Legacy Mode` are that
game's own names for them in English.

**The log is not one of the screens** and stays English whichever language is chosen: it is read beside
a source tree written in English, and a file that changed language with the machine it was written on
is one no two reports of a defect could be compared across. So does every line the launcher prints,
for the same reason and to the same reader.

**A refusal says the same thing twice, then.** The line is that English, and the dialog put up where
nothing is going to read a line is in the machine's language and in sentences — a printed line being
terse where a dialog holds a paragraph. Two of them are refusals somebody playing meets: no game where
orb was pointed, and an exe that is no build orb knows. The timer is the third and is refused before any
file has been read, so its dialog is in the machine's own language whatever a file would have said. A
command line orb cannot read is the fourth, and what clap wrote about it is kept rather than replaced:
the option it suggests instead is the useful half of it. What is left — a file that cannot be read, a
process that will not start — says whatever Windows said, in a dialog that gets the line as it stands.

**And every one of them goes into `orb.log` as well, before the dialog goes up.** Which is the only
one of the three that is still there afterwards: the line needs a console, and a launch from a shortcut
has none — `--pacing` for `--no-pacing` ended a launch with nothing on the screen and nothing written
down. So the log holds `launch refused:` and that line, beside `orb.exe` where a run's own log is, and
the launcher writes nothing else to it. Before the dialog because that call does not return until
somebody answers it, and a dialog left standing and then killed would otherwise leave nothing at all.

**The settings dialog is in whatever the file said when it was built**, and the row that changes the
language does not change the dialog's own words: every label is in the template the dialog manager has
already read, and rebuilding the window under the hand answering it would be a dialog that vanishes
mid-answer. What the row asks for is what the game and the next launch are in. The two languages are
named in themselves — `日本語` and `English` — the way every language picker names them; only the
machine's own is a word the dialog translates, `自動` or `Automatic`.

**Which language the *game* is in is not orb's**, and orb does not touch it: 紅魔郷 draws its own text
from its own data files, and an English one is a patch over the game — see *Which game a launch is* for
what orb accepts as the exe.

How the answer reaches each screen, and what was weighed against threading it there, is
[docs/adr/0014](docs/adr/0014-a-language-is-threaded-into-every-screen-and-the-machines-comes-through-the-seam.md).

## The translation patch beside the game

**The game's own words are thcrap's, and `orb.exe` is the only file started.** Somebody who has
installed both otherwise has two launchers for one game and has to pick: through thcrap and there is no
orb in the process, through orb and there is no translation. So a launch looks in the game's directory
for a thcrap, brings its patch stack up to date, and hands it the game orb has just created — the same
launch, whichever of the two the player thinks of as the launcher. `thcrap: true` and `false` are how
somebody says which they want whatever their machine is.

**And `auto` — what a file with nothing in it says — is answered by the language orb writes its own
screens in: off where that is Japanese and on where it is anything else.** Japanese is somebody who reads
the words the game already has, and putting an English patch over them would be orb translating a game
away from the language its owner reads; anything else is somebody who cannot read those words, and a
thcrap installed beside the game is what they installed it for. It is that answer and not
`GetUserDefaultUILanguage` directly, so `language` decides this too where it names a language: somebody
who says which language they read has said it about the patch as well. Neither answer is a guess that
stands: the settings dialog shows it as the box it starts from, and whatever comes back from it is
written down by name, so one launch through that dialog settles it for good.

**Where thcrap is comes out of the launcher its own configuration tool wrote.** That tool leaves
`th06 (en).exe` in the game's directory, whose string resources 0, 1 and 2 are the `bin` it runs from,
the command line it runs, and the exe it runs — and resource 2 naming `thcrap_loader.exe` is what says a
file is one of those rather than the game's own exe or orb's. So the patch stack, the run configuration
and the game id are read out of what thcrap wrote about itself, and orb reproduces no step of thcrap's
own installation. Every exe in the directory is asked, in the order the names sort, since which patch
stack it is is in the name and orb has no more idea than the player which of two is meant.

**Two of thcrap's own exports do the injection, in the order its own loader calls them.** A process
created suspended has not had the loader initialisation Windows does before an entry point, and thcrap
put into one in that state leaves the game standing — measured, a game with thcrap's DLLs in it and no
window. `WaitUntilEntryPoint` is thcrap's answer, exported for this: it runs the process to its entry
point, which is after Windows' initialisation and before any of the game's code, and stops it there.
`thcrap_inject_into_running` then takes the process and the run configuration and does the rest itself.
Both are cdecl — `THCRAP_API` names no convention — and called as stdcall instead the stack comes back
short and orb dies with the game left at its entry point, which is what it did.

**orb's own DLL goes in second.** The two rewrite some of the same import entries, `CreateWindowExA`
among them — orb's to size the window, thcrap's to read its title as UTF-8 — and whichever is second
owns the entry. orb's rewrite calls through to what was there and thcrap's does not, so orb second
leaves both working and orb first leaves the window the size the game asked for with orb's letterbox
holding no client. Measured, as a launch that came up in the game's own 640x480 with `screen:
fullscreen` in the file.

**The patch stack is brought up to date the way thcrap's loader does it**: `runconfig_thcrap_dir_set`
with the installation, then the run configuration, then `stack_update_wrapper` twice — the
game-independent files under thcrap's global filter and this game's under its games filter, which is
what keeps a machine with one game from fetching the translations of thirty. thcrap's own
`update_at_exit` decides when: before the game starts unless that box is ticked, in which case the pass
after the game is running is the one that fetches, and the next launch is the one that plays with the
new files. `thcrap_update.dll` is loaded by orb first, with the same altered search path, because the
wrapper in `thcrap.dll` loads it *by name* — from orb's directory, where it is not — and answers with a
fallback that updates nothing when it cannot: measured, a launch that fetched none of two files deleted
by hand.

**Nothing is passed where thcrap's progress callback goes**, and a launch says only that the update ran.
Giving it one corrupted the process's heap — `0xc0000374` first, then the same access violation 477
times and a stack overflow to finish, on two runs out of two with files to fetch — where the same two
calls with nothing there fetched the same thirty files and came back.

**A failure anywhere in this does not stop the launch.** The game is orb's business and the words in it
are thcrap's, so a patch that cannot be found, loaded or injected is a game somebody plays in Japanese,
which is the game they had before installing either. What each step did is in the line the launcher
prints.

What is supported is a thcrap whose configuration tool left one of its launchers in the game's directory,
and *Not supported* is where the rest of its wizard's choices are. Why the whole of this is read out of
that one file, and what each step of it was measured against, is
[docs/adr/0016](docs/adr/0016-the-game-is-handed-to-a-thcrap-installed-beside-it.md).

## Building the midstage table

Boss boundaries need no table; a stage's waves are a script on a clock, so those boundaries are
frame numbers someone has to pick. Under `--tune`, boundaries are proposed as the
stage is played, a second into each gap between waves: nothing left to shoot at, no boss fight
to interrupt, and two seconds at least into the chapter now running. One per gap, on the frame
the gap becomes one, so a long lull does not fill up with them.

Bullets in the air are not part of the test. The game was not written around chapters, so a
frame with none of them is rare — asking for one found three places in a stage where a retry
unit wants a dozen — and a snapshot restores whatever is in the air exactly, so a boundary does
not need the screen to be clear. What it needs is a gap in the script, and enemies are what say
that.

**One table for every difficulty**, because the clock it is written in does not move with the
play. The enemy timeline runs through the midstage and through a midboss fight, and stands
still only while the stage's own boss is fought: on stage 1 the chapters at script 2009, 2666
and 3180 track the stage's own clock exactly across the midboss, while every chapter of the
boss fight reads 5282. So a frame number always names the same point of the script, however
fast the midboss was killed and whatever a difficulty adds to it.

What differs is what is on screen at that point: a midboss still alive because this run was
slow, the leftovers of a spellcard a harder difficulty adds. A boundary worth keeping
therefore sits in a gap the script leaves rather than one a fast clear happened to open, and
the `tuning: gap of N frames` line is what tells them apart — a few hundred frames is the
script's, a few dozen is that run's. It is also why the difficulty to tune on is the hardest:
a gap on Lunatic is a gap on every difficulty below it, and the reverse does not hold.

What the pass has proposed is the list chapters are then counted from, exactly as the
compiled-in table is, so a stage played twice divides the same way both times. A boundary is
proposed only on ground the pass has not covered yet: replaying part of a stage neither writes
one down twice nor brings back one that was removed by hand.

**Two files, and what is decided is kept.** `chapters.rs` is the table as Rust, to paste over
`crates/orb-core/src/game/th06/chapters.rs`; `tuning.txt` beside it is the same boundaries with what
has been decided about each — `keep`, `adjust` or `drop`, and whether a person put it there —
and is read back when orb starts. So a stage is not one sitting's work: it can be looked at,
judged in part, left, and picked up where it was. Both are written whole as soon as anything is
decided, as well as at every stage's end and on `tuning_write_key` — looking at a few boundaries
and then closing the game is how a sitting ends, and nothing about it should have to be
remembered. So edit `tuning.txt` only while nothing is running.

Entering a stage keeps everything already known about it and forgets only how far the detector
had got, which is about the pass rather than about the stage. A stage neither visited nor read
back keeps whatever is compiled in, so stages can still be done one at a time: tune a stage,
paste the table in, rebuild, and go on to the next.

`--replay` puts a replay in charge of the playing, and the two passes over one are a word each,
each of which is `--tune --replay` and one more decision:

- **`--collect`**, which nobody watches: the replay runs to the end of the run, nothing stops,
  and every boundary is proposed and written to `chapters.rs` beside the launcher as each stage
  ends. The frames still come at the display's cadence and each carries 64 updates, so twenty
  minutes of run go by in twenty seconds — with one frame in 64 drawn, which is nothing to judge
  by eye.
- **`--judge`**, which somebody does: one update a frame, and stepping between the boundaries.
  What says whether one is in the right place is the frames around it in motion, since a still
  frame shows no bullets on screen and that is the exact thing the detector already tested.
  Stepping is instant whatever the speed, so above 1 buys nothing here.

The keys, read while a run is being tracked, which under `--replay` includes a replay. They are
fixed in the code rather than settings, since whoever is building a table is the only person who
presses them. The stepping keys are a replay's alone — holding a run someone is playing
still is what the retry menu is for:

| | |
| --- | --- |
| `right` | runs updates with nothing drawn until the chapter changes, and holds the game there — the ending skip's mechanism, aimed at a boundary instead of the end of a scene |
| `left` | restores the stage's start, which rewinds the replay along with everything else, and runs forward to the chapter before this frame. A stage is a few thousand updates and an update is tens of microseconds, so it costs a visible pause |
| `space` | holds the game where it is, or lets it carry on. Anywhere, not only on a boundary |
| `shift`, held | held with next or back to move between stages, which the game does by starting the replay at one |
| `ctrl`, held | held with next or back to move between the boundaries judged out of the table, which the stepping otherwise passes over. The way one is reached, and so the way a `Rejected` is taken back |
| `up`, `down` | judge the boundary the chapter being looked at began at, one step per press: `Rejected` → `Adjust` → `Keep` and back. Neither end wraps, so pressing on past `Keep` cannot throw a boundary away |
| `a` | puts a boundary at the frame the game is on, for a gap the detector misses, and begins its chapter there. Judged like any other afterwards, so it can be taken out again |
| `d` | writes both files, which each stage's end does anyway |

**Neither key leaves the stage, and neither does playing on.** What lies outside a stage is
the game's to load rather than a snapshot's to put back, so a step stops at the stage's own
start going back, and the replay is held at the stage's end however it got there. That end is
the game leaving the gameplay scene to build the next stage — not the boss going down, which
is well before the stage is done with itself.

Moving between stages is asked of the game instead, with `chapter_across_key` held: it starts
the replay at another of its stages, the way its own menu does. `GameManager::RegisterChain`
raises `currentStage` by one and `ReplayManager::RegisterChain` reads that stage's record —
the seed, the rank, the lives, the bombs, the power, the score so far — so writing the stage
before the one meant and `GAMEMANAGER_REINIT` into `curState` is the whole of it. The
supervisor cuts this stage's chain and registers the next, and nothing of orb's has to be kept
whole. A stage the replay does not cover is refused rather than asked for, since the game's
own answer to that is to drop to its main menu.

The run's score goes back to nothing on the way, along with the count of extra lives it has
already paid for, because those are the one thing the game leaves to whichever path it took in.
`GameManager::RegisterChain` clears them only when it is starting a run rather than
reinitialising, and a stage move reinitialises; the replay then puts the score back from the
record of the stage before the one being started, which the first stage does not have. Left
alone, stage 1 begins with the score the run was left on, crosses the first extra life's
10,000,000 part way through, and gains a life the recording never had — and with it the rank the
extra life carries, which the enemies read. The count of extra lives goes back to none rather
than to what the score says, since the stage's own loop only ever raises it: from none it can
reach what the restored score has paid for, and from a later stage's count it cannot come down.

One thing has to be held back for that to be repeatable. Tearing a stage down ends the run's
recording — `GameManager::DeletedCallback` calls `ReplayManager::StopRecording`, which closes
the record of inputs off with a blank entry and a frame number no run reaches — and the game
does it whether the run was recorded or is a replay being watched. Played back, the record it
writes into is the replay's own, at the entry playback has reached, so leaving a stage part way
terminates that stage where it was left: play it again and the player takes no input from that
frame on, standing still until it is hit. So while a replay is being played back that function
does nothing, which costs a recording nothing — the game's own replay writing calls it only for
a run it recorded.

**Every chapter boundary, not only the midstage ones.** A boss's attacks are chapters too, and a
step that skipped them would put a fight out of reach. They are in no table — they are found as
the fight runs — and the script clock the table is written in stands still during a fight, so
what the stepping moves between is the stage frame each chapter of this stage has begun at.

**A boundary the fight has is taken wherever it falls.** Dying and bombing are both seconds
long and a fight ends or moves on under them, so neither holds a boundary back: a chapter that
begins mid-death kills whoever restores it and one that begins mid-bomb hands them a cleared
screen, and both are better than a boundary that is not where the attack began. The way out of a
start like that is the chapter before it. What is held back is dialogue and a pause, where the
run is not getting anywhere at all — and held rather than dropped, so what the fight did under
one is taken on the first frame that can carry a chapter, which for the dialogue a boss arrives
with is the frame the fight starts on.

**A fight underway outranks the table.** The enemy timeline runs on through a midboss, so a fight
that drags reaches the frames the waves after it are divided at, and a chapter beginning half way
through a fight is a retry point for neither the fight nor the waves. The entry is spent where it
falls rather than held back: the fight's own end is the boundary those waves want, and an entry
held over it would fire a frame later as that boundary's double. The same frame in a run that
killed the midboss quickly is past the fight and is a boundary — that is the clock's doing, and
the table cannot say which run it is in.

Both keys are relative to the frame the game is on rather than to the chapter it is in, and
strictly so: a boundary a frame or two behind is what back reaches, and forward from there moves
on rather than staying put. Neither stops at a boundary judged out of the table: it begins no
chapter, so stopping there would be stopping at nothing. It is judged out and not gone, though,
so `chapter_dropped_key` with either key goes between exactly those — in the frames of the script
clock, since nothing has a stage frame for a boundary no chapter ever began at — and that is how
a `Rejected` is reached and taken back.

**Why the chapter changed is on the status line, not what the chapter is.** A pass is deciding
whether a boundary belongs in the table, and what answers that is which signal produced it —
which is also what says where to look when one turns up where none belongs. So the line under
`CH 05` is one of:

| | |
| --- | --- |
| `STAGE`, `STAGE_AFTER_MIDBOSS`, `MIDBOSS_NONSPELL`, `MIDBOSS_SPELL`, `BOSS_NONSPELL`, `BOSS_SPELL` | the game's own, settled as the run goes. In no table, and nothing to judge |
| `AUTO 1886 KEEP` | the table's, as the detector proposed it, with what has been decided about it |
| `HAND 1886 KEEP` | the table's, put there by hand — the number nothing would find again |

Told apart that finely because which fight it is and whether the attack has a name is what says
where in a stage the game is standing. A boss arriving is its first attack starting, so there is
nothing else to call that; and what follows a midboss is the stage carrying on, so it is named
for that rather than for the defeat behind it. `HOLD` says only that the game's update is being
held, which is a different question and is on its own line.

A run shows the chapter's name in place of both lines — see *The status line* — because there
nothing is being judged and where dying will send the player is the whole of what the line is
for.

**Which fight it is comes from the music**, because that is where the game says it: a stage's data
names two songs, its own and its boss's, and the second is played for the fight the stage ends
with and for nothing else. The answer is watched for as long as the fight lasts and only ever
raised — it starts at the fight a stage runs on through, and the first frame the stage's own track
is gone it becomes the other for good. Both ends need that: the track changes when the fight
begins, which is not always the frame the boss arrives on, and by the frame the boss is beaten the
sound may be being taken down again. The defeat is where the answer matters most, a midboss going
down being a boundary and the stage's own boss going down being the stage ending.

**The judging keys work only with the game held on a boundary.** A frame is a sixtieth of a
second, so a key pressed while the game runs lands on whichever frame it reached, which is no way
to aim at one; and a chapter judged from anywhere inside it is a boundary edited from where it
cannot be seen. So what a key acts on is always the frame on screen, and the status line is
always naming it.

A boundary is `Keep` from the moment it is proposed. `Adjust` keeps it and writes it out marked
— `proposed(1886) /* adjust */` — for a gap that is real where the frame is not quite right.
`Rejected` keeps it out of the table while remembering it, because a decision outranks the
detector: the same stage played again would otherwise propose it back, and taking a rejection
back means finding it again.

Whose hand a boundary came from is written out as the entry itself — `hand(1886)` against
`proposed(1886)` — since that is the number nothing would propose again if it were lost, and
refusing one put there by hand takes it out altogether rather than remembering it as refused.
There is nothing for a refusal to hold back there, and `a` is the way back.

**The shortest a chapter may be does not apply to one put there by hand.** That floor is there to
stop a boss's opening flurry of script transitions carving out chapters a fraction of a second
long, and a hand is not that: stage 5's 2363 was added 54 frames after the boundary at 2309, and
being dropped on every pass but the one it was added on would lose what somebody wrote down while
leaving it in the table to look at. Which is why `hand` is an entry of the table and not a comment
on a number: the exemption is read in play, where a comment could not be, and a table that only
remarked on it divided a stage differently from the pass that chose it.

**A boundary and the start of a chapter are one thing**, and adding one by hand begins its
chapter on its own frame. Crossing a boundary is otherwise noticed on the update where the script
clock has reached it, and for a frame the game is already standing on that is the update after —
so the chapter would begin one frame past the number written down, while the same table read on a
later pass began it on the number itself. That frame is what the status line shows, what the
flash goes off on and what a step lands on: three places for one thing to be, and the one on
screen the wrong one.

Beginning it there also keeps the detector from offering one of its own a frame or two later.
Both are reading the same lull — a hand goes down when the screen empties, the detector a second
into it — so a boundary added just before the detector would have spoken leaves two a frame
apart, and the shortest a chapter may be is the rule that already says no to that.

The frame a boundary falls on is one frame — a sixtieth of a second under `--judge`, and
one of the ones that go undrawn above it — so a step holds the game on it. What is held back is
the game's update; the drawing carries on, so that frame stays on screen and the status line's
`SCRIPT` is the number that would be written down.

**A judging pass washes brighter and longer than a run does**, and whichever way
`boundary_flash` is left: the setting is about a run somebody is playing, and a pass with no
wash has nothing to say a boundary has been reached by. A boundary is one frame among a stage's
thousands, and nothing is being dodged on the frame underneath, so it can take the field for the
sixth of a second it has — one frame in twenty is all that frame gets — and still be gone before
the next boundary. It marks one the game runs past under `space` as well as one a step stops on.
The wash itself is described under *Chapters and retries*.

Every gap long enough to have been a candidate goes to the log at `verbose`, taken or not, with
its length and how far into the chapter it fell. What `ENEMY_GAP_FRAMES` should be is a question
about this game's waves, and that line is how a pass answers it.

## Checking the snapshot engine

`--self-check` restores every snapshot immediately after taking it and compares the result, and
reports memory that changed since a snapshot without being covered by it. The one moment a
restore can be held against what it should have produced is the instant it was taken from, which
is why it happens there. It fingerprints every private page in the process, so it pauses the game
for as long as that takes.

Nothing saves and restores by hand any more. What the two keys that did are for is what chapter
retries, `left` between boundaries and `--stress` do already, and with the game's own bookkeeping
around them rather than without it.

`--stress=N` restores the current chapter every N frames, a few times per chapter
and then moving on, so a replay walks through the midstage, the midboss and every boss attack
restoring as it goes.

## The seam between orb and its host

Some of what orb gets from the host it runs on goes through `orb-api`. Each area of it — the game's
memory, the clock, which keys are down, where the mouse pointer is and whether it is drawn, which thread
is running, the log file, the modules the process
has loaded — is a facade of
free functions with two answers behind it: the real one, under `#[cfg(windows)]`, and whatever
`Win` implementation a test has installed.

Mostly Win32 calls, but that is not what decides membership and one member is not a call at all: the
`pause` that finishes a spin is an instruction, and it is here because how long a spin takes is a thing
the host does to orb and a thing no test could otherwise decide. See
[docs/adr/0007](docs/adr/0007-the-spins-pause-is-behind-the-seam.md).

What is behind the seam is what a test could not otherwise get at. The game's memory is behind it so
that `Th06` can be read with no game running. The log's deferral turns on which thread is asking and
its lines are stamped with the host's clock, so a test that cannot be two threads or say what time it
is cannot reach the mechanism at all. The display and its compositor are behind it because the pacing
reads two numbers they answer — the compositor's own spacing, which the cadence is counted in, and the
monitor's rate, which says what the desktop is like — and the case that matters is the two disagreeing,
which otherwise wants two monitors of different rates and a window on one of them. And the keyboard is behind it because orb's own questions read it
themselves, the game being frozen on the frames they are up on: which mode a run is, and so whether it
has chapters at all, is decided by keys nobody could press in a test. The mouse is behind it for both
halves of the same reason: a hand moving one is what orb puts the pointer back for, and whether the
pointer is drawn at all is a counter the host keeps for the process — so a test that cannot move a mouse
and cannot read that counter back cannot reach any of it. The modal and the `ExitProcess`
that turn away a host orb cannot pace on are behind it for the same reason at its plainest: an e2e test
that raised a real `MessageBoxW` would wait for a click that is never coming, and one that really
exited would take the harness's child with it.

A Win32 call with nothing like that behind it stays where it is: orb ships for Windows and only for
Windows, so being able to build without a call buys nothing on its own — what buys something is being
able to *decide* what the call answers.

Two things the simulated host is deliberately as unhelpful about as the real one, because a kinder
simulation would let a test come to rely on something false: `cRefresh` counts compositions rather
than refreshes of the display, and `cFramesLate` reads zero through a broken cadence as much as an
even one. And one thing it must model that looks like an implementation detail and is not: **spinning
costs time.** `wait_until` spins the last stretch to its deadline, so a clock that moved only on a wait
would never arrive. Both halves of that loop are therefore behind the seam and both are charged for — the
counter read at a tick, and the `pause` at `PAUSE_TICKS`, which is the one member of the seam that is not
a call into Windows at all. What that number is and why it is not a faithful `pause` is
[docs/adr/0007](docs/adr/0007-the-spins-pause-is-behind-the-seam.md).

**The simulated host is not a metronome, and that is deliberate.** Windows is not one from an
application's side: it wakes a thread when it gets round to it, and its compositor now and then takes
far longer over a frame than it usually does. Both are modelled, from measured distributions rather
than chosen ranges — the blanks are an exact grid and it is the *return* of a flush that is delayed,
because that is the shape the measurement has. The delays come from a seeded stream, each e2e test runs
over several seeds, and the seed goes in every assertion so a failure replays exactly. An e2e test that
holds for one seed and not another has found something a real machine can do, which makes it a defect
rather than a flake.

So what the pacing e2e tests assert is a rate and not a schedule: **what share of the seconds ran at
sixty frames a second, within half a frame, once a few seconds of grace have passed.** That is the
question somebody playing has — the music and every timer in the game are counted in its own frames, so
a second at the wrong rate is a second of the game at the wrong speed however the average over the run
reads. Every display the pacing accepts holds every second of the run, a desktop whose compositor is
timing another monitor's rate included — nine such rates against a 120Hz monitor, four hosts apiece.

The call sites keep their shape: `mem::read(address)` is what it was before the seam went in.
Making every caller carry a `&dyn Win` instead would have rewritten two thousand lines of structure
walking to say nothing new. What a test installs is a thread-local, because the harness runs tests
side by side in one process and a simulated Windows in a static would be two tests writing each
other's game; it comes off again when the installation is dropped, since the harness hands its
threads out to whatever runs there next.

`orb-sim` is the other implementation, and every test in its own `tests/` drives the real `orb` and
`orb-core` against it — orb's code, composed as the DLL composes it, with the host answered by hand.
They live there rather than in either of those crates because a crate compiled as a *dependency* of a test
binary has `cfg(test)` false, so the install point is reached through a cargo feature instead, and a crate
cannot turn a feature on for itself. Which crate's `tests/` is then free, and it is the simulator's because
that is where the thing every one of them installs lives. `orb-sim` reaching `orb` closes a cycle —
`orb-sim` → `orb` → `orb-core` → `orb-sim` — which cargo allows because the edge into `orb` is a
dev-dependency and so outside the normal build graph. See
[docs/adr/0005](docs/adr/0005-every-e2e-test-lives-in-orb-sims-tests.md).

That feature is why the DLL the game loads pays nothing for any of this: with `sim` off the install
point does not exist, and `mem::read` — called thousands of times a frame — compiles to the volatile
read and nothing else.

`orb-core` carries no `windows-sys` and it is not portable all the same: `Th06` calls the game's own
functions through transmuted pointers, by the conventions MSVC6 compiled them with — `thiscall` and
`fastcall` — and those exist on 32-bit x86 and nowhere else. What is on the far side of that is the
game's code rather than the host's, which is why no seam over Windows reaches it, and why the crate
being free of Windows is a boundary kept by hand rather than one a build proves.

## Reaching the game's memory with no game there

Every read and write of the game's memory goes through `orb_api::mem`, and a test can put an
address space of its own in front of the real one: regions at the game's own bases, holding bytes,
with `Th06` reading them through the same four functions it reads a running game through. So the
offsets, the structure walks and the patching are all exercised, rather than a second
implementation of `Game` standing in for them.

The space answers more than bytes, because that is what the code asks. A region can be
committed, reserved without being committed, or a guard page — which is how a pointer into a
structure the game has not built yet comes back as nothing rather than as a process that has
died — and it can be an image or an allocation, which is how a live COM object's vtable is told
from the stale pointer left in a block the allocator did not scrub. A restore that finds a region
gone commits it again, as it does when the game has handed a few megabytes back to the OS.

Installed per thread and taken off again when the test ends, as the rest of the seam is.

What this cannot catch is an offset that is wrong: the space is written from the same constants
the reads use, so a wrong one is wrong on both sides at once. Offsets are settled against
東方紅魔郷 1.02h running, each read held against what the screen showed, and each is written down beside
the offset it was read at — see `orb_core::game::th06::image`. Everything built on top of them
is what the space is for.

It also answers what a snapshot covers. In a real process that is a walk of the heaps the game took
from the OS, which the import hooks hand over as they see them; a laid-out space *is* the game's
memory, so it says which regions those are itself. Either way `memtrack` asks `mem::game_regions` like
every other host call, since a branch on `cfg(test)` there is a branch an e2e test does not reach — and
holding the answer to no two regions covering the same pages is the answering host's, a heap region and a
reservation being able to name the same ones where two laid-out objects that abut are two objects.

In a build without the `sim` feature the space does not exist and none of `mem`'s functions branch.

## Running the game with no game there

Above the memory there is a 東方紅魔郷 that plays the game's part rather than the address space's:
`orb-e2e/src/fake`. It owns a laid-out image, has a front end and a stage of its own, and calls orb's
hook bodies where the real game's code calls them — the draw chain and then the update, with the input read
inside the update, which is the game's own order. `orb_core::runtime::attach_to` puts a runtime in place with
its functions where the trampolines `orb`'s own install lists leave behind would be, so nothing is patched
and nothing is a real process — and nothing in that crate is named from an e2e test at all.

**And the device, the sound and the glyphs are the simulated Windows'**, not objects of the fake's own:
`orb_sim::DEVICE` is what the game writes into its own memory as its `IDirect3DDevice8`, `orb_sim::BUFFER`
is the buffer its music is played out of, and a string baked at a height comes back as a mask carrying
which string it is. So what an e2e test reads off the screen is what the drawing asked for rather than
pixels held against a second bake — see
[docs/adr/0009](docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).

**Its state is that memory and nothing beside it**, which is what makes a chapter restored underneath
it take the run back: there is nothing else for the run to be. Where each thing lives is
`orb-core`'s `th06::image`, beside the offsets it is written from; what the game does over time is the
fake game's.

**Each e2e test is a process of its own**, which is what lets them all run at once: a launch is a
process — the runtime, the record of what a run pressed, the score's file and the device are one apiece,
and the runtime cannot be per-thread because `DllMain` writes it on the injector's thread and the frame
hook reads it on the game's — so an e2e test spawns the test binary again, told to run that one test, and
reports what the child made of it.

**An e2e test says which window is in front, presses keys, and runs frames.** Everything it asserts on
it reads back: the game's own memory through `read_state`, the game's own records — the count of
attempts against a spell card — the quads orb drew, and orb's log. Nothing is added to orb so that a
test has something to look at, and nothing calls orb's own functions to move a run along: to have orb
in some state, the game is played into it. See
[docs/adr/0001](docs/adr/0001-a-fake-th06-drives-orb-end-to-end.md).

**Its own loop calls orb's frame loop**, as the real game's does, and that is what a shipped run has on:
the two calls the loop makes into the game are addresses `Game::frame_calls` hands over, so this game
hands over two of its own. Its `Present` is where an e2e test counts a frame handed over — the
tick, and the host told a frame is in the compositor's hands — and its `PlaySounds` is nothing, a laid-out
game having no sound system to hand a frame's sounds to. A launch started `--no-frame-loop` runs 紅魔郷's
own draw-then-update order instead, which is the other configuration orb ships, and an e2e test reads which
of the two ran off the order the loop asked the game for things in.

**And ten more of the game's own functions are handed over the same way**, because they are calls orb
makes into the game rather than reads of it: `CreateWindowExA`, which the window's rewrite calls through;
`CreateFileA`, which the score file's fork calls through; `ReplayManager::StopRecording`, which is held
back while a replay is being watched; `GameWindow::Create`, which the hook that overrules the display
setting calls through; `joyGetPosEx` as the import table held it, which the replacement of that entry calls
through where it has no sample of its own; `ReplayManager::SaveReplay`, whose write a
cleared run refuses; `GameWindow::InitD3dDevice`, which orb gets in front of to redirect the device's
`Present`; `Chain::Cut`, which takes a screen shake down at a stage move; and `SoundPlayer::StopBGM` and
`Supervisor::PlayAudio`, which a restore puts the sound down and starts it again through where the track has
been replaced since the chapter was taken. Those it reaches through a hook are `Originals`', and those it
calls in the game are `Th06`'s
own — see [docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).

**`Controller::GetControllerInput` is not among them**, and that is the one hook orb does not call through:
the pad half of the input read is orb's own answer, so there is nothing for a laid-out game to hand over —
it calls the hook where its keyboard read tail-calls that function and the word comes back with every pad in
it. See [docs/adr/0013](docs/adr/0013-the-pad-half-of-the-input-read-is-orbs.md).

**One of those seven moved a gate.** A real launch decides whether to refuse a replay write by *installing
the hook or not* — only under `--clear`. A game that hands its own function over has one call site whichever
way the launch was configured, so the decision cannot be the installation there: it is a flag orb sets where
`attach` decides to install, and a laid-out launch nobody asked to block a save writes one.

**Its update is the chain's own walk.** `Fake::update` is `Chain::RunCalcChain` over the list of jobs its
scenes have registered, in the order their priorities put them in — the supervisor at 0, the front end at 2,
the ending at 3, the gameplay scene at 4, the result screen at 13 and a bomb's screen shake at 14 — and the
list is walked live out of the game's own memory. Which is what makes a scene's first update fall on the
frame it was built: the supervisor is the first job and everything it registers goes in above its own
priority, so a job registered from inside the walk is linked behind the position the walk has reached. A job
answers as a chain callback does, and the walk answers the count of jobs it ran. See
[docs/adr/0008](docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md), which is where what the
fake owes the game it is standing in for, and what it deliberately does not, is settled.

**A track is streamed through a sound of the host's**, which is the one part of a laid-out game that is a
real object rather than laid-out memory: orb *calls* a DirectSound buffer's vtable and asks winmm for
`mmioSeek` and `mmioRead`, so `orb_sim::Sound` is a buffer of its own behind a vtable of Rust functions and
a wave file kept in a `Vec` — the same answer the Direct3D device orb draws through is. What the address
space is told is only where that object is, so that orb's check of the pointer at its head answers what it
answers in a real process. Which is what makes the music across a chapter answerable: the position a
chapter is written down with, which chapters put their song back, where the countdown the track's loop is
taken on lands after a seek, the buffer and the play cursor and the file position coming back byte for
byte with a chapter, and the distance to the next write that a listener hears the music break up once it
runs out.

**And the pads the machine has are the host's too.** One of them is the other of the two devices the game's
own read has — `Controller::GetControllerInput` asks winmm for joystick 0 only where its own enumeration
found no game controller — and the rest are pads only orb reads. So `orb_sim::Joystick` is every socket
winmm has and `orb_sim::Xinput` is XInput's four slots, both plugged in and pushed by an e2e test, behind
`orb_api::joystick` and `orb_api::xinput`; the game's own read reaches joystick 0 through orb's replacement
of that import entry.

**An e2e test declares the display the window is on**: what the monitor reports, what the compositor is
timing, what composing a frame takes, and which stream of wake delays the host has. Which is the whole of
what the pacing is paced against, and how the frame loop's own e2e tests exist at all — the rate read off
the ticks the game was handed its frames over at, and orb's own `frame:` line read out of the log beside
it. See [docs/adr/0002](docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).

Everything a game can drive is driven that way: the question that chooses a mode, on the keyboard and on
a controller the game answers with, the whole of a `State` read at frames a game reached by being played
to them, and the frame loop. **Every one of them is a `#[cfg(test)]` module of `crates/orb-e2e/src/`**, and the four
files beside them that do not begin that way are the ones no game drives — the log, and one `Pacing` that
is handed numbers rather than run. What `log!` formats and which level keeps which line is the log's own
business, and no game decides it. See
[docs/adr/0005](docs/adr/0005-every-e2e-test-lives-in-orb-sims-tests.md).

## Holding the game still, checked against real threads

The rule `threads` exists for — every thread the game made stops while its memory is copied, and
the one playing the music does not — is checked with real threads rather than a model of them, since
what is being relied on is what `SuspendThread` does. A test registers threads of its own as the
game's and reads whether each is still counting.

Counted turns rather than elapsed time: what says a thread is running is that it got somewhere
while another one was getting somewhere, and a wall-clock window is the flaky way to ask. The
exception is the case where nothing is left running to count against, which has to wait.

`SuspendThread` returns before the thread it names has stopped — measured as a dozen more counts
after the call came back. So what a test reads is the count once that window has passed, not the one
from before the call.

## Drawing into something that keeps it

The overlay needs no seam at all. A Direct3D 8 device is a pointer to a vtable, so a vtable of
Rust functions is a device as far as everything that calls one is concerned: the overlay creates
its state block, uploads its textures and draws its quads through exactly the calls it makes
against the game's device, and they land in a record instead of on a screen.

What is kept is the request — each quad's rectangle and colour, which texture it went through, the
clears, the viewports, in order — **and what was uploaded into each texture**. That answers the
questions worth asking of a frame: whether the mark covers the row the lives are counted in and
leaves the rows either side alone, whether a menu put its items where it says, which of the two
flashes a boundary got, and *which text is at which position* — a quad's texture is held against the
same string baked again through the same font, since a device is handed a bitmap and never a string.
The colour answers the rest: `menu_ui` draws the item under a cursor in `SELECTED` and the others in
`NORMAL`, so which one is chosen is on the screen rather than in a field. A
rasteriser would answer the same ones at the cost of being a rasteriser, and the one thing here
that is genuinely about pixels — the brush stroke's coverage — is baked from the picture by
`build.rs`, which is where it is checked.

## How it fits together

`orb.dll` is injected while the game is still suspended, so its `DllMain` runs before the
game's entry point and the memory hooks see the first allocation.

| | |
| --- | --- |
| `crates/launcher` | checks the exe, starts it suspended, injects `orb`, resumes it |
| `launcher/settings.rs` | the dialog that asks for every setting in `orb.yaml` before the game starts |
| `launcher/thcrap.rs` | the translation patch installed beside the game: what its own launcher says about where it is, its patch stack brought up to date, and the game handed to it |
| `launcher/pad.rs` | reading the pads on the launcher's side — winmm's joystick 0 and every XInput slot — so that dialog answers to one |
| `crates/orb-config` | `orb.yaml` — read by both halves, written by the launcher — and the command line |
| `crates/orb-api` | the seam: the `Win` trait, the neutral types, and the facades every host call goes through |
| `orb-api/real/` | the Windows behind it — `#[cfg(windows)]`, and the only part of the crate that is |
| `crates/orb-core` | everything that decides what happens to a run, over that seam and with no `windows-sys` anywhere in it — 32-bit x86 all the same, since `Th06` calls the game's own code by its own conventions. `cargo xtask seam` holds it to that |
| `orb-core/frame.rs` | the frame loop's pacing and its measurements |
| `orb-core/input.rs` | the keyboard orb reads for itself, and what it does when the game is not the window in front |
| `orb-core/menu.rs` | the keys orb's questions read, whose press each is, and where a cursor over them goes |
| `orb-core/mode.rs` | the two modes, the question that chooses between them, and what each choice says |
| `crates/orb-sim` | the simulated Windows: the memory, the clock, the display, the keyboard, the mouse, the pad, the sound, the device orb draws through and the strings it bakes. In its `tests/` are the four no game drives |
| `crates/orb-e2e` | the launches: a game playing the game's part in `src/fake/`, compiled once, with every e2e test a `#[cfg(test)]` module beside it |
| `orb-sim/display.rs` | a monitor and a compositor a test declares: the refresh period, what the compose takes and how often it spikes, and the blank a flush returns at |
| `orb-sim/window.rs` | the panel a test declares and the window manager over it: the two sizes one monitor reports either side of `SetProcessDPIAware`, the frame it costs to get a client of a given size, the windows it has been asked to make, and every stack of lines it has been asked to write in the black beside the game |
| `orb-sim/keyboard.rs` | the keys a test holds down, the keys another program sent — which `GetKeyboardState` reports and an exclusive foreground device does not — and a host that refuses to say what is down at all |
| `orb-sim/mouse.rs` | the pointer a test moves, the display counter `ShowCursor` keeps and how many times it has been asked to move it, and a host that refuses to say where the pointer is |
| `orb-sim/noise.rs` | the seeded stream the host's delays are drawn from, so a run that fails replays |
| `orb-sim/space.rs` | an address space laid out by hand, which is how a test has a game to read |
| `orb/lib.rs` | `DllMain` and the install lists: which prologue goes with which hook, and which of them a `Config` asks for |
| `orb-core/runtime.rs` | what those hooks *do*: their bodies, the `Runtime` they carry between frames, `Originals` — the game's own calls each one goes on to make, all but the pad read, which is the one nothing goes on to — and `attach_to`, which is how a game laid out by hand is attached to with no process to patch |
| `orb/hook.rs` | trampoline and import-table hooks |
| `orb/memtrack.rs` | the import hooks that notice the heaps and reservations the game takes from the OS, each handing over what it saw |
| `orb-core/memtrack.rs` | the set those make up, as a snapshot asks for it |
| `orb-core/snapshot.rs` | save and restore of `.data`, those regions, and the music |
| `orb/threads.rs` | the `CreateThread` import, which is the only way to know which of the process's threads are the game's. Suspending them is `orb-api`'s |
| `orb/joystick.rs` | the write over the `joyGetPosEx` entry, which is the one thing here no e2e test reaches |
| `orb-core/joystick.rs` | the thread that samples every socket a pad can be in off the game's own, the entry's replacement answered out of joystick 0's last sample, and what a sample means: whether what answered is a pad, which of the pads was last pushed, and what the word and orb's own menus read off it |
| `orb-core/audio.rs` | the sound buffer and file position, which live outside the game's memory |
| `orb-core/chapter.rs` | where chapters begin, and which snapshots are kept |
| `orb-core/resume.rs` | the buttons a run has pressed, and the file that lets its chapter be played to again |
| `orb-core/retry_ui.rs` | the menu shown where the chapter was lost |
| `orb-core/lives_ui.rs` | the brush stroke over the game's count of lives, for a run that cannot lose one |
| `orb-core/build.rs`, `orb-core/brush.png` | that stroke, and the bake that turns the picture of it into coverage |
| `orb-core/mode_ui.rs` | that question drawn — the labels and the wash. What it decides is `orb-core/mode.rs` |
| `orb-core/resume_ui.rs` | the question after the character select: from where it stopped, or from the beginning |
| `orb-core/menu_ui.rs` | the list they draw, and the colours they draw it in |
| `orb/score.rs` | the write over the `CreateFileA` import |
| `orb-core/score.rs` | that entry's replacement, the walk of the path the game handed over, and which file the open lands in: the fork's choice of name, and the refusing of a clear run's write |
| `orb-api/mem.rs` | the reads and writes of the game's memory, and what makes an address safe to read |
| `orb-api/real/mem.rs` | the page operations behind that — committing what a restore needs, unprotecting it, swapping a word in a page that is read-only — and the walk of the heaps and reservations the game took, which is what a chapter is a copy of |
| `orb-api/window.rs` | which window is in front, the sizes the host decides — what the monitor measures, the frame it puts round a client area, and the client a created window came out with — the GDI a stack of lines is measured and written to the window with, and the modal orb puts up itself |
| `orb-api/mouse.rs` | where the pointer is, and the display counter the host draws it by |
| `orb-api/clock.rs` | the counter, the stamp every log line carries divided down from it, the wait to a frame's own deadline, and the coarse one a thread nobody is waiting for takes between two reads of a device |
| `orb-api/codepage.rs` | `MultiByteToWideChar`, for the one string a Win32 `-A` call answers in the machine's own code page: the name winmm gives a pad |
| `orb-api/locale.rs` | `GetUserDefaultUILanguage`, which is the language orb writes its screens in where `orb.yaml` names none — apart from the code page because Windows keeps the two apart |
| `orb-api/real/gdi.rs` | the GDI calls that carry orb's own text, read out of gdi32's export directory rather than called through this DLL's imports, which something else injected into the game rewrites — see [docs/adr/0015](docs/adr/0015-orbs-own-text-leaves-through-gdi32s-exports.md). Below the seam and reached from `real/window.rs` and `real/text.rs` alone |
| `orb-api/joystick.rs` | the joysticks winmm has: joystick 0, which is the one the game's own startup check asks about, and every other index it will answer about |
| `orb-api/xinput.rs` | the pads XInput has, which is where the pad is on a machine whose winmm has only the phantom. The library it is read through is loaded by name, which of the three being the machine's business |
| `orb-api/process.rs` | ending the process, for the one host orb declines to run on |
| `orb-core/tuning.rs` | building the midstage table |
| `orb/window.rs` | the writes over the two window imports, and the black brush the rewrite of `RegisterClassA` swaps in |
| `orb/mouse.rs` | the write over the `ShowCursor` import, which is what leaves the display counter orb's |
| `orb-core/mouse.rs` | how long the mouse has to have been still for the pointer to go, the read that follows it, and that entry's replacement — the game's own ask answered rather than passed on |
| `orb-core/window.rs` | how big that window is and where it goes — the style, the centring, and the rectangle a 4:3 game is presented into — the device's `Present` slot redirected into it, and where orb's own lines go in the black beside it |
| `orb-core/overlay.rs` | drawing over the game's frame: the state block round every draw, the quads, and the labels and pictures baked into textures |
| `orb-api/d3d8.rs`, `orb-api/real/d3d8.rs` | the slots of the game's device orb calls, and the only code in the tree that calls a Direct3D vtable |
| `orb-api/dsound.rs`, `orb-api/real/dsound.rs` | the eight of the buffer its music is played out of, the same way |
| `orb-api/text.rs`, `orb-api/real/text.rs` | a string baked to a coverage mask, and the GDI that bakes one |
| `orb-sim/drawing.rs` | a device that keeps what it was asked to draw, so an e2e test can say what is on the screen — and which string went into each texture |
| `orb-sim/text.rs` | the fonts an e2e test says are there, and what a string comes out as: a declared metric rather than a rasteriser |
| `orb-core/log.rs`, `profile.rs` | the log and its levels, and where a frame's time went |
| `orb/crash.rs` | the handler that names the module and offset a fault happened at |
| `orb-core/game/mod.rs` | `Game` and `State`: everything above is written against these |
| `orb-core/game/th06/` | the addresses and offsets that make it 東方紅魔郷 |
| `orb-core/game/th06/image.rs` | those addresses laid out in a simulated Windows, so the real `Th06` has something to read — and where each thing a game does to its own memory is written |
| `orb-e2e/src/fake/` | the games that play the game's part. `mod.rs` is the half any launch has — the display, the device orb draws through, what a frame's own work costs, and `in_its_own_process`; `th06.rs` is 紅魔郷's own memory, front end and stage, with orb's hooks called where the real game's code calls them, and `th07.rs` is as much of 妖々夢 as `Th07` reads |
| `orb-e2e/src/pointdevice_run.rs`, `orb-e2e`'s `legacy_run` | the two e2e tests over a whole run, which press keys and read back the game's memory, its records and orb's log |
| `orb-e2e/src/pacing.rs` | every e2e test about orb's own frame loop, in a section apiece, over the functions that judge a rate: the moments the game was handed its frames over at, and orb's own `frame:` line taken apart |
| `orb-e2e/src/mode_question.rs`, `orb-e2e`'s `mode_on_the_pad`, `orb-e2e`'s `mode_on_a_winmm_pad` | the question over the game's title menu answered on the keyboard, answered on a controller the game owns, and answered on a pad winmm has where the game owns none — with the empty socket and the pad that turns up in it later beside it |
| `orb-e2e/src/the_dpad_moving_the_player.rs` | the player moved by the d-pad on both of the devices the game reads a pad on, in the direction it points and by the step a held direction moves them every frame, with the pad at rest moving nothing and `dpad_moves: false` handing the game the word its own read produced |
| `orb-e2e/src/a_pad_the_game_has_no_device_for.rs` | a pad the game holds no device for driving the run: its d-pad, its stick and its buttons in the word, the player held still on a focus button and on one button that is both focus and shoot held for the frames the game counts, a pad at a winmm index the game never asks about, a question of orb's answered on the pad in XInput's second slot where winmm has only the phantom — and two pads in front of one person, where the run is played with the one last pushed and the other is not merged into it |
| `orb-e2e/src/the_pad_half_of_the_input_read.rs` | the other half of that: what reading a pad produces, over the device the game's own enumeration found — every mapped button as its bit, each axis against its own threshold, the player held still off one button that is both, a device whose poll fails leaving the word as the keyboard read it and being asked for again, the hat as the only thing `dpad_moves` takes out, and what the read costs reaching the `perf:` line |
| `orb-e2e/src/the_run_read_back.rs` | `Th06::read_state` — every offset, every pointer chase — over a game that got where it is by being played |
| `orb-e2e/src/the_window.rs` | the window orb makes on a monitor the e2e test declares: the client being the size asked for whatever the frame costs, the monitor's real pixels once the process says it is DPI aware, the black either side of a 4:3 game, and the status line written in it — which of the two bars, at which height, where the block landed, and a shorter stack afterwards clearing the rows the longer one wrote in |
| `orb-e2e/src/the_mark_over_the_lives.rs` | the two edges of the mark over the count of lives — the one frame a stage transition takes, the frame a chapter is put back on, and the frame the game paints after the run has ended — and the panel's own tile the strips beside the count are painted with |
| `orb-e2e/src/a_clear_on_demand.rs` | `--clear` through six stages with a bullet sitting on the player, the screen that saves a replay written past rather than answered, and neither score file written |
| `orb-e2e/src/the_score_file.rs` | which of the two files each of the game's own opens lands in: the front end's read, which is the game's own file whatever the mode, and each mode's ranking screen reading and writing its own |
| `orb-e2e/src/th07.rs` | a laid-out 妖々夢 with orb attached to `Th07`, which asks that orb got in and did none of what it does to 紅魔郷 |
| `orb-e2e/src/keys_from_another_program.rs` | `--sent-keys`: a key another program sent, refused by the keyboard device the game holds exclusively and seen once orb has let that device go, and the two moments in the front end that spend a press on nothing |
| `orb-e2e/src/the_ending.rs` | the ending run out inside the frame it begins on, stopping where its script hands over to the staff roll and the track changes on the same update, and the roll left to play at sixty |
| `orb-e2e/src/moving_between_a_replays_stages.rs` | a replay moved between its stages: the teardown's write into the record held back, the score and the extra lives put back to nothing, a screen shake taken down before it reaches the next stage, and two passes over one stage agreeing to the last digit |
| `orb-e2e/src/the_music_across_a_restore.rs` | which of a stage's chapters put their song back, asked of the song rather than of the chapter's kind; a seek that moves the countdown with the file so the track loops where it did; the buffer, the play cursor and the file position coming back byte for byte with a chapter; the track that has gone since taken down and started again through the game; and a file handle orb cannot read, where the buffer comes back and the file is left where it is |
| `orb-e2e/src/a_chapter_table_collected.rs` | `--collect` and `--judge`: the gap in a stage's waves proposed, the boundary judged out and stepped back to and back into the table, one placed by hand and taken away again, both files written, what one sitting decided read back by the next, and a hand-edited state file whose unreadable lines are named by path and line |
| `orb-e2e/src/a_chapter_out_of_the_table.rs` | the boundary the baked table holds for a stage beginning a chapter, with the fight won first so that nothing outranks the table, and the Extra reading the row of its own |
| `orb-e2e/src/a_practice_run.rs` | one stage practised: the mode question over its own item of the title menu, the chapter put back after a death, and nothing written down for a run the game keeps no slot for |
| `orb-e2e/src/the_handles_a_restore_leaves_alone.rs` | a texture handle the game holds left where a restore finds it, and the rest of the block it is in put back |
| `orb-e2e/src/the_window_going_behind.rs` | the keys dropped while the game's window is behind, and the keyboard device taken again — as many times as it takes — when it comes forward |
| `orb-e2e/src/the_mouse_pointer_over_the_game.rs` | the pointer taken off the screen three seconds after the mouse was last moved and put back on the frame it moves again, with the wait running from that movement rather than from the launch; the display counter one step from where it started; the pointer put back for as long as the game's window is behind, and the wait run again from the frame it comes forward; the game's own ask for the pointer answered without reaching the host; and a host that will not say where the pointer is left alone |
| `orb-e2e/src/a_stage_transition.rs` | what a stage transition carries and what only the start of a run puts in place: the lives, the bombs, the power and the deaths a run walks through six stages with, the one read of the score file a run makes, the rank its difficulty is played at, the arcade region, and the box the player is held inside |
| `orb-e2e/src/the_frame_a_scene_is_built_on.rs` | a scene's own first update falling on the frame it was built, at a stage's transition and at the front end alike, with the input word zeroed there so a button still held reads as a fresh press on the frame after |
| `orb-e2e/src/the_player_a_stage_starts.rs` | the player a stage starts: invulnerable with the first of 240 frames already spent, and the 240th the one a bullet sitting on them kills on |
| `orb-e2e/src/the_launch_before_its_device.rs` | a launch orb is attached to before the game has a Direct3D device, which is every real one: nothing drawn until the game's own setup runs, and the overlay ready once it has |
| `orb-e2e/src/the_screens_in_english.rs` | a launch on a machine whose own windows are English with nothing in `orb.yaml` about the language: the question over the title menu, the menu where a chapter was lost with the question one of its items asks, and the question about a run left unfinished with the mark before it — each read back in English with none of its Japanese anywhere on the screen |
| `orb-sim/tests/log_writes.rs`, `log_off_thread.rs`, `log_overflow.rs`, `log_refused.rs`, `pacing_no_timer.rs` | the ones no game drives, which is what their being `orb-sim`'s rather than `orb-e2e`'s says |

Only `th06` implements `Game`. Porting to another Touhou game means supplying its addresses
and offsets.

## Not supported

- Restoring a snapshot across launches: it holds pointers to Direct3D and DirectSound objects, and
  those addresses differ per launch. A chapter is reached again by the run being played to it
  instead — see *Picking a run up again*.
- More than one chapter of the same run written down. Its file holds the newest, and starting that
  same run from the beginning replaces it. Runs of different difficulties, characters and shots each
  have their own — see *Picking a run up again*.
- Returning the music to where it was across a track change: the track starts again from its
  beginning instead. See *Chapters and retries*.
- A replay of a pointdevice run, because a rewound run does not play back: the screen that offers
  to save one is skipped rather than the write being refused. Its score is kept, in orb's own
  file — see *The score file*. A normal run saves replays the way the game always did.
- Sound effects cut off on a restore rather than rewinding. Only the music is restored.
- An exe orb's addresses have not been read against, whatever put it there: a translation patch that
  rewrote the exe, a build of the game orb has not been taught, another game under a name orb reads.
  The launch is refused naming every game and build orb knows — see *Which game a launch is*. A patch
  that leaves the exe alone is not this: what orb reads off the file is its md5, and a patch applied
  while the game runs does not change one.
- Making the game itself run on a machine whose code page is not Japanese. 紅魔郷 opens its own data
  files through `CreateFileA` with their names held in the exe as Shift-JIS bytes — `紅魔郷ST.dat` at
  file offset 0x6af74 and five more like it — and rasterises its own text through `CreateFontA` and
  `TextOutA`, so a machine whose code page reads those bytes as something else is a game that cannot
  find its own data. That is what a locale emulator or thcrap's own locale independence is for, and orb
  does nothing about it either way: it hooks that import for one thing only, which is which file the
  score goes in — see *The score file*.
- A thcrap that left no launcher of its own in the game's directory. Its configuration tool also offers
  shortcuts instead of wrapper exes, and the desktop and the start menu instead of the game's directory,
  and orb looks in none of those: the game starts untranslated and the line says none is installed where
  the game is. A wrapper exe copied into the game's directory is read like any other — see *The translation
  patch beside the game*.
