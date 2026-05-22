import type {
  EventCaptured,
  Macro,
  MacroMeta,
  Mode,
  PlaybackProgress,
  Settings,
} from "./types";

export type ViewName = "recorder" | "playback" | "library" | "settings";

/** A mounted view. High-frequency backend events are forwarded to the optional
 * hooks so views can update specific DOM nodes without a full re-render. */
export interface ViewController {
  el: HTMLElement;
  onMode?(m: Mode): void;
  onEventCaptured?(p: EventCaptured): void;
  onMousePos?(x: number, y: number): void;
  onProgress?(p: PlaybackProgress): void;
  onRecordingStopped?(m: Macro): void;
  onPlaybackFinished?(stopped: boolean): void;
  destroy?(): void;
}

/** Shared services handed to every view. */
export interface Ctx {
  settings: Settings;
  getMode(): Mode;
  switchView(v: ViewName, arg?: string): void;
  refreshMacros(): Promise<void>;
  macros(): MacroMeta[];
  saveSettings(s: Settings): Promise<void>;
  /** Flip between dark/light, apply live, and persist. */
  toggleTheme(): Promise<void>;
  /** Manually check for an update; shows the banner if one is available.
   * Returns true if an update was found. */
  checkForUpdates(): Promise<boolean>;
  /** A finished-but-unsaved recording, cached so it survives tab switches. */
  pendingRecording: Macro | null;
  /** Wall-clock ms when the current recording began (for the elapsed timer). */
  recordStartedAt: number | null;
}
