//! Cross-platform gamepad capture (via `gilrs`) and Windows-only virtual
//! controller playback (via `vigem-client` + ViGEmBus driver).
//!
//! On non-Windows platforms controller events are captured and stored in macros
//! but playback is silently skipped — a toast warns the user once per session.

use crate::model::{GamepadAxis, GamepadButton, MacroEvent};
use crate::state::{AppState, Mode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

// ---------------------------------------------------------------------------
// gilrs → model mappings
// ---------------------------------------------------------------------------

fn to_model_button(btn: gilrs::Button) -> Option<GamepadButton> {
    use gilrs::Button as B;
    Some(match btn {
        B::South => GamepadButton::A,
        B::East => GamepadButton::B,
        B::West => GamepadButton::X,
        B::North => GamepadButton::Y,
        B::LeftTrigger => GamepadButton::LeftShoulder,
        B::RightTrigger => GamepadButton::RightShoulder,
        B::LeftTrigger2 => GamepadButton::LeftTrigger,
        B::RightTrigger2 => GamepadButton::RightTrigger,
        B::LeftThumb => GamepadButton::LeftStick,
        B::RightThumb => GamepadButton::RightStick,
        B::DPadUp => GamepadButton::DPadUp,
        B::DPadDown => GamepadButton::DPadDown,
        B::DPadLeft => GamepadButton::DPadLeft,
        B::DPadRight => GamepadButton::DPadRight,
        B::Start => GamepadButton::Start,
        B::Select => GamepadButton::Back,
        B::Mode => GamepadButton::Guide,
        _ => return None,
    })
}

fn to_model_axis(axis: gilrs::Axis) -> Option<GamepadAxis> {
    use gilrs::Axis as A;
    Some(match axis {
        A::LeftStickX => GamepadAxis::LeftStickX,
        A::LeftStickY => GamepadAxis::LeftStickY,
        A::RightStickX => GamepadAxis::RightStickX,
        A::RightStickY => GamepadAxis::RightStickY,
        A::LeftZ => GamepadAxis::LeftTrigger,
        A::RightZ => GamepadAxis::RightTrigger,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Capture thread
// ---------------------------------------------------------------------------

#[derive(Clone, serde::Serialize)]
struct GamepadEventCaptured {
    count: usize,
    t: u64,
    label: String,
}

#[derive(Clone, serde::Serialize)]
struct GamepadStatus {
    connected: bool,
    name: String,
}

/// Spawn the gamepad polling thread. Call once during setup.
pub fn spawn_listener(state: Arc<AppState>, app: AppHandle) {
    std::thread::Builder::new()
        .name("mimic-gamepad".into())
        .spawn(move || {
            let mut gilrs = match gilrs::Gilrs::new() {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("[mimic] gilrs init failed: {:?}", e);
                    return;
                }
            };

            let mut last_status_emit = Instant::now() - Duration::from_secs(5);
            let mut was_connected = false;

            loop {
                // Emit connection status periodically
                let any_connected = gilrs.gamepads().any(|g| g.1.is_connected());
                if any_connected != was_connected
                    || last_status_emit.elapsed() >= Duration::from_secs(3)
                {
                    was_connected = any_connected;
                    last_status_emit = Instant::now();
                    let name = gilrs
                        .gamepads()
                        .find(|g| g.1.is_connected())
                        .map(|g| g.1.name().to_string())
                        .unwrap_or_default();
                    let _ = app.emit(
                        "gamepad_status",
                        GamepadStatus {
                            connected: any_connected,
                            name,
                        },
                    );
                }

                while let Some(event) = gilrs.next_event() {
                    // Axis changes are frequent — throttle them using the
                    // recording state's per-axis bookkeeping.
                    if let gilrs::EventType::AxisChanged(axis, value, _id) = event.event {
                        if state.mode() != Mode::Recording {
                            continue;
                        }
                        let axis_model = match to_model_axis(axis) {
                            Some(a) => a,
                            None => continue,
                        };
                        let (interval_ms, deadzone) = {
                            let s = state.settings.lock();
                            match s {
                                Ok(s) => (s.gamepad_axis_interval_ms, s.gamepad_axis_deadzone),
                                Err(_) => continue,
                            }
                        };
                        let mut rec = match state.recording.lock() {
                            Ok(r) => r,
                            Err(_) => continue,
                        };
                        if !rec.capture_gamepad {
                            continue;
                        }
                        let start = match rec.start {
                            Some(s) => s,
                            None => continue,
                        };
                        let t = start.elapsed().as_millis() as u64;

                        // Throttle: time + deadzone
                        let last_t = rec.gp_axis_last_t.get(&axis_model).copied().unwrap_or(0);
                        let last_v = rec.gp_axis_values.get(&axis_model).copied().unwrap_or(0.0);
                        let dt = t.saturating_sub(last_t);
                        let dv = (value as f64 - last_v).abs();
                        if dt < interval_ms && dv < deadzone {
                            continue;
                        }
                        rec.gp_axis_last_t.insert(axis_model.clone(), t);
                        rec.gp_axis_values.insert(axis_model.clone(), value as f64);
                        let ev = MacroEvent::GamepadAxis {
                            t,
                            axis: axis_model,
                            value: value as f64,
                        };
                        let label = ev.label();
                        rec.events.push(ev);
                        let count = rec.events.len();
                        drop(rec);
                        let _ = app.emit(
                            "event_captured",
                            GamepadEventCaptured { count, t, label },
                        );
                        continue;
                    }

                    // Buttons only while recording
                    if state.mode() != Mode::Recording {
                        continue;
                    }

                    let button_model = match event.event {
                        gilrs::EventType::ButtonPressed(b, _) | gilrs::EventType::ButtonReleased(b, _) => {
                            to_model_button(b)
                        }
                        _ => None,
                    };

                    let Some(button) = button_model else { continue };

                    let mut rec = match state.recording.lock() {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    if !rec.capture_gamepad {
                        continue;
                    }
                    let start = match rec.start {
                        Some(s) => s,
                        None => continue,
                    };
                    let t = start.elapsed().as_millis() as u64;

                    let ev = match event.event {
                        gilrs::EventType::ButtonPressed(..) => MacroEvent::GamepadButtonPress { t, button },
                        gilrs::EventType::ButtonReleased(..) => MacroEvent::GamepadButtonRelease { t, button },
                        _ => continue,
                    };
                    let label = ev.label();
                    rec.events.push(ev);
                    let count = rec.events.len();
                    drop(rec);
                    let _ = app.emit(
                        "event_captured",
                        GamepadEventCaptured { count, t, label },
                    );
                }

                std::thread::sleep(Duration::from_millis(2));
            }
        })
        .expect("failed to spawn gamepad thread");
}

// ---------------------------------------------------------------------------
// Playback simulation
// ---------------------------------------------------------------------------

/// Apply a single gamepad event to the virtual controller.
/// On non-Windows this is a no-op.
pub fn dispatch(ev: &MacroEvent, ctx: &mut VirtualGamepadContext) {
    #[cfg(windows)]
    {
        if let Some(ref mut vc) = ctx.controller {
            vc.apply(ev);
        }
    }
    let _ = ev;
    let _ = ctx;
}

/// Context maintained across a playback session.
pub struct VirtualGamepadContext {
    #[cfg(windows)]
    controller: Option<WindowsVirtualGamepad>,
    #[cfg(not(windows))]
    warned: bool,
}

impl VirtualGamepadContext {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            VirtualGamepadContext {
                controller: WindowsVirtualGamepad::new(),
            }
        }
        #[cfg(not(windows))]
        {
            VirtualGamepadContext { warned: false }
        }
    }

    pub fn warn_if_unsupported(&mut self) {
        #[cfg(not(windows))]
        if !self.warned {
            self.warned = true;
            eprintln!("[mimic] gamepad playback is only supported on Windows with ViGEmBus installed");
        }
    }
}

// ---------------------------------------------------------------------------
// Windows: ViGEm virtual Xbox 360 controller
// ---------------------------------------------------------------------------

#[cfg(windows)]
struct WindowsVirtualGamepad {
    // Kept alive so the target's borrow remains valid.
    #[allow(dead_code)]
    client: std::sync::Arc<vigem_client::Client>,
    target: vigem_client::Xbox360Wired<std::sync::Arc<vigem_client::Client>>,
    state: vigem_client::XGamepad,
}

#[cfg(windows)]
impl WindowsVirtualGamepad {
    fn new() -> Option<Self> {
        let client = match vigem_client::Client::connect() {
            Ok(c) => std::sync::Arc::new(c),
            Err(e) => {
                eprintln!("[mimic] ViGEmBus not available (gamepad playback disabled): {:?}", e);
                return None;
            }
        };
        let mut target = vigem_client::Xbox360Wired::new(
            client.clone(),
            vigem_client::TargetId::XBOX360_WIRED,
        );
        if let Err(e) = target.plugin() {
            eprintln!("[mimic] failed to plug virtual gamepad: {:?}", e);
            return None;
        }
        if let Err(e) = target.wait_ready() {
            eprintln!("[mimic] virtual gamepad not ready: {:?}", e);
            let _ = target.unplug();
            return None;
        }
        Some(WindowsVirtualGamepad {
            client,
            target,
            state: vigem_client::XGamepad::default(),
        })
    }

    fn apply(&mut self, ev: &MacroEvent) {
        use crate::model::GamepadButton as GB;
        use vigem_client::XButtons;

        match ev {
            MacroEvent::GamepadButtonPress { button, .. } => {
                let flag = match button {
                    GB::A => XButtons::A,
                    GB::B => XButtons::B,
                    GB::X => XButtons::X,
                    GB::Y => XButtons::Y,
                    GB::LeftShoulder => XButtons::LB,
                    GB::RightShoulder => XButtons::RB,
                    GB::LeftStick => XButtons::LTHUMB,
                    GB::RightStick => XButtons::RTHUMB,
                    GB::DPadUp => XButtons::UP,
                    GB::DPadDown => XButtons::DOWN,
                    GB::DPadLeft => XButtons::LEFT,
                    GB::DPadRight => XButtons::RIGHT,
                    GB::Start => XButtons::START,
                    GB::Back => XButtons::BACK,
                    GB::Guide => XButtons::GUIDE,
                    // Analog triggers handled via axis; map button press to max
                    GB::LeftTrigger => {
                        self.state.left_trigger = 255;
                        let _ = self.target.update(&self.state);
                        return;
                    }
                    GB::RightTrigger => {
                        self.state.right_trigger = 255;
                        let _ = self.target.update(&self.state);
                        return;
                    }
                };
                self.state.buttons.raw |= flag;
                let _ = self.target.update(&self.state);
            }
            MacroEvent::GamepadButtonRelease { button, .. } => {
                let flag = match button {
                    GB::A => XButtons::A,
                    GB::B => XButtons::B,
                    GB::X => XButtons::X,
                    GB::Y => XButtons::Y,
                    GB::LeftShoulder => XButtons::LB,
                    GB::RightShoulder => XButtons::RB,
                    GB::LeftStick => XButtons::LTHUMB,
                    GB::RightStick => XButtons::RTHUMB,
                    GB::DPadUp => XButtons::UP,
                    GB::DPadDown => XButtons::DOWN,
                    GB::DPadLeft => XButtons::LEFT,
                    GB::DPadRight => XButtons::RIGHT,
                    GB::Start => XButtons::START,
                    GB::Back => XButtons::BACK,
                    GB::Guide => XButtons::GUIDE,
                    GB::LeftTrigger => {
                        self.state.left_trigger = 0;
                        let _ = self.target.update(&self.state);
                        return;
                    }
                    GB::RightTrigger => {
                        self.state.right_trigger = 0;
                        let _ = self.target.update(&self.state);
                        return;
                    }
                };
                self.state.buttons.raw &= !flag;
                let _ = self.target.update(&self.state);
            }
            MacroEvent::GamepadAxis { axis, value, .. } => {
                use crate::model::GamepadAxis as GA;
                // gilrs range: sticks [-1,1], triggers [0,1]
                // XGamepad: sticks [-32768,32767], triggers [0,255]
                let clamped = value.clamp(-1.0, 1.0);
                match axis {
                    GA::LeftStickX => {
                        self.state.thumb_lx = (clamped * 32767.0) as i16;
                    }
                    GA::LeftStickY => {
                        self.state.thumb_ly = (clamped * 32767.0) as i16;
                    }
                    GA::RightStickX => {
                        self.state.thumb_rx = (clamped * 32767.0) as i16;
                    }
                    GA::RightStickY => {
                        self.state.thumb_ry = (clamped * 32767.0) as i16;
                    }
                    GA::LeftTrigger => {
                        self.state.left_trigger = ((clamped.max(0.0)) * 255.0) as u8;
                    }
                    GA::RightTrigger => {
                        self.state.right_trigger = ((clamped.max(0.0)) * 255.0) as u8;
                    }
                }
                let _ = self.target.update(&self.state);
            }
            _ => {}
        }
    }
}

#[cfg(windows)]
impl Drop for WindowsVirtualGamepad {
    fn drop(&mut self) {
        let _ = self.target.unplug();
    }
}
