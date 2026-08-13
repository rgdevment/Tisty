const queues = new Map<string, Promise<unknown>>();

let waiting: (() => void) | null = null;

export const queued = <T>(id: string, run: () => Promise<T>): Promise<T> => {
  const before = queues.get(id) ?? Promise.resolve();
  const mine = before.catch(() => {}).then(run);
  queues.set(
    id,
    mine.catch(() => {}),
  );
  return mine;
};

export const busy = (id: string): Promise<unknown> | undefined => queues.get(id);

export const holds = (flush: () => void): (() => void) => {
  waiting = flush;
  return () => {
    if (waiting === flush) waiting = null;
  };
};

export const settled = async (): Promise<void> => {
  waiting?.();
  await Promise.allSettled([...queues.values()]);
  await Promise.allSettled([...queues.values()]);
};
