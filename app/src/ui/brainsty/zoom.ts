export type Level = "far" | "mid" | "near";

const NEAR_AT = 0.08;
const STEP = 0.05;
const MID_FROM = 0.25;
const NEAR_FROM = 0.7;

const clamp = (value: number): number => Math.min(1, Math.max(0, value));

// How close the camera is, from 0 with the whole network in view to 1 near enough to read every title.
export const zoomOf = (ratio: number): number => {
  if (!(ratio > 0)) return 0;
  return clamp(Math.log(1 / ratio) / Math.log(1 / NEAR_AT));
};

export const quantized = (zoom: number): number => Math.round(clamp(zoom) / STEP) * STEP;

export const levelOf = (zoom: number): Level => {
  if (zoom < MID_FROM) return "far";
  return zoom < NEAR_FROM ? "mid" : "near";
};

export const ramp = (zoom: number, from: number, to: number): number =>
  clamp((zoom - from) / (to - from));

export const bigTitles = (zoom: number): number => ramp(zoom, MID_FROM - 0.05, MID_FROM + 0.05);

export const allTitles = (zoom: number): number => ramp(zoom, NEAR_FROM - 0.05, NEAR_FROM + 0.05);

export const lineDetail = (zoom: number): number => ramp(zoom, NEAR_FROM - 0.1, NEAR_FROM + 0.05);

export const zonesShown = (zoom: number): number =>
  Math.min(
    ramp(zoom, MID_FROM - 0.1, MID_FROM + 0.05),
    1 - ramp(zoom, NEAR_FROM - 0.05, NEAR_FROM + 0.1),
  );

export const thumbsShown = (zoom: number): boolean => zoom >= NEAR_FROM + 0.05;

export const iconsShown = (zoom: number): number => ramp(zoom, NEAR_FROM - 0.05, NEAR_FROM + 0.05);

export const lerp = (from: number, to: number, at: number): number => from + (to - from) * at;

// Up close a node grows enough to hold the icon of what it is, and sigma's clicks find it at that size.
export const ICON_ROOM = 9;

export const roomFor = (size: number, zoom: number): number =>
  lerp(size, Math.max(size, ICON_ROOM), iconsShown(zoom));
