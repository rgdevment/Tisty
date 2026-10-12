import { ask } from "@tauri-apps/plugin-dialog";
import { docs, type Pick, paperRifts, type Rift, settlePaper, weavePaper } from "./core";
import { fill, t } from "./locales";
import { type Refusal, saidPlainly } from "./refusal";

const later = new Set<string>();
const reminded = new Set<string>();
const decidedAt = new Map<string, number>();
let decisions = 0;
let queue: Promise<unknown> = Promise.resolve();

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
  const said = called?.trim() || t("untitledDoc");
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
};

export const putOffSaid = (count: number): string =>
  count === 1 ? t("putOffDocIsOne") : fill("putOffDocs", String(count));

export interface Decided {
  shut: string[];
  said: string | null;
  putOff: string[];
}

export const decideAll = (ids: string[], byHand = false): Promise<Decided> => {
  // What was decided after this round looked is newer than what it saw, so it is not asked twice.
  const since = decisions;
  const turn = queue.then(() => decideInTurn(ids, byHand, since));
  queue = turn.catch(() => undefined);
  return turn;
};

const decideInTurn = async (ids: string[], byHand: boolean, since: number): Promise<Decided> => {
  const fresh = ids.filter((id) => (decidedAt.get(id) ?? since) <= since);
  const asking = fresh.filter((id) => byHand || !later.has(id));
  const putOff = byHand ? [] : fresh.filter((id) => later.has(id) && !reminded.has(id));
  const finish = (shut: string[], said: string | null): Decided => {
    for (const id of putOff) reminded.add(id);
    return { shut, said, putOff };
  };
  if (!asking.length) return finish([], null);
  const told = new Set<string>();
  const found = await docs().catch((problem: unknown) => {
    told.add(saidPlainly(problem));
    return null;
  });
  if (!found) return finish([], [...told].join(" "));
  const named = (id: string) =>
    found.docs.find((one) => one.file === id)?.title?.trim() || t("untitledDoc");
  const shut = new Set(found.docs.filter((one) => one.locked).map((one) => one.file));
  for (const id of asking) {
    if (shut.has(id)) continue;
    try {
      const settled = await decide(id, named(id));
      decisions += 1;
      decidedAt.set(id, decisions);
      if (settled) {
        later.delete(id);
        reminded.delete(id);
      } else {
        later.add(id);
        putOff.push(id);
      }
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
  return finish(locked, [...lockedSaid, ...told].join(" ") || null);
};
