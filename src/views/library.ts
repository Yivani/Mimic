import { open, save } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { icons } from "../icons";
import type { Ctx, ViewController } from "../store";
import type { Macro, MacroEvent, MacroMeta } from "../types";
import {
  card,
  chip,
  clear,
  confirmDialog,
  fmtDate,
  fmtDuration,
  h,
  promptDialog,
  toast,
} from "../ui";

export function createLibrary(ctx: Ctx, openId?: string): ViewController {
  const listWrap = h("div", { class: "macro-list" });
  const detailWrap = h("div", { class: "detail-wrap hidden" });

  const importBtn = h(
    "button",
    {
      class: "btn ghost",
      onClick: async () => {
        const path = await open({
          multiple: false,
          filters: [{ name: "Mimic macro", extensions: ["json"] }],
        });
        if (typeof path !== "string") return;
        try {
          await api.importMacro(path);
          await ctx.refreshMacros();
          renderList();
          toast("Macro imported", "ok");
        } catch (e) {
          toast(String(e), "error");
        }
      },
    },
    icons.upload(15),
    h("span", {}, "Import"),
  );

  const body = card(
    {
      title: "Library",
      subtitle: "Saved macros",
    },
    h("div", { class: "library-toolbar" }, importBtn),
    listWrap,
    detailWrap,
  );

  const root = h("div", { class: "view" }, body);

  function renderList() {
    detailWrap.classList.add("hidden");
    listWrap.classList.remove("hidden");
    clear(listWrap);
    const macros = ctx.macros();
    if (macros.length === 0) {
      listWrap.append(
        h(
          "div",
          { class: "empty" },
          "No macros yet. Head to the Recorder to capture one.",
        ),
      );
      return;
    }
    for (const m of macros) listWrap.append(macroRow(m));
  }

  function macroRow(m: MacroMeta): HTMLElement {
    const meta = h(
      "div",
      { class: "row-meta" },
      chip("Events", String(m.event_count), true),
      chip("Length", fmtDuration(m.duration_ms), true),
      chip("Created", fmtDate(m.created_at), true),
    );

    const actions = h(
      "div",
      { class: "row-actions" },
      iconBtn("Play", icons.play(15), async () => {
        try {
          await api.startPlayback({
            macroId: m.id,
            speed: ctx.settings.default_speed,
            loops: 1,
            infinite: false,
            includeKeyboard: true,
            includeMouse: true,
            includeMouseMove: true,
            includeGamepad: true,
          });
        } catch (e) {
          toast(String(e), "error");
        }
      }),
      iconBtn("Details", icons.list(15), () => openDetail(m.id)),
      iconBtn("Rename", icons.edit(15), () => doRename(m)),
      iconBtn("Duplicate", icons.copy(15), async () => {
        try {
          await api.duplicateMacro(m.id);
          await ctx.refreshMacros();
          renderList();
        } catch (e) {
          toast(String(e), "error");
        }
      }),
      iconBtn("Export", icons.download(15), async () => {
        const path = await save({
          defaultPath: `${m.name}.json`,
          filters: [{ name: "Mimic macro", extensions: ["json"] }],
        });
        if (!path) return;
        try {
          await api.exportMacro(m.id, path);
          toast("Exported", "ok");
        } catch (e) {
          toast(String(e), "error");
        }
      }),
      iconBtn("Delete", icons.trash(15), async () => {
        const ok = await confirmDialog({
          title: "Delete macro",
          message: `Delete “${m.name}”? This can’t be undone.`,
          okLabel: "Delete",
          danger: true,
        });
        if (!ok) return;
        try {
          await api.deleteMacro(m.id);
          await ctx.refreshMacros();
          renderList();
        } catch (e) {
          toast(String(e), "error");
        }
      }),
    );

    return h(
      "div",
      { class: "macro-row" },
      h(
        "div",
        { class: "row-main" },
        h("div", { class: "row-name" }, m.name),
        meta,
      ),
      actions,
    );
  }

  async function doRename(m: MacroMeta) {
    const name = await promptDialog({
      title: "Rename macro",
      value: m.name,
      placeholder: "Macro name",
      okLabel: "Rename",
    });
    if (name == null) return;
    const trimmed = name.trim();
    if (!trimmed) return;
    try {
      await api.renameMacro(m.id, trimmed);
      await ctx.refreshMacros();
      renderList();
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function openDetail(id: string) {
    let full: Macro;
    try {
      full = await api.loadMacro(id);
    } catch (e) {
      toast(String(e), "error");
      return;
    }
    listWrap.classList.add("hidden");
    detailWrap.classList.remove("hidden");
    clear(detailWrap);

    const back = h(
      "button",
      { class: "btn ghost small", onClick: renderList },
      "← Back",
    );

    const head = h(
      "div",
      { class: "detail-head" },
      back,
      h("div", { class: "detail-name" }, full.name),
      h(
        "div",
        { class: "detail-chips" },
        chip("Events", String(full.events.length), true),
        chip("Length", fmtDuration(full.duration_ms), true),
        chip("v", String(full.version), true),
      ),
    );

    const timeline = h("div", { class: "timeline" });
    const cap = 2000;
    full.events.slice(0, cap).forEach((ev) => timeline.append(eventRow(ev)));
    if (full.events.length > cap) {
      timeline.append(
        h(
          "div",
          { class: "timeline-more" },
          `… ${full.events.length - cap} more events`,
        ),
      );
    }

    detailWrap.append(head, timeline);
  }

  if (openId) {
    openDetail(openId);
  } else {
    renderList();
  }

  return {
    el: root,
    onMode() {
      // keep list fresh-looking; nothing high-frequency here
    },
  };
}

function eventRow(ev: MacroEvent): HTMLElement {
  return h(
    "div",
    { class: `tl-row k-${ev.kind}` },
    h("span", { class: "tl-t mono" }, `${ev.t}ms`),
    h("span", { class: "tl-kind" }, ev.kind),
    h("span", { class: "tl-detail mono" }, eventDetail(ev)),
  );
}

function eventDetail(ev: MacroEvent): string {
  switch (ev.kind) {
    case "KeyPress":
    case "KeyRelease":
      return ev.key ?? "";
    case "ButtonPress":
    case "ButtonRelease":
      return ev.button ?? "";
    case "MouseMove":
      return `(${Math.round(ev.x ?? 0)}, ${Math.round(ev.y ?? 0)})`;
    case "Wheel":
      return `Δ(${ev.dx ?? 0}, ${ev.dy ?? 0})`;
    case "GamepadButtonPress":
    case "GamepadButtonRelease":
      return ev.button ?? "";
    case "GamepadAxis":
      return `${ev.axis ?? ""} ${(ev.value ?? 0).toFixed(3)}`;
    default:
      return "";
  }
}

function iconBtn(
  title: string,
  glyph: SVGElement,
  onClick: () => void,
): HTMLElement {
  return h("button", { class: "icon-btn", title, onClick }, glyph);
}
