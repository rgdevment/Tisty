const WORDLESS = new Set([
  "button",
  "checkbox",
  "color",
  "file",
  "image",
  "radio",
  "range",
  "reset",
  "submit",
]);

const heldInAnIsland = (at: HTMLElement) => {
  const node = at.ownerDocument?.getSelection?.()?.anchorNode;
  const holder = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
  return Boolean(holder && at.contains(holder) && holder.closest('[contenteditable="false"]'));
};

const writesWords = (at: HTMLElement) => {
  if (at.isContentEditable) return !heldInAnIsland(at);
  const field = at as HTMLInputElement;
  if (field.readOnly || field.disabled) return false;
  return at.tagName === "TEXTAREA" || (at.tagName === "INPUT" && !WORDLESS.has(field.type));
};

export const unnavigated = (event: KeyboardEvent) => {
  if (event.key !== "Backspace" || event.defaultPrevented) return;
  const at = event.target as HTMLElement | null;
  if (typeof at?.closest !== "function" || writesWords(at)) return;
  event.preventDefault();
};
