import { describe, expect, it } from "vitest";
import {
  accessOf,
  endsOf,
  fileOf,
  type Seal,
  type Signed,
  sealed,
  sealOf,
  tagged,
  tistyOf,
  whenOf,
} from "../src/frame";
import { wordsFor } from "../src/words";

const say = wordsFor(true);
const NOW = 1_793_361_600;
const DAY = 86_400;

const mario: Signed = { is: "holds", fingerprint: "12345 67890 12345 67890", suffix: "r8f1" };
const ana: Signed = { is: "holds", fingerprint: "99999 88888 77777 66666", suffix: "k7qp" };

describe("when it was shared", () => {
  const facts = { shared: NOW - 3 * DAY, written: NOW - 3 * 3_600, gated: false };

  it("says the day it was shared and how long ago it was edited", () => {
    expect(whenOf(facts, 2, NOW, "es", say)).toBe(
      "Compartido el 27 de octubre de 2026 · editado hace 3 horas",
    );
  });

  it("calls no first version edited, since it was written when it was shared", () => {
    expect(whenOf(facts, 1, NOW, "es", say)).toBe("Compartido el 27 de octubre de 2026");
  });

  it("says nothing the server did not tell", () => {
    expect(whenOf({ gated: false }, 3, NOW, "es", say)).toBe("");
  });
});

describe("the seal", () => {
  it("names who wrote it and their fingerprint", () => {
    expect(sealOf({ wrote: mario, sent: mario })).toEqual({
      verdict: "holds",
      who: "r8f1",
      fingerprint: mario.fingerprint,
      sender: undefined,
    });
  });

  it("names who uploaded it only when it was someone else", () => {
    expect(sealOf({ wrote: mario, sent: ana }).sender).toBe("k7qp");
    expect(sealOf({ sent: ana })).toMatchObject({ verdict: "holds", who: "k7qp" });
  });

  it("says when nobody signed it, or the signature does not hold", () => {
    expect(sealOf({})).toEqual({ verdict: "unsigned" });
    expect(sealOf({ wrote: { is: "refused" } })).toMatchObject({
      verdict: "refused",
      who: undefined,
    });
  });

  it("is drawn with its verdict, and without a print it does not have", () => {
    const box = document.createElement("section");
    box.hidden = true;
    box.innerHTML =
      '<span class="seal-who"></span><code class="seal-print"></code><span class="seal-verdict"></span><span class="seal-sender"></span>';
    const drawnAs = (seal: Seal) => {
      sealed(box, seal, say);
      return [...box.children].map((one) => one.textContent);
    };
    expect(drawnAs(sealOf({ wrote: mario, sent: ana }))).toEqual([
      "Firmado por r8f1",
      mario.fingerprint,
      "Firma válida",
      "Subido por k7qp",
    ]);
    expect(box.dataset.verdict).toBe("holds");
    expect(box.hidden).toBe(false);
    drawnAs({ verdict: "unsigned" });
    expect(box.querySelector<HTMLElement>(".seal-print")?.hidden).toBe(true);
    expect(box.dataset.verdict).toBe("unsigned");
  });
});

describe("what the side says", () => {
  it("names who can open it", () => {
    expect(say(accessOf({ gated: false }))).toBe("Solo con el link");
    expect(say(accessOf({ gated: true }))).toBe("Con el link y una clave");
  });

  it("gives the day it ends, which a reading pushes back", () => {
    expect(endsOf({ ends: NOW + 30 * DAY, gated: false }, "es", say)).toBe(
      "29 de noviembre de 2026, si nadie lo abre antes",
    );
    expect(endsOf({ gated: false }, "es", say)).toBe("");
  });
});

describe("the downloaded file", () => {
  it("is named after the title, without what some systems forbid", () => {
    expect(fileOf("Docker: guía/rápida?", "Documento")).toBe("Docker guíarápida.md");
    expect(fileOf("  ", "Documento compartido")).toBe("Documento compartido.md");
  });
});

describe("opening it in Tisty", () => {
  it("hands Tisty the whole link, key and all", () => {
    expect(tistyOf("https://tisty.cl/k7Qm2xPa#clave+/=")).toBe(
      "tisty://share?link=https%3A%2F%2Ftisty.cl%2Fk7Qm2xPa%23clave%2B%2F%3D",
    );
  });
});

describe("the tags", () => {
  it("are shown above the body, and the row hides when there are none", () => {
    const row = document.createElement("p");
    tagged(row, ["guía", "docker"]);
    expect([...row.children].map((one) => one.textContent)).toEqual(["#guía", "#docker"]);
    expect(row.hidden).toBe(false);
    tagged(row, []);
    expect(row.children).toHaveLength(0);
    expect(row.hidden).toBe(true);
  });
});
