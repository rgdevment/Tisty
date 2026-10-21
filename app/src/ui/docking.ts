import dressed from "./bar.css?raw";

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

const DRESSED = "tisty-bar";

// The window has none of the page's rules, so the bar brings its own when it is laid there.
const dress = (place: Document) => {
  if (place.getElementById(DRESSED)) return;
  const style = place.createElement("style");
  style.id = DRESSED;
  style.textContent = dressed;
  place.head.append(style);
};

// The desk right of the sheet takes the bar standing up, unless the panel holds that side or it has
// no room; then the bar lies across the top of the page instead.
const slotIn = (place: Document | undefined, width: number): HTMLElement | null => {
  if (place?.querySelector("[data-beside]")) return null;
  const desk = place?.querySelector<HTMLElement>('[data-bar-slot="desk"]');
  const room = desk?.getBoundingClientRect();
  // The desk ends where the documents' own area does, not at the window's edge.
  const bound = desk?.closest("main")?.getBoundingClientRect();
  return desk && room && bound && room.left + width + 8 <= bound.right ? desk : null;
};

// The page grows to its full height inside whatever scrolls it, so the bar follows that scroll, not its own.
export const docked = (
  bar: HTMLElement,
  hint: HTMLElement,
  aim?: () => number | null,
  under?: HTMLElement,
  side = false,
): Dock => {
  const page = bar.ownerDocument;
  const view = page.defaultView;
  const sheet = page.querySelector<HTMLElement>(".page");
  const frame = view?.frameElement ?? null;
  const outer = frame?.ownerDocument.defaultView ?? null;
  bar.classList.add("cm-dock");
  hint.classList.add("cm-dock");
  // The whole document runs past the screen, so its bar stands up in the page's margin instead.
  bar.classList.toggle("side", side);
  page.body.append(bar);
  // The help sits under the block being written, where the eye is, not at the foot of a long page.
  if (under && sheet) {
    hint.classList.add("under");
    under.after(hint);
  } else (sheet ?? page.body).append(hint);
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
    const look = view.getComputedStyle(sheet);
    const slot = side ? slotIn(frame?.ownerDocument, 40) : null;
    if (slot) {
      dress(slot.ownerDocument);
      // The slot only says where: a node of React's own holds nothing it did not draw.
      const room = slot.ownerDocument.body;
      if (bar.parentElement !== room) room.append(bar);
      bar.classList.add("docked", "side");
      bar.dataset.keepsWriting = "";
      const at = slot.getBoundingClientRect();
      bar.style.top = `${at.top}px`;
      bar.style.left = `${at.left}px`;
      return;
    }
    if (bar.parentElement !== page.body) page.body.append(bar);
    bar.classList.remove("docked");
    delete bar.dataset.keepsWriting;
    bar.classList.remove("side");
    if (side) {
      bar.style.top = `${Math.min(bottom, Math.max(top + REST, stuck))}px`;
      const width = Math.min(box.width, view.innerWidth * zoom) / zoom;
      bar.style.left = `${(Math.max(0, box.left) + view.scrollX) / zoom + (width - bar.offsetWidth) / 2}px`;
      return;
    }
    const line = aim?.();
    const over = line == null ? 0 : (line + view.scrollY) / zoom - bar.offsetHeight - STUCK;
    bar.style.top = `${Math.min(bottom, Math.max(top + REST, stuck, over))}px`;
    const left = Number.parseFloat(look.paddingLeft) || 0;
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
  // The panel opens and shuts without the page changing size; it comes and goes beside the sheet.
  const Watch = outer?.MutationObserver;
  const panels = side && Watch ? new Watch(place) : null;
  const room = frame?.closest("main");
  if (room) panels?.observe(room, { childList: true });
  const stop = () => {
    outer?.removeEventListener("scroll", scrolled, listen);
    outer?.removeEventListener("resize", place);
    view?.removeEventListener("scroll", place, listen);
    inner?.disconnect();
    around?.disconnect();
    panels?.disconnect();
    bar.remove();
    hint.remove();
  };
  return { stop, place };
};
