const REST = 4;
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

// The page grows to its full height inside whatever scrolls it, so the bar follows that scroll, not its own.
export const docked = (bar: HTMLElement, hint: HTMLElement): (() => void) => {
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
    bar.style.top = `${Math.min(bottom, Math.max(top + REST, stuck))}px`;
    const left = Number.parseFloat(view.getComputedStyle(sheet).paddingLeft) || 0;
    bar.style.left = `${(box.left + view.scrollX) / zoom + left - LEFT}px`;
  };
  place();
  const listen = { capture: true, passive: true };
  outer?.addEventListener("scroll", place, listen);
  outer?.addEventListener("resize", place);
  view?.addEventListener("scroll", place, listen);
  view?.addEventListener("resize", place);
  return () => {
    outer?.removeEventListener("scroll", place, listen);
    outer?.removeEventListener("resize", place);
    view?.removeEventListener("scroll", place, listen);
    view?.removeEventListener("resize", place);
    bar.remove();
    hint.remove();
  };
};
