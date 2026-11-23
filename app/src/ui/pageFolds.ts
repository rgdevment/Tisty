import { elementOf, wordsEndAt } from "./pageNodes";

const FOLD = ".page details[data-at]";

type FoldReach = { readonly block: Element; readonly whole: boolean };

const foldsEnded = (range: AbstractRange) => {
  const ended: Element[] = [];
  let fold = elementOf(range.endContainer)?.closest(FOLD) ?? null;
  while (fold && wordsEndAt(fold, range.endContainer, range.endOffset)) {
    ended.push(fold);
    fold = fold.parentElement?.closest(FOLD) ?? null;
  }
  return ended;
};

const lastBlockIn = (fold: Element, start: Node): Element | null => {
  const blocks = fold.querySelectorAll(":scope > [data-at]");
  const last = blocks[blocks.length - 1] ?? null;
  return last?.matches(FOLD) && last.contains(start) ? lastBlockIn(last, start) : last;
};

export const foldReached = (range: AbstractRange): FoldReach | null => {
  if (range.collapsed) return null;
  const ended = foldsEnded(range);
  const left = ended.filter((fold) => !fold.contains(range.startContainer));
  const outer = left[left.length - 1];
  if (outer) return { block: outer, whole: true };
  const amongWords = elementOf(range.endContainer)?.closest("[data-src]");
  const last = ended[0] && !amongWords ? lastBlockIn(ended[0], range.startContainer) : null;
  return last ? { block: last, whole: last.matches(FOLD) } : null;
};
