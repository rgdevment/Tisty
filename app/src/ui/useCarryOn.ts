import { useCallback, useEffect, useRef } from "react";
import { onEnded } from "../coming";
import { useRows } from "./RowNotes";
import { held as isHeld } from "./working";

const AGAIN_AFTER = 10_000;

/** Runs a round, and again once free when another was holding the cloud, so what it waits on moves by itself. */
export function useCarryOn(carry: () => void): () => void {
  const { said, trouble, busy } = useRows();
  const held = isHeld(busy, "sync");
  const told = said.sync;
  const wrong = trouble.sync;
  // The sign-in that asks for the round still holds the cloud when it ends, and a carry read then is dropped.
  const free = useRef(carry);
  if (!held) free.current = carry;
  const holding = useRef(held);
  holding.current = held;
  const owed = useRef(false);
  const letGo = useRef<() => void>(undefined);

  useEffect(() => {
    if (wrong || (told && !told.busy)) owed.current = false;
    if (!owed.current || !told?.busy) return;
    letGo.current?.();
    const again = () => {
      letGo.current?.();
      if (holding.current) return wait();
      free.current();
    };
    const wait = () => {
      const off = onEnded(again);
      const waiting = setTimeout(again, AGAIN_AFTER);
      letGo.current = () => {
        off();
        clearTimeout(waiting);
      };
    };
    wait();
  }, [told, wrong]);
  useEffect(() => () => letGo.current?.(), []);

  return useCallback(() => {
    owed.current = true;
    free.current();
  }, []);
}
