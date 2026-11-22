import { docMerge, docOpened, docWrite, type Opened } from "./core";
import { kept, landedWords, lostTrack, readAgain, turned, unansweredWords } from "./saving";

const REMEMBERS = 12;

export type Read = { text: string; print?: string; unwoven?: boolean };

export const remembered = (held: Map<string, Read>, file: string, read: Read) => {
  held.delete(file);
  held.set(file, read);
  while (held.size > REMEMBERS) {
    const oldest = held.keys().next().value;
    if (oldest === undefined) break;
    held.delete(oldest);
  }
};

export const opening = (file: string): Promise<Opened & { left?: boolean }> => {
  const left = unansweredWords(file);
  if (left !== undefined) return Promise.resolve({ body: left, print: "", left: true });
  readAgain(file);
  return docOpened(file, turned());
};

export const savedOnce = (
  id: string,
  text: string,
  anyway: boolean | undefined,
  read: () => Read | undefined,
) =>
  kept(id, text, async (turn) => {
    try {
      const fresh = await docWrite(id, text, anyway, read()?.print, turn);
      return { fresh, woven: null };
    } catch (e) {
      const refused = e as { code?: string; landed?: number } | null;
      const base = landedWords(id, refused?.landed) ?? read()?.text;
      if (refused?.code !== "documentMoved" || base === undefined) throw e;
      if (lostTrack(id, refused.landed)) throw e;
      const woven = await docMerge(id, base, text, turn);
      if (!woven) throw e;
      return { fresh: null, woven };
    }
  });
