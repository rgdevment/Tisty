/** One mark around a cell's words: its opening wall and its closing one, a pen's colour included. */
export type Mark = { readonly open: string; readonly close: string };

/** A cell's words, the marks wrapping them from the outside in, and how to wrap new words in marks. */
export type Peeled = {
  readonly words: string;
  readonly marks: readonly Mark[];
  readonly wrap: (words: string, marks?: readonly Mark[]) => string;
};

// Longest first, so `**` is never read as two italics.
const WALLS = ["**", "__", "++", "~~", "*", "_", "`"];
const PEN = /^==(\S(?:.*\S)?)==(\{[a-z]+\})?$/;

// One mark around the whole cell, its words holding none of its own walls: a pen, bold, italic, underline, struck or code.
const around = (said: string): { inner: string; mark: Mark } | null => {
  const pen = PEN.exec(said);
  if (pen && !pen[1].includes("=="))
    return { inner: pen[1], mark: { open: "==", close: `==${pen[2] ?? ""}` } };
  for (const wall of WALLS) {
    if (said.length <= wall.length * 2 || !said.startsWith(wall) || !said.endsWith(wall)) continue;
    const inner = said.slice(wall.length, -wall.length);
    if (inner.includes(wall) || inner !== inner.trim()) return null;
    return { inner, mark: { open: wall, close: wall } };
  }
  return null;
};

const marksOf = (said: string): { words: string; marks: Mark[] } => {
  const one = around(said.trim());
  if (!one) return { words: said, marks: [] };
  const deeper = marksOf(one.inner);
  return { words: deeper.words, marks: [one.mark, ...deeper.marks] };
};

const wrapped = (words: string, marks: readonly Mark[]): string => {
  const kept = words.trim();
  // Emptied, the cell has nothing left to wrap.
  if (!kept) return "";
  // A mark whose walls the words hold would tear, so it is left off rather than broken.
  return marks.reduceRight((inner, one) => {
    const next = `${one.open}${inner}${one.close}`;
    return marksOf(next).words === kept ? next : inner;
  }, kept);
};

/** The words of a cell wholly wrapped in marks, a pen in bold, say; a cell marked in parts keeps its Markdown. */
export const peeled = (said: string): Peeled => {
  const { words, marks } = marksOf(said);
  return { words, marks, wrap: (typed, chosen = marks) => wrapped(typed, chosen) };
};

// Italic is one mark however it is written, and so is bold.
const kindOf = (open: string) => (open === "_" ? "*" : open === "__" ? "**" : open);

/** The marks with one put on or taken off; a pen of another colour turns the pen that colour. */
export const toggled = (marks: readonly Mark[], open: string, close: string): Mark[] => {
  const at = marks.findIndex((one) => kindOf(one.open) === kindOf(open));
  if (at < 0) return [...marks, { open, close }];
  const was = marks[at];
  if (open === "==" && was.close !== close)
    return marks.map((one, i) => (i === at ? { open, close } : one));
  return marks.filter((_, i) => i !== at);
};

/** Whether a mark of this kind and, for a pen, this colour wraps the cell. */
export const holds = (marks: readonly Mark[], open: string, close: string): boolean =>
  marks.some(
    (one) =>
      kindOf(one.open) === kindOf(open) &&
      (open !== "==" || one.close === close || (one.close === "=={yellow}" && close === "==")),
  );
