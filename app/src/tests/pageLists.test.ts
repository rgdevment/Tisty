import { describe, expect, it } from "vitest";
import { pastedAround } from "../ui/pageBlocks";
import { addressed } from "../ui/pageFormats";
import { joinedAt, nested, openedAbove, outdented, unmarked } from "../ui/pageLists";
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
    ["a bullet loses its bullet", "## Uno\n\n- uno\n- dos", 10, "## Uno\n\nuno\n- dos", 8],
    ["an item in a quote keeps the quote", "> - x", 4, "> x", 2],
  ])("on Backspace, %s", (_, text, at, expected, caret) => {
    const moved = unmarked(text, at);
    const applied = moved
      ? text.slice(0, moved.edit.from) + moved.edit.insert + text.slice(moved.edit.to)
      : text;
    expect(applied).toBe(expected);
    expect(moved?.caret).toBe(caret);
  });

  it("leaves Backspace inside an item's words alone", () => {
    expect(unmarked("- uno", 3)).toBeNull();
  });

  it.each([
    ["a paragraph", "Uno\n\nDos", 5, { at: 5, seed: "", tail: "\n\n" }],
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
