import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { sourceAt, spotOf } from "../ui/pageSpots";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY = "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n";
const PLACED =
  '<h1 data-at="0-6"><span data-src="2-6">Hola</span></h1>' +
  '<p data-at="8-24"><span data-src="8-12">Uno </span><strong><span data-src="14-17">dos</span></strong><span data-src="19-24"> tres</span></p>' +
  '<ul data-at="26-46"><li data-at="26-35"><span data-src="28-35">primero</span></li>' +
  '<li data-at="36-45"><span data-src="38-45">segundo</span></li></ul>';

let writing: PageWriting | null = null;

// The core draws the page again after each change, every place past it moved along.
const moved = (html: string, past: number, by: number) =>
  html.replace(/data-(at|src)="(\d+)-(\d+)"/g, (_, kind: string, from: string, to: string) => {
    const shift = (at: number) => (at > past ? at + by : at);
    return `data-${kind}="${shift(Number(from))}-${shift(Number(to))}"`;
  });

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
const answers: string[] = [];

const writer = (body = BODY) => {
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
    translated: (_, plain) => Promise.resolve(plain),
    ask: (_kind, _at, put) => {
      const answer = answers.shift();
      if (answer !== undefined) put(answer);
    },
    blocks: (_at, slash) => menus.push(slash),
    acts: () => {},
  });
  return said;
};

const block = (key: string) => SLASHED.find((one) => one.key === key) as Slashed;

describe("the places of the page the core draws to be written on", () => {
  it("finds the Markdown under a point in the words, past the marks the page hides", () => {
    pageOf();
    expect(sourceAt(BODY, piece("dos"), 1)).toBe(BODY.indexOf("dos") + 1);
    expect(sourceAt(BODY, piece(" tres"), 0)).toBe(BODY.indexOf(" tres"));
  });

  it("takes the start of the words for a point at the start of their block", () => {
    pageOf();
    const block = document.querySelector('p[data-at="8-24"]') as Element;
    expect(sourceAt(BODY, block, 0)).toBe(BODY.indexOf("Uno"));
  });

  it("lays the caret inside the piece that holds a place in the Markdown", () => {
    pageOf();
    const spot = spotOf(document, BODY, BODY.indexOf("dos") + 2);
    expect(spot?.[0]).toBe(piece("dos"));
    expect(spot?.[1]).toBe(2);
  });
});

describe("writing on the page the core draws", () => {
  it("writes a key into the Markdown where the caret stands", () => {
    const said = writer();
    caretIn("dos", 3);
    asked("insertText", "!");
    expect(said[said.length - 1]).toBe("# Hola\n\nUno **dos!** tres\n\n- primero\n- segundo\n");
  });

  it("starts a paragraph with Enter, and a new item inside a list", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    expect(said[said.length - 1]?.startsWith("# Hola\n\n\n\nUno")).toBe(true);
    writing?.stop();
    const items = writer();
    caretIn("primero", 7);
    asked("insertParagraph");
    expect(items[items.length - 1]).toContain("- primero\n- \n- segundo");
  });

  it("stands a paragraph in for one just begun, and writes into it", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    pageOf(moved(PLACED, 7, 2));
    writing?.drawn();
    expect(document.querySelector("[data-ghost]")).not.toBeNull();
    asked("insertText", "x");
    expect(said[said.length - 1]?.startsWith("# Hola\n\nx\n\nUno")).toBe(true);
  });

  it("wraps the words chosen in a format's marks", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("primero"), 0);
    range.setEnd(piece("primero"), 7);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    asked("formatBold");
    expect(said[said.length - 1]).toContain("- **primero**");
  });

  it("writes what a composition ends with, as an accent typed with a dead key", () => {
    const said = writer();
    caretIn("Hola", 4);
    const page = document.querySelector(".page") as Element;
    page.dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true }));
    page.dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data: "é" }));
    expect(said[said.length - 1]?.startsWith("# Holaé\n")).toBe(true);
  });

  it("takes a change back with undo, and again with redo", () => {
    const said = writer();
    caretIn("dos", 0);
    asked("insertText", "y ");
    asked("historyUndo");
    expect(said[said.length - 1]).toBe(BODY);
    asked("historyRedo");
    expect(said[said.length - 1]).toContain("**y dos**");
  });

  it("takes a format off words already wearing it, and keeps the words chosen before a button", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("dos"), 0);
    range.setEnd(piece("dos"), 3);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
    document.getSelection()?.removeAllRanges();
    writing?.format("**", "**");
    expect(said[said.length - 1]).toBe("# Hola\n\nUno dos tres\n\n- primero\n- segundo\n");
  });

  it("cuts the words chosen and hands their Markdown", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("Uno "), 0);
    range.setEnd(piece("dos"), 3);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    expect(writing?.chosen()).toBe("Uno **dos");
    writing?.cut();
    expect(said[said.length - 1]).toBe("# Hola\n\n** tres\n\n- primero\n- segundo\n");
  });

  it("keeps the caret it lays through a key, and lets it go with an arrow", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertText", "a");
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "b" }));
    asked("insertText", "b");
    expect(said[said.length - 1]?.startsWith("# Holaab\n")).toBe(true);
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
    caretIn("Hola", 0);
    asked("insertText", "¡");
    expect(said[said.length - 1]?.startsWith("# ¡Holaab\n")).toBe(true);
  });

  it("leaves tables, drawings and rules to their own hands", () => {
    writer();
    pageOf(`${PLACED}<div class="table" data-at="47-60"></div><hr data-at="61-64">`);
    writing?.drawn();
    expect(document.querySelector(".table")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("hr")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("p")?.hasAttribute("contenteditable")).toBe(false);
  });
});

describe("the blocks put on the page written in place", () => {
  it("offers the blocks at a slash on a line just begun, and writes it anywhere else", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    pageOf(moved(PLACED, 7, 2));
    writing?.drawn();
    asked("insertText", "/");
    expect(menus).toEqual([true]);
    caretIn("dos", 1);
    asked("insertText", "/");
    expect(said[said.length - 1]).toContain("**d/os**");
  });

  it("turns the line the caret stands in into a title, its list mark gone", () => {
    const said = writer();
    caretIn("primero", 3);
    writing?.slashed(block("h2"));
    expect(said[said.length - 1]).toContain("## primero\n- segundo");
  });

  it("puts a table after the block the caret stands in, a blank line apart", () => {
    const said = writer();
    caretIn("dos", 1);
    writing?.slashed(block("table"));
    expect(said[said.length - 1]).toContain("Uno **dos** tres\n\n|  |  |\n| --- | --- |");
  });

  it("puts what the window answers for a diagram as a block of its own", () => {
    const said = writer();
    answers.push("```mermaid\nflowchart LR\n```");
    caretIn("Hola", 4);
    writing?.slashed(block("diagram"));
    const now = said[said.length - 1] ?? "";
    expect(now.startsWith("# Hola\n\n```mermaid\nflowchart LR\n```\n\nUno")).toBe(true);
  });
});

describe("blocks moved with the keys on the page written in place", () => {
  it("moves the item the caret stands in up with Alt and an arrow", () => {
    const said = writer();
    caretIn("segundo", 2);
    document
      .querySelector(".page")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true, bubbles: true }),
      );
    expect(said[said.length - 1]?.endsWith("- segundo\n- primero\n")).toBe(true);
  });
});

describe("a line begun on a task's page written in place", () => {
  it("writes nothing until a key, then joins a step to the list above", () => {
    const said = writer();
    const list = document.querySelector("ul") as Element;
    writing?.begin(BODY.length - 1, "- [ ] ", list);
    expect(said).toHaveLength(0);
    expect(document.querySelector("[data-ghost]")).not.toBeNull();
    asked("insertText", "x");
    expect(said[said.length - 1]).toBe(
      "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n- [ ] x\n",
    );
  });

  it("stands a line apart from the block above, and leaves it unwritten on a press back", () => {
    const said = writer();
    const label = document.querySelector("h1") as Element;
    writing?.begin(6, "", label);
    asked("deleteContentBackward");
    expect(said).toHaveLength(0);
    expect(document.querySelector("[data-ghost]")).toBeNull();
    writing?.begin(6, "", label);
    asked("insertText", "y");
    expect(said[said.length - 1]?.startsWith("# Hola\n\ny\n\nUno")).toBe(true);
  });

  it("mends a section's hint in a field, keeping the box it asks for", async () => {
    const body = "## Pasos\n<!-- - [ ] El primer paso -->\n";
    const said = writer(body);
    pageOf(
      '<h2 data-at="0-8">Pasos</h2><div class="hint step-hint" data-at="9-38" data-hint="El primer paso"></div>',
    );
    writing?.hintWords([9, 38], document.querySelector(".hint") as HTMLElement);
    const field = document.querySelector<HTMLInputElement>(".page-address");
    expect(field?.value).toBe("El primer paso");
    if (field) field.value = "Lo primero";
    field?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await Promise.resolve();
    expect(said[said.length - 1]).toBe("## Pasos\n<!-- - [ ] Lo primero -->\n");
  });

  it("fills a task under its labels, and writes only its labels while its sections are asked for", () => {
    writer();
    document.documentElement.classList.add("untitled");
    writing?.drawn();
    expect(document.querySelector("h1")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("p")?.hasAttribute("contenteditable")).toBe(false);
    document.documentElement.classList.add("shaping");
    writing?.drawn();
    expect(document.querySelector("h1")?.hasAttribute("contenteditable")).toBe(false);
    expect(document.querySelector("p")?.getAttribute("contenteditable")).toBe("false");
    document.documentElement.classList.remove("untitled", "shaping");
  });
});

describe("more of writing on the page the core draws", () => {
  const chooseIn = (said: string, from: number, to: number) => {
    const range = document.createRange();
    range.setStart(piece(said), from);
    range.setEnd(piece(said), to);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
  };

  const pasted = (html: string | null, plain: string) => {
    const event = new InputEvent("beforeinput", {
      inputType: "insertFromPaste",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "dataTransfer", {
      value: { getData: (kind: string) => (kind === "text/html" ? (html ?? "") : plain) },
    });
    document.querySelector(".page")?.dispatchEvent(event);
  };

  it("pastes what the translator makes of a web page, never its HTML", async () => {
    const said: string[] = [];
    let text = BODY;
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
      translated: (html, plain) => Promise.resolve(html ? "**pegado**" : plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    caretIn("Hola", 4);
    pasted("<b>pegado</b>", "pegado");
    await Promise.resolve();
    await Promise.resolve();
    expect(said[said.length - 1]?.startsWith("# Hola**pegado**\n")).toBe(true);
  });

  it("writes over the words chosen, and breaks a line with Shift and Enter", () => {
    const said = writer();
    chooseIn("Hola", 0, 4);
    asked("insertText", "Chao");
    expect(said[said.length - 1]?.startsWith("# Chao\n")).toBe(true);
    caretIn(" tres", 5);
    asked("insertLineBreak");
    expect(said[said.length - 1]).toContain("**dos** tres\n\n");
  });

  it("links the words chosen to the address written in its field", async () => {
    const said = writer();
    chooseIn("primero", 0, 7);
    document.dispatchEvent(new Event("selectionchange"));
    writing?.format("[", "]()");
    const field = document.querySelector<HTMLInputElement>(".page-address");
    if (field) field.value = "https://tisty.app";
    field?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await Promise.resolve();
    expect(said[said.length - 1]).toContain("- [primero](https://tisty.app)");
  });

  it("offers the blocks with Ctrl and a slash, and puts a mark or a word where the caret stands", () => {
    const said = writer();
    caretIn("dos", 1);
    document
      .querySelector(".page")
      ?.dispatchEvent(new KeyboardEvent("keydown", { key: "/", ctrlKey: true, bubbles: true }));
    expect(menus).toEqual([false]);
    writing?.slashed(block("quote"));
    expect(said[said.length - 1]).toContain("> Uno **dos** tres");
    pageOf(moved(PLACED, 7, 2));
    writing?.drawn();
    caretIn("segundo", 7);
    writing?.slashed(block("step"));
    expect(said[said.length - 1]).toContain("- segundo[[#");
  });

  it("asks the window for an icon and writes it where the caret stands", () => {
    const said = writer();
    answers.push("::dog::");
    caretIn("Hola", 4);
    writing?.slashed(block("icon"));
    expect(said[said.length - 1]?.startsWith("# Hola::dog::\n")).toBe(true);
  });
});
