import { useLayoutEffect, useRef, useState } from "react";

const EDGE = 8;

export interface Away {
  right: boolean;
  up: boolean;
}

/**
 * Which way a floating panel has to open to stay inside the window.
 *
 * Anchored below-left with no regard for the edge, a panel opened from the
 * journal — the last thing on a long page — fell off the bottom, where nothing
 * scrolls it back. Measured after the first paint and flipped before anyone
 * sees the wrong side.
 */
export function useEdge<T extends HTMLElement>() {
  const box = useRef<T>(null);
  const [away, setAway] = useState<Away>({ right: false, up: false });

  useLayoutEffect(() => {
    const at = box.current?.getBoundingClientRect();
    if (!at) return;
    setAway({
      right: at.right > window.innerWidth - EDGE,
      // Only if there is room the other way: a panel taller than the window
      // would just fall off the top instead.
      up: at.bottom > window.innerHeight - EDGE && at.height < at.top - EDGE,
    });
  }, []);

  return { box, away };
}
