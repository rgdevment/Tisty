const FOUND = "tisty-found";
const HERE = "tisty-found-here";

export type Sought = { readonly at: number; readonly of: number };

const rangesOf = (within: HTMLElement, said: string): Range[] => {
  const wanted = said.toLocaleLowerCase();
  const found: Range[] = [];
  const walker = document.createTreeWalker(within, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = (node.nodeValue ?? "").toLocaleLowerCase();
    for (let at = text.indexOf(wanted); at >= 0; at = text.indexOf(wanted, at + wanted.length)) {
      const range = document.createRange();
      range.setStart(node, at);
      range.setEnd(node, at + wanted.length);
      found.push(range);
    }
  }
  return found;
};

const painting = (name: string, ranges: Range[]): void => {
  if ("highlights" in CSS) CSS.highlights.set(name, new Highlight(...ranges));
};

export const unsought = (): void => {
  if (!("highlights" in CSS)) return;
  CSS.highlights.delete(FOUND);
  CSS.highlights.delete(HERE);
};

export const seeker = (within: HTMLElement) => {
  let ranges: Range[] = [];
  let at = 0;
  const show = (): Sought => {
    const range = ranges[at];
    if (!range) return { at: 0, of: 0 };
    painting(HERE, [range]);
    range.startContainer.parentElement?.scrollIntoView({ block: "center" });
    return { at: at + 1, of: ranges.length };
  };
  return {
    seek(said: string): Sought {
      ranges = said.trim() ? rangesOf(within, said) : [];
      at = 0;
      unsought();
      painting(FOUND, ranges);
      return show();
    },
    step(by: number): Sought {
      if (ranges.length > 0) at = (at + by + ranges.length) % ranges.length;
      return show();
    },
  };
};
