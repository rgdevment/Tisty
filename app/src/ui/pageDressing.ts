import { readAs } from "tisty-editor/read";
import { staged } from "tisty-sheet";
import { locale } from "../locales";
import { cardsFor } from "./carding";
import { datedSteps, diagramBars, languaged, mathBars, pictures, type Reaching } from "./dressing";
import { keptInPlace, takenOver } from "./pagePatching";
import { SAY } from "./saying";
import { tabled } from "./tabled";
import { GLIMPSES } from "./webcarding";
import { gripped } from "./widening";

const dressedAway = (one: Element, sheet: Element) => {
  const mark = one.ownerDocument.createComment("");
  one.replaceWith(mark);
  sheet.append(one);
  return () => mark.replaceWith(one);
};

// Taking a block out of the page restyles every paragraph of a long one, so a stand-in of its kind is dressed.
const dressedThrough = (one: Element, sheet: Element) => {
  const stand = sheet.ownerDocument.importNode(one, false);
  // A player taken to another page starts over, so a copy without its sound goes in its place while its words are dressed.
  const players = Array.from(one.querySelectorAll(".media"), (player) => {
    const copy = player.cloneNode(true) as Element;
    for (const sound of copy.querySelectorAll("audio, video")) sound.remove();
    player.replaceWith(copy);
    return [copy, player] as const;
  });
  stand.append(...one.childNodes);
  sheet.append(stand);
  return () => {
    takenOver(one, stand);
    for (const [copy, player] of players) copy.replaceWith(player);
  };
};

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
  const back = laid.map((one) =>
    keptInPlace(one) ? dressedThrough(one, sheet) : dressedAway(one, sheet),
  );
  if (reach.onDiagram || reach.onDiagramExport)
    diagramBars(staging, { open: Boolean(reach.onDiagram), out: Boolean(reach.onDiagramExport) });
  if (reach.onPiece) mathBars(staging);
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
    tasks: reach.cited && { task: reach.cited, acting: Boolean(reach.onTaskDone) },
    sites: { glimpses: GLIMPSES },
  });
  for (const put of back) put();
  if (reach.onTable) tabled(page);
  if (reach.onWiden) gripped(page);
  if (reach.onLanguage) languaged(page);
  staged(page);
  datedSteps(page, reach.stepDates);
  return [...held, ...read.kept];
};
