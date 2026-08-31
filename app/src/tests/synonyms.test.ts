import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { alsoNamed, everyNamed } from "../synonyms";
import { sifted } from "../ui/Icons";

const named = (): string[] => {
  const at = resolve(process.cwd(), "../crates/tisty-core/src/model/icon.rs");
  const said = readFileSync(at, "utf8");
  const body = said.slice(said.indexOf("ICONS"), said.indexOf("];"));
  return [...body.matchAll(/"([a-z-]+)"/g)].map((found) => found[1]);
};

const all = named();

describe("looking for an icon by what it draws", () => {
  it("points only at icons the core actually offers", () => {
    expect(everyNamed().filter((key) => !all.includes(key))).toEqual([]);
  });

  it("finds the faces nobody would look for under mood", () => {
    expect(sifted(all, "cara")).toContain("mood-happy");
    expect(sifted(all, "smile")).toContain("mood");
    expect(sifted(all, "triste")).toContain("mood-sad");
  });

  it("finds what is named for its use, by its shape", () => {
    expect(sifted(all, "avión")).toContain("travel");
    expect(sifted(all, "apretón")).toContain("deal");
    expect(sifted(all, "cohete")).toContain("sprint");
    expect(sifted(all, "moto")).toContain("motorbike");
    expect(sifted(all, "puño")).toContain("fist");
  });

  it("reads a word typed without its accent", () => {
    expect(sifted(all, "avion")).toEqual(sifted(all, "avión"));
    expect(sifted(all, "PUNO")).toContain("fist");
  });

  it("still finds by the name itself, and keeps the catalogue's order", () => {
    const shown = sifted(all, "car");
    expect(shown).toContain("car");
    expect(shown).toEqual(all.filter((one) => shown.includes(one)));
  });

  it("says nothing to a word it does not know", () => {
    expect(alsoNamed("qwerty")).toEqual([]);
    expect(alsoNamed("x")).toEqual([]);
  });
});
