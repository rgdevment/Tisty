import { afterEach, describe, expect, it } from "vitest";
import { joinedAcross } from "../ui/pageMarks";
import { type PageWriting, type Turns, writtenOn } from "../ui/pageWriting";

const TITLE = '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1>';
const ANTES = '<p data-at="5-11"><span data-src="5-10">Antes</span></p>';

const PAGES = {
  two: {
    body: "# T\n\nHola mundo bonito\n\nSegundo párrafo\n",
    placed: `${TITLE}<p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p><p data-at="24-41"><span data-src="24-40">Segundo párrafo</span></p>`,
  },
  freedItem: {
    body: "# T\n\nAntes\n\nuno\n\n- dos\n",
    placed: `${TITLE}${ANTES}<p data-at="12-16"><span data-src="12-15">uno</span></p><ul data-at="17-23"><li data-at="17-23"><span data-src="19-22">dos</span></li></ul>`,
  },
  freedTask: {
    body: "# T\n\n- [ ] a\n\nb\n\n- [x] c\n",
    placed: `${TITLE}<ul data-at="5-14"><li class="task" data-at="5-14"><span class="check" data-done="false"></span><span data-src="11-12">a</span></li></ul><p data-at="14-16"><span data-src="14-15">b</span></p><ul data-at="17-25"><li class="task" data-at="17-25"><span class="check" data-done="true"></span><span data-src="23-24">c</span></li></ul>`,
  },
  split: {
    body: "# T\n\nSegundo\n\n párrafo\n",
    placed: `${TITLE}<p data-at="5-13"><span data-src="5-12">Segundo</span></p><p data-at="15-24"><span data-src="15-23">párrafo</span></p>`,
  },
  code: {
    body: "# T\n\n```\nuno\n\ndos\n```\n",
    placed: `${TITLE}<figure class="code" data-at="5-21"><pre><code data-src="9-18">uno\n\ndos\n</code></pre></figure>`,
  },
  plain: {
    body: "# T\n\nAntes\n\nTercero\n",
    placed: `${TITLE}${ANTES}<p data-at="12-20"><span data-src="12-19">Tercero</span></p>`,
  },
  picture: {
    body: "# T\n\nAntes\n\n![Rina](rina.png)\n\nTercero\n",
    placed: `${TITLE}${ANTES}<p data-at="12-30"><img src="rina.png" alt="Rina" loading="lazy" data-src="12-29" contenteditable="false"></p><p data-at="31-39"><span data-src="31-38">Tercero</span></p>`,
  },
  pictureAndLines: {
    body: "# T\n\nAntes\n\n![Rina](rina.png)\n\nTercero\nFin\n",
    placed: `${TITLE}${ANTES}<p data-at="12-30"><img src="rina.png" alt="Rina" loading="lazy" data-src="12-29" contenteditable="false"></p><p data-at="31-43"><span data-src="31-38">Tercero</span><br data-src="38-39" contenteditable="false"><span data-src="39-42">Fin</span></p>`,
  },
  rule: {
    body: "# T\n\nAntes\n\n---\n\nTercero\n",
    placed: `${TITLE}${ANTES}<hr data-at="12-16"><p data-at="17-25"><span data-src="17-24">Tercero</span></p>`,
  },
  table: {
    body: "# T\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nTercero\n",
    placed: `${TITLE}${ANTES}<div class="table" data-at="12-42"><table><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table></div><p data-at="43-51"><span data-src="43-50">Tercero</span></p>`,
  },
  aboveTable: {
    body: "# T\n\nPara\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    placed: `${TITLE}<p data-at="5-10"><span data-src="5-9">Para</span></p><div class="table" data-at="11-41"><table><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table></div>`,
  },
  quote: {
    body: "# T\n\nAntes\n\n> una cita\n",
    placed: `${TITLE}${ANTES}<blockquote data-at="12-23"><p data-at="14-23"><span data-src="14-22">una cita</span></p></blockquote>`,
  },
  asked: {
    body: "# T\n\nHola\n\n¿Qué tal?\n",
    placed: `${TITLE}<p data-at="5-10"><span data-src="5-9">Hola</span></p><p data-at="11-23"><span data-src="11-22">¿Qué tal?</span></p>`,
  },
  shortCode: {
    body: "# T\n\n```\nuno\n```\n",
    placed: `${TITLE}<figure class="code" data-at="5-16"><pre><code data-src="9-13">uno\n</code></pre></figure>`,
  },
  sections: {
    body: "Antes\n\n## Sección\n\nDespués\n",
    placed:
      '<p data-at="0-6"><span data-src="0-5">Antes</span></p><h2 id="sección" data-at="7-19"><span data-src="10-18">Sección</span></h2><p data-at="20-29"><span data-src="20-28">Después</span></p>',
  },
  steps: {
    body: "Antes\n\n## Pasos\n\n- uno\n- dos\n",
    placed:
      '<p data-at="0-6"><span data-src="0-5">Antes</span></p><h2 id="pasos" data-at="7-16"><span data-src="10-15">Pasos</span></h2><ul data-at="17-29"><li data-at="17-23"><span data-src="19-22">uno</span></li><li data-at="23-29"><span data-src="25-28">dos</span></li></ul>',
  },
  list: {
    body: "# T\n\nAntes\n\n- uno\n- dos\n- tres\n",
    placed: `${TITLE}${ANTES}<ul data-at="12-31"><li data-at="12-18"><span data-src="14-17">uno</span></li><li data-at="18-24"><span data-src="20-23">dos</span></li><li data-at="24-31"><span data-src="26-30">tres</span></li></ul>`,
  },
};

type Shown = (typeof PAGES)[keyof typeof PAGES];

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = ({ body, placed }: Shown, turns?: Turns) => {
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
    turns,
  });
  return () => text;
};

const words = (said: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(".page") as Node,
    NodeFilter.SHOW_TEXT,
  );
  for (let node = walk.nextNode(); node; node = walk.nextNode())
    if ((node as Text).data.includes(said)) return node as Text;
  throw new Error(`no ${said} on the page`);
};

const wordsEnd = (said: string): [Node, number] => [
  words(said),
  words(said).data.indexOf(said) + said.length,
];
const wordsStart = (said: string): [Node, number] => [words(said), words(said).data.indexOf(said)];
const blockStart = (said: string): [Node, number] => [
  words(said).parentElement?.closest("[data-at]") as Node,
  0,
];

const between = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  return range;
};

const caretAt = ([node, offset]: [Node, number]) => document.getSelection()?.collapse(node, offset);

const asked = (inputType: string, reach?: Range, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => (reach ? [reach] : []) });
  document.querySelector(".page")?.dispatchEvent(event);
};

const pressed = (key: string) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  document.querySelector(".page")?.dispatchEvent(event);
  return event.defaultPrevented;
};

const backFrom = (spot: [Node, number]) => {
  const [node, offset] = spot;
  return between([node, offset - 1], spot);
};

describe("a key at a line's edge joins the lines whole", () => {
  it("takes the line feeds and the line's own marks back to the words above", () => {
    expect(joinedAcross("Hola\n\nSegundo", 6, true)).toEqual({ from: 4, to: 6 });
    expect(joinedAcross("Hola\r\n\r\nSegundo", 8, true)).toEqual({ from: 4, to: 8 });
    expect(joinedAcross("Antes\n\n## Sección", 10, true)).toEqual({ from: 5, to: 10 });
    expect(joinedAcross("Segundo\n\n párrafo", 9, true)).toEqual({ from: 7, to: 9 });
    expect(joinedAcross("Hola\n\n> cita", 8, true)).toEqual({ from: 4, to: 8 });
    expect(joinedAcross("uno\ndos", 4, true)).toEqual({ from: 3, to: 4 });
    expect(joinedAcross("Hola\n\n**mundo**", 8, true)).toEqual({ from: 4, to: 6 });
    expect(joinedAcross("> uno\n>\n> dos", 10, true)).toEqual({ from: 5, to: 10 });
    expect(joinedAcross("uno\n   \nTercero", 8, true)).toEqual({ from: 3, to: 8 });
  });

  it("leaves a place among the words, or on the first line, to the key's own reach", () => {
    expect(joinedAcross("Hola\n\nSegundo", 8, true)).toBeNull();
    expect(joinedAcross("Hola\n\nSeg undo", 10, true)).toBeNull();
    expect(joinedAcross("Hola mundo", 0, true)).toBeNull();
    expect(joinedAcross("Hola\n\n> cita", 9, true)).toBeNull();
  });

  it("takes the line feeds and the next line's marks forward to the words below", () => {
    expect(joinedAcross("bonitoX\n\nSegundo", 7, false)).toEqual({ from: 7, to: 9 });
    expect(joinedAcross("uno\r\n\r\ndos", 3, false)).toEqual({ from: 3, to: 7 });
    expect(joinedAcross("uno\n\n## Dos", 3, false)).toEqual({ from: 3, to: 8 });
    expect(joinedAcross("uno\n\n", 3, false)).toEqual({ from: 3, to: 5 });
    expect(joinedAcross("Antes\n\n\n\n## Sección", 7, false)).toEqual({ from: 7, to: 9 });
    expect(joinedAcross("Hola **mundo**\n\nSegundo", 12, false)).toEqual({ from: 14, to: 16 });
    expect(joinedAcross("> uno\n>\n> dos", 5, false)).toEqual({ from: 5, to: 10 });
    expect(joinedAcross("uno dos\n\ntres", 3, false)).toBeNull();
    expect(joinedAcross("uno", 3, false)).toBeNull();
  });
});

describe("a key on the caret laid at a line's edge", () => {
  it("joins a paragraph whole to the one above, not by one line feed", () => {
    const text = opened(PAGES.two);
    writing?.placeAt(24);
    asked("deleteContentBackward", between(wordsEnd("bonito"), wordsStart("Segundo")));
    expect(text()).toBe("# T\n\nHola mundo bonitoSegundo párrafo\n");
  });

  it("joins the paragraph below whole on a Delete at a paragraph's end", () => {
    const text = opened(PAGES.two);
    writing?.placeAt(22);
    asked("deleteContentForward", between(wordsEnd("bonito"), wordsStart("Segundo")));
    expect(text()).toBe("# T\n\nHola mundo bonitoSegundo párrafo\n");
  });

  it("takes a list's first item, freed of its bullet, back into the paragraph above", () => {
    const text = opened(PAGES.freedItem);
    writing?.placeAt(12);
    asked("deleteContentBackward", between(wordsEnd("Antes"), wordsStart("uno")));
    expect(text()).toBe("# T\n\nAntesuno\n\n- dos\n");
  });

  it("takes a step's words, freed of their box and bullet, back into the step above", () => {
    const text = opened(PAGES.freedTask);
    writing?.placeAt(14);
    asked("deleteContentBackward", between(wordsEnd("a"), wordsStart("b")));
    expect(text()).toBe("# T\n\n- [ ] ab\n- [x] c\n");
  });

  it("joins a paragraph that opens with a space, as Enter leaves it, whole", () => {
    const text = opened(PAGES.split);
    writing?.placeAt(14);
    asked("deleteContentBackward", between(wordsEnd("Segundo"), wordsStart("párrafo")));
    expect(text()).toBe("# T\n\nSegundo párrafo\n");
  });

  it("takes one line feed in a code block, where each one is a line", () => {
    const text = opened(PAGES.code);
    writing?.placeAt(14);
    const code = words("dos");
    asked("deleteContentBackward", between([code, 4], [code, 5]));
    expect(text()).toBe("# T\n\n```\nuno\ndos\n```\n");
  });
});

describe("a join beside a task page's section heading", () => {
  it("leaves a task page's section heading whole on either side of it", () => {
    const text = opened(PAGES.sections);
    document.documentElement.classList.add("untitled");
    writing?.placeAt(10);
    asked("deleteContentBackward", between(wordsEnd("Antes"), wordsStart("Sección")));
    writing?.placeAt(20);
    asked("deleteContentBackward", between(wordsEnd("Sección"), wordsStart("Después")));
    writing?.placeAt(18);
    asked("deleteContentForward", between(wordsEnd("Sección"), wordsStart("Después")));
    document.documentElement.classList.remove("untitled");
    expect(text()).toBe(PAGES.sections.body);
  });
});

describe("a key that comes before the page shows the last one, among the lines", () => {
  it("frees the first item under a task page's section of its bullet", () => {
    const text = opened(PAGES.steps);
    document.documentElement.classList.add("untitled");
    writing?.placeAt(19);
    asked("deleteContentBackward", between(wordsEnd("Pasos"), wordsStart("uno")));
    document.documentElement.classList.remove("untitled");
    expect(text()).toContain("## Pasos\n\n");
    expect(text()).not.toContain("- uno");
    expect(text()).toContain("uno\n");
  });

  it("gives a paragraph back whole on Enter then Backspace before the page draws", () => {
    const text = opened(PAGES.two);
    caretAt(wordsEnd("bonito"));
    asked("insertParagraph");
    asked("deleteContentBackward");
    expect(text()).toBe(PAGES.two.body);
  });

  it("takes the words a format chose before the page draws, though the page's caret fell back", () => {
    const text = opened(PAGES.two);
    document.getSelection()?.setBaseAndExtent(...wordsStart("bonito"), ...wordsEnd("bonito"));
    asked("formatBold");
    expect(text()).toContain("**bonito**");
    caretAt(blockStart("Hola"));
    asked("deleteContentForward");
    expect(text()).not.toContain("bonito");
    expect(text()).toContain("Segundo párrafo");
  });

  it("takes a word back at an item's start without joining it to the item above", () => {
    const text = opened(PAGES.list);
    caretAt(wordsStart("dos"));
    for (const letter of "abc") asked("insertText", undefined, letter);
    asked("deleteWordBackward");
    asked("deleteWordBackward");
    expect(text()).not.toContain("unodos");
    expect(text()).toContain("- uno\n");
  });
});

describe("a key that comes before the page shows the last one", () => {
  const typedAhead = (shown: Shown) => {
    const text = opened(shown);
    caretAt(wordsEnd("Tercero"));
    asked("deleteContentBackward", backFrom(wordsEnd("Tercero")));
    expect(text()).toContain("Tercer\n");
    caretAt(blockStart("Tercero"));
    return text;
  };

  it("takes the letter before the caret kept, though the page dropped its own at the paragraph's start", () => {
    const text = typedAhead(PAGES.plain);
    asked("deleteContentBackward", between(wordsEnd("Antes"), blockStart("Tercero")));
    expect(text()).toBe("# T\n\nAntes\n\nTerce\n");
  });

  it("leaves a picture alone on its line above in place", () => {
    const text = typedAhead(PAGES.picture);
    asked("deleteContentBackward", between(wordsEnd("Antes"), blockStart("Tercero")));
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n\nTerce\n");
  });

  it("leaves a page break above in place", () => {
    const text = typedAhead(PAGES.rule);
    asked("deleteContentBackward", between(wordsEnd("Antes"), blockStart("Tercero")));
    expect(text()).toBe("# T\n\nAntes\n\n---\n\nTerce\n");
  });

  it("still takes a letter under a table", () => {
    const text = typedAhead(PAGES.table);
    asked("deleteContentBackward", between(wordsEnd("Antes"), blockStart("Tercero")));
    expect(text()).toBe("# T\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nTerce\n");
  });

  it("lets the Backspace's own key pass to the words, so a picture above stays", () => {
    const text = typedAhead(PAGES.picture);
    expect(pressed("Backspace")).toBe(false);
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n\nTercer\n");
  });

  it("takes the letter typed last in an item, not its bullet, while the page still shows the item's start", () => {
    const text = opened(PAGES.list);
    caretAt(wordsStart("dos"));
    for (const letter of "abc") asked("insertText", undefined, letter);
    expect(text()).toBe("# T\n\nAntes\n\n- uno\n- abcdos\n- tres\n");
    asked("deleteContentBackward", between(wordsEnd("uno"), wordsStart("dos")));
    expect(text()).toBe("# T\n\nAntes\n\n- uno\n- abdos\n- tres\n");
  });

  it("joins the paragraph whole once the keys ahead reach its start", () => {
    const text = opened(PAGES.two);
    caretAt(wordsStart("Segundo"));
    for (const letter of "abc") asked("insertText", undefined, letter);
    for (let left = 3; left > 0; left--)
      asked("deleteContentBackward", between(wordsEnd("bonito"), wordsStart("Segundo")));
    expect(text()).toBe(PAGES.two.body);
    asked("deleteContentBackward", between(wordsEnd("bonito"), wordsStart("Segundo")));
    expect(text()).toBe("# T\n\nHola mundo bonitoSegundo párrafo\n");
  });
});

describe("a delete the page cannot place yet", () => {
  it("does nothing at a paragraph's end the page does not show, so the table below stays whole", () => {
    const text = opened(PAGES.aboveTable);
    caretAt(wordsEnd("Para"));
    asked("insertText", undefined, "X");
    asked("deleteContentForward");
    expect(text()).toBe("# T\n\nParaX\n\n| a | b |\n|---|---|\n| 1 | 2 |\n");
  });

  it("takes a line left empty under a picture with nothing after it, and the picture stays", () => {
    const text = opened(PAGES.picture);
    caretAt(wordsEnd("Tercero"));
    for (let left = 7; left > 0; left--) asked("deleteContentBackward");
    caretAt(blockStart("Tercero"));
    asked("deleteContentBackward", between(wordsEnd("Antes"), blockStart("Tercero")));
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n");
  });

  it("does nothing on an emptied line the page does not show yet when words follow it closely", () => {
    const text = opened(PAGES.pictureAndLines);
    caretAt(wordsEnd("Tercero"));
    for (let left = 7; left > 0; left--) asked("deleteContentBackward");
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n\n\nFin\n");
    caretAt(blockStart("Fin"));
    asked("deleteContentBackward", between(wordsEnd("Antes"), blockStart("Fin")));
    expect(pressed("Backspace")).toBe(true);
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n\n\nFin\n");
  });

  it("does nothing past the code the page shows, so no line of code loses a mark", () => {
    const text = opened(PAGES.shortCode);
    caretAt([words("uno"), 3]);
    asked("insertParagraph");
    for (const letter of "- ") asked("insertText", undefined, letter);
    asked("deleteContentBackward");
    expect(text()).toBe("# T\n\n```\nuno\n- \n```\n");
  });
});

describe("a key that comes before the page shows an undo that emptied it", () => {
  const titled = {
    body: "# a",
    placed: '<h1 id="a" data-at="0-3"><span data-src="2-3">a</span></h1>',
  };
  const typedTitle = (): Turns => ({
    undone: [{ from: 0, removed: "", added: "# a", at: 0 }],
    redone: [],
  });

  it("writes the next letter as the title, as a Backspace that emptied it would", () => {
    const text = opened(titled, typedTitle());
    caretAt(wordsEnd("a"));
    asked("historyUndo");
    expect(text()).toBe("");
    asked("insertText", undefined, "H");
    asked("insertText", undefined, "o");
    expect(text()).toBe("# Ho");
  });

  it("writes the next letter as the title after a redo that emptied it", () => {
    const text = opened(titled, {
      undone: [],
      redone: [{ from: 0, removed: "", added: "# a", at: 0 }],
    });
    caretAt(wordsEnd("a"));
    asked("historyRedo");
    expect(text()).toBe("");
    asked("insertText", undefined, "H");
    expect(text()).toBe("# H");
  });

  it("writes an untitled page's first letter bare after an undo that emptied it", () => {
    document.documentElement.classList.add("untitled");
    try {
      const text = opened(
        { body: "a", placed: '<p data-at="0-1"><span data-src="0-1">a</span></p>' },
        { undone: [{ from: 0, removed: "", added: "a", at: 0 }], redone: [] },
      );
      caretAt(wordsEnd("a"));
      asked("historyUndo");
      expect(text()).toBe("");
      asked("insertText", undefined, "H");
      expect(text()).toBe("H");
    } finally {
      document.documentElement.classList.remove("untitled");
    }
  });
});

describe("a Backspace that comes before the page shows the letter it takes", () => {
  it("takes the title's letter on the key alone, as WebKit sends no beforeinput on an empty title", () => {
    const text = opened({
      body: "# a",
      placed: '<h1 id="a" data-at="0-3"><span data-src="2-3">a</span></h1>',
    });
    caretAt(wordsEnd("a"));
    asked("deleteContentBackward", backFrom(wordsEnd("a")));
    expect(text()).toBe("");
    (document.querySelector(".page") as Element).innerHTML =
      '<h1 data-title="Título" data-ghost=""></h1>';
    writing?.drawn();
    asked("insertText", undefined, "a");
    expect(text()).toBe("# a");
    expect(pressed("Backspace")).toBe(true);
    expect(text()).toBe("");
  });

  it("takes the title's next letter on the key alone while the page left its caret before the title", () => {
    const text = opened({
      body: "# Hola",
      placed: '<h1 id="hola" data-at="0-6"><span data-src="2-6">Hola</span></h1>',
    });
    caretAt(wordsEnd("Hola"));
    asked("deleteContentBackward", backFrom(wordsEnd("Hola")));
    expect(text()).toBe("# Hol");
    caretAt([document.querySelector(".page") as Node, 0]);
    expect(pressed("Backspace")).toBe(true);
    expect(text()).toBe("# Ho");
  });

  it("leaves the key to the browser while its own caret still has letters to take", () => {
    const text = opened({
      body: "# Hola",
      placed: '<h1 id="hola" data-at="0-6"><span data-src="2-6">Hola</span></h1>',
    });
    caretAt(wordsEnd("Hola"));
    asked("deleteContentBackward", backFrom(wordsEnd("Hola")));
    caretAt([words("Hola"), 2]);
    expect(pressed("Backspace")).toBe(false);
    expect(text()).toBe("# Hol");
  });
});

describe("a right-click on a line just begun", () => {
  const titled = {
    body: "# a1",
    placed: '<h1 id="a1" data-at="0-4"><span data-src="2-4">a1</span></h1>',
  };

  const begunUnderTitle = () => {
    const text = opened(titled);
    caretAt(wordsEnd("a1"));
    asked("insertParagraph");
    const ghost = document.createElement("p");
    ghost.setAttribute("data-ghost", "");
    document.querySelector(".page")?.append(ghost);
    return { text, ghost };
  };

  const rightDown = (on: Element, button: number) =>
    on.dispatchEvent(new MouseEvent("mousedown", { button, bubbles: true, cancelable: true }));

  it("keeps the line begun, so what comes next is written on it", () => {
    const { text, ghost } = begunUnderTitle();
    rightDown(ghost, 2);
    asked("insertText", undefined, "x");
    expect(text()).toBe("# a1\n\nx");
  });

  it("lets a plain click put the caret where it lands, as before", () => {
    const { text, ghost } = begunUnderTitle();
    rightDown(ghost, 0);
    asked("insertText", undefined, "x");
    expect(text()).toBe("# a1x\n\n");
  });
});
