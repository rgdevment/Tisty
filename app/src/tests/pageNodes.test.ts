import { afterEach, describe, expect, it } from "vitest";
import { opened } from "../ui/pageNodes";
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
