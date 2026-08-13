import { describe, expect, it, vi } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { Image } from "@tiptap/extension-image";
import { Markdown } from "tiptap-markdown";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const Pictured = Image.extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: { write: (text: string) => void; closeBlock: (node: unknown) => void },
          node: { attrs: Record<string, string> },
        ) {
          state.write(`![${node.attrs.alt ?? ""}](${node.attrs.src ?? ""})`);
          state.closeBlock(node);
        },
        parse: {},
      },
    };
  },
});

const make = (content = "") =>
  new Editor({
    extensions: [
      StarterKit,
      Pictured,
      Table.configure({ resizable: false }),
      TableRow,
      TableHeader,
      TableCell,
      Markdown.configure({ html: true, linkify: true, breaks: true, transformPastedText: false }),
    ],
    content,
  });

const md = (e: Editor) =>
  (e.storage as unknown as { markdown: { getMarkdown: () => string } }).markdown.getMarkdown();

const through = (text: string) => {
  const editor = make(text);
  const out = md(editor);
  editor.destroy();
  return out;
};

describe("a picture with something after it", () => {
  it("does not swallow the block that follows", () => {
    const saved = through("![x](a.png)\n\n## Después\n\ncuerpo");

    expect(saved).toContain("![x](a.png)\n\n## Después");
  });

  it("leaves a table after a picture still a table when reopened", () => {
    const saved = through("![x](a.png)\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n");
    const editor = make(saved);
    const back = editor.getHTML();
    editor.destroy();

    expect(back).toContain("<table");
    expect(back).toContain("<img");
  });
});

describe("a slash inside code", () => {
  it("is a path or a division, and the editor can tell", () => {
    const editor = make("```\ncd /\n```");
    editor.commands.setTextSelection(editor.state.doc.content.size - 1);

    expect(editor.isActive("codeBlock")).toBe(true);

    editor.destroy();
  });

  it("is a command anywhere else", () => {
    const editor = make("una nota /");
    editor.commands.setTextSelection(editor.state.doc.content.size - 1);

    expect(editor.isActive("codeBlock")).toBe(false);
    expect(editor.isActive("code")).toBe(false);

    editor.destroy();
  });
});

describe("underline, which markdown has no syntax for", () => {
  it("survives being saved and reopened", () => {
    const editor = make("hola mundo");
    editor.commands.selectAll();
    editor.commands.toggleUnderline();

    const saved = md(editor);
    editor.destroy();

    expect(saved).toBe("<u>hola mundo</u>");
    expect(through(saved)).toBe("<u>hola mundo</u>");
  });

  it("comes back as a real mark, not as letters on the page", () => {
    const editor = make("<u>subrayado</u>");

    expect(editor.getHTML()).toContain("<u>subrayado</u>");

    editor.destroy();
  });
});

describe("a table markdown has no syntax for", () => {
  const joined =
    '<table><tbody><tr><th colspan="2">wide</th></tr><tr><td>1</td><td>2</td></tr></tbody></table>';

  it("keeps its cells instead of collapsing to the word table", () => {
    const saved = through(joined);

    expect(saved).not.toContain("[table]");
    expect(saved).toContain("wide");
    expect(saved).toContain("<table");
  });

  it("is still a table after being saved and reopened", () => {
    const editor = make(through(joined));
    const back = editor.getHTML();
    editor.destroy();

    expect(back).toContain("<table");
    expect(back).toContain("wide");
  });
});

describe("text that only looks like markup", () => {
  it("reads back as the characters a person typed, not as formatting", () => {
    const editor = make(through("2 * 3 * 4 and a_b_c"));

    expect(editor.getText()).toBe("2 * 3 * 4 and a_b_c");

    editor.destroy();
  });
});

describe("html a person wrote in their own file", () => {
  it("is no longer escaped into visible entities", () => {
    expect(through("Text with <br/> break")).not.toContain("&lt;");
  });
});
