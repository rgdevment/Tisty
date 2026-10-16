import { describe, expect, it } from "vitest";
import { keptAll } from "../kept";

describe("files kept together", () => {
  it("lets in the ones that could be kept and tells the first that could not", async () => {
    const told: string[] = [];
    const kept = await keptAll(
      [
        Promise.resolve("![a](a.png)"),
        Promise.reject({ code: "cannotRead", name: "b.png" }),
        Promise.resolve("![c](c.png)"),
      ],
      (said) => told.push(said),
    );
    expect(kept).toBe("![a](a.png)\n\n![c](c.png)");
    expect(told).toHaveLength(1);
  });

  it("tells nothing when every file was kept", async () => {
    const told: string[] = [];
    expect(await keptAll([Promise.resolve("x")], (said) => told.push(said))).toBe("x");
    expect(told).toEqual([]);
  });
});
