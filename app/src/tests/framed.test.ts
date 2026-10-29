import { afterEach, describe, expect, it } from "vitest";
import { dressedAsWindow, framed } from "../ui/framed";

const page = (html: string): Document => new DOMParser().parseFromString(html, "text/html");

afterEach(() => document.documentElement.removeAttribute("data-theme"));

describe("a page in the window", () => {
  it("is built in the window's theme", () => {
    document.documentElement.dataset.theme = "dark";
    expect(page(framed("<p>x</p>")).documentElement.classList.contains("dark")).toBe(true);
    document.documentElement.dataset.theme = "light";
    expect(page(framed("<p>x</p>")).documentElement.classList.contains("dark")).toBe(false);
  });

  it("turns as the window turns", () => {
    const drawn = page(framed("<p>x</p>"));
    document.documentElement.dataset.theme = "dark";
    dressedAsWindow(drawn);
    expect(drawn.documentElement.classList.contains("dark")).toBe(true);
    document.documentElement.dataset.theme = "light";
    dressedAsWindow(drawn);
    expect(drawn.documentElement.classList.contains("dark")).toBe(false);
  });
});
