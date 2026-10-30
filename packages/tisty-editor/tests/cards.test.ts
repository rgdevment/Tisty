import { describe, expect, it } from "vitest";
import { type Carded, type CardWord, carded } from "../src/cards";

const page = (html: string): Document => {
  const made = document.implementation.createHTMLDocument("");
  made.body.innerHTML = `<article class="page">${html}</article>`;
  return made;
};

const say = (word: CardWord, ...filled: string[]) => [word, ...filled].join(" ");

const CARDS: Record<string, Carded> = {
  "rina.md": { kind: "page", title: "La Rina", at: 2, of: 3 },
  "otro.md": { kind: "doc", title: "Otro" },
  "aqui.md": { kind: "self", title: "Aquí" },
  "viejo.md": { kind: "away", title: "Viejo" },
};
const card = (file: string): Carded => CARDS[file] ?? { kind: "gone" };

const link = (file: string, said: string) =>
  `<p><a class="card" data-doc="${file}">${said}</a></p>`;

describe("the cards a sheet draws", () => {
  it("draws a page with its place among the pages, and a way to open it", async () => {
    const drawn = page(link("rina.md", "La Rina"));
    carded(drawn, { card, say, blurb: async () => "Esta hoja es una página" }, () => true);
    const box = drawn.querySelector<HTMLElement>(".doc-card");
    expect(box?.className).toBe("doc-card is-page");
    expect(box?.dataset.doc).toBe("rina.md");
    expect(box?.getAttribute("role")).toBe("link");
    expect(box?.querySelector(".card-name")?.textContent).toBe("La Rina");
    expect(box?.querySelector(".chip")?.textContent).toBe("leafOfMany 2 3");
    expect(box?.querySelector(".card-go")?.textContent).toBe("›");
    expect(box?.querySelector(".card-tile svg")?.innerHTML).not.toBe("");
    await Promise.resolve();
    await Promise.resolve();
    expect(box?.querySelector(".card-blurb")?.textContent).toBe("Esta hoja es una página");
  });

  it("names every kind of card in the host's words", () => {
    const drawn = page(
      link("otro.md", "Otro") +
        link("aqui.md", "Aquí") +
        link("viejo.md", "Viejo") +
        link("ido.md", "Ido"),
    );
    carded(drawn, { card, say }, () => true);
    const chips = [...drawn.querySelectorAll(".card-meta > :first-child")].map(
      (one) => one.textContent,
    );
    expect(chips).toEqual(["kindDoc", "cardHere", "cardAway", "cardGone"]);
    const [, self, , gone] = [...drawn.querySelectorAll<HTMLElement>(".doc-card")];
    expect(self.querySelector(".card-blurb")?.textContent).toBe("docItself");
    expect(self.dataset.doc).toBeUndefined();
    expect(gone.querySelector(".card-name")?.textContent).toBe("Ido");
    expect(gone.querySelector(".card-go")).toBeNull();
  });

  it("puts the host's controls before the way in, and leaves a card inside a sentence alone", () => {
    const drawn = page(
      `${link("otro.md", "Otro")}<p>Ver <a class="card" data-doc="otro.md">Otro</a> antes.</p>`,
    );
    const controls = () => [drawn.createElement("button")];
    carded(drawn, { card, say, controls }, () => true);
    const meta = drawn.querySelector(".card-meta");
    expect([...(meta?.children ?? [])].map((one) => one.tagName)).toEqual([
      "SPAN",
      "BUTTON",
      "SPAN",
    ]);
    expect(drawn.querySelectorAll(".doc-card")).toHaveLength(1);
    expect(drawn.querySelector("p a.card")).not.toBeNull();
  });

  it("opens with the keyboard as with a click, and leaves a blurb out once the sheet is gone", async () => {
    const drawn = page(link("otro.md", "Otro"));
    carded(drawn, { card, say, blurb: async () => "tarde" }, () => false);
    const box = drawn.querySelector<HTMLElement>(".doc-card");
    let clicked = 0;
    box?.addEventListener("click", () => clicked++);
    box?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    box?.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));
    expect(clicked).toBe(1);
    await Promise.resolve();
    expect(box?.querySelector(".card-blurb")?.textContent).toBe("");
  });
});
