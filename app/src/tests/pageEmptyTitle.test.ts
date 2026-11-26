import { afterEach, expect, it, vi } from "vitest";
import { wired, worn } from "../ui/dressing";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.documentElement.classList.remove("untitled");
  document.body.innerHTML = "";
});

const empty = (redraw: (now: string) => void = () => {}) => {
  let text = "";
  document.body.innerHTML = '<article class="page"></article>';
  const reach = {
    body: "",
    onDoc: () => {},
    onBegin: (at: number, seed: string, near: Element) => writing?.begin(at, seed, near),
  };
  worn(document, () => true, reach);
  wired(document, () => reach);
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw,
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
  });
  return () => text;
};

const sheet = () => document.querySelector(".page") as HTMLElement;
const slot = () => document.querySelector<HTMLElement>(".page > h1[data-title]");
const press = (on: Element | null) => {
  for (const kind of ["mousedown", "mouseup", "click"])
    on?.dispatchEvent(
      new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, clientY: 900 }),
    );
};
const key = (inputType: string, data: string | null = null) =>
  sheet().dispatchEvent(
    new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true }),
  );

it("keeps the title's slot and writes in it after a press elsewhere on the page", () => {
  const text = empty();
  press(slot());
  press(sheet());
  expect(slot()).not.toBeNull();
  expect(document.querySelector("[data-ghost]")).toBe(slot());
  expect(sheet().contains(document.getSelection()?.anchorNode ?? null)).toBe(true);
  key("insertText", "O");
  expect(text()).toBe("# O");
});

it("keeps the title's slot on a press back, and the next key is still the title's", () => {
  const text = empty();
  press(slot());
  key("deleteContentBackward");
  expect(slot()).not.toBeNull();
  key("insertText", "O");
  expect(text()).toBe("# O");
});

it("begins a task's first line inside the sheet", () => {
  document.documentElement.classList.add("untitled");
  const text = empty();
  press(sheet());
  press(sheet());
  expect(sheet().firstElementChild?.matches("[data-ghost]")).toBe(true);
  expect(sheet().querySelector(".add-section")).not.toBeNull();
  expect(document.querySelectorAll("[data-ghost]")).toHaveLength(1);
  key("insertText", "O");
  expect(text()).toBe("O");
});

it("waits at the title again once what was written is erased, also after Enter", async () => {
  const drawn = (now: string) => {
    if (now) return;
    sheet().innerHTML = "";
    worn(document, () => true, { body: "", onDoc: () => {}, onBegin: () => {} });
    writing?.drawn();
  };
  const text = empty(drawn);
  press(slot());
  key("insertText", "O");
  sheet().innerHTML = '<h1 data-at="0-3"><span data-src="2-3">O</span></h1>';
  document.getSelection()?.collapse(sheet().querySelector("span")?.firstChild as Text, 1);
  key("deleteContentBackward");
  expect(text()).toBe("");
  await vi.waitFor(() => expect(slot()).not.toBeNull());
  expect(document.querySelector("[data-ghost]")).toBe(slot());
  expect(slot()?.querySelector("br")).toBeNull();
  key("insertParagraph");
  expect(text()).toBe("");
  key("insertText", "H");
  expect(text()).toBe("# H");
});
