const PER_PIXEL = 0.008;

const turning = (event: MouseEvent): boolean =>
  event.button === 2 || (event.button === 0 && event.shiftKey);

// The right button or Shift turns the globe; sigma never hears those presses, so a plain drag still moves the view.
export function turnedByHand(
  container: HTMLElement,
  turnBy: (yaw: number, pitch: number) => void,
): () => void {
  const page = container.ownerDocument;
  let from: [number, number] | null = null;
  const held = (event: PointerEvent | MouseEvent) => {
    if (!turning(event)) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.type === "pointerdown") from = [event.clientX, event.clientY];
  };
  const moved = (event: PointerEvent) => {
    if (!from) return;
    // A button let go outside the window never says so here, but the next move comes with no button held.
    if (event.buttons === 0) {
      from = null;
      return;
    }
    turnBy((event.clientX - from[0]) * PER_PIXEL, (event.clientY - from[1]) * PER_PIXEL);
    from = [event.clientX, event.clientY];
  };
  const let_go = () => {
    from = null;
  };
  const kept = (event: Event) => event.preventDefault();
  container.addEventListener("pointerdown", held, true);
  container.addEventListener("mousedown", held, true);
  container.addEventListener("contextmenu", kept);
  page.addEventListener("pointermove", moved);
  page.addEventListener("pointerup", let_go);
  page.addEventListener("pointercancel", let_go);
  return () => {
    container.removeEventListener("pointerdown", held, true);
    container.removeEventListener("mousedown", held, true);
    container.removeEventListener("contextmenu", kept);
    page.removeEventListener("pointermove", moved);
    page.removeEventListener("pointerup", let_go);
    page.removeEventListener("pointercancel", let_go);
  };
}
