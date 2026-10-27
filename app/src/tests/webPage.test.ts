import { describe, expect, it, vi } from "vitest";

const drawn = vi.hoisted(() => ({ asked: [] as string[], refused: false }));

vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  docDrawn: (body: string) =>
    Promise.resolve(
      `<h1 data-at="0-6">${body}</h1><p data-at="7-40">ver <a class="card" data-doc="x.md">Otro</a> y <a href="tisty:doc/y.md">aquel</a></p><p><img src="foto-0006bf09.png" alt="foto"><img src="acta-0006bf10.pdf" alt="acta"></p>`,
    ),
  attached: (reference: string) => {
    drawn.asked.push(reference);
    return drawn.refused ? Promise.reject(new Error("gone")) : Promise.resolve([137, 80, 78, 71]);
  },
}));

import { webPage } from "../ui/webPage";

describe("a document as one web page that travels alone", () => {
  it("carries its pictures inside and leaves the window's own links as words", async () => {
    const made = await webPage(["Plan", "Anexo"], "Plan & más");

    expect(made.startsWith("<!doctype html>")).toBe(true);
    expect(made).toContain("<title>Plan &amp; más</title>");
    expect(made).toContain('src="data:image/png;base64,iVBORw=="');
    expect(made).toContain("Anexo");
    expect(made).not.toContain("data-at");
    expect(made).not.toContain("tisty:doc");
    expect(made).not.toContain('class="card"');
    expect(made).not.toContain("inert");
    expect(made).toContain("acta");
    expect(drawn.asked).toEqual(["foto-0006bf09.png", "foto-0006bf09.png"]);
  });

  it("names a picture it cannot read instead of pointing at nothing", async () => {
    drawn.refused = true;
    const made = await webPage(["Plan"], "Plan");

    expect(made).not.toContain("foto-0006bf09.png");
    expect(made).toContain("foto");
  });
});

describe("a document as a page on paper", () => {
  it("takes the sheet, folds what folds on screen, opens with a cover and credits Tisty at the end", async () => {
    drawn.refused = false;
    const light = document.createElement("style");
    light.textContent = ":root{--tisty-ink:#123456}";
    document.head.append(light);
    const made = await webPage(["Plan<details><summary>Más</summary>x</details>"], "Plan", {
      leaf: "a4",
      stamp: { card: [{ label: "Autor", said: "Ana Pérez" }], credited: true },
    });
    light.remove();
    const sheet = new DOMParser().parseFromString(made, "text/html").querySelector("article.page");

    expect(made).toContain("@page{size:A4");
    expect(made).not.toContain("<details open");
    const cover = sheet?.firstElementChild;
    expect(cover?.className).toBe("cover");
    expect(cover?.querySelector(".stamp")?.textContent).toBe("AutorAna Pérez");
    expect(cover?.nextElementSibling?.tagName).toBe("H1");
    expect(sheet?.lastElementChild?.className).toBe("credit");
  });

  it("turns the paper where the screen's sheets turn, and only in the first body", async () => {
    const made = await webPage(["Plan", "Anexo"], "Plan", { leaf: "a4", turns: ["7-40"] });
    const sheet = new DOMParser().parseFromString(made, "text/html").querySelector("article.page");
    const turned = sheet?.querySelectorAll(".turned");

    expect(turned).toHaveLength(1);
    expect(turned?.[0].textContent).toContain("ver");
    expect(made).toContain(".page>.turned{break-before:page");
  });

  it("marks a table too wide for the paper, only in the first body, so it alone breaks its words", async () => {
    const made = await webPage(["Plan", "Anexo"], "Plan", { leaf: "a4", wide: ["7-40"] });
    const sheet = new DOMParser().parseFromString(made, "text/html").querySelector("article.page");

    expect(sheet?.querySelectorAll(".wide")).toHaveLength(1);
  });

  it("leaves the card and the credit out unless asked for", async () => {
    const made = await webPage(["Plan"], "Plan", { leaf: "a4", stamp: { credited: false } });

    expect(made).not.toContain('class="cover"');
    expect(made).not.toContain('class="credit"');
  });
});
