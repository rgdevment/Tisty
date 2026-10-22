import { t } from "../locales";
import type { Told } from "./printing";

// The card sits under the title, on screen as on paper, so the sheets lay out the same in both.
export const cardOn = (page: Document, card: readonly Told[] | undefined) => {
  const sheet = page.querySelector("article.page");
  for (const one of sheet?.querySelectorAll(":scope > .stamp") ?? []) one.remove();
  if (!sheet || !card?.length) return;
  const list = page.createElement("dl");
  list.className = "stamp";
  list.setAttribute("inert", "");
  for (const { label, said } of card) {
    const pair = page.createElement("div");
    const named = page.createElement("dt");
    named.textContent = label;
    const told = page.createElement("dd");
    told.textContent = said;
    pair.append(named, told);
    list.append(pair);
  }
  const head = Array.from(sheet.children).find(
    (one) => !one.matches(".front, .fronted, .sheet-turn, .break-said"),
  );
  if (head && /^H[1-3]$/.test(head.tagName)) head.after(list);
  else if (head) head.before(list);
  else sheet.append(list);
};

export const creditOn = (page: Document) => {
  const said = page.createElement("p");
  said.className = "credit";
  const home = page.createElement("a");
  home.href = "https://www.tisty.app";
  home.textContent = t("pdfCredit");
  said.append(home);
  page.querySelector("article.page")?.append(said);
};
