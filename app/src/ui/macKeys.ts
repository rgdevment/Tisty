type Held = { readonly ctrlKey: boolean; readonly metaKey: boolean };
type Moving = Held & { readonly altKey: boolean; readonly shiftKey: boolean };

export const onMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Macintosh");

export const modKey = (event: Held): boolean =>
  onMac ? event.metaKey : event.ctrlKey || event.metaKey;

export const moveKey = (event: Moving): boolean =>
  onMac ? event.ctrlKey && event.altKey && !event.metaKey && !event.shiftKey : event.altKey;

const WALKS: Readonly<Record<string, "line" | "letter">> = {
  n: "line",
  p: "line",
  f: "letter",
  b: "letter",
  a: "letter",
  e: "letter",
  v: "line",
};

export const caretWalk = (event: Moving & { readonly key: string }): "line" | "letter" | null =>
  onMac && event.ctrlKey && !event.metaKey && !event.altKey
    ? (WALKS[event.key.toLowerCase()] ?? null)
    : null;

export const MOVE_KEYS = onMac ? "⌃⌥" : "Alt";

export const MOVE_SHORTCUTS = onMac
  ? "Control+Alt+ArrowUp Control+Alt+ArrowDown"
  : "Alt+ArrowUp Alt+ArrowDown";
