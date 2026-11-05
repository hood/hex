//! The main-thread pump's pace: a frame while the HUD animates, else a slow
//! idle tick that whatever sends the pump something cuts short.

use std::sync::atomic::{AtomicBool, Ordering};

use objc2_foundation::{NSDate, NSTimer};

use super::{PUMP, WINDOW};

/// The pump's pace at rest. Every channel it drains wakes it at once
/// ([`wake_pump`]), so this only paces the menu refreshes, the updater's
/// restart, the shutdown check, and settings that apply on the next tick.
const IDLE_INTERVAL: f64 = 0.25;

/// Set while the pump's next tick is at most a frame away: the HUD animates,
/// or a wake is on its way.
static AWAKE: AtomicBool = AtomicBool::new(false);

/// Ticks the main-thread pump now rather than at its idle pace, from any
/// thread: whoever sends the pump something calls this after sending.
pub(crate) fn wake_pump() {
    if AWAKE.swap(true, Ordering::AcqRel) {
        return;
    }
    if let Some(window) = WINDOW.get() {
        window.update_if(|_, _| {
            PUMP.with(|slot| {
                if let Some(state) = slot.borrow().as_ref() {
                    state.timer.setFireDate(&NSDate::now());
                }
            });
            false
        });
    }
}

/// At the start of a tick, before it drains anything, so a send from then
/// on wakes the next one.
pub(super) fn turn_started() {
    AWAKE.store(false, Ordering::Release);
}

/// After a tick: the next comes a frame on while the HUD animates, else at
/// the idle pace unless something wakes the pump first.
pub(super) fn pace_pump(timer: &NSTimer, animating: bool) {
    if animating {
        AWAKE.store(true, Ordering::Release);
    } else {
        timer.setFireDate(&NSDate::dateWithTimeIntervalSinceNow(IDLE_INTERVAL));
    }
}
