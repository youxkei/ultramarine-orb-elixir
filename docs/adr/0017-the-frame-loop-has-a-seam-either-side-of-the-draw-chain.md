# 17. The frame loop has a seam either side of the draw chain

**Status:** accepted and built. `Game::begin_drawing` and `Game::end_drawing` are what `runtime::render`
calls inside the scene it draws in, `Th06` answers nothing to both, `Th07::hooks` asks for `render` at
0x4346e0, and `crates/orb-e2e/src/th07.rs` drives orb's own loop over a laid-out 妖々夢 whose draw chain
queues a quad the way a real one does.

It answers the question [0004](0004-th07-is-a-second-game-chosen-at-the-attach.md) and
[issue 2](https://github.com/youxkei/ultramarine-orb-elixir/issues/2) parked: *whether `prepare_frame` is
where those belong or whether 妖々夢 wants a different loop.* Neither. They belong either side of the draw
chain, and 妖々夢 wants the loop it has.

It stands on [0002](0002-the-frame-loops-two-calls-into-the-game-are-addresses.md), whose rule the three
calls this adds are handed over under: code is the one thing an address space laid out by hand cannot
hold.

## Context

**妖々夢's frame is the same shape as 紅魔郷's, and it is not the same frame.** `GameWindow::Render` at
0x4346e0, `__thiscall` on the window object at 0x575c20, reached from the message pump at 0x4341f2 on the
frames `TestCooperativeLevel` answers `D3D_OK` for. 0x434708 to 0x434a33 is one draw, one update, the
sounds and the present, and around them a loop deciding — from the frame-skip setting at 0x575a8b and the
timing at 0x575c34 — whether a pass draws at all and whether to take another before presenting. 紅魔郷's is
0x4206e0 and has the same counter at `[this+0x10]`, the same skip setting, the same two chain exits mapped
to 1 and 2. That is the shape, and it is why replacing it looked like the same job.

What differs is inside the scene:

| | 紅魔郷, 0x4206e0 | 妖々夢, 0x4346e0 |
| --- | --- | --- |
| before the draw chain | `BeginScene` at 0x4207d1 and nothing else | `BeginScene` at 0x434728, the queue of quads emptied at 0x434734, the fog flag written at 0x434739, the fog put out at 0x434748 |
| the draw chain | 0x4207dc | 0x43474d |
| after it | `EndScene` at 0x4207ef, `SetTexture(0, NULL)` after it | the queue drawn at 0x43475d, `SetTexture(0, NULL)` at 0x434774, `EndScene` at 0x434788, the queue drawn again at 0x43478e where it does nothing |

**The queue is the one that killed the game.** The object at `[0x4b9e44]` is 0x17e560 bytes off the
allocator at 0x434119, and three of its own fields are the whole of it: a count at 0x2e530, a write
pointer at 0x17e534 and a draw pointer at 0x17e538, over a buffer that is 0x2e534 up to 0x17e534 of
itself. 0x44f580 empties it — count zero, write pointer at the buffer, draw pointer to match. The append
at 0x44f690 copies six vertices of 0x1c bytes to where the write pointer says, adds 0xa8, and counts one.
0x44f5c0 hands the count's worth of triangle pairs to `DrawPrimitiveUP` from the draw pointer and puts the
count back to zero.

orb's first loop over that frame called neither the empty nor the draw. So the first quad the drawing
queued went to where nothing had pointed the write pointer, which is `crash: code 0xc0000005 at 0x0044f6aa
in th07.exe+0x4f6aa`, `writing 0x00000000` — the `rep movsd` at the top of the append. Every address that
loop used was right. See 0004's *What the two launches said*.

**Two things 0004 read out of that pair of launches were wrong, and this is where they are corrected.**

- `perf: … update=8us/frame worst=1084us calls=1200 draw=3us/frame worst=341us calls=570` over 600 frames
  was read there as 妖々夢's timing loop *running the update twice a frame*. It is not: `runtime.rs`
  records the update phase twice per update-hook call, once before the chain call and once at the end of
  the hook, so 1200 is 600 hook calls. What the line says is 600 logic frames against 570 drawings in the
  same reporting period, and which of 妖々夢's two skip paths the missing 30 are is not in it.
- The two calls on `[0x4b9e44]` at 0x43472e and 0x434757 were read as *a render-state block*. They are the
  queue of quads emptied and drawn.

Neither changes what the launch established, which is that the loop and not the two patches took the game
down.

**And what the frame writes at 0x434739 is not a word nothing reads.** `mov DWORD PTR ds:0x575c0c,0xff`
looks like a global with two writers and no reader anywhere in the exe. 0x575c0c is 0x575950 + 0x2bc,
which is the fog flag — the field 0x43a1bd and 0x43a207 compare before they touch the device. So the frame
writes it and then calls 0x43a207, and what that buys is the `SetRenderState(D3DRS_FOGENABLE, FALSE)`
reaching the device on every frame drawn rather than only on the frame after a stage's background drawing
turned fog on. **The e2e test found that**, and it could not have been found by reading: the write and the
call are two lines of disassembly apart and the arithmetic between their addresses is what joins them.

## Decision

**What the game's own frame does around its own draw chain is two methods of `Game`, and orb's loop calls
them inside the scene.**

- **`begin_drawing(device)` after `BeginScene` and before the draw chain, `end_drawing(device)` after the
  chain and before `EndScene`.** 紅魔郷 answers nothing to both, which is measured of its frame and not a
  gap in it. 妖々夢 empties the queue, writes the fog flag and puts the fog out in the first, and draws the
  queue in the second.

- **Not `prepare_frame`, which is a different place in the frame.** That one runs before the wait and
  before the update, outside any scene, and it is the whole-output viewport and the clear a game's options
  ask for. A queue emptied there would be emptied after the previous frame's drawing had already filled
  it; a queue drawn there would be drawn outside the scene it was queued in.

- **Not a different loop for 妖々夢 either.** The loop's order — the viewport, the wait, the update, the
  sounds, the drawing, the hold, the present — is what removes the frame of input lag, and 妖々夢's frame
  is that same list in the game's own order with a skip loop around it. orb takes the skipping out here as
  it takes 紅魔郷's out.

- **The three calls are `handed_over!` slots.** They are calls out of the seam into the game, so a game
  laid out by hand has to be able to answer them, which is 0002's rule and the third time it has been
  applied. The macro moves from `th06/mod.rs` to `game/mod.rs`: two games have such calls now, and a macro
  apiece is one of them drifting.

- **What orb calls the two queue functions on is `[0x4b9e44]`, read per call.** The object is on the heap,
  so there is nothing at that address until the game has been through its own startup — unlike the sound
  player and the chain, which are statics `Call` can carry.

- **The second of the frame's two calls to draw the queue is not made.** 0x43478e is outside the scene and
  the count the first one left at zero is what it returns on, so it is the game's own no-op on every frame
  it drew.

## Consequences

**What a launch in 妖々夢 now gets.** Its window at the right shape, and orb's own frame: the update before
the draw, the sounds where the game hands its own over, the drawing inside one scene, and the frame held
for the blank before the one it is aimed at. Which is a frame of input lag gone and the cadence
[0006](0006-the-frame-loop-waits-on-a-high-resolution-timer.md) and
[0011](0011-the-frame-is-held-for-the-blank-before-the-one-it-is-aimed-at.md) describe. Nothing drawn
still — there is no `font.ttf` beside that exe — and nothing of a run.

**What says it works, and what cannot.** `crates/orb-e2e/src/th07.rs` holds the e2e tests: the order of the
four calls a frame makes, the queue emptied and drawn once a frame with the write pointer back where it
started, the fog taken off to the device on *every* frame, and the game's own score file left alone. Each
is red without this decision, and the queue's is red the way the real game was red — the laid-out game's
draw chain queues a quad through the same field, so a loop that skips the empty writes 168 bytes to address
zero and the sim says so. What none of them can say is that the addresses are right: a laid-out 妖々夢 is
written from the same constants `Th07` reads, so a wrong one is wrong on both sides at once.

**What is left open, and the line that decides it.** orb's overlay is drawn inside the draw hook, so the
queue drawn by `end_drawing` is drawn *over* it. Nothing today — 妖々夢 is the game with a queue and it has
no font to build an overlay from — and the day it has one, `end_drawing` moves into the draw hook ahead of
`after_draw`, which is where the game's own frame has it relative to everything drawn inside the chain.
The comment beside the call in `runtime::render` says so.

**What it costs.** Two calls per frame that do nothing at all in 紅魔郷, and a `&dyn Game` call for each:
the loop already makes five of that kind. And every game after this one has two more methods to answer,
which is the price of the seam being the frame's *order* rather than a list of addresses.

**What it rules out.**

- **Composing the frame from `FrameCalls`.** Three more `Call`s there and three more `Originals` fields
  would put 妖々夢's frame in `runtime::render`, where 紅魔郷 would have to answer for a queue it has not
  got. The seam exists so that the loop is one frame and each game says what its own frame is.
- **Reading the game's own skip setting.** orb draws every frame it updates. 妖々夢's frame at 0x575a8b and
  紅魔郷's at 0x6c6e4b are the same setting, and orb's loop has ignored 紅魔郷's since it was written.
- **Deciding which of the frame's writes matter.** The fog flag looked like a write nothing reads, and
  leaving it out would have cost `D3DRS_FOGENABLE` on every frame but one. What the frame does, orb's
  frame does.
- **`[this+8]` and `[this+0x10]`, the frame's own two fields.** The first is what makes the game's frame
  return at once — 紅魔郷 has it too, at 0x6c6bdc, and orb's loop has always ignored it, that being what
  keeps a run going while the window is behind. The second is the skip counter, which nothing outside the
  frame reads. Neither is touched.

## Plan

1. **Built.** The laid-out 妖々夢 grows the queue: the block at the length the game's own has, the three
   functions over it in `th07/image.rs` where the block is, and a draw chain that queues a quad.
   `Launch::new` is handed `orbs_own_loop`, so a game that declines `render` is one the e2e tests do not
   drive a loop for — which 妖々夢 was, and which is what let the tests be red for the right reason twice.
2. **Built.** `Th07::hooks` asks for `render`, and the e2e tests over the queue fault where the game
   faulted.
3. **Built.** The seam, the loop's two calls, `Th06` answering nothing, and `Th07` composing the frame.
