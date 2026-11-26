import { noteTrouble } from "./core";
import { stored } from "./stored";

/** Words a clash kept from the disk, with the body they were typed over, so a restart can still weave them. */
export type Draft = { words: string; base: string };

const drafted = (one: unknown): one is Draft =>
  typeof (one as Draft | null)?.words === "string" && typeof (one as Draft).base === "string";

const drafts = stored<Record<string, Draft>>("tisty.clashing", (said) =>
  Object.fromEntries(Object.entries(said).filter((one): one is [string, Draft] => drafted(one[1]))),
);

export const draftOf = (id: string): Draft | undefined => drafts.current()[id];

// A draft the browser had no room for lives only in this window, so the log says so.
export const keepDraft = (id: string, draft: Draft): void => {
  if (!drafts.put({ ...drafts.current(), [id]: draft }))
    noteTrouble("draftUnkept", id).catch(() => {});
};

// A draft of a document no longer listed has nothing left to be woven into.
export const keepDraftsOf = (alive: ReadonlySet<string>): void => {
  const now = drafts.current();
  const left = Object.entries(now).filter(([id]) => alive.has(id));
  if (left.length < Object.keys(now).length) drafts.put(Object.fromEntries(left));
};

export const letDraftGo = (id: string): void => {
  if (draftOf(id) === undefined) return;
  const { [id]: _, ...rest } = drafts.current();
  drafts.put(rest);
};
