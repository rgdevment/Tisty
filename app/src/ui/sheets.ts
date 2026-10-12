import type { Paper } from "../core";

export const LEAVES: Paper[] = ["a4", "letter", "tabloid"];

export const PAPER: Record<Paper, number> = { a4: 820, letter: 843, tabloid: 1090 };

export const PAGE: Record<Paper, string> = { a4: "A4", letter: "Letter", tabloid: "11in 17in" };

export const TALL: Record<Paper, number> = { a4: 297 / 210, letter: 279 / 216, tabloid: 432 / 279 };

export const leafWord = (one: Paper) =>
  one === "a4" ? "leafA4" : one === "letter" ? "leafLetter" : "leafTabloid";

export const pageFor = (leaf: Paper) => {
  const id = "tisty-page";
  let tag = document.getElementById(id) as HTMLStyleElement | null;
  if (!tag) {
    tag = document.createElement("style");
    tag.id = id;
    document.head.append(tag);
  }
  tag.textContent = `@page { size: ${PAGE[leaf]}; margin: 22mm 20mm; }`;
};
