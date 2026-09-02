import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";
import { leaning } from "../ui/Tabled";
import { asMarkdown, written } from "../ui/writing";

const built = (content: string) => new Editor({ extensions: written(), content });

const inCell = (editor: Editor, row: number, column: number) => {
  let at = -1;
  let rows = 0;
  editor.state.doc.descendants((node, pos) => {
    if (node.type.name !== "tableRow") return true;
    if (rows === row) {
      let spot = pos + 1;
      node.forEach((cell, _offset, index) => {
        if (index === column) at = spot + 2;
        spot += cell.nodeSize;
      });
    }
    rows += 1;
    return false;
  });
  editor.commands.setTextSelection(at);
};

describe("leaning a column from the table bar", () => {
  const plain = "| a | b |\n| --- | --- |\n| 1 | 2 |";

  it("writes the alignment into the delimiter row, for the whole column", () => {
    const editor = built(plain);
    inCell(editor, 0, 1);
    leaning(editor, "right");
    const out = asMarkdown(editor) ?? "";
    editor.destroy();
    expect(out).toBe("| a | b |\n| --- | ---: |\n| 1 | 2 |\n");
  });

  it("leans the column even when the cursor sits in a body row", () => {
    const editor = built(plain);
    inCell(editor, 1, 0);
    leaning(editor, "center");
    const out = asMarkdown(editor) ?? "";
    editor.destroy();
    expect(out).toBe("| a | b |\n| :---: | --- |\n| 1 | 2 |\n");
  });

  it("takes the lean away again", () => {
    const editor = built("| a | b |\n| --- | ---: |\n| 1 | 2 |");
    inCell(editor, 0, 1);
    leaning(editor, null);
    const out = asMarkdown(editor) ?? "";
    editor.destroy();
    expect(out).toBe("| a | b |\n| --- | --- |\n| 1 | 2 |\n");
  });

  it("does nothing outside a table", () => {
    const editor = built("solo texto");
    editor.commands.setTextSelection(2);
    expect(leaning(editor, "right")).toBe(false);
    editor.destroy();
  });
});

describe("leaning a column of a table with merged cells", () => {
  const merged =
    "<table><tbody>" +
    "<tr><th>a</th><th>b</th><th>c</th></tr>" +
    "<tr><td rowspan='2'>x</td><td>y</td><td>z</td></tr>" +
    "<tr><td>q</td><td>r</td></tr>" +
    "</tbody></table>";

  const leans = (editor: Editor) => {
    const rows: (string | null)[][] = [];
    editor.state.doc.descendants((node) => {
      if (node.type.name !== "tableRow") return true;
      const held: (string | null)[] = [];
      node.forEach((cell) => {
        held.push((cell.attrs.textAlign as string) ?? null);
      });
      rows.push(held);
      return false;
    });
    return rows;
  };

  const cellAt = (editor: Editor, row: number, column: number) => {
    let at = -1;
    let rows = 0;
    editor.state.doc.descendants((node, pos) => {
      if (node.type.name !== "tableRow") return true;
      if (rows === row) {
        let spot = pos + 1;
        node.forEach((cell, _offset, index) => {
          if (index === column) at = spot + 2;
          spot += cell.nodeSize;
        });
      }
      rows += 1;
      return false;
    });
    editor.commands.setTextSelection(at);
  };

  it("counts the column past a cell that spans two rows", () => {
    const editor = built(merged);
    cellAt(editor, 0, 1);
    leaning(editor, "center");
    const held = leans(editor);
    editor.destroy();
    expect(held).toEqual([
      [null, "center", null],
      [null, "center", null],
      ["center", null],
    ]);
  });

  it("leans the column the spanning cell holds, once", () => {
    const editor = built(merged);
    cellAt(editor, 1, 0);
    leaning(editor, "right");
    const held = leans(editor);
    editor.destroy();
    expect(held).toEqual([
      ["right", null, null],
      ["right", null, null],
      [null, null],
    ]);
  });
});
