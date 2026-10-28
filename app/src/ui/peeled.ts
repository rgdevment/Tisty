/** A cell's words and how to wrap new words in the marks that wrapped the old ones. */
export type Peeled = { readonly words: string; readonly wrap: (words: string) => string };

// Longest first, so `**` is never read as two italics.
const WALLS = ["**", "__", "++", "~~", "*", "_", "`"];
const PEN = /^==(\S(?:.*\S)?)==(\{[a-z]+\})?$/;

// One mark around the whole cell, its words holding none of its own walls: a pen, bold, italic, underline, struck or code.
const around = (said: string): { inner: string; wrap: (words: string) => string } | null => {
  const pen = PEN.exec(said);
  if (pen && !pen[1].includes("==")) {
    const hue = pen[2] ?? "";
    return { inner: pen[1], wrap: (words) => `==${words}==${hue}` };
  }
  for (const wall of WALLS) {
    if (said.length <= wall.length * 2 || !said.startsWith(wall) || !said.endsWith(wall)) continue;
    const inner = said.slice(wall.length, -wall.length);
    if (inner.includes(wall[0]) || inner !== inner.trim()) return null;
    return { inner, wrap: (words) => `${wall}${words}${wall}` };
  }
  return null;
};

/** The words of a cell wholly wrapped in marks, a pen in bold, say; a cell marked in parts keeps its Markdown. */
export const peeled = (said: string): Peeled => {
  const kept = said.trim();
  const one = around(kept);
  if (!one) return { words: said, wrap: (words) => words };
  const deeper = peeled(one.inner);
  return {
    words: deeper.words,
    wrap: (words) => {
      const kept = words.trim();
      // Emptied, the cell has nothing left to wrap.
      if (!kept) return "";
      const made = one.wrap(deeper.wrap(kept));
      // Words holding the mark's own walls would tear it, so they go in plain rather than broken.
      return peeled(made).words === kept ? made : kept;
    },
  };
};
