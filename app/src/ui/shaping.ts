import type { Run, Shape } from "./paper";

interface Node {
  type: string;
  attrs?: Record<string, unknown>;
  content?: Node[];
  text?: string;
  marks?: { type: string; attrs?: Record<string, unknown> }[];
}

const inked = (nodes: Node[] | undefined): Run[] => {
  if (!nodes) return [];
  const runs: Run[] = [];
  for (const one of nodes) {
    if (one.type === "hardBreak") {
      runs.push({ text: "\n" });
      continue;
    }
    if (one.type === "image") continue;
    if (typeof one.text !== "string") {
      runs.push(...inked(one.content));
      continue;
    }
    const marks = one.marks ?? [];
    const pen = marks.find((m) => m.type === "highlight");
    runs.push({
      text: one.text,
      bold: marks.some((m) => m.type === "bold"),
      italic: marks.some((m) => m.type === "italic"),
      code: marks.some((m) => m.type === "code"),
      lit: pen ? ((pen.attrs?.color as string | undefined) ?? "yellow") : undefined,
      href: marks.find((m) => m.type === "link")?.attrs?.href as string | undefined,
    });
  }
  return runs;
};

const listed = (node: Node, deep: number, ordered: boolean, out: Shape[]): void => {
  let count = ordered ? Number(node.attrs?.start ?? 1) - 1 : 0;
  for (const item of node.content ?? []) {
    count += 1;
    const done = item.attrs?.checked === true;
    const mark = ordered ? `${count}.` : item.type === "taskItem" ? (done ? "☑" : "☐") : "•";
    const first = item.content?.[0];
    out.push({ kind: "bullet", mark, deep, runs: inked(first?.content) });
    for (const kid of (item.content ?? []).slice(1)) shape(kid, out, deep + 1);
  }
};

const shape = (node: Node, out: Shape[], deep = 0): void => {
  switch (node.type) {
    case "heading":
      out.push({
        kind: "heading",
        level: Number(node.attrs?.level ?? 1),
        runs: inked(node.content),
      });
      return;
    case "paragraph": {
      const picture = node.content?.find((one) => one.type === "image");
      if (picture) {
        out.push({
          kind: "image",
          src: String(picture.attrs?.src ?? ""),
          alt: String(picture.attrs?.alt ?? ""),
        });
      }
      const runs = inked(node.content);
      if (runs.length) {
        out.push({ kind: "para", runs, towards: node.attrs?.textAlign as string | undefined });
      }
      return;
    }
    case "image":
      out.push({
        kind: "image",
        src: String(node.attrs?.src ?? ""),
        alt: String(node.attrs?.alt ?? ""),
      });
      return;
    case "blockquote":
      out.push({ kind: "quote", runs: inked(node.content) });
      return;
    case "codeBlock":
      out.push({
        kind: "code",
        deep,
        runs: (node.content?.[0]?.text ?? "").split("\n").map((text) => ({ text })),
      });
      return;
    case "horizontalRule":
      out.push({ kind: "rule" });
      return;
    case "bulletList":
    case "taskList":
      listed(node, deep, false, out);
      return;
    case "orderedList":
      listed(node, deep, true, out);
      return;
    case "table": {
      const rows = (node.content ?? []).map((row) =>
        (row.content ?? []).map((cell) => inked(cell.content?.[0]?.content)),
      );
      if (rows.length) out.push({ kind: "table", rows });
      return;
    }
    default:
      for (const kid of node.content ?? []) shape(kid, out, deep);
  }
};

export const titled = (shapes: Shape[]): Shape[] => {
  const first = shapes[0];
  if (first?.kind !== "para") return shapes;
  return [{ kind: "heading", level: 1, runs: first.runs }, ...shapes.slice(1)];
};

export const shapesOf = (doc: unknown): Shape[] => {
  const out: Shape[] = [];
  const root = doc as Node | undefined;
  for (const node of root?.content ?? []) shape(node, out);
  return titled(out);
};

const KIND: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  svg: "image/svg+xml",
  avif: "image/avif",
};

export const asData = (bytes: number[], src: string): string => {
  const kind = KIND[src.split(".").pop()?.toLowerCase() ?? ""] ?? "image/png";
  let raw = "";
  for (const one of bytes) raw += String.fromCharCode(one);
  return `data:${kind};base64,${btoa(raw)}`;
};

export const fetched = async (
  shapes: Shape[],
  read: (reference: string) => Promise<number[]>,
): Promise<Shape[]> => {
  const held = new Map<string, string>();
  const out: Shape[] = [];

  for (const one of shapes) {
    if (one.kind !== "image" || /^(https?|data):/i.test(one.src)) {
      out.push(one);
      continue;
    }
    const seen = held.get(one.src);
    if (seen !== undefined) {
      out.push({ ...one, src: seen });
      continue;
    }
    try {
      const src = asData(await read(one.src), one.src);
      held.set(one.src, src);
      out.push({ ...one, src });
    } catch {
      held.set(one.src, "");
      out.push({ ...one, src: "" });
    }
  }
  return out;
};
