export type Sheet = {
  readonly wide: number;
  readonly tall: number;
  readonly x: number;
  readonly y: number;
  readonly said?: string;
};

const TURN = "sheet-turn";

const SAID = "break-said";

const BREAKS = ".page > hr, .page > .sheet-turn";

const HEADING = /^H[1-6]$/;

const topOf = (one: HTMLElement, sheet: HTMLElement) =>
  one.offsetParent === sheet ? one.offsetTop : one.offsetTop - sheet.offsetTop;

const bottomOf = (one: HTMLElement, sheet: HTMLElement) => topOf(one, sheet) + one.offsetHeight;

// The band between two sheets starts at the foot of the one above, its bottom margin included.
export const sheeted = (page: Document, { tall, y, said }: Sheet) => {
  const sheet = page.querySelector<HTMLElement>(".page");
  if (!sheet || page.querySelector(".editing")) return false;
  for (const one of sheet.querySelectorAll(`.${TURN}, .${SAID}`)) one.remove();
  for (const one of sheet.querySelectorAll<HTMLElement>(BREAKS)) one.style.removeProperty("--fill");
  // A pixel short of the sheet, since the paper's own room is a fraction less than the rounded one here.
  const room = tall - 2 * y - 1;
  let begins = y;
  let ended = y;
  let fresh = true;
  // A heading never stays alone at the foot: the turn takes the headings right above along.
  let heads: HTMLElement[] = [];
  let headsEnded = y;
  let headsOpen = false;
  for (const one of Array.from(sheet.children) as HTMLElement[]) {
    if (one.matches(BREAKS)) {
      one.style.setProperty("--fill", `${Math.max(0, begins + room - ended)}px`);
      begins = bottomOf(one, sheet) + y;
      ended = begins;
      fresh = true;
      heads = [];
      continue;
    }
    if (!fresh && bottomOf(one, sheet) - begins > room) {
      const moving = heads.length > 0 && !headsOpen;
      const turn = page.createElement("div");
      turn.className = TURN;
      turn.setAttribute("aria-hidden", "true");
      turn.style.setProperty(
        "--fill",
        `${Math.max(0, begins + room - (moving ? headsEnded : ended))}px`,
      );
      (moving ? heads[0] : one).before(turn);
      begins = bottomOf(turn, sheet) + y;
      fresh = true;
      if (!moving) heads = [];
      headsOpen = true;
    }
    if (!HEADING.test(one.tagName)) heads = [];
    else if (heads.push(one) === 1) {
      headsEnded = ended;
      headsOpen = fresh;
    }
    fresh = false;
    ended = bottomOf(one, sheet);
  }
  sheet.style.setProperty("--last", `${Math.max(0, begins + room - ended)}px`);
  if (!said) return true;
  for (const one of sheet.querySelectorAll<HTMLElement>(":scope > hr")) {
    const label = page.createElement("div");
    label.className = SAID;
    label.setAttribute("aria-hidden", "true");
    label.textContent = said;
    label.style.top = `${topOf(one, sheet) + one.offsetHeight / 2}px`;
    sheet.append(label);
  }
  return true;
};

// Pictures that arrive late move every foot, so the sheets are laid again when the page grows.
export const sheeting = (page: Document, sheet: () => Sheet | undefined) => {
  const root = page.documentElement;
  const body = page.querySelector<HTMLElement>(".page");
  let laidAt = -1;
  const lay = () => {
    const now = sheet();
    root.classList.toggle("sheeted", Boolean(now));
    if (!now) return;
    root.style.setProperty("--sheet-x", `${now.x}px`);
    root.style.setProperty("--sheet-y", `${now.y}px`);
    laidAt = sheeted(page, now) ? (body?.offsetHeight ?? -1) : -1;
  };
  let pending = 0;
  const later = () => {
    if (pending || body?.offsetHeight === laidAt) return;
    pending =
      page.defaultView?.requestAnimationFrame(() => {
        pending = 0;
        lay();
      }) ?? 0;
  };
  lay();
  const View = page.defaultView?.ResizeObserver;
  if (!body || !View || !sheet()) return { lay, stop: () => {} };
  const watching = new View(later);
  watching.observe(body);
  return { lay, stop: () => watching.disconnect() };
};
