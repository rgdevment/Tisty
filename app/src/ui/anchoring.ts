const shared = (was: string, now: string): [number, number] => {
  const most = Math.min(was.length, now.length);
  let head = 0;
  while (head < most && was[head] === now[head]) head++;
  let tail = 0;
  while (tail < most - head && was[was.length - 1 - tail] === now[now.length - 1 - tail]) tail++;
  return [head, tail];
};

// Null when `now` changed inside the block, which then ends: it spoke of the old words.
export const reanchored = (
  was: string,
  now: string,
  from: number,
  end: number,
): [number, number] | null => {
  const [head, tail] = shared(was, now);
  if (head >= end) return [from, end];
  if (was.length - tail > from) return null;
  const moved = now.length - was.length;
  return [from + moved, end + moved];
};

// In bytes, as the page's `data-at` counts them.
export const changedIn = (was: string, now: string): [number, number] | null => {
  if (was === now) return null;
  const [head, tail] = shared(was, now);
  const bytes = (upto: number) => new TextEncoder().encode(now.slice(0, upto)).length;
  return [bytes(head), bytes(now.length - tail)];
};

// A passage taken out leaves no bytes, so the blocks either side of where it was count as touched.
export const touches = (at: readonly [number, number], span: readonly [number, number]): boolean =>
  span[0] === span[1] ? at[0] <= span[0] && at[1] >= span[0] : at[0] < span[1] && at[1] > span[0];
