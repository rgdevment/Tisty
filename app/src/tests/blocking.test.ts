import { describe, expect, it } from "vitest";
import { edited } from "../ui/blocking";

const block = () => {
  const page = document.createElement("article");
  page.className = "page";
  const one = document.createElement("p");
  one.textContent = "Leche y pan.";
  page.append(one);
  document.body.append(page);
  return { page, one };
};

describe("writing a block in place", () => {
  it("takes the block's place with its words as written, and hands back each change", () => {
    const { page, one } = block();
    const changes: string[] = [];
    const editing = edited(
      one,
      "Leche *y* pan.\n",
      (text) => changes.push(text),
      () => {},
    );
    expect(page.querySelector(".editing .cm-editor")).not.toBeNull();
    expect(editing.view.state.doc.toString()).toBe("Leche *y* pan.");
    editing.view.dispatch({ changes: { from: 0, insert: "Más " } });
    expect(changes).toEqual(["Más Leche *y* pan."]);
    editing.leave();
    page.remove();
  });

  it("hands back the words once when it is left, however often it is asked to leave", () => {
    const { page, one } = block();
    const left: string[] = [];
    const editing = edited(
      one,
      "Hola",
      () => {},
      (text) => left.push(text),
    );
    editing.view.dispatch({ changes: { from: 4, insert: "!" } });
    editing.leave();
    editing.leave();
    expect(left).toEqual(["Hola!"]);
    page.remove();
  });

  it("is left with Escape", () => {
    const { page, one } = block();
    const left: string[] = [];
    const editing = edited(
      one,
      "Hola",
      () => {},
      (text) => left.push(text),
    );
    editing.view.contentDOM.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
    );
    expect(left).toEqual(["Hola"]);
    page.remove();
  });

  it("puts the drawn block back once it is left, so nothing is missing until the page is drawn again", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "Leche y pan.",
      () => {},
      () => {},
    );
    expect(page.contains(one)).toBe(false);
    editing.leave();
    expect(page.contains(one)).toBe(true);
    expect(page.querySelector(".editing")).toBeNull();
    page.remove();
  });

  it("shows pens, underlines and icons as they will read, but never inside code or for unknown names", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "==a== ==b=={green} ==c=={silver} ++d++ ::dog orange:: ::star:: ::nope:: ::dog plaid:: `==e==`",
      () => {},
      () => {},
    );
    const classes = Array.from(page.querySelectorAll(".editing .cm-content span")).map(
      (one) => `${one.className}:${one.textContent}`,
    );
    expect(classes).toContain("cm-pen cm-pen-yellow:==a==");
    expect(classes).toContain("cm-pen cm-pen-green:==b=={green}");
    expect(classes).toContain("cm-under:++d++");
    expect(classes).toContain("cm-ico ico-orange:::dog orange::");
    expect(classes).toContain("cm-ico:::star::");
    expect(classes.some((one) => one.includes("silver"))).toBe(false);
    expect(classes.some((one) => one.startsWith("cm-ico") && one.includes("nope"))).toBe(false);
    expect(classes.some((one) => one.startsWith("cm-ico") && one.includes("plaid"))).toBe(false);
    expect(classes.some((one) => one.startsWith("cm-pen") && one.includes("==e=="))).toBe(false);
    editing.leave();
    page.remove();
  });

  it("paints nothing in a block shown as it was written", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "==a=={green} ++b++",
      () => {},
      () => {},
      false,
    );
    expect(page.querySelector(".editing .cm-pen, .editing .cm-under")).toBeNull();
    editing.leave();
    page.remove();
  });

  it("keeps the line endings the block was written with", () => {
    const { page, one } = block();
    const changes: string[] = [];
    const editing = edited(
      one,
      "uno\r\ndos\r\n",
      (text) => changes.push(text),
      () => {},
    );
    expect(editing.view.state.doc.lines).toBe(2);
    editing.view.dispatch({ changes: { from: 3, insert: "!" } });
    expect(changes).toEqual(["uno!\r\ndos"]);
    editing.leave();
    page.remove();
  });
});
