//! Shared application state and the single `Idle | Recording | Playing` machine.
//!
//! The mode is an atomic so the always-on capture callback (a hot path) can
//! read it without taking a lock. Recording and playback can never overlap:
//! every transition is gated on the current mode.

use crate::model::{MacroEvent, Settings};
use rdev::Key;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Idle,
    Recording,
    Playing,
}

impl Mode {
    fn from_u8(v: u8) -> Mode {
        match v {
            1 => Mode::Recording,
            2 => Mode::Playing,
            _ => Mode::Idle,
        }
    }
    fn as_u8(self) -> u8 {
        match self {
            Mode::Idle => 0,
            Mode::Recording => 1,
            Mode::Playing => 2,
        }
    }
}

/// Mutable state for an in-progress recording. The throttle parameters are
/// snapshotted from settings at recording start so the capture callback never
/// has to lock the settings mutex.
#[derive(Default)]
pub struct RecordingState {
    pub start: Option<Instant>,
    pub events: Vec<MacroEvent>,
    pub interval_ms: u64,
    pub distance_px: f64,
    pub last_move_t: u64,
    pub last_x: f64,
    pub last_y: f64,
    pub have_last: bool,
}

pub struct AppState {
    mode: AtomicU8,
    pub recording: Mutex<RecordingState>,
    /// Set true to ask the running playback loop to stop ASAP.
    pub stop_playback: AtomicBool,
    pub settings: Mutex<Settings>,
    /// When true, global hotkeys are ignored (e.g. while capturing a new combo).
    pub hotkeys_suspended: AtomicBool,
    /// Keys belonging to hotkeys, filtered out of recordings (small; linear ok).
    pub filter_keys: Mutex<Vec<Key>>,
    pub last_macro_id: Mutex<Option<String>>,
    pub cur_x: AtomicI32,
    pub cur_y: AtomicI32,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        AppState {
            mode: AtomicU8::new(Mode::Idle.as_u8()),
            recording: Mutex::new(RecordingState::default()),
            stop_playback: AtomicBool::new(false),
            settings: Mutex::new(settings),
            hotkeys_suspended: AtomicBool::new(false),
            filter_keys: Mutex::new(Vec::new()),
            last_macro_id: Mutex::new(None),
            cur_x: AtomicI32::new(0),
            cur_y: AtomicI32::new(0),
        }
    }

    pub fn mode(&self) -> Mode {
        Mode::from_u8(self.mode.load(Ordering::SeqCst))
    }

    pub fn set_mode(&self, m: Mode) {
        self.mode.store(m.as_u8(), Ordering::SeqCst);
    }

    /// Atomically move from `from` to `to`; returns false if not in `from`.
    pub fn try_transition(&self, from: Mode, to: Mode) -> bool {
        self.mode
            .compare_exchange(from.as_u8(), to.as_u8(), Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn is_filtered(&self, key: &Key) -> bool {
        self.filter_keys
            .lock()
            .map(|f| f.contains(key))
            .unwrap_or(false)
    }
}
