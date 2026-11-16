import { EditorView, type KeyBinding } from "@codemirror/view";
import { blocking, livingKeys } from "./blocks";
import { gripping } from "./handle";
import { marksShown, writtenPlain } from "./seeing";

// The page the core draws is the one preview; this editor is the Markdown, written as plain words.
const plain = EditorView.editorAttributes.of({ class: "cm-plain" });

const living = EditorView.baseTheme({
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

export const lively = [writtenPlain.of(true), marksShown, blocking, gripping, living, plain];

export const livelyKeys: KeyBinding[] = livingKeys;
