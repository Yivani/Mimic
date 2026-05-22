// Minimal stroked line icons (feather-style). No emojis. All use currentColor
// so they adapt to the active theme automatically. Each call returns a fresh
// SVG node (DOM nodes can't be shared between mount points).

const NS = "http://www.w3.org/2000/svg";

function make(paths: string[], opts: { size?: number; fill?: boolean } = {}): SVGElement {
  const size = opts.size ?? 18;
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("fill", opts.fill ? "currentColor" : "none");
  svg.setAttribute("stroke", opts.fill ? "none" : "currentColor");
  svg.setAttribute("stroke-width", "1.7");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.classList.add("icon");
  for (const d of paths) {
    const p = document.createElementNS(NS, "path");
    p.setAttribute("d", d);
    svg.appendChild(p);
  }
  return svg;
}

export const icons = {
  recordDot: (s?: number) => make(["M12 7a5 5 0 100 10 5 5 0 000-10z"], { fill: true, size: s }),
  play: (s?: number) => make(["M7 5l12 7-12 7z"], { fill: true, size: s }),
  stop: (s?: number) => make(["M7 7h10v10H7z"], { fill: true, size: s }),

  list: (s?: number) =>
    make(
      [
        "M8 6h13",
        "M8 12h13",
        "M8 18h13",
        "M3.5 6h.01",
        "M3.5 12h.01",
        "M3.5 18h.01",
      ],
      { size: s },
    ),
  gear: (s?: number) =>
    make(
      [
        "M12 15a3 3 0 100-6 3 3 0 000 6z",
        "M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 11-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 11-4 0v-.09A1.65 1.65 0 008 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 11-2.83-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 110-4h.09A1.65 1.65 0 004.6 8a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 112.83-2.83l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 114 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 112.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 110 4h-.09a1.65 1.65 0 00-1.51 1z",
      ],
      { size: s },
    ),

  clock: (s?: number) => make(["M12 21a9 9 0 100-18 9 9 0 000 18z", "M12 7v5l3 2"], { size: s }),
  activity: (s?: number) => make(["M22 12h-4l-3 9L9 3l-3 9H2"], { size: s }),
  cursor: (s?: number) => make(["M4 3l7.5 18 2.5-7.5L21.5 11z"], { size: s }),

  edit: (s?: number) =>
    make(["M12 20h9", "M16.5 3.5a2.1 2.1 0 013 3L7 19l-4 1 1-4z"], { size: s }),
  copy: (s?: number) =>
    make(
      [
        "M20 9H11a2 2 0 00-2 2v9a2 2 0 002 2h9a2 2 0 002-2v-9a2 2 0 00-2-2z",
        "M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1",
      ],
      { size: s },
    ),
  download: (s?: number) =>
    make(["M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4", "M7 10l5 5 5-5", "M12 15V3"], { size: s }),
  upload: (s?: number) =>
    make(["M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4", "M17 8l-5-5-5 5", "M12 3v12"], { size: s }),
  trash: (s?: number) =>
    make(
      [
        "M3 6h18",
        "M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2",
        "M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6",
        "M10 11v6",
        "M14 11v6",
      ],
      { size: s },
    ),

  pin: (s?: number) =>
    make(["M12 21s7-7.6 7-12a7 7 0 10-14 0c0 4.4 7 12 7 12z", "M12 11a2 2 0 100-4 2 2 0 000 4z"], {
      size: s,
    }),
  minimize: (s?: number) => make(["M5 12h14"], { size: s }),
  close: (s?: number) => make(["M6 6l12 12", "M18 6L6 18"], { size: s }),

  sun: (s?: number) =>
    make(
      [
        "M12 17a5 5 0 100-10 5 5 0 000 10z",
        "M12 1v2",
        "M12 21v2",
        "M4.2 4.2l1.4 1.4",
        "M18.4 18.4l1.4 1.4",
        "M1 12h2",
        "M21 12h2",
        "M4.2 19.8l1.4-1.4",
        "M18.4 5.6l1.4-1.4",
      ],
      { size: s },
    ),
  moon: (s?: number) => make(["M21 12.8A9 9 0 1111.2 3 7 7 0 0021 12.8z"], { size: s }),
};
