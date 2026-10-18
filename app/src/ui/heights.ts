import { useEffect, useRef } from "react";

const KEY = "tisty.height:";

const read = (key: string): number | undefined => {
  try {
    const said = Number(sessionStorage.getItem(KEY + key));
    return said > 0 ? said : undefined;
  } catch {
    return undefined;
  }
};

/** The height a page last had, so it opens at that size instead of growing into it. */
export function useKeptHeight<T extends HTMLElement>(key: string) {
  const ref = useRef<T>(null);
  useEffect(() => {
    const box = ref.current;
    if (!box || typeof ResizeObserver === "undefined") return;
    const seeing = new ResizeObserver(() => {
      const tall = Math.round(box.getBoundingClientRect().height);
      if (tall <= 0) return;
      try {
        sessionStorage.setItem(KEY + key, String(tall));
      } catch {}
    });
    seeing.observe(box);
    return () => seeing.disconnect();
  }, [key]);
  return { ref, height: read(key) };
}
