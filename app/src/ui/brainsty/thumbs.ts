import type { Point } from "./zones";

export interface Thumbs {
  place: (toScreen: (node: number) => Point | undefined, shown: boolean) => void;
  clear: () => void;
}

const SIDE = 44;
const MOST = 24;

export function thumbsIn(
  layer: HTMLElement,
  files: readonly { node: number; id: string }[],
  urlOf: (id: string) => Promise<string>,
): Thumbs {
  const pictures = new Map<string, HTMLImageElement>();
  const picture = (id: string): HTMLImageElement => {
    const had = pictures.get(id);
    if (had) return had;
    const img = layer.ownerDocument.createElement("img");
    img.alt = "";
    img.hidden = true;
    Object.assign(img.style, {
      position: "absolute",
      top: "0",
      left: "0",
      width: `${SIDE}px`,
      height: `${SIDE}px`,
      objectFit: "cover",
      borderRadius: "6px",
      boxShadow: "var(--tisty-lift)",
      pointerEvents: "none",
    });
    layer.append(img);
    pictures.set(id, img);
    urlOf(id).then(
      (url) => {
        img.src = url;
      },
      () => img.remove(),
    );
    return img;
  };
  const hideAll = () => {
    for (const img of pictures.values()) img.hidden = true;
  };
  return {
    place: (toScreen, shown) => {
      hideAll();
      if (!shown) return;
      const { clientWidth: width, clientHeight: height } = layer;
      const seen = files
        .map((one) => ({ id: one.id, at: toScreen(one.node) }))
        .filter((one): one is { id: string; at: Point } => {
          const at = one.at;
          return at !== undefined && at[0] >= 0 && at[1] >= 0 && at[0] <= width && at[1] <= height;
        })
        .sort(
          (one, other) =>
            Math.hypot(one.at[0] - width / 2, one.at[1] - height / 2) -
            Math.hypot(other.at[0] - width / 2, other.at[1] - height / 2),
        )
        .slice(0, MOST);
      for (const one of seen) {
        const img = picture(one.id);
        img.style.transform = `translate(${one.at[0] + 8}px, ${one.at[1] - SIDE / 2}px)`;
        img.hidden = false;
      }
    },
    clear: () => {
      for (const img of pictures.values()) img.remove();
      pictures.clear();
    },
  };
}
