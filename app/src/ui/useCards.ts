import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Heard, Which, Word } from "./Card";
import type { Working } from "./working";

const KEPT_MS = 1800;
const SAID_MS = 4000;
const PER_CHAR_MS = 45;
const MOST_MS = 12000;

export const fadesAfter = (text: string): number =>
  text === t("settingsKept")
    ? KEPT_MS
    : Math.min(MOST_MS, Math.max(SAID_MS, text.length * PER_CHAR_MS));

export type Work<T> = Promise<T> | (() => Promise<T>);

export interface Cards {
  busy: Working;
  said: Heard;
  trouble: Heard;
  start: (card: Which) => void;
  stop: (card: Which) => void;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  clear: (card: Which) => void;
  run: <T>(card: Which, work: Work<T>, done: (answer: T) => void) => void;
  quietly: <T>(card: Which, work: Work<T>, done: (answer: T) => void) => void;
}

const without = <V>(all: Partial<Record<Which, V>>, card: Which): Partial<Record<Which, V>> => {
  if (!(card in all)) return all;
  const { [card]: _gone, ...rest } = all;
  return rest;
};

export function useCards(after: () => void): Cards {
  const [busy, setBusy] = useState<Working>(new Map());
  const [said, setSaid] = useState<Heard>({});
  const [trouble, setTrouble] = useState<Heard>({});
  const then = useRef(after);
  then.current = after;
  const fading = useRef(new Map<Which, ReturnType<typeof setTimeout>>());

  useEffect(() => {
    const timers = fading.current;
    return () => {
      for (const timer of timers.values()) clearTimeout(timer);
    };
  }, []);

  const start = useCallback((card: Which) => {
    setBusy((was) => new Map(was).set(card, Date.now()));
  }, []);

  const stop = useCallback((card: Which) => {
    setBusy((was) => {
      if (!was.has(card)) return was;
      const now = new Map(was);
      now.delete(card);
      return now;
    });
  }, []);

  const hush = useCallback((card: Which) => {
    clearTimeout(fading.current.get(card));
    fading.current.delete(card);
    setSaid((was) => without(was, card));
  }, []);

  const tell = useCallback(
    (word?: Word) => {
      if (!word) return;
      const card = word.card;
      clearTimeout(fading.current.get(card));
      setSaid((was) => ({ ...was, [card]: word }));
      setTrouble((was) => without(was, card));
      fading.current.set(
        card,
        setTimeout(() => hush(card), fadesAfter(word.text)),
      );
    },
    [hush],
  );

  const fail = useCallback(
    (word?: Word) => {
      if (!word) return setTrouble({});
      hush(word.card);
      setTrouble((was) => ({ ...was, [word.card]: word }));
    },
    [hush],
  );

  const clear = useCallback(
    (card: Which) => {
      hush(card);
      setTrouble((was) => without(was, card));
    },
    [hush],
  );

  const quietly = useCallback(
    <T>(card: Which, work: Work<T>, done: (answer: T) => void) => {
      const again = typeof work === "function" ? () => quietly(card, work, done) : undefined;
      start(card);
      clear(card);
      (typeof work === "function" ? work() : work)
        .then(done)
        .catch((e) =>
          setTrouble((was) => ({ ...was, [card]: { card, text: saidPlainly(e), again } })),
        )
        .finally(() => stop(card));
    },
    [start, stop, clear],
  );

  const run = useCallback(
    <T>(card: Which, work: Work<T>, done: (answer: T) => void) =>
      quietly(card, work, (answer) => {
        done(answer);
        then.current();
      }),
    [quietly],
  );

  return useMemo(
    () => ({ busy, said, trouble, start, stop, tell, fail, clear, run, quietly }),
    [busy, said, trouble, start, stop, tell, fail, clear, run, quietly],
  );
}
