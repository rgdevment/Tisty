import type { Filed } from "../core";

const NONE: readonly Filed[] = [];

/** The documents standing in each folder and the pages of each document, in the order of the list. */
export interface TreeIndex {
  folders: Map<string | null, Filed[]>;
  pages: Map<string, Filed[]>;
}

// Read once per list, so a row asks a map instead of walking every document of the library.
export const treeIndex = (docs: readonly Filed[]): TreeIndex => {
  const folders = new Map<string | null, Filed[]>();
  const pages = new Map<string, Filed[]>();
  const put = <K>(map: Map<K, Filed[]>, key: K, one: Filed) => {
    const there = map.get(key);
    if (there) there.push(one);
    else map.set(key, [one]);
  };
  for (const one of docs) {
    if (one.pageOf) put(pages, one.pageOf, one);
    else put(folders, one.folder, one);
  }
  return { folders, pages };
};

export const standingIn = (index: TreeIndex, folder: string | null): readonly Filed[] =>
  index.folders.get(folder) ?? NONE;

export const pagesIn = (index: TreeIndex, doc: string): readonly Filed[] =>
  index.pages.get(doc) ?? NONE;

/** One answer per key for as long as it is kept: a tree drawing asks the same folder once per row. */
export const remembered = <K, V>(read: (key: K) => V): ((key: K) => V) => {
  const seen = new Map<K, V>();
  return (key) => {
    if (!seen.has(key)) seen.set(key, read(key));
    return seen.get(key) as V;
  };
};
