import { unfronted } from "../front";

/// The same reading the core does when it saves: a hash pinned to a letter or a digit, with a
/// separator in front of it. `# Heading` carries a space, and a colour or a web address carries
/// something that is not a separator.
const AT = /(^|[^\p{L}\p{N}#/\-_.:])#([\p{L}\p{N}][\p{L}\p{N}\-_]*)/gu;

export const worthReading = (tag: string): boolean => tag.length > 1 && /\p{L}/u.test(tag);

/// The same shape the core gives a tag — accents off, lowercase, underscores and spaces as dashes,
/// nothing else kept, and never two dashes in a row. A word that lands anywhere else opens a
/// screen with nothing on it.
export const named = (said: string): string =>
  said
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[\s_]/g, "-")
    .replace(/[^\p{L}\p{N}-]/gu, "")
    .replace(/-{2,}/g, "-")
    .replace(/^-+|-+$/g, "");

export const often = (body: string): Map<string, number> => {
  const many = new Map<string, number>();
  let fence: string | null = null;
  for (const line of unfronted(body).split(/\r?\n/)) {
    const rail = /^ {0,3}(`{3,}|~{3,})/.exec(line);
    if (rail) {
      if (!fence) fence = rail[1][0];
      else if (rail[1][0] === fence) fence = null;
      continue;
    }
    if (fence) continue;
    const said = line.replace(/`[^`]*`/g, (one) => " ".repeat(one.length));
    AT.lastIndex = 0;
    for (let hit = AT.exec(said); hit; hit = AT.exec(said)) {
      const tag = named(hit[2]);
      if (worthReading(tag)) many.set(tag, (many.get(tag) ?? 0) + 1);
    }
  }
  return many;
};
