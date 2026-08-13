import { getCurrentWebview } from "@tauri-apps/api/webview";

export const CATCHES = "data-catches-files";

type Caught = (target: Element, paths: string[]) => void;

const takers = new WeakMap<Element, (written: string) => void>();

export function takesFiles(target: Element, put: (written: string) => void): () => void {
  takers.set(target, put);
  return () => {
    if (takers.get(target) === put) takers.delete(target);
  };
}

export function handTo(target: Element, written: string): boolean {
  const put = takers.get(target);
  put?.(written);
  return put !== undefined;
}

export function whenFilesLand(onCaught: Caught): () => void {
  let stop: (() => void) | undefined;
  let live = true;

  const under = (x: number, y: number): Element | null => {
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
