const queues = new Map<string, Promise<unknown>>();

let waiting: (() => void) | null = null;

/// One queue per document, so a slow write does not hold up opening another,
/// and every one of them is reachable from outside the editor: the window is
/// asked to finish before the app leaves, and the asking happens elsewhere.
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

/// Registered by whatever holds unsaved text, so leaving can flush it without
/// reaching into the component that owns it.
export const holds = (flush: () => void): (() => void) => {
  waiting = flush;
  return () => {
    if (waiting === flush) waiting = null;
  };
};

export const settled = async (): Promise<void> => {
  waiting?.();
  await Promise.allSettled([...queues.values()]);
  // Twice: the flush above enqueues a write, and its own `onKept` can enqueue
  // the listing that follows it.
  await Promise.allSettled([...queues.values()]);
};
