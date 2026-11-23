import { afterEach, describe, expect, it } from "vitest";
import { besideBox, opened } from "../ui/pageNodes";
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
  it("leaves the caret an empty place after it, past the marks around it, once", () => {
    document.body.innerHTML = `<article class="page">${ENDS}</article>`;
    opened(document);
    for (const one of ['[data-src="6-10"]', '[data-src="27-32"]', "strong", '[data-src="67-72"]']) {
      expect(nextOf(one)?.nodeType).toBe(Node.TEXT_NODE);
      expect(nextOf(one)?.textContent).toBe("");
    }
    expect(nextOf('[data-src="49-53"]')).toBeNull();
    expect(nextOf('[data-src="81-86"]')?.textContent).toBe(" medio");
    expect(nextOf('[data-src="100-104"]')?.nodeName).toBe("BR");
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
      "Antes #etiuno codAntes #etiAntes x\u200By medio#etidos",
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
    writing.stop();
    expect(document.querySelector('[data-src="6-9"]')?.lastChild?.textContent).toBe("\u200B");
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
