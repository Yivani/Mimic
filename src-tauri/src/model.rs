//! Serializable data model for macros and settings.
//!
//! `rdev::Key` / `rdev::Button` are reused directly (rdev's `serialize` feature
//! gives them clean serde reprs: unit variants like `KeyA` serialize to the
//! string `"KeyA"`, `Left` to `"Left"`). This guarantees a lossless round-trip
//! between capture and playback without a hand-written enum mapping.

use rdev::{Button, Key};
use serde::{Deserialize, Serialize};

/// Normalised gamepad button names (Xbox layout).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    LeftShoulder,
    RightShoulder,
    LeftTrigger,
    RightTrigger,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Start,
    Back,
    Guide,
}

/// Normalised gamepad axis names.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

/// A single recorded input event. `t` is milliseconds since recording start.
///
/// Internally tagged on `kind` so JSON looks like:
/// `{ "kind": "KeyPress", "t": 0, "key": "KeyA" }`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum MacroEvent {
    KeyPress { t: u64, key: Key },
    KeyRelease { t: u64, key: Key },
    ButtonPress { t: u64, button: Button },
    ButtonRelease { t: u64, button: Button },
    MouseMove { t: u64, x: f64, y: f64 },
    Wheel { t: u64, dx: i64, dy: i64 },
    GamepadButtonPress { t: u64, button: GamepadButton },
    GamepadButtonRelease { t: u64, button: GamepadButton },
    GamepadAxis { t: u64, axis: GamepadAxis, value: f64 },
}

impl MacroEvent {
    pub fn t(&self) -> u64 {
        match self {
            MacroEvent::KeyPress { t, .. }
            | MacroEvent::KeyRelease { t, .. }
            | MacroEvent::ButtonPress { t, .. }
            | MacroEvent::ButtonRelease { t, .. }
            | MacroEvent::MouseMove { t, .. }
            | MacroEvent::Wheel { t, .. }
            | MacroEvent::GamepadButtonPress { t, .. }
            | MacroEvent::GamepadButtonRelease { t, .. }
            | MacroEvent::GamepadAxis { t, .. } => *t,
        }
    }

    pub fn is_keyboard(&self) -> bool {
        matches!(self, MacroEvent::KeyPress { .. } | MacroEvent::KeyRelease { .. })
    }

    pub fn is_mouse_move(&self) -> bool {
        matches!(self, MacroEvent::MouseMove { .. })
    }

    /// Mouse buttons + scroll wheel (everything mouse-ish that isn't movement).
    pub fn is_mouse_action(&self) -> bool {
        matches!(
            self,
            MacroEvent::ButtonPress { .. } | MacroEvent::ButtonRelease { .. } | MacroEvent::Wheel { .. }
        )
    }

    pub fn is_gamepad(&self) -> bool {
        matches!(
            self,
            MacroEvent::GamepadButtonPress { .. }
                | MacroEvent::GamepadButtonRelease { .. }
                | MacroEvent::GamepadAxis { .. }
        )
    }

    /// Short human label for the timeline preview, e.g. "KeyPress A".
    pub fn label(&self) -> String {
        match self {
            MacroEvent::KeyPress { key, .. } => format!("KeyPress {:?}", key),
            MacroEvent::KeyRelease { key, .. } => format!("KeyRelease {:?}", key),
            MacroEvent::ButtonPress { button, .. } => format!("ButtonPress {:?}", button),
            MacroEvent::ButtonRelease { button, .. } => format!("ButtonRelease {:?}", button),
            MacroEvent::MouseMove { x, y, .. } => format!("MouseMove ({:.0},{:.0})", x, y),
            MacroEvent::Wheel { dx, dy, .. } => format!("Wheel ({},{})", dx, dy),
            MacroEvent::GamepadButtonPress { button, .. } => {
                format!("GP BtnPress {:?}", button)
            }
            MacroEvent::GamepadButtonRelease { button, .. } => {
                format!("GP BtnRelease {:?}", button)
            }
            MacroEvent::GamepadAxis { axis, value, .. } => {
                format!("GP Axis {:?} {:.3}", axis, value)
            }
        }
    }
}

pub const MACRO_VERSION: u32 = 1;

/// A complete, saveable macro.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Macro {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: u32,
    pub created_at: String,
    pub duration_ms: u64,
    /// Virtual-desktop resolution captured at record time. Lets playback scale
    /// mouse coordinates to whatever resolution it later runs on. 0 = unknown.
    #[serde(default)]
    pub screen_width: u32,
    #[serde(default)]
    pub screen_height: u32,
    pub events: Vec<MacroEvent>,
}

fn default_version() -> u32 {
    MACRO_VERSION
}

/// Lightweight metadata returned for library listings (no event payload).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MacroMeta {
    pub id: String,
    pub name: String,
    pub duration_ms: u64,
    pub event_count: usize,
    pub created_at: String,
}

impl From<&Macro> for MacroMeta {
    fn from(m: &Macro) -> Self {
        MacroMeta {
            id: m.id.clone(),
            name: m.name.clone(),
            duration_ms: m.duration_ms,
            event_count: m.events.len(),
            created_at: m.created_at.clone(),
        }
    }
}

/// A configurable hotkey. `code` uses W3C `KeyboardEvent.code` values
/// (e.g. "F9", "KeyR", "Pause") so it maps cleanly to both the global-shortcut
/// plugin and the browser-side capture UI.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Hotkey {
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
    pub code: String,
}

impl Hotkey {
    pub fn simple(code: &str) -> Self {
        Hotkey {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            code: code.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Hotkeys {
    pub record: Hotkey,
    pub play: Hotkey,
    pub stop: Hotkey,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Hotkeys {
            record: Hotkey::simple("F9"),
            play: Hotkey::simple("F10"),
            stop: Hotkey::simple("F8"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub hotkeys: Hotkeys,
    /// Minimum ms between recorded mouse-move samples.
    pub sample_interval_ms: u64,
    /// Minimum pixel distance before a new mouse-move sample is kept.
    pub sample_distance_px: f64,
    pub default_speed: f64,
    pub accent: String,
    pub theme: String,
    pub launch_at_login: bool,
    pub start_minimized: bool,
    /// Hard cap on iterations when "infinite" loop is selected (safety).
    pub infinite_loop_cap: u32,
    /// Reference screen resolution the user selected for mouse accuracy.
    /// 0 = auto-detect at runtime.
    pub screen_width: u32,
    pub screen_height: u32,
    /// Include Xbox / gamepad input while recording.
    pub capture_gamepad: bool,
    /// Minimum ms between recorded gamepad-axis samples.
    pub gamepad_axis_interval_ms: u64,
    /// Minimum axis delta before a new gamepad-axis sample is kept.
    pub gamepad_axis_deadzone: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hotkeys: Hotkeys::default(),
            sample_interval_ms: 12,
            sample_distance_px: 3.0,
            default_speed: 1.0,
            accent: "#ff4b6e".to_string(),
            theme: "dark".to_string(),
            launch_at_login: false,
            start_minimized: false,
            infinite_loop_cap: 1000,
            screen_width: 0,
            screen_height: 0,
            capture_gamepad: true,
            gamepad_axis_interval_ms: 16,
            gamepad_axis_deadzone: 0.05,
        }
    }
}
