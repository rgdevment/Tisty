import { invoke } from "@tauri-apps/api/core";
import type { Doc, Filed } from "./core";
import { locale } from "./locales";

export const docTranslate = (id: string, lang: string, from?: string): Promise<Doc> =>
  invoke("doc_translate", { id, lang, from });
/// Joins a document to the languages of `of`, or with no `of` takes it out of its own.
export const docTongue = (id: string, of?: string, lang?: string, ofLang?: string): Promise<void> =>
  invoke("doc_tongue", { id, of, lang, ofLang });

/// The paper its other languages are laid on follows the one just changed.
export const docDress = (id: string, paper: string, sheets: boolean, landscape: boolean) =>
  invoke<void>("doc_dress", { id, paper, sheets, landscape });

const CHOSEN = "tisty.tongue.";

// The reader's own choice, kept on this computer: which language a group opens in is nobody else's.
export const chosenTongue = (anchor: string): string | null => {
  try {
    return localStorage.getItem(CHOSEN + anchor);
  } catch {
    return null;
  }
};

export const chooseTongue = (anchor: string, lang: string) => {
  try {
    localStorage.setItem(CHOSEN + anchor, lang);
  } catch {
    // Without storage the choice lasts until the window closes, which is all it loses.
  }
};

/// A language by its own name, as a reader of it would look for it: «Español», «English».
export const tongueName = (tag: string): string => {
  try {
    const said = new Intl.DisplayNames([tag], { type: "language" }).of(tag);
    return said ? said.charAt(0).toLocaleUpperCase(tag) + said.slice(1) : tag;
  } catch {
    return tag;
  }
};

const base = (tag: string | null | undefined) => tag?.split("-")[0].toLowerCase();

/// The languages a document is one of, in the order they joined; empty when it stands alone.
export const tonguesOf = (docs: readonly Filed[], doc: Filed | undefined): Filed[] => {
  const anchor = doc?.tongue?.[0];
  if (!anchor) return [];
  return docs
    .filter((one) => one.tongue?.[0] === anchor)
    .sort((a, b) => (a.tongue?.[1] ?? 0) - (b.tongue?.[1] ?? 0));
};

/// The one a group opens in: the one open now, the reader's choice, the app's language, the first.
export const preferred = (members: readonly Filed[], open?: string | null): Filed | undefined => {
  const here = members.find((one) => one.file === open);
  if (here) return here;
  const anchor = members[0]?.tongue?.[0];
  const picked = anchor ? chosenTongue(anchor) : null;
  const app = locale();
  return (
    members.find((one) => picked && one.lang === picked) ??
    members.find((one) => one.lang === app) ??
    members.find((one) => base(one.lang) === base(app)) ??
    members[0]
  );
};

/// One row for each group of languages that stand together; the others step aside behind it.
export const shown = (docs: readonly Filed[], open?: string | null): Filed[] => {
  const groups = new Map<string, Filed[]>();
  for (const one of docs) {
    if (!one.tongue) continue;
    const key = `${one.tongue[0]}|${one.folder ?? ""}|${one.away}`;
    groups.set(key, [...(groups.get(key) ?? []), one]);
  }
  const standing = new Set<string>();
  for (const members of groups.values()) {
    members.sort((a, b) => (a.tongue?.[1] ?? 0) - (b.tongue?.[1] ?? 0));
    const one = preferred(members, open);
    if (one) standing.add(one.id);
  }
  return docs.filter((one) => !one.tongue || standing.has(one.id));
};

/// Where a page stands in its own language, found in another: the page at the same place, or the
/// document itself when that language has fewer pages.
export const twinOf = (docs: readonly Filed[], here: Filed, tongue: Filed): Filed => {
  if (!here.pageOf) return tongue;
  const at = docs
    .filter((one) => one.pageOf === here.pageOf)
    .findIndex((one) => one.id === here.id);
  return docs.filter((one) => one.pageOf === tongue.id)[at] ?? tongue;
};
