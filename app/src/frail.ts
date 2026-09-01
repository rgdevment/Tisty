const BLOCKS =
  /<\/?(div|details|summary|section|article|aside|figure|figcaption|table|iframe|video|audio|form|blockquote)\b/i;

/// The same fences `docs::survives` skips: either marker, indented or not, and closed by its own.
const fenceless = (text: string): string => {
  const out: string[] = [];
  let shut: string | null = null;
  for (const line of text.split("\n")) {
    const said = line.trim().replace(/^(>\s*)+/, "");
    const opens = /^(```+|~~~+)/.exec(said);
    if (shut === null) {
      if (opens) {
        shut = opens[1][0];
        continue;
      }
      out.push(line);
      continue;
    }
    if (opens && opens[1][0] === shut) shut = null;
  }
  return out.join("\n");
};

export const frail = (text: string): string[] => {
  const found: string[] = [];
  const bare = fenceless(text).replace(/^ {4}.*$/gm, "");

  if (/^---\r?\n[\s\S]*?\r?\n---\r?\n/.test(text)) found.push("frailFront");
  if (BLOCKS.test(bare)) found.push("frailHtml");
  if (/(^|[^\\])\[\^[^\]]+\](?![([])/m.test(bare)) found.push("frailNotes");
  if (/^\[[^\\\]]+\]:\s*\S/m.test(bare)) found.push("frailRefs");
  return found;
};
