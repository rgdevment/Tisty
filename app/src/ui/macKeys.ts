type Held = { readonly ctrlKey: boolean; readonly metaKey: boolean };
type Moving = Held & { readonly altKey: boolean; readonly shiftKey: boolean };

export const onMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Macintosh");
export const onWindows =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");

export const modKey = (event: Held): boolean =>
  onMac ? event.metaKey : event.ctrlKey || event.metaKey;

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

const letterOf = (event: { readonly key: string; readonly code: string }) => {
  if (/^[a-z]$/i.test(event.key)) return event.key.toLowerCase();
  return /^\p{L}$/u.test(event.key) && !/\p{Script=Latin}/u.test(event.key)
    ? event.code.replace(/^Key/, "").toLowerCase()
    : "";
};

export const caretWalk = (
  event: Moving & { readonly key: string; readonly code: string },
): Walk | null =>
  onMac && event.ctrlKey && !event.metaKey && !event.altKey
    ? (WALKS[letterOf(event)] ?? null)
    : null;

export const MOVE_KEYS = onMac ? "⌃⌥" : "Alt";

export const MOVE_SHORTCUTS = onMac
  ? "Control+Alt+ArrowUp Control+Alt+ArrowDown"
  : "Alt+ArrowUp Alt+ArrowDown";
