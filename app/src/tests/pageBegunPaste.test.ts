import { afterEach, describe, expect, it } from "vitest";
import { wired, worn } from "../ui/dressing";
import { begunOpening } from "../ui/pageBlocks";
import { foldedInto, type Turn, turnsAnew } from "../ui/pageWriter";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;
const writes: string[] = [];
let swap: (next: string) => void = () => {};

afterEach(() => {
  writing?.stop();
  writing = null;
  document.documentElement.classList.remove("untitled");
  document.body.innerHTML = "";
});

const settled = () => new Promise((done) => setTimeout(done, 0));

describe("the opening kept for a paste on a line just begun", () => {
  it.each([
    ["# ", "Hola", "# "],
    ["# ", "# Título\n\nTexto", ""],
    ["# ", "uno\ndos", ""],
    ["\n\n", "- a\n- b", "\n\n"],
    ["- ", "texto", "- "],
    ["- ", "uno\ndos", ""],
    ["- ", "- a", ""],
    ["* ", "uno\ndos", ""],
    ["- [ ] ", "dos", "- [ ] "],
    ["- [ ] ", "- dos\n- tres", ""],
    ["- [x] ", "uno\ndos", ""],
    ["1. ", "texto", "1. "],
    ["1. ", "uno\ndos", ""],
    ["1) ", "- a", ""],
    ["# ", "1. Introducción", "# "],
    ["# ", "- algo", "# "],
    ["# ", "> cita", "# "],
    ["# ", "## Sub", ""],
    ["# ", "Título\n\nTexto", ""],
    ["Antes\n- [ ] ", "- dos\n- tres", "Antes\n"],
    ["Antes\n", "- a", "Antes\n"],
  ])("keeps «%j» for the paste «%j» as «%j»", (lead, said, expected) => {
    expect(begunOpening(lead, said)).toBe(expected);
  });
});

type Held = { answer: (said: string) => void };

const onPage = (body: string, placed: string, translated?: (plain: string) => Promise<string>) => {
  let text = body;
  swap = (next) => {
    text = next;
  };
  writes.length = 0;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  const reach = {
    body: "",
    onDoc: () => {},
    onBegin: (at: number, seed: string, near: Element) => writing?.begin(at, seed, near),
  };
  worn(document, () => true, reach);
  wired(document, () => reach);
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
      writes.push(next);
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: (_, plain) => (translated ? translated(plain) : Promise.resolve(plain)),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return () => text;
};

const sheet = () => document.querySelector(".page") as HTMLElement;
const key = (inputType: string, data: string | null = null) =>
  sheet().dispatchEvent(
    new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
  );
const pasted = (plain: string, html = "") => {
  const event = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  Object.defineProperty(event, "dataTransfer", {
    value: { getData: (kind: string) => (kind === "text/html" ? html : plain) },
  });
  sheet().dispatchEvent(event);
};
const press = (on: Element | null) => {
  for (const kind of ["mousedown", "mouseup", "click"])
    on?.dispatchEvent(
      new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, clientY: 900 }),
    );
};

const emptyPage = (translated?: (plain: string) => Promise<string>) => {
  const text = onPage("", "", translated);
  press(document.querySelector(".page > h1[data-title]"));
  return text;
};

describe("a paste on an empty new page", () => {
  it("makes one line its title", async () => {
    const text = emptyPage();
    pasted("Hola");
    await settled();
    expect(text()).toBe("# Hola");
  });

  it("lands several lines with no empty heading before them", async () => {
    const text = emptyPage();
    pasted("# Título\n\nTexto");
    await settled();
    expect(text()).toBe("# Título\n\nTexto");
  });

  it("writes the plain words when their translation fails", async () => {
    const text = emptyPage(() => Promise.reject(new Error("no translation")));
    pasted("Hola");
    await settled();
    expect(text()).toBe("# Hola");
  });

  it("puts the caret after the pasted words", async () => {
    const text = emptyPage();
    pasted("Hola");
    await settled();
    key("insertText", "!");
    expect(text()).toBe("# Hola!");
  });

  it("keeps the words typed while the translation is still in flight", async () => {
    const held: Held = { answer: () => {} };
    const text = emptyPage(
      (plain) =>
        new Promise((done) => {
          held.answer = () => done(plain);
        }),
    );
    pasted("Hola");
    key("insertText", "!");
    held.answer("");
    await settled();
    expect(text()).toBe("# Hola!");
  });
});

const ITEM = "- a\n";
const ITEM_PLACED = '<ul data-at="0-4"><li data-at="0-4"><span data-src="2-3">a</span></li></ul>';

describe("a paste on an item just begun", () => {
  it("keeps the words typed while the translation is still in flight", async () => {
    const held: Held = { answer: () => {} };
    const text = onPage(
      ITEM,
      ITEM_PLACED,
      (plain) =>
        new Promise((done) => {
          held.answer = () => done(plain);
        }),
    );
    const words = document.querySelector('[data-src="2-3"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, words.length);
    key("insertParagraph");
    pasted("x");
    key("insertText", "!");
    held.answer("");
    await settled();
    expect(text()).toBe("- a\n- x!\n");
  });

  it("keeps the item's opening", async () => {
    const text = onPage(ITEM, ITEM_PLACED);
    const words = document.querySelector('[data-src="2-3"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, words.length);
    key("insertParagraph");
    pasted("x");
    await settled();
    expect(text()).toBe("- a\n- x\n");
  });
});

describe("a paste after Shift+Enter", () => {
  it("lands on the new line of the same paragraph", async () => {
    const text = onPage("Hola\n", '<p data-at="0-5"><span data-src="0-4">Hola</span></p>');
    const words = document.querySelector('[data-src="0-4"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, words.length);
    sheet().dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Enter",
        shiftKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
    pasted("x");
    await settled();
    expect(text()).toBe("Hola\\\nx\n");
  });
});

describe("a paste on a line just begun, taken back and brought back", () => {
  it("on an empty page leaves the text as it was with one undo, and returns with redo", async () => {
    const text = emptyPage();
    pasted("Hola");
    await settled();
    expect(text()).toBe("# Hola");
    key("historyUndo");
    expect(text()).toBe("");
    key("historyRedo");
    expect(text()).toBe("# Hola");
  });

  it("after Shift+Enter leaves the text as it was with one undo, and returns with redo", async () => {
    const text = onPage("Hola\n", '<p data-at="0-5"><span data-src="0-4">Hola</span></p>');
    const words = document.querySelector('[data-src="0-4"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, words.length);
    sheet().dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Enter",
        shiftKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
    pasted("x");
    await settled();
    const pastedText = text();
    key("historyUndo");
    expect(text()).toBe("Hola\n");
    key("historyRedo");
    expect(text()).toBe(pastedText);
  });

  it("writes nothing for a paste of only spaces and adds no step to undo", async () => {
    const text = emptyPage();
    pasted("  ");
    await settled();
    expect(text()).toBe("");
    expect(writes).toEqual([]);
    key("historyUndo");
    expect(writes).toEqual([]);
  });
});

const STEP = "- [ ] uno\n";
const STEP_PLACED =
  '<ul data-at="0-10"><li class="task" data-at="0-10"><span class="check" data-done="false"></span><span data-src="6-9">uno</span></li></ul>';

const begunAfter = (
  body: string,
  placed: string,
  src: string,
  translated?: (plain: string) => Promise<string>,
) => {
  const text = onPage(body, placed, translated);
  const words = document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;
  document.getSelection()?.collapse(words, words.length);
  key("insertParagraph");
  return text;
};

describe("a paste on a step just begun", () => {
  it("leaves no empty step before pasted items", async () => {
    const text = begunAfter(STEP, STEP_PLACED, "6-9");
    pasted("- dos\n- tres");
    await settled();
    expect(text()).toBe("- [ ] uno\n- dos\n- tres\n");
  });

  it("keeps the step's mark for a single line", async () => {
    const text = begunAfter(STEP, STEP_PLACED, "6-9");
    pasted("dos");
    await settled();
    expect(text()).toBe("- [ ] uno\n- [ ] dos\n");
  });
});

describe("a paste of lines on an item just begun", () => {
  it("leaves no empty item behind", async () => {
    const text = begunAfter(ITEM, ITEM_PLACED, "2-3");
    pasted("uno\ndos");
    await settled();
    expect(text()).not.toMatch(/^- $/m);
    expect(text()).toContain("uno\ndos");
  });
});

describe("a paste on an empty new page that opens like a block", () => {
  it.each([
    ["1. Introducción", "# 1. Introducción"],
    ["- algo", "# - algo"],
    ["> cita", "# > cita"],
    ["# Título", "# Título"],
  ])("lands «%j» as «%j»", async (said, expected) => {
    const text = emptyPage();
    pasted(said);
    await settled();
    expect(text()).toBe(expected);
  });

  it("keeps no empty heading before a title and its text", async () => {
    const text = emptyPage();
    pasted("Título\n\nTexto");
    await settled();
    expect(text()).toBe("Título\n\nTexto");
  });
});

describe("a paste whose translation comes back empty", () => {
  it("leaves the text as it was on an empty page, with no step to undo", async () => {
    const text = emptyPage(() => Promise.resolve(""));
    pasted("Hola");
    await settled();
    expect(text()).toBe("");
    key("historyUndo");
    expect(text()).toBe("");
  });

  it("changes nothing on an item that Enter already wrote", async () => {
    const text = begunAfter(ITEM, ITEM_PLACED, "2-3", () => Promise.resolve(""));
    const before = text();
    pasted("dos");
    await settled();
    expect(text()).toBe(before);
  });
});

describe("a paste answered after the text changed under it", () => {
  it("does not take the opening back when the translation is empty", async () => {
    const held: Held = { answer: () => {} };
    const text = emptyPage(
      () =>
        new Promise((done) => {
          held.answer = done;
        }),
    );
    pasted("dos");
    swap("Otro");
    held.answer("");
    await settled();
    expect(text()).toBe("Otro");
  });

  it("lands where its words were, keeping what was written meanwhile", async () => {
    const held: Held = { answer: () => {} };
    const text = emptyPage(
      () =>
        new Promise((done) => {
          held.answer = done;
        }),
    );
    pasted("dos");
    swap("# Otro");
    held.answer("dos");
    await settled();
    expect(text()).toContain("Otro");
    expect(text()).toContain("dos");
  });
});

describe("the opening and the paste written in it", () => {
  const opening: Turn = { from: 0, removed: "", added: "# ", at: 0, after: 2 };
  const inside: Turn = { from: 2, removed: "", added: "Hola", at: 2, after: 6 };

  it("become one step when the paste lands inside the opening", () => {
    const turns = turnsAnew();
    turns.undone.push(opening, inside);
    foldedInto(turns, opening, "# Hola");
    expect(turns.undone).toEqual([{ from: 0, removed: "", added: "# Hola", at: 0, after: 6 }]);
  });

  it("leaves both when another step came between them", () => {
    const turns = turnsAnew();
    const between: Turn = { from: 2, removed: "", added: "!", at: 2, after: 3 };
    turns.undone.push(opening, between, inside);
    foldedInto(turns, opening, "# Hola!");
    expect(turns.undone).toEqual([opening, between, inside]);
  });

  it("leaves both when the paste lands outside the opening", () => {
    const turns = turnsAnew();
    const outside: Turn = { from: 5, removed: "", added: "Hola", at: 5, after: 9 };
    turns.undone.push(opening, outside);
    foldedInto(turns, opening, "# xxxHola");
    expect(turns.undone).toEqual([opening, outside]);
  });
});
