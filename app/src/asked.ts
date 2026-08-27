import { useEffect, useRef, useState } from "react";

/// A late answer must not land on a newer question, and a stale `onError` must not relaunch the ask.
export function useAsked<T>(
  ask: () => Promise<T>,
  on: unknown[],
  onError?: (problem: unknown) => void,
): T | null {
  const [held, setHeld] = useState<T | null>(null);
  const warn = useRef(onError);
  warn.current = onError;
  const asking = useRef(ask);
  asking.current = ask;

  useEffect(() => {
    let alive = true;
    setHeld(null);
    asking
      .current()
      .then((answer) => {
        if (alive) setHeld(answer);
      })
      .catch((problem) => {
        if (alive) warn.current?.(problem);
      });
    return () => {
      alive = false;
    };
    // biome-ignore lint/correctness/useExhaustiveDependencies: the caller says what the ask depends on
  }, on);

  return held;
}
