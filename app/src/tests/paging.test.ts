import { describe, expect, it } from "vitest";
import type { Filed } from "../core";
import { DOC, docCard } from "../markdown";
import { card, filed, named, paged, pagesOf, under } from "../paging";

const all: Filed[] = [
  { id: "01A", file: "a3f1-0001", title: "Bases de datos", folder: null, archived: false },
  { id: "01B", file: "a3f1-0002", title: "El pod", folder: null, archived: false, pageOf: "01A" },
  { id: "01C", file: "a3f1-0003", title: "El túnel", folder: null, archived: false, pageOf: "01A" },
  { id: "01D", file: "a3f1-0004", title: "Otro", folder: null, archived: false },
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
  it("picks up every document it points at, as a card or as a link", () => {
    const body = "uno ![A](tisty:doc/a3f1-0002)\n\ndos [B](tisty:doc/a3f1-0003)";

    expect([...named(body)]).toEqual(["a3f1-0002", "a3f1-0003"]);
  });

  it("does not mistake an ordinary link or an attachment for a document", () => {
    const body = "[fuera](https://ejemplo.org) ![f](<attachments/charla-a3f9.mp4>)";

    expect(named(body).size).toBe(0);
  });

  it("counts nothing written inside code, which the core does not count either", () => {
    expect(named("`![A](tisty:doc/a3f1-0002)`").size).toBe(0);
    expect(named("```\n![A](tisty:doc/a3f1-0002)\n```").size).toBe(0);
  });

  it("reads a destination wrapped in angles the same as a bare one", () => {
    expect([...named("![A](<tisty:doc/a3f1-0002>)")]).toEqual(["a3f1-0002"]);
  });

  it("names what the block put in the text points at, so putting one in is found again", () => {
    expect(card("a3f1-0002", "El pod").attrs.src).toBe(`${DOC}a3f1-0002`);
    expect(named(`ya está: ${docCard("a3f1-0002", "El pod")}`).has("a3f1-0002")).toBe(true);
  });
});
