import { describe, expect, it } from "vitest";
import { saidPlainly } from "../refusal";

describe("saidPlainly", () => {
  it("fills the name into the sentence", () => {
    expect(saidPlainly({ code: "noSuchList", name: "work" })).toBe("No list matches work");
  });

  it("uses the bare sentence when there is no name to fill", () => {
    expect(saidPlainly({ code: "untitled" })).toBe("A title is required");
  });

  it("shows an unknown code rather than swallowing what went wrong", () => {
    expect(saidPlainly({ code: "somethingNewer" })).toBe("somethingNewer");
    expect(saidPlainly({ code: "somethingNewer", name: "the detail" })).toBe("the detail");
  });

  it("survives being handed something that is not a refusal at all", () => {
    expect(saidPlainly("the window is on fire")).toBe("the window is on fire");
    expect(saidPlainly(undefined)).toBe("undefined");
    expect(saidPlainly({ code: 7 })).toBe("[object Object]");
  });
});
