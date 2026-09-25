import { describe, expect, it } from "vitest";
import type { Filed } from "../core";
import { fill } from "../locales";
import { DOC, docCard } from "../markdown";
import { card, filed, inTextOrder, named, paged, pagesOf, under } from "../paging";

const all: Filed[] = [
  {
    id: "01A",
    file: "a3f1-0001",
    title: "Bases de datos",
    folder: null,
    archived: false,
    away: false,
  },
  {
    id: "01B",
    file: "a3f1-0002",
    title: "El pod",
    folder: null,
    archived: false,
    away: false,
    pageOf: "01A",
  },
  {
    id: "01C",
    file: "a3f1-0003",
    title: "El túnel",
    folder: null,
    archived: false,
    away: false,
    pageOf: "01A",
  },
  { id: "01D", file: "a3f1-0004", title: "Otro", folder: null, archived: false, away: false },
];

describe("the pages of a document", () => {
  it("comes back in the order the papers arrive in, which is the order they sit in", () => {
    expect(paged(all, "a3f1-0001")).toEqual(["a3f1-0002", "a3f1-0003"]);
  });

  it("is empty for a page, because a page holds none", () => {
    expect(pagesOf(all, "a3f1-0002")).toEqual([]);
  });

  it("is empty for a document nobody knows", () => {
    expect(paged(all, "a3f1-0009")).toEqual([]);
    expect(paged(undefined, "a3f1-0001")).toEqual([]);
  });

  it("finds the document a page belongs to, and none for a document of its own", () => {
    expect(under(all, filed(all, "a3f1-0002"))?.title).toBe("Bases de datos");
    expect(under(all, filed(all, "a3f1-0004"))).toBeUndefined();
  });
});

describe("what a body names", () => {
  it("does not count a card shown inside a fence, whichever marker wrote it", () => {
    for (const fence of ["```", "~~~"]) {
      const body = `# Libro\n\n${fence}md\n![Uno](tisty:doc/a-0001)\n${fence}\n\n![Dos](tisty:doc/a-0002)\n`;

      expect([...named(body)]).toEqual(["a-0002"]);
    }
  });

  it("reads a fence written inside a quote or a list item as code all the same", () => {
    for (const body of [
      "# Libro\n\n> ```\n> ![Uno](tisty:doc/a-0001)\n> ```\n\n![Dos](tisty:doc/a-0002)\n",
      "# Libro\n\n- ```\n  ![Uno](tisty:doc/a-0001)\n  ```\n\n![Dos](tisty:doc/a-0002)\n",
    ]) {
      expect([...named(body)]).toEqual(["a-0002"]);
    }
  });

  it("holds a fence open until it closes at least as wide as it opened", () => {
    const body =
      "# Libro\n\n````md\n```\n![Uno](tisty:doc/a-0001)\n```\n````\n\n![Dos](tisty:doc/a-0002)\n";

    expect([...named(body)]).toEqual(["a-0002"]);
  });

  it("counts the cards, and leaves a page merely mentioned in a sentence alone", () => {
    const body = "uno ![A](tisty:doc/a3f1-0002)\n\ncomo conté en [B](tisty:doc/a3f1-0003), ya está";

    expect([...named(body)]).toEqual(["a3f1-0002"]);
  });

  it("does not mistake an ordinary link or an attachment for a document", () => {
    const body = "[fuera](https://ejemplo.org) ![f](<attachments/charla-a3f9.mp4>)";

    expect(named(body).size).toBe(0);
  });

  it("counts nothing written inside code, which the core does not count either", () => {
    expect(named("`![A](tisty:doc/a3f1-0002)`").size).toBe(0);
    expect(named("```\n![A](tisty:doc/a3f1-0002)\n```").size).toBe(0);
  });

  it("reads a title that carries brackets, which the core reads too", () => {
    expect([...named(docCard("a3f1-0002", "Capítulo 1 [borrador]"))]).toEqual(["a3f1-0002"]);
  });

  it("refuses a label holding a link, which is what the core refuses", () => {
    expect(named("[uno [dos](tisty:doc/a3f1-0002)](tisty:doc/a3f1-0003)").has("a3f1-0003")).toBe(
      false,
    );
  });

  it("reads a destination wrapped in angles the same as a bare one", () => {
    expect([...named("![A](<tisty:doc/a3f1-0002>)")]).toEqual(["a3f1-0002"]);
  });

  it("names what the block put in the text points at, so putting one in is found again", () => {
    expect(card("a3f1-0002", "El pod").attrs.src).toBe(`${DOC}a3f1-0002`);
    expect(named(`ya está: ${docCard("a3f1-0002", "El pod")}`).has("a3f1-0002")).toBe(true);
  });
});

describe("the order a book is read in", () => {
  const page = (file: string, title: string): Filed => ({
    id: file,
    file,
    title,
    folder: null,
    archived: false,
    away: false,
    pageOf: "01A",
  });

  const held = [page("a-0002", "Uno"), page("a-0003", "Dos"), page("a-0004", "Tres")];

  it("takes the pages the text names, where it names them", () => {
    const said = inTextOrder(held, ["a-0004", "a-0002", "a-0003"]);

    expect(said.map((one) => one.file)).toEqual(["a-0004", "a-0002", "a-0003"]);
  });

  it("leaves the ones it does not name exactly where they were", () => {
    const said = inTextOrder(held, ["a-0004"]);

    expect(said.map((one) => one.file)).toEqual(["a-0002", "a-0003", "a-0004"]);
  });

  it("deals the named ones back out between their own places, moving no other", () => {
    const said = inTextOrder(held, ["a-0004", "a-0002"]);

    expect(said.map((one) => one.file)).toEqual(["a-0004", "a-0003", "a-0002"]);
  });

  it("says nothing about a name the document does not hold", () => {
    const said = inTextOrder(held, ["a-9999", "a-0003"]);

    expect(said.map((one) => one.file)).toEqual(["a-0002", "a-0003", "a-0004"]);
  });

  it("keeps the log's order when the text names none of them", () => {
    expect(inTextOrder(held, []).map((one) => one.file)).toEqual(["a-0002", "a-0003", "a-0004"]);
  });
});

describe("a name a sentence says more than once", () => {
  it("is written in every place the sentence says it", () => {
    const said = fill("pageOfSure", "Notas", "Diario");

    expect(said).not.toContain("{other}");
    expect(said).not.toContain("{name}");
    expect(said.split("Diario").length - 1).toBeGreaterThan(1);
  });
});
