import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { chooseTongue, preferred, shown, tongueName, tonguesOf, twinOf } from "../tongues";

vi.mock("../locales", () => ({ locale: () => "es" }));

const doc = (id: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title: id,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

const es = doc("es", { lang: "es", tongue: ["es", 0] });
const en = doc("en", { lang: "en", tongue: ["es", 1] });
const alone = doc("alone");

describe("languages of a document", () => {
  beforeEach(() => localStorage.clear());

  it("shows one row for the languages that stand together, in the app's language", () => {
    expect(shown([en, es, alone]).map((one) => one.id)).toEqual(["es", "alone"]);
  });

  it("shows the one open now, then the reader's choice", () => {
    expect(shown([es, en], "f-en").map((one) => one.id)).toEqual(["en"]);
    chooseTongue("es", "en");
    expect(preferred([es, en])?.id).toBe("en");
  });

  it("keeps apart a language that stands in another folder", () => {
    const moved = { ...en, folder: "elsewhere" };
    expect(shown([es, moved]).map((one) => one.id)).toEqual(["es", "en"]);
  });

  it("lists a group's languages in the order they joined, and none for one alone", () => {
    expect(tonguesOf([en, alone, es], en).map((one) => one.id)).toEqual(["es", "en"]);
    expect(tonguesOf([es, en, alone], alone)).toEqual([]);
  });

  it("finds the page that stands in the same place, or the document when there is none", () => {
    const pages = [
      doc("es1", { pageOf: "es" }),
      doc("es2", { pageOf: "es" }),
      doc("en1", { pageOf: "en" }),
    ];
    const all = [es, en, ...pages];
    expect(twinOf(all, pages[0], en).id).toBe("en1");
    expect(twinOf(all, pages[1], en).id).toBe("en");
    expect(twinOf(all, es, en).id).toBe("en");
  });

  it("names a language in its own words", () => {
    expect(tongueName("es")).toBe("Español");
    expect(tongueName("en")).toBe("English");
  });
});
