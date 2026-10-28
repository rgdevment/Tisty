import mermaid from "mermaid";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { adopt } from "../locales";
import { readClasses } from "../ui/drawing/classes";
import { readFlow } from "../ui/drawing/flow";
import { kindOf } from "../ui/drawing/framing";
import { readMind } from "../ui/drawing/mindmap";
import { readSchema } from "../ui/drawing/schema";
import {
  fenced,
  fencesOf,
  firstWords,
  freshFence,
  STARTS,
  type Start,
  startOf,
} from "../ui/drawing/starting";

// `⏎` is a line break, so a whole document fits on its line.
const unfolded = (said: string) => said.replace(/⏎/g, "\n");
const lines = (table: string) => table.split("\n").filter((line) => line.trim());

const valid = async (source: string) =>
  Boolean(await mermaid.parse(source, { suppressErrors: true }));

const READERS: Partial<Record<Start, (source: string) => unknown>> = {
  mindmap: readMind,
  erDiagram: readSchema,
  flowchart: readFlow,
  classDiagram: readClasses,
};

beforeAll(() => {
  mermaid.initialize({ startOnLoad: false });
});

afterAll(() => adopt("en"));

describe("the example a new diagram begins with", () => {
  it("is valid Mermaid of its kind in every language, and the window can edit it", async () => {
    for (const language of ["en", "es"]) {
      adopt(language);
      for (const kind of STARTS) {
        const source = startOf(kind);
        const said = `${language} ${kind}`;
        expect(source.endsWith("\n"), said).toBe(true);
        expect(await valid(source), said).toBe(true);
        expect(kindOf(source), said).toBe(kind === "other" ? "other" : kind);
        if (READERS[kind]) expect(READERS[kind]?.(source), said).not.toBeNull();
      }
    }
  });
});

// One per line: a document → where each diagram block starts, in bytes, and what it says.
// `diagrams_test.rs` checks the core against the same table.
const FENCES = String.raw`
# T⏎⏎${"```"}mermaid⏎a⏎${"```"} → 5:a⏎
# Ñu⏎${"```"}mmd⏎b⏎c⏎${"```"}⏎${"```"}mermaid⏎d⏎${"```"} → 6:b⏎c⏎ | 21:d⏎
~~~~ mermaid⏎x⏎~~~⏎y⏎~~~~ → 0:x⏎~~~⏎y⏎
${"```"}js⏎${"```"}mermaid⏎${"```"} →
${"```"}Mermaid⏎a⏎${"```"} →
   ${"```"}mermaid⏎sin cerrar⏎ → 3:sin cerrar⏎
# Compras⏎⏎leche⏎⏎${"```"}mermaid⏎a⏎${"```"} → 18:a⏎
> ${"```"}mermaid⏎> a⏎> ${"```"} → 2:a⏎
- item⏎⏎  ${"```"}mermaid⏎  b⏎  ${"```"} → 10:b⏎
> ${"```"}mermaid⏎> a⏎> ${"```"}⏎⏎${"```"}mermaid⏎c⏎${"```"} → 2:a⏎ | 24:c⏎
> - ${"```"}mermaid⏎>   d⏎>   ${"```"} → 4:d⏎
    ${"```"}mermaid⏎    x⏎    ${"```"} →
> ${"```"}mermaid⏎> a⏎⏎${"```"}mermaid⏎c⏎${"```"} → 2:a⏎ | 18:c⏎
- ${"```"}mermaid⏎  a⏎⏎${"```"}mermaid⏎c⏎${"```"} → 2:a⏎⏎ | 18:c⏎
`;

describe("finding the diagram blocks of a document", () => {
  it("finds them as the core does: where each starts, in bytes, and its lines", () => {
    for (const case_ of lines(FENCES)) {
      const [text, said = ""] = case_.split(" →");
      const found = fencesOf(unfolded(text)).map((one) => `${one.at}:${one.source}`);
      const expected = said.trim() ? said.split(" | ").map((one) => unfolded(one.trim())) : [];
      expect(found, case_).toEqual(expected);
    }
  });
});

// One per line: the text before → after → which block saying `a` is the new one (-1 for none).
const FRESH = String.raw`
# T → # T⏎${"```"}mermaid⏎a⏎${"```"} → 0
# T⏎${"```"}mermaid⏎a⏎${"```"} → # T⏎${"```"}mermaid⏎a⏎${"```"}⏎${"```"}mermaid⏎a⏎${"```"} → 1
# T⏎${"```"}mermaid⏎b⏎${"```"} → # T⏎${"```"}mermaid⏎a⏎${"```"}⏎${"```"}mermaid⏎b⏎${"```"} → 0
# T → # T⏎${"```"}mermaid⏎b⏎${"```"} → -1
`;

describe("telling which diagram was just written", () => {
  it("takes the block saying it that the text before did not have at its place", () => {
    for (const case_ of lines(FRESH)) {
      const [before, after, nth] = case_.split(" → ").map((one) => unfolded(one.trim()));
      expect(freshFence(before, after, "a\n")?.nth ?? -1, case_).toBe(Number(nth));
    }
  });
});

// One per line: a diagram → the words that tell it apart in a list.
const FIRST = `
mindmap⏎  root((Idea)) → root((Idea))
---⏎title: Pagos⏎---⏎%% nota⏎flowchart TD⏎%% otra⏎⏎  A --> B → A --> B
erDiagram → 
`;

describe("telling diagrams apart in a list", () => {
  it("takes the first line after the header, past front matter and comments", () => {
    for (const case_ of lines(FIRST)) {
      const [source, said = ""] = case_.split(" →");
      expect(firstWords(unfolded(source)), case_).toBe(said.trim());
    }
  });

  it("fences a copy longer than any run of backticks it holds", () => {
    expect(fenced("graph TD\n")).toBe("```mermaid\ngraph TD\n```");
    expect(fenced("a\n  ````x\n")).toBe("`````mermaid\na\n  ````x\n`````");
  });
});
