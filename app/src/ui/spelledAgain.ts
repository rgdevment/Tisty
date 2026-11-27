// The browser marks a word once and keeps the mark, so a word just taught to the system's speller is weighed again.
export const spelledAgain = (sheet: HTMLElement | null | undefined) => {
  if (!sheet) return;
  sheet.spellcheck = false;
  sheet.ownerDocument.defaultView?.requestAnimationFrame(() => {
    sheet.spellcheck = true;
  });
};
