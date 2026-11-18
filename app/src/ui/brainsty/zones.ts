import type { Neuron } from "../../brainsty";
import { HUE_KEYS, rgba } from "./looks";
import type { Spots } from "./settling";

export type Point = readonly [number, number];

export interface Zone {
  group: string;
  hue: string;
  hull: Point[];
}

const FEWEST = 6;

const cross = (o: Point, a: Point, b: Point): number =>
  (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);

export function hullOf(points: readonly Point[]): Point[] {
  const sorted = [...points].sort((one, other) => one[0] - other[0] || one[1] - other[1]);
  if (sorted.length < 3) return sorted;
  const half = (from: Point[]): Point[] => {
    const kept: Point[] = [];
    for (const one of from) {
      while (kept.length >= 2 && cross(kept[kept.length - 2], kept[kept.length - 1], one) <= 0) {
        kept.pop();
      }
      kept.push(one);
    }
    kept.pop();
    return kept;
  };
  return [...half(sorted), ...half([...sorted].reverse())];
}

const stableHue = (group: string): string => {
  let sum = 0;
  for (const one of group) sum = (sum * 31 + (one.codePointAt(0) ?? 0)) % 1_000_003;
  return HUE_KEYS[sum % HUE_KEYS.length];
};

export function zonesOf(
  nodes: readonly Neuron[],
  { x, y }: Spots,
  hueOf: (group?: string) => string | undefined,
): Zone[] {
  const members = new Map<string, Point[]>();
  nodes.forEach((one, n) => {
    if (!one.group) return;
    const points = members.get(one.group) ?? [];
    points.push([x[n], y[n]]);
    members.set(one.group, points);
  });
  return [...members.entries()]
    .filter(([, points]) => points.length >= FEWEST)
    .map(([group, points]) => ({
      group,
      hue: hueOf(group) ?? stableHue(group),
      hull: hullOf(points),
    }));
}

export function paintZones(
  canvas: HTMLCanvasElement,
  zones: readonly Zone[],
  toScreen: (point: Point) => Point,
  hues: Record<string, string>,
  shown: number,
): void {
  const brush = canvas.getContext("2d");
  if (!brush) return;
  brush.clearRect(0, 0, canvas.width, canvas.height);
  if (shown <= 0) return;
  const scale = canvas.width / (canvas.clientWidth || canvas.width);
  brush.lineJoin = "round";
  brush.lineWidth = 24 * scale;
  for (const zone of zones) {
    if (zone.hull.length < 3) continue;
    brush.beginPath();
    zone.hull.forEach((point, n) => {
      const [sx, sy] = toScreen(point);
      if (n === 0) brush.moveTo(sx * scale, sy * scale);
      else brush.lineTo(sx * scale, sy * scale);
    });
    brush.closePath();
    const colour = hues[zone.hue] ?? zone.hue;
    brush.fillStyle = rgba(colour, 0.09 * shown);
    brush.strokeStyle = rgba(colour, 0.09 * shown);
    brush.fill();
    brush.stroke();
  }
}
