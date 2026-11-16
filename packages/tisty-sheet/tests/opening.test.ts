import { describe, expect, it } from "vitest";
import { source } from "./source";

const css = () => source("page.css").replace(/\s+/g, " ");

const ruleFor = (text: string, selector: string): string => {
  const at = text.indexOf(selector);
  expect(at).toBeGreaterThan(-1);
  return text.slice(at, text.indexOf("}", at));
};

const topLevel = (selector: string, marks: string): number[] => {
  const found: number[] = [];
  let depth = 0;
  for (let at = 0; at < selector.length; at++) {
    const one = selector[at];
    if (one === "(" || one === "[") depth++;
    else if (one === ")" || one === "]") depth--;
    else if (depth === 0 && marks.includes(one)) found.push(at);
  }
  return found;
};

// Every element a sibling combinator's compound may name is restyled whenever a block comes or goes.
const pastSibling = (selector: string): string | null => {
  const last = topLevel(selector, "+~").pop();
  if (last === undefined) return null;
  const tail = selector.slice(last + 1).trim();
  return tail.slice(0, topLevel(tail, " >").shift() ?? tail.length);
};

const byKind = (compound: string): boolean => {
  let bare = compound;
  while (/\([^()]*\)/.test(bare)) bare = bare.replace(/\([^()]*\)/g, "");
  return !/[.#[]/.test(bare) || /^:not\(/.test(compound);
};

const OPENS = ".page:where(:not(.as-written)) > :is(h1, h2, h3, p):first-child";
const ALONE = ".page:where(:not(.as-written)) > :is(h1, h2, h3, p):first-child:only-child";

describe("the first line of a document", () => {
  it("is set as a title even when it was never marked as one", () => {
    const rule = ruleFor(css(), OPENS);

    expect(rule).toMatch(/font-size:\s*1\.65em/);
    expect(rule).toMatch(/font-weight:\s*640/);
  });

  it("is the line after the properties when a note opens with them", () => {
    const rule = ruleFor(css(), OPENS);

    expect(rule).toContain(".page:where(:not(.as-written)) > .title");
  });

  it("reaches no block by its kind through a sibling, which would restyle them all as blocks come and go", () => {
    const sheet = ["page.css", "tables.css", "diary.css"]
      .map((name) => source(name))
      .join("")
      .replace(/\/\*[\s\S]*?\*\//g, "");
    const reached = sheet
      .split("}")
      .map((rule) => rule.split("{"))
      .map((parts) => (parts.length > 1 ? parts[parts.length - 2] : ""))
      .flatMap((list) => list.replace(/\s+/g, " ").split(/,(?![^(]*\))/))
      .map((one) => one.trim())
      .filter((one) => /\.page\b/.test(one))
      .filter((one) => byKind(pastSibling(one) ?? ".none"));

    expect(reached).toEqual([]);
  });

  it("tells a sibling rule that reaches blocks by their kind from one narrowed to a class", () => {
    const reaching = [
      ".page > .front:first-child + :is(h1, h2, h3, p)",
      ".page > .front:first-child + :is(h1, h2, h3)",
      ".page > * + p",
      ".page > hr ~ *",
      ".page > .front + p:first-line",
      ".page > .front + :not(.x)",
      ".page > .front + :where(p, ul) span",
    ];
    const narrowed = [
      ".page > .front + p.title",
      ".page > .step-hint + p[data-ghost]",
      ".page > li:nth-child(2n+1)",
      ".page > [class~=x]",
    ];
    expect(reaching.filter((one) => !byKind(pastSibling(one) ?? ".none"))).toEqual([]);
    expect(narrowed.filter((one) => byKind(pastSibling(one) ?? ".none"))).toEqual([]);
  });

  it("is parted from the body by a hairline", () => {
    const rule = ruleFor(css(), OPENS);

    expect(rule).toMatch(/border-bottom:\s*1px solid var\(--tisty-hair\)/);
    expect(rule).toMatch(/padding-bottom/);
  });

  it("carries no parting line when there is nothing yet to part from", () => {
    const rule = ruleFor(css(), `${ALONE} {`);

    expect(rule).toMatch(/border-bottom:\s*0/);
    expect(rule).toMatch(/padding-bottom:\s*0/);
  });
});
