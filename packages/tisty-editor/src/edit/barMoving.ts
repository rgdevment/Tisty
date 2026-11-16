const STILL_PX = 4;
const HANDLES = ".cm-bar-drag, .cm-bar-fold";

export const MOVED = "moved";

export const movable = (bar: HTMLElement): (() => void) => {
  const page = bar.ownerDocument;
  const pressed = (event: MouseEvent) => {
    if (event.button !== 0 || !(event.target as Element).closest?.(HANDLES)) return;
    const box = bar.getBoundingClientRect();
    const start = { x: event.clientX, y: event.clientY };
    let moving = false;
    const moved = (next: MouseEvent) => {
      const dx = next.clientX - start.x;
      const dy = next.clientY - start.y;
      if (!moving && Math.hypot(dx, dy) < STILL_PX) return;
      moving = true;
      bar.dataset[MOVED] = "";
      bar.style.position = "fixed";
      bar.style.left = `${box.left + dx}px`;
      bar.style.top = `${box.top + dy}px`;
    };
    const ended = () => {
      page.removeEventListener("mousemove", moved);
      page.removeEventListener("mouseup", ended);
      if (!moving) return;
      // The press that moved the bar is not a press on the button it started on.
      const swallowed = (click: Event) => {
        click.stopPropagation();
        click.preventDefault();
      };
      bar.addEventListener("click", swallowed, { capture: true, once: true });
      setTimeout(() => bar.removeEventListener("click", swallowed, { capture: true }), 0);
    };
    page.addEventListener("mousemove", moved);
    page.addEventListener("mouseup", ended);
  };
  bar.addEventListener("mousedown", pressed, true);
  return () => bar.removeEventListener("mousedown", pressed, true);
};
