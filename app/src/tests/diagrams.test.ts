import { beforeEach, describe, expect, it, vi } from "vitest";
import { diagramShot, diagrams, drawing } from "../ui/diagrams";

const drawn = vi.hoisted(() => ({
  themes: [] as string[],
  sources: [] as string[],
  busy: false,
  overlapped: false,
}));

vi.mock("mermaid", () => ({
  default: {
    initialize: (config: { theme: string }) => {
      drawn.themes.push(config.theme);
    },
    render: async (_: string, source: string) => {
      if (drawn.busy) drawn.overlapped = true;
      drawn.busy = true;
      drawn.sources.push(source);
      await new Promise((done) => setTimeout(done, 1));
      drawn.busy = false;
      if (source.includes("broken")) throw new Error("Parse error");
      return { svg: `<svg xmlns="http://www.w3.org/2000/svg"><text>${source}</text></svg>` };
    },
  },
}));

const sheet = (...sources: string[]) => {
  document.body.innerHTML = `<article class="page">${sources
    .map(
      (one, at) =>
        `<figure class="diagram" data-at="${at}-${at + 1}"><pre><code class="language-mermaid">${one}</code></pre></figure>`,
    )
    .join("")}</article>`;
  return document;
};

beforeEach(() => {
  drawn.themes.length = 0;
  drawn.sources.length = 0;
  drawn.overlapped = false;
});

describe("a diagram on a page", () => {
  it("is drawn in place of its words, once for each look", async () => {
    const page = sheet("graph TD; Rina-->Walk");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    expect(figure.classList.contains("drawn")).toBe(true);
    expect(figure.querySelector(".diagram-sheet svg text")?.textContent).toBe(
      "graph TD; Rina-->Walk",
    );
    expect(figure.querySelector("pre")).toBeNull();

    await diagrams(sheet("graph TD; Rina-->Walk"), false, () => true);
    expect(drawn.sources).toEqual(["graph TD; Rina-->Walk"]);
    expect(drawn.themes).toEqual(["default"]);
  });

  it("is drawn again when the look turns dark", async () => {
    const page = sheet("graph LR; Day-->Night");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    await diagrams(page, true, () => true);
    expect(figure.getAttribute("data-theme")).toBe("dark");
    expect(drawn.themes).toEqual(["default", "dark"]);
    expect(decodeURIComponent(diagramShot(figure) ?? "")).toContain("Day--&gt;Night");
  });

  it("stays as the words it is made of when it cannot be drawn", async () => {
    const page = sheet("graph TD; broken");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    expect(figure.classList.contains("unreadable")).toBe(true);
    expect(figure.querySelector("pre code")?.textContent).toBe("graph TD; broken");
  });

  it("is left alone when the page it was asked for is gone", async () => {
    const page = sheet("graph TD; Gone-->Away");
    await diagrams(page, false, () => false);
    expect(page.querySelector("figure.diagram.drawn")).toBeNull();
  });

  it("takes its turn, since the drawer keeps one look for the whole window", async () => {
    await Promise.all([
      drawing("graph TD; One-->Two", false),
      drawing("graph TD; Two-->Three", true),
      drawing("graph TD; Three-->Four", false),
    ]);
    expect(drawn.overlapped).toBe(false);
    expect(drawn.themes).toEqual(["default", "dark", "default"]);
  });

  it("is not drawn when it is empty or too long to lay out", async () => {
    expect(await drawing("   ", false)).toBeNull();
    expect(await drawing(`graph TD\n${"A-->B\n".repeat(8000)}`, false)).toBeNull();
    expect(await drawing(`graph TD\n${"A-->B;".repeat(1200)}`, false)).toBeNull();
    expect(drawn.sources).toEqual([]);
  });

  it("keeps the look last asked for when an older drawing lands late", async () => {
    const page = sheet("graph TD; Late-->Look");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    const dark = diagrams(page, true, () => true);
    await diagrams(page, false, () => true);
    await dark;
    expect(figure.getAttribute("data-theme")).toBe("light");
  });

  it("skips a drawing nobody waits for any more", async () => {
    const page = sheet("graph TD; Typed-->Once");
    const first = diagrams(page, false, () => true);
    sheet("graph TD; Typed-->Twice");
    await first;
    expect(drawn.sources).toEqual([]);
  });

  it("shows a dark drawing in the viewer on the dark sheet it was drawn for", async () => {
    const page = sheet("graph TD; Night-->Owl");
    await diagrams(page, true, () => true);
    const shot = decodeURIComponent(
      diagramShot(page.querySelector("figure.diagram") as Element) ?? "",
    );
    expect(shot).toContain("<rect");
    expect(shot).toContain("Night--&gt;Owl");
  });
});
