import { designNamed, fontWorn, wearDesign } from "tisty-sheet";
import { isDark } from "./sheet";

// Only the sheet wears it, so the window around it keeps its own colors, and it turns as the theme does.
export const worn = (page: HTMLElement, said: string | null | undefined) => {
  const design = designNamed(said ?? undefined);
  const wear = () => wearDesign(page, design, isDark());
  wear();
  void fontWorn(page.ownerDocument, design);
  new MutationObserver(wear).observe(document.documentElement, { attributeFilter: ["class"] });
};
