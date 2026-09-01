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

describe("a table reaches the book leaning the way it was written", () => {
  const doc = (leans: (string | null)[]) => ({
    type: "doc",
    content: [
      {
        type: "table",
        content: [
          {
            type: "tableRow",
            content: leans.map((textAlign) => ({
              type: "tableHeader",
              attrs: { textAlign },
              content: [{ type: "paragraph", content: [{ type: "text", text: "h" }] }],
            })),
          },
        ],
      },
    ],
  });

  it("carries what each column leans", () => {
    const one = shapesOf(doc(["left", null, "right"]) as never)[0];
    expect(one.kind).toBe("table");
    if (one.kind !== "table") return;
    expect(one.leans).toEqual(["left", null, "right"]);
  });
});

describe("a picture inside a callout reaches the book like any other", () => {
  const doc = {
    type: "doc",
    content: [
      {
        type: "callout",
        attrs: { kind: "warning" },
        content: [{ type: "image", attrs: { src: "attachments/ab/una.png", alt: "una" } }],
      },
    ],
  };

  it("is carried in, not left as a path react-pdf cannot read", async () => {
    const { fetched } = await import("../ui/shaping");
    const bytes = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, ...new Array(40).fill(0)];
    const out = await fetched(shapesOf(doc as never), async () => bytes);
    const one = out[0];
    expect(one.kind).toBe("said");
    if (one.kind !== "said") return;
    const kid = one.inner[0];
    expect(kid.kind).toBe("image");
    if (kid.kind !== "image") return;
    expect(kid.src.startsWith("attachments/")).toBe(false);
  });
});
