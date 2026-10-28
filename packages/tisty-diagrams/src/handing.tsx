import { createContext, useContext } from "react";
import type { Draw } from "./exporting";
import { type Say, sayIn } from "./words";

/** What the app the studio lives in hands it: its words, Mermaid's drawing, the clipboard and its looks. */
export interface Hands {
  readonly say: Say;
  readonly draw: Draw;
  readonly copy: (text: string) => Promise<void>;
  // The app's own classes, so the studio's buttons look like the rest of it.
  readonly looks: Looks;
}

export interface Looks {
  readonly button: string;
}

const Handed = createContext<Hands>({
  say: sayIn("en"),
  draw: () => Promise.resolve(null),
  copy: () => Promise.resolve(),
  looks: { button: "" },
});

export const HandsGiven = Handed.Provider;
export const useHands = (): Hands => useContext(Handed);
export const useSay = (): Say => useContext(Handed).say;
