import { markup } from "tisty-editor/read";
import type { Network, Neuron, NeuronKind } from "../../brainsty";
import type { Drawing } from "./drawing";
import { fitted, LAID } from "./overlays";
import { ICON_ROOM, iconsShown, thumbsShown } from "./zoom";

export interface Icons {
  sift: (hidden: readonly boolean[]) => void;
  light: (nodes: ReadonlySet<number> | null) => void;
  retheme: () => void;
  stop: () => void;
}

const GLYPH: Record<NeuronKind, string> = {
  doc: "page",
  page: "alignleft",
  task: "checkbox",
  tag: "hash",
  file: "clip",
  folder: "folder",
  list: "list",
};

const DONE = "check-mark";
const DIMMED = 0.15;
const DRAWN_AT = 2;

export const glyphOf = (neuron: Neuron): string =>
  neuron.kind === "task" && neuron.state === "done" ? DONE : GLYPH[neuron.kind];

export const iconSource = (name: string, colour: string): string | null => {
  const drawn = markup(name);
  if (!drawn) return null;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="${24 * DRAWN_AT}" height="${24 * DRAWN_AT}" fill="none" stroke="${colour}" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">${drawn}</svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
};

export function iconsOn(
  container: HTMLElement,
  drawing: Drawing,
  network: Network,
  colourOf: (node: number) => string,
  shows: (id: string) => boolean,
): Icons {
  const canvas = container.ownerDocument.createElement("canvas");
  Object.assign(canvas.style, LAID);
  container.append(canvas);
  const glyphs = network.nodes.map(glyphOf);
  const tinted = () => network.nodes.map((_, node) => colourOf(node));
  let colours = tinted();
  let images = new Map<string, HTMLImageElement | undefined>();
  let hidden: readonly boolean[] = [];
  let lit: ReadonlySet<number> | null = null;

  const painted = () => {
    const brush = canvas.getContext("2d");
    if (!brush) return;
    fitted(canvas, container);
    brush.clearRect(0, 0, canvas.width, canvas.height);
    const shown = iconsShown(drawing.zoom());
    if (shown === 0) return;
    const ratio = canvas.width / Math.max(container.clientWidth, 1);
    const side = ICON_ROOM * 2 * ratio;
    const pictures = thumbsShown(drawing.zoom());
    network.nodes.forEach((one, node) => {
      if (hidden[node] || (pictures && one.kind === "file" && shows(one.id))) return;
      const at = drawing.nodeOnScreen(node);
      if (!at) return;
      const [x, y] = [at[0] * ratio, at[1] * ratio];
      if (x < -side || y < -side || x > canvas.width + side || y > canvas.height + side) return;
      const image = imageOf(glyphs[node], colours[node]);
      if (!image?.complete) return;
      brush.globalAlpha = shown * (lit && !lit.has(node) ? DIMMED : 1);
      brush.drawImage(image, x - side / 2, y - side / 2, side, side);
    });
    brush.globalAlpha = 1;
  };

  // Each glyph in each colour is drawn once and kept, so a frame only stamps images.
  function imageOf(glyph: string, colour: string): HTMLImageElement | undefined {
    const key = `${glyph}|${colour}`;
    const kept = images.get(key);
    if (kept) return kept;
    const source = iconSource(glyph, colour);
    if (!source) return undefined;
    const image = new Image();
    image.onload = painted;
    image.src = source;
    images.set(key, image);
    return image;
  }
  const unhook = drawing.onFrame(painted);

  return {
    sift: (next) => {
      hidden = next;
      painted();
    },
    light: (nodes) => {
      lit = nodes;
      painted();
    },
    retheme: () => {
      images = new Map();
      colours = tinted();
      painted();
    },
    stop: () => {
      unhook();
      canvas.remove();
    },
  };
}
