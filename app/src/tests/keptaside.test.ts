import { describe, expect, it, vi } from "vitest";
import { asideWords, letAsideGo, setAside } from "../keptAside";
import { cut, kept, leftUnsaved, unansweredWords } from "../saving";

describe("words a save refused for good", () => {
  it("are kept aside while the document stays, never sent again, and closing does not ask", async () => {
    for (const code of ["documentTooLong", "documentLocked"]) {
      let sent = 0;
      await kept("aparte", `leche ${code}`, () => {
        sent += 1;
        return Promise.reject({ code });
      }).catch(() => {});

      expect(asideWords("aparte"), code).toBe(`leche ${code}`);
      expect(unansweredWords("aparte"), code).toBeUndefined();
      expect(await leftUnsaved(), code).toBe(false);
      expect(sent, code).toBe(1);
    }
    letAsideGo("aparte");
  });

  it("are not kept for a document that is gone", async () => {
    await kept("borrado", "leche", () => Promise.reject({ code: "noSuchDoc" })).catch(() => {});

    expect(asideWords("borrado")).toBeUndefined();
    cut("borrado");
  });

  it("stay on this computer after Tisty closes and opens again, until they are let go", async () => {
    setAside("guardado", "huevos");
    vi.resetModules();
    const again = await import("../keptAside");

    expect(again.asideWords("guardado")).toBe("huevos");
    again.letAsideGo("guardado");
    expect(again.asideWords("guardado")).toBeUndefined();
    letAsideGo("guardado");
  });
});
