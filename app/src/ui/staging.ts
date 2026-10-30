import { charsAt, spanned, stepLine, steppedTitle } from "tisty-editor/read";

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
