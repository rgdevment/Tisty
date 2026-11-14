import { readAs } from "tisty-editor/read";
import { staged } from "tisty-sheet";
import { locale } from "../locales";
import { cardsFor } from "./carding";
import { datedSteps, diagramBars, languaged, pictures, type Reaching } from "./dressing";
import { SAY } from "./saying";
import { tabled } from "./tabled";
import { GLIMPSES } from "./webcarding";
import { gripped } from "./widening";

// The blocks the core drew again while the page is written on are read on a page of their own, as the whole
// page is read, and laid back; what counts across the page, as its stages, is counted again on the page.
export const wornLaid = (
  page: Document,
  laid: readonly Element[],
  live: () => boolean,
  reach: Reaching,
): (() => void)[] => {
  const staging = page.implementation.createHTMLDocument("");
  staging.documentElement.className = page.documentElement.className;
  const sheet = staging.createElement("article");
  sheet.className = "page";
  staging.body.append(sheet);
  const marks = laid.map((one) => {
    const mark = page.createComment("");
    one.replaceWith(mark);
    sheet.append(one);
    return mark;
  });
  if (reach.onDiagram || reach.onDiagramExport || reach.onSource)
    diagramBars(staging, {
      open: Boolean(reach.onDiagram),
      out: Boolean(reach.onDiagramExport),
      words: Boolean(reach.onSource),
    });
  // Rows are numbered before their cells are drawn, as on the whole page, or each cell names no row.
  if (reach.onTable) tabled(staging);
  let held: (() => void)[] = [];
  const read = readAs(staging, {
    say: SAY,
    locale: locale(),
    live,
    writable: Boolean(reach.onTable),
    brought: () => {
      held = pictures(staging, live, { more: Boolean(reach.onMore) });
    },
    cards: cardsFor(page, reach),
    parts: reach.part,
    sites: { glimpses: GLIMPSES },
  });
  marks.forEach((mark, at) => {
    const one = laid[at];
    if (one) mark.replaceWith(one);
  });
  if (reach.onTable) tabled(page);
  if (reach.onWiden) gripped(page);
  if (reach.onLanguage) languaged(page);
  for (const one of page.querySelectorAll('.page li.task[id^="stage--"]'))
    one.removeAttribute("id");
  staged(page);
  datedSteps(page, reach.stepDates);
  return [...held, ...read.kept];
};
