import { describe, expect, it } from "vitest";
import { saidPlainly } from "../refusal";

describe("saidPlainly", () => {
  it("fills the name into the sentence", () => {
    expect(saidPlainly({ code: "noSuchList", name: "work" })).toBe("No list matches work");
  });

  it("uses the bare sentence when there is no name to fill", () => {
    expect(saidPlainly({ code: "untitled" })).toBe("A title is required");
  });

  /// The Rust text is English and technical; it stays, because it is what makes
  /// a report worth anything, but never as the whole message.
  it("keeps an unknown code but puts a human sentence in front of it", () => {
    expect(saidPlainly({ code: "somethingNewer" })).toBe("Something went wrong — somethingNewer");
    expect(saidPlainly({ code: "somethingNewer", name: "the detail" })).toBe(
      "Something went wrong — the detail",
    );
  });

  it("survives being handed something that is not a refusal at all", () => {
    expect(saidPlainly("the window is on fire")).toMatch(/the window is on fire$/);
    expect(saidPlainly(undefined)).toMatch(/undefined$/);
    expect(saidPlainly({ code: 7 })).toMatch(/^Something went wrong/);
  });

  /// This was the whole message before: the raw Display of a Rust error, in
  /// English, at the worst possible moment.
  it("never hands over a bare Rust error", () => {
    const said = saidPlainly({ code: "internalNamed", name: "i/o error: access denied" });

    expect(said).not.toBe("i/o error: access denied");
    expect(said).toMatch(/^Something went wrong/);
    expect(said).toMatch(/access denied/);
  });

  it("leaves a refusal people can act on exactly as it was written", () => {
    expect(saidPlainly({ code: "pastDeadline" })).not.toMatch(/Something went wrong/);
  });
});
