export type DiagramKind = "mindmap" | "erDiagram" | "flowchart" | "classDiagram" | "other";

const KINDS: Record<string, DiagramKind> = {
  mindmap: "mindmap",
  erdiagram: "erDiagram",
  flowchart: "flowchart",
  graph: "flowchart",
  classdiagram: "classDiagram",
};

// Front matter, `%%` comments and `%%{init}%%` come before the word that says what a diagram is.
export const kindOf = (source: string): DiagramKind => {
  let inFront = false;
  for (const raw of source.split("\n")) {
    const line = raw.trim();
    if (line === "---") {
      inFront = !inFront;
      continue;
    }
    if (inFront || !line || line.startsWith("%%")) continue;
    const word = line.split(/[\s;]/)[0].toLowerCase();
    return KINDS[word] ?? "other";
  }
  return "other";
};

/** What the diagram says once written, as Rust reads it back: every line ends with a break. */
export const asRead = (text: string): string =>
  text
    .replace(/\r/g, "")
    .split("\n")
    .map((line, at, all) => (at === all.length - 1 && line === "" ? "" : `${line}\n`))
    .join("");

export interface View {
  scale: number;
  x: number;
  y: number;
}

export interface Size {
  wide: number;
  tall: number;
}

export const LEAST = 0.25;
export const MOST = 4;
export const STEP = 1.2;

const held = (scale: number) => Math.min(MOST, Math.max(LEAST, scale));

/** Zooms keeping still the point under the pointer, so the eye does not lose its place. */
export const zoomedAt = (view: View, by: number, point: { x: number; y: number }): View => {
  const scale = held(view.scale * by);
  const grown = scale / view.scale;
  return {
    scale,
    x: point.x - (point.x - view.x) * grown,
    y: point.y - (point.y - view.y) * grown,
  };
};

// A small diagram is shown at its own size rather than blown up to fill the room.
export const fitted = (drawn: Size, room: Size, margin = 32): View => {
  if (drawn.wide <= 0 || drawn.tall <= 0) return { scale: 1, x: margin, y: margin };
  const scale = held(
    Math.min(1, (room.wide - margin * 2) / drawn.wide, (room.tall - margin * 2) / drawn.tall),
  );
  return {
    scale,
    x: (room.wide - drawn.wide * scale) / 2,
    y: (room.tall - drawn.tall * scale) / 2,
  };
};

/** The size Mermaid drew at, read from the SVG's own `viewBox`. */
export const sizeOf = (svg: string): Size | null => {
  const box = /viewBox="\s*[-\d.]+[\s,]+[-\d.]+[\s,]+([\d.]+)[\s,]+([\d.]+)\s*"/.exec(svg);
  if (!box) return null;
  const [wide, tall] = [Number(box[1]), Number(box[2])];
  return wide > 0 && tall > 0 ? { wide, tall } : null;
};
