//! Playback engine.
//!
//! HARD PROBLEM #2 (timing): OS sleep granularity is ~15ms on Windows, far too
//! coarse to reproduce input cadence. Each event's target time is anchored to a
//! per-loop `Instant` (so error never accumulates across the run) and we wait
//! with `spin_sleep`, which sleeps most of the interval then busy-spins the last
//! sub-millisecond for tight accuracy. Speed scaling simply divides each target
//! offset by the multiplier, so 2x halves every gap and 0.5x doubles it.

use crate::model::{Macro, MacroEvent};
use crate::simulate;
use crate::state::{AppState, Mode};
use rdev::{Button, Key};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub struct PlayParams {
    pub speed: f64,
    pub loops: u32,
    pub infinite: bool,
    pub include_keyboard: bool,
    pub include_mouse: bool,
    pub include_mouse_move: bool,
    pub infinite_cap: u32,
    /// Source resolution to scale mouse coordinates from (0 = live desktop).
    pub src_width: f64,
    pub src_height: f64,
}

#[derive(Clone, Serialize)]
struct Progress {
    loop_index: u32,
    loop_total: u32,
    infinite: bool,
    event_index: usize,
    event_total: usize,
    percent: f64,
}

#[derive(Clone, Serialize)]
struct Finished {
    stopped: bool,
}

fn included(ev: &MacroEvent, p: &PlayParams) -> bool {
    if ev.is_keyboard() {
        p.include_keyboard
    } else if ev.is_mouse_move() {
        p.include_mouse_move
    } else if ev.is_mouse_action() {
        p.include_mouse
    } else {
        true
    }
}

/// Runs the playback loop to completion (or until stopped). Intended to be
/// called on a dedicated thread; resets mode to Idle and emits completion.
pub fn run(state: Arc<AppState>, app: AppHandle, m: Macro, p: PlayParams) {
    let speed = p.speed.clamp(0.25, 5.0);
    let events: Vec<MacroEvent> = m.events.into_iter().filter(|e| included(e, &p)).collect();
    let total = events.len();

    let iterations = if p.infinite {
        p.infinite_cap.max(1)
    } else {
        p.loops.max(1)
    };

    let mut last_progress = Instant::now() - Duration::from_secs(1);
    let mut stopped = false;

    // Track keys/buttons we press so we can release any left held if playback
    // is stopped mid-press — otherwise a modifier could get stuck down.
    let mut held_keys: Vec<Key> = Vec::new();
    let mut held_buttons: Vec<Button> = Vec::new();

    'outer: for loop_i in 0..iterations {
        let base = Instant::now();
        for (idx, ev) in events.iter().enumerate() {
            if state.stop_playback.load(Ordering::SeqCst) {
                stopped = true;
                break 'outer;
            }
            let target =
                base + Duration::from_secs_f64((ev.t() as f64 / speed) / 1000.0);
            let now = Instant::now();
            if target > now {
                spin_sleep::sleep(target - now);
            }
            // Re-check after the wait so an emergency stop fires immediately.
            if state.stop_playback.load(Ordering::SeqCst) {
                stopped = true;
                break 'outer;
            }
            simulate::dispatch(ev, p.src_width, p.src_height);
            match ev {
                MacroEvent::KeyPress { key, .. } => {
                    if !held_keys.contains(key) {
                        held_keys.push(*key);
                    }
                }
                MacroEvent::KeyRelease { key, .. } => held_keys.retain(|k| k != key),
                MacroEvent::ButtonPress { button, .. } => {
                    if !held_buttons.contains(button) {
                        held_buttons.push(*button);
                    }
                }
                MacroEvent::ButtonRelease { button, .. } => {
                    held_buttons.retain(|b| b != button)
                }
                _ => {}
            }

            if last_progress.elapsed() >= Duration::from_millis(40) || idx + 1 == total {
                last_progress = Instant::now();
                let percent = if total == 0 {
                    100.0
                } else {
                    ((idx + 1) as f64 / total as f64) * 100.0
                };
                let _ = app.emit(
                    "playback_progress",
                    Progress {
                        loop_index: loop_i + 1,
                        loop_total: iterations,
                        infinite: p.infinite,
                        event_index: idx + 1,
                        event_total: total,
                        percent,
                    },
                );
            }
        }
    }

    // Safety net: release anything still held (stuck modifiers, buttons).
    for key in held_keys {
        simulate::dispatch(&MacroEvent::KeyRelease { t: 0, key }, 0.0, 0.0);
    }
    for button in held_buttons {
        simulate::dispatch(&MacroEvent::ButtonRelease { t: 0, button }, 0.0, 0.0);
    }

    state.set_mode(Mode::Idle);
    let _ = app.emit("playback_finished", Finished { stopped });
    let _ = app.emit("status_changed", Mode::Idle);
}
