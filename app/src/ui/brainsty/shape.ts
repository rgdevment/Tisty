import type { Spots } from "./settling";

const WIDE = 1.3;
const TALL = 0.92;
const FISSURE = Math.PI + 0.62;
const CEREBELLUM = -0.78;
const KEPT_REACH = 0.97;

const turned = (angle: number): number => Math.atan2(Math.sin(angle), Math.cos(angle));

const bump = (angle: number, at: number, width: number): number =>
  Math.exp(-(turned(angle - at) ** 2) / width);

export function outline(angle: number): number {
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const ellipse = (WIDE * TALL) / Math.hypot(TALL * cos, WIDE * sin);
  const flatBottom = 1 - 0.22 * Math.max(0, -sin) ** 3;
  const fissure = 1 - 0.16 * bump(angle, FISSURE, 0.012);
  const cerebellum = 1 + 0.1 * bump(angle, CEREBELLUM, 0.04);
  return ellipse * flatBottom * fissure * cerebellum;
}

// Each node keeps its direction from the centre and its rank in distance, so zones stay whole inside the silhouette.
export function brainShaped({ x, y }: Spots): Spots {
  const count = x.length;
  if (count < 3) return { x: x.slice(), y: y.slice() };
  let [cx, cy] = [0, 0];
  for (let n = 0; n < count; n++) {
    cx += x[n];
    cy += y[n];
  }
  cx /= count;
  cy /= count;
  const away = Array.from({ length: count }, (_, n) => Math.hypot(x[n] - cx, y[n] - cy));
  const order = away.map((_, n) => n).sort((one, other) => away[one] - away[other]);
  const reach = away[order[Math.min(count - 1, Math.floor(count * KEPT_REACH))]] || 1;
  const shapedX = new Float32Array(count);
  const shapedY = new Float32Array(count);
  order.forEach((n, rank) => {
    const angle = Math.atan2(y[n] - cy, x[n] - cx);
    const depth = Math.sqrt((rank + 0.5) / count) * reach * outline(angle);
    shapedX[n] = cx + Math.cos(angle) * depth;
    shapedY[n] = cy + Math.sin(angle) * depth;
  });
  return { x: shapedX, y: shapedY };
}
