import { afterEach, describe, expect, it } from "vitest";
import { takenOver } from "../ui/pagePatching";

const block = (html: string) => {
  const holder = document.createElement("div");
  holder.innerHTML = html;
  return holder.firstElementChild as Element;
};

afterEach(() => {
  document.body.innerHTML = "";
});

const OLD =
  '<p data-at="0-9"><span data-src="0-4">hola</span><b><span data-src="5-9">uno</span></b></p>';
const NEW =
  '<p data-at="0-11"><span data-src="0-6">hola!!</span><b><span data-src="7-11">dos</span></b></p>';

describe("a block taken over by its fresh drawing", () => {
  it("keeps the very same text nodes of a block of the same shape", () => {
    const old = block(OLD);
    document.body.append(old);
    const before = Array.from(old.querySelectorAll("span")).map((one) => one.firstChild);
    takenOver(old, block(NEW));
    const after = Array.from(old.querySelectorAll("span")).map((one) => one.firstChild);
    expect(after[0]).toBe(before[0]);
    expect(after[1]).toBe(before[1]);
    expect(old.textContent).toBe("hola!!dos");
    expect(old.getAttribute("data-at")).toBe("0-11");
    expect(old.querySelectorAll("span")[0].getAttribute("data-src")).toBe("0-6");
    expect(old.querySelectorAll("span")[1].getAttribute("data-src")).toBe("7-11");
  });

  it("drops an attribute the fresh block no longer has", () => {
    const old = block('<p data-at="0-4" class="x"><span data-src="0-4">hola</span></p>');
    takenOver(old, block('<p data-at="0-4"><span data-src="0-4">hola</span></p>'));
    expect(old.hasAttribute("class")).toBe(false);
  });

  it("shifts a caret after the edit point with the letters inserted", () => {
    const old = block('<p data-at="0-9"><span data-src="0-9">hola mundo</span></p>');
    document.body.append(old);
    const text = old.querySelector("span")?.firstChild as Text;
    document.getSelection()?.collapse(text, 8);
    takenOver(old, block('<p data-at="0-11"><span data-src="0-11">hola XXmundo</span></p>'));
    const selection = document.getSelection();
    expect(text.data).toBe("hola XXmundo");
    expect(selection?.anchorNode).toBe(text);
    expect(selection?.anchorOffset).toBe(10);
  });

  it("keeps a caret before the edit point where it was", () => {
    const old = block('<p data-at="0-9"><span data-src="0-9">hola mundo</span></p>');
    document.body.append(old);
    const text = old.querySelector("span")?.firstChild as Text;
    document.getSelection()?.collapse(text, 2);
    takenOver(old, block('<p data-at="0-11"><span data-src="0-11">hola XXmundo</span></p>'));
    expect(document.getSelection()?.anchorNode).toBe(text);
    expect(document.getSelection()?.anchorOffset).toBe(2);
  });

  it("changes only the differing middle when letters are removed", () => {
    const old = block('<p data-at="0-9"><span data-src="0-9">hola mundo</span></p>');
    const text = old.querySelector("span")?.firstChild as Text;
    takenOver(old, block('<p data-at="0-6"><span data-src="0-6">hola o</span></p>'));
    expect(old.querySelector("span")?.firstChild).toBe(text);
    expect(text.data).toBe("hola o");
  });

  it("replaces the nodes when a span was added", () => {
    const old = block('<p data-at="0-4"><span data-src="0-4">hola</span></p>');
    const was = old.querySelector("span")?.firstChild;
    takenOver(
      old,
      block(
        '<p data-at="0-8"><span data-src="0-4">hola</span><span data-src="4-8">uno </span></p>',
      ),
    );
    expect(old.querySelectorAll("span")).toHaveLength(2);
    expect(old.querySelector("span")?.firstChild).not.toBe(was);
    expect(old.textContent).toBe("holauno ");
  });

  it("replaces the nodes when a tag differs", () => {
    const old = block('<p data-at="0-4"><span data-src="0-4">hola</span></p>');
    takenOver(old, block('<p data-at="0-6"><em><span data-src="1-5">hola</span></em></p>'));
    expect(old.querySelector("em")).not.toBeNull();
    expect(old.querySelector(":scope > span")).toBeNull();
  });

  it("replaces the nodes when a text node became an element", () => {
    const old = block('<p data-at="0-4">hola</p>');
    takenOver(old, block('<p data-at="0-4"><span data-src="0-4">hola</span></p>'));
    expect(old.firstElementChild?.tagName).toBe("SPAN");
  });
});
