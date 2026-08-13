import { useLayoutEffect, useRef, useState } from "react";

const EDGE = 8;

export interface Away {
  right: boolean;
  up: boolean;
}

export function useEdge<T extends HTMLElement>() {
  const box = useRef<T>(null);
  const [away, setAway] = useState<Away>({ right: false, up: false });

  useLayoutEffect(() => {
    const at = box.current?.getBoundingClientRect();
    if (!at) return;
    setAway({
      right: at.right > window.innerWidth - EDGE,
      up: at.bottom > window.innerHeight - EDGE && at.height < at.top - EDGE,
    });
  }, []);

  return { box, away };
}
