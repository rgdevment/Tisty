// A hyphen is plain outside a class; escaping it would break a `u` pattern.
const escaped = (said: string) => said.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

// A name stands alone when nothing that could be part of a name touches it.
const nameIn = (name: string) =>
  new RegExp(`(?<![\\p{L}\\p{N}_*-])${escaped(name)}(?![\\p{L}\\p{N}_*-])`, "gu");

/** Whether a line names something as a word of its own, outside quotes. */
export const names = (line: string, name: string): boolean =>
  line.split(/("[^"]*")/).some((part) => !part.startsWith('"') && nameIn(name).test(part));

// Comments, titles, descriptions and words in quotes are the person's own and never change.
export const renamedLine = (line: string, from: string, to: string): string => {
  if (/^\s*(%%|accTitle|accDescr)/.test(line)) return line;
  return line
    .split(/("[^"]*")/)
    .map((part) => (part.startsWith('"') ? part : part.replace(nameIn(from), to)))
    .join("");
};
