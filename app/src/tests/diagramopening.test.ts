import { afterEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { diagrams } from "../ui/diagrams";
import { answered, DOUBLE_MS, diagramBars, type Reaching, wired, worn } from "../ui/dressing";
import { shell, went } from "../ui/paging";

vi.mock("mermaid", () => ({
  default: {
    initialize: () => {},
    render: async (_: string, source: string) => ({
      svg: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><text>${source}</text></svg>`,
    }),
  },
}));

const page = (html: string): Document => new DOMParser().parseFromString(shell(html), "text/html");

const reaching = (opens = true) => {
  const heard = { diagrams: [] as [number, number, string][], pictures: [] as number[], away: 0 };
  const reach: Reaching = {
    body: "",
    onDoc: () => {},
    onPicture: (_, nth) => heard.pictures.push(nth),
    onAway: () => {
      heard.away += 1;
    },
    onDiagram: opens ? (at, nth, source) => heard.diagrams.push([at, nth, source]) : undefined,
  };
  return { heard, reach };
};

// One diagram at the top and one inside a quote, whose own place differs from the quote's.
const BODY =
  '<figure class="diagram" data-at="0-30"><pre><code class="language-mermaid">graph TD</code></pre></figure>' +
  '<blockquote data-at="32-80"><figure class="diagram" data-at="34-78"><pre><code class="language-mermaid">mindmap</code></pre></figure></blockquote>';

const drawnPage = () =>
  page(
    '<figure class="diagram drawn" data-at="0-30"><div class="diagram-sheet"><svg viewBox="0 0 10 10"><text>Cifrado</text></svg></div></figure>' +
      '<p data-at="32-40"><a class="card" data-doc="ab12-0001">Otro</a></p>',
  );

const press = (on: Element | null | undefined, detail: number, kind = "click") =>
  on?.dispatchEvent(new MouseEvent(kind, { bubbles: true, cancelable: true, button: 0, detail }));

// A double press on a label selects its word before the second click arrives.
const selectWord = (own: Document, word: Element | null) => {
  const range = own.createRange();
  if (word) range.selectNodeContents(word);
  own.getSelection()?.removeAllRanges();
  own.getSelection()?.addRange(range);
};

describe("opening a diagram of the page in its own window", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("gives each diagram a way to its window, only where the page can be written", () => {
    const own = page(BODY);
    worn(own, false, () => true, reaching().reach);
    const buttons = own.querySelectorAll(
      'figure.diagram > .diagram-bar > button[data-asked="diagram"]',
    );
    expect(Array.from(buttons, (one) => one.getAttribute("aria-label"))).toEqual([
      t("diagramOpen"),
      t("diagramOpen"),
    ]);
    expect(own.body.textContent).not.toContain(t("diagramOpen"));

    const stranger = page(BODY);
    worn(stranger, true, () => true, reaching().reach);
    const reading = page(BODY);
    worn(reading, false, () => true, reaching(false).reach);
    expect(stranger.querySelector(".diagram-bar")).toBeNull();
    expect(reading.querySelector(".diagram-bar")).toBeNull();
  });

  it("names the diagram by its own place, even inside a quote, in the bytes the window reads", () => {
    const own = page(BODY);
    diagramBars(own);
    diagramBars(own);
    const { heard, reach } = reaching();
    wired(own, () => reach);

    for (const button of own.querySelectorAll(".diagram-bar button")) press(button, 1);

    expect(own.querySelectorAll(".diagram-bar")).toHaveLength(2);
    expect(heard.diagrams).toEqual([
      [0, 0, "graph TD"],
      [34, 1, "mindmap"],
    ]);
    expect(went(own.querySelectorAll(".diagram-bar button")[1], false)).toEqual({
      kind: "diagram",
      at: [34, 78],
    });
  });

  it("keeps the way to the window when Mermaid draws the figure, in either theme", async () => {
    const own = page(BODY);
    diagramBars(own);

    await diagrams(own, false, () => true);
    await diagrams(own, true, () => true);

    const figures = own.querySelectorAll("figure.diagram.drawn");
    expect(figures).toHaveLength(2);
    for (const figure of figures) {
      expect(figure.querySelector(":scope > .diagram-sheet svg")).not.toBeNull();
      expect(figure.querySelector(":scope > .diagram-bar button")).not.toBeNull();
    }
    const { heard, reach } = reaching();
    answered(own, { kind: "diagram", at: [34, 78] }, reach);
    answered(own, { kind: "diagram", at: [1, 2] }, reach);
    expect(heard.diagrams).toEqual([[34, 1, "mindmap"]]);
  });

  it("opens the window on a double press on a label, word selected or not, and never the picture", () => {
    vi.useFakeTimers();
    const own = drawnPage();
    const { heard, reach } = reaching();
    wired(own, () => reach);
    const label = own.querySelector("text");

    press(label, 1);
    selectWord(own, label);
    press(label, 2);
    press(label, 2, "dblclick");
    vi.advanceTimersByTime(DOUBLE_MS);

    expect(heard.diagrams).toEqual([[0, 0, ""]]);
    expect(heard.away).toBe(0);
    expect(heard.pictures).toEqual([]);
    expect(own.getSelection()?.isCollapsed ?? true).toBe(true);
  });

  it("looks at the picture after a single press, unless something else was pressed meanwhile", () => {
    vi.useFakeTimers();
    const own = drawnPage();
    const { heard, reach } = reaching();
    wired(own, () => reach);
    const svg = own.querySelector("svg");

    press(svg, 1);
    expect(heard.pictures).toEqual([]);
    vi.advanceTimersByTime(DOUBLE_MS);
    expect(heard.pictures).toEqual([0]);

    press(svg, 1);
    press(own.querySelector("a.card"), 1);
    vi.advanceTimersByTime(DOUBLE_MS);
    expect(heard.pictures).toEqual([0]);

    press(svg, 1);
    own.querySelector("figure")?.remove();
    vi.advanceTimersByTime(DOUBLE_MS);
    expect(heard.pictures).toEqual([0]);
  });

  it("looks at the picture at once where the diagram cannot be opened", () => {
    const own = drawnPage();
    const { heard, reach } = reaching(false);
    wired(own, () => reach);

    press(own.querySelector("svg"), 1);

    expect(heard.pictures).toEqual([0]);
    answered(own, { kind: "diagram", at: [0, 30] }, reach);
    expect(heard.diagrams).toEqual([]);
  });
});
