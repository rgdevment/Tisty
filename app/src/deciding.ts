import { useSyncExternalStore } from "react";
import {
  docs,
  type Pick,
  paperRifts,
  type Rift,
  settlePaper,
  stillAtOdds,
  taskOf,
  weavePaper,
} from "./core";
import { fill, t } from "./locales";
import { type Refusal, saidPlainly } from "./refusal";
import { sure } from "./sure";

const later = new Set<string>();
const reminded = new Set<string>();
const decidedAt = new Map<string, number>();
let decisions = 0;
let queue: Promise<unknown> = Promise.resolve();
const watching = new Set<() => void>();
const tell = () => {
  for (const one of watching) one();
};

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
  if (await sure(fill("bothChanged", said))) return "both";
  return (await sure(fill("whoseWins", said))) ? "mine" : "theirs";
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

const letGo = (id: string) => {
  later.delete(id);
  reminded.delete(id);
};

// A task's page is left out of the documents, so it is named after the task it belongs to.
const taskTitled = (id: string): Promise<string | undefined> => {
  const task = /^t_([0-9a-z]{26})-\d+$/i.exec(id)?.[1];
  if (!task) return Promise.resolve(undefined);
  return taskOf(task.toUpperCase()).then(
    (one) => one.title?.trim() || undefined,
    () => undefined,
  );
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
    tell();
    return { shut, said, putOff };
  };
  if (!asking.length) return finish([], null);
  const told = new Set<string>();
  const found = await docs().catch((problem: unknown) => {
    told.add(saidPlainly(problem));
    return null;
  });
  if (!found) return finish([], [...told].join(" "));
  const tasks = new Map<string, string>();
  for (const id of asking.filter((one) => !found.docs.some((doc) => doc.file === one))) {
    const title = await taskTitled(id);
    if (title) tasks.set(id, title);
  }
  const named = (id: string) =>
    found.docs.find((one) => one.file === id)?.title?.trim() || tasks.get(id) || t("untitledDoc");
  const shut = new Set(found.docs.filter((one) => one.locked).map((one) => one.file));
  for (const id of asking) {
    if (shut.has(id)) continue;
    try {
      const settled = await decide(id, named(id));
      decisions += 1;
      decidedAt.set(id, decisions);
      if (settled) {
        letGo(id);
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
      // Still at odds, the next round names it again; settled elsewhere, it would otherwise wait forever.
      letGo(id);
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

export const leftForLater = (): string[] => [...later];

let sifting: Promise<void> | null = null;

export const settledElsewhere = (): Promise<void> => {
  if (sifting || !later.size) return sifting ?? Promise.resolve();
  const ids = [...later];
  const since = decisions;
  sifting = stillAtOdds(ids)
    .catch(() => ids)
    .then((said) => {
      const still = new Set(said);
      // One decided again while this looked was looked at too early to let go.
      const gone = ids.filter(
        (id) => !still.has(id) && later.has(id) && (decidedAt.get(id) ?? 0) <= since,
      );
      for (const id of gone) letGo(id);
      if (gone.length) tell();
    })
    .finally(() => {
      sifting = null;
    });
  return sifting;
};

const subscribe = (change: () => void) => {
  watching.add(change);
  return () => {
    watching.delete(change);
  };
};

export const useLeftForLater = (): number => useSyncExternalStore(subscribe, () => later.size);
