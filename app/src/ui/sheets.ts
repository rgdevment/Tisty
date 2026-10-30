import { type Paper, PRINTED, sizeOf } from "tisty-editor/read";

const WORDS = {
  a4: "leafA4",
  letter: "leafLetter",
  folio: "leafFolio",
  tabloid: "leafTabloid",
} as const;

export const leafWord = (one: Paper) => WORDS[one];

export const pageFor = (leaf: Paper, landscape = false) => {
  const id = "tisty-page";
  let tag = document.getElementById(id) as HTMLStyleElement | null;
  if (!tag) {
    tag = document.createElement("style");
    tag.id = id;
    document.head.append(tag);
  }
  tag.textContent = `@page { size: ${sizeOf(leaf, landscape)}; ${PRINTED}; }`;
};
