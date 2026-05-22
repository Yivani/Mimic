export type Mode = "idle" | "recording" | "playing";

export interface MacroEvent {
  kind:
    | "KeyPress"
    | "KeyRelease"
    | "ButtonPress"
    | "ButtonRelease"
    | "MouseMove"
    | "Wheel";
  t: number;
  key?: string;
  button?: string;
  x?: number;
  y?: number;
  dx?: number;
  dy?: number;
}

export interface Macro {
  id: string;
  name: string;
  version: number;
  created_at: string;
  duration_ms: number;
  screen_width?: number;
  screen_height?: number;
  events: MacroEvent[];
}

export interface MacroMeta {
  id: string;
  name: string;
  duration_ms: number;
  event_count: number;
  created_at: string;
}

export interface Hotkey {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
  code: string;
}

export interface Hotkeys {
  record: Hotkey;
  play: Hotkey;
  stop: Hotkey;
}

export interface Settings {
  hotkeys: Hotkeys;
  sample_interval_ms: number;
  sample_distance_px: number;
  default_speed: number;
  accent: string;
  theme: string;
  launch_at_login: boolean;
  start_minimized: boolean;
  infinite_loop_cap: number;
  screen_width: number;
  screen_height: number;
}

export interface PlaybackProgress {
  loop_index: number;
  loop_total: number;
  infinite: boolean;
  event_index: number;
  event_total: number;
  percent: number;
}

export interface EventCaptured {
  count: number;
  t: number;
  label: string;
}
