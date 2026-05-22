//! Global input capture.
//!
//! HARD PROBLEM #1 (rdev lifecycle): `rdev::listen` is blocking and has no stop
//! API. Rather than fight it, we start ONE listener for the whole app lifetime
//! on a dedicated thread and gate behavior on the current `Mode`. Benefits:
//!   * No need to ever stop/restart the listener (which rdev can't do cleanly).
//!   * Events we *simulate* during playback are naturally ignored, because the
//!     mode is `Playing`, not `Recording` — no feedback loop.
//!
//! On Windows the low-level hook needs a message pump on its thread; rdev runs
//! one internally, which is exactly why it blocks — so a dedicated thread is
//! the correct home for it. macOS needs the event tap on the main thread
//! (documented limitation in the README).

use crate::model::MacroEvent;
use crate::state::{AppState, Mode};
use rdev::{Event, EventType};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
struct EventCaptured {
    count: usize,
    t: u64,
    label: String,
}

#[derive(Clone, Serialize)]
struct MousePos {
    x: i32,
    y: i32,
}

/// Spawn the always-on capture listener. Call once during setup.
pub fn spawn_listener(state: Arc<AppState>, app: AppHandle) {
    std::thread::Builder::new()
        .name("mimic-capture".into())
        .spawn(move || {
            let mut last_pos_emit = Instant::now() - Duration::from_secs(1);

            let cb = move |event: Event| {
                // Track cursor position for the live readout regardless of mode.
                if let EventType::MouseMove { x, y } = event.event_type {
                    state.cur_x.store(x as i32, Ordering::Relaxed);
                    state.cur_y.store(y as i32, Ordering::Relaxed);
                    if last_pos_emit.elapsed() >= Duration::from_millis(50) {
                        last_pos_emit = Instant::now();
                        let _ = app.emit(
                            "mouse_position",
                            MousePos {
                                x: x as i32,
                                y: y as i32,
                            },
                        );
                    }
                }

                if state.mode() != Mode::Recording {
                    return;
                }

                let mut rec = match state.recording.lock() {
                    Ok(r) => r,
                    Err(_) => return,
                };
                let start = match rec.start {
                    Some(s) => s,
                    None => return,
                };
                let t = start.elapsed().as_millis() as u64;

                let macro_event = match event.event_type {
                    EventType::KeyPress(key) => {
                        if state.is_filtered(&key) {
                            return;
                        }
                        Some(MacroEvent::KeyPress { t, key })
                    }
                    EventType::KeyRelease(key) => {
                        if state.is_filtered(&key) {
                            return;
                        }
                        Some(MacroEvent::KeyRelease { t, key })
                    }
                    EventType::ButtonPress(button) => Some(MacroEvent::ButtonPress { t, button }),
                    EventType::ButtonRelease(button) => {
                        Some(MacroEvent::ButtonRelease { t, button })
                    }
                    EventType::Wheel { delta_x, delta_y } => Some(MacroEvent::Wheel {
                        t,
                        dx: delta_x,
                        dy: delta_y,
                    }),
                    EventType::MouseMove { x, y } => {
                        // Throttle: keep a sample only once enough time AND
                        // distance have accumulated. last_* only update on a
                        // kept sample, so slow drift still accumulates distance.
                        let keep = if !rec.have_last {
                            true
                        } else {
                            let dt = t.saturating_sub(rec.last_move_t);
                            let dx = x - rec.last_x;
                            let dy = y - rec.last_y;
                            let dist2 = dx * dx + dy * dy;
                            dt >= rec.interval_ms && dist2 >= rec.distance_px * rec.distance_px
                        };
                        if !keep {
                            return;
                        }
                        rec.have_last = true;
                        rec.last_move_t = t;
                        rec.last_x = x;
                        rec.last_y = y;
                        Some(MacroEvent::MouseMove { t, x, y })
                    }
                };

                if let Some(ev) = macro_event {
                    let label = ev.label();
                    rec.events.push(ev);
                    let count = rec.events.len();
                    drop(rec); // release before emitting
                    let _ = app.emit("event_captured", EventCaptured { count, t, label });
                }
            };

            if let Err(e) = rdev::listen(cb) {
                eprintln!("[mimic] capture listener error: {:?}", e);
            }
        })
        .expect("failed to spawn capture thread");
}
