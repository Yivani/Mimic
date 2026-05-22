import { getCurrentWindow } from "@tauri-apps/api/window";
import { h } from "./ui";
import { icons } from "./icons";
import type { Mode } from "./types";

export function createTitlebar(opts: {
  onToggleTheme: () => void;
  onOpenSettings: () => void;
}): {
  el: HTMLElement;
  setStatus(m: Mode): void;
  flash(kind: "error" | "ok" | "info" | "update", label: string): void;
  setTheme(theme: string): void;
  setActiveView(name: string): void;
} {
  const win = getCurrentWindow();

  const settingsBtn = h(
    "button",
    { class: "win-btn settings-btn", title: "Settings", onClick: opts.onOpenSettings },
    icons.gear(16),
  );

  const dot = h("span", { class: "status-dot" });
  const text = h("span", { class: "status-text" }, "Idle");
  const status = h("div", { class: "status-pill idle" }, dot, text);

  const themeBtn = h(
    "button",
    { class: "win-btn", title: "Toggle theme", onClick: opts.onToggleTheme },
    icons.moon(16),
  );

  let pinned = false;
  const pinBtn = h(
    "button",
    {
      class: "win-btn pin",
      title: "Always on top",
      onClick: async () => {
        pinned = !pinned;
        await win.setAlwaysOnTop(pinned);
        pinBtn.classList.toggle("active", pinned);
      },
    },
    icons.pin(16),
  );
  const minBtn = h(
    "button",
    { class: "win-btn", title: "Minimize", onClick: () => win.minimize() },
    icons.minimize(16),
  );
  const closeBtn = h(
    "button",
    { class: "win-btn close", title: "Close", onClick: () => win.close() },
    icons.close(16),
  );

  const logo = h(
    "div",
    { class: "logo" },
    h("span", { class: "logo-accent" }, "Mi"),
    h("span", {}, "mic"),
  );

  const el = h(
    "div",
    { class: "titlebar" },
    h("div", { class: "tb-left", "data-tauri-drag-region": "" }, logo),
    h("div", { class: "tb-center", "data-tauri-drag-region": "" }, status),
    h("div", { class: "tb-right" }, settingsBtn, themeBtn, pinBtn, minBtn, closeBtn),
  );

  let baseMode: Mode = "idle";
  let flashTimer: number | null = null;

  const render = (state: string, label: string) => {
    status.className = `status-pill ${state}`;
    text.textContent = label;
    // retrigger the entrance animation
    status.style.animation = "none";
    void status.offsetWidth;
    status.style.animation = "";
  };
  const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

  function clearFlash() {
    if (flashTimer != null) {
      clearTimeout(flashTimer);
      flashTimer = null;
    }
  }

  function setStatus(m: Mode) {
    baseMode = m;
    clearFlash();
    render(m, cap(m));
  }

  // Briefly show a transient state (error, done, etc.), then revert to the mode.
  function flash(kind: "error" | "ok" | "info" | "update", label: string) {
    clearFlash();
    render(kind, label);
    flashTimer = window.setTimeout(() => {
      flashTimer = null;
      render(baseMode, cap(baseMode));
    }, 2600);
  }

  function setTheme(theme: string) {
    themeBtn.replaceChildren(theme === "light" ? icons.moon(16) : icons.sun(16));
  }

  function setActiveView(name: string) {
    settingsBtn.classList.toggle("active", name === "settings");
  }

  return { el, setStatus, flash, setTheme, setActiveView };
}
