import { syntaxTree } from "@codemirror/language";
import type { EditorState } from "@codemirror/state";
import type { Tree, TreeCursor } from "@lezer/common";

export type Kind = "drawn" | "turn" | "unseen";
export type Rich = { readonly from: number; readonly to: number; readonly kind: Kind };
export type Code = {
  readonly open: { from: number; to: number };
  readonly shut: { from: number; to: number } | null;
  readonly said: string;
};

const DRAWN_FENCES = new Set(["mermaid", "mmd", "math", "widget"]);
const IDEA = /^\s*>\s*\[!idea\]/i;
const COMMENT = /^<!--[\s\S]*-->$/;
const FORMULA = /^\$\$[\s\S]*\$\$$/;
const FENCE = /^\s*(`{3,}|~{3,})\s*$/;

const pictured = (state: EditorState, cursor: TreeCursor): boolean => {
  const node = cursor.node;
  let pictures = 0;
  for (let child = node.firstChild; child; child = child.nextSibling) {
    if (child.name !== "Image") return false;
    pictures++;
  }
  const rest = state.sliceDoc(node.from, node.to).replace(/!\[[^\]]*\]\([^)]*\)/g, "");
  return pictures > 0 && !rest.trim();
};

const languageOf = (state: EditorState, cursor: TreeCursor): string => {
  const info = cursor.node.getChild("CodeInfo");
  return info ? state.sliceDoc(info.from, info.to).trim().toLowerCase() : "";
};

const kindOf = (state: EditorState, cursor: TreeCursor): Kind | "code" | null => {
  const text = () => state.doc.sliceString(cursor.from, cursor.to).trim();
  if (cursor.name === "Table") return "drawn";
  if (cursor.name === "HorizontalRule") return "turn";
  if (cursor.name === "CommentBlock") return "unseen";
  if (cursor.name === "HTMLBlock") return COMMENT.test(text()) ? "unseen" : "drawn";
  if (cursor.name === "FencedCode")
    return DRAWN_FENCES.has(languageOf(state, cursor)) ? "drawn" : "code";
  if (cursor.name === "Blockquote")
    return IDEA.test(state.doc.lineAt(cursor.from).text) ? "drawn" : null;
  if (cursor.name !== "Paragraph") return null;
  return pictured(state, cursor) || FORMULA.test(text()) ? "drawn" : null;
};

const lastLine = (state: EditorState, to: number) =>
  state.doc.lineAt(to > 0 && state.doc.sliceString(to - 1, to) === "\n" ? to - 1 : to);

type Found = { readonly rich: Rich[]; readonly code: Code[] };

const known = new WeakMap<Tree, Found>();

export const blocksDrawn = (state: EditorState): Found => {
  const tree = syntaxTree(state);
  const kept = known.get(tree);
  if (kept) return kept;
  const found: Found = { rich: [], code: [] };
  known.set(tree, found);
  const cursor = tree.topNode.cursor();
  if (!cursor.firstChild()) return found;
  do {
    const kind = kindOf(state, cursor);
    if (!kind) continue;
    const first = state.doc.lineAt(cursor.from);
    const last = lastLine(state, cursor.to);
    if (kind !== "code") {
      found.rich.push({ from: first.from, to: last.to, kind });
      continue;
    }
    const closed = last.number > first.number && FENCE.test(last.text);
    found.code.push({
      open: { from: first.from, to: first.to },
      shut: closed ? { from: last.from, to: last.to } : null,
      said: first.text.replace(/^\s*(`{3,}|~{3,})/, "").trim(),
    });
  } while (cursor.nextSibling());
  return found;
};

export const richIn = (state: EditorState): Rich[] => blocksDrawn(state).rich;

// Each line of the code keeps its own end, as the core reads a diagram.
export const codeOf = (source: string): string => {
  const inner = source
    .replace(/\r/g, "")
    .replace(/^\s*(`{3,}|~{3,})[^\n]*\n/, "")
    .replace(/\n?\s*(`{3,}|~{3,})\s*$/, "");
  return inner ? `${inner}\n` : "";
};

const DIAGRAM = /^\s*(`{3,}|~{3,})\s*(mermaid|mmd)\b/i;

export const diagramsBefore = (state: EditorState, at: number): number =>
  blocksDrawn(state).rich.filter(
    (one) => one.from < at && DIAGRAM.test(state.doc.lineAt(one.from).text),
  ).length;
