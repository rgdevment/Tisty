const REST = 2;
const LEFT = 6;
const STUCK = 6;

const scrollerOf = (from: Element | null | undefined): Element | null => {
  for (let one = from?.parentElement; one; one = one.parentElement) {
    const { overflowY } = one.ownerDocument.defaultView?.getComputedStyle(one) ?? {};
    if ((overflowY === "auto" || overflowY === "scroll") && one.scrollHeight > one.clientHeight)
      return one;
  }
  return null;
};

export type Dock = { readonly stop: () => void; readonly place: () => void };

// The page grows to its full height inside whatever scrolls it, so the bar follows that scroll, not its own.
export const docked = (bar: HTMLElement, hint: HTMLElement, aim?: () => number | null): Dock => {
  const page = bar.ownerDocument;
  const view = page.defaultView;
  const sheet = page.querySelector<HTMLElement>(".page");
  const frame = view?.frameElement ?? null;
  const outer = frame?.ownerDocument.defaultView ?? null;
  bar.classList.add("cm-dock");
  hint.classList.add("cm-dock");
  page.body.append(bar);
  (sheet ?? page.body).append(hint);
  const place = () => {
    if (!sheet || !view) return;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const box = sheet.getBoundingClientRect();
    const scroller = scrollerOf(frame);
    const clip = scroller?.getBoundingClientRect().top ?? 0;
    const seen = Math.max(0, clip - (frame?.getBoundingClientRect().top ?? 0));
    const top = (box.top + view.scrollY) / zoom;
    const bottom = (box.bottom + view.scrollY) / zoom - bar.offsetHeight;
    const stuck = (seen + view.scrollY) / zoom + STUCK;
    const line = aim?.();
    const over = line == null ? 0 : (line + view.scrollY) / zoom - bar.offsetHeight - STUCK;
    bar.style.top = `${Math.min(bottom, Math.max(top + REST, stuck, over))}px`;
    const left = Number.parseFloat(view.getComputedStyle(sheet).paddingLeft) || 0;
    bar.style.left = `${(box.left + view.scrollX) / zoom + left - LEFT}px`;
  };
  // Only a scroll that carries the page moves it; the list beside it scrolling says nothing.
  const scrolled = (event: Event) => {
    const moved = event.target;
    if (!frame || !(moved instanceof (outer?.Element ?? Element)) || moved.contains(frame)) place();
  };
  place();
  const listen = { capture: true, passive: true };
  outer?.addEventListener("scroll", scrolled, listen);
  outer?.addEventListener("resize", place);
  view?.addEventListener("scroll", place, listen);
  const Inner = view?.ResizeObserver;
  const Outer = outer?.ResizeObserver;
  const inner = Inner && sheet ? new Inner(place) : null;
  const around = Outer && frame ? new Outer(place) : null;
  if (sheet) inner?.observe(sheet);
  if (frame) around?.observe(frame);
  const stop = () => {
    outer?.removeEventListener("scroll", scrolled, listen);
    outer?.removeEventListener("resize", place);
    view?.removeEventListener("scroll", place, listen);
    inner?.disconnect();
    around?.disconnect();
    bar.remove();
    hint.remove();
  };
  return { stop, place };
};
