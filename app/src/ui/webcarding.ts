import type { Glimpse, Glimpses } from "tisty-editor/read";
import { glimpseFetch, glimpseKept } from "../core";

// One question to the core per link and window, however many times the page is drawn again.
const asked = new Map<string, Promise<Glimpse | null>>();

export const GLIMPSES: Glimpses = {
  kept: (at) => {
    const was = asked.get(at);
    if (was) return was;
    const said = glimpseKept(at).catch(() => null);
    asked.set(at, said);
    return said;
  },
  fetched: (at) =>
    glimpseFetch(at).then((said) => {
      if (said && (said.title || said.said || said.shot)) asked.set(at, Promise.resolve(said));
      return said;
    }),
};
