import { describe, expect, it } from "vitest";
import { docLink, docOf, dressed } from "../markdown";

describe("a note the core drew, readied for the window", () => {
  it("drops the places in the source, which only the page needs", () => {
    const html = dressed('<p data-at="0-19"><strong>firm</strong> and <em>soft</em></p>');
    expect(html).toBe("<p><strong>firm</strong> and <em>soft</em></p>");
  });

  it("asks for what lives under the data root by name, and leaves the address empty", () => {
    const html = dressed(
      '<p data-at="0-32"><img src="attachments/ab/cd.png" alt="shot" loading="lazy"></p>',
    );
    expect(html).toContain('data-inside="attachments/ab/cd.png"');
    expect(html).toContain('src=""');
  });

  it("leaves an address that leaves the machine alone", () => {
    const html = dressed('<p><a href="https://jira.example/browse/OPS-3465">OPS-3465</a></p>');
    expect(html).not.toContain("data-inside");
    expect(html).toContain('href="https://jira.example/browse/OPS-3465"');
    expect(
      dressed('<p><img src="https://evil.example/p.png" alt="x" loading="lazy"></p>'),
    ).toContain('src="https://evil.example/p.png"');
  });

  it("leaves an absolute path alone, whatever the platform writes", () => {
    for (const at of [
      "C:/Users/Mario/clip.mkv",
      "/home/mario/clip.mkv",
      "//server/share/clip.mkv",
    ]) {
      expect(dressed(`<p><a href="${at}">clip</a></p>`)).not.toContain("data-inside");
    }
  });

  it("finds a bare address without being asked", () => {
    const html = dressed('<p data-at="0-23">see https://x.example/1.</p>');
    expect(html).toContain('<a href="https://x.example/1">https://x.example/1</a>.');
    expect(dressed("<p>at www.x.example</p>")).toContain('href="https://www.x.example"');
  });

  it("does not make a link out of an address already in one, or in code", () => {
    expect(dressed('<p><a href="https://a.b">https://a.b</a></p>').match(/<a /g)).toHaveLength(1);
    expect(dressed("<p><code>https://a.b</code></p>")).not.toContain("<a ");
  });

  it("turns a card for a document into a chip the view resolves", () => {
    const html = dressed('<p><a class="card" data-doc="mac0-0007">el informe</a></p>');
    expect(html).toContain('data-inside="tisty:doc/mac0-0007"');
    expect(html).toContain('alt="el informe"');
    expect(html).not.toContain("card");
  });

  it("dresses a link to a document apart from one that leaves the app", () => {
    const html = dressed('<p><a href="tisty:doc/mac0-0001">Informe</a></p>');
    expect(html).toContain('class="paper"');
    expect(dressed('<p><a href="https://ejemplo.org">Fuera</a></p>')).not.toContain("paper");
  });
});

describe("a link to a document, built with a title nobody chose carefully", () => {
  it("escapes the brackets and the backslash a title carries", () => {
    expect(docLink("mac0-0001", "Informe [ borrador")).toBe(
      "[Informe [ borrador](tisty:doc/mac0-0001)",
    );
    expect(docLink("mac0-0001", "a\b")).toBe("[a\\b](tisty:doc/mac0-0001)");
  });

  it("does not need to escape the parentheses a title carries", () => {
    expect(docLink("mac0-0001", "Nota (borrador)")).toBe("[Nota (borrador)](tisty:doc/mac0-0001)");
  });
});

describe("what docOf makes of an address", () => {
  it("says no to anything that is not one of ours", () => {
    expect(docOf("")).toBeNull();
    expect(docOf("tisty:do")).toBeNull();
    expect(docOf("https://tisty:doc/mac0-0001")).toBeNull();
  });

  it("reads an id with nothing after the scheme as an empty one, not a missing one", () => {
    expect(docOf("tisty:doc/")).toBe("");
  });

  it("takes whatever comes after the scheme, slashes included", () => {
    expect(docOf("tisty:doc/a/b")).toBe("a/b");
  });
});
