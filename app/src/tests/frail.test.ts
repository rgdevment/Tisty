import { describe, expect, it } from "vitest";
import { frail } from "../frail";

describe("what a document brings that the editor cannot keep", () => {
  it("says nothing about a document made of what the editor writes", () => {
    const kept = [
      "# Título",
      "",
      "Un párrafo con **negrita**, *cursiva* y `código`.",
      "",
      "- una lista",
      "- con dos cosas",
      "",
      "| a | b |",
      "| --- | --- |",
      "| uno | dos |",
      "",
      "> una cita",
      "",
      "[un enlace](https://ejemplo.org) y ![una imagen](<attachments/ab/x.png>)",
    ].join("\n");

    expect(frail(kept)).toEqual([]);
  });

  it("sees the front matter, which grows a backslash on every save", () => {
    expect(frail("---\ntitle: algo\n---\n\n# Hola")).toContain("frailFront");
  });

  it("sees a block of html, which loses its tag and keeps only the words", () => {
    expect(frail("Hola\n\n<details>\n<summary>ver</summary>\nel detalle\n</details>")).toContain(
      "frailHtml",
    );
    expect(frail('<div class="warn">Cuidado</div>')).toContain("frailHtml");
  });

  it("sees a footnote, which comes back with its brackets escaped", () => {
    expect(frail("una nota[^1]\n\n[^1]: el pie")).toContain("frailNotes");
  });

  it("sees a link by reference, whose definition is thrown away", () => {
    expect(frail("mira [esto][uno]\n\n[uno]: https://ejemplo.org")).toContain("frailRefs");
  });

  it("sees the alignment of a table, which comes back centred on nothing", () => {
    expect(frail("| tarea | horas |\n| :--- | ---: |\n| una | 3 |")).toContain("frailAligned");
  });

  it("leaves alone what a person wrote inside a code fence", () => {
    const fenced = ["Mira esto:", "", "```html", '<div class="warn">x</div>', "```"].join("\n");

    expect(frail(fenced)).toEqual([]);
  });

  it("leaves alone html written as indented code", () => {
    expect(frail("Ejemplo:\n\n    <div>x</div>\n")).toEqual([]);
  });

  it("names every kind it finds, not only the first", () => {
    const messy = "---\na: b\n---\n\n<div>x</div>\n\nnota[^1]\n\n[^1]: pie";

    expect(frail(messy)).toEqual(["frailFront", "frailHtml", "frailNotes", "frailRefs"]);
  });
});
