import { describe, expect, it } from "vitest";
import { readAs } from "../src/reading";
import type { ReadWord } from "../src/words";

const say = (word: ReadWord, ...filled: string[]) => [word, ...filled].join(" ");

const page = (html: string): Document => {
  const made = document.implementation.createHTMLDocument("");
  made.body.innerHTML = `<article class="page">${html}</article>`;
  return made;
};

describe("a sheet read as Tisty draws it", () => {
  it("names its callouts, stages and cards in the host's words, and draws a web link as its site", async () => {
    const drawn = page(
      '<blockquote data-callout="tip"><p>Ojo</p></blockquote>' +
        '<ul><li class="task">Primero</li></ul>' +
        '<p><a class="card" data-doc="otro.md">Otro</a></p>' +
        '<p><a href="https://tisty.app">tisty.app</a></p>',
    );
    const read = readAs(drawn, {
      say,
      locale: "es",
      live: () => true,
      cards: { card: () => ({ kind: "doc", title: "Otro" }) },
      sites: {},
    });
    await read.drawn;
    expect(drawn.querySelector<HTMLElement>("blockquote")?.dataset.said).toBe("saidtip");
    expect(drawn.querySelector("li.task")?.id).toBe("stage--1");
    expect(drawn.querySelector(".doc-card .kind")?.textContent).toBe("kindDoc");
    expect(drawn.querySelector("a.doc-card.is-web .kind")?.textContent).toBe("tisty.app");
  });

  it("brings the host's own pieces after the stages are read, and leaves out what it is not given", () => {
    const drawn = page(
      '<ul><li class="task">Paso</li></ul><p>Ver <span class="ref" data-ref="#1"></span></p>' +
        '<p><a class="card" data-doc="otro.md">Otro</a></p><p><a href="https://x.cl">x.cl</a></p>',
    );
    readAs(drawn, {
      say,
      locale: "es",
      live: () => true,
      brought: () => drawn.querySelector("li.task")?.append(" y más"),
    });
    expect(drawn.querySelector(".ref")?.textContent).toBe("Paso");
    expect(drawn.querySelector("li.task")?.textContent).toBe("Paso y más");
    expect(drawn.querySelector(".doc-card")).toBeNull();
  });
});
