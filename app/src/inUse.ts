import { served } from "./core";

export const ONCE_A_DAY = 24 * 60 * 60 * 1000;
const REFUSED_FOR = 60 * 1000;

const shown = new Map<string, number>();
const flying = new Map<string, Promise<string>>();
const refused = new Map<string, { at: number; why: unknown }>();
let timer: ReturnType<typeof setInterval> | undefined;

/**
 * One ask at a time for each attachment: with a cloud of its own, asking is bringing it down,
 * and a refusal is not asked again for a minute.
 */
export function serve(reference: string): Promise<string> {
  const failed = refused.get(reference);
  if (failed && Date.now() - failed.at < REFUSED_FOR) return Promise.reject(failed.why);
  const already = flying.get(reference);
  if (already) return already;
  const asked = served(reference)
    .then(
      (at) => {
        refused.delete(reference);
        return at;
      },
      (why: unknown) => {
        refused.set(reference, { at: Date.now(), why });
        throw why;
      },
    )
    .finally(() => flying.delete(reference));
  flying.set(reference, asked);
  return asked;
}

/** Asking again by hand is never answered with an old refusal. */
export function askAgain(reference: string): void {
  refused.delete(reference);
}

// A picture left on screen for weeks is still in use, so it is asked for again while it stays.
export function showing(reference: string): () => void {
  shown.set(reference, (shown.get(reference) ?? 0) + 1);
  timer ??= setInterval(() => {
    for (const one of shown.keys()) serve(one).catch(() => undefined);
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
