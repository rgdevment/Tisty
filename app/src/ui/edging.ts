import { type RefObject, useEffect, useState } from "react";

// The card fits its content and centres itself, so only its own box says where it starts.
export function useEdge(card: RefObject<HTMLElement | null>, on?: boolean): number | undefined {
  const [edge, setEdge] = useState<number>();

  useEffect(() => {
    const held = card.current;
    const main = held?.closest("main");
    if (!on || !held || !main) return;
    const measure = () =>
      setEdge(Math.round(held.getBoundingClientRect().left - main.getBoundingClientRect().left));
    measure();
    const seen = new ResizeObserver(measure);
    seen.observe(held);
    seen.observe(main);
    return () => seen.disconnect();
  }, [card, on]);

  return edge;
}
