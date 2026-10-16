const breaks = (one: string | undefined) => one === "\n" || one === "\r";

const headOf = (text: string) => {
  let at = text.length;
  while (at > 0 && breaks(text[at - 1])) at -= 1;
  return text.slice(0, at);
};

const restOf = (text: string) => {
  let at = 0;
  while (at < text.length && breaks(text[at])) at += 1;
  return text.slice(at);
};

const newlines = (text: string) => text.split("\n").length - 1;

export const parting = (side: string, after: boolean, line = "\n"): string => {
  if (!side.trim()) return "";
  const edge = after
    ? side.slice(0, side.length - restOf(side).length)
    : side.slice(headOf(side).length);
  return line.repeat(Math.max(0, 2 - newlines(edge)));
};

// A block written empty goes with the blank line that kept it apart, so no gap is left behind.
export const closed = (text: string, from: number, end: number): string => {
  const line = text.includes("\r\n") ? "\r\n" : "\n";
  const head = headOf(text.slice(0, from));
  const rest = restOf(text.slice(end));
  if (!head) return rest;
  if (!rest) return `${head}${line}`;
  return `${head}${line}${line}${rest}`;
};

export const laid = (text: string, at: number, block: string): string => {
  const line = text.includes("\r\n") ? "\r\n" : "\n";
  const head = headOf(text.slice(0, at));
  const rest = restOf(text.slice(at));
  const whole = [head, block, rest].filter(Boolean).join(`${line}${line}`);
  return rest ? whole : `${whole}${line}`;
};
