import { afterEach, describe, expect, it, vi } from "vitest";
import { fontCarried, fontLaid, fontWorn } from "../src/fonts";
import { FONTS } from "../src/ownDesign";

class Face {
  constructor(
    readonly family: string,
    readonly source: string,
    readonly said: { unicodeRange: string },
  ) {}
}

const paged = (load = () => Promise.resolve([] as unknown[])) => {
  const held: Face[] = [];
  const fonts = {
    add: (one: Face) => held.push(one),
    load: vi.fn(load),
    [Symbol.iterator]: () => held[Symbol.iterator](),
  };
  const page = { defaultView: { FontFace: Face }, fonts } as unknown as Document;
  return { page, held, fonts };
};

afterEach(() => vi.unstubAllGlobals());

describe("the font a design of its own wears", () => {
  it("is asked for once in a page, its Latin and its extended files", async () => {
    const { page, held, fonts } = paged();
    await fontLaid(page, "lora");
    await fontLaid(page, "lora");
    expect(held.map((one) => one.family)).toEqual(Array(2).fill("Lora Variable"));
    expect(held[0].said.unicodeRange).not.toBe(held[1].said.unicodeRange);
    expect(fonts.load).toHaveBeenCalledTimes(2);
  });

  it("is none for the system's, for a factory design or for a name nobody knows", async () => {
    const { page, held } = paged();
    await fontWorn(page, "coral");
    await fontWorn(page, "own/arena/vino/cafe/sans");
    await fontLaid(page, "papyrus");
    expect(held).toEqual([]);
  });

  it("lets the page be when its file does not come", async () => {
    const { page } = paged(() => Promise.reject(new Error("gone")));
    await expect(fontWorn(page, "own/arena/vino/cafe/caveat")).resolves.toBeUndefined();
  });

  it("goes inside a page that leaves, and a page leaves without it all the same", async () => {
    const answer = { arrayBuffer: () => Promise.resolve(new Uint8Array([1, 2, 3]).buffer) };
    vi.stubGlobal("fetch", () => Promise.resolve(answer));
    for (const font of Object.keys(FONTS)) {
      const said = await fontCarried(`own/arena/vino/cafe/${font}`);
      expect(said.match(/@font-face/g)).toHaveLength(2);
      expect(said).toContain(`font-family:"${FONTS[font].family}"`);
      expect(said).toContain("base64,AQID");
    }
    expect(await fontCarried("coral")).toBe("");
    vi.stubGlobal("fetch", () => Promise.reject(new Error("gone")));
    expect(await fontCarried("own/arena/vino/cafe/lora")).toBe("");
  });
});
