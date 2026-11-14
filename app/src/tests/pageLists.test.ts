import { describe, expect, it } from "vitest";
import { joinedAt, nested, outdented } from "../ui/pageLists";

const applied = (text: string, moved: ReturnType<typeof nested>) =>
  moved ? text.slice(0, moved.edit.from) + moved.edit.insert + text.slice(moved.edit.to) : text;

describe("list items moved in and out by their indent", () => {
  it.each([
    ["an item under a bullet", "- a\n- b\n", 6, false, "- a\n  - b\n"],
    ["an item under a number", "1. a\n2. b\n", 8, false, "1. a\n   2. b\n"],
    ["an item back to its parent", "- a\n  - b\n", 8, true, "- a\n- b\n"],
    ["an item inside a quote", "> - a\n> - b\n", 10, false, "> - a\n>   - b\n"],
    ["an item back out inside a quote", "> - a\n>   - b\n", 12, true, "> - a\n> - b\n"],
    ["a first item, which has nothing above", "- a\n", 2, false, "- a\n"],
    ["a line that is no item", "uno\n", 2, false, "uno\n"],
  ])("moves %s", (_, text, at, out, expected) => {
    expect(applied(text, nested(text, at, out))).toBe(expected);
  });

  it("reads a line of many quote marks and no item at once, not in a time that doubles with each mark", () => {
    const line = `>${" >".repeat(28)} x`;
    const started = performance.now();
    expect(nested(line, line.length, false)).toBeNull();
    expect(performance.now() - started).toBeLessThan(1000);
  });

  it("brings an empty nested item up a level, its caret at the line's end", () => {
    const moved = outdented("- a\n  - ", 8);
    expect(applied("- a\n  - ", moved)).toBe("- a\n- ");
    expect(moved?.caret).toBe(6);
  });

  it("joins an item asked past a list's blank line to the list, Windows line ends too", () => {
    expect(joinedAt("- a\n\n## B\n", 5, "- [ ] ")).toBe(4);
    expect(joinedAt("- a\r\n\r\n## B\r\n", 7, "- [ ] ")).toBe(5);
    expect(joinedAt("Uno\n\n## B\n", 5, "- [ ] ")).toBe(5);
    expect(joinedAt("- a\n\n## B\n", 5, "")).toBe(5);
  });
});
