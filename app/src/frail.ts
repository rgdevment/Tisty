const BLOCKS = /<\/?(div|details|summary|section|article|aside|figure|figcaption|table|iframe|video|audio|form|blockquote)\b/i;

export const frail = (text: string): string[] => {
  const found: string[] = [];
  const bare = text.replace(/^```[\s\S]*?^```/gm, "").replace(/^ {4}.*$/gm, "");

  if (/^---\r?\n[\s\S]*?\r?\n---\r?\n/.test(text)) found.push("frailFront");
  if (BLOCKS.test(bare)) found.push("frailHtml");
  if (/(^|[^\\])\[\^[^\]]+\]/m.test(bare)) found.push("frailNotes");
  if (/^\[[^\\\]]+\]:\s*\S/m.test(bare)) found.push("frailRefs");
  if (/^\|[\s|]*:?-+:[\s|]*\|/m.test(bare) || /\|\s*:-+\s*\|/.test(bare)) found.push("frailAligned");

  return found;
};
