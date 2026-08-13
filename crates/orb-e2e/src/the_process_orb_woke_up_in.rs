//! **Which game a process is, and the process that is none of them.**
//!
//! Every launch begins here. orb reads addresses out of absolute memory, so a process it has no addresses
//! for is one it must leave exactly as it found it — and the whole of what settles that is the exe's own
//! file name, nothing else of a game being readable before the game it is has been settled. See
//! [docs/adr/0004](../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).
//!
//! **The launch that does not happen is the e2e test here.** Every other file in this crate is a game
//! playing the game's part; this one is the moment before there is a game at all, which is the one thing a
//! laid-out game cannot be: a `Fake` is 紅魔郷, and there is no such thing as laying out a program orb knows
//! nothing about. So what it drives is the table, with a simulated Windows under it for the log to go to —
//! the same shape `orb-sim`'s own `log_writes` has, and for the same reason.
//!
//! The other half, the answer for a game orb *does* know, is asserted by every launch in this crate: each
//! fake asks the table which game the name it is running under is, rather than naming one. See
//! `fake::th06::the_game_this_is`.

use std::path::PathBuf;
use std::sync::Arc;

use crate::fake::in_its_own_process;
use orb_core::game::{KNOWN, found};
use orb_sim::Sim;

/// A name no entry holds, and one that is nothing like a game's: what orb must do inside it is nothing.
const NOT_A_GAME: &str = "notepad.exe";

#[test]
fn a_process_no_entry_names_is_refused_and_every_build_orb_knows_is_named() {
    in_its_own_process(|| {
        let sim = Arc::new(Sim::new());
        let _installed = sim.enter();
        sim.set_host_exe(PathBuf::from("windows").join(NOT_A_GAME));
        orb_core::log::open();

        // Nothing, which is what stops every launch below this line: `orb::attach` returns on it, and
        // nothing of the host has been touched by the time it does.
        assert!(
            found(NOT_A_GAME).is_none(),
            "orb thinks {NOT_A_GAME} is a game it has addresses for",
        );

        // And the refusal names every game *and* every build, because a build that is not the one orb read
        // is as likely to be another game's exe under a name orb reads as this game's own next release. The
        // list is asserted entry by entry out of the table, so a game added without its version being said
        // fails here.
        let said = sim.log().lines().join("\n");
        assert!(
            said.contains(&format!("game: nothing orb knows is called {NOT_A_GAME}")),
            "the refusal does not name the process it was in:\n{said}",
        );
        for known in KNOWN {
            assert!(
                said.contains(&format!("{} {}", known.exe, known.builds_named())),
                "the refusal does not name {} {}:\n{said}",
                known.exe,
                known.builds_named(),
            );
        }
        assert!(
            said.contains("orb is doing nothing this run"),
            "the refusal does not say what it costs:\n{said}",
        );

        orb_core::log::close();
    });
}

/// And a name that is a game's, whichever way it is spelled: an exe copied out in capitals is the same
/// game, these being Windows file names.
///
/// Every entry, because an entry no name finds is a game orb carries addresses for and never uses — and
/// the ASCII of a name is what folds, a name in kanji having no case to fold.
#[test]
fn every_game_in_the_table_is_found_by_the_name_its_exe_has() {
    in_its_own_process(|| {
        let sim = Arc::new(Sim::new());
        let _installed = sim.enter();
        orb_core::log::open();

        for known in KNOWN {
            let by_name = found(known.exe).expect("an entry found by its own exe");
            assert_eq!(by_name.builds_named(), known.builds_named());
            assert!(
                found(&known.exe.to_ascii_uppercase()).is_some(),
                "{} is not found under the name a copied exe often has",
                known.exe,
            );
            // And the line it wrote says which build every address orb has for it was read off, which is
            // the one thing a report of a defect has to carry: two builds of one game read differently.
            assert!(
                sim.log()
                    .said(&format!("was read off {}", known.builds_named())),
                "nothing said which build {}'s addresses came from:\n  {}",
                known.exe,
                sim.log().lines().join("\n  ")
            );
        }

        orb_core::log::close();
    });
}
