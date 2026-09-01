import { fill, t, type Word } from "../locales";
import { DOC } from "../markdown";
import { KINDS } from "../previews";
import { redrawn, upright } from "../upright";
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
        out.push({ kind: "para", runs });
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
    case "callout": {
      const inner: Shape[] = [];
      for (const kid of node.content ?? []) shape(kid, inner, deep);
      out.push({ kind: "said", said: String(node.attrs?.kind ?? "note"), inner });
      return;
    }
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
      const leans = (node.content?.[0]?.content ?? []).map(
        (cell) => (cell.attrs?.textAlign as string | undefined) ?? null,
      );
      if (rows.length) out.push({ kind: "table", rows, leans });
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

const PRINTABLE = ["png", "jpg", "jpeg"];

const named = (src: string): string => {
  const leaf = src.split(/[?#]/)[0].split("/").pop() ?? src;
  try {
    return decodeURI(leaf);
  } catch {
    return leaf;
  }
};

const kindOf = (src: string): string => {
  const kind = src.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "";
  return KINDS[kind] ? t(KINDS[kind] as Word) : kind.toUpperCase();
};

const pictured = (bytes: number[]): boolean => {
  const png = [137, 80, 78, 71];
  const jpeg = [255, 216, 255];
  return (
    png.every((byte, at) => bytes[at] === byte) || jpeg.every((byte, at) => bytes[at] === byte)
  );
};

const printable = (src: string): boolean =>
  PRINTABLE.includes(src.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "");

export const fetched = async (
  shapes: Shape[],
  read: (reference: string) => Promise<number[]>,
  leaf?: (file: string) => number | null,
): Promise<Shape[]> => {
  const held = new Map<string, string | null>();
  const out: Shape[] = [];

  const carded = (one: { src: string; alt?: string }): Shape => ({
    kind: "file",
    name: one.alt || named(one.src),
    said: kindOf(one.src),
  });

  for (const one of shapes) {
    if (one.kind !== "image" || /^(https?|data):/i.test(one.src)) {
      out.push(one);
      continue;
    }
    if (one.src.startsWith(DOC)) {
      const file = one.src.slice(DOC.length);
      const at = leaf?.(file) ?? null;
      out.push({
        kind: "file",
        name: one.alt || file,
        said: at === null ? t("kindDoc") : fill("kindLeaf", String(at)),
      });
      continue;
    }
    if (!printable(one.src)) {
      out.push(carded(one));
      continue;
    }
    const seen = held.get(one.src);
    if (seen !== undefined) {
      out.push(seen === null ? carded(one) : { ...one, src: seen });
      continue;
    }
    try {
      const bytes = await read(one.src);
      if (!pictured(bytes)) {
        held.set(one.src, null);
        out.push(carded(one));
        continue;
      }
      const drawn = asData(bytes, one.src);
      const src = redrawn(bytes) ? await upright(drawn) : drawn;
      held.set(one.src, src);
      out.push({ ...one, src });
    } catch {
      held.set(one.src, "");
      out.push({ ...one, src: "" });
    }
  }
  return out;
};
