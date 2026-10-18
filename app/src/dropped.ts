import { getCurrentWebview } from "@tauri-apps/api/webview";

export const CATCHES = "data-catches-files";

type Caught = (target: Element, paths: string[], at: { left: number; top: number }) => void;

type Put = (written: string, at?: { left: number; top: number }) => void;

const takers = new WeakMap<Element, Put>();

export function takesFiles(target: Element, put: Put): () => void {
  takers.set(target, put);
  return () => {
    if (takers.get(target) === put) takers.delete(target);
  };
}

export function handTo(
  target: Element,
  written: string,
  at?: { left: number; top: number },
): boolean {
  const put = takers.get(target);
  put?.(written, at);
  return put !== undefined;
}

export const asCss = (x: number, y: number, mac: boolean, scale: number): [number, number] =>
  mac ? [x, y] : [x / (scale || 1), y / (scale || 1)];

export const CALENDAR = /\.(ics|ical)$/i;

const caughtAt = (x: number, y: number): Element | null => {
  const [left, top] = asCss(
    x,
    y,
    navigator.userAgent.includes("Macintosh"),
    window.devicePixelRatio,
  );
  return document.elementFromPoint(left, top)?.closest(`[${CATCHES}]`) ?? null;
};

// Away from a document a calendar brings tasks, so while one is carried there the window says so.
export function whenCalendarsHover(onHover: (on: boolean) => void): () => void {
  let stop: (() => void) | undefined;
  let live = true;
  let carried = false;
  getCurrentWebview()
    .onDragDropEvent(({ payload }) => {
      if (payload.type === "enter") carried = payload.paths.some((one) => CALENDAR.test(one));
      const over = payload.type === "enter" || payload.type === "over";
      onHover(over && carried && !caughtAt(payload.position.x, payload.position.y));
    })
    .then((off) => {
      if (!live) return off();
      stop = off;
    });
  return () => {
    live = false;
    stop?.();
  };
}

export function whenFilesLand(onCaught: Caught, onLoose?: (paths: string[]) => void): () => void {
  let stop: (() => void) | undefined;
  let live = true;

  const here = (x: number, y: number) =>
    asCss(x, y, navigator.userAgent.includes("Macintosh"), window.devicePixelRatio);

  const under = (x: number, y: number): Element | null => {
    const [left, top] = here(x, y);
    return document.elementFromPoint(left, top)?.closest(`[${CATCHES}]`) ?? null;
  };

  const paint = (target: Element | null) => {
    document.querySelectorAll(`[${CATCHES}].catching`).forEach((one) => {
      if (one !== target) one.classList.remove("catching");
    });
    target?.classList.add("catching");
  };

  getCurrentWebview()
    .onDragDropEvent(({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") {
        paint(under(payload.position.x, payload.position.y));
        return;
      }
      if (payload.type !== "drop") {
        paint(null);
        return;
      }
      const target = under(payload.position.x, payload.position.y);
      const [left, top] = here(payload.position.x, payload.position.y);
      paint(null);
      if (target && payload.paths.length) onCaught(target, payload.paths, { left, top });
      else if (payload.paths.length) onLoose?.(payload.paths);
    })
    .then((off) => {
      if (!live) return off();
      stop = off;
    });

  return () => {
    live = false;
    stop?.();
    paint(null);
  };
}
