import { type Cards, carded } from "./cards";
import { celled, today } from "./cells";
import { colored } from "./coloring";
import { diagrams } from "./diagrams";
import { iconed } from "./icons";
import { type PartCarded, partCarded } from "./parts";
import { HUES, saidKinds } from "./shell";
import { sideways } from "./sideways";
import { staged } from "./stages";
import { type Glimpses, webCarded } from "./webCards";
import type { ReadWord, Say } from "./words";

// What the sheet needs from whoever shows it; what is left out is drawn as a stranger sees it, or not at all.
export type Reading = {
  readonly say: Say;
  readonly locale: string;
  readonly live: () => boolean;
  /** Stars and switches in a table are pressed, not only read. */
  readonly writable?: boolean;
  /** The host's own pictures and attachments, drawn once the cells are and before the wide tables. */
  readonly brought?: () => void;
  readonly cards?: Omit<Cards, "say">;
  readonly parts?: (id: string) => PartCarded;
  /** Web links drawn as cards of their sites, with the glimpses the host keeps, if it keeps any. */
  readonly sites?: { readonly glimpses?: Glimpses };
};

export type Read = { readonly kept: (() => void)[]; readonly drawn: Promise<void> };

// A step's words are read before anything is added to it, so a stage named by its number says only them.
export const readAs = (page: Document, reading: Reading): Read => {
  const { say, live } = reading;
  saidKinds(page, (kind) => say(`said${kind}` as ReadWord));
  iconed(page);
  staged(page);
  celled(page, {
    today: today(),
    locale: reading.locale,
    hues: HUES,
    live: Boolean(reading.writable),
    say,
  });
  reading.brought?.();
  const kept = sideways(page, say);
  if (reading.cards) carded(page, { ...reading.cards, say }, live);
  if (reading.parts) partCarded(page, { part: reading.parts, say });
  if (reading.sites) webCarded(page, say, live, reading.sites.glimpses);
  const drawn = diagrams(page, page.documentElement.classList.contains("dark"), live);
  colored(page, live);
  return { kept, drawn };
};
