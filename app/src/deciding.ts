import { ask } from "@tauri-apps/plugin-dialog";
import { docs, type Pick, paperRifts, type Rift, settlePaper, weavePaper } from "./core";
import { fill, t } from "./locales";
import { type Refusal, saidPlainly } from "./refusal";

const settling = new Set<string>();
const later = new Set<string>();
const decided = new Set<string>();
let queue: Promise<unknown> = Promise.resolve();
let queued = 0;

type Asking = (named: string, rifts: Rift[]) => Promise<Pick[] | null>;
type AskingWhole = (named: string) => Promise<Pick | null>;

let byBlock: Asking | null = null;
let whole: AskingWhole | null = null;

export const decidesByBlock = (asking: Asking | null): void => {
  byBlock = asking;
};

export const decidesWhole = (asking: AskingWhole | null): void => {
  whole = asking;
};

const askedOutright = async (said: string): Promise<Pick | null> => {
  if (whole) return whole(said);
  if (await ask(fill("bothChanged", said), { kind: "warning" })) return "both";
  return (await ask(fill("whoseWins", said), { kind: "warning" })) ? "mine" : "theirs";
};

export const decide = async (id: string, called?: string): Promise<boolean> => {
  if (settling.has(id)) return false;
  settling.add(id);
  const said = called?.trim() || t("untitledDoc");
  try {
    if (byBlock) {
      const torn = await paperRifts(id).catch(() => null);
      if (torn?.rifts.length) {
        const picks = await byBlock(said, torn.rifts);
        if (!picks) return false;
        const wove = await weavePaper(id, picks, torn.print).then(
          () => true,
          () => false,
        );
        if (wove) {
          await settlePaper(id, "mine");
          return true;
        }
      }
    }
    const keep = await askedOutright(said);
    if (!keep) return false;
    await settlePaper(id, keep, keep === "both" ? t("otherVersion") : undefined);
    return true;
  } finally {
    settling.delete(id);
  }
};

export interface Decided {
  shut: string[];
  said: string | null;
}

/** `byHand` is the person asking for a sync: what they put off before is asked again. */
export const decideAll = (ids: string[], byHand = false): Promise<Decided> => {
  queued += 1;
  const turn = queue.then(async () => {
    try {
      return await decideInTurn(ids, byHand);
    } finally {
      queued -= 1;
      // A round that waited behind this one named what was decided meanwhile, so it is not asked twice.
      if (queued === 0) decided.clear();
    }
  });
  queue = turn.catch(() => undefined);
  return turn;
};

const decideInTurn = async (ids: string[], byHand: boolean): Promise<Decided> => {
  const asking = ids.filter((id) => !decided.has(id) && (byHand || !later.has(id)));
  if (!asking.length) return { shut: [], said: null };
  const told = new Set<string>();
  const found = await docs().catch((problem: unknown) => {
    told.add(saidPlainly(problem));
    return null;
  });
  if (!found) return { shut: [], said: [...told].join(" ") };
  const named = (id: string) =>
    found.docs.find((one) => one.file === id)?.title?.trim() || t("untitledDoc");
  const shut = new Set(found.docs.filter((one) => one.locked).map((one) => one.file));
  for (const id of asking) {
    if (shut.has(id)) continue;
    try {
      const settled = await decide(id, named(id));
      decided.add(id);
      if (settled) later.delete(id);
      else later.add(id);
    } catch (problem) {
      const code = (problem as Refusal | undefined)?.code;
      if (code === "documentLocked") {
        shut.add(id);
        continue;
      }
      const plain = saidPlainly(problem);
      told.add(code === "movedUnderfoot" ? fill("changedWhileDeciding", named(id)) : plain);
    }
  }
  const locked = asking.filter((id) => shut.has(id));
  const lockedSaid = locked.length
    ? [fill("someLockedAtOdds", locked.map((id) => `«${named(id)}»`).join(", "))]
    : [];
  return { shut: locked, said: [...lockedSaid, ...told].join(" ") || null };
};
