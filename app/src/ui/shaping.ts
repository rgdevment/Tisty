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
    runs.push({
      text: one.text,
      bold: marks.some((m) => m.type === "bold"),
      italic: marks.some((m) => m.type === "italic"),
      code: marks.some((m) => m.type === "code"),
      href: marks.find((m) => m.type === "link")?.attrs?.href as string | undefined,
    });
  }
  return runs;
};

const listed = (node: Node, deep: number, ordered: boolean, out: Shape[]): void => {
  let count = 0;
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
      if (picture) out.push({ kind: "image", src: String(picture.attrs?.src ?? "") });
      const runs = inked(node.content);
      if (runs.length) out.push({ kind: "para", runs });
      return;
    }
    case "image":
      out.push({ kind: "image", src: String(node.attrs?.src ?? "") });
      return;
    case "blockquote":
      out.push({ kind: "quote", runs: inked(node.content) });
      return;
    case "codeBlock":
      out.push({
        kind: "code",
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
    case "table":
      for (const row of node.content ?? []) {
        const cells = (row.content ?? []).map((cell) =>
          inked(cell.content?.[0]?.content)
            .map((run) => run.text)
            .join(""),
        );
        out.push({ kind: "para", runs: [{ text: cells.join("   ·   ") }] });
      }
      return;
    default:
      for (const kid of node.content ?? []) shape(kid, out, deep);
  }
};

export const titled = (shapes: Shape[]): Shape[] => {
  const first = shapes[0];
  if (!first || first.kind !== "para") return shapes;
  return [{ kind: "heading", level: 1, runs: first.runs }, ...shapes.slice(1)];
};

export const shapesOf = (doc: unknown): Shape[] => {
  const out: Shape[] = [];
  const root = doc as Node | undefined;
  for (const node of root?.content ?? []) shape(node, out);
  return titled(out);
};
