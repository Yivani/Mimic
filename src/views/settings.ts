import { getVersion } from "@tauri-apps/api/app";
import { api } from "../api";
import type { Ctx, ViewController } from "../store";
import type { Hotkey, Settings } from "../types";
import { card, clear, h, pillToggle, segmented, toast } from "../ui";

export function createSettings(ctx: Ctx): ViewController {
  // Work on a draft copy; commit on Save.
  const draft: Settings = JSON.parse(JSON.stringify(ctx.settings));

  const recordCapture = hotkeyCapture(draft.hotkeys.record, (hk) => (draft.hotkeys.record = hk));
  const playCapture = hotkeyCapture(draft.hotkeys.play, (hk) => (draft.hotkeys.play = hk));
  const stopCapture = hotkeyCapture(draft.hotkeys.stop, (hk) => (draft.hotkeys.stop = hk));

  const intervalInput = numInput(draft.sample_interval_ms, 1, 100, (v) => (draft.sample_interval_ms = v));
  const distanceInput = numInput(draft.sample_distance_px, 0, 50, (v) => (draft.sample_distance_px = v));
  const speedInput = numInput(draft.default_speed, 0.25, 5, (v) => (draft.default_speed = v), 0.05);
  const capInput = numInput(draft.infinite_loop_cap, 1, 1000000, (v) => (draft.infinite_loop_cap = v));

  const themeSeg = segmented(
    [
      { value: "dark", label: "Dark" },
      { value: "light", label: "Light" },
    ],
    draft.theme === "light" ? "light" : "dark",
    (v) => {
      draft.theme = v;
      document.documentElement.setAttribute("data-theme", v);
    },
  );

  const resSelect = h("select", { class: "select" }) as HTMLSelectElement;
  resSelect.addEventListener("change", () => {
    const [w, ht] = resSelect.value.split("x").map(Number);
    draft.screen_width = w || 0;
    draft.screen_height = ht || 0;
  });
  void (async () => {
    let dw = 0;
    let dh = 0;
    try {
      const d = await api.getScreenResolution();
      dw = d[0];
      dh = d[1];
    } catch {
      /* ignore */
    }
    const common: [number, number][] = [
      [3840, 2160],
      [2560, 1440],
      [1920, 1200],
      [1920, 1080],
      [1680, 1050],
      [1600, 900],
      [1440, 900],
      [1366, 768],
      [1280, 800],
      [1280, 720],
    ];
    const list: [number, number][] = [];
    const add = (w: number, ht: number) => {
      if (w > 0 && !list.some((p) => p[0] === w && p[1] === ht)) list.push([w, ht]);
    };
    add(dw, dh);
    if (draft.screen_width > 0) add(draft.screen_width, draft.screen_height);
    for (const c of common) add(c[0], c[1]);

    clear(resSelect);
    for (const p of list) {
      const det = p[0] === dw && p[1] === dh;
      resSelect.append(
        h("option", { value: `${p[0]}x${p[1]}` }, `${p[0]} × ${p[1]}${det ? "  (detected)" : ""}`),
      );
    }
    if (draft.screen_width <= 0 && dw > 0) {
      draft.screen_width = dw;
      draft.screen_height = dh;
    }
    resSelect.value = `${draft.screen_width}x${draft.screen_height}`;
  })();

  const launchToggle = pillToggle(draft.launch_at_login, (v) => (draft.launch_at_login = v));
  const minimizedToggle = pillToggle(draft.start_minimized, (v) => (draft.start_minimized = v));

  const saveBtn = h(
    "button",
    {
      class: "btn primary",
      onClick: async () => {
        try {
          await ctx.saveSettings(draft);
          toast("Settings saved", "ok");
        } catch (e) {
          toast(String(e), "error");
        }
      },
    },
    "Save settings",
  );

  const hotkeysCard = card(
    { title: "Hotkeys", subtitle: "Click a field, then press the keys" },
    settingRow("Toggle recording", recordCapture.el),
    settingRow("Toggle playback", playCapture.el),
    settingRow("Emergency stop", stopCapture.el),
  );

  const captureCard = card(
    { title: "Capture", subtitle: "Mouse-movement sampling" },
    settingRow("Min sample interval (ms)", intervalInput),
    settingRow("Min sample distance (px)", distanceInput),
  );

  const playbackCard = card(
    { title: "Playback", subtitle: "Defaults & safety" },
    settingRow("Default speed (x)", speedInput),
    settingRow("Infinite-loop hard cap", capInput),
  );

  const displayCard = card(
    {
      title: "Display",
      subtitle: "Screen resolution — macros auto-scale to this for mouse accuracy",
    },
    settingRow("Resolution", resSelect),
  );

  const appearanceCard = card(
    { title: "Appearance", subtitle: "Theme" },
    settingRow("Theme", themeSeg),
  );

  const systemCard = card(
    { title: "System", subtitle: "Startup (persisted)" },
    settingRow("Launch at login", launchToggle),
    settingRow("Start minimized", minimizedToggle),
  );

  const versionLabel = h("span", { class: "version-label" }, "Mimic");
  void getVersion()
    .then((v) => (versionLabel.textContent = `Mimic v${v}`))
    .catch(() => {});

  const root = h(
    "div",
    { class: "view settings-view" },
    hotkeysCard,
    h("div", { class: "settings-grid" }, captureCard, playbackCard),
    displayCard,
    appearanceCard,
    systemCard,
    h(
      "div",
      { class: "settings-footer" },
      versionLabel,
      h("div", { class: "settings-save" }, saveBtn),
    ),
  );

  return { el: root };
}

function settingRow(label: string, control: HTMLElement): HTMLElement {
  return h(
    "div",
    { class: "setting-row" },
    h("span", { class: "setting-label" }, label),
    control,
  );
}

function numInput(
  value: number,
  min: number,
  max: number,
  onChange: (v: number) => void,
  step = 1,
): HTMLElement {
  const input = h("input", {
    class: "num-input wide",
    type: "number",
    min: String(min),
    max: String(max),
    step: String(step),
    value: String(value),
  }) as HTMLInputElement;
  input.addEventListener("input", () => {
    const v = parseFloat(input.value);
    if (!isNaN(v)) onChange(Math.min(max, Math.max(min, v)));
  });
  return input;
}

function hkLabel(k: Hotkey): string {
  const parts: string[] = [];
  if (k.ctrl) parts.push("Ctrl");
  if (k.shift) parts.push("Shift");
  if (k.alt) parts.push("Alt");
  if (k.meta) parts.push("Meta");
  if (k.code) parts.push(k.code.replace(/^Key|^Digit/, ""));
  return parts.join(" + ") || "Unset";
}

function hotkeyCapture(
  initial: Hotkey,
  onChange: (hk: Hotkey) => void,
): { el: HTMLElement } {
  const current: Hotkey = { ...initial };
  const btn = h("button", { class: "hotkey-capture", type: "button" }, hkLabel(current));
  let capturing = false;

  function stop() {
    capturing = false;
    btn.classList.remove("capturing");
    window.removeEventListener("keydown", onKey, true);
  }

  function onKey(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();
    // Ignore lone modifier presses; wait for a real key.
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
    const hk: Hotkey = {
      ctrl: e.ctrlKey,
      shift: e.shiftKey,
      alt: e.altKey,
      meta: e.metaKey,
      code: e.code,
    };
    Object.assign(current, hk);
    btn.textContent = hkLabel(current);
    onChange({ ...current });
    stop();
  }

  btn.addEventListener("click", () => {
    if (capturing) {
      stop();
      btn.textContent = hkLabel(current);
      return;
    }
    capturing = true;
    btn.classList.add("capturing");
    btn.textContent = "Press keys…";
    window.addEventListener("keydown", onKey, true);
  });

  return { el: btn };
}
