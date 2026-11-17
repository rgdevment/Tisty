import type { Carded, Cards } from "tisty-sheet";
import { cardsOf } from "./host";

const named = (one: Carded | undefined, untitled: string): Carded => {
  if (!one) return { kind: "gone" };
  return one.kind !== "gone" && !one.title.trim() ? { ...one, title: untitled } : one;
};

// The sheet asks for each card at once, so what every card names is brought before it draws them.
export const cardsIn = async (page: HTMLElement, untitled: string): Promise<Omit<Cards, "say">> => {
  const files = new Set(
    Array.from(
      page.querySelectorAll<HTMLElement>("a.card[data-doc]"),
      (one) => one.dataset.doc ?? "",
    ),
  );
  const seen = files.size > 0 ? await cardsOf([...files]) : {};
  return { card: (file) => named(seen[file], untitled) };
};
