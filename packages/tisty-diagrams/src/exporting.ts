import { type Size, sizeOf } from "./framing";
import { PALETTES, svgOf } from "./picture";
import { pictureOf } from "./pictures";

/** Mermaid's own drawing of a diagram as SVG, from whichever app holds Mermaid; null when it cannot. */
export type Draw = (source: string, dark: boolean, still?: () => boolean) => Promise<string | null>;

const SCALE = 2;
// Past these a browser hands back no image at all: WebKit holds a canvas to about 16 million pixels.
const MOST_SIDE = 16384;
const MOST_AREA = 16_000_000;

export interface Drawn {
  readonly svg: string;
  readonly size: Size;
}

// A refusal by its code, so the app around says it in its own words.
const unreadable = () => ({ code: "diagramUnreadable" });
const unpainted = () => ({ code: "diagramUnpainted" });

// Mermaid draws to the width of its place; a picture on its own needs a size, and a ground when asked.
export const grounded = (svg: string, size: Size, ground: string | null): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${size.wide}" height="${size.tall}" viewBox="0 0 ${size.wide} ${size.tall}">${ground ? `<rect width="100%" height="100%" fill="${ground}"/>` : ""}${svg}</svg>`;

// A drawing too large to paint at twice its size is painted as large as the browser allows.
export const scaleFor = (size: Size): number =>
  Math.min(
    SCALE,
    MOST_SIDE / size.wide,
    MOST_SIDE / size.tall,
    Math.sqrt(MOST_AREA / (size.wide * size.tall)),
  );

export const svgFor = async (
  source: string,
  dark: boolean,
  ground: boolean,
  font: string,
  draw: Draw,
): Promise<Drawn> => {
  const palette = PALETTES[dark ? "dark" : "light"];
  const back = ground ? palette.sheet : null;
  const own = pictureOf(source);
  if (own) return { svg: svgOf(own, palette, back, font), size: own.size };
  const drawn = await draw(source, dark);
  const size = drawn && sizeOf(drawn);
  if (!drawn || !size) throw unreadable();
  return { svg: grounded(drawn, size, back), size };
};

// Twice its size, so the image stays sharp on a dense screen.
export const rasterized = (drawn: Drawn, type: "image/png" | "image/jpeg"): Promise<Uint8Array> =>
  new Promise((done, fail) => {
    const image = new Image();
    image.onload = () => {
      const scale = scaleFor(drawn.size);
      const canvas = document.createElement("canvas");
      canvas.width = Math.ceil(drawn.size.wide * scale);
      canvas.height = Math.ceil(drawn.size.tall * scale);
      const pen = canvas.getContext("2d");
      if (!pen) return fail(unpainted());
      pen.scale(scale, scale);
      pen.drawImage(image, 0, 0, drawn.size.wide, drawn.size.tall);
      canvas.toBlob(
        (blob) =>
          blob
            ? blob.arrayBuffer().then((bytes) => done(new Uint8Array(bytes)), fail)
            : fail(unpainted()),
        type,
        0.92,
      );
    };
    image.onerror = () => fail(unpainted());
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(drawn.svg)}`;
  });
