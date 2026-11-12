import { beforeEach, describe, expect, it } from "vitest";
import { WINKS_ONCE_IN, winksNow } from "../wink";

beforeEach(() => localStorage.clear());

describe("the hidden wink", () => {
  it("comes exactly once in every run of a hundred openings", () => {
    for (let run = 0; run < 5; run += 1) {
      const winks = Array.from({ length: WINKS_ONCE_IN }, () => winksNow()).filter(Boolean);
      expect(winks).toHaveLength(1);
    }
  });

  it("starts a fresh run when what was kept makes no sense", () => {
    localStorage.setItem("tisty.wink", JSON.stringify({ seen: 500, at: -3 }));

    const winks = Array.from({ length: WINKS_ONCE_IN }, () => winksNow()).filter(Boolean);

    expect(winks).toHaveLength(1);
  });
});
