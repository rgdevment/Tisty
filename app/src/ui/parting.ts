import { heldTogether, LEADS } from "tisty-editor/edit";

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

export const joined = (text: string, at: number): { text: string; at: number } => {
  const head = headOf(text.slice(0, at));
  const rest = restOf(text.slice(at)).replace(LEADS, "");
  const start = head.lastIndexOf("\n") + 1;
  const lineEnd = rest.search(/\r?\n/);
  const first = lineEnd < 0 ? rest : rest.slice(0, lineEnd);
  const held = heldTogether(head.slice(start), first);
  if (!held) return { text: `${head}${rest}`, at: head.length };
  const before = `${head.slice(0, start)}${held.left}`;
  return { text: `${before}${held.right}${rest.slice(first.length)}`, at: before.length };
};

export const laid = (text: string, at: number, block: string): string => {
  const line = text.includes("\r\n") ? "\r\n" : "\n";
  const head = headOf(text.slice(0, at));
  const rest = restOf(text.slice(at));
  const whole = [head, block, rest].filter(Boolean).join(`${line}${line}`);
  return rest ? whole : `${whole}${line}`;
};
