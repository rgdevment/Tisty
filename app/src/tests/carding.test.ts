import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";

const read = vi.hoisted(() => ({ asked: [] as string[] }));

vi.mock("../core", () => ({
  docRead: (file: string) => {
    read.asked.push(file);
    return Promise.resolve(`# ${file}\n\nPrimeras palabras de ${file}.`);
  },
}));

import { blurbing, carded, cardsOf } from "../ui/carding";

const doc = (file: string, more: Partial<Filed> = {}): Filed => ({
  id: `id-${file}`,
  file,
  title: file,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

const known = [
  doc("libro"),
  doc("cap1", { pageOf: "id-libro" }),
  doc("cap2", { pageOf: "id-libro" }),
  doc("viejo", { pageOf: "id-libro", away: true }),
  doc("suelto", { title: "  " }),
];

const page = (html: string): Document => {
  const one = document.implementation.createHTMLDocument("");
  one.body.innerHTML = `<article class="page">${html}</article>`;
  return one;
};

describe("what a card says about the document it names", () => {
  it("numbers the pages of this book in the order the text names them", () => {
    const seen = cardsOf(known, "libro", ["cap2", "cap1"]);
    expect(seen("cap2")).toEqual({ kind: "page", title: "cap2", at: 1, of: 2 });
    expect(seen("cap1")).toEqual({ kind: "page", title: "cap1", at: 2, of: 2 });
  });

  it("tells this document, an archived one, an untitled one and a missing one apart", () => {
    const seen = cardsOf(known, "libro", []);
    expect(seen("libro")).toEqual({ kind: "self", title: "libro" });
    expect(seen("viejo")).toEqual({ kind: "away", title: "viejo" });
    expect(seen("suelto").kind).toBe("doc");
    expect(seen("nada")).toEqual({ kind: "gone" });
    expect(cardsOf([doc("x", { gone: true })], undefined, [])("x")).toEqual({ kind: "gone" });
  });

  it("reads a blurb once for each version of its document", async () => {
    read.asked = [];
    const blurb = blurbing();
    expect(await blurb(doc("cap1", { wrote: "a" }))).toBe("Primeras palabras de cap1.");
    await blurb(doc("cap1", { wrote: "a" }));
    await blurb(doc("cap1", { wrote: "b" }));
    expect(read.asked).toEqual(["cap1", "cap1"]);
    expect(await blurb(undefined)).toBe("");
  });
});

describe("a card drawn on the page", () => {
  it("draws a card alone in its paragraph whole, and leaves one inside a sentence as a word", async () => {
    const drawn = page(
      '<p data-at="0-9"><a class="card" data-doc="cap1">cap1</a></p><p data-at="10-30">mira <a class="card" data-doc="cap2">cap2</a> ahí</p>',
    );
    carded(
      drawn,
      {
        card: cardsOf(known, "libro", ["cap1", "cap2"]),
        blurb: () => Promise.resolve("Hola."),
        onBlock: () => {},
      },
      () => true,
    );
    const card = drawn.querySelector<HTMLElement>(".doc-card");
    expect(card?.getAttribute("role")).toBe("link");
    expect(card?.dataset.doc).toBe("cap1");
    expect(card?.className).toBe("doc-card is-page");
    expect(card?.querySelector(".card-name")?.textContent).toBe("cap1");
    expect(card?.querySelector(".card-meta .chip")?.textContent).toBe("Page 1 of 2");
    expect(card?.querySelector<HTMLElement>(".card-edit")?.dataset.asked).toBe("code");
    let opened = 0;
    card?.addEventListener("click", () => {
      opened += 1;
    });
    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    card?.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
    expect(opened).toBe(1);
    expect(drawn.querySelector('p a.card[data-doc="cap2"]')).not.toBeNull();
    await Promise.resolve();
    expect(card?.querySelector(".card-blurb")?.textContent).toBe("Hola.");
  });

  it("opens nothing from this document's own card or a missing one, and says why", () => {
    const drawn = page(
      '<p><a class="card" data-doc="libro">libro</a></p><p><a class="card" data-doc="nada">Viaje</a></p>',
    );
    carded(drawn, { card: cardsOf(known, "libro", []) }, () => true);
    const [self, gone] = Array.from(drawn.querySelectorAll<HTMLElement>(".doc-card"));
    expect(self.dataset.doc).toBeUndefined();
    expect(self.querySelector(".card-edit")).toBeNull();
    expect(self.querySelector(".card-blurb")?.textContent).toBe("This very document");
    expect(gone.querySelector(".card-name")?.textContent).toBe("Viaje");
    expect(gone.querySelector(".chip.gone")?.textContent).toBe("Gone");
  });

  it("draws a card with blank words around it, and keeps the chips where the page knows nothing", () => {
    const spaced = page('<p> <a class="card" data-doc="cap1">cap1</a> </p>');
    carded(spaced, { card: cardsOf(known, "libro", []) }, () => true);
    expect(spaced.querySelector(".doc-card")).not.toBeNull();
  });

  it("keeps the chips where the page knows nothing about documents", () => {
    const drawn = page('<p><a class="card" data-doc="cap1">cap1</a></p>');
    carded(drawn, {}, () => true);
    expect(drawn.querySelector(".doc-card")).toBeNull();
  });

  it("writes no blurb into a page that is gone", async () => {
    const drawn = page('<p><a class="card" data-doc="cap1">cap1</a></p>');
    carded(
      drawn,
      { card: cardsOf(known, "libro", []), blurb: () => Promise.resolve("Tarde.") },
      () => false,
    );
    await Promise.resolve();
    expect(drawn.querySelector(".card-blurb")?.textContent).toBe("");
  });
});
