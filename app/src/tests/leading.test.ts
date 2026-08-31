import { describe, expect, it } from "vitest";
import { led } from "../leading";

describe("a title that opens with an emoji", () => {
  it("wears it and reads without it", () => {
    expect(led("🧰 Proyecto: RGTools (2026)")).toEqual({
      mark: "🧰",
      rest: "Proyecto: RGTools (2026)",
    });
  });

  it("keeps the whole of an emoji made of several", () => {
    expect(led("👨‍👩‍👧 Familia").mark).toBe("👨‍👩‍👧");
    expect(led("🇨🇱 Chile").mark).toBe("🇨🇱");
    expect(led("👍🏽 Hecho").mark).toBe("👍🏽");
    expect(led("⚠️ Riesgos").mark).toBe("⚠️");
  });

  it("leaves alone a title that is only an emoji", () => {
    expect(led("🧰")).toEqual({ mark: null, rest: "🧰" });
  });

  it("leaves alone a title that opens with anything else", () => {
    for (const one of ["Proyecto", "#4 y luego", "1. Primero", "— guion", "«comillas»"]) {
      expect(led(one)).toEqual({ mark: null, rest: one });
    }
  });

  it("does not take an emoji from the middle", () => {
    expect(led("Riesgos ⚠️ del plan").mark).toBeNull();
  });
});
