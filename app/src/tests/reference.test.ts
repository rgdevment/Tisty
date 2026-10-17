import { describe, expect, it } from "vitest";
import { examplesIn, plain, shownBeside, topicsOf } from "../guide/reference";

const text = [
  "# The Tisty language",
  "",
  "Intro.",
  "",
  "## At a glance",
  "",
  "| a | b |",
  "",
  "## Pens",
  "",
  "````markdown",
  "```markdown",
  "## not a topic",
  "```",
  "````",
  "",
  "## Párrafos y saltos",
  "",
  "Text.",
].join("\n");

describe("the language reference read as a guide", () => {
  it("starts with the intro and the glance, then one topic per heading", () => {
    const found = topicsOf(text);
    expect(found.map((one) => one.slug)).toEqual(["index", "pens", "parrafos-y-saltos"]);
    expect(found[0].heading).toBeNull();
    expect(found[0].body).toContain("## At a glance");
    expect(found[1].body).toContain("## not a topic");
    expect(found[2].heading).toBe("Párrafos y saltos");
  });

  it("names a topic by its heading without accents or marks", () => {
    expect(plain("Diagramas, fórmulas y widgets")).toBe("diagramas-formulas-y-widgets");
    expect(plain("  ¿Qué?  ")).toBe("que");
  });

  it("puts every markdown example beside what Tisty draws of it", () => {
    const page = new DOMParser().parseFromString(
      '<article><figure class="code"><pre><code class="language-markdown">==ojo==\n</code></pre></figure><figure class="code"><pre><code class="language-rust">fn x() {}</code></pre></figure></article>',
      "text/html",
    );
    const examples = examplesIn(page);
    expect(examples).toHaveLength(1);
    shownBeside(examples[0], "<p><mark>ojo</mark></p>", {
      written: "Escribes",
      drawn: "Tisty muestra",
    });
    const figure = page.querySelector("figure.example");
    expect(figure?.querySelector(".written")?.getAttribute("data-said")).toBe("Escribes");
    expect(figure?.querySelector(".written code")?.textContent).toBe("==ojo==");
    expect(figure?.querySelector(".drawn mark")?.textContent).toBe("ojo");
    expect(page.querySelectorAll("figure.code")).toHaveLength(1);
  });
});
