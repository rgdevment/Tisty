import { afterEach, describe, expect, it } from "vitest";
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

const pressed = (key: string, isComposing = false) => {
  const event = new KeyboardEvent("keydown", { key, isComposing, bubbles: true, cancelable: true });
  document.querySelector(".page")?.dispatchEvent(event);
  return event;
};

const backspace = () => pressed("Backspace");

const TAGGED = "Texto #tag[^1] fin.\n\n[^1]: Nota.\n";
const TAGGED_PLACED =
  '<p data-at="0-20"><span data-src="0-6">Texto </span>' +
  '<span class="tag" data-tag="tag" data-src="6-10" contenteditable="false">#tag</span>' +
  '<sup class="fnref" data-src="10-14" contenteditable="false"><a href="#fn-1">1</a></sup>' +
  '<span data-src="14-19"> fin.</span></p>';

describe("Backspace at the very start of a document that opens with a list", () => {
  it("takes the first item's mark away by itself, as WebKit sends no key there", () => {
    const text = opened(
      "- uno\n\nFin.\n",
      '<ul data-at="0-6"><li data-at="0-6"><span data-src="2-5">uno</span></li></ul><p data-at="7-12"><span data-src="7-11">Fin.</span></p>',
    );
    document
      .getSelection()
      ?.collapse(document.querySelector('[data-src="2-5"]')?.firstChild as Text, 0);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("uno\n\nFin.\n");
  });

  it("takes a task's box away by itself between the box and a chip, as WebKit sends no key there either", () => {
    const text = opened(
      "- [ ] uno\n- [ ] #tag dos\n",
      '<ul data-at="0-25"><li class="task" data-at="0-10"><span class="check" data-done="false" contenteditable="false"></span><span data-src="6-9">uno</span></li>' +
        '<li class="task" data-at="10-25"><span class="check" data-done="false" contenteditable="false"></span><span class="tag" data-tag="tag" data-src="16-20" contenteditable="false">#tag</span><span data-src="20-24"> dos</span></li></ul>',
    );
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as HTMLLIElement, 1);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("- [ ] uno\n- #tag dos\n");
  });

  it("takes away a picture that opens an item when Backspace comes after it, not the item's mark", () => {
    const text = opened(
      "- uno\n- ![p](p.png) dos\n",
      '<ul data-at="0-24"><li data-at="0-6"><span data-src="2-5">uno</span></li>' +
        '<li data-at="6-24"><img src="p.png" alt="p" data-src="8-19" contenteditable="false"><span data-src="19-23"> dos</span></li></ul>',
    );
    document.getSelection()?.collapse(document.querySelectorAll("li")[1] as HTMLLIElement, 1);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("- uno\n- dos\n");
  });

  it("leaves Backspace to the browser anywhere past the first line", () => {
    const text = opened(
      "Antes\n\n- uno\n",
      '<p data-at="0-6"><span data-src="0-5">Antes</span></p><ul data-at="7-13"><li data-at="7-13"><span data-src="9-12">uno</span></li></ul>',
    );
    document
      .getSelection()
      ?.collapse(document.querySelector('[data-src="9-12"]')?.firstChild as Text, 0);
    expect(backspace().defaultPrevented).toBe(false);
    expect(text()).toBe("Antes\n\n- uno\n");
  });
});

describe("a key with the caret between two pieces drawn whole, where WebKit sends no key", () => {
  it("takes the piece before the caret on Backspace, never leaving the key to the browser", () => {
    const text = opened(TAGGED, TAGGED_PLACED);
    document.getSelection()?.collapse(document.querySelector("p") as HTMLParagraphElement, 2);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("Texto [^1] fin.\n\n[^1]: Nota.\n");
  });

  it("takes the piece after the caret on Delete", () => {
    const text = opened(TAGGED, TAGGED_PLACED);
    document.getSelection()?.collapse(document.querySelector("p") as HTMLParagraphElement, 2);
    expect(pressed("Delete").defaultPrevented).toBe(true);
    expect(text()).toBe("Texto #tag fin.\n\n[^1]: Nota.\n");
  });

  it("clears a format waiting to be typed when the browser's key is held at a widget's edge and takes the widget", () => {
    const text = opened(
      "```widget\nx\n```\n\nHola\n",
      '<figure class="widget" data-at="0-16" contenteditable="false"><div>x</div></figure>' +
        '<p data-at="17-22"><span data-src="17-21">Hola</span></p>',
    );
    const words = document.querySelector('[data-src="17-21"]')?.firstChild as Text;
    document.getSelection()?.collapse(words, 0);
    writing?.format("**", "**");
    expect(backspace().defaultPrevented).toBe(false);
    const send = (inputType: string, data: string | null) =>
      document
        .querySelector(".page")
        ?.dispatchEvent(
          new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
        );
    send("deleteContentBackward", null);
    send("insertText", "X");
    expect(text()).toBe("XHola\n");
  });

  it("clears a format waiting to be typed when the key is held and takes the widget", () => {
    const text = opened(
      "```widget\nx\n```\n\nHola\n",
      '<figure class="widget" data-at="0-16" contenteditable="false"><div>x</div></figure>' +
        '<p data-at="17-22"><span data-src="17-21">Hola</span></p>',
    );
    document.getSelection()?.collapse(document.querySelector("p") as HTMLParagraphElement, 0);
    writing?.format("**", "**");
    expect(backspace().defaultPrevented).toBe(true);
    const event = new InputEvent("beforeinput", {
      inputType: "insertText",
      data: "X",
      bubbles: true,
      cancelable: true,
    });
    document.querySelector(".page")?.dispatchEvent(event);
    expect(text()).toBe("XHola\n");
  });
});

describe("a key beside a piece drawn whole among words", () => {
  it("takes one of the spaces around the piece with it", () => {
    const text = opened(
      "Texto #tag fin\n",
      '<p data-at="0-15"><span data-src="0-6">Texto </span><span class="tag" data-tag="tag" data-src="6-10" contenteditable="false">#tag</span><span data-src="10-14"> fin</span></p>',
    );
    document.getSelection()?.collapse(document.querySelector("p") as HTMLParagraphElement, 2);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("Texto fin\n");
  });

  it("takes the format the piece was all of", () => {
    const text = opened(
      "Texto **#tag** fin\n",
      '<p data-at="0-19"><span data-src="0-6">Texto </span><strong><span class="tag" data-tag="tag" data-src="8-12" contenteditable="false">#tag</span></strong><span data-src="14-18"> fin</span></p>',
    );
    document.getSelection()?.collapse(document.querySelector("strong") as HTMLElement, 1);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("Texto  fin\n");
  });
});

describe("a key beside a piece that has no place in the text", () => {
  it("takes a task's box with Delete before it, as Backspace after it does", () => {
    const text = opened(
      "- [ ] uno\n",
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span class="check" data-done="false" contenteditable="false"></span><span data-src="6-9">uno</span></li></ul>',
    );
    document.getSelection()?.collapse(document.querySelector("li") as HTMLLIElement, 0);
    expect(pressed("Delete").defaultPrevented).toBe(true);
    expect(text()).toBe("- uno\n");
  });

  it("leaves the key to the browser when words stand on the caret's other side", () => {
    const text = opened(
      "Antes y después\n",
      '<p data-at="0-16"><span data-src="0-6">Antes </span><span class="widget" contenteditable="false">w</span><span data-src="6-15">y después</span></p>',
    );
    const holder = document.querySelector("p") as HTMLParagraphElement;
    document.getSelection()?.collapse(holder, 2);
    expect(backspace().defaultPrevented).toBe(false);
    document.getSelection()?.collapse(holder, 1);
    expect(pressed("Delete").defaultPrevented).toBe(false);
    expect(text()).toBe("Antes y después\n");
  });
});

describe("a key beside a step's button to make it a part", () => {
  const STEP = "- [ ] uno #tag\n";
  const STEP_PLACED =
    '<ul data-at="0-15"><li class="task" data-at="0-15"><span class="check" data-done="false" contenteditable="false"></span><span data-src="6-10">uno </span>' +
    '<span class="tag" data-tag="tag" data-src="10-14" contenteditable="false">#tag</span><button type="button" class="to-part" contenteditable="false">↗</button></li></ul>';

  it("leaves Delete before the button to the browser, as the button holds no words", () => {
    const text = opened(STEP, STEP_PLACED);
    document.getSelection()?.collapse(document.querySelector("li") as HTMLLIElement, 3);
    expect(pressed("Delete").defaultPrevented).toBe(false);
    expect(text()).toBe(STEP);
  });

  it("takes the tag before the button on Backspace after it", () => {
    const text = opened(STEP, STEP_PLACED);
    document.getSelection()?.collapse(document.querySelector("li") as HTMLLIElement, 4);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("- [ ] uno \n");
  });
});

describe("Backspace at the first item of a list held inside a block drawn whole", () => {
  it("is held there, as the key from the page would be, and leaves the item's mark", () => {
    const text = opened(
      "> - uno\n",
      '<blockquote data-callout="note" data-at="0-8"><ul data-at="2-8"><li data-at="2-8"><span data-src="4-7">uno</span></li></ul></blockquote>',
    );
    document.getSelection()?.collapse(document.querySelector("li") as HTMLLIElement, 0);
    expect(backspace().defaultPrevented).toBe(true);
    expect(text()).toBe("> - uno\n");
  });
});

describe("Backspace while a composition is open", () => {
  it("is left to the composition, even at the first item's start", () => {
    const text = opened(
      "- uno\n\nFin.\n",
      '<ul data-at="0-6"><li data-at="0-6"><span data-src="2-5">uno</span></li></ul><p data-at="7-12"><span data-src="7-11">Fin.</span></p>',
    );
    document
      .getSelection()
      ?.collapse(document.querySelector('[data-src="2-5"]')?.firstChild as Text, 0);
    expect(pressed("Backspace", true).defaultPrevented).toBe(false);
    expect(text()).toBe("- uno\n\nFin.\n");
  });
});
