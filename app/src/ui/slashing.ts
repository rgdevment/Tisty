import {
  autocompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
  completionStatus,
  startCompletion,
} from "@codemirror/autocomplete";
import { type EditorState, Prec, StateEffect, StateField } from "@codemirror/state";
import { EditorView, type KeyBinding, keymap, tooltips } from "@codemirror/view";
import { matched } from "../finding";
import { markup } from "../glyphs";
import { t } from "../locales";
import { anchored, coded, linking, TAG_EDGE, TAG_TAIL, TAG_WORD, wrapped } from "./formatting";

type Said = Parameters<typeof t>[0];

export type Asked = "newPage" | "page" | "newDoc" | "doc" | "icon" | "file";

export type Asking = {
  readonly kinds: readonly Asked[];
  readonly tags?: readonly string[];
  readonly pasted?: (files: readonly File[]) => Promise<string>;
  readonly lost?: () => void;
  readonly failed?: (problem: unknown) => void;
  readonly steps?: () => readonly string[];
  readonly ask: (
    kind: Asked,
    at: { x: number; y: number },
    put: (text: string) => boolean,
    first: boolean,
  ) => void;
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
  {
    key: "inline",
    label: "codeSpan",
    hint: "`",
    icon: "code",
    put: { wrap: "`" },
    keys: "Shift C",
  },
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
    key: "step",
    label: "insertStep",
    hint: "[[#]]",
    icon: "checks",
    put: { inline: "[[#", at: 3 },
  },
  {
    key: "icon",
    label: "insertIcon",
    hint: "::",
    icon: "emoji",
    put: { ask: "icon", block: false },
  },
  {
    key: "file",
    label: "attachFile",
    hint: "+",
    icon: "file-add",
    put: { ask: "file", block: true },
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
    key: "fold",
    label: "foldSection",
    hint: ":::",
    icon: "list-down",
    put: { block: ":::details \n\n\n\n:::", at: 11, blank: true },
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

// What goes before a block so it starts a line of its own, and a blank one where Markdown would glue it on.
const opening = (state: EditorState, at: number, blank: boolean): string => {
  const line = state.doc.lineAt(at);
  const before = state.sliceDoc(line.from, at);
  if (before.trim()) return blank ? "\n\n" : "\n";
  if (!blank || line.number === 1) return "";
  return state.doc.line(line.number - 1).text.trim() ? "\n" : "";
};

const spacesBefore = (state: EditorState, at: number) => {
  const before = state.sliceDoc(state.doc.lineAt(at).from, at);
  return before.length - before.trimEnd().length;
};

const spacesAfter = (state: EditorState, at: number) => {
  const after = state.sliceDoc(at, state.doc.lineAt(at).to);
  return after.length - after.trimStart().length;
};

// Words left after the slash would join a table, a quote or a fence, so they go below it, apart.
const closing = (state: EditorState, at: number): string =>
  state.sliceDoc(at, state.doc.lineAt(at).to).trim() ? "\n\n" : "";

export const putBlock = (view: EditorView, text: string) => {
  const { from, to } = view.state.selection.main;
  blocked(view, from, to, text, text.length, true);
};

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
  const start = ahead ? from - spacesBefore(state, from) : from;
  const after = closing(state, to);
  const end = after ? to + spacesAfter(state, to) : to;
  view.dispatch({
    changes: { from: start, to: end, insert: `${ahead}${text}${after}` },
    selection: { anchor: start + ahead.length + at },
    userEvent: "input.complete",
  });
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

// What was selected when the menu was opened from the bar, so a format picked there wraps it.
const held = new WeakMap<EditorView, { from: number; to: number; slash: number }>();

export const holding = (view: EditorView, chosen: { from: number; to: number; slash: number }) =>
  held.set(view, chosen);

export const put = (view: EditorView, one: Slashed, from: number, to: number, asking?: Asking) => {
  const { state } = view;
  const how = one.put;
  const chosen = held.get(view);
  held.delete(view);
  if (chosen?.slash === from && "wrap" in how) {
    view.dispatch({
      changes: { from: chosen.to, to },
      selection: { anchor: chosen.from, head: chosen.to },
      userEvent: "input.complete",
    });
    wrapped(view, how.wrap, how.close);
    return;
  }
  if ("ask" in how) {
    view.dispatch({ changes: { from, to }, userEvent: "input.complete" });
    const first = !view.state.sliceDoc(0, from).trim();
    asking?.ask(
      how.ask,
      onScreen(view, from),
      (text) => {
        if (!view.dom.isConnected) return false;
        const at = Math.min(from, view.state.doc.length);
        if (how.block) blocked(view, at, at, text, text.length, true);
        else
          view.dispatch({
            changes: { from: at, insert: text },
            selection: { anchor: at + text.length },
          });
        view.focus();
        return true;
      },
      first,
    );
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
    // An empty item cannot break into words above it, so a list or quote starts apart from them.
    const ahead = opening(state, from, /^([-*+>]|\d)/.test(how.line));
    const start = ahead ? from - spacesBefore(state, from) : from;
    const end = to + spacesAfter(state, to);
    view.dispatch({
      changes: { from: start, to: end, insert: `${ahead}${how.line}` },
      selection: { anchor: start + ahead.length + how.line.length },
      userEvent: "input.complete",
    });
    return;
  }
  blocked(view, from, to, how.block, how.at, how.blank ?? false);
};

const mac = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform);
export const pressed = (keys: string): string => `${mac ? "⌘" : "Ctrl"} ${keys}`;

const ASKED = /(?:^|\s)\/[\p{L}\p{N}_-]*$/u;
const WORD = /^[\p{L}\p{N}_-]*$/u;

const opened = StateEffect.define<number>();

// The menu asked for without a slash lives while what follows its place is still one word.
const openedAt = StateField.define<number | null>({
  create: () => null,
  update(at, tr) {
    for (const one of tr.effects) if (one.is(opened)) return one.value;
    if (at === null || tr.isUserEvent("input.complete")) return null;
    if (completionStatus(tr.startState) && !completionStatus(tr.state)) return null;
    const next = tr.changes.mapPos(at);
    const head = tr.state.selection.main.head;
    return head >= next && WORD.test(tr.state.sliceDoc(next, head)) ? next : null;
  },
});

export const openMenu = (view: EditorView): boolean => {
  const { from, to } = view.state.selection.main;
  if (from < to) holding(view, { from, to, slash: to });
  else held.delete(view);
  view.dispatch({ effects: opened.of(to), selection: { anchor: to } });
  view.focus();
  startCompletion(view);
  return true;
};

const askedHere = (context: CompletionContext): { from: number; word: number } | null => {
  const at = context.state.field(openedAt, false);
  if (at !== null && at !== undefined) return { from: at, word: at };
  const found = context.matchBefore(ASKED);
  if (!found) return null;
  const slash = found.from + found.text.indexOf("/");
  return { from: slash, word: slash + 1 };
};

export const offered =
  (asking?: () => Asking | undefined) =>
  (context: CompletionContext): CompletionResult | null => {
    const asked = askedHere(context);
    if (!asked || coded(context.state, asked.from)) return null;
    const slash = asked.from;
    const word = context.state.sliceDoc(asked.word, context.pos);
    const shown = SLASHED.filter(
      (one) =>
        (!("ask" in one.put) || asking?.()?.kinds.includes(one.put.ask)) &&
        (one.key !== "step" || (asking?.()?.steps?.().length ?? 0) > 0) &&
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
          apply: (view, _, from, to) => {
            put(view, one, from, to, asking?.());
            // A step is named by its number, so the list of them opens right where it is asked for.
            if (one.key === "step") startCompletion(view);
          },
          type: one.icon,
        }),
      ),
    };
  };

const STEPPED = /\[\[#\d*$/;

export const BOX = /^[\s>]*(?:[-*+]|\d+[.)])\s+\[[ xX]\]\s+(.*)$/;

// The boxes a page holds, counted as they are read, which is how `[[#2]]` names the second.
export const stagesIn = (text: string): string[] => {
  const out: string[] = [];
  let fence: string | null = null;
  for (const line of text.split(/\r?\n/)) {
    const mark = /^\s*(```|~~~)/.exec(line)?.[1] ?? null;
    if (mark && (fence === null || fence === mark)) {
      fence = fence === null ? mark : null;
      continue;
    }
    const box = fence ? null : BOX.exec(line);
    if (box) out.push(box[1].trim());
  }
  return out;
};

export const stepping =
  (asking?: () => Asking | undefined) =>
  (context: CompletionContext): CompletionResult | null => {
    const found = context.matchBefore(STEPPED);
    const steps = found ? (asking?.()?.steps?.() ?? []) : [];
    if (!found || steps.length === 0) return null;
    const typed = found.text.slice(3);
    const options = steps
      .map(
        (text, at): Completion => ({
          label: `${at + 1}`,
          detail: text,
          apply: `[[#${at + 1}]]`,
          boost: -at,
        }),
      )
      .filter((one) => one.label.startsWith(typed));
    return { from: found.from, filter: false, options };
  };

const TAGGED = new RegExp(`${TAG_EDGE}#${TAG_WORD}$`, "u");
const REST = new RegExp(`^${TAG_TAIL}*`, "u");

const plain = (word: string) => word.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

// Tags already in use are offered as a word after # is typed, so one tag is never spelled two ways.
export const tagging =
  (asking?: () => Asking | undefined) =>
  (context: CompletionContext): CompletionResult | null => {
    const found = context.matchBefore(TAGGED);
    if (!found) return null;
    const hash = found.from;
    const word = found.text.slice(1);
    if (!/\p{L}/u.test(word) || coded(context.state, hash) || anchored(context.state, hash))
      return null;
    const key = plain(word);
    const tags = asking?.()?.tags ?? [];
    if (tags.includes(key)) return null;
    const shown = tags.filter((one) => one.startsWith(key));
    if (shown.length === 0) return null;
    const line = context.state.doc.lineAt(context.pos);
    const to =
      context.pos + (REST.exec(context.state.sliceDoc(context.pos, line.to))?.[0].length ?? 0);
    // What was typed comes first, so Enter keeps a new tag instead of swapping it for a longer one.
    return {
      from: hash,
      to,
      filter: false,
      options: [context.state.sliceDoc(hash + 1, to), ...shown.slice(0, 30)].map((one) => ({
        label: `#${one}`,
        apply: `#${one}`,
        type: "tag",
      })),
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

const formats: readonly KeyBinding[] = SLASHED.flatMap((one) => {
  const how = one.put;
  if (!one.keys || !("wrap" in how)) return [];
  const parts = one.keys.split(" ");
  const key = ["Mod", ...parts.slice(0, -1), parts[parts.length - 1].toLowerCase()].join("-");
  return [{ key, run: (view: EditorView) => wrapped(view, how.wrap, how.close) }];
});

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

export const slashing = (asking?: () => Asking | undefined) => [
  autocompletion({
    override: [offered(asking), tagging(asking), stepping(asking)],
    icons: false,
    addToOptions: [{ render: drawn, position: 20 }],
    aboveCursor: false,
  }),
  openedAt,
  Prec.high(keymap.of([...formats, linking, { key: "Mod-/", run: openMenu }])),
  tooltips({ tooltipSpace: room }),
  dressed(),
];
