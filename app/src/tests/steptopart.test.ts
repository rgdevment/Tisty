import { stepLine, steppedTitle } from "tisty-editor/read";
import { describe, expect, it, vi } from "vitest";

describe("a step offered as a part", () => {
  it("is read back the way the core reads its words", () => {
    expect(steppedTitle("- [ ] Pack  the   books")).toBe("Pack the books");
    expect(steppedTitle("- [ ] Pack the books\\\\")).toBe("Pack the books");
    expect(steppedTitle("- [x] Pack the books")).toBeNull();
    expect(steppedTitle("- [ ] ")).toBeNull();
  });

  it("is read at once however many blanks pad it before a break it cannot cross", () => {
    const began = vi.getRealSystemTime();
    expect(steppedTitle(`- [ ] ${" ".repeat(100_000)}x\u2028y`)).toBeNull();
    expect(vi.getRealSystemTime() - began).toBeLessThan(1_000);
  });

  it("is only a whole line of its own, whatever ends it", () => {
    const body = "## Steps\r\n- [ ] Pack\r\n- [ ] Move\r\n";
    const from = body.indexOf("- [ ] Pack");
    expect(stepLine(body, [from, from + "- [ ] Pack\r\n".length])).toBe("- [ ] Pack");
    expect(stepLine(body, [from + 2, from + 10])).toBeNull();
    const runs = "- [ ] Pack\\\nthe books\n";
    expect(stepLine(runs, [0, runs.length])).toBeNull();
  });
});
