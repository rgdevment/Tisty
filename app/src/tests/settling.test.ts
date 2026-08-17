import { describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import { asMarkdown, written } from "../ui/writing";

const settled = (body: string) => (body === "" || body.endsWith("\n") ? body : `${body}\n`);

const roundtripped = (content: string) => {
  const editor = new Editor({ extensions: written(), content });
  const out = asMarkdown(editor) ?? "";
  editor.destroy();
  return settled(out);
};

const BLOCKS = [
  "# Un titulo",
  "## Otro titulo",
  "un parrafo cualquiera",
  "un parrafo con **negrita** y *cursiva* y `codigo`",
  "una linea\\\ny otra tras un salto duro",
  "- uno\n- dos\n- tres",
  "1. uno\n2. dos",
  "- [ ] pendiente\n- [x] hecha",
  "> una cita",
  "| a | b |\n| --- | --- |\n| uno | dos |",
  "```\ncodigo suelto\n```",
  "```rust\nfn main() {}\n```",
  "---",
  "![el contrato](attachments/ab/cd.pdf)",
  "![una foto](attachments/aa/1.png)",
  "mira [el informe](tisty:doc/mac0-0007) aqui",
  "texto con acentós y ñandú y 日本語 y 🎉",
  "un parrafo con [enlace](https://ejemplo.org)",
];

const joined = (parts: string[]) => `${parts.join("\n\n")}\n`;

describe("what a merge hands back has to survive the editor untouched", () => {
  it("measures the whole road, editor and store, not just the editor", () => {
    expect(settled("x")).toBe("x\n");
    expect(settled("x\n")).toBe("x\n");
    expect(settled("")).toBe("");
  });

  it("settles every block on its own", () => {
    const drifted = BLOCKS.filter((one) => roundtripped(joined([one])) !== joined([one]));

    expect(drifted).toEqual([]);
  });

  const lists = (block: string) => /^\s*(?:[-*+]\s|\d+[.)]\s)/.test(block);

  it("settles every pair, which is where a seam could bite", () => {
    const drifted: string[] = [];
    for (const first of BLOCKS) {
      for (const second of BLOCKS) {
        if (lists(first) && lists(second)) continue;
        const said = joined([first, second]);
        if (roundtripped(said) !== said) {
          drifted.push(`${JSON.stringify(said)} -> ${JSON.stringify(roundtripped(said))}`);
        }
      }
    }

    expect(drifted).toEqual([]);
  });

  it("skips only what the engine refuses to write, and nothing else", () => {
    const skipped = BLOCKS.flatMap((first) =>
      BLOCKS.filter((second) => lists(first) && lists(second)).map(
        (second) => `${first} | ${second}`,
      ),
    );

    expect(skipped).toHaveLength(9);
    expect(skipped.every((one) => lists(one.split(" | ")[0]))).toBe(true);
  });

  const APART = BLOCKS.filter((one, at) => !(lists(one) && at > 0 && lists(BLOCKS[at - 1])));

  it("settles a document made of all of them at once", () => {
    const said = joined(APART);

    expect(roundtripped(said)).toBe(said);
  });

  it("settles on the second pass too, so nothing creeps", () => {
    const said = joined(APART);
    const once = roundtripped(said);

    expect(roundtripped(once)).toBe(once);
  });

  const TRICKY = [
    "# Un titulo",
    "un parrafo cualquiera",
    "una linea\\\ny otra tras un salto duro",
    "- uno\n- dos\n- tres",
    "- [ ] pendiente\n- [x] hecha",
    "> una cita",
    "| a | b |\n| --- | --- |\n| uno | dos |",
    "```\ncodigo suelto\n```",
    "---",
    "![el contrato](attachments/ab/cd.pdf)",
  ];

  it("settles every three in a row, where a seam meets a seam", { timeout: 120_000 }, () => {
    const drifted: string[] = [];
    for (const first of TRICKY) {
      for (const second of TRICKY) {
        for (const third of TRICKY) {
          if ((lists(first) && lists(second)) || (lists(second) && lists(third))) continue;
          const said = joined([first, second, third]);
          if (roundtripped(said) !== said) drifted.push(said);
        }
      }
    }

    expect(drifted).toEqual([]);
  });
});
