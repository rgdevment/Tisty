import { describe, expect, it } from "vitest";
import { cardOf, movedCard, putCard } from "../ui/carded";

const book = [
  "# Libro",
  "",
  "Algo.",
  "",
  "![El pod](tisty:doc/a3f1-0002)",
  "",
  "![El túnel](tisty:doc/a3f1-0003)",
  "",
].join("\n");

const order = (said: string) =>
  Array.from(said.matchAll(/!\[([^\]]*)\]\(tisty:doc\//g)).map((one) => one[1]);

describe("a card for a page", () => {
  it("names the page and escapes the brackets of its title", () => {
    expect(cardOf("a3f1-0002", "El [gran] pod")).toBe("![El \\[gran\\] pod](tisty:doc/a3f1-0002)");
  });

  it("goes at the end of what the book says, a blank line apart", () => {
    expect(putCard("# Libro\n\nAlgo.\n\n\n", "a3f1-0004", "Otra")).toBe(
      "# Libro\n\nAlgo.\n\n![Otra](tisty:doc/a3f1-0004)\n",
    );
  });

  it("goes nowhere in a book that says nothing yet, which it would name", () => {
    expect(putCard("  \n", "a3f1-0004", "Otra")).toBeNull();
  });
});

describe("moving a page's card", () => {
  it("sends it to the end", () => {
    const done = movedCard(book, "a3f1-0002", null);
    expect(done.moved).toBe("done");
    expect(order(done.body)).toEqual(["El túnel", "El pod"]);
  });

  it("puts it before another card", () => {
    const done = movedCard(book, "a3f1-0003", "a3f1-0002");
    expect(done.moved).toBe("done");
    expect(order(done.body)).toEqual(["El túnel", "El pod"]);
    expect(done.body.startsWith("# Libro\n\nAlgo.\n\n")).toBe(true);
  });

  it("changes nothing when the card already sits there", () => {
    expect(movedCard(book, "a3f1-0003", null)).toEqual({ moved: "done", body: book });
    expect(movedCard(book, "a3f1-0002", "a3f1-0003")).toEqual({ moved: "done", body: book });
  });

  it("says when it cannot find the card it was asked to move", () => {
    expect(movedCard(book, "a3f1-0009", null).moved).toBe("unseen");
    expect(movedCard(book, "a3f1-0002", "a3f1-0009").moved).toBe("unseen");
  });

  it("holds a card that sits inside other words or a list", () => {
    const inside = "# Libro\n\nmira ![El pod](tisty:doc/a3f1-0002) aquí\n";
    expect(movedCard(inside, "a3f1-0002", null).moved).toBe("held");
  });

  it("never takes a card from the top, nor puts one there, which would rename the book", () => {
    const top = "![El pod](tisty:doc/a3f1-0002)\n\n![El túnel](tisty:doc/a3f1-0003)\n";
    expect(movedCard(top, "a3f1-0002", null).moved).toBe("titled");
    expect(movedCard(top, "a3f1-0003", "a3f1-0002").moved).toBe("titled");
  });

  it("does not see a card written inside a code fence", () => {
    const fenced = "# Libro\n\n```\n![El pod](tisty:doc/a3f1-0002)\n```\n";
    expect(movedCard(fenced, "a3f1-0002", null).moved).toBe("unseen");
  });
});
