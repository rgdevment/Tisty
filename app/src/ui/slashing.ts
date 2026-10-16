import {
  autocompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
} from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { type EditorState, Prec } from "@codemirror/state";
import { EditorView, type KeyBinding, keymap, tooltips } from "@codemirror/view";
import { matched } from "../finding";
import { markup } from "../glyphs";
import { t } from "../locales";

type Said = Parameters<typeof t>[0];

export type Asked = "newPage" | "page" | "newDoc" | "doc" | "icon";

export type Asking = {
  readonly kinds: readonly Asked[];
  readonly ask: (kind: Asked, at: { x: number; y: number }, put: (text: string) => void) => void;
};

type Put =
  | { line: string }
  | { block: string; at: number; blank?: boolean }
  | { inline: string; at: number }
  | { wrap: string; close?: string }
  | { ask: Asked; block: boolean };

export type Slashed = {
  readonly key: string;
  readonly label: Said;
  readonly hint: string;
  readonly icon: string;
  readonly put: Put;
  readonly keys?: string;
};

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
  { key: "h3", label: "smallTitle", hint: "###", icon: "heading", put: { line: "### " } },
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
  { key: "bold", label: "bold", hint: "**", icon: "bold", put: { wrap: "**" }, keys: "B" },
  { key: "italic", label: "italic", hint: "*", icon: "italic", put: { wrap: "*" }, keys: "I" },
  {
    key: "underline",
    label: "underlined",
    hint: "++",
    icon: "underline",
    put: { wrap: "++" },
    keys: "U",
  },
  {
    key: "strike",
    label: "struck",
    hint: "~~",
    icon: "strike",
    put: { wrap: "~~" },
    keys: "Shift X",
  },
  { key: "inline", label: "codeSpan", hint: "`", icon: "code", put: { wrap: "`" }, keys: "E" },
  {
    key: "link",
    label: "linkIt",
    hint: "[ ]( )",
    icon: "link",
    put: { wrap: "[", close: "]()" },
    keys: "K",
  },
  { key: "pen", label: "penIt", hint: "==", icon: "highlight", put: { wrap: "==" } },
  {
    key: "icon",
    label: "insertIcon",
    hint: "::",
    icon: "emoji",
    put: { ask: "icon", block: false },
  },
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
  {
    key: "rule",
    label: "leafBreak",
    hint: "---",
    icon: "rule",
    put: { block: "---\n", at: 4, blank: true },
  },
  {
    key: "newpage",
    label: "insertNewPage",
    hint: "+",
    icon: "pages",
    put: { ask: "newPage", block: true },
  },
  {
    key: "page",
    label: "insertPage",
    hint: "![ ]",
    icon: "pages",
    put: { ask: "page", block: true },
  },
  { key: "doc", label: "insertDoc", hint: "![ ]", icon: "page", put: { ask: "doc", block: true } },
  {
    key: "newdoc",
    label: "insertNewDoc",
    hint: "+",
    icon: "plus",
    put: { ask: "newDoc", block: true },
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

// Words left after the slash would join a table, a quote or a fence, so they go below it, apart.
const closing = (state: EditorState, at: number): string =>
  state.sliceDoc(at, state.doc.lineAt(at).to).trim() ? "\n\n" : "";

const blocked = (
  view: EditorView,
  from: number,
  to: number,
  text: string,
  at: number,
  blank: boolean,
) => {
  const { state } = view;
  const ahead = opening(state, from, blank);
  view.dispatch({
    changes: { from, to, insert: `${ahead}${text}${closing(state, to)}` },
    selection: { anchor: from + ahead.length + at },
    userEvent: "input.complete",
  });
};

export const wrapped = (view: EditorView, mark: string, close = mark): boolean => {
  const { from, to } = view.state.selection.main;
  const inside = view.state.sliceDoc(from, to);
  view.dispatch({
    changes: { from, to, insert: `${mark}${inside}${close}` },
    selection: inside
      ? { anchor: from + mark.length, head: from + mark.length + inside.length }
      : { anchor: from + mark.length },
    userEvent: "input.format",
  });
  return true;
};

// The window's own pickers sit outside the page, so the place they open at is the caret's on screen.
const onScreen = (view: EditorView, at: number) => {
  const caret = view.coordsAtPos(at);
  const frame = view.dom.ownerDocument.defaultView?.frameElement?.getBoundingClientRect();
  return {
    x: (caret?.left ?? 0) + (frame?.left ?? 0),
    y: (caret?.bottom ?? 0) + (frame?.top ?? 0),
  };
};

export const put = (view: EditorView, one: Slashed, from: number, to: number, asking?: Asking) => {
  const { state } = view;
  const how = one.put;
  if ("ask" in how) {
    view.dispatch({ changes: { from, to }, userEvent: "input.complete" });
    asking?.ask(how.ask, onScreen(view, from), (text) => {
      if (!view.dom.isConnected) return;
      const at = Math.min(from, view.state.doc.length);
      if (how.block) blocked(view, at, at, text, text.length, true);
      else
        view.dispatch({
          changes: { from: at, insert: text },
          selection: { anchor: at + text.length },
        });
      view.focus();
    });
    return;
  }
  if ("wrap" in how) {
    view.dispatch({
      changes: { from, to },
      selection: { anchor: from },
      userEvent: "input.complete",
    });
    wrapped(view, how.wrap, how.close);
    return;
  }
  if ("inline" in how) {
    view.dispatch({
      changes: { from, to, insert: how.inline },
      selection: { anchor: from + how.inline.length },
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
  blocked(view, from, to, how.block, how.at, how.blank ?? false);
};

const mac = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform);
const pressed = (keys: string): string => `${mac ? "⌘" : "Ctrl"} ${keys}`;

const ASKED = /(?:^|\s)\/[\p{L}\p{N}_-]*$/u;

export const offered =
  (asking?: Asking) =>
  (context: CompletionContext): CompletionResult | null => {
    const found = context.matchBefore(ASKED);
    if (!found) return null;
    const slash = found.from + found.text.indexOf("/");
    if (coded(context.state, slash)) return null;
    const word = context.state.sliceDoc(slash + 1, context.pos);
    const shown = SLASHED.filter(
      (one) =>
        (!("ask" in one.put) || asking?.kinds.includes(one.put.ask)) &&
        (!word || matched(t(one.label), word) || matched(one.key, word)),
    );
    if (shown.length === 0) return null;
    return {
      from: slash,
      filter: false,
      options: shown.map(
        (one, rank): Completion => ({
          label: t(one.label),
          detail: one.keys ? `${one.hint}   ${pressed(one.keys)}` : one.hint,
          boost: -rank,
          apply: (view, _, from, to) => put(view, one, from, to, asking),
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
  return {
    top: Math.max(0, -frame.top),
    left: 0,
    bottom: Math.min(frame.height, window.innerHeight - frame.top),
    right: width,
  };
};

const formats: readonly KeyBinding[] = [
  { key: "Mod-b", run: (view) => wrapped(view, "**") },
  { key: "Mod-i", run: (view) => wrapped(view, "*") },
  { key: "Mod-u", run: (view) => wrapped(view, "++") },
  { key: "Mod-e", run: (view) => wrapped(view, "`") },
  { key: "Mod-Shift-x", run: (view) => wrapped(view, "~~") },
  { key: "Mod-k", run: (view) => wrapped(view, "[", "]()") },
];

const dressed = () =>
  EditorView.theme({
    ".cm-tooltip.cm-tooltip-autocomplete": {
      border: "1px solid var(--tisty-hair)",
      borderRadius: "10px",
      background: "var(--tisty-rail)",
      boxShadow: "0 8px 24px var(--tisty-cast)",
      padding: "4px",
      fontFamily: "inherit",
    },
    ".cm-tooltip.cm-tooltip-autocomplete::after": {
      content: JSON.stringify(t("slashKeys")),
      display: "block",
      borderTop: "1px solid var(--tisty-hair)",
      margin: "4px 2px 0",
      padding: "5px 6px 2px",
      fontSize: "10.5px",
      lineHeight: "1.3",
      color: "var(--tisty-faint)",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul": {
      fontFamily: "inherit",
      maxHeight: "260px",
      minWidth: "240px",
      scrollbarWidth: "thin",
      scrollbarColor: "var(--tisty-hair) transparent",
      overscrollBehavior: "contain",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul::-webkit-scrollbar": { width: "6px" },
    ".cm-tooltip.cm-tooltip-autocomplete > ul::-webkit-scrollbar-thumb": {
      background: "var(--tisty-hair)",
      borderRadius: "3px",
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
      paddingLeft: "12px",
      whiteSpace: "pre",
      fontStyle: "normal",
      fontFamily: "ui-monospace, 'Cascadia Mono', 'SF Mono', Menlo, monospace",
      fontSize: "10.5px",
      color: "var(--tisty-faint)",
    },
  });

export const slashing = (asking?: Asking) => [
  autocompletion({
    override: [offered(asking)],
    icons: false,
    addToOptions: [{ render: drawn, position: 20 }],
    aboveCursor: false,
  }),
  Prec.high(keymap.of(formats)),
  tooltips({ tooltipSpace: room }),
  dressed(),
];
