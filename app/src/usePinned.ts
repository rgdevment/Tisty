import { useEffect, useMemo, useState } from "react";
import { brainstormOf, type Pinned } from "./brainstorm";
import type { Papers } from "./core";

// Ideas changes with its own sheet or with which documents there are, so the words saved in another ask nothing.
const lookedFor = (papers: Papers, sheet: string | undefined): string =>
  papers.docs
    .map((one) =>
      one.file === sheet
        ? JSON.stringify(one)
        : `${one.file}:${one.archived}:${one.away}:${one.pageOf ?? ""}`,
    )
    .join("|");

export function usePinned(papers: Papers): Pinned | null {
  const [pinned, setPinned] = useState<Pinned | null>(null);
  const asked = useMemo(() => lookedFor(papers, pinned?.file), [papers, pinned?.file]);
  useEffect(() => {
    let current = true;
    brainstormOf()
      .then((found) => current && setPinned(found))
      .catch(() => current && setPinned(null));
    return () => {
      current = false;
    };
  }, [asked]);
  return pinned;
}
