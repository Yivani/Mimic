//! JSON persistence for macros and settings under the OS app-data directory.
//!
//! Layout:
//!   <app_data_dir>/settings.json
//!   <app_data_dir>/macros/<uuid>.json

use crate::model::{Macro, MacroMeta, Settings, MACRO_VERSION};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("no app data dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn macros_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = data_dir(app)?.join("macros");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn macro_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    // Guard against path traversal: ids are uuids, but be defensive.
    if id.is_empty() || id.contains(['/', '\\', '.']) {
        return Err("invalid macro id".into());
    }
    Ok(macros_dir(app)?.join(format!("{id}.json")))
}

// ---- settings ----

pub fn load_settings(app: &AppHandle) -> Settings {
    let path = match data_dir(app) {
        Ok(d) => d.join("settings.json"),
        Err(_) => return Settings::default(),
    };
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = data_dir(app)?.join("settings.json");
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ---- macros ----

pub fn save_macro(app: &AppHandle, m: &mut Macro) -> Result<MacroMeta, String> {
    if m.id.is_empty() {
        m.id = Uuid::new_v4().to_string();
    }
    if m.version == 0 {
        m.version = MACRO_VERSION;
    }
    let path = macro_path(app, &m.id)?;
    let json = serde_json::to_string_pretty(m).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(MacroMeta::from(&*m))
}

pub fn load_macro(app: &AppHandle, id: &str) -> Result<Macro, String> {
    let path = macro_path(app, id)?;
    let s = fs::read_to_string(&path).map_err(|e| format!("macro not found: {e}"))?;
    let mut m: Macro = serde_json::from_str(&s).map_err(|e| format!("corrupt macro: {e}"))?;
    if m.id.is_empty() {
        m.id = id.to_string();
    }
    Ok(m)
}

pub fn list_macros(app: &AppHandle) -> Result<Vec<MacroMeta>, String> {
    let dir = macros_dir(app)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(mut m) = serde_json::from_str::<Macro>(&s) {
                if m.id.is_empty() {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        m.id = stem.to_string();
                    }
                }
                out.push(MacroMeta::from(&m));
            }
        }
    }
    // Newest first.
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

pub fn rename_macro(app: &AppHandle, id: &str, name: &str) -> Result<(), String> {
    let mut m = load_macro(app, id)?;
    m.name = name.to_string();
    save_macro(app, &mut m).map(|_| ())
}

pub fn duplicate_macro(app: &AppHandle, id: &str) -> Result<MacroMeta, String> {
    let mut m = load_macro(app, id)?;
    m.id = String::new(); // force a new id
    m.name = format!("{} (copy)", m.name);
    m.created_at = chrono::Utc::now().to_rfc3339();
    save_macro(app, &mut m)
}

pub fn delete_macro(app: &AppHandle, id: &str) -> Result<(), String> {
    let path = macro_path(app, id)?;
    fs::remove_file(&path).map_err(|e| e.to_string())
}

pub fn import_macro(app: &AppHandle, src: &str) -> Result<MacroMeta, String> {
    let s = fs::read_to_string(src).map_err(|e| format!("cannot read file: {e}"))?;
    let mut m: Macro = serde_json::from_str(&s).map_err(|e| format!("invalid macro file: {e}"))?;
    m.id = String::new(); // assign a fresh id on import
    if m.created_at.is_empty() {
        m.created_at = chrono::Utc::now().to_rfc3339();
    }
    save_macro(app, &mut m)
}

pub fn export_macro(app: &AppHandle, id: &str, dest: &str) -> Result<(), String> {
    let m = load_macro(app, id)?;
    let json = serde_json::to_string_pretty(&m).map_err(|e| e.to_string())?;
    fs::write(dest, json).map_err(|e| e.to_string())
}
