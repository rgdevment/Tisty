import { describe, expect, it } from "vitest";
import { source } from "./source";

const selectorLists = (css: string) =>
  [...css.replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/([^{}]+)\{[^{}]*\}/g)].map(([, list]) =>
    list
      .split(/,(?![^(]*\))/)
      .map((one) => one.trim())
      .filter(Boolean),
  );

describe("what only the editor wears", () => {
  it("stays out of the page's sheet, which the light viewer loads, but for the icon its hues follow", () => {
    const editorOnly = selectorLists(source("page.css"))
      .filter((list) => list.every((one) => one.includes(".cm-")))
      .map((list) => list.join(", "));

    expect(editorOnly).toEqual([".cm-ico"]);
  });
});
