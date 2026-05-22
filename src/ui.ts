// Tiny DOM helpers — no framework, just typed hyperscript + a few components.

type Props = Record<string, unknown>;
type Child = Node | string | number | null | undefined | false;

export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props?: Props,
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  if (props) {
    for (const [k, v] of Object.entries(props)) {
      if (v == null || v === false) continue;
      if (k === "class") el.className = String(v);
      else if (k === "html") el.innerHTML = String(v);
      else if (k === "style" && typeof v === "object")
        Object.assign(el.style, v as object);
      else if (k === "dataset" && typeof v === "object")
        Object.assign(el.dataset, v as object);
      else if (k.startsWith("on") && typeof v === "function")
        el.addEventListener(k.slice(2).toLowerCase(), v as EventListener);
      else if (k.includes("-") || k.startsWith("aria"))
        el.setAttribute(k, String(v));
      else (el as Record<string, unknown>)[k] = v;
    }
  }
  for (const c of children) {
    if (c == null || c === false) continue;
    el.append(c instanceof Node ? c : document.createTextNode(String(c)));
  }
  return el;
}

export function clear(el: HTMLElement) {
  while (el.firstChild) el.removeChild(el.firstChild);
}

/** Card with the signature accent tab in the top-left corner. */
export function card(
  opts: { title?: string; subtitle?: string; tab?: boolean; class?: string },
  ...body: Child[]
): HTMLElement {
  const head =
    opts.title || opts.subtitle
      ? h(
          "div",
          { class: "card-head" },
          opts.title && h("div", { class: "card-title" }, opts.title),
          opts.subtitle && h("div", { class: "card-sub" }, opts.subtitle),
        )
      : null;
  return h(
    "section",
    { class: "card" + (opts.class ? " " + opts.class : "") },
    opts.tab !== false && h("span", { class: "card-tab" }),
    head,
    h("div", { class: "card-body" }, ...body),
  );
}

/** Pill segmented OFF/ON toggle matching the reference aesthetic. */
export function pillToggle(
  value: boolean,
  onChange: (v: boolean) => void,
): HTMLElement {
  const el = h("button", {
    class: "pill" + (value ? " on" : ""),
    type: "button",
    "aria-pressed": String(value),
  });
  const off = h("span", { class: "pill-seg off" }, "OFF");
  const on = h("span", { class: "pill-seg on" }, "ON");
  el.append(off, on);
  el.addEventListener("click", () => {
    const next = !el.classList.contains("on");
    el.classList.toggle("on", next);
    el.setAttribute("aria-pressed", String(next));
    onChange(next);
  });
  return el;
}

/** Two-or-more option segmented control (e.g. Dark / Light). */
export function segmented(
  options: { value: string; label: string }[],
  value: string,
  onChange: (v: string) => void,
): HTMLElement {
  const el = h("div", { class: "segmented" });
  const btns: HTMLButtonElement[] = [];
  for (const o of options) {
    const b = h(
      "button",
      {
        class: "seg-btn" + (o.value === value ? " active" : ""),
        type: "button",
        onClick: () => {
          btns.forEach((x) => x.classList.toggle("active", x.dataset.val === o.value));
          onChange(o.value);
        },
      },
      o.label,
    );
    b.dataset.val = o.value;
    btns.push(b);
    el.append(b);
  }
  return el;
}

/** Inline label + value chip, e.g. `Speed  1.0x`. */
export function chip(label: string, value: string, mono = false): HTMLElement {
  return h(
    "span",
    { class: "chip" },
    h("span", { class: "chip-label" }, label),
    h("span", { class: "chip-val" + (mono ? " mono" : "") }, value),
  );
}

export function fmtDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const s = ms / 1000;
  if (s < 60) return `${s.toFixed(s < 10 ? 1 : 0)}s`;
  const m = Math.floor(s / 60);
  const rem = Math.round(s % 60);
  return `${m}m ${rem}s`;
}

export function fmtDate(iso: string): string {
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function fmtClock(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  const cs = Math.floor((ms % 1000) / 10);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}.${String(cs).padStart(2, "0")}`;
}

// ---------------- Modal dialogs (replace native alert/prompt/confirm) ----------------

interface ModalOpts {
  title: string;
  body?: Node;
  okLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}

function presentModal(opts: ModalOpts, resolve: (ok: boolean) => void) {
  const okBtn = h(
    "button",
    { class: "btn " + (opts.danger ? "danger" : "primary") },
    opts.okLabel ?? "OK",
  );
  const cancelBtn = h("button", { class: "btn ghost" }, opts.cancelLabel ?? "Cancel");

  const dialog = h(
    "div",
    { class: "modal" },
    h("span", { class: "card-tab" }),
    h("div", { class: "modal-title" }, opts.title),
    opts.body && h("div", { class: "modal-body" }, opts.body),
    h("div", { class: "modal-actions" }, cancelBtn, okBtn),
  );
  const overlay = h("div", { class: "modal-overlay" }, dialog);
  document.body.append(overlay);
  requestAnimationFrame(() => overlay.classList.add("show"));

  let done = false;
  const finish = (ok: boolean) => {
    if (done) return;
    done = true;
    overlay.classList.remove("show");
    document.removeEventListener("keydown", onKey, true);
    setTimeout(() => overlay.remove(), 180);
    resolve(ok);
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      finish(false);
    } else if (e.key === "Enter") {
      e.preventDefault();
      finish(true);
    }
  };
  document.addEventListener("keydown", onKey, true);
  overlay.addEventListener("mousedown", (e) => {
    if (e.target === overlay) finish(false);
  });
  okBtn.addEventListener("click", () => finish(true));
  cancelBtn.addEventListener("click", () => finish(false));
}

export function confirmDialog(opts: {
  title: string;
  message?: string;
  okLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}): Promise<boolean> {
  return new Promise((resolve) => {
    presentModal(
      {
        title: opts.title,
        body: opts.message ? h("p", { class: "modal-msg" }, opts.message) : undefined,
        okLabel: opts.okLabel ?? "Confirm",
        cancelLabel: opts.cancelLabel,
        danger: opts.danger,
      },
      resolve,
    );
  });
}

export function promptDialog(opts: {
  title: string;
  value?: string;
  placeholder?: string;
  okLabel?: string;
}): Promise<string | null> {
  return new Promise((resolve) => {
    const input = h("input", {
      class: "text-input",
      type: "text",
      value: opts.value ?? "",
      placeholder: opts.placeholder ?? "",
    }) as HTMLInputElement;
    presentModal(
      { title: opts.title, body: input, okLabel: opts.okLabel ?? "OK" },
      (ok) => resolve(ok ? input.value : null),
    );
    setTimeout(() => {
      input.focus();
      input.select();
    }, 30);
  });
}

let toastHost: HTMLElement | null = null;
export function toast(message: string, kind: "info" | "error" | "ok" = "info") {
  if (!toastHost) toastHost = document.getElementById("toast-host");
  if (!toastHost) return;
  const t = h("div", { class: `toast ${kind}` }, message);
  toastHost.append(t);
  requestAnimationFrame(() => t.classList.add("show"));
  setTimeout(() => {
    t.classList.remove("show");
    setTimeout(() => t.remove(), 250);
  }, 2800);
}
