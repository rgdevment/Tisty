import type { Network } from "../../brainsty";
import type { Drawing, HueOf } from "./drawing";
import type { Palette } from "./looks";
import type { Spots } from "./settling";
import { thumbsIn } from "./thumbs";
import { paintZones, type Zone, zonesOf } from "./zones";
import { thumbsShown, zonesShown } from "./zoom";

export const LAID = {
  position: "absolute",
  inset: "0",
  width: "100%",
  height: "100%",
  pointerEvents: "none",
};

export interface Overlays {
  reshape: (spots: Spots) => void;
  sift: (hidden: readonly boolean[]) => void;
  retheme: (palette: Palette) => void;
  stop: () => void;
}

export interface Pictures {
  shows: (id: string) => boolean;
  urlOf: (id: string) => Promise<string>;
}

export function fitted(canvas: HTMLCanvasElement, container: HTMLElement): number {
  const ratio = container.ownerDocument.defaultView?.devicePixelRatio ?? 1;
  const width = Math.round(container.clientWidth * ratio);
  const height = Math.round(container.clientHeight * ratio);
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
  return ratio;
}

export function overlaysOn(
  container: HTMLElement,
  drawing: Drawing,
  network: Network,
  hueOf: HueOf,
  palette: Palette,
  pictures: Pictures,
): Overlays {
  const page = container.ownerDocument;
  const canvas = page.createElement("canvas");
  Object.assign(canvas.style, LAID);
  container.prepend(canvas);
  const files = network.nodes.flatMap((one, node) =>
    one.kind === "file" && pictures.shows(one.id) ? [{ node, id: one.id }] : [],
  );
  const thumbs = thumbsIn(container, files, pictures.urlOf);
  let zones: Zone[] = [];
  let placed: Spots | null = null;
  let hidden: readonly boolean[] = [];
  const zoned = () => {
    if (!placed) return;
    const { x, y } = placed;
    const seen = network.nodes.flatMap((_, n) => (hidden[n] ? [] : [n]));
    zones = zonesOf(
      seen.map((n) => network.nodes[n]),
      { x: Float32Array.from(seen, (n) => x[n]), y: Float32Array.from(seen, (n) => y[n]) },
      hueOf,
    );
  };
  let hues = palette.hues;
  const frame = () => {
    fitted(canvas, container);
    const zoom = drawing.zoom();
    paintZones(canvas, zones, drawing.toScreen, hues, zonesShown(zoom));
    thumbs.place(
      (node) => (hidden[node] ? undefined : drawing.nodeOnScreen(node)),
      thumbsShown(zoom),
    );
  };
  const unhook = drawing.onFrame(frame);
  return {
    reshape: (spots) => {
      if (spots.x.length !== network.nodes.length) return;
      placed = spots;
      zoned();
    },
    sift: (next) => {
      hidden = next;
      zoned();
    },
    retheme: (next) => {
      hues = next.hues;
    },
    stop: () => {
      unhook();
      thumbs.clear();
      canvas.remove();
    },
  };
}
