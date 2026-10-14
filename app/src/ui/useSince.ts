import { useEffect, useState } from "react";

const TICK = 10_000;

/** Seconds since `on` last turned true, ticking while it stays so. */
export const useSince = (on: boolean): number => {
  const [began, setBegan] = useState<number>();
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!on) {
      setBegan(undefined);
      return;
    }
    const at = Date.now();
    setBegan(at);
    setNow(at);
    const ticking = setInterval(() => setNow(Date.now()), TICK);
    return () => clearInterval(ticking);
  }, [on]);
  return began === undefined ? 0 : Math.max(0, Math.floor((now - began) / 1000));
};
