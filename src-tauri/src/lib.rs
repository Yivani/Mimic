mod capture;
mod controller;
mod hotkeys;
mod mapping;
mod model;
mod playback;
mod simulate;
mod state;
mod storage;

use hotkeys::HotkeyRegistry;
use model::{Macro, MacroMeta, Settings};
use playback::PlayParams;
use state::{AppState, Mode};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

type Cmd<T> = Result<T, String>;

// ---------------- recording ----------------

#[tauri::command]
fn start_recording(app: AppHandle, state: State<Arc<AppState>>) -> Cmd<()> {
    controller::start_recording(&app, state.inner())
}

#[tauri::command]
fn stop_recording(app: AppHandle, state: State<Arc<AppState>>) -> Cmd<Macro> {
    controller::stop_recording(&app, state.inner())
}

// ---------------- playback ----------------

#[allow(clippy::too_many_arguments)]
#[tauri::command]
fn start_playback(
    app: AppHandle,
    state: State<Arc<AppState>>,
    macro_id: String,
    speed: f64,
    loops: u32,
    infinite: bool,
    include_keyboard: bool,
    include_mouse: bool,
    include_mouse_move: bool,
) -> Cmd<()> {
    let m = storage::load_macro(&app, &macro_id)?;
    let cap = state
        .settings
        .lock()
        .map(|s| s.infinite_loop_cap)
        .unwrap_or(1000);
    let (src_width, src_height) = controller::source_resolution(&m, state.inner());
    controller::start_playback(
        &app,
        state.inner(),
        m,
        PlayParams {
            speed,
            loops,
            infinite,
            include_keyboard,
            include_mouse,
            include_mouse_move,
            infinite_cap: cap,
            src_width,
            src_height,
        },
    )
}

#[tauri::command]
fn stop_playback(state: State<Arc<AppState>>) -> Cmd<()> {
    controller::stop_playback(state.inner());
    Ok(())
}

// ---------------- macro library ----------------

#[tauri::command]
fn save_macro(app: AppHandle, mut macro_data: Macro) -> Cmd<MacroMeta> {
    storage::save_macro(&app, &mut macro_data)
}

#[tauri::command]
fn load_macro(app: AppHandle, id: String) -> Cmd<Macro> {
    storage::load_macro(&app, &id)
}

#[tauri::command]
fn list_macros(app: AppHandle) -> Cmd<Vec<MacroMeta>> {
    storage::list_macros(&app)
}

#[tauri::command]
fn rename_macro(app: AppHandle, id: String, name: String) -> Cmd<()> {
    storage::rename_macro(&app, &id, &name)
}

#[tauri::command]
fn duplicate_macro(app: AppHandle, id: String) -> Cmd<MacroMeta> {
    storage::duplicate_macro(&app, &id)
}

#[tauri::command]
fn delete_macro(app: AppHandle, id: String) -> Cmd<()> {
    storage::delete_macro(&app, &id)
}

#[tauri::command]
fn import_macro(app: AppHandle, path: String) -> Cmd<MacroMeta> {
    storage::import_macro(&app, &path)
}

#[tauri::command]
fn export_macro(app: AppHandle, id: String, path: String) -> Cmd<()> {
    storage::export_macro(&app, &id, &path)
}

// ---------------- settings & status ----------------

#[tauri::command]
fn get_settings(state: State<Arc<AppState>>) -> Settings {
    state
        .settings
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn set_settings(app: AppHandle, state: State<Arc<AppState>>, settings: Settings) -> Cmd<()> {
    storage::save_settings(&app, &settings)?;
    if let Ok(mut s) = state.settings.lock() {
        *s = settings.clone();
    }
    hotkeys::apply(&app, state.inner(), &settings);
    Ok(())
}

#[tauri::command]
fn get_status(state: State<Arc<AppState>>) -> Mode {
    state.mode()
}

/// While capturing a new hotkey, fully unregister global shortcuts so the key
/// (including function keys, which the OS otherwise swallows for registered
/// shortcuts) reaches the capture field. Re-registers from settings when done.
#[tauri::command]
fn suspend_hotkeys(app: AppHandle, state: State<Arc<AppState>>, suspended: bool) {
    state
        .hotkeys_suspended
        .store(suspended, std::sync::atomic::Ordering::SeqCst);
    if suspended {
        let _ = app.global_shortcut().unregister_all();
    } else {
        let settings = state.settings.lock().map(|s| s.clone()).unwrap_or_default();
        hotkeys::apply(&app, state.inner(), &settings);
    }
}

#[tauri::command]
fn get_mouse_position(state: State<Arc<AppState>>) -> [i32; 2] {
    [
        state.cur_x.load(std::sync::atomic::Ordering::Relaxed),
        state.cur_y.load(std::sync::atomic::Ordering::Relaxed),
    ]
}

#[tauri::command]
fn set_selected_macro(state: State<Arc<AppState>>, id: Option<String>) -> Cmd<()> {
    if let Ok(mut last) = state.last_macro_id.lock() {
        *last = id;
    }
    Ok(())
}

/// Current virtual-desktop resolution in physical pixels (0,0 if unavailable).
#[tauri::command]
fn get_screen_resolution() -> [u32; 2] {
    let (w, h) = simulate::current_resolution();
    [w, h]
}

/// Platform-specific warnings to surface in the UI (permissions, Wayland, etc).
#[tauri::command]
fn platform_warnings() -> Vec<String> {
    let mut out = Vec::new();
    #[cfg(target_os = "macos")]
    {
        out.push(
            "macOS: grant Accessibility and Input Monitoring permissions in System Settings \u{2192} Privacy & Security, then restart Mimic.".into(),
        );
    }
    #[cfg(target_os = "linux")]
    {
        if std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
        {
            out.push(
                "Linux/Wayland detected: global input capture requires X11. Log into an X11/Xorg session for Mimic to work.".into(),
            );
        }
    }
    #[cfg(target_os = "windows")]
    {
        out.push(
            "Windows: to control apps running as administrator, launch Mimic as administrator too.".into(),
        );
    }
    out
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        hotkeys::handle(app, shortcut);
                    }
                })
                .build(),
        )
        .manage(HotkeyRegistry::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = storage::load_settings(&handle);
            let state = Arc::new(AppState::new(settings.clone()));
            app.manage(state.clone());

            capture::spawn_listener(state.clone(), handle.clone());
            hotkeys::apply(&handle, &state, &settings);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            start_playback,
            stop_playback,
            save_macro,
            load_macro,
            list_macros,
            rename_macro,
            duplicate_macro,
            delete_macro,
            import_macro,
            export_macro,
            get_settings,
            set_settings,
            get_status,
            suspend_hotkeys,
            get_mouse_position,
            set_selected_macro,
            get_screen_resolution,
            platform_warnings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mimic");
}
