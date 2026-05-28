import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Macro, MacroMeta, Mode, Settings } from "./types";

export interface PlaybackArgs {
  macroId: string;
  speed: number;
  loops: number;
  infinite: boolean;
  includeKeyboard: boolean;
  includeMouse: boolean;
  includeMouseMove: boolean;
  includeGamepad: boolean;
}

export const api = {
  startRecording: () => invoke<void>("start_recording"),
  stopRecording: () => invoke<Macro>("stop_recording"),

  startPlayback: (args: PlaybackArgs) =>
    invoke<void>("start_playback", args as unknown as Record<string, unknown>),
  stopPlayback: () => invoke<void>("stop_playback"),

  saveMacro: (macroData: Macro) =>
    invoke<MacroMeta>("save_macro", { macroData }),
  loadMacro: (id: string) => invoke<Macro>("load_macro", { id }),
  listMacros: () => invoke<MacroMeta[]>("list_macros"),
  renameMacro: (id: string, name: string) =>
    invoke<void>("rename_macro", { id, name }),
  duplicateMacro: (id: string) =>
    invoke<MacroMeta>("duplicate_macro", { id }),
  deleteMacro: (id: string) => invoke<void>("delete_macro", { id }),
  importMacro: (path: string) => invoke<MacroMeta>("import_macro", { path }),
  exportMacro: (id: string, path: string) =>
    invoke<void>("export_macro", { id, path }),

  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) =>
    invoke<void>("set_settings", { settings }),

  getStatus: () => invoke<Mode>("get_status"),
  suspendHotkeys: (suspended: boolean) =>
    invoke<void>("suspend_hotkeys", { suspended }),
  getMousePosition: () => invoke<[number, number]>("get_mouse_position"),
  getScreenResolution: () => invoke<[number, number]>("get_screen_resolution"),
  setSelectedMacro: (id: string | null) =>
    invoke<void>("set_selected_macro", { id }),
  platformWarnings: () => invoke<string[]>("platform_warnings"),
};

export function on<T>(
  event: string,
  cb: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (e) => cb(e.payload));
}
