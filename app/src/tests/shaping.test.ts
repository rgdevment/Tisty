import { describe, expect, it } from "vitest";
import { endlessTall, MARGIN, SIZES } from "../ui/paper";
import { shapesOf, titled } from "../ui/shaping";

const doc = (...content: unknown[]) => ({ type: "doc", content });
const words = (text: string) => [{ type: "text", text }];

describe("turning a document into shapes for the page", () => {
  it("keeps a heading with its level", () => {
    const found = shapesOf(doc({ type: "heading", attrs: { level: 2 }, content: words("Hola") }));

    expect(found).toEqual([
      { kind: "heading", level: 2, runs: [expect.objectContaining({ text: "Hola" })] },
    ]);
  });

  it("carries bold, italic and code through", () => {
    const found = shapesOf(
      doc(
        { type: "paragraph", content: words("título") },
        {
          type: "paragraph",
          content: [
            { type: "text", text: "a", marks: [{ type: "bold" }] },
            { type: "text", text: "b", marks: [{ type: "italic" }] },
            { type: "text", text: "c", marks: [{ type: "code" }] },
          ],
        },
      ),
    );

    const runs = found[1].kind === "para" ? found[1].runs : [];
    expect(runs[0].bold).toBe(true);
    expect(runs[1].italic).toBe(true);
    expect(runs[2].code).toBe(true);
  });

  it("keeps where a link points", () => {
    const found = shapesOf(
      doc(
        { type: "paragraph", content: words("título") },
        {
          type: "paragraph",
          content: [
            { type: "text", text: "ir", marks: [{ type: "link", attrs: { href: "https://a.b" } }] },
          ],
        },
      ),
    );

    const runs = found[1].kind === "para" ? found[1].runs : [];
    expect(runs[0].href).toBe("https://a.b");
  });

  it("numbers an ordered list and bullets an unordered one", () => {
    const item = (text: string) => ({
      type: "listItem",
      content: [{ type: "paragraph", content: words(text) }],
    });
    const ordered = shapesOf(doc({ type: "orderedList", content: [item("uno"), item("dos")] }));
    const bullets = shapesOf(doc({ type: "bulletList", content: [item("uno")] }));

    expect(ordered.map((one) => (one.kind === "bullet" ? one.mark : ""))).toEqual(["1.", "2."]);
    expect(bullets[0].kind === "bullet" && bullets[0].mark).toBe("•");
  });

  it("shows a task as ticked or not", () => {
    const task = (checked: boolean) => ({
      type: "taskItem",
      attrs: { checked },
      content: [{ type: "paragraph", content: words("algo") }],
    });
    const found = shapesOf(doc({ type: "taskList", content: [task(true), task(false)] }));

    expect(found.map((one) => (one.kind === "bullet" ? one.mark : ""))).toEqual(["☑", "☐"]);
  });

  it("takes an image out of the paragraph that held it", () => {
    const found = shapesOf(
      doc({ type: "paragraph", content: [{ type: "image", attrs: { src: "a.png" } }] }),
    );

    expect(found).toEqual([{ kind: "image", src: "a.png" }]);
  });

  it("splits a code block into its lines", () => {
    const found = shapesOf(
      doc({ type: "codeBlock", content: [{ type: "text", text: "uno\ndos" }] }),
    );

    expect(found[0].kind === "code" && found[0].runs.map((one) => one.text)).toEqual([
      "uno",
      "dos",
    ]);
  });

  it("keeps a rule, which is where a page break lives", () => {
    expect(shapesOf(doc({ type: "horizontalRule" }))).toEqual([{ kind: "rule" }]);
  });

  it("leaves an empty paragraph out rather than printing a blank line", () => {
    expect(shapesOf(doc({ type: "paragraph" }))).toEqual([]);
  });
});

describe("the size of the sheet", () => {
  it("gives A4 and Letter the sizes a printer knows", () => {
    expect(SIZES.a4).toEqual([595.28, 841.89]);
    expect(SIZES.letter).toEqual([612, 792]);
  });

  it("grows an endless sheet with what it holds", () => {
    const one = endlessTall([{ kind: "para", runs: [{ text: "corto" }] }]);
    const many = endlessTall(
      Array.from({ length: 40 }, () => ({
        kind: "para" as const,
        runs: [{ text: "x".repeat(200) }],
      })),
    );

    expect(many).toBeGreaterThan(one);
    expect(one).toBeGreaterThanOrEqual(MARGIN * 2);
  });

  it("never asks for a sheet taller than a reader will open", () => {
    const huge = Array.from({ length: 5000 }, () => ({
      kind: "image" as const,
      src: "a.png",
    }));

    expect(endlessTall(huge)).toBeLessThanOrEqual(14000);
  });
});

describe("the first line, which is also the title", () => {
  it("reads a plain first line as the title, as the screen does", () => {
    const found = shapesOf(
      doc(
        { type: "paragraph", content: words("prueba") },
        { type: "paragraph", content: words("cuerpo") },
      ),
    );

    expect(found[0]).toEqual({
      kind: "heading",
      level: 1,
      runs: [expect.objectContaining({ text: "prueba" })],
    });
    expect(found[1].kind).toBe("para");
  });

  it("leaves a real heading alone", () => {
    const found = titled([{ kind: "heading", level: 2, runs: [{ text: "ya" }] }]);

    expect(found[0]).toEqual({ kind: "heading", level: 2, runs: [{ text: "ya" }] });
  });

  it("does not promote an image that opens the document", () => {
    const found = titled([{ kind: "image", src: "a.png" }]);

    expect(found[0].kind).toBe("image");
  });

  it("copes with an empty document", () => {
    expect(titled([])).toEqual([]);
  });
});
