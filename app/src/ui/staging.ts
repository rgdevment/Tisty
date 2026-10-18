import { charsAt, spanned } from "./paging";

const SHOWN_AT_MOST = 60;

// A document's stages are counted as they are read, so #2 is the second box a reader meets.
// A heading's id never holds two dashes in a row, so a stage's id cannot take one of theirs.
export const staged = (page: Document) => {
  const stages = Array.from(page.querySelectorAll<HTMLElement>(".page li.task"));
  stages.forEach((one, at) => {
    one.id ||= `stage--${at + 1}`;
  });
  for (const ref of page.querySelectorAll<HTMLElement>('.page .ref[data-ref^="#"]')) {
    const asked = Number(/^#(\d{1,3})$/.exec(ref.dataset.ref ?? "")?.[1] ?? 0);
    const stage = asked > 0 ? stages[asked - 1] : undefined;
    if (!stage) continue;
    const words = (stage.textContent ?? "").replace(/\s+/g, " ").trim();
    ref.textContent =
      words.length > SHOWN_AT_MOST ? `${words.slice(0, SHOWN_AT_MOST - 1)}…` : words;
    ref.title = `#${asked}`;
    ref.classList.add("is-stage");
    ref.dataset.stage = stage.id;
  }
};

export const steppedTitle = (line: string): string | null => {
  const found = /^[-*+]\s+\[ \]\s+(.+)$/.exec(line);
  const title = found?.[1]
    .replace(/\\+\s*$/, "")
    .replace(/\s+/g, " ")
    .trim();
  return title || null;
};

export const stepLine = (body: string, [from, end]: [number, number]): string | null => {
  if (from > 0 && body[from - 1] !== "\n") return null;
  const line = body.slice(from, end).replace(/\r?\n?$/, "");
  return line.includes("\n") ? null : line;
};

// Only an open step with nothing under it can leave as a part: what it holds would have nowhere to go.
export const offeredAsParts = (page: Document, said: string, body: string) => {
  for (const one of page.querySelectorAll<HTMLElement>(".page > ul > li.task[data-at]")) {
    if (one.querySelector(':scope > .check[data-done="true"], ul, ol, pre, blockquote')) continue;
    const at = spanned(one.dataset.at ?? null);
    const line = at && stepLine(body, [charsAt(body, at[0]), charsAt(body, at[1])]);
    if (!line || !steppedTitle(line)) continue;
    const ask = page.createElement("button");
    ask.type = "button";
    ask.className = "to-part";
    ask.dataset.asked = "part";
    ask.textContent = "↗";
    ask.title = said;
    ask.setAttribute("aria-label", said);
    one.append(ask);
  }
};
