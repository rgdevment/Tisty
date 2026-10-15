import { dressed } from "../markdown";

// The shape the core draws, enough for the window's side of a note; the core's own tests hold the rest.
export const crated = (body: string): string =>
  body
    .split(/\n{2,}/)
    .map((block, at) => `<p data-at="${at}-${at + 1}">${inline(block)}</p>`)
    .join("");

const inline = (said: string): string =>
  said
    .replace(
      /!\[([^\]]*)\]\(<?(tisty:doc\/[^)>]+)>?\)/g,
      (_, alt, to) => `<a class="card" data-doc="${to.slice(10)}">${alt}</a>`,
    )
    .replace(/!\[([^\]]*)\]\(<?([^)>]+)>?\)/g, '<img src="$2" alt="$1" loading="lazy">')
    .replace(/\[([^\]]*)\]\(<?([^)>]+)>?\)/g, '<a href="$2">$1</a>')
    .replace(/\[\[([^\]]+)\]\]/g, '<span class="ref" data-ref="$1">$1</span>')
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");

export const composed = (body: string, steps?: string[]): string => dressed(crated(body), steps);
