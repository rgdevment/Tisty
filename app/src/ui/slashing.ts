import {
  autocompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
} from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import type { EditorState } from "@codemirror/state";
import { EditorView, tooltips } from "@codemirror/view";
import { matched } from "../finding";
import { markup } from "../glyphs";
import { t } from "../locales";

type Said = Parameters<typeof t>[0];

export type Slashed = {
  readonly key: string;
  readonly label: Said;
  readonly hint: string;
  readonly icon: string;
  readonly put: Put;
};

type Put =
  | { line: string }
  | { block: string; at: number; blank?: boolean }
  | { inline: string; at: number };

const fenced = (lang: string): Put => ({ block: `\`\`\`${lang}\n\n\`\`\``, at: lang.length + 4 });

const CALLOUTS = [
  ["note", "info"],
  ["tip", "star"],
  ["important", "bell"],
  ["warning", "warning"],
  ["caution", "siren"],
] as const;

export const SLASHED: readonly Slashed[] = [
  { key: "h1", label: "bigTitle", hint: "#", icon: "heading1", put: { line: "# " } },
  { key: "h2", label: "midTitle", hint: "##", icon: "heading2", put: { line: "## " } },
  { key: "bullets", label: "bullets", hint: "-", icon: "bullets", put: { line: "- " } },
  { key: "numbers", label: "numbers", hint: "1.", icon: "numbers", put: { line: "1. " } },
  { key: "todo", label: "checks", hint: "[ ]", icon: "checks", put: { line: "- [ ] " } },
  { key: "quote", label: "quote", hint: ">", icon: "quote", put: { line: "> " } },
  ...CALLOUTS.map(([kind, icon]) => {
    const head = `> [!${kind.toUpperCase()}]\n> `;
    return {
      key: `callout-${kind}`,
      label: `said${kind}` as Said,
      hint: `[!${kind.toUpperCase()}]`,
      icon,
      put: { block: head, at: head.length },
    };
  }),
  { key: "mermaid", label: "diagramIt", hint: "```mermaid", icon: "graph", put: fenced("mermaid") },
  { key: "math", label: "formulaIt", hint: "```math", icon: "maths", put: fenced("math") },
  { key: "widget", label: "widgetIt", hint: "```widget", icon: "layout", put: fenced("widget") },
  { key: "code", label: "codeBlock", hint: "```", icon: "code", put: fenced("") },
  {
    key: "table",
    label: "table",
    hint: "|",
    icon: "table",
    put: { block: "|  |  |\n| --- | --- |\n|  |  |", at: 2, blank: true },
  },
  { key: "link", label: "linkIt", hint: "[ ]( )", icon: "link", put: { inline: "[]()", at: 1 } },
  { key: "pen", label: "penIt", hint: "==", icon: "highlight", put: { inline: "====", at: 2 } },
  {
    key: "rule",
    label: "leafBreak",
    hint: "---",
    icon: "rule",
    put: { block: "---\n", at: 4, blank: true },
  },
];

const CODE = new Set(["InlineCode", "FencedCode", "CodeBlock"]);

export const coded = (state: EditorState, at: number): boolean => {
  for (let node = syntaxTree(state).resolveInner(at, 1); node.parent; node = node.parent) {
    if (CODE.has(node.name)) return true;
  }
  return false;
};

// What goes before a block so it starts a line of its own, and a blank one where Markdown would glue it on.
const opening = (state: EditorState, at: number, blank: boolean): string => {
  const line = state.doc.lineAt(at);
  const before = state.sliceDoc(line.from, at);
  if (before.trim()) return blank ? "\n\n" : "\n";
  if (!blank || line.number === 1) return "";
  return state.doc.line(line.number - 1).text.trim() ? "\n" : "";
};

export const put = (view: EditorView, one: Slashed, from: number, to: number) => {
  const { state } = view;
  const how = one.put;
  if ("inline" in how) {
    view.dispatch({
      changes: { from, to, insert: how.inline },
      selection: { anchor: from + how.at },
      userEvent: "input.complete",
    });
    return;
  }
  if ("line" in how) {
    const ahead = opening(state, from, false);
    view.dispatch({
      changes: { from, to, insert: `${ahead}${how.line}` },
      selection: { anchor: from + ahead.length + how.line.length },
      userEvent: "input.complete",
    });
    return;
  }
  const ahead = opening(state, from, how.blank ?? false);
  // Words left after the slash would join a table, a quote or a fence, so they go below it, apart.
  const behind = state.sliceDoc(to, state.doc.lineAt(to).to).trim() ? "\n\n" : "";
  view.dispatch({
    changes: { from, to, insert: `${ahead}${how.block}${behind}` },
    selection: { anchor: from + ahead.length + how.at },
    userEvent: "input.complete",
  });
};

const ASKED = /(?:^|\s)\/[\p{L}\p{N}_-]*$/u;

export const offered = (context: CompletionContext): CompletionResult | null => {
  const found = context.matchBefore(ASKED);
  if (!found) return null;
  const slash = found.from + found.text.indexOf("/");
  if (coded(context.state, slash)) return null;
  const word = context.state.sliceDoc(slash + 1, context.pos);
  const shown = SLASHED.filter(
    (one) => !word || matched(t(one.label), word) || matched(one.key, word),
  );
  if (shown.length === 0) return null;
  return {
    from: slash,
    filter: false,
    options: shown.map(
      (one, rank): Completion => ({
        label: t(one.label),
        detail: one.hint,
        boost: -rank,
        apply: (view, _, from, to) => put(view, one, from, to),
        type: one.icon,
      }),
    ),
  };
};

const drawn = (completion: Completion, _: EditorState, view: EditorView): Node | null => {
  const path = completion.type ? markup(completion.type) : null;
  if (!path) return null;
  const svg = view.dom.ownerDocument.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("class", "cm-slash-glyph");
  svg.innerHTML = path;
  return svg;
};

// The page is as tall as the whole document, so the room a menu has is the part of it on screen.
const room = (view: EditorView) => {
  const page = view.dom.ownerDocument.defaultView;
  const frame = page?.frameElement?.getBoundingClientRect();
  const width = page?.innerWidth ?? 0;
  if (!frame) return { top: 0, left: 0, bottom: page?.innerHeight ?? 0, right: width };
  const seen = window.innerHeight;
  return {
    top: Math.max(0, -frame.top),
    left: 0,
    bottom: Math.min(frame.height, seen - frame.top),
    right: width,
  };
};

export const slashing = [
  autocompletion({
    override: [offered],
    icons: false,
    addToOptions: [{ render: drawn, position: 20 }],
    aboveCursor: false,
  }),
  tooltips({ tooltipSpace: room }),
  EditorView.theme({
    ".cm-tooltip.cm-tooltip-autocomplete": {
      border: "1px solid var(--tisty-hair)",
      borderRadius: "10px",
      background: "var(--tisty-rail)",
      boxShadow: "0 8px 24px var(--tisty-cast)",
      padding: "4px",
      fontFamily: "inherit",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul": {
      fontFamily: "inherit",
      maxHeight: "204px",
      minWidth: "212px",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul > li": {
      display: "flex",
      alignItems: "center",
      gap: "8px",
      padding: "5px 8px",
      borderRadius: "6px",
      fontSize: "12.5px",
      lineHeight: "1.4",
      color: "var(--tisty-ink)",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]": {
      background: "var(--tisty-accent-soft)",
      color: "var(--tisty-ink)",
    },
    ".cm-slash-glyph": {
      width: "15px",
      height: "15px",
      flexShrink: "0",
      fill: "none",
      stroke: "currentColor",
      strokeWidth: "2",
      strokeLinecap: "round",
      strokeLinejoin: "round",
      color: "var(--tisty-faint)",
    },
    ".cm-slash-glyph .solid": { fill: "currentColor", stroke: "none" },
    "li[aria-selected] .cm-slash-glyph": { color: "var(--tisty-accent)" },
    ".cm-completionLabel": { flex: "1" },
    ".cm-completionDetail": {
      marginLeft: "auto",
      fontStyle: "normal",
      fontFamily: "ui-monospace, 'Cascadia Mono', 'SF Mono', Menlo, monospace",
      fontSize: "10.5px",
      color: "var(--tisty-faint)",
    },
  }),
];
