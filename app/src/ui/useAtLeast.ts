import { useEffect, useRef, useState } from "react";

/** Stays on for at least `ms`, so what ends at once is still seen. */
export const useAtLeast = (on: boolean, ms: number): boolean => {
  const [kept, setKept] = useState(on);
  const began = useRef(0);
  useEffect(() => {
    if (on) {
      began.current = Date.now();
      setKept(true);
      return;
    }
    const left = began.current + ms - Date.now();
    if (left <= 0) {
      setKept(false);
      return;
    }
    const ending = setTimeout(() => setKept(false), left);
    return () => clearTimeout(ending);
  }, [on, ms]);
  return on || kept;
};
