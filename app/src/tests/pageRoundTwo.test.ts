import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it, vi } from "vitest";
import { type Hands, type PageWriting, writtenOn } from "../ui/pageWriting";

const escaped = (said: string) => said.replace(/&/g, "&amp;").replace(/</g, "&lt;");
const span = (from: number, to: number, said: string) =>
  from < to ? `<span data-src="${from}-${to}">${escaped(said)}</span>` : "";

type Line = { readonly text: string; readonly from: number; readonly end: number };

const linesOf = (body: string): Line[] => {
  let from = 0;
  return body.split("\n").map((raw) => {
    const text = raw.replace(/\r$/, "");
    const line = { text, from, end: from + text.length };
    from += raw.length + 1;
    return line;
  });
};

const HEAD = /^(#{1,6})( +|$)/;
const RULE = /^(-{3,}|\*{3,})\s*$/;
const ITEM = /^(\s*(?:[-*+]|\d+[.)])(?: +|$))(\[[ xX]\] ?)?/;
const QUOTE = /^(\s*>\s?)/;

const itemOf = (line: Line, to: number) => {
  const mark = ITEM.exec(line.text);
  const start = line.from + (mark?.[0].length ?? 0);
  const said = line.text.slice(mark?.[0].length ?? 0);
  return `<li data-at="${line.from}-${to}">${span(start, line.end, said)}</li>`;
};

// A small stand-in for the core: it draws each block with the places of its words in the Markdown.
const drawn = (body: string): string => {
  const lines = linesOf(body);
  const starts = (line: Line | undefined) =>
    Boolean(
      line &&
        (HEAD.test(line.text) ||
          RULE.test(line.text) ||
          ITEM.test(line.text) ||
          QUOTE.test(line.text)),
    );
  let html = "";
  for (let at = 0; at < lines.length; ) {
    const line = lines[at] as Line;
    const head = HEAD.exec(line.text);
    if (!line.text.trim()) {
      at++;
    } else if (head) {
      const size = head[1]?.length ?? 1;
      const from = line.from + head[0].length;
      const inner = span(from, line.end, line.text.slice(head[0].length));
      html += `<h${size} data-at="${line.from}-${line.end}">${inner}</h${size}>`;
      at++;
    } else if (RULE.test(line.text)) {
      html += `<hr data-at="${line.from}-${line.end}">`;
      at++;
    } else if (ITEM.test(line.text) || QUOTE.test(line.text)) {
      const quote = QUOTE.test(line.text);
      const group: Line[] = [];
      while (lines[at] && (quote ? QUOTE : ITEM).test((lines[at] as Line).text)) {
        group.push(lines[at++] as Line);
        let ahead = at;
        while (lines[ahead] && !(lines[ahead] as Line).text.trim()) ahead++;
        if (!quote && ahead > at && ITEM.test(lines[ahead]?.text ?? "")) at = ahead;
      }
      const last = group[group.length - 1] as Line;
      const place = `data-at="${line.from}-${last.end}"`;
      if (quote) {
        const inner = group.map((one) => {
          const lead = QUOTE.exec(one.text)?.[0].length ?? 0;
          return span(one.from + lead, one.end, one.text.slice(lead));
        });
        html += `<blockquote ${place}><p>${inner.join("<br>")}</p></blockquote>`;
      } else {
        const tag = /^\s*\d/.test(line.text) ? "ol" : "ul";
        const next = lines.slice(at).find((one) => one.text.trim());
        const room = next ? next.from : lines.reduce((sum, one) => Math.max(sum, one.end), 0);
        const items = group.map((one, index) => itemOf(one, group[index + 1]?.from ?? room));
        html += `<${tag} data-at="${line.from}-${room}">${items.join("")}</${tag}>`;
      }
    } else {
      const group: Line[] = [line];
      at++;
      while (lines[at]?.text.trim() && !starts(lines[at])) group.push(lines[at++] as Line);
      const last = group[group.length - 1] as Line;
      const inner = group.map((one) => span(one.from, one.end, one.text));
      html += `<p data-at="${line.from}-${last.end}">${inner.join("<br>")}</p>`;
    }
  }
  return html;
};

type Session = { text: string; live: boolean; said: string[] };

let writing: PageWriting | null = null;
let session: Session = { text: "", live: false, said: [] };
const menus: boolean[] = [];

afterEach(() => {
  session.live = false;
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const sheet = () => document.querySelector(".page") as HTMLElement;

const draw = (body: string) => {
  const page = document.querySelector(".page");
  if (page) page.innerHTML = drawn(body);
};

const writer = (body: string, more: Partial<Hands> = {}) => {
  writing?.stop();
  session.live = false;
  const mine: Session = { text: body, live: true, said: [] };
  session = mine;
  menus.length = 0;
  document.body.innerHTML = '<article class="page"></article>';
  draw(body);
  writing = writtenOn(document, {
    text: () => mine.text,
    write: (next) => {
      mine.text = next;
      mine.said.push(next);
    },
    shown: () => {},
    redraw: (now) => {
      if (!mine.live) return;
      draw(now);
      writing?.drawn();
    },
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
    ask: () => {},
    blocks: (_at, slash) => menus.push(slash),
    acts: () => {},
    ...more,
  });
  writing.drawn();
  return mine;
};

const at = (place: number) => writing?.placeAt(place);

// A page with the caret laid in it, as the window leaves it after the page is drawn.
const open = (body: string, caret: number, more: Partial<Hands> = {}) => {
  const one = writer(body, more);
  at(caret);
  return one;
};

const text = () => session.text;
const settle = () => new Promise<void>((done) => setTimeout(done, 0));

const flush = async () => {
  for (let turn = 0; turn < 30; turn++) await Promise.resolve();
};

const asked = (inputType: string, data: string | null = null) =>
  sheet().dispatchEvent(
    new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
  );

const type = (said: string) => {
  for (const one of said) asked("insertText", one);
};

const typeSlowly = async (said: string) => {
  for (const one of said) {
    asked("insertText", one);
    await settle();
  }
};

const enter = () => asked("insertParagraph");
const shiftEnter = () => asked("insertLineBreak");
const back = () => asked("deleteContentBackward");
const forward = () => asked("deleteContentForward");

const key = (name: string, more: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", {
    key: name,
    bubbles: true,
    cancelable: true,
    ...more,
  });
  sheet().dispatchEvent(event);
  return event;
};

const ctrl = (name: string, more: KeyboardEventInit = {}) => key(name, { ctrlKey: true, ...more });

const choose = (from: number, to: number) => {
  const range = document.createRange();
  const start = spotAt(from);
  const end = spotAt(to);
  if (!start || !end) throw new Error("no place to choose");
  range.setStart(start[0], start[1]);
  range.setEnd(end[0], end[1]);
  document.getSelection()?.removeAllRanges();
  document.getSelection()?.addRange(range);
  document.dispatchEvent(new Event("selectionchange"));
};

const spotAt = (place: number): [Node, number] | null => {
  const walker = document.createTreeWalker(sheet(), NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const holder = (node.parentElement as Element).closest("[data-src]");
    const range = holder?.getAttribute("data-src")?.split("-").map(Number);
    const [from, to] = range ?? [];
    if (from !== undefined && to !== undefined && from <= place && place <= to)
      return [node, place - from];
  }
  return null;
};

const block = (name: string) => SLASHED.find((one) => one.key === name) as Slashed;

const bareLineFeeds = (said: string) => /(?<!\r)\n/.test(said);

describe("Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y on the page", () => {
  it("takes the last key back with Ctrl+Z", () => {
    open("Hola\n", 4);
    type("x");
    expect(text()).toBe("Holax\n");
    ctrl("z");
    expect(text()).toBe("Hola\n");
  });

  it("takes it back with Cmd+Z too", () => {
    open("Hola\n", 4);
    type("x");
    key("z", { metaKey: true });
    expect(text()).toBe("Hola\n");
  });

  it("does the key again with Ctrl+Shift+Z", () => {
    open("Hola\n", 4);
    type("x");
    ctrl("z");
    ctrl("Z", { shiftKey: true });
    expect(text()).toBe("Holax\n");
  });

  it("does the key again with Ctrl+Y", () => {
    open("Hola\n", 4);
    type("x");
    ctrl("z");
    ctrl("y");
    expect(text()).toBe("Holax\n");
  });

  it("holds back the browser's own undo for those keys", () => {
    open("Hola\n", 4);
    type("x");
    expect(ctrl("z").defaultPrevented).toBe(true);
    expect(ctrl("y").defaultPrevented).toBe(true);
  });

  it("changes nothing with no key to take back or to do again", () => {
    const one = open("Hola\n", 4);
    ctrl("z");
    ctrl("y");
    expect(one.said).toEqual([]);
  });

  it("takes back words typed together in one step, down to the start", () => {
    open("Hola\n", 4);
    type("abc");
    ctrl("z");
    expect(text()).toBe("Hola\n");
    ctrl("z");
    expect(text()).toBe("Hola\n");
    ctrl("y");
    expect(text()).toBe("Holaabc\n");
  });

  it("takes back words typed after a pause as a step of their own", () => {
    const clock = vi.spyOn(Date, "now").mockReturnValue(1000);
    open("Hola\n", 4);
    type("ab");
    clock.mockReturnValue(5000);
    type("cd");
    ctrl("z");
    expect(text()).toBe("Holaab\n");
    ctrl("z");
    expect(text()).toBe("Hola\n");
    clock.mockRestore();
  });

  it("takes back the words after a new line apart from those before it", () => {
    open("Hola\n", 4);
    type("ab");
    enter();
    type("cd");
    ctrl("z");
    expect(text()).not.toContain("cd");
    expect(text()).toContain("Holaab");
  });

  it("takes back a Backspace run in one step", () => {
    open("Hola\n", 4);
    for (let n = 0; n < 3; n++) asked("deleteContentBackward");
    expect(text()).toBe("H\n");
    ctrl("z");
    expect(text()).toBe("Hola\n");
  });

  it("forgets what could be done again once a new key is written", () => {
    open("Hola\n", 4);
    type("a");
    ctrl("z");
    type("b");
    ctrl("y");
    expect(text()).toBe("Holab\n");
  });

  it("leaves the page alone when Alt is held with Ctrl and Z", () => {
    open("Hola\n", 4);
    type("x");
    ctrl("z", { altKey: true });
    expect(text()).toBe("Holax\n");
  });

  it("leaves the cell's own field to undo its words", () => {
    open("Hola\n", 4);
    type("x");
    sheet().insertAdjacentHTML("beforeend", '<input class="table-input">');
    (sheet().querySelector("input") as HTMLInputElement).dispatchEvent(
      new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }),
    );
    expect(text()).toBe("Holax\n");
  });

  it("drops a format waiting for its first key", () => {
    open("Hola\n", 4);
    type("x");
    ctrl("b");
    ctrl("z");
    type("y");
    expect(text()).toBe("Holay\n");
  });

  it("lays the caret where the undone key was, so the next key goes there", async () => {
    open("Hola mundo\n", 4);
    type("!");
    await settle();
    ctrl("z");
    await settle();
    type("?");
    expect(text()).toBe("Hola? mundo\n");
  });

  it("takes back a block made from the menu with Ctrl+Z", () => {
    open("Hola\n", 2);
    writing?.slashed(block("h2"));
    expect(text()).toBe("## Hola\n");
    ctrl("z");
    expect(text()).toBe("Hola\n");
  });

  it("takes back an Enter that left a list, and does it again", async () => {
    open("- a\n- ", 6);
    enter();
    expect(text()).toBe("- a\n");
    ctrl("z");
    expect(text()).toBe("- a\n- ");
    ctrl("y");
    expect(text()).toBe("- a\n");
  });
});

describe("Enter on a list item or a quote line with nothing written", () => {
  it.each([
    ["a bullet", "- a\n- ", "- a\n"],
    ["a star bullet", "* a\n* ", "* a\n"],
    ["a plus bullet", "+ a\n+ ", "+ a\n"],
    ["a number", "1. a\n2. ", "1. a\n"],
    ["a number with a parenthesis", "1) a\n2) ", "1) a\n"],
    ["an unchecked box", "- [ ] a\n- [ ] ", "- [ ] a\n"],
    ["a checked box", "- [x] a\n- [x] ", "- [x] a\n"],
    ["a box with no space after it", "- [ ] a\n- [ ]", "- [ ] a\n"],
    ["a quote line with a space", "> a\n> ", "> a\n"],
    ["a quote line with no space", "> a\n>", "> a\n"],
    ["a bullet in a quote", "> - a\n> - ", "> - a\n"],
    ["a number in a quote", "> 1. a\n> 2. ", "> 1. a\n"],
  ])("takes away %s", (_, body, expected) => {
    open(body, body.length);
    enter();
    expect(text()).toBe(expected);
  });

  it("brings an empty nested bullet up to its parent's level instead of leaving the list", () => {
    open("- a\n  - ", 8);
    enter();
    expect(text()).toBe("- a\n- ");
  });

  it("begins a paragraph after a blank line with the next key", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    type("x");
    expect(text()).toBe("- a\n\nx");
  });

  it("keeps the blank line when the next key comes before the page is drawn again", () => {
    open("- a\n- ", 6);
    enter();
    type("x");
    expect(text()).toBe("- a\n\nx");
  });

  it.each([
    ["a bullet", "- a\n- ", "- a\n\nx"],
    ["a number", "1. a\n2. ", "1. a\n\nx"],
    ["a box", "- [ ] a\n- [ ] ", "- [ ] a\n\nx"],
    ["a quote line", "> a\n> ", "> a\n\nx"],
    ["a bullet in a quote", "> - a\n> - ", "> - a\n\nx"],
  ])(
    "writes the next key of %s as a paragraph apart from the list or the quote",
    async (_, body, expected) => {
      open(body, body.length);
      enter();
      await settle();
      type("x");
      expect(text()).toBe(expected);
    },
  );

  it("shows a line just begun where the item was, until a key is written", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    expect(document.querySelector("[data-ghost]")).not.toBeNull();
  });

  it("leaves a list in the middle of a page and keeps the words after it apart", async () => {
    open("- a\n- \n- b", 6);
    enter();
    await settle();
    type("x");
    expect(text()).toBe("- a\n\nx\n\n- b");
  });

  it("begins the page's first paragraph when the only item is left", () => {
    open("- ", 2);
    enter();
    type("x");
    expect(text()).toBe("x");
  });

  it("leaves a quote when the item left is under a quote's text", () => {
    open("> a\n>\n> ", 8);
    enter();
    expect(text()).toBe("> a\n");
  });

  it("takes away both empty quote lines when Enter comes on the second", () => {
    open("> a\n> \n> ", 9);
    enter();
    expect(text()).toBe("> a\n");
  });

  it("adds an item when the item has words", () => {
    open("- a", 3);
    enter();
    expect(text()).toBe("- a\n- ");
  });

  it("splits an item when the caret is not at its end", () => {
    open("- ab", 3);
    enter();
    expect(text()).toBe("- a\n- b");
  });

  it("leaves the list with a body written with CRLF, keeping CRLF", async () => {
    open("- a\r\n- ", 7);
    enter();
    await settle();
    type("x");
    expect(text()).toBe("- a\r\n\r\nx");
    expect(bareLineFeeds(text())).toBe(false);
  });

  it("goes on with an empty item written after an empty one is left", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    enter();
    await settle();
    type("x");
    expect(text().endsWith("x")).toBe(true);
    expect(text().startsWith("- a\n")).toBe(true);
  });
});

describe("a line just begun, left with Backspace", () => {
  it("is cancelled by Backspace and nothing else is deleted", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    back();
    expect(document.querySelector("[data-ghost]")).toBeNull();
    expect(text()).toBe("- a\n");
  });

  it("is cancelled by Delete too", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    forward();
    expect(document.querySelector("[data-ghost]")).toBeNull();
    expect(text()).toBe("- a\n");
  });

  it("is cancelled when a break was laid and nothing was written under it", async () => {
    open("Uno\n", 3);
    ctrl("Enter");
    await settle();
    back();
    expect(document.querySelector("[data-ghost]")).toBeNull();
    expect(text()).toBe("Uno\n\n---\n");
  });

  it("joins the block above when a paragraph begun with Enter gets a Backspace", async () => {
    open("# Hola\n\nUno\n", 6);
    enter();
    await settle();
    back();
    expect(text()).toBe("# Hola\n\nUno\n");
  });
});

describe("Ctrl+Enter puts a page break after the caret's block", () => {
  it("lays the break a blank line after the paragraph", () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    expect(text()).toBe("Uno\n\n---\n\nDos\n");
  });

  it("writes the next key under the break when it comes before the page is drawn again", () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    type("x");
    expect(text()).toBe("Uno\n\n---\n\nx\n\nDos\n");
  });

  it("writes the next key under the break after the page is drawn again", async () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    await settle();
    type("x");
    expect(text()).toBe("Uno\n\n---\n\nx\n\nDos\n");
  });

  it("never glues the paragraph above to the break as a setext title", async () => {
    open("Uno", 3);
    ctrl("Enter");
    await settle();
    type("x");
    expect(text()).not.toMatch(/Uno\n---/);
    expect(text()).toBe("Uno\n\n---\n\nx\n");
  });

  it("never glues the key to the rule when it comes before the page is drawn again", () => {
    open("Uno", 3);
    ctrl("Enter");
    type("x");
    expect(text()).not.toMatch(/---x/);
  });

  it("puts the break after the whole paragraph when the caret stands at its start", () => {
    open("Uno\ndos\n\nTres", 0);
    ctrl("Enter");
    expect(text()).toBe("Uno\ndos\n\n---\n\nTres");
  });

  it("puts the break after the whole list, not inside it", () => {
    open("- a\n- b\n\nTres\n", 2);
    ctrl("Enter");
    expect(text()).toBe("- a\n- b\n\n---\n\nTres\n");
  });

  it("puts the break after the whole quote", () => {
    open("> a\n> b\n\nTres\n", 2);
    ctrl("Enter");
    expect(text()).toBe("> a\n> b\n\n---\n\nTres\n");
  });

  it("puts the break after a title", () => {
    open("# Hola\n\nUno\n", 3);
    ctrl("Enter");
    expect(text()).toBe("# Hola\n\n---\n\nUno\n");
  });

  it("keeps CRLF in the lines it adds", async () => {
    open("Uno\r\n\r\nDos\r\n", 3);
    ctrl("Enter");
    await settle();
    type("x");
    expect(text()).toBe("Uno\r\n\r\n---\r\n\r\nx\r\n\r\nDos\r\n");
  });

  it("works with Cmd and Enter", () => {
    open("Uno\n", 3);
    key("Enter", { metaKey: true });
    expect(text()).toBe("Uno\n\n---\n");
  });

  it("holds back the browser's own Ctrl+Enter", () => {
    open("Uno\n", 3);
    expect(ctrl("Enter").defaultPrevented).toBe(true);
  });

  it("takes the page's last paragraph with no newline at the end", () => {
    open("Uno", 1);
    ctrl("Enter");
    expect(text()).toBe("Uno\n\n---\n");
  });

  it("is taken back with one Ctrl+Z", () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    ctrl("z");
    expect(text()).toBe("Uno\n\nDos\n");
  });
});

describe("the caret kept in the source decides Backspace and Delete", () => {
  it("takes away the key just written before the page is drawn again", () => {
    open("Hola\n", 4);
    type("ab");
    back();
    expect(text()).toBe("Holaa\n");
    back();
    expect(text()).toBe("Hola\n");
  });

  it("takes away what comes after the key just written with Delete", () => {
    open("Holaxyz\n", 4);
    type("a");
    forward();
    expect(text()).toBe("Holaayz\n");
  });

  it("takes away the key just written after the page is drawn again", async () => {
    open("Hola\n", 4);
    await typeSlowly("ab");
    back();
    expect(text()).toBe("Holaa\n");
  });

  it("deletes a trailing space the page does not show", () => {
    open("Hola  \n", 4);
    at(6);
    back();
    expect(text()).toBe("Hola \n");
  });

  it("deletes a space after the caret that the page does not show", () => {
    open("Hola  y\n", 4);
    at(4);
    forward();
    expect(text()).toBe("Hola y\n");
  });

  it("deletes the key typed after a trailing space, not the word before", () => {
    open("Hola \n", 4);
    at(5);
    type("x");
    back();
    expect(text()).toBe("Hola \n");
  });

  it("lets a caret moved by an arrow decide again", async () => {
    open("Hola mundo\n", 10);
    type("!");
    await settle();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
    sheet().dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }));
    at(2);
    back();
    expect(text()).toBe("Hla mundo!\n");
  });

  it("takes away a chosen stretch, not the caret kept", () => {
    open("Hola mundo\n", 4);
    type("!");
    choose(0, 4);
    back();
    expect(text()).toBe("! mundo\n");
  });
});

describe("a caret laid in an empty block", () => {
  it("gets a line break in an empty heading so it can stay there", () => {
    open("# \n\nUno\n", 2);
    expect(document.querySelector("h1 br")).not.toBeNull();
    expect(document.getSelection()?.anchorNode).toBe(document.querySelector("h1"));
  });

  it("lands the typing in the heading", async () => {
    open("# \n\nUno\n", 2);
    type("x");
    await settle();
    type("y");
    expect(text()).toBe("# xy\n\nUno\n");
  });

  it("gets a line break in an empty quote too", () => {
    open(">\n\nUno\n", 1);
    expect(document.querySelector("blockquote br")).not.toBeNull();
  });

  it("lands the typing in the quote", () => {
    open("> \n\nUno\n", 2);
    type("x");
    expect(text()).toBe("> x\n\nUno\n");
  });

  it("does not add a second line break when the block already has one", async () => {
    open("# \n\nUno\n", 2);
    writing?.drawn();
    writing?.drawn();
    expect(document.querySelectorAll("h1 br")).toHaveLength(1);
  });

  it("leaves a block with words as it is", () => {
    open("# Hola\n\nUno\n", 6);
    expect(document.querySelector("h1 br")).toBeNull();
  });
});

describe("formats around the words chosen", () => {
  it("leaves the spaces at the ends of the words chosen outside the marks", () => {
    open("Uno dos tres\n", 0);
    choose(3, 8);
    writing?.format("**", "**");
    expect(text()).toBe("Uno **dos** tres\n");
  });

  it("leaves several spaces at each end outside", () => {
    open("Uno  dos  tres\n", 0);
    choose(3, 10);
    writing?.format("*", "*");
    expect(text()).toBe("Uno  *dos*  tres\n");
  });

  it("changes nothing when only spaces are chosen", () => {
    const one = open("Uno   tres\n", 0);
    choose(3, 6);
    writing?.format("**", "**");
    expect(one.said).toEqual([]);
  });

  it.each([
    ["bold", "**", "a **b** c", "**a b c**"],
    ["strike", "~~", "a ~~b~~ c", "~~a b c~~"],
    ["underline", "++", "a ++b++ c", "++a b c++"],
    ["italic", "*", "a *b* c", "*a b c*"],
    ["pen", "==", "a ==b== c", "==a b c=="],
  ])("makes words partly in %s wholly so with no marks left inside", (_, mark, body, expected) => {
    open(`${body}\n`, 0);
    choose(0, body.length);
    writing?.format(mark, mark);
    expect(text()).toBe(`${expected}\n`);
  });

  it("takes a format off words wholly wearing it", () => {
    open("a **b c** d\n", 0);
    choose(4, 7);
    writing?.format("**", "**");
    expect(text()).toBe("a b c d\n");
  });

  it("keeps a bold word inside an italic stretch whole", () => {
    open("a **b** c\n", 0);
    choose(0, 9);
    writing?.format("*", "*");
    expect(text()).toBe("*a **b** c*\n");
  });

  it("wraps the chosen words when Ctrl+B comes with a choice", () => {
    open("Uno dos tres\n", 0);
    choose(4, 7);
    ctrl("b");
    expect(text()).toBe("Uno **dos** tres\n");
  });
});

describe("a format started with nothing chosen", () => {
  it("keeps going across spaces until pressed again", () => {
    open("\n", 0);
    ctrl("b");
    type("a b c");
    expect(text()).toBe("**a b c**\n");
  });

  it("keeps going across spaces when the page is drawn again after each key", async () => {
    open("\n", 0);
    ctrl("b");
    await typeSlowly("a b c");
    expect(text()).toBe("**a b c**\n");
  });

  it("carries the closing marks past several spaces", () => {
    open("\n", 0);
    ctrl("b");
    type("a  b");
    expect(text()).toBe("**a  b**\n");
  });

  it("ends when pressed again, and the next key is plain", () => {
    open("\n", 0);
    ctrl("b");
    type("a");
    ctrl("b");
    type("c");
    expect(text()).toBe("**a**c\n");
  });

  it("ends when pressed again after the page is drawn again", async () => {
    open("\n", 0);
    ctrl("b");
    await typeSlowly("a");
    ctrl("b");
    await typeSlowly("c");
    expect(text()).toBe("**a**c\n");
  });

  it("stacks several formats pending", () => {
    open("\n", 0);
    ctrl("b");
    ctrl("i");
    type("x");
    expect(text()).toBe("***x***\n");
  });

  it("stacks bold and strike and keeps them across a space", () => {
    open("\n", 0);
    ctrl("b");
    ctrl("X", { shiftKey: true });
    type("a b");
    expect(text()).toBe("**~~a b~~**\n");
  });

  it("lets one of the stacked formats go when it is pressed again", () => {
    open("\n", 0);
    ctrl("b");
    ctrl("i");
    ctrl("b");
    type("x");
    expect(text()).toBe("*x*\n");
  });

  it("rejoins a format closed just before the caret", () => {
    open("**a** b\n", 5);
    ctrl("b");
    type("c");
    expect(text()).toBe("**ac** b\n");
  });

  it("rejoins an italic closed just before the caret", () => {
    open("*a* b\n", 3);
    ctrl("i");
    type("c");
    expect(text()).toBe("*ac* b\n");
  });

  it("rejoins a strike and keeps what follows", () => {
    open("~~a~~ b\n", 5);
    writing?.format("~~", "~~");
    type("c");
    expect(text()).toBe("~~ac~~ b\n");
  });

  it("does not rejoin when a space comes first", () => {
    open("**a** b\n", 5);
    ctrl("b");
    type(" c");
    expect(text()).toBe("**a** **c** b\n");
  });

  it("starts a format after plain words with the next key", () => {
    open("hola\n", 4);
    ctrl("u");
    type("x");
    expect(text()).toBe("hola++x++\n");
  });

  it("lets the pending format go when an arrow moves the caret", () => {
    open("hola\n", 4);
    ctrl("b");
    sheet().dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }));
    at(1);
    type("x");
    expect(text()).toBe("hxola\n");
  });

  it("lets the pending format go when Enter is pressed", () => {
    open("hola\n", 4);
    ctrl("b");
    enter();
    type("x");
    expect(text()).toBe("hola\n\nx\n");
  });
});

describe("deleting the last letter of a format", () => {
  it.each([
    ["bold", "x **a** y", 5, "x  y"],
    ["italic", "x *a* y", 4, "x  y"],
    ["strike", "x ~~a~~ y", 5, "x  y"],
    ["underline", "x ++a++ y", 5, "x  y"],
    ["code", "x `a` y", 4, "x  y"],
    ["pen", "x ==a== y", 5, "x  y"],
  ])("takes the marks of %s away with it", (_, body, caret, expected) => {
    open(`${body}\n`, caret);
    back();
    expect(text()).toBe(`${expected}\n`);
  });

  it("takes the marks away when the letter is deleted forward", () => {
    open("x **a** y\n", 4);
    forward();
    expect(text()).toBe("x  y\n");
  });

  it("keeps the marks while words are left in them", () => {
    open("x **ab** y\n", 6);
    back();
    expect(text()).toBe("x **a** y\n");
  });

  it("takes both pairs away when a letter wore bold and italic", () => {
    open("***a***\n", 4);
    back();
    expect(text()).toBe("\n");
  });

  it("takes the marks away after the format was typed and the page drawn again", async () => {
    open("\n", 0);
    ctrl("b");
    await typeSlowly("a");
    back();
    expect(text()).toBe("\n");
  });

  it("takes the marks away of a format typed with no redraw in between", () => {
    open("\n", 0);
    ctrl("b");
    type("a");
    back();
    expect(text()).toBe("\n");
  });
});

describe("a body written with CRLF", () => {
  it("keeps CRLF when Enter starts a paragraph", () => {
    open("Uno\r\n\r\nDos\r\n", 3);
    enter();
    expect(bareLineFeeds(text())).toBe(false);
    expect(text()).toBe("Uno\r\n\r\n\r\n\r\nDos\r\n");
  });

  it("keeps CRLF when Shift+Enter breaks a line", () => {
    open("Uno\r\n\r\nDos\r\n", 3);
    shiftEnter();
    type("X");
    expect(text()).toBe("Uno\\\r\nX\r\n\r\nDos\r\n");
  });

  it("keeps CRLF when Enter adds an item", () => {
    open("- a\r\n- b\r\n", 3);
    enter();
    expect(text()).toBe("- a\r\n- \r\n- b\r\n");
  });

  it("keeps CRLF when Enter goes on in a quote", () => {
    open("> a\r\n\r\nz\r\n", 3);
    enter();
    expect(bareLineFeeds(text())).toBe(false);
  });

  it("keeps CRLF when a block comes from the menu", () => {
    open("Uno\r\n\r\nDos\r\n", 1);
    writing?.slashed(block("table"));
    expect(bareLineFeeds(text())).toBe(false);
  });

  it("keeps CRLF when a quote is made from a paragraph of two lines", () => {
    open("uno\r\ndos\r\n", 1);
    writing?.slashed(block("quote"));
    expect(text()).toBe("> uno\r\n> dos\r\n");
  });

  it("keeps CRLF when a typed key follows a line just begun", async () => {
    open("- a\r\n- ", 7);
    enter();
    await settle();
    type("x");
    expect(bareLineFeeds(text())).toBe(false);
  });

  it("keeps CRLF when a word is typed in a pending format", () => {
    open("Uno\r\n\r\nDos\r\n", 3);
    ctrl("b");
    type("x");
    enter();
    expect(bareLineFeeds(text())).toBe(false);
  });
});

describe("Enter and Shift+Enter past the marks closing a format opened on the line", () => {
  it.each([
    ["bold", "**dos** tres\n", 5, "**dos**\n\n tres\n"],
    ["italic", "*dos* tres\n", 4, "*dos*\n\n tres\n"],
    ["strike", "~~dos~~ tres\n", 5, "~~dos~~\n\n tres\n"],
    ["underline", "++dos++ tres\n", 5, "++dos++\n\n tres\n"],
    ["code", "`dos` tres\n", 4, "`dos`\n\n tres\n"],
    ["a link", "[dos](http://a.b) tres\n", 4, "[dos](http://a.b)\n\n tres\n"],
  ])("steps Enter past the closing marks of %s", (_, body, caret, expected) => {
    open(body, caret);
    enter();
    expect(text()).toBe(expected);
  });

  it("steps Shift+Enter past the closing marks", () => {
    open("**dos** tres\n", 5);
    shiftEnter();
    expect(text()).toBe("**dos**\\\n tres\n");
  });

  it("steps past two closings at once", () => {
    open("**a ~~b~~** c\n", 7);
    enter();
    expect(text()).toBe("**a ~~b~~**\n\n c\n");
  });

  it("does not step past the opening marks of the next word", () => {
    open("a **b** c\n", 2);
    enter();
    expect(text()).toBe("a \n\n**b** c\n");
  });

  it("does not step past marks that close nothing opened on the line", () => {
    open("**dos tres\n\ncuatro** cinco\n", 10);
    enter();
    expect(text().startsWith("**dos tres\n\n")).toBe(true);
  });

  it("steps a new item past the closing marks too", () => {
    open("- **a** b\n", 5);
    enter();
    expect(text()).toBe("- **a**\n-  b\n");
  });
});

describe("blocks made on the page", () => {
  it("marks every line of a paragraph with a line broken inside when it is made a quote", () => {
    open("uno\ndos\ntres\n", 1);
    writing?.slashed(block("quote"));
    expect(text()).toBe("> uno\n> dos\n> tres\n");
  });

  it("marks every line of a quote made from the caret at the paragraph's last line", () => {
    open("uno\ndos\ntres\n", 10);
    writing?.slashed(block("quote"));
    expect(text()).toBe("> uno\n> dos\n> tres\n");
  });

  it("makes a quote of the paragraph the caret stands in and no other", () => {
    open("uno\ndos\n\notro\n", 1);
    writing?.slashed(block("quote"));
    expect(text()).toBe("> uno\n> dos\n\notro\n");
  });

  it("makes a quote of a line made with Shift+Enter", async () => {
    open("uno\n", 3);
    shiftEnter();
    type("dos");
    await flush();
    writing?.slashed(block("quote"));
    expect(text()).toBe("> uno\\\n> dos\n");
  });

  it("makes a one-line quote of a paragraph of one line", () => {
    open("uno\n\ndos\n", 1);
    writing?.slashed(block("quote"));
    expect(text()).toBe("> uno\n\ndos\n");
  });

  it("makes a title of the paragraph's first line, whichever line the caret is in", () => {
    open("uno\ndos\n", 5);
    writing?.slashed(block("h2"));
    expect(text()).toBe("## uno\ndos\n");
  });

  it("turns a list item into a quote with no mark left over", () => {
    open("- a\n- b\n", 1);
    writing?.slashed(block("quote"));
    expect(text()).toBe("> a\n- b\n");
  });

  it("puts a bulleted list on the line just begun after an item is left", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    writing?.slashed(block("numbers"));
    expect(text()).toBe("- a\n\n1. ");
  });
});

describe("Duplicate and Delete of a block", () => {
  const top = (nth: number) => sheet().children[nth] as Element;

  it("puts the copy a blank line after the block", () => {
    open("a\n\nb\n\nc\n", 0);
    writing?.unitActed(top(1), "double");
    expect(text()).toBe("a\n\nb\n\nb\n\nc\n");
  });

  it("puts the copy after the last block when the page has no newline at its end", () => {
    open("a\n\nb", 0);
    writing?.unitActed(top(1), "double");
    expect(text()).toBe("a\n\nb\n\nb");
  });

  it("copies the first block", () => {
    open("a\n\nb\n", 0);
    writing?.unitActed(top(0), "double");
    expect(text()).toBe("a\n\na\n\nb\n");
  });

  it("copies a title with its section", () => {
    open("# T\n\nuno\n\n# U\n", 0);
    writing?.unitActed(top(0), "double");
    expect(text()).toBe("# T\n\nuno\n\n# T\n\nuno\n\n# U\n");
  });

  it("copies a list whole", () => {
    open("- a\n- b\n\nz\n", 0);
    writing?.unitActed(top(0), "double");
    expect(text()).toBe("- a\n- b\n\n- a\n- b\n\nz\n");
  });

  it("takes a middle block away with one blank line left", () => {
    open("a\n\nb\n\nc\n", 0);
    writing?.unitActed(top(1), "drop");
    expect(text()).toBe("a\n\nc\n");
  });

  it("takes the first block away with no blank line left at the top", () => {
    open("a\n\nb\n\nc\n", 0);
    writing?.unitActed(top(0), "drop");
    expect(text()).toBe("b\n\nc\n");
  });

  it("takes the last block away with a single newline at the end", () => {
    open("a\n\nb\n\nc\n", 0);
    writing?.unitActed(top(2), "drop");
    expect(text()).toBe("a\n\nb\n");
  });

  it("takes the last block away when the page has no newline at its end", () => {
    open("a\n\nb\n\nc", 0);
    writing?.unitActed(top(2), "drop");
    expect(text()).toBe("a\n\nb\n");
  });

  it("takes the only block away and leaves nothing", () => {
    open("a\n", 0);
    writing?.unitActed(top(0), "drop");
    expect(text()).toBe("");
  });

  it("takes a title away with its section and leaves the next one", () => {
    open("# T\n\nuno\n\n# U\n\ndos\n", 0);
    writing?.unitActed(top(0), "drop");
    expect(text()).toBe("# U\n\ndos\n");
  });

  it("takes a block away when more than one blank line stood after it", () => {
    open("a\n\n\n\nb\n\nc\n", 0);
    writing?.unitActed(top(1), "drop");
    expect(text()).toBe("a\n\n\n\nc\n");
  });

  it("takes a block away from a CRLF body keeping CRLF", () => {
    open("a\r\n\r\nb\r\n\r\nc\r\n", 0);
    writing?.unitActed(top(1), "drop");
    expect(text()).toBe("a\r\n\r\nc\r\n");
  });

  it("copies a block of a CRLF body keeping CRLF", () => {
    open("a\r\n\r\nb\r\n", 0);
    writing?.unitActed(top(0), "double");
    expect(bareLineFeeds(text())).toBe(false);
  });

  it("is taken back with one Ctrl+Z", () => {
    open("a\n\nb\n\nc\n", 0);
    writing?.unitActed(top(1), "drop");
    ctrl("z");
    expect(text()).toBe("a\n\nb\n\nc\n");
  });
});

describe("Alt and an arrow on list items", () => {
  const alt = (name: string) => key(name, { altKey: true });

  const LIST = "- a\n- b\n- c\n\nTail\n";

  it("moves an item up and keeps the blank line after the list", () => {
    open(LIST, 6);
    alt("ArrowUp");
    expect(text()).toBe("- b\n- a\n- c\n\nTail\n");
  });

  it("moves the last item up and keeps the blank line after the list", () => {
    open(LIST, 11);
    alt("ArrowUp");
    expect(text()).toBe("- a\n- c\n- b\n\nTail\n");
  });

  it("moves an item down into the last place and keeps the blank line after the list", () => {
    open(LIST, 6);
    alt("ArrowDown");
    expect(text()).toBe("- a\n- c\n- b\n\nTail\n");
  });

  it("moves the first item down", () => {
    open(LIST, 2);
    alt("ArrowDown");
    expect(text()).toBe("- b\n- a\n- c\n\nTail\n");
  });

  it("keeps two blank lines after the list in place", () => {
    open("- a\n- b\n\n\nTail\n", 6);
    alt("ArrowUp");
    expect(text()).toBe("- b\n- a\n\n\nTail\n");
  });

  it("keeps a list at the end of the page with no newline", () => {
    open("- a\n- b", 6);
    alt("ArrowUp");
    expect(text()).toBe("- b\n- a");
  });

  it("changes nothing for the first item going up", () => {
    const one = open(LIST, 2);
    alt("ArrowUp");
    expect(one.said).toEqual([]);
  });

  it("changes nothing for the last item going down", () => {
    const one = open(LIST, 11);
    alt("ArrowDown");
    expect(one.said).toEqual([]);
  });

  it("keeps each place's number when numbered items trade places", () => {
    open("1. a\n2. b\n3. c\n\nTail\n", 8);
    alt("ArrowUp");
    expect(text()).toBe("1. b\n2. a\n3. c\n\nTail\n");
  });

  it("moves a box with its words", () => {
    open("- [ ] a\n- [x] b\n\nTail\n", 12);
    alt("ArrowUp");
    expect(text()).toBe("- [x] b\n- [ ] a\n\nTail\n");
  });

  it("keeps CRLF between the items moved", () => {
    open("- a\r\n- b\r\n\r\nTail\r\n", 7);
    alt("ArrowUp");
    expect(text()).toBe("- b\r\n- a\r\n\r\nTail\r\n");
  });

  it("holds back the browser's own Alt and arrow", () => {
    open(LIST, 6);
    expect(alt("ArrowUp").defaultPrevented).toBe(true);
  });

  it("is taken back with Ctrl+Z", () => {
    open(LIST, 6);
    alt("ArrowUp");
    ctrl("z");
    expect(text()).toBe(LIST);
  });

  it("moves a paragraph block with Alt and an arrow", () => {
    open("a\n\nb\n\nc\n", 3);
    alt("ArrowUp");
    expect(text()).toBe("b\n\na\n\nc\n");
  });
});

const mulberry32 = (seed: number) => {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let mixed = Math.imul(state ^ (state >>> 15), state | 1);
    mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296;
  };
};

const pick = <T>(random: () => number, from: readonly T[]): T =>
  from[Math.floor(random() * from.length)] as T;

type Walk = { readonly body: string; readonly caret: number; readonly steps: readonly string[] };
type Trace = { readonly texts: readonly string[]; readonly errors: readonly string[] };

const STEP_KINDS = [
  "type",
  "type",
  "type",
  "type",
  "bs",
  "del",
  "enter",
  "enter",
  "br",
  "break",
  "undo",
  "redo",
  "bold",
  "italic",
  "wait",
  "wait",
];

const STEP_LETTERS = [..."abcxyz 01é"];

const WITHOUT_DELETES = STEP_KINDS.filter((one) => one !== "bs" && one !== "del");

const randomSteps = (random: () => number, size: number, kinds: readonly string[]) =>
  Array.from({ length: size }, () => {
    const kind = pick(random, kinds);
    return kind === "type" ? `type:${pick(random, STEP_LETTERS)}` : kind;
  });

const step = async (one: string) => {
  if (one.startsWith("type:")) asked("insertText", one.slice(5));
  else if (one === "bs") back();
  else if (one === "del") forward();
  else if (one === "enter") enter();
  else if (one === "br") shiftEnter();
  else if (one === "break") ctrl("Enter");
  else if (one === "undo") ctrl("z");
  else if (one === "redo") ctrl("y");
  else if (one === "bold") ctrl("b");
  else if (one === "italic") ctrl("i");
  else await flush();
};

const walked = async (walk: Walk): Promise<Trace> => {
  const errors: string[] = [];
  const fail = (event: ErrorEvent) => {
    errors.push(String(event.message));
    event.preventDefault();
  };
  window.addEventListener("error", fail);
  open(walk.body, walk.caret);
  const texts: string[] = [];
  for (const one of walk.steps) {
    try {
      await step(one);
      await flush();
    } catch (thrown) {
      errors.push(String(thrown));
    }
    texts.push(text());
  }
  await settle();
  window.removeEventListener("error", fail);
  writing?.stop();
  writing = null;
  session.live = false;
  return { texts, errors };
};

type Check = (
  walk: Walk,
  trace: Trace,
  again: (steps: readonly string[]) => Promise<Trace>,
) => string | null | Promise<string | null>;

const noThrow: Check = (_, trace) => trace.errors[0] ?? null;

const keepsLineEnds: Check = (walk, trace) => {
  if (!walk.body.includes("\r\n")) return null;
  const bare = trace.texts.findIndex((one) => bareLineFeeds(one));
  return bare < 0
    ? null
    : `a bare line feed after step ${bare}: ${JSON.stringify(trace.texts[bare])}`;
};

const undoesToStart: Check = async (walk, _, again) => {
  const undone = await again([...walk.steps, ...Array(walk.steps.length + 2).fill("undo")]);
  const last = undone.texts[undone.texts.length - 1];
  return last === walk.body ? null : `undo ended on ${JSON.stringify(last)}`;
};

const redoesToLast: Check = async (walk, trace, again) => {
  const last = trace.texts[trace.texts.length - 1];
  const count = walk.steps.length + 2;
  const redone = await again([
    ...walk.steps,
    ...Array(count).fill("undo"),
    ...Array(count).fill("redo"),
  ]);
  const top = redone.texts[redone.texts.length - 1];
  return top === last ? null : `redo ended on ${JSON.stringify(top)}, not ${JSON.stringify(last)}`;
};

const keepsTypedLetters: Check = (walk, trace) => {
  const typed = walk.steps
    .filter((one) => one.startsWith("type:") && /S/.test(one.slice(5)))
    .map((one) => one.slice(5));
  const last = trace.texts[trace.texts.length - 1] ?? "";
  let from = 0;
  for (const one of typed) {
    from = last.indexOf(one, from);
    if (from < 0)
      return `typed ${JSON.stringify(typed.join(""))} is not in ${JSON.stringify(last)}`;
    from++;
  }
  return null;
};

const WITHOUT_HISTORY = STEP_KINDS.filter((one) => one !== "undo" && one !== "redo");
const ONLY_WRITING = STEP_KINDS.filter((one) => !["bs", "del", "undo", "redo"].includes(one));

const noGluedRule: Check = (_, trace) => {
  const glued = trace.texts.findIndex(
    (one) => /(^|\n)-{3}[^\s-]/.test(one) || /\S\n-{3}\s*(\n|$)/.test(one),
  );
  return glued < 0
    ? null
    : `a rule glued after step ${glued}: ${JSON.stringify(trace.texts[glued])}`;
};

const checked = async (walk: Walk, check: Check) => {
  const trace = await walked(walk);
  return check(walk, trace, (steps) => walked({ ...walk, steps }));
};

const shrunk = async (walk: Walk, check: Check) => {
  let small = [...walk.steps];
  for (let size = Math.ceil(small.length / 2); size >= 1; size = Math.floor(size / 2)) {
    for (let from = 0; from + size <= small.length; ) {
      const without = [...small.slice(0, from), ...small.slice(from + size)];
      if (await checked({ ...walk, steps: without }, check)) small = without;
      else from += size;
    }
  }
  return small;
};

const BODIES = [
  "# Hola\n\nUno **dos** tres\n\n- primero\n- segundo\n",
  "- a\n- b\n\n> q\n> r\n\nz",
  "# T\r\n\r\nUno dos\r\n\r\n1. uno\r\n2. dos\r\n\r\n> q\r\n",
];

// A caret stands in a line's words, never between a CR and its LF nor on a blank line the page does not draw.
const whole = (body: string, caret: number) => {
  let at = caret;
  while (at > 0 && /[\r\n]/.test(body[at] ?? "\n") && /[\r\n]/.test(body[at - 1] ?? "")) at--;
  return at;
};

const fuzzed = async (check: Check, kinds: readonly string[] = STEP_KINDS, seeds = 25) => {
  for (const body of BODIES) {
    for (let seed = 1; seed <= seeds; seed++) {
      const random = mulberry32(seed);
      const walk = {
        body,
        caret: whole(body, Math.floor(random() * (body.length + 1))),
        steps: randomSteps(random, 15 + Math.floor(random() * 15), kinds),
      };
      const found = await checked(walk, check);
      if (found) {
        const small = await shrunk(walk, check);
        const why = await checked({ ...walk, steps: small }, check);
        throw new Error(
          `seed ${seed}, body ${JSON.stringify(body)}, caret ${walk.caret}, steps ${JSON.stringify(small)}: ${why}`,
        );
      }
    }
  }
};

describe("random walks over the second round's keys", () => {
  it("throws nowhere", async () => {
    await fuzzed(noThrow);
  }, 120000);

  it("never leaves a bare line feed in a body written with CRLF", async () => {
    await fuzzed(keepsLineEnds);
  }, 120000);

  it("undoes every change back to the start", async () => {
    await fuzzed(undoesToStart);
  }, 120000);

  it("never glues a page break to a key or to the paragraph above", async () => {
    await fuzzed(noGluedRule, WITHOUT_DELETES);
  }, 120000);

  it("comes forward to the last text with redo after a full undo", async () => {
    await fuzzed(redoesToLast, WITHOUT_HISTORY);
  }, 120000);

  it("keeps every letter typed, in order, while only writing", async () => {
    await fuzzed(keepsTypedLetters, ONLY_WRITING);
  }, 120000);
});

describe("what the second round found on keys and blocks", () => {
  it("ends a format pressed again after a space has been carried past its closing marks", () => {
    open("\n", 0);
    ctrl("b");
    type("a ");
    ctrl("b");
    type("b");
    expect(text()).toBe("**a** b\n");
  });

  it("ends that format the same way when the page is drawn again after each key", async () => {
    open("\n", 0);
    ctrl("b");
    await typeSlowly("a ");
    ctrl("b");
    await typeSlowly("b");
    expect(text()).toBe("**a** b\n");
  });

  it("opens the block menu at a slash typed on a line begun by leaving a list", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    type("/");
    expect(menus).toEqual([true]);
    expect(text()).toBe("- a\n");
  });

  it("opens the block menu at a slash typed on a line begun under a page break", async () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    await settle();
    type("/");
    expect(menus).toEqual([true]);
  });

  it("opens the block menu at a slash typed on a paragraph begun with Enter", async () => {
    open("Uno\n\nDos\n", 3);
    enter();
    await settle();
    type("/");
    expect(menus).toEqual([true]);
  });

  it("keeps the next key off the rule after the line under a break is cancelled with Backspace", async () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    await settle();
    back();
    await settle();
    type("x");
    expect(text()).not.toContain("---x");
  });

  it("keeps the next key off the rule after the line under a break is cancelled with Delete", async () => {
    open("Uno\n\nDos\n", 3);
    ctrl("Enter");
    await settle();
    forward();
    await settle();
    type("x");
    expect(text()).not.toContain("---x");
  });

  it("puts the caret after the bullet when the Enter that left a list is taken back", async () => {
    open("- a\n- \n", 6);
    enter();
    await settle();
    ctrl("z");
    await settle();
    type("x");
    expect(text()).toBe("- a\n- x\n");
  });

  it("leaves a list from an item whose marker has no space after it", () => {
    open("- a\n-", 5);
    enter();
    expect(text()).toBe("- a\n");
  });

  it("leaves a numbered list from an item whose number has no space after it", () => {
    open("1. a\n2.", 7);
    enter();
    expect(text()).toBe("1. a\n");
  });

  it("moves the last block of a page with no newline at its end up without a blank line left at the end", () => {
    open("a\n\nb", 0);
    writing?.unitActed(sheet().children[1] as Element, "up");
    expect(text()).toBe("b\n\na\n");
  });

  it("moves an item up in a list with blank lines between its items, the blank lines staying", () => {
    open("- a\n\n- b\n\n- c\n\nTail\n", 7);
    key("ArrowUp", { altKey: true });
    expect(text()).toBe("- b\n\n- a\n\n- c\n\nTail\n");
  });

  it("moves the last item of such a list up and keeps the blank line before the next block", () => {
    open("- a\n\n- b\n\n- c\n\nTail\n", 12);
    key("ArrowUp", { altKey: true });
    expect(text()).toBe("- a\n\n- c\n\n- b\n\nTail\n");
  });

  it("keeps uneven blank lines between items where they were", () => {
    open("- a\n\n\n- b\n- c\n\nTail\n", 2);
    key("ArrowDown", { altKey: true });
    expect(text()).toBe("- b\n\n\n- a\n- c\n\nTail\n");
  });

  it("leaves a quote of two empty lines with one Enter", () => {
    open("> uno\n>\n> ", 10);
    enter();
    expect(text()).toBe("> uno\n");
  });

  it("leaves the quote and begins a paragraph after a blank line with the next key", async () => {
    open("> uno\n>\n> ", 10);
    enter();
    await settle();
    type("x");
    expect(text()).toBe("> uno\n\nx");
  });

  it("leaves an item that sits alone between two paragraphs and writes the next key between them", async () => {
    open("Uno\n\n- \n\nDos\n", 7);
    enter();
    await settle();
    type("x");
    expect(text()).toBe("Uno\n\nx\n\nDos\n");
  });

  it("takes back the key written after leaving a list, then the leaving", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    await typeSlowly("xy");
    ctrl("z");
    expect(text()).toBe("- a\n\nx");
    ctrl("z");
    expect(text()).toBe("- a\n");
  });

  it("puts a quote on the line begun by leaving a list", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    writing?.slashed(block("quote"));
    expect(text()).toBe("- a\n\n> ");
  });

  it("puts a title on the line begun by leaving a list", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    writing?.slashed(block("h2"));
    expect(text()).toBe("- a\n\n## ");
  });

  it("puts a page break a blank line under the list just left", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    ctrl("Enter");
    expect(text()).toBe("- a\n\n---\n");
  });

  it("holds Enter on a line just begun without writing anything", async () => {
    open("- a\n- ", 6);
    enter();
    await settle();
    enter();
    await settle();
    expect(text()).toBe("- a\n");
  });
});

describe("a selection that starts or ends inside a format", () => {
  it("makes a selection that starts in bold and ends past it wholly bold with no marks doubled", () => {
    open("a **b** c\n", 0);
    choose(4, 9);
    writing?.format("**", "**");
    expect(text()).not.toContain("****");
  });

  it("makes a selection that starts before bold and ends inside it wholly bold with no marks doubled", () => {
    open("a **b** c\n", 0);
    choose(0, 5);
    writing?.format("**", "**");
    expect(text()).not.toContain("****");
  });

  it("does the same for a selection that starts inside a strike and ends past it", () => {
    open("a ~~b~~ c\n", 0);
    choose(0, 6);
    writing?.format("~~", "~~");
    expect(text()).not.toContain("~~~~");
  });

  it("does the same for a selection that starts in a word already bold and ends in another", () => {
    open("**a** b\n", 0);
    choose(2, 7);
    writing?.format("**", "**");
    expect(text()).not.toContain("****");
  });

  it("makes a selection covering a whole bold word and the words around it wholly bold", () => {
    open("a **b** c\n", 0);
    choose(0, 9);
    writing?.format("**", "**");
    expect(text()).toBe("**a b c**\n");
  });

  it("takes bold off a selection of only the bold word's letters", () => {
    open("a **b** c\n", 0);
    choose(4, 5);
    writing?.format("**", "**");
    expect(text()).toBe("a b c\n");
  });

  it("adds italic around a bold word's letters inside the bold", () => {
    open("a **b** c\n", 0);
    choose(4, 5);
    writing?.format("*", "*");
    expect(text()).toBe("a ***b*** c\n");
  });
});

describe("a page break laid after a line broken inside a paragraph", () => {
  it("never turns the paragraph above into a setext title", async () => {
    open("Uno", 3);
    shiftEnter();
    await settle();
    ctrl("Enter");
    await settle();
    expect(text()).not.toMatch(/Uno\n---/);
  });

  it("never turns the paragraph above into a setext title when the key just typed is the line", async () => {
    open("Uno\n", 3);
    ctrl("Enter");
    await settle();
    type("é");
    await settle();
    shiftEnter();
    await settle();
    ctrl("Enter");
    await settle();
    expect(text()).not.toMatch(/é\n---/);
  });
});
