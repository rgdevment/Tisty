import { letDraftGo } from "./drafts";
import { setAside } from "./keptAside";

export const PATIENCE = 30_000;

export type SaveTurn = { hand: string; n: number };

type Sent = { n: number; words: string };

type Left = Sent & { stuck: boolean };

type Holder = { flush: () => void; resend?: () => void; holding?: () => boolean };

type Line = { last: Promise<unknown>; era: number };

const lines = new Map<string, Line>();
const unanswered = new Map<string, Left>();
const mayLand = new Map<string, Sent[]>();
const holders = new Set<Holder>();
let pending = 0;

const hand = Array.from(crypto.getRandomValues(new Uint32Array(2)), (part) =>
  part.toString(36),
).join("");
let turns = 0;

export const turned = (): SaveTurn => {
  turns += 1;
  return { hand, n: turns };
};

const lineOf = (id: string): Line => {
  const line = lines.get(id) ?? { last: Promise.resolve(), era: 0 };
  lines.set(id, line);
  return line;
};

export const queued = <T>(id: string, run: () => Promise<T>): Promise<T> => {
  const line = lineOf(id);
  pending += 1;
  const mine = line.last.then(run).finally(() => {
    pending -= 1;
  });
  const last = mine.catch(() => {});
  line.last = last;
  void last.then(() => {
    if (lines.get(id) === line && line.last === last) lines.delete(id);
  });
  return mine;
};

export const codeOf = (problem: unknown): string | undefined =>
  (problem as { code?: string } | null)?.code;

export const stuck = (problem: unknown): boolean => codeOf(problem) === "saveStuck";

const mayHaveWritten = (problem: unknown): boolean =>
  stuck(problem) || codeOf(problem) === "internal";

const SET_ASIDE = new Set(["documentTooLong", "documentLocked"]);

const FOR_GOOD = new Set([...SET_ASIDE, "noSuchDoc"]);

export const refusedForGood = (problem: unknown): boolean => FOR_GOOD.has(codeOf(problem) ?? "");

const within = <T>(work: Promise<T>): Promise<T> =>
  new Promise<T>((done, failed) => {
    const late = setTimeout(() => failed({ code: "saveStuck" }), PATIENCE);
    work.then(done, failed).finally(() => clearTimeout(late));
  });

const stopped = (id: string, one: Sent) => {
  const first = mayLand.get(id)?.[0];
  mayLand.set(id, first ? [first, one] : [one]);
};

const forget = (id: string) => {
  unanswered.delete(id);
  mayLand.delete(id);
};

export const kept = <T>(
  id: string,
  words: string,
  run: (turn: SaveTurn) => Promise<T>,
): Promise<T> => {
  const line = lineOf(id);
  const era = line.era;
  const uncut = () => line.era === era;
  return queued(id, () => {
    if (!uncut()) return Promise.reject({ code: "saveLate" });
    const turn = turned();
    return within(run(turn)).then(
      (done) => {
        forget(id);
        return done;
      },
      (problem: unknown) => {
        if (!uncut()) throw { code: "saveLate" };
        if (mayHaveWritten(problem)) stopped(id, { n: turn.n, words });
        if (SET_ASIDE.has(codeOf(problem) ?? "")) setAside(id, words);
        if (refusedForGood(problem)) {
          unanswered.delete(id);
          letDraftGo(id);
        } else if (codeOf(problem) !== "saveLate")
          unanswered.set(id, { n: turn.n, words, stuck: stuck(problem) });
        throw problem;
      },
    );
  });
};

export const landedWords = (id: string, n?: number): string | undefined =>
  mayLand.get(id)?.find((one) => one.n === n)?.words;

export const readAgain = (id: string): void => {
  mayLand.delete(id);
};

export const lostTrack = (id: string, n?: number): boolean =>
  mayLand.has(id) && landedWords(id, n) === undefined;

export const unansweredWords = (id: string): string | undefined => unanswered.get(id)?.words;

export const stalledOn = (id: string): boolean => unanswered.get(id)?.stuck ?? false;

export const allUnanswered = (): [string, string][] =>
  [...unanswered].map(([id, one]) => [id, one.words]);

export const cut = (id: string): void => {
  const line = lines.get(id);
  if (line) line.era += 1;
  forget(id);
  letDraftGo(id);
};

export const busy = (id: string): Promise<unknown> | undefined => lines.get(id)?.last;

export const holds = (flush: () => void, also: Omit<Holder, "flush"> = {}): (() => void) => {
  const holder = { flush, ...also };
  holders.add(holder);
  return () => {
    holders.delete(holder);
  };
};

const holding = () => [...holders].some((one) => one.holding?.() ?? false);

const lasts = () => [...lines.values()].map((one) => one.last);

const landed = async (): Promise<void> => {
  await Promise.allSettled(lasts());
  await Promise.allSettled(lasts());
};

export const settled = async (): Promise<void> => {
  for (const one of holders) one.flush();
  await landed();
};

export const settledThen = (then: () => Promise<void>) => (): void => {
  void settled().finally(() => void then());
};

const resent = () => {
  for (const one of holders) (one.resend ?? one.flush)();
};

export const resentThen = (then: () => Promise<void>) => (): void => {
  resent();
  void landed().finally(() => void then());
};

export const unsettled = (): boolean => pending > 0 || unanswered.size > 0 || holding();

export const leftUnsaved = async (): Promise<boolean> => {
  resent();
  let late: ReturnType<typeof setTimeout> | undefined;
  const gaveUp = new Promise<boolean>((done) => {
    late = setTimeout(() => done(true), PATIENCE);
  });
  try {
    return await Promise.race([landed().then(() => unanswered.size > 0 || holding()), gaveUp]);
  } finally {
    clearTimeout(late);
  }
};
