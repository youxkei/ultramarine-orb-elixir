# Ultramarine Orb Elixir — specification

What orb does, and the facts about 東方紅魔郷 1.02h and about Windows that shape how. The
README is the guide to using it; this describes the thing itself.

**This document describes the final form only.** No history of what was tried and rejected,
and no record of what a mechanism used to be — only what it is, and the facts it rests on. The
reasons an alternative was rejected belong in a comment beside the code that would otherwise
tempt someone back to it, which is where they are. What has been checked and how is in
[DONE.md](DONE.md); what is left is in [TODO.md](TODO.md).

Only 1.02h (`md5 fa3d64768b1bfc50703dedc2db92f7fa`). Every address below was read off that
build and cross-checked against the
[GensokyoClub/th06](https://github.com/GensokyoClub/th06) decompilation.

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

Three ways on, and the chapter is the first of them:

| | |
| --- | --- |
| チャプターをやり直す | the snapshot taken where this chapter began |
| ステージをやり直す | the snapshot taken where the stage began, which is chapter 1 |
| タイトルに戻る | the run given up, and the game on its way to the title menu |

The third is named for where it ends up rather than for what it gives up, that being the half of
it somebody reading the item does not already know: that the run is over is the obvious part, and
that the game carries on into its own front end is not.

**The chapter acts on the press; the other two ask first.** A fight worth grinding loses a
chapter every few seconds, so the first item is answered hundreds of times in a session — a
question in front of it would be answered without being read, and that trains the hand which then
answers the other two. Those two are one press away from that hand and neither can be taken back:
the stage's start throws away everything the stage has gained since it, and giving up throws away
the run. So each of them puts up a second question naming what it is about to do, with the cursor
on いいえ, which is where the game's own quit question puts it.

The question is the whole of what is said there. A line under it spelling out what would be lost
was tried and taken out: the question already names what is about to happen, and a screen somebody
arrives at by dying is not the place to be read at.

`x`, escape or the pad's cancel takes a confirmation back to the three items.

**Neither of orb's menus writes its keys on the screen.** Both had a `Z 決定    X 戻る` line under
them and both lost it: the keys are the game's own — `z` shoots and `x` bombs, which is what its own
menus take as decide and back — so the line was telling somebody playing 紅魔郷 the one thing they
already know. Where a way out is worth pointing at, the screen does it with an item rather than with
a key: a confirmation's cursor sits on いいえ, and the retry menu's third item says where it goes.

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

**What is asked, and where.** `どこから始める`, with `つづきから` and `はじめから`, on the game's own shot
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

**And said before the press, as a mark rather than a question.** `中断データあり`, with the
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
| 完全無欠モード | chapters, snapshots, the retry menu, the wash a chapter gets, the lives painted over, the retry count on the status line, and `pointdevice_score.dat` |
| レガシーモード | the game as it was: dying costs a life, a replay can be saved, and the score goes in the game's own `score.dat` |

On screen they are 紺珠伝's own two names, since that is where the mode comes from and those are
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
the tree carries is the picture of it: `crates/orb/brush.png`. `crates/orb/build.rs` bakes that into
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
sleep                  until (blank + one frame) − (our drawing + the compositor's)
update(chain)          the game's logic; reads the keyboard as its first act
draw(chain)            between BeginScene and EndScene
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

**Cadence.** One game frame is a whole number of refreshes of the monitor the game's window
is on, taken from `MonitorFromWindow` and `EnumDisplaySettingsW`: two at 120Hz, one at 60Hz.
The compositor's `qpcRefreshPeriod` follows one monitor of the desktop, which on a mixed-rate
desktop need not be this one, so it is used only when it agrees with that rate.

**Both of those numbers are rounded, and neither rounding may decide anything.**
`dmDisplayFrequency` is whole Hz, so the NTSC-derived rates report short — 119.88Hz says 119, and
59.94Hz says 59 — while the compositor's period in whole microseconds puts the same display at
120. Two consequences, and both were live faults until a 119.88Hz display was actually run:

- A rate within two per cent of a multiple of 60 *is* that multiple, and gets its constant count.
  Read as fractional instead, 119 sends the grid chasing an exact sixtieth against a rate 0.8%
  away from one, which it settles by putting a one-refresh frame in about once a second. The
  period is taken from the nominal multiple as well, that being the nearer of the two: a 119.88Hz
  refresh is 8341µs, which 120 puts at 8333 and 119 at 8403.
- Agreement between the two is within two per cent, not equality. At 119 against 120 an equality
  test refused the blanks altogether and paced a 119.88Hz display by the clock. What the test is
  for is the compositor timing a different monitor — 144 against 120, which ran the game at 72
  frames a second — and that is not a rounding apart.

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
That settles: measured at 144Hz as an aim averaging 2.2 refreshes with a frame in five landing a
refresh late, which adds back to the 2.4 the display wants, so the rate reads correct while a
fifth of the frames are shown somewhere nobody asked for. Anchoring the aim instead took that from
117 frames of every 600 to none.

A grid moment the blank in hand has already passed is a frame that has been missed, and it is
dropped: the grid starts again one frame from that blank. Left where it was, the aim comes out at
one refresh and stays there until the difference is made up — and making it up means an update per
refresh, so the frames that were missed are paid for by running the game fast, which brings none of
them back.

A rate that *does* divide is given the same count every frame instead of following that grid.
A display sold as 120Hz is often 119.88, and chasing an exact sixtieth there would spend three
refreshes on a frame every few minutes to make the difference up. Two every time is 59.94fps —
a tenth of a percent on a clock nobody can see, against a hitch anybody can.

Only a display whose refresh rate is not known at all, or one the compositor will not admit to
timing, is paced by the clock. A replay being run fast, or a run being cleared fast, keeps
the cadence like anything else: `--speed` is updates per drawn frame, so the frames come one per
turn and only carry more of the game with them.

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
the two cases do not overlap: every frame that made its blank came back within ±900µs of it, and
every frame that missed came back 5944µs or more late, on a refresh of 8333. Half a refresh is
the boundary and one frame decides it — nothing to average, and no waiting for a fault that
happens three times in six hundred frames.

Raised 100µs the moment a frame misses, and **never lowered**. Every microsecond of it is input
lag on every frame, so the least that works is what is wanted — but the only way to learn that a
value is too little is a frame missing its blank at it, so a value that comes down is a value
being wagered again, and a lost wager is a stutter in the middle of a run. It starts at 2500µs
and climbs from there; a display wanting less pays the difference in lag and nothing else.

Shaving it back was tried and is why it is not done. From 2000µs at 100µs a second the first
frame missed at 2000, which set a floor of 2100 that said nothing about 2100 being enough; the
walk down passed the real edge without dwelling anywhere long enough to catch a value that only
fails sometimes; and from 2100 it climbed back at a stutter a step, through the whole of stage 1.

Three kinds of miss are counted and only one of them climbs:

| | |
| --- | --- |
| overshoot beyond a whole turn | a stage load, or an update that ran long |
| the frame after one of those | still picking itself up, and not the compositor's to answer for |
| a frame whose own drawing outgrew its budget | late whatever the compositor had been given |

The middle one was measured: of three climbs over a 37,800-frame replay, two happened in the
periods where a boss appeared — the game stopping for 225ms while it loads one — and the value
they climbed from had sat through thirteen quiet periods without missing once. The last was
measured too, and worse: at 144Hz a heavy frame at startup climbed the share to its ceiling, and
since the budget was capped at the same figure the drawing had no allowance at all, so every
frame reached the compositor late and every one of those asked to climb again. 120 frames of
every 600, for the rest of the run.

**Two ceilings, and they are not the same.** A frame is handed over `compose` before the blank it
is aimed at — that is the whole of what decides where the handover lands, because the drawing
happens before it and only moves when the drawing starts. So it is the compositor's share, and
not the budget, that has to stay inside one refresh: hand over earlier than the blank before the
aimed one and the compositor takes it at that earlier blank. The budget may run to most of a game
frame, since it only decides how early the drawing starts — but only while it is a prediction of
what the work takes, which is the same trap from the other side: see *The work estimate*.

Getting that wrong is invisible at 120Hz, where half a game frame is exactly one refresh. At
144Hz a refresh is 6944µs, and a share of 8333 collapsed the gaps to one refresh apiece — `gaps
in refreshes 1x418 2x179`, a hundred frames a second.

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

A frame that wanted more than the whole budget is left out of that. It is a scene being built
rather than a heavy frame — `RunCalcChain` runs 252ms where a run ends and the next one is set up —
and it says nothing about what the frame after it will take. Believed, it pinned the estimate to
the ceiling, and the frames that followed, two milliseconds of work apiece, were then started
12.5ms before a blank 8.3ms away: handed over that early they were composed for the blank *before*
the one they were aimed at, `DwmFlush` returned there with them, so the anchor the next aim is
counted from moved a refresh early and the frame after was handed over just as early again. One
frame per refresh is one update per refresh, so the game and everything in it ran at double speed
for the thirty frames the estimate took to decay back — after every stage load, and with nothing in
the log saying so, since the buckets take `max(0)` of the overshoot and a frame a refresh early read
as one that landed exactly where it asked to. Those are counted now.

`timeBeginPeriod(1)` is asked for at startup and released on detach. Without it `Sleep` is
only accurate to the system tick, some fifteen milliseconds.

**What a late frame says.** `--pacing` writes a line per frame whose gap was not the cadence,
and the line accounts for the whole gap in spans that add up to it:

```
after present   what orb did once the last frame was handed over
loop            the game's own frame loop, between orb returning and being called back
clear           prepare_frame
pace            the frame's turn worked out, settle's display query inside it
flush           DwmFlush, and how far its anchor sits after the compositor's own qpcVBlank
sleep           the rest of the turn
update sound draw present
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
made only while `--pacing` is on, and after the flush, so it is spent out of the slack rather
than out of the compositor's own.

Per period, alongside the gap buckets, the worst arrival against the blank and how many frames
were past it — the rate being what says whether a stutter is one cause or the weather. Arrivals
beyond a whole turn are counted apart and left out of the worst: the frame after a stage load
is that late through no fault of the compositor, and one of those would otherwise be the
whole of the worst.

**The log is an instrument and is weighed as one.** A `WriteFile` takes what it takes, and one
in the millisecond before a handover costs that frame a refresh, so what writing the log cost
is written down beside what it is reporting — per frame and per period, the frame's own thread
and orb's kept apart, since the appends serialise on one handle and either can hold a frame up.
For the same reason the frame loop's own lines are not written where they are worked out: they
are held and written on the far side of the flush, where what is left of the turn is slack.

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

**The joystick is read on a thread of orb's own.** `Controller::GetControllerInput` (0x41cfc0,
`__cdecl`) is a tail call inside `GetInput` that adds a joystick's buttons to the keyboard's.
Where it gets them is `joyGetPosEx(0, JOY_RETURNALL)`, winmm's, through the exe's import
table. The DirectInput branch beside it is only entered where the game's `EnumDevices` found
an attached game controller at startup — where none was, `g_Supervisor.controller` (0x6c6d2c)
stays null and every frame goes to winmm.

Where nothing answers, that call takes 8.7ms and spends nearly all of it on the CPU — see
[DONE.md](DONE.md) — which is half a 16.67ms frame, and being work rather than waiting there
is nowhere cheap in the frame to put it. Where a joystick does answer it costs under a
microsecond, so what it charges for is the looking and not the reading. orb
redirects the exe's import of it and answers the game out of the last sample a thread of its
own took: every 4ms while a joystick answers, once a second while none does, and never sooner
than the read itself took, so no device can hold a core of its own. What a sample means —
which button is shot, where an axis becomes a direction, the auto-repeat behind holding one —
is left to the game's function, all of it downstream of the call orb replaced.

Where a controller was enumerated the frame's read is that other branch's `Poll` and
`GetDeviceState`, which orb leaves alone, and the sample answers only the startup check that
asks whether a pad exists at all.

**A menu of orb's reads whichever of the two the game reads**, and that is not a nicety: on this
machine, with the pad in XInput's second slot, winmm has no pad at all. `joyGetNumDevs` says 16,
index 0 answers `joyGetPosEx` with `JOYERR_NOERROR` and every field zero — `mid=413d pid=2104`,
no buttons and no axes, which is what Windows leaves there while the slot the pad is in is not
the first — and 1 to 15 are all `JOYERR_UNPLUGGED`. DirectInput has the pad, the game therefore
has it, and orb's own menus had nothing. Those numbers are the measurement and they are in
[DONE.md](DONE.md); what took them is a probe of the same shape as the one the joystick read's own
figures came from, which lives outside the tree like that one.

So a menu of orb's asks the game, and 紅魔郷 answers by trying its own controller first: `Poll`,
then `GetDeviceState` into the `DIJOYSTATE2` the format it set fills, and the buttons and the Y
axis read out of it exactly as `Controller::GetControllerInput` reads them — the buttons by the
same numbers the mapping names, since `rgbButtons` is indexed by the very number a winmm mask is
shifted by, and the axis against `cfg.padYAxis` in the ±1000 the game gave every axis. Only where
there is no such device does the winmm sample answer.

The acquire after a lost device is orb's own to do there. The game takes its controller
`DISCL_EXCLUSIVE | DISCL_FOREGROUND`, so anything that took the foreground away leaves it
unacquired — and the frames a menu of orb's is up are exactly the frames the game's own read,
which is where that acquire lives, is frozen out of. Asked for once and the frame given up, the
way the game asks.

**A device with no buttons and no axes is not a pad.** What it is answered with goes to the game
unchanged, since that is what the game would have read for itself, but it drives no menu of orb's
and its caps are not written into the game's calibration — the axes of a device that has none
describe nothing. The log names it rather than reporting a pad.

**The calibration goes with the sample.** `GetControllerInput` places the centre of each axis
at `(wXmin + wXmax) / 2`, with a dead zone of a quarter of the travel, out of the `JOYCAPSA`
at 0x69d760 — which the game fills once, in its startup check, and only where a joystick
answered there. A pad plugged in later is measured against zeros, and its centred axes read as
held over. So orb writes the answering device's caps there, with every sample it hands over
rather than once when the device appeared: the caps are in `.data`, and restoring a chapter
from before the pad arrived puts the zeros back.

There is no setting for any of this. There was one — the read cost most of a frame and turning
it off was the only way out — and now that the frame pays a copy there is nothing left for it
to be off for. `GetControllerInput` is still hooked at `verbose`, to time what it now costs.

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
ask for, so `1280x720` is 1280x720 of game however thick this machine's frames are. A window too
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
the black beside the game with it, and with that the numbers written there. Measured on a machine
running one — a `1280x720` window was created with a client of exactly 1280x720, still 1280x720
when the device was created, and 2880x2160 three and a half seconds later, which is 4:3 filling the
height of a 3840x2160 monitor. The client the window came out with is logged next to the size asked
for, and again at the device, so a run where this happens says so rather than looking like orb
getting the size wrong. vpatch's `[Window]` section is the same hazard from inside.

A monitor-sized back buffer would not scale the game: its 2D drawing uses `D3DFVF_XYZRHW`,
whose coordinates are already screen space, so a viewport does not transform them. Scaling
that way would need a render target and a full-screen quad.

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
it on the screen. An update of an ending cost 13µs when the whole 36,932 were timed, which puts
29,040 of them under 400ms; what the frame the skip runs in actually took is not measured, only
that it was five refreshes or more.

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
capturing the card is what clearing the chapter *is*, and keeping the two apart would mean orb
knowing which word of the record is which.

**A session that stops without the game writing is taken through the ranking.** The write is reached
from one place, so a run given up or quit with `ESC` leaves what it counted about spell cards in
memory and nowhere else — 紅魔郷 loses it, and orb does not. On the first frame the front end is up
after a run that ended anywhere else, orb writes `MainMenu.gameState` with what the `Score` item
writes — 0xa, from the handler at 0x437f56 — runs the updates that brings inside that one frame with
nothing drawn, and writes `curState` back to the front end the way the ranking leaves itself. The
deleted callback then writes the file. Bounded at 180 updates, against the 60 the front end waits and
the one each scene change costs; at 30µs an update the whole trip is under a frame's worth of time
and none of it is seen, the draw chain running once a frame after this.

**The record is put back in the middle of that trip.** The added callback fills the captures out of
the file it read, which is what they were before the session counted anything, so orb puts back what
it took before asking the screen to leave. The ranking is the file's own and has to be: the write
takes it out of the table that read just filled — which is also why the trip is through the game's
screen rather than orb calling the write itself.

Whether a run wrote on its own is asked rather than assumed: every open for writing is noted, and a
run that ended at the result screen has one. Both modes make the trip. A legacy session stopping
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
orb's file and unlocks nothing, the front end being lit from `score.dat`. Whether a pointdevice
clear ought to unlock the Extra stage is in [TODO.md](TODO.md), and would be the union of the two
files' `clrd`.

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

`orb.yaml` is five keys: `screen`, `skip_ending`, `always_draw`, `boundary_flash` and
`ask_at_startup`. YAML read with serde; four switches written `true` or `false`, and `screen`
written `fullscreen` or a size like `1280x720`. `deny_unknown_fields`, so a key nobody reads is
an error naming it — including one that used to be a key, which is a file to edit rather than one
to pass over quietly. A setting that is not read is a setting somebody thinks is on.

**The launcher asks for all five before it starts the game**, and writes back what it is told.
Which is why they are the five they are: each is about the machine the game is being played on,
and somebody who has just installed one file has nothing to edit. It is orb's own window rather
than a dialog resource — a resource means a `.rc` and a resource compiler in the build, for six
controls — with the system's own message font asked for rather than a face named, since a face
named here is one that is missing on somebody's machine and the text is Japanese.

**A real dialog, class `#32770`**, from a `DLGTEMPLATE` built in memory: a resource would mean a
`.rc` and a resource compiler in the build, and this is six controls. Being one rather than looking
like one is the point — a window manager decides whether to leave a window alone by asking what it
is, and the dialog class is the answer it looks for. A window of orb's own class with a dialog's
styles was tiled by the one on the machine this was run on, which is how the difference showed up.
Its measurements are therefore in dialog units, a quarter of the font's average character width
across and an eighth of its height down, so the whole of it scales with the font the system gives
it and nothing in it is in pixels — which is also why display scaling costs the dialog nothing. The
dialog manager brings the rest: tab between the controls, return on `はじめる`, escape on
`やめる`. The window sizes
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
its own — the same call the game makes, and slow for the same reason: 15ms with a pad awake and 33ms
with none, so a message loop cannot be made to wait for it — read again as soon as each read
finishes, because a press is only ever seen if a read lands while the button is down. A cycle that
waited 120ms between reads was 155ms long and lost quick taps between two of them, which is what a
pad that answers *sometimes* looks like; the launcher now also prints whether a pad answered at all
and how many pushes it sent, since a pad that was never there and one that was pushed and ignored
want opposite things done about them. The thread posts what it sees to the dialog,
which turns it into what the dialog manager and the controls already answer to.

**What the pad does is a menu's, not a dialog's.** Up and down move a row at a time, and the two
buttons are one row — left and right choose between them, the way a menu with two answers on one
line works. Left and right otherwise change what the row holds more than one of, which is the list
of sizes; a switch is on or off, so turning one over is the decide button's job rather than a
sideways push, which does nothing there. The decide button does whatever the row it is on needs
doing to it: a list is opened, a switch is turned over, a button is pressed. While the list is open
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

The file is written out as text with a comment over each key rather than through a serialiser,
which would leave five bare keys and nothing beside them to say what any of it is for. There is
no copy of it in this repository to install: it is written by the thing that asks, and a second
hand-kept copy is one that goes stale.

Everything to do with building the midstage table, reaching an ending, or looking into a fault
is an argument to `orb.exe` instead — `--help` lists them — because a file is the wrong
place for something that is different every time it is run. The two passes over a replay are one
word each:

| | |
| --- | --- |
| `--collect` | propose boundaries over the whole replay, at 64 updates a frame, with nothing stopping and nobody at the keyboard |
| `--judge` | step between them at one update a frame and decide about each: the pass somebody watches |

and beside them `--tune`, `--replay`, `--speed=N`, `--log=quiet|normal|verbose`, `--pacing`,
`--compose=N`, `--self-check`, `--stress=N`, `--sent-keys`, and `--no-chapters`, `--no-memory`,
`--no-resume`, `--no-frame-loop` and `--no-hooks` for taking orb apart until a fault stops happening.

**`--sent-keys`** has the game read its keyboard the way it does when DirectInput gave it no device:
orb lets the device go — `Unacquire`, `Release`, the pointer cleared, which is what
`Supervisor::RegisterChain` itself does with one it cannot set up — and `Controller::GetInput` then
takes its `GetKeyboardState` branch. What that buys is a session another program can drive, which is
otherwise impossible: a device held `DISCL_EXCLUSIVE | DISCL_FOREGROUND` does not see keys injected
with `SendInput`, and every screen orb has a question over is the game's own screen answered on the
game's own keyboard. Nothing else about a run changes — the same code turns those keys into the same
buttons — and what it made possible is in [DONE.md](DONE.md): the whole of picking a run up again,
watched twice through the front end without a hand on the keyboard.

`--game-dir=PATH`, `--orb-dll=PATH`, `--config=PATH` and `--settings` are the launcher's own
four, and the only ones it does not hand on: each answers a question the DLL, being inside a game
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

**`--pacing`** writes what every frame that missed the cadence spent its turn on, at whatever
`--log` says rather than as a tier of it — see *What a late frame says*. Its own switch because
what the log writes is one of the reasons a frame misses its blank, so it goes with
`--log=quiet`: nothing in the file but the startup lines and this, and every write in the run is
one it made. It also turns on the two questions that cost a call each to answer and are asked
once a frame: whether the anchor is a blank, and which blank the last frame reached.

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
`crates/orb/src/game/th06/chapters.rs`; `tuning.txt` beside it is the same boundaries with what
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
presses them. The three stepping keys are a replay's alone — holding a run someone is playing
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

## Reaching the game's memory with no game there

Every read and write of the game's memory goes through `mem`, and a test can put an address
space of its own in front of the real one: regions at the game's own bases, holding bytes, with
`Th06` reading them through the same four functions it reads a running game through. So the
offsets, the structure walks and the patching are all exercised, rather than a second
implementation of `Game` standing in for them.

The space answers more than bytes, because that is what the code asks. A region can be
committed, reserved without being committed, or a guard page — which is how a pointer into a
structure the game has not built yet comes back as nothing rather than as a process that has
died — and it can be an image or an allocation, which is how a live COM object's vtable is told
from the stale pointer left in a block the allocator did not scrub. A restore that finds a region
gone commits it again, as it does when the game has handed a few megabytes back to the OS.

Installed per thread and taken off again when the test ends. The harness runs tests side by side
in one process and hands its threads out again, so a space in a static would be two tests
writing each other's game, and one left installed would be the next test reading a game it did
not lay out.

What this cannot catch is an offset that is wrong: the space is written from the same constants
the reads use, so a wrong one is wrong on both sides at once. Offsets are settled against the
real game, which is what the measurements in `DONE.md` are for. Everything built on top of them
is what the space is for.

In a build that is not a test the space does not exist and none of `mem`'s functions branch.

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

What is kept is the request rather than pixels — each quad's rectangle and colour, which texture it
went through, the clears, the viewports, in order. That answers the questions worth asking of a
frame: whether the mark covers the row the lives are counted in and leaves the rows either side
alone, whether a menu put its items where it says, which of the two flashes a boundary got. A
rasteriser would answer the same ones at the cost of being a rasteriser, and the one thing here
that is genuinely about pixels — the brush stroke's coverage — is baked from the picture by
`build.rs`, which is where it is checked.

## How it fits together

`orb.dll` is injected while the game is still suspended, so its `DllMain` runs before the
game's entry point and the memory hooks see the first allocation.

| | |
| --- | --- |
| `crates/launcher` | checks the exe, starts it suspended, injects `orb`, resumes it |
| `launcher/settings.rs` | the dialog that asks for the five settings before the game starts |
| `launcher/pad.rs` | reading a pad on the launcher's side, so that dialog answers to one |
| `crates/orb-config` | `orb.yaml` — read by both halves, written by the launcher — and the command line |
| `orb/lib.rs` | `DllMain`, the hooks orb installs, and the frame it runs in place of the game's |
| `orb/hook.rs` | trampoline and import-table hooks |
| `orb/memtrack.rs` | import hooks recording the heaps and reservations the game takes from the OS |
| `orb/snapshot.rs` | save and restore of `.data`, those regions, and the music |
| `orb/threads.rs` | finding and suspending the game's other threads, leaving the audio one alone |
| `orb/audio.rs` | the sound buffer and file position, which live outside the game's memory |
| `orb/frame.rs` | the frame loop's pacing and its measurements |
| `orb/chapter.rs` | where chapters begin, and which snapshots are kept |
| `orb/resume.rs` | the buttons a run has pressed, and the file that lets its chapter be played to again |
| `orb/retry_ui.rs` | the menu shown where the chapter was lost |
| `orb/lives_ui.rs` | the brush stroke over the game's count of lives, for a run that cannot lose one |
| `orb/build.rs`, `orb/brush.png` | that stroke, and the bake that turns the picture of it into coverage |
| `orb/mode_ui.rs` | the question put over the game's own menu: pointdevice or normal |
| `orb/resume_ui.rs` | the question after the character select: from where it stopped, or from the beginning |
| `orb/menu_ui.rs` | what those three have in common — the keys they read for themselves, and the list they draw |
| `orb/score.rs` | the fork of the game's score file, and the refusing of a clear run's write |
| `orb/mem/mod.rs` | the reads and writes of the game's memory, and what makes an address safe to read |
| `orb/mem/real.rs` | the page operations behind that — committing what a restore needs, and unprotecting it |
| `orb/mem/space.rs` | an address space laid out by hand, which is how a test has a game to read |
| `orb/tuning.rs` | building the midstage table |
| `orb/window.rs` | the window and how big it is, the letterbox, the status line |
| `orb/input.rs` | orb's own reading of the keyboard, for its keys rather than the game's |
| `orb/overlay.rs`, `text.rs`, `d3d8/mod.rs` | drawing over the game's frame |
| `orb/d3d8/recording.rs` | a device that keeps what it was asked to draw, so a test can say what is on the screen |
| `orb/log.rs`, `profile.rs` | the log and its levels, and where a frame's time went |
| `orb/crash.rs` | the handler that names the module and offset a fault happened at |
| `orb/game/mod.rs` | `Game` and `State`: everything above is written against these |
| `orb/game/th06/` | the addresses and offsets that make it 東方紅魔郷 |
| `orb/game/th06/image.rs` | those addresses laid out in a space, so the real `Th06` has something to read |

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
