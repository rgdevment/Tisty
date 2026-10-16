const edges = (side: string, after: boolean) =>
  ((after ? /^(?:\r?\n)*/ : /(?:\r?\n)*$/).exec(side)?.[0].match(/\n/g) ?? []).length;

export const parting = (side: string, after: boolean, line = "\n"): string => {
  if (!side.trim()) return "";
  return line.repeat(Math.max(0, 2 - edges(side, after)));
};

// A block written empty goes with the blank line that kept it apart, so no gap is left behind.
export const closed = (text: string, from: number, end: number): string => {
  const line = text.includes("\r\n") ? "\r\n" : "\n";
  const head = text.slice(0, from).replace(/(?:\r?\n)+$/, "");
  const rest = text.slice(end).replace(/^(?:\r?\n)+/, "");
  if (!head) return rest;
  if (!rest) return `${head}${line}`;
  return `${head}${line}${line}${rest}`;
};

// A block laid at a point of the text, kept apart from what is before and after it by blank lines.
export const laid = (text: string, at: number, block: string): string => {
  const line = text.includes("\r\n") ? "\r\n" : "\n";
  const head = text.slice(0, at).replace(/(?:\r?\n)+$/, "");
  const rest = text.slice(at).replace(/^(?:\r?\n)+/, "");
  const whole = [head, block, rest].filter(Boolean).join(`${line}${line}`);
  return rest ? whole : `${whole}${line}`;
};
