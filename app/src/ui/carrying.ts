type Carry = {
  readonly over: (next: PointerEvent) => void;
  readonly ended: () => void;
  readonly dropped: () => void;
};

export const carrying = (
  page: Document,
  held: HTMLElement,
  start: PointerEvent,
  far: number,
  carry: Carry,
) => {
  let going = false;
  held.setPointerCapture?.(start.pointerId);
  const over = (next: PointerEvent) => {
    if (!going && Math.abs(next.clientY - start.clientY) <= far) return;
    if (!going) {
      going = true;
      page.getSelection()?.removeAllRanges();
      page.documentElement.classList.add("carrying");
    }
    next.preventDefault();
    carry.over(next);
  };
  const ended = () => {
    page.removeEventListener("pointermove", over);
    page.removeEventListener("pointerup", up);
    page.removeEventListener("pointercancel", ended);
    held.removeEventListener("lostpointercapture", ended);
    page.documentElement.classList.remove("carrying");
    carry.ended();
  };
  const up = () => {
    ended();
    if (!going) return;
    // The click that ends a carry is not a press on what was carried.
    const swallowed = (one: Event) => one.stopPropagation();
    page.addEventListener("click", swallowed, { capture: true, once: true });
    setTimeout(() => page.removeEventListener("click", swallowed, { capture: true }), 0);
    carry.dropped();
  };
  page.addEventListener("pointermove", over);
  page.addEventListener("pointerup", up);
  page.addEventListener("pointercancel", ended);
  held.addEventListener("lostpointercapture", ended);
};
