import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { writtenLink } from "../ui/pageFormats";
import { type Hands, type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-24"><span data-src="8-12">Uno </span><strong><span data-src="14-17">dos</span></strong><span data-src="19-24"> tres</span></p>' +
  '<ul data-at="26-46"><li data-at="26-35"><span data-src="28-35">primero</span></li>' +
  '<li data-at="36-45"><span data-src="38-45">segundo</span></li></ul>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const pageOf = (html = PLACED) => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document;
};

const piece = (said: string) =>
  Array.from(document.querySelectorAll("[data-src]")).find((one) => one.textContent === said)
    ?.firstChild as Text;

const caretIn = (said: string, at: number) => document.getSelection()?.collapse(piece(said), at);

const asked = (inputType: string, data: string | null = null) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
    );

const menus: boolean[] = [];

const writer = (body = BODY, more: Partial<Hands> = {}) => {
  writing?.stop();
  const said: string[] = [];
  let text = body;
  menus.length = 0;
  writing = writtenOn(pageOf(), {
    text: () => text,
    write: (next) => {
      text = next;
      said.push(next);
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: (_at, slash) => menus.push(slash),
    acts: () => {},
    ...more,
  });
  return said;
};

const block = (key: string) => SLASHED.find((one) => one.key === key) as Slashed;

const last = (said: string[]) => said[said.length - 1] ?? "";

const key = (name: string, more: KeyboardEventInit = {}) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true, ...more }),
    );

const choose = (said: string, from: number, to: number) => {
  const range = document.createRange();
  range.setStart(piece(said), from);
  range.setEnd(piece(said), to);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
  document.dispatchEvent(new Event("selectionchange"));
};

const collapsed = () => document.dispatchEvent(new Event("selectionchange"));

const clickAt = (said: string, at: number) => {
  document.querySelector(".page")?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  caretIn(said, at);
  collapsed();
};

// One paragraph holding the whole source as a single piece, the caret at a place of it.
const lineAt = (source: string, caret: number, more: Partial<Hands> = {}) => {
  const said = writer(source, more);
  pageOf(`<p data-at="0-${source.length}"><span data-src="0-${source.length}"></span></p>`);
  (document.querySelector("[data-src]") as Element).textContent = source;
  writing?.drawn();
  caretIn(source, caret);
  return said;
};

const codeAt = (source: string, from: number, to: number, caret: number) => {
  const said = writer(source);
  const inner = source.slice(from, to);
  pageOf(`<p data-at="0-${source.length}"><code data-src="${from}-${to}">${inner}</code></p>`);
  writing?.drawn();
  const code = document.querySelector("code")?.firstChild as Text;
  document.getSelection()?.collapse(code, caret);
  return said;
};

const field = () => document.querySelector<HTMLInputElement>(".page-address");

const answer = async (said: string, how = "Enter") => {
  const asking = field();
  if (asking) asking.value = said;
  asking?.dispatchEvent(new KeyboardEvent("keydown", { key: how, bubbles: true }));
  await Promise.resolve();
};

describe("a format pressed with nothing chosen", () => {
  it("writes nothing until the next key", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    expect(said).toEqual([]);
  });

  it("wraps the first key and leaves the caret inside the marks", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
    asked("insertText", "y");
    expect(last(said).startsWith("# Hola**xy**\n")).toBe(true);
  });

  it.each([
    ["Ctrl and B", "b", { ctrlKey: true }, "**"],
    ["Ctrl and I", "i", { ctrlKey: true }, "*"],
    ["Ctrl and U", "u", { ctrlKey: true }, "++"],
    ["Ctrl, Shift and X", "X", { ctrlKey: true, shiftKey: true }, "~~"],
    ["Ctrl, Shift and C", "C", { ctrlKey: true, shiftKey: true }, "`"],
    ["Command and B", "b", { metaKey: true }, "**"],
    ["Ctrl and a capital B", "B", { ctrlKey: true }, "**"],
  ])("wraps the first key after %s", (_, name, more, mark) => {
    const said = writer();
    caretIn("Hola", 4);
    key(name, more);
    asked("insertText", "x");
    expect(last(said).startsWith(`# Hola${mark}x${mark}\n`)).toBe(true);
  });

  it.each([["**"], ["=="], ["~~"]])("wraps the first key after the format call for %s", (mark) => {
    const said = writer();
    caretIn("Hola", 4);
    writing?.format(mark, mark);
    asked("insertText", "x");
    expect(last(said).startsWith(`# Hola${mark}x${mark}\n`)).toBe(true);
  });

  it.each([
    ["bold", "**"],
    ["italic", "*"],
    ["underline", "++"],
    ["strike", "~~"],
    ["inline", "`"],
    ["pen", "=="],
  ])("wraps the first key after the %s entry of the menu", (name, mark) => {
    const said = writer();
    caretIn("Hola", 4);
    writing?.slashed(block(name));
    expect(said).toEqual([]);
    asked("insertText", "x");
    expect(last(said).startsWith(`# Hola${mark}x${mark}\n`)).toBe(true);
  });

  it.each([
    ["formatBold", "**"],
    ["formatItalic", "*"],
    ["formatUnderline", "++"],
    ["formatStrikeThrough", "~~"],
  ])("waits for the key when the browser asks for %s", (kind, mark) => {
    const said = writer();
    caretIn("Hola", 4);
    asked(kind);
    expect(said).toEqual([]);
    asked("insertText", "x");
    expect(last(said).startsWith(`# Hola${mark}x${mark}\n`)).toBe(true);
  });

  it("keeps a space before the first key plain", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", " ");
    expect(last(said).startsWith("# Hola \n")).toBe(true);
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola **x**\n")).toBe(true);
  });

  it("keeps several spaces plain and the format waiting", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("u", { ctrlKey: true });
    asked("insertText", " ");
    asked("insertText", " ");
    expect(last(said).startsWith("# Hola  \n")).toBe(true);
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola  ++x++\n")).toBe(true);
  });

  it("lets the format go when pressed again before typing", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("lets the format go when pressed again after a plain space", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", " ");
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola x\n")).toBe(true);
  });

  it("waits again when pressed a third time", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    key("b", { ctrlKey: true });
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });

  it.each([
    ["b", { ctrlKey: true }, "**"],
    ["i", { ctrlKey: true }, "*"],
    ["u", { ctrlKey: true }, "++"],
    ["X", { ctrlKey: true, shiftKey: true }, "~~"],
    ["C", { ctrlKey: true, shiftKey: true }, "`"],
  ])("steps past the closing marks when %s is pressed again after typing", (name, more, mark) => {
    const said = writer();
    caretIn("Hola", 4);
    key(name, more);
    asked("insertText", "x");
    key(name, more);
    asked("insertText", "y");
    expect(last(said).startsWith(`# Hola${mark}x${mark}y\n`)).toBe(true);
  });

  it("nests an italic in bold when it is pressed inside the bold words", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    key("i", { ctrlKey: true });
    asked("insertText", "y");
    expect(last(said).startsWith("# Hola**x*y***\n")).toBe(true);
  });

  it("puts another format inside the first one's marks", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    key("u", { ctrlKey: true });
    asked("insertText", "y");
    expect(last(said).startsWith("# Hola**x++y++**\n")).toBe(true);
  });

  it("never writes empty marks while the keys only press formats", () => {
    const said = writer();
    caretIn("Hola", 4);
    for (const name of ["b", "i", "u", "b", "i", "u", "b"]) key(name, { ctrlKey: true });
    key("X", { ctrlKey: true, shiftKey: true });
    key("C", { ctrlKey: true, shiftKey: true });
    writing?.format("==", "==");
    writing?.slashed(block("bold"));
    expect(said).toEqual([]);
  });

  it("never writes the marks of a link with no words and no address", async () => {
    const said = writer();
    caretIn("Hola", 4);
    key("k", { ctrlKey: true });
    await answer("", "Enter");
    expect(said).toEqual([]);
  });

  it("wraps the text of a composition as the first key", () => {
    const said = writer();
    caretIn("Hola", 4);
    const page = document.querySelector(".page") as Element;
    key("b", { ctrlKey: true });
    page.dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true }));
    page.dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data: "é" }));
    expect(last(said).startsWith("# Hola**é**\n")).toBe(true);
  });

  it("writes a composition plain when no format waits", () => {
    const said = writer();
    caretIn("Hola", 4);
    const page = document.querySelector(".page") as Element;
    page.dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true }));
    page.dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data: "é" }));
    expect(last(said).startsWith("# Holaé\n")).toBe(true);
  });

  it("keeps the format waiting through the keys that only press a letter", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    key("x");
    key("Shift");
    key("Dead");
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });

  it.each([
    ["ArrowLeft"],
    ["ArrowRight"],
    ["ArrowUp"],
    ["ArrowDown"],
    ["Home"],
    ["End"],
    ["PageUp"],
  ])("drops the waiting format when %s moves the caret", (name) => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    key(name);
    caretIn("Hola", 4);
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("drops the waiting format when a mouse press moves the caret", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    document
      .querySelector(".page")
      ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    caretIn("Hola", 2);
    asked("insertText", "x");
    expect(last(said).startsWith("# Hoxla\n")).toBe(true);
  });

  it("drops the waiting format at Enter", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertParagraph");
    asked("insertText", "x");
    expect(last(said)).not.toContain("**x**");
    expect(last(said)).toContain("\nx\n");
  });

  it("drops the waiting format at a paste", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertFromPaste");
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("drops the waiting format at an undo", () => {
    const said = writer();
    caretIn("dos", 0);
    asked("insertText", "y");
    key("b", { ctrlKey: true });
    asked("historyUndo");
    asked("insertText", "x");
    expect(said[0]).toContain("**ydos**");
    expect(last(said)).not.toContain("****");
    expect(last(said)).not.toContain("**x**");
  });

  it("drops the waiting format at a redo", () => {
    const said = writer();
    caretIn("dos", 0);
    asked("insertText", "y");
    asked("historyUndo");
    key("b", { ctrlKey: true });
    asked("historyRedo");
    asked("insertText", "x");
    expect(last(said)).not.toContain("**x**");
  });

  it("does nothing with Ctrl and Alt together", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true, altKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("does nothing with Ctrl, Shift and a letter that is no format", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("B", { ctrlKey: true, shiftKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("waits for nothing inside code", () => {
    const said = codeAt("`ab`", 1, 3, 1);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said)).toBe("`axb`");
  });

  it("waits for nothing while a field has the focus", () => {
    const said = writer();
    caretIn("Hola", 4);
    const input = document.createElement("input");
    document.body.append(input);
    input.focus();
    key("b", { ctrlKey: true });
    input.remove();
    caretIn("Hola", 4);
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("keeps the caret inside the marks once the page is drawn again", () => {
    const said = lineAt("ab", 2);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said)).toBe("ab**x**");
    pageOf(
      '<p data-at="0-7"><span data-src="0-2">ab</span><strong><span data-src="4-5">x</span></strong></p>',
    );
    writing?.drawn();
    asked("insertText", "y");
    expect(last(said)).toBe("ab**xy**");
  });

  it("takes the wrapped key back with one undo", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    asked("historyUndo");
    expect(last(said)).toBe(BODY);
  });

  it("wraps a whole word typed at once as the first key", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("i", { ctrlKey: true });
    asked("insertText", "ab");
    expect(last(said).startsWith("# Hola*ab*\n")).toBe(true);
  });

  it("keeps a leading space of the first key outside the marks", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", " x");
    expect(last(said).startsWith("# Hola **x**\n")).toBe(true);
  });
});

describe("words chosen earlier", () => {
  it("are not wrapped by Ctrl and B after the caret collapsed elsewhere", () => {
    const said = writer();
    choose("primero", 0, 7);
    caretIn("Hola", 4);
    collapsed();
    key("b", { ctrlKey: true });
    expect(said).toEqual([]);
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });

  it("are not wrapped by the format call after the caret collapsed in the same words", () => {
    const said = writer();
    choose("primero", 0, 7);
    caretIn("primero", 3);
    collapsed();
    writing?.format("**", "**");
    expect(said).toEqual([]);
  });

  it("are not wrapped by an entry of the menu after the caret collapsed elsewhere", () => {
    const said = writer();
    choose("segundo", 0, 7);
    caretIn("Hola", 4);
    collapsed();
    writing?.slashed(block("italic"));
    expect(said).toEqual([]);
  });

  it("are not linked after the caret collapsed elsewhere", () => {
    writer();
    choose("segundo", 0, 7);
    caretIn("Hola", 4);
    collapsed();
    key("k", { ctrlKey: true });
    expect(field()?.className).toBe("page-address");
  });

  it("are no longer reported as chosen", () => {
    writer();
    choose("dos", 0, 3);
    expect(writing?.chosen()).toBe("**dos**");
    caretIn("Hola", 2);
    collapsed();
    expect(writing?.chosen()).toBeNull();
  });

  it("are not cut after the caret collapsed elsewhere", () => {
    const said = writer();
    choose("dos", 0, 3);
    caretIn("Hola", 2);
    collapsed();
    writing?.cut();
    expect(said).toEqual([]);
  });

  it("are still wrapped when the selection is lost without a collapse, as by a button", () => {
    const said = writer();
    choose("segundo", 0, 7);
    document.getSelection()?.removeAllRanges();
    key("b", { ctrlKey: true });
    expect(last(said)).toContain("- **segundo**");
  });

  it("are wrapped by every format key while they stay chosen", () => {
    const said = writer();
    choose("primero", 0, 7);
    key("u", { ctrlKey: true });
    expect(last(said)).toContain("- ++primero++");
  });

  it("take the bold off again and keep it off when the caret then collapses", () => {
    const said = writer();
    choose("dos", 0, 3);
    key("b", { ctrlKey: true });
    expect(last(said)).toContain("Uno dos tres");
    clickAt("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });

  it("wraps in bold and italic together when italic is pressed on bold words", () => {
    const said = writer();
    choose("dos", 0, 3);
    key("i", { ctrlKey: true });
    expect(last(said)).toContain("Uno ***dos*** tres");
  });
});

describe("a space typed at the end of formatted words", () => {
  it.each([
    ["bold", "word **dos**", 10],
    ["italic with stars", "word *dos*", 9],
    ["italic with underscores", "word _dos_", 9],
    ["bold with underscores", "word __dos__", 10],
    ["underline", "word ++dos++", 10],
    ["strike", "word ~~dos~~", 10],
    ["pen", "word ==dos==", 10],
    ["pen with a colour", "word ==dos=={green}", 10],
    ["a link's bold words", "a **[x](u)**", 10],
  ])("goes past the marks of %s", (_, source, caret) => {
    const said = lineAt(source, caret);
    asked("insertText", " ");
    const at = source.length;
    const tail = source.slice(caret);
    expect(last(said)).toBe(`${source.slice(0, at - tail.length)}${tail} `);
  });

  it("stays plain in a word that is not formatted", () => {
    const said = lineAt("word **dos**", 3);
    asked("insertText", " ");
    expect(last(said)).toBe("wor d **dos**");
  });

  it("stays inside when the character before is a space", () => {
    const said = lineAt("word **dos **", 11);
    asked("insertText", " ");
    expect(last(said)).toBe("word **dos  **");
  });

  it("stays inside inline code", () => {
    const said = lineAt("`a **b**c`", 6);
    asked("insertText", " ");
    expect(last(said)).toBe("`a **b **c`");
  });

  it("stays inside inline code that holds the marks as words", () => {
    const said = lineAt("**`x`**", 4);
    asked("insertText", " ");
    expect(last(said)).toBe("**`x `**");
  });

  it("stays plain before the marks of words that open", () => {
    const said = lineAt("a b**dos**", 3);
    asked("insertText", " ");
    expect(last(said)).toBe("a b **dos**");
  });

  it("stays plain before an opening mark that follows a closed format", () => {
    const said = lineAt("**a** b *c*", 7);
    asked("insertText", " ");
    expect(last(said)).toBe("**a** b  *c*");
  });

  it("stays plain when the key is a letter", () => {
    const said = lineAt("word **dos**", 10);
    asked("insertText", "x");
    expect(last(said)).toBe("word **dosx**");
  });

  it("replaces the words chosen instead of going past the marks", () => {
    const said = lineAt("word **dos**", 7);
    choose("word **dos**", 7, 10);
    asked("insertText", " ");
    expect(last(said)).toBe("word ** **");
  });

  it("stays plain at the start of the page", () => {
    const said = lineAt("**dos**", 0);
    asked("insertText", " ");
    expect(last(said)).toBe(" **dos**");
  });

  it("keeps the bold marks together when the words are bold and italic", () => {
    const said = lineAt("***dos***", 6);
    asked("insertText", " ");
    expect(last(said)).toBe("***dos*** ");
  });

  it("keeps the italic marks together when the bold is closed around them", () => {
    const said = lineAt("**a *b***", 7);
    asked("insertText", " ");
    expect(last(said)).toBe("**a *b*** ");
  });

  it("goes past the marks of the first format only when each one closes", () => {
    const said = lineAt("**a** b **c**", 11);
    asked("insertText", " ");
    expect(last(said)).toBe("**a** b **c** ");
  });

  it("does not move a space past marks that belong to a word that follows", () => {
    const said = lineAt("**a**b**c**", 6);
    asked("insertText", " ");
    expect(last(said)).toBe("**a**b **c**");
  });
});

describe("Enter at the end of a line whose rest is closing marks", () => {
  it.each([
    ["bold", "**dos**", 5],
    ["italic with a star", "*dos*", 4],
    ["italic with an underscore", "_dos_", 4],
    ["bold with underscores", "__dos__", 5],
    ["underline", "++dos++", 5],
    ["strike", "~~dos~~", 5],
    ["pen", "==dos==", 5],
    ["pen with a colour", "==dos=={green}", 5],
    ["bold and italic", "***dos***", 6],
    ["bold in italic", "*a **dos***", 9],
    ["code", "a `dos` b `x`", 12],
    ["a link", "[dos](https://tisty.app)", 4],
    ["a link with a title", '[dos](https://tisty.app "its title")', 4],
    ["a bold link", "**[dos](https://tisty.app)**", 6],
    ["a link in a strike", "~~a [dos](u)~~", 8],
  ])("starts the paragraph past the marks of %s", (_, source, caret) => {
    const said = lineAt(source, caret);
    asked("insertParagraph");
    expect(last(said)).toBe(`${source}\n\n`);
  });

  it("starts the paragraph past the marks when more lines follow", () => {
    const said = lineAt("**dos**\nnext", 5);
    asked("insertParagraph");
    expect(last(said)).toBe("**dos**\n\n\nnext");
  });

  it("starts the paragraph past the marks of a line that ends in a return and a newline", () => {
    const said = lineAt("**dos**\r\nnext", 5);
    asked("insertParagraph");
    expect(last(said)).toBe("**dos**\r\n\r\n\r\nnext");
  });

  it("splits the line in the middle of the words", () => {
    const said = lineAt("**dos tres**", 6);
    asked("insertParagraph");
    expect(last(said)).toBe("**dos \n\ntres**");
  });

  it("starts the paragraph past marks that words follow", () => {
    const said = lineAt("**dos** tres", 5);
    asked("insertParagraph");
    expect(last(said)).toBe("**dos**\n\n tres");
  });

  it("starts the paragraph past a link's address when words follow it", () => {
    const said = lineAt("[dos](u) tres", 4);
    asked("insertParagraph");
    expect(last(said)).toBe("[dos](u)\n\n tres");
  });

  it("leaves a link's address alone when it holds a space", () => {
    const said = lineAt("[dos](a b)", 4);
    asked("insertParagraph");
    expect(last(said)).toBe("[dos\n\n](a b)");
  });

  it("starts a paragraph at the end of plain words as always", () => {
    const said = lineAt("dos", 3);
    asked("insertParagraph");
    expect(last(said)).toBe("dos\n\n");
  });

  it("replaces the words chosen and does not step past the marks", () => {
    const said = lineAt("**dos**", 2);
    choose("**dos**", 2, 5);
    asked("insertParagraph");
    expect(last(said)).toBe("**\n\n**");
  });

  it("stays inside code", () => {
    const said = codeAt("`dos`", 1, 4, 2);
    asked("insertParagraph");
    expect(last(said)).toBe("`do\ns`");
  });

  it("stays inside code at its end, whatever follows", () => {
    const said = codeAt("`dos`", 1, 4, 3);
    asked("insertParagraph");
    expect(last(said)).toBe("`dos\n`");
  });

  it("starts the next item of a list past the marks", () => {
    const source = "- **dos**\n- tres";
    const said = writer(source);
    pageOf(
      '<ul data-at="0-16"><li data-at="0-9"><span data-src="2-9">**dos**</span></li><li data-at="10-16"><span data-src="12-16">tres</span></li></ul>',
    );
    writing?.drawn();
    caretIn("**dos**", 5);
    asked("insertParagraph");
    expect(last(said)).toBe("- **dos**\n- \n- tres");
  });

  it("starts the next numbered item past the marks", () => {
    const said = lineAt("1. **dos**", 8);
    asked("insertParagraph");
    expect(last(said)).toBe("1. **dos**\n2. ");
  });

  it("goes on in the quote past the marks", () => {
    const said = lineAt("> **dos**", 7);
    asked("insertParagraph");
    expect(last(said)).toBe("> **dos**\n>\n> ");
  });

  it("puts the caret after the new break, not between the marks", () => {
    const said = lineAt("**dos**", 5);
    asked("insertParagraph");
    asked("insertText", "x");
    expect(last(said)).toBe("**dos**\n\nx");
  });
});

describe("Ctrl and K", () => {
  it("asks the address in a field of the page", () => {
    writer();
    choose("dos", 0, 3);
    key("k", { ctrlKey: true });
    expect(field()).not.toBeNull();
  });

  it("writes the words chosen as the text of the link", async () => {
    const said = writer();
    choose("dos", 0, 3);
    key("k", { ctrlKey: true });
    await answer("https://tisty.app");
    expect(last(said)).toContain("Uno **[dos](https://tisty.app)** tres");
  });

  it("links the words chosen when the field took the selection away", async () => {
    const said = writer();
    choose("segundo", 0, 7);
    document.getSelection()?.removeAllRanges();
    key("k", { ctrlKey: true });
    await answer("https://tisty.app");
    expect(last(said)).toContain("- [segundo](https://tisty.app)");
  });

  it("writes the address as its own text when no words were chosen", async () => {
    const said = writer();
    caretIn("Hola", 4);
    key("k", { ctrlKey: true });
    await answer("https://tisty.app");
    expect(last(said).startsWith("# Hola[https://tisty.app](https://tisty.app)\n")).toBe(true);
  });

  it("links with Command and K as well", async () => {
    const said = writer();
    choose("dos", 0, 3);
    key("k", { metaKey: true });
    await answer("u");
    expect(last(said)).toContain("[dos](u)");
  });

  it("writes nothing when the field is answered with Escape", async () => {
    const said = writer();
    choose("dos", 0, 3);
    key("k", { ctrlKey: true });
    await answer("https://tisty.app", "Escape");
    expect(said).toEqual([]);
    expect(field()).toBeNull();
  });

  it("writes nothing when no words are chosen and the field is answered with Escape", async () => {
    const said = writer();
    caretIn("Hola", 4);
    key("k", { ctrlKey: true });
    await answer("https://tisty.app", "Escape");
    expect(said).toEqual([]);
  });

  it("writes nothing when the address is blank", async () => {
    const said = writer();
    choose("dos", 0, 3);
    key("k", { ctrlKey: true });
    await answer("   ");
    expect(said).toEqual([]);
  });

  it("writes the address without the spaces around it", async () => {
    const said = writer();
    choose("dos", 0, 3);
    key("k", { ctrlKey: true });
    const asking = field();
    if (asking) asking.type = "text";
    await answer("  u  ");
    expect(last(said)).toContain("[dos](u)");
  });

  it("takes the field away once answered", async () => {
    writer();
    choose("dos", 0, 3);
    key("k", { ctrlKey: true });
    await answer("u");
    expect(field()).toBeNull();
  });

  it("asks nothing inside code", () => {
    codeAt("`ab`", 1, 3, 2);
    key("k", { ctrlKey: true });
    expect(field()).toBeNull();
  });

  it("puts a link after the words when the address is typed on a caret at the end of a line", async () => {
    const said = lineAt("see ", 4);
    key("k", { ctrlKey: true });
    await answer("u");
    expect(last(said)).toBe("see [u](u)");
  });

  it("links through the format call too", async () => {
    const said = writer();
    choose("dos", 0, 3);
    writing?.format("[", "]()");
    await answer("u");
    expect(last(said)).toContain("[dos](u)");
  });

  it("links through the entry of the menu", async () => {
    const said = writer();
    choose("dos", 0, 3);
    writing?.slashed(block("link"));
    await answer("u");
    expect(last(said)).toContain("[dos](u)");
  });

  describe("on a link already written", () => {
    it("offers its address and changes only that", async () => {
      const said = lineAt('ver [Rina](a.com "su sitio") hoy', 6);
      key("k", { ctrlKey: true });
      expect(field()?.value).toBe("a.com");
      await answer("rina.app");
      expect(last(said)).toBe('ver [Rina](https://rina.app "su sitio") hoy');
    });

    it("keeps an address with spaces between angle brackets", async () => {
      const said = lineAt("[plan](<mi plan.pdf>)", 2);
      key("k", { ctrlKey: true });
      expect(field()?.value).toBe("mi plan.pdf");
      await answer("otro plan.pdf");
      expect(last(said)).toBe("[plan](<otro plan.pdf>)");
    });

    it("takes an address with parentheses whole", async () => {
      const said = lineAt("ver [perro](https://es.wikipedia.org/wiki/Perro_(animal)) hoy", 6);
      key("k", { ctrlKey: true });
      expect(field()?.value).toBe("https://es.wikipedia.org/wiki/Perro_(animal)");
      await answer("a.com");
      expect(last(said)).toBe("ver [perro](https://a.com) hoy");
    });

    it("leaves a link written by reference as it is, never nesting another inside", () => {
      const said = writer("ver [mapa][r] hoy\n\n[r]: https://tisty.app\n");
      pageOf(
        '<p data-at="0-17"><span data-src="0-4">ver </span><a href="https://tisty.app"><span data-src="5-9">mapa</span></a><span data-src="13-17"> hoy</span></p>',
      );
      writing?.drawn();
      caretIn("mapa", 2);
      key("k", { ctrlKey: true });
      expect(field()).toBeNull();
      expect(said).toEqual([]);
    });

    it("leaves the link as it was when the field is left with Escape", async () => {
      const said = lineAt("ver [Rina](a.com) hoy", 6);
      key("k", { ctrlKey: true });
      await answer("b.com", "Escape");
      expect(said).toEqual([]);
    });
  });
});

describe("a link found around the caret", () => {
  it("is a link and never a picture, and only one that holds the caret", () => {
    const text = "![x](a.png) y [Rina](a.com) y [Roger](b.com)";
    expect(writtenLink(text, { from: 2, to: 2 })).toBeNull();
    expect(writtenLink(text, { from: 16, to: 16 })?.address).toBe("a.com");
    expect(writtenLink(text, { from: 33, to: 36 })?.label).toBe("Roger");
    expect(writtenLink(text, { from: 12, to: 12 })).toBeNull();
  });
});

describe("Ctrl and Enter", () => {
  const broken = (body: string, html: string, said: string, at: number) => {
    const result = writer(body);
    pageOf(html);
    writing?.drawn();
    caretIn(said, at);
    key("Enter", { ctrlKey: true });
    return result;
  };

  it("puts a break after a paragraph", () => {
    const said = writer();
    caretIn("dos", 1);
    key("Enter", { ctrlKey: true });
    expect(last(said)).toMatch(/Uno \*\*dos\*\* tres\n\n+---\n/);
    expect(last(said)).toContain("- primero\n- segundo\n");
  });

  it("leaves one blank line on each side of the break after a paragraph", () => {
    const said = writer();
    caretIn("dos", 1);
    key("Enter", { ctrlKey: true });
    expect(last(said)).toContain("Uno **dos** tres\n\n---\n\n- primero");
  });

  it("puts a break after a heading", () => {
    const said = writer();
    caretIn("Hola", 2);
    key("Enter", { ctrlKey: true });
    expect(last(said).startsWith("# Hola\n\n---\n")).toBe(true);
  });

  it("puts a break after the list the item is in", () => {
    const said = writer();
    caretIn("primero", 3);
    key("Enter", { ctrlKey: true });
    expect(last(said)).toMatch(/- primero\n- segundo\n\n+---\n/);
  });

  it("ends the page with a single blank line before a break put after the last list", () => {
    const said = writer();
    caretIn("primero", 3);
    key("Enter", { ctrlKey: true });
    expect(last(said)).toContain("- primero\n- segundo\n\n---\n");
  });

  it("puts a break after the quote", () => {
    const said = broken(
      "> hola\n\nfin\n",
      '<blockquote data-at="0-6"><p><span data-src="2-6">hola</span></p></blockquote><p data-at="8-11"><span data-src="8-11">fin</span></p>',
      "hola",
      2,
    );
    expect(last(said)).toMatch(/^> hola\n\n+---\n\n+fin\n$/);
  });

  it("puts a break after the last block of the page", () => {
    const said = broken("fin", '<p data-at="0-3"><span data-src="0-3">fin</span></p>', "fin", 3);
    expect(last(said)).toBe("fin\n\n---\n");
  });

  it("puts nothing inside code", () => {
    const said = codeAt("`ab`", 1, 3, 1);
    key("Enter", { ctrlKey: true });
    expect(said).toEqual([]);
  });

  it("puts nothing inside a code block", () => {
    const said = broken(
      "```\nuno\n```\n",
      '<figure class="code" data-at="0-12"><pre><code data-src="4-7">uno</code></pre></figure>',
      "uno",
      1,
    );
    expect(said).toEqual([]);
  });

  it("puts a break with Command and Enter as well", () => {
    const said = writer();
    caretIn("dos", 1);
    key("Enter", { metaKey: true });
    expect(last(said)).toContain("Uno **dos** tres\n\n---\n");
  });

  it("keeps the browser from breaking the line too", () => {
    writer();
    caretIn("dos", 1);
    const event = new KeyboardEvent("keydown", {
      key: "Enter",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  it("puts nothing when the caret is nowhere", () => {
    const said = writer();
    document.getSelection()?.removeAllRanges();
    key("Enter", { ctrlKey: true });
    expect(said).toEqual([]);
  });

  it("lets a plain Enter alone", () => {
    writer();
    caretIn("dos", 1);
    const event = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
  });
});

describe("Ctrl and a slash", () => {
  it("opens the blocks menu, not for a slash", () => {
    writer();
    caretIn("dos", 1);
    key("/", { ctrlKey: true });
    expect(menus).toEqual([false]);
  });

  it("opens it with Command too", () => {
    writer();
    caretIn("dos", 1);
    key("/", { metaKey: true });
    expect(menus).toEqual([false]);
  });

  it("keeps the browser from using the key", () => {
    writer();
    caretIn("dos", 1);
    const event = new KeyboardEvent("keydown", {
      key: "/",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  it("opens nothing for a slash alone", () => {
    writer();
    caretIn("dos", 1);
    key("/");
    expect(menus).toEqual([]);
  });

  it("opens nothing for Ctrl and Alt", () => {
    writer();
    caretIn("dos", 1);
    key("/", { ctrlKey: true, altKey: true });
    expect(menus).toEqual([]);
  });

  it("writes nothing while it opens the menu", () => {
    const said = writer();
    caretIn("dos", 1);
    key("/", { ctrlKey: true });
    expect(said).toEqual([]);
  });

  it("keeps a waiting format, as the key moves no caret", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    key("/", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });
});
