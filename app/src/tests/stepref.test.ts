import { describe, expect, it } from "vitest";
import { composed as render } from "../markdown";

describe("referring to a step from the journal", () => {
  it("gives a step reference its own mark", () => {
    const html = render("para arreglar [[#4]] ejecuté el pipeline");

    expect(html).toContain('class="ref step"');
    expect(html).toContain('data-step="4"');
    expect(html).toContain("#4");
  });

  it("leaves an ordinary reference alone", () => {
    const html = render("ver [[el informe]]");

    expect(html).toContain('class="ref"');
    expect(html).not.toContain("step");
  });

  it.each(["[[#]]", "[[#abc]]", "[[#4x]]", "[[# 4]]", "[[#1234]]"])(
    "does not take %s for a step",
    (said) => {
      expect(render(said)).not.toContain("ref step");
    },
  );

  it("survives a step number nobody wrote a step for", () => {
    expect(() => render("[[#99]]")).not.toThrow();
  });

  it("does not let a reference smuggle markup in", () => {
    const html = render("[[<img src=x onerror=alert(1)>]]");

    expect(html).not.toContain("<img");
  });
});
