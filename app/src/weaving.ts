import { type Clashing, docMerge, docOpened, docWrite, type Opened, type Woven } from "./core";
import { draftOf, keepDraft, letDraftGo } from "./drafts";
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

export type Left = Opened & { left?: boolean; base?: string };

// Words still unsaved come back first; after a restart a clash's draft does, with the body it was typed over.
export const opening = (file: string): Promise<Left> => {
  const left = unansweredWords(file);
  if (left !== undefined) return Promise.resolve({ body: left, print: "", left: true });
  readAgain(file);
  const draft = draftOf(file);
  // Opened all the same, so this window holds the file's turn even when the core outlived a reload.
  if (draft)
    return docOpened(file, turned()).then(() => ({
      body: draft.words,
      print: "",
      left: true,
      base: draft.base,
    }));
  return docOpened(file, turned());
};

const wove = (merged: Woven | Clashing | null): merged is Woven =>
  merged !== null && "body" in merged;

/** How many places clashed, or `null` when standing by mine would put the whole page back; `front` when the properties did too. */
export type Clash = { torn?: number | null; front?: boolean };

type Saved = { fresh: Awaited<ReturnType<typeof docWrite>> | null; woven: Woven | null };

export const savedOnce = (
  id: string,
  text: string,
  mineStands: boolean | undefined,
  read: () => Read | undefined,
) =>
  kept(id, text, async (turn): Promise<Saved> => {
    const landed = (saved: Saved) => {
      letDraftGo(id);
      return saved;
    };
    // Words a clash leaves unsaved stay on this computer with the body they were typed over.
    const drafted = (base: string | undefined) => keepDraft(id, { words: text, base: base ?? "" });
    if (mineStands) {
      const merged = await docMerge(id, read()?.text, text, turn, "mine").catch((e) => {
        drafted(read()?.text);
        throw e;
      });
      if (!wove(merged)) throw { code: "documentMoved" };
      return landed({ fresh: null, woven: merged });
    }
    try {
      return landed({
        fresh: await docWrite(id, text, undefined, read()?.print, turn),
        woven: null,
      });
    } catch (e) {
      const refused = e as { code?: string; landed?: number } | null;
      const base = landedWords(id, refused?.landed) ?? read()?.text;
      if (refused?.code !== "documentMoved") throw e;
      if (base === undefined || lostTrack(id, refused.landed)) {
        drafted(base);
        throw e;
      }
      const merged = await docMerge(id, base, text, turn).catch((problem) => {
        drafted(base);
        throw problem;
      });
      if (wove(merged)) return landed({ fresh: null, woven: merged });
      drafted(base);
      throw { ...refused, torn: merged?.torn, front: merged?.front };
    }
  });
