import { beforeEach, describe, expect, it } from "vitest";
import { adopt, t } from "../locales";
import { found, landedOn, ROW_IDS, ROWS, settingAt } from "../settingsRows";

beforeEach(() => adopt("es"));

describe("searching a setting", () => {
  it("matches a title, a description or a keyword, accents or not, in the language spoken", () => {
    expect(found("idioma").map((hit) => hit.id)).toEqual(["tongue", "greet"]);
    expect(found("atajo").map((hit) => hit.id)).toEqual(["quick", "idea"]);
    expect(found("teclado").map((hit) => hit.id)).toEqual(["quick", "idea"]);
    expect(found("RESPALDO").map((hit) => hit.id)).toContain("backup");
    expect(found("sincronizacion").map((hit) => hit.id)).toContain("syncOff");
    adopt("en");
    expect(found("language").map((hit) => hit.id)).toEqual(["tongue", "greet"]);
    expect(found("idioma")).toEqual([]);
  });

  it("asks every word to turn up, in any order", () => {
    expect(found("rápida idea").map((hit) => hit.id)).toEqual(["idea"]);
    expect(found("idea zzz")).toEqual([]);
  });

  it("names each result and the page that holds it, in the order of the bar", () => {
    const hits = found("equipos");
    expect(hits.map((hit) => [hit.title, hit.page])).toContainEqual([
      "Tus equipos",
      "Sincronización",
    ]);
    const pages = hits.map((hit) => hit.row.page);
    expect(pages.indexOf("sync")).toBeLessThan(pages.indexOf("security"));
  });

  it("finds nothing in an empty box", () => {
    expect(found("   ")).toEqual([]);
  });

  it("suggests words that do find something, in both languages", () => {
    for (const tongue of ["es", "en"] as const) {
      adopt(tongue);
      const suggested = [...t("findNone").matchAll(/«([^»]+)»/g)].map((one) => one[1] ?? "");
      expect(suggested).toHaveLength(3);
      for (const word of suggested) expect(found(word).length).toBeGreaterThan(0);
    }
  });
});

describe("the rows", () => {
  it("each carry a title that exists in both languages", () => {
    for (const tongue of ["es", "en"] as const) {
      adopt(tongue);
      for (const id of ROW_IDS) expect(t(ROWS[id].title)).not.toBe("");
    }
  });

  it("mark where a landing finds them", () => {
    expect(settingAt("alias")).toEqual({ id: "setting-alias", "data-setting": "alias" });
  });

  it("land on a page, its tab and its row", () => {
    expect(landedOn({ page: "agents", shelf: "wired", row: "wired" })).toEqual({
      named: "keeping",
      tab: "agents",
      shelf: "wired",
      row: "wired",
      landedAt: expect.any(Number),
    });
  });
});
