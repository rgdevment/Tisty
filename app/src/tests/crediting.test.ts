import { describe, expect, it } from "vitest";
import { creditRead } from "../crediting";

describe("reading how a signature was chosen", () => {
  it("falls back to the shipped choice when nothing usable was kept", () => {
    const shipped = { rows: "agent", inside: "named" };
    expect(creditRead({})).toEqual(shipped);
    expect(creditRead({ rows: "x", inside: 3 })).toEqual(shipped);
    expect(creditRead({ rows: null, inside: [], mark: "yes" })).toEqual(shipped);
  });

  it("carries the old switches over: on named it, off said an agent", () => {
    expect(creditRead({ rows: false, inside: true, mark: false })).toEqual({
      rows: "agent",
      inside: "named",
    });
    expect(creditRead({ rows: true, inside: false, mark: false })).toEqual({
      rows: "named",
      inside: "agent",
    });
  });

  it("carries the old diamond over to both places, whatever the other two said", () => {
    expect(creditRead({ rows: true, inside: true, mark: true })).toEqual({
      rows: "mark",
      inside: "mark",
    });
    expect(creditRead({ mark: true })).toEqual({ rows: "mark", inside: "mark" });
  });

  it("keeps a new choice as it was", () => {
    expect(creditRead({ rows: "mark", inside: "agent" })).toEqual({
      rows: "mark",
      inside: "agent",
    });
    expect(creditRead({ rows: "named", inside: "named" })).toEqual({
      rows: "named",
      inside: "named",
    });
  });

  it("lets a new choice win over an old diamond, place by place", () => {
    expect(creditRead({ rows: "named", inside: true, mark: true })).toEqual({
      rows: "named",
      inside: "mark",
    });
    expect(creditRead({ rows: "agent", inside: "x", mark: true })).toEqual({
      rows: "agent",
      inside: "mark",
    });
  });

  it("migrates one place and ships the other when only one was kept", () => {
    expect(creditRead({ rows: true })).toEqual({ rows: "named", inside: "named" });
    expect(creditRead({ inside: false })).toEqual({ rows: "agent", inside: "agent" });
  });
});
