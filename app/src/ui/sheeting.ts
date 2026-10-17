export type Sheet = {
  readonly wide: number;
  readonly tall: number;
  readonly x: number;
  readonly y: number;
};

const TURN = "sheet-turn";

const BREAKS = ".page > hr, .page > .sheet-turn";

const topOf = (one: HTMLElement, sheet: HTMLElement) => one.offsetTop - sheet.offsetTop;

// A block that would cross the foot of a sheet starts the next one, as it does on paper.
export const sheeted = (page: Document, { tall, y }: Sheet) => {
  const sheet = page.querySelector<HTMLElement>(".page");
  if (!sheet || page.querySelector(".editing")) return;
  for (const one of sheet.querySelectorAll(`.${TURN}`)) one.remove();
  for (const one of sheet.querySelectorAll<HTMLElement>(BREAKS)) one.style.removeProperty("--fill");
  const room = tall - 2 * y;
  let begins = y;
  for (const one of Array.from(sheet.children) as HTMLElement[]) {
    const top = topOf(one, sheet);
    if (one.matches(BREAKS)) {
      one.style.setProperty("--fill", `${Math.max(0, begins + room - top)}px`);
      begins = topOf(one, sheet) + one.offsetHeight + y;
      continue;
    }
    const bottom = top + one.offsetHeight;
    if (bottom - begins <= room || top <= begins) continue;
    const turn = page.createElement("div");
    turn.className = TURN;
    turn.setAttribute("aria-hidden", "true");
    turn.style.setProperty("--fill", `${Math.max(0, begins + room - top)}px`);
    one.before(turn);
    begins = topOf(one, sheet);
  }
  const end = sheet.lastElementChild as HTMLElement | null;
  const last = end ? topOf(end, sheet) + end.offsetHeight : begins;
  sheet.style.setProperty("--last", `${Math.max(0, begins + room - last)}px`);
};

// Pictures that arrive late and a paper that changes move every foot, so the sheets are laid again.
export const sheeting = (page: Document, sheet: () => Sheet | undefined) => {
  const root = page.documentElement;
  const lay = () => {
    const now = sheet();
    root.classList.toggle("sheeted", Boolean(now));
    if (!now) return;
    root.style.setProperty("--sheet-x", `${now.x}px`);
    root.style.setProperty("--sheet-y", `${now.y}px`);
    sheeted(page, now);
  };
  let pending = 0;
  const later = () => {
    if (pending) return;
    pending =
      page.defaultView?.requestAnimationFrame(() => {
        pending = 0;
        lay();
      }) ?? 0;
  };
  lay();
  const body = page.querySelector(".page");
  const View = page.defaultView?.ResizeObserver;
  if (!body || !View) return { lay, stop: () => {} };
  const watching = new View(later);
  watching.observe(body);
  return { lay, stop: () => watching.disconnect() };
};
