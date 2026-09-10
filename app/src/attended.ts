import { useEffect, useRef } from "react";

export function useAttended<T extends HTMLElement>(when = true) {
  const one = useRef<T>(null);

  useEffect(() => {
    if (when) one.current?.focus();
  }, [when]);

  return one;
}
