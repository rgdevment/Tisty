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
  let low = 0;
  let high = page.children.length - 1;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    if (page.children[middle].getBoundingClientRect().bottom >= y) high = middle;
    else low = middle + 1;
  }
  return Math.max(low, 0);
};
