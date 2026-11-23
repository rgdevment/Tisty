type Held = { readonly ctrlKey: boolean; readonly metaKey: boolean };
type Typed = { readonly key: string; readonly code: string; readonly altKey: boolean };

const agent = typeof navigator !== "undefined" ? navigator.userAgent : "";

export const onMac = agent.includes("Macintosh");
export const onWindows = agent.includes("Windows");

export const modKey = (event: Held): boolean =>
  onMac ? event.metaKey : event.ctrlKey || event.metaKey;

const writesLetter = (event: Typed): boolean => !event.altKey && /^[\p{L}\p{M}]$/u.test(event.key);

export const letterOf = (event: Typed): string => {
  if (/^[a-z]$/i.test(event.key)) return event.key.toLowerCase();
  if (!writesLetter(event) || /\p{Script=Latin}/u.test(event.key)) return "";
  return (/^Key([A-Z])$/.exec(event.code)?.[1] ?? "").toLowerCase();
};

export const keyOf = (event: Typed): string => {
  const digit = /^Digit(\d)$/.exec(event.code)?.[1];
  if (digit && writesLetter(event)) return digit;
  return letterOf(event) || event.key.toLowerCase();
};

export const shortcut = (key: string): string => `${onMac ? "⌘" : "Ctrl+"}${key}`;
