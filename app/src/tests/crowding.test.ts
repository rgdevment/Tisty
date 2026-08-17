import { describe, expect, it } from "vitest";
import { crowd, MANY } from "../previews";

describe("counting what a document will have to draw", () => {
  it("counts nothing in words alone", () => {
    expect(crowd("# Titulo\n\nun parrafo cualquiera")).toBe(0);
  });

  it("counts an attachment however its address is wrapped", () => {
    expect(crowd("[charla](<attachments/ab/charla.mp4>)")).toBe(1);
    expect(crowd("[charla](attachments/ab/charla.mp4)")).toBe(1);
  });

  it("counts images, which are drawn just the same", () => {
    expect(crowd("![una foto](<attachments/ab/foto.png>)")).toBe(1);
  });

  it("counts a document card, which also costs a decoration", () => {
    expect(crowd("[la minuta](tisty:doc/mac0-0001)")).toBe(1);
  });

  it("sees a document reference the way the editor will, brackets and all", () => {
    expect(crowd("[la minuta](<tisty:doc/mac0-0001>)")).toBe(1);
  });

  it("leaves out a plain address, which draws nothing", () => {
    expect(crowd("mira [la web](https://ejemplo.org) y [otra](http://x.dev)")).toBe(0);
  });

  it("counts every one of them, not only the first", () => {
    const many = Array.from(
      { length: 7 },
      (_, i) => `- [uno ${i}](<attachments/ab/uno${i}.pdf>)`,
    ).join("\n");

    expect(crowd(many)).toBe(7);
  });

  it("does not choke on a bracket left open", () => {
    expect(crowd("[a medio escribir](<attachments/ab/x.pdf")).toBe(0);
  });

  it("warns before the count is where it hurts, not after", () => {
    expect(MANY).toBeLessThan(534);
    expect(MANY).toBeGreaterThan(20);
  });
});
