import type { Network } from "../../brainsty";
import type { Drawing } from "./drawing";
import { alphaOf, type Curve, curvesOf, curvesShown, type Showing, type Shown } from "./guessing";
import { type Palette, rgba } from "./looks";
import { fitted, LAID } from "./overlays";
import type { Spots } from "./settling";
import type { Point } from "./zones";

export interface Curves {
  reshape: (spots: Spots) => void;
  show: (on: boolean) => void;
  pick: (node: number | null) => void;
  sift: (hidden: readonly boolean[]) => void;
  retheme: (palette: Palette) => void;
  stop: () => void;
}

const BEND = 0.2;
export const WIDTH = 1.5;

export const bendOf = ([x1, y1]: Point, [x2, y2]: Point): Point => [
  (x1 + x2) / 2 - (y2 - y1) * BEND,
  (y1 + y2) / 2 + (x2 - x1) * BEND,
];

const outside = (points: readonly Point[], width: number, height: number): boolean =>
  points.every(([x]) => x < 0) ||
  points.every(([x]) => x > width) ||
  points.every(([, y]) => y < 0) ||
  points.every(([, y]) => y > height);

export function paintCurves(
  canvas: HTMLCanvasElement,
  shown: readonly Shown[],
  onScreen: (node: number) => Point | undefined,
  amber: string,
  scale: number,
): void {
  const brush = canvas.getContext("2d");
  if (!brush) return;
  brush.clearRect(0, 0, canvas.width, canvas.height);
  brush.lineWidth = WIDTH * scale;
  brush.lineCap = "round";
  const { width, height } = canvas;
  for (const { curve, fade } of shown) {
    const from = onScreen(curve.from);
    const to = onScreen(curve.to);
    if (!from || !to) continue;
    const [a, bend, b] = [from, bendOf(from, to), to].map(
      ([x, y]): Point => [x * scale, y * scale],
    );
    if (outside([a, bend, b], width, height)) continue;
    brush.strokeStyle = rgba(amber, alphaOf(curve.strength) * fade);
    brush.beginPath();
    brush.moveTo(a[0], a[1]);
    brush.quadraticCurveTo(bend[0], bend[1], b[0], b[1]);
    brush.stroke();
  }
}

// Laid beneath sigma's canvases, so a curve runs under the nodes it joins; the zones go beneath it.
export function curvesOn(
  container: HTMLElement,
  drawing: Drawing,
  network: Network,
  palette: Palette,
): Curves {
  const canvas = container.ownerDocument.createElement("canvas");
  Object.assign(canvas.style, LAID);
  container.prepend(canvas);
  const curves: Curve[] = curvesOf(network.alike ?? []);
  const showing: Omit<Showing, "zoom" | "inView"> = { on: true, picked: null, hidden: [] };
  let amber = palette.hues.amber;
  let placed: Spots | null = null;
  // Near up, a guess shows only between two nodes in view: one reaching far would cross the whole view.
  const inViewNow = (): ((node: number) => boolean) => {
    if (!placed) return () => false;
    const { x, y } = placed;
    const corners = [
      drawing.toGraph([0, 0]),
      drawing.toGraph([container.clientWidth, container.clientHeight]),
    ];
    const [left, right] = [
      Math.min(corners[0][0], corners[1][0]),
      Math.max(corners[0][0], corners[1][0]),
    ];
    const [top, bottom] = [
      Math.min(corners[0][1], corners[1][1]),
      Math.max(corners[0][1], corners[1][1]),
    ];
    return (node) => x[node] >= left && x[node] <= right && y[node] >= top && y[node] <= bottom;
  };
  const paint = () => {
    const scale = fitted(canvas, container);
    const shown = curvesShown(curves, { ...showing, zoom: drawing.zoom(), inView: inViewNow() });
    paintCurves(canvas, shown, drawing.nodeOnScreen, amber, scale);
  };
  const unhook = curves.length > 0 ? drawing.onFrame(paint) : () => undefined;
  const repaint = () => curves.length > 0 && paint();
  return {
    reshape: (spots) => {
      if (spots.x.length === network.nodes.length) placed = spots;
    },
    show: (on) => {
      showing.on = on;
      repaint();
    },
    pick: (node) => {
      showing.picked = node;
      repaint();
    },
    sift: (hidden) => {
      showing.hidden = hidden;
      repaint();
    },
    retheme: (next) => {
      amber = next.hues.amber;
      repaint();
    },
    stop: () => {
      unhook();
      canvas.remove();
    },
  };
}
