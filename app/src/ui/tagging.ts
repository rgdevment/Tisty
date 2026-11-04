import { frontTags, unfronted } from "../front";

/// The same reading the core does when it saves: a hash pinned to a letter or a digit, with a
/// separator in front of it. `# Heading` carries a space, and a colour or a web address carries
/// something that is not a separator.
const AT = /#([\p{L}\p{N}](?:[\p{L}\p{N}\p{M}\-_]|[\u200C\u200D](?=[\p{L}\p{N}]))*)/gu;
const MARK = /\p{M}/u;
const TIES = /[\p{L}\p{N}#/\-_.:]/u;

// A mark counts as what it sits on, so an emoji before the hash ties nothing and a letter does.
const tied = (before: string): boolean => {
  const chars = [...before];
  let at = chars.length - 1;
  while (at >= 0 && MARK.test(chars[at])) at -= 1;
  return at >= 0 && TIES.test(chars[at]);
};

// Where a mark is an accent or a vowel a reader can do without, over a letter that stands without it.
const ACCENTS = /([\u0021-\u06FF\u0750-\u077F\u08A0-\u08FF\u1E00-\u1FFF])\p{M}+/gu;
// It only asks for another drawing of what comes before it, so two tags would look the same and be two.
const DRAWINGS = /[\u180B-\u180D\uFE00-\uFE0F\u{E0100}-\u{E01EF}]/gu;

export const worthReading = (tag: string): boolean => tag.length > 1 && /\p{L}/u.test(tag);

/// The same shape the core gives a tag — accents off where a letter stands without them, lowercase, underscores and spaces as dashes,
/// nothing else kept, and never two dashes in a row. A word that lands anywhere else opens a
/// screen with nothing on it.
export const named = (said: string): string =>
  said
    .toLowerCase()
    .normalize("NFD")
    .replace(DRAWINGS, "")
    .replace(/[\p{White_Space}_]/gu, "-")
    .replace(/[^\p{L}\p{N}\p{M}-]\p{M}*/gu, "")
    .replace(ACCENTS, "$1")
    .replace(/^\p{M}+/u, "")
    .replace(/-{2,}/g, "-")
    .replace(/^-|-$/g, "")
    .normalize("NFC");

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
      if (tied(said.slice(0, hit.index))) continue;
      const tag = named(hit[1]);
      if (worthReading(tag)) many.set(tag, (many.get(tag) ?? 0) + 1);
    }
  }
  for (const word of frontTags(body)) {
    const tag = named(word);
    if (tag) many.set(tag, (many.get(tag) ?? 0) + 1);
  }
  return many;
};
