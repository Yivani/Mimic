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
use rdev::{simulate, EventType};

/// `src_w`/`src_h` are the resolution the macro was recorded at, used to scale
/// mouse movement to the current screen (0 = use the live desktop size).
pub fn dispatch(ev: &MacroEvent, src_w: f64, src_h: f64) {
    match ev {
        MacroEvent::KeyPress { key, .. } => {
            let _ = simulate(&EventType::KeyPress(*key));
        }
        MacroEvent::KeyRelease { key, .. } => {
            let _ = simulate(&EventType::KeyRelease(*key));
        }
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
