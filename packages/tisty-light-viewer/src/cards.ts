import { cardKey, cardsOn, cardsPlaced } from "tisty-sheet";
import { cardsOf } from "./host";

// Only one's own document draws its cards, and only a document's: a task cited there stays its words.
export const cardsDrawn = async (page: Document): Promise<() => void> => {
  const asked = cardsOn(page).filter((one) => one.kind === "doc");
  const drawn = asked.length > 0 ? await cardsOf(asked) : [];
  const cards = new Map(
    asked.flatMap((one, at) => (drawn[at] === undefined ? [] : [[cardKey(one), drawn[at]]])),
  );
  return () => void cardsPlaced(page, cards);
};
