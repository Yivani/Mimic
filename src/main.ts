import "./styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, on } from "./api";
import { createTitlebar } from "./titlebar";
import { icons } from "./icons";
import { createLibrary } from "./views/library";
import { createPlayback } from "./views/playback";
import { createRecorder } from "./views/recorder";
import { createSettings } from "./views/settings";
import { checkForUpdate, type AvailableUpdate } from "./updater";
import type { Ctx, ViewController, ViewName } from "./store";
import type {
  EventCaptured,
  Macro,
  MacroMeta,
  Mode,
  PlaybackProgress,
  Settings,
} from "./types";
import { clear, h, setToastListener, toast } from "./ui";

let settings: Settings;
let mode: Mode = "idle";
let macrosCache: MacroMeta[] = [];
let current: ViewController | null = null;
let currentName: ViewName = "recorder";
let tb: ReturnType<typeof createTitlebar>;

const app = document.getElementById("app")!;
const content = document.getElementById("content")!;
const sidebar = document.getElementById("sidebar")!;
const titlebarHost = document.getElementById("titlebar")!;

function applyTheme(theme: string) {
  const t = theme === "light" ? "light" : "dark";
  document.documentElement.setAttribute("data-theme", t);
  tb?.setTheme(t);
}

const ctx: Ctx = {
  settings: undefined as unknown as Settings,
  getMode: () => mode,
  switchView,
  refreshMacros: async () => {
    macrosCache = await api.listMacros();
  },
  macros: () => macrosCache,
  saveSettings: async (s: Settings) => {
    await api.setSettings(s);
    settings = s;
    ctx.settings = s;
    applyTheme(s.theme);
  },
  toggleTheme: async () => {
    settings.theme = settings.theme === "light" ? "dark" : "light";
    ctx.settings = settings;
    applyTheme(settings.theme);
    try {
      await api.setSettings(settings);
    } catch {
      /* ignore persistence failure */
    }
  },
  checkForUpdates: async () => {
    const upd = await checkForUpdate();
    if (upd) {
      tb.flash("update", "Update");
      showUpdateBanner(upd);
      return true;
    }
    return false;
  },
  pendingRecording: null,
  recordStartedAt: null,
};

const NAV: { name: ViewName; label: string; icon: () => SVGElement }[] = [
  { name: "recorder", label: "Recorder", icon: () => icons.recordDot(16) },
  { name: "playback", label: "Playback", icon: () => icons.play(16) },
  { name: "library", label: "Library", icon: () => icons.list(16) },
  { name: "settings", label: "Settings", icon: () => icons.gear(16) },
];

function buildSidebar() {
  clear(sidebar);
  const tab = (n: (typeof NAV)[number]) => {
    const b = h(
      "button",
      { class: "tab-btn", "aria-label": n.label, onClick: () => switchView(n.name) },
      h("span", { class: "tab-glyph" }, n.icon()),
      h("span", { class: "tab-label" }, n.label),
    );
    b.dataset.view = n.name;
    return b;
  };
  for (const n of NAV) {
    if (n.name === "settings") continue; // Settings lives in the title bar now
    sidebar.append(tab(n));
  }
}

function updateNavActive() {
  sidebar.querySelectorAll<HTMLElement>(".tab-btn").forEach((b) => {
    b.classList.toggle("active", b.dataset.view === currentName);
  });
  tb?.setActiveView(currentName);
}

function switchView(name: ViewName, arg?: string) {
  if (current?.destroy) current.destroy();
  clear(content);
  currentName = name;
  let v: ViewController;
  switch (name) {
    case "recorder":
      v = createRecorder(ctx);
      break;
    case "playback":
      v = createPlayback(ctx);
      break;
    case "library":
      v = createLibrary(ctx, arg);
      break;
    case "settings":
      v = createSettings(ctx);
      break;
  }
  current = v;
  content.append(v.el);
  updateNavActive();
}

function setMode(m: Mode) {
  mode = m;
  tb.setStatus(m);
  current?.onMode?.(m);
  // Hide the window while recording OR playing, so Mimic is fully out of the
  // way and the simulated input reaches the target app, not Mimic itself.
  applyHiddenWindow(m === "recording" || m === "playing");
}

function applyHiddenWindow(hidden: boolean) {
  const w = getCurrentWindow();
  if (hidden) {
    w.hide().catch(() => {});
  } else {
    w.show().catch(() => {});
    w.setFocus().catch(() => {});
  }
}

function registerEvents() {
  on<Mode>("status_changed", (m) => setMode(m));
  on<void>("recording_started", () => {
    ctx.recordStartedAt = Date.now();
    ctx.pendingRecording = null;
  });
  on<EventCaptured>("event_captured", (p) => current?.onEventCaptured?.(p));
  on<{ x: number; y: number }>("mouse_position", (p) =>
    current?.onMousePos?.(p.x, p.y),
  );
  on<PlaybackProgress>("playback_progress", (p) => current?.onProgress?.(p));
  on<{ stopped: boolean }>("playback_finished", (p) => {
    current?.onPlaybackFinished?.(p.stopped);
  });
  on<Macro>("recording_stopped", (m) => {
    ctx.recordStartedAt = null;
    ctx.pendingRecording = m.events.length > 0 ? m : null;
    if (currentName !== "recorder") switchView("recorder");
    current?.onRecordingStopped?.(m);
  });
}

async function init() {
  settings = await api.getSettings();
  ctx.settings = settings;

  tb = createTitlebar({
    onToggleTheme: () => void ctx.toggleTheme(),
    onOpenSettings: () => switchView("settings"),
  });
  titlebarHost.append(tb.el);
  applyTheme(settings.theme);

  // Surface errors/results in the status indicator, not just toasts.
  setToastListener((_msg, kind) => {
    if (kind === "error") tb.flash("error", "Error");
    else if (kind === "ok") tb.flash("ok", "Done");
  });

  try {
    mode = await api.getStatus();
  } catch {
    mode = "idle";
  }
  await ctx.refreshMacros();

  buildSidebar();
  switchView("recorder");
  tb.setStatus(mode);
  registerEvents();

  try {
    const warnings = await api.platformWarnings();
    warnings.forEach((w) => toast(w, "info"));
  } catch {
    /* ignore */
  }

  // Check for updates in the background; show a banner if one is available.
  void checkForUpdate().then((upd) => {
    if (upd) {
      tb.flash("update", "Update");
      showUpdateBanner(upd);
    }
  });
}

function showUpdateBanner(upd: AvailableUpdate) {
  document.querySelector(".update-banner")?.remove(); // avoid stacking
  const text = h(
    "span",
    { class: "update-text" },
    "Update available  ",
    h("b", {}, `v${upd.currentVersion}`),
    "  →  ",
    h("b", {}, `v${upd.version}`),
  );
  const progress = h("span", { class: "update-progress" });
  const btn = h("button", { class: "btn primary small" }, "Download and Install");
  const dismiss = h("button", { class: "update-dismiss", title: "Dismiss" }, icons.close(14));

  const banner = h(
    "div",
    { class: "update-banner" },
    text,
    progress,
    btn,
    dismiss,
  );

  dismiss.addEventListener("click", () => banner.remove());
  btn.addEventListener("click", async () => {
    btn.disabled = true;
    btn.textContent = "Downloading…";
    try {
      await upd.install((f) => {
        progress.textContent = `${Math.round(f * 100)}%`;
        if (f >= 1) btn.textContent = "Installing…";
      });
      // App relaunches on success; this line is rarely reached.
    } catch (e) {
      btn.disabled = false;
      btn.textContent = "Retry";
      toast(`Update failed: ${String(e)}`, "error");
    }
  });

  app.insertBefore(banner, content);
}

init().catch((e) => {
  document.body.append(
    h("div", { class: "fatal" }, `Failed to start Mimic: ${String(e)}`),
  );
});
