//! Core actions shared by Tauri commands and global-hotkey handlers, so a
//! recording started by a keypress and one started by a button click run the
//! exact same path. All mode transitions funnel through here.

use crate::model::{Macro, MACRO_VERSION};
use crate::playback::{self, PlayParams};
use crate::simulate;
use crate::state::{AppState, Mode};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

fn emit_status(app: &AppHandle, mode: Mode) {
    let _ = app.emit("status_changed", mode);
}

pub fn start_recording(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    if !state.try_transition(Mode::Idle, Mode::Recording) {
        return Err("Mimic is busy (already recording or playing).".into());
    }
    let (interval, distance) = {
        let s = state.settings.lock().map_err(|_| "settings lock")?;
        (s.sample_interval_ms, s.sample_distance_px)
    };
    {
        let mut rec = state.recording.lock().map_err(|_| "recording lock")?;
        rec.start = Some(Instant::now());
        rec.events.clear();
        rec.interval_ms = interval;
        rec.distance_px = distance;
        rec.have_last = false;
        rec.last_move_t = 0;
        rec.last_x = 0.0;
        rec.last_y = 0.0;
        rec.pressed_keys.clear();
    }
    let _ = app.emit("recording_started", ());
    emit_status(app, Mode::Recording);
    Ok(())
}

pub fn stop_recording(app: &AppHandle, state: &Arc<AppState>) -> Result<Macro, String> {
    if state.mode() != Mode::Recording {
        return Err("Not recording.".into());
    }
    state.set_mode(Mode::Idle);

    let (events, duration_ms) = {
        let mut rec = state.recording.lock().map_err(|_| "recording lock")?;
        let dur = rec
            .start
            .take()
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0);
        (std::mem::take(&mut rec.events), dur)
    };

    // Stamp the recording resolution: the user's selected reference if set,
    // otherwise the live desktop. Playback scales mouse coords from this.
    let (screen_width, screen_height) = {
        let s = state.settings.lock().map_err(|_| "settings lock")?;
        if s.screen_width > 0 && s.screen_height > 0 {
            (s.screen_width, s.screen_height)
        } else {
            simulate::current_resolution()
        }
    };

    let name = format!("Recording {}", chrono::Local::now().format("%b %d %H:%M:%S"));
    let m = Macro {
        id: String::new(),
        name,
        version: MACRO_VERSION,
        created_at: chrono::Utc::now().to_rfc3339(),
        duration_ms,
        screen_width,
        screen_height,
        events,
    };

    let _ = app.emit("recording_stopped", m.clone());
    emit_status(app, Mode::Idle);
    Ok(m)
}

/// Resolve the resolution to scale a macro's mouse coordinates from: the
/// macro's own stamp, else the user's selected reference, else the live desktop.
pub fn source_resolution(m: &Macro, state: &Arc<AppState>) -> (f64, f64) {
    if m.screen_width > 0 && m.screen_height > 0 {
        return (m.screen_width as f64, m.screen_height as f64);
    }
    if let Ok(s) = state.settings.lock() {
        if s.screen_width > 0 && s.screen_height > 0 {
            return (s.screen_width as f64, s.screen_height as f64);
        }
    }
    let (w, h) = simulate::current_resolution();
    (w as f64, h as f64)
}

pub fn start_playback(
    app: &AppHandle,
    state: &Arc<AppState>,
    m: Macro,
    params: PlayParams,
) -> Result<(), String> {
    if !state.try_transition(Mode::Idle, Mode::Playing) {
        return Err("Mimic is busy (already recording or playing).".into());
    }
    state.stop_playback.store(false, Ordering::SeqCst);
    if let Ok(mut last) = state.last_macro_id.lock() {
        if !m.id.is_empty() {
            *last = Some(m.id.clone());
        }
    }
    emit_status(app, Mode::Playing);
    let _ = app.emit("playback_started", ());

    let st = state.clone();
    let ah = app.clone();
    std::thread::Builder::new()
        .name("mimic-playback".into())
        .spawn(move || playback::run(st, ah, m, params))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Emergency stop: halt playback immediately, and abort any recording too.
pub fn emergency_stop(app: &AppHandle, state: &Arc<AppState>) {
    state.stop_playback.store(true, Ordering::SeqCst);
    if state.mode() == Mode::Recording {
        let _ = stop_recording(app, state);
    }
}

pub fn stop_playback(state: &Arc<AppState>) {
    state.stop_playback.store(true, Ordering::SeqCst);
}
