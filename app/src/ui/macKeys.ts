import { letterOf, onMac } from "tisty-diagrams/macKeys";

export { keyOf, letterOf, modKey, onMac, onWindows, shortcut } from "tisty-diagrams/macKeys";

type Moving = {
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly altKey: boolean;
  readonly shiftKey: boolean;
};

export const moveKey = (event: Moving): boolean =>
  onMac ? event.ctrlKey && event.altKey && !event.metaKey && !event.shiftKey : event.altKey;

type Walk = "line" | "letter" | "start" | "end";

const WALKS: Readonly<Record<string, Walk>> = {
  n: "line",
  p: "line",
  f: "letter",
  b: "letter",
  a: "start",
  e: "end",
  v: "line",
};

type Pressed = Moving & { readonly key: string; readonly code: string };

export const caretWalk = (event: Pressed): Walk | null =>
  onMac && event.ctrlKey && !event.metaKey && !event.altKey
    ? (WALKS[letterOf(event)] ?? null)
    : null;

export type Edge = { readonly toStart: boolean; readonly visual: boolean };

const COMMAND_EDGES: Readonly<Record<string, Edge>> = {
  ArrowLeft: { toStart: true, visual: true },
  ArrowRight: { toStart: false, visual: true },
};

export const edgeWalk = (event: Pressed & { readonly isComposing: boolean }): Edge | null => {
  if (event.shiftKey || event.isComposing) return null;
  const walked = caretWalk(event);
  if (walked === "start" || walked === "end") return { toStart: walked === "start", visual: false };
  return onMac && event.metaKey && !event.ctrlKey && !event.altKey
    ? (COMMAND_EDGES[event.key] ?? null)
    : null;
};

export const MOVE_KEYS = onMac ? "⌃⌥" : "Alt";

export const MOVE_SHORTCUTS = onMac
  ? "Control+Alt+ArrowUp Control+Alt+ArrowDown"
  : "Alt+ArrowUp Alt+ArrowDown";
