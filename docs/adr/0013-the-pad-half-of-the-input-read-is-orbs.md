# 13. The pad half of the input read is orb's, and every pad the machine has goes through it

**Status:** accepted and built. `Controller::GetControllerInput` (0x41cfc0) is hooked and **not called
through**: `orb_core::runtime::get_controller_input` answers with `Game::pad_word`, which reads the device
the game's own enumeration found and the pad `orb_core::joystick` last saw pushed, and turns both into the
bits of the word the game acts on. The hook goes in on every launch rather than at `verbose`, the write of a
device's caps into the game's `JOYCAPSA` at 0x69d760 is gone with the reader that needed it, and the count
behind a held shot button is kept in the game's own `g_IsEigthFrameOfHeldInput` at 0x69d8f4.

**It moves the ground under one bullet of
[0008](0008-the-fake-game-copies-the-game-orb-is-injected-into.md)'s rejected list** — *modelling the pad's
merge into the input word* — whose premise was that a pad moves the player in a real launch and answers only
orb's own menus in the fake. It moves the player in both now, because orb is what moves it. What that bullet
refused stays refused: the fake reproduces no pad arithmetic, and there is one implementation rather than
two.

## Context

**紅魔郷 holds exactly one game controller and settles which at startup.** `g_Supervisor.controller`
(0x6c6d2c) is written in one place in the whole exe — the enumeration's callback at 0x423da0, which calls
`CreateDevice` only while the pointer is still null and returns `DIENUM_STOP` the moment it succeeds — and
the one other device it will read is winmm's joystick 0. So a second pad, a pad plugged in after the game
started, and, on the machine orb was measured on, the pad itself — sitting in XInput's second slot while
winmm's joystick 0 holds a phantom with no buttons and no axes — are pads the game can do nothing with.
This is a one-player game, so every one of them is that one player's.

**Reading them and adding to the word the game's read handed back gets most of the way there**, and that is
what was built first: the buttons through the game's own mapping, the axes through its own arithmetic, and
the hat behind `dpad_moves`. It leaves one thing behind that cannot be added from outside.

**`g_IsEigthFrameOfHeldInput` (0x69d8f4) is that thing.** Where the mapping puts focus and shoot on one
button, `GetControllerInput` sets no focus bit from a button at all: it counts the frames that one button has
been held — up to 16 at 0x41d06e and 0x41d3a4, the player held still from the eighth at 0x41d08e and
0x41d3c3, the count brought back down by 8 a frame while above 8 and to nothing below it at 0x41d0ac and
0x41d3e3 — and that is what holding the shot button to move slowly is. The count is fed by the game's read
of the device it holds. A pad it has no device for never reaches it, and orb cannot raise it from the outside
either: the game's read runs first in the frame and has already brought the count down by the time orb's
hook over `GetInput` is entered, so a frame orb added is a frame the game takes off again and the eighth
never arrives.

## The decision

**Hook `GetControllerInput` and do not call through.** What a pad does to a frame is one answer, for every
pad, in one place: `Game::pad_word`.

Which means orb answers for all of what that function did, and all of it was read off the exe first:

- the device the game holds — `Poll` (vtable +0x64), `GetDeviceState` (+0x24) into a 0x110-byte
  `DIJOYSTATE2`, and the keyboard's word unchanged where either fails;
- winmm's joystick 0 and every other socket, which `orb_core::joystick` samples off the game's thread;
- the nine mapped buttons, by the numbering `SetButtonFromDirectInputJoystate` (0x41d580) and
  `SetButtonFromControllerInputs` (0x41d600) share — shoot 0x1, bomb 0x2, focus 0x4, menu 0x8, the four
  directions 0x10 to 0x80, skip 0x100, and a mapping entry below zero naming no button;
- focus both ways: from its own button, and otherwise from that count;
- the axes by the two rules the two devices get — the one the game holds against `cfg.padXAxis` and
  `cfg.padYAxis` in the ±1000 `Supervisor::RegisterChain` gave its axes, and every other pad against the
  travel its own caps report;
- and the hat, which the game reads on no device at all, behind `dpad_moves`.

**The hook goes in on every launch.** It was installed at `verbose` and only to time the read; a launch
without it now would be a launch where no pad does anything.

## What follows

- **The write of a device's caps into `g_JoyCaps` is deleted.** Every read of that struct's fields was in
  the winmm branch of the function orb now answers — 0x41d18b to 0x41d22f, and the only other mention of the
  address in the exe is the `joyGetDevCapsA` call that fills it — so it has no reader left. Each pad carries
  the bounds its own axes are measured against, in `Reading`.
- **The frame stops paying twice for the device it holds.** `dpad_moves` used to read it a second time for
  the hat, the game's own read having thrown the hat away; one read answers everything now.
- **The count lives in the game's field and not a static of orb's**, so a chapter restored rewinds it with
  the rest of `.data` — which is what the game's own code would have done with it.
- **`Game::joystick_calibration` is gone from the seam**, with th06's `Some(G_JOY_CAPS)` and th07's `None`.
- **The risk moved onto the path that already worked.** A pad the game holds used to go through the game's
  own arithmetic and now goes through orb's, so every bullet above wants an e2e test over that device and
  not only over the pads the game cannot see. `orb-e2e/src/the_pad_half_of_the_input_read.rs` is that file,
  and `a_pad_the_game_has_no_device_for.rs` is the other half — which pads are read, against what reading
  one produces.

## What was weighed and refused

- **Leaving the read the game's and adding to its answer**, which is what the first pass built. It works for
  everything except the count above, and a pad that shoots without ever holding the player still is a pad
  that does not play this game. Keeping it would also have kept the second read of the device, the write
  into `g_JoyCaps`, and two implementations of where an axis becomes a direction — one of them the game's,
  reachable only through a device orb cannot write.
- **Writing the game's count from outside its read.** Measured against the order of the frame: the game's
  own read has already run when orb's hook is entered, so the write is undone before it can be read.
- **Copying the game's acquire loop.** Its own read asks `Acquire` up to 400 times while the answer is
  `DIERR_INPUTLOST`, at 0x41d2a5 to 0x41d2f0. orb asks once and gives the frame up, which is what
  `Th06::controller_state` already did for the menus and the reason is written there: the next frame is a
  sixtieth of a second away, and a frame spent in a loop is the thing orb exists to protect.
- **Having the fake reproduce the arithmetic** so that the e2e tests could hold orb's answer against the
  game's. That is [0008](0008-the-fake-game-copies-the-game-orb-is-injected-into.md)'s refused bullet and it
  stays refused: two copies of one rule is the thing being removed, not the thing to add. What the fake
  hands over is the device and its state; what the tests assert is the word orb produced from it, against the
  exe the addresses were read off.
