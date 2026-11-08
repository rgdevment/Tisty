import { describe, expect, it } from "vitest";
import { saidPlainly } from "../refusal";

describe("saidPlainly", () => {
  it("fills the name into the sentence", () => {
    expect(saidPlainly({ code: "noSuchList", name: "work" })).toBe("No list matches work");
  });

  it("says the room Folio leaves a key as the round figure it is", () => {
    expect(saidPlainly({ code: "folioRoomFull", name: "25000000" })).toContain("the 25 MB your");
    expect(saidPlainly({ code: "folioOwnerRoomFull", name: "30000000" })).toContain(
      "the 30 MB their",
    );
    expect(saidPlainly({ code: "folioRoomFull", name: "2500000" })).toContain("the 2.5 MB your");
  });

  it("says what the key holds, what fits and what this one weighs", () => {
    expect(saidPlainly({ code: "folioRoomFullOf", name: "23400000 4000000 25000000" })).toBe(
      "Your shared documents take 23.4 MB of 25 MB and this one weighs 4 MB. Stop sharing one or wait for one to expire.",
    );
    expect(saidPlainly({ code: "folioOwnerRoomFullOf", name: "23400000 4000000 25000000" })).toBe(
      "The owner's documents take 23.4 MB of 25 MB and this version weighs 4 MB. More can be uploaded once one of them expires.",
    );
  });

  it("says how long to wait, and at what time the network may upload again", () => {
    const inFive = String(Date.now() + 5 * 60_000);
    expect(saidPlainly({ code: "folioWait", name: inFive })).toBe(
      "Wait 5 min to upload another version.",
    );
    const at = new Date(2026, 5, 10, 14, 30).getTime();
    expect(saidPlainly({ code: "folioQuota", name: String(at) })).toContain(
      new Date(at).toLocaleTimeString("en", { hour: "2-digit", minute: "2-digit" }),
    );
  });

  it("uses the bare sentence when there is no name to fill", () => {
    expect(saidPlainly({ code: "untitled" })).toBe("A title is required");
  });

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
