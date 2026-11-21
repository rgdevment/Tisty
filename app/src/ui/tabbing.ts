import type { KeyboardEvent } from "react";

const BACK: Record<string, boolean> = { ArrowLeft: true, ArrowUp: true };
const ON: Record<string, boolean> = { ArrowRight: true, ArrowDown: true };

export const nextTab = <T>(key: string, all: readonly T[], at: T, vertical: boolean): T | null => {
  const here = all.indexOf(at);
  if (here < 0 || all.length === 0) return null;
  if (key === "Home") return all[0];
  if (key === "End") return all[all.length - 1];
  const along = vertical ? ["ArrowUp", "ArrowDown"] : ["ArrowLeft", "ArrowRight"];
  if (!along.includes(key)) return null;
  if (BACK[key]) return all[(here - 1 + all.length) % all.length];
  if (ON[key]) return all[(here + 1) % all.length];
  return null;
};

export function tabKeys<T>(
  all: readonly T[],
  vertical: boolean,
  idOf: (one: T) => string,
  onMove?: (one: T) => void,
) {
  return (event: KeyboardEvent<HTMLElement>, at: T) => {
    const next = nextTab(event.key, all, at, vertical);
    if (next === null) return;
    event.preventDefault();
    onMove?.(next);
    document.getElementById(idOf(next))?.focus();
  };
}
