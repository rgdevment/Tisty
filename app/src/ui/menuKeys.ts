export const asksForMenu = (e: React.KeyboardEvent) =>
  e.key === "ContextMenu" || (e.shiftKey && e.key === "F10");

export const menuAt = (e: React.KeyboardEvent) => {
  const box = (e.target as HTMLElement).getBoundingClientRect();
  return { x: box.left + 24, y: box.bottom };
};
