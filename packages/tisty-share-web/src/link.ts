// What a link names: the id the server keeps, and the key in the # that never leaves the browser.
export type Link = { readonly id: string; readonly key: string };

// Eight letters and digits, as the server mints them (tisty-share-server's keys.rs).
const ID = /^\/([0-9A-Za-z]{8})\/?$/;

// tisty.app already answers a page that is not there, in the reader's language.
export const NOWHERE = "https://www.tisty.app/404";

// A path that names no document, or one the server no longer has, is a page that is not there.
const LOST = new Set(["nolink", "missing", "gone"]);

export const lost = (word: string): boolean => LOST.has(word);

// A path that names no document is no link; one without its # still is, with no key to open it.
export const linkOf = (path: string, hash: string): Link | null => {
  const id = ID.exec(path)?.[1];
  return id ? { id, key: hash.replace(/^#/, "") } : null;
};
