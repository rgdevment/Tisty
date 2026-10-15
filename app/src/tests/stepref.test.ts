import { describe, expect, it } from "vitest";
import { dressed } from "../markdown";

const ref = (name: string) =>
  `<p>para arreglar <span class="ref" data-ref="${name}">${name}</span></p>`;

describe("referring to a step from the journal", () => {
  it("gives a step reference its own mark and calls it by what the step says", () => {
    const html = dressed(ref("#2"), ["abrir", "ejecutar el pipeline"]);

    expect(html).toContain('class="ref step"');
    expect(html).toContain('data-step="2"');
    expect(html).toContain(">ejecutar el pipeline<");
  });

  it("leaves an ordinary reference alone", () => {
    const html = dressed(ref("el informe"));

    expect(html).toContain('class="ref"');
    expect(html).not.toContain("step");
  });

  it.each(["#", "#abc", "#4x", "# 4", "#1234"])("does not take %s for a step", (said) => {
    expect(dressed(ref(said))).not.toContain("ref step");
  });

  it("survives a step number nobody wrote a step for", () => {
    expect(dressed(ref("#99"))).toContain(">#99<");
  });

  it("does not let a step's words smuggle markup in", () => {
    const html = dressed(ref("#1"), ["<img src=x onerror=alert(1)>"]);

    expect(html).not.toContain("<img");
  });
});
