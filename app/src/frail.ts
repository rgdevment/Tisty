const BLOCKS =
  /<\/?(div|details|summary|section|article|aside|figure|figcaption|table|iframe|video|audio|form|blockquote)\b/i;

const fenceless = (text: string): string => {
  const out: string[] = [];
  let shut: string | null = null;
  let held: string[] = [];
  for (const line of text.split("\n")) {
    const said = line.trim().replace(/^(>\s*)+/, "");
    const opens = /^(```+|~~~+)/.exec(said);
    if (shut === null) {
      if (opens) {
        shut = opens[1];
        held = [];
        continue;
      }
      out.push(line);
      continue;
    }
    if (opens && opens[1][0] === shut[0] && opens[1].length >= shut.length) {
      shut = null;
      held = [];
      continue;
    }
    held.push(line);
  }
  return out.concat(shut === null ? [] : held).join("\n");
};

export const frail = (text: string): string[] => {
  const found: string[] = [];
  const bare = fenceless(text)
    .replace(/^ {4}.*$/gm, "")
    .replace(/^\t.*$/gm, "");
  const spanless = bare.replace(/`[^`\n]*`/g, "");

  if (/^﻿?---\r?\n[\s\S]*?\r?\n---\r?\n/.test(text)) found.push("frailFront");
  if (BLOCKS.test(spanless)) found.push("frailHtml");
  if (/(^|[^\\])\[\^[^\]]+\](?![([])/m.test(spanless)) found.push("frailNotes");
  if (/^\[[^\\\]]+\]:\s*\S/m.test(spanless)) found.push("frailRefs");

  return found;
};
