import { served } from "./core";

export const ONCE_A_DAY = 24 * 60 * 60 * 1000;

const shown = new Map<string, number>();
let timer: ReturnType<typeof setInterval> | undefined;

// A picture left on screen for weeks is still in use, so it is asked for again while it stays.
export function showing(reference: string): () => void {
  shown.set(reference, (shown.get(reference) ?? 0) + 1);
  timer ??= setInterval(() => {
    for (const one of shown.keys()) served(one).catch(() => undefined);
  }, ONCE_A_DAY);
  let gone = false;
  return () => {
    if (gone) return;
    gone = true;
    const left = (shown.get(reference) ?? 1) - 1;
    if (left > 0) shown.set(reference, left);
    else shown.delete(reference);
    if (shown.size === 0 && timer !== undefined) {
      clearInterval(timer);
      timer = undefined;
    }
  };
}
