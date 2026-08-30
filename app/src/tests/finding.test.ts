import { describe, expect, it } from "vitest";
import { begun, folded, matched, terms } from "../finding";

describe("finding", () => {
  it("cuts a search into words without their accents", () => {
    expect(terms("  Análisis   del  Repositorio ")).toEqual(["analisis", "del", "repositorio"]);
    expect(terms("MERKÉN")).toEqual(["merken"]);
  });

  it("has nothing to look for when nothing was typed", () => {
    expect(terms("")).toEqual([]);
    expect(terms("   ")).toEqual([]);
    expect(terms('""')).toEqual([]);
  });

  it("holds a phrase together between quotes", () => {
    expect(terms('"casa de campo" verde')).toEqual(["casa de campo", "verde"]);
    expect(terms("“casa de campo”")).toEqual(["casa de campo"]);
  });

  it("cannot grow long enough to scan the store a hundred times", () => {
    const many = terms(Array.from({ length: 40 }, (_, n) => `w${n}`).join(" "));

    expect(many).toHaveLength(12);
  });

  it("finds the words in any order and without the accent", () => {
    const title = "Multilogin B2B Falabella — Análisis del repositorio";

    expect(matched(title, "analisis")).toBe(true);
    expect(matched(title, "ANÁLISIS")).toBe(true);
    expect(matched(title, "multilogin falabella")).toBe(true);
    expect(matched(title, "repositorio multilogin")).toBe(true);
    expect(matched(title, "multilogin dentista")).toBe(false);
  });

  it("keeps the order of a phrase in quotes", () => {
    expect(matched("Análisis del repositorio", '"del repositorio"')).toBe(true);
    expect(matched("Análisis del repositorio", '"repositorio del"')).toBe(false);
  });

  it("shows everything while nothing is typed", () => {
    expect(matched("cualquier cosa", "")).toBe(true);
  });

  it("matches the start of a word whether or not the accent was typed", () => {
    expect(begun("Título", "titu")).toBe(true);
    expect(begun("Título", "TÍT")).toBe(true);
    expect(begun("Título", "tulo")).toBe(false);
    expect(folded("ÁÉÍÓÚñ")).toBe("aeioun");
  });
});
