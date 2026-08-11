import { describe, expect, it } from "vitest";
import { composed as render } from "../markdown";

describe("referring to a step from the journal", () => {
  /// «para arreglar #4 ejecuté el pipeline»: a step is a number in the same
  /// panel, so it gets its own tint instead of reading as one more document.
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

  /// Anything that is not a plain number is a name, not a step.
  it.each(["[[#]]", "[[#abc]]", "[[#4x]]", "[[# 4]]", "[[#1234]]"])(
    "does not take %s for a step",
    (said) => {
      expect(render(said)).not.toContain("ref step");
    },
  );

  it("survives a step number nobody wrote a step for", () => {
    expect(() => render("[[#99]]")).not.toThrow();
  });

  /// The renderer escapes rather than trusting: this is prose someone typed.
  it("does not let a reference smuggle markup in", () => {
    const html = render("[[<img src=x onerror=alert(1)>]]");

    expect(html).not.toContain("<img");
  });
});
