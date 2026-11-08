import { beforeEach, describe, expect, it, vi } from "vitest";
import { DARK_PIE, diagramShot, diagrams, drawing } from "../src/diagrams";

const drawn = vi.hoisted(() => ({
  themes: [] as string[],
  variables: [] as (Record<string, string> | undefined)[],
  sources: [] as string[],
  busy: false,
  overlapped: false,
}));

vi.mock("mermaid", () => ({
  default: {
    initialize: (config: { theme: string; themeVariables?: Record<string, string> }) => {
      drawn.themes.push(config.theme);
      drawn.variables.push(config.themeVariables);
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

const luminance = (hex: string) => {
  const [r, g, b] = [1, 3, 5].map((at) => {
    const one = Number.parseInt(hex.slice(at, at + 2), 16) / 255;
    return one <= 0.03928 ? one / 12.92 : ((one + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};

const contrast = (one: string, other: string) => {
  const [hi, lo] = [luminance(one), luminance(other)].sort((a, b) => b - a);
  return (hi + 0.05) / (lo + 0.05);
};

describe("a pie on a dark sheet", () => {
  it("gets slices that stand out from the sheet and from its words", async () => {
    await drawing('pie\n  "A" : 1\n  "B" : 2', true);

    const said = drawn.variables[drawn.variables.length - 1];
    expect(DARK_PIE.map((_, at) => said?.[`pie${at + 1}`])).toEqual(DARK_PIE);
    for (const slice of DARK_PIE) {
      expect(contrast(slice, "#232326")).toBeGreaterThan(4.5);
      expect(contrast(slice, said?.pieSectionTextColor ?? "#000000")).toBeGreaterThan(4.5);
    }
  });

  it("keeps Mermaid's own colours on a light sheet", async () => {
    await drawing('pie\n  "C" : 1', false);

    expect(drawn.variables[drawn.variables.length - 1]).toBeUndefined();
  });
});

beforeEach(() => {
  drawn.themes.length = 0;
  drawn.variables.length = 0;
  drawn.sources.length = 0;
  drawn.overlapped = false;
});

describe("a diagram on a page", () => {
  it("is drawn in place of its words, once for each look", async () => {
    const page = sheet("journey; Rina-->Walk");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    expect(figure.classList.contains("drawn")).toBe(true);
    expect(figure.querySelector(".diagram-sheet svg text")?.textContent).toBe(
      "journey; Rina-->Walk",
    );
    expect(figure.querySelector("pre")).toBeNull();

    await diagrams(sheet("journey; Rina-->Walk"), false, () => true);
    expect(drawn.sources).toEqual(["journey; Rina-->Walk"]);
    expect(drawn.themes).toEqual(["default"]);
  });

  it("is drawn again when the look turns dark", async () => {
    const page = sheet("journey; Day-->Night");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    await diagrams(page, true, () => true);
    expect(figure.getAttribute("data-theme")).toBe("dark");
    expect(drawn.themes).toEqual(["default", "dark"]);
    expect(decodeURIComponent(diagramShot(figure) ?? "")).toContain("Day--&gt;Night");
  });

  it("stays as the words it is made of when it cannot be drawn", async () => {
    const page = sheet("journey; broken");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    expect(figure.classList.contains("unreadable")).toBe(true);
    expect(figure.querySelector("pre code")?.textContent).toBe("journey; broken");
  });

  it("is left alone when the page it was asked for is gone", async () => {
    const page = sheet("journey; Gone-->Away");
    await diagrams(page, false, () => false);
    expect(page.querySelector("figure.diagram.drawn")).toBeNull();
  });

  it("takes its turn, since the drawer keeps one look for the whole window", async () => {
    await Promise.all([
      drawing("journey; One-->Two", false),
      drawing("journey; Two-->Three", true),
      drawing("journey; Three-->Four", false),
    ]);
    expect(drawn.overlapped).toBe(false);
    expect(drawn.themes).toEqual(["default", "dark", "default"]);
  });

  it("is drawn as the diagram window draws it when the window draws its kind, places and all", async () => {
    const svg = await drawing(
      'flowchart LR\n  A@{ shape: cloud, label: "Nube" } --> B[Fin]\n  %% tisty: A 300 100',
      false,
    );

    expect(svg).toContain(">Nube<");
    expect(svg).toMatch(/^<svg style="max-width:[\d.]+px" /);
    expect(drawn.sources).toEqual([]);
  });

  it("names its markers apart from another drawing's, so a line never reaches a hidden one", async () => {
    const flow = "flowchart TD\n  Apart --> Named\n";
    const markers = (svg: string | null) =>
      [...(svg ?? "").matchAll(/<marker id="([^"]+)"/g)].map((one) => one[1]);

    const light = await drawing(flow, false);
    const dark = await drawing(flow, true);

    expect(markers(light).length).toBeGreaterThan(0);
    expect(markers(light).filter((one) => markers(dark).includes(one))).toEqual([]);
    expect(markers(light).some((one) => light?.includes(`marker-end="url(#${one})"`))).toBe(true);
  });

  it("names its markers apart when the same drawing is placed twice", async () => {
    const page = sheet("flowchart TD\n  Twice --> Placed\n", "flowchart TD\n  Twice --> Placed\n");
    await diagrams(page, false, () => true);

    const [first, second] = [...page.querySelectorAll("figure.diagram")].map((figure) => ({
      markers: [...figure.querySelectorAll("marker")].map((one) => one.id),
      line: figure.querySelector("[marker-end]")?.getAttribute("marker-end"),
    }));

    expect(first.markers.length).toBeGreaterThan(0);
    expect(first.markers.filter((one) => second.markers.includes(one))).toEqual([]);
    expect(first.markers.map((one) => `url(#${one})`)).toContain(first.line);
    expect(second.markers.map((one) => `url(#${one})`)).toContain(second.line);
  });

  it("is not drawn when it is empty or too long to lay out", async () => {
    expect(await drawing("   ", false)).toBeNull();
    expect(await drawing(`graph TD\n${"A-->B\n".repeat(8000)}`, false)).toBeNull();
    expect(await drawing(`graph TD\n${"A-->B;".repeat(1200)}`, false)).toBeNull();
    expect(drawn.sources).toEqual([]);
  });

  it("keeps the look last asked for when an older drawing lands late", async () => {
    const page = sheet("journey; Late-->Look");
    await diagrams(page, false, () => true);
    const figure = page.querySelector("figure.diagram") as Element;
    const dark = diagrams(page, true, () => true);
    await diagrams(page, false, () => true);
    await dark;
    expect(figure.getAttribute("data-theme")).toBe("light");
  });

  it("skips a drawing nobody waits for any more", async () => {
    const page = sheet("journey; Typed-->Once");
    const first = diagrams(page, false, () => true);
    sheet("journey; Typed-->Twice");
    await first;
    expect(drawn.sources).toEqual([]);
  });

  it("shows a dark drawing in the viewer on the dark sheet it was drawn for", async () => {
    const page = sheet("journey; Night-->Owl");
    await diagrams(page, true, () => true);
    const shot = decodeURIComponent(
      diagramShot(page.querySelector("figure.diagram") as Element) ?? "",
    );
    expect(shot).toContain("<rect");
    expect(shot).toContain("Night--&gt;Owl");
  });
});
