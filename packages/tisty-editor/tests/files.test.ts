import { describe, expect, it } from "vitest";
import { fileCard, kept } from "../src/files";

const pageOf = (html: string): Document => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = html;
  return page;
};

describe("an attachment's name", () => {
  it("carries a stamp of its bytes, as the core names what it keeps", () => {
    expect(kept("cartilla-0123abcd.pdf")).toBe(true);
    expect(kept("attachments/01/cartilla-0123abcd.pdf")).toBe(true);
    for (const other of ["cartilla.pdf", "https://x.test/a-0123abcd.png", "Foto-0123ABCD.jpg"])
      expect(kept(other)).toBe(false);
  });
});

describe("a file drawn in place of its picture", () => {
  it("is a card on a line of its own, with its glyph, name and what the host says of it", () => {
    const page = pageOf('<p><img alt="La cartilla"></p>');
    const { card, kindIs } = fileCard(
      page,
      page.querySelector("img") as Element,
      "pdf",
      "La cartilla",
      "PDF",
    );
    expect(card.className).toBe("doc-card is-file");
    expect(card.dataset.family).toBe("pdf");
    expect(card.querySelector(".card-tile svg")).not.toBeNull();
    expect(card.querySelector(".card-name")?.textContent).toBe("La cartilla");
    expect(kindIs?.textContent).toBe("PDF");
    expect(page.querySelector("img")).toBeNull();
  });

  it("is a chip the sheet styles when it sits within words", () => {
    const page = pageOf('<p>Mira <img alt="Un ladrido"> aquí</p>');
    const { card, kindIs } = fileCard(
      page,
      page.querySelector("img") as Element,
      "mp3",
      "Un ladrido",
      "MP3",
    );
    expect(card.tagName).toBe("A");
    expect(card.className).toBe("card");
    expect(card.dataset.family).toBe("audio");
    expect(kindIs).toBeUndefined();
  });
});
