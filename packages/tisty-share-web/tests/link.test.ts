import { describe, expect, it } from "vitest";
import { linkOf, lost } from "../src/link";

describe("a link", () => {
  it("names the document by its path and keeps the key the # carries", () => {
    expect(linkOf("/k7Qm2xPa", "#qR12Ff")).toEqual({ id: "k7Qm2xPa", key: "qR12Ff" });
    expect(linkOf("/k7Qm2xPa/", "#qR12Ff")).toEqual({ id: "k7Qm2xPa", key: "qR12Ff" });
  });

  it("drops only the # that opens it, never one inside the key", () => {
    expect(linkOf("/k7Qm2xPa", "ab#cd")?.key).toBe("ab#cd");
  });

  it("without its # still names the document, with no key to open it", () => {
    expect(linkOf("/k7Qm2xPa", "")).toEqual({ id: "k7Qm2xPa", key: "" });
  });

  it("is no link when the path names no document", () => {
    for (const path of [
      "/",
      "/k7Qm2xP",
      "/k7Qm2xPa9",
      "/api/v1/limits",
      "/k7Qm-xPa",
      "/a/k7Qm2xPa",
    ])
      expect(linkOf(path, "#qR12Ff"), path).toBeNull();
  });

  it("that names nothing the server still has is lost, as is a path that is no link", () => {
    for (const word of ["nolink", "missing", "gone"]) expect(lost(word), word).toBe(true);
    for (const word of ["paused", "blocked", "keyless", "locked", "down", "offline", "slow"])
      expect(lost(word), word).toBe(false);
  });
});
