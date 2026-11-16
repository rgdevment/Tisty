import { afterEach, describe, expect, it } from "vitest";
import { fieldAsked } from "../ui/pageFields";

afterEach(() => {
  document.body.innerHTML = "";
  document.documentElement.style.zoom = "";
});

const near = { bottom: 40, left: 30 } as DOMRect;

const field = () => document.querySelector<HTMLInputElement>(".page-address");

const key = (one: HTMLElement | null, name: string) =>
  one?.dispatchEvent(new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true }));

describe("a field asked for under the words of the page", () => {
  it("hands back what is written once Enter is pressed, trimmed, and goes away", async () => {
    const answer = fieldAsked(document, near, "Address", { kind: "url" });
    const one = field();
    expect(one?.type).toBe("url");
    expect(one?.placeholder).toBe("Address");
    expect(document.activeElement).toBe(one);
    if (one) one.value = "  https://tisty.app  ";
    key(one, "Enter");
    expect(await answer).toBe("https://tisty.app");
    expect(field()).toBeNull();
  });

  it("starts with the words it is given, chosen, to write over them", () => {
    void fieldAsked(document, near, "Hint", { value: "The first step" });
    const one = field();
    expect(one?.value).toBe("The first step");
    expect(one?.selectionStart).toBe(0);
    expect(one?.selectionEnd).toBe("The first step".length);
  });

  it("hands back nothing on Escape, when left, or when left empty", async () => {
    const escaped = fieldAsked(document, near, "Address");
    key(field(), "Escape");
    expect(await escaped).toBeNull();
    const left = fieldAsked(document, near, "Address");
    field()?.dispatchEvent(new FocusEvent("blur"));
    expect(await left).toBeNull();
    const empty = fieldAsked(document, near, "Address");
    key(field(), "Enter");
    expect(await empty).toBeNull();
  });

  it("stands under the words as the page is zoomed, and lets other keys through", () => {
    document.documentElement.style.zoom = "2";
    void fieldAsked(document, near, "Address");
    const one = field();
    expect(one?.style.top).toBe(`${40 / 2 + 6}px`);
    expect(one?.style.left).toBe(`${30 / 2}px`);
    const typed = new KeyboardEvent("keydown", { key: "a", bubbles: true, cancelable: true });
    one?.dispatchEvent(typed);
    expect(typed.defaultPrevented).toBe(false);
    expect(field()).not.toBeNull();
  });
});
