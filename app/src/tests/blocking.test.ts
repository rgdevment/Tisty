import { completionStatus, startCompletion } from "@codemirror/autocomplete";
import { describe, expect, it, vi } from "vitest";
import { caretAt, edited } from "../ui/blocking";
import { wrapped } from "../ui/formatting";

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

  it("keeps files pasted from the clipboard as attachments, and leaves pasted words alone", async () => {
    const { page, one } = block();
    const pasted: string[] = [];
    let lost = 0;
    const editing = edited(
      one,
      "Ver",
      () => {},
      () => {},
      () => ({
        kinds: [],
        ask: () => {},
        pasted: (files) => {
          pasted.push(...files.map((file) => file.name));
          return Promise.resolve("![captura](captura.png)");
        },
        lost: () => {
          lost += 1;
        },
      }),
    );
    const view = editing.view;
    view.dispatch({ selection: { anchor: 3 } });
    const paste = (files: File[]) => {
      const event = new Event("paste", { bubbles: true, cancelable: true });
      Object.defineProperty(event, "clipboardData", { value: { files, getData: () => "" } });
      view.contentDOM.dispatchEvent(event);
      return event;
    };
    paste([]);
    expect(pasted).toEqual([]);
    const sheet = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(sheet, "clipboardData", {
      value: { files: [new File(["x"], "image.png")], getData: () => "A1\tB1" },
    });
    view.contentDOM.dispatchEvent(sheet);
    expect(pasted).toEqual([]);
    expect(paste([new File(["x"], "captura.png")]).defaultPrevented).toBe(true);
    await vi.waitFor(() =>
      expect(view.state.sliceDoc()).toBe("VerA1\tB1\n\n![captura](captura.png)"),
    );
    expect(pasted).toEqual(["captura.png"]);
    paste([new File(["x"], "tarde.png")]);
    editing.leave();
    await vi.waitFor(() => expect(lost).toBe(1));
    page.remove();
  });

  it("redoes with Ctrl+Shift+Z as well as Ctrl+Y", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "Hola",
      () => {},
      () => {},
    );
    const view = editing.view;
    view.dispatch({ changes: { from: 4, insert: "!" }, userEvent: "input.type" });
    view.contentDOM.dispatchEvent(
      new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true, cancelable: true }),
    );
    expect(view.state.sliceDoc()).toBe("Hola");
    view.contentDOM.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Z",
        ctrlKey: true,
        shiftKey: true,
        keyCode: 90,
        bubbles: true,
        cancelable: true,
      }),
    );
    expect(view.state.sliceDoc()).toBe("Hola!");
    editing.leave();
    page.remove();
  });

  it("closes the / menu first with Escape, then leaves, and tidies a format typed with nothing selected", async () => {
    const { page, one } = block();
    const left: string[] = [];
    const editing = edited(
      one,
      "Esto es ",
      () => {},
      (text) => left.push(text),
    );
    const view = editing.view;
    const pressEscape = () =>
      view.contentDOM.dispatchEvent(
        new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
      );
    view.dispatch({ selection: { anchor: 8 } });
    wrapped(view, "==");
    view.dispatch({ changes: { from: 10, insert: "juan " }, selection: { anchor: 15 } });
    view.dispatch({ changes: { from: 17, insert: " /" }, selection: { anchor: 19 } });
    startCompletion(view);
    await vi.waitFor(() => expect(completionStatus(view.state)).toBe("active"));
    pressEscape();
    expect(left).toEqual([]);
    pressEscape();
    expect(left).toEqual(["Esto es ==juan==  /"]);
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

  it("ends a template's label on Enter and keeps its name to twenty characters", () => {
    const { page, one } = block();
    const left = vi.fn();
    const editing = edited(one, "## Notas", () => {}, left, undefined, true);
    editing.view.dispatch({ changes: { from: 8, insert: " de la semana pasada" } });
    expect(editing.view.state.doc.toString()).toBe("## Notas");
    editing.view.dispatch({ changes: { from: 8, insert: " del mes" } });
    expect(editing.view.state.doc.toString()).toBe("## Notas del mes");
    page.remove();
  });

  it("keeps the bar over the page and the keys once at its end, never inside the block, and takes both away", () => {
    const { page, one } = block();
    const editing = edited(
      one,
      "Leche y pan.",
      () => {},
      () => {},
    );
    const bar = document.querySelector<HTMLElement>(".cm-bar.cm-dock");
    expect(bar?.parentElement).toBe(document.body);
    expect(page.lastElementChild?.matches(".cm-hint.cm-dock")).toBe(true);
    expect(page.querySelector(".editing .cm-bar, .editing .cm-hint")).toBeNull();
    expect(bar?.style.top).not.toBe("");
    editing.leave();
    expect(document.querySelector(".cm-dock")).toBeNull();
    page.remove();
  });

  it("shows pens, underlines and icons as they will read, marks hidden, but never inside code or for unknown names", () => {
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
    expect(classes).toContain("cm-pen cm-pen-green:b");
    expect(classes).toContain("cm-under:d");
    const icons = Array.from(page.querySelectorAll(".editing .cm-glyph")).map(
      (one) => one.className,
    );
    expect(icons).toEqual(["cm-glyph ico-orange", "cm-glyph"]);
    expect(page.querySelector(".editing .cm-content")?.textContent).toContain("==c=={silver}");
    expect(classes.some((one) => one.startsWith("cm-pen") && one.includes("silver"))).toBe(false);
    expect(page.querySelector(".editing .cm-content")?.textContent).toContain("::nope::");
    expect(page.querySelector(".editing .cm-content")?.textContent).toContain("::dog plaid::");
    expect(classes.some((one) => one.startsWith("cm-pen") && one.includes("==e=="))).toBe(false);
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

describe("the caret a click leaves in a block", () => {
  it("sits after the words read before the click, past the marks the page does not show", () => {
    expect(caretAt("Leche y **pan** fresco", "Leche y pan")).toBe(13);
    expect(caretAt("# Compras del mes", "Compras")).toBe(9);
    expect(caretAt("uno, dos, uno, dos", "uno, dos, uno")).toBe(13);
  });

  it("starts at the beginning when nothing was read, and at the end when the words are not found", () => {
    expect(caretAt("Leche", "")).toBe(0);
    expect(caretAt("Leche", "§")).toBe(5);
  });
});
