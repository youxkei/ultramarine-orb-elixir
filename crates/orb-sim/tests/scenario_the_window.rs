//! **The window orb makes: the size asked for, the monitor's real pixels, and the black beside the game.**
//!
//! Every scenario here is a stub: `#[ignore]`d, and `todo!()` where the assertion goes. What each one
//! holds is the measurement it has to reproduce, taken on this machine — a 3840x2160 monitor that reads
//! as 2560x1440 to a process that has not asked otherwise.
//!
//! What it takes to un-stub them: `orb::window` is driven through `CreateWindowExA`'s own arguments and
//! `SetProcessDPIAware`, neither of which is behind the seam. The arithmetic already has tests —
//! `window.rs`'s `a_window_is_centred_on_its_monitor` and the four beside it — and what these add is the
//! part that arithmetic cannot reach: what the host reports before and after the process says it is DPI
//! aware, and the frame the host puts round a client of the size asked for.

/// The client is exactly the size asked for, and the frame this machine adds is outside it.
///
/// Measured: `screen: 1280x720` came out as `screen: 1280x720 — window at 1277,700 sized 1286x760,
/// client 1280x720`. The frame is the **6x40** between the two, and the window is centred on a monitor
/// read as 3840x2160. Still `client 1280x720` when the device was created.
#[test]
#[ignore = "the frame a host puts round a window is not behind the seam"]
fn the_client_is_the_size_asked_for_and_the_frame_is_outside_it() {
    todo!("declare a host whose window frame is 6x40 and assert the client comes out 1280x720")
}

/// Display scaling is ignored, which is what makes every size the monitor's real pixels.
///
/// Measured on the same monitor: it reads as **2560x1440** before `SetProcessDPIAware` and **3840x2160**
/// after. Without the call every size would have been scaled behind the game's back — a 1280x720 client
/// asked for on a 3840x2160 panel would have been laid out against 2560x1440.
#[test]
#[ignore = "SetProcessDPIAware is not behind the seam"]
fn the_monitor_is_its_real_pixels_once_the_process_says_it_is_dpi_aware() {
    todo!("read the monitor before and after the call and assert 2560x1440 then 3840x2160")
}

/// Borderless fullscreen keeps the aspect ratio and blacks the rest, with no frame to remove.
///
/// Measured: `screen: fullscreen — window at 0,0 sized 3840x2160, client 3840x2160` on that monitor.
/// The game's own `CreateWindowExA` arguments are rewritten, so there is no frame to take off and
/// nothing flashes first.
#[test]
#[ignore = "the CreateWindowExA argument rewrite is not behind the seam"]
fn borderless_fullscreen_fills_the_monitor_with_no_frame_and_no_flash() {
    todo!("rewrite the arguments for a fullscreen launch and assert 0,0 and 3840x2160 both ways")
}

/// A chosen size is centred exactly, and the status line gets the black either side of a 4:3 game.
///
/// Measured: `screen: 2560x1440 — window at 637,340 sized 2566x1480, client 2560x1440`, centred
/// exactly — **(3840−2566)/2 = 637** and **(2160−1480)/2 = 340**. The game is letterboxed to
/// **1920x1440** inside that client, **320 pixels either side**, and the `no black to write in` line a
/// 4:3 client produces never appeared.
#[test]
#[ignore = "the frame a host puts round a window is not behind the seam"]
fn a_chosen_size_is_centred_and_leaves_the_status_line_its_black() {
    todo!("assert 637,340 out of the arithmetic and 320 pixels of black either side of 1920x1440")
}
