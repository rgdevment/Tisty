export type Keyed = {
  readonly find: () => void;
  readonly side: () => void;
  readonly theme: () => void;
  readonly zoom: (step: number) => void;
  readonly head: (by: number) => void;
  readonly back: () => void;
  readonly source: () => void;
  readonly print: () => void;
  readonly close: () => void;
  readonly escape: () => void;
};

const withControl = (keyed: Keyed, e: KeyboardEvent): (() => void) | undefined => {
  const key = e.key.toLowerCase();
  if (e.shiftKey) {
    if (key === "m") return keyed.source;
    if (key === "l") return keyed.theme;
    return key === "+" ? () => keyed.zoom(1) : undefined;
  }
  const acts: Record<string, () => void> = {
    f: keyed.find,
    "\\": keyed.side,
    p: keyed.print,
    w: keyed.close,
    "=": () => keyed.zoom(1),
    "+": () => keyed.zoom(1),
    "-": () => keyed.zoom(-1),
    "0": () => keyed.zoom(0),
  };
  return acts[key];
};

export const keyHeard =
  (keyed: Keyed) =>
  (e: KeyboardEvent): void => {
    let act: (() => void) | undefined;
    if (e.key === "Escape") act = keyed.escape;
    else if (e.ctrlKey || e.metaKey) act = withControl(keyed, e);
    else if (e.altKey && e.key === "ArrowDown") act = () => keyed.head(1);
    else if (e.altKey && e.key === "ArrowUp") act = () => keyed.head(-1);
    else if (e.altKey && e.key === "ArrowLeft") act = keyed.back;
    if (!act) return;
    e.preventDefault();
    act();
  };
