import type { ReactNode } from "react";
import type { Heard, Word } from "../ui/Card";
import { RowsContext } from "../ui/RowNotes";
import { type Cards, useCards } from "../ui/useCards";
import { IDLE, type Working } from "../ui/working";

export const heard = (word?: Word): Heard => (word ? { [word.card]: word } : {});

export function Rows({
  busy = IDLE,
  said,
  trouble,
  children,
}: {
  busy?: Working;
  said?: Word;
  trouble?: Word;
  children: ReactNode;
}) {
  return (
    <RowsContext.Provider value={{ busy, said: heard(said), trouble: heard(trouble) }}>
      {children}
    </RowsContext.Provider>
  );
}

export function LiveRows({ children }: { children: (cards: Cards) => ReactNode }) {
  const cards = useCards(() => {});
  return <RowsContext.Provider value={cards}>{children(cards)}</RowsContext.Provider>;
}
