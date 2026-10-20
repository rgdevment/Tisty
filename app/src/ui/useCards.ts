import { useCallback, useRef, useState } from "react";
import { saidPlainly } from "../refusal";
import type { Which, Word } from "./Card";

export function useCards(after: () => void) {
  const [busy, setHeld] = useState<Which | null>(null);
  const [busySince, setBusySince] = useState(0);
  const [said, setSaid] = useState<Word>();
  const [trouble, setTrouble] = useState<Word>();
  const then = useRef(after);
  then.current = after;

  const setBusy = useCallback((card: Which | null) => {
    setHeld(card);
    setBusySince(card ? Date.now() : 0);
  }, []);

  const quietly = <T>(card: Which, work: Promise<T>, done: (answer: T) => void) => {
    setBusy(card);
    setSaid(undefined);
    setTrouble(undefined);
    work
      .then(done)
      .catch((e) => setTrouble({ card, text: saidPlainly(e) }))
      .finally(() => setBusy(null));
  };

  const run = <T>(card: Which, work: Promise<T>, done: (answer: T) => void) =>
    quietly(card, work, (answer) => {
      done(answer);
      then.current();
    });

  return { busy, busySince, setBusy, said, setSaid, trouble, setTrouble, run, quietly };
}
