import { describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";

const make = (content = "") => new Editor({ extensions: written(), content });

const md = (e: Editor) => asMarkdown(e) ?? "";

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

describe("a pipe typed inside a cell", () => {
  const barred =
    "<table><tbody><tr><th>a</th><th>b</th></tr><tr><td>x|y</td><td>2</td></tr></tbody></table>";

  it("stays in its own cell instead of starting a new column", () => {
    const editor = make(through(barred));
    const cells = editor.getText().split("\n").filter(Boolean);
    editor.destroy();

    expect(cells).toContain("x|y");
    expect(cells).toContain("2");
  });

  it("does not take the rest of the row with it on a second save", () => {
    const once = through(barred);
    const twice = through(once);

    expect(twice).toBe(once);
  });

  it("leaves formatting inside the same cell alone", () => {
    const editor = make(
      through(
        "<table><tbody><tr><th>a</th></tr><tr><td>x|y con <em>énfasis</em></td></tr></tbody></table>",
      ),
    );
    const back = editor.getHTML();
    editor.destroy();

    expect(back).toContain("<em>énfasis</em>");
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
