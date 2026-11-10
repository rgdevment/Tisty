import { type RefObject, useEffect, useRef, useState } from "react";

export function useRoom(held: RefObject<HTMLElement | null>, axis: "width" | "height"): number {
  const [room, setRoom] = useState(Number.POSITIVE_INFINITY);
  useEffect(() => {
    const el = held.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const seen = new ResizeObserver(([entry]) =>
      setRoom(axis === "width" ? entry.contentRect.width : entry.contentRect.height),
    );
    seen.observe(el);
    return () => seen.disconnect();
  }, [held, axis]);
  return room;
}

export const fitting = (many: number, room: number, each: number): number =>
  many * each <= room ? many : Math.max(0, Math.floor((room - each) / each));

export function useFades<T extends HTMLElement>() {
  const held = useRef<T>(null);
  useEffect(() => {
    const el = held.current;
    if (!el) return;
    const shade = () => {
      el.style.setProperty("--fade-top", el.scrollTop > 1 ? "18px" : "0px");
      const below = el.scrollTop + el.clientHeight < el.scrollHeight - 1;
      el.style.setProperty("--fade-bottom", below ? "24px" : "0px");
    };
    shade();
    el.addEventListener("scroll", shade, { passive: true });
    const grown = typeof MutationObserver === "undefined" ? null : new MutationObserver(shade);
    grown?.observe(el, { childList: true, subtree: true });
    const sized = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(shade);
    sized?.observe(el);
    return () => {
      el.removeEventListener("scroll", shade);
      grown?.disconnect();
      sized?.disconnect();
    };
  }, []);
  return held;
}
