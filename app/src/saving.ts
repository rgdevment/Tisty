export const PATIENCE = 30_000;

export type SaveTurn = { hand: string; n: number };

type Left = { n: number; words: string; stuck: boolean };

type Holder = { flush: () => void; resend?: () => void; holding?: () => boolean };

type Line = { last: Promise<unknown>; era: number };

const lines = new Map<string, Line>();
const unanswered = new Map<string, Left[]>();
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

const within = <T>(work: Promise<T>): Promise<T> =>
  new Promise<T>((done, failed) => {
    const late = setTimeout(() => failed({ code: "saveStuck" }), PATIENCE);
    work.then(done, failed).finally(() => clearTimeout(late));
  });

const left = (id: string, one: Left) =>
  unanswered.set(id, [...(unanswered.get(id) ?? []).filter((was) => was.stuck), one]);

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
        unanswered.delete(id);
        return done;
      },
      (problem: unknown) => {
        if (!uncut()) throw { code: "saveLate" };
        if (codeOf(problem) !== "saveLate") left(id, { n: turn.n, words, stuck: stuck(problem) });
        throw problem;
      },
    );
  });
};

export const landedWords = (id: string, n?: number): string | undefined =>
  unanswered.get(id)?.find((one) => one.n === n)?.words;

const latest = (id: string): Left | undefined => unanswered.get(id)?.slice(-1)[0];

export const unansweredWords = (id: string): string | undefined => latest(id)?.words;

export const stalledOn = (id: string): boolean => latest(id)?.stuck ?? false;

export const allUnanswered = (): [string, string][] =>
  [...unanswered].flatMap(([id, left]) =>
    left.slice(-1).map((one): [string, string] => [id, one.words]),
  );

export const forgotten = (id: string): void => {
  unanswered.delete(id);
};

export const cut = (id: string): void => {
  const line = lines.get(id);
  if (line) line.era += 1;
  unanswered.delete(id);
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
