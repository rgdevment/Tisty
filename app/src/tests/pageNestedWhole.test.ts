import { afterEach, describe, expect, it } from "vitest";
import { opened, unwritable } from "../ui/pageNodes";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const BODY =
  "- uno `dos`\n\n  | norte | sur |\n  |---|---|\n  | lejos | cerca |\n- tres\n\n  ```mermaid\n  graph TD\n  A-->B\n  ```\n- cuatro\n\n  ```math\n  y^2\n  ```\n- cinco\n\n> cita\n>\n> | este | oeste |\n> |---|---|\n> | arriba | abajo |\n";

const PLACED =
  '<ul data-at="0-150"><li data-at="0-63"><p data-at="2-12"><span data-src="2-6">uno </span><code data-src="6-11" contenteditable="false">dos</code></p>' +
  '<div class="table" data-at="15-63"><table><thead><tr><th scope="col">norte</th><th scope="col">sur</th></tr></thead><tbody><tr><td>lejos</td><td>cerca</td></tr></tbody></table></div></li>' +
  '<li data-at="63-109"><p data-at="65-70"><span data-src="65-69">tres</span></p><figure class="diagram" data-at="73-108"><pre><code class="language-mermaid">graph TD\nA--&gt;B\n</code></pre></figure></li>' +
  '<li data-at="109-141"><p data-at="111-118"><span data-src="111-117">cuatro</span></p><div class="math" data-at="121-140"><math display="block"><msup><mi>y</mi><mn>2</mn></msup></math></div></li>' +
  '<li data-at="141-150"><p data-at="143-149"><span data-src="143-148">cinco</span></p></li></ul>' +
  '<blockquote data-at="150-211"><p data-at="152-157"><span data-src="152-156">cita</span></p><div class="table" data-at="161-211"><table><thead><tr><th scope="col">este</th><th scope="col">oeste</th></tr></thead><tbody><tr><td>arriba</td><td>abajo</td></tr></tbody></table></div></blockquote>';

const TIGHT = "- uno `dos`\n  | norte | sur |\n  |---|---|\n  | lejos | cerca |\n- tres\n";

const TIGHT_PLACED =
  '<ul data-at="0-69"><li data-at="0-62"><span data-src="2-6">uno </span><code data-src="6-11" contenteditable="false">dos</code>' +
  '<div class="table" data-at="14-62"><table><thead><tr><th scope="col">norte</th><th scope="col">sur</th></tr></thead><tbody><tr><td>lejos</td><td>cerca</td></tr></tbody></table></div></li>' +
  '<li data-at="62-69"><span data-src="64-68">tres</span></li></ul>';

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const written = (body = BODY, placed = PLACED) => {
  let text = body;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  writing.drawn();
  return () => text;
};

const wordsIn = (selector: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(selector) as Element,
    NodeFilter.SHOW_TEXT,
  );
  return walk.nextNode() as Text;
};

const typedAt = (node: Node, offset: number, typed: string) => {
  document.getSelection()?.collapse(node, offset);
  const event = new InputEvent("beforeinput", {
    inputType: "insertText",
    data: typed,
    bubbles: true,
    cancelable: true,
  });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("a table, a diagram or a formula inside a list item or a quote", () => {
  it("is drawn whole, as it is at the top of the page, and the words around it stay open", () => {
    written();
    const whole = Array.from(
      document.querySelectorAll(".page :is(li, blockquote) > :is(.table, figure, .math)"),
    );
    expect(whole.map((one) => `${one.parentElement?.tagName} ${one.className}`)).toEqual([
      "LI table",
      "LI diagram",
      "LI math",
      "BLOCKQUOTE table",
    ]);
    for (const one of whole) expect(one.getAttribute("contenteditable")).toBe("false");
    for (const one of document.querySelectorAll(".page :is(li, blockquote) > p"))
      expect(one.hasAttribute("contenteditable")).toBe(false);
    expect(unwritable(wordsIn("td:last-child"))).toBe(true);
    expect(unwritable(wordsIn(".math"))).toBe(true);
    expect(unwritable(wordsIn("figure.diagram"))).toBe(true);
    expect(unwritable(wordsIn('[data-src="2-6"]'))).toBe(false);
    expect(unwritable(document.querySelector('code[data-src="6-11"]') as Element)).toBe(false);
  });

  it("keeps a piece that ends the item's own line dressed beside it, outside the block", () => {
    written(TIGHT, TIGHT_PLACED);
    const item = document.querySelector("li") as Element;
    const piece = item.querySelector('code[data-src="6-11"]') as Element;
    const table = item.querySelector(".table") as Element;
    expect(table.parentElement).toBe(item);
    expect(table.getAttribute("contenteditable")).toBe("false");
    expect(piece.parentElement).toBe(item);
    expect(unwritable(piece)).toBe(false);
    const laid = item.parentElement as Element;
    table.removeAttribute("contenteditable");
    opened(document, [laid]);
    expect(table.getAttribute("contenteditable")).toBe("false");
    expect(piece.parentElement).toBe(item);
  });

  it("takes no letter typed with the caret in its cells, its drawing or its formula", () => {
    const text = written();
    typedAt(wordsIn("td:last-child"), 2, "X");
    typedAt(wordsIn("blockquote td:last-child"), 2, "X");
    typedAt(wordsIn("figure.diagram"), 3, "X");
    typedAt(wordsIn(".math"), 1, "X");
    expect(text()).toBe(BODY);
  });

  it("goes whole with a choice that ends right after it", () => {
    const text = written();
    const item = document.querySelector("li") as Element;
    const held = document.getSelection();
    held?.setBaseAndExtent(wordsIn('[data-src="2-6"]'), 1, item, item.childNodes.length);
    const event = new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "X",
      bubbles: true,
      cancelable: true,
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(text()).toBe(`- uX${BODY.slice(BODY.indexOf("\n- tres"))}`);
  });
});

const TASK = "## Pasos\n\n- [ ] uno\n- [ ] dos\n";

const TASK_PLACED =
  '<h2 data-at="0-9"><span data-src="3-8">Pasos</span><button type="button" class="drop-section" data-at="0-9"></button></h2>' +
  '<ul data-at="10-30"><li class="task" data-at="10-20"><span class="check"></span><span data-src="16-19">uno</span></li>' +
  '<li class="task" data-at="20-30"><span class="check"></span><span data-src="26-29">dos</span></li></ul>';

const HINTED = "## Pasos\n\n<!-- Lo que haces -->\n";

const HINTED_PLACED =
  '<h2 data-at="0-9"><span data-src="3-8">Pasos</span><button type="button" class="drop-section" data-at="0-32"></button></h2>' +
  '<div class="hint" data-at="10-32"></div>';

const ALIGNED = 'Antes\n\n<div align="center">\n\n| a |\n|---|\n| 1 |\n\n</div>\n';

const ALIGNED_PLACED =
  '<p data-at="0-6"><span data-src="0-5">Antes</span></p>' +
  '<div align="center" data-at="7-50"><div class="table" data-at="29-46"><table><tbody><tr><td>1</td></tr></tbody></table></div></div>';

const LOOSE = "- uno\n\n  | a | b |\n  |---|---|\n  | 1 | 2 |\n- dos\n";

const LOOSE_PLACED =
  '<ul data-at="0-49"><li data-at="0-43"><p data-at="2-6"><span data-src="2-5">uno</span></p>' +
  '<div class="table" data-at="9-43"><table><tbody><tr><td>1</td><td>2</td></tr></tbody></table></div></li>' +
  '<li data-at="43-49"><p data-at="45-49"><span data-src="45-48">dos</span></p></li></ul>';

const choseAll = (from: Node) => {
  document.getSelection()?.collapse(from, 1);
  const key = { key: "a", code: "KeyA", ctrlKey: true, metaKey: true, bubbles: true };
  from.parentElement?.dispatchEvent(new KeyboardEvent("keydown", { ...key, cancelable: true }));
  return document.getSelection()?.getRangeAt(0) as Range;
};

describe("the page's edges and a choice's end beside blocks drawn whole", () => {
  it("leave a task's section label out when the whole page is chosen", () => {
    document.documentElement.classList.add("untitled");
    try {
      written(TASK, TASK_PLACED);
      const chosen = choseAll(wordsIn('[data-src="16-19"]'));
      expect(document.querySelector("ul")?.contains(chosen.startContainer)).toBe(true);
    } finally {
      document.documentElement.classList.remove("untitled");
    }
  });

  it("end a choice over a section's label at its words, not at the end of the hint under it", () => {
    document.documentElement.classList.add("untitled", "shaping");
    try {
      const text = written(HINTED, HINTED_PLACED);
      const label = document.querySelector("h2") as Element;
      document
        .getSelection()
        ?.setBaseAndExtent(wordsIn('[data-src="3-8"]'), 0, label, label.childNodes.length);
      const event = new InputEvent("beforeinput", {
        inputType: "insertText",
        data: "X",
        bubbles: true,
        cancelable: true,
      });
      document.querySelector(".page")?.dispatchEvent(event);
      expect(text()).toBe("## X\n\n<!-- Lo que haces -->\n");
    } finally {
      document.documentElement.classList.remove("untitled", "shaping");
    }
  });

  it("end the whole page after a block drawn whole that holds a table, not inside it", () => {
    written(ALIGNED, ALIGNED_PLACED);
    const chosen = choseAll(wordsIn('[data-src="0-5"]'));
    const sheet = document.querySelector(".page") as Element;
    expect(chosen.endContainer).toBe(sheet);
    expect(chosen.endOffset).toBe(sheet.childNodes.length);
  });

  it("lay a caret asked inside a nested table's source where the item's words end", () => {
    const text = written(LOOSE, LOOSE_PLACED);
    writing?.placeAt(LOOSE.indexOf("| 1 |") + 2);
    const event = new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "X",
      bubbles: true,
      cancelable: true,
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(text()).toBe(LOOSE.replace("- uno", "- unoX"));
  });
});

describe("a delete key pressed before the engine tells where the arrow left the caret", () => {
  const letterAfterCode = () => {
    const letter = document.querySelector("li > code")?.nextSibling as Text;
    expect(letter.data).toBe("\u200B");
    return letter;
  };

  const keyed = (key: string, init: KeyboardEventInit = {}) => {
    const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
    document.querySelector(".page")?.dispatchEvent(event);
    return event;
  };

  const leftInLetter = (offset: number) => {
    keyed("ArrowLeft");
    document.getSelection()?.collapse(letterAfterCode(), offset);
  };

  it.each([0, 1])("takes the code on Backspace from the unseen letter at %i", (offset) => {
    const text = written(TIGHT, TIGHT_PLACED);
    leftInLetter(offset);
    expect(keyed("Backspace").defaultPrevented).toBe(true);
    expect(text()).toBe(TIGHT.replace("`dos`", ""));
  });

  it("stands right after the code for a delete the engine makes itself", () => {
    const text = written(TIGHT, TIGHT_PLACED);
    leftInLetter(1);
    expect(keyed("Backspace", { altKey: true }).defaultPrevented).toBe(false);
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([document.querySelector("li"), 2]);
    expect(text()).toBe(TIGHT);
  });

  it.each([0, 1])(
    "takes a formula whole on Backspace from the unseen letter in it at %i",
    (offset) => {
      const text = written(
        "uno $x$\n",
        '<p data-at="0-8"><span data-src="0-4">uno </span><span class="math" data-src="4-7" contenteditable="false"><math><mi>x</mi></math></span></p>',
      );
      const letter = document.querySelector("math")?.nextSibling as Text;
      expect(letter.data).toBe("\u200B");
      document.getSelection()?.collapse(letter, offset);
      expect(keyed("Backspace").defaultPrevented).toBe(true);
      expect(text()).toBe("uno \n");
    },
  );

  it("stays in the letter for a key that deletes nothing", () => {
    written(TIGHT, TIGHT_PLACED);
    leftInLetter(1);
    const letter = letterAfterCode();
    keyed("x");
    const held = document.getSelection();
    expect([held?.anchorNode, held?.anchorOffset]).toEqual([letter, 1]);
  });
});
