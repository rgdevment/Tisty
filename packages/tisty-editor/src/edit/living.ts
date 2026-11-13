import { EditorView, type KeyBinding, type Panel, showPanel } from "@codemirror/view";
import { blocking, livingKeys } from "./blocks";
import { gripping } from "./handle";
import { type EditWord, hosted } from "./hosted";
import { marksShown, showMarks } from "./seeing";
import { drawing, drawnKeys } from "./widgets";

const plain = EditorView.editorAttributes.compute([marksShown], (state) => ({
  class: state.field(marksShown) ? "cm-plain" : "",
}));

// Seeing the Markdown is a look at this document, never a setting the next editor starts with.
export const viewTurned = (view: EditorView, plainly = !view.state.field(marksShown)) => {
  if (view.state.field(marksShown) !== plainly) view.dispatch({ effects: showMarks.of(plainly) });
};

const viewTabs = (view: EditorView): Panel => {
  const page = view.dom.ownerDocument;
  const say = view.state.facet(hosted).say;
  const dom = page.createElement("div");
  dom.className = "cm-views";
  const pair = page.createElement("div");
  pair.className = "cm-views-pair";
  pair.setAttribute("role", "group");
  dom.append(pair);
  const tab = (word: EditWord, plainly: boolean) => {
    const one = page.createElement("button");
    one.type = "button";
    one.className = "cm-view-tab";
    one.textContent = say(word);
    one.addEventListener("mousedown", (event) => event.preventDefault());
    one.addEventListener("click", () => {
      viewTurned(view, plainly);
      view.focus();
    });
    return one;
  };
  const preview = tab("viewPreview", false);
  const markdown = tab("viewMarkdown", true);
  pair.append(preview, markdown);
  const pressed = (on: boolean) => {
    preview.setAttribute("aria-pressed", String(!on));
    markdown.setAttribute("aria-pressed", String(on));
  };
  pressed(view.state.field(marksShown));
  return { dom, top: true, update: (update) => pressed(update.state.field(marksShown)) };
};

const living = EditorView.baseTheme({
  ".cm-panels.cm-panels-top": {
    background: "transparent",
    border: "none",
    pointerEvents: "none",
  },
  ".cm-views": {
    display: "flex",
    justifyContent: "flex-end",
    padding: "4px 0 10px",
  },
  ".cm-views-pair": {
    display: "inline-flex",
    gap: "2px",
    padding: "3px",
    borderRadius: "10px",
    background: "color-mix(in srgb, var(--tisty-ink) 6%, var(--tisty-sheet))",
    boxShadow: "inset 0 0 0 1px color-mix(in srgb, var(--tisty-ink) 6%, transparent)",
    pointerEvents: "auto",
  },
  ".cm-view-tab": {
    padding: "4px 12px",
    border: "none",
    borderRadius: "7px",
    background: "none",
    color: "var(--tisty-faint)",
    font: "inherit",
    fontSize: "12px",
    fontWeight: "550",
    letterSpacing: "0.01em",
    cursor: "pointer",
    transition: "background-color 120ms ease, color 120ms ease",
  },
  ".cm-view-tab:hover": { color: "var(--tisty-ink)" },
  ".cm-view-tab:focus-visible": { outline: "2px solid var(--tisty-accent)", outlineOffset: "1px" },
  ".cm-view-tab[aria-pressed=true]": {
    color: "var(--tisty-ink)",
    background: "var(--tisty-sheet)",
    boxShadow:
      "0 1px 2px var(--tisty-cast), 0 0 0 1px color-mix(in srgb, var(--tisty-ink) 8%, transparent)",
  },
  ".cm-rich summary": { pointerEvents: "auto", cursor: "pointer" },
  ".cm-rich td, .cm-rich th, .cm-rich [data-act], .cm-rich .col-grip": {
    pointerEvents: "auto",
    cursor: "text",
  },
  ".cm-rich [data-act]": { cursor: "pointer" },
  ".cm-rich .col-grip": { cursor: "col-resize" },
  ".cm-table-menu": {
    position: "absolute",
    zIndex: "6",
    display: "flex",
    flexDirection: "column",
    minWidth: "200px",
    padding: "4px",
    borderRadius: "10px",
    border: "1px solid var(--tisty-hair)",
    background: "var(--tisty-rail)",
    boxShadow: "0 12px 32px -10px var(--tisty-cast), 0 2px 6px var(--tisty-cast)",
    fontSize: "12.5px",
  },
  ".cm-table-menu button": {
    padding: "6px 10px",
    border: "none",
    borderRadius: "6px",
    background: "none",
    color: "var(--tisty-ink)",
    font: "inherit",
    textAlign: "left",
    cursor: "pointer",
    whiteSpace: "pre",
  },
  ".cm-table-menu button:hover": { background: "var(--tisty-hover)" },
  ".cm-table-menu button.danger": { color: "var(--tisty-hue-red)" },
  ".cm-table-menu-apart": { height: "1px", margin: "4px 2px", background: "var(--tisty-hair)" },
  ".cm-table-menu-heading": {
    padding: "6px 10px 2px",
    fontSize: "11px",
    fontWeight: "600",
    letterSpacing: "0.04em",
    textTransform: "uppercase",
    color: "var(--tisty-faint)",
  },
  ".cm-cell-day": {
    width: "100%",
    boxSizing: "border-box",
    padding: "2px 4px",
    border: "1px solid var(--tisty-accent)",
    borderRadius: "6px",
    background: "var(--tisty-sheet)",
    color: "var(--tisty-ink)",
    font: "inherit",
    colorScheme: "light dark",
  },
  ".cm-cell-editing": {
    outline: "2px solid var(--tisty-accent)",
    outlineOffset: "-2px",
    background: "color-mix(in srgb, var(--tisty-accent) 8%, transparent)",
  },
  "&.cm-plain .cm-content": {
    fontFamily: "ui-monospace, 'Cascadia Mono', 'SF Mono', Menlo, monospace",
    fontSize: "0.9em",
  },
  "&.cm-plain .cm-line, &.cm-plain .cm-line *": {
    fontSize: "inherit !important",
    fontWeight: "400 !important",
    fontStyle: "normal !important",
    lineHeight: "1.6 !important",
    color: "inherit !important",
    background: "none !important",
    textDecoration: "none !important",
    textTransform: "none !important",
    letterSpacing: "normal !important",
  },
  "&.cm-plain .cm-line": {
    border: "none !important",
    padding: "0 !important",
  },
  ".cm-chosen": {
    background: "color-mix(in srgb, var(--tisty-accent) 12%, transparent)",
    borderRadius: "4px",
  },
  ".cm-chosen .cm-selectionBackground, &.cm-focused .cm-chosen ::selection": {
    background: "transparent",
  },
  ".cm-rich": {
    display: "flow-root",
    margin: "0",
    padding: "6px 0",
    borderRadius: "6px",
    cursor: "default",
    outline: "2px solid transparent",
    transition: "outline-color 120ms ease",
  },
  ".cm-rich:hover": { outlineColor: "var(--tisty-hair)" },
  ".cm-rich.cm-rich-chosen": {
    outlineColor: "var(--tisty-accent)",
    background: "color-mix(in srgb, var(--tisty-accent) 8%, transparent)",
  },
  ".cm-rich > *": { pointerEvents: "none" },
  ".cm-grip": {
    position: "absolute",
    left: "-30px",
    width: "30px",
    height: "24px",
    lineHeight: "24px",
    textAlign: "center",
    letterSpacing: "-3px",
    color: "var(--tisty-faint)",
    cursor: "grab",
    userSelect: "none",
    borderRadius: "4px",
    zIndex: "2",
  },
  ".cm-grip:hover": { background: "var(--tisty-hair)", color: "var(--tisty-soft)" },
  ".cm-grip:active": { cursor: "grabbing" },
  ".cm-drop": {
    position: "absolute",
    left: "0",
    right: "0",
    height: "2px",
    marginTop: "-1px",
    background: "var(--tisty-accent)",
    pointerEvents: "none",
    zIndex: "2",
  },
});

export const lively = [
  marksShown,
  blocking,
  drawing,
  gripping,
  living,
  plain,
  showPanel.of(viewTabs),
];

export const livelyKeys: KeyBinding[] = [...drawnKeys, ...livingKeys];
