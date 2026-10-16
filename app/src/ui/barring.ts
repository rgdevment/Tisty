import { startCompletion } from "@codemirror/autocomplete";
import type { EditorView, KeyBinding } from "@codemirror/view";
import { markup } from "../glyphs";
import { t } from "../locales";
import { wrapped } from "./formatting";
import { marksShown, showMarks } from "./seeing";
import { pressed } from "./slashing";

type Said = Parameters<typeof t>[0];

export const KINDS: readonly (readonly [Said, string])[] = [
  ["paragraph", ""],
  ["bigTitle", "# "],
  ["midTitle", "## "],
  ["smallTitle", "### "],
  ["bullets", "- "],
  ["numbers", "1. "],
  ["checks", "- [ ] "],
  ["quote", "> "],
];

const LEAD = /^(\s*)(#{1,6} |[-*+] \[[ xX]\] |[-*+] |\d+[.)] |> ?)?/;

export const kindOf = (line: string): Said => {
  const lead = LEAD.exec(line)?.[2] ?? "";
  if (lead.startsWith("#"))
    return lead.length === 2 ? "bigTitle" : lead.length === 3 ? "midTitle" : "smallTitle";
  if (lead.includes("[")) return "checks";
  if (/^[-*+]/.test(lead)) return "bullets";
  if (/^\d/.test(lead)) return "numbers";
  if (lead.startsWith(">")) return "quote";
  return "paragraph";
};

// Every line the selection touches takes the new kind, so a list is turned in one go.
export const retyped = (view: EditorView, prefix: string): boolean => {
  const { state } = view;
  const { from, to } = state.selection.main;
  const changes = [];
  const last = state.doc.lineAt(to);
  const end = to > from && to === last.from ? last.number - 1 : last.number;
  for (let at = state.doc.lineAt(from).number; at <= end; at++) {
    const line = state.doc.line(at);
    const [, space = "", lead = ""] = LEAD.exec(line.text) ?? [];
    // Only an item keeps its indent: an indented title, quote or paragraph would read as code.
    const kept = /^([-*+]|\d)/.test(prefix) ? space.length : 0;
    changes.push({
      from: line.from + kept,
      to: line.from + space.length + lead.length,
      insert: prefix,
    });
  }
  view.dispatch({ changes, userEvent: "input.type" });
  view.focus();
  return true;
};

const BARE = /^((> ?)+|(> ?)*([-*+] (\[[ xX]\] )?|\d+[.)] ))\s*$/;
const LEADS = /^(?:>\s?|#{1,6}\s|[-*+]\s(?:\[[ xX]\]\s)?|\d+[.)]\s)+/;

export const blockKeys: KeyBinding[] = [
  {
    // An empty item ends the list with a blank line, so the words after it never join the last item.
    key: "Enter",
    run: (view) => {
      const { main } = view.state.selection;
      const line = view.state.doc.lineAt(main.head);
      if (!main.empty || main.head !== line.to || line.number === 1 || !BARE.test(line.text))
        return false;
      view.dispatch({
        changes: { from: line.from, to: line.to, insert: "\n" },
        selection: { anchor: line.from + 1 },
        userEvent: "input",
      });
      return true;
    },
  },
  {
    // Backspace right after a title's, item's or quote's mark turns the line back into words.
    key: "Backspace",
    run: (view) => {
      const { main } = view.state.selection;
      const line = view.state.doc.lineAt(main.head);
      const lead = LEADS.exec(line.text)?.[0].length ?? 0;
      if (!main.empty || !lead || main.head !== line.from + lead) return false;
      // A callout's name goes with its kind menu, never with a stray key.
      if (/^\[![a-z]+\]/i.test(line.text.slice(lead))) return true;
      view.dispatch({ changes: { from: line.from, to: line.from + lead }, userEvent: "delete" });
      return true;
    },
  },
];

export const flipped = (view: EditorView): boolean => {
  view.dispatch({ effects: showMarks.of(!view.state.field(marksShown)) });
  return true;
};

const PENS = [
  ["yellow", "penIt", "=="],
  ["green", "penGreen", "=={green}"],
  ["blue", "penBlue", "=={blue}"],
  ["pink", "penPink", "=={pink}"],
] as const;

const FORMATS = [
  ["bold", "B", "**", "**", "bold"],
  ["italic", "I", "*", "*", "italic"],
  ["underlined", "U", "++", "++", "underline"],
  ["struck", "Shift X", "~~", "~~", "strike"],
  ["codeSpan", "E", "`", "`", "code"],
  ["linkIt", "K", "[", "]()", "link"],
] as const;

export type Barred = {
  readonly bar: HTMLElement;
  readonly hint: HTMLElement;
  readonly follow: (view: EditorView) => void;
  readonly kept: (yes: boolean) => void;
};

// The bar floats over the block, so pressing it must never take the caret out of the words.
const held = <T extends HTMLElement>(one: T): T => {
  one.addEventListener("mousedown", (event) => event.preventDefault());
  return one;
};

export const barred = (view: EditorView): Barred => {
  const page = view.dom.ownerDocument;
  const made = <K extends keyof HTMLElementTagNameMap>(tag: K, className: string, text = "") => {
    const one = page.createElement(tag);
    one.className = className;
    if (text) one.textContent = text;
    return one;
  };
  const button = (className: string, label: string, run: () => void, text = "") => {
    const one = held(made("button", className, text));
    one.type = "button";
    one.setAttribute("aria-label", label);
    one.title = label;
    one.addEventListener("click", run);
    return one;
  };
  const drawn = (name: string) => {
    const holder = made("span", "cm-bar-glyph");
    const svg = page.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("aria-hidden", "true");
    svg.innerHTML = markup(name) ?? "";
    holder.append(svg);
    return holder;
  };

  const bar = made("div", "cm-bar");
  bar.setAttribute("role", "toolbar");
  bar.setAttribute("aria-label", t("formatting"));

  const kind = button("cm-bar-kind", t("blockKind"), () => menu.toggleAttribute("hidden"));
  const kindSaid = made("span", "", t("paragraph"));
  kind.append(kindSaid, made("span", "cm-bar-caret", "▾"));
  const menu = made("div", "cm-bar-menu");
  menu.setAttribute("role", "menu");
  menu.hidden = true;
  for (const [said, prefix] of KINDS) {
    const one = button(
      "cm-bar-item",
      t(said),
      () => {
        menu.hidden = true;
        retyped(view, prefix);
      },
      t(said),
    );
    one.setAttribute("role", "menuitem");
    menu.append(one);
  }
  const kinds = made("span", "cm-bar-kinds");
  kinds.append(kind, menu);

  const formats = FORMATS.map(([said, keys, open, close, icon]) => {
    const one = button("cm-bar-button", `${t(said)} (${pressed(keys)})`, () =>
      wrapped(view, open, close),
    );
    one.append(drawn(icon));
    return one;
  });
  const pens = PENS.map(([hue, said, close]) => {
    const one = button("cm-bar-button", t(said), () => wrapped(view, "==", close));
    one.append(made("span", `cm-bar-pen cm-pen-${hue}`));
    return one;
  });
  const slash = button("cm-bar-button cm-bar-more", t("insertLabel"), () => {
    const { from } = view.state.selection.main;
    const before = view.state.sliceDoc(Math.max(0, from - 1), from);
    const insert = before && !/\s/.test(before) ? " /" : "/";
    view.dispatch({ changes: { from, insert }, selection: { anchor: from + insert.length } });
    view.focus();
    startCompletion(view);
  });
  slash.append(made("kbd", "", "/"), page.createTextNode(t("more")));

  bar.append(kinds, made("span", "cm-bar-sep"), ...formats);
  bar.append(made("span", "cm-bar-sep"), ...pens, made("span", "cm-bar-sep"), slash);

  const hint = made("div", "cm-hint");
  const key = (keys: string, said: string) => {
    const one = made("span", "cm-hint-item");
    one.append(made("kbd", "", keys), page.createTextNode(` ${said}`));
    return one;
  };
  const state = made("span", "cm-hint-kept");
  hint.append(
    key("/", t("hintInsert")),
    key(pressed("B"), t("bold").toLocaleLowerCase()),
    key(pressed("K"), t("linkIt").toLocaleLowerCase()),
    key("# ␣", t("bigTitle").toLocaleLowerCase()),
    key(pressed("Shift M"), t("hintMarks")),
    key("Esc", t("hintRead")),
    state,
  );

  const follow = (current: EditorView) => {
    const line = current.state.doc.lineAt(current.state.selection.main.head);
    kindSaid.textContent = t(kindOf(line.text));
    menu.hidden = true;
  };
  const kept = (yes: boolean) => {
    state.textContent = yes ? `● ${t("keptNow")}` : t("saving");
    state.classList.toggle("on", yes);
  };
  bar.addEventListener("mousedown", (event) => {
    if (!kinds.contains(event.target as Node)) menu.hidden = true;
  });
  follow(view);
  kept(true);
  return { bar, hint, follow, kept };
};
