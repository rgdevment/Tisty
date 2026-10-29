import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = () => readFileSync("../packages/tisty-reader/src/page.css", "utf8");

const ruleFor = (text: string, selector: string): string => {
  const at = text.indexOf(selector);
  expect(at).toBeGreaterThan(-1);
  return text.slice(at, text.indexOf("}", at));
};

const OPENS = ".page > :is(h1, h2, h3, p):first-child";
const ALONE = ".page > :is(h1, h2, h3, p):first-child:only-child";

describe("the first line of a document", () => {
  it("is set as a title even when it was never marked as one", () => {
    const rule = ruleFor(css(), OPENS);

    expect(rule).toMatch(/font-size:\s*1\.65em/);
    expect(rule).toMatch(/font-weight:\s*640/);
  });

  it("is the line after the properties when a note opens with them", () => {
    const rule = ruleFor(css(), OPENS);

    expect(rule).toContain(".page > :is(.front, .fronted):first-child + :is(h1, h2, h3, p)");
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
