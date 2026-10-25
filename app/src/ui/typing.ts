export const typing = (at: EventTarget | null): boolean =>
  at instanceof HTMLElement &&
  (at.isContentEditable || at.closest("input, textarea, select") !== null);
