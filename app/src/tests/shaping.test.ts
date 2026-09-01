import { generateJSON } from "@tiptap/core";
import { describe, expect, it } from "vitest";
import { composed } from "../markdown";
import { type Shape, SIZES } from "../ui/paper";
import { asData, fetched, shapesOf, titled } from "../ui/shaping";
import { written } from "../ui/writing";

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

    expect(found).toEqual([{ kind: "image", src: "a.png", alt: "" }]);
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

  it("gives the tabloid sheet a wider page than either", () => {
    expect(SIZES.tabloid).toEqual([792, 1224]);
    expect(SIZES.tabloid[0]).toBeGreaterThan(SIZES.letter[0]);
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

describe("a table on the page", () => {
  const cell = (text: string) => ({
    type: "tableCell",
    content: [{ type: "paragraph", content: [{ type: "text", text }] }],
  });
  const row = (...texts: string[]) => ({ type: "tableRow", content: texts.map(cell) });

  it("keeps its rows and columns instead of running them together", () => {
    const found = shapesOf(
      doc(
        { type: "paragraph", content: words("t") },
        {
          type: "table",
          content: [row("Papel", "Ancho"), row("A4", "210 mm")],
        },
      ),
    );
    const table = found.find((one) => one.kind === "table");

    expect(table?.kind === "table" && table.rows.length).toBe(2);
    expect(table?.kind === "table" && table.rows[1].map((c) => c[0].text)).toEqual([
      "A4",
      "210 mm",
    ]);
  });

  it("leaves an empty table out", () => {
    expect(shapesOf(doc({ type: "table", content: [] }))).toEqual([]);
  });
});

describe("carrying an attachment into the PDF", () => {
  it("turns bytes into something the page can draw", () => {
    const src = asData([137, 80, 78, 71], "attachments/a.png");

    expect(src.startsWith("data:image/png;base64,")).toBe(true);
  });

  it("reads the kind from the name, not from the bytes", () => {
    expect(asData([1], "a.jpg")).toContain("image/jpeg");
    expect(asData([1], "a.svg")).toContain("image/svg+xml");
    expect(asData([1], "a.what")).toContain("image/png");
  });

  it("swaps every local image for its data", async () => {
    const shapes = await fetched(
      [
        { kind: "image", src: "attachments/a.png", alt: "una" },
        { kind: "para", runs: [{ text: "x" }] },
      ],
      async () => [137, 80, 78, 71, 13, 10],
    );

    expect(shapes[0].kind === "image" && shapes[0].src.startsWith("data:")).toBe(true);
    expect(shapes[1].kind).toBe("para");
  });

  it("reads a repeated attachment once", async () => {
    let asked = 0;
    await fetched(
      [
        { kind: "image", src: "attachments/a.png" },
        { kind: "image", src: "attachments/a.png" },
      ],
      async () => {
        asked += 1;
        return [137, 80, 78, 71];
      },
    );

    expect(asked).toBe(1);
  });

  it("leaves a remote image alone", async () => {
    const shapes = await fetched([{ kind: "image", src: "https://a.b/c.png" }], async () => [1]);

    expect(shapes[0].kind === "image" && shapes[0].src).toBe("https://a.b/c.png");
  });

  it("keeps the name as a reference when the file cannot be read", async () => {
    const shapes = await fetched(
      [{ kind: "image", src: "attachments/gone.png", alt: "perdida" }],
      async () => {
        throw new Error("no");
      },
    );

    expect(shapes[0].kind === "image" && shapes[0].src).toBe("");
    expect(shapes[0].kind === "image" && shapes[0].alt).toBe("perdida");
  });
});

describe("what the printed page can and cannot draw", () => {
  it("turns a pdf attachment into a named card instead of a broken picture", async () => {
    const shapes: Shape[] = [{ kind: "image", src: "attachments/ab/informe-a1b2.pdf", alt: "" }];

    const out = await fetched(shapes, () => Promise.resolve([1, 2, 3]));

    expect(out[0]).toEqual({ kind: "file", name: "informe-a1b2.pdf", said: "PDF" });
  });

  it("keeps the label the writer gave the file", async () => {
    const shapes: Shape[] = [
      { kind: "image", src: "attachments/ef/minuta-e5f6.docx", alt: "Minuta del lunes" },
    ];

    const out = await fetched(shapes, () => Promise.resolve([1]));

    expect(out[0]).toEqual({ kind: "file", name: "Minuta del lunes", said: "Word" });
  });

  it("prints a document card as a document, not as a file with a strange type", async () => {
    const shapes: Shape[] = [
      { kind: "image", src: "tisty:doc/65w7xrqp-0001", alt: "Cómo funciona Tisty" },
    ];

    const out = await fetched(shapes, () => Promise.resolve([1]));

    expect(out[0]).toEqual({ kind: "file", name: "Cómo funciona Tisty", said: "Document" });
  });

  it("refuses to draw a file that only pretends to be a picture", async () => {
    const shapes: Shape[] = [{ kind: "image", src: "attachments/ab/roto-a1b2.png", alt: "roto" }];

    const out = await fetched(shapes, () => Promise.resolve([37, 80, 68, 70]));

    expect(out[0]).toEqual({ kind: "file", name: "roto", said: "Image" });
  });

  it("still embeds a png", async () => {
    const shapes: Shape[] = [{ kind: "image", src: "attachments/ab/foto-a1b2.png", alt: "foto" }];

    const out = await fetched(shapes, () => Promise.resolve([137, 80, 78, 71]));

    expect((out[0] as { src: string }).src.startsWith("data:image/png;base64,")).toBe(true);
  });
});

describe("a page that is not the one being edited", () => {
  it("reads off disk into the same shapes the editor would have given", () => {
    const body =
      "# Marzo\n\nlo que se **dijo**.\n\n- uno\n- dos\n\n![plano](<attachments/ab/plano-a1b2.png>)";

    const found = shapesOf(generateJSON(composed(body), written()));

    expect(found.map((one) => one.kind)).toEqual(["heading", "para", "bullet", "bullet", "image"]);
    expect((found[0] as { runs: { text: string }[] }).runs[0].text).toBe("Marzo");
  });

  it("keeps a table and a quote, which is what a book of minutes is made of", () => {
    const body = "> lo dijo el comité\n\n| a | b |\n| - | - |\n| 1 | 2 |\n";

    const found = shapesOf(generateJSON(composed(body), written()));

    expect(found.map((one) => one.kind)).toEqual(["quote", "table"]);
  });
});
