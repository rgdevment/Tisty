import { DOC } from "../markdown";
import type { Moved } from "../paging";

const escapedAlt = (title: string): string => title.replace(/[\\[\]]/g, (one) => `\\${one}`);

export const cardOf = (file: string, title: string): string =>
  `![${escapedAlt(title)}](${DOC}${file})`;

type Lines = { lines: string[]; fenced: boolean[] };

const read = (body: string): Lines => {
  const lines = body.split("\n");
  const fenced: boolean[] = [];
  let wall: string | null = null;
  for (const line of lines) {
    const fence = /^ {0,3}(`{3,}|~{3,})/.exec(line)?.[1];
    if (wall === null && fence) {
      wall = fence;
      fenced.push(true);
      continue;
    }
    fenced.push(wall !== null);
    if (wall !== null && fence && fence[0] === wall[0] && fence.length >= wall.length) wall = null;
  }
  return { lines, fenced };
};

const blank = (line: string | undefined): boolean => line === undefined || !line.trim();

const cardIn = (line: string, file: string): boolean =>
  new RegExp(`!\\[(?:\\\\[\\s\\S]|[^\\\\\\[\\]\\n])*\\]\\(\\s*<?${DOC}${file}[\\s)>]`).test(line);

const firstSaid = (said: Lines): number =>
  said.lines.findIndex((line, at) => !said.fenced[at] && !blank(line));

// A card stands on its own only as a whole paragraph at the top of the document.
const standing = (said: Lines, file: string): number => {
  const at = said.lines.findIndex((line, i) => !said.fenced[i] && cardIn(line, file));
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
