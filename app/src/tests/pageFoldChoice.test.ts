import { afterEach, describe, expect, it, vi } from "vitest";
import { TISTY_MARKDOWN } from "../ui/clipboarding";
import { colonsKept, colonsMoved } from "../ui/pageColons";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (body: string, placed: string) => {
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

const asked = (inputType: string, reach: Range | null, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => (reach ? [reach] : []) });
  document.querySelector(".page")?.dispatchEvent(event);
};

const wordsOf = (said: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(".page") as Node,
    NodeFilter.SHOW_TEXT,
  );
  for (let node = walk.nextNode(); node; node = walk.nextNode())
    if ((node as Text).data === said) return node as Text;
  throw new Error(`no "${said}" on the page`);
};

const rangeOf = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  return range;
};

const chose = (from: [Node, number], to: [Node, number]) => {
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(rangeOf(from, to));
};

const TABLE_PLACED =
  '<table><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead>' +
  "<tbody><tr><td>1</td><td>2</td></tr></tbody></table>";

const FOLDED =
  "# T\n\nAntes y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::\n\nDespués\n";
const FOLDED_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-18"><span data-src="5-17">Antes y más</span></p>' +
  '<details data-at="19-86"><summary>Plegable</summary>' +
  '<p data-at="40-50"><span data-src="40-49">Contenido</span></p>' +
  `<div class="table" data-at="51-81">${TABLE_PLACED}</div></details>` +
  '<p data-at="87-96"><span data-src="87-95">Después</span></p>';

const FOLDED_LAST = FOLDED.replace("\n\nDespués\n", "\n");
const FOLDED_LAST_PLACED = FOLDED_PLACED.replace(
  '<p data-at="87-96"><span data-src="87-95">Después</span></p>',
  "",
);

const NESTED =
  "# T\n\nAntes\n\n:::details Fuera\n\nUno y dos\n\n:::details Dentro\n\nDos\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::\n\n:::\n\nDespués\n";
const NESTED_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-11"><span data-src="5-10">Antes</span></p>' +
  '<details data-at="12-105"><summary>Fuera</summary>' +
  '<p data-at="30-40"><span data-src="30-39">Uno y dos</span></p>' +
  '<details data-at="41-100"><summary>Dentro</summary>' +
  '<p data-at="60-64"><span data-src="60-63">Dos</span></p>' +
  `<div class="table" data-at="65-95">${TABLE_PLACED}</div></details></details>` +
  '<p data-at="106-115"><span data-src="106-114">Después</span></p>';

const tableEnd = (selector: string): [Node, number] => {
  const table = document.querySelector(selector) as Element;
  return [table, table.childNodes.length];
};

const foldEnd = (selector: string): [Node, number] => {
  const fold = document.querySelector(selector) as Element;
  return [fold, fold.childNodes.length];
};

const summaryEnd = (): [Node, number] => {
  const name = document.querySelector(".page details > summary")?.firstChild as Text;
  return [name, name.length];
};

describe("a choice from the words above a fold to the fold's end", () => {
  it.each([
    [
      "at its table's end, named by the engine as the choice",
      () => tableEnd("details .table"),
      false,
    ],
    [
      "at its table's end, named by the engine as far as the fold's name",
      () => tableEnd("details .table"),
      true,
    ],
    [
      "at the fold's own end, named by the engine as far as the fold's name",
      () => foldEnd(".page > details"),
      true,
    ],
  ])("ending %s, is typed over with the fold and its closing line", (_, end, cut) => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    const to = end();
    chose([words, 6], to);
    asked("insertText", rangeOf([words, 6], cut ? summaryEnd() : to), "X");
    expect(text()).toBe("# T\n\nAntes X\n\nDespués\n");
  });

  it("is taken by Backspace with the fold and its closing line, the words after it apart", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    const to = tableEnd("details .table");
    chose([words, 6], to);
    asked("deleteContentBackward", rangeOf([words, 6], to));
    expect(text()).toBe("# T\n\nAntes \n\nDespués\n");
  });

  it("is taken by Delete at the page's end, though the engine names it as far as the fold's name", () => {
    const text = opened(FOLDED_LAST, FOLDED_LAST_PLACED);
    const words = wordsOf("Antes y más");
    chose([words, 6], tableEnd("details .table"));
    asked("deleteContentForward", rangeOf([words, 6], summaryEnd()));
    expect(text()).toBe("# T\n\nAntes \n");
  });

  it("from inside an outer fold, takes only the inner fold it leaves, closing line and all", () => {
    const text = opened(NESTED, NESTED_PLACED);
    const words = wordsOf("Uno y dos");
    const to = tableEnd("details details .table");
    chose([words, 4], to);
    asked("insertText", rangeOf([words, 4], to), "X");
    expect(text()).toBe("# T\n\nAntes\n\n:::details Fuera\n\nUno X\n\n:::\n\nDespués\n");
  });
});

describe("a choice inside a fold to the fold's own end", () => {
  it("takes the fold's last table and keeps its closing line", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Contenido");
    const to = foldEnd(".page > details");
    chose([words, 5], to);
    asked("insertText", rangeOf([words, 5], to), "X");
    expect(text()).toBe(FOLDED.replace("nido\n\n| a | b |\n|---|---|\n| 1 | 2 |", "X"));
  });

  it("from inside the inner fold to the outer one's end, keeps both closing lines", () => {
    const text = opened(NESTED, NESTED_PLACED);
    const words = wordsOf("Dos");
    const to = foldEnd(".page > details");
    chose([words, 1], to);
    asked("insertText", rangeOf([words, 1], to), "X");
    expect(text()).toBe(NESTED.replace("os\n\n| a | b |\n|---|---|\n| 1 | 2 |", "X"));
  });
});

const keyed = (key: string) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

const clipped = (kind: "copy" | "cut") => {
  const put: string[] = [];
  const event = new Event(kind, { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", {
    value: {
      setData: (type: string, said: string) => type === TISTY_MARKDOWN && put.push(said),
    },
  });
  document.querySelector(".page")?.dispatchEvent(event);
  return put;
};

const draggedTo = (from: [Node, number], to: [Node, number]) => {
  const spots: Record<number, [Node, number]> = { 10: from, 60: to };
  Object.defineProperty(document, "caretRangeFromPoint", {
    configurable: true,
    value: (x: number) => rangeOf(spots[x], spots[x]),
  });
  const mouse = (kind: string, x: number, on: EventTarget) =>
    on.dispatchEvent(
      new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, clientX: x, detail: 1 }),
    );
  mouse("mousedown", 10, from[0].parentElement as Element);
  mouse("mousemove", 60, document);
  mouse("mouseup", 60, document);
  Reflect.deleteProperty(document, "caretRangeFromPoint");
};

const FOLD_CHOSEN =
  "y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::";

const CODED =
  "# T\n\nAntes\n\n:::details Plegable\n\nUno\n\n```js\nconst a = 1;\nconst b = 2;\n```\n\n:::\n\nDespués\n";
const CODED_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-11"><span data-src="5-10">Antes</span></p>' +
  '<details data-at="12-79"><summary>Plegable</summary>' +
  '<p data-at="33-37"><span data-src="33-36">Uno</span></p>' +
  '<figure class="code" data-at="38-73"><pre><code class="language-js" data-src="44-70">' +
  "const a = 1;\nconst b = 2;\n</code></pre></figure></details>" +
  '<p data-at="80-89"><span data-src="80-88">Después</span></p>';

describe("a choice inside a fold's last code block to the fold's end", () => {
  it("keeps the code's closing fence on a line of its own", () => {
    const text = opened(CODED, CODED_PLACED);
    const code = document.querySelector(".page code")?.firstChild as Text;
    const to = foldEnd(".page > details");
    chose([code, 20], to);
    asked("deleteContentBackward", rangeOf([code, 20], to));
    expect(text()).toBe(CODED.replace("const b = 2;\n```", "const b\n```"));
  });
});

describe("a letter taken inside a fold", () => {
  it("reads no more of the fold than the engine names", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Contenido");
    document.getSelection()?.collapse(words, 5);
    const read: string[] = [];
    const cloned = Range.prototype.cloneContents;
    const spy = vi.spyOn(Range.prototype, "cloneContents").mockImplementation(function (
      this: Range,
    ) {
      const part = cloned.call(this);
      read.push(part.textContent ?? "");
      return part;
    });
    asked("deleteContentBackward", rangeOf([words, 4], [words, 5]));
    spy.mockRestore();
    expect(text()).toBe(FOLDED.replace("Contenido", "Contnido"));
    expect(read.filter((one) => one.includes("12"))).toEqual([]);
  });
});

describe("the same choice pressed or clipped", () => {
  it("is taken by Backspace on the key, the fold with it", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    chose([words, 6], tableEnd("details .table"));
    keyed("Backspace");
    expect(text()).toBe("# T\n\nAntes \n\nDespués\n");
  });

  it("is copied with the fold's closing line, so the fold pasted elsewhere closes", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    chose([words, 6], tableEnd("details .table"));
    expect(clipped("copy")).toEqual([FOLD_CHOSEN]);
    expect(text()).toBe(FOLDED);
  });

  it("is dragged with the fold's closing line, leaving none behind", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    chose([words, 6], tableEnd("details .table"));
    draggedTo([words, 8], [wordsOf("Después"), 7]);
    expect(text()).toBe(`# T\n\nAntes \n\nDespués${FOLD_CHOSEN}\n`);
  });

  it("is written over by a composed letter, the fold with it", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    chose([words, 6], tableEnd("details .table"));
    const sheet = document.querySelector(".page") as Element;
    sheet.dispatchEvent(new CompositionEvent("compositionstart", { data: "", bubbles: true }));
    sheet.dispatchEvent(new CompositionEvent("compositionend", { data: "é", bubbles: true }));
    expect(text()).toBe("# T\n\nAntes é\n\nDespués\n");
  });

  it("is cut with the fold's closing line, leaving none behind", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    chose([words, 6], tableEnd("details .table"));
    expect(clipped("cut")).toEqual([FOLD_CHOSEN]);
    expect(text()).toBe("# T\n\nAntes \n\nDespués\n");
  });
});

const TWO_IN_A_FOLD =
  "# T\n\nAntes y más\n\n:::details Plegable\n\nContenido\n\nOtro\n\n:::\n\nDespués\n";
const TWO_IN_A_FOLD_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-18"><span data-src="5-17">Antes y más</span></p>' +
  '<details data-at="19-61"><summary>Plegable</summary>' +
  '<p data-at="40-50"><span data-src="40-49">Contenido</span></p>' +
  '<p data-at="51-56"><span data-src="51-55">Otro</span></p></details>' +
  '<p data-at="62-71"><span data-src="62-70">Después</span></p>';

describe("a choice from inside a fold out past its end", () => {
  it("keeps the fold's closing line, and the words after the fold stay outside it", () => {
    const text = opened(TWO_IN_A_FOLD, TWO_IN_A_FOLD_PLACED);
    const inside = wordsOf("Otro");
    const after = wordsOf("Después");
    chose([inside, 2], [after, 3]);
    asked("insertText", rangeOf([inside, 2], [after, 3]), "X");
    expect(text()).toBe(TWO_IN_A_FOLD.replace("Otro\n\n:::\n\nDespués", "OtX\n\n:::\n\npués"));
  });

  it("is taken by Backspace without the fold's closing line", () => {
    const text = opened(TWO_IN_A_FOLD, TWO_IN_A_FOLD_PLACED);
    const inside = wordsOf("Otro");
    const after = wordsOf("Después");
    chose([inside, 2], [after, 3]);
    asked("deleteContentBackward", rangeOf([inside, 2], [after, 3]));
    expect(text()).toBe(TWO_IN_A_FOLD.replace("Otro\n\n:::\n\nDespués", "Ot\n\n:::\n\npués"));
  });
});

const NAMED_NEST =
  "# T\n\nAntes y más\n\n:::details Fuera\n\n:::details Dentro\n\nx\n\n:::\n\nMás\n\n:::\n\nDespués\n";
const NAMED_NEST_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-18"><span data-src="5-17">Antes y más</span></p>' +
  '<details data-at="19-74"><summary>Fuera</summary>' +
  '<details data-at="37-63"><summary>Dentro</summary>' +
  '<p data-at="56-58"><span data-src="56-57">x</span></p></details>' +
  '<p data-at="64-69"><span data-src="64-68">Más</span></p></details>' +
  '<p data-at="75-84"><span data-src="75-83">Después</span></p>';

const UNDONE = "# T\n\nAntes Xnido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nDespués\n";

const intoTheFold = () => {
  const words = wordsOf("Antes y más");
  const inside = wordsOf("Contenido");
  chose([words, 6], [inside, 5]);
  return rangeOf([words, 6], [inside, 5]);
};

const pastedOver = async (reach: Range, said: string) => {
  const event = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [reach] });
  Object.defineProperty(event, "dataTransfer", {
    value: { getData: (kind: string) => (kind === "text/plain" ? said : "") },
  });
  document.querySelector(".page")?.dispatchEvent(event);
  for (let turn = 0; turn < 30; turn++) await Promise.resolve();
};

describe("a choice that takes a fold's name and not its end", () => {
  it("undoes the fold when typed over, and what it left of the fold stays as blocks", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    asked("insertText", intoTheFold(), "X");
    expect(text()).toBe(UNDONE);
  });

  it.each([
    ["Backspace", "deleteContentBackward"],
    ["Delete", "deleteContentForward"],
  ])("undoes the fold on %s", (_, kind) => {
    const text = opened(FOLDED, FOLDED_PLACED);
    asked(kind, intoTheFold());
    expect(text()).toBe(UNDONE.replace("Antes X", "Antes "));
  });

  it("undoes the fold when words are pasted over it", async () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    await pastedOver(intoTheFold(), "Hola");
    expect(text()).toBe(UNDONE.replace("Antes X", "Antes Hola"));
  });

  it("undoes the fold when a composed letter is written over it", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    intoTheFold();
    const sheet = document.querySelector(".page") as Element;
    sheet.dispatchEvent(new CompositionEvent("compositionstart", { data: "", bubbles: true }));
    sheet.dispatchEvent(new CompositionEvent("compositionend", { data: "é", bubbles: true }));
    expect(text()).toBe(UNDONE.replace("Antes X", "Antes é"));
  });

  it("is copied as a fold that closes, and cut leaving the fold undone", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    intoTheFold();
    const chosen = "y más\n\n:::details Plegable\n\nConte\n\n:::";
    expect(clipped("copy")).toEqual([chosen]);
    expect(clipped("cut")).toEqual([chosen]);
    expect(text()).toBe(UNDONE.replace("Antes X", "Antes "));
  });

  it("is dragged as a fold that closes, leaving the fold undone", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    intoTheFold();
    draggedTo([wordsOf("Antes y más"), 8], [wordsOf("Después"), 7]);
    expect(text()).toBe(
      UNDONE.replace("Antes X", "Antes ").replace(
        "Después\n",
        "Despuésy más\n\n:::details Plegable\n\nConte\n\n:::\n",
      ),
    );
  });

  it("ending in the fold's name, takes the whole name and undoes the fold", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    const name = wordsOf("Plegable");
    chose([words, 6], [name, 4]);
    asked("insertText", rangeOf([words, 6], [name, 4]), "X");
    expect(text()).toBe(
      FOLDED.replace("y más\n\n:::details Plegable", "X").replace("| 1 | 2 |\n\n:::", "| 1 | 2 |"),
    );
  });

  it("ending right before the fold's name, leaves the fold whole", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const words = wordsOf("Antes y más");
    const name = wordsOf("Plegable");
    chose([words, 6], [name, 0]);
    asked("insertText", rangeOf([words, 6], [name, 0]), "X");
    expect(text()).toBe(FOLDED.replace("y más", "X"));
  });

  it("starting in the fold's name and running past it, takes the whole fold", () => {
    const text = opened(TWO_IN_A_FOLD, TWO_IN_A_FOLD_PLACED);
    const name = wordsOf("Plegable");
    const after = wordsOf("Después");
    chose([name, 3], [after, 3]);
    asked("insertText", rangeOf([name, 3], [after, 3]), "X");
    expect(text()).toBe("# T\n\nAntes y más\n\nXpués\n");
  });

  it("over an inner fold that is not the outer one's last block, undoes the outer one", () => {
    const text = opened(NAMED_NEST, NAMED_NEST_PLACED);
    const words = wordsOf("Antes y más");
    const inner = wordsOf("x");
    chose([words, 6], [inner, 1]);
    asked("insertText", rangeOf([words, 6], [inner, 1]), "X");
    expect(text()).toBe("# T\n\nAntes X\n\nMás\n\nDespués\n");
  });
});

describe("a choice from inside a fold out past its end, clipped", () => {
  it("is copied without the fold's closing line, and cut keeping it", () => {
    const text = opened(TWO_IN_A_FOLD, TWO_IN_A_FOLD_PLACED);
    chose([wordsOf("Otro"), 2], [wordsOf("Después"), 3]);
    expect(clipped("copy")).toEqual(["ro\n\nDes"]);
    expect(clipped("cut")).toEqual(["ro\n\nDes"]);
    expect(text()).toBe(TWO_IN_A_FOLD.replace("Otro\n\n:::\n\nDespués", "Ot\n\n:::\n\npués"));
  });
});

describe("a fold's name", () => {
  it("takes the letter typed in it", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const name = wordsOf("Plegable");
    document.getSelection()?.collapse(name, 4);
    asked("insertText", rangeOf([name, 4], [name, 4]), "X");
    expect(text()).toBe(FOLDED.replace("Plegable", "PlegXable"));
  });

  it("gives up the letter taken back in it", () => {
    const text = opened(FOLDED, FOLDED_PLACED);
    const name = wordsOf("Plegable");
    document.getSelection()?.collapse(name, 4);
    asked("deleteContentBackward", rangeOf([name, 3], [name, 4]));
    expect(text()).toBe(FOLDED.replace("Plegable", "Pleable"));
  });
});

describe("the words a choice takes across a fold's edge, read from the text", () => {
  const typedOver = (text: string, from: string, to: string, insert = "X") => {
    const edit = colonsKept(text, {
      from: text.indexOf(from),
      to: text.indexOf(to) + to.length,
      insert,
    });
    return text.slice(0, edit.from) + edit.insert + text.slice(edit.to);
  };

  it("keeps a mark that ends the fold's last line, as an address's closing bracket", () => {
    const text =
      "Antes y más\n\n:::details X\n\nContenido\n\nVer <https://x.com>\n\n:::\n\nDespués";
    expect(typedOver(text, "y más", "Conte")).toBe("Antes Xnido\n\nVer <https://x.com>\n\nDespués");
  });

  it("finds a fold right under a heading, with no blank line between them", () => {
    const text = "## Sección\n:::details X\n\nContenido\n\n:::\n\nDespués";
    expect(typedOver(text, "ción", "Conte")).toBe("## SecXnido\n\nDespués");
  });

  it("finds a fold under code that a quote opened and closed by ending", () => {
    const text = "> ```\n> code\n\nAntes\n\n:::details X\n\nContenido\n\n:::\n\nDespués";
    expect(typedOver(text, "tes", "Conte")).toBe("> ```\n> code\n\nAnXnido\n\nDespués");
  });

  it("leaves a fold written inside code alone", () => {
    const text = "```\n:::details X\n\nContenido\n\n:::\n```\n";
    expect(typedOver(text, "details", "Conte")).toBe("```\n:::Xnido\n\n:::\n```\n");
  });

  it("leaves a blank line under a kept closing line when words follow on the next line", () => {
    const text = ":::details A\n\nOtro\n\n:::\n\nDespués\nMás";
    const edit = colonsKept(text, {
      from: text.indexOf("ro"),
      to: text.indexOf("Después") + "Después".length,
      insert: "X",
    });
    expect(text.slice(0, edit.from) + edit.insert + text.slice(edit.to)).toBe(
      ":::details A\n\nOtX\n\n:::\n\nMás",
    );
  });

  it("pairs a closing line as the core does, where a mark and a space open nothing", () => {
    const text = "Antes\n\n:::details A\n\n::: note\n\nx\n\n:::\n\nafter\n\n:::";
    expect(typedOver(text, "tes", "x")).toBe("AnX\n\nafter\n\n:::");
  });

  const moved = (text: string, from: number, to: number, at: number) => {
    const done = colonsMoved(text, { from, to }, at);
    return done && text.slice(0, done[0].from) + done[0].insert + text.slice(done[0].to);
  };

  it("drops a fold it carries on a line of its own, under the words above", () => {
    const text = "Hola\nmundo\n\n:::details X\n\nZ\n\n:::\n\nFin\n";
    expect(moved(text, text.indexOf("X"), text.indexOf("Fin") + 1, text.indexOf("mundo"))).toBe(
      "Hola\n\n:::details X\n\nZ\n\n:::\n\nFmundo\n\nin\n",
    );
  });

  it("drops a fold it carries in the middle of words, apart from the words after", () => {
    const text = "Antes y más\n\n:::details X\n\nContenido\n\nOtro\n\n:::\n\nFin";
    const from = text.indexOf("y más");
    const to = text.indexOf("Contenido") + 5;
    expect(moved(text, from, to, text.indexOf("Otro") + 2)).toBe(
      "Antes nido\n\nOty más\n\n:::details X\n\nConte\n\n:::\n\nro\n\nFin",
    );
  });
});

describe("a fold's edge crossed inside a quote, by its name or under a rule", () => {
  const typedOver = (text: string, from: number, to: number) => {
    const edit = colonsKept(text, { from, to, insert: "X" });
    return text.slice(0, edit.from) + edit.insert + text.slice(edit.to);
  };

  it("keeps the words after a quoted fold's kept closing line in the quote", () => {
    const text = "> Antes\n>\n> :::details A\n>\n> Otro\n>\n> :::\n>\n> Después";
    expect(typedOver(text, text.indexOf("ro"), text.indexOf("Des") + 3)).toBe(
      "> Antes\n>\n> :::details A\n>\n> OtX\n>\n> :::\n>\n> pués",
    );
  });

  it("keeps the quote's mark when a choice starting in a quoted fold's name undoes it", () => {
    const text = "> Antes\n>\n> :::details Plegable\n>\n> Contenido\n>\n> :::\n>\n> Después";
    expect(typedOver(text, text.indexOf("gable"), text.indexOf("nido"))).toBe(
      "> Antes\n>\n> Xnido\n>\n> Después",
    );
  });

  it("drops a fold it carries into a quote inside the quote", () => {
    const text = "> Hola mundo\n\n:::details X\n\nZ\n\n:::\n\nFin";
    const done = colonsMoved(
      text,
      { from: text.indexOf("X"), to: text.indexOf("Fin") + 1 },
      text.indexOf("mundo"),
    );
    expect(done && text.slice(0, done[0].from) + done[0].insert + text.slice(done[0].to)).toBe(
      "> Hola \n>\n> :::details X\n>\n> Z\n>\n> :::\n>\n> Fmundo\n\nin",
    );
  });

  it("leaves a fold whose name is not reached by a choice ending right before it", () => {
    const text = "Antes y más\n\n:::details Plegable\n\nContenido\n\n:::\n\nDespués";
    expect(typedOver(text, text.indexOf("y más"), text.indexOf("Plegable"))).toBe(
      "Antes X\n\n:::details Plegable\n\nContenido\n\n:::\n\nDespués",
    );
  });

  it("leaves a fold whose name a choice starts after, past its trailing spaces", () => {
    const text = ":::details Plegable  \n\nContenido\n\nOtro\n\n:::\n\nDespués";
    expect(typedOver(text, text.indexOf("le  ") + 3, text.indexOf("nido"))).toBe(
      ":::details Plegable  \n\nXnido\n\nOtro\n\n:::\n\nDespués",
    );
  });

  it.each([
    ["a rule", "Texto\n\n---\n:::details X\n\nContenido\n\n:::\n\nDespués"],
    ["a heading underlined", "Título\n===\n:::details X\n\nContenido\n\n:::\n\nDespués"],
  ])("finds a fold right under %s", (_, text) => {
    expect(typedOver(text, 0, text.indexOf("nido"))).toBe("Xnido\n\nDespués");
  });
});

const OPENED_IN_AN_ITEM =
  "# Pliegues\n\n- uno\n\n  :::details Plegable\n\n  dentro\n\n:::\n\nFin.\n";
const OPENED_IN_AN_ITEM_PLACED =
  '<h1 data-at="0-11"><span data-src="2-10">Pliegues</span></h1>' +
  '<ul data-at="12-52"><li data-at="12-52"><p data-at="14-18"><span data-src="14-17">uno</span></p>' +
  '<p data-at="21-41"><span data-src="21-40">:::details Plegable</span></p>' +
  '<p data-at="44-51"><span data-src="44-50">dentro</span></p></li></ul>' +
  '<p data-at="52-56"><span data-src="52-55">:::</span></p>' +
  '<p data-at="57-62"><span data-src="57-61">Fin.</span></p>';

const CLOSED_IN_AN_ITEM = ":::details Plegable\n\n- uno\n\n  dentro\n\n  :::\n\nFin.\n";
const CLOSED_IN_AN_ITEM_PLACED =
  '<p data-at="0-20"><span data-src="0-19">:::details Plegable</span></p>' +
  '<ul data-at="21-45"><li data-at="21-45"><p data-at="23-27"><span data-src="23-26">uno</span></p>' +
  '<p data-at="30-37"><span data-src="30-36">dentro</span></p>' +
  '<p data-at="40-44"><span data-src="40-43">:::</span></p></li></ul>' +
  '<p data-at="45-50"><span data-src="45-49">Fin.</span></p>';

describe("Backspace at an item that holds one end of a fold", () => {
  it.each([
    [
      "its opening line",
      OPENED_IN_AN_ITEM,
      OPENED_IN_AN_ITEM_PLACED,
      "# Pliegues\n\nuno\n\n:::details Plegable\n\ndentro\n\n:::\n\nFin.\n",
    ],
    [
      "its closing line",
      CLOSED_IN_AN_ITEM,
      CLOSED_IN_AN_ITEM_PLACED,
      ":::details Plegable\n\nuno\n\ndentro\n\n:::\n\nFin.\n",
    ],
  ])(
    "raises %s with the item's words, and the fold keeps both its lines",
    (_, body, placed, now) => {
      const text = opened(body, placed);
      document.getSelection()?.collapse(wordsOf("uno"), 0);
      asked("deleteContentBackward", null);
      expect(text()).toBe(now);
    },
  );
});
