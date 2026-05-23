import { api } from "../api";
import type { Ctx, ViewController } from "../store";
import type { Mode, PlaybackProgress } from "../types";
import { card, clear, confirmDialog, h, pillToggle, toast } from "../ui";

export function createPlayback(ctx: Ctx): ViewController {
  let selectedId: string | null = null;
  let speed = ctx.settings.default_speed;
  let loops = 1;
  let infinite = false;
  let includeKeyboard = true;
  let includeMouse = true;
  let includeMouseMove = true;

  const selector = h("select", { class: "select" }) as HTMLSelectElement;
  selector.addEventListener("change", () => {
    selectedId = selector.value || null;
    api.setSelectedMacro(selectedId).catch(() => {});
  });

  const speedVal = h("span", { class: "chip-val mono" }, `${speed.toFixed(2)}x`);
  const speedSlider = h("input", {
    class: "slider",
    type: "range",
    min: "0.25",
    max: "5",
    step: "0.05",
    value: String(speed),
  }) as HTMLInputElement;
  speedSlider.addEventListener("input", () => {
    speed = parseFloat(speedSlider.value);
    speedVal.textContent = `${speed.toFixed(2)}x`;
  });

  const loopsInput = h("input", {
    class: "num-input",
    type: "number",
    min: "1",
    max: "100000",
    value: "1",
  }) as HTMLInputElement;
  loopsInput.addEventListener("input", () => {
    loops = Math.max(1, parseInt(loopsInput.value) || 1);
  });

  const infiniteToggle = pillToggle(false, (v) => {
    infinite = v;
    loopsInput.disabled = v;
    loopsInput.classList.toggle("disabled", v);
  });

  const progressFill = h("div", { class: "progress-fill" });
  const progressBar = h("div", { class: "progress-bar" }, progressFill);
  const progressText = h("div", { class: "progress-text" }, "Ready");

  const playBtn = h("button", { class: "btn primary big", type: "button" }, "Play");
  const stopBtn = h(
    "button",
    { class: "btn danger big", type: "button", disabled: true },
    "Stop",
  );

  playBtn.addEventListener("click", async () => {
    if (!selectedId) {
      toast("Select a macro first", "info");
      return;
    }
    if (infinite) {
      const ok = await confirmDialog({
        title: "Start infinite loop?",
        message: `Capped at ${ctx.settings.infinite_loop_cap} runs for safety. The emergency-stop hotkey (${ctx.settings.hotkeys.stop.code}) halts it instantly.`,
        okLabel: "Start",
      });
      if (!ok) return;
    }
    try {
      await api.startPlayback({
        macroId: selectedId,
        speed,
        loops,
        infinite,
        includeKeyboard,
        includeMouse,
        includeMouseMove,
      });
    } catch (e) {
      toast(String(e), "error");
    }
  });
  stopBtn.addEventListener("click", () => api.stopPlayback().catch(() => {}));

  const optsRow = h(
    "div",
    { class: "opts-grid" },
    optRow("Include keyboard", pillToggle(includeKeyboard, (v) => (includeKeyboard = v))),
    optRow("Include mouse clicks & scroll", pillToggle(includeMouse, (v) => (includeMouse = v))),
    optRow("Include mouse movement", pillToggle(includeMouseMove, (v) => (includeMouseMove = v))),
  );

  const body = card(
    { title: "Playback", subtitle: "Replay a macro with original timing" },
    field("Macro", selector),
    field(
      "Speed",
      h("div", { class: "slider-wrap" }, speedSlider, speedVal),
    ),
    field(
      "Loops",
      h(
        "div",
        { class: "loops-wrap" },
        loopsInput,
        h("span", { class: "loops-inf" }, "∞", infiniteToggle),
      ),
    ),
    optsRow,
    h("div", { class: "play-actions" }, playBtn, stopBtn),
    h("div", { class: "progress-wrap" }, progressBar, progressText),
    h(
      "div",
      { class: "play-hint" },
      `Mimic hides while playing — press ${ctx.settings.hotkeys.stop.code} (emergency stop) to halt.`,
    ),
  );

  const root = h("div", { class: "view" }, body);

  function fillSelector() {
    const macros = ctx.macros();
    clear(selector);
    if (macros.length === 0) {
      selector.append(h("option", { value: "" }, "No macros yet — record one"));
      selectedId = null;
      return;
    }
    for (const m of macros) {
      selector.append(
        h(
          "option",
          { value: m.id },
          `${m.name}  ·  ${m.event_count} events`,
        ),
      );
    }
    if (!selectedId || !macros.find((m) => m.id === selectedId)) {
      selectedId = macros[0].id;
    }
    selector.value = selectedId;
    api.setSelectedMacro(selectedId).catch(() => {});
  }

  function setMode(m: Mode) {
    const playing = m === "playing";
    playBtn.disabled = playing || m === "recording";
    stopBtn.disabled = !playing;
    selector.disabled = playing;
  }

  fillSelector();
  setMode(ctx.getMode());

  return {
    el: root,
    onMode: setMode,
    onProgress(p: PlaybackProgress) {
      progressFill.style.width = `${p.percent}%`;
      const loopLabel = p.infinite
        ? `Loop ${p.loop_index}/∞`
        : `Loop ${p.loop_index}/${p.loop_total}`;
      progressText.textContent = `${loopLabel} · event ${p.event_index}/${p.event_total}`;
    },
    onPlaybackFinished(stopped) {
      progressFill.style.width = "0%";
      progressText.textContent = stopped ? "Stopped" : "Finished";
      toast(stopped ? "Playback stopped" : "Playback finished", stopped ? "info" : "ok");
    },
  };
}

function field(label: string, control: HTMLElement): HTMLElement {
  return h(
    "div",
    { class: "field" },
    h("label", { class: "field-label" }, label),
    control,
  );
}

function optRow(label: string, toggle: HTMLElement): HTMLElement {
  return h("div", { class: "opt-row" }, h("span", {}, label), toggle);
}
