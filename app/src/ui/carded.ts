import { DOC } from "../markdown";
import { type Moved, unfenced } from "../paging";

const escapedAlt = (title: string): string => title.replace(/[\\[\]]/g, (one) => `\\${one}`);

const literal = (said: string): string => said.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

export const cardOf = (file: string, title: string): string =>
  `![${escapedAlt(title)}](${DOC}${file})`;

type Lines = { lines: string[]; seen: string[] };

const read = (body: string): Lines => ({
  lines: body.split("\n"),
  seen: unfenced(body).split("\n"),
});

const blank = (line: string | undefined): boolean => line === undefined || !line.trim();

const cardIn = (line: string, file: string): boolean =>
  new RegExp(
    `!\\[(?:\\\\[\\s\\S]|[^\\\\\\[\\]\\n])*\\]\\(\\s*<?${literal(DOC + file)}[\\s)>]`,
  ).test(line);

const firstSaid = (said: Lines): number => said.lines.findIndex((line) => !blank(line));

// A card stands on its own only as a whole paragraph at the top of the document.
const standing = (said: Lines, file: string): number => {
  const at = said.seen.findIndex((line) => cardIn(line, file));
  if (at < 0) return -1;
  const line = said.lines[at];
  const alone = /^!\[/.test(line) && /\)\s*$/.test(line);
  return alone && blank(said.lines[at - 1]) && blank(said.lines[at + 1]) ? at : -2;
};

export const putCard = (body: string, file: string, title: string): string | null => {
  // A document takes its title from the first thing it says, so a card put into one that says
  // nothing yet would become its name. The core refuses the same write for the same reason.
  if (!body.trim()) return null;
  return `${body.replace(/\n+$/, "")}\n\n${cardOf(file, title)}\n`;
};

export const movedCard = (
  body: string,
  file: string,
  before: string | null,
): { moved: Moved; body: string } => {
  const said = read(body);
  const from = standing(said, file);
  const to = before === null ? said.lines.length : standing(said, before);
  if (from === -2 || to === -2) return { moved: "held", body };
  if (from === -1 || to === -1) return { moved: "unseen", body };
  if (to === from || to === from + 2) return { moved: "done", body };
  const first = firstSaid(said);
  if (from === first || to === first) return { moved: "titled", body };
  const line = said.lines[from];
  const kept = said.lines.filter((_, at) => at !== from && at !== from + 1);
  const goes = to > from ? to - 2 : to;
  if (before === null) {
    while (kept.length && blank(kept[kept.length - 1])) kept.pop();
    return { moved: "done", body: `${[...kept, "", line].join("\n")}\n` };
  }
  kept.splice(goes, 0, line, "");
  return { moved: "done", body: kept.join("\n") };
};

// A card is a card only alone in its paragraph, so a blank line is kept on either side of it.
export const cardedAt = (body: string, from: number, end: number, card: string): string => {
  const line = body.includes("\r\n") ? "\r\n" : "\n";
  const head = body.slice(0, from);
  const tail = body.slice(end);
  const before = !head || /(\r?\n){2}$/.test(head) ? "" : line;
  const last = /^(\r?\n)?$/.test(tail);
  const after = last ? (tail ? "" : line) : /^(\r?\n){2}/.test(tail) ? "" : line;
  return `${head}${before}${card}${after}${tail}`;
};
