import { afterEach, expect, it } from "vitest";
import { wired, worn } from "../ui/dressing";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.documentElement.classList.remove("untitled");
  document.body.innerHTML = "";
});

const empty = () => {
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
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: (_, plain) => Promise.resolve(plain),
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
