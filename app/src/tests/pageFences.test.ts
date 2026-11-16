import { describe, expect, it } from "vitest";
import { fenceJustOpened, fenceNamed, fenceOpened } from "../ui/pageFences";

const applied = (text: string, moved: ReturnType<typeof fenceOpened>) =>
  moved ? text.slice(0, moved.edit.from) + moved.edit.insert + text.slice(moved.edit.to) : text;

describe("a code block typed with its backticks", () => {
  it.each([
    ["on a line of its own", "Uno\n\n``\n\nFin\n", 7, "`", "Uno\n\n```\n\n```\n\nFin\n", 9],
    ["indented", "-  a\n\n  ``", 10, "`", "-  a\n\n  ```\n  \n  ```", 14],
    ["with tildes", "~~", 2, "~", "~~~\n\n~~~", 4],
  ])("closes itself when opened %s", (_, text, at, typed, expected, caret) => {
    const moved = fenceOpened(text, at, typed);
    expect(applied(text, moved)).toBe(expected);
    expect(moved?.caret).toBe(caret);
  });

  it.each([
    ["inside words", "dos ``", 6, "`"],
    ["with words after the caret", "``tres", 2, "`"],
    ["with a fourth backtick", "```", 3, "`"],
    ["with another mark", "``", 2, "~"],
  ])("leaves the backtick alone %s", (_, text, at, typed) => {
    expect(fenceOpened(text, at, typed)).toBeNull();
  });

  it("takes a language typed on the first line up to the fence, and the caret to an empty line", () => {
    const text = "```\njs\n```\n\nFin\n";
    const moved = fenceNamed(text, 6);
    expect(applied(text, moved)).toBe("```js\n\n```\n\nFin\n");
    expect(moved?.caret).toBe(6);
  });

  it("leaves a first line that is code, or a fence that already names its language", () => {
    expect(fenceNamed("```\nlet a\n```", 9)).toBeNull();
    expect(fenceNamed("```rust\njs\n```", 10)).toBeNull();
    expect(fenceNamed("Uno\njs", 6)).toBeNull();
  });

  it("works on Windows line ends", () => {
    const text = "```\r\nmermaid\r\n```";
    const moved = fenceNamed(text, 12);
    expect(applied(text, moved)).toBe("```mermaid\n\r\n```");
  });
});

describe("Enter right after the backticks", () => {
  it("finds the caret on the block's first line already, so it adds no blank line", () => {
    expect(fenceJustOpened("```\n\n```", 4)).toBe(true);
    expect(fenceJustOpened("```js\n\n```", 6)).toBe(true);
  });

  it("still breaks a line in a block with words in it", () => {
    expect(fenceJustOpened("```\nlet a\n```", 9)).toBe(false);
    expect(fenceJustOpened("```\n\nlet a\n```", 4)).toBe(false);
  });
});
