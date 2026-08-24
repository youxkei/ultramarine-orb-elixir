//! **The stepping keys over a replay of th07: one chapter at a time, inside the stage.**
//!
//! What `stepping_inside_a_stage.rs` asks of th06, over this game's own chapters. Both keys go through
//! `Chapters::previous_start` and `next_start`, whose whole job is that a step lands **strictly** either side of
//! where the game is — so a press moves rather than staying put, and a frame or two past a boundary steps back
//! onto it.
//!
//! **Which chapters they land on is this game's own fight**, th07's midstage table being empty until a tuning
//! run fills it: the stage's own start and the three a fight has, read off the manager's boss slot and the timer
//! of an attack. So a step here is a restore of a snapshot of *this* game, and the frames it lands on are the
//! frames this file's own driving of that fight put the chapters at.
//!
//! Over a replay, because that is what a judging pass steps through: `--chapter-stepping` with
//! `--during-replay`, which is the launch that has nobody to answer a retry menu and every chapter kept all the
//! same.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, UPDATE, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;

/// One press of a stepping key with nothing else held, and the frame it left the game on.
///
/// # Panics
/// Where the step wrote no line saying where it held the game, which is a press that did nothing.
fn steps(game: &Fake, key: u8) -> u32 {
    let held = game.log().written();
    game.press(key);
    let said = game.log().lines().split_off(held).join("\n  ");
    assert!(
        said.contains("step: held"),
        "the press of {key:#04x} held the game nowhere:\n  {said}",
    );
    game.state().stage_frames
}

/// The launch a judging pass is: the stepping keys on over a replay, and nobody there to answer a retry menu.
///
/// Shared with the e2e test at the foot of this file, because the frame a replay is *asked for* is a frame of
/// the same pass: the two differ in what the game has built by then and not in how orb was launched.
fn judging(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_stepping = true;
            config.during_replay = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.streams_its_song(0);
    game
}

#[test]
fn a_step_inside_a_stage_of_th07_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = judging("th07-stepping-inside");
        let log = game.log();
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        let the_stages_own_start = game.state().stage_frames;

        // The fight, chapter by chapter, with the frame each began on kept: those are the frames a step has to
        // land on, and every one of them is where this game put a chapter rather than a number written here.
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        let the_midboss = game.state().stage_frames;

        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
        let the_card = game.state().stage_frames;

        game.frames(APART);
        game.the_card_is_over();
        game.the_attack_changes();
        game.frames_until("the chapter after the card", APART, || {
            log.said("chapter 4")
        });
        let after_the_card = game.state().stage_frames;

        // Held a little way into the last of them: a step back from inside a chapter goes to that chapter's own
        // start, which is what makes a press the way onto the boundary just passed.
        game.frames(APART / 2);
        let past = game.state().stage_frames;
        game.press(HOLD);
        assert_eq!(
            game.state().stage_frames,
            past,
            "the hold let the game past the frame it was aimed at",
        );

        // Back through them, one press each. Strictly either side, so the third press does not stay on the frame
        // the second landed on.
        assert_eq!(steps(&game, BACK), after_the_card, "one press back");
        assert_eq!(steps(&game, BACK), the_card, "two presses back");
        assert_eq!(steps(&game, BACK), the_midboss, "three presses back");
        assert_eq!(
            steps(&game, BACK),
            the_stages_own_start,
            "four presses back",
        );

        // And the press after that, which has nowhere left to go inside the stage: the stage's own start is what
        // lies before its first boundary, and the restore has already arrived there.
        game.press(BACK);
        assert!(
            log.said(&format!(
                "step: held at the stage's start (script {the_stages_own_start})"
            )),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            the_stages_own_start,
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same three: what a step forward runs to is the next start where the stage
        // has one.
        assert_eq!(steps(&game, NEXT), the_midboss, "one press forward");
        assert_eq!(steps(&game, NEXT), the_card, "two presses forward");
        assert_eq!(steps(&game, NEXT), after_the_card, "three presses forward");
    });
}

/// **A step back over a stage longer than ten minutes lands on the boundary and not on the stop that bounds
/// the run forward.**
///
/// A step back restores the stage's start and runs forward to the frame asked for, and that run has a stop on
/// it for a target it can never reach. The stop was ten minutes of game time, written down as longer than any
/// stage — and the Extra is not: a replay of it fought its boss to frame 42127, so every step back to a chapter
/// past the ten minutes ran out the stop and held the game wherever that fell. Which reads as a step that
/// worked: a pass stepping around 式神「橙」 came back into the middle of it, at the same frame every time, and
/// the log said `held … 36000 update(s) run` the way it says a landing.
#[test]
fn a_step_back_over_a_stage_longer_than_the_stop_lands_on_the_chapter_it_asks_for() {
    in_its_own_process(|| {
        let game = judging("th07-stepping-a-long-stage");
        let log = game.log();
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // Past the stop the run forward used to have, without playing the frames: what the stop bounds is the
        // *replay* from the stage's start, and the stage's own clock is what it counts.
        const PAST_THE_OLD_STOP: u32 = 60 * 60 * 10 + 100;
        game.image().has_been_in_the_stage_for(PAST_THE_OLD_STOP);

        game.the_boss_arrives();
        game.frames_until("the fight's own chapter", APART, || log.said("chapter 2"));
        let the_boss = game.state().stage_frames;
        assert!(
            the_boss > 60 * 60 * 10,
            "the fight's chapter is inside the old stop, so this proves nothing: {the_boss}",
        );

        game.frames(APART);
        game.the_attack_changes();
        game.frames_until("the attack's own chapter", APART, || log.said("chapter 3"));

        assert_eq!(
            steps(&game, BACK),
            the_boss,
            "the step back stopped short of the chapter it asked for",
        );
    });
}

/// **A chapter taken while the game's own sound is mid-ask calls nothing on the buffer.**
///
/// A step back whose chapter was taken under another track takes the music down and asks the game for the
/// track that chapter had — and that ask is a command in the player's own queue, walked by the frame's own
/// sound. The step forward after it is `run_to`, which walks the calc chain and nothing else, so the ask
/// stands for every frame of the replay while the memory copy has put the player's stream field back to the
/// object the take-down deleted. Reading the sound there is reading a released COM object: what its methods
/// do is not defined, and orb must not call them.
///
/// A launch of th07 is why. Stepping back to stage 2's own start wrote
/// `restore: the track has changed since this snapshot; taking the music down` and
/// `music: restarting slot 0, the chapter's own track being th07_04.wav`, and the replay after it faulted
/// taking a chapter — `0x7ae3ec1c`, `Snapshot::update+0x143c`, reading `0x60` with `edi` zero. That
/// instruction is `self.untracked = Vec::new()` and `edi` is `self`: the report's `esp=0x001aefac` is four
/// bytes below where that frame's own prologue puts it, so every `[esp+X]` of the function read one slot
/// along and `self`, spilled once, came back zero. One of the calls on the dead buffer had returned with the
/// stack short, and what it corrupted was orb's own frame.
#[test]
fn a_chapter_of_th07_taken_while_the_sound_is_mid_ask_calls_nothing_on_the_buffer() {
    in_its_own_process(|| {
        // The table being built as well as stepped through, because of what the boundary has to be: a
        // boundary of the pass's own list is one the replay reaches again off the script clock alone, where
        // the fight a laid-out game is driven through by hand is not put back by a restore.
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-stepping-mid-ask",
            Work::wandering(WORK_US),
            |config| {
                config.chapter_stepping = true;
                config.chapter_tuning = true;
                config.during_replay = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        game.streams_its_song(0);
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // A wave, and then nothing to shoot at: the gap the pass proposes a boundary a second into.
        game.image().enemies_alive(A_WAVE);
        game.frames(THE_WAVE_LASTS);
        let wave_ended_at = game.state().script_frames;
        game.image().enemies_alive(0);
        let proposed_at = wave_ended_at + A_SECOND_IN;
        game.frames_until("the boundary the gap is", 600, || {
            log.said(&format!("(script {proposed_at}): tl {proposed_at}"))
        });
        game.frames(APART);

        // The fight's own track playing, which is what leaves the chapter behind it taken under a track that
        // is gone: the step back onto that boundary is then the restore that takes the music down and asks
        // the game for the track the chapter it puts back had.
        game.plays_another_song();
        steps(&game, BACK);
        assert!(
            log.said("taking the music down"),
            "the step back did not take the music down, so nothing here is about a released buffer:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            orb_sim::Sound::calls_on_a_released_buffer(),
            0,
            "the buffer the game had taken down was called, by orb reading the sound of a chapter \
             taken mid-ask or by the game's own play walking a stream the copy resurrected:\n  {}",
            log.lines().join("\n  ")
        );

        // **And the song back, which is the half a count of calls cannot say.** A restore that leaves the
        // sound in a state the game cannot work with is one whose track never starts — and every arm of the
        // game's own play that meets a stream it does not own gives up quietly, so the track starting is what
        // says the restore left something playable behind.
        let started = game.image().music_started();
        assert!(
            started
                .last()
                .is_some_and(|track| track.contains("th07_02")),
            "the chapter's own track is not the last one the game started: {started:?}\n  {}",
            log.lines().join("\n  ")
        );
        // And the stream that track is playing through made over the file every track is in, which is the name
        // that call takes: a track's name there is what the game answers -1 to.
        assert_eq!(
            game.image().made_a_stream_over().as_deref(),
            Some("thbgm.dat"),
            "the stream was not made over the game's own music file:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// How many enemies the wave before the gap has: anything above nothing, the detector asking only whether
/// there is something to shoot at.
const A_WAVE: i32 = 4;
/// How long that wave lasts, in frames: past the shortest a chapter may be, so the gap after it is one the
/// proposal's own floor on the frames since the last chapter is already clear of.
const THE_WAVE_LASTS: u32 = 180;
/// The frames of nothing to shoot at a proposal is made a second into, which is `tuning::ENEMY_GAP_FRAMES` —
/// written out here for the reason every other number an e2e test holds is: a test that read it off the code
/// under test would agree with it whatever either said.
const A_SECOND_IN: i32 = 60;

/// A pad on the host, declared the way every e2e test about one declares it: buttons enough for the game's
/// own mapping to name any of them, and an axis over a whole 16-bit travel.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// Where a hat points, as winmm gives it: hundredths of a degree clockwise from up, and `0xffff` for a hat
/// that is not being pushed at all.
const HAT_LEFT: u32 = 27000;
const HAT_CENTRED: u32 = 0xffff;

/// **The judging hand on a pad**, which is the hand this pass is driven with: somebody watching a stage go
/// by has a controller in their hands and the keyboard across the room.
///
/// The hat where the stepping keys are, since the four directions are the four things a boundary is judged
/// with: right and left step between them, up and down judge the one the game is standing on. What the
/// buttons are is `Th07`'s own mapping — shoot, bomb, focus, menu, up, down, left, right, skip — so a press
/// here is the press the game itself would read.
#[test]
fn the_judging_hand_of_th07_steps_on_a_pad() {
    in_its_own_process(|| {
        let game = judging("th07-judging-on-a-pad");
        let log = game.log();
        game.sim()
            .joystick()
            .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.frames_until_a_thread("the pad named", || log.said("joystick: winmm 0 is"));
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // A chapter to step back to, which is the fight arriving.
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the fight's own chapter", APART, || log.said("chapter 2"));
        let the_fight = game.state().stage_frames;
        game.frames(APART);

        // The hat pushed left and let go, which is the press the left key is.
        let from = log.written();
        game.sim().joystick().pushes_the_hat(HAT_LEFT);
        game.frames_until_a_thread("the hat read", || log.written() > from);
        game.frames(2);
        game.sim().joystick().pushes_the_hat(HAT_CENTRED);
        game.frames(2);

        assert!(
            log.said_since(from, "step: held"),
            "the hat pushed left held the pass nowhere:\n  {}",
            log.lines().split_off(from).join("\n  "),
        );
        assert_eq!(
            game.state().stage_frames,
            the_fight,
            "the step back on the pad did not land on the chapter the fight began",
        );
    });
}

/// The frames the pass is given between the ask and the build, over which a hold would still be a hold.
const BEFORE_THE_BUILD: u32 = 2;

/// **The frame a replay was asked for and nothing of it built is not a stage that has ended.**
///
/// The end of a stage holds the pass and keeps it held, that being the one place carrying on loses the stage
/// — and what says a stage ended is the run being in no gameplay scene. th07 reads as exactly that before
/// its first stage: `Th07::read_state` counts a run's *build* as being in the run, the supervisor's scene
/// lagging that build by the rest of an update, so the frame its front end answered a stage select on is a
/// run with nothing playing in it yet and no stage anywhere.
///
/// **And a hold there is one nothing lifts**: the update the hold stops is the update that would build the
/// stage, so the pass waits for a stage the game cannot reach. Measured on th07 — `--judge` over a replay,
/// stage 1 chosen, `step: stage 0 has ended` in the log with stage 0 being no stage at all, the game frozen
/// on the stage select and closed by hand.
#[test]
fn the_frame_a_replay_of_th07_was_asked_for_is_not_a_stage_that_has_ended() {
    in_its_own_process(|| {
        let game = judging("th07-a-replay-asked-for");
        let log = game.log();

        // What the front end's own update leaves behind on that frame: a replay being watched, the run's block
        // allocated with none of its numbers written into it yet, and the scene still the front end's with a run
        // the one asked for.
        game.image().is_watching_a_replay();
        game.image().has_a_runs_numbers();
        game.image().a_run_has_been_chosen();

        game.forget_asked();
        game.frames(BEFORE_THE_BUILD);
        assert!(
            !log.said("has ended"),
            "the frame a replay was asked for was taken for the end of a stage:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.asked().contains(&UPDATE),
            "the game was given no update while a replay was being asked for, which is the update that would \
             build the stage: {:?}",
            game.asked(),
        );

        // And the build that update makes, which is where a stage comes from here: the pass goes on into the
        // stage and takes its first chapter, rather than waiting outside for one it stopped from being built.
        game.builds_a_stage();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
    });
}

/// When the fight this stage ends with is won, and how long the stage runs on after that: the second is
/// what th07's own stages do, which is 281 to 496 frames of a boss going down, its items and what is
/// said over them.
const FIGHT_WON: u32 = 600;
const STAGE_ENDS: u32 = FIGHT_WON + 300;

/// **A step forward stays in the stage it was made in.**
///
/// Leaving one is the across key's own, and what it takes is the game being asked to start the replay
/// somewhere else — see `moving_between_a_replays_stages_of_th07.rs`. A plain step forward runs the game
/// on to the next chapter of *this* stage, so what has to stop it where the stage has no next chapter is
/// the stage's own end.
///
/// **th06 stops it by leaving the gameplay scene and th07 never does**: its supervisor does a whole
/// transition inside one update, so the run is in the scene a run is played in on every frame of one, and
/// a loop watching that scene runs straight through into the stage after. A launch found it: `next` at the
/// end of a fight ran on and the pass came to rest in stage 2, with the stage it was judging torn down
/// behind it and nothing of its chapters left to go back into.
#[test]
fn a_step_forward_inside_a_stage_of_th07_does_not_leave_it() {
    in_its_own_process(|| {
        let game = judging("th07-a-step-forward-past-the-stage");
        let log = game.log();
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        // The fight the stage ends with, under the boss's own song, and the stage running on past the win
        // the way this game's do. Past the shortest a chapter may be first, so the fight's own arrival is a
        // chapter rather than one folded into the stage's start.
        game.frames(APART);
        game.the_stages_own_boss_arrives();
        game.frames_until("the boss's chapter", APART, || log.said("chapter 2"));
        game.the_fight_is_won_at(FIGHT_WON);
        game.stages_last(STAGE_ENDS);

        // A step forward with no chapter ahead of it: what it aims at is a chapter this stage never begins,
        // so what has to stop it is the fight being won — and if nothing does, the stage ends under it.
        let from = log.written();
        game.press(NEXT);
        assert_eq!(
            game.state().stage,
            0,
            "the step forward left the stage it was made in:\n  {}",
            log.lines().split_off(from).join("\n  "),
        );
        assert!(
            log.said_since(from, "step: held in stage 1"),
            "the step forward did not come to rest in the stage it was made in:\n  {}",
            log.lines().split_off(from).join("\n  "),
        );
    });
}

/// The wall's own line, which says which stage it is about and how to leave it.
const WALL: &str = "step: stage 1 has ended; the across key with next or back leaves it";

/// The frames a signal is given to reach the wall: the update it is read on, and the frame whose stepping is
/// decided on what that update left.
const TO_THE_WALL: u32 = 2;

/// **The fight a stage ends with being won is the end of that stage, and the pass is held inside it.**
///
/// th07 spends no frame outside the gameplay scene at a stage's end — its supervisor does the whole
/// transition inside one update, which is what
/// [`Image::the_next_stage_is_being_asked_for`](orb_core::game::th07::Image::the_next_stage_is_being_asked_for)
/// was read out of the exe for — so the run being in no gameplay scene, which is the end of a stage on th06,
/// never comes round here at all: a `--judge` pass over a Lunatic replay played all six stages through without
/// stopping once.
///
/// What it stops on instead is the stage's own fight being won, and that is **inside** the stage, which is the
/// whole point of a wall: every chapter of the stage is still a snapshot orb holds, so a step back goes into the
/// fight. Once the next stage has loaded nothing of the one before it can be gone back into — what a snapshot of
/// it names was released by that load — and the only way back is asking the game to play the stage again.
///
/// **Which frame that is, the game decides.** A defeat under a dialogue is acted on when the dialogue lifts,
/// `Chapters::observe` holding what the guard turned away, so a boss beaten mid-conversation walls after it. Each
/// of th07's stages runs on for 281 to 496 frames after its boss goes down, and what is in them was not
/// measured: nothing here has to know, and nothing here says.
#[test]
fn the_fight_a_stage_of_th07_ends_with_being_won_holds_the_pass_inside_the_stage() {
    in_its_own_process(|| {
        let game = judging("th07-the-fight-won");
        let log = game.log();
        game.watches_a_replay();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // The fight the stage ends with, which is the boss arriving under the boss's own song — the music being
        // what tells that fight from a midboss — and the chapter its first attack begins.
        game.frames(APART);
        game.the_stages_own_boss_arrives();
        game.frames_until("the boss's chapter", APART, || log.said("chapter 2"));
        let the_boss = game.state().stage_frames;
        game.frames(APART);

        // And the fight won, which is the manager's slot emptied. No chapter begins at a defeat — a chapter whose
        // first act is that the fight is already won has nothing to retry — so the wall is the whole of what this
        // frame does.
        let from = log.written();
        game.the_boss_is_beaten();
        game.frames(TO_THE_WALL);
        assert!(
            log.said_since(from, WALL),
            "the fight being won did not hold the pass:\n  {}",
            log.lines().join("\n  ")
        );

        // Held there and kept held: the hold key is a toggle everywhere else in the stage and moves nothing here.
        let at_the_wall = game.state();
        game.press(HOLD);
        game.frames(30);
        assert_eq!(
            game.state(),
            at_the_wall,
            "the pass was let past the end of the stage",
        );

        // And a step back goes into the fight, which is what a wall inside the stage is for.
        assert_eq!(
            steps(&game, BACK),
            the_boss,
            "a step back at the wall did not go into the fight the stage ended with",
        );
    });
}
