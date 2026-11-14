import { afterEach, describe, expect, it, vi } from "vitest";
import type { Reaching } from "../ui/dressing";
import { wornLaid } from "../ui/pageDressing";

vi.mock("../inUse", async (original) => ({
  ...(await original<typeof import("../inUse")>()),
  serve: (reference: string) => Promise.resolve(`C:/kept/${reference}`),
}));

vi.mock("@tauri-apps/api/core", async (original) => ({
  ...(await original<typeof import("@tauri-apps/api/core")>()),
  convertFileSrc: (path: string) => `asset://${path}`,
}));

const REACH = {
  body: "",
  onDoc: () => {},
  onTable: () => {},
  onWiden: () => {},
  stepDates: { segundo: "Vence mañana" },
} as unknown as Reaching;

const TABLE =
  '<div class="table" data-at="40-70"><table><thead><tr><th data-type="text">a</th></tr></thead><tbody><tr><td>1</td></tr></tbody></table></div>';

afterEach(() => {
  document.body.innerHTML = "";
});

const pageOf = (inner: string) => {
  document.body.innerHTML = `<article class="page">${inner}</article>`;
  return document.querySelector(".page") as Element;
};

describe("the blocks the core draws again on the page written in place", () => {
  it("are read as the whole page is, and stay where they were laid", () => {
    const sheet = pageOf(
      '<p data-at="0-5">Antes</p><blockquote class="callout" data-callout="tip" data-at="7-20"><p>Ojo</p></blockquote><p data-at="22-28">Después</p>',
    );
    const laid = [sheet.children[1] as Element];
    wornLaid(document, laid, () => true, REACH);
    const children = Array.from(sheet.children);
    expect(children[1]).toBe(laid[0]);
    expect(children.map((one) => one.textContent)).toEqual(["Antes", "Ojo", "Después"]);
    expect((laid[0] as HTMLElement).dataset.said).toBeTruthy();
  });

  it("dress a table laid at the top of the page, the block itself", () => {
    const sheet = pageOf(TABLE);
    wornLaid(document, [sheet.children[0] as Element], () => true, REACH);
    expect(sheet.querySelector(".table")?.classList.contains("dressed")).toBe(true);
    expect(sheet.querySelector(".table")?.classList.contains("celled")).toBe(true);
    expect(sheet.querySelector(".col-grip")).not.toBeNull();
  });

  it("count the stages again across the page, and give a step its date", () => {
    const sheet = pageOf(
      '<ul data-at="0-30"><li class="task" id="stage--1" data-at="0-10">primero</li></ul>' +
        '<ul data-at="32-50"><li class="task" data-at="32-50">segundo</li></ul>',
    );
    const laid = [sheet.children[1] as Element];
    wornLaid(document, laid, () => true, REACH);
    const stages = Array.from(sheet.querySelectorAll("li.task"), (one) => one.id);
    expect(stages).toEqual(["stage--1", "stage--2"]);
    expect(sheet.querySelectorAll("li.task")[1]?.getAttribute("title")).toBe("Vence mañana");
  });
});
