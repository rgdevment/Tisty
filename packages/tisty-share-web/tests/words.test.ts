import { describe, expect, it } from "vitest";
import { spanish, wordsFor } from "../src/words";

describe("the page's words", () => {
  it("are Spanish when the reader asks for any Spanish, and English otherwise", () => {
    expect(spanish(["es-CL", "en"])).toBe(true);
    expect(spanish(["en-US", "ES"])).toBe(true);
    expect(spanish(["en-US", "pt-BR"])).toBe(false);
    expect(spanish([])).toBe(false);
  });

  it("turn each refusal the server or the envelope gives into a sentence", () => {
    const say = wordsFor(true);
    for (const word of ["missing", "gone", "paused", "blocked", "locked", "keyless", "offline"])
      expect(say(word), word).not.toBe(say("failed"));
  });

  it("fall back to one plain sentence for a word they do not know", () => {
    expect(wordsFor(false)("store")).toBe(wordsFor(false)("failed"));
  });

  it("name the callouts as Tisty does", () => {
    expect(wordsFor(true)("saidwarning")).toBe("Aviso");
    expect(wordsFor(false)("saidwarning")).toBe("Warning");
  });
});
