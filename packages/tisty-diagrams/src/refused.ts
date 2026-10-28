import type { Word } from "./words";

/** A failure named by one of the studio's words, so the app around it can say it in its language. */
export class Refused extends Error {
  constructor(readonly code: Word) {
    super(code);
    this.name = "Refused";
  }
}
