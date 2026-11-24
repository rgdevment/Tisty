import { CONTRACTS } from "@tisty/contracts";
import { afterEach, describe, expect, it } from "vitest";
import sheetPackage from "../package.json";
import { DESIGNS, designNamed, LOOKS, PLAIN, SIZE, TINT, wearDesign } from "../src/designs";
import { FONTS, HEADINGS, INKS, OWN, SHEETS } from "../src/ownDesign";

type Published = {
  light: Record<string, string>;
  dark: Record<string, string>;
  type: { text: string; headings: string; size: string; line_height: number; align: string };
  title: string;
  table: string;
};
const said = (CONTRACTS as { designs: { values: Record<string, unknown> } }).designs.values as {
  designs: Record<string, Published>;
  colors: { accent_soft_percent: Record<string, unknown> };
};
type OwnSaid = {
  sheets: object;
  headings: object;
  inks: object;
  defaults: object;
  soft: string;
  line: string;
  fonts: Record<string, { name: string; package: string; version: string }>;
};
const DEPENDS = sheetPackage.dependencies as Record<string, string>;
const root = document.documentElement;
const worn = (name: string) => root.style.getPropertyValue(name);

afterEach(() => root.removeAttribute("style"));

describe("the designs a sheet wears", () => {
  it("are the ones the contract publishes, value by value", () => {
    expect(Object.keys(said.designs)).toEqual([...DESIGNS]);
    for (const [name, look] of Object.entries(LOOKS)) {
      const one = said.designs[name];
      expect({ light: look.light, dark: look.dark, title: look.title }).toEqual({
        light: one.light,
        dark: one.dark,
        title: one.title,
      });
      expect(one.type).toEqual({
        text: "sans",
        headings: "sans",
        size: SIZE,
        line_height: 1.65,
        align: "left",
      });
      expect(one.table).toBe("tint");
    }
    expect(said.designs.tisty).toMatchObject(PLAIN);
    expect(said.colors.accent_soft_percent).toMatchObject(TINT);
  });

  it("reads a name nobody knows, or none, as Tisty", () => {
    expect([undefined, "", "pergamino", "constructor"].map(designNamed)).toEqual(
      Array(4).fill("tisty"),
    );
    expect(designNamed("fiordo")).toBe("fiordo");
  });

  it.each(Object.keys(LOOKS) as (keyof typeof LOOKS)[])("wears %s in light and in dark", (one) => {
    for (const dark of [false, true]) {
      wearDesign(root, one, dark);
      const colors = LOOKS[one][dark ? "dark" : "light"];
      expect(worn("--tisty-sheet")).toBe(colors.sheet);
      expect(worn("--tisty-heading")).toBe(colors.heading);
      expect(worn("--tisty-accent-soft")).toContain(dark ? "11%" : "8%");
    }
    expect(root.hasAttribute("data-designed")).toBe(true);
  });

  it("offers for a design of its own the names and colors the contract publishes", () => {
    const own = (said as unknown as { own: OwnSaid }).own;
    const plain = (list: object) =>
      Object.fromEntries(
        Object.entries(list).map(([key, { es, en, light, dark }]) => [
          key,
          { names: { es, en }, light, dark },
        ]),
      );
    expect(plain(SHEETS)).toEqual(own.sheets);
    expect(plain(HEADINGS)).toEqual(own.headings);
    expect(plain(INKS)).toEqual(own.inks);
    expect(OWN).toEqual(own.defaults);
    const { sans, ...files } = own.fonts;
    expect(sans).toBeDefined();
    expect(Object.keys(FONTS)).toEqual(Object.keys(files));
    for (const [key, { name, family, falls }] of Object.entries(FONTS)) {
      expect(files[key]).toMatchObject({ name, family, falls_to: falls });
      expect(DEPENDS[files[key].package]).toBe(files[key].version);
    }
    expect(own.soft).toBe("color-mix(in srgb, ink 78%, sheet)");
    expect(own.line).toBe("color-mix(in srgb, heading 28%, transparent)");
  });

  it("wears a design of its own, and reads each of its names alone", () => {
    const one = designNamed("own/arena/nadie/cafe/lora");
    expect(one).toBe("own/arena/grafito/cafe/lora");
    expect(designNamed("own")).toBe("own/papel/grafito/tinta/sans");
    expect(designNamed("own/constructor/valueof")).toBe("own/papel/grafito/tinta/sans");
    for (const none of ["own/#ff0000", "own/a/b/c/d/e", "ownx", "own/../x", "own/Arena"])
      expect(designNamed(none)).toBe("tisty");
    wearDesign(root, one, true);
    expect(worn("--tisty-sheet")).toBe(SHEETS.arena.dark);
    expect(worn("--tisty-accent")).toBe(HEADINGS.grafito.dark);
    expect(worn("--design-font")).toBe('"Lora Variable", serif');
    expect(worn("--own-font")).toContain("sans-serif");
  });

  it("keeps Tisty's own colors for what has them, in every design", () => {
    for (const one of Object.keys(LOOKS) as (keyof typeof LOOKS)[]) {
      wearDesign(root, one, false);
      expect(worn("--own-sheet")).toBe("#ffffff");
      wearDesign(root, one, true);
      expect(worn("--own-sheet")).toBe("#232326");
    }
  });

  it("leaves nothing of its own behind when it goes back to Tisty", () => {
    wearDesign(root, "coral", true);
    wearDesign(root, "tisty", true);
    expect(Array.from(root.style).filter((one) => /^--(design|own)-|heading/.test(one))).toEqual(
      [],
    );

    expect(root.hasAttribute("data-designed")).toBe(false);
  });
});
