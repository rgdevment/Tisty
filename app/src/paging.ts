import type { Filed } from "./core";
import { DOC } from "./markdown";

const bare = (line: string): string => line.replace(/^\s*(?:>\s?)*/, "").replace(/^[-*+]\s+/, "");

const unfenced = (body: string): string => {
  let open: { mark: string; wide: number } | null = null;
  return body
    .split("\n")
    .map((line) => {
      const said = bare(line);
      const mark = said[0] === "`" || said[0] === "~" ? said[0] : null;
      const wide = mark ? said.length - said.replace(new RegExp(`^\\${mark}+`), "").length : 0;
      if (open) {
        if (mark === open.mark && wide >= open.wide) open = null;
        return "";
      }
      if (mark && wide >= 3) {
        open = { mark, wide };
        return "";
      }
      return line;
    })
    .join("\n");
};

/// The pages a body names, in the order it names them — which is the order they are read in, so
/// the index can be drawn from the text itself rather than from what the log last settled.
export const namesIn = (body: string): string[] => {
  const found = new Set<string>();
  const said = unfenced(body).replace(/(`+)[\s\S]*?\1/g, " ");
  const asks = /!\[(?:\\[\s\S]|[^\\[\]\n])*\]\(\s*<?tisty:doc\/([^)>\s]+)/g;
  for (const [, id] of said.matchAll(asks)) found.add(id);
  return [...found];
};

export const named = (body: string): Set<string> => new Set(namesIn(body));

export type Moved = "done" | "held" | "unseen";

/// The pages of a document in the order it reads them: the ones its text names, where it names
/// them, and then the ones it does not, which keep the places they came in with. Whatever draws a
/// book — the index, the print, the parcel — reads it the same way.
export const inTextOrder = (pages: Filed[], told: string[]): Filed[] => {
  const held = new Set(told);
  const named = told
    .map((file) => pages.find((one) => one.file === file))
    .filter((one): one is Filed => Boolean(one));
  return [...named, ...pages.filter((one) => !held.has(one.file))];
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
