import { t } from "../locales";
import type { Told } from "./printing";

// The card is a cover of its own, so the pages after it are the sheets seen on screen.
export const coverOn = (page: Document, card: readonly Told[] | undefined) => {
  const sheet = page.querySelector("article.page");
  if (!sheet || !card?.length) return;
  const cover = page.createElement("section");
  cover.className = "cover";
  const named = sheet.querySelector(":scope > h1")?.textContent?.trim();
  if (named) {
    const title = page.createElement("p");
    title.className = "cover-title";
    title.textContent = named;
    cover.append(title);
  }
  const list = page.createElement("dl");
  list.className = "stamp";
  for (const { label, said } of card) {
    const pair = page.createElement("div");
    const term = page.createElement("dt");
    term.textContent = label;
    const told = page.createElement("dd");
    told.textContent = said;
    pair.append(term, told);
    list.append(pair);
  }
  cover.append(list);
  sheet.prepend(cover);
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
