import { readFileSync } from "node:fs";
import { join } from "node:path";
import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const PAGES = join(process.cwd(), "src/tests/pages");

// Pages as the app drew them, caught from a real window, so what the core really marks is what is met.
const real = (name: string) => ({
  html: readFileSync(join(PAGES, `${name}.html`), "utf8"),
  body: readFileSync(join(PAGES, `${name}.md`), "utf8"),
});

let writing: PageWriting | null = null;
const answers: string[] = [];

afterEach(() => {
  writing?.stop();
  writing = null;
  answers.length = 0;
  document.body.innerHTML = "";
});

const opened = (name: string) => {
  const { html, body } = real(name);
  document.body.innerHTML = html;
  let text = body;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
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
    blocks: () => {},
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const block = (key: string) => SLASHED.find((one) => one.key === key) as Slashed;

// The browser names the place a key acts on from the caret it shows, as Chrome's target ranges do.
const asked = (inputType: string, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  const held = document.getSelection();
  const shown = held?.rangeCount ? held.getRangeAt(0).cloneRange() : null;
  Object.defineProperty(event, "getTargetRanges", { value: () => (shown ? [shown] : []) });
  document.querySelector(".page")?.dispatchEvent(event);
};

// Chrome moves a caret laid before a block's words into their first letter, which jsdom leaves be.
const settled = () => {
  const held = document.getSelection();
  const range = held?.rangeCount ? held.getRangeAt(0) : null;
  const next = range?.startContainer.childNodes[range.startOffset];
  if (!held || !next || next.nodeType !== Node.ELEMENT_NODE) return;
  const letter = document.createTreeWalker(next, NodeFilter.SHOW_TEXT).nextNode();
  if (letter) held.collapse(letter, 0);
};

const caretAfter = (said: string) => {
  const piece = Array.from(document.querySelectorAll("[data-src]")).find(
    (one) => one.textContent === said,
  )?.firstChild as Text;
  document.getSelection()?.collapse(piece, piece.length);
};

describe("blocks asked for on pages the app drew", () => {
  it("puts stages under a table on a line of their own, apart from the list under it", () => {
    const text = opened("table");
    writing?.placeAt(text().indexOf("| 1 | 2 |"));
    writing?.slashed(block("todo"));
    asked("insertText", "Z");
    expect(text()).toContain("| 1 | 2 |\n\n- [ ] Z\n\n- primero");
  });

  it("leaves what is typed after a document card on a line under the card", () => {
    const text = opened("nested");
    caretAfter("Uno dos tres");
    asked("insertParagraph");
    answers.push("![Rina](tisty:doc/abc)");
    writing?.slashed(block("doc"));
    asked("insertText", "Z");
    expect(text()).toContain("![Rina](tisty:doc/abc)\n\nZ\n\n- primero");
  });

  it("leaves what is typed after a card put at a paragraph's end on a line under the card", () => {
    const text = opened("nested");
    caretAfter("Uno dos tres");
    answers.push("![Rina](tisty:doc/abc)");
    writing?.slashed(block("doc"));
    asked("insertText", "Z");
    expect(text()).toContain("Uno dos tres\n\n![Rina](tisty:doc/abc)\n\nZ\n\n- primero");
  });

  it("starts a new item after an item that holds others, and writes there once it is drawn", () => {
    const text = opened("nested");
    caretAfter("segundo");
    asked("insertParagraph");
    expect(text()).toBe(real("nestedAfter").body);
    document.body.innerHTML = real("nestedAfter").html;
    writing?.drawn();
    settled();
    asked("insertText", "Z");
    expect(text()).toContain("- segundo\n- Z\n  - anidado");
  });
});

describe("a link written alone on its line", () => {
  it("leaves what is typed next on a line under it, as it is drawn as a card", async () => {
    const text = opened("nested");
    caretAfter("Uno dos tres");
    asked("insertParagraph");
    writing?.format("[", "]()");
    const field = document.querySelector<HTMLInputElement>(".page-address") as HTMLInputElement;
    field.value = "tisty.app";
    field.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await Promise.resolve();
    asked("insertText", "Z");
    expect(text()).toContain("[https://tisty.app](https://tisty.app)\n\nZ\n\n- primero");
  });
});

describe("words written inside a collapsible", () => {
  it("go under its name with Enter, and the collapsible stays whole", () => {
    const text = opened("foldEmpty");
    const name = document.querySelector("summary")?.firstChild as Text;
    document.getSelection()?.collapse(name, name.length);
    asked("insertParagraph");
    asked("insertText", "Z");
    expect(text()).toContain(":::details Plegable\n\nZ\n\n");
    expect(text()).toMatch(/Z\n+:::\n/);
  });

  it("go under its name when its empty body is clicked", () => {
    const text = opened("foldEmpty");
    const fold = document.querySelector("details") as HTMLDetailsElement;
    fold.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    asked("insertText", "Z");
    expect(text()).toContain(":::details Plegable\n\nZ\n\n");
  });
});

describe("words a menu hands back after it closes", () => {
  it("land where the caret stood when it opened, though a click moved it since", () => {
    const text = opened("nested");
    caretAfter("Uno dos tres");
    asked("insertParagraph");
    const resume = writing?.held();
    writing?.placeAt(text().indexOf("fin") + 3);
    resume?.();
    writing?.typed("/Tab");
    expect(text()).toContain("Uno dos tres\n\n/Tab");
    expect(text()).not.toContain("fin/Tab");
  });
});

describe("a heading's marks typed on a task's page", () => {
  it("stay words, as every heading there is a section of the task", () => {
    document.documentElement.classList.add("untitled");
    try {
      const text = opened("nested");
      caretAfter("Uno dos tres");
      asked("insertParagraph");
      asked("insertText", "#");
      asked("insertText", " ");
      asked("insertText", "Z");
      expect(text()).toContain(`Uno dos tres\n\n${"\\"}# Z`);
    } finally {
      document.documentElement.classList.remove("untitled");
    }
  });
});

describe("keys at the edge of a collapsible that holds words", () => {
  it("join nothing to it, from the block after it or the one before it", () => {
    const text = opened("foldFilled");
    const was = text();
    const after = Array.from(document.querySelectorAll("[data-src]")).find(
      (one) => one.textContent === "despues",
    )?.firstChild as Text;
    document.getSelection()?.collapse(after, 0);
    asked("deleteContentBackward");
    caretAfter("Uno dos tres");
    asked("deleteContentForward");
    expect(text()).toBe(was);
  });
});

describe("a caret laid where the page draws nothing to write in", () => {
  it("stands where the words begin when it comes back from the properties", () => {
    document.body.innerHTML =
      '<article class="page"><dl class="front" data-at="0-18"></dl><h1 data-at="19-26"><span data-src="21-25">Pan.</span></h1></article>';
    let text = "---\ntags: [a]\n---\n\n# Pan.\n";
    writing = writtenOn(document, {
      text: () => text,
      write: (next) => {
        text = next;
      },
      shown: () => {},
      redraw: () => {},
      near: () => Promise.reject(new Error("no core here")),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    writing.drawn();
    writing.placeAt(5);
    asked("insertText", "Z");
    expect(text).toBe("---\ntags: [a]\n---\n\n# ZPan.\n");
  });

  it("opens no line in the words while only a task's labels are asked for", () => {
    document.documentElement.classList.add("shaping");
    try {
      const text = opened("widget");
      writing?.placeAt(text().indexOf("<b>"));
      expect(document.querySelector("[data-ghost]")).toBeNull();
    } finally {
      document.documentElement.classList.remove("shaping");
    }
  });
});
