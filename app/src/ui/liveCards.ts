import { type CardAsked, cardKey } from "tisty-editor/read";
import { type CardsSeen, cardsDrawn } from "../citing";
import { dateOf, whenLabel } from "../format";

export type CardsShown = {
  readonly docs?: boolean;
  readonly tasks?: boolean;
  readonly examples?: boolean;
  readonly here?: string;
  readonly heard?: unknown;
};

const wanted = (asked: readonly CardAsked[], shown?: CardsShown): CardAsked[] => {
  if (!shown) return [];
  const keys = new Set<string>();
  return asked.filter((one) => {
    const kept = one.kind === "doc" ? shown.docs : shown.tasks;
    const key = cardKey(one);
    if (!kept || keys.has(key)) return false;
    keys.add(key);
    return true;
  });
};

export const drawnFor = async (
  asked: readonly CardAsked[],
  shown: CardsShown | undefined,
  seen: CardsSeen,
): Promise<Map<string, string>> => {
  const asks = wanted(asked, shown);
  if (asks.length === 0) return new Map();
  const drawn = await cardsDrawn(asks, seen);
  return new Map(
    asks.flatMap((one, at) => (drawn[at] === undefined ? [] : [[cardKey(one), drawn[at]]])),
  );
};

const closedOn = (iso: string): string => {
  const at = Date.parse(iso);
  return Number.isNaN(at) ? "" : dateOf(at / 1000);
};

export const cardDays = (box: Element) => {
  for (const time of box.querySelectorAll<HTMLElement>("time[data-say]")) {
    const at = time.getAttribute("datetime") ?? "";
    const has_time = time.hasAttribute("data-timed");
    time.textContent =
      time.dataset.say === "day"
        ? closedOn(at)
        : whenLabel({ at, tz: "", floating: true, has_time });
  }
};
