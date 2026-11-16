import { useEffect, useState } from "react";
import { brainstormOf, type Pinned } from "./brainstorm";
import type { Papers } from "./core";

export function usePinned(papers: Papers): Pinned | null {
  const [pinned, setPinned] = useState<Pinned | null>(null);
  useEffect(() => {
    let current = true;
    brainstormOf()
      .then((found) => current && setPinned(found))
      .catch(() => current && setPinned(null));
    return () => {
      current = false;
    };
  }, [papers]);
  return pinned;
}
