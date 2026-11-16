import { afterEach, describe, expect, it } from "vitest";
import { opened } from "../ui/pageNodes";

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
});
