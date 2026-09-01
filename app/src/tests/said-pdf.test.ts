import { describe, expect, it } from "vitest";
import { shapesOf } from "../ui/shaping";

describe("a callout reaches the book with its frame", () => {
  const doc = {
    type: "doc",
    content: [
      {
        type: "callout",
        attrs: { kind: "warning" },
        content: [
          { type: "paragraph", content: [{ type: "text", text: "Cuidado." }] },
          {
            type: "bulletList",
            content: [
              {
                type: "listItem",
                content: [{ type: "paragraph", content: [{ type: "text", text: "uno" }] }],
              },
            ],
          },
        ],
      },
    ],
  };

  it("keeps the kind and does not flatten what it holds", () => {
    const shapes = shapesOf(doc as never);
    expect(shapes).toHaveLength(1);
    const one = shapes[0];
    expect(one.kind).toBe("said");
    if (one.kind !== "said") return;
    expect(one.said).toBe("warning");
    expect(one.inner.map((kid) => kid.kind)).toEqual(["para", "bullet"]);
  });
});
