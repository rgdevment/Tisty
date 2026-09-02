import { Editor } from "@tiptap/core";
import { MarkdownSerializerState } from "prosemirror-markdown";
import { describe, expect, it } from "vitest";
import { asMarkdown, unaided, written } from "../ui/writing";

const build = (content: string) => new Editor({ extensions: written(), content });

const both = (content: string) => {
  const editor = build(content);
  const buffered = asMarkdown(editor) ?? "";
  const plain = unaided(() => asMarkdown(editor)) ?? "";
  editor.destroy();
  return { buffered, plain };
};

const same = (content: string) => {
  const { buffered, plain } = both(content);
  expect(buffered).toBe(plain);
  return buffered;
};

const LINE = "Revisar el informe trimestral antes del cierre de mes y anotar los pendientes.";

const mixed = (blocks: number) =>
  Array.from({ length: blocks }, (_, i) => {
    const turn = i % 12;
    if (turn === 0) return `## Seccion ${i}`;
    if (turn === 3) return `- ${LINE}\n- ${LINE}\n- ${LINE}`;
    if (turn === 5) return `> ${LINE}`;
    if (turn === 7) return `${LINE} con **negrita** y *cursiva* y \`codigo\`.`;
    if (turn === 9) return `${LINE} y un [enlace](https://example.com/a/${i}).`;
    return `${LINE} ${LINE}`;
  }).join("\n\n");

const flat = (blocks: number) =>
  Array.from({ length: blocks }, (_, i) => `${LINE} ${i}`).join("\n\n");

const shapes: Record<string, string> = {
  "spaces stuck inside the marks": "un **texto ** con *espacios * raros",
  "bold and italic at once": "***todo***",
  "every mark in one line":
    "un **texto ** con *espacios * raros y ![img](a.png) y ***todo*** y `x`",
  "a table with marks in its cells": "| a | b |\n| --- | --- |\n| **x** | y |\n| z | *w* |",
  "a table cell carrying a pipe": "| a \\| b | c |\n| --- | --- |\n| **x \\| y** | z |",
  "nested task lists": "- [ ] uno **dos**\n- [x] tres\n  - [ ] anidada\n    - [x] mas honda",
  "a quote holding a list and a fence":
    "> cita **fuerte**\n>\n> 1. uno\n> 2. dos\n>\n> ```js\n> const a = 1;\n> ```",
  "a picture right behind an exclamation": "texto ! ![alt](x.png) y !![](y.png) y \\!![z](z.png)",
  "a link glued to an exclamation": '<p>hola!<a href="https://x.dev/a">ver</a></p>',
  "a link glued to an escaped exclamation": '<p>hola\\!<a href="https://x.dev/a">ver</a></p>',
  "a link glued to an exclamation that opens the document":
    '<p>!<a href="https://x.dev/a">ver</a></p>',
  "a link glued to an exclamation of its own":
    '<p>a<strong>x</strong>!<a href="https://x.dev/a">ver</a></p>',
  "links into this Tisty": "ver [titulo](tisty:doc/abc) y [[algo]] fin",
  "front matter above the body": "---\ntitle: cosa\ntags: [a, b]\n---\n\ncuerpo **fuerte**",
  "windows backslashes": "abre C:\\Users\\Mario\\clip.mkv y [x](<C:\\a\\b.png>) y `D:\\raiz\\x`",
  "blank lines at both ends": "\n\n\ntexto en medio\n\n\n",
  "nothing but blank lines": "\n\n\n",
  "an empty document": "",
  "every heading depth": "# uno\n## dos\n### tres\n#### cuatro\n##### cinco\n###### seis",
  "characters that need escaping": "un \\* asterisco, un _guion_bajo_ y un ~~tachado~~",
  "a rule and a hard break": "uno\n\n---\n\ndos  \ntres",
  "a fenced block next to prose": "```ts\nconst a = 1;\n```\n\ntexto",
  "quotes inside lists inside quotes": "> - uno\n>   - dos\n>     - > tres **fuerte**",
  "marks with no room between them": "**a** **b**_c_`d`~~e~~ y * suelto *",
  "marks around nothing": "**** y ** ** y * *",
  "a link wearing a mark": "**[titulo](https://x.dev/a)** y *[b](tisty:doc/z)*",
  "raw html the editor keeps": "un <u>subrayado</u> y <br> suelto",
  "accents and emoji": "ñandú 🌵 con **acentós** y *cursivá*",
};

const PIECES = [
  "**",
  "*",
  "`",
  "~~",
  "!",
  "[",
  "]",
  "(",
  ")",
  "\\",
  " ",
  "\n",
  "\n\n",
  "a",
  "ñ",
  "#",
  "-",
  ">",
  "|",
  "_",
  "1.",
];

const rolled = (seed: number) => {
  let at = seed;
  return () => {
    at = (at * 1103515245 + 12345) & 0x7fffffff;
    return at / 0x7fffffff;
  };
};

const scrambled = (seed: number, pieces: number) => {
  const roll = rolled(seed);
  let text = "";
  for (let i = 0; i < pieces; i++) text += PIECES[Math.floor(roll() * PIECES.length)];
  return text;
};

type Loose = Record<string, unknown>;

const glimpsed = (): Loose => {
  const proto = MarkdownSerializerState.prototype as unknown as Loose;
  const esc = proto.esc as (...args: unknown[]) => unknown;
  let caught: Loose | null = null;
  proto.esc = function (this: Loose, ...args: unknown[]) {
    caught ??= this;
    return esc.apply(this, args);
  };
  const editor = build("hola **mundo ** y *algo*");
  asMarkdown(editor);
  editor.destroy();
  proto.esc = esc;
  if (!caught) throw new Error("no serializer state reached esc()");
  return caught;
};

describe("converting a document the editor cannot keep", () => {
  it("leaves something it will not ask about again", async () => {
    const { frail } = await import("../frail");
    const brings = [
      "# Compras\n\n<div>algo</div>",
      "# Compras\n\n<details>\n<summary>ver</summary>\nx\n</details>",
      "---\ntitle: x\n---\n\n# Compras",
      "una nota[^1]\n\n[^1]: el pie",
      "una nota[^1] y otra[^2]\n\n[^1]: uno\n\n[^2]: dos",
      "tres[^a] notas[^b] aqui[^c]\n\n[^a]: uno\n\n[^b]: dos\n\n[^c]: tres",
      "mira [esto][uno]\n\n[uno]: https://x.dev",
      "<div><ul><li>uno</li><li>dos<ul><li>anidado</li></ul></li></ul></div>",
      'mira este video <video src="clip.mp4"></video> antes de seguir',
      '<figure><img src="a.png"><figcaption>pie</figcaption></figure>',
    ];

    for (const one of brings) {
      expect(frail(one).length, one).toBeGreaterThan(0);
      const editor = new Editor({ extensions: written(), content: one });
      const after = asMarkdown(editor) ?? "";
      editor.destroy();

      expect(frail(after), `converting left it frail: ${JSON.stringify(after)}`).toEqual([]);
    }
  });
});

describe("a footnote whose definition is one word becomes a link, not a broken note", () => {
  const twoNotes = "una nota[^1] y otra[^2]\n\n[^1]: primera\n\n[^2]: segunda";

  it("turns into plain links, because that is what markdown says they are", async () => {
    const editor = build(twoNotes);
    const after = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(after).toBe("una nota[^1](primera) y otra[^2](segunda)");
  });

  it("stops being called frail once it is one, so the warning closes", async () => {
    const { frail } = await import("../frail");
    const editor = build(twoNotes);
    const after = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(frail(twoNotes)).toContain("frailNotes");
    expect(frail(after)).toEqual([]);
  });

  it("settles, so converting it again changes nothing", async () => {
    const editor = build(twoNotes);
    const first = asMarkdown(editor) ?? "";
    editor.destroy();

    const again = build(first);
    const second = asMarkdown(again) ?? "";
    again.destroy();

    expect(second).toBe(first);
  });
});

describe("what comes out of the editor goes back in unchanged", () => {
  const settled = (content: string) => {
    const first = build(content);
    const once = asMarkdown(first) ?? "";
    first.destroy();
    const again = build(once);
    const twice = asMarkdown(again) ?? "";
    again.destroy();
    return { once, twice };
  };

  const DRIFTS = ["front matter above the body"];

  it.each(Object.entries(shapes).filter(([name]) => !DRIFTS.includes(name)))(
    "settles after one pass on %s",
    (_name, content) => {
      const { once, twice } = settled(content);
      expect(twice).toBe(once);
    },
  );

  it("settles on a mixed document of any length, which is what a merge needs", () => {
    for (const blocks of [1, 5, 40, 200]) {
      const { once, twice } = settled(mixed(blocks));
      expect(twice, `${blocks} bloques`).toBe(once);
    }
  });

  it("front matter drifts, and it is the one frail already refuses", async () => {
    const { frail } = await import("../frail");
    const { once, twice } = settled(shapes["front matter above the body"]);

    expect(twice).not.toBe(once);
    expect(frail(shapes["front matter above the body"])).toContain("frailFront");
  });

  it("settles after two passes rather than one, everywhere it drifts at all", () => {
    for (const name of DRIFTS) {
      const { twice } = settled(shapes[name]);
      const again = build(twice);
      const thrice = asMarkdown(again) ?? "";
      again.destroy();
      expect(thrice, name).toBe(twice);
    }
  });

  it("keeps a literal backslash before a link from turning that link into an image", () => {
    const { once, twice } = settled(shapes["a link glued to an escaped exclamation"]);

    expect(twice).toBe(once);
    expect(twice).toMatch(/\\!\[ver\]/);
    expect(twice).not.toMatch(/[^\\]!\[ver\]/);
  });
});

describe("a table with cells nobody filled in", () => {
  it("keeps every empty cell exactly where it was, not just the ones with text", () => {
    const table = "|  |  |\n| --- | --- |\n|  |  |\n";
    const editor = build(table);
    const out = asMarkdown(editor) ?? "";
    editor.destroy();

    expect(out).toBe(table);
  });
});

describe("the buffered serializer writes what the plain one writes", () => {
  it("really lifts the buffer off while it compares, or it would be grading its own work", () => {
    const proto = MarkdownSerializerState.prototype;
    expect(typeof Object.getOwnPropertyDescriptor(proto, "out")?.get).toBe("function");
    unaided(() => {
      expect(Object.getOwnPropertyDescriptor(proto, "out")).toBeUndefined();
    });
    expect(typeof Object.getOwnPropertyDescriptor(proto, "out")?.get).toBe("function");
  });

  it.each([10, 100, 400])("agrees on a mixed document of %i blocks", (blocks) => {
    expect(same(mixed(blocks)).length).toBeGreaterThan(0);
  });

  it("agrees on plain prose with no marks at all", () => {
    expect(same(flat(120))).not.toContain("*");
  });

  const REPAIRED = ["a link glued to an escaped exclamation"];

  it.each(Object.entries(shapes).filter(([name]) => !REPAIRED.includes(name)))(
    "agrees on %s",
    (_name, content) => {
      same(content);
    },
  );

  it("parts from the plain one only where the plain one corrupts, and the repair is the point", () => {
    const content = shapes["a link glued to an escaped exclamation"];
    const { buffered, plain } = both(content);

    expect(plain).toBe("hola\\\\![ver](https://x.dev/a)");
    expect(buffered).toBe("hola\\\\\\![ver](https://x.dev/a)");

    const settle = (said: string) => {
      const editor = build(said);
      const out = asMarkdown(editor) ?? "";
      editor.destroy();
      return out;
    };

    expect(settle(plain)).not.toBe(plain);
    expect(settle(buffered)).toBe(buffered);
  });

  it("turns a link into an image when the plain serializer is left to it, which is the bug", () => {
    const editor = build(shapes["a link glued to an escaped exclamation"]);
    const plain = unaided(() => asMarkdown(editor)) ?? "";
    editor.destroy();

    const again = build(plain);
    const twice = asMarkdown(again) ?? "";
    again.destroy();

    expect(twice).toMatch(/!\[ver\]/);
  });

  it("still reaches the escape a link glued to an exclamation needs, so those cases are not idle", () => {
    expect(same('<p>hola!<a href="https://x.dev/a">ver</a></p>')).toBe(
      "hola\\![ver](https://x.dev/a)",
    );
    expect(both('<p>hola\\!<a href="https://x.dev/a">ver</a></p>').buffered).toBe(
      "hola\\\\\\![ver](https://x.dev/a)",
    );
    expect(same('<p>a<strong>x</strong>!<a href="https://x.dev/a">ver</a></p>')).toBe(
      "a**x**\\![ver](https://x.dev/a)",
    );
  });

  it("agrees on every length as a shape grows, where the buffer boundaries move", () => {
    for (let n = 1; n <= 24; n++) {
      same(
        Array.from(
          { length: n },
          (_, i) => `parrafo ${i} con **negrita ** y *cursiva * pegadas`,
        ).join("\n\n"),
      );
      same(
        Array.from({ length: n }, (_, i) => `- item ${i} con ![i](a${i}.png) tras !`).join("\n"),
      );
      same(
        Array.from(
          { length: n },
          (_, i) => `> cita ${i} con \`codigo\` y [e](tisty:doc/${i})`,
        ).join("\n\n"),
      );
    }
  });

  it("agrees on scrambled markup that no sane person would write", () => {
    for (let seed = 1; seed <= 250; seed++) same(scrambled(seed, 60));
  });

  it("keeps agreeing when tiptap-markdown's own state stays untouched, the shape an upgrade would leave", () => {
    const sub = Object.getPrototypeOf(glimpsed()) as Loose;
    const library: Loose = {};
    unaided(() => {
      const plainSub = Object.getPrototypeOf(glimpsed()) as Loose;
      library.markString = plainSub.markString;
      library.normalizeInline = plainSub.normalizeInline;
    });
    const ours = { markString: sub.markString, normalizeInline: sub.normalizeInline };
    Object.assign(sub, library);
    try {
      const held = Object.entries(shapes)
        .filter(([name]) => !REPAIRED.includes(name))
        .map(([, content]) => content);
      for (const content of [mixed(40), ...held]) {
        const editor = build(content);
        const half = asMarkdown(editor) ?? "";
        const plain = unaided(() => asMarkdown(editor)) ?? "";
        editor.destroy();
        expect(half).toBe(plain);
      }
    } finally {
      Object.assign(sub, ours);
    }
  });
});
