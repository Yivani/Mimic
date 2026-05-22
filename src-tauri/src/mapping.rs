//! Bridges three key representations:
//!   * W3C `KeyboardEvent.code` strings (what the settings UI captures),
//!   * `rdev::Key` (what the capture listener sees — needed to filter the
//!     hotkey out of recordings),
//!   * `tauri_plugin_global_shortcut::Shortcut` (what we register globally).

use crate::model::Hotkey;
use rdev::Key;
use std::str::FromStr;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

/// Build a global-shortcut `Shortcut` from our `Hotkey`. Returns `None` if the
/// code string isn't a recognized W3C code.
pub fn to_shortcut(hk: &Hotkey) -> Option<Shortcut> {
    let code = Code::from_str(&hk.code).ok()?;
    let mut mods = Modifiers::empty();
    if hk.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if hk.shift {
        mods |= Modifiers::SHIFT;
    }
    if hk.alt {
        mods |= Modifiers::ALT;
    }
    if hk.meta {
        mods |= Modifiers::META;
    }
    Some(Shortcut::new(
        if mods.is_empty() { None } else { Some(mods) },
        code,
    ))
}

/// Map a W3C code to the `rdev::Key` the listener will report, so we can drop
/// the hotkey's trigger key from recordings. Returns `None` for codes we don't
/// need to filter (modifiers are intentionally not filtered).
pub fn code_to_rdev_key(code: &str) -> Option<Key> {
    let k = match code {
        "Escape" => Key::Escape,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "Backquote" => Key::BackQuote,
        "Minus" => Key::Minus,
        "Equal" => Key::Equal,
        "Backspace" => Key::Backspace,
        "Tab" => Key::Tab,
        "BracketLeft" => Key::LeftBracket,
        "BracketRight" => Key::RightBracket,
        "Enter" | "NumpadEnter" => Key::Return,
        "Semicolon" => Key::SemiColon,
        "Quote" => Key::Quote,
        "Comma" => Key::Comma,
        "Period" => Key::Dot,
        "Slash" => Key::Slash,
        "Space" => Key::Space,
        "Insert" => Key::Insert,
        "Delete" => Key::Delete,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "ArrowUp" => Key::UpArrow,
        "ArrowDown" => Key::DownArrow,
        "ArrowLeft" => Key::LeftArrow,
        "ArrowRight" => Key::RightArrow,
        "Pause" => Key::Pause,
        "ScrollLock" => Key::ScrollLock,
        "PrintScreen" => Key::PrintScreen,
        "NumLock" => Key::NumLock,
        "CapsLock" => Key::CapsLock,
        "Digit0" => Key::Num0,
        "Digit1" => Key::Num1,
        "Digit2" => Key::Num2,
        "Digit3" => Key::Num3,
        "Digit4" => Key::Num4,
        "Digit5" => Key::Num5,
        "Digit6" => Key::Num6,
        "Digit7" => Key::Num7,
        "Digit8" => Key::Num8,
        "Digit9" => Key::Num9,
        "KeyA" => Key::KeyA,
        "KeyB" => Key::KeyB,
        "KeyC" => Key::KeyC,
        "KeyD" => Key::KeyD,
        "KeyE" => Key::KeyE,
        "KeyF" => Key::KeyF,
        "KeyG" => Key::KeyG,
        "KeyH" => Key::KeyH,
        "KeyI" => Key::KeyI,
        "KeyJ" => Key::KeyJ,
        "KeyK" => Key::KeyK,
        "KeyL" => Key::KeyL,
        "KeyM" => Key::KeyM,
        "KeyN" => Key::KeyN,
        "KeyO" => Key::KeyO,
        "KeyP" => Key::KeyP,
        "KeyQ" => Key::KeyQ,
        "KeyR" => Key::KeyR,
        "KeyS" => Key::KeyS,
        "KeyT" => Key::KeyT,
        "KeyU" => Key::KeyU,
        "KeyV" => Key::KeyV,
        "KeyW" => Key::KeyW,
        "KeyX" => Key::KeyX,
        "KeyY" => Key::KeyY,
        "KeyZ" => Key::KeyZ,
        _ => return None,
    };
    Some(k)
}
