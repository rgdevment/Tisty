import { describe, expect, it } from "vitest";
import { DOC } from "../markdown";
import { cardMoved } from "../ui/writing";
import { opened } from "./mounted";

const card = (file: string, title: string) => `![${title}](${DOC}${file})`;

describe("moving a page's card in the editor", () => {
  it("puts the card before the one it was sent to", () => {
    const open = opened(`# Libro\n\n${card("a-0001", "Uno")}\n\n${card("a-0002", "Dos")}\n`);

    expect(cardMoved(open.editor, "a-0002", "a-0001")).toBe("done");

    const said = open.markdown();
    expect(said.indexOf("a-0002")).toBeLessThan(said.indexOf("a-0001"));
    open.shut();
  });

  it("puts the card last when it is sent to nowhere in particular", () => {
    const open = opened(
      `# Libro\n\n${card("a-0001", "Uno")}\n\n${card("a-0002", "Dos")}\n\n${card("a-0003", "Tres")}\n`,
    );

    expect(cardMoved(open.editor, "a-0001", null)).toBe("done");

    const said = open.markdown();
    expect(said.indexOf("a-0001")).toBeGreaterThan(said.indexOf("a-0003"));
    open.shut();
  });

  it("leaves everything alone when a card is sent where it already is", () => {
    const open = opened(`# Libro\n\n${card("a-0001", "Uno")}\n\n${card("a-0002", "Dos")}\n`);
    const was = open.markdown();

    expect(cardMoved(open.editor, "a-0001", "a-0001")).toBe("done");

    expect(open.markdown()).toBe(was);
    open.shut();
  });

  it("holds a card that sits inside a quote, since taking it out would tear the quote", () => {
    const open = opened(`# Libro\n\n> ${card("a-0001", "Uno")}\n\n${card("a-0002", "Dos")}\n`);
    const was = open.markdown();

    expect(cardMoved(open.editor, "a-0001", "a-0002")).toBe("held");

    expect(open.markdown()).toBe(was);
    open.shut();
  });

  it("holds a card that sits in a table cell, which is where it was sent", () => {
    const open = opened(
      `# Libro

| uno | dos |
| --- | --- |
| ${card("a-0001", "Uno")} | x |

${card("a-0002", "Dos")}
`,
    );
    const was = open.markdown();

    expect(cardMoved(open.editor, "a-0002", "a-0001")).toBe("held");

    expect(open.markdown()).toBe(was);
    open.shut();
  });

  it("says it never saw a page the body does not draw a card for", () => {
    const open = opened(`# Libro\n\n${card("a-0001", "Uno")}\n`);

    expect(cardMoved(open.editor, "a-0009", "a-0001")).toBe("unseen");
    expect(cardMoved(open.editor, "a-0001", "a-0009")).toBe("unseen");
    open.shut();
  });

  it("says it never saw a page named by a link in the middle of the text", () => {
    const open = opened(
      `# Libro\n\nComo conté en [Uno](${DOC}a-0001), ya está.\n\n${card("a-0002", "Dos")}\n`,
    );

    expect(cardMoved(open.editor, "a-0001", "a-0002")).toBe("unseen");
    open.shut();
  });
});
