import { drawsHeic } from "./host";
import type { Words } from "./words";

const PAINT_WAIT = 100;
const FRAME_BUDGET = 12;
const SETTLED_A_FRAME = 400;
const THEME = "tisty.viewer.theme";

const painted = (): Promise<void> =>
  new Promise((done) => {
    // A window still hidden paints no frame, and the drawing must not wait for one that never comes.
    const fallback = setTimeout(done, PAINT_WAIT);
    requestAnimationFrame(() => {
      clearTimeout(fallback);
      setTimeout(done);
    });
  });

export const laidIn = async (page: HTMLElement, pieces: readonly string[]): Promise<void> => {
  let at = 0;
  while (at < pieces.length) {
    await painted();
    const started = performance.now();
    do {
      page.insertAdjacentHTML("beforeend", pieces[at]);
      at += 1;
    } while (at < pieces.length && performance.now() - started < FRAME_BUDGET);
  }
};

// A block left to content-visibility only guesses its height, so the bar and the orange line would jump while reading.
export const settled = async (page: HTMLElement): Promise<void> => {
  const blocks = Array.from(page.children) as HTMLElement[];
  for (let at = 0; at < blocks.length; at += SETTLED_A_FRAME) {
    await painted();
    for (const block of blocks.slice(at, at + SETTLED_A_FRAME)) {
      block.style.contentVisibility = "visible";
    }
  }
};

export const isDark = (): boolean => document.documentElement.classList.contains("dark");

export const dress = async (page: HTMLElement, words: Words, own = false): Promise<void> => {
  await painted();
  const { readAs } = await import("./drawing");
  const [owned, cards, heic] = own
    ? await Promise.all([
        import("./owned"),
        import("./cards")
          .then(({ cardsDrawn }) => cardsDrawn(page.ownerDocument))
          .catch(() => undefined),
        drawsHeic().catch(() => false),
      ])
    : [null, undefined, false];
  // In one's own document a web link alone in its paragraph is a card of its site, never a preview brought from it.
  const sites = own ? {} : undefined;
  const brought = owned ? () => void owned.drawnOwn(document, words, isDark, heic) : undefined;
  const reading = { say: words.sayToSheet, locale: words.locale, live: () => true };
  cards?.();
  await readAs(document, { ...reading, sites, brought }).drawn;
};

export const redraw = async (): Promise<void> => {
  const { diagrams } = await import("./drawing");
  await diagrams(document, isDark(), () => true);
};

const shaded = (dark: boolean): void => {
  const root = document.documentElement;
  root.dataset.theme = dark ? "dark" : "light";
  root.classList.toggle("dark", dark);
};

// Paper is light, and a dark theme's near-white words would be printed on it.
export const printed = async (): Promise<void> => {
  await document.fonts?.ready;
  const dark = isDark();
  if (dark) {
    shaded(false);
    await redraw();
    window.addEventListener(
      "afterprint",
      () => {
        shaded(true);
        void redraw();
      },
      { once: true },
    );
  }
  window.print();
};

export const turnTheme = (): void => {
  const dark = !isDark();
  shaded(dark);
  try {
    localStorage.setItem(THEME, dark ? "dark" : "light");
  } catch {
    return;
  }
};
