import type { Filed } from "./core";
import { DOC } from "./markdown";

export const named = (body: string): Set<string> => {
  const found = new Set<string>();
  const said = body.replace(/(`+)[\s\S]*?\1/g, " ");
  const asks = /\[(?:\\[\s\S]|[^\\[\]\n])*\]\(\s*<?tisty:doc\/([^)>\s]+)/g;
  for (const [, id] of said.matchAll(asks)) found.add(id);
  return found;
};

export const card = (file: string, title: string) => ({
  type: "image" as const,
  attrs: { src: DOC + file, alt: title },
});

export const pagesOf = (all: Filed[] | undefined, file: string | undefined): Filed[] => {
  const mine = filed(all, file);
  return mine && !mine.pageOf ? (all ?? []).filter((one) => one.pageOf === mine.id) : [];
};

export const paged = (all: Filed[] | undefined, file: string | undefined): string[] =>
  pagesOf(all, file).map((one) => one.file);

export const filed = (all: Filed[] | undefined, file: string | undefined): Filed | undefined =>
  file ? all?.find((one) => one.file === file) : undefined;

export const under = (all: Filed[] | undefined, page: Filed | undefined): Filed | undefined =>
  page?.pageOf ? all?.find((one) => one.id === page.pageOf) : undefined;
