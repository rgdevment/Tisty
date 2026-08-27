import { describe, expect, it } from "vitest";
import { CAP, orientedOf, redrawn, scaled, sizedOf, upright } from "../upright";

const exif = (turn: number, little = true): number[] => {
  const tiff = little
    ? [0x49, 0x49, 0x2a, 0x00, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1, 0, 0, 0, turn, 0, 0, 0]
    : [0x4d, 0x4d, 0x00, 0x2a, 0, 0, 0, 8, 0, 1, 0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, turn, 0, 0];
  const body = [0x45, 0x78, 0x69, 0x66, 0, 0, ...tiff, 0, 0, 0, 0];
  const size = body.length + 2;
  return [0xff, 0xd8, 0xff, 0xe1, size >> 8, size & 0xff, ...body];
};

describe("the turn a camera wrote into a photo", () => {
  it("reads the turn, which the printed page would otherwise ignore", () => {
    expect(orientedOf(exif(6))).toBe(6);
    expect(orientedOf(exif(8))).toBe(8);
  });

  it("reads it whichever way round the camera wrote its numbers", () => {
    expect(orientedOf(exif(6, false))).toBe(6);
  });

  it("says no turn for a photo already the right way up", () => {
    expect(orientedOf(exif(1))).toBe(1);
  });

  it("says no turn for a png, which carries none", () => {
    expect(orientedOf([137, 80, 78, 71, 13, 10, 26, 10])).toBe(1);
  });

  it("says no turn for a jpeg with no such tag", () => {
    expect(orientedOf([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46])).toBe(1);
  });

  it("does not trip over a file that ends mid-tag", () => {
    expect(orientedOf(exif(6).slice(0, 12))).toBe(1);
    expect(orientedOf([0xff, 0xd8])).toBe(1);
    expect(orientedOf([])).toBe(1);
  });

  it("ignores a turn no camera would write", () => {
    expect(orientedOf(exif(0))).toBe(1);
    expect(orientedOf(exif(9))).toBe(1);
  });
});

describe("how large a photo goes onto the page", () => {
  it("shrinks a photo far larger than any sheet, which would bloat the file", () => {
    expect(scaled(4618, 3464)).toEqual([2400, 1800]);
    expect(scaled(3464, 4618)).toEqual([1800, 2400]);
  });

  it("leaves a photo the sheet can already hold alone", () => {
    expect(scaled(800, 600)).toEqual([800, 600]);
    expect(scaled(CAP, 100)).toEqual([CAP, 100]);
  });

  it("keeps a sliver of a photo from vanishing to nothing", () => {
    expect(scaled(9600, 3)).toEqual([2400, 1]);
  });

  it("copes with a photo of no size at all", () => {
    expect(scaled(0, 0)).toEqual([0, 0]);
  });
});

const jpeg = (wide: number, tall: number, head: number[] = []): number[] => [
  0xff,
  0xd8,
  ...head,
  0xff,
  0xc0,
  0x00,
  0x11,
  0x08,
  tall >> 8,
  tall & 0xff,
  wide >> 8,
  wide & 0xff,
  3,
];

describe("how big a photo is, read without decoding it", () => {
  it("reads width and height the way they are written, height first", () => {
    expect(sizedOf(jpeg(4618, 3464))).toEqual([4618, 3464]);
  });

  it("steps over what the camera wrote before the size", () => {
    expect(sizedOf(jpeg(800, 600, [0xff, 0xe0, 0x00, 0x06, 0x4a, 0x46, 0x49, 0x46]))).toEqual([
      800, 600,
    ]);
  });

  it("gives no size for a png, which is not read this way", () => {
    expect(sizedOf([137, 80, 78, 71, 13, 10, 26, 10])).toEqual([0, 0]);
  });

  it("does not trip over a file that ends mid-header", () => {
    expect(sizedOf(jpeg(800, 600).slice(0, 8))).toEqual([0, 0]);
  });
});

describe("which photos are worth redrawing before printing", () => {
  it("redraws one larger than any sheet needs", () => {
    expect(redrawn(jpeg(4618, 3464))).toBe(true);
  });

  it("redraws one the camera left on its side, however small", () => {
    expect(redrawn([...exif(6), ...jpeg(800, 600).slice(2)])).toBe(true);
  });

  it("leaves a photo that is already fit for the sheet alone", () => {
    expect(redrawn(jpeg(1600, 1200))).toBe(false);
  });

  it("never redraws a png, which would lose what shows through it", () => {
    expect(redrawn([137, 80, 78, 71, 13, 10, 26, 10])).toBe(false);
  });
});

describe("redrawing a photo so the page gets it the right way up", () => {
  const stand = (over: Partial<Record<string, unknown>> = {}) => {
    class Standing {
      onload: (() => void) | null = null;
      onerror: (() => void) | null = null;
      naturalWidth = 4000;
      naturalHeight = 3000;
      set src(_: string) {
        queueMicrotask(() => (over.fail ? this.onerror?.() : this.onload?.()));
      }
    }
    const was = { image: globalThis.Image, canvas: HTMLCanvasElement.prototype.getContext };
    globalThis.Image = Standing as unknown as typeof Image;
    HTMLCanvasElement.prototype.getContext = ((kind: string) =>
      over.blind || kind !== "2d" ? null : { drawImage: () => {} }) as never;
    HTMLCanvasElement.prototype.toDataURL = (() => "data:image/jpeg;base64,drawn") as never;
    return () => {
      globalThis.Image = was.image;
      HTMLCanvasElement.prototype.getContext = was.canvas;
    };
  };

  it("hands back a photo no larger than the cap, drawn again", async () => {
    const undo = stand();

    await expect(upright("data:image/jpeg;base64,huge")).resolves.toBe(
      "data:image/jpeg;base64,drawn",
    );
    undo();
  });

  it("keeps what it was given when the photo will not load", async () => {
    const undo = stand({ fail: true });

    await expect(upright("data:image/jpeg;base64,broken")).resolves.toBe(
      "data:image/jpeg;base64,broken",
    );
    undo();
  });

  it("keeps what it was given when there is nothing to draw on", async () => {
    const undo = stand({ blind: true });

    await expect(upright("data:image/jpeg;base64,nocanvas")).resolves.toBe(
      "data:image/jpeg;base64,nocanvas",
    );
    undo();
  });
});
