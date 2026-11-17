import { type Paper, PRINTED, sizeOf } from "tisty-editor/read";

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
