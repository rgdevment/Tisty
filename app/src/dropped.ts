import { getCurrentWebview } from "@tauri-apps/api/webview";

/** Marks the element that accepts files; the drop arrives by position, not by event. */
export const CATCHES = "data-catches-files";

type Caught = (target: Element, paths: string[]) => void;

/**
 * Routed by the element itself, not by a name: a name outlives the field that
 * chose it, and the file would land in whatever mounted next under it.
 */
const takers = new WeakMap<Element, (written: string) => void>();

export function takesFiles(target: Element, put: (written: string) => void): () => void {
  takers.set(target, put);
  return () => takers.delete(target);
}

/** False when the field is gone, so the caller can say so instead of losing it. */
export function handTo(target: Element, written: string): boolean {
  const put = takers.get(target);
  put?.(written);
  return put !== undefined;
}

/**
 * The webview swallows OS file drags, so they never reach an element as a DOM
 * event: they arrive once, globally, with a position to look under.
 */
export function whenFilesLand(onCaught: Caught): () => void {
  let stop: (() => void) | undefined;
  let live = true;

  const under = (x: number, y: number): Element | null => {
    // Device pixels in, CSS pixels out: at 125% the point lands a quarter low.
    const scale = window.devicePixelRatio || 1;
    return document.elementFromPoint(x / scale, y / scale)?.closest(`[${CATCHES}]`) ?? null;
  };

  const paint = (target: Element | null) => {
    document
      .querySelectorAll(`[${CATCHES}].catching`)
      .forEach((one) => one !== target && one.classList.remove("catching"));
    target?.classList.add("catching");
  };

  getCurrentWebview()
    .onDragDropEvent(({ payload }) => {
      // `enter` carries the paths too, and it is not a drop.
      if (payload.type === "enter" || payload.type === "over") {
        paint(under(payload.position.x, payload.position.y));
        return;
      }
      if (payload.type !== "drop") {
        paint(null);
        return;
      }
      const target = under(payload.position.x, payload.position.y);
      paint(null);
      if (target && payload.paths.length) onCaught(target, payload.paths);
    })
    .then((off) => (live ? (stop = off) : off()));

  return () => {
    live = false;
    stop?.();
    paint(null);
  };
}
