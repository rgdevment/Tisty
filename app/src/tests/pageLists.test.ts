import { describe, expect, it, vi } from "vitest";
import { pastedAround } from "../ui/pageBlocks";
import { linedAs } from "../ui/pageEdits";
import { addressed } from "../ui/pageFormats";
import {
  hardBreak,
  joinedApart,
  joinedAt,
  nested,
  openedAbove,
  outdented,
  unmarked,
} from "../ui/pageLists";
import { balanced, worn, wornEach } from "../ui/pageMarks";

const applied = (text: string, moved: ReturnType<typeof nested>) =>
  moved ? text.slice(0, moved.edit.from) + moved.edit.insert + text.slice(moved.edit.to) : text;

describe("list items moved in and out by their indent", () => {
  it.each([
    ["an item under a bullet", "- a\n- b\n", 6, false, "- a\n  - b\n"],
    ["an item under a number", "1. a\n2. b\n", 8, false, "1. a\n   2. b\n"],
    ["an item back to its parent", "- a\n  - b\n", 8, true, "- a\n- b\n"],
    ["an item inside a quote", "> - a\n> - b\n", 10, false, "> - a\n>   - b\n"],
    ["an item back out inside a quote", "> - a\n>   - b\n", 12, true, "> - a\n> - b\n"],
    ["a first item, which has nothing above", "- a\n", 2, false, "- a\n"],
    ["a line that is no item", "uno\n", 2, false, "uno\n"],
    [
      "an item one level down, past the deeper items above it",
      "- a\n  - b\n- c\n",
      12,
      false,
      "- a\n  - b\n  - c\n",
    ],
    [
      "an item under one in a quote written without its space",
      "> - a\n>- b\n",
      9,
      false,
      "> - a\n>  - b\n",
    ],
    [
      "an item back out under a quote written without its space",
      "> - a\n>  - b\n",
      11,
      true,
      "> - a\n> - b\n",
    ],
  ])("moves %s", (_, text, at, out, expected) => {
    expect(applied(text, nested(text, at, out))).toBe(expected);
  });

  it("reads a line of many quote marks and no item at once, not in a time that doubles with each mark", () => {
    const line = `>${" >".repeat(28)} x`;
    const started = performance.now();
    expect(nested(line, line.length, false)).toBeNull();
    expect(performance.now() - started).toBeLessThan(1000);
  });

  it("brings an empty nested item up a level, its caret at the line's end", () => {
    const moved = outdented("- a\n  - ", 8);
    expect(applied("- a\n  - ", moved)).toBe("- a\n- ");
    expect(moved?.caret).toBe(6);
  });

  it("joins an item asked past a list's blank line to the list, Windows line ends too", () => {
    expect(joinedAt("- a\n\n## B\n", 5, "- [ ] ")).toBe(4);
    expect(joinedAt("- a\r\n\r\n## B\r\n", 7, "- [ ] ")).toBe(5);
    expect(joinedAt("Uno\n\n## B\n", 5, "- [ ] ")).toBe(5);
    expect(joinedAt("- a\n\n## B\n", 5, "")).toBe(5);
  });
});

describe("words pasted that open with a block of their own", () => {
  const pasted = (text: string, at: number, said: string) =>
    text.slice(0, at) + pastedAround(text, at, at, said) + text.slice(at);

  it.each([
    [
      "a heading at a line's end starts under it",
      "Primer parrafo.",
      15,
      "## Titulo\n\nUn texto",
      "Primer parrafo.\n\n## Titulo\n\nUn texto",
    ],
    [
      "a heading mid-line splits the line around it",
      "Primer parrafo.",
      6,
      "### Sub\n\ncola\n",
      "Primer\n\n### Sub\n\ncola\n\n parrafo.",
    ],
    ["a list after an item joins its list", "- tres", 6, "- p1\n- p2", "- tres\n- p1\n- p2"],
    ["plain words stay in the line", "uno dos", 3, "hola", "unohola dos"],
    ["a heading on an empty line is left as it is", "", 0, "## Titulo", "## Titulo"],
    [
      "paragraphs pasted in a note stay in it",
      "> consejo",
      9,
      "AAA\n\nBBB",
      "> consejoAAA\n> \n> BBB",
    ],
  ])("%s", (_, text, at, said, expected) => {
    expect(pasted(text, at, said)).toBe(expected);
  });
});

describe("a format over words chosen across blocks", () => {
  const wrapped = (text: string, chosen: string, open: string, close = open) => {
    const from = text.indexOf(chosen);
    const edit = wornEach(text, { from, to: from + chosen.length }, open, close);
    return text.slice(0, edit.from) + edit.insert + text.slice(edit.to);
  };

  it.each([
    [
      "two paragraphs",
      "ocho nueve diez\n\nonce doce trece",
      "nueve diez\n\nonce doce",
      "**",
      "ocho **nueve diez**\n\n**once doce** trece",
    ],
    [
      "two items, past their bullets",
      "- item con palabra\n- otro item",
      "palabra\n- otro",
      "`",
      "- item con `palabra`\n- `otro` item",
    ],
    [
      "a fold's opening line, left as it is",
      "Antes y más\n\n:::details Plegable\n\nContenido",
      "y más\n\n:::details Plegable\n\nConte",
      "**",
      "Antes **y más**\n\n:::details Plegable\n\n**Conte**nido",
    ],
    [
      "a fold's closing line, left as it is",
      "Otro\n\n:::\n\nDespués",
      "ro\n\n:::\n\nDes",
      "**",
      "Ot**ro**\n\n:::\n\n**Des**pués",
    ],
    [
      "a fold inside a quote, left as it is",
      "> Antes\n>\n> :::details Plegable\n>\n> Contenido",
      "Antes\n>\n> :::details Plegable\n>\n> Conte",
      "*",
      "> *Antes*\n>\n> :::details Plegable\n>\n> *Conte*nido",
    ],
    [
      "a heading and a list",
      "## Titulo con palabra\n\n- item con palabra",
      "Titulo con palabra\n\n- item",
      "*",
      "## *Titulo con palabra*\n\n- *item* con palabra",
    ],
    [
      "Windows line ends",
      "uno dos\r\n\r\ntres cuatro",
      "dos\r\n\r\ntres",
      "~~",
      "uno ~~dos~~\r\n\r\n~~tres~~ cuatro",
    ],
  ])("puts a pair in each block of %s", (_, text, chosen, open, expected) => {
    expect(wrapped(text, chosen, open)).toBe(expected);
  });
});

describe("Backspace and Enter at the start of a line's words", () => {
  it.each([
    ["a step loses its box first", "- [ ] paso a\n- [x] paso b", 6, "- paso a\n- [x] paso b", 2],
    ["then the bullet", "- paso a", 2, "paso a", 0],
    [
      "a bullet loses its bullet and stands apart from the items under it",
      "## Uno\n\n- uno\n- dos",
      10,
      "## Uno\n\nuno\n\n- dos",
      8,
    ],
    ["an item in a quote keeps the quote", "> - x", 4, "> x", 2],
    [
      "a step freed of its bullet stands apart from the steps around it",
      "- [ ] uno\n- dos\n- [ ] tres",
      12,
      "- [ ] uno\n\ndos\n\n- [ ] tres",
      11,
    ],
    [
      "a numbered item stands apart from the items around it",
      "1. uno\n2. dos\n3. tres",
      10,
      "1. uno\n\ndos\n\n3. tres",
      8,
    ],
    [
      "an item in a quote stands apart inside the quote",
      "> - uno\n> - dos",
      12,
      "> - uno\n>\n> dos",
      12,
    ],
    ["a nested item becomes a paragraph of the item above", "- a\n  - b", 8, "- a\n\n  b", 7],
    [
      "a numbered item nested deep stands apart from the numbered one beside it",
      "1. a\n    1. b\n    2. c",
      12,
      "1. a\n\n    b\n\n    2. c",
      10,
    ],
    [
      "a bullet over an item indented as deep as code raises that item, which code would not take",
      "- uno\n    - sub",
      2,
      "uno\n\n- sub",
      0,
    ],
  ])("on Backspace, %s", (_, text, at, expected, caret) => {
    const moved = unmarked(text, at);
    const applied = moved
      ? text.slice(0, moved.edit.from) + moved.edit.insert + text.slice(moved.edit.to)
      : text;
    expect(applied).toBe(expected);
    expect(moved?.caret).toBe(caret);
  });

  it.each([
    ["a heading", "## Sección", 6, "\n\n"],
    ["a heading inside a quote", "> ## Cita", 7, "\n>\n> "],
  ])("on a line break in %s, opens a paragraph as Enter does", (_, text, at, expected) => {
    expect(hardBreak(text, at)).toBe(expected);
  });

  it("leaves Backspace inside an item's words alone", () => {
    expect(unmarked("- uno", 3)).toBeNull();
  });

  const backspaced = (text: string, words: string) => {
    const moved = unmarked(text, text.indexOf(words));
    if (!moved) return { now: text, caret: -1 };
    const [edit, caret] = linedAs(text, ...joinedApart(text, moved.edit, moved.caret));
    return { now: text.slice(0, edit.from) + edit.insert + text.slice(edit.to), caret };
  };

  it.each([
    ["bullets", "- uno\n    - sub a\n    - sub b", "uno\n\n- sub a\n- sub b"],
    [
      "bullets two spaces in, one list with the item after",
      "- uno\n  - sub a\n  - sub b\n- dos",
      "uno\n\n- sub a\n- sub b\n- dos",
    ],
    [
      "steps, one list with the step after",
      "- uno\n    - [ ] sub a\n    - [x] sub b\n- [ ] dos",
      "uno\n\n- [ ] sub a\n- [x] sub b\n- [ ] dos",
    ],
    [
      "numbered ones, the numbers after left as they are",
      "1. cero\n2. uno\n    1. sub a\n    2. sub b\n3. dos",
      "1. cero\n\nuno\n\n1. sub a\n2. sub b\n3. dos",
    ],
    ["numbered ones three spaces in", "1. uno\n   1. sub a\n2. dos", "uno\n\n1. sub a\n2. dos"],
    [
      "the items under them, a level each",
      "- uno\n    - sub a\n        - hondo\n    - sub b\n- dos",
      "uno\n\n- sub a\n    - hondo\n- sub b\n- dos",
    ],
    [
      "items of a nested item, inside the item above",
      "- a\n  - uno\n    - c\n  - d",
      "- a\n\n  uno\n\n  - c\n  - d",
    ],
    [
      "items whose words go on over a second line",
      "- uno\n    - sub a\n      más de a\n    - sub b",
      "uno\n\n- sub a\n  más de a\n- sub b",
    ],
    [
      "an item whose words go on over a line with no indent",
      "- uno\n    - sub a\nsigue a\n    - sub b",
      "uno\n\n- sub a\nsigue a\n- sub b",
    ],
    [
      "an item holding a fence, its code as it was",
      "- uno\n    - sub\n      ```\n      - x\n        y\n      ```\n- dos",
      "uno\n\n- sub\n  ```\n  - x\n    y\n  ```\n- dos",
    ],
    [
      "an item holding indented code, still its code",
      "- uno\n    - sub\n\n          código\n- dos",
      "uno\n\n- sub\n\n      código\n- dos",
    ],
    [
      "a paragraph of the item after a blank line, with the items",
      "- uno\n\n  párrafo\n\n  - sub\n- dos",
      "uno\n\npárrafo\n\n- sub\n- dos",
    ],
    [
      "bullets apart from the numbered item after them",
      "1. uno\n    - a\n    - b\n2. dos",
      "uno\n\n- a\n- b\n\n2. dos",
    ],
    [
      "numbered ones whose words go on, one list with the item after",
      "1. uno\n    1. a\n       más\n2. dos",
      "uno\n\n1. a\n   más\n2. dos",
    ],
    [
      "items in a quote, inside the quote",
      "> - uno\n>     - sub a\n> - dos",
      "> uno\n>\n> - sub a\n> - dos",
    ],
    [
      "items of a file with Windows line ends",
      "- uno\r\n    - sub a\r\n    - sub b\r\n- dos",
      "uno\r\n\r\n- sub a\r\n- sub b\r\n- dos",
    ],
    [
      "indented code first, still code",
      "- uno\n\n      código\n- dos",
      "uno\n\n    código\n\n- dos",
    ],
    [
      "a fence whose lines open with a tab, the tab kept",
      "- uno\n    - sub\n      ```make\n      all:\n      \tcc x\n      ```\n- dos",
      "uno\n\n- sub\n  ```make\n  all:\n  \tcc x\n  ```\n- dos",
    ],
    ["an item with a tab after its bullet", "-\tuno\n\t- sub", "uno\n\n- sub"],
    [
      "an item indented with spaces and a tab, its tab stops read",
      "- a\n  \t- uno\n\t\t- sub",
      "- a\n\n  \tuno\n\n\t- sub",
    ],
  ])("on Backspace at an item that holds others, raises them a level: %s", (_, text, expected) => {
    const { now, caret } = backspaced(text, "uno");
    expect(now).toBe(expected);
    expect(now.slice(caret)).toMatch(/^uno/);
  });

  it("takes only the bullet of an item whose words go on and hold nothing more", () => {
    expect(unmarked("- uno\n  más\n\nPárrafo", 2)?.edit).toEqual({ from: 0, to: 5, insert: "uno" });
  });

  it("raises the items under a step once its box and then its bullet are gone", () => {
    const text = "- [ ] uno\n    - [ ] sub a\n- [x] dos";
    const boxed = backspaced(text, "uno").now;
    expect(boxed).toBe("- uno\n    - [ ] sub a\n- [x] dos");
    expect(backspaced(boxed, "uno").now).toBe("uno\n\n- [ ] sub a\n- [x] dos");
  });

  it.each([
    ["\n", "1. uno\n   más\n2. dos", "uno\n   más\n\n2. dos"],
    ["\r\n", "1. uno\r\n   más\r\n2. dos", "uno\r\n   más\r\n\r\n2. dos"],
  ])(
    "on Backspace at an item whose words go on, keeps the numbered item after it a list, with %j",
    (_, text, expected) => {
      expect(backspaced(text, "uno").now).toBe(expected);
    },
  );

  it.each([
    [
      "keeps the numbered items under a paragraph a list",
      "Antes.\n\n1. uno\n2. dos\n",
      [6, 11],
      "Antes.uno\n\n2. dos\n",
    ],
    [
      "keeps them a list inside a quote",
      "> Antes.\n>\n> 1. uno\n> 2. dos",
      [8, 16],
      "> Antes.uno\n>\n> 2. dos",
    ],
    [
      "joins bullets as they are, which open a list anywhere",
      "Antes.\n\n- uno\n- dos\n",
      [6, 10],
      "Antes.uno\n- dos\n",
    ],
    [
      "joins a list's own items as they are",
      "1. uno\n2. dos\n3. tres",
      [6, 10],
      "1. unodos\n3. tres",
    ],
    ["takes words within a line as they are", "Antes\n2. dos", [2, 3], "Anes\n2. dos"],
    [
      "leaves an item indented as code would be as it is",
      "Antes.\n\n1. uno\n    2. dos",
      [6, 11],
      "Antes.uno\n    2. dos",
    ],
    [
      "leaves one indented with a tab as it is",
      "Antes.\n\n1. uno\n\t2. dos",
      [6, 11],
      "Antes.uno\n\t2. dos",
    ],
    [
      "leaves lines of code as they are",
      "```\nuno\n\n1. dos\n2. tres\n```\n",
      [7, 12],
      "```\nunodos\n2. tres\n```\n",
    ],
    [
      "leaves code as it is inside a longer fence that holds a shorter one",
      "````\n```js\nuno\n\n1. dos\n2. tres\n````\n",
      [14, 19],
      "````\n```js\nunodos\n2. tres\n````\n",
    ],
    [
      "leaves code as it is inside a fence that holds the other mark",
      "```\n~~~\nuno\n\n1. dos\n2. tres\n```\n",
      [11, 16],
      "```\n~~~\nunodos\n2. tres\n```\n",
    ],
    [
      "leaves a whole line taken away as it is, which joins nothing",
      "Antes.\n\nquitar\ntexto\n2. dos",
      [8, 15],
      "Antes.\n\ntexto\n2. dos",
    ],
    [
      "leaves an item the paragraph already held as it is",
      "Uno\ndos tres\n2. falta",
      [1, 6],
      "Us tres\n2. falta",
    ],
    [
      "keeps a list apart whose item went on over a second line",
      "Antes.\n\n1. uno\n   más palabras\n2. dos",
      [6, 22],
      "Antes.palabras\n\n2. dos",
    ],
    [
      "keeps the items under a title joined to a paragraph a list",
      "Antes.\n# Título\n2. dos\n3. tres",
      [6, 7],
      "Antes.# Título\n\n2. dos\n3. tres",
    ],
    [
      "keeps the items under a title underlined a list",
      "Antes.\n\nTítulo\n===\n2. dos\n3. tres",
      [6, 15],
      "Antes.===\n\n2. dos\n3. tres",
    ],
    [
      "leaves an item held under a line of mixed marks, which are words",
      "Antes.\n\nx\n-*-\n2. dos",
      [6, 10],
      "Antes.-*-\n2. dos",
    ],
    [
      "keeps the list apart after a line whose backticks open no fence",
      "```js``` corre.\n\nAntes.\n\n1. uno\n2. dos",
      [23, 28],
      "```js``` corre.\n\nAntes.uno\n\n2. dos",
    ],
    [
      "keeps the list apart after a fence the end of its quote closed",
      "> ```\n> código\n\nAntes.\n\n1. uno\n2. dos",
      [22, 27],
      "> ```\n> código\n\nAntes.uno\n\n2. dos",
    ],
  ] as const)("on a join across a line end, %s", (_, text, [from, to], expected) => {
    const [edit] = joinedApart(text, { from, to, insert: "" });
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(expected);
  });

  it.each([
    ["\n", "- [ ] a\n\nb\n\n- [x] c", 7, 9],
    ["\r\n", "- [ ] a\r\n\r\nb\r\n\r\n- [x] c", 7, 11],
  ])(
    "takes words freed of a step back into the step above as one tight list, with %j",
    (feed, text, from, to) => {
      const [edit] = joinedApart(text, { from, to, insert: "" });
      expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
        `- [ ] ab${feed}- [x] c`,
      );
    },
  );

  it("reads a quote of many tabbed marks above a join in linear time", () => {
    const quoted = `>${"\t>".repeat(40)} x`;
    const text = `${quoted}\n\nAntes.\n\n1. uno\n2. dos`;
    const from = quoted.length + 8;
    const began = vi.getRealSystemTime();
    const [edit] = joinedApart(text, { from, to: from + 5, insert: "X" });
    expect(vi.getRealSystemTime() - began).toBeLessThan(1_000);
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
      `${quoted}\n\nAntes.Xuno\n\n2. dos`,
    );
  });

  it("keeps the list apart when words written over a choice across lines join them", () => {
    const text = "Antes.\n\n1. uno\n2. dos";
    const [edit, caret] = joinedApart(text, { from: 6, to: 11, insert: "X" });
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
      "Antes.Xuno\n\n2. dos",
    );
    expect(caret).toBe(7);
  });

  it("leaves code as it is where the joined line opens a fence", () => {
    const text = "uno\n1. dos\n2. tres";
    const [edit] = joinedApart(text, { from: 0, to: 7, insert: "```" });
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe("```dos\n2. tres");
  });

  it("keeps a caret past the joined line on its letter in a file with Windows line ends", () => {
    const text = "Antes.\r\n\r\n1. uno\r\n2. dos";
    const [edit, caret] = linedAs(text, ...joinedApart(text, { from: 6, to: 13, insert: "" }, 14));
    const now = text.slice(0, edit.from) + edit.insert + text.slice(edit.to);
    expect(now).toBe("Antes.uno\r\n\r\n2. dos");
    expect(now[caret]).toBe("d");
  });

  it("moves a caret past the joined line by the line end it adds", () => {
    const text = "Antes.\n\n1. uno\n2. dos";
    const [edit, caret] = joinedApart(text, { from: 6, to: 11, insert: "" }, 14);
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
      "Antes.uno\n\n2. dos",
    );
    expect(caret).toBe(15);
  });

  it.each([
    ["a paragraph", "Uno\n\nDos", 5, { at: 5, seed: "", tail: "\n\n" }],
    [
      "a note's words, inside the note",
      "> [!TIP]\n> Un consejo",
      11,
      { at: 9, seed: "> ", tail: "\n>\n" },
    ],
    ["a heading", "## Titulo", 3, { at: 0, seed: "", tail: "\n\n" }],
    ["a step, unticked", "- [x] hecho", 6, { at: 0, seed: "- [ ] ", tail: "\n" }],
  ])("on Enter, opens a line above %s", (_, text, at, expected) => {
    expect(openedAbove(text, at)).toEqual(expected);
  });

  it("leaves Enter in an empty item or mid-words to its own ways", () => {
    expect(openedAbove("- ", 2)).toBeNull();
    expect(openedAbove("Uno dos", 3)).toBeNull();
  });
});

describe("an address typed in a link's field", () => {
  it.each([
    ["tisty.app", "https://tisty.app"],
    ["www.tisty.app/es/", "https://www.tisty.app/es/"],
    ["https://tisty.app", "https://tisty.app"],
    ["mailto:hola@tisty.app", "mailto:hola@tisty.app"],
    ["notas.md", "https://notas.md"],
    ["#titulo", "#titulo"],
    ["./otro.md", "./otro.md"],
  ])("reads «%s» as %s", (typed, expected) => {
    expect(addressed(typed)).toBe(expected);
  });
});

describe("a format pressed again on words chosen with their marks", () => {
  it("takes the marks off, as a double click past them chooses them too", () => {
    const text = "Uno **dos** tres";
    expect(worn(text, { from: 4, to: 11 }, "**", "**")).toEqual({ from: 4, to: 11, insert: "dos" });
    expect(worn("a ~~b~~ c", { from: 2, to: 7 }, "~~", "~~")).toEqual({
      from: 2,
      to: 7,
      insert: "b",
    });
  });

  it("leaves bold alone when italic is asked for over it", () => {
    expect(worn("**dos**", { from: 0, to: 7 }, "*", "*").insert).not.toBe("*dos*");
  });
});

describe("words cut or copied out of formatted words", () => {
  const text = "Uno **dos** tres";
  it("carry the format they start or end inside, whole", () => {
    expect(balanced(text, { from: 0, to: 9 })).toEqual({ from: 0, to: 11 });
    expect(balanced(text, { from: 6, to: 16 })).toEqual({ from: 4, to: 16 });
  });

  it("carry the marks of words chosen exactly, and a pen's colour with its marks", () => {
    expect(balanced(text, { from: 6, to: 9 })).toEqual({ from: 4, to: 11 });
    expect(balanced("a ==dos=={green} b", { from: 4, to: 7 })).toEqual({ from: 2, to: 16 });
  });

  it("leave plain words as they are", () => {
    expect(balanced(text, { from: 0, to: 3 })).toEqual({ from: 0, to: 3 });
  });
});
