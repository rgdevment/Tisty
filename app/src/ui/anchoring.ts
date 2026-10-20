/**
 * Where the block `[from, end)` of `was` sits in `now`, or null when `now` changed inside it: a
 * change elsewhere leaves the block being written open, one inside it ends it.
 */
export const reanchored = (
  was: string,
  now: string,
  from: number,
  end: number,
): [number, number] | null => {
  const most = Math.min(was.length, now.length);
  let head = 0;
  while (head < most && was[head] === now[head]) head++;
  let tail = 0;
  while (tail < most - head && was[was.length - 1 - tail] === now[now.length - 1 - tail]) tail++;
  if (head >= end) return [from, end];
  if (was.length - tail > from) return null;
  const moved = now.length - was.length;
  return [from + moved, end + moved];
};

/** The bytes of `now` that differ from `was`, as the page's `data-at` counts them; null when none do. */
export const changedIn = (was: string, now: string): [number, number] | null => {
  if (was === now) return null;
  const most = Math.min(was.length, now.length);
  let head = 0;
  while (head < most && was[head] === now[head]) head++;
  let tail = 0;
  while (tail < most - head && was[was.length - 1 - tail] === now[now.length - 1 - tail]) tail++;
  const bytes = (upto: number) => new TextEncoder().encode(now.slice(0, upto)).length;
  return [bytes(head), bytes(now.length - tail)];
};
