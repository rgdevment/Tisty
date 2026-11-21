import { afterEach, describe, expect, it } from "vitest";
import { unnavigated } from "../unnavigated";

afterEach(() => {
  document.body.innerHTML = "";
});

const pressed = (on: HTMLElement, key = "Backspace") => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  on.addEventListener("keydown", unnavigated);
  on.dispatchEvent(event);
  return event.defaultPrevented;
};

describe("Backspace outside anything written in", () => {
  it("never walks the window back to an earlier page", () => {
    document.body.innerHTML = "<button>Abrir</button>";
    expect(pressed(document.querySelector("button") as HTMLElement)).toBe(true);
  });

  it("still deletes in a field or on the page being written", () => {
    document.body.innerHTML =
      '<input value="hola"><textarea>hola</textarea><div contenteditable="true"><p>hola</p></div>';
    expect(pressed(document.querySelector("input") as HTMLElement)).toBe(false);
    expect(pressed(document.querySelector("textarea") as HTMLElement)).toBe(false);
    const page = document.querySelector("[contenteditable]") as HTMLElement;
    Object.defineProperty(page, "isContentEditable", { value: true });
    expect(pressed(page)).toBe(false);
  });

  it("lets a key the page hands to the window through untouched", () => {
    const passed = new KeyboardEvent("keydown", { key: "Backspace", cancelable: true });
    window.addEventListener("keydown", unnavigated);
    window.dispatchEvent(passed);
    window.removeEventListener("keydown", unnavigated);
    expect(passed.defaultPrevented).toBe(false);
  });

  it("never walks back from a box, a choice or a slider either", () => {
    document.body.innerHTML =
      '<input type="checkbox"><input type="radio"><input type="range"><select><option>a</option></select>';
    for (const one of document.querySelectorAll<HTMLElement>("input, select"))
      expect(pressed(one), one.outerHTML).toBe(true);
  });

  it("never walks back from a field that takes no words, as one shown only to read", () => {
    document.body.innerHTML = '<input value="ruta" readonly><textarea disabled>hola</textarea>';
    expect(pressed(document.querySelector("input") as HTMLElement)).toBe(true);
    expect(pressed(document.querySelector("textarea") as HTMLElement)).toBe(true);
  });

  it("never walks back when the caret stands in a piece of the page that takes no words, as a table", () => {
    document.body.innerHTML =
      '<div contenteditable="true"><p>hola</p><div contenteditable="false"><table><tr><td>3</td></tr></table></div></div>';
    const page = document.querySelector("[contenteditable]") as HTMLElement;
    Object.defineProperty(page, "isContentEditable", { value: true });
    document.getSelection()?.collapse(document.querySelector("td")?.firstChild as Text, 0);
    expect(pressed(page)).toBe(true);
    document.getSelection()?.collapse(document.querySelector("p")?.firstChild as Text, 2);
    expect(pressed(page)).toBe(false);
  });

  it("leaves other keys alone", () => {
    document.body.innerHTML = "<button>Abrir</button>";
    expect(pressed(document.querySelector("button") as HTMLElement, "Delete")).toBe(false);
  });
});
