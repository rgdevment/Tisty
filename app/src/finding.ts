const TERMS_AT_MOST = 12;

export const folded = (text: string): string =>
  text.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

// Kept in step with `text::terms` in the core: the window and the store have to agree on what
// a search means.
export const terms = (query: string): string[] => {
  const found: string[] = [];
  let word = "";
  let quoted = false;

  for (const c of folded(query)) {
    if (c === '"' || c === "“" || c === "”") {
      if (word) {
        found.push(word);
        word = "";
      }
      quoted = !quoted;
    } else if (/\s/.test(c) && !quoted) {
      if (word) {
        found.push(word);
        word = "";
      }
    } else {
      word += c;
    }
    if (found.length >= TERMS_AT_MOST) return found;
  }
  if (word) found.push(word);
  return found;
};

export const matched = (text: string, query: string): boolean => {
  const flat = folded(text);
  return terms(query).every((one) => flat.includes(one));
};

export const begun = (text: string, query: string): boolean =>
  folded(text).startsWith(folded(query.trim()));
