//! Global hotkey registration and dispatch via tauri-plugin-global-shortcut.
//!
//! The emergency-stop hotkey is registered the same way as the others, so it
//! keeps working while a macro is playing: the OS delivers the keypress to our
//! handler thread independently of the playback thread, which then flips the
//! stop flag the playback loop polls between every event.

use crate::controller;
use crate::mapping;
use crate::model::Settings;
use crate::playback::PlayParams;
use crate::state::{AppState, Mode};
use crate::storage;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[derive(Default)]
pub struct HotkeyRegistry {
    pub record: Mutex<Option<Shortcut>>,
    pub play: Mutex<Option<Shortcut>>,
    pub stop: Mutex<Option<Shortcut>>,
}

/// (Re)apply hotkeys from the given settings: refresh the recording filter set
/// and re-register the three global shortcuts.
pub fn apply(app: &AppHandle, state: &Arc<AppState>, settings: &Settings) {
    if let Ok(mut f) = state.filter_keys.lock() {
        f.clear();
        for hk in [
            &settings.hotkeys.record,
            &settings.hotkeys.play,
            &settings.hotkeys.stop,
        ] {
            if let Some(k) = mapping::code_to_rdev_key(&hk.code) {
                f.push(k);
            }
        }
    }

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let r = mapping::to_shortcut(&settings.hotkeys.record);
    let p = mapping::to_shortcut(&settings.hotkeys.play);
    let s = mapping::to_shortcut(&settings.hotkeys.stop);
    if let Some(sc) = r {
        let _ = gs.register(sc);
    }
    if let Some(sc) = p {
        let _ = gs.register(sc);
    }
    if let Some(sc) = s {
        let _ = gs.register(sc);
    }

    let reg = app.state::<HotkeyRegistry>();
    let set = |slot: &Mutex<Option<Shortcut>>, val: Option<Shortcut>| {
        if let Ok(mut g) = slot.lock() {
            *g = val;
        }
    };
    set(&reg.record, r);
    set(&reg.play, p);
    set(&reg.stop, s);
}

/// Called from the plugin handler on a key press.
pub fn handle(app: &AppHandle, shortcut: &Shortcut) {
    let state = app.state::<Arc<AppState>>().inner().clone();
    if state
        .hotkeys_suspended
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return; // e.g. user is capturing a new hotkey in Settings
    }
    let reg = app.state::<HotkeyRegistry>();

    let matches = |slot: &Mutex<Option<Shortcut>>| -> bool {
        slot.lock()
            .ok()
            .and_then(|g| *g)
            .map(|s| &s == shortcut)
            .unwrap_or(false)
    };

    if matches(&reg.stop) {
        controller::emergency_stop(app, &state);
    } else if matches(&reg.record) {
        match state.mode() {
            Mode::Recording => {
                let _ = controller::stop_recording(app, &state);
            }
            Mode::Idle => {
                let _ = controller::start_recording(app, &state);
            }
            Mode::Playing => {}
        }
    } else if matches(&reg.play) {
        match state.mode() {
            Mode::Playing => controller::stop_playback(&state),
            Mode::Idle => {
                let _ = play_selected(app, &state);
            }
            Mode::Recording => {}
        }
    }
}

/// Play the last-selected macro (or the most recent one) with default settings.
fn play_selected(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    let id = {
        let last = state.last_macro_id.lock().ok().and_then(|g| g.clone());
        match last {
            Some(id) => id,
            None => storage::list_macros(app)?
                .first()
                .map(|m| m.id.clone())
                .ok_or("No macros available to play.")?,
        }
    };
    let m = storage::load_macro(app, &id)?;
    let (speed, cap) = {
        let s = state.settings.lock().map_err(|_| "settings lock")?;
        (s.default_speed, s.infinite_loop_cap)
    };
    let (src_width, src_height) = controller::source_resolution(&m, state);
    controller::start_playback(
        app,
        state,
        m,
        PlayParams {
            speed,
            loops: 1,
            infinite: false,
            include_keyboard: true,
            include_mouse: true,
            include_mouse_move: true,
            include_gamepad: true,
            infinite_cap: cap,
            src_width,
            src_height,
        },
    )
}
