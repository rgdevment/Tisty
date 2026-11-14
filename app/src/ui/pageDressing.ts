import { celledBox, diagrams, HUES, today } from "tisty-sheet";
import { locale } from "../locales";
import { diagramBar, picturesWithin } from "./dressing";
import { SAY } from "./saying";
import { dressedTable } from "./tabled";
import { darkNow } from "./theming";
import { gripsOn } from "./widening";

const tableDressed = (page: Document, box: Element) => {
  dressedTable(page, box);
  celledBox(page, box, {
    today: today(),
    locale: locale(),
    hues: HUES,
    live: true,
    say: SAY,
  });
  gripsOn(page, box.querySelectorAll<HTMLElement>("thead th"));
};

const within = (laid: readonly Element[], selector: string) =>
  laid.flatMap((one) => [
    ...(one.matches(selector) ? [one] : []),
    ...one.querySelectorAll(selector),
  ]);

// Blocks drawn again while the page is written on are dressed alone; a diagram already drawn is left as it is.
export const dressedLaid = (sheet: Element, laid: readonly Element[]) => {
  const live = () => sheet.isConnected;
  for (const one of laid) picturesWithin(one, live);
  for (const figure of within(laid, "figure.diagram"))
    diagramBar(figure as HTMLElement, { open: true, out: true, words: false });
  for (const box of within(laid, ".table")) tableDressed(sheet.ownerDocument, box);
  void diagrams(sheet, darkNow(), live);
};
