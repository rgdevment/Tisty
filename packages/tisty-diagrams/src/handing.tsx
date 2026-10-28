import { createContext, useContext } from "react";
import type { Draw } from "./exporting";
import { type Say, sayIn } from "./words";

/** What the app the studio lives in hands it: its words, Mermaid's drawing and the clipboard. */
export interface Hands {
  readonly say: Say;
  readonly draw: Draw;
  readonly copy: (text: string) => Promise<void>;
}

const Handed = createContext<Hands>({
  say: sayIn("en"),
  draw: () => Promise.resolve(null),
  copy: () => Promise.resolve(),
});

export const HandsGiven = Handed.Provider;
export const useHands = (): Hands => useContext(Handed);
export const useSay = (): Say => useContext(Handed).say;
