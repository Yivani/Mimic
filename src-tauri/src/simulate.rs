//! Input simulation for playback.
//!
//! Keyboard / mouse-button / wheel events go through `rdev::simulate`.
//! Mouse *movement* is special-cased on Windows: rdev normalizes coordinates
//! against the primary monitor only (`SM_CXSCREEN`), which breaks on
//! multi-monitor / mixed-DPI setups. We instead emit a native `SendInput` with
//! `MOUSEEVENTF_VIRTUALDESK`, normalizing across the whole virtual desktop so
//! recorded absolute coordinates land where intended. Capture reports physical
//! pixels and the process is per-monitor DPI aware, so this round-trips.

use crate::model::MacroEvent;
use rdev::{simulate, EventType, Key};

/// `src_w`/`src_h` are the resolution the macro was recorded at, used to scale
/// mouse movement to the current screen (0 = use the live desktop size).
pub fn dispatch(ev: &MacroEvent, src_w: f64, src_h: f64) {
    match ev {
        MacroEvent::KeyPress { key, .. } => press_key(*key, false),
        MacroEvent::KeyRelease { key, .. } => press_key(*key, true),
        MacroEvent::ButtonPress { button, .. } => {
            let _ = simulate(&EventType::ButtonPress(*button));
        }
        MacroEvent::ButtonRelease { button, .. } => {
            let _ = simulate(&EventType::ButtonRelease(*button));
        }
        MacroEvent::Wheel { dx, dy, .. } => {
            let _ = simulate(&EventType::Wheel {
                delta_x: *dx,
                delta_y: *dy,
            });
        }
        MacroEvent::MouseMove { x, y, .. } => move_mouse(*x, *y, src_w, src_h),
    }
}

/// Press or release a key. On Windows we inject the hardware **scancode**
/// (KEYEVENTF_SCANCODE) rather than a virtual-key code, because most games read
/// scancodes via DirectInput/Raw Input and ignore virtual-key injection — this
/// is what makes held keys and chords actually register in games. Falls back to
/// rdev for keys we don't have a scancode for, and on non-Windows.
fn press_key(key: Key, up: bool) {
    #[cfg(windows)]
    {
        if send_scancode(key, up) {
            return;
        }
    }
    let _ = simulate(&if up {
        EventType::KeyRelease(key)
    } else {
        EventType::KeyPress(key)
    });
}

#[cfg(windows)]
fn send_scancode(key: Key, up: bool) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
    };

    let (scan, extended) = match scancode(key) {
        Some(v) => v,
        None => return false,
    };

    unsafe {
        let mut flags = KEYEVENTF_SCANCODE;
        if extended {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        if up {
            flags |= KEYEVENTF_KEYUP;
        }
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
    true
}

/// Map an rdev key to its Set-1 make code and whether it's an extended (E0) key.
#[cfg(windows)]
fn scancode(key: Key) -> Option<(u16, bool)> {
    let v = match key {
        Key::Escape => (0x01, false),
        Key::Num1 => (0x02, false),
        Key::Num2 => (0x03, false),
        Key::Num3 => (0x04, false),
        Key::Num4 => (0x05, false),
        Key::Num5 => (0x06, false),
        Key::Num6 => (0x07, false),
        Key::Num7 => (0x08, false),
        Key::Num8 => (0x09, false),
        Key::Num9 => (0x0A, false),
        Key::Num0 => (0x0B, false),
        Key::Minus => (0x0C, false),
        Key::Equal => (0x0D, false),
        Key::Backspace => (0x0E, false),
        Key::Tab => (0x0F, false),
        Key::KeyQ => (0x10, false),
        Key::KeyW => (0x11, false),
        Key::KeyE => (0x12, false),
        Key::KeyR => (0x13, false),
        Key::KeyT => (0x14, false),
        Key::KeyY => (0x15, false),
        Key::KeyU => (0x16, false),
        Key::KeyI => (0x17, false),
        Key::KeyO => (0x18, false),
        Key::KeyP => (0x19, false),
        Key::LeftBracket => (0x1A, false),
        Key::RightBracket => (0x1B, false),
        Key::Return => (0x1C, false),
        Key::ControlLeft => (0x1D, false),
        Key::KeyA => (0x1E, false),
        Key::KeyS => (0x1F, false),
        Key::KeyD => (0x20, false),
        Key::KeyF => (0x21, false),
        Key::KeyG => (0x22, false),
        Key::KeyH => (0x23, false),
        Key::KeyJ => (0x24, false),
        Key::KeyK => (0x25, false),
        Key::KeyL => (0x26, false),
        Key::SemiColon => (0x27, false),
        Key::Quote => (0x28, false),
        Key::BackQuote => (0x29, false),
        Key::ShiftLeft => (0x2A, false),
        Key::BackSlash => (0x2B, false),
        Key::IntlBackslash => (0x56, false),
        Key::KeyZ => (0x2C, false),
        Key::KeyX => (0x2D, false),
        Key::KeyC => (0x2E, false),
        Key::KeyV => (0x2F, false),
        Key::KeyB => (0x30, false),
        Key::KeyN => (0x31, false),
        Key::KeyM => (0x32, false),
        Key::Comma => (0x33, false),
        Key::Dot => (0x34, false),
        Key::Slash => (0x35, false),
        Key::ShiftRight => (0x36, false),
        Key::KpMultiply => (0x37, false),
        Key::Alt => (0x38, false),
        Key::Space => (0x39, false),
        Key::CapsLock => (0x3A, false),
        Key::F1 => (0x3B, false),
        Key::F2 => (0x3C, false),
        Key::F3 => (0x3D, false),
        Key::F4 => (0x3E, false),
        Key::F5 => (0x3F, false),
        Key::F6 => (0x40, false),
        Key::F7 => (0x41, false),
        Key::F8 => (0x42, false),
        Key::F9 => (0x43, false),
        Key::F10 => (0x44, false),
        Key::NumLock => (0x45, false),
        Key::ScrollLock => (0x46, false),
        Key::Kp7 => (0x47, false),
        Key::Kp8 => (0x48, false),
        Key::Kp9 => (0x49, false),
        Key::KpMinus => (0x4A, false),
        Key::Kp4 => (0x4B, false),
        Key::Kp5 => (0x4C, false),
        Key::Kp6 => (0x4D, false),
        Key::KpPlus => (0x4E, false),
        Key::Kp1 => (0x4F, false),
        Key::Kp2 => (0x50, false),
        Key::Kp3 => (0x51, false),
        Key::Kp0 => (0x52, false),
        Key::F11 => (0x57, false),
        Key::F12 => (0x58, false),
        // Extended (E0-prefixed) keys
        Key::ControlRight => (0x1D, true),
        Key::AltGr => (0x38, true),
        Key::KpReturn => (0x1C, true),
        Key::KpDivide => (0x35, true),
        Key::MetaLeft => (0x5B, true),
        Key::MetaRight => (0x5C, true),
        Key::Insert => (0x52, true),
        Key::Delete => (0x53, true),
        Key::Home => (0x47, true),
        Key::End => (0x4F, true),
        Key::PageUp => (0x49, true),
        Key::PageDown => (0x51, true),
        Key::UpArrow => (0x48, true),
        Key::DownArrow => (0x50, true),
        Key::LeftArrow => (0x4B, true),
        Key::RightArrow => (0x4D, true),
        _ => return None,
    };
    Some(v)
}

/// Current virtual-desktop resolution in physical pixels (0,0 if unavailable).
#[cfg(windows)]
pub fn current_resolution() -> (u32, u32) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    };
    unsafe {
        let w = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1) as u32;
        let h = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1) as u32;
        (w, h)
    }
}

#[cfg(not(windows))]
pub fn current_resolution() -> (u32, u32) {
    (0, 0)
}

#[cfg(windows)]
pub fn move_mouse(x: f64, y: f64, src_w: f64, src_h: f64) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE,
        MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    };

    unsafe {
        // Use the recording resolution (if known) as the basis; otherwise the
        // live desktop. We convert the pixel to a fraction of that resolution,
        // then MOUSEEVENTF_VIRTUALDESK maps the 0..=65535 fraction onto the
        // actual virtual desktop — so a macro recorded at one resolution lands
        // correctly on any other (auto-scaling).
        let (w, h) = if src_w > 1.0 && src_h > 1.0 {
            (src_w, src_h)
        } else {
            (
                GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1) as f64,
                GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1) as f64,
            )
        };

        let fx = (x / (w - 1.0).max(1.0)).clamp(0.0, 1.0);
        let fy = (y / (h - 1.0).max(1.0)).clamp(0.0, 1.0);
        let nx = (fx * 65535.0).round() as i32;
        let ny = (fy * 65535.0).round() as i32;

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: nx,
                    dy: ny,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(not(windows))]
pub fn move_mouse(x: f64, y: f64, _src_w: f64, _src_h: f64) {
    // rdev handles absolute positioning on macOS/Linux. Resolution scaling and
    // multi-monitor caveats are documented in the README.
    let _ = simulate(&EventType::MouseMove { x, y });
}
