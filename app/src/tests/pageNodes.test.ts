import { afterEach, describe, expect, it } from "vitest";
import { besideBox, brokenBy, letterAhead, opened } from "../ui/pageNodes";
import { writtenOn } from "../ui/pageWriting";

const PAGE =
  '<p data-at="0-4">Uno.</p>' +
  '<div class="table" data-at="6-20"><table><tr><td>x</td></tr></table></div>' +
  '<figure class="code" data-at="22-40"><pre><code>fn x() {}</code></pre></figure>' +
  '<p data-at="42-46">Dos.</p>';

afterEach(() => {
  document.body.innerHTML = "";
});

const watched = () => {
  const watching = new MutationObserver(() => {});
  watching.observe(document.body, { attributes: true, subtree: true });
  return watching;
};

describe("a page opened for writing", () => {
  it("writes nothing it already holds when opened again", () => {
    document.body.innerHTML = `<article class="page">${PAGE}</article>`;
    opened(document);
    const watching = watched();
    opened(document);
    expect(watching.takeRecords()).toEqual([]);
    expect(document.querySelector(".table")?.getAttribute("contenteditable")).toBe("false");
    expect(document.querySelector("code")?.getAttribute("spellcheck")).toBe("false");
  });

  it("opens only the blocks a key laid, leaving the rest untouched", () => {
    document.body.innerHTML = `<article class="page">${PAGE}</article>`;
    opened(document);
    const sheet = document.querySelector(".page") as Element;
    const laid = document.createElement("div");
    laid.className = "table";
    laid.setAttribute("data-at", "6-21");
    sheet.children[1].replaceWith(laid);
    const watching = watched();
    opened(document, [laid]);
    const touched = watching.takeRecords().map((one) => one.target);
    expect(touched).toEqual([laid]);
    expect(laid.getAttribute("contenteditable")).toBe("false");
  });

  it("leaves the caret a place between a step's box and a piece drawn whole that opens it", () => {
    document.body.innerHTML =
      '<article class="page"><ul data-at="0-40">' +
      '<li class="task" data-at="0-10"><span class="check"></span><span data-src="6-9">uno</span></li>' +
      '<li class="task" data-at="10-25"><span class="check"></span><span class="tag" data-src="16-20" contenteditable="false">#eti</span><span data-src="20-24"> dos</span></li>' +
      '<li data-at="25-40"><code data-src="27-32" contenteditable="false">cod</code></li>' +
      '</ul><p data-at="41-50"><code data-src="41-46" contenteditable="false">cod</code></p></article>';
    opened(document);
    const [words, tagged] = document.querySelectorAll("li.task");
    expect(tagged.children[1]?.tagName).toBe("WBR");
    expect(tagged.children[2]?.matches(".tag")).toBe(true);
    expect(tagged.textContent).toBe("#eti dos");
    expect(document.querySelectorAll("wbr")).toHaveLength(1);
    expect(words.querySelector("wbr")).toBeNull();
    const watching = new MutationObserver(() => {});
    watching.observe(document.body, { childList: true, subtree: true });
    opened(document);
    opened(document, [document.querySelector("ul") as Element]);
    expect(watching.takeRecords()).toEqual([]);
  });

  it("copies words chosen across that place as their Markdown, with nothing of it", () => {
    const body = "- [ ] uno\n- [ ] #eti dos\n";
    document.body.innerHTML =
      '<article class="page"><ul data-at="0-25">' +
      '<li class="task" data-at="0-10"><span class="check"></span><span data-src="6-9">uno</span></li>' +
      '<li class="task" data-at="10-25"><span class="check"></span><span class="tag" data-src="16-20" contenteditable="false">#eti</span><span data-src="20-24"> dos</span></li>' +
      "</ul></article>";
    const writing = writtenOn(document, {
      text: () => body,
      write: () => {},
      shown: () => {},
      redraw: () => {},
      near: () => Promise.reject(new Error("no core here")),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    const words = (src: string) =>
      document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;
    document.getSelection()?.setBaseAndExtent(words("6-9"), 0, words("20-24"), 4);
    const put: string[] = [];
    const copying = new Event("copy", { bubbles: true, cancelable: true });
    Object.defineProperty(copying, "clipboardData", {
      value: { setData: (_: string, said: string) => put.push(said) },
    });
    document.querySelector(".page")?.dispatchEvent(copying);
    writing.stop();
    expect(document.querySelectorAll("wbr")).toHaveLength(1);
    expect(put).toEqual(["uno\n- [ ] #eti dos"]);
  });
});

const ENDS =
  '<p data-at="0-20"><span data-src="0-6">Antes </span><span class="tag" data-src="6-10" contenteditable="false">#eti</span></p>' +
  '<ul data-at="21-40"><li data-at="21-40"><span data-src="23-27">uno </span><code data-src="27-32" contenteditable="false">cod</code></li></ul>' +
  '<p data-at="41-60"><span data-src="41-47">Antes </span><strong><span class="tag" data-src="49-53" contenteditable="false">#eti</span></strong></p>' +
  '<p data-at="61-80"><span data-src="61-67">Antes </span><span data-src="67-72" contenteditable="false"><math><mi>x</mi></math></span></p>' +
  '<p data-at="81-99"><span data-src="81-86" contenteditable="false"><math><mi>y</mi></math></span><span data-src="86-92"> medio</span></p>' +
  '<p data-at="100-120"><span class="tag" data-src="100-104" contenteditable="false">#eti</span><br data-src="104-106" contenteditable="false"><span data-src="106-110">dos</span></p>';

const nextOf = (selector: string) => document.querySelector(selector)?.nextSibling;

describe("a piece drawn whole at the end of its line", () => {
  it("leaves the caret an empty place after it, once, and none to a piece inside marks", () => {
    document.body.innerHTML = `<article class="page">${ENDS}</article>`;
    opened(document);
    for (const one of ['[data-src="6-10"]', '[data-src="27-32"]', '[data-src="67-72"]']) {
      expect(nextOf(one)?.nodeType).toBe(Node.TEXT_NODE);
      expect(nextOf(one)?.textContent).toBe("");
    }
    expect(nextOf("strong")).toBeNull();
    expect(nextOf('[data-src="49-53"]')).toBeNull();
    expect(nextOf('[data-src="81-86"]')?.textContent).toBe(" medio");
    expect(nextOf('[data-src="100-104"]')?.textContent).toBe("\u200B");
    expect(nextOf('[data-src="100-104"]')?.nextSibling?.nodeName).toBe("BR");
    const watching = new MutationObserver(() => {});
    watching.observe(document.body, { childList: true, subtree: true, characterData: true });
    opened(document);
    opened(document, Array.from(document.querySelectorAll(".page > *")));
    expect(watching.takeRecords()).toEqual([]);
  });

  it("gives no place to a piece outside every block", () => {
    document.body.innerHTML =
      '<article class="page"><span class="tag" data-src="0-4" contenteditable="false">#eti</span></article>';
    opened(document);
    expect(document.querySelector(".page")?.childNodes).toHaveLength(1);
    expect(document.querySelector(".page")?.nextSibling).toBeNull();
  });

  it("keeps one unseen letter in a formula whose empty place was lost", () => {
    document.body.innerHTML = `<article class="page">${ENDS}</article>`;
    opened(document);
    const formula = document.querySelector('[data-src="67-72"]') as Element;
    formula.nextSibling?.remove();
    opened(document);
    expect(formula.textContent).toBe("x\u200B");
    expect(formula.nextSibling?.textContent).toBe("");
  });

  it("gives a formula ending its line a letter that shows nothing, and no other piece one", () => {
    document.body.innerHTML = `<article class="page">${ENDS}</article>`;
    opened(document);
    const ending = document.querySelector('[data-src="67-72"]');
    expect(ending?.lastChild?.textContent).toBe("\u200B");
    expect(document.querySelector('[data-src="81-86"]')?.lastChild?.nodeName).toBe("math");
    expect(document.querySelector('[data-src="6-10"]')?.textContent).toBe("#eti");
    expect(document.querySelector(".page")?.textContent).toBe(
      "Antes #etiuno codAntes #etiAntes x\u200By medio#eti\u200Bdos",
    );
  });

  it("copies words chosen across a formula that ends its line as their Markdown", () => {
    const body = "Antes $x$\n\nFin\n";
    document.body.innerHTML =
      '<article class="page"><p data-at="0-10"><span data-src="0-6">Antes </span><span data-src="6-9" contenteditable="false"><math><mi>x</mi></math></span></p>' +
      '<p data-at="11-14"><span data-src="11-14">Fin</span></p></article>';
    const writing = writtenOn(document, {
      text: () => body,
      write: () => {},
      shown: () => {},
      redraw: () => {},
      near: () => Promise.reject(new Error("no core here")),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    const words = (src: string) =>
      document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;
    document.getSelection()?.setBaseAndExtent(words("0-6"), 0, words("11-14"), 3);
    const put: string[] = [];
    const copying = new Event("copy", { bubbles: true, cancelable: true });
    Object.defineProperty(copying, "clipboardData", {
      value: { setData: (_: string, said: string) => put.push(said) },
    });
    document.querySelector(".page")?.dispatchEvent(copying);
    expect(document.querySelector('[data-src="6-9"]')?.lastChild?.textContent).toBe("\u200B");
    writing.stop();
    expect(put).toEqual(["Antes $x$\n\nFin"]);
  });
});

describe("a caret beside a step's box", () => {
  it("is told apart before and after the box, and nowhere else", () => {
    document.body.innerHTML =
      '<article class="page"><ul data-at="0-10"><li class="task" data-at="0-10"><span class="check" contenteditable="false"></span><span data-src="6-9">uno</span></li></ul>' +
      '<p data-at="11-14"><span data-src="11-14">Fin</span></p></article>';
    const step = document.querySelector("li") as Element;
    const words = document.querySelector('[data-src="6-9"]')?.firstChild as Text;
    expect(besideBox(step, 0)).toBe("before");
    expect(besideBox(step, 1)).toBe("after");
    expect(besideBox(step, 2)).toBeNull();
    expect(besideBox(words, 0)).toBeNull();
    expect(besideBox(document.querySelector("p") as Element, 0)).toBeNull();
  });
});

const BROKEN_BODY = "uno #eti\\\ndos\n\n- tres `c`\n  - dentro\n";
const BROKEN =
  '<p data-at="0-14"><span data-src="0-4">uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span><br data-src="8-10" contenteditable="false"><span data-src="10-13">dos</span></p>' +
  '<ul data-at="15-38"><li data-at="15-38"><span data-src="17-22">tres </span><code data-src="22-25" contenteditable="false">c</code>' +
  '<ul data-at="28-38"><li data-at="28-38"><span data-src="30-36">dentro</span></li></ul></li></ul>';

describe("a piece that ends a line broken by a break or a nested list", () => {
  it("gets a letter that shows nothing after it, once, and tells what breaks the line", () => {
    document.body.innerHTML = `<article class="page">${BROKEN}</article>`;
    opened(document);
    const tag = document.querySelector('[data-src="4-8"]') as Element;
    const code = document.querySelector('[data-src="22-25"]') as Element;
    expect(tag.nextSibling?.textContent).toBe("\u200B");
    expect(code.nextSibling?.textContent).toBe("\u200B");
    expect(brokenBy(tag)?.nodeName).toBe("BR");
    expect(brokenBy(code)?.nodeName).toBe("UL");
    const watching = new MutationObserver(() => {});
    watching.observe(document.body, { childList: true, subtree: true, characterData: true });
    opened(document);
    expect(watching.takeRecords()).toEqual([]);
  });

  it("is walked past with ← and →, one stop after it and one on each side of the break", () => {
    document.body.innerHTML = `<article class="page">${BROKEN}</article>`;
    const writing = writtenOn(document, {
      text: () => BROKEN_BODY,
      write: () => {},
      shown: () => {},
      redraw: () => {},
      near: () => Promise.reject(new Error("no core here")),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    const sheet = document.querySelector(".page") as Element;
    const words = (src: string) =>
      document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;
    const arrow = (key: string) => {
      const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
      sheet.dispatchEvent(event);
      const held = document.getSelection();
      return [event.defaultPrevented, held?.anchorNode, held?.anchorOffset];
    };
    const line = document.querySelector("p") as Element;
    const item = document.querySelector("li") as Element;
    document.getSelection()?.collapse(words("0-4"), 4);
    expect(arrow("ArrowRight")).toEqual([true, line, 2]);
    expect(arrow("ArrowRight")).toEqual([true, line, 4]);
    expect(arrow("ArrowLeft")).toEqual([true, line, 2]);
    expect(arrow("ArrowLeft")).toEqual([true, line, 1]);
    document.getSelection()?.collapse(words("17-22"), 5);
    expect(arrow("ArrowRight")).toEqual([true, item, 2]);
    expect(arrow("ArrowRight")).toEqual([true, words("30-36"), 0]);
    expect(arrow("ArrowLeft")).toEqual([true, item, 2]);
    writing.stop();
  });
});

const DRAWN_WHOLE =
  '<div class="table" data-at="0-30" contenteditable="false"><table><tr>' +
  '<td><span class="tag" data-src="2-6" contenteditable="false">#eti</span><br data-src="6-10" contenteditable="false">b</td>' +
  '<td><span data-src="12-15" contenteditable="false"><math><mi>x</mi></math></span></td></tr></table></div>' +
  '<p data-at="31-50"><span data-src="31-37">Antes </span><strong><span class="tag" data-src="39-43" contenteditable="false">#eti</span></strong>' +
  '<br data-src="45-47" contenteditable="false"><span data-src="47-50">dos</span></p>';

describe("a piece the caret never stands beside", () => {
  it("is left undressed inside a block drawn whole, and dressed past the marks around it", () => {
    document.body.innerHTML = `<article class="page">${DRAWN_WHOLE}</article>`;
    opened(document);
    expect(document.querySelector(".table")?.textContent).toBe("#etibx");
    expect(nextOf("strong")?.textContent).toBe("\u200B");
    expect(nextOf("strong")?.nextSibling?.nodeName).toBe("BR");
    expect(document.querySelector("strong")?.textContent).toBe("#eti");
  });

  it("is dressed before a nested block that is not a list", () => {
    document.body.innerHTML =
      '<article class="page"><ul data-at="0-30"><li data-at="0-30"><span data-src="2-6">uno </span>' +
      '<span class="tag" data-src="6-10" contenteditable="false">#eti</span>' +
      '<figure class="code" data-at="13-30"><pre><code data-src="19-21">x\n</code></pre></figure></li></ul></article>';
    opened(document);
    expect(nextOf('[data-src="6-10"]')?.textContent).toBe("\u200B");
    expect(brokenBy(document.querySelector('[data-src="6-10"]') as Element)?.nodeName).toBe(
      "FIGURE",
    );
  });
});

const EVERY_BODY =
  "uno #eti\\\ndos $x$\n\n- tres `c`\n  - dentro\n\n**#b**\\\n[enlace](u)\n\n| #t |\n|---|\n| x |\n\n- cinco `d`\n  ```\n  y\n  ```\n";
const EVERY_PIECE =
  '<p data-at="0-17"><span data-src="0-4">uno </span><span class="tag" data-src="4-8" contenteditable="false">#eti</span>' +
  '<br data-src="8-10" contenteditable="false"><span data-src="10-14">dos </span>' +
  '<span data-src="14-17" contenteditable="false"><math><mi>x</mi></math></span></p>' +
  '<ul data-at="19-41"><li data-at="19-41"><span data-src="21-26">tres </span><code data-src="26-29" contenteditable="false">c</code>' +
  '<ul data-at="30-41"><li data-at="30-41"><span data-src="34-40">dentro</span></li></ul></li></ul>' +
  '<p data-at="42-61"><strong><span class="tag" data-src="44-46" contenteditable="false">#b</span></strong>' +
  '<br data-src="48-50" contenteditable="false"><a href="u"><span data-src="51-57">enlace</span></a></p>' +
  '<div class="table" data-at="63-82" contenteditable="false"><table><tr><td>' +
  '<span class="tag" data-src="65-67" contenteditable="false">#t</span><br data-src="67-67" contenteditable="false">x' +
  "</td></tr></table></div>" +
  '<ul data-at="83-111"><li data-at="83-111"><span data-src="85-91">cinco </span><code data-src="91-94" contenteditable="false">d</code>' +
  '<figure class="code" data-at="95-111"><pre><code data-src="103-105">y\n</code></pre></figure></li></ul>';

describe("the letter that shows nothing", () => {
  it("never leaves the page through a copy or a cut, whatever pieces it stands beside", () => {
    document.body.innerHTML = `<article class="page">${EVERY_PIECE}</article>`;
    let text = EVERY_BODY;
    const written: string[] = [];
    const writing = writtenOn(document, {
      text: () => text,
      write: (next) => {
        text = next;
        written.push(next);
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
    const sheet = document.querySelector(".page") as Element;
    expect(sheet.textContent?.split("\u200B")).toHaveLength(6);
    const words = (src: string) =>
      document.querySelector(`[data-src="${src}"]`)?.firstChild as Text;
    document.getSelection()?.setBaseAndExtent(words("0-4"), 0, words("103-105"), 1);
    const clipped: string[] = [];
    for (const type of ["copy", "cut"]) {
      const asked = new Event(type, { bubbles: true, cancelable: true });
      Object.defineProperty(asked, "clipboardData", {
        value: { setData: (_: string, said: string) => clipped.push(said) },
      });
      sheet.dispatchEvent(asked);
    }
    writing.stop();
    expect(clipped).toHaveLength(2);
    expect(written.length).toBeGreaterThan(0);
    for (const said of [...clipped, ...written]) expect(said).not.toContain("\u200B");
  });

  it("copies nothing for a choice of the letter alone, and leaves the page with writing", () => {
    document.body.innerHTML = `<article class="page">${EVERY_PIECE}</article>`;
    const writing = writtenOn(document, {
      text: () => EVERY_BODY,
      write: () => {},
      shown: () => {},
      redraw: () => {},
      near: () => Promise.reject(new Error("no core here")),
      dressed: () => {},
      translated: (_, plain) => Promise.resolve(plain),
      ask: () => {},
      blocks: () => {},
      acts: () => {},
    });
    const sheet = document.querySelector(".page") as Element;
    const letter = document.querySelector('[data-src="4-8"]')?.nextSibling as Text;
    document.getSelection()?.setBaseAndExtent(letter, 0, letter, 1);
    const clipped: string[] = [];
    const asked = new Event("copy", { bubbles: true, cancelable: true });
    Object.defineProperty(asked, "clipboardData", {
      value: { setData: (_: string, said: string) => clipped.push(said) },
    });
    sheet.dispatchEvent(asked);
    expect(asked.defaultPrevented).toBe(true);
    expect(clipped).toEqual([""]);
    writing.stop();
    expect(sheet.textContent).not.toContain("\u200B");
  });
});

describe("the letter ahead of a caret", () => {
  it("is found from the end inside the marks around a piece, and only there", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-30"><span data-src="0-4">uno </span><strong><span data-src="6-10">bold </span>' +
      '<span class="tag" data-src="10-14" contenteditable="false">#eti</span></strong><br data-src="16-18" contenteditable="false">' +
      '<span data-src="18-21">dos</span></p></article>';
    opened(document);
    const line = document.querySelector("p") as Element;
    const bold = document.querySelector("strong") as Element;
    expect(letterAhead(bold, bold.childNodes.length)).toEqual([line, 2]);
    expect(letterAhead(line, 2)).toEqual([line, 2]);
    expect(letterAhead(bold, 1)).toBeNull();
    expect(letterAhead(line, 3)).toBeNull();
  });
});
