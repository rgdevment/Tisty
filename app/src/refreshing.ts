const GATHERED_MS = 200;

export type Refresh = {
  readonly tasks: boolean;
  readonly papers: boolean;
  readonly fresh: boolean;
  readonly seen: boolean;
};

const NONE: Refresh = { tasks: false, papers: false, fresh: false, seen: false };

// A round tells the window several times in one breath, and each telling redraws the whole window.
export const gathering = (refresh: (asked: Refresh) => void, within = GATHERED_MS) => {
  let asked = NONE;
  let waiting: ReturnType<typeof setTimeout> | undefined;
  const ask = (more: Partial<Refresh>) => {
    asked = {
      tasks: asked.tasks || Boolean(more.tasks),
      papers: asked.papers || Boolean(more.papers),
      fresh: asked.fresh || Boolean(more.fresh),
      seen: asked.seen || Boolean(more.seen),
    };
    if (waiting !== undefined) return;
    waiting = setTimeout(() => {
      waiting = undefined;
      const now = asked;
      asked = NONE;
      refresh(now);
    }, within);
  };
  const stop = () => {
    clearTimeout(waiting);
    waiting = undefined;
    asked = NONE;
  };
  return { ask, stop };
};
