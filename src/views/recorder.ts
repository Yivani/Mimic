import { api } from "../api";
import { icons } from "../icons";
import type { Ctx, ViewController } from "../store";
import type { EventCaptured, Macro, Mode } from "../types";
import { card, chip, clear, fmtClock, fmtDuration, h, toast } from "../ui";

export function createRecorder(ctx: Ctx): ViewController {
  let startTime = 0;
  let timer: number | null = null;
  let pending: Macro | null = null;

  const elapsedVal = h("span", { class: "stat-val mono" }, "00:00.00");
  const eventsVal = h("span", { class: "stat-val mono" }, "0");
  const cursorVal = h("span", { class: "stat-val mono" }, "—");

  const bigBtn = h("button", { class: "record-btn", type: "button" });
  bigBtn.append(
    h("span", { class: "rb-pulse" }),
    h("span", { class: "rb-pulse delay" }),
    h("span", { class: "rb-core" }),
  );
  const btnLabel = h("span", { class: "rb-label" }, "Record");
  const btnHint = h("span", { class: "rb-hint" }, "");

  const stage = h(
    "div",
    { class: "record-stage" },
    bigBtn,
    h("div", { class: "rb-caption" }, btnLabel, btnHint),
  );

  const savePanel = h("div", { class: "save-panel hidden" });

  const stats = h(
    "div",
    { class: "stat-row" },
    statChip("Elapsed", icons.clock(16), "elapsed", elapsedVal),
    statChip("Events", icons.activity(16), "events", eventsVal),
    statChip("Cursor", icons.cursor(16), "cursor", cursorVal),
  );

  const recorderCard = card(
    { title: "Recorder", subtitle: "Capture keyboard, mouse & scroll globally" },
    stage,
    stats,
    savePanel,
  );

  const root = h("div", { class: "view" }, recorderCard);

  function setButton(mode: Mode) {
    bigBtn.classList.toggle("recording", mode === "recording");
    bigBtn.classList.toggle("disabled", mode === "playing");
    stage.classList.toggle("recording", mode === "recording");
    btnLabel.textContent = mode === "recording" ? "Stop" : "Record";
    btnHint.textContent =
      mode === "playing"
        ? "Playback in progress"
        : mode === "recording"
          ? `Window hidden while recording — press ${hk(ctx.settings.hotkeys.record)} to stop`
          : `Click or press ${hk(ctx.settings.hotkeys.record)}`;
  }

  function startTimer() {
    startTime = ctx.recordStartedAt ?? Date.now();
    stopTimer();
    timer = window.setInterval(() => {
      elapsedVal.textContent = fmtClock(Date.now() - startTime);
    }, 33);
  }
  function stopTimer() {
    if (timer != null) {
      clearInterval(timer);
      timer = null;
    }
  }

  bigBtn.addEventListener("click", async () => {
    if (ctx.getMode() === "playing") return;
    try {
      if (ctx.getMode() === "recording") {
        await api.stopRecording(); // result arrives via recording_stopped event
      } else {
        eventsVal.textContent = "0";
        savePanel.classList.add("hidden");
        await api.startRecording();
      }
    } catch (e) {
      toast(String(e), "error");
    }
  });

  function showSavePanel(m: Macro) {
    pending = m;
    ctx.pendingRecording = m;
    clear(savePanel);
    savePanel.classList.remove("hidden");
    const nameInput = h("input", {
      class: "text-input",
      type: "text",
      value: m.name,
      placeholder: "Macro name",
    }) as HTMLInputElement;

    const meta = h(
      "div",
      { class: "save-meta" },
      chip("Duration", fmtDuration(m.duration_ms), true),
      chip("Events", String(m.events.length), true),
    );

    const saveBtn = h(
      "button",
      {
        class: "btn primary",
        onClick: async () => {
          const macro = { ...pending!, name: nameInput.value.trim() || m.name };
          try {
            await api.saveMacro(macro);
            await ctx.refreshMacros();
            toast("Macro saved", "ok");
            savePanel.classList.add("hidden");
            pending = null;
            ctx.pendingRecording = null;
          } catch (e) {
            toast(String(e), "error");
          }
        },
      },
      "Save macro",
    );
    const discardBtn = h(
      "button",
      {
        class: "btn ghost",
        onClick: () => {
          savePanel.classList.add("hidden");
          pending = null;
          ctx.pendingRecording = null;
        },
      },
      "Discard",
    );

    savePanel.append(
      h("div", { class: "save-title" }, "Recording finished"),
      meta,
      h("div", { class: "save-row" }, nameInput),
      h("div", { class: "save-actions" }, saveBtn, discardBtn),
    );
  }

  setButton(ctx.getMode());
  if (ctx.getMode() === "recording") {
    startTimer();
  } else if (ctx.pendingRecording) {
    // Restore the unsaved recording's save panel after a tab switch.
    elapsedVal.textContent = fmtClock(ctx.pendingRecording.duration_ms);
    eventsVal.textContent = String(ctx.pendingRecording.events.length);
    showSavePanel(ctx.pendingRecording);
  }

  return {
    el: root,
    onMode(m) {
      setButton(m);
      if (m === "recording") {
        eventsVal.textContent = "0";
        startTimer();
      } else {
        stopTimer();
      }
    },
    onEventCaptured(p: EventCaptured) {
      eventsVal.textContent = String(p.count);
    },
    onMousePos(x, y) {
      cursorVal.textContent = `${x}, ${y}`;
    },
    onRecordingStopped(m) {
      stopTimer();
      elapsedVal.textContent = fmtClock(m.duration_ms);
      if (m.events.length > 0) showSavePanel(m);
      else toast("Nothing recorded", "info");
    },
    destroy() {
      stopTimer();
    },
  };
}

function statChip(
  label: string,
  icon: SVGElement,
  kind: string,
  valEl: HTMLElement,
): HTMLElement {
  return h(
    "div",
    { class: `stat stat-${kind}` },
    h("span", { class: "stat-icon" }, icon),
    h(
      "div",
      { class: "stat-body" },
      h("span", { class: "stat-label" }, label),
      valEl,
    ),
  );
}

function hk(k: { ctrl: boolean; shift: boolean; alt: boolean; meta: boolean; code: string }): string {
  const parts: string[] = [];
  if (k.ctrl) parts.push("Ctrl");
  if (k.shift) parts.push("Shift");
  if (k.alt) parts.push("Alt");
  if (k.meta) parts.push("Meta");
  parts.push(k.code.replace(/^Key|^Digit/, ""));
  return parts.join("+");
}
