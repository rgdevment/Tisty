const KEPT = "tisty.viewer.places";
const AT_MOST = 200;

type Places = Record<string, number>;

const read = (): Places => {
  try {
    return JSON.parse(localStorage.getItem(KEPT) ?? "{}") as Places;
  } catch {
    return {};
  }
};

const write = (places: Places): void => {
  try {
    localStorage.setItem(KEPT, JSON.stringify(places));
  } catch {
    return;
  }
};

// Which block was at the top rather than how far down: a page not yet laid out only guesses its heights.
export const remembered = (path: string): number | undefined => read()[path];

export const remember = (path: string, block: number): void => {
  const { [path]: _, ...others } = read();
  const kept = Object.fromEntries(Object.entries(others).slice(-(AT_MOST - 1)));
  write(block > 0 ? { ...kept, [path]: block } : kept);
};

export const blockAt = (page: HTMLElement, y: number): number => {
  const x = page.getBoundingClientRect().left + page.clientWidth / 2;
  let there = document.elementFromPoint(x, y);
  while (there && there.parentElement !== page) there = there.parentElement;
  return there ? Array.prototype.indexOf.call(page.children, there) : 0;
};
