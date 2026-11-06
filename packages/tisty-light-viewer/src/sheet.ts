import type { Say } from "tisty-sheet";

const PAINT_WAIT = 100;
const FRAME_BUDGET = 12;
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

export const isDark = (): boolean => document.documentElement.classList.contains("dark");

export const dress = async (say: Say, locale: string): Promise<void> => {
  await painted();
  const { readAs } = await import("./drawing");
  await readAs(document, { say, locale, live: () => true }).drawn;
};

export const redraw = async (): Promise<void> => {
  const { diagrams } = await import("./drawing");
  await diagrams(document, isDark(), () => true);
};

export const turnTheme = (): void => {
  const root = document.documentElement;
  const dark = !isDark();
  root.dataset.theme = dark ? "dark" : "light";
  root.classList.toggle("dark", dark);
  try {
    localStorage.setItem(THEME, dark ? "dark" : "light");
  } catch {
    return;
  }
};
