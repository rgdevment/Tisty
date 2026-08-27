import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";
import { asMarkdown, written } from "../ui/writing";

const made = (content = "<p></p>") => new Editor({ extensions: written(), content });

/// What the clipboard hands over is text; the editor has to read it as Markdown or a pasted
/// document arrives as one long line of hashes and asterisks.
const pasted = (editor: Editor, text: string) => {
  const parser = editor.view.someProp("clipboardTextParser");
  if (!parser) throw new Error("nothing parses pasted text");
  const at = editor.state.selection.$from;
  return parser(text, at, false, editor.view);
};

describe("pasting Markdown into a document", () => {
  it("reads a heading as a heading, not as a line starting with a hash", () => {
    const editor = made();
    const slice = pasted(editor, "# Análisis del repositorio\n\nUn párrafo.");
    editor.view.dispatch(editor.state.tr.replaceSelection(slice));
    const out = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(out).toContain("# Análisis del repositorio");
    expect(out).not.toContain(String.raw`\#`);
    expect(out).toContain("Un párrafo.");
  });

  it("reads a list, a quote and a fence as themselves", () => {
    const editor = made();
    const slice = pasted(editor, "> Una cita\n\n- uno\n- dos\n\n```ts\nconst a = 1;\n```");
    editor.view.dispatch(editor.state.tr.replaceSelection(slice));
    const out = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(out).toContain("> Una cita");
    expect(out).toMatch(/[-*] uno/);
    expect(out).toContain("const a = 1;");
  });

  it("reads a table, which is where a literal paste hurts most", () => {
    const editor = made();
    const slice = pasted(editor, "| # | Cambio |\n|---|--------|\n| 1 | Ampliar el tipo |");
    editor.view.dispatch(editor.state.tr.replaceSelection(slice));
    const out = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(out).toContain("Ampliar el tipo");
    expect(out).toContain("|");
  });

  it("leaves it literal where a paste asked to be left alone", () => {
    const editor = made();
    const at = editor.state.selection.$from;
    const parser = editor.view.someProp("clipboardTextParser");

    // What ProseMirror sets inside a code block, and for a paste held with shift.
    expect(parser?.("# not a heading here", at, true, editor.view)).toBeNull();
    editor.destroy();
  });
});

describe("pasting something that already carries its formatting", () => {
  const dropped = (html: string) => {
    const editor = made();
    editor.commands.insertContent(html);
    const out = asMarkdown(editor) ?? "";
    editor.destroy();
    return out;
  };

  it("keeps a heading, bold and a link that arrived as HTML", () => {
    const out = dropped(
      "<h2>Un título</h2><p>algo <strong>en negrita</strong> y " +
        '<a href="https://ejemplo.org">un enlace</a>.</p>',
    );

    expect(out).toContain("## Un título");
    expect(out).toContain("**en negrita**");
    expect(out).toContain("[un enlace](https://ejemplo.org)");
  });

  it("keeps a list and a table that arrived as HTML", () => {
    const out = dropped(
      "<ul><li>uno</li><li>dos</li></ul>" +
        "<table><thead><tr><th><p>Campo</p></th><th><p>Valor</p></th></tr></thead>" +
        "<tbody><tr><td><p>a</p></td><td><p>b</p></td></tr></tbody></table>",
    );

    expect(out).toMatch(/[-*] uno/);
    expect(out).toContain("| Campo | Valor |");
    expect(out).toContain("| a | b |");
  });

  it("takes the text branch only when no formatting came with it", () => {
    const editor = made();
    const at = editor.state.selection.$from;
    const parser = editor.view.someProp("clipboardTextParser");

    // ProseMirror hands text to this parser only when the clipboard carried no HTML;
    // rich content goes down the DOM branch instead and never reaches here.
    expect(parser?.("# heading", at, false, editor.view)).not.toBeNull();
    editor.destroy();
  });
});
