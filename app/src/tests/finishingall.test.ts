import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { finishAll, finishWhole } from "../guide/finishing";

const finished = vi.hoisted(() => vi.fn());
vi.mock("tisty-editor/read", () => ({ finished }));

type Answer = { source: string; drawn: boolean; wrong: string[] };

const look = (theme: string, inside: string, box = "0 0 9 9") =>
  `<svg data-theme="${theme}" viewBox="${box}">${inside}</svg>`;
const inFigure = (...looks: string[]) => `<figure class="diagram">${looks.join("")}</figure>`;
const WORDS = '<text x="1" y="1">a</text>';

const answered = async (html: string, notDrawn = 0, sources = ["graph"]): Promise<Answer[]> => {
  finished.mockResolvedValue({ html, notDrawn });
  const asked = vi.fn(async (url: string, _?: RequestInit) => ({
    json: async () => (url === "/sources" ? sources : null),
  }));
  vi.stubGlobal("fetch", asked);
  await finishAll();
  const [where, sent] = asked.mock.calls[1];
  expect(where).toBe("/drawn");
  expect(sent?.method).toBe("POST");
  return JSON.parse(String(sent?.body));
};

describe("what Chrome answers of each diagram", () => {
  beforeEach(() => finished.mockReset());
  afterEach(() => vi.unstubAllGlobals());

  it("hands each source over as a diagram not drawn yet, with its words as words", async () => {
    await answered(inFigure(look("light", ""), look("dark", "")), 0, ["a <b> & c"]);
    expect(finished).toHaveBeenCalledWith(
      '<figure class="diagram"><pre><code class="language-mermaid">a &lt;b&gt; &amp; c</code></pre></figure>',
    );
  });

  it("takes two whole looks as drawn, and nothing is wrong with them", async () => {
    const [one] = await answered(inFigure(look("light", WORDS), look("dark", WORDS)));
    expect(one).toEqual({ source: "graph", drawn: true, wrong: [] });
  });

  it("lets the boxes differ, since gitGraph makes up the ids it measures", async () => {
    const [one] = await answered(
      inFigure(look("light", WORDS, "0 0 9 9"), look("dark", WORDS, "0 0 8 9")),
    );
    expect(one.drawn).toBe(true);
  });

  it("does not take an icon for the other look", async () => {
    const icon = '<svg viewBox="0 0 8 8"></svg>';
    const [one] = await answered(inFigure(look("light", icon + icon), look("dark", "")));
    expect(one.drawn).toBe(false);
  });

  it("does not take a look that lost words", async () => {
    const [one] = await answered(inFigure(look("light", WORDS + WORDS), look("dark", WORDS)));
    expect(one.drawn).toBe(false);
  });

  it("wants each look, once and in its place", async () => {
    for (const html of [
      inFigure(look("light", "")),
      inFigure(look("dark", ""), look("light", "")),
      inFigure(look("light", ""), look("light", "")),
      inFigure(look("dark", ""), look("dark", "")),
      inFigure(look("light", ""), look("dark", ""), look("dark", "")),
      "<p>a</p>",
    ]) {
      const [one] = await answered(html);
      expect(one.drawn).toBe(false);
    }
  });

  it("says when one could not be drawn, whatever came back", async () => {
    const [one] = await answered(inFigure(look("light", ""), look("dark", "")), 1);
    expect(one.drawn).toBe(false);
  });

  it("answers what the contract says of it", async () => {
    const [one] = await answered("<script>a</script>");
    expect(one.wrong).toContain("<script> is in no contract");
  });

  it("hands a whole document back as it came out, under its name", async () => {
    finished.mockResolvedValue({ html: "<p>drawn</p>", notDrawn: 1 });
    const asked = vi.fn(async (_: string, __?: RequestInit) => ({
      json: async () => [{ name: "paseos", html: "<p>a</p>" }],
    }));
    vi.stubGlobal("fetch", asked);
    await finishWhole();
    expect(finished).toHaveBeenCalledWith("<p>a</p>");
    expect(asked.mock.calls[1][0]).toBe("/drawn");
    expect(JSON.parse(String(asked.mock.calls[1][1]?.body))).toEqual([
      { name: "paseos", html: "<p>drawn</p>", notDrawn: 1 },
    ]);
  });

  it("answers for every source", async () => {
    const said = await answered(inFigure(look("light", ""), look("dark", "")), 0, ["a", "b"]);
    expect(said.map((one) => one.source)).toEqual(["a", "b"]);
  });
});
