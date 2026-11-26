type Timer = ReturnType<typeof setTimeout>;

export interface Beat {
  soon: () => void;
  stop: () => void;
}

// Typing without a pause still lands every `most` ms, or a crash would take everything since the last pause.
export const beat = (fire: () => void, settles: number, most: number): Beat => {
  let timer: Timer | null = null;
  let since: number | null = null;
  const stop = () => {
    if (timer) clearTimeout(timer);
    timer = null;
    since = null;
  };
  const soon = () => {
    const now = Date.now();
    since ??= now;
    if (timer) clearTimeout(timer);
    timer = setTimeout(
      () => {
        timer = null;
        since = null;
        fire();
      },
      Math.max(0, Math.min(settles, since + most - now)),
    );
  };
  return { soon, stop };
};

export const BACKING_OFF = [1_000, 2_000, 4_000, 8_000, 15_000];

export interface Backoff {
  later: () => void;
  reset: () => void;
}

export const backoff = (fire: () => void, waits: readonly number[] = BACKING_OFF): Backoff => {
  let tries = 0;
  let timer: Timer | null = null;
  const later = () => {
    if (timer) return;
    const wait = waits[Math.min(tries, waits.length - 1)];
    tries += 1;
    timer = setTimeout(() => {
      timer = null;
      fire();
    }, wait);
  };
  const reset = () => {
    if (timer) clearTimeout(timer);
    timer = null;
    tries = 0;
  };
  return { later, reset };
};
