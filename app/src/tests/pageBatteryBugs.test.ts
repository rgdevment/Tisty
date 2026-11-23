import { afterEach, describe, expect, it } from "vitest";
import { applied, reachIn } from "../ui/pageEdits";
import { wornWords } from "../ui/pageFormats";
import { unlined } from "../ui/pageLists";
import { worn, wornEach } from "../ui/pageMarks";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const opened = (body: string, placed: string, drawings: Record<string, string> = {}) => {
  let text = body;
  document.body.innerHTML = `<article class="page">${placed}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: (now) => {
      const drawn = drawings[now];
      const sheet = document.querySelector(".page");
      if (drawn === undefined || !sheet) return;
      sheet.innerHTML = drawn;
      writing?.drawn();
    },
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return () => text;
};

const asked = (inputType: string, reach?: Range, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => (reach ? [reach] : []) });
  document.querySelector(".page")?.dispatchEvent(event);
};

const textIn = (selector: string) => {
  const holder = document.querySelector(selector);
  const walk = document.createTreeWalker(holder as Node, NodeFilter.SHOW_TEXT);
  const found: Text[] = [];
  for (let node = walk.nextNode(); node; node = walk.nextNode()) found.push(node as Text);
  return found;
};

const rangeOf = (from: [Node, number], to: [Node, number]) => {
  const range = document.createRange();
  range.setStart(...from);
  range.setEnd(...to);
  return range;
};

const caretOn = (node: Node, offset: number) => document.getSelection()?.collapse(node, offset);

const chose = (from: [Node, number], to: [Node, number]) => {
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(rangeOf(from, to));
};

const settled = async () => {
  for (let turn = 0; turn < 30; turn++) await Promise.resolve();
};

const CODE = "# T\n\nAntes\n\n```js\nconst a = 1;\nconst b = 2;\n```\n\nDespués\n";
const CODE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-11"><span data-src="5-10">Antes</span></p>' +
  '<figure class="code" data-at="12-47"><pre spellcheck="false"><code class="language-js" data-src="18-44">' +
  "const a = 1;\nconst b = 2;\n</code></pre></figure>" +
  '<p data-at="49-58"><span data-src="49-57">Después</span></p>';

describe("Delete at the end of a code block's last line", () => {
  it("takes nothing, so the closing fence stays and the paragraph under it is not joined", () => {
    const text = opened(CODE, CODE_PLACED);
    const [code] = textIn(".page code");
    const end = code.data.length - 1;
    caretOn(code, end);
    const after = document.querySelectorAll(".page > p")[1] as Element;
    asked("deleteContentForward", rangeOf([code, end], [after, 0]));
    expect(text()).toBe(CODE);
  });

  it("still takes the line feed between two of its lines", () => {
    const text = opened(CODE, CODE_PLACED);
    const [code] = textIn(".page code");
    const end = code.data.indexOf("\n");
    caretOn(code, end);
    asked("deleteContentForward", rangeOf([code, end], [code, end + 1]));
    expect(text()).toBe(CODE.replace("1;\nconst b", "1;const b"));
  });
});

const TITLED = "# Hola\n";
const TITLED_PLACED = '<h1 data-at="0-7"><span data-src="2-6">Hola</span></h1>';
const EMPTY_PLACED = '<h1 data-title="Title" data-ghost=""></h1>';

describe("a page emptied with a key", () => {
  it("waits at its title, so what is typed next is the title again", async () => {
    const text = opened(TITLED, TITLED_PLACED, { "": EMPTY_PLACED });
    const [words] = textIn(".page h1");
    chose([words, 0], [words, 4]);
    asked("deleteContentBackward");
    await settled();
    expect(text()).toBe("");
    expect(document.getSelection()?.anchorNode).toBe(document.querySelector(".page > h1"));
    asked("insertText", undefined, "O");
    expect(text()).toBe("# O");
  });

  it("waits at its title when the choice took the title's mark too", async () => {
    const text = opened(TITLED, TITLED_PLACED, { "": EMPTY_PLACED });
    const [words] = textIn(".page h1");
    chose([document.querySelector(".page") as Node, 0], [words, 4]);
    asked("deleteContentForward");
    await settled();
    asked("insertText", undefined, "O");
    expect(text()).toBe("# O");
  });

  it("on a task's page, whose headings are its sections, writes plain words again", async () => {
    document.documentElement.classList.add("untitled");
    try {
      const text = opened("# Notas\n", '<h1 data-at="0-8"><span data-src="2-7">Notas</span></h1>', {
        "": "",
      });
      const [words] = textIn(".page h1");
      chose([document.querySelector(".page") as Node, 0], [words, 5]);
      asked("deleteContentBackward");
      await settled();
      asked("insertText", undefined, "O");
      expect(text()).toBe("O");
    } finally {
      document.documentElement.classList.remove("untitled");
    }
  });

  it("waits at its title after the last letter of the title goes", async () => {
    const text = opened("# H\n", '<h1 data-at="0-4"><span data-src="2-3">H</span></h1>', {
      "": EMPTY_PLACED,
    });
    const [words] = textIn(".page h1");
    caretOn(words, 1);
    asked("deleteContentBackward");
    await settled();
    asked("insertText", undefined, "O");
    expect(text()).toBe("# O");
  });
});

const HEAD = "# T\n\nAntes\n\n## Sección\n\nTexto bajo\n";
const HEAD_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-11"><span data-src="5-10">Antes</span></p>' +
  '<h2 data-at="12-24"><span data-src="15-23">Sección</span></h2>' +
  '<p data-at="25-36"><span data-src="25-35">Texto bajo</span></p>';
const QUOTE = "# T\n\nAntes\n\n> una cita\n> segunda línea\n\nDespués\n";
const QUOTE_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-11"><span data-src="5-10">Antes</span></p>' +
  '<blockquote data-at="12-40"><p data-at="14-40"><span data-src="14-22">una cita</span>' +
  '<br data-src="22-23" contenteditable="false"><span data-src="25-39">segunda línea</span></p></blockquote>' +
  '<p data-at="41-50"><span data-src="41-49">Después</span></p>';

const wordsOf = (said: string) => textIn(".page").find((one) => one.data === said) as Text;

describe("Backspace right after a line's mark", () => {
  it("turns a heading back into words, apart from the paragraph above", () => {
    const text = opened(HEAD, HEAD_PLACED);
    caretOn(wordsOf("Sección"), 0);
    asked("deleteContentBackward");
    expect(text()).toBe(HEAD.replace("## Sección", "Sección"));
  });

  it("takes a quote's mark off its first line, which stands apart from the rest of the quote", () => {
    const text = opened(QUOTE, QUOTE_PLACED);
    caretOn(wordsOf("una cita"), 0);
    asked("deleteContentBackward");
    expect(text()).toBe(QUOTE.replace("> una cita\n", "una cita\n\n"));
  });

  it("joins a quote's later line to the one before it, inside the quote", () => {
    const text = opened(QUOTE, QUOTE_PLACED);
    const line = wordsOf("segunda línea");
    caretOn(line, 0);
    asked("deleteContentBackward", rangeOf([wordsOf("una cita"), 8], [line, 0]));
    expect(text()).toBe(QUOTE.replace("una cita\n> segunda", "una citasegunda"));
  });

  it("leaves the page's title a title", () => {
    const text = opened(HEAD, HEAD_PLACED);
    caretOn(wordsOf("T"), 0);
    asked("deleteContentBackward");
    expect(text()).toBe(HEAD);
  });

  it("leaves the page's first line a title, whatever its level", () => {
    const body = "## Plan\n\nTexto\n";
    const text = opened(
      body,
      '<h2 data-at="0-8"><span data-src="3-7">Plan</span></h2>' +
        '<p data-at="9-15"><span data-src="9-14">Texto</span></p>',
    );
    caretOn(wordsOf("Plan"), 0);
    asked("deleteContentBackward");
    expect(text()).toBe(body);
  });

  it("keeps the words freed apart from the lines around them, in the quote they stood in", () => {
    const free = (said: string, at: number) => {
      const moved = unlined(said, at);
      return moved && applied(said, moved.edit);
    };
    expect(free("Antes\n## Sección\nTexto", 9)).toBe("Antes\n\nSección\n\nTexto");
    expect(free("> a\n> b\n> c", 6)).toBe("> a\n\nb\n\n> c");
    expect(free("> > a", 4)).toBe("> a");
    expect(free("> a\n> ## b", 9)).toBe("> a\n>\n> b");
    expect(free("> #tag b", 2)).toBe("#tag b");
    expect(free("## Sección", 1)).toBeNull();
  });

  it("turns a heading underlined with = or - into words, in a quote too", () => {
    const free = (said: string, at: number) => {
      const moved = unlined(said, at);
      return moved && applied(said, moved.edit);
    };
    expect(free("Título\n===\n\nTexto", 0)).toBe("Título\n\nTexto");
    expect(free("Sub\n---\nTexto", 0)).toBe("Sub\n\nTexto");
    expect(free("> Título\n> ===\n> resto", 2)).toBe("> Título\n>\n> resto");
    expect(free("Título\r\n===\r\n", 0)).toBe("Título\r\n");
  });

  it("frees a quote's line whatever the spaces after its mark", () => {
    const free = (said: string, at: number) => {
      const moved = unlined(said, at);
      return moved && applied(said, moved.edit);
    };
    expect(free(">  una cita", 3)).toBe("una cita");
    expect(free("> >   una cita", 6)).toBe("> una cita");
  });
});

const PARAGRAPHS = "# T\n\nHola mundo\n\n- uno\n- dos\n";
const PARAGRAPHS_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-16"><span data-src="5-15">Hola mundo</span></p>' +
  '<ul data-at="17-29"><li data-at="17-23"><span data-src="19-22">uno</span></li>' +
  '<li data-at="23-29"><span data-src="25-28">dos</span></li></ul>';

const keyed = (key: string, more: KeyboardEventInit = {}) =>
  document
    .querySelector(".page")
    ?.dispatchEvent(
      new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...more }),
    );

describe("Shift+Enter, which an engine may name a new paragraph", () => {
  it("breaks the line inside a paragraph", () => {
    const text = opened(PARAGRAPHS, PARAGRAPHS_PLACED);
    caretOn(wordsOf("Hola mundo"), 4);
    keyed("Enter", { shiftKey: true });
    expect(text()).toBe(PARAGRAPHS.replace("Hola mundo", "Hola\\\n mundo"));
  });

  it("breaks the line inside a list item, never opening another item", () => {
    const text = opened(PARAGRAPHS, PARAGRAPHS_PLACED);
    caretOn(wordsOf("dos"), 3);
    keyed("Enter", { shiftKey: true });
    expect(text()).toBe(PARAGRAPHS);
    asked("insertText", undefined, "m");
    expect(text()).toBe(PARAGRAPHS.replace("- dos", "- dos\\\n  m"));
  });

  it("at a line's end writes nothing until a key comes, so no backslash is left alone", () => {
    const text = opened(PARAGRAPHS, PARAGRAPHS_PLACED);
    caretOn(wordsOf("Hola mundo"), 10);
    keyed("Enter", { shiftKey: true });
    expect(text()).toBe(PARAGRAPHS);
    asked("insertText", undefined, "X");
    expect(text()).toBe(PARAGRAPHS.replace("Hola mundo", "Hola mundo\\\nX"));
  });

  it("in a heading starts the next line, as a heading holds no break", () => {
    const text = opened(HEAD, HEAD_PLACED);
    caretOn(wordsOf("Sección"), 3);
    keyed("Enter", { shiftKey: true });
    expect(text()).toBe(HEAD.replace("## Sección", "## Sec\nción"));
  });

  it("is left to the browser while a composition is open or being committed", () => {
    const text = opened(PARAGRAPHS, PARAGRAPHS_PLACED);
    caretOn(wordsOf("Hola mundo"), 4);
    keyed("Enter", { shiftKey: true, isComposing: true });
    keyed("Enter", { shiftKey: true, keyCode: 229 });
    expect(text()).toBe(PARAGRAPHS);
  });

  it("leaves Enter alone a new paragraph", () => {
    const text = opened(PARAGRAPHS, PARAGRAPHS_PLACED);
    caretOn(wordsOf("Hola mundo"), 4);
    keyed("Enter");
    asked("insertParagraph");
    expect(text()).toBe(PARAGRAPHS.replace("Hola mundo", "Hola\n\n mundo"));
  });
});

const TWO = "# T\n\nHola mundo bonito\n\nSegundo párrafo\n";
const TWO_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p>' +
  '<p data-at="24-41"><span data-src="24-40">Segundo párrafo</span></p>';
const JOINED = "# T\n\nHola mundo bonitoSegundo párrafo\n";

describe("a word's delete at a block's edge, where WebKit reaches a word of the block beside", () => {
  it("going back from a paragraph's start only joins it to the one above", () => {
    const text = opened(TWO, TWO_PLACED);
    const second = wordsOf("Segundo párrafo");
    caretOn(second, 0);
    asked(
      "deleteWordBackward",
      rangeOf([wordsOf("Hola mundo bonito"), 11], [second.parentElement?.parentElement as Node, 0]),
    );
    expect(text()).toBe(JOINED);
  });

  it("going forward from a paragraph's end only joins the one below to it", () => {
    const text = opened(TWO, TWO_PLACED);
    const first = wordsOf("Hola mundo bonito");
    caretOn(first, 17);
    asked("deleteWordForward", rangeOf([first, 17], [wordsOf("Segundo párrafo"), 7]));
    expect(text()).toBe(JOINED);
  });

  it("going forward from an item's end only joins the item nested under it", () => {
    const text = opened(
      "# T\n\n- Item\n  - Sub\n",
      '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
        '<ul data-at="5-20"><li data-at="5-20"><span data-src="7-11">Item</span>' +
        '<ul data-at="12-20"><li data-at="12-20"><span data-src="16-19">Sub</span></li></ul></li></ul>',
    );
    const item = wordsOf("Item");
    caretOn(item, 4);
    asked("deleteWordForward", rangeOf([item, 4], [wordsOf("Sub"), 3]));
    expect(text()).toBe("# T\n\n- ItemSub\n");
  });

  it("joins the same when the engine already stops at the edge, as Chromium does", () => {
    const text = opened(TWO, TWO_PLACED);
    const second = wordsOf("Segundo párrafo");
    caretOn(second, 0);
    asked(
      "deleteWordBackward",
      rangeOf([wordsOf("Hola mundo bonito"), 17], [second.parentElement?.parentElement as Node, 0]),
    );
    expect(text()).toBe(JOINED);
  });
});

describe("a word's delete back from a space the page does not show", () => {
  it("takes the word with that space, so no two spaces are left", () => {
    const text = opened(
      "# T\n\nHola mundo \n\nSegundo\n",
      '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
        '<p data-at="5-17"><span data-src="5-15">Hola mundo</span></p>' +
        '<p data-at="18-26"><span data-src="18-25">Segundo</span></p>',
    );
    writing?.placeAt(16);
    const words = wordsOf("Hola mundo");
    asked("deleteWordBackward", rangeOf([words, 5], [words, 10]));
    expect(text()).toBe("# T\n\nHola \n\nSegundo\n");
  });
});

const TABLED = "# T\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
const TABLED_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-18"><span data-src="5-17">Antes y más</span></p>' +
  '<div class="table celled" data-at="19-49" contenteditable="false"><table><tbody>' +
  "<tr><td>1</td><td>2</td></tr></tbody></table></div>";

const choseToTheEnd = () => {
  const sheet = document.querySelector(".page") as Element;
  const words = wordsOf("Antes y más");
  chose([words, 6], [sheet, sheet.childNodes.length]);
  return words;
};

describe("a choice that reaches past a table at the page's end", () => {
  it("is typed over whole, though the engine names a caret where it starts", () => {
    const text = opened(TABLED, TABLED_PLACED);
    const words = choseToTheEnd();
    asked("insertText", rangeOf([words, 6], [words, 6]), "X");
    expect(text()).toBe("# T\n\nAntes X");
  });

  it("is taken whole by Backspace, though the engine names the letter before it", () => {
    const text = opened(TABLED, TABLED_PLACED);
    const words = choseToTheEnd();
    asked("deleteContentBackward", rangeOf([words, 5], [words, 6]));
    expect(text()).toBe("# T\n\nAntes ");
  });

  it("only touching a table, keeps the blank line that parts the table from the words", () => {
    const text = opened(
      "# T\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nHola mundo\n",
      '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
        '<div class="table celled" data-at="5-35" contenteditable="false"><table><tbody>' +
        "<tr><td>1</td><td>2</td></tr></tbody></table></div>" +
        '<p data-at="36-47"><span data-src="36-46">Hola mundo</span></p>',
    );
    const words = wordsOf("Hola mundo");
    chose([document.querySelector(".page") as Node, 2], [words, 5]);
    asked("deleteContentBackward", rangeOf([words, 0], [words, 5]));
    expect(text()).toBe("# T\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nmundo\n");
  });

  it("starting right before a table, never joins the paragraph after it to the one above", () => {
    const text = opened(
      "# T\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nHola mundo\n",
      '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
        '<p data-at="5-11"><span data-src="5-10">Antes</span></p>' +
        '<div class="table celled" data-at="12-42" contenteditable="false"><table><tbody>' +
        "<tr><td>1</td><td>2</td></tr></tbody></table></div>" +
        '<p data-at="43-54"><span data-src="43-53">Hola mundo</span></p>',
    );
    const words = wordsOf("Hola mundo");
    chose([document.querySelector(".page") as Node, 2], [words, 5]);
    asked("deleteContentBackward", rangeOf([words, 0], [words, 5]));
    expect(text()).toBe("# T\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nmundo\n");
  });

  it("is taken whole by Delete, though the engine sends nothing for it", () => {
    const text = opened(TABLED, TABLED_PLACED);
    choseToTheEnd();
    keyed("Delete");
    expect(text()).toBe("# T\n\nAntes ");
  });
});

const PLAIN = "# T\n\nHola mundo bonito\n";
const PLAIN_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p>';
const BOLD = "# T\n\nHola **mundo** bonito\n";
const BOLD_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-27"><span data-src="5-10">Hola </span><strong><span data-src="12-17">mundo</span></strong>' +
  '<span data-src="19-26"> bonito</span></p>';
const DRAWN = { [PLAIN]: PLAIN_PLACED, [BOLD]: BOLD_PLACED };

const choseMundo = () => {
  const words = wordsOf("Hola mundo bonito");
  chose([words, 5], [words, 10]);
};

describe("the words chosen when a format key is pressed", () => {
  it("stay chosen once the page is drawn again", async () => {
    const text = opened(PLAIN, PLAIN_PLACED, DRAWN);
    choseMundo();
    writing?.format("**", "**");
    await settled();
    expect(text()).toBe(BOLD);
    expect(document.getSelection()?.toString()).toBe("mundo");
  });

  it("take a second press of the same key off again", async () => {
    const text = opened(PLAIN, PLAIN_PLACED, DRAWN);
    choseMundo();
    writing?.format("**", "**");
    await settled();
    writing?.format("**", "**");
    await settled();
    expect(text()).toBe(PLAIN);
    expect(document.getSelection()?.toString()).toBe("mundo");
  });

  it("take another format too, once drawn or still waiting to be", async () => {
    const drawn = opened(PLAIN, PLAIN_PLACED, DRAWN);
    choseMundo();
    writing?.format("**", "**");
    await settled();
    writing?.format("*", "*");
    expect(drawn()).toBe("# T\n\nHola ***mundo*** bonito\n");
    const waiting = opened(PLAIN, PLAIN_PLACED);
    choseMundo();
    writing?.format("**", "**");
    writing?.format("*", "*");
    expect(waiting()).toBe("# T\n\nHola ***mundo*** bonito\n");
  });
});

describe("the words a format key leaves chosen", () => {
  const chosenAfter = (text: string, one: { from: number; to: number }, mark: string) => {
    const spread = text.slice(one.from, one.to).includes("\n");
    const change = (spread ? wornEach : worn)(text, one, mark, mark);
    const words = wornWords(text, one, change, mark, mark);
    return applied(text, change).slice(words.from, words.to);
  };

  it("are the words inside the marks put on, and the words left when marks come off", () => {
    expect(chosenAfter("Hola mundo bonito", { from: 5, to: 10 }, "**")).toBe("mundo");
    expect(chosenAfter("Hola **mundo** bonito", { from: 5, to: 14 }, "**")).toBe("mundo");
    expect(chosenAfter("Hola **mundo** bonito", { from: 7, to: 12 }, "**")).toBe("mundo");
    expect(chosenAfter("***a** y **b***", { from: 1, to: 14 }, "*")).toBe("**a** y **b**");
    expect(chosenAfter("uno\ndos", { from: 0, to: 7 }, "*")).toBe("uno*\n*dos");
  });

  it("leave out the marks a format takes from inside the words it wraps", () => {
    expect(chosenAfter("x a **b** c y", { from: 2, to: 11 }, "**")).toBe("a b c");
  });

  it("are typed over, never the marks around them", () => {
    const text = opened(
      "# T\n\nx a **b** c y\n",
      '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
        '<p data-at="5-19"><span data-src="5-9">x a </span><strong><span data-src="11-12">b</span></strong>' +
        '<span data-src="14-18"> c y</span></p>',
    );
    chose([wordsOf("x a "), 2], [wordsOf(" c y"), 2]);
    writing?.format("**", "**");
    expect(text()).toBe("# T\n\nx **a b c** y\n");
    asked("insertText", undefined, "Z");
    expect(text()).toBe("# T\n\nx **Z** y\n");
  });
});

const BROKEN_BODY = "# T\n\nHola\\\nX mundo\n\n- dos\\\n  más\n";
const BROKEN_PLACED =
  '<h1 data-at="0-4"><span data-src="2-3">T</span></h1>' +
  '<p data-at="5-19"><span data-src="5-9">Hola</span><br data-src="9-11" contenteditable="false">' +
  '<span data-src="11-18">X mundo</span></p>' +
  '<ul data-at="20-35"><li data-at="20-35"><span data-src="22-25">dos</span>' +
  '<br data-src="25-27" contenteditable="false"><span data-src="29-32">más</span></li></ul>';

describe("a line break written as a backslash at the line's end", () => {
  it("goes whole with Backspace right after it, never leaving the backslash", () => {
    const text = opened(BROKEN_BODY, BROKEN_PLACED);
    writing?.placeAt(BROKEN_BODY.indexOf("X mundo"));
    asked("deleteContentBackward");
    expect(text()).toBe(BROKEN_BODY.replace("Hola\\\nX", "HolaX"));
  });

  it("goes whole with its line's lead when Backspace starts a list item's next line", () => {
    const text = opened(BROKEN_BODY, BROKEN_PLACED);
    writing?.placeAt(BROKEN_BODY.indexOf("más"));
    asked("deleteContentBackward");
    expect(text()).toBe(BROKEN_BODY.replace("dos\\\n  más", "dosmás"));
  });

  it("goes whole with Delete right before it", () => {
    const text = opened(BROKEN_BODY, BROKEN_PLACED);
    writing?.placeAt(BROKEN_BODY.indexOf("\\"));
    asked("deleteContentForward");
    expect(text()).toBe(BROKEN_BODY.replace("Hola\\\nX", "HolaX"));
  });

  it("goes whole with Delete when the browser names no reach past the end of the words", () => {
    const text = opened(BROKEN_BODY, BROKEN_PLACED);
    const hola = wordsOf("Hola");
    caretOn(hola, 4);
    const p = hola.parentElement?.parentElement as Element;
    asked("deleteContentForward", rangeOf([hola, 4], [p, 1]));
    expect(text()).toBe(BROKEN_BODY.replace("Hola\\\nX", "HolaX"));
  });

  it("stays a backslash when it was written twice, as an escaped one", () => {
    const body = "# T\n\nruta\\\\\nX\n";
    expect(reachIn(body, body.indexOf("X"), "deleteContentBackward")).toEqual({
      from: body.indexOf("X") - 1,
      to: body.indexOf("X"),
    });
  });
});
