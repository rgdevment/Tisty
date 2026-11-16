import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { movedWords } from "../ui/pageDragging";
import { sourceAt, spotOf } from "../ui/pageSpots";
import { type Hands, type PageWriting, turnsAnew, writtenOn } from "../ui/pageWriting";

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
    translated: (_, plain) => Promise.resolve(plain),
    ask: (_kind, _at, put) => {
      const answer = answers.shift();
      if (answer !== undefined) put(answer);
    },
    blocks: (_at, slash) => menus.push(slash),
    acts: () => {},
    ...more,
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
    expect(writing?.chosen()).toBe("Uno **dos**");
    writing?.cut();
    expect(said[said.length - 1]).toBe("# Hola\n\n tres\n\n- primero\n- segundo\n");
  });

  describe("words carried by the pointer", () => {
    const pointing = (spots: Record<number, [string, number]>) => {
      Object.defineProperty(document, "caretRangeFromPoint", {
        configurable: true,
        value: (x: number) => {
          const [said, at] = spots[x];
          const range = document.createRange();
          range.setStart(piece(said), at);
          return range;
        },
      });
    };
    const mouse = (kind: string, x: number) =>
      (kind === "mousedown" ? piece("Uno ").parentElement : document)?.dispatchEvent(
        new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, clientX: x, detail: 1 }),
      );

    afterEach(() => Reflect.deleteProperty(document, "caretRangeFromPoint"));

    it("moves words back before others, and nowhere when let go inside themselves", () => {
      const text = "uno dos tres";
      const back = movedWords(text, { from: 8, to: 12 }, 0);
      expect(back && text.slice(0, back[0].from) + back[0].insert + text.slice(back[0].to)).toBe(
        "tresuno dos ",
      );
      expect(back?.[1]).toBe(4);
      expect(movedWords(text, { from: 4, to: 7 }, 5)).toBeNull();
    });

    it("moves the words chosen where they are let go, in one step back", () => {
      const said = writer();
      const range = document.createRange();
      range.setStart(piece("Uno "), 0);
      range.setEnd(piece("Uno "), 4);
      document.getSelection()?.removeAllRanges();
      document.getSelection()?.addRange(range);
      pointing({ 10: ["Uno ", 2], 60: ["primero", 0] });
      mouse("mousedown", 10);
      mouse("mousemove", 60);
      mouse("mouseup", 60);
      expect(said[said.length - 1]).toBe("# Hola\n\n**dos** tres\n\n- Uno primero\n- segundo\n");
    });

    it("puts the caret where the press was when the pointer did not move", () => {
      const said = writer();
      const range = document.createRange();
      range.setStart(piece("Uno "), 0);
      range.setEnd(piece("Uno "), 4);
      document.getSelection()?.removeAllRanges();
      document.getSelection()?.addRange(range);
      pointing({ 10: ["Uno ", 2] });
      mouse("mousedown", 10);
      mouse("mouseup", 10);
      expect(said).toEqual([]);
      expect(document.getSelection()?.isCollapsed).toBe(true);
    });
  });

  describe("Ctrl+C and Ctrl+X", () => {
    const clip = (kind: "copy" | "cut", on: Element | null = document.querySelector(".page")) => {
      const put: string[] = [];
      const event = new Event(kind, { bubbles: true, cancelable: true });
      Object.defineProperty(event, "clipboardData", {
        value: { setData: (_: string, said: string) => put.push(said) },
      });
      on?.dispatchEvent(event);
      return { put, taken: event.defaultPrevented };
    };
    const choose = () => {
      const range = document.createRange();
      range.setStart(piece("Uno "), 0);
      range.setEnd(piece("dos"), 3);
      document.getSelection()?.removeAllRanges();
      document.getSelection()?.addRange(range);
    };

    it("copies the Tisty Markdown of the words chosen, leaving the page as it is", () => {
      const said = writer();
      choose();
      expect(clip("copy")).toEqual({ put: ["Uno **dos**"], taken: true });
      expect(said).toEqual([]);
    });

    it("cuts the Tisty Markdown of the words chosen and takes them away", () => {
      const said = writer();
      choose();
      expect(clip("cut")).toEqual({ put: ["Uno **dos**"], taken: true });
      expect(said[said.length - 1]).toBe("# Hola\n\n tres\n\n- primero\n- segundo\n");
    });

    it("leaves the browser's own copy when nothing is chosen", () => {
      writer();
      caretIn("dos", 1);
      document.dispatchEvent(new Event("selectionchange"));
      expect(clip("copy")).toEqual({ put: [], taken: false });
    });
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

  it("keeps the caret off a step's box and its ↗, so ↑ and ↓ go from step to step", () => {
    writer("- [ ] uno\n");
    pageOf(
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span class="check" data-done="false"></span><p data-at="6-9"><span data-src="6-9">uno</span></p><button class="to-part">↗</button></li></ul>',
    );
    writing?.drawn();
    expect(document.querySelector(".check")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector(".to-part")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("li p")?.hasAttribute("contenteditable")).toBe(false);
  });
});

describe("the blocks put on the page written in place", () => {
  it("offers the blocks at a slash on a line just begun, and writes it inside a word", () => {
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

  it("offers the blocks at a slash after a space too, as a word could start there", () => {
    writer();
    caretIn(" tres", 1);
    asked("insertText", "/");
    expect(menus).toEqual([true]);
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

  it("gives the keys back to the page once the window has answered", () => {
    writer();
    const sheet = document.querySelector(".page") as HTMLElement;
    sheet.tabIndex = -1;
    const field = document.body.appendChild(document.createElement("input"));
    field.focus();
    answers.push("![Informe](tisty:doc/mac0-0001)");
    caretIn("Hola", 4);
    writing?.slashed(block("doc"));
    expect(document.activeElement).toBe(sheet);
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

  it("never lays a drawing still queued once the writer stopped, as the page shows the next document", async () => {
    const asked: number[] = [];
    writer(BODY, {
      near: () => {
        asked.push(1);
        return Promise.reject(new Error("no core here"));
      },
    });
    caretIn("Hola", 4);
    document.querySelector(".page")?.dispatchEvent(
      new InputEvent("beforeinput", {
        inputType: "insertText",
        data: "!",
        bubbles: true,
        cancelable: true,
      }),
    );
    writing?.stop();
    await Promise.resolve();
    await Promise.resolve();
    expect(asked).toEqual([]);
  });

  it("keeps a line begun while words arrive from beside the page, moved along with them", () => {
    const said = writer();
    const list = document.querySelector("ul") as Element;
    writing?.begin(BODY.length - 1, "- [ ] ", list);
    writing?.rewrite(BODY.replace("Uno", "Uno y más"));
    asked("insertText", "x");
    expect(said[said.length - 1]).toBe(
      "# Hola\n\nUno y más **dos** tres\n\n- primero\n- segundo\n- [ ] x\n",
    );
  });

  it("keeps a line begun in its place when words change after it", () => {
    const said = writer("Uno\n\nDos\n");
    pageOf(
      '<p data-at="0-3"><span data-src="0-3">Uno</span></p><p data-at="5-8"><span data-src="5-8">Dos</span></p>',
    );
    writing?.drawn();
    writing?.begin(3, "", document.querySelector("p") as Element);
    writing?.rewrite("Uno\n\nDos y tres\n");
    asked("insertText", "x");
    expect(said[said.length - 1]).toBe("Uno\n\nx\n\nDos y tres\n");
  });

  it("joins a step asked past the blank line that ends the list, so the list stays tight", () => {
    const said = writer("- [ ] uno\n\n## Notas\n");
    pageOf(
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li></ul><h2 data-at="11-19"><span data-src="14-19">Notas</span></h2>',
    );
    writing?.drawn();
    writing?.begin(11, "- [ ] ", document.querySelector("ul") as Element);
    asked("insertText", "x");
    expect(said[said.length - 1]).toBe("- [ ] uno\n- [ ] x\n\n## Notas\n");
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

  it("asks the window for an icon and writes it where the caret stands, a space apart", () => {
    const said = writer();
    answers.push("::dog::");
    caretIn("Hola", 4);
    writing?.slashed(block("icon"));
    expect(said[said.length - 1]?.startsWith("# Hola ::dog:: \n")).toBe(true);
    writing?.stop();
    const again = writer();
    answers.push("::cat::");
    caretIn("Uno ", 4);
    writing?.slashed(block("icon"));
    expect(again[again.length - 1]).toContain("Uno ::cat:: **dos**");
  });
});

describe("what the page written in place leaves alone", () => {
  it("lets a field inside the page write its own words", () => {
    const said = writer();
    const field = document.createElement("input");
    document.querySelector("ul")?.append(field);
    const typed = new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "x",
      bubbles: true,
      cancelable: true,
    });
    field.dispatchEvent(typed);
    expect(typed.defaultPrevented).toBe(false);
    expect(said).toEqual([]);
  });

  it("writes nothing on a block that holds no words to write among", () => {
    const said = writer();
    const table = document.createElement("div");
    table.setAttribute("data-at", "8-24");
    table.setAttribute("contenteditable", "false");
    table.textContent = "cell";
    document.querySelector(".page")?.append(table);
    document.getSelection()?.collapse(table.firstChild as Text, 2);
    asked("insertText", "o");
    expect(said).toEqual([]);
  });

  it("takes words changed from beside the page and keeps the caret where it was", () => {
    const said = writer();
    writing?.rewrite(BODY.replace("primero", "primer"));
    expect(said).toEqual([BODY.replace("primero", "primer")]);
    writing?.rewrite(BODY.replace("primero", "primer"));
    expect(said).toHaveLength(1);
  });

  it("begins the first line of an empty sticky note inside it", () => {
    const body = "Uno\n\n> [!IDEA] 2026-11-13 · blue\n>\n";
    const said = writer(body);
    pageOf(
      '<p data-at="0-3"><span data-src="0-3">Uno</span></p><blockquote class="callout" data-at="5-35"></blockquote>',
    );
    document.querySelector("blockquote")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(document.querySelector("blockquote [data-ghost]")).not.toBeNull();
    asked("insertText", "h");
    expect(said[said.length - 1]).toBe("Uno\n\n> [!IDEA] 2026-11-13 · blue\n> h\n");
  });

  it("puts no format on words chosen in code", () => {
    const said = writer("```js\nlet a;\n```\n");
    pageOf(
      '<figure class="code" data-at="0-17"><pre><code data-src="6-13">let a;\n</code></pre></figure>',
    );
    const code = document.querySelector("code")?.firstChild as Text;
    const range = document.createRange();
    range.setStart(code, 0);
    range.setEnd(code, 3);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    expect(writing?.formattable()).toBe(false);
    writing?.format("**", "**");
    expect(said).toEqual([]);
  });
});

describe("a page break on the page written in place", () => {
  const BROKEN = "Uno\n\n---\n\n## Dos\n";
  const DRAWN_BROKEN =
    '<p data-at="0-3"><span data-src="0-3">Uno</span></p><hr data-at="5-9" contenteditable="false">' +
    '<h2 data-at="10-16"><span data-src="13-16">Dos</span></h2>';

  it("is taken away by Backspace at the start of the block after it, that block kept as it is", () => {
    const said = writer(BROKEN);
    pageOf(DRAWN_BROKEN);
    caretIn("Dos", 0);
    asked("deleteContentBackward");
    expect(said).toEqual(["Uno\n\n## Dos\n"]);
  });

  it("is taken away by Delete at the end of the block before it", () => {
    const said = writer(BROKEN);
    pageOf(DRAWN_BROKEN);
    caretIn("Uno", 3);
    asked("deleteContentForward");
    expect(said).toEqual(["Uno\n\n## Dos\n"]);
  });
});

describe("the speller on the page written in place", () => {
  it("leaves code, its name and the keys alone, and words to it", () => {
    writer('```rs title="a.rs"\nfn a() {}\n```\n\nUna `b` frase\n');
    pageOf(
      '<figure class="code" data-at="0-30"><figcaption>a.rs</figcaption><pre><code data-src="20-30">fn a() {}</code></pre></figure>' +
        '<p data-at="32-46"><span data-src="32-36">Una </span><code data-src="36-39" contenteditable="false">b</code><span data-src="39-45"> frase</span></p>',
    );
    writing?.drawn();
    for (const one of document.querySelectorAll("pre, code, figcaption"))
      expect(one.getAttribute("spellcheck")).toBe("false");
    expect(document.querySelector("p")?.hasAttribute("spellcheck")).toBe(false);
  });
});

describe("the keys and words the Markdown view had, on the page", () => {
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
  const last = (said: string[]) => said[said.length - 1] ?? "";

  it("never wraps words chosen before the caret moved away", () => {
    const said = writer();
    choose("primero", 0, 7);
    caretIn("Hola", 4);
    document.dispatchEvent(new Event("selectionchange"));
    asked("formatBold");
    expect(said).toEqual([]);
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });

  it("starts a format with nothing chosen at the first key, and lets it go when pressed again", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("keeps a space before the first key plain, and the format waiting after it", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("i", { ctrlKey: true });
    asked("insertText", " ");
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola *x*\n")).toBe(true);
  });

  it("steps past a format's marks when it is pressed again after its words", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "x");
    key("b", { ctrlKey: true });
    asked("insertText", "y");
    expect(last(said).startsWith("# Hola**x**y\n")).toBe(true);
  });

  it("puts a space typed at the end of bold words past their marks", () => {
    const said = writer();
    caretIn("dos", 3);
    asked("insertText", " ");
    expect(last(said)).toContain("Uno **dos**  tres");
  });

  it("strikes and codes the words chosen with Ctrl, Shift and a letter", () => {
    const said = writer();
    choose("primero", 0, 7);
    key("X", { ctrlKey: true, shiftKey: true });
    expect(last(said)).toContain("- ~~primero~~");
    writing?.stop();
    const coded = writer();
    choose("segundo", 0, 7);
    key("C", { ctrlKey: true, shiftKey: true });
    expect(last(coded)).toContain("- `segundo`");
  });

  it("links with Ctrl and K, the address standing for words when none were chosen", async () => {
    const said = writer();
    caretIn("Hola", 4);
    key("k", { ctrlKey: true });
    const field = document.querySelector<HTMLInputElement>(".page-address");
    if (field) field.value = "https://tisty.app";
    field?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await Promise.resolve();
    expect(last(said).startsWith("# Hola[https://tisty.app](https://tisty.app)\n")).toBe(true);
  });

  it("breaks the page after the block with Ctrl and Enter, never inside code", () => {
    const said = writer();
    caretIn("dos", 1);
    key("Enter", { ctrlKey: true });
    expect(last(said)).toContain("Uno **dos** tres\n\n---\n");
    writing?.stop();
    const coded = writer("```\nuno\n```\n");
    pageOf('<pre data-at="0-12"><code><span data-src="4-7">uno</span></code></pre>');
    writing?.drawn();
    caretIn("uno", 1);
    key("Enter", { ctrlKey: true });
    expect(coded).toEqual([]);
  });

  it("formats from the slash menu as from the bar, at the next key", () => {
    const said = writer();
    caretIn("Hola", 4);
    writing?.slashed(block("bold"));
    asked("insertText", "x");
    expect(last(said).startsWith("# Hola**x**\n")).toBe(true);
  });

  it("offers the tags in use after a hash, and writes the one lit with Tab", () => {
    const said = writer(BODY, { tags: () => ["rina", "roger", "casa"] });
    caretIn(" tres", 5);
    asked("insertText", " #r");
    writing?.drawn();
    const shown = Array.from(document.querySelectorAll(".tag-list [role=option]")).map(
      (one) => one.textContent,
    );
    expect(shown).toEqual(["#rina", "#roger"]);
    key("ArrowDown");
    key("ArrowDown");
    key("Tab");
    expect(last(said)).toContain("Uno **dos** tres #roger \n");
    expect(document.querySelector<HTMLElement>(".tag-list")?.hidden).toBe(true);
  });

  it("lets Enter end the line while no tag is lit, and Escape put the list away", () => {
    const said = writer(BODY, { tags: () => ["rina"] });
    caretIn(" tres", 5);
    asked("insertText", " #r");
    writing?.drawn();
    const list = document.querySelector<HTMLElement>(".tag-list");
    expect(list?.hidden).toBe(false);
    key("Escape");
    expect(list?.hidden).toBe(true);
    asked("insertText", "i");
    writing?.drawn();
    expect(list?.hidden).toBe(false);
    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    document.querySelector(".page")?.dispatchEvent(enter);
    expect(enter.defaultPrevented).toBe(false);
    expect(last(said)).toContain("tres #ri\n");
  });

  it("offers no tags for one already in use, or inside code", () => {
    writer(BODY, { tags: () => ["rina"] });
    caretIn(" tres", 5);
    asked("insertText", " #rina");
    writing?.drawn();
    expect(document.querySelector<HTMLElement>(".tag-list")?.hidden).toBe(true);
    asked("insertText", " `#r");
    writing?.drawn();
    expect(document.querySelector<HTMLElement>(".tag-list")?.hidden).toBe(true);
  });

  it("attaches a picture alone on the clipboard after the block the caret stands in", async () => {
    const files: File[] = [];
    const said = writer(BODY, {
      pasted: (given) => {
        files.push(...given);
        return Promise.resolve("![captura](captura.png)");
      },
    });
    caretIn("dos", 1);
    const event = new Event("paste", { bubbles: true, cancelable: true });
    const picture = new File(["x"], "captura.png", { type: "image/png" });
    Object.defineProperty(event, "clipboardData", {
      value: { files: [picture], getData: () => "" },
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    await Promise.resolve();
    await Promise.resolve();
    expect(files).toEqual([picture]);
    expect(last(said)).toContain("Uno **dos** tres\n\n![captura](captura.png)");
    asked("insertText", "Z");
    expect(last(said)).toContain("![captura](captura.png)\n\nZ\n\n- primero");
  });

  it("leaves words on the clipboard to the paste that translates them", () => {
    const said = writer(BODY, { pasted: () => Promise.resolve("![x](x.png)") });
    caretIn("dos", 1);
    const event = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(event, "clipboardData", {
      value: { files: [new File(["x"], "x.png")], getData: () => "palabras" },
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
    expect(said).toEqual([]);
  });
});

describe("Enter at the end of formatted words on the page", () => {
  it("starts the paragraph past the marks that close a link or a format", () => {
    const linked = "Ver [la guía](https://tisty.app)\n";
    const said = writer(linked);
    pageOf(
      '<p data-at="0-33"><span data-src="0-4">Ver </span><a href="https://tisty.app"><span data-src="5-13">la guía</span></a></p>',
    );
    writing?.drawn();
    caretIn("la guía", 7);
    asked("insertParagraph");
    expect(said[said.length - 1]).toBe("Ver [la guía](https://tisty.app)\n\n\n");
    writing?.stop();
    const bold = writer();
    caretIn("dos", 3);
    asked("insertParagraph");
    expect(bold[bold.length - 1]).toContain("Uno **dos**\n\n tres");
  });
});

describe("what a person found writing on the page", () => {
  const key = (name: string, more: KeyboardEventInit = {}) =>
    document
      .querySelector(".page")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true, ...more }),
      );
  const last = (said: string[]) => said[said.length - 1] ?? "";

  it("takes a change back with Ctrl and Z, and again with Ctrl and Y", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertText", "x");
    key("z", { ctrlKey: true });
    expect(last(said)).toBe(BODY);
    key("y", { ctrlKey: true });
    expect(last(said).startsWith("# Holax\n")).toBe(true);
    key("z", { ctrlKey: true });
    key("Z", { ctrlKey: true, shiftKey: true });
    expect(last(said).startsWith("# Holax\n")).toBe(true);
  });

  it("leaves a list with Enter on an item with nothing written", () => {
    const said = writer("- Uno\n");
    pageOf('<ul data-at="0-6"><li data-at="0-6"><span data-src="2-5">Uno</span></li></ul>');
    writing?.drawn();
    caretIn("Uno", 3);
    asked("insertParagraph");
    asked("insertParagraph");
    writing?.drawn();
    asked("insertText", "x");
    expect(last(said)).toBe("- Uno\n\nx\n");
  });

  it("leaves a quote with Enter on a quote line with nothing written", () => {
    const said = writer("> cita\n");
    pageOf('<blockquote data-at="0-7"><p><span data-src="2-6">cita</span></p></blockquote>');
    writing?.drawn();
    caretIn("cita", 4);
    asked("insertParagraph");
    asked("insertParagraph");
    writing?.drawn();
    asked("insertText", "x");
    expect(last(said)).toBe("> cita\n\nx\n");
  });

  it("writes under a page break just put with Ctrl and Enter", () => {
    const said = writer();
    caretIn("dos", 1);
    key("Enter", { ctrlKey: true });
    writing?.drawn();
    asked("insertText", "S");
    expect(last(said)).toContain("Uno **dos** tres\n\n---\n\nS\n\n- primero");
  });

  it("takes away the space the page does not show at a paragraph's end, not the letter before it", () => {
    const said = writer("Hola mundo\n");
    pageOf('<p data-at="0-10"><span data-src="0-10">Hola mundo</span></p>');
    writing?.drawn();
    caretIn("Hola mundo", 10);
    asked("insertText", " ");
    pageOf('<p data-at="0-11"><span data-src="0-10">Hola mundo</span></p>');
    writing?.drawn();
    asked("deleteContentBackward");
    expect(last(said)).toBe("Hola mundo\n");
  });

  it("leaves out the space a double click takes with a word when it formats it", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("Uno "), 0);
    range.setEnd(piece("Uno "), 4);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    key("b", { ctrlKey: true });
    expect(last(said)).toContain("**Uno** **dos** tres");
  });

  it("turns a paragraph with a line broken inside it into one quote, every line marked", () => {
    const said = writer("Línea uno\nlínea dos\n");
    pageOf(
      '<p data-at="0-20"><span data-src="0-9">Línea uno</span><br><span data-src="10-19">línea dos</span></p>',
    );
    writing?.drawn();
    caretIn("línea dos", 3);
    writing?.slashed(block("quote"));
    expect(last(said)).toBe("> Línea uno\n> línea dos\n");
  });

  it("makes words partly bold wholly bold, with no marks left inside them", () => {
    const said = writer();
    const range = document.createRange();
    range.setStart(piece("Uno "), 0);
    range.setEnd(piece(" tres"), 5);
    document.getSelection()?.removeAllRanges();
    document.getSelection()?.addRange(range);
    key("b", { ctrlKey: true });
    expect(last(said)).toContain("**Uno dos tres**");
  });

  it("keeps a format typed with nothing chosen going across spaces until it is pressed again", () => {
    const said = writer();
    caretIn("Hola", 4);
    key("b", { ctrlKey: true });
    asked("insertText", "uno");
    asked("insertText", " ");
    asked("insertText", "dos");
    key("b", { ctrlKey: true });
    asked("insertText", " ");
    asked("insertText", "tres");
    expect(last(said).startsWith("# Hola**uno dos** tres\n")).toBe(true);
  });
});

describe("what a second look found writing on the page", () => {
  const key = (name: string, more: KeyboardEventInit = {}) =>
    document
      .querySelector(".page")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true, ...more }),
      );
  const last = (said: string[]) => said[said.length - 1] ?? "";

  it("puts the caret after what a redo brings back", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertText", "x");
    key("z", { ctrlKey: true });
    key("y", { ctrlKey: true });
    asked("insertText", "y");
    expect(last(said).startsWith("# Holaxy\n")).toBe(true);
  });

  it("turns a box typed at a line's start into a step", () => {
    const said = writer("Antes\n\n");
    pageOf('<p data-at="0-5"><span data-src="0-5">Antes</span></p>');
    writing?.drawn();
    caretIn("Antes", 5);
    asked("insertParagraph");
    asked("insertText", "[");
    asked("insertText", " ");
    asked("insertText", "]");
    asked("insertText", " ");
    expect(last(said)).toMatch(/^Antes\n\n- \[ \] \n/);
  });

  it("holds a second Enter on a line left blank, so no hidden blank lines pile up", () => {
    const said = writer();
    caretIn("Hola", 4);
    asked("insertParagraph");
    const once = last(said);
    asked("insertParagraph");
    expect(last(said)).toBe(once);
  });

  it("keeps the caret where it was when a change comes from beside the page", () => {
    const said = writer();
    caretIn("Hola", 4);
    writing?.rewrite(BODY.replace("- segundo", "- [x] segundo"));
    asked("insertText", "!");
    expect(last(said).startsWith("# Hola!\n")).toBe(true);
  });

  it("leaves the caret at the end of the words when Delete takes the break after them", () => {
    const body = "Uno\n\n---\n\nDos\n";
    const said = writer(body);
    pageOf(
      '<p data-at="0-3"><span data-src="0-3">Uno</span></p><hr data-at="5-8"><p data-at="10-13"><span data-src="10-13">Dos</span></p>',
    );
    writing?.drawn();
    caretIn("Uno", 3);
    asked("deleteContentForward");
    asked("insertText", "!");
    expect(last(said)).toBe("Uno!\n\nDos\n");
  });

  describe("a picture beside the caret", () => {
    const PICTURED = "Uno\n\n![a](a.png)\n\nDos ![b](b.png) tres\n";
    const drawn = () => {
      const said = writer(PICTURED);
      pageOf(
        '<p data-at="0-3"><span data-src="0-3">Uno</span></p><p data-at="5-16"><img alt="a" data-src="5-16" contenteditable="false"></p><p data-at="18-38"><span data-src="18-22">Dos </span><img alt="b" data-src="22-33" contenteditable="false"><span data-src="33-38"> tres</span></p>',
      );
      writing?.drawn();
      return said;
    };

    it("takes a picture alone on its line away, with its line, from the words above it", () => {
      const said = drawn();
      caretIn("Uno", 3);
      asked("deleteContentForward");
      asked("insertText", "!");
      expect(last(said)).toBe("Uno!\n\nDos ![b](b.png) tres\n");
    });

    it("takes a picture alone on its line away from the words under it", () => {
      const said = drawn();
      caretIn("Dos ", 0);
      asked("deleteContentBackward");
      expect(last(said)).toBe("Uno\n\nDos ![b](b.png) tres\n");
    });

    it("takes a picture among words away on Delete before it, leaving one space", () => {
      const said = drawn();
      caretIn("Dos ", 4);
      asked("deleteContentForward");
      expect(last(said)).toBe("Uno\n\n![a](a.png)\n\nDos tres\n");
    });

    it("takes a picture among words away on Backspace after it, leaving one space", () => {
      const said = drawn();
      caretIn(" tres", 0);
      asked("deleteContentBackward");
      expect(last(said)).toBe("Uno\n\n![a](a.png)\n\nDos tres\n");
    });

    it("still deletes only a letter of words that stand near a picture", () => {
      const said = drawn();
      caretIn(" tres", 5);
      const reach = document.createRange();
      reach.setStart(piece(" tres"), 4);
      reach.setEnd(piece(" tres"), 5);
      const event = new InputEvent("beforeinput", {
        inputType: "deleteContentBackward",
        bubbles: true,
        cancelable: true,
      });
      Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
      document.querySelector(".page")?.dispatchEvent(event);
      expect(last(said)).toBe("Uno\n\n![a](a.png)\n\nDos ![b](b.png) tre\n");
    });
  });
});

describe("Tab on the page written in place", () => {
  const key = (name: string, more: KeyboardEventInit = {}) =>
    document
      .querySelector(".page")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true, ...more }),
      );
  const last = (said: string[]) => said[said.length - 1] ?? "";
  const LIST =
    '<ul data-at="0-8"><li data-at="0-4"><span data-src="2-3">a</span></li><li data-at="4-8"><span data-src="6-7">b</span></li></ul>';

  it("nests an item under the one above, and Shift with Tab takes it back out", () => {
    const said = writer("- a\n- b\n");
    pageOf(LIST);
    writing?.drawn();
    caretIn("b", 1);
    key("Tab");
    expect(last(said)).toBe("- a\n  - b\n");
    key("Tab", { shiftKey: true });
    expect(last(said)).toBe("- a\n- b\n");
  });

  it("keeps the focus on the page when Tab comes outside a list", () => {
    writer();
    caretIn("Hola", 2);
    const tab = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
    document.querySelector(".page")?.dispatchEvent(tab);
    expect(tab.defaultPrevented).toBe(true);
  });

  it("indents inside code with two spaces", () => {
    const said = writer("```\nuno\n```\n");
    pageOf('<pre data-at="0-12"><code><span data-src="4-7">uno</span></code></pre>');
    writing?.drawn();
    caretIn("uno", 0);
    key("Tab");
    expect(last(said)).toBe("```\n  uno\n```\n");
  });
});

describe("a collapsible written on the page", () => {
  const FOLD = ":::details \n\nx\n\n:::\n";
  const drawnFold = (name: string) =>
    pageOf(
      `<details data-at="0-${19 + name.length}"><summary>${name}</summary>` +
        `<p data-at="${13 + name.length}-${14 + name.length}"><span data-src="${13 + name.length}-${14 + name.length}">x</span></p></details>`,
    );

  it("writes its name in the summary, letter after letter across each redraw", () => {
    const said = writer(FOLD);
    drawnFold("");
    writing?.placeAt(11);
    asked("insertText", "R");
    expect(said[said.length - 1]).toBe(":::details R\n\nx\n\n:::\n");
    drawnFold("R");
    writing?.drawn();
    asked("insertText", "e");
    expect(said[said.length - 1]).toBe(":::details Re\n\nx\n\n:::\n");
  });
});

describe("pieces drawn whole on the page", () => {
  const key = (name: string, more: KeyboardEventInit = {}) => {
    const event = new KeyboardEvent("keydown", {
      key: name,
      bubbles: true,
      cancelable: true,
      ...more,
    });
    document.querySelector(".page")?.dispatchEvent(event);
    return event;
  };
  const TAGGED =
    '<p data-at="0-8"><span data-src="0-4">Uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span></p>' +
    '<h2 data-at="10-16"><span data-src="13-16">Sub</span></h2>';

  it("ends a line that ends in a tag after the tag, not in the next block", async () => {
    writer("Uno #eti\n\n## Sub\n");
    pageOf(TAGGED);
    writing?.drawn();
    caretIn("Uno ", 0);
    key("End");
    document.getSelection()?.collapse(piece("Sub"), 0);
    await new Promise((done) => setTimeout(done, 0));
    const held = document.getSelection();
    expect(held?.anchorNode).toBe(document.querySelector("p"));
    expect(held?.anchorOffset).toBe(2);
  });

  it("takes Ctrl with End to the end of the last block written", () => {
    writer("Uno #eti\n\n## Sub\n");
    pageOf(TAGGED);
    writing?.drawn();
    caretIn("Uno ", 0);
    expect(key("End", { ctrlKey: true }).defaultPrevented).toBe(true);
    const held = document.getSelection();
    expect(held?.anchorNode).toBe(piece("Sub"));
    expect(held?.anchorOffset).toBe(3);
  });

  it("stands a caret dropped inside code beside it, on the side of the half it fell in", () => {
    writer("Texto `cod` fin\n");
    pageOf(
      '<p data-at="0-15"><span data-src="0-6">Texto </span><code data-src="6-11" contenteditable="false">cod</code><span data-src="11-15"> fin</span></p>',
    );
    writing?.drawn();
    const dropped = (at: number) => {
      document.getSelection()?.collapse(document.querySelector("code")?.firstChild as Text, at);
      document.dispatchEvent(new Event("selectionchange"));
      const held = document.getSelection();
      expect(held?.anchorNode).toBe(document.querySelector("p"));
      return held?.anchorOffset;
    };
    expect(dropped(1)).toBe(1);
    expect(dropped(2)).toBe(2);
  });
});

describe("Ctrl+Z across a turn to the Markdown", () => {
  const undo = () =>
    document
      .querySelector(".page")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }),
      );

  it("walks back what the Markdown wrote as one step, then what the page wrote", () => {
    const turns = turnsAnew();
    writer("Uno\n", { turns });
    pageOf('<p data-at="0-3"><span data-src="0-3">Uno</span></p>');
    writing?.drawn();
    caretIn("Uno", 3);
    asked("insertText", "s");
    writing?.stop();
    const said = writer("Unos dos\n", { turns });
    pageOf('<p data-at="0-8"><span data-src="0-8">Unos dos</span></p>');
    writing?.drawn();
    undo();
    expect(said[said.length - 1]).toBe("Unos\n");
    undo();
    expect(said[said.length - 1]).toBe("Uno\n");
  });

  it("walks back nothing of another document", () => {
    writer("Uno\n");
    caretIn("primero", 7);
    asked("insertText", "s");
    writing?.stop();
    const said = writer("Otro\n", { turns: turnsAnew() });
    undo();
    expect(said).toEqual([]);
  });
});

describe("telling what was written once a block is left", () => {
  it("tells once the caret lands in another block, at Enter, the window left and the end, never within one", () => {
    let told = 0;
    writer(BODY, {
      settle: () => {
        told += 1;
      },
    });
    const key = (name: string) =>
      document
        .querySelector(".page")
        ?.dispatchEvent(new KeyboardEvent("keydown", { key: name, bubbles: true }));
    const landed = (said: string) => {
      caretIn(said, 1);
      document.dispatchEvent(new Event("selectionchange"));
    };
    caretIn("dos", 1);
    asked("insertText", "x");
    key("ArrowLeft");
    landed("dos");
    expect(told).toBe(0);
    document.querySelector(".page")?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    landed(" tres");
    expect(told).toBe(0);
    key("ArrowDown");
    landed("primero");
    expect(told).toBe(1);
    asked("insertParagraph");
    expect(told).toBe(2);
    window.dispatchEvent(new Event("blur"));
    expect(told).toBe(3);
    writing?.stop();
    writing = null;
    expect(told).toBe(4);
  });
});

describe("an empty step left behind", () => {
  it("is taken away when the caret goes to another block, and the words around stay", () => {
    const said = writer("- [ ] uno\n- [ ] \n\nFin\n");
    pageOf(
      '<ul data-at="0-17"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li><li class="task" data-at="10-17"></li></ul>' +
        '<p data-at="18-21"><span data-src="18-21">Fin</span></p>',
    );
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1], 0);
    document.querySelector(".page")?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    caretIn("Fin", 1);
    document.dispatchEvent(new Event("selectionchange"));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
  });

  it("stays while a word is still to come in it", () => {
    const said = writer("- [ ] uno\n- [ ] \n\nFin\n");
    pageOf(
      '<ul data-at="0-17"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li><li class="task" data-at="10-17"></li></ul>' +
        '<p data-at="18-21"><span data-src="18-21">Fin</span></p>',
    );
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelectorAll("li")[1], 0);
    document.dispatchEvent(new Event("selectionchange"));
    expect(said).toEqual([]);
  });

  it("is taken away with Escape, as the caret leaving it would", () => {
    const said = writer("- [ ] uno\n- [ ] \n\nFin\n");
    pageOf(
      '<ul data-at="0-17"><li class="task" data-at="0-10"><span data-src="6-9">uno</span></li><li class="task" data-at="10-17"></li></ul>' +
        '<p data-at="18-21"><span data-src="18-21">Fin</span></p>',
    );
    writing?.drawn();
    writing?.placeAt(16);
    document
      .querySelector(".page")
      ?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(said[said.length - 1]).toBe("- [ ] uno\n\nFin\n");
  });
});

describe("a key at the edge of a block drawn whole", () => {
  // The browser's own reach for Backspace or Delete skips a block it cannot write in, so it spans the block whole.
  const across = (
    inputType: string,
    from: readonly [string, number],
    to: readonly [string, number],
  ) => {
    const reach = document.createRange();
    reach.setStart(piece(from[0]), from[1]);
    reach.setEnd(piece(to[0]), to[1]);
    const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
    Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
    document.querySelector(".page")?.dispatchEvent(event);
  };
  const BACK = "deleteContentBackward";
  const FORTH = "deleteContentForward";
  const TABLE =
    '<div class="table" data-at="8-27" contenteditable="false"><table><tbody><tr><td>1</td></tr></tbody></table></div>';

  // name | body | page | the caret | the browser's reach | key
  it.each([
    [
      "Backspace under a table",
      "Antes.\n\n| A |\n| --- |\n| 1 |\n\nTras.\n",
      `<p data-at="0-6"><span data-src="0-6">Antes.</span></p>${TABLE}<p data-at="29-34"><span data-src="29-34">Tras.</span></p>`,
      ["Tras.", 0],
      [
        ["Antes.", 6],
        ["Tras.", 0],
      ],
      BACK,
    ],
    [
      "Delete over a table",
      "Antes.\n\n| A |\n| --- |\n| 1 |\n\nTras.\n",
      `<p data-at="0-6"><span data-src="0-6">Antes.</span></p>${TABLE}<p data-at="29-34"><span data-src="29-34">Tras.</span></p>`,
      ["Antes.", 6],
      [
        ["Antes.", 6],
        ["Tras.", 0],
      ],
      FORTH,
    ],
    [
      "Backspace under a table with Windows line ends",
      "Antes.\r\n\r\n| A |\r\n| --- |\r\n| 1 |\r\n\r\nTras.\r\n",
      '<p data-at="0-6"><span data-src="0-6">Antes.</span></p><div class="table" data-at="10-31" contenteditable="false"><table></table></div><p data-at="35-40"><span data-src="35-40">Tras.</span></p>',
      ["Tras.", 0],
      [
        ["Antes.", 6],
        ["Tras.", 0],
      ],
      BACK,
    ],
    [
      "Backspace under a code block",
      "Uno.\n\n```js\nlet a;\n```\n\nTras.\n",
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p><figure class="code" data-at="6-22"><pre><code><span data-src="12-18">let a;</span></code></pre><button class="code-lang" contenteditable="false">js</button></figure><p data-at="24-29"><span data-src="24-29">Tras.</span></p>',
      ["Tras.", 0],
      [
        ["let a;", 6],
        ["Tras.", 0],
      ],
      BACK,
    ],
    [
      "Backspace at a code block's first line",
      "Uno.\n\n```js\nlet a;\n```\n\nTras.\n",
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p><figure class="code" data-at="6-22"><pre><code><span data-src="12-18">let a;</span></code></pre><button class="code-lang" contenteditable="false">js</button></figure><p data-at="24-29"><span data-src="24-29">Tras.</span></p>',
      ["let a;", 0],
      [
        ["Uno.", 4],
        ["let a;", 0],
      ],
      BACK,
    ],
    [
      "Delete at a code block's last line, past its language",
      "Uno.\n\n```js\nlet a;\n```\n\nTras.\n",
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p><figure class="code" data-at="6-22"><pre><code><span data-src="12-18">let a;</span></code></pre><button class="code-lang" contenteditable="false">js</button></figure><p data-at="24-29"><span data-src="24-29">Tras.</span></p>',
      ["let a;", 6],
      [
        ["let a;", 6],
        ["Tras.", 0],
      ],
      FORTH,
    ],
    [
      "Backspace under a folded section",
      "Antes.\n\n:::details Pliegue\n\ndentro\n\n:::\n\nTras.\n",
      '<p data-at="0-6"><span data-src="0-6">Antes.</span></p><details data-at="8-39"><summary>Pliegue</summary><p data-at="28-34"><span data-src="28-34">dentro</span></p></details><p data-at="41-46"><span data-src="41-46">Tras.</span></p>',
      ["Tras.", 0],
      [
        ["dentro", 6],
        ["Tras.", 0],
      ],
      BACK,
    ],
    [
      "Backspace at the start of a fold's words",
      "Antes.\n\n:::details Pliegue\n\ndentro\n\n:::\n\nTras.\n",
      '<p data-at="0-6"><span data-src="0-6">Antes.</span></p><details data-at="8-39" open><summary>Pliegue</summary><p data-at="28-34"><span data-src="28-34">dentro</span></p></details><p data-at="41-46"><span data-src="41-46">Tras.</span></p>',
      ["dentro", 0],
      [
        ["Antes.", 6],
        ["dentro", 0],
      ],
      BACK,
    ],
    [
      "Delete over a note",
      "Antes.\n\n> [!NOTE]\n> nota\n",
      '<p data-at="0-6"><span data-src="0-6">Antes.</span></p><blockquote data-callout="note" data-at="8-24"><p><span data-src="20-24">nota</span></p></blockquote>',
      ["Antes.", 6],
      [
        ["Antes.", 6],
        ["nota", 0],
      ],
      FORTH,
    ],
  ] as const)("takes nothing on %s", (_, body, html, at, reach, kind) => {
    const said = writer(body);
    pageOf(html);
    writing?.drawn();
    caretIn(at[0], at[1]);
    across(kind, reach[0], reach[1]);
    expect(said).toEqual([]);
  });

  it("takes nothing on Delete at a step's end over a task's section", () => {
    document.documentElement.classList.add("untitled");
    const said = writer("- [ ] paso\n\n## Notas\n");
    pageOf(
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span data-src="6-10">paso</span></li></ul><h2 data-at="12-20"><span data-src="15-20">Notas</span></h2>',
    );
    writing?.drawn();
    caretIn("paso", 4);
    across(FORTH, ["paso", 4], ["Notas", 0]);
    document.documentElement.classList.remove("untitled");
    expect(said).toEqual([]);
  });

  it("takes a picture alone in its paragraph away on Backspace under it, never joining words into it", () => {
    const said = writer("![a](x.png)\n\nTras.\n");
    pageOf(
      '<p data-at="0-11"><img data-src="0-11" contenteditable="false" src="x.png" alt="a"></p><p data-at="13-18"><span data-src="13-18">Tras.</span></p>',
    );
    writing?.drawn();
    caretIn("Tras.", 0);
    const reach = document.createRange();
    reach.setStart(document.querySelector("p") as Element, 1);
    reach.setEnd(piece("Tras."), 0);
    const event = new InputEvent("beforeinput", {
      inputType: BACK,
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(said).toEqual(["Tras.\n"]);
  });

  it("still joins two paragraphs, as any editor does", () => {
    const said = writer("Uno.\n\nDos.\n");
    pageOf(
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p><p data-at="6-10"><span data-src="6-10">Dos.</span></p>',
    );
    writing?.drawn();
    caretIn("Dos.", 0);
    across(BACK, ["Uno.", 4], ["Dos.", 0]);
    expect(said[said.length - 1]).toBe("Uno.Dos.\n");
  });
});

describe("a caret left on a page break", () => {
  const BREAK =
    '<p data-at="0-3"><span data-src="0-3">Uno</span></p><hr data-at="5-8"><p data-at="10-13"><span data-src="10-13">Dos</span></p>';

  it("writes a key on a new line after the break, never on the break's own line", () => {
    const said = writer("Uno\n\n---\n\nDos\n");
    pageOf(BREAK);
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelector(".page") as Element, 1);
    asked("insertText", "Q");
    expect(said[said.length - 1]).toBe("Uno\n\n---\n\nQ\n\nDos\n");
  });

  it("lets Enter and Backspace there take nothing", () => {
    const said = writer("Uno\n\n---\n\nDos\n");
    pageOf(BREAK);
    writing?.drawn();
    document.getSelection()?.collapse(document.querySelector(".page") as Element, 1);
    asked("insertParagraph");
    asked("deleteContentBackward");
    expect(said).toEqual([]);
  });
});

describe("Enter and Backspace at the start of a block's words", () => {
  it("opens a line above a paragraph with Enter, and the next key is written there", () => {
    const said = writer("Uno.\n\nDos.\n");
    pageOf(
      '<p data-at="0-4"><span data-src="0-4">Uno.</span></p><p data-at="6-10"><span data-src="6-10">Dos.</span></p>',
    );
    writing?.drawn();
    caretIn("Dos.", 0);
    asked("insertParagraph");
    expect(said).toEqual([]);
    asked("insertText", "X");
    expect(said[said.length - 1]).toBe("Uno.\n\nX\n\nDos.\n");
  });

  it("takes a step's box away with Backspace at the start of its words", () => {
    const said = writer("- [ ] paso a\n- [x] paso b\n");
    pageOf(
      '<ul data-at="0-25"><li class="task" data-at="0-13"><span data-src="6-12">paso a</span></li><li class="task" data-at="13-25"><span data-src="19-25">paso b</span></li></ul>',
    );
    writing?.drawn();
    caretIn("paso a", 0);
    asked("deleteContentBackward");
    expect(said[said.length - 1]).toBe("- paso a\n- [x] paso b\n");
  });
});

describe("a page emptied again", () => {
  it("leaves no title's mark behind when its one letter is taken away", () => {
    const said = writer("# X\n");
    pageOf('<h1 data-at="0-3"><span data-src="2-3">X</span></h1>');
    writing?.drawn();
    caretIn("X", 1);
    const letter = document.createRange();
    letter.setStart(piece("X"), 0);
    letter.setEnd(piece("X"), 1);
    const event = new InputEvent("beforeinput", {
      inputType: "deleteContentBackward",
      bubbles: true,
      cancelable: true,
    });
    Object.defineProperty(event, "getTargetRanges", { value: () => [letter] });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(said[said.length - 1]).toBe("");
  });
});

describe("a page break from the slash menu", () => {
  it("leaves what is typed next on a line under the break", () => {
    const said = writer("Fin.\n");
    pageOf('<p data-at="0-4"><span data-src="0-4">Fin.</span></p>');
    writing?.drawn();
    caretIn("Fin.", 4);
    asked("insertParagraph");
    writing?.slashed(block("rule"));
    asked("insertText", "Z");
    expect(said[said.length - 1]).toMatch(/^Fin\.\n\n---\n\nZ\n+$/);
  });
});

describe("a block drawn from a sample on the page written in place", () => {
  it("puts a formula that reads at once and leaves what is typed next under it", () => {
    const said = writer("Fin.\n");
    pageOf('<p data-at="0-4"><span data-src="0-4">Fin.</span></p>');
    writing?.drawn();
    caretIn("Fin.", 4);
    writing?.slashed(block("math"));
    expect(said[said.length - 1]).toBe("Fin.\n\n```math\na^2 + b^2 = c^2\n```\n");
    asked("insertText", "Z");
    expect(said[said.length - 1]).toMatch(/^Fin\.\n\n```math\na\^2 \+ b\^2 = c\^2\n```\n\nZ\n*$/);
  });

  it("puts a diagram and a widget with words of their own, never an empty fence", () => {
    for (const key of ["mermaid", "widget"]) {
      const said = writer("Fin.\n");
      pageOf('<p data-at="0-4"><span data-src="0-4">Fin.</span></p>');
      writing?.drawn();
      caretIn("Fin.", 4);
      writing?.slashed(block(key));
      const now = said[said.length - 1] ?? "";
      expect(now).toContain(`\`\`\`${key}\n`);
      expect(now).not.toContain(`\`\`\`${key}\n\n\`\`\``);
    }
  });
});

describe("blocks asked for where the words cannot take them", () => {
  const FENCED = "Uno\n\n```js\nlet a\n```\n";
  const FENCED_HTML =
    '<p data-at="0-3"><span data-src="0-3">Uno</span></p>' +
    '<figure class="code" data-at="5-20"><pre><code class="language-js"><span data-src="11-16">let a</span></code></pre></figure>';

  it("puts stages asked for inside code under the code, leaving its lines whole", () => {
    const said = writer(FENCED);
    pageOf(FENCED_HTML);
    writing?.drawn();
    caretIn("let a", 5);
    writing?.slashed(block("todo"));
    expect(said[said.length - 1]).toBe("Uno\n\n```js\nlet a\n```\n\n- [ ] \n");
  });

  it("lays a caret asked inside a widget's words on a line begun under the widget", () => {
    const body = "Uno\n\n```widget\n<b>1</b>\n```\n";
    const said = writer(body);
    pageOf(
      '<p data-at="0-3"><span data-src="0-3">Uno</span></p><figure class="widget" data-at="5-27"><div>1</div></figure>',
    );
    writing?.drawn();
    writing?.placeAt(body.indexOf("<b>"));
    writing?.slashed(block("todo"));
    expect(said[said.length - 1]).toBe(`${body}\n- [ ] \n`);
  });
});

describe("every block from the slash menu, wherever the caret stands", () => {
  const fences = (text: string) => text.split("\n").filter((line) => /^\s*```/.test(line)).length;
  const WIDGET = "Uno\n\n```widget\n<b>1</b>\n```\n\n> cita\n";
  const WIDGET_HTML =
    '<p data-at="0-3"><span data-src="0-3">Uno</span></p>' +
    '<figure class="widget" data-at="5-27"><div>1</div></figure>' +
    '<blockquote data-at="29-35"><p data-at="31-35"><span data-src="31-35">cita</span></p></blockquote>';
  const CODE = "Uno\n\n```js\nlet a\n```\n";
  const CODE_HTML =
    '<p data-at="0-3"><span data-src="0-3">Uno</span></p>' +
    '<figure class="code" data-at="5-20"><pre><code class="language-js"><span data-src="11-16">let a</span></code></pre></figure>';
  const places: [string, string, string, () => void][] = [
    ["inside code", CODE, CODE_HTML, () => caretIn("let a", 3)],
    ["inside a widget's words", WIDGET, WIDGET_HTML, () => writing?.placeAt(WIDGET.indexOf("<b>"))],
    [
      "on a line begun under a widget",
      WIDGET,
      WIDGET_HTML,
      () => writing?.begin(27, "", document.querySelector("figure") as Element),
    ],
    ["inside a quote", WIDGET, WIDGET_HTML, () => caretIn("cita", 4)],
  ];
  for (const [where, body, html, laid] of places)
    for (const one of SLASHED.filter((each) => !("ask" in each.put) && !("wrap" in each.put)))
      it(`keeps every fence whole with ${one.key} ${where}`, () => {
        const said = writer(body);
        pageOf(html);
        writing?.drawn();
        laid();
        writing?.slashed(one);
        const now = said[said.length - 1] ?? body;
        for (const fence of body.match(/```[^\n]*\n[\s\S]*?\n```/g) ?? [])
          expect(now).toContain(fence);
        expect(fences(now) % 2).toBe(0);
        expect(now).not.toMatch(/^(?:[-*>]|\d+\.)[^\n]*```/m);
      });
});

describe("a block asked for on a line just begun", () => {
  it("turns only the new line, never the heading under it", () => {
    const said = writer("Uno\n\n## Dos\n");
    pageOf(
      '<p data-at="0-3"><span data-src="0-3">Uno</span></p><h2 data-at="5-11"><span data-src="8-11">Dos</span></h2>',
    );
    writing?.drawn();
    caretIn("Uno", 3);
    asked("insertParagraph");
    writing?.slashed(block("todo"));
    asked("insertText", "Z");
    expect(said[said.length - 1]).toBe("Uno\n\n- [ ] Z\n\n## Dos\n");
  });

  it("turns only the new line under a widget, never its closing fence", () => {
    const body = "Uno\n\n```widget\n<b>1</b>\n```\n\n## Dos\n";
    const said = writer(body);
    pageOf(
      '<p data-at="0-3"><span data-src="0-3">Uno</span></p><figure class="widget" data-at="5-27"><div>1</div></figure><h2 data-at="29-35"><span data-src="32-35">Dos</span></h2>',
    );
    writing?.drawn();
    writing?.begin(27, "", document.querySelector("figure") as Element);
    asked("insertText", "a");
    asked("insertParagraph");
    writing?.slashed(block("todo"));
    asked("insertText", "Z");
    expect(said[said.length - 1]).toBe(
      "Uno\n\n```widget\n<b>1</b>\n```\n\na\n\n- [ ] Z\n\n## Dos\n",
    );
  });
});
