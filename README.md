# Mimic

A polished, cross-platform desktop **macro recorder/player**. Mimic records your
keyboard, mouse movement, clicks and scroll with precise timing, then replays the
exact sequence on demand — AutoHotkey-style power behind a clean, dark, card-based
GUI.

Built with **Tauri 2** (Rust backend) + **vanilla TypeScript / Vite** frontend.

---

## Features

- **Global recording** of keyboard (press/release), mouse movement, clicks and
  scroll — even while Mimic is in the background.
- **Faithful replay** reproducing original inter-event timing, with a
  **0.25×–5× speed** multiplier and **loop count / infinite** mode.
- **Playback subsets** — toggle keyboard, mouse clicks/scroll, and mouse movement
  independently ("keys only", "mouse only", etc.).
- **Global hotkeys** for record toggle, playback toggle, and an **emergency stop**
  that halts a runaway macro instantly. All configurable and persisted.
- **Macro library** — save / rename / duplicate / delete / import / export macros
  as JSON, with a per-macro **event timeline preview**.
- **Live feedback** while recording: elapsed timer, event count, cursor readout,
  pulsing status indicator.
- **Single state machine** — recording and playback can never run simultaneously.

---

## Prerequisites

- **Node.js** 20+ and **npm**
- **Rust** (stable) + Cargo
- Platform build deps for Tauri 2 (WebView2 on Windows is preinstalled on Win10/11;
  see <https://tauri.app/start/prerequisites/> for macOS/Linux).

## Setup & running

```bash
npm install          # install frontend deps + the Tauri CLI (local devDependency)
npm run tauri:dev    # launch the app with hot-reload
```

The first `tauri:dev` compiles the Rust backend (a few minutes); subsequent runs
are fast.

## Building an installer

```bash
npm run tauri:build
```

Produces a platform installer/bundle under `src-tauri/target/release/bundle/`
(MSI/NSIS on Windows, `.dmg`/`.app` on macOS, AppImage/deb on Linux).

---

## Per-OS permissions & caveats

### Windows
- Works out of the box. To **control apps running as administrator**, launch Mimic
  as administrator too (a non-elevated process cannot inject input into an elevated
  window — this is a Windows security boundary). The app surfaces this on startup.
- Mouse positioning uses the native virtual-desktop coordinate space, so
  **multi-monitor and mixed-DPI** setups replay correctly.

### macOS
- Grant **Accessibility** *and* **Input Monitoring** under
  *System Settings → Privacy & Security*, then restart Mimic. Without these, the OS
  silently blocks both capture and simulation.
- **Known limitation:** macOS requires the event tap (`CGEventTap`) to run on the
  main thread, which Tauri owns. The current always-on listener model runs the tap
  on a dedicated thread; on macOS you may need to run it from the main run-loop. See
  *Architecture → Hard problem #1* below — this is the one platform where the
  threading model needs a platform-specific adjustment.

### Linux
- Requires an **X11 / Xorg** session. `rdev`'s global hooks do **not** work under
  **Wayland** (Wayland deliberately isolates global input for security). Mimic
  detects a Wayland session and warns you on startup; log into an X11 session to use
  it.

### Safety
- The **emergency-stop hotkey** (default `F8`) is honored between every replayed
  event, so it halts playback essentially instantly even mid-run.
- **Infinite loops are hard-capped** (default 1000 runs, configurable) and require a
  confirmation before starting.

---

## Macro JSON format

Macros are stored as JSON in the OS app-data directory
(`%APPDATA%\com.mimic.app\macros\` on Windows, `~/Library/Application Support/...`
on macOS, `~/.local/share/...` on Linux). `t` is **milliseconds since recording
start**.

```jsonc
{
  "id": "f1c2…",              // uuid (assigned on save)
  "name": "My Macro",
  "version": 1,
  "created_at": "2026-05-22T12:00:00Z",
  "duration_ms": 12345,
  "events": [
    { "kind": "KeyPress",      "t": 0,   "key": "KeyA" },
    { "kind": "MouseMove",     "t": 120, "x": 540, "y": 300 },
    { "kind": "ButtonPress",   "t": 130, "button": "Left" },
    { "kind": "ButtonRelease", "t": 200, "button": "Left" },
    { "kind": "Wheel",         "t": 210, "dx": 0, "dy": -1 },
    { "kind": "KeyRelease",    "t": 250, "key": "KeyA" }
  ]
}
```

`key` and `button` use `rdev`'s enum names (`KeyA`, `ControlLeft`, `Left`,
`Middle`, …). The model reuses `rdev::Key`/`rdev::Button` directly via rdev's
`serialize` feature, guaranteeing a lossless round-trip between capture and replay
without a hand-maintained enum mapping.

---

## Architecture

```
src-tauri/src/
  model.rs       data model (MacroEvent, Macro, Settings, Hotkey) + serde
  state.rs       AppState + atomic Idle|Recording|Playing machine
  capture.rs     always-on rdev listener (recording)
  playback.rs    spin_sleep timing engine
  simulate.rs    event simulation (+ native Windows mouse move)
  storage.rs     JSON persistence in app-data dir
  hotkeys.rs     global-shortcut registration & dispatch
  mapping.rs     W3C code ⇄ rdev::Key ⇄ Shortcut bridge
  controller.rs  shared actions for commands + hotkeys
  lib.rs         Tauri commands + app bootstrap
src/             TypeScript frontend (views, components, event wiring)
```

The frontend is fully **event-driven**: the backend emits `status_changed`,
`recording_started/stopped`, `event_captured`, `playback_progress` and
`playback_finished`, so the UI always mirrors real backend state. All mode
transitions funnel through `controller.rs`, so a recording started by a hotkey and
one started by a button click take the identical path.

### The three hard problems

**1. `rdev` lifecycle & threading.**
`rdev::listen` is blocking and has no stop API. Instead of fighting it, Mimic starts
**one** listener for the whole app lifetime on a dedicated thread and gates behavior
on the current `Mode`. Benefits: we never need to stop/restart the listener; and
events we *simulate* during playback are naturally ignored because the mode is
`Playing`, not `Recording` — no feedback loop. On Windows the low-level hook needs a
message pump on its thread, which `rdev` runs internally (this is exactly why it
blocks), so a dedicated thread is its correct home. *macOS* needs the tap on the main
thread — the documented platform caveat above.

**2. Timing precision for replay.**
OS sleep granularity is ~15 ms on Windows — far too coarse. Each event's target time
is anchored to a per-loop `Instant` (so error never accumulates across a run) and we
wait with **`spin_sleep`**, which sleeps most of the interval then busy-spins the last
sub-millisecond. Speed scaling simply divides each target offset by the multiplier, so
2× halves every gap and 0.5× doubles it.

**3. Event volume & coordinates.**
Mouse movement is throttled by **both** a minimum time interval (default 12 ms) and a
minimum pixel distance (default 3 px); `last_*` only update on a kept sample, so slow
drift still accumulates distance and isn't lost. The hotkey trigger key is filtered
out of recordings so it never lands inside a macro. Coordinates are stored as
**absolute virtual-desktop pixels**; on Windows replay uses a native `SendInput` with
`MOUSEEVENTF_VIRTUALDESK`, normalizing across the whole virtual desktop so
**multi-monitor / DPI-scaled** layouts land where intended (rdev's built-in mouse
simulation normalizes against the primary monitor only, which we deliberately bypass).

---

## License

MIT
