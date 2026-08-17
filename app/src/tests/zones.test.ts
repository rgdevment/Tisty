import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { wroteAt } from "../format";

const MADRID_NIGHT = "2026-08-10T21:30:00Z";
const NOW = new Date("2026-08-11T12:00:00Z");

describe("a journal entry keeps the zone it was written in", () => {
  beforeEach(() => vi.setSystemTime(NOW));
  afterEach(() => vi.useRealTimers());

  it("says the hour its author saw, not the reader's", () => {
    expect(wroteAt(MADRID_NIGHT, "Europe/Madrid", NOW, "America/Santiago")).toMatch(
      /11:30 ?\s?PM/i,
    );
  });

  it("does not shift the day either", () => {
    const said = wroteAt(MADRID_NIGHT, "Europe/Madrid", NOW, "America/Santiago");

    expect(said).toMatch(/yesterday/i);
  });

  it("shows neither the hour nor the day the reader's zone would have given", () => {
    const said = wroteAt(MADRID_NIGHT, "Europe/Madrid", NOW, "America/Santiago");

    expect(said).not.toMatch(/5:30 ?\s?PM/i);
  });

  it("names the place when it is not the reader's zone", () => {
    expect(wroteAt(MADRID_NIGHT, "Europe/Madrid", NOW, "America/Santiago")).toMatch(/Madrid/);
  });

  it("stays quiet about the place when it is the reader's own", () => {
    expect(wroteAt(MADRID_NIGHT, "Europe/Madrid", NOW, "Europe/Madrid")).not.toMatch(/Madrid/);
  });

  it("does not show an underscore from an IANA name to anyone", () => {
    expect(wroteAt(MADRID_NIGHT, "America/New_York", NOW, "Europe/Madrid")).toMatch(/New York/);
  });

  it("falls back to the reader's zone when none was stored", () => {
    expect(wroteAt(MADRID_NIGHT, undefined, NOW)).not.toBe("");
  });

  it("survives a zone this machine has never heard of", () => {
    expect(() => wroteAt(MADRID_NIGHT, "Mars/Olympus", NOW)).not.toThrow();
    expect(wroteAt(MADRID_NIGHT, "Mars/Olympus", NOW)).not.toBe("");
  });

  it("says nothing for a stamp that is not a moment", () => {
    expect(wroteAt("whenever", "Europe/Madrid", NOW)).toBe("");
  });

  it("dates an entry older than a day, in the author's calendar", () => {
    const said = wroteAt("2026-08-04T21:30:00Z", "Europe/Madrid", NOW, "Europe/Madrid");

    expect(said).toMatch(/Aug/);
    expect(said).toMatch(/11:30 ?\s?PM/i);
  });
});
