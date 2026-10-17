import { beforeEach, describe, expect, it, vi } from "vitest";

const net = vi.hoisted(() => ({ kept: null as unknown, fetched: null as unknown, asked: 0 }));

vi.mock("../core", () => ({
  glimpseKept: () => Promise.resolve(net.kept),
  glimpseFetch: () => {
    net.asked += 1;
    return Promise.resolve(net.fetched);
  },
}));

import { webCarded } from "../ui/webcarding";

const page = (html: string) => {
  const doc = document.implementation.createHTMLDocument("p");
  doc.body.innerHTML = `<article class="page">${html}</article>`;
  return doc;
};

const settle = () => new Promise((done) => setTimeout(done, 0));

beforeEach(() => {
  net.kept = null;
  net.fetched = null;
  net.asked = 0;
});

describe("a web link alone in its paragraph", () => {
  it("is drawn as a card with its host, and a link inside a sentence stays a link", async () => {
    const doc = page(
      '<p data-at="0-20"><a href="https://www.tisty.app/docs">Tisty</a></p><p>ver <a href="https://x.cl">x</a> hoy</p>',
    );
    webCarded(doc, () => true);
    const card = doc.querySelector<HTMLAnchorElement>("a.doc-card.is-web");
    expect(card?.getAttribute("href")).toBe("https://www.tisty.app/docs");
    expect(card?.querySelector(".kind")?.textContent).toBe("tisty.app");
    expect(doc.querySelectorAll("a.doc-card")).toHaveLength(1);
    expect(card?.closest("p")?.dataset.at).toBe("0-20");
  });

  it("shows a kept preview, and asks the network only when the person presses for it", async () => {
    net.kept = { title: "Tisty", said: "Notas y tareas" };
    const kept = page('<p><a href="https://tisty.app">tisty.app</a></p>');
    webCarded(kept, () => true);
    await settle();
    expect(kept.querySelector(".card-name")?.textContent).toBe("Tisty");
    expect(kept.querySelector<HTMLButtonElement>(".card-edit")?.hidden).toBe(true);

    net.kept = null;
    net.fetched = { title: "Traída", shot: "data:image/png;base64,AA==" };
    const bare = page(
      '<p><a href="https://tisty.app/otra">tisty.app</a></p><p><a href="https://x.cl"><img src="b.svg"></a></p>',
    );
    webCarded(bare, () => true);
    await settle();
    const ask = bare.querySelector<HTMLButtonElement>(".card-edit");
    expect(ask?.hidden).toBe(false);
    expect(net.asked).toBe(0);
    ask?.click();
    await settle();
    expect(net.asked).toBe(1);
    expect(bare.querySelector(".card-name")?.textContent).toBe("Traída");
    expect(bare.querySelector("img.card-thumb")).not.toBeNull();
    expect(bare.querySelectorAll(".doc-card")).toHaveLength(1);
    expect(bare.querySelector('img[src="b.svg"]')).not.toBeNull();
    expect(bare.querySelector(".is-web.has-shot .card-tile")).toBeNull();
  });

  it("reads apart from its neighbours with nothing of its own: its site's letter and a plain address", async () => {
    const doc = page(
      '<p><a href="https://www.tisty.app/blog/">https://www.tisty.app/blog/</a></p><p><a href="https://www.tisty.app/x">otra</a></p>',
    );
    webCarded(doc, () => true);
    await settle();
    const [first, second] = doc.querySelectorAll<HTMLElement>(".doc-card");
    expect(first.classList.contains("is-bare")).toBe(true);
    expect(first.querySelector(".card-tile.is-letter")?.textContent).toBe("T");
    expect(first.querySelector(".card-name")?.textContent).toBe("tisty.app/blog");
    expect(second.querySelector(".card-name")?.textContent).toBe("otra");
    const hue = (one: Element) =>
      (one.querySelector(".card-tile") as HTMLElement).style.getPropertyValue("--hue");
    expect(hue(first)).toMatch(/^var\(--tisty-hue-[a-z]+\)$/);
    expect(hue(first)).toBe(hue(second));

    net.fetched = { title: "El blog" };
    first.querySelector<HTMLButtonElement>(".card-edit")?.click();
    await settle();
    expect(first.classList.contains("is-bare")).toBe(false);
  });
});
