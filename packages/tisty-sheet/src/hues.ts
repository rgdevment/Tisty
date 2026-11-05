export const HUES = [
  "red",
  "orange",
  "amber",
  "green",
  "teal",
  "blue",
  "indigo",
  "purple",
  "pink",
  "brown",
  "gray",
] as const;

export type Hue = (typeof HUES)[number];

export const isHue = (one: string): one is Hue => (HUES as readonly string[]).includes(one);
