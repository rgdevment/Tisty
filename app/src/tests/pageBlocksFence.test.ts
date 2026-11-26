import { describe, expect, it } from "vitest";
import { unfenced } from "../ui/pageBlocks";

describe("the lines of a whole fenced block", () => {
  it.each([
    ["```js\nlet a = 1;\n\nb();\n```", "let a = 1;\n\nb();"],
    ["~~~\na\n~~~", "a"],
    ["````md\n```\nx\n```\n````", "```\nx\n```"],
    ["  ```\n    a\n  ```", "    a"],
    ["```\na\n```\n", "a"],
  ])("takes the lines out of «%j» as «%j»", (said, expected) => {
    expect(unfenced(said)).toBe(expected);
  });

  it.each([
    "a\nb",
    "```\na\n```\nmás texto",
    "antes\n```\na\n```",
    "```\nsin cierre",
    "~~~\na\n```",
  ])("leaves «%j» as it is", (said) => {
    expect(unfenced(said)).toBe(said);
  });
});
